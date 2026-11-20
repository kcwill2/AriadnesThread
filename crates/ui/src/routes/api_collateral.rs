//! On-demand collateral routes (see `collateral_index`): one file's row,
//! one header's column, and the per-header burden totals.

use crate::collateral_index::{CollateralBurden, CollateralIndex};
use crate::state::{read_state, UiState};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CollateralFileQuery {
    pub file: String,
}

/// The installed index, else one built from the payload's own map (servers
/// started from a ready payload, e.g. `UiState::new_loaded`); 503 while
/// nothing is loaded yet.
fn current_index(state: &UiState) -> Result<Arc<CollateralIndex>, StatusCode> {
    if let Some(index) = state.collateral_index() {
        return Ok(index);
    }
    let payload = read_state(&state.payload);
    let payload = payload.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Arc::new(CollateralIndex::from_nested_map(&payload.file_collateral)))
}

fn parse_file_id(query: &CollateralFileQuery) -> Result<u64, StatusCode> {
    query.file.parse::<u64>().map_err(|_| StatusCode::BAD_REQUEST)
}

pub(crate) async fn api_collateral_row(
    State(state): State<Arc<UiState>>,
    Query(query): Query<CollateralFileQuery>,
) -> Result<Json<BTreeMap<String, usize>>, StatusCode> {
    let root = parse_file_id(&query)?;
    Ok(Json(current_index(&state)?.row(root)))
}

pub(crate) async fn api_collateral_column(
    State(state): State<Arc<UiState>>,
    Query(query): Query<CollateralFileQuery>,
) -> Result<Json<BTreeMap<String, usize>>, StatusCode> {
    let source = parse_file_id(&query)?;
    Ok(Json(current_index(&state)?.column(source)))
}

pub(crate) async fn api_collateral_burden(
    State(state): State<Arc<UiState>>,
) -> Result<Json<BTreeMap<String, CollateralBurden>>, StatusCode> {
    Ok(Json(current_index(&state)?.burden()))
}
