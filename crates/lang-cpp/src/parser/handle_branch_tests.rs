// UTA-T2 pin locks for the ParserCtx handle_* branches (handoff items 2, 3,
// 8, 9, 13). Pins assert CURRENT behavior; none of these drove a src change.
// Existing suites (characterization_tests, brace_depth_tests) stay untouched.

use super::*;
use crate::analyzer::CppSemanticAnalyzer;

fn analyze(source: &str) -> TuAnalysis {
    CppSemanticAnalyzer::analyze_source(FileId::from_relative_path("uta-t2.cpp"), source, "Debug|x64")
}

fn edge_kinds<'a>(analysis: &'a TuAnalysis, src: &str, dst: &str) -> Vec<&'a DependencyKind> {
    analysis
        .edges
        .iter()
        .filter(|edge| edge.src_symbol.0 == src && edge.dst_symbol.0 == dst)
        .map(|edge| &edge.kind)
        .collect()
}

fn edge_pairs(analysis: &TuAnalysis) -> Vec<(&str, &str, &DependencyKind)> {
    analysis
        .edges
        .iter()
        .map(|edge| (edge.src_symbol.0.as_str(), edge.dst_symbol.0.as_str(), &edge.kind))
        .collect()
}

fn declared_symbols(analysis: &TuAnalysis) -> Vec<&str> {
    analysis
        .declarations
        .iter()
        .map(|decl| decl.symbol_id.0.as_str())
        .collect()
}

// - item 2: handle_new ----------------------------------------------------

#[test]
fn pin_handle_new_from_class_body_adds_trailing_complete_edge() {
    // `B* p` contributes the Incomplete edge first; `new B` adds the
    // Complete one. Order is the token order, so the vector pins both.
    let analysis = analyze("class A { B* p = new B; };");
    assert_eq!(
        edge_kinds(&analysis, "A", "B"),
        vec![&DependencyKind::Incomplete, &DependencyKind::Complete],
        "the `new B` site contributes exactly one trailing Complete A->B edge"
    );
}

#[test]
fn pin_trailing_new_at_eof_completes_without_panic() {
    // Completion is the assertion; additionally the dangling `new` must not
    // forge any edge (no token follows it, so handle_new returns early).
    let analysis = analyze("class A { B* p; new");
    assert_eq!(
        edge_pairs(&analysis),
        vec![("A", "B", &DependencyKind::Incomplete)],
        "the recorded B* edge survives and the EOF-trailing `new` adds nothing"
    );
}

// - item 3: handle_call_site reserved words --------------------------------

#[test]
fn pin_call_site_reserved_words_only_z_is_a_call() {
    let analysis = analyze("void f(){ if(a) while(b) z(); }");
    let callees: Vec<&str> = analysis
        .call_sites
        .iter()
        .map(|site| site.callee_name.as_str())
        .collect();
    assert_eq!(callees, vec!["z"], "if/while are reserved; only z() lands");
    assert_eq!(analysis.call_sites[0].caller_symbol.0, "f");
}

// - item 8: handle_class_or_struct ------------------------------------------

#[test]
fn pin_class_at_stream_end_early_returns() {
    let analysis = analyze("class");
    assert!(analysis.symbols.is_empty(), "bare `class` at EOF records nothing");
    assert!(analysis.declarations.is_empty());
    assert!(analysis.edges.is_empty());
}

#[test]
fn pin_class_colon_public_base_records_complete_edge_and_definition() {
    let analysis = analyze("class A : public B;");
    assert_eq!(analysis.symbols.len(), 1, "only A becomes a symbol");
    assert_eq!(analysis.symbols[0].id.0, "A");
    assert_eq!(analysis.symbols[0].kind, SymbolKind::Class);
    assert_eq!(
        edge_pairs(&analysis),
        vec![("A", "B", &DependencyKind::Complete)],
        "the base scan stops at the first non-visibility word"
    );
}

