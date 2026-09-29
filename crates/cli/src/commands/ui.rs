use anyhow::{Context, Result};
use ariadne_index::StageFingerprints;
use ariadne_pipeline::{LoadMilestone, Pipeline, ProgressEvent};
use ariadne_ui::{GraphExportPayload, StageInfo, StageStatus, UiState};
use crate::commands::progress::{strip_stage_prefix, TerminalStepBars};
use crate::commands::ui_payload::{
    build_extraction_proposals, build_file_collateral, build_path_to_id,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct UiBuildResult {
    pub include_graph: ariadne_graph::IncludeGraph,
    pub reduced_model: ariadne_graph::ReducedSemanticModel,
    pub call_graph: ariadne_graph::CallGraph,
    /// The generation's ONE shared memo, closure-seeded from stage output
    /// (plan C1): every live route request reuses it.
    pub load_memo: Arc<ariadne_graph::LoadMemo>,
    pub payload: GraphExportPayload,
    pub status: StageStatus,
    pub fps: StageFingerprints,
}

/// Poison recovery: the holder already panicked elsewhere; the data stays
/// memory-consistent, so keep serving/rebuilding instead of cascading panics.
fn read_state<'a, T>(lock: &'a RwLock<T>) -> std::sync::RwLockReadGuard<'a, T> {
    lock.read().unwrap_or_else(|e| e.into_inner())
}

fn write_state<'a, T>(lock: &'a RwLock<T>) -> std::sync::RwLockWriteGuard<'a, T> {
    lock.write().unwrap_or_else(|e| e.into_inner())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn stage_status_from_pipeline(pipeline: &Pipeline) -> StageStatus {
    StageStatus {
        stages: pipeline
            .stats
            .stages
            .iter()
            .map(|s| StageInfo {
                stage: s.stage.clone(),
                reuse: s.reuse.as_str().to_string(),
            })
            .collect(),
        last_refresh: Some(now_ms()),
        rebuilding: false,
        reused_tus: pipeline.stats.reused_tus,
        reanalyzed_tus: pipeline.stats.reanalyzed_tus,
    }
}

/// One progress update: which `[k/5]` step produced it (passed explicitly,
/// never parsed back out of the phase string), the global pipeline percent
/// (what the web `UiState` stores) and the display strings.
#[derive(Debug, Clone)]
pub struct ProgressReport {
    pub stage: usize,
    pub percent: usize,
    pub phase: &'static str,
    pub message: String,
}

type ProgressFn = Arc<dyn Fn(ProgressReport) + Send + Sync>;

fn stage_report(
    stage: usize,
    percent: usize,
    phase: &'static str,
    message: &str,
) -> ProgressReport {
    ProgressReport {
        stage,
        percent,
        phase,
        message: message.to_string(),
    }
}

const ARTIFACTS_PHASE: &str = "[1/5] Loading artifacts...";
const TU_INDEX_PHASE: &str = "[2/5] Indexing translation units...";
const REDUCTION_PHASE: &str = "[3/5] Reducing semantic model...";
const CALL_GRAPH_PHASE: &str = "[4/5] Building call graph...";
const COLLATERAL_PHASE: &str = "[5/5] Analyzing collateral...";
const STAGES_PER_RUN: usize = 5;

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
// Every milestone sits at the nearest integer point of its CUMULATIVE
// measured time (milestone boundaries within ~0.2 pt of measured reality).
// The largest forward error is <=1 pt mid-file-loop, where the loop-end
// rounding leads briefly and the save milestone reclaims the slack.
// ---------------------------------------------------------------------------
/// Symbol pass + analyzer prep (~301ms -> 0.8 pt, nearest integer 81): the
/// bar leaves the stage-start tick only once the symbol pass has really
/// collected.
const SYMBOLS_ANALYZED_PERCENT: usize = 81;
/// File loop upper bound (~75ms of events, rounded up to 1 pt; the loop is
/// a fast pass since A2 — the old [80,95] span it owned is what made the
/// bar look frozen everywhere else).
const COLLATERAL_FILE_SPAN_END: usize = 82;
/// `save_load_artifacts` + cache wrap complete (~1418ms; cumulative
/// ~1794ms -> 84.9 pt). Emitted by the pipeline milestone only when the
/// bytes are on disk.
const ARTIFACTS_SAVED_PERCENT: usize = 85;
/// Emitted right before `build_extraction_proposals`: the maps before the
/// planner are ~39ms, so cumulative stays ~85.0 pt.
const PLANNING_EXTRACTIONS_PERCENT: usize = 85;
/// Emitted right before `build_file_justification_with_memo`: the planner
/// facts+closures run just before (~232ms; cumulative ~2066ms -> 85.6).
const JUSTIFYING_SYMBOLS_PERCENT: usize = 86;
/// Emitted after the justification memo returns: the dominant tail
/// (~5313ms ~= 14.4 of the 15 points left after the save) would run past
/// 100 exactly, so it is capped one point short — the terminal 100% frame
/// is reserved for "Payload assembled" (progress.rs finalizes only a
/// step-5 100% frame, so the run must end on that milestone).
const JUSTIFICATION_COMPLETE_PERCENT: usize = 99;
/// Terminal milestone after payload assembly, before the build returns.
/// `swap_into_state` separately marks the web state "Ready".
const PAYLOAD_ASSEMBLED_PERCENT: usize = 100;

