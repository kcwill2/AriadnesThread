//! B1 memo-sharing suite (plan 1790608181551): analyzers are cheap handles
//! over one shared `Arc<LoadMemo>`; a second handle must reuse everything
//! the first warmed. Split from `load_perf_tests.rs` to keep both files
//! under the R2 line limit (mirrors the explain/characterization split).

use super::*;
use super::impact::{setup_impact_fixture, D_H, FAT, MAIN};
use super::perf::setup_concurrency_fixture;

/// B1(a) compile contract: the memo is the object shared across UI requests
/// and rayon workers, so it must be `Send + Sync` by construction (I4). A
/// non-thread-safe field creeping in (e.g. `RefCell`) fails to compile —
/// exactly the job of this test, mirroring the analyzer assert in the
/// perf suite.
#[test]
fn test_load_memo_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LoadMemo>();
}

/// B1(b) shared-memo zero re-work: analyzer 1 warms the memo (file + symbol
/// analyses + an `extraction_impact`, exercising closures, semantic BFS,
/// physical expansion and include adjacency). Analyzer 2 over the SAME memo
/// must add ZERO semantic BFS, ZERO physical expansions and ZERO adjacency
/// builds (the counters live on the shared memo, so both handles observe the
/// unchanged warmed totals), while every metric equals the warm analyzer's.
#[test]
fn test_shared_memo_second_analyzer_adds_zero_cache_work() {
    let (ig, model) = setup_impact_fixture(&[(MAIN, D_H, "d.h")], false);
    let paths: Vec<&PathBuf> = ig.file_paths.values().collect();
    let names: Vec<String> = model.symbols.values().map(|r| r.name.clone()).collect();
    let seeds = [SymbolId::new("FatA"), SymbolId::new("FatB")];
    assert!(!paths.is_empty() && !names.is_empty());

    let warm = LoadAnalyzer::new(&ig, &model);
    let warm_files: Vec<LoadMetric> = paths
        .iter()
        .map(|p| warm.analyze_file_load(p).expect("warm file load"))
        .collect();
    let warm_symbols: Vec<LoadMetric> = names
        .iter()
        .map(|n| warm.analyze_symbol_load(n).expect("warm symbol load"))
        .collect();
    let warm_impact = warm
        .extraction_impact(MAIN, FAT, &seeds)
        .expect("warm impact");

    let bfs = warm.memo().semantic_bfs_traversal_count();
    let expansions = warm.memo().physical_expansion_count();
    let adjacency = warm.memo().include_adjacency_build_count();
    assert!(
        bfs > 0 && expansions > 0 && adjacency > 0,
        "warming must have exercised all three caches, got bfs={bfs} \
         expansions={expansions} adjacency={adjacency}"
    );

    let cold = LoadAnalyzer::with_memo(&ig, &model, warm.memo());
    let cold_files: Vec<LoadMetric> = paths
        .iter()
        .map(|p| cold.analyze_file_load(p).expect("shared-memo file load"))
        .collect();
    let cold_symbols: Vec<LoadMetric> = names
        .iter()
        .map(|n| cold.analyze_symbol_load(n).expect("shared-memo symbol load"))
        .collect();
    let cold_impact = cold
        .extraction_impact(MAIN, FAT, &seeds)
        .expect("shared-memo impact");

    assert_eq!(
        cold.memo().semantic_bfs_traversal_count(),
        bfs,
        "analyzer 2 over the shared memo must add zero semantic BFS traversals"
    );
    assert_eq!(
        cold.memo().physical_expansion_count(),
        expansions,
        "analyzer 2 over the shared memo must add zero physical expansions"
    );
    assert_eq!(
        cold.memo().include_adjacency_build_count(),
        adjacency,
        "analyzer 2 over the shared memo must add zero include-adjacency builds"
    );
    assert_eq!(cold_files, warm_files, "shared-memo file metrics must match");
    assert_eq!(
        cold_symbols, warm_symbols,
        "shared-memo symbol metrics must match"
    );
    assert_eq!(cold_impact, warm_impact, "shared-memo impact must match");
    assert!(
        std::sync::Arc::ptr_eq(&cold.memo(), &warm.memo()),
        "with_memo must share the one memo, not clone its contents"
    );
}

/// B1(c) shared-memo concurrency (I1/I4): two analyzer handles over ONE memo
/// driven through rayon `par_iter` must produce byte-identical metrics to a
/// sequential reference — both for the symbol path and the file path — and
/// the run itself (no deadlock) is the lock.
#[test]
fn test_concurrent_shared_memo_handles_match_sequential() {
    use rayon::prelude::*;

    let (ig, model) = setup_concurrency_fixture();
    let symbol_names: Vec<String> = model.symbols.values().map(|r| r.name.clone()).collect();
    let paths: Vec<PathBuf> = ig.file_paths.values().cloned().collect();
    assert!(symbol_names.len() >= 8 && paths.len() >= 4);

    let reference = LoadAnalyzer::new(&ig, &model);
    let sequential_symbols: Vec<LoadMetric> = symbol_names
        .iter()
        .map(|n| reference.analyze_symbol_load(n).expect("sequential symbol load"))
        .collect();
    let sequential_files: Vec<LoadMetric> = paths
        .iter()
        .map(|p| reference.analyze_file_load(p).expect("sequential file load"))
        .collect();

    let memo = std::sync::Arc::new(LoadMemo::new());
    let handle_a = LoadAnalyzer::with_memo(&ig, &model, std::sync::Arc::clone(&memo));
    let handle_b = LoadAnalyzer::with_memo(&ig, &model, std::sync::Arc::clone(&memo));

    let concurrent_symbols: Vec<(usize, LoadMetric)> = (0..symbol_names.len())
        .into_par_iter()
        .map(|index| {
            let analyzer = if index % 2 == 0 { &handle_a } else { &handle_b };
            (
                index,
                analyzer
                    .analyze_symbol_load(&symbol_names[index])
                    .expect("shared-memo concurrent symbol load"),
            )
        })
        .collect();
    for (index, metric) in concurrent_symbols {
        assert_eq!(
            metric, sequential_symbols[index],
            "shared-memo concurrent metric for {} must equal the sequential reference",
            symbol_names[index]
        );
    }

    let concurrent_files: Vec<(usize, LoadMetric)> = (0..paths.len())
        .into_par_iter()
        .map(|index| {
            let analyzer = if index % 2 == 0 { &handle_b } else { &handle_a };
            (
                index,
                analyzer
                    .analyze_file_load(&paths[index])
                    .expect("shared-memo concurrent file load"),
            )
        })
        .collect();
    for (index, metric) in concurrent_files {
        assert_eq!(
            metric, sequential_files[index],
            "shared-memo concurrent file metric for {} must equal the sequential reference",
            paths[index].display()
        );
    }
}
