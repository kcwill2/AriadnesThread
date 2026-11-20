//! On-demand collateral routes: the live page fetches one file's row, one
//! header's column, and the per-header burden totals instead of the whole
//! file x header map in `/api/export`. Answers come from the installed
//! compact index, or from the payload's own map when none is installed.

use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph};
use ariadne_ui::{build_router, CollateralIndex, GraphExportPayload};
use axum::http::StatusCode;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Roots 1 and 2 both pay header 10; root 2 also pays header 11.
fn nested_fixture() -> HashMap<String, HashMap<String, usize>> {
    let mut map = HashMap::new();
    map.insert("1".to_string(), HashMap::from([("10".to_string(), 5usize)]));
    map.insert(
        "2".to_string(),
        HashMap::from([("10".to_string(), 3usize), ("11".to_string(), 7usize)]),
    );
    map
}

fn router_with(payload: GraphExportPayload, index: Option<CollateralIndex>) -> axum::Router {
    let state = common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        payload,
    );
    if let Some(index) = index {
        state.install_collateral_index(Arc::new(index));
    }
    build_router(state)
}

async fn assert_routes_answer(app: axum::Router) {
    let (status, _, row) = common::request_route(app.clone(), "/api/collateral/row?file=2").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(row, json!({"10": 3, "11": 7}));

    let (_, _, column) = common::request_route(app.clone(), "/api/collateral/column?file=10").await;
    assert_eq!(column, json!({"1": 5, "2": 3}));

    let (_, _, burden) = common::request_route(app.clone(), "/api/collateral/burden").await;
    assert_eq!(
        burden,
        json!({"10": {"burden": 8, "affected": 2}, "11": {"burden": 7, "affected": 1}})
    );

    let (_, _, unknown) = common::request_route(app, "/api/collateral/row?file=99").await;
    assert_eq!(unknown, json!({}), "a file without collateral answers an empty row");
}

#[tokio::test]
async fn routes_answer_from_the_installed_index() {
    let index = CollateralIndex::from_nested_map(&nested_fixture());
    assert_routes_answer(router_with(common::empty_export_payload(), Some(index))).await;
}

#[tokio::test]
async fn routes_fall_back_to_the_payload_map_without_an_index() {
    let mut payload = common::empty_export_payload();
    payload.file_collateral = nested_fixture();
    assert_routes_answer(router_with(payload, None)).await;
}

#[tokio::test]
async fn a_non_numeric_file_id_is_a_bad_request() {
    let app = router_with(common::empty_export_payload(), None);
    let (status, _, _) = common::request_route(app, "/api/collateral/row?file=abc").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[test]
fn the_index_round_trips_the_nested_map() {
    let index = CollateralIndex::from_nested_map(&nested_fixture());
    assert_eq!(index.to_nested_map(), nested_fixture());
}

/// The live page reads justification and proposals from this lean route;
/// the three graphs have their own routes and are not repeated here.
#[tokio::test]
async fn analysis_route_carries_justification_without_the_graphs() {
    let mut payload = common::empty_export_payload();
    payload.file_justification.insert("1".to_string(), HashMap::new());
    let app = router_with(payload, None);
    let (status, _, body) = common::request_route(app, "/api/analysis").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["file_justification"], json!({"1": {}}));
    assert_eq!(body["extraction_proposals"], json!([]));
    for graph in ["include_graph", "symbol_graph", "call_graph", "file_collateral"] {
        assert!(body.get(graph).is_none(), "/api/analysis must not repeat {graph}");
    }
}