/// Stage k of 5 owns the percent span `[(k-1)*20, k*20]`.
fn stage_start_percent(stage: usize) -> usize {
    (stage - 1) * 20
}

fn stage_end_percent(stage: usize) -> usize {
    stage * 20
}

/// Project a global pipeline percent onto the step's own 0..=100 scale for
/// per-step terminal bars. Clamped at both ends.
fn step_percent(stage: usize, global_percent: usize) -> usize {
    let lower = stage_start_percent(stage);
    let span = stage_end_percent(stage) - lower;
    let within = global_percent.saturating_sub(lower);
    (within * 100 / span).min(100)
}

fn mapped_span_percent(lower: usize, upper: usize, done: usize, total: usize) -> usize {
    if total == 0 {
        return lower;
    }
    (lower + (upper - lower) * done / total).min(upper)
}

fn tu_index_percent(done: usize, total: usize) -> usize {
    mapped_span_percent(stage_start_percent(2), stage_end_percent(2), done, total)
}

fn collateral_file_percent(done: usize, total: usize) -> usize {
    mapped_span_percent(
        SYMBOLS_ANALYZED_PERCENT,
        COLLATERAL_FILE_SPAN_END,
        done,
        total,
    )
}

/// Drive the five artifact-backed stages, emitting start/end milestones that
/// keep every percent inside the span owned by its `[k/5]` step label.
fn run_pipeline_stages(
    pipeline: &mut Pipeline,
    progress: &ProgressFn,
) -> Result<ariadne_graph::ReducedSemanticModel> {
    progress(stage_report(
        1,
        stage_start_percent(1),
        ARTIFACTS_PHASE,
        "Hashing sources",
    ));
    pipeline.ensure_include_graph()?;
    progress(stage_report(
        1,
        stage_end_percent(1),
        ARTIFACTS_PHASE,
        "Include graph ready",
    ));
    progress(stage_report(
        2,
        stage_start_percent(2),
        TU_INDEX_PHASE,
        "Parsing sources",
    ));
    pipeline.ensure_index()?;
    progress(stage_report(
        2,
        stage_end_percent(2),
        TU_INDEX_PHASE,
        "Index complete",
    ));
    progress(stage_report(
        3,
        stage_start_percent(3),
        REDUCTION_PHASE,
        "Deduplicating",
    ));
    let reduced_model = pipeline.get_or_build_reduced()?;
    progress(stage_report(
        3,
        stage_end_percent(3),
        REDUCTION_PHASE,
        "Model reduced",
    ));
    progress(stage_report(
        4,
        stage_start_percent(4),
        CALL_GRAPH_PHASE,
        "Constructing",
    ));
    pipeline.ensure_callgraph()?;
    progress(stage_report(
        4,
        stage_end_percent(4),
        CALL_GRAPH_PHASE,
        "Call graph ready",
    ));
    progress(stage_report(
        5,
        stage_start_percent(5),
        COLLATERAL_PHASE,
        "Load analysis",
    ));
    pipeline.ensure_load()?;
    // A4: on a cold rebuild the pipeline callback has already emitted the
    // "Load artifacts saved" milestone at 85; this frame repeats that
    // percent under the stage's own label. On a warm load-stage reuse no
    // pipeline events fire at all, so this is the only load-complete marker
    // and the bar honestly jumps 80 -> 85 once the cached stage returned.
    progress(stage_report(
        5,
        ARTIFACTS_SAVED_PERCENT,
        COLLATERAL_PHASE,
        "Collateral mapped",
    ));
    Ok(reduced_model)
}

