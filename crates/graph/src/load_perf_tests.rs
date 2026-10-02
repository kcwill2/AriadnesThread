use super::*;
use crate::reduced::{ReducedSemanticEdge, ReducedSymbolRecord};
use crate::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

// Chain fixture mirrors load_justification_tests: Thing.h semantically needs
// Helper.h and Base.h symbols, HugeThirdParty.h stays collateral. Helper.h
// owns three symbols so file-level analysis takes a multi-seed input whose
// members can all be warmed individually first.
fn setup_chain_fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(1), PathBuf::from("Thing.h"));
    ig.register_file(FileId(2), PathBuf::from("Helper.h"));
    ig.register_file(FileId(3), PathBuf::from("HugeThirdParty.h"));
    ig.register_file(FileId(4), PathBuf::from("Base.h"));

    let include = |ig: &mut IncludeGraph, from: FileId, to: FileId, name: &str| {
        ig.add_edge(IncludeEdge {
            source: from,
            destination: Some(to),
            context_name: "".to_string(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: name.to_string(),
            resolved_path: Some(PathBuf::from(name)),
            language: ariadne_repo::SourceLanguage::Cpp,
        });
    };
    include(&mut ig, FileId(1), FileId(2), "Helper.h");
    include(&mut ig, FileId(1), FileId(4), "Base.h");
    include(&mut ig, FileId(2), FileId(3), "HugeThirdParty.h");

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

    let thing = add_sym(&mut model, "Thing", FileId(1));
    let helper_fn = add_sym(&mut model, "HelperFn", FileId(2));
    let helper_fn2 = add_sym(&mut model, "HelperFn2", FileId(2));
    let helper_util = add_sym(&mut model, "HelperUtil", FileId(2));
    let base_struct = add_sym(&mut model, "BaseStruct", FileId(4));
    add_sym(&mut model, "HugeThing", FileId(3));
    add_sym(&mut model, "HugeOther", FileId(3));

    model.file_symbols.insert(FileId(1), vec![thing.clone()]);
    model.file_symbols.insert(
        FileId(2),
        vec![helper_fn.clone(), helper_fn2.clone(), helper_util.clone()],
    );
    model.file_symbols.insert(
        FileId(3),
        vec![SymbolId::new("HugeThing"), SymbolId::new("HugeOther")],
    );
    model
        .file_symbols
        .insert(FileId(4), vec![base_struct.clone()]);

    let dep = |src: &SymbolId, dst: &SymbolId| ReducedSemanticEdge {
        src_symbol: src.clone(),
        dst_symbol: dst.clone(),
        kind: DependencyKind::Complete,
    };
    model.edges = vec![
        dep(&thing, &helper_fn),
        dep(&helper_fn, &helper_fn2),
        dep(&thing, &helper_util),
        dep(&thing, &base_struct),
    ];

    (ig, model)
}

// Seed fixture mirrors the required_headers setup in load_justification_tests:
// S1 → X1, X2 (a.h); X1 → Y1 (b.h); S2 → Y1; Lonely lives isolated in seed.h;
// Ghost is never registered anywhere. Seeding S1 together with X1 exercises a
// seed whose closure re-reaches another seed.
fn setup_seed_fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(1), PathBuf::from("seed.h"));
    ig.register_file(FileId(2), PathBuf::from("a.h"));
    ig.register_file(FileId(3), PathBuf::from("b.h"));

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

    let s1 = add_sym(&mut model, "S1", FileId(1));
    let s2 = add_sym(&mut model, "S2", FileId(1));
    add_sym(&mut model, "Lonely", FileId(1));
    let x1 = add_sym(&mut model, "X1", FileId(2));
    let x2 = add_sym(&mut model, "X2", FileId(2));
    let y1 = add_sym(&mut model, "Y1", FileId(3));

    let dep = |src: &SymbolId, dst: &SymbolId| ReducedSemanticEdge {
        src_symbol: src.clone(),
        dst_symbol: dst.clone(),
        kind: DependencyKind::Complete,
    };
    model.edges = vec![dep(&s1, &x1), dep(&s1, &x2), dep(&x1, &y1), dep(&s2, &y1)];

    (ig, model)
}

type HeaderRows = Vec<(FileId, PathBuf, usize)>;

