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
    /// 1-based column in CHARACTERS, never bytes. AR-31/M27 lock: the
    /// column counter advances one per decoded char and the flush
    /// subtraction is char-length - a byte-counted column or byte-length
    /// subtraction is the M27 underflow class (`tokenizer/
    /// underflow_fix_tests.rs`; ASCII parity locked by
    /// `tokenizer/fixture_parity_tests.rs`).
    pub(crate) column: usize,
    /// Index of the token's first char within the source's char view.
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

/// A pending identifier run. Word runs are contiguous stretches of source
/// chars (every interruption - whitespace, punct, comment, quote - flushes
/// first), so the run is a byte range plus its char-span bookkeeping: no
/// per-char push, and the token text is one exact-size allocation from the
/// source at flush (AR-31; the pre-rework code materialized a `Vec<char>`
/// per TU and cloned a `String` buffer per token).
struct PendingWord {
    /// Byte offset of the run's first char.
    text_start: usize,
    /// Byte offset one past the run's last char (exclusive).
    text_end: usize,
    /// Index of the run's first char in the source's char view.
    char_start: usize,
    /// Number of chars in the run.
    chars: usize,
}

impl PendingWord {
    /// Empty (nothing pending).
    const INACTIVE: PendingWord = PendingWord {
        text_start: 0,
        text_end: 0,
        char_start: 0,
        chars: 0,
    };

    fn is_pending(&self) -> bool {
        self.chars > 0
    }

    /// Emits the run as a [`TokenKind::Word`] at the flush-time line and
    /// char column, then resets. `column - chars` is char-minus-char: the
    /// M27 fix class - subtracting the BYTE length here underflowed or
    /// wrapped on non-ASCII runs.
    fn take_into(
        &mut self,
        tokens: &mut Vec<RawToken>,
        source: &str,
        line: usize,
        column: usize,
    ) {
        if self.is_pending() {
            let text = &source[self.text_start..self.text_end];
            tokens.push(RawToken {
                kind: TokenKind::Word(text.to_owned()),
                line,
                column: column - self.chars,
                start: self.char_start,
                end: self.char_start + self.chars,
            });
        }
        *self = Self::INACTIVE;
    }
}

/// Tokenizes C++ source into the raw stream (kind/text/line/char-column/
/// char-span). Iterates the source bytes with lazy char decoding (no
/// per-TU `Vec<char>` materialization).
///
/// Punct-buffer invariant: the dispatch emits every punctuation char as a
/// single-char token the moment it is read, so no pending-punct buffer
/// exists - the pre-rework buffer was provably empty between iterations
/// (its only push site was immediately followed by its own flush, and
/// every other flush call saw an empty buffer), so those flush sites were
/// no-ops and are gone.
pub(crate) fn tokenize_raw(source: &str) -> Vec<RawToken> {
    let bytes = source.as_bytes();
    let mut tokens: Vec<RawToken> = Vec::new();
    let mut word = PendingWord::INACTIVE;
    let mut line = 1usize;
    let mut column = 1usize;
    let mut byte = 0usize;
    let mut char_index = 0usize;

    while byte < bytes.len() {
        let current = char_at(source, byte);
        let width = current.len_utf8();

        if current == '/' && byte_at_eq(bytes, byte + 1, b'/') {
            word.take_into(&mut tokens, source, line, column);
            scan_line_comment(source, &mut byte, &mut char_index, &mut column);
            continue;
        }

        if current == '/' && byte_at_eq(bytes, byte + 1, b'*') {
            word.take_into(&mut tokens, source, line, column);
            scan_block_comment(source, &mut byte, &mut char_index, &mut line, &mut column);
            continue;
        }

        if current == '"' || current == '\'' {
            scan_quote(
                source,
                current,
                &mut tokens,
                &mut word,
                &mut byte,
                &mut char_index,
                &mut line,
                &mut column,
            );
            continue;
        }

        if current == '\n' {
            word.take_into(&mut tokens, source, line, column);
            line += 1;
            column = 1;
            byte += 1;
            char_index += 1;
            continue;
        }

        if current.is_alphanumeric() || current == '_' {
            if !word.is_pending() {
                word.text_start = byte;
                word.char_start = char_index;
            }
            word.text_end = byte + width;
            word.chars += 1;
        } else if current.is_whitespace() {
            word.take_into(&mut tokens, source, line, column);
        } else {
            word.take_into(&mut tokens, source, line, column);
            tokens.push(RawToken {
                kind: TokenKind::Punct(current.to_string()),
                line,
                column,
                start: char_index,
                end: char_index + 1,
            });
        }
        column += 1;
        byte += width;
        char_index += 1;
    }
    word.take_into(&mut tokens, source, line, column);
    tokens
}

