use ariadne_repo::{Repository, RepositoryDiscoveryResult, FileId};
use ariadne_build::{BuildDiscoveryResult, CompilationContext};
use ariadne_index::{IndexResult, IncrementalCache, IncrementalStats, TuCacheFingerprint, TuCacheEntry, CppSemanticAnalyzer};
use ariadne_graph::IncludeResolver;
use anyhow::Result;

pub struct IncrementalAnalysisResult {
    pub index_result: IndexResult,
    pub cache: IncrementalCache,
    pub stats: IncrementalStats,
}

impl IncrementalAnalysisResult {
    pub fn save_artifacts(&self, repo: &Repository) -> Result<()> {
        let manager = ariadne_report::ArtifactManager::prepare_for_repo(repo)?;
        
        let manifest = ariadne_report::CacheManifest {
            repository_root: repo.root.clone(),
            analysis_version: self.cache.analysis_version.clone(),
            total_cached_tus: self.cache.len(),
            reused_tus: self.stats.reused_tus,
            reanalyzed_tus: self.stats.reanalyzed_tus,
            hit_rate: self.stats.hit_rate,
        };
        manager.save_cache_artifacts(&self.cache, &manifest)?;
        manager.save_index_artifacts(&self.index_result)?;
        
        let reduced_model = ariadne_graph::ReducedSemanticModel::reduce(&self.index_result);
        manager.save_reduced_artifacts(&reduced_model)?;
        
        Ok(())
    }
}

pub struct IncrementalAnalyzer<'a> {
    repo: &'a Repository,
    repo_discovery: &'a RepositoryDiscoveryResult,
    build_result: &'a BuildDiscoveryResult,
    cache: IncrementalCache,
    analysis_version: String,
}

impl<'a> IncrementalAnalyzer<'a> {
    pub fn new(
        repo: &'a Repository,
        repo_discovery: &'a RepositoryDiscoveryResult,
        build_result: &'a BuildDiscoveryResult,
    ) -> Self {
        let cache = if let Ok(manager) = ariadne_report::ArtifactManager::prepare_for_repo(repo) {
            manager.load_cache_artifacts().unwrap_or_else(|_| IncrementalCache::new(ariadne_index::CURRENT_ANALYSIS_VERSION))
        } else {
            IncrementalCache::new(ariadne_index::CURRENT_ANALYSIS_VERSION)
        };

        Self {
            repo,
            repo_discovery,
            build_result,
            cache,
            analysis_version: ariadne_index::CURRENT_ANALYSIS_VERSION.to_string(),
        }
    }

    pub fn with_cache(mut self, cache: IncrementalCache) -> Self {
        self.cache = cache;
        self
    }

    pub fn with_analysis_version(mut self, version: impl Into<String>) -> Self {
        self.analysis_version = version.into();
        self
    }

