//! Route contract locks for the apply backend (extraction-auto-apply A-3):
//! the POST `/api/extraction/:plan_id/apply` refusal table of shape C3
//! (503 analysis_loading / 409 stale_generation / 404 unknown_plan /
//! 428 dry_run_required / 409 apply_in_progress / 500 apply_failed / 200
//! outcome DTO), the dry-run route's approval-token mint (shape C4), the
//! A1 no-write string locks over the new module and the server
//! registration, the surviving W1 locks over the GET module, and the A9
//! pure-ASCII assertion over every body this route family answers.
//!
//! Model: extraction_wizard_api.rs route tests. Bodies are pulled
//! directly here (not via common::request_route) because the error bodies
//! matter at non-200 statuses, where the shared helper parses nothing.
//! The helpers are duplicated from the wizard file on purpose: its copies
//! are private to that module, and the wizard file is frozen apart from
//! its single C8 amendment line.
//!
//! Path-parameter note: the contract documents the apply route as
//! `/api/extraction/{plan_id}/apply`; axum 0.7 registers path parameters
//! with `:name` syntax, so the route table (and these locks) use
//! `:plan_id`. The requested URLs are identical.

use crate::common;
use ariadne_graph::IncludeGraph;
use ariadne_ui::{
    build_router, ExtractionApplyOutcomeDto, ExtractionApplyRefusal, ExtractionDryRunDto,
    ExtractionManifestDto, ExtractionManifestReader, ExtractionPlanApplier, ExtractionTierResultDto,
    UiState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tower::ServiceExt;

const APPLY_ROUTE_PATH: &str = "/api/extraction/:plan_id/apply";
const MANIFEST_ROUTE_PATH: &str = "/api/extraction/:plan_id/manifest";
const DRY_RUN_ROUTE_PATH: &str = "/api/extraction/:plan_id/dry-run";

/// `UiState::new_loaded` performs exactly one `install_analysis`, so a
/// live state built that way sits at epoch 1 (the wizard suite's finding).
/// Appliers in the 200/404/428 tests are built for this epoch; the stale
/// test follows the W6 injection recipe instead (epoch 0 after one
/// install).
const LIVE_EPOCH_AFTER_NEW_LOADED: u64 = 1;

const KNOWN_PLAN_ID: &str = "PLAN-002";
const UNKNOWN_PLAN_ID: &str = "PLAN-999";
const FOREIGN_PLAN_ID: &str = "PLAN-777";

/// The scripted token the scripted applier accepts and mints. Its shape
/// mirrors shape C4 (`apply-{epoch}-{plan_id}-{counter}`); the counter is
/// the applier's implementation detail - the route contract here is the
/// field's presence versus emptiness and the refusal shapes.
const SCRIPTED_TOKEN: &str = "apply-1-PLAN-002-7";

enum ScriptedAnswer {
    Approved,
    InProgress,
    TransactionFailed(String),
}

/// A scripted applier answering exactly one known plan, with a fixed
/// epoch. The cli provider (A-4) implements this trait for real; here it
/// is the scripted stand-in the route contract is checked against.
/// `apply_calls` counts trait consultations, so the foreign-token test can
/// prove the route never consults the applier twice for one request.
struct ScriptedApplier {
    epoch: u64,
    known_plan: String,
    minted_token: Option<String>,
    accepted_token: String,
    outcome: ExtractionApplyOutcomeDto,
    answer: ScriptedAnswer,
    apply_calls: AtomicUsize,
}

impl ScriptedApplier {
    fn live() -> Self {
        Self::with_epoch(LIVE_EPOCH_AFTER_NEW_LOADED)
    }

    fn with_epoch(epoch: u64) -> Self {
        Self {
            epoch,
            known_plan: KNOWN_PLAN_ID.to_string(),
            minted_token: Some(SCRIPTED_TOKEN.to_string()),
            accepted_token: SCRIPTED_TOKEN.to_string(),
            outcome: sample_outcome_dto(),
            answer: ScriptedAnswer::Approved,
            apply_calls: AtomicUsize::new(0),
        }
    }

    fn answering_in_progress() -> Self {
        Self {
            answer: ScriptedAnswer::InProgress,
            ..Self::live()
        }
    }

    fn answering_transaction_failure(detail: &str) -> Self {
        Self {
            answer: ScriptedAnswer::TransactionFailed(detail.to_string()),
            ..Self::live()
        }
    }

    fn apply_call_count(&self) -> usize {
        self.apply_calls.load(Ordering::SeqCst)
    }
}

impl ExtractionPlanApplier for ScriptedApplier {
    fn epoch(&self) -> u64 {
        self.epoch
    }

    fn mint_apply_token(&self, plan_id: &str) -> Option<String> {
        if plan_id == self.known_plan {
            self.minted_token.clone()
        } else {
            None
        }
    }

    fn apply(
        &self,
        plan_id: &str,
        token: &str,
    ) -> Result<ExtractionApplyOutcomeDto, ExtractionApplyRefusal> {
        self.apply_calls.fetch_add(1, Ordering::SeqCst);
        if plan_id != self.known_plan {
            return Err(ExtractionApplyRefusal::UnknownPlan);
        }
        if token != self.accepted_token {
            return Err(ExtractionApplyRefusal::NotApproved);
        }
        match &self.answer {
            ScriptedAnswer::Approved => Ok(self.outcome.clone()),
            ScriptedAnswer::InProgress => Err(ExtractionApplyRefusal::InProgress),
            ScriptedAnswer::TransactionFailed(detail) => {
                Err(ExtractionApplyRefusal::Failed(detail.clone()))
            }
        }
    }
}

/// A reader serving the same known plan, so the dry-run 200 exists to
/// carry the mint (the applier alone does not answer the GET).
struct ScriptedDryRunReader {
    epoch: u64,
    known_plan: String,
}

impl ScriptedDryRunReader {
    fn live() -> Self {
        Self {
            epoch: LIVE_EPOCH_AFTER_NEW_LOADED,
            known_plan: KNOWN_PLAN_ID.to_string(),
        }
    }
}

impl ExtractionManifestReader for ScriptedDryRunReader {
    fn epoch(&self) -> u64 {
        self.epoch
    }

    fn manifest(&self, _plan_id: &str) -> Option<ExtractionManifestDto> {
        None
    }

    fn dry_run(&self, plan_id: &str) -> Option<ExtractionDryRunDto> {
        if plan_id == self.known_plan {
            Some(sample_dry_run_dto())
        } else {
            None
        }
    }
}

fn sample_outcome_dto() -> ExtractionApplyOutcomeDto {
    ExtractionApplyOutcomeDto {
        plan_id: KNOWN_PLAN_ID.to_string(),
        generation_epoch: LIVE_EPOCH_AFTER_NEW_LOADED,
        committed: true,
        tiers: vec![
            ExtractionTierResultDto {
                tier: "Structural".to_string(),
                status: "pass".to_string(),
                detail: String::new(),
            },
            ExtractionTierResultDto {
                tier: "GeneratedHeaders".to_string(),
                status: "pass".to_string(),
                detail: String::new(),
            },
            ExtractionTierResultDto {
                tier: "AffectedTUs".to_string(),
                status: "skipped".to_string(),
                detail: "clang toolchain absent".to_string(),
            },
        ],
        new_files: vec!["include/alpha_extracted.h".to_string()],
        output_text: "Committed: 1 file(s) written, tiers pass/pass/skip".to_string(),
    }
}

fn sample_dry_run_dto() -> ExtractionDryRunDto {
    ExtractionDryRunDto {
        plan_id: KNOWN_PLAN_ID.to_string(),
        generation_epoch: LIVE_EPOCH_AFTER_NEW_LOADED,
        new_files: vec!["include/alpha_extracted.h".to_string()],
        rewrites: vec![],
        rewrites_truncated: false,
        move_removals: 0,
        copy_only: true,
        includes_unavailable: false,
        apply_token: String::new(),
    }
}

/// Ready state with empty graphs: enough for the applier seam, which the
/// scripted stand-in answers directly and never exercises for real.
fn apply_live_state() -> Arc<UiState> {
    common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        common::empty_export_payload(),
    )
}

