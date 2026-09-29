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

use std::collections::HashMap;
use std::path::PathBuf;

use ariadne_graph::{
    is_includable_header, ContributingSource, ExtractionImpact, LoadAnalyzer, LoadMemo, LoadMetric,
    ReducedSemanticModel,
};
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

/// Verbatim copy of the PRE-D1 per-file-BFS closure algorithm, kept as an
/// independent bench-side oracle: the SCC-condensation rewrite of
/// `compute_closures` (plan 1790608181551 D1) must match it byte-for-byte on
/// this SCC-dense 520-header fixture, and timing the two back to back gives
/// the direct before/after number for the rewrite.
fn bfs_closure_oracle(
    graph: &ariadne_graph::IncludeGraph,
) -> HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure> {
    let mut adj: HashMap<ariadne_repo::FileId, Vec<ariadne_repo::FileId>> = HashMap::new();
    for e in &graph.edges {
        if let Some(dest) = e.destination {
            adj.entry(e.source).or_default().push(dest);
        }
    }
    for dests in adj.values_mut() {
        dests.sort();
        dests.dedup();
    }

    let mut all_files: std::collections::HashSet<ariadne_repo::FileId> =
        graph.file_paths.keys().copied().collect();
    for e in &graph.edges {
        all_files.insert(e.source);
        if let Some(dest) = e.destination {
            all_files.insert(dest);
        }
    }

    let mut closures = HashMap::new();
    for &file_id in &all_files {
        let direct = adj.get(&file_id).cloned().unwrap_or_default();
        let mut transitive = std::collections::HashSet::new();
        let mut visited = std::collections::HashSet::new();
        let mut stack = direct.clone();
        visited.insert(file_id);
        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            transitive.insert(current);
            if let Some(next_dests) = adj.get(&current) {
                for &next_dest in next_dests {
                    if !visited.contains(&next_dest) {
                        stack.push(next_dest);
                    }
                }
            }
        }
        let mut transitive_vec: Vec<ariadne_repo::FileId> = transitive.into_iter().collect();
        transitive_vec.sort();
        closures.insert(
            file_id,
            ariadne_graph::IncludeClosure::new(file_id, direct, transitive_vec),
        );
    }
    closures
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

    // D1 (plan 1790608181551): time `compute_closures` alone (new
    // SCC-condensation DP) against the verbatim per-file-BFS oracle on the
    // same cold graph, and lock I1 — both tables must be identical. The
    // stage-cached closures (built by `ensure_include_graph`) must agree too.
    let (bench_graph, stage_closures) = pipeline.get_or_build_include_graph().unwrap();
    let started = Instant::now();
    let oracle_closures = bfs_closure_oracle(&bench_graph);
    let bfs_oracle_ms = millis(started);
    let started = Instant::now();
    let dp_closures = bench_graph.compute_closures();
    let dp_compute_ms = millis(started);
    assert_eq!(
        dp_closures, oracle_closures,
        "D1 rewrite must equal the per-file-BFS oracle on the 520-header fixture"
    );
    assert_eq!(
        dp_closures, stage_closures,
        "recomputed closures must equal the stage-cached closures"
    );
    println!(
        "LOAD_BENCH compute_closures_bfs_ms={bfs_oracle_ms:.1} \
         compute_closures_dp_ms={dp_compute_ms:.1} closure_files={} edges={}",
        dp_closures.len(),
        bench_graph.edges.len(),
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

// ---------------------------------------------------------------------------
// Scenario 2 — justification payload assembly + simulated live route.
// The first pass's lesson: the real floor hides OUTSIDE the loop being
// timed. Both UI justification consumers build a COLD `LoadAnalyzer` — the
// payload once per export (ui/src/dto.rs:205 `build_file_justification`) and
// the live `/api/justification` route once PER REQUEST (ui/src/server.rs:669
// `api_justification`). The ui crate is not a dependency of this bench
// crate, so the exact dto/server call pattern is mirrored here against the
// public graph API with the same caps.
// ---------------------------------------------------------------------------

/// Bounded payload caps mirrored from ui/src/dto.rs (I3). The live route
/// passes `usize::MAX` to the same builder shape instead.
const JUST_MAX_SOURCES: usize = 16;
const JUST_MAX_NAMES: usize = 12;
const JUST_MAX_HEADERS: usize = 8;
const IMPACT_SLOTS_PER_ROOT: usize = 8;

/// The four builder caps as one parameter: the payload uses the dto values,
/// the live route mirrors the `usize::MAX` unbounded twin.
#[derive(Clone, Copy)]
struct JustCaps {
    max_sources: usize,
    max_names: usize,
    max_headers: usize,
    impact_slot_limit: usize,
}

const PAYLOAD_CAPS: JustCaps = JustCaps {
    max_sources: JUST_MAX_SOURCES,
    max_names: JUST_MAX_NAMES,
    max_headers: JUST_MAX_HEADERS,
    impact_slot_limit: IMPACT_SLOTS_PER_ROOT,
};
const ROUTE_CAPS: JustCaps = JustCaps {
    max_sources: usize::MAX,
    max_names: usize::MAX,
    max_headers: usize::MAX,
    impact_slot_limit: usize::MAX,
};

/// Scenario sizing: 24 representative roots for the payload path; the live
/// route is simulated as 4 roots x 2 repeat requests, each with a fresh
/// cold analyzer (worst-case click traffic on the fixture).
const PAYLOAD_SAMPLE_FILES: usize = 24;
const ROUTE_ROOT_FILES: usize = 4;
const ROUTE_REQUEST_REPEATS: usize = 2;

/// DTO-shaped entry mirroring `JustificationEntry` + the part of
/// `ImpactSummary` this bench retains (the ui crate types stay out).
struct EntryDto {
    justified_symbols: Vec<String>,
    collateral_symbols: Vec<String>,
    justified_count: usize,
    collateral_count: usize,
    required_headers: Vec<String>,
    dropped_from_root_count: usize,
}

/// One contributor's justified/collateral sides, mirroring dto.rs
/// `collect_source_sides` (headers with symbols only, E-I9).
#[derive(Default)]
struct SourceSides<'a> {
    justified: Option<&'a ContributingSource>,
    collateral: Option<&'a ContributingSource>,
}

