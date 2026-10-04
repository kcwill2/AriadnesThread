use crate::commands::progress::{strip_stage_prefix, TerminalStepBars};
use anyhow::{Context, Result};
use ariadne_index::StageFingerprints;
use ariadne_pipeline::{LoadMilestone, Pipeline, ProgressEvent};
use ariadne_ui::{GraphExportPayload, StageInfo, StageStatus, UiState};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod node_maps;

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
// generate_report+detect_cycle_groups 5.6ms · build_include_elements 5.1ms ·
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
    let cycle_groups = include_graph.detect_cycle_groups();
let node_maps = node_maps::build(
        &include_graph,
        &pipeline.discovery.files,
        &crate::registry::default_registry(),
    );
    let include_elements = ariadne_ui::build_include_elements_from_groups(
        &include_graph,
        &closures,
        &cycle_groups,
        &node_maps.languages,
        &node_maps.unit_kinds,
    );
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
    let path_to_id = node_maps::build_path_to_id(&include_graph);
    let file_collateral = build_file_collateral(&path_to_id, &load_report.file_metrics);
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
        symbol_graph: ariadne_ui::build_symbol_elements(&reduced_model, &node_maps.paths),
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

/// Map each target file (by numeric FileId) to the symbol counts contributed
/// by its collateral sources, keeping only files with non-zero contributors.
fn build_file_collateral(
    path_to_id: &HashMap<PathBuf, u64>,
    file_metrics: &[ariadne_graph::LoadMetric],
) -> HashMap<String, HashMap<String, usize>> {
    let mut file_collateral: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for metric in file_metrics {
        let Some(path) = metric.target_file.as_ref().and_then(|p| path_to_id.get(p)) else {
            continue;
        };
        let mut collat = HashMap::new();
        for src in &metric.collateral_sources {
            if src.symbol_count > 0 {
                collat.insert(src.file_id.0.to_string(), src.symbol_count);
            }
        }
        if !collat.is_empty() {
            file_collateral.insert(path.to_string(), collat);
        }
    }
    file_collateral
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

/// Conservative extraction proposals for the payload (I8): runs the same
/// planner as `tool optimize` and applies the identical Conservative filter
/// as `plan_safe_extractions`; display-only, no behavior coupling. Candidate
/// screening consumes the stage's load metrics and closures rather than
/// re-running the load analysis cold (plan load-tail A1), so plan output is
/// byte-equal to the CLI's `plan_extractions` — locked on real C++ fixtures by
/// `tests/integration/tests/m00_facts_path_parity_test.rs`.
fn build_extraction_proposals(
    include_graph: &ariadne_graph::IncludeGraph,
    reduced_model: &ariadne_graph::ReducedSemanticModel,
    discovery_files: &[ariadne_repo::FileRecord],
    tu_file_ids: &[ariadne_repo::FileId],
    file_metrics: &[ariadne_graph::LoadMetric],
    closures: &HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
) -> Vec<ariadne_ui::ExtractionProposalDto> {
    let planner = ariadne_cluster::planner::OptimizationPlanner::with_screening_facts(
        include_graph,
        reduced_model,
        discovery_files,
        tu_file_ids,
        &ariadne_lang_cpp::cpp_aggregation_filter,
        &ariadne_lang_cpp::cpp_target_path_for,
        None,
        ariadne_lang::CyclePolicy::Reject,
        ariadne_cluster::planner::screening_facts_from_metrics(include_graph, file_metrics),
        closures,
    );
    conservative_proposals(planner.plan())
}

/// Pure filter/sort/cap stage (I6: `(delta desc, plan_id asc)`; I3: top 12),
/// kept separate from the planner so the ordering contract is testable
/// without planner machinery.
fn conservative_proposals(
    plans: Vec<ariadne_cluster::plan::OptimizationPlan>,
) -> Vec<ariadne_ui::ExtractionProposalDto> {
    let mut ranked: Vec<_> = plans
        .into_iter()
        .filter(|p| {
            matches!(p.risk, ariadne_cluster::plan::RiskLevel::Conservative)
                && p.collateral_load_before != p.collateral_load_after
        })
        .collect();
    ranked.sort_by(|a, b| {
        proposal_delta(b)
            .cmp(&proposal_delta(a))
            .then(a.id.0.cmp(&b.id.0))
    });
    ranked
        .into_iter()
        .take(ariadne_ui::EXTRACTION_PROPOSAL_MAX)
        .map(|p| extraction_proposal_dto(&p))
        .collect()
}

fn proposal_delta(plan: &ariadne_cluster::plan::OptimizationPlan) -> usize {
    plan.collateral_load_before
        .saturating_sub(plan.collateral_load_after)
}

fn extraction_proposal_dto(
    plan: &ariadne_cluster::plan::OptimizationPlan,
) -> ariadne_ui::ExtractionProposalDto {
    let total_symbols = plan.proposed_extractions.len();
    let mut extractions: Vec<&ariadne_cluster::plan::ProposedExtraction> =
        plan.proposed_extractions.iter().collect();
    extractions.sort_by(|a, b| a.symbol_id.cmp(&b.symbol_id));
    let symbol_names: Vec<String> = extractions
        .into_iter()
        .take(ariadne_ui::EXTRACTION_PROPOSAL_MAX_NAMES)
        .map(|e| e.symbol_name.clone())
        .collect();
    let mut required_headers: Vec<String> = plan
        .optimization_frontier
        .iter()
        .filter(|p| *p != &plan.target_file)
        .map(|p| proposal_basename(p))
        .collect();
    required_headers.sort();
    required_headers.dedup();
    required_headers.truncate(ariadne_ui::EXTRACTION_PROPOSAL_MAX_HEADERS);
    ariadne_ui::ExtractionProposalDto {
        plan_id: plan.id.0.clone(),
        target_file: proposal_basename(&plan.target_file),
        target_file_id: plan.target_file_id.0.to_string(),
        problem: plan.problem.clone(),
        collateral_load_before: plan.collateral_load_before,
        collateral_load_after: plan.collateral_load_after,
        symbol_names,
        total_symbols,
        required_headers,
        consumer_tu_count: plan.validation_required_tus.len(),
        risk: plan.risk.to_string(),
    }
}

/// Basename with the graph crate's whole-path fallback for component-less
/// paths (mirrors dto.rs `path_basename`).
fn proposal_basename(path: &std::path::Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
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
        let load_started = std::time::Instant::now();
        let out = build_ui_payload(&repo_path, make_bar_progress())?;
        crate::commands::progress::print_load_complete(load_started.elapsed());
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

        tokio::task::spawn_blocking(move || {
            let load_started = std::time::Instant::now();
            match build_ui_payload(&boot_repo, boot_progress) {
                Ok(out) => {
                    crate::commands::progress::print_load_complete(load_started.elapsed());
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
mod tests;

#[cfg(test)]
mod progress_contract_tests;

#[cfg(test)]
mod builder_tests;
