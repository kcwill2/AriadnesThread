// Characterization tests: they lock the CURRENT behavior of `tokenize` and
// `ParserCtx::handle_global_function` before a later pass decomposes them.
// Several asserts below deliberately encode quirky-but-current behavior;
// if one fails, the code changed — that is a finding, not something to "fix" here.

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
fn tokenize_word_and_punct_run_locks_one_based_line_and_column() {
    assert_eq!(
        tokenize("int f();"),
        vec![
            Token {
                kind: TokenKind::Word("int".into()),
                line: 1,
                column: 1
            },
            Token {
                kind: TokenKind::Word("f".into()),
                line: 1,
                column: 5
            },
            Token {
                kind: TokenKind::Punct("(".into()),
                line: 1,
                column: 6
            },
            Token {
                kind: TokenKind::Punct(")".into()),
                line: 1,
                column: 7
            },
            Token {
                kind: TokenKind::Punct(";".into()),
                line: 1,
                column: 8
            },
        ]
    );
}

#[test]
fn tokenize_splits_multi_char_operators_into_single_char_puncts() {
    expect_token_kinds(
        "x == 42 >= y",
        &[
            "word:x", "punct:=", "punct:=", "word:42", "punct:>", "punct:=", "word:y",
        ],
    );
}

#[test]
fn tokenize_string_literal_with_braces_and_comment_openers_yields_no_string_token() {
    expect_token_kinds(
        "x = \"}{ /* // */\"; y();",
        &[
            "word:x", "punct:=", "punct:;", "word:y", "punct:(", "punct:)", "punct:;",
        ],
    );
}

#[test]
fn tokenize_char_literal_is_swallowed_like_a_string_literal() {
    expect_token_kinds(
        "char c = '}';",
        &["word:char", "word:c", "punct:=", "punct:;"],
    );
}

#[test]
fn tokenize_escaped_quote_does_not_terminate_string_early() {
    expect_token_kinds(
        "x = \"a\\\"b\"; f();",
        &[
            "word:x", "punct:=", "punct:;", "word:f", "punct:(", "punct:)", "punct:;",
        ],
    );
}

#[test]
fn tokenize_string_spanning_newline_advances_line_number_and_hides_braces() {
    assert_eq!(
        tokenize("a \"x\n} y\" b"),
        vec![
            Token {
                kind: TokenKind::Word("a".into()),
                line: 1,
                column: 1
            },
            Token {
                kind: TokenKind::Word("b".into()),
                line: 2,
                column: 6
            },
        ]
    );
}

#[test]
fn tokenize_unterminated_string_literal_swallows_rest_of_file() {
    expect_token_kinds("x = \"{", &["word:x", "punct:="]);
}

#[test]
fn tokenize_unterminated_block_comment_drops_rest_of_file() {
    expect_token_kinds("a /* b } c", &["word:a"]);
}

#[test]
fn tokenize_block_comment_hides_braces_until_terminator() {
    expect_token_kinds("a /* b } */ c", &["word:a", "word:c"]);
}

#[test]
fn tokenize_line_comment_skips_to_newline_without_emitting_tokens() {
    assert_eq!(
        tokenize("// }\n}"),
        vec![Token {
            kind: TokenKind::Punct("}".into()),
            line: 2,
            column: 1
        }]
    );
    expect_token_kinds("a } // trailing", &["word:a", "punct:}"]);
}

#[test]
fn tokenize_line_comment_with_trailing_backslash_does_not_continue_to_next_line() {
    assert_eq!(
        tokenize("a // b \\\n}"),
        vec![
            Token {
                kind: TokenKind::Word("a".into()),
                line: 1,
                column: 1
            },
            Token {
                kind: TokenKind::Punct("}".into()),
                line: 2,
                column: 1
            },
        ]
    );
}

#[test]
fn tokenize_stray_block_comment_terminator_emits_two_puncts() {
    expect_token_kinds("a */ b", &["word:a", "punct:*", "punct:/", "word:b"]);
}

#[test]
fn tokenize_lone_slash_is_punct_when_not_followed_by_comment_opener() {
    expect_token_kinds("a / b", &["word:a", "punct:/", "word:b"]);
    expect_token_kinds("a /", &["word:a", "punct:/"]);
}

#[test]
fn tokenize_crlf_input_produces_identical_tokens_to_lf_input() {
    assert_eq!(
        tokenize("int f();\r\nint g();\r\n"),
        tokenize("int f();\nint g();\n")
    );
}

#[test]
fn tokenize_tab_counts_as_one_column() {
    assert_eq!(
        tokenize("\tfoo"),
        vec![Token {
            kind: TokenKind::Word("foo".into()),
            line: 1,
            column: 2
        }]
    );
}

#[test]
fn tokenize_nested_template_angle_brackets_split_into_two_gt_puncts() {
    expect_token_kinds(
        "std::vector<std::pair<int,char>>",
        &[
            "word:std",
            "punct::",
            "punct::",
            "word:vector",
            "punct:<",
            "word:std",
            "punct::",
            "punct::",
            "word:pair",
            "punct:<",
            "word:int",
            "punct:,",
            "word:char",
            "punct:>",
            "punct:>",
        ],
    );
}

#[test]
fn tokenize_nested_namespace_braces_produce_separate_puncts() {
    expect_token_kinds(
        "namespace a { namespace b { } }",
        &[
            "word:namespace",
            "word:a",
            "punct:{",
            "word:namespace",
            "word:b",
            "punct:{",
            "punct:}",
            "punct:}",
        ],
    );
}

#[test]
fn tokenize_macro_call_keeps_macro_name_and_parens_as_separate_tokens() {
    expect_token_kinds(
        "MACRO(func())",
        &[
            "word:MACRO",
            "punct:(",
            "word:func",
            "punct:(",
            "punct:)",
            "punct:)",
        ],
    );
}

#[test]
fn tokenize_bang_and_double_ampersand_split_into_single_char_puncts() {
    expect_token_kinds(
        "DCHECK(!x) && y != z;",
        &[
            "word:DCHECK",
            "punct:(",
            "punct:!",
            "word:x",
            "punct:)",
            "punct:&",
            "punct:&",
            "word:y",
            "punct:!",
            "punct:=",
            "word:z",
            "punct:;",
        ],
    );
}

#[test]
fn tokenize_preprocessor_include_lines_keep_angle_brackets_but_swallow_quoted_paths() {
    expect_token_kinds(
        "#include <vector>",
        &[
            "punct:#",
            "word:include",
            "punct:<",
            "word:vector",
            "punct:>",
        ],
    );
    expect_token_kinds("#include \"q.h\"", &["punct:#", "word:include"]);
}

#[test]
fn tokenize_numbers_split_on_dot_and_digit_separators_swallow_groups() {
    expect_token_kinds(
        "f = 1.5 + 1'000'000 + 0x1F;",
        &[
            "word:f",
            "punct:=",
            "word:1",
            "punct:.",
            "word:5",
            "punct:+",
            "word:1",
            "word:000",
            "punct:+",
            "word:0x1F",
            "punct:;",
        ],
    );
}
