//! Apply route family (extraction-auto-apply A-3): the mutation seam trait
//! the host (cli) implements, the apply DTOs' route contract, and the POST
//! registration `/api/extraction/:plan_id/apply` behind the
//! dry-run-then-approve gate (axum 0.7 registers path parameters with
//! `:name` syntax; the contract documents the `{plan_id}` form, the
//! requested URLs are identical).
//!
//! The deliberate twin of the read-only `ExtractionManifestReader` seam
//! (`routes/api_extraction.rs`, ui-extraction-wizard W-2b): the host (cli)
//! implements this trait and every mutation runs through the shared
//! `apply_plan_with` transactional core (shape C1); the ui crate names no
//! write. Shape A1 (single write path): this module contains no filesystem
//! write token and registers no route - the verb registration lives in
//! `server.rs` alone - string locks in
//! `crates/ui/tests/ui_test/extraction_apply_api.rs` keep it that way.
//!
//! Error bodies follow the uniform ASCII shapes of shape C3, checked in
//! this order: no applier installed -> 503 `analysis_loading`; applier
//! epoch differs from the live `analysis_epoch` -> 409 `stale_generation`
//! with the live epoch; then the trait's answer, mapped per the C3 refusal
//! table. A missing, absent or unparseable request body is the 428
//! `dry_run_required` refusal, never an extractor 4xx (W2b uniform error
//! family; 428 Precondition Required is chosen so 409 keeps its single
//! stale-generation meaning across the whole route family).

use crate::dto::{ExtractionApplyOutcomeDto, ExtractionApplyRequestDto};
use crate::state::UiState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

/// The mutation seam, the deliberate twin of the read-only
/// `ExtractionManifestReader`: the host (cli) implements it; the ui crate
/// names no write. Ledger invariants encoded: every mutation goes through
/// the existing TransactionApplier path (the implementation delegates to
/// `apply_plan_with`), one apply at a time, token-gated, epoch-bound.
pub trait ExtractionPlanApplier: Send + Sync + 'static {
    fn epoch(&self) -> u64;
    /// Single-use approval token bound to (plan_id, epoch); None when the
    /// plan does not resolve in this generation.
    fn mint_apply_token(&self, plan_id: &str) -> Option<String>;
    /// The gated apply. Every refusal is a value, never a panic.
    fn apply(
        &self,
        plan_id: &str,
        token: &str,
    ) -> Result<ExtractionApplyOutcomeDto, ExtractionApplyRefusal>;
}

/// Every refusal shape the apply route can answer, verbatim from shape C2.
#[derive(Debug)]
pub enum ExtractionApplyRefusal {
    UnknownPlan,
    StaleGeneration,
    NotApproved,    // missing, unknown, replayed or foreign-plan token
    InProgress,     // another apply holds this generation's lock
    Failed(String), // transaction error; the tree was rolled back (AR-04)
}

pub(crate) async fn api_extraction_apply(
    State(state): State<Arc<UiState>>,
    Path(plan_id): Path<String>,
    body: axum::body::Bytes,
) -> Response {
    let (applier, live_epoch) = match applier_ready_for_request(&state) {
        Ok(ready) => ready,
        Err(refused) => return refused,
    };
    let token = match approved_token_from_body(&body) {
        Some(token) => token,
        None => return dry_run_required_response(&plan_id),
    };
    // C3: the transaction runs on the blocking pool (the `spawn_watcher`
    // build precedent) so a multi-second apply never blocks the async
    // runtime; the trait bound `Send + Sync` makes the `Arc` movable.
    let plan_for_task = plan_id.clone();
    let transaction =
        tokio::task::spawn_blocking(move || applier.apply(&plan_for_task, &token)).await;
    match transaction {
        Ok(Ok(outcome)) => (StatusCode::OK, Json(outcome)).into_response(),
        Ok(Err(refusal)) => refusal_response(refusal, &plan_id, live_epoch),
        // A stopped blocking task never commits: the honest shape is the
        // transaction-error row, with the tree untouched.
        Err(join_failure) => {
            apply_failed_response(&plan_id, &format!("apply task stopped: {join_failure}"))
        }
    }
}

