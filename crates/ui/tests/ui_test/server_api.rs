use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_ui::{build_router, GraphExportPayload, SummaryStats, UiState};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;

#[tokio::test]
async fn test_server_routes() {
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
        },
    ));

    let app = build_router(state);

    let req = Request::builder()
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_async_ui_state_loading() {
    let state = Arc::new(UiState::new_loading(std::path::PathBuf::from(".")));
    let app = build_router(state.clone());

    // Before loading, status should be loading
    let req = Request::builder().uri("/api/status").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let status: ariadne_ui::server::StatusResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(status.progress.state, "loading");
    assert_eq!(status.progress.percent, 0);

    // Graph endpoints should return 503 or empty array when loading. Let's say empty array as per unit 1 or 503.
    let req = Request::builder().uri("/api/graph/include").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);

    // Populate data
    {
        let mut progress = state.progress.write().unwrap();
        progress.state = "ready".to_string();
        progress.percent = 100;
        *state.payload.write().unwrap() = Some(GraphExportPayload {
            repo_name: "test".to_string(),
            include_graph: vec![],
            symbol_graph: vec![],
            call_graph: vec![],
            stats: SummaryStats { total_files: 0, total_symbols: 0, total_calls: 0, cycle_count: 0 },
            file_collateral: HashMap::new(),
        });
        *state.include_graph.write().unwrap() = Some(IncludeGraph::new());
        *state.reduced_model.write().unwrap() = Some(ReducedSemanticModel {
            repo_root: std::path::PathBuf::from("."),
            symbols: std::collections::BTreeMap::new(),
            edges: vec![],
            tu_usage: std::collections::BTreeMap::new(),
            file_symbols: std::collections::BTreeMap::new(),
        });
        *state.call_graph.write().unwrap() = Some(CallGraph::default());
    }

    // Now status should be ready
    let req = Request::builder().uri("/api/status").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let status: ariadne_ui::server::StatusResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(status.progress.state, "ready");
    
    // Graph endpoints should now return OK
    let req = Request::builder().uri("/api/graph/include").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_server_index_route() {
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
        },
    ));

    let app = build_router(state);

    let req = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get("Content-Type").unwrap(), "text/html; charset=utf-8");
    assert_eq!(res.headers().get("Cache-Control").unwrap(), "no-store",
        "index must never be cached: stale HTML silently hides new UI features");
}

#[tokio::test]
async fn test_index_route_body_matches_bundled_index_html() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let (status, headers, body) = common::request_raw_body(app, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get("Cache-Control").unwrap(), "no-store");
    assert_eq!(headers.get("Content-Type").unwrap(), "text/html; charset=utf-8");
    assert_eq!(
        body,
        ariadne_ui::INDEX_HTML.as_bytes(),
        "index route must serve the exact bundled INDEX_HTML"
    );
}

#[tokio::test]
async fn test_unknown_path_returns_404() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let (status, _headers, _json) = common::request_route(app, "/api/no-such-route").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_status_route_rejects_post_with_method_not_allowed() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let request = Request::builder()
        .method("POST")
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_status_route_locks_ready_response_shape() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let (status, _headers, body) = common::request_route(app, "/api/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repo_name"], "fixture-repo");
    assert_eq!(body["progress"]["state"], "ready");
    assert_eq!(body["progress"]["percent"], 100);
    assert!(body["stats"].is_object(), "stats must be an object when loaded");
    assert_eq!(body["stats"]["total_files"], 0);
    assert_eq!(body["stages"], serde_json::json!([]));
    assert_eq!(body["last_refresh"], serde_json::Value::Null);
    assert_eq!(body["rebuilding"], false);
    assert_eq!(body["reused_tus"], 0);
    assert_eq!(body["reanalyzed_tus"], 0);
}

#[tokio::test]
async fn test_export_route_locks_payload_shape_and_omits_empty_collateral() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let (status, _headers, body) = common::request_route(app, "/api/export").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repo_name"], "fixture-repo");
    assert!(body["include_graph"].is_array());
    assert!(body["symbol_graph"].is_array());
    assert!(body["call_graph"].is_array());
    assert!(body["stats"].is_object());
    assert!(
        body.get("file_collateral").is_none(),
        "empty file_collateral is skipped during serialization; clients must default it"
    );
}

#[tokio::test]
async fn test_export_route_includes_file_collateral_when_non_empty() {
    let mut payload = common::empty_export_payload();
    let mut burden = HashMap::new();
    burden.insert("2".to_string(), 5usize);
    payload.file_collateral.insert("1".to_string(), burden);

    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        payload,
    ));

    let (status, _headers, body) = common::request_route(app, "/api/export").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["file_collateral"]["1"]["2"], 5);
}

#[tokio::test]
async fn test_export_route_returns_503_while_loading() {
    let app = build_router(common::loading_state());

    let (status, _headers, _json) = common::request_route(app, "/api/export").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}
