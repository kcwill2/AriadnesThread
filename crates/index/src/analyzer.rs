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

pub struct CppSemanticAnalyzer;

impl CppSemanticAnalyzer {
    pub fn analyze_source(tu_file_id: FileId, source_code: &str, context_name: &str) -> TuAnalysis {
        let tokens = tokenize(source_code);
        let mut edges = Vec::new();
        let mut symbols = Vec::new();
        let mut declarations = Vec::new();

        let mut current_class: Option<(String, usize)> = None;
        let mut brace_depth = 0;
        let mut i = 0;

        let mut decl_counter = 0;

        while i < tokens.len() {
            let token = &tokens[i];
            match &token.kind {
                TokenKind::Punct(p) if p == "{" => {
                    brace_depth += 1;
                }
                TokenKind::Punct(p) if p == "}" => {
                    brace_depth -= 1;
                    if let Some((_, depth)) = current_class {
                        if brace_depth < depth {
                            current_class = None;
                        }
                    }
                }
                TokenKind::Word(w) if w == "class" || w == "struct" => {
                    if i + 1 < tokens.len() {
                        if let TokenKind::Word(name) = &tokens[i + 1].kind {
                            let loc = FileLocation {
                                file_id: tu_file_id.clone(),
                                line: tokens[i + 1].line,
                                column: tokens[i + 1].column,
                            };
                            let symbol_id = SymbolId::new(name.clone());
                            decl_counter += 1;
                            let decl_id = DeclId::new(format!("decl_{}", decl_counter));

                            let mut is_definition = false;
                            
                            // Check forward decl vs definition vs inheritance
                            if i + 2 < tokens.len() {
                                match &tokens[i + 2].kind {
                                    TokenKind::Punct(p) if p == ";" => {
                                        // Forward decl
                                        is_definition = false;
                                        i += 2;
                                    }
                                    TokenKind::Punct(p) if p == "{" => {
                                        is_definition = true;
                                        current_class = Some((name.clone(), brace_depth + 1));
                                        i += 2; // will hit '{' in next iterations, wait actually we skip it if we i+=2. So just i+=1 and let the main loop handle '{'
                                        i -= 1; 
                                    }
                                    TokenKind::Punct(p) if p == ":" => {
                                        is_definition = true;
                                        current_class = Some((name.clone(), brace_depth + 1));
                                        // Find base class
                                        let mut j = i + 3;
                                        while j < tokens.len() {
                                            if let TokenKind::Word(base_w) = &tokens[j].kind {
                                                if base_w != "public" && base_w != "private" && base_w != "protected" {
                                                    edges.push(SemanticEdge {
                                                        src_symbol: SymbolId::new(name.clone()),
                                                        dst_symbol: SymbolId::new(base_w.clone()),
                                                        kind: DependencyKind::Complete,
                                                        src_file: tu_file_id.clone(),
                                                        dst_file: tu_file_id.clone(),
                                                    });
                                                    break;
                                                }
                                            }
                                            j += 1;
                                        }
                                        i = j;
                                    }
                                    _ => {}
                                }
                            }

                            declarations.push(SymbolDecl {
                                id: decl_id.clone(),
                                symbol_id: symbol_id.clone(),
                                location: loc,
                                is_definition,
                            });

                            let kind = if w == "class" { SymbolKind::Class } else { SymbolKind::Struct };

                            if let Some(existing) = symbols.iter_mut().find(|s: &&mut SymbolRecord| s.id == symbol_id) {
                                existing.declarations.push(decl_id.clone());
                                if is_definition {
                                    existing.definition = Some(decl_id);
                                }
                            } else {
                                symbols.push(SymbolRecord {
                                    id: symbol_id,
                                    name: name.clone(),
                                    kind,
                                    owning_file: tu_file_id.clone(),
                                    declarations: vec![decl_id.clone()],
                                    definition: if is_definition { Some(decl_id) } else { None },
                                });
                            }
                            
                            i += 1;
                            continue;
                        }
                    }
                }
                TokenKind::Word(w) if w == "sizeof" => {
                    if i + 2 < tokens.len() {
                        if let TokenKind::Punct(p) = &tokens[i + 1].kind {
                            if p == "(" {
                                if let TokenKind::Word(target) = &tokens[i + 2].kind {
                                    if let Some((ref class_name, _)) = current_class {
                                        edges.push(SemanticEdge {
                                            src_symbol: SymbolId::new(class_name.clone()),
                                            dst_symbol: SymbolId::new(target.clone()),
                                            kind: DependencyKind::Complete,
                                            src_file: tu_file_id.clone(),
                                            dst_file: tu_file_id.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                TokenKind::Word(w) if w == "new" => {
                    if i + 1 < tokens.len() {
                        if let TokenKind::Word(target) = &tokens[i + 1].kind {
                            if let Some((ref class_name, _)) = current_class {
                                edges.push(SemanticEdge {
                                    src_symbol: SymbolId::new(class_name.clone()),
                                    dst_symbol: SymbolId::new(target.clone()),
                                    kind: DependencyKind::Complete,
                                    src_file: tu_file_id.clone(),
                                    dst_file: tu_file_id.clone(),
                                });
                            }
                        }
                    }
                }
                TokenKind::Word(w) => {
                    if brace_depth == 0 && i >= 1 && i + 1 < tokens.len() {
                        if let TokenKind::Punct(p) = &tokens[i + 1].kind {
                            if p == "(" {
                                if let TokenKind::Word(prev_w) = &tokens[i - 1].kind {
                                    if prev_w != "class" && prev_w != "struct" {
                                        // This looks like a global function definition/declaration
                                        let name = w.clone();
                                        let loc = FileLocation {
                                            file_id: tu_file_id.clone(),
                                            line: token.line,
                                            column: token.column,
                                        };
                                        let symbol_id = SymbolId::new(name.clone());
                                        decl_counter += 1;
                                        let decl_id = DeclId::new(format!("decl_{}", decl_counter));
                                        
                                        // Find if it's a definition (has body later)
                                        let mut is_definition = false;
                                        let mut j = i + 2;
                                        let mut paren_depth = 1;
                                        while j < tokens.len() && paren_depth > 0 {
                                            if let TokenKind::Punct(p2) = &tokens[j].kind {
                                                if p2 == "(" { paren_depth += 1; }
                                                else if p2 == ")" { paren_depth -= 1; }
                                            }
                                            j += 1;
                                        }
                                        if j < tokens.len() {
                                            if let TokenKind::Punct(p3) = &tokens[j].kind {
                                                if p3 == "{" {
                                                    is_definition = true;
                                                }
                                            }
                                        }

                                        declarations.push(SymbolDecl {
                                            id: decl_id.clone(),
                                            symbol_id: symbol_id.clone(),
                                            location: loc,
                                            is_definition,
                                        });

                                        if let Some(existing) = symbols.iter_mut().find(|s: &&mut SymbolRecord| s.id == symbol_id) {
                                            existing.declarations.push(decl_id.clone());
                                            if is_definition {
                                                existing.definition = Some(decl_id);
                                            }
                                        } else {
                                            symbols.push(SymbolRecord {
                                                id: symbol_id,
                                                name: name.clone(),
                                                kind: SymbolKind::Function,
                                                owning_file: tu_file_id.clone(),
                                                declarations: vec![decl_id.clone()],
                                                definition: if is_definition { Some(decl_id) } else { None },
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Check for variable declarations inside class
                    if let Some((ref class_name, _)) = current_class {
                        if i + 1 < tokens.len() {
                            let type_name = w.clone();
                            if let TokenKind::Word(_) = &tokens[i + 1].kind {
                                if type_name != "void" && type_name != "return" {
                                    let mut is_function = false;
                                    if i + 2 < tokens.len() {
                                        if let TokenKind::Punct(p) = &tokens[i + 2].kind {
                                            if p == "(" {
                                                is_function = true;
                                            }
                                        }
                                    }
                                    
                                    edges.push(SemanticEdge {
                                        src_symbol: SymbolId::new(class_name.clone()),
                                        dst_symbol: SymbolId::new(type_name),
                                        kind: if is_function { DependencyKind::Incomplete } else { DependencyKind::Complete },
                                        src_file: tu_file_id.clone(),
                                        dst_file: tu_file_id.clone(),
                                    });
                                }
                            } else if let TokenKind::Punct(p) = &tokens[i + 1].kind {
                                if p == "*" || p == "&" {
                                    edges.push(SemanticEdge {
                                        src_symbol: SymbolId::new(class_name.clone()),
                                        dst_symbol: SymbolId::new(type_name),
                                        kind: DependencyKind::Incomplete,
                                        src_file: tu_file_id.clone(),
                                        dst_file: tu_file_id.clone(),
                                    });
                                } else if p == "<" {
                                    if type_name == "unique_ptr" || type_name == "shared_ptr" {
                                        if i + 2 < tokens.len() {
                                            if let TokenKind::Word(target) = &tokens[i + 2].kind {
                                                edges.push(SemanticEdge {
                                                    src_symbol: SymbolId::new(class_name.clone()),
                                                    dst_symbol: SymbolId::new(target.clone()),
                                                    kind: DependencyKind::Incomplete,
                                                    src_file: tu_file_id.clone(),
                                                    dst_file: tu_file_id.clone(),
                                                });
                                            }
                                        }
                                    }
                                } else if p == "(" {
                                    if i > 0 {
                                        if let TokenKind::Word(_) = &tokens[i - 1].kind {
                                            // Something like `new B()`
                                        } else {
                                            edges.push(SemanticEdge {
                                                src_symbol: SymbolId::new(class_name.clone()),
                                                dst_symbol: SymbolId::new(type_name),
                                                kind: DependencyKind::Complete,
                                                src_file: tu_file_id.clone(),
                                                dst_file: tu_file_id.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }

        TuAnalysis {
            tu_file_id,
            context_name: context_name.to_string(),
            symbols,
            declarations,
            edges,
            references: vec![],
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
}
