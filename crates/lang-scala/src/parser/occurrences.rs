//! Reference, edge, and call-site extraction (D16).
//!
//! Every occurrence of a *resolvable* name yields a `SymbolReference` (all
//! occurrences kept — the load metric's `tu_usage` consumes them) plus a
//! deduplicated `SemanticEdge` from the enclosing tracked symbol. The
//! resolution chain per occurrence (first hit wins):
//! 1. a named import selector (a dotted chain is matched whole against
//!    import targets);
//! 2. an in-file tracked declaration — tie-break per D17: type position
//!    takes the class-ish id, call position `X(` takes the companion
//!    object id (`<name>$`) when one exists;
//! 3. the single active wildcard base ⇒ an `Incomplete` edge + reference to
//!    the qualified guess, gated to uppercase, non-intrinsic names.
//! Unresolvable non-parent names produce nothing except a raw-named
//! `CallSite` for `X(` / `new X` (the documented "pick one rule" = nothing).
//! `extends`/`with` parents always emit an edge (Complete when resolvable,
//! else Incomplete with a `<package>.<Name>` qualified guess) and reference
//! only when resolvable.
//!
//! Intrinsic names (Scala value types / common stdlib, see the list below)
//! are never attributed to a wildcard import, so wildcard-heavy files do
//! not fabricate `scala.collection.mutable.Int`-style edges — a documented
//! limitation of the offline heuristic.

use super::declarations::{qualified_name, ResolvedDecl};
use super::directives::{consumed_end, read_dotted_chain};
use crate::scanner::Token;
use ariadne_lang_api::types::{
    CallSite, DependencyKind, FileLocation, SemanticEdge, SymbolId, SymbolKind, SymbolReference,
};
use ariadne_repo::FileId;
use std::collections::BTreeMap;

pub(crate) struct OccurrencePass {
    pub edges: Vec<SemanticEdge>,
    pub references: Vec<SymbolReference>,
    pub call_sites: Vec<CallSite>,
    /// Typed `(subtype, supertype)` pairs from `extends`/`with` parent
    /// positions only (plan M23, D-M23-8) — never occurrence edges.
    pub extends_edges: Vec<(SymbolId, SymbolId)>,
}

pub(crate) struct Scope<'a> {
    pub namespace: &'a str,
    pub imports: &'a BTreeMap<String, String>,
    pub wildcard_base: Option<&'a str>,
    pub decls: &'a [ResolvedDecl],
    pub skip: &'a [(usize, usize)],
    pub name_tokens: &'a [usize],
}

/// simple name -> (class-ish declaration, object declaration) for in-file
/// resolution with the D17 tie-break.
struct InFileIndex<'a> {
    entries: BTreeMap<&'a str, InFilePair<'a>>,
}

#[derive(Default)]
struct InFilePair<'a> {
    classish: Option<&'a ResolvedDecl>,
    object: Option<&'a ResolvedDecl>,
}

impl<'a> InFileIndex<'a> {
    fn build(decls: &'a [ResolvedDecl]) -> Self {
        let mut entries: BTreeMap<&str, InFilePair> = BTreeMap::new();
        for decl in decls {
            let pair = entries.entry(decl.simple_name.as_str()).or_default();
            match pair {
                pair if decl.kind == SymbolKind::Object && pair.object.is_none() => {
                    pair.object = Some(decl)
                }
                pair if decl.kind != SymbolKind::Object && pair.classish.is_none() => {
                    pair.classish = Some(decl)
                }
                _ => {}
            }
        }
        Self { entries }
    }

    fn resolve(&self, name: &str, prefer_object: bool) -> Option<&'a ResolvedDecl> {
        let pair = self.entries.get(name)?;
        if prefer_object {
            pair.object.or(pair.classish)
        } else {
            pair.classish.or(pair.object)
        }
    }
}

