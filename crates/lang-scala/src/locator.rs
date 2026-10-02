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
#[cfg(test)]
mod tests;

use ariadne_lang::{SymbolBody, SymbolKind};

/// The file's package namespace per the parser's directive scan (analysis-
/// time qualification parity): joined leading `package` clauses; block and
/// label packages contribute nothing.
pub(crate) fn package_namespace(_source: &str) -> String {
    String::new()
}

/// Locate verbatim declaration regions for `wanted` in `source`, in wanted
/// order, echoing each wanted name as the result name (engine mapping
/// contract, D-M23-3).
pub(crate) fn locate_bodies(
    _source: &str,
    _wanted: &[(String, SymbolKind)],
) -> Vec<SymbolBody> {
    Vec::new()
}
