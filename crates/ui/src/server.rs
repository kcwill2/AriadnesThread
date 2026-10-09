use crate::dto::{
    CollateralResponse, CytoscapeElement, EntrypointsResponse, FileJustificationResponse,
    GraphExportPayload, PathQuery, SearchQuery, SearchResultItem, TraverseQuery,
    TraversalQuery, TraversalResponse, TraversalSubtree, TraversalSubtreeNode, TraversalSubtreeEdge, TraversalSummary,
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

/// Poison recovery: a poisoned lock only means some other thread already
/// panicked while holding it; the data is still memory-consistent, so the UI
/// keeps serving instead of cascading panics into every request handler.
fn read_state<'a, T>(lock: &'a std::sync::RwLock<T>) -> std::sync::RwLockReadGuard<'a, T> {
    lock.read().unwrap_or_else(|e| e.into_inner())
}

/// Poison recovery twin for writers: same reasoning as `read_state`.
fn write_state<'a, T>(lock: &'a std::sync::RwLock<T>) -> std::sync::RwLockWriteGuard<'a, T> {
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

pub fn build_router(state: Arc<UiState>) -> axum::Router {
    Router::new()
        .route("/", get(api_index))
        .route("/styles.css", get(asset_styles_css))
        .route("/themes.css", get(asset_themes_css))
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
        .layer(CorsLayer::new().allow_origin(AllowOrigin::predicate(is_same_origin)))
        .with_state(state)
}

async fn api_index() -> axum::response::Response {
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

async fn asset_styles_css() -> axum::response::Response {
    ui_asset(crate::STYLES_CSS, "text/css; charset=utf-8")
}

async fn asset_themes_css() -> axum::response::Response {
    ui_asset(crate::THEMES_CSS, "text/css; charset=utf-8")
}

async fn asset_api_js() -> axum::response::Response {
    ui_asset(crate::API_JS, "text/javascript; charset=utf-8")
}

async fn asset_graph_js() -> axum::response::Response {
    ui_asset(crate::GRAPH_JS, "text/javascript; charset=utf-8")
}

async fn asset_traversal_js() -> axum::response::Response {
    ui_asset(crate::TRAVERSAL_JS, "text/javascript; charset=utf-8")
}

async fn asset_file_view_js() -> axum::response::Response {
    ui_asset(crate::FILE_VIEW_JS, "text/javascript; charset=utf-8")
}

async fn asset_hotspots_js() -> axum::response::Response {
    ui_asset(crate::HOTSPOTS_JS, "text/javascript; charset=utf-8")
}

async fn asset_preview_js() -> axum::response::Response {
    ui_asset(crate::PREVIEW_JS, "text/javascript; charset=utf-8")
}

async fn asset_provenance_js() -> axum::response::Response {
    ui_asset(crate::PROVENANCE_JS, "text/javascript; charset=utf-8")
}

async fn asset_main_js() -> axum::response::Response {
    ui_asset(crate::MAIN_JS, "text/javascript; charset=utf-8")
}

async fn asset_theme_js() -> axum::response::Response {
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

async fn api_status(State(state): State<Arc<UiState>>) -> Json<StatusResponse> {
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

async fn api_graph_include(
    State(state): State<Arc<UiState>>,
) -> Result<Json<Vec<CytoscapeElement>>, axum::http::StatusCode> {
    let payload = read_state(&state.payload);
    payload
        .as_ref()
        .map(|p| Json(p.include_graph.clone()))
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)
}

async fn api_graph_symbol(
    State(state): State<Arc<UiState>>,
) -> Result<Json<Vec<CytoscapeElement>>, axum::http::StatusCode> {
    let payload = read_state(&state.payload);
    payload
        .as_ref()
        .map(|p| Json(p.symbol_graph.clone()))
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)
}

async fn api_graph_call(
    State(state): State<Arc<UiState>>,
) -> Result<Json<Vec<CytoscapeElement>>, axum::http::StatusCode> {
    let payload = read_state(&state.payload);
    payload
        .as_ref()
        .map(|p| Json(p.call_graph.clone()))
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)
}

