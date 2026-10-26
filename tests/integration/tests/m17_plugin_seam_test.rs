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
    // AR-24: compile-forced seam signature only; body and locks untouched.
    fn validate_compile(
        &self,
        _files: &[PathBuf],
        _args: &[String],
    ) -> Result<(), ariadne_lang::PluginError> {
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
    const PLUGIN_CRATES: [&str; 3] = ["crates/lang-cpp", "crates/lang-scala", "crates/lang-rust"];
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
const GUARD_CORE_CRATES: [&str; 8] = [
    "crates/index",
    "crates/graph",
    "crates/cluster",
    "crates/transform",
    "crates/validate",
    "crates/report",
    "crates/pipeline",
    "crates/artifact",
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

// -- 4. Transitive plugin-cone guard (plan rule R1, transitive form) --------

/// Core-pipeline crates no plugin crate may reach, not even through a chain
/// of local path edges (plan rule R1 must hold transitively; the AR-20b
/// end-state cone per `tasks/ar-20-shape.md` invariant 6).
const TRANSITIVE_FORBIDDEN_DEPS: [&str; 7] = [
    "ariadne-scheduler",
    "ariadne-report",
    "ariadne-cluster",
    "ariadne-graph",
    "ariadne-transform",
    "ariadne-validate",
    "ariadne-artifact",
];

/// Transitive form of `plugin_crates_do_not_depend_on_core_pipeline` (plan
/// rule R1, shape `tasks/ar-20-shape.md` invariants 5/6): walks the local
/// path-dependency closure of each plugin manifest over the
/// `[dependencies]` / `[build-dependencies]` tables, dev edges skipped,
/// cycle-safe visited set, asserting none of scheduler/report/cluster/
/// graph/transform/validate is reachable. Mechanism mirrors `cargo tree`
/// restricted to compiled edges. Landed GREEN with the AR-20b switch; its
/// falsification evidence is the AR-20 journal RED run (`6 passed / 1
/// failed`, `d63c0d0`/`7adbeca`) and its load-bearingness is locked by
/// `transitive_guard_detects_planted_scheduler_edge` below.
#[test]
fn plugin_crates_transitive_tree_excludes_core_pipeline() {
    const PLUGIN_CRATES: [&str; 3] = ["crates/lang-cpp", "crates/lang-scala", "crates/lang-rust"];
    let mut offenders: Vec<String> = Vec::new();
    for plugin_crate in PLUGIN_CRATES {
        let manifest = repo_root().join(plugin_crate).join("Cargo.toml");
        if !manifest.exists() {
            continue; // the guard starts asserting the moment a plugin crate appears
        }
        let reachable = reachable_local_crate_names(&manifest, &filesystem_manifest_reader);
        let found: Vec<&str> = TRANSITIVE_FORBIDDEN_DEPS
            .into_iter()
            .filter(|forbidden| reachable.iter().any(|name| name == forbidden))
            .collect();
        if !found.is_empty() {
            offenders.push(format!(
                "{plugin_crate} transitively compiles against core-pipeline \
                 crates {found:?} (plan rule R1 must hold transitively)"
            ));
        }
    }
    assert!(offenders.is_empty(), "{}", offenders.join("\n"));
}

/// Non-vacuity self-test for the transitive guard (shape invariant 9): the
/// closure walker must actually report a planted `ariadne-scheduler` edge
/// two hops down the chain, and must never follow a dev-dependencies edge.
/// In-memory planted manifests substitute for the unobservable in-tree RED
/// (whose evidence is the AR-20 journal's `6 passed / 1 failed` run at
/// `d63c0d0`/`7adbeca`), mirroring the
/// `guard_neutral_vocab_stripping_cannot_mask_plugin_names` pattern.
#[test]
fn transitive_guard_detects_planted_scheduler_edge() {
    let planted_root = concat!(
        "[dependencies]\n",
        "planted-intermediate = { path = \"../planted-intermediate\" }\n",
        "\n",
        "[dev-dependencies]\n",
        "planted-dev-only = { path = \"../planted-dev-only\" }\n",
    );
    let planted_intermediate = concat!(
        "[dependencies]\n",
        "ariadne-scheduler = { path = \"../scheduler\" }\n",
    );
    let planted_reader = |manifest: &Path| -> Option<String> {
        let path_text = manifest.to_string_lossy().replace('\\', "/");
        if path_text.ends_with("planted-intermediate/Cargo.toml") {
            Some(planted_intermediate.to_string())
        } else if path_text.ends_with("planted-lang-cpp/Cargo.toml") {
            Some(planted_root.to_string())
        } else {
            None
        }
    };
    let planted_root_manifest = repo_root().join("crates/planted-lang-cpp/Cargo.toml");
    let reachable = reachable_local_crate_names(&planted_root_manifest, &planted_reader);
    assert!(
        reachable.iter().any(|name| name == "ariadne-scheduler"),
        "planted scheduler edge not detected through closure descent: {reachable:?}"
    );
    assert!(
        !reachable.iter().any(|name| name == "planted-dev-only"),
        "dev-dependencies edges must not be walked: {reachable:?}"
    );
}

/// Crate names reachable from `root_manifest` through local-path compiled
/// edges, transitively. `manifest_reader` abstracts manifest access so the
/// planted-edge self-test can plant manifests in memory.
fn reachable_local_crate_names(
    root_manifest: &Path,
    manifest_reader: &dyn Fn(&Path) -> Option<String>,
) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut reachable_names = Vec::new();
    collect_reachable_local_deps(
        root_manifest,
        manifest_reader,
        &mut visited,
        &mut reachable_names,
    );
    reachable_names
}

/// Depth-first descent over local-path compiled edges; every manifest is
/// visited once (normalized-path visited set) so a cycle terminates.
fn collect_reachable_local_deps(
    manifest: &Path,
    manifest_reader: &dyn Fn(&Path) -> Option<String>,
    visited: &mut HashSet<String>,
    reachable_names: &mut Vec<String>,
) {
    let visit_key = normalize_manifest_path(manifest);
    if !visited.insert(visit_key) {
        return; // cycle-safe: each manifest is walked once
    }
    let Some(manifest_text) = manifest_reader(manifest) else {
        return; // absent/unreadable manifest: no further edges
    };
    let manifest_dir = manifest.parent().unwrap_or(Path::new("."));
    for (dep_name, dep_manifest) in parse_local_path_deps(&manifest_text, manifest_dir) {
        reachable_names.push(dep_name);
        collect_reachable_local_deps(
            &dep_manifest,
            manifest_reader,
            visited,
            reachable_names,
        );
    }
}

/// Direct local-path dependency edges declared in one manifest text. Only
/// the `[dependencies]` / `[build-dependencies]` tables are read (the
/// compiled-edge shape the journal RED proved); `[dev-dependencies]`,
/// target-scoped tables, and every other section are skipped.
fn parse_local_path_deps(
    manifest_text: &str,
    manifest_dir: &Path,
) -> Vec<(String, PathBuf)> {
    let mut edges = Vec::new();
    let mut in_compiled_table = false;
    for raw_line in manifest_text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            let table_name = line.trim_matches(|c| c == '[' || c == ']');
            in_compiled_table =
                table_name == "dependencies" || table_name == "build-dependencies";
            continue;
        }
        if !in_compiled_table {
            continue;
        }
        let Some((name_side, value_side)) = line.split_once('=') else {
            continue;
        };
        let Some(relative_path) = quoted_path_value(value_side) else {
            continue; // registry/workspace dep: no local edge to walk
        };
        let dep_name = name_side.trim().to_string();
        if dep_name.is_empty() {
            continue;
        }
        let dep_manifest = manifest_dir.join(relative_path).join("Cargo.toml");
        edges.push((dep_name, dep_manifest));
    }
    edges
}

