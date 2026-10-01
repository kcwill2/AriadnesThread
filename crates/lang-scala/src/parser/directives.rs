//! Package clauses and the import/export map (D16).
//!
//! This is the *parser's own* lexical import scan (`directive_parser.rs` is
//! frozen and belongs to the m20 resolver path). It records:
//! - `namespace`: consecutive leading `package <qualifier>` clauses joined
//!   with `.` (Scala semantics: `package a` + `package b` ⇒ `a.b`).
//! - `named_imports`: selector simple-name → qualified target, from
//!   `import a.b.C` and `import a.b.{C, D}` (and `export`, treated alike).
//! - `wildcard_base`: from `import a.b._` — at most one active wildcard
//!   attribution (D16): the first wildcard wins.
//! - `consumed_ranges`: `[start, end_exclusive)` token spans of package
//!   clauses (incl. gracefully-skipped `package x {` blocks and Scala-3
//!   `package x:` labels running to EOF) and import/export statements, so
//!   the declaration and occurrence passes do not read their tokens.
//!
//! Limitations (documented): selector renames (`{C => D}`) are consumed but
//! not mapped; `package object` starts are left to the declaration pass.

use crate::scanner::Token;
use std::collections::BTreeMap;

pub(crate) struct DirectiveScan {
    pub namespace: String,
    pub named_imports: BTreeMap<String, String>,
    pub wildcard_base: Option<String>,
    /// `[start, end_exclusive)` token ranges, sorted by start.
    pub consumed_ranges: Vec<(usize, usize)>,
}

pub(crate) fn scan_directives(tokens: &[Token]) -> DirectiveScan {
    let mut scan = DirectiveScan {
        namespace: String::new(),
        named_imports: BTreeMap::new(),
        wildcard_base: None,
        consumed_ranges: Vec::new(),
    };
    let mut index = 0usize;
    while index < tokens.len() {
        match tokens[index].ident() {
            Some("package") if !preceded_by_dot(tokens, index) => {
                index = scan_package_clause(tokens, index, &mut scan);
            }
            Some("import") | Some("export") if !preceded_by_dot(tokens, index) => {
                index = scan_directive_statement(tokens, index, &mut scan);
            }
            _ => index += 1,
        }
    }
    scan
}

fn preceded_by_dot(tokens: &[Token], index: usize) -> bool {
    index > 0 && tokens[index - 1].is_punct('.')
}

/// Consumes `package <qualifier>` (namespace update), `package x { … }`
/// (graceful skip of the whole block), and Scala-3 `package x:` (graceful
/// skip to EOF). A `package object` start is left for the declaration pass.
fn scan_package_clause(tokens: &[Token], start: usize, scan: &mut DirectiveScan) -> usize {
    let cursor = start + 1;
    let Some(first) = tokens.get(cursor).and_then(Token::ident) else {
        return start + 1;
    };
    if first == "object" {
        return start + 1;
    }
    let (segments, after_chain) = read_dotted_chain(tokens, cursor);
    if segments.is_empty() {
        return start + 1;
    }
    let qualifier = segments.join(".");
    match tokens.get(after_chain) {
        Some(token) if token.is_punct('{') => {
            let end = matching_brace_end(tokens, after_chain);
            scan.consumed_ranges.push((start, end));
            end
        }
        Some(token) if token.is_punct(':') => {
            scan.consumed_ranges.push((start, tokens.len()));
            tokens.len()
        }
        _ => {
            scan.namespace = match scan.namespace.is_empty() {
                true => qualifier,
                false => format!("{}.{}", scan.namespace, qualifier),
            };
            after_chain
        }
    }
}

/// Parses one `import`/`export` statement at `start`, recording selectors
/// into `scan`; returns the index after the statement (or `start + 1` if the
/// statement is malformed).
fn scan_directive_statement(tokens: &[Token], start: usize, scan: &mut DirectiveScan) -> usize {
    let mut cursor = start + 1;
    loop {
        let (segments, after_chain) = match read_dotted_chain(tokens, cursor) {
            (segments, next) if !segments.is_empty() => (segments, next),
            _ => break,
        };
        cursor = after_chain;
        cursor = consume_selector_suffix(tokens, &segments, cursor, scan);
        if tokens.get(cursor).is_some_and(|t| t.is_punct(',')) {
            cursor += 1;
        } else {
            break;
        }
    }
    if cursor > start + 1 {
        scan.consumed_ranges.push((start, cursor));
    }
    cursor.max(start + 1)
}

