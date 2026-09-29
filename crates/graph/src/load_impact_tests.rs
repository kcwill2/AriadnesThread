use super::*;
use crate::reduced::{ReducedSemanticEdge, ReducedSymbolRecord};
use crate::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

// Base fixture for the extraction-impact suite (Pass E, plan 1790521470000).
//
// Include graph (strict-closure convention: closure(X) = transitive includes
// of X, NOT containing X itself — same as IncludeClosure::transitive_includes):
//
//   main.cpp -> fat.h
//   fat.h    -> base.h, c.h
//   c.h      -> d.h, e.h
//
// Justified symbols FatA/FatB are defined in fat.h; their semantic closure
// reaches only BaseT@base.h, so R = required_headers = {base.h} (fat.h is the
// seed defining file and excluded per the A2 rule).

// `pub(super)` so the perf suite (sibling module of the shared `load`
// parent) can reuse this fixture for its cold-vs-warm impact nets.
pub(super) const MAIN: FileId = FileId(1);
pub(super) const FAT: FileId = FileId(2);
const BASE: FileId = FileId(3);
pub(super) const C_H: FileId = FileId(4);
pub(super) const D_H: FileId = FileId(5);
const E_H: FileId = FileId(6);

fn include(from: FileId, to: FileId, name: &str) -> IncludeEdge {
    IncludeEdge {
        source: from,
        destination: Some(to),
        context_name: "".to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: name.to_string(),
        resolved_path: Some(PathBuf::from(name)),
    }
}

fn add_sym(model: &mut ReducedSemanticModel, name: &str, file: FileId) -> SymbolId {
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
        },
    );
    id
}

fn dep(src: &SymbolId, dst: &SymbolId) -> ReducedSemanticEdge {
    ReducedSemanticEdge {
        src_symbol: src.clone(),
        dst_symbol: dst.clone(),
        kind: DependencyKind::Complete,
    }
}

/// Extra edges: pairs of `(from, to)` include edges added on top of the base
/// fixture (e.g. `[(MAIN, D_H)]` for the sibling-path trap). Lone, when true,
/// registers a seed symbol in fat.h with NO semantic edges — an extraction
/// justified by it requires zero headers (R = {}).
pub(super) fn setup_impact_fixture(
    extra_edges: &[(FileId, FileId, &str)],
    lone: bool,
) -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(MAIN, PathBuf::from("main.cpp"));
    ig.register_file(FAT, PathBuf::from("fat.h"));
    ig.register_file(BASE, PathBuf::from("base.h"));
    ig.register_file(C_H, PathBuf::from("c.h"));
    ig.register_file(D_H, PathBuf::from("d.h"));
    ig.register_file(E_H, PathBuf::from("e.h"));

    ig.add_edge(include(MAIN, FAT, "fat.h"));
    ig.add_edge(include(FAT, BASE, "base.h"));
    ig.add_edge(include(FAT, C_H, "c.h"));
    ig.add_edge(include(C_H, D_H, "d.h"));
    ig.add_edge(include(C_H, E_H, "e.h"));
    for &(from, to, name) in extra_edges {
        ig.add_edge(include(from, to, name));
    }

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),
    };

    let fat_a = add_sym(&mut model, "FatA", FAT);
    let fat_b = add_sym(&mut model, "FatB", FAT);
    if lone {
        // Deliberately NOT inserted into file_symbols: fat.h is never in any
        // dropped list, so its symbol_count must stay 2 in the other tests.
        add_sym(&mut model, "Lone", FAT);
    }
    let base_t = add_sym(&mut model, "BaseT", BASE);
    let c_sym = add_sym(&mut model, "Csym", C_H);
    let d_sym = add_sym(&mut model, "Dsym", D_H);
    let d_sym2 = add_sym(&mut model, "Dsym2", D_H);
    let e_sym = add_sym(&mut model, "Esym", E_H);

    model
        .file_symbols
        .insert(FAT, vec![fat_a.clone(), fat_b.clone()]);
    model.file_symbols.insert(BASE, vec![base_t.clone()]);
    model.file_symbols.insert(C_H, vec![c_sym.clone()]);
    model
        .file_symbols
        .insert(D_H, vec![d_sym.clone(), d_sym2.clone()]);
    model.file_symbols.insert(E_H, vec![e_sym.clone()]);

    model.edges = vec![dep(&fat_a, &base_t), dep(&fat_b, &base_t)];

    (ig, model)
}

