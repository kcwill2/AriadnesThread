use ariadne_graph::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_repo::FileId;
use ariadne_ui::{
    build_call_elements, build_include_elements, build_symbol_elements,
    CytoscapeElement, GraphExportPayload, SummaryStats,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_dto_builders() {
    let include_graph = IncludeGraph::new();
    let closures = HashMap::new();
    let cycle_groups = vec![];

    let elements = build_include_elements(
        &include_graph,
        &closures,
        &cycle_groups,
        &HashMap::new(),
        &HashMap::new(),
    );
    assert!(elements.is_empty());

    let reduced_model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols: std::collections::BTreeMap::new(),
        edges: vec![],
        tu_usage: std::collections::BTreeMap::new(),
        file_symbols: std::collections::BTreeMap::new(),
        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    let sym_elements = build_symbol_elements(&reduced_model, &HashMap::new());
    assert!(sym_elements.is_empty());

    let call_graph = CallGraph::default();
    let call_elements = build_call_elements(&call_graph);
    assert!(call_elements.is_empty());
}

#[test]
fn test_export_html() {
    let payload = GraphExportPayload {
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
    };
    let template = "<html><body>/*__ARIADNE_DATA__*/</body></html>";
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("out.html");

    ariadne_ui::export_standalone_html(&payload, template, &out_path).unwrap();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("window.__ARIADNE_DATA__"));
}

#[test]
fn test_export_html_empty_template_uses_index() {
    let payload = GraphExportPayload {
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
    };
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("out2.html");

    // Empty template should trigger fallback to INDEX_HTML
    ariadne_ui::export_standalone_html(&payload, "", &out_path).unwrap();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("window.__ARIADNE_DATA__"));
    assert!(content.contains("Include Graph")); // Check if it used the INDEX_HTML
}

#[test]
fn test_build_symbol_elements_filters_unindexed() {
    let mut symbols = std::collections::BTreeMap::new();
    let sym_a = ariadne_index::SymbolId("A".to_string());
    let sym_b = ariadne_index::SymbolId("B".to_string());
    let sym_char = ariadne_index::SymbolId("char".to_string());

    symbols.insert(
        sym_a.clone(),
        ariadne_graph::ReducedSymbolRecord {
            id: sym_a.clone(),
            name: "A".to_string(),
            kind: ariadne_index::SymbolKind::Class,
            defining_file: None,
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        },
    );
    symbols.insert(
        sym_b.clone(),
        ariadne_graph::ReducedSymbolRecord {
            id: sym_b.clone(),
            name: "B".to_string(),
            kind: ariadne_index::SymbolKind::Class,
            defining_file: None,
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        },
    );

    let edges = vec![
        ariadne_graph::ReducedSemanticEdge {
            src_symbol: sym_a.clone(),
            dst_symbol: sym_b.clone(),
            kind: ariadne_index::DependencyKind::Complete,
        },
        ariadne_graph::ReducedSemanticEdge {
            src_symbol: sym_a.clone(),
            dst_symbol: sym_char.clone(),
            kind: ariadne_index::DependencyKind::Complete,
        },
    ];

    let model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols,
        edges,
        tu_usage: std::collections::BTreeMap::new(),
        file_symbols: std::collections::BTreeMap::new(),
        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };

    let elements = build_symbol_elements(&model, &HashMap::new());

    // Nodes for A and B = 2 elements.
    // Edge A->B = 1 element.
    // Total should be 3.
    assert_eq!(elements.len(), 3, "Expected 2 nodes and 1 valid edge");

    // A's out_degree should be 1
    let node_a = elements.iter().find(|e| e.data["id"] == "A").unwrap();
    assert_eq!(node_a.data["out_degree"], 1, "A should have out_degree 1");
}

