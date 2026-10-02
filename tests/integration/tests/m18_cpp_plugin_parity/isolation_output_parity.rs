//! Section 3 — isolation output parity (plan §5 M18 acceptance, third
//! clause).
//!
//! The neutral `IsolationEngine` + `CppSourceExtractor` + `CppDependencyRewriter`
//! must reproduce the bytes the transform pipeline produced before the
//! deletions: every generated header (placeholder bodies included), the
//! `MigrationPreview` JSON, and the `format_diff_with` diff text.
//!
//! The locks below were captured at HEAD (ceac55f) from the pre-deletion
//! code by running, on throwaway fixture copies:
//! registry `handle_index_with_registry` → `ReducedSemanticModel::reduce` →
//! `build_dependency_graph` → `OptimizationPlanner` (C++ hooks, conservative
//! plans) → `IsolationEngine::extract` → `write_to_disk(&CppSourceExtractor)`,
//! plus the `IncludeRewriter::build_preview` JSON and
//! `preview.format_diff_with(&CppDependencyRewriter)` text.
//!
//! Every captured string is stored through the golden convention's
//! separator normalization ([`super::lockable`], `\\` → `/`), so Windows
//! path rendering inside dependency strings does not platform-lock the
//! suite; everything else is byte-exact. `production_write_path_reproduces_
//! the_same_bytes` additionally cross-checks the engine+extractor bytes
//! against the production transactional write (validator apply) on the same
//! plan, so the lock cannot drift between the two emitters.

use super::{
    isolated_repo, lockable, normalized_rel, repo_inputs, IsolatedRepo,
};
use anyhow::{Context, Result};
use ariadne_build::BuildDiscoveryResult;
use ariadne_cli::handle_index_with_registry;
use ariadne_cluster::{
    plan::{OptimizationPlan, RiskLevel},
    planner::OptimizationPlanner,
};
use ariadne_graph::{build_dependency_graph, ReducedSemanticModel};
use ariadne_index::IndexResult;
use ariadne_lang::CyclePolicy;
use ariadne_lang_cpp::{
    cpp_aggregation_filter, cpp_target_path_for, CppDependencyRewriter, CppSourceExtractor,
};
use ariadne_repo::RepositoryDiscoveryResult;
use ariadne_transform::{ExtractionResult, GeneratedUnit, IncludeRewriter, IsolationEngine};
use ariadne_validate::TransactionApplier;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// One plan's locked isolation output: id, generated file bytes (path →
/// content, both locked normalized), the `MigrationPreview` pretty JSON, and
/// the plugin-rendered diff text. Captured at ceac55f from the pre-deletion
/// code on the `basic` fixture (1 conservative plan).
const BASIC_PLAN_LOCKS: &[PlanLock] = &[PlanLock {
    plan_id: "PLAN-001",
    generated: &[
        (
            "include/ThingConfig_extracted.h",
            "#pragma once\n\n// Symbol: ThingConfig (Struct)\n",
        ),
        (
            "include/Thing_Thing_extracted.h",
            "#pragma once\n\n#include \"include/ThingConfig_extracted.h\"\n#include \"include/Thing_extracted.h\"\n\n// Symbol: Thing::Thing (Function)\n",
        ),
        (
            "include/Thing_extracted.h",
            "#pragma once\n\n#include \"include/ThingConfig_extracted.h\"\n\n// Symbol: Thing (Class)\n",
        ),
        (
            "include/Thing_get_id_extracted.h",
            "#pragma once\n\n// Symbol: Thing::get_id (Function)\n",
        ),
    ],
    preview_json: "{\n  \"plan_id\": \"PLAN-001\",\n  \"rewrites\": [\n    {\n      \"consumer_file\": \"<ROOT>/src/thing.cpp\",\n      \"new_includes\": [\n        \"generated/include/ThingConfig_extracted.h\",\n        \"generated/include/Thing_Thing_extracted.h\",\n        \"generated/include/Thing_extracted.h\",\n        \"generated/include/Thing_get_id_extracted.h\"\n      ],\n      \"old_includes\": [\n        \"include/thing.h\"\n      ],\n      \"side_effect_preservations\": []\n    },\n    {\n      \"consumer_file\": \"<ROOT>/tests/consumer.cpp\",\n      \"new_includes\": [\n        \"generated/include/ThingConfig_extracted.h\",\n        \"generated/include/Thing_Thing_extracted.h\",\n        \"generated/include/Thing_extracted.h\",\n        \"generated/include/Thing_get_id_extracted.h\"\n      ],\n      \"old_includes\": [\n        \"include/thing.h\"\n      ],\n      \"side_effect_preservations\": []\n    }\n  ]\n}",
    diff_text: "<ROOT>/src/thing.cpp\n    - #include \"include/thing.h\"\n    + #include \"generated/include/ThingConfig_extracted.h\"\n    + #include \"generated/include/Thing_Thing_extracted.h\"\n    + #include \"generated/include/Thing_extracted.h\"\n    + #include \"generated/include/Thing_get_id_extracted.h\"\n\n<ROOT>/tests/consumer.cpp\n    - #include \"include/thing.h\"\n    + #include \"generated/include/ThingConfig_extracted.h\"\n    + #include \"generated/include/Thing_Thing_extracted.h\"\n    + #include \"generated/include/Thing_extracted.h\"\n    + #include \"generated/include/Thing_get_id_extracted.h\"\n\n",
}];

