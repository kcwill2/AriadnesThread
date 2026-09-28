use crate::{IncludeClosure, IncludeGraph, ReducedSemanticModel};
use anyhow::{anyhow, Result};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

// Only the test-only BFS counter touches atomics; the import stays
// `cfg(test)` so release builds emit no unused-import warning.
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

/// Renders a path's final component, falling back to the full path for names
/// without one (e.g. `..`), mirroring `format_summary`'s file_name fallback.
fn path_basename(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

/// E-I9 (plan 1790521470000): needs-sets carry includable headers only —
/// `.h`/`.hpp`/`.hxx`, case-sensitive, mirroring the ui node-kind detection
/// in `dto.rs`. `.ipp`/`.inl` template-impl files are deliberately OUT for
/// now: whether they belong in the includable set is a future question, not
/// today's. Paths without a final component (e.g. `..`, empty) are never
/// includable.
pub fn is_includable_header(path: &Path) -> bool {
    match path.file_name() {
        Some(name) => {
            let name = name.to_string_lossy();
            name.ends_with(".h") || name.ends_with(".hpp") || name.ends_with(".hxx")
        }
        None => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributingSource {
    pub file_id: FileId,
    pub path: PathBuf,
    pub symbol_count: usize,
    pub symbols: Vec<SymbolId>,
}

/// Collateral sources are contributing sources on the unused side of the
/// include-closure split; the alias keeps existing consumers unchanged.
pub type CollateralSource = ContributingSource;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadMetric {
    pub target_name: String,
    pub target_file: Option<PathBuf>,
    pub semantic_load: usize,
    pub include_load: usize,
    pub collateral_load: usize,
    pub total_load: usize,
    pub semantic_symbols: Vec<SymbolId>,
    pub collateral_symbols: Vec<SymbolId>,
    pub collateral_sources: Vec<CollateralSource>,
    pub largest_collateral_source: Option<CollateralSource>,
    #[serde(default)]
    pub semantic_sources: Vec<ContributingSource>,
}

impl LoadMetric {
    pub fn format_summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.target_name);
        out.push_str("\n\nSemantic load:\n");
        out.push_str(&format!("    {} symbols\n\n", self.semantic_load));

        out.push_str("Include load:\n");
        out.push_str(&format!("    {} symbols\n\n", self.include_load));

        out.push_str("Collateral load:\n");
        out.push_str(&format!("    {} symbols\n\n", self.collateral_load));

        out.push_str("Largest collateral source:\n");
        if let Some(ref lcs) = self.largest_collateral_source {
            if let Some(name) = lcs.path.file_name().and_then(|s| s.to_str()) {
                out.push_str(&format!("    {}\n", name));
            } else {
                out.push_str(&format!("    {}\n", lcs.path.display()));
            }
            out.push_str(&format!("    {} symbols\n", lcs.symbol_count));
        } else {
            out.push_str("    None\n    0 symbols\n");
        }

        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadExplanation {
    pub target_name: String,
    pub target_file: PathBuf,
    pub path: Vec<String>,
    pub collateral_source: String,
    pub collateral_symbols_count: usize,
    pub reason: String,
}

impl LoadExplanation {
    pub fn format_explanation(&self) -> String {
        let mut out = String::new();
        let path_len = self.path.len();
        for (i, node) in self.path.iter().enumerate() {
            out.push_str(node);
            out.push('\n');
            if i < path_len - 1 {
                out.push_str("  ↓\n");
            }
        }

        out.push_str("\nReason:\n");
        let indented_reason = self
            .reason
            .split('\n')
            .map(|line| format!("  {}", line))
            .collect::<Vec<_>>()
            .join("\n");
        out.push_str(&indented_reason);
        out.push('\n');

        out
    }
}

/// Thread-safe (`Send + Sync`) so the pipeline can share ONE memoized
/// analyzer across rayon workers (plan LoadPerfImprovements Phase 4).
/// Contention-free reads (plan LoadPerfImprovements Phase 5): `closures`
/// and `symbol_adjacency` are `OnceLock`s — after initialization every
/// access is a plain shared reference with NO lock held, so BFS traversals
/// never serialize on an adjacency mutex; `semantic_closure_cache` keeps a
/// short `Mutex` only around map lookup/insert, and cache hits hand out an
/// `Arc` refcount bump instead of cloning the whole closure set. The
/// `Mutex` poisoning via `.unwrap()` is acceptable because poisoning
/// requires a prior panic while holding that same lock, which already
/// aborts the run. No lock is ever held while acquiring another
/// (`OnceLock::get_or_init` for `symbol_adjacency` is invoked OUTSIDE any
/// other guard, and the `semantic_closure_cache` guard ends before the
/// miss-path BFS starts), so the caches cannot deadlock, and no guard
/// crosses a rayon job boundary.
pub struct LoadAnalyzer<'a> {
    pub include_graph: &'a IncludeGraph,
    pub model: &'a ReducedSemanticModel,
    closures: OnceLock<HashMap<FileId, IncludeClosure>>,
    symbol_adjacency: OnceLock<HashMap<SymbolId, Vec<SymbolId>>>,
    /// Append-only memo of single-seed semantic closures, keyed by seed.
    /// `LoadAnalyzer` never mutates `model` or `include_graph` after
    /// construction, so a closure computed once can never go stale: entries
    /// are permanent and NO invalidation logic is required. Memory grows by
    /// one `Arc<HashSet<SymbolId>>` per distinct seed ever queried; the Arc
    /// makes concurrent cache hits a refcount bump instead of a full
    /// `HashSet` clone.
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
    /// Test-only tally of real per-def-file physical expansions (transitive
    /// include closure unioned into `model.file_symbols`) executed by this
    /// analyzer; the memo under test must serve repeats without a new
    /// expansion. Absent from release builds entirely, mirroring
    /// `semantic_bfs_traversals`.
    #[cfg(test)]
    physical_expansions: AtomicUsize,
    /// Test-only tally of actual semantic-closure BFS traversals executed by
    /// this analyzer. Instrumented per analyzer (not a global counter) so
    /// parallel test threads cannot observe each other's traversals; absent
    /// from release builds entirely. Atomic (not `Cell`) because the struct
    /// must be `Sync` so the pipeline can share one analyzer across rayon
    /// workers.
    #[cfg(test)]
    semantic_bfs_traversals: AtomicUsize,
}

/// One header row in an `ExtractionImpact` list. `symbol_count` is the total
/// number of symbols the reduced model attributes to the file (0 when the
/// file has none registered).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactHeader {
    pub file_id: FileId,
    pub path: PathBuf,
    pub symbol_count: usize,
}

/// Per-consumer extraction preview for one edge (root F × source S × justified
/// symbols). All header sets use the STRICT closure convention —
/// `closure(X)` = transitive includes of X, NOT containing X itself (matching
/// `IncludeClosure::transitive_includes`; `get_transitive_includes` unions the
/// seed and is NOT used for these sets). `closure_after(F)` excludes F too.
/// Every list is sorted by `(path asc, file_id asc)` for determinism.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionImpact {
    pub root_file_id: FileId,
    pub source_file_id: FileId,
    /// R: required headers of the justified symbols (header-only per E-I9,
    /// never containing S itself).
    pub required_headers: Vec<ImpactHeader>,
    /// `closure(S) \ closure({S} ∪ R)` — the subtree S's slim edge sheds.
    pub dropped_directly: Vec<ImpactHeader>,
    /// `closure(F) \ closure_after(F)` — what F actually stops seeing.
    pub dropped_from_root: Vec<ImpactHeader>,
    /// `dropped_directly \ dropped_from_root` — shed headers that keep
    /// arriving via sibling paths; never counted as savings.
    pub still_reachable: Vec<ImpactHeader>,
    pub root_closure_before: usize,
    pub root_closure_after: usize,
    /// Σ symbol_count over `dropped_from_root` only.
    pub symbols_no_longer_parsed: usize,
}

