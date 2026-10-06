//! Stage-5 collateral RSS-curve bench (M28 `stage5-collateral-memory-ceiling`
//! T0a; shape `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`
//! sec. 3.1 + sec. 6). Measurement hop: this file lands the peak-RSS oracle
//! the milestone is named after, before any fix commit exists.
//!
//! Rows follow the `label | value` grammar of `tools/bench_compare.py:88-109`
//! (exactly one pipe, finite numeric value). RSS values are MiB and the
//! profile is INSIDE every label (E5/M27-E3: the profile confounds any
//! measurement, so every row carries a debug or release identity and the
//! evidence table is built from BOTH profile runs). The default `bench_compare`
//! tuple never sees these rows - they live in a different bench file and are
//! only produced by the ignored curve tests.
//!
//! Peak-RSS source (shape sec. 3.1): the process peak working set, read via
//! raw `extern "system"` FFI to `K32GetProcessMemoryInfo` (psapi) - zero new
//! dependencies. A windows-surface crate check was performed first and the
//! result is journaled: `windows-sys` appears in `Cargo.lock` only as a
//! transitive dependency (getrandom/rustix/tempfile/tokio edges), which is
//! NOT reachability - it cannot be named from this test target, so the raw
//! FFI fallback is the route. Linux (ubuntu CI) reads `VmHWM`/`VmRSS` from
//! `/proc/self/status` - a real live probe, not a stub. Any other platform
//! returns `None`; every consumer degrades LOUDLY (stderr note + suppressed
//! RSS rows + failing oracle test), never silently.
//!
//! ## Fixture (shape sec. 3.1: hub-heavy, unlike the 4-wide fanout precedent)
//!
//! Four hub headers form a call ring (hub h's slot k calls hub (h+1)'s slot
//! k, the ring advancing one slot per full cycle), so every hub function's
//! semantic closure reaches EVERY hub function - the closure set scales with
//! the corpus (the E1 shape). Hub slot count is derived from the leaf count,
//! keeping the reachable hub cluster at corpus scale at any N. Every leaf
//! header includes `hub0.h` (the high-fanin hub) and calls one hub function,
//! so each leaf symbol's semantic closure and each leaf's physical expansion
//! are corpus-scale. This is the accumulation shape the field OOM hit
//! (kickoff sec. 2 E1/E2), not the bounded fanout of `load_perf_bench.rs`.
//!
//! Scale is parameterized by `STAGE5_MEM_BENCH_FILES` (default 1280 = the
//! PROPOSED N0 only - DP-4 ratifies N0 at the written evidence review; this
//! bench measures and proposes, it never ratifies). Curve scales {N0/2, N0}.
//!
//! Row isolation: peak RSS is process-lifetime monotonic, so a peak is only
//! honest for a process that ran exactly one stage-5 boot. Every curve row
//! therefore comes from a fresh child process (`current_exe --exact
//! --ignored`): the scale tests and the growth-ratio test spawn the
//! orchestrated `stage5_rss_scale_runner`, which runs only when
//! `STAGE5_MEM_BENCH_FILES` is set (it skips with a loud note inside the
//! plain `--ignored` run, so no shared-harness process ever prints a peak).
//! The warm rows come from a fresh measure child over a fixture populated by
//! a cold-prep child, so the row is the A7 read side, not a cold boot in
//! disguise.
//!
//! Run with (debug curve; release = same form after `cargo test --release`):
//!   python tools/distill.py -- cargo test -p ariadne-cli --test \
//!     stage5_memory_bench -- --ignored --nocapture --test-threads=1

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use ariadne_graph::{IncludeGraph, LoadMemo, ReducedSemanticModel};
use ariadne_pipeline::{LoadMilestone, Pipeline, ProgressEvent};
use ariadne_report::LoadReport;

// ---------------------------------------------------------------------------
// RSS probe: process peak working set + current working set, zero deps
// ---------------------------------------------------------------------------

#[cfg(windows)]
mod rss_probe {
    use core::ffi::c_void;

