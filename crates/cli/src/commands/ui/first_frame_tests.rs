//! I5/I6 locks (boot-progress-visibility V1): the first boot frame must
//! PRECEDE pipeline construction, appear exactly once in production code,
//! and land in the web step rows as an Active stage-1 row. Frame order is
//! locked in source text (imitating the A4 precedent
//! `progress_contract_tests::test_planning_milestone_precedes_justifying_in_source`)
//! and in behavior via a real cold boot of a two-header temp repo.

use super::boot_progress::BootStepSnapshot;
use super::{
    build_ui_payload, ProgressFn, ProgressReport, ARTIFACTS_PHASE, RUN_PHASES, STAGES_PER_RUN,
};
use ariadne_ui::ProgressStepStatus;
use std::sync::{Arc, Mutex};

/// Source-text lock: in the production section of `../ui.rs` (before the
/// first `#[cfg(test)]`), the first `"Hashing sources"` occurrence must sit
/// strictly before the first `"Pipeline::new"`. The needle deliberately has
/// no `(` so it stays green when V3 renames the call to
/// `Pipeline::new_with_progress`.
#[test]
fn test_first_frame_precedes_pipeline_construction() {
    let ui_code = include_str!("../ui.rs");
    let production_end = ui_code
        .find("#[cfg(test)]")
        .expect("ui.rs must have a test module to cut the production section at");
    let production = &ui_code[..production_end];
    let first_frame_at = production
        .find("Hashing sources")
        .expect("frame-1 label must be emitted in production code");
    let pipeline_new_at = production
        .find("Pipeline::new")
        .expect("the pipeline construction call site must exist in production code");
    assert!(
        first_frame_at < pipeline_new_at,
        "\"Hashing sources\" (byte {first_frame_at}) must precede \"Pipeline::new\" (byte {pipeline_new_at}): the bar must exist before the constructor's serial hashing"
    );
}

/// Dedup lock: the frame-1 start label appears exactly once in the
/// production section - moving the emission into `build_ui_payload` must
/// DELETE the one in `run_pipeline_stages`, never duplicate it.
#[test]
fn test_stage_one_start_frame_emitted_exactly_once() {
    let ui_code = include_str!("../ui.rs");
    let production_end = ui_code
        .find("#[cfg(test)]")
        .expect("ui.rs must have a test module to cut the production section at");
    let production = &ui_code[..production_end];
    assert_eq!(
        production.matches("Hashing sources").count(),
        1,
        "the frame-1 start label must be emitted exactly once in production code"
    );
}

/// Cold-boot a two-header repo (no build db) through the REAL
/// `build_ui_payload` with a recording progress fn.
fn cold_boot_reports() -> Vec<ProgressReport> {
    let repo_dir = tempfile::TempDir::new().expect("temp repo");
    let include_dir = repo_dir.path().join("include");
    std::fs::create_dir_all(&include_dir).expect("include dir");
    std::fs::write(
        include_dir.join("alpha.h"),
        "#pragma once\nstruct Alpha { int value; };\n",
    )
    .expect("write alpha.h");
    std::fs::write(
        include_dir.join("beta.h"),
        "#pragma once\n#include \"alpha.h\"\nstruct Beta { Alpha base; };\n",
    )
    .expect("write beta.h");
    let collected: Arc<Mutex<Vec<ProgressReport>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = collected.clone();
    let progress: ProgressFn = Arc::new(move |report| sink.lock().unwrap().push(report));
    let repo_path = repo_dir.path().to_path_buf();
    build_ui_payload(&repo_path, progress).expect("cold boot of a two-header repo must succeed");
    let reports = collected.lock().unwrap().clone();
    reports
}

#[test]
fn test_cold_boot_first_report_is_stage_one_and_two_states() {
    let reports = cold_boot_reports();
    assert!(!reports.is_empty(), "a cold boot must emit frames");
    let first = &reports[0];
    assert_eq!(first.stage, 1, "the very first frame belongs to step [1/5]");
    assert_eq!(first.percent, 0, "frame-1 carries the stage-1 start percent");
    assert_eq!(first.phase, ARTIFACTS_PHASE);
    assert_eq!(first.message, "Hashing sources");
    let stage_one: Vec<&ProgressReport> = reports.iter().filter(|r| r.stage == 1).collect();
    let distinct_states: std::collections::BTreeSet<(usize, &str)> = stage_one
        .iter()
        .map(|r| (r.percent, r.message.as_str()))
        .collect();
    assert!(
        distinct_states.len() >= 2,
        "stage 1 must show movement: at least start and end states, got {distinct_states:?}"
    );
    for report in &stage_one {
        assert!(
            (0..=20).contains(&report.percent),
            "I9: stage-1 frames stay inside [0, 20], got {}",
            report.percent
        );
    }
    assert!(
        stage_one.iter().any(|r| r.percent == 20),
        "the stage-1 end frame (1, 20) must be present (I9 span)"
    );
}

#[test]
fn test_boot_snapshot_mirrors_first_frame() {
    let reports = cold_boot_reports();
    let mut snapshot = BootStepSnapshot::seeded(STAGES_PER_RUN, &RUN_PHASES);
    // Applying ONLY the first frame is the parity probe: after later stages
    // the row legitimately moves to Done. The I6 property under test is
    // that the first frame already rewrites the DP-1 Pending row.
    let rows = snapshot.apply(&reports[0]);
    let stage_one_row = &rows[0];
    assert_eq!(stage_one_row.stage, 1);
    assert_eq!(
        stage_one_row.status,
        ProgressStepStatus::Active,
        "no frozen Pending row may survive the first frame (I6 parity)"
    );
    assert_eq!(
        stage_one_row.message, reports[0].message,
        "the stage-1 web row carries the first frame's message"
    );
}
