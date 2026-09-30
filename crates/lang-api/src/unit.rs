//! Analysis-unit descriptor (plan §3.3): the neutral replacement for the
//! build stage's C++-specific `TranslationUnit` shaping. The `UnitShaper`
//! trait that produces these lives in `ariadne-lang`.

use crate::identity::LanguageId;
use crate::modeling::UnitKind;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// One file that a plugin decided becomes an analysis unit, with its
/// language-specific context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitDescriptor {
    pub file_id: FileId,
    pub language: LanguageId,
    pub unit_kind: UnitKind,
    /// Human-readable build context (e.g. `"Debug|x64"`, sbt module name).
    pub context_name: String,
    /// Aggregation artifact this unit is loaded through, if any
    /// (C++: the extracted header target; Scala: package object).
    pub aggregation_target: Option<FileId>,
    /// Language-specific build invocation ingredients
    /// (flags for C++, sbt settings for Scala, ...).
    pub build_spec: Vec<String>,
    /// Preprocessor symbols the unit's compilation context defines
    /// (plan M18.4 additive: `build` synthesizes the `CompilationContext`
    /// from these shaper-provided ingredients, never from literals).
    #[serde(default)]
    pub defines: Vec<String>,
    /// Search directories the unit's compilation context contributes
    /// (plan M18.4 additive; C++: the include dirs, Scala: source roots).
    #[serde(default)]
    pub include_dirs: Vec<PathBuf>,
}

#[cfg(test)]
mod unit_descriptor_tests {
    use super::*;

    fn descriptor() -> UnitDescriptor {
        UnitDescriptor {
            file_id: FileId::from_relative_path("src/main.cpp"),
            language: LanguageId::Cpp,
            unit_kind: UnitKind::TranslationUnit,
            context_name: "Debug|x64".to_string(),
            aggregation_target: None,
            build_spec: vec!["/std:c++20".to_string()],
            defines: vec!["DEBUG".to_string()],
            include_dirs: vec![PathBuf::from("include")],
        }
    }

    #[test]
    fn round_trips_through_json_and_bincode() {
        let unit = descriptor();
        let json = serde_json::to_string(&unit).unwrap();
        let decoded: UnitDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(unit, decoded);
        let bin = bincode::serialize(&unit).unwrap();
        let decoded_bin: UnitDescriptor = bincode::deserialize(&bin).unwrap();
        assert_eq!(unit, decoded_bin);
    }

    #[test]
    fn pre_m18_4_artifacts_deserialize_with_empty_defaults() {
        // Additive-field contract (plan M18.4): artifacts written before the
        // `defines`/`include_dirs` keys existed keep deserializing, reading
        // as empty ingredients.
        let json = serde_json::to_string(&descriptor()).unwrap();
        let without_defines = json.replace(",\"defines\":[\"DEBUG\"]", "");
        let without_both = without_defines.replace(",\"include_dirs\":[\"include\"]", "");
        let decoded: UnitDescriptor = serde_json::from_str(&without_both).unwrap();
        assert!(decoded.defines.is_empty());
        assert!(decoded.include_dirs.is_empty());
        assert_eq!(decoded.context_name, "Debug|x64");
    }
}
