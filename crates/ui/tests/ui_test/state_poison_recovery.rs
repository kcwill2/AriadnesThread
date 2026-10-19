//! UTA2-R1 item 11: documented-behavior pins for the poison-recovery twins
//! `read_state` / `write_state` (state.rs). Their contract: a poisoned lock
//! only means some other thread panicked while holding it, the data is
//! memory-consistent, so routes KEEP SERVING instead of cascading panics.
//! These are pin-only (no fix): the arms ship deliberately, so there is no
//! red phase - each test poisons a lock through a panicking thread and
//! asserts the route/state path still answers. Journal: tasks/uta2-r1-journal.md.

use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph};
use ariadne_ui::build_router;
use axum::http::StatusCode;
use std::sync::Arc;

/// Run a thread that panics while holding the write lock, poisoning it. The
/// guard's unwind marks the RwLock poisoned before the thread dies.
fn poison_payload_lock(state: &Arc<ariadne_ui::UiState>) {
    let state = Arc::clone(state);
    let handle = std::thread::spawn(move || {
        let _guard = state.payload.write().unwrap();
        panic!("deliberate payload-lock poison (recovery pin fixture)");
    });
    assert!(
        handle.join().is_err(),
        "the poison thread must panic while holding the guard"
    );
}

fn poison_stage_status_lock(state: &Arc<ariadne_ui::UiState>) {
    let state = Arc::clone(state);
    let handle = std::thread::spawn(move || {
        let _guard = state.stage_status.write().unwrap();
        panic!("deliberate stage-status-lock poison (recovery pin fixture)");
    });
    assert!(
        handle.join().is_err(),
        "the poison thread must panic while holding the guard"
    );
}

fn poison_analysis_lock(state: &Arc<ariadne_ui::UiState>) {
    let state = Arc::clone(state);
    let handle = std::thread::spawn(move || {
        let _guard = state.analysis.write().unwrap();
        panic!("deliberate analysis-lock poison (recovery pin fixture)");
    });
    assert!(
        handle.join().is_err(),
        "the poison thread must panic while holding the guard"
    );
}

fn poisoned_fixture() -> Arc<ariadne_ui::UiState> {
    let (include_graph, payload) = common::fixture_include_graph();
    common::ready_state(
        std::path::PathBuf::from("."),
        include_graph,
        CallGraph::default(),
        payload,
    )
}

#[tokio::test]
async fn test_read_state_serves_after_payload_lock_poison() {
    // read_state arm pin: `unwrap_or_else(|e| e.into_inner())` - a poisoned
    // payload lock must not turn routes into 500s or panics.
    let state = poisoned_fixture();
    poison_payload_lock(&state);
    let app = build_router(state);
    let (status, _headers, body) = common::request_route(app, "/api/graph/include").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "read_state must recover the payload data from the poisoned lock"
    );
    assert!(
        !body.as_array().expect("element array").is_empty(),
        "the recovered payload still carries the fixture elements"
    );
}

#[tokio::test]
async fn test_status_route_serves_after_stage_status_lock_poison() {
    // read_state arm pin on the second lock family api_status touches.
    let state = poisoned_fixture();
    poison_stage_status_lock(&state);
    let app = build_router(state);
    let (status, _headers, body) = common::request_route(app, "/api/status").await;
    assert_eq!(status, StatusCode::OK, "status must survive the poison");
    assert_eq!(body["rebuilding"], false, "the consistent data still serves");
}

#[tokio::test]
async fn test_write_state_install_analysis_recovers_poisoned_analysis_lock() {
    // write_state arm pin: `install_analysis` (the ONLY generation swap)
    // must take the poisoned lock via into_inner, not panic, and the new
    // generation must serve afterwards.
    let state = poisoned_fixture();
    poison_analysis_lock(&state);
    state.install_analysis(
        IncludeGraph::new(),
        common::empty_reduced_model(),
        Arc::new(ariadne_graph::LoadMemo::new()),
    );
    assert!(
        state.analysis().is_some(),
        "write_state must publish through the poisoned lock"
    );
    let app = build_router(state);
    let (status, _headers, _body) = common::request_route(app, "/api/graph/include").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the generation swapped through the poisoned lock serves normally"
    );
}
