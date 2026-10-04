//! Canonical emission order for the standalone export payload.
//!
//! The std `HashMap` seed is per-process, so a double-build equality test
//! inside ONE test process is vacuously green for any map-iteration order.
//! These tests therefore assert against explicitly constructed canonical
//! expectations (`path asc, file_id asc`, the repo's established tie-break);
//! the cross-process gate itself is the three-way SHA256 smoke on
//! `ariadne ui --export`.

use ariadne_graph::{
    IncludeClosure, IncludeEdge, IncludeGraph, IncludeKind, LanguageId, SourceLocation,
};
use ariadne_repo::FileId;
use ariadne_ui::{
    build_include_elements, GraphExportPayload, JustificationEntry,
    SummaryStats,
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
    let elements = build_include_elements(
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

/// Non-vacuity witness for the frozen-expectation lock below
/// (`acyclic_include_elements_match_the_frozen_acyclic_payload`): the SAME
/// serialization comparison must be able to FAIL once a real cycle currency
/// exists, so a byte equality over this builder's output is never
/// green-by-construction.
///
/// P2 has no Red of its own: this `assert_ne!` and P1's mutated `assert_eq!`
/// (the executed mutation trail, handoff sec4 step R2) are the SAME comparison
/// over the SAME fixture family with OPPOSITE expectations, so the recorded R2
/// run -- in which the group-based side differs from the legacy side -- is
/// simultaneously the executed proof that this comparison has teeth. This is
/// not a determinism test and not a re-test of IV-2/IV-3 or the OMITS rule
/// (`dto_builders.rs` owns those); it exists only to keep the frozen
/// expectation honest.
#[test]
fn the_parity_comparison_detects_a_cycle_classification_difference() {
    let graph = cyclic_shuffled_file_graph();
    let groups = graph.detect_cycle_groups();
    assert_eq!(
        groups.len(),
        1,
        "fixture sanity: the cyclic fixture must carry exactly one entangled group"
    );
    assert_eq!(
        groups[0].member_files,
        vec![FileId(2), FileId(5), FileId(11)],
        "fixture sanity: the entangled group is exactly members 2, 5 and 11"
    );
    let legacy_elements = build_include_elements(
        &graph,
        &HashMap::new(),
        &[],
        &HashMap::new(),
        &HashMap::new(),
    );
    let grouped_elements = build_include_elements(
        &graph,
        &HashMap::new(),
        &groups,
        &HashMap::new(),
        &HashMap::new(),
    );
    let legacy_json = serde_json::to_string(&legacy_elements).expect("legacy elements serialize");
    let grouped_json = serde_json::to_string(&grouped_elements).expect("grouped serializes");
    assert_ne!(
        grouped_json, legacy_json,
        "the byte comparison P1 relies on must be able to fail: with a real group \
         currency the group-based side must differ from the legacy side"
    );
}

/// The payload the surviving include-element builder emitted for the fixture
/// below at `b0a91d2`, captured once and frozen deliberately. Assembled in
/// chunks because a ~1.6 KB single literal would breach the 2000-character
/// source-line rule and hide its own tail from every reader.
const ACYCLIC_EMPTY_CURRENCY_PAYLOAD: &str = concat!(
    "[{\"data\":{\"closure\":2,\"id\":\"5\",\"in_degree\":0,\"is_cycle\":false,\"kind\":\"",
    "HEADER\",\"label\":\"alpha.h\",\"out_degree\":1,\"path\":\"alpha.h\"}},{\"data\":{\"",
    "closure\":0,\"id\":\"11\",\"in_degree\":0,\"is_cycle\":false,\"kind\":\"HEADER\",\"",
    "label\":\"alpha.h\",\"out_degree\":0,\"path\":\"alpha.h\"}},{\"data\":{\"closure\"",
    ":0,\"id\":\"2\",\"in_degree\":1,\"is_cycle\":false,\"kind\":\"HEADER\",\"label\":\"",
    "bravo.h\",\"out_degree\":0,\"path\":\"bravo.h\"}},{\"data\":{\"closure\":0,\"id\":\"",
    "9\",\"in_degree\":1,\"is_cycle\":false,\"kind\":\"HEADER\",\"label\":\"charlie.h\",\"",
    "out_degree\":0,\"path\":\"charlie.h\"}},{\"data\":{\"closure\":1,\"id\":\"1\",\"",
    "in_degree\":0,\"is_cycle\":false,\"kind\":\"HEADER\",\"label\":\"delta.h\",\"",
    "language\":\"cpp\",\"out_degree\":1,\"path\":\"delta.h\",\"unit_kind\":\"",
    "translation-unit\"}},{\"data\":{\"closure\":0,\"id\":\"7\",\"in_degree\":0,\"",
    "is_cycle\":false,\"kind\":\"HEADER\",\"label\":\"echo.h\",\"out_degree\":0,\"path\"",
    ":\"echo.h\"}},{\"data\":{\"closure\":0,\"id\":\"3\",\"in_degree\":0,\"is_cycle\"",
    ":false,\"kind\":\"SOURCE\",\"label\":\"foxtrot.cc\",\"language\":\"cpp\",\"",
    "out_degree\":1,\"path\":\"foxtrot.cc\"}},{\"data\":{\"closure\":0,\"id\":\"8\",\"",
    "in_degree\":0,\"is_cycle\":false,\"kind\":\"HEADER\",\"label\":\"golf.h\",\"",
    "out_degree\":0,\"path\":\"golf.h\"}},{\"data\":{\"closure\":0,\"id\":\"4\",\"",
    "in_degree\":0,\"is_cycle\":false,\"kind\":\"HEADER\",\"label\":\"hotel.h\",\"",
    "out_degree\":0,\"path\":\"hotel.h\"}},{\"data\":{\"closure\":0,\"id\":\"6\",\"",
    "in_degree\":1,\"is_cycle\":false,\"kind\":\"SOURCE\",\"label\":\"india.cc\",\"",
    "out_degree\":0,\"path\":\"india.cc\"}},{\"data\":{\"id\":\"5-2\",\"is_cycle\"",
    ":false,\"kind\":\"Local\",\"source\":\"5\",\"target\":\"2\"}},{\"data\":{\"id\":\"",
    "1-9\",\"is_cycle\":false,\"kind\":\"Local\",\"source\":\"1\",\"target\":\"9\"}},{\"",
    "data\":{\"id\":\"3-6\",\"is_cycle\":false,\"kind\":\"Local\",\"source\":\"3\",\"",
    "target\":\"6\"}}]",
);

/// Frozen-byte lock -- what remains of the unit T3-D2 parity lock after unit
/// T3-E1 deleted the path-based builder. What it locks: on a graph carrying NO
/// cycle currency, the ONE surviving builder
/// `build_include_elements` must emit exactly the recorded bytes.
///
/// Where the recorded bytes came from: they were captured from this builder's
/// own output at `b0a91d2`, where the committed parity lock of unit T3-D2
/// (`42bb44e`, this file's former
/// `acyclic_include_elements_are_byte_identical_under_both_cycle_currencies`)
/// asserted `grouped_json == legacy_json` on this exact input. So the frozen
/// bytes are the agreed output of BOTH builders, and that committed lock --
/// not the capture alone -- is what makes E1's deletion behaviour-preserving
/// on this input.
///
/// What this test does NOT lock: emission-order determinism. The builder runs
/// in ONE process over ONE graph, so any hash-iteration-order perturbation
/// moves this expectation along with the payload and the comparison cannot see
/// it (module doc at the top of this file). Determinism lives in
/// `graph_api.rs:741-789` in-process and in the CLI two-process export lock
/// `export_bytes_are_identical_across_two_separate_processes` in
/// `crates/cli/tests/payload_tail_bench.rs`.
///
/// The expectation is FROZEN deliberately: a diff here means emitted bytes
/// changed, which a human re-approves -- never the implementer by editing the
/// expectation to match the output.
#[test]
fn acyclic_include_elements_match_the_frozen_acyclic_payload() {
    // Step 1 -- fixture sanity, machine-checked rather than taken on the
    // handoff's word.
    let graph = shuffled_file_graph();
    assert!(
        graph.detect_cycle_groups().is_empty(),
        "the frozen-payload fixture must be acyclic, or &[] is not the pinned currency"
    );

    // Step 2 -- the inputs the frozen bytes were recorded against. Every map
    // is a NON-EMPTY strict subset of the fixture's ids, so the decorated
    // nodes (closure size, D27 `language`/`unit_kind`) and the undecorated
    // ones are BOTH inside the locked bytes.
    let mut closures = HashMap::new();
    closures.insert(
        FileId(5),
        IncludeClosure::new(FileId(5), vec![FileId(2)], vec![FileId(2), FileId(9)]),
    );
    closures.insert(
        FileId(1),
        IncludeClosure::new(FileId(1), vec![FileId(9)], vec![FileId(9)]),
    );
    let mut file_languages = HashMap::new();
    file_languages.insert(1, "cpp".to_string());
    file_languages.insert(3, "cpp".to_string());
    let mut file_unit_kinds = HashMap::new();
    file_unit_kinds.insert(1, "translation-unit".to_string());

    // Step 3 -- EXACTLY ONE builder call, with empty cycle currency. Two calls
    // of the surviving name on one input would assert a value equals itself;
    // the single-invocation rule is this lock's anti-tautology invariant.
    let elements = build_include_elements(
        &graph,
        &closures,
        &[],
        &file_languages,
        &file_unit_kinds,
    );

    // Step 4 -- non-vacuity BEFORE the byte lock: without this, "the payload
    // is empty, therefore equal" would satisfy the comparison.
    assert!(
        !elements.is_empty(),
        "the surviving builder must emit elements; it emitted {}",
        elements.len()
    );
    let node_elements: Vec<&ariadne_ui::CytoscapeElement> = elements
        .iter()
        .filter(|element| {
            matches!(
                element.data["kind"].as_str(),
                Some("HEADER") | Some("SOURCE")
            )
        })
        .collect();
    let nodes_with_closure = node_elements
        .iter()
        .filter(|element| element.data["closure"].as_u64().unwrap_or(0) > 0)
        .count();
    assert!(
        nodes_with_closure > 0,
        "at least one node must carry a non-zero closure size inside the compared bytes"
    );
    let nodes_with_language = node_elements
        .iter()
        .filter(|element| element.data.get("language").is_some())
        .count();
    assert!(
        nodes_with_language > 0,
        "at least one node must carry the D27 `language` key inside the compared bytes"
    );
    assert!(
        nodes_with_language < node_elements.len(),
        "at least one node must OMIT the D27 `language` key inside the compared bytes"
    );

    // Step 5 -- the byte lock. `serde_json::Map` is a BTreeMap in this
    // workspace, so the serialized string pins the emitted key/value
    // sequence, which a structural `PartialEq` (key/value sets) would not.
    let payload = serde_json::to_string(&elements).expect("elements serialize");
    assert_eq!(
        payload, ACYCLIC_EMPTY_CURRENCY_PAYLOAD,
        "frozen payload: on acyclic input with empty cycle currency the surviving \
         builder must emit the bytes recorded at b0a91d2"
    );
}
