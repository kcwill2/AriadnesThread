//! M27 T2 underflow-fix locks - the byte-vs-char column class at the two
//! tokenizer flush sites (E3 root cause: `col` advances per CHAR at
//! `tokenizer.rs:115` while `flush_word`/`flush_punct` subtracted the run's
//! BYTE length from the char-counted column).
//!
//! Two profiles lock the class (shape I14): DEBUG (overflow-checks ON) the
//! subtraction PANICS with E3's exact `attempt to subtract with overflow`
//! at `:311` (word shape) / `:331` (punct shape); RELEASE (overflow-checks
//! off) it WRAPS SILENTLY to huge columns. The bound property below is
//! falsifiable in release where no panic can fire, so post-fix the wrap is
//! mathematically impossible, not merely absent. Run this module under BOTH
//! `cargo test -p ariadne-lang-cpp` and `cargo test --release -p
//! ariadne-lang-cpp`.
//!
//! Authored pure ASCII: every multi-byte specimen is built from its stored
//! UTF-8 byte sequences (`b"\xc3\xa9"`-style escapes), never from non-ASCII
//! literals (AGENTS.md UTF-8 rule; mirrors the committed T0b fixture).

use super::*;

/// U+00E9 LATIN SMALL E WITH ACUTE - word-run character of the E3 class
/// (stored bytes C3 A9; the T0b fixture `word_run.cpp` carries the pair).
const E_ACUTE: &[u8] = b"\xc3\xa9";
/// U+00AB GUILLEMET - two-byte punct-run character (stored bytes C2 AB;
/// the fixture punct shape). Alone it can only floor the column at 0,
/// never panic - proved empirically by the T0b Gate-1 capture.
const GUILLEMET: &[u8] = b"\xc2\xab";
/// U+20AC EURO SIGN - three-byte punct PANIC DRIVER (stored bytes
/// E2 82 AC): at column 1 the punct flush subtracts 3 bytes from the
/// char-counted column+1 of 2 - the exact E3 site `tokenizer.rs:331:21`.
/// Same driver as the committed T0b `punct_run.cpp`.
const EURO: &[u8] = b"\xe2\x82\xac";
/// U+4E2D CJK IDEOGRAPH - three-byte WORD character (stored bytes
/// E4 B8 AD): one such char at column 1 over-subtracts flush_word by one
/// (3 bytes vs 1 char), driving the `:311` panic class.
const CJK_MID: &[u8] = b"\xe4\xb8\xad";

/// Concatenates byte parts and decodes them, so every multi-byte specimen
/// character enters the source via its stored UTF-8 bytes, never a literal.
fn specimen(parts: &[&[u8]]) -> String {
    let mut bytes: Vec<u8> = Vec::new();
    for part in parts {
        bytes.extend_from_slice(part);
    }
    String::from_utf8(bytes).expect("specimen parts are single-encoded UTF-8")
}

/// I14's falsifiable statement applied to one source: `tokenize_raw`
/// neither panics (debug) nor wraps (release); every `RawToken.column`
/// lands in `[1, line_chars + 1]`; every token's char span obeys
/// `end - start == text.chars().count()`.
fn assert_token_columns_bounded(source: &str) {
    let line_char_counts: Vec<usize> = source
        .split('\n')
        .map(|line| line.chars().count())
        .collect();
    let tokens = tokenize_raw(source);
    for token in &tokens {
        let bound = line_char_counts[token.line - 1] + 1;
        assert!(
            (1..=bound).contains(&token.column),
            "column {} outside [1, {bound}] on line {} of {source:?} - token {:?}",
            token.column,
            token.line,
            token.kind,
        );
        let text = match &token.kind {
            TokenKind::Word(text) | TokenKind::Punct(text) => text,
        };
        assert_eq!(
            token.end - token.start,
            text.chars().count(),
            "char span end - start must equal chars().count() for {:?} in {source:?}",
            token.kind,
        );
    }
}