fn collect_source_sides(metric: &LoadMetric) -> HashMap<u64, SourceSides<'_>> {
    let mut sides: HashMap<u64, SourceSides> = HashMap::new();
    for src in &metric.semantic_sources {
        if src.symbol_count > 0 && is_includable_header(&src.path) {
            sides.entry(src.file_id.0).or_default().justified = Some(src);
        }
    }
    for src in &metric.collateral_sources {
        if src.symbol_count > 0 && is_includable_header(&src.path) {
            sides.entry(src.file_id.0).or_default().collateral = Some(src);
        }
    }
    sides
}

/// Contributor ids ordered `(combined count desc, id asc)`, mirroring
/// dto.rs `rank_source_ids`.
fn rank_source_ids(sides: &HashMap<u64, SourceSides<'_>>) -> Vec<u64> {
    let combined = |id: &u64| -> usize {
        let s = &sides[id];
        s.justified.map_or(0, |x| x.symbol_count) + s.collateral.map_or(0, |x| x.symbol_count)
    };
    let mut ranked: Vec<u64> = sides.keys().copied().collect();
    ranked.sort_by(|a, b| combined(b).cmp(&combined(a)).then(a.cmp(b)));
    ranked
}

fn path_basename(path: &std::path::Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

/// Capped symbol names, mirroring dto.rs `justification_names`.
fn justification_names(
    side: Option<&ContributingSource>,
    model: &ReducedSemanticModel,
    max_names: usize,
) -> Vec<String> {
    let Some(side) = side else {
        return Vec::new();
    };
    let mut ids: Vec<&ariadne_index::SymbolId> = side.symbols.iter().collect();
    ids.sort();
    ids.into_iter()
        .take(max_names)
        .map(|id| {
            model
                .symbols
                .get(id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| id.0.clone())
        })
        .collect()
}

/// Capped basename required-header set, mirroring dto.rs
/// `justification_required_headers` (source basename excluded).
fn justification_required_headers(
    analyzer: &LoadAnalyzer,
    justified: &[ariadne_index::SymbolId],
    source_path: &std::path::Path,
    max_headers: usize,
) -> Vec<String> {
    if justified.is_empty() {
        return Vec::new();
    }
    let excluded = path_basename(source_path);
    let mut headers: Vec<String> = analyzer
        .required_headers_for_symbols(justified)
        .into_iter()
        .map(|(_, path, _)| path_basename(&path))
        .filter(|name| *name != excluded)
        .collect();
    headers.sort();
    headers.dedup();
    headers.truncate(max_headers);
    headers
}

/// Mirrors the shared per-entry core of dto.rs `build_file_justification`
/// (payload caps: 16 sources / 8 impact slots) and
/// `build_file_justification_detail` (route: unbounded caps and impact on
/// every eligible entry). Returns the DTO rows and the number of
/// `extraction_impact` BFS calls actually made.
fn build_justification_entries(
    analyzer: &LoadAnalyzer,
    model: &ReducedSemanticModel,
    metric: &LoadMetric,
    root_id: u64,
    caps: &JustCaps,
) -> (Vec<EntryDto>, usize) {
    let sides = collect_source_sides(metric);
    let ranked = rank_source_ids(&sides);
    let mut out = Vec::new();
    let mut impacts_used = 0usize;
    for id in ranked.into_iter().take(caps.max_sources) {
        let s = &sides[&id];
        let justified_count = s.justified.map_or(0, |x| x.symbol_count);
        let collateral_count = s.collateral.map_or(0, |x| x.symbol_count);
        let required_headers = s
            .justified
            .map(|j| justification_required_headers(analyzer, &j.symbols, &j.path, caps.max_headers))
            .unwrap_or_default();
        let mut dropped_from_root_count = 0usize;
        let eligible = justified_count > 0 && collateral_count > 0 && root_id != id;
        if eligible && impacts_used < caps.impact_slot_limit {
            let justified_symbols: &[ariadne_index::SymbolId] =
                s.justified.map_or(&[], |j| j.symbols.as_slice());
            let impact: Option<ExtractionImpact> = analyzer.extraction_impact(
                ariadne_repo::FileId(root_id),
                ariadne_repo::FileId(id),
                justified_symbols,
            );
            if let Some(impact) = impact {
                dropped_from_root_count = impact.dropped_from_root.len();
                impacts_used += 1;
            }
        }
        out.push(EntryDto {
            justified_symbols: justification_names(s.justified, model, caps.max_names),
            collateral_symbols: justification_names(s.collateral, model, caps.max_names),
            justified_count,
            collateral_count,
            required_headers,
            dropped_from_root_count,
        });
    }
    (out, impacts_used)
}

/// Top-collateral roots with a registered path, `(collateral desc, path asc)`
/// — the files the justification UI actually opens.
fn select_justification_roots(file_metrics: &[LoadMetric], count: usize) -> Vec<&LoadMetric> {
    let mut ranked: Vec<&LoadMetric> = file_metrics
        .iter()
        .filter(|m| m.collateral_load > 0 && m.target_file.is_some())
        .collect();
    ranked.sort_by(|a, b| {
        b.collateral_load
            .cmp(&a.collateral_load)
            .then_with(|| a.target_file.cmp(&b.target_file))
    });
    ranked.truncate(count);
    ranked
}

/// (a) Payload assembly: ONE fresh cold analyzer across the sample,
/// mirroring dto.rs build_file_justification (dto.rs:205). Returns
/// `(rows, impacts, prep_ms, loop_ms)`.
fn run_payload_assembly(
    graph: &ariadne_graph::IncludeGraph,
    reduced: &ReducedSemanticModel,
    sample: &[&LoadMetric],
    path_to_id: &HashMap<PathBuf, u64>,
) -> (Vec<EntryDto>, usize, f64, f64) {
    let started = Instant::now();
    let payload_analyzer = LoadAnalyzer::new(graph, reduced);
    let payload_prep_ms = millis(started);
    let started = Instant::now();
    let mut payload_rows: Vec<EntryDto> = Vec::new();
    let mut payload_impacts = 0usize;
    for metric in sample {
        let root_id = path_to_id[metric.target_file.as_ref().unwrap()];
        let (entries, impacts) = build_justification_entries(
            &payload_analyzer,
            reduced,
            metric,
            root_id,
            &PAYLOAD_CAPS,
        );
        payload_impacts += impacts;
        payload_rows.extend(entries);
    }
    (payload_rows, payload_impacts, payload_prep_ms, millis(started))
}

/// (b) Simulated `/api/justification` route: fresh cold analyzer per request
/// (server.rs:669), analyze_file_load (server.rs:671), then the unbounded
/// per-entry queries + impacts (build_file_justification_detail). Returns
/// `(requests, construct_ms, analyze_file_ms, entries_ms, entries_total,
/// impacts_total)`.
fn run_route_simulation(
    graph: &ariadne_graph::IncludeGraph,
    reduced: &ReducedSemanticModel,
    route_roots: &[&LoadMetric],
    path_to_id: &HashMap<PathBuf, u64>,
    repeats: usize,
) -> (usize, f64, f64, f64, usize, usize) {
    let (mut req_count, mut construct_ms, mut file_ms, mut entries_ms) =
        (0usize, 0f64, 0f64, 0f64);
    let (mut entries_total, mut impacts_total) = (0usize, 0usize);
    for _ in 0..repeats {
        for metric in route_roots {
            let file_path = metric.target_file.as_ref().unwrap();
            let root_id = path_to_id[file_path];
            let started = Instant::now();
            let analyzer = LoadAnalyzer::new(graph, reduced);
            construct_ms += millis(started);
            let started = Instant::now();
            let route_metric = analyzer.analyze_file_load(file_path).expect("route metric");
            file_ms += millis(started);
            let started = Instant::now();
            let (entries, impacts) = build_justification_entries(
                &analyzer,
                reduced,
                &route_metric,
                root_id,
                &ROUTE_CAPS,
            );
            entries_ms += millis(started);
            entries_total += entries.len();
            impacts_total += impacts;
            req_count += 1;
        }
    }
    (
        req_count,
        construct_ms,
        file_ms,
        entries_ms,
        entries_total,
        impacts_total,
    )
}

/// (b, AFTER-B1C1) Simulated `/api/justification` over ONE boot-seeded shared
/// memo: `LoadMemo::with_closures` from the stage closures (post-C1 ui boot),
/// then a cheap `LoadAnalyzer::with_memo` handle + by-id analyze per request
/// (post-C1 route). The first request pays the remaining cold work
/// (adjacency + semantic BFS); later requests ride the warm memo. Returns
/// `(requests, per_request_ms, entries_total, impacts_total)`.
fn run_route_simulation_shared_memo(
    graph: &ariadne_graph::IncludeGraph,
    reduced: &ReducedSemanticModel,
    closures: HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    route_roots: &[&LoadMetric],
    path_to_id: &HashMap<PathBuf, u64>,
    repeats: usize,
) -> (usize, Vec<f64>, usize, usize) {
    let memo = Arc::new(LoadMemo::with_closures(closures));
    let mut per_request_ms = Vec::new();
    let (mut entries_total, mut impacts_total) = (0usize, 0usize);
    for _ in 0..repeats {
        for metric in route_roots {
            let file_path = metric.target_file.as_ref().unwrap();
            let root_id = ariadne_repo::FileId(path_to_id[file_path]);
            let started = Instant::now();
            let analyzer = LoadAnalyzer::with_memo(graph, reduced, Arc::clone(&memo));
            let route_metric = analyzer
                .analyze_file_load_by_id(root_id)
                .expect("route metric");
            let (entries, impacts) = build_justification_entries(
                &analyzer,
                reduced,
                &route_metric,
                root_id.0,
                &ROUTE_CAPS,
            );
            per_request_ms.push(millis(started));
            entries_total += entries.len();
            impacts_total += impacts;
        }
    }
    (per_request_ms.len(), per_request_ms, entries_total, impacts_total)
}

/// Everything the AFTER-B1C1 route comparison needs: the shared-memo scenario
/// inputs (graph, reduced model, boot-seeded `closures`, request roots and id
/// map) plus the cold BEFORE-B1C1 run's tallies that serve as the identity (I1)
/// baseline (`cold_*` fields).
struct AfterRouteComparison<'a> {
    graph: &'a ariadne_graph::IncludeGraph,
    reduced: &'a ReducedSemanticModel,
    closures: HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    route_roots: &'a [&'a LoadMetric],
    path_to_id: &'a HashMap<PathBuf, u64>,
    cold_requests: usize,
    cold_entries: usize,
    cold_impacts: usize,
    cold_total_ms: f64,
}

