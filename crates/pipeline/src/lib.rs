//! Artifact-first analysis pipeline.
//!
//! Every stage (`include graph`, `index`, `reduced`, `call graph`, `load`)
//! persists under `artifacts/` with an **input fingerprint**. A stage is
//! loaded when its fingerprint still matches the code on disk; otherwise it
//! is rebuilt wholesale and re-persisted. Translation-unit parsing inside the
//! index stage is still reused per-TU through the shard store, so editing one
//! header re-parses only the TUs whose closure contains it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult};
use ariadne_graph::{
    build_include_graph, CallGraph, IncludeClosure, IncludeGraph, LoadAnalyzer, LoadMetric,
    ReducedSemanticModel,
};
use ariadne_index::{IncrementalStats, IndexResult, StageFingerprints, CURRENT_ANALYSIS_VERSION};
use ariadne_repo::{FileId, Repository, RepositoryDiscoveryResult};
use ariadne_report::{ArtifactManager, LoadReport};
use ariadne_scheduler::{
    analyze_closure, compute_tu_closure, load_shard, save_shard, shard_dir,
    ProductionAnalysisStats, ProductionAnalyzer, ShardCheckpoint, TuStoreContext,
};

pub const STAGE_INCLUDE_GRAPH: &str = "include-graph";
pub const STAGE_INDEX: &str = "index";
pub const STAGE_REDUCED: &str = "reduced";
pub const STAGE_CALLGRAPH: &str = "callgraph";
pub const STAGE_LOAD: &str = "load";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StageReuse {
    Loaded,
    Rebuilt,
}

