use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ariadne_build::{BuildDiscoveryResult, TranslationUnit};
use ariadne_graph::ReducedSemanticModel;
use ariadne_index::{IndexResult, TuAnalysis, CURRENT_ANALYSIS_VERSION};
use ariadne_lang::{LanguagePlugin, LanguageRegistry};
use ariadne_repo::{Repository, RepositoryDiscoveryResult};

use crate::pool::{WorkerPool, WorkerPoolConfig};
use crate::queue::PrioritizedTuQueue;
use crate::shard::ShardedReducer;
use crate::tu_store::{
    compute_tu_closure_plugin, load_shard, save_shard, shard_dir, ShardCheckpoint,
    TuStoreContext,
};
use crate::worker::{SharedPluginWorkerBackend, WorkerBackend};

#[derive(Debug, Clone)]
pub struct ProductionAnalysisStats {
    pub total_tus: usize,
    pub num_workers: usize,
    pub progress_completed: usize,
    pub progress_total: usize,
    pub total_semantic_edges: usize,
    pub peak_memory_bytes: u64,
    pub elapsed: Duration,
    pub reused_tus: usize,
    pub reanalyzed_tus: usize,
}

impl ProductionAnalysisStats {
    pub fn format_memory(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn format_elapsed(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        if total_secs >= 60 {
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!("{}m {}s", mins, secs)
        } else {
            format!("{:.2}s", duration.as_secs_f64())
        }
    }

    pub fn format_m13_output(&self) -> String {
        format!(
            "Translation units: {}\n\nWorkers:\n    {}\n\nProgress:\n    {} / {}\n\nSemantic edges:\n    {}\n\nPeak memory:\n    {}\n\nElapsed:\n    {}",
            self.total_tus,
            self.num_workers,
            self.progress_completed,
            self.progress_total,
            self.total_semantic_edges,
            Self::format_memory(self.peak_memory_bytes),
            Self::format_elapsed(self.elapsed),
        )
    }
}

pub struct ProductionAnalysisResult {
    pub reduced_model: ReducedSemanticModel,
    pub index_result: IndexResult,
    pub stats: ProductionAnalysisStats,
}

/// Outcome of the shard prepass: checkpointed analyses that could be reused
/// plus the TUs that must run through the worker pool, each with the
/// fingerprint its result shard will be enveloped with.
struct ShardPrepass {
    reused_analyses: Vec<TuAnalysis>,
    pending_tus: Vec<TranslationUnit>,
    pending_fingerprints: HashMap<u64, String>,
}

/// Check for existing checkpointed shards; a shard is reused only when its
/// envelope fingerprint still matches the code on disk. The closure walk runs
/// through the unit's language plugin (M18.11: the registry path is the only
/// path), so shard fingerprints match what the plugin workers re-stamp.
fn shard_prepass(
    translation_units: &[TranslationUnit],
    repo_root: &Path,
    store_ctx: &TuStoreContext,
    checkpoint_dir: &Path,
    registry: &LanguageRegistry,
) -> Result<ShardPrepass> {
    let mut prepass = ShardPrepass {
        reused_analyses: Vec::new(),
        pending_tus: Vec::new(),
        pending_fingerprints: HashMap::new(),
    };
    for tu in translation_units {
        let plugin = plugin_for_tu(registry, tu)?;
        let closure = compute_tu_closure_plugin(
            tu,
            repo_root,
            store_ctx,
            CURRENT_ANALYSIS_VERSION,
            plugin.resolver(),
            plugin.id(),
            &plugin.analysis_fingerprint(),
        )?;
        let fp = closure.fingerprint.combined_hash.clone();
        if let Some(shard) = load_shard(checkpoint_dir, tu.file_id) {
            if shard.fingerprint == fp {
                prepass.reused_analyses.push(shard.analysis);
                continue;
            }
        }
        prepass.pending_fingerprints.insert(tu.file_id.0, fp);
        prepass.pending_tus.push(tu.clone());
    }
    Ok(prepass)
}

/// Envelope each fresh analysis with the fingerprint the prepass claimed for
/// its file id, persist it, and move the analysis into `completed_analyses`.
///
/// UTA-B6 surfacing (replaces the `unwrap_or_default()` + `let _ =` swallows):
/// * an analysis whose `tu_file_id` was never claimed by this shard run is
///   NOT checkpointed (an empty fingerprint would fake a valid envelope);
///   the incident is warned per unit and counted.
/// * a failed `save_shard` is warned with its IO error and counted; the
///   analysis itself is still valid, so the run continues (a lost checkpoint
///   costs a re-analysis next run, never a wrong answer).
///
/// Run-level incident totals are warned as one aggregate line when nonzero.
fn persist_shard_checkpoints(
    new_analyses: Vec<TuAnalysis>,
    pending_fingerprints: &mut HashMap<u64, String>,
    checkpoint_dir: &Path,
    completed_analyses: &mut Vec<TuAnalysis>,
) {
    let mut skipped_foreign = 0usize;
    let mut save_failures = 0usize;
    for analysis in new_analyses {
        let claimed_fingerprint = pending_fingerprints.remove(&analysis.tu_file_id.0);
        let Some(fingerprint) = claimed_fingerprint else {
            skipped_foreign += 1;
            eprintln!(
                "ariadne-scheduler: WARNING: dropping shard checkpoint for foreign TU file id \
                 {} (no prepass claim; an empty fingerprint would fake a valid envelope); \
                 the analysis itself still flows into the reduction",
                analysis.tu_file_id.0
            );
            completed_analyses.push(analysis);
            continue;
        };
        let checkpoint = ShardCheckpoint {
            fingerprint,
            analysis: analysis.clone(),
        };
        if let Err(save_error) = save_shard(checkpoint_dir, &checkpoint) {
            save_failures += 1;
            eprintln!(
                "ariadne-scheduler: WARNING: shard checkpoint save failed for TU file id \
                 {}: {save_error}; the analysis itself still flows into the reduction",
                analysis.tu_file_id.0
            );
        }
        completed_analyses.push(analysis);
    }
    if skipped_foreign > 0 || save_failures > 0 {
        eprintln!(
            "ariadne-scheduler: WARNING: shard checkpoint incidents this run: \
             {skipped_foreign} skipped-foreign, {save_failures} failed saves"
        );
    }
}

/// The plugin that owns a unit's language; an unclaimed path is a clear
/// "unhandled language" error (§3.3 fact 3), never a silent C++ analysis.
fn plugin_for_tu<'a>(
    registry: &'a LanguageRegistry,
    tu: &TranslationUnit,
) -> Result<&'a dyn LanguagePlugin> {
    let language = registry.detect_language(&tu.source_path).ok_or_else(|| {
        anyhow::anyhow!(
            "unhandled language for {} (no registered plugin claims it)",
            tu.source_path.display()
        )
    })?;
    registry
        .plugin_for_id(language.clone())
        .ok_or_else(|| anyhow::anyhow!("language plugin not registered: {language:?}"))
}

