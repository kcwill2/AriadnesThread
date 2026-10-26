//! Machinery for the RISK-MODTREE consumer mechanism (rust-language R-8).
//!
//! Only the syntax-independent primitives the mechanism in
//! [`crate::dependency_rewriter`] needs: span-to-byte location (rewrites
//! splice into the ORIGINAL bytes - untouched bytes stay verbatim),
//! `use`-leaf reading, the bare mod-space spelling grammar, stem sanitizing
//! (the lang-cpp/Scala discipline, mirrored: private in those crates), and
//! the descending splice with Scala's full-line widening.

use std::path::Path;

/// Byte offset of the first byte of each line (line 1 => `offsets[0]`).
pub(crate) fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            offsets.push(index + 1);
        }
    }
    offsets
}

/// Resolve a `proc_macro2` position (1-based line, 0-based CHAR column) to a
/// byte offset. Columns count characters, offsets count bytes, so
/// multibyte lines stay exact; a column past the last character resolves to
/// the line end (the span-end convention).
pub(crate) fn resolve_offset(source: &str, line_starts: &[usize], line: usize, column_chars: usize) -> Option<usize> {
    if line == 0 || line > line_starts.len() {
        return None;
    }
    let line_start = line_starts[line - 1];
    let line_end = line_starts.get(line).copied().unwrap_or(source.len());
    let mut byte = line_start;
    for _ in 0..column_chars {
        let Some(character) = source[byte..line_end].chars().next() else { break };
        byte += character.len_utf8();
    }
    Some(byte)
}

/// Locate one parsed span back into the original bytes:
/// `(start_line, end_line, start_byte, end_byte)` with byte-exclusive end.
pub(crate) fn located_span(source: &str, line_starts: &[usize], span: proc_macro2::Span) -> Option<(usize, usize, usize, usize)> {
    let (start_line, end_line) = (span.start().line, span.end().line);
    let start_byte = resolve_offset(source, line_starts, start_line, span.start().column)?;
    let end_byte = resolve_offset(source, line_starts, end_line, span.end().column)?;
    (end_byte >= start_byte).then_some((start_line, end_line, start_byte, end_byte))
}

/// One leaf of a `use` statement as written: leaf-path spelling (leading
/// `::` kept), glob flag, rename alias - the `parser_uses` raw stance.
pub(crate) struct UseLeaf {
    pub spelling: String,
    pub star: bool,
    pub alias: Option<String>,
}

/// Read every leaf of one `use` statement in source order.
pub(crate) fn use_leaves(use_item: &syn::ItemUse) -> Vec<UseLeaf> {
    let root = if use_item.leading_colon.is_some() { "::" } else { "" };
    let mut leaves = Vec::new();
    collect_leaves(&use_item.tree, &mut Vec::new(), root, &mut leaves);
    leaves
}

fn collect_leaves(tree: &syn::UseTree, segments: &mut Vec<String>, root: &str, leaves: &mut Vec<UseLeaf>) {
    use syn::UseTree;
    let depth = segments.len();
    match tree {
        UseTree::Path(use_path) => {
            segments.push(use_path.ident.to_string());
            collect_leaves(&use_path.tree, segments, root, leaves);
        }
        UseTree::Name(use_name) => {
            segments.push(use_name.ident.to_string());
            leaves.push(UseLeaf { spelling: spell(root, segments), star: false, alias: None });
        }
        UseTree::Rename(use_rename) => {
            segments.push(use_rename.ident.to_string());
            let alias = use_rename.rename.to_string();
            leaves.push(UseLeaf { spelling: spell(root, segments), star: false, alias: Some(alias) });
        }
        UseTree::Glob(_) => {
            leaves.push(UseLeaf { spelling: spell(root, segments), star: true, alias: None });
        }
        UseTree::Group(use_group) => {
            for item in &use_group.items {
                collect_leaves(item, segments, root, leaves);
            }
        }
    }
    segments.truncate(depth);
}

fn spell(root: &str, segments: &[String]) -> String {
    format!("{root}{}", segments.join("::"))
}

/// True when a dependency spelling is a bare mod-space name (one ASCII
/// identifier segment): single-segment spellings drive the parent's
/// `mod <name>;` clause, every multi-segment spelling is `use`-space.
pub(crate) fn is_bare_mod_spelling(spelling: &str) -> bool {
    let mut characters = spelling.chars();
    match characters.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    spelling.chars().all(|character| character.is_ascii_alphanumeric() || character == '_')
}

/// Filesystem-and-identifier-safe module stem: the `sanitize_symbol_stem`
/// discipline lang-cpp invented and Scala mirrored (ASCII alphanumeric and
/// underscore survive, other runs collapse to one `_`, edge trims,
/// all-illegal falls back to `"symbol"`), plus a guard: a leading digit
/// gains a `_` prefix.
pub(crate) fn sanitize_module_stem(stem: &str) -> String {
    let mut kept = String::with_capacity(stem.len());
    let mut pending_underscore = false;
    for character in stem.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            if pending_underscore {
                kept.push('_');
                pending_underscore = false;
            }
            kept.push(character);
        } else if !kept.is_empty() {
            pending_underscore = true;
        }
    }
    let trimmed = kept.trim_matches('_').to_string();
    let sanitized = if trimmed.is_empty() { "symbol".to_string() } else { trimmed };
    if sanitized.chars().next().is_some_and(|first| first.is_ascii_digit()) {
        format!("_{sanitized}")
    } else {
        sanitized
    }
}

