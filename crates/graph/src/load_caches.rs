//! Shared memoization caches of the load analyzer, extracted verbatim from
//! `load.rs` by plan 1790608181551 (task B1): the caches moved out of the
//! per-query `LoadAnalyzer` handle so one warmed `Arc<LoadMemo>` can be
//! shared across many handles (UI requests, rayon workers) instead of being
//! cold-started per construction.
//!
//! Thread-safe (`Send + Sync`) by construction (I4): `closures`,
//! `symbol_adjacency` and `file_adjacency` are `OnceLock`s — after
//! initialization every access is a plain shared reference with NO lock held,
//! so BFS traversals never serialize on an adjacency mutex;
//! `semantic_closure_cache` keeps a short `Mutex` only around map
//! lookup/insert, and cache hits hand out an `Arc` refcount bump instead of
//! cloning the whole closure set. The `Mutex` poisoning via `.unwrap()` is
//! acceptable because poisoning requires a prior panic while holding that
//! same lock, which already aborts the run. No lock is ever held while
//! acquiring another (`OnceLock::get_or_init` for `symbol_adjacency` is
//! invoked OUTSIDE any other guard, and the `semantic_closure_cache` guard
//! ends before the miss-path BFS starts), so the caches cannot deadlock, and
//! no guard crosses a rayon job boundary.
//!
//! The immutable inputs (`IncludeGraph`, `ReducedSemanticModel`) stay outside
//! the memo and are passed per call: the memo NEVER stores or clones them, so
//! shared entries stay valid for exactly as long as their graph generation
//! does (memo lifetime = analysis generation, I5).

use crate::{IncludeClosure, IncludeGraph, ReducedSemanticModel};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

// Only the test-only counters touch atomics; the import stays gated so
// release builds emit no unused-import warning. `feature = "test-instrument"`
// exposes the same counters to downstream integration tests (ui C1 lock).
#[cfg(any(test, feature = "test-instrument"))]
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct LoadMemo {
    closures: OnceLock<HashMap<FileId, IncludeClosure>>,
    symbol_adjacency: OnceLock<HashMap<SymbolId, Vec<SymbolId>>>,
    /// Include adjacency memo for `override_closure` (plan LoadPerf A1):
    /// built at most once from `IncludeGraph::adjacency()` — the ONE build
    /// call site — after which every BFS reads a plain `&`, no guard, no
    /// rebuild. The modeled after-state overrides stay per-call.
    file_adjacency: OnceLock<HashMap<FileId, Vec<FileId>>>,
    /// Append-only memo of single-seed semantic closures, keyed by seed.
    /// Neither the memo's owner nor any shared handle ever mutates `model`
    /// or `include_graph` after construction, so a closure computed once can
    /// never go stale: entries are permanent and NO invalidation logic is
    /// required. Memory grows by one `Arc<HashSet<SymbolId>>` per distinct
    /// seed ever queried; the Arc makes concurrent cache hits a refcount bump
    /// instead of a full `HashSet` clone.
    semantic_closure_cache: Mutex<HashMap<SymbolId, Arc<HashSet<SymbolId>>>>,
    /// Append-only memo of the physical symbol expansion of a SINGLE
    /// def-file — the union of `model.file_symbols` over the file's
    /// transitive includes (seed included). Like the semantic memo, the
    /// inputs (include closures, reduced model) are immutable after
    /// construction, so entries are permanent and need no invalidation;
    /// `compute_load`'s `def_files.len() == 1` fast path borrows the `Arc`
    /// instead of re-cloning thousands of `SymbolId`s per call. Multi-file
    /// def-file sets (none in practice) use the non-memoized path. Memory
    /// grows by one `Arc<HashSet<SymbolId>>` per distinct def-file ever
    /// queried, mirroring the semantic memo's per-seed growth.
    physical_expansion_cache: Mutex<HashMap<FileId, Arc<HashSet<SymbolId>>>>,
    /// Test-only tally of full include-closure table computations
    /// (`IncludeGraph::compute_closures`) executed by this memo; the OnceLock
    /// caps it at one per memo. The ui C1 route lock (plan 1790608181551)
    /// asserts the SECOND request adds zero. Gated identically to the other
    /// counters so it exists under `feature = "test-instrument"` for
    /// downstream tests.
    #[cfg(any(test, feature = "test-instrument"))]
    closure_computations: AtomicUsize,
    /// Test-only tally of real per-def-file physical expansions (transitive
    /// include closure unioned into `model.file_symbols`) executed by this
    /// memo; the memo under test must serve repeats without a new expansion.
    /// Absent from release builds entirely, mirroring
    /// `semantic_bfs_traversals`.
    /// Atomic (not `Cell`) because the struct must be `Sync` so one memo can
    /// be shared across rayon workers.
    #[cfg(any(test, feature = "test-instrument"))]
    physical_expansions: AtomicUsize,
    /// Test-only tally of actual semantic-closure BFS traversals executed by
    /// this memo. Instrumented per memo (not a global counter) so parallel
    /// test threads cannot observe each other's traversals; absent from
    /// release builds entirely. Atomic (not `Cell`) because the struct must
    /// be `Sync` so one memo can be shared across rayon workers.
    #[cfg(any(test, feature = "test-instrument"))]
    semantic_bfs_traversals: AtomicUsize,
    /// Test-only tally of include-adjacency map builds executed by this
    /// memo; the OnceLock memo (plan LoadPerf A1) must cap it at one.
    #[cfg(any(test, feature = "test-instrument"))]
    include_adjacency_builds: AtomicUsize,
}

