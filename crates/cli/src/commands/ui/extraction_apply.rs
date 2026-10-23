//! Engine-backed extraction applier (extraction-auto-apply A-4): the host
//! implementation of `ariadne_ui::ExtractionPlanApplier` for ONE analysis
//! generation, installed by `swap_into_state` exactly like the W-2a reader
//! (`extraction_reader.rs` is the structural model; the reader is frozen, so
//! the fixture-test helpers below are deliberately duplicated - the priced
//! trade of the handoff's Frozen-locks section).
//!
//! Safety contract (shape C4/C5, kickoff items 4-5): single write path (A1 -
//! this module holds the milestone's exactly-one applier-side delegation into
//! the shared `apply_plan_with` transactional core (A-2);
//! `crates/validate/**` stays the only writer); dry-run-then-approve gate
//! (A2 - a single-use ASCII token, minted for one (plan id, epoch) pair and
//! consumed atomically at transaction start; missing, unknown, replayed or
//! foreign-plan tokens are refused `NotApproved`); epoch binding (A3 - the
//! registry lives on THIS generation's applier, so an epoch swap discards
//! every outstanding approval); one apply at a time (A4 - `apply_lock` is
//! try-locked, a held lock refuses `InProgress` immediately, applies are
//! never queued, the refusal burns no approval); honest outcome (A5 - three
//! tier rows, the committed/rolled-back verdict, the repo-relative created
//! files and the verbatim `ApplyOutcome::format_output` text); every DTO
//! string sanitized to ASCII (A9).

use crate::commands::transform::{apply_plan_with, ApplyTransactionReport};
use ariadne_cluster::plan::OptimizationPlan;
use ariadne_ui::{
    AnalysisGeneration, ExtractionApplyOutcomeDto, ExtractionApplyRefusal, ExtractionPlanApplier,
    ExtractionTierResultDto, UiState,
};
use ariadne_validate::{ApplyOutcome, TierResult, ValidationStatus, ValidationTier};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// Tier labels of record: the `ValidationTier` serde names, stability
/// locked by `status_serde_names_stable_and_skipped_is_additive` (validate).
const TIER_STRUCTURAL: &str = "Structural";
const TIER_GENERATED_HEADERS: &str = "GeneratedHeaders";
const TIER_AFFECTED_TUS: &str = "AffectedTUs";

/// Answers the apply route for one analysis generation. `plans` are the
/// generation's own proposal plans (the identical input the W-2a reader
/// gets); every mutation delegates to the shared core with the
/// generation's in-memory model - the model the approved dry-run was
/// computed from (A6, shape R10).
pub struct EngineBackedExtractionApplier {
    epoch: u64,
    repo_root: PathBuf,
    plans: Vec<OptimizationPlan>,
    generation: Arc<AnalysisGeneration>,
    /// C4 registry: token -> (plan_id, epoch). An epoch swap installs a
    /// NEW applier and therefore discards every outstanding token.
    token_registry: Mutex<HashMap<String, (String, u64)>>,
    /// C5 serialization: one apply per generation, try-lock only, never
    /// queued. Arc-wrapped so the test-only probe can hand out a handle.
    apply_lock: Arc<Mutex<()>>,
    /// C4 per-applier counter; the registry is the gate, never the
    /// deterministic ASCII token string itself.
    token_counter: AtomicU64,
}

impl EngineBackedExtractionApplier {
    pub fn new(
        epoch: u64,
        repo_root: PathBuf,
        plans: Vec<OptimizationPlan>,
        generation: Arc<AnalysisGeneration>,
    ) -> Self {
        Self {
            epoch,
            repo_root,
            plans,
            generation,
            token_registry: Mutex::new(HashMap::new()),
            apply_lock: Arc::new(Mutex::new(())),
            token_counter: AtomicU64::new(0),
        }
    }

    /// A4 test-only probe: hands out THIS generation's apply lock so a
    /// test can pre-hold it and watch `apply` refuse `InProgress` without
    /// burning the approval (the honest-retry proof). Test-only: no
    /// production caller, so an ungated method would be dead-code in
    /// every non-test build.
    #[cfg(test)]
    pub fn apply_lock_handle(&self) -> Arc<std::sync::Mutex<()>> {
        self.apply_lock.clone()
    }

