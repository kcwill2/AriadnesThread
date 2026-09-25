use std::path::Path;
use ariadne_repo::FileId;
use crate::types::*;

#[derive(Debug, Clone, PartialEq)]
struct Token {
    kind: TokenKind,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Word(String),
    Punct(String),
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    let mut current_punct = String::new();
    let mut line = 1;
    let mut col = 1;

    let flush_punct = |tokens: &mut Vec<Token>, cp: &mut String, l, c| {
        if !cp.is_empty() {
            tokens.push(Token {
                kind: TokenKind::Punct(cp.clone()),
                line: l,
                column: c - cp.len(),
            });
            cp.clear();
        }
    };

    let flush_word = |tokens: &mut Vec<Token>, cw: &mut String, l, c| {
        if !cw.is_empty() {
            tokens.push(Token {
                kind: TokenKind::Word(cw.clone()),
                line: l,
                column: c - cw.len(),
            });
            cw.clear();
        }
    };

    for c in source.chars() {
        if c == '\n' {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
            line += 1;
            col = 1;
            continue;
        }

        if c.is_alphanumeric() || c == '_' {
            flush_punct(&mut tokens, &mut current_punct, line, col);
            current_word.push(c);
        } else if c.is_whitespace() {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
        } else {
            flush_word(&mut tokens, &mut current_word, line, col);
            current_punct.push(c);
            flush_punct(&mut tokens, &mut current_punct, line, col + 1); // single char punct for now
        }
        col += 1;
    }
    flush_word(&mut tokens, &mut current_word, line, col);
    flush_punct(&mut tokens, &mut current_punct, line, col);
    tokens
}

struct ParserCtx<'a> {
    tu_file_id: FileId,
    tokens: &'a [Token],
    edges: Vec<SemanticEdge>,
    symbols: Vec<SymbolRecord>,
    declarations: Vec<SymbolDecl>,
    call_sites: Vec<CallSite>,
    current_class: Option<(String, usize)>,
    current_function: Option<(SymbolId, usize)>,
    brace_depth: usize,
    decl_counter: usize,
    i: usize,
}

impl<'a> ParserCtx<'a> {
    fn new(tu_file_id: FileId, tokens: &'a [Token]) -> Self {
        Self {
            tu_file_id,
            tokens,
            edges: Vec::new(),
            symbols: Vec::new(),
            declarations: Vec::new(),
            call_sites: Vec::new(),
            current_class: None,
            current_function: None,
            brace_depth: 0,
            decl_counter: 0,
            i: 0,
        }
    }

    fn handle_brace_open(&mut self) {
        self.brace_depth += 1;
    }

    fn handle_brace_close(&mut self) {
        self.brace_depth -= 1;
        if let Some((_, depth)) = self.current_class {
            if self.brace_depth < depth {
                self.current_class = None;
            }
        }
        if let Some((_, depth)) = self.current_function {
            if self.brace_depth < depth {
                self.current_function = None;
            }
        }
    }

    fn handle_class_or_struct(&mut self, w: &str) {
        if self.i + 1 >= self.tokens.len() {
            return;
        }
        if let TokenKind::Word(name) = &self.tokens[self.i + 1].kind {
            let loc = FileLocation {
                file_id: self.tu_file_id.clone(),
                line: self.tokens[self.i + 1].line,
                column: self.tokens[self.i + 1].column,
            };
            let symbol_id = SymbolId::new(name.clone());
            self.decl_counter += 1;
            let decl_id = DeclId::new(format!("decl_{}", self.decl_counter));

            let mut is_definition = false;
            
            if self.i + 2 < self.tokens.len() {
                match &self.tokens[self.i + 2].kind {
                    TokenKind::Punct(p) if p == ";" => {
                        is_definition = false;
                        self.i += 2;
                    }
                    TokenKind::Punct(p) if p == "{" => {
                        is_definition = true;
                        self.current_class = Some((name.clone(), self.brace_depth + 1));
                        self.i += 1;
                    }
                    TokenKind::Punct(p) if p == ":" => {
                        is_definition = true;
                        self.current_class = Some((name.clone(), self.brace_depth + 1));
                        let mut j = self.i + 3;
                        while j < self.tokens.len() {
                            if let TokenKind::Word(base_w) = &self.tokens[j].kind {
                                if base_w != "public" && base_w != "private" && base_w != "protected" {
                                    self.edges.push(SemanticEdge {
                                        src_symbol: SymbolId::new(name.clone()),
                                        dst_symbol: SymbolId::new(base_w.clone()),
                                        kind: DependencyKind::Complete,
                                        src_file: self.tu_file_id.clone(),
                                        dst_file: self.tu_file_id.clone(),
                                    });
                                    break;
                                }
                            }
                            j += 1;
                        }
                        self.i = j;
                    }
                    _ => {}
                }
            }

            self.declarations.push(SymbolDecl {
                id: decl_id.clone(),
                symbol_id: symbol_id.clone(),
                location: loc,
                is_definition,
            });

            let kind = if w == "class" { SymbolKind::Class } else { SymbolKind::Struct };

            if let Some(existing) = self.symbols.iter_mut().find(|s| s.id == symbol_id) {
                existing.declarations.push(decl_id.clone());
                if is_definition {
                    existing.definition = Some(decl_id);
                }
            } else {
                self.symbols.push(SymbolRecord {
                    id: symbol_id,
                    name: name.clone(),
                    kind,
                    owning_file: self.tu_file_id.clone(),
                    declarations: vec![decl_id.clone()],
                    definition: if is_definition { Some(decl_id) } else { None },
                });
            }
        }
    }

