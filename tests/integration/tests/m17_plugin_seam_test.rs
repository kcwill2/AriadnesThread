//! M17 plugin-seam suite (plan §6.3): proves the `ariadne-lang-api` seam is
//! the same vocabulary every crate already uses, that old-shape artifact
//! JSON still reads back unchanged, and enforces the two architecture
//! guards:
//!
//! * `plugin_crates_do_not_depend_on_core_pipeline` — plugin crates must not
//!   depend on `graph`/`cluster`/`transform`/`validate`. The plugin crates
//!   (`lang-cpp`, `lang-scala`) do not exist yet: the test skips an absent
//!   `Cargo.toml` and starts asserting the moment they appear (M18/M19).
//! * `core_crates_do_not_name_languages` — the R2 guard: the seven core
//!   pipeline crates (incl. pipeline, minus its named composition-root
//!   registry) contain no plugin or language names in any source line;
//!   enabled unconditionally since the M18.11 shim removal and hardened by
//!   D-M23-10 (M23 T3).

use ariadne_lang::{
    AggregationKind, ClusteringConstraints, CompilerValidator, CyclePolicy, DependencyRewriter,
    GeneratedSourceFile, GuardKind, LanguageCapabilities, LanguageDetector, LanguageId,
    DependencyVocabulary, LanguageModeling, LanguagePlugin, LanguageRegistry, RawDepEdge,
    ResolutionContext, RewriteSpec,
    SemanticAnalyzer, SourceExtractor, SourceFileKind, SymbolBody, SymbolPlacement, SymbolRecord,
    ToolchainStatus, UnitKind,
};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Repository working dir (package lives at `<repo>/tests/integration`).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|tests_dir| tests_dir.parent())
        .expect("integration tests must live under <repo>/tests/integration")
        .to_path_buf()
}

// ── 1. Seam: type identity through `ariadne-index` ──────────────────────────

/// The seam proof target: a function that only speaks `ariadne_index::types`.
fn consumes_index_type(tu: ariadne_index::types::TuAnalysis) -> String {
    tu.context_name
}

#[test]
fn lang_api_types_are_the_same_types_index_re_exports() {
    // A value built through `ariadne_lang_api::types` is accepted where an
    // `ariadne_index::types` value is required — the re-export seam is one
    // type, not a conversion.
    let tu_api = ariadne_lang_api::types::TuAnalysis {
        tu_file_id: ariadne_repo::FileId::from_relative_path("src/a.cpp"),
        context_name: "Debug|x64".into(),
        ..Default::default()
    };
    assert_eq!(consumes_index_type(tu_api), "Debug|x64");

    // Crate-root paths resolve to the same type as well.
    fn root_paths(_: ariadne_index::TuAnalysis, _: ariadne_lang::TuAnalysis) {}
    root_paths(
        ariadne_lang_api::types::TuAnalysis::default(),
        ariadne_lang_api::types::TuAnalysis::default(),
    );
}

#[test]
fn old_shape_json_without_language_reads_as_cpp() {
    // Verbatim shape of a pre-M17 artifact: no `language` key anywhere.
    let old_json = r#"{
        "tu_file_id": 42,
        "context_name": "Debug|x64",
        "symbols": [],
        "declarations": [],
        "edges": [],
        "references": [],
        "call_sites": []
    }"#;
    let tu: ariadne_lang_api::types::TuAnalysis = serde_json::from_str(old_json).unwrap();
    assert_eq!(tu.context_name, "Debug|x64");
    assert_eq!(tu.language, ariadne_repo::SourceLanguage::Cpp);

    // FileRecord-shaped old JSON (repository manifest entry) likewise.
    let old_record = r#"{
        "id": 7,
        "path": "src/a.cpp",
        "canonical_path": "/abs/src/a.cpp",
        "kind": "Source",
        "size": 3,
        "content_hash": "h",
        "ownership": "Repository",
        "optimization_policy": "Optimizable"
    }"#;
    let record: ariadne_repo::FileRecord = serde_json::from_str(old_record).unwrap();
    assert_eq!(record.language, ariadne_repo::SourceLanguage::Cpp);
}