impl StageReuse {
    pub fn as_str(&self) -> &'static str {
        match self {
            StageReuse::Loaded => "loaded",
            StageReuse::Rebuilt => "rebuilt",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageRecord {
    pub stage: String,
    pub reuse: StageReuse,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunStats {
    pub stages: Vec<StageRecord>,
    pub reused_tus: usize,
    pub reanalyzed_tus: usize,
}

impl RunStats {
    pub fn record(&mut self, stage: &str, reuse: StageReuse) {
        self.stages.retain(|s| s.stage != stage);
        self.stages.push(StageRecord {
            stage: stage.to_string(),
            reuse,
        });
    }

    pub fn stage_reuse(&self, stage: &str) -> Option<StageReuse> {
        self.stages
            .iter()
            .find(|s| s.stage == stage)
            .map(|s| s.reuse)
    }

    pub fn format_summary(&self) -> String {
        let loaded: Vec<&str> = self
            .stages
            .iter()
            .filter(|s| s.reuse == StageReuse::Loaded)
            .map(|s| s.stage.as_str())
            .collect();
        let rebuilt: Vec<&str> = self
            .stages
            .iter()
            .filter(|s| s.reuse == StageReuse::Rebuilt)
            .map(|s| s.stage.as_str())
            .collect();
        format!(
            "{} TUs reused, {} reanalyzed; stages loaded: {}; rebuilt: {}",
            self.reused_tus,
            self.reanalyzed_tus,
            if loaded.is_empty() {
                "none".into()
            } else {
                loaded.join(", ")
            },
            if rebuilt.is_empty() {
                "none".into()
            } else {
                rebuilt.join(", ")
            },
        )
    }
}

#[derive(Debug, Clone)]
pub enum ProgressEvent {
    TuIndex { done: usize, total: usize },
    CollateralFile { done: usize, total: usize },
}

type ProgressCb = Box<dyn FnMut(&ProgressEvent) + Send>;

pub struct Pipeline {
    pub repo: Repository,
    pub manager: ArtifactManager,
    pub discovery: RepositoryDiscoveryResult,
    pub build: BuildDiscoveryResult,
    pub fps: StageFingerprints,
    pub stats: RunStats,
    /// Real measured stats from a workers-path index rebuild, if one ran
    /// during this pipeline. `None` when the index stage was loaded/reused:
    /// callers must then report honest zeros instead of invented numbers.
    pub production_stats: Option<ProductionAnalysisStats>,
    store_ctx: ariadne_scheduler::SharedTuStoreContext,
    workers: Option<usize>,
    progress: Option<ProgressCb>,
    include_cache: Option<(IncludeGraph, HashMap<ariadne_repo::FileId, IncludeClosure>)>,
    index_cache: Option<IndexResult>,
    reduced_cache: Option<ReducedSemanticModel>,
    callgraph_cache: Option<CallGraph>,
    load_cache: Option<Arc<LoadReport>>,
}

impl Pipeline {
    pub fn new(repo_path: &Path, workers: Option<usize>) -> Result<Self> {
        let (repo, discovery, build, fps) = Self::compute_inputs(repo_path)?;
        let manager = ArtifactManager::prepare_for_repo(&repo)?;

        // Repository + contexts artifacts are cheap; refresh them whenever
        // their fingerprint no longer matches.
        if !manager.stage_is_fresh("repository", &fps.repository) {
            manager.save_repository_artifacts(&discovery, &fps.repository)?;
        }
        if !manager.stage_is_fresh("contexts", &fps.contexts) {
            manager.save_build_context_artifacts(&build, &fps.contexts)?;
        }

        let store_ctx = TuStoreContext::new(&discovery, build.contexts.clone());

        Ok(Self {
            repo,
            manager,
            discovery,
            build,
            fps,
            stats: RunStats::default(),
            production_stats: None,
            store_ctx,
            workers,
            progress: None,
            include_cache: None,
            index_cache: None,
            reduced_cache: None,
            callgraph_cache: None,
            load_cache: None,
        })
    }

    pub fn with_progress<F>(mut self, cb: F) -> Self
    where
        F: FnMut(&ProgressEvent) + Send + 'static,
    {
        self.progress = Some(Box::new(cb));
        self
    }

    /// Fresh discovery + build discovery + stage fingerprint chain, computed
    /// identically by every path that derives fingerprints.
    fn compute_inputs(
        repo_path: &Path,
    ) -> Result<(
        Repository,
        RepositoryDiscoveryResult,
        BuildDiscoveryResult,
        StageFingerprints,
    )> {
        let repo = Repository::locate(repo_path)?;
        let discovery = repo.discover()?;
        let build = BuildDiscoverer::new(&repo.root)
            .with_repo_discovery(&discovery)
            .discover()?;
        let fps = StageFingerprints::from_inputs(
            &repo.root,
            &discovery,
            &build.contexts,
            &build.translation_units,
        );
        Ok((repo, discovery, build, fps))
    }

    /// Re-derive the stage fingerprint chain from the current code on disk
    /// without opening, saving, or refreshing any artifacts. Used for pure
    /// freshness checks (e.g. the UI watcher's no-op tick detection).
    pub fn rehash_inputs(repo_path: &Path) -> Result<StageFingerprints> {
        Ok(Self::compute_inputs(repo_path)?.3)
    }

    pub fn artifact_root(&self) -> PathBuf {
        self.manager.artifact_root.clone()
    }

    fn notify(&mut self, event: &ProgressEvent) {
        if let Some(cb) = self.progress.as_mut() {
            cb(event);
        }
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo.root
    }

    /// Report whether a stage artifact is currently fresh on disk.
    pub fn stage_is_fresh(&self, stage: &str, fingerprint: &str) -> bool {
        self.manager.stage_is_fresh(stage, fingerprint)
    }

    // ------------------------------------------------------------------
    // Include graph stage
    // ------------------------------------------------------------------

    pub fn ensure_include_graph(&mut self) -> Result<()> {
        if self.include_cache.is_some() {
            return Ok(());
        }

        if self
            .manager
            .stage_is_fresh("includes", &self.fps.include_graph)
        {
            if let Ok(loaded) = self
                .manager
                .load_include_checked(&self.fps.include_graph, &self.discovery)
            {
                self.include_cache = Some(loaded);
                self.stats.record(STAGE_INCLUDE_GRAPH, StageReuse::Loaded);
                return Ok(());
            }
        }

        let (_repo, graph, closures, _report) = build_include_graph(&self.repo.root)?;
        let report = graph.generate_report();
        self.manager
            .save_include_artifacts(&graph, &closures, &report, &self.fps.include_graph)?;
        self.include_cache = Some((graph, closures));
        self.stats.record(STAGE_INCLUDE_GRAPH, StageReuse::Rebuilt);
        Ok(())
    }

    pub fn get_or_build_include_graph(
        &mut self,
    ) -> Result<(IncludeGraph, HashMap<ariadne_repo::FileId, IncludeClosure>)> {
        self.ensure_include_graph()?;
        self.cached_include_graph()
    }

    // ------------------------------------------------------------------
    // Index stage (TU-level reuse via the shard store)
    // ------------------------------------------------------------------

    pub fn ensure_index(&mut self) -> Result<()> {
        if self.index_cache.is_some() {
            return Ok(());
        }

        if self.manager.stage_is_fresh("index", &self.fps.index) {
            if let Ok(loaded) = self.manager.load_index_checked(&self.fps.index) {
                self.index_cache = Some(loaded);
                self.stats.record(STAGE_INDEX, StageReuse::Loaded);
                return Ok(());
            }
        }

        let (index_result, stats, production_stats) = match self.workers {
            Some(workers) => {
                let analyzer = ProductionAnalyzer::new(&self.repo, &self.discovery, &self.build)
                    .with_workers(workers);
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()?;
                let analysis_result = rt.block_on(analyzer.analyze())?;
                let production = analysis_result.stats;
                let reuse =
                    IncrementalStats::calculate(production.reused_tus, production.reanalyzed_tus);
                (analysis_result.index_result, reuse, Some(production))
            }
            None => {
                let (index_result, stats) = self.rebuild_index_single_threaded()?;
                (index_result, stats, None)
            }
        };

        self.production_stats = production_stats;

        self.manager
            .save_index_artifacts(&index_result, &self.fps.index, Some(&stats))?;
        self.stats.reused_tus = stats.reused_tus;
        self.stats.reanalyzed_tus = stats.reanalyzed_tus;
        self.index_cache = Some(index_result);
        self.stats.record(STAGE_INDEX, StageReuse::Rebuilt);
        Ok(())
    }

    fn rebuild_index_single_threaded(&mut self) -> Result<(IndexResult, IncrementalStats)> {
        let store_dir = shard_dir(&self.repo.root);
        let store_ctx = self.store_ctx.clone();
        let repo_root = self.repo.root.clone();
        let translation_units = self.build.translation_units.clone();
        let mut progress = self.progress.take();
        let mut notify = |event: &ProgressEvent| {
            if let Some(cb) = progress.as_mut() {
                cb(event);
            }
        };

        let mut reused = 0usize;
        let mut reanalyzed = 0usize;
        let mut analyses = Vec::new();

        let total = translation_units.len();
        for (i, tu) in translation_units.iter().enumerate() {
            notify(&ProgressEvent::TuIndex { done: i, total });

            let closure = compute_tu_closure(tu, &repo_root, &store_ctx, CURRENT_ANALYSIS_VERSION)?;
            let analysis = match load_shard(&store_dir, tu.file_id) {
                Some(shard) if shard.fingerprint == closure.fingerprint.combined_hash => {
                    reused += 1;
                    shard.analysis
                }
                _ => {
                    reanalyzed += 1;
                    let analysis = analyze_closure(tu, &closure);
                    save_shard(
                        &store_dir,
                        &ShardCheckpoint {
                            fingerprint: closure.fingerprint.combined_hash.clone(),
                            analysis: analysis.clone(),
                        },
                    )?;
                    analysis
                }
            };
            analyses.push(analysis);
        }
        notify(&ProgressEvent::TuIndex { done: total, total });

        self.progress = progress;
        let index_result = IndexResult::new(repo_root, analyses);
        Ok((
            index_result,
            IncrementalStats::calculate(reused, reanalyzed),
        ))
    }

    pub fn get_or_build_index(&mut self) -> Result<IndexResult> {
        self.ensure_index()?;
        self.cached_index()
    }

    // ------------------------------------------------------------------
    // Reduced stage
    // ------------------------------------------------------------------

    pub fn ensure_reduced(&mut self) -> Result<()> {
        if self.reduced_cache.is_some() {
            return Ok(());
        }

        if self.manager.stage_is_fresh("reduced", &self.fps.reduced) {
            if let Ok(loaded) = self.manager.load_reduced_checked(&self.fps.reduced) {
                self.reduced_cache = Some(loaded);
                self.stats.record(STAGE_REDUCED, StageReuse::Loaded);
                return Ok(());
            }
        }

        self.ensure_index()?;
        let index = self
            .index_cache
            .as_ref()
            .context("index stage cache must exist after ensure_index")?;
        let model = ReducedSemanticModel::reduce(index);
        self.manager
            .save_reduced_artifacts(&model, &self.fps.reduced)?;
        self.reduced_cache = Some(model);
        self.stats.record(STAGE_REDUCED, StageReuse::Rebuilt);
        Ok(())
    }

    pub fn get_or_build_reduced(&mut self) -> Result<ReducedSemanticModel> {
        self.ensure_reduced()?;
        self.cached_reduced()
    }

    // ------------------------------------------------------------------
    // Call graph stage
    // ------------------------------------------------------------------

    pub fn ensure_callgraph(&mut self) -> Result<()> {
        if self.callgraph_cache.is_some() {
            return Ok(());
        }

        if self
            .manager
            .stage_is_fresh("callgraph", &self.fps.callgraph)
        {
            if let Ok(loaded) = self.manager.load_callgraph_checked(&self.fps.callgraph) {
                self.callgraph_cache = Some(loaded);
                self.stats.record(STAGE_CALLGRAPH, StageReuse::Loaded);
                return Ok(());
            }
        }

        self.ensure_index()?;
        self.ensure_reduced()?;
        let index = self
            .index_cache
            .as_ref()
            .context("index stage cache must exist after ensure_index")?;
        let reduced = self
            .reduced_cache
            .as_ref()
            .context("reduced stage cache must exist after ensure_reduced")?;
        let callgraph = CallGraph::build_from_index(index, reduced);
        self.manager
            .save_callgraph_artifacts(&callgraph, &self.fps.callgraph)?;
        self.callgraph_cache = Some(callgraph);
        self.stats.record(STAGE_CALLGRAPH, StageReuse::Rebuilt);
        Ok(())
    }

    pub fn get_or_build_callgraph(&mut self) -> Result<CallGraph> {
        self.ensure_callgraph()?;
        self.cached_callgraph()
    }

    // ------------------------------------------------------------------
    // Load stage
    // ------------------------------------------------------------------

    pub fn ensure_load(&mut self) -> Result<()> {
        if self.load_cache.is_some() {
            return Ok(());
        }

        if self.manager.stage_is_fresh("load", &self.fps.load) {
            if let Ok(loaded) = self.manager.load_load_checked(&self.fps.load) {
                self.load_cache = Some(Arc::new(loaded));
                self.stats.record(STAGE_LOAD, StageReuse::Loaded);
                return Ok(());
            }
        }

        self.ensure_include_graph()?;
        self.ensure_reduced()?;
        let (graph, closures) = self.cached_include_graph()?;
        let reduced = self.cached_reduced()?;

        // Phase 4 (plans/LoadPerfImprovements): both analysis loops are
        // CPU-bound and independent per item, so they run under rayon with
        // `par_iter().filter_map().collect()`, which preserves the
        // sequential item order element-for-element. ONE memoized
        // `LoadAnalyzer` is shared via `&` across all workers (it is
        // `Send + Sync` — compile-asserted in the graph crate) so the
        // append-only semantic-closure memo keeps warming across seeds;
        // building an analyzer per item would lose that cross-seed reuse.
        //
        // Deliberate deviation from the plan: there is NO cargo feature
        // flag for parallel vs sequential. A flag would ship a default
        // path that no CI run ever exercises; instead the `workers` knob
        // (already the parallelism control for the index stage) selects
        // the pool size, and the test
        // `test_load_report_matches_sequential_reference` locks the
        // parallel output against an in-test sequential reference.
        let analyzer = LoadAnalyzer::with_closures(&graph, &reduced, closures);
        let total_files = reduced.file_symbols.len();
        let repository_root = self.repo.root.clone();
        // take/restore the callback around the parallel section, mirroring
        // `rebuild_index_single_threaded`; the Mutex lets every worker share
        // one `FnMut` sink and guards are only held for a single event.
        let progress = Mutex::new(self.progress.take());
        let completed_files = AtomicUsize::new(0);

        let report = self.run_on_configured_pool(|| LoadReport {
            repository_root,
            symbol_metrics: collect_symbol_metrics(&analyzer, &reduced),
            file_metrics: collect_file_metrics(
                &analyzer,
                &graph,
                &reduced,
                total_files,
                &progress,
                &completed_files,
            ),
        })?;

        self.progress = progress.into_inner().unwrap_or_else(|err| err.into_inner());
        self.notify(&ProgressEvent::CollateralFile {
            done: total_files,
            total: total_files,
        });

        self.manager.save_load_artifacts(&report, &self.fps.load)?;
        self.load_cache = Some(Arc::new(report));
        self.stats.record(STAGE_LOAD, StageReuse::Rebuilt);
        Ok(())
    }

    pub fn get_or_build_load(&mut self) -> Result<LoadReport> {
        self.ensure_load()?;
        self.cached_load()
    }

    /// Runs `work` on a rayon-managed thread set honoring the `workers`
    /// knob: `Some(n)` builds a scoped pool of exactly `n` threads and
    /// `install`s the work into it; `None` runs it inline, letting the
    /// `par_iter` calls inside use the global pool. Same knob the index
    /// stage already uses — the deliberate no-feature-flag design is
    /// documented at the call site in `ensure_load`.
    fn run_on_configured_pool<T, F>(&self, work: F) -> Result<T>
    where
        T: Send,
        F: FnOnce() -> T + Send,
    {
        match self.workers {
            Some(num_threads) => {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(num_threads.max(1))
                    .build()
                    .map_err(|err| anyhow!("failed to build rayon thread pool: {err}"))?;
                Ok(pool.install(work))
            }
            None => Ok(work()),
        }
    }

    fn cached_include_graph(
        &self,
    ) -> Result<(IncludeGraph, HashMap<ariadne_repo::FileId, IncludeClosure>)> {
        self.include_cache
            .clone()
            .context("include stage cache must exist after ensure_include_graph")
    }

    fn cached_index(&self) -> Result<IndexResult> {
        self.index_cache
            .clone()
            .context("index stage cache must exist after ensure_index")
    }

    fn cached_reduced(&self) -> Result<ReducedSemanticModel> {
        self.reduced_cache
            .clone()
            .context("reduced stage cache must exist after ensure_reduced")
    }

    fn cached_callgraph(&self) -> Result<CallGraph> {
        self.callgraph_cache
            .clone()
            .context("callgraph stage cache must exist after ensure_callgraph")
    }

    fn cached_load(&self) -> Result<LoadReport> {
        let report = self
            .load_cache
            .as_ref()
            .context("load stage cache must exist after ensure_load")?;
        Ok(Arc::unwrap_or_clone(Arc::clone(report)))
    }
}

/// Parallel, order-preserving symbol metric pass for `ensure_load`. The
/// BTreeMap `.values()` view is materialized into a `Vec` first (b-tree
/// value iterators are not rayon parallel iterators); rayon's
/// `par_iter().filter_map().collect()` then yields the same vector — and
/// the same order — as the old sequential loop.
fn collect_symbol_metrics(
    analyzer: &LoadAnalyzer<'_>,
    reduced: &ReducedSemanticModel,
) -> Vec<LoadMetric> {
    let symbol_names: Vec<String> = reduced
        .symbols
        .values()
        .map(|record| record.name.clone())
        .collect();
    symbol_names
        .par_iter()
        .filter_map(|name| analyzer.analyze_symbol_load(name).ok())
        .collect()
}

/// Parallel, order-preserving file metric pass for `ensure_load`. Each
/// worker emits one `CollateralFile` progress event after its file
/// completes; `done` comes from `AtomicUsize::fetch_add`, so the values
/// stay unique (0..total-1) while event arrival may interleave — the UI
/// only reads done/total. The `done == total` completion event is the
/// caller's. `completed` is incremented once per file id (matching the
/// sequential `enumerate`), even when the id has no registered path.
fn collect_file_metrics(
    analyzer: &LoadAnalyzer<'_>,
    graph: &IncludeGraph,
    reduced: &ReducedSemanticModel,
    total_files: usize,
    progress: &Mutex<Option<ProgressCb>>,
    completed: &AtomicUsize,
) -> Vec<LoadMetric> {
    let file_ids: Vec<FileId> = reduced.file_symbols.keys().copied().collect();
    file_ids
        .par_iter()
        .map(|&file_id| {
            // By-id analysis drops the per-file O(V) `find_file_id` string
            // scan the old path round-trip paid (plan LoadPerf A2). The
            // registration check preserves the old skip contract: an id
            // without a path yields None and disappears via `flatten()`.
            let metric = if graph.file_paths.contains_key(&file_id) {
                analyzer.analyze_file_load_by_id(file_id).ok()
            } else {
                None
            };
            let done = completed.fetch_add(1, Ordering::SeqCst);
            if let Some(cb) = progress.lock().unwrap().as_mut() {
                cb(&ProgressEvent::CollateralFile {
                    done,
                    total: total_files,
                });
            }
            metric
        })
        .flatten()
        .collect()
}

#[cfg(test)]
#[path = "pipeline_tests.rs"]
mod pipeline_tests;