pub struct ProductionAnalyzer<'a> {
    pub repo: &'a Repository,
    pub repo_discovery: &'a RepositoryDiscoveryResult,
    pub build_discovery: &'a BuildDiscoveryResult,
    pub num_workers: usize,
    pub num_reducers: usize,
    pub checkpoint_dir: Option<PathBuf>,
    /// M18.3/M18.11: worker backend selected by the caller; when unset the
    /// pool runs a [`SharedPluginWorkerBackend`] over the injected registry.
    backend: Option<Arc<dyn WorkerBackend>>,
    /// M18.11: the language registry whose plugins own every unit's closure
    /// walk, fingerprint, and analysis. Required (the pre-M11 hardcoded-C++
    /// default is removed); `analyze` errors when nothing was injected.
    registry: Option<Arc<LanguageRegistry>>,
}

impl<'a> ProductionAnalyzer<'a> {
    pub fn new(
        repo: &'a Repository,
        repo_discovery: &'a RepositoryDiscoveryResult,
        build_discovery: &'a BuildDiscoveryResult,
    ) -> Self {
        Self {
            repo,
            repo_discovery,
            build_discovery,
            num_workers: 4,
            num_reducers: 4,
            checkpoint_dir: None,
            backend: None,
            registry: None,
        }
    }

    /// Run the worker pool through this backend (plan M18.3 registry wiring).
    pub fn with_backend(mut self, backend: Arc<dyn WorkerBackend>) -> Self {
        self.backend = Some(backend);
        self
    }

