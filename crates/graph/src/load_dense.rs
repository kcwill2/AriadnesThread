//! Dense stage-5 passes: the integer index and the symbol pass over it.
//!
//! The per-symbol and per-file entries (`analyze_symbol_load_counts_with_
//! expansion`, `analyze_file_load_compact_by_id`) walk the semantic edges
//! over string-keyed hash sets: every reached symbol costs a string clone
//! and a string hash, and the capped boot memo cannot keep the walks. With
//! calls counted as uses a function reaches everything it transitively
//! calls, so both passes grew with (symbols or files) x reached symbols and
//! dominated cold loads.
//!
//! `DenseLoadIndex` does the same counting over integers. Symbols are
//! numbered once in ascending id order (so integer order IS id order), call
//! cycles collapse into strongly connected components (all members of a
//! cycle reach exactly the same symbols), and a walk visits components by
//! stamping an array: no hashing, no allocation per node.
//!
//! Symbol pass, one defining file's group at a time:
//!
//! 1. weigh each component by how many of its symbols the file's include
//!    closure brings in (the physical expansion), then
//! 2. walk the component graph once per distinct component in the group,
//!    summing the weights it reaches. That sum is the symbol's semantic load.
//!
//! The file pass lives in `load_dense_file.rs`. Both produce metrics equal
//! to the reference entries' (locked by `tests/dense_load_index_
//! equivalence_test.rs`).

use super::{LoadAnalyzer, LoadMemo, LoadMetric};
use crate::dense_graph::{condensation, strongly_connected_components, Adjacency};
use crate::{IncludeGraph, ReducedSemanticModel, ReducedSymbolRecord};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Marks "no source" in `DenseLoadIndex::source_of`.
pub(super) const NO_SOURCE: u32 = u32::MAX;

/// Idle scratch values shared by the worker threads: taking one reuses a
/// returned value when there is one, so each thread allocates its
/// index-sized arrays once, not once per call.
pub(super) struct Pool<T> {
    idle: Mutex<Vec<T>>,
}

impl<T> Pool<T> {
    fn new() -> Self {
        Self { idle: Mutex::new(Vec::new()) }
    }

    pub(super) fn take_or(&self, make: impl FnOnce() -> T) -> Pooled<'_, T> {
        let idle = self.idle.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).pop();
        Pooled { pool: self, value: Some(idle.unwrap_or_else(make)) }
    }
}

/// A pool value on loan; it returns to the pool when dropped.
pub(super) struct Pooled<'p, T> {
    pool: &'p Pool<T>,
    value: Option<T>,
}

impl<T> Deref for Pooled<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value.as_ref().expect("held until drop")
    }
}

impl<T> DerefMut for Pooled<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.as_mut().expect("held until drop")
    }
}

impl<T> Drop for Pooled<'_, T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.pool.idle.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).push(value);
        }
    }
}

/// Reusable per-walk state: which components the current walk has seen
/// (`stamps[component] == generation`) and its work stack.
pub(super) struct VisitScratch {
    pub(super) stamps: Vec<u32>,
    generation: u32,
    pub(super) stack: Vec<u32>,
}

impl VisitScratch {
    pub(super) fn new(component_count: usize) -> Self {
        Self {
            stamps: vec![0; component_count],
            generation: 0,
            stack: Vec::new(),
        }
    }

    /// Start a new walk: a fresh generation invalidates every old stamp
    /// without touching the array (refilled only on the rare wrap-around).
    pub(super) fn begin_walk(&mut self) -> u32 {
        if self.generation == u32::MAX {
            self.stamps.fill(0);
            self.generation = 0;
        }
        self.generation += 1;
        self.stack.clear();
        self.generation
    }
}

/// Integer view of one reduced model for the stage-5 passes. Build it once
/// with `LoadAnalyzer::dense_load_index`; every query takes `&self` and may
/// run from any number of threads.
pub struct DenseLoadIndex<'a> {
    pub(super) include_graph: &'a IncludeGraph,
    pub(super) model: &'a ReducedSemanticModel,
    memo: Arc<LoadMemo>,
    /// Symbol number -> id, ascending by id.
    pub(super) ids: Vec<&'a SymbolId>,
    index_of: HashMap<&'a SymbolId, u32>,
    pub(super) component_of: Vec<u32>,
    /// Component graph of the semantic edges (acyclic).
    pub(super) component_uses: Adjacency,
    /// `model.file_symbols` as symbol numbers.
    pub(super) file_symbols: HashMap<FileId, Vec<u32>>,
    /// Per symbol: the number of the file it is credited to
    /// (`LoadAnalyzer::attribution_file`), or `NO_SOURCE`.
    pub(super) source_of: Vec<u32>,
    /// Source number -> file.
    pub(super) source_files: Vec<FileId>,
    /// Idle all-zero per-component weight arrays (symbol pass).
    weight_buffers: Pool<Vec<u32>>,
    visit_buffers: Pool<VisitScratch>,
    pub(super) file_buffers: Pool<super::dense_file::FileScratch>,
}

