use std::fs;
use std::net::TcpListener;
use tempfile::tempdir;

mod common;
use common::fixture_path;

/// Poll `/api/status` until the artifact-first boot reports ready (poll-first
/// idiom, mirrors m21_run_ui_smoke `poll_until_ready`): probe immediately,
/// only sleep BETWEEN attempts. Same 120 x 250 ms budget as the original
/// fixed-sleep loop; returns the last successful status snapshot (Null when
/// the endpoint never answered). Readiness itself is asserted by the caller.
async fn poll_until_ready_status(base_url: &str) -> serde_json::Value {
    let mut status_json = serde_json::Value::Null;
    for _ in 0..120 {
        let resp = match reqwest::get(format!("{}/api/status", base_url)).await {
            Ok(r) if r.status().is_success() => r,
            _ => {
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                continue;
            }
        };
        status_json = resp.json().await.unwrap_or(serde_json::Value::Null);
        if status_json["progress"]["state"].as_str() == Some("ready") {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    }
    status_json
}

#[tokio::test]
async fn test_m15_graph_ui_command() {
    let repo_dir = fixture_path("basic");
    let export_dir = tempdir().unwrap();
    let export_file = export_dir.path().join("export.html");

    // 1. Test standalone HTML export
    let result = ariadne_cli::run_ui(
        Some(repo_dir.clone()),
        0, // dummy port
        true,
        Some(export_file.clone()),
        2,
        true,
        None, // S6: keep the 127.0.0.1 bind default
    )
    .await;

    assert!(result.is_ok(), "run_ui failed for export");

    // Verify file exists and has content
    assert!(export_file.exists(), "Export file should exist");
    let content = fs::read_to_string(&export_file).unwrap();
    assert!(content.len() > 1024, "Export file too small");
    assert!(
        content.contains("window.__ARIADNE_DATA__"),
        "Export missing data payload"
    );

    // 2. Test HTTP server running
    // We bind to port 0 to get an ephemeral port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Free the port so the server can use it

    let server_task = tokio::spawn(async move {
        let result = ariadne_cli::run_ui(
            Some(repo_dir.clone()),
            port,
            true, // no open
            None,
            2,
            true, // no watch: keep the test deterministic
            None, // S6: keep the 127.0.0.1 bind default
        )
        .await;
        assert!(result.is_ok());
    });

    // Poll the artifact-first boot until ready (AR-17: no leading fixed sleep).
    let base_url = format!("http://127.0.0.1:{}", port);
    let status_json = poll_until_ready_status(&base_url).await;
    assert_eq!(
        status_json["progress"]["state"].as_str(),
        Some("ready"),
        "UI should report ready after artifact-first startup"
    );
    assert!(
        status_json["last_refresh"].as_u64().unwrap_or(0) > 0,
        "ready status should carry last_refresh"
    );

    // Test status endpoint
    let status_resp = reqwest::get(&format!("{}/api/status", base_url))
        .await
        .expect("Failed to request status endpoint");
    assert!(status_resp.status().is_success());

    // Test include graph endpoint
    let include_resp = reqwest::get(&format!("{}/api/graph/include", base_url))
        .await
        .expect("Failed to request include endpoint");
    assert!(include_resp.status().is_success());

    // Test symbol graph endpoint
    let symbol_resp = reqwest::get(&format!("{}/api/graph/symbol", base_url))
        .await
        .expect("Failed to request symbol endpoint");
    assert!(symbol_resp.status().is_success());

    // Test call graph endpoint
    let call_resp = reqwest::get(&format!("{}/api/graph/call", base_url))
        .await
        .expect("Failed to request call endpoint");
    assert!(call_resp.status().is_success());

    // We don't gracefully shut down in this simple test, but aborting the task is fine
    server_task.abort();
}
