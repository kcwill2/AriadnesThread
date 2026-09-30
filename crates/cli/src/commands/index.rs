use anyhow::Result;
use ariadne_graph::ReducedSemanticModel;
use ariadne_index::IndexResult;
use ariadne_lang::LanguageRegistry;
use ariadne_pipeline::{Pipeline, ProgressEvent};
use ariadne_repo::{Repository, SourceLanguage};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The `index` command's stdout contract — identical text for the registry
/// path and the transitional legacy path (byte-identical CLI output).
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
    if let ProgressEvent::TuIndex { done, total } = event {
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
/// C++ output is byte-identical to the transitional registry-less path
/// (closure order, merge order, cache fingerprints), locked by
/// `m18_parity_interim_test`.
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
    if let Some(language) = language {
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

/// Transitional (until the M18.11 final parity suite): the pre-M18.3
/// registry-less index path, byte-for-byte the old `build_source_index`
/// body. `m18_parity_interim_test` holds it against
/// [`handle_index_with_registry`] on both fixtures; delete it when the M18.11
/// suite (and the `#[deprecated]` shims) replace this seam.
#[deprecated(
    note = "pre-M18.3 registry-less path; retained only for interim old-vs-new parity (removed in M18.11)"
)]
pub fn build_source_index_legacy_for_parity(repo_path: &PathBuf) -> Result<IndexResult> {
    let mut pipeline = Pipeline::new(repo_path, None)?.with_progress(index_progress_cb);

    let index_result = pipeline.get_or_build_index()?;
    report_index_result(&pipeline, &index_result);
    Ok(index_result)
}

pub fn get_or_build_reduced_model(
    repo_path: &PathBuf,
) -> Result<(Repository, ReducedSemanticModel)> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let model = pipeline.get_or_build_reduced()?;
    Ok((pipeline.repo.clone(), model))
}
