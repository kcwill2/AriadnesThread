//! Pure-text scanner for Scala `import`/`export` directives (plan M20.1).
//!
//! [`parse_import_directives`] walks the source characters with a small
//! state machine and never touches the filesystem. Documented scope limits
//! (lexical fidelity belongs to the M21 semantic parser):
//! * at most one directive keyword per line is *required* to be found; the
//!   comma form `import a.C, d.F` yields only its first path (selectors
//!   after a top-level comma carry no keyword and are ignored as noise);
//! * multi-line `{...}` selector groups are not supported;
//! * triple-quoted (`"""`) literals are consumed run-to-run (content until
//!   the next 3+-quote run) without interpreting `$`-splices or escapes;
//! * selector forms that are not plain dotted identifiers (`given`,
//!   `type C`, alias leftovers) are ignored as noise (IV-2);
//! * a selector-less import is decided by Scala's naming convention (IV-2's
//!   documented choice): a final segment starting lowercase is a package and
//!   becomes a wildcard (`import a.b`), a capitalized final segment names a
//!   member and stays a named edge (`import a.b.C`).

/// One dependency directive parsed from source text, before any `FileCatalog`
/// mapping (consumed by [`crate::resolver`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedImport {
    /// Dotted package+name (grouped selectors expand) or package (wildcard).
    pub raw_target: String,
    /// `true` for `._`, brace-group `_`, and bare package imports (IV-2).
    pub is_wildcard: bool,
    /// 1-based line of the directive keyword.
    pub line: u32,
    /// 1-based column of the directive keyword (IV-3).
    pub column: u32,
}

/// Scans `source` and returns every `import`/`export` directive it finds,
/// skipping line comments, nested block comments, and (interpolated) string
/// literals (IV-4). `package` clauses are scope context and never appear
/// (IV-1).
pub(crate) fn parse_import_directives(source: &str) -> Vec<ParsedImport> {
    let mut cursor = ScanCursor::new(source);
    let mut directives = Vec::new();
    while !cursor.at_eof() {
        match cursor.peek() {
            Some('/') if cursor.peek_at(1) == Some('/') => skip_line_comment(&mut cursor),
            Some('/') if cursor.peek_at(1) == Some('*') => skip_block_comment(&mut cursor),
            Some('"') => {
                if starts_triple_quote(&cursor) {
                    skip_triple_quoted_literal(&mut cursor);
                } else {
                    let interpolated = string_is_interpolated(&cursor);
                    skip_string_literal(&mut cursor, interpolated);
                }
            }
            Some(c) if is_identifier_char(c) => {
                let (line, column) = (cursor.line, cursor.column);
                let word = cursor.take_identifier();
                // A bare `import`/`export` with no path after it (e.g. at
                // line end) parses to nothing inside `parse_directive_targets`.
                if matches!(word.as_str(), "import" | "export") {
                    directives.extend(parse_directive_targets(&mut cursor, line, column));
                }
            }
            _ => cursor.advance(),
        }
    }
    directives
}

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Character-position cursor: index plus 1-based line/column tracking so
/// directives carry their true source position (IV-3).
struct ScanCursor {
    chars: Vec<char>,
    index: usize,
    line: u32,
    column: u32,
}

impl ScanCursor {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.index + offset).copied()
    }

    fn at_eof(&self) -> bool {
        self.index >= self.chars.len()
    }

    fn advance(&mut self) {
        match self.chars.get(self.index).copied() {
            None => {}
            Some('\n') => {
                self.index += 1;
                self.line += 1;
                self.column = 1;
            }
            Some(_) => {
                self.index += 1;
                self.column += 1;
            }
        }
    }

    fn take_identifier(&mut self) -> String {
        let mut word = String::new();
        while self.peek().is_some_and(is_identifier_char) {
            word.push(self.chars[self.index]);
            self.advance();
        }
        word
    }

    /// Identifier run starting `offset` characters ahead, without consuming.
    fn identifier_run_ahead(&self, offset: usize) -> String {
        let mut run = String::new();
        let mut at = self.index + offset;
        while let Some(&c) = self.chars.get(at) {
            if !is_identifier_char(c) {
                break;
            }
            run.push(c);
            at += 1;
        }
        run
    }
}

fn skip_line_comment(cursor: &mut ScanCursor) {
    while !cursor.at_eof() && cursor.peek() != Some('\n') {
        cursor.advance();
    }
    cursor.advance();
}

