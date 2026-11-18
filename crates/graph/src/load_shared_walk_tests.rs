//! Boost-scale stage-5 speedups (fixes 1 + 2 of the collateral-pass
//! investigation, 2026-11-18):
//!
//! 1. Once the capped semantic-closure memo is FULL, a multi-seed closure
//!    query runs ONE shared walk over its uncached seeds instead of one
//!    walk per seed (the per-seed union only paid off when every seed could
//!    be memoized; at boost scale the boot cap fills almost immediately).
//! 2. `physical_expansion` + `analyze_symbol_load_with_expansion` let the
//!    pipeline's symbol pass expand each defining file ONCE and reuse it for
//!    every symbol the file defines, instead of re-expanding per symbol
//!    after the first-N physical memo fills.
//!
//! Both are pure time trades: every result is locked equal to the uncapped
//! reference.

use super::*;
use crate::reduced::{ReducedSemanticEdge, ReducedSymbolRecord};
use crate::types::{IncludeEdge, IncludeKind, LanguageId, SourceLocation};
use ariadne_index::types::{DependencyKind, SymbolKind};
use std::collections::BTreeMap;

fn record(id: &str, file: u64) -> ReducedSymbolRecord {
    ReducedSymbolRecord {
        id: SymbolId::new(id),
        name: id.to_string(),
        kind: SymbolKind::Class,
        defining_file: Some(FileId(file)),
        declarations: vec![],
        definition: None,
        declaring_files: vec![],
    }
}

fn semantic_edge(src: &str, dst: &str) -> ReducedSemanticEdge {
    ReducedSemanticEdge {
        src_symbol: SymbolId::new(src),
        dst_symbol: SymbolId::new(dst),
        kind: DependencyKind::Complete,
    }
}

/// Files: 1 = {a, b}, 2 = {c, d}, 3 = {e, x}; include edge 1 -> 2.
/// Semantic edges: a -> b -> c, d -> c; e and x are isolated.
fn fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut graph = IncludeGraph::new();
    graph.register_file(FileId(1), PathBuf::from("one.h"));
    graph.register_file(FileId(2), PathBuf::from("two.h"));
    graph.register_file(FileId(3), PathBuf::from("three.h"));
    graph.add_edge(IncludeEdge {
        source: FileId(1),
        destination: Some(FileId(2)),
        context_name: "".to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "two.h".to_string(),
        resolved_path: Some(PathBuf::from("two.h")),
        language: LanguageId::cpp(),
    });

    let placements = [("a", 1), ("b", 1), ("c", 2), ("d", 2), ("e", 3), ("x", 3)];
    let mut symbols = BTreeMap::new();
    let mut file_symbols: BTreeMap<FileId, Vec<SymbolId>> = BTreeMap::new();
    for (id, file) in placements {
        symbols.insert(SymbolId::new(id), record(id, file));
        file_symbols
            .entry(FileId(file))
            .or_default()
            .push(SymbolId::new(id));
    }
    let model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols,
        edges: vec![
            semantic_edge("a", "b"),
            semantic_edge("b", "c"),
            semantic_edge("d", "c"),
        ],
        tu_usage: BTreeMap::new(),
        file_symbols,
        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    (graph, model)
}

fn ids(names: &[&str]) -> Vec<SymbolId> {
    names.iter().map(|name| SymbolId::new(*name)).collect()
}

fn id_set(names: &[&str]) -> HashSet<SymbolId> {
    ids(names).into_iter().collect()
}

fn capped_memo(semantic_cap: usize, physical_cap: usize) -> LoadMemo {
    LoadMemo::with_closures_and_capacity(HashMap::new(), semantic_cap, physical_cap)
}

#[test]
fn full_semantic_memo_serves_uncached_seeds_with_one_shared_walk() {
    let (_graph, model) = fixture();
    let memo = capped_memo(1, 0);
    memo.cached_single_seed_closure(&model, &SymbolId::new("x"));
    assert_eq!(memo.semantic_closure_cache_len(), 1, "cap reached");
    let walks_before = memo.semantic_bfs_traversal_count();

    let closure = memo.get_semantic_closure_of_symbols(&model, &ids(&["a", "d", "e"]));

    assert_eq!(closure, id_set(&["a", "b", "c", "d", "e"]));
    assert_eq!(
        memo.semantic_bfs_traversal_count() - walks_before,
        1,
        "three uncached seeds on a full memo must share ONE walk"
    );
    assert_eq!(memo.semantic_closure_cache_len(), 1, "full memo admits nothing");
}

