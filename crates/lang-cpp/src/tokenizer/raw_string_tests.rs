// Raw-string lexing locks for the cpp-brace-depth-underflow fix (spec I4).
// The characterization suite is a frozen snapshot of pre-fix quirks and stays
// UNEDITED; the new raw-string behavior is pinned here instead.

use super::*;

fn token_kinds(source: &str) -> Vec<String> {
    tokenize(source)
        .iter()
        .map(|token| match &token.kind {
            TokenKind::Word(word) => format!("word:{word}"),
            TokenKind::Punct(punct) => format!("punct:{punct}"),
        })
        .collect()
}

fn expect_token_kinds(source: &str, expected: &[&str]) {
    let actual = token_kinds(source);
    let actual_refs: Vec<&str> = actual.iter().map(String::as_str).collect();
    assert_eq!(actual_refs, expected, "token kinds for {source:?}");
}

#[test]
fn raw_string_braces_emit_no_tokens() {
    expect_token_kinds("a R\"( }{ )\" b", &["word:a", "word:b"]);
}

#[test]
fn delimited_raw_string_braces_emit_no_tokens() {
    expect_token_kinds("a R\"d( } )d\" b", &["word:a", "word:b"]);
    // The `R"DOC(...)DOC"` documentation-raw class (clickhouse TableFunctions
    // and friends): a quoted phrase inside the content must not break pairing.
    expect_token_kinds(
        "a R\"DOC({ \"quoted\" } )DOC\" b",
        &["word:a", "word:b"],
    );
}

#[test]
fn multi_line_raw_string_advances_line_counter() {
    let source = "a R\"( }\n{\n )\" b";
    let physical_line_of_b = source.matches('\n').count() + 1;
    assert_eq!(token_kinds(source), vec!["word:a", "word:b"]);
    let tokens = tokenize(source);
    let trailing_word = tokens
        .iter()
        .find(|token| matches!(&token.kind, TokenKind::Word(word) if word == "b"))
        .expect("word b after the raw string");
    assert_eq!(
        trailing_word.line, physical_line_of_b,
        "b must sit on the physical line {physical_line_of_b}, content lines must not leak"
    );
}

#[test]
fn unterminated_raw_string_swallows_to_eof() {
    // Mirrors the locked unterminated-string behavior
    // (tokenizer/characterization_tests.rs:116): an unterminated raw swallows
    // the rest of the file and emits no tokens itself.
    expect_token_kinds("a R\"( }{", &["word:a"]);
}

#[test]
fn raw_quote_without_delim_falls_back_to_string_scan() {
    // `R"` without a legal delimiter+`(` header is not a raw string: the
    // ordinary string scan keeps its pre-fix swallow (fallback lock - green
    // before and after the fix on purpose).
    expect_token_kinds("a R\"x\" b", &["word:a", "word:R", "word:b"]);
    expect_token_kinds("a R\"\" b", &["word:a", "word:R", "word:b"]);
}

#[test]
fn api_cpp_line155_reproduction_lexes_without_brace_leak() {
    // Reproduction of the clickhouse `src/Server/ACME/API.cpp:155` construct
    // `+ R"("})"` (encoded as a fixture literal, corpus file not read).
    // Pre-fix, the content `}` leaked as a `punct:}` token with no `{`
    // partner - the exact source of the brace-depth underflow.
    expect_token_kinds(
        "f(x + R\"(\"})\" );",
        &[
            "word:f",
            "punct:(",
            "word:x",
            "punct:+",
            "punct:)",
            "punct:;",
        ],
    );
}
