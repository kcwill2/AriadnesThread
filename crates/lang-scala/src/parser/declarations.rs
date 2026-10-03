//! Top-level declaration recognition (D17).
//!
//! Only brace-depth-0 declarations become tracked symbols. Body spans are
//! brace-matched *through* the scanner (comments/strings are single tokens,
//! so raw depth counting is exact), which (a) ends the top-level region,
//! (b) provides the enclosing symbol for references/call-sites, and (c)
//! keeps nested declarations out of symbol output. Span text is not stored
//! (M23 concern).

use super::directives::{self, DirectiveScan};
use crate::scanner::Token;
use ariadne_lang_api::types::{DeclId, SymbolId, SymbolKind};

/// A recognized top-level declaration before id resolution.
pub(crate) struct TrackedDecl {
    pub simple_name: String,
    pub kind: SymbolKind,
    pub name_token: usize,
    pub span_start: usize,
    /// Exclusive end when a brace body was matched; `None` means the span
    /// runs until the next top-level boundary (braceless `def`/`val`).
    pub span_end: Option<usize>,
    pub is_package_object: bool,
    /// Modifiers the analysis deliberately records — `sealed` only
    /// (plan M23, D-M23-8); every other modifier keyword is consumed
    /// silently as before.
    pub modifiers: Vec<String>,
}

pub(crate) struct DeclPass {
    pub decls: Vec<TrackedDecl>,
    pub name_tokens: Vec<usize>,
}

/// A declaration after companion-id resolution (D17: `object X` next to a
/// class-ish `X` in the same file takes the id `<qual>.X$`).
pub(crate) struct ResolvedDecl {
    pub simple_name: String,
    pub kind: SymbolKind,
    pub id: SymbolId,
    pub decl_id: DeclId,
    pub name_line: usize,
    pub name_column: usize,
    pub span_start: usize,
    pub span_end_exclusive: usize,
    pub modifiers: Vec<String>,
}

/// Kind mapping per D17 for the keywords that map independently of context.
pub(crate) fn declaration_kind_for(keyword: &str) -> Option<SymbolKind> {
    match keyword {
        "class" => Some(SymbolKind::Class),
        "trait" => Some(SymbolKind::Trait),
        "object" => Some(SymbolKind::Object),
        "enum" => Some(SymbolKind::Enum),
        "def" => Some(SymbolKind::Function),
        "val" => Some(SymbolKind::Variable),
        _ => None,
    }
}

/// D17: `given` with a value parameter list is a Function, otherwise a
/// Variable.
pub(crate) fn given_is_function(has_parameter_list: bool) -> SymbolKind {
    if has_parameter_list {
        SymbolKind::Function
    } else {
        SymbolKind::Variable
    }
}

/// D17 anonymous-given heuristic: named givens follow the camelCase
/// convention; a non-lowercase start means an anonymous given type.
fn starts_with_lowercase(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_lowercase)
}

/// Declaration modifiers that may precede a declaration keyword and are not
/// part of the construct itself (`sealed trait X`, `final case class Y`, …).
pub(crate) fn is_modifier_keyword(word: &str) -> bool {
    matches!(
        word,
        "abstract"
            | "final"
            | "implicit"
            | "lazy"
            | "override"
            | "private"
            | "protected"
            | "sealed"
    )
}

pub(crate) fn collect_top_level_decls(tokens: &[Token], directives: &DirectiveScan) -> DeclPass {
    let mut pass = DeclPass {
        decls: Vec::new(),
        name_tokens: Vec::new(),
    };
    let mut depth: i64 = 0;
    let mut index = 0usize;
    while index < tokens.len() {
        if let Some(end) = directives::consumed_end(&directives.consumed_ranges, index) {
            index = end;
            continue;
        }
        match tokens[index].kind {
            crate::scanner::TokenKind::Punct('{') => depth += 1,
            crate::scanner::TokenKind::Punct('}') => depth = (depth - 1).max(0),
            _ => {}
        }
        if depth == 0 {
            if let Some((decl, next)) = recognize_declaration(tokens, index) {
                pass.name_tokens.push(decl.name_token);
                pass.decls.push(decl);
                index = next;
                continue;
            }
        }
        index += 1;
    }
    pass
}