/// The honest Rust module path of an src-rooted file (amendment sec. 1a(1)):
/// - `src/lib.rs` / `src/main.rs` ARE the crate-root module: `Some("crate")`.
/// - a file named `mod.rs` IS its directory's module: `crate::<dirs>`.
/// - every other src-rooted path: `crate::<dirs>::<stem>` (byte-identical
///   to the R-8 rule).
/// `None` when the path carries no `src` directory before the file name
/// (Scala's conservative outside-the-source-root stance, unchanged). The
/// written stem rides verbatim here - it names analyzer ids; only
/// mod-space spellings get sanitized.
pub(crate) fn module_path_of_src_file(path: &Path) -> Option<String> {
    let parts: Vec<String> =
        path.components().map(|component| component.as_os_str().to_string_lossy().into_owned()).collect();
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let root = parts.iter().position(|part| part == "src")?;
    (root + 1 < parts.len()).then(|| module_spelling(&parts[root + 1..parts.len() - 1], &stem))
}

/// The spelling for one (directories, file-stem) pair inside a `src` root,
/// per the module-semantics rules on [`module_path_of_src_file`].
fn module_spelling(directories: &[String], stem: &str) -> String {
    if stem == "mod" {
        // mod.rs IS the directory's module; an empty directory chain is the
        // crate root, spelled `crate`.
        return if directories.is_empty() {
            "crate".to_string()
        } else {
            format!("crate::{}", directories.join("::"))
        };
    }
    let is_crate_root_file = directories.is_empty() && (stem == "lib" || stem == "main");
    if is_crate_root_file {
        return "crate".to_string();
    }
    format!("crate::{}{stem}", directories.join("::") + if directories.is_empty() { "" } else { "::" })
}

/// Whitespace prefix of the line holding byte `byte`.
pub(crate) fn line_indent_at(source: &str, byte: usize) -> String {
    let head = &source[..byte];
    let line_start = head.rfind('\n').map_or(0, |newline| newline + 1);
    let prefix = &head[line_start..];
    prefix.chars().all(char::is_whitespace).then(|| prefix.to_string()).unwrap_or_default()
}

/// One byte-range replacement against the ORIGINAL source.
pub(crate) struct SpliceEdit {
    pub from: usize,
    pub to: usize,
    pub replacement: String,
}

/// Splice edits descending by start so earlier indices stay valid. An empty
/// replacement removes the statement's whole line (leading whitespace and
/// the `\r?\n` terminator included) when nothing else lives on it - the
/// Scala applier's widening rule.
pub(crate) fn apply_splices(source: &str, mut edits: Vec<SpliceEdit>) -> String {
    let mut content = source.to_string();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.from));
    for edit in edits {
        let (from, to) = if edit.replacement.is_empty() {
            widen_to_full_line(&content, edit.from, edit.to)
        } else {
            (edit.from, edit.to)
        };
        content.replace_range(from..to, &edit.replacement);
    }
    content
}

fn widen_to_full_line(content: &str, from: usize, to: usize) -> (usize, usize) {
    let head = &content[..from];
    let line_start = head.rfind('\n').map_or(0, |newline| newline + 1);
    let tail = &content[to..];
    let line_end_relative = tail.find('\n').map_or(tail.len(), |newline| newline + 1);
    let alone = head[line_start..].chars().all(char::is_whitespace)
        && tail[..line_end_relative].chars().all(char::is_whitespace);
    alone.then(|| (line_start, to + line_end_relative)).unwrap_or((from, to))
}

#[cfg(test)]
mod tests {
    use super::module_path_of_src_file;
    use std::path::Path;

    /// R-9a F1 (amendment sec. 1b red 1): honest Rust module semantics for
    /// the two special forms; every other src-rooted path keeps the R-8
    /// spelling byte-identically (the green-at-birth legs below).
    #[test]
    fn module_path_pins_the_rust_module_semantics() {
        assert_eq!(
            module_path_of_src_file(Path::new("src/lib.rs")),
            Some("crate".to_string()),
            "src/lib.rs IS the crate-root module (the R-8 stem rule said `crate::lib`)"
        );
        assert_eq!(
            module_path_of_src_file(Path::new("src/main.rs")),
            Some("crate".to_string()),
            "src/main.rs is a crate-root file exactly like src/lib.rs"
        );
        assert_eq!(
            module_path_of_src_file(Path::new("src/models/mod.rs")),
            Some("crate::models".to_string()),
            "mod.rs IS its directory's module (the R-8 stem rule said `crate::models::mod`)"
        );
        // Unchanged forms (green-at-birth legs of this pin): the amendment's
        // byte-identity clause for every other shape.
        assert_eq!(
            module_path_of_src_file(Path::new("src/models/user.rs")),
            Some("crate::models::user".to_string()),
            "every other src-rooted path is byte-identical to the R-8 rule"
        );
        assert_eq!(
            module_path_of_src_file(Path::new("src/widgets.rs")),
            Some("crate::widgets".to_string()),
            "a src-rooted sibling keeps its own stem segment"
        );
        assert_eq!(
            module_path_of_src_file(Path::new("include/thing.h")),
            None,
            "outside a src dir the conservative None stance is unchanged"
        );
    }
}
