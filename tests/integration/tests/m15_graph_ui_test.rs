use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_m15_graph_ui_command() {
    let repo_dir = PathBuf::from("../../tests/fixtures/basic");
    let export_dir = tempdir().unwrap();
    let export_file = export_dir.path().join("export.html");

    // 1. Test standalone HTML export
    let result = ariadne_cli::run_ui(
        Some(repo_dir.clone()),
        0, // dummy port
        true,
        Some(export_file.clone()),
    ).await;
    
    assert!(result.is_ok(), "run_ui failed for export");
    
    // Verify file exists and has content
    assert!(export_file.exists(), "Export file should exist");
    let content = fs::read_to_string(&export_file).unwrap();
    assert!(content.len() > 1024, "Export file too small");
    assert!(content.contains("window.__ARIADNE_DATA__"), "Export missing data payload");
    
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
        ).await;
        assert!(result.is_ok());
    });
    
    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Make requests to test endpoints
    let base_url = format!("http://127.0.0.1:{}", port);
    
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
