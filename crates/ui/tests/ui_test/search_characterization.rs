//! AR-10 characterization locks (P7): the /api/graph/search semantics are
//! pinned against a populated payload BEFORE the wildcard-regex hoist, so the
//! refactor is provably behavior-preserving. Locked semantics: case-insensitive
//! substring contains, anchored case-insensitive wildcard glob (`*`/`?`),
//! case-SENSITIVE unanchored `/regex/` literals matched against the raw
//! label/path, wildcard-branch precedence over the substring fallback, the
//! single-`/` length guard, include->symbol->call result ordering, node-only
//! filtering (edges with `source` never surface), and limit truncation.
//! Every expectation below was captured against the pre-change handler and
//! must hold byte-identical after the change.

use crate::common;
use ariadne_ui::{build_router, CytoscapeElement, GraphExportPayload};
use axum::http::StatusCode;
use serde_json::{json, Map, Value};

/// One graph node element; absent label/path are omitted entirely so the
/// missing-label fallback (empty-string label) is itself under lock.
fn node(id: &str, label: Option<&str>, path: Option<&str>, kind: &str) -> CytoscapeElement {
    let mut fields = Map::new();
    fields.insert("id".to_string(), json!(id));
    if let Some(value) = label {
        fields.insert("label".to_string(), json!(value));
    }
    if let Some(value) = path {
        fields.insert("path".to_string(), json!(value));
    }
    fields.insert("kind".to_string(), json!(kind));
    CytoscapeElement {
        data: Value::Object(fields),
        classes: None,
    }
}

/// One graph edge element (carries `source`): the handler must never scan it.
fn edge(id: &str, source: &str, target: &str, label: &str) -> CytoscapeElement {
    CytoscapeElement {
        data: json!({"id": id, "source": source, "target": target, "label": label}),
        classes: None,
    }
}

fn filler_nodes() -> Vec<CytoscapeElement> {
    (0..25)
        .map(|filler_index| {
            node(
                &format!("f{filler_index:02}"),
                Some(&format!("Filler{filler_index:02}")),
                Some(&format!("filler/{filler_index:02}.h")),
                "file",
            )
        })
        .collect()
}

/// Populated three-graph fixture with mixed-case labels/paths. Probe nodes:
/// 5 (literal `*` inside the label), 6 (regex-vs-glob precedence bait),
/// 7 (substring `a/` and `/a`), 8 (substring `/` and glob-`/*a*/` target),
/// 9 (no label, no `/` in path, empty kind), plus an edge and 25 fillers to
/// drive ordering and truncation.
fn search_app() -> axum::Router {
    let mut include_graph = vec![
        node("1", Some("Renderer"), Some("src/render/Renderer.h"), "file"),
        node("2", Some("render_engine"), Some("src/render_engine.cc"), "file"),
        node("3", Some("widget.cc"), Some("src/widget.cc"), "file"),
        node("4", Some("WidgetBackup"), Some("src/widget.cc.bak"), "file"),
        node("5", Some("xa*by"), Some("src/xa*by.h"), "file"),
        node("6", Some("xxabxx"), Some("src/xxabxx.h"), "file"),
        node("7", Some("DataApi"), Some("data/api.h"), "file"),
        node("8", Some("/api/cache/"), Some("/opt/a/api/"), "file"),
        node("9", None, Some("header_only.h"), ""),
        edge("e1", "1", "2", "RenderEdge"),
    ];
    include_graph.extend(filler_nodes());

    let mut payload: GraphExportPayload = common::empty_export_payload();
    payload.include_graph = include_graph;
    payload.symbol_graph = vec![
        node("s1", Some("RenderWidget"), Some("src/sym/RenderWidget.cc"), "class"),
        node("s2", Some("compute_TOTAL"), Some("src/sym/compute_total.cc"), "function"),
    ];
    payload.call_graph = vec![
        node("c1", Some("Main"), Some("src/call/Main.cc"), "function"),
        node("c2", Some("helper_fn"), Some("src/call/helper_fn.cc"), "function"),
    ];

    let state = common::ready_state(
        std::path::PathBuf::from("."),
        ariadne_graph::IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        payload,
    );
    build_router(state)
}

