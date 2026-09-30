use anyhow::{anyhow, bail, Result};
use ariadne_build::BuildDiscoverer;
use ariadne_graph::LoadAnalyzer;
use ariadne_lang::LanguageRegistry;
use ariadne_pipeline::Pipeline;
use ariadne_repo::{Repository, SourceLanguage};
use ariadne_report::{ArtifactManager, LoadReport};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn run_incremental_analysis(
    repo_path: &PathBuf,
) -> Result<ariadne_scheduler::IncrementalAnalysisResult> {
    run_incremental_analysis_filtered(repo_path, None)
}

/// Registry-wired incremental analysis (plan M18.3): the unit shaper and the
/// `IncrementalAnalyzer`'s analyzer/resolver injection come from the single
/// registered plugin (multi-plugin registries dispatch per unit only after
/// the M19 per-language split); units no plugin claims are reported as
/// "unhandled language" (§3.3 fact 3), and `--lang` restricts the unit set.
pub fn run_incremental_analysis_filtered(
    repo_path: &PathBuf,
    language: Option<SourceLanguage>,
) -> Result<ariadne_scheduler::IncrementalAnalysisResult> {
    let registry = crate::registry::default_registry();
    if let Some(language) = language {
        crate::registry::ensure_language_registered(&registry, language)?;
    }

    let repo = Repository::locate(repo_path)?;
    let repo_discovery = repo.discover()?;
    let mut build_discovery = BuildDiscoverer::new(&repo.root)
        .with_repo_discovery(&repo_discovery)
        .with_unit_shaper(registry.unit_shaper_for())
        .discover()?;
    let discovered_units = build_discovery.translation_units.len();
    restrict_and_validate_units(&registry, language, &mut build_discovery)?;
    let authoritative = artifacts_are_authoritative(
        language,
        build_discovery.translation_units.len(),
        discovered_units,
    );

    let plugin = registry
        .sole_plugin()
        .ok_or_else(|| anyhow!("incremental indexing needs exactly one registered language plugin (multi-language lands in M19)"))?;
    let mut analyzer =
        ariadne_scheduler::IncrementalAnalyzer::new(&repo, &repo_discovery, &build_discovery)
            .with_analyzer(plugin.analyzer())
            .with_resolver(plugin.resolver())
            .with_plugin_identity(plugin.id(), plugin.analysis_fingerprint());
    let result = analyzer.analyze()?;
    if authoritative {
        result.save_artifacts(&repo)?;
    } else {
        println!(
            "Restricted run: skipping authoritative index/reduced artifact persistence \
             (a filtered view is never the full-repo artifact)."
        );
    }

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

/// Apply the `--lang` restriction (when given) and then require every
/// retained unit to be claimed by some registered plugin (§3.3 fact 3).
fn restrict_and_validate_units(
    registry: &LanguageRegistry,
    language: Option<SourceLanguage>,
    build: &mut ariadne_build::BuildDiscoveryResult,
) -> Result<()> {
    if let Some(language) = language {
        build
            .translation_units
            .retain(|tu| registry.detect_language(&tu.source_path) == Some(language));
    }
    for tu in &build.translation_units {
        if registry.detect_language(&tu.source_path).is_none() {
            bail!(
                "unhandled language for {} (no registered plugin claims it)",
                tu.source_path.display()
            );
        }
    }
    Ok(())
}

/// A run whose unit set was narrowed below the discovered set — an active
/// `--lang` filter or any other restriction — produces a VIEW, never the
/// authoritative full-stage artifact. Persisting a filtered result under the
/// full index/reduced fingerprint would poison every later full run, so only
/// the unfiltered run persists (mirrors `Pipeline`'s
/// `language_filter.is_none()` index guard). Today `--lang` is the only
/// restriction and it always runs before saving, so the false branch is
/// unreachable — but the guard must exist before M19's per-language split can
/// actually narrow the unit set.
fn artifacts_are_authoritative(
    language_filter: Option<SourceLanguage>,
    retained_units: usize,
    discovered_units: usize,
) -> bool {
    language_filter.is_none() && retained_units >= discovered_units
}

pub fn run_production_scale_analysis(
    repo_path: &PathBuf,
    workers: Option<usize>,
) -> Result<ariadne_scheduler::ProductionAnalysisResult> {
    run_production_scale_analysis_filtered(repo_path, workers, None)
}

/// Registry-wired production-scale analysis (plan M18.3): the worker pool
/// runs through `PluginWorkerBackend` semantics over the default registry
/// (`SharedPluginWorkerBackend`), and `--lang` restricts the indexed units.
pub fn run_production_scale_analysis_filtered(
    repo_path: &PathBuf,
    workers: Option<usize>,
    language: Option<SourceLanguage>,
) -> Result<ariadne_scheduler::ProductionAnalysisResult> {
    let num_workers = workers.unwrap_or(4);
    let registry = Arc::new(crate::registry::default_registry());
    if let Some(language) = language {
        crate::registry::ensure_language_registered(&registry, language)?;
    }
    let mut pipeline = Pipeline::with_registry(repo_path, Some(num_workers), registry)?
        .language_filter(language);
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
    } else     if let Some(file_path) = file {
        let explanation = analyzer.explain_file_load(file_path, source)?;
        println!("{}", explanation.format_explanation());
    } else {
        bail!("--symbol or --file is required to explain load");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrestricted_full_unit_set_is_authoritative() {
        assert!(artifacts_are_authoritative(None, 8, 8));
    }

    #[test]
    fn active_language_filter_is_never_authoritative() {
        assert!(!artifacts_are_authoritative(
            Some(SourceLanguage::Cpp),
            8,
            8
        ));
    }

    #[test]
    fn unit_set_narrowed_below_discovery_is_never_authoritative() {
        assert!(!artifacts_are_authoritative(
            None, 5, 8
        ));
    }
}
