use std::path::PathBuf;
use anyhow::Result;
use ariadne_graph::{build_include_graph, ReducedSemanticModel};
use crate::commands::index::handle_index;

pub async fn run_ui(repo_opt: Option<PathBuf>, port: u16, no_open: bool, export_opt: Option<PathBuf>) -> Result<()> {
    let repo_path = repo_opt.unwrap_or_else(|| std::env::current_dir().unwrap());
    
    // Discover files, build compilation contexts, build include graph & closures & cycles.
    let (repository, include_graph, _closures, report) = build_include_graph(&repo_path)?;
    
    // Index source files & build reduced semantic model.
    let index_result = handle_index(&repo_path)?;
    let reduced_model = ReducedSemanticModel::reduce(&index_result);
    
    // Build CallGraph from index result & reduced model.
    let call_graph = ariadne_graph::CallGraph::build_from_index(&index_result, &reduced_model);
    
    // Build GraphExportPayload
    let cycles = include_graph.detect_cycles();
    
    let analyzer = ariadne_graph::LoadAnalyzer::new(&include_graph, &reduced_model);
    let mut file_collateral = std::collections::HashMap::new();
    for (file_id, path) in &include_graph.file_paths {
        if let Ok(metric) = analyzer.analyze_file_load(path) {
            let mut collat = std::collections::HashMap::new();
            for src in metric.collateral_sources {
                if src.symbol_count > 0 {
                    collat.insert(src.file_id.0.to_string(), src.symbol_count);
                }
            }
            if !collat.is_empty() {
                file_collateral.insert(file_id.0.to_string(), collat);
            }
        }
    }

    let payload = ariadne_ui::GraphExportPayload {
        repo_name: repository.root.file_name().unwrap_or_default().to_string_lossy().into_owned(),
        include_graph: ariadne_ui::build_include_elements(&include_graph, &_closures, &cycles),
        symbol_graph: ariadne_ui::build_symbol_elements(&reduced_model),
        call_graph: ariadne_ui::build_call_elements(&call_graph),
        stats: ariadne_ui::SummaryStats {
            total_files: include_graph.file_paths.len(),
            total_symbols: reduced_model.symbols.len(),
            total_calls: call_graph.edges.len(),
            cycle_count: report.cycle_count,
        },
        file_collateral,
    };
    
    if let Some(path) = export_opt {
        ariadne_ui::export_standalone_html(&payload, "", &path)?;
        println!("Report exported successfully to {}", path.display());
    } else {
        println!("AriadnesThread Visualizer running at http://localhost:{}", port);
        
        let state = std::sync::Arc::new(ariadne_ui::UiState {
            repo_root: repository.root.clone(),
            include_graph,
            reduced_model,
            call_graph,
            payload,
        });
        
        let app = ariadne_ui::build_router(state);
        
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        
        if !no_open {
            let _ = open::that(format!("http://localhost:{}", port));
        }
        
        axum::serve(listener, app).await?;
    }
    
    Ok(())
}