#[derive(Deserialize)]
struct SourceQuery {
    file: String,
    /// Optional local path prefix to map server paths to local paths.
    /// If provided, the server strips its own `repo_root` prefix and
    /// prepends `local_prefix` before reading the file.
    local_prefix: Option<String>,
}

async fn api_source(
    State(state): State<Arc<UiState>>,
    Query(query): Query<SourceQuery>,
) -> Result<String, axum::http::StatusCode> {
    let candidate = if let Some(prefix) = &query.local_prefix {
        // Cross-environment navigation: strip server repo_root, apply local
        // prefix. The containment below sandboxes the result INTO the repo:
        // an escaping prefix is rejected, never followed.
        let server_root = &state.repo_root;
        let requested = std::path::Path::new(&query.file);
        let relative = requested.strip_prefix(server_root).unwrap_or(requested);
        let local_path = std::path::Path::new(prefix)
            .join(relative.to_string_lossy().trim_start_matches('/'));
        local_path
    } else {
        state.repo_root.join(&query.file)
    };
    // S1 containment (AR-01): canonicalize BOTH ends and require
    // `starts_with`, so absolute `file=` values, `..` traversal, escaping
    // `local_prefix=`, and in-repo symlinks that resolve outside all land
    // outside the canonical root and are rejected. Canonicalization failure
    // means a missing path: the same 404 the read produced before. Contained
    // reads keep the exact pre-fix bytes (locked by source_containment).
    let canonical_root = tokio::fs::canonicalize(&state.repo_root)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let contained_path = tokio::fs::canonicalize(&candidate)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
    if !contained_path.starts_with(&canonical_root) {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    tokio::fs::read_to_string(contained_path)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)
}

async fn api_export(
    State(state): State<Arc<UiState>>,
) -> Result<Json<GraphExportPayload>, axum::http::StatusCode> {
    let payload = read_state(&state.payload);
    payload
        .as_ref()
        .map(|p| Json(p.clone()))
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)
}

/// Repository metadata endpoint: exposes the server's `repo_root` and
/// file-path mapping so extension hosts can reconcile server-side paths
/// with their own local source tree (cross-environment navigation).
#[derive(serde::Serialize)]
pub struct RepoInfoResponse {
    /// Absolute path on the server where the analysis was generated.
    pub repo_root: String,
    /// File paths relative to `repo_root` (as the server sees them).
    pub file_paths: Vec<String>,
}

async fn api_repo_info(
    State(state): State<Arc<UiState>>,
) -> Json<RepoInfoResponse> {
    let analysis = state.analysis();
    let file_paths = analysis
        .as_ref()
        .map(|gen| {
            gen.include_graph
                .file_paths
                .keys()
                .map(|id| id.0.to_string())
                .collect()
        })
        .unwrap_or_default();

    Json(RepoInfoResponse {
        repo_root: state.repo_root.to_string_lossy().into_owned(),
        file_paths,
    })
}