fn basenames(headers: &[ImpactHeader]) -> Vec<String> {
    headers
        .iter()
        .map(|h| h.path.file_name().unwrap().to_string_lossy().to_string())
        .collect()
}

fn id_set(headers: &[ImpactHeader]) -> HashSet<FileId> {
    headers.iter().map(|h| h.file_id).collect()
}

// Scenario 1 — base fixture, no sibling paths.
//
// Hand computation (strict-closure convention):
//   closure(FAT)    = {base, c, d, e}
//   kept            = override BFS from FAT with FAT -> {BASE}: {fat, base}
//   dropped_directly = {c, d, e}
//   closure(MAIN)   = {fat, base, c, d, e}            -> before = 5
//   closure_after(MAIN) = override BFS from MAIN, FAT -> {BASE}: {fat, base}
//                                                    -> after  = 2
//   dropped_from_root = {c, d, e};  still_reachable = {}
//   symbols_no_longer_parsed = c(1) + d(2) + e(1) = 4
#[test]
fn test_impact_base_fixture_drop_sets_and_counts() {
    let (ig, model) = setup_impact_fixture(&[], false);
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let impact = analyzer
        .extraction_impact(MAIN, FAT, &[SymbolId::new("FatA"), SymbolId::new("FatB")])
        .expect("base impact must exist");

    assert_eq!(impact.root_file_id, MAIN);
    assert_eq!(impact.source_file_id, FAT);
    assert_eq!(basenames(&impact.required_headers), vec!["base.h"]);
    assert_eq!(impact.required_headers[0].file_id, BASE);

    assert_eq!(
        basenames(&impact.dropped_directly),
        vec!["c.h", "d.h", "e.h"]
    );
    assert_eq!(
        basenames(&impact.dropped_from_root),
        vec!["c.h", "d.h", "e.h"]
    );
    assert!(impact.still_reachable.is_empty());

    assert_eq!(impact.root_closure_before, 5);
    assert_eq!(impact.root_closure_after, 2);
    assert_eq!(impact.symbols_no_longer_parsed, 4);

    // Per-header symbol counts come from file_symbols: c=1, d=2, e=1.
    let counts: Vec<(String, usize)> = impact
        .dropped_from_root
        .iter()
        .map(|h| {
            (
                h.path.file_name().unwrap().to_string_lossy().to_string(),
                h.symbol_count,
            )
        })
        .collect();
    assert_eq!(
        counts,
        vec![
            ("c.h".to_string(), 1),
            ("d.h".to_string(), 2),
            ("e.h".to_string(), 1)
        ]
    );

    // E-I1 partition: dropped_directly = dropped_from_root ⊎ still_reachable.
    let union = id_set(&impact.dropped_from_root)
        .union(&id_set(&impact.still_reachable))
        .copied()
        .collect::<HashSet<FileId>>();
    assert_eq!(union, id_set(&impact.dropped_directly));
    assert!(id_set(&impact.dropped_from_root).is_disjoint(&id_set(&impact.still_reachable)));

    // Required set never contains the source itself (A2 rule).
    assert!(impact.required_headers.iter().all(|h| h.file_id != FAT));
    // Count consistency: after = before - |dropped_from_root|.
    assert_eq!(
        impact.root_closure_after,
        impact.root_closure_before - impact.dropped_from_root.len()
    );
}

