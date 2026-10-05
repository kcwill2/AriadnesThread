//! Boot-stage timing bench for the M27 `tu-index-hang-throughput`
//! SUFFICIENCY gate (shape sec. 3.3, handoff T1a): measures the UI-boot
//! stage path in the RELEASE profile on a synthetic 2000-header / 400-TU
//! fixture, printing the `label | ms` table whose eight release labels are
//! T1b's `DOCUMENTED_PHASES_BOOT_STAGES` contract (byte-exact - T1b parses
//! them via `bench_compare.parse_bench_table`, so context lines below carry
//! no pipe).
//!
//! Run the release column (handoff gate 1):
//!   cargo test --release -p ariadne-pipeline --test boot_stages_bench -- \
//!     boot_stages_release_timing --ignored --nocapture --test-threads=1
//! Run the debug context row (handoff gate 2):
//!   cargo test -p ariadne-pipeline --test boot_stages_bench -- \
//!     boot_stages_debug_context_timing --ignored --nocapture
//!
//! Method - replicate the boot path like `load_perf_bench` does, over the
//! public surface only (pipeline `src/` untouched):
//! - rows 1-3 replicate `Pipeline::compute_inputs` in boot order:
//!   `Repository::locate` + `discover()` (discover+hash), `BuildDiscoverer`
//!   with the pipeline registry shaper selection, `StageFingerprints::
//!   from_inputs`;
//! - stage rows run fresh `Pipeline` instances over temp fixtures; cold =
//!   fresh artifacts tree;
//! - WARM LOAD SUBTRACTION METHOD: a warm row times fresh-pipeline
//!   construction followed by the stage load, then subtracts the
//!   construction-equivalent cost - the time of an identical `Pipeline::new`
//!   over the same tree (the `rehash_inputs`-equivalent input recomputation
//!   plus artifact-manager prep) measured as an ANCHOR directly after a
//!   throwaway construct and directly before the boot measure, so anchor
//!   and boot hash in the same steady-state page cache. (A plain
//!   `Pipeline::rehash_inputs` subtraction was measured to run NEGATIVE
//!   ~ -93 ms on the fast index-reuse row: the anchor hashing while the
//!   second fixture's page churn had not settled, the boot hashing warm.
//!   The anchored form is the stated method.) Each warm row also asserts
//!   its stage recorded `StageReuse::Loaded`, so the row can never
//!   silently measure a rebuild;
//! - W=N uses `Pipeline::new(&path, Some(n))`, `n = min(8,
//!   available_parallelism)` - today's library capability, NOT the T3
//!   workers dispatch. It runs on a second freshly generated fixture,
//!   because the W=1 cold run wrote index artifacts + shards into the first
//!   tree and a second cold run needs a pristine artifacts tree.
//!
//! ## MEASUREMENT TABLE (A4 calibration-evidence precedent: the measured
//! ## table lives beside the consts; medians of 3 runs, all values ms;
//! ## per-run rows, idle-ness note and full verdict:
//! ## `tasks/tu-index-hang-throughput-t1-measurement.md`)
//! Date 2026-10-05, profile RELEASE, machine
//! `windows-AMD64-AMD64 Family 26 Model 68 Stepping 0, AuthenticAMD`,
//! row width n=8:
//!
//! | phase                                     | median   |
//! |-------------------------------------------|----------|
//! | discover+hash total (release)             |    224.6 |
//! | build discovery (contexts) total (rel.)   |      1.0 |
//! | stage fingerprints total (release)        |      0.7 |
//! | include-graph cold total (release)        |    464.3 |
//! | include-graph warm load total (release)   |     44.3 |
//! | tu-index cold total (W=1, release)        |  60553.4 |
//! | tu-index cold total (W=N, release)        |  85681.1 |
//! | tu-index warm reuse total (release)       |   3176.2 |
//!
//! Sufficiency headline: 151.4 ms/TU at W=1 -> ~14.5 min projected stage-2
//! cold wall at 5744 TU; W=N speedup factor 0.71 (the production path is
//! measurably SLOWER at n=8 on this fixture). Debug context row: 12.6 ms,
//! done-post-T2.
//!
//! Scale consts below are the shape sec. 3.3 "start" values, ratifiable at
//! the evidence review. If a release run exceeds the 600 s gate timeout the
//! scale is HALVED and the halving recorded (R9) - never silently scaled.
//!
//! A4 calibration-evidence precedent: the measured table belongs beside
//! these consts (compare `crates/cli/src/commands/ui/stage_spans.rs`).

