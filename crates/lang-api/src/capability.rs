//! Capability gating (plan §3.2/§3.3): which pipeline stages a language
//! supports, and whether its toolchain is present.

use serde::{Deserialize, Serialize};

/// Stage gating: the pipeline skips (with an explanatory message) any stage
/// whose flag is `false` instead of silently producing empty output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageCapabilities {
    pub discovery: bool,
    pub dependency_graph: bool,
    pub semantic_index: bool,
    pub load_metric: bool,
    pub planning: bool,
    pub extraction: bool,
    pub rewriting: bool,
    pub validation: bool,
    /// `false` => the language works fully offline (plan assumption A3).
    pub requires_toolchain: bool,
}

/// Result of probing a language toolchain, so tier-3 validation can be
/// `Skipped`, not a `Fail`, when the compiler is legitimately absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolchainStatus {
    Available(String),
    Missing,
    Unknown,
}
