//! Extraction-proposal display DTOs and their payload caps, moved verbatim
//! out of `dto.rs` (ui-extraction-wizard W-1) so `dto.rs` drops under the
//! 1000-line R2 hygiene ceiling and the wizard DTOs (W-2b) have headroom.
//! Re-exported by `dto.rs` (`pub use extraction::*;`), so every public path
//! through `lib.rs` (`pub use dto::*;`) is unchanged.
//!
//! Plain display data only: the cluster crate's `OptimizationPlan` never
//! crosses into the ui crate; everything here is serde-plain display data
//! for the export payload. This rule was documented in the pre-split
//! `dto.rs` and is carried forward here unchanged.

use serde::{Deserialize, Serialize};

use super::{JUSTIFICATION_MAX_HEADERS, JUSTIFICATION_MAX_NAMES};

/// Extraction-proposal payload caps (I3/I8): top proposals per payload,
/// symbol names per proposal (I6: sorted by SymbolId before capping; the UI
/// infers truncation from `symbol_names.len() < total_symbols`), required
/// header basenames per proposal. Name/header caps reuse the justification
/// caps.
pub const EXTRACTION_PROPOSAL_MAX: usize = 12;
pub const EXTRACTION_PROPOSAL_MAX_NAMES: usize = JUSTIFICATION_MAX_NAMES;
pub const EXTRACTION_PROPOSAL_MAX_HEADERS: usize = JUSTIFICATION_MAX_HEADERS;

/// Cap for destination-file rows per proposal (mirrors the header cap).
pub const EXTRACTION_PROPOSAL_MAX_TARGET_FILES: usize = 8;

/// One suggested destination file and the symbols it should contain.
/// `path` is repo-relative, lossy-rendered like the plan's PathBuf (plain
/// display data only — the cluster plan type never crosses into ui).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedTargetFileDto {
    /// Repo-relative destination path suggested by the plan.
    pub path: String,
    /// Symbols destined for this file, sorted by symbol_id, capped at
    /// `EXTRACTION_PROPOSAL_MAX_NAMES`.
    pub symbol_names: Vec<String>,
    /// Uncapped member count for this file (truncation of `symbol_names`
    /// is `symbol_count - symbol_names.len()`).
    pub symbol_count: usize,
}

/// One Conservative extraction proposal shown in both surfaces (I5/I8):
/// plain display data only — the cluster crate's `OptimizationPlan` never
/// crosses into the ui crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionProposalDto {
    pub plan_id: String,
    pub target_file: String,
    pub target_file_id: String,
    pub problem: String,
    pub collateral_load_before: usize,
    pub collateral_load_after: usize,
    pub symbol_names: Vec<String>,
    pub total_symbols: usize,
    pub required_headers: Vec<String>,
    pub consumer_tu_count: usize,
    pub risk: String,
    /// Suggested destination files with per-file symbol manifests, grouped
    /// from the plan's proposed extractions; `serde(default)` keeps payloads
    /// written before this field decodable (empty = no file plan known).
    #[serde(default)]
    pub target_files: Vec<ProposedTargetFileDto>,
    /// True iff destination-file rows were capped at
    /// `EXTRACTION_PROPOSAL_MAX_TARGET_FILES` by the producer.
    #[serde(default)]
    pub target_files_truncated: bool,
}
