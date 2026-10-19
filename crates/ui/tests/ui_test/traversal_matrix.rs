//! UTA2-R1 (audit row 5): full coverage matrix for `/api/graph/traversal`.
//! The endpoint had ZERO tests before this hand; the matrix pins every payload
//! field of the necessary/actual/collateral subtrees plus every status arm,
//! and carries the red pins for UTB-1 (orphan edges) and UTB-2 (HashSet-order
//! nondeterminism). Journal: tasks/uta2-r1-journal.md.

use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_ui::{build_router, UiState};
use axum::http::StatusCode;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;

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

/// Matrix fixture (journal-documented):
/// root.cc(1) -> dep.h(2), tail.cc(5), spare.h(50);
/// dep.h(2) -> req.h(30), aux.h(4); aux.h(4) -> tail.cc(5), ghost(44).
/// FileId 44 is an UNREGISTERED closure member (edge destination absent from
/// `file_paths`) - the ghost that makes the UTB-1 orphan-edge red reachable
/// (see journal: strict_closure seed-exclusion means the dep itself can
/// never be the orphan; the nodes filter_map vs edges closure-set mismatch
/// is the real mechanism).
/// `entry` (file 1) justifies `used_sym` (dep.h) which uses `helper` (req.h),
/// so the root->dep edge carries exactly one justified symbol and req.h is
/// the necessary header. aux.h's `drag` and tail.cc's `tail_sym` are the
/// collateral; tail.cc stays reachable for the root via its own direct edge
/// (decoy); spare.h is a direct include with NO justified symbols (the
/// reachable empty-seed early return).
fn matrix_state() -> Arc<UiState> {
    let mut graph = IncludeGraph::new();
    for (id, path) in [
        (1u64, "root.cc"),
        (2, "dep.h"),
        (4, "aux.h"),
        (5, "tail.cc"),
        (30, "req.h"),
        (50, "spare.h"),
    ] {
        graph
            .file_paths
            .insert(ariadne_repo::FileId(id), std::path::PathBuf::from(path));
    }
    for (source, destination) in [(1, 2), (1, 5), (1, 50), (2, 30), (2, 4), (4, 5), (4, 44)] {
        graph.edges.push(include_edge(source, destination));
    }

    let mut symbols = BTreeMap::new();
    let mut file_symbols: BTreeMap<ariadne_repo::FileId, Vec<ariadne_index::SymbolId>> =
        BTreeMap::new();
    let mut add_sym = |id: &str, file: u64| {
        let record = symbol_record(id, file);
        symbols.insert(record.id.clone(), record);
        file_symbols
            .entry(ariadne_repo::FileId(file))
            .or_default()
            .push(ariadne_index::SymbolId(id.to_string()));
    };
    add_sym("entry", 1);
    add_sym("used_sym", 2);
    add_sym("drag", 4);
    add_sym("tail_sym", 5);
    add_sym("helper", 30);
    add_sym("unused", 50);

    let model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols,
        edges: vec![semantic_edge("entry", "used_sym"), semantic_edge("used_sym", "helper")],
        tu_usage: BTreeMap::new(),
        file_symbols,
        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    Arc::new(UiState::new_loaded(
        std::path::PathBuf::from("."),
        graph,
        model,
        CallGraph::default(),
        common::empty_export_payload(),
    ))
}

fn node_ids(subtree: &Value) -> Vec<String> {
    subtree["nodes"]
        .as_array()
        .expect("subtree nodes must be an array")
        .iter()
        .map(|node| node["id"].as_str().expect("string id").to_string())
        .collect()
}

fn sorted_node_ids(subtree: &Value) -> Vec<String> {
    let mut ids = node_ids(subtree);
    ids.sort();
    ids
}

fn edge_pairs(subtree: &Value) -> Vec<String> {
    subtree["edges"]
        .as_array()
        .expect("subtree edges must be an array")
        .iter()
        .map(|edge| {
            format!(
                "{}->{}",
                edge["source"].as_str().expect("string source"),
                edge["target"].as_str().expect("string target")
            )
        })
        .collect()
}

