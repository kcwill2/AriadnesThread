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

/// Runs every stage through a fresh cold pipeline and returns the shared
/// handle to the load report exactly as `ensure_load` built it (not
/// re-read from disk).
fn cold_load_report(repo_path: &Path) -> Arc<LoadReport> {
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

/// Structural guard for the Arc-shared load cache (plan load-tail A3):
/// right after a cold `ensure_load` the cached Arc must have strong count
/// exactly 1 — the report was MOVED into the cache (`Arc::new(report)`,
/// not `report.clone()`), nothing else holds it. The public
/// `cached_load`/`get_or_build_load` hand out a SHARED `Arc` handle (a
/// refcount bump, no deep copy): while a handle is alive, cache + handle
/// = 2, and dropping the handle returns the cache to 1. Content equality
/// against the cache is locked alongside.
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

    let handle = pipeline.cached_load().unwrap();
    assert_eq!(
        std::sync::Arc::strong_count(pipeline.load_cache.as_ref().unwrap()),
        2,
        "a live cached_load handle means cache + handle = 2 references"
    );
    let built = pipeline.get_or_build_load().unwrap();
    assert_eq!(
        std::sync::Arc::strong_count(pipeline.load_cache.as_ref().unwrap()),
        3,
        "two live handles mean cache + 2 handles = 3 references"
    );
    assert_eq!(handle.repository_root, built.repository_root);
    assert_eq!(
        handle.symbol_metrics, built.symbol_metrics,
        "hand-outs must equal the cached content"
    );
    assert_eq!(
        handle.file_metrics, built.file_metrics,
        "repeated hand-outs must stay equal to the cache"
    );
    assert!(
        Arc::ptr_eq(&handle, &built),
        "hand-outs must share the ONE cached allocation, never deep copies"
    );
    drop(handle);
    drop(built);
    assert_eq!(
        std::sync::Arc::strong_count(pipeline.load_cache.as_ref().unwrap()),
        1,
        "dropping every handle must return the cache to a lone reference"
    );
}

/// A3: `take_include_graph` must MOVE the pair out of the cache — a
/// subsequent `cached_include_graph` read hits the documented missing-cache
/// error (asserted as an error, never a panic), and the taken values must
/// carry the full map a cold `ensure_include_graph` produces on the same
/// fixture (a clone-less hand-out loses nothing).
#[test]
fn test_take_include_graph_empties_the_cache() {
    let dir = setup_repo();
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    pipeline.ensure_include_graph().unwrap();

    let (graph, closures) = pipeline.take_include_graph().unwrap();
    assert!(
        !graph.file_paths.is_empty() && !closures.is_empty(),
        "take after ensure must hand out populated stage data"
    );

    assert!(
        pipeline.include_cache.is_none(),
        "take must empty the cache"
    );
    assert_missing_cache_error(
        pipeline.cached_include_graph(),
        "include stage cache must exist",
    );
}

/// A3: taking before the stage ran must surface the documented missing-
/// cache error instead of building or panicking.
#[test]
fn test_take_include_graph_before_ensure_errors() {
    let dir = setup_repo();
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    assert_missing_cache_error(
        pipeline.take_include_graph(),
        "include stage cache must exist",
    );
}

/// A3: the taken closures are exactly the stage closures — compare the
/// moved-out map against the content a second, cold pipeline computes from
/// the same fixture, so the hand-out is proven loss-free (move, not a
/// partial drain).
#[test]
fn test_take_include_graph_moves_the_full_closures_map() {
    let dir = setup_repo();
    let mut cold = Pipeline::new(dir.path(), None).unwrap();
    cold.ensure_include_graph().unwrap();
    let (_graph, taken_closures) = cold.take_include_graph().unwrap();
    drop(cold);

    let mut reference = Pipeline::new(dir.path(), None).unwrap();
    reference.ensure_include_graph().unwrap();
    let expected_closures = reference.cached_include_graph().unwrap().1;

    let mut taken_keys: Vec<_> = taken_closures.keys().copied().collect();
    let mut expected_keys: Vec<_> = expected_closures.keys().copied().collect();
    taken_keys.sort_by_key(|id| id.0);
    expected_keys.sort_by_key(|id| id.0);
    assert_eq!(
        taken_keys, expected_keys,
        "take must hand out every closure key the cache held"
    );
    for (id, expected) in &expected_closures {
        assert_eq!(
            taken_closures.get(id),
            Some(expected),
            "closure for {id:?} must be identical after the move"
        );
    }
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

/// A4: the load stage must bracket its work with honest milestones. On a
/// cold rebuild the emitted sequence must be: `SymbolsAnalyzed { symbols > 0 }`
/// strictly before the first `CollateralFile` event (the symbol pass runs
/// before any file event), a terminal `CollateralFile { done == total }`
/// once the file loop has fully collected, and `ArtifactsSaved` LAST (after
/// `save_load_artifacts` returns, so a client never sees the save claimed
/// before the bytes are on disk).
#[test]
fn test_load_milestone_event_order() {
    let dir = setup_repo();
    let events: Arc<Mutex<Vec<ProgressEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let recorder = events.clone();
    let mut pipeline = Pipeline::new(dir.path(), None)
        .unwrap()
        .with_progress(move |event| recorder.lock().unwrap().push(event.clone()));
    pipeline.ensure_load().unwrap();

    let recorded = events.lock().unwrap();
    let symbols_at = recorded.iter().position(|event| {
        matches!(
            event,
            ProgressEvent::LoadMilestone(LoadMilestone::SymbolsAnalyzed { symbols }) if *symbols > 0
        )
    });
    let first_file_at = recorded
        .iter()
        .position(|event| matches!(event, ProgressEvent::CollateralFile { .. }));
    let artifacts_at = recorded
        .iter()
        .position(|event| matches!(event, ProgressEvent::LoadMilestone(LoadMilestone::ArtifactsSaved)));

    let symbols_at = symbols_at.expect("cold rebuild must emit SymbolsAnalyzed with symbols > 0");
    let first_file_at = first_file_at.expect("cold rebuild must emit CollateralFile events");
    assert!(
        symbols_at < first_file_at,
        "SymbolsAnalyzed must arrive strictly before the first CollateralFile, got order {symbols_at} vs {first_file_at}"
    );

    let last_file_at = recorded
        .iter()
        .rposition(|event| matches!(event, ProgressEvent::CollateralFile { .. }))
        .expect("terminal CollateralFile event must arrive");
    match &recorded[last_file_at] {
        ProgressEvent::CollateralFile { done, total } => {
            assert_eq!(done, total, "last CollateralFile must be the terminal done == total event");
        }
        other => panic!("expected CollateralFile at {last_file_at}, got {other:?}"),
    }

    let artifacts_at = artifacts_at.expect("save_load_artifacts completion must emit ArtifactsSaved");
    assert!(
        artifacts_at > last_file_at,
        "ArtifactsSaved must follow the terminal CollateralFile event"
    );
    assert_eq!(
        artifacts_at,
        recorded.len() - 1,
        "ArtifactsSaved must be the LAST load-stage event"
    );
}

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
