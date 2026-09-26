use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

/// Keeps the spawned `run_ui` server alive for the test body and guarantees the
/// task is aborted (its axum listener dropped) even if an assertion panics.
struct LiveServer {
    handle: tokio::task::JoinHandle<()>,
}

impl LiveServer {
    fn spawn(repo_root: PathBuf, port: u16) -> Self {
        let handle = tokio::spawn(async move {
            // Same entrypoint the `tool ui` binary calls: no_open=true, no export,
            // no_watch=true keeps the boot deterministic (no background rehash watcher).
            let result = ariadne_cli::run_ui(Some(repo_root), port, true, None, 2, true).await;
            assert!(result.is_ok(), "run_ui server errored: {:?}", result.err());
        });
        Self { handle }
    }
}

impl Drop for LiveServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Snapshot of the index + graph routes taken while the server reported "loading".
/// Best-effort: only captured if a loading snapshot is observed before ready, which
/// locks the "start web server immediately with live progress" behavior when the
/// build is slow enough to be sampled (it is near-instant on a warm artifact cache).
struct LiveProbe {
    index_status: u16,
    index_cache_control: Option<String>,
    index_content_type: Option<String>,
    graph_include_status: u16,
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn provision_basic_repo_copy() -> Result<(TempDir, PathBuf)> {
    let guard = tempdir()?;
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/basic");
    let repo_root = guard.path().join("basic");
    copy_dir_all(&fixture_root, &repo_root)?;
    Ok((guard, repo_root))
}

fn reserve_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
    let port = listener.local_addr().expect("read ephemeral port").port();
    drop(listener);
    port
}

async fn fetch_status(client: &Client, base_url: &str) -> Option<Value> {
    client
        .get(format!("{base_url}/api/status"))
        .send()
        .await
        .ok()?
        .json::<Value>()
        .await
        .ok()
}

async fn fetch_index_page(client: &Client, base_url: &str) -> Option<(u16, Option<String>, Option<String>, usize)> {
    let response = client.get(format!("{base_url}/")).send().await.ok()?;
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let body = response.text().await.ok()?;
    let header_value = |name: reqwest::header::HeaderName| -> Option<String> {
        headers.get(name).and_then(|v| v.to_str().ok()).map(str::to_string)
    };
    Some((
        status,
        header_value(reqwest::header::CACHE_CONTROL),
        header_value(reqwest::header::CONTENT_TYPE),
        body.len(),
    ))
}

async fn fetch_graph_include_status(client: &Client, base_url: &str) -> u16 {
    client
        .get(format!("{base_url}/api/graph/include"))
        .send()
        .await
        .map(|response| response.status().as_u16())
        .unwrap_or(0)
}

async fn probe_live_routes(client: &Client, base_url: &str) -> LiveProbe {
    let (index_status, index_cache_control, index_content_type, _body_len) =
        fetch_index_page(client, base_url).await.unwrap_or((0, None, None, 0));
    let graph_include_status = fetch_graph_include_status(client, base_url).await;
    LiveProbe {
        index_status,
        index_cache_control,
        index_content_type,
        graph_include_status,
    }
}

