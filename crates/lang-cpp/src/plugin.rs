//! [`CppPlugin`] — the registry entry point for C++ (plan M18.1).
//!
//! Wires the relocated components (analyzer M18.2, directive parsing/
//! resolution M18.1) into `ariadne_lang::LanguagePlugin`. Some pipeline
//! components are deliberately not implemented yet and fail loudly when
//! reached; their milestones are named on each stub.

use crate::analyzer::CppSemanticAnalyzer;
use crate::resolver::CppDependencyResolver;
use crate::shaper::CppUnitShaper;
use ariadne_lang::{
    AggregationKind, ClusteringConstraints, CompilerValidator, CyclePolicy, DependencyResolver,
    GeneratedSourceFile, GuardKind, LanguageCapabilities, LanguageDetector, LanguageId,
    LanguageModeling, LanguagePlugin, RewriteSpec, SemanticAnalyzer, SourceExtractor,
    SourceFileKind, SymbolBody, SymbolId, SymbolPlacement, SymbolRecord, UnitKind, UnitShaper,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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

/// Placeholder until the byte-identical `CppSourceExtractor` lands in M18.8
/// (plan §3.6). No pipeline code calls this before then; a call panics
/// loudly instead of silently emitting wrong bytes.
pub struct CppSourceExtractor;

impl SourceExtractor for CppSourceExtractor {
    fn extract(
        &self,
        _symbols: &[SymbolBody],
        _namespace: &str,
        _required_deps: &[String],
    ) -> GeneratedSourceFile {
        unimplemented!("CppSourceExtractor lands in M18.8 (plan §3.6)")
    }
}

/// Placeholder until the byte-identical `CppDependencyRewriter` lands in
/// M18.8 (plan §3.6). No pipeline code calls this before then.
pub struct CppDependencyRewriter;

impl ariadne_lang::DependencyRewriter for CppDependencyRewriter {
    fn rewrite(&self, _source: &str, _spec: &RewriteSpec) -> String {
        unimplemented!("CppDependencyRewriter lands in M18.8 (plan §3.6)")
    }
}

/// Placeholder for real compiler invocation: `validate_compile` reports a
/// failure (never a silent pass) until tier-3 wiring lands in M18.10/M23.
/// `toolchain_status()` keeps the trait default (`Unknown`).
pub struct CppCompilerValidator;

impl CompilerValidator for CppCompilerValidator {
    fn validate_compile(&self, _files: &[PathBuf], _args: &[String]) -> Result<(), String> {
        Err("C++ CompilerValidator not wired yet (M18.10/M23)".to_string())
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

    /// Bump when the tokenizer/`ParserCtx`/analyzer behaviour changes.
    fn analysis_fingerprint(&self) -> String {
        "cpp-analyzer-1".to_string()
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