#[test]
fn test_file_collateral_dto_serialization() {
    let mut file_collateral = HashMap::new();
    let mut collat = HashMap::new();
    collat.insert("2".to_string(), 5);
    file_collateral.insert("1".to_string(), collat);

    let payload = GraphExportPayload {
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
        file_collateral,
        file_justification: HashMap::new(),
        extraction_proposals: vec![],
    };

    let serialized = serde_json::to_string(&payload).unwrap();
    assert!(serialized.contains(r#""file_collateral":{"1":{"2":5}}"#));

    let deserialized: GraphExportPayload = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
        deserialized.file_collateral.get("1").unwrap().get("2"),
        Some(&5)
    );
}

fn d2_sample_proposal() -> ariadne_ui::ExtractionProposalDto {
    ariadne_ui::ExtractionProposalDto {
        plan_id: "D2PLAN-42".to_string(),
        target_file: "graph.h".to_string(),
        target_file_id: "1".to_string(),
        problem: "84 collateral symbols across consumers".to_string(),
        collateral_load_before: 84,
        collateral_load_after: 12,
        symbol_names: vec!["Thing".to_string(), "ThingBuilder".to_string()],
        total_symbols: 2,
        required_headers: vec!["a.h".to_string(), "b.h".to_string()],
        consumer_tu_count: 7,
        risk: "Conservative".to_string(),
        target_files: vec![],
        target_files_truncated: false,
    }
}

fn payload_with_proposals(proposals: Vec<ariadne_ui::ExtractionProposalDto>) -> GraphExportPayload {
    GraphExportPayload {
        repo_name: "d2-fixture".to_string(),
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
        extraction_proposals: proposals,
    }
}

fn export_data_blob(content: &str) -> String {
    let start = content
        .find("window.__ARIADNE_DATA__ = ")
        .expect("exported data blob");
    let end = content[start..]
        .find("</script>")
        .expect("data blob closes its inline script");
    let mut blob = content[start..start + end].trim().to_string();
    while blob.ends_with(';') {
        blob.pop();
    }
    blob
}

#[test]
fn test_standalone_export_embeds_extraction_proposals() {
    // I5 both-surface lock: report.html and the live UI share one data path.
    // The standalone export (empty template => real index + inlined assets) must
    // embed the proposal DATA and ship the card RENDERER in the same file.
    let payload = payload_with_proposals(vec![d2_sample_proposal()]);
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("d2-report.html");
    ariadne_ui::export_standalone_html(&payload, "", &out_path).unwrap();
    let content = std::fs::read_to_string(&out_path).unwrap();

    let blob = export_data_blob(&content);
    assert!(
        blob.contains("D2PLAN-42"),
        "proposal data must be embedded in the __ARIADNE_DATA__ blob"
    );
    assert!(
        blob.contains("\"extraction_proposals\""),
        "non-empty proposals must appear in the blob"
    );
    assert!(
        content.contains("function renderExtractionProposalCard("),
        "the standalone report must inline the shared proposal card renderer"
    );

    // Empty payload: no spurious proposals in the data blob (I4 / skip_serializing_if).
    let empty_path = temp_dir.path().join("d2-empty.html");
    ariadne_ui::export_standalone_html(&payload_with_proposals(vec![]), "", &empty_path).unwrap();
    let empty_content = std::fs::read_to_string(&empty_path).unwrap();
    let empty_blob = export_data_blob(&empty_content);
    assert!(
        !empty_blob.contains("extraction_proposals"),
        "empty proposals must be absent from the exported data blob (I4)"
    );
    assert!(!empty_blob.contains("D2PLAN-42"));
}

fn impact_fixture_entry(
    impact: Option<ariadne_ui::ImpactSummary>,
) -> ariadne_ui::JustificationEntry {
    ariadne_ui::JustificationEntry {
        justified_symbols: vec!["Parser".to_string()],
        collateral_symbols: vec![],
        justified_count: 1,
        collateral_count: 0,
        justified_truncated: false,
        collateral_truncated: false,
        required_headers: vec!["core.h".to_string()],
        impact,
    }
}

fn payload_with_impact(impact: Option<ariadne_ui::ImpactSummary>) -> GraphExportPayload {
    let mut entries = HashMap::new();
    entries.insert("5".to_string(), impact_fixture_entry(impact));
    let mut file_justification = HashMap::new();
    file_justification.insert("3".to_string(), entries);
    GraphExportPayload {
        repo_name: "impact-fixture".to_string(),
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
        file_justification,
        extraction_proposals: vec![],
    }
}

fn sample_impact_summary() -> ariadne_ui::ImpactSummary {
    ariadne_ui::ImpactSummary {
        dropped_from_root_count: 7,
        still_reachable_count: 2,
        dropped_sample: vec!["vendored_marker.h".to_string()],
        dropped_sample_truncated: false,
        root_closure_before: 31,
        root_closure_after: 24,
        symbols_no_longer_parsed: 123,
    }
}

#[test]
fn test_standalone_export_embeds_impact_summaries() {
    // R1 chain lock: an impact summary on a file_justification entry must
    // survive dto serialization all the way into the standalone export's
    // __ARIADNE_DATA__ blob, and the same file must ship the preview.js
    // consumer that reads it (resolvePreviewImpact reads entry.impact).
    let payload = payload_with_impact(Some(sample_impact_summary()));
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("impact-report.html");
    ariadne_ui::export_standalone_html(&payload, "", &out_path).unwrap();
    let content = std::fs::read_to_string(&out_path).unwrap();
    let blob = export_data_blob(&content);

    for key in [
        "\"impact\"",
        "\"dropped_from_root_count\"",
        "\"dropped_sample\"",
        "\"symbols_no_longer_parsed\"",
    ] {
        assert!(
            blob.contains(key),
            "impact key {key} must be embedded in the __ARIADNE_DATA__ blob"
        );
    }
    assert!(blob.contains("\"dropped_from_root_count\":7"));
    assert!(blob.contains("\"symbols_no_longer_parsed\":123"));
    assert!(
        blob.contains("vendored_marker.h"),
        "the distinctive dropped_sample marker must survive serialization"
    );
    assert!(
        content.contains("resolvePreviewImpact"),
        "the standalone report must inline the preview consumer for impact data"
    );

    // Negative half: impact: None must skip the key entirely (E-I4 serde lock),
    // while the entry itself still ships.
    let none_path = temp_dir.path().join("impact-none.html");
    ariadne_ui::export_standalone_html(&payload_with_impact(None), "", &none_path).unwrap();
    let none_content = std::fs::read_to_string(&none_path).unwrap();
    let none_blob = export_data_blob(&none_content);
    assert!(
        none_blob.contains("\"file_justification\""),
        "the entry ships even without an impact summary"
    );
    assert!(
        !none_blob.contains("\"impact\""),
        "impact: None must be skipped from the exported blob"
    );
    assert!(!none_blob.contains("vendored_marker.h"));
}

// ---------------------------------------------------------------------------
// M23.5 T3-A -- group-based cycle classification (plan
// 1790821308681-cycle-substrate-scc-m23_5 sec1.3, IV-2/IV-3, sec3.3 node
// data). Fixture: a.h(1) -> b.h(2) -> c.h(3) -> a.h(1) plus the chord
// a.h -> c.h plus the acyclic tail edge c.h -> d.h(4). The old DFS path
// representation reports the single path [1, 2, 3] for this graph; the SCC
// representation yields one group {1, 2, 3} and leaves d.h ungrouped.
// ---------------------------------------------------------------------------

fn chord_cycle_edge(source: u64, destination: u64) -> ariadne_graph::IncludeEdge {
    ariadne_graph::IncludeEdge {
        source: FileId(source),
        destination: Some(FileId(destination)),
        context_name: String::new(),
        source_location: ariadne_graph::SourceLocation { line: 1, column: 1 },
        kind: ariadne_graph::IncludeKind::Local,
        raw_path: String::new(),
        resolved_path: None,
        language: ariadne_graph::LanguageId::Cpp,
    }
}

fn chord_cycle_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    for (id, path) in [(1u64, "a.h"), (2, "b.h"), (3, "c.h"), (4, "d.h")] {
        graph
            .file_paths
            .insert(FileId(id), std::path::PathBuf::from(path));
    }
    for (source, destination) in [(1, 2), (2, 3), (3, 1), (1, 3), (3, 4)] {
        graph.edges.push(chord_cycle_edge(source, destination));
    }
    graph
}

