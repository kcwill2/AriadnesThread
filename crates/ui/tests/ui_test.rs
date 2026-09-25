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

#[tokio::test]
async fn test_search_advanced_queries_and_dto_paths() {
    // We will build a UI state with some mock elements to test search and DTO builder
    let mut include_graph = IncludeGraph::new();
    let file1 = ariadne_repo::FileId(1);
    include_graph.file_paths.insert(file1, std::path::PathBuf::from("include/test1.h"));
    
    let mut call_graph = CallGraph::default();
    call_graph.nodes.push(ariadne_graph::CallNode {
        symbol_id: ariadne_index::SymbolId("func1".to_string()),
        name: "func1".to_string(),
        owning_file: file1,
        location: Some(ariadne_index::types::FileLocation { file_id: file1, line: 10, column: 5 }),
    });

    let reduced_model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols: std::collections::BTreeMap::from([(
            ariadne_index::SymbolId("sym1".to_string()),
            ariadne_graph::ReducedSymbolRecord {
                id: ariadne_index::SymbolId("sym1".to_string()),
                name: "sym1".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                defining_file: Some(file1),
                declarations: vec![],
                definition: None,
            }
        )]),
        edges: vec![],
        tu_usage: std::collections::BTreeMap::new(),
        file_symbols: std::collections::BTreeMap::new(),
    };

    let include_elements = build_include_elements(&include_graph, &std::collections::HashMap::new(), &[]);
    let call_elements = build_call_elements(&call_graph);
    let sym_elements = build_symbol_elements(&reduced_model);
    
    // Check DTO `path` field
    assert_eq!(call_elements[0].data["path"], "1:10"); // DTO populates path as file_id:line
    assert_eq!(sym_elements[0].data["path"], "1");

    let state = Arc::new(UiState {
        repo_root: std::path::PathBuf::from("."),
        include_graph,
        reduced_model,
        call_graph,
        payload: GraphExportPayload {
            repo_name: "test".to_string(),
            include_graph: include_elements,
            symbol_graph: sym_elements,
            call_graph: call_elements,
            stats: SummaryStats { total_files: 0, total_symbols: 0, total_calls: 0, cycle_count: 0 },
        },
    });

    let app = build_router(state);

    // Test wildcard
    let req = Request::builder().uri("/api/graph/search?q=*test*").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let items: Vec<ariadne_ui::SearchResultItem> = serde_json::from_slice(&body).unwrap();
    assert!(!items.is_empty(), "Expected wildcard match");

    // Test regex
    let req = Request::builder().uri("/api/graph/search?q=/^test/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let items: Vec<ariadne_ui::SearchResultItem> = serde_json::from_slice(&body).unwrap();
    assert!(!items.is_empty(), "Expected regex match");

    // Test path
    let req = Request::builder().uri("/api/graph/search?q=include/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let items: Vec<ariadne_ui::SearchResultItem> = serde_json::from_slice(&body).unwrap();
    assert!(!items.is_empty(), "Expected path match");
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

#[test]
fn test_ui_layout_and_interactive_traversal() {
    let html = ariadne_ui::INDEX_HTML;
    
    // 1. Initial active button is hopAllCap (formerly hopAll)
    assert!(html.contains("id=\"hopAllCap\" class=\"active\"") || html.contains("class=\"active\" id=\"hopAllCap\"") || html.contains("id=\"hopAllCap\" class=\"active\""));
    assert!(!html.contains("id=\"degMinus\" class=\"active\""));
    
    // 2. Layout wrapping logic
    assert!(html.contains("maxRowsPerCol"));
    assert!(html.contains("numSubCols"));
    
    // 3. findPath and showEntrypoints have non-empty implementations
    assert!(html.contains("fetch(`/api/graph/path"));
    assert!(html.contains("fetch(`/api/graph/entrypoints"));
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

#[test]
fn test_ui_filter_and_degree_stepper_elements() {
    let html = ariadne_ui::INDEX_HTML;
    
    // Degree stepper controls
    assert!(html.contains("id=\"degMinus\""));
    assert!(html.contains("id=\"degPlus\""));
    assert!(html.contains("id=\"degValue\""));
    
    // ALL-WITH-CAP toggle
    assert!(html.contains("id=\"hopAllCap\""));
    
    // Node Cap selector
    assert!(html.contains("id=\"nodeCapSelect\""));
    assert!(html.contains("value=\"25\""));
    assert!(html.contains("value=\"50\""));
    assert!(html.contains("value=\"100\""));
    assert!(html.contains("value=\"200\""));
    assert!(html.contains("value=\"500\""));
    
    // Search bar enhancements
    assert!(html.contains("id=\"searchClearBtn\""));
    assert!(html.contains("id=\"searchMatchCount\""));
    
    // Cap warning badge/indicator
    assert!(html.contains("id=\"capWarningBadge\""));
    
    // Clear filter button
    assert!(html.contains("id=\"btnClearFilter\""));
}

#[test]
fn test_ui_subgraph_filter_and_expansion_engine() {
    let html = ariadne_ui::INDEX_HTML;
    
    // Core engine functions
    assert!(html.contains("function compileMatcher"));
    assert!(html.contains("function matchesFilter"));
    assert!(html.contains("function filterAndExpandSubgraph"));
    
    // Degree stepper logic implementations
    assert!(html.contains("currentDegree = Math.max"));
    assert!(html.contains("textContent = currentDegree"));
    assert!(html.contains("isAllCapped = "));
    assert!(html.contains("nodeCap = cap"));
    
    // BFS checks
    assert!(html.contains("dirSelect"));
    assert!(html.contains("activeNodes"));
    assert!(html.contains("activeEdges"));
    assert!(html.contains("capWarningBadge').style.display"));
    
    // Matcher logic check
    assert!(html.contains("path:"));
    assert!(html.contains("kind:"));
}

#[test]
fn test_ui_dependency_tree_direction_preserving() {
    let html = ariadne_ui::INDEX_HTML;
    
    // Check dirSelect default is 'out' (Dependencies)
    assert!(html.contains("<option value=\"out\" selected>"));
    
    // Check BFS uses branchDir logic
    assert!(html.contains("let branchDir = 'both'"));
    assert!(html.contains("branchDir: nextDir"));
    assert!(html.contains("branchDir === 'both'"));
    assert!(html.contains("branchDir === 'out'"));
    assert!(html.contains("branchDir === 'in'"));
}

#[test]
fn test_build_symbol_elements_filters_unindexed() {
    let mut symbols = std::collections::BTreeMap::new();
    let sym_a = ariadne_index::SymbolId("A".to_string());
    let sym_b = ariadne_index::SymbolId("B".to_string());
    let sym_char = ariadne_index::SymbolId("char".to_string());
    
    symbols.insert(sym_a.clone(), ariadne_graph::ReducedSymbolRecord {
        id: sym_a.clone(),
        name: "A".to_string(),
        kind: ariadne_index::SymbolKind::Class,
        defining_file: None,
        declarations: vec![],
        definition: None,
    });
    symbols.insert(sym_b.clone(), ariadne_graph::ReducedSymbolRecord {
        id: sym_b.clone(),
        name: "B".to_string(),
        kind: ariadne_index::SymbolKind::Class,
        defining_file: None,
        declarations: vec![],
        definition: None,
    });

    let edges = vec![
        ariadne_graph::ReducedSemanticEdge {
            src_symbol: sym_a.clone(),
            dst_symbol: sym_b.clone(),
            kind: ariadne_index::DependencyKind::Complete,
        },
        ariadne_graph::ReducedSemanticEdge {
            src_symbol: sym_a.clone(),
            dst_symbol: sym_char.clone(),
            kind: ariadne_index::DependencyKind::Complete,
        },
    ];

    let model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols,
        edges,
        tu_usage: std::collections::BTreeMap::new(),
        file_symbols: std::collections::BTreeMap::new(),
    };

    let elements = build_symbol_elements(&model);
    
    // Nodes for A and B = 2 elements.
    // Edge A->B = 1 element.
    // Total should be 3.
    assert_eq!(elements.len(), 3, "Expected 2 nodes and 1 valid edge");

    // A's out_degree should be 1
    let node_a = elements.iter().find(|e| e.data["id"] == "A").unwrap();
    assert_eq!(node_a.data["out_degree"], 1, "A should have out_degree 1");
}
