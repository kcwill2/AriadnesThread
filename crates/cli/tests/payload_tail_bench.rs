//! Stage-5 "Analyzing collateral" tail-phase bench (plan
//! `.kilo/plans/1790652617995-load-tail-phase-perf.md`, task A0).
//!
//! Mirrors the CLI UI boot tail (`crates/cli/src/commands/ui.rs`
//! `build_ui_payload`): the `ensure_load` stage is driven with a progress
//! receiver that timestamps `CollateralFile` events, then every payload
//! assembly builder is timed at its public API boundary, in boot order.
//! The synthetic fixture generator is a verbatim duplicate of
//! `crates/pipeline/tests/load_perf_bench.rs` (test-code duplication is
//! intentional; the cli crate already depends on all crates used here).
//!
//! Phase-boundary notes (public APIs only, no production changes):
//! - `symbol_pass` is COMBINED with analyzer prep and the first file
//!   completion: the upper bound is the FIRST `CollateralFile` event after
//!   `ensure_load` starts (the symbol loop runs before any file event).
//! - `file_pass` is the FIRST..TERMINAL `CollateralFile` event span, so it
//!   excludes the first file's completion tail counted into `symbol_pass`.
//! - `save_load_artifacts` is COMBINED with the `Arc::new(report)` cache
//!   wrap: measured as terminal-event -> `ensure_load` return.
//! - `generate_report`/`detect_cycle_groups` and the `path_to_id`/`file_collateral`
//!   maps are each timed COMBINED (two adjacent public calls per row).
//!
//! ## Baseline table (recorded 2026-09-29, combined cycle row re-pinned
//! ## 2026-10-04, this bench, `--test-threads=1`)
//!
//! Fixture: 520 headers / 40 TUs, 2160 reduced symbols, 560 file metrics,
//! 8960 justification entries, 520 plans (all Conservative rows).
//!
//! ```text
//! phase                                            |       ms
//! -----------------------------------------------------------
//! symbol_pass+prep (combined, <= first event)      |    322.0
//! file_pass (event span)                           |     89.5
//! save_load_artifacts+cache (combined)             |   1799.7
//! ensure_load total                                |   2211.3
//! get_or_build_include_graph clone (cold)          |      1.2
//! get_or_build_load clone (cold)                   |   1048.8
//! get_or_build_include_graph clone (warm)          |      1.4
//! get_or_build_load clone (warm)                   |   1056.5
//! generate_report+detect_cycle_groups (combined)   |      5.6
//! build_include_elements                           |      5.4
//! path_to_id+file_collateral maps (combined)       |     29.5
//! build_file_justification_with_memo               |  15383.9
//! OptimizationPlanner::plan() cold                 |   2392.6
//! conservative filter pass                         |      0.0
//! ```
//!
//! Headline: `build_file_justification_with_memo` (15.4 s) dominates the
//! stage-5 tail, then `OptimizationPlanner::plan()` cold (2.4 s), the
//! inline artifact save (1.8 s), and the `LoadReport` deep clone (1.05 s,
//! cold == warm — it is an unconditional `Arc::unwrap_or_clone` copy).
//! The include-graph clone (~1 ms) is negligible.
//!
//! A6 honesty note: the `OptimizationPlanner::plan() cold` row in the A0
//! baseline table predates plan A1. The bench now mirrors the CURRENT boot
//! wiring in ui.rs `build_extraction_proposals` (screening facts from the
//! stage's file metrics + reuse of the stage closures, and the rows run in
//! true boot order — proposals before the justification memo takes the
//! closures), so the current planner row is NOT comparable against the A0
//! cold number kept above for history.
//!
//! M23.5 T3.2 re-pin (2026-10-04): the `generate_report+detect_cycles`
//! combined column became `generate_report+detect_cycle_groups` (the SCC
//! replacement for `detect_cycles`; `detect_cycles` still exists until the
//! crates/graph unit D deletes it). This row was re-pinned above to `5.6` ms
//! from a `--release -- --ignored --test-threads=1` run on 2026-10-04. The
//! prior combined figure was `5.8` ms (this bench's 2026-09-29 table). The
//! plan's stated baseline was a DIFFERENT number, `4.7` ms, quoted in the
//! `ui.rs:103` stage-5 calibration table for the same combined call; `4.7`
//! and `5.8` were never reconciled in the tree. I re-pinned against this
//! bench's OWN recorded `5.8` (the same bench, same table, same combined
//! call) and measured `5.6`: parity within single-run noise, the expected
//! outcome since SCC detection is linear in V+E. Both sites now record the
//! single measured `5.6` (the `ui.rs` stage-5 calibration comment moved
//! with this row per handoff sec 3.3); the split `5.8`-vs-`4.7` pre-unit
//! baseline is recorded here and in `tasks/m235-t3b-journal.md` rather than
//! silently resolved.
//!
//! UNVERIFIED (handoff sec 5): `5.6` is a SINGLE LOCAL MEASUREMENT on this
//! machine on 2026-10-04, NOT a bench-compare-verified result.
//! `python tools/bench_compare.py` is REFUSED by the reviewer bash allowlist
//! (ledger M23.5 F10) and was NOT run, so there is NO reviewer-run
//! perf-regression evidence for `detect_cycle_groups`. "Expected parity" is
//! not "measured parity": the comparison above is UNVERIFIED against
//! `bench_compare.py`, and it was NOT routed through `distill.py` to evade
//! the denial.
//!
//! Run with:
//!   cargo test -p ariadne-cli --test payload_tail_bench --release -- \
//!     --ignored --nocapture --test-threads=1

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use ariadne_pipeline::{Pipeline, ProgressEvent};