/// C3 check order, the twin of the GET family's `reader_ready_for_request`:
/// no applier installed -> 503; applier epoch != live analysis epoch -> 409
/// with the live epoch. `Ok` carries the request's `Arc` handle plus the
/// live epoch the refusal bodies echo; `Err` carries the finished response.
fn applier_ready_for_request(
    state: &UiState,
) -> Result<(Arc<dyn ExtractionPlanApplier>, u64), Response> {
    let applier = state
        .extraction_applier()
        .ok_or_else(analysis_loading_response)?;
    let live_epoch = state.analysis_epoch();
    if applier.epoch() != live_epoch {
        return Err(stale_generation_response(live_epoch));
    }
    Ok((applier, live_epoch))
}

/// Tolerant parse (C3, W2b uniform error family): an absent, unparseable
/// or empty-token body is the `dry_run_required` refusal - never an
/// extractor 4xx. `Bytes` reads the raw body without a content-type
/// requirement, so only the JSON parse itself decides.
fn approved_token_from_body(body: &[u8]) -> Option<String> {
    let request: ExtractionApplyRequestDto = serde_json::from_slice(body).ok()?;
    if request.apply_token.is_empty() {
        return None;
    }
    Some(request.apply_token)
}

/// The C3 refusal table, every variant mapped, including
/// `StaleGeneration -> 409 stale_generation` as the defensive second line
/// behind the route's own epoch check (it also keeps the variant
/// constructible for the A-4 provider).
fn refusal_response(refusal: ExtractionApplyRefusal, plan_id: &str, live_epoch: u64) -> Response {
    match refusal {
        ExtractionApplyRefusal::UnknownPlan => unknown_plan_response(plan_id),
        ExtractionApplyRefusal::StaleGeneration => stale_generation_response(live_epoch),
        ExtractionApplyRefusal::NotApproved => dry_run_required_response(plan_id),
        ExtractionApplyRefusal::InProgress => apply_in_progress_response(plan_id),
        ExtractionApplyRefusal::Failed(detail) => apply_failed_response(plan_id, &detail),
    }
}

fn analysis_loading_response() -> Response {
    error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        serde_json::json!({ "error": "analysis_loading" }),
    )
}

fn stale_generation_response(live_epoch: u64) -> Response {
    error_response(
        StatusCode::CONFLICT,
        serde_json::json!({ "error": "stale_generation", "epoch": live_epoch }),
    )
}

fn unknown_plan_response(plan_id: &str) -> Response {
    error_response(
        StatusCode::NOT_FOUND,
        serde_json::json!({ "error": "unknown_plan", "plan_id": plan_id }),
    )
}

fn dry_run_required_response(plan_id: &str) -> Response {
    error_response(
        StatusCode::PRECONDITION_REQUIRED,
        serde_json::json!({ "error": "dry_run_required", "plan_id": plan_id }),
    )
}

fn apply_in_progress_response(plan_id: &str) -> Response {
    error_response(
        StatusCode::CONFLICT,
        serde_json::json!({ "error": "apply_in_progress", "plan_id": plan_id }),
    )
}

/// The transaction-error row: the tree is already rolled back (AR-04) -
/// the detail text arrives ASCII-sanitized by the applier (A-4 owns the
/// sanitization; this route only carries it).
fn apply_failed_response(plan_id: &str, detail: &str) -> Response {
    error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        serde_json::json!({
            "error": "apply_failed",
            "plan_id": plan_id,
            "detail": detail,
        }),
    )
}

fn error_response(status: StatusCode, body: serde_json::Value) -> Response {
    (status, Json(body)).into_response()
}
