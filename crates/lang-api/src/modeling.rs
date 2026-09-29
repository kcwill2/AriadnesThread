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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