// ── 2. Mock plugin exercising the registry through the seam ─────────────────

struct SeamDetector;
impl LanguageDetector for SeamDetector {
    fn name(&self) -> &str {
        "seam-mock"
    }
    fn detect(&self, path: &Path) -> SourceFileKind {
        match path.extension().and_then(|e| e.to_str()) {
            Some("scala") => SourceFileKind::Source,
            _ => SourceFileKind::Other,
        }
    }
}

struct SeamAnalyzer;
impl SemanticAnalyzer for SeamAnalyzer {
    fn analyze(
        &self,
        file_id: ariadne_repo::FileId,
        _source: &str,
        _context: &str,
    ) -> ariadne_lang::TuAnalysis {
        ariadne_lang::TuAnalysis {
            tu_file_id: file_id,
            ..Default::default()
        }
    }
}

struct SeamResolver;
impl ariadne_lang::DependencyResolver for SeamResolver {
    fn resolve_edges(
        &self,
        _file_id: ariadne_repo::FileId,
        _rel_path: &Path,
        _source: &str,
        _ctx: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge> {
        vec![]
    }
}

struct SeamConstraints;
impl ClusteringConstraints for SeamConstraints {
    fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<ariadne_lang::SymbolId>> {
        vec![]
    }
    fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<ariadne_lang::SymbolId> {
        HashSet::new()
    }
}

struct SeamExtractor;
impl SourceExtractor for SeamExtractor {
    fn extract(&self, _bodies: &[SymbolBody], _ns: &str, _deps: &[String]) -> GeneratedSourceFile {
        GeneratedSourceFile {
            path: PathBuf::from("out.scala"),
            content: String::new(),
        }
    }
}

struct SeamRewriter;
impl DependencyRewriter for SeamRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
    }
}

struct SeamValidator;
impl CompilerValidator for SeamValidator {
    fn validate_compile(
        &self,
        _files: &[PathBuf],
        _args: &[String],
    ) -> Result<(), String> {
        Ok(())
    }
}

