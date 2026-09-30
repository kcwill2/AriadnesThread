//! [`CppPlugin`] — the registry entry point for C++ (plan M18.1).
//!
//! Wires the relocated components (analyzer M18.2, directive parsing/
//! resolution M18.1) into `ariadne_lang::LanguagePlugin`. Some pipeline
//! components are deliberately not implemented yet and fail loudly when
//! reached; their milestones are named on each stub.

use crate::analyzer::CppSemanticAnalyzer;
use crate::resolver::CppDependencyResolver;
use crate::shaper::CppUnitShaper;
use crate::source_extractor::CppSourceExtractor;
use ariadne_lang::{
    AggregationKind, ClusteringConstraints, CompilerValidator, CyclePolicy, DependencyResolver,
    GuardKind, LanguageCapabilities, LanguageDetector, LanguageId, LanguageModeling, LanguagePlugin,
    SemanticAnalyzer, SourceExtractor, SourceFileKind, SymbolId, SymbolPlacement, SymbolRecord,
    ToolchainStatus, UnitKind, UnitShaper,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Single source of truth (plan §3.5 cache row, M18.10) for the C++
/// `plugin_fingerprint` ingredient of `TuCacheFingerprint`: every registry
/// walk feeds [`CppPlugin::analysis_fingerprint`], which returns this value.
/// Bump whenever tokenizer/`ParserCtx`/analyzer behaviour changes (same rule
/// as `analysis_fingerprint`).
pub const CPP_ANALYSIS_FINGERPRINT: &str = "cpp-analyzer-1";

/// Classifies C/C++ sources and headers by extension.
pub struct CppDetector;

impl LanguageDetector for CppDetector {
    fn name(&self) -> &str {
        "c++"
    }

    fn detect(&self, path: &Path) -> SourceFileKind {
        match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("cpp" | "cc" | "cxx" | "c") => SourceFileKind::Source,
            Some("h" | "hpp" | "hxx" | "hh" | "inl" | "ipp") => SourceFileKind::Header,
            _ => SourceFileKind::Other,
        }
    }
}

/// C++ has no atomic symbol groups beyond ownership/policy (plan §3.4) and
/// no language-level frozen symbols — the policy layer owns freezing.
pub struct CppClusteringConstraints;

impl ClusteringConstraints for CppClusteringConstraints {
    fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
        Vec::new()
    }

    fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
        HashSet::new()
    }
}

pub use crate::dependency_rewriter::CppDependencyRewriter;

/// Placeholder for real compiler invocation: `validate_compile` reports a
/// failure (never a silent pass) until tier-3 wiring lands in M23. M18.10
/// landed `toolchain_status()` below so the compile tier can report
/// `Skipped` — not a vacuous `Pass` — when no compiler exists.
pub struct CppCompilerValidator;

/// Probes one compiler binary the way `tools/clang-worker/src/main.rs::
/// check_clang_available` does (spawn `--version`, success if the process
/// could run): a usable compiler yields `Available(<version banner line>)`,
/// an unrunnable one `Missing` — never a panic (spawn errors are plain
/// `io::Error`s mapped to `Missing`).
pub(crate) fn probe_compiler_status(binary: &str) -> ToolchainStatus {
    match std::process::Command::new(binary)
        .arg("--version")
        .output()
    {
        Ok(out) => {
            let banner = String::from_utf8_lossy(&out.stdout);
            let version = banner
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .unwrap_or(binary);
            ToolchainStatus::Available(version.to_string())
        }
        Err(_) => ToolchainStatus::Missing,
    }
}

impl CompilerValidator for CppCompilerValidator {
    fn validate_compile(&self, _files: &[PathBuf], _args: &[String]) -> Result<(), String> {
        Err("C++ CompilerValidator not wired yet (M23)".to_string())
    }

    /// M18.10 (plan §3.4 toolchain row): probes the clang driver exactly as
    /// the clang-worker sidecar does. The worker mirrors an MSVC-compatible
    /// clang invocation (`tools/clang-worker/src/main.rs::check_clang_available`
    /// only ever consults `clang`), so absence of clang means this pipeline
    /// has no compiler it could run — the probe reports `Missing` and the
    /// affected-TU tier `Skipped`s instead of pretending to have compiled.
    fn toolchain_status(&self) -> ToolchainStatus {
        probe_compiler_status("clang")
    }
}

/// The C++ language plugin (plan §3.4 lifecycle mapping table).
///
/// [`CppDependencyResolver`] is stateless (M18.5): directive parsing carries
/// the includer path and spelling on each [`RawDepEdge`](ariadne_lang::RawDepEdge),
/// and per-repository inputs (catalog, search paths) arrive per call through
/// the trait arguments — so one instance serves every repository.
pub struct CppPlugin {
    detector: CppDetector,
    analyzer: CppSemanticAnalyzer,
    resolver: CppDependencyResolver,
    constraints: CppClusteringConstraints,
    extractor: CppSourceExtractor,
    rewriter: CppDependencyRewriter,
    validator: CppCompilerValidator,
    shaper: CppUnitShaper,
}

