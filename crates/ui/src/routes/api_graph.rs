//! AR-13 extraction (verbatim from server.rs): the `/api/graph/*` family and
//! `/api/justification`, including the traversal subtree builders.

use crate::dto::{
    CollateralResponse, CytoscapeElement, EntrypointsResponse, FileJustificationResponse,
    PathQuery, SearchQuery, SearchResultItem, TraverseQuery,
    TraversalResponse, TraversalSubtree, TraversalSubtreeNode, TraversalSubtreeEdge, TraversalSummary,
};
use crate::state::{read_state, UiState};
use ariadne_graph::traversal::TraversalDirection;
use ariadne_index::SymbolId;
use ariadne_repo::FileId;
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

pub(crate) async fn api_graph_include(
    State(state): State<Arc<UiState>>,
) -> Result<Json<Vec<CytoscapeElement>>, axum::http::StatusCode> {
    let payload = read_state(&state.payload);
    payload
        .as_ref()
        .map(|p| Json(p.include_graph.clone()))
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)
}

pub(crate) async fn api_graph_symbol(
    State(state): State<Arc<UiState>>,
) -> Result<Json<Vec<CytoscapeElement>>, axum::http::StatusCode> {
    let payload = read_state(&state.payload);
    payload
        .as_ref()
        .map(|p| Json(p.symbol_graph.clone()))
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)
}

pub(crate) async fn api_graph_call(
    State(state): State<Arc<UiState>>,
) -> Result<Json<Vec<CytoscapeElement>>, axum::http::StatusCode> {
    let payload = read_state(&state.payload);
    payload
        .as_ref()
        .map(|p| Json(p.call_graph.clone()))
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)
}

