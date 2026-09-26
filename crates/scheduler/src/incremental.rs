use ariadne_repo::{Repository, RepositoryDiscoveryResult};
use ariadne_build::BuildDiscoveryResult;
use ariadne_index::{IndexResult, IncrementalStats, StageFingerprints, CURRENT_ANALYSIS_VERSION};
use anyhow::Result;

use crate::tu_store::{
    analyze_closure, compute_tu_closure, load_shard, save_shard, shard_dir,
    SharedTuStoreContext, ShardCheckpoint, TuStoreContext,
};

pub struct IncrementalAnalysisResult {
    pub index_result: IndexResult,
    pub stats: IncrementalStats,
    pub fingerprints: StageFingerprints,
    store_ctx: SharedTuStoreContext,
}

impl IncrementalAnalysisResult {
    pub fn save_artifacts(&self, repo: &Repository) -> Result<()> {
        let manager = ariadne_report::ArtifactManager::prepare_for_repo(repo)?;

        manager.save_repository_artifacts(&self.store_ctx.discovery, &self.fingerprints.repository)?;

        manager.save_index_artifacts(&self.index_result, &self.fingerprints.index, Some(&self.stats))?;

        let reduced_model = ariadne_graph::ReducedSemanticModel::reduce(&self.index_result);
        manager.save_reduced_artifacts(&reduced_model, &self.fingerprints.reduced)?;

        Ok(())
    }
}

pub struct IncrementalAnalyzer<'a> {
    repo: &'a Repository,
    repo_discovery: &'a RepositoryDiscoveryResult,
    build_result: &'a BuildDiscoveryResult,
    analysis_version: String,
}

impl<'a> IncrementalAnalyzer<'a> {
    pub fn new(
        repo: &'a Repository,
        repo_discovery: &'a RepositoryDiscoveryResult,
        build_result: &'a BuildDiscoveryResult,
    ) -> Self {
        Self {
            repo,
            repo_discovery,
            build_result,
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        }
    }

    pub fn with_analysis_version(mut self, version: impl Into<String>) -> Self {
        self.analysis_version = version.into();
        self
    }

