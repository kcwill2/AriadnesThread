#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

/// A [`Token`] plus its char span in the source ([`tokenize_raw`] output).
/// The M23 verbatim locator (plan D-M23-1) slices declaration text by these
/// offsets; `tokenize` keeps the offset-free view every legacy consumer
/// sees, so the characterization tests lock one identical stream.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RawToken {
    pub(crate) kind: TokenKind,
    pub(crate) line: usize,
    pub(crate) column: usize,
    /// Index of the token's first char within the `Vec<char>` view.
    pub(crate) start: usize,
    /// One past the token's last char (exclusive).
    pub(crate) end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenKind {
    Word(String),
    Punct(String),
}

pub(crate) fn tokenize(source: &str) -> Vec<Token> {
    tokenize_raw(source)
        .into_iter()
        .map(|raw| Token {
            kind: raw.kind,
            line: raw.line,
            column: raw.column,
        })
        .collect()
}

pub(crate) fn tokenize_raw(source: &str) -> Vec<RawToken> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    let mut current_word_start = 0;
    let mut current_punct = String::new();
    let mut current_punct_start = 0;
    let mut line = 1;
    let mut col = 1;

    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            flush_word(&mut tokens, &mut current_word, line, col, current_word_start);
            flush_punct(&mut tokens, &mut current_punct, line, col, current_punct_start);
            scan_line_comment(&chars, &mut i, &mut col);
            continue;
        }

        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            flush_word(&mut tokens, &mut current_word, line, col, current_word_start);
            flush_punct(&mut tokens, &mut current_punct, line, col, current_punct_start);
            scan_block_comment(&chars, &mut i, &mut line, &mut col);
            continue;
        }

        if c == '"' || c == '\'' {
            scan_quote(
                &chars,
                c,
                &mut tokens,
                &mut current_word,
                &mut current_punct,
                &mut i,
                &mut line,
                &mut col,
                current_word_start,
                current_punct_start,
            );
            continue;
        }

        if c == '\n' {
            flush_word(&mut tokens, &mut current_word, line, col, current_word_start);
            flush_punct(&mut tokens, &mut current_punct, line, col, current_punct_start);
            line += 1;
            col = 1;
            i += 1;
            continue;
        }

        if c.is_alphanumeric() || c == '_' {
            flush_punct(&mut tokens, &mut current_punct, line, col, current_punct_start);
            if current_word.is_empty() {
                current_word_start = i;
            }
            current_word.push(c);
        } else if c.is_whitespace() {
            flush_word(&mut tokens, &mut current_word, line, col, current_word_start);
            flush_punct(&mut tokens, &mut current_punct, line, col, current_punct_start);
        } else {
            flush_word(&mut tokens, &mut current_word, line, col, current_word_start);
            current_punct_start = i;
            current_punct.push(c);
            flush_punct(
                &mut tokens,
                &mut current_punct,
                line,
                col + 1,
                current_punct_start,
            ); // single char punct for now
        }
        col += 1;
        i += 1;
    }
    flush_word(
        &mut tokens,
        &mut current_word,
        line,
        col,
        current_word_start,
    );
    flush_punct(
        &mut tokens,
        &mut current_punct,
        line,
        col,
        current_punct_start,
    );
    tokens
}

/// Handles a `"` or `'` met by the main dispatch: emits no token for the
/// literal itself, only advancing position. A `"` opening a raw string
/// consumes the whole `R"delim(...)delim"` via [`scan_raw_string`] and drops
/// the pending identifier run (its trailing `R` is the raw prefix, not a
/// word). Any other quote - or a raw whose terminator is missing at EOF -
/// takes the ordinary [`scan_string_literal`] swallow, exactly as before the
/// raw-string fix. Flushes pending word/punct like the original branch.
#[allow(clippy::too_many_arguments)]
fn scan_quote(
    chars: &[char],
    quote: char,
    tokens: &mut Vec<RawToken>,
    word: &mut String,
    punct: &mut String,
    i: &mut usize,
    line: &mut usize,
    col: &mut usize,
    word_start: usize,
    punct_start: usize,
) {
    if quote == '"' && raw_string_open(chars, *i) {
        word.clear();
        flush_punct(tokens, punct, *line, *col, punct_start);
        if !scan_raw_string(chars, *i, i, line, col) {
            scan_string_literal(chars, quote, i, line, col);
        }
        return;
    }
    flush_word(tokens, word, *line, *col, word_start);
    flush_punct(tokens, punct, *line, *col, punct_start);
    scan_string_literal(chars, quote, i, line, col);
}

fn scan_line_comment(chars: &[char], i: &mut usize, col: &mut usize) {
    while *i < chars.len() && chars[*i] != '\n' {
        *i += 1;
        *col += 1;
    }
}