async fn search_items(app: axum::Router, uri: &str) -> Vec<Value> {
    let (status, _headers, body) = common::request_route(app, uri).await;
    assert_eq!(status, StatusCode::OK, "{uri} must answer 200");
    body.as_array()
        .unwrap_or_else(|| panic!("search must answer a JSON array, got {body}"))
        .clone()
}

async fn search_ids(app: axum::Router, uri: &str) -> Vec<String> {
    search_items(app, uri)
        .await
        .into_iter()
        .map(|item| {
            item["id"]
                .as_str()
                .expect("each search result must carry a string id")
                .to_string()
        })
        .collect()
}

async fn expect_ids(app: axum::Router, uri: &str, expected: &[&str]) {
    let ids = search_ids(app, uri).await;
    let expected: Vec<String> = expected.iter().map(|id| id.to_string()).collect();
    assert_eq!(ids, expected, "unexpected result set for {uri}");
}

#[tokio::test]
async fn test_substring_is_case_insensitive_over_label_and_path() {
    // Substring contains over lowercased label OR path; include precedes
    // symbol precedes call; the edge's "RenderEdge" label is never scanned.
    expect_ids(search_app(), "/api/graph/search?q=render", &["1", "2", "s1"]).await;
    expect_ids(search_app(), "/api/graph/search?q=RENDERER", &["1"]).await;
    expect_ids(search_app(), "/api/graph/search?q=renderedge", &[]).await;
    expect_ids(search_app(), "/api/graph/search?q=e1", &[]).await;
}

#[tokio::test]
async fn test_substring_dot_and_slash_are_literal() {
    // No wildcards in the query: `.` and `/` stay literal substrings.
    expect_ids(search_app(), "/api/graph/search?q=widget.cc", &["3", "4", "s1"]).await;
    expect_ids(search_app(), "/api/graph/search?q=a/", &["7", "8"]).await;
    expect_ids(search_app(), "/api/graph/search?q=/a&limit=50", &["7", "8"]).await;
}

#[tokio::test]
async fn test_wildcard_glob_is_anchored_and_case_insensitive() {
    // `*`/`?` compile an ANCHORED case-insensitive glob: prefix/suffix only.
    expect_ids(
        search_app(),
        "/api/graph/search?q=*.cc",
        &["2", "3", "s1", "s2", "c1", "c2"],
    )
    .await;
    expect_ids(search_app(), "/api/graph/search?q=widget?c?", &["3"]).await;
    expect_ids(search_app(), "/api/graph/search?q=RENDER_*", &["2"]).await;
    expect_ids(search_app(), "/api/graph/search?q=*engine", &["2"]).await;
    expect_ids(search_app(), "/api/graph/search?q=WIDGET*", &["3", "4"]).await;
}

#[tokio::test]
async fn test_wildcard_query_escapes_literal_metacharacters_in_target() {
    // Query metacharacters are escaped (`.` in the query is literal under
    // the glob), and a literal `*` stored in the label is just a character.
    expect_ids(search_app(), "/api/graph/search?q=xa*by", &["5"]).await;
    expect_ids(search_app(), "/api/graph/search?q=widget.cc.bak", &["4"]).await;
}

#[tokio::test]
async fn test_wildcard_branch_suppresses_substring_fallback() {
    // A wildcard query that compiles NEVER falls through to substring
    // contains: "xa*by" contains "a*b" as text, yet the anchored glob
    // `^a.*b$` must not match and substring must not rescue it. Likewise
    // `w?dget.cc*` (glob `^w.dget\.cc.*$`) only reaches label 3 via `?`.
    expect_ids(search_app(), "/api/graph/search?q=a*b", &[]).await;
    expect_ids(search_app(), "/api/graph/search?q=w?dget.cc*", &["3"]).await;
}