impl LoadMemo {
    /// Fully cold memo — the state every `LoadAnalyzer::new` construction
    /// gets (unchanged pre-B1 behavior).
    pub fn new() -> Self {
        Self {
            closures: OnceLock::new(),
            symbol_adjacency: OnceLock::new(),
            file_adjacency: OnceLock::new(),
            semantic_closure_cache: Mutex::new(HashMap::new()),
            physical_expansion_cache: Mutex::new(HashMap::new()),
            #[cfg(any(test, feature = "test-instrument"))]
            closure_computations: AtomicUsize::new(0),
            #[cfg(any(test, feature = "test-instrument"))]
            physical_expansions: AtomicUsize::new(0),
            #[cfg(any(test, feature = "test-instrument"))]
            semantic_bfs_traversals: AtomicUsize::new(0),
            #[cfg(any(test, feature = "test-instrument"))]
            include_adjacency_builds: AtomicUsize::new(0),
        }
    }

    /// Reuse include closures already computed elsewhere (e.g. by
    /// `build_include_graph`) instead of recomputing them lazily on the
    /// first query.
    pub fn with_closures(closures: HashMap<FileId, IncludeClosure>) -> Self {
        Self {
            closures: OnceLock::from(closures),
            symbol_adjacency: OnceLock::new(),
            file_adjacency: OnceLock::new(),
            semantic_closure_cache: Mutex::new(HashMap::new()),
            physical_expansion_cache: Mutex::new(HashMap::new()),
            #[cfg(any(test, feature = "test-instrument"))]
            closure_computations: AtomicUsize::new(0),
            #[cfg(any(test, feature = "test-instrument"))]
            physical_expansions: AtomicUsize::new(0),
            #[cfg(any(test, feature = "test-instrument"))]
            semantic_bfs_traversals: AtomicUsize::new(0),
            #[cfg(any(test, feature = "test-instrument"))]
            include_adjacency_builds: AtomicUsize::new(0),
        }
    }

    /// Shared include-closure table, built at most once via `get_or_init`
    /// and thereafter handed out as a plain shared reference. The single
    /// `get_or_init` expression previously inlined at both call sites
    /// (`get_transitive_includes`, `strict_closure`) — behavior unchanged.
    fn closures_map(&self, include_graph: &IncludeGraph) -> &HashMap<FileId, IncludeClosure> {
        self.closures.get_or_init(|| {
            #[cfg(any(test, feature = "test-instrument"))]
            self.closure_computations.fetch_add(1, Ordering::Relaxed);
            include_graph.compute_closures()
        })
    }

    pub fn get_transitive_includes(
        &self,
        include_graph: &IncludeGraph,
        file_id: FileId,
    ) -> HashSet<FileId> {
        let closures = self.closures_map(include_graph);
        let mut all = HashSet::new();
        all.insert(file_id);
        if let Some(closure) = closures.get(&file_id) {
            all.extend(closure.transitive_includes.iter().copied());
        }
        all
    }

