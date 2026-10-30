//! rust-language R-9b2 symbol-isolation suite (amendment-2 sec. 4, amendment-1
//! secs. 5/7): the WIRED Rust extraction transaction end to end on a temp copy
//! of the frozen `rust-basic` fixture (fixture read-only, L3). Stages per
//! amendment-1 sec. 7, VERBATIM; every test owns a tempdir copy and synthetic
//! files (the consumer, the twin pair) are written INTO the copy only.
//! - caps: the suite's own all-true byte-lock, authored GREEN-SIDE with the
//!   flip (R-6 D5 precedent; the red evidence is the renamed in-src pin).
//! - plan: the hook dispatch selects `src/lib.rs` + `src/models/mod.rs`; the
//!   wired planner proposes exactly the two fn extractions; the F2 freeze
//!   shows its observable zero-legs (no Module proposals, no frozen-child
//!   targets, no `crate::greeting`).
//! - preview: the discovery probes are listed honestly (moved ids + sanitized
//!   stem); the wired preview reaches the consumer rewrite; the grown-oracle
//!   (SM-4) dry-runs the shipped `move_out` hook over REAL parent bytes.
//! - apply: the m23-mirror assertion VERBATIM - the temp consumer (authored
//!   IN the copy, `mod consumer;` added to lib.rs's copy, R-6 precedent) ends
//!   up containing `use crate::fixture_marker::fixture_marker;` and
//!   `use crate::models::models_marker::models_marker;` and NOT the old
//!   one-line imports (amendment-1 sec. 1c option (b) + the R-9a verdict's
//!   "F1 FINAL WORD" byte pair); parents lose their fn bytes and gain their
//!   `mod` line exactly once; the frozen fixture children stay byte-exact.
//! - rollback: `tree_digest` (m23 idiom) byte-identity + re-plan convergence
//!   + rollback-then-re-apply byte convergence.
//! - tier-3: `Skipped` via `RustToolchainValidator::with_runner` (scripted
//!   all-None) consumed by `TierValidator::validate_affected_tus` (the
//!   crates/validate mapping consumed, never re-implemented - the granted
//!   manifest dev-dep makes this composition real, journal sec. 8); the
//!   scripted-available counter-leg keeps it non-vacuous and the recorded
//!   call log proves INV-10.
//! - F5 (`flat_id_shared_mod_and_fn_record_is_planning_frozen`): the flat-id
//!   shared fold is frozen at the planning seam; fold-order-dependent bytes
//!   (the folded `kind`/`defining_file`) are deliberately NOT pinned.

use anyhow::Result;
use ariadne_cli::commands::transform::{
    plan_consumer_discovery_targets, plan_rewrite_targets, plan_target_consumers, repository_source_reader,
};
use ariadne_cli::registry::{catalog_languages, default_registry, plan_backend};
use ariadne_cli::{apply_transform_plan, build_migration_preview, plan_extractions};
use ariadne_cluster::plan::{OptimizationPlan, ProposedExtraction};
use ariadne_graph::{ReducedSemanticModel, ReducedSymbolRecord};
use ariadne_lang::{
    ClusteringConstraints, DependencyRewriter, LanguageCapabilities, LanguageId, SourceExtractor,
    SymbolKind, SymbolRecord,
};
use ariadne_lang_rust::validator::{CommandOutcome, CommandRunner};
use ariadne_lang_rust::{RustClusteringConstraints, RustToolchainValidator};
use ariadne_pipeline::Pipeline;
use ariadne_repo::FileId;
use ariadne_transform::{ExtractionResult, IncludeRewriter, MigrationPreview};
use ariadne_validate::{TierValidator, TransactionApplier};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const FIXTURE: &str = "rust-basic";
const LIB_REL: &str = "src/lib.rs";
const GREETING_REL: &str = "src/greeting.rs";
const MODELS_REL: &str = "src/models/mod.rs";
const USER_REL: &str = "src/models/user.rs";
const MACRO_REL: &str = "src/macro_specimen.rs";
const CONSUMER_REL: &str = "src/consumer.rs";

/// The temp-copy consumer (R-6 suite-authored precedent): honest written paths
/// for both aggregation files' fns; `mod consumer;` rides the lib.rs copy.
/// These are the OLD imports the VERBATIM apply assertion requires gone after.
const CONSUMER_SOURCE: &str = "use crate::fixture_marker;\nuse crate::models::models_marker;\n";

/// The F5 leg-3 twin pair: two files each declaring `fn twin()`; the flat-id
/// analyzer gives both the SAME id `crate::twin` - the ambiguity proof the
/// planning freeze must refuse; twin_a's out-of-line `mod twin_b;` lets the
/// hook qualify it. The shipped fold dedupes by DeclId identity
/// (`<id>#L<line>`), so twin_b's fn rides a distinct written line: two
/// DECLARATIONS, not one deduped DeclId.
const TWIN_A_REL: &str = "src/twin_a.rs";
const TWIN_B_REL: &str = "src/twin_b.rs";
const TWIN_A_SOURCE: &str = "pub fn twin() -> &'static str {\n    \"from twin_a\"\n}\n\nmod twin_b;\n";
const TWIN_B_SOURCE: &str = "// twin pair, second file: the written fn rides a distinct line so\n\
     // the fold carries two DeclIds (the shipped fold dedupes by identity).\npub fn twin() -> &'static str {\n    \"from twin_b\"\n}\n";

