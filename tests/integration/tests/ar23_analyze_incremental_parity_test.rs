//! AR-23 parity lock: the cli incremental analyze path must behave
//! IDENTICALLY before and after it switched from the hand-rolled
//! locate/discover/shaper/restrict mirror to `Pipeline::for_incremental`.
//! Every expected value here is transcribed VERBATIM from the pre-change
//! captures in `tasks/ar-23-handoff-journal.md` (C1 cold full run, C2 warm
//! full run, C3 filtered full run on a warm copy, C4 pristine filtered run),
//! taken from real `tool analyze` binary runs on a cold copy of
//! `tests/fixtures/basic` before the switch landed. This suite passed
//! against the UNMODIFIED mirror first (proving the capture is authentic),
//! and the identical expectations must hold after the switch.
//!
//! Honest surface (journal-stated, not faked into the lock): the reuse
//! lines' and cache-rate line's DATA (`stats.reused_tus`,
//! `stats.reanalyzed_tus`, `stats.hit_rate`) are locked to the captured
//! numbers; the surrounding println text lives in `analyze.rs` untouched by
//! the switch and is locked by the verbatim pre/post binary-run stdout
//! comparison in the journal. The `Restricted run: ...` line's BRANCH
//! (non-authoritative persist skip) is locked observably: a filtered run
//! may write shard-cache files but never authority artifacts. Per AR-22
//! Finding 2 the observable is the artifact FILE SET, never directory
//! existence; per AR-22 Finding 3 no fps-value parity is claimed.

use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

mod common;
use common::fixture_path;

use ariadne_cli::{run_incremental_analysis, run_incremental_analysis_filtered};
use ariadne_repo::SourceLanguage;

/// Journal C1: the 15 artifact files a cold full (authoritative) run writes.
fn captured_authoritative_artifact_set() -> Vec<String> {
    [
        "index/index.json",
        "index/manifest.json",
        "index/tu_analyses.bin",
        "reduced/edges.bin",
        "reduced/extends-edges.bin",
        "reduced/file-symbols.bin",
        "reduced/manifest.json",
        "reduced/sealed-symbols.bin",
        "reduced/symbols.bin",
        "reduced/tu-usage.bin",
        "repository/files.bin",
        "repository/files.json",
        "repository/manifest.json",
        "shards/12388715314208539073.json",
        "shards/5683510519367502559.json",
    ]
    .iter()
    .map(|rel| rel.to_string())
    .collect()
}

/// Journal C4: a pristine filtered run writes ONLY the shard cache (the
/// authority gate skips repository/index/reduced persistence entirely).
fn captured_shard_cache_only_set() -> Vec<String> {
    [
        "shards/12388715314208539073.json",
        "shards/5683510519367502559.json",
    ]
    .iter()
    .map(|rel| rel.to_string())
    .collect()
}

/// Cold boot copy: the fixture's tracked pre-built `artifacts/` tree is
/// skipped so every run starts from an empty cache (same seam as the m21
/// boot lock; the tracked fixture is never modified).
fn provision_cold_basic_repo_copy() -> Result<(TempDir, PathBuf)> {
    let guard = tempdir()?;
    let repo_root = guard.path().join("basic");
    copy_dir_excluding_dir(&fixture_path("basic"), &repo_root, "artifacts")?;
    Ok((guard, repo_root))
}