/// Loading state plus one `install_analysis` (live epoch 1) plus an
/// applier built for epoch 0 - the wizard W6 stale-injection recipe
/// (extraction_wizard_api.rs:130-139), applied to the applier seam.
fn apply_stale_applier_state() -> Arc<UiState> {
    let state = UiState::new_loading(std::path::PathBuf::from("."));
    state.install_analysis(
        IncludeGraph::new(),
        common::empty_reduced_model(),
        Arc::new(ariadne_graph::LoadMemo::new()),
    );
    state.install_extraction_applier(Arc::new(ScriptedApplier::with_epoch(0)));
    Arc::new(state)
}

fn assert_ascii_body(raw: &[u8], what: &str) {
    assert!(raw.is_ascii(), "{what} must be pure ASCII, got bytes: {raw:?}");
}

async fn route_json(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Body,
) -> (StatusCode, serde_json::Value, Vec<u8>) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .body(body)
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let raw = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    let parsed = serde_json::from_slice(&raw).unwrap_or(serde_json::Value::Null);
    (status, parsed, raw)
}

fn apply_uri(plan_id: &str) -> String {
    format!("/api/extraction/{plan_id}/apply")
}

async fn post_apply(
    app: axum::Router,
    plan_id: &str,
    body: Body,
) -> (StatusCode, serde_json::Value, Vec<u8>) {
    route_json(app, "POST", &apply_uri(plan_id), body).await
}