fn header_rows(rows: &[(FileId, PathBuf, usize)]) -> Vec<(String, usize)> {
    rows.iter()
        .map(|(_, path, count)| {
            (
                path.file_name().unwrap().to_string_lossy().to_string(),
                *count,
            )
        })
        .collect()
}

/// Compile-time contract for the parallel load stage (plan
/// LoadPerfImprovements Phase 4): the pipeline shares ONE memoized
/// analyzer across rayon workers, so `LoadAnalyzer` must stay `Send +
/// Sync`. A regression here (e.g. a `RefCell`/`Cell` field creeping back
/// into the struct) fails to compile — that is this test's entire job.
#[test]
fn test_load_analyzer_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LoadAnalyzer<'static>>();
}

#[test]
fn test_single_seed_bfs_runs_once_per_analyzer() {
    let (ig, model) = setup_chain_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let first = analyzer
        .analyze_symbol_load("Thing")
        .expect("first analyze_symbol_load failed");
    let second = analyzer
        .analyze_symbol_load("Thing")
        .expect("second analyze_symbol_load failed");

    assert_eq!(
        first, second,
        "repeated analysis must return identical metrics"
    );
    assert_eq!(
        analyzer.semantic_bfs_traversal_count(),
        1,
        "the second identical single-seed closure must be served from the cache, not re-traversed"
    );
}

#[test]
fn test_symbol_then_file_analysis_reuses_symbol_closures() {
    let (ig, model) = setup_chain_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    for name in ["HelperFn", "HelperFn2", "HelperUtil"] {
        analyzer
            .analyze_symbol_load(name)
            .unwrap_or_else(|_| panic!("analyze_symbol_load({name}) failed"));
    }
    let warmed = analyzer.semantic_bfs_traversal_count();
    assert_eq!(warmed, 3, "warming three distinct seeds must run three BFS");

    let file_metric = analyzer
        .analyze_file_load(Path::new("Helper.h"))
        .expect("analyze_file_load failed");

    assert_eq!(
        analyzer.semantic_bfs_traversal_count(),
        warmed,
        "file analysis over individually warmed member closures must add zero BFS traversals"
    );
    assert!(
        file_metric.semantic_load > 0,
        "fixture must yield semantic load"
    );
}

#[test]
fn test_multi_seed_query_never_bfs_after_seeds_warm() {
    let (ig, model) = setup_seed_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let s1 = SymbolId::new("S1");
    let s2 = SymbolId::new("S2");

    analyzer.required_headers_for_symbols(std::slice::from_ref(&s1));
    analyzer.required_headers_for_symbols(std::slice::from_ref(&s2));
    let warmed = analyzer.semantic_bfs_traversal_count();

    let combined = analyzer.required_headers_for_symbols(&[s1.clone(), s2.clone()]);

    assert_eq!(
        analyzer.semantic_bfs_traversal_count(),
        warmed,
        "multi-seed query over already-warm seeds must add zero BFS traversals"
    );
    assert_eq!(
        header_rows(&combined),
        vec![("a.h".to_string(), 2), ("b.h".to_string(), 1)],
        "union-served multi-seed query must match the direct multi-seed result"
    );
}

