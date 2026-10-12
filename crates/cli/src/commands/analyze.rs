use anyhow::{bail, Result};
use ariadne_graph::LoadAnalyzer;
use ariadne_lang::{DependencyVocabulary, LanguageRegistry};
use ariadne_pipeline::Pipeline;
use ariadne_repo::SourceLanguage;
use ariadne_report::{ArtifactManager, LoadReport};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn run_incremental_analysis(
    repo_path: &PathBuf,
) -> Result<ariadne_scheduler::IncrementalAnalysisResult> {
    run_incremental_analysis_filtered(repo_path, None)
}

/// Registry-wired incremental analysis (plan M18.3, M19.3, A6/AR-23): the
/// construction sequence (locate, repo/build discovery, registry-shaper
/// selection, the `--lang` unit restriction, the registration and
/// unhandled-language checks, and the artifact-authority rule) is owned by
/// `Pipeline::for_incremental` - the single implementation, no cli mirror.
/// The CLI pairs the analyzer backend from the restricted build through
/// `registry::incremental_backend` (distinct `tu.language` dispatch: one
/// language -> its plugin, unclaimed language -> "unhandled language"
/// (sec 3.3 fact 3), mixed -> the M21 language-scoped-cache error, zero
/// units -> sole-plugin fallback).
pub fn run_incremental_analysis_filtered(
    repo_path: &PathBuf,
    language: Option<SourceLanguage>,
) -> Result<ariadne_scheduler::IncrementalAnalysisResult> {
    let pipeline = Pipeline::for_incremental(repo_path, language)?;
    let authoritative = pipeline
        .incremental_view
        .as_ref()
        .expect("for_incremental always records its unit-restriction/authority state")
        .artifacts_are_authoritative();

    let registry = crate::registry::default_registry();
    // AR-25 (A7 rider): the seam's typed error propagates through `?` with
    // its anyhow source intact - no `.map_err(anyhow::Error::msg)` re-wrap
    // (that flattened typed errors to message bytes and erased the kind).
    let plugin = crate::registry::incremental_backend(&registry, &pipeline.build)?;
    let mut analyzer = ariadne_scheduler::IncrementalAnalyzer::new(
        &pipeline.repo,
        &pipeline.discovery,
        &pipeline.build,
        plugin,
    );
    let result = analyzer.analyze()?;
    if authoritative {
        result.save_artifacts(&pipeline.repo)?;
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
    if let Some(language) = language.clone() {
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

/// D26 (plan:425 + plan:509): pick the run's dependency vocabulary and
/// heuristic caveats through registry lookups over the discovered record
/// languages (rule R3 — no `match` on a language). Single non-default
/// language whose plugin is registered -> that plugin's vocabulary;
/// C++-only, mixed, or unclaimed runs keep today's default wording
/// (`None` -> the m6-locked call path). Caveats collect from EVERY
/// analyzed language carrying one — a mixed run still surfaces the
/// heuristic label — naming the language by Debug-formatting the
/// `LanguageId` value (data, not a literal). Pure so the selection rule is
/// testable without a repository.
fn run_vocabulary(
    languages: &[SourceLanguage],
    registry: &LanguageRegistry,
) -> (Option<DependencyVocabulary>, Vec<String>) {
    let mut distinct: Vec<SourceLanguage> = Vec::new();
    for language in languages {
        if !distinct.contains(language) {
            distinct.push(language.clone());
        }
    }
    let vocabulary = match distinct.as_slice() {
        [language] if *language != SourceLanguage::default() => registry
            .plugin_for_id(ariadne_lang::LanguageId::from(language.clone()))
            .map(|plugin| plugin.modeling().dependency_vocabulary),
        _ => None,
    };
    let mut caveats: Vec<String> = Vec::new();
    for language in &distinct {
        let Some(plugin) = registry.plugin_for_id(ariadne_lang::LanguageId::from(language.clone())) else {
            continue;
        };
        if let Some(caveat) = plugin.modeling().analysis_caveat {
            caveats.push(format!("Analysis caveat [{language:?}]: {caveat}"));
        }
    }
    (vocabulary, caveats)
}

/// Discovery record languages of the run, deduplicated in first-seen order.
fn discovered_languages(pipeline: &Pipeline) -> Vec<SourceLanguage> {
    let mut languages: Vec<SourceLanguage> = Vec::new();
    for record in &pipeline.discovery.files {
        if !languages.contains(&record.language) {
            languages.push(record.language.clone());
        }
    }
    languages
}

/// Emits the caveat lines (nothing at all when no analyzed language carries
/// one — the C++ stdout bytes stay untouched).
pub(crate) fn print_caveats(caveats: &[String]) {
    if caveats.is_empty() {
        return;
    }
    for caveat in caveats {
        println!("{caveat}");
    }
    println!();
}

pub(crate) fn discovered_run_vocabulary(
    pipeline: &Pipeline,
) -> (Option<DependencyVocabulary>, Vec<String>) {
    let registry = crate::registry::default_registry();
    run_vocabulary(&discovered_languages(pipeline), &registry)
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
    let (vocabulary, caveats) = discovered_run_vocabulary(&pipeline);

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
        print_caveats(&caveats);
        match &vocabulary {
            Some(vocabulary) => println!("{}", metric.format_summary_with(vocabulary)),
            None => println!("{}", metric.format_summary()),
        }
        load_report.symbol_metrics.push(metric);
    } else if let Some(file_path) = file {
        let metric = analyzer.analyze_file_load(file_path)?;
        print_caveats(&caveats);
        match &vocabulary {
            Some(vocabulary) => println!("{}", metric.format_summary_with(vocabulary)),
            None => println!("{}", metric.format_summary()),
        }
        load_report.file_metrics.push(metric);
    } else {
        print_caveats(&caveats);
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
        // M22 T-D: the same discovery-record languages `Pipeline::ensure_load`
        // feeds (`pipeline/src/lib.rs` load stage) — a Scala run's load
        // manifest carries `languages`, an all-C++ run stays empty-skipped
        // and byte-identical (D25, m00 witness).
        let stage_languages: Vec<SourceLanguage> =
            pipeline.discovery.files.iter().map(|f| f.language.clone()).collect();
        artifacts.save_load_artifacts_with_languages(
            &load_report,
            &pipeline.fps.load,
            &stage_languages,
        )?;
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
    let (vocabulary, caveats) = discovered_run_vocabulary(&pipeline);

    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);

    if let Some(sym_name) = symbol {
        let explanation = analyzer.explain_symbol_load(sym_name, source)?;
        print_caveats(&caveats);
        match &vocabulary {
            Some(vocabulary) => println!("{}", explanation.format_explanation_with(vocabulary)),
            None => println!("{}", explanation.format_explanation()),
        }
    } else     if let Some(file_path) = file {
        let explanation = analyzer.explain_file_load(file_path, source)?;
        print_caveats(&caveats);
        match &vocabulary {
            Some(vocabulary) => println!("{}", explanation.format_explanation_with(vocabulary)),
            None => println!("{}", explanation.format_explanation()),
        }
    } else {
        bail!("--symbol or --file is required to explain load");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scala_only_run_selects_import_vocabulary_and_scala_caveat() {
        let registry = crate::registry::default_registry();
        let (vocabulary, caveats) = run_vocabulary(&[SourceLanguage::Scala], &registry);
        let vocabulary =
            vocabulary.expect("single non-default language selects its plugin vocabulary");
        assert_eq!(vocabulary.directive_label, "Import");
        assert_eq!(vocabulary.directive_passive_verb, "imported");
        assert_eq!(vocabulary.aggregation_noun, "package aggregation");
        assert_eq!(caveats.len(), 1, "the Scala caveat must be reported: {caveats:?}");
        assert!(caveats[0].contains("Scala"), "caveat names the language by data");
        assert!(caveats[0].contains("heuristic"), "caveat is the heuristic disclaimer");
    }

    #[test]
    fn cpp_only_run_keeps_todays_default_and_no_caveat() {
        let registry = crate::registry::default_registry();
        let (vocabulary, caveats) =
            run_vocabulary(&[SourceLanguage::Cpp, SourceLanguage::Cpp], &registry);
        assert!(
            vocabulary.is_none(),
            "a C++-only run must keep today's default calls (m6 byte locks)"
        );
        assert!(caveats.is_empty());
    }

    #[test]
    fn mixed_run_keeps_default_wording_but_reports_the_scala_caveat() {
        let registry = crate::registry::default_registry();
        let (vocabulary, caveats) =
            run_vocabulary(&[SourceLanguage::Cpp, SourceLanguage::Scala], &registry);
        assert!(vocabulary.is_none(), "mixed runs keep today's default wording");
        assert_eq!(caveats.len(), 1);
        assert!(caveats[0].contains("Scala"));
    }

    #[test]
    fn empty_and_unclaimed_languages_use_the_default_with_no_caveats() {
        let registry = crate::registry::default_registry();
        assert!(run_vocabulary(&[], &registry).0.is_none());
        let (vocabulary, caveats) = run_vocabulary(&[SourceLanguage::Other], &registry);
        assert!(vocabulary.is_none(), "an unclaimed language has no vocabulary to select");
        assert!(caveats.is_empty());
    }
}
