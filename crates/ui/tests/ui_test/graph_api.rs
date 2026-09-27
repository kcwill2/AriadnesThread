use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_ui::{
    build_call_elements, build_include_elements, build_router, build_symbol_elements,
    GraphExportPayload, SummaryStats, UiState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;

#[tokio::test]
async fn test_server_traversal_routes() {
    let state = Arc::new(UiState::new_loaded(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        ReducedSemanticModel {
            repo_root: std::path::PathBuf::from("."),
            symbols: std::collections::BTreeMap::new(),
            edges: vec![],
            tu_usage: std::collections::BTreeMap::new(),
            file_symbols: std::collections::BTreeMap::new(),
        },
        CallGraph::default(),
        GraphExportPayload {
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
            file_justification: std::collections::HashMap::new(),
        },
    ));

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

fn given_mock_search_graphs() -> (IncludeGraph, CallGraph, ReducedSemanticModel) {
    // We will build a UI state with some mock elements to test search and DTO builder
    let mut include_graph = IncludeGraph::new();
    let file1 = ariadne_repo::FileId(1);
    include_graph
        .file_paths
        .insert(file1, std::path::PathBuf::from("include/test1.h"));

    let mut call_graph = CallGraph::default();
    call_graph.nodes.push(ariadne_graph::CallNode {
        symbol_id: ariadne_index::SymbolId("func1".to_string()),
        name: "func1".to_string(),
        owning_file: file1,
        location: Some(ariadne_index::types::FileLocation {
            file_id: file1,
            line: 10,
            column: 5,
        }),
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
            },
        )]),
        edges: vec![],
        tu_usage: std::collections::BTreeMap::new(),
        file_symbols: std::collections::BTreeMap::new(),
    };

    (include_graph, call_graph, reduced_model)
}

async fn assert_search_query_matches(app: axum::Router, uri: &str, query_kind: &str) {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let items: Vec<ariadne_ui::SearchResultItem> = serde_json::from_slice(&body).unwrap();
    assert!(!items.is_empty(), "Expected {} match", query_kind);
}

#[tokio::test]
async fn test_search_advanced_queries_and_dto_paths() {
    let (include_graph, call_graph, reduced_model) = given_mock_search_graphs();

    let include_elements =
        build_include_elements(&include_graph, &std::collections::HashMap::new(), &[]);
    let call_elements = build_call_elements(&call_graph);
    let sym_elements = build_symbol_elements(&reduced_model);

    // Check DTO `path` field
    assert_eq!(call_elements[0].data["path"], "1:10"); // DTO populates path as file_id:line
    assert_eq!(sym_elements[0].data["path"], "1");

    let state = Arc::new(UiState::new_loaded(
        std::path::PathBuf::from("."),
        include_graph,
        reduced_model,
        call_graph,
        GraphExportPayload {
            repo_name: "test".to_string(),
            include_graph: include_elements,
            symbol_graph: sym_elements,
            call_graph: call_elements,
            stats: SummaryStats {
                total_files: 0,
                total_symbols: 0,
                total_calls: 0,
                cycle_count: 0,
            },
            file_collateral: std::collections::HashMap::new(),
            file_justification: std::collections::HashMap::new(),
        },
    ));

    let app = build_router(state);

    assert_search_query_matches(app.clone(), "/api/graph/search?q=*test*", "wildcard").await;
    assert_search_query_matches(app.clone(), "/api/graph/search?q=/^test/", "regex").await;
    assert_search_query_matches(app, "/api/graph/search?q=include/", "path").await;
}

#[tokio::test]
async fn test_server_collateral_endpoint() {
    let mut file_collateral = HashMap::new();
    let mut collat = HashMap::new();
    collat.insert("2".to_string(), 5);
    file_collateral.insert("1".to_string(), collat);

    let state = Arc::new(UiState::new_loaded(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        ReducedSemanticModel {
            repo_root: std::path::PathBuf::from("."),
            symbols: std::collections::BTreeMap::new(),
            edges: vec![],
            tu_usage: std::collections::BTreeMap::new(),
            file_symbols: std::collections::BTreeMap::new(),
        },
        CallGraph::default(),
        GraphExportPayload {
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
            file_justification: HashMap::new(),
        },
    ));

    let app = build_router(state);

    let req = Request::builder()
        .uri("/api/graph/collateral?file=1")
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: ariadne_ui::CollateralResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(response.target_id, "1");
    // total_collateral, direct_collateral, transitive_collateral should be computed in the server logic
}