fn node_by_id<'a>(subtree: &'a Value, id: &str) -> &'a Value {
    subtree["nodes"]
        .as_array()
        .expect("subtree nodes")
        .iter()
        .find(|node| node["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("node {id} missing from subtree: {subtree}"))
}

async fn traversal_get(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let (status, _headers, body) = common::request_route(app, uri).await;
    (status, body)
}

// ---- Status arms ----------------------------------------------------------

#[tokio::test]
async fn test_traversal_matrix_400_on_non_numeric_seeds() {
    let app = build_router(matrix_state());
    for uri in [
        "/api/graph/traversal?root=abc&dep=2",
        "/api/graph/traversal?root=1&dep=xy",
        "/api/graph/traversal?root=abc&dep=xy",
    ] {
        let (status, _body) = traversal_get(app.clone(), uri).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri} must be 400");
    }
}

#[tokio::test]
async fn test_traversal_matrix_404_on_unknown_files() {
    let app = build_router(matrix_state());
    for uri in [
        "/api/graph/traversal?root=999&dep=2",
        "/api/graph/traversal?root=1&dep=999",
    ] {
        let (status, _body) = traversal_get(app.clone(), uri).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri} must be 404");
    }
}

#[tokio::test]
async fn test_traversal_matrix_400_when_dep_not_direct_include() {
    let app = build_router(matrix_state());
    // aux.h(4) and req.h(30) exist but are not direct includes of root(1);
    // dep-not-direct must lose to 404 (registration checked first).
    for uri in [
        "/api/graph/traversal?root=1&dep=4",
        "/api/graph/traversal?root=1&dep=30",
        "/api/graph/traversal?root=2&dep=1",
    ] {
        let (status, _body) = traversal_get(app.clone(), uri).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri} must be 400");
    }
}

#[tokio::test]
async fn test_traversal_matrix_400_without_required_params() {
    let app = build_router(matrix_state());
    for uri in ["/api/graph/traversal", "/api/graph/traversal?root=1"] {
        let (status, _body) = traversal_get(app.clone(), uri).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri} must be 400");
    }
}

#[tokio::test]
async fn test_traversal_matrix_503_while_loading_after_seed_checks() {
    let app = build_router(common::loading_state());
    let (status, _body) =
        traversal_get(app.clone(), "/api/graph/traversal?root=1&dep=2").await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "numeric seeds pass resolution, then the missing analysis must 503"
    );
    let (status, _body) =
        traversal_get(app, "/api/graph/traversal?root=abc&dep=2").await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "seed resolution precedes the 503 readiness gate (check-order pin)"
    );
}

// ---- Full payload fields (item 5) -----------------------------------------

#[tokio::test]
async fn test_traversal_matrix_seeds_and_response_keys() {
    let app = build_router(matrix_state());
    let (status, body) =
        traversal_get(app, "/api/graph/traversal?root=1&dep=2").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["root_file_id"], "1");
    assert_eq!(body["dep_file_id"], "2");
    assert_eq!(
        body["analyzed_symbol_ids"],
        serde_json::json!(["used_sym"]),
        "empty symbols param must resolve the justified set of the root->dep edge"
    );
    let mut keys: Vec<String> = body
        .as_object()
        .expect("response object")
        .keys()
        .cloned()
        .collect();
    keys.sort();
    // Re-pin (successor session, journal: tasks/uta2-r1-journal.md): the
    // authored literal was NOT in ascending order - "actual_subtree" sorts
    // before "analyzed_symbol_ids" ("ac" < "an") - so the assertion could
    // never match its own `keys.sort()`. Same key set, honest order now.
    assert_eq!(
        keys,
        vec![
            "actual_subtree",
            "analyzed_symbol_ids",
            "collateral_delta",
            "dep_file_id",
            "necessary_subtree",
            "root_file_id",
            "summary"
        ]
    );
}

#[tokio::test]
async fn test_traversal_matrix_necessary_subtree_fields() {
    let app = build_router(matrix_state());
    let (_status, body) =
        traversal_get(app, "/api/graph/traversal?root=1&dep=2").await;
    let necessary = &body["necessary_subtree"];
    assert_eq!(necessary["file_count"], 1, "only req.h is necessary");
    assert_eq!(necessary["total_symbols"], 1);
    assert_eq!(sorted_node_ids(necessary), vec!["30".to_string()]);
    let req = node_by_id(necessary, "30");
    assert_eq!(req["label"], "req.h");
    assert_eq!(req["path"], "req.h");
    assert_eq!(req["kind"], "HEADER", ".h classifies as HEADER");
    assert_eq!(req["is_necessary"], true);
    assert_eq!(req["is_collateral"], false);
    assert_eq!(req["is_decoy"], false);
    assert_eq!(req["symbol_count"], 1, "file_symbols count for req.h");
    assert_eq!(
        edge_pairs(necessary),
        Vec::<String>::new(),
        "req.h includes nothing inside the necessary set"
    );
}