/// After a dotted chain in an import: `_` wildcard, `{C, D}` grouped
/// selectors, or the chain's own last segment as a plain selector.
fn consume_selector_suffix(
    tokens: &[Token],
    segments: &[String],
    cursor: usize,
    scan: &mut DirectiveScan,
) -> usize {
    let base = segments.join(".");
    let Some(dot) = tokens.get(cursor) else {
        register_plain_selector(segments, scan);
        return cursor;
    };
    if !dot.is_punct('.') {
        register_plain_selector(segments, scan);
        return cursor;
    }
    match tokens.get(cursor + 1) {
        Some(next) if next.ident() == Some("_") => {
            if scan.wildcard_base.is_none() {
                scan.wildcard_base = Some(base);
            }
            cursor + 2
        }
        Some(next) if next.is_punct('{') => consume_grouped_selectors(tokens, &base, cursor + 1, scan),
        _ => {
            register_plain_selector(segments, scan);
            cursor
        }
    }
}

fn consume_grouped_selectors(
    tokens: &[Token],
    base: &str,
    brace_index: usize,
    scan: &mut DirectiveScan,
) -> usize {
    let mut cursor = brace_index + 1;
    loop {
        while tokens.get(cursor).is_some_and(|t| t.is_punct(',')) {
            cursor += 1;
        }
        let Some(token) = tokens.get(cursor) else {
            return tokens.len();
        };
        if token.is_punct('}') {
            return cursor + 1;
        }
        if token.ident() == Some("_") {
            if scan.wildcard_base.is_none() {
                scan.wildcard_base = Some(base.to_string());
            }
            cursor += 1;
            continue;
        }
        match token.ident() {
            Some(word) => {
                scan.named_imports
                    .insert(word.to_string(), format!("{base}.{word}"));
                cursor += 1;
                cursor = skip_rename_suffix(tokens, cursor);
            }
            _ => return cursor,
        }
    }
}

/// Consumes a `{C => D}` rename tail (the renamed name is not mapped:
/// documented limitation) and returns the index after it.
fn skip_rename_suffix(tokens: &[Token], cursor: usize) -> usize {
    if tokens.get(cursor).is_some_and(|t| t.is_punct('='))
        && tokens.get(cursor + 1).is_some_and(|t| t.is_punct('>'))
    {
        let after_arrow = cursor + 2;
        if tokens.get(after_arrow).and_then(Token::ident).is_some() {
            return after_arrow + 1;
        }
        return after_arrow;
    }
    cursor
}

fn register_plain_selector(segments: &[String], scan: &mut DirectiveScan) {
    if let Some(last) = segments.last() {
        scan.named_imports
            .insert(last.clone(), segments.join("."));
    }
}

/// Reads `ident (. ident)*` starting at `start`; returns the segments and
/// the index after the chain.
pub(crate) fn read_dotted_chain(tokens: &[Token], start: usize) -> (Vec<String>, usize) {
    let mut segments: Vec<String> = Vec::new();
    let mut cursor = start;
    while let Some(word) = tokens.get(cursor).and_then(Token::ident) {
        segments.push(word.to_string());
        cursor += 1;
        if tokens.get(cursor).is_some_and(|t| t.is_punct('.'))
            && tokens
                .get(cursor + 1)
                .and_then(Token::ident)
                .is_some_and(|next| !is_selector_stop_word(next))
        {
            cursor += 1;
        } else {
            break;
        }
    }
    (segments, cursor)
}

fn is_selector_stop_word(word: &str) -> bool {
    word == "_"
}

