//! Canonical emission order for the standalone export payload.
//!
//! The std `HashMap` seed is per-process, so a double-build equality test
//! inside ONE test process is vacuously green for any map-iteration order.
//! These tests therefore assert against explicitly constructed canonical
//! expectations (`path asc, file_id asc`, the repo's established tie-break);
//! the cross-process gate itself is the three-way SHA256 smoke on
//! `ariadne ui --export`.

use ariadne_graph::{IncludeEdge, IncludeGraph, IncludeKind, LanguageId, SourceLocation};
use ariadne_repo::FileId;
use ariadne_ui::{
    build_include_elements, build_include_elements_from_groups, GraphExportPayload,
    JustificationEntry, SummaryStats,
};
use std::collections::HashMap;

fn include_edge(source: u64, destination: u64) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: Some(FileId(destination)),
        context_name: String::new(),
        source_location: SourceLocation { line: 1, column: 1 },
        kind: IncludeKind::Local,
        raw_path: String::new(),
        resolved_path: None,
        language: LanguageId::Cpp,
    }
}

/// 10 files inserted in an order deliberately scrambled against the
/// canonical `(path asc, file_id asc)` emission order; the duplicate path on
/// ids 5/11 pins the file_id tie-break.
fn shuffled_file_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    for (id, path) in [
        (1u64, "delta.h"),
        (9, "charlie.h"),
        (3, "foxtrot.cc"),
        (11, "alpha.h"),
        (7, "echo.h"),
        (2, "bravo.h"),
        (6, "india.cc"),
        (5, "alpha.h"),
        (4, "hotel.h"),
        (8, "golf.h"),
    ] {
        graph
            .file_paths
            .insert(FileId(id), std::path::PathBuf::from(path));
    }
    graph.edges.push(include_edge(5, 2));
    graph.edges.push(include_edge(1, 9));
    graph.edges.push(include_edge(3, 6));
    graph
}

fn emitted_ids(elements: &[ariadne_ui::CytoscapeElement]) -> Vec<&str> {
    elements
        .iter()
        .map(|element| {
            element.data["id"]
                .as_str()
                .expect("element id must be a string")
        })
        .collect()
}

fn node_ids(elements: &[ariadne_ui::CytoscapeElement]) -> Vec<&str> {
    elements
        .iter()
        .filter(|element| {
            matches!(
                element.data["kind"].as_str(),
                Some("HEADER") | Some("SOURCE")
            )
        })
        .map(|element| {
            element.data["id"]
                .as_str()
                .expect("node id must be a string")
        })
        .collect()
}

#[test]
fn include_nodes_emit_in_canonical_path_then_file_id_order() {
    let graph = shuffled_file_graph();
    let elements = build_include_elements(
        &graph,
        &HashMap::new(),
        &[],
        &HashMap::new(),
        &HashMap::new(),
    );
    assert_eq!(
        node_ids(&elements),
        vec!["5", "11", "2", "9", "1", "7", "3", "8", "4", "6"],
        "include node array must follow canonical (path asc, file_id asc) order"
    );
}

#[test]
fn include_edges_keep_source_vec_order_behind_sorted_nodes() {
    let graph = shuffled_file_graph();
    let elements = build_include_elements(
        &graph,
        &HashMap::new(),
        &[],
        &HashMap::new(),
        &HashMap::new(),
    );
    assert_eq!(
        emitted_ids(&elements),
        vec![
            "5", "11", "2", "9", "1", "7", "3", "8", "4", "6", "5-2", "1-9", "3-6",
        ],
        "nodes sort canonically, edges keep the deterministic graph.edges order"
    );
}

/// The shuffled insertion of `shuffled_file_graph` bent into a cycle:
/// alpha.h(5) -> bravo.h(2) -> alpha.h(11) -> alpha.h(5) plus the chord
/// alpha.h(5) -> alpha.h(11), and the acyclic tail foxtrot.cc(3) ->
/// india.cc(6). The entangled group {2, 5, 11} sits on the duplicated path
/// (ids 5/11), so classification touches the tie-break members directly.
fn cyclic_shuffled_file_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    for (id, path) in [
        (1u64, "delta.h"),
        (9, "charlie.h"),
        (3, "foxtrot.cc"),
        (11, "alpha.h"),
        (7, "echo.h"),
        (2, "bravo.h"),
        (6, "india.cc"),
        (5, "alpha.h"),
        (4, "hotel.h"),
        (8, "golf.h"),
    ] {
        graph
            .file_paths
            .insert(FileId(id), std::path::PathBuf::from(path));
    }
    for (source, destination) in [(5, 2), (2, 11), (11, 5), (5, 11), (3, 6)] {
        graph.edges.push(include_edge(source, destination));
    }
    graph
}