#[tokio::test]
async fn test_traversal_matrix_actual_subtree_fields() {
    let app = build_router(matrix_state());
    let (_status, body) =
        traversal_get(app, "/api/graph/traversal?root=1&dep=2").await;
    let actual = &body["actual_subtree"];
    // Re-pin (successor session, journal: tasks/uta2-r1-journal.md): the
    // authored `sorted_node_ids` sorts lexicographically ("30" before "4"),
    // so it could never equal the numeric-order literal - the assertion
    // could not pass against any code. Post-UTB-2 the emission order is
    // pinned ascending by numeric FileId, so compare the raw order: same
    // set intent, strictly stronger.
    assert_eq!(
        node_ids(actual),
        vec!["4".to_string(), "5".to_string(), "30".to_string()],
        "ghost 44 is dropped from nodes (unregistered) and, per the UTB-1 fix, \
         never referenced by an edge either (see the orphan-edge test)"
    );
    assert_eq!(actual["total_symbols"], 3, "helper + drag + tail_sym");
    assert_eq!(
        actual["file_count"], 3,
        "quirk pin: closure len(4) - saturating_sub(1) even though the strict \
         closure already excludes the dep (journal: undercount owner decision)"
    );
    assert_eq!(node_by_id(actual, "5")["kind"], "SOURCE", ".cc classifies as SOURCE");
    assert_eq!(node_by_id(actual, "5")["label"], "tail.cc");
    assert_eq!(node_by_id(actual, "4")["symbol_count"], 1);
    assert_eq!(node_by_id(actual, "30")["is_necessary"], false);
    assert!(
        edge_pairs(actual).contains(&"4->5".to_string()),
        "aux.h -> tail.cc is inside the closure: {actual}"
    );
}

#[tokio::test]
async fn test_traversal_matrix_collateral_and_summary_fields() {
    let app = build_router(matrix_state());
    let (_status, body) =
        traversal_get(app, "/api/graph/traversal?root=1&dep=2").await;
    let collateral = &body["collateral_delta"];
    assert_eq!(collateral["file_count"], 2);
    assert_eq!(collateral["total_symbols"], 1, "drag; the ghost carries 0 symbols");
    assert_eq!(
        node_ids(collateral),
        vec!["4".to_string(), "44".to_string()],
        "impact rows are path-sorted: aux.h before the unregistered id's 'unknown' fallback"
    );
    let aux = node_by_id(collateral, "4");
    assert_eq!(aux["is_collateral"], true);
    assert_eq!(aux["is_necessary"], false);
    assert_eq!(aux["kind"], "HEADER");
    let ghost = node_by_id(collateral, "44");
    assert_eq!(ghost["path"], "unknown", "ImpactHeader fallback for unregistered ids");
    assert_eq!(ghost["symbol_count"], 0);
    assert_eq!(ghost["kind"], "SOURCE", "'unknown' is not a header suffix");
    assert_eq!(
        edge_pairs(collateral),
        vec!["4->44".to_string()],
        "header-based builder keeps the ghost edge (its node list carries the ghost)"
    );
    assert_eq!(collateral["edges"][0]["is_necessary"], true,
        "quirk pin: build_subtree_edges hardcodes is_necessary=true for BOTH callers");

    let summary = &body["summary"];
    assert_eq!(summary["necessary_files"], 1);
    assert_eq!(summary["necessary_symbols"], 1);
    assert_eq!(summary["actual_files"], 3);
    assert_eq!(summary["actual_symbols"], 3);
    assert_eq!(summary["collateral_files"], 2);
    assert_eq!(summary["collateral_symbols"], 1);
    assert_eq!(summary["decoy_files"], 1, "tail.cc stays reachable via the root's own edge");
    assert_eq!(summary["decoy_symbols"], 1);
}

