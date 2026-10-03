// Characterization tests: they lock the CURRENT behavior of `tokenize` and
// `ParserCtx::handle_global_function` before a later pass decomposes them.
// Several asserts below deliberately encode quirky-but-current behavior;
// if one fails, the code changed — that is a finding, not something to "fix" here.

use super::*;
use crate::analyzer::CppSemanticAnalyzer;
use crate::tokenizer::tokenize;

struct GlobalFunctionOutcome {
    symbols: Vec<SymbolRecord>,
    declarations: Vec<SymbolDecl>,
    edges: Vec<SemanticEdge>,
    opened_function: Option<(String, usize)>,
}

fn analyze_global_function(
    source: &str,
    target: &str,
    enclosing_class: Option<&str>,
) -> GlobalFunctionOutcome {
    let tokens = tokenize(source);
    let mut ctx = ParserCtx::new(FileId::from_relative_path("characterization.cpp"), &tokens);
    if let Some(class_name) = enclosing_class {
        ctx.current_class = Some((class_name.to_string(), ctx.brace_depth + 1));
    }
    let target_index = tokens
        .iter()
        .position(|token| matches!(&token.kind, TokenKind::Word(word) if word.as_str() == target))
        .unwrap_or_else(|| panic!("expected a Word token '{target}' in {source:?}"));
    ctx.i = target_index;
    let target_token = ctx.tokens[target_index].clone();
    ctx.handle_global_function(target, &target_token);
    GlobalFunctionOutcome {
        symbols: ctx.symbols,
        declarations: ctx.declarations,
        edges: ctx.edges,
        opened_function: ctx
            .current_function
            .map(|(symbol_id, depth)| (symbol_id.0, depth)),
    }
}

fn edge_targets(edges: &[SemanticEdge], source_name: &str) -> Vec<String> {
    edges
        .iter()
        .filter(|edge| edge.src_symbol.0 == source_name)
        .map(|edge| edge.dst_symbol.0.clone())
        .collect()
}

#[test]
fn handle_global_function_simple_declaration_locks_symbol_location_and_return_edge() {
    let outcome = analyze_global_function("int f();", "f", None);
    assert_eq!(outcome.symbols.len(), 1);
    assert_eq!(outcome.symbols[0].id.0, "f");
    assert_eq!(outcome.symbols[0].kind, SymbolKind::Function);
    assert_eq!(outcome.declarations.len(), 1);
    assert!(!outcome.declarations[0].is_definition);
    assert_eq!(
        (
            outcome.declarations[0].location.line,
            outcome.declarations[0].location.column
        ),
        (1, 5)
    );
    assert_eq!(edge_targets(&outcome.edges, "f"), vec!["int"]);
    assert_eq!(outcome.edges[0].kind, DependencyKind::Incomplete);
    assert!(outcome.opened_function.is_none());
}

#[test]
fn handle_global_function_records_location_from_multiline_input() {
    let outcome = analyze_global_function("\n  int located();", "located", None);
    assert_eq!(
        (
            outcome.declarations[0].location.line,
            outcome.declarations[0].location.column
        ),
        (2, 7)
    );
}

#[test]
fn handle_global_function_semicolon_is_declaration_but_brace_opens_definition() {
    let declaration = analyze_global_function("void d();", "d", None);
    assert!(!declaration.declarations[0].is_definition);
    assert!(declaration.opened_function.is_none());
    assert!(
        declaration.edges.is_empty(),
        "void return type is excluded from edges"
    );

    let definition = analyze_global_function("void e() {}", "e", None);
    assert!(definition.declarations[0].is_definition);
    assert_eq!(
        definition.opened_function,
        Some(("e".to_string(), 1)),
        "definition records (symbol, brace_depth + 1)"
    );
    assert!(definition.edges.is_empty());
}

#[test]
fn handle_global_function_reserved_keywords_never_record_a_declaration() {
    for keyword in [
        "if",
        "while",
        "for",
        "switch",
        "return",
        "sizeof",
        "catch",
        "new",
        "delete",
        "throw",
        "class",
        "struct",
        "namespace",
        "else",
    ] {
        let outcome = analyze_global_function(&format!("{keyword}(x);"), keyword, None);
        assert!(
            outcome.symbols.is_empty() && outcome.declarations.is_empty(),
            "reserved keyword {keyword} must not record a function declaration"
        );
    }
}