/// Renders the full token stream (kind/line/column/start/end) as one line
/// per token - the shape the ASCII-parity baseline locks byte-for-byte.
fn render_token_stream(source: &str) -> String {
    tokenize_raw(source)
        .iter()
        .map(|token| {
            format!(
                "{:?}|{}|{}|{}|{}",
                token.kind, token.line, token.column, token.start, token.end
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

// Baselines recorded from the PRE-FIX code via the Gate-2 RED capture
// (target/testrun-capture.txt) - characterization values, quirky ones
// included (the string-literal column lag mirrors tokenizer/
// characterization_tests.rs). `test_ascii_path_tokenization_unchanged`
// passes at Red by design: it locks what the fix must not change.
const ASCII_BASELINE_CALL: &str = "Word(\"int\")|1|1|0|3\nWord(\"f\")|1|5|4|5\nPunct(\"(\")|1|6|5|6\nPunct(\")\")|1|7|6|7\nPunct(\";\")|1|8|7|8";
const ASCII_BASELINE_COMMENT_AND_STRING: &str = "Word(\"a1\")|1|1|0|2\nWord(\"b2\")|1|4|3|5\nWord(\"x\")|2|1|12|13\nPunct(\"=\")|2|3|14|15\nPunct(\"+\")|2|9|20|21\nPunct(\";\")|2|14|25|26";
const ASCII_BASELINE_TWO_FUNCTIONS: &str = "Word(\"int\")|1|1|0|3\nWord(\"f\")|1|5|4|5\nPunct(\"(\")|1|6|5|6\nPunct(\")\")|1|7|6|7\nPunct(\";\")|1|8|7|8\nWord(\"int\")|2|1|9|12\nWord(\"g\")|2|5|13|14\nPunct(\"(\")|2|6|14|15\nPunct(\")\")|2|7|15|16\nPunct(\";\")|2|8|16|17";

#[test]
fn test_e3_punct_shape_no_panic_and_bounded_columns() {
    // Single line, punct run at the line start carrying the U+00AB
    // guillemet named by the handoff, driven by the three-byte U+20AC
    // panic driver at column 1 (the committed T0b `punct_run.cpp` shape).
    // Pre-fix DEBUG: panic `attempt to subtract with overflow` at
    // tokenizer.rs:331:21 (the E3 site). Pre-fix RELEASE: the euro
    // token's column wraps to ~usize::MAX - the bound assert below fires.
    // (A bare two-byte guillemet alone floors the column at 0 - bounded
    // failure, never panic - so the three-byte driver is what pins the
    // :331 panic class on this unit source; T0b journal, Gate 1.)
    let source = specimen(&[EURO, GUILLEMET, b"x = 1;"]);
    assert_token_columns_bounded(&source);
}

#[test]
fn test_word_shape_no_panic_and_bounded_columns() {
    // Word run at column 1 carrying TWO U+00E9 pairs (9 chars / 11 bytes,
    // char-counted column of 10 at flush - the committed T0b
    // `word_run.cpp` shape). Pre-fix DEBUG: panic `subtract with
    // overflow` at tokenizer.rs:311 (one flush earlier than the E3 site,
    // same class); pre-fix RELEASE: the word's column wraps huge.
    let source = specimen(&[b"caf", E_ACUTE, b"_caf", E_ACUTE, b" = 2;"]);
    assert_token_columns_bounded(&source);
}

#[test]
fn test_mixed_multibyte_matrix_bounded() {
    // Small matrix per I14's falsifiable statement, one line per shape:
    // leading/middle/trailing multi-byte in word runs and in punct runs,
    // adjacent to line/block comments and to a string path. Pre-fix the
    // two-byte rows floor their column at 0 (bound violation in both
    // profiles) and the three-byte rows panic in debug / wrap in
    // release; post-fix every line is bounded. The comment and string
    // rows pass pre-fix too - they lock the class ABSENT on those paths
    // (comment/string content never reaches a flush subtraction).
    let matrix_lines: Vec<String> = vec![
        specimen(&[E_ACUTE, b" = 1;"]),
        specimen(&[b"ab", E_ACUTE, b"cd = 2;"]),
        specimen(&[b"abc", E_ACUTE, b" = 3;"]),
        specimen(&[GUILLEMET, b" = 4;"]),
        specimen(&[b"x = 1 ", GUILLEMET, b" 2;"]),
        specimen(&[CJK_MID, b"ab = 5;"]),
        specimen(&[EURO, b"x = 6;"]),
        specimen(&[E_ACUTE, b" = 7; // comment with ", E_ACUTE]),
        specimen(&[b"/* comment with ", E_ACUTE, b" */ x;"]),
        specimen(&[b"const char* s = \"", E_ACUTE, b"\"; t = 1;"]),
    ];
    for line in &matrix_lines {
        assert_token_columns_bounded(line);
    }
}

#[test]
fn test_ascii_path_tokenization_unchanged() {
    // I1 promise: only NON-ASCII column values are corrected. For ASCII
    // runs `run.len() == run.chars().count()`, so the fixed
    // `column - len` evaluates identically to the old byte-length
    // subtraction; the token stream (kind/line/column/start/end) must
    // byte-equal the pre-fix baseline recorded in this test. Passes at
    // Red by design (it locks what must NOT change; the Red failure
    // comes from the three multibyte tests, not this one).
    let ascii_sources: [&str; 3] = [
        "int f();",
        "a1 b2 // c3\nx = \"q\" + 'z';\n",
        "int f();\nint g();\n",
    ];
    let expected_streams: [&str; 3] = [
        ASCII_BASELINE_CALL,
        ASCII_BASELINE_COMMENT_AND_STRING,
        ASCII_BASELINE_TWO_FUNCTIONS,
    ];
    for (source, expected) in ascii_sources.iter().zip(expected_streams.iter()) {
        assert_eq!(
            render_token_stream(source),
            *expected,
            "ASCII token stream drifted for {source:?} (I1: ASCII-path output must not change)"
        );
    }
}
