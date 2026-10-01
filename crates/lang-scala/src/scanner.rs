//! Offline-fidelity Scala token scanner (M21 T-A, D18).
//!
//! Hand-written, zero dependencies, deterministic, and total: invalid input
//! never panics; unterminated constructs simply stop at EOF. Comment and
//! string content is *removed* by lexing, which is what makes the parser's
//! negative guarantees (commented-out / quoted code yields nothing) fall out
//! of structure rather than pattern-matching.
//!
//! Lexical fidelity scope (D18): `//` line comments, nested `/* */` block
//! comments, `'c'` char literals (incl. escapes and the `'` quote-run edge
//! case), `"…"`, `"""…"""`, interpolated `s"…"`/`f"…"`/`raw"…` whose `${…}`
//! splices are carried inside the string token and therefore NOT analyzed
//! (documented limitation), and backtick-quoted identifiers lexed as a single
//! identifier with verbatim content.
//!
//! Known limitations (documented, D18 carry-forwards):
//! - A quote character inside an interpolation splice (e.g. `s"${"q"}"`) ends
//!   the string token early — the same run-to-run leak concession M20 made
//!   for invalid input only.
//! - Unterminated block comments run to EOF. An unterminated char literal
//!   is *not* a literal: `scan_char_literal` returns false, the quote lexes
//!   as punctuation, and the tail continues as ordinary tokens (behavior;
//!   an earlier doc line claimed run-to-EOF).
//! - Numeric literals are consumed greedily over `[0-9A-Za-z._]` tails;
//!   postfix-dot member access on a literal (`1.toString`) lexes as one
//!   token. Never analyzed either way.

/// One lexeme with its 1-based source position (start column of the token,
/// matching the `lang-cpp` tokenizer convention).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Identifier or keyword (unquoted); or the verbatim content of a
    /// backtick-quoted identifier.
    Ident(String),
    /// Single structural/operator character.
    Punct(char),
    /// String (incl. triple-quoted and interpolated), char, or numeric
    /// literal. Content is never analyzed by the parser.
    Literal,
}

impl Token {
    pub fn is_punct(&self, ch: char) -> bool {
        matches!(&self.kind, TokenKind::Punct(p) if *p == ch)
    }

    pub fn ident(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::Ident(name) => Some(name.as_str()),
            _ => None,
        }
    }
}

/// Lex Scala source into tokens. Comments vanish; strings (including
/// interpolated splices), chars, and numbers collapse to [`TokenKind::Literal`].
pub fn tokenize(source: &str) -> Vec<Token> {
    let chars: Vec<char> = source.chars().collect();
    let mut cursor = ScanCursor::new(&chars);
    let mut tokens = Vec::new();
    while cursor.has_next() {
        cursor.skip_blanks_and_comments();
        if !cursor.has_next() {
            break;
        }
        if let Some(token) = cursor.scan_token() {
            tokens.push(token);
        }
    }
    tokens
}

struct ScanCursor<'a> {
    chars: &'a [char],
    index: usize,
    line: usize,
    column: usize,
}

impl<'a> ScanCursor<'a> {
    fn new(chars: &'a [char]) -> Self {
        Self {
            chars,
            index: 0,
            line: 1,
            column: 1,
        }
    }