/// Extract the quoted value of a `path = "..."` key from a dependency
/// line's value side; `None` when the line declares no local path.
fn quoted_path_value(value_side: &str) -> Option<String> {
    let after_path_key = value_side.split_once("path")?.1;
    let opening_quote = after_path_key.find('"')?;
    let after_open_quote = &after_path_key[opening_quote + 1..];
    let closing_quote = after_open_quote.find('"')?;
    Some(after_open_quote[..closing_quote].to_string())
}

/// Collapse `..` segments textually so the same crate reached through
/// different relative routes shares one visited key.
fn normalize_manifest_path(path: &Path) -> String {
    let mut segments: Vec<&std::ffi::OsStr> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                segments.pop();
            }
            other => segments.push(other.as_os_str()),
        }
    }
    PathBuf::from_iter(segments)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Production manifest reader used by the guard test proper.
fn filesystem_manifest_reader(manifest: &Path) -> Option<String> {
    fs::read_to_string(manifest).ok()
}

// -- 5. Scheduler cone guard (AR-26b, shape ruling `tasks/ar-26b-shape-ruling.md`) --

/// Local crates the scheduler must never reach through transitive local-path
/// compiled edges (AR-26b Ruling 1: `"ariadne-report" NOT-IN Cone(scheduler)`).
/// Exactly this one-element set is locked: `ariadne-cluster`,
/// `ariadne-transform`, and `ariadne-validate` are EXPECTED cone members
/// through `scheduler -> artifact` (precision note P2) and must never enter
/// this const - broadening it requires a separate shape ruling.
/// This const + the guard test are ratified verbatim by Ruling 4.
const SCHEDULER_CONE_FORBIDDEN: [&str; 1] = ["ariadne-report"];