// Scenario 2 — SIBLING-PATH TRAP: base fixture + edge main.cpp -> d.h.
//
// Hand computation (this is the honest math, checked by hand BEFORE asserting;
// the plan prose guessed dropped_from_root == [] — it is WRONG because c.h
// reaches main ONLY through fat.h, which the slim edge genuinely sheds):
//   closure(MAIN) = {fat, base, c, d, e}                       -> before = 5
//   R = {base}
//   kept (override BFS from FAT, FAT -> {BASE}) = {fat, base}
//   dropped_directly = closure(FAT) \ kept = {c, d, e}   (the slim edge sheds
//                                                        all three regardless
//                                                        of sibling paths)
//   closure_after(MAIN) = BFS from MAIN over real edges with FAT's outgoing
//     edges replaced by FAT -> {BASE}:
//       MAIN -> {fat, d};  fat -> {base};  d -> {}  =>  {main, fat, d, base}
//     strictly below MAIN: {fat, base, d}                     -> after = 3
//   dropped_from_root = {c, e}   (c sheds via the slim edge; e followed c)
//   still_reachable   = {d}      (d arrives directly via main -> d.h)
//   symbols_no_longer_parsed = c(1) + e(1) = 2
// d.h is the still-reachable decoy: dropped_directly sheds it, yet it never
// leaves main's closure — the exact reason the two drop-sets both exist.
#[test]
fn test_impact_sibling_path_trap() {
    let (ig, model) = setup_impact_fixture(&[(MAIN, D_H, "d.h")], false);
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let impact = analyzer
        .extraction_impact(MAIN, FAT, &[SymbolId::new("FatA"), SymbolId::new("FatB")])
        .expect("trap impact must exist");

    assert_eq!(
        basenames(&impact.dropped_directly),
        vec!["c.h", "d.h", "e.h"],
        "the slim edge sheds the whole subtree of S regardless of sibling paths"
    );
    assert_eq!(basenames(&impact.dropped_from_root), vec!["c.h", "e.h"]);
    assert_eq!(basenames(&impact.still_reachable), vec!["d.h"]);
    assert!(id_set(&impact.still_reachable).contains(&D_H));

    assert_eq!(impact.root_closure_before, 5);
    assert_eq!(impact.root_closure_after, 3);
    assert_eq!(impact.symbols_no_longer_parsed, 2);

    // E-I1 partition + subset relations on the mixed fixture.
    let dd = id_set(&impact.dropped_directly);
    let dfr = id_set(&impact.dropped_from_root);
    let sr = id_set(&impact.still_reachable);
    assert!(dfr.is_subset(&dd));
    assert!(sr.is_subset(&dd));
    assert_eq!(dfr.union(&sr).copied().collect::<HashSet<FileId>>(), dd);
    assert!(dfr.is_disjoint(&sr));

    // E-I2: the source itself still arrives under closure_after.
    assert!(impact.dropped_from_root.iter().all(|h| h.file_id != FAT));
}

// Scenario 3 — PARTIAL TRAP: base fixture + edge main.cpp -> c.h only.
//
// Hand computation:
//   closure(MAIN) = {fat, base, c, d, e}                       -> before = 5
//   closure_after(MAIN) = BFS from MAIN with FAT -> {BASE}:
//     MAIN -> {fat, c};  fat -> {base};  c -> {d, e}
//     => {main, fat, c, base, d, e}; strictly below MAIN = all five -> after = 5
//   dropped_from_root = {} — YES, the math says []: c.h is retained directly
//     by the sibling edge and d.h/e.h keep arriving through c.h, so NOTHING
//     actually leaves main's closure even though dropped_directly still
//     reports the whole shed subtree.
//   dropped_directly = {c, d, e};  still_reachable = {c, d, e}
//   symbols_no_longer_parsed = 0
#[test]
fn test_impact_partial_sibling_keeps_root_closure_whole() {
    let (ig, model) = setup_impact_fixture(&[(MAIN, C_H, "c.h")], false);
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let impact = analyzer
        .extraction_impact(MAIN, FAT, &[SymbolId::new("FatA"), SymbolId::new("FatB")])
        .expect("partial trap impact must exist");

    assert_eq!(
        basenames(&impact.dropped_directly),
        vec!["c.h", "d.h", "e.h"]
    );
    assert!(
        impact.dropped_from_root.is_empty(),
        "c.h retained directly; d.h and e.h arrive via c.h — nothing leaves main"
    );
    assert_eq!(
        basenames(&impact.still_reachable),
        vec!["c.h", "d.h", "e.h"]
    );
    assert_eq!(impact.root_closure_before, 5);
    assert_eq!(impact.root_closure_after, 5);
    assert_eq!(impact.symbols_no_longer_parsed, 0);
}