async fn api_graph_traverse(
    State(state): State<Arc<UiState>>,
    Query(query): Query<TraverseQuery>,
) -> Result<Json<Vec<CytoscapeElement>>, axum::http::StatusCode> {
    let depth = query.depth.unwrap_or(1);
    let direction = match query.direction.as_deref() {
        Some("in") | Some("incoming") => TraversalDirection::Incoming,
        Some("out") | Some("outgoing") => TraversalDirection::Outgoing,
        _ => TraversalDirection::Both,
    };

    let mut elements = Vec::new();
    let payload = read_state(&state.payload);
    let payload_ref = payload
        .as_ref()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    match query.graph_type.as_str() {
        "include" => {
            if let Ok(id) = query.center.parse::<u64>() {
                let center = FileId(id);
                let gen = state
                    .analysis()
                    .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
                let subgraph = gen.include_graph.traverse_neighborhood(center, depth, direction);
                let node_ids: std::collections::HashSet<String> = subgraph
                    .nodes
                    .into_iter()
                    .map(|n| n.0.to_string())
                    .collect();
                let edge_ids: std::collections::HashSet<String> = subgraph
                    .edges
                    .into_iter()
                    .map(|(src, dst)| format!("{}-{}", src.0, dst.0))
                    .collect();
                elements = filter_elements(&payload_ref.include_graph, &node_ids, &edge_ids);
            }
        }
        "symbol" => {
            let center = SymbolId(query.center.clone());
            let gen = state
                .analysis()
                .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
            let subgraph = gen.reduced_model.traverse_neighborhood(&center, depth, direction);
            let node_ids: std::collections::HashSet<String> =
                subgraph.nodes.into_iter().map(|n| n.0).collect();
            let edge_ids: std::collections::HashSet<String> = subgraph
                .edges
                .into_iter()
                .map(|(src, dst)| format!("{}-{}", src.0, dst.0))
                .collect();
            elements = filter_elements(&payload_ref.symbol_graph, &node_ids, &edge_ids);
        }
        "call" => {
            let center = SymbolId(query.center.clone());
            let g = read_state(&state.call_graph);
            let g_ref = g
                .as_ref()
                .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
            let subgraph = g_ref.traverse_neighborhood(&center, depth, direction);
            let node_ids: std::collections::HashSet<String> =
                subgraph.nodes.into_iter().map(|n| n.0).collect();
            let edge_ids: std::collections::HashSet<String> = subgraph
                .edges
                .into_iter()
                .map(|(src, dst)| format!("{}-{}", src.0, dst.0))
                .collect();
            elements = filter_elements(&payload_ref.call_graph, &node_ids, &edge_ids);
        }
        _ => {}
    }
    Ok(Json(elements))
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
) -> Result<Json<Option<Vec<String>>>, axum::http::StatusCode> {
    let path = match query.graph_type.as_str() {
        "include" => {
            if let (Ok(from), Ok(to)) = (query.from.parse::<u64>(), query.to.parse::<u64>()) {
                let gen = state
                    .analysis()
                    .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
                gen.include_graph
                    .shortest_include_path(FileId(from), FileId(to))
                    .map(|p| p.into_iter().map(|id| id.0.to_string()).collect())
            } else {
                None
            }
        }
        "symbol" => {
            let gen = state
                .analysis()
                .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
            gen.reduced_model
                .shortest_dependency_path(
                    &SymbolId(query.from.clone()),
                    &SymbolId(query.to.clone()),
                )
                .map(|p| p.into_iter().map(|id| id.0).collect())
        }
        "call" => {
            let g = read_state(&state.call_graph);
            let g_ref = g
                .as_ref()
                .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
            g_ref
                .shortest_call_path(&SymbolId(query.from.clone()), &SymbolId(query.to.clone()))
                .map(|p| p.into_iter().map(|id| id.0).collect())
        }
        _ => None,
    };
    Ok(Json(path))
}

#[derive(Deserialize)]
struct EntrypointsQuery {
    graph_type: String,
    limit: Option<usize>,
}