#[tokio::test]
async fn test_traversal_matrix_empty_seed_early_return() {
    let app = build_router(matrix_state());
    let (status, body) =
        traversal_get(app.clone(), "/api/graph/traversal?root=1&dep=50").await;
    assert_eq!(
        status, StatusCode::OK,
        "spare.h is a direct include with a collateral-only entry: the empty \
         symbol seed is the reachable early-return arm (the analyze Err arm is \
         unreachable over HTTP - journal)"
    );
    assert_eq!(body["root_file_id"], "1");
    assert_eq!(body["dep_file_id"], "50");
    assert_eq!(body["analyzed_symbol_ids"], serde_json::json!([]));
    for key in ["necessary_subtree", "actual_subtree", "collateral_delta"] {
        assert_eq!(body[key]["nodes"], serde_json::json!([]), "{key} nodes");
        assert_eq!(body[key]["edges"], serde_json::json!([]), "{key} edges");
        assert_eq!(body[key]["total_symbols"], 0, "{key} total_symbols");
        assert_eq!(body[key]["file_count"], 0, "{key} file_count");
    }
    assert_eq!(body["summary"]["necessary_files"], 0);
    assert_eq!(body["summary"]["decoy_files"], 0);

    // A symbols param of only empty splits behaves like the unseeded edge.
    let (status, body) =
        traversal_get(app, "/api/graph/traversal?root=1&dep=2&symbols=,").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["analyzed_symbol_ids"],
        serde_json::json!([]),
        "empty splits are filtered, so the seed is empty"
    );
    assert_eq!(body["necessary_subtree"]["file_count"], 0);
}

#[tokio::test]
async fn test_traversal_matrix_symbols_param_overrides_justified_set() {
    let app = build_router(matrix_state());
    let (_status, single) =
        traversal_get(app.clone(), "/api/graph/traversal?root=1&dep=2&symbols=drag").await;
    assert_eq!(
        single["analyzed_symbol_ids"],
        serde_json::json!(["drag"]),
        "the explicit param must win over the justified set (used_sym)"
    );
    // Re-pin (successor session, journal: tasks/uta2-r1-journal.md): the
    // authored "drag makes aux.h necessary" expectation is provably wrong
    // under the locked graph-crate A2 rule - required_headers_for_symbols
    // drops seed defining files WHOLESALE (load.rs:322-331; locked by
    // test_required_headers_seed_defining_file_dropped_wholesale and
    // test_required_headers_isolated_seed_returns_empty: an edgeless seed
    // contributes only itself, so R = {}). The override intent stands:
    // analyzed proves the param won, and necessary is honestly empty.
    assert_eq!(single["necessary_subtree"]["file_count"], 0);
    assert_eq!(
        node_ids(&single["necessary_subtree"]),
        Vec::<String>::new(),
        "drag's own defining file (aux.h) is its seed file - never a required header"
    );

    let (_status, pair) = traversal_get(
        app,
        "/api/graph/traversal?root=1&dep=2&symbols=used_sym,unused",
    )
    .await;
    assert_eq!(
        pair["analyzed_symbol_ids"],
        serde_json::json!(["used_sym", "unused"]),
        "comma-separated ids are echoed in order"
    );
    // Re-pin (same A2 rule as above): `unused` is defined in spare.h(50),
    // its own seed defining file, so spare.h is dropped wholesale; only
    // helper@req.h(30) - a non-seed closure symbol in a header - survives
    // into R. Authored "req.h + spare.h" expectation was wrong.
    assert_eq!(pair["necessary_subtree"]["file_count"], 1);
    assert_eq!(
        node_ids(&pair["necessary_subtree"]),
        vec!["30".to_string()],
        "only helper's defining header req.h is required; spare.h is a seed file"
    );
}

// ---- UTB-1: induced edges must live on the node set ------------------------

#[tokio::test]
async fn test_traversal_actual_subtree_has_no_orphan_edges() {
    // RED (UTB-1): every edge of actual_subtree must reference only ids that
    // are present in the subtree's OWN node list. The ghost FileId(44) is in
    // dep.h's closure but unregistered in file_paths: nodes drop it
    // (filter_map), edges must not reference it as source OR target.
    let app = build_router(matrix_state());
    let (_status, body) =
        traversal_get(app, "/api/graph/traversal?root=1&dep=2").await;
    let actual = &body["actual_subtree"];
    let node_set: Vec<String> = node_ids(actual);
    let edges = edge_pairs(actual);
    for edge in actual["edges"].as_array().expect("edges array") {
        let source = edge["source"].as_str().expect("source string");
        let target = edge["target"].as_str().expect("target string");
        assert!(
            node_set.iter().any(|id| id == source),
            "orphan edge source {source} is absent from the node list {node_set:?}: {edges:?}"
        );
        assert!(
            node_set.iter().any(|id| id == target),
            "orphan edge target {target} is absent from the node list {node_set:?}: {edges:?}"
        );
    }
    assert_eq!(
        edges,
        vec!["4->5".to_string()],
        "the induced edge set over nodes {{4,5,30}} is exactly aux.h -> tail.cc"
    );
    let dep_id = body["dep_file_id"].as_str().expect("dep id");
    let references_dep = edges.iter().any(|edge| edge.contains(dep_id));
    assert!(
        !references_dep,
        "the traversal dep must never appear in its own subtree edges: {edges:?}"
    );
}

