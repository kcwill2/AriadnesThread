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
            plan_id: plan_id.to_string(),
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
mod tests {
    //! A-4 required tests 1-11 (fixture: the `basic` C++ fixture,
    //! PLAN-001). Providers boot through the real `swap_into_state` wiring
    //! (the `boot_swapped_fixture` idiom duplicated from the frozen
    //! `extraction_reader.rs` per the Frozen-locks note). Every mutation
    //! runs inside a `tempfile::TempDir` fixture copy; the tracked fixture
    //! is never written.

    use super::super::{build_ui_payload, swap_into_state, ProgressFn};
    use super::{EngineBackedExtractionApplier, TIER_AFFECTED_TUS, TIER_GENERATED_HEADERS, TIER_STRUCTURAL};
    use crate::commands::transform::ApplyTransactionReport;
    use ariadne_cluster::plan::{OptimizationPlan, PlanId};
    use ariadne_graph::{IncludeGraph, LoadMemo, ReducedSemanticModel};
    use ariadne_lang::SymbolKind;
    use ariadne_transform::{
        ConsumerRewrite, ExtractedSymbol, ExtractionResult, GeneratedUnit, MigrationPreview,
    };
    use ariadne_ui::{
        AnalysisGeneration, ExtractionApplyOutcomeDto, ExtractionApplyRefusal,
        ExtractionManifestReader, ExtractionPlanApplier, ExtractionProposalDto, UiState,
    };
    use ariadne_validate::{
        ApplyOutcome, TierResult, TierValidator, TransactionApplier, ValidationStatus, ValidationTier,
    };
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    /// Boot a fresh fixture copy and swap it into a `new_loading` state
    /// exactly the way `run_ui` boots (install epoch 1), returning the
    /// guard, the state, and both proposal/plan sets cloned pre-swap.
    fn boot_swapped_fixture(fixture: &str) -> (tempfile::TempDir, Arc<UiState>, Vec<ExtractionProposalDto>, Vec<OptimizationPlan>) {
        let (temp_dir, repo_path) = fixture_copy(fixture);
        let quiet: ProgressFn = Arc::new(|_report| {});
        let mut built =
            build_ui_payload(&repo_path, quiet).expect("cold boot of the fixture must succeed");
        // Mirrors the run_ui boot closure: this state was `new_loading`
        // (epoch 0), so boot's swap installs generation epoch 1.
        built.install_epoch = 1;
        let proposals = built.payload.extraction_proposals.clone();
        let plans = built.extraction_plans.clone();
        let state = Arc::new(UiState::new_loading(repo_path.clone()));
        swap_into_state(&state, built);
        (temp_dir, state, proposals, plans)
    }

    fn installed_reader(state: &UiState) -> Arc<dyn ExtractionManifestReader> {
        state
            .extraction_reader()
            .expect("swap_into_state must install the generation's reader")
    }

    fn installed_applier(state: &UiState) -> Arc<dyn ExtractionPlanApplier> {
        state
            .extraction_applier()
            .expect("swap_into_state must install the generation's applier")
    }

    /// A concrete applier over the SAME booted generation, for the
    /// inherent probe surface.
    fn concrete_applier_for_booted_generation(
        state: &UiState,
        plans: Vec<OptimizationPlan>,
    ) -> Arc<EngineBackedExtractionApplier> {
        Arc::new(EngineBackedExtractionApplier::new(
            1,
            state.repo_root.clone(),
            plans,
            state.analysis().expect("boot installed the generation"),
        ))
    }

    /// A standalone applier over an empty generation: for the pure
    /// mapping and token-value tests, which need no live analysis.
    fn standalone_applier(epoch: u64, plans: Vec<OptimizationPlan>) -> EngineBackedExtractionApplier {
        let empty_generation = Arc::new(AnalysisGeneration {
            include_graph: IncludeGraph::new(),
            reduced_model: ReducedSemanticModel::default(),
            memo: Arc::new(LoadMemo::new()),
        });
        EngineBackedExtractionApplier::new(epoch, PathBuf::from("."), plans, empty_generation)
    }

