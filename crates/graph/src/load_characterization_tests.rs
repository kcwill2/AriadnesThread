use super::*;
use crate::reduced::ReducedSymbolRecord;
use crate::types::{IncludeEdge, IncludeKind, LanguageId, SourceLocation};
use ariadne_index::types::{SymbolId, SymbolKind};
use ariadne_repo::FileId;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn setup_mock_data() -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(1), PathBuf::from("Thing.h"));
    ig.register_file(FileId(2), PathBuf::from("graphics.h"));
    ig.register_file(FileId(3), PathBuf::from("HugeThirdParty.h"));

    ig.add_edge(IncludeEdge {
        source: FileId(1),
        destination: Some(FileId(2)),
        context_name: "".to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "graphics.h".to_string(),
        resolved_path: Some(PathBuf::from("graphics.h")),
        language: LanguageId::cpp(),
    });
    ig.add_edge(IncludeEdge {
        source: FileId(2),
        destination: Some(FileId(3)),
        context_name: "".to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "HugeThirdParty.h".to_string(),
        resolved_path: Some(PathBuf::from("HugeThirdParty.h")),
        language: LanguageId::cpp(),
    });

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };

    let sym_thing = SymbolId::new("Thing");
    model.symbols.insert(
        sym_thing.clone(),
        ReducedSymbolRecord {
            id: sym_thing.clone(),
            name: "Thing".to_string(),
            kind: SymbolKind::Class,
            defining_file: Some(FileId(1)),
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        },
    );
    model
        .file_symbols
        .insert(FileId(1), vec![sym_thing.clone()]);

    let sym_huge = SymbolId::new("HugeThing");
    model.symbols.insert(
        sym_huge.clone(),
        ReducedSymbolRecord {
            id: sym_huge.clone(),
            name: "HugeThing".to_string(),
            kind: SymbolKind::Class,
            defining_file: Some(FileId(3)),
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        },
    );
    model.file_symbols.insert(FileId(3), vec![sym_huge.clone()]);

    (ig, model)
}

#[test]
fn test_analyze_symbol_load() {
    let (ig, model) = setup_mock_data();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let metric = analyzer
        .analyze_symbol_load("Thing")
        .expect("analyze_symbol_load failed");
    assert_eq!(metric.target_name, "Thing");
    assert_eq!(metric.semantic_load, 1);
    assert_eq!(metric.include_load, 2);
    assert_eq!(metric.collateral_load, 1);
    assert_eq!(metric.total_load, 2);
    assert_eq!(
        metric.largest_collateral_source.as_ref().unwrap().path,
        PathBuf::from("HugeThirdParty.h")
    );
}

#[test]
fn test_explain_symbol_load() {
    let (ig, model) = setup_mock_data();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let explanation = analyzer
        .explain_symbol_load("Thing", None)
        .expect("explain_symbol_load failed");
    assert_eq!(explanation.target_name, "Thing");
    assert_eq!(explanation.collateral_source, "HugeThirdParty.h");
    assert_eq!(
        explanation.path,
        vec!["Thing", "Thing.h", "graphics.h", "HugeThirdParty.h"]
    );

    let formatted = explanation.format_explanation();
    assert!(formatted.contains("HugeThirdParty.h is included by graphics.h."));
    assert!(formatted
        .contains("No semantic dependency from Thing reaches symbols owned by HugeThirdParty.h."));
}

#[test]
fn test_analyze_file_load() {
    let (ig, model) = setup_mock_data();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let metric = analyzer
        .analyze_file_load(std::path::Path::new("Thing.h"))
        .expect("analyze_file_load failed");
    assert_eq!(metric.target_name, "Thing.h");
    assert_eq!(metric.semantic_load, 1);
    assert_eq!(metric.include_load, 2);
    assert_eq!(metric.collateral_load, 1);
    assert_eq!(metric.total_load, 2);
    assert_eq!(
        metric.largest_collateral_source.as_ref().unwrap().path,
        PathBuf::from("HugeThirdParty.h")
    );

    // Error case
    assert!(analyzer
        .analyze_file_load(std::path::Path::new("NonExistent.h"))
        .is_err());
}