use std::fs;
use std::path::Path;
use std::time::Instant;

use ariadne_build::BuildDiscoverer;
use ariadne_index::StageFingerprints;
use ariadne_pipeline::{
    default_registry, Pipeline, StageReuse, STAGE_INCLUDE_GRAPH, STAGE_INDEX,
};
use ariadne_repo::Repository;

mod common;

/// Headers on the modular fanout (start setting per shape sec. 3.3).
const HEADER_COUNT: usize = 2000;
/// Translation units, each including 5 spread headers (start setting).
const TU_COUNT: usize = 400;
/// Upper bound for the W=N row (second N value for the sufficiency gate;
/// the exact auto-workers cap is Q7, ratified at review only if T3 fires).
const WORKER_CAP: usize = 8;

/// The debug context row label (E3 shape tracking row, non-gate - it is
/// deliberately NOT one of T1b's eight documented release labels).
const DEBUG_ROW_LABEL: &str =
    "tu-index non-ASCII repro TU (debug, panic-pre-T2 / done-post-T2)";

fn print_table_header() {
    println!("{:<48} | {:>8}", "phase", "ms");
    println!("{}", "-".repeat(59));
}

/// Shared timed-section print: one `label | ms` table row in the exact
/// grammar `bench_compare.parse_bench_table` consumes.
fn print_row(label: &str, ms: f64) {
    println!("{:<48} | {:>8.1}", label, ms);
}

/// Machine class facts for the run header (no pipe: bench_compare skips
/// non-table lines, a stray pipe would parse-drift the gate).
fn machine_label() -> String {
    let architecture =
        std::env::var("PROCESSOR_ARCHITECTURE").unwrap_or_else(|_| "unknown".to_string());
    let processor =
        std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "unknown".to_string());
    format!("windows-{architecture}-{processor}")
}

/// Auto-workers row width: min(8, available_parallelism), per the handoff.
fn parallel_row_width() -> usize {
    let cores = std::thread::available_parallelism()
        .map(|width| width.get())
        .unwrap_or(1);
    usize::min(WORKER_CAP, cores)
}

/// Cold include-graph row: fresh pipeline over the fixture, timed
/// `ensure_include_graph` on the fresh artifacts tree (construction
/// untimed - rows 1-3 price the construction components separately).
fn measure_include_graph_cold(root: &Path) -> f64 {
    let mut pipeline = Pipeline::new(root, None).expect("cold include pipeline");
    let started = Instant::now();
    pipeline
        .ensure_include_graph()
        .expect("cold include graph build");
    let cold_ms = common::millis(started);
    assert_eq!(
        pipeline.stats.stage_reuse(STAGE_INCLUDE_GRAPH),
        Some(StageReuse::Rebuilt),
        "cold include row must actually rebuild"
    );
    cold_ms
}

