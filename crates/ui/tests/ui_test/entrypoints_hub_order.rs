//! UTA2-R1 (audit row UTB-4): `/api/graph/entrypoints` `top_hubs` must carry
//! the degree-rank order `get_entrypoints` already returns. The handler
//! re-filtered the ranks through payload emission order (the admitted skip
//! at api_graph.rs:289), silently replacing the documented rank semantics.
//! JS inspection (journal): NO consumer depends on the old emission order -
//! traversal.js never calls this endpoint; api.js's sole consumer maps the
//! object as an array and always falls into its error branch - so the
//! handoff's decision row lands on FIX.

use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph};
use ariadne_ui::{build_include_elements, build_router};
use axum::http::StatusCode;
use serde_json::Value;
use std::collections::HashMap;

fn include_edge(source: u64, destination: u64) -> ariadne_graph::IncludeEdge {
    ariadne_graph::IncludeEdge {
        source: ariadne_repo::FileId(source),
        destination: Some(ariadne_repo::FileId(destination)),
        context_name: String::new(),
        source_location: ariadne_graph::SourceLocation { line: 1, column: 1 },
        kind: ariadne_graph::IncludeKind::Local,
        raw_path: String::new(),
        resolved_path: None,
        language: ariadne_graph::LanguageId::Cpp,
    }
}

/// Degrees: b.h(2)=in1+out1=2, g.h(7)=out2=2, a.h(1)=in1, z.h(30)=in1.
/// get_entrypoints ranks hubs by degree-desc, id-asc tie-break: [2,7,1,30].
/// Payload elements emit in (path asc, file_id asc): [1,2,7,30] - a
/// DIFFERENT order, so the rank sequence is observable.
fn hub_order_app() -> axum::Router {
    let mut graph = IncludeGraph::new();
    for (id, path) in [(1u64, "a.h"), (2, "b.h"), (7, "g.h"), (30, "z.h")] {
        graph
            .file_paths
            .insert(ariadne_repo::FileId(id), std::path::PathBuf::from(path));
    }
    for (source, destination) in [(7u64, 30u64), (7, 2), (2, 1)] {
        graph.edges.push(include_edge(source, destination));
    }
    let mut payload = common::empty_export_payload();
    payload.include_graph = build_include_elements(
        &graph,
        &HashMap::new(),
        &[],
        &HashMap::new(),
        &HashMap::new(),
    );
    build_router(common::ready_state(
        std::path::PathBuf::from("."),
        graph,
        CallGraph::default(),
        payload,
    ))
}

fn hub_ids(body: &Value) -> Vec<String> {
    body["top_hubs"]
        .as_array()
        .expect("top_hubs must be an array")
        .iter()
        .map(|element| {
            element["data"]["id"]
                .as_str()
                .expect("string id")
                .to_string()
        })
        .collect()
}

#[tokio::test]
async fn test_entrypoints_top_hubs_follow_degree_rank_order() {
    // RED (UTB-4): the raw emission sequence must be the rank order, NOT
    // the payload element order (pre-fix: ["1","2","7","30"]).
    let app = hub_order_app();
    let (status, _headers, body) =
        common::request_route(app, "/api/graph/entrypoints?graph_type=include").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["graph_type"], "include");
    assert_eq!(body["cycles"], serde_json::json!([]));
    assert_eq!(
        hub_ids(&body),
        vec!["2".to_string(), "7".to_string(), "1".to_string(), "30".to_string()],
        "top_hubs must preserve get_entrypoints degree-rank order"
    );
    let mut roots: Vec<String> = body["roots"]
        .as_array()
        .expect("roots array")
        .iter()
        .map(|element| element["data"]["id"].as_str().expect("id").to_string())
        .collect();
    roots.sort();
    assert_eq!(roots, vec!["7".to_string()], "only g.h has zero in-degree");
    let mut leaves: Vec<String> = body["leaves"]
        .as_array()
        .expect("leaves array")
        .iter()
        .map(|element| element["data"]["id"].as_str().expect("id").to_string())
        .collect();
    leaves.sort();
    assert_eq!(
        leaves,
        vec!["1".to_string(), "30".to_string()],
        "a.h and z.h have zero out-degree"
    );
}

#[tokio::test]
async fn test_entrypoints_top_hubs_limit_truncates_the_rank_prefix() {
    // RED (UTB-4): limit must keep the TOP-ranked hubs, not the
    // emission-order subset (pre-fix limit=3 filtered emission gives
    // ["1","2","7"] - a different set than the rank prefix ["2","7","1"]).
    let app = hub_order_app();
    let (status, _headers, body) = common::request_route(
        app,
        "/api/graph/entrypoints?graph_type=include&limit=3",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        hub_ids(&body),
        vec!["2".to_string(), "7".to_string(), "1".to_string()],
        "limit=3 must yield the three highest-ranked hubs in rank order"
    );
}
