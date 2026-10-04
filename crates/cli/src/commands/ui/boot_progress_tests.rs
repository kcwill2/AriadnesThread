use super::{
    make_boot_progress, read_state, stage_progress_callback, stage_report, stage_start_percent,
    step_percent, ARTIFACTS_PHASE, ARTIFACTS_SAVED_PERCENT, CALL_GRAPH_PHASE, COLLATERAL_PHASE,
    JUSTIFICATION_COMPLETE_PERCENT, JUSTIFYING_SYMBOLS_PERCENT, PAYLOAD_ASSEMBLED_PERCENT,
    PLANNING_EXTRACTIONS_PERCENT, ProgressFn, ProgressReport, REDUCTION_PHASE, RUN_PHASES,
    STAGES_PER_RUN, TU_INDEX_PHASE,
};
use super::boot_progress::BootStepSnapshot;
use crate::commands::progress::{strip_stage_prefix, STEP_BAR_WIDTH, TerminalStepBars};
use ariadne_pipeline::{LoadMilestone, ProgressEvent};
use ariadne_ui::{ProgressStepStatus, ProgressStepView, UiState};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Labels exactly as the production seed passes them (phase constants with
/// the `[k/N] ` prefix stripped by `seeded`).
fn boot_labels() -> Vec<String> {
    RUN_PHASES
        .iter()
        .map(|phase| strip_stage_prefix(phase).to_string())
        .collect()
}

fn seeded_snapshot() -> BootStepSnapshot {
    BootStepSnapshot::seeded(STAGES_PER_RUN, &RUN_PHASES)
}

/// Stage 1..4 start/end frames, verbatim in `run_pipeline_stages` order
/// (messages included). The stage-5 frames follow in `boot_replay_stream`.
fn stage_one_to_four_frames() -> Vec<ProgressReport> {
    vec![
        stage_report(1, 0, ARTIFACTS_PHASE, "Hashing sources"),
        stage_report(1, 20, ARTIFACTS_PHASE, "Include graph ready"),
        stage_report(2, 20, TU_INDEX_PHASE, "Parsing sources"),
        stage_report(2, 40, TU_INDEX_PHASE, "Index complete"),
        stage_report(3, 40, REDUCTION_PHASE, "Deduplicating"),
        stage_report(3, 60, REDUCTION_PHASE, "Model reduced"),
        stage_report(4, 60, CALL_GRAPH_PHASE, "Constructing"),
        stage_report(4, 80, CALL_GRAPH_PHASE, "Call graph ready"),
    ]
}

/// The pinned stage-5 calibrated sequence, replayed through the REAL
/// emission machinery: static milestones via `stage_report` with the
/// production percents, pipeline events through `stage_progress_callback`
/// (the mapping frozen by
/// `progress_contract_tests::test_stage_five_milestone_sequence_is_monotonic_and_contained`:
/// globals 80, 81, 81x5, 82, 85, 85, 85, 86, 99, 100).
fn stage_five_calibrated_frames() -> Vec<ProgressReport> {
    let collected: Arc<Mutex<Vec<ProgressReport>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = collected.clone();
    let progress: ProgressFn = Arc::new(move |report| sink.lock().unwrap().push(report));
    let mut event_cb = stage_progress_callback(progress.clone());

    progress(stage_report(
        5,
        stage_start_percent(5),
        COLLATERAL_PHASE,
        "Load analysis",
    ));
    event_cb(&ProgressEvent::LoadMilestone(LoadMilestone::SymbolsAnalyzed {
        symbols: 21,
    }));
    for done in 0..=4 {
        event_cb(&ProgressEvent::CollateralFile { done, total: 4 });
    }
    event_cb(&ProgressEvent::LoadMilestone(LoadMilestone::ArtifactsSaved));
    progress(stage_report(
        5,
        ARTIFACTS_SAVED_PERCENT,
        COLLATERAL_PHASE,
        "Collateral mapped",
    ));
    progress(stage_report(
        5,
        PLANNING_EXTRACTIONS_PERCENT,
        COLLATERAL_PHASE,
        "Planning extractions",
    ));
    progress(stage_report(
        5,
        JUSTIFYING_SYMBOLS_PERCENT,
        COLLATERAL_PHASE,
        "Justifying symbols...",
    ));
    progress(stage_report(
        5,
        JUSTIFICATION_COMPLETE_PERCENT,
        COLLATERAL_PHASE,
        "Symbols justified",
    ));
    progress(stage_report(
        5,
        PAYLOAD_ASSEMBLED_PERCENT,
        COLLATERAL_PHASE,
        "Payload assembled",
    ));

    let reports = collected.lock().unwrap().clone();
    let pinned_globals: Vec<usize> = reports.iter().map(|r| r.percent).collect();
    assert_eq!(
        pinned_globals,
        vec![80, 81, 81, 81, 81, 81, 82, 85, 85, 85, 86, 99, 100],
        "stage-5 replay globals must equal the sequence pinned in progress_contract_tests"
    );
    reports
}

/// The full 21-report boot emission order: `run_pipeline_stages` stage 1..4
/// start/end frames, then the pinned stage-5 calibrated sequence (whose
/// first frame is the stage-5 start and whose 9th frame is the
/// "Collateral mapped" repeat emitted by `run_pipeline_stages` itself).
fn boot_replay_stream() -> Vec<ProgressReport> {
    let mut stream = stage_one_to_four_frames();
    stream.extend(stage_five_calibrated_frames());
    stream
}

fn replay(rows_sink: &mut Vec<Vec<ProgressStepView>>, stream: &[ProgressReport]) {
    let mut snapshot = seeded_snapshot();
    for report in stream {
        rows_sink.push(snapshot.apply(report));
    }
}

fn row_at(rows: &[ProgressStepView], stage: usize) -> &ProgressStepView {
    rows.iter()
        .find(|row| row.stage == stage)
        .unwrap_or_else(|| panic!("row for stage {stage} must exist"))
}

// ---------------------------------------------------------------------------
// Shape-example fixture (shape lines 125-137): reproducible from the seed
// plus the stage-1 end and stage-2 mid frames.
// ---------------------------------------------------------------------------

#[test]
fn test_shape_json_example_rows_are_reproducible() {
    let mut snapshot = seeded_snapshot();
    snapshot.apply(&stage_report(1, 20, ARTIFACTS_PHASE, "Include graph ready"));
    let rows = snapshot.apply(&stage_report(2, 26, TU_INDEX_PHASE, "6/24 TUs"));
    let labels = boot_labels();
    let expected = vec![
        ProgressStepView {
            stage: 1,
            label: labels[0].clone(),
            percent: 100,
            message: "Include graph ready".to_string(),
            status: ProgressStepStatus::Done,
        },
        ProgressStepView {
            stage: 2,
            label: labels[1].clone(),
            percent: 30,
            message: "6/24 TUs".to_string(),
            status: ProgressStepStatus::Active,
        },
        ProgressStepView {
            stage: 3,
            label: labels[2].clone(),
            percent: 0,
            message: String::new(),
            status: ProgressStepStatus::Pending,
        },
        ProgressStepView {
            stage: 4,
            label: labels[3].clone(),
            percent: 0,
            message: String::new(),
            status: ProgressStepStatus::Pending,
        },
        ProgressStepView {
            stage: 5,
            label: labels[4].clone(),
            percent: 0,
            message: String::new(),
            status: ProgressStepStatus::Pending,
        },
    ];
    assert_eq!(rows, expected, "rows must equal the shape's mid-boot JSON example field-for-field");
    assert_eq!(
        row_at(&rows, 2).percent,
        step_percent(2, 26),
        "shape example percent 30 is step_percent(2, 26)"
    );
}

// ---------------------------------------------------------------------------
// DP-1 seed contract.
// ---------------------------------------------------------------------------

#[test]
fn test_seeded_snapshot_is_all_pending() {
    let rows = seeded_snapshot().rows();
    assert_eq!(rows.len(), STAGES_PER_RUN);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row.stage, index + 1, "row order must be stage order");
        assert_eq!(row.status, ProgressStepStatus::Pending);
        assert_eq!(row.percent, 0);
        assert_eq!(row.message, "");
    }
    assert_eq!(
        rows.iter().map(|row| row.label.clone()).collect::<Vec<_>>(),
        boot_labels(),
        "labels must be the phases minus the [k/5] prefix"
    );
}

