//! Cold `ensure_load` timing bench for plan `plans/LoadPerfImprovements.md`.
//!
//! Generates a synthetic C++ repo at the plan's Performance Targets scale
//! (1000+ symbols, 500+ files), runs index/include-graph/reduced untimed,
//! then times ONLY the collateral load stage on guaranteed-cold artifacts
//! (fresh temp dir, repo-local shard store). Run with:
//!   cargo test --release -p ariadne-pipeline --test load_perf_bench -- \
//!     --ignored --nocapture --test-threads=1

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;

use ariadne_graph::{LoadAnalyzer, LoadMetric};
use ariadne_report::LoadReport;

/// Headers on a 4-wide modular fanout (h_i includes h_(i+10..i+13) mod N),
/// cycle-tolerant: the closure walk is visited-guarded and the include graph
/// treats cycles as a first-class case.
const HEADER_COUNT: usize = 520;
const FANOUT_BASE: usize = 10;
const FANOUT_WIDTH: usize = 4;
/// Translation units, each including 5 spread headers (stride 103 is coprime
/// with HEADER_COUNT, so TU roots plus the strongly-connected header fanout
/// reach every header).
const TU_COUNT: usize = 40;
const TU_ROOT_STRIDE: usize = 103;
const TU_ROOT_COUNT: usize = 5;

/// Fanout targets of header `index`: h_(index+10)..h_(index+13) mod 520.
fn fanout_targets(index: usize) -> Vec<usize> {
    (0..FANOUT_WIDTH)
        .map(|offset| (index + FANOUT_BASE + offset) % HEADER_COUNT)
        .collect()
}

/// One stress header: 2 structs with cross-header pointer members plus 2
/// cross-calling free functions (4 symbols/header) in ~20 lines. Forward
/// pointer/reference uses keep the cycle-tolerant layout valid C++ even
/// when a cyclic `#include` is skipped by the pragma guard.
fn header_source(index: usize) -> String {
    let targets = fanout_targets(index);
    let mut source = String::from("#pragma once\n");
    for target in &targets {
        source.push_str(&format!("#include \"h{target}.h\"\n"));
    }
    source.push_str(&format!(
        "struct S{index} {{\n    S{t0}* pa;\n    S{t1}* pb;\n    long value;\n}};\n",
        t0 = targets[0],
        t1 = targets[1],
    ));
    source.push_str(&format!(
        "struct T{index} {{\n    S{t2}* pc;\n    S{t3}* pd;\n}};\n",
        t2 = targets[2],
        t3 = targets[3],
    ));
    source.push_str(&format!(
        "void f{index}(S{t0}* in) {{\n    g{index}(in);\n}}\n",
        t0 = targets[0],
    ));
    source.push_str(&format!(
        "void g{index}(S{t0}* in) {{\n    f{t1}(in);\n    f{t2}(in);\n}}\n",
        t0 = targets[0],
        t1 = targets[1],
        t2 = targets[2],
    ));
    source
}

fn write_headers(include_dir: &Path) {
    for index in 0..HEADER_COUNT {
        fs::write(
            include_dir.join(format!("h{index}.h")),
            header_source(index),
        )
        .unwrap();
    }
}

/// One translation unit: a struct holding pointers into 5 spread headers
/// plus one driver function calling across them.
fn tu_source(index: usize) -> String {
    let roots: Vec<usize> = (0..TU_ROOT_COUNT)
        .map(|slot| (index + slot * TU_ROOT_STRIDE) % HEADER_COUNT)
        .collect();
    let mut source = String::new();
    for root in &roots {
        source.push_str(&format!("#include \"h{root}.h\"\n"));
    }
    source.push_str(&format!("struct Tu{index} {{\n"));
    for (slot, root) in roots.iter().enumerate() {
        source.push_str(&format!("    S{root}* a{slot};\n"));
    }
    source.push_str("};\n");
    source.push_str(&format!(
        "void tu_run{index}() {{\n    f{r0}(0);\n    f{r2}(0);\n    g{r4}(0);\n}}\n",
        r0 = roots[0],
        r2 = roots[2],
        r4 = roots[4],
    ));
    source
}

/// compile_commands.json mirroring `setup_repo()` in the pipeline test mod:
/// forward-slash absolute directory, `clang++ -Iinclude -c src/tuN.cpp`.
fn write_compile_commands(root: &Path) {
    let root_str = root.to_string_lossy().replace('\\', "/");
    let mut entries = Vec::new();
    for index in 0..TU_COUNT {
        entries.push(format!(
            "    {{ \"directory\": \"{root_str}\", \"command\": \"clang++ -Iinclude -c src/tu{index}.cpp\", \"file\": \"src/tu{index}.cpp\" }}"
        ));
    }
    fs::write(
        root.join("compile_commands.json"),
        format!("[\n{}\n]\n", entries.join(",\n")),
    )
    .unwrap();
}

fn generate_fixture(root: &Path) {
    let include_dir = root.join("include");
    let src_dir = root.join("src");
    fs::create_dir_all(&include_dir).unwrap();
    fs::create_dir_all(&src_dir).unwrap();
    write_headers(&include_dir);
    for index in 0..TU_COUNT {
        fs::write(src_dir.join(format!("tu{index}.cpp")), tu_source(index)).unwrap();
    }
    write_compile_commands(root);
}

/// Milliseconds since `started`, for the phase table.
fn millis(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1000.0
}