    /// Test-only probe (required test 8): asserts a token is still bound
    /// and unconsumed WITHOUT running a transaction. Same test-only rationale
    /// as `apply_lock_handle`.
    #[cfg(test)]
    pub fn token_registry_contains(&self, token: &str, plan_id: &str) -> bool {
        self.token_registry_guard().get(token).map_or(false, |(bound_plan, bound_epoch)| {
            bound_plan == plan_id && *bound_epoch == self.epoch
        })
    }

    fn plan_for(&self, plan_id: &str) -> Option<&OptimizationPlan> {
        self.plans.iter().find(|plan| plan.id.0 == plan_id)
    }

    fn token_registry_guard(&self) -> MutexGuard<'_, HashMap<String, (String, u64)>> {
        self.token_registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// C5: `try_lock` - `None` means "held right now, refuse, never queue".
    /// Poison recovery mirrors the crate idiom: the unit payload stays sound.
    fn apply_lock_guard(&self) -> Option<MutexGuard<'_, ()>> {
        match self.apply_lock.try_lock() {
            Ok(guard) => Some(guard),
            Err(std::sync::TryLockError::WouldBlock) => None,
            Err(std::sync::TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
        }
    }

    /// C4 single-use consumption: lookup + remove under ONE registry-lock
    /// hold; a wrong binding is refused and left in place (one request's
    /// bad token must not burn somebody else's approval).
    fn consume_token(&self, plan_id: &str, token: &str) -> Result<(), ExtractionApplyRefusal> {
        let mut registry = self.token_registry_guard();
        match registry.get(token) {
            Some((bound_plan, bound_epoch))
                if bound_plan.as_str() == plan_id && *bound_epoch == self.epoch =>
            {
                registry.remove(token);
                Ok(())
            }
            _ => Err(ExtractionApplyRefusal::NotApproved),
        }
    }

    /// The module's single delegation site into the shared transactional
    /// core (required test 11 locks it at exactly one call; an `Err` here
    /// means validate's AR-04 machinery already restored the tree).
    fn run_transaction(&self, plan: &OptimizationPlan) -> Result<ApplyTransactionReport, String> {
        apply_plan_with(&self.repo_root, plan, &self.generation.reduced_model)
            .map_err(|failure| format!("{failure:?}"))
    }

    /// The A5 mapping: three tier rows in canonical order, the honest
    /// verdict, repo-relative created files, verbatim `format_output` (A9).
    fn outcome_dto(&self, plan_id: &str, report: &ApplyTransactionReport) -> ExtractionApplyOutcomeDto {
        let (committed, passed_tiers, failed_tier) = match &report.outcome {
            ApplyOutcome::Committed { tiers_passed } => (true, tiers_passed.as_slice(), None),
            ApplyOutcome::RolledBack { failed_tier, tiers_passed } => {
                (false, tiers_passed.as_slice(), Some(failed_tier))
            }
        };
        let tiers = [TIER_STRUCTURAL, TIER_GENERATED_HEADERS, TIER_AFFECTED_TUS]
            .into_iter()
            .map(|label| tier_row(label, passed_tiers, failed_tier))
            .collect();
        let new_files = report
            .new_files
            .iter()
            .map(|path| self.repo_relative(path))
            .collect();
        ExtractionApplyOutcomeDto {
            plan_id: ascii_only(plan_id),
            generation_epoch: self.epoch,
            committed,
            tiers,
            new_files,
            output_text: ascii_only(&report.outcome.format_output()),
        }
    }

    /// Repo-relative lossy rendering (the shape `staged_write_paths`
    /// produces in the reader). The core runs on the canonicalized root
    /// (`Repository::locate` canonicalizes; Windows renders the `\\?\`
    /// verbatim form), so the strip retries against the canonical root -
    /// the W-2a parity fact, its journal D5.
    fn repo_relative(&self, path: &Path) -> String {
        if let Ok(relative) = path.strip_prefix(&self.repo_root) {
            return relative.to_string_lossy().to_string();
        }
        if let Ok(canonical_root) = std::fs::canonicalize(&self.repo_root) {
            if let Ok(relative) = path.strip_prefix(canonical_root) {
                return relative.to_string_lossy().to_string();
            }
        }
        path.to_string_lossy().to_string()
    }
}

