// UTA2-R5 row 2: multibyte-adjacent raw-string pins + the raw_string_open
// leading guard. Authored pure ASCII: the non-ASCII specimen char is built
// from byte escapes (0xC3 0xA9 = U+00E9 LATIN SMALL LETTER E WITH ACUTE),
// never from a literal, per the repo UTF-8 hazard rule.

use super::*;

fn e_acute() -> String {
    String::from_utf8(vec![0xC3, 0xA9]).expect("0xC3 0xA9 is a valid two-byte U+00E9")
}

fn token_kinds(source: &str) -> Vec<String> {
    tokenize(source)
        .iter()
        .map(|token| match &token.kind {
            TokenKind::Word(word) => format!("word:{word}"),
            TokenKind::Punct(punct) => format!("punct:{punct}"),
        })
        .collect()
}

#[test]
fn raw_string_open_reads_byte_offsets_through_a_multibyte_prefix() {
    // Specimen `...e-acute...R"( { } )" tail`: the two-byte e-acute shifts byte offsets
    // one ahead of char indexes, exactly the confusion class the byte-level
    // equivalence argument (tokenizer.rs:322-325) rules out.
    let accent = e_acute();
    let source = format!("...{accent}R\"( {{ }} )\" tail");
    let quote_byte = source.find("R\"").expect("the R-quote is present") + 1;
    assert!(
        raw_string_open(&source, quote_byte),
        "the R-quote opens a raw string with a multi-byte char upstream"
    );
    assert!(
        !raw_string_open(&source, quote_byte - 1),
        "the byte just before the quote (the R) is not itself a raw open"
    );

    // Multi-byte char glued directly in front of the R: e-acute occupies bytes
    // 0..2, so a char-indexed probe would read the wrong byte for `bytes[q-1]`.
    let glued = format!("{accent}R\"( }} )\" z");
    let glued_quote = glued.find("R\"").expect("the R-quote is present") + 1;
    assert!(
        raw_string_open(&glued, glued_quote),
        "continuation byte 0xA9 upstream must not disturb the byte checks"
    );

    // Leading guard: a quote at byte 0 has no predecessor - the guard must
    // answer false instead of indexing bytes[-1].
    assert!(
        !raw_string_open("\"abc", 0),
        "quote_byte 0 must return false, never underflow"
    );
}

#[test]
fn multibyte_before_raw_string_leaks_no_brace_tokens_and_keeps_char_columns() {
    let accent = e_acute();
    let source = format!("...{accent}...R\"( {{ }} )\" tail");
    let kinds = token_kinds(&source);
    let accent_word = format!("word:{accent}");
    let actual_refs: Vec<&str> = kinds.iter().map(String::as_str).collect();
    assert_eq!(
        actual_refs,
        vec![
            "punct:.",
            "punct:.",
            "punct:.",
            accent_word.as_str(),
            "punct:.",
            "punct:.",
            "punct:.",
            "word:tail",
        ],
        "the raw content ` {{ }} ` must emit no tokens at all"
    );
    assert!(
        !kinds.iter().any(|kind| kind.contains('{') || kind.contains('}')),
        "no brace may leak: {kinds:?}"
    );

    // Exact position of the token after the raw string. Char view: the raw
    // run `R"( { } )"` covers columns 8..=17, the separator space is 18, so
    // `tail` sits at COLUMN 19 - a byte-counted column would say 20 because
    // of e-acute's two bytes. Line stays 1.
    let tokens = tokenize(&source);
    let trailing = tokens
        .iter()
        .find(|token| matches!(&token.kind, TokenKind::Word(word) if word == "tail"))
        .expect("word tail after the raw string");
    assert_eq!(trailing.line, 1, "line must stay 1 through the raw string");
    assert_eq!(
        trailing.column, 19,
        "column must be CHAR-counted: the e-acute contributes one column, not two"
    );
}

#[test]
fn glued_multibyte_prefix_run_is_dropped_into_the_raw_string_wholesale() {
    // `e-acuteR"( } )" z`: e-acute is alphanumeric, so the pending run is `e-acuteR` when the
    // raw opens; scan_quote drops the whole pending run (doc tokenizer.rs:
    // 207-208: the trailing R is the raw prefix, not a word). Honest pin of
    // the documented drop rule - the e-acute disappears with the R.
    let accent = e_acute();
    let source = format!("{accent}R\"( }} )\" z");
    assert_eq!(token_kinds(&source), vec!["word:z".to_string()]);
    let tokens = tokenize(&source);
    let only = tokens.first().expect("exactly one token");
    assert_eq!(only.line, 1);
    assert_eq!(
        only.column, 11,
        "char view counts the two-byte acute as one column: the tail sits at 11"
    );
}

#[test]
fn multiline_multibyte_raw_string_keeps_line_and_char_columns() {
    // e-acute on the raw's opening line: after the terminator the counter must
    // carry line 2 and the char column the content ends at, byte-escaping
    // nothing into the following token's position.
    let accent = e_acute();
    let source = format!("a{accent} R\"( }}\n{{ )\" tail");
    assert_eq!(
        token_kinds(&source),
        vec![format!("word:a{accent}"), "word:tail".to_string()],
        "the newline inside the raw content must not split or leak tokens"
    );
    let tokens = tokenize(&source);
    let trailing = tokens
        .iter()
        .find(|token| matches!(&token.kind, TokenKind::Word(word) if word == "tail"))
        .expect("word tail after the raw string");
    assert_eq!(trailing.line, 2, "the raw content newline advances the line");
    assert_eq!(trailing.column, 6, "char-view column after the terminator + space");
}

#[test]
fn header_guard_of_scan_raw_string_with_fallback_falls_back_to_the_plain_swallow() {
    // The handoff marks this guard REPORT-only "unless directly testable".
    // It IS directly reachable from this inline module (the only production
    // route pre-validates the header via raw_string_open, making the guard a
    // deliberate paranoia check), so it gets a pin instead of a report: the
    // owner's debug_assert-vs-delete decision stays open, but whichever way
    // it lands the fallback behavior below must not silently change.
    //
    // Feed the guard a shape raw_string_open would REJECT (`R"x"` - a
    // delimiter char then a quote, no `(`): the content-opener check trips
    // and the call must route into the plain scan_string_literal swallow from
    // the quote position, byte/char/line/column exactly as the fallback owns.
    let source = "a R\"x\" b";
    let quote_byte = source.find("R\"").expect("the R-quote is present") + 1;
    let mut byte = quote_byte;
    let mut char_index = quote_byte;
    let mut line = 1usize;
    let mut column = quote_byte + 1;
    scan_raw_string_with_fallback(&source, quote_byte, &mut byte, &mut char_index, &mut line, &mut column);
    assert_eq!(
        (byte, char_index, line, column),
        (6, 6, 1, 7),
        "the guard must swallow exactly `\"x\"` (quote, delimiter char, quote) \
         and leave the cursor on the space before b"
    );
}