fn node_element<'a>(elements: &'a [CytoscapeElement], id: &str) -> &'a CytoscapeElement {
    elements
        .iter()
        .find(|element| {
            element.data["id"] == id
                && matches!(element.data["kind"].as_str(), Some("HEADER") | Some("SOURCE"))
        })
        .unwrap_or_else(|| panic!("no node element with id {id} among {} elements", elements.len()))
}

fn edge_element<'a>(
    elements: &'a [CytoscapeElement],
    source: u64,
    destination: u64,
) -> &'a CytoscapeElement {
    elements
        .iter()
        .find(|element| {
            element.data["source"] == source.to_string()
                && element.data["target"] == destination.to_string()
        })
        .unwrap_or_else(|| {
            panic!("no edge element from {source} to {destination} among {} elements", elements.len())
        })
}

/// The group of `member` as the real producer (phase 2 `detect_cycle_groups`)
/// sees it on the chord fixture: exactly one entangled group.
fn single_group_containing(graph: &IncludeGraph, member: FileId) -> ariadne_graph::CycleGroup {
    let groups = graph.detect_cycle_groups();
    assert_eq!(groups.len(), 1, "chord fixture must yield exactly one cycle group");
    assert!(
        groups[0].member_files.contains(&member),
        "the group {:?} must contain {:?}",
        groups[0].member_files,
        member
    );
    groups[0].clone()
}

