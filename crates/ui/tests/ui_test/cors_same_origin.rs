//! AR-02 security lock (S6 HIGH): CORS must not be permissive.
//!
//! A page from a foreign Origin must never receive an
//! `Access-Control-Allow-Origin` grant from this server; only an Origin
//! exactly equal to the scheme + Host it was sent to is mirrored back.
//! Before AR-02 this router served `CorsLayer::permissive()`, which let any
//! site a victim visited read the UI's data through their browser.

use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph};
use ariadne_ui::build_router;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

async fn allow_origin_header(request_host: &str, request_origin: &str) -> Option<String> {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));
    let request = Request::builder()
        .method("GET")
        .uri("/api/status")
        .header("Host", request_host)
        .header("Origin", request_origin)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response
        .headers()
        .get("access-control-allow-origin")
        .map(|value| value.to_str().unwrap().to_string())
}

#[tokio::test]
async fn test_cors_denies_foreign_origin_request() {
    assert_eq!(
        allow_origin_header("localhost:7742", "http://evil.example.com").await,
        None,
        "a foreign Origin must not receive an allow-origin grant \
         (S6: this route served CorsLayer::permissive)"
    );
}

#[tokio::test]
async fn test_cors_denies_same_host_with_mismatched_port() {
    assert_eq!(
        allow_origin_header("localhost:7742", "http://localhost:9999").await,
        None,
        "the same host on another port is still a cross-origin request"
    );
}

#[tokio::test]
async fn test_cors_mirrors_same_origin_request() {
    assert_eq!(
        allow_origin_header("localhost:7742", "http://localhost:7742")
            .await
            .as_deref(),
        Some("http://localhost:7742"),
        "an Origin equal to the request Host it was sent to must be allowed"
    );
}

/// UTA2-R1 item 11: the https arm of the predicate. The predicate accepts
/// both `http://` and `https://` forms of the exact Host; a TLS-fronted
/// deployment must not silently deny its own origin.
#[tokio::test]
async fn test_cors_mirrors_https_scheme_origin() {
    assert_eq!(
        allow_origin_header("localhost:7742", "https://localhost:7742")
            .await
            .as_deref(),
        Some("https://localhost:7742"),
        "the https scheme of the exact host:port is the same origin (S6 arm pin)"
    );
}

/// UTA2-R1 item 11: the missing-Host arm. Without a Host header there is no
/// scheme+Host to compare against, so NOTHING can be mirrored back - the
/// deny path must not fall open.
#[tokio::test]
async fn test_cors_denies_request_without_host_header() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));
    let request = Request::builder()
        .method("GET")
        .uri("/api/status")
        // deliberately no Host header: the predicate has nothing to match
        .header("Origin", "http://localhost:7742")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("access-control-allow-origin"),
        None,
        "a request without Host must never receive an allow-origin grant"
    );
}
