use ariadne_graph::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_ui::{build_call_elements, build_include_elements, GraphExportPayload, SummaryStats, UiState};
use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;

pub fn empty_reduced_model() -> ReducedSemanticModel {
    ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols: std::collections::BTreeMap::new(),
        edges: vec![],
        tu_usage: std::collections::BTreeMap::new(),
        file_symbols: std::collections::BTreeMap::new(),
    }
}

pub fn empty_export_payload() -> GraphExportPayload {
    GraphExportPayload {
        repo_name: "fixture-repo".to_string(),
        include_graph: vec![],
        symbol_graph: vec![],
        call_graph: vec![],
        stats: SummaryStats {
            total_files: 0,
            total_symbols: 0,
            total_calls: 0,
            cycle_count: 0,
        },
        file_collateral: HashMap::new(),
    }
}

pub fn ready_state(repo_root: std::path::PathBuf, include_graph: IncludeGraph, call_graph: CallGraph, payload: GraphExportPayload) -> Arc<UiState> {
    Arc::new(UiState::new_loaded(
        repo_root,
        include_graph,
        empty_reduced_model(),
        call_graph,
        payload,
    ))
}

pub fn loading_state() -> Arc<UiState> {
    Arc::new(UiState::new_loading(std::path::PathBuf::from(".")))
}

/// Include graph fixture: a.h(1) -> b.h(2) -> c.cc(3), plus isolated d.h(7).
pub fn fixture_include_graph() -> (IncludeGraph, GraphExportPayload) {
    use ariadne_repo::FileId;
    let mut graph = IncludeGraph::new();
    graph.file_paths.insert(FileId(1), std::path::PathBuf::from("a.h"));
    graph.file_paths.insert(FileId(2), std::path::PathBuf::from("b.h"));
    graph.file_paths.insert(FileId(3), std::path::PathBuf::from("c.cc"));
    graph.file_paths.insert(FileId(7), std::path::PathBuf::from("d.h"));

    let edge = |source: u64, destination: u64| ariadne_graph::IncludeEdge {
        source: FileId(source),
        destination: Some(FileId(destination)),
        context_name: String::new(),
        source_location: ariadne_graph::SourceLocation { line: 1, column: 1 },
        kind: ariadne_graph::IncludeKind::Local,
        raw_path: String::new(),
        resolved_path: None,
    };
    graph.edges.push(edge(1, 2));
    graph.edges.push(edge(2, 3));

    let mut payload = empty_export_payload();
    payload.include_graph = build_include_elements(&graph, &HashMap::new(), &[]);
    (graph, payload)
}

pub fn fixture_call_graph() -> (CallGraph, GraphExportPayload) {
    use ariadne_index::SymbolId;
    use ariadne_repo::FileId;
    let graph = CallGraph {
        nodes: vec![
            ariadne_graph::CallNode {
                symbol_id: SymbolId("main".to_string()),
                name: "main".to_string(),
                owning_file: FileId(3),
                location: None,
            },
            ariadne_graph::CallNode {
                symbol_id: SymbolId("helper".to_string()),
                name: "helper".to_string(),
                owning_file: FileId(3),
                location: None,
            },
        ],
        edges: vec![ariadne_graph::CallEdge {
            caller: SymbolId("main".to_string()),
            callee: SymbolId("helper".to_string()),
            call_count: 2,
            call_locations: vec![],
        }],
    };
    let mut payload = empty_export_payload();
    payload.call_graph = build_call_elements(&graph);
    (graph, payload)
}

pub async fn request_route(app: axum::Router, uri: &str) -> (StatusCode, HeaderMap, serde_json::Value) {
    let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json = if status == StatusCode::OK {
        serde_json::from_slice(&body).expect("OK response from a JSON route must carry a JSON body")
    } else {
        serde_json::Value::Null
    };
    (status, headers, json)
}

pub async fn request_raw_body(app: axum::Router, uri: &str) -> (StatusCode, HeaderMap, Vec<u8>) {
    let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap().to_vec();
    (status, headers, body)
}
