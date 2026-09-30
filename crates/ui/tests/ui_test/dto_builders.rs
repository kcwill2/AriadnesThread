use ariadne_graph::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_ui::{
    build_call_elements, build_include_elements, build_symbol_elements, GraphExportPayload,
    SummaryStats,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_dto_builders() {
    let include_graph = IncludeGraph::new();
    let closures = HashMap::new();
    let cycles = vec![];

    let elements = build_include_elements(&include_graph, &closures, &cycles);
    assert!(elements.is_empty());

    let reduced_model = ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols: std::collections::BTreeMap::new(),
        edges: vec![],
        tu_usage: std::collections::BTreeMap::new(),
        file_symbols: std::collections::BTreeMap::new(),
    };
    let sym_elements = build_symbol_elements(&reduced_model);
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
    };

    let elements = build_symbol_elements(&model);

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

fn impact_fixture_entry(impact: Option<ariadne_ui::ImpactSummary>) -> ariadne_ui::JustificationEntry {
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
