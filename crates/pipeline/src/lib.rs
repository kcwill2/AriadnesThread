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

use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult, TranslationUnit};
use ariadne_graph::{
    build_dependency_graph, CallGraph, IncludeClosure, IncludeGraph, LoadAnalyzer, LoadMetric,
    ReducedSemanticModel,
};
use ariadne_index::{
    IncrementalStats, IndexResult, StageFingerprints, TuAnalysis, CURRENT_ANALYSIS_VERSION,
};
use ariadne_lang::{CyclePolicy, LanguageId, LanguagePlugin, LanguageRegistry, UnitShaper};
use ariadne_repo::{FileId, Repository, RepositoryDiscoveryResult, SourceLanguage};
use ariadne_report::{ArtifactManager, LoadReport};
use ariadne_scheduler::{
    analyze_closure_with, compute_tu_closure_plugin, load_shard, save_shard, shard_dir,
    ProductionAnalysisStats, ProductionAnalyzer, ShardCheckpoint, SharedPluginWorkerBackend,
    SharedTuStoreContext, TuClosure, TuStoreContext,
};

pub const STAGE_INCLUDE_GRAPH: &str = "include-graph";
pub const STAGE_INDEX: &str = "index";
pub const STAGE_REDUCED: &str = "reduced";
pub const STAGE_CALLGRAPH: &str = "callgraph";
pub const STAGE_LOAD: &str = "load";

// The plugin enumeration moved to `registry.rs` (M23 T3, D-M23-10): the R2
// architecture guard walks this crate's `src`, and `registry.rs` is the one
// NAMED composition-root file it allowlists for plugin naming. Re-exports
// keep `pipeline::default_registry` / `pipeline::build_default_registry`
// callers unchanged (cli delegation contract, D9).
mod registry;
pub use registry::{build_default_registry, default_registry};

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
    /// A4: named checkpoints inside the load stage that no per-item event
    /// covers — the symbol pass boundary and the artifact-save completion.
    LoadMilestone(LoadMilestone),
}

/// Load-stage milestones (plan load-tail A4). `SymbolsAnalyzed` fires once
/// the symbol pass has fully collected (before any `CollateralFile` event);
/// `ArtifactsSaved` fires only after `save_load_artifacts` has returned, so
/// progress consumers never observe the save as complete before the bytes
/// are on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadMilestone {
    SymbolsAnalyzed { symbols: usize },
    ArtifactsSaved,
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
    store_ctx: SharedTuStoreContext,
    workers: Option<usize>,
    progress: Option<ProgressCb>,
    /// Language plugins driving unit shaping, closure walks, analysis, and
    /// worker dispatch (plan M18.3; required since M18.11 — the transitional
    /// hardcoded-C++ paths are removed).
    registry: Arc<LanguageRegistry>,
    /// `--lang` restriction: the index rebuild keeps only units whose plugin
    /// the registry detects for the unit path. Registration is validated by
    /// the construction site; the index stage is the only consumer.
    language_filter: Option<SourceLanguage>,
    include_cache: Option<(IncludeGraph, HashMap<ariadne_repo::FileId, IncludeClosure>)>,
    index_cache: Option<IndexResult>,
    reduced_cache: Option<ReducedSemanticModel>,
    callgraph_cache: Option<CallGraph>,
    load_cache: Option<Arc<LoadReport>>,
}

impl Pipeline {
    /// Pipeline over the shared default plugin set ([`build_default_registry`],
    /// D9). The library-level entry point stays plugin-dispatched end to
    /// end (M18.11 removed the hardcoded-C++ fallbacks - a pipeline without
    /// plugins cannot shape, walk, or analyze units at all); the C++-first
    /// registration order plus Scala's `false` capability literals keep C++
    /// output byte-identical (locked by the m00 goldens and
    /// `m18_cpp_plugin_parity_test`).
    pub fn new(repo_path: &Path, workers: Option<usize>) -> Result<Self> {
        Self::build(repo_path, workers, default_registry())
    }

