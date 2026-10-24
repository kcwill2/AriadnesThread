//! Test module moved out of `../extraction_apply.rs` VERBATIM
//! (UTA-T7b row 11: the source file sat at the 1000-line ceiling).
//! `super` is still the `extraction_apply` module and `super::super`
//! still `commands::ui`, so every import path survived the move.
//
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

    mod move_proof;


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
        // CW-2 (shape C4): the generation's plans feed the expected-modified
        // oracle, so the binding is no longer underscored.
        let (_temp_dir, state, proposals, plans) = boot_swapped_fixture("basic");
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
        // C5/SM-I2: the expected sets and the fold are all read PRE-APPLY (C5.1).
        let expectation = move_proof::expectation_pre_apply(&repo_root, &plans, &plan_id,
            &dry_run.rewrites, &state.analysis().expect("boot installed the generation"));

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
        // Faithfulness equality (extraction-consumer-rewrite): the
        // subset-parity clause used to be the whole claim - the "UNSATISFIABLE
        // / journal D6" workaround for the latent vacuous-rewrite bug, then
        // characterized by m20's apply_thing_plan_locks_... test ("latent
        // vacuous-rewrite bug, characterization only"). CW-1's plugin hook
        // makes the consumer rewrite real, so the deferred row's
        // expected-modified equality is the claim of record now; the subset
        // clause survives as belt-and-braces coverage of the
        // outside-the-approval case. The equality is falsifiable: it fails
        // if any rewrite silently drops, any file changes outside the
        // expected set, or any pre-existing file disappears.
        assert!(
            !approved_consumers.is_empty(),
            "the approved dry-run must actually carry consumer rewrite rows, \
             carried none"
        );
        assert!(
            modified.iter().all(|file| expectation.union.contains(file)),
            "the apply must not modify any file outside the SM-I2 expected \
             set (consumers UNION move sources): modified={modified:?}"
        );
        // The post-apply faithfulness audit, factored for the R1 ceiling:
        assert_modified_faithfully_applied(
            &before, &after, &modified, &expectation.union, &expectation.consumers,
            &approved_consumers, &repo_root);
        // C5.4/C5.5: marker proof for the marked original file, incl. SM-I3 parity.
        move_proof::assert_marked_regions(&plans, &plan_id, Path::new("include/thing.h"),
            &before, &after, dry_run.move_removals, &expectation.fold);

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

    /// Faithfulness-oracle helper for required test 1 (invariant X1,
    /// extraction-consumer-rewrite): the expected-modified set is exactly
    /// the approved dry-run's consumer rows whose PRE-APPLY on-disk text
    /// has a directive hit for at least one of the plan's plugin discovery
    /// spellings. Spellings come from the plugin
    /// (`plan_consumer_discovery_targets`), not from the row's
    /// `old_includes` strings, because post-CW-1 the rewrite matches
    /// exactly those spellings (the hook maps paths through the same
    /// bare-file-name rule): discovery's predicate and the rewrite's
    /// matcher agree BY CONSTRUCTION (X1) while the DTO rows stay honest
    /// path data (X3). Host-canonical `consumer_file` strings (the parity
    /// fact recorded above the `canonicalized_lossy` helper) return sorted.
    fn expected_modified_consumers(
        rewrites: &[ariadne_ui::ExtractionRewriteDto],
        plans: &[OptimizationPlan],
        plan_id: &str,
    ) -> Vec<String> {
        use ariadne_lang::DependencyRewriter;
        let plan = plans
            .iter()
            .find(|candidate| candidate.id.0 == plan_id)
            .expect("the approved plan resolves in the generation's plan set");
        let registry = crate::registry::default_registry();
        let spellings = crate::commands::transform::plan_consumer_discovery_targets(&registry, plan)
            .expect("the plan's language registers a rewriting backend");
        let rewriter = crate::registry::plan_backend(
            &registry,
            &plan.target_file,
            |caps| caps.rewriting,
            "rewriting",
        )
        .expect("the plan's language registers a rewriting backend")
        .rewriter();
        let mut expected: Vec<String> = rewrites
            .iter()
            .filter(|row| {
                let consumer_text =
                    std::fs::read_to_string(std::path::Path::new(&row.consumer_file))
                        .expect("an approved consumer exists on disk before the apply");
                spellings.iter().any(|spelling| {
                    !rewriter.find_dependency_targets(&consumer_text, spelling).is_empty()
                })
            })
            .map(|row| row.consumer_file.clone())
            .collect();
        expected.sort();
        expected
    }

    /// Post-apply half of the faithfulness oracle for required test 1
    /// (invariant X1, extraction-consumer-rewrite). Factored out so the
    /// test body stays under the R1 function ceiling - every claim still
    /// runs inside required test 1's call, nothing is weakened:
    /// (1) modified == expected-modified - no rewrite silently dropped,
    /// nothing changed outside the expected set; (2) on this fixture the
    /// expected set IS the approved dry-run's consumer set (the deferred
    /// row's restore-equality claim, machine-witnessed); (3) the A-4
    /// observations row's whole-tree extension - no pre-existing file
    /// outside the artifact trees is deleted by the approved apply.
    fn assert_modified_faithfully_applied(
        before: &BTreeMap<PathBuf, Option<Vec<u8>>>,
        after: &BTreeMap<PathBuf, Option<Vec<u8>>>,
        modified: &[String],
        expected_modified: &[String],
        expected_consumers: &[String],
        approved_consumers: &[String],
        repo_root: &Path,
    ) {
        assert_eq!(
            modified, expected_modified,
            "faithfulness oracle SM-I2 (extraction-symbol-move): the modified \
             set equals the expected set (ex-D6 equality, inputs widened to \
             consumers UNION move sources) - none silently dropped"
        );
        assert_eq!(
            expected_consumers, approved_consumers,
            "the surviving CW-2 sub-claim: on this fixture the expected \
             consumer set IS the approved dry-run's consumer set - the \
             deferred row's restore-equality claim, machine-witnessed"
        );
        // C5.2 fixture equality: the original file became a modified file.
        move_proof::assert_fixture_union(modified, approved_consumers, repo_root);
        // A-4 non-numbered observations row (ledger ~327): nothing present
        // before is absent after, artifact trees excluded (the apply's own
        // record lives there).
        assert!(
            before
                .keys()
                .filter(|relative| !is_artifact_tree(relative))
                .all(|relative| after.contains_key(relative)),
            "the approved apply must not delete any pre-existing file"
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
        let (_temp_dir, _state, proposals, plans) = boot_swapped_fixture("basic");
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
        // include_str! is file-relative: one more `../` than the pre-move
        // inline module needed (the verbatim-move path fix, journaled).
        let transform_source = include_str!("../../transform.rs");
        let applier_source = include_str!("../extraction_apply.rs");
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

    // UTA-T7b row 11: the gate order under a core-level transaction failure.

    #[test]
    fn pin_failed_transaction_burns_the_token_and_reports_failed_text() {
        // The apply gate is a strict pipeline: (1) resolve, (2) try-lock,
        // (3) CONSUME, (4-5) the shared core. A plan whose transaction
        // fails inside the core's own routing (a `.rs` target no plugin
        // claims) surfaces as Failed(Debug-shaped text, ASCII-sanitized)
        // - and the approval is ALREADY consumed at step (3): a retry with
        // the same token answers NotApproved at the token gate, never a
        // second transaction attempt.
        let (_temp_dir, state, _proposals, plans) = boot_swapped_fixture("basic");
        let shipped = plans.first().expect("fixture ships a plan").clone();
        let plan_id = shipped.id.0.clone();
        let doomed = OptimizationPlan {
            target_file: PathBuf::from("solver.rs"),
            ..shipped
        };
        let applier = EngineBackedExtractionApplier::new(
            1,
            state.repo_root.clone(),
            vec![doomed],
            state.analysis().expect("boot installed the generation"),
        );
        let token = applier
            .mint_apply_token(&plan_id)
            .expect("the plan resolves in this generation");
        assert!(
            applier.token_registry_contains(&token, &plan_id),
            "minting registers the approval"
        );

        let refusal = match applier.apply(&plan_id, &token) {
            Ok(outcome) => panic!(
                "a .rs target must fail the core routing, got Ok(committed={})",
                outcome.committed
            ),
            Err(refusal) => refusal,
        };
        let text = match refusal {
            ExtractionApplyRefusal::Failed(text) => text,
            other => panic!("the core must fail the transaction, got {other:?}"),
        };
        assert!(
            text.chars().all(|ch| ch.is_ascii()),
            "A9: the Failed text is ASCII-sanitized, got {text}"
        );
        assert!(
            !text.is_empty(),
            "Failed must carry the core error's Debug-shaped text"
        );

        assert!(
            !applier.token_registry_contains(&token, &plan_id),
            "the token is consumed at transaction start - a failed \
             transaction never leaves an approval behind"
        );
        let retry = match applier.apply(&plan_id, &token) {
            Ok(outcome) => panic!(
                "a burned token must never re-run the transaction, got Ok(committed={})",
                outcome.committed
            ),
            Err(retry) => retry,
        };
        assert!(
            matches!(retry, ExtractionApplyRefusal::NotApproved),
            "the retry is refused at the token gate, not the transaction: {retry:?}"
        );
    }

    // CW-2 (A9 carve-out row, plan_id half): the sanitizer itself, proven
    // directly. The e2e fixture is ASCII-only, which is exactly why the
    // route-side required test 10 alone could not prove the plan_id
    // mapping sanitizes at all.
    #[test]
    fn test_outcome_dto_sanitizes_a_non_ascii_plan_id() {
        let applier = standalone_applier(9, Vec::new());
        let committed_report = ApplyTransactionReport {
            outcome: ApplyOutcome::Committed {
                tiers_passed: vec![TierResult {
                    tier: ValidationTier::Structural,
                    status: ValidationStatus::Pass,
                }],
            },
            new_files: Vec::new(),
        };
        // The non-ASCII byte rides a \u{e9} ESCAPE: the authored source
        // stays pure ASCII (AGENTS.md); the raw char renders at panic time.
        let dto = applier.outcome_dto("PLAN-\u{e9}N-001", &committed_report);
        // ASCII guard first: the handoff's predicted Red panic comes from
        // here pre-fix (outcome_dto passes plan_id through unsanitized).
        assert_all_strings_ascii(&dto, "the non-ASCII plan_id mapping");
        assert_eq!(dto.plan_id, "PLAN-?N-001");
    }