// ---------------------------------------------------------------------------
// Synthetic fixture — verbatim duplicate of load_perf_bench.rs (plan A0).
// ---------------------------------------------------------------------------

/// Headers on a 4-wide modular fanout (h_i includes h_(i+10..i+13) mod N),
/// cycle-tolerant like the pipeline bench fixture.
const HEADER_COUNT: usize = 520;
const FANOUT_BASE: usize = 10;
const FANOUT_WIDTH: usize = 4;
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
/// cross-calling free functions (4 symbols/header) in ~20 lines.
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

/// compile_commands.json mirroring the pipeline bench: forward-slash
/// absolute directory, `clang++ -Iinclude -c src/tuN.cpp`.
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
    for index in 0..HEADER_COUNT {
        fs::write(include_dir.join(format!("h{index}.h")), header_source(index)).unwrap();
    }
    for index in 0..TU_COUNT {
        fs::write(src_dir.join(format!("tu{index}.cpp")), tu_source(index)).unwrap();
    }
    write_compile_commands(root);
}

// ---------------------------------------------------------------------------
// Timing helpers
// ---------------------------------------------------------------------------

/// Milliseconds since `started`, for the phase table.
fn millis(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1000.0
}

/// Run `f` and return its value with its wall time in ms.
fn timed<T>(f: impl FnOnce() -> T) -> (T, f64) {
    let started = Instant::now();
    let out = f();
    (out, millis(started))
}

/// First/last `CollateralFile` arrival stamps, captured while `ensure_load`
/// runs. First arrival upper-bounds the symbol pass; the terminal
/// `done == total` event (emitted after the file loop collects) marks the
/// file-pass end and the start of the serial artifact-save tail.
#[derive(Default)]
struct CollateralEventMarks {
    first: Option<Instant>,
    last: Option<Instant>,
}

fn track_collateral(marks: &Arc<Mutex<CollateralEventMarks>>) -> impl FnMut(&ProgressEvent) + Send {
    let marks = Arc::clone(marks);
    move |event| {
        if matches!(event, ProgressEvent::CollateralFile { .. }) {
            let mut guard = marks.lock().unwrap();
            if guard.first.is_none() {
                guard.first = Some(Instant::now());
            }
            guard.last = Some(Instant::now());
        }
    }
}

/// `ensure_load` split via the progress-event stamps: combined symbol pass,
/// event-span file pass, combined save+cache-wrap tail, plus the total.
struct LoadStageBreakdown {
    total_ms: f64,
    symbol_and_prep_ms: f64,
    file_event_span_ms: f64,
    save_tail_ms: f64,
}

fn measure_load_stage(
    pipeline: &mut Pipeline,
    marks: &Arc<Mutex<CollateralEventMarks>>,
) -> LoadStageBreakdown {
    let started = Instant::now();
    pipeline.ensure_load().unwrap();
    let total_ms = millis(started);
    let guard = marks.lock().unwrap();
    let first = guard
        .first
        .expect("fresh artifact store must rebuild the load stage and emit CollateralFile events");
    let last = guard.last.expect("terminal CollateralFile event must arrive");
    LoadStageBreakdown {
        total_ms,
        symbol_and_prep_ms: first.duration_since(started).as_secs_f64() * 1000.0,
        file_event_span_ms: last.duration_since(first).as_secs_f64() * 1000.0,
        save_tail_ms: last.elapsed().as_secs_f64() * 1000.0,
    }
}