/// Shared warm-load row per the module-doc subtraction method: a throwaway
/// construct settles the page cache, an identical `Pipeline::new` is timed
/// as the construction ANCHOR, then construction + stage work are timed
/// back to back and the anchor is subtracted, isolating the stage load.
/// The `ensure` closure must assert its stage recorded `StageReuse::Loaded`
/// so the row can never silently measure a rebuild.
fn measure_warm_load(root: &Path, ensure: impl Fn(&mut Pipeline)) -> f64 {
    let warmup = Pipeline::new(root, None).expect("warm-up construction");
    drop(warmup);

    let anchor_start = Instant::now();
    let anchor = Pipeline::new(root, None).expect("anchor construction");
    let anchor_ms = common::millis(anchor_start);
    drop(anchor);

    let boot_start = Instant::now();
    let mut pipeline = Pipeline::new(root, None).expect("warm pipeline");
    ensure(&mut pipeline);
    let construction_plus_load_ms = common::millis(boot_start);
    construction_plus_load_ms - anchor_ms
}

/// Cold TU-index row for one worker setting: fresh pipeline over `root`,
/// timed `ensure_index`. `workers = Some(n)` proves the production path
/// ran (asserted via `production_stats`).
fn measure_index_cold(root: &Path, workers: Option<usize>) -> f64 {
    let mut pipeline = Pipeline::new(root, workers).expect("cold index pipeline");
    let started = Instant::now();
    pipeline.ensure_index().expect("cold index build");
    let cold_ms = common::millis(started);
    assert_eq!(
        pipeline.stats.stage_reuse(STAGE_INDEX),
        Some(StageReuse::Rebuilt),
        "cold index row must actually rebuild"
    );
    assert_eq!(
        pipeline.stats.reanalyzed_tus, TU_COUNT,
        "cold index row must reanalyze every TU (no shard reuse)"
    );
    if workers.is_some() {
        assert!(
            pipeline.production_stats.is_some(),
            "W=N row must prove the Some(workers) production path ran"
        );
    }
    cold_ms
}

#[test]
#[ignore = "synthetic 2000-header/400-TU boot-stage bench; run explicitly for the M27 sufficiency gate"]
fn boot_stages_release_timing() {
    let fixture_dir = tempfile::TempDir::new().expect("fixture temp dir");
    let root = fixture_dir.path();
    common::generate_fixture_sized(root, HEADER_COUNT, TU_COUNT);

    let registry = default_registry();

    let started = Instant::now();
    let repo = Repository::locate(root).expect("repo locate");
    let discovery = repo.discover().expect("repo discover");
    let discover_hash_ms = common::millis(started);

    let started = Instant::now();
    let build = BuildDiscoverer::new(&repo.root)
        .with_repo_discovery(&discovery)
        .with_unit_shaper(registry.unit_shaper_for_repo(&repo.root))
        .discover()
        .expect("build discovery");
    let build_discovery_ms = common::millis(started);
    assert_eq!(
        build.translation_units.len(),
        TU_COUNT,
        "fixture must shape exactly TU_COUNT units"
    );

    let started = Instant::now();
    let _stage_fps = StageFingerprints::from_inputs(
        &repo.root,
        &discovery,
        &build.contexts,
        &build.translation_units,
    );
    let fingerprints_ms = common::millis(started);

    let include_cold_ms = measure_include_graph_cold(root);
    let include_warm_ms = measure_warm_load(root, |pipeline| {
        pipeline
            .ensure_include_graph()
            .expect("warm include graph load");
        assert_eq!(
            pipeline.stats.stage_reuse(STAGE_INCLUDE_GRAPH),
            Some(StageReuse::Loaded),
            "warm include row must LOAD from artifacts, never rebuild"
        );
    });

    let index_cold_w1_ms = measure_index_cold(root, None);

    // Second pristine fixture for the W=N cold row (the first tree now
    // carries index artifacts + shards written by the W=1 row).
    let parallel_dir = tempfile::TempDir::new().expect("parallel fixture temp dir");
    let parallel_root = parallel_dir.path();
    common::generate_fixture_sized(parallel_root, HEADER_COUNT, TU_COUNT);
    let workers_n = parallel_row_width();
    let index_cold_wn_ms = measure_index_cold(parallel_root, Some(workers_n));

    // Warm reuse rides the W=1 fixture, whose index artifacts + shards the
    // cold row just wrote (boot-shape: second boot over the same tree).
    let index_warm_ms = measure_warm_load(root, |pipeline| {
        pipeline.ensure_index().expect("warm index load");
        assert_eq!(
            pipeline.stats.stage_reuse(STAGE_INDEX),
            Some(StageReuse::Loaded),
            "warm index row must LOAD the index stage, never rebuild"
        );
    });

    println!(
        "BOOT_STAGES_BENCH headers={HEADER_COUNT} tus={TU_COUNT} workers_n={workers_n} \
         profile=release debug_assertions={} machine={}",
        cfg!(debug_assertions),
        machine_label(),
    );
    print_table_header();
    print_row("discover+hash total (release)", discover_hash_ms);
    print_row("build discovery (contexts) total (release)", build_discovery_ms);
    print_row("stage fingerprints total (release)", fingerprints_ms);
    print_row("include-graph cold total (release)", include_cold_ms);
    print_row("include-graph warm load total (release)", include_warm_ms);
    print_row("tu-index cold total (W=1, release)", index_cold_w1_ms);
    print_row("tu-index cold total (W=N, release)", index_cold_wn_ms);
    print_row("tu-index warm reuse total (release)", index_warm_ms);
}

