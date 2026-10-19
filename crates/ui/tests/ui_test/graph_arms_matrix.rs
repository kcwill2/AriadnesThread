//! UTA2-R1 (audit rows 7-9): the `/api/graph/traverse` symbol and call arms,
//! the `direction` parameter, `filter_elements` edge-id format verified
//! against REAL payload ids, the `/api/graph/path` symbol/call arms, and the
//! loaded-state pass-through bodies of `/api/graph/{symbol,call,include}`.

use crate::common;
use ariadne_graph::{CallGraph, ReducedSemanticModel};
use ariadne_ui::{
    build_call_elements, build_router, build_symbol_elements, UiState,
};
use axum::http::StatusCode;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

fn symbol_record(id: &str, file: u64) -> ariadne_graph::ReducedSymbolRecord {
    ariadne_graph::ReducedSymbolRecord {
        id: ariadne_index::SymbolId(id.to_string()),
        name: id.to_string(),
        kind: ariadne_index::SymbolKind::Function,
        defining_file: Some(ariadne_repo::FileId(file)),
        declarations: vec![],
        definition: None,
        declaring_files: vec![],
    }
}

fn semantic_edge(src: &str, dst: &str) -> ariadne_graph::ReducedSemanticEdge {
    ariadne_graph::ReducedSemanticEdge {
        src_symbol: ariadne_index::SymbolId(src.to_string()),
        dst_symbol: ariadne_index::SymbolId(dst.to_string()),
        kind: ariadne_index::DependencyKind::Complete,
    }
}

fn call_node(id: &str) -> ariadne_graph::CallNode {
    ariadne_graph::CallNode {
        symbol_id: ariadne_index::SymbolId(id.to_string()),
        name: id.to_string(),
        owning_file: ariadne_repo::FileId(3),
        location: None,
    }
}

fn call_edge(caller: &str, callee: &str) -> ariadne_graph::CallEdge {
    ariadne_graph::CallEdge {
        caller: ariadne_index::SymbolId(caller.to_string()),
        callee: ariadne_index::SymbolId(callee.to_string()),
        call_count: 1,
        call_locations: vec![],
    }
}

/// Loaded fixture: include chain a.h(1)->b.h(2)->c.cc(3) + isolated d.h(7)
/// (shared fixture), a symbol chain a1->a2->a3 plus an unconnected `solo`,
/// and a call chain m->h1->h2. The payload carries the REAL builder outputs,
/// so filter-element id formats are verified against production ids.
fn arms_state() -> (Arc<UiState>, Vec<ariadne_ui::CytoscapeElement>, Vec<ariadne_ui::CytoscapeElement>) {
    let (include_graph, mut payload) = common::fixture_include_graph();
    let model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols: std::collections::BTreeMap::from([
            (
                ariadne_index::SymbolId("a1".to_string()),
                symbol_record("a1", 1),
            ),
            (
                ariadne_index::SymbolId("a2".to_string()),
                symbol_record("a2", 2),
            ),
            (
                ariadne_index::SymbolId("a3".to_string()),
                symbol_record("a3", 3),
            ),
            (
                ariadne_index::SymbolId("solo".to_string()),
                symbol_record("solo", 1),
            ),
        ]),
        edges: vec![semantic_edge("a1", "a2"), semantic_edge("a2", "a3")],
        tu_usage: std::collections::BTreeMap::new(),
        file_symbols: std::collections::BTreeMap::from([
            (
                ariadne_repo::FileId(1),
                vec![
                    ariadne_index::SymbolId("a1".to_string()),
                    ariadne_index::SymbolId("solo".to_string()),
                ],
            ),
            (
                ariadne_repo::FileId(2),
                vec![ariadne_index::SymbolId("a2".to_string())],
            ),
            (
                ariadne_repo::FileId(3),
                vec![ariadne_index::SymbolId("a3".to_string())],
            ),
        ]),
        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    let call_graph = CallGraph {
        nodes: vec![call_node("m"), call_node("h1"), call_node("h2")],
        edges: vec![call_edge("m", "h1"), call_edge("h1", "h2")],
    };
    let sym_paths: HashMap<u64, String> = [(1u64, "a.h"), (2, "b.h"), (3, "c.cc"), (7, "d.h")]
        .into_iter()
        .map(|(id, path)| (id, path.to_string()))
        .collect();
    let symbol_elements = build_symbol_elements(&model, &sym_paths);
    let call_elements = build_call_elements(&call_graph);
    payload.symbol_graph = symbol_elements.clone();
    payload.call_graph = call_elements.clone();
    // Fixture fix (successor session, journal: tasks/uta2-r1-journal.md):
    // `common::ready_state` hardwires an EMPTY reduced model, so the symbol
    // arm of traverse/path never saw this fixture's chain (center-only /
    // Null reds). The symbol arm reads `state.analysis().reduced_model`,
    // so the loaded state is built directly through `UiState::new_loaded`
    // with the model the assertions are written against; no assertion
    // weakened - the routes under test finally receive their documented
    // input.
    let state = Arc::new(UiState::new_loaded(
        std::path::PathBuf::from("."),
        include_graph,
        model,
        call_graph,
        payload,
    ));
    (state, symbol_elements, call_elements)
}

