//! Route contract locks for the wizard backend (ui-extraction-wizard W-2b):
//! the two GET routes and the uniform error shapes of shape C3
//! (503 analysis_loading / 409 stale_generation / 404 unknown_plan), plus
//! the W1 GET-only and no-write string locks.
//!
//! Model: graph_api.rs route tests. Bodies are pulled directly here (not via
//! common::request_route) because the error bodies matter at non-200
//! statuses, where the shared helper parses nothing.
//!
//! Path-parameter note: the contract documents the routes as
//! `/api/extraction/{plan_id}/manifest` and `/api/extraction/{plan_id}/dry-run`;
//! axum 0.7 registers path parameters with `:name` syntax, so the route
//! table (and these locks) use `:plan_id`. The requested URLs are identical.

use crate::common;
use ariadne_graph::IncludeGraph;
use ariadne_ui::{
    build_router, ExtractionDryRunDto, ExtractionManifestDto, ExtractionManifestReader,
    ExtractionRewriteDto, ExtractionUnitManifestDto, UiState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

const MANIFEST_ROUTE_PATH: &str = "/api/extraction/:plan_id/manifest";
const DRY_RUN_ROUTE_PATH: &str = "/api/extraction/:plan_id/dry-run";

/// `UiState::new_loaded` performs exactly one `install_analysis`
/// (state.rs:143), so a live state built that way sits at epoch 1. Readers
/// in the 200/404 tests are built for this epoch; the stale tests follow the
/// handoff W6 injection recipe instead (epoch 0 after one install).
const LIVE_EPOCH_AFTER_NEW_LOADED: u64 = 1;

const KNOWN_PLAN_ID: &str = "PLAN-002";
const UNKNOWN_PLAN_ID: &str = "PLAN-999";

/// A fixture reader answering exactly one known plan, with a fixed epoch.
/// The cli provider of W-2a will implement this trait for real; here it is
/// the scripted stand-in the route contract is checked against.
struct ScriptedReader {
    epoch: u64,
    known_plan: String,
    manifest: ExtractionManifestDto,
    dry_run: ExtractionDryRunDto,
}

impl ScriptedReader {
    fn serving_sample_with_epoch(epoch: u64) -> Self {
        Self {
            epoch,
            known_plan: KNOWN_PLAN_ID.to_string(),
            manifest: sample_manifest_dto(),
            dry_run: sample_dry_run_dto(),
        }
    }
}

impl ExtractionManifestReader for ScriptedReader {
    fn epoch(&self) -> u64 {
        self.epoch
    }

    fn manifest(&self, plan_id: &str) -> Option<ExtractionManifestDto> {
        if plan_id == self.known_plan {
            Some(self.manifest.clone())
        } else {
            None
        }
    }

    fn dry_run(&self, plan_id: &str) -> Option<ExtractionDryRunDto> {
        if plan_id == self.known_plan {
            Some(self.dry_run.clone())
        } else {
            None
        }
    }
}

fn sample_manifest_dto() -> ExtractionManifestDto {
    ExtractionManifestDto {
        plan_id: KNOWN_PLAN_ID.to_string(),
        generation_epoch: LIVE_EPOCH_AFTER_NEW_LOADED,
        files: vec![ExtractionUnitManifestDto {
            path: "include/alpha_extracted.h".to_string(),
            symbol_names: vec!["Alpha".to_string(), "Beta".to_string()],
            symbol_count: 2,
            required_includes: vec!["core/types.h".to_string()],
            includes_unavailable: false,
        }],
        files_truncated: false,
        cycle_error: None,
    }
}

fn sample_dry_run_dto() -> ExtractionDryRunDto {
    ExtractionDryRunDto {
        plan_id: KNOWN_PLAN_ID.to_string(),
        generation_epoch: LIVE_EPOCH_AFTER_NEW_LOADED,
        new_files: vec!["include/alpha_extracted.h".to_string()],
        rewrites: vec![ExtractionRewriteDto {
            consumer_file: "src/main.cc".to_string(),
            old_includes: vec!["alpha.h".to_string()],
            new_includes: vec!["include/alpha_extracted.h".to_string()],
            side_effect_preservations: vec![],
        }],
        rewrites_truncated: false,
        move_removals: 0,
        copy_only: true,
        includes_unavailable: false,
    }
}

/// Ready state with empty graphs: enough for the reader seam, which never
/// touches the analysis itself.
fn wizard_live_state() -> Arc<UiState> {
    common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        common::empty_export_payload(),
    )
}

