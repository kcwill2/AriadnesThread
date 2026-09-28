use super::*;
use std::fs;
use tempfile::TempDir;

fn setup_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(
        root.join("include/header_a.h"),
        "#include \"common.h\"\nstruct ThingA { Common c; };",
    )
    .unwrap();
    fs::write(
        root.join("include/header_b.h"),
        "struct ThingB { double x; };",
    )
    .unwrap();
    fs::write(
        root.join("src/tu1.cpp"),
        "#include \"header_a.h\"\nvoid use_a(ThingA a) {}",
    )
    .unwrap();
    fs::write(
        root.join("src/tu2.cpp"),
        "#include \"header_b.h\"\nvoid use_b(ThingB b) {}",
    )
    .unwrap();

    let root_str = root.to_string_lossy().replace('\\', "/");
    let compile_commands = format!(
        r#"[
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/tu1.cpp", "file": "src/tu1.cpp" }},
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/tu2.cpp", "file": "src/tu2.cpp" }}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();
    dir
}

#[test]
fn test_cold_then_warm_all_stages() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    // Cold run: everything rebuilds and persists.
    let mut cold = Pipeline::new(&root, None).unwrap();
    cold.ensure_index().unwrap();
    cold.ensure_include_graph().unwrap();
    cold.ensure_reduced().unwrap();
    cold.ensure_callgraph().unwrap();
    cold.ensure_load().unwrap();
    assert_eq!(cold.stats.reanalyzed_tus, 2);
    assert_eq!(cold.stats.reused_tus, 0);
    for stage in [
        STAGE_INCLUDE_GRAPH,
        STAGE_INDEX,
        STAGE_REDUCED,
        STAGE_CALLGRAPH,
        STAGE_LOAD,
    ] {
        assert_eq!(cold.stats.stage_reuse(stage), Some(StageReuse::Rebuilt));
    }

    // Warm run: every stage loads, zero re-parses.
    let mut warm = Pipeline::new(&root, None).unwrap();
    warm.ensure_index().unwrap();
    warm.ensure_include_graph().unwrap();
    warm.ensure_reduced().unwrap();
    warm.ensure_callgraph().unwrap();
    warm.ensure_load().unwrap();
    assert_eq!(
        warm.stats.reused_tus, 0,
        "loaded index means zero parse work"
    );
    assert_eq!(warm.stats.reanalyzed_tus, 0);
    for stage in [
        STAGE_INCLUDE_GRAPH,
        STAGE_INDEX,
        STAGE_REDUCED,
        STAGE_CALLGRAPH,
        STAGE_LOAD,
    ] {
        assert_eq!(
            warm.stats.stage_reuse(stage),
            Some(StageReuse::Loaded),
            "stage {} should load warm",
            stage
        );
    }
}

#[test]
fn test_header_edit_invalidates_dependents_only() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let mut cold = Pipeline::new(&root, None).unwrap();
    cold.ensure_index().unwrap();
    cold.ensure_include_graph().unwrap();
    cold.ensure_reduced().unwrap();

    // Edit common.h: tu1 depends on it via header_a.h, tu2 does not.
    fs::write(
        root.join("include/common.h"),
        "struct Common { int id; int extra; };",
    )
    .unwrap();

    let mut next = Pipeline::new(&root, None).unwrap();
    next.ensure_index().unwrap();
    next.ensure_include_graph().unwrap();
    assert_eq!(next.stats.reanalyzed_tus, 1, "only tu1 should re-parse");
    assert_eq!(next.stats.reused_tus, 1, "tu2 shard stays valid");
    assert_eq!(
        next.stats.stage_reuse(STAGE_INDEX),
        Some(StageReuse::Rebuilt)
    );
    assert_eq!(
        next.stats.stage_reuse(STAGE_INCLUDE_GRAPH),
        Some(StageReuse::Rebuilt)
    );
}

