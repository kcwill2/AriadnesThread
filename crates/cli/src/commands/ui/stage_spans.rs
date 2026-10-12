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

// ---------------------------------------------------------------------------
// A4: calibrated stage-5 sub-spans, replacing the old fixed [80,95] lie that
// burned 75% of the stage on the ~75 ms file loop while the real tail ran
// with a frozen bar. Calibration = the tail bench on the 520-header/40-TU
// fixture (crates/cli/tests/payload_tail_bench.rs), post-A4 re-run table of
// 2026-09-29: symbol_pass+prep 301ms · file_pass 75.4ms ·
// save_load_artifacts+cache 1417.8ms · ensure_load total 1794.2ms (vs the
// post-A6 runs 1893/1743 — milestone notifies cost nothing measurable) ·
// generate_report+detect_cycles 4.7ms · build_include_elements 5.1ms ·
// path_to_id+file_collateral 28.9ms · planner facts+closures 232.4ms ·
// build_file_justification_with_memo 5313.4ms (DOMINANT) · conservative
// 0.0ms. The stage-5 window totals ~7.38 s over the 20-point global span
// [80,100], so 1 point ~= 0.37 s.
// Re-pin 2026-10-04 (M23.5 T3.2): the cycle row above is now
// detect_cycle_groups at a measured 5.6 ms (--release run of the tail
// bench, 2026-10-04). It replaces 4.7 ms, which already disagreed with
// payload_tail_bench.rs's own 2026-09-29 row of 5.8 ms for the same
// combined call; the measured 5.6 sits with the bench table (parity
// within single-run noise, as linear-time SCC predicts), so the
// pre-existing 4.7-vs-5.8 split is recorded, not papered over.
// UNVERIFIED against bench_compare.py: that tool is off the reviewer
// allowlist (ledger M23.5 F10) and was not run; 5.6 ms is a single local
// measurement, not bench-compare-verified perf evidence. The percent
// constants below are unmoved: this span is a fraction of a point either
// way. Full provenance: payload_tail_bench.rs header + T3-B journal.
// Every milestone sits at the nearest integer point of its CUMULATIVE
// measured time (milestone boundaries within ~0.2 pt of measured reality).
// The largest forward error is <=1 pt mid-file-loop, where the loop-end
// rounding leads briefly and the save milestone reclaims the slack.
// ---------------------------------------------------------------------------
/// Symbol pass + analyzer prep (~301ms -> 0.8 pt, nearest integer 81): the
/// bar leaves the stage-start tick only once the symbol pass has really
/// collected.
pub const SYMBOLS_ANALYZED_PERCENT: usize = 81;
/// File loop upper bound (~75ms of events, rounded up to 1 pt; the loop is
/// a fast pass since A2 — the old [80,95] span it owned is what made the
/// bar look frozen everywhere else).
pub const COLLATERAL_FILE_SPAN_END: usize = 82;
/// `save_load_artifacts` + cache wrap complete (~1418ms; cumulative
/// ~1794ms -> 84.9 pt). Emitted by the pipeline milestone only when the
/// bytes are on disk.
pub const ARTIFACTS_SAVED_PERCENT: usize = 85;
/// Emitted right before `build_extraction_proposals`: the maps before the
/// planner are ~39ms, so cumulative stays ~85.0 pt.
pub const PLANNING_EXTRACTIONS_PERCENT: usize = 85;
/// Emitted right before `build_file_justification_with_memo`: the planner
/// facts+closures run just before (~232ms; cumulative ~2066ms -> 85.6).
pub const JUSTIFYING_SYMBOLS_PERCENT: usize = 86;
/// Emitted after the justification memo returns: the dominant tail
/// (~5313ms ~= 14.4 of the 15 points left after the save) would run past
/// 100 exactly, so it is capped one point short — the terminal 100% frame
/// is reserved for "Payload assembled" (progress.rs finalizes only a
/// step-5 100% frame, so the run must end on that milestone).
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