impl<'a> LoadAnalyzer<'a> {
    pub fn new(include_graph: &'a IncludeGraph, model: &'a ReducedSemanticModel) -> Self {
        Self {
            include_graph,
            model,
            closures: OnceLock::new(),
            symbol_adjacency: OnceLock::new(),
            semantic_closure_cache: Mutex::new(HashMap::new()),
            physical_expansion_cache: Mutex::new(HashMap::new()),
            #[cfg(test)]
            physical_expansions: AtomicUsize::new(0),
            #[cfg(test)]
            semantic_bfs_traversals: AtomicUsize::new(0),
        }
    }

    /// Reuse include closures already computed elsewhere (e.g. by `build_include_graph`)
    /// instead of recomputing them lazily on the first query.
    pub fn with_closures(
        include_graph: &'a IncludeGraph,
        model: &'a ReducedSemanticModel,
        closures: HashMap<FileId, IncludeClosure>,
    ) -> Self {
        Self {
            include_graph,
            model,
            closures: OnceLock::from(closures),
            symbol_adjacency: OnceLock::new(),
            semantic_closure_cache: Mutex::new(HashMap::new()),
            physical_expansion_cache: Mutex::new(HashMap::new()),
            #[cfg(test)]
            physical_expansions: AtomicUsize::new(0),
            #[cfg(test)]
            semantic_bfs_traversals: AtomicUsize::new(0),
        }
    }

