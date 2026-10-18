//! In-process C++ semantic analyzer (plan M18.2: relocated verbatim from
//! `crates/index/src/analyzer.rs`; only the value-type import changed) plus
//! the three-line `ariadne_lang::SemanticAnalyzer` adapter (plan §3.3 fact 1).

use crate::parser::ParserCtx;
use crate::tokenizer::{tokenize, TokenKind};
use ariadne_lang_api::types::*;
use ariadne_repo::{FileId, SourceLanguage};
use std::path::Path;

pub struct CppSemanticAnalyzer;

impl CppSemanticAnalyzer {
    pub fn analyze_source(tu_file_id: FileId, source_code: &str, context_name: &str) -> TuAnalysis {
        let tokens = tokenize(source_code);
        let mut ctx = ParserCtx::new(tu_file_id.clone(), &tokens);

        while ctx.i < ctx.tokens.len() {
            let token = ctx.tokens[ctx.i].clone();
            match &token.kind {
                TokenKind::Punct(p) if p == "{" => ctx.handle_brace_open(),
                TokenKind::Punct(p) if p == "}" => ctx.handle_brace_close(),
                TokenKind::Word(w) if w == "class" || w == "struct" => {
                    let w = w.clone();
                    ctx.handle_class_or_struct(&w);
                }
                TokenKind::Word(w) if w == "sizeof" => {
                    ctx.handle_sizeof();
                }
                TokenKind::Word(w) if w == "new" => {
                    ctx.handle_new();
                }
                TokenKind::Word(w) => {
                    let w = w.clone();
                    ctx.handle_word(&w, &token);
                }
                _ => {}
            }
            ctx.i += 1;
        }

        TuAnalysis {
            tu_file_id,
            context_name: context_name.to_string(),
            symbols: ctx.symbols,
            declarations: ctx.declarations,
            edges: ctx.edges,
            references: vec![],
            call_sites: ctx.call_sites,
            language: SourceLanguage::Cpp,

            extends_edges: Vec::new(),
        }
    }

    pub fn analyze_tu(
        &self,
        tu: &ariadne_build::TranslationUnit,
        repo_root: &Path,
    ) -> anyhow::Result<TuAnalysis> {
        let file_path = repo_root.join(&tu.source_path);
        let source_code = std::fs::read_to_string(&file_path)?;
        Ok(Self::analyze_source(
            tu.file_id.clone(),
            &source_code,
            &tu.context_name,
        ))
    }
}

