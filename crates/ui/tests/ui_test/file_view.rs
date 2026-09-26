use crate::common;
use ariadne_ui::build_router;
use axum::http::StatusCode;

/// Source-view fixture: a temp "project" with `src/widget.cc`, served as repo_root.
fn source_view_app() -> (tempfile::TempDir, axum::Router) {
    let project_dir = tempfile::tempdir().unwrap();
    let nested_dir = project_dir.path().join("src");
    std::fs::create_dir(&nested_dir).unwrap();
    std::fs::write(nested_dir.join("widget.cc"), "// widget source\n").unwrap();

    let state = common::ready_state(
        project_dir.path().to_path_buf(),
        ariadne_graph::IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        common::empty_export_payload(),
    );
    (project_dir, build_router(state))
}

#[tokio::test]
async fn test_source_route_returns_file_content() {
    let (_project_dir, app) = source_view_app();
    let (status, headers, body) =
        common::request_raw_body(app, "/api/source?file=src/widget.cc").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers.get("Content-Type").unwrap(),
        "text/plain; charset=utf-8"
    );
    assert_eq!(body, b"// widget source\n".to_vec());
}

#[tokio::test]
async fn test_source_route_missing_file_returns_404() {
    let (_project_dir, app) = source_view_app();
    let (status, _headers, _body) =
        common::request_raw_body(app, "/api/source?file=src/missing.cc").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_source_route_missing_directory_returns_404() {
    let (_project_dir, app) = source_view_app();
    let (status, _headers, _body) =
        common::request_raw_body(app, "/api/source?file=nope/widget.cc").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_source_route_returns_400_without_file_param() {
    let (_project_dir, app) = source_view_app();
    let (status, _headers, _body) = common::request_raw_body(app, "/api/source").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_index_html_source_drawer_uses_source_route() {
    let html = common::page_content();

    assert!(
        html.contains("id=\"sourceDrawer\""),
        "missing source drawer container"
    );
    assert!(
        html.contains("openSourceDrawer()"),
        "missing openSourceDrawer handler"
    );
    assert!(
        html.contains("closeSourceDrawer()"),
        "missing closeSourceDrawer handler"
    );
    assert!(
        html.contains("id=\"drawerTitle\""),
        "missing drawer title element"
    );
    assert!(
        html.contains("fetch(`/api/source?file="),
        "source drawer must fetch /api/source with an encoded file path"
    );
    assert!(
        html.contains("sourceLoadError"),
        "source drawer must surface load errors to the user"
    );
}
