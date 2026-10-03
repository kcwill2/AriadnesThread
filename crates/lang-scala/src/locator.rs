//! Verbatim declaration-region locator for Scala sources (plan M23, T5,
//! D-M23-1) — the Scala sibling of `lang-cpp`'s `symbol_locator`.
//!
//! Regions come from the *same* passes the analyzer uses
//! ([`crate::parser::program_of`]): the scanner removes comment/string
//! contents, so an occurrence inside a comment or literal never produces a
//! declaration to match, and brace-matching is exact. The slice is the
//! scanner-token span text: from the first char of the declaration's first
//! token (modifiers included, doc comments excluded) through the last char
//! of its last token — never trailing whitespace or a following comment.
//!
//! Name matching mirrors the C++ locator's discipline, ported to `.`-joined
//! names: the wanted name and the declaration's qualified id must be equal
//! or a segment-suffix of one another; kind must match exactly (a companion
//! `object X` lives under the D17 id `<pkg>.X$` and is reachable by that id,
//! never by a bare `X`); the first (source-order) match wins; unresolved
//! names are omitted so the engine keeps its placeholder (I4).

#[cfg(test)]
mod corpus;
pub(crate) mod removal;
#[cfg(test)]
mod tests;

use crate::parser::{program_of, ResolvedDecl, SourceProgram};
use ariadne_lang::{SymbolBody, SymbolKind};

/// The file's package namespace per the parser's directive scan (analysis-
/// time qualification parity): joined leading `package` clauses; block and
/// label packages contribute nothing.
pub(crate) fn package_namespace(source: &str) -> String {
    program_of(source).namespace
}

/// Locate verbatim declaration regions for `wanted` in `source`, in wanted
/// order, echoing each wanted name as the result name (engine mapping
/// contract, D-M23-3).
pub(crate) fn locate_bodies(
    source: &str,
    wanted: &[(String, SymbolKind)],
) -> Vec<SymbolBody> {
    let program = program_of(source);
    wanted
        .iter()
        .filter_map(|(name, kind)| {
            program
                .matching_decl(name, *kind)
                .map(|decl| SymbolBody {
                    name: name.clone(),
                    body: program.region_text(source, decl),
                })
        })
        .collect()
}

impl SourceProgram {
    /// Source-order first declaration with exactly `kind` whose qualified id
    /// suffix-matches `wanted`.
    fn matching_decl(&self, wanted: &str, kind: SymbolKind) -> Option<&ResolvedDecl> {
        self.decls
            .iter()
            .find(|decl| decl.kind == kind && qualified_suffix_matches(wanted, &decl.id.0))
    }

    /// The declaration's verbatim text: first char of its first token
    /// through the last char of its last token. Braceless regions end at
    /// the parser's boundary rule (`span_end_exclusive`), so no trailing
    /// whitespace or neighbour comment enters the slice.
    fn region_text(&self, source: &str, decl: &ResolvedDecl) -> String {
        let last_token = decl
            .span_end_exclusive
            .min(self.tokens.len())
            .max(decl.span_start + 1)
            - 1;
        let from = self.tokens[decl.span_start].start;
        let to = self.tokens[last_token].end;
        source.chars().skip(from).take(to - from).collect()
    }
}

/// True when the wanted qualified name and the declaration's qualified id
/// are equal, or one is a segment-suffix of the other (`.`-joined names,
/// mirroring the C++ `qualified_suffix_matches` discipline).
pub(crate) fn qualified_suffix_matches(wanted: &str, qualified: &str) -> bool {
    let wanted_segments: Vec<&str> = wanted.split('.').collect();
    let qualified_segments: Vec<&str> = qualified.split('.').collect();
    (qualified_segments.len() <= wanted_segments.len()
        && wanted_segments.ends_with(&qualified_segments))
        || (wanted_segments.len() <= qualified_segments.len()
            && qualified_segments.ends_with(&wanted_segments))
}