struct IsolatedRustRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("fixtures").join(name)
}

/// Throwaway copy of the frozen fixture (m19/m22 isolation; every synthetic
/// file and every artifact lands only inside the copy).
fn isolated_rust_repo(label: &str) -> Result<IsolatedRustRepo> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join(label);
    copy_tree(&fixture_path(FIXTURE), &root)?;
    Ok(IsolatedRustRepo { _temp: temp, root })
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// The suite consumer authored INTO the copy: `src/consumer.rs` plus the
/// `mod consumer;` line added to lib.rs's copy (handoff item 5).
fn add_temp_consumer(root: &Path) -> Result<()> {
    fs::write(root.join(CONSUMER_REL), CONSUMER_SOURCE)?;
    let lib_text = fs::read_to_string(root.join(LIB_REL))?;
    let grown = lib_text.replace("mod models;\n", "mod models;\nmod consumer;\n");
    assert_ne!(lib_text, grown, "the copy's lib.rs declares `mod models;`");
    fs::write(root.join(LIB_REL), grown)?;
    Ok(())
}

fn normalized_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

/// The serde label of a `ValidationStatus` value ("Pass" | "Fail" |
/// "Skipped") - the m23 helper idiom.
fn status_label(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.to_string();
    }
    value.as_object().and_then(|map| map.keys().next().cloned()).unwrap_or_default()
}

fn serialized_status_label(status: &impl serde::Serialize) -> String {
    let value = serde_json::to_value(status).expect("ValidationStatus is serializable");
    status_label(&value)
}

/// sha256(relpath || bytes) per file, whole subtree, sorted - the rollback
/// dir-hash identity digest (the m23 idiom).
fn tree_digest(root: &Path) -> Result<Vec<(String, String)>> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) -> Result<()> {
        let mut entries: Vec<PathBuf> = fs::read_dir(dir)?.map(|entry| entry.unwrap().path()).collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, root, out)?;
            } else {
                let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
                let mut hasher = Sha256::new();
                hasher.update(rel.as_bytes());
                hasher.update(fs::read(&path)?);
                out.push((rel, format!("{:x}", hasher.finalize())));
            }
        }
        Ok(())
    }
    let mut digest = Vec::new();
    walk(root, root, &mut digest)?;
    Ok(digest)
}

fn count_line_matches(text: &str, wanted: &str) -> usize {
    text.lines().filter(|line| line.trim() == wanted).count()
}

fn reduced_model(root: &Path) -> Result<ReducedSemanticModel> {
    let mut pipeline = Pipeline::new(root, None)?;
    Ok(pipeline.get_or_build_reduced()?)
}

/// The reduced model's records restated in the constraints-seam grammar (the
/// fields the shipped freeze reads: id, name, kind, declarations).
fn model_records_as_lang_api(model: &ReducedSemanticModel) -> Vec<SymbolRecord> {
    model
        .symbols
        .values()
        .map(|record| SymbolRecord {
            id: record.id.clone(),
            name: record.name.clone(),
            kind: record.kind.clone(),
            declarations: record.declarations.clone(),
            ..SymbolRecord::default()
        })
        .collect()
}

fn plan_for<'a>(plans: &'a [OptimizationPlan], rel: &str) -> &'a OptimizationPlan {
    plans
        .iter()
        .find(|plan| normalized_rel(&plan.target_file) == rel)
        .unwrap_or_else(|| panic!("a plan targets {rel}"))
}

/// The committed transaction's tier labels from the wired apply artifacts
/// (m23's `assert_committed_offline_tiers` shape).
fn committed_tier_labels(root: &Path) -> Result<Vec<String>> {
    let manifest = read_json(&root.join("artifacts/transaction/manifest.json"))?;
    assert_eq!(manifest["outcome"], "Committed", "the apply commits (I7)");
    let outcome = read_json(&root.join("artifacts/transaction/outcome.json"))?;
    let labels: Vec<String> = outcome["Committed"]["tiers_passed"]
        .as_array()
        .expect("committed tiers")
        .iter()
        .map(|tier| status_label(&tier["status"]))
        .collect();
    Ok(labels)
}

/// Byte census of the frozen fixture children (never a plan child, never a
/// rewrite target).
fn assert_frozen_children_untouched(root: &Path) -> Result<()> {
    for rel in [GREETING_REL, USER_REL] {
        assert_eq!(
            fs::read(root.join(rel))?,
            fs::read(fixture_path(FIXTURE).join(rel))?,
            "{rel} stays byte-identical through the transaction"
        );
    }
    Ok(())
}

fn sorted_file_ids(files: &[FileId]) -> Vec<FileId> {
    let mut sorted = files.to_vec();
    sorted.sort_by_key(|file| file.0);
    sorted
}

fn proposal_signature(plans: &[OptimizationPlan]) -> Vec<(String, String, String)> {
    let mut signature: Vec<(String, String, String)> = plans
        .iter()
        .flat_map(|plan| {
            plan.proposed_extractions.iter().map(move |proposal| {
                (
                    normalized_rel(&plan.target_file),
                    proposal.symbol_id.0.clone(),
                    normalized_rel(&proposal.target_header),
                )
            })
        })
        .collect();
    signature.sort();
    signature
}

// --- Shared stage composition (apply-identical, the m23 rollback-test shape):
// engine extraction, apply-identical rewrite targets, neutral discovery,
// TransactionApplier::apply_with.