async fn api_graph_entrypoints(
    State(state): State<Arc<UiState>>,
    Query(query): Query<EntrypointsQuery>,
) -> Result<Json<EntrypointsResponse>, axum::http::StatusCode> {
    let limit = query.limit.unwrap_or(10);

    let payload = read_state(&state.payload);
    let payload_ref = payload
        .as_ref()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    let (roots_ids, leaves_ids, hubs_ids, source_elements) = match query.graph_type.as_str() {
        "include" => {
            let gen = state
                .analysis()
                .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
            let (r, l, h) = gen.include_graph.get_entrypoints(limit);
            (
                r.into_iter().map(|id| id.0.to_string()).collect(),
                l.into_iter().map(|id| id.0.to_string()).collect(),
                h.into_iter().map(|id| id.0.to_string()).collect(),
                &payload_ref.include_graph,
            )
        }
        "symbol" => {
            let gen = state
                .analysis()
                .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
            let (r, l, h) = gen.reduced_model.get_entrypoints(limit);
            (
                r.into_iter().map(|id| id.0).collect(),
                l.into_iter().map(|id| id.0).collect(),
                h.into_iter().map(|id| id.0).collect(),
                &payload_ref.symbol_graph,
            )
        }
        "call" => {
            let g = read_state(&state.call_graph);
            let g_ref = g
                .as_ref()
                .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;
            let (r, l, h) = g_ref.get_entrypoints(limit);
            (
                r.into_iter().map(|id| id.0).collect(),
                l.into_iter().map(|id| id.0).collect(),
                h.into_iter().map(|id| id.0).collect(),
                &payload_ref.call_graph,
            )
        }
        _ => (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            &payload_ref.include_graph,
        ),
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

    Ok(Json(EntrypointsResponse {
        graph_type: query.graph_type,
        cycles,
        top_hubs: hubs,
        roots,
        leaves,
    }))
}

async fn api_graph_search(
    State(state): State<Arc<UiState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResultItem>>, axum::http::StatusCode> {
    let mut results = Vec::new();
    let limit = query.limit.unwrap_or(20);

    let payload = read_state(&state.payload);
    let payload_ref = payload
        .as_ref()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    let check_include =
        query.graph_type.as_deref() == Some("include") || query.graph_type.is_none();
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
            let escaped = regex::escape(&q_lower)
                .replace("\\*", ".*")
                .replace("\\?", ".");
            if let Ok(re) = regex::Regex::new(&format!("^{}$", escaped)) {
                return re.is_match(&label.to_lowercase()) || re.is_match(&path.to_lowercase());
            }
        }

        label.to_lowercase().contains(&q_lower) || path.to_lowercase().contains(&q_lower)
    };

    let search_in =
        |graph: &[CytoscapeElement], graph_type: &str, results: &mut Vec<SearchResultItem>| {
            for el in graph {
                if el.data.get("source").is_none() {
                    let label = el
                        .data
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let path = el
                        .data
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();

                    if is_match(label, path) {
                        results.push(SearchResultItem {
                            id: el
                                .data
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            label: label.to_string(),
                            kind: el
                                .data
                                .get("kind")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            graph_type: graph_type.to_string(),
                        });
                    }
                }
            }
        };

    if check_include {
        search_in(&payload_ref.include_graph, "include", &mut results);
    }
    if check_symbol {
        search_in(&payload_ref.symbol_graph, "symbol", &mut results);
    }
    if check_call {
        search_in(&payload_ref.call_graph, "call", &mut results);
    }

    results.truncate(limit);
    Ok(Json(results))
}

#[derive(Deserialize)]
pub struct CollateralQuery {
    pub file: String,
}

pub async fn api_graph_collateral(
    State(state): State<Arc<UiState>>,
    Query(query): Query<CollateralQuery>,
) -> Result<Json<CollateralResponse>, axum::http::StatusCode> {
    let mut direct_collateral = std::collections::HashMap::new();
    let mut transitive_collateral = std::collections::HashMap::new();
    let mut total_collateral = 0;

    if let Ok(file_id_num) = query.file.parse::<u64>() {
        let file_id = ariadne_repo::FileId(file_id_num);

        let gen = state
            .analysis()
            .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

        // Cheap per-request handle over the generation's ONE warm memo (C1).
        let analyzer = ariadne_graph::LoadAnalyzer::with_memo(
            &gen.include_graph,
            &gen.reduced_model,
            Arc::clone(&gen.memo),
        );
        if gen.include_graph.file_paths.contains_key(&file_id) {
            if let Ok(metric) = analyzer.analyze_file_load_by_id(file_id) {
                total_collateral = metric.collateral_load;

                let direct_includes = gen.include_graph.direct_includes(file_id);
                let mut direct_set = std::collections::HashSet::new();
                for d in direct_includes {
                    direct_set.insert(d);
                }

                for source in metric.collateral_sources {
                    let src_str = source.file_id.0.to_string();
                    if direct_set.contains(&source.file_id) {
                        direct_collateral.insert(src_str, source.symbol_count);
                    } else {
                        transitive_collateral.insert(src_str, source.symbol_count);
                    }
                }
            }
        }
    }

    Ok(Json(CollateralResponse {
        target_id: query.file.clone(),
        total_collateral,
        direct_collateral,
        transitive_collateral,
    }))
}