#[test]
fn handle_global_function_qualified_name_prefixes_class_but_drops_return_type_edge() {
    let outcome = analyze_global_function("B MyClass::method() {}", "method", None);
    assert_eq!(outcome.symbols[0].id.0, "MyClass::method");
    assert!(outcome.declarations[0].is_definition);
    assert_eq!(
        outcome.opened_function,
        Some(("MyClass::method".to_string(), 1))
    );
    assert!(
        outcome.edges.is_empty(),
        "quirk: return type sits before the :: puncts, so the B->edge is dropped"
    );
}

#[test]
fn handle_global_function_inside_class_scope_prefixes_class_name() {
    let outcome = analyze_global_function("int m();", "m", Some("K"));
    assert_eq!(outcome.symbols[0].id.0, "K::m");
    assert_eq!(edge_targets(&outcome.edges, "K::m"), vec!["int"]);
}

#[test]
fn handle_global_function_pointer_and_reference_returns_drop_return_type_edge() {
    let pointer = analyze_global_function("B* make_b();", "make_b", None);
    assert_eq!(pointer.symbols[0].id.0, "make_b");
    assert!(
        pointer.edges.is_empty(),
        "quirk: tokens[i - 1] is Punct(*), so no return-type edge is emitted"
    );

    let reference = analyze_global_function("C& take_c();", "take_c", None);
    assert_eq!(reference.symbols[0].id.0, "take_c");
    assert!(reference.edges.is_empty());
}

#[test]
fn handle_global_function_named_parameters_emit_incomplete_edges() {
    let outcome = analyze_global_function("void take(A a, B* b, const C& c);", "take", None);
    assert_eq!(edge_targets(&outcome.edges, "take"), vec!["A", "B", "C"]);
    assert!(
        outcome
            .edges
            .iter()
            .all(|edge| edge.kind == DependencyKind::Incomplete),
        "all parameter edges are Incomplete"
    );
}

#[test]
fn handle_global_function_unnamed_parameter_emits_no_edge() {
    let unnamed = analyze_global_function("void one(int);", "one", None);
    assert!(
        unnamed.edges.is_empty(),
        "quirk: `int` followed by `)` fails the followed-by-Word test, so no edge"
    );

    let named = analyze_global_function("void two(int x);", "two", None);
    assert_eq!(edge_targets(&named.edges, "two"), vec!["int"]);
}

#[test]
fn handle_global_function_default_arguments_only_edge_leading_words() {
    let outcome = analyze_global_function("void def(A a = B(), int x = 5) {}", "def", None);
    assert!(outcome.declarations[0].is_definition);
    assert_eq!(
        edge_targets(&outcome.edges, "def"),
        vec!["A", "int"],
        "quirk: B in `= B()` gets no edge, but the second param's type word does"
    );
}

#[test]
fn handle_global_function_template_emits_return_and_param_edge_to_same_type() {
    let outcome = analyze_global_function(
        "template<typename T> T identity(T x) { return x; }",
        "identity",
        None,
    );
    assert_eq!(outcome.symbols.len(), 1);
    assert_eq!(outcome.symbols[0].id.0, "identity");
    assert!(outcome.declarations[0].is_definition);
    assert_eq!(
        edge_targets(&outcome.edges, "identity"),
        vec!["T", "T"],
        "return-type edge and parameter edge both target T"
    );
    assert_eq!(outcome.opened_function, Some(("identity".to_string(), 1)));
}

#[test]
fn handle_global_function_macro_case_line_records_macro_name_only() {
    let outcome = analyze_global_function("TEST_F(MySuite, MyTest) { helper(); }", "TEST_F", None);
    assert_eq!(
        outcome.symbols[0].id.0, "TEST_F",
        "quirk: suite/case names are lost; every TEST_F shares one symbol"
    );
    assert!(outcome.declarations[0].is_definition);
    assert_eq!(outcome.opened_function, Some(("TEST_F".to_string(), 1)));
    assert!(
        outcome.edges.is_empty(),
        "quirk: params followed by `,`/`)` fail the followed-by-Word test, so no edges"
    );
}

#[test]
fn handle_global_function_repeated_macro_definition_overwrites_definition_decl() {
    let tokens = tokenize("TEST_F(Suite, Case) { a(); } TEST_F(Suite, Case) { b(); }");
    let mut ctx = ParserCtx::new(FileId::from_relative_path("macro_case.cpp"), &tokens);
    let macro_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            matches!(&token.kind, TokenKind::Word(word) if word.as_str() == "TEST_F")
                .then_some(index)
        })
        .collect();
    assert_eq!(macro_indices.len(), 2);
    for (round, index) in macro_indices.into_iter().enumerate() {
        if round > 0 {
            // emulate the main loop processing `{` and `}` of the first body
            ctx.handle_brace_open();
            ctx.handle_brace_close();
        }
        ctx.i = index;
        let name_token = ctx.tokens[index].clone();
        ctx.handle_global_function("TEST_F", &name_token);
    }
    assert_eq!(ctx.symbols.len(), 1);
    assert_eq!(ctx.symbols[0].name, "TEST_F");
    assert_eq!(
        ctx.symbols[0].declarations,
        vec![DeclId("decl_1".into()), DeclId("decl_2".into())]
    );
    assert_eq!(
        ctx.symbols[0].definition,
        Some(DeclId("decl_2".into())),
        "quirk: the later definition overwrites the earlier one"
    );
    assert!(ctx.declarations.iter().all(|decl| decl.is_definition));
}

