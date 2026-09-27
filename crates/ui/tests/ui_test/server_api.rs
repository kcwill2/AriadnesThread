use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_ui::{build_router, GraphExportPayload, SummaryStats, UiState};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;

#[tokio::test]
async fn test_server_routes() {
    let state = Arc::new(UiState::new_loaded(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        ReducedSemanticModel {
            repo_root: std::path::PathBuf::from("."),
            symbols: std::collections::BTreeMap::new(),
            edges: vec![],
            tu_usage: std::collections::BTreeMap::new(),
            file_symbols: std::collections::BTreeMap::new(),
        },
        CallGraph::default(),
        GraphExportPayload {
            repo_name: "test".to_string(),
            include_graph: vec![],
            symbol_graph: vec![],
            call_graph: vec![],
            stats: SummaryStats {
                total_files: 0,
                total_symbols: 0,
                total_calls: 0,
                cycle_count: 0,
            },
            file_collateral: std::collections::HashMap::new(),
            file_justification: std::collections::HashMap::new(),
            extraction_proposals: vec![],
        },
    ));

    let app = build_router(state);

    let req = Request::builder()
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_async_ui_state_loading() {
    let state = Arc::new(UiState::new_loading(std::path::PathBuf::from(".")));
    let app = build_router(state.clone());

    // Before loading, status should be loading
    let req = Request::builder()
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let status: ariadne_ui::server::StatusResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(status.progress.state, "loading");
    assert_eq!(status.progress.percent, 0);

    // Graph endpoints should return 503 or empty array when loading. Let's say empty array as per unit 1 or 503.
    let req = Request::builder()
        .uri("/api/graph/include")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);

    // Populate data
    {
        let mut progress = state.progress.write().unwrap();
        progress.state = "ready".to_string();
        progress.percent = 100;
        *state.payload.write().unwrap() = Some(GraphExportPayload {
            repo_name: "test".to_string(),
            include_graph: vec![],
            symbol_graph: vec![],
            call_graph: vec![],
            stats: SummaryStats {
                total_files: 0,
                total_symbols: 0,
                total_calls: 0,
                cycle_count: 0,
            },
            file_collateral: HashMap::new(),
            file_justification: HashMap::new(),
            extraction_proposals: vec![],
        });
        *state.include_graph.write().unwrap() = Some(IncludeGraph::new());
        *state.reduced_model.write().unwrap() = Some(ReducedSemanticModel {
            repo_root: std::path::PathBuf::from("."),
            symbols: std::collections::BTreeMap::new(),
            edges: vec![],
            tu_usage: std::collections::BTreeMap::new(),
            file_symbols: std::collections::BTreeMap::new(),
        });
        *state.call_graph.write().unwrap() = Some(CallGraph::default());
    }

    // Now status should be ready
    let req = Request::builder()
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let status: ariadne_ui::server::StatusResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(status.progress.state, "ready");

    // Graph endpoints should now return OK
    let req = Request::builder()
        .uri("/api/graph/include")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_server_index_route() {
    let state = Arc::new(UiState::new_loaded(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        ReducedSemanticModel {
            repo_root: std::path::PathBuf::from("."),
            symbols: std::collections::BTreeMap::new(),
            edges: vec![],
            tu_usage: std::collections::BTreeMap::new(),
            file_symbols: std::collections::BTreeMap::new(),
        },
        CallGraph::default(),
        GraphExportPayload {
            repo_name: "test".to_string(),
            include_graph: vec![],
            symbol_graph: vec![],
            call_graph: vec![],
            stats: SummaryStats {
                total_files: 0,
                total_symbols: 0,
                total_calls: 0,
                cycle_count: 0,
            },
            file_collateral: std::collections::HashMap::new(),
            file_justification: std::collections::HashMap::new(),
            extraction_proposals: vec![],
        },
    ));

    let app = build_router(state);

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get("Content-Type").unwrap(),
        "text/html; charset=utf-8"
    );
    assert_eq!(
        res.headers().get("Cache-Control").unwrap(),
        "no-store",
        "index must never be cached: stale HTML silently hides new UI features"
    );
}