    /// Semantic closure of `start_symbols`, served per seed from
    /// `semantic_closure_cache`. Reachability is seed-local, so
    /// `closure({a, b}) == closure({a}) ∪ closure({b})`: the union of the
    /// per-seed results equals the old single multi-seed BFS exactly, seeds
    /// included (an unknown/edgeless seed yields the singleton `{itself}`).
    /// The single-seed case copies the memoized set once; multi-seed calls
    /// union the Arc contents into the owned result.
    pub fn get_semantic_closure_of_symbols(
        &self,
        model: &ReducedSemanticModel,
        start_symbols: &[SymbolId],
    ) -> HashSet<SymbolId> {
        if let [only_seed] = start_symbols {
            return (*self.cached_single_seed_closure(model, only_seed)).clone();
        }
        let mut visited = HashSet::new();
        for seed in start_symbols {
            visited.extend(self.cached_single_seed_closure(model, seed).iter().cloned());
        }
        visited
    }

    /// Single-seed semantic closure, memoized in `semantic_closure_cache`:
    /// the cache `lock()` guard ends at the end of the lookup statement, so
    /// the miss-path insert lock never overlaps another lock. Hits clone
    /// only the `Arc` (refcount bump), never the closure set itself.
    pub fn cached_single_seed_closure(
        &self,
        model: &ReducedSemanticModel,
        seed: &SymbolId,
    ) -> Arc<HashSet<SymbolId>> {
        let memoized = self
            .semantic_closure_cache
            .lock()
            .unwrap()
            .get(seed)
            .cloned();
        if let Some(closure) = memoized {
            return closure;
        }
        let closure = Arc::new(self.compute_single_seed_closure(model, seed));
        self.semantic_closure_cache
            .lock()
            .unwrap()
            .insert(seed.clone(), Arc::clone(&closure));
        closure
    }

    /// One BFS traversal from a single seed over the lazily built
    /// `symbol_adjacency`. Seeds are always inserted into `visited`, so an
    /// unknown symbol yields exactly `{seed}` — the behavior the cache
    /// memoizes bit-for-bit. `OnceLock::get_or_init` builds the map at most
    /// once and afterwards hands out a plain `&` — no lock is held during
    /// the traversal, so concurrent cache-miss BFS runs in parallel.
    fn compute_single_seed_closure(
        &self,
        model: &ReducedSemanticModel,
        seed: &SymbolId,
    ) -> HashSet<SymbolId> {
        #[cfg(any(test, feature = "test-instrument"))]
        self.record_semantic_bfs_traversal();

        let adjacency = self.symbol_adjacency.get_or_init(|| {
            let mut adj: HashMap<SymbolId, Vec<SymbolId>> = HashMap::new();
            for edge in &model.edges {
                adj.entry(edge.src_symbol.clone())
                    .or_default()
                    .push(edge.dst_symbol.clone());
            }
            adj
        });

        let mut visited = HashSet::new();
        visited.insert(seed.clone());
        let mut stack = vec![seed.clone()];

        while let Some(current) = stack.pop() {
            if let Some(nexts) = adjacency.get(&current) {
                for next in nexts {
                    if visited.insert(next.clone()) {
                        stack.push(next.clone());
                    }
                }
            }
        }
        visited
    }

    /// Test-only: counts each semantic-closure BFS traversal actually run by
    /// this memo. Gated behind `cfg(test)` / `test-instrument` so release
    /// builds pay nothing.
    #[cfg(any(test, feature = "test-instrument"))]
    fn record_semantic_bfs_traversal(&self) {
        self.semantic_bfs_traversals.fetch_add(1, Ordering::Relaxed);
    }

    /// Test-only: tally of include-closure table computations run so far;
    /// the OnceLock `closures` memo caps this at one per memo (zero when
    /// seeded via `with_closures`).
    #[cfg(any(test, feature = "test-instrument"))]
    pub fn closure_computation_count(&self) -> usize {
        self.closure_computations.load(Ordering::Relaxed)
    }

    /// Test-only: tally of semantic-closure BFS traversals run so far.
    #[cfg(any(test, feature = "test-instrument"))]
    pub fn semantic_bfs_traversal_count(&self) -> usize {
        self.semantic_bfs_traversals.load(Ordering::Relaxed)
    }

    /// Test-only: counts each real physical expansion executed by this memo
    /// (a per-def-file cache miss, or a multi-def-file expansion that is
    /// never memoized). Gated behind `cfg(test)`.
    #[cfg(any(test, feature = "test-instrument"))]
    fn record_physical_expansion(&self) {
        self.physical_expansions.fetch_add(1, Ordering::Relaxed);
    }

    /// Test-only: tally of real physical expansions run so far.
    #[cfg(any(test, feature = "test-instrument"))]
    pub fn physical_expansion_count(&self) -> usize {
        self.physical_expansions.load(Ordering::Relaxed)
    }