/// Query for the traversal view: given a root file and a direct dependency,
/// show the necessary vs actual import subtree and the collateral delta.
#[derive(Deserialize)]
pub struct TraversalApiQuery {
    pub root: String,
    pub dep: String,
    /// Optional specific symbol IDs to analyze (comma-separated); if empty, uses all justified
    /// symbols from the root->dep edge.
    #[serde(default)]
    pub symbols: String,
}

/// Live-mode traversal analysis: computes what the dependency actually pulls in
/// versus what's strictly needed for the symbols the root uses from it.
pub async fn api_graph_traversal(
    State(state): State<Arc<UiState>>,
    Query(query): Query<TraversalApiQuery>,
) -> Result<Json<TraversalResponse>, axum::http::StatusCode> {
    let root_id = query.root.parse::<u64>().map(ariadne_repo::FileId);
    let dep_id = query.dep.parse::<u64>().map(ariadne_repo::FileId);
    let (root_id, dep_id) = match (root_id, dep_id) {
        (Ok(r), Ok(d)) => (r, d),
        _ => return Err(axum::http::StatusCode::BAD_REQUEST),
    };

    let gen = state
        .analysis()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    // Verify both files exist in the graph
    if !gen.include_graph.file_paths.contains_key(&root_id)
        || !gen.include_graph.file_paths.contains_key(&dep_id)
    {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    // Verify dep is a direct dependency of root
    let direct_includes = gen.include_graph.direct_includes(root_id);
    if !direct_includes.contains(&dep_id) {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    let analyzer = ariadne_graph::LoadAnalyzer::with_memo(
        &gen.include_graph,
        &gen.reduced_model,
        Arc::clone(&gen.memo),
    );

    // Get justification data to find which symbols root uses from dep
    let root_metric = match analyzer.analyze_file_load_by_id(root_id) {
        Ok(m) => m,
        Err(_) => {
            return Ok(Json(TraversalResponse {
                root_file_id: root_id.0.to_string(),
                dep_file_id: dep_id.0.to_string(),
                analyzed_symbol_ids: vec![],
                necessary_subtree: TraversalSubtree::empty(),
                actual_subtree: TraversalSubtree::empty(),
                collateral_delta: TraversalSubtree::empty(),
                summary: TraversalSummary::empty(),
            }));
        }
    };
    let justification_detail = crate::dto::build_file_justification_detail(
        &root_id.0.to_string(),
        root_id,
        &root_metric,
        &analyzer,
        &gen.reduced_model,
    );

    // Find the justification entry for this specific dep
    let dep_str = dep_id.0.to_string();
    let dep_entry = justification_detail.entries.iter().find(|e| e.source_id == dep_str);

    // Determine which symbols to analyze
    let symbol_ids: Vec<ariadne_index::SymbolId> = if query.symbols.is_empty() {
        // Use all justified symbols from this edge
        dep_entry
            .map(|e| e.entry.justified_symbols.iter().map(|s| ariadne_index::SymbolId(s.clone())).collect())
            .unwrap_or_default()
    } else {
        // Use explicitly provided symbol IDs (comma-separated)
        query.symbols.split(',').filter(|s| !s.is_empty()).map(|s| ariadne_index::SymbolId(s.trim().to_string())).collect()
    };

    if symbol_ids.is_empty() {
        // No symbols to analyze - return empty response
        return Ok(Json(TraversalResponse {
            root_file_id: root_id.0.to_string(),
            dep_file_id: dep_id.0.to_string(),
            analyzed_symbol_ids: vec![],
            necessary_subtree: TraversalSubtree::empty(),
            actual_subtree: TraversalSubtree::empty(),
            collateral_delta: TraversalSubtree::empty(),
            summary: TraversalSummary::empty(),
        }));
    }

    // Compute extraction impact for these symbols
    let impact = analyzer.extraction_impact(root_id, dep_id, &symbol_ids);

    let (necessary_subtree, actual_subtree, collateral_delta, summary) = if let Some(impact) = impact {
        // Build necessary subtree from required_headers
        let necessary_nodes = build_subtree_nodes(&impact.required_headers, true, false, &gen);
        let necessary_edges = build_subtree_edges(&impact.required_headers, &gen);
        let necessary_symbols: usize = impact.required_headers.iter().map(|h| h.symbol_count).sum();
        let necessary_subtree = TraversalSubtree {
            nodes: necessary_nodes,
            edges: necessary_edges,
            total_symbols: necessary_symbols,
            file_count: impact.required_headers.len(),
        };

        // Build actual subtree from full closure of dep (excluding dep itself)
        let dep_closure = gen.memo.strict_closure(&gen.include_graph, dep_id);
        let actual_nodes = build_subtree_nodes_from_closure(&dep_closure, &gen, dep_id);
        let actual_edges = build_subtree_edges_from_closure(&dep_closure, &gen);
        let actual_symbols: usize = actual_nodes.iter().map(|n| n.symbol_count).sum();
        let actual_subtree = TraversalSubtree {
            nodes: actual_nodes,
            edges: actual_edges,
            total_symbols: actual_symbols,
            file_count: dep_closure.len().saturating_sub(1),
        };

        // Build collateral delta from dropped_from_root
        let delta_nodes = build_subtree_nodes(&impact.dropped_from_root, false, true, &gen);
        let delta_edges = build_subtree_edges(&impact.dropped_from_root, &gen);
        let delta_symbols: usize = impact.dropped_from_root.iter().map(|h| h.symbol_count).sum();
        let collateral_delta = TraversalSubtree {
            nodes: delta_nodes,
            edges: delta_edges,
            total_symbols: delta_symbols,
            file_count: impact.dropped_from_root.len(),
        };

        // Decoy headers are in still_reachable
        let decoy_files = impact.still_reachable.len();
        let decoy_symbols: usize = impact.still_reachable.iter().map(|h| h.symbol_count).sum();

        let summary = TraversalSummary {
            necessary_files: necessary_subtree.file_count,
            actual_files: actual_subtree.file_count,
            collateral_files: collateral_delta.file_count,
            necessary_symbols: necessary_subtree.total_symbols,
            actual_symbols: actual_subtree.total_symbols,
            collateral_symbols: collateral_delta.total_symbols,
            decoy_files,
            decoy_symbols,
        };

        (necessary_subtree, actual_subtree, collateral_delta, summary)
    } else {
        // No impact (e.g., root == dep or no justified symbols)
        (
            TraversalSubtree::empty(),
            TraversalSubtree::empty(),
            TraversalSubtree::empty(),
            TraversalSummary::empty(),
        )
    };

    let analyzed_symbol_ids: Vec<String> = symbol_ids.iter().map(|s| s.0.clone()).collect();

    Ok(Json(TraversalResponse {
        root_file_id: root_id.0.to_string(),
        dep_file_id: dep_id.0.to_string(),
        analyzed_symbol_ids,
        necessary_subtree,
        actual_subtree,
        collateral_delta,
        summary,
    }))
}

// Helper functions for building subtree representations

fn build_subtree_nodes(
    headers: &[ariadne_graph::ImpactHeader],
    is_necessary: bool,
    is_collateral: bool,
    gen: &crate::AnalysisGeneration,
) -> Vec<TraversalSubtreeNode> {
    headers
        .iter()
        .map(|h| {
            let path_str = h.path.to_string_lossy().to_string();
            let fname = h
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&path_str);
            let kind = if fname.ends_with(".h") || fname.ends_with(".hpp") || fname.ends_with(".hxx") {
                "HEADER"
            } else {
                "SOURCE"
            };
            TraversalSubtreeNode {
                id: h.file_id.0.to_string(),
                label: fname.to_string(),
                path: path_str,
                kind: kind.to_string(),
                is_necessary,
                is_collateral,
                is_decoy: false,
                symbol_count: h.symbol_count,
            }
        })
        .collect()
}

fn build_subtree_nodes_from_closure(
    closure: &std::collections::HashSet<ariadne_repo::FileId>,
    gen: &crate::AnalysisGeneration,
    exclude: ariadne_repo::FileId,
) -> Vec<TraversalSubtreeNode> {
    closure
        .iter()
        .filter(|&&id| id != exclude)
        .filter_map(|id| gen.include_graph.file_paths.get(id).map(|p| (id, p)))
        .map(|(id, path)| {
            let path_str = path.to_string_lossy().to_string();
            let fname = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&path_str);
            let kind = if fname.ends_with(".h") || fname.ends_with(".hpp") || fname.ends_with(".hxx") {
                "HEADER"
            } else {
                "SOURCE"
            };
            let symbol_count = gen
                .reduced_model
                .file_symbols
                .get(id)
                .map(|s| s.len())
                .unwrap_or(0);
            TraversalSubtreeNode {
                id: id.0.to_string(),
                label: fname.to_string(),
                path: path_str,
                kind: kind.to_string(),
                is_necessary: false,
                is_collateral: false,
                is_decoy: false,
                symbol_count,
            }
        })
        .collect()
}