#[tokio::test]
async fn test_index_route_body_matches_bundled_index_html() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let (status, headers, body) = common::request_raw_body(app, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get("Cache-Control").unwrap(), "no-store");
    assert_eq!(
        headers.get("Content-Type").unwrap(),
        "text/html; charset=utf-8"
    );
    assert_eq!(
        body,
        ariadne_ui::INDEX_HTML.as_bytes(),
        "index route must serve the exact bundled INDEX_HTML"
    );
}

#[tokio::test]
async fn test_unknown_path_returns_404() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let (status, _headers, _json) = common::request_route(app, "/api/no-such-route").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_status_route_rejects_post_with_method_not_allowed() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let request = Request::builder()
        .method("POST")
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_status_route_locks_ready_response_shape() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let (status, _headers, body) = common::request_route(app, "/api/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repo_name"], "fixture-repo");
    assert_eq!(body["progress"]["state"], "ready");
    assert_eq!(body["progress"]["percent"], 100);
    assert!(
        body["stats"].is_object(),
        "stats must be an object when loaded"
    );
    assert_eq!(body["stats"]["total_files"], 0);
    assert_eq!(body["stages"], serde_json::json!([]));
    assert_eq!(body["last_refresh"], serde_json::Value::Null);
    assert_eq!(body["rebuilding"], false);
    assert_eq!(body["reused_tus"], 0);
    assert_eq!(body["reanalyzed_tus"], 0);
}

#[tokio::test]
async fn test_export_route_locks_payload_shape_and_omits_empty_collateral() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));

    let (status, _headers, body) = common::request_route(app, "/api/export").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["repo_name"], "fixture-repo");
    assert!(body["include_graph"].is_array());
    assert!(body["symbol_graph"].is_array());
    assert!(body["call_graph"].is_array());
    assert!(body["stats"].is_object());
    assert!(
        body.get("file_collateral").is_none(),
        "empty file_collateral is skipped during serialization; clients must default it"
    );
}

#[tokio::test]
async fn test_export_route_includes_file_collateral_when_non_empty() {
    let mut payload = common::empty_export_payload();
    let mut burden = HashMap::new();
    burden.insert("2".to_string(), 5usize);
    payload.file_collateral.insert("1".to_string(), burden);

    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        payload,
    ));

    let (status, _headers, body) = common::request_route(app, "/api/export").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["file_collateral"]["1"]["2"], 5);
}

