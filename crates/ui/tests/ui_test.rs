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

#[tokio::test]
async fn test_server_traversal_routes() {
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
        .uri("/api/graph/traverse?graph_type=include&center=test")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/api/graph/path?graph_type=call&from=a&to=b")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/api/graph/entrypoints?graph_type=symbol")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/api/graph/search?q=test")
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
    
    // M16 Unit 3: Interactive Frontend Traversal elements
    assert!(html.contains("id=\"ariadneThread\"") || html.contains("class=\"thread-bar\""));
    assert!(html.contains("id=\"entrypointsModal\"") || html.contains("id=\"entrypointsPanel\""));
    assert!(html.contains("id=\"pathfinderModal\""));
    assert!(html.contains("clientTraverseNeighborhood"));
    assert!(html.contains("clientShortestPath"));
    assert!(html.contains("clientGetEntrypoints"));
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

#[test]
fn test_selected_node_edge_highlighting() {
    let html = ariadne_ui::INDEX_HTML;
    
    // Check edge detection relative to selectedNodeId
    assert!(html.contains("e.target === selectedNodeId"));
    assert!(html.contains("e.source === selectedNodeId"));
    
    // Check highlighted classes
    assert!(html.contains("highlighted"));
    assert!(html.contains("entering") || html.contains("edge-entering"));
    assert!(html.contains("leaving") || html.contains("edge-leaving"));
    
    // Check marker assignment
    assert!(html.contains("url(#arrow-selected)"));
    assert!(html.contains("url(#arrow-entering)"));
    assert!(html.contains("url(#arrow-leaving)"));
    
    // Check CSS rules
    assert!(html.contains(".edge-path.highlighted"));
}
