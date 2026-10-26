//! [`RustPlugin`] - the registry entry point for Rust (rust-language R-3).
//!
//! Wires the real kickoff components ([`crate::detector::RustDetector`] and
//! [`crate::shaper::RustUnitShaper`]) and the honest never-dispatched stubs
//! (`RustAnalyzer`, `RustImportResolver`, `RustClusteringConstraints`,
//! `RustSourceExtractor`, `RustDependencyRewriter`, `RustToolchainValidator`)
//! into `ariadne_lang::LanguagePlugin`. The crate is UNREGISTERED at this hop
//! (registration lands at R-4) and every capability bit is false: INV-5
//! (capability monotonicity) in its strongest form - no pipeline stage can be
//! dispatched to this plugin until the component behind its bit lands and the
//! per-hand flip turns the bit on.

use crate::analyzer::RustAnalyzer;
use crate::boundary::EXTERNAL_PATH_MARKERS;
use crate::constraints::RustClusteringConstraints;
use crate::dependency_rewriter::RustDependencyRewriter;
use crate::detector::RustDetector;
use crate::extractor::RustSourceExtractor;
use crate::resolver::RustImportResolver;
use crate::shaper::RustUnitShaper;
use crate::validator::RustToolchainValidator;
use ariadne_lang::{
    AggregationKind, ClusteringConstraints, CompilerValidator, CyclePolicy, DependencyResolver,
    DependencyRewriter, DependencyVocabulary, GuardKind, LanguageCapabilities, LanguageDetector,
    LanguageId, LanguageModeling, LanguagePlugin, SemanticAnalyzer, SourceExtractor,
    SymbolPlacement, UnitKind, UnitShaper,
};

/// D-2 honest limit (shape sec. 3.3 `analysis_caveat` row): the disclaimer
/// the CLI prints for Rust runs. Lives here as plugin data - core crates
/// never hard-code it. The literal must contain the word "macro" (test
/// predicate idiom mirrored from Scala's `contains("heuristic")`); the
/// `macro_rules!` spelling below carries it.
const RUST_ANALYSIS_CAVEAT: &str =
    "written-code analysis only: macro expansion is out of scope, so include! \
     and macro_rules! generated mod structure is not seen";

/// The Rust language plugin (shape sec. 3.3 modeling decision table).
///
/// Stateless at kickoff: every component is a value struct, so one
/// instance serves every repository, mirroring the C++/Scala plugin stance.
pub struct RustPlugin {
    detector: RustDetector,
    shaper: RustUnitShaper,
    analyzer: RustAnalyzer,
    resolver: RustImportResolver,
    constraints: RustClusteringConstraints,
    extractor: RustSourceExtractor,
    rewriter: RustDependencyRewriter,
    validator: RustToolchainValidator,
}

impl RustPlugin {
    pub fn new() -> Self {
        Self {
            detector: RustDetector,
            shaper: RustUnitShaper,
            analyzer: RustAnalyzer::new(),
            resolver: RustImportResolver,
            constraints: RustClusteringConstraints,
            extractor: RustSourceExtractor,
            rewriter: RustDependencyRewriter,
            validator: RustToolchainValidator,
        }
    }
}

