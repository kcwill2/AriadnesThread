use std::path::PathBuf;
use anyhow::Result;
use ariadne_repo::Repository;
use ariadne_report::ArtifactManager;
use ariadne_pipeline::Pipeline;

pub fn handle_plan(repo_path: &PathBuf) -> Result<Vec<ariadne_cluster::plan::OptimizationPlan>> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let (graph, _closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;
    let tu_file_ids: Vec<_> = pipeline.build.translation_units.iter().map(|tu| tu.file_id).collect();

    let planner = ariadne_cluster::planner::OptimizationPlanner::new(&graph, &model, &pipeline.discovery.files, &tu_file_ids);
    let plans = planner.plan();

    pipeline.manager.save_plan_artifacts(&pipeline.repo.root, &plans, &pipeline.fps.plan)?;

    println!("Optimization plans: {}", plans.len());
    println!();
    println!("{}", pipeline.stats.format_summary());
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