    fn find_symbol_id(&self, symbol_name_or_id: &str) -> Option<SymbolId> {
        if let Some((k, _)) = self
            .model
            .symbols
            .iter()
            .find(|(k, v)| k.0 == symbol_name_or_id || v.name == symbol_name_or_id)
        {
            return Some(k.clone());
        }
        None
    }

    /// Renders the file name for a graph file id, falling back to the
    /// `"unknown"` placeholder `compute_load` uses when the id is referenced
    /// only by edges and never registered in `file_paths`.
    fn file_basename(&self, id: FileId) -> String {
        match self.include_graph.file_paths.get(&id) {
            Some(path) => path_basename(path),
            None => "unknown".to_string(),
        }
    }

    fn get_transitive_includes(&self, file_id: FileId) -> HashSet<FileId> {
        let closures = self
            .closures
            .get_or_init(|| self.include_graph.compute_closures());
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
    fn get_semantic_closure_of_symbols(&self, start_symbols: &[SymbolId]) -> HashSet<SymbolId> {
        if let [only_seed] = start_symbols {
            return (*self.cached_single_seed_closure(only_seed)).clone();
        }
        let mut visited = HashSet::new();
        for seed in start_symbols {
            visited.extend(self.cached_single_seed_closure(seed).iter().cloned());
        }
        visited
    }

    /// Single-seed semantic closure, memoized in `semantic_closure_cache`:
    /// the cache `lock()` guard ends at the end of the lookup statement, so
    /// the miss-path insert lock never overlaps another lock. Hits clone
    /// only the `Arc` (refcount bump), never the closure set itself.
    fn cached_single_seed_closure(&self, seed: &SymbolId) -> Arc<HashSet<SymbolId>> {
        let memoized = self
            .semantic_closure_cache
            .lock()
            .unwrap()
            .get(seed)
            .cloned();
        if let Some(closure) = memoized {
            return closure;
        }
        let closure = Arc::new(self.compute_single_seed_closure(seed));
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
    fn compute_single_seed_closure(&self, seed: &SymbolId) -> HashSet<SymbolId> {
        #[cfg(test)]
        self.record_semantic_bfs_traversal();

        let adjacency = self.symbol_adjacency.get_or_init(|| {
            let mut adj: HashMap<SymbolId, Vec<SymbolId>> = HashMap::new();
            for edge in &self.model.edges {
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
    /// this analyzer. Gated behind `cfg(test)` so release builds pay nothing.
    #[cfg(test)]
    fn record_semantic_bfs_traversal(&self) {
        self.semantic_bfs_traversals.fetch_add(1, Ordering::Relaxed);
    }

    /// Test-only: tally of semantic-closure BFS traversals run so far.
    #[cfg(test)]
    fn semantic_bfs_traversal_count(&self) -> usize {
        self.semantic_bfs_traversals.load(Ordering::Relaxed)
    }

    /// Test-only: counts each real physical expansion executed by this
    /// analyzer (a per-def-file cache miss, or a multi-def-file expansion
    /// that is never memoized). Gated behind `cfg(test)`.
    #[cfg(test)]
    fn record_physical_expansion(&self) {
        self.physical_expansions.fetch_add(1, Ordering::Relaxed);
    }

    /// Test-only: tally of real physical expansions run so far.
    #[cfg(test)]
    fn physical_expansion_count(&self) -> usize {
        self.physical_expansions.load(Ordering::Relaxed)
    }

    /// Test-only: number of memoized single-seed semantic closures.
    #[cfg(test)]
    fn semantic_closure_cache_len(&self) -> usize {
        self.semantic_closure_cache.lock().unwrap().len()
    }

    /// Test-only: number of memoized single-def-file physical expansions.
    #[cfg(test)]
    fn physical_expansion_cache_len(&self) -> usize {
        self.physical_expansion_cache.lock().unwrap().len()
    }

    /// Groups symbols by defining file into contributor entries, applying the
    /// shared `(symbol_count desc, path asc)` ordering contract.
    fn contributing_sources(&self, symbols: &[SymbolId]) -> Vec<ContributingSource> {
        let mut by_file: HashMap<FileId, Vec<SymbolId>> = HashMap::new();
        for sym in symbols {
            if let Some(record) = self.model.symbols.get(sym) {
                if let Some(f) = record.defining_file {
                    by_file.entry(f).or_default().push(sym.clone());
                }
            }
        }

        let mut sources: Vec<ContributingSource> = by_file
            .into_iter()
            .map(|(file_id, syms)| ContributingSource {
                path: self
                    .include_graph
                    .file_paths
                    .get(&file_id)
                    .cloned()
                    .unwrap_or_else(|| PathBuf::from("unknown")),
                file_id,
                symbol_count: syms.len(),
                symbols: syms,
            })
            .collect();

        sources.sort_by(|a, b| {
            b.symbol_count
                .cmp(&a.symbol_count)
                .then_with(|| a.path.cmp(&b.path))
        });
        sources
    }

    /// Required headers for a set of symbols: the defining files of the
    /// semantic closure of `symbol_ids` (with contributing closure-symbol
    /// counts), EXCLUDING the seeds' own defining files — a header does not
    /// "require" the file the seed itself is defined in. The exclusion is
    /// per seed defining-file, not per symbol: a non-seed closure symbol is
    /// dropped too whenever its defining file equals ANY seed defining file.
    /// Rows are includable headers only (E-I9): a closure symbol defined in
    /// a translation unit (`.cc`/`.cpp`/...) contributes no include
    /// requirement — it is satisfied by a declaration elsewhere or by
    /// linking — and defining files unregistered in `file_paths` cannot be
    /// shown to be headers, so they are dropped as well. Filtering happens
    /// before grouping, so counts never fold in source files. Unknown
    /// symbol ids are ignored. Ordered `(count desc, path asc)` per I6,
    /// sharing the grouping contract of `contributing_sources`.
    pub fn required_headers_for_symbols(
        &self,
        symbol_ids: &[SymbolId],
    ) -> Vec<(FileId, PathBuf, usize)> {
        let seed_files: HashSet<FileId> = symbol_ids
            .iter()
            .filter_map(|s| self.model.symbols.get(s).and_then(|r| r.defining_file))
            .collect();
        let closure = self.get_semantic_closure_of_symbols(symbol_ids);
        let required: Vec<SymbolId> = closure
            .into_iter()
            .filter(|sym| {
                self.model
                    .symbols
                    .get(sym)
                    .and_then(|r| r.defining_file)
                    .is_some_and(|f| {
                        !seed_files.contains(&f)
                            && self
                                .include_graph
                                .file_paths
                                .get(&f)
                                .is_some_and(|p| is_includable_header(p))
                    })
            })
            .collect();
        self.contributing_sources(&required)
            .into_iter()
            .map(|src| (src.file_id, src.path, src.symbol_count))
            .collect()
    }

    /// Strict include closure of a file: transitive includes WITHOUT the file
    /// itself, served from the shared `closures` cache (real edges only).
    /// The OnceLock guard is never held across derived work — the returned
    /// `&HashMap` is a plain shared borrow.
    fn strict_closure(&self, file_id: FileId) -> HashSet<FileId> {
        self.closures
            .get_or_init(|| self.include_graph.compute_closures())
            .get(&file_id)
            .map(|c| c.transitive_includes.iter().copied().collect())
            .unwrap_or_default()
    }

    /// BFS from `start` over real include edges where `source`'s outgoing
    /// edges are REPLACED by edges to every header in `replacement`
    /// (override expansion; the graph is never cloned). The result contains
    /// `start` itself; callers using strict-closure sizes subtract it.
    /// The `closures` cache models real edges only, so closure_after cannot
    /// reuse it — this BFS is the modeled after-state.
    fn override_closure(
        &self,
        start: FileId,
        source: FileId,
        replacement: &HashSet<FileId>,
    ) -> HashSet<FileId> {
        let mut adj: HashMap<FileId, Vec<FileId>> = HashMap::new();
        for edge in &self.include_graph.edges {
            if let Some(dest) = edge.destination {
                adj.entry(edge.source).or_default().push(dest);
            }
        }

        let mut visited: HashSet<FileId> = HashSet::new();
        visited.insert(start);
        let mut stack = vec![start];
        while let Some(current) = stack.pop() {
            let nexts: Vec<FileId> = if current == source {
                replacement.iter().copied().collect()
            } else {
                adj.get(&current).cloned().unwrap_or_default()
            };
            for next in nexts {
                if visited.insert(next) {
                    stack.push(next);
                }
            }
        }
        visited
    }

    /// One header row: identity plus the model's total symbol count for the
    /// file (0 when none registered).
    fn impact_header(&self, file_id: FileId) -> ImpactHeader {
        ImpactHeader {
            file_id,
            path: self
                .include_graph
                .file_paths
                .get(&file_id)
                .cloned()
                .unwrap_or_else(|| PathBuf::from("unknown")),
            symbol_count: self
                .model
                .file_symbols
                .get(&file_id)
                .map(|s| s.len())
                .unwrap_or(0),
        }
    }

    fn impact_headers(&self, ids: &HashSet<FileId>) -> Vec<ImpactHeader> {
        let mut rows: Vec<ImpactHeader> = ids.iter().map(|&id| self.impact_header(id)).collect();
        rows.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.file_id.cmp(&b.file_id)));
        rows
    }

    /// Per-consumer extraction preview (E-I1..E-I3): given a consumer edge
    /// root F → source S and the symbols justified on that edge, compute what
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
        if root == source || justified_symbols.is_empty() {
            return None;
        }
        if !self.include_graph.file_paths.contains_key(&root)
            || !self.include_graph.file_paths.contains_key(&source)
        {
            return None;
        }

        // R never contains S (contract: "excluding S itself"); the A2 rule
        // drops seed defining files, but justified symbols need not all be
        // defined in S, so filter defensively.
        let required: Vec<(FileId, PathBuf, usize)> = self
            .required_headers_for_symbols(justified_symbols)
            .into_iter()
            .filter(|(id, _, _)| *id != source)
            .collect();
        let required_set: HashSet<FileId> = required.iter().map(|(id, _, _)| *id).collect();

        let kept_subtree = self.override_closure(source, source, &required_set);
        let dropped_directly: HashSet<FileId> = self
            .strict_closure(source)
            .difference(&kept_subtree)
            .copied()
            .collect();

        let root_after = self.override_closure(root, source, &required_set);
        let root_closure_after = root_after.len().saturating_sub(1);
        let root_before = self.strict_closure(root);
        let dropped_from_root: HashSet<FileId> =
            root_before.difference(&root_after).copied().collect();
        let still_reachable: HashSet<FileId> = dropped_directly
            .difference(&dropped_from_root)
            .copied()
            .collect();

        let partition_union: HashSet<FileId> =
            dropped_from_root.union(&still_reachable).copied().collect();
        debug_assert!(
            partition_union == dropped_directly && dropped_from_root.is_disjoint(&still_reachable),
            "E-I1: dropped_directly must partition into dropped_from_root ⊎ still_reachable"
        );

        let dropped_from_root_rows = self.impact_headers(&dropped_from_root);
        Some(ExtractionImpact {
            root_file_id: root,
            source_file_id: source,
            required_headers: self.impact_headers(&required_set),
            dropped_directly: self.impact_headers(&dropped_directly),
            dropped_from_root: dropped_from_root_rows.clone(),
            still_reachable: self.impact_headers(&still_reachable),
            root_closure_before: root_before.len(),
            root_closure_after,
            symbols_no_longer_parsed: dropped_from_root_rows.iter().map(|h| h.symbol_count).sum(),
        })
    }

    /// Physical expansion of a def-file set: the union of
    /// `model.file_symbols` over each def-file's transitive includes (the
    /// seed files included), exactly as `compute_load` used to build it
    /// inline. Each call is one REAL expansion and counts on the test-only
    /// `physical_expansions` tally; the single-file memo below decides how
    /// often this runs.
    fn expand_physical_symbols(&self, def_files: &HashSet<FileId>) -> HashSet<SymbolId> {
        #[cfg(test)]
        self.record_physical_expansion();

        let mut physical_include_files = HashSet::new();
        for f in def_files {
            physical_include_files.extend(self.get_transitive_includes(*f));
        }

        let mut physical_symbols = HashSet::new();
        for f in &physical_include_files {
            if let Some(syms) = self.model.file_symbols.get(f) {
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
    fn cached_physical_expansion(&self, file_id: FileId) -> Arc<HashSet<SymbolId>> {
        let memoized = self
            .physical_expansion_cache
            .lock()
            .unwrap()
            .get(&file_id)
            .cloned();
        if let Some(expansion) = memoized {
            return expansion;
        }
        let expansion = Arc::new(self.expand_physical_symbols(&HashSet::from([file_id])));
        self.physical_expansion_cache
            .lock()
            .unwrap()
            .insert(file_id, Arc::clone(&expansion));
        expansion
    }

    /// The semantic closure is consumed by reference: every caller either
    /// owns a freshly built set or borrows the memoized `Arc` in
    /// `analyze_symbol_load`, and this pass only ever queries `contains`,
    /// so the closure set is never cloned just to be read here.
    fn compute_load(
        &self,
        target_name: String,
        target_file: Option<PathBuf>,
        def_files: HashSet<FileId>,
        semantic_closure: &HashSet<SymbolId>,
    ) -> Result<LoadMetric> {
        // Single-def-file fast path (both public analyzers always hit it):
        // borrow the memoized physical-expansion `Arc` instead of re-unioning
        // the def-file's whole include-closure symbol set (~thousands of
        // `SymbolId` string clones per call). Multi-file def-file sets (none
        // in practice) keep the old non-memoized expansion, so every metric
        // is computed from the identical element set either way.
        let memoized: Option<Arc<HashSet<SymbolId>>> = match def_files.len() {
            1 => def_files
                .iter()
                .next()
                .map(|&f| self.cached_physical_expansion(f)),
            _ => None,
        };
        let expanded: HashSet<SymbolId>;
        let physical_symbols: &HashSet<SymbolId> = match &memoized {
            Some(expansion) => expansion,
            None => {
                expanded = self.expand_physical_symbols(&def_files);
                &expanded
            }
        };

        let mut collateral_symbols: Vec<SymbolId> = physical_symbols
            .iter()
            .filter(|s| !semantic_closure.contains(s))
            .cloned()
            .collect();
        let mut semantic_sym_vec: Vec<SymbolId> = physical_symbols
            .iter()
            .filter(|s| semantic_closure.contains(s))
            .cloned()
            .collect();

        collateral_symbols.sort();
        semantic_sym_vec.sort();

        let collateral_sources = self.contributing_sources(&collateral_symbols);
        let largest_collateral_source = collateral_sources.first().cloned();
        let semantic_sources = self.contributing_sources(&semantic_sym_vec);

        Ok(LoadMetric {
            target_name,
            target_file,
            semantic_load: semantic_sym_vec.len(),
            include_load: physical_symbols.len(),
            collateral_load: collateral_symbols.len(),
            total_load: physical_symbols.len(),
            semantic_symbols: semantic_sym_vec,
            collateral_symbols,
            collateral_sources,
            largest_collateral_source,
            semantic_sources,
        })
    }

    pub fn analyze_symbol_load(&self, symbol_name_or_id: &str) -> Result<LoadMetric> {
        let sym_id = self
            .find_symbol_id(symbol_name_or_id)
            .ok_or_else(|| anyhow!("Symbol not found: {}", symbol_name_or_id))?;

        let record = self
            .model
            .symbols
            .get(&sym_id)
            .ok_or_else(|| anyhow!("Symbol record not found for: {}", symbol_name_or_id))?;
        let def_file = record
            .defining_file
            .ok_or_else(|| anyhow!("No defining file for symbol"))?;
        let target_file_path = self.include_graph.file_paths.get(&def_file).cloned();

        // Hot path: borrow the memoized `Arc` closure directly — the
        // pipeline's per-symbol pass calls this once per symbol, and
        // cloning the whole closure set per call was the dominant
        // cache-hit cost the Arc memo exists to remove.
        let semantic_closure = self.cached_single_seed_closure(&sym_id);

        let mut def_files = HashSet::new();
        def_files.insert(def_file);

        self.compute_load(
            record.name.clone(),
            target_file_path,
            def_files,
            &semantic_closure,
        )
    }

    pub fn analyze_file_load(&self, file_path_or_id: &Path) -> Result<LoadMetric> {
        let file_id = self
            .include_graph
            .find_file_id(file_path_or_id)
            .ok_or_else(|| anyhow!("File not found in include graph"))?;

        let target_file = self.include_graph.file_paths.get(&file_id).cloned();
        let target_name = target_file
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let syms = self
            .model
            .file_symbols
            .get(&file_id)
            .cloned()
            .unwrap_or_default();
        let semantic_closure = self.get_semantic_closure_of_symbols(&syms);

        let mut def_files = HashSet::new();
        def_files.insert(file_id);

        self.compute_load(target_name, target_file, def_files, &semantic_closure)
    }

    fn find_shortest_include_path(&self, start: FileId, end: FileId) -> Option<Vec<FileId>> {
        if start == end {
            return Some(vec![start]);
        }

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(vec![start]);
        let mut visited = HashSet::new();
        visited.insert(start);

        while let Some(path) = queue.pop_front() {
            // Queue entries are seeded with `[start]` and grow one hop at a
            // time, so `last()` is a defensive guard, not a reachable None.
            let current = match path.last() {
                Some(&id) => id,
                None => continue,
            };
            for dest in self.include_graph.direct_includes(current) {
                if dest == end {
                    let mut new_path = path.clone();
                    new_path.push(dest);
                    return Some(new_path);
                }
                if !visited.contains(&dest) {
                    visited.insert(dest);
                    let mut new_path = path.clone();
                    new_path.push(dest);
                    queue.push_back(new_path);
                }
            }
        }
        None
    }

    pub fn explain_symbol_load(
        &self,
        symbol_name_or_id: &str,
        source_filter: Option<&str>,
    ) -> Result<LoadExplanation> {
        let metric = self.analyze_symbol_load(symbol_name_or_id)?;

        let col_source = if let Some(filter) = source_filter {
            metric
                .collateral_sources
                .iter()
                .find(|cs| cs.path.to_string_lossy().contains(filter))
                .cloned()
        } else {
            metric.largest_collateral_source.clone()
        };

        let col_source =
            col_source.ok_or_else(|| anyhow!("No matching collateral source found"))?;

        let sym_id = self
            .find_symbol_id(symbol_name_or_id)
            .ok_or_else(|| anyhow!("Symbol not found: {}", symbol_name_or_id))?;
        let record = self
            .model
            .symbols
            .get(&sym_id)
            .ok_or_else(|| anyhow!("Symbol record not found for: {}", symbol_name_or_id))?;
        let target_file_id = record
            .defining_file
            .ok_or_else(|| anyhow!("No defining file for symbol"))?;

        let path_ids = self
            .find_shortest_include_path(target_file_id, col_source.file_id)
            .ok_or_else(|| anyhow!("No include path found to collateral source"))?;

        let mut path_strs = Vec::new();
        path_strs.push(metric.target_name.clone());
        for id in &path_ids {
            path_strs.push(self.file_basename(*id));
        }

        let parent_file_id = if path_ids.len() >= 2 {
            path_ids[path_ids.len() - 2]
        } else {
            path_ids[0]
        };
        let parent_name = self.file_basename(parent_file_id);
        let col_name = path_basename(&col_source.path);

        let reason = format!(
            "{} is included by {}.\nNo semantic dependency from {} reaches symbols owned by {}.",
            col_name, parent_name, metric.target_name, col_name
        );

        Ok(LoadExplanation {
            target_name: metric.target_name,
            target_file: metric
                .target_file
                .unwrap_or_else(|| PathBuf::from("unknown")),
            path: path_strs,
            collateral_source: col_name,
            collateral_symbols_count: col_source.symbol_count,
            reason,
        })
    }

    pub fn explain_file_load(
        &self,
        file_path_or_id: &Path,
        source_filter: Option<&str>,
    ) -> Result<LoadExplanation> {
        let metric = self.analyze_file_load(file_path_or_id)?;
        let file_id = self
            .include_graph
            .find_file_id(file_path_or_id)
            .ok_or_else(|| anyhow!("File not found in include graph"))?;

        let col_source = if let Some(filter) = source_filter {
            metric
                .collateral_sources
                .iter()
                .find(|cs| cs.path.to_string_lossy().contains(filter))
                .cloned()
        } else {
            metric.largest_collateral_source.clone()
        };

        let col_source =
            col_source.ok_or_else(|| anyhow!("No matching collateral source found"))?;

        let path_ids = self
            .find_shortest_include_path(file_id, col_source.file_id)
            .ok_or_else(|| anyhow!("No include path found to collateral source"))?;

        let mut path_strs = Vec::new();
        for id in &path_ids {
            path_strs.push(self.file_basename(*id));
        }

        let parent_file_id = if path_ids.len() >= 2 {
            path_ids[path_ids.len() - 2]
        } else {
            path_ids[0]
        };
        let parent_name = self.file_basename(parent_file_id);
        let col_name = path_basename(&col_source.path);

        let reason = format!(
            "{} is included by {}.\nNo semantic dependency from {} reaches symbols owned by {}.",
            col_name, parent_name, metric.target_name, col_name
        );

        Ok(LoadExplanation {
            target_name: metric.target_name,
            target_file: metric
                .target_file
                .unwrap_or_else(|| PathBuf::from("unknown")),
            path: path_strs,
            collateral_source: col_name,
            collateral_symbols_count: col_source.symbol_count,
            reason,
        })
    }
}

/// Characterization suite: locks CURRENT behavior of the load analyzer
/// (explain_symbol_load outputs incl. error paths) via mock fixtures;
/// moved to a sibling module following the discovery.rs pattern.
#[cfg(test)]
#[path = "load_characterization_tests.rs"]
mod characterization;

/// Justification suite: locks the semantic (used-symbol) side of the
/// include-closure split as grouped per-source contributions.
#[cfg(test)]
#[path = "load_justification_tests.rs"]
mod justification;

/// Extraction-impact suite: locks the drop-set semantics of
/// `extraction_impact` (Pass E, plan 1790521470000).
#[cfg(test)]
#[path = "load_impact_tests.rs"]
mod impact;

/// Load-performance suite: locks per-seed semantic-closure memoization
/// (BFS-run counts and cold-vs-warm equivalence, plan LoadPerfImprovements).
#[cfg(test)]
#[path = "load_perf_tests.rs"]
mod perf;
