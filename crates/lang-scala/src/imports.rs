//! Import-statement scanning and the D-M23-13 `referenced_imports` producer.
//!
//! [`scan_imports`] walks the M21 scanner token stream (comments and string
//! contents vanish there, so a commented or quoted `import` is never seen
//! here) and records every top-level `import`/`export` statement with its
//! token span and its selectors: plain chains, grouped members (incl.
//! renames, kept verbatim for rewrite replay), and wildcards — ALL of them,
//! unlike the analysis-time [`crate::parser`] directive scan which keeps at
//! most one wildcard (D16) and no selector positions.
//!
//! [`referenced_imports`] is the pure producer seam D-M23-13(b) defines for
//! T7's `unit_imports`: module-path import strings for a moved region,
//! selected against the region bodies. Locked rules:
//! - named/grouped selectors are kept only when the selector's simple name
//!   occurs as an IDENTIFIER token in the concatenated bodies (an occurrence
//!   inside a string literal or comment is not an identifier token, so it
//!   never counts);
//! - renames (`{C => D}`) match on the SELECTED (left) name and emit the
//!   ORIGINAL module path `base.C` — the frozen m20 resolver grammar's
//!   choice, kept so the resolver and this producer cannot disagree;
//! - wildcards are always kept (never provably unreferenced), spelled
//!   `base._`;
//! - output is deduped and sorted module-path strings.

use crate::parser::{preceded_by_dot, read_dotted_chain};
use crate::scanner::{tokenize, Token};

/// One selector inside an import statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SelectorForm {
    /// `import a.b.C` — a plain dotted chain.
    Plain { module_path: String, simple_name: String },
    /// A member of `import a.b.{ … }`: `member_chain` is the selected (left
    /// of any `=>`) dotted chain, `simple_name` its last segment.
    Grouped { base: String, member_chain: String, simple_name: String },
    /// `import a.b._` or a `_` member of a group.
    Wildcard { base: String },
}

impl SelectorForm {
    /// The qualified module path this selector imports (`a.b.C`), `None` for
    /// a wildcard (the package is carried in the variant).
    pub(crate) fn module_path(&self) -> Option<String> {
        match self {
            SelectorForm::Plain { module_path, .. } => Some(module_path.clone()),
            SelectorForm::Grouped { base, member_chain, .. } => {
                Some(format!("{base}.{member_chain}"))
            }
            SelectorForm::Wildcard { .. } => None,
        }
    }

    /// The name a body reference is matched against; `None` for wildcards,
    /// which are never proven unreferenced.
    pub(crate) fn reference_name(&self) -> Option<&str> {
        match self {
            SelectorForm::Plain { simple_name, .. } | SelectorForm::Grouped { simple_name, .. } => {
                Some(simple_name)
            }
            SelectorForm::Wildcard { .. } => None,
        }
    }
}

/// One selector inside a statement, with the token span of its verbatim text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedSelector {
    pub form: SelectorForm,
    pub tok_start: usize,
    pub tok_end: usize,
}

/// One `import`/`export` statement: keyword token index through one-past its
/// last token, with its selectors in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportStatement {
    pub keyword: String,
    pub start_tok: usize,
    pub end_tok: usize,
    pub selectors: Vec<ParsedSelector>,
}

/// Tokenized source with its import statements.
pub(crate) struct ImportPass {
    pub tokens: Vec<Token>,
    pub statements: Vec<ImportStatement>,
}

impl ImportPass {
    /// Verbatim source text of token span `[from_tok, to_tok)` as char
    /// offsets into `source`.
    pub(crate) fn span_chars(&self, from_tok: usize, to_tok: usize) -> (usize, usize) {
        (self.tokens[from_tok].start, self.tokens[to_tok - 1].end)
    }

    /// Verbatim text of one selector.
    pub(crate) fn selector_text(&self, source: &str, selector: &ParsedSelector) -> String {
        self.text(source, selector.tok_start, selector.tok_end)
    }

    /// Verbatim text of one whole statement (keyword through last token).
    pub(crate) fn statement_text(&self, source: &str, statement: &ImportStatement) -> String {
        self.text(source, statement.start_tok, statement.end_tok)
    }

    fn text(&self, source: &str, from_tok: usize, to_tok: usize) -> String {
        let (from, to) = self.span_chars(from_tok, to_tok);
        source.chars().skip(from).take(to - from).collect()
    }
}