// ---------------------------------------------------------------------------
// I1 per-step scale.
// ---------------------------------------------------------------------------

#[test]
fn test_i1_stage_five_row_matches_pinned_step_percent_sequence() {
    let mut snapshots: Vec<Vec<ProgressStepView>> = Vec::new();
    replay(&mut snapshots, &boot_replay_stream());
    let stage_five_start = stage_one_to_four_frames().len();
    let step_percents: Vec<usize> = snapshots[stage_five_start..]
        .iter()
        .map(|rows| row_at(rows, 5).percent)
        .collect();
    assert_eq!(
        step_percents,
        vec![0, 5, 5, 5, 5, 5, 10, 25, 25, 25, 30, 95, 100],
        "stage-5 row percents must equal the shape's pinned sequence"
    );
}

#[test]
fn test_i1_earlier_rows_reach_100_only_at_own_stage_end() {
    let mut snapshots: Vec<Vec<ProgressStepView>> = Vec::new();
    replay(&mut snapshots, &boot_replay_stream());
    for stage in 1..=4usize {
        let own_end_index = 2 * stage - 1; // frame index of the stage's end report
        for (frame_index, rows) in snapshots.iter().enumerate() {
            let percent = row_at(rows, stage).percent;
            if frame_index < own_end_index {
                assert!(
                    percent < 100,
                    "stage {stage} hit 100 before its own end frame ({frame_index})"
                );
            } else {
                assert_eq!(
                    percent, 100,
                    "stage {stage} must sit at 100 from its end frame on (frame {frame_index})"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// I3 row monotonicity.
// ---------------------------------------------------------------------------

#[test]
fn test_i3_row_percents_are_monotonic_and_never_leave_done() {
    for stream in [
        boot_replay_stream(),
        boot_replay_stream()[..12].to_vec(),
        vec![stage_report(3, 40, REDUCTION_PHASE, "Deduplicating")],
    ] {
        let mut snapshots: Vec<Vec<ProgressStepView>> = Vec::new();
        replay(&mut snapshots, &stream);
        for stage in 1..=STAGES_PER_RUN {
            let mut previous_percent = 0;
            let mut previous_done = false;
            for rows in snapshots.iter() {
                let row = row_at(rows, stage);
                assert!(
                    row.percent >= previous_percent,
                    "row {stage} percent regressed {previous_percent} -> {}",
                    row.percent
                );
                let is_done = row.status == ProgressStepStatus::Done;
                assert!(
                    !previous_done || is_done,
                    "row {stage} left Done with percent {}",
                    row.percent
                );
                previous_percent = row.percent;
                previous_done = is_done;
            }
        }
    }
}

#[test]
fn test_i3_stale_frame_for_earlier_stage_is_ignored() {
    let mut snapshot = seeded_snapshot();
    let after_stage_two = snapshot.apply(&stage_report(2, 26, TU_INDEX_PHASE, "6/24 TUs"));
    // A stale stage-1 frame (defensive: producers are stage-monotone).
    let after_stale = snapshot.apply(&stage_report(1, 12, ARTIFACTS_PHASE, "stale"));
    assert_eq!(after_stale, after_stage_two, "stale frame must not change any row");
    let after_stage_four =
        snapshot.apply(&stage_report(4, 70, CALL_GRAPH_PHASE, "Constructing"));
    let stale_low_percent = snapshot.apply(&stage_report(2, 21, TU_INDEX_PHASE, "stale low"));
    assert_eq!(
        stale_low_percent, after_stage_four,
        "stale frame must be ignored even when its projected percent is lower"
    );
}

// ---------------------------------------------------------------------------
// Finalize rule (mirrors StepBarRenderer's one-shot finalize).
// ---------------------------------------------------------------------------

#[test]
fn test_finalize_rule_completes_step_five_once() {
    let mut snapshot = seeded_snapshot();
    let stream = boot_replay_stream();
    let mut rows: Vec<Vec<ProgressStepView>> = Vec::new();
    for report in &stream[..stream.len() - 1] {
        rows.push(snapshot.apply(report));
    }
    assert_eq!(row_at(rows.last().unwrap(), 5).status, ProgressStepStatus::Active);
    assert_eq!(row_at(rows.last().unwrap(), 5).percent, 95);
    let final_rows = snapshot.apply(stream.last().unwrap());
    let final_row = row_at(&final_rows, 5);
    assert_eq!(final_row.status, ProgressStepStatus::Done);
    assert_eq!(final_row.percent, 100);
    assert_eq!(final_row.message, "Payload assembled");
    // Repeated 100% frames (same or different message) must not move the row.
    let repeat = snapshot.apply(&stage_report(5, 100, COLLATERAL_PHASE, "Payload assembled"));
    assert_eq!(repeat, final_rows, "repeated finalize frames must be inert");
    let late = snapshot.apply(&stage_report(5, 100, COLLATERAL_PHASE, "late message"));
    assert_eq!(
        row_at(&late, 5).message,
        "Payload assembled",
        "post-finalize frames must not rewrite the committed row (renderer never re-commits)"
    );
}

// ---------------------------------------------------------------------------
// I2 TERMINAL-PARITY ORACLE (milestone gate G2 - frozen vs UNCHANGED
// progress.rs renderer).
// ---------------------------------------------------------------------------

/// Test sink duplicated from `progress.rs::step_bar_tests::MemorySink`; the
/// frozen model file itself is NOT touched (handoff frozen lock).
#[derive(Clone)]
struct MemorySink {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl MemorySink {
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn handle(&self) -> Arc<Mutex<dyn std::io::Write + Send>> {
        self.buffer.clone()
    }

    fn text(&self) -> String {
        String::from_utf8(self.buffer.lock().unwrap().clone()).unwrap()
    }
}

/// One terminal bar line decoded by FIELD POSITION from the
/// `format_step_bar` contract `[{stage}/{total}] {label} [{20 cells}] {pp:>3}%  {message}`.
/// No block glyphs are authored here: the bar is sliced by width, never
/// matched by character class.
struct ParsedStepLine {
    stage: usize,
    total_stages: usize,
    label: String,
    percent: usize,
    message: String,
}

fn parse_step_line(line: &str) -> ParsedStepLine {
    // Sanity: the reference render of the same fields must start identically
    // up to the bar (the bar cells come from frozen progress.rs).
    assert!(
        line.starts_with('['),
        "step line must open with the [stage/total] label: {line:?}"
    );
    let stage_close = line.find(']').expect("stage label must close with ]");
    let inside = &line[1..stage_close];
    let (stage_raw, total_raw) = inside.split_once('/').expect("stage label is stage/total");
    let after_label = line
        .get(stage_close + 2..)
        .expect("a space must follow the stage label");
    let bar_open = after_label.find('[').expect("the step bar follows the label");
    let label = after_label[..bar_open].trim_end().to_string();
    let bar_field = &after_label[bar_open + 1..];
    let after_bar_cells: String = bar_field.chars().skip(STEP_BAR_WIDTH).collect();
    assert!(
        after_bar_cells.starts_with("] "),
        "the bar must close with '] ': {line:?}"
    );
    let percent_field = &after_bar_cells[2..];
    let percent_end = percent_field.find('%').expect("percent field ends with %");
    let percent: usize = percent_field[..percent_end]
        .trim()
        .parse()
        .expect("percent field is a number");
    let message = percent_field[percent_end + 1..].to_string();
    assert!(
        message.starts_with("  "),
        "format_step_bar separates the message by two spaces: {line:?}"
    );
    ParsedStepLine {
        stage: stage_raw.parse().expect("stage is a number"),
        total_stages: total_raw.parse().expect("total is a number"),
        label,
        percent,
        message: message.trim_start().to_string(),
    }
}

/// Bytes after the last committed `\n`: the still-open region holds every
/// `\r` redraw of the live line; the last one is what the terminal shows.
fn open_line_of(text: &str) -> Option<String> {
    let after_last_newline = match text.rfind('\n') {
        Some(index) => &text[index + 1..],
        None => text,
    };
    after_last_newline
        .rfind('\r')
        .map(|carriage| after_last_newline[carriage + 1..].to_string())
}

/// Lines the renderer's newline logic has committed: every segment ending
/// at a `\n`, reduced to its final `\r`-chunk (the last frame drawn there).
fn committed_lines_of(text: &str) -> Vec<String> {
    let mut segment_start = 0usize;
    let mut committed: Vec<String> = Vec::new();
    for (byte_index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            let segment = &text[segment_start..byte_index];
            if let Some(carriage) = segment.rfind('\r') {
                committed.push(segment[carriage + 1..].to_string());
            }
            segment_start = byte_index + 1;
        }
    }
    committed
}

/// The oracle itself: replay a stream into BOTH the unchanged
/// `TerminalStepBars` (over a `MemorySink`) and a `BootStepSnapshot`, then
/// assert every newly committed terminal line equals the snapshot's `Done`
/// row for that stage at that moment, and the final open line equals the
/// `Active` row.
fn assert_terminal_parity(stream: &[ProgressReport]) {
    let sink = MemorySink::new();
    let bars = TerminalStepBars::with_sink(sink.handle());
    let mut snapshot = seeded_snapshot();
    let mut committed_seen = 0usize;
    for report in stream {
        let step = step_percent(report.stage, report.percent);
        bars.render(
            report.stage,
            STAGES_PER_RUN,
            report.phase,
            step,
            &report.message,
        );
        let rows = snapshot.apply(report);
        let text = sink.text();
        let committed = committed_lines_of(&text);
        assert!(
            committed.len() >= committed_seen,
            "committed line count can never shrink"
        );
        for line in &committed[committed_seen..] {
            let parsed = parse_step_line(line);
            let row = row_at(&rows, parsed.stage);
            assert_eq!(parsed.total_stages, STAGES_PER_RUN);
            assert_eq!(
                row.status,
                ProgressStepStatus::Done,
                "a committed stage must be Done in the snapshot: {line:?}"
            );
            assert_eq!(parsed.label, row.label);
            assert_eq!(parsed.percent, row.percent);
            assert_eq!(parsed.message, row.message);
        }
        committed_seen = committed.len();
    }
    let text = sink.text();
    match open_line_of(&text) {
        Some(open) => {
            let parsed = parse_step_line(&open);
            let final_rows = snapshot.rows();
            let active_row = row_at(&final_rows, parsed.stage);
            assert_eq!(
                active_row.status,
                ProgressStepStatus::Active,
                "the open terminal line must sit on the Active row: {open:?}"
            );
            assert_eq!(parsed.label, active_row.label);
            assert_eq!(parsed.percent, active_row.percent);
            assert_eq!(parsed.message, active_row.message);
        }
        None => {
            let rows = snapshot.rows();
            assert!(
                rows.iter().all(|row| row.status != ProgressStepStatus::Active),
                "a fully committed run (no open line) must leave no Active row"
            );
        }
    }
    // Drop-time commit of an open line is owned by the renderer's Drop; the
    // parity above compared the pre-drop state, which is what the browser
    // reads while booting.
    drop(bars);
}

#[test]
fn test_i2_terminal_parity_full_boot_run() {
    assert_terminal_parity(&boot_replay_stream());
}

#[test]
fn test_i2_terminal_parity_mid_boot_prefix() {
    // Prefix cut one frame before the finalize: the run stops with an OPEN
    // terminal line (stage 5 at 95), so the open-line half of the parity
    // contract is exercised, not just the committed lines.
    assert_terminal_parity(&boot_replay_stream()[..20]);
}

// ---------------------------------------------------------------------------
// Red wiring test (handoff RGB step 1): the boot progress closure must
// write per-step rows into `state.progress` alongside the globals.
// ---------------------------------------------------------------------------

#[test]
fn test_boot_progress_snapshot_writes_active_row() {
    let state = Arc::new(UiState::new_loading(PathBuf::from("repo-under-test")));
    let progress = make_boot_progress(state.clone());
    progress(ProgressReport {
        stage: 2,
        percent: 26,
        phase: TU_INDEX_PHASE,
        message: "6/24 TUs".to_string(),
    });
    let p = read_state(&state.progress);
    assert!(!p.steps.is_empty());
    assert_eq!(p.total_stages, STAGES_PER_RUN);
    let active = p
        .steps
        .iter()
        .find(|row| row.stage == 2)
        .expect("stage-2 row must exist once boot rows are seeded");
    assert_eq!(
        active.percent,
        step_percent(2, 26),
        "active row must carry the step's own projected percent"
    );
    assert_eq!(active.status, ProgressStepStatus::Active);
    // Globals must stay written exactly as before the widening (I4).
    assert_eq!(p.state, "loading");
    assert_eq!(p.percent, 26);
    assert_eq!(p.phase, TU_INDEX_PHASE);
    assert_eq!(p.message, "6/24 TUs");
}