fn element_sequence(body: &Value) -> Vec<String> {
    body.as_array()
        .expect("element array")
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
async fn test_traverse_symbol_arm_depth_and_edge_format_against_payload() {
    let (state, symbol_elements, _calls) = arms_state();
    let app = build_router(state);
    let payload_edge_ids: Vec<String> = symbol_elements
        .iter()
        .filter(|element| element.data.get("source").is_some())
        .map(|element| element.data["id"].as_str().expect("id").to_string())
        .collect();

    let (status, _headers, one_hop) = common::request_route(
        app.clone(),
        "/api/graph/traverse?graph_type=symbol&center=a1&depth=1",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        element_sequence(&one_hop),
        vec!["a1".to_string(), "a2".to_string(), "a1-a2".to_string()],
        "depth 1 from a1: payload order preserved by filter_elements"
    );
    let edge = &one_hop[2];
    assert_eq!(edge["data"]["id"], "a1-a2");
    assert_eq!(edge["data"]["source"], "a1");
    assert_eq!(edge["data"]["target"], "a2");
    assert!(
        payload_edge_ids.contains(&"a1-a2".to_string()),
        "the traversed edge id must exist among the real symbol-graph payload edge ids {payload_edge_ids:?}"
    );

    let (_status, _headers, two_hops) = common::request_route(
        app.clone(),
        "/api/graph/traverse?graph_type=symbol&center=a1&depth=2",
    )
    .await;
    assert_eq!(
        element_sequence(&two_hops),
        vec![
            "a1".to_string(),
            "a2".to_string(),
            "a3".to_string(),
            "a1-a2".to_string(),
            "a2-a3".to_string()
        ]
    );

    let (_status, _headers, alone) = common::request_route(
        app.clone(),
        "/api/graph/traverse?graph_type=symbol&center=solo",
    )
    .await;
    assert_eq!(element_sequence(&alone), vec!["solo".to_string()]);

    let (_status, _headers, ghost) = common::request_route(
        app,
        "/api/graph/traverse?graph_type=symbol&center=never-seen",
    )
    .await;
    assert_eq!(
        ghost,
        serde_json::json!([]),
        "a center outside the graph yields an empty element list (no error)"
    );
}

#[tokio::test]
async fn test_traverse_symbol_arm_direction_parameter() {
    let (state, _symbols, _calls) = arms_state();
    let app = build_router(state);
    let cases: [(&str, Vec<String>); 4] = [
        (
            "&direction=in",
            vec!["a1".to_string(), "a2".to_string(), "a1-a2".to_string()],
        ),
        (
            "&direction=incoming",
            vec!["a1".to_string(), "a2".to_string(), "a1-a2".to_string()],
        ),
        (
            "&direction=out",
            vec!["a2".to_string(), "a3".to_string(), "a2-a3".to_string()],
        ),
        (
            "&direction=outgoing",
            vec!["a2".to_string(), "a3".to_string(), "a2-a3".to_string()],
        ),
    ];
    for (suffix, expected) in cases {
        let uri = format!("/api/graph/traverse?graph_type=symbol&center=a2&depth=1{suffix}");
        let (status, _headers, body) = common::request_route(app.clone(), &uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert_eq!(element_sequence(&body), expected, "{uri}");
    }
    let (_status, _headers, both) = common::request_route(
        app,
        "/api/graph/traverse?graph_type=symbol&center=a2&depth=1",
    )
    .await;
    assert_eq!(
        element_sequence(&both),
        vec![
            "a1".to_string(),
            "a2".to_string(),
            "a3".to_string(),
            "a1-a2".to_string(),
            "a2-a3".to_string()
        ],
        "absent direction defaults to Both"
    );
}

#[tokio::test]
async fn test_traverse_call_arm_direction_and_edge_format_against_payload() {
    let (state, _symbols, call_elements) = arms_state();
    let app = build_router(state);
    let payload_edge_ids: Vec<String> = call_elements
        .iter()
        .filter(|element| element.data.get("source").is_some())
        .map(|element| element.data["id"].as_str().expect("id").to_string())
        .collect();

    let cases: [(&str, Vec<String>); 3] = [
        (
            "",
            vec![
                "m".to_string(),
                "h1".to_string(),
                "h2".to_string(),
                "m-h1".to_string(),
                "h1-h2".to_string(),
            ],
        ),
        (
            "&direction=in",
            vec!["m".to_string(), "h1".to_string(), "m-h1".to_string()],
        ),
        (
            "&direction=out",
            vec!["h1".to_string(), "h2".to_string(), "h1-h2".to_string()],
        ),
    ];
    for (suffix, expected) in cases {
        let uri = format!("/api/graph/traverse?graph_type=call&center=h1&depth=1{suffix}");
        let (status, _headers, body) = common::request_route(app.clone(), &uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert_eq!(element_sequence(&body), expected, "{uri}");
    }
    let (_status, _headers, body) = common::request_route(
        app,
        "/api/graph/traverse?graph_type=call&center=h1&depth=1&direction=in",
    )
    .await;
    let edge = &body[2];
    assert_eq!(edge["data"]["id"], "m-h1");
    assert_eq!(edge["data"]["source"], "m");
    assert_eq!(edge["data"]["target"], "h1");
    assert!(
        payload_edge_ids.contains(&"m-h1".to_string()),
        "the traversed call edge id must exist among the real call payload edge ids {payload_edge_ids:?}"
    );
}

#[tokio::test]
async fn test_graph_path_symbol_and_call_arms_exact_sequences() {
    let (state, _symbols, _calls) = arms_state();
    let app = build_router(state);
    let (status, _headers, body) = common::request_route(
        app.clone(),
        "/api/graph/path?graph_type=symbol&from=a1&to=a3",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        serde_json::json!(["a1", "a2", "a3"]),
        "symbol arm: exact id sequence along the semantic chain"
    );
    let (_status, _headers, reverse) = common::request_route(
        app.clone(),
        "/api/graph/path?graph_type=symbol&from=a3&to=a1",
    )
    .await;
    assert_eq!(reverse, serde_json::Value::Null, "edges are directional");
    let (_status, _headers, same) = common::request_route(
        app.clone(),
        "/api/graph/path?graph_type=symbol&from=a2&to=a2",
    )
    .await;
    assert_eq!(same, serde_json::json!(["a2"]), "from==to short-circuits");

    let (_status, _headers, call_chain) = common::request_route(
        app.clone(),
        "/api/graph/path?graph_type=call&from=m&to=h2",
    )
    .await;
    assert_eq!(
        call_chain,
        serde_json::json!(["m", "h1", "h2"]),
        "call arm: exact id sequence along the call chain"
    );
    let (_status, _headers, call_reverse) = common::request_route(
        app,
        "/api/graph/path?graph_type=call&from=h2&to=m",
    )
    .await;
    assert_eq!(call_reverse, serde_json::Value::Null);
}

#[tokio::test]
async fn test_graph_symbol_call_include_routes_pass_through_loaded_payload() {
    let (state, symbol_elements, call_elements) = arms_state();
    let app = build_router(state);
    let (status, _headers, symbol_body) = common::request_route(app.clone(), "/api/graph/symbol").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        symbol_body,
        serde_json::to_value(&symbol_elements).expect("symbol elements serialize"),
        "/api/graph/symbol must serve the payload's symbol_graph verbatim"
    );

    let (_status, _headers, call_body) =
        common::request_route(app.clone(), "/api/graph/call").await;
    assert_eq!(
        call_body,
        serde_json::to_value(&call_elements).expect("call elements serialize"),
        "/api/graph/call must serve the payload's call_graph verbatim"
    );

    let (_status, _headers, include_body) = common::request_route(app, "/api/graph/include").await;
    assert!(include_body.is_array(), "include passthrough serves an array");
    assert!(
        !include_body.as_array().expect("array").is_empty(),
        "the loaded fixture's include payload is non-empty"
    );
    let ids = element_sequence(&include_body);
    for expected in ["1", "2", "3", "7", "1-2", "2-3"] {
        assert!(ids.iter().any(|id| id == expected), "include id {expected}: {ids:?}");
    }
}
