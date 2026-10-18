//! UTA-T1 item 12 (hand H1): pin the `include_load == total_load`
//! degeneracy (`crates/graph/src/load.rs` `compute_load` fills BOTH fields
//! from the same `physical_symbols.len()`), reported for an owner decision -
//! changing it would widen the frozen `LoadMetric` artifact shape's meaning.
//! The extraction_impact guards (item 6) are already pinned by
//! `test_impact_guards_return_none` in `load_impact_tests.rs` and are not
//! duplicated here.

use super::*;
use crate::reduced::ReducedSymbolRecord;
use ariadne_index::types::SymbolKind;
use std::collections::BTreeMap;

fn record(name: &str, defining_file: FileId) -> ReducedSymbolRecord {
    ReducedSymbolRecord {
        id: SymbolId::new(name),
        name: name.to_string(),
        kind: SymbolKind::Function,
        defining_file: Some(defining_file),
        declarations: Vec::new(),
        definition: None,
        declaring_files: Vec::new(),
    }
}

/// thing.h -> dep.h with one symbol per file and no semantic edges: the
/// dep.h symbol is pure collateral, so the split is nonzero and the
/// degeneracy is visible (not an all-zero coincidence).
fn split_fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(1), PathBuf::from("thing.h"));
    ig.register_file(FileId(2), PathBuf::from("dep.h"));
    ig.add_edge(crate::types::IncludeEdge {
        source: FileId(1),
        destination: Some(FileId(2)),
        context_name: String::new(),
        source_location: crate::types::SourceLocation::new(1, 1),
        kind: crate::types::IncludeKind::Local,
        raw_path: "dep.h".to_string(),
        resolved_path: None,
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
    model
        .symbols
        .insert(SymbolId::new("Thing"), record("Thing", FileId(1)));
    model
        .symbols
        .insert(SymbolId::new("Helper"), record("Helper", FileId(2)));
    model.file_symbols.insert(FileId(1), vec![SymbolId::new("Thing")]);
    model.file_symbols.insert(FileId(2), vec![SymbolId::new("Helper")]);
    (ig, model)
}

#[test]
fn pin_include_load_equals_total_load() {
    // Pin (semantics undecided, reported): `include_load` is NOT the
    // collateral-inclusive total minus something - it equals `total_load`
    // verbatim on every metric, split or not. If a fix ever separates the
    // two, this pin turns red and the owner decides.
    let (ig, model) = split_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let file_metric = analyzer
        .analyze_file_load_by_id(FileId(1))
        .expect("file load on the split fixture");
    assert_eq!(file_metric.semantic_load, 1, "fixture has one semantic symbol");
    assert_eq!(
        file_metric.collateral_load, 1,
        "fixture has one collateral symbol - the split is nonzero"
    );
    assert_eq!(
        file_metric.include_load, file_metric.total_load,
        "degeneracy: include_load is total_load, not semantic+collateral-by-definition"
    );

    let symbol_metric = analyzer
        .analyze_symbol_load("Thing")
        .expect("symbol load on the split fixture");
    assert_eq!(
        symbol_metric.include_load, symbol_metric.total_load,
        "the degeneracy holds on the symbol lane too"
    );
}
