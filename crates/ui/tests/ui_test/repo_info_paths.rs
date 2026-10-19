//! UTA2-R1 (audit rows 3-4): `/api/repo-info` payload contract.
//! UTB-3 red: `file_paths` documented "relative paths" emitted numeric
//! FileId strings, defeating the endpoint's cross-environment purpose.
//! Consumer greps (extension host, ui assets, ui_test, integration, tools)
//! in tasks/uta2-r1-journal.md: nothing consumes the old numeric values.

use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph};
use ariadne_ui::build_router;
use axum::http::StatusCode;

/// repo_root with one under-root path, one deeper under-root path, and one
/// path OUTSIDE the root (cross-environment fallback keeps the full string).
fn repo_info_state() -> axum::Router {
    let mut graph = IncludeGraph::new();
    for (id, path) in [
        (3u64, "repo/inc/x.h"),
        (9, "repo/y.cc"),
        (12, "elsewhere/z.h"),
    ] {
        graph
            .file_paths
            .insert(ariadne_repo::FileId(id), std::path::PathBuf::from(path));
    }
    build_router(common::ready_state(
        std::path::PathBuf::from("repo"),
        graph,
        CallGraph::default(),
        common::empty_export_payload(),
    ))
}

#[tokio::test]
async fn test_repo_info_file_paths_are_relative_paths_not_ids() {
    // RED (UTB-3): values are repo-relative path strings, sorted for
    // determinism; a path outside repo_root keeps its full lossy string.
    // Pre-fix this endpoint emitted ["12","3","9"] (numeric FileId strings).
    let app = repo_info_state();
    let (status, _headers, body) = common::request_route(app, "/api/repo-info").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["repo_root"], "repo",
        "repo_root stays the server's root string"
    );
    assert_eq!(
        body["file_paths"],
        serde_json::json!(["elsewhere/z.h", "inc/x.h", "y.cc"]),
        "file_paths must carry the documented repo-relative path values"
    );
}

#[tokio::test]
async fn test_repo_info_serves_empty_mapping_while_loading() {
    let app = build_router(common::loading_state());
    let (status, _headers, body) = common::request_route(app, "/api/repo-info").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repo_root"], ".");
    assert_eq!(
        body["file_paths"],
        serde_json::json!([]),
        "no analysis yet: empty mapping, not an error (liveness-probe contract)"
    );
}

#[tokio::test]
async fn test_repo_info_loaded_but_empty_graph_is_empty_mapping() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));
    let (status, _headers, body) = common::request_route(app, "/api/repo-info").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["file_paths"], serde_json::json!([]));
}
