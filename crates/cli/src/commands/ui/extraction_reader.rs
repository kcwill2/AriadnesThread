//! Read-only extraction manifest/dry-run provider (ui-extraction-wizard
//! W-2a): the host implementation of `ariadne_ui::ExtractionManifestReader`
//! over one analysis generation. The reader is built inside
//! `swap_into_state` right after `install_analysis`, so it answers exactly
//! for the generation whose epoch the wizard routes check (W6 surface).
//!
//! Read-only invariant (W1, W2): no write call exists in this provider. The
//! manifest comes from the isolation engine over the generation's reduced
//! model; the dry-run reproduces `build_migration_preview` exactly (C4):
//! the same `plan_rewrite_targets`, the same consumer discovery, the same
//! `IncludeRewriter::build_preview`, plus the plugin's own copy-vs-move
//! decision (W4) and the paths `tool apply` would write.
//!
//! Engine-sourced includes (W3): every `required_includes` entry comes from
//! `GeneratedUnit.required_deps`. When the engine cannot resolve deps the
//! list is empty and `includes_unavailable` is true - never a fabricated
//! include. The dry-run is memoized per (plan id, epoch) behind a mutex
//! (R3: consumer discovery is the expensive half).

use crate::commands::transform::{
    plan_consumer_discovery_targets, plan_rewrite_targets, plan_target_consumers, PlanConsumers,
};
use ariadne_build::BuildDiscoverer;
use ariadne_cluster::plan::OptimizationPlan;
use ariadne_repo::Repository;
use ariadne_transform::{
    ConsumerRewrite, ExtractionResult, ExtractedSymbol, GeneratedUnit, IncludeRewriter,
    IsolationEngine, MigrationPreview,
};
use ariadne_ui::{
    AnalysisGeneration, ExtractionDryRunDto, ExtractionManifestDto, ExtractionManifestReader,
    ExtractionRewriteDto, ExtractionUnitManifestDto, EXTRACTION_DRY_RUN_MAX_REWRITES,
    EXTRACTION_MANIFEST_MAX_FILES, EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE,
};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// Answers the two wizard routes for one analysis generation. `plans` are
/// the generation's own proposal plans (the ones behind the boot payload's
/// `extraction_proposals` ids); `generation` is the exact
/// `install_analysis` publication this reader was installed for.
pub struct EngineBackedExtractionReader {
    epoch: u64,
    repo_root: PathBuf,
    plans: Vec<OptimizationPlan>,
    generation: Arc<AnalysisGeneration>,
    dry_run_cache: Mutex<HashMap<(String, u64), ExtractionDryRunDto>>,
    consumer_discovery_runs: AtomicUsize,
}

