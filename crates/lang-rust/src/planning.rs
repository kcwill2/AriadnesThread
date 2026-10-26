//! Rust planning surface (rust-language R-9b, handoff item 1).
//!
//! The ADR-0002 Decision 3 hook pair - the aggregation-file filter and the
//! extraction-target naming - lives here in the plugin crate, never in the
//! language-neutral cluster engine (the lang-scala `planning.rs` split
//! mirrored; shape sec. 3.5 records the shape).
//!
//! Aggregation rule (shape sec. 3.5): **a file is an aggregation unit iff
//! its content parses (syn) AND declares >= 1 NON-INLINE `mod x;` item** -
//! the disk-level aggregation edge. Inline `mod x { .. }` blocks are NOT
//! aggregation children (their body is in the same file). Unreadable or
//! unparseable input answers `Some(false)` - the filter invents nothing
//! (Scala parity).
//!
//! Path contract (mirrored from the Scala module docs): the cluster planner
//! feeds the hook the repo-relative `IncludeGraph::file_paths` values, so
//! [`aggregation_filter`] reads `repo_root.join(path)`; an already-absolute
//! path is used as given (tolerance, not the planner's contract).
//!
//! Target naming: `{sanitized_symbol}.rs` beside the aggregation file. A
//! qualified symbol id (`crate::models::models_marker`) loses its module
//! qualifier first - the target lands beside the aggregation file, so the
//! qualifier would only restate directories the target already sits in -
//! and the remaining simple name goes through
//! [`crate::mod_tree::sanitize_module_stem`], the SAME single source of
//! truth the move-retention hook anchors its `mod <stem>;` insertions to
//! (one grammar: `fixture_marker` -> `fixture_marker.rs` AND
//! `mod fixture_marker;`).

use crate::mod_tree::sanitize_module_stem;
use std::path::{Path, PathBuf};

/// The planning hook: is `path` (repo-relative against `repo_root`) an
/// aggregation unit? `Some(true)` iff the file parses and declares at least
/// one out-of-line `mod x;` item; unreadable/unparseable -> `Some(false)`.
/// The answer is never `None` - `None` would mean "planning unsupported",
/// which the trait-default hooks answer; this hop enrolls Rust (INV-5: the
/// planning bit ships with these hooks).
pub fn aggregation_filter(repo_root: &Path, path: &Path) -> Option<bool> {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    let Ok(source) = std::fs::read_to_string(resolved) else {
        return Some(false);
    };
    Some(declares_out_of_line_mod(&source))
}

/// True when `source` parses (syn) and declares a top-level out-of-line
/// `mod x;` item. An inline `mod x { .. }` carries `content == Some(..)`
/// and never fires; unparseable bytes are honestly "not an aggregation
/// unit", never an invented unit and never a panic.
pub fn declares_out_of_line_mod(source: &str) -> bool {
    syn::parse_file(source)
        .map(|file| file.items.iter().any(is_out_of_line_mod_item))
        .unwrap_or(false)
}

fn is_out_of_line_mod_item(item: &syn::Item) -> bool {
    matches!(item, syn::Item::Mod(module) if module.content.is_none())
}