fn copy_dir_excluding_dir(src: &Path, dst: &Path, skip_dir_name: &str) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if entry.file_type()?.is_dir() && name == skip_dir_name {
            continue;
        }
        let target = dst.join(name);
        if entry.file_type()?.is_dir() {
            copy_dir_excluding_dir(&entry.path(), &target, skip_dir_name)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// The artifact FILE SET (journal Finding-2 discipline: files, never
/// directory existence) as sorted repo-relative slash-separated paths.
fn artifact_file_set(repo_root: &Path) -> Vec<String> {
    let root = repo_root.join("artifacts");
    let mut files: Vec<String> = Vec::new();
    if root.exists() {
        collect_relative_files(&root, &root, &mut files);
        files.sort();
    }
    files
}

fn collect_relative_files(root: &Path, dir: &Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(dir).expect("artifact dir readable") {
        let entry = entry.expect("artifact entry readable");
        let path = entry.path();
        if path.is_dir() {
            collect_relative_files(root, &path, files);
        } else {
            let relative = path.strip_prefix(root).expect("path under artifacts root");
            files.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}

/// File set plus sizes - catches any silent rewrite of an existing artifact.
fn artifact_file_sizes(repo_root: &Path) -> BTreeMap<String, u64> {
    let mut sizes = BTreeMap::new();
    for relative in artifact_file_set(repo_root) {
        let len = fs::metadata(repo_root.join("artifacts").join(&relative))
            .expect("captured artifact readable")
            .len();
        sizes.insert(relative, len);
    }
    sizes
}

/// Journal C1: cold full run reuses 0, reanalyzes 2, hits 0.0%, and writes
/// the 15-file authoritative artifact set.
#[test]
fn cold_full_run_matches_captured_reuse_line_data_and_artifact_set() -> Result<()> {
    let (_guard, repo_root) = provision_cold_basic_repo_copy()?;
    let result = run_incremental_analysis(&repo_root)?;
    assert_eq!(
        result.stats.reused_tus, 0,
        "journal C1 reuse line: 'Reused: 0 translation units'"
    );
    assert_eq!(
        result.stats.reanalyzed_tus, 2,
        "journal C1 reanalyzed line: 'Reanalyzed: 2 translation units'"
    );
    assert_eq!(result.stats.hit_rate, 0.0, "journal C1 cache hit rate 0.0%");
    assert_eq!(
        artifact_file_set(&repo_root),
        captured_authoritative_artifact_set(),
        "journal C1 authoritative artifact file set"
    );
    Ok(())
}

/// Journal C2: the second run reuses everything (the warm-path reuse lines
/// 2/0/100.0%) and leaves the artifact file set identical.
#[test]
fn warm_full_run_matches_captured_reuse_line_data_and_unchanged_set() -> Result<()> {
    let (_guard, repo_root) = provision_cold_basic_repo_copy()?;
    run_incremental_analysis(&repo_root)?;
    let warm = run_incremental_analysis(&repo_root)?;
    assert_eq!(
        warm.stats.reused_tus, 2,
        "journal C2 warm reuse line: 'Reused: 2 translation units'"
    );
    assert_eq!(
        warm.stats.reanalyzed_tus, 0,
        "journal C2 warm reanalyzed line: 'Reanalyzed: 0 translation units'"
    );
    assert_eq!(warm.stats.hit_rate, 100.0, "journal C2 cache hit rate 100.0%");
    assert_eq!(
        artifact_file_set(&repo_root),
        captured_authoritative_artifact_set(),
        "journal C2: warm run leaves the artifact file set unchanged"
    );
    Ok(())
}

/// Journal C4: a filtered run from a pristine copy reports the restricted
/// branch, analyzes the same 2 units cold, and persists ONLY the shard
/// cache - never any authority artifact.
#[test]
fn pristine_filtered_run_matches_captured_cold_stats_and_shard_only_set() -> Result<()> {
    let (_guard, repo_root) = provision_cold_basic_repo_copy()?;
    let result = run_incremental_analysis_filtered(&repo_root, Some(SourceLanguage::Cpp))?;
    assert_eq!(
        result.stats.reused_tus, 0,
        "journal C4 pristine filtered reuse line: 'Reused: 0 translation units'"
    );
    assert_eq!(
        result.stats.reanalyzed_tus, 2,
        "journal C4 pristine filtered reanalyzed line: 'Reanalyzed: 2 translation units'"
    );
    assert_eq!(result.stats.hit_rate, 0.0, "journal C4 cache hit rate 0.0%");
    assert_eq!(
        artifact_file_set(&repo_root),
        captured_shard_cache_only_set(),
        "journal C4: the restricted branch writes shard cache only, no authority artifacts"
    );
    Ok(())
}

/// Journal C3: a filtered run on a warm copy keeps the warm reuse lines
/// (2/0/100.0%) and touches nothing - the full artifact set survives with
/// identical sizes.
#[test]
fn filtered_run_after_full_run_matches_captured_warm_lines_and_persists_nothing() -> Result<()> {
    let (_guard, repo_root) = provision_cold_basic_repo_copy()?;
    run_incremental_analysis(&repo_root)?;
    let artifacts_before_full_run_snapshot = artifact_file_sizes(&repo_root);
    let warm_filtered =
        run_incremental_analysis_filtered(&repo_root, Some(SourceLanguage::Cpp))?;
    assert_eq!(
        warm_filtered.stats.reused_tus, 2,
        "journal C3 warm filtered reuse line: 'Reused: 2 translation units'"
    );
    assert_eq!(
        warm_filtered.stats.reanalyzed_tus, 0,
        "journal C3 warm filtered reanalyzed line: 'Reanalyzed: 0 translation units'"
    );
    assert_eq!(
        warm_filtered.stats.hit_rate,
        100.0,
        "journal C3 warm filtered cache hit rate 100.0%"
    );
    assert_eq!(
        artifact_file_sizes(&repo_root),
        artifacts_before_full_run_snapshot,
        "journal C3: the restricted run persists nothing new over the full-run artifacts"
    );
    Ok(())
}
