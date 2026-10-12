use anyhow::Result;
use ariadne_graph::ReducedSemanticModel;
use ariadne_index::IndexResult;
use ariadne_lang::LanguageRegistry;
use ariadne_pipeline::{Pipeline, ProgressEvent};
use ariadne_repo::{Repository, SourceLanguage};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The `index` command's stdout contract (stable across the registry and
/// `--lang` filtered paths).
fn report_index_result(pipeline: &Pipeline, index_result: &IndexResult) {
    println!(
        "Indexed {} translation units",
        index_result.tu_analyses.len()
    );
    println!();
    println!("{}", pipeline.stats.format_summary());
    println!();
    println!("Symbols:");
    println!("  {}", index_result.total_symbols);
    println!();
    println!("Semantic edges:");
    println!("  {}", index_result.total_edges);
    println!();
    println!("Complete:");
    println!("  {}", index_result.complete_edges);
    println!();
    println!("Incomplete:");
    println!("  {}", index_result.incomplete_edges);
}

fn index_progress_cb(event: &ProgressEvent) {
    // V3 rider (open Q8 default): TuStarted drives the same terminal bar as
    // the terminal TuIndex; the path itself is a UI affordance, so `tool
    // index` behavior stays neutral.
    if let ProgressEvent::TuIndex { done, total } | ProgressEvent::TuStarted { done, total, .. } =
        event
    {
        crate::commands::progress::print_progress(*done, *total, "Indexing translation units", "TUs")
    }
}

/// Registry-aware index handler (plan M18.3): per unit, the plugin's
/// resolver drives the dependency walk (search-path-aware, catalog-driven —
/// `scheduler::compute_tu_closure_plugin` semantics) and the plugin's
/// analyzer produces the merged `TuAnalysis`; the merged analysis records
/// the unit's language. A language no registered plugin claims is reported
/// as "unhandled language" (§3.3 fact 3), never silently parsed as C++.
///
/// C++ output is byte-identical to the pre-M18 hardcoded walk (closure
/// order, merge order, cache fingerprints), locked by the m00 goldens and
/// the vendored oracles in `m18_cpp_plugin_parity_test`.
pub fn handle_index_with_registry(
    repo_path: &Path,
    registry: Arc<LanguageRegistry>,
) -> Result<IndexResult> {
    handle_index_with_registry_filtered(repo_path, registry, None)
}

/// `--lang` restriction of [`handle_index_with_registry`] (plan M18.3):
/// absent = per-file detection (today's behaviour, byte-identical);
/// present = only units whose plugin the registry detects as that language
/// are indexed, and an unregistered language is a clear error up front.
/// A restricted run produces a filtered view and never overwrites the
/// authoritative full-stage artifacts.
pub fn handle_index_with_registry_filtered(
    repo_path: &Path,
    registry: Arc<LanguageRegistry>,
    language: Option<SourceLanguage>,
) -> Result<IndexResult> {
    if let Some(language) = language.clone() {
        crate::registry::ensure_language_registered(&registry, language)?;
    }

    let mut pipeline = Pipeline::with_registry(repo_path, None, registry)?
        .language_filter(language)
        .with_progress(index_progress_cb);

    let index_result = pipeline.get_or_build_index()?;
    report_index_result(&pipeline, &index_result);
    Ok(index_result)
}

pub fn build_source_index(repo_path: &PathBuf) -> Result<IndexResult> {
    build_source_index_filtered(repo_path, None)
}

/// `tool index --lang <language>` (plan M18.3). The registry is enumerated
/// in one place: [`crate::registry::default_registry`].
pub fn build_source_index_filtered(
    repo_path: &PathBuf,
    language: Option<SourceLanguage>,
) -> Result<IndexResult> {
    handle_index_with_registry_filtered(
        repo_path.as_path(),
        Arc::new(crate::registry::default_registry()),
        language,
    )
}

pub fn get_or_build_reduced_model(
    repo_path: &PathBuf,
) -> Result<(Repository, ReducedSemanticModel)> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let model = pipeline.get_or_build_reduced()?;
    Ok((pipeline.repo.clone(), model))
}