/// Decodes the char starting at `byte`. The cursor only ever advances by
/// decoded widths and by whole ASCII pairs (`/*`, `*/`, `//` lookahead
/// guards), so it always sits on a UTF-8 boundary.
fn char_at(source: &str, byte: usize) -> char {
    source[byte..]
        .chars()
        .next()
        .expect("cursor sits on a char boundary")
}

/// True when `probe` is in range and carries the ASCII byte `wanted`. An
/// ASCII byte never occurs inside a multi-byte sequence (every continuation
/// and lead byte is >= 0x80), so this matches exactly the chars the old
/// `chars[i + 1] == '/'`-style lookaheads matched.
fn byte_at_eq(bytes: &[u8], probe: usize, wanted: u8) -> bool {
    probe < bytes.len() && bytes[probe] == wanted
}

/// Handles a `"` or `'` met by the main dispatch: emits no token for the
/// literal itself, only advancing position. A `"` opening a raw string
/// consumes the whole `R"delim(...)delim"` via
/// [`scan_raw_string_with_fallback`] and drops the pending identifier run
/// (its trailing `R` is the raw prefix, not a word). Any other quote - or a
/// raw whose terminator is missing at EOF - takes the ordinary
/// [`scan_string_literal`] swallow, exactly as before the raw-string fix.
#[allow(clippy::too_many_arguments)]
fn scan_quote(
    source: &str,
    quote: char,
    tokens: &mut Vec<RawToken>,
    word: &mut PendingWord,
    byte: &mut usize,
    char_index: &mut usize,
    line: &mut usize,
    column: &mut usize,
) {
    if quote == '"' && raw_string_open(source, *byte) {
        *word = PendingWord::INACTIVE;
        scan_raw_string_with_fallback(source, *byte, byte, char_index, line, column);
        return;
    }
    word.take_into(tokens, source, *line, *column);
    scan_string_literal(source, quote, byte, char_index, line, column);
}

/// Consumes a `//` line comment: every char up to (not including) the
/// terminating newline, or to EOF, advancing column per CHAR.
fn scan_line_comment(source: &str, byte: &mut usize, char_index: &mut usize, column: &mut usize) {
    while *byte < source.len() {
        let current = char_at(source, *byte);
        if current == '\n' {
            break;
        }
        *byte += current.len_utf8();
        *char_index += 1;
        *column += 1;
    }
}

/// Consumes a `/* ... */` block comment (opening at `byte`), including an
/// unterminated one to EOF, with per-CHAR line/column accounting.
fn scan_block_comment(
    source: &str,
    byte: &mut usize,
    char_index: &mut usize,
    line: &mut usize,
    column: &mut usize,
) {
    *byte += 2;
    *char_index += 2;
    *column += 2;
    while *byte < source.len() {
        let current = char_at(source, *byte);
        if current == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
        if current == '*' && byte_at_eq(source.as_bytes(), *byte + 1, b'/') {
            *byte += 2;
            *char_index += 2;
            *column += 1;
            break;
        }
        *byte += current.len_utf8();
        *char_index += 1;
    }
}

/// Swallows a quoted string/char literal (opening at `byte`) to its closing
/// unescaped quote or EOF, with per-CHAR line/column accounting. Emits no
/// token.
fn scan_string_literal(
    source: &str,
    quote: char,
    byte: &mut usize,
    char_index: &mut usize,
    line: &mut usize,
    column: &mut usize,
) {
    *byte += quote.len_utf8();
    *char_index += 1;
    *column += 1;
    let mut escaped = false;
    while *byte < source.len() {
        let current = char_at(source, *byte);
        if current == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
        if escaped {
            escaped = false;
        } else if current == '\\' {
            escaped = true;
        } else if current == quote {
            *byte += current.len_utf8();
            *char_index += 1;
            break;
        }
        *byte += current.len_utf8();
        *char_index += 1;
    }
}

/// True when the `"` at `quote_byte` opens a raw string literal: the char
/// immediately before it is `R` AND the bytes right after it are a legal
/// delimiter run `[A-Za-z0-9_]*` terminated by `(` (covers `R"`, `u8R"`,
/// `uR"`, `UR"`, `LR"`). Identifier-immediately-followed-by-quote is
/// invalid C++ outside raw contexts, so false positives cannot appear in
/// code that compiles; the `(`-delimiter requirement rejects `R""`-style
/// oddities. Terminator presence is decided by
/// [`scan_raw_string_with_fallback`].
///
/// Byte-level equivalence argument: UTF-8 multi-byte sequences consist only
/// of bytes >= 0x80, so an ASCII lead/continuation byte (`R`, delimiter
/// chars, `(`) is always its own full char - the char-run checks and these
/// byte checks accept the same positions.
fn raw_string_open(source: &str, quote_byte: usize) -> bool {
    let bytes = source.as_bytes();
    if quote_byte == 0 || bytes[quote_byte - 1] != b'R' {
        return false;
    }
    let mut probe = quote_byte + 1;
    while probe < bytes.len() && (bytes[probe].is_ascii_alphanumeric() || bytes[probe] == b'_') {
        probe += 1;
    }
    probe < bytes.len() && bytes[probe] == b'('
}