impl CppPlugin {
    pub fn new() -> Self {
        Self {
            detector: CppDetector,
            analyzer: CppSemanticAnalyzer,
            resolver: CppDependencyResolver::new(),
            constraints: CppClusteringConstraints,
            extractor: CppSourceExtractor,
            rewriter: CppDependencyRewriter,
            validator: CppCompilerValidator,
            shaper: CppUnitShaper,
        }
    }

    /// C++ planner hook (plan M18.6): header extensions are the aggregation
    /// units the planner may propose against.
    pub fn aggregation_filter(&self, path: &Path) -> bool {
        crate::planning::cpp_aggregation_filter(path)
    }

    /// C++ planner hook (plan M18.6/§3.6): extracted symbols land in a
    /// `{symbol}_extracted.h` beside the aggregation file.
    pub fn target_path_for(&self, aggregation_path: &Path, symbol_name: &str) -> PathBuf {
        crate::planning::cpp_target_path_for(aggregation_path, symbol_name)
    }
}

impl Default for CppPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguagePlugin for CppPlugin {
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

    fn rewriter(&self) -> &dyn ariadne_lang::DependencyRewriter {
        &self.rewriter
    }

    fn validator(&self) -> &dyn CompilerValidator {
        &self.validator
    }

    fn id(&self) -> LanguageId {
        LanguageId::Cpp
    }

    /// Plan §3.4 C++ column: TU unit, header aggregation that re-exports,
    /// `#pragma once` (today's emit when both guards are possible), split
    /// declaration/definition placement, cycles are defects. External
    /// boundary: `third_party/`, `vendor/` path components plus angle
    /// system includes (the latter is directive syntax the resolver knows,
    /// not expressible as a path marker).
    fn modeling(&self) -> LanguageModeling {
        LanguageModeling {
            language: LanguageId::Cpp,
            unit: UnitKind::TranslationUnit,
            aggregation: AggregationKind::Header,
            guard: GuardKind::PragmaOnce,
            symbol_placement: SymbolPlacement::SplitDeclaration,
            aggregation_reexports: true,
            cycle_policy: CyclePolicy::Reject,
            external_path_markers: vec!["third_party".into(), "vendor".into()],
        }
    }

    /// Everything works offline through the built-in analyzer/tokenizer;
    /// the clang-worker sidecar is an optional accelerator, not a
    /// requirement (plan A3, requires_toolchain = false).
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

    /// Bump when the tokenizer/`ParserCtx`/analyzer behaviour changes —
    /// the value lives in [`CPP_ANALYSIS_FINGERPRINT`] so the transitional
    /// legacy cache walk can feed the identical string (M18.10).
    fn analysis_fingerprint(&self) -> String {
        CPP_ANALYSIS_FINGERPRINT.to_string()
    }