/// Same capture for the `expensive` fixture.
const EXPENSIVE_PLAN_LOCKS: &[PlanLock] = &[PlanLock {
    plan_id: "PLAN-001",
    generated: &[
        (
            "include/Thing_extracted.h",
            "#pragma once\n\n#include \"include/heavy_func_extracted.h\"\n\n// Symbol: Thing (Class)\n",
        ),
        (
            "include/Thing_use_heavy_extracted.h",
            "#pragma once\n\n// Symbol: Thing::use_heavy (Function)\n",
        ),
        (
            "include/heavy_func_extracted.h",
            "#pragma once\n\n// Symbol: heavy_func (Function)\n",
        ),
    ],
    preview_json: "{\n  \"plan_id\": \"PLAN-001\",\n  \"rewrites\": []\n}",
    diff_text: "",
}];

/// One plan's embedded capture (see the module docs for the capture run).
pub struct PlanLock {
    pub plan_id: &'static str,
    pub generated: &'static [(&'static str, &'static str)],
    pub preview_json: &'static str,
    pub diff_text: &'static str,
}

struct FixtureHarness {
    repo: IsolatedRepo,
    discovery: RepositoryDiscoveryResult,
    build: BuildDiscoveryResult,
    model: ReducedSemanticModel,
}

/// The plan set + model one production `optimize` run would act on:
/// registry index → reduce → registry graph → conservative planner output.
fn fixture_harness(fixture: &str, label: &str) -> Result<FixtureHarness> {
    let repo = isolated_repo(fixture, label)?;
    let (discovery, build) = repo_inputs(&repo.root)?;
    let index: IndexResult = handle_index_with_registry(
        &repo.root,
        Arc::new(ariadne_cli::registry::default_registry()),
    )
    .with_context(|| format!("{fixture}: registry index failed"))?;
    let model = ReducedSemanticModel::reduce(&index);
    Ok(FixtureHarness {
        repo,
        discovery,
        build,
        model,
    })
}

fn conservative_plans(harness: &FixtureHarness) -> Result<Vec<OptimizationPlan>> {
    let registry = ariadne_cli::registry::default_registry();
    let graph = build_dependency_graph(&harness.repo.root, &registry)?;
    let tu_file_ids: Vec<ariadne_repo::FileId> = harness
        .build
        .translation_units
        .iter()
        .map(|tu| tu.file_id)
        .collect();
    let planner = OptimizationPlanner::new(
        &graph,
        &harness.model,
        &harness.discovery.files,
        &tu_file_ids,
        &cpp_aggregation_filter,
        &cpp_target_path_for,
        None,
        CyclePolicy::Reject,
    );
    Ok(planner
        .plan()
        .into_iter()
        .filter(|plan| matches!(plan.risk, RiskLevel::Conservative))
        .collect())
}

/// One plan's live isolation output, computed the same way the capture run
/// produced the embedded locks. Every field is stored in its LOCKED form:
/// paths normalized to forward slashes, and the per-run temp root replaced
/// with the golden convention's `<ROOT>` placeholder (the production
/// `MigrationPreview` carries the TU's absolute canonical path, which
/// cannot be embedded verbatim into a permanent suite).
struct PlanCapture {
    plan_id: String,
    generated: BTreeMap<String, String>,
    preview_json: String,
    diff_text: String,
}

/// Content provider over the isolated fixture copy (M23 T1, D-M23-3): reads
/// each extraction's `source_file` through the discovery catalog; unreadable
/// files yield `None` and keep the engine's placeholder fallback (I4).
fn harness_source_reader(
    harness: &FixtureHarness,
) -> impl Fn(ariadne_repo::FileId) -> Option<String> {
    let path_by_id: BTreeMap<ariadne_repo::FileId, PathBuf> = harness
        .discovery
        .files
        .iter()
        .map(|record| (record.id, record.path.clone()))
        .collect();
    let repo_root = harness.repo.root.clone();
    move |file_id| {
        let relative_path = path_by_id.get(&file_id)?;
        std::fs::read_to_string(repo_root.join(relative_path)).ok()
    }
}

