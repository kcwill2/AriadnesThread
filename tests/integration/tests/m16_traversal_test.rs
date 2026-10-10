use ariadne_cli::run_ui;
use serde_json::Value;
use std::fs;
use std::net::TcpListener;
use tempfile::tempdir;

mod common;
use common::fixture_path;

fn spawn_fixture_ui_server() -> (String, tokio::task::JoinHandle<()>) {
    let repo_dir = fixture_path("basic");

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
            None, // S6: keep the 127.0.0.1 bind default
        )
        .await;
        assert!(result.is_ok());
    });

    (format!("http://127.0.0.1:{}", port), server_task)
}

async fn wait_for_ready_status(base_url: &str) {
    // Poll-first idiom (mirrors m21_run_ui_smoke poll_until_ready): probe
    // immediately, only sleep BETWEEN attempts. Readiness itself stays
    // characterised by the assertions on the graph routes that follow.
    for _ in 0..120 {
        let mut ready_now = false;
        if let Ok(resp) = reqwest::get(format!("{}/api/status", base_url)).await {
            if resp.status().is_success() {
                let status: Value = resp.json().await.unwrap_or(Value::Null);
                ready_now = status["progress"]["state"].as_str() == Some("ready");
            }
        }
        if ready_now {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    }
}

async fn get_json_ok(url: String, context: &str) -> Value {
    let resp = reqwest::get(url).await.expect(context);
    assert!(resp.status().is_success());
    resp.json().await.unwrap()
}

fn node_id_at_ordinal(include_json: &Value, ordinal: usize) -> &str {
    include_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|el| el.get("data").unwrap().get("source").is_none()) // find nodes, not edges
        .nth(ordinal)
        .unwrap()
        .get("data")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap()
}

async fn fetch_traverse(base_url: &str, center: &str, depth: u32) -> Value {
    get_json_ok(
        format!(
            "{}/api/graph/traverse?graph_type=include&center={}&depth={}&direction=both",
            base_url, center, depth
        ),
        "Failed to request traverse endpoint",
    )
    .await
}

#[tokio::test]
async fn test_e2e_ui_traversal_and_pathfinding() {
    let (base_url, server_task) = spawn_fixture_ui_server();
    wait_for_ready_status(&base_url).await;

    // 1. Get a valid node ID first
    let include_json = get_json_ok(
        format!("{}/api/graph/include", base_url),
        "Failed to request include endpoint",
    )
    .await;
    let valid_node_id = node_id_at_ordinal(&include_json, 0);

    // 2. Verify /api/graph/traverse (1-hop)
    let traverse_json = fetch_traverse(&base_url, valid_node_id, 1).await;
    assert!(
        traverse_json.as_array().unwrap().len() > 0,
        "Traverse 1-hop should return nodes and edges"
    );

    // 2b. Verify /api/graph/traverse (2-hop)
    let traverse2_json = fetch_traverse(&base_url, valid_node_id, 2).await;
    assert!(
        traverse2_json.as_array().unwrap().len() >= traverse_json.as_array().unwrap().len(),
        "Traverse 2-hop should return at least as many nodes as 1-hop"
    );

    let valid_node_id_2 = node_id_at_ordinal(&include_json, 1);

    // 3. Verify /api/graph/path
    let path_json = get_json_ok(
        format!(
            "{}/api/graph/path?graph_type=include&from={}&to={}",
            base_url, valid_node_id, valid_node_id_2
        ),
        "Failed to request path endpoint",
    )
    .await;
    // Assuming path exists or doesn't, but the endpoint must return a valid JSON array or null
    assert!(path_json.is_array() || path_json.is_null());

    // 3. Verify /api/graph/entrypoints
    let ep_json = get_json_ok(
        format!(
            "{}/api/graph/entrypoints?graph_type=include&limit=5",
            base_url
        ),
        "Failed to request entrypoints endpoint",
    )
    .await;
    assert!(
        ep_json.get("roots").is_some(),
        "Entrypoints should have roots"
    );
    assert!(
        ep_json.get("top_hubs").is_some(),
        "Entrypoints should have hubs"
    );

    // 4. Verify /api/graph/search
    let search_json = get_json_ok(
        format!("{}/api/graph/search?q=main", base_url),
        "Failed to request search endpoint",
    )
    .await;
    let search_arr = search_json.as_array().unwrap();
    assert!(search_arr.len() > 0, "Search should return matching items");

    server_task.abort();
}

#[tokio::test]
async fn test_e2e_standalone_html_export_traversal() {
    let repo_dir = fixture_path("basic");
    let export_dir = tempdir().unwrap();
    let export_file = export_dir.path().join("export_traversal.html");

    let result = run_ui(
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

    assert!(export_file.exists(), "Export file should exist");
    let content = fs::read_to_string(&export_file).unwrap();
    assert!(content.len() > 1024, "Export file too small");
    assert!(
        content.contains("window.__ARIADNE_DATA__"),
        "Export missing data payload"
    );

    // Check for traversal engine functions
    assert!(
        content.contains("clientTraverseNeighborhood"),
        "Missing clientTraverseNeighborhood"
    );
    assert!(
        content.contains("clientShortestPath"),
        "Missing clientShortestPath"
    );
    assert!(
        content.contains("clientGetEntrypoints"),
        "Missing clientGetEntrypoints"
    );
    assert!(
        content.contains("ariadneThread"),
        "Missing ariadneThread entry point"
    );
}