    pub fn analyze(&mut self) -> Result<IncrementalAnalysisResult> {
        let mut reused = 0usize;
        let mut reanalyzed = 0usize;
        let mut tu_analyses = Vec::new();

        let store_ctx = TuStoreContext::new(self.repo_discovery, self.build_result.contexts.clone());
        let store_dir = shard_dir(&self.repo.root);

        for tu in &self.build_result.translation_units {
            let closure = compute_tu_closure(tu, &self.repo.root, &store_ctx, &self.analysis_version)?;

            let analysis = if let Some(shard) = load_shard(&store_dir, tu.file_id) {
                if shard.fingerprint == closure.fingerprint.combined_hash {
                    reused += 1;
                    shard.analysis
                } else {
                    reanalyzed += 1;
                    let analysis = analyze_closure(tu, &closure);
                    save_shard(&store_dir, &ShardCheckpoint {
                        fingerprint: closure.fingerprint.combined_hash.clone(),
                        analysis: analysis.clone(),
                    })?;
                    analysis
                }
            } else {
                reanalyzed += 1;
                let analysis = analyze_closure(tu, &closure);
                save_shard(&store_dir, &ShardCheckpoint {
                    fingerprint: closure.fingerprint.combined_hash.clone(),
                    analysis: analysis.clone(),
                })?;
                analysis
            };

            tu_analyses.push(analysis);
        }

        let stats = IncrementalStats::calculate(reused, reanalyzed);
        let index_result = IndexResult::new(self.repo.root.clone(), tu_analyses);

        let fingerprints = StageFingerprints::from_inputs(
            &self.repo.root,
            self.repo_discovery,
            &self.build_result.contexts,
            &self.build_result.translation_units,
        );

        Ok(IncrementalAnalysisResult {
            index_result,
            stats,
            fingerprints,
            store_ctx,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_repo::{Repository, RepositoryDiscoveryResult, FileId, FileRecord, FileKind, FileOwnership, OptimizationPolicy};
    use ariadne_build::{BuildDiscoveryResult, CompilationContext, TranslationUnit, CompileCommand};
    use tempfile::TempDir;
    use std::collections::HashMap;

    fn setup_test_data() -> (Repository, RepositoryDiscoveryResult, BuildDiscoveryResult, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().canonicalize().unwrap();

        let repo = Repository { root: root.clone() };

        // Create some files
        std::fs::write(root.join("main.cpp"), "#include \"header.h\"\nint main() { return 0; }").unwrap();
        std::fs::write(root.join("header.h"), "void foo();").unwrap();
        std::fs::write(root.join("utils.cpp"), "void bar() {}").unwrap();

        let mut repo_discovery = RepositoryDiscoveryResult {
            root: root.clone(),
            files: vec![],
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        };

        let f_main = FileRecord {
            id: FileId::from_relative_path("main.cpp"),
            path: root.join("main.cpp"),
            canonical_path: root.join("main.cpp"),
            kind: FileKind::Source,
            size: 10,
            content_hash: "hash_main1".to_string(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
        };
        let f_header = FileRecord {
            id: FileId::from_relative_path("header.h"),
            path: root.join("header.h"),
            canonical_path: root.join("header.h"),
            kind: FileKind::Header,
            size: 10,
            content_hash: "hash_header1".to_string(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
        };
        let f_utils = FileRecord {
            id: FileId::from_relative_path("utils.cpp"),
            path: root.join("utils.cpp"),
            canonical_path: root.join("utils.cpp"),
            kind: FileKind::Source,
            size: 10,
            content_hash: "hash_utils1".to_string(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
        };


        repo_discovery.files.push(f_main);
        repo_discovery.files.push(f_header);
        repo_discovery.files.push(f_utils);

        let mut build_result = BuildDiscoveryResult {
            repo_root: root.clone(),
            contexts: vec![],
            translation_units: vec![],
        };

        let ctx = CompilationContext::new("test_ctx", "clang++");
        build_result.contexts.push(ctx.clone());

        let tu_main = TranslationUnit {
            file_id: FileId::from_relative_path("main.cpp"),
            source_path: root.join("main.cpp"),
            context_name: ctx.name.clone(),
            compile_command: CompileCommand {
                directory: root.clone(),
                file: root.join("main.cpp"),
                command: Some("clang++ -c main.cpp".to_string()),
                arguments: vec!["clang++".to_string(), "-c".to_string(), "main.cpp".to_string()],
                context_name: ctx.name.clone(),
                output: None,
            },
        };

        let tu_utils = TranslationUnit {
            file_id: FileId::from_relative_path("utils.cpp"),
            source_path: root.join("utils.cpp"),
            context_name: ctx.name.clone(),
            compile_command: CompileCommand {
                directory: root.clone(),
                file: root.join("utils.cpp"),
                command: Some("clang++ -c utils.cpp".to_string()),
                arguments: vec!["clang++".to_string(), "-c".to_string(), "utils.cpp".to_string()],
                context_name: ctx.name.clone(),
                output: None,
            },
        };

        build_result.translation_units.push(tu_main);
        build_result.translation_units.push(tu_utils);

        (repo, repo_discovery, build_result, temp_dir)
    }

    #[test]
    fn test_cold_run() {
        let (repo, repo_discovery, build_result, _temp) = setup_test_data();
        let mut analyzer = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.stats.reused_tus, 0);
        assert_eq!(result.stats.reanalyzed_tus, 2);
        assert_eq!(result.stats.hit_rate, 0.0);
    }

    #[test]
    fn test_warm_run() {
        let (repo, repo_discovery, build_result, _temp) = setup_test_data();
        let mut analyzer1 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        analyzer1.analyze().unwrap();

        let mut analyzer2 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        let result2 = analyzer2.analyze().unwrap();

        assert_eq!(result2.stats.reused_tus, 2);
        assert_eq!(result2.stats.reanalyzed_tus, 0);
        assert_eq!(result2.stats.hit_rate, 100.0);
    }

    #[test]
    fn test_header_modification() {
        let (mut repo, mut repo_discovery, build_result, _temp) = setup_test_data();
        let mut analyzer1 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        analyzer1.analyze().unwrap();

        // modify header.h on disk and update its recorded hash
        std::fs::write(repo.root.join("header.h"), "void foo(); void bar();").unwrap();
        if let Some(f) = repo_discovery.files.iter_mut().find(|f| f.id == FileId::from_relative_path("header.h")) {
            f.content_hash = "hash_header2".to_string();
        }
        repo = Repository { root: repo.root.clone() };

        let mut analyzer2 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        let result2 = analyzer2.analyze().unwrap();

        // main.cpp should be reanalyzed (because it includes header.h), utils.cpp reused
        assert_eq!(result2.stats.reused_tus, 1);
        assert_eq!(result2.stats.reanalyzed_tus, 1);
    }

    #[test]
    fn test_analysis_version_mismatch() {
        let (repo, repo_discovery, build_result, _temp) = setup_test_data();
        let mut analyzer1 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        analyzer1.analyze().unwrap();

        let mut analyzer2 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result)
            .with_analysis_version("new_version");
        let result2 = analyzer2.analyze().unwrap();

        assert_eq!(result2.stats.reused_tus, 0);
        assert_eq!(result2.stats.reanalyzed_tus, 2);
    }

    #[test]
    fn test_old_plain_shard_treated_stale() {
        let (repo, repo_discovery, build_result, _temp) = setup_test_data();
        let mut analyzer = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        let result = analyzer.analyze().unwrap();

        // Overwrite one shard with the old plain TuAnalysis format.
        let dir = shard_dir(&repo.root);
        let tu_id = build_result.translation_units[0].file_id;
        let analysis = result.index_result.tu_analyses.iter()
            .find(|a| a.tu_file_id == tu_id).unwrap().clone();
        std::fs::write(dir.join(format!("{}.json", tu_id.0)), serde_json::to_string(&analysis).unwrap()).unwrap();

        let mut analyzer2 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        let result2 = analyzer2.analyze().unwrap();

        assert_eq!(result2.stats.reused_tus, 1);
        assert_eq!(result2.stats.reanalyzed_tus, 1);
    }
}