/// One-TU temp corpus carrying the E3 shape (non-ASCII WORD run and
/// non-ASCII PUNCT run), built from numeric byte arrays so this authored
/// source stays pure ASCII. Lines mirror the approved T0b fixture
/// `tests/fixtures/m27-nonascii-underflow` (U+00E9 pairs in the leading
/// word; line-leading U+20AC punct run + U+00AB guillemet mid-run).
/// Constructed in a temp dir only - NEVER committed.
fn write_e3_shape_fixture(root: &Path) {
    let include_dir = root.join("include");
    let src_dir = root.join("src");
    fs::create_dir_all(&include_dir).expect("include dir");
    fs::create_dir_all(&src_dir).expect("src dir");
    fs::write(
        include_dir.join("runs.h"),
        "#pragma once\nstruct RunsConfig { int ticks; };\n",
    )
    .expect("header write");

    // U+00E9 as C3 A9: "caf" + e-acute.
    let cafe = String::from_utf8(vec![0x63, 0x61, 0x66, 0xC3, 0xA9]).expect("valid utf8");
    // U+20AC euro sign (E2 82 AC) and U+00AB guillemet (C2 AB).
    let euro = String::from_utf8(vec![0xE2, 0x82, 0xAC]).expect("valid utf8");
    let guillemet = String::from_utf8(vec![0xC2, 0xAB]).expect("valid utf8");

    let mut source = String::from("#include \"runs.h\"\n\n");
    source.push_str(&format!("{}_{} {}ine = {}_cup;\n", cafe, cafe, cafe, cafe));
    source.push_str(&format!("{}uro seam = 1 {} 2;\n", euro, guillemet));
    fs::write(src_dir.join("tu0.cpp"), source.as_bytes()).expect("tu write");
    common::write_compile_commands(root, 1);
}

#[test]
#[ignore = "E3-shape debug-context row over a one-TU temp corpus; context evidence, never a gate row (M27 T1a)"]
fn boot_stages_debug_context_timing() {
    let fixture_dir = tempfile::TempDir::new().expect("e3 fixture temp dir");
    let root = fixture_dir.path();
    write_e3_shape_fixture(root);

    let started = Instant::now();
    let mut pipeline = Pipeline::new(root, None).expect("e3 pipeline");
    pipeline.ensure_index().expect("e3 cold index");
    let cold_ms = common::millis(started);

    println!(
        "BOOT_STAGES_DEBUG profile=debug debug_assertions={} machine={}",
        cfg!(debug_assertions),
        machine_label(),
    );
    print_table_header();
    println!("{:<48} | {:>8.1}", DEBUG_ROW_LABEL, cold_ms);
}