// ---- UTB-2: emission must be deterministic and numerically ordered --------

/// Dep with a fat closure of eight registered files at scattered ids, so the
/// ascending-numeric assertion has 8! = 40320 possible pre-fix orders to
/// stumble into by HashSet luck.
fn fat_closure_state() -> Arc<UiState> {
    let mut graph = IncludeGraph::new();
    graph
        .file_paths
        .insert(ariadne_repo::FileId(1), std::path::PathBuf::from("rootD.cc"));
    graph
        .file_paths
        .insert(ariadne_repo::FileId(2), std::path::PathBuf::from("depD.h"));
    graph.edges.push(include_edge(1, 2));
    let mut symbols = BTreeMap::new();
    let mut file_symbols: BTreeMap<ariadne_repo::FileId, Vec<ariadne_index::SymbolId>> =
        BTreeMap::new();
    let mut add_sym = |id: &str, file: u64| {
        let record = symbol_record(id, file);
        symbols.insert(record.id.clone(), record);
        file_symbols
            .entry(ariadne_repo::FileId(file))
            .or_default()
            .push(ariadne_index::SymbolId(id.to_string()));
    };
    add_sym("entryD", 1);
    add_sym("usedD", 2);
    for id in [61u64, 16, 42, 8, 31, 23, 47, 3] {
        graph
            .file_paths
            .insert(ariadne_repo::FileId(id), std::path::PathBuf::from(format!("f{id:02}.h")));
        graph.edges.push(include_edge(2, id));
        add_sym(&format!("s{id:02}"), id);
    }
    let model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols,
        edges: vec![semantic_edge("entryD", "usedD")],
        tu_usage: BTreeMap::new(),
        file_symbols,
        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    Arc::new(UiState::new_loaded(
        std::path::PathBuf::from("."),
        graph,
        model,
        CallGraph::default(),
        common::empty_export_payload(),
    ))
}

#[tokio::test]
async fn test_traversal_actual_subtree_is_deterministic_and_numeric() {
    // RED (UTB-2): two in-process builds (two fresh HashSet seeds, the
    // accepted in-process determinism pattern of the export-stability lock)
    // must agree, AND the node order must be numeric FileId ascending, never
    // the lexicographic "16" < "3" nor HashSet-arbitrary.
    let app = build_router(fat_closure_state());
    let (_status, first) =
        traversal_get(app.clone(), "/api/graph/traversal?root=1&dep=2").await;
    let (_status, second) =
        traversal_get(app, "/api/graph/traversal?root=1&dep=2").await;
    assert_eq!(
        first, second,
        "two builds of the same payload must be byte-equal (UTB-2/UTA-B4 class)"
    );
    let actual = &first["actual_subtree"];
    let ids: Vec<u64> = node_ids(actual)
        .iter()
        .map(|id| id.parse::<u64>().expect("numeric id"))
        .collect();
    let mut ascending = ids.clone();
    ascending.sort_unstable();
    assert_eq!(ids, ascending, "nodes must ascend by numeric FileId: {ids:?}");
    assert_eq!(
        ids,
        vec![3u64, 8, 16, 23, 31, 42, 47, 61],
        "the full eight-leaf closure must be present in numeric order"
    );
    // Quirk pin (journal): file_count = closure_len - 1 even though the
    // strict closure already excludes the dep; eight registered leaves give
    // eight nodes but a seven count. Owner decision, not this hand's fix.
    assert_eq!(actual["nodes"].as_array().expect("nodes").len(), 8);
    assert_eq!(actual["file_count"], 7);
    assert_eq!(first["summary"]["actual_files"], 7);
    assert_eq!(actual["total_symbols"], 8);
    assert_eq!(
        edge_pairs(actual),
        Vec::<String>::new(),
        "star-shaped closure: no edges between the leaves"
    );
}
