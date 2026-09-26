//! [`LanguagePlugin`] aggregate trait and [`LanguageRegistry`].

use crate::analyzer::SemanticAnalyzer;
use crate::constraints::ClusteringConstraints;
use crate::detector::{LanguageDetector, SourceFileKind};
use crate::extractor::SourceExtractor;
use crate::resolver::DependencyResolver;
use crate::rewriter::DependencyRewriter;
use crate::validator::CompilerValidator;
use std::path::Path;

/// Bundles all language-specific pipeline components into one object.
///
/// Each supported language provides exactly one `LanguagePlugin` implementation
/// and registers it with [`LanguageRegistry`]. The core pipeline never
/// references a concrete language type directly.
pub trait LanguagePlugin: Send + Sync {
    fn detector(&self) -> &dyn LanguageDetector;
    fn analyzer(&self) -> &dyn SemanticAnalyzer;
    fn resolver(&self) -> &dyn DependencyResolver;
    fn constraints(&self) -> &dyn ClusteringConstraints;
    fn extractor(&self) -> &dyn SourceExtractor;
    fn rewriter(&self) -> &dyn DependencyRewriter;
    fn validator(&self) -> &dyn CompilerValidator;
}

/// Dispatches pipeline calls to the appropriate [`LanguagePlugin`] based on
/// file path.
///
/// Build the registry once at startup and share it throughout the pipeline:
/// ```rust,ignore
/// let registry = LanguageRegistry::new()
///     .register(Box::new(CppPlugin::new()))
///     .register(Box::new(ScalaPlugin::new()));
/// ```
pub struct LanguageRegistry {
    plugins: Vec<Box<dyn LanguagePlugin>>,
}

impl LanguageRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Appends `plugin` and returns `self` for builder-style chaining.
    pub fn register(mut self, plugin: Box<dyn LanguagePlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Returns the first plugin whose detector classifies `path` as non-`Other`.
    pub fn plugin_for(&self, path: &Path) -> Option<&dyn LanguagePlugin> {
        self.plugins
            .iter()
            .find(|p| p.detector().detect(path) != SourceFileKind::Other)
            .map(|p| p.as_ref())
    }

    /// Returns the detected [`SourceFileKind`] for `path`, or `Other` if no
    /// plugin recognises it.
    pub fn detect(&self, path: &Path) -> SourceFileKind {
        self.plugins
            .iter()
            .map(|p| p.detector().detect(path))
            .find(|k| *k != SourceFileKind::Other)
            .unwrap_or(SourceFileKind::Other)
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::ClusteringConstraints;
    use crate::extractor::{GeneratedSourceFile, SourceExtractor, SymbolBody};
    use crate::resolver::{DependencyResolver, RawDepEdge, ResolutionContext};
    use crate::rewriter::{DependencyRewriter, RewriteSpec};
    use ariadne_index::{SymbolId, SymbolRecord, TuAnalysis};
    use ariadne_repo::FileId;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    // ── minimal stubs ────────────────────────────────────────────────────────

    struct MockDetector;
    impl LanguageDetector for MockDetector {
        fn name(&self) -> &str {
            "mock"
        }
        fn detect(&self, path: &Path) -> SourceFileKind {
            match path.extension().and_then(|e| e.to_str()) {
                Some("mock") => SourceFileKind::Source,
                _ => SourceFileKind::Other,
            }
        }
    }

    struct MockAnalyzer;
    impl SemanticAnalyzer for MockAnalyzer {
        fn analyze(&self, file_id: FileId, _source: &str, _ctx: &str) -> TuAnalysis {
            TuAnalysis {
                tu_file_id: file_id,
                ..TuAnalysis::default()
            }
        }
    }

    struct MockResolver;
    impl DependencyResolver for MockResolver {
        fn resolve_edges(
            &self,
            _fid: FileId,
            _p: &Path,
            _src: &str,
            _ctx: &ResolutionContext<'_>,
        ) -> Vec<RawDepEdge> {
            vec![]
        }
    }

    struct MockConstraints;
    impl ClusteringConstraints for MockConstraints {
        fn colocated_groups(&self, _: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
            vec![]
        }
        fn frozen_symbols(&self, _: &[SymbolRecord]) -> HashSet<SymbolId> {
            HashSet::new()
        }
    }

    struct MockExtractor;
    impl SourceExtractor for MockExtractor {
        fn extract(&self, _: &[SymbolBody], _ns: &str, _deps: &[String]) -> GeneratedSourceFile {
            GeneratedSourceFile {
                path: PathBuf::from("out.mock"),
                content: String::new(),
            }
        }
    }

    struct MockRewriter;
    impl DependencyRewriter for MockRewriter {
        fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
            source.to_string()
        }
    }

    struct MockValidator;
    impl CompilerValidator for MockValidator {
        fn validate_compile(&self, _files: &[PathBuf], _args: &[String]) -> Result<(), String> {
            Ok(())
        }
    }

    struct MockPlugin {
        detector: MockDetector,
        analyzer: MockAnalyzer,
        resolver: MockResolver,
        constraints: MockConstraints,
        extractor: MockExtractor,
        rewriter: MockRewriter,
        validator: MockValidator,
    }

    impl MockPlugin {
        fn new() -> Self {
            Self {
                detector: MockDetector,
                analyzer: MockAnalyzer,
                resolver: MockResolver,
                constraints: MockConstraints,
                extractor: MockExtractor,
                rewriter: MockRewriter,
                validator: MockValidator,
            }
        }
    }

    impl LanguagePlugin for MockPlugin {
        fn detector(&self) -> &dyn LanguageDetector {
            &self.detector
        }
        fn analyzer(&self) -> &dyn SemanticAnalyzer {
            &self.analyzer
        }
        fn resolver(&self) -> &dyn DependencyResolver {
            &self.resolver
        }
        fn constraints(&self) -> &dyn ClusteringConstraints {
            &self.constraints
        }
        fn extractor(&self) -> &dyn SourceExtractor {
            &self.extractor
        }
        fn rewriter(&self) -> &dyn DependencyRewriter {
            &self.rewriter
        }
        fn validator(&self) -> &dyn CompilerValidator {
            &self.validator
        }
    }

    // ── tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn empty_registry_returns_none() {
        let registry = LanguageRegistry::new();
        assert!(registry.plugin_for(Path::new("src/main.cpp")).is_none());
        assert_eq!(
            registry.detect(Path::new("src/main.cpp")),
            SourceFileKind::Other
        );
    }

    #[test]
    fn registered_plugin_is_found_for_matching_extension() {
        let registry = LanguageRegistry::new().register(Box::new(MockPlugin::new()));
        assert!(registry.plugin_for(Path::new("src/main.mock")).is_some());
        assert_eq!(
            registry.detect(Path::new("src/main.mock")),
            SourceFileKind::Source
        );
    }

    #[test]
    fn registered_plugin_not_found_for_unknown_extension() {
        let registry = LanguageRegistry::new().register(Box::new(MockPlugin::new()));
        assert!(registry.plugin_for(Path::new("src/main.cpp")).is_none());
    }

    #[test]
    fn plugin_name_accessible_via_detector() {
        let registry = LanguageRegistry::new().register(Box::new(MockPlugin::new()));
        let plugin = registry.plugin_for(Path::new("src/main.mock")).unwrap();
        assert_eq!(plugin.detector().name(), "mock");
    }
}
