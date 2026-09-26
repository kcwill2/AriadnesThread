// Characterization tests: they lock the CURRENT behavior of `tokenize` and
// `ParserCtx::handle_global_function` before a later pass decomposes them.
// Several asserts below deliberately encode quirky-but-current behavior;
// if one fails, the code changed — that is a finding, not something to "fix" here.

use super::*;

fn token_kinds(source: &str) -> Vec<String> {
    tokenize(source)
        .iter()
        .map(|token| match &token.kind {
            TokenKind::Word(word) => format!("word:{word}"),
            TokenKind::Punct(punct) => format!("punct:{punct}"),
        })
        .collect()
}

fn expect_token_kinds(source: &str, expected: &[&str]) {
    let actual = token_kinds(source);
    let actual_refs: Vec<&str> = actual.iter().map(String::as_str).collect();
    assert_eq!(actual_refs, expected, "token kinds for {source:?}");
}

#[test]
fn tokenize_word_and_punct_run_locks_one_based_line_and_column() {
    assert_eq!(
        tokenize("int f();"),
        vec![
            Token { kind: TokenKind::Word("int".into()), line: 1, column: 1 },
            Token { kind: TokenKind::Word("f".into()), line: 1, column: 5 },
            Token { kind: TokenKind::Punct("(".into()), line: 1, column: 6 },
            Token { kind: TokenKind::Punct(")".into()), line: 1, column: 7 },
            Token { kind: TokenKind::Punct(";".into()), line: 1, column: 8 },
        ]
    );
}

#[test]
fn tokenize_splits_multi_char_operators_into_single_char_puncts() {
    expect_token_kinds(
        "x == 42 >= y",
        &["word:x", "punct:=", "punct:=", "word:42", "punct:>", "punct:=", "word:y"],
    );
}

#[test]
fn tokenize_string_literal_with_braces_and_comment_openers_yields_no_string_token() {
    expect_token_kinds(
        "x = \"}{ /* // */\"; y();",
        &["word:x", "punct:=", "punct:;", "word:y", "punct:(", "punct:)", "punct:;"],
    );
}

#[test]
fn tokenize_char_literal_is_swallowed_like_a_string_literal() {
    expect_token_kinds(
        "char c = '}';",
        &["word:char", "word:c", "punct:=", "punct:;"],
    );
}

#[test]
fn tokenize_escaped_quote_does_not_terminate_string_early() {
    expect_token_kinds(
        "x = \"a\\\"b\"; f();",
        &["word:x", "punct:=", "punct:;", "word:f", "punct:(", "punct:)", "punct:;"],
    );
}

#[test]
fn tokenize_string_spanning_newline_advances_line_number_and_hides_braces() {
    assert_eq!(
        tokenize("a \"x\n} y\" b"),
        vec![
            Token { kind: TokenKind::Word("a".into()), line: 1, column: 1 },
            Token { kind: TokenKind::Word("b".into()), line: 2, column: 6 },
        ]
    );
}

#[test]
fn tokenize_unterminated_string_literal_swallows_rest_of_file() {
    expect_token_kinds("x = \"{", &["word:x", "punct:="]);
}

#[test]
fn tokenize_unterminated_block_comment_drops_rest_of_file() {
    expect_token_kinds("a /* b } c", &["word:a"]);
}

#[test]
fn tokenize_block_comment_hides_braces_until_terminator() {
    expect_token_kinds("a /* b } */ c", &["word:a", "word:c"]);
}

#[test]
fn tokenize_line_comment_skips_to_newline_without_emitting_tokens() {
    assert_eq!(
        tokenize("// }\n}"),
        vec![Token { kind: TokenKind::Punct("}".into()), line: 2, column: 1 }]
    );
    expect_token_kinds("a } // trailing", &["word:a", "punct:}"]);
}

#[test]
fn tokenize_line_comment_with_trailing_backslash_does_not_continue_to_next_line() {
    assert_eq!(
        tokenize("a // b \\\n}"),
        vec![
            Token { kind: TokenKind::Word("a".into()), line: 1, column: 1 },
            Token { kind: TokenKind::Punct("}".into()), line: 2, column: 1 },
        ]
    );
}

#[test]
fn tokenize_stray_block_comment_terminator_emits_two_puncts() {
    expect_token_kinds("a */ b", &["word:a", "punct:*", "punct:/", "word:b"]);
}

#[test]
fn tokenize_lone_slash_is_punct_when_not_followed_by_comment_opener() {
    expect_token_kinds("a / b", &["word:a", "punct:/", "word:b"]);
    expect_token_kinds("a /", &["word:a", "punct:/"]);
}

#[test]
fn tokenize_crlf_input_produces_identical_tokens_to_lf_input() {
    assert_eq!(
        tokenize("int f();\r\nint g();\r\n"),
        tokenize("int f();\nint g();\n")
    );
}

#[test]
fn tokenize_tab_counts_as_one_column() {
    assert_eq!(
        tokenize("\tfoo"),
        vec![Token { kind: TokenKind::Word("foo".into()), line: 1, column: 2 }]
    );
}

#[test]
fn tokenize_nested_template_angle_brackets_split_into_two_gt_puncts() {
    expect_token_kinds(
        "std::vector<std::pair<int,char>>",
        &[
            "word:std", "punct::", "punct::", "word:vector", "punct:<",
            "word:std", "punct::", "punct::", "word:pair", "punct:<",
            "word:int", "punct:,", "word:char", "punct:>", "punct:>",
        ],
    );
}

#[test]
fn tokenize_nested_namespace_braces_produce_separate_puncts() {
    expect_token_kinds(
        "namespace a { namespace b { } }",
        &[
            "word:namespace", "word:a", "punct:{",
            "word:namespace", "word:b", "punct:{", "punct:}", "punct:}",
        ],
    );
}

#[test]
fn tokenize_macro_call_keeps_macro_name_and_parens_as_separate_tokens() {
    expect_token_kinds(
        "MACRO(func())",
        &["word:MACRO", "punct:(", "word:func", "punct:(", "punct:)", "punct:)"],
    );
}

#[test]
fn tokenize_preprocessor_include_lines_keep_angle_brackets_but_swallow_quoted_paths() {
    expect_token_kinds(
        "#include <vector>",
        &["punct:#", "word:include", "punct:<", "word:vector", "punct:>"],
    );
    expect_token_kinds("#include \"q.h\"", &["punct:#", "word:include"]);
}

#[test]
fn tokenize_numbers_split_on_dot_and_digit_separators_swallow_groups() {
    expect_token_kinds(
        "f = 1.5 + 1'000'000 + 0x1F;",
        &[
            "word:f", "punct:=", "word:1", "punct:.", "word:5", "punct:+",
            "word:1", "word:000", "punct:+", "word:0x1F", "punct:;",
        ],
    );
}

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
        opened_function: ctx.current_function.map(|(symbol_id, depth)| (symbol_id.0, depth)),
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
    assert!(declaration.edges.is_empty(), "void return type is excluded from edges");

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
        "if", "while", "for", "switch", "return", "sizeof", "catch", "new", "delete", "throw",
        "class", "struct", "namespace", "else",
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
        outcome.edges.iter().all(|edge| edge.kind == DependencyKind::Incomplete),
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
            matches!(&token.kind, TokenKind::Word(word) if word.as_str() == "TEST_F").then_some(index)
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
    let symbol_names: Vec<String> = tu.symbols.iter().map(|symbol| symbol.name.clone()).collect();
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
