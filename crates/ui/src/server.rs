//! AR-13 shell: the router wiring, CORS predicate, and re-exports that keep
//! the crate's pub surface identical after the route-family extraction
//! (`state.rs`, `routes/api_payloads.rs`, `routes/api_source.rs`,
//! `routes/api_graph.rs`). The route table is the pre-split one plus the
//! branding-assets `/icon.svg` route (ui_smoke is the authority).

pub use crate::routes::api_graph::{
    api_graph_collateral, api_graph_traversal, api_justification, CollateralQuery,
    JustificationQuery, TraversalApiQuery,
};
pub use crate::routes::api_payloads::StatusResponse;
pub use crate::routes::api_source::RepoInfoResponse;
pub use crate::state::{AnalysisGeneration, UiState};

use crate::routes::api_extraction::{api_extraction_dry_run, api_extraction_manifest};
use crate::routes::api_graph::{
    api_graph_call, api_graph_entrypoints, api_graph_include, api_graph_path, api_graph_search,
    api_graph_symbol, api_graph_traverse,
};
use crate::routes::api_payloads::{
    api_export, api_index, api_status, asset_api_js, asset_file_view_js, asset_graph_js,
    asset_hotspots_js, asset_icon_svg, asset_main_js, asset_preview_js, asset_provenance_js,
    asset_styles_css, asset_theme_js, asset_themes_css, asset_traversal_js,
};
use crate::routes::api_source::{api_repo_info, api_source};
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// S6 (AR-02): same-origin CORS. A cross-origin request is granted only when
/// its Origin exactly equals the scheme + Host it was sent to; every other
/// origin gets no Access-Control-Allow-Origin header at all, so a foreign
/// page cannot read this server's data through a browser.
fn is_same_origin(origin: &axum::http::HeaderValue, parts: &axum::http::request::Parts) -> bool {
    let Ok(origin_text) = origin.to_str() else {
        return false;
    };
    let Some(host_text) = parts
        .headers
        .get(axum::http::header::HOST)
        .and_then(|host| host.to_str().ok())
    else {
        return false;
    };
    origin_text == format!("http://{host_text}") || origin_text == format!("https://{host_text}")
}

pub fn build_router(state: Arc<UiState>) -> axum::Router {
    Router::new()
        .route("/", get(api_index))
        .route("/styles.css", get(asset_styles_css))
        .route("/themes.css", get(asset_themes_css))
        .route("/icon.svg", get(asset_icon_svg))
        .route("/app/api.js", get(asset_api_js))
        .route("/app/graph.js", get(asset_graph_js))
        .route("/app/traversal.js", get(asset_traversal_js))
        .route("/app/file-view.js", get(asset_file_view_js))
        .route("/app/hotspots.js", get(asset_hotspots_js))
        .route("/app/preview.js", get(asset_preview_js))
        .route("/app/provenance.js", get(asset_provenance_js))
        .route("/app/main.js", get(asset_main_js))
        .route("/app/theme.js", get(asset_theme_js))
        .route("/api/status", get(api_status))
        .route("/api/repo-info", get(api_repo_info))
        .route("/api/graph/include", get(api_graph_include))
        .route("/api/graph/symbol", get(api_graph_symbol))
        .route("/api/graph/call", get(api_graph_call))
        .route("/api/graph/traverse", get(api_graph_traverse))
        .route("/api/graph/path", get(api_graph_path))
        .route("/api/graph/entrypoints", get(api_graph_entrypoints))
        .route("/api/graph/search", get(api_graph_search))
        .route("/api/graph/collateral", get(api_graph_collateral))
        .route("/api/graph/traversal", get(api_graph_traversal))
        .route("/api/justification", get(api_justification))
        .route("/api/source", get(api_source))
        .route("/api/export", get(api_export))
        .route("/api/extraction/:plan_id/manifest", get(api_extraction_manifest))
        .route("/api/extraction/:plan_id/dry-run", get(api_extraction_dry_run))
        .layer(CorsLayer::new().allow_origin(AllowOrigin::predicate(is_same_origin)))
        .with_state(state)
}