/// Loading state plus one `install_analysis` (live epoch 1) plus a reader
/// built for epoch 0 - the handoff's W6 stale-injection recipe.
fn wizard_stale_reader_state() -> Arc<UiState> {
    let state = UiState::new_loading(std::path::PathBuf::from("."));
    state.install_analysis(
        IncludeGraph::new(),
        common::empty_reduced_model(),
        Arc::new(ariadne_graph::LoadMemo::new()),
    );
    state.install_extraction_reader(Arc::new(ScriptedReader::serving_sample_with_epoch(0)));
    Arc::new(state)
}

fn assert_ascii_body(raw: &[u8], what: &str) {
    assert!(raw.is_ascii(), "{what} must be pure ASCII, got bytes: {raw:?}");
}

async fn request_json_body(app: axum::Router, uri: &str) -> (StatusCode, serde_json::Value, Vec<u8>) {
    let request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let raw = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    let body = serde_json::from_slice(&raw).unwrap_or(serde_json::Value::Null);
    (status, body, raw)
}

// ---- manifest route: the four contract shapes (required tests 1-4) ----

#[tokio::test]
async fn test_manifest_route_returns_503_analysis_loading_while_no_reader() {
    let app = build_router(common::loading_state());
    let (status, body, raw) =
        request_json_body(app, &format!("/api/extraction/{KNOWN_PLAN_ID}/manifest")).await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "expected 503 while no reader is installed"
    );
    assert_eq!(body, serde_json::json!({ "error": "analysis_loading" }));
    assert_ascii_body(&raw, "503 loading body");
}

#[tokio::test]
async fn test_manifest_route_returns_200_with_reader_manifest() {
    let state = wizard_live_state();
    state.install_extraction_reader(Arc::new(ScriptedReader::serving_sample_with_epoch(
        LIVE_EPOCH_AFTER_NEW_LOADED,
    )));
    let app = build_router(state);
    let (status, body, _raw) =
        request_json_body(app, &format!("/api/extraction/{KNOWN_PLAN_ID}/manifest")).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "expected status 200 for a live-epoch reader's known plan"
    );
    let expected = serde_json::to_value(sample_manifest_dto()).expect("DTO is serde-plain");
    assert_eq!(body, expected, "200 body must be the reader's manifest DTO");
}

#[tokio::test]
async fn test_manifest_route_returns_404_unknown_plan() {
    let state = wizard_live_state();
    state.install_extraction_reader(Arc::new(ScriptedReader::serving_sample_with_epoch(
        LIVE_EPOCH_AFTER_NEW_LOADED,
    )));
    let app = build_router(state);
    let (status, body, raw) =
        request_json_body(app, &format!("/api/extraction/{UNKNOWN_PLAN_ID}/manifest")).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "expected 404 for a plan the reader does not know"
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "unknown_plan", "plan_id": UNKNOWN_PLAN_ID }),
        "404 body must echo the requested plan id"
    );
    assert_ascii_body(&raw, "404 unknown_plan body");
}

#[tokio::test]
async fn test_manifest_route_returns_409_stale_generation() {
    let app = build_router(wizard_stale_reader_state());
    let (status, body, raw) =
        request_json_body(app, &format!("/api/extraction/{KNOWN_PLAN_ID}/manifest")).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "expected 409 for a reader whose epoch differs from the live epoch"
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "stale_generation", "epoch": 1 }),
        "409 body must carry the live epoch"
    );
    let body_text = String::from_utf8_lossy(&raw).to_string();
    assert!(
        body_text.contains("\"epoch\":1"),
        "the 409 body must carry \"epoch\":1, got: {body_text}"
    );
    assert_ascii_body(&raw, "409 stale_generation body");
}

// ---- dry-run route: the same four shapes (required test 5) ----