struct StagedPlan<'a> {
    extraction: ExtractionResult,
    extractor: &'a dyn SourceExtractor,
    rewriter: &'a dyn DependencyRewriter,
}

fn stage_plan<'a>(
    root: &Path,
    plan: &OptimizationPlan,
    registry: &'a ariadne_lang::LanguageRegistry,
) -> Result<StagedPlan<'a>> {
    let mut pipeline = Pipeline::new(root, None)?;
    let model = pipeline.get_or_build_reduced()?;
    let source_reader = repository_source_reader(&pipeline.repo)?;
    let extractor =
        plan_backend(registry, &plan.target_file, |caps| caps.extraction, "extraction")?.extractor();
    let extraction =
        ariadne_transform::IsolationEngine::new(&model).extract(plan, &source_reader, extractor);
    let rewriter =
        plan_backend(registry, &plan.target_file, |caps| caps.rewriting, "rewriting")?.rewriter();
    Ok(StagedPlan { extraction, extractor, rewriter })
}

fn compose_plan_migration(root: &Path, plans: &[OptimizationPlan], rel: &str) -> Result<MigrationPreview> {
    let plan = plan_for(plans, rel);
    let registry = default_registry();
    let mut pipeline = Pipeline::new(root, None)?;
    pipeline.get_or_build_reduced()?;
    let targets = plan_consumer_discovery_targets(&registry, plan)?;
    let all_names: Vec<String> =
        plan.proposed_extractions.iter().map(|proposal| proposal.symbol_name.clone()).collect();
    let consumers = plan_target_consumers(
        &registry,
        &catalog_languages(&registry, &pipeline.discovery),
        &pipeline.build.translation_units,
        root,
        &targets,
        &plan.target_file,
        &all_names,
    );
    let (symbol_name_to_header, units) = plan_rewrite_targets(plan);
    Ok(IncludeRewriter::new(symbol_name_to_header, units).build_preview(plan.id.0.clone(), &consumers))
}

/// Apply ONE plan through the applier core (no tiers/commit - the m23
/// rollback idiom), returning the rollback handle.
fn apply_one_plan_with_transaction(
    root: &Path,
    plans: &[OptimizationPlan],
    rel: &str,
) -> Result<(ariadne_validate::FileSnapshot, Vec<PathBuf>)> {
    let registry = default_registry();
    let plan = plan_for(plans, rel);
    let migration = compose_plan_migration(root, plans, rel)?;
    let staged = stage_plan(root, plan, &registry)?;
    Ok(TransactionApplier::apply_with(
        root,
        &staged.extraction,
        &migration,
        staged.rewriter,
        staged.extractor,
    )?)
}

/// Apply BOTH plans (lib.rs first), digest the applied tree, then roll back in
/// reverse order. The digest rides the `src/` subtree - the transaction's
/// write surface; `artifacts/` manifests carry build stamps no byte claim can
/// honestly re-verify, and no transaction path writes there.
fn apply_pair_capture_digest_then_rollback(
    root: &Path,
    plans: &[OptimizationPlan],
) -> Result<Vec<(String, String)>> {
    let mut captures = Vec::new();
    for rel in [LIB_REL, MODELS_REL] {
        captures.push(apply_one_plan_with_transaction(root, plans, rel)?);
    }
    let applied = tree_digest(&root.join("src"))?;
    for (snapshot, new_paths) in captures.into_iter().rev() {
        TransactionApplier::rollback(&snapshot, &new_paths)?;
    }
    Ok(applied)
}

/// The wired apply of ONE plan (preview then apply, tiers asserted from the
/// committed artifacts) - shared by the apply-stage tests.
fn wired_apply_plan(root: &PathBuf, plans: &[OptimizationPlan], rel: &str) -> Result<()> {
    let plan_id = plan_for(plans, rel).id.0.clone();
    build_migration_preview(root, &plan_id, false)?;
    apply_transform_plan(root, &plan_id)?;
    assert_eq!(
        committed_tier_labels(root)?,
        vec!["Pass", "Pass", "Pass"],
        "wired apply on a toolchain-present machine commits with tiers [Pass, Pass, Pass]"
    );
    Ok(())
}

// --- 1. The suite's own all-true caps byte-lock (authored GREEN-SIDE with
// the flip, R-6 D5 precedent; amendment-2 sec. 4 "caps").

#[test]
fn capabilities_lock_the_symbol_isolation_parity_literal() {
    let registry = default_registry();
    let rust = registry
        .plugin_for_id(LanguageId::Rust)
        .expect("Rust registered in the CLI default registry (R-4)");
    assert_eq!(
        rust.capabilities(),
        LanguageCapabilities {
            discovery: true, dependency_graph: true, semantic_index: true, load_metric: true,
            planning: true, extraction: true, rewriting: true, validation: true,
            requires_toolchain: false,
        },
        "the suite's byte-lock: all eight stage bits true (INV-5 final \
         literal), requires_toolchain false (A3/D-2)"
    );
    let caps = rust.capabilities();
    let enabled_stage_bits = [
        caps.discovery, caps.dependency_graph, caps.semantic_index, caps.load_metric,
        caps.planning, caps.extraction, caps.rewriting, caps.validation,
    ];
    assert_eq!(
        enabled_stage_bits.iter().filter(|bit| **bit).count(),
        8,
        "the true-count pin: all eight stage bits true at full parity"
    );
}
// --- 2. Hook dispatch: the aggregation filter answers over every copy file
// and the target-naming hook lands `{symbol}.rs` beside the aggregation file.

