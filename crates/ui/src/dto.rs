use serde::{Deserialize, Serialize};

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
pub struct GraphExportPayload {
    pub repo_name: String,
    pub include_graph: Vec<CytoscapeElement>,
    pub symbol_graph: Vec<CytoscapeElement>,
    pub call_graph: Vec<CytoscapeElement>,
    pub stats: SummaryStats,
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

    for (file_id, path) in &graph.file_paths {
        let path_str = path.to_string_lossy();
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or(&path_str);
        let kind = if fname.ends_with(".h") || fname.ends_with(".hpp") || fname.ends_with(".hxx") {
            "HEADER"
        } else {
            "SOURCE"
        };
        let closure_size = closures.get(file_id).map(|c| c.closure_size).unwrap_or(0);
        let is_cycle = cycle_nodes.contains(file_id);

        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": file_id.0.to_string(),
                "label": fname,
                "path": path_str,
                "kind": kind,
                "closure": closure_size,
                "is_cycle": is_cycle,
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
    for sym in model.symbols.values() {
        let kind_str = format!("{:?}", sym.kind).to_uppercase();
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": sym.id.0.clone(),
                "label": sym.name.clone(),
                "kind": kind_str,
                "defining_file": sym.defining_file.map(|f| f.0.to_string()),
                "declarations_count": sym.declarations.len(),
            }),
            classes: None,
        });
    }
    for edge in &model.edges {
        let is_incomplete = edge.kind == ariadne_index::DependencyKind::Incomplete;
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": format!("{}-{}", edge.src_symbol.0, edge.dst_symbol.0),
                "source": edge.src_symbol.0.clone(),
                "target": edge.dst_symbol.0.clone(),
                "kind": format!("{:?}", edge.kind),
                "incomplete": is_incomplete,
            }),
            classes: if is_incomplete { Some("incomplete".into()) } else { Some("complete".into()) },
        });
    }
    elements
}

pub fn build_call_elements(callgraph: &ariadne_graph::CallGraph) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    for node in &callgraph.nodes {
        elements.push(CytoscapeElement {
            data: serde_json::json!({
                "id": node.symbol_id.0.clone(),
                "label": node.name.clone(),
                "kind": "FUNCTION",
                "owning_file": node.owning_file.0.to_string(),
                "line": node.location.as_ref().map(|l| l.line),
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
            classes: if is_self_recursion { Some("cycle".into()) } else { None },
        });
    }
    elements
}