impl EngineBackedExtractionReader {
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
            dry_run_cache: Mutex::new(HashMap::new()),
            consumer_discovery_runs: AtomicUsize::new(0),
        }
    }

    /// R3 observability: how often the expensive consumer-discovery half
    /// actually ran (cache hits must not move this counter). Test-only
    /// probe: no production caller exists, so an ungated inherent method
    /// would be dead code in every non-test build.
    #[cfg(test)]
    pub fn consumer_discovery_runs(&self) -> usize {
        self.consumer_discovery_runs.load(Ordering::SeqCst)
    }

    fn plan_for(&self, plan_id: &str) -> Option<&OptimizationPlan> {
        self.plans.iter().find(|plan| plan.id.0 == plan_id)
    }

    fn dry_run_cache_guard(&self) -> MutexGuard<'_, HashMap<(String, u64), ExtractionDryRunDto>> {
        self.dry_run_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Runs the isolation engine over the generation's reduced model for
    /// one plan, read-only (W2: no write call exists on this path). Body
    /// policy: the manifest returns NO text (kickoff scope-out - bodies
    /// are the apply-time artifact), so every symbol keeps the engine's
    /// placeholder body (invariant I4), which is why `sources` reads
    /// nothing. `required_deps` is computed from the model's dependency
    /// edges, never from bodies, so the W3 data path is unaffected. Also
    /// returns the plugin's staging decision
    /// (`places_generated_units_in_place`) so dry-run callers need no
    /// second registry hop.
    fn run_engine(&self, plan: &OptimizationPlan) -> Option<(ExtractionResult, bool)> {
        let registry = crate::registry::default_registry();
        let Ok(backend) = crate::registry::plan_backend(
            &registry,
            &plan.target_file,
            |caps| caps.extraction,
            "extraction",
        ) else {
            return None;
        };
        let extractor = backend.extractor();
        let places_units_in_place = extractor.places_generated_units_in_place();
        let engine = IsolationEngine::new(&self.generation.reduced_model);
        let extraction = engine.extract(plan, &|_source_file| None, extractor);
        Some((extraction, places_units_in_place))
    }

    /// One destination-file row (shape C2): members ordered by symbol id
    /// (the frozen DTO contract), names capped, `symbol_count` uncapped;
    /// `required_includes` are the engine's `required_deps` lossy-rendered
    /// (W3). `includes_unavailable` is honest in one precise sense: true
    /// iff a member symbol is absent from the generation's reduced model,
    /// which is exactly when the engine can resolve no dependency edges
    /// for that unit. A resolved symbol with no cross-unit dependency
    /// renders an empty, AVAILABLE list - never a fabricated include.
    fn unit_manifest_row(&self, unit: &GeneratedUnit) -> ExtractionUnitManifestDto {
        let mut members: Vec<&ExtractedSymbol> = unit.symbols.iter().collect();
        members.sort_by(|left, right| left.qualified_id.cmp(&right.qualified_id));
        let symbol_count = members.len();
        let symbol_names: Vec<String> = members
            .iter()
            .map(|member| member.name.clone())
            .take(EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE)
            .collect();
        let required_includes: Vec<String> = unit
            .required_deps
            .iter()
            .map(|dep| dep.to_string_lossy().to_string())
            .collect();
        let includes_unavailable = members.iter().any(|member| {
            self.generation
                .reduced_model
                .find_symbol(&member.qualified_id)
                .is_none()
        });
        ExtractionUnitManifestDto {
            path: unit.path.to_string_lossy().to_string(),
            symbol_names,
            symbol_count,
            required_includes,
            includes_unavailable,
        }
    }

    /// The complete C4 answer: the engine pass (unit list plus
    /// includes-availability flags), the apply-identical rewrite targets,
    /// the consumer discovery, and the plugin's own copy-vs-move decision
    /// (W4). `None` whenever a mandatory half cannot be computed
    /// honestly - the caller then answers with the documented honest-absent
    /// shape rather than a partial claim.
    fn compute_dry_run(&self, plan: &OptimizationPlan) -> Option<ExtractionDryRunDto> {
        let (extraction, places_units_in_place) = self.run_engine(plan)?;
        let consumers = self.discover_plan_consumers(plan)?;
        // C4 parity: the SAME rewrite targets `build_migration_preview`
        // feeds the rewriter (the conservative empty-deps units from
        // `plan_rewrite_targets`), never the engine units - the parity
        // test compares against the CLI artifact field by field.
        let (symbol_name_to_header, rewrite_targets) = plan_rewrite_targets(plan);
        let migration = IncludeRewriter::new(symbol_name_to_header, rewrite_targets)
            .build_preview(plan.id.0.clone(), &consumers);
        let rows: Vec<ExtractionUnitManifestDto> = extraction
            .headers
            .iter()
            .map(|unit| self.unit_manifest_row(unit))
            .collect();
        let move_removals = self.move_removal_count(plan, &extraction, &migration);
        let rewrites_truncated = migration.rewrites.len() > EXTRACTION_DRY_RUN_MAX_REWRITES;
        let rewrites: Vec<ExtractionRewriteDto> = migration
            .rewrites
            .iter()
            .take(EXTRACTION_DRY_RUN_MAX_REWRITES)
            .map(rewrite_row)
            .collect();
        Some(ExtractionDryRunDto {
            plan_id: plan.id.0.clone(),
            generation_epoch: self.epoch,
            new_files: staged_write_paths(&extraction, places_units_in_place),
            rewrites,
            rewrites_truncated,
            move_removals,
            copy_only: move_removals == 0,
            includes_unavailable: rows.iter().any(|row| row.includes_unavailable),
            apply_token: String::new(),
        })
    }

    /// The expensive half (R3), built from the same public pieces the CLI
    /// preview's private `find_plan_consumers` composes, so C4 parity is
    /// literally the same code, not a lookalike. Read-only: repository
    /// catalog, build discovery, and TU contents; writes nothing.
    fn discover_plan_consumers(&self, plan: &OptimizationPlan) -> Option<PlanConsumers> {
        self.consumer_discovery_runs.fetch_add(1, Ordering::SeqCst);
        let repository = Repository::locate(&self.repo_root).ok()?;
        let repo_discovery = repository.discover().ok()?;
        let registry = crate::registry::default_registry();
        let build_discoverer = BuildDiscoverer::new(&repository.root)
            .with_repo_discovery(&repo_discovery)
            .with_unit_shaper(registry.unit_shaper_for_repo(&repository.root));
        let build_result = build_discoverer.discover().ok()?;
        let targets = plan_consumer_discovery_targets(&registry, plan).ok()?;
        let all_symbol_names: Vec<String> = plan
            .proposed_extractions
            .iter()
            .map(|member| member.symbol_name.clone())
            .collect();
        Some(plan_target_consumers(
            &registry,
            &crate::registry::catalog_languages(&registry, &repo_discovery),
            &build_result.translation_units,
            &repository.root,
            &targets,
            &plan.target_file,
            &all_symbol_names,
        ))
    }

    /// W4 decided by the plugin exactly as `TransactionApplier` decides: the
    /// moved-unit candidates are offered to the plan-language rewriter's
    /// move hook `move_out_top_level_symbols` with `plan.id.0` - the same
    /// plan id the apply's markers bind (m20's `PLAN-001` byte truth) -
    /// over the on-disk source (read-only). The loop is a FOLD mirroring
    /// validate's `strip_moved_symbols` exactly: BTreeSet candidate order
    /// with a per-path running-text cache, so an offer sees the text the
    /// earlier offer on the same path would leave behind (the merge plus
    /// the idempotence refusals). Each fold success contributes its unit's
    /// symbol count. Computed, not asserted (W4): the fold equals exactly
    /// what the apply will bind, so the dry-run move claim matches the
    /// markers apply writes (SM-I3). A refusal, a copy-only language's
    /// default, or an unreadable file contributes nothing - the file stays
    /// out of the move claim, byte-identical.
    fn move_removal_count(
        &self,
        plan: &OptimizationPlan,
        extraction: &ExtractionResult,
        migration: &MigrationPreview,
    ) -> usize {
        let registry = crate::registry::default_registry();
        let Ok(backend) = crate::registry::plan_backend(
            &registry,
            &plan.target_file,
            |caps| caps.rewriting,
            "rewriting",
        ) else {
            return 0;
        };
        let rewriter = backend.rewriter();
        let mut moved_symbols = 0usize;
        let mut folded_text: HashMap<PathBuf, String> = HashMap::new();
        let candidates = move_removal_candidates(&self.repo_root, migration, extraction);
        for (source_path, moved_names) in candidates {
            let current_text = match folded_text.get(&source_path) {
                Some(text) => text.clone(),
                None => {
                    let Ok(fresh) = std::fs::read_to_string(self.repo_root.join(&source_path))
                    else {
                        continue;
                    };
                    fresh
                }
            };
            if let Some(post_move) = rewriter
                .move_out_top_level_symbols(&current_text, &moved_names, &plan.id.0)
            {
                folded_text.insert(source_path, post_move);
                moved_symbols += moved_names.len();
            }
        }
        moved_symbols
    }
}

