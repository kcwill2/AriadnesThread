//! Byte-exact locks for the verbatim declaration locator (plan M23, T1).
//!
//! Every assert pins the exact sliced bytes returned by
//! [`crate::symbol_locator::locate`]: construct coverage, comment/string/
//! `#if 0` invisibility, qualified-name suffix matching, and the
//! omission-on-unresolved contract the engine's placeholder fallback (I4)
//! builds on.

use super::locate;
use crate::tokenizer::{tokenize_raw, TokenKind};
use ariadne_lang::SymbolKind;

fn kinds(pairs: &[(&str, SymbolKind)]) -> Vec<(String, SymbolKind)> {
    pairs
        .iter()
        .map(|(name, kind)| (name.to_string(), *kind))
        .collect()
}

fn bodies(source: &str, wanted: &[(&str, SymbolKind)]) -> Vec<(String, String)> {
    locate(source, &kinds(wanted))
        .into_iter()
        .map(|body| (body.name, body.body))
        .collect()
}

#[test]
fn class_slice_runs_through_closing_brace_and_semicolon() {
    let source = "class Widget {\n    int a;\n};\n";
    assert_eq!(
        bodies(source, &[("Widget", SymbolKind::Class)]),
        vec![("Widget".to_string(), "class Widget {\n    int a;\n};".to_string())]
    );
}

#[test]
fn struct_slice_is_byte_exact() {
    let source = "#pragma once\n\nstruct ThingConfig {\n    int id;\n    const char* name;\n};\n";
    assert_eq!(
        bodies(source, &[("ThingConfig", SymbolKind::Struct)]),
        vec![(
            "ThingConfig".to_string(),
            "struct ThingConfig {\n    int id;\n    const char* name;\n};".to_string()
        )]
    );
}

#[test]
fn class_with_member_declarations_resolves_class_and_both_members() {
    let source = "class Thing {\npublic:\n    explicit Thing(ThingConfig config);\n    int get_id() const;\n\nprivate:\n    ThingConfig config_;\n};\n";
    let found = bodies(
        source,
        &[
            ("Thing", SymbolKind::Class),
            ("Thing::Thing", SymbolKind::Function),
            ("Thing::get_id", SymbolKind::Function),
        ],
    );
    assert_eq!(
        found,
        vec![
            (
                "Thing".to_string(),
                "class Thing {\npublic:\n    explicit Thing(ThingConfig config);\n    int get_id() const;\n\nprivate:\n    ThingConfig config_;\n};".to_string()
            ),
            (
                "Thing::Thing".to_string(),
                "explicit Thing(ThingConfig config);".to_string()
            ),
            (
                "Thing::get_id".to_string(),
                "int get_id() const;".to_string()
            ),
        ],
        "results follow the wanted order"
    );
}

#[test]
fn inline_function_definition_ends_at_matching_brace_without_semicolon() {
    let source = "void use_heavy() {\n    heavy_func();\n}\n";
    assert_eq!(
        bodies(source, &[("use_heavy", SymbolKind::Function)]),
        vec![(
            "use_heavy".to_string(),
            "void use_heavy() {\n    heavy_func();\n}".to_string()
        )]
    );
}

#[test]
fn template_prefix_is_part_of_the_slice() {
    let source = "template <typename T>\nstruct Box {\n    T value;\n};\n";
    assert_eq!(
        bodies(source, &[("Box", SymbolKind::Struct)]),
        vec![(
            "Box".to_string(),
            "template <typename T>\nstruct Box {\n    T value;\n};".to_string()
        )]
    );
}

#[test]
fn typedef_and_using_slices_terminate_at_semicolon() {
    let source = "typedef unsigned long ulong_type;\nusing counter_t = unsigned int;\n";
    let found = bodies(
        source,
        &[
            ("ulong_type", SymbolKind::TypeAlias),
            ("counter_t", SymbolKind::TypeAlias),
        ],
    );
    assert_eq!(
        found,
        vec![
            (
                "ulong_type".to_string(),
                "typedef unsigned long ulong_type;".to_string()
            ),
            (
                "counter_t".to_string(),
                "using counter_t = unsigned int;".to_string()
            ),
        ]
    );
}

