use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProgress {
    pub state: String,
    pub phase: String,
    pub percent: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseQuery {
    pub graph_type: String,
    pub center: String,
    pub depth: Option<usize>,
    pub direction: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathQuery {
    pub graph_type: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub graph_type: Option<String>,
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub graph_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntrypointsResponse {
    pub graph_type: String,
    pub cycles: Vec<Vec<String>>,
    pub top_hubs: Vec<CytoscapeElement>,
    pub roots: Vec<CytoscapeElement>,
    pub leaves: Vec<CytoscapeElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CytoscapeElement {
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub total_files: usize,
    pub total_symbols: usize,
    pub total_calls: usize,
    pub cycle_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralResponse {
    pub target_id: String,
    pub total_collateral: usize,
    pub direct_collateral: std::collections::HashMap<String, usize>,
    pub transitive_collateral: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExportPayload {
    pub repo_name: String,
    pub include_graph: Vec<CytoscapeElement>,
    pub symbol_graph: Vec<CytoscapeElement>,
    pub call_graph: Vec<CytoscapeElement>,
    pub stats: SummaryStats,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub file_collateral:
        std::collections::HashMap<String, std::collections::HashMap<String, usize>>,
}

pub fn build_include_elements(
    graph: &ariadne_graph::IncludeGraph,
    closures: &std::collections::HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    cycles: &[Vec<ariadne_repo::FileId>],
) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    let mut cycle_nodes = std::collections::HashSet::new();
    let mut cycle_edges = std::collections::HashSet::new();
    for cycle in cycles {
        for w in cycle.windows(2) {
            cycle_nodes.insert(w[0]);
            cycle_nodes.insert(w[1]);
            cycle_edges.insert((w[0], w[1]));
        }
        if let (Some(&first), Some(&last)) = (cycle.first(), cycle.last()) {
            cycle_nodes.insert(first);
            cycle_nodes.insert(last);
            cycle_edges.insert((last, first));
        }
    }

    let mut in_degrees = std::collections::HashMap::new();
    let mut out_degrees = std::collections::HashMap::new();
    for edge in &graph.edges {
        if let Some(dest) = edge.destination {
            *out_degrees.entry(edge.source).or_insert(0) += 1;
            *in_degrees.entry(dest).or_insert(0) += 1;
        }
    }

    for (file_id, path) in &graph.file_paths {
        let path_str = path.to_string_lossy();
        let fname = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&path_str);
        let kind = if fname.ends_with(".h") || fname.ends_with(".hpp") || fname.ends_with(".hxx") {
            "HEADER"
        } else {
            "SOURCE"
        };
        let closure_size = closures.get(file_id).map(|c| c.closure_size).unwrap_or(0);
        let is_cycle = cycle_nodes.contains(file_id);
        let in_degree = in_degrees.get(file_id).copied().unwrap_or(0);
        let out_degree = out_degrees.get(file_id).copied().unwrap_or(0);

        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": file_id.0.to_string(),
                "label": fname,
                "path": path_str,
                "kind": kind,
                "closure": closure_size,
                "is_cycle": is_cycle,
                "in_degree": in_degree,
                "out_degree": out_degree,
            }),
            classes: if is_cycle { Some("cycle".into()) } else { None },
        });
    }

    for edge in &graph.edges {
        if let Some(dest) = edge.destination {
            let is_cycle = cycle_edges.contains(&(edge.source, dest));
            elements.push(CytoscapeElement {
                data: serde_json::json!({
                    "id": format!("{}-{}", edge.source.0, dest.0),
                    "source": edge.source.0.to_string(),
                    "target": dest.0.to_string(),
                    "kind": format!("{:?}", edge.kind),
                    "is_cycle": is_cycle,
                }),
                classes: if is_cycle { Some("cycle".into()) } else { None },
            });
        }
    }
    elements
}

