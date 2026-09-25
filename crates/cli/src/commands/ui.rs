use std::path::PathBuf;
use anyhow::Result;
use ariadne_graph::{build_include_graph, ReducedSemanticModel};
use crate::commands::index::handle_index;

pub async fn run_ui(repo_opt: Option<PathBuf>, port: u16, no_open: bool, export_opt: Option<PathBuf>) -> Result<()> {
    let repo_path = repo_opt.unwrap_or_else(|| std::env::current_dir().unwrap());
    
    if let Some(path) = export_opt {
        // Synchronous run
        let (repository, include_graph, _closures, report) = build_include_graph(&repo_path)?;
        let index_result = handle_index(&repo_path)?;
        let reduced_model = ReducedSemanticModel::reduce(&index_result);
        
        crate::commands::progress::print_progress(0, 1, "Building call graph", "phase");
        let call_graph = ariadne_graph::CallGraph::build_from_index(&index_result, &reduced_model);
        crate::commands::progress::print_progress(1, 1, "Building call graph", "phase");
        
        let cycles = include_graph.detect_cycles();
        
        let analyzer = ariadne_graph::LoadAnalyzer::new(&include_graph, &reduced_model);
        let mut file_collateral = std::collections::HashMap::new();
        let total_files = include_graph.file_paths.len();
        for (i, (file_id, path)) in include_graph.file_paths.iter().enumerate() {
            crate::commands::progress::print_progress(i, total_files, "Analyzing collateral", "files");
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
        crate::commands::progress::print_progress(total_files, total_files, "Analyzing collateral", "files");

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
        
        ariadne_ui::export_standalone_html(&payload, "", &path)?;
        println!("Report exported successfully to {}", path.display());
    } else {
        // Async run
        let state = std::sync::Arc::new(ariadne_ui::UiState::new_loading(repo_path.clone()));
        
        let app = ariadne_ui::build_router(state.clone());
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        
        println!("AriadnesThread Visualizer loading at http://localhost:{}", port);
        if !no_open {
            let _ = open::that(format!("http://localhost:{}", port));
        }

        let state_clone = state.clone();
        let repo_path_clone = repo_path.clone();
        
        tokio::spawn(async move {
            let update_progress = |phase: &str, msg: &str, percent: usize| {
                let mut p = state_clone.progress.write().unwrap();
                p.phase = phase.to_string();
                p.message = msg.to_string();
                p.percent = percent;
            };

            update_progress("[1/4] Discovering include graph...", "Parsing includes", 10);
            let (repository, include_graph, _closures, report) = match build_include_graph(&repo_path_clone) {
                Ok(res) => res,
                Err(e) => {
                    let mut p = state_clone.progress.write().unwrap();
                    p.state = "error".to_string();
                    p.message = e.to_string();
                    return;
                }
            };
            
            update_progress("[2/4] Indexing translation units...", "Building index", 20);
            let index_result = match handle_index(&repo_path_clone) {
                Ok(res) => res,
                Err(e) => {
                    let mut p = state_clone.progress.write().unwrap();
                    p.state = "error".to_string();
                    p.message = e.to_string();
                    return;
                }
            };
            
            update_progress("[2/4] Indexing translation units...", "Reducing semantic model", 60);
            let reduced_model = ReducedSemanticModel::reduce(&index_result);
            
            update_progress("[3/4] Building call graph...", "Constructing call graph", 70);
            let call_graph = ariadne_graph::CallGraph::build_from_index(&index_result, &reduced_model);
            update_progress("[3/4] Building call graph...", "Done constructing call graph", 80);
            
            update_progress("[4/4] Analyzing collateral...", "Detecting cycles", 85);
            let cycles = include_graph.detect_cycles();
            
            let analyzer = ariadne_graph::LoadAnalyzer::new(&include_graph, &reduced_model);
            let mut file_collateral = std::collections::HashMap::new();
            let total_files = include_graph.file_paths.len();
            for (i, (file_id, path)) in include_graph.file_paths.iter().enumerate() {
                if i % 10 == 0 {
                    let percent = 85 + (10 * i / total_files.max(1));
                    update_progress("[4/4] Analyzing collateral...", &format!("File {}/{}", i, total_files), percent);
                }
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
            update_progress("[4/4] Analyzing collateral...", "Done analyzing", 95);

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
            
            {
                *state_clone.include_graph.write().unwrap() = Some(include_graph);
                *state_clone.reduced_model.write().unwrap() = Some(reduced_model);
                *state_clone.call_graph.write().unwrap() = Some(call_graph);
                *state_clone.payload.write().unwrap() = Some(payload);
                let mut p = state_clone.progress.write().unwrap();
                p.state = "ready".to_string();
                p.phase = "Complete".to_string();
                p.message = "Ready".to_string();
                p.percent = 100;
            }
        });

        axum::serve(listener, app).await?;
    }
    
    Ok(())
}
