use std::path::PathBuf;
use anyhow::{bail, Result};
use ariadne_repo::FileId;
use ariadne_graph::build_include_graph;
use ariadne_report::ArtifactManager;

pub fn handle_includes(repo_path: &PathBuf) -> Result<()> {
    let (repository, _graph, _closures, report) = build_include_graph(repo_path)?;

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_include_artifacts(&_graph, &_closures, &report)?;

    println!("Include Graph Summary");
    println!("  Repository: {}", repository.root.display());
    println!("  Total Edges: {}", report.total_edges);
    println!("  Unique Included Headers: {}", report.unique_headers);
    println!("  Local Headers: {}", report.local_headers_count);
    println!("  System Headers: {}", report.system_headers_count);
    println!("  External Headers: {}", report.external_headers_count);
    println!("  Unresolved Includes: {}", report.unresolved_includes_count);
    println!("  Cycles Detected: {}", report.cycle_count);
    println!();
    println!("Top Headers by Transitive Include Closure:");
    for (idx, (_id, path, closure_size)) in report.top_headers_by_closure.iter().enumerate() {
        println!("  {}. {} (closure size: {})", idx + 1, path.display(), closure_size);
    }

    Ok(())
}

pub fn handle_includes_explain(repo_path: &PathBuf, file_path: &PathBuf) -> Result<()> {
    let (_repository, graph, _closures, _report) = build_include_graph(repo_path)?;

    let target_id = match graph.find_file_id(file_path) {
        Some(id) => id,
        None => {
            let rel_str = file_path.to_string_lossy().replace('\\', "/");
            FileId::from_relative_path(&rel_str)
        }
    };

    if !graph.file_paths.contains_key(&target_id) {
        bail!("File {:?} not found in repository include graph", file_path);
    }

    let tree_str = graph.format_explain(target_id);
    print!("{}", tree_str);

    Ok(())
}