#[test]
fn test_explain_file_load() {
    let (ig, model) = setup_mock_data();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    // Automatic largest collateral source
    let explanation = analyzer
        .explain_file_load(std::path::Path::new("Thing.h"), None)
        .expect("explain_file_load failed");
    assert_eq!(explanation.target_name, "Thing.h");
    assert_eq!(explanation.collateral_source, "HugeThirdParty.h");
    assert_eq!(
        explanation.path,
        vec!["Thing.h", "graphics.h", "HugeThirdParty.h"]
    );

    // Explicit collateral source
    let explanation_explicit = analyzer
        .explain_file_load(std::path::Path::new("Thing.h"), Some("HugeThirdParty"))
        .expect("explain_file_load failed");
    assert_eq!(explanation_explicit.collateral_source, "HugeThirdParty.h");

    // Error cases
    assert!(analyzer
        .explain_file_load(std::path::Path::new("NonExistent.h"), None)
        .is_err());
    assert!(analyzer
        .explain_file_load(std::path::Path::new("HugeThirdParty.h"), None)
        .is_err());
}

#[test]
fn test_format_summary() {
    let (ig, model) = setup_mock_data();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    // With largest collateral source
    let metric = analyzer
        .analyze_file_load(std::path::Path::new("Thing.h"))
        .unwrap();
    let summary = metric.format_summary();
    assert!(summary.contains("Thing.h"));
    assert!(summary.contains("Semantic load:"));
    assert!(summary.contains("1 symbols"));
    assert!(summary.contains("Largest collateral source:"));
    assert!(summary.contains("HugeThirdParty.h"));

    // Without collateral source
    let metric_empty = analyzer
        .analyze_file_load(std::path::Path::new("HugeThirdParty.h"))
        .unwrap();
    let summary_empty = metric_empty.format_summary();
    assert!(summary_empty.contains("None"));
    assert!(summary_empty.contains("0 symbols"));
}

#[test]
fn test_explain_symbol_load_locks_complete_output() {
    let (ig, model) = setup_mock_data();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let explanation = analyzer
        .explain_symbol_load("Thing", None)
        .expect("explain_symbol_load failed");
    assert_eq!(explanation.target_name, "Thing");
    assert_eq!(explanation.target_file, PathBuf::from("Thing.h"));
    assert_eq!(
        explanation.path,
        vec!["Thing", "Thing.h", "graphics.h", "HugeThirdParty.h"]
    );
    assert_eq!(explanation.collateral_source, "HugeThirdParty.h");
    assert_eq!(explanation.collateral_symbols_count, 1);
    assert_eq!(
        explanation.reason,
        "HugeThirdParty.h is included by graphics.h.\nNo semantic dependency from Thing reaches symbols owned by HugeThirdParty.h."
    );

    let expected_format = concat!(
        "Thing\n",
        "  ↓\n",
        "Thing.h\n",
        "  ↓\n",
        "graphics.h\n",
        "  ↓\n",
        "HugeThirdParty.h\n",
        "\n",
        "Reason:\n",
        "  HugeThirdParty.h is included by graphics.h.\n",
        "  No semantic dependency from Thing reaches symbols owned by HugeThirdParty.h.\n",
    );
    assert_eq!(explanation.format_explanation(), expected_format);
}

#[test]
fn test_explain_file_load_locks_complete_output() {
    let (ig, model) = setup_mock_data();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let explanation = analyzer
        .explain_file_load(std::path::Path::new("Thing.h"), None)
        .expect("explain_file_load failed");
    assert_eq!(explanation.target_name, "Thing.h");
    assert_eq!(explanation.target_file, PathBuf::from("Thing.h"));
    assert_eq!(
        explanation.path,
        vec!["Thing.h", "graphics.h", "HugeThirdParty.h"]
    );
    assert_eq!(explanation.collateral_source, "HugeThirdParty.h");
    assert_eq!(explanation.collateral_symbols_count, 1);
    assert_eq!(
        explanation.reason,
        "HugeThirdParty.h is included by graphics.h.\nNo semantic dependency from Thing.h reaches symbols owned by HugeThirdParty.h."
    );

    let expected_format = concat!(
        "Thing.h\n",
        "  ↓\n",
        "graphics.h\n",
        "  ↓\n",
        "HugeThirdParty.h\n",
        "\n",
        "Reason:\n",
        "  HugeThirdParty.h is included by graphics.h.\n",
        "  No semantic dependency from Thing.h reaches symbols owned by HugeThirdParty.h.\n",
    );
    assert_eq!(explanation.format_explanation(), expected_format);
}