impl ExtractionManifestReader for EngineBackedExtractionReader {
    fn epoch(&self) -> u64 {
        self.epoch
    }

    fn manifest(&self, plan_id: &str) -> Option<ExtractionManifestDto> {
        let plan = self.plan_for(plan_id)?;
        let Some((extraction, _places_units_in_place)) = self.run_engine(plan) else {
            // Documented W3 degraded mode (the Red-protocol honest-absent
            // shape): the plan id resolves within this generation but its
            // extraction backend is unresolvable, so there is no engine
            // data to render - an honest empty row set, never a
            // fabrication, and never a wrong 404 for a real plan id.
            return Some(ExtractionManifestDto {
                plan_id: plan.id.0.clone(),
                generation_epoch: self.epoch,
                files: Vec::new(),
                files_truncated: false,
                cycle_error: None,
            });
        };
        let rows: Vec<ExtractionUnitManifestDto> = extraction
            .headers
            .iter()
            .map(|unit| self.unit_manifest_row(unit))
            .collect();
        let files_truncated = rows.len() > EXTRACTION_MANIFEST_MAX_FILES;
        Some(ExtractionManifestDto {
            plan_id: plan.id.0.clone(),
            generation_epoch: self.epoch,
            files: rows.into_iter().take(EXTRACTION_MANIFEST_MAX_FILES).collect(),
            files_truncated,
            cycle_error: cycle_error_of(&extraction),
        })
    }