/// Every symbol id the model mentions - record keys, both ends of every
/// edge, every file-listed symbol - distinct and ascending.
fn sorted_symbol_ids(model: &ReducedSemanticModel) -> Vec<&SymbolId> {
    let mut distinct: HashSet<&SymbolId> = model.symbols.keys().collect();
    for edge in &model.edges {
        distinct.insert(&edge.src_symbol);
        distinct.insert(&edge.dst_symbol);
    }
    distinct.extend(model.file_symbols.values().flatten());
    let mut ids: Vec<&SymbolId> = distinct.into_iter().collect();
    ids.par_sort_unstable();
    ids
}

/// Number the files symbols are credited to; returns each symbol's source
/// number (`NO_SOURCE` when it has no record or no creditable file) and the
/// number -> file table.
fn number_sources(analyzer: &LoadAnalyzer<'_>, ids: &[&SymbolId]) -> (Vec<u32>, Vec<FileId>) {
    let credited: Vec<Option<FileId>> = ids
        .par_iter()
        .map(|id| analyzer.model.symbols.get(*id).and_then(|record| analyzer.attribution_file(record)))
        .collect();
    let mut source_files: Vec<FileId> = credited.iter().flatten().copied().collect();
    source_files.sort_unstable();
    source_files.dedup();
    let source_of = credited
        .iter()
        .map(|file| match file {
            Some(file) => source_files.binary_search(file).map_or(NO_SOURCE, |number| number as u32),
            None => NO_SOURCE,
        })
        .collect();
    (source_of, source_files)
}

impl<'a> LoadAnalyzer<'a> {
    /// Build the dense stage-5 index over this analyzer's model.
    pub fn dense_load_index(&self) -> DenseLoadIndex<'a> {
        let model = self.model;
        let ids = sorted_symbol_ids(model);
        let index_of: HashMap<&SymbolId, u32> =
            ids.iter().enumerate().map(|(number, id)| (*id, number as u32)).collect();
        let pairs: Vec<(u32, u32)> = model
            .edges
            .par_iter()
            .map(|edge| (index_of[&edge.src_symbol], index_of[&edge.dst_symbol]))
            .collect();
        let uses = Adjacency::from_pairs(ids.len(), pairs);
        let components = strongly_connected_components(&uses);
        let component_uses = condensation(&uses, &components);
        let file_symbols = model
            .file_symbols
            .iter()
            .map(|(file, listed)| (*file, listed.iter().map(|id| index_of[id]).collect()))
            .collect();
        let (source_of, source_files) = number_sources(self, &ids);
        DenseLoadIndex {
            include_graph: self.include_graph,
            model,
            memo: self.memo(),
            ids,
            index_of,
            component_of: components.component_of,
            component_uses,
            file_symbols,
            source_of,
            source_files,
            weight_buffers: Pool::new(),
            visit_buffers: Pool::new(),
            file_buffers: Pool::new(),
        }
    }
}

