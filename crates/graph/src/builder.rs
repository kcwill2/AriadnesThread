use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{IncludeClosure, IncludeGraph, IncludeReport, IncludeResolver};
use ariadne_build::BuildDiscoverer;
use ariadne_repo::{FileId, Repository};

pub fn build_include_graph(
    repo_path: &Path,
) -> Result<(
    Repository,
    IncludeGraph,
    HashMap<FileId, IncludeClosure>,
    IncludeReport,
)> {
    let repository = Repository::locate(repo_path)?;
    let repo_discovery = repository.discover()?;

    let build_discoverer =
        BuildDiscoverer::new(&repository.root).with_repo_discovery(&repo_discovery);
    let build_result = build_discoverer.discover()?;

    let mut graph = IncludeGraph::new();

    for file_record in &repo_discovery.files {
        graph.register_file(file_record.id, file_record.path.clone());
    }

    let mut scanned_files: HashSet<FileId> = HashSet::new();
    let mut scan_queue: VecDeque<(FileId, PathBuf, Option<String>)> = VecDeque::new();

    for tu in &build_result.translation_units {
        scan_queue.push_back((
            tu.file_id,
            tu.source_path.clone(),
            Some(tu.context_name.clone()),
        ));
    }

    while let Some((file_id, rel_path, ctx_name_opt)) = scan_queue.pop_front() {
        if scanned_files.contains(&file_id) {
            continue;
        }
        scanned_files.insert(file_id);

        let abs_path = repository.root.join(&rel_path);
        let content = match fs::read_to_string(&abs_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let target_ctx = ctx_name_opt
            .as_ref()
            .and_then(|name| build_result.contexts.iter().find(|c| &c.name == name))
            .or_else(|| build_result.contexts.first());

        let mut resolver =
            IncludeResolver::new(&repository.root).with_repo_discovery(&repo_discovery);
        if let Some(ctx) = target_ctx {
            resolver = resolver.with_context(ctx);
        }

        let edges = resolver.resolve_file_includes(file_id, &rel_path, &content);
        for edge in edges {
            if let Some(dest_id) = edge.destination {
                if !scanned_files.contains(&dest_id) {
                    if let Some(dest_path) = graph.file_paths.get(&dest_id).cloned() {
                        scan_queue.push_back((dest_id, dest_path, ctx_name_opt.clone()));
                    }
                }
            }
            graph.add_edge(edge);
        }
    }

    let closures = graph.compute_closures();
    let report = graph.generate_report();

    Ok((repository, graph, closures, report))
}
