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
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult};
use ariadne_graph::{
    build_dependency_graph, CallGraph, IncludeClosure, IncludeGraph, ReducedSemanticModel,
};
use ariadne_index::{IndexResult, StageFingerprints};
use ariadne_lang::{CyclePolicy, LanguageId, LanguageRegistry};
use ariadne_repo::{FileId, Repository, RepositoryDiscoveryResult, SourceLanguage};
use ariadne_report::{ArtifactManager, LoadReport};
use ariadne_scheduler::{ProductionAnalysisStats, SharedTuStoreContext, TuStoreContext};

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

// Stage modules extracted VERBATIM from this file (boot-progress-visibility
// V3a, hygiene sec. 9: this file sat AT the 1000-line ceiling). The index
// and load stages are `impl Pipeline` blocks in these modules; behavior
// zero-change. M27 T3/T4 add INTO these modules, not `lib.rs`.
mod stage_index;
mod stage_load;

// A6 (AR-22): the `for_incremental` construction seam (an `impl Pipeline`
// block, the `stage_index`/`stage_load` precedent) plus the unit-restriction
// + artifact-authority state it records.
mod for_incremental;
pub use for_incremental::IncrementalView;

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
    /// boot-progress-visibility V3 (stage 1): boot milestones at/before the
    /// include stage - discovery hashing and the cold include-scan substeps.
    BootMilestone(BootMilestone),
    /// boot-progress-visibility V3 (stage 2, I10/R7): fired once per TU as
    /// its analysis starts, carrying the repo-relative source path so the
    /// boot row names the current unit. Replaces the per-TU `TuIndex`
    /// event; the terminal `TuIndex { done: total, total }` stays.
    TuStarted {
        done: usize,
        total: usize,
        source: std::path::PathBuf,
    },
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