#[test]
fn aggregation_hook_dispatch_selects_exactly_the_two_aggregation_files() -> Result<()> {
    let repo = isolated_rust_repo("hooks")?;
    add_temp_consumer(&repo.root)?;
    let registry = default_registry();
    let plugin = registry.plugin_for_id(LanguageId::Rust).expect("Rust registered");

    let expected: [(&str, bool); 6] = [
        (LIB_REL, true), (MODELS_REL, true), (GREETING_REL, false),
        (USER_REL, false), (MACRO_REL, false), (CONSUMER_REL, false),
    ];
    let mut aggregation_files: Vec<String> = Vec::new();
    for (rel, is_aggregation) in expected {
        let answer = plugin.aggregation_filter(&repo.root, Path::new(rel));
        assert_eq!(answer, Some(is_aggregation), "{rel}: the hook answers Some(its written aggregation truth), never None (planning is enrolled)");
        if answer == Some(true) {
            aggregation_files.push(rel.to_string());
        }
    }
    assert_eq!(aggregation_files, vec![LIB_REL.to_string(), MODELS_REL.to_string()], "the hook qualifies EXACTLY the two aggregation files (out-of-line `mod` declarations)");

    let lib_target = plugin
        .target_path_for(Path::new(LIB_REL), "crate::fixture_marker")
        .expect("the target-naming hook is enrolled");
    assert_eq!(
        normalized_rel(&lib_target),
        "src/fixture_marker.rs",
        "the qualifier is stripped and `{{symbol}}.rs` lands beside src/lib.rs"
    );
    let models_target = plugin
        .target_path_for(Path::new(MODELS_REL), "crate::models_marker")
        .expect("the target-naming hook is enrolled");
    assert_eq!(
        normalized_rel(&models_target),
        "src/models/models_marker.rs",
        "the extraction target lands beside src/models/mod.rs"
    );
    Ok(())
}

// --- 3. Plan stage: the wired planner proposes EXACTLY the two fn
// extractions, with the freeze's observable zero-legs.

#[test]
fn wired_plan_proposes_exactly_the_two_fn_extractions() -> Result<()> {
    let repo = isolated_rust_repo("plan")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;

    let mut targets: Vec<String> = plans.iter().map(|plan| normalized_rel(&plan.target_file)).collect();
    targets.sort();
    assert_eq!(
        targets,
        vec![LIB_REL.to_string(), MODELS_REL.to_string()],
        "the candidate set is EXACTLY the two aggregation files"
    );

    let lib_plan = plan_for(&plans, LIB_REL);
    assert_eq!(
        lib_plan.proposed_extractions.len(),
        1,
        "lib.rs proposes exactly one extraction (the mod records are Module-frozen)"
    );
    let fixture_proposal = &lib_plan.proposed_extractions[0];
    assert_eq!(fixture_proposal.symbol_id.0, "crate::fixture_marker");
    assert_eq!(fixture_proposal.symbol_name, "fixture_marker");
    assert_eq!(
        normalized_rel(&fixture_proposal.target_header),
        "src/fixture_marker.rs",
        "the extraction target lands beside the aggregation file"
    );
    assert_eq!(
        fixture_proposal.source_file,
        Some(FileId::from_relative_path(LIB_REL)),
        "the proposal carries its source provenance (D-M23-3 idiom)"
    );

    let models_plan = plan_for(&plans, MODELS_REL);
    assert_eq!(
        models_plan.proposed_extractions.len(),
        1,
        "models/mod.rs proposes exactly one extraction (`pub mod user;` is Module-frozen)"
    );
    let models_proposal = &models_plan.proposed_extractions[0];
    assert_eq!(models_proposal.symbol_id.0, "crate::models_marker");
    assert_eq!(normalized_rel(&models_proposal.target_header), "src/models/models_marker.rs");

    assert_freeze_zero_legs(&repo.root, &plans)?;

    let persisted = read_json(&repo.root.join("artifacts/plans/plans.json"))?;
    assert_eq!(
        persisted.as_array().map(Vec::len),
        Some(2),
        "the persisted plan stage mirrors the two returned plans"
    );
    Ok(())
}

/// The freeze's observable zero-legs over any plan set: no proposal names a
/// Module-kind symbol, no proposal targets a frozen child file, no proposal
/// names `crate::greeting` (F5 leg 2).
fn assert_freeze_zero_legs(root: &Path, plans: &[OptimizationPlan]) -> Result<()> {
    let model = reduced_model(root)?;
    for plan in plans {
        for proposal in &plan.proposed_extractions {
            if let Some(record) = model.symbols.get(&proposal.symbol_id) {
                assert_ne!(
                    record.kind,
                    SymbolKind::Module,
                    "F2: no proposal may name a Module-kind symbol ({} is Module)",
                    proposal.symbol_id.0
                );
            }
            let target_rel = normalized_rel(&proposal.target_header);
            assert_ne!(target_rel, GREETING_REL, "no proposal targets the frozen src/greeting.rs");
            assert_ne!(target_rel, USER_REL, "no proposal targets the frozen src/models/user.rs");
            assert_ne!(
                proposal.symbol_id.0, "crate::greeting",
                "F5 leg 2: no proposal names crate::greeting"
            );
        }
    }
    Ok(())
}

