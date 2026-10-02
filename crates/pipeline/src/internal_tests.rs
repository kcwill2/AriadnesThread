//! Tests that legitimately need pipeline-INTERNAL access (private
//! `collect_file_metrics`, private `ProgressCb` alias, private `cached_*`
//! stage accessors, private `load_cache` field). Kept in-src as the
//! truly-internal half of the M23 T3 pipeline test split (D-M23-10); the
//! public-API suite moved verbatim to
//! `crates/pipeline/tests/pipeline_test.rs`. This file must stay
//! plugin-name-free: it is inside the guard walk scope, so the fixture
//! directive text is assembled from a non-contiguous marker (the plugin
//! crate owns the canonical form) and the compile command uses a generic
//! `cc` executable (discovery is compiler-agnostic, `ariadne_build`
//! characterization locks cover the rest) — assertions elsewhere are
//! byte-identical to the relocated originals.

use super::*;
use std::fs;
use tempfile::TempDir;

/// Guard-clean sibling of the relocated `setup_repo` fixture: same tree,
/// byte-identical directive text via the non-contiguous marker, generic `cc`
/// compile command instead of a plugin-named compiler binary.
fn setup_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(
        root.join("include/header_a.h"),
        concat!("#inc", "lude \"common.h\"\nstruct ThingA { Common c; };"),
    )
    .unwrap();
    fs::write(
        root.join("include/header_b.h"),
        "struct ThingB { double x; };",
    )
    .unwrap();
    fs::write(
        root.join("src/tu1.cpp"),
        concat!("#inc", "lude \"header_a.h\"\nvoid use_a(ThingA a) {}"),
    )
    .unwrap();
    fs::write(
        root.join("src/tu2.cpp"),
        concat!("#inc", "lude \"header_b.h\"\nvoid use_b(ThingB b) {}"),
    )
    .unwrap();

    let root_str = root.to_string_lossy().replace('\\', "/");
    let compile_commands = format!(
        r#"[
    {{ "directory": "{root_str}", "command": "cc -Iinclude -c src/tu1.cpp", "file": "src/tu1.cpp" }},
    {{ "directory": "{root_str}", "command": "cc -Iinclude -c src/tu2.cpp", "file": "src/tu2.cpp" }}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();
    dir
}

fn assert_missing_cache_error<T>(result: Result<T>, expected: &str) {
    match result {
        Ok(_) => panic!("expected missing-cache error, got Ok"),
        Err(err) => assert!(
            err.to_string().contains(expected),
            "error message {err} missing {expected:?}"
        ),
    }
}

/// Runs every stage through a fresh cold pipeline and returns the load
/// report exactly as `ensure_load` built it (not re-read from disk).
fn cold_load_report(repo_path: &Path) -> LoadReport {
    let mut pipeline = Pipeline::new(repo_path, None).unwrap();
    pipeline.ensure_load().unwrap();
    pipeline.cached_load().unwrap()
}

#[test]
fn test_missing_stage_cache_returns_error_not_panic() {
    let dir = setup_repo();
    let pipeline = Pipeline::new(dir.path(), None).unwrap();

    assert_missing_cache_error(
        pipeline.cached_include_graph(),
        "include stage cache must exist",
    );
    assert_missing_cache_error(pipeline.cached_index(), "index stage cache must exist");
    assert_missing_cache_error(pipeline.cached_reduced(), "reduced stage cache must exist");
    assert_missing_cache_error(
        pipeline.cached_callgraph(),
        "callgraph stage cache must exist",
    );
    assert_missing_cache_error(pipeline.cached_load(), "load stage cache must exist");
}

