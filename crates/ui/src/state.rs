//! AR-13 extraction (verbatim from server.rs): analysis-generation state.

use crate::dto::GraphExportPayload;
use std::sync::Arc;

/// Poison recovery: a poisoned lock only means some other thread already
/// panicked while holding it; the data is still memory-consistent, so the UI
/// keeps serving instead of cascading panics into every request handler.
pub(crate) fn read_state<'a, T>(lock: &'a std::sync::RwLock<T>) -> std::sync::RwLockReadGuard<'a, T> {
    lock.read().unwrap_or_else(|e| e.into_inner())
}

/// Poison recovery twin for writers: same reasoning as `read_state`.
pub(crate) fn write_state<'a, T>(lock: &'a std::sync::RwLock<T>) -> std::sync::RwLockWriteGuard<'a, T> {
    lock.write().unwrap_or_else(|e| e.into_inner())
}

/// One analysis generation: the include graph, the reduced model, and the
/// ONE warm `LoadMemo` that serves every request until the next rebuild.
/// The memo keys on FileId/SymbolId, which are generation-local, so memo
/// lifetime == analysis generation: swapping this struct (never its parts)
/// is what makes stale memo entries unobservable (plan C1, I5).
pub struct AnalysisGeneration {
    pub include_graph: ariadne_graph::IncludeGraph,
    pub reduced_model: ariadne_graph::ReducedSemanticModel,
    pub memo: Arc<ariadne_graph::LoadMemo>,
}

pub struct UiState {
    pub repo_root: std::path::PathBuf,
    /// `{graph, model, memo}` swap as ONE unit: rebuilds install a fresh
    /// `Arc<AnalysisGeneration>` under this single lock (`install_analysis`),
    /// so no request can pair one generation's memo with another's graph.
    pub analysis: std::sync::RwLock<Option<Arc<AnalysisGeneration>>>,
    pub call_graph: std::sync::RwLock<Option<ariadne_graph::CallGraph>>,
    pub payload: std::sync::RwLock<Option<GraphExportPayload>>,
    pub progress: std::sync::RwLock<crate::dto::ServerProgress>,
    pub stage_status: std::sync::RwLock<crate::dto::StageStatus>,
}

impl UiState {
    pub fn new_loading(repo_root: std::path::PathBuf) -> Self {
        Self {
            repo_root,
            analysis: std::sync::RwLock::new(None),
            call_graph: std::sync::RwLock::new(None),
            payload: std::sync::RwLock::new(None),
            progress: std::sync::RwLock::new(crate::dto::ServerProgress {
                state: "loading".to_string(),
                phase: "Initializing".to_string(),
                percent: 0,
                message: "Server started".to_string(),
                steps: Vec::new(),
                total_stages: 0,
            }),
            stage_status: std::sync::RwLock::new(crate::dto::StageStatus::default()),
        }
    }

    pub fn new_loaded(
        repo_root: std::path::PathBuf,
        include_graph: ariadne_graph::IncludeGraph,
        reduced_model: ariadne_graph::ReducedSemanticModel,
        call_graph: ariadne_graph::CallGraph,
        payload: GraphExportPayload,
    ) -> Self {
        let state = Self {
            repo_root,
            analysis: std::sync::RwLock::new(None),
            call_graph: std::sync::RwLock::new(Some(call_graph)),
            payload: std::sync::RwLock::new(Some(payload)),
            progress: std::sync::RwLock::new(crate::dto::ServerProgress {
                state: "ready".to_string(),
                phase: "Done".to_string(),
                percent: 100,
                message: "Ready".to_string(),
                steps: Vec::new(),
                total_stages: 0,
            }),
            stage_status: std::sync::RwLock::new(crate::dto::StageStatus::default()),
        };
        // Cold memo; the live server path installs a closure-seeded memo via
        // `install_analysis` instead (cli boot seeds `LoadMemo::with_closures`
        // from the pipeline stage output).
        state.install_analysis(include_graph, reduced_model, Arc::new(ariadne_graph::LoadMemo::new()));
        state
    }

    /// Swap a whole analysis generation in one lock-protected assignment —
    /// THE only way graph/model/memo change after boot (cli boot + watcher
    /// rebuild both call this, mirroring `swap_into_state`). Dropping the
    /// `Arc` here detaches the old memo with its generation (I5: no stale
    /// entry window; no new lock layer).
    pub fn install_analysis(
        &self,
        include_graph: ariadne_graph::IncludeGraph,
        reduced_model: ariadne_graph::ReducedSemanticModel,
        memo: Arc<ariadne_graph::LoadMemo>,
    ) {
        let generation = Arc::new(AnalysisGeneration {
            include_graph,
            reduced_model,
            memo,
        });
        *write_state(&self.analysis) = Some(generation);
    }

    /// Current generation for one request: an `Arc` refcount bump, after
    /// which the graph/model/memo stay alive for the whole request even if
    /// a rebuild swaps in a newer generation mid-flight.
    pub fn analysis(&self) -> Option<Arc<AnalysisGeneration>> {
        read_state(&self.analysis).clone()
    }
}