    /// Pipeline whose index stage runs through the caller's plugin registry
    /// (plan M18.3): registry-driven unit shaping, plugin-resolver closure
    /// walks, plugin-analyzer merge, and registry-dispatching worker backends.
    /// C++ output is byte-identical to the C++-default [`Pipeline::new`]
    /// (locked by the m00 goldens and `m18_cpp_plugin_parity_test`). A unit
    /// whose language no registered plugin claims is reported as "unhandled
    /// language" (§3.3 fact 3), never silently parsed as C++.
    pub fn with_registry(
        repo_path: &Path,
        workers: Option<usize>,
        registry: Arc<LanguageRegistry>,
    ) -> Result<Self> {
        Self::build(repo_path, workers, registry)
    }

    /// Restrict the index stage to units of one language (`--lang`, plan
    /// M18.3). Callers must have validated the language's plugin is registered.
    pub fn language_filter(mut self, language: Option<SourceLanguage>) -> Self {
        self.language_filter = language;
        self
    }

    fn build(
        repo_path: &Path,
        workers: Option<usize>,
        registry: Arc<LanguageRegistry>,
    ) -> Result<Self> {
        let (repo, discovery, build, fps) = Self::compute_inputs(repo_path, &registry)?;
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
            registry,
            language_filter: None,
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
    /// identically by every path that derives fingerprints. The unit shaper
    /// comes from the registry's marker-based selection against the located
    /// repository root (plan M22 D22; first-wins fallback preserved, so
    /// every marker-free repository shapes exactly as before).
    fn compute_inputs(
        repo_path: &Path,
        registry: &LanguageRegistry,
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
            .with_unit_shaper(registry.unit_shaper_for_repo(&repo.root))
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
    /// freshness checks (e.g. the UI watcher's no-op tick detection). Runs
    /// over the same default plugin set [`Pipeline::new`] uses, so the chain
    /// matches what that pipeline derives.
    pub fn rehash_inputs(repo_path: &Path) -> Result<StageFingerprints> {
        Ok(Self::compute_inputs(repo_path, &default_registry())?.3)
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

        let graph = build_dependency_graph(&self.repo.root, &self.registry)?;
        let closures = graph.compute_closures();
        let cycle_policies = cycle_policy_by_file(&self.registry, &self.discovery);
        let report = graph.generate_report_with_cycle_policies(&|file_id| {
            *cycle_policies
                .get(&file_id)
                .unwrap_or(&CyclePolicy::Reject)
        });
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

    /// MOVE the `(graph, closures)` pair out of the include cache — the
    /// zero-copy hand-out for consumers that take ownership of the whole
    /// stage output (UI payload assembly, plan load-tail A3). Returns the
    /// same documented missing-cache error as `cached_include_graph` when
    /// the stage has not run or was already taken.
    ///
    /// SAFETY PRECONDITION — nothing downstream may read `include_cache`
    /// after a take. The only user, `build_ui_payload`, satisfies this:
    /// every stage that consumes the include graph (`ensure_load` reads the
    /// cache at `Self::cached_include_graph`) runs via `run_pipeline_stages` BEFORE
    /// the take, and the moved graph/closures are used locally afterwards.
    /// The watcher refresh is safe because each refresh constructs a FRESH
    /// `Pipeline` (`spawn_watcher` -> `build_ui_payload` -> `Pipeline::new`).
    pub fn take_include_graph(
        &mut self,
    ) -> Result<(IncludeGraph, HashMap<ariadne_repo::FileId, IncludeClosure>)> {
        self.include_cache
            .take()
            .context("include stage cache must exist after ensure_include_graph")
    }

    // ------------------------------------------------------------------
    // Index stage (TU-level reuse via the shard store)
    // ------------------------------------------------------------------

    pub fn ensure_index(&mut self) -> Result<()> {
        if self.index_cache.is_some() {
            return Ok(());
        }

        if self.manager.stage_is_fresh("index", &self.fps.index) && self.language_filter.is_none() {
            if let Ok(loaded) = self.manager.load_index_checked(&self.fps.index) {
                self.index_cache = Some(loaded);
                self.stats.record(STAGE_INDEX, StageReuse::Loaded);
                return Ok(());
            }
        }

        let indexed_build = self.indexed_build()?;
        let (index_result, stats, production_stats) = match self.workers {
            Some(workers) => {
                // M18.11: the production path always runs the registry-backed
                // plugin backend and the plugin-driven shard prepass.
                let analyzer =
                    ProductionAnalyzer::new(&self.repo, &self.discovery, &indexed_build)
                        .with_workers(workers)
                        .with_backend(Arc::new(SharedPluginWorkerBackend::new(Arc::clone(
                            &self.registry,
                        ))))
                        .with_registry(Arc::clone(&self.registry));
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
                let (index_result, stats) = self.rebuild_index_single_threaded(&indexed_build)?;
                (index_result, stats, None)
            }
        };

        self.production_stats = production_stats;

        // A `--lang` restriction is a filtered view, never the authoritative
        // full-stage artifact; saving it under the full index fingerprint would
        // poison every later full run, so only the unfiltered result persists.
        if self.language_filter.is_none() {
            self.manager
                .save_index_artifacts(&index_result, &self.fps.index, Some(&stats))?;
        }
        self.stats.reused_tus = stats.reused_tus;
        self.stats.reanalyzed_tus = stats.reanalyzed_tus;
        self.index_cache = Some(index_result);
        self.stats.record(STAGE_INDEX, StageReuse::Rebuilt);
        Ok(())
    }

    /// The build result the index stage analyzes: full, or restricted to the
    /// `--lang` language (plan M18.3). Contexts are kept intact either way —
    /// closures resolve per TU context exactly as in a full run.
    ///
    /// A filtered run re-shapes through the filter's plugin shaper (D15):
    /// construction-time discovery ran the registry's marker-based selection
    /// (D22), which may recognize none of the filtered language's build
    /// markers at all (a repo shaped by another language's shaper yields
    /// zero units), so retention alone could never surface units of the
    /// filtered language.
    /// The unfiltered path returns the construction-time result unchanged —
    /// `select_unit_shaper`'s `None` branch is that same marker-based call,
    /// so `compute_inputs`' discovery and every derived fingerprint are
    /// exactly what the unfiltered run analyzes.
    fn indexed_build(&mut self) -> Result<BuildDiscoveryResult> {
        let mut build = match self.language_filter {
            None => self.build.clone(),
            Some(_) => {
                let reshaped = BuildDiscoverer::new(&self.repo.root)
                    .with_repo_discovery(&self.discovery)
                    .with_unit_shaper(select_unit_shaper(
                        &self.registry,
                        self.language_filter,
                        &self.repo.root,
                    ))
                    .discover()?;
                // The shard store context must see the contexts the shaping
                // named, exactly as construction-time shaping provided them
                // (context content is part of every TU cache fingerprint).
                self.store_ctx =
                    TuStoreContext::new(&self.discovery, reshaped.contexts.clone());
                reshaped
            }
        };
        if let Some(language) = self.language_filter {
            // M24 Part A: the `--lang` filter arrives as the closed file-kind
            // label, while detection answers in the open identity, so the
            // comparison goes through the one direction that exists.
            let selected = LanguageId::from(language);
            build
                .translation_units
                .retain(|tu| self.registry.detect_language(&tu.source_path).as_ref() == Some(&selected));
        }
        Ok(build)
    }

    fn rebuild_index_single_threaded(
        &mut self,
        build: &BuildDiscoveryResult,
    ) -> Result<(IndexResult, IncrementalStats)> {
        let store_dir = shard_dir(&self.repo.root);
        let store_ctx = self.store_ctx.clone();
        let repo_root = self.repo.root.clone();
        let mut progress = self.progress.take();
        let mut notify = |event: &ProgressEvent| {
            if let Some(cb) = progress.as_mut() {
                cb(event);
            }
        };

        let mut reused = 0usize;
        let mut reanalyzed = 0usize;
        let mut analyses = Vec::new();

        let total = build.translation_units.len();
        for (i, tu) in build.translation_units.iter().enumerate() {
            notify(&ProgressEvent::TuIndex { done: i, total });

            let (analysis, was_reused) = index_unit_with_plugin(
                tu,
                &repo_root,
                &store_ctx,
                &store_dir,
                plugin_for_path(&self.registry, &tu.source_path)?,
            )?;
            if was_reused {
                reused += 1;
            } else {
                reanalyzed += 1;
            }
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
        // A4: the two analysis passes run as separate pool sections so the
        // symbol-pass boundary can notify through `self.notify` while the
        // callback is still on `self`; the file pass then takes the callback
        // into a Mutex for the rayon workers, mirroring
        // `rebuild_index_single_threaded`. Field order of the old `LoadReport`
        // struct literal already guaranteed symbols-before-files; the split
        // keeps that order and makes it a notifiable milestone.
        let symbol_metrics =
            self.run_on_configured_pool(|| collect_symbol_metrics(&analyzer, &reduced))?;
        self.notify(&ProgressEvent::LoadMilestone(LoadMilestone::SymbolsAnalyzed {
            symbols: symbol_metrics.len(),
        }));

        // take/restore the callback around the parallel section, mirroring
        // `rebuild_index_single_threaded`; the Mutex lets every worker share
        // one `FnMut` sink and guards are only held for a single event.
        let progress = Mutex::new(self.progress.take());
        let completed_files = AtomicUsize::new(0);
        let file_metrics = self.run_on_configured_pool(|| {
            collect_file_metrics(
                &analyzer,
                &graph,
                &reduced,
                total_files,
                &progress,
                &completed_files,
            )
        })?;

        self.progress = progress.into_inner().unwrap_or_else(|err| err.into_inner());
        self.notify(&ProgressEvent::CollateralFile {
            done: total_files,
            total: total_files,
        });

        let report = LoadReport {
            repository_root,
            symbol_metrics,
            file_metrics,
        };
        // M22 D25: the load report itself carries no language data, so the
        // stage feeds the manifest from discovery records; an all-default
        // repository still writes byte-identical pre-M22 manifest bytes.
        let stage_languages: Vec<SourceLanguage> =
            self.discovery.files.iter().map(|f| f.language).collect();
        self.manager.save_load_artifacts_with_languages(
            &report,
            &self.fps.load,
            &stage_languages,
        )?;
        self.notify(&ProgressEvent::LoadMilestone(LoadMilestone::ArtifactsSaved));
        self.load_cache = Some(Arc::new(report));
        self.stats.record(STAGE_LOAD, StageReuse::Rebuilt);
        Ok(())
    }

    /// A3: hands out a shared `Arc<LoadReport>` handle, not a copy.
    pub fn get_or_build_load(&mut self) -> Result<Arc<LoadReport>> {
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

    fn cached_load(&self) -> Result<Arc<LoadReport>> {
        // A3: hand out a shared `Arc` handle (refcount bump, no deep copy).
        // The old `Arc::unwrap_or_clone` copied the whole report per call
        // (~1 s on the UI boot fixture, plan load-tail A0).
        self.load_cache
            .clone()
            .context("load stage cache must exist after ensure_load")
    }
}

/// Unit shaper for the index stage's unit shaping (D15 + D22): with an
/// explicit `--lang` filter the shaper comes from that language's plugin — a
/// plugin that provides no shaper honestly stays `None`, leaving build
/// discovery neutral instead of guessing another language's defaults;
/// without a filter the registry's marker-based selection against the
/// repository root applies (the same call `compute_inputs` made at
/// construction). Registry lookups only — no language-identity branching
/// (R3).
fn select_unit_shaper<'a>(
    registry: &'a LanguageRegistry,
    language_filter: Option<SourceLanguage>,
    repo_root: &Path,
) -> Option<&'a dyn UnitShaper> {
    match language_filter {
        Some(language) => registry
            .plugin_for_id(LanguageId::from(language))
            .and_then(|plugin| plugin.shaper()),
        None => registry.unit_shaper_for_repo(repo_root),
    }
}

/// Per-file cycle stance for the include-graph report (plan M20 D11, IV-5):
/// each discovery record's `language` is resolved through the registry
/// (`plugin_for_id` — R3-legal registry lookup, no `match LanguageId`), and
/// files no registered plugin claims fall back to `Reject`, which keeps the
/// C++ report byte-identical to the pre-M20 `generate_report()` output.
fn cycle_policy_by_file(
    registry: &LanguageRegistry,
    discovery: &RepositoryDiscoveryResult,
) -> HashMap<FileId, CyclePolicy> {
    discovery
        .files
        .iter()
        .map(|record| {
            let policy = registry
                .plugin_for_id(LanguageId::from(record.language))
                .map(|plugin| plugin.modeling().cycle_policy)
                .unwrap_or(CyclePolicy::Reject);
            (record.id, policy)
        })
        .collect()
}

/// Plugin that owns `path`'s language (plan §3.3 fact 3): the registry
/// detection is the dispatch key, and an unclaimed path is reported as
/// "unhandled language" rather than silently parsed with some other plugin.
fn plugin_for_path<'a>(
    registry: &'a LanguageRegistry,
    path: &Path,
) -> Result<&'a dyn LanguagePlugin> {
    let language = registry.detect_language(path).ok_or_else(|| {
        anyhow!(
            "unhandled language for {} (no registered plugin claims it)",
            path.display()
        )
    })?;
    registry
        .plugin_for_id(language.clone())
        .ok_or_else(|| anyhow!("language plugin not registered: {language:?}"))
}

/// Shard-backed analysis of one unit through the plugin resolver/analyzer
/// (plan M18.3, the only path since M18.11). `compute_tu_closure_plugin`
/// reproduces the canonical closure file order byte-for-byte (locked by the
/// vendored oracles in `m18_cpp_plugin_parity_test`), so shard envelopes and
/// merged analyses are identical to the pre-M11 hardcoded-C++ path.
fn index_unit_with_plugin(
    tu: &TranslationUnit,
    repo_root: &Path,
    store_ctx: &SharedTuStoreContext,
    store_dir: &Path,
    plugin: &dyn LanguagePlugin,
) -> Result<(TuAnalysis, bool)> {
    let closure = compute_tu_closure_plugin(
        tu,
        repo_root,
        store_ctx,
        CURRENT_ANALYSIS_VERSION,
        plugin.resolver(),
        // M18.10 fingerprint identity: `LanguageId` IS `SourceLanguage`
        // (plan §3.2), and for C++ these two values equal the constants the
        // pre-M11 walk hardcoded — keeping shard envelopes byte-identical.
        plugin.id(),
        &plugin.analysis_fingerprint(),
    )?;
    sharded_analysis(tu, &closure, store_dir, || {
        analyze_closure_with(tu, &closure, plugin.analyzer())
    })
}

/// Shared shard envelope contract of both unit paths: reuse the checkpoint
/// when the closure fingerprint still matches, else analyze and re-stamp.
fn sharded_analysis(
    tu: &TranslationUnit,
    closure: &TuClosure,
    store_dir: &Path,
    analyze: impl FnOnce() -> TuAnalysis,
) -> Result<(TuAnalysis, bool)> {
    if let Some(shard) = load_shard(store_dir, tu.file_id) {
        if shard.fingerprint == closure.fingerprint.combined_hash {
            return Ok((shard.analysis, true));
        }
    }
    let analysis = analyze();
    save_shard(
        store_dir,
        &ShardCheckpoint {
            fingerprint: closure.fingerprint.combined_hash.clone(),
            analysis: analysis.clone(),
        },
    )?;
    Ok((analysis, false))
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

// The full cold/warm pipeline suite moved to `crates/pipeline/tests/
// pipeline_test.rs` (M23 T3, D-M23-10): its fixtures embed C++ compile-command
// and directive text, so they live outside the guard walk scope. Tests that
// need the private collection internals stay in `internal_tests.rs`.
#[cfg(test)]
mod internal_tests;
