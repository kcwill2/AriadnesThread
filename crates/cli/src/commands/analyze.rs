use std::path::{Path, PathBuf};
use anyhow::{bail, Result};
use ariadne_repo::Repository;
use ariadne_build::BuildDiscoverer;
use ariadne_report::{ArtifactManager, LoadReport};
use ariadne_graph::LoadAnalyzer;
use ariadne_graph::build_include_graph;
use crate::commands::index::get_or_build_reduced_model;

pub fn handle_analyze(repo_path: &PathBuf) -> Result<ariadne_scheduler::IncrementalAnalysisResult> {
    let repo = Repository::locate(repo_path)?;
    let repo_discovery = repo.discover()?;
    let build_discovery = BuildDiscoverer::new(&repo.root).with_repo_discovery(&repo_discovery).discover()?;
    let mut analyzer = ariadne_scheduler::IncrementalAnalyzer::new(&repo, &repo_discovery, &build_discovery);
    let result = analyzer.analyze()?;
    result.save_artifacts(&repo)?;

    println!("Analyzing...\n");
    println!("Reused:\n    {} translation units\n", result.stats.reused_tus);
    println!("Reanalyzed:\n    {} translation units\n", result.stats.reanalyzed_tus);
    println!("Cache hit rate:\n    {:.1}%", result.stats.hit_rate);

    Ok(result)
}

pub fn handle_production_analyze(repo_path: &PathBuf, workers: Option<usize>) -> Result<ariadne_scheduler::ProductionAnalysisResult> {
    let repo = Repository::locate(repo_path)?;
    let repo_discovery = repo.discover()?;
    let build_discovery = BuildDiscoverer::new(&repo.root).with_repo_discovery(&repo_discovery).discover()?;

    let num_workers = workers.unwrap_or(4);
    let analyzer = ariadne_scheduler::ProductionAnalyzer::new(&repo, &repo_discovery, &build_discovery)
        .with_workers(num_workers);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let result = rt.block_on(analyzer.analyze())?;

    let manager = ArtifactManager::prepare_for_repo(&repo)?;
    manager.save_reduced_artifacts(&result.reduced_model)?;
    manager.save_index_artifacts(&result.index_result)?;

    println!("{}", result.stats.format_m13_output());

    Ok(result)
}

pub fn handle_analyze_load(repo_path: &PathBuf, symbol: Option<&str>, file: Option<&Path>) -> Result<()> {
    let (repository, graph, closures, _report) = build_include_graph(repo_path)?;
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);
    let mut load_report = LoadReport {
        repository_root: repository.root.clone(),
        symbol_metrics: Vec::new(),
        file_metrics: Vec::new(),
    };

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

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_load_artifacts(&load_report)?;

    Ok(())
}

pub fn handle_explain_load(repo_path: &PathBuf, symbol: Option<&str>, file: Option<&Path>, source: Option<&str>) -> Result<()> {
    let (_repository, graph, closures, _report) = build_include_graph(repo_path)?;
    let (_, model) = get_or_build_reduced_model(repo_path)?;

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