// Justification-style chain (Thing → HelperFn → HelperFn2, HelperUtil,
// BaseStruct + collateral HugeThirdParty) plus two disjoint dependency chains
// (alpha.h → alpha_helper.h → alpha_leaf.h, beta.h → beta_helper.h) so
// concurrent workers hit independent closures as well as the shared adjacency
// and memo tables.
pub fn setup_concurrency_fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(1), PathBuf::from("Thing.h"));
    ig.register_file(FileId(2), PathBuf::from("Helper.h"));
    ig.register_file(FileId(3), PathBuf::from("HugeThirdParty.h"));
    ig.register_file(FileId(4), PathBuf::from("Base.h"));
    ig.register_file(FileId(5), PathBuf::from("alpha.h"));
    ig.register_file(FileId(6), PathBuf::from("alpha_helper.h"));
    ig.register_file(FileId(7), PathBuf::from("alpha_leaf.h"));
    ig.register_file(FileId(8), PathBuf::from("beta.h"));
    ig.register_file(FileId(9), PathBuf::from("beta_helper.h"));

    let include = |ig: &mut IncludeGraph, from: FileId, to: FileId, name: &str| {
        ig.add_edge(IncludeEdge {
            source: from,
            destination: Some(to),
            context_name: "".to_string(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: name.to_string(),
            resolved_path: Some(PathBuf::from(name)),
            language: ariadne_repo::SourceLanguage::Cpp,
        });
    };
    include(&mut ig, FileId(1), FileId(2), "Helper.h");
    include(&mut ig, FileId(1), FileId(4), "Base.h");
    include(&mut ig, FileId(2), FileId(3), "HugeThirdParty.h");
    include(&mut ig, FileId(5), FileId(6), "alpha_helper.h");
    include(&mut ig, FileId(6), FileId(7), "alpha_leaf.h");
    include(&mut ig, FileId(8), FileId(9), "beta_helper.h");

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

    let thing = add_sym(&mut model, "Thing", FileId(1));
    let helper_fn = add_sym(&mut model, "HelperFn", FileId(2));
    let helper_fn2 = add_sym(&mut model, "HelperFn2", FileId(2));
    let helper_fn3 = add_sym(&mut model, "HelperFn3", FileId(2));
    let helper_util = add_sym(&mut model, "HelperUtil", FileId(2));
    let base_struct = add_sym(&mut model, "BaseStruct", FileId(4));
    let base_util = add_sym(&mut model, "BaseUtil", FileId(4));
    add_sym(&mut model, "HugeThing", FileId(3));
    add_sym(&mut model, "HugeOther", FileId(3));
    let alpha = add_sym(&mut model, "Alpha", FileId(5));
    let alpha_helper = add_sym(&mut model, "AlphaHelper", FileId(6));
    let alpha_leaf = add_sym(&mut model, "AlphaLeaf", FileId(7));
    add_sym(&mut model, "AlphaLeaf2", FileId(7));
    let beta = add_sym(&mut model, "Beta", FileId(8));
    let beta_helper = add_sym(&mut model, "BetaHelper", FileId(9));
    add_sym(&mut model, "BetaStandalone", FileId(8));

    let dep = |src: &SymbolId, dst: &SymbolId| ReducedSemanticEdge {
        src_symbol: src.clone(),
        dst_symbol: dst.clone(),
        kind: DependencyKind::Complete,
    };
    model.edges = vec![
        dep(&thing, &helper_fn),
        dep(&helper_fn, &helper_fn2),
        dep(&helper_fn2, &helper_fn3),
        dep(&thing, &helper_util),
        dep(&thing, &base_struct),
        dep(&base_struct, &base_util),
        dep(&alpha, &alpha_helper),
        dep(&alpha_helper, &alpha_leaf),
        dep(&beta, &beta_helper),
    ];

    (ig, model)
}

/// Shared-state regression net for the contention-free cache rework (plan
/// LoadPerfImprovements): eight threads share ONE analyzer through `&` —
/// this only compiles because `LoadAnalyzer` is `Sync` — and every
/// concurrently served `analyze_symbol_load` metric must be byte-identical
/// to the sequential reference. A racy OnceLock/Arc rewrite (torn cache
/// entries, BFS reading a half-built adjacency map) or a deadlock surfaces
/// here as a mismatch or a hang, not as a flaky pass elsewhere.
#[test]
fn test_concurrent_shared_analyzer_matches_sequential() {
    const WORKER_COUNT: usize = 8;
    let (ig, model) = setup_concurrency_fixture();
    let symbol_names: Vec<String> = model.symbols.values().map(|r| r.name.clone()).collect();
    assert!(
        symbol_names.len() >= 2 * WORKER_COUNT,
        "fixture must have more symbols than workers so every worker does real work"
    );

    let reference = LoadAnalyzer::new(&ig, &model);
    let sequential: Vec<LoadMetric> = symbol_names
        .iter()
        .map(|name| {
            reference
                .analyze_symbol_load(name)
                .unwrap_or_else(|_| panic!("sequential analyze_symbol_load({name}) failed"))
        })
        .collect();

    let shared = LoadAnalyzer::new(&ig, &model);
    let names = &symbol_names;
    let per_worker: Vec<Vec<(usize, LoadMetric)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..WORKER_COUNT)
            .map(|worker| {
                let analyzer = &shared;
                scope.spawn(move || {
                    (worker..names.len())
                        .step_by(WORKER_COUNT)
                        .map(|index| {
                            let metric = analyzer
                                .analyze_symbol_load(&names[index])
                                .unwrap_or_else(|_| {
                                    panic!("concurrent analyze_symbol_load at {index} failed")
                                });
                            (index, metric)
                        })
                        .collect()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("a concurrent worker panicked"))
            .collect()
    });

    let mut seen_count = 0usize;
    for results in per_worker {
        for (index, metric) in results {
            assert_eq!(
                metric, sequential[index],
                "concurrent metric for {} must equal the sequential reference",
                symbol_names[index]
            );
            seen_count += 1;
        }
    }
    assert_eq!(
        seen_count,
        symbol_names.len(),
        "every symbol must be covered exactly once by the striped workers"
    );
}