    fn dry_run(&self, plan_id: &str) -> Option<ExtractionDryRunDto> {
        let plan = self.plan_for(plan_id)?;
        let cache_key = (plan_id.to_string(), self.epoch);
        if let Some(cached) = self.dry_run_cache_guard().get(&cache_key) {
            return Some(cached.clone());
        }
        match self.compute_dry_run(plan) {
            Some(computed) => {
                self.dry_run_cache_guard().insert(cache_key, computed.clone());
                Some(computed)
            }
            // R3: a degraded answer is never cached - the expensive half
            // did not complete, so a later call must still be able to
            // retry and the cache never freezes an honest-absent shape.
            None => Some(honest_absent_dry_run(plan_id, self.epoch)),
        }
    }
}

/// The engine's unit-graph cycle verdict for the DTO; `None` is the
/// honest absence (the engine reports a cycle ONLY through `errors`).
fn cycle_error_of(extraction: &ExtractionResult) -> Option<String> {
    if extraction.errors.is_empty() {
        None
    } else {
        Some(extraction.errors.join("; "))
    }
}

/// Lossy repo-relative rendering of a path list (shape C2: every DTO path
/// is a lossy string; the parity test compares these exact renderings).
fn lossy_paths(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect()
}

/// The mechanical field-for-field mapping `ConsumerRewrite` ->
/// `ExtractionRewriteDto` (the shape C2 note made literal).
fn rewrite_row(rewrite: &ConsumerRewrite) -> ExtractionRewriteDto {
    ExtractionRewriteDto {
        consumer_file: rewrite.consumer_file.to_string_lossy().to_string(),
        old_includes: lossy_paths(&rewrite.old_includes),
        new_includes: lossy_paths(&rewrite.new_includes),
        side_effect_preservations: lossy_paths(&rewrite.side_effect_preservations),
    }
}

/// Moved-unit strip candidates: the provider-local read-only mirror of
/// validate's private `collect_move_removals` derivation (that function
/// decides the same set inside the apply transaction): a rewrite's old
/// dependency that exists on disk, is not the consumer itself, and is not
/// one of this plan's generated targets, paired with each new-include
/// unit's symbol names. `pub(crate)` (extraction-symbol-move SM-4, declared
/// mechanical visibility raise, zero behavior): the apply-side move-proof
/// oracle reads the same derivation validate runs inside apply (SM-I6: no
/// public surface moves).
pub(crate) fn move_removal_candidates(
    repo_root: &Path,
    migration: &MigrationPreview,
    extraction: &ExtractionResult,
) -> BTreeSet<(PathBuf, Vec<String>)> {
    let mut candidates: BTreeSet<(PathBuf, Vec<String>)> = BTreeSet::new();
    let generated_unit_paths: BTreeSet<PathBuf> = extraction
        .headers
        .iter()
        .map(|u| Path::new("generated").join(&u.path))
        .collect();
    for rewrite in &migration.rewrites {
        for old_dep in &rewrite.old_includes {
            if old_dep == &rewrite.consumer_file || is_unit_target(old_dep, extraction) {
                continue;
            }
            if !repo_root.join(old_dep).is_file() {
                continue;
            }
            for unit in &extraction.headers {
                let matches = rewrite.new_includes.contains(&unit.path)
                    || rewrite.new_includes.iter().any(|p| generated_unit_paths.contains(p));
                if !matches || unit.symbols.is_empty() {
                    continue;
                }
                let moved_names: Vec<String> = unit
                    .symbols
                    .iter()
                    .map(|symbol| symbol.name.clone())
                    .collect();
                candidates.insert((old_dep.clone(), moved_names));
            }
        }
    }
    candidates
}

fn is_unit_target(path: &Path, extraction: &ExtractionResult) -> bool {
    extraction
        .headers
        .iter()
        .any(|unit| unit.path.as_path() == path)
}