/// Index after the `}` that matches the `{` at `open_index` (brace depth
/// over scanner tokens, which already hide comment/string braces).
pub(crate) fn matching_brace_end(tokens: &[Token], open_index: usize) -> usize {
    let mut depth = 0i64;
    let mut cursor = open_index;
    while cursor < tokens.len() {
        if tokens[cursor].is_punct('{') {
            depth += 1;
        } else if tokens[cursor].is_punct('}') {
            depth -= 1;
            if depth == 0 {
                return cursor + 1;
            }
        }
        cursor += 1;
    }
    tokens.len()
}

/// The `end_exclusive` of the consumed range containing `token_index`.
pub(crate) fn consumed_end(consumed_ranges: &[(usize, usize)], token_index: usize) -> Option<usize> {
    consumed_ranges
        .iter()
        .find(|(start, end_exclusive)| *start <= token_index && token_index < *end_exclusive)
        .map(|(_, end_exclusive)| *end_exclusive)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::tokenize;

    fn scan(source: &str) -> DirectiveScan {
        scan_directives(&tokenize(source))
    }

    #[test]
    fn leading_package_clauses_set_and_join_the_namespace() {
        assert_eq!(scan("package com.example\n").namespace, "com.example");
        assert_eq!(
            scan("package a\npackage b\n").namespace,
            "a.b",
            "consecutive package clauses nest per Scala semantics"
        );
    }

    #[test]
    fn named_and_grouped_imports_populate_the_selector_map() {
        let single = scan("import a.b.C\n");
        assert_eq!(single.named_imports.get("C").map(String::as_str), Some("a.b.C"));

        let grouped = scan("import a.b.{C, D}\n");
        assert_eq!(
            grouped.named_imports.get("C").map(String::as_str),
            Some("a.b.C")
        );
        assert_eq!(
            grouped.named_imports.get("D").map(String::as_str),
            Some("a.b.D")
        );
    }

    #[test]
    fn wildcard_records_only_the_first_base() {
        let scan_result = scan("import scala.collection.mutable._\nimport other.wild._\n");
        assert_eq!(
            scan_result.wildcard_base.as_deref(),
            Some("scala.collection.mutable"),
            "at most one active wildcard attribution (D16)"
        );
    }

    #[test]
    fn export_is_treated_like_import() {
        let exported = scan("export a.b.Helper\n");
        assert_eq!(
            exported.named_imports.get("Helper").map(String::as_str),
            Some("a.b.Helper")
        );
    }

    #[test]
    fn package_block_and_label_are_consumed_whole() {
        let block = scan("package outer {\nclass Hidden\n}\nclass Real\n");
        assert_eq!(block.namespace, "", "braced package is a documented limitation");
        let (start, end) = block.consumed_ranges[0];
        assert_eq!(start, 0);
        assert!(end >= 5, "the whole block up to the matching brace is consumed");

        let label = scan("package outer:\nclass Hidden\n");
        assert_eq!(label.consumed_ranges[0], (0, tokenize("package outer:\nclass Hidden\n").len()));
    }

    #[test]
    fn package_object_start_is_left_for_the_declaration_pass() {
        let scan_result = scan("package com.example\npackage object models {}\n");
        assert_eq!(scan_result.namespace, "com.example");
        assert!(
            scan_result
                .consumed_ranges
                .iter()
                .all(|(_, end)| *end <= 4),
            "the `package object` tokens must not be swallowed by the clause scan"
        );
    }

    #[test]
    fn dotted_package_accessor_is_not_a_package_clause() {
        let scan_result = scan("package a\n\nval q = x.package\nval r = 1\n");
        assert_eq!(
            scan_result.namespace, "a",
            "Gate-A note 6: `x.package` member access must not read `val` as a \
             package qualifier; the same dot-guard import/export already have"
        );
        assert!(
            scan_result.consumed_ranges.iter().all(|(_start, end)| *end <= 4),
            "only the leading `package a` clause may be consumed"
        );
    }

    #[test]
    fn consumed_end_covers_ranges_exclusively() {
        let ranges = [(1usize, 3usize)];
        assert_eq!(consumed_end(&ranges, 1), Some(3));
        assert_eq!(consumed_end(&ranges, 2), Some(3));
        assert_eq!(consumed_end(&ranges, 3), None);
    }
}
