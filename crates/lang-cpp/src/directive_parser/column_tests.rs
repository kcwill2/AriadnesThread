//! UTA-T2 item 14: exact `column` values for include directives.
//! The column is 1-based and anchored at the FIRST delimiter char found on
//! the line (`line.find('"') + 1` / `line.find('<') + 1`), tabs included as
//! single columns.

use super::parse_include_directives;

#[test]
fn pin_tabbed_angled_directive_column() {
    // `#`1 TAB2 i3 n4 c5 l6 u7 d8 e9 TAB10 <11
    let directives = parse_include_directives("#\tinclude\t<windows.h>\n");
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].column, 11);
    assert_eq!(directives[0].raw_path, "windows.h");
    assert!(directives[0].is_angled);
}

#[test]
fn pin_spaced_angled_directive_column() {
    // `#`1 SP2 SP3 i4 n5 c6 l7 u8 d9 e10 SP11 <12
    let directives = parse_include_directives("#  include <vector>\n");
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].column, 12);
    assert_eq!(directives[0].raw_path, "vector");
}

#[test]
fn pin_quoted_directive_column() {
    // `#`1 i2 n3 c4 l5 u6 d7 e8 SP9 "10
    let directives = parse_include_directives("#include \"heavy.h\" // comment\n");
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].column, 10);
    assert_eq!(directives[0].raw_path, "heavy.h");
    assert!(!directives[0].is_angled);
}

#[test]
fn pin_tabbed_quoted_directive_column() {
    // `#`1 TAB2 i3 n4 c5 l6 u7 d8 e9 TAB10 "11
    let directives = parse_include_directives("#\tinclude\t\"tabbed.h\"\n");
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].column, 11);
    assert_eq!(directives[0].raw_path, "tabbed.h");
}
