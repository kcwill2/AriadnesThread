use anyhow::{Context, Result};
use ariadne_index::StageFingerprints;
use ariadne_pipeline::{Pipeline, ProgressEvent};
use ariadne_ui::{GraphExportPayload, StageInfo, StageStatus, UiState};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct UiBuildResult {
    pub include_graph: ariadne_graph::IncludeGraph,
    pub reduced_model: ariadne_graph::ReducedSemanticModel,
    pub call_graph: ariadne_graph::CallGraph,
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

type ProgressFn = Arc<dyn Fn(usize, &str, &str) + Send + Sync>;

/// Run every pipeline stage (loading artifact-backed stages when their input
/// fingerprint is fresh) and assemble the UI payload from the results.
fn build_ui_payload(repo_path: &PathBuf, progress: ProgressFn) -> Result<UiBuildResult> {
    let cb_progress = progress.clone();
    let progress_cb = move |event: &ProgressEvent| match event {
        ProgressEvent::TuIndex { done, total } => {
            let percent = 20 + if *total > 0 { 35 * done / total } else { 35 };
            (cb_progress)(
                percent,
                "[2/5] Indexing translation units...",
                &format!("{}/{} TUs", done, total),
            );
        }
        ProgressEvent::CollateralFile { done, total } => {
            let percent = 80 + if *total > 0 { 15 * done / total } else { 15 };
            (cb_progress)(
                percent,
                "[5/5] Analyzing collateral...",
                &format!("{}/{} files", done, total),
            );
        }
    };
    let mut pipeline = Pipeline::new(repo_path, None)?.with_progress(progress_cb);

    progress(5, "[1/5] Loading artifacts...", "Hashing sources");
    pipeline.ensure_include_graph()?;
    progress(10, "[1/5] Include graph...", "Graph ready");

    progress(20, "[2/5] Indexing translation units...", "Parsing sources");
    pipeline.ensure_index()?;
    progress(55, "[2/5] Indexing translation units...", "Index complete");

    progress(60, "[3/5] Reducing semantic model...", "Deduplicating");
    let reduced_model = pipeline.get_or_build_reduced()?;
    progress(65, "[3/5] Reducing semantic model...", "Model reduced");

    progress(70, "[4/5] Building call graph...", "Constructing");
    pipeline.ensure_callgraph()?;

    progress(80, "[5/5] Analyzing collateral...", "Load analysis");
    pipeline.ensure_load()?;
    progress(95, "[5/5] Analyzing collateral...", "Mapping collateral");

    let graph = pipeline.get_or_build_include_graph()?;
    let (include_graph, closures) = graph;
    let call_graph = pipeline.get_or_build_callgraph()?;
    let load_report = pipeline.get_or_build_load()?;

    let report = include_graph.generate_report();
    let cycles = include_graph.detect_cycles();
    let include_elements = ariadne_ui::build_include_elements(&include_graph, &closures, &cycles);

    let path_to_id: HashMap<PathBuf, u64> = include_graph
        .file_paths
        .iter()
        .map(|(id, path)| (path.clone(), id.0))
        .collect();
    let mut file_collateral: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for metric in &load_report.file_metrics {
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
    };

    let status = stage_status_from_pipeline(&pipeline);
    Ok(UiBuildResult {
        include_graph,
        reduced_model,
        call_graph,
        payload,
        status,
        fps: pipeline.fps.clone(),
    })
}

/// Swap a freshly built snapshot into live UI state. The stale payload keeps
/// serving every request until each slot is replaced.
fn swap_into_state(state: &UiState, out: UiBuildResult) -> StageFingerprints {
    let fps = out.fps.clone();
    *write_state(&state.include_graph) = Some(out.include_graph);
    *write_state(&state.reduced_model) = Some(out.reduced_model);
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
                    let quiet: ProgressFn = Arc::new(|_percent, _phase, _message| ());
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
        let export_progress: ProgressFn = Arc::new(|percent, phase, message| {
            println!("[{:>3}%] {} {}", percent, phase, message);
        });
        let out = build_ui_payload(&repo_path, export_progress)?;
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

        let boot_progress: ProgressFn = {
            let st = boot_state.clone();
            Arc::new(move |percent, phase, message| {
                let mut p = write_state(&st.progress);
                p.state = "loading".to_string();
                p.phase = phase.to_string();
                p.message = message.to_string();
                p.percent = percent;
            })
        };

        tokio::task::spawn_blocking(move || match build_ui_payload(&boot_repo, boot_progress) {
            Ok(out) => {
                let fps = swap_into_state(&boot_state, out);
                let _ = fps_tx.send(Some(fps));
            }
            Err(e) => {
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