pub(crate) fn scan_occurrences(
    tokens: &[Token],
    file_id: &FileId,
    scope: &Scope<'_>,
) -> OccurrencePass {
    let in_file = InFileIndex::build(scope.decls);
    let mut pass = OccurrencePass {
        edges: Vec::new(),
        references: Vec::new(),
        call_sites: Vec::new(),
        extends_edges: Vec::new(),
    };
    let mut cursor = 0usize;
    while cursor < tokens.len() {
        if let Some(end) = consumed_end(scope.skip, cursor) {
            cursor = end;
            continue;
        }
        let Some(name) = tokens[cursor].ident() else {
            cursor += 1;
            continue;
        };
        if is_language_keyword(name) {
            cursor += 1;
            continue;
        }
        let prev = cursor.checked_sub(1).map(|index| &tokens[index]);
        let next = tokens.get(cursor + 1);
        let enclosing = enclosing_symbol(scope.decls, cursor);
        if is_parent_position(prev) {
            let (segments, after_chain) = read_dotted_chain(tokens, cursor);
            emit_parent_edge(&tokens[cursor], &segments, file_id, scope, enclosing, &in_file, &mut pass);
            cursor = after_chain;
            continue;
        }
        if !scope.name_tokens.contains(&cursor) {
            if is_invoked(prev, next) {
                push_call_site(&tokens[cursor], name, file_id, enclosing, &mut pass);
            }
            if !is_member_access(prev) {
                emit_reference_and_edge(
                    &tokens[cursor], name, next, file_id, scope, enclosing, &in_file, &mut pass,
                );
            }
        }
        cursor += 1;
    }
    pass
}

fn is_parent_position(prev: Option<&Token>) -> bool {
    matches!(prev.and_then(Token::ident), Some("extends") | Some("with"))
}

fn is_member_access(prev: Option<&Token>) -> bool {
    prev.is_some_and(|token| token.is_punct('.'))
}

/// `X(` or `new X`, excluding definition heads (`def f(`, `class X(`, …) so
/// definitions are never mistaken for calls.
fn is_invoked(prev: Option<&Token>, next: Option<&Token>) -> bool {
    let invoked_by_paren = next.is_some_and(|token| token.is_punct('('));
    let invoked_by_new = prev.and_then(Token::ident) == Some("new");
    let definition_head = prev
        .and_then(Token::ident)
        .is_some_and(is_definition_head_keyword);
    (invoked_by_paren || invoked_by_new) && !definition_head
}

fn is_definition_head_keyword(word: &str) -> bool {
    matches!(
        word,
        "class" | "trait" | "object" | "enum" | "def" | "val" | "given"
    )
}

fn push_call_site(
    token: &Token,
    name: &str,
    file_id: &FileId,
    enclosing: Option<&ResolvedDecl>,
    pass: &mut OccurrencePass,
) {
    let Some(caller) = enclosing else { return };
    pass.call_sites.push(CallSite {
        caller_symbol: caller.id.clone(),
        callee_name: name.to_string(),
        location: token_location(token, file_id),
    });
}

fn token_location(token: &Token, file_id: &FileId) -> FileLocation {
    FileLocation {
        file_id: file_id.clone(),
        line: token.line,
        column: token.column,
    }
}

/// One simple-name occurrence outside imports/parents: resolve, then emit a
/// reference (always, when resolved) and an edge from the enclosing tracked
/// symbol (when one exists).
fn emit_reference_and_edge(
    token: &Token,
    name: &str,
    next: Option<&Token>,
    file_id: &FileId,
    scope: &Scope<'_>,
    enclosing: Option<&ResolvedDecl>,
    in_file: &InFileIndex<'_>,
    pass: &mut OccurrencePass,
) {
    let is_call = next.is_some_and(|token| token.is_punct('('));
    let Some((dst_symbol, kind)) = resolve_occurrence(name, is_call, scope, in_file) else {
        return;
    };
    pass.references.push(SymbolReference {
        symbol_id: SymbolId::new(&dst_symbol),
        file_id: file_id.clone(),
        location: token_location(token, file_id),
        is_write: false,
    });
    if let Some(src) = enclosing {
        pass.edges.push(SemanticEdge {
            src_symbol: src.id.clone(),
            dst_symbol: SymbolId::new(&dst_symbol),
            kind,
            src_file: file_id.clone(),
            dst_file: file_id.clone(),
        });
    }
}

fn resolve_occurrence(
    name: &str,
    is_call: bool,
    scope: &Scope<'_>,
    in_file: &InFileIndex<'_>,
) -> Option<(String, DependencyKind)> {
    if let Some(target) = scope.imports.get(name) {
        return Some((target.clone(), DependencyKind::Complete));
    }
    if let Some(decl) = in_file.resolve(name, is_call) {
        return Some((decl.id.0.clone(), DependencyKind::Complete));
    }
    if let Some(base) = scope.wildcard_base {
        if is_wildcard_attributable(name) {
            return Some((format!("{base}.{name}"), DependencyKind::Incomplete));
        }
    }
    None
}

