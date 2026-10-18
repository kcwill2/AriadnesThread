// UTA-T2 pin locks for literal-swallow position bookkeeping (handoff item
// 10): the unterminated char literal that crosses newlines, and the
// raw-string fallback path with a multi-byte char and a newline. Pins
// assert CURRENT behavior; none drove a src change.
//
// Authored pure ASCII: every multi-byte / quote specimen enters the source
// via its stored bytes (the underflow_fix_tests.rs idiom), never a literal.

use super::*;

/// Concatenates byte parts and decodes them, so quote marks and multi-byte
/// chars arrive through stored UTF-8 bytes only.
fn specimen(parts: &[&[u8]]) -> String {
    let mut bytes: Vec<u8> = Vec::new();
    for part in parts {
        bytes.extend_from_slice(part);
    }
    String::from_utf8(bytes).expect("specimen parts are single-encoded UTF-8")
}

const APOSTROPHE: &[u8] = b"\x27";
const E_ACUTE: &[u8] = b"\xc3\xa9";

fn rendered_tokens(source: &str) -> Vec<(String, usize, usize)> {
    tokenize_raw(source)
        .into_iter()
        .map(|token| {
            let text = match token.kind {
                TokenKind::Word(word) => word,
                TokenKind::Punct(punct) => punct,
            };
            (text, token.line, token.column)
        })
        .collect()
}

#[test]
fn pin_unterminated_char_literal_swallows_newlines_and_brace() {
    // char c = 'x            <- literal 1 opens, never closes on its line
    // };                      <- swallowed whole (the brace must not emit)
    // char d = 'y';          <- literal 1 closes at this opening quote;
    // }                        `y` tokenizes at (3, 11), then literal 2
    //                          opens at the closing quote and eats to EOF
    let source = specimen(&[
        b"char c = ",
        APOSTROPHE,
        b"x\n};\nchar d = ",
        APOSTROPHE,
        b"y",
        APOSTROPHE,
        b"\n}\n",
    ]);
    assert_eq!(
        rendered_tokens(&source),
        vec![
            ("char".to_string(), 1, 1),
            ("c".to_string(), 1, 6),
            ("=".to_string(), 1, 8),
            ("y".to_string(), 3, 11),
        ],
        "both braces are inside swallowed literals; y's char-based column is 11"
    );
}

#[test]
fn pin_raw_string_fallback_counts_multibyte_and_newline_as_chars() {
    // auto raw = R"del(multi   <- raw opens, terminator `)del"` never occurs
    // éend" ;                   <- fallback stops at this unescaped quote;
    // int tail = 3;                `;` lands at (2, 7): the 2-byte é
    //                              advanced the column by ONE
    let source = specimen(&[
        b"auto raw = R\"del(multi\n",
        E_ACUTE,
        b"end\" ;\nint tail = 3;\n",
    ]);
    assert_eq!(
        rendered_tokens(&source),
        vec![
            ("auto".to_string(), 1, 1),
            ("raw".to_string(), 1, 6),
            ("=".to_string(), 1, 10),
            (";".to_string(), 2, 7),
            ("int".to_string(), 3, 1),
            ("tail".to_string(), 3, 5),
            ("=".to_string(), 3, 10),
            ("3".to_string(), 3, 12),
            (";".to_string(), 3, 13),
        ],
        "fallback position bookkeeping is char-based across the newline"
    );
}