fn fixture_sample_state() -> Arc<UiState> {
    let (include_graph, mut payload) = common::fixture_include_graph();
    let (call_graph, call_payload) = common::fixture_call_graph();
    payload.call_graph = call_payload.call_graph;
    common::ready_state(
        std::path::PathBuf::from("."),
        include_graph,
        call_graph,
        payload,
    )
}

fn element_ids(elements: &serde_json::Value) -> Vec<String> {
    let mut ids: Vec<String> = elements
        .as_array()
        .expect("expected a JSON array of Cytoscape elements")
        .iter()
        .map(|element| {
            element["data"]["id"]
                .as_str()
                .expect("element id must be a string")
                .to_string()
        })
        .collect();
    ids.sort();
    ids
}

#[tokio::test]
async fn test_all_graph_payload_routes_return_503_while_loading() {
    let app = build_router(common::loading_state());
    let loading_uris = [
        "/api/graph/include",
        "/api/graph/symbol",
        "/api/graph/call",
        "/api/graph/traverse?graph_type=include&center=1",
        "/api/graph/path?graph_type=include&from=1&to=3",
        "/api/graph/entrypoints?graph_type=include",
        "/api/graph/search?q=main",
        "/api/graph/collateral?file=1",
    ];
    for uri in loading_uris {
        let (status, _headers, _body) = common::request_route(app.clone(), uri).await;
        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "{uri} must be unavailable while loading"
        );
    }
}

#[tokio::test]
async fn test_graph_traverse_returns_400_without_required_params() {
    let app = build_router(fixture_sample_state());
    for uri in [
        "/api/graph/traverse",
        "/api/graph/traverse?graph_type=include",
        "/api/graph/traverse?center=1",
    ] {
        let (status, _headers, _body) = common::request_route(app.clone(), uri).await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "{uri} must require both graph_type and center"
        );
    }
}

#[tokio::test]
async fn test_graph_traverse_include_depth_one_returns_center_and_neighbors() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) = common::request_route(
        app,
        "/api/graph/traverse?graph_type=include&center=1&depth=1",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let ids = element_ids(&body);
    assert!(ids.contains(&"1".to_string()) && ids.contains(&"2".to_string()));
    assert!(
        ids.contains(&"1-2".to_string()),
        "edge 1-2 must be included"
    );
    assert!(
        !ids.contains(&"3".to_string()),
        "depth 1 must not reach file 3"
    );
    assert!(
        !ids.contains(&"7".to_string()),
        "isolated file 7 must not appear"
    );
}

#[tokio::test]
async fn test_graph_traverse_include_depth_two_reaches_full_chain() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) = common::request_route(
        app,
        "/api/graph/traverse?graph_type=include&center=1&depth=2",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let ids = element_ids(&body);
    assert_eq!(
        ids,
        vec![
            "1".to_string(),
            "1-2".to_string(),
            "2".to_string(),
            "2-3".to_string(),
            "3".to_string()
        ]
    );
}

#[tokio::test]
async fn test_graph_traverse_include_with_non_numeric_center_yields_empty_array() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) = common::request_route(
        app,
        "/api/graph/traverse?graph_type=include&center=not-a-file-id",
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "FINDING: a non-numeric include center is silently accepted, not rejected"
    );
    assert_eq!(
        body,
        serde_json::json!([]),
        "invalid centers are indistinguishable from empty neighborhoods"
    );
}

#[tokio::test]
async fn test_graph_traverse_unknown_graph_type_yields_empty_array() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/traverse?graph_type=nope&center=1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!([]));
}

#[tokio::test]
async fn test_graph_path_returns_400_without_required_params() {
    let app = build_router(fixture_sample_state());
    for uri in [
        "/api/graph/path",
        "/api/graph/path?graph_type=include",
        "/api/graph/path?from=1&to=3",
    ] {
        let (status, _headers, _body) = common::request_route(app.clone(), uri).await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "{uri} must require graph_type, from and to"
        );
    }
}