pub(crate) async fn api_graph_traverse(
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

pub(crate) async fn api_graph_path(
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
pub(crate) struct EntrypointsQuery {
    graph_type: String,
    limit: Option<usize>,
}

pub(crate) async fn api_graph_entrypoints(
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

/// P7: the wildcard glob for one search request, compiled ONCE per request
/// instead of once per element. Engaged only when the query carries `*`/`?`
/// and no `/regex/` literal claimed it; the case-insensitive flag subsumes
/// the old per-element lowercase of label and path (identical outcomes for
/// the ASCII targets the graph labels carry). A failed compile yields None
/// and the substring fallback stays live, exactly as the old per-element
/// path behaved.
fn build_wildcard_glob(
    q_lower: &str,
    regex_pattern: &Option<regex::Regex>,
) -> Option<regex::Regex> {
    if regex_pattern.is_some() || !(q_lower.contains('*') || q_lower.contains('?')) {
        return None;
    }
    let escaped = regex::escape(q_lower)
        .replace("\\*", ".*")
        .replace("\\?", ".");
    regex::Regex::new(&format!("(?i)^{}$", escaped)).ok()
}

pub(crate) async fn api_graph_search(
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

    // P7: wildcard glob compiled once per request (see build_wildcard_glob).
    // The branch order is locked by the search_characterization suite.
    let wildcard_pattern = build_wildcard_glob(&q_lower, &regex_pattern);

    let is_match = |label: &str, path: &str| -> bool {
        if let Some(re) = &regex_pattern {
            return re.is_match(label) || re.is_match(path);
        }
        if let Some(glob) = &wildcard_pattern {
            return glob.is_match(label) || glob.is_match(path);
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
/// AR-13 decomposition of the former 161-line body (R1 row): the request
/// handler now sequences seed resolution, impact computation and projection;
/// every check keeps its original order and status code.
pub async fn api_graph_traversal(
    State(state): State<Arc<UiState>>,
    Query(query): Query<TraversalApiQuery>,
) -> Result<Json<TraversalResponse>, axum::http::StatusCode> {
    let (root_id, dep_id) = resolve_traversal_file_pair(&query)?;

    let gen = state
        .analysis()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    ensure_traversal_edge(&gen, root_id, dep_id)?;

    let analyzer = ariadne_graph::LoadAnalyzer::with_memo(
        &gen.include_graph,
        &gen.reduced_model,
        Arc::clone(&gen.memo),
    );

    // Get justification data to find which symbols root uses from dep
    let root_metric = match analyzer.analyze_file_load_by_id(root_id) {
        Ok(m) => m,
        Err(_) => {
            return Ok(Json(empty_traversal_response(root_id, dep_id)));
        }
    };
    let justification_detail = crate::dto::build_file_justification_detail(
        &root_id.0.to_string(),
        root_id,
        &root_metric,
        &analyzer,
        &gen.reduced_model,
    );

    // Determine which symbols to analyze
    let symbol_ids = resolve_traversal_symbols(&query.symbols, &justification_detail, dep_id);

    if symbol_ids.is_empty() {
        // No symbols to analyze - return empty response
        return Ok(Json(empty_traversal_response(root_id, dep_id)));
    }

    // Compute extraction impact for these symbols
    let impact = analyzer.extraction_impact(root_id, dep_id, &symbol_ids);

    let (necessary_subtree, actual_subtree, collateral_delta, summary) =
        project_traversal_subtrees(impact, &gen, dep_id);

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

/// Seed resolution (AR-13): parse the root/dep query strings into file ids.
/// An unparsable seed yields 400 exactly as the inline original did.
fn resolve_traversal_file_pair(
    query: &TraversalApiQuery,
) -> Result<(ariadne_repo::FileId, ariadne_repo::FileId), axum::http::StatusCode> {
    let root_id = query.root.parse::<u64>().map(ariadne_repo::FileId);
    let dep_id = query.dep.parse::<u64>().map(ariadne_repo::FileId);
    match (root_id, dep_id) {
        (Ok(r), Ok(d)) => Ok((r, d)),
        _ => Err(axum::http::StatusCode::BAD_REQUEST),
    }
}

/// Seed validation (AR-13): both files must exist in the graph (404) and dep
/// must be a direct include of root (400); same checks, same order, same
/// status codes as the pre-decomposition body.
fn ensure_traversal_edge(
    gen: &crate::AnalysisGeneration,
    root_id: ariadne_repo::FileId,
    dep_id: ariadne_repo::FileId,
) -> Result<(), axum::http::StatusCode> {
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
    Ok(())
}

/// The "nothing to analyze" payload shared by the two early returns (an
/// unanalysable root metric, or an empty symbol seed).
fn empty_traversal_response(
    root_id: ariadne_repo::FileId,
    dep_id: ariadne_repo::FileId,
) -> TraversalResponse {
    TraversalResponse {
        root_file_id: root_id.0.to_string(),
        dep_file_id: dep_id.0.to_string(),
        analyzed_symbol_ids: vec![],
        necessary_subtree: TraversalSubtree::empty(),
        actual_subtree: TraversalSubtree::empty(),
        collateral_delta: TraversalSubtree::empty(),
        summary: TraversalSummary::empty(),
    }
}

/// Symbol seed resolution (AR-13): explicit comma-separated ids win;
/// otherwise all justified symbols of the root->dep edge, empty when that
/// edge carries no justification entry.
fn resolve_traversal_symbols(
    symbols_param: &str,
    justification_detail: &FileJustificationResponse,
    dep_id: ariadne_repo::FileId,
) -> Vec<ariadne_index::SymbolId> {
    // Find the justification entry for this specific dep
    let dep_str = dep_id.0.to_string();
    let dep_entry = justification_detail.entries.iter().find(|e| e.source_id == dep_str);

    if symbols_param.is_empty() {
        // Use all justified symbols from this edge
        dep_entry
            .map(|e| e.entry.justified_symbols.iter().map(|s| ariadne_index::SymbolId(s.clone())).collect())
            .unwrap_or_default()
    } else {
        // Use explicitly provided symbol IDs (comma-separated)
        symbols_param.split(',').filter(|s| !s.is_empty()).map(|s| ariadne_index::SymbolId(s.trim().to_string())).collect()
    }
}

/// Closure computation + projection (AR-13): turns an extraction impact into
/// the necessary / actual / collateral subtrees plus the summary; `None`
/// impact (root == dep, no justified symbols) yields all-empty parts.
fn project_traversal_subtrees(
    impact: Option<ariadne_graph::ExtractionImpact>,
    gen: &crate::AnalysisGeneration,
    dep_id: ariadne_repo::FileId,
) -> (
    TraversalSubtree,
    TraversalSubtree,
    TraversalSubtree,
    TraversalSummary,
) {
    if let Some(impact) = impact {
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
    }
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