#[test]
fn grouped_nodes_keep_the_canonical_emission_order_and_edge_order() {
    // M23.5 T3-A: group classification must not perturb the canonical
    // emission contract the two tests above lock for acyclic graphs: nodes
    // in (path asc, file_id asc) order, edges in graph.edges order, with the
    // grouped members sitting ON the path-duplicate tie-break (ids 5/11).
    let graph = cyclic_shuffled_file_graph();
    let groups = graph.detect_cycle_groups();
    assert_eq!(groups.len(), 1, "fixture must carry exactly one entangled group");
    assert_eq!(
        groups[0].member_files,
        vec![FileId(2), FileId(5), FileId(11)],
        "fixture sanity: the entangled group is exactly members 2, 5 and 11"
    );
    let elements = build_include_elements_from_groups(
        &graph,
        &HashMap::new(),
        &groups,
        &HashMap::new(),
        &HashMap::new(),
    );
    assert_eq!(
        emitted_ids(&elements),
        vec![
            "5", "11", "2", "9", "1", "7", "3", "8", "4", "6", "5-2", "2-11", "11-5", "5-11",
            "3-6",
        ],
        "grouped nodes keep canonical (path asc, file_id asc) order and edges keep graph.edges order"
    );
    for grouped_id in ["2", "5", "11"] {
        let node = node_element_by_id(&elements, grouped_id);
        assert_eq!(
            node.classes,
            Some("cycle".to_string()),
            "grouped node {grouped_id} keeps its cycle class without disturbing emission order"
        );
    }
    let chord = elements
        .iter()
        .find(|element| element.data["id"] == "5-11")
        .expect("chord edge element 5-11");
    assert_eq!(
        chord.classes,
        Some("cycle".to_string()),
        "IV-3 chord coverage holds under shuffled insertion too"
    );
}

fn node_element_by_id<'a>(
    elements: &'a [ariadne_ui::CytoscapeElement],
    id: &str,
) -> &'a ariadne_ui::CytoscapeElement {
    elements
        .iter()
        .find(|element| {
            element.data["id"] == id
                && matches!(element.data["kind"].as_str(), Some("HEADER") | Some("SOURCE"))
        })
        .unwrap_or_else(|| panic!("no node element with id {id}"))
}

fn entry_fixture() -> JustificationEntry {
    JustificationEntry {
        justified_symbols: vec!["s1".to_string()],
        collateral_symbols: vec![],
        justified_count: 1,
        collateral_count: 2,
        justified_truncated: false,
        collateral_truncated: false,
        required_headers: vec![],
        impact: None,
    }
}

fn payload_with_map_sections() -> GraphExportPayload {
    let mut file_collateral: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for outer in ["a2", "a10", "a3"] {
        let mut inner = HashMap::new();
        for suffix in ["2", "10", "3"] {
            inner.insert(format!("{outer}k{suffix}"), suffix.len());
        }
        file_collateral.insert(outer.to_string(), inner);
    }
    let mut file_justification: HashMap<String, HashMap<String, JustificationEntry>> =
        HashMap::new();
    for root in ["c2", "c10", "c3"] {
        let mut per_root = HashMap::new();
        per_root.insert("s7".to_string(), entry_fixture());
        file_justification.insert(root.to_string(), per_root);
    }
    GraphExportPayload {
        repo_name: "order-fixture".to_string(),
        include_graph: vec![],
        symbol_graph: vec![],
        call_graph: vec![],
        stats: SummaryStats {
            total_files: 0,
            total_symbols: 0,
            total_calls: 0,
            cycle_count: 0,
        },
        file_collateral,
        file_justification,
        extraction_proposals: vec![],
    }
}

fn key_position(json: &str, key: &str) -> usize {
    json.find(&format!("\"{key}\":"))
        .unwrap_or_else(|| panic!("key {key} missing from the serialized payload"))
}

fn assert_ascending(json: &str, keys: &[&str]) {
    let positions: Vec<usize> = keys.iter().map(|key| key_position(json, key)).collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "keys {keys:?} must serialize in ascending key order, byte offsets were {positions:?}"
    );
}

#[test]
fn export_map_sections_serialize_keys_in_sorted_order() {
    let json = serde_json::to_string(&payload_with_map_sections()).unwrap();
    assert_ascending(&json, &["a10", "a2", "a3"]);
    assert_ascending(&json, &["a10k10", "a10k2", "a10k3"]);
    assert_ascending(&json, &["a2k10", "a2k2", "a2k3"]);
    assert_ascending(&json, &["a3k10", "a3k2", "a3k3"]);
    assert_ascending(&json, &["c10", "c2", "c3"]);
}