fn scan_block_comment(chars: &[char], i: &mut usize, line: &mut usize, col: &mut usize) {
    *i += 2;
    *col += 2;
    while *i < chars.len() {
        if chars[*i] == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        if chars[*i] == '*' && *i + 1 < chars.len() && chars[*i + 1] == '/' {
            *i += 2;
            *col += 1;
            break;
        }
        *i += 1;
    }
}

fn scan_string_literal(
    chars: &[char],
    quote: char,
    i: &mut usize,
    line: &mut usize,
    col: &mut usize,
) {
    *i += 1;
    *col += 1;
    let mut escape = false;
    while *i < chars.len() {
        let quote_char = chars[*i];
        if quote_char == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        if escape {
            escape = false;
        } else if quote_char == '\\' {
            escape = true;
        } else if quote_char == quote {
            *i += 1;
            break;
        }
        *i += 1;
    }
}

/// True when the `"` at `quote_at` opens a raw string literal: the chars
/// immediately before it form a non-empty `[A-Za-z0-9_]+` run ending in `R`
/// (covers `R"`, `u8R"`, `uR"`, `UR"`, `LR"`) AND the chars right after it
/// are a legal delimiter run `[A-Za-z0-9_]*` terminated by `(`. Identifier-
/// immediately-followed-by-quote is invalid C++ outside raw contexts, so
/// false positives cannot appear in code that compiles; the `(`-delimiter
/// requirement rejects `R""`-style oddities. Terminator presence is decided
/// by [`scan_raw_string`].
fn raw_string_open(chars: &[char], quote_at: usize) -> bool {
    if quote_at == 0 || chars[quote_at - 1] != 'R' {
        return false;
    }
    let mut probe = quote_at + 1;
    while probe < chars.len() && (chars[probe].is_ascii_alphanumeric() || chars[probe] == '_') {
        probe += 1;
    }
    probe < chars.len() && chars[probe] == '('
}

/// Consumes the raw string literal `R"delim(...)delim"` whose opening `"` is
/// at `quote_at`, tracking line/col like [`scan_string_literal`]. Emits NO
/// tokens and applies NO backslash escaping inside the content (raw content
/// is verbatim per the standard). Returns `false` with position untouched
/// when no closing `)delim"` exists before EOF; the caller then falls back to
/// [`scan_string_literal`], so an unterminated raw swallows the rest of the
/// file exactly like the locked unterminated-string behavior
/// (`tokenizer/characterization_tests.rs:116`). Assumes [`raw_string_open`]
/// was true for this quote.
fn scan_raw_string(
    chars: &[char],
    quote_at: usize,
    i: &mut usize,
    line: &mut usize,
    col: &mut usize,
) -> bool {
    let delimiter_start = quote_at + 1;
    let mut content_start = delimiter_start;
    while content_start < chars.len()
        && (chars[content_start].is_ascii_alphanumeric() || chars[content_start] == '_')
    {
        content_start += 1;
    }
    if content_start >= chars.len() || chars[content_start] != '(' {
        return false;
    }
    let delimiter_len = content_start - delimiter_start;
    let mut cursor = content_start + 1;
    loop {
        if cursor >= chars.len() {
            return false;
        }
        let closing_quote = cursor + 1 + delimiter_len;
        if chars[cursor] == ')'
            && closing_quote < chars.len()
            && chars[closing_quote] == '"'
            && chars[cursor + 1..closing_quote] == chars[delimiter_start..content_start]
        {
            // Commit: consume from the opening quote through the terminator's
            // closing quote inclusive, mirroring scan_string_literal's
            // line/col accounting.
            while *i <= closing_quote {
                if chars[*i] == '\n' {
                    *line += 1;
                    *col = 1;
                } else {
                    *col += 1;
                }
                *i += 1;
            }
            return true;
        }
        cursor += 1;
    }
}

fn flush_word(
    tokens: &mut Vec<RawToken>,
    word: &mut String,
    line: usize,
    column: usize,
    start: usize,
) {
    if !word.is_empty() {
        let len = word.chars().count();
        tokens.push(RawToken {
            kind: TokenKind::Word(word.clone()),
            line,
            column: column - len,
            start,
            end: start + len,
        });
        word.clear();
    }
}

fn flush_punct(
    tokens: &mut Vec<RawToken>,
    punct: &mut String,
    line: usize,
    column: usize,
    start: usize,
) {
    if !punct.is_empty() {
        let len = punct.chars().count();
        tokens.push(RawToken {
            kind: TokenKind::Punct(punct.clone()),
            line,
            column: column - len,
            start,
            end: start + len,
        });
        punct.clear();
    }
}

#[cfg(test)]
mod characterization_tests;

#[cfg(test)]
mod raw_string_tests;

#[cfg(test)]
mod underflow_fix_tests;
