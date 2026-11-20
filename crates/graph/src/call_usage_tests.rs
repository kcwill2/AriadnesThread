//! Call sites become semantic edges, scoped to what the calling unit sees.

use std::path::PathBuf;

use ariadne_index::types::{
    CallSite, DeclId, DependencyKind, FileLocation, IndexResult, SemanticEdge, SymbolId,
    SymbolKind, SymbolRecord, TuAnalysis,
};
use ariadne_repo::FileId;

use crate::reduced::ReducedSemanticModel;

fn function(id: &str, name: &str, owner: FileId) -> SymbolRecord {
    SymbolRecord {
        id: SymbolId::new(id),
        name: name.into(),
        kind: SymbolKind::Function,
        owning_file: owner,
        declarations: vec![DeclId::new(format!("decl_{id}"))],
        definition: None,
        modifiers: Vec::new(),
    }
}

fn call(caller: &str, callee: &str, unit: FileId) -> CallSite {
    CallSite {
        caller_symbol: SymbolId::new(caller),
        callee_name: callee.into(),
        location: FileLocation { file_id: unit, line: 2, column: 5 },
    }
}

fn unit(file: FileId, symbols: Vec<SymbolRecord>, calls: Vec<CallSite>) -> TuAnalysis {
    TuAnalysis {
        tu_file_id: file,
        context_name: "test".into(),
        language: ariadne_repo::SourceLanguage::Cpp,
        symbols,
        declarations: vec![],
        edges: vec![],
        references: vec![],
        call_sites: calls,
        extends_edges: Vec::new(),
        shared_files: Vec::new(),
    }
}

/// user0.cpp includes api.h (api0, api1, Shown::run) and calls `api0`,
/// `run`, `missing` and itself. hidden.cpp alone sees hidden.h's
/// `Hidden::run`.
fn fixture() -> IndexResult {
    let user = FileId::from_relative_path("user0.cpp");
    let api = FileId::from_relative_path("api.h");
    let hidden_cpp = FileId::from_relative_path("hidden.cpp");
    let hidden_h = FileId::from_relative_path("hidden.h");
    let user_unit = unit(
        user,
        vec![
            function("user0", "user0", user),
            function("api0", "api0", api),
            function("api1", "api1", api),
            function("Shown::run", "Shown::run", api),
        ],
        vec![
            call("user0", "api0", user),
            call("user0", "run", user),
            call("user0", "missing", user),
            call("user0", "user0", user),
        ],
    );
    let hidden_unit = unit(hidden_cpp, vec![function("Hidden::run", "Hidden::run", hidden_h)], vec![]);
    IndexResult::new(PathBuf::from("/test"), vec![user_unit, hidden_unit])
}

fn callees_of(model: &ReducedSemanticModel, caller: &str) -> Vec<(String, DependencyKind)> {
    model
        .edges
        .iter()
        .filter(|edge| edge.src_symbol.0 == caller)
        .map(|edge| (edge.dst_symbol.0.clone(), edge.kind))
        .collect()
}

#[test]
fn a_call_becomes_an_edge_to_each_visible_same_named_symbol() {
    let model = ReducedSemanticModel::reduce(&fixture());
    assert_eq!(
        callees_of(&model, "user0"),
        vec![
            ("Shown::run".to_string(), DependencyKind::Incomplete),
            ("api0".to_string(), DependencyKind::Incomplete),
        ],
        "api0 and the visible Shown::run are used; api1 is not called, \
         Hidden::run is not visible to user0.cpp, missing resolves to nothing, \
         and a self-call adds no edge"
    );
}

#[test]
fn a_call_never_downgrades_an_existing_complete_edge() {
    let mut index = fixture();
    let user = FileId::from_relative_path("user0.cpp");
    index.tu_analyses[0].edges.push(SemanticEdge {
        src_symbol: SymbolId::new("user0"),
        dst_symbol: SymbolId::new("api0"),
        kind: DependencyKind::Complete,
        src_file: user,
        dst_file: FileId::from_relative_path("api.h"),
    });
    let model = ReducedSemanticModel::reduce(&index);
    assert!(callees_of(&model, "user0").contains(&("api0".to_string(), DependencyKind::Complete)));
}

#[test]
fn a_call_from_an_unknown_caller_adds_nothing() {
    let mut index = fixture();
    let user = FileId::from_relative_path("user0.cpp");
    index.tu_analyses[0].call_sites = vec![call("ghost", "api0", user)];
    let model = ReducedSemanticModel::reduce(&index);
    assert!(model.edges.is_empty(), "edges: {:?}", model.edges);
}