    pub fn analyze(&mut self) -> Result<IncrementalAnalysisResult> {
        let mut reused = 0;
        let mut reanalyzed = 0;
        let mut tu_analyses = Vec::new();
        
        let mut new_cache = IncrementalCache::new(self.analysis_version.clone());
        let get_hash = |file_id: &FileId| -> String {
            self.repo_discovery.files.iter()
                .find(|f| &f.id == file_id)
                .map(|f| f.content_hash.clone())
                .unwrap_or_default()
        };
        
        for tu in &self.build_result.translation_units {
            let context = self.build_result.contexts.iter()
                .find(|c| c.name == tu.context_name)
                .cloned()
                .unwrap_or_else(|| CompilationContext::new(&tu.context_name, "clang++"));
                
            let source_hash = get_hash(&tu.file_id);
            
            let content = std::fs::read_to_string(self.repo.root.join(&tu.source_path)).unwrap_or_default();
            
            let resolver = IncludeResolver::new(&self.repo.root)
                .with_repo_discovery(self.repo_discovery)
                .with_context(&context);
                
            let mut visited = std::collections::HashSet::new();
            visited.insert(tu.file_id.clone());
            
            let mut queue = vec![(tu.file_id.clone(), tu.source_path.clone(), content.clone())];
            let mut all_dest_ids = std::collections::HashSet::new();
            
            while let Some((curr_id, curr_path, curr_content)) = queue.pop() {
                let edges = resolver.resolve_file_includes(curr_id.clone(), &curr_path, &curr_content);
                for edge in edges {
                    if let Some(dest_id) = edge.destination {
                        if !visited.contains(&dest_id) {
                            visited.insert(dest_id.clone());
                            all_dest_ids.insert(dest_id.clone());
                            
                            // Load content for further resolution
                            let dest_path = self.repo_discovery.files.iter()
                                .find(|f| f.id == dest_id)
                                .map(|f| f.path.clone());
                                
                            if let Some(path) = dest_path {
                                if let Ok(dest_content) = std::fs::read_to_string(self.repo.root.join(&path)) {
                                    queue.push((dest_id, path.clone(), dest_content));
                                }
                            }
                        }
                    }
                }
            }
                
            let mut header_hashes = Vec::new();
            for dest_id in all_dest_ids {
                header_hashes.push((dest_id.clone(), get_hash(&dest_id)));
            }
            // Sort to ensure deterministic fingerprint
            header_hashes.sort_by(|a, b| a.0.cmp(&b.0));
            
            let current_fingerprint = TuCacheFingerprint::compute(
                &self.analysis_version,
                &context,
                &tu.file_id,
                &source_hash,
                &header_hashes,
                None,
            );
            
            let analysis = if let Some(cached_analysis) = self.cache.validate_and_get(&tu.file_id, &tu.context_name, &current_fingerprint) {
                reused += 1;
                cached_analysis.clone()
            } else {
                reanalyzed += 1;
                let analyzer = CppSemanticAnalyzer;
                analyzer.analyze_tu(tu, &self.repo.root)?
            };
            
            tu_analyses.push(analysis.clone());
            
            new_cache.insert(TuCacheEntry {
                tu_file_id: tu.file_id.clone(),
                source_path: tu.source_path.clone(),
                context_name: tu.context_name.clone(),
                fingerprint: current_fingerprint,
                analysis,
            });
        }
        
        let stats = IncrementalStats::calculate(reused, reanalyzed);
        let index_result = IndexResult::new(self.repo.root.clone(), tu_analyses);
        
        Ok(IncrementalAnalysisResult {
            index_result,
            cache: new_cache,
            stats,
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
        let result1 = analyzer1.analyze().unwrap();

        let mut analyzer2 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result)
            .with_cache(result1.cache);
        let result2 = analyzer2.analyze().unwrap();
        
        assert_eq!(result2.stats.reused_tus, 2);
        assert_eq!(result2.stats.reanalyzed_tus, 0);
        assert_eq!(result2.stats.hit_rate, 100.0);
    }

    #[test]
    fn test_header_modification() {
        let (repo, mut repo_discovery, build_result, _temp) = setup_test_data();
        let mut analyzer1 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        let result1 = analyzer1.analyze().unwrap();

        // modify header.h
        if let Some(f) = repo_discovery.files.iter_mut().find(|f| f.id == FileId::from_relative_path("header.h")) {
            f.content_hash = "hash_header2".to_string();
        }

        let mut analyzer2 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result)
            .with_cache(result1.cache);
        let result2 = analyzer2.analyze().unwrap();
        
        // main.cpp should be reanalyzed (because it includes header.h), utils.cpp reused
        assert_eq!(result2.stats.reused_tus, 1);
        assert_eq!(result2.stats.reanalyzed_tus, 1);
    }
    
    #[test]
    fn test_analysis_version_mismatch() {
        let (repo, repo_discovery, build_result, _temp) = setup_test_data();
        let mut analyzer1 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result);
        let result1 = analyzer1.analyze().unwrap();

        let mut analyzer2 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result)
            .with_cache(result1.cache)
            .with_analysis_version("new_version");
        let result2 = analyzer2.analyze().unwrap();
        
        assert_eq!(result2.stats.reused_tus, 0);
        assert_eq!(result2.stats.reanalyzed_tus, 2);
    }
}