struct SeamPlugin;
impl LanguagePlugin for SeamPlugin {
    fn detector(&self) -> &dyn LanguageDetector {
        &SeamDetector
    }
    fn analyzer(&self) -> &dyn SemanticAnalyzer {
        &SeamAnalyzer
    }
    fn resolver(&self) -> &dyn ariadne_lang::DependencyResolver {
        &SeamResolver
    }
    fn constraints(&self) -> &dyn ClusteringConstraints {
        &SeamConstraints
    }
    fn extractor(&self) -> &dyn SourceExtractor {
        &SeamExtractor
    }
    fn rewriter(&self) -> &dyn DependencyRewriter {
        &SeamRewriter
    }
    fn validator(&self) -> &dyn CompilerValidator {
        &SeamValidator
    }
    fn id(&self) -> LanguageId {
        LanguageId::Scala
    }
    fn modeling(&self) -> LanguageModeling {
        LanguageModeling {
            language: LanguageId::Scala,
            unit: UnitKind::SourceFile,
            aggregation: AggregationKind::PackageObject,
            guard: GuardKind::None,
            symbol_placement: SymbolPlacement::SingleFile,
            aggregation_reexports: false,
            cycle_policy: CyclePolicy::TolerateWithinPackage,
            external_path_markers: vec!["lib".into(), "target".into()],
            // D26 fields: the seam mock mirrors Scala wording; the caveat
            // stays None because this mock is not the heuristic parser.
            dependency_vocabulary: DependencyVocabulary {
                directive_label: "Import".into(),
                directive_passive_verb: "imported".into(),
                aggregation_noun: "package aggregation".into(),
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
    // shaper() and analysis_fingerprint() deliberately left at defaults.
}

#[test]
fn seam_mock_plugin_drives_registry_lookups() {
    let registry = LanguageRegistry::new().register(Box::new(SeamPlugin));

    assert_eq!(registry.languages(), vec![LanguageId::Scala]);
    let plugin = registry
        .plugin_for_id(LanguageId::Scala)
        .expect("scala plugin registered");
    assert_eq!(plugin.analysis_fingerprint(), "v1"); // default
    assert!(plugin.shaper().is_none()); // default
    assert_eq!(plugin.validator().toolchain_status(), ToolchainStatus::Unknown); // default
    assert_eq!(plugin.modeling().symbol_placement, SymbolPlacement::SingleFile);
    assert!(!plugin.capabilities().requires_toolchain);

    assert_eq!(
        registry.detect_language(Path::new("src/main.scala")),
        Some(LanguageId::Scala)
    );
    assert_eq!(
        registry.detect_language(Path::new("README.txt")),
        None,
        "FileKind::Other maps to no language"
    );
    assert!(registry.plugin_for_id(LanguageId::Other).is_none());
}

// ── 3. Architecture guards (plan §6.3) ──────────────────────────────────────

/// Core-pipeline crates no plugin crate may depend on (plan rule R1:
/// plugin crates depend inward, never on the pipeline).
#[test]
fn plugin_crates_do_not_depend_on_core_pipeline() {
    const PLUGIN_CRATES: [&str; 2] = ["crates/lang-cpp", "crates/lang-scala"];
    const FORBIDDEN_DEPS: [&str; 4] = [
        "ariadne-graph",
        "ariadne-cluster",
        "ariadne-transform",
        "ariadne-validate",
    ];
    for crate_dir in PLUGIN_CRATES {
        let manifest = repo_root().join(crate_dir).join("Cargo.toml");
        if !manifest.exists() {
            // The plugin crates arrive in M18 (lang-cpp) and M19 (lang-scala);
            // the guard starts asserting the moment each one appears.
            continue;
        }
        let text = fs::read_to_string(&manifest)
            .unwrap_or_else(|e| panic!("unreadable {manifest:?}: {e}"));
        for dep in FORBIDDEN_DEPS {
            let offender = text
                .lines()
                .map(str::trim)
                .any(|line| line.starts_with(&format!("{dep} ")) || line.starts_with(&format!("{dep}=")));
            assert!(
                !offender,
                "{crate_dir} must not depend on {dep} (plan §6.3 rule R1)"
            );
        }
    }
}

/// Production lines AND in-src test lines (`#[cfg(test)]` mods are walked;
/// `crates/*/tests/` integration files are out of scope) of the core pipeline
/// crates must not name any concrete plugin, language, compiler binary,
/// directive syntax, or header suffix (plan §7 rule R2, I2).
///
/// M18.11 un-ignored this test once the C++ code moved into
/// `ariadne-lang-cpp` and the `#[deprecated]` re-export shims were removed.
/// M23 T3 (D-M23-10) hardened it: the walk scope gained `crates/pipeline`,
/// the token table gained the plugin vocabulary (`Cpp`, `lang_cpp`,
/// `ariadne-lang-cpp`), and plugin-named in-src tests were relocated to
/// `crates/*/tests/` files or neutral fakes.
///
/// Allowlist mechanism, exactly two kinds:
/// * doc-comment / whole-line-comment lines (below), and
/// * one NAMED composition-root file, `crates/pipeline/src/registry.rs`,
///   whose single duty is enumerating the plugin set — the same privilege
///   class as `cli/src/registry.rs`, which delegates here (D9 direction:
///   cli depends on pipeline, never the reverse). cli, scheduler, and build
///   stay outside the walk scope by design: the invariant is *analysis-core*
///   neutrality, and composition roots are named, not anonymous.
///
/// CARVE-OUT rulings, one rationale each (D-M23-10 + orchestrator ruling on
/// the T3 ask):
/// * `"c++20"`-style language-STANDARD DATA literals (crates/build fixtures,
///   index cache fixture) are language-standard values, not plugin names —
///   intentionally not tokenized.
/// * `SourceLanguage::Cpp` (stripped below, exact full-path string only) is
///   spine language-TAXONOMY DATA owned by `crates/repo`, which is out of the
///   walk scope BY DESIGN because the repository spine must name languages.
///   The guard's invariant targets plugin identifiers and plugin crate names:
///   `CppSourceExtractor`, `CppDependencyRewriter`, `CppPlugin`, `lang_cpp`,
///   `ariadne-lang-cpp` stay fully banned (locked by
///   `guard_neutral_vocab_stripping_cannot_mask_plugin_names`). Stripping is
///   exact-string only; whitespace-obfuscated plugin names (`Cpp Plugin`) are
///   out of scope for a substring guard.
const GUARD_CORE_CRATES: [&str; 7] = [
    "crates/index",
    "crates/graph",
    "crates/cluster",
    "crates/transform",
    "crates/validate",
    "crates/report",
    "crates/pipeline",
];
const GUARD_FORBIDDEN: [&str; 7] = [
    "clang",
    "g++",
    "#include",
    "\".h\"",
    "Cpp",
    "lang_cpp",
    "ariadne-lang-cpp",
];
const GUARD_ALLOWED_FILES: [&str; 1] = ["crates/pipeline/src/registry.rs"];
const GUARD_NEUTRAL_VOCAB: [&str; 1] = ["SourceLanguage::Cpp"];

fn is_allowlisted_line(trimmed: &str) -> bool {
    trimmed.starts_with("//") || trimmed.starts_with("*")
}

fn repo_relative(file: &Path) -> String {
    file.strip_prefix(repo_root())
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Tokens a single source line commits, after stripping exact spine-taxonomy
/// data strings (see the two allowlist kinds + carve-out rulings above).
fn line_guard_hits(line: &str) -> Vec<&'static str> {
    if is_allowlisted_line(line.trim_start()) {
        return Vec::new();
    }
    let mut scannable = line.to_string();
    for neutral in GUARD_NEUTRAL_VOCAB {
        scannable = scannable.replace(neutral, "");
    }
    GUARD_FORBIDDEN
        .into_iter()
        .filter(|pattern| scannable.contains(pattern))
        .collect()
}

fn guard_line_violations(file: &Path, crate_dir: &str) -> Vec<String> {
    let text = fs::read_to_string(file).unwrap_or_default();
    let mut hits = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        for pattern in line_guard_hits(line) {
            hits.push(format!(
                "{}:{}: contains {pattern:?} — {}",
                crate_dir,
                idx + 1,
                line.trim_start().trim_end()
            ));
        }
    }
    hits
}

/// Self-test for the GUARD_NEUTRAL_VOCAB machinery: stripping must be
/// limited to the exact spine-taxonomy string, never masking a plugin
/// identifier that merely shares a line with spine data (orchestrator
/// condition on the D-M23-10 carve-out ruling).
#[test]
fn guard_neutral_vocab_stripping_cannot_mask_plugin_names() {
    assert!(
        line_guard_hits("language: ariadne_repo::SourceLanguage::Cpp,").is_empty(),
        "exact spine-taxonomy data must not be a violation"
    );
    let planted = [
        "use ariadne_lang_cpp::CppPlugin;",
        "ariadne-lang-cpp = { path = \"../lang-cpp\" }",
        "let _ = ariadne_repo::SourceLanguage::Cpp; register(CppSourceExtractor);",
        ".register(Box::new(ariadne_lang_cpp::CppPlugin::new()))",
    ];
    for line in planted {
        assert!(
            !line_guard_hits(line).is_empty(),
            "planted plugin-name fixture must stay RED through stripping: {line}"
        );
    }
}

#[test]
fn core_crates_do_not_name_languages() {
    let mut violations: Vec<String> = Vec::new();
    for crate_dir in GUARD_CORE_CRATES {
        let src_dir = repo_root().join(crate_dir).join("src");
        let mut files = Vec::new();
        collect_rs_files(&src_dir, &mut files);
        for file in files {
            if GUARD_ALLOWED_FILES.contains(&repo_relative(&file).as_str()) {
                continue; // named composition root (see doc comment above)
            }
            violations.extend(guard_line_violations(&file, crate_dir));
        }
    }
    assert!(
        violations.is_empty(),
        "core crates name languages (plan §7 rule R2):\n{}",
        violations.join("\n")
    );
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return; // absent crate src dir: nothing to walk
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