impl ariadne_lang::SemanticAnalyzer for CppSemanticAnalyzer {
    fn analyze(&self, file_id: FileId, source: &str, context: &str) -> TuAnalysis {
        Self::analyze_source(file_id, source, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_declaration_incomplete() {
        let code = r#"
            class B;
            class A {
                B* value;
            };
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        let edge = tu
            .edges
            .iter()
            .find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
        assert!(edge.is_some(), "Edge A -> B must exist");
        assert_eq!(edge.unwrap().kind, DependencyKind::Incomplete);
    }

    #[test]
    fn test_by_value_member_complete() {
        let code = r#"
            class B {};
            class A {
                B value;
            };
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        let edge = tu
            .edges
            .iter()
            .find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
        assert!(edge.is_some(), "Edge A -> B must exist");
        assert_eq!(edge.unwrap().kind, DependencyKind::Complete);
    }

    #[test]
    fn test_inheritance_complete() {
        let code = r#"
            class B {};
            class A : public B {};
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        let edge = tu
            .edges
            .iter()
            .find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
        assert!(edge.is_some(), "Edge A -> B must exist");
        assert_eq!(edge.unwrap().kind, DependencyKind::Complete);
    }

    #[test]
    fn test_sizeof_complete() {
        let code = r#"
            class B {};
            class A {
                void foo() {
                    sizeof(B);
                }
            };
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        let edge = tu
            .edges
            .iter()
            .find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
        assert!(edge.is_some(), "Edge A -> B must exist");
        assert_eq!(edge.unwrap().kind, DependencyKind::Complete);
    }

    #[test]
    fn test_function_declaration_return_incomplete() {
        let code = r#"
            class B;
            class A {
                B make_b();
            };
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        let edge = tu
            .edges
            .iter()
            .find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
        assert!(edge.is_some(), "Edge A -> B must exist");
        assert_eq!(edge.unwrap().kind, DependencyKind::Incomplete);
    }

    #[test]
    fn test_call_site_extraction() {
        let code = r#"
            void helper() {}
            void target() {
                helper();
                helper();
            }
            void recursive() {
                recursive();
            }
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        assert_eq!(tu.call_sites.len(), 3);

        let target_calls: Vec<_> = tu
            .call_sites
            .iter()
            .filter(|c| c.caller_symbol.0 == "target")
            .collect();
        assert_eq!(target_calls.len(), 2);
        assert_eq!(target_calls[0].callee_name, "helper");
        assert_eq!(target_calls[1].callee_name, "helper");

        let recursive_calls: Vec<_> = tu
            .call_sites
            .iter()
            .filter(|c| c.caller_symbol.0 == "recursive")
            .collect();
        assert_eq!(recursive_calls.len(), 1);
        assert_eq!(recursive_calls[0].callee_name, "recursive");
    }

    #[test]
    fn test_namespace_and_qualified_function_definition() {
        let code = r#"
            namespace my_ns {
                class MyClass {
                    void method() const;
                };

                void MyClass::method() const {
                    helper();
                }

                void global_func(int x) noexcept {
                    helper2();
                }
            }
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        let myclass_method_calls: Vec<_> = tu
            .call_sites
            .iter()
            .filter(|c| c.caller_symbol.0 == "method" || c.caller_symbol.0 == "MyClass::method")
            .collect();
        assert_eq!(
            myclass_method_calls.len(),
            1,
            "Call site inside method must be extracted"
        );
        assert_eq!(myclass_method_calls[0].callee_name, "helper");

        let global_func_calls: Vec<_> = tu
            .call_sites
            .iter()
            .filter(|c| c.caller_symbol.0 == "global_func")
            .collect();
        assert_eq!(
            global_func_calls.len(),
            1,
            "Call site inside global_func must be extracted"
        );
        assert_eq!(global_func_calls[0].callee_name, "helper2");
    }

    #[test]
    fn test_tokenizer_ignores_comments_and_strings() {
        let code = r#"
            void foo() {
                // } 
                /* } */
                const char* s = "}";
                char c = '}';
                MACRO(func());
            }
            void bar() {
                MACRO(func());
            }
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        let foo_calls: Vec<_> = tu
            .call_sites
            .iter()
            .filter(|c| c.caller_symbol.0 == "foo")
            .collect();
        assert_eq!(
            foo_calls.len(),
            2,
            "Call to MACRO and func should be captured"
        );
        let bar_calls: Vec<_> = tu
            .call_sites
            .iter()
            .filter(|c| c.caller_symbol.0 == "bar")
            .collect();
        assert_eq!(
            bar_calls.len(),
            2,
            "bar should also capture MACRO and func calls"
        );
    }

    #[test]
    fn test_member_methods_in_class_bodies() {
        let code = r#"
            class MyClass {
                void inline_method() {
                    helper();
                }
            };
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        let calls: Vec<_> = tu
            .call_sites
            .iter()
            .filter(|c| {
                c.caller_symbol.0 == "inline_method"
                    || c.caller_symbol.0 == "MyClass::inline_method"
            })
            .collect();
        assert_eq!(
            calls.len(),
            1,
            "Call site inside inline member method must be extracted"
        );
        assert_eq!(calls[0].callee_name, "helper");
    }

    #[test]
    fn test_function_scope_dependency_edges() {
        let code = r#"
            class B;
            class C;
            void global_func(B* b_ptr) {
                C c_val;
            }
        "#;
        let tu = CppSemanticAnalyzer::analyze_source(
            FileId::from_relative_path("test.cpp"),
            code,
            "Debug|x64",
        );
        let b_edge = tu
            .edges
            .iter()
            .find(|e| e.src_symbol.0 == "global_func" && e.dst_symbol.0 == "B");
        assert!(
            b_edge.is_some(),
            "Edge global_func -> B must exist (incomplete)"
        );
        assert_eq!(b_edge.unwrap().kind, DependencyKind::Incomplete);

        let c_edge = tu
            .edges
            .iter()
            .find(|e| e.src_symbol.0 == "global_func" && e.dst_symbol.0 == "C");
        assert!(
            c_edge.is_some(),
            "Edge global_func -> C must exist (complete)"
        );
        assert_eq!(c_edge.unwrap().kind, DependencyKind::Complete);
    }

    #[test]
    fn trait_adapter_matches_analyze_source_byte_for_byte() {
        use ariadne_lang::SemanticAnalyzer;

        let code = r#"
            class B;
            class A : public B {
                B* p;
                void f() { helper(); }
            };
            namespace ns { void g() { helper2(); } }
        "#;
        let file_id = FileId::from_relative_path("adapter_test.cpp");
        let direct = CppSemanticAnalyzer::analyze_source(file_id.clone(), code, "Debug|x64");
        let via_trait = CppSemanticAnalyzer.analyze(file_id, code, "Debug|x64");
        assert_eq!(direct, via_trait);
    }
}

#[cfg(test)]
mod tu_io_pin_tests;