/// The paths `tool apply` would write: every generated unit under the
/// plugin's staging decision (`places_generated_units_in_place` - the
/// same decision `TransactionApplier::apply_with` follows: in place for
/// move-language plugins, under `generated/` otherwise), rendered
/// repo-relative and lossy.
fn staged_write_paths(extraction: &ExtractionResult, places_units_in_place: bool) -> Vec<String> {
    extraction
        .headers
        .iter()
        .map(|unit| {
            let staged = if places_units_in_place {
                unit.path.clone()
            } else {
                Path::new("generated").join(&unit.path)
            };
            staged.to_string_lossy().to_string()
        })
        .collect()
}

/// The Red-protocol honest-absent dry-run shape (the documented W3
/// degraded mode): nothing computed is claimed, `includes_unavailable`
/// flags the degradation. Used ONLY when a mandatory half failed; never
/// cached, never on a healthy plan.
fn honest_absent_dry_run(plan_id: &str, epoch: u64) -> ExtractionDryRunDto {
    ExtractionDryRunDto {
        plan_id: plan_id.to_string(),
        generation_epoch: epoch,
        new_files: Vec::new(),
        rewrites: Vec::new(),
        rewrites_truncated: false,
        move_removals: 0,
        copy_only: false,
        includes_unavailable: true,
        apply_token: String::new(),
    }
}

#[cfg(test)]
mod tests {
    //! W-2a required tests 1-7 (fixture: the `basic` C++ fixture, PLAN-001
    //! pinned by `tests/golden/basic/plans_plans.json` and by the untouched
    //! `m00_facts_path_parity` lock). Providers are built through the real
    //! boot + `swap_into_state` wiring (the `builder_tests` fixture
    //! idiom, extended to a cold boot of an isolated fixture copy).

    use super::super::{build_ui_payload, swap_into_state, ProgressFn};
    use super::EngineBackedExtractionReader;
    use ariadne_graph::{IncludeGraph, LoadMemo, ReducedSemanticModel};
    use ariadne_ui::{ExtractionManifestReader, ExtractionProposalDto, UiState};
    use ariadne_transform::MigrationPreview;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    /// The W9 byte oracle: `extraction_proposals` serialized bytes captured
    /// verbatim from the starting tree at Red time (live
    /// `build_ui_payload` on an isolated copy of `basic`, no mutation yet).
    /// Backslash separators are the lossy `PathBuf` rendering of this
    /// Windows host, exactly as the capture recorded them.
    const CAPTURED_PROPOSALS_JSON: &str = concat!(
        r#"[{"plan_id":"PLAN-001","target_file":"thing.h","#,
        r#""target_file_id":"5780424644969596122","#,
        r#""problem":"2 symbols of collateral load","#,
        r#""collateral_load_before":2,"collateral_load_after":0,"#,
        r#""symbol_names":["Thing","Thing::Thing","Thing::get_id","ThingConfig"],"#,
        r#""total_symbols":4,"required_headers":[],"consumer_tu_count":2,"#,
        r#""risk":"Conservative","target_files":[{"path":"include\\ThingConfig_extracted.h","#,
        r#""symbol_names":["ThingConfig"],"symbol_count":1},"#,
        r#"{"path":"include\\Thing_Thing_extracted.h","#,
        r#""symbol_names":["Thing::Thing"],"symbol_count":1},"#,
        r#"{"path":"include\\Thing_extracted.h","#,
        r#""symbol_names":["Thing"],"symbol_count":1},"#,
        r#"{"path":"include\\Thing_get_id_extracted.h","#,
        r#""symbol_names":["Thing::get_id"],"symbol_count":1}],"#,
        r#""target_files_truncated":false}]"#,
    );