fn build_subtree_edges(
    headers: &[ariadne_graph::ImpactHeader],
    gen: &crate::AnalysisGeneration,
) -> Vec<TraversalSubtreeEdge> {
    let mut edges = Vec::new();
    let header_ids: std::collections::HashSet<_> = headers.iter().map(|h| h.file_id).collect();
    for h in headers {
        // Find edges between headers in the set
        let direct_includes = gen.include_graph.direct_includes(h.file_id);
        for target in direct_includes {
            if header_ids.contains(&target) {
                edges.push(TraversalSubtreeEdge {
                    source: h.file_id.0.to_string(),
                    target: target.0.to_string(),
                    is_necessary: true,
                });
            }
        }
    }
    edges
}

fn build_subtree_edges_from_closure(
    closure: &std::collections::HashSet<ariadne_repo::FileId>,
    gen: &crate::AnalysisGeneration,
) -> Vec<TraversalSubtreeEdge> {
    let mut edges = Vec::new();
    let closure_set: std::collections::HashSet<_> = closure.iter().copied().collect();
    for &source in closure {
        let direct_includes = gen.include_graph.direct_includes(source);
        for target in direct_includes {
            if closure_set.contains(&target) {
                edges.push(TraversalSubtreeEdge {
                    source: source.0.to_string(),
                    target: target.0.to_string(),
                    is_necessary: false,
                });
            }
        }
    }
    edges
}

