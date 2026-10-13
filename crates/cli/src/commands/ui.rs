// `TerminalStepBars` moved out with the bar renderer (A-1): the frozen
// test modules import it from `crate::commands::progress` directly.
use crate::commands::progress::strip_stage_prefix;
use crate::commands::ui::boot_progress::BootStepSnapshot;
use anyhow::{Context, Result};
use ariadne_index::StageFingerprints;
use ariadne_pipeline::{BootMilestone, LoadMilestone, Pipeline, ProgressEvent};
use ariadne_ui::{GraphExportPayload, StageInfo, StageStatus, UiState};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
// Re-exported after the verbatim move to `ui/stage_spans.rs`: EXACTLY the
// names the frozen test modules import via `use super::{...}`, plus every
// name production code here still calls (child modules see parent-private
// `use` bindings, so the frozen import lines stay UNEDITED).
use stage_spans::{
    collateral_file_percent, discovery_hashed_percent, include_scan_percent, stage_end_percent,
    stage_start_percent, step_percent, tu_index_percent, ARTIFACTS_PHASE, ARTIFACTS_SAVED_PERCENT,
    CALL_GRAPH_PHASE, COLLATERAL_PHASE, JUSTIFICATION_COMPLETE_PERCENT, JUSTIFYING_SYMBOLS_PERCENT,
    PAYLOAD_ASSEMBLED_PERCENT, PLANNING_EXTRACTIONS_PERCENT, REDUCTION_PHASE, RUN_PHASES,
    STAGES_PER_RUN, SYMBOLS_ANALYZED_PERCENT, TU_INDEX_PHASE,
};
// Re-exported after the verbatim move to `ui/boot_progress.rs`: EXACTLY the
// names the frozen test modules import via `use super::{...}`, plus every
// name production code here still calls (child modules see parent-private
// `use` bindings, so the frozen import lines stay UNEDITED).
use boot_progress::{
    build_boot_generation, drive_boot_build, make_bar_progress, make_boot_progress,
    surface_boot_failure,
};

mod boot_progress;
mod extraction_reader;
mod node_maps;
mod stage_spans;

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
    /// The plans behind `payload.extraction_proposals` (ui-extraction-wizard
    /// W-2a): the wizard reader resolves plan ids against THIS generation,
    /// so the planner's output that the payload was rendered from travels
    /// with the build result instead of being dropped.
    pub extraction_plans: Vec<ariadne_cluster::plan::OptimizationPlan>,
    /// The `UiState` analysis epoch this generation gets when swapped in
    /// (boot installs epoch 1, the watcher counts up). The export path
    /// never swaps and leaves it at 0.
    pub install_epoch: u64,
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

/// Drive the five artifact-backed stages, emitting start/end milestones that
/// keep every percent inside the span owned by its `[k/5]` step label.
/// I5 (V1): the stage-1 START frame now belongs to `build_ui_payload`; every
/// frame emitted here stays byte-identical in order.
fn run_pipeline_stages(
    pipeline: &mut Pipeline,
    progress: &ProgressFn,
) -> Result<ariadne_graph::ReducedSemanticModel> {
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
        ProgressEvent::BootMilestone(milestone) => (cb_progress)(match milestone {
            BootMilestone::DiscoveryHashed { done, total } => stage_report(
                1,
                discovery_hashed_percent(*done, *total),
                ARTIFACTS_PHASE,
                &format!("Hashing {done} files"),
            ),
            BootMilestone::IncludeScan { done, total } => stage_report(
                1,
                include_scan_percent(*done, *total),
                ARTIFACTS_PHASE,
                &format!("{done}/{total} include scan"),
            ),
        }),
        ProgressEvent::TuStarted { done, total, source } => (cb_progress)(stage_report(
            2,
            tu_index_percent(*done, *total),
            TU_INDEX_PHASE,
            // I10 verbatim literal: double space before the relative path.
            &format!("{done}/{total} TUs  {}", source.display()),
        )),
    }
}

/// One of build_ui_payload's own stage-5 assembly milestones.
fn stage5_report(progress: &ProgressFn, percent: usize, message: &str) {
    progress(stage_report(5, percent, COLLATERAL_PHASE, message));
}

