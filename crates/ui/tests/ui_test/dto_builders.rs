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
