use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::queue::TuTask;
use ariadne_index::TuAnalysis;
use ariadne_lang::LanguageRegistry;

/// Registry-backed backends must outlive the tokio worker tasks that hold
/// them (`WorkerPool` requires `B: 'static`), so the shared-ownership variant
/// of [`PluginWorkerBackend`] carries the registry as an [`Arc`] (plan M18.3
/// cli wiring). Dispatch, byte-parity, and the unhandled-language error are
/// identical to the borrowed variant — it just delegates.
pub struct SharedPluginWorkerBackend {
    registry: Arc<LanguageRegistry>,
}

impl SharedPluginWorkerBackend {
    pub fn new(registry: Arc<LanguageRegistry>) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &LanguageRegistry {
        &self.registry
    }
}

impl WorkerBackend for SharedPluginWorkerBackend {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        PluginWorkerBackend::new(&self.registry).analyze_task(task, repo_root)
    }
}

/// `WorkerPool` stores backends as `Arc<B>`, and the registry wiring hands it
/// an `Arc<dyn WorkerBackend>` (M18.3): dispatch through the `Arc` to the
/// wrapped backend.
impl<T: WorkerBackend + ?Sized> WorkerBackend for Arc<T> {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        T::analyze_task(self, task, repo_root)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerStatus {
    Idle,
    Busy(usize),
    Failed(String),
}

pub trait WorkerBackend: Send + Sync {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis>;
}

/// Registry-dispatching worker backend (plan M18.2; since M18.11 the ONLY
/// in-process backend): the analyzer and the closure walk are resolved from
/// [`LanguageRegistry`] by the unit's language. Dispatch mechanism:
/// `registry.detect_language(&task.tu.source_path)` — the extension →
/// `FileKind` mapping lives in `ariadne-repo` (no extension literals here)
/// and each plugin's detector must claim the path.
///
/// Byte-for-byte equal to the pre-M18 hardcoded-C++ in-process backend for
/// tasks whose language maps to a registered plugin (locked by
/// `m18_cpp_plugin_parity_test`): the body-only path calls the plugin's
/// `SemanticAnalyzer` (for C++ the three-line adapter over the same
/// analyzer), and a task carrying a store context runs the canonical
/// `tu_store` closure merge through the plugin's resolver/analyzer. An
/// unhandled language is reported as an error instead of being silently
/// parsed (plan §3.3 fact 3).
pub struct PluginWorkerBackend<'a> {
    pub registry: &'a LanguageRegistry,
}

impl<'a> PluginWorkerBackend<'a> {
    pub fn new(registry: &'a LanguageRegistry) -> Self {
        Self { registry }
    }
}

impl WorkerBackend for PluginWorkerBackend<'_> {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        let language = self
            .registry
            .detect_language(&task.tu.source_path)
            .ok_or_else(|| anyhow::anyhow!("unhandled language for {:?}", task.tu.source_path))?;
        let plugin = self
            .registry
            .plugin_for_id(language.clone())
            .ok_or_else(|| anyhow::anyhow!("no plugin registered for language {language:?}"))?;
        if let Some(ctx) = &task.context {
            // Canonical closure merge through the unit's own plugin — the
            // C++ bytes match the pre-M18 hardcoded merge exactly.
            let (analysis, _fingerprint) =
                crate::tu_store::analyze_tu_full_with_plugin(&task.tu, repo_root, ctx, plugin)?;
            return Ok(analysis);
        }
        let full_path = repo_root.join(&task.tu.source_path);
        let source_code = std::fs::read_to_string(&full_path).unwrap_or_default();
        Ok(plugin
            .analyzer()
            .analyze(task.tu.file_id.clone(), &source_code, &task.tu.context_name))
    }
}

// The pre-M18 clang-worker sidecar backend relocated to
// `ariadne-lang-cpp` (`ariadne_lang_cpp::ClangWorkerBackend`) in M18.11, as
// sanctioned by the M18.1–M18.2 execution note (b): the relocation needs
// `lang-cpp -> scheduler` as a normal dependency, which only becomes acyclic
// once the `index`/`graph -> lang-cpp` transitional shim deps are removed.