/// AFTER-B1C1 measurement: run the same requests over one boot-seeded memo,
/// lock results identity against the cold run (I1), and print the comparison
/// table. `cold_total_ms` is the cold scenario's summed time.
fn report_after_route_comparison(comparison: AfterRouteComparison<'_>) {
    let AfterRouteComparison {
        graph,
        reduced,
        closures,
        route_roots,
        path_to_id,
        cold_requests: req_count,
        cold_entries: entries_total,
        cold_impacts: route_impacts,
        cold_total_ms: before_total_ms,
    } = comparison;
    let (shared_count, shared_per_request_ms, shared_entries, shared_impacts) =
        run_route_simulation_shared_memo(
            graph,
            reduced,
            closures,
            route_roots,
            path_to_id,
            ROUTE_REQUEST_REPEATS,
        );
    assert_eq!(shared_count, req_count, "same request count");
    assert_eq!(
        shared_entries, entries_total,
        "shared memo must serve byte-identical entry counts"
    );
    assert_eq!(
        shared_impacts, route_impacts,
        "shared memo must run the same number of extraction impacts"
    );
    let shared_total_ms: f64 = shared_per_request_ms.iter().sum();
    let before_per_request = before_total_ms / req_count as f64;
    let after_per_request = shared_total_ms / shared_count as f64;
    println!(
        "JUSTIFY_BENCH AFTER_B1C1_ROUTE requests={shared_count} entries_per_request={} \
         impacts={shared_impacts} total_ms={shared_total_ms:.1} per_request_ms={after_per_request:.1} \
         first_request_ms={:.1} last_request_ms={:.1} before_per_request_ms={before_per_request:.1} \
         improvement_factor={:.1}x",
        shared_entries / shared_count,
        shared_per_request_ms[0],
        shared_per_request_ms[shared_per_request_ms.len() - 1],
        before_per_request / after_per_request,
    );
}

