// UTA-T2 pin locks for symbol_locator branch shapes (handoff items 5, 6, 7):
// masked_region outcomes via byte-exact locate(), extend_with_template
// depth handling, and match_group failure omissions. Pins assert CURRENT
// behavior; none drove a src change. The frozen tests.rs suite stays untouched.

use super::locate;
use ariadne_lang::SymbolKind;

fn bodies(source: &str, wanted: &[(&str, SymbolKind)]) -> Vec<(String, String)> {
    locate(
        source,
        &wanted
            .iter()
            .map(|(name, kind)| (name.to_string(), *kind))
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|body| (body.name, body.body))
    .collect()
}

const NO_BODIES: Vec<(String, String)> = Vec::new();

// - item 5: masked_region outcomes ---------------------------------------------

#[test]
fn pin_if_zero_else_masks_both_branches() {
    // `#else` of a disabled `#if 0` re-pushes the parent's disabled state,
    // so today NEITHER branch is locatable. Conservative (compiler would
    // see the #else branch) - pinned as-is, report-flagged for owner.
    let source = "#if 0\nclass A {\n    int a;\n};\n#else\nclass A2 {\n    int b;\n};\n#endif\n";
    assert_eq!(bodies(source, &[("A", SymbolKind::Class)]), NO_BODIES);
    assert_eq!(bodies(source, &[("A2", SymbolKind::Class)]), NO_BODIES);
}

#[test]
fn pin_enabled_if_else_exposes_both_visibility_rules() {
    // `#if 1` pushes the visible state, so its `#else` branch stays visible:
    // A2 is locatable byte-exactly.
    let source = "#if 1\nclass A {\n    int a;\n};\n#else\nclass A2 {\n    int b;\n};\n#endif\n";
    assert_eq!(
        bodies(source, &[("A", SymbolKind::Class)]),
        vec![("A".to_string(), "class A {\n    int a;\n};".to_string())]
    );
    assert_eq!(
        bodies(source, &[("A2", SymbolKind::Class)]),
        vec![("A2".to_string(), "class A2 {\n    int b;\n};".to_string())]
    );
}

#[test]
fn pin_elif_branch_is_masked_sibling_survives() {
    let source = "#if 1\nclass First {\n    int x;\n};\n#elif OTHER\nclass Second {\n    int y;\n};\n#endif\n";
    assert_eq!(
        bodies(
            source,
            &[("First", SymbolKind::Class), ("Second", SymbolKind::Class)]
        ),
        vec![("First".to_string(), "class First {\n    int x;\n};".to_string())],
        "the #elif branch masks unconditionally (conservative)"
    );
}

#[test]
fn pin_nested_if_zero_inside_if_one_masks_only_the_dead_block() {
    let source = "#if 1\nclass Live {\n    int a;\n};\n#if 0\nclass Dead {\n    int d;\n};\n#endif\nclass Later {\n    int l;\n};\n#endif\n";
    assert_eq!(
        bodies(
            source,
            &[
                ("Live", SymbolKind::Class),
                ("Dead", SymbolKind::Class),
                ("Later", SymbolKind::Class),
            ]
        ),
        vec![
            ("Live".to_string(), "class Live {\n    int a;\n};".to_string()),
            ("Later".to_string(), "class Later {\n    int l;\n};".to_string()),
        ],
        "results follow wanted order with Dead omitted"
    );
}

#[test]
fn pin_unmatched_endif_pops_harmlessly_and_region_locates() {
    let source = "#endif\nclass A {\n    int x;\n};\n";
    assert_eq!(
        bodies(source, &[("A", SymbolKind::Class)]),
        vec![("A".to_string(), "class A {\n    int x;\n};".to_string())],
        "a lone #endif (pop on an empty stack) must not poison the file"
    );
}

// - item 6: extend_with_template ----------------------------------------------

#[test]
fn pin_nested_angle_depth_above_one_extends_over_template_prefix() {
    let source = "template <template <typename U> class T>\nusing Holder = Box<T>;\n";
    assert_eq!(
        bodies(source, &[("Holder", SymbolKind::TypeAlias)]),
        vec![(
            "Holder".to_string(),
            "template <template <typename U> class T>\nusing Holder = Box<T>;".to_string()
        )],
        "depth 2 `<>` nesting must still find the leading `template` word"
    );
}

#[test]
fn pin_function_template_slice_starts_at_template_keyword() {
    let source = "template <typename T>\nT make() {\n    return T();\n}\n";
    assert_eq!(
        bodies(source, &[("make", SymbolKind::Function)]),
        vec![(
            "make".to_string(),
            "template <typename T>\nT make() {\n    return T();\n}".to_string()
        )]
    );
}

#[test]
fn pin_two_parameter_template_record_is_byte_exact() {
    let source = "template <class A, class B>\nstruct Pair {\n    int x;\n};\n";
    assert_eq!(
        bodies(source, &[("Pair", SymbolKind::Class)]),
        vec![(
            "Pair".to_string(),
            "template <class A, class B>\nstruct Pair {\n    int x;\n};".to_string()
        )],
        "the template-internal `class` keyword must not hijack record matching"
    );
}

// - item 7: match_group failure omits the candidate ----------------------------

#[test]
fn pin_unclosed_paren_candidate_is_omitted() {
    let source = "void broken( { x(); }\n";
    assert_eq!(bodies(source, &[("broken", SymbolKind::Function)]), NO_BODIES);
}

#[test]
fn pin_unclosed_brace_candidate_is_omitted() {
    let source = "void open() { x();\n";
    assert_eq!(bodies(source, &[("open", SymbolKind::Function)]), NO_BODIES);
}
