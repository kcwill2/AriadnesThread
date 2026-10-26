//! [`RustPlugin`] - the registry entry point for Rust (rust-language R-3,
//! registered at R-4).
//!
//! Wires the real components into `ariadne_lang::LanguagePlugin`: the
//! detector, shaper, boundary markers and modeling literals landed at R-3;
//! the `syn` analyzer at R-5; the import resolver at R-6; the source
//! extractor and the dependency rewriter (use-path rewrite plus the
//! RISK-MODTREE parent-`mod` consumer mechanism,
//! [`crate::dependency_rewriter`]) at R-8. The planning hook pair
//! ([`crate::planning`], ADR-0002 Decision 3 - registry dispatch enrolls the
//! overrides below with ZERO CLI edit) and the probe-only toolchain validator
//! ([`crate::validator`]) land at R-9b, completing the enrollment: every
//! capability bit now sits behind a shipped component. Registration appended the
//! plugin LAST in the composition root at R-4; the capability literal
//! tracks component landing per hand under INV-5 (capability
//! monotonicity) - no pipeline stage is dispatched to a component whose
//! bit is off, and no bit turns on before its component exists.

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
            validator: RustToolchainValidator::new(),
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

    /// The FINAL all-true parity literal (INV-5, monotone: every bit landed
    /// WITH its component - `discovery` R-4, `semantic_index` R-5,
    /// `dependency_graph` R-6, `load_metric` R-7 (R-7 owns it; it does not
    /// re-appear here), the extraction/rewriting mechanism R-8 + R-9a,
    /// `planning` this hop's hook pair (the overrides below, retiring the
    /// trait defaults), `validation` this hop's probe-only validator - no
    /// bit precedes its component, and the hop that ships a component flips
    /// exactly its bits).
    /// `requires_toolchain` is false at every hop (A3/D-2): the pipeline never
    /// invokes a Rust compile.
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

    /// INV-8: `"rust-1"` rides the first analyzer-shipping commit (this one)
    /// and is the per-plugin cache-digest input; bumps ride future parser
    /// changes. The C++/Scala fingerprint literals are separate digest inputs,
    /// unedited; the cache epoch `CURRENT_ANALYSIS_VERSION` is never the
    /// vehicle (frozen by M27 - zero bytes under `crates/index`).
    fn analysis_fingerprint(&self) -> String {
        "rust-1".to_string()
    }

    /// R-9b enrollment (ADR-0002 Decision 3 pair): a file is an aggregation
    /// unit when it parses and declares >= 1 out-of-line `mod x;` item
    /// (shape sec. 3.5); delegating to [`crate::planning::aggregation_filter`].
    /// Registry dispatch (`crates/cli/src/registry.rs` `planning_hooks`)
    /// enrolls these overrides with ZERO CLI edit - the trait default `None`
    /// ("planning unsupported") is retired WITH the planning bit.
    fn aggregation_filter(
        &self,
        repo_root: &std::path::Path,
        path: &std::path::Path,
    ) -> Option<bool> {
        crate::planning::aggregation_filter(repo_root, path)
    }

    /// R-9b enrollment: the extraction target is the sanitized
    /// `{symbol}.rs` beside the aggregation file, delegating to
    /// [`crate::planning::target_path_for`]. One grammar with the
    /// move-retention hook's `mod <stem>;` insertions -
    /// `sanitize_module_stem` is the single source of truth (R-9a rode it).
    fn target_path_for(
        &self,
        aggregation_path: &std::path::Path,
        symbol_name: &str,
    ) -> Option<std::path::PathBuf> {
        crate::planning::target_path_for(aggregation_path, symbol_name)
    }

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
    /// `capabilities_match_the_m23_flip_literal`, plugin.rs:431): the R-9b
    /// parity flip turns ALL EIGHT stage bits true - every one behind a
    /// shipped component (`discovery` R-4, `semantic_index` R-5,
    /// `dependency_graph` R-6, `load_metric` R-7, the extraction/rewriting
    /// mechanism R-8 + R-9a, `planning` this hop's hook pair, `validation`
    /// this hop's probe-only validator), an INDEPENDENT full-struct literal,
    /// the D-M23-6 style Scala proved catches bit slips. Renamed from the
    /// R-7 load-metric pin of the same discipline (implementer-band rename,
    /// amendment sec. 9 - flipped WITH the flip). `requires_toolchain` is
    /// false at every hop including this one (A3/D-2).
    #[test]
    fn capabilities_match_the_parity_flip_literal() {
        assert_eq!(
            RustPlugin::new().capabilities(),
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

    /// R-9b evolution (amendment sec. 9): the two hook legs turned from
    /// trait-default `None` pins into Some-dispatch pins the moment planning
    /// enrolled; the pins name the exact hook bytes - an out-of-line `mod x;`
    /// IS an aggregation unit, an inline `mod x { .. }` block is NOT,
    /// unreadable input answers the honest `Some(false)`, and target naming
    /// is the sanitized `{symbol}.rs` beside the aggregation file. The
    /// fingerprint seam was never part of this test; it is pinned separately
    /// by `analysis_fingerprint_is_rust_1` (the R-3/R-4 era's `"v1"`
    /// assertion lived here and retired with the override, R-5 journal D4).
    #[test]
    fn plugin_default_seams_stay_at_trait_defaults() {
        use std::path::{Path, PathBuf};
        let dir = tempfile::tempdir().expect("temp hook-specimen dir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).expect("specimen src dir");
        std::fs::write(root.join("src").join("lib.rs"), "mod greeting;\nmod models;\n")
            .expect("out-of-line mod specimen");
        std::fs::write(
            root.join("src").join("inline.rs"),
            "mod inlined {\n    pub fn buried() {}\n}\n",
        )
        .expect("inline mod specimen");
        let plugin = RustPlugin::new();
        assert_eq!(
            plugin.aggregation_filter(root, Path::new("src/lib.rs")),
            Some(true),
            "planning enrolled at R-9b: an out-of-line `mod x;` declaration makes the file an aggregation unit (INV-5 - the bit ships with its hooks)"
        );
        assert_eq!(
            plugin.aggregation_filter(root, Path::new("src/inline.rs")),
            Some(false),
            "an inline `mod x {{ .. }}` block declares no aggregation child (handoff item 1)"
        );
        assert_eq!(
            plugin.aggregation_filter(root, Path::new("src/absent.rs")),
            Some(false),
            "unreadable input is honestly not an aggregation unit - the filter invents nothing (Scala parity)"
        );
        assert_eq!(
            plugin.target_path_for(Path::new("src/lib.rs"), "fixture_marker"),
            Some(PathBuf::from("src/fixture_marker.rs")),
            "planning enrolled at R-9b: target naming is the sanitized `{{symbol}}.rs` beside the aggregation file"
        );
    }
}
