use crate::dto::GraphExportPayload;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

pub struct UiState {
    pub repo_root: std::path::PathBuf,
    pub include_graph: ariadne_graph::IncludeGraph,
    pub reduced_model: ariadne_graph::ReducedSemanticModel,
    pub call_graph: ariadne_graph::CallGraph,
    pub payload: GraphExportPayload,
}

pub fn build_router(state: Arc<UiState>) -> axum::Router {
    Router::new()
        .route("/", get(api_index))
        .route("/api/status", get(api_status))
        .route("/api/graph/include", get(api_graph_include))
        .route("/api/graph/symbol", get(api_graph_symbol))
        .route("/api/graph/call", get(api_graph_call))
        .route("/api/source", get(api_source))
        .route("/api/export", get(api_export))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn api_index() -> axum::response::Html<&'static str> {
    axum::response::Html(crate::INDEX_HTML)
}

#[derive(serde::Serialize)]
pub struct StatusResponse {
    pub repo_name: String,
    pub stats: crate::dto::SummaryStats,
}

async fn api_status(State(state): State<Arc<UiState>>) -> Json<StatusResponse> {
    Json(StatusResponse {
        repo_name: state.payload.repo_name.clone(),
        stats: state.payload.stats.clone(),
    })
}

async fn api_graph_include(State(state): State<Arc<UiState>>) -> Json<Vec<crate::dto::CytoscapeElement>> {
    Json(state.payload.include_graph.clone())
}

async fn api_graph_symbol(State(state): State<Arc<UiState>>) -> Json<Vec<crate::dto::CytoscapeElement>> {
    Json(state.payload.symbol_graph.clone())
}

async fn api_graph_call(State(state): State<Arc<UiState>>) -> Json<Vec<crate::dto::CytoscapeElement>> {
    Json(state.payload.call_graph.clone())
}

#[derive(Deserialize)]
struct SourceQuery {
    file: String,
}

async fn api_source(
    State(state): State<Arc<UiState>>,
    Query(query): Query<SourceQuery>,
) -> Result<String, axum::http::StatusCode> {
    let path = state.repo_root.join(&query.file);
    // Be careful with directory traversal in real code, simple for now.
    tokio::fs::read_to_string(path)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)
}

async fn api_export(State(state): State<Arc<UiState>>) -> Json<GraphExportPayload> {
    Json(state.payload.clone())
}