impl Default for RustPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguagePlugin for RustPlugin {
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
        LanguageId::Rust
    }

    /// Shape sec. 3.3 decision table, Rust column: one `.rs` file is a
    /// module (source-file units); `mod x;` + `mod.rs`/`x.rs` sibling is
    /// Rust's aggregation mechanism (the appended `ModDeclaration` variant -
    /// PackageObject naming would lie, None would understate); no duplicate
    /// definition guard; single-file symbol placement (declared record only,
    /// no dispatch); mod declarations re-aggregate no code; `use`-edge cycles
    /// inside one crate's module tree are legal Rust; boundary markers are
    /// the shared [`EXTERNAL_PATH_MARKERS`] list, single-sourced so modeling
    /// and the future resolver cannot drift; use-flavored wording matches the
    /// `use` keyword; the caveat names the macro limit (D-2).
    fn modeling(&self) -> LanguageModeling {
        LanguageModeling {
            language: LanguageId::Rust,
            unit: UnitKind::SourceFile,
            aggregation: AggregationKind::ModDeclaration,
            guard: GuardKind::None,
            symbol_placement: SymbolPlacement::SingleFile,
            aggregation_reexports: false,
            cycle_policy: CyclePolicy::TolerateWithinPackage,
            external_path_markers: EXTERNAL_PATH_MARKERS
                .iter()
                .map(|marker| marker.to_string())
                .collect(),
            dependency_vocabulary: DependencyVocabulary {
                directive_label: "Use".into(),
                directive_passive_verb: "used".into(),
                aggregation_noun: "mod aggregation".into(),
            },
            analysis_caveat: Some(RUST_ANALYSIS_CAVEAT.into()),
        }
    }

    /// ALL-FALSE skeleton literal (INV-5, strongest form): the crate is
    /// unregistered and no component behind any bit exists in this hop.
    /// `requires_toolchain` is false at every hop (A3/D-2): the pipeline
    /// never invokes a Rust compile.
    fn capabilities(&self) -> LanguageCapabilities {
        LanguageCapabilities {
            discovery: false,
            dependency_graph: false,
            semantic_index: false,
            load_metric: false,
            planning: false,
            extraction: false,
            rewriting: false,
            validation: false,
            requires_toolchain: false,
        }
    }

    // NO analysis_fingerprint override at this hop: the trait default "v1"
    // stays. The "rust-1" fingerprint lands at R-5 in the same commit as the
    // real `syn` analyzer it fingerprints (INV-8).
    //
    // NO aggregation_filter / target_path_for overrides: the trait defaults
    // (None = planning unsupported) stand until planning enrollment lands at
    // R-9 (INV-5).

    /// The Rust shaper owns the cargo-layout unit-shaping literals
    /// (shape sec. 3.5).
    fn shaper(&self) -> Option<&dyn UnitShaper> {
        Some(&self.shaper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_identity_and_wired_components() {
        let plugin = RustPlugin::new();
        assert_eq!(plugin.id(), LanguageId::Rust);
        assert_eq!(plugin.detector().name(), "rust");
        let shaper = plugin.shaper().expect("RustUnitShaper landed at R-3");
        assert_eq!(shaper.build_markers(), vec!["Cargo.toml"]);
        assert_eq!(shaper.default_compiler(), "rustc");
    }

    /// Byte-exact literal lock of all ten modeling fields (shape sec. 3.3).
    #[test]
    fn modeling_matches_the_shape_decision_table() {
        let modeling = RustPlugin::new().modeling();
        assert_eq!(modeling.language, LanguageId::Rust);
        assert_eq!(modeling.unit, UnitKind::SourceFile);
        assert_eq!(modeling.aggregation, AggregationKind::ModDeclaration);
        assert_eq!(modeling.guard, GuardKind::None);
        assert_eq!(modeling.symbol_placement, SymbolPlacement::SingleFile);
        assert!(
            !modeling.aggregation_reexports,
            "a mod declaration re-aggregates no code; `pub use` is a separate \
             explicit idiom, not a property of the mechanism"
        );
        assert_eq!(modeling.cycle_policy, CyclePolicy::TolerateWithinPackage);
        assert_eq!(
            modeling.external_path_markers,
            vec![
                "target".to_string(),
                "vendor".to_string(),
                ".cargo".to_string()
            ]
        );
        assert_eq!(modeling.dependency_vocabulary.directive_label, "Use");
        assert_eq!(
            modeling.dependency_vocabulary.directive_passive_verb,
            "used"
        );
        assert_eq!(
            modeling.dependency_vocabulary.aggregation_noun,
            "mod aggregation"
        );
        assert_eq!(
            modeling.analysis_caveat.as_deref(),
            Some(RUST_ANALYSIS_CAVEAT),
            "D-2: the macro-expansion limit is declared, never silent"
        );
        assert!(
            modeling.analysis_caveat.is_some_and(|caveat| caveat.contains("macro")),
            "the caveat names the macro limit by word (Scala's contains idiom)"
        );
    }

    /// Byte-exact capability literal lock (pattern of Scala's
    /// `capabilities_match_the_m23_flip_literal`, plugin.rs:431): the R-3
    /// skeleton is ALL-FALSE - unregistered crate, no component landed yet.
    #[test]
    fn capabilities_match_the_skeleton_literal() {
        assert_eq!(
            RustPlugin::new().capabilities(),
            LanguageCapabilities {
                discovery: false,
                dependency_graph: false,
                semantic_index: false,
                load_metric: false,
                planning: false,
                extraction: false,
                rewriting: false,
                validation: false,
                requires_toolchain: false,
            }
        );
    }

    /// INV-5 default-seam pin: no fingerprint bump and no planning-hook
    /// enrollment before their components exist.
    #[test]
    fn plugin_default_seams_stay_at_trait_defaults() {
        use std::path::Path;
        let plugin = RustPlugin::new();
        assert_eq!(
            plugin.analysis_fingerprint(),
            "v1",
            "R-3 keeps the trait default; \"rust-1\" rides the real analyzer at R-5 (INV-8)"
        );
        assert_eq!(
            plugin.aggregation_filter(Path::new("."), Path::new("src/lib.rs")),
            None,
            "planning enrollment is R-9's hop (INV-5)"
        );
        assert_eq!(
            plugin.target_path_for(Path::new("src/lib.rs"), "Widget"),
            None,
            "no extraction target naming before planning enrolls (INV-5)"
        );
    }
}
