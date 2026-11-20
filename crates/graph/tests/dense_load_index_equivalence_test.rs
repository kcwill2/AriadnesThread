//! Locks the dense stage-5 passes (`DenseLoadIndex`) against the reference
//! entries they replace in the pipeline: the per-symbol
//! `analyze_symbol_load_counts_with_expansion` and the per-file
//! `analyze_file_load_compact_by_id`. Every metric of every defining-file
//! group and of every file must be equal, on cyclic call-heavy models and on
//! the odd corners (unknown edge ends, a symbol listed under two files,
//! records without a usable defining file, symbols defined in a source file
//! and credited to their declaring header).

#[path = "common/synthetic_load_model.rs"]
mod synthetic_load_model;

use ariadne_graph::{
    CompactShape, IncludeGraph, LoadAnalyzer, LoadMetric, ReducedSemanticEdge, ReducedSemanticModel,
    ReducedSymbolRecord,
};
use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use std::collections::BTreeMap;
use synthetic_load_model::SyntheticShape;

fn small_shape(seed: u64, out_of_closure_percent: u64) -> SyntheticShape {
    SyntheticShape {
        headers: 40,
        units: 12,
        symbols_per_file: 6,
        uses_per_symbol: 3,
        out_of_closure_percent,
        seed,
    }
}

/// The pipeline's grouping: every record id under its defining file.
fn groups_by_defining_file(model: &ReducedSemanticModel) -> BTreeMap<FileId, Vec<&SymbolId>> {
    let mut groups: BTreeMap<FileId, Vec<&SymbolId>> = BTreeMap::new();
    for (id, record) in &model.symbols {
        if let Some(file) = record.defining_file {
            groups.entry(file).or_default().push(id);
        }
    }
    groups
}

/// The per-symbol entry the dense pass must reproduce, miss paths as None.
fn reference_group(analyzer: &LoadAnalyzer<'_>, file: FileId, ids: &[&SymbolId]) -> Vec<Option<LoadMetric>> {
    let expansion = analyzer.physical_expansion(file);
    ids.iter()
        .map(|id| analyzer.analyze_symbol_load_counts_with_expansion(id, &expansion).ok())
        .collect()
}

fn assert_all_groups_equal(graph: &IncludeGraph, model: &ReducedSemanticModel) -> usize {
    let analyzer = LoadAnalyzer::new(graph, model);
    let index = analyzer.dense_load_index();
    let mut compared = 0;
    for (file, ids) in groups_by_defining_file(model) {
        let dense = index.def_file_group_counts(file, &ids);
        let reference = reference_group(&analyzer, file, &ids);
        assert_eq!(dense, reference, "group of file {file:?} differs");
        compared += ids.len();
    }
    compared
}

/// Every listed file (and one file nothing lists) under `shape`.
fn assert_all_files_equal(graph: &IncludeGraph, model: &ReducedSemanticModel, shape: CompactShape) {
    let analyzer = LoadAnalyzer::new(graph, model);
    let index = analyzer.dense_load_index();
    let unlisted = FileId(888_888);
    for file in model.file_symbols.keys().copied().chain([unlisted]) {
        let dense = index.file_load_compact(file, shape);
        let reference = analyzer.analyze_file_load_compact_by_id(file, shape);
        assert_eq!(dense, reference, "file {file:?} differs under {shape:?}");
    }
}

/// The shapes worth locking: the pipeline default, a shape that truncates
/// both the names and the named sources, and the degenerate zero shape.
fn compact_shapes() -> [CompactShape; 3] {
    [
        CompactShape::default(),
        CompactShape { names_per_source: 2, named_sources: 3 },
        CompactShape { names_per_source: 0, named_sources: 0 },
    ]
}

/// The odd-corner model: see the module docs for what each twist covers.
fn odd_corner_model() -> (IncludeGraph, ReducedSemanticModel) {
    let (graph, mut model) = synthetic_load_model::build(&small_shape(9, 40));
    let known: Vec<SymbolId> = model.symbols.keys().take(3).cloned().collect();

    // Edge ends no record names, in both directions.
    let ghost_target = SymbolId::new("ghost::target");
    let ghost_source = SymbolId::new("ghost::source");
    model.edges.push(edge(&known[0], &ghost_target));
    model.edges.push(edge(&ghost_source, &known[1]));
    // A self edge and a duplicate edge.
    model.edges.push(edge(&known[2], &known[2]));
    model.edges.push(edge(&known[0], &ghost_target));
    // One symbol listed under a second file as well: counted once.
    let second_file = *model.file_symbols.keys().nth(5).expect("fixture has files");
    model.file_symbols.get_mut(&second_file).expect("listed").push(known[0].clone());
    // A file-listed symbol with no record at all.
    model.file_symbols.get_mut(&second_file).expect("listed").push(ghost_target.clone());
    // Records without a defining file, and with an unregistered one.
    let homeless = SymbolId::new("odd::homeless");
    model.symbols.insert(homeless.clone(), record(&homeless, None));
    model.file_symbols.get_mut(&second_file).expect("listed").push(homeless);
    let stray = SymbolId::new("odd::stray");
    model.symbols.insert(stray.clone(), record(&stray, Some(FileId(999_999))));
    model.edges.push(edge(&stray, &known[1]));
    model.file_symbols.get_mut(&second_file).expect("listed").push(stray);
    credit_some_header_symbols_to_source_files(&graph, &mut model);
    (graph, model)
}

