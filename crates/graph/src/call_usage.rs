//! Calls count as uses.
//!
//! The C++ analysis records a call only as a `CallSite` naming the callee
//! by its bare spelling (`api0`, `size`); the semantic edges carry type and
//! declaration uses but never calls. The load metric walks semantic edges
//! alone, so a function a file calls looked unused and was charged to its
//! header as collateral. This pass turns each call site into a semantic
//! edge from the caller to every same-named symbol the calling unit can
//! see (its own symbols plus everything its headers declare), so calls
//! join the closure without matching unrelated same-named functions
//! elsewhere in the repository.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use ariadne_index::types::{DependencyKind, IndexResult, SymbolId, TuAnalysis};
use ariadne_repo::FileId;
use rayon::prelude::*;

use crate::reduced::ReducedSymbolRecord;

/// The most same-named visible symbols one call may resolve to. A bare
/// name cannot say which class a member call targets, so a call matching
/// more (a common member name such as `get` or `size` seen through many
/// classes) stays unresolved, as every call was before calls counted:
/// fanning it out to all of them inflated each closure, the stage-5 passes
/// and the symbol graph (3.2 GB on a generated 1500-unit repo). Free
/// functions and one class's overloads share one id, so this rarely bites.
pub(crate) const MAX_VISIBLE_CALL_TARGETS: usize = 4;

/// A same-named candidate with its using units resolved up front, so each
/// visibility test is an integer set probe, not a string-keyed map lookup.
type Candidate<'m> = (&'m SymbolId, &'m BTreeSet<FileId>);

/// The unqualified tail of a C++ name: `ns::Type::method` -> `method`.
fn unqualified(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

/// Every used symbol grouped by its unqualified name, in id order. A symbol
/// no unit uses can never be visible to a caller, so it is left out.
fn candidates_by_unqualified_name<'m>(
    symbols: &'m BTreeMap<SymbolId, ReducedSymbolRecord>,
    tu_usage: &'m BTreeMap<SymbolId, BTreeSet<FileId>>,
) -> HashMap<&'m str, Vec<Candidate<'m>>> {
    let mut by_name: HashMap<&str, Vec<Candidate>> = HashMap::new();
    for (id, record) in symbols {
        if let Some(units) = tu_usage.get(id) {
            by_name.entry(unqualified(&record.name)).or_default().push((id, units));
        }
    }
    by_name
}

/// One unit's call edges: each distinct (caller, callee name) pair resolved
/// to the same-named candidates the unit can see, self-calls excluded. Both
/// ends borrow the symbol table's own keys, so pairs dedupe by address.
fn unit_call_edges<'m>(
    tu: &TuAnalysis,
    symbols: &'m BTreeMap<SymbolId, ReducedSymbolRecord>,
    by_name: &HashMap<&str, Vec<Candidate<'m>>>,
) -> Vec<(&'m SymbolId, &'m SymbolId)> {
    let mut seen: HashSet<(&SymbolId, &str)> = HashSet::new();
    let mut found = Vec::new();
    for site in &tu.call_sites {
        let name = unqualified(&site.callee_name);
        if !seen.insert((&site.caller_symbol, name)) {
            continue;
        }
        let Some((caller, _)) = symbols.get_key_value(&site.caller_symbol) else {
            continue;
        };
        let candidates = by_name.get(name).map(Vec::as_slice).unwrap_or_default();
        if let Some(targets) = visible_targets(candidates, caller, tu.tu_file_id) {
            found.extend(targets.into_iter().map(|callee| (caller, callee)));
        }
    }
    found
}

/// The candidates `unit` can see, self excluded; None once more than
/// `MAX_VISIBLE_CALL_TARGETS` match (the name is too ambiguous to resolve).
fn visible_targets<'m>(
    candidates: &[Candidate<'m>],
    caller: &SymbolId,
    unit: FileId,
) -> Option<Vec<&'m SymbolId>> {
    let mut targets = Vec::new();
    for &(callee, units) in candidates {
        if callee != caller && units.contains(&unit) {
            if targets.len() == MAX_VISIBLE_CALL_TARGETS {
                return None;
            }
            targets.push(callee);
        }
    }
    Some(targets)
}

/// The distinct pairs in id order: an address dedupe first (cheap, and
/// most duplicates are the same inline caller seen by many units), then
/// one value sort so the edge map can be bulk-built.
fn distinct_sorted<'m>(mut pairs: Vec<(&'m SymbolId, &'m SymbolId)>) -> Vec<(&'m SymbolId, &'m SymbolId)> {
    let address = |symbol: &SymbolId| symbol as *const SymbolId as usize;
    pairs.par_sort_unstable_by_key(|&(caller, callee)| (address(caller), address(callee)));
    pairs.dedup_by(|a, b| std::ptr::eq(a.0, b.0) && std::ptr::eq(a.1, b.1));
    pairs.par_sort_unstable();
    pairs
}

/// Add a caller -> callee edge for each call site whose callee name matches
/// a symbol visible in the calling unit. An existing edge keeps its kind
/// (a `Complete` use is never downgraded); new call edges are `Incomplete`
/// because a call needs only the callee's declaration. Units resolve in
/// parallel; the merge into the ordered edge map keeps the result
/// deterministic.
pub(crate) fn add_call_edges(
    index_result: &IndexResult,
    symbols: &BTreeMap<SymbolId, ReducedSymbolRecord>,
    tu_usage: &BTreeMap<SymbolId, BTreeSet<FileId>>,
    raw_edges: &mut BTreeMap<(SymbolId, SymbolId), DependencyKind>,
) {
    let by_name = candidates_by_unqualified_name(symbols, tu_usage);
    let pairs: Vec<(&SymbolId, &SymbolId)> = index_result
        .tu_analyses
        .par_iter()
        .flat_map_iter(|tu| unit_call_edges(tu, symbols, &by_name))
        .collect();
    let mut merged: BTreeMap<(SymbolId, SymbolId), DependencyKind> = distinct_sorted(pairs)
        .into_iter()
        .map(|(caller, callee)| ((caller.clone(), callee.clone()), DependencyKind::Incomplete))
        .collect();
    // `append` lets the existing entries overwrite: declared kinds win.
    merged.append(raw_edges);
    *raw_edges = merged;
}

#[cfg(test)]
#[path = "call_usage_tests.rs"]
mod tests;
