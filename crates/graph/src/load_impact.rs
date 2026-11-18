//! Extraction-impact entries of the load analyzer, split from `load.rs` to
//! keep it under the R2 line limit. A child module of `load`, so it reaches
//! the analyzer's private fields and helpers (`memo`, `override_closure`,
//! `impact_header`) directly.
//!
//! The summary entry and the required-header visitor (2026-11-18 boost
//! justification fix) serve the bounded UI payload, tens of thousands of
//! calls per boot on 24 rayon threads. Each fresh-and-dropped set of
//! thousands of file ids is a large heap block, and on Windows those
//! serialize on the process heap's lock: measured 67 s of a 24-thread run
//! spent ~80% blocked. Both entries therefore work in per-thread scratch
//! sets that keep their capacity between calls (bounded by the file count,
//! a few hundred KB per set per thread), over borrowed closure slices.

use super::{ExtractionImpact, ImpactHeader, LoadAnalyzer};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::Path;

/// Reusable per-thread sets for the summary and visitor entries. Every user
/// clears what it fills, so no state leaks from one call into the next.
#[derive(Default)]
struct ImpactScratch {
    seed_files: HashSet<FileId>,
    required: HashSet<FileId>,
    reach: HashSet<FileId>,
    dropped_directly: HashSet<FileId>,
    stack: Vec<FileId>,
}

thread_local! {
    static IMPACT_SCRATCH: RefCell<ImpactScratch> = RefCell::new(ImpactScratch::default());
}

/// Runs `work` on this thread's scratch; a re-entrant call (a visitor that
/// calls back in) gets fresh sets instead of a borrow panic.
fn with_impact_scratch<R>(work: impl FnOnce(&mut ImpactScratch) -> R) -> R {
    IMPACT_SCRATCH.with(|cell| match cell.try_borrow_mut() {
        Ok(mut scratch) => work(&mut scratch),
        Err(_) => work(&mut ImpactScratch::default()),
    })
}

/// Keeps `kept` as the sorted, distinct `limit` smallest items seen so far.
fn insert_bounded<T: Ord>(kept: &mut Vec<T>, item: T, limit: usize) {
    if let Err(position) = kept.binary_search(&item) {
        if position < limit {
            kept.insert(position, item);
            kept.truncate(limit);
        }
    }
}

/// Bounded twin of `ExtractionImpact`: the same counts, with only the first
/// `sample_max` rows of the `(path asc, file_id asc)`-sorted
/// `dropped_from_root` list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractionImpactSummary {
    pub dropped_from_root_count: usize,
    pub still_reachable_count: usize,
    pub dropped_sample: Vec<ImpactHeader>,
    pub root_closure_before: usize,
    pub root_closure_after: usize,
    pub symbols_no_longer_parsed: usize,
}

/// The id sets behind one impact preview (see `ExtractionImpact` for the
/// definition of each set).
struct ImpactSets {
    required: HashSet<FileId>,
    dropped_directly: HashSet<FileId>,
    dropped_from_root: HashSet<FileId>,
    still_reachable: HashSet<FileId>,
    root_closure_before: usize,
    root_closure_after: usize,
}

impl<'a> LoadAnalyzer<'a> {
    /// Per-consumer extraction preview (E-I1..E-I3): given a consumer edge
    /// root F -> source S and the symbols justified on that edge, compute what
    /// the slim edge sheds (`dropped_directly`), what F actually stops seeing
    /// (`dropped_from_root`, via the modeled `closure_after`), and the decoy
    /// headers that keep arriving via sibling paths (`still_reachable`).
    /// Returns `None` for `root == source`, an empty justified set, or file
    /// ids unregistered in `file_paths`.
    pub fn extraction_impact(
        &self,
        root: FileId,
        source: FileId,
        justified_symbols: &[SymbolId],
    ) -> Option<ExtractionImpact> {
        let sets = self.impact_sets(root, source, justified_symbols)?;
        let dropped_from_root = self.impact_headers(&sets.dropped_from_root);
        Some(ExtractionImpact {
            root_file_id: root,
            source_file_id: source,
            required_headers: self.impact_headers(&sets.required),
            dropped_directly: self.impact_headers(&sets.dropped_directly),
            symbols_no_longer_parsed: dropped_from_root.iter().map(|h| h.symbol_count).sum(),
            dropped_from_root,
            still_reachable: self.impact_headers(&sets.still_reachable),
            root_closure_before: sets.root_closure_before,
            root_closure_after: sets.root_closure_after,
        })
    }

    /// Visits each id of `required_header_files_for_symbols(symbol_ids)`
    /// once, in no particular order, without allocating a result set.
    pub fn for_each_required_header_file(
        &self,
        symbol_ids: &[SymbolId],
        mut visit: impl FnMut(FileId),
    ) {
        with_impact_scratch(|scratch| {
            self.required_header_files_into(symbol_ids, &mut scratch.seed_files, &mut scratch.required);
            scratch.required.iter().for_each(|&file_id| visit(file_id));
        });
    }

