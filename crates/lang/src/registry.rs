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
use ariadne_lang_api::identity::LanguageId;
use ariadne_lang_api::modeling::LanguageModeling;
use std::fs;
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

/// Language of `path`, decided by the registered plugin's detector.
///
/// M24 Part A, D-A2 -- plugin detection is AUTHORITATIVE. This used to
/// require BOTH that the closed `FileKind` table map to a real language AND
/// that a registered plugin's detector claim the path, returning `None`
/// whenever the table said `FileKind::Other`. That veto is what made a third
/// language unreachable: no closed table can name it, so a third language's
/// files were refused outright. The plugin now decides, and `None` means
/// exactly one thing -- **no registered plugin claims this path**.
///
/// With several claiming plugins the FIRST registered wins (deterministic,
/// registration-order precedence; invariant I3, locked by
/// `detect_language_first_registered_claiming_plugin_wins`).
///
/// Behaviour for C++ and Scala repositories is unchanged: every path those
/// detectors claim was already classified by the `FileKind` table, so the
/// removed branch never fired for them.
pub fn detect_language(&self, path: &Path) -> Option<LanguageId> {
    self.plugin_for(path).map(|plugin| plugin.id())
}

    /// Build-stage unit shaper for registry-driven discovery (plan M18.3):
    /// the single registered plugin with a shaper — today exactly one — wins.
    /// With several shaper-providing plugins the FIRST registered wins as a
    /// deterministic placeholder; marker-based precedence (build markers
    /// matched against the repository) is M18.5.
    pub fn unit_shaper_for(&self) -> Option<&dyn UnitShaper> {
        self.plugins.iter().find_map(|p| p.shaper())
    }

    /// Build-stage unit shaper selected from the repository root
    /// (plan M22 D22): the FIRST plugin in registration order that
    /// provides a shaper declaring a `build_markers()` entry present at
    /// `repo_root` wins; when NO plugin's markers are present the
    /// registry falls back to first-wins [`Self::unit_shaper_for`] —
    /// today's behavior, byte-identical. Selection differs from
    /// first-wins ONLY where a later plugin's markers are present while
    /// every earlier plugin's are absent.
    ///
    /// Marker presence rule (honest statement): a marker names an entry
    /// (file or directory) of the repository root. A non-dot marker
    /// (e.g. `CMakeLists.txt`) is present only on an exact-name match; a
    /// dot-prefixed marker (e.g. `.vcxproj`) is a SUFFIX-style project
    /// file extension and is present when ANY root entry name ends with
    /// it (so `Foo.vcxproj` matches `.vcxproj`). Registry iteration is
    /// pure data — no language identity is referenced (R2/R3).
    pub fn unit_shaper_for_repo(&self, repo_root: &Path) -> Option<&dyn UnitShaper> {
        self.plugins
            .iter()
            .find_map(|plugin| {
                let shaper = plugin.shaper()?;
                has_root_marker(repo_root, &shaper.build_markers()).then_some(shaper)
            })
            .or_else(|| self.unit_shaper_for())
    }

    /// The only registered plugin (M18.3): `Some` exactly when the registry
    /// holds one plugin, backing the single-language call sites (e.g.
    /// `IncrementalAnalyzer`'s one injected analyzer/resolver) without a
    /// hidden C++ default. Multi-language registries dispatch per unit via
    /// `detect_language`/`plugin_for_id` instead (`IncrementalAnalyzer`'s
    /// per-language split lands with M19).
    pub fn sole_plugin(&self) -> Option<&dyn LanguagePlugin> {
        if self.plugins.len() == 1 {
            Some(self.plugins[0].as_ref())
        } else {
            None
        }
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// True when any of `markers` is present at `repo_root` — presence rule
/// documented on [`LanguageRegistry::unit_shaper_for_repo`].
fn has_root_marker(repo_root: &Path, markers: &[&'static str]) -> bool {
    markers
        .iter()
        .any(|marker| root_marker_present(repo_root, marker))
}

fn root_marker_present(repo_root: &Path, marker: &str) -> bool {
    if repo_root.join(marker).exists() {
        return true;
    }
    marker.starts_with('.') && root_entries_end_with(repo_root, marker)
}

fn root_entries_end_with(repo_root: &Path, suffix: &str) -> bool {
    fs::read_dir(repo_root)
        .map(|entries| {
            entries.flatten().any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .ends_with(suffix)
            })
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::ClusteringConstraints;
    use crate::extractor::{GeneratedSourceFile, SourceExtractor, SymbolBody};
    use crate::resolver::{DependencyResolver, RawDepEdge, ResolutionContext};
    use crate::rewriter::{DependencyRewriter, RewriteSpec};
    use crate::{
        AggregationKind, CyclePolicy, DependencyVocabulary, GuardKind, LanguageCapabilities,
        SymbolId, SymbolRecord, SymbolPlacement, TuAnalysis, UnitKind,
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

    struct MockShaper {
        markers: &'static [&'static str],
        identity: &'static str,
    }

    impl UnitShaper for MockShaper {
        fn build_markers(&self) -> Vec<&'static str> {
            self.markers.to_vec()
        }
        // The shaper's identity surfaces through the compiler name so marker
        // selection tests can tell one plugin's shaper from another's.
        fn default_compiler(&self) -> &str {
            self.identity
        }
        fn shape_units(
            &self,
            _repo_root: &Path,
            _files: &[ariadne_repo::FileRecord],
        ) -> Vec<crate::UnitDescriptor> {
            vec![]
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
        shaper: Option<MockShaper>,
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
                shaper: None,
            }
        }

        fn with_marker_shaper(
            extension: &'static str,
            language: LanguageId,
            markers: &'static [&'static str],
            identity: &'static str,
        ) -> Self {
            let mut plugin = Self::with_extension(extension, language);
            plugin.shaper = Some(MockShaper {
                markers,
                identity,
            });
            plugin
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
            self.language.clone()
        }
        fn modeling(&self) -> LanguageModeling {
            LanguageModeling {
                language: self.language.clone(),
                unit: UnitKind::SourceFile,
                aggregation: AggregationKind::None,
                guard: GuardKind::None,
                symbol_placement: SymbolPlacement::SingleFile,
                aggregation_reexports: false,
                cycle_policy: CyclePolicy::Tolerate,
                external_path_markers: vec![],
                // D26 fields: default-path wording, no caveat (the mock
                // makes no wording assertions).
                dependency_vocabulary: DependencyVocabulary {
                    directive_label: "Include".into(),
                    directive_passive_verb: "included".into(),
                    aggregation_noun: "header".into(),
                },
                analysis_caveat: None,
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
        fn shaper(&self) -> Option<&dyn UnitShaper> {
            self.shaper
                .as_ref()
                .map(|shaper| shaper as &dyn UnitShaper)
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
    fn detect_language_defers_to_the_claiming_plugin_not_the_file_kind_table() {
        // M24 Part A, D-A2 -- CONTRACT CHANGE, not a bug fix. `.mock` is not a
        // recognised FileKind, so `FileKind::from_path` sends it to
        // `FileKind::Other`. Before this milestone `detect_language` consulted
        // that closed table first and returned `None`, discarding the answer of
        // a plugin whose detector demonstrably claims the extension -- so a
        // third language's files were refused rather than mislabelled. The
        // registered plugin's detector is now the authority.
        let registry = LanguageRegistry::new().register(Box::new(MockPlugin::new()));
        assert!(registry.plugin_for(Path::new("src/main.mock")).is_some());
        assert_eq!(
            registry.detect_language(Path::new("src/main.mock")),
            Some(LanguageId::Cpp),
            "the claiming plugin's identity is the answer; the FileKind spine no longer vetoes it"
        );
        // A path NO plugin claims still has no language -- and that is now the
        // only way to get `None`.
        assert_eq!(registry.detect_language(Path::new("notes.txt")), None);
    }

    #[test]
    fn other_is_never_a_plugin_identity_and_never_a_detection_result() {
        // D-A3: `Other` is legacy-only. It stays the `#[serde(default)]` value so
        // pre-existing artifacts read as C++-compatible, but it is never a
        // plugin's `id()` and never a `detect_language` answer.
        let registry = LanguageRegistry::new()
            .register(Box::new(MockPlugin::with_extension("scala", LanguageId::Scala)));
        assert!(
            registry.plugin_for_id(LanguageId::Other).is_none(),
            "no plugin is registered for the legacy sentinel"
        );
        assert_eq!(
            registry.detect_language(Path::new("src/Main.scala")),
            Some(LanguageId::Scala),
            "the claimed path resolves to the plugin's own identity, not Other"
        );
        assert_eq!(
            registry.detect_language(Path::new("notes.txt")),
            None,
            "an unclaimed path has no language rather than a wrong one"
        );
    }

    #[test]
    fn unit_shaper_for_tracks_plugin_shapers() {
        // MockPlugin never overrides `shaper()`, so the trait default (None)
        // must surface as "no shaper" rather than a hidden C++ fallback.
        assert!(LanguageRegistry::new().unit_shaper_for().is_none());
        let registry = LanguageRegistry::new().register(Box::new(MockPlugin::new()));
        assert!(registry.unit_shaper_for().is_none());
    }

    #[test]
    fn sole_plugin_requires_exactly_one_registered_plugin() {
        assert!(LanguageRegistry::new().sole_plugin().is_none());
        let one = LanguageRegistry::new().register(Box::new(MockPlugin::new()));
        assert_eq!(one.sole_plugin().unwrap().id(), LanguageId::Cpp);
        let two = one.register(Box::new(MockPlugin::with_extension("scala", LanguageId::Scala)));
        assert!(two.sole_plugin().is_none());
    }

    // ── unit_shaper_for_repo (plan M22 D22: marker-based selection) ──────────

    const FIRST_MARKER: &str = "alpha.marker";
    const SECOND_MARKER: &str = "beta.marker";

    fn marker_shaper_registry() -> LanguageRegistry {
        LanguageRegistry::new()
            .register(Box::new(MockPlugin::with_marker_shaper(
                "one",
                LanguageId::Cpp,
                &[FIRST_MARKER],
                "shaper-one",
            )))
            .register(Box::new(MockPlugin::with_marker_shaper(
                "two",
                LanguageId::Scala,
                &[SECOND_MARKER],
                "shaper-two",
            )))
    }

    fn selected_shaper_identity(registry: &LanguageRegistry, root: &Path) -> String {
        registry
            .unit_shaper_for_repo(root)
            .expect("a shaper-providing plugin is registered")
            .default_compiler()
            .to_string()
    }

    fn repo_root_with_markers(markers: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp repo root");
        for marker in markers {
            std::fs::write(dir.path().join(marker), "").expect("write marker");
        }
        dir
    }

    #[test]
    fn unit_shaper_for_repo_selects_the_later_plugin_when_only_its_marker_is_present() {
        let registry = marker_shaper_registry();
        let root = repo_root_with_markers(&[SECOND_MARKER]);
        assert_eq!(
            selected_shaper_identity(&registry, root.path()),
            "shaper-two",
            "the later-registered plugin wins when its marker is present \
             and the first plugin's is absent (D22)"
        );
    }

    #[test]
    fn unit_shaper_for_repo_registration_order_wins_when_both_markers_are_present() {
        let registry = marker_shaper_registry();
        let root = repo_root_with_markers(&[FIRST_MARKER, SECOND_MARKER]);
        assert_eq!(
            selected_shaper_identity(&registry, root.path()),
            "shaper-one",
            "registration order breaks the marker tie (D22)"
        );
    }

    #[test]
    fn unit_shaper_for_repo_selects_the_first_plugin_when_only_its_marker_is_present() {
        let registry = marker_shaper_registry();
        let root = repo_root_with_markers(&[FIRST_MARKER]);
        assert_eq!(
            selected_shaper_identity(&registry, root.path()),
            "shaper-one",
            "the first plugin's own marker selects its shaper"
        );
    }

    #[test]
    fn unit_shaper_for_repo_falls_back_to_first_wins_when_no_marker_is_present() {
        let registry = marker_shaper_registry();
        let root = repo_root_with_markers(&[]);
        assert_eq!(
            selected_shaper_identity(&registry, root.path()),
            "shaper-one",
            "no marker present anywhere ⇒ today's first-wins unit_shaper_for() \
             behavior (IV2 fallback)"
        );
    }

    #[test]
    fn unit_shaper_for_repo_treats_dot_markers_as_name_suffixes() {
        let registry = LanguageRegistry::new().register(Box::new(
            MockPlugin::with_marker_shaper("one", LanguageId::Cpp, &[".proj"], "shaper-dot"),
        ));
        let root = repo_root_with_markers(&["Anything.proj"]);
        assert_eq!(
            selected_shaper_identity(&registry, root.path()),
            "shaper-dot",
            "a dot-prefixed marker matches any root entry ending in it"
        );
    }

    #[test]
    fn unit_shaper_for_repo_skips_plugins_without_a_shaper() {
        let registry = LanguageRegistry::new()
            .register(Box::new(MockPlugin::with_extension("plain", LanguageId::Cpp)))
            .register(Box::new(MockPlugin::with_marker_shaper(
                "two",
                LanguageId::Scala,
                &[SECOND_MARKER],
                "shaper-two",
            )));
        let root = repo_root_with_markers(&[SECOND_MARKER]);
        assert_eq!(
            selected_shaper_identity(&registry, root.path()),
            "shaper-two",
            "a shaper-less plugin can never win marker selection"
        );
    }

    #[test]
    fn unit_shaper_for_repo_empty_registry_is_none() {
        let root = repo_root_with_markers(&[FIRST_MARKER]);
        assert!(
            LanguageRegistry::new()
                .unit_shaper_for_repo(root.path())
                .is_none()
        );
    }
}