/// Map one pipeline `ProgressEvent` onto the stage-owned report. A4: the
/// load milestones become named stage-5 reports at their calibrated points;
/// the new-variant arm is enforced by the compiler, so a future event can
/// never be silently dropped on the floor here.
fn stage_progress_callback(cb_progress: ProgressFn) -> impl FnMut(&ProgressEvent) + Send {
    move |event| match event {
        ProgressEvent::TuIndex { done, total } => (cb_progress)(stage_report(
            2,
            tu_index_percent(*done, *total),
            TU_INDEX_PHASE,
            &format!("{done}/{total} TUs"),
        )),
        ProgressEvent::CollateralFile { done, total } => (cb_progress)(stage_report(
            5,
            collateral_file_percent(*done, *total),
            COLLATERAL_PHASE,
            &format!("{done}/{total} files"),
        )),
        ProgressEvent::LoadMilestone(milestone) => (cb_progress)(match milestone {
            LoadMilestone::SymbolsAnalyzed { symbols } => stage_report(
                5,
                SYMBOLS_ANALYZED_PERCENT,
                COLLATERAL_PHASE,
                &format!("{symbols} symbols analyzed"),
            ),
            LoadMilestone::ArtifactsSaved => stage_report(
                5,
                ARTIFACTS_SAVED_PERCENT,
                COLLATERAL_PHASE,
                "Load artifacts saved",
            ),
        }),
    }
}

/// One of build_ui_payload's own stage-5 assembly milestones.
fn stage5_report(progress: &ProgressFn, percent: usize, message: &str) {
    progress(stage_report(5, percent, COLLATERAL_PHASE, message));
}

/// Run every pipeline stage (loading artifact-backed stages when their input
/// fingerprint is fresh) and assemble the UI payload from the results.
fn build_ui_payload(repo_path: &PathBuf, progress: ProgressFn) -> Result<UiBuildResult> {
    let progress_cb = stage_progress_callback(progress.clone());
    let mut pipeline = Pipeline::new(repo_path, None)?.with_progress(progress_cb);

    let reduced_model = run_pipeline_stages(&mut pipeline, &progress)?;
    // A3: graph/closures MOVE out of the single-use pipeline, no clone.
    let (include_graph, closures) = pipeline.take_include_graph()?;
    let call_graph = pipeline.get_or_build_callgraph()?;
    let load_report = pipeline.get_or_build_load()?;
    let report = include_graph.generate_report();
    let cycles = include_graph.detect_cycles();
    let include_elements = ariadne_ui::build_include_elements(&include_graph, &closures, &cycles);
    let path_to_id = build_path_to_id(&include_graph);
    let file_collateral = build_file_collateral(&path_to_id, &load_report.file_metrics);
    // A1: planner candidates are screened from the stage's load metrics and
    // reuse the stage closures, before the memo below takes them.
    // Milestones follow the CALL ORDER, not the cost order: the planner
    // really runs before the justification memo, so "Planning extractions"
    // is emitted here (A4 honesty rule — the boot-time ms table in the
    // constants block shows why the planner earns only ~0.6 pt).
    stage5_report(
        &progress,
        PLANNING_EXTRACTIONS_PERCENT,
        "Planning extractions",
    );
    let extraction_proposals = build_extraction_proposals(
        &include_graph,
        &reduced_model,
        &pipeline.discovery.files,
        &translation_unit_file_ids(&pipeline.build),
        &load_report.file_metrics,
        &closures,
    );
    // C1: ONE memo for the whole generation, seeded with the stage closures
    // (`with_closures` pattern of `ensure_load`). Payload assembly and every
    // live route share it; rebuilds swap graph+model+memo as one unit.
    let load_memo = Arc::new(ariadne_graph::LoadMemo::with_closures(closures));
    // Span floor of the dominant tail: the memo call emits no per-file
    // events, so 86 -> 99 is one honest span, not a frozen 12-point jump.
    stage5_report(
        &progress,
        JUSTIFYING_SYMBOLS_PERCENT,
        "Justifying symbols...",
    );
    let file_justification = ariadne_ui::build_file_justification_with_memo(
        &load_report.file_metrics,
        &include_graph,
        &reduced_model,
        &path_to_id,
        Some(&load_memo),
    );
    stage5_report(&progress, JUSTIFICATION_COMPLETE_PERCENT, "Symbols justified");
    let payload = GraphExportPayload {
        repo_name: pipeline
            .repo
            .root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        include_graph: include_elements,
        symbol_graph: ariadne_ui::build_symbol_elements(&reduced_model),
        call_graph: ariadne_ui::build_call_elements(&call_graph),
        stats: ariadne_ui::SummaryStats {
            total_files: include_graph.file_paths.len(),
            total_symbols: reduced_model.symbols.len(),
            total_calls: call_graph.edges.len(),
            cycle_count: report.cycle_count,
        },
        file_collateral,
        file_justification,
        extraction_proposals,
    };
    let status = stage_status_from_pipeline(&pipeline);
    stage5_report(&progress, PAYLOAD_ASSEMBLED_PERCENT, "Payload assembled");
    Ok(UiBuildResult {
        include_graph,
        reduced_model,
        call_graph,
        load_memo,
        payload,
        status,
        fps: pipeline.fps.clone(),
    })
}