#[cfg(test)]
mod dispatch_contract_tests {
    //! AR-16 characterization locks for every dispatch arm of
    //! [`PluginWorkerBackend::analyze_task`] (audit finding T3: this dispatch
    //! had zero direct tests; its error contract was locked only indirectly
    //! by the C++/Scala parity suites).
    //!
    //! Handoff item mapping (handoff wording -> dispatch arm under lock):
    //! * missing-language dispatch (M24 third-language error contract) ->
    //!   the `detect_language` arm; wording locked against the integration
    //!   suites that already pin it (`m24_language_identity_test` asserts the
    //!   error contains "unhandled language" and never "Cpp";
    //!   `m18_cpp_plugin_parity::worker_backend_parity` asserts the same
    //!   phrase), never redefined here.
    //! * unknown-task path -> the `plugin_for_id` arm: dispatch answers a
    //!   language it cannot produce a plugin for. With a consistent registry
    //!   this arm is unreachable by construction (`detect_language` derives
    //!   the id FROM the claiming plugin, which `plugin_for_id` then finds),
    //!   so it is characterized through the only observable shape of that
    //!   disagreement: `SelfRenamingIdPlugin` below.
    //! * cancel path -> the store-context arm: a task carrying a store
    //!   context CANCELS the body-only analysis branch and must run the
    //!   canonical closure merge through the plugin's resolver/analyzer
    //!   instead. (There is no handle-abort cancellation inside a single
    //!   `analyze_task` call; the pool-level break paths already have their
    //!   own characterization in `pool.rs`.)

    use super::*;
    use crate::tu_store::{SharedTuStoreContext, TuStoreContext};
    use ariadne_build::{CompileCommand, TranslationUnit};
    use ariadne_lang::{
        AggregationKind, ClusteringConstraints, CompilerValidator, CyclePolicy, DependencyResolver,
        DependencyRewriter, DependencyVocabulary, FileCatalog, GeneratedSourceFile, GuardKind,
        LanguageCapabilities, LanguageDetector, LanguageId, LanguageModeling, LanguagePlugin,
        RawDepEdge, ResolveOutcome, ResolutionContext, RewriteSpec, SemanticAnalyzer,
        SourceExtractor, SourceFileKind, SymbolBody, SymbolPlacement, SymbolRecord, UnitKind,
    };
    use ariadne_repo::{
        FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    /// Extension the mock plugin's detector claims - a language identity no
    /// shipped plugin knows, mirroring the M24 third-language test shape.
    const MOCK_EXTENSION: &str = "mocklang";
    const MOCK_LANGUAGE: &str = "mocklang";
    /// Identity `SelfRenamingIdPlugin` answers AFTER its first `id()` call.
    const RETIRED_LANGUAGE: &str = "mocklang-retired";

    fn mock_language() -> LanguageId {
        LanguageId::new(MOCK_LANGUAGE)
    }

    fn make_tu(source_rel: &str) -> TranslationUnit {
        TranslationUnit {
            file_id: FileId::from_relative_path(source_rel),
            source_path: PathBuf::from(source_rel),
            context_name: "dispatch-contract".to_string(),
            compile_command: CompileCommand {
                directory: PathBuf::new(),
                file: PathBuf::from(source_rel),
                command: None,
                arguments: Vec::new(),
                context_name: "dispatch-contract".to_string(),
                output: None,
            },
            language: Default::default(),
        }
    }

    fn tu_task(
        task_id: usize,
        source_rel: &str,
        context: Option<SharedTuStoreContext>,
    ) -> TuTask {
        TuTask {
            task_id,
            tu: make_tu(source_rel),
            estimated_cost: 1,
            context,
        }
    }

    fn write_repo_file(root: &std::path::Path, rel: &str, content: &str) {
        let full = root.join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("create fixture directory");
        }
        std::fs::write(&full, content).expect("write fixture file");
    }

