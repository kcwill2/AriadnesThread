//! AR-13 extraction (verbatim from server.rs): index, bundled UI assets, and
//! prebuilt-payload endpoints (`/`, `/styles.css`, `/app/*.js|css`,
//! `/api/status`, `/api/export`).

use crate::state::{payload_bytes_etag, read_state, ExportSnapshot, UiState};
use axum::extract::State;
use axum::http::header;
use axum::Json;
use std::sync::Arc;

pub(crate) async fn api_index() -> axum::response::Response {
    // UI assets change between server builds; stale cached HTML silently hides
    // new features, so browsers must never reuse a previous copy of the page.
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CACHE_CONTROL, "no-store")
        .header(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(axum::body::Body::from(crate::INDEX_HTML))
        // R5: infallible — static status/body pair
        .unwrap()
}

fn ui_asset(content: &'static str, content_type: &'static str) -> axum::response::Response {
    // Same no-store rule as the index route: every UI asset is rebuilt into the
    // binary, so a cached copy can never match the running server's build.
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CACHE_CONTROL, "no-store")
        .header(axum::http::header::CONTENT_TYPE, content_type)
        .body(axum::body::Body::from(content))
        // R5: infallible — static status/body pair
        .unwrap()
}

pub(crate) async fn asset_styles_css() -> axum::response::Response {
    ui_asset(crate::STYLES_CSS, "text/css; charset=utf-8")
}

pub(crate) async fn asset_themes_css() -> axum::response::Response {
    ui_asset(crate::THEMES_CSS, "text/css; charset=utf-8")
}

pub(crate) async fn asset_icon_svg() -> axum::response::Response {
    ui_asset(crate::ICON_SVG, "image/svg+xml")
}

pub(crate) async fn asset_api_js() -> axum::response::Response {
    ui_asset(crate::API_JS, "text/javascript; charset=utf-8")
}

pub(crate) async fn asset_graph_js() -> axum::response::Response {
    ui_asset(crate::GRAPH_JS, "text/javascript; charset=utf-8")
}

pub(crate) async fn asset_traversal_js() -> axum::response::Response {
    ui_asset(crate::TRAVERSAL_JS, "text/javascript; charset=utf-8")
}

pub(crate) async fn asset_file_view_js() -> axum::response::Response {
    ui_asset(crate::FILE_VIEW_JS, "text/javascript; charset=utf-8")
}

pub(crate) async fn asset_hotspots_js() -> axum::response::Response {
    ui_asset(crate::HOTSPOTS_JS, "text/javascript; charset=utf-8")
}

pub(crate) async fn asset_preview_js() -> axum::response::Response {
    ui_asset(crate::PREVIEW_JS, "text/javascript; charset=utf-8")
}

pub(crate) async fn asset_provenance_js() -> axum::response::Response {
    ui_asset(crate::PROVENANCE_JS, "text/javascript; charset=utf-8")
}

pub(crate) async fn asset_main_js() -> axum::response::Response {
    ui_asset(crate::MAIN_JS, "text/javascript; charset=utf-8")
}

pub(crate) async fn asset_theme_js() -> axum::response::Response {
    ui_asset(crate::THEME_JS, "text/javascript; charset=utf-8")
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StatusResponse {
    pub repo_name: Option<String>,
    pub stats: Option<crate::dto::SummaryStats>,
    pub progress: crate::dto::ServerProgress,
    pub stages: Vec<crate::dto::StageInfo>,
    pub last_refresh: Option<u64>,
    pub rebuilding: bool,
    pub reused_tus: usize,
    pub reanalyzed_tus: usize,
}

pub(crate) async fn api_status(State(state): State<Arc<UiState>>) -> Json<StatusResponse> {
    let payload = read_state(&state.payload);
    let progress = read_state(&state.progress).clone();
    let stage_status = read_state(&state.stage_status).clone();

    Json(StatusResponse {
        repo_name: payload.as_ref().map(|p| p.repo_name.clone()),
        stats: payload.as_ref().map(|p| p.stats.clone()),
        progress,
        stages: stage_status.stages,
        last_refresh: stage_status.last_refresh,
        rebuilding: stage_status.rebuilding,
        reused_tus: stage_status.reused_tus,
        reanalyzed_tus: stage_status.reanalyzed_tus,
    })
}

/// P5 (AR-15): serves the export payload from the generation snapshot
/// (see `state::ExportSnapshot`). Body bytes, content type, and the 503
/// while loading are byte-identical to the removed `Json(payload.clone())`
/// path (characterization-locked in `payload_etag`); the only additive
/// change is the `ETag` validator. Conditional requests are exact-match
/// only: any other `If-None-Match` value — lists, `*`, weak tags — gets
/// the full 200 body, byte-identical to the first-request path.
pub(crate) async fn api_export(
    State(state): State<Arc<UiState>>,
    headers: axum::http::HeaderMap,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    let snapshot = current_export_snapshot(&state)?;
    let if_none_match = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok());
    Ok(export_response(&snapshot, if_none_match))
}

/// Snapshot hit = an `Arc` refcount bump over the shared `Bytes`; a miss
/// serializes the CURRENT payload once through the shared reference (the
/// only buffer is the wire bytes themselves — no payload deep clone) and
/// publishes it for every later request of this generation.
fn current_export_snapshot(
    state: &Arc<UiState>,
) -> Result<Arc<ExportSnapshot>, axum::http::StatusCode> {
    let epoch = state.analysis_epoch();
    let refresh_key = read_state(&state.stage_status).last_refresh;
    if let Some(hit) = state.export_snapshot_matching(epoch, refresh_key) {
        return Ok(hit);
    }
    let payload = read_state(&state.payload);
    let payload_ref = payload
        .as_ref()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
    state.note_export_serialization();
    let bytes = axum::body::Bytes::from(
        serde_json::to_vec(payload_ref)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?,
    );
    drop(payload);
    let etag = payload_bytes_etag(&bytes);
    let snapshot = Arc::new(ExportSnapshot {
        epoch,
        refresh_key,
        bytes,
        etag,
    });
    state.publish_export_snapshot(Arc::clone(&snapshot));
    Ok(snapshot)
}

fn export_response(
    snapshot: &ExportSnapshot,
    if_none_match: Option<&str>,
) -> axum::response::Response {
    let etag_header = axum::http::HeaderValue::from_str(&snapshot.etag)
        // The validator is quoted lowercase hex by construction.
        .expect("payload ETag must be a valid header value");
    if if_none_match == Some(snapshot.etag.as_str()) {
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_MODIFIED)
            .header(header::ETAG, etag_header)
            .body(axum::body::Body::empty())
            // R5: infallible — status + static validator + empty body
            .unwrap();
    }
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ETAG, etag_header)
        .body(axum::body::Body::from(snapshot.bytes.clone()))
        // R5: infallible — static status/header pair over owned bytes
        .unwrap()
}
