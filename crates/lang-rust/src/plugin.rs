//! [`RustPlugin`] - the registry entry point for Rust (rust-language R-3,
//! registered at R-4).
//!
//! Wires the real kickoff components ([`crate::detector::RustDetector`] and
//! [`crate::shaper::RustUnitShaper`]) and the honest never-dispatched stubs
//! (`RustAnalyzer`, `RustImportResolver`, `RustClusteringConstraints`,
//! `RustSourceExtractor`, `RustDependencyRewriter`, `RustToolchainValidator`)
//! into `ariadne_lang::LanguagePlugin`. Registered at R-4 (appended LAST in
//! the composition root); the R-4 capability flip turns on `discovery` ONLY
//! (detector + shaper + fixture evidence exist): INV-5 (capability
//! monotonicity) - no pipeline stage can be dispatched to this plugin until
//! the component behind its bit lands and the per-hand flip turns the bit
//! on.

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

    /// The R-5 semantic-index flip literal (INV-5, monotone): `discovery`
    /// (R-4) stays and `semantic_index` turns on with this hop's real `syn`
    /// analyzer; every later stage's component lands with its own hand and
    /// its own bit. `requires_toolchain` is false at every hop (A3/D-2): the
    /// pipeline never invokes a Rust compile.
    fn capabilities(&self) -> LanguageCapabilities {
        LanguageCapabilities {
            discovery: true,
            dependency_graph: false,
            semantic_index: true,
            load_metric: false,
            planning: false,
            extraction: false,
            rewriting: false,
            validation: false,
            requires_toolchain: false,
        }
    }

    /// INV-8: `"rust-1"` rides the first analyzer-shipping commit (this one)
    /// and is the per-plugin cache-digest input; bumps ride future parser
    /// changes. The C++/Scala fingerprint literals are separate digest inputs,
    /// unedited; the cache epoch `CURRENT_ANALYSIS_VERSION` is never the
    /// vehicle (frozen by M27 - zero bytes under `crates/index`).
    fn analysis_fingerprint(&self) -> String {
        "rust-1".to_string()
    }

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
    /// `capabilities_match_the_m23_flip_literal`, plugin.rs:431): the R-5
    /// semantic-index flip turns on `discovery` + `semantic_index` and ONLY
    /// those - an INDEPENDENT full-struct literal, the D-M23-6 style Scala
    /// proved catches bit slips. Renamed from the R-4 registration-pin of
    /// the same discipline (amendment-style byte-pin, flipped WITH the flip).
    #[test]
    fn capabilities_match_the_semantic_index_flip_literal() {
        assert_eq!(
            RustPlugin::new().capabilities(),
            LanguageCapabilities {
                discovery: true,
                dependency_graph: false,
                semantic_index: true,
                load_metric: false,
                planning: false,
                extraction: false,
                rewriting: false,
                validation: false,
                requires_toolchain: false,
            }
        );
    }

    /// R-4 in-src pin: with the plugin registry-dispatchable, `RustPlugin`
    /// answers `detect_language` for `.rs` (the detector behavior itself is
    /// already pinned from R-3 in `detector.rs`; this locks the wiring the
    /// registration routes through).
    #[test]
    fn rust_plugin_answers_detect_language_for_rs_paths() {
        use ariadne_lang::SourceFileKind;
        use std::path::Path;
        let plugin = RustPlugin::new();
        assert_eq!(
            plugin.detector().detect(Path::new("widget.rs")),
            SourceFileKind::Source,
            "post-registration the registry asks THIS detector for .rs"
        );
        assert_eq!(
            plugin.detector().detect(Path::new("src/lib.rs")),
            SourceFileKind::Source
        );
    }

    /// R-5 in-src pin (INV-8): the fingerprint literal `"rust-1"` lands in
    /// the FIRST commit shipping an analyzer. Bumps ride future parser
    /// changes; the C++/Scala fingerprint literals are separate digest inputs,
    /// unedited (cache digests for their fixtures must not move).
    #[test]
    fn analysis_fingerprint_is_rust_1() {
        assert_eq!(
            RustPlugin::new().analysis_fingerprint(),
            "rust-1",
            "INV-8: the fingerprint rides the real syn analyzer at R-5"
        );
    }

    /// INV-5 default-seam pin: no planning-hook enrollment before R-9. The
    /// fingerprint left the default seam at R-5 with the analyzer it
    /// fingerprints - pinned separately by
    /// `analysis_fingerprint_is_rust_1` (the R-3/R-4 era's `"v1"` assertion
    /// lived here and retired with the override, journal D4).
    #[test]
    fn plugin_default_seams_stay_at_trait_defaults() {
        use std::path::Path;
        let plugin = RustPlugin::new();
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
