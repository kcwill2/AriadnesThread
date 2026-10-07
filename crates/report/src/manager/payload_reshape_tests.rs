//! M28 T4 payload-reshape contract locks (handoff `t4-payload-reshape`).
//!
//! The persisted load artifact drops the never-consumed per-symbol detail
//! (user DP-2 decision 2026-10-06) and the save/load paths stream instead of
//! materializing whole-payload byte strings (the ~4096 MiB save-span
//! transient, t0c measurement sec. 4). These tests state the behavior law:
//! - `report.json` carries the scalar summary + full file detail, NOT the
//!   `symbol_metrics` array (R1);
//! - `report.bin` round-trips the per-symbol scalars and the full
//!   `file_metrics` detail (R1 companion);
//! - a `2.1.0`-era manifest reaches `StaleArtifact` (rebuild), never a
//!   decode error - the epoch bump IS the invalidation (R2, shape sec. 3.4
//!   R6: bincode is strict, the fingerprint chain hashes content not code);
//! - the manifest totals stay identical to the summary-derived totals (R3,
//!   IV2 - a green-by-design pin, labeled as a pin in the journal, not a Red).

use super::{is_stale, ArtifactManager};
use super::tests::{fixture_load_report, manager_in_fresh_tempdir, FP};
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use crate::manifests::LoadReport;
use serde_json::{json, Value};
use std::fs;
use tempfile::tempdir;

/// Fixture totals (single source for the pins below): the shared
/// `fixture_load_report` carries 2 symbol rows (collateral 4 + 0) and 1
/// file row (collateral 1).
const FIXTURE_SYMBOL_ROWS: u64 = 2;
const FIXTURE_SYMBOL_COLLATERAL: u64 = 4;
const FIXTURE_FILE_ROWS: u64 = 1;
const FIXTURE_TOTAL_COLLATERAL: u64 = 5;

/// The shared fixture re-rooted at the temp manager: `check_manifest`
/// compares `repository_root` against the manager's repo, so any test that
/// takes the CHECKED load path must save a report rooted where it loads.
fn fixture_load_report_for(mgr: &ArtifactManager) -> LoadReport {
    let mut report = fixture_load_report();
    report.repository_root = mgr.repo_root();
    report
}

fn persisted_json(load_dir: &std::path::Path) -> Value {
    let content = fs::read_to_string(load_dir.join("report.json")).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn persisted_manifest(load_dir: &std::path::Path) -> Value {
    let content = fs::read_to_string(load_dir.join("manifest.json")).unwrap();
    serde_json::from_str(&content).unwrap()
}

/// R1 (Red carrier 1): the persisted `report.json` must NOT carry the
/// never-consumed per-symbol detail array, and must carry the summary
/// scalars plus the full consumed file detail.
#[test]
fn persisted_json_drops_symbol_detail_and_carries_summary_scalars() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let report = fixture_load_report();
    let load_dir = mgr.save_load_artifacts(&report, FP).unwrap();

    let persisted = persisted_json(&load_dir);
    assert!(
        persisted.get("symbol_metrics").is_none(),
        "persisted report.json must not carry the never-consumed per-symbol detail: found key symbol_metrics"
    );
    let summary = persisted
        .get("symbol_summary")
        .expect("persisted report.json must carry the symbol_summary scalars");
    assert_eq!(
        summary["analyzed_symbols"].as_u64(),
        Some(FIXTURE_SYMBOL_ROWS),
        "symbol_summary.analyzed_symbols must equal the saved symbol count"
    );
    assert_eq!(
        summary["collateral_load"].as_u64(),
        Some(FIXTURE_SYMBOL_COLLATERAL),
        "symbol_summary.collateral_load must equal the saved symbol collateral sum"
    );
    let files = persisted["file_metrics"]
        .as_array()
        .expect("persisted report.json must carry the full file_metrics detail");
    assert_eq!(files.len(), report.file_metrics.len());
    assert_eq!(
        files[0]["collateral_symbols"]
            .as_array()
            .expect("file rows keep their detail in the persisted json")
            .len(),
        1,
        "the UI-consumed file detail must survive the payload reshape"
    );
}

/// R1 companion: the bin side round-trips the per-symbol SCALARS (name,
/// target file, the four load numbers) and the full `file_metrics` detail
/// through `load_load_checked`. Green-by-design at Red (today's loaded rows
/// are the full rows); after the reshape it locks the scalar survival.
#[test]
fn loaded_artifact_round_trips_summary_scalars_and_full_file_detail() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let report = fixture_load_report_for(&mgr);
    mgr.save_load_artifacts(&report, FP).unwrap();

    let loaded = mgr.load_load_checked(FP).unwrap();
    assert_eq!(loaded.symbol_metrics.len(), report.symbol_metrics.len());
    for (saved, reloaded) in report.symbol_metrics.iter().zip(&loaded.symbol_metrics) {
        assert_eq!(saved.target_name, reloaded.target_name);
        assert_eq!(saved.target_file, reloaded.target_file);
        assert_eq!(saved.semantic_load, reloaded.semantic_load);
        assert_eq!(saved.include_load, reloaded.include_load);
        assert_eq!(saved.collateral_load, reloaded.collateral_load);
        assert_eq!(saved.total_load, reloaded.total_load);
    }
    assert_eq!(
        loaded.file_metrics, report.file_metrics,
        "loaded file_metrics must equal the saved file_metrics element-for-element"
    );
}