fn translation_unit_file_ids(
    build: &ariadne_build::BuildDiscoveryResult,
) -> Vec<ariadne_repo::FileId> {
    build
        .translation_units
        .iter()
        .map(|tu| tu.file_id)
        .collect()
}

/// Swap a freshly built snapshot into live UI state. The stale payload keeps
/// serving until each slot is replaced. `{graph, model, memo}` enter as ONE
/// generation via `install_analysis` (plan C1), so no request can pair one
/// generation's memo with another generation's graph (no stale entries).
fn swap_into_state(state: &UiState, out: UiBuildResult) -> StageFingerprints {
    let fps = out.fps.clone();
    state.install_analysis(out.include_graph, out.reduced_model, out.load_memo);
    *write_state(&state.call_graph) = Some(out.call_graph);
    *write_state(&state.payload) = Some(out.payload);
    *write_state(&state.stage_status) = out.status;
    let mut p = write_state(&state.progress);
    p.state = "ready".to_string();
    p.phase = "Complete".to_string();
    p.message = "Ready".to_string();
    p.percent = 100;
    fps
}

/// Polling re-hash watcher: cheaply re-derives stage fingerprints on an
/// interval; on any change it rebuilds stale stages in the background and
/// atomically swaps the UI payload. Queued detections collapse into one pass.
fn spawn_watcher(
    state: Arc<UiState>,
    repo_path: PathBuf,
    interval: Duration,
    initial_fps: StageFingerprints,
) {
    let rebuilding = Arc::new(AtomicBool::new(false));
    let current_fps = Arc::new(RwLock::new(initial_fps));

    tokio::spawn(async move {
        let mut tick = tokio::time::interval(interval);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            if rebuilding.swap(true, Ordering::SeqCst) {
                continue;
            }
            let state = state.clone();
            let repo = repo_path.clone();
            let fps_lock = current_fps.clone();
            let rebuilding = rebuilding.clone();
            let _handle = tokio::task::spawn_blocking(move || {
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    // Pure, artifact-free freshness check: re-hash inputs only.
                    let fresh_fps = match Pipeline::rehash_inputs(&repo) {
                        Ok(fps) => fps,
                        Err(e) => {
                            tracing::warn!("UI freshness check failed: {e}");
                            return;
                        }
                    };
                    if fresh_fps == *read_state(&fps_lock) {
                        return;
                    }
                    write_state(&state.stage_status).rebuilding = true;
                    // Exactly one pipeline build serves the new fingerprints
                    // and the payload; its own fps replace the rehashed ones.
                    // One plain line per report: this thread shares stdout
                    // with live server logs, so no `\r` redraws here.
                    let quiet: ProgressFn = Arc::new(|report: ProgressReport| {
                        println!(
                            "[refresh {}/{} {:>3}%] {} {} (background refresh)",
                            report.stage,
                            STAGES_PER_RUN,
                            report.percent,
                            strip_stage_prefix(report.phase),
                            report.message
                        );
                    });
                    match build_ui_payload(&repo, quiet) {
                        Ok(out) => {
                            let new_fps = swap_into_state(&state, out);
                            *write_state(&fps_lock) = new_fps;
                        }
                        Err(e) => {
                            tracing::warn!("UI live refresh failed: {e}");
                        }
                    }
                    write_state(&state.stage_status).rebuilding = false;
                }));
                let _ = outcome;
                rebuilding.store(false, Ordering::SeqCst);
            });
        }
    });
}