fn capture_plan(harness: &FixtureHarness, plan: &OptimizationPlan) -> Result<PlanCapture> {
    let source_reader = harness_source_reader(harness);
    let extraction =
        IsolationEngine::new(&harness.model).extract(plan, &source_reader, &CppSourceExtractor);
    let output_dir = tempfile::tempdir()?;
    extraction.write_to_disk(output_dir.path(), &CppSourceExtractor)?;
    let generated = read_text_tree(output_dir.path())?
        .into_iter()
        .map(|(path, content)| (path, lockable(&content)))
        .collect();

    let (symbol_name_to_header, relocated) = relocate_to_generated(&extraction);
    let consumers = consumers_including_plan_target(harness, plan);
    let rewriter = IncludeRewriter::new(symbol_name_to_header, relocated);
    let preview = rewriter.build_preview(plan.id.to_string(), &consumers);
    Ok(PlanCapture {
        plan_id: plan.id.to_string(),
        generated,
        preview_json: lock_preview_json(&preview, &harness.repo.root)?,
        diff_text: redact_run_root(
            &preview.format_diff_with(&CppDependencyRewriter),
            &harness.repo.root,
        ),
    })
}

/// `MigrationPreview` JSON in its locked form: the golden normalizer
/// (multiset-sorted arrays, every run root → `<ROOT>`, forward slashes)
/// applied to the pretty JSON, so both capture and assertion feed identical
/// pipelines and the lock is platform- and temp-path-independent.
fn lock_preview_json(preview: &ariadne_transform::MigrationPreview, root: &Path) -> Result<String> {
    let value = serde_json::to_value(preview)?;
    let normalized = crate::common::normalize_json_with_roots(value, &[root.to_path_buf()]);
    Ok(serde_json::to_string_pretty(&normalized)?)
}

/// Replaces every platform rendering of the run root (canonical verbatim
/// first, then the raw form — longest first) in a lockable()-normalized
/// string with `<ROOT>`, mirroring golden rule 3 for raw text.
fn redact_run_root(text: &str, root: &Path) -> String {
    let canonical = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let mut forms = vec![canonical.display().to_string(), root.display().to_string()];
    forms.sort_by_key(|form| std::cmp::Reverse(form.len()));
    let mut redacted = lockable(text);
    for form in forms {
        redacted = redacted.replace(&lockable(&form), "<ROOT>");
    }
    redacted
}

fn assert_plan_capture_matches_lock(fixture: &str, capture: &PlanCapture, lock: &PlanLock) {
    assert_eq!(
        capture.plan_id, lock.plan_id,
        "{fixture}: plan id sequence drifted"
    );
    let expected: BTreeMap<String, String> = lock
        .generated
        .iter()
        .map(|(path, content)| (path.to_string(), content.to_string()))
        .collect();
    assert_eq!(
        expected, capture.generated,
        "{fixture}:{}: generated header bytes drifted (paths normalized to \'/\')",
        capture.plan_id
    );
    assert_eq!(
        lock.preview_json, capture.preview_json,
        "{fixture}:{}: MigrationPreview JSON drifted",
        capture.plan_id
    );
    assert_eq!(
        lock.diff_text, capture.diff_text,
        "{fixture}:{}: plugin-rendered diff text drifted",
        capture.plan_id
    );
}

#[test]
fn isolation_output_matches_vendored_bytes_basic() -> Result<()> {
    plan_lock_parity("basic", BASIC_PLAN_LOCKS)
}

#[test]
fn isolation_output_matches_vendored_bytes_expensive() -> Result<()> {
    plan_lock_parity("expensive", EXPENSIVE_PLAN_LOCKS)
}

fn plan_lock_parity(fixture: &str, locks: &[PlanLock]) -> Result<()> {
    let harness = fixture_harness(fixture, "isolation")?;
    let plans = conservative_plans(&harness)?;
    assert_eq!(
        plans.len(),
        locks.len(),
        "{fixture}: conservative plan count drifted from the ceac55f capture"
    );
    for (plan, lock) in plans.iter().zip(locks) {
        let capture = capture_plan(&harness, plan)?;
        assert_plan_capture_matches_lock(fixture, &capture, lock);
    }
    Ok(())
}