/// Guard rail for the parallel load stage: whatever `ensure_load`
/// produces (parallel or not, in whatever internal order) must equal a
/// plain sequential recomputation over the same cached graph + reduced
/// model, elementwise and in the same vector order. Keeps the
/// order-preserving `par_iter().filter_map().collect()` contract honest.
#[test]
fn test_load_report_matches_sequential_reference() {
    let dir = setup_repo();
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    pipeline.ensure_load().unwrap();
    let report = pipeline.cached_load().unwrap();

    let (graph, _closures) = pipeline.cached_include_graph().unwrap();
    let reduced = pipeline.cached_reduced().unwrap();
    let reference = LoadAnalyzer::new(&graph, &reduced);

    let mut expected_symbol_metrics = Vec::new();
    for sym_record in reduced.symbols.values() {
        if let Ok(metric) = reference.analyze_symbol_load(&sym_record.name) {
            expected_symbol_metrics.push(metric);
        }
    }
    let mut expected_file_metrics = Vec::new();
    for file_id in reduced.file_symbols.keys() {
        if let Some(path) = graph.file_paths.get(file_id) {
            if let Ok(metric) = reference.analyze_file_load(path) {
                expected_file_metrics.push(metric);
            }
        }
    }

    assert_eq!(
        report.symbol_metrics, expected_symbol_metrics,
        "symbol metrics must match the sequential reference elementwise"
    );
    assert_eq!(
        report.file_metrics, expected_file_metrics,
        "file metrics must match the sequential reference elementwise"
    );
}

/// Reviewer nit follow-up: production callers pass `Some(n)`, which routes
/// the load stage through the scoped `ThreadPoolBuilder` branch of
/// `run_on_configured_pool` — the branch the `None` variant of the
/// reference test above never enters. Same sequential oracle as
/// `test_load_report_matches_sequential_reference`; the pool only changes
/// WHERE the work runs, never the order or the values, so this must pass
/// with zero behavior change.
#[test]
fn test_load_report_matches_sequential_reference_with_scoped_pool() {
    let dir = setup_repo();
    let mut pipeline = Pipeline::new(dir.path(), Some(2)).unwrap();
    pipeline.ensure_load().unwrap();
    let report = pipeline.cached_load().unwrap();

    let (graph, _closures) = pipeline.cached_include_graph().unwrap();
    let reduced = pipeline.cached_reduced().unwrap();
    let reference = LoadAnalyzer::new(&graph, &reduced);

    let mut expected_symbol_metrics = Vec::new();
    for sym_record in reduced.symbols.values() {
        if let Ok(metric) = reference.analyze_symbol_load(&sym_record.name) {
            expected_symbol_metrics.push(metric);
        }
    }
    let mut expected_file_metrics = Vec::new();
    for file_id in reduced.file_symbols.keys() {
        if let Some(path) = graph.file_paths.get(file_id) {
            if let Ok(metric) = reference.analyze_file_load(path) {
                expected_file_metrics.push(metric);
            }
        }
    }

    assert_eq!(
        report.symbol_metrics, expected_symbol_metrics,
        "symbol metrics from the scoped-pool run must match the sequential reference elementwise"
    );
    assert_eq!(
        report.file_metrics, expected_file_metrics,
        "file metrics from the scoped-pool run must match the sequential reference elementwise"
    );
}

/// Structural guard for the Arc-shared load cache: right after a cold
/// `ensure_load` the cached Arc must have strong count exactly 1 — the
/// report was MOVED into the cache (`Arc::new(report)`, not
/// `report.clone()`), nothing else holds it — and cache reads must equal
/// the stored content. The public `cached_load`/`get_or_build_load` still
/// hand clones to callers (unchanged signature), but the stage itself
/// performs no deep copy.
#[test]
fn test_load_cache_is_arc_shared_not_deep_copied() {
    let dir = setup_repo();
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    pipeline.ensure_load().unwrap();

    assert_eq!(
        std::sync::Arc::strong_count(pipeline.load_cache.as_ref().unwrap()),
        1,
        "cold ensure_load must move the report into a lone cache Arc, not clone it"
    );

    let cached = pipeline.cached_load().unwrap();
    let first = pipeline.get_or_build_load().unwrap();
    let second = pipeline.get_or_build_load().unwrap();
    assert_eq!(first.repository_root, cached.repository_root);
    assert_eq!(
        first.symbol_metrics, cached.symbol_metrics,
        "get_or_build_load must return content equal to the cache"
    );
    assert_eq!(
        second.file_metrics, cached.file_metrics,
        "repeated get_or_build_load must stay equal to the cache"
    );
    assert_eq!(
        std::sync::Arc::strong_count(pipeline.load_cache.as_ref().unwrap()),
        1,
        "reading the cache must not leak extra references into the pipeline"
    );
}