async fn poll_until_ready(
    client: &Client,
    base_url: &str,
    server: &LiveServer,
) -> Result<(Vec<Value>, Option<LiveProbe>)> {
    let mut trace = Vec::new();
    let mut live_probe = None;
    for _ in 0..400 {
        if let Some(snapshot) = fetch_status(client, base_url).await {
            let state = snapshot["progress"]["state"].as_str();
            if live_probe.is_none() && state == Some("loading") {
                live_probe = Some(probe_live_routes(client, base_url).await);
            }
            let ready = state == Some("ready");
            trace.push(snapshot);
            if ready {
                return Ok((trace, live_probe));
            }
        } else if server.handle.is_finished() {
            return Err(anyhow::anyhow!(
                "run_ui server task exited before reporting ready (see run_ui error above)"
            ));
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    Err(anyhow::anyhow!(
        "run_ui did not reach ready within the 40s polling window"
    ))
}

async fn wait_until_port_released(port: u16) -> bool {
    for _ in 0..40 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return true;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    false
}

#[tokio::test]
async fn run_ui_boots_live_serves_no_store_html_then_reports_ready() -> Result<()> {
    let (_repo_guard, repo_root) = provision_basic_repo_copy()?;
    let port = reserve_free_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let client = Client::new();
    let server = LiveServer::spawn(repo_root, port);

    // Collect every status snapshot until the pipeline finishes and reports ready.
    let (trace, live_probe) = poll_until_ready(&client, &base_url, &server).await?;

    // The status route always reports a valid shape while loading and once ready.
    for snapshot in &trace {
        let state = snapshot["progress"]["state"].as_str().unwrap_or_default();
        assert!(
            matches!(state, "loading" | "ready"),
            "status.progress.state must be loading or ready, got {state}"
        );
        assert!(snapshot["progress"]["percent"].is_number(), "progress.percent must be numeric");
        assert!(snapshot["stages"].is_array(), "status.stages must be an array");
    }

    // Best-effort live-during-load lock: while loading the index route already serves
    // HTML (Cache-Control: no-store) yet the graph route stays gated until ready.
    if let Some(probe) = &live_probe {
        assert_eq!(probe.index_status, 200, "index route must serve while loading");
        assert_eq!(
            probe.index_cache_control.as_deref(),
            Some("no-store"),
            "index route must send Cache-Control: no-store"
        );
        assert_eq!(
            probe.index_content_type.as_deref(),
            Some("text/html; charset=utf-8"),
            "index route should serve HTML while loading"
        );
        assert_eq!(probe.graph_include_status, 503, "graph route must stay gated until ready");
    }

    // Best-effort loading-window contract whenever a loading snapshot is observable.
    if let Some(loading) = trace.iter().find(|s| s["progress"]["state"].as_str() == Some("loading")) {
        let percent = loading["progress"]["percent"].as_u64().unwrap_or(100);
        assert!(percent < 100, "loading snapshot must report percent below 100, got {percent}");
        let phase = loading["progress"]["phase"].as_str().unwrap_or_default();
        assert!(
            phase == "Initializing" || (phase.starts_with('[') && phase.contains("/5]")),
            "loading phase must be the init label or a [N/5] stage label, got {phase}"
        );
    }

    // Lock the terminal ready state and the labels run_ui actually exposes.
    let ready = trace.last().expect("trace ends on a ready snapshot");
    assert_eq!(ready["progress"]["state"].as_str(), Some("ready"));
    assert_eq!(ready["progress"]["percent"].as_u64(), Some(100), "ready must report 100%");
    assert_eq!(ready["progress"]["phase"].as_str(), Some("Complete"), "ready phase label");
    assert_eq!(ready["progress"]["message"].as_str(), Some("Ready"), "ready message label");
    assert!(ready["last_refresh"].as_u64().unwrap_or(0) > 0, "ready status carries last_refresh");
    assert_eq!(ready["rebuilding"].as_bool(), Some(false), "ready status should not be rebuilding");
    assert_eq!(ready["repo_name"].as_str(), Some("basic"), "ready status carries the repo name");
    assert!(ready["stats"]["total_files"].as_u64().unwrap_or(0) > 0, "ready stats report files");

    // The index route serves HTML with Cache-Control: no-store (both while loading,
    // if sampled, and once ready).
    let (index_status, cache_control, content_type, body_len) = fetch_index_page(&client, &base_url)
        .await
        .context("index route reachable after ready")?;
    assert_eq!(index_status, 200, "index route should serve 200 after ready");
    assert_eq!(cache_control.as_deref(), Some("no-store"), "index route must send Cache-Control: no-store");
    assert_eq!(content_type.as_deref(), Some("text/html; charset=utf-8"));
    assert!(body_len > 0, "index HTML should be non-empty");

    // The graph API becomes reachable once the payload is swapped in at ready.
    assert_eq!(fetch_graph_include_status(&client, &base_url).await, 200, "include graph reachable once ready");

    // Clean shutdown: abort drops the axum listener and the port becomes reusable.
    server.handle.abort();
    drop(server);
    assert!(wait_until_port_released(port).await, "server port should be released after shutdown");
    Ok(())
}
