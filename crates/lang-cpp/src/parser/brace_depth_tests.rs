// Brace-depth floor locks for the cpp-brace-depth-underflow fix (spec
// I1/I2/I5). Adversarial token streams must degrade to defined data, never
// to `attempt to subtract with overflow` (debug) nor to the release wraparound
// where depth becomes usize::MAX and every later scope-exit check passes,
// silently dropping symbols and call sites for the rest of the TU.
// The characterization suite stays UNEDITED; these new locks live here.

use super::*;
use crate::analyzer::CppSemanticAnalyzer;
use crate::tokenizer::tokenize;

fn analyze_never_panics(source: &str) {
    // The I1 assertion is the completion itself: any brace-depth underflow
    // panics with "attempt to subtract with overflow" in this debug
    // (overflow-checks on) test run before analyze_source can return.
    CppSemanticAnalyzer::analyze_source(
        FileId::from_relative_path("adversarial.cpp"),
        source,
        "Debug|x64",
    );
}

#[test]
fn stray_closes_only_input_completes_without_underflow() {
    analyze_never_panics("}}}");
}

#[test]
fn open_then_two_closes_completes_without_underflow() {
    analyze_never_panics("{}}");
}

#[test]
fn api_cpp_style_raw_string_with_stray_close_in_function_body_completes() {
    // The API.cpp:155 construct wrapped in a function body; with the lexer
    // fix the raw is consumed as a unit, the floor bounds any residual drift.
    analyze_never_panics("void f() { char* s = \"a\" + R\"(\"})\" ; }");
}

#[test]
fn unterminated_raw_string_at_eof_completes_without_underflow() {
    analyze_never_panics("void f() { char* s = R\"( }{");
}

#[test]
fn hash_if_zero_dead_branch_stray_close_completes_without_underflow() {
    // The undecidable-at-lexer residual class (spec 2.3): a dead `#if 0`
    // branch may carry a brace-unbalanced body the tokenizer cannot discard.
    analyze_never_panics("void a() { }\n#if 0\n}\n#endif\nvoid b() { c(); }");
}

#[test]
fn stray_apostrophe_swallow_fragment_completes_without_underflow() {
    analyze_never_panics("char c = '; } void d() { e(); }");
}

#[test]
fn stray_close_then_next_function_definition_resumes_scoping() {
    // I2/I5: after a stray close, depth stays 0 and the NEXT definition still
    // records its body and call sites - defined degradation, not poisoning.
    let analysis = CppSemanticAnalyzer::analyze_source(
        FileId::from_relative_path("stray.cpp"),
        "} void f() { g(); }",
        "Debug|x64",
    );
    let function_decl = analysis
        .declarations
        .iter()
        .find(|decl| decl.symbol_id.0 == "f")
        .expect("f must be recorded after the stray close");
    assert!(
        function_decl.is_definition,
        "f must be recorded as a definition, not just a declaration"
    );
    let resumed_call = analysis
        .call_sites
        .iter()
        .find(|site| site.caller_symbol.0 == "f" && site.callee_name == "g");
    assert!(
        resumed_call.is_some(),
        "the g() call site inside f must survive the stray close (I5)"
    );
}

#[test]
fn stray_closes_at_zero_depth_keep_bookkeeping_usable() {
    // Direct brace-bookkeeping pin, mirroring the
    // parser/characterization_tests.rs:251 direct-call pattern.
    let tokens = tokenize("void later() { g(); }");
    let mut ctx = ParserCtx::new(FileId::from_relative_path("floor-direct.cpp"), &tokens);
    ctx.handle_brace_close();
    ctx.handle_brace_close();
    assert_eq!(
        ctx.brace_depth, 0,
        "stray closes must leave depth at exactly zero (no wraparound)"
    );

    ctx.handle_brace_open();
    let name_index = tokens
        .iter()
        .position(|token| matches!(&token.kind, TokenKind::Word(word) if word == "later"))
        .expect("word later");
    ctx.i = name_index;
    let name_token = ctx.tokens[name_index].clone();
    ctx.handle_global_function("later", &name_token);
    let later_decl = ctx
        .declarations
        .iter()
        .find(|decl| decl.symbol_id.0 == "later")
        .expect("later must be recorded after the strays");
    assert!(later_decl.is_definition, "later must be a definition");
    assert_eq!(
        ctx.current_function
            .as_ref()
            .map(|(id, depth)| (id.0.clone(), *depth)),
        Some(("later".to_string(), 2)),
        "definition depth recorded as brace_depth + 1 from the floored state"
    );

    let call_index = tokens
        .iter()
        .position(|token| matches!(&token.kind, TokenKind::Word(word) if word == "g"))
        .expect("word g");
    ctx.i = call_index;
    let call_token = ctx.tokens[call_index].clone();
    ctx.handle_word("g", &call_token);
    assert!(
        ctx.call_sites
            .iter()
            .any(|site| site.caller_symbol.0 == "later" && site.callee_name == "g"),
        "the g() call site must be recorded from the floored state"
    );

    ctx.handle_brace_close();
    assert!(
        ctx.current_function.is_none(),
        "the body close must end the function again"
    );
}