/// Machine lock, mechanism (a) of AR-26b Ruling 1 (the companion mechanism
/// (b) is the handoff's gate 6: `cargo tree -p ariadne-scheduler -e normal`
/// exact-token grep, both required).
/// Walks `reachable_local_crate_names` over the scheduler manifest - the
/// same closure walker the AR-20b transitive guard uses, helpers reused
/// without forking - and asserts the cone excludes every name in
/// `SCHEDULER_CONE_FORBIDDEN`.
/// Landed RED-first per the AR-26b RGB protocol: the falsification evidence
/// at dispatch HEAD (scheduler still pulled report) is the AR-26b journal
/// `9 passed / 1 failed / 0 ignored / 1 suites` run, never committed.
/// Load-bearingness is locked by
/// `scheduler_cone_guard_detects_planted_report_edge` below.
/// The AR-26b hop also extended the two section-3/4 tables:
/// `GUARD_CORE_CRATES` gained `crates/artifact` (the moved store IO keeps
/// its R2 neutrality walk) and `TRANSITIVE_FORBIDDEN_DEPS` gained
/// `ariadne-artifact` (plugin crates must not reach the artifact store).
#[test]
fn scheduler_cone_excludes_report() {
    let root_manifest = repo_root().join("crates/scheduler/Cargo.toml");
    let reachable = reachable_local_crate_names(&root_manifest, &filesystem_manifest_reader);
    let mut offenders: Vec<String> = Vec::new();
    for forbidden in SCHEDULER_CONE_FORBIDDEN {
        if reachable.iter().any(|name| name == forbidden) {
            offenders.push(format!(
                "scheduler must not transitively compile against {forbidden:?} - \
                 consume ariadne-artifact directly (AR-26b); \
                 reachable from {root_manifest:?}: {reachable:?}"
            ));
        }
    }
    assert!(offenders.is_empty(), "{}", offenders.join("\n"));
}

/// Non-vacuity self-test for the scheduler cone guard (AR-26b Ruling 1
/// non-vacuity lock): the closure walker must report a planted
/// `ariadne-report` edge when the scheduler manifest declares one.
/// In-memory planted manifests substitute for the in-tree RED (whose
/// evidence is the AR-26b journal run), mirroring
/// `transitive_guard_detects_planted_scheduler_edge`. Green immediately by
/// design: this is the mechanism proof, not the invariant.
#[test]
fn scheduler_cone_guard_detects_planted_report_edge() {
    let planted_scheduler = concat!(
        "[dependencies]\n",
        "ariadne-report = { path = \"../report\" }\n",
    );
    let planted_reader = |manifest: &Path| -> Option<String> {
        let path_text = manifest.to_string_lossy().replace('\\', "/");
        if path_text.ends_with("planted-scheduler/Cargo.toml") {
            Some(planted_scheduler.to_string())
        } else {
            None
        }
    };
    let reachable = reachable_local_crate_names(
        &repo_root().join("crates/planted-scheduler/Cargo.toml"),
        &planted_reader,
    );
    assert!(
        reachable.iter().any(|name| name == "ariadne-report"),
        "planted report edge not detected from the scheduler cone shape: {reachable:?}"
    );
}