#[tokio::test]
async fn test_export_route_returns_503_while_loading() {
    let app = build_router(common::loading_state());

    let (status, _headers, _json) = common::request_route(app, "/api/export").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

fn justification_sym(id: &str, file: u64) -> ariadne_graph::ReducedSymbolRecord {
    ariadne_graph::ReducedSymbolRecord {
        id: ariadne_index::SymbolId(id.to_string()),
        name: id.to_string(),
        kind: ariadne_index::SymbolKind::Function,
        defining_file: Some(ariadne_repo::FileId(file)),
        declarations: vec![],
        definition: None,
    }
}

/// Live-route fixture: root.cc(1) includes jobserver.h(2), huge.h(3), tiny.h(4),
/// d3.h(104). `main` justifies all 13 jobserver symbols (past the payload
/// 12-name cap); those reach r1@req1.h(5) and r2@req2.h(6). `main` also
/// justifies h000/h001 in huge.h (3) so the huge.h edge becomes E-I4-eligible
/// (justified>0 and collateral>0); huge.h drags 18 collateral symbols (h002..
/// h019) and tiny.h 13 — the tiny.h tie with jobserver.h's combined count
/// pins the source_id tie-break, while huge.h outranks despite its higher id.
/// huge.h owns a drop subtree d0.h..d3.h + mid.h(105) with mid.h -> d4..d8
/// (106..110); the direct root.cc -> d3.h edge makes d3.h the still-reachable
/// decoy. The route recomputes via analyze_file_load, so the model needs real
/// file_symbols/edges, not a prebuilt payload.
fn justification_state() -> Arc<UiState> {
    use std::collections::BTreeMap;

    let mut graph = IncludeGraph::new();
    let drop_paths: Vec<(u64, String)> = (0..4u64)
        .map(|i| (101 + i, format!("d{i}.h")))
        .chain(std::iter::once((105u64, "mid.h".to_string())))
        .chain((4..9u64).map(|i| (102 + i, format!("d{i}.h"))))
        .collect();
    let core_paths: Vec<(u64, std::path::PathBuf)> = vec![
        (1, "root.cc".into()),
        (2, "jobserver.h".into()),
        (3, "huge.h".into()),
        (4, "tiny.h".into()),
        (5, "req1.h".into()),
        (6, "req2.h".into()),
    ];
    for (id, path) in core_paths
        .into_iter()
        .chain(drop_paths.into_iter().map(|(id, p)| (id, p.into())))
    {
        graph
            .file_paths
            .insert(ariadne_repo::FileId(id), path);
    }
    let include_edge = |source: u64, destination: u64| ariadne_graph::IncludeEdge {
        source: ariadne_repo::FileId(source),
        destination: Some(ariadne_repo::FileId(destination)),
        context_name: String::new(),
        source_location: ariadne_graph::SourceLocation { line: 1, column: 1 },
        kind: ariadne_graph::IncludeKind::Local,
        raw_path: String::new(),
        resolved_path: None,
    };
    graph.edges.push(include_edge(1, 2));
    graph.edges.push(include_edge(1, 3));
    graph.edges.push(include_edge(1, 4));
    graph.edges.push(include_edge(1, 104));
    for dest in 101..=105 {
        graph.edges.push(include_edge(3, dest));
    }
    for dest in 106..=110 {
        graph.edges.push(include_edge(105, dest));
    }

    let mut symbols: BTreeMap<ariadne_index::SymbolId, ariadne_graph::ReducedSymbolRecord> =
        BTreeMap::new();
    let mut file_symbols: BTreeMap<ariadne_repo::FileId, Vec<ariadne_index::SymbolId>> =
        BTreeMap::new();
    let mut add_sym = |id: &str, file: u64| {
        let record = justification_sym(id, file);
        symbols.insert(record.id.clone(), record);
        file_symbols
            .entry(ariadne_repo::FileId(file))
            .or_default()
            .push(ariadne_index::SymbolId(id.to_string()));
    };
    add_sym("main", 1);
    for i in 0..13 {
        add_sym(&format!("j{i:03}"), 2);
    }
    for i in 0..20 {
        add_sym(&format!("h{i:03}"), 3);
    }
    for i in 0..13 {
        add_sym(&format!("t{i:03}"), 4);
    }
    add_sym("r1", 5);
    add_sym("r2", 6);

    let link = |src: &str, dst: &str| ariadne_graph::ReducedSemanticEdge {
        src_symbol: ariadne_index::SymbolId(src.to_string()),
        dst_symbol: ariadne_index::SymbolId(dst.to_string()),
        kind: ariadne_index::DependencyKind::Complete,
    };
    let mut edges: Vec<ariadne_graph::ReducedSemanticEdge> =
        (0..13).map(|i| link("main", &format!("j{i:03}"))).collect();
    edges.push(link("main", "h000"));
    edges.push(link("main", "h001"));
    edges.push(link("j000", "r1"));
    edges.push(link("j001", "r2"));

    let model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols,
        edges,
        tu_usage: BTreeMap::new(),
        file_symbols,
    };
    Arc::new(UiState::new_loaded(
        std::path::PathBuf::from("."),
        graph,
        model,
        CallGraph::default(),
        common::empty_export_payload(),
    ))
}