fn token_body(token: &str) -> Body {
    Body::from(format!("{{\"apply_token\":\"{token}\"}}"))
}

// ---- the refusal table (required tests 1-11) ----

#[tokio::test]
async fn test_apply_route_returns_503_analysis_loading_with_no_applier() {
    let app = build_router(common::loading_state());
    // A well-formed token body must still answer 503: the no-applier check
    // precedes the body parse (W2b check order).
    let (status, body, raw) = post_apply(app, KNOWN_PLAN_ID, token_body(SCRIPTED_TOKEN)).await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "expected 503 while no applier is installed, got {}",
        status.as_u16()
    );
    assert_eq!(body, serde_json::json!({ "error": "analysis_loading" }));
    assert_ascii_body(&raw, "503 loading body");
}

#[tokio::test]
async fn test_apply_route_returns_428_when_the_body_is_absent() {
    let state = apply_live_state();
    state.install_extraction_applier(Arc::new(ScriptedApplier::live()));
    let app = build_router(state);
    let (status, body, raw) = post_apply(app, KNOWN_PLAN_ID, Body::empty()).await;
    assert_eq!(
        status,
        StatusCode::PRECONDITION_REQUIRED,
        "expected 428 for an absent body, got {}",
        status.as_u16()
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "dry_run_required", "plan_id": KNOWN_PLAN_ID }),
        "428 body must name the plan and the dry-run gate"
    );
    assert_ascii_body(&raw, "428 absent-body refusal");
}

#[tokio::test]
async fn test_apply_route_returns_428_when_the_body_is_unparseable() {
    let state = apply_live_state();
    state.install_extraction_applier(Arc::new(ScriptedApplier::live()));
    let app = build_router(state);
    let (status, body, raw) = post_apply(app, KNOWN_PLAN_ID, Body::from("this is not json {{")).await;
    assert_eq!(
        status,
        StatusCode::PRECONDITION_REQUIRED,
        "expected 428 for an unparseable body (tolerant parse, never an \
         extractor 4xx), got {}",
        status.as_u16()
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "dry_run_required", "plan_id": KNOWN_PLAN_ID }),
        "428 body must name the plan and the dry-run gate"
    );
    assert_ascii_body(&raw, "428 unparseable-body refusal");
}

#[tokio::test]
async fn test_apply_route_returns_428_when_the_token_is_empty() {
    let state = apply_live_state();
    state.install_extraction_applier(Arc::new(ScriptedApplier::live()));
    let app = build_router(state);
    let (status, body, raw) = post_apply(app, KNOWN_PLAN_ID, token_body("")).await;
    assert_eq!(
        status,
        StatusCode::PRECONDITION_REQUIRED,
        "expected 428 for an empty apply_token, got {}",
        status.as_u16()
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "dry_run_required", "plan_id": KNOWN_PLAN_ID }),
        "428 body must name the plan and the dry-run gate"
    );
    assert_ascii_body(&raw, "428 empty-token refusal");
}

#[tokio::test]
async fn test_apply_route_returns_428_for_an_unknown_token() {
    let state = apply_live_state();
    state.install_extraction_applier(Arc::new(ScriptedApplier::live()));
    let app = build_router(state);
    let (status, body, raw) =
        post_apply(app, KNOWN_PLAN_ID, token_body("apply-1-PLAN-002-99")).await;
    assert_eq!(
        status,
        StatusCode::PRECONDITION_REQUIRED,
        "expected 428 when the applier answers NotApproved, got {}",
        status.as_u16()
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "dry_run_required", "plan_id": KNOWN_PLAN_ID }),
        "428 body must name the plan and the dry-run gate"
    );
    assert_ascii_body(&raw, "428 unknown-token refusal");
}