/// Same shared-analyzer race net for the multi-seed public path: concurrent
/// `required_headers_for_symbols` calls over mixed seed groups (including
/// groups whose members cross chains and an unregistered seed) must equal
/// the sequential reference row-for-row.
#[test]
fn test_concurrent_required_headers_matches_sequential() {
    const WORKER_COUNT: usize = 8;
    let (ig, model) = setup_concurrency_fixture();
    let symbol_ids: Vec<SymbolId> = model.symbols.keys().cloned().collect();
    assert!(symbol_ids.len() >= 2 * WORKER_COUNT);

    let mut groups: Vec<Vec<SymbolId>> = symbol_ids.chunks(3).map(|chunk| chunk.to_vec()).collect();
    groups.push(vec![symbol_ids[0].clone(), symbol_ids[5].clone()]);
    groups.push(vec![
        symbol_ids[1].clone(),
        SymbolId::new("GhostNotInModel"),
    ]);
    groups.push(vec![
        symbol_ids[2].clone(),
        symbol_ids[4].clone(),
        symbol_ids[7].clone(),
    ]);

    let reference = LoadAnalyzer::new(&ig, &model);
    let sequential: Vec<HeaderRows> = groups
        .iter()
        .map(|group| reference.required_headers_for_symbols(group))
        .collect();

    let shared = LoadAnalyzer::new(&ig, &model);
    let seed_groups = &groups;
    let per_worker: Vec<Vec<(usize, HeaderRows)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..WORKER_COUNT)
            .map(|worker| {
                let analyzer = &shared;
                scope.spawn(move || {
                    (worker..seed_groups.len())
                        .step_by(WORKER_COUNT)
                        .map(|index| {
                            (
                                index,
                                analyzer.required_headers_for_symbols(&seed_groups[index]),
                            )
                        })
                        .collect()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("a concurrent worker panicked"))
            .collect()
    });

    let mut seen_count = 0usize;
    for results in per_worker {
        for (index, rows) in results {
            assert_eq!(
                rows, sequential[index],
                "concurrent required_headers_for_symbols for group {index} must equal the sequential reference"
            );
            seen_count += 1;
        }
    }
    assert_eq!(seen_count, groups.len());
}