// Scenario 4 — E-I3 guards: all return None, never panic.
#[test]
fn test_impact_guards_return_none() {
    let (ig, model) = setup_impact_fixture(&[], false);
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let seeds = [SymbolId::new("FatA")];

    assert_eq!(analyzer.extraction_impact(FAT, FAT, &seeds), None);
    assert_eq!(analyzer.extraction_impact(MAIN, MAIN, &seeds), None);
    assert_eq!(analyzer.extraction_impact(MAIN, FAT, &[]), None);

    let unknown = FileId(99);
    assert_eq!(analyzer.extraction_impact(unknown, FAT, &seeds), None);
    assert_eq!(analyzer.extraction_impact(MAIN, unknown, &seeds), None);
    assert_eq!(analyzer.extraction_impact(unknown, unknown, &seeds), None);
}

// Scenario 5 — empty R: justified symbol with NO semantic closure edges.
//
// Hand computation (Lone@fat.h, no dependency edges -> R = {}):
//   kept = override BFS from FAT with FAT -> {}: {fat}
//   dropped_directly = closure(FAT) \ {fat} = {base, c, d, e}  (strictly
//     below S — S itself is NEVER in a dropped list)
//   closure_after(MAIN) = {main, fat} -> strictly below MAIN: {fat} -> after = 1
//   dropped_from_root = {base, c, d, e};  still_reachable = {}
//   symbols_no_longer_parsed = base(1) + c(1) + d(2) + e(1) = 5
#[test]
fn test_impact_empty_required_set_never_drops_source_and_closure_after_is_solely_s() {
    let (ig, model) = setup_impact_fixture(&[], true);
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let impact = analyzer
        .extraction_impact(MAIN, FAT, &[SymbolId::new("Lone")])
        .expect("empty-R impact must exist");

    assert!(impact.required_headers.is_empty());
    assert_eq!(
        basenames(&impact.dropped_directly),
        vec!["base.h", "c.h", "d.h", "e.h"]
    );
    assert_eq!(
        basenames(&impact.dropped_from_root),
        vec!["base.h", "c.h", "d.h", "e.h"]
    );
    assert!(impact.still_reachable.is_empty());

    // S is in none of the dropped lists, even with R empty.
    for list in [
        &impact.dropped_directly,
        &impact.dropped_from_root,
        &impact.still_reachable,
    ] {
        assert!(list.iter().all(|h| h.file_id != FAT));
    }

    // closure_after consists of exactly S (the slim header still arrives).
    assert_eq!(impact.root_closure_after, 1);
    assert_eq!(impact.symbols_no_longer_parsed, 5);
}

// Scenario 6 — determinism: building the same impact twice yields equal
// structs (list order is path-asc, never hash-map order).
#[test]
fn test_impact_deterministic_across_runs() {
    let (ig, model) = setup_impact_fixture(&[(MAIN, D_H, "d.h")], false);
    let seeds = [SymbolId::new("FatA"), SymbolId::new("FatB")];
    let first = LoadAnalyzer::new(&ig, &model)
        .extraction_impact(MAIN, FAT, &seeds)
        .expect("first run");
    let second = LoadAnalyzer::new(&ig, &model)
        .extraction_impact(MAIN, FAT, &seeds)
        .expect("second run");
    assert_eq!(first, second);

    // Same analyzer instance, second call (closures cache is warm on call 2).
    let warm = LoadAnalyzer::new(&ig, &model);
    let _ = warm.extraction_impact(MAIN, FAT, &seeds);
    let again = warm.extraction_impact(MAIN, FAT, &seeds).expect("warm run");
    assert_eq!(first, again);
}

// Scenario 7 — E-I2 on the base fixture: the source node itself is inside
// closure_after, observable as `source ∈ closure_before \ dropped_from_root`.
// Base: after = before \ dropped_from_root = {fat, base} ∋ fat.
#[test]
fn test_impact_source_still_arrives_in_closure_after() {
    let (ig, model) = setup_impact_fixture(&[], false);
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let impact = analyzer
        .extraction_impact(MAIN, FAT, &[SymbolId::new("FatA"), SymbolId::new("FatB")])
        .expect("base impact must exist");

    assert!(impact.dropped_from_root.iter().all(|h| h.file_id != FAT));
    assert!(impact.root_closure_after >= 1);
    assert_eq!(impact.root_closure_after, 2); // {fat (S, kept), base (R)}
}