#[test]
fn test_explain_symbol_load_same_file_collateral_locks_single_step_path() {
    // A symbol whose unreached siblings live in its own defining file makes
    // the collateral source the target file itself, so the include path is
    // a single node (start == end) and the parent renders as that file.
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(10), PathBuf::from("Self.h"));
    ig.register_file(FileId(11), PathBuf::from("Dep.h"));
    ig.add_edge(IncludeEdge {
        source: FileId(10),
        destination: Some(FileId(11)),
        context_name: "".to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "Dep.h".to_string(),
        resolved_path: Some(PathBuf::from("Dep.h")),
        language: LanguageId::cpp(),
    });

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };

    let sym_main = SymbolId::new("Main");
    model.symbols.insert(
        sym_main.clone(),
        ReducedSymbolRecord {
            id: sym_main.clone(),
            name: "Main".to_string(),
            kind: SymbolKind::Function,
            defining_file: Some(FileId(10)),
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        },
    );
    let sym_sidecar = SymbolId::new("Sidecar");
    model.symbols.insert(
        sym_sidecar.clone(),
        ReducedSymbolRecord {
            id: sym_sidecar.clone(),
            name: "Sidecar".to_string(),
            kind: SymbolKind::Function,
            defining_file: Some(FileId(10)),
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        },
    );
    model
        .file_symbols
        .insert(FileId(10), vec![sym_main, sym_sidecar]);

    let analyzer = LoadAnalyzer::new(&ig, &model);

    let metric = analyzer
        .analyze_symbol_load("Main")
        .expect("analyze_symbol_load failed");
    assert_eq!(metric.semantic_load, 1);
    assert_eq!(metric.collateral_load, 1);
    assert_eq!(
        metric
            .largest_collateral_source
            .as_ref()
            .expect("same-file collateral source")
            .path,
        PathBuf::from("Self.h")
    );

    let explanation = analyzer
        .explain_symbol_load("Main", None)
        .expect("explain_symbol_load failed");
    assert_eq!(explanation.path, vec!["Main", "Self.h"]);
    assert_eq!(explanation.target_file, PathBuf::from("Self.h"));
    assert_eq!(explanation.collateral_source, "Self.h");
    assert_eq!(explanation.collateral_symbols_count, 1);
    assert_eq!(
        explanation.reason,
        "Self.h is included by Self.h.\nNo semantic dependency from Main reaches symbols owned by Self.h."
    );
}

#[test]
fn test_explain_unregistered_file_ids_render_unknown_placeholder() {
    // File ids referenced only by edges are absent from file_paths but the
    // include BFS still traverses them. Rendering falls back to the
    // "unknown" placeholder compute_load already uses for missing
    // collateral paths, and LoadExplanation.target_file uses the same
    // value when the target file has no registered path.
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(22), PathBuf::from("Root.h"));
    ig.add_edge(IncludeEdge {
        source: FileId(22),
        destination: Some(FileId(20)),
        context_name: "".to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "edge.h".to_string(),
        resolved_path: None,
        language: LanguageId::cpp(),
    });
    ig.add_edge(IncludeEdge {
        source: FileId(20),
        destination: Some(FileId(21)),
        context_name: "".to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "edge.h".to_string(),
        resolved_path: None,
        language: LanguageId::cpp(),
    });

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };

    let add_sym = |model: &mut ReducedSemanticModel, name: &str, file: FileId| {
        let id = SymbolId::new(name);
        model.symbols.insert(
            id.clone(),
            ReducedSymbolRecord {
                id: id.clone(),
                name: name.to_string(),
                kind: SymbolKind::Function,
                defining_file: Some(file),
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );
        id
    };
    let solo = add_sym(&mut model, "Solo", FileId(20));
    let bulk = add_sym(&mut model, "Bulk", FileId(21));
    let bulk2 = add_sym(&mut model, "Bulk2", FileId(21));
    let root = add_sym(&mut model, "Root", FileId(22));
    model.file_symbols.insert(FileId(20), vec![solo]);
    model.file_symbols.insert(FileId(21), vec![bulk, bulk2]);
    model.file_symbols.insert(FileId(22), vec![root]);

    let analyzer = LoadAnalyzer::new(&ig, &model);

    let metric = analyzer
        .analyze_symbol_load("Solo")
        .expect("analyze_symbol_load failed");
    assert_eq!(metric.target_file, None);

    let explanation = analyzer
        .explain_symbol_load("Solo", None)
        .expect("explain_symbol_load failed");
    assert_eq!(explanation.target_file, PathBuf::from("unknown"));
    assert_eq!(explanation.path, vec!["Solo", "unknown", "unknown"]);
    assert_eq!(explanation.collateral_source, "unknown");
    assert_eq!(
        explanation.reason,
        "unknown is included by unknown.\nNo semantic dependency from Solo reaches symbols owned by unknown."
    );

    let file_explanation = analyzer
        .explain_file_load(std::path::Path::new("Root.h"), None)
        .expect("explain_file_load failed");
    assert_eq!(file_explanation.target_file, PathBuf::from("Root.h"));
    assert_eq!(file_explanation.path, vec!["Root.h", "unknown", "unknown"]);
    assert_eq!(file_explanation.collateral_source, "unknown");
    assert_eq!(
        file_explanation.reason,
        "unknown is included by unknown.\nNo semantic dependency from Root.h reaches symbols owned by unknown."
    );
}

