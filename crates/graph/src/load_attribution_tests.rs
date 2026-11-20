use super::*;
use crate::reduced::{ReducedSemanticEdge, ReducedSymbolRecord};
use crate::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use std::collections::BTreeMap;
use std::path::PathBuf;

// Root.cpp (1) includes Api.h (2). ApiFn and ApiUnused are declared in
// Api.h but defined in Api.cpp (3), which nothing includes - the usual
// split of a C++ library. Root uses ApiFn only.
fn split_definition_fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut graph = IncludeGraph::new();
    graph.register_file(FileId(1), PathBuf::from("Root.cpp"));
    graph.register_file(FileId(2), PathBuf::from("Api.h"));
    graph.register_file(FileId(3), PathBuf::from("Api.cpp"));
    graph.add_edge(IncludeEdge {
        source: FileId(1),
        destination: Some(FileId(2)),
        context_name: String::new(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "Api.h".to_string(),
        resolved_path: Some(PathBuf::from("Api.h")),
        language: crate::types::LanguageId::cpp(),
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
    let mut add = |name: &str, defined_in: FileId, declared_in: Vec<FileId>| {
        let id = SymbolId::new(name);
        model.symbols.insert(
            id.clone(),
            ReducedSymbolRecord {
                id: id.clone(),
                name: name.to_string(),
                kind: SymbolKind::Function,
                defining_file: Some(defined_in),
                declarations: vec![],
                definition: None,
                declaring_files: declared_in,
            },
        );
        id
    };
    let root_fn = add("RootFn", FileId(1), vec![]);
    let api_fn = add("ApiFn", FileId(3), vec![FileId(2)]);
    let api_unused = add("ApiUnused", FileId(3), vec![FileId(2)]);
    model.file_symbols.insert(FileId(1), vec![root_fn.clone()]);
    model.file_symbols.insert(FileId(2), vec![api_fn.clone(), api_unused]);
    model.edges = vec![ReducedSemanticEdge {
        src_symbol: root_fn,
        dst_symbol: api_fn,
        kind: DependencyKind::Complete,
    }];
    (graph, model)
}

fn source_files(sources: &[CollateralSource]) -> Vec<(FileId, usize)> {
    sources.iter().map(|source| (source.file_id, source.symbol_count)).collect()
}

/// A root can only include the header, so both sides credit Api.h, never
/// the unincludable Api.cpp where the bodies happen to live.
#[test]
fn source_defined_symbols_are_credited_to_their_declaring_header() {
    let (graph, model) = split_definition_fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let full = analyzer.analyze_file_load_by_id(FileId(1)).unwrap();
    assert_eq!(source_files(&full.collateral_sources), vec![(FileId(2), 1)]);
    assert!(full.semantic_sources.iter().any(|source| source.file_id == FileId(2)));
    assert!(full.semantic_sources.iter().all(|source| source.file_id != FileId(3)));

    let compact = analyzer.analyze_file_load_compact_by_id(FileId(1), CompactShape::default());
    assert_eq!(source_files(&compact.collateral_sources), vec![(FileId(2), 1)]);
    assert_eq!(compact.semantic_sources, full.semantic_sources);
}

/// A symbol defined in a header keeps its defining file, and one with no
/// declaring header keeps its source file: only the split case moves.
#[test]
fn header_defined_and_undeclared_symbols_keep_their_defining_file() {
    let (graph, mut model) = split_definition_fixture();
    if let Some(record) = model.symbols.get_mut(&SymbolId::new("ApiUnused")) {
        record.declaring_files.clear();
    }
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let full = analyzer.analyze_file_load_by_id(FileId(1)).unwrap();
    assert_eq!(source_files(&full.collateral_sources), vec![(FileId(3), 1)]);
}