/// R2 (Red carrier 2, the bump's own carrier): a `2.1.0`-era load manifest
/// with an otherwise-fresh fingerprint must reach `StaleArtifact` (rebuild
/// trigger) via `load_load_checked`, never a bincode/serde error surface.
/// At Red (pre-bump) the literal "2.1.0" IS the current epoch, so the
/// artifact is accepted fresh and this test fails exactly there; it goes
/// green the moment the 2.2.0 bump lands and stays meaningful forever
/// (same past-epoch pattern as `stale_analysis_epoch_is_rejected_before_blob_read`).
#[test]
fn stale_prior_epoch_load_manifest_reaches_rebuild_not_decode_error() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    mgr.save_load_artifacts(&fixture_load_report_for(&mgr), FP)
        .unwrap();

    let manifest_path = mgr.artifact_root.join("load").join("manifest.json");
    let mut manifest = persisted_manifest(&mgr.artifact_root.join("load"));
    manifest["analysis_version"] = json!("2.1.0");
    fs::write(&manifest_path, manifest.to_string()).unwrap();

    let err = mgr
        .load_load_checked(FP)
        .expect_err("a 2.1.0-era manifest must never be trusted after the 2.2.0 epoch bump");
    assert!(
        is_stale(&err),
        "stale epoch must report StaleArtifact (rebuild trigger): {err}"
    );
    assert!(
        err.to_string().contains("analysis version mismatch"),
        "error must name the epoch gate: {err}"
    );
    assert!(
        !err.to_string().contains("eserialize"),
        "blob must never be deserialized once the epoch is stale: {err}"
    );
    assert!(!mgr.stage_is_fresh("load", FP));
}

/// R3 pin (IV2, green-by-design - labeled a pin in the journal): the
/// manifest totals equal the summary-derived totals computed from the
/// loaded artifact AND from the in-memory report. This is what makes
/// `tool measure`'s number identical by construction.
#[test]
fn manifest_totals_equal_summary_derived_totals() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let report = fixture_load_report_for(&mgr);
    let load_dir = mgr.save_load_artifacts(&report, FP).unwrap();

    let manifest = persisted_manifest(&load_dir);
    let loaded = mgr.load_load_checked(FP).unwrap();

    let loaded_symbol_sum: u64 = loaded
        .symbol_metrics
        .iter()
        .map(|metric| metric.collateral_load as u64)
        .sum();
    let loaded_file_sum: u64 = loaded
        .file_metrics
        .iter()
        .map(|metric| metric.collateral_load as u64)
        .sum();
    let in_memory_symbol_sum: u64 = report
        .symbol_metrics
        .iter()
        .map(|metric| metric.collateral_load as u64)
        .sum();
    let in_memory_file_sum: u64 = report
        .file_metrics
        .iter()
        .map(|metric| metric.collateral_load as u64)
        .sum();

    assert_eq!(
        manifest["total_analyzed_symbols"].as_u64(),
        Some(report.symbol_metrics.len() as u64)
    );
    assert_eq!(
        manifest["total_analyzed_files"].as_u64(),
        Some(report.file_metrics.len() as u64)
    );
    assert_eq!(
        manifest["total_collateral_load"].as_u64(),
        Some(loaded_symbol_sum + loaded_file_sum),
        "manifest total must equal the loaded-artifact summary-derived total"
    );
    assert_eq!(
        manifest["total_collateral_load"].as_u64(),
        Some(in_memory_symbol_sum + in_memory_file_sum),
        "manifest total must equal the in-memory build-side total"
    );
}

/// Manifest-bytes pin (required test "Manifest bytes"): the saved manifest
/// is exactly the fixture's fields with `analysis_version` reading the
/// current epoch - nothing else may drift across the reshape + bump.
#[test]
fn load_manifest_json_fields_are_exactly_totals_plus_current_version() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let load_dir = mgr
        .save_load_artifacts(&fixture_load_report(), FP)
        .unwrap();

    let expected = json!({
        "repository_root": "fixture/repo",
        "total_analyzed_symbols": FIXTURE_SYMBOL_ROWS,
        "total_analyzed_files": FIXTURE_FILE_ROWS,
        "total_collateral_load": FIXTURE_TOTAL_COLLATERAL,
        "input_fingerprint": FP,
        "analysis_version": CURRENT_ANALYSIS_VERSION,
    });
    assert_eq!(
        persisted_manifest(&load_dir),
        expected,
        "manifest must carry exactly the surviving fields; only analysis_version may move"
    );
}