    fn handle_sizeof(&mut self) {
        if self.i + 2 < self.tokens.len() {
            if let TokenKind::Punct(p) = &self.tokens[self.i + 1].kind {
                if p == "(" {
                    if let TokenKind::Word(target) = &self.tokens[self.i + 2].kind {
                        if let Some((ref class_name, _)) = self.current_class {
                            self.edges.push(SemanticEdge {
                                src_symbol: SymbolId::new(class_name.clone()),
                                dst_symbol: SymbolId::new(target.clone()),
                                kind: DependencyKind::Complete,
                                src_file: self.tu_file_id.clone(),
                                dst_file: self.tu_file_id.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    fn handle_new(&mut self) {
        if self.i + 1 < self.tokens.len() {
            if let TokenKind::Word(target) = &self.tokens[self.i + 1].kind {
                if let Some((ref class_name, _)) = self.current_class {
                    self.edges.push(SemanticEdge {
                        src_symbol: SymbolId::new(class_name.clone()),
                        dst_symbol: SymbolId::new(target.clone()),
                        kind: DependencyKind::Complete,
                        src_file: self.tu_file_id.clone(),
                        dst_file: self.tu_file_id.clone(),
                    });
                }
            }
        }
    }

    fn handle_word(&mut self, w: &str, token: &Token) {
        self.handle_call_site(w, token);
        self.handle_global_function(w, token);
        self.handle_variable_decl(w);
    }

    fn handle_call_site(&mut self, w: &str, token: &Token) {
        if let Some((ref func_symbol, _)) = self.current_function {
            if self.brace_depth > 0 && self.i + 1 < self.tokens.len() {
                if let TokenKind::Punct(p) = &self.tokens[self.i + 1].kind {
                    if p == "(" {
                        let reserved = ["if", "while", "for", "switch", "return", "sizeof", "catch", "new", "delete", "throw"];
                        if !reserved.contains(&w) {
                            self.call_sites.push(CallSite {
                                caller_symbol: func_symbol.clone(),
                                callee_name: w.to_string(),
                                location: FileLocation {
                                    file_id: self.tu_file_id.clone(),
                                    line: token.line,
                                    column: token.column,
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    fn handle_global_function(&mut self, w: &str, token: &Token) {
        if self.current_class.is_none() && self.current_function.is_none() && self.i + 1 < self.tokens.len() {
            if let TokenKind::Punct(p) = &self.tokens[self.i + 1].kind {
                if p == "(" {
                    let reserved = ["if", "while", "for", "switch", "return", "sizeof", "catch", "new", "delete", "throw", "class", "struct", "namespace", "else"];
                    if reserved.contains(&w) {
                        return;
                    }

                    let name = w.to_string();
                    let loc = FileLocation {
                        file_id: self.tu_file_id.clone(),
                        line: token.line,
                        column: token.column,
                    };
                    let symbol_id = SymbolId::new(name.clone());
                    
                    let mut is_definition = false;
                    let mut j = self.i + 2;
                    let mut paren_depth = 1;
                    while j < self.tokens.len() && paren_depth > 0 {
                        if let TokenKind::Punct(p2) = &self.tokens[j].kind {
                            if p2 == "(" { paren_depth += 1; }
                            else if p2 == ")" { paren_depth -= 1; }
                        }
                        j += 1;
                    }
                    
                    while j < self.tokens.len() {
                        if let TokenKind::Punct(p3) = &self.tokens[j].kind {
                            if p3 == "{" {
                                is_definition = true;
                                self.current_function = Some((symbol_id.clone(), self.brace_depth + 1));
                                break;
                            } else if p3 == ";" {
                                break;
                            }
                        }
                        j += 1;
                    }

                    self.decl_counter += 1;
                    let decl_id = DeclId::new(format!("decl_{}", self.decl_counter));

                    self.declarations.push(SymbolDecl {
                        id: decl_id.clone(),
                        symbol_id: symbol_id.clone(),
                        location: loc,
                        is_definition,
                    });

                    if let Some(existing) = self.symbols.iter_mut().find(|s| s.id == symbol_id) {
                        existing.declarations.push(decl_id.clone());
                        if is_definition && existing.kind == SymbolKind::Function {
                            existing.definition = Some(decl_id);
                        }
                    } else {
                        self.symbols.push(SymbolRecord {
                            id: symbol_id,
                            name: name.clone(),
                            kind: SymbolKind::Function,
                            owning_file: self.tu_file_id.clone(),
                            declarations: vec![decl_id.clone()],
                            definition: if is_definition { Some(decl_id) } else { None },
                        });
                    }
                }
            }
        }
    }

    fn handle_variable_decl(&mut self, w: &str) {
        if let Some((ref class_name, _)) = self.current_class {
            if self.i + 1 < self.tokens.len() {
                let type_name = w.to_string();
                if let TokenKind::Word(_) = &self.tokens[self.i + 1].kind {
                    if type_name != "void" && type_name != "return" {
                        let mut is_function = false;
                        if self.i + 2 < self.tokens.len() {
                            if let TokenKind::Punct(p) = &self.tokens[self.i + 2].kind {
                                if p == "(" {
                                    is_function = true;
                                }
                            }
                        }
                        
                        self.edges.push(SemanticEdge {
                            src_symbol: SymbolId::new(class_name.clone()),
                            dst_symbol: SymbolId::new(type_name),
                            kind: if is_function { DependencyKind::Incomplete } else { DependencyKind::Complete },
                            src_file: self.tu_file_id.clone(),
                            dst_file: self.tu_file_id.clone(),
                        });
                    }
                } else if let TokenKind::Punct(p) = &self.tokens[self.i + 1].kind {
                    if p == "*" || p == "&" {
                        self.edges.push(SemanticEdge {
                            src_symbol: SymbolId::new(class_name.clone()),
                            dst_symbol: SymbolId::new(type_name),
                            kind: DependencyKind::Incomplete,
                            src_file: self.tu_file_id.clone(),
                            dst_file: self.tu_file_id.clone(),
                        });
                    } else if p == "<" {
                        if type_name == "unique_ptr" || type_name == "shared_ptr" {
                            if self.i + 2 < self.tokens.len() {
                                if let TokenKind::Word(target) = &self.tokens[self.i + 2].kind {
                                    self.edges.push(SemanticEdge {
                                        src_symbol: SymbolId::new(class_name.clone()),
                                        dst_symbol: SymbolId::new(target.clone()),
                                        kind: DependencyKind::Incomplete,
                                        src_file: self.tu_file_id.clone(),
                                        dst_file: self.tu_file_id.clone(),
                                    });
                                }
                            }
                        }
                    } else if p == "(" {
                        if self.i > 0 {
                            if let TokenKind::Word(_) = &self.tokens[self.i - 1].kind {
                                // Something like `new B()`
                            } else {
                                self.edges.push(SemanticEdge {
                                    src_symbol: SymbolId::new(class_name.clone()),
                                    dst_symbol: SymbolId::new(type_name),
                                    kind: DependencyKind::Complete,
                                    src_file: self.tu_file_id.clone(),
                                    dst_file: self.tu_file_id.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

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
        let edge = tu.edges.iter().find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
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
        let edge = tu.edges.iter().find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
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
        let edge = tu.edges.iter().find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
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
        let edge = tu.edges.iter().find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
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
        let edge = tu.edges.iter().find(|e| e.src_symbol.0 == "A" && e.dst_symbol.0 == "B");
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
        
        let target_calls: Vec<_> = tu.call_sites.iter().filter(|c| c.caller_symbol.0 == "target").collect();
        assert_eq!(target_calls.len(), 2);
        assert_eq!(target_calls[0].callee_name, "helper");
        assert_eq!(target_calls[1].callee_name, "helper");
        
        let recursive_calls: Vec<_> = tu.call_sites.iter().filter(|c| c.caller_symbol.0 == "recursive").collect();
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
        let myclass_method_calls: Vec<_> = tu.call_sites.iter().filter(|c| c.caller_symbol.0 == "method").collect();
        assert_eq!(myclass_method_calls.len(), 1, "Call site inside method must be extracted");
        assert_eq!(myclass_method_calls[0].callee_name, "helper");

        let global_func_calls: Vec<_> = tu.call_sites.iter().filter(|c| c.caller_symbol.0 == "global_func").collect();
        assert_eq!(global_func_calls.len(), 1, "Call site inside global_func must be extracted");
        assert_eq!(global_func_calls[0].callee_name, "helper2");
    }
}
