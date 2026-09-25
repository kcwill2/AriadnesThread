use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use ariadne_repo::Repository;
use ariadne_build::BuildDiscoverer;
use ariadne_graph::{IncludeResolver, ReducedSemanticModel};
use ariadne_index::{CppSemanticAnalyzer, IndexResult, TuAnalysis};
use ariadne_report::ArtifactManager;

pub fn handle_index(repo_path: &PathBuf) -> Result<IndexResult> {
    let repository = Repository::locate(repo_path)?;
    let repo_discovery = repository.discover()?;

    let build_discoverer = BuildDiscoverer::new(&repository.root).with_repo_discovery(&repo_discovery);
    let build_result = build_discoverer.discover()?;

    let mut tu_analyses = Vec::new();

    for tu in &build_result.translation_units {
        let abs_path = repository.root.join(&tu.source_path);
        let content = fs::read_to_string(&abs_path)?;
        let target_ctx = build_result.contexts.iter().find(|c| c.name == tu.context_name)
            .or_else(|| build_result.contexts.first());

        let mut resolver = IncludeResolver::new(&repository.root).with_repo_discovery(&repo_discovery);
        if let Some(ctx) = target_ctx {
            resolver = resolver.with_context(ctx);
        }

        let edges = resolver.resolve_file_includes(tu.file_id.clone(), &tu.source_path, &content);
        
        let mut all_files = vec![(tu.file_id.clone(), tu.source_path.clone(), content)];
        
        let mut visited = HashSet::new();
        visited.insert(tu.file_id.clone());
        let mut queue = edges;
        
        while let Some(edge) = queue.pop() {
            if let Some(dest_id) = edge.destination {
                if visited.insert(dest_id.clone()) {
                    if let Some(dest_path) = repo_discovery.files.iter().find(|f| f.id == dest_id).map(|f| f.path.clone()) {
                        let header_abs = repository.root.join(&dest_path);
                        if let Ok(header_content) = fs::read_to_string(&header_abs) {
                            all_files.push((dest_id.clone(), dest_path.clone(), header_content.clone()));
                            let new_edges = resolver.resolve_file_includes(dest_id, &dest_path, &header_content);
                            queue.extend(new_edges);
                        }
                    }
                }
            }
        }

        let mut combined_symbols = Vec::new();
        let mut combined_declarations = Vec::new();
        let mut combined_edges = Vec::new();

        for (file_id, _, file_content) in all_files {
            let mut file_analysis = CppSemanticAnalyzer::analyze_source(file_id, &file_content, &tu.context_name);
            combined_symbols.append(&mut file_analysis.symbols);
            combined_declarations.append(&mut file_analysis.declarations);
            combined_edges.append(&mut file_analysis.edges);
        }
        
        tu_analyses.push(TuAnalysis {
            tu_file_id: tu.file_id.clone(),
            context_name: tu.context_name.clone(),
            symbols: combined_symbols,
            declarations: combined_declarations,
            edges: combined_edges,
            references: vec![],
            call_sites: vec![],
        });
    }

    let index_result = IndexResult::new(repository.root.clone(), tu_analyses);

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_index_artifacts(&index_result)?;

    println!("Indexed {} translation units", index_result.tu_analyses.len());
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
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    if let Ok(model) = artifacts.load_reduced_artifacts() {
        return Ok((repository, model));
    }

    // Need to build it
    let index_result = handle_index(repo_path)?;
    let model = ReducedSemanticModel::reduce(&index_result);
    artifacts.save_reduced_artifacts(&model)?;
    
    Ok((repository, model))
}