    /// Summary form of `extraction_impact` with identical `None` guards and
    /// identical counts; `dropped_sample` equals the full entry's
    /// `dropped_from_root` cut to `sample_max` rows.
    pub fn extraction_impact_summary(
        &self,
        root: FileId,
        source: FileId,
        justified_symbols: &[SymbolId],
        sample_max: usize,
    ) -> Option<ExtractionImpactSummary> {
        if !self.impact_guards_pass(root, source, justified_symbols) {
            return None;
        }
        Some(with_impact_scratch(|scratch| {
            self.summary_in_scratch(root, source, justified_symbols, sample_max, scratch)
        }))
    }

    /// The summary over reused scratch: `reach` holds first the kept subtree
    /// of S, then closure_after(F). dropped_from_root is never materialized;
    /// it is streamed from the borrowed sorted closure of F.
    fn summary_in_scratch(
        &self,
        root: FileId,
        source: FileId,
        justified_symbols: &[SymbolId],
        sample_max: usize,
        scratch: &mut ImpactScratch,
    ) -> ExtractionImpactSummary {
        let ImpactScratch { seed_files, required, reach, dropped_directly, stack } = scratch;
        self.required_header_files_into(justified_symbols, seed_files, required);
        required.remove(&source);
        self.override_closure_into(source, source, required, reach, stack);
        let source_closure = self.memo.strict_closure_slice(self.include_graph, source);
        dropped_directly.clear();
        dropped_directly.extend(source_closure.iter().filter(|id| !reach.contains(id)));

        self.override_closure_into(root, source, required, reach, stack);
        let root_closure = self.memo.strict_closure_slice(self.include_graph, root);
        let unknown = Path::new("unknown");
        let mut sample_keys: Vec<(&Path, FileId)> = Vec::new();
        let (mut dropped_from_root_count, mut symbols_no_longer_parsed) = (0usize, 0usize);
        for &id in root_closure.iter().filter(|id| !reach.contains(id)) {
            dropped_from_root_count += 1;
            symbols_no_longer_parsed += self.model.file_symbols.get(&id).map_or(0, |s| s.len());
            let path = self.include_graph.file_paths.get(&id).map_or(unknown, |p| p.as_path());
            insert_bounded(&mut sample_keys, (path, id), sample_max);
        }
        // still_reachable = dropped_directly \ dropped_from_root, where an id
        // is in dropped_from_root iff it is in closure(F) but not reached.
        let in_dropped_from_root =
            |id: &FileId| root_closure.binary_search(id).is_ok() && !reach.contains(id);
        let still_reachable_count =
            dropped_directly.iter().filter(|id| !in_dropped_from_root(id)).count();
        ExtractionImpactSummary {
            dropped_from_root_count,
            still_reachable_count,
            dropped_sample: sample_keys.into_iter().map(|(_, id)| self.impact_header(id)).collect(),
            root_closure_before: root_closure.len(),
            root_closure_after: reach.len().saturating_sub(1),
            symbols_no_longer_parsed,
        }
    }

    /// The shared `None` guards: self-edge, empty justified set, or an id
    /// unregistered in `file_paths`.
    fn impact_guards_pass(&self, root: FileId, source: FileId, justified: &[SymbolId]) -> bool {
        let paths = &self.include_graph.file_paths;
        root != source
            && !justified.is_empty()
            && paths.contains_key(&root)
            && paths.contains_key(&source)
    }

    /// Shared set computation of both impact entries; `None` on the guards.
    fn impact_sets(
        &self,
        root: FileId,
        source: FileId,
        justified_symbols: &[SymbolId],
    ) -> Option<ImpactSets> {
        if !self.impact_guards_pass(root, source, justified_symbols) {
            return None;
        }
        // R never contains S (contract: "excluding S itself"); the A2 rule
        // drops seed defining files, but justified symbols need not all be
        // defined in S, so filter defensively. Only the id set is consumed,
        // so the file-level entry serves it without building symbol
        // closures (2026-11-18 boost fix).
        let required: HashSet<FileId> = self
            .required_header_files_for_symbols(justified_symbols)
            .into_iter()
            .filter(|id| *id != source)
            .collect();

        let kept_subtree = self.override_closure(source, source, &required);
        let dropped_directly: HashSet<FileId> = self
            .memo
            .strict_closure(self.include_graph, source)
            .difference(&kept_subtree)
            .copied()
            .collect();

        let root_after = self.override_closure(root, source, &required);
        let root_before = self.memo.strict_closure(self.include_graph, root);
        let dropped_from_root: HashSet<FileId> =
            root_before.difference(&root_after).copied().collect();
        let still_reachable: HashSet<FileId> = dropped_directly
            .difference(&dropped_from_root)
            .copied()
            .collect();
        debug_assert!(
            dropped_from_root.is_disjoint(&still_reachable)
                && dropped_from_root.is_subset(&dropped_directly),
            "E-I1: dropped_directly must partition into dropped_from_root + still_reachable"
        );
        Some(ImpactSets {
            required,
            dropped_directly,
            dropped_from_root,
            still_reachable,
            root_closure_before: root_before.len(),
            root_closure_after: root_after.len().saturating_sub(1),
        })
    }
}
