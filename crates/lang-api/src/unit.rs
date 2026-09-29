//! Analysis-unit descriptor (plan §3.3): the neutral replacement for the
//! build stage's C++-specific `TranslationUnit` shaping. The `UnitShaper`
//! trait that produces these lives in `ariadne-lang`.

use crate::identity::LanguageId;
use crate::modeling::UnitKind;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};

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
}