/// Boot milestones of stage 1 (boot-progress-visibility V3, shape sec. 3.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootMilestone {
    /// Input-computation hashing progressed; total None while the
    /// single-pass walk has not bounded the file set.
    DiscoveryHashed { done: usize, total: Option<usize> },
    /// Include-scan progress of the [1/5] cold rebuild.
    IncludeScan { done: usize, total: usize },
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
    /// A6 (AR-22): the unit-restriction + artifact-authority state carried by
    /// [`Pipeline::for_incremental`] (the `analyze.rs` incremental mirror's
    /// `restrict_and_validate_units` + `artifacts_are_authoritative` data).
    /// `None` for every other constructor - their runs carry no incremental
    /// view. The public surface a parity test locks.
    pub incremental_view: Option<IncrementalView>,
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
        Self::build_with_progress(repo_path, workers, registry, None)
    }

    /// `build` with the construction-time callback threaded into input
    /// computation (boot-progress-visibility V3 construction seam): the repo
    /// hook can emit `BootMilestone::DiscoveryHashed` while hashing. `None`
    /// keeps the exact pre-V3 sequence - the behavioral no-op the G1-5
    /// exemption rests on (locked by `boot_events_test.rs` test 1).
    fn build_with_progress(
        repo_path: &Path,
        workers: Option<usize>,
        registry: Arc<LanguageRegistry>,
        mut progress: Option<ProgressCb>,
    ) -> Result<Self> {
        let (repo, discovery, build, fps) =
            Self::compute_inputs_with_progress(repo_path, &registry, progress.as_mut())?;
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
            progress,
            registry,
            language_filter: None,
            incremental_view: None,
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

    /// Pipeline whose CONSTRUCTION already reports through `progress`
    /// (boot-progress-visibility V3 construction seam).
    pub fn new_with_progress(
        repo_path: &Path,
        workers: Option<usize>,
        progress: Box<dyn FnMut(&ProgressEvent) + Send>,
    ) -> Result<Self> {
        Self::build_with_progress(repo_path, workers, default_registry(), Some(progress))
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
        Self::compute_inputs_with_progress(repo_path, registry, None)
    }

    /// `compute_inputs` with the boot callback threaded into the repo hook
    /// (boot-progress-visibility V3): `Some` routes discovery through
    /// `Repository::discover_with_progress` and emits one
    /// `BootMilestone::DiscoveryHashed { done, total: None }` per hashed
    /// file - the single-pass walk has no bounded total, so `total` is
    /// always `None` on this path (enum doc). `None` runs the identical
    /// pre-V3 sequence (`repo.discover()` is already V2's no-op delegation
    /// of the same walk), so the no-callback path executes zero new code.
    fn compute_inputs_with_progress(
        repo_path: &Path,
        registry: &LanguageRegistry,
        mut progress: Option<&mut ProgressCb>,
    ) -> Result<(
        Repository,
        RepositoryDiscoveryResult,
        BuildDiscoveryResult,
        StageFingerprints,
    )> {
        let repo = Repository::locate(repo_path)?;
        let discovery = match progress.as_mut() {
            None => repo.discover()?,
            Some(cb) => repo.discover_with_progress(&mut |hashed, _discovered| {
                cb(&ProgressEvent::BootMilestone(BootMilestone::DiscoveryHashed {
                    done: hashed,
                    total: None,
                }))
            })?,
        };
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

        // V3 boot milestones: one honest notify per real substep, cold
        // rebuild branch only - the warm-load branch above returned early
        // and emits nothing (warm runs already jump honestly). There is no
        // per-file loop at this seam (the BFS lives in `crates/graph`,
        // which V3 must not touch), so substeps are the honest granularity.
        let graph = build_dependency_graph(&self.repo.root, &self.registry)?;
        self.notify(&ProgressEvent::BootMilestone(BootMilestone::IncludeScan {
            done: 1,
            total: 4,
        }));
        let closures = graph.compute_closures();
        self.notify(&ProgressEvent::BootMilestone(BootMilestone::IncludeScan {
            done: 2,
            total: 4,
        }));
        let cycle_policies = cycle_policy_by_file(&self.registry, &self.discovery);
        let report = graph.generate_report_with_cycle_policies(&|file_id| {
            *cycle_policies
                .get(&file_id)
                .unwrap_or(&CyclePolicy::Reject)
        });
        self.notify(&ProgressEvent::BootMilestone(BootMilestone::IncludeScan {
            done: 3,
            total: 4,
        }));
        self.manager
            .save_include_artifacts(&graph, &closures, &report, &self.fps.include_graph)?;
        self.notify(&ProgressEvent::BootMilestone(BootMilestone::IncludeScan {
            done: 4,
            total: 4,
        }));
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

    /// M28 T2 (a) seam: MOVE the three stage-5 inputs (include graph,
    /// include closures, reduced model) out of the stage caches so
    /// `ensure_load` runs on the single owned copy of each. The pre-M28-T2
    /// path deep-cloned the graph/closures pair and the reduced model at
    /// `stage_load.rs:43-44` while the caches still held the originals
    /// (A4 double residency); this seam replaces both clones.
    ///
    /// SAFETY PRECONDITION: `restore_stage5_inputs` must run before
    /// `ensure_load` returns - boot's `take_include_graph` after stage 5
    /// (`ui.rs:250`) and the warm-reuse tests read these caches after the
    /// load stage. Same documented missing-cache errors as the clone
    /// accessors when a stage has not run.
    pub(crate) fn take_graph_and_reduced_for_load(
        &mut self,
    ) -> Result<(
        IncludeGraph,
        HashMap<ariadne_repo::FileId, IncludeClosure>,
        ReducedSemanticModel,
    )> {
        let (graph, closures) = self
            .include_cache
            .take()
            .context("include stage cache must exist after ensure_include_graph")?;
        let reduced = self
            .reduced_cache
            .take()
            .context("reduced stage cache must exist after ensure_reduced")?;
        Ok((graph, closures, reduced))
    }

    /// Inverse of `take_graph_and_reduced_for_load`: the taken tables go
    /// back into the stage caches by move (no deep copy) before
    /// `ensure_load` returns, keeping the post-stage-5 state contract that
    /// boot's `take_include_graph` and the warm-reuse tests depend on.
    pub(crate) fn restore_stage5_inputs(
        &mut self,
        graph: IncludeGraph,
        closures: HashMap<ariadne_repo::FileId, IncludeClosure>,
        reduced: ReducedSemanticModel,
    ) {
        self.include_cache = Some((graph, closures));
        self.reduced_cache = Some(reduced);
    }

    /// M28 T2 (b): drop the index result before the stage-5 span runs (A5).
    /// Stage 5 reads include-graph and reduced-model data only
    /// (`stage_load.rs`), so the full `IndexResult` sitting through the
    /// passes and the save span is dead weight at corpus scale.
    ///
    /// RELOAD CONTRACT: after a release, `ensure_index` takes the
    /// fingerprint-fresh disk branch - the index artifacts were saved (or
    /// verified fresh) by the earlier stage-2 run - so every later
    /// consumer (`ensure_callgraph`, `get_or_build_index`) behaves exactly
    /// as a fresh pipeline's warm stage 2, only paying the documented
    /// reload. Boot runs the callgraph BEFORE the load stage and pays zero
    /// reloads. A `--lang`-filtered run has no persisted full-index
    /// artifact; its consumers rebuild through the shard store
    /// (TU-level reuse intact) - correct results, extra CPU on that
    /// filtered-view edge only (T2 journal D6).
    pub(crate) fn release_index_result_for_load(&mut self) {
        self.index_cache = None;
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

// The full cold/warm pipeline suite moved to `crates/pipeline/tests/
// pipeline_test.rs` (M23 T3, D-M23-10): its fixtures embed C++ compile-command
// and directive text, so they live outside the guard walk scope. Tests that
// need the private collection internals stay in `internal_tests.rs`.
//
// Test-only re-export: `internal_tests` reaches the moved
// `collect_file_metrics` through the crate-root glob (`use super::*`),
// exactly as it did before the stage modules were split out (V3a). The
// cfg(test) imports below are the companion set: the same glob used to
// inherit them from the pre-split production import block; they stay
// test-gated so the production build keeps a zero unused-import surface.
#[cfg(test)]
use ariadne_graph::{LoadAnalyzer, LoadMetric};
#[cfg(test)]
use stage_load::collect_file_metrics;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
mod internal_tests;

#[cfg(test)]
mod stage_load_tests;