/// Re-home every fifth header symbol to a source file that declares it in
/// its original header: the metric must credit the header, not the source.
fn credit_some_header_symbols_to_source_files(graph: &IncludeGraph, model: &mut ReducedSemanticModel) {
    let is_source = |file: &FileId| graph.file_paths[file].extension().is_some_and(|ext| ext == "cpp");
    let source_file = *model.file_symbols.keys().find(|file| is_source(file)).expect("a unit");
    let mut position = 0;
    for symbol_record in model.symbols.values_mut() {
        let Some(header) = symbol_record.defining_file.filter(|file| graph.file_paths.contains_key(file)) else {
            continue;
        };
        position += 1;
        if position % 5 == 0 && !is_source(&header) {
            symbol_record.defining_file = Some(source_file);
            symbol_record.declaring_files = vec![header];
        }
    }
}

fn record(id: &SymbolId, defining_file: Option<FileId>) -> ReducedSymbolRecord {
    ReducedSymbolRecord {
        id: id.clone(),
        name: format!("name_of_{}", id.0),
        kind: SymbolKind::Function,
        defining_file,
        declarations: vec![],
        definition: None,
        declaring_files: vec![],
    }
}

fn edge(source: &SymbolId, target: &SymbolId) -> ReducedSemanticEdge {
    ReducedSemanticEdge {
        src_symbol: source.clone(),
        dst_symbol: target.clone(),
        kind: DependencyKind::Incomplete,
    }
}

#[test]
fn dense_counts_equal_the_reference_on_layered_models() {
    for seed in 1..=4 {
        let (graph, model) = synthetic_load_model::build(&small_shape(seed, 0));
        let compared = assert_all_groups_equal(&graph, &model);
        assert_eq!(compared, model.symbols.len());
    }
}

#[test]
fn dense_counts_equal_the_reference_on_cyclic_call_heavy_models() {
    for seed in 1..=4 {
        let (graph, model) = synthetic_load_model::build(&small_shape(seed, 60));
        assert_all_groups_equal(&graph, &model);
    }
}

#[test]
fn dense_counts_equal_the_reference_on_the_odd_corners() {
    let (graph, model) = odd_corner_model();
    assert_all_groups_equal(&graph, &model);
}

#[test]
fn dense_file_loads_equal_the_reference_on_layered_and_cyclic_models() {
    for (seed, out_of_closure_percent) in [(1, 0), (2, 0), (3, 60), (4, 60), (5, 15)] {
        let (graph, model) = synthetic_load_model::build(&small_shape(seed, out_of_closure_percent));
        for shape in compact_shapes() {
            assert_all_files_equal(&graph, &model, shape);
        }
    }
}

#[test]
fn dense_file_loads_equal_the_reference_on_the_odd_corners() {
    let (graph, model) = odd_corner_model();
    for shape in compact_shapes() {
        assert_all_files_equal(&graph, &model, shape);
    }
}

#[test]
fn the_fixtures_exercise_both_sides_of_the_split() {
    let (graph, model) = synthetic_load_model::build(&small_shape(1, 0));
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let index = analyzer.dense_load_index();
    let metrics: Vec<LoadMetric> = model
        .file_symbols
        .keys()
        .map(|file| index.file_load_compact(*file, CompactShape::default()))
        .collect();
    assert!(metrics.iter().any(|metric| metric.collateral_load > 0 && metric.semantic_load > 0));
    assert!(metrics.iter().any(|metric| metric.collateral_sources.len() > 1));
    assert!(metrics.iter().any(|metric| metric.semantic_sources.len() > 1));
}

#[test]
fn symbols_outside_the_group_file_or_unknown_yield_none() {
    let (graph, model) = synthetic_load_model::build(&small_shape(3, 20));
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let index = analyzer.dense_load_index();
    let groups = groups_by_defining_file(&model);
    let mut files = groups.keys();
    let first_file = *files.next().expect("two groups");
    let other_file = *files.next().expect("two groups");
    let foreign = groups[&other_file][0];
    let own = groups[&first_file][0];
    let unknown = SymbolId::new("never::registered");

    let metrics = index.def_file_group_counts(first_file, &[foreign, own, &unknown]);

    assert_eq!(metrics.len(), 3);
    assert!(metrics[0].is_none(), "a symbol of another file is refused");
    assert_eq!(metrics[1], reference_group(&analyzer, first_file, &[own])[0]);
    assert!(metrics[2].is_none(), "an unregistered id is refused");
}