#[tokio::test]
async fn test_apply_route_returns_428_for_a_foreign_plan_token() {
    let state = apply_live_state();
    let applier = Arc::new(ScriptedApplier::live());
    let applier_handle: Arc<dyn ExtractionPlanApplier> = applier.clone();
    state.install_extraction_applier(applier_handle);
    let app = build_router(state);
    // A token minted for a different plan id: the scripted applier answers
    // NotApproved; the route must not consult the applier twice for one
    // request (no pre-validation side channel).
    let foreign_token = format!("apply-1-{FOREIGN_PLAN_ID}-3");
    let (status, body, raw) = post_apply(app, KNOWN_PLAN_ID, token_body(&foreign_token)).await;
    assert_eq!(
        status,
        StatusCode::PRECONDITION_REQUIRED,
        "expected 428 for a token minted for another plan, got {}",
        status.as_u16()
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "dry_run_required", "plan_id": KNOWN_PLAN_ID }),
        "428 body must name the plan and the dry-run gate"
    );
    assert!(
        applier.apply_call_count() <= 1,
        "the route must not consult the applier twice for one request, got {} calls",
        applier.apply_call_count()
    );
    assert_ascii_body(&raw, "428 foreign-plan-token refusal");
}

#[tokio::test]
async fn test_apply_route_returns_200_with_a_valid_token() {
    let state = apply_live_state();
    state.install_extraction_applier(Arc::new(ScriptedApplier::live()));
    let app = build_router(state);
    let (status, body, raw) = post_apply(app, KNOWN_PLAN_ID, token_body(SCRIPTED_TOKEN)).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "expected status 200 for an approved apply, got {}",
        status.as_u16()
    );
    let expected = serde_json::to_value(sample_outcome_dto()).expect("DTO is serde-plain");
    assert_eq!(
        body, expected,
        "200 body must be the applier's outcome DTO: three tier rows, \
         committed, new_files, generation_epoch and the verbatim output_text"
    );
    assert_ascii_body(&raw, "200 outcome body");
}

#[tokio::test]
async fn test_apply_route_returns_404_for_a_plan_the_applier_does_not_know() {
    let state = apply_live_state();
    state.install_extraction_applier(Arc::new(ScriptedApplier::live()));
    let app = build_router(state);
    let (status, body, raw) = post_apply(app, UNKNOWN_PLAN_ID, token_body(SCRIPTED_TOKEN)).await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "expected 404 for a plan the applier does not know, got {}",
        status.as_u16()
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "unknown_plan", "plan_id": UNKNOWN_PLAN_ID }),
        "404 body must echo the requested plan id"
    );
    assert_ascii_body(&raw, "404 unknown_plan body");
}

#[tokio::test]
async fn test_apply_route_returns_409_stale_generation_for_a_stale_applier() {
    let app = build_router(apply_stale_applier_state());
    let (status, body, raw) = post_apply(app, KNOWN_PLAN_ID, token_body(SCRIPTED_TOKEN)).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "expected 409 for an applier whose epoch differs from the live epoch, got {}",
        status.as_u16()
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "stale_generation", "epoch": 1 }),
        "409 body must carry the live epoch (shape A3)"
    );
    let body_text = String::from_utf8_lossy(&raw).to_string();
    assert!(
        body_text.contains("\"epoch\":1"),
        "the 409 body must carry \"epoch\":1, got: {body_text}"
    );
    assert_ascii_body(&raw, "409 stale_generation body");
}

#[tokio::test]
async fn test_apply_route_returns_409_apply_in_progress_while_the_lock_is_held() {
    let state = apply_live_state();
    state.install_extraction_applier(Arc::new(ScriptedApplier::answering_in_progress()));
    let app = build_router(state);
    let (status, body, raw) = post_apply(app, KNOWN_PLAN_ID, token_body(SCRIPTED_TOKEN)).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "expected 409 apply_in_progress, got {}",
        status.as_u16()
    );
    assert_eq!(
        body,
        serde_json::json!({ "error": "apply_in_progress", "plan_id": KNOWN_PLAN_ID }),
        "409 apply_in_progress body must echo the plan id"
    );
    assert_ascii_body(&raw, "409 apply_in_progress body");
}

