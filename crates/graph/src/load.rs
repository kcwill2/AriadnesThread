use crate::{IncludeClosure, IncludeGraph, ReducedSemanticModel};
use anyhow::{anyhow, Result};
use ariadne_index::types::SymbolId;
use ariadne_lang_api::modeling::DependencyVocabulary;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Renders a path's final component, falling back to the full path for names
/// without one (e.g. `..`), mirroring `format_summary`'s file_name fallback.
fn path_basename(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

/// E-I9 (plan 1790521470000): needs-sets carry includable headers only. The
/// suffix predicate itself lives in `ariadne-repo::types` (plan §6.3 rule R2
/// keeps header-suffix taxonomy out of the core pipeline crates); this
/// re-export keeps `ariadne_graph::is_includable_header` (ui dto.rs, load
/// justification) as the stable consumer path.
pub use ariadne_repo::is_includable_header;

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
        self.render_summary("Include")
    }

    /// D26 (plan:425): identical rendering with the directive label supplied
    /// by a plugin's `DependencyVocabulary` — the other labels have no
    /// language vocabulary and stay byte-identical.
    pub fn format_summary_with(&self, vocabulary: &DependencyVocabulary) -> String {
        self.render_summary(&vocabulary.directive_label)
    }

    fn render_summary(&self, directive_label: &str) -> String {
        let mut out = String::new();
        out.push_str(&self.target_name);
        out.push_str("\n\nSemantic load:\n");
        out.push_str(&format!("    {} symbols\n\n", self.semantic_load));

        out.push_str(&format!("{directive_label} load:\n"));
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

/// Cheap query handle over an `IncludeGraph` + `ReducedSemanticModel` pair
/// and one shared `Arc<LoadMemo>` (plan 1790608181551 B1): every cache field
/// and cache method moved verbatim to `load_caches.rs`, so constructing an
/// analyzer is a refcount bump, and `with_memo` lets a UI server or the
/// pipeline share ONE warmed memo across requests/workers instead of
/// cold-starting per construction. Thread-safe (`Send + Sync`) so the
/// pipeline can share ONE memoized analyzer across rayon workers (plan
/// LoadPerfImprovements Phase 4); the lock discipline (I4) is documented on
/// `LoadMemo`.
pub struct LoadAnalyzer<'a> {
    pub include_graph: &'a IncludeGraph,
    pub model: &'a ReducedSemanticModel,
    /// Shared cache generation (I5): cloning a handle bumps the refcount;
    /// the memo's entry bound stays per-analysis-generation.
    memo: Arc<LoadMemo>,
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

/// Single-pass split of the physical expansion into `(semantic, collateral)`
/// vectors (plan LoadPerf A3): one walk of `physical_symbols` instead of the
/// old two filter passes, then one ascending sort each. Sorting after the
/// walk keeps the order fed to `contributing_sources` deterministic no matter
/// how the input `HashSet` iterates (I3).
fn partition_symbols(
    physical_symbols: &HashSet<SymbolId>,
    semantic_closure: &HashSet<SymbolId>,
) -> (Vec<SymbolId>, Vec<SymbolId>) {
    let mut semantic: Vec<SymbolId> = Vec::new();
    let mut collateral: Vec<SymbolId> = Vec::new();
    for symbol in physical_symbols {
        if semantic_closure.contains(symbol) {
            semantic.push(symbol.clone());
        } else {
            collateral.push(symbol.clone());
        }
    }
    semantic.sort();
    collateral.sort();
    (semantic, collateral)
}

impl<'a> LoadAnalyzer<'a> {
    /// Cold-start behavior unchanged from pre-B1: every fresh analyzer gets
    /// a fresh `Arc<LoadMemo>` — same once-per-instance memoization as before
    /// the caches moved out.
    pub fn new(include_graph: &'a IncludeGraph, model: &'a ReducedSemanticModel) -> Self {
        Self {
            include_graph,
            model,
            memo: Arc::new(LoadMemo::new()),
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
            memo: Arc::new(LoadMemo::with_closures(closures)),
        }
    }

    /// Handle over an ALREADY SHARED memo (plan 1790608181551 B1): O(1)
    /// construction that reuses whatever the memo has already warmed — the
    /// cheap handle the UI route path (C1) builds per request. Sharing one
    /// memo across graph/model generations is the caller's contract: the
    /// memo keys on FileId/SymbolId, which are generation-local.
    pub fn with_memo(
        include_graph: &'a IncludeGraph,
        model: &'a ReducedSemanticModel,
        memo: Arc<LoadMemo>,
    ) -> Self {
        Self {
            include_graph,
            model,
            memo,
        }
    }

    /// Clone of the shared memo handle (refcount bump) — what `with_memo`
    /// takes to build sibling handles over the same caches.
    pub fn memo(&self) -> Arc<LoadMemo> {
        Arc::clone(&self.memo)
    }

    /// Resolves a symbol NAME or id STRING to its `SymbolId`: a query
    /// matches either the BTreeMap key's string (`id.0`) or the record's
    /// `name` field, first match in ascending key order wins. This is the
    /// O(S) string scan the audit (P1 part 1) flags; kept verbatim for the
    /// legacy name entry so its public behavior is byte-identical. Callers
    /// that already hold ids should use `analyze_symbol_load_by_id` and
    /// skip this resolution entirely.
    pub fn symbol_id_by_name(&self, symbol_name_or_id: &str) -> Option<SymbolId> {
        self.model
            .symbols
            .iter()
            .find(|(k, v)| k.0 == symbol_name_or_id || v.name == symbol_name_or_id)
            .map(|(k, _)| k.clone())
    }

    /// Private alias kept for the `load_explain` child module (out of
    /// AR-07's touch-set); delegates to the public helper verbatim.
    fn find_symbol_id(&self, symbol_name_or_id: &str) -> Option<SymbolId> {
        self.symbol_id_by_name(symbol_name_or_id)
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

    /// Test-only: tally of semantic-closure BFS traversals run so far on
    /// this analyzer's shared memo (delegates to `LoadMemo`).
    #[cfg(test)]
    fn semantic_bfs_traversal_count(&self) -> usize {
        self.memo.semantic_bfs_traversal_count()
    }

    /// Test-only: tally of real physical expansions run so far on this
    /// analyzer's shared memo (delegates to `LoadMemo`).
    #[cfg(test)]
    fn physical_expansion_count(&self) -> usize {
        self.memo.physical_expansion_count()
    }

    /// Test-only: tally of include-adjacency builds run so far on this
    /// analyzer's shared memo; the OnceLock memo contract caps this at one.
    #[cfg(test)]
    fn include_adjacency_build_count(&self) -> usize {
        self.memo.include_adjacency_build_count()
    }

    /// Test-only: number of memoized single-seed semantic closures.
    #[cfg(test)]
    fn semantic_closure_cache_len(&self) -> usize {
        self.memo.semantic_closure_cache_len()
    }

    /// Test-only: number of memoized single-def-file physical expansions.
    #[cfg(test)]
    fn physical_expansion_cache_len(&self) -> usize {
        self.memo.physical_expansion_cache_len()
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
        let closure = self
            .memo
            .get_semantic_closure_of_symbols(self.model, symbol_ids);
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

    /// BFS from `start` over real include edges where `source`'s outgoing
    /// edges are REPLACED by edges to every header in `replacement`
    /// (override expansion; the graph is never cloned). The result contains
    /// `start` itself; callers using strict-closure sizes subtract it.
    /// The `closures` cache models real edges only, so closure_after cannot
    /// reuse it — this BFS is the modeled after-state, run over the shared
    /// `file_adjacency` memo.
    fn override_closure(
        &self,
        start: FileId,
        source: FileId,
        replacement: &HashSet<FileId>,
    ) -> HashSet<FileId> {
        let adj = self.memo.file_adjacency_map(self.include_graph);
        let mut visited: HashSet<FileId> = HashSet::new();
        visited.insert(start);
        let mut stack = vec![start];
        while let Some(current) = stack.pop() {
            if current == source {
                // The modeled slim edge replaces S's real outgoing edges.
                // Set order is irrelevant: the traversal result is a set.
                for &next in replacement {
                    if visited.insert(next) {
                        stack.push(next);
                    }
                }
            } else if let Some(nexts) = adj.get(&current) {
                for &next in nexts {
                    if visited.insert(next) {
                        stack.push(next);
                    }
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
            .memo
            .strict_closure(self.include_graph, source)
            .difference(&kept_subtree)
            .copied()
            .collect();

        let root_after = self.override_closure(root, source, &required_set);
        let root_closure_after = root_after.len().saturating_sub(1);
        let root_before = self
            .memo
            .strict_closure(self.include_graph, root);
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
            1 => def_files.iter().next().map(|&f| {
                self.memo
                    .cached_physical_expansion(self.include_graph, self.model, f)
            }),
            _ => None,
        };
        let expanded: HashSet<SymbolId>;
        let physical_symbols: &HashSet<SymbolId> = match &memoized {
            Some(expansion) => expansion,
            None => {
                expanded = self
                    .memo
                    .expand_physical_symbols(self.include_graph, self.model, &def_files);
                &expanded
            }
        };

        let (semantic_sym_vec, collateral_symbols) =
            partition_symbols(physical_symbols, semantic_closure);

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

    /// Name entry: resolve via `symbol_id_by_name`, then delegate. Zero
    /// behavior change — the metric body lives in `analyze_symbol_load_by_id`
    /// alone, so the two entries can never drift (mirrors the
    /// `analyze_file_load` / `analyze_file_load_by_id` precedent).
    pub fn analyze_symbol_load(&self, symbol_name_or_id: &str) -> Result<LoadMetric> {
        let sym_id = self
            .find_symbol_id(symbol_name_or_id)
            .ok_or_else(|| anyhow!("Symbol not found: {}", symbol_name_or_id))?;
        self.analyze_symbol_load_by_id(&sym_id)
    }

    /// Symbol load from a pre-resolved `SymbolId`: skips the O(S)
    /// `symbol_id_by_name` string scan that the name entry pays per call —
    /// the pipeline's per-symbol pass (the AR-08 consumer of this seam)
    /// already holds the ids it is iterating. An id unregistered in
    /// `model.symbols` errors; a registered record without a defining file
    /// errors with the same `"No defining file for symbol"` message the
    /// name entry has always produced.
    pub fn analyze_symbol_load_by_id(&self, symbol_id: &SymbolId) -> Result<LoadMetric> {
        let record = self
            .model
            .symbols
            .get(symbol_id)
            .ok_or_else(|| anyhow!("Symbol record not found for: {}", symbol_id.0))?;
        let def_file = record
            .defining_file
            .ok_or_else(|| anyhow!("No defining file for symbol"))?;
        let target_file_path = self.include_graph.file_paths.get(&def_file).cloned();

        // Hot path: borrow the memoized `Arc` closure directly — the
        // pipeline's per-symbol pass calls this once per symbol, and
        // cloning the whole closure set per call was the dominant
        // cache-hit cost the Arc memo exists to remove.
        let semantic_closure = self
            .memo
            .cached_single_seed_closure(self.model, symbol_id);

        let mut def_files = HashSet::new();
        def_files.insert(def_file);

        self.compute_load(
            record.name.clone(),
            target_file_path,
            def_files,
            &semantic_closure,
        )
    }

    /// Path entry: resolve the id, then delegate. Zero behavior change —
    /// the metric body lives in `analyze_file_load_by_id` alone, so the two
    /// entries can never drift.
    pub fn analyze_file_load(&self, file_path_or_id: &Path) -> Result<LoadMetric> {
        let file_id = self
            .include_graph
            .find_file_id(file_path_or_id)
            .ok_or_else(|| anyhow!("File not found in include graph"))?;
        self.analyze_file_load_by_id(file_id)
    }

    /// File load from a pre-resolved `FileId`: skips the O(V) `find_file_id`
    /// string scan that the path entry pays per call — the pipeline file
    /// pass (plan LoadPerf A2) already holds the ids it is iterating. An id
    /// unregistered in `file_paths` analyzes as the `"unknown"`-named empty
    /// target (callers that must skip such ids check registration first).
    pub fn analyze_file_load_by_id(&self, file_id: FileId) -> Result<LoadMetric> {
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
        let semantic_closure = self.memo.get_semantic_closure_of_symbols(self.model, &syms);

        let mut def_files = HashSet::new();
        def_files.insert(file_id);

        self.compute_load(target_name, target_file, def_files, &semantic_closure)
    }
}

/// Shared memoization caches (`LoadMemo`) relocated to a sibling module to
/// keep both files comfortably under the R2 line limit; pure code move
/// (plan 1790608181551 B1). `pub use` keeps it reachable as
/// `ariadne_graph::LoadMemo` for the memo-sharing consumers (C1).
#[path = "load_caches.rs"]
mod caches;

pub use caches::LoadMemo;

/// Explanation surface (`LoadExplanation` + the `explain_*` methods and
/// their shared shortest-path BFS) relocated to a sibling module to keep
/// both files comfortably under the R2 line limit; pure code move.
#[path = "load_explain.rs"]
mod explain;

pub use explain::LoadExplanation;

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

/// B1 memo-sharing suite: locks shared-`Arc<LoadMemo>` handles (Send+Sync
/// contract, zero re-work on a warm shared memo, rayon equivalence; plan
/// 1790608181551). Split from the perf suite to keep both under R2.
#[cfg(test)]
#[path = "load_memo_tests.rs"]
mod memo_perf;

/// D26 vocabulary suite: locks `format_summary_with` against the unchanged
/// default summary rendering (plan M22, plan:425).
#[cfg(test)]
#[path = "load_vocabulary_tests.rs"]
mod vocabulary;

/// UTA-T1 degeneracy suite: pins the `include_load == total_load` identity
/// of `compute_load` (UTA-P row, owner decision pending; the
/// `extraction_impact` guard rows are already locked by
/// `test_impact_guards_return_none` in the impact suite).
#[cfg(test)]
#[path = "load_degeneracy_tests.rs"]
mod degeneracy;

/// AR-07 by-id suite: locks the `analyze_symbol_load_by_id` /
/// `symbol_id_by_name` seam (audit P1 part 1) — parity with the name entry,
/// resolution semantics (id-string OR record-name match), and the exact
/// error strings of every miss path. Inline (not a sibling `#[path]` file)
/// because the AR-07 touch-set scopes new files to `load.rs` +
/// `crates/graph/tests/**`.
#[cfg(test)]
mod symbol_load_by_id {
    use super::*;
    use crate::reduced::ReducedSymbolRecord;
    use crate::types::{IncludeEdge, IncludeKind, LanguageId, SourceLocation};
    use ariadne_index::types::SymbolKind;
    use std::collections::BTreeMap;

    /// Thing.h -> graphics.h -> HugeThirdParty.h with one symbol in Thing.h
    /// (whose record name deliberately differs from its id string, locking
    /// the dual-match of the resolution helper) and one collateral symbol in
    /// HugeThirdParty.h. Mirrors the characterization fixture shape.
    fn fixture() -> (IncludeGraph, ReducedSemanticModel) {
        let mut ig = IncludeGraph::new();
        ig.register_file(FileId(1), PathBuf::from("Thing.h"));
        ig.register_file(FileId(2), PathBuf::from("graphics.h"));
        ig.register_file(FileId(3), PathBuf::from("HugeThirdParty.h"));
        ig.add_edge(IncludeEdge {
            source: FileId(1),
            destination: Some(FileId(2)),
            context_name: "".to_string(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "graphics.h".to_string(),
            resolved_path: Some(PathBuf::from("graphics.h")),
            language: LanguageId::cpp(),
        });
        ig.add_edge(IncludeEdge {
            source: FileId(2),
            destination: Some(FileId(3)),
            context_name: "".to_string(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "HugeThirdParty.h".to_string(),
            resolved_path: Some(PathBuf::from("HugeThirdParty.h")),
            language: LanguageId::cpp(),
        });

        let mut model = ReducedSemanticModel {
            repo_root: PathBuf::from("/"),
            symbols: BTreeMap::new(),
            edges: vec![],
            tu_usage: BTreeMap::new(),
            file_symbols: BTreeMap::new(),
            extends_edges: Vec::new(),
            sealed_symbols: Vec::new(),
        };

        let thing_id = SymbolId::new("ns::Thing");
        model.symbols.insert(
            thing_id.clone(),
            ReducedSymbolRecord {
                id: thing_id.clone(),
                name: "Thing".to_string(),
                kind: SymbolKind::Class,
                defining_file: Some(FileId(1)),
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );
        model.file_symbols.insert(FileId(1), vec![thing_id]);

        let ghost_id = SymbolId::new("Ghost");
        model.symbols.insert(
            ghost_id.clone(),
            ReducedSymbolRecord {
                id: ghost_id,
                name: "Ghost".to_string(),
                kind: SymbolKind::Function,
                defining_file: None,
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );

        (ig, model)
    }

    #[test]
    fn by_id_entry_returns_the_identical_metric_as_the_name_entry() {
        let (ig, model) = fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);
        let resolved = analyzer
            .symbol_id_by_name("Thing")
            .expect("fixture symbol resolvable by record name");

        let by_id = analyzer
            .analyze_symbol_load_by_id(&resolved)
            .expect("by-id analysis failed");
        let by_name = analyzer
            .analyze_symbol_load("Thing")
            .expect("name analysis failed");
        assert_eq!(by_id, by_name);
        assert_eq!(by_id.target_name, "Thing");
        assert_eq!(by_id.target_file, Some(PathBuf::from("Thing.h")));
    }

    #[test]
    fn resolution_helper_matches_id_string_or_record_name_and_misses_are_none() {
        let (ig, model) = fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        let by_record_name = analyzer
            .symbol_id_by_name("Thing")
            .expect("record-name match");
        let by_id_string = analyzer
            .symbol_id_by_name("ns::Thing")
            .expect("id-string match");
        assert_eq!(by_record_name, by_id_string);
        assert_eq!(by_record_name, SymbolId::new("ns::Thing"));
        assert_eq!(analyzer.symbol_id_by_name("Absent"), None);
    }

    #[test]
    fn name_entry_locks_unknown_symbol_error_string() {
        let (ig, model) = fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);
        let error = analyzer
            .analyze_symbol_load("Absent")
            .expect_err("unknown name must error");
        assert_eq!(error.to_string(), "Symbol not found: Absent");
    }

    #[test]
    fn by_id_entry_locks_unregistered_id_error_string() {
        let (ig, model) = fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);
        let error = analyzer
            .analyze_symbol_load_by_id(&SymbolId::new("Absent"))
            .expect_err("unregistered id must error");
        assert_eq!(error.to_string(), "Symbol record not found for: Absent");
    }

    #[test]
    fn by_id_entry_locks_missing_defining_file_error_and_parities_with_name_entry() {
        let (ig, model) = fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);
        let error = analyzer
            .analyze_symbol_load_by_id(&SymbolId::new("Ghost"))
            .expect_err("record without defining file must error");
        assert_eq!(error.to_string(), "No defining file for symbol");

        let name_error = analyzer
            .analyze_symbol_load("Ghost")
            .expect_err("name entry agrees on the same miss");
        assert_eq!(name_error.to_string(), error.to_string());
    }
}
