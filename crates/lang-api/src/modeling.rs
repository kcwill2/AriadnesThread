//! Language modeling vocabulary (plan §3.2): replaces ".h"/TU assumptions
//! spread across the core pipeline with data supplied by each plugin.

use crate::identity::LanguageId;
use serde::{Deserialize, Serialize};

/// What one analysis unit is for this language (C++: TU; Scala: source file).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitKind {
    TranslationUnit,
    SourceFile,
}

/// What "owns" symbols and is re-loaded by consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationKind {
    Header,
    PackageObject,
    None,
    /// Rust's aggregation mechanism: a parent module's `mod x;` declaration
    /// paired with its sibling file (`x.rs` or `x/mod.rs`).
    /// `PackageObject` would lie for Rust and `None` would understate a
    /// real mechanism (rust-language shape sec. 3.3).
    /// FROZEN indices: pre-existing variants keep positional (bincode)
    /// indices 0/1/2 unmoved; this variant takes index 3 at the END (INV-12).
    ModDeclaration,
}

/// Duplicate-definition guard mechanism (was: hardcoded `#pragma once`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardKind {
    PragmaOnce,
    IncludeGuard,
    None,
}

/// Does one symbol's declaration and definition live in different files?
///
/// C++ (header declares, source defines) => [`SymbolPlacement::SplitDeclaration`].
/// Scala/Go/TS/Python (one file per top-level symbol) => [`SymbolPlacement::SingleFile`].
/// Drives symbol-isolation targets (plan §3.6) and reduced-model provenance (§3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolPlacement {
    SplitDeclaration,
    SingleFile,
}

/// How this language must treat dependency cycles for planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CyclePolicy {
    Reject,
    TolerateWithinPackage,
    Tolerate,
}

/// Wording one language contributes to the dependency-story output
/// (plan M22, D26): the CLI/graph render paths consume these fields instead
/// of language literals, so "Include"-flavored and "Import"-flavored runs
/// share one code path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyVocabulary {
    /// Label for one dependency directive ("Include" | "Import").
    pub directive_label: String,
    /// Passive voice of the directive ("included" | "imported").
    pub directive_passive_verb: String,
    /// Noun for the aggregation artifact ("header" | "package aggregation").
    pub aggregation_noun: String,
}

