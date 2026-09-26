use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use tempfile::tempdir;
use ariadne_cli::run_ui;
use serde_json::Value;

#[tokio::test]
async fn test_e2e_ui_traversal_and_pathfinding() {
    let repo_dir = PathBuf::from("../../tests/fixtures/basic");
    
    // Bind to port 0 to get an ephemeral port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Free the port
    
    let server_task = tokio::spawn(async move {
        let result = run_ui(
            Some(repo_dir.clone()),
            port,
            true, // no open
            None,
            2,
            true, // no watch
        ).await;
        assert!(result.is_ok());
    });
    
    let base_url = format!("http://127.0.0.1:{}", port);
    for _ in 0..120 {
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
        let Ok(resp) = reqwest::get(format!("{}/api/status", base_url)).await else {
            continue;
        };
        if !resp.status().is_success() {
            continue;
        }
        let status: Value = resp.json().await.unwrap_or(Value::Null);
        if status["progress"]["state"].as_str() == Some("ready") {
            break;
        }
    }
    
    // 1. Get a valid node ID first
    let include_resp = reqwest::get(&format!("{}/api/graph/include", base_url))
        .await
        .expect("Failed to request include endpoint");
    assert!(include_resp.status().is_success());
    let include_json: Value = include_resp.json().await.unwrap();
    let valid_node_id = include_json.as_array().unwrap()
        .iter()
        .find(|el| el.get("data").unwrap().get("source").is_none()) // find a node, not an edge
        .unwrap()
        .get("data").unwrap().get("id").unwrap().as_str().unwrap();

    // 2. Verify /api/graph/traverse (1-hop)
    let traverse_resp = reqwest::get(&format!("{}/api/graph/traverse?graph_type=include&center={}&depth=1&direction=both", base_url, valid_node_id))
        .await
        .expect("Failed to request traverse endpoint");
    assert!(traverse_resp.status().is_success());
    let traverse_json: Value = traverse_resp.json().await.unwrap();
    assert!(traverse_json.as_array().unwrap().len() > 0, "Traverse 1-hop should return nodes and edges");
    
    // 2b. Verify /api/graph/traverse (2-hop)
    let traverse2_resp = reqwest::get(&format!("{}/api/graph/traverse?graph_type=include&center={}&depth=2&direction=both", base_url, valid_node_id))
        .await
        .expect("Failed to request traverse endpoint");
    assert!(traverse2_resp.status().is_success());
    let traverse2_json: Value = traverse2_resp.json().await.unwrap();
    assert!(traverse2_json.as_array().unwrap().len() >= traverse_json.as_array().unwrap().len(), "Traverse 2-hop should return at least as many nodes as 1-hop");
    
    let valid_node_id_2 = include_json.as_array().unwrap()
        .iter()
        .filter(|el| el.get("data").unwrap().get("source").is_none()) // find nodes
        .nth(1) // get second node
        .unwrap()
        .get("data").unwrap().get("id").unwrap().as_str().unwrap();

    // 3. Verify /api/graph/path
    let path_resp = reqwest::get(&format!("{}/api/graph/path?graph_type=include&from={}&to={}", base_url, valid_node_id, valid_node_id_2))
        .await
        .expect("Failed to request path endpoint");
    assert!(path_resp.status().is_success());
    let path_json: Value = path_resp.json().await.unwrap();
    // Assuming path exists or doesn't, but the endpoint must return a valid JSON array or null
    assert!(path_json.is_array() || path_json.is_null());
    
    // 3. Verify /api/graph/entrypoints
    let ep_resp = reqwest::get(&format!("{}/api/graph/entrypoints?graph_type=include&limit=5", base_url))
        .await
        .expect("Failed to request entrypoints endpoint");
    assert!(ep_resp.status().is_success());
    let ep_json: Value = ep_resp.json().await.unwrap();
    assert!(ep_json.get("roots").is_some(), "Entrypoints should have roots");
    assert!(ep_json.get("top_hubs").is_some(), "Entrypoints should have hubs");
    
    // 4. Verify /api/graph/search
    let search_resp = reqwest::get(&format!("{}/api/graph/search?q=main", base_url))
        .await
        .expect("Failed to request search endpoint");
    assert!(search_resp.status().is_success());
    let search_json: Value = search_resp.json().await.unwrap();
    let search_arr = search_json.as_array().unwrap();
    assert!(search_arr.len() > 0, "Search should return matching items");
    
    server_task.abort();
}

#[tokio::test]
async fn test_e2e_standalone_html_export_traversal() {
    let repo_dir = PathBuf::from("../../tests/fixtures/basic");
    let export_dir = tempdir().unwrap();
    let export_file = export_dir.path().join("export_traversal.html");

    let result = run_ui(
        Some(repo_dir.clone()),
        0, // dummy port
        true,
        Some(export_file.clone()),
        2,
        true,
    ).await;
    
    assert!(result.is_ok(), "run_ui failed for export");
    
    assert!(export_file.exists(), "Export file should exist");
    let content = fs::read_to_string(&export_file).unwrap();
    assert!(content.len() > 1024, "Export file too small");
    assert!(content.contains("window.__ARIADNE_DATA__"), "Export missing data payload");
    
    // Check for traversal engine functions
    assert!(content.contains("clientTraverseNeighborhood"), "Missing clientTraverseNeighborhood");
    assert!(content.contains("clientShortestPath"), "Missing clientShortestPath");
    assert!(content.contains("clientGetEntrypoints"), "Missing clientGetEntrypoints");
    assert!(content.contains("ariadneThread"), "Missing ariadneThread entry point");
}