#[test]
fn test_explain_paths_without_file_name_render_full_path() {
    // A registered path can lack a final component (`..`); rendering falls
    // back to the full path string instead of panicking on file_name().
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(40), PathBuf::from("Host.h"));
    ig.register_file(FileId(41), PathBuf::from(".."));
    ig.add_edge(IncludeEdge {
        source: FileId(40),
        destination: Some(FileId(41)),
        context_name: "".to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "..".to_string(),
        resolved_path: None,
        language: LanguageId::cpp(),
    });

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };

    let host = SymbolId::new("Host");
    model.symbols.insert(
        host.clone(),
        ReducedSymbolRecord {
            id: host.clone(),
            name: "Host".to_string(),
            kind: SymbolKind::Function,
            defining_file: Some(FileId(40)),
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        },
    );
    let odd = SymbolId::new("Odd");
    model.symbols.insert(
        odd.clone(),
        ReducedSymbolRecord {
            id: odd.clone(),
            name: "Odd".to_string(),
            kind: SymbolKind::Function,
            defining_file: Some(FileId(41)),
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        },
    );
    model.file_symbols.insert(FileId(40), vec![host]);
    model.file_symbols.insert(FileId(41), vec![odd]);

    let analyzer = LoadAnalyzer::new(&ig, &model);

    let explanation = analyzer
        .explain_symbol_load("Host", None)
        .expect("explain_symbol_load failed");
    assert_eq!(explanation.path, vec!["Host", "Host.h", ".."]);
    assert_eq!(explanation.target_file, PathBuf::from("Host.h"));
    assert_eq!(explanation.collateral_source, "..");
    assert_eq!(
        explanation.reason,
        ".. is included by Host.h.\nNo semantic dependency from Host reaches symbols owned by ..."
    );

    let file_explanation = analyzer
        .explain_file_load(std::path::Path::new("Host.h"), None)
        .expect("explain_file_load failed");
    assert_eq!(file_explanation.path, vec!["Host.h", ".."]);
    assert_eq!(file_explanation.collateral_source, "..");
    assert_eq!(
        file_explanation.reason,
        ".. is included by Host.h.\nNo semantic dependency from Host.h reaches symbols owned by ..."
    );
}

#[test]
fn test_missing_defining_file_yields_err_not_panic() {
    // Records with defining_file: None are legitimate; symbol analysis and
    // explanation must surface Err instead of panicking.
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(50), PathBuf::from("Ghost.h"));

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    let ghost = SymbolId::new("Ghost");
    model.symbols.insert(
        ghost.clone(),
        ReducedSymbolRecord {
            id: ghost.clone(),
            name: "Ghost".to_string(),
            kind: SymbolKind::Function,
            defining_file: None,
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        },
    );
    model.file_symbols.insert(FileId(50), vec![ghost]);

    let analyzer = LoadAnalyzer::new(&ig, &model);
    assert!(analyzer.analyze_symbol_load("Ghost").is_err());
    assert!(analyzer.explain_symbol_load("Ghost", None).is_err());
}
