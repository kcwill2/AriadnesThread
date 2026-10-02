//! [`ScalaPlugin`] - the registry entry point for Scala (plan M19.1).
//!
//! Wires the kickoff components (`ScalaDetector`, `ScalaUnitShaper`), the
//! M20.1 resolver ([`crate::resolver::ScalaImportResolver`]), the M21
//! analyzer ([`crate::analyzer::ScalaAnalyzer`] over the M21 parser), and
//! the documented milestone stubs (M23, see [`crate::stubs`]) into
//! `ariadne_lang::LanguagePlugin`. The capability literals below are the
//! gate that keeps the pipeline from reaching the stubs whose milestones
//! have not landed.

use crate::analyzer::ScalaAnalyzer;
use crate::boundary::EXTERNAL_PATH_MARKERS;
use crate::constraints::ScalaClusteringConstraints;
use crate::detector::ScalaDetector;
use crate::resolver::ScalaImportResolver;
use crate::shaper::ScalaUnitShaper;
use crate::stubs::{StubExtractor, StubRewriter, StubValidator};
use ariadne_lang::{
    AggregationKind, ClusteringConstraints, CompilerValidator, CyclePolicy, DependencyResolver,
    DependencyRewriter, DependencyVocabulary, GuardKind, LanguageCapabilities, LanguageDetector,
    LanguageId, LanguageModeling, LanguagePlugin, SemanticAnalyzer, SourceExtractor,
    SymbolPlacement, UnitKind, UnitShaper,
};

/// D26 (plan:509): the disclaimer the CLI prints for runs that analyzed
/// Scala. Lives here as plugin data — core crates never hard-code it.
const SCALA_ANALYSIS_CAVEAT: &str =
    "offline heuristic parser — exact types require the Scala toolchain analyzer";

/// The Scala language plugin (plan sec 3.4 modeling table, M19.1).
///
/// Stateless at kickoff: every component is a value struct, so one
/// instance serves every repository, mirroring [`ariadne_lang::
/// LanguageRegistry`]'s C++ plugin stance.
pub struct ScalaPlugin {
    detector: ScalaDetector,
    shaper: ScalaUnitShaper,
    analyzer: ScalaAnalyzer,
    resolver: ScalaImportResolver,
    constraints: ScalaClusteringConstraints,
    extractor: StubExtractor,
    rewriter: StubRewriter,
    validator: StubValidator,
}