    /// The C++ shaper owns every build literal that used to sit in
    /// `ariadne-build` (plan M18.4).
    fn shaper(&self) -> Option<&dyn UnitShaper> {
        Some(&self.shaper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang::{DirectiveStyle, LanguageRegistry, ResolutionContext};
    use ariadne_repo::FileId;

    #[test]
    fn plugin_identity_and_fingerprint() {
        let plugin = CppPlugin::new();
        assert_eq!(plugin.id(), LanguageId::Cpp);
        assert_eq!(plugin.analysis_fingerprint(), "cpp-analyzer-1");
        assert!(plugin.shaper().is_some(), "CppUnitShaper landed in M18.4");
        assert_eq!(
            plugin.shaper().unwrap().build_markers(),
            vec!["CMakeLists.txt", "compile_commands.json", ".vcxproj"]
        );
    }

    #[test]
    fn toolchain_status_reports_a_concrete_probe_never_unknown() {
        // Mirror of `clang-worker`'s `check_clang_available` semantics without
        // requiring any toolchain on PATH: the probe must answer quickly and
        // concretely (Available or Missing) on machines with no compiler.
        let start = std::time::Instant::now();
        let status = CppCompilerValidator.toolchain_status();
        assert!(
            matches!(status, ToolchainStatus::Available(_) | ToolchainStatus::Missing),
            "C++ probe must report a concrete status, got {status:?}"
        );
        assert!(
            start.elapsed() < std::time::Duration::from_secs(30),
            "toolchain probe must return promptly"
        );
    }

    #[test]
    fn probe_of_absent_binary_is_missing_and_graceful() {
        // Deterministic Missing path — no PATH manipulation required.
        assert_eq!(
            probe_compiler_status("definitely-not-a-compiler-xyzzy"),
            ToolchainStatus::Missing
        );
    }

    #[test]
    fn analysis_fingerprint_backed_by_shared_const() {
        // Single source of truth for the cache fingerprint (M18.10): both the
        // legacy shim walk and `analysis_fingerprint()` must feed the SAME
        // value into `TuCacheFingerprint`.
        assert_eq!(
            CppPlugin::new().analysis_fingerprint(),
            crate::CPP_ANALYSIS_FINGERPRINT
        );
    }

    #[test]
    fn modeling_matches_plan_section_3_4() {
        let modeling = CppPlugin::new().modeling();
        assert_eq!(modeling.language, LanguageId::Cpp);
        assert_eq!(modeling.unit, ariadne_lang::UnitKind::TranslationUnit);
        assert_eq!(modeling.aggregation, AggregationKind::Header);
        assert!(modeling.aggregation_reexports);
        assert_eq!(modeling.guard, GuardKind::PragmaOnce);
        assert_eq!(modeling.symbol_placement, SymbolPlacement::SplitDeclaration);
        assert_eq!(modeling.cycle_policy, CyclePolicy::Reject);
        assert_eq!(
            modeling.external_path_markers,
            vec!["third_party".to_string(), "vendor".to_string()]
        );
    }

    #[test]
    fn capabilities_all_on_and_offline() {
        let caps = CppPlugin::new().capabilities();
        assert!(
            caps.discovery
                && caps.dependency_graph
                && caps.semantic_index
                && caps.load_metric
                && caps.planning
                && caps.extraction
                && caps.rewriting
                && caps.validation
        );
        assert!(
            !caps.requires_toolchain,
            "built-in analyzer path must work without a compiler (A3)"
        );
    }

    #[test]
    fn detector_classifies_cpp_kinds() {
        let detector = CppDetector;
        assert_eq!(
            detector.detect(Path::new("a/main.cpp")),
            SourceFileKind::Source
        );
        assert_eq!(
            detector.detect(Path::new("a/main.cc")),
            SourceFileKind::Source
        );
        assert_eq!(
            detector.detect(Path::new("a/main.cxx")),
            SourceFileKind::Source
        );
        assert_eq!(
            detector.detect(Path::new("a/main.c")),
            SourceFileKind::Source
        );
        assert_eq!(
            detector.detect(Path::new("a/main.h")),
            SourceFileKind::Header
        );
        assert_eq!(
            detector.detect(Path::new("a/main.hpp")),
            SourceFileKind::Header
        );
        assert_eq!(
            detector.detect(Path::new("a/main.hxx")),
            SourceFileKind::Header
        );
        assert_eq!(
            detector.detect(Path::new("a/Main.scala")),
            SourceFileKind::Other
        );
        assert_eq!(
            detector.detect(Path::new("README.txt")),
            SourceFileKind::Other
        );
    }

    #[test]
    fn registry_routes_cpp_paths_to_plugin() {
        let registry = LanguageRegistry::new().register(Box::new(CppPlugin::new()));
        assert_eq!(registry.languages(), vec![LanguageId::Cpp]);
        assert!(registry.plugin_for_id(LanguageId::Cpp).is_some());
        assert_eq!(
            registry.detect_language(Path::new("src/main.cpp")),
            Some(LanguageId::Cpp)
        );
        assert_eq!(registry.detect_language(Path::new("notes.txt")), None);
    }

    #[test]
    fn plugin_analyzer_produces_symbol_analysis() {
        let plugin = CppPlugin::new();
        let analysis = plugin.analyzer().analyze(
            FileId::from_relative_path("a.cpp"),
            "class Widget {};",
            "Default",
        );
        assert!(analysis.symbols.iter().any(|s| s.name == "Widget"));
        assert_eq!(analysis.language, ariadne_repo::SourceLanguage::Cpp);
    }

    #[test]
    fn plugin_resolver_scans_directives() {
        let plugin = CppPlugin::new();
        let ctx = ResolutionContext::default();
        let edges = plugin.resolver().resolve_edges(
            FileId::from_relative_path("a.cpp"),
            Path::new("a.cpp"),
            "#include \"a.h\"\n#include <vector>\n",
            &ctx,
        );
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0].raw_target, "a.h");
        assert_eq!(edges[0].style, DirectiveStyle::Quoted);
        assert_eq!(edges[1].raw_target, "vector");
        assert_eq!(edges[1].style, DirectiveStyle::Angled);
    }

    #[test]
    fn plugin_resolver_targets_resolve_through_caller_catalog() {
        // The stateless plugin resolver answers `resolve_target` from the
        // arguments alone (M18.3 seam): no per-repository construction.
        use ariadne_lang::{FileCatalog, ResolveOutcome};
        let plugin = CppPlugin::new();
        let record = ariadne_repo::FileRecord {
            id: ariadne_repo::FileId::from_relative_path("src/a.h"),
            path: PathBuf::from("src/a.h"),
            canonical_path: PathBuf::from("src/a.h"),
            kind: ariadne_repo::FileKind::Header,
            size: 0,
            content_hash: String::new(),
            ownership: ariadne_repo::FileOwnership::Repository,
            optimization_policy: ariadne_repo::OptimizationPolicy::Optimizable,
            language: ariadne_repo::SourceLanguage::Cpp,
        };
        let catalog = FileCatalog::from_records(std::slice::from_ref(&record));
        let ctx = ResolutionContext::default();
        let edge = plugin
            .resolver()
            .resolve_edges(
                FileId::from_relative_path("src/a.cpp"),
                Path::new("src/a.cpp"),
                "#include \"a.h\"\n",
                &ctx,
            )
            .remove(0);
        assert_eq!(
            plugin.resolver().resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(record.id)
        );
    }
}
