use ariadne_graph::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_ui::{
    build_call_elements, build_include_elements, build_symbol_elements, export_standalone_html,
    build_router, GraphExportPayload, SummaryStats, UiState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;

#[tokio::test]
async fn test_dto_builders() {
    let include_graph = IncludeGraph::new();
    let closures = HashMap::new();
    let cycles = vec![];

    let elements = build_include_elements(&include_graph, &closures, &cycles);
    assert!(elements.is_empty());

    let reduced_model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols: std::collections::BTreeMap::new(),
        edges: vec![],
        tu_usage: std::collections::BTreeMap::new(),
        file_symbols: std::collections::BTreeMap::new(),
    };
    let sym_elements = build_symbol_elements(&reduced_model);
    assert!(sym_elements.is_empty());

    let call_graph = CallGraph::default();
    let call_elements = build_call_elements(&call_graph);
    assert!(call_elements.is_empty());
}

#[tokio::test]
async fn test_server_routes() {
    let state = Arc::new(UiState {
        repo_root: std::path::PathBuf::from("."),
        include_graph: IncludeGraph::new(),
        reduced_model: ReducedSemanticModel {
            repo_root: std::path::PathBuf::from("."),
            symbols: std::collections::BTreeMap::new(),
            edges: vec![],
            tu_usage: std::collections::BTreeMap::new(),
            file_symbols: std::collections::BTreeMap::new(),
        },
        call_graph: CallGraph::default(),
        payload: GraphExportPayload {
            repo_name: "test".to_string(),
            include_graph: vec![],
            symbol_graph: vec![],
            call_graph: vec![],
            stats: SummaryStats {
                total_files: 0,
                total_symbols: 0,
                total_calls: 0,
                cycle_count: 0,
            },
        },
    });

    let app = build_router(state);

    let req = Request::builder()
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[test]
fn test_export_html() {
    let payload = GraphExportPayload {
        repo_name: "test".to_string(),
        include_graph: vec![],
        symbol_graph: vec![],
        call_graph: vec![],
        stats: SummaryStats {
            total_files: 0,
            total_symbols: 0,
            total_calls: 0,
            cycle_count: 0,
        },
    };
    let template = "<html><body>/*__ARIADNE_DATA__*/</body></html>";
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("out.html");

    export_standalone_html(&payload, template, &out_path).unwrap();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("window.__ARIADNE_DATA__"));
}

#[test]
fn test_index_html_content() {
    let html = ariadne_ui::INDEX_HTML;
    assert!(html.contains("/*__ARIADNE_DATA__*/"));
    assert!(html.contains("Include Graph"));
    assert!(html.contains("Symbol Dependency Graph"));
    assert!(html.contains("Call Graph"));
}

#[tokio::test]
async fn test_server_index_route() {
    let state = Arc::new(UiState {
        repo_root: std::path::PathBuf::from("."),
        include_graph: IncludeGraph::new(),
        reduced_model: ReducedSemanticModel {
            repo_root: std::path::PathBuf::from("."),
            symbols: std::collections::BTreeMap::new(),
            edges: vec![],
            tu_usage: std::collections::BTreeMap::new(),
            file_symbols: std::collections::BTreeMap::new(),
        },
        call_graph: CallGraph::default(),
        payload: GraphExportPayload {
            repo_name: "test".to_string(),
            include_graph: vec![],
            symbol_graph: vec![],
            call_graph: vec![],
            stats: SummaryStats {
                total_files: 0,
                total_symbols: 0,
                total_calls: 0,
                cycle_count: 0,
            },
        },
    });

    let app = build_router(state);

    let req = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get("Content-Type").unwrap(), "text/html; charset=utf-8");
}

#[test]
fn test_export_html_empty_template_uses_index() {
    let payload = GraphExportPayload {
        repo_name: "test".to_string(),
        include_graph: vec![],
        symbol_graph: vec![],
        call_graph: vec![],
        stats: SummaryStats {
            total_files: 0,
            total_symbols: 0,
            total_calls: 0,
            cycle_count: 0,
        },
    };
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("out2.html");

    // Empty template should trigger fallback to INDEX_HTML
    export_standalone_html(&payload, "", &out_path).unwrap();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("window.__ARIADNE_DATA__"));
    assert!(content.contains("Include Graph")); // Check if it used the INDEX_HTML
}