    /// Drive the shard prepass (and the default pool backend) through these
    /// plugins (plan M18.3, required since M18.11).
    pub fn with_registry(mut self, registry: Arc<LanguageRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn with_workers(mut self, workers: usize) -> Self {
        self.num_workers = workers.max(1);
        self
    }

    pub fn with_reducers(mut self, reducers: usize) -> Self {
        self.num_reducers = reducers.max(1);
        self
    }

    pub fn with_checkpoint_dir(mut self, dir: PathBuf) -> Self {
        self.checkpoint_dir = Some(dir);
        self
    }

    pub async fn analyze(&self) -> Result<ProductionAnalysisResult> {
        let registry = Arc::clone(
            self.registry
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("ProductionAnalyzer requires with_registry(..): the hardcoded-C++ default was removed in M18.11"))?,
        );
        let start_time = Instant::now();
        let total_tus = self.build_discovery.translation_units.len();

        let checkpoint_dir = self
            .checkpoint_dir
            .clone()
            .unwrap_or_else(|| shard_dir(&self.repo.root));
        fs::create_dir_all(&checkpoint_dir)?;

        let store_ctx =
            TuStoreContext::new(self.repo_discovery, self.build_discovery.contexts.clone());

        // Reuse checkpointed shards; only fingerprint misses go to the pool.
        let prepass = shard_prepass(
            &self.build_discovery.translation_units,
            &self.repo.root,
            &store_ctx,
            &checkpoint_dir,
            &registry,
        )?;
        let mut completed_analyses = prepass.reused_analyses;
        let reused = completed_analyses.len();
        let remaining_tus = prepass.pending_tus;
        let reanalyzed = remaining_tus.len();
        let mut pending_fingerprints = prepass.pending_fingerprints;

        let mut peak_mem: u64 = 0;

        if !remaining_tus.is_empty() {
            let mut queue = PrioritizedTuQueue::from_translation_units_with_context(
                &self.repo.root,
                remaining_tus,
                Some(store_ctx.clone()),
            );
            // M18.11: the default pool backend is the registry-dispatching
            // plugin backend over the injected registry; callers may still
            // inject a custom backend via `with_backend`.
            let backend: Arc<dyn WorkerBackend> = self
                .backend
                .clone()
                .unwrap_or_else(|| Arc::new(SharedPluginWorkerBackend::new(Arc::clone(&registry))));
            let pool = WorkerPool::new(
                backend,
                WorkerPoolConfig {
                    num_workers: self.num_workers,
                    max_retries: 3,
                    memory_limit_bytes: None,
                },
            );

            let new_analyses = pool.run_worker_pool(&self.repo.root, &mut queue).await?;
            peak_mem = pool.peak_memory_bytes();

            persist_shard_checkpoints(
                new_analyses,
                &mut pending_fingerprints,
                &checkpoint_dir,
                &mut completed_analyses,
            );
        }

        // Sharded reduction across partitions
        let reducer = ShardedReducer::new(self.num_reducers);
        let reduced_model = reducer
            .reduce(self.repo.root.clone(), &completed_analyses)
            .await;

        let total_semantic_edges = reduced_model.edges.len();
        let elapsed = start_time.elapsed();
        let progress_completed = completed_analyses.len();

        let index_result = IndexResult::new(self.repo.root.clone(), completed_analyses);

        let stats = self.build_analysis_stats(
            total_tus,
            progress_completed,
            total_semantic_edges,
            peak_mem,
            elapsed,
            reused,
            reanalyzed,
        );

        Ok(ProductionAnalysisResult {
            reduced_model,
            index_result,
            stats,
        })
    }