/// Recognizes one top-level declaration starting at `index`, skipping
/// modifiers, type-parameter and value-parameter lists; returns the tracked
/// decl plus the resume index (the body/`=` region is left to the walk).
fn recognize_declaration(tokens: &[Token], index: usize) -> Option<(TrackedDecl, usize)> {
    let mut cursor = index;
    let mut modifiers: Vec<String> = Vec::new();
    while tokens
        .get(cursor)
        .and_then(Token::ident)
        .is_some_and(is_modifier_keyword)
    {
        if tokens.get(cursor).and_then(Token::ident) == Some("sealed") {
            modifiers.push("sealed".to_string());
        }
        cursor += 1;
        if tokens.get(cursor).is_some_and(|t| t.is_punct('[')) {
            cursor = skip_balanced(tokens, cursor, '[', ']');
        }
    }
    let keyword = tokens.get(cursor).and_then(Token::ident)?;
    let (keyword, keyword_index) = match keyword {
        "case"
            if matches!(
                tokens.get(cursor + 1).and_then(Token::ident),
                Some("class") | Some("object")
            ) =>
        {
            (tokens[cursor + 1].ident()?, cursor + 1)
        }
        "package" if tokens.get(cursor + 1).and_then(Token::ident) == Some("object") => {
            return recognize_package_object(tokens, index, cursor + 2)
        }
        "class" | "trait" | "object" | "enum" | "def" | "val" | "given" => (keyword, cursor),
        _ => return None,
    };
    let name_index = keyword_index + 1;
    let name = tokens.get(name_index).and_then(Token::ident)?;
    if keyword == "given" && !starts_with_lowercase(name) {
        return None; // anonymous given: documented limitation, no symbol
    }
    let mut after_name = name_index + 1;
    after_name = skip_balanced_if_angle(tokens, after_name);
    let given_has_parameter_list = tokens.get(after_name).is_some_and(|t| t.is_punct('('));
    let after_parameters = skip_value_parameter_lists(tokens, after_name);
    let kind = match keyword {
        "given" => given_is_function(given_has_parameter_list),
        other => declaration_kind_for(other)?,
    };
    let span_end = if is_classish_or_container(kind) {
        find_class_body_end(tokens, after_parameters)
    } else {
        None
    };
    Some((
        TrackedDecl {
            simple_name: name.to_string(),
            kind,
            name_token: name_index,
            span_start: index,
            span_end,
            is_package_object: false,
            modifiers,
        },
        after_parameters,
    ))
}

fn recognize_package_object(
    tokens: &[Token],
    index: usize,
    name_index: usize,
) -> Option<(TrackedDecl, usize)> {
    let name = tokens.get(name_index).and_then(Token::ident)?;
    let span_end = find_class_body_end(tokens, name_index + 1);
    Some((
        TrackedDecl {
            simple_name: name.to_string(),
            kind: SymbolKind::Object,
            name_token: name_index,
            span_start: index,
            span_end,
            is_package_object: true,
            modifiers: Vec::new(),
        },
        name_index + 1,
    ))
}

fn is_classish_or_container(kind: SymbolKind) -> bool {
    matches!(
        kind,
        SymbolKind::Class | SymbolKind::Trait | SymbolKind::Object | SymbolKind::Enum
    )
}

/// True when a declaration-start word, `=`, or `;` ends the header before a
/// body brace (making the declaration braceless).
fn find_class_body_end(tokens: &[Token], from: usize) -> Option<usize> {
    let mut cursor = from;
    while cursor < tokens.len() {
        let token = &tokens[cursor];
        if token.is_punct('{') {
            return Some(super::directives::matching_brace_end(tokens, cursor));
        }
        if token.is_punct('=') || token.is_punct(';') {
            return None;
        }
        if token.ident().is_some_and(is_declaration_start_word) {
            return None;
        }
        cursor += 1;
    }
    None
}

fn is_declaration_start_word(word: &str) -> bool {
    matches!(
        word,
        "class"
            | "trait"
            | "object"
            | "enum"
            | "def"
            | "val"
            | "case"
            | "given"
            | "package"
            | "import"
            | "export"
    )
}

/// Skips a type-parameter list after a declaration name, accepting both the
/// Scala-2 `<…>` and Scala-3 `[…]` bracket forms.
fn skip_balanced_if_angle(tokens: &[Token], index: usize) -> usize {
    if tokens.get(index).is_some_and(|t| t.is_punct('<')) {
        skip_balanced(tokens, index, '<', '>')
    } else if tokens.get(index).is_some_and(|t| t.is_punct('[')) {
        skip_balanced(tokens, index, '[', ']')
    } else {
        index
    }
}

fn skip_value_parameter_lists(tokens: &[Token], mut index: usize) -> usize {
    while tokens.get(index).is_some_and(|t| t.is_punct('(')) {
        index = skip_balanced(tokens, index, '(', ')');
    }
    index
}

fn skip_balanced(tokens: &[Token], open_index: usize, open: char, close: char) -> usize {
    let mut depth = 0i64;
    let mut cursor = open_index;
    while cursor < tokens.len() {
        if tokens[cursor].is_punct(open) {
            depth += 1;
        } else if tokens[cursor].is_punct(close) {
            depth -= 1;
            if depth == 0 {
                return cursor + 1;
            }
        }
        cursor += 1;
    }
    tokens.len()
}

