//! Real-fixture byte-identity lock for the planning fast path (load-tail A1/A2).
//!
//! `m00_golden_baseline_test` pins the planner's COLD path: it drives
//! `plan_extractions`, which builds the planner with a cold `LoadAnalyzer` and
//! writes `artifacts/plans/plans.json`. Nothing in the golden suite exercises
//! the UI's screening-facts path, so a divergence between the two screening
//! sources would only ever show up as a UI/CLI disagreement at runtime.
//!
//! This suite closes that gap on REAL C++ input: it runs the same production
//! pipeline, then rebuilds the planner over the identical graph, model,
//! records and translation units with screening facts taken from the load
//! stage's `file_metrics` (and include closures taken from the include stage),
//! and asserts the resulting plans are byte-identical to the bytes production
//! wrote. Synthetic-graph parity lives in
//! `crates/cluster/tests/planner_facts_parity.rs`; this is the real-input
//! counterpart.
//!
//! Red/green contract: this test fails to compile until the fast-path planner
//! surface exists (`with_screening_facts` + `screening_facts_from_metrics`).
//! It must never be weakened to compare a filtered or reordered subset -- the
//! value of the lock is the full byte stream, plan ids and ordering included.

mod common;

use anyhow::{Context, Result};
use ariadne_build::BuildDiscoveryResult;
use ariadne_cluster::plan::OptimizationPlan;
use ariadne_cluster::planner::{screening_facts_from_metrics, OptimizationPlanner};
use ariadne_lang::CyclePolicy;
use ariadne_lang_cpp::{cpp_aggregation_filter, cpp_target_path_for};
use ariadne_pipeline::Pipeline;
use ariadne_repo::FileId;
use std::fs;
use std::path::{Path, PathBuf};

use common::fixture_path;

/// Translation-unit file ids, matching the production derivation in
/// `crates/cli/src/commands/ui.rs` (`translation_unit_file_ids`).
fn translation_unit_file_ids(build: &BuildDiscoveryResult) -> Vec<FileId> {
    build
        .translation_units
        .iter()
        .map(|unit| unit.file_id)
        .collect()
}

/// Copy the fixture into a temp dir with no prebuilt artifacts, so every stage
/// rebuilds and binds its fingerprints to the copy root (mirrors
/// `m00_golden_baseline_test::setup_isolated_repo`).
fn isolated_fixture(fixture: &str) -> Result<(tempfile::TempDir, PathBuf)> {
    let temp_dir = tempfile::tempdir().context("failed to create temp dir")?;
    let repo_path = temp_dir.path().join(fixture);
    copy_dir_filtered(&fixture_path(fixture), &repo_path)?;
    for stale in ["artifacts", ".ariadne"] {
        let _ = fs::remove_dir_all(repo_path.join(stale));
    }
    Ok((temp_dir, repo_path))
}

fn copy_dir_filtered(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;
    for entry in fs::read_dir(source)
        .with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_filtered(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)
                .with_context(|| format!("failed to copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}

#[test]
fn facts_path_reproduces_production_plans_basic() -> Result<()> {
    assert_facts_path_matches_production("basic")
}

#[test]
fn facts_path_reproduces_production_plans_expensive() -> Result<()> {
    assert_facts_path_matches_production("expensive")
}

fn assert_facts_path_matches_production(fixture: &str) -> Result<()> {
    let (_temp_dir, repo_path) = isolated_fixture(fixture)?;

    ariadne_cli::discover_repository_files(&repo_path)?;
    ariadne_cli::discover_build_contexts(&repo_path)?;
    let mut pipeline = Pipeline::new(&repo_path, None)?;
    pipeline.get_or_build_include_graph()?;
    ariadne_cli::build_source_index(&repo_path)?;
    ariadne_cli::get_or_build_reduced_model(&repo_path)?;
    ariadne_cli::analyze_dependency_load(&repo_path, None, None)?;
    ariadne_cli::plan_extractions(&repo_path)?;

    let production_path = repo_path.join("artifacts").join("plans").join("plans.json");
    let production_bytes = fs::read(&production_path)
        .with_context(|| format!("{fixture}: missing {}", production_path.display()))?;
    let production_plans: Vec<OptimizationPlan> = serde_json::from_slice(&production_bytes)
        .with_context(|| format!("{fixture}: plans.json is not a Vec<OptimizationPlan>"))?;

    let (include_graph, closures) = pipeline.get_or_build_include_graph()?;
    let reduced_model = pipeline.get_or_build_reduced()?;
    let load_report = pipeline.get_or_build_load()?;
    let tu_file_ids = translation_unit_file_ids(&pipeline.build);

    let facts_plans = OptimizationPlanner::with_screening_facts(
        &include_graph,
        &reduced_model,
        &pipeline.discovery.files,
        &tu_file_ids,
        &cpp_aggregation_filter,
        &cpp_target_path_for,
        None,
        CyclePolicy::Reject,
        screening_facts_from_metrics(&include_graph, &load_report.file_metrics),
        &closures,
    )
    .plan();

    assert_eq!(
        serde_json::to_string_pretty(&facts_plans)?,
        String::from_utf8(production_bytes.clone())
            .context("production plans.json is not valid UTF-8")?,
        "{fixture}: fast-path plans must be byte-identical to the production plans.json"
    );
    assert_eq!(
        facts_plans, production_plans,
        "{fixture}: fast-path plans must equal the production plans elementwise"
    );
    Ok(())
}