// --- 4. Preview stage: the discovery probes the plugin lists, the consumer
// rewrite the preview reaches, and the parent-MODIFIED grown-oracle on the
// engine's REAL dry-run bytes (nothing written to disk by this test).

#[test]
fn preview_lists_discovery_targets_and_the_parent_oracle_names_the_mod_line() -> Result<()> {
    let repo = isolated_rust_repo("preview")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;
    let registry = default_registry();

    let lib_probes = plan_consumer_discovery_targets(&registry, plan_for(&plans, LIB_REL))?;
    assert_eq!(
        lib_probes,
        vec!["crate::fixture_marker".to_string(), "lib".to_string()],
        "plan 1 probes: the moved id (use-space) plus the sanitized target stem (mod-space)"
    );
    let models_probes = plan_consumer_discovery_targets(&registry, plan_for(&plans, MODELS_REL))?;
    assert_eq!(
        models_probes,
        // R-9b3 re-pin (AMENDMENT-3 ruling 4b, dated 2026-10-27): the old
        // two-probe list pinned the discovery defect; INV-R9B3-1 rides now.
        vec!["crate::models_marker".to_string(), "crate::models::models_marker".to_string(), "mod".to_string()],
        "plan 2 probes: the moved id, its honest parent-qualified spelling, plus the mod.rs stem (an inert mod-space anchor, pinned in-crate)"
    );

    let lib_plan_id = plan_for(&plans, LIB_REL).id.0.clone();
    build_migration_preview(&repo.root, &lib_plan_id, false)?;
    let preview = read_json(&repo.root.join("artifacts/migration/preview.json"))?;
    let consumer_rewrite = preview["rewrites"]
        .as_array()
        .expect("preview rewrites array")
        .iter()
        .find(|rewrite| {
            normalized_rel(Path::new(rewrite["consumer_file"].as_str().unwrap_or_default()))
                .ends_with(CONSUMER_REL)
        })
        .expect("the consumer is discovered for plan 1 through the wired route");
    let new_includes: Vec<String> = consumer_rewrite["new_includes"]
        .as_array()
        .expect("new_includes array")
        .iter()
        .map(|entry| normalized_rel(Path::new(entry.as_str().unwrap_or_default())))
        .collect();
    assert!(
        new_includes.iter().any(|path| path == "src/fixture_marker.rs"),
        "the preview redirect points the consumer at the moved unit: {new_includes:?}"
    );

    assert_parent_grown_oracle(
        &repo.root,
        LIB_REL,
        "fixture_marker",
        "mod fixture_marker;",
        &["mod greeting;", "mod models;"],
    )?;
    assert_parent_grown_oracle(
        &repo.root,
        MODELS_REL,
        "models_marker",
        "pub mod models_marker;",
        &["pub mod user;"],
    )?;
    Ok(())
}

/// The grown-oracle (SM-4 discipline): the shipped `move_out` hook run over
/// the REAL parent bytes as a dry-run - names the parent path AND the exact
/// inserted `mod` line, on bytes the engine itself produces. The engine's
/// re-offer after a completed move refuses, so the oracle is non-vacuous.
fn assert_parent_grown_oracle(
    root: &Path,
    parent_rel: &str,
    fn_name: &str,
    inserted_mod: &str,
    survivor_lines: &[&str],
) -> Result<()> {
    let registry = default_registry();
    let rewriter =
        plan_backend(&registry, &PathBuf::from(parent_rel), |caps| caps.rewriting, "rewriting")?
            .rewriter();
    let parent_before = fs::read_to_string(root.join(parent_rel))?;
    let parent_after = rewriter
        .move_out_top_level_symbols(&parent_before, &[fn_name.to_string()], "PLAN-R9B2")
        .unwrap_or_else(|| panic!("the move hook accepts {fn_name} in {parent_rel}"));
    assert!(
        !parent_after.contains(&format!("pub fn {fn_name}")),
        "the parent {parent_rel} loses its fn bytes in the dry-run"
    );
    assert_eq!(
        count_line_matches(&parent_after, inserted_mod),
        1,
        "the parent {parent_rel} gains the inserted `{inserted_mod}` line exactly once"
    );
    for survivor in survivor_lines {
        assert!(
            parent_after.contains(survivor),
            "the parent {parent_rel} keeps its pre-existing `{survivor}` line through the dry-run"
        );
    }
    Ok(())
}

// --- 5-7. Apply stage. The m23-mirror assertion VERBATIM (weakening =
// blocker): consumer bytes gain the new imports and lose the old one-line
// imports; parents lose their fn bytes and gain their `mod` line exactly
// once; the frozen fixture children stay byte-exact.

