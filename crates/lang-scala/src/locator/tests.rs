//! Byte-exact locks for the Scala verbatim region locator (plan M23, T5).
//!
//! Every assert pins the exact sliced bytes `locate_bodies` returns: brace
//! and braceless regions, suffix matching, strict kinds, comment/string
//! invisibility, first-matchable-wins, and the omission-on-unresolved
//! contract the engine's placeholder fallback (I4) builds on.

use super::{locate_bodies, package_namespace};
use ariadne_lang::SymbolKind;

fn bodies(source: &str, wanted: &[(&str, SymbolKind)]) -> Vec<(String, String)> {
    let wanted: Vec<(String, SymbolKind)> = wanted
        .iter()
        .map(|(name, kind)| (name.to_string(), *kind))
        .collect();
    locate_bodies(source, &wanted)
        .into_iter()
        .map(|body| (body.name, body.body))
        .collect()
}

#[test]
fn brace_matched_class_re_slices_byte_verbatim() {
    let source = "package com.p\n\nclass Widget {\n  val size = 1\n}";
    assert_eq!(
        bodies(source, &[("com.p.Widget", SymbolKind::Class)]),
        vec![(
            "com.p.Widget".to_string(),
            "class Widget {\n  val size = 1\n}".to_string()
        )],
        "region = declaration text only: no package line, no trailing newline"
    );
}

#[test]
fn braceless_regions_end_at_the_boundary_rule() {
    let table: Vec<(&str, &str, &str, SymbolKind, &str)> = vec![
        (
            "braceless def",
            "package p\n\ndef greet(): String = \"hi\"\n\nval limit = 4",
            "greet",
            SymbolKind::Function,
            "def greet(): String = \"hi\"",
        ),
        (
            "braceless val",
            "package p\n\nval limit = 4\n\ndef greet(): Unit = ()",
            "limit",
            SymbolKind::Variable,
            "val limit = 4",
        ),
        (
            "braceless trait stops before modifiers",
            "package p\n\nsealed trait Shape\nfinal case class Box(x: Int)",
            "p.Shape",
            SymbolKind::Trait,
            "sealed trait Shape",
        ),
        (
            "braceless at EOF",
            "package p\n\nclass Done\nobject Later {\n}",
            "Done",
            SymbolKind::Class,
            "class Done",
        ),
    ];
    for (label, source, name, kind, expected) in table {
        assert_eq!(
            bodies(source, &[(name, kind)]),
            vec![(name.to_string(), expected.to_string())],
            "{label}: braceless region is the declaration text up to the \
             next top-level boundary, with no trailing whitespace"
        );
    }
}

#[test]
fn nested_braces_balance_and_string_braces_are_invisible() {
    let source = "package p\n\nclass A {\n  def f(): Unit = { val s = \"} {\" }\n}\nclass B";
    assert_eq!(
        bodies(source, &[("A", SymbolKind::Class)]),
        vec![(
            "A".to_string(),
            "class A {\n  def f(): Unit = { val s = \"} {\" }\n}".to_string()
        )],
        "the region stops at the real closing brace, not the string's"
    );
}

#[test]
fn crlf_source_slices_the_declaration_with_its_crlf_verbatim() {
    let source = "package p\r\n\r\nclass W {\r\n  val x = 1\r\n}\r\n";
    assert_eq!(
        bodies(source, &[("W", SymbolKind::Class)]),
        vec![("W".to_string(), "class W {\r\n  val x = 1\r\n}".to_string())],
        "char offsets are taken on the raw String, CRLF included"
    );
}

#[test]
fn comment_and_string_occurrences_never_locate_a_region() {
    let commented = "// class Ghost {}\n/* class AlsoGhost {} */\npackage p\n\nclass Real";
    assert_eq!(
        bodies(commented, &[("Ghost", SymbolKind::Class)]),
        Vec::<(String, String)>::new(),
        "a commented-out declaration cannot be located (parser suppression)"
    );
    assert_eq!(
        bodies(commented, &[("p.Real", SymbolKind::Class)]).len(),
        1,
        "the real declaration after the comments still locates"
    );
    let quoted = "package p\n\nclass Real {\n  val s = \"class Ghost {}\"\n}";
    assert_eq!(
        bodies(quoted, &[("Ghost", SymbolKind::Class)]),
        Vec::<(String, String)>::new(),
        "a class quoted inside a string literal cannot be located"
    );
}

#[test]
fn doc_comment_is_excluded_and_region_starts_at_the_declaration() {
    let source = "/** doc for Documented */\nclass Documented {\n}";
    assert_eq!(
        bodies(source, &[("Documented", SymbolKind::Class)]),
        vec![("Documented".to_string(), "class Documented {\n}".to_string())],
        "LOCK: doc comments above a declaration are NOT part of its region \
         (comments are invisible tokens; the slice starts at the first \
         modifier/keyword token — T6's removal hook owns doc-comment \
         adjacency when moving code)"
    );
}

#[test]
fn qualified_suffix_matching_follows_the_cpp_discipline() {
    let source = "package com.p\n\nclass Widget {\n}";
    let region = "class Widget {\n}";
    assert_eq!(
        bodies(source, &[("Widget", SymbolKind::Class)]),
        vec![("Widget".to_string(), region.to_string())],
        "simple wanted is a segment-suffix of the qualified id"
    );
    assert_eq!(
        bodies(source, &[("p.Widget", SymbolKind::Class)]),
        vec![("p.Widget".to_string(), region.to_string())],
        "shorter prefix spelling still suffix-matches"
    );
    assert_eq!(
        bodies(source, &[("com.p.Widget", SymbolKind::Class)]).len(),
        1,
        "exact qualified match"
    );
    assert_eq!(
        bodies(source, &[("other.Widget", SymbolKind::Class)]),
        Vec::<(String, String)>::new(),
        "same simple name under a different qualification must not match"
    );
    assert_eq!(
        bodies(source, &[("Widget.com", SymbolKind::Class)]),
        Vec::<(String, String)>::new(),
        "suffixes run left-to-right only"
    );
}

