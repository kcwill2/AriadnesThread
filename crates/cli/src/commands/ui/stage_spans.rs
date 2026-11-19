//! Calibrated stage-percent span model for the boot progress frames.
//!
//! Extracted VERBATIM from `ui.rs` (boot-progress-visibility V1, hygiene
//! pre-V3): the five `[k/5]` phase labels, the run-phase order, the A4
//! stage-5 calibration evidence, and the pure percent-mapping helpers.
//! Values are byte-identical to the pre-move block (I2/I9: a pure move,
//! no percent value changes). `ui.rs` re-exports these names so the frozen
//! contract-test imports resolve unchanged. Items are `pub` at the module
//! boundary; the module itself is private to `commands::ui`.

pub const ARTIFACTS_PHASE: &str = "[1/5] Loading artifacts...";
pub const TU_INDEX_PHASE: &str = "[2/5] Indexing translation units...";
pub const REDUCTION_PHASE: &str = "[3/5] Reducing semantic model...";
pub const CALL_GRAPH_PHASE: &str = "[4/5] Building call graph...";
pub const COLLATERAL_PHASE: &str = "[5/5] Analyzing collateral...";
pub const STAGES_PER_RUN: usize = 5;

/// The five `[k/5]` phase labels in stage order: the boot-seed input for
/// `BootStepSnapshot::seeded` (run_ui) and the boot closure's own snapshot.
pub const RUN_PHASES: [&'static str; STAGES_PER_RUN] = [
    ARTIFACTS_PHASE,
    TU_INDEX_PHASE,
    REDUCTION_PHASE,
    CALL_GRAPH_PHASE,
    COLLATERAL_PHASE,
];

// Stage-1 split point: end of discovery/hashing, start of the include
// graph walk. Declared here so V3 grows this file, not `ui.rs`.
// TODO(calibrate M27 T1): replace 10 with the measured stage-1 split
// point from the boot_stages release table (A4 precedent).
pub const DISCOVERY_HASHED_SPAN_END: usize = 10;

// Stage-1 scan end: one point short of the stage boundary so the stage-end
// frame `(1, 20)` keeps sole ownership of 20 (I9, contract test 4).
pub const INCLUDE_SCAN_SPAN_END: usize = 19;

// Stage-3 split: the cold reduce reports `ModelReduced` between building
// the model and saving its artifacts. Measured on a generated 1500-TU,
// 2500-file repo (release, 2026-11-19): reduce 1.51 s, save 0.66 s, so the
// build ends at ~70% of the stage: 40 + 14.
pub const MODEL_REDUCED_PERCENT: usize = 54;

// ---------------------------------------------------------------------------
// Stage-5 sub-spans, recalibrated 2026-11-19 after the store-once index and
// the compact file metric. The earlier A4 table (2026-09-29, 40-TU fixture)
// had the justification memo dominant (5.3 s) and gave the file pass one
// point, so on large repos the bar sat at "5%" through the longest step.
// Release measurement on the generated 1500-TU, 2500-file repo, cold load
// stage: symbol pass ~1 s, file pass 4-7 s (one CollateralFile event per
// file), save <0.1 s, include report + cycles 0.15 s, planner 0.5 s, path
// maps + collateral map 0.44 s, justification 0.41 s, payload 0.1 s. The
// stage totals ~7-10 s over 20 points, so 1 point ~= 0.4 s; every
// milestone sits at the nearest point of its cumulative share.
// ---------------------------------------------------------------------------
/// Symbol pass + analyzer prep (~1 s -> 3 pt).
pub const SYMBOLS_ANALYZED_PERCENT: usize = 83;
/// File pass upper bound: the dominant step, mapped per file over [83, 93].
pub const COLLATERAL_FILE_SPAN_END: usize = 93;
/// `save_load_artifacts` + cache wrap complete. Emitted by the pipeline
/// milestone only when the bytes are on disk.
pub const ARTIFACTS_SAVED_PERCENT: usize = 94;
/// Emitted right before the planner; the include report and cycle groups
/// before it take a fraction of a point.
pub const PLANNING_EXTRACTIONS_PERCENT: usize = 94;
/// Emitted after the planner (~0.5 s -> 2 pt), before the path maps and
/// the justification memo.
pub const JUSTIFYING_SYMBOLS_PERCENT: usize = 96;
/// Emitted after the justification memo returns (path maps + memo ~0.85 s
/// -> 3 pt). The terminal 100% frame stays reserved for "Payload
/// assembled" (progress.rs finalizes only a step-5 100% frame, so the run
/// must end on that milestone).
pub const JUSTIFICATION_COMPLETE_PERCENT: usize = 99;
/// Terminal milestone after payload assembly, before the build returns.
/// `swap_into_state` separately marks the web state "Ready".
pub const PAYLOAD_ASSEMBLED_PERCENT: usize = 100;

/// Stage k of 5 owns the percent span `[(k-1)*20, k*20]`.
pub fn stage_start_percent(stage: usize) -> usize {
    (stage - 1) * 20
}

pub fn stage_end_percent(stage: usize) -> usize {
    stage * 20
}

/// Project a global pipeline percent onto the step's own 0..=100 scale for
/// per-step terminal bars. Clamped at both ends.
pub fn step_percent(stage: usize, global_percent: usize) -> usize {
    let lower = stage_start_percent(stage);
    let span = stage_end_percent(stage) - lower;
    let within = global_percent.saturating_sub(lower);
    (within * 100 / span).min(100)
}

pub fn mapped_span_percent(lower: usize, upper: usize, done: usize, total: usize) -> usize {
    if total == 0 {
        return lower;
    }
    (lower + (upper - lower) * done / total).min(upper)
}

pub fn tu_index_percent(done: usize, total: usize) -> usize {
    mapped_span_percent(stage_start_percent(2), stage_end_percent(2), done, total)
}

pub fn collateral_file_percent(done: usize, total: usize) -> usize {
    mapped_span_percent(
        SYMBOLS_ANALYZED_PERCENT,
        COLLATERAL_FILE_SPAN_END,
        done,
        total,
    )
}

/// V3 stage-1 split: `DiscoveryHashed` percent over [stage start(1), D].
/// `total: None` (the single-pass walk has no bounded total) holds the
/// percent at the stage start while the message advances (honest, I9).
pub fn discovery_hashed_percent(done: usize, total: Option<usize>) -> usize {
    total.map_or(stage_start_percent(1), |bounded| {
        mapped_span_percent(
            stage_start_percent(1),
            DISCOVERY_HASHED_SPAN_END,
            done,
            bounded,
        )
    })
}

/// V3 stage-1 split: the cold include-scan substeps over [D, 19].
pub fn include_scan_percent(done: usize, total: usize) -> usize {
    mapped_span_percent(
        DISCOVERY_HASHED_SPAN_END,
        INCLUDE_SCAN_SPAN_END,
        done,
        total,
    )
}