#[test]
fn pin_class_colon_visibility_only_base_scans_to_eof_without_edge() {
    // `: public` with EOF before any base word: the scan walks to the end,
    // records no edge, and the class is still recorded as a definition.
    let analysis = analyze("class A : public");
    assert_eq!(analysis.symbols.len(), 1);
    assert_eq!(analysis.symbols[0].id.0, "A");
    assert!(analysis.symbols[0].definition.is_some(), "the `:` arm marks is_definition");
    assert!(analysis.edges.is_empty(), "visibility-only run yields no base edge");
}

#[test]
fn pin_anonymous_class_opener_records_no_symbol_without_panic() {
    let analysis = analyze("class { int x; };");
    assert!(analysis.symbols.is_empty(), "`class {{` has no name token to record");
    assert!(analysis.declarations.is_empty());
    assert!(analysis.edges.is_empty(), "the nameless body has no parent to edge from");
}

// - item 9: handle_variable_decl ---------------------------------------------

#[test]
fn pin_unique_ptr_member_records_incomplete_edge_alone() {
    let analysis = analyze("class A { unique_ptr<B> p; };");
    assert_eq!(
        edge_kinds(&analysis, "A", "B"),
        vec![&DependencyKind::Incomplete],
        "the unique_ptr branch fires once and nothing else edges out of A"
    );
}

#[test]
fn pin_ctor_shape_variable_decl_is_incomplete() {
    // `A a();` double-fires: the type-name word gives the Incomplete
    // Holder->A variable-decl edge, and the `a(` word is separately seen as
    // a member-signature start whose return-type edge is Holder::a -> A
    // (report-flagged quirk: a ctor-shaped line yields a phantom member
    // function name). Pinned exactly as produced, both Incomplete.
    let analysis = analyze("class Holder { A a(); };");
    assert_eq!(
        edge_pairs(&analysis),
        vec![
            ("Holder", "A", &DependencyKind::Incomplete),
            ("Holder::a", "A", &DependencyKind::Incomplete),
        ]
    );
}

// - item 13: stream-start handling --------------------------------------------

#[test]
fn pin_stream_start_double_colon_call_records_bare_symbol() {
    // `::f()` at token 0: qualified_function_name requires i >= 3 to fold
    // the qualifier, so the symbol lands under the bare name `f`. The
    // recorded self-edge f->f is the handle_variable_decl `(` branch firing
    // after current_function was just set (pre-existing quirk, report-flagged).
    let analysis = analyze("::f() { return; }");
    assert_eq!(declared_symbols(&analysis), vec!["f"]);
    assert_eq!(analysis.symbols[0].kind, SymbolKind::Function);
    assert!(analysis.symbols[0].definition.is_some());
    assert_eq!(
        edge_pairs(&analysis),
        vec![("f", "f", &DependencyKind::Complete)],
        "KNOWN QUIRK (report-only): the just-set current_function makes the
         definition name its own variable-decl parent at i > 0"
    );
}

#[test]
fn pin_i_zero_return_type_guard_does_not_panic() {
    // `f(int x);` puts the name at index 0: the `self.i > 0` guard in
    // add_return_type_edge prevents the tokens[i - 1] access. Only the
    // parameter edge lands.
    let analysis = analyze("f(int x);");
    assert_eq!(declared_symbols(&analysis), vec!["f"]);
    assert_eq!(
        edge_pairs(&analysis),
        vec![("f", "int", &DependencyKind::Incomplete)],
        "no return-type edge at i == 0; the int parameter edge is present"
    );
}

#[test]
fn pin_sizeof_at_top_level_function_produces_zero_edges() {
    // handle_sizeof needs a current_class owner; at function scope (no
    // enclosing class) it records nothing.
    let analysis = analyze("void probe() { sizeof(B); }");
    assert!(analysis.edges.is_empty(), "sizeof without a class context edges nowhere");
}