/// Char-offset → byte-offset map of `source` (`len + 1` entries, the last
/// pointing one past the final byte) — token offsets are char indices and
/// every splice here is byte-indexed.
pub(crate) fn char_to_byte_map(source: &str) -> Vec<usize> {
    let mut map: Vec<usize> = source.char_indices().map(|(byte, _)| byte).collect();
    map.push(source.len());
    map
}

/// Scan `source` for import/export statements over the M21 scanner stream.
pub(crate) fn scan_imports(source: &str) -> ImportPass {
    let tokens = tokenize(source);
    let mut statements = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let is_directive = matches!(tokens[index].ident(), Some("import") | Some("export"))
            && !preceded_by_dot(&tokens, index);
        if is_directive {
            let statement = parse_statement(&tokens, index);
            index = statement.end_tok.max(index + 1);
            statements.push(statement);
        } else {
            index += 1;
        }
    }
    ImportPass { tokens, statements }
}

/// Parse one statement whose keyword sits at `start`; a malformed tail ends
/// the statement early but never panics (scanner-total discipline).
fn parse_statement(tokens: &[Token], start: usize) -> ImportStatement {
    let mut statement = ImportStatement {
        keyword: tokens[start].ident().unwrap_or_default().to_string(),
        start_tok: start,
        end_tok: start + 1,
        selectors: Vec::new(),
    };
    let mut cursor = start + 1;
    loop {
        let (segments, after_chain) = read_dotted_chain(tokens, cursor);
        if segments.is_empty() {
            break;
        }
        cursor = consume_selector_suffix(tokens, &segments, cursor, after_chain, &mut statement);
        if tokens.get(cursor).is_some_and(|token| token.is_punct(',')) {
            cursor += 1;
        } else {
            break;
        }
    }
    statement.end_tok = cursor.max(start + 1);
    statement
}

/// After a dotted chain: `._` wildcard, `.{ … }` group, or the chain itself
/// as a plain selector; returns the cursor after the selector.
fn consume_selector_suffix(
    tokens: &[Token],
    segments: &[String],
    chain_start: usize,
    after_chain: usize,
    statement: &mut ImportStatement,
) -> usize {
    let base = segments.join(".");
    if tokens.get(after_chain).is_some_and(|token| token.is_punct('.')) {
        match tokens.get(after_chain + 1) {
            Some(next) if next.ident() == Some("_") => {
                push(statement, SelectorForm::Wildcard { base }, chain_start, after_chain + 2);
                return after_chain + 2;
            }
            Some(next) if next.is_punct('{') => {
                return consume_group(tokens, &base, after_chain + 1, statement);
            }
            _ => {}
        }
    }
    push(
        statement,
        SelectorForm::Plain {
            module_path: base.clone(),
            simple_name: segments.last().cloned().unwrap_or_default(),
        },
        chain_start,
        after_chain,
    );
    after_chain
}

fn push(statement: &mut ImportStatement, form: SelectorForm, tok_start: usize, tok_end: usize) {
    if tok_end > tok_start {
        statement
            .selectors
            .push(ParsedSelector { form, tok_start, tok_end });
    }
}

/// Members of `{ … }`: `_` (wildcard member) or a dotted member with an
/// optional `=> alias` tail, each kept verbatim by token span.
fn consume_group(
    tokens: &[Token],
    base: &str,
    brace_index: usize,
    statement: &mut ImportStatement,
) -> usize {
    let mut cursor = brace_index + 1;
    loop {
        while tokens.get(cursor).is_some_and(|token| token.is_punct(',')) {
            cursor += 1;
        }
        let Some(token) = tokens.get(cursor) else {
            return tokens.len();
        };
        if token.is_punct('}') {
            return cursor + 1;
        }
        if token.ident() == Some("_") {
            let mut end = cursor + 1;
            if tokens.get(end).is_some_and(|t| t.is_punct('*')) {
                end += 1;
            }
            push(statement, SelectorForm::Wildcard { base: base.to_string() }, cursor, end);
            cursor = end;
            continue;
        }
        let member_start = cursor;
        let (segments, after_chain) = read_dotted_chain(tokens, cursor);
        if segments.is_empty() {
            return after_chain.max(member_start + 1);
        }
        cursor = skip_rename(tokens, after_chain);
        let member_chain = segments.join(".");
        let selector = ParsedSelector {
            form: SelectorForm::Grouped {
                base: base.to_string(),
                simple_name: last_segment(&member_chain).to_string(),
                member_chain,
            },
            tok_start: member_start,
            tok_end: cursor.max(member_start + 1),
        };
        statement.selectors.push(selector);
    }
}