#[test]
fn test_union_of_single_seed_closures_equals_direct_multi_seed() {
    let (ig, model) = setup_seed_fixture();
    let s1 = SymbolId::new("S1");
    let s2 = SymbolId::new("S2");
    let x1 = SymbolId::new("X1");
    let lonely = SymbolId::new("Lonely");
    let ghost = SymbolId::new("GhostNotInModel");

    let cases: Vec<Vec<SymbolId>> = vec![
        vec![s1.clone(), s2.clone()],
        vec![s1.clone(), x1.clone()],
        vec![lonely.clone()],
        vec![ghost.clone()],
        vec![ghost.clone(), s2.clone()],
        vec![lonely.clone(), s1.clone(), ghost.clone()],
        vec![],
    ];

    for seeds in &cases {
        let direct = LoadAnalyzer::new(&ig, &model)
            .memo()
            .get_semantic_closure_of_symbols(&model, seeds);

        let warm = LoadAnalyzer::new(&ig, &model);
        let mut union = HashSet::new();
        for seed in seeds {
            union.extend(
                warm.memo()
                    .get_semantic_closure_of_symbols(&model, std::slice::from_ref(seed)),
            );
        }
        let via_warm = warm.memo().get_semantic_closure_of_symbols(&model, seeds);

        assert_eq!(
            direct, union,
            "closure({seeds:?}) must equal the union of the per-seed closures"
        );
        assert_eq!(
            direct, via_warm,
            "multi-seed closure over warmed seeds must equal the cold direct closure"
        );
    }

    // Unknown and edgeless seeds contribute themselves only, mirroring
    // test_required_headers_unknown_symbols_ignored semantics.
    let ghost_closure = LoadAnalyzer::new(&ig, &model)
        .memo()
        .get_semantic_closure_of_symbols(&model, std::slice::from_ref(&ghost));
    assert_eq!(ghost_closure, HashSet::from([ghost.clone()]));
    let lonely_closure = LoadAnalyzer::new(&ig, &model)
        .memo()
        .get_semantic_closure_of_symbols(&model, std::slice::from_ref(&lonely));
    assert_eq!(lonely_closure, HashSet::from([lonely.clone()]));

    // The public projection agrees cold-vs-warm too, even with an unknown seed mixed in.
    let seeds = [s1.clone(), ghost.clone(), x1.clone()];
    let cold_rows = LoadAnalyzer::new(&ig, &model).required_headers_for_symbols(&seeds);
    let warm = LoadAnalyzer::new(&ig, &model);
    for seed in &seeds {
        warm.required_headers_for_symbols(std::slice::from_ref(seed));
    }
    let warm_rows = warm.required_headers_for_symbols(&seeds);
    assert_eq!(
        cold_rows, warm_rows,
        "required_headers_for_symbols must be identical cold and seed-warmed"
    );
}

/// Locks the per-def-file physical-expansion memo (plan
/// LoadPerfImprovements): the transitive-include symbol expansion of a
/// single def-file is computed ONCE per analyzer and reused — across
/// repeated analyses of the same symbol and between the symbol and file
/// analyses that share one def-file — while every returned metric stays
/// identical to a fresh (cold) analyzer.
#[test]
fn test_physical_expansion_reused_single_def_file() {
    let (ig, model) = setup_chain_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let first = analyzer
        .analyze_symbol_load("Thing")
        .expect("first analyze_symbol_load failed");
    assert_eq!(
        analyzer.physical_expansion_count(),
        1,
        "the first analysis of a def-file must run exactly one real expansion"
    );

    let second = analyzer
        .analyze_symbol_load("Thing")
        .expect("second analyze_symbol_load failed");
    assert_eq!(
        first, second,
        "repeated analysis must return identical metrics"
    );
    assert_eq!(
        analyzer.physical_expansion_count(),
        1,
        "the second analysis of the same def-file must reuse the memoized expansion"
    );

    let file_metric = analyzer
        .analyze_file_load(Path::new("Thing.h"))
        .expect("analyze_file_load failed");
    assert_eq!(
        analyzer.physical_expansion_count(),
        1,
        "file analysis over the same def-file must reuse the memoized expansion"
    );

    let fresh = LoadAnalyzer::new(&ig, &model);
    assert_eq!(
        fresh
            .analyze_symbol_load("Thing")
            .expect("fresh symbol analysis failed"),
        first,
        "memoized symbol metrics must equal a cold analyzer's"
    );
    assert_eq!(
        fresh
            .analyze_file_load(Path::new("Thing.h"))
            .expect("fresh file analysis failed"),
        file_metric,
        "memoized file metrics must equal a cold analyzer's"
    );
}

#[test]
fn test_memoized_closure_matches_uncached_across_all_justification_fixtures() {
    let (ig, model) = setup_chain_fixture();
    let symbol_names: Vec<String> = model.symbols.values().map(|r| r.name.clone()).collect();
    let file_paths: Vec<PathBuf> = ig.file_paths.values().cloned().collect();
    assert!(!symbol_names.is_empty() && file_paths.len() >= 4);

    let warm = LoadAnalyzer::new(&ig, &model);
    for name in &symbol_names {
        warm.analyze_symbol_load(name)
            .unwrap_or_else(|_| panic!("warm analyze_symbol_load({name}) failed"));
    }
    for path in &file_paths {
        warm.analyze_file_load(path)
            .unwrap_or_else(|_| panic!("warm analyze_file_load({}) failed", path.display()));
    }

    for name in &symbol_names {
        let cached = warm
            .analyze_symbol_load(name)
            .unwrap_or_else(|_| panic!("cached analyze_symbol_load({name}) failed"));
        let fresh = LoadAnalyzer::new(&ig, &model)
            .analyze_symbol_load(name)
            .unwrap_or_else(|_| panic!("fresh analyze_symbol_load({name}) failed"));
        assert_eq!(
            cached, fresh,
            "symbol metrics from the warm analyzer must match a fresh one for {name}"
        );
    }
    for path in &file_paths {
        let cached = warm
            .analyze_file_load(path)
            .unwrap_or_else(|_| panic!("cached analyze_file_load failed"));
        let fresh = LoadAnalyzer::new(&ig, &model)
            .analyze_file_load(path)
            .unwrap_or_else(|_| panic!("fresh analyze_file_load failed"));
        assert_eq!(
            cached,
            fresh,
            "file metrics from the warm analyzer must match a fresh one for {}",
            path.display()
        );
    }
}