/// Consumes a `/* */` comment; Scala block comments nest, so depth counts
/// openers until the matching closer (or EOF swallows the rest).
fn skip_block_comment(cursor: &mut ScanCursor) {
    cursor.advance();
    cursor.advance();
    let mut open_depth = 1u32;
    while open_depth > 0 && !cursor.at_eof() {
        match (cursor.peek(), cursor.peek_at(1)) {
            (Some('/'), Some('*')) => {
                cursor.advance();
                cursor.advance();
                open_depth += 1;
            }
            (Some('*'), Some('/')) => {
                cursor.advance();
                cursor.advance();
                open_depth -= 1;
            }
            _ => cursor.advance(),
        }
    }
}

/// `ident"..."` is a string interpolator for any identifier in Scala 3, so
/// an identifier char before the quote selects interpolation semantics
/// (`${...}` splices) rather than plain-string semantics.
fn string_is_interpolated(cursor: &ScanCursor) -> bool {
    cursor
        .chars
        .get(cursor.index.saturating_sub(1))
        .copied()
        .is_some_and(|before| before != '"' && is_identifier_char(before))
}

/// True when a run of at least three quotes starts exactly at the cursor
/// (the current char included — the run test must not fire one char early).
fn starts_triple_quote(cursor: &ScanCursor) -> bool {
    cursor.peek() == Some('"')
        && cursor.peek_at(1) == Some('"')
        && cursor.peek_at(2) == Some('"')
}

/// Consumes a `"""..."""` multi-line literal run-to-run: after the opening
/// three quotes, everything up to and including the next run of at least
/// three quotes is literal text (no directive, no escape, and — an
/// accepted M20 limitation — no interpolation splices inside `s"""..."""`
/// are interpreted). Content beginning with a quote can therefore merge
/// with the closing run; full lexical fidelity is the M21 parser's job.
fn skip_triple_quoted_literal(cursor: &mut ScanCursor) {
    cursor.advance();
    cursor.advance();
    cursor.advance();
    while !cursor.at_eof() {
        if starts_triple_quote(cursor) {
            cursor.advance();
            cursor.advance();
            cursor.advance();
            return;
        }
        cursor.advance();
    }
}

fn skip_string_literal(cursor: &mut ScanCursor, interpolated: bool) {
    cursor.advance();
    while !cursor.at_eof() {
        match cursor.peek() {
            Some('\\') => {
                cursor.advance();
                cursor.advance();
            }
            Some('"') => {
                cursor.advance();
                return;
            }
            Some('$') if interpolated && cursor.peek_at(1) == Some('{') => {
                cursor.advance();
                cursor.advance();
                skip_interpolation_splice(cursor);
            }
            _ => cursor.advance(),
        }
    }
}

/// Consumes a `${...}` splice (brace-balanced; nested string literals are
/// scanned so their quotes cannot end the outer interpolated string).
fn skip_interpolation_splice(cursor: &mut ScanCursor) {
    let mut open_braces = 1u32;
    while open_braces > 0 && !cursor.at_eof() {
        match cursor.peek() {
            Some('{') => {
                cursor.advance();
                open_braces += 1;
            }
            Some('}') => {
                cursor.advance();
                open_braces -= 1;
            }
            Some('"') => {
                let interpolated = string_is_interpolated(cursor);
                skip_string_literal(cursor, interpolated);
            }
            _ => cursor.advance(),
        }
    }
}

/// Parses everything after an `import`/`export` keyword into directives:
/// the dotted package path, then `.{C, D}` selectors, a `._` wildcard, or a
/// bare path decided by [`bare_import`].
fn parse_directive_targets(
    cursor: &mut ScanCursor,
    line: u32,
    column: u32,
) -> Vec<ParsedImport> {
    skip_inline_spaces(cursor);
    let package_path = take_dotted_path(cursor);
    if package_path.is_empty() {
        return Vec::new();
    }
    match (cursor.peek(), cursor.peek_at(1)) {
        (Some('.'), Some('{')) => {
            cursor.advance();
            cursor.advance();
            collect_braced_selectors(cursor, &package_path, line, column)
        }
        (Some('.'), Some('_')) => {
            cursor.advance();
            cursor.advance();
            vec![wildcard_import(&package_path, line, column)]
        }
        _ => vec![bare_import(&package_path, line, column)],
    }
}

fn skip_inline_spaces(cursor: &mut ScanCursor) {
    while matches!(cursor.peek(), Some(' ') | Some('\t')) {
        cursor.advance();
    }
}

fn wildcard_import(package_path: &str, line: u32, column: u32) -> ParsedImport {
    ParsedImport {
        raw_target: package_path.to_string(),
        is_wildcard: true,
        line,
        column,
    }
}

