use anyhow::{Context, Result};
use ariadne_index::StageFingerprints;
use ariadne_pipeline::{Pipeline, ProgressEvent};
use ariadne_ui::{GraphExportPayload, StageInfo, StageStatus, UiState};
use crate::commands::progress::{strip_stage_prefix, TerminalStepBars};
use std::collections::HashMap;
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
/// Stage 5 reserves 95→100 for payload assembly; the assembly milestone
/// finalizes the step-5 terminal bar at 100%.
const COLLATERAL_MAPPED_PERCENT: usize = 95;
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

fn tu_index_percent(done: usize, total: usize) -> usize {
    let (lower, upper) = (stage_start_percent(2), stage_end_percent(2));
    if total == 0 {
        return lower;
    }
    (lower + (upper - lower) * done / total).min(upper)
}

fn collateral_file_percent(done: usize, total: usize) -> usize {
    let (lower, upper) = (stage_start_percent(5), COLLATERAL_MAPPED_PERCENT);
    if total == 0 {
        return lower;
    }
    (lower + (upper - lower) * done / total).min(upper)
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
    progress(stage_report(
        5,
        COLLATERAL_MAPPED_PERCENT,
        COLLATERAL_PHASE,
        "Collateral mapped",
    ));
    Ok(reduced_model)
}

