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

// ---- Wizard route DTOs (ui-extraction-wizard W-2b, shape C2) ----
//
// Additive, serde-plain display data only (the same rule as the module
// doc): every field carries `serde(default)` so partial payloads decode
// honestly, and no cluster/transform/validate type crosses the boundary.
// The caps are shared by the cli provider (W-2a, which enforces them) and
// the wizard renderer (W-3a/W-3b, which displays the truncation flags).

/// Cap for destination-file rows per manifest (shape C6).
pub const EXTRACTION_MANIFEST_MAX_FILES: usize = 64;

/// Cap for symbol names per file row in a manifest; `symbol_count` carries
/// the uncapped truth (shape C6, same inference idiom as
/// `ExtractionProposalDto`).
pub const EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE: usize = 200;

/// Cap for consumer rewrite rows per dry-run (shape C6).
pub const EXTRACTION_DRY_RUN_MAX_REWRITES: usize = 200;

/// One destination file with its capped symbol list and the includes the
/// new file needs (the S1/S2 step payload, shape C2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionUnitManifestDto {
    /// Repo-relative, lossy-rendered.
    #[serde(default)]
    pub path: String,
    /// Capped at `EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE`, symbol_id
    /// order.
    #[serde(default)]
    pub symbol_names: Vec<String>,
    /// Uncapped members.
    #[serde(default)]
    pub symbol_count: usize,
    /// Engine required_deps, lossy-rendered (W3).
    #[serde(default)]
    pub required_includes: Vec<String>,
    /// True iff the engine could not resolve deps.
    #[serde(default)]
    pub includes_unavailable: bool,
}

/// Per-plan manifest answer of the read-only reader (the S1/S2/S3 step
/// payload, shape C2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionManifestDto {
    #[serde(default)]
    pub plan_id: String,
    #[serde(default)]
    pub generation_epoch: u64,
    #[serde(default)]
    pub files: Vec<ExtractionUnitManifestDto>,
    #[serde(default)]
    pub files_truncated: bool,
    /// Engine unit-graph cycle, when any.
    #[serde(default)]
    pub cycle_error: Option<String>,
}

/// Dry-run answer: which new files would be written and which consumer
/// includes would be rewritten (the S4 step payload, shape C2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionDryRunDto {
    #[serde(default)]
    pub plan_id: String,
    #[serde(default)]
    pub generation_epoch: u64,
    /// Paths the apply would write.
    #[serde(default)]
    pub new_files: Vec<String>,
    #[serde(default)]
    pub rewrites: Vec<ExtractionRewriteDto>,
    #[serde(default)]
    pub rewrites_truncated: bool,
    /// Symbols the plugin will strip; 0 for C++.
    #[serde(default)]
    pub move_removals: usize,
    /// True when no source symbol is removed.
    #[serde(default)]
    pub copy_only: bool,
    #[serde(default)]
    pub includes_unavailable: bool,
    /// Single-use approval token for the apply route (extraction-auto-apply
    /// shape C4): minted by the dry-run route; empty without an applier.
    /// Additive and serde-defaulted, so pre-apply payloads and struct
    /// literals decode unchanged.
    #[serde(default)]
    pub apply_token: String,
}

/// One consumer include rewrite (old -> new); the field names mirror
/// `ConsumerRewrite` in the transform crate so the provider mapping is
/// mechanical and reviewable (shape C2 note).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionRewriteDto {
    #[serde(default)]
    pub consumer_file: String,
    #[serde(default)]
    pub old_includes: Vec<String>,
    #[serde(default)]
    pub new_includes: Vec<String>,
    #[serde(default)]
    pub side_effect_preservations: Vec<String>,
}

// ---- Apply route DTOs (extraction-auto-apply A-3, shape C3) ----
//
// Additive, serde-plain display data only (the same plain-display-data
// rule as the module doc): every field carries `serde(default)`, and no
// validate/transform/cluster type crosses into the ui crate. The cli
// applier (A-4) owns the `ValidationStatus` -> `status` + `detail` mapping
// and the ASCII sanitization; this module owns the shape.

/// Request body of the apply route (shape C3): the single-use approval
/// token minted by the GET dry-run route. The handler parses it
/// tolerantly - an absent, unparseable or empty-token body is the 428
/// `dry_run_required` refusal, never an extractor 4xx.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionApplyRequestDto {
    #[serde(default)]
    pub apply_token: String,
}

/// One validation tier's result row (shape C3): `tier` mirrors
/// `ValidationTier`'s serde names ("Structural", "GeneratedHeaders",
/// "AffectedTUs" - stability locked in `crates/validate/src/lib.rs`),
/// `status` is "pass", "fail" or "skipped", and `detail` carries the
/// fail/skip reason (empty on pass), ASCII-sanitized by the producer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionTierResultDto {
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub detail: String,
}

/// Outcome answer of the apply route (shape C3, invariant A5): the three
/// tier results, the committed-vs-rolled-back verdict (both complete
/// answers are 200), the files the transaction created (repo-relative,
/// lossy-rendered) and the verbatim `ApplyOutcome::format_output` text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionApplyOutcomeDto {
    #[serde(default)]
    pub plan_id: String,
    /// The epoch the approval was minted on.
    #[serde(default)]
    pub generation_epoch: u64,
    #[serde(default)]
    pub committed: bool,
    #[serde(default)]
    pub tiers: Vec<ExtractionTierResultDto>,
    /// Files the transaction created, lossy.
    #[serde(default)]
    pub new_files: Vec<String>,
    /// `ApplyOutcome::format_output()`, verbatim.
    #[serde(default)]
    pub output_text: String,
}