    fn build_analysis_stats(
        &self,
        total_tus: usize,
        progress_completed: usize,
        total_semantic_edges: usize,
        peak_mem: u64,
        elapsed: Duration,
        reused: usize,
        reanalyzed: usize,
    ) -> ProductionAnalysisStats {
        ProductionAnalysisStats {
            total_tus,
            num_workers: self.num_workers,
            progress_completed,
            progress_total: total_tus,
            total_semantic_edges,
            peak_memory_bytes: peak_mem.max(1024 * 1024),
            elapsed,
            reused_tus: reused,
            reanalyzed_tus: reanalyzed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_build::{CompileCommand, TranslationUnit};
    use ariadne_repo::FileId;
    use tempfile::tempdir;

    #[test]
    fn test_format_stats_m13() {
        let stats = ProductionAnalysisStats {
            total_tus: 8421,
            num_workers: 32,
            progress_completed: 8421,
            progress_total: 8421,
            total_semantic_edges: 12381442,
            peak_memory_bytes: 31 * 1024 * 1024 * 1024,
            elapsed: Duration::from_secs(522),
            reused_tus: 0,
            reanalyzed_tus: 8421,
        };
        let formatted = stats.format_m13_output();
        assert!(formatted.contains("Translation units: 8421"));
        assert!(formatted.contains("Workers:\n    32"));
        assert!(formatted.contains("Progress:\n    8421 / 8421"));
        assert!(formatted.contains("Semantic edges:\n    12381442"));
        assert!(formatted.contains("Peak memory:\n    31.0 GB"));
        assert!(formatted.contains("Elapsed:\n    8m 42s"));
    }

    #[tokio::test]
    async fn test_resumability_and_checkpointing() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path().to_path_buf();
        let src_dir = repo_root.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let f1_path = src_dir.join("a.cpp");
        let f2_path = src_dir.join("b.cpp");
        fs::write(&f1_path, "class Alpha {};").unwrap();
        fs::write(&f2_path, "class Beta { Alpha a; };").unwrap();

        let repo = Repository {
            root: repo_root.clone(),
        };
        let f1_id = FileId::from_relative_path("src/a.cpp");
        let f2_id = FileId::from_relative_path("src/b.cpp");

        let repo_discovery = ariadne_repo::RepositoryDiscoveryResult {
            root: repo_root.clone(),
            files: vec![
                ariadne_repo::FileRecord {
                    id: f1_id.clone(),
                    path: PathBuf::from("src/a.cpp"),
                    canonical_path: f1_path.clone(),
                    kind: ariadne_repo::FileKind::Source,
                    size: 100,
                    ownership: ariadne_repo::FileOwnership::Repository,
                    optimization_policy: ariadne_repo::OptimizationPolicy::Optimizable,
                    content_hash: "hash1".into(),
                    language: ariadne_repo::SourceLanguage::Cpp,
                },
                ariadne_repo::FileRecord {
                    id: f2_id.clone(),
                    path: PathBuf::from("src/b.cpp"),
                    canonical_path: f2_path.clone(),
                    kind: ariadne_repo::FileKind::Source,
                    size: 100,
                    ownership: ariadne_repo::FileOwnership::Repository,
                    optimization_policy: ariadne_repo::OptimizationPolicy::Optimizable,
                    content_hash: "hash2".into(),
                    language: ariadne_repo::SourceLanguage::Cpp,
                },
            ],
            ownership_summary: std::collections::HashMap::new(),
            policy_summary: std::collections::HashMap::new(),
        };

        let cmd = CompileCommand {
            directory: repo_root.clone(),
            file: PathBuf::from("src/a.cpp"),
            command: None,
            arguments: vec![],
            context_name: "default".into(),
            output: None,
        };

        let build_discovery = BuildDiscoveryResult {
            repo_root: repo_root.clone(),
            contexts: vec![],
            translation_units: vec![
                TranslationUnit {
                    file_id: f1_id.clone(),
                    source_path: PathBuf::from("src/a.cpp"),
                    context_name: "default".into(),
                    compile_command: cmd.clone(),
                    language: Default::default(),
                },
                TranslationUnit {
                    file_id: f2_id.clone(),
                    source_path: PathBuf::from("src/b.cpp"),
                    context_name: "default".into(),
                    compile_command: cmd,
                    language: Default::default(),
                },
            ],
        };

        let checkpoint_dir = repo_root.join("artifacts").join("shards");
        let registry = Arc::new(
            ariadne_lang::LanguageRegistry::new()
                .register(Box::new(ariadne_lang_cpp::CppPlugin::new())),
        );
        let analyzer = ProductionAnalyzer::new(&repo, &repo_discovery, &build_discovery)
            .with_checkpoint_dir(checkpoint_dir.clone())
            .with_registry(registry);

        // First run: cold run, creates checkpoints
        let res1 = analyzer.analyze().await.unwrap();
        assert_eq!(res1.stats.progress_completed, 2);
        assert_eq!(res1.stats.reanalyzed_tus, 2);
        assert!(checkpoint_dir.join(format!("{}.json", f1_id.0)).exists());
        assert!(checkpoint_dir.join(format!("{}.json", f2_id.0)).exists());

        // Second run: should resume using checkpoints
        let res2 = analyzer.analyze().await.unwrap();
        assert_eq!(res2.stats.progress_completed, 2);
        assert_eq!(res2.stats.reused_tus, 2);
        assert_eq!(res2.stats.reanalyzed_tus, 0);
        assert_eq!(res1.reduced_model.symbols, res2.reduced_model.symbols);
    }

    // ---- UTA-T5 item 3 (UTA-B6): checkpoint-loop failure surfacing ---------

    /// Two-unit C++ fixture shared by the checkpoint-loop tests.
    fn production_pin_fixture(
        root: &std::path::Path,
    ) -> (
        Repository,
        RepositoryDiscoveryResult,
        BuildDiscoveryResult,
    ) {
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("a.cpp"), "class Alpha {};").unwrap();
        fs::write(src_dir.join("b.cpp"), "class Beta { Alpha a; };").unwrap();
        let records = ["src/a.cpp", "src/b.cpp"]
            .iter()
            .enumerate()
            .map(|(index, rel)| ariadne_repo::FileRecord {
                id: FileId::from_relative_path(rel),
                path: PathBuf::from(rel),
                canonical_path: root.join(rel),
                kind: ariadne_repo::FileKind::Source,
                size: 100,
                ownership: ariadne_repo::FileOwnership::Repository,
                optimization_policy: ariadne_repo::OptimizationPolicy::Optimizable,
                content_hash: format!("pin-hash-{index}"),
                language: ariadne_repo::SourceLanguage::Cpp,
            })
            .collect();
        let repo_discovery = RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files: records,
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        };
        let cmd = CompileCommand {
            directory: root.to_path_buf(),
            file: PathBuf::from("src/a.cpp"),
            command: None,
            arguments: vec![],
            context_name: "default".into(),
            output: None,
        };
        let build_discovery = BuildDiscoveryResult {
            repo_root: root.to_path_buf(),
            contexts: vec![],
            translation_units: ["src/a.cpp", "src/b.cpp"]
                .iter()
                .map(|rel| TranslationUnit {
                    file_id: FileId::from_relative_path(rel),
                    source_path: PathBuf::from(rel),
                    context_name: "default".into(),
                    compile_command: cmd.clone(),
                    language: Default::default(),
                })
                .collect(),
        };
        (
            Repository {
                root: root.to_path_buf(),
            },
            repo_discovery,
            build_discovery,
        )
    }

    fn pin_registry() -> Arc<LanguageRegistry> {
        Arc::new(
            LanguageRegistry::new().register(Box::new(ariadne_lang_cpp::CppPlugin::new())),
        )
    }

    #[tokio::test]
    async fn foreign_tu_file_id_is_never_checkpointed_with_an_empty_fingerprint() {
        use crate::queue::TuTask;

        // A backend that answers every task with a foreign unit's file id:
        // the checkpoint loop must not write an envelope for an id this shard
        // run never claimed (the empty `unwrap_or_default()` fingerprint).
        struct ForeignIdBackend;
        impl WorkerBackend for ForeignIdBackend {
            fn analyze_task(&self, task: &TuTask, _repo_root: &Path) -> Result<TuAnalysis> {
                Ok(TuAnalysis {
                    tu_file_id: FileId::from_relative_path("foreign/ghost.cpp"),
                    context_name: task.tu.context_name.clone(),
                    ..Default::default()
                })
            }
        }

        let dir = tempdir().unwrap();
        let root = dir.path();
        let (repo, repo_discovery, build_discovery) = production_pin_fixture(root);
        let checkpoint_dir = root.join("artifacts").join("shards");

        let analyzer = ProductionAnalyzer::new(&repo, &repo_discovery, &build_discovery)
            .with_checkpoint_dir(checkpoint_dir.clone())
            .with_registry(pin_registry())
            .with_backend(Arc::new(ForeignIdBackend));
        let result = analyzer.analyze().await.unwrap();

        // The foreign analyses still flow through the reduction (no data
        // loss from skipping the checkpoint write).
        assert_eq!(result.stats.progress_completed, 2);
        assert_eq!(result.index_result.tu_analyses.len(), 2);

        // But no shard may be written under an unclaimed id.
        let foreign_id = FileId::from_relative_path("foreign/ghost.cpp");
        assert!(
            load_shard(&checkpoint_dir, foreign_id).is_none(),
            "a foreign tu_file_id must not be checkpointed with the \
             unwrap_or_default() empty fingerprint"
        );
        let json_shards: Vec<std::path::PathBuf> = fs::read_dir(&checkpoint_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.to_string_lossy().ends_with(".json"))
            .collect();
        assert!(
            json_shards.is_empty(),
            "no shard may be written for an unclaimed analysis: {json_shards:?}"
        );
    }

    #[tokio::test]
    async fn checkpoint_save_failure_surfaces_but_does_not_kill_the_run() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let (repo, repo_discovery, build_discovery) = production_pin_fixture(root);
        let checkpoint_dir = root.join("artifacts").join("shards");
        fs::create_dir_all(&checkpoint_dir).unwrap();

        // A directory squatting on a.cpp's shard path makes fs::rename fail
        // there (EISDIR on unix, ERROR_ACCESS_DENIED on Windows) while b.cpp
        // saves normally.
        let a_id = FileId::from_relative_path("src/a.cpp");
        let blocked_path = checkpoint_dir.join(format!("{}.json", a_id.0));
        fs::create_dir_all(&blocked_path).unwrap();

        let analyzer = ProductionAnalyzer::new(&repo, &repo_discovery, &build_discovery)
            .with_checkpoint_dir(checkpoint_dir.clone())
            .with_registry(pin_registry());
        let result = analyzer
            .analyze()
            .await
            .expect("a failed checkpoint save must not fail the whole analysis");

        // Both units still complete and reduce.
        assert_eq!(result.stats.progress_completed, 2);

        // The blocked shard stayed absent (the rename never replaced the
        // directory); the healthy shard saved normally. The incident itself
        // surfaces as a WARNING on stderr (non-silent channel; see journal
        // for why the count is not a return-value field).
        assert!(
            blocked_path.is_dir(),
            "a failed save must leave the target untouched, not a partial shard"
        );
        let b_id = FileId::from_relative_path("src/b.cpp");
        let saved = load_shard(&checkpoint_dir, b_id).expect("the healthy shard must be saved");
        assert!(!saved.fingerprint.is_empty(), "saved fingerprints stay honest");
    }

    // ---- UTA-T5 item 6: stat formatting boundaries + peak floor ------------

    #[test]
    fn format_memory_boundary_table() {
        assert_eq!(ProductionAnalysisStats::format_memory(0), "0 B");
        assert_eq!(ProductionAnalysisStats::format_memory(1023), "1023 B");
        assert_eq!(ProductionAnalysisStats::format_memory(1024), "1.0 KB");
        assert_eq!(ProductionAnalysisStats::format_memory(1_048_575), "1024.0 KB");
        assert_eq!(ProductionAnalysisStats::format_memory(1_048_576), "1.0 MB");
        assert_eq!(ProductionAnalysisStats::format_memory(1_073_741_823), "1024.0 MB");
        assert_eq!(ProductionAnalysisStats::format_memory(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn format_elapsed_boundary_table() {
        assert_eq!(
            ProductionAnalysisStats::format_elapsed(Duration::from_secs_f64(0.0)),
            "0.00s"
        );
        // Just under a minute: still seconds-formatted with 2 decimals.
        assert_eq!(
            ProductionAnalysisStats::format_elapsed(Duration::from_secs_f64(59.99)),
            "59.99s"
        );
        // Exactly a minute flips to the minute form.
        assert_eq!(
            ProductionAnalysisStats::format_elapsed(Duration::from_secs(60)),
            "1m 0s"
        );
        assert_eq!(
            ProductionAnalysisStats::format_elapsed(Duration::from_secs(522)),
            "8m 42s"
        );
    }

    #[test]
    fn peak_memory_floor_is_one_megabyte_when_nothing_was_measured() {
        let dir = tempdir().unwrap();
        let (repo, repo_discovery, build_discovery) = production_pin_fixture(dir.path());
        let analyzer = ProductionAnalyzer::new(&repo, &repo_discovery, &build_discovery);
        // A run whose pool never measured (empty backend path): the reported
        // peak floors at 1 MB instead of an honest-looking zero.
        let floored = analyzer.build_analysis_stats(2, 2, 0, 0, Duration::ZERO, 0, 2);
        assert_eq!(floored.peak_memory_bytes, 1024 * 1024);
        // Anything above the floor is reported verbatim.
        let measured =
            analyzer.build_analysis_stats(2, 2, 0, 5 * 1024 * 1024, Duration::ZERO, 0, 2);
        assert_eq!(measured.peak_memory_bytes, 5 * 1024 * 1024);
    }
}