#[tokio::test]
async fn test_graph_path_include_chain_returns_id_sequence() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/path?graph_type=include&from=1&to=3").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!(["1", "2", "3"]));
}

#[tokio::test]
async fn test_graph_path_include_non_numeric_endpoints_return_null() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/path?graph_type=include&from=a&to=3").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "FINDING: non-numeric include endpoints are not rejected as invalid input"
    );
    assert_eq!(
        body,
        serde_json::Value::Null,
        "invalid endpoints are indistinguishable from 'no path exists'"
    );
}

#[tokio::test]
async fn test_graph_path_unreachable_files_return_null() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/path?graph_type=include&from=1&to=7").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::Value::Null);
}

#[tokio::test]
async fn test_graph_entrypoints_returns_400_without_graph_type() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, _body) = common::request_route(app, "/api/graph/entrypoints").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_graph_entrypoints_chain_locks_roots_leaves_hubs() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/entrypoints?graph_type=include").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["graph_type"], "include");
    assert_eq!(body["cycles"], serde_json::json!([]));
    assert_eq!(
        element_ids(&body["roots"]),
        vec!["1".to_string(), "7".to_string()]
    );
    assert_eq!(
        element_ids(&body["leaves"]),
        vec!["3".to_string(), "7".to_string()]
    );
    assert_eq!(
        element_ids(&body["top_hubs"]),
        vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "7".to_string()
        ]
    );
}

#[tokio::test]
async fn test_graph_entrypoints_limit_truncates_each_list() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/entrypoints?graph_type=include&limit=1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(element_ids(&body["roots"]), vec!["1".to_string()]);
    assert_eq!(element_ids(&body["leaves"]), vec!["3".to_string()]);
    assert_eq!(element_ids(&body["top_hubs"]), vec!["2".to_string()]);
}

#[tokio::test]
async fn test_graph_entrypoints_reports_empty_cycles_even_for_cyclic_graph() {
    let (mut include_graph, payload) = common::fixture_include_graph();
    use ariadne_repo::FileId;
    include_graph.edges.push(ariadne_graph::IncludeEdge {
        source: FileId(3),
        destination: Some(FileId(1)),
        context_name: String::new(),
        source_location: ariadne_graph::SourceLocation { line: 1, column: 1 },
        kind: ariadne_graph::IncludeKind::Local,
        raw_path: String::new(),
        resolved_path: None,
    });
    let call_graph = CallGraph::default();
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        include_graph,
        call_graph,
        payload,
    ));

    let (status, _headers, body) =
        common::request_route(app, "/api/graph/entrypoints?graph_type=include").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["cycles"].as_array().unwrap().is_empty(),
        "FINDING: server.rs hardcodes cycles to [] (server.rs:347); the cycles key can never inform clients"
    );
    let isolated_only = vec!["7".to_string()];
    assert_eq!(
        element_ids(&body["roots"]),
        isolated_only.clone(),
        "only the isolated file has zero in-degree once 1-2-3-1 is cyclic"
    );
    assert_eq!(
        element_ids(&body["leaves"]),
        isolated_only,
        "only the isolated file has zero out-degree once 1-2-3-1 is cyclic"
    );
}

#[tokio::test]
async fn test_graph_entrypoints_unknown_graph_type_returns_empty_sets() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/entrypoints?graph_type=nope").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["graph_type"], "nope",
        "unknown graph_type is echoed back unvalidated"
    );
    let empty: Vec<String> = vec![];
    assert_eq!(element_ids(&body["roots"]), empty);
    assert_eq!(element_ids(&body["leaves"]), empty);
    assert_eq!(element_ids(&body["top_hubs"]), empty);
    assert_eq!(body["cycles"], serde_json::json!([]));
}

#[tokio::test]
async fn test_graph_search_returns_400_without_q() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, _body) = common::request_route(app, "/api/graph/search").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_graph_search_locks_result_item_shape_and_skips_edges() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app.clone(), "/api/graph/search?q=a.h").await;
    assert_eq!(status, StatusCode::OK);
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 1, "exactly the a.h node must match q=a.h");
    assert_eq!(items[0]["id"], "1");
    assert_eq!(items[0]["label"], "a.h");
    assert_eq!(items[0]["kind"], "HEADER");
    assert_eq!(items[0]["graph_type"], "include");

    let (_status, _headers, edge_query) =
        common::request_route(app, "/api/graph/search?q=1-2").await;
    assert_eq!(
        edge_query.as_array().unwrap().len(),
        0,
        "search matches label/path only; edge ids are never searchable"
    );
}