#[test]
fn apply_moves_fixture_marker_and_rewrites_the_consumer_import() -> Result<()> {
    let repo = isolated_rust_repo("apply-plan-one")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;
    wired_apply_plan(&repo.root, &plans, LIB_REL)?;

    let consumer = fs::read_to_string(repo.root.join(CONSUMER_REL))?;
    assert!(
        consumer.contains("use crate::fixture_marker::fixture_marker;"),
        "the consumer import retargets onto the moved unit"
    );
    assert!(
        !consumer.contains("use crate::fixture_marker;"),
        "the old one-line import is GONE (NOT-OLD leg, VERBATIM)"
    );
    assert!(
        consumer.contains("use crate::models::models_marker;"),
        "plan 1 leaves the models-side import byte-untouched"
    );
    let lib_after = fs::read_to_string(repo.root.join(LIB_REL))?;
    assert!(!lib_after.contains("pub fn fixture_marker"), "the parent loses the fn bytes (move, not copy)");
    assert_eq!(
        count_line_matches(&lib_after, "mod fixture_marker;"),
        1,
        "the parent gains `mod fixture_marker;` exactly once"
    );
    assert!(
        lib_after.contains("mod greeting;") && lib_after.contains("mod models;"),
        "the pre-existing module declarations survive"
    );
    let moved_unit = fs::read_to_string(repo.root.join("src/fixture_marker.rs"))?;
    assert!(
        moved_unit.contains("extracted Rust unit"),
        "the moved unit is the engine's generated file (generation header marker)"
    );
    assert!(
        moved_unit.contains("pub fn fixture_marker") && moved_unit.contains("\"rust-basic\""),
        "the moved unit carries the moved fn - syn-printed by contract (the \
         generation header states 'syn-printed content'; the match is \
         token-spacing-free because the token printer owns spacing)"
    );
    assert_frozen_children_untouched(&repo.root)?;
    Ok(())
}

#[test]
fn apply_moves_models_marker_and_rewrites_the_consumer_import() -> Result<()> {
    let repo = isolated_rust_repo("apply-plan-two")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;
    wired_apply_plan(&repo.root, &plans, MODELS_REL)?;

    let consumer = fs::read_to_string(repo.root.join(CONSUMER_REL))?;
    assert!(
        consumer.contains("use crate::models::models_marker::models_marker;"),
        "F1 FINAL WORD byte pair: the derived written spelling retargets the consumer"
    );
    assert!(
        !consumer.contains("use crate::models::models_marker;"),
        "the old one-line import is GONE (NOT-OLD leg, VERBATIM)"
    );
    let models_after = fs::read_to_string(repo.root.join(MODELS_REL))?;
    assert!(!models_after.contains("pub fn models_marker"), "the parent loses the fn bytes (move, not copy)");
    assert_eq!(
        count_line_matches(&models_after, "pub mod models_marker;"),
        1,
        "the parent gains `pub mod models_marker;` exactly once (visibility copied from `pub mod user;`)"
    );
    assert!(
        models_after.contains("pub mod user;"),
        "the pre-existing `pub mod user;` declaration survives"
    );
    let moved_unit = fs::read_to_string(repo.root.join("src/models/models_marker.rs"))?;
    assert!(
        moved_unit.contains("extracted Rust unit"),
        "the moved unit is the engine's generated file (generation header marker)"
    );
    assert!(
        moved_unit.contains("pub fn models_marker") && moved_unit.contains("\"models\""),
        "the moved unit carries the moved fn - syn-printed by contract (the \
         generation header states 'syn-printed content'; the match is \
         token-spacing-free because the token printer owns spacing)"
    );
    assert_frozen_children_untouched(&repo.root)?;
    Ok(())
}

#[test]
fn apply_isolates_both_functions_and_rewrites_the_consumer_verbatim() -> Result<()> {
    let repo = isolated_rust_repo("apply-combined")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;
    for rel in [LIB_REL, MODELS_REL] {
        wired_apply_plan(&repo.root, &plans, rel)?;
    }

    let consumer = fs::read_to_string(repo.root.join(CONSUMER_REL))?;
    assert!(
        consumer.contains("use crate::fixture_marker::fixture_marker;"),
        "consumer gains the fixture_marker retarget"
    );
    assert!(
        consumer.contains("use crate::models::models_marker::models_marker;"),
        "consumer gains the models_marker retarget (segment-repeat spelling, pinned)"
    );
    assert!(
        !consumer.contains("use crate::fixture_marker;") && !consumer.contains("use crate::models::models_marker;"),
        "NEITHER old one-line import survives (the VERBATIM NOT-OLD pair)"
    );
    assert_eq!(
        count_line_matches(&fs::read_to_string(repo.root.join(LIB_REL))?, "mod fixture_marker;"),
        1,
        "lib.rs gains its mod line exactly once"
    );
    assert_eq!(
        count_line_matches(&fs::read_to_string(repo.root.join(MODELS_REL))?, "pub mod models_marker;"),
        1,
        "models/mod.rs gains its mod line exactly once"
    );
    assert_frozen_children_untouched(&repo.root)?;
    Ok(())
}

// --- 8-9. Rollback stage (m23 idiom): bytes back by dir-hash identity,
// re-plan convergence, and rollback-then-re-apply byte convergence.