/// The Rust extraction target: `{sanitized_symbol}.rs` alongside the
/// aggregation file (shape sec. 3.5). Never `None` once enrolled; the
/// qualifier strip mirrors `scala_target_path_for`'s simple-name
/// discipline at the Rust `::` separator.
pub fn target_path_for(aggregation_path: &Path, symbol_name: &str) -> Option<PathBuf> {
    let simple_name = symbol_name.rsplit("::").next().unwrap_or(symbol_name);
    Some(
        aggregation_path
            .with_file_name(format!("{}.rs", sanitize_module_stem(simple_name))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn written_tree(root: &Path, files: &[(&str, &str)]) {
        for (relative, content) in files {
            let full = root.join(relative);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, content).unwrap();
        }
    }

    #[test]
    fn aggregation_filter_detects_out_of_line_mod_declarations() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        written_tree(
            root,
            &[
                ("src/lib.rs", "mod greeting;\nmod models;\n\npub fn run() {}\n"),
                ("src/models/mod.rs", "pub mod user;\n"),
                ("src/inlined.rs", "mod inlined {\n    pub fn buried() {}\n}\n"),
                ("src/leaf.rs", "pub fn helper() {}\n"),
                ("src/broken.rs", "fn unclosed( {\n"),
            ],
        );
        assert_eq!(
            aggregation_filter(root, Path::new("src/lib.rs")),
            Some(true),
            "out-of-line `mod x;` declarations ARE the disk-level aggregation edges"
        );
        assert_eq!(
            aggregation_filter(root, Path::new("src/models/mod.rs")),
            Some(true),
            "`pub mod user;` is an out-of-line declaration too (visibility is not the rule)"
        );
        assert_eq!(
            aggregation_filter(root, Path::new("src/inlined.rs")),
            Some(false),
            "an inline `mod x {{ .. }}` block declares no aggregation child (its body is in this file)"
        );
        assert_eq!(
            aggregation_filter(root, Path::new("src/leaf.rs")),
            Some(false),
            "a mod-free file gets no aggregation - the planner invents no fake header"
        );
        assert_eq!(
            aggregation_filter(root, Path::new("src/broken.rs")),
            Some(false),
            "unparseable bytes answer the honest Some(false), never an invented unit"
        );
        assert_eq!(
            aggregation_filter(root, Path::new("src/absent.rs")),
            Some(false),
            "an unreadable path is not an aggregation unit; the filter invents nothing (Scala parity)"
        );
        assert_eq!(
            aggregation_filter(root, &root.join("src/lib.rs")),
            Some(true),
            "tolerance: an already-absolute path is used as given"
        );
    }

    #[test]
    fn target_path_lands_beside_the_aggregation_file() {
        assert_eq!(
            target_path_for(Path::new("src/lib.rs"), "fixture_marker"),
            Some(PathBuf::from("src/fixture_marker.rs")),
            "the simple anchor name the planner passes lands beside the aggregation file"
        );
        assert_eq!(
            target_path_for(Path::new("src/models/mod.rs"), "crate::models_marker"),
            Some(PathBuf::from("src/models/models_marker.rs")),
            "a flat qualified id loses its `crate::` qualifier - the target already sits in the aggregation directory"
        );
        assert_eq!(
            target_path_for(Path::new("src/models/mod.rs"), "crate::models::user"),
            Some(PathBuf::from("src/models/user.rs")),
            "a multi-segment id reduces to its simple name (scala_target_path_for discipline at the Rust separator)"
        );
    }

    /// The target grammar is the SHARED module-stem grammar (R-9a rides):
    /// the move-retention hook inserts `mod <stem>;` for exactly these
    /// stems, so file name and mod declaration can never drift.
    #[test]
    fn target_stem_grammar_is_the_shared_module_stem() {
        assert_eq!(
            target_path_for(Path::new("src/lib.rs"), ":::"),
            Some(PathBuf::from("src/symbol.rs")),
            "all-illegal names fall back to `symbol` - the sanitize_module_stem contract"
        );
        assert_eq!(
            target_path_for(Path::new("src/lib.rs"), "_x_"),
            Some(PathBuf::from("src/x.rs")),
            "edge underscores are trimmed"
        );
        assert_eq!(
            target_path_for(Path::new("src/lib.rs"), "2phase"),
            Some(PathBuf::from("src/_2phase.rs")),
            "the leading-digit guard prefixes `_` so the stem stays identifier-safe (module-stem form)"
        );
        assert_eq!(
            sanitize_module_stem("crate::greeting".rsplit("::").next().unwrap()),
            "greeting",
            "single source of truth: planning and move-retention read the same sanitizer"
        );
    }
}