#[test]
#[ignore = "synthetic 520-header stress fixture; run explicitly for justification-path timing"]
fn justification_payload_and_route_cold_timing() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();
    generate_fixture(root);

    let mut pipeline = ariadne_pipeline::Pipeline::new(root, None).unwrap();
    pipeline.ensure_index().unwrap();
    pipeline.ensure_include_graph().unwrap();
    pipeline.ensure_reduced().unwrap();
    pipeline.ensure_load().unwrap();

    let (graph, closures) = pipeline.get_or_build_include_graph().unwrap();
    let reduced = pipeline.get_or_build_reduced().unwrap();
    let report = pipeline.get_or_build_load().unwrap();

    let path_to_id: HashMap<PathBuf, u64> = graph
        .file_paths
        .iter()
        .map(|(id, path)| (path.clone(), id.0))
        .collect();
    let sample = select_justification_roots(&report.file_metrics, PAYLOAD_SAMPLE_FILES);
    assert_eq!(sample.len(), PAYLOAD_SAMPLE_FILES, "sample roots expected");

    let (payload_rows, payload_impacts, payload_prep_ms, payload_loop_ms) =
        run_payload_assembly(&graph, &reduced, &sample, &path_to_id);
    println!(
        "JUSTIFY_BENCH BEFORE_B1C1_PAYLOAD headers={HEADER_COUNT} roots={} entries={} impacts={} \
         prep_ms={:.1} loop_ms={:.1} total_ms={:.1}",
        sample.len(),
        payload_rows.len(),
        payload_impacts,
        payload_prep_ms,
        payload_loop_ms,
        payload_prep_ms + payload_loop_ms,
    );

    let route_roots = &sample[..ROUTE_ROOT_FILES];
    let (req_count, construct_ms, file_ms, entries_ms, entries_total, route_impacts) =
        run_route_simulation(&graph, &reduced, route_roots, &path_to_id, ROUTE_REQUEST_REPEATS);
    let total_ms = construct_ms + file_ms + entries_ms;
    println!(
        "JUSTIFY_BENCH BEFORE_B1C1_ROUTE requests={} entries_per_request={} impacts={} \
         construct_ms={:.1} analyze_file_ms={:.1} entries_ms={:.1} total_ms={:.1} \
         per_request_ms={:.1}",
        req_count,
        entries_total / req_count,
        route_impacts,
        construct_ms,
        file_ms,
        entries_ms,
        total_ms,
        total_ms / req_count as f64,
    );

    // AFTER-B1C1: the SAME requests over ONE boot-seeded shared memo; the
    // helper locks entry/impact counts against the cold run (I1).
    report_after_route_comparison(AfterRouteComparison {
        graph: &graph,
        reduced: &reduced,
        closures,
        route_roots,
        path_to_id: &path_to_id,
        cold_requests: req_count,
        cold_entries: entries_total,
        cold_impacts: route_impacts,
        cold_total_ms: total_ms,
    });

    let has_symbols = payload_rows
        .iter()
        .any(|e| !e.justified_symbols.is_empty() || !e.collateral_symbols.is_empty());
    assert!(has_symbols, "payload rows must carry symbols");
    let payload_counts: usize = payload_rows
        .iter()
        .map(|e| {
            e.justified_count
                + e.collateral_count
                + e.required_headers.len()
                + e.dropped_from_root_count
        })
        .sum();
    assert!(payload_counts > 0, "DTO fields must be populated");
    assert!(
        route_impacts > 0,
        "route simulation must run real extraction impacts"
    );
}
