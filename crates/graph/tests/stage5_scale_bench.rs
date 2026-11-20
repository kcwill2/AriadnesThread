//! Stage-5 pass timing at scale on the synthetic load model. Run with:
//!   cargo test --release -p ariadne-graph --test stage5_scale_bench -- \
//!     --ignored --nocapture
//! Sizes come from `STAGE5_BENCH_HEADERS` / `_UNITS` / `_SYMBOLS` /
//! `_OUT_PERCENT` (defaults below). The loops replicate the pipeline's
//! symbol pass (grouped by defining file) and file pass over the public
//! analyzer entries, with the boot memo caps.

#[path = "common/synthetic_load_model.rs"]
mod synthetic_load_model;

use ariadne_graph::{CompactShape, LoadAnalyzer, LoadMemo};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;
use synthetic_load_model::SyntheticShape;

fn env_size(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|raw| raw.parse().ok()).unwrap_or(default)
}

fn bench_shape() -> SyntheticShape {
    SyntheticShape {
        headers: env_size("STAGE5_BENCH_HEADERS", 3000),
        units: env_size("STAGE5_BENCH_UNITS", 1500),
        symbols_per_file: env_size("STAGE5_BENCH_SYMBOLS", 30),
        uses_per_symbol: 4,
        out_of_closure_percent: env_size("STAGE5_BENCH_OUT_PERCENT", 30) as u64,
        seed: 7,
    }
}

/// The per-symbol entry the pipeline used before the dense index: one
/// string-set walk per symbol over a shared expansion of its file.
fn reference_symbol_pass(analyzer: &LoadAnalyzer<'_>, groups: &[(FileId, Vec<&SymbolId>)]) -> usize {
    groups
        .par_iter()
        .map(|(file, ids)| {
            let expansion = analyzer.physical_expansion(*file);
            ids.par_iter()
                .map(|id| {
                    analyzer
                        .analyze_symbol_load_counts_with_expansion(id, &expansion)
                        .map_or(0, |metric| metric.semantic_load)
                })
                .sum::<usize>()
        })
        .sum()
}

#[test]
#[ignore = "scale bench; run explicitly for stage-5 pass timing"]
fn stage5_passes_at_scale_timing() {
    let shape = bench_shape();
    let (graph, model) = synthetic_load_model::build(&shape);
    let closures = graph.compute_closures();
    let memo = LoadMemo::with_closures_and_capacity(
        closures,
        LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP,
        LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP,
    );
    let analyzer = LoadAnalyzer::with_memo(&graph, &model, Arc::new(memo));
    println!(
        "shape {shape:?}: {} files, {} symbols, {} edges",
        graph.file_paths.len(),
        model.symbols.len(),
        model.edges.len()
    );

    let mut groups: BTreeMap<FileId, Vec<&SymbolId>> = BTreeMap::new();
    for (id, record) in &model.symbols {
        if let Some(file) = record.defining_file {
            groups.entry(file).or_default().push(id);
        }
    }
    let groups: Vec<(FileId, Vec<&SymbolId>)> = groups.into_iter().collect();

    // The per-symbol reference pass is quadratic: skip it at large sizes.
    let reference_total = if std::env::var("STAGE5_BENCH_SKIP_REFERENCE").is_ok() {
        None
    } else {
        let started = Instant::now();
        let total = reference_symbol_pass(&analyzer, &groups);
        println!("reference symbol pass | {:.2} s", started.elapsed().as_secs_f64());
        Some(total)
    };

    let started = Instant::now();
    let index = analyzer.dense_load_index();
    let index_seconds = started.elapsed().as_secs_f64();
    let dense_total: usize = groups
        .par_iter()
        .map(|(file, ids)| {
            let metrics = index.def_file_group_counts(*file, ids);
            metrics.iter().flatten().map(|metric| metric.semantic_load).sum::<usize>()
        })
        .sum();
    println!(
        "dense symbol pass | {:.2} s (index build {index_seconds:.2} s, mean semantic load {:.0})",
        started.elapsed().as_secs_f64(),
        dense_total as f64 / model.symbols.len() as f64
    );
    if let Some(reference_total) = reference_total {
        assert_eq!(dense_total, reference_total, "dense pass must count the same loads");
    }

    let files: Vec<FileId> = model.file_symbols.keys().copied().collect();
    if reference_total.is_some() {
        let started = Instant::now();
        files.par_iter().for_each(|file| {
            analyzer.analyze_file_load_compact_by_id(*file, CompactShape::default());
        });
        println!("reference file pass | {:.2} s", started.elapsed().as_secs_f64());
    }
    let started = Instant::now();
    let include_total: usize = files
        .par_iter()
        .map(|file| index.file_load_compact(*file, CompactShape::default()).include_load)
        .sum();
    println!(
        "dense file pass | {:.2} s (mean include load {:.0})",
        started.elapsed().as_secs_f64(),
        include_total as f64 / files.len() as f64
    );
}
