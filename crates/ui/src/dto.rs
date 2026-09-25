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
    _closures: &std::collections::HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    _cycles: &[Vec<ariadne_repo::FileId>],
) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    for (file_id, path) in &graph.file_paths {
        elements.push(CytoscapeElement {
            data: serde_json::json!({ "id": file_id.0, "label": path.to_string_lossy() }),
            classes: None,
        });
    }
    for edge in &graph.edges {
        if let Some(dest) = edge.destination {
            elements.push(CytoscapeElement {
                data: serde_json::json!({ "source": edge.source.0, "target": dest.0 }),
                classes: None,
            });
        }
    }
    elements
}

pub fn build_symbol_elements(model: &ariadne_graph::ReducedSemanticModel) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    for sym in model.symbols.values() {
        elements.push(CytoscapeElement {
            data: serde_json::json!({ "id": sym.id.0, "label": sym.name }),
            classes: None,
        });
    }
    for edge in &model.edges {
        elements.push(CytoscapeElement {
            data: serde_json::json!({ "source": edge.src_symbol.0, "target": edge.dst_symbol.0 }),
            classes: None,
        });
    }
    elements
}

pub fn build_call_elements(callgraph: &ariadne_graph::CallGraph) -> Vec<CytoscapeElement> {
    let mut elements = Vec::new();
    for node in &callgraph.nodes {
        elements.push(CytoscapeElement {
            data: serde_json::json!({ "id": node.symbol_id.0, "label": node.name }),
            classes: None,
        });
    }
    for edge in &callgraph.edges {
        elements.push(CytoscapeElement {
            data: serde_json::json!({ "source": edge.caller.0, "target": edge.callee.0 }),
            classes: None,
        });
    }
    elements
}