// ---------------------------------------------------------------------------
// Payload assembly — mirrors build_ui_payload (cli/src/commands/ui.rs) in
// boot order, timing every builder at its public call.
// ---------------------------------------------------------------------------

/// Shared path→numeric-FileId lookup (duplicated from ui.rs `build_path_to_id`).
fn build_path_to_id(include_graph: &ariadne_graph::IncludeGraph) -> HashMap<PathBuf, u64> {
    include_graph
        .file_paths
        .iter()
        .map(|(id, path)| (path.clone(), id.0))
        .collect()
}

/// Duplicated from ui.rs `translation_unit_file_ids`.
fn translation_unit_file_ids(
    build: &ariadne_build::BuildDiscoveryResult,
) -> Vec<ariadne_repo::FileId> {
    build
        .translation_units
        .iter()
        .map(|tu| tu.file_id)
        .collect()
}

/// Assembly rows in boot order plus the payload sizes worth asserting.
struct AssemblyOutcome {
    rows: Vec<(&'static str, f64)>,
    justification_entries: usize,
    plans: usize,
    proposal_rows: usize,
}

fn run_assembly(
    pipeline: &mut Pipeline,
    reduced: &ariadne_graph::ReducedSemanticModel,
) -> AssemblyOutcome {
    // A3 hand-outs, cold then warm: `get_or_build_load` is now an Arc
    // refcount bump (~0); the graph row still times the clone API boot no
    // longer uses (boot moved to `take_include_graph`).
    let ((graph, closures), graph_cold_ms) =
        timed(|| pipeline.get_or_build_include_graph().unwrap());
    let (load_report, load_cold_ms) = timed(|| pipeline.get_or_build_load().unwrap());
    let (_, graph_warm_ms) = timed(|| pipeline.get_or_build_include_graph().unwrap());
    let (_, load_warm_ms) = timed(|| pipeline.get_or_build_load().unwrap());

    // ui.rs:238-239 — two adjacent serial calls, timed together.
    let ((_graph_report, cycle_groups), report_cycles_ms) =
        timed(|| (graph.generate_report(), graph.detect_cycle_groups()));
    let (_elements, include_elements_ms) = timed(|| {
        let no_decoration = std::collections::HashMap::new();
        ariadne_ui::build_include_elements_from_groups(&graph, &closures, &cycle_groups, &no_decoration, &no_decoration)
    });
    // ui.rs:247-248 — two cheap serial maps, timed together.
    let (justification_maps_ms, path_to_id) = {
        let started = Instant::now();
        let path_to_id = build_path_to_id(&graph);
        let _collateral = build_file_collateral_like_ui(&path_to_id, &load_report.file_metrics);
        (millis(started), path_to_id)
    };

    // ui.rs:238-245 — boot runs the planner on screening facts from the
    // stage's file metrics and reuses the stage closures (plan A1), BEFORE
    // the memo below takes `closures`; mirrored here.
    let tu_ids = translation_unit_file_ids(&pipeline.build);
    let (plans, planner_ms) = timed(|| {
        ariadne_cluster::planner::OptimizationPlanner::with_screening_facts(
            &graph,
            reduced,
            &pipeline.discovery.files,
            &tu_ids,
            &ariadne_lang_cpp::cpp_aggregation_filter,
            &ariadne_lang_cpp::cpp_target_path_for,
            None,
            ariadne_lang::CyclePolicy::Reject,
            ariadne_cluster::planner::screening_facts_from_metrics(
                &graph,
                &load_report.file_metrics,
            ),
            &closures,
        )
        .plan()
    });
    let (conservative_rows, proposal_filter_ms) = timed(|| {
        plans
            .iter()
            .filter(|p| {
                matches!(
                    p.risk,
                    ariadne_cluster::plan::RiskLevel::Conservative
                ) && p.collateral_load_before != p.collateral_load_after
            })
            .count()
    });

    // ui.rs:250 — boot seeds the ONE shared memo by MOVING the closures
    // (out of its take); this bench moves its cloned map here likewise.
    let load_memo = Arc::new(ariadne_graph::LoadMemo::with_closures(closures));
    let (justification, justification_ms) = timed(|| {
        ariadne_ui::build_file_justification_with_memo(
            &load_report.file_metrics,
            &graph,
            reduced,
            &path_to_id,
            Some(&load_memo),
        )
    });

    let justification_entries: usize = justification.values().map(|m| m.len()).sum();
    let rows = vec![
        ("get_or_build_include_graph clone (cold)", graph_cold_ms),
        ("get_or_build_load hand-out (cold)", load_cold_ms),
        ("get_or_build_include_graph clone (warm)", graph_warm_ms),
        ("get_or_build_load hand-out (warm)", load_warm_ms),
        ("generate_report+detect_cycle_groups (combined)", report_cycles_ms),
        ("build_include_elements", include_elements_ms),
        (
            "path_to_id+file_collateral maps (combined)",
            justification_maps_ms,
        ),
        ("OptimizationPlanner::plan() facts+closures", planner_ms),
        ("conservative filter pass", proposal_filter_ms),
        ("build_file_justification_with_memo", justification_ms),
    ];
    AssemblyOutcome {
        rows,
        justification_entries,
        plans: plans.len(),
        proposal_rows: conservative_rows,
    }
}

/// Mirrors ui.rs `build_file_collateral` so the timed map-building pair
/// matches boot work (only its wall time matters here).
fn build_file_collateral_like_ui(
    path_to_id: &HashMap<PathBuf, u64>,
    file_metrics: &[ariadne_graph::LoadMetric],
) -> HashMap<String, HashMap<String, usize>> {
    let mut out: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for metric in file_metrics {
        let Some(path) = metric.target_file.as_ref().and_then(|p| path_to_id.get(p)) else {
            continue;
        };
        let mut collat = HashMap::new();
        for src in &metric.collateral_sources {
            if src.symbol_count > 0 {
                collat.insert(src.file_id.0.to_string(), src.symbol_count);
            }
        }
        if !collat.is_empty() {
            out.insert(path.to_string(), collat);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Table printing + test
// ---------------------------------------------------------------------------

fn print_table(rows: &[(&str, f64)]) {
    println!("{:<48} | {:>8}", "phase", "ms");
    println!("{}", "-".repeat(59));
    for (name, ms) in rows {
        println!("{:<48} | {:>8.1}", name, ms);
    }
}

#[test]
#[ignore = "synthetic 520-header stress fixture; run explicitly for stage-5 tail timing"]
fn payload_tail_phase_timing() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();
    generate_fixture(root);

    let marks = Arc::new(Mutex::new(CollateralEventMarks::default()));
    let mut pipeline = Pipeline::new(root, None)
        .unwrap()
        .with_progress(track_collateral(&marks));

    // Stages 1-4 untimed (the A0 target is the stage-5 tail only).
    pipeline.ensure_include_graph().unwrap();
    pipeline.ensure_index().unwrap();
    let reduced = pipeline.get_or_build_reduced().unwrap();
    pipeline.ensure_callgraph().unwrap();

    let stage5 = measure_load_stage(&mut pipeline, &marks);
    let assembly = run_assembly(&mut pipeline, &reduced);

    assert!(
        reduced.symbols.len() >= 1000,
        "fixture must hit the plan target of 1000+ symbols, got {}",
        reduced.symbols.len()
    );
    let load = pipeline.get_or_build_load().unwrap();
    assert!(
        load.file_metrics.len() >= 500,
        "fixture must hit the plan target of 500+ files, got {}",
        load.file_metrics.len()
    );
    assert!(
        assembly.justification_entries > 0,
        "justification payload must be populated on this fixture"
    );

    println!(
        "TAIL_BENCH headers={HEADER_COUNT} tus={TU_COUNT} reduced_symbols={} \
         file_metrics={} justification_entries={} plans={} conservative_rows={}",
        reduced.symbols.len(),
        load.file_metrics.len(),
        assembly.justification_entries,
        assembly.plans,
        assembly.proposal_rows,
    );
    let mut rows: Vec<(&str, f64)> = vec![
        (
            "symbol_pass+prep (combined, <= first event)",
            stage5.symbol_and_prep_ms,
        ),
        ("file_pass (event span)", stage5.file_event_span_ms),
        ("save_load_artifacts+cache (combined)", stage5.save_tail_ms),
        ("ensure_load total", stage5.total_ms),
    ];
    rows.extend(assembly.rows.iter().map(|(n, ms)| (*n, *ms)));
    print_table(&rows);
}

// ---------------------------------------------------------------------------
// Cross-process export byte-stability lock (M23.5 T3-B, item 2).
//
// T3-A verdict sec 8 / finding 1 deferred this lock to unit B ("before
// T4.1"): IV-5 requires `--export` to stay byte-stable across PROCESSES.
// Two calls in one process cannot detect per-process hash-seed,
// map-iteration-order, or address-derived nondeterminism, so each export
// here is its OWN OS process: a separate `std::process::Command`
// invocation of the real `ariadne` binary, running the export branch of
// `ui.rs` (build_ui_payload + export_standalone_html; no server, no port).
// This is the crate's only in-touch-set integration target, which is why
// the lock lives beside the bench rather than in a new test file.
// ---------------------------------------------------------------------------

/// Cycle-bearing mini repo: a.h <-> b.h mutual includes form one nontrivial
/// SCC (group size 2), pulled into the load stage by one TU. Deliberately
/// tiny - the property locked is byte-stability across processes, not
/// throughput.
fn generate_cyclic_export_fixture(root: &Path) {
    let include_dir = root.join("include");
    let src_dir = root.join("src");
    fs::create_dir_all(&include_dir).unwrap();
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        include_dir.join("a.h"),
        "#pragma once\n#include \"b.h\"\nstruct A { B* peer; int value; };\n",
    )
    .unwrap();
    fs::write(
        include_dir.join("b.h"),
        "#pragma once\n#include \"a.h\"\nstruct B { A* peer; int value; };\n",
    )
    .unwrap();
    fs::write(
        src_dir.join("tu.cpp"),
        "#include \"a.h\"\nstruct Tu { A* entry; };\nvoid run_entry(A* a) { (void)a; }\n",
    )
    .unwrap();
    let root_str = root.to_string_lossy().replace('\\', "/");
    fs::write(
        root.join("compile_commands.json"),
        format!(
            "[\n    {{ \"directory\": \"{root_str}\", \"command\": \
             \"clang++ -Iinclude -c src/tu.cpp\", \"file\": \"src/tu.cpp\" }}\n]\n"
        ),
    )
    .unwrap();
}

/// One genuine OS process: the built `ariadne` binary running
/// `ui <repo> --export <path>` to completion.
fn run_cold_cli_export(repo: &Path, export_path: &Path) {
    let cli_binary = env!("CARGO_BIN_EXE_ariadne");
    let run = std::process::Command::new(cli_binary)
        .arg("ui")
        .arg(repo)
        .arg("--export")
        .arg(export_path)
        .output()
        .unwrap_or_else(|spawn_failure| panic!("export process failed to start: {spawn_failure}"));
    assert!(
        run.status.success(),
        "ariadne ui --export exited {}: {}",
        run.status,
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn export_bytes_are_identical_across_two_separate_processes() {
    let dir = tempfile::TempDir::new().unwrap();
    let repo = dir.path().join("cyclicrepo");
    generate_cyclic_export_fixture(&repo);
    let first_export = dir.path().join("export-first.html");
    let second_export = dir.path().join("export-second.html");

    run_cold_cli_export(&repo, &first_export);
    // Wipe every persisted stage state (incl. the TU shard cache, all under
    // `<repo>/artifacts/`) so the second process recomputes symmetrically
    // from scratch instead of reading the first process's cache.
    fs::remove_dir_all(repo.join("artifacts")).unwrap();
    run_cold_cli_export(&repo, &second_export);

    let first_bytes = fs::read(&first_export).unwrap();
    let second_bytes = fs::read(&second_export).unwrap();

    // Non-vacuity: the payload region must actually carry the cycle-group
    // currency this milestone introduced (no asset JS contains this key,
    // so a payload-only substring is a real proof, not a template echo).
    let data_marker = b"window.__ARIADNE_DATA__ = ";
    let payload_start = first_bytes
        .windows(data_marker.len())
        .position(|window| window == data_marker)
        .expect("export must embed the payload behind the data marker");
    let group_size_key = b"cycle_group_size";
    assert!(
        first_bytes[payload_start..]
            .windows(group_size_key.len())
            .any(|window| window == group_size_key),
        "payload region must carry cycle_group_size after the T3-B rewiring \
         (a payload with no cycle-group data could never falsify this lock)"
    );

    // The lock itself: byte equality across the two separate processes.
    assert_eq!(
        first_bytes.len(),
        second_bytes.len(),
        "the two exports differ in length: {} vs {} bytes",
        first_bytes.len(),
        second_bytes.len()
    );
    let first_difference = first_bytes
        .iter()
        .zip(second_bytes.iter())
        .position(|(first, second)| first != second);
    assert_eq!(
        first_difference, None,
        "export bytes diverged at byte offset {:?} (payload starts at byte {payload_start})",
        first_difference
    );
}