fn entry_ids(body: &serde_json::Value, key: &str) -> Vec<String> {
    body["entries"]
        .as_array()
        .expect("entries must be an array")
        .iter()
        .map(|e| e[key].as_str().expect("string field").to_string())
        .collect()
}

fn sorted_keys(value: &serde_json::Value) -> Vec<String> {
    let mut keys: Vec<String> = value
        .as_object()
        .expect("expected a JSON object")
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

#[tokio::test]
async fn test_justification_route_returns_unbounded_detail() {
    let app = build_router(justification_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/justification?file=1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["target_id"], "1");

    assert_eq!(
        entry_ids(&body, "source_id"),
        vec!["3", "2", "4"],
        "entries sort by combined count desc, then source_id asc on the 13-13 tie; \
         the root.cc self row is header-filtered out (E-I9 mirror)"
    );
    assert_eq!(
        entry_ids(&body, "source_label"),
        vec!["huge.h", "jobserver.h", "tiny.h"],
        "source_label carries the path basename; header sources only"
    );

    let job = &body["entries"][1]["entry"];
    assert_eq!(job["justified_count"], 13);
    assert_eq!(job["collateral_count"], 0);
    let names = job["justified_symbols"]
        .as_array()
        .expect("justified symbols array");
    assert_eq!(
        names.len(),
        13,
        "the route must not apply the payload 12-name cap"
    );
    assert!(
        names.iter().any(|n| n == "j012"),
        "13th justified name survives unbounded: {names:?}"
    );
    assert_eq!(job["justified_truncated"], false);
    assert_eq!(job["collateral_truncated"], false);
    assert_eq!(
        job["required_headers"],
        serde_json::json!(["req1.h", "req2.h"]),
        "per-entry required headers exclude the source's own basename"
    );

    let huge = &body["entries"][0]["entry"];
    assert_eq!(huge["justified_count"], 2);
    assert_eq!(
        huge["collateral_symbols"]
            .as_array()
            .expect("collateral symbols array")
            .len(),
        18,
        "collateral names are unbounded too (E2 fixture moves h000/h001 to justified)"
    );
    assert_eq!(huge["collateral_truncated"], false);
    assert_eq!(huge["required_headers"], serde_json::json!([]));

    assert_eq!(
        body["total_justified"], 15,
        "13 jobserver + 2 huge; the root.cc self row is header-filtered out"
    );
    assert_eq!(body["total_collateral"], 31, "18 huge + 13 tiny");
}

#[tokio::test]
async fn test_justification_route_locks_response_keys() {
    let app = build_router(justification_state());
    let (_status, _headers, body) =
        common::request_route(app, "/api/justification?file=1").await;
    assert_eq!(
        sorted_keys(&body),
        vec!["entries", "target_id", "total_collateral", "total_justified"]
    );
    // The eligible entry (huge.h: justified>0 and collateral>0, root!=source)
    // gains the full impact sub-object on the SOURCE row (not the entry).
    assert_eq!(
        sorted_keys(&body["entries"][0]),
        vec!["entry", "impact", "source_id", "source_label"]
    );
    assert_eq!(
        sorted_keys(&body["entries"][0]["impact"]),
        vec![
            "dropped_directly",
            "dropped_from_root",
            "required_headers",
            "root_closure_after",
            "root_closure_before",
            "root_file_id",
            "source_file_id",
            "still_reachable",
            "symbols_no_longer_parsed",
        ]
    );
    // The payload-only summary shape must NOT leak into the route (E-I4).
    assert_eq!(
        sorted_keys(&body["entries"][0]["impact"]["dropped_from_root"][0]),
        vec!["file_id", "path", "symbol_count"]
    );
    // Ineligible entry (jobserver.h: collateral_count 0) omits impact entirely.
    assert_eq!(
        sorted_keys(&body["entries"][1]),
        vec!["entry", "source_id", "source_label"],
        "ineligible source omits the impact key (skip_serializing_if)"
    );

    assert_eq!(
        sorted_keys(&body["entries"][0]["entry"]),
        vec![
            "collateral_count",
            "collateral_symbols",
            "collateral_truncated",
            "justified_count",
            "justified_symbols",
            "justified_truncated",
            "required_headers",
        ]
    );
}

#[tokio::test]
async fn test_justification_route_impact_is_unbounded_full_lists() {
    let app = build_router(justification_state());
    let (_status, _headers, body) =
        common::request_route(app, "/api/justification?file=1").await;

    // huge.h entry is index 0. Its subtree under drop (dropped_directly) is the
    // whole {d0,d1,d2,d3,mid,d4,d5,d6,d7,d8} = 10 headers. The sibling edge
    // root.cc -> d3.h keeps d3 in closure_after, so dropped_from_root drops
    // d3 (9 headers) while still_reachable keeps only d3.h.
    let impact = &body["entries"][0]["impact"];

    let basenames = |arr: &serde_json::Value| -> Vec<String> {
        arr.as_array()
            .expect("impact list")
            .iter()
            .map(|h| h["path"].as_str().expect("path").to_string())
            .collect()
    };

    // FULL lists, not capped at 8 (E-I4): 10 dropped_directly survive intact.
    assert_eq!(basenames(&impact["dropped_directly"]).len(), 10);
    assert_eq!(basenames(&impact["dropped_directly"])[0], "d0.h");
    assert_eq!(basenames(&impact["dropped_from_root"]).len(), 9);
    assert_eq!(basenames(&impact["still_reachable"]), vec!["d3.h"]);

    let dfr = basenames(&impact["dropped_from_root"]);
    assert!(
        !dfr.contains(&"d3.h".to_string()),
        "d3.h survives via the sibling root.cc -> d3.h edge: {dfr:?}"
    );
    assert_eq!(
        impact["dropped_from_root"].as_array().unwrap().len(),
        9,
        "route is NOT capped at 8 (contrast the payload summary)"
    );

    // closure_after = closure_before - dropped_from_root; seed-excluded.
    let before = impact["root_closure_before"].as_u64().unwrap() as usize;
    let after = impact["root_closure_after"].as_u64().unwrap() as usize;
    assert_eq!(
        after,
        before - impact["dropped_from_root"].as_array().unwrap().len()
    );
    assert_eq!(impact["root_file_id"].as_u64().unwrap(), 1);
    assert_eq!(impact["source_file_id"].as_u64().unwrap(), 3);

    // The route impact object never carries the payload-only summary keys.
    assert!(impact.get("dropped_sample").is_none());
    assert!(impact.get("dropped_sample_truncated").is_none());
    assert!(impact.get("dropped_from_root_count").is_none());
}

#[tokio::test]
async fn test_justification_route_non_numeric_file_returns_empty_detail() {
    let app = build_router(justification_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/justification?file=abc").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["target_id"], "abc", "the raw file value is echoed back");
    assert_eq!(body["total_justified"], 0);
    assert_eq!(body["total_collateral"], 0);
    assert_eq!(body["entries"], serde_json::json!([]));
}

#[tokio::test]
async fn test_justification_route_unknown_file_returns_empty_detail() {
    let app = build_router(justification_state());
    let (status, _headers, body) =
        common::request_route(app, "/api/justification?file=999").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["target_id"], "999");
    assert_eq!(body["total_justified"], 0);
    assert_eq!(body["total_collateral"], 0);
    assert_eq!(body["entries"], serde_json::json!([]));
}

#[tokio::test]
async fn test_justification_route_requires_file_param() {
    let app = build_router(justification_state());
    let (status, _headers, _body) = common::request_route(app, "/api/justification").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
