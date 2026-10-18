//! UTA-T2 item 1 (audit UTA-B3): word-boundary locks on the `include`
//! directive keyword.
//!
//! Audit claim: `starts_with("include")` at `directive_parser.rs:24` accepts
//! keyword runs longer than `include`. Handoff red specimens: `#includemy
//! "a.h"` and `#include_next <x>` must yield 0 directives; ordinary quoted
//! and angled includes must keep parsing.

use super::parse_include_directives;

#[test]
fn includemy_prefixed_keyword_yields_no_directive() {
    // UTA-B3 red specimen: `#includemy "a.h"` must never parse as a real
    // include (phantom dependency edge class).
    assert_eq!(parse_include_directives("#includemy \"a.h\"\n").len(), 0);
}

#[test]
fn include_next_yields_no_directive() {
    // Report-flagged: `#include_next <x>` is not an `#include` directive
    // for this parser (GCC-only extension; silently dropped).
    assert_eq!(parse_include_directives("#include_next <x>\n").len(), 0);
}

#[test]
fn normal_quoted_and_angled_includes_still_parse() {
    // Control for the boundary rule: the chars that may legally follow the
    // keyword (space, tab, `"`/`<` directly) keep working, including the
    // no-separator `#include"tight.h"` shape.
    let content = "#include \"ok.h\"\n#include <vec>\n#include\"tight.h\"\n";
    let directives = parse_include_directives(content);
    assert_eq!(directives.len(), 3);
    assert_eq!(directives[0].raw_path, "ok.h");
    assert!(!directives[0].is_angled);
    assert_eq!(directives[1].raw_path, "vec");
    assert!(directives[1].is_angled);
    assert_eq!(directives[2].raw_path, "tight.h");
    assert!(!directives[2].is_angled);
}