impl<'a> DenseLoadIndex<'a> {
    /// Counts-only load metric of each of `symbols`, all defined in
    /// `def_file`: equal to `analyze_symbol_load_counts_with_expansion` over
    /// `physical_expansion(def_file)`, element for element. `None` marks an
    /// id that entry would refuse: no record, or a different defining file.
    pub fn def_file_group_counts(&self, def_file: FileId, symbols: &[&SymbolId]) -> Vec<Option<LoadMetric>> {
        let members: Vec<Option<(&ReducedSymbolRecord, u32)>> =
            symbols.iter().map(|id| self.group_member(def_file, id)).collect();
        let mut components: Vec<u32> = members.iter().flatten().map(|(_, component)| *component).collect();
        components.sort_unstable();
        components.dedup();
        if components.is_empty() {
            return members.iter().map(|_| None).collect();
        }

        let mut weights = self.weight_buffers.take_or(|| vec![0; self.component_uses.node_count()]);
        let include_load = self.add_physical_weights(def_file, &mut weights);
        let component_count = self.component_uses.node_count();
        let weighed: &[u32] = &weights;
        let semantic_loads: Vec<usize> = components
            .par_iter()
            .map_init(
                || self.visit_buffers.take_or(|| VisitScratch::new(component_count)),
                |scratch, &component| self.reachable_weight(component, weighed, include_load, scratch),
            )
            .collect();
        // The buffer goes back to the pool all-zero.
        self.clear_physical_weights(def_file, &mut weights);

        let target_file = self.include_graph.file_paths.get(&def_file);
        members
            .into_iter()
            .map(|member| {
                let (record, component) = member?;
                let position = components.binary_search(&component).ok()?;
                Some(counts_metric(record, target_file.cloned(), semantic_loads[position], include_load))
            })
            .collect()
    }

    /// The record and component of a group symbol, when the per-symbol
    /// entry would accept it for `def_file`.
    fn group_member(&self, def_file: FileId, id: &SymbolId) -> Option<(&'a ReducedSymbolRecord, u32)> {
        let record = self.model.symbols.get(id)?;
        if record.defining_file != Some(def_file) {
            return None;
        }
        let index = *self.index_of.get(id)?;
        Some((record, self.component_of[index as usize]))
    }

    /// The symbol number lists of `file` and of every file it includes: its
    /// physical expansion, possibly naming a symbol more than once.
    pub(super) fn physical_symbol_lists(&self, file: FileId) -> impl Iterator<Item = &Vec<u32>> {
        let included = self.memo.strict_closure_slice(self.include_graph, file);
        std::iter::once(file)
            .chain(included.iter().copied().filter(move |other| *other != file))
            .filter_map(|listed_file| self.file_symbols.get(&listed_file))
    }

    /// Add one to each component's weight per DISTINCT symbol of the file's
    /// physical expansion; returns the number of distinct symbols (the
    /// include load). A symbol listed under several files counts once.
    fn add_physical_weights(&self, def_file: FileId, weights: &mut [u32]) -> usize {
        let mut counted = vec![0u64; self.ids.len().div_ceil(64)];
        let mut include_load = 0;
        for listed in self.physical_symbol_lists(def_file) {
            for &symbol in listed {
                let (word, bit) = (symbol as usize / 64, 1u64 << (symbol % 64));
                if counted[word] & bit == 0 {
                    counted[word] |= bit;
                    weights[self.component_of[symbol as usize] as usize] += 1;
                    include_load += 1;
                }
            }
        }
        include_load
    }

    /// Zero exactly the weights `add_physical_weights` touched.
    fn clear_physical_weights(&self, def_file: FileId, weights: &mut [u32]) {
        for listed in self.physical_symbol_lists(def_file) {
            for &symbol in listed {
                weights[self.component_of[symbol as usize] as usize] = 0;
            }
        }
    }

    /// Sum of `weights` over every component reachable from `start`
    /// (itself included). Stops early once the sum reaches `full_weight`,
    /// the total of all weights: nothing further can add to it.
    fn reachable_weight(&self, start: u32, weights: &[u32], full_weight: usize, scratch: &mut VisitScratch) -> usize {
        let generation = scratch.begin_walk();
        scratch.stamps[start as usize] = generation;
        scratch.stack.push(start);
        let mut reached = 0usize;
        while let Some(component) = scratch.stack.pop() {
            reached += weights[component as usize] as usize;
            if reached == full_weight {
                break;
            }
            for &next in self.component_uses.successors(component) {
                if scratch.stamps[next as usize] != generation {
                    scratch.stamps[next as usize] = generation;
                    scratch.stack.push(next);
                }
            }
        }
        reached
    }
}

/// The counts-only metric shape the persisted symbol rows keep.
fn counts_metric(
    record: &ReducedSymbolRecord,
    target_file: Option<PathBuf>,
    semantic_load: usize,
    include_load: usize,
) -> LoadMetric {
    LoadMetric {
        target_name: record.name.clone(),
        target_file,
        semantic_load,
        include_load,
        collateral_load: include_load - semantic_load,
        total_load: include_load,
        semantic_symbols: Vec::new(),
        collateral_symbols: Vec::new(),
        collateral_sources: Vec::new(),
        largest_collateral_source: None,
        semantic_sources: Vec::new(),
    }
}
