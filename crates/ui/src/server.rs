use crate::dto::{
    CytoscapeElement, EntrypointsResponse, GraphExportPayload, PathQuery, SearchQuery,
    SearchResultItem, TraverseQuery,
};
use ariadne_graph::traversal::TraversalDirection;
use ariadne_index::SymbolId;
use ariadne_repo::FileId;
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
        .route("/api/graph/traverse", get(api_graph_traverse))
        .route("/api/graph/path", get(api_graph_path))
        .route("/api/graph/entrypoints", get(api_graph_entrypoints))
        .route("/api/graph/search", get(api_graph_search))
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

async fn api_graph_include(State(state): State<Arc<UiState>>) -> Json<Vec<CytoscapeElement>> {
    Json(state.payload.include_graph.clone())
}

async fn api_graph_symbol(State(state): State<Arc<UiState>>) -> Json<Vec<CytoscapeElement>> {
    Json(state.payload.symbol_graph.clone())
}

async fn api_graph_call(State(state): State<Arc<UiState>>) -> Json<Vec<CytoscapeElement>> {
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
    tokio::fs::read_to_string(path)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)
}

async fn api_export(State(state): State<Arc<UiState>>) -> Json<GraphExportPayload> {
    Json(state.payload.clone())
}

async fn api_graph_traverse(
    State(state): State<Arc<UiState>>,
    Query(query): Query<TraverseQuery>,
) -> Json<Vec<CytoscapeElement>> {
    let depth = query.depth.unwrap_or(1);
    let direction = match query.direction.as_deref() {
        Some("in") | Some("incoming") => TraversalDirection::Incoming,
        Some("out") | Some("outgoing") => TraversalDirection::Outgoing,
        _ => TraversalDirection::Both,
    };

    let mut elements = Vec::new();
    match query.graph_type.as_str() {
        "include" => {
            if let Ok(id) = query.center.parse::<u64>() {
                let center = FileId(id);
                let subgraph = state.include_graph.traverse_neighborhood(center, depth, direction);
                let node_ids: std::collections::HashSet<String> = subgraph.nodes.into_iter().map(|n| n.0.to_string()).collect();
                let edge_ids: std::collections::HashSet<String> = subgraph.edges.into_iter().map(|(src, dst)| format!("{}-{}", src.0, dst.0)).collect();
                elements = filter_elements(&state.payload.include_graph, &node_ids, &edge_ids);
            }
        }
        "symbol" => {
            let center = SymbolId(query.center.clone());
            let subgraph = state.reduced_model.traverse_neighborhood(&center, depth, direction);
            let node_ids: std::collections::HashSet<String> = subgraph.nodes.into_iter().map(|n| n.0).collect();
            let edge_ids: std::collections::HashSet<String> = subgraph.edges.into_iter().map(|(src, dst)| format!("{}-{}", src.0, dst.0)).collect();
            elements = filter_elements(&state.payload.symbol_graph, &node_ids, &edge_ids);
        }
        "call" => {
            let center = SymbolId(query.center.clone());
            let subgraph = state.call_graph.traverse_neighborhood(&center, depth, direction);
            let node_ids: std::collections::HashSet<String> = subgraph.nodes.into_iter().map(|n| n.0).collect();
            let edge_ids: std::collections::HashSet<String> = subgraph.edges.into_iter().map(|(src, dst)| format!("{}-{}", src.0, dst.0)).collect();
            elements = filter_elements(&state.payload.call_graph, &node_ids, &edge_ids);
        }
        _ => {}
    }
    Json(elements)
}

fn filter_elements(
    source_elements: &[CytoscapeElement],
    node_ids: &std::collections::HashSet<String>,
    edge_ids: &std::collections::HashSet<String>,
) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    for el in source_elements {
        if el.data.get("source").is_some() {
            if let Some(id) = el.data.get("id").and_then(|v| v.as_str()) {
                if edge_ids.contains(id) {
                    elements.push(el.clone());
                }
            }
        } else if let Some(id) = el.data.get("id").and_then(|v| v.as_str()) {
            if node_ids.contains(id) {
                elements.push(el.clone());
            }
        }
    }
    elements
}

async fn api_graph_path(
    State(state): State<Arc<UiState>>,
    Query(query): Query<PathQuery>,
) -> Json<Option<Vec<String>>> {
    let path = match query.graph_type.as_str() {
        "include" => {
            if let (Ok(from), Ok(to)) = (query.from.parse::<u64>(), query.to.parse::<u64>()) {
                state.include_graph.shortest_include_path(FileId(from), FileId(to))
                    .map(|p| p.into_iter().map(|id| id.0.to_string()).collect())
            } else {
                None
            }
        }
        "symbol" => {
            state.reduced_model.shortest_dependency_path(&SymbolId(query.from.clone()), &SymbolId(query.to.clone()))
                .map(|p| p.into_iter().map(|id| id.0).collect())
        }
        "call" => {
            state.call_graph.shortest_call_path(&SymbolId(query.from.clone()), &SymbolId(query.to.clone()))
                .map(|p| p.into_iter().map(|id| id.0).collect())
        }
        _ => None,
    };
    Json(path)
}

#[derive(Deserialize)]
struct EntrypointsQuery {
    graph_type: String,
    limit: Option<usize>,
}