#[tokio::test]
async fn test_graph_search_degenerate_node_data_yields_empty_id_and_kind() {
    // Pass 4 D2: production dto builders always emit string id/kind, but the
    // handler must stay total on degenerate data: missing or non-string id/kind
    // keys yield "" result fields (the closure's unwrap_or_default convention
    // for label/path), never a panic.
    let mut payload = common::empty_export_payload();
    payload.include_graph.push(ariadne_ui::CytoscapeElement {
        data: serde_json::json!({ "label": "ghost.h" }),
        classes: None,
    });
    payload.include_graph.push(ariadne_ui::CytoscapeElement {
        data: serde_json::json!({ "id": 42, "label": "numid.h", "kind": true }),
        classes: None,
    });
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        payload,
    ));

    let (status, _headers, body) =
        common::request_route(app, "/api/graph/search?q=.h&graph_type=include").await;
    assert_eq!(status, StatusCode::OK);
    let items = body.as_array().unwrap();
    assert_eq!(
        items.len(),
        2,
        "degenerate nodes are still searched and returned"
    );
    assert_eq!(items[0]["id"], "");
    assert_eq!(items[0]["kind"], "");
    assert_eq!(items[0]["label"], "ghost.h");
    assert_eq!(
        items[1]["id"], "",
        "non-string id falls back to empty string"
    );
    assert_eq!(
        items[1]["kind"], "",
        "non-string kind falls back to empty string"
    );
    assert_eq!(items[1]["label"], "numid.h");
}

#[tokio::test]
async fn test_graph_search_respects_limit_param_and_default_limit() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, all) =
        common::request_route(app.clone(), "/api/graph/search?q=&limit=99").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        all.as_array().unwrap().len(),
        6,
        "empty q matches every node (4 include + 2 call) via substring semantics"
    );

    let (_status, _headers, capped) =
        common::request_route(app, "/api/graph/search?q=&limit=1").await;
    assert_eq!(capped.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_graph_search_graph_type_filter_limits_scope() {
    let app = build_router(fixture_sample_state());
    let (_status, _headers, unfiltered) =
        common::request_route(app.clone(), "/api/graph/search?q=main").await;
    let items = unfiltered.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["graph_type"], "call");
    assert_eq!(items[0]["id"], "main");

    let (_status, _headers, include_only) =
        common::request_route(app.clone(), "/api/graph/search?q=main&graph_type=include").await;
    assert_eq!(
        include_only.as_array().unwrap().len(),
        0,
        "main is not an include-graph node"
    );

    let (_status, _headers, call_only) =
        common::request_route(app, "/api/graph/search?q=main&graph_type=call").await;
    assert_eq!(call_only.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_graph_collateral_returns_400_without_file_param() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, _body) = common::request_route(app, "/api/graph/collateral").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_graph_collateral_non_numeric_file_returns_zeroed_response() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/collateral?file=not-a-number").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["target_id"], "not-a-number",
        "the raw file value is echoed back"
    );
    assert_eq!(body["total_collateral"], 0);
    assert_eq!(body["direct_collateral"], serde_json::json!({}));
    assert_eq!(body["transitive_collateral"], serde_json::json!({}));
}

#[tokio::test]
async fn test_graph_collateral_unknown_numeric_file_returns_zeroed_response() {
    let app = build_router(fixture_sample_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/collateral?file=999").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["target_id"], "999");
    assert_eq!(body["total_collateral"], 0);
}

#[tokio::test]
async fn test_graph_collateral_invalid_file_returns_200_zeros_even_while_loading() {
    let app = build_router(common::loading_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/collateral?file=not-a-number").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "FINDING: file parsing happens before the 503 readiness gate, so an invalid file id \
         returns a 200 zeroed payload even while the server is still loading"
    );
    assert_eq!(body["total_collateral"], 0);
}
