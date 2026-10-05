//! Parameterized COPY of the pure synthetic-corpus generators from
//! `crates/pipeline/tests/load_perf_bench.rs` (M27 T1a).
//!
//! Duplication is deliberate and shape-allowed (shape sec. 9):
//! `load_perf_bench.rs` stays byte-identical at its 866-line ceiling, and
//! this module carries the same generator shape as free functions
//! parameterized by `(header_count, tu_count)` instead of the
//! load_perf_bench `HEADER_COUNT`/`TU_COUNT` consts (520/40). The consumer
//! is `crates/pipeline/tests/boot_stages_bench.rs` (start setting 2000/400
//! per shape sec. 3.3; ratifiable at the evidence review).
//!
//! Repository shape (unchanged from the source): a cycle-tolerant modular
//! fanout (`h_i` includes `h_(i+10..i+13)` mod `header_count`) plus TUs
//! each including 5 spread headers. The closure walk is visited-guarded and
//! the include graph treats cycles as a first-class case.

use std::fs;
use std::path::Path;
use std::time::Instant;

/// Fanout base/st width identical to load_perf_bench: each header includes
/// `FANOUT_WIDTH` successors starting `FANOUT_BASE` ahead, modulo the
/// header count, so the walk wraps into the strongly-connected component.
const FANOUT_BASE: usize = 10;
const FANOUT_WIDTH: usize = 4;
/// TU root stride, identical to load_perf_bench. Must be coprime with
/// `header_count` so TU roots plus the strongly-connected header fanout
/// reach every header (holds for both 520 and the 2000 start setting).
const TU_ROOT_STRIDE: usize = 103;
const TU_ROOT_COUNT: usize = 5;

/// Fanout targets of header `index`: `h_(index+10)..h_(index+13)` mod
/// `header_count`.
pub fn fanout_targets(index: usize, header_count: usize) -> Vec<usize> {
    (0..FANOUT_WIDTH)
        .map(|offset| (index + FANOUT_BASE + offset) % header_count)
        .collect()
}

/// One stress header: 2 structs with cross-header pointer members plus 2
/// cross-calling free functions (4 symbols/header) in ~20 lines. Forward
/// pointer/reference uses keep the cycle-tolerant layout valid C++ even
/// when a cyclic `#include` is skipped by the pragma guard.
pub fn header_source(index: usize, header_count: usize) -> String {
    let targets = fanout_targets(index, header_count);
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

/// Write `header_count` fanout headers into `include_dir`.
pub fn write_headers(include_dir: &Path, header_count: usize) {
    for index in 0..header_count {
        fs::write(
            include_dir.join(format!("h{index}.h")),
            header_source(index, header_count),
        )
        .unwrap();
    }
}

/// One translation unit: a struct holding pointers into 5 spread headers
/// plus one driver function calling across them.
pub fn tu_source(index: usize, header_count: usize) -> String {
    let roots: Vec<usize> = (0..TU_ROOT_COUNT)
        .map(|slot| (index + slot * TU_ROOT_STRIDE) % header_count)
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
pub fn write_compile_commands(root: &Path, tu_count: usize) {
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

/// Generate the whole synthetic repository (`header_count` fanout headers,
/// `tu_count` TUs, matching compile_commands.json) under `root`.
pub fn generate_fixture_sized(root: &Path, header_count: usize, tu_count: usize) {
    let include_dir = root.join("include");
    let src_dir = root.join("src");
    fs::create_dir_all(&include_dir).unwrap();
    fs::create_dir_all(&src_dir).unwrap();
    write_headers(&include_dir, header_count);
    for index in 0..tu_count {
        fs::write(src_dir.join(format!("tu{index}.cpp")), tu_source(index, header_count)).unwrap();
    }
    write_compile_commands(root, tu_count);
}

/// Milliseconds since `started`, for the phase table.
pub fn millis(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1000.0
}