    fn has_next(&self) -> bool {
        self.index < self.chars.len()
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.index + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek(0)?;
        self.index += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn skip_blanks_and_comments(&mut self) {
        loop {
            match self.peek(0) {
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some('/') if self.peek(1) == Some('/') => {
                    while self.has_next() && self.peek(0) != Some('\n') {
                        self.advance();
                    }
                }
                Some('/') if self.peek(1) == Some('*') => self.skip_nested_block_comment(),
                _ => return,
            }
        }
    }

    fn skip_nested_block_comment(&mut self) {
        self.advance();
        self.advance();
        let mut depth = 1usize;
        while self.has_next() && depth > 0 {
            match (self.peek(0), self.peek(1)) {
                (Some('/'), Some('*')) => {
                    depth += 1;
                    self.advance();
                    self.advance();
                }
                (Some('*'), Some('/')) => {
                    depth -= 1;
                    self.advance();
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn scan_token(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_column = self.column;
        let ch = self.peek(0)?;
        let kind = if ch == '"' {
            self.scan_string_literal()?;
            TokenKind::Literal
        } else if ch == '\'' {
            if !self.scan_char_literal() {
                self.advance();
                TokenKind::Punct('\'')
            } else {
                TokenKind::Literal
            }
        } else if ch == '`' {
            self.scan_backtick_ident()?
        } else if is_ident_start(ch) {
            self.scan_ident_or_prefixed_string()?
        } else if ch.is_ascii_digit() {
            self.scan_numeric_literal();
            TokenKind::Literal
        } else {
            self.advance();
            TokenKind::Punct(ch)
        };
        Some(Token {
            kind,
            line: start_line,
            column: start_column,
        })
    }

    /// Consumes `"…"`, `"""…"""`; an unterminated literal runs to EOF.
    fn scan_string_literal(&mut self) -> Option<()> {
        if self.peek(1) == Some('"') && self.peek(2) == Some('"') {
            self.advance();
            self.advance();
            self.advance();
            while self.has_next() {
                if self.peek(0) == Some('"')
                    && self.peek(1) == Some('"')
                    && self.peek(2) == Some('"')
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    return Some(());
                }
                self.advance();
            }
            return Some(());
        }
        self.advance();
        while self.has_next() {
            match self.peek(0) {
                Some('\\') => {
                    self.advance();
                    self.advance();
                }
                Some('"') | Some('\n') => {
                    if self.peek(0) == Some('"') {
                        self.advance();
                    }
                    return Some(());
                }
                _ => {
                    self.advance();
                }
            }
        }
        Some(())
    }

    /// `'c'`, `'\n'`, `'\''`; also swallows the `'''` quote-run edge case as
    /// one literal. Returns false when the quote is not a char literal
    /// (caller then lexes it as punctuation).
    fn scan_char_literal(&mut self) -> bool {
        if self.peek(1) == Some('\\') && self.peek(3) == Some('\'') {
            self.advance();
            self.advance();
            self.advance();
            self.advance();
            return true;
        }
        if self.peek(2) == Some('\'') {
            self.advance();
            self.advance();
            self.advance();
            return true;
        }
        false
    }

    fn scan_backtick_ident(&mut self) -> Option<TokenKind> {
        self.advance();
        let mut content = String::new();
        while self.has_next() && self.peek(0) != Some('`') {
            content.push(self.advance()?);
        }
        if self.peek(0) == Some('`') {
            self.advance();
        }
        Some(TokenKind::Ident(content))
    }

    /// Identifier, or an interpolation prefix (`s"`, `f"`, `raw"`) folded
    /// together with its string so splices are never analyzed.
    fn scan_ident_or_prefixed_string(&mut self) -> Option<TokenKind> {
        let start = self.index;
        while self.has_next() && is_ident_continue(self.peek(0)?) {
            self.advance();
        }
        let word: String = self.chars[start..self.index].iter().collect();
        if self.peek(0) == Some('"') && is_string_interpolator(&word) {
            self.scan_string_literal()?;
            return Some(TokenKind::Literal);
        }
        Some(TokenKind::Ident(word))
    }

    fn scan_numeric_literal(&mut self) {
        while let Some(ch) = self.peek(0) {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' {
                self.advance();
            } else {
                return;
            }
        }
    }
}

fn is_string_interpolator(word: &str) -> bool {
    matches!(word, "s" | "f" | "raw")
}

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == '$'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<String> {
        tokenize(source)
            .into_iter()
            .map(|token| match token.kind {
                TokenKind::Ident(name) => format!("Ident({name})"),
                TokenKind::Punct(ch) => format!("Punct({ch})"),
                TokenKind::Literal => "Literal".to_string(),
            })
            .collect()
    }

    fn brace_delta(source: &str) -> i32 {
        tokenize(source)
            .into_iter()
            .filter(|token| token.ident().is_none() && !matches!(token.kind, TokenKind::Literal))
            .fold(0i32, |depth, token| match token.kind {
                TokenKind::Punct('{') => depth + 1,
                TokenKind::Punct('}') => depth - 1,
                _ => depth,
            })
    }

    #[test]
    fn line_comment_hides_braces_and_text() {
        assert_eq!(brace_delta("class C {} // } { class Ghost"), 0);
        assert_eq!(kinds("val x = 1 // val y = 2"), ["Ident(val)", "Ident(x)", "Punct(=)", "Literal"]);
    }

    #[test]
    fn block_comments_nest() {
        assert_eq!(brace_delta("/* /* } */ } */ class C {}"), 0);
        assert_eq!(kinds("a /* b /* c */ d */ e"), ["Ident(a)", "Ident(e)"]);
    }

    #[test]
    fn unterminated_block_comment_runs_to_eof() {
        assert_eq!(kinds("a /* never closed"), ["Ident(a)"]);
    }

    #[test]
    fn strings_hide_their_content_and_braces() {
        assert_eq!(brace_delta("val s = \"} { class Ghost {}\""), 0);
        assert_eq!(kinds("val s = \"a\\\"b\" end"), ["Ident(val)", "Ident(s)", "Punct(=)", "Literal", "Ident(end)"]);
    }

    #[test]
    fn triple_quoted_string_keeps_inner_quotes() {
        assert_eq!(kinds("val big = \"\"\"a\"b\"\"\" tail"), ["Ident(val)", "Ident(big)", "Punct(=)", "Literal", "Ident(tail)"]);
    }

    #[test]
    fn interpolated_string_with_splice_is_one_literal() {
        assert_eq!(
            kinds("val g = s\"${user.name} and ${order.id}\" end"),
            ["Ident(val)", "Ident(g)", "Punct(=)", "Literal", "Ident(end)"]
        );
        assert_eq!(kinds("f\"%d\" + raw\"x\""), ["Literal", "Punct(+)", "Literal"]);
    }

    #[test]
    fn char_literals_with_escapes_and_quote_run() {
        assert_eq!(kinds("'a' 'x'"), ["Literal", "Literal"]);
        assert_eq!(kinds("'\\n'"), ["Literal"]);
        assert_eq!(kinds("'''"), ["Literal"]);
        assert_eq!(kinds("')'"), ["Literal"]);
    }

    #[test]
    fn backtick_identifiers_keep_content_verbatim() {
        assert_eq!(kinds("`my-op` + 1"), ["Ident(my-op)", "Punct(+)", "Literal"]);
    }

    #[test]
    fn positions_are_one_based_start_columns() {
        let tokens = tokenize("class\n  Widget");
        assert_eq!((tokens[0].line, tokens[0].column), (1, 1));
        assert_eq!((tokens[1].line, tokens[1].column), (2, 3));
    }
}
