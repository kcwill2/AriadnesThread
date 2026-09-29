use anyhow::Result;
use ariadne_build::BuildDiscoveryResult;
use ariadne_index::{IncrementalStats, IndexResult, StageFingerprints, CURRENT_ANALYSIS_VERSION};
use ariadne_repo::{Repository, RepositoryDiscoveryResult};

use crate::tu_store::{
    analyze_closure, compute_tu_closure, load_shard, save_shard, shard_dir, ShardCheckpoint,
    SharedTuStoreContext, TuStoreContext,
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

        manager
            .save_repository_artifacts(&self.store_ctx.discovery, &self.fingerprints.repository)?;

        manager.save_index_artifacts(
            &self.index_result,
            &self.fingerprints.index,
            Some(&self.stats),
        )?;

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

        let store_ctx =
            TuStoreContext::new(self.repo_discovery, self.build_result.contexts.clone());
        let store_dir = shard_dir(&self.repo.root);

        for tu in &self.build_result.translation_units {
            let closure =
                compute_tu_closure(tu, &self.repo.root, &store_ctx, &self.analysis_version)?;

            let analysis = if let Some(shard) = load_shard(&store_dir, tu.file_id) {
                if shard.fingerprint == closure.fingerprint.combined_hash {
                    reused += 1;
                    shard.analysis
                } else {
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
            } else {
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
    use ariadne_build::{
        BuildDiscoveryResult, CompilationContext, CompileCommand, TranslationUnit,
    };
    use ariadne_repo::{
        FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, Repository,
        RepositoryDiscoveryResult,
    };
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::TempDir;

    fn setup_test_data() -> (
        Repository,
        RepositoryDiscoveryResult,
        BuildDiscoveryResult,
        TempDir,
    ) {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().canonicalize().unwrap();

        let repo = Repository { root: root.clone() };

        write_incremental_fixture_files(&root);
        let repo_discovery = given_discovered_files(&root);
        let build_result = given_build_units(&root);

        (repo, repo_discovery, build_result, temp_dir)
    }

    fn write_incremental_fixture_files(root: &Path) {
        // Create some files
        std::fs::write(
            root.join("main.cpp"),
            "#include \"header.h\"\nint main() { return 0; }",
        )
        .unwrap();
        std::fs::write(root.join("header.h"), "void foo();").unwrap();
        std::fs::write(root.join("utils.cpp"), "void bar() {}").unwrap();
    }

    fn record_repository_file(
        root: &Path,
        rel: &str,
        kind: FileKind,
        content_hash: &str,
    ) -> FileRecord {
        FileRecord {
            id: FileId::from_relative_path(rel),
            path: root.join(rel),
            canonical_path: root.join(rel),
            kind,
            size: 10,
            content_hash: content_hash.to_string(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language: ariadne_repo::SourceLanguage::Cpp,
        }
    }

    fn given_discovered_files(root: &Path) -> RepositoryDiscoveryResult {
        RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files: vec![
                record_repository_file(root, "main.cpp", FileKind::Source, "hash_main1"),
                record_repository_file(root, "header.h", FileKind::Header, "hash_header1"),
                record_repository_file(root, "utils.cpp", FileKind::Source, "hash_utils1"),
            ],
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        }
    }

    fn translation_unit_for(root: &Path, rel: &str, context_name: &str) -> TranslationUnit {
        TranslationUnit {
            file_id: FileId::from_relative_path(rel),
            source_path: root.join(rel),
            context_name: context_name.to_string(),
            compile_command: CompileCommand {
                directory: root.to_path_buf(),
                file: root.join(rel),
                command: Some(format!("clang++ -c {}", rel)),
                arguments: vec!["clang++".to_string(), "-c".to_string(), rel.to_string()],
                context_name: context_name.to_string(),
                output: None,
            },
        }
    }

    fn given_build_units(root: &Path) -> BuildDiscoveryResult {
        let ctx = CompilationContext::new("test_ctx", "clang++");
        BuildDiscoveryResult {
            repo_root: root.to_path_buf(),
            contexts: vec![ctx.clone()],
            translation_units: vec![
                translation_unit_for(root, "main.cpp", &ctx.name),
                translation_unit_for(root, "utils.cpp", &ctx.name),
            ],
        }
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
        if let Some(f) = repo_discovery
            .files
            .iter_mut()
            .find(|f| f.id == FileId::from_relative_path("header.h"))
        {
            f.content_hash = "hash_header2".to_string();
        }
        repo = Repository {
            root: repo.root.clone(),
        };

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
        let analysis = result
            .index_result
            .tu_analyses
            .iter()
            .find(|a| a.tu_file_id == tu_id)
            .unwrap()
            .clone();
        std::fs::write(
            dir.join(format!("{}.json", tu_id.0)),
            serde_json::to_string(&analysis).unwrap(),
        )
        .unwrap();

        let mut analyzer2 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        let result2 = analyzer2.analyze().unwrap();

        assert_eq!(result2.stats.reused_tus, 1);
        assert_eq!(result2.stats.reanalyzed_tus, 1);
    }
}
