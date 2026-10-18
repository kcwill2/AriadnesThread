// UTA-T2 pin lock for sanitize_symbol_stem (handoff item 15): the
// filesystem-safe stem fall-throughs. The frozen planning.rs test module
// stays untouched; child-module access reaches the private stem fn.

use super::sanitize_symbol_stem;

#[test]
fn pin_empty_and_punctuation_only_names_fall_back_to_symbol() {
    assert_eq!(sanitize_symbol_stem(""), "symbol");
    assert_eq!(sanitize_symbol_stem("..."), "symbol");
    assert_eq!(sanitize_symbol_stem("-!-"), "symbol");
}

#[test]
fn pin_non_ascii_bytes_are_never_kept_in_the_stem() {
    // U+00DC (bytes C3 9C) and U+00E9 (bytes C3 A9), constructed from their
    // stored bytes so this file stays pure ASCII. A leading non-ASCII char
    // leaves a pending underscore that trim_matches strips; a trailing one
    // is dropped entirely (pending is never flushed).
    let u_umlaut = std::str::from_utf8(b"\xc3\x9c").expect("valid UTF-8");
    let e_acute = std::str::from_utf8(b"\xc3\xa9").expect("valid UTF-8");
    assert_eq!(
        sanitize_symbol_stem(&format!("{u_umlaut}ber")),
        "ber",
        "non-ASCII lead char is replaced by the separator, then trimmed away"
    );
    assert_eq!(
        sanitize_symbol_stem(&format!("Caf{e_acute}")),
        "Caf",
        "non-ASCII tail char is dropped with its unflushed pending separator"
    );
    assert_eq!(sanitize_symbol_stem(u_umlaut), "symbol");
}
