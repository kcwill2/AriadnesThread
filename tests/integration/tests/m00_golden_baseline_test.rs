//! M17.0 golden-artifact baseline (plan §6.1): runs the FULL C++ pipeline on
//! both fixtures through the same `ariadne_cli` handlers the existing suites
//! call, then compares every stage's production-written JSON against
//! `tests/golden/<fixture>/<artifact>.json`.
//!
//! The pipeline runs on a throwaway copy of the fixture so each invocation is
//! a cold, isolated run (fingerprints bind to the copy root, forcing rebuild
//! of every stage) and nothing is written into the tracked fixture dirs.
//! Generation mode: `UPDATE_GOLDENS=1 cargo test -p ariadne-integration-tests
//! --test m00_golden_baseline_test`.

mod common;

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// (golden artifact name, stage directory under artifacts/, JSON file the
/// production writer emits) — serialized by the production code paths in
/// `crates/report/src/manager/*`, never by an ad-hoc field list here.
const GOLDEN_ARTIFACTS: [(&str, &str, &str); 14] = [
    ("repository_manifest", "repository", "manifest.json"),
    ("repository_files", "repository", "files.json"),
    ("build_manifest", "contexts", "manifest.json"),
    ("build_contexts", "contexts", "contexts.json"),
    ("build_commands", "contexts", "commands.json"),
    ("includes_manifest", "includes", "manifest.json"),
    ("includes_report", "includes", "report.json"),
    ("index_manifest", "index", "manifest.json"),
    ("index_index", "index", "index.json"),
    ("reduced_manifest", "reduced", "manifest.json"),
    ("load_manifest", "load", "manifest.json"),
    ("load_report", "load", "report.json"),
    ("plans_manifest", "plans", "manifest.json"),
    ("plans_plans", "plans", "plans.json"),
];

#[test]
fn golden_baseline_basic() -> Result<()> {
    run_fixture_baseline("basic")
}

#[test]
fn golden_baseline_expensive() -> Result<()> {
    run_fixture_baseline("expensive")
}

fn run_fixture_baseline(fixture: &str) -> Result<()> {
    let (temp_dir, repo_path) = setup_isolated_repo(fixture)?;
    run_full_pipeline(&repo_path)?;

    let normalization_roots = vec![repo_path.clone(), temp_dir.path().to_path_buf()];
    for (artifact_name, stage_dir, json_file) in GOLDEN_ARTIFACTS {
        let artifact_path = repo_path.join("artifacts").join(stage_dir).join(json_file);
        let raw: Value = read_json(&artifact_path)
            .with_context(|| format!("{} stage artifact missing: {:?}", artifact_name, artifact_path))?;
        let normalized = common::normalize_json_with_roots(raw, &normalization_roots);
        common::assert_golden_matches(fixture, artifact_name, &normalized);
    }
    Ok(())
}

/// discover → build → graph → index → reduce → load → plan, via the exact
/// `ariadne_cli` handler entry points the m6/m7/m14 suites use. The graph
/// stage's persisted artifacts are written by the Pipeline when load/plan
/// rebuild it.
fn run_full_pipeline(repo_path: &Path) -> Result<()> {
    let repo_path_buf = repo_path.to_path_buf();
    ariadne_cli::discover_repository_files(&repo_path_buf)?;
    ariadne_cli::discover_build_contexts(&repo_path_buf)?;
    let mut pipeline = ariadne_pipeline::Pipeline::new(repo_path, None)?;
    pipeline.get_or_build_include_graph()?;
    ariadne_cli::build_source_index(&repo_path_buf)?;
    ariadne_cli::get_or_build_reduced_model(&repo_path_buf)?;
    ariadne_cli::analyze_dependency_load(&repo_path_buf, None, None)?;
    ariadne_cli::plan_extractions(&repo_path_buf)?;
    Ok(())
}

fn read_json(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {:?}", path))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse JSON in {:?}", path))
}

/// Copies `tests/fixtures/<name>` into a temp dir, skipping regenerable
/// caches, so the pipeline always runs cold and isolated.
fn setup_isolated_repo(fixture: &str) -> Result<(tempfile::TempDir, PathBuf)> {
    let temp_dir = tempfile::tempdir()?;
    let repo_path = temp_dir.path().join(fixture);
    copy_fixture(common::fixture_path(fixture), &repo_path)?;
    Ok((temp_dir, repo_path))
}

fn copy_fixture(src: PathBuf, dst: &Path) -> Result<()> {
    copy_dir_filtered(&src, dst)
        .with_context(|| format!("failed to copy fixture {:?} to {:?}", src, dst))
}

fn copy_dir_filtered(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_owned();
        if name_str == "artifacts" || name_str == ".ariadne" || name_str.starts_with('.') {
            continue;
        }
        let target = dst.join(name);
        if entry.file_type()?.is_dir() {
            copy_dir_filtered(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