#[test]
fn full_semantic_memo_unions_cached_hits_before_the_shared_walk() {
    let (_graph, model) = fixture();
    let memo = capped_memo(2, 0);
    memo.cached_single_seed_closure(&model, &SymbolId::new("b"));
    memo.cached_single_seed_closure(&model, &SymbolId::new("x"));
    let walks_before = memo.semantic_bfs_traversal_count();

    let closure = memo.get_semantic_closure_of_symbols(&model, &ids(&["a", "b", "d"]));

    assert_eq!(closure, id_set(&["a", "b", "c", "d"]));
    assert_eq!(
        memo.semantic_bfs_traversal_count() - walks_before,
        1,
        "the cached seed is a hit; the two misses share one walk"
    );
}

#[test]
fn shared_walk_matches_uncapped_reference_for_every_seed_subset() {
    let (_graph, model) = fixture();
    let all = ["a", "b", "c", "d", "e", "x"];
    for mask in 1u32..(1 << all.len()) {
        let seeds: Vec<&str> = (0..all.len())
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| all[bit])
            .collect();
        let reference = LoadMemo::new().get_semantic_closure_of_symbols(&model, &ids(&seeds));
        let full = capped_memo(0, 0).get_semantic_closure_of_symbols(&model, &ids(&seeds));
        assert_eq!(full, reference, "seed subset {seeds:?}");
    }
}

#[test]
fn shared_expansion_entry_matches_by_id_entry() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let expansion = analyzer.physical_expansion(FileId(1));
    for name in ["a", "b"] {
        let symbol_id = SymbolId::new(name);
        let shared = analyzer
            .analyze_symbol_load_with_expansion(&symbol_id, &expansion)
            .expect("shared-expansion analysis");
        let by_id = analyzer
            .analyze_symbol_load_by_id(&symbol_id)
            .expect("by-id analysis");
        assert_eq!(shared, by_id, "symbol {name}");
    }
}

#[test]
fn shared_expansion_is_expanded_once_for_all_symbols_of_a_file() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::with_memo(&graph, &model, Arc::new(capped_memo(0, 0)));
    let expansion = analyzer.physical_expansion(FileId(1));
    for name in ["a", "b"] {
        analyzer
            .analyze_symbol_load_with_expansion(&SymbolId::new(name), &expansion)
            .expect("shared-expansion analysis");
    }
    assert_eq!(
        analyzer.physical_expansion_count(),
        1,
        "two symbols of one file must reuse one expansion even with the memo off"
    );
}

#[test]
fn shared_expansion_rejects_a_symbol_from_another_file() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let expansion = analyzer.physical_expansion(FileId(1));
    let error = analyzer
        .analyze_symbol_load_with_expansion(&SymbolId::new("c"), &expansion)
        .expect_err("c is defined in file 2, not file 1");
    assert_eq!(
        error.to_string(),
        "Expansion is for a different defining file than symbol: c"
    );
}

#[test]
fn shared_expansion_keeps_the_by_id_miss_errors() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let expansion = analyzer.physical_expansion(FileId(1));
    let error = analyzer
        .analyze_symbol_load_with_expansion(&SymbolId::new("absent"), &expansion)
        .expect_err("unregistered id must error");
    assert_eq!(error.to_string(), "Symbol record not found for: absent");
}

/// Counts-only symbol entry (2026-11-18 boost memory fix): the six scalars
/// the persisted report keeps, with the never-read detail vectors empty -
/// the exact shape a reloaded `report.bin` row already has.
#[test]
fn counts_entry_matches_full_metric_scalars_with_no_detail() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    for (name, file) in [("a", 1), ("b", 1), ("c", 2), ("e", 3)] {
        let symbol_id = SymbolId::new(name);
        let expansion = analyzer.physical_expansion(FileId(file));
        let counts = analyzer
            .analyze_symbol_load_counts_with_expansion(&symbol_id, &expansion)
            .expect("counts analysis");
        let full = analyzer
            .analyze_symbol_load_by_id(&symbol_id)
            .expect("full analysis");
        assert_eq!(counts, full.summary_only(), "symbol {name}");
        assert!(counts.semantic_symbols.is_empty() && counts.collateral_sources.is_empty());
    }
}