#[tokio::test]
async fn test_regex_literal_is_case_sensitive_and_unanchored() {
    // `/regex/` matches the RAW (unlowercased) label/path, unanchored.
    expect_ids(search_app(), "/api/graph/search?q=/Renderer/", &["1"]).await;
    expect_ids(search_app(), "/api/graph/search?q=/renderer/", &[]).await;
    expect_ids(search_app(), "/api/graph/search?q=/src/call/", &["c1", "c2"]).await;
    expect_ids(search_app(), "/api/graph/search?q=/SRC/CALL/", &[]).await;
    expect_ids(search_app(), "/api/graph/search?q=/Main.cc/", &["c1"]).await;
    expect_ids(search_app(), "/api/graph/search?q=/main.cc/", &[]).await;
    // `[` and `]` are illegal raw in an URI query: percent-encoded form.
    expect_ids(
        search_app(),
        "/api/graph/search?q=/%5B.%5Dcc$/",
        &["2", "3", "s1", "s2", "c1", "c2"],
    )
    .await;
}

#[tokio::test]
async fn test_invalid_regex_falls_back_to_wildcard_glob() {
    // "/*a*/" looks like a regex but `*a*` fails to compile, so the wildcard
    // glob wins: escape -> `^/.*a.*/$` anchored, case-insensitive.
    expect_ids(search_app(), "/api/graph/search?q=/*a*/", &["8"]).await;
}

#[tokio::test]
async fn test_slash_boundary_locks() {
    // "/" alone is NOT a regex (len > 1 guard): plain substring "/", so node
    // 9 (path without any slash) drops out. "//" IS an (empty) regex that
    // matches every node, node 9 included.
    let all_but_slashless = {
        let mut ids: Vec<String> = (1..=8).map(|index| index.to_string()).collect();
        ids.extend((0..25).map(|filler| format!("f{filler:02}")));
        ids.extend(["s1".to_string(), "s2".to_string(), "c1".into(), "c2".into()]);
        ids
    };
    let mut all_nodes = all_but_slashless.clone();
    all_nodes.insert(8, "9".to_string());

    let slash_ids = search_ids(search_app(), "/api/graph/search?q=/&limit=50").await;
    assert_eq!(slash_ids.len(), 37, "37 slash-bearing nodes expected");
    assert!(!slash_ids.iter().any(|id| id == "9"), "node 9 has no '/'");

    let double_slash_ids = search_ids(search_app(), "/api/graph/search?q=//&limit=50").await;
    let expected: Vec<String> = all_nodes;
    assert_eq!(double_slash_ids, expected, "// must match every node");
    let expected_prefix: Vec<String> = all_but_slashless.into_iter().take(37).collect();
    assert_eq!(&slash_ids[..], &expected_prefix[..]);
}

#[tokio::test]
async fn test_graph_type_selects_scanned_graphs() {
    expect_ids(
        search_app(),
        "/api/graph/search?q=render&graph_type=include",
        &["1", "2"],
    )
    .await;
    expect_ids(
        search_app(),
        "/api/graph/search?q=render&graph_type=symbol",
        &["s1"],
    )
    .await;
    expect_ids(
        search_app(),
        "/api/graph/search?q=main&graph_type=call",
        &["c1"],
    )
    .await;
    expect_ids(
        search_app(),
        "/api/graph/search?q=render&graph_type=whatever",
        &[],
    )
    .await;
}

#[tokio::test]
async fn test_limit_truncates_in_graph_order() {
    expect_ids(search_app(), "/api/graph/search?q=&limit=2", &["1", "2"]).await;
    expect_ids(search_app(), "/api/graph/search?q=&limit=0", &[]).await;
    // Node 9 (no label) still matches the empty query, so the filler run
    // starts one position later and slot 20 lands on f10.
    let default_ids = search_ids(search_app(), "/api/graph/search?q=").await;
    assert_eq!(default_ids.len(), 20, "default limit is 20");
    assert_eq!(default_ids[19], "f10", "truncation cuts after f10");
    expect_ids(
        search_app(),
        "/api/graph/search?q=*.cc&limit=3",
        &["2", "3", "s1"],
    )
    .await;
}

#[tokio::test]
async fn test_missing_label_node_matches_by_path_with_empty_fields() {
    // Node 9 has no label and no kind: it matches via path and the result
    // item carries empty strings for both, proving the unwrap_or_default
    // fallbacks stay intact.
    let items = search_items(search_app(), "/api/graph/search?q=header_only").await;
    assert_eq!(
        items,
        vec![json!({"id": "9", "label": "", "kind": "", "graph_type": "include"})]
    );
}