/// Run every pipeline stage (loading artifact-backed stages when their input
/// fingerprint is fresh) and assemble the UI payload from the results.
fn build_ui_payload(repo_path: &PathBuf, progress: ProgressFn) -> Result<UiBuildResult> {
    // I5 frame-1-first (V1): the bar exists before the constructor hashes
    // sources serially. Exactly one such start emission - dedup-locked.
    progress(stage_report(1, stage_start_percent(1), ARTIFACTS_PHASE, "Hashing sources"));
    let progress_cb = stage_progress_callback(progress.clone());
    let mut pipeline = Pipeline::new_with_progress(repo_path, None, Box::new(progress_cb))?;

    let reduced_model = run_pipeline_stages(&mut pipeline, &progress)?;
    // A3: graph/closures MOVE out of the single-use pipeline, no clone.
    let (include_graph, closures) = pipeline.take_include_graph()?;
    let call_graph = pipeline.get_or_build_callgraph()?;
    let load_report = pipeline.get_or_build_load()?;
    let report = include_graph.generate_report();
    let cycle_groups = include_graph.detect_cycle_groups();
    // AR-19 (A1 part 2): one registry per payload build - shared by the node
    // language maps and the registry-dispatched planning hooks below.
    let registry = crate::registry::default_registry();
    let node_maps = node_maps::build(&include_graph, &pipeline.discovery.files, &registry);
    let include_elements = ariadne_ui::build_include_elements(
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
    let (extraction_proposals, extraction_plans) = build_extraction_proposals_and_plans(
        &include_graph, &reduced_model, &pipeline.discovery.files,
        &translation_unit_file_ids(&pipeline.build), &load_report.file_metrics,
        &closures, &registry, &pipeline.repo.root,
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
        extraction_plans,
        install_epoch: 0,
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
/// W-2a: the payload path now calls
/// `build_extraction_proposals_and_plans` directly, leaving this
/// single-output wrapper as the `builder_tests` import surface only.
/// `#[allow(dead_code)]`, not a test-cfg gate: the string lock in
/// `ui/tests.rs` cuts the production section at the first test-cfg
/// attribute line in this file, so an earlier one would hide the
/// bind-honesty contract from that lock.
#[allow(dead_code)]
fn build_extraction_proposals(
    include_graph: &ariadne_graph::IncludeGraph,
    reduced_model: &ariadne_graph::ReducedSemanticModel,
    discovery_files: &[ariadne_repo::FileRecord],
    tu_file_ids: &[ariadne_repo::FileId],
    file_metrics: &[ariadne_graph::LoadMetric],
    closures: &HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    registry: &ariadne_lang::LanguageRegistry,
    repo_root: &std::path::Path,
) -> Vec<ariadne_ui::ExtractionProposalDto> {
    build_extraction_proposals_and_plans(
        include_graph,
        reduced_model,
        discovery_files,
        tu_file_ids,
        file_metrics,
        closures,
        registry,
        repo_root,
    )
    .0
}

/// Same single planner run (m00 parity: this hand adds no second planner
/// run), returning the DTOs AND the plans behind them: the wizard reader
/// (W-2a) resolves plan ids against the generation's own plans, so
/// `conservative_proposals`'s capped survivors travel out with the build
/// result instead of being dropped at the payload boundary.
#[allow(clippy::too_many_arguments)]
fn build_extraction_proposals_and_plans(
    include_graph: &ariadne_graph::IncludeGraph,
    reduced_model: &ariadne_graph::ReducedSemanticModel,
    discovery_files: &[ariadne_repo::FileRecord],
    tu_file_ids: &[ariadne_repo::FileId],
    file_metrics: &[ariadne_graph::LoadMetric],
    closures: &HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    registry: &ariadne_lang::LanguageRegistry,
    repo_root: &std::path::Path,
) -> (
    Vec<ariadne_ui::ExtractionProposalDto>,
    Vec<ariadne_cluster::plan::OptimizationPlan>,
) {
    // AR-19 (A1 part 2): the hooks are the registry dispatch shared with the
    // plan and optimize lanes (crate::registry::planning_hooks); the former
    // positional `ariadne_lang_cpp::cpp_*` references are gone. C++ answers
    // stay byte-identical (trait-hook parity, AR-18; m00 parity UNEDITED).
    let hooks = crate::registry::planning_hooks(registry, repo_root);
    let planner = ariadne_cluster::planner::OptimizationPlanner::with_screening_facts(
        include_graph,
        reduced_model,
        discovery_files,
        tu_file_ids,
        &hooks.aggregation,
        &hooks.target,
        None,
        ariadne_lang::CyclePolicy::Reject,
        ariadne_cluster::planner::screening_facts_from_metrics(include_graph, file_metrics),
        closures,
    );
    conservative_proposals_with_plans(planner.plan())
}

/// Pure filter/sort/cap stage (I6: `(delta desc, plan_id asc)`; I3: top 12),
/// kept separate from the planner so the ordering contract is testable
/// without planner machinery.
/// W-2a: production now goes through
/// `conservative_proposals_with_plans`, leaving this wrapper as the
/// `builder_tests` import surface only (that file is outside this hand).
/// `#[allow(dead_code)]` for the test-cfg-gate reason spelled out on
/// `build_extraction_proposals`.
#[allow(dead_code)]
fn conservative_proposals(
    plans: Vec<ariadne_cluster::plan::OptimizationPlan>,
) -> Vec<ariadne_ui::ExtractionProposalDto> {
    conservative_proposals_with_plans(plans).0
}

/// The same filter/sort/cap, also returning the plans the DTOs were
/// rendered from (same order, same cap): the wizard reader answers
/// exactly those plan ids for this generation.
fn conservative_proposals_with_plans(
    plans: Vec<ariadne_cluster::plan::OptimizationPlan>,
) -> (
    Vec<ariadne_ui::ExtractionProposalDto>,
    Vec<ariadne_cluster::plan::OptimizationPlan>,
) {
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
    let chosen: Vec<ariadne_cluster::plan::OptimizationPlan> = ranked
        .into_iter()
        .take(ariadne_ui::EXTRACTION_PROPOSAL_MAX)
        .collect();
    let dtos = chosen
        .iter()
        .map(|plan| extraction_proposal_dto(plan))
        .collect();
    (dtos, chosen)
}

fn proposal_delta(plan: &ariadne_cluster::plan::OptimizationPlan) -> usize {
    plan.collateral_load_before
        .saturating_sub(plan.collateral_load_after)
}

/// Pure per-file grouping of the plan's symbol -> `target_header` mapping
/// (shape C2): a `BTreeMap` keyed by `target_header` gives path-ascending
/// determinism; each group's symbols are sorted by `symbol_id`, mirroring
/// the `symbol_names` sort in `extraction_proposal_dto`. Names per file are
/// capped at `EXTRACTION_PROPOSAL_MAX_NAMES` (with `symbol_count` keeping
/// the uncapped total, same semantics as `total_symbols`); file rows are
/// capped at `EXTRACTION_PROPOSAL_MAX_TARGET_FILES` and `truncated` is set
/// iff the group count exceeds that cap. No planner, no I/O.
fn target_file_groups(
    extractions: &[ariadne_cluster::plan::ProposedExtraction],
) -> (Vec<ariadne_ui::ProposedTargetFileDto>, bool) {
    let mut groups: std::collections::BTreeMap<
        std::path::PathBuf,
        Vec<&ariadne_cluster::plan::ProposedExtraction>,
    > = std::collections::BTreeMap::new();
    for extraction in extractions {
        groups
            .entry(extraction.target_header.clone())
            .or_default()
            .push(extraction);
    }
    let truncated = groups.len() > ariadne_ui::EXTRACTION_PROPOSAL_MAX_TARGET_FILES;
    let rows = groups
        .into_iter()
        .take(ariadne_ui::EXTRACTION_PROPOSAL_MAX_TARGET_FILES)
        .map(|(path, mut members)| {
            members.sort_by(|a, b| a.symbol_id.cmp(&b.symbol_id));
            let symbol_count = members.len();
            let symbol_names = members
                .into_iter()
                .take(ariadne_ui::EXTRACTION_PROPOSAL_MAX_NAMES)
                .map(|e| e.symbol_name.clone())
                .collect();
            ariadne_ui::ProposedTargetFileDto {
                path: path.to_string_lossy().to_string(),
                symbol_names,
                symbol_count,
            }
        })
        .collect();
    (rows, truncated)
}

fn extraction_proposal_dto(
    plan: &ariadne_cluster::plan::OptimizationPlan,
) -> ariadne_ui::ExtractionProposalDto {
    let total_symbols = plan.proposed_extractions.len();
    let (target_files, target_files_truncated) = target_file_groups(&plan.proposed_extractions);
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
        target_files,
        target_files_truncated,
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
    let reader_epoch = out.install_epoch;
    let reader_plans = out.extraction_plans;
    state.install_analysis(out.include_graph, out.reduced_model, out.load_memo);
    // W-2a (shape C2): the reader belongs to THIS generation - installed in
    // the same sequence right after `install_analysis`, from the plans this
    // payload was rendered from and the generation Arc `install_analysis`
    // just published (zero-copy). A reader can never outlive its generation.
    let reader = extraction_reader::EngineBackedExtractionReader::new(
        reader_epoch,
        state.repo_root.clone(),
        reader_plans,
        state
            .analysis()
            .expect("install_analysis publishes the generation it just installed"),
    );
    state.install_extraction_reader(Arc::new(reader));
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
    // W-2a: boot consumed epoch 1 of the boot `UiState`; each watcher swap
    // installs the next one (only successful swaps reach the counter).
    let next_epoch = Arc::new(AtomicU64::new(2));

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
            let next_epoch = next_epoch.clone();
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
                        Ok(mut out) => {
                            out.install_epoch = next_epoch.fetch_add(1, Ordering::SeqCst);
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

/// S6 (AR-02): the UI binds localhost by default; `--expose <addr>` is the
/// single opt-in to a wider bind address (LAN exposure warns at startup).
pub fn resolve_bind_addr(expose: Option<&str>) -> String {
    match expose {
        Some(addr) if !addr.trim().is_empty() => addr.trim().to_string(),
        _ => "127.0.0.1".to_string(),
    }
}

/// S6: an active `--expose` widens the bind beyond localhost; the R8
/// never-silent rule puts that trade on the console at every such boot.
fn warn_if_expose_used(expose: Option<&str>) {
    let Some(addr) = expose.map(str::trim).filter(|addr| !addr.is_empty()) else {
        return;
    };
    eprintln!(
        "warning: --expose {addr} binds the UI beyond localhost: it has no authentication and is reachable from the local network"
    );
}

pub async fn run_ui(
    repo_opt: Option<PathBuf>,
    port: u16,
    no_open: bool,
    export_opt: Option<PathBuf>,
    watch_interval: u64,
    no_watch: bool,
    expose_opt: Option<String>,
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
        // DP-1 (ui-progress-parity T2): every step row exists from boot;
        // not-yet-started rows are `Pending` until the boot progress
        // closure's first report rewrites them.
        {
            let mut p = write_state(&state.progress);
            p.steps = BootStepSnapshot::seeded(STAGES_PER_RUN, &RUN_PHASES).rows();
            p.total_stages = STAGES_PER_RUN;
        }

        let app = ariadne_ui::build_router(state.clone());
        // S6: localhost-only unless the operator opts in via `--expose`.
        let bind_addr = resolve_bind_addr(expose_opt.as_deref());
        warn_if_expose_used(expose_opt.as_deref());
        let listener = tokio::net::TcpListener::bind(format!("{bind_addr}:{port}")).await?;

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
        let join_state = state.clone();

        // I11: the JoinHandle stays alive (the pre-V1 code dropped it). The
        // bridge makes the build panic-proof; this route covers a double
        // panic or runtime shutdown - if the task died, `fps_tx` died with
        // it, the watcher parks, and the dead boot is surfaced here (R8).
        let boot_handle = tokio::task::spawn_blocking(move || {
            drive_boot_build(&boot_state, fps_tx, || {
                build_boot_generation(&boot_repo, boot_progress)
            });
        });
        tokio::spawn(async move {
            if let Err(join_error) = boot_handle.await {
                surface_boot_failure(&join_state, join_error.to_string());
                tracing::error!("UI boot task failed: {join_error}");
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

// Test-only re-export: the frozen `progress_contract_tests.rs` imports
// `COLLATERAL_FILE_SPAN_END` via `use super::{...}` (no production caller
// here). PLACED after ALL production code: the frozen source-text locks cut
// the production section at the FIRST `#[cfg(test)]` token.
#[cfg(test)]
use stage_spans::COLLATERAL_FILE_SPAN_END;

// Test-only re-export: the frozen `boot_panic_tests.rs` imports
// `make_boot_progress_with_sink` via `use super::{...}`; its only remaining
// callers are that test module and the moved family itself. Same placement
// rule.
#[cfg(test)]
use boot_progress::make_boot_progress_with_sink;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod progress_contract_tests;

#[cfg(test)]
mod builder_tests;

#[cfg(test)]
mod boot_progress_tests;

#[cfg(test)]
mod target_files_tests;

#[cfg(test)]
mod first_frame_tests;

#[cfg(test)]
mod boot_panic_tests;
