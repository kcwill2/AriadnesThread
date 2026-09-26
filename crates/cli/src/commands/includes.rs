use std::path::PathBuf;
use anyhow::{bail, Result};
use ariadne_repo::FileId;
use ariadne_pipeline::Pipeline;

pub fn handle_includes(repo_path: &PathBuf) -> Result<()> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let (graph, _closures) = pipeline.get_or_build_include_graph()?;
    let report = graph.generate_report();

    println!("Include Graph Summary");
    println!("  Repository: {}", pipeline.repo.root.display());
    println!("  Total Edges: {}", report.total_edges);
    println!("  Unique Included Headers: {}", report.unique_headers);
    println!("  Local Headers: {}", report.local_headers_count);
    println!("  System Headers: {}", report.system_headers_count);
    println!("  External Headers: {}", report.external_headers_count);
    println!("  Unresolved Includes: {}", report.unresolved_includes_count);
    println!("  Cycles Detected: {}", report.cycle_count);
    println!();
    println!("{}", pipeline.stats.format_summary());
    println!();
    println!("Top Headers by Transitive Include Closure:");
    for (idx, (_id, path, closure_size)) in report.top_headers_by_closure.iter().enumerate() {
        println!("  {}. {} (closure size: {})", idx + 1, path.display(), closure_size);
    }

    Ok(())
}

pub fn handle_includes_explain(repo_path: &PathBuf, file_path: &PathBuf) -> Result<()> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let (graph, _closures) = pipeline.get_or_build_include_graph()?;

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