#[test]
fn production_write_path_reproduces_the_same_bytes_basic() -> Result<()> {
    // The engine+extractor capture must also equal what the production
    // transactional apply (validate crate, plugin rewriter) puts on disk —
    // both emitters locked against each other on a fresh fixture copy.
    let harness = fixture_harness("basic", "isolation-apply")?;
    let plans = conservative_plans(&harness)?;
    assert!(!plans.is_empty(), "basic fixture must produce a plan");
    for plan in &plans {
        let source_reader = harness_source_reader(&harness);
        let extraction = IsolationEngine::new(&harness.model)
            .extract(plan, &source_reader, &CppSourceExtractor);
        let output_dir = tempfile::tempdir()?;
        extraction.write_to_disk(output_dir.path(), &CppSourceExtractor)?;

        let (symbol_name_to_header, relocated) = relocate_to_generated(&extraction);
        let consumers = consumers_including_plan_target(&harness, plan);
        let preview =
            IncludeRewriter::new(symbol_name_to_header, relocated).build_preview(
                plan.id.to_string(),
                &consumers,
            );
        let (_snapshot, _new_paths) = TransactionApplier::apply_with(
            &harness.repo.root,
            &extraction,
            &preview,
            &CppDependencyRewriter,
        )?;
        for unit in &extraction.headers {
            let produced = std::fs::read_to_string(
                harness.repo.root.join("generated").join(&unit.path),
            )?;
            let engine_bytes = std::fs::read_to_string(output_dir.path().join(&unit.path))?;
            assert_eq!(
                engine_bytes,
                produced,
                "{}: production apply wrote different bytes than engine+extractor",
                normalized_rel(&unit.path)
            );
        }
    }
    Ok(())
}

/// Local copy of `optimize.rs`'s private relocation step (plan target headers
/// hang under `generated/`): keeps this permanent lock independent of CLI
/// internals. Mirrors `git show ceac55f:crates/cli/src/optimize.rs`.
fn relocate_to_generated(
    extraction: &ExtractionResult,
) -> (BTreeMap<String, PathBuf>, Vec<GeneratedUnit>) {
    let mut symbol_name_to_header = BTreeMap::new();
    let mut generated_units = Vec::new();
    for unit in &extraction.headers {
        let relocated_path = PathBuf::from("generated").join(&unit.path);
        let mut relocated = unit.clone();
        relocated.path = relocated_path.clone();
        relocated.required_deps = unit
            .required_deps
            .iter()
            .map(|dep| PathBuf::from("generated").join(dep))
            .collect();
        generated_units.push(relocated);
        for symbol in &unit.symbols {
            symbol_name_to_header.insert(symbol.name.clone(), relocated_path.clone());
        }
    }
    (symbol_name_to_header, generated_units)
}

/// Local copy of `optimize.rs`'s private consumer scan: TUs whose source
/// text references the plan target header by its bare file name.
fn consumers_including_plan_target(
    harness: &FixtureHarness,
    plan: &OptimizationPlan,
) -> Vec<(PathBuf, Vec<PathBuf>, Vec<String>)> {
    let target_include = plan
        .target_file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let symbol_names: Vec<String> = plan
        .proposed_extractions
        .iter()
        .map(|extraction| extraction.symbol_name.clone())
        .collect();
    let mut consumers = Vec::new();
    for tu in &harness.build.translation_units {
        let Ok(content) = std::fs::read_to_string(harness.repo.root.join(&tu.source_path))
        else {
            continue;
        };
        if content.contains(&format!("#include \"{target_include}\""))
            || content.contains(&format!("#include <{target_include}>"))
        {
            consumers.push((
                tu.source_path.clone(),
                vec![plan.target_file.clone()],
                symbol_names.clone(),
            ));
        }
    }
    consumers
}

// (M23 T1, D-M23-4) `write_to_disk` now creates each unit's parent chain
// itself — the `ensure_unit_parents` mirror this file used before M23 is gone.

fn read_text_tree(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.insert(
                    normalized_rel(&path.strip_prefix(root)?),
                    std::fs::read_to_string(&path)?,
                );
            }
        }
    }
    Ok(files)
}

#[test]
fn zz_capture_isolation() -> Result<()> {
    for fixture in ["basic", "expensive"] {
        let harness = fixture_harness(fixture, "capture-iso")?;
        let plans = conservative_plans(&harness)?;
        for plan in &plans {
            let capture = capture_plan(&harness, plan)?;
            println!(
                "@@PLAN\t{}\t{}",
                fixture,
                serde_json::to_string(&capture.plan_id)?
            );
            for (path, content) in &capture.generated {
                println!(
                    "@@GEN\t{}\t{}\t{}\t{}",
                    fixture,
                    serde_json::to_string(&capture.plan_id)?,
                    serde_json::to_string(path)?,
                    serde_json::to_string(content)?
                );
            }
            println!(
                "@@PREVIEW\t{}\t{}\t{}",
                fixture,
                serde_json::to_string(&capture.plan_id)?,
                serde_json::to_string(&capture.preview_json)?
            );
            println!(
                "@@DIFF\t{}\t{}\t{}",
                fixture,
                serde_json::to_string(&capture.plan_id)?,
                serde_json::to_string(&capture.diff_text)?
            );
        }
    }
    Ok(())
}
