//! M24 Part A acceptance suite: language identity is OPEN and detection is
//! PLUGIN-AUTHORITATIVE.
//!
//! This suite exists to keep the milestone's promise honest. The project
//! intent is multi-language support through a plugin mechanism, and the plan
//! promises that adding a language costs one crate plus one registration line
//! with zero edits to the core crates. That promise was false for a structural
//! reason, and this file is the regression lock against it returning.
//!
//! What the suite proves:
//!
//! 1. `LanguageId` can name a language no closed enum knows about, and every
//!    pre-existing artifact spelling round-trips byte-identically.
//! 2. A registered plugin with a non-Cpp/Scala identity resolves through
//!    `detect_language`, `plugin_for_id`, AND the CLI dispatch path.
//! 3. A language with no plugin yields an explicit unhandled-language error,
//!    never a C++ default.
//! 4. No `_ => Cpp` fallback survives anywhere in the core crates -- checked by
//!    scanning the sources, because a `match` on an open type cannot be
//!    non-exhaustive and the compiler cannot catch a deliberate default.
//! 5. The golden C++ artifacts still spell their language exactly as before.

use ariadne_build::{CompileCommand, TranslationUnit};
use ariadne_lang::{
    AggregationKind, ClusteringConstraints, CompilerValidator, CyclePolicy, DependencyResolver,
    DependencyRewriter, GeneratedSourceFile, GuardKind, LanguageCapabilities, LanguageDetector,
    LanguageId, LanguageModeling, LanguagePlugin, LanguageRegistry, RawDepEdge, ResolutionContext,
    RewriteSpec, SemanticAnalyzer, SourceExtractor, SourceFileKind, SymbolBody, SymbolPlacement,
    SymbolRecord, UnitKind,
};
use ariadne_repo::{FileId, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ── a deliberately neutral third language ────────────────────────────────────

/// The identity Part A exists to permit. No such plugin ships yet -- building a
/// real third-language plugin is Part C -- so these tests register a MINIMAL
/// one whose only job is to prove the identity type and the dispatch path
/// accept a name that is neither C++ nor Scala.
const THIRD: &str = "Fortran";
const THIRD_EXTENSION: &str = "f90";

fn third_language() -> LanguageId {
    LanguageId::new(THIRD)
}

struct ThirdDetector;

impl LanguageDetector for ThirdDetector {
    fn name(&self) -> &str {
        THIRD
    }
    fn detect(&self, path: &Path) -> SourceFileKind {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case(THIRD_EXTENSION) => SourceFileKind::Source,
            _ => SourceFileKind::Other,
        }
    }
}

struct NeutralAnalyzer;

impl SemanticAnalyzer for NeutralAnalyzer {
    fn analyze(&self, file_id: FileId, _source: &str, _ctx: &str) -> ariadne_lang::TuAnalysis {
        ariadne_lang::TuAnalysis {
            tu_file_id: file_id,
            ..ariadne_lang::TuAnalysis::default()
        }
    }
}

struct NeutralResolver;

impl DependencyResolver for NeutralResolver {
    fn resolve_edges(
        &self,
        _file_id: FileId,
        _path: &Path,
        _source: &str,
        _context: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge> {
        Vec::new()
    }
}

struct NeutralConstraints;

impl ClusteringConstraints for NeutralConstraints {
    fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<ariadne_lang::SymbolId>> {
        Vec::new()
    }
    fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<ariadne_lang::SymbolId> {
        HashSet::new()
    }
}

struct NeutralExtractor;

impl SourceExtractor for NeutralExtractor {
    fn extract(&self, _bodies: &[SymbolBody], _ns: &str, _deps: &[String]) -> GeneratedSourceFile {
        GeneratedSourceFile {
            path: PathBuf::from("out.f90"),
            content: String::new(),
        }
    }
}

struct NeutralRewriter;

impl DependencyRewriter for NeutralRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
    }
}

struct NeutralValidator;

impl CompilerValidator for NeutralValidator {
    // AR-24: compile-forced seam signature only; body and locks untouched.
    fn validate_compile(
        &self,
        _files: &[PathBuf],
        _args: &[String],
    ) -> Result<(), ariadne_lang::PluginError> {
        Ok(())
    }
}

struct ThirdPlugin;