    fn fixture_copy(fixture: &str) -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("temp repo");
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(fixture);
        let destination = temp_dir.path().join(fixture);
        copy_dir_all(&source, &destination).expect("copy fixture");
        for stale in ["artifacts", ".ariadne"] {
            let _ = std::fs::remove_dir_all(destination.join(stale));
        }
        (temp_dir, destination)
    }

    fn copy_dir_all(source: &Path, destination: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(destination)?;
        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let target = destination.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                copy_dir_all(&entry.path(), &target)?;
            } else {
                std::fs::copy(entry.path(), &target)?;
            }
        }
        Ok(())
    }

    /// FNV-1a/64 over the sorted tree walk (path bytes, presence marker,
    /// content bytes) - the whole-tree hash oracle of required test 3's
    /// acceptance, duplicated from the frozen reader's idiom.
    fn tree_hash(root: &Path) -> u64 {
        let files = walked_files(root);
        let mut hash: u64 = 0xcbf29ce484222325;
        let mix = |hash: &mut u64, bytes: &[u8]| {
            for byte in bytes {
                *hash ^= *byte as u64;
                *hash = hash.wrapping_mul(0x100000001b3);
            }
        };
        for (relative, content) in &files {
            mix(&mut hash, relative.to_string_lossy().as_bytes());
            match content {
                Some(bytes) => {
                    mix(&mut hash, &[1]);
                    mix(&mut hash, bytes);
                }
                None => mix(&mut hash, &[0]),
            }
        }
        hash
    }

    /// The whole-tree walk: repo-relative path -> content (None if the
    /// content was unreadable at walk time).
    fn walked_files(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
        let mut files: BTreeMap<PathBuf, Option<Vec<u8>>> = BTreeMap::new();
        collect_tree(root, root, &mut files);
        files
    }

    fn collect_tree(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in std::fs::read_dir(directory).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if entry.file_type().expect("file type").is_dir() {
                collect_tree(root, &path, files);
            } else {
                let content = std::fs::read(&path).ok();
                files.insert(path.strip_prefix(root).expect("relative").to_path_buf(), content);
            }
        }
    }

    /// The apply's own outcome record lands under `artifacts/transaction`;
    /// neither it nor the artifact tree is a created source file (handoff:
    /// the written set EXCLUDES the repo's artifact tree).
    fn is_artifact_tree(relative: &Path) -> bool {
        relative.components().any(|component| {
            matches!(component, std::path::Component::Normal(name)
                if name == std::ffi::OsStr::new(".ariadne")
                    || name == std::ffi::OsStr::new("artifacts"))
        })
    }

    /// Files the transaction WROTE: present after, absent before.
    fn written_relative(
        before: &BTreeMap<PathBuf, Option<Vec<u8>>>,
        after: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    ) -> Vec<String> {
        after
            .keys()
            .filter(|relative| {
                !before.contains_key(relative.as_path()) && !is_artifact_tree(relative)
            })
            .map(|relative| relative.to_string_lossy().to_string())
            .collect()
    }

    /// Files the transaction MODIFIED: present in both walks with
    /// different content.
    fn modified_relative(
        before: &BTreeMap<PathBuf, Option<Vec<u8>>>,
        after: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    ) -> Vec<PathBuf> {
        before
            .iter()
            .filter_map(|(relative, before_content)| {
                if is_artifact_tree(relative) {
                    return None;
                }
                let after_content = after.get(relative.as_path())?;
                (before_content != after_content).then(|| relative.clone())
            })
            .collect()
    }

    /// Host-canonical rendering of a repo-relative path (the discovery
    /// stack names `consumer_file` in this form on Windows, `\\?\`
    /// included - the fact the W-2a parity test recorded, its D5).
    fn canonicalized_lossy(repo_root: &Path, relative: &Path) -> String {
        std::fs::canonicalize(repo_root.join(relative))
            .expect("a modified file exists on disk")
            .to_string_lossy()
            .to_string()
    }

    fn first_proposal_id(proposals: &[ExtractionProposalDto]) -> String {
        proposals
            .first()
            .expect("the basic fixture boots with at least one proposal")
            .plan_id
            .clone()
    }

    fn assert_all_strings_ascii(dto: &ExtractionApplyOutcomeDto, context: &str) {
        let mut strings: Vec<&String> =
            vec![&dto.plan_id, &dto.output_text].into_iter().chain(dto.new_files.iter()).collect();
        for row in &dto.tiers {
            strings.extend([&row.tier, &row.status, &row.detail]);
        }
        for value in strings {
            assert!(value.is_ascii(), "{context}: DTO string must be ASCII, got {value:?}");
        }
    }

    // Required test 1 (A6 - THE proof): the files a real apply WROTE
    // equal the approved dry-run's `new_files`, field by field, through
    // the real boot + swap + applier path on an isolated fixture copy.
    #[test]
    fn test_a_valid_token_applies_the_plan_end_to_end() {
        let (_temp_dir, state, proposals, _plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let reader = installed_reader(&state);
        let applier = installed_applier(&state);
        let dry_run = reader
            .dry_run(&plan_id)
            .expect("the boot proposal id must resolve within its generation");
        let token = applier
            .mint_apply_token(&plan_id)
            .expect("the boot proposal id must mint an approval");
        let repo_root = state.repo_root.clone();
        let before = walked_files(&repo_root);

        let attempt = applier.apply(&plan_id, &token);
        assert!(attempt.is_ok(), "expected an approved apply, got {attempt:?}");
        let outcome = attempt.expect("the approved apply answer");
        let after = walked_files(&repo_root);

        let mut written = written_relative(&before, &after);
        written.sort();
        let mut approved_new_files = dry_run.new_files.clone();
        approved_new_files.sort();
        assert!(
            !written.is_empty(),
            "the fixture apply must actually write files, wrote nothing"
        );
        assert_eq!(
            written, approved_new_files,
            "A6: the files the apply WROTE must equal the approved dry-run's new_files"
        );

        let mut modified: Vec<String> = modified_relative(&before, &after)
            .iter()
            .map(|relative| canonicalized_lossy(&repo_root, relative))
            .collect();
        modified.sort();
        let mut approved_consumers: Vec<String> = dry_run
            .rewrites
            .iter()
            .map(|rewrite| rewrite.consumer_file.clone())
            .collect();
        approved_consumers.sort();
        // The handoff's literal "MODIFIED == consumer_file set" is
        // UNSATISFIABLE here (journal D6): the shipped apply is a
        // CHARACTERIZED identity rewrite of the consumers - it searches for
        // the `#include "include/thing.h"` spelling while the sources hold
        // the bare `#include "thing.h"` - locked, with that reason verbatim,
        // by m20's apply_thing_plan_locks_transaction_outcome_and_disk_effects
        // ("latent vacuous-rewrite bug, characterization only"). The
        // achievable parity: WRITTEN == new_files (above), the dry-run really
        // carries consumer rows, and MODIFIED may only ever be a SUBSET of
        // the approved consumers - equality the day the bug is fixed; the
        // transaction's non-emptiness rides on the written set.
        assert!(
            !approved_consumers.is_empty(),
            "the approved dry-run must actually carry consumer rewrite rows, \
             carried none"
        );
        assert!(
            modified.iter().all(|file| approved_consumers.contains(file)),
            "the apply must not modify any file outside the approved dry-run's \
             consumer_file set: modified={modified:?}"
        );

        let mut dto_new_files = outcome.new_files.clone();
        dto_new_files.sort();
        assert_eq!(
            dto_new_files, written,
            "the outcome DTO's new_files must equal the written set"
        );
        assert!(
            outcome.committed,
            "the fixture transaction must commit, output:\n{}",
            outcome.output_text
        );
        assert!(
            outcome.output_text.ends_with("\nCommitted."),
            "output_text must end with the committed line of format_output(), got:\n{}",
            outcome.output_text
        );
        assert_eq!(outcome.plan_id, plan_id);
        assert_eq!(
            outcome.generation_epoch, 1,
            "the DTO carries the epoch the approval was minted on"
        );
    }

    // Required test 2 (A5, DTO half): a fabricated RolledBack outcome
    // through the applier's own mapping.
    #[test]
    fn test_a_rolled_back_outcome_maps_to_the_honest_dto() {
        let applier = standalone_applier(3, Vec::new());
        let failed_tier = TierResult {
            tier: ValidationTier::GeneratedHeaders,
            status: ValidationStatus::Fail(
                "Missing include: ghost_header_for_test.h".to_string(),
            ),
        };
        let tiers_passed = vec![TierResult {
            tier: ValidationTier::Structural,
            status: ValidationStatus::Pass,
        }];
        let outcome = ApplyOutcome::RolledBack {
            failed_tier,
            tiers_passed,
        };
        let expected_output = outcome.format_output();
        let report = ApplyTransactionReport {
            outcome,
            new_files: Vec::new(),
        };

        let dto = applier.outcome_dto("PLAN-001", &report);
        assert!(!dto.committed, "a rolled-back outcome maps to committed == false");
        assert_eq!(dto.plan_id, "PLAN-001");
        assert_eq!(dto.generation_epoch, 3);
        assert_eq!(dto.tiers.len(), 3, "the honest DTO carries exactly three tier rows");
        assert_eq!(dto.tiers[0].tier, TIER_STRUCTURAL);
        assert_eq!(dto.tiers[0].status, "pass");
        assert!(dto.tiers[0].detail.is_empty(), "a pass row carries no detail");
        assert_eq!(dto.tiers[1].tier, TIER_GENERATED_HEADERS);
        assert_eq!(dto.tiers[1].status, "fail");
        assert!(
            dto.tiers[1].detail.contains("ghost_header_for_test.h"),
            "the failing row's detail carries the failure reason, got {:?}",
            dto.tiers[1].detail
        );
        assert_eq!(dto.tiers[2].tier, TIER_AFFECTED_TUS);
        assert_eq!(
            dto.tiers[2].status, "skipped",
            "the tier after the short-circuit never ran: honest skipped, never a fabricated pass"
        );
        assert_eq!(
            dto.output_text, expected_output,
            "output_text equals ApplyOutcome::format_output() byte for byte"
        );
    }

    // Required test 3 (A5, restoration half): a forced tier-2 failure run
    // through the REAL machinery restores the tree byte-identically.
    #[test]
    fn test_a_failed_tier_restores_the_tree_byte_identically() {
        let (_temp_dir, repo_root) = fixture_copy("basic");
        let registry = crate::registry::default_registry();
        let plan_target = PathBuf::from("thing.h");
        let backend = |capability: fn(&ariadne_lang::LanguageCapabilities) -> bool, name: &str| {
            crate::registry::plan_backend(&registry, &plan_target, capability, name)
        };
        let extractor = backend(|caps| caps.extraction, "extraction")
            .expect("the basic fixture plan routes to an extraction backend")
            .extractor();
        let rewriter = backend(|caps| caps.rewriting, "rewriting")
            .expect("the basic fixture plan routes to a rewriting backend")
            .rewriter();

        // The fabricated extraction: one unit whose required_deps name a
        // header that exists NOWHERE on disk, so tier 2 must fail.
        let forced_unit = GeneratedUnit {
            path: PathBuf::from("include/forced_rollback_proof.h"),
            symbols: vec![ExtractedSymbol {
                name: "ForcedRollbackProof".to_string(),
                qualified_id: "ForcedRollbackProof".to_string(),
                kind: SymbolKind::Struct,
                body: String::new(),
            }],
            required_deps: vec![PathBuf::from("ghost_header_for_the_rollback_proof.h")],
            primary_source_text: None,
        };
        let extraction = ExtractionResult {
            plan_id: "PLAN-ROLLED-BACK".to_string(),
            headers: vec![forced_unit],
            errors: Vec::new(),
        };
        let migration = MigrationPreview {
            plan_id: "PLAN-ROLLED-BACK".to_string(),
            rewrites: vec![ConsumerRewrite {
                consumer_file: PathBuf::from("tests/consumer.cpp"),
                old_includes: vec![PathBuf::from("thing.h")],
                new_includes: vec![PathBuf::from("include/forced_rollback_proof.h")],
                side_effect_preservations: Vec::new(),
            }],
        };

        let before = tree_hash(&repo_root);
        let (snapshot, new_paths) =
            TransactionApplier::apply_with(&repo_root, &extraction, &migration, rewriter, extractor)
                .expect("the fabricated transaction must write through the real machinery");
        assert!(
            !new_paths.is_empty(),
            "the fabricated unit must have been written"
        );
        assert_ne!(
            tree_hash(&repo_root),
            before,
            "the apply must actually have changed the tree"
        );

        let tier_one = TierValidator::validate_structural(&extraction);
        assert!(
            matches!(tier_one.status, ValidationStatus::Pass),
            "tier 1 must pass before the forced tier-2 failure, got {:?}",
            tier_one.status
        );
        let tier_two =
            TierValidator::validate_generated_headers(&extraction, &repo_root.join("generated"));
        assert!(
            tier_two.status.is_fail(),
            "the missing required dep must FAIL tier 2, got {:?}",
            tier_two.status
        );

        TransactionApplier::rollback(&snapshot, &new_paths).expect("rollback must succeed");
        TransactionApplier::verify_restored(&snapshot)
            .expect("verify_restored must pass after the rollback");
        assert_eq!(
            tree_hash(&repo_root),
            before,
            "A5: a failed tier-2 leaves the whole tree byte-identical (whole-tree hash, acceptance 2)"
        );
    }

    // Required test 4 (A2): a consumed token cannot apply twice.
    #[test]
    fn test_a_consumed_token_cannot_apply_twice() {
        let (_temp_dir, state, proposals, _plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let applier = installed_applier(&state);
        let token = applier
            .mint_apply_token(&plan_id)
            .expect("the boot proposal id must mint an approval");

        let first = applier.apply(&plan_id, &token);
        assert!(
            first.is_ok(),
            "the first apply with a fresh token must succeed, got {first:?}"
        );
        let repo_root = state.repo_root.clone();
        let tree_after_first = walked_files(&repo_root);

        let second = applier.apply(&plan_id, &token);
        assert!(
            matches!(second, Err(ExtractionApplyRefusal::NotApproved)),
            "the second apply with the same token must be refused, got {second:?}"
        );
        assert_eq!(
            walked_files(&repo_root),
            tree_after_first,
            "the refused second apply must not mutate the tree a second time"
        );
    }

    // Required test 5 (A4): a held lock refuses WITHOUT consuming the
    // token - the honest-retry proof.
    #[test]
    fn test_a_held_lock_refuses_without_consuming_the_token() {
        let (_temp_dir, state, proposals, plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let applier = concrete_applier_for_booted_generation(&state, plans);
        let token = applier
            .mint_apply_token(&plan_id)
            .expect("the boot proposal id must mint an approval");

        let handle = applier.apply_lock_handle();
        let held = handle.lock().expect("the test thread takes the apply lock");
        let repo_root = state.repo_root.clone();
        let tree_before = walked_files(&repo_root);
        let refused = applier.apply(&plan_id, &token);
        assert!(
            matches!(refused, Err(ExtractionApplyRefusal::InProgress)),
            "expected InProgress while the lock is held, got {refused:?}"
        );
        assert_eq!(
            walked_files(&repo_root),
            tree_before,
            "the refused apply must not touch the tree"
        );
        drop(held);

        let retry = applier.apply(&plan_id, &token);
        assert!(
            retry.is_ok(),
            "the same token must apply after the lock is released, got {retry:?}"
        );
        assert!(
            retry.expect("the honest-retry answer").committed,
            "the honest retry must commit"
        );
    }

    // Required test 6 (A3): a token minted on another epoch is refused,
    // after the epoch-1 control apply rules out a broken applier.
    #[test]
    fn test_a_token_minted_for_another_epoch_is_refused() {
        let (_temp_dir, state, proposals, plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let applier = installed_applier(&state);
        let control_token = applier
            .mint_apply_token(&plan_id)
            .expect("the live epoch mints");
        let control = applier.apply(&plan_id, &control_token);
        assert!(control.is_ok(), "expected an approved apply, got {control:?}");

        let other_epoch = EngineBackedExtractionApplier::new(
            2,
            state.repo_root.clone(),
            plans,
            state.analysis().expect("boot installed the generation"),
        );
        let foreign_token = other_epoch
            .mint_apply_token(&plan_id)
            .expect("the other epoch resolves the same plan");
        let attempt = applier.apply(&plan_id, &foreign_token);
        assert!(
            matches!(attempt, Err(ExtractionApplyRefusal::NotApproved)),
            "expected NotApproved for a token from another epoch, got {attempt:?}"
        );
    }

    // Required test 7 (A2): a token bound to plan A cannot apply plan B.
    #[test]
    fn test_a_foreign_plan_token_is_refused() {
        let (_temp_dir, state, proposals, plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let mut plan_b = plans
            .first()
            .expect("the basic fixture boots with a plan")
            .clone();
        plan_b.id = PlanId("PLAN-B".to_string());
        let mut two_plans = plans.clone();
        two_plans.push(plan_b);
        let applier = concrete_applier_for_booted_generation(&state, two_plans);

        let token_a = applier
            .mint_apply_token(&plan_id)
            .expect("plan A mints its own approval");
        let attempt = applier.apply("PLAN-B", &token_a);
        assert!(
            matches!(attempt, Err(ExtractionApplyRefusal::NotApproved)),
            "expected NotApproved for a foreign-plan token, got {attempt:?}"
        );
        assert!(
            applier.token_registry_contains(&token_a, &plan_id),
            "a foreign-plan refusal must not burn plan A's approval"
        );
    }

    // Required test 8 (A1's boundary): an unknown plan is refused with
    // the registry untouched.
    #[test]
    fn test_an_unknown_plan_is_refused() {
        let (_temp_dir, state, proposals, plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let applier = concrete_applier_for_booted_generation(&state, plans);
        assert!(
            applier.mint_apply_token("PLAN-DOES-NOT-EXIST").is_none(),
            "an unknown plan mints no token (the dry-run stays honest)"
        );
        let valid = applier
            .mint_apply_token(&plan_id)
            .expect("the known plan mints");
        let attempt = applier.apply("PLAN-DOES-NOT-EXIST", &valid);
        assert!(
            matches!(attempt, Err(ExtractionApplyRefusal::UnknownPlan)),
            "expected UnknownPlan for a plan outside the generation, got {attempt:?}"
        );
        assert!(
            applier.token_registry_contains(&valid, &plan_id),
            "the unknown-plan refusal must leave the token registry untouched"
        );
    }

    // Required test 9 (C4): the token value is deterministic, epoch- and
    // plan-bound, and the sequence advances per mint.
    #[test]
    fn test_the_token_value_is_deterministic_epoch_and_plan_bound() {
        let (_temp_dir, state, proposals, plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let first_applier = standalone_applier(1, plans.clone());
        let first = first_applier
            .mint_apply_token(&plan_id)
            .expect("a resolvable plan mints");
        let second = first_applier
            .mint_apply_token(&plan_id)
            .expect("a resolvable plan mints again");
        assert_eq!(first, format!("apply-1-{plan_id}-1"), "C4 token value shape");
        assert_eq!(
            second,
            format!("apply-1-{plan_id}-2"),
            "the per-applier counter advances per mint"
        );

        let twin = standalone_applier(1, plans.clone());
        assert_eq!(
            twin.mint_apply_token(&plan_id).as_deref(),
            Some(first.as_str()),
            "two appliers for the same epoch mint the same first token: determinism is a feature, the gate is the single-use registry"
        );
        assert!(
            standalone_applier(1, Vec::new())
                .mint_apply_token(&plan_id)
                .is_none(),
            "a generation holding no plans resolves no plan and mints nothing"
        );
    }

    // Required test 10 (A9): every DTO string is ASCII, for both the
    // committed and the rolled-back mapping.
    #[test]
    fn test_every_dto_string_is_ascii() {
        let applier = standalone_applier(9, Vec::new());
        let committed_report = ApplyTransactionReport {
            outcome: ApplyOutcome::Committed {
                tiers_passed: vec![
                    TierResult {
                        tier: ValidationTier::Structural,
                        status: ValidationStatus::Pass,
                    },
                    TierResult {
                        tier: ValidationTier::GeneratedHeaders,
                        status: ValidationStatus::Pass,
                    },
                    TierResult {
                        tier: ValidationTier::AffectedTUs,
                        status: ValidationStatus::Skipped(
                            "no compiler toolchain available to compile affected TUs".to_string(),
                        ),
                    },
                ],
            },
            new_files: Vec::new(),
        };
        let committed_dto = applier.outcome_dto("PLAN-001", &committed_report);
        assert_all_strings_ascii(&committed_dto, "the committed mapping");

        let rolled_report = ApplyTransactionReport {
            outcome: ApplyOutcome::RolledBack {
                failed_tier: TierResult {
                    tier: ValidationTier::GeneratedHeaders,
                    status: ValidationStatus::Fail("Missing include: ghost.h".to_string()),
                },
                tiers_passed: vec![TierResult {
                    tier: ValidationTier::Structural,
                    status: ValidationStatus::Pass,
                }],
            },
            new_files: Vec::new(),
        };
        let rolled_dto = applier.outcome_dto("PLAN-001", &rolled_report);
        assert_all_strings_ascii(&rolled_dto, "the rolled-back mapping");
    }

    // Required test 11 (A1, caller census of record): transform.rs holds
    // exactly definition + CLI wrapper call; this file exactly ONE
    // delegation site. The needle is built with `format!` (the transform.rs
    // idiom) so this test's own source cannot match itself.
    #[test]
    fn test_the_transactional_core_has_exactly_one_applier_delegation_site() {
        let transform_source = include_str!("../transform.rs");
        let applier_source = include_str!("extraction_apply.rs");
        let needle = format!("apply_plan_with{}", "(");
        let core_sites = transform_source.matches(needle.as_str()).count();
        let delegation_sites = applier_source.matches(needle.as_str()).count();
        assert_eq!(
            core_sites, 2,
            "transform.rs must hold exactly the core definition and the CLI wrapper's call, got {core_sites}"
        );
        assert_eq!(
            delegation_sites, 1,
            "the applier must hold exactly ONE delegation site into the core; a second one forks the transactional path"
        );
    }
}