pub fn build_symbol_elements(model: &ariadne_graph::ReducedSemanticModel) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    let mut in_degrees = std::collections::HashMap::new();
    let mut out_degrees = std::collections::HashMap::new();

    let valid_edges: Vec<_> = model
        .edges
        .iter()
        .filter(|e| {
            model.symbols.contains_key(&e.src_symbol) && model.symbols.contains_key(&e.dst_symbol)
        })
        .collect();

    for edge in &valid_edges {
        *out_degrees.entry(edge.src_symbol.clone()).or_insert(0) += 1;
        *in_degrees.entry(edge.dst_symbol.clone()).or_insert(0) += 1;
    }

    for sym in model.symbols.values() {
        let kind_str = format!("{:?}", sym.kind).to_uppercase();
        let in_degree = in_degrees.get(&sym.id).copied().unwrap_or(0);
        let out_degree = out_degrees.get(&sym.id).copied().unwrap_or(0);
        let path_str = sym
            .defining_file
            .map(|f| f.0.to_string())
            .unwrap_or_default();
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": sym.id.0.clone(),
                "label": sym.name.clone(),
                "kind": kind_str,
                "defining_file": sym.defining_file.map(|f| f.0.to_string()),
                "path": path_str,
                "declarations_count": sym.declarations.len(),
                "in_degree": in_degree,
                "out_degree": out_degree,
            }),
            classes: None,
        });
    }
    for edge in &valid_edges {
        let is_incomplete = edge.kind == ariadne_index::DependencyKind::Incomplete;
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": format!("{}-{}", edge.src_symbol.0, edge.dst_symbol.0),
                "source": edge.src_symbol.0.clone(),
                "target": edge.dst_symbol.0.clone(),
                "kind": format!("{:?}", edge.kind),
                "incomplete": is_incomplete,
            }),
            classes: if is_incomplete {
                Some("incomplete".into())
            } else {
                Some("complete".into())
            },
        });
    }
    elements
}

pub fn build_call_elements(callgraph: &ariadne_graph::CallGraph) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    let mut in_degrees = std::collections::HashMap::new();
    let mut out_degrees = std::collections::HashMap::new();
    for edge in &callgraph.edges {
        *out_degrees.entry(edge.caller.clone()).or_insert(0) += 1;
        *in_degrees.entry(edge.callee.clone()).or_insert(0) += 1;
    }

    for node in &callgraph.nodes {
        let in_degree = in_degrees.get(&node.symbol_id).copied().unwrap_or(0);
        let out_degree = out_degrees.get(&node.symbol_id).copied().unwrap_or(0);
        let path_str = if let Some(loc) = &node.location {
            format!("{}:{}", loc.file_id.0, loc.line)
        } else {
            node.owning_file.0.to_string()
        };
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": node.symbol_id.0.clone(),
                "label": node.name.clone(),
                "kind": "FUNCTION",
                "owning_file": node.owning_file.0.to_string(),
                "path": path_str,
                "line": node.location.as_ref().map(|l| l.line),
                "in_degree": in_degree,
                "out_degree": out_degree,
            }),
            classes: None,
        });
    }
    for edge in &callgraph.edges {
        let is_self_recursion = edge.caller == edge.callee;
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": format!("{}-{}", edge.caller.0, edge.callee.0),
                "source": edge.caller.0.clone(),
                "target": edge.callee.0.clone(),
                "call_count": edge.call_count,
                "is_cycle": is_self_recursion,
            }),
            classes: if is_self_recursion {
                Some("cycle".into())
            } else {
                None
            },
        });
    }
    elements
}

/// Per-stage persistence reuse reported by the artifact pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageInfo {
    pub stage: String,
    pub reuse: String,
}

/// Live persistence/watcher status surfaced through `/api/status`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StageStatus {
    pub stages: Vec<StageInfo>,
    pub last_refresh: Option<u64>,
    pub rebuilding: bool,
    pub reused_tus: usize,
    pub reanalyzed_tus: usize,
}
