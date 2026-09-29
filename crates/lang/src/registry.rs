//! [`LanguagePlugin`] aggregate trait and [`LanguageRegistry`].

use crate::analyzer::SemanticAnalyzer;
use crate::constraints::ClusteringConstraints;
use crate::detector::{LanguageDetector, SourceFileKind};
use crate::extractor::SourceExtractor;
use crate::resolver::DependencyResolver;
use crate::rewriter::DependencyRewriter;
use crate::unit::UnitShaper;
use crate::validator::CompilerValidator;
use ariadne_lang_api::capability::LanguageCapabilities;
use ariadne_lang_api::identity::{language_of_file_kind, LanguageId};
use ariadne_lang_api::modeling::LanguageModeling;
use ariadne_repo::FileKind;
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

    /// Dispatch key for `plugin_for_id` and per-language cache scoping.
    fn id(&self) -> LanguageId;
    /// Replaces the ".h"/TU assumptions previously hardcoded in core crates.
    fn modeling(&self) -> LanguageModeling;
    /// Stage gating: disabled stages are reported, never silently empty.
    fn capabilities(&self) -> LanguageCapabilities;
    /// Ingredient of every cache fingerprint: bump when the plugin's parser changes.
    fn analysis_fingerprint(&self) -> String {
        "v1".into()
    }
    /// Unit shaping (build stage). `None` = use the built-in
    /// CMake/VS/compile_commands shaper.
    fn shaper(&self) -> Option<&dyn UnitShaper> {
        None
    }
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

    /// Returns the first plugin registered for exactly `id`.
    pub fn plugin_for_id(&self, id: LanguageId) -> Option<&dyn LanguagePlugin> {
        self.plugins.iter().find(|p| p.id() == id).map(|p| p.as_ref())
    }

    /// All plugin language ids, in registration order.
    pub fn languages(&self) -> Vec<LanguageId> {
        self.plugins.iter().map(|p| p.id()).collect()
    }

    /// Language of `path`, but only when BOTH the file kind maps to a real
    /// language (`FileKind::Other` ⇒ `SourceLanguage::Other` ⇒ `None`, plan
    /// §3.3: `Other` deliberately has no plugin) AND a registered plugin's
    /// detector claims the path. With several claiming plugins, the first
    /// registered wins (deterministic, registration-order precedence).
    pub fn detect_language(&self, path: &Path) -> Option<LanguageId> {
        if language_of_file_kind(FileKind::from_path(path)) == LanguageId::Other {
            return None;
        }
        self.plugin_for(path).map(|plugin| plugin.id())
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
    use crate::{
        AggregationKind, CyclePolicy, GuardKind, LanguageCapabilities, SymbolId, SymbolRecord,
        SymbolPlacement, TuAnalysis, UnitKind,
    };
    use ariadne_repo::FileId;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    // ── minimal stubs ────────────────────────────────────────────────────────

    struct MockDetector {
        extension: &'static str,
    }

    impl MockDetector {
        fn new(extension: &'static str) -> Self {
            Self { extension }
        }
    }

    impl LanguageDetector for MockDetector {
        fn name(&self) -> &str {
            "mock"
        }
        fn detect(&self, path: &Path) -> SourceFileKind {
            match path.extension().and_then(|e| e.to_str()) {
                Some(ext) if ext == self.extension => SourceFileKind::Source,
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
        language: LanguageId,
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
            Self::with_extension("mock", LanguageId::Cpp)
        }

        fn with_extension(extension: &'static str, language: LanguageId) -> Self {
            Self {
                language,
                detector: MockDetector::new(extension),
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
        fn id(&self) -> LanguageId {
            self.language
        }
        fn modeling(&self) -> LanguageModeling {
            LanguageModeling {
                language: self.language,
                unit: UnitKind::SourceFile,
                aggregation: AggregationKind::None,
                guard: GuardKind::None,
                symbol_placement: SymbolPlacement::SingleFile,
                aggregation_reexports: false,
                cycle_policy: CyclePolicy::Tolerate,
                external_path_markers: vec![],
            }
        }
        fn capabilities(&self) -> LanguageCapabilities {
            LanguageCapabilities {
                discovery: true,
                dependency_graph: true,
                semantic_index: true,
                load_metric: true,
                planning: false,
                extraction: false,
                rewriting: false,
                validation: false,
                requires_toolchain: false,
            }
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

    #[test]
    fn plugin_for_id_hits_registered_and_misses_unknown() {
        let registry = LanguageRegistry::new()
            .register(Box::new(MockPlugin::with_extension("scala", LanguageId::Scala)));
        assert!(registry.plugin_for_id(LanguageId::Scala).is_some());
        assert_eq!(registry.plugin_for_id(LanguageId::Scala).unwrap().id(), LanguageId::Scala);
        assert!(registry.plugin_for_id(LanguageId::Cpp).is_none());
    }

    #[test]
    fn languages_lists_registration_order() {
        let registry = LanguageRegistry::new()
            .register(Box::new(MockPlugin::with_extension("scala", LanguageId::Scala)))
            .register(Box::new(MockPlugin::with_extension("cpp", LanguageId::Cpp)));
        assert_eq!(
            registry.languages(),
            vec![LanguageId::Scala, LanguageId::Cpp]
        );
    }

    #[test]
    fn detect_language_first_registered_claiming_plugin_wins() {
        let first = MockPlugin::with_extension("scala", LanguageId::Scala);
        let second = MockPlugin::with_extension("scala", LanguageId::Cpp);
        let registry = LanguageRegistry::new()
            .register(Box::new(first))
            .register(Box::new(second));
        assert_eq!(
            registry.detect_language(Path::new("src/Main.scala")),
            Some(LanguageId::Scala)
        );
    }

    #[test]
    fn detect_language_yields_none_when_file_kind_maps_to_other() {
        // `.mock` is not a recognised FileKind ⇒ SourceLanguage::Other, and
        // Other deliberately has no plugin even though this plugin's
        // detector claims the extension.
        let registry = LanguageRegistry::new().register(Box::new(MockPlugin::new()));
        assert!(registry.plugin_for(Path::new("src/main.mock")).is_some());
        assert_eq!(registry.detect_language(Path::new("src/main.mock")), None);
        assert_eq!(registry.detect_language(Path::new("notes.txt")), None);
    }
}