#[test]
fn handle_global_function_unclosed_paren_still_records_declaration() {
    let outcome = analyze_global_function("void broken(; int z;", "broken", None);
    assert_eq!(outcome.symbols.len(), 1);
    assert_eq!(outcome.symbols[0].id.0, "broken");
    assert_eq!(outcome.declarations.len(), 1);
    assert!(
        !outcome.declarations[0].is_definition,
        "quirk: paren scan runs to EOF without finding `)` and still records a decl"
    );
    assert!(outcome.opened_function.is_none());
}

#[test]
fn handle_global_function_declaration_then_definition_merges_into_one_symbol() {
    let tokens = tokenize("int f(); int f() {}");
    let mut ctx = ParserCtx::new(FileId::from_relative_path("merge.cpp"), &tokens);
    let name_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            matches!(&token.kind, TokenKind::Word(word) if word.as_str() == "f").then_some(index)
        })
        .collect();
    assert_eq!(name_indices.len(), 2);
    for index in name_indices {
        ctx.i = index;
        let name_token = ctx.tokens[index].clone();
        ctx.handle_global_function("f", &name_token);
    }
    assert_eq!(ctx.symbols.len(), 1);
    assert_eq!(
        ctx.symbols[0].declarations,
        vec![DeclId("decl_1".into()), DeclId("decl_2".into())]
    );
    assert_eq!(ctx.symbols[0].definition, Some(DeclId("decl_2".into())));
    assert_eq!(edge_targets(&ctx.edges, "f"), vec!["int", "int"]);
}

#[test]
fn handle_global_function_through_analyzer_drops_namespace_qualifiers() {
    let tu = CppSemanticAnalyzer::analyze_source(
        FileId::from_relative_path("ns.cpp"),
        "namespace a { namespace b { int inner() { return 1; } } }",
        "Debug|x64",
    );
    let symbol_names: Vec<String> = tu
        .symbols
        .iter()
        .map(|symbol| symbol.name.clone())
        .collect();
    assert_eq!(
        symbol_names,
        vec!["inner"],
        "quirk: namespaces are ignored, inner is recorded by bare name"
    );
    assert_eq!(tu.symbols[0].kind, SymbolKind::Function);
    assert!(tu.symbols[0].definition.is_some());
    assert_eq!(edge_targets(&tu.edges, "inner"), vec!["int"]);
}

#[test]
fn handle_global_function_name_at_end_of_token_stream_records_nothing() {
    let outcome = analyze_global_function("int trailing", "trailing", None);
    assert!(
        outcome.symbols.is_empty() && outcome.declarations.is_empty(),
        "quirk: a word with no following token never reaches the `(` check"
    );
    assert!(outcome.opened_function.is_none());
}

#[test]
fn handle_global_function_dangling_open_paren_at_eof_still_records_declaration() {
    let outcome = analyze_global_function("void open(", "open", None);
    assert_eq!(outcome.symbols[0].id.0, "open");
    assert_eq!(outcome.declarations.len(), 1);
    assert!(
        !outcome.declarations[0].is_definition,
        "quirk: both forward scans see no tokens at all and still record a decl"
    );
    assert!(outcome.opened_function.is_none());
    assert!(
        outcome.edges.is_empty(),
        "the only preceding word is void, which is excluded"
    );
}

#[test]
fn handle_global_function_return_edge_only_from_immediately_preceding_word() {
    let plain_void = analyze_global_function("static inline void v();", "v", None);
    assert!(
        plain_void.edges.is_empty(),
        "void is excluded even when static/inline precede it"
    );

    let virtual_class = analyze_global_function("virtual B vb();", "vb", None);
    assert_eq!(edge_targets(&virtual_class.edges, "vb"), vec!["B"]);

    let multi_word_type = analyze_global_function("unsigned int count();", "count", None);
    assert_eq!(
        edge_targets(&multi_word_type.edges, "count"),
        vec!["int"],
        "quirk: only the word immediately before the name is treated as return type"
    );
}