/// Hand-counted lock for both cache-len accessors: warming Thing (Thing.h)
/// and HelperFn (Helper.h) must seed 2 semantic and 2 physical memo entries;
/// re-analyzing Thing.h — sole member Thing, already warm — adds neither.
#[test]
fn test_cache_entry_counts_match_hand_computed_fixture() {
    let (ig, model) = setup_chain_fixture();
    let warm = LoadAnalyzer::new(&ig, &model);
    warm.analyze_symbol_load("Thing").expect("Thing");
    warm.analyze_symbol_load("HelperFn").expect("HelperFn");
    warm.analyze_file_load(Path::new("Thing.h")).expect("file");
    assert_eq!(
        (
            warm.semantic_closure_cache_len(),
            warm.physical_expansion_cache_len()
        ),
        (2, 2),
        "hand-counted: 2 warm seeds, 2 distinct def-files"
    );
}

// ---------------------------------------------------------------------------
// A1 — override_closure must reuse ONE lockless include-adjacency map.
// ---------------------------------------------------------------------------

use super::impact::{setup_impact_fixture, C_H, D_H, FAT, MAIN};

/// A1 (I4-adjacent performance lock): `extraction_impact` runs TWO
/// `override_closure` BFSes per call (kept_subtree + root_after) and is
/// itself invoked once per justification entry, yet the include-adjacency
/// map never changes — building it per BFS was pure re-work. N impact calls
/// must perform AT MOST one adjacency build and warmed calls must add zero
/// (the OnceLock memo the suite demands).
#[test]
fn test_extraction_impact_builds_include_adjacency_at_most_once() {
    const REPEATS: usize = 4;
    let (ig, model) = setup_impact_fixture(&[(MAIN, D_H, "d.h")], false);
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let seeds = [SymbolId::new("FatA"), SymbolId::new("FatB")];

    let first = analyzer
        .extraction_impact(MAIN, FAT, &seeds)
        .expect("impact must exist");
    assert!(
        analyzer.include_adjacency_build_count() <= 1,
        "extraction_impact must share one adjacency build across its BFSes, got {}",
        analyzer.include_adjacency_build_count()
    );

    let warmed = analyzer.include_adjacency_build_count();
    for _ in 1..REPEATS {
        let repeated = analyzer
            .extraction_impact(MAIN, FAT, &seeds)
            .expect("repeated impact must exist");
        assert_eq!(repeated, first, "repeated impact must be identical");
    }
    assert_eq!(
        analyzer.include_adjacency_build_count(),
        warmed,
        "warmed impact calls must add zero adjacency builds"
    );
}

