//! [`ScalaPlugin`] - the registry entry point for Scala (plan M19.1).
//!
//! Wires the kickoff components (`ScalaDetector`, `ScalaUnitShaper`) and
//! the documented milestone stubs (M20/M21/M23, see [`crate::stubs`]) into
//! `ariadne_lang::LanguagePlugin`. The capability literals below are the
//! gate that keeps the pipeline from reaching the stubs whose milestones
//! have not landed.

use crate::detector::ScalaDetector;
use crate::shaper::ScalaUnitShaper;
use crate::stubs::{
    StubAnalyzer, StubConstraints, StubExtractor, StubResolver, StubRewriter, StubValidator,
};
use ariadne_lang::{
    AggregationKind, ClusteringConstraints, CompilerValidator, CyclePolicy, DependencyResolver,
    DependencyRewriter, GuardKind, LanguageCapabilities, LanguageDetector, LanguageId,
    LanguageModeling, LanguagePlugin, SemanticAnalyzer, SourceExtractor, SymbolPlacement,
    UnitKind, UnitShaper,
};

/// The Scala language plugin (plan sec 3.4 modeling table, M19.1).
///
/// Stateless at kickoff: every component is a value struct, so one
/// instance serves every repository, mirroring [`ariadne_lang::
/// LanguageRegistry`]'s C++ plugin stance.
pub struct ScalaPlugin {
    detector: ScalaDetector,
    shaper: ScalaUnitShaper,
    analyzer: StubAnalyzer,
    resolver: StubResolver,
    constraints: StubConstraints,
    extractor: StubExtractor,
    rewriter: StubRewriter,
    validator: StubValidator,
}

impl ScalaPlugin {
    pub fn new() -> Self {
        Self {
            detector: ScalaDetector,
            shaper: ScalaUnitShaper,
            analyzer: StubAnalyzer,
            resolver: StubResolver,
            constraints: StubConstraints,
            extractor: StubExtractor,
            rewriter: StubRewriter,
            validator: StubValidator,
        }
    }
}

impl Default for ScalaPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguagePlugin for ScalaPlugin {
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
        LanguageId::Scala
    }

    /// Plan sec 3.4 Scala column: source-file units, package-object
    /// aggregation that re-exports nominally (no C++-style re-aggregation),
    /// no duplicate-definition guard, single-file symbol placement, cycles
    /// tolerated within a package. External boundary markers are M20's
    /// dependency-directory list; the plugin must supply it now (plan
    /// sec 3.4 + M20 marker list).
    fn modeling(&self) -> LanguageModeling {
        LanguageModeling {
            language: LanguageId::Scala,
            unit: UnitKind::SourceFile,
            aggregation: AggregationKind::PackageObject,
            guard: GuardKind::None,
            symbol_placement: SymbolPlacement::SingleFile,
            aggregation_reexports: false,
            cycle_policy: CyclePolicy::TolerateWithinPackage,
            external_path_markers: vec![
                "lib".to_string(),
                "target".to_string(),
                ".ivy2".to_string(),
                ".cache".to_string(),
            ],
        }
    }

    /// Discovery through load metric work offline at kickoff; planning and
    /// the extraction-side stages stay off until their milestones
    /// (plan:381). `requires_toolchain: false` per plan assumption A3:
    /// kickoff never invokes scalac/sbt.
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

    // `analysis_fingerprint()` intentionally keeps the trait default "v1":
    // it is bumped together with the M21 Scala parser, the first plugin
    // behavior that feeds a cache fingerprint (plan sec 3.5).

    /// The Scala shaper owns the sbt-layout unit-shaping literals
    /// (plan M19.2).
    fn shaper(&self) -> Option<&dyn UnitShaper> {
        Some(&self.shaper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_repo::{FileId, SourceLanguage};

    #[test]
    fn plugin_identity_and_fingerprint() {
        let plugin = ScalaPlugin::new();
        assert_eq!(plugin.id(), LanguageId::Scala);
        assert_eq!(
            plugin.analysis_fingerprint(),
            "v1",
            "trait default until the M21 parser bumps it"
        );
        assert_eq!(plugin.detector().name(), "scala");
        let shaper = plugin.shaper().expect("ScalaUnitShaper landed in M19.2");
        assert_eq!(
            shaper.build_markers(),
            vec!["build.sbt", "build.sc", "pom.xml"]
        );
    }

    #[test]
    fn modeling_matches_plan_section_3_4_scala_column() {
        let modeling = ScalaPlugin::new().modeling();
        assert_eq!(modeling.language, LanguageId::Scala);
        assert_eq!(modeling.unit, UnitKind::SourceFile);
        assert_eq!(modeling.aggregation, AggregationKind::PackageObject);
        assert_eq!(modeling.guard, GuardKind::None);
        assert_eq!(modeling.symbol_placement, SymbolPlacement::SingleFile);
        assert!(!modeling.aggregation_reexports);
        assert_eq!(modeling.cycle_policy, CyclePolicy::TolerateWithinPackage);
        assert_eq!(
            modeling.external_path_markers,
            vec![
                "lib".to_string(),
                "target".to_string(),
                ".ivy2".to_string(),
                ".cache".to_string()
            ]
        );
    }

    #[test]
    fn capabilities_match_the_plan_381_literal() {
        assert_eq!(
            ScalaPlugin::new().capabilities(),
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
        );
    }

    #[test]
    fn stub_analyzer_reports_empty_scala_analysis_through_the_plugin() {
        let plugin = ScalaPlugin::new();
        let file_id = FileId::from_relative_path("src/main/scala/com/example/Main.scala");
        let analysis = plugin.analyzer().analyze(file_id, "object Main {}", "root");
        assert_eq!(analysis.tu_file_id, file_id);
        assert_eq!(analysis.language, SourceLanguage::Scala);
        assert!(analysis.symbols.is_empty());
    }
}