#[test]
fn intra_group_non_tree_edge_carries_the_cycle_class() {
    // R1 (IV-3, plan sec1.1 item 1): the chord a.h -> c.h sits inside the
    // entangled group {a.h, b.h, c.h}, but the old DFS path enumeration only
    // ever reports the walked tree path [a, b, c] plus its closing pair, so
    // the chord rendered UNSTYLED between two nodes that both carry the
    // `cycle` class. Under group membership the chord cannot be missed:
    // both endpoints are in the SAME group, so the edge must class `cycle`.
    // TDD trail: this Red failed against the old path-based entry point
    // (pair (1, 3) emitted is_cycle=false, journal checkpoint 1); the Green
    // step re-pointed it to the group-based builder per handoff sec4.
    let graph = chord_cycle_graph();
    let group = single_group_containing(&graph, FileId(1));
    let elements = build_include_elements(
        &graph,
        &HashMap::new(),
        &[group],
        &HashMap::new(),
        &HashMap::new(),
    );
    let chord = edge_element(&elements, 1, 3);
    assert_eq!(
        chord.data["is_cycle"],
        serde_json::json!(true),
        "intra-group chord edge (a.h -> c.h), pair (1, 3), must carry is_cycle = true"
    );
    assert_eq!(
        chord.classes,
        Some("cycle".to_string()),
        "intra-group chord edge (a.h -> c.h), pair (1, 3), must carry the cycle class"
    );
}

#[test]
fn node_cycle_data_reports_group_size_and_witness_only_for_grouped_nodes() {
    // R2 (sec3.3): a grouped node gains `cycle_group_size` (its group's member
    // count) and `cycle_witness` (the group's representative-cycle path labels
    // in representative_cycle order, IV-5); a node in no group carries
    // NEITHER key -- the D27 OMITS rule, never null and never 0.
    // TDD trail: this Red failed against the old path-based entry point
    // (cycle_group_size key absent, journal checkpoint 1); the Green step
    // re-pointed it to the group-based builder per handoff sec4.
    let graph = chord_cycle_graph();
    let group = single_group_containing(&graph, FileId(2));
    let elements = build_include_elements(
        &graph,
        &HashMap::new(),
        &[group.clone()],
        &HashMap::new(),
        &HashMap::new(),
    );
    let node_b = node_element(&elements, "2");
    assert_eq!(
        node_b.data.get("cycle_group_size"),
        Some(&serde_json::json!(3)),
        "grouped node b.h (FileId 2) must expose cycle_group_size = 3; actual data map: {}",
        node_b.data
    );
    let expected_witness: Vec<String> = group
        .representative_cycle
        .iter()
        .map(|member| graph.file_paths[member].to_string_lossy().to_string())
        .collect();
    assert_eq!(
        node_b.data.get("cycle_witness"),
        Some(&serde_json::json!(expected_witness)),
        "cycle_witness on b.h must equal the group's representative_cycle path labels in witness order; actual data map: {}",
        node_b.data
    );
    let node_d = node_element(&elements, "4");
    assert!(
        node_d.data.get("cycle_group_size").is_none(),
        "ungrouped node d.h (FileId 4) must NOT carry cycle_group_size; actual data map: {}",
        node_d.data
    );
    assert!(
        node_d.data.get("cycle_witness").is_none(),
        "ungrouped node d.h (FileId 4) must NOT carry cycle_witness; actual data map: {}",
        node_d.data
    );
}