/// Selector-less import: Scala's naming convention decides. A final segment
/// starting lowercase is a package, so the import is a bare wildcard of it
/// (IV-2); a capitalized final segment names a member (`import a.b.C`) and
/// is a plain named edge.
fn bare_import(package_path: &str, line: u32, column: u32) -> ParsedImport {
    let last_segment = package_path.rsplit('.').next().unwrap_or(package_path);
    ParsedImport {
        raw_target: package_path.to_string(),
        is_wildcard: !last_segment.starts_with(char::is_uppercase),
        line,
        column,
    }
}

/// Consumes `segment (. segment)*`, stopping before `._` and before the
/// dot that opens a `.{` selector group.
fn take_dotted_path(cursor: &mut ScanCursor) -> String {
    let mut path = cursor.take_identifier();
    while cursor.peek() == Some('.') {
        let ahead = cursor.identifier_run_ahead(1);
        if ahead.is_empty() || ahead == "_" {
            break;
        }
        cursor.advance();
        path.push('.');
        path.push_str(&cursor.take_identifier());
    }
    path
}

/// Consumes a `{...}` selector group and expands it to one directive per
/// selected name (IV-2).
fn collect_braced_selectors(
    cursor: &mut ScanCursor,
    package_path: &str,
    line: u32,
    column: u32,
) -> Vec<ParsedImport> {
    let mut selectors: Vec<String> = Vec::new();
    let mut current = String::new();
    while let Some(c) = cursor.peek() {
        cursor.advance();
        match c {
            '}' => break,
            ',' => selectors.push(std::mem::take(&mut current)),
            other => current.push(other),
        }
    }
    if !current.trim().is_empty() {
        selectors.push(current);
    }
    selectors
        .iter()
        .filter_map(|selector| selector_import(selector, package_path, line, column))
        .collect()
}

/// Maps one selector to a directive: `_`/`_*` yields the package wildcard,
/// renames keep the selected (left) name, `given` and every non-identifier
/// form is ignored as noise (documented IV-2 choice).
fn selector_import(
    selector: &str,
    package_path: &str,
    line: u32,
    column: u32,
) -> Option<ParsedImport> {
    let selected = selector.split("=>").next().unwrap_or("").trim();
    if matches!(selected, "_" | "_*") {
        return Some(wildcard_import(package_path, line, column));
    }
    if selected == "given" || !is_dotted_identifier(selected) {
        return None;
    }
    Some(ParsedImport {
        raw_target: format!("{package_path}.{selected}"),
        is_wildcard: false,
        line,
        column,
    })
}

fn is_dotted_identifier(path: &str) -> bool {
    !path.is_empty()
        && path.starts_with(is_identifier_char)
        && path.split('.').all(|segment| {
            !segment.is_empty() && segment.chars().all(is_identifier_char)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_comment_only_source_parse_to_nothing() {
        assert!(parse_import_directives("").is_empty());
        assert!(parse_import_directives("/* nothing here */").is_empty());
    }

    #[test]
    fn parsed_import_records_true_line_and_column() {
        let parsed = parse_import_directives("\n  import com.example.models.{Order}\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!((parsed[0].line, parsed[0].column), (2, 3));
        assert_eq!(parsed[0].raw_target, "com.example.models.Order");
        assert!(!parsed[0].is_wildcard);
    }

    #[test]
    fn unterminated_string_and_comment_swallow_trailing_directives() {
        assert!(parse_import_directives("val a = \"unclosed import x.y").is_empty());
        assert!(parse_import_directives("/* unclosed import x.y").is_empty());
    }

    #[test]
    fn keyword_like_identifiers_are_not_directives() {
        assert!(parse_import_directives("val importLike = 1\nval exported = export_thing\n").is_empty());
    }

    #[test]
    fn triple_quoted_string_content_never_leaks_directives() {
        assert!(
            parse_import_directives(
                "val doc = \"\"\"tip: write \"import com.example.snippet.Hidden\" first\"\"\""
            )
            .is_empty(),
            "quoted text inside a triple-quoted literal is not code"
        );
        assert!(
            parse_import_directives(
                "val doc = s\"\"\"${name} note: \"import x.y\" done\"\"\"\n"
            )
            .is_empty(),
            "an interpolated s\"\"\"...\"\"\" literal is consumed run-to-run"
        );
    }

    #[test]
    fn code_after_a_closing_triple_quote_is_scanned_normally() {
        let parsed = parse_import_directives(
            "val doc = \"\"\"note: \"import x.y\" still content\"\"\"\nimport com.example.models.Real\n",
        );
        assert_eq!(parsed.len(), 1, "only the code after the closing quote");
        assert_eq!(parsed[0].raw_target, "com.example.models.Real");
    }
}