impl TraversalSubtree {
    fn empty() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            total_symbols: 0,
            file_count: 0,
        }
    }
}

impl TraversalSummary {
    fn empty() -> Self {
        Self {
            necessary_files: 0,
            actual_files: 0,
            collateral_files: 0,
            necessary_symbols: 0,
            actual_symbols: 0,
            collateral_symbols: 0,
            decoy_files: 0,
            decoy_symbols: 0,
        }
    }
}

#[derive(Deserialize)]
pub struct JustificationQuery {
    pub file: String,
}

/// Live-mode on-demand detail: full unbounded justification for one file,
/// recomputed per request like `/api/graph/collateral`. Unparseable or
/// unknown file ids mirror the collateral tolerance: 200 with empty detail.
pub async fn api_justification(
    State(state): State<Arc<UiState>>,
    Query(query): Query<JustificationQuery>,
) -> Result<Json<FileJustificationResponse>, axum::http::StatusCode> {
    let mut response = FileJustificationResponse {
        target_id: query.file.clone(),
        total_justified: 0,
        total_collateral: 0,
        entries: Vec::new(),
    };

    if let Ok(file_id_num) = query.file.parse::<u64>() {
        let file_id = ariadne_repo::FileId(file_id_num);

        let gen = state
            .analysis()
            .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

        // C1: O(1) handle over the generation's ONE warm memo — no more
        // per-request `LoadAnalyzer::new` (which re-ran compute_closures and
        // cold BFS on every click).
        let analyzer = ariadne_graph::LoadAnalyzer::with_memo(
            &gen.include_graph,
            &gen.reduced_model,
            Arc::clone(&gen.memo),
        );
        if gen.include_graph.file_paths.contains_key(&file_id) {
            // A4 by-id: the id is already in hand; skip the O(V) path scan.
            if let Ok(metric) = analyzer.analyze_file_load_by_id(file_id) {
                response = crate::dto::build_file_justification_detail(
                    &query.file,
                    file_id,
                    &metric,
                    &analyzer,
                    &gen.reduced_model,
                );
            }
        }
    }

    Ok(Json(response))
}