/// Two disjoint entangled groups plus a cross-group chord and a grouped ->
/// ungrouped tail: a.h(10) <-> b.h(11) <-> c.h(12), x.h(20) <-> y.h(21),
/// chord-like cross edge c.h -> x.h, tail y.h -> off.cc(30).
fn two_group_chord_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    for (id, path) in [
        (10u64, "a.h"),
        (11, "b.h"),
        (12, "c.h"),
        (20, "x.h"),
        (21, "y.h"),
        (30, "off.cc"),
    ] {
        graph
            .file_paths
            .insert(FileId(id), std::path::PathBuf::from(path));
    }
    for (source, destination) in [(10, 11), (11, 10), (11, 12), (12, 11), (12, 20), (20, 21), (21, 20), (21, 30)] {
        graph.edges.push(chord_cycle_edge(source, destination));
    }
    graph
}

#[test]
fn grouped_nodes_and_edges_are_classed_cycle_from_membership_not_from_reported_paths() {
    // IV-2 + IV-3 at the DTO layer. The groups below are hand-built on
    // purpose: c.h (12) is a member_files entry that NO witness path covers,
    // the exact shape under which the old representation could class a member
    // from its reported paths only. The builder must read member_files and
    // nothing else. Edge (12, 20) is the group-IDENTITY lock: both endpoints
    // sit in SOME group, but in DIFFERENT groups -- IV-3 (same-group) says
    // it stays unclassed; a co-membership bug would class it.
    let graph = two_group_chord_graph();
    let first_group = ariadne_graph::CycleGroup {
        member_files: vec![FileId(10), FileId(11), FileId(12)],
        representative_cycle: vec![FileId(10), FileId(11), FileId(10)],
        external_in_count: 0,
        external_out_count: 1,
    };
    let second_group = ariadne_graph::CycleGroup {
        member_files: vec![FileId(20), FileId(21)],
        representative_cycle: vec![FileId(20), FileId(21), FileId(20)],
        external_in_count: 1,
        external_out_count: 1,
    };
    assert!(
        !first_group.representative_cycle.contains(&FileId(12)),
        "fixture sanity: c.h must be grouped without appearing on any witness path"
    );
    let elements = build_include_elements(
        &graph,
        &HashMap::new(),
        &[first_group, second_group],
        &HashMap::new(),
        &HashMap::new(),
    );

    for grouped_id in ["10", "11", "12", "20", "21"] {
        let node = node_element(&elements, grouped_id);
        assert_eq!(
            node.data["is_cycle"],
            serde_json::json!(true),
            "member node {grouped_id} must be classed cycle from member_files membership (IV-2)"
        );
        assert_eq!(
            node.classes,
            Some("cycle".to_string()),
            "member node {grouped_id} must carry the cycle class"
        );
    }
    let ungrouped = node_element(&elements, "30");
    assert_eq!(
        ungrouped.data["is_cycle"],
        serde_json::json!(false),
        "ungrouped node 30 must not be classed cycle"
    );
    assert_eq!(ungrouped.classes, None, "ungrouped node 30 carries no class");

    for (source, destination) in [(10, 11), (11, 10), (11, 12), (12, 11), (20, 21), (21, 20)] {
        let same_group_edge = edge_element(&elements, source, destination);
        assert_eq!(
            same_group_edge.data["is_cycle"],
            serde_json::json!(true),
            "same-group edge ({source}, {destination}) must class cycle (IV-3), witness coverage aside"
        );
        assert_eq!(
            same_group_edge.classes,
            Some("cycle".to_string()),
            "same-group edge ({source}, {destination}) must carry the cycle class"
        );
    }

    let cross_group_edge = edge_element(&elements, 12, 20);
    assert_eq!(
        cross_group_edge.data["is_cycle"],
        serde_json::json!(false),
        "edge (12, 20) joins group 1 to group 2: both endpoints are grouped but NOT in the same group, so IV-3 identity requires it to stay unclassed"
    );
    assert_eq!(
        cross_group_edge.classes,
        None,
        "cross-group edge (12, 20) must not carry the cycle class"
    );
    let tail_edge = edge_element(&elements, 21, 30);
    assert_eq!(
        tail_edge.data["is_cycle"],
        serde_json::json!(false),
        "edge (21, 30) joins a grouped file to an ungrouped file and must stay unclassed"
    );
    assert_eq!(tail_edge.classes, None, "tail edge (21, 30) carries no class");

    assert_eq!(
        node_element(&elements, "12").data["cycle_group_size"],
        serde_json::json!(3),
        "c.h rides group 1's member count"
    );
    assert_eq!(
        node_element(&elements, "20").data["cycle_group_size"],
        serde_json::json!(2),
        "x.h rides group 2's member count"
    );
}