/// Live per-step terminal bar renderer. TerminalStepBars is Send + Sync, so
/// the closure satisfies ProgressFn's Send + Sync bound and may move into
/// spawn_blocking; dropping the closure at the end of the build also drops
/// the guard, which commits any bar line left open by a failed build.
fn make_bar_progress() -> ProgressFn {
    let bars = TerminalStepBars::new();
    Arc::new(move |report: ProgressReport| {
        let step = step_percent(report.stage, report.percent);
        bars.render(
            report.stage,
            STAGES_PER_RUN,
            report.phase,
            step,
            &report.message,
        );
    })
}

/// Boot progress for the interactive server: mirrors the global percent into
/// web `UiState` (browser bar unchanged) and drives the live per-step bar.
fn make_boot_progress(state: Arc<UiState>) -> ProgressFn {
    let bar = make_bar_progress();
    Arc::new(move |report: ProgressReport| {
        {
            let mut p = write_state(&state.progress);
            p.state = "loading".to_string();
            p.phase = report.phase.to_string();
            p.message = report.message.clone();
            p.percent = report.percent;
        }
        bar(report);
    })
}

pub async fn run_ui(
    repo_opt: Option<PathBuf>,
    port: u16,
    no_open: bool,
    export_opt: Option<PathBuf>,
    watch_interval: u64,
    no_watch: bool,
) -> Result<()> {
    let repo_path = match repo_opt {
        Some(path) => path,
        None => std::env::current_dir().context("current working directory unavailable")?,
    };

    if let Some(path) = export_opt {
        let out = build_ui_payload(&repo_path, make_bar_progress())?;
        println!(
            "{} TUs reused, {} reanalyzed; stages: {}",
            out.status.reused_tus,
            out.status.reanalyzed_tus,
            out.status
                .stages
                .iter()
                .map(|s| format!("{}:{}", s.stage, s.reuse))
                .collect::<Vec<_>>()
                .join(", ")
        );
        ariadne_ui::export_standalone_html(&out.payload, "", &path)?;
        println!("Report exported successfully to {}", path.display());
    } else {
        let state = Arc::new(UiState::new_loading(repo_path.clone()));

        let app = ariadne_ui::build_router(state.clone());
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

        println!(
            "AriadnesThread Visualizer loading at http://localhost:{}",
            port
        );
        if !no_open {
            let _ = open::that(format!("http://localhost:{}", port));
        }

        let boot_state = state.clone();
        let boot_repo = repo_path.clone();
        let (fps_tx, fps_rx) = tokio::sync::oneshot::channel();

        let boot_progress = make_boot_progress(boot_state.clone());

        tokio::task::spawn_blocking(move || match build_ui_payload(&boot_repo, boot_progress) {
            Ok(out) => {
                let fps = swap_into_state(&boot_state, out);
                let _ = fps_tx.send(Some(fps));
            }
            Err(e) => {
                // `build_ui_payload` consumed the progress Arc, so its
                // TerminalStepBars guard has already dropped and committed
                // any half-drawn bar line; anything printed from here on,
                // including this error report, starts on a fresh line.
                let mut p = write_state(&boot_state.progress);
                p.state = "error".to_string();
                p.message = e.to_string();
                let _ = fps_tx.send(None);
            }
        });

        if !no_watch {
            tokio::spawn(async move {
                if let Ok(Some(fps)) = fps_rx.await {
                    spawn_watcher(
                        state,
                        repo_path,
                        Duration::from_secs(watch_interval.max(1)),
                        fps,
                    );
                }
            });
        }

        axum::serve(listener, app).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ui_is_artifact_first() {
        let ui_code = include_str!("ui.rs");
        assert!(ui_code.contains("Pipeline::new"));
        assert!(ui_code.contains("ensure_include_graph"));
        assert!(ui_code.contains("get_or_build_callgraph"));
        assert!(ui_code.contains("get_or_build_load"));
        assert!(ui_code.contains("spawn_watcher"));

        let index_code = include_str!("index.rs");
        assert!(index_code.contains("ProgressEvent::TuIndex"));
        assert!(index_code.contains("print_progress"));

        // Artifact-first must survive the payload-builder split (plan A5):
        // the builder file never constructs its own Pipeline/LoadAnalyzer —
        // it only receives stage output — and it carries the A1 boot wiring.
        let payload_code = include_str!("ui_payload.rs");
        assert!(
            !payload_code.contains("Pipeline::new"),
            "ui_payload must not construct a Pipeline; only build_ui_payload does"
        );
        assert!(
            !payload_code.contains("LoadAnalyzer::new"),
            "ui_payload must not run its own load analysis; boot hands it the stage metrics"
        );
        assert!(
            payload_code.contains("with_screening_facts"),
            "ui_payload must carry the A1 screening-facts planner wiring"
        );
    }
}

#[cfg(test)]
mod progress_contract_tests {
    use super::{
        collateral_file_percent, stage_end_percent, stage_progress_callback, stage_report,
        stage_start_percent, step_percent, tu_index_percent, ARTIFACTS_PHASE,
        ARTIFACTS_SAVED_PERCENT, CALL_GRAPH_PHASE, COLLATERAL_FILE_SPAN_END, COLLATERAL_PHASE,
        JUSTIFICATION_COMPLETE_PERCENT, JUSTIFYING_SYMBOLS_PERCENT, PAYLOAD_ASSEMBLED_PERCENT,
        PLANNING_EXTRACTIONS_PERCENT, ProgressFn, ProgressReport, REDUCTION_PHASE, STAGES_PER_RUN,
        SYMBOLS_ANALYZED_PERCENT, TU_INDEX_PHASE,
    };
    use ariadne_pipeline::{LoadMilestone, ProgressEvent};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    const POINTS_PER_STAGE: usize = 100 / STAGES_PER_RUN;

    /// The ordered static milestones exactly as `build_ui_payload` emits them,
    /// built from the same constants and helpers the real code path calls.
    /// A4: stage 5's milestone set is {80 start, 81 "Symbols analyzed",
    /// [81,82] file-event span, 85 artifacts-saved / "Collateral mapped" /
    /// "Planning extractions", 86 "Justifying symbols...", 99 "Symbols
    /// justified", 100 payload finalization}, all within [80, 100].
    fn emitted_milestones() -> Vec<(usize, &'static str)> {
        vec![
            (stage_start_percent(1), ARTIFACTS_PHASE),
            (stage_end_percent(1), ARTIFACTS_PHASE),
            (stage_start_percent(2), TU_INDEX_PHASE),
            (stage_end_percent(2), TU_INDEX_PHASE),
            (stage_start_percent(3), REDUCTION_PHASE),
            (stage_end_percent(3), REDUCTION_PHASE),
            (stage_start_percent(4), CALL_GRAPH_PHASE),
            (stage_end_percent(4), CALL_GRAPH_PHASE),
            (stage_start_percent(5), COLLATERAL_PHASE),
            (SYMBOLS_ANALYZED_PERCENT, COLLATERAL_PHASE),
            (ARTIFACTS_SAVED_PERCENT, COLLATERAL_PHASE),
            (PLANNING_EXTRACTIONS_PERCENT, COLLATERAL_PHASE),
            (JUSTIFYING_SYMBOLS_PERCENT, COLLATERAL_PHASE),
            (JUSTIFICATION_COMPLETE_PERCENT, COLLATERAL_PHASE),
            (100, COLLATERAL_PHASE),
        ]
    }

    fn stage_number_of(phase: &str) -> usize {
        phase
            .as_bytes()
            .get(1)
            .and_then(|byte| (*byte as char).to_digit(10))
            .unwrap_or_else(|| panic!("milestone phase must carry a [k/5] label: {phase:?}"))
            as usize
    }

    #[test]
    fn test_stage_milestones_match_step_labels() {
        let milestones = emitted_milestones();
        for pair in milestones.windows(2) {
            assert!(
                pair[0].0 <= pair[1].0,
                "percent must be monotonic, got {} before {}",
                pair[0].0,
                pair[1].0
            );
        }
        for (percent, phase) in &milestones {
            let stage = stage_number_of(phase);
            let lower = (stage - 1) * POINTS_PER_STAGE;
            let upper = stage * POINTS_PER_STAGE;
            assert!(
                (lower..=upper).contains(percent),
                "milestone {phase} at {percent}% escapes its [{lower}, {upper}] span"
            );
        }
        // The contract table must not be fiction: the payload-assembly
        // milestone that finalizes step 5 at 100% must really be emitted.
        // Built via format! so this test's own source cannot match itself.
        let assembled_needle = format!("Payload {}", "assembled");
        assert!(
            include_str!("ui.rs").contains(assembled_needle.as_str()),
            "step 5 must finalize with a 100% payload-assembly milestone"
        );
    }

    #[test]
    fn test_step_percent_maps_stage_span_onto_zero_hundred() {
        assert_eq!(step_percent(1, 0), 0);
        assert_eq!(step_percent(1, 10), 50);
        assert_eq!(step_percent(1, 20), 100);
        assert_eq!(step_percent(2, 20), 0, "stage start is step start");
        assert_eq!(step_percent(2, 30), 50, "stage midpoint is 50% of the step");
        assert_eq!(step_percent(2, 40), 100, "stage end is step end");
        assert_eq!(step_percent(5, 80), 0);
        assert_eq!(step_percent(5, 90), 50);
        assert_eq!(step_percent(5, 100), 100);
        assert_eq!(step_percent(2, 5), 0, "clamped below own span");
        assert_eq!(step_percent(2, 95), 100, "clamped above own span");
        assert_eq!(step_percent(5, 120), 100, "clamped past 100");
    }

    #[test]
    fn test_tu_index_events_stay_in_stage_two_span() {
        assert_eq!(tu_index_percent(0, 0), 20, "empty TU set must sit at stage start");
        assert_eq!(tu_index_percent(0, 7), 20);
        assert_eq!(tu_index_percent(7, 7), 40);
        let mut previous = tu_index_percent(0, 9);
        for done in 1..=9 {
            let percent = tu_index_percent(done, 9);
            assert!((20..=40).contains(&percent), "{done}/9 mapped to {percent}%");
            assert!(percent >= previous, "percent regressed at {done}/9");
            previous = percent;
        }
    }

    #[test]
    fn test_collateral_events_stay_in_stage_five_span() {
        assert_eq!(
            collateral_file_percent(0, 0),
            81,
            "empty collateral set must sit at the file-loop floor"
        );
        assert_eq!(collateral_file_percent(0, 7), 81);
        assert_eq!(collateral_file_percent(7, 7), COLLATERAL_FILE_SPAN_END);
        let mut previous = collateral_file_percent(0, 9);
        for done in 1..=9 {
            let percent = collateral_file_percent(done, 9);
            assert!(
                (SYMBOLS_ANALYZED_PERCENT..=COLLATERAL_FILE_SPAN_END).contains(&percent),
                "{done}/9 mapped to {percent}%"
            );
            assert!(percent >= previous, "percent regressed at {done}/9");
            previous = percent;
        }
    }

    /// A4: replay the exact stage-5 emission order of `run_pipeline_stages`
    /// and `stage_progress_callback` and `build_ui_payload` (static
    /// milestones through the same `stage_report` helper, pipeline events
    /// through the real callback mapping) and pin the produced report
    /// sequence: percent monotonic non-decreasing, every report inside
    /// [80, 100], and exactly one 100% frame — the terminal "Payload
    /// assembled" (progress.rs finalizes only a step-5 100% frame, so this
    /// is the contract that keeps the finalize at the true end of the
    /// build).
    #[test]
    fn test_stage_five_milestone_sequence_is_monotonic_and_contained() {
        let collected: Arc<Mutex<Vec<ProgressReport>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = collected.clone();
        let progress: ProgressFn = Arc::new(move |report| sink.lock().unwrap().push(report));
        let mut event_cb = stage_progress_callback(progress.clone());

        progress(stage_report(5, stage_start_percent(5), COLLATERAL_PHASE, "Load analysis"));
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

        let reports = collected.lock().unwrap();
        let sequence: Vec<(usize, &str)> = reports
            .iter()
            .map(|report| (report.percent, report.message.as_str()))
            .collect();
        assert_eq!(
            sequence,
            vec![
                (80, "Load analysis"),
                (81, "21 symbols analyzed"),
                (81, "0/4 files"),
                (81, "1/4 files"),
                (81, "2/4 files"),
                (81, "3/4 files"),
                (82, "4/4 files"),
                (85, "Load artifacts saved"),
                (85, "Collateral mapped"),
                (85, "Planning extractions"),
                (86, "Justifying symbols..."),
                (99, "Symbols justified"),
                (100, "Payload assembled"),
            ],
            "stage-5 emission order must be the calibrated milestone sequence"
        );
        for pair in reports.windows(2) {
            assert!(
                pair[0].percent <= pair[1].percent,
                "percent regressed: {} -> {}",
                pair[0].percent,
                pair[1].percent
            );
        }
        for report in reports.iter() {
            assert_eq!(report.stage, 5, "every stage-5 report stays on step 5");
            assert!(
                (80..=100).contains(&report.percent),
                "{}% escapes the stage-5 span",
                report.percent
            );
        }
        let terminals = reports.iter().filter(|r| r.percent == 100).count();
        assert_eq!(
            terminals, 1,
            "only the terminal 'Payload assembled' may show 100%"
        );
        assert_eq!(reports.last().unwrap().message, "Payload assembled");
    }

    /// A4 minor: the replay above proves the helper mapping is right, but a
    /// future edit could still reorder the real `stage5_report` calls in
    /// `build_ui_payload` and the replay would keep passing. Pin the real
    /// call order in the SOURCE TEXT: inside the production section of
    /// ui.rs (everything before the first test attribute), the byte offset
    /// of "Planning extractions" must be strictly before "Justifying
    /// symbols", and both strictly before the guarded
    /// `build_file_justification_with_memo(` call. Reordering the planner
    /// behind the memo — misrepresenting the boot bar — fails here loudly.
    #[test]
    fn test_planning_milestone_precedes_justifying_in_source() {
        let ui_code = include_str!("ui.rs");
        let production_end = ui_code
            .find("#[cfg(test)]")
            .expect("ui.rs must have a test module to cut the production section at");
        let production = &ui_code[..production_end];
        // The production cut already excludes this test's own literals.
        let planning_at = production
            .find("Planning extractions")
            .expect("planning milestone label must be emitted in production code");
        let justifying_at = production
            .find("Justifying symbols")
            .expect("justifying milestone label must be emitted in production code");
        // Needle carries the open paren: the doc comments naming the memo
        // function never match, only the real call site does.
        let justification_call_at = production
            .find("build_file_justification_with_memo(")
            .expect("the guarded justification call must exist in production code");
        assert!(
            planning_at < justifying_at,
            "\"Planning extractions\" (byte {planning_at}) must precede \"Justifying symbols\" (byte {justifying_at}) in ui.rs source — the planner really runs before the memo"
        );
        assert!(
            planning_at < justification_call_at,
            "\"Planning extractions\" (byte {planning_at}) must precede the justification call (byte {justification_call_at})"
        );
        assert!(
            justifying_at < justification_call_at,
            "\"Justifying symbols\" (byte {justifying_at}) must immediately precede the call it guards (byte {justification_call_at})"
        );
    }

    #[test]
    fn test_phase_label_stable_within_stage() {
        let mut phase_by_stage: HashMap<usize, &str> = HashMap::new();
        for (_, phase) in emitted_milestones() {
            let stage = stage_number_of(phase);
            if let Some(previous) = phase_by_stage.insert(stage, phase) {
                assert_eq!(
                    previous, phase,
                    "stage {stage} milestones disagree: {previous:?} vs {phase:?}"
                );
            }
        }
        // Plan A5: the phase-label scan must cover both files of the
        // split, so a `[k/5]` literal can never hide in the payload
        // builders and escape label-stability checking.
        let ui_code =
            format!("{}{}", include_str!("ui.rs"), include_str!("ui_payload.rs"));
        for stage in 1..=STAGES_PER_RUN {
            let prefix = format!("[{stage}/5]");
            let tail = &phase_by_stage[&stage][prefix.len()..];
            let mut haystack = ui_code.as_str();
            while let Some(index) = haystack.find(&prefix) {
                let rest = &haystack[index + prefix.len()..];
                assert!(
                    rest.starts_with(tail),
                    "stage {stage} emits an inconsistent phase label near: {:?}",
                    &rest[..rest.len().min(32)]
                );
                haystack = rest;
            }
        }
    }
}