#[tokio::test]
async fn test_dry_run_route_returns_503_analysis_loading_while_no_reader() {
    let app = build_router(common::loading_state());
    let (status, body, raw) =
        request_json_body(app, &format!("/api/extraction/{KNOWN_PLAN_ID}/dry-run")).await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "expected 503 while no reader is installed"
    );
    assert_eq!(body, serde_json::json!({ "error": "analysis_loading" }));
    assert_ascii_body(&raw, "dry-run 503 loading body");
}

#[tokio::test]
async fn test_dry_run_route_returns_200_with_reader_dry_run() {
    let state = wizard_live_state();
    state.install_extraction_reader(Arc::new(ScriptedReader::serving_sample_with_epoch(
        LIVE_EPOCH_AFTER_NEW_LOADED,
    )));
    let app = build_router(state);
    let (status, body, _raw) =
        request_json_body(app, &format!("/api/extraction/{KNOWN_PLAN_ID}/dry-run")).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "expected status 200 for a live-epoch reader's known plan"
    );
    let expected = serde_json::to_value(sample_dry_run_dto()).expect("DTO is serde-plain");
    assert_eq!(body, expected, "200 body must be the reader's dry-run DTO");
}

#[tokio::test]
async fn test_dry_run_route_returns_404_unknown_plan() {
    let state = wizard_live_state();
    state.install_extraction_reader(Arc::new(ScriptedReader::serving_sample_with_epoch(
        LIVE_EPOCH_AFTER_NEW_LOADED,
    )));
    let app = build_router(state);
    let (status, body, raw) =
        request_json_body(app, &format!("/api/extraction/{UNKNOWN_PLAN_ID}/dry-run")).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "expected 404 for a plan the reader does not know"
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "unknown_plan", "plan_id": UNKNOWN_PLAN_ID }),
        "404 body must echo the requested plan id"
    );
    assert_ascii_body(&raw, "dry-run 404 unknown_plan body");
}

#[tokio::test]
async fn test_dry_run_route_returns_409_stale_generation() {
    let app = build_router(wizard_stale_reader_state());
    let (status, body, raw) =
        request_json_body(app, &format!("/api/extraction/{KNOWN_PLAN_ID}/dry-run")).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "expected 409 for a reader whose epoch differs from the live epoch"
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "stale_generation", "epoch": 1 }),
        "409 body must carry the live epoch"
    );
    let body_text = String::from_utf8_lossy(&raw).to_string();
    assert!(
        body_text.contains("\"epoch\":1"),
        "the 409 body must carry \"epoch\":1, got: {body_text}"
    );
    assert_ascii_body(&raw, "dry-run 409 stale_generation body");
}

// ---- W1 string locks (required tests 6-8) ----

fn registering_lines(source: &str, path: &str) -> Vec<String> {
    source
        .lines()
        .filter(|line| line.contains(path))
        .map(|line| line.trim().to_string())
        .collect()
}

#[test]
fn test_server_registers_extraction_routes_with_get_only() {
    let server_source = include_str!("../../src/server.rs");
    for path in [MANIFEST_ROUTE_PATH, DRY_RUN_ROUTE_PATH] {
        let lines = registering_lines(server_source, path);
        assert!(
            !lines.is_empty(),
            "server.rs must register the {path} route (shape C2)"
        );
        for line in lines {
            assert!(line.contains("get("), "route line `{line}` must use get(...)");
            for verb in ["post(", "put(", "delete(", "patch("] {
                assert!(
                    !line.contains(verb),
                    "route line `{line}` must not register {verb} (W1)"
                );
            }
        }
    }
}

#[test]
fn test_extraction_route_module_has_no_write_calls() {
    let route_source = include_str!("../../src/routes/api_extraction.rs");
    for forbidden in [
        "fs::write",
        "fs::remove_file",
        "fs::create_dir",
        "ArtifactManager",
    ] {
        assert!(
            !route_source.contains(forbidden),
            "routes/api_extraction.rs must not contain {forbidden} (W1)"
        );
    }
}

#[test]
fn test_extraction_route_module_has_no_mutation_route_registrations() {
    let route_source = include_str!("../../src/routes/api_extraction.rs");
    for verb in ["post(", "put(", "delete(", "patch("] {
        assert!(
            !route_source.contains(verb),
            "routes/api_extraction.rs must not register {verb} (W1)"
        );
    }
}
