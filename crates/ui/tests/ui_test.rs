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
            file_collateral: std::collections::HashMap::new(),
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
            file_collateral: std::collections::HashMap::new(),
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
            file_collateral: std::collections::HashMap::new(),
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
        file_collateral: std::collections::HashMap::new(),
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
            file_collateral: std::collections::HashMap::new(),
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
        file_collateral: std::collections::HashMap::new(),
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
fn test_ui_selected_node_focal_priority() {
    let html = ariadne_ui::INDEX_HTML;
    
    // selectedNodeId should take precedence over q when determining seeds
    assert!(html.contains("if (selectedNodeId) {"));
    assert!(html.contains("} else if (q) {"));
    
    // selectNode should trigger filterAndExpandSubgraph regardless of search input
    assert!(html.contains("if (!skipFilter) {"));
    
    // btnShowAllMatches should exist
    assert!(html.contains("id=\"btnShowAllMatches\""));
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

#[test]
fn test_file_collateral_dto_serialization() {
    let mut file_collateral = HashMap::new();
    let mut collat = HashMap::new();
    collat.insert("2".to_string(), 5);
    file_collateral.insert("1".to_string(), collat);

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
        file_collateral,
    };

    let serialized = serde_json::to_string(&payload).unwrap();
    assert!(serialized.contains(r#""file_collateral":{"1":{"2":5}}"#));

    let deserialized: GraphExportPayload = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.file_collateral.get("1").unwrap().get("2"), Some(&5));
}

#[tokio::test]
async fn test_server_collateral_endpoint() {
    let mut file_collateral = HashMap::new();
    let mut collat = HashMap::new();
    collat.insert("2".to_string(), 5);
    file_collateral.insert("1".to_string(), collat);

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
            file_collateral,
        },
    });

    let app = build_router(state);

    let req = Request::builder()
        .uri("/api/graph/collateral?file=1")
        .body(Body::empty())
        .unwrap();
    
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let response: ariadne_ui::CollateralResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(response.target_id, "1");
    // total_collateral, direct_collateral, transitive_collateral should be computed in the server logic
}

#[test]
fn test_ui_noticeable_fading_elements() {
    let html = ariadne_ui::INDEX_HTML;
    
    assert!(html.contains(".node-group.faded") && html.contains("opacity: 0.12"));
    assert!(html.contains(".edge-path.faded") && html.contains("opacity: 0.06"));
    assert!(html.contains(".node-group.heat-root"));
    
    // Assert INDEX_HTML contains logic applying faded to non-dependencies when a node is selected in the include graph, 
    // and clearing faded on canvas click or Esc.
    assert!(html.contains("currentTab === 'include'") && html.contains("selectedNodeId"));
    assert!(html.contains("reachableDependencies.add") || html.contains("reachableDependencies"));
    assert!(html.contains("faded"));
}

#[test]
fn test_ui_heat_map_coloring() {
    let html = ariadne_ui::INDEX_HTML;

    // Heat-critical CSS class for drop-shadow glow
    assert!(html.contains(".node-group.heat-critical"), "missing .node-group.heat-critical CSS");

    // getHeatColor function must exist and derive color from collateral ratio
    assert!(html.contains("getHeatColor"), "missing getHeatColor function");

    // Heat tier thresholds: 5-tier (0, 0.15, 0.35, 0.60, 0.85)
    assert!(html.contains("0.15") && html.contains("0.35") && html.contains("0.60") && html.contains("0.85"),
        "missing heat tier thresholds");

    // graphData.file_collateral used to retrieve collateral counts
    assert!(html.contains("file_collateral"), "missing file_collateral lookup");

    // Dynamic legend switch to heat mode when selectedNodeId is active in include graph
    assert!(html.contains("updateHeatLegend") || html.contains("Heat Map"),
        "missing dynamic legend heat mode");

    // Heat badges on node cards ([ROOT] pill and +XX col badge)
    assert!(html.contains("[ROOT]") && html.contains("col") && html.contains("heat-badge"),
        "missing heat badges on node cards");
}

#[test]
fn test_ui_inspector_collateral_breakdown_and_path_illumination() {
    let html = ariadne_ui::INDEX_HTML;

    // Inspector collateral breakdown list container
    assert!(html.contains("inspCollateralList"), "missing inspCollateralList container in HTML");

    // Sorting by highest collateral first
    assert!(html.contains(".col - a.col") || html.contains("b.col - a.col"),
        "missing descending collateral sort in inspector");

    // Collateral summary update in inspector insight
    assert!(html.contains("Total Collateral:") || html.contains("Top contributor:"),
        "missing collateral summary in inspector insight");

    // Path illumination helper to trace path from selected root to dependency
    assert!(html.contains("illuminatePath") || html.contains("findIncludePath"),
        "missing path illumination logic");
}

#[test]
fn test_onboarding_and_help_content() {
    let html = ariadne_ui::INDEX_HTML;

    assert!(html.contains("id=\"tourOverlay\""));
    assert!(html.contains("id=\"tourCard\""));
    assert!(html.contains("id=\"welcomeModal\""));
    assert!(html.contains("id=\"helpCenterModal\""));
    assert!(html.contains("id=\"btnHelp\""));
    
    // Check for tour/help center content covering required features
    assert!(html.contains("Collateral Heat Map"));
    assert!(html.contains("Path Illumination"));
    assert!(html.contains("Degree"));
    assert!(html.contains("Breadcrumbs") || html.contains("Ariadne's Thread"));
    assert!(html.contains("Direction"));
    assert!(html.contains("ALL-WITH-CAP"));
    assert!(html.contains("Cold/Teal")); // Heat map legend colors
    assert!(html.contains("Finding & Decoupling Header Bloat"));
    assert!(html.contains("Isolating Focused Subgraphs"));
    assert!(html.contains("Breaking Circular Dependencies"));

    assert!(html.contains("function startTour"));
    assert!(html.contains("function triggerFirstVisitExperience"));
    assert!(html.contains("function openHelpCenter"));
}

#[test]
fn test_tour_overlay_dismissal_and_positioning() {
    let html = ariadne_ui::INDEX_HTML;
    
    // Verify positionTourCard function exists and clamps
    assert!(html.contains("function positionTourCard("));
    assert!(html.contains("Math.max("));
    assert!(html.contains("Math.min(winH"));
    assert!(html.contains("Math.min(winW"));
    
    // Verify tourOverlay has an onclick dismissal
    assert!(html.contains("onclick=\"endTour()\"") && html.contains("id=\"tourOverlay\""));

    // Verify .tour-highlighted-element CSS
    assert!(html.contains(".tour-highlighted-element"));
    assert!(html.contains("z-index: 1002 !important"));
    assert!(html.contains("outline: 2px solid var(--cyan) !important"));

    // Verify tourCard smooth transition
    assert!(html.contains("transition: top 0.25s cubic-bezier"));

    // Verify btnTourPrev hidden on step 0
}

#[test]
fn test_ui_loading_overlay_and_progress() {
    let html = ariadne_ui::INDEX_HTML;
    
    assert!(html.contains("id=\"loadingOverlay\""));
    assert!(html.contains("id=\"loadingProgress\""));
    assert!(html.contains("id=\"loadingText\""));
    
    assert!(html.contains("function showProgress"));
    assert!(html.contains("function updateProgress"));
    assert!(html.contains("function hideProgress"));
    
    // Check integration in init()
    assert!(html.contains("[1/3] Loading graph datasets... 25%"));
    assert!(html.contains("[2/3] Processing graph models... 65%"));
    assert!(html.contains("[3/3] Rendering visualization... 100%"));
}
