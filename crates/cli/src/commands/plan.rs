use std::path::PathBuf;
use anyhow::Result;
use ariadne_repo::Repository;
use ariadne_build::BuildDiscoverer;
use ariadne_graph::build_include_graph;
use ariadne_report::ArtifactManager;
use crate::commands::index::get_or_build_reduced_model;

pub fn handle_plan(repo_path: &PathBuf) -> Result<Vec<ariadne_cluster::plan::OptimizationPlan>> {
    let repository = Repository::locate(repo_path)?;
    let discovery = repository.discover()?;

    let build_discoverer = BuildDiscoverer::new(&repository.root).with_repo_discovery(&discovery);
    let build_result = build_discoverer.discover()?;
    let tu_file_ids: Vec<_> = build_result.translation_units.iter().map(|tu| tu.file_id).collect();

    let (_repository, graph, _closures, _report) = build_include_graph(repo_path)?;
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let planner = ariadne_cluster::planner::OptimizationPlanner::new(&graph, &model, &discovery.files, &tu_file_ids);
    let plans = planner.plan();

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_plan_artifacts(&repository.root, &plans)?;

    println!("Optimization plans: {}", plans.len());
    println!();
    for plan in &plans {
        println!("{}", plan.format_detail());
    }

    Ok(plans)
}

pub fn handle_plan_show(repo_path: &PathBuf, id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plans = match artifacts.load_plan_artifacts() {
        Ok(p) => p,
        Err(_) => handle_plan(repo_path)?,
    };

    if let Some(plan) = plans.iter().find(|p| p.id.0 == id) {
        println!("{}", plan.format_detail());
    } else {
        println!("Plan {} not found.", id);
    }

    Ok(())
}