    /// Boot a fresh fixture copy and swap it into a `new_loading` state
    /// exactly the way `run_ui` boots (install epoch 1). Returns the temp
    /// dir guarding the copy, the state, the payload proposals, and the
    /// generation's own plans (both cloned before the swap moved them).
    fn boot_swapped_fixture(
        fixture: &str,
    ) -> (
        tempfile::TempDir,
        Arc<UiState>,
        Vec<ExtractionProposalDto>,
        Vec<ariadne_cluster::plan::OptimizationPlan>,
    ) {
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

    /// A concrete reader over the SAME booted generation for the inherent
    /// probe surface (`consumer_discovery_runs`); the installed reader is a
    /// trait object and shares no cache with this probe by design.
    fn concrete_reader_for_booted_generation(
        state: &UiState,
        plans: Vec<ariadne_cluster::plan::OptimizationPlan>,
    ) -> Arc<EngineBackedExtractionReader> {
        Arc::new(EngineBackedExtractionReader::new(
            1,
            state.repo_root.clone(),
            plans,
            state.analysis().expect("boot installed the generation"),
        ))
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
    /// content bytes) - the W2 before/after oracle.
    fn tree_hash(root: &Path) -> u64 {
        let mut files: std::collections::BTreeMap<PathBuf, Option<Vec<u8>>> =
            std::collections::BTreeMap::new();
        collect_tree(root, root, &mut files);
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

    fn collect_tree(
        root: &Path,
        directory: &Path,
        files: &mut std::collections::BTreeMap<PathBuf, Option<Vec<u8>>>,
    ) {
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

    fn first_proposal_id(proposals: &[ExtractionProposalDto]) -> String {
        proposals
            .first()
            .expect("the basic fixture boots with at least one proposal")
            .plan_id
            .clone()
    }

    // Required test 1 (W3 + per-file manifest shape).
    #[test]
    fn test_manifest_lists_engine_sourced_includes_for_the_fixture_plan() {
        let (_temp_dir, state, proposals, _plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let reader = installed_reader(&state);
        let manifest = reader
            .manifest(&plan_id)
            .expect("the boot proposal id must resolve within its generation");
        assert_eq!(manifest.plan_id, plan_id);
        assert_eq!(manifest.generation_epoch, 1, "the reader answers its own epoch");

        let proposal = proposals.first().expect("proposal present");
        let manifest_paths: Vec<&str> = manifest.files.iter().map(|row| row.path.as_str()).collect();
        let proposal_paths: Vec<&str> = proposal
            .target_files
            .iter()
            .map(|row| row.path.as_str())
            .collect();
        assert_eq!(
            manifest_paths, proposal_paths,
            "the manifest rows must equal the plan's destination-file rows"
        );
        assert!(!manifest.files_truncated, "4 files stay under the 64-file cap");
        let total_members: usize = manifest.files.iter().map(|row| row.symbol_count).sum();
        assert_eq!(
            total_members, proposal.total_symbols,
            "symbol_count carries the uncapped members"
        );

        let target_paths: Vec<&str> = proposal_paths.clone();
        for row in &manifest.files {
            assert!(!row.includes_unavailable, "the model resolves the fixture plan symbols for {}", row.path);
            for include in &row.required_includes {
                assert!(
                    target_paths.contains(&include.as_str()),
                    "required include {include} was fabricated: it is not a plan destination"
                );
            }
        }

        let thing_row = manifest
            .files
            .iter()
            // File-NAME equality, not a suffix test: "Thing_Thing_extracted.h"
            // also ENDS with "Thing_extracted.h" (masked at Red by the
            // earlier rows assertion - deviation D4 in the journal).
            .find(|row| {
                row.path.rsplit(['\\', '/']).next() == Some("Thing_extracted.h")
            })
            .expect("the Thing destination row exists");
        assert_eq!(thing_row.symbol_names, vec!["Thing".to_string()]);
        assert_eq!(thing_row.symbol_count, 1);
        assert!(
            !thing_row.required_includes.is_empty(),
            "Thing depends on ThingConfig, so its unit must carry the engine-sourced include"
        );
        assert!(
            thing_row.required_includes.iter().any(|include| include
                .ends_with("ThingConfig_extracted.h")),
            "the known inter-unit dependency must name ThingConfig's unit, got {:?}",
            thing_row.required_includes
        );
    }

    // Required test 2 (C4 dry-run parity against the CLI preview).
    #[test]
    fn test_dry_run_rewrites_match_the_cli_migration_preview() {
        let (_boot_dir, state, proposals, _plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let reader = installed_reader(&state);
        let dry_run = reader
            .dry_run(&plan_id)
            .expect("the boot proposal id must resolve within its generation");

        // The CLI path itself, against the SAME booted tree. On this host
        // the discovery stack renders `consumer_file` as an absolute
        // canonical path (`\\?\...`, verified in the first Green run: two
        // copies of the fixture differ ONLY by their temp root), so byte
        // parity is only assertable on one root - and one shared root is
        // the STRONGER parity claim. The preview's plan/migration
        // artifacts land in this test's own temp copy; the W2 read-only
        // proof keeps its own separate boot (deviation D5 in the journal).
        let boot_repo = state.repo_root.clone();
        crate::commands::transform::build_migration_preview(&boot_repo, &plan_id, false)
            .expect("the CLI migration preview must succeed on the fixture");
        let preview_bytes = std::fs::read(
            boot_repo
                .join("artifacts")
                .join("migration")
                .join("preview.json"),
        )
        .expect("the CLI preview artifact");
        let preview: MigrationPreview =
            serde_json::from_slice(&preview_bytes).expect("preview.json decodes");

        assert!(
            !preview.rewrites.is_empty(),
            "the fixture must exercise at least one consumer rewrite"
        );
        assert_eq!(
            dry_run.rewrites.len(),
            preview.rewrites.len(),
            "the dry-run must answer with exactly the CLI preview's rewrites"
        );
        assert!(!dry_run.rewrites_truncated, "2 rewrites stay under the 200 cap");
        for (dto, consumer) in dry_run.rewrites.iter().zip(&preview.rewrites) {
            assert_eq!(dto.consumer_file, consumer.consumer_file.to_string_lossy());
            let lossy = |paths: &[PathBuf]| -> Vec<String> {
                paths.iter().map(|p| p.to_string_lossy().to_string()).collect()
            };
            assert_eq!(dto.old_includes, lossy(&consumer.old_includes));
            assert_eq!(dto.new_includes, lossy(&consumer.new_includes));
            assert_eq!(
                dto.side_effect_preservations,
                lossy(&consumer.side_effect_preservations)
            );
        }

        // new_files == the paths `apply` would write: the plan's destination
        // units under the plugin's staging decision (C++ stages under
        // `generated/` - the default of `places_generated_units_in_place`).
        let proposal = proposals.first().expect("proposal present");
        let mut expected_new: Vec<String> = proposal
            .target_files
            .iter()
            .map(|row| {
                Path::new("generated")
                    .join(&row.path)
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        expected_new.sort();
        let mut actual_new = dry_run.new_files.clone();
        actual_new.sort();
        assert_eq!(
            actual_new, expected_new,
            "the dry-run's new files must equal the apply write set"
        );
    }

    // Required test 3 (R3 cache: two calls, discovery ran once).
    #[test]
    fn test_dry_run_caches_consumer_discovery_per_plan_epoch() {
        let (_temp_dir, state, proposals, plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let provider = concrete_reader_for_booted_generation(&state, plans);
        assert_eq!(provider.consumer_discovery_runs(), 0, "nothing ran before the calls");
        let first = provider
            .dry_run(&plan_id)
            .expect("the boot proposal id must resolve");
        let second = provider
            .dry_run(&plan_id)
            .expect("the second call resolves from cache");
        assert_eq!(
            Some(first.clone()),
            Some(second),
            "the cached answer must equal the first"
        );
        assert_eq!(
            provider.consumer_discovery_runs(),
            1,
            "the expensive consumer-discovery half must run exactly once"
        );
    }

    // Required test 4 (W2 read-only proof).
    #[test]
    fn test_manifest_and_dry_run_leave_the_fixture_tree_identical() {
        let (_temp_dir, state, proposals, _plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let repo_root = state.repo_root.clone();
        let reader = installed_reader(&state);
        let before = tree_hash(&repo_root);
        assert!(reader.manifest(&plan_id).is_some());
        assert!(reader.dry_run(&plan_id).is_some());
        assert_eq!(
            before,
            tree_hash(&repo_root),
            "running a manifest and a dry-run must leave the fixture tree byte-identical"
        );
    }

    // Required test 5 (W4 copy-vs-move honesty on the C++ fixture) -
    // extraction-symbol-move amendment 3 (kickoff section 7 item 3): the
    // dry run states the MOVE truth now that the C++ hook retires the
    // locatable fixture declarations. The old name stated the superseded
    // copy-only truth (the m19 CW-1 rename precedent).
    #[test]
    fn test_dry_run_reports_move_truth_on_the_cpp_fixture() {
        let (_temp_dir, state, proposals, _plans) = boot_swapped_fixture("basic");
        let plan_id = first_proposal_id(&proposals);
        let reader = installed_reader(&state);
        let dry_run = reader
            .dry_run(&plan_id)
            .expect("the boot proposal id must resolve within its generation");
        assert!(
            dry_run.move_removals > 0,
            "the C++ move hook retires the fixture's locatable top-level \
             declaration(s) under ARIADNE-EXTRACT-OUT markers - the dry run \
             must report the move, got move_removals={}",
            dry_run.move_removals
        );
        assert!(
            !dry_run.copy_only,
            "a dry run that reports moves must not claim copy-only \
             semantics, got copy_only={}",
            dry_run.copy_only
        );
    }

    // Required test 6 (W9 boot payload byte lock).
    #[test]
    fn test_boot_payload_extraction_proposals_bytes_are_unchanged() {
        let (_temp_dir, _state, proposals, _plans) = boot_swapped_fixture("basic");
        let serialized =
            serde_json::to_string(&proposals).expect("the proposals payload must serialize");
        assert_eq!(
            serialized, CAPTURED_PROPOSALS_JSON,
            "the boot payload's extraction_proposals bytes must equal the Red-time capture"
        );
    }

    // Required test 7 (W6 surface: the reader reports its own epoch) plus
    // the install-wiring check this hand owes.
    #[test]
    fn test_reader_reports_the_epoch_of_its_generation() {
        let (_temp_dir, state, _proposals, _plans) = boot_swapped_fixture("basic");
        assert!(
            state.extraction_reader().is_some(),
            "swap_into_state must install the reader with the generation"
        );
        let reader = installed_reader(&state);
        assert_eq!(
            reader.epoch(),
            1,
            "boot consumed the first install of a new_loading state (epoch 0 start)"
        );
        let standalone = EngineBackedExtractionReader::new(
            7,
            PathBuf::from("."),
            Vec::new(),
            Arc::new(ariadne_ui::AnalysisGeneration {
                include_graph: IncludeGraph::new(),
                reduced_model: ReducedSemanticModel::default(),
                memo: Arc::new(LoadMemo::new()),
            }),
        );
        assert_eq!(standalone.epoch(), 7, "the reader reports the epoch it was built for");
        assert!(
            standalone.manifest("PLAN-001").is_none(),
            "a reader with no plans resolves no plan (route answers 404)"
        );
    }

    // UTA-T7b row 10: dry-run degraded-mode (honest-absent) pins.

    #[test]
    fn pin_degraded_dry_run_is_honest_absent_and_never_cached() {
        // A reader over a repo root that cannot be canonicalized: the
        // cheap engine half completes (pure in-memory), the expensive
        // consumer-discovery half bails honestly, and `dry_run` answers
        // with the documented W3 honest-absent shape - never cached, so
        // the discovery counter moves on EVERY call (R3 retryability).
        let (_temp_dir, _state, _proposals, plans) = boot_swapped_fixture("basic");
        let shipped = plans.first().expect("the fixture ships a plan").clone();
        let plan_id = shipped.id.0.clone();
        let ghost_root = {
            let scratch = tempfile::tempdir().expect("scratch temp dir");
            scratch.path().join("ghost_repo_that_does_not_exist")
        };
        let reader = EngineBackedExtractionReader::new(
            7,
            ghost_root,
            vec![shipped],
            Arc::new(ariadne_ui::AnalysisGeneration {
                include_graph: IncludeGraph::new(),
                reduced_model: ReducedSemanticModel::default(),
                memo: Arc::new(LoadMemo::new()),
            }),
        );

        assert_eq!(
            reader.dry_run("no-such-plan"),
            None,
            "an unknown plan is a 404, never the honest-absent shape"
        );

        let first = reader.dry_run(&plan_id).expect("a degraded answer is still an answer");
        assert_eq!(first.plan_id, plan_id);
        assert_eq!(first.generation_epoch, 7, "the epoch is reported verbatim");
        assert!(first.new_files.is_empty(), "nothing computed is claimed");
        assert!(first.rewrites.is_empty(), "nothing computed is claimed");
        assert!(!first.rewrites_truncated);
        assert_eq!(first.move_removals, 0);
        assert!(!first.copy_only, "a degraded run must not claim copy-only semantics");
        assert!(first.includes_unavailable, "the W3 degradation flag is set");
        assert!(first.apply_token.is_empty(), "the honest-absent shape mints no token");
        assert_eq!(reader.consumer_discovery_runs(), 1);

        let second = reader.dry_run(&plan_id).expect("the degraded answer is retryable");
        assert_eq!(
            reader.consumer_discovery_runs(),
            2,
            "honest-absent is never cached: the second call ran discovery again"
        );
        assert_eq!(second.plan_id, plan_id);
        assert!(
            second.includes_unavailable && second.apply_token.is_empty(),
            "the retry answers with the same honest-absent shape"
        );
    }
}
