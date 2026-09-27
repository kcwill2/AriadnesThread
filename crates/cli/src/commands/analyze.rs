use anyhow::{bail, Result};
use ariadne_build::BuildDiscoverer;
use ariadne_graph::LoadAnalyzer;
use ariadne_pipeline::Pipeline;
use ariadne_repo::Repository;
use ariadne_report::{ArtifactManager, LoadReport};
use std::path::{Path, PathBuf};

pub fn run_incremental_analysis(
    repo_path: &PathBuf,
) -> Result<ariadne_scheduler::IncrementalAnalysisResult> {
    let repo = Repository::locate(repo_path)?;
    let repo_discovery = repo.discover()?;
    let build_discovery = BuildDiscoverer::new(&repo.root)
        .with_repo_discovery(&repo_discovery)
        .discover()?;
    let mut analyzer =
        ariadne_scheduler::IncrementalAnalyzer::new(&repo, &repo_discovery, &build_discovery);
    let result = analyzer.analyze()?;
    result.save_artifacts(&repo)?;

    println!("Analyzing...\n");
    println!(
        "Reused:\n    {} translation units\n",
        result.stats.reused_tus
    );
    println!(
        "Reanalyzed:\n    {} translation units\n",
        result.stats.reanalyzed_tus
    );
    println!("Cache hit rate:\n    {:.1}%", result.stats.hit_rate);

    Ok(result)
}

pub fn run_production_scale_analysis(
    repo_path: &PathBuf,
    workers: Option<usize>,
) -> Result<ariadne_scheduler::ProductionAnalysisResult> {
    let num_workers = workers.unwrap_or(4);
    let mut pipeline = Pipeline::new(repo_path, Some(num_workers))?;
    let index_result = pipeline.get_or_build_index()?;
    let reduced_model = pipeline.get_or_build_reduced()?;

    // When the index stage actually ran through the worker pool, report its
    // real measured stats. When it was loaded/reused from artifacts, no work
    // was measured: report honest zeros rather than invented numbers.
    let stats = match pipeline.production_stats.clone() {
        Some(stats) => stats,
        None => ariadne_scheduler::ProductionAnalysisStats {
            total_tus: index_result.tu_analyses.len(),
            num_workers,
            progress_completed: 0,
            progress_total: index_result.tu_analyses.len(),
            total_semantic_edges: reduced_model.edges.len(),
            peak_memory_bytes: 0,
            elapsed: std::time::Duration::ZERO,
            reused_tus: pipeline.stats.reused_tus,
            reanalyzed_tus: pipeline.stats.reanalyzed_tus,
        },
    };

    println!("{}", stats.format_m13_output());
    println!();
    println!("{}", pipeline.stats.format_summary());

    Ok(ariadne_scheduler::ProductionAnalysisResult {
        reduced_model,
        index_result,
        stats,
    })
}

pub fn analyze_dependency_load(
    repo_path: &PathBuf,
    symbol: Option<&str>,
    file: Option<&Path>,
) -> Result<()> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let (graph, closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;
    let repository = pipeline.repo.clone();

    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);
    let mut load_report = LoadReport {
        repository_root: repository.root.clone(),
        symbol_metrics: Vec::new(),
        file_metrics: Vec::new(),
    };

    // Targeted queries produce a single-metric partial report. Persisting it
    // under the full-stage fingerprint would poison the load stage that
    // Pipeline::ensure_load loads as authoritative, silently degrading the
    // UI collateral heatmap/hotspots. Only the full-report branch may save.
    let targeted = symbol.is_some() || file.is_some();

    if let Some(sym_name) = symbol {
        let metric = analyzer.analyze_symbol_load(sym_name)?;
        println!("{}", metric.format_summary());
        load_report.symbol_metrics.push(metric);
    } else if let Some(file_path) = file {
        let metric = analyzer.analyze_file_load(file_path)?;
        println!("{}", metric.format_summary());
        load_report.file_metrics.push(metric);
    } else {
        println!("Analyzing all symbols in model...");
        for sym_record in model.symbols.values() {
            if let Ok(metric) = analyzer.analyze_symbol_load(&sym_record.name) {
                load_report.symbol_metrics.push(metric);
            }
        }
        println!("Analyzed {} symbols.", load_report.symbol_metrics.len());

        println!("Analyzing all files in model...");
        for (file_id, _) in &model.file_symbols {
            if let Some(file_path) = graph.file_paths.get(file_id) {
                if let Ok(metric) = analyzer.analyze_file_load(file_path) {
                    load_report.file_metrics.push(metric);
                }
            }
        }
        println!("Analyzed {} files.", load_report.file_metrics.len());
    }

    if !targeted {
        let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
        artifacts.save_load_artifacts(&load_report, &pipeline.fps.load)?;
    }

    println!();
    println!("{}", pipeline.stats.format_summary());

    Ok(())
}

pub fn explain_dependency_load(
    repo_path: &PathBuf,
    symbol: Option<&str>,
    file: Option<&Path>,
    source: Option<&str>,
) -> Result<()> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let (graph, closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;

    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);

    if let Some(sym_name) = symbol {
        let explanation = analyzer.explain_symbol_load(sym_name, source)?;
        println!("{}", explanation.format_explanation());
    } else if let Some(file_path) = file {
        let explanation = analyzer.explain_file_load(file_path, source)?;
        println!("{}", explanation.format_explanation());
    } else {
        bail!("--symbol or --file is required to explain load");
    }

    Ok(())
}
