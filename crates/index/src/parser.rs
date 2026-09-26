use crate::tokenizer::{Token, TokenKind};
use crate::types::*;
use ariadne_repo::FileId;

pub(crate) struct ParserCtx<'a> {
    tu_file_id: FileId,
    pub(crate) tokens: &'a [Token],
    pub(crate) edges: Vec<SemanticEdge>,
    pub(crate) symbols: Vec<SymbolRecord>,
    pub(crate) declarations: Vec<SymbolDecl>,
    pub(crate) call_sites: Vec<CallSite>,
    current_class: Option<(String, usize)>,
    current_function: Option<(SymbolId, usize)>,
    brace_depth: usize,
    decl_counter: usize,
    pub(crate) i: usize,
}

impl<'a> ParserCtx<'a> {
    pub(crate) fn new(tu_file_id: FileId, tokens: &'a [Token]) -> Self {
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

    pub(crate) fn handle_brace_open(&mut self) {
        self.brace_depth += 1;
    }

    pub(crate) fn handle_brace_close(&mut self) {
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

    pub(crate) fn handle_class_or_struct(&mut self, w: &str) {
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
                                if base_w != "public"
                                    && base_w != "private"
                                    && base_w != "protected"
                                {
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

            let kind = if w == "class" {
                SymbolKind::Class
            } else {
                SymbolKind::Struct
            };

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

    pub(crate) fn handle_sizeof(&mut self) {
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

    pub(crate) fn handle_new(&mut self) {
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

    pub(crate) fn handle_word(&mut self, w: &str, token: &Token) {
        self.handle_call_site(w, token);
        self.handle_global_function(w, token);
        self.handle_variable_decl(w);
    }

    fn handle_call_site(&mut self, w: &str, token: &Token) {
        if let Some((ref func_symbol, _)) = self.current_function {
            if self.brace_depth > 0 && self.i + 1 < self.tokens.len() {
                if let TokenKind::Punct(p) = &self.tokens[self.i + 1].kind {
                    if p == "(" {
                        let reserved = [
                            "if", "while", "for", "switch", "return", "sizeof", "catch", "new",
                            "delete", "throw",
                        ];
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
        if self.current_function.is_none() && self.i + 1 < self.tokens.len() {
            if let TokenKind::Punct(p) = &self.tokens[self.i + 1].kind {
                if p == "(" {
                    let reserved = [
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
                    ];
                    if reserved.contains(&w) {
                        return;
                    }

                    let mut name = w.to_string();
                    if let Some((ref class_name, _)) = self.current_class {
                        name = format!("{}::{}", class_name, name);
                    } else if self.i >= 2 {
                        // Check if it's a qualified name like MyClass::method
                        if let TokenKind::Punct(colon1) = &self.tokens[self.i - 1].kind {
                            if colon1 == ":" {
                                if let TokenKind::Punct(colon2) = &self.tokens[self.i - 2].kind {
                                    if colon2 == ":" && self.i >= 3 {
                                        if let TokenKind::Word(class_name) =
                                            &self.tokens[self.i - 3].kind
                                        {
                                            name = format!("{}::{}", class_name, name);
                                        }
                                    }
                                }
                            }
                        }
                    }
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
                            if p2 == "(" {
                                paren_depth += 1;
                            } else if p2 == ")" {
                                paren_depth -= 1;
                            }
                        }
                        j += 1;
                    }

                    while j < self.tokens.len() {
                        if let TokenKind::Punct(p3) = &self.tokens[j].kind {
                            if p3 == "{" {
                                is_definition = true;
                                self.current_function =
                                    Some((symbol_id.clone(), self.brace_depth + 1));
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
                            id: symbol_id.clone(),
                            name: name.clone(),
                            kind: SymbolKind::Function,
                            owning_file: self.tu_file_id.clone(),
                            declarations: vec![decl_id.clone()],
                            definition: if is_definition { Some(decl_id) } else { None },
                        });
                    }

                    // Extract return type edge
                    if self.i > 0 {
                        if let TokenKind::Word(ret_type) = &self.tokens[self.i - 1].kind {
                            if ret_type != "void"
                                && ret_type != "virtual"
                                && ret_type != "static"
                                && ret_type != "inline"
                            {
                                self.edges.push(SemanticEdge {
                                    src_symbol: symbol_id.clone(),
                                    dst_symbol: SymbolId::new(ret_type.clone()),
                                    kind: DependencyKind::Incomplete,
                                    src_file: self.tu_file_id.clone(),
                                    dst_file: self.tu_file_id.clone(),
                                });
                            }
                        }
                    }

                    // Extract parameter edges
                    let mut k = self.i + 2;
                    let mut param_paren = 1;
                    while k < self.tokens.len() && param_paren > 0 {
                        match &self.tokens[k].kind {
                            TokenKind::Punct(p) => {
                                if p == "(" {
                                    param_paren += 1;
                                } else if p == ")" {
                                    param_paren -= 1;
                                }
                            }
                            TokenKind::Word(param_type) => {
                                // If followed by a word (param name) or * or & or , or ), it might be a type
                                if param_type != "const" {
                                    if k + 1 < self.tokens.len() {
                                        let next = &self.tokens[k + 1].kind;
                                        if matches!(next, TokenKind::Word(_))
                                            || matches!(next, TokenKind::Punct(p) if p == "*" || p == "&")
                                        {
                                            self.edges.push(SemanticEdge {
                                                src_symbol: symbol_id.clone(),
                                                dst_symbol: SymbolId::new(param_type.clone()),
                                                kind: DependencyKind::Incomplete, // pointers/refs are incomplete, by value incomplete for params is usually fine
                                                src_file: self.tu_file_id.clone(),
                                                dst_file: self.tu_file_id.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        k += 1;
                    }
                }
            }
        }
    }

    fn handle_variable_decl(&mut self, w: &str) {
        let parent_symbol = if let Some((ref class_name, _)) = self.current_class {
            Some(class_name.clone())
        } else if let Some((ref func_id, _)) = self.current_function {
            Some(func_id.0.clone())
        } else {
            None
        };

        if let Some(parent) = parent_symbol {
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
                            src_symbol: SymbolId::new(parent.clone()),
                            dst_symbol: SymbolId::new(type_name),
                            kind: if is_function {
                                DependencyKind::Incomplete
                            } else {
                                DependencyKind::Complete
                            },
                            src_file: self.tu_file_id.clone(),
                            dst_file: self.tu_file_id.clone(),
                        });
                    }
                } else if let TokenKind::Punct(p) = &self.tokens[self.i + 1].kind {
                    if p == "*" || p == "&" {
                        self.edges.push(SemanticEdge {
                            src_symbol: SymbolId::new(parent.clone()),
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
                                        src_symbol: SymbolId::new(parent.clone()),
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
                                    src_symbol: SymbolId::new(parent.clone()),
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

#[cfg(test)]
mod characterization_tests;
