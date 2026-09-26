use std::path::PathBuf;
use anyhow::Result;
use ariadne_repo::Repository;
use ariadne_index::IndexResult;
use ariadne_graph::ReducedSemanticModel;
use ariadne_pipeline::{Pipeline, ProgressEvent};

pub fn handle_index(repo_path: &PathBuf) -> Result<IndexResult> {
    let mut pipeline = Pipeline::new(repo_path, None)?.with_progress(|event| {
        if let ProgressEvent::TuIndex { done, total } = event {
            crate::commands::progress::print_progress(*done, *total, "Indexing translation units", "TUs")
        }
    });

    let index_result = pipeline.get_or_build_index()?;

    println!("Indexed {} translation units", index_result.tu_analyses.len());
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

    Ok(index_result)
}

pub fn get_or_build_reduced_model(repo_path: &PathBuf) -> Result<(Repository, ReducedSemanticModel)> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let model = pipeline.get_or_build_reduced()?;
    Ok((pipeline.repo.clone(), model))
}