pub(crate) fn resolve_decl_ids(
    decls: Vec<TrackedDecl>,
    tokens: &[Token],
    directives: &DirectiveScan,
) -> Vec<ResolvedDecl> {
    let classish_names: std::collections::BTreeSet<String> = decls
        .iter()
        .filter(|decl| {
            matches!(
                decl.kind,
                SymbolKind::Class | SymbolKind::Trait | SymbolKind::Enum
            )
        })
        .map(|decl| decl.simple_name.clone())
        .collect();
    let boundaries = span_boundaries(&decls, directives, tokens.len());
    decls
        .into_iter()
        .map(|decl| {
            let dollar_suffixed = decl.kind == SymbolKind::Object
                && !decl.is_package_object
                && classish_names.contains(decl.simple_name.as_str());
            let qualified =
                qualified_name(&directives.namespace, &decl.simple_name, dollar_suffixed);
            let name_token = &tokens[decl.name_token];
            let span_end_exclusive = decl
                .span_end
                .unwrap_or_else(|| next_boundary(&boundaries, decl.span_start));
            ResolvedDecl {
                simple_name: decl.simple_name,
                kind: decl.kind,
                id: SymbolId::new(&qualified),
                decl_id: DeclId::new(format!("{}#L{}", qualified, name_token.line)),
                name_line: name_token.line,
                name_column: name_token.column,
                span_start: decl.span_start,
                span_end_exclusive,
                modifiers: decl.modifiers,
            }
        })
        .collect()
}

pub(crate) fn qualified_name(namespace: &str, simple_name: &str, dollar_suffixed: bool) -> String {
    let mut qualified = if namespace.is_empty() {
        simple_name.to_string()
    } else {
        format!("{namespace}.{simple_name}")
    };
    if dollar_suffixed {
        qualified.push('$');
    }
    qualified
}

/// Sorted token indices where a braceless top-level span must stop: the
/// next declaration start, the next consumed directive range start, or EOF.
fn span_boundaries(
    decls: &[TrackedDecl],
    directives: &DirectiveScan,
    token_count: usize,
) -> Vec<usize> {
    let mut boundaries: Vec<usize> = decls
        .iter()
        .map(|decl| decl.span_start)
        .chain(directives.consumed_ranges.iter().map(|(start, _)| *start))
        .collect();
    boundaries.push(token_count);
    boundaries.sort_unstable();
    boundaries.dedup();
    boundaries
}

fn next_boundary(boundaries: &[usize], after: usize) -> usize {
    boundaries
        .iter()
        .copied()
        .find(|boundary| *boundary > after)
        .unwrap_or(usize::MAX)
}

pub(crate) fn symbol_payloads(
    resolved: &[ResolvedDecl],
    file_id: &ariadne_repo::FileId,
) -> (
    Vec<ariadne_lang_api::types::SymbolRecord>,
    Vec<ariadne_lang_api::types::SymbolDecl>,
) {
    let symbols = resolved
        .iter()
        .map(|decl| ariadne_lang_api::types::SymbolRecord {
            id: decl.id.clone(),
            name: decl.simple_name.clone(),
            kind: decl.kind,
            owning_file: file_id.clone(),
            declarations: vec![decl.decl_id.clone()],
            definition: Some(decl.decl_id.clone()),
            modifiers: decl.modifiers.clone(),
        })
        .collect();
    let declarations = resolved
        .iter()
        .map(|decl| ariadne_lang_api::types::SymbolDecl {
            id: decl.decl_id.clone(),
            symbol_id: decl.id.clone(),
            location: ariadne_lang_api::types::FileLocation {
                file_id: file_id.clone(),
                line: decl.name_line,
                column: decl.name_column,
            },
            is_definition: true,
        })
        .collect();
    (symbols, declarations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d17_keyword_kind_mapping() {
        assert_eq!(declaration_kind_for("class"), Some(SymbolKind::Class));
        assert_eq!(declaration_kind_for("trait"), Some(SymbolKind::Trait));
        assert_eq!(declaration_kind_for("object"), Some(SymbolKind::Object));
        assert_eq!(declaration_kind_for("enum"), Some(SymbolKind::Enum));
        assert_eq!(declaration_kind_for("def"), Some(SymbolKind::Function));
        assert_eq!(declaration_kind_for("val"), Some(SymbolKind::Variable));
        assert_eq!(declaration_kind_for("extends"), None);
    }

    #[test]
    fn given_kind_follows_parameter_list() {
        assert_eq!(given_is_function(true), SymbolKind::Function);
        assert_eq!(given_is_function(false), SymbolKind::Variable);
    }

    #[test]
    fn modifiers_are_recognized() {
        assert!(is_modifier_keyword("sealed"));
        assert!(is_modifier_keyword("implicit"));
        assert!(!is_modifier_keyword("class"));
    }
}