/// Per-phase wall time of the replicated `ensure_load` sequence (Deliverable
/// A of plans/LoadPerfImprovements): `prep` = analyzer construction from the
/// cached stage clones, `symbol_loop`/`file_loop` = the two rayon passes,
/// `save` = `ArtifactManager::save_load_artifacts` (json + bincode + manifest
/// atomic writes), `cache_wrap` = the O(1) `Arc::new(report)` move
/// `ensure_load` stores in `load_cache` (was a deep `report.clone()`).
struct PhaseTimings {
    prep_ms: f64,
    symbol_loop_ms: f64,
    file_loop_ms: f64,
    save_ms: f64,
    cache_wrap_ms: f64,
}

/// Faithful replication of the private phase sequence inside
/// `Pipeline::ensure_load` (pipeline/src/lib.rs) using public APIs only:
/// same cached inputs, same fresh `LoadAnalyzer` cold state, same
/// symbols-then-files evaluation order, same `workers: None` path (plain
/// `par_iter` on the global pool), same `save_load_artifacts` call, same
/// `Arc::new(report)` cache step (proxied as `Arc::new(&report)` so the
/// report stays available for the fidelity check — one heap allocation,
/// no deep copy either way). The returned report is checked against the
/// pipeline's own load cache so the replication cannot silently drift.
fn replicate_load_phases(pipeline: &mut ariadne_pipeline::Pipeline) -> (PhaseTimings, LoadReport) {
    let prep_started = Instant::now();
    let (graph, closures) = pipeline.get_or_build_include_graph().unwrap();
    let reduced = pipeline.get_or_build_reduced().unwrap();
    let analyzer = LoadAnalyzer::with_closures(&graph, &reduced, closures);
    let repository_root = pipeline.repo.root.clone();
    let prep_ms = millis(prep_started);

    let started = Instant::now();
    let symbol_metrics: Vec<LoadMetric> = reduced
        .symbols
        .values()
        .map(|record| record.name.clone())
        .collect::<Vec<String>>()
        .par_iter()
        .filter_map(|name| analyzer.analyze_symbol_load(name).ok())
        .collect();
    let symbol_loop_ms = millis(started);

    let started = Instant::now();
    let file_ids: Vec<_> = reduced.file_symbols.keys().copied().collect();
    let file_metrics: Vec<LoadMetric> = file_ids
        .par_iter()
        .map(|file_id| {
            graph
                .file_paths
                .get(file_id)
                .and_then(|path| analyzer.analyze_file_load(path).ok())
        })
        .flatten()
        .collect();
    let file_loop_ms = millis(started);

    let started = Instant::now();
    let report = LoadReport {
        repository_root,
        symbol_metrics,
        file_metrics,
    };
    pipeline
        .manager
        .save_load_artifacts(&report, &pipeline.fps.load)
        .unwrap();
    let save_ms = millis(started);

    let started = Instant::now();
    let _cached = Arc::new(&report);
    let cache_wrap_ms = millis(started);

    (
        PhaseTimings {
            prep_ms,
            symbol_loop_ms,
            file_loop_ms,
            save_ms,
            cache_wrap_ms,
        },
        report,
    )
}

#[test]
#[ignore = "synthetic 520-header stress fixture; run explicitly for load-stage timing"]
fn collateral_load_stage_cold_timing() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();
    generate_fixture(root);

    let mut pipeline = ariadne_pipeline::Pipeline::new(root, None).unwrap();
    pipeline.ensure_index().unwrap();
    pipeline.ensure_include_graph().unwrap();
    pipeline.ensure_reduced().unwrap();
    let reduced = pipeline.get_or_build_reduced().unwrap();
    let fixture_symbols = reduced.symbols.len();
    let fixture_edges = reduced.edges.len();
    drop(reduced);

    let started = Instant::now();
    pipeline.ensure_load().unwrap();
    let ensure_load_ms = millis(started);

    let (phases, replication_report) = replicate_load_phases(&mut pipeline);
    let reference = pipeline.get_or_build_load().unwrap();
    assert_eq!(
        reference.repository_root, replication_report.repository_root,
        "replication must feed the same repository root"
    );
    assert_eq!(
        reference.symbol_metrics, replication_report.symbol_metrics,
        "replicated symbol loop must match ensure_load element-for-element"
    );
    assert_eq!(
        reference.file_metrics, replication_report.file_metrics,
        "replicated file loop must match ensure_load element-for-element"
    );

    let symbol_metric_count = reference.symbol_metrics.len();
    let file_metric_count = reference.file_metrics.len();

    assert!(
        fixture_symbols >= 1000,
        "fixture must hit the plan target of 1000+ symbols, got {fixture_symbols}"
    );
    assert!(
        file_metric_count >= 500,
        "fixture must hit the plan target of 500+ files, got {file_metric_count}"
    );

    println!(
        "LOAD_BENCH headers={HEADER_COUNT} tus={TU_COUNT} reduced_symbols={fixture_symbols} \
         semantic_edges={fixture_edges} symbol_metrics={symbol_metric_count} \
         file_metrics={file_metric_count} ensure_load_ms={ensure_load_ms:.1}"
    );
    println!(
        "LOAD_BENCH symbol_loop_ms={:.1} file_loop_ms={:.1} save_ms={:.1} \
         prep_ms={:.1} cache_wrap_ms={:.1} total_ensure_load_ms={:.1}",
        phases.symbol_loop_ms,
        phases.file_loop_ms,
        phases.save_ms,
        phases.prep_ms,
        phases.cache_wrap_ms,
        ensure_load_ms,
    );
}