/// One impact-fixture configuration: extra include edges + the lone flag.
type ImpactVariant = (Vec<(FileId, FileId, &'static str)>, bool);

/// A1 equivalence lock (I1/I3): the reused adjacency must not change a
/// single byte of `ExtractionImpact` — every fixture variant (base,
/// sibling trap, partial trap, empty-R) agrees between a cold analyzer, a
/// warmed repeat on the same analyzer, and a fresh analyzer.
#[test]
fn test_extraction_impact_cold_matches_warm_across_variants() {
    let seeds = [SymbolId::new("FatA"), SymbolId::new("FatB")];
    let variants: Vec<ImpactVariant> = vec![
        (vec![], false),
        (vec![(MAIN, D_H, "d.h")], false),
        (vec![(MAIN, C_H, "c.h")], false),
        (vec![], true),
    ];
    for (extra_edges, lone) in &variants {
        let (ig, model) = setup_impact_fixture(extra_edges, *lone);
        let warm = LoadAnalyzer::new(&ig, &model);
        let cold = warm
            .extraction_impact(MAIN, FAT, &seeds)
            .expect("cold impact must exist");
        let repeated = warm
            .extraction_impact(MAIN, FAT, &seeds)
            .expect("warm impact must exist");
        let fresh = LoadAnalyzer::new(&ig, &model)
            .extraction_impact(MAIN, FAT, &seeds)
            .expect("fresh impact must exist");
        assert_eq!(cold, repeated, "warm repeat must equal cold run");
        assert_eq!(cold, fresh, "cold run must equal fresh analyzer");
    }
}

// ---------------------------------------------------------------------------
// A2 — analyze_file_load_by_id must stop re-deriving FileIds.
// ---------------------------------------------------------------------------

/// Counter sanity (characterization of the OLD cost): the path-based public
/// entry resolves through `find_file_id`, so an N-file pass pays exactly N
/// O(V) string scans. The A2 by-id variant must drive this to zero.
#[test]
fn test_path_based_file_analysis_pays_one_scan_per_file() {
    let (ig, model) = setup_chain_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);
    assert_eq!(ig.find_file_id_scan_count(), 0);
    let paths: Vec<&PathBuf> = ig.file_paths.values().collect();
    assert!(paths.len() >= 4, "chain fixture must have >= 4 files");
    for path in &paths {
        analyzer
            .analyze_file_load(path)
            .unwrap_or_else(|_| panic!("analyze_file_load({}) failed", path.display()));
    }
    assert_eq!(
        ig.find_file_id_scan_count(),
        paths.len(),
        "each path-based file analysis must perform exactly one find_file_id scan"
    );
}

/// A2 (I1 lock): for every fixture file — chain and impact fixtures alike —
/// the by-id analysis must return a `LoadMetric` byte-identical to the
/// path-based one, so `analyze_file_load` can safely delegate to it.
#[test]
fn test_analyze_file_load_by_id_matches_path_variant_on_all_fixtures() {
    let fixtures = [
        setup_chain_fixture(),
        setup_impact_fixture(&[], false),
        setup_impact_fixture(&[(MAIN, D_H, "d.h")], false),
    ];
    for (ig, model) in &fixtures {
        let analyzer = LoadAnalyzer::new(ig, model);
        for (&file_id, path) in &ig.file_paths {
            let by_path = analyzer
                .analyze_file_load(path)
                .unwrap_or_else(|_| panic!("analyze_file_load({}) failed", path.display()));
            let by_id = analyzer
                .analyze_file_load_by_id(file_id)
                .unwrap_or_else(|_| panic!("analyze_file_load_by_id({file_id:?}) failed"));
            assert_eq!(
                by_id,
                by_path,
                "by-id metric must equal the path metric for {}",
                path.display()
            );
        }
    }
}

/// A2 (performance lock): an N-file by-id pass — exactly what the pipeline
/// `collect_file_metrics` does per file — must perform ZERO `find_file_id`
/// scans, unlike the identical path-based pass (see the counter sanity test).
#[test]
fn test_by_id_file_pass_performs_zero_find_file_id_scans() {
    let (ig, model) = setup_impact_fixture(&[], false);
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let ids: Vec<FileId> = ig.file_paths.keys().copied().collect();
    assert!(ids.len() >= 6, "impact fixture must have >= 6 files");
    for file_id in &ids {
        analyzer
            .analyze_file_load_by_id(*file_id)
            .unwrap_or_else(|_| panic!("analyze_file_load_by_id({file_id:?}) failed"));
    }
    assert_eq!(
        ig.find_file_id_scan_count(),
        0,
        "the by-id pipeline-shaped pass must never call find_file_id"
    );
}

// ---------------------------------------------------------------------------
// A3 — compute_load single-pass partition.
// ---------------------------------------------------------------------------

/// Verbatim reconstruction of `compute_load`'s current two-pass output:
/// filter pair over `physical_symbols`, each vector sorted ascending. This
/// is the legacy oracle the single-pass partition must match exactly.
fn legacy_symbol_split(
    physical_symbols: &HashSet<SymbolId>,
    semantic_closure: &HashSet<SymbolId>,
) -> (Vec<SymbolId>, Vec<SymbolId>) {
    let mut collateral: Vec<SymbolId> = physical_symbols
        .iter()
        .filter(|s| !semantic_closure.contains(s))
        .cloned()
        .collect();
    let mut semantic: Vec<SymbolId> = physical_symbols
        .iter()
        .filter(|s| semantic_closure.contains(s))
        .cloned()
        .collect();
    collateral.sort();
    semantic.sort();
    (semantic, collateral)
}

/// A3 characterization lock (I3): the single-pass `partition_symbols` must
/// equal the legacy filter-pair output — vector contents AND ascending sort
/// order — for every def-file of the chain/justification and concurrency
/// fixtures, plus the empty-closure and empty-physical edge cases.
#[test]
fn test_partition_symbols_equals_legacy_split_including_order() {
    for (ig, model) in [setup_chain_fixture(), setup_concurrency_fixture()] {
        let analyzer = LoadAnalyzer::new(&ig, &model);
        for (file_id, seeds) in &model.file_symbols {
            let physical =
                analyzer
                    .memo
                    .expand_physical_symbols(&ig, &model, &HashSet::from([*file_id]));
            let closure = analyzer.memo.get_semantic_closure_of_symbols(&model, seeds);
            assert_eq!(
                partition_symbols(&physical, &closure),
                legacy_symbol_split(&physical, &closure),
                "single-pass partition must equal the legacy filter pair for {file_id:?}"
            );
        }
        // Edge cases: nothing semantic, nothing physical.
        let physical =
            analyzer
                .memo
                .expand_physical_symbols(&ig, &model, &HashSet::from([FileId(1)]));
        assert_eq!(
            partition_symbols(&physical, &HashSet::new()),
            legacy_symbol_split(&physical, &HashSet::new()),
            "empty semantic closure must partition all to collateral"
        );
        assert_eq!(
            partition_symbols(&HashSet::new(), &HashSet::new()),
            legacy_symbol_split(&HashSet::new(), &HashSet::new()),
            "empty physical set must partition to two empty vectors"
        );
    }
}

/// A3 warm-vs-cold equivalence net (I1): every file- and symbol-level metric
/// over the concurrency fixture must be identical between a warmed analyzer
/// (caches + memos full) and fresh analyzers.
#[test]
fn test_compute_load_warm_matches_cold_on_concurrency_fixture() {
    let (ig, model) = setup_concurrency_fixture();
    let paths: Vec<PathBuf> = ig.file_paths.values().cloned().collect();
    let names: Vec<String> = model.symbols.values().map(|r| r.name.clone()).collect();

    let warm = LoadAnalyzer::new(&ig, &model);
    let cold_files: Vec<LoadMetric> = paths
        .iter()
        .map(|p| warm.analyze_file_load(p).expect("file load"))
        .collect();
    let cold_symbols: Vec<LoadMetric> = names
        .iter()
        .map(|n| warm.analyze_symbol_load(n).expect("symbol load"))
        .collect();

    let warm_files: Vec<LoadMetric> = paths
        .iter()
        .map(|p| warm.analyze_file_load(p).expect("warm file load"))
        .collect();
    let warm_symbols: Vec<LoadMetric> = names
        .iter()
        .map(|n| warm.analyze_symbol_load(n).expect("warm symbol load"))
        .collect();

    let fresh = LoadAnalyzer::new(&ig, &model);
    let fresh_files: Vec<LoadMetric> = paths
        .iter()
        .map(|p| fresh.analyze_file_load(p).expect("fresh file load"))
        .collect();
    let fresh_symbols: Vec<LoadMetric> = names
        .iter()
        .map(|n| fresh.analyze_symbol_load(n).expect("fresh symbol load"))
        .collect();

    assert_eq!(
        warm_files, cold_files,
        "warmed file metrics must equal cold ones"
    );
    assert_eq!(
        warm_symbols, cold_symbols,
        "warmed symbol metrics must equal cold ones"
    );
    assert_eq!(
        warm_files, fresh_files,
        "warmed file metrics must equal fresh analyzers'"
    );
    assert_eq!(
        warm_symbols, fresh_symbols,
        "warmed symbol metrics must equal fresh analyzers'"
    );
}