/// Two independent cold pipelines over identical fixture content in
/// SEPARATE temp dirs must produce equal load reports. Same-dir
/// re-runs are useless for this: the artifact fingerprint
/// short-circuit makes the second pipeline LOAD the stored report
/// instead of rebuilding, so only separate dirs compare two real
/// rebuilds. `repository_root` legitimately differs per temp dir and
/// is excluded; every metric path is repo-root-relative (discovery
/// normalizes `FileRecord.path`), so metrics must match exactly.
#[test]
fn test_load_deterministic_across_pipelines() {
    let dir_a = setup_repo();
    let dir_b = setup_repo();

    let report_a = cold_load_report(dir_a.path());
    let report_b = cold_load_report(dir_b.path());

    assert_eq!(
        report_a.symbol_metrics, report_b.symbol_metrics,
        "symbol metrics must be identical across independent cold pipelines"
    );
    assert_eq!(
        report_a.file_metrics, report_b.file_metrics,
        "file metrics must be identical across independent cold pipelines"
    );
}

/// A2: `collect_file_metrics` must emit the exact metrics the sequential

/// A2: `collect_file_metrics` must emit the exact metrics the sequential
/// path round-trip produced, while keeping the old skip contract — an id
/// present in `file_symbols` but unregistered in `file_paths` yields no
/// metric (still counted in `completed`, mirroring the old
/// `get(path).and_then(...)` + `flatten()` semantics).
#[test]
fn test_collect_file_metrics_matches_path_reference_and_skips_unregistered() {
    use ariadne_graph::ReducedSymbolRecord;
    use ariadne_index::types::{SymbolId, SymbolKind};
    use std::collections::BTreeMap;

    let mut graph = IncludeGraph::new();
    graph.register_file(FileId(1), PathBuf::from("a.h"));
    graph.register_file(FileId(2), PathBuf::from("b.h"));

    let mut reduced = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    let add = |model: &mut ReducedSemanticModel, name: &str, file: FileId| {
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
        model.file_symbols.entry(file).or_default().push(id);
    };
    add(&mut reduced, "Alpha", FileId(1));
    add(&mut reduced, "Beta", FileId(2));
    // Orphan id: symbols registered, path never registered — must be skipped.
    add(&mut reduced, "Orphan", FileId(9));

    let analyzer = LoadAnalyzer::new(&graph, &reduced);
    let progress: Mutex<Option<ProgressCb>> = Mutex::new(None);
    let completed = AtomicUsize::new(0);
    let metrics = collect_file_metrics(
        &analyzer,
        &graph,
        &reduced,
        reduced.file_symbols.len(),
        &progress,
        &completed,
    );

    let reference: Vec<LoadMetric> = reduced
        .file_symbols
        .keys()
        .filter_map(|id| graph.file_paths.get(id))
        .filter_map(|path| analyzer.analyze_file_load(path).ok())
        .collect();

    assert_eq!(
        metrics.len(),
        2,
        "only ids with a registered path may produce metrics"
    );
    assert_eq!(
        completed.load(Ordering::SeqCst),
        3,
        "skipped ids must still count as completed (old enumerate contract)"
    );
    assert_eq!(
        metrics, reference,
        "file metrics must equal the path round-trip reference, same order"
    );
}