#[tokio::test]
async fn test_apply_route_returns_500_apply_failed_on_a_transaction_error() {
    let state = apply_live_state();
    state.install_extraction_applier(Arc::new(
        ScriptedApplier::answering_transaction_failure("rollback restored the tree"),
    ));
    let app = build_router(state);
    let (status, body, raw) = post_apply(app, KNOWN_PLAN_ID, token_body(SCRIPTED_TOKEN)).await;
    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "expected 500 for a transaction error, got {}",
        status.as_u16()
    );
    assert_eq!(
        body,
        serde_json::json!({
            "error": "apply_failed",
            "plan_id": KNOWN_PLAN_ID,
            "detail": "rollback restored the tree",
        }),
        "500 body must echo the plan id and carry the ASCII detail"
    );
    assert_ascii_body(&raw, "500 apply_failed body");
}

// ---- the dry-run mint (required test 12) ----

#[tokio::test]
async fn test_dry_run_route_mints_an_apply_token_with_a_live_applier() {
    let state = apply_live_state();
    state.install_extraction_reader(Arc::new(ScriptedDryRunReader::live()));
    state.install_extraction_applier(Arc::new(ScriptedApplier::live()));
    let app = build_router(state);
    let (status, body, raw) = route_json(
        app,
        "GET",
        &format!("/api/extraction/{KNOWN_PLAN_ID}/dry-run"),
        Body::empty(),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "expected 200 for the reader's dry-run answer, got {}",
        status.as_u16()
    );
    assert_eq!(
        body.get("apply_token").and_then(serde_json::Value::as_str),
        Some(SCRIPTED_TOKEN),
        "the dry-run 200 body must carry the minted apply_token"
    );
    assert_ascii_body(&raw, "dry-run 200 body carrying the mint");
}

// ---- A1 / W1 / C3 string locks (required tests 13-15) ----

fn registering_lines(source: &str, path: &str) -> Vec<String> {
    source
        .lines()
        .filter(|line| line.contains(path))
        .map(|line| line.trim().to_string())
        .collect()
}

#[test]
fn test_apply_route_module_has_no_write_or_registration_tokens() {
    let apply_source = include_str!("../../src/routes/api_extraction_apply.rs");
    for forbidden in [
        "fs::write",
        "fs::remove_file",
        "fs::create_dir",
        "ArtifactManager",
        "post(",
        "put(",
        "delete(",
        "patch(",
    ] {
        assert!(
            !apply_source.contains(forbidden),
            "routes/api_extraction_apply.rs must not contain {forbidden} \
             (A1 single write path; the registration lives in server.rs)"
        );
    }
}

#[test]
fn test_server_registers_the_apply_route_exactly_once_with_post() {
    let server_source = include_str!("../../src/server.rs");
    let apply_lines = registering_lines(server_source, APPLY_ROUTE_PATH);
    assert_eq!(
        apply_lines.len(),
        1,
        "server.rs must register {APPLY_ROUTE_PATH} exactly once (shape C3), got {apply_lines:?}"
    );
    for line in &apply_lines {
        assert!(
            line.contains("post("),
            "the apply route line `{line}` must register the handler with post(...)"
        );
    }
    for path in [MANIFEST_ROUTE_PATH, DRY_RUN_ROUTE_PATH] {
        for line in registering_lines(server_source, path) {
            for verb in ["post(", "put(", "delete(", "patch("] {
                assert!(
                    !line.contains(verb),
                    "route line `{line}` must not register {verb} \
                     (the W1 GET-only lock survives A-3)"
                );
            }
        }
    }
}

#[test]
fn test_wizard_get_module_still_has_no_write_or_mutation_tokens() {
    let get_source = include_str!("../../src/routes/api_extraction.rs");
    for forbidden in [
        "fs::write",
        "fs::remove_file",
        "fs::create_dir",
        "ArtifactManager",
        "post(",
        "put(",
        "delete(",
        "patch(",
    ] {
        assert!(
            !get_source.contains(forbidden),
            "routes/api_extraction.rs must not contain {forbidden} \
             (W1 survival across the A-3 mint)"
        );
    }
}
