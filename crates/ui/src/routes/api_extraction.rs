//! Wizard route family (ui-extraction-wizard W-2b): the plain-data reader
//! trait the host (cli) implements, plus the two GET routes
//! `/api/extraction/:plan_id/manifest` and `/api/extraction/:plan_id/dry-run`
//! (axum 0.7 path-parameter syntax; the contract documents them with the
//! `{plan_id}` form, the requested URLs are identical).
//!
//! Read-only invariant (W1): this module registers GET routes only and
//! performs no filesystem write and no artifact-manager access (string
//! locks in `crates/ui/tests/ui_test/extraction_wizard_api.rs` rule out the
//! exact forbidden tokens without quoting them here); the mutation surface
//! is deferred to the `extraction-auto-apply` milestone.
//!
//! Error bodies follow the uniform ASCII shapes of shape C3, checked in
//! this order: no reader installed -> 503 `analysis_loading`; reader epoch
//! differs from the live `analysis_epoch` -> 409 `stale_generation` with
//! the live epoch (W6); plan unknown to the reader -> 404 `unknown_plan`
//! with the requested id.

use crate::dto::{ExtractionDryRunDto, ExtractionManifestDto};
use crate::state::UiState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;

/// Read-only extraction computation the wizard routes need. The host (cli)
/// implements it against the isolation engine and the include rewriter; no
/// implementation may write to disk (W1, W2).
pub trait ExtractionManifestReader: Send + Sync + 'static {
    fn epoch(&self) -> u64;
    fn manifest(&self, plan_id: &str) -> Option<ExtractionManifestDto>;
    fn dry_run(&self, plan_id: &str) -> Option<ExtractionDryRunDto>;
}

pub(crate) async fn api_extraction_manifest(
    State(state): State<Arc<UiState>>,
    Path(plan_id): Path<String>,
) -> Response {
    let reader = match reader_ready_for_request(&state) {
        Ok(reader) => reader,
        Err(refused) => return refused,
    };
    match reader.manifest(&plan_id) {
        Some(manifest) => (StatusCode::OK, Json(manifest)).into_response(),
        None => unknown_plan_response(&plan_id),
    }
}

pub(crate) async fn api_extraction_dry_run(
    State(state): State<Arc<UiState>>,
    Path(plan_id): Path<String>,
) -> Response {
    let reader = match reader_ready_for_request(&state) {
        Ok(reader) => reader,
        Err(refused) => return refused,
    };
    match reader.dry_run(&plan_id) {
        Some(mut dry_run) => {
            dry_run.apply_token = approved_apply_token(&state, &plan_id);
            (StatusCode::OK, Json(dry_run)).into_response()
        }
        None => unknown_plan_response(&plan_id),
    }
}

/// C4 mint point (extraction-auto-apply A-3): after a `Some` dry-run
/// answer, a live-epoch applier mints the single-use approval token for
/// this plan into the response's `apply_token` field. Without an applier,
/// or on an epoch mismatch, the field stays empty - the approval gate
/// never fabricates a token. The value is a gate, not a credential: the
/// server has no authentication by design and keeps the localhost bind
/// default (A8).
fn approved_apply_token(state: &UiState, plan_id: &str) -> String {
    let Some(applier) = state.extraction_applier() else {
        return String::new();
    };
    if applier.epoch() != state.analysis_epoch() {
        return String::new();
    }
    applier.mint_apply_token(plan_id).unwrap_or_default()
}

/// C3 check order, shared by both handlers: no reader installed -> 503;
/// reader epoch != live analysis epoch -> 409 (W6). `Ok` carries the
/// request's `Arc` handle; `Err` carries the finished error response.
fn reader_ready_for_request(
    state: &UiState,
) -> Result<Arc<dyn ExtractionManifestReader>, Response> {
    let reader = state
        .extraction_reader()
        .ok_or_else(analysis_loading_response)?;
    let live_epoch = state.analysis_epoch();
    if reader.epoch() != live_epoch {
        return Err(stale_generation_response(live_epoch));
    }
    Ok(reader)
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

fn error_response(status: StatusCode, body: serde_json::Value) -> Response {
    (status, Json(body)).into_response()
}
