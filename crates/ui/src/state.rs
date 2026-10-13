//! AR-13 extraction (verbatim from server.rs): analysis-generation state.

use crate::dto::GraphExportPayload;
use crate::routes::api_extraction::ExtractionManifestReader;
use crate::routes::api_extraction_apply::ExtractionPlanApplier;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// P5 (AR-15): the `/api/export` payload precomputed once per analysis
/// generation: the serialized bytes plus the `ETag` validator hashed over
/// exactly those bytes. Cloning this snapshot for a response is a
/// refcount bump (`Bytes` is an Arc'd immutable buffer; `Bytes::clone`
/// never copies payload), replacing the per-request deep clone of the
/// `GraphExportPayload` the old `Json(payload.clone())` path paid.
///
/// Validation is the PAIR `(epoch, refresh_key)`:
/// - `epoch` — bumped by every `install_analysis` (the ONLY graph/model/
///   memo swap, called by boot, the watcher rebuild, and the ui tests);
/// - `refresh_key` — the `stage_status.last_refresh` a completed swap
///   installs AFTER the payload write (cli `swap_into_state` order:
///   install -> call_graph -> payload -> stage_status).
/// A swap in flight poisons at most this one snapshot for the microseconds
/// between the epoch bump and the stage-status write, and the key change
/// then forces refiltering — the same bounded torn-swap window
/// `swap_into_state` already documents for direct payload reads. A payload
/// write that signals NEITHER keeps serving the old snapshot; the sole
/// production writer signals both.
pub(crate) struct ExportSnapshot {
    pub(crate) epoch: u64,
    pub(crate) refresh_key: Option<u64>,
    pub(crate) bytes: axum::body::Bytes,
    pub(crate) etag: String,
}

/// P5 (AR-15): the served `ETag` spec — FNV-1a/64 over the payload bytes,
/// lowercase 16-hex, wrapped in RFC-7232 strong-validator quotes. The
/// byte-lock oracle test carries its own reference implementation of this
/// exact scheme. Chosen over a crypto hash because a validator only needs
/// change-detection here: xor-then-multiply with the odd FNV prime is
/// injective per step, so two byte strings differing in a single byte can
/// never share a value — the mutation lock relies on that.
pub(crate) fn payload_bytes_etag(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("\"{hash:016x}\"")
}


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
    /// P5 (AR-15): bumped by every `install_analysis`; see `ExportSnapshot`.
    analysis_epoch: AtomicU64,
    /// P5 (AR-15): last published `/api/export` snapshot (see above).
    export_snapshot: std::sync::RwLock<Option<Arc<ExportSnapshot>>>,
    /// P5 (AR-15): instrumentation for the no-deep-clone lock — counts
    /// payload serializations actually performed (steady requests must
    /// serve snapshot clones, so this stays flat between rebuilds).
    export_serialize_count: AtomicU64,
    /// Wizard reader seam (ui-extraction-wizard W-2b, shape C2): the
    /// host-installed read-only extraction provider. `None` until the cli
    /// install sequence has run; the wizard routes answer 503 until then
    /// (shape C3). The install happens with the generation, in the same
    /// bounded window the payload write already documents above.
    extraction_reader:
        std::sync::RwLock<Option<Arc<dyn ExtractionManifestReader>>>,
    /// Apply seam (extraction-auto-apply A-3, shape C2): the host-installed
    /// mutation delegate, the twin of `extraction_reader`. `None` until the
    /// cli install sequence has run; the apply route answers 503 until then
    /// (shape C3). The install happens with the generation, immediately
    /// after `install_extraction_reader`, inside the same bounded window
    /// the payload write documents above.
    extraction_applier:
        std::sync::RwLock<Option<Arc<dyn ExtractionPlanApplier>>>,
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
            analysis_epoch: AtomicU64::new(0),
            export_snapshot: std::sync::RwLock::new(None),
            export_serialize_count: AtomicU64::new(0),
            extraction_reader: std::sync::RwLock::new(None),
            extraction_applier: std::sync::RwLock::new(None),
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
            analysis_epoch: AtomicU64::new(0),
            export_snapshot: std::sync::RwLock::new(None),
            export_serialize_count: AtomicU64::new(0),
            extraction_reader: std::sync::RwLock::new(None),
            extraction_applier: std::sync::RwLock::new(None),
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
        // P5 (AR-15): the epoch bump marks any published export snapshot
        // stale from the swap onwards (see `ExportSnapshot`).
        self.analysis_epoch.fetch_add(1, Ordering::SeqCst);
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

    /// Install the read-only extraction provider (shape C2). The host (cli)
    /// calls this right after `install_analysis` in the same
    /// `swap_into_state` sequence, so a reader always belongs to the
    /// generation the routes serve; `UiState::new_loading`/`new_loaded`
    /// keep their signatures and start at `None`.
    pub fn install_extraction_reader(&self, reader: Arc<dyn ExtractionManifestReader>) {
        *write_state(&self.extraction_reader) = Some(reader);
    }

    /// Current reader for one request: an `Arc` refcount bump, the same
    /// lifetime guarantee `analysis()` gives the generation.
    pub fn extraction_reader(&self) -> Option<Arc<dyn ExtractionManifestReader>> {
        read_state(&self.extraction_reader).clone()
    }

    /// Install the mutation delegate (extraction-auto-apply shape C2). The
    /// host (cli) calls this right after `install_extraction_reader` in the
    /// same `swap_into_state` sequence, so an applier always belongs to the
    /// generation the routes serve; `UiState::new_loading`/`new_loaded`
    /// keep their signatures and start the field at `None`.
    pub fn install_extraction_applier(&self, applier: Arc<dyn ExtractionPlanApplier>) {
        *write_state(&self.extraction_applier) = Some(applier);
    }

    /// Current applier for one request: an `Arc` refcount bump, the same
    /// lifetime guarantee `analysis()` gives the generation.
    pub fn extraction_applier(&self) -> Option<Arc<dyn ExtractionPlanApplier>> {
        read_state(&self.extraction_applier).clone()
    }

    /// P5 (AR-15): the current analysis epoch (install count).
    pub(crate) fn analysis_epoch(&self) -> u64 {
        self.analysis_epoch.load(Ordering::SeqCst)
    }

    /// P5 (AR-15): the published export snapshot, but only while its
    /// validation pair still matches the live state.
    pub(crate) fn export_snapshot_matching(
        &self,
        epoch: u64,
        refresh_key: Option<u64>,
    ) -> Option<Arc<ExportSnapshot>> {
        let held = read_state(&self.export_snapshot).clone();
        held.filter(|snap| snap.epoch == epoch && snap.refresh_key == refresh_key)
    }

    /// P5 (AR-15): publish a freshly serialized snapshot, re-checking the
    /// validation pair; a swap that started mid-serialization invalidates
    /// the candidate and the next request re-serves from current bytes.
    pub(crate) fn publish_export_snapshot(&self, candidate: Arc<ExportSnapshot>) {
        if self.analysis_epoch() != candidate.epoch {
            return;
        }
        if read_state(&self.stage_status).last_refresh != candidate.refresh_key {
            return;
        }
        *write_state(&self.export_snapshot) = Some(candidate);
    }

    /// P5 (AR-15): records one payload serialization (the slow path).
    pub(crate) fn note_export_serialization(&self) {
        self.export_serialize_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Instrumentation for the no-deep-clone lock (AR-15): serializations
    /// performed so far. Steady polling after the first request per
    /// generation must not move this counter.
    pub fn export_serialization_count(&self) -> u64 {
        self.export_serialize_count.load(Ordering::SeqCst)
    }
}