    /// `PROCESS_MEMORY_COUNTERS` layout; only the first three fields are
    /// consumed. `usize`-wide fields match the typedef.
    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_size: usize,
        quota_paged_pool_size: usize,
        quota_peak_non_paged_pool_size: usize,
        quota_non_paged_pool_size: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    #[link(name = "psapi")]
    extern "system" {
        fn K32GetProcessMemoryInfo(
            process: *mut c_void,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }

    /// The documented `GetCurrentProcess()` pseudo handle. Pseudo handles
    /// are never closed.
    fn current_process_pseudo_handle() -> *mut c_void {
        (-1isize) as *mut c_void
    }

    /// One live probe: (peak working set, current working set) in bytes.
    pub fn sample() -> Option<(u64, u64)> {
        let mut counters: ProcessMemoryCounters = unsafe { core::mem::zeroed() };
        counters.cb = core::mem::size_of::<ProcessMemoryCounters>() as u32;
        let ok = unsafe {
            K32GetProcessMemoryInfo(
                current_process_pseudo_handle(),
                &mut counters,
                counters.cb,
            )
        };
        if ok == 0 {
            return None;
        }
        Some((
            counters.peak_working_set_size as u64,
            counters.working_set_size as u64,
        ))
    }
}

#[cfg(target_os = "linux")]
mod rss_probe {
    /// One live probe from procfs: (VmHWM peak, VmRSS current) in bytes.
    pub fn sample() -> Option<(u64, u64)> {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let peak_kib = status_field_kib(&status, "VmHWM:")?;
        let rss_kib = status_field_kib(&status, "VmRSS:")?;
        Some((peak_kib * 1024, rss_kib * 1024))
    }