impl ScalaPlugin {
    pub fn new() -> Self {
        Self {
            detector: ScalaDetector,
            shaper: ScalaUnitShaper,
            analyzer: ScalaAnalyzer::new(),
            resolver: ScalaImportResolver,
            constraints: ScalaClusteringConstraints,
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
    /// tolerated within a package. External boundary markers are the
    /// shared [`EXTERNAL_PATH_MARKERS`] list (M19 lock, M20 resolver) —
    /// single-sourced so modeling and resolution cannot drift.
    fn modeling(&self) -> LanguageModeling {
        LanguageModeling {
            language: LanguageId::Scala,
            unit: UnitKind::SourceFile,
            aggregation: AggregationKind::PackageObject,
            guard: GuardKind::None,
            symbol_placement: SymbolPlacement::SingleFile,
            aggregation_reexports: false,
            cycle_policy: CyclePolicy::TolerateWithinPackage,
            external_path_markers: EXTERNAL_PATH_MARKERS
                .iter()
                .map(|marker| marker.to_string())
                .collect(),
            // D26 (plan:425 + risk 509): import-flavored wording and the
            // heuristic caveat the CLI prints to stdout (never manifests).
            dependency_vocabulary: DependencyVocabulary {
                directive_label: "Import".into(),
                directive_passive_verb: "imported".into(),
                aggregation_noun: "package aggregation".into(),
            },
            analysis_caveat: Some(SCALA_ANALYSIS_CAVEAT.into()),
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

    /// M21 bump (D19): the M21 Scala parser replaced the stub analyzer, so
    /// the cache digest input moves from M20.1's "scala-1" to
    /// "scala-parser-1". C++'s fingerprint literal is a separate digest
    /// input and stays where it is (m18 parity).
    fn analysis_fingerprint(&self) -> String {
        "scala-parser-1".to_string()
    }

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
            "scala-parser-1",
            "bumped with the M21 parser (D19): ScalaAnalyzer/ScalaSourceParser are new \
             analysis behavior feeding the cache digest; M20.1 introduced this lock at \
             \"scala-1\" over the trait default \"v1\"; C++ keeps its own \"v1\" input \
             untouched"
        );
        assert_eq!(plugin.detector().name(), "scala");
        let shaper = plugin.shaper().expect("ScalaUnitShaper landed in M19.2");
        assert_eq!(
            shaper.build_markers(),
            vec!["build.sbt", "build.sc", "pom.xml"]
        );
    }

    #[test]
    fn plugin_resolver_reads_the_named_edges_from_real_fixture_source() {
        use ariadne_lang::{DirectiveStyle, ResolutionContext};
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../tests/fixtures/scala-basic/src/main/scala/com/example/services/OrderService.scala",
        );
        let source = std::fs::read_to_string(&fixture)
            .unwrap_or_else(|error| panic!("fixture {} readable: {error}", fixture.display()));
        let edges = ScalaPlugin::new().resolver().resolve_edges(
            FileId::from_relative_path("src/main/scala/com/example/services/OrderService.scala"),
            std::path::Path::new("src/main/scala/com/example/services/OrderService.scala"),
            &source,
            &ResolutionContext::default(),
        );
        let targets: Vec<&str> = edges.iter().map(|edge| edge.raw_target.as_str()).collect();
        // D27 fixture evolution: the selector import `com.example.models.{Order, User}`
        // became `import com.example.models._`, so the directive edge carries the
        // package (not the `._` spelling) instead of two named targets.
        assert_eq!(
            targets,
            vec!["com.example.models", "scala.collection.mutable",],
            "the package clause emits no edge and the interpolated string emits none; both \
             imports are wildcard spelling and each carries its package, not the `._` suffix"
        );
        assert!(edges
            .iter()
            .all(|edge| edge.style == DirectiveStyle::Unspecified));
        assert_eq!(
            edges.iter().map(|edge| edge.line).collect::<Vec<u32>>(),
            vec![3, 4],
            "the models wildcard import sits on line 3, the mutable wildcard import on line 4"
        );
        assert!(edges.iter().all(|edge| edge.column == 1));
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
        // D26 added dependency_vocabulary + analysis_caveat: Scala renders
        // import-flavored wording and carries the heuristic caveat.
        assert_eq!(modeling.dependency_vocabulary.directive_label, "Import");
        assert_eq!(
            modeling.dependency_vocabulary.directive_passive_verb,
            "imported"
        );
        assert_eq!(
            modeling.dependency_vocabulary.aggregation_noun,
            "package aggregation"
        );
        assert_eq!(
            modeling.analysis_caveat.as_deref(),
            Some(SCALA_ANALYSIS_CAVEAT),
            "plan:509 — Scala output is labelled heuristic"
        );
    }

    #[test]
    fn plugin_constraints_are_the_scala_clustering_constraints() {
        use ariadne_lang::{SymbolId, SymbolKind, SymbolRecord};
        let batch = vec![
            SymbolRecord {
                id: SymbolId::new("com.example.Counter"),
                name: "Counter".to_string(),
                kind: SymbolKind::Class,
                ..SymbolRecord::default()
            },
            SymbolRecord {
                id: SymbolId::new("com.example.Counter$"),
                name: "Counter".to_string(),
                kind: SymbolKind::Object,
                ..SymbolRecord::default()
            },
        ];
        let plugin = ScalaPlugin::new();
        let groups = plugin.constraints().colocated_groups(&batch);
        assert_eq!(
            groups,
            vec![vec![
                SymbolId::new("com.example.Counter"),
                SymbolId::new("com.example.Counter$")
            ]],
            "D24: the plugin serves ScalaClusteringConstraints, not the retired stub \
             (which always returned an empty Vec)"
        );
        assert!(
            plugin.constraints().frozen_symbols(&batch).is_empty(),
            "IV3: frozen stays empty — the policy layer owns freezing"
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
    fn analyzer_reports_real_scala_analysis_through_the_plugin() {
        let plugin = ScalaPlugin::new();
        let file_id = FileId::from_relative_path("src/main/scala/com/example/Main.scala");
        let analysis = plugin.analyzer().analyze(file_id, "package com.example\n\nobject Main {}", "root");
        assert_eq!(analysis.tu_file_id, file_id);
        assert_eq!(analysis.language, SourceLanguage::Scala);
        assert_eq!(analysis.context_name, "root");
        assert_eq!(
            analysis
                .symbols
                .iter()
                .map(|symbol| symbol.id.0.clone())
                .collect::<Vec<String>>(),
            vec!["com.example.Main"],
            "M21 parser wired: the stub's empty analysis is retired"
        );
    }

    #[test]
    fn analyzer_yields_the_fixture_symbol_with_wildcard_import_edges() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../tests/fixtures/scala-basic/src/main/scala/com/example/services/OrderService.scala",
        );
        let source = std::fs::read_to_string(&fixture)
            .unwrap_or_else(|error| panic!("fixture {} readable: {error}", fixture.display()));
        let analysis = ScalaPlugin::new().analyzer().analyze(
            FileId::from_relative_path("src/main/scala/com/example/services/OrderService.scala"),
            &source,
            "scala-basic",
        );
        let symbol_ids: Vec<&str> = analysis
            .symbols
            .iter()
            .map(|symbol| symbol.id.0.as_str())
            .collect();
        assert_eq!(symbol_ids, vec!["com.example.services.OrderService"]);
        // D27 fixture evolution (`import com.example.models._`): the two
        // plan:413 usage edges now ride the active wildcard and carry
        // D16's Incomplete kind; the ArrayBuffer guess (also Incomplete,
        // guessed into the first wildcard's package) is not part of this
        // plan:413 witness.
        let usage: Vec<(String, String)> = analysis
            .edges
            .iter()
            .filter(|edge| edge.kind == ariadne_lang::DependencyKind::Incomplete)
            .filter(|edge| {
                edge.dst_symbol.0 == "com.example.models.User"
                    || edge.dst_symbol.0 == "com.example.models.Order"
            })
            .map(|edge| (edge.src_symbol.0.clone(), edge.dst_symbol.0.clone()))
            .collect();
        assert_eq!(
            usage,
            vec![
                (
                    "com.example.services.OrderService".to_string(),
                    "com.example.models.User".to_string()
                ),
                (
                    "com.example.services.OrderService".to_string(),
                    "com.example.models.Order".to_string()
                ),
            ],
            "plan:413 acceptance edges via the active wildcard import, in source order \
             (`summarize(user: User, order: Order)` names User before Order)"
        );
    }
}