/// Install the applier for one generation, in the SAME sequence as the
/// W-2a reader install it follows, from the same plans and the same
/// generation Arc (journal D1: living here keeps `ui.rs`'s A-1 ceiling
/// headroom while preserving every element of the mandated install).
pub(crate) fn install_with_generation(
    state: &UiState,
    epoch: u64,
    plans: Vec<OptimizationPlan>,
) {
    let applier = EngineBackedExtractionApplier::new(
        epoch,
        state.repo_root.clone(),
        plans,
        state
            .analysis()
            .expect("install_analysis publishes the generation it just installed"),
    );
    state.install_extraction_applier(Arc::new(applier));
}

/// One honest tier row: pass/skip/fail as observed; a tier that never
/// ran (the transaction short-circuits at the first `Fail`) is "skipped"
/// with the reason, never a fabricated pass (A5).
fn tier_row(
    label: &str,
    passed_tiers: &[TierResult],
    failed_tier: Option<&TierResult>,
) -> ExtractionTierResultDto {
    let observed = passed_tiers
        .iter()
        .chain(failed_tier)
        .find(|row| tier_label_of(&row.tier) == label);
    let (status, detail) = match observed.map(|row| &row.status) {
        Some(ValidationStatus::Pass) => ("pass".to_string(), String::new()),
        Some(ValidationStatus::Fail(reason)) => ("fail".to_string(), ascii_only(reason)),
        Some(ValidationStatus::Skipped(reason)) => ("skipped".to_string(), ascii_only(reason)),
        None => ("skipped".to_string(), "not run: an earlier tier failed the transaction".to_string()),
    };
    ExtractionTierResultDto {
        tier: label.to_string(),
        status,
        detail,
    }
}

fn tier_label_of(tier: &ValidationTier) -> &'static str {
    match tier {
        ValidationTier::Structural => TIER_STRUCTURAL,
        ValidationTier::GeneratedHeaders => TIER_GENERATED_HEADERS,
        ValidationTier::AffectedTUs => TIER_AFFECTED_TUS,
    }
}

/// A9: every DTO string is sanitized to ASCII; non-ASCII becomes `?`.
fn ascii_only(text: &str) -> String {
    text.chars()
        .map(|ch| if ch.is_ascii() { ch } else { '?' })
        .collect()
}

impl ExtractionPlanApplier for EngineBackedExtractionApplier {
    fn epoch(&self) -> u64 {
        self.epoch
    }

    fn mint_apply_token(&self, plan_id: &str) -> Option<String> {
        self.plan_for(plan_id)?;
        // C4 value shape: deterministic ASCII, `+ 1` so a fresh applier's
        // first token ends in `-1` (required test 9: determinism is a
        // feature, the single-use registry is the gate).
        let counter = self.token_counter.fetch_add(1, Ordering::SeqCst) + 1;
        let token = format!("apply-{}-{}-{}", self.epoch, plan_id, counter);
        self.token_registry_guard()
            .insert(token.clone(), (plan_id.to_string(), self.epoch));
        Some(token)
    }

    fn apply(
        &self,
        plan_id: &str,
        token: &str,
    ) -> Result<ExtractionApplyOutcomeDto, ExtractionApplyRefusal> {
        // (1) the plan must resolve within THIS generation (404 before the
        // lock and before any token is touched).
        let plan = self
            .plan_for(plan_id)
            .ok_or(ExtractionApplyRefusal::UnknownPlan)?;
        // (2) C5: try-lock only; the registry is NOT touched, so the
        // approval survives the refusal (A4, honest retry).
        let _serial = self
            .apply_lock_guard()
            .ok_or(ExtractionApplyRefusal::InProgress)?;
        // (3) C4: atomic consumption at transaction start.
        self.consume_token(plan_id, token)?;
        // (4-5) the shared transactional core (A1); an `Err` here means
        // validate's AR-04 machinery already restored the tree.
        let report = self.run_transaction(plan).map_err(|failure| {
            ExtractionApplyRefusal::Failed(ascii_only(&failure))
        })?;
        // (6) the honest outcome DTO (A5), every string ASCII (A9).
        Ok(self.outcome_dto(plan_id, &report))
    }
}

#[cfg(test)]
mod tests;