/// Position snapshot with all four counters (byte cursor, char index, line,
/// char column).
struct ScanState {
    byte: usize,
    char_index: usize,
    line: usize,
    column: usize,
}

/// Consumes the raw string literal `R"delim(...)delim"` whose opening `"`
/// is at `quote_byte`, in ONE pass over the region. Emits NO tokens and
/// applies NO backslash escaping inside the content (raw content is
/// verbatim per the standard).
///
/// The pass simultaneously decides which of the two locked outcomes applies
/// and records the full position for it, so the unterminated-at-EOF case no
/// longer scans the remainder twice (raw search to EOF, then the fallback
/// string re-scan):
/// - Raw commit: the byte pattern `)` + `delim` + `"` (terminator) exists
///   anywhere after the content opener. Commit wins over any earlier
///   unescaped `"` - the pre-rework code ran the raw search first and
///   committed whenever it found a terminator, regardless of quotes inside
///   the content. `)` / delimiter / `"` are all ASCII, so the pattern
///   cannot match inside a multi-byte sequence.
/// - Fallback: no terminator exists. The ordinary [`scan_string_literal`]
///   swallow (kept for the odd header-guard path and as the locked
///   unterminated-string behavior, `tokenizer/characterization_tests.rs:116`)
///   would stop at the FIRST unescaped `"` - the content backslash state is
///   tracked here so that stop is recorded in the same pass - or at EOF if
///   no such quote appears.
///
/// Assumes [`raw_string_open`] was true for this quote; the header guard is
/// paranoid and routes to the plain fallback if ever violated.
fn scan_raw_string_with_fallback(
    source: &str,
    quote_byte: usize,
    byte: &mut usize,
    char_index: &mut usize,
    line: &mut usize,
    column: &mut usize,
) {
    let bytes = source.as_bytes();
    let delimiter_start = quote_byte + 1;
    let mut content_opener = delimiter_start;
    while content_opener < bytes.len()
        && (bytes[content_opener].is_ascii_alphanumeric() || bytes[content_opener] == b'_')
    {
        content_opener += 1;
    }
    if content_opener >= bytes.len() || bytes[content_opener] != b'(' {
        scan_string_literal(source, '"', byte, char_index, line, column);
        return;
    }
    let delimiter_len = content_opener - delimiter_start;

    // Header consume: opening quote + delimiter chars + `(` - all ASCII,
    // none can be a newline, backslash, or quote, so the fallback's escape
    // state enters the content clean and each char is a plain +1 column.
    *column += delimiter_len + 2;
    *char_index += delimiter_len + 2;
    let mut cursor = content_opener + 1;

    // Invariant: (byte..cursor) is fully accounted in the local position
    // counters below as the loop advances the cursor.
    let mut escaped = false;
    let mut string_stop: Option<ScanState> = None;
    let mut raw_commit: Option<ScanState> = None;
    while cursor < bytes.len() {
        if bytes[cursor] == b')' {
            let closing_quote = cursor + 1 + delimiter_len;
            if closing_quote < bytes.len()
                && bytes[cursor + 1..closing_quote] == bytes[delimiter_start..content_opener]
                && bytes[closing_quote] == b'"'
            {
                // Terminator chars `)`, delimiter, `"` are delimiter_len + 2
                // ASCII chars, none a newline, so position advances plainly.
                raw_commit = Some(ScanState {
                    byte: closing_quote + 1,
                    char_index: *char_index + delimiter_len + 2,
                    line: *line,
                    column: *column + delimiter_len + 2,
                });
                break;
            }
        }
        let current = char_at(source, cursor);
        let width = current.len_utf8();
        if current == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
        if escaped {
            escaped = false;
        } else if current == '\\' {
            escaped = true;
        } else if current == '"' && string_stop.is_none() {
            string_stop = Some(ScanState {
                byte: cursor + width,
                char_index: *char_index + 1,
                line: *line,
                column: *column,
            });
        }
        *char_index += 1;
        cursor += width;
    }

    // Raw commit outranks the recorded fallback stop (old priority: the
    // terminator search ran first and committed on any hit).
    let final_state = raw_commit.or(string_stop).unwrap_or(ScanState {
        byte: source.len(),
        char_index: *char_index,
        line: *line,
        column: *column,
    });
    *byte = final_state.byte;
    *char_index = final_state.char_index;
    *line = final_state.line;
    *column = final_state.column;
}

#[cfg(test)]
mod characterization_tests;

#[cfg(test)]
mod raw_string_tests;

#[cfg(test)]
mod underflow_fix_tests;

#[cfg(test)]
mod fixture_parity_tests;