#[test]
fn commented_out_duplicate_does_not_shadow_the_real_declaration() {
    let source = "// class Ghost { int fake; };\n/* class Ghost { int also_fake; } */\nclass Ghost {\n    int real;\n};\n";
    assert_eq!(
        bodies(source, &[("Ghost", SymbolKind::Class)]),
        vec![(
            "Ghost".to_string(),
            "class Ghost {\n    int real;\n};".to_string()
        )]
    );
}

#[test]
fn name_inside_string_literal_is_neither_matched_nor_matched_from() {
    let source = "const char* note = \"class Fake { };\";\nclass Ghost {\n    int x;\n};\n";
    let found = bodies(
        source,
        &[
            ("Fake", SymbolKind::Class),
            ("Ghost", SymbolKind::Class),
        ],
    );
    assert_eq!(
        found,
        vec![(
            "Ghost".to_string(),
            "class Ghost {\n    int x;\n};".to_string()
        )],
        "\"Fake\" appears only inside a string literal and is omitted"
    );
}

#[test]
fn declaration_hidden_by_if_zero_is_not_matched() {
    let source = "#if 0\nclass Dead {\n    int x;\n};\n#endif\nclass Dead {\n    int live;\n};\n";
    assert_eq!(
        bodies(source, &[("Dead", SymbolKind::Class)]),
        vec![(
            "Dead".to_string(),
            "class Dead {\n    int live;\n};".to_string()
        )]
    );
}

#[test]
fn unresolved_name_is_omitted_from_the_result() {
    let source = "class Widget {\n    int a;\n};\n";
    assert!(bodies(
        source,
        &[
            ("Missing", SymbolKind::Class),
            ("Widget", SymbolKind::Class),
        ]
    )
    .iter()
    .all(|(name, _)| name != "Missing"));
}

#[test]
fn unqualified_wanted_matches_a_namespace_nested_declaration() {
    let source = "namespace gfx {\nclass Canvas {\n    int w;\n};\n}\n";
    assert_eq!(
        bodies(source, &[("Canvas", SymbolKind::Class)]),
        vec![(
            "Canvas".to_string(),
            "class Canvas {\n    int w;\n};".to_string()
        )],
        "the slice starts at the declaration keyword, the namespace wrapper is not included"
    );
}

#[test]
fn qualified_wanted_matches_qualified_source_and_suffix_wanted_shares_the_slice() {
    let source = "int Helper::compute(int a) {\n    return a;\n}\n";
    let full = "int Helper::compute(int a) {\n    return a;\n}";
    assert_eq!(
        bodies(source, &[("Helper::compute", SymbolKind::Function)]),
        vec![("Helper::compute".to_string(), full.to_string())]
    );
    assert_eq!(
        bodies(source, &[("compute", SymbolKind::Function)]),
        vec![("compute".to_string(), full.to_string())],
        "the wanted suffix may be shorter than the source qualification"
    );
}

#[test]
fn forward_declaration_is_skipped_in_favor_of_the_definition() {
    let source = "class Widget;\nclass Widget {\n    int x;\n};\n";
    assert_eq!(
        bodies(source, &[("Widget", SymbolKind::Class)]),
        vec![(
            "Widget".to_string(),
            "class Widget {\n    int x;\n};".to_string()
        )]
    );
}

#[test]
fn same_suffix_with_different_qualification_does_not_match() {
    let source = "int Other::get_id() const {\n    return 0;\n}\n";
    assert_eq!(
        bodies(source, &[("Thing::get_id", SymbolKind::Function)]),
        Vec::<(String, String)>::new(),
        "get_id under a different qualifier must not answer a Thing::get_id lookup"
    );
}