#[test]
fn kind_mismatch_is_strict_and_companion_objects_need_their_dollar_id() {
    let source = "package p\n\nclass Counter\nobject Counter";
    assert_eq!(
        bodies(source, &[("p.Counter", SymbolKind::Class)]),
        vec![("p.Counter".to_string(), "class Counter".to_string())],
        "the class answers a Class wanted"
    );
    assert_eq!(
        bodies(source, &[("Counter", SymbolKind::Object)]),
        Vec::<(String, String)>::new(),
        "LOCK: strict kinds — a bare-name Object wanted never answers a \
         companion class, and the `X$` companion object is reached through \
         its own D17 qualified id"
    );
    assert_eq!(
        bodies(source, &[("p.Counter", SymbolKind::Object)]),
        Vec::<(String, String)>::new(),
        "the companion object's id is `p.Counter$`, not `p.Counter`"
    );
    assert_eq!(
        bodies(source, &[("p.Counter$", SymbolKind::Object)]),
        vec![("p.Counter$".to_string(), "object Counter".to_string())],
        "the companion object locates through its D17 id"
    );
}

#[test]
fn lone_object_matches_by_simple_and_qualified_name() {
    let source = "package p\n\nobject Main {\n  def go(): Unit = {}\n}";
    assert_eq!(
        bodies(source, &[("Main", SymbolKind::Object)]).len(),
        1,
        "suffix match on the object's `p.Main` id"
    );
    assert_eq!(
        bodies(source, &[("p.Main", SymbolKind::Class)]),
        Vec::<(String, String)>::new(),
        "kind Class does not answer an object"
    );
}

#[test]
fn duplicate_declarations_resolve_the_first_matchable_occurrence() {
    let source = "package p\n\nclass Dup {\n  val first = 1\n}\nclass Dup {\n  val second = 2\n}";
    assert_eq!(
        bodies(source, &[("Dup", SymbolKind::Class)]),
        vec![(
            "Dup".to_string(),
            "class Dup {\n  val first = 1\n}".to_string()
        )],
        "source-order first wins, mirroring the Gate G1 F5 lock on the C++ side"
    );
}

#[test]
fn package_object_region_carries_its_namespace_id() {
    let source = "package com.example\n\npackage object models {\n  val x = 1\n}";
    assert_eq!(
        bodies(source, &[("models", SymbolKind::Object)]),
        vec![(
            "models".to_string(),
            "package object models {\n  val x = 1\n}".to_string()
        )],
        "the package object's qualified id is `<ns>.models`"
    );
}

#[test]
fn multiple_wanted_names_echo_wanted_and_keep_wanted_order() {
    let source = "package p\n\nclass A {\n}\n\nclass B {\n}";
    let result = bodies(
        source,
        &[("p.B", SymbolKind::Class), ("A", SymbolKind::Class)],
    );
    assert_eq!(
        result,
        vec![
            ("p.B".to_string(), "class B {\n}".to_string()),
            ("A".to_string(), "class A {\n}".to_string()),
        ],
        "result order follows wanted, names echo the wanted spelling (engine \
         maps located bodies back by name)"
    );
}

#[test]
fn unresolved_wanted_names_are_omitted() {
    let source = "package p\n\nclass Real {\n}";
    assert_eq!(
        bodies(
            source,
            &[
                ("Ghost", SymbolKind::Class),
                ("p.Real", SymbolKind::Class),
                ("p.Gone", SymbolKind::Function),
            ]
        ),
        vec![("p.Real".to_string(), "class Real {\n}".to_string())],
        "unlocatable names are simply absent (I4 placeholder contract)"
    );
}

#[test]
fn block_and_label_package_files_locate_nothing_and_have_no_namespace() {
    let block = "package outer {\n  class Hidden\n}";
    assert_eq!(
        bodies(block, &[("Hidden", SymbolKind::Class)]),
        Vec::<(String, String)>::new(),
        "block-package contents are consumed whole by the parser"
    );
    assert_eq!(package_namespace(block), "");
    let label = "package outer:\nclass Hidden";
    assert_eq!(
        bodies(label, &[("outer.Hidden", SymbolKind::Class)]),
        Vec::<(String, String)>::new(),
        "Scala-3 label-package contents are consumed whole (EOF)"
    );
    assert_eq!(package_namespace(label), "");
}

#[test]
fn namespace_re_derivation_matches_the_parser_joining_rules() {
    let table: Vec<(&str, &str, &str)> = vec![
        ("single", "package a.b\n\nclass C", "a.b"),
        ("nested clauses join", "package a\npackage b.c\n\nclass C", "a.b.c"),
        ("comments around package", "// c\npackage p\n// d\n\nclass C", "p"),
        ("no package", "class C", ""),
        ("package object adds nothing", "package com.example\n\npackage object models {}", "com.example"),
        ("block package yields empty", "package outer {\nclass Hidden\n}\nclass Real", ""),
        ("label package yields empty", "package outer:\nclass Hidden", ""),
        ("dotted accessor is no clause", "package a\n\nval q = x.package", "a"),
    ];
    for (label, source, expected) in table {
        assert_eq!(
            package_namespace(source),
            expected,
            "{label}: unit_namespace must equal DirectiveScan.namespace \
             exactly (analysis-time qualification parity, D-M23-2)"
        );
    }
}