impl LanguagePlugin for ThirdPlugin {
    fn detector(&self) -> &dyn LanguageDetector {
        &ThirdDetector
    }
    fn analyzer(&self) -> &dyn SemanticAnalyzer {
        &NeutralAnalyzer
    }
    fn resolver(&self) -> &dyn DependencyResolver {
        &NeutralResolver
    }
    fn constraints(&self) -> &dyn ClusteringConstraints {
        &NeutralConstraints
    }
    fn extractor(&self) -> &dyn SourceExtractor {
        &NeutralExtractor
    }
    fn rewriter(&self) -> &dyn DependencyRewriter {
        &NeutralRewriter
    }
    fn validator(&self) -> &dyn CompilerValidator {
        &NeutralValidator
    }
    fn id(&self) -> LanguageId {
        third_language()
    }
    fn modeling(&self) -> LanguageModeling {
        LanguageModeling {
            language: third_language(),
            unit: UnitKind::SourceFile,
            aggregation: AggregationKind::None,
            guard: GuardKind::None,
            symbol_placement: SymbolPlacement::SingleFile,
            aggregation_reexports: false,
            cycle_policy: CyclePolicy::Tolerate,
            external_path_markers: Vec::new(),
            dependency_vocabulary: ariadne_lang::DependencyVocabulary {
                directive_label: "Use".into(),
                directive_passive_verb: "used".into(),
                aggregation_noun: "module".into(),
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
            planning: true,
            extraction: true,
            rewriting: true,
            validation: true,
            requires_toolchain: false,
        }
    }
}

/// The default registry plus one registered third language. C++ stays first so
/// registration-order precedence (invariant I3) is unchanged for C++.
fn registry_with_third_language() -> LanguageRegistry {
    LanguageRegistry::new()
        .register(Box::new(ThirdPlugin))
        .register(Box::new(ariadne_lang_cpp::CppPlugin::new()))
        .register(Box::new(ariadne_lang_scala::ScalaPlugin::new()))
}

fn record_with(rel: &str, kind: ariadne_repo::FileKind, language: SourceLanguage) -> FileRecord {
    let path = PathBuf::from(rel);
    FileRecord {
        id: FileId::from_relative_path(rel),
        path: path.clone(),
        canonical_path: path,
        kind,
        size: 0,
        content_hash: String::new(),
        ownership: FileOwnership::Repository,
        optimization_policy: OptimizationPolicy::Optimizable,
        language,
    }
}

fn discovery_of(records: Vec<FileRecord>) -> ariadne_repo::RepositoryDiscoveryResult {
    ariadne_repo::RepositoryDiscoveryResult {
        root: PathBuf::from("."),
        files: records,
        ownership_summary: Default::default(),
        policy_summary: Default::default(),
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|tests_dir| tests_dir.parent())
        .expect("integration tests live under <repo>/tests/integration")
        .to_path_buf()
}

// ── 1. the identity type is open, and artifact-compatible ─────────────────────

#[test]
fn language_identity_can_name_a_language_no_closed_enum_knows() {
    // D-A1: this is the whole milestone in one assertion. Before Part A,
    // `LanguageId` WAS `SourceLanguage`, so naming "Fortran" was not merely
    // unsupported -- it did not compile.
    let named = third_language();
    assert_eq!(named.as_str(), THIRD);
    assert_ne!(named, LanguageId::Cpp);
    assert_ne!(named, LanguageId::Scala);
    assert_ne!(named, LanguageId::Other);

    let registry = registry_with_third_language();
    assert!(
        registry.languages().contains(&third_language()),
        "a plugin may name a language the core crates have never heard of"
    );
}

#[test]
fn every_legacy_artifact_spelling_round_trips_byte_identically() {
    // The artifact-compatibility claim the plan depends on. `#[serde(
    // transparent)]` means the newtype serializes as the bare name, so every
    // pre-existing artifact keeps its exact bytes in both directions.
    for id in [
        LanguageId::Cpp,
        LanguageId::Scala,
        LanguageId::Other,
        third_language(),
    ] {
        let json = serde_json::to_string(&id).expect("serialize a language id");
        assert_eq!(
            json,
            format!("\"{}\"", id.as_str()),
            "a language id is a bare string, never an object or a tagged enum"
        );
        let back: LanguageId = serde_json::from_str(&json).expect("deserialize a language id");
        assert_eq!(back, id);
    }

    // Debug is the bare name too, because the TU cache digest hashes
    // `format!("{language:?}")`. A derived Debug would invalidate every
    // cache store in the field with no failing test to show for it.
    assert_eq!(format!("{:?}", third_language()), THIRD);
    assert_eq!(format!("{}", third_language()), THIRD);
    assert_eq!(LanguageId::default(), LanguageId::Cpp);
}

// ── 2. a third language resolves through every dispatch path ──────────────────

#[test]
fn a_third_language_plugin_resolves_through_detection_and_id_lookup() {
    let registry = registry_with_third_language();
    let claimed = PathBuf::from("src/solver.f90");

    // The extension is unknown to FileKind, so the closed table cannot name it.
    assert_eq!(
        ariadne_lang_api::identity::language_of_file_kind(ariadne_repo::FileKind::from_path(
            &claimed
        )),
        LanguageId::Other,
        "no FileKind variant names a third language"
    );

    assert_eq!(
        registry.detect_language(&claimed),
        Some(third_language()),
        "D-A2: the claiming plugin's detector decides, so an unknown \
         extension is no longer a refusal"
    );
    assert_eq!(
        registry
            .plugin_for_id(third_language())
            .expect("the third-language plugin is registered")
            .id(),
        third_language()
    );
    assert!(
        registry.plugin_for(&claimed).is_some(),
        "plugin_for needs no language-identity precondition"
    );
}

#[test]
fn a_third_language_dispatches_through_the_cli_catalog() {
    // The CLI dispatch path. The record carries the deliberately WRONG label
    // (`Other`) because that is exactly the shape a real third language has
    // today: no FileKind variant names it, so discovery can only say `Other`.
    // Before D-A5 the catalog copied that label and the file got NO plugin.
    let registry = LanguageRegistry::new()
        .register(Box::new(ThirdPlugin))
        .register(Box::new(ariadne_lang_cpp::CppPlugin::new()))
        .register(Box::new(ariadne_lang_scala::ScalaPlugin::new()));

    let third = record_with("src/solver.f90", ariadne_repo::FileKind::Other, SourceLanguage::Other);
    let file_id = third.id;
    let discovery = discovery_of(vec![third]);
    let catalog =
        ariadne_cli::registry::catalog_languages(&registry, &discovery);

    assert_eq!(
        catalog.get(&file_id),
        Some(&third_language()),
        "the CLI catalog resolves through the registry, not the record label"
    );
    assert_eq!(
        ariadne_cli::registry::plugin_for_catalog_file(&registry, &catalog, file_id)
            .expect("a file labelled Other still dispatches when a plugin claims it")
            .id(),
        third_language()
    );
}

// ── 3. an unclaimed language errors, it never defaults to C++ ────────────────

#[test]
fn a_language_with_no_plugin_yields_an_explicit_unhandled_language_error() {
    let registry = ariadne_cli::registry::default_registry();
    let unclaimed = PathBuf::from("src/nowhere.mock");

    assert_eq!(
        registry.detect_language(&unclaimed),
        None,
        "no plugin claims it, so it has no language - not a wrong one"
    );

    // The incremental stage's own error path, exercised end to end: a unit in
    // an unclaimed language must say so.
    let unit = TranslationUnit {
        file_id: FileId::from_relative_path("src/nowhere.mock"),
        source_path: unclaimed,
        context_name: "root".to_string(),
        compile_command: CompileCommand {
            directory: PathBuf::from("."),
            file: PathBuf::from("src/nowhere.mock"),
            command: None,
            arguments: Vec::new(),
            context_name: "root".to_string(),
            output: None,
        },
        language: SourceLanguage::Other,
    };
    let build = ariadne_build::BuildDiscoveryResult {
        repo_root: PathBuf::from("."),
        contexts: Vec::new(),
        translation_units: vec![unit],
    };

    let err = ariadne_cli::registry::incremental_backend(&registry, &build)
        .err()
        // AR-25: the seam now errors as a typed anyhow source; render for the
        // text lock (wording assertions below UNEDITED).
        .expect("an unclaimed unit language must error")
        .to_string();
    assert!(
        err.contains("unhandled language"),
        "the error names the failure honestly: {err}"
    );
    assert!(
        !err.contains("Cpp"),
        "and never names a C++ default: {err}"
    );
}

// ── 4. no C++ fallback survives anywhere in the core crates ──────────────────

/// Source-level lock for D-A4. The compiler CANNOT catch this one: a `match`
/// on an open type has no exhaustiveness requirement, so a deliberate
/// `_ => cpp` compiles cleanly and silently mis-dispatches. This scans for it.
#[test]
fn no_cpp_fallback_survives_in_a_core_crate_dispatch_match() {
    // `crates/repo` is excluded by design: the repository spine is allowed to
    // name languages, and its `from_file_kind` match is the closed LABEL
    // taxonomy D-A6 deliberately retains, not a dispatch fallback.
    const CORE_CRATES: [&str; 11] = [
        "crates/lang",
        "crates/lang-api",
        "crates/index",
        "crates/graph",
        "crates/cluster",
        "crates/transform",
        "crates/validate",
        "crates/report",
        "crates/pipeline",
        "crates/scheduler",
        "crates/cli",
    ];

    let mut violations: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    for crate_dir in CORE_CRATES {
        let src = repo_root().join(crate_dir).join("src");
        let mut files = Vec::new();
        collect_rs_files(&src, &mut files);
        for file in files {
            scanned += 1;
            let text = std::fs::read_to_string(&file).unwrap_or_default();
            for (idx, line) in text.lines().enumerate() {
                for pattern in forbidden_cpp_fallbacks_in(line) {
                    violations.push(format!(
                        "{}:{}: {pattern:?} in {}",
                        crate_dir,
                        idx + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(scanned > 0, "the scan actually walked the core crates");
    assert!(
        violations.is_empty(),
        "a C++ dispatch fallback survived (D-A4 forbids every one):\n{}",
        violations.join("\n")
    );
}

fn forbidden_cpp_fallbacks_in(line: &str) -> Vec<&'static str> {
    const FORBIDDEN: [&str; 3] = [
        "_ => ariadne_lang_cpp",
        "_ => cpp_",
        "=> SourceLanguage::Cpp",
    ];
    let trimmed = line.trim_start();
    // Comment lines describe the history, including the fallback this
    // milestone removed; only live code is scanned.
    if trimmed.starts_with("//") || trimmed.starts_with("*") {
        return Vec::new();
    }
    FORBIDDEN
        .into_iter()
        .filter(|pattern| line.contains(pattern))
        .collect()
}

/// Self-test for the scan above, so it can never go vacuous. Without this the
/// D-A4 lock would pass trivially if the pattern list stopped matching anything
/// — and a lock that cannot fail proves nothing. Same discipline as
/// `m17_plugin_seam_test::guard_neutral_vocab_stripping_cannot_mask_plugin_names`.
#[test]
fn the_cpp_fallback_scan_actually_matches_a_real_fallback() {
    // The EXACT shape this milestone removed from
    // `crates/cli/src/commands/plan.rs`.
    let removed = "            _ => ariadne_lang_cpp::cpp_aggregation_filter(path),";
    assert!(
        !forbidden_cpp_fallbacks_in(removed).is_empty(),
        "the scan must catch the pre-D-A4 misroute verbatim"
    );
    assert!(
        !forbidden_cpp_fallbacks_in("            _ => cpp_target_path_for(candidate, name),")
            .is_empty(),
        "and the namer half"
    );

    // Live code that dispatches correctly stays clean.
    for clean in [
        "            Some(language) if *language == LanguageId::Cpp => {",
        "            Some(_) | None => false,",
        "            Some(language) if *language == LanguageId::Scala => {",
    ] {
        assert!(
            forbidden_cpp_fallbacks_in(clean).is_empty(),
            "correct dispatch must not be flagged: {clean}"
        );
    }

    // A comment describing the removed fallback is history, not live code.
    assert!(
        forbidden_cpp_fallbacks_in("        // was `_ => cpp_...` before D-A4").is_empty(),
        "a comment must not be flagged as live code"
    );
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

// ── 5. the golden C++ artifacts are untouched ────────────────────────────────

#[test]
fn golden_cpp_artifacts_still_spell_their_language_as_before() {
    // The sharpest available check that the identity type stayed
    // artifact-compatible: the committed golden bytes are still exactly what
    // the pre-milestone enum produced.
    let expected: [(&str, usize); 6] = [
        ("tests/golden/basic/repository_files.json", 5),
        ("tests/golden/expensive/repository_files.json", 4),
        ("tests/golden/basic/build_contexts.json", 1),
        ("tests/golden/expensive/build_contexts.json", 1),
        ("tests/golden/basic/index_index.json", 2),
        ("tests/golden/expensive/index_index.json", 2),
    ];

    for (relative, minimum_occurrences) in expected {
        let path = repo_root().join(relative);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("unreadable {relative}: {e}"));
        let occurrences = text.matches("\"language\": \"Cpp\"").count();
        assert!(
            occurrences >= minimum_occurrences,
            "{relative} lost its language field: expected at least \
             {minimum_occurrences} \"language\": \"Cpp\", found {occurrences}"
        );
    }

    // And a golden FileRecord still reads back to the same closed label, which
    // is why `FileRecord.language` was NOT migrated to the open identity.
    let golden = std::fs::read_to_string(repo_root().join("tests/golden/basic/repository_files.json"))
        .expect("read the golden repository files");
    let parsed: Vec<FileRecord> =
        serde_json::from_value(serde_json::from_str(&golden).expect("golden JSON"))
            .expect("golden records deserialize");
    assert!(!parsed.is_empty(), "the golden set is not empty");
    assert!(
        parsed.iter().all(|file| file.language == SourceLanguage::Cpp),
        "a C++ golden record still reads back as the Cpp label"
    );
}