/// Wildcard attribution is restricted to uppercase, non-intrinsic names —
/// lowercase identifiers are locals/parameters/methods, and Scala's
/// predefined types would otherwise produce absurd guesses.
fn is_wildcard_attributable(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase) && !is_intrinsic_type_name(name)
}

fn emit_parent_edge(
    token: &Token,
    segments: &[String],
    file_id: &FileId,
    scope: &Scope<'_>,
    enclosing: Option<&ResolvedDecl>,
    in_file: &InFileIndex<'_>,
    pass: &mut OccurrencePass,
) {
    let Some(src) = enclosing else { return };
    let full = segments.join(".");
    let (dst_symbol, kind) = if scope.imports.values().any(|target| *target == full) {
        (full.clone(), DependencyKind::Complete)
    } else if segments.len() == 1 {
        match in_file.resolve(&full, false) {
            Some(decl) => (decl.id.0.clone(), DependencyKind::Complete),
            None => (
                qualified_name(scope.namespace, &full, false),
                DependencyKind::Incomplete,
            ),
        }
    } else {
        (full.clone(), DependencyKind::Incomplete)
    };
    if kind == DependencyKind::Complete {
        pass.references.push(SymbolReference {
            symbol_id: SymbolId::new(&dst_symbol),
            file_id: file_id.clone(),
            location: token_location(token, file_id),
            is_write: false,
        });
    }
    pass.edges.push(SemanticEdge {
        src_symbol: src.id.clone(),
        dst_symbol: SymbolId::new(&dst_symbol),
        kind,
        src_file: file_id.clone(),
        dst_file: file_id.clone(),
    });
    pass.extends_edges
        .push((src.id.clone(), SymbolId::new(&dst_symbol)));
}

/// Innermost tracked span containing the token (spans do not nest; the last
/// match is the enclosing top-level declaration).
fn enclosing_symbol<'a>(decls: &'a [ResolvedDecl], token_index: usize) -> Option<&'a ResolvedDecl> {
    decls
        .iter()
        .rev()
        .find(|decl| decl.span_start <= token_index && token_index < decl.span_end_exclusive)
}

fn is_language_keyword(word: &str) -> bool {
    matches!(
        word,
        "abstract"
            | "case"
            | "catch"
            | "class"
            | "def"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "final"
            | "finally"
            | "for"
            | "given"
            | "if"
            | "implicit"
            | "import"
            | "lazy"
            | "match"
            | "new"
            | "object"
            | "override"
            | "package"
            | "private"
            | "protected"
            | "return"
            | "sealed"
            | "super"
            | "this"
            | "throw"
            | "trait"
            | "try"
            | "type"
            | "val"
            | "var"
            | "while"
            | "with"
            | "yield"
    )
}

/// Scala predefined/stdlib type names that must never be attributed to a
/// wildcard import (documented offline-heuristic limitation).
const INTRINSIC_TYPE_NAMES: [&str; 34] = [
    "Any",
    "AnyRef",
    "AnyVal",
    "App",
    "Array",
    "BigDecimal",
    "BigInt",
    "Boolean",
    "Byte",
    "Char",
    "Double",
    "Either",
    "Enumeration",
    "Float",
    "Int",
    "Iterable",
    "Iterator",
    "Left",
    "List",
    "Long",
    "Map",
    "Nil",
    "None",
    "Nothing",
    "Null",
    "Option",
    "PartialFunction",
    "Right",
    "Seq",
    "Set",
    "Short",
    "Some",
    "String",
    "Unit",
];

pub(crate) fn is_intrinsic_type_name(name: &str) -> bool {
    INTRINSIC_TYPE_NAMES.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predefined_types_are_intrinsic_but_application_types_are_not() {
        assert!(is_intrinsic_type_name("Int"));
        assert!(is_intrinsic_type_name("String"));
        assert!(is_intrinsic_type_name("Unit"));
        assert!(is_intrinsic_type_name("App"));
        assert!(!is_intrinsic_type_name("ArrayBuffer"));
        assert!(!is_intrinsic_type_name("Order"));
    }

    #[test]
    fn wildcard_attribution_requires_uppercase_non_intrinsic() {
        assert!(is_wildcard_attributable("ArrayBuffer"));
        assert!(!is_wildcard_attributable("buffer"));
        assert!(!is_wildcard_attributable("String"));
    }
}