#[test]
fn summary_only_keeps_scalars_and_drops_detail() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let full = analyzer
        .analyze_symbol_load_by_id(&SymbolId::new("a"))
        .expect("full analysis");
    let summary = full.summary_only();
    assert_eq!(
        (summary.semantic_load, summary.include_load, summary.collateral_load),
        (full.semantic_load, full.include_load, full.collateral_load)
    );
    assert_eq!(summary.total_load, full.total_load);
    assert_eq!(summary.target_name, full.target_name);
    assert_eq!(summary.target_file, full.target_file);
    assert!(summary.semantic_symbols.is_empty());
    assert!(summary.collateral_symbols.is_empty());
    assert!(summary.semantic_sources.is_empty());
    assert!(summary.collateral_sources.is_empty());
    assert!(summary.largest_collateral_source.is_none());
}

#[test]
fn counts_entry_keeps_the_mismatch_and_miss_errors() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let expansion = analyzer.physical_expansion(FileId(1));
    let mismatch = analyzer
        .analyze_symbol_load_counts_with_expansion(&SymbolId::new("c"), &expansion)
        .expect_err("c is defined in file 2");
    assert_eq!(
        mismatch.to_string(),
        "Expansion is for a different defining file than symbol: c"
    );
    let missing = analyzer
        .analyze_symbol_load_counts_with_expansion(&SymbolId::new("absent"), &expansion)
        .expect_err("unregistered id");
    assert_eq!(missing.to_string(), "Symbol record not found for: absent");
}

/// File-level required headers (2026-11-18 boost justification fix): the
/// justification payload and `extraction_impact` only ever consumed the FILE
/// IDS of `required_headers_for_symbols`, yet each call materialized and
/// memoized whole symbol closures. The file-level entry must return exactly
/// those ids for every seed subset.
#[test]
fn required_header_files_match_required_headers_ids_for_every_seed_subset() {
    let (graph, model) = fixture();
    let all = ["a", "b", "c", "d", "e", "x", "absent"];
    for mask in 1u32..(1 << all.len()) {
        let seeds: Vec<&str> = (0..all.len())
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| all[bit])
            .collect();
        let analyzer = LoadAnalyzer::new(&graph, &model);
        let expected: HashSet<FileId> = analyzer
            .required_headers_for_symbols(&ids(&seeds))
            .into_iter()
            .map(|(file_id, _, _)| file_id)
            .collect();
        let fresh = LoadAnalyzer::new(&graph, &model);
        assert_eq!(
            fresh.required_header_files_for_symbols(&ids(&seeds)),
            expected,
            "seed subset {seeds:?}"
        );
    }
}

#[test]
fn required_header_files_reach_across_files_and_exclude_seed_files() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    assert_eq!(
        analyzer.required_header_files_for_symbols(&ids(&["a"])),
        HashSet::from([FileId(2)]),
        "a -> b -> c: c lives in two.h; one.h is the seed's own file"
    );
}

#[test]
fn required_header_files_memoize_per_seed_without_growing_symbol_closures() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    analyzer.required_header_files_for_symbols(&ids(&["a", "d"]));
    let walks_after_first = analyzer.semantic_bfs_traversal_count();
    assert_eq!(
        analyzer.semantic_closure_cache_len(),
        0,
        "file-level entry must not retain whole symbol closures"
    );
    analyzer.required_header_files_for_symbols(&ids(&["d", "a"]));
    assert_eq!(
        analyzer.semantic_bfs_traversal_count(),
        walks_after_first,
        "repeat seeds are served from the per-seed file memo"
    );
}

/// Allocation-free visitor (2026-11-18 boost heap-contention fix): visits
/// exactly the owned entry's ids, each once, for every seed subset, with
/// one analyzer reused across calls so per-thread scratch never leaks state
/// from one call into the next.
#[test]
fn required_header_file_visitor_matches_owned_set_for_every_seed_subset() {
    let (graph, model) = fixture();
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let all = ["a", "b", "c", "d", "e", "x", "absent"];
    for mask in 1u32..(1 << all.len()) {
        let seeds: Vec<&str> = (0..all.len())
            .filter(|bit| mask & (1 << bit) != 0)
            .map(|bit| all[bit])
            .collect();
        let expected = analyzer.required_header_files_for_symbols(&ids(&seeds));
        let mut visited: Vec<FileId> = Vec::new();
        analyzer.for_each_required_header_file(&ids(&seeds), |file_id| visited.push(file_id));
        let visited_len = visited.len();
        let visited_set: HashSet<FileId> = visited.into_iter().collect();
        assert_eq!(visited_len, visited_set.len(), "each file once: {seeds:?}");
        assert_eq!(visited_set, expected, "seed subset {seeds:?}");
    }
}