/// Consumes `=> alias` (or a `=>` with a missing tail) after a member.
fn skip_rename(tokens: &[Token], cursor: usize) -> usize {
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

/// A grouped member's reference name is its first chain's LAST segment —
/// `import a.b.{c.D}` binds `D` locally, matching the frozen grammar's
/// dotted-identifier selector handling.
fn last_segment(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// D-M23-13(b): module-path import strings the bodies still reference.
pub fn referenced_imports(source_text: &str, body_texts: &[&str]) -> Vec<String> {
    let pass = scan_imports(source_text);
    let referenced: std::collections::BTreeSet<String> = body_texts
        .iter()
        .flat_map(|body| tokenize(body))
        .filter_map(|token| token.ident().map(str::to_string))
        .collect();
    let mut kept = std::collections::BTreeSet::new();
    for statement in &pass.statements {
        for selector in &statement.selectors {
            match &selector.form {
                SelectorForm::Wildcard { base } => {
                    kept.insert(format!("{base}._"));
                }
                _ => {
                    let name = selector.form.reference_name().unwrap_or_default();
                    if referenced.contains(name) {
                        if let Some(path) = selector.form.module_path() {
                            kept.insert(path);
                        }
                    }
                }
            }
        }
    }
    kept.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refs(source: &str, bodies: &[&str]) -> Vec<String> {
        referenced_imports(source, bodies)
    }

    #[test]
    fn named_import_kept_when_selector_identifier_occurs_in_bodies() {
        assert_eq!(
            refs("import a.b.Widget\n", &["class Use {\n  val w: Widget = null\n}"]),
            vec!["a.b.Widget".to_string()]
        );
    }

    #[test]
    fn named_import_dropped_when_selector_unreferenced() {
        assert!(
            refs("import a.b.Widget\nimport a.b.Gadget\n", &["class Use {\n  val w: Widget = null\n}"])
                == vec!["a.b.Widget".to_string()],
            "Gadget is never named in the bodies"
        );
    }

    #[test]
    fn grouped_selectors_split_and_kept_per_selector() {
        assert_eq!(
            refs("import a.b.{Alpha, Beta}\n", &["def f(x: Alpha): Unit = ???"]),
            vec!["a.b.Alpha".to_string()],
            "grouped statements split per selector: only the referenced member survives"
        );
        assert_eq!(
            refs("import a.b.{Alpha, Beta}\n", &["def f(x: Alpha, y: Beta): Unit = ???"]),
            vec!["a.b.Alpha".to_string(), "a.b.Beta".to_string()]
        );
    }

    #[test]
    fn wildcard_imports_always_kept_spelled_base_underscore() {
        assert_eq!(
            refs("import scala.collection.mutable._\nimport x.y.{A, _}\n", &["class Empty"]),
            vec![
                "scala.collection.mutable._".to_string(),
                "x.y._".to_string()
            ],
            "a wildcard is never provably unreferenced (D-M23-13), even from a group"
        );
    }

    #[test]
    fn identifier_only_in_string_literal_is_no_reference() {
        assert!(
            refs("import a.b.Widget\n", &["object Doc {\n  val hint = \"Widget is nice\"\n}"])
                .is_empty(),
            "the scanner hides string contents: a quoted name is not an identifier token"
        );
        assert!(
            refs("import a.b.Widget\n", &["object Doc {\n  val big = \"\"\"class Widget {}\"\"\"\n}"])
                .is_empty(),
            "triple-quoted contents count for nothing either"
        );
    }

    #[test]
    fn identifier_only_in_comment_is_no_reference() {
        assert!(
            refs(
                "import a.b.Widget\n",
                &["// Widget mentioned\n/* Widget again */\nclass Unrelated"]
            )
            .is_empty(),
            "the scanner removes comments: a commented name is not an identifier token"
        );
    }

    #[test]
    fn rename_locks_reference_on_left_name_and_emits_original_module_path() {
        assert_eq!(
            refs("import a.b.{C => D}\n", &["val x: C = ???"]),
            vec!["a.b.C".to_string()],
            "LOCK (frozen m20 grammar IV-2): the SELECTED (left) name is the \
             reference, the emitted path is the ORIGINAL module path"
        );
        assert!(
            refs("import a.b.{C => D}\n", &["val x: D = ???"]).is_empty(),
            "LOCK: the alias name does NOT count as a reference (grammar choice)"
        );
    }

    #[test]
    fn output_is_sorted_and_deduped_module_paths() {
        assert_eq!(
            refs(
                "import z.y.Last\nimport a.b.C\nimport a.b.C\nimport a.b.{C, A_early}\n",
                &["use C and A_early and Last"]
            ),
            vec![
                "a.b.A_early".to_string(),
                "a.b.C".to_string(),
                "z.y.Last".to_string()
            ]
        );
    }

    #[test]
    fn export_participates_like_import() {
        assert_eq!(
            refs("export a.b.Helper\n", &["val h: Helper = null"]),
            vec!["a.b.Helper".to_string()],
            "D16 parity: export selectors read like import selectors"
        );
    }

    #[test]
    fn bare_import_binds_last_segment_and_keeps_its_full_path() {
        assert_eq!(
            refs("import a.b.util\n", &["val xs = util.Map()"]),
            vec!["a.b.util".to_string()],
            "parser-parity plain selector: reference name = last segment"
        );
        assert!(refs("import a.b.util\n", &["class None1"]).is_empty());
    }

    #[test]
    fn commented_and_quoted_import_statements_are_never_seen() {
        assert!(
            refs("// import a.b.Ghost\nval code = \"import a.b.Ghost2\"\n", &["use Ghost"])
                .is_empty()
        );
    }

    #[test]
    fn scan_records_comma_chains_and_statement_spans_verbatim() {
        let source = "  import a.b.C, d.e.{F => G, _}\nclass C";
        let pass = scan_imports(source);
        assert_eq!(pass.statements.len(), 1, "a top-level comma continues one statement");
        let statement = &pass.statements[0];
        assert_eq!(statement.keyword, "import");
        let paths: Vec<Option<String>> =
            statement.selectors.iter().map(|s| s.form.module_path()).collect();
        assert_eq!(
            paths,
            vec![Some("a.b.C".to_string()), Some("d.e.F".to_string()), None],
            "the renamed member's module path is the ORIGINAL left-side path"
        );
        assert_eq!(
            pass.statement_text(source, statement),
            "import a.b.C, d.e.{F => G, _}"
        );
        assert_eq!(
            pass.selector_text(source, &statement.selectors[1]),
            "F => G"
        );
    }

    #[test]
    fn grouped_rename_module_path_uses_original_left_name() {
        let pass = scan_imports("import m.n.{P => Q}\n");
        let selector = &pass.statements[0].selectors[0];
        assert_eq!(selector.form.module_path().as_deref(), Some("m.n.P"));
        assert_eq!(selector.form.reference_name(), Some("P"));
    }

    #[test]
    fn dotted_group_member_binds_its_last_segment() {
        let pass = scan_imports("import a.b.{c.D}\n");
        let selector = &pass.statements[0].selectors[0];
        assert_eq!(selector.form.module_path().as_deref(), Some("a.b.c.D"));
        assert_eq!(selector.form.reference_name(), Some("D"));
    }

    #[test]
    fn char_to_byte_map_maps_multibyte_offsets_to_bytes() {
        let map = char_to_byte_map("ä→ok");
        assert_eq!(map[0], 0);
        assert_eq!(map[1], 2, "the two-byte umlaut shifts the next char");
        assert_eq!(map[2], 5, "the three-byte arrow starts at byte 5");
        assert_eq!(map[3], 6);
        assert_eq!(map[4], "ä→ok".len());
    }

    #[test]
    fn pin_truncated_group_swallows_remaining_idents_to_eof_bounded() {
        let pass = scan_imports("import a.b.{Old\nclass Use\n");
        assert_eq!(
            pass.statements.len(),
            1,
            "the unclosed open brace never closes, so one statement owns the rest of the file"
        );
        let statement = &pass.statements[0];
        assert_eq!(statement.end_tok, pass.tokens.len(), "bounded: the statement ends exactly at EOF");
        let paths: Vec<Option<String>> =
            statement.selectors.iter().map(|s| s.form.module_path()).collect();
        assert_eq!(
            paths,
            vec![
                Some("a.b.Old".to_string()),
                Some("a.b.class".to_string()),
                Some("a.b.Use".to_string())
            ],
            "LOCK (documented consequence of the open brace): every following \
             identifier becomes a group member of base `a.b` -- weird but total, \
             and the rewrite scan always sees a bounded statement, never a hang"
        );
    }
}