    /// Test-only: counts each include-adjacency map build run by this memo.
    /// Gated behind `cfg(test)`.
    #[cfg(any(test, feature = "test-instrument"))]
    fn record_include_adjacency_build(&self) {
        self.include_adjacency_builds.fetch_add(1, Ordering::Relaxed);
    }

    /// Test-only: tally of include-adjacency builds run so far; the
    /// OnceLock memo contract caps this at one per memo.
    #[cfg(any(test, feature = "test-instrument"))]
    pub fn include_adjacency_build_count(&self) -> usize {
        self.include_adjacency_builds.load(Ordering::Relaxed)
    }

    /// Test-only: number of memoized single-seed semantic closures.
    #[cfg(any(test, feature = "test-instrument"))]
    pub fn semantic_closure_cache_len(&self) -> usize {
        self.semantic_closure_cache.lock().unwrap().len()
    }

    /// Test-only: number of memoized single-def-file physical expansions.
    #[cfg(any(test, feature = "test-instrument"))]
    pub fn physical_expansion_cache_len(&self) -> usize {
        self.physical_expansion_cache.lock().unwrap().len()
    }

    /// Strict include closure of a file: transitive includes WITHOUT the file
    /// itself, served from the shared `closures` cache (real edges only).
    /// The OnceLock guard is never held across derived work — the returned
    /// `&HashMap` is a plain shared borrow.
    pub fn strict_closure(
        &self,
        include_graph: &IncludeGraph,
        file_id: FileId,
    ) -> HashSet<FileId> {
        self.closures_map(include_graph)
            .get(&file_id)
            .map(|c| c.transitive_includes.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Include adjacency map from the `file_adjacency` memo. `get_or_init`
    /// runs with NO other guard held and at most once per memo; after
    /// init this hands out a plain shared reference, so `override_closure`
    /// BFSes never serialize and never rebuild (plan LoadPerf A1, I4).
    pub fn file_adjacency_map(
        &self,
        include_graph: &IncludeGraph,
    ) -> &HashMap<FileId, Vec<FileId>> {
        self.file_adjacency.get_or_init(|| {
            #[cfg(any(test, feature = "test-instrument"))]
            self.record_include_adjacency_build();
            include_graph.adjacency()
        })
    }

    /// Physical expansion of a def-file set: the union of
    /// `model.file_symbols` over each def-file's transitive includes (the
    /// seed files included), exactly as `compute_load` used to build it
    /// inline. Each call is one REAL expansion and counts on the test-only
    /// `physical_expansions` tally; the single-file memo below decides how
    /// often this runs.
    pub fn expand_physical_symbols(
        &self,
        include_graph: &IncludeGraph,
        model: &ReducedSemanticModel,
        def_files: &HashSet<FileId>,
    ) -> HashSet<SymbolId> {
        #[cfg(any(test, feature = "test-instrument"))]
        self.record_physical_expansion();

        let mut physical_include_files = HashSet::new();
        for f in def_files {
            physical_include_files.extend(self.get_transitive_includes(include_graph, *f));
        }

        let mut physical_symbols = HashSet::new();
        for f in &physical_include_files {
            if let Some(syms) = model.file_symbols.get(f) {
                physical_symbols.extend(syms.iter().cloned());
            }
        }
        physical_symbols
    }

    /// Single-def-file physical expansion, memoized in
    /// `physical_expansion_cache`: the `lock()` guard ends at the end of the
    /// lookup statement, and the miss-path expansion (which touches the
    /// `closures` OnceLock) runs outside every guard — the same append-only
    /// pattern as `cached_single_seed_closure`, so cache hits are an `Arc`
    /// refcount bump and the caches cannot deadlock.
    pub fn cached_physical_expansion(
        &self,
        include_graph: &IncludeGraph,
        model: &ReducedSemanticModel,
        file_id: FileId,
    ) -> Arc<HashSet<SymbolId>> {
        let memoized = self
            .physical_expansion_cache
            .lock()
            .unwrap()
            .get(&file_id)
            .cloned();
        if let Some(expansion) = memoized {
            return expansion;
        }
        let expansion = Arc::new(self.expand_physical_symbols(
            include_graph,
            model,
            &HashSet::from([file_id]),
        ));
        self.physical_expansion_cache
            .lock()
            .unwrap()
            .insert(file_id, Arc::clone(&expansion));
        expansion
    }
}

impl Default for LoadMemo {
    fn default() -> Self {
        Self::new()
    }
}