#[test]
fn rollback_restores_the_tree_and_replan_converges() -> Result<()> {
    let repo = isolated_rust_repo("rollback")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;
    let proposals_before = proposal_signature(&plans);
    // Digest the src/ subtree: the transaction's write surface. artifacts/
    // carries stage manifests with build stamps - no byte-identity claim
    // survives them, and no transaction path writes there.
    let before = tree_digest(&repo.root.join("src"))?;

    let mut captures = Vec::new();
    for rel in [LIB_REL, MODELS_REL] {
        captures.push(apply_one_plan_with_transaction(&repo.root, &plans, rel)?);
    }
    let applied = tree_digest(&repo.root.join("src"))?;
    assert_ne!(applied, before, "the pair of applies mutates the tree (sanity)");

    for (snapshot, new_paths) in captures.into_iter().rev() {
        TransactionApplier::rollback(&snapshot, &new_paths)?;
        TransactionApplier::verify_restored(&snapshot)
            .map_err(|reason| anyhow::anyhow!("rollback verification failed: {reason}"))?;
    }
    assert_eq!(
        tree_digest(&repo.root.join("src"))?,
        before,
        "rollback restores dir-hash identity across the src subtree (the transaction's write surface)"
    );
    assert!(
        fs::read_to_string(repo.root.join(LIB_REL))?.contains("pub fn fixture_marker"),
        "lib.rs regains its fn bytes"
    );
    assert!(
        fs::read_to_string(repo.root.join(MODELS_REL))?.contains("pub fn models_marker"),
        "models/mod.rs regains its fn bytes"
    );

    fs::remove_dir_all(repo.root.join("artifacts"))?;
    let replanned = plan_extractions(&repo.root)?;
    assert_eq!(
        proposal_signature(&replanned),
        proposals_before,
        "re-plan after rollback converges to the pre-apply proposals"
    );
    Ok(())
}

#[test]
fn reapply_after_rollback_converges_to_the_same_bytes() -> Result<()> {
    let repo = isolated_rust_repo("reapply")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;

    let first_run = apply_pair_capture_digest_then_rollback(&repo.root, &plans)?;
    let second_run = apply_pair_capture_digest_then_rollback(&repo.root, &plans)?;
    assert_eq!(
        first_run,
        second_run,
        "rollback-then-re-apply byte-converges (the transaction is byte-deterministic)"
    );
    Ok(())
}

// --- 10. Tier-3 stage: `Skipped` through the scripted all-None runner
// consumed by `TierValidator::validate_affected_tus` (the crates/validate
// mapping consumed, never re-implemented), with the scripted-available
// counter-leg and the INV-10 call-log witness.

/// A scripted [`CommandRunner`]: every call is recorded into a shared log;
/// unlisted binaries answer `None` (the all-None script is the empty map).
struct ScriptedProbeRunner {
    answers: HashMap<&'static str, Option<CommandOutcome>>,
    call_log: Arc<Mutex<Vec<String>>>,
}

impl ScriptedProbeRunner {
    fn all_none() -> (Self, Arc<Mutex<Vec<String>>>) {
        let call_log = Arc::new(Mutex::new(Vec::new()));
        (Self { answers: HashMap::new(), call_log: Arc::clone(&call_log) }, call_log)
    }

    fn cargo_available() -> (Self, Arc<Mutex<Vec<String>>>) {
        let call_log = Arc::new(Mutex::new(Vec::new()));
        let mut answers = HashMap::new();
        answers.insert(
            "cargo",
            Some(CommandOutcome {
                success: true,
                stdout: "cargo 0.0.0-scripted\n".into(),
                stderr: String::new(),
            }),
        );
        (Self { answers, call_log: Arc::clone(&call_log) }, call_log)
    }
}

impl CommandRunner for ScriptedProbeRunner {
    fn run(&self, binary: &str, args: &[&str]) -> Option<CommandOutcome> {
        self.call_log.lock().expect("the call log lock").push(format!("{binary} {}", args.join(" ")));
        self.answers.get(binary).cloned().flatten()
    }
}

fn recorded_calls(call_log: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    call_log.lock().expect("the call log lock").clone()
}

#[test]
fn tier_three_skips_via_scripted_missing_toolchain_and_passes_when_available() -> Result<()> {
    let repo = isolated_rust_repo("tier-three")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;

    let migration = compose_plan_migration(&repo.root, &plans, LIB_REL)?;
    assert!(!migration.rewrites.is_empty(), "the neutral discovery finds the consumer before apply (m23 idiom)");
    let registry = default_registry();
    let staged = stage_plan(&repo.root, plan_for(&plans, LIB_REL), &registry)?;
    TransactionApplier::apply_with(&repo.root, &staged.extraction, &migration, staged.rewriter, staged.extractor)?;
    // The moved unit now exists on disk, so the existence walk stays honest
    // and the toolchain answer is what Skips the tier.

    let (missing_runner, missing_log) = ScriptedProbeRunner::all_none();
    let missing_validator = RustToolchainValidator::with_runner(Box::new(missing_runner));
    let missing_tier = TierValidator::validate_affected_tus(&migration, &repo.root, Some(&missing_validator));
    assert_eq!(serialized_status_label(&missing_tier.status), "Skipped", "the crates/validate Missing -> Skipped mapping, consumed not re-implemented (F6)");
    assert_eq!(
        recorded_calls(&missing_log),
        vec!["cargo --version".to_string(), "rustc --version".to_string()],
        "INV-10: the probe walk spawns nothing beyond the `--version` probes"
    );

    let (available_runner, available_log) = ScriptedProbeRunner::cargo_available();
    let available_validator = RustToolchainValidator::with_runner(Box::new(available_runner));
    let available_tier =
        TierValidator::validate_affected_tus(&migration, &repo.root, Some(&available_validator));
    assert_eq!(
        serialized_status_label(&available_tier.status),
        "Pass",
        "counter-leg: an Available toolchain keeps the tier an honest Pass (non-vacuity)"
    );
    assert_eq!(
        recorded_calls(&available_log),
        vec!["cargo --version".to_string()],
        "the available answer comes from the first probe alone"
    );
    Ok(())
}