/// The complete modeling contract one plugin declares about its language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageModeling {
    pub language: LanguageId,
    pub unit: UnitKind,
    pub aggregation: AggregationKind,
    pub guard: GuardKind,
    pub symbol_placement: SymbolPlacement,
    /// Does an aggregation artifact physically re-aggregate dependency code
    /// (C++ header) or only re-export/nominalise it (Scala package object)?
    pub aggregation_reexports: bool,
    pub cycle_policy: CyclePolicy,
    /// Path components that mark the external (frozen) boundary; replaces
    /// `is_third_party_path(".h")`.
    pub external_path_markers: Vec<String>,
    /// D26 (plan:425): source of the CLI load/explain wording for this
    /// language; the default rendering stays whatever the plugin declares
    /// here, never a core-crate literal.
    pub dependency_vocabulary: DependencyVocabulary,
    /// D26 (plan:509): heuristic disclaimer the CLI prints to stdout for
    /// runs that analyzed this language; `None` for exact-toolchain
    /// languages. Deliberately NOT surfaced in manifests (D25 ruling).
    pub analysis_caveat: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// D26 added the `dependency_vocabulary` + `analysis_caveat` fields;
    /// these helpers carry them so the round-trip covers the D26 shape too.
    fn cpp_modeling() -> LanguageModeling {
        LanguageModeling {
            language: LanguageId::Cpp,
            unit: UnitKind::TranslationUnit,
            aggregation: AggregationKind::Header,
            guard: GuardKind::PragmaOnce,
            symbol_placement: SymbolPlacement::SplitDeclaration,
            aggregation_reexports: true,
            cycle_policy: CyclePolicy::Reject,
            external_path_markers: vec!["third_party".into(), "vendor".into()],
            dependency_vocabulary: cpp_vocabulary(),
            analysis_caveat: None,
        }
    }

    fn scala_modeling() -> LanguageModeling {
        LanguageModeling {
            language: LanguageId::Scala,
            unit: UnitKind::SourceFile,
            aggregation: AggregationKind::PackageObject,
            guard: GuardKind::None,
            symbol_placement: SymbolPlacement::SingleFile,
            aggregation_reexports: false,
            cycle_policy: CyclePolicy::TolerateWithinPackage,
            external_path_markers: vec!["lib".into(), "target".into()],
            dependency_vocabulary: scala_vocabulary(),
            analysis_caveat: Some(
                "offline heuristic parser — exact types require the Scala toolchain analyzer"
                    .into(),
            ),
        }
    }

    fn cpp_vocabulary() -> DependencyVocabulary {
        DependencyVocabulary {
            directive_label: "Include".into(),
            directive_passive_verb: "included".into(),
            aggregation_noun: "header".into(),
        }
    }

    fn scala_vocabulary() -> DependencyVocabulary {
        DependencyVocabulary {
            directive_label: "Import".into(),
            directive_passive_verb: "imported".into(),
            aggregation_noun: "package aggregation".into(),
        }
    }

    #[test]
    fn modeling_round_trips_through_serde() {
        for modeling in [cpp_modeling(), scala_modeling()] {
            let json = serde_json::to_string(&modeling).unwrap();
            let back: LanguageModeling = serde_json::from_str(&json).unwrap();
            assert_eq!(back, modeling);
        }
    }

    #[test]
    fn symbol_placement_round_trips_through_serde() {
        for placement in [
            SymbolPlacement::SplitDeclaration,
            SymbolPlacement::SingleFile,
        ] {
            let json = serde_json::to_string(&placement).unwrap();
            let back: SymbolPlacement = serde_json::from_str(&json).unwrap();
            assert_eq!(back, placement);
        }
    }

    #[test]
    fn dependency_vocabulary_round_trips_through_serde() {
        for vocabulary in [cpp_vocabulary(), scala_vocabulary()] {
            let json = serde_json::to_string(&vocabulary).unwrap();
            let back: DependencyVocabulary = serde_json::from_str(&json).unwrap();
            assert_eq!(back, vocabulary);
        }
    }

    #[test]
    fn analysis_caveat_is_none_for_cpp_and_some_for_scala() {
        assert_eq!(cpp_modeling().analysis_caveat, None);
        let scala_caveat = scala_modeling().analysis_caveat.expect("D26: Scala carries a caveat");
        assert!(scala_caveat.contains("heuristic"), "caveat names the heuristic: {scala_caveat}");
    }

    #[test]
    fn vocabulary_fields_survive_full_modeling_round_trip() {
        let json = serde_json::to_string(&scala_modeling()).unwrap();
        let back: LanguageModeling = serde_json::from_str(&json).unwrap();
        assert_eq!(back.dependency_vocabulary.directive_label, "Import");
        assert_eq!(back.dependency_vocabulary.directive_passive_verb, "imported");
        assert_eq!(
            back.dependency_vocabulary.aggregation_noun,
            "package aggregation"
        );
        assert!(back.analysis_caveat.is_some());
    }

    #[test]
    fn cpp_fixture_carries_split_declaration_scala_single_file() {
        assert_eq!(
            cpp_modeling().symbol_placement,
            SymbolPlacement::SplitDeclaration
        );
        assert_eq!(
            scala_modeling().symbol_placement,
            SymbolPlacement::SingleFile
        );
    }

    #[test]
    fn aggregation_kind_append_is_serde_index_stable() {
        // INV-12/L11: ModDeclaration is appended LAST, so every pre-existing
        // variant keeps its historical serde name (JSON) and its positional
        // (bincode) index order; the new name appears only at index 3.
        let cases = [
            (AggregationKind::Header, "\"Header\""),
            (AggregationKind::PackageObject, "\"PackageObject\""),
            (AggregationKind::None, "\"None\""),
            (AggregationKind::ModDeclaration, "\"ModDeclaration\""),
        ];
        for (kind, expected_json) in cases {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(
                json, expected_json,
                "INV-12/L11: {kind:?} must serialize under its historical name \
                 and ModDeclaration may only ever occupy the last index"
            );
        }
    }
}
