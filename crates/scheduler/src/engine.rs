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
        .plugin_for_id(language)
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

            for analysis in new_analyses {
                let fingerprint = pending_fingerprints
                    .remove(&analysis.tu_file_id.0)
                    .unwrap_or_default();
                let checkpoint = ShardCheckpoint {
                    fingerprint,
                    analysis: analysis.clone(),
                };
                let _ = save_shard(&checkpoint_dir, &checkpoint);
                completed_analyses.push(analysis);
            }
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
}
