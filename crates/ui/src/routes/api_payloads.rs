//! AR-13 extraction (verbatim from server.rs): index, bundled UI assets, and
//! prebuilt-payload endpoints (`/`, `/styles.css`, `/app/*.js|css`,
//! `/api/status`, `/api/export`).

use crate::dto::GraphExportPayload;
use crate::state::{read_state, UiState};
use axum::extract::State;
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

pub(crate) async fn api_export(
    State(state): State<Arc<UiState>>,
) -> Result<Json<GraphExportPayload>, axum::http::StatusCode> {
    let payload = read_state(&state.payload);
    payload
        .as_ref()
        .map(|p| Json(p.clone()))
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)
}