// --- 11. F5 (EXACT reserved name): the flat-id shared fold is
// planning-frozen.

#[test]
fn flat_id_shared_mod_and_fn_record_is_planning_frozen() -> Result<()> {
    let repo = isolated_rust_repo("f5")?;
    add_temp_consumer(&repo.root)?;
    let model = reduced_model(&repo.root)?;

    // Leg 1: EXACTLY ONE record under `crate::greeting`, two declarations,
    // the declaring_files pair. The folded kind/defining_file stay UNPINNED
    // (fold-order-dependent bytes are not pinned, honesty over completeness).
    let greeting_records: Vec<&ReducedSymbolRecord> = model
        .symbols
        .values()
        .filter(|record| record.id.0 == "crate::greeting")
        .collect();
    assert_eq!(greeting_records.len(), 1, "the fold answers exactly one record");
    let greeting = greeting_records[0];
    assert_eq!(greeting.declarations.len(), 2, "the fn decl and the mod decl fold together");
    let declaration_ids: Vec<String> = greeting.declarations.iter().map(|decl| decl.0.clone()).collect();
    assert!(
        declaration_ids
            .iter()
            .any(|decl| decl.ends_with("#L4") && decl.starts_with("crate::greeting")),
        "the fn declaration rides line 4 of src/greeting.rs: {declaration_ids:?}"
    );
    assert!(
        declaration_ids
            .iter()
            .any(|decl| decl.ends_with("#L6") && decl.starts_with("crate::greeting")),
        "the mod declaration rides line 6 of src/lib.rs: {declaration_ids:?}"
    );
    let pair = sorted_file_ids(&[FileId::from_relative_path(GREETING_REL), FileId::from_relative_path(LIB_REL)]);
    assert_eq!(sorted_file_ids(&greeting.declaring_files), pair, "declaring_files is the sorted pair");

    // Leg 2: the wired plans carry no `crate::greeting` proposal and no
    // src/greeting.rs target.
    let plans = plan_extractions(&repo.root)?;
    assert_freeze_zero_legs(&repo.root, &plans)?;

    // Leg 3: the same >=2-declaration freeze on the suite-authored twin pair.
    assert_twin_fold_is_planning_frozen(&repo)?;
    Ok(())
}

/// F5 leg 3: the suite-authored twin pair folds into ONE `crate::twin`
/// record with provably TWO declarations; the constraints seam freezes it
/// (with the greeting fold and the Module-kind records) and leaves the
/// plannable fns unfrozen; the aggregation hook still qualifies the twin
/// candidate; and no wired plan proposes anything for `crate::twin`.
fn assert_twin_fold_is_planning_frozen(repo: &IsolatedRustRepo) -> Result<()> {
    fs::write(repo.root.join(TWIN_A_REL), TWIN_A_SOURCE)?;
    fs::write(repo.root.join(TWIN_B_REL), TWIN_B_SOURCE)?;
    let twin_model = reduced_model(&repo.root)?;
    let twin: Vec<&ReducedSymbolRecord> = twin_model
        .symbols
        .values()
        .filter(|record| record.id.0 == "crate::twin")
        .collect();
    assert_eq!(twin.len(), 1, "both temp files fold into ONE crate::twin record");
    assert_eq!(twin[0].declarations.len(), 2, "two written fns -> provably-ambiguous fold");
    let twin_pair = sorted_file_ids(&[FileId::from_relative_path(TWIN_A_REL), FileId::from_relative_path(TWIN_B_REL)]);
    assert_eq!(
        sorted_file_ids(&twin[0].declaring_files),
        twin_pair,
        "the twin fold's declaring_files name both temp files"
    );

    let frozen = RustClusteringConstraints.frozen_symbols(&model_records_as_lang_api(&twin_model));
    let frozen_ids: Vec<String> = frozen.iter().map(|id| id.0.clone()).collect();
    assert!(
        frozen_ids.iter().any(|id| id == "crate::twin"),
        "the constraints seam freezes the ambiguous twin fold: {frozen_ids:?}"
    );
    assert!(frozen_ids.iter().any(|id| id == "crate::greeting"), "the constraints seam freezes the fixture fold");
    assert!(frozen_ids.iter().any(|id| id == "crate::models"), "the seam freezes Module-kind records");
    assert!(!frozen_ids.iter().any(|id| id == "crate::fixture_marker"), "the plannable fn is NOT frozen");
    assert!(!frozen_ids.iter().any(|id| id == "crate::models_marker"), "the plannable models-side fn is NOT frozen");

    let registry = default_registry();
    let plugin = registry.plugin_for_id(LanguageId::Rust).expect("Rust registered");
    assert_eq!(
        plugin.aggregation_filter(&repo.root, Path::new(TWIN_A_REL)),
        Some(true),
        "twin_a is an aggregation candidate that lists crate::twin"
    );
    let twin_copy_plans = plan_extractions(&repo.root)?;
    let twin_proposals: Vec<&ProposedExtraction> =
        twin_copy_plans.iter().flat_map(|twin_plan| twin_plan.proposed_extractions.iter()).collect();
    for proposal in twin_proposals {
        assert_ne!(proposal.symbol_id.0, "crate::twin", "any candidate listing crate::twin proposes nothing for it");
    }
    Ok(())
}