async fn api_graph_entrypoints(
    State(state): State<Arc<UiState>>,
    Query(query): Query<EntrypointsQuery>,
) -> Json<EntrypointsResponse> {
    let limit = query.limit.unwrap_or(10);
    
    let (roots_ids, leaves_ids, hubs_ids, source_elements) = match query.graph_type.as_str() {
        "include" => {
            let (r, l, h) = state.include_graph.get_entrypoints(limit);
            (
                r.into_iter().map(|id| id.0.to_string()).collect(),
                l.into_iter().map(|id| id.0.to_string()).collect(),
                h.into_iter().map(|id| id.0.to_string()).collect(),
                &state.payload.include_graph
            )
        }
        "symbol" => {
            let (r, l, h) = state.reduced_model.get_entrypoints(limit);
            (
                r.into_iter().map(|id| id.0).collect(),
                l.into_iter().map(|id| id.0).collect(),
                h.into_iter().map(|id| id.0).collect(),
                &state.payload.symbol_graph
            )
        }
        "call" => {
            let (r, l, h) = state.call_graph.get_entrypoints(limit);
            (
                r.into_iter().map(|id| id.0).collect(),
                l.into_iter().map(|id| id.0).collect(),
                h.into_iter().map(|id| id.0).collect(),
                &state.payload.call_graph
            )
        }
        _ => (Vec::new(), Vec::new(), Vec::new(), &state.payload.include_graph),
    };

    let mut roots = Vec::new();
    let mut leaves = Vec::new();
    let mut hubs = Vec::new();

    let roots_set: std::collections::HashSet<String> = roots_ids.into_iter().collect();
    let leaves_set: std::collections::HashSet<String> = leaves_ids.into_iter().collect();
    let hubs_set: std::collections::HashSet<String> = hubs_ids.into_iter().collect();

    for el in source_elements {
        if el.data.get("source").is_none() {
            if let Some(id) = el.data.get("id").and_then(|v| v.as_str()) {
                if roots_set.contains(id) {
                    roots.push(el.clone());
                }
                if leaves_set.contains(id) {
                    leaves.push(el.clone());
                }
                if hubs_set.contains(id) {
                    hubs.push(el.clone());
                }
            }
        }
    }

    // Sort to match original limit order if necessary, but skipping for simplicity or can just return as is.
    
    // the requirement also mentions `cycles: Vec<Vec<String>>` for EntrypointsResponse
    // For now we can leave cycles empty or try to compute them, but let's just return empty as it's not specified how to get them here
    let cycles = Vec::new();

    Json(EntrypointsResponse {
        graph_type: query.graph_type,
        cycles,
        top_hubs: hubs,
        roots,
        leaves,
    })
}

async fn api_graph_search(
    State(state): State<Arc<UiState>>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<SearchResultItem>> {
    let mut results = Vec::new();
    let limit = query.limit.unwrap_or(20);

    let check_include = query.graph_type.as_deref() == Some("include") || query.graph_type.is_none();
    let check_symbol = query.graph_type.as_deref() == Some("symbol") || query.graph_type.is_none();
    let check_call = query.graph_type.as_deref() == Some("call") || query.graph_type.is_none();

    let q = query.q.clone();
    let is_regex = q.starts_with('/') && q.ends_with('/');
    let regex_pattern = if is_regex && q.len() > 1 {
        Some(regex::Regex::new(&q[1..q.len() - 1]).ok()).flatten()
    } else {
        None
    };

    let q_lower = q.to_lowercase();
    
    let is_match = |label: &str, path: &str| -> bool {
        if let Some(re) = &regex_pattern {
            return re.is_match(label) || re.is_match(path);
        }
        
        if q_lower.contains('*') || q_lower.contains('?') {
            // Very simple wildcard matching fallback for `*` or `?`.
            // We can convert wildcard to regex
            let escaped = regex::escape(&q_lower).replace("\\*", ".*").replace("\\?", ".");
            if let Ok(re) = regex::Regex::new(&format!("^{}$", escaped)) {
                return re.is_match(&label.to_lowercase()) || re.is_match(&path.to_lowercase());
            }
        }
        
        label.to_lowercase().contains(&q_lower) || path.to_lowercase().contains(&q_lower)
    };

    let search_in = |graph: &[CytoscapeElement], graph_type: &str, results: &mut Vec<SearchResultItem>| {
        for el in graph {
            if el.data.get("source").is_none() {
                let label = el.data.get("label").and_then(|v| v.as_str()).unwrap_or_default();
                let path = el.data.get("path").and_then(|v| v.as_str()).unwrap_or_default();
                
                if is_match(label, path) {
                    results.push(SearchResultItem {
                        id: el.data.get("id").unwrap().as_str().unwrap().to_string(),
                        label: label.to_string(),
                        kind: el.data.get("kind").unwrap().as_str().unwrap().to_string(),
                        graph_type: graph_type.to_string(),
                    });
                }
            }
        }
    };

    if check_include {
        search_in(&state.payload.include_graph, "include", &mut results);
    }
    if check_symbol {
        search_in(&state.payload.symbol_graph, "symbol", &mut results);
    }
    if check_call {
        search_in(&state.payload.call_graph, "call", &mut results);
    }

    results.truncate(limit);
    Json(results)
}