#[test]
fn duplicate_definitions_resolve_the_first_matchable_occurrence() {
    // Gate G1 (F5): every locate path scans tokens in ascending order and
    // returns the first candidate whose body scan succeeds, so two
    // identically-matchable definitions of one name resolve to the first
    // (forward declarations are candidates that fail the body scan, hence
    // skipped — see `scan_record_body`).
    let cases: Vec<(&str, &str, &str, SymbolKind, &str)> = vec![
        (
            "class duplicates keep the first",
            "class Dup { int first; };\nclass Dup { int second; };\n",
            "Dup",
            SymbolKind::Class,
            "class Dup { int first; };",
        ),
        (
            "struct duplicates keep the first",
            "struct Dup { int first; };\nstruct Dup { int second; };\n",
            "Dup",
            SymbolKind::Struct,
            "struct Dup { int first; };",
        ),
        (
            "enum duplicates keep the first",
            "enum Dup { kFirst };\nenum Dup { kSecond };\n",
            "Dup",
            SymbolKind::Enum,
            "enum Dup { kFirst };",
        ),
        (
            "function duplicates keep the first",
            "int dup() { return 1; }\nint dup() { return 2; }\n",
            "dup",
            SymbolKind::Function,
            "int dup() { return 1; }",
        ),
        (
            "using-alias duplicates keep the first",
            "using Dup = int;\nusing Dup = long;\n",
            "Dup",
            SymbolKind::TypeAlias,
            "using Dup = int;",
        ),
        (
            "unmatchable first candidate is skipped",
            "class Dup;\nclass Dup { int second; };\n",
            "Dup",
            SymbolKind::Class,
            "class Dup { int second; };",
        ),
    ];
    for (label, source, wanted, kind, expected) in cases {
        assert_eq!(
            bodies(source, &[(wanted, kind)]),
            vec![(wanted.to_string(), expected.to_string())],
            "{label}"
        );
    }
}

#[test]
fn multiple_wanted_names_resolve_in_wanted_order() {
    let source = "class A { int a; };\nclass B { int b; };\n";
    let found = bodies(
        source,
        &[
            ("B", SymbolKind::Class),
            ("A", SymbolKind::Class),
        ],
    );
    assert_eq!(
        found,
        vec![
            ("B".to_string(), "class B { int b; };".to_string()),
            ("A".to_string(), "class A { int a; };".to_string()),
        ]
    );
}

#[test]
fn enum_class_slice_is_byte_exact() {
    let source = "enum class Color {\n    Red,\n    Green,\n};\n";
    assert_eq!(
        bodies(source, &[("Color", SymbolKind::Enum)]),
        vec![(
            "Color".to_string(),
            "enum class Color {\n    Red,\n    Green,\n};".to_string()
        )]
    );
}

#[test]
fn pure_virtual_declaration_slice_includes_the_initializer() {
    let source = "class Runner {\n    virtual void run() = 0;\n};\n";
    assert_eq!(
        bodies(source, &[("run", SymbolKind::Function)]),
        vec![(
            "run".to_string(),
            "virtual void run() = 0;".to_string()
        )]
    );
}

#[test]
fn call_sites_are_not_matched_as_function_declarations() {
    let source = "void driver() {\n    helper();\n    helper(5);\n}\n";
    let found = bodies(
        source,
        &[
            ("driver", SymbolKind::Function),
            ("helper", SymbolKind::Function),
        ],
    );
    assert_eq!(
        found,
        vec![(
            "driver".to_string(),
            "void driver() {\n    helper();\n    helper(5);\n}".to_string()
        )],
        "helper appears only as call sites and is omitted"
    );
}

#[test]
fn declaration_after_preprocessor_directive_does_not_swallow_the_directive() {
    let source = "#include <vector>\nint make();\n";
    assert_eq!(
        bodies(source, &[("make", SymbolKind::Function)]),
        vec![("make".to_string(), "int make();".to_string())]
    );
}

#[test]
fn prefixless_constructor_declaration_in_class_body_is_matched_for_qualified_wanted() {
    let source = "class Thing {\n    Thing(int id);\n};\n";
    assert_eq!(
        bodies(source, &[("Thing::Thing", SymbolKind::Function)]),
        vec![(
            "Thing::Thing".to_string(),
            "Thing(int id);".to_string()
        )]
    );
}

#[test]
fn raw_token_offsets_re_slice_their_own_lexemes_byte_verbatim() {
    let source = "// comment } {\nclass A { const char* s = \"( { )\"; } /* x */ int y;\n";
    let chars: Vec<char> = source.chars().collect();
    for raw in tokenize_raw(source) {
        let lexeme: String = chars[raw.start..raw.end].iter().collect();
        let expected = match &raw.kind {
            TokenKind::Word(word) => word.clone(),
            TokenKind::Punct(punct) => punct.clone(),
        };
        assert_eq!(
            lexeme, expected,
            "offsets {:?}..{:?} must re-slice the lexeme",
            raw.start, raw.end
        );
    }
}