    fn status_field_kib(status: &str, key: &str) -> Option<u64> {
        status
            .lines()
            .find(|line| line.starts_with(key))?
            .split_whitespace()
            .nth(1)?
            .parse::<u64>()
            .ok()
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod rss_probe {
    /// Unsupported platform: no probe. Consumers fail loudly on `None` - a
    /// silently-absent oracle is exactly the E7 acceptance failure this
    /// milestone gates against.
    pub fn sample() -> Option<(u64, u64)> {
        None
    }
}

/// Live process peak working set, bytes (`None` = probe unavailable).
fn peak_working_set_bytes() -> Option<u64> {
    rss_probe::sample().map(|(peak, _workset)| peak)
}

/// Live process working set sample, bytes (per-phase checkpoints).
fn working_set_bytes() -> Option<u64> {
    rss_probe::sample().map(|(_peak, workset)| workset)
}

/// The probe locks the oracle itself: a known ~256 MiB touched slab MUST
/// move the peak working set by at least ~200 MiB. Fails loudly on any
/// platform or stub state where the probe does not track allocation.
#[test]
fn peak_working_set_bytes_tracks_allocation() {
    const SLAB_BYTES: usize = 256 * 1024 * 1024;
    const MIN_TRACKED_GROWTH_BYTES: u64 = 200 * 1024 * 1024;
    const PAGE_BYTES: usize = 4096;
    const PROBE_UNAVAILABLE: &str = "RSS probe must be live (windows psapi / \
        linux procfs) - refusing to bench against a stub oracle";

    let peak_before = peak_working_set_bytes().expect(PROBE_UNAVAILABLE);
    let mut slab = vec![0u8; SLAB_BYTES];
    for page in slab.chunks_mut(PAGE_BYTES) {
        page[0] = 1;
    }
    let peak_after = peak_working_set_bytes().expect(PROBE_UNAVAILABLE);
    let growth_bytes = peak_after.saturating_sub(peak_before);
    assert!(
        growth_bytes >= MIN_TRACKED_GROWTH_BYTES,
        "peak working set must move by >= {} bytes after touching a {} byte \
         slab, moved {} bytes (probe is a stub or not live)",
        MIN_TRACKED_GROWTH_BYTES,
        SLAB_BYTES,
        growth_bytes
    );
}

// ---------------------------------------------------------------------------
// Scale + profile parameters
// ---------------------------------------------------------------------------

/// PROPOSED N0 only (DP-4: ratified at the evidence review from this hop's
/// measured table; a proposal, not a gate constant).
const DEFAULT_N0_FILES: usize = 1280;
/// Floor so the N0/2 half-scale stays a real fixture (hubs + TUs + leaves).
const MIN_CURVE_FILES: usize = 96;
const SCALE_FILES_ENV: &str = "STAGE5_MEM_BENCH_FILES";
const WARM_DIR_ENV: &str = "STAGE5_MEM_BENCH_WARM_DIR";
const RUNNER_TEST: &str = "stage5_rss_scale_runner";
const WARM_PREPARE_TEST: &str = "stage5_warm_prepare";
const WARM_MEASURE_TEST: &str = "stage5_warm_measure";

const CURVE_TU_COUNT: usize = 40;
const HUB_FILE_COUNT: usize = 4;
const WARM_TOTAL_FILES: usize = 40;
const WARM_TU_COUNT: usize = 4;
const MEMO_SAMPLE_TARGET: usize = 32;
const ROW_LABEL_WIDTH: usize = 64;

fn profile_label() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

fn proposed_file_count() -> usize {
    match std::env::var(SCALE_FILES_ENV) {
        Ok(raw) => raw
            .trim()
            .parse::<usize>()
            .unwrap_or_else(|_| {
                panic!("{SCALE_FILES_ENV} must be an integer file count, got {raw:?}")
            })
            .max(MIN_CURVE_FILES),
        Err(_) => DEFAULT_N0_FILES,
    }
}

// ---------------------------------------------------------------------------
// Synthetic hub-heavy fixture (sanctioned verbatim-duplication pattern,
// `payload_tail_bench.rs:8-10` precedent; generator lives HERE, scaled).
// ---------------------------------------------------------------------------

/// Reachable hub-cluster functions ~= `HUB_FILE_COUNT * slots`; choosing
/// slots = leaves / HUB_FILE_COUNT keeps the corpus-scale closure property
/// (the E1 shape) at any N.
fn hub_slots_for(leaf_count: usize) -> usize {
    (leaf_count / HUB_FILE_COUNT).max(4)
}

fn hub_source(hub_index: usize, slots: usize, leaf_count: usize) -> String {
    let next_hub = (hub_index + 1) % HUB_FILE_COUNT;
    let far_hub = (hub_index + 2) % HUB_FILE_COUNT;
    let mut source = String::from("#pragma once\n");
    source.push_str(&format!(
        "#include \"hub{next_hub}.h\"\n#include \"hub{far_hub}.h\"\n"
    ));
    // High fanin: each hub pulls in its round-robin slice of the leaves, so
    // every file reached from any hub sees the whole corpus (the E1 shape,
    // where common headers drag the entire include world into every file's
    // expansion, and every file therefore enters the reduced model).
    for slice_leaf in (hub_index..leaf_count).step_by(HUB_FILE_COUNT) {
        source.push_str(&format!("#include \"l{slice_leaf}.h\"\n"));
    }
    for slot in 0..slots {
        // The ring advances one slot only when wrapping hub3 -> hub0, so one
        // full cycle shifts the slot by 1: every (hub, slot) pair reaches
        // every other pair at any slot count (no parity/coprime holes).
        let call_slot = if next_hub == 0 {
            (slot + 1) % slots
        } else {
            slot
        };
        source.push_str(&format!(
            "struct H{hub_index}_{slot} {{\n    H{next_hub}_{slot}* member;\n    long value;\n}};\n"
        ));
        source.push_str(&format!(
            "void hs{hub_index}_{slot}(long seed) {{\n    hs{next_hub}_{call_slot}(seed + 1);\n}}\n"
        ));
    }
    source
}

/// One leaf: includes the high-fanin hub and calls into the hub ring, so
/// its semantic closure and its physical expansion are hub-cluster sized.
fn leaf_source(index: usize, slots: usize) -> String {
    let target_slot = index % slots;
    format!(
        "#pragma once\n#include \"hub0.h\"\n\
         struct L{index} {{\n    H0_{target_slot}* link;\n    long value;\n}};\n\
         void lf{index}(long seed) {{\n    hs0_{target_slot}(seed);\n}}\n"
    )
}

fn tu_source(index: usize, leaf_count: usize, slots: usize) -> String {
    let root_a = (index * 97 + 5) % leaf_count;
    let root_b = (index * 97 + 41) % leaf_count;
    let root_c = (index * 97 + 77) % leaf_count;
    let slot_a = index % slots;
    let slot_c = (index + slots / 2) % slots;
    format!(
        "#include \"hub0.h\"\n#include \"hub1.h\"\n\
         #include \"l{root_a}.h\"\n#include \"l{root_b}.h\"\n#include \"l{root_c}.h\"\n\
         struct Tu{index} {{\n    L{root_a}* a0;\n    L{root_b}* a1;\n    L{root_c}* a2;\n}};\n\
         void tu_run{index}(long seed) {{\n    hs1_{slot_a}(seed);\n    hs3_{slot_c}(seed);\n}}\n"
    )
}

/// compile_commands.json mirroring the precedent benches: forward-slash
/// absolute directory, `clang++ -Iinclude -c src/tuN.cpp`.
fn write_compile_commands(root: &Path, tu_count: usize) {
    let root_str = root.to_string_lossy().replace('\\', "/");
    let mut entries = Vec::new();
    for index in 0..tu_count {
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

fn generate_fixture(root: &Path, total_files: usize, tu_count: usize) {
    let leaf_count = total_files.saturating_sub(HUB_FILE_COUNT + tu_count);
    assert!(
        leaf_count >= 8,
        "fixture of {total_files} files with {tu_count} TUs leaves only \
         {leaf_count} leaves - too small for the hub-heavy shape"
    );
    let slots = hub_slots_for(leaf_count);
    let include_dir = root.join("include");
    let src_dir = root.join("src");
    fs::create_dir_all(&include_dir).unwrap();
    fs::create_dir_all(&src_dir).unwrap();
    for hub_index in 0..HUB_FILE_COUNT {
        fs::write(
            include_dir.join(format!("hub{hub_index}.h")),
            hub_source(hub_index, slots, leaf_count),
        )
        .unwrap();
    }
    for leaf_index in 0..leaf_count {
        fs::write(
            include_dir.join(format!("l{leaf_index}.h")),
            leaf_source(leaf_index, slots),
        )
        .unwrap();
    }
    for tu_index in 0..tu_count {
        fs::write(
            src_dir.join(format!("tu{tu_index}.cpp")),
            tu_source(tu_index, leaf_count, slots),
        )
        .unwrap();
    }
    write_compile_commands(root, tu_count);
}

// ---------------------------------------------------------------------------
// Per-phase RSS checkpoints, captured from the load-stage progress events
// (stage_load.rs: SymbolsAnalyzed = symbol-pass end, CollateralFile stream =
// file pass, ArtifactsSaved = save-tail end).
// ---------------------------------------------------------------------------

#[derive(Default)]
struct RssCheckpoints {
    symbol_pass_end_bytes: Option<u64>,
    /// Max working set sampled across the CollateralFile event stream: the
    /// honest tail of the per-file loop where the A3 accumulation peaks.
    file_pass_max_workset_bytes: Option<u64>,
    file_pass_end_bytes: Option<u64>,
    save_tail_end_bytes: Option<u64>,
}

fn track_stage5_rss(
    marks: &Arc<Mutex<RssCheckpoints>>,
) -> impl FnMut(&ProgressEvent) + Send {
    let marks = Arc::clone(marks);
    move |event| match event {
        ProgressEvent::LoadMilestone(LoadMilestone::SymbolsAnalyzed { .. }) => {
            let sample = working_set_bytes();
            marks.lock().unwrap().symbol_pass_end_bytes = sample;
        }
        ProgressEvent::CollateralFile { done, total } => {
            let sample = working_set_bytes();
            let mut guard = marks.lock().unwrap();
            if let Some(bytes) = sample {
                guard.file_pass_max_workset_bytes = Some(
                    guard
                        .file_pass_max_workset_bytes
                        .map_or(bytes, |seen| seen.max(bytes)),
                );
                if *done + 1 >= *total {
                    guard.file_pass_end_bytes = Some(bytes);
                }
            }
        }
        ProgressEvent::LoadMilestone(LoadMilestone::ArtifactsSaved) => {
            let sample = working_set_bytes();
            marks.lock().unwrap().save_tail_end_bytes = sample;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Row printer: the bounded `label | value` table (R9; one pipe, MiB)
// ---------------------------------------------------------------------------

fn print_row(label: &str, value: f64) {
    // `+ 0.0` normalizes negative zero ("-0.0" parses finite but reads as a
    // signed zero in the table; the value it came from is not negative).
    let value = value + 0.0;
    println!("{label:<ROW_LABEL_WIDTH$} | {value:>12.1}");
}

fn mib(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn delta_mib(after: u64, before: u64) -> f64 {
    (after as f64 - before as f64) / (1024.0 * 1024.0)
}

fn print_optional_mib_row(label: &str, bytes: Option<u64>) {
    if let Some(bytes) = bytes {
        print_row(label, mib(bytes));
    }
}

fn print_delta_row(label: &str, after: Option<u64>, before: Option<u64>) {
    if let (Some(after), Some(before)) = (after, before) {
        print_row(label, delta_mib(after, before));
    }
}

fn print_rss_rows(
    profile: &str,
    scale: usize,
    prep: Option<u64>,
    marks: &RssCheckpoints,
    post_return: Option<u64>,
    peak: Option<u64>,
) {
    let suffix = format!("({profile}, N={scale})");
    print_optional_mib_row(&format!("RSS checkpoint prep {suffix}"), prep);
    print_optional_mib_row(
        &format!("RSS checkpoint symbol-pass end {suffix}"),
        marks.symbol_pass_end_bytes,
    );
    print_optional_mib_row(
        &format!("RSS checkpoint file-pass tail {suffix}"),
        marks.file_pass_max_workset_bytes,
    );
    print_optional_mib_row(
        &format!("RSS checkpoint save tail {suffix}"),
        marks.save_tail_end_bytes,
    );
    print_optional_mib_row(
        &format!("RSS checkpoint post-return {suffix}"),
        post_return,
    );
    print_delta_row(
        &format!("symbol-pass RSS delta {suffix}"),
        marks.symbol_pass_end_bytes,
        prep,
    );
    print_delta_row(
        &format!("file-pass RSS delta {suffix}"),
        marks.file_pass_max_workset_bytes,
        marks.symbol_pass_end_bytes,
    );
    print_delta_row(
        &format!("save-tail RSS delta {suffix}"),
        marks.save_tail_end_bytes,
        marks.file_pass_end_bytes,
    );
    print_delta_row(
        &format!("post-return RSS delta {suffix}"),
        post_return,
        marks.save_tail_end_bytes,
    );
    print_optional_mib_row(&format!("stage5 peak RSS {suffix}"), peak);
}

fn print_wall_rows(
    profile: &str,
    scale: usize,
    boot_wall_seconds: f64,
    load_wall_seconds: f64,
) {
    let suffix = format!("({profile}, N={scale})");
    print_row(
        &format!("boot stages 1-4 wall seconds {suffix}"),
        boot_wall_seconds,
    );
    print_row(
        &format!("ensure_load wall seconds {suffix}"),
        load_wall_seconds,
    );
}

// ---------------------------------------------------------------------------
// Memo attribution (shape sec. 6 block 4): entry counts x sampled entry
// size, arithmetic printed in the table. The stage's own memo is internal
// to `ensure_load`; the doc'd cache contract (load_caches.rs:46-64) fixes
// entry counts (one per distinct seed / def-file ever queried), and entry
// CONTENT is sampled from an equal-generation memo warmed through the same
// public queries.
// ---------------------------------------------------------------------------

fn sampled_closure_elems(
    memo: &LoadMemo,
    model: &ReducedSemanticModel,
) -> (u64, u64) {
    let stride = (model.symbols.len() / MEMO_SAMPLE_TARGET).max(1);
    let mut total_elems = 0u64;
    let mut samples = 0u64;
    for (position, seed) in model.symbols.keys().enumerate() {
        if position % stride != 0 {
            continue;
        }
        total_elems += memo.cached_single_seed_closure(model, seed).len() as u64;
        samples += 1;
    }
    (total_elems, samples)
}

fn sampled_expansion_elems(
    memo: &LoadMemo,
    graph: &IncludeGraph,
    model: &ReducedSemanticModel,
) -> (u64, u64) {
    let stride = (model.file_symbols.len() / MEMO_SAMPLE_TARGET).max(1);
    let mut total_elems = 0u64;
    let mut samples = 0u64;
    for (position, file_id) in model.file_symbols.keys().enumerate() {
        if position % stride != 0 {
            continue;
        }
        let expansion = memo.cached_physical_expansion(graph, model, *file_id);
        total_elems += expansion.len() as u64;
        samples += 1;
    }
    (total_elems, samples)
}

#[allow(clippy::too_many_arguments)]
fn print_attribution_rows(
    profile: &str,
    scale: usize,
    pipeline: &mut Pipeline,
    model: &ReducedSemanticModel,
    report: &LoadReport,
) {
    let suffix = format!("({profile}, N={scale})");
    // Cloned inputs, equal generation: the SAME closures the stage analyzed.
    let (graph, closures) = pipeline.get_or_build_include_graph().unwrap();
    let memo = LoadMemo::with_closures(closures);

    let (closure_elems, closure_samples) = sampled_closure_elems(&memo, model);
    assert!(closure_samples > 0, "semantic-closure sampling must run");
    let closure_entries = report.symbol_metrics.len();
    let avg_closure_elems = closure_elems as f64 / closure_samples as f64;
    print_row(
        &format!("memo semantic-closure entries {suffix}"),
        closure_entries as f64,
    );
    print_row(
        &format!("memo semantic-closure sampled seeds {suffix}"),
        closure_samples as f64,
    );
    print_row(
        &format!("memo semantic-closure sampled entry size {suffix}"),
        avg_closure_elems,
    );
    print_row(
        &format!("memo semantic-closure est aggregate refs {suffix}"),
        avg_closure_elems * closure_entries as f64,
    );

    let (expansion_elems, expansion_samples) =
        sampled_expansion_elems(&memo, &graph, model);
    assert!(expansion_samples > 0, "physical-expansion sampling must run");
    let expansion_entries = report.file_metrics.len();
    let avg_expansion_elems = expansion_elems as f64 / expansion_samples as f64;
    print_row(
        &format!("memo physical-expansion entries {suffix}"),
        expansion_entries as f64,
    );
    print_row(
        &format!("memo physical-expansion sampled files {suffix}"),
        expansion_samples as f64,
    );
    print_row(
        &format!("memo physical-expansion sampled entry size {suffix}"),
        avg_expansion_elems,
    );
    print_row(
        &format!("memo physical-expansion est aggregate refs {suffix}"),
        avg_expansion_elems * expansion_entries as f64,
    );
}

// ---------------------------------------------------------------------------
// Curve runner + isolated child spawning (peak RSS is process-monotonic:
// one stage-5 boot per process, medians taken ACROSS process invocations)
// ---------------------------------------------------------------------------

fn run_curve_at_scale(total_files: usize) {
    let profile = profile_label();
    let dir = tempfile::TempDir::new().unwrap();
    generate_fixture(dir.path(), total_files, CURVE_TU_COUNT);
    let marks = Arc::new(Mutex::new(RssCheckpoints::default()));
    let mut pipeline = Pipeline::new(dir.path(), None)
        .unwrap()
        .with_progress(track_stage5_rss(&marks));

    let boot_started = Instant::now();
    pipeline.ensure_include_graph().unwrap();
    pipeline.ensure_index().unwrap();
    let reduced = pipeline.get_or_build_reduced().unwrap();
    pipeline.ensure_callgraph().unwrap();
    let boot_wall_seconds = boot_started.elapsed().as_secs_f64();

    let prep_bytes = working_set_bytes();
    let load_started = Instant::now();
    pipeline.ensure_load().unwrap();
    let load_wall_seconds = load_started.elapsed().as_secs_f64();
    let post_return_bytes = working_set_bytes();
    // Read the peak BEFORE attribution clones: the row must carry the
    // stage-5 peak, not the bench's own after-measurement sampling memory.
    let peak_bytes = peak_working_set_bytes();
    let report = pipeline.get_or_build_load().unwrap();

    println!(
        "STAGE5_BENCH profile={profile} files={total_files} symbols={} file_metrics={}",
        reduced.symbols.len(),
        report.file_metrics.len()
    );
    assert!(
        peak_bytes.is_some(),
        "RSS probe must be live on supported platforms (see file header)"
    );
    assert!(
        reduced.symbols.len() >= total_files,
        "hub-heavy fixture of {total_files} files must reach >= {total_files} \
         reduced symbols, got {}",
        reduced.symbols.len()
    );
    assert!(
        report.file_metrics.len() >= total_files * 9 / 10,
        "stage 5 must analyze ~all {total_files} files, got {}",
        report.file_metrics.len()
    );
    assert!(
        marks.lock().unwrap().symbol_pass_end_bytes.is_some(),
        "cold rebuild must emit SymbolsAnalyzed for the symbol-pass checkpoint"
    );

    let guard = marks.lock().unwrap();
    print_rss_rows(
        profile,
        total_files,
        prep_bytes,
        &guard,
        post_return_bytes,
        peak_bytes,
    );
    drop(guard);
    print_wall_rows(
        profile,
        total_files,
        boot_wall_seconds,
        load_wall_seconds,
    );
    print_attribution_rows(
        profile,
        total_files,
        &mut pipeline,
        &reduced,
        &report,
    );
}

fn decode_output(stdout: &[u8], stderr: &[u8]) -> String {
    let mut text = String::from_utf8_lossy(stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(stderr));
    text
}

fn tail_lines(text: &str, count: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(count);
    lines[start..].join("\n")
}

/// Run the orchestrated runner test in a fresh process at `scale_files`,
/// returning its captured output. Children never nest (the exact-name filter
/// matches only the runner, which never spawns), so nothing can recurse.
fn run_isolated_runner_child(scale_files: usize) -> String {
    let executable = std::env::current_exe().expect("test binary path");
    let child = Command::new(executable)
        .arg(RUNNER_TEST)
        .arg("--exact")
        .arg("--ignored")
        .arg("--nocapture")
        .env(SCALE_FILES_ENV, scale_files.to_string())
        .output()
        .unwrap_or_else(|err| panic!("spawn isolated scale child: {err}"));
    let text = decode_output(&child.stdout, &child.stderr);
    assert!(
        child.status.success(),
        "isolated child {RUNNER_TEST} at N={scale_files} failed \
         (status {:?}):\n{}",
        child.status.code(),
        tail_lines(&text, 8)
    );
    text
}

/// Re-emit a child's bounded table rows (context line + `label | value`
/// rows) into this test's output. Harness noise lines are dropped; the
/// emitted table stays <= ~25 lines (R9).
fn emit_child_table_rows(text: &str) {
    let mut emitted = 0usize;
    for line in text.lines() {
        if line.starts_with("STAGE5_BENCH") || line.matches('|').count() == 1 {
            println!("{line}");
            emitted += 1;
        }
    }
    assert!(
        emitted > 1,
        "child printed no table rows:\n{}",
        tail_lines(text, 12)
    );
}

fn parse_peak_row(text: &str, scale: usize) -> Option<f64> {
    let wanted = format!("stage5 peak RSS ({}, N={scale})", profile_label());
    text.lines()
        .filter(|line| line.matches('|').count() == 1)
        .find_map(|line| {
            let (label, value) = line.split_once('|')?;
            if label.trim() == wanted {
                value.trim().parse::<f64>().ok()
            } else {
                None
            }
        })
}

fn isolated_peak_mib(scale_files: usize) -> f64 {
    let text = run_isolated_runner_child(scale_files);
    parse_peak_row(&text, scale_files).unwrap_or_else(|| {
        panic!(
            "isolated child at N={scale_files} printed no peak RSS row:\n{}",
            tail_lines(&text, 12)
        )
    })
}

fn half_scale_files() -> usize {
    (proposed_file_count() / 2).max(MIN_CURVE_FILES)
}

// ---------------------------------------------------------------------------
// Curve tests (shape sec. 6 blocks 1, 2, 4, 6 for THIS profile). Each scale
// boots in its own fresh process via the runner (peak RSS monotonicity);
// the tests forward the child's bounded table.
// ---------------------------------------------------------------------------

/// Orchestrated scale runner. Runs ONLY when `STAGE5_MEM_BENCH_FILES` is set
/// (the orchestrators always set it): inside the plain `--ignored` run it
/// skips with a loud note, so a shared harness process never gets the chance
/// to print a peak contaminated by an earlier boot. Standalone reviews set
/// the env var and `--exact` this test directly.
#[test]
#[ignore = "orchestrated by the scale/growth tests per scale; runs only with STAGE5_MEM_BENCH_FILES set"]
fn stage5_rss_scale_runner() {
    match std::env::var(SCALE_FILES_ENV) {
        Ok(raw) => {
            let scale = raw
                .trim()
                .parse::<usize>()
                .unwrap_or_else(|_| {
                    panic!("{SCALE_FILES_ENV} must be an integer file count, got {raw:?}")
                })
                .max(MIN_CURVE_FILES);
            run_curve_at_scale(scale);
        }
        Err(_) => println!(
            "stage5_rss_scale_runner: orchestrated - {SCALE_FILES_ENV} unset, \
             nothing measured here (peak RSS is process-lifetime monotonic; \
             the scale and growth-ratio tests spawn one isolated runner per \
             scale). Set {SCALE_FILES_ENV} and filter this test to run it."
        ),
    }
}

#[test]
#[ignore = "spawns one isolated half-scale boot; M28 T0a RSS curve (N0/2)"]
fn stage5_rss_curve_half_scale() {
    emit_child_table_rows(&run_isolated_runner_child(half_scale_files()));
}

#[test]
#[ignore = "spawns one isolated full-scale boot; M28 T0a RSS curve (N0)"]
fn stage5_rss_curve_full_scale() {
    emit_child_table_rows(&run_isolated_runner_child(proposed_file_count()));
}

/// I7b growth cap input: `RSS(N0)/RSS(N0/2)`, both peaks measured by their
/// own isolated runner processes. The children's full tables are consumed
/// here (only the ratio row is printed) - the scale rows come from the
/// scale tests' own children, so every label appears exactly once per run.
#[test]
#[ignore = "spawns two isolated full boots; M28 T0a growth-ratio row"]
fn stage5_rss_growth_ratio() {
    let n0 = proposed_file_count();
    let half = half_scale_files();
    let half_peak_mib = isolated_peak_mib(half);
    let full_peak_mib = isolated_peak_mib(n0);
    print_row(
        &format!(
            "stage5 RSS growth ratio ({}, N={}->{})",
            profile_label(),
            half,
            n0
        ),
        full_peak_mib / half_peak_mib,
    );
}

// ---------------------------------------------------------------------------
// Warm-deserialize rows (shape sec. 6 block 5): small-fixture artifact in a
// temp dir, cold pass in one CHILD process, deserialize measurement in a
// FRESH measure child (peak RSS monotonicity again - the measuring process
// must never have run the cold pass, or the peak is a cold boot in disguise).
// ---------------------------------------------------------------------------

struct TempDirGuard {
    path: PathBuf,
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn cold_boot_fixture(root: &Path) {
    generate_fixture(root, WARM_TOTAL_FILES, WARM_TU_COUNT);
    let mut pipeline = Pipeline::new(root, None).unwrap();
    // Cold path saves the load artifacts the measure child deserializes.
    pipeline.ensure_load().unwrap();
}

fn count_load_progress(counter: &Arc<AtomicUsize>) -> impl FnMut(&ProgressEvent) + Send {
    let counter = Arc::clone(counter);
    move |event| {
        if matches!(
            event,
            ProgressEvent::CollateralFile { .. } | ProgressEvent::LoadMilestone(_)
        ) {
            counter.fetch_add(1, Ordering::SeqCst);
        }
    }
}

fn file_mib(path: &Path) -> f64 {
    let metadata = fs::metadata(path)
        .unwrap_or_else(|err| panic!("expected committed-shape artifact at {path:?}: {err}"));
    mib(metadata.len())
}

fn spawn_warm_child(test_name: &str, fixture_root: &Path) -> String {
    let executable = std::env::current_exe().expect("test binary path");
    let child = Command::new(executable)
        .arg(test_name)
        .arg("--exact")
        .arg("--ignored")
        .arg("--nocapture")
        .env(WARM_DIR_ENV, fixture_root)
        .output()
        .unwrap_or_else(|err| panic!("spawn warm child {test_name}: {err}"));
    let text = decode_output(&child.stdout, &child.stderr);
    assert!(
        child.status.success(),
        "warm child {test_name} failed (status {:?}):\n{}",
        child.status.code(),
        tail_lines(&text, 8)
    );
    text
}

/// Orchestrator: cold-prep child populates the fixture, a FRESH measure
/// child deserializes it, and the child's bounded rows are forwarded. The
/// fixture lives in a temp dir (never committed) and is cleaned up here.
#[test]
#[ignore = "spawns cold-prep + fresh measure children; M28 T0a warm rows"]
fn stage5_warm_deserialize_peak() {
    let fixture_root =
        std::env::temp_dir().join(format!("stage5-warm-fixture-{}", std::process::id()));
    let _ = fs::remove_dir_all(&fixture_root);
    fs::create_dir_all(&fixture_root).unwrap();
    let _fixture_guard = TempDirGuard {
        path: fixture_root.clone(),
    };

    spawn_warm_child(WARM_PREPARE_TEST, &fixture_root);
    emit_child_table_rows(&spawn_warm_child(WARM_MEASURE_TEST, &fixture_root));
}

/// Fresh-process measure of the A7 read side: run ONLY orchestrated
/// (`STAGE5_MEM_BENCH_WARM_DIR` set by the orchestrator; the fixture must
/// already hold a cold-booted report). Standalone inside the plain
/// `--ignored` run it skips with a loud note so no shared-process peak is
/// ever printed.
#[test]
#[ignore = "orchestrated by stage5_warm_deserialize_peak; runs only with STAGE5_MEM_BENCH_WARM_DIR set"]
fn stage5_warm_measure() {
    let Some(dir) = std::env::var_os(WARM_DIR_ENV) else {
        println!(
            "stage5_warm_measure: orchestrated - {WARM_DIR_ENV} unset, nothing \
             measured (a shared-process peak would be cold-boot contaminated). \
             Run stage5_warm_deserialize_peak to orchestrate it."
        );
        return;
    };
    let fixture_root = PathBuf::from(dir);
    let profile = profile_label();

    let workset_before = working_set_bytes();
    let load_events = Arc::new(AtomicUsize::new(0));
    let mut pipeline = Pipeline::new(&fixture_root, None)
        .unwrap()
        .with_progress(count_load_progress(&load_events));
    let started = Instant::now();
    pipeline.ensure_load().unwrap();
    let wall_seconds = started.elapsed().as_secs_f64();
    assert_eq!(
        load_events.load(Ordering::SeqCst),
        0,
        "warm boot must deserialize report.bin - any CollateralFile or \
         LoadMilestone event means a cold rebuild ran instead"
    );
    let report = pipeline.get_or_build_load().unwrap();
    assert!(
        report.file_metrics.len() >= WARM_TOTAL_FILES * 9 / 10,
        "deserialized report must cover the fixture, got {} of {WARM_TOTAL_FILES}",
        report.file_metrics.len()
    );

    let load_dir = pipeline.manager.artifact_root.join("load");
    let bin_mib = file_mib(&load_dir.join("report.bin"));
    let json_mib = file_mib(&load_dir.join("report.json"));
    let peak = peak_working_set_bytes();
    let workset_after = working_set_bytes();
    drop(report);
    drop(pipeline);

    println!(
        "STAGE5_BENCH profile={profile} warm_fixture_files={WARM_TOTAL_FILES}"
    );
    let suffix = format!("({profile})");
    print_optional_mib_row(&format!("warm deserialize peak {suffix}"), peak);
    print_delta_row(
        &format!("warm deserialize delta {suffix}"),
        workset_after,
        workset_before,
    );
    print_row(&format!("warm deserialize wall seconds {suffix}"), wall_seconds);
    print_row(&format!("warm report.bin size {suffix}"), bin_mib);
    print_row(&format!("warm report.json size {suffix}"), json_mib);
}

/// Orchestrated by `stage5_warm_deserialize_peak` (fixture dir via env, kept
/// for the parent to measure and clean); standalone it self-cleans.
#[test]
#[ignore = "cold-boot fixture helper for the warm-deserialize rows"]
fn stage5_warm_prepare() {
    match std::env::var_os(WARM_DIR_ENV) {
        Some(dir) => cold_boot_fixture(Path::new(&dir)),
        None => {
            let dir = tempfile::TempDir::new().unwrap();
            cold_boot_fixture(dir.path());
        }
    }
}
