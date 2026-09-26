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

use anyhow::Result;
use serde::{Deserialize, Serialize};

use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult};
use ariadne_graph::{
    build_include_graph, CallGraph, IncludeClosure, IncludeGraph, LoadAnalyzer,
    ReducedSemanticModel,
};
use ariadne_index::{
    IncrementalStats, IndexResult, StageFingerprints, CURRENT_ANALYSIS_VERSION,
};
use ariadne_repo::{Repository, RepositoryDiscoveryResult};
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
        self.stages.iter().find(|s| s.stage == stage).map(|s| s.reuse)
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
            if loaded.is_empty() { "none".into() } else { loaded.join(", ") },
            if rebuilt.is_empty() { "none".into() } else { rebuilt.join(", ") },
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
    load_cache: Option<LoadReport>,
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
            if let Ok(loaded) = self.manager.load_include_checked(&self.fps.include_graph, &self.discovery) {
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
        Ok(self.include_cache.clone().unwrap())
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
                let res = rt.block_on(analyzer.analyze())?;
                let production = res.stats;
                let reuse =
                    IncrementalStats::calculate(production.reused_tus, production.reanalyzed_tus);
                (res.index_result, reuse, Some(production))
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

    fn rebuild_index_single_threaded(
        &mut self,
    ) -> Result<(IndexResult, IncrementalStats)> {
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
        Ok((index_result, IncrementalStats::calculate(reused, reanalyzed)))
    }

    pub fn get_or_build_index(&mut self) -> Result<IndexResult> {
        self.ensure_index()?;
        Ok(self.index_cache.clone().unwrap())
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
        let model = ReducedSemanticModel::reduce(self.index_cache.as_ref().unwrap());
        self.manager.save_reduced_artifacts(&model, &self.fps.reduced)?;
        self.reduced_cache = Some(model);
        self.stats.record(STAGE_REDUCED, StageReuse::Rebuilt);
        Ok(())
    }

    pub fn get_or_build_reduced(&mut self) -> Result<ReducedSemanticModel> {
        self.ensure_reduced()?;
        Ok(self.reduced_cache.clone().unwrap())
    }

    // ------------------------------------------------------------------
    // Call graph stage
    // ------------------------------------------------------------------

    pub fn ensure_callgraph(&mut self) -> Result<()> {
        if self.callgraph_cache.is_some() {
            return Ok(());
        }

        if self.manager.stage_is_fresh("callgraph", &self.fps.callgraph) {
            if let Ok(loaded) = self.manager.load_callgraph_checked(&self.fps.callgraph) {
                self.callgraph_cache = Some(loaded);
                self.stats.record(STAGE_CALLGRAPH, StageReuse::Loaded);
                return Ok(());
            }
        }

        self.ensure_index()?;
        self.ensure_reduced()?;
        let callgraph = CallGraph::build_from_index(
            self.index_cache.as_ref().unwrap(),
            self.reduced_cache.as_ref().unwrap(),
        );
        self.manager
            .save_callgraph_artifacts(&callgraph, &self.fps.callgraph)?;
        self.callgraph_cache = Some(callgraph);
        self.stats.record(STAGE_CALLGRAPH, StageReuse::Rebuilt);
        Ok(())
    }

    pub fn get_or_build_callgraph(&mut self) -> Result<CallGraph> {
        self.ensure_callgraph()?;
        Ok(self.callgraph_cache.clone().unwrap())
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
                self.load_cache = Some(loaded);
                self.stats.record(STAGE_LOAD, StageReuse::Loaded);
                return Ok(());
            }
        }

        self.ensure_include_graph()?;
        self.ensure_reduced()?;
        let (graph, closures) = self.include_cache.clone().unwrap();
        let reduced = self.reduced_cache.clone().unwrap();

        let analyzer = LoadAnalyzer::with_closures(&graph, &reduced, closures);
        let mut symbol_metrics = Vec::new();
        for sym_record in reduced.symbols.values() {
            if let Ok(metric) = analyzer.analyze_symbol_load(&sym_record.name) {
                symbol_metrics.push(metric);
            }
        }

        let mut file_metrics = Vec::new();
        let total_files = reduced.file_symbols.len();
        for (i, file_id) in reduced.file_symbols.keys().enumerate() {
            self.notify(&ProgressEvent::CollateralFile {
                done: i,
                total: total_files,
            });
            if let Some(path) = graph.file_paths.get(file_id) {
                if let Ok(metric) = analyzer.analyze_file_load(path) {
                    file_metrics.push(metric);
                }
            }
        }
        self.notify(&ProgressEvent::CollateralFile {
            done: total_files,
            total: total_files,
        });

        let report = LoadReport {
            repository_root: self.repo.root.clone(),
            symbol_metrics,
            file_metrics,
        };
        self.manager.save_load_artifacts(&report, &self.fps.load)?;
        self.load_cache = Some(report.clone());
        self.stats.record(STAGE_LOAD, StageReuse::Rebuilt);
        Ok(())
    }

    pub fn get_or_build_load(&mut self) -> Result<LoadReport> {
        self.ensure_load()?;
        Ok(self.load_cache.clone().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("include")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
        fs::write(
            root.join("include/header_a.h"),
            "#include \"common.h\"\nstruct ThingA { Common c; };",
        )
        .unwrap();
        fs::write(root.join("include/header_b.h"), "struct ThingB { double x; };").unwrap();
        fs::write(
            root.join("src/tu1.cpp"),
            "#include \"header_a.h\"\nvoid use_a(ThingA a) {}",
        )
        .unwrap();
        fs::write(
            root.join("src/tu2.cpp"),
            "#include \"header_b.h\"\nvoid use_b(ThingB b) {}",
        )
        .unwrap();

        let root_str = root.to_string_lossy().replace('\\', "/");
        let compile_commands = format!(
            r#"[
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/tu1.cpp", "file": "src/tu1.cpp" }},
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/tu2.cpp", "file": "src/tu2.cpp" }}
]"#,
            root_str = root_str
        );
        fs::write(root.join("compile_commands.json"), compile_commands).unwrap();
        dir
    }

    #[test]
    fn test_cold_then_warm_all_stages() {
        let dir = setup_repo();
        let root = dir.path().to_path_buf();

        // Cold run: everything rebuilds and persists.
        let mut cold = Pipeline::new(&root, None).unwrap();
        cold.ensure_index().unwrap();
        cold.ensure_include_graph().unwrap();
        cold.ensure_reduced().unwrap();
        cold.ensure_callgraph().unwrap();
        cold.ensure_load().unwrap();
        assert_eq!(cold.stats.reanalyzed_tus, 2);
        assert_eq!(cold.stats.reused_tus, 0);
        for stage in [STAGE_INCLUDE_GRAPH, STAGE_INDEX, STAGE_REDUCED, STAGE_CALLGRAPH, STAGE_LOAD] {
            assert_eq!(cold.stats.stage_reuse(stage), Some(StageReuse::Rebuilt));
        }

        // Warm run: every stage loads, zero re-parses.
        let mut warm = Pipeline::new(&root, None).unwrap();
        warm.ensure_index().unwrap();
        warm.ensure_include_graph().unwrap();
        warm.ensure_reduced().unwrap();
        warm.ensure_callgraph().unwrap();
        warm.ensure_load().unwrap();
        assert_eq!(warm.stats.reused_tus, 0, "loaded index means zero parse work");
        assert_eq!(warm.stats.reanalyzed_tus, 0);
        for stage in [STAGE_INCLUDE_GRAPH, STAGE_INDEX, STAGE_REDUCED, STAGE_CALLGRAPH, STAGE_LOAD] {
            assert_eq!(
                warm.stats.stage_reuse(stage),
                Some(StageReuse::Loaded),
                "stage {} should load warm",
                stage
            );
        }
    }

    #[test]
    fn test_header_edit_invalidates_dependents_only() {
        let dir = setup_repo();
        let root = dir.path().to_path_buf();

        let mut cold = Pipeline::new(&root, None).unwrap();
        cold.ensure_index().unwrap();
        cold.ensure_include_graph().unwrap();
        cold.ensure_reduced().unwrap();

        // Edit common.h: tu1 depends on it via header_a.h, tu2 does not.
        fs::write(root.join("include/common.h"), "struct Common { int id; int extra; };").unwrap();

        let mut next = Pipeline::new(&root, None).unwrap();
        next.ensure_index().unwrap();
        next.ensure_include_graph().unwrap();
        assert_eq!(next.stats.reanalyzed_tus, 1, "only tu1 should re-parse");
        assert_eq!(next.stats.reused_tus, 1, "tu2 shard stays valid");
        assert_eq!(next.stats.stage_reuse(STAGE_INDEX), Some(StageReuse::Rebuilt));
        assert_eq!(next.stats.stage_reuse(STAGE_INCLUDE_GRAPH), Some(StageReuse::Rebuilt));
    }

    #[test]
    fn test_corrupt_artifact_is_not_trusted() {
        let dir = setup_repo();
        let root = dir.path().to_path_buf();

        let mut cold = Pipeline::new(&root, None).unwrap();
        cold.ensure_index().unwrap();

        // Corrupt the fingerprint in the index manifest.
        let manifest_path = root.join("artifacts").join("index").join("manifest.json");
        let manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
        let mut corrupt = manifest.as_object().unwrap().clone();
        corrupt.insert(
            "input_fingerprint".into(),
            serde_json::Value::String("0".repeat(64)),
        );
        fs::write(&manifest_path, serde_json::to_string_pretty(&corrupt).unwrap()).unwrap();

        let mut next = Pipeline::new(&root, None).unwrap();
        next.ensure_index().unwrap();
        assert_eq!(next.stats.stage_reuse(STAGE_INDEX), Some(StageReuse::Rebuilt));
        // The rebuilt index artifact is valid again for subsequent runs.
        let mut after = Pipeline::new(&root, None).unwrap();
        after.ensure_index().unwrap();
        assert_eq!(after.stats.stage_reuse(STAGE_INDEX), Some(StageReuse::Loaded));
    }

    #[test]
    fn test_production_stats_captured_only_on_workers_rebuild() {
        let dir = setup_repo();
        let root = dir.path().to_path_buf();

        let mut cold = Pipeline::new(&root, Some(2)).unwrap();
        cold.ensure_index().unwrap();
        let captured = cold.production_stats.clone().expect("workers run captures real stats");
        assert_eq!(captured.total_tus, 2);
        assert_eq!(captured.progress_completed, 2);
        assert_eq!(captured.progress_total, 2);
        assert!(captured.peak_memory_bytes > 0);

        // Warm run: the index stage loads, no worker run measured anything.
        let mut warm = Pipeline::new(&root, Some(2)).unwrap();
        warm.ensure_index().unwrap();
        assert_eq!(warm.stats.stage_reuse(STAGE_INDEX), Some(StageReuse::Loaded));
        assert!(
            warm.production_stats.is_none(),
            "loaded stage must not invent measurements"
        );
    }

    #[test]
    fn test_rehash_inputs_matches_pipeline_fps_without_side_effects() {
        let dir = setup_repo();
        let root = dir.path().to_path_buf();

        let expected = Pipeline::new(&root, None).unwrap().fps.clone();
        assert_eq!(Pipeline::rehash_inputs(&root).unwrap(), expected);

        // Pure check: no artifact open/save side effects.
        fs::remove_dir_all(root.join("artifacts")).unwrap();
        let rehashed = Pipeline::rehash_inputs(&root).unwrap();
        assert_eq!(rehashed, expected);
        assert!(!root.join("artifacts").exists());

        fs::write(root.join("include/header_b.h"), "struct ThingB { int x; };").unwrap();
        assert_ne!(Pipeline::rehash_inputs(&root).unwrap(), expected);
    }

    #[test]
    fn test_run_stats_format_summary() {
        let mut stats = RunStats::default();
        stats.record(STAGE_INDEX, StageReuse::Rebuilt);
        stats.record(STAGE_REDUCED, StageReuse::Loaded);
        stats.record(STAGE_INCLUDE_GRAPH, StageReuse::Loaded);
        stats.reused_tus = 4;
        stats.reanalyzed_tus = 1;
        let summary = stats.format_summary();
        assert_eq!(
            summary,
            "4 TUs reused, 1 reanalyzed; stages loaded: reduced, include-graph; rebuilt: index"
        );
    }

    #[test]
    fn test_hash_helper_consistency() {
        // The contexts fingerprint material must be order-stable.
        assert_eq!(
            ariadne_index::hash_parts(&["a", "b"]),
            ariadne_index::hash_parts(&["a", "b"])
        );
    }
}