/// Run every pipeline stage (loading artifact-backed stages when their input
/// fingerprint is fresh) and assemble the UI payload from the results.
fn build_ui_payload(repo_path: &PathBuf, progress: ProgressFn) -> Result<UiBuildResult> {
    let cb_progress = progress.clone();
    let progress_cb = move |event: &ProgressEvent| match event {
        ProgressEvent::TuIndex { done, total } => (cb_progress)(stage_report(
            2,
            tu_index_percent(*done, *total),
            TU_INDEX_PHASE,
            &format!("{}/{} TUs", done, total),
        )),
        ProgressEvent::CollateralFile { done, total } => (cb_progress)(stage_report(
            5,
            collateral_file_percent(*done, *total),
            COLLATERAL_PHASE,
            &format!("{}/{} files", done, total),
        )),
    };
    let mut pipeline = Pipeline::new(repo_path, None)?.with_progress(progress_cb);

    let reduced_model = run_pipeline_stages(&mut pipeline, &progress)?;

    let graph = pipeline.get_or_build_include_graph()?;
    let (include_graph, closures) = graph;
    let call_graph = pipeline.get_or_build_callgraph()?;
    let load_report = pipeline.get_or_build_load()?;

    let report = include_graph.generate_report();
    let cycles = include_graph.detect_cycles();
    let include_elements = ariadne_ui::build_include_elements(&include_graph, &closures, &cycles);

    // C1: ONE memo for the whole generation, seeded with the stage closures
    // (`with_closures` pattern of `ensure_load`). Payload assembly and every
    // live route share it; rebuilds swap graph+model+memo as one unit.
    let load_memo = Arc::new(ariadne_graph::LoadMemo::with_closures(closures));

    let path_to_id = build_path_to_id(&include_graph);
    let file_collateral = build_file_collateral(&path_to_id, &load_report.file_metrics);
    let file_justification = ariadne_ui::build_file_justification_with_memo(
        &load_report.file_metrics,
        &include_graph,
        &reduced_model,
        &path_to_id,
        Some(&load_memo),
    );
    let extraction_proposals = build_extraction_proposals(
        &include_graph,
        &reduced_model,
        &pipeline.discovery.files,
        &translation_unit_file_ids(&pipeline.build),
    );

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
    progress(stage_report(
        5,
        PAYLOAD_ASSEMBLED_PERCENT,
        COLLATERAL_PHASE,
        "Payload assembled",
    ));
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

/// Shared path→numeric-FileId lookup used by every per-file payload builder.
fn build_path_to_id(include_graph: &ariadne_graph::IncludeGraph) -> HashMap<PathBuf, u64> {
    include_graph
        .file_paths
        .iter()
        .map(|(id, path)| (path.clone(), id.0))
        .collect()
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
/// as `plan_safe_extractions`; display-only, no behavior coupling.
fn build_extraction_proposals(
    include_graph: &ariadne_graph::IncludeGraph,
    reduced_model: &ariadne_graph::ReducedSemanticModel,
    discovery_files: &[ariadne_repo::FileRecord],
    tu_file_ids: &[ariadne_repo::FileId],
) -> Vec<ariadne_ui::ExtractionProposalDto> {
    let planner = ariadne_cluster::planner::OptimizationPlanner::new(
        include_graph, reduced_model, discovery_files, tu_file_ids,
        &ariadne_lang_cpp::cpp_aggregation_filter, &ariadne_lang_cpp::cpp_target_path_for,
        None, ariadne_lang::CyclePolicy::Reject,
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
    }
}

#[cfg(test)]
mod progress_contract_tests {
    use super::{
        collateral_file_percent, stage_end_percent, stage_start_percent, step_percent,
        tu_index_percent, ARTIFACTS_PHASE, CALL_GRAPH_PHASE, COLLATERAL_MAPPED_PERCENT,
        COLLATERAL_PHASE, REDUCTION_PHASE, STAGES_PER_RUN, TU_INDEX_PHASE,
    };
    use std::collections::HashMap;

    const POINTS_PER_STAGE: usize = 100 / STAGES_PER_RUN;

    /// The ordered static milestones exactly as `build_ui_payload` emits them,
    /// built from the same constants and helpers the real code path calls.
    /// Stage 5's milestone set is {80 start, 80–95 event span, 95 "Collateral
    /// mapped", 100 payload finalization}, all within [80, 100].
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
            (COLLATERAL_MAPPED_PERCENT, COLLATERAL_PHASE),
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
            80,
            "empty collateral set must sit at stage start"
        );
        assert_eq!(collateral_file_percent(0, 7), 80);
        assert_eq!(collateral_file_percent(7, 7), 95);
        let mut previous = collateral_file_percent(0, 9);
        for done in 1..=9 {
            let percent = collateral_file_percent(done, 9);
            assert!((80..=95).contains(&percent), "{done}/9 mapped to {percent}%");
            assert!(percent >= previous, "percent regressed at {done}/9");
            previous = percent;
        }
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
        let ui_code = include_str!("ui.rs");
        for stage in 1..=STAGES_PER_RUN {
            let prefix = format!("[{stage}/5]");
            let tail = &phase_by_stage[&stage][prefix.len()..];
            let mut haystack = ui_code;
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

#[cfg(test)]
mod builder_tests {
    use super::{build_extraction_proposals, conservative_proposals};
    use ariadne_cluster::plan::{OptimizationPlan, PlanId, ProposedExtraction, RiskLevel};
    use ariadne_graph::{IncludeGraph, ReducedSemanticModel};
    use ariadne_index::types::SymbolId;
    use ariadne_repo::FileId;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn plan_with(id: &str, before: usize, after: usize, risk: RiskLevel) -> OptimizationPlan {
        OptimizationPlan {
            id: PlanId(id.to_string()),
            target_file: PathBuf::from("include/graph.h"),
            target_file_id: FileId(7),
            problem: format!("{before} symbols of collateral load"),
            collateral_load_before: before,
            collateral_load_after: after,
            proposed_extractions: vec![ProposedExtraction {
                symbol_name: "Thing".to_string(),
                symbol_id: SymbolId::new("sym_1"),
                target_header: PathBuf::from("graph_types.h"),
            }],
            optimization_frontier: vec![
                PathBuf::from("inc/graphics.h"),
                PathBuf::from("inc/config.h"),
            ],
            risk,
            validation_required_tus: vec![PathBuf::from("a.cc"), PathBuf::from("b.cc")],
            rationale: String::new(),
        }
    }

    #[test]
    fn test_conservative_plan_maps_every_dto_field() {
        let out = conservative_proposals(vec![plan_with(
            "PLAN-001",
            300,
            40,
            RiskLevel::Conservative,
        )]);
        assert_eq!(out.len(), 1);
        let dto = &out[0];
        assert_eq!(dto.plan_id, "PLAN-001");
        assert_eq!(dto.target_file, "graph.h");
        assert_eq!(dto.target_file_id, "7");
        assert_eq!(dto.problem, "300 symbols of collateral load");
        assert_eq!(dto.collateral_load_before, 300);
        assert_eq!(dto.collateral_load_after, 40);
        assert_eq!(dto.symbol_names, vec!["Thing".to_string()]);
        assert_eq!(dto.total_symbols, 1);
        assert_eq!(
            dto.required_headers,
            vec!["config.h".to_string(), "graphics.h".to_string()]
        );
        assert_eq!(dto.consumer_tu_count, 2);
        assert_eq!(dto.risk, "Conservative");
    }

    #[test]
    fn test_moderate_and_aggressive_plans_are_excluded() {
        let out = conservative_proposals(vec![
            plan_with("PLAN-001", 100, 10, RiskLevel::Moderate),
            plan_with("PLAN-002", 200, 20, RiskLevel::Conservative),
            plan_with("PLAN-003", 300, 30, RiskLevel::Aggressive),
            plan_with("PLAN-004", 400, 40, RiskLevel::Conservative),
        ]);
        let ids: Vec<&str> = out.iter().map(|p| p.plan_id.as_str()).collect();
        assert_eq!(ids, vec!["PLAN-004", "PLAN-002"]);
        assert!(
            out.iter().all(|p| p.risk == "Conservative"),
            "I8: only Conservative plans may reach the payload"
        );
    }

    #[test]
    fn test_cap_keeps_top_twelve_and_drops_lowest_delta() {
        let plans: Vec<OptimizationPlan> = (1..=13)
            .map(|n| {
                let mut p = plan_with(
                    &format!("PLAN-{n:02}"),
                    n * 100,
                    n * 10,
                    RiskLevel::Conservative,
                );
                p.collateral_load_after = 0;
                p
            })
            .collect();
        let out = conservative_proposals(plans);
        assert_eq!(out.len(), 12);
        assert_eq!(out[0].plan_id, "PLAN-13");
        assert_eq!(out[11].plan_id, "PLAN-02");
        assert!(
            out.iter().all(|p| p.plan_id != "PLAN-01"),
            "the lowest-delta plan must be the one dropped by the cap"
        );
    }

    #[test]
    fn test_equal_deltas_sorted_by_plan_id_ascending() {
        let out = conservative_proposals(vec![
            plan_with("PLAN-002", 500, 100, RiskLevel::Conservative),
            plan_with("PLAN-001", 500, 100, RiskLevel::Conservative),
        ]);
        assert_eq!(
            out.iter().map(|p| p.plan_id.as_str()).collect::<Vec<_>>(),
            vec!["PLAN-001", "PLAN-002"]
        );
    }

    #[test]
    fn test_symbol_names_capped_at_twelve_with_true_total() {
        let mut p = plan_with("PLAN-001", 900, 100, RiskLevel::Conservative);
        p.proposed_extractions = (1..=15)
            .map(|n| ProposedExtraction {
                symbol_name: format!("Sym{n:02}"),
                symbol_id: SymbolId::new(format!("sym_{n:02}")),
                target_header: PathBuf::from("graph_types.h"),
            })
            .collect();
        let out = conservative_proposals(vec![p]);
        assert_eq!(out[0].symbol_names.len(), 12);
        assert_eq!(out[0].symbol_names[0], "Sym01");
        assert_eq!(out[0].symbol_names[11], "Sym12");
        assert_eq!(out[0].total_symbols, 15);
    }

    #[test]
    fn test_mapping_is_deterministic() {
        let plans = || {
            vec![
                plan_with("PLAN-002", 400, 40, RiskLevel::Moderate),
                plan_with("PLAN-001", 500, 50, RiskLevel::Conservative),
                plan_with("PLAN-003", 500, 50, RiskLevel::Conservative),
            ]
        };
        assert_eq!(conservative_proposals(plans()), conservative_proposals(plans()));
    }

    #[test]
    fn test_planner_wiring_on_empty_inputs_yields_no_proposals() {
        let model = ReducedSemanticModel {
            repo_root: PathBuf::from("."),
            symbols: BTreeMap::new(),
            edges: vec![],
            tu_usage: BTreeMap::new(),
            file_symbols: BTreeMap::new(),
        };
        let out = build_extraction_proposals(&IncludeGraph::new(), &model, &[], &[]);
        assert!(out.is_empty());
    }

    #[test]
    fn test_zero_collateral_proposals_are_filtered() {
        let out = conservative_proposals(vec![
            plan_with("PLAN-001", 100, 100, RiskLevel::Conservative),
            plan_with("PLAN-002", 200, 50, RiskLevel::Conservative),
            plan_with("PLAN-003", 300, 300, RiskLevel::Conservative),
            plan_with("PLAN-004", 400, 40, RiskLevel::Conservative),
        ]);
        let ids: Vec<&str> = out.iter().map(|p| p.plan_id.as_str()).collect();
        assert_eq!(ids, vec!["PLAN-004", "PLAN-002"]);
        assert!(
            out.iter().all(|p| p.collateral_load_before != p.collateral_load_after),
            "Zero-collateral proposals (before == after) must be filtered out"
        );
    }

    #[test]
    fn test_self_reference_removed_from_required_headers() {
        let mut plan = plan_with("PLAN-001", 300, 40, RiskLevel::Conservative);
        plan.target_file = PathBuf::from("include/graph.h");
        plan.optimization_frontier = vec![
            PathBuf::from("include/graph.h"),
            PathBuf::from("inc/graphics.h"),
            PathBuf::from("inc/config.h"),
        ];
        let out = conservative_proposals(vec![plan]);
        assert_eq!(out.len(), 1);
        let dto = &out[0];
        assert_eq!(
            dto.required_headers,
            vec!["config.h".to_string(), "graphics.h".to_string()]
        );
        assert!(
            !dto.required_headers.contains(&"graph.h".to_string()),
            "Target file should not appear in its own required_headers"
        );
    }
}