    fn discovery_of(root: &std::path::Path, rels: &[&str]) -> ariadne_repo::RepositoryDiscoveryResult {
        let files = rels
            .iter()
            .map(|rel| FileRecord {
                id: FileId::from_relative_path(rel),
                path: PathBuf::from(rel),
                canonical_path: root.join(rel),
                kind: FileKind::Other,
                size: 0,
                ownership: FileOwnership::Repository,
                optimization_policy: OptimizationPolicy::Optimizable,
                content_hash: "dispatch-contract-hash".to_string(),
                language: SourceLanguage::Other,
            })
            .collect();
        ariadne_repo::RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files,
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        }
    }

    // ── mock plugin: neutral, records every analysis it is asked to run ──────

    struct MockDetector;

    impl LanguageDetector for MockDetector {
        fn name(&self) -> &str {
            MOCK_LANGUAGE
        }
        fn detect(&self, path: &std::path::Path) -> SourceFileKind {
            match path.extension().and_then(|ext| ext.to_str()) {
                Some(ext) if ext.eq_ignore_ascii_case(MOCK_EXTENSION) => SourceFileKind::Source,
                _ => SourceFileKind::Other,
            }
        }
    }

    /// Records every `analyze` call so tests can prove dispatch aborted
    /// before running any analysis, or visited exactly the closure files.
    struct RecordingAnalyzer {
        analyzed_files: Arc<Mutex<Vec<FileId>>>,
    }

    impl RecordingAnalyzer {
        fn with_shared_recorder() -> (Self, Arc<Mutex<Vec<FileId>>>) {
            let recorder = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    analyzed_files: Arc::clone(&recorder),
                },
                recorder,
            )
        }
    }

    impl SemanticAnalyzer for RecordingAnalyzer {
        fn analyze(
            &self,
            file_id: FileId,
            _source: &str,
            _context: &str,
        ) -> ariadne_lang::TuAnalysis {
            self.analyzed_files.lock().unwrap().push(file_id);
            ariadne_lang::TuAnalysis {
                tu_file_id: file_id,
                ..Default::default()
            }
        }
    }

    /// Resolver fixture for the closure-merge arm: the root unit declares one
    /// directive and every directive resolves to the one header file.
    struct RootToHeaderResolver {
        root: FileId,
        header: FileId,
    }

    impl DependencyResolver for RootToHeaderResolver {
        fn resolve_edges(
            &self,
            file_id: FileId,
            rel_path: &std::path::Path,
            _source: &str,
            _context: &ResolutionContext<'_>,
        ) -> Vec<RawDepEdge> {
            if file_id != self.root {
                return Vec::new();
            }
            vec![RawDepEdge {
                source_file: file_id,
                source_rel_path: rel_path.to_path_buf(),
                raw_target: "ghost.mockinc".to_string(),
                is_wildcard: false,
                style: ariadne_lang::DirectiveStyle::Quoted,
                line: 1,
                column: 1,
            }]
        }

        fn resolve_target(
            &self,
            _edge: &RawDepEdge,
            _catalog: &FileCatalog,
            _context: &ResolutionContext<'_>,
        ) -> ResolveOutcome {
            ResolveOutcome::Resolved(self.header)
        }
    }

    struct MockConstraints;

    impl ClusteringConstraints for MockConstraints {
        fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<ariadne_lang::SymbolId>> {
            Vec::new()
        }
        fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> std::collections::HashSet<ariadne_lang::SymbolId> {
            std::collections::HashSet::new()
        }
    }

    struct MockExtractor;

    impl SourceExtractor for MockExtractor {
        fn extract(&self, _bodies: &[SymbolBody], _ns: &str, _deps: &[String]) -> GeneratedSourceFile {
            GeneratedSourceFile {
                path: PathBuf::from("out.mocklang"),
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
        fn validate_compile(
            &self,
            _files: &[PathBuf],
            _args: &[String],
        ) -> Result<(), String> {
            Ok(())
        }
    }

    fn mock_modeling() -> LanguageModeling {
        LanguageModeling {
            language: mock_language(),
            unit: UnitKind::SourceFile,
            aggregation: AggregationKind::PackageObject,
            guard: GuardKind::None,
            symbol_placement: SymbolPlacement::SingleFile,
            aggregation_reexports: false,
            cycle_policy: CyclePolicy::TolerateWithinPackage,
            external_path_markers: Vec::new(),
            dependency_vocabulary: DependencyVocabulary {
                directive_label: "Directive".to_string(),
                directive_passive_verb: "directived".to_string(),
                aggregation_noun: "aggregation".to_string(),
            },
            analysis_caveat: None,
        }
    }

    fn mock_capabilities() -> LanguageCapabilities {
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

    struct MockLangPlugin {
        analyzer: RecordingAnalyzer,
        resolver: RootToHeaderResolver,
    }

    impl LanguagePlugin for MockLangPlugin {
        fn detector(&self) -> &dyn LanguageDetector {
            &MockDetector
        }
        fn analyzer(&self) -> &dyn SemanticAnalyzer {
            &self.analyzer
        }
        fn resolver(&self) -> &dyn DependencyResolver {
            &self.resolver
        }
        fn constraints(&self) -> &dyn ClusteringConstraints {
            &MockConstraints
        }
        fn extractor(&self) -> &dyn SourceExtractor {
            &MockExtractor
        }
        fn rewriter(&self) -> &dyn DependencyRewriter {
            &MockRewriter
        }
        fn validator(&self) -> &dyn CompilerValidator {
            &MockValidator
        }
        fn id(&self) -> LanguageId {
            mock_language()
        }
        fn modeling(&self) -> LanguageModeling {
            mock_modeling()
        }
        fn capabilities(&self) -> LanguageCapabilities {
            mock_capabilities()
        }
    }

    /// Reproduces the only observable shape of dispatch's `plugin_for_id`
    /// error arm: detection answers a language, but by the time dispatch
    /// looks that language up no plugin carries it any more (the first
    /// `id()` call - made by `detect_language` - and the comparison made by
    /// `plugin_for_id` disagree). With a consistent registry this arm is
    /// unreachable by construction; this mock pins the arm's CONTRACT, not a
    /// real plugin's behavior: a clear error and zero analysis, never a
    /// silent fall-through to some other plugin's analysis.
    struct SelfRenamingIdPlugin {
        inner: MockLangPlugin,
        first_answer_pending: AtomicBool,
    }

    impl LanguagePlugin for SelfRenamingIdPlugin {
        fn detector(&self) -> &dyn LanguageDetector {
            self.inner.detector()
        }
        fn analyzer(&self) -> &dyn SemanticAnalyzer {
            self.inner.analyzer()
        }
        fn resolver(&self) -> &dyn DependencyResolver {
            self.inner.resolver()
        }
        fn constraints(&self) -> &dyn ClusteringConstraints {
            self.inner.constraints()
        }
        fn extractor(&self) -> &dyn SourceExtractor {
            self.inner.extractor()
        }
        fn rewriter(&self) -> &dyn DependencyRewriter {
            self.inner.rewriter()
        }
        fn validator(&self) -> &dyn CompilerValidator {
            self.inner.validator()
        }
        fn id(&self) -> LanguageId {
            if self.first_answer_pending.swap(false, Ordering::SeqCst) {
                mock_language()
            } else {
                LanguageId::new(RETIRED_LANGUAGE)
            }
        }
        fn modeling(&self) -> LanguageModeling {
            self.inner.modeling()
        }
        fn capabilities(&self) -> LanguageCapabilities {
            self.inner.capabilities()
        }
    }

    // ── 1. missing-language dispatch: the M24 third-language contract ─────────

    #[test]
    fn unclaimed_third_language_task_errors_as_unhandled_and_never_default_analyzes() {
        // Wording locked against, not redefined by, the integration suites:
        // m24_language_identity_test asserts contains "unhandled language"
        // and never "Cpp"; m18 worker_backend_parity asserts the same phrase.
        let registry =
            LanguageRegistry::new().register(Box::new(ariadne_lang_cpp::CppPlugin::new()));
        let backend = PluginWorkerBackend::new(&registry);
        let repo = tempfile::tempdir().expect("repo tempdir");
        write_repo_file(repo.path(), "src/solver.f90", "program ghost\nend program\n");
        write_repo_file(repo.path(), "src/main.cpp", "class Alpha {};\n");

        let unclaimed = tu_task(6001, "src/solver.f90", None);
        let err = backend
            .analyze_task(&unclaimed, repo.path())
            .expect_err("a unit in a language no plugin claims must error, never parse");
        let err_text = err.to_string();
        assert!(
            err_text.contains("unhandled language"),
            "error must name the failure honestly: {err_text}"
        );
        assert!(
            err_text.contains("solver.f90"),
            "error must carry the unclaimed path: {err_text}"
        );
        assert!(
            !err_text.contains("Cpp"),
            "and never name a default language: {err_text}"
        );

        // Positive control through the same registry and backend: a claimed
        // unit dispatches fine, so the error above is the language verdict,
        // not a broken test harness.
        let claimed = tu_task(6002, "src/main.cpp", None);
        let analysis = backend
            .analyze_task(&claimed, repo.path())
            .expect("a registered language must dispatch");
        assert_eq!(analysis.tu_file_id, FileId::from_relative_path("src/main.cpp"));
    }

    #[test]
    fn shared_and_arc_delegate_backends_carry_the_unhandled_language_error_verbatim() {
        // The two wrapper backends document "identical ... just delegates";
        // a refactor that drops or diverges the error path through either
        // wrapper fails here, not in the CLI.
        let registry =
            Arc::new(LanguageRegistry::new().register(Box::new(ariadne_lang_cpp::CppPlugin::new())));
        let repo = tempfile::tempdir().expect("repo tempdir");
        let unclaimed = tu_task(6101, "src/solver.f90", None);

        let borrowed_text = PluginWorkerBackend::new(&registry)
            .analyze_task(&unclaimed, repo.path())
            .expect_err("borrowed variant must reject the unclaimed language")
            .to_string();
        let shared = SharedPluginWorkerBackend::new(Arc::clone(&registry));
        let shared_text = shared
            .analyze_task(&unclaimed, repo.path())
            .expect_err("shared variant must reject the unclaimed language")
            .to_string();
        let dyn_backend: Arc<dyn WorkerBackend> =
            Arc::new(SharedPluginWorkerBackend::new(Arc::clone(&registry)));
        let dyn_text = dyn_backend
            .analyze_task(&unclaimed, repo.path())
            .expect_err("Arc<dyn> dispatch must reject the unclaimed language")
            .to_string();

        assert!(
            borrowed_text.contains("unhandled language"),
            "the borrowed-variant phrase is the contract: {borrowed_text}"
        );
        assert_eq!(
            shared_text, borrowed_text,
            "SharedPluginWorkerBackend must delegate the error unchanged"
        );
        assert_eq!(
            dyn_text, borrowed_text,
            "dispatch through Arc<dyn WorkerBackend> must delegate the error unchanged"
        );
    }

    // ── 2. unknown-task path: answered language, producible plugin missing ───

    #[test]
    fn answered_language_without_a_producible_plugin_errors_before_any_analysis() {
        let root_id = FileId::from_relative_path("src/ghost.mocklang");
        let header_id = FileId::from_relative_path("include/ghost.mockinc");
        let (analyzer, analyzed_files) = RecordingAnalyzer::with_shared_recorder();
        let plugin = SelfRenamingIdPlugin {
            inner: MockLangPlugin {
                analyzer,
                resolver: RootToHeaderResolver {
                    root: root_id,
                    header: header_id,
                },
            },
            first_answer_pending: AtomicBool::new(true),
        };
        let registry = LanguageRegistry::new().register(Box::new(plugin));
        let backend = PluginWorkerBackend::new(&registry);
        let repo = tempfile::tempdir().expect("repo tempdir");

        let task = tu_task(6201, "src/ghost.mocklang", None);
        let err = backend
            .analyze_task(&task, repo.path())
            .expect_err("a language with no producible plugin must error, never fall through");
        let err_text = err.to_string();
        assert!(
            err_text.contains("no plugin registered for language"),
            "error must name the missing plugin honestly: {err_text}"
        );
        assert!(
            err_text.contains(MOCK_LANGUAGE),
            "error must carry the answered language: {err_text}"
        );
        assert!(
            analyzed_files.lock().unwrap().is_empty(),
            "dispatch must abort before running any analyzer"
        );
    }

    // ── 3. cancel path: a store context cancels the body-only branch ─────────

    #[test]
    fn store_context_task_cancels_the_body_only_branch_and_runs_the_closure_merge() {
        let root_rel = "src/ghost.mocklang";
        let header_rel = "include/ghost.mockinc";
        let root_id = FileId::from_relative_path(root_rel);
        let header_id = FileId::from_relative_path(header_rel);
        let (analyzer, analyzed_files) = RecordingAnalyzer::with_shared_recorder();
        let plugin = MockLangPlugin {
            analyzer,
            resolver: RootToHeaderResolver {
                root: root_id,
                header: header_id,
            },
        };
        let registry = LanguageRegistry::new().register(Box::new(plugin));
        let backend = PluginWorkerBackend::new(&registry);
        let repo = tempfile::tempdir().expect("repo tempdir");
        write_repo_file(repo.path(), root_rel, "root unit content\n");
        write_repo_file(repo.path(), header_rel, "header content\n");

        // Body-only twin first: the direct branch analyzes ONLY the unit.
        let body_only = tu_task(6301, root_rel, None);
        let direct = backend
            .analyze_task(&body_only, repo.path())
            .expect("body-only dispatch must analyze through the plugin");
        assert_eq!(
            *analyzed_files.lock().unwrap(),
            vec![root_id],
            "the body-only branch analyzes the unit file and nothing else"
        );
        assert_eq!(direct.tu_file_id, root_id);
        analyzed_files.lock().unwrap().clear();

        // The same task carrying a store context must cancel that branch:
        // dispatch runs the canonical closure merge through the plugin's
        // resolver/analyzer, visiting the declared header - a file the
        // body-only branch can never observe.
        let store_ctx =
            TuStoreContext::new(&discovery_of(repo.path(), &[root_rel, header_rel]), Vec::new());
        let contextual = tu_task(6302, root_rel, Some(store_ctx));
        let merged = backend
            .analyze_task(&contextual, repo.path())
            .expect("store-context dispatch must run the closure merge");
        assert_eq!(
            *analyzed_files.lock().unwrap(),
            vec![root_id, header_id],
            "the closure merge analyzes the unit plus its resolved header, in closure order"
        );
        assert_eq!(
            merged.tu_file_id, root_id,
            "the merged analysis is stamped with the unit's own file id"
        );
    }
}