#[test]
fn test_corrupt_artifact_is_not_trusted() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let mut cold = Pipeline::new(&root, None).unwrap();
    cold.ensure_index().unwrap();

    // Corrupt the fingerprint in the index manifest.
    let manifest_path = root.join("artifacts").join("index").join("manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let mut corrupt = manifest.as_object().unwrap().clone();
    corrupt.insert(
        "input_fingerprint".into(),
        serde_json::Value::String("0".repeat(64)),
    );
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&corrupt).unwrap(),
    )
    .unwrap();

    let mut next = Pipeline::new(&root, None).unwrap();
    next.ensure_index().unwrap();
    assert_eq!(
        next.stats.stage_reuse(STAGE_INDEX),
        Some(StageReuse::Rebuilt)
    );
    // The rebuilt index artifact is valid again for subsequent runs.
    let mut after = Pipeline::new(&root, None).unwrap();
    after.ensure_index().unwrap();
    assert_eq!(
        after.stats.stage_reuse(STAGE_INDEX),
        Some(StageReuse::Loaded)
    );
}

#[test]
fn test_production_stats_captured_only_on_workers_rebuild() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let mut cold = Pipeline::new(&root, Some(2)).unwrap();
    cold.ensure_index().unwrap();
    let captured = cold
        .production_stats
        .clone()
        .expect("workers run captures real stats");
    assert_eq!(captured.total_tus, 2);
    assert_eq!(captured.progress_completed, 2);
    assert_eq!(captured.progress_total, 2);
    assert!(captured.peak_memory_bytes > 0);

    // Warm run: the index stage loads, no worker run measured anything.
    let mut warm = Pipeline::new(&root, Some(2)).unwrap();
    warm.ensure_index().unwrap();
    assert_eq!(
        warm.stats.stage_reuse(STAGE_INDEX),
        Some(StageReuse::Loaded)
    );
    assert!(
        warm.production_stats.is_none(),
        "loaded stage must not invent measurements"
    );
}

#[test]
fn test_rehash_inputs_matches_pipeline_fps_without_side_effects() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let expected = Pipeline::new(&root, None).unwrap().fps.clone();
    assert_eq!(Pipeline::rehash_inputs(&root).unwrap(), expected);

    // Pure check: no artifact open/save side effects.
    fs::remove_dir_all(root.join("artifacts")).unwrap();
    let rehashed = Pipeline::rehash_inputs(&root).unwrap();
    assert_eq!(rehashed, expected);
    assert!(!root.join("artifacts").exists());

    fs::write(root.join("include/header_b.h"), "struct ThingB { int x; };").unwrap();
    assert_ne!(Pipeline::rehash_inputs(&root).unwrap(), expected);
}

#[test]
fn test_run_stats_format_summary() {
    let mut stats = RunStats::default();
    stats.record(STAGE_INDEX, StageReuse::Rebuilt);
    stats.record(STAGE_REDUCED, StageReuse::Loaded);
    stats.record(STAGE_INCLUDE_GRAPH, StageReuse::Loaded);
    stats.reused_tus = 4;
    stats.reanalyzed_tus = 1;
    let summary = stats.format_summary();
    assert_eq!(
        summary,
        "4 TUs reused, 1 reanalyzed; stages loaded: reduced, include-graph; rebuilt: index"
    );
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

#[test]
fn test_hash_helper_consistency() {
    // The contexts fingerprint material must be order-stable.
    assert_eq!(
        ariadne_index::hash_parts(&["a", "b"]),
        ariadne_index::hash_parts(&["a", "b"])
    );
}

/// Runs every stage through a fresh cold pipeline and returns the load
/// report exactly as `ensure_load` built it (not re-read from disk).
fn cold_load_report(repo_path: &Path) -> LoadReport {
    let mut pipeline = Pipeline::new(repo_path, None).unwrap();
    pipeline.ensure_load().unwrap();
    pipeline.cached_load().unwrap()
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
