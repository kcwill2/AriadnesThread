use anyhow::Result;
use ariadne_pipeline::Pipeline;
use ariadne_repo::Repository;
use ariadne_report::ArtifactManager;
use std::path::PathBuf;

/// Distinct discovered languages whose registered plugin disables the
/// planning stage (plan M22, D23). Registry lookup only (rule R3): a language
/// with no plugin (e.g. `Other`) is not a gap — the planner never hears of
/// it; a plugin with `planning == true` never is either. Each gap carries the
/// first discovered path of that language so the rejection names an honest
/// `target_file`. Pure so the rule is testable without a repository.
fn planning_gaps(
    files: &[ariadne_repo::FileRecord],
    registry: &ariadne_lang::LanguageRegistry,
) -> Vec<ariadne_cluster::planner::PlanningGap> {
    let mut gaps: Vec<ariadne_cluster::planner::PlanningGap> = Vec::new();
    for record in files {
        let Some(plugin) = registry.plugin_for_id(record.language) else {
            continue;
        };
        if plugin.capabilities().planning {
            continue;
        }
        if gaps.iter().any(|gap| gap.language == record.language) {
            continue;
        }
        gaps.push(ariadne_cluster::planner::PlanningGap {
            language: record.language,
            sample_path: record.path.clone(),
        });
    }
    gaps
}

pub fn plan_extractions(
    repo_path: &PathBuf,
) -> Result<Vec<ariadne_cluster::plan::OptimizationPlan>> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let (graph, _closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;
    let tu_file_ids: Vec<_> = pipeline
        .build
        .translation_units
        .iter()
        .map(|tu| tu.file_id)
        .collect();

    let registry = crate::registry::default_registry();
    let gaps = planning_gaps(&pipeline.discovery.files, &registry);

    let mut planner = ariadne_cluster::planner::OptimizationPlanner::new(
        &graph,
        &model,
        &pipeline.discovery.files,
        &tu_file_ids,
        &ariadne_lang_cpp::cpp_aggregation_filter,
        &ariadne_lang_cpp::cpp_target_path_for,
        None,
        ariadne_lang::CyclePolicy::Reject,
    );
    planner.with_planning_gaps(gaps);
    let (plans, rejections) = planner.plan_with_rejections();

    pipeline
        .manager
        .save_plan_artifacts(&pipeline.repo.root, &plans, &pipeline.fps.plan)?;

    println!("Optimization plans: {}", plans.len());
    if !rejections.is_empty() {
        println!();
        for rejection in &rejections {
            println!("Planning rejected: {}", rejection.explanation);
        }
    }
    println!();
    println!("{}", pipeline.stats.format_summary());
    println!();
    for plan in &plans {
        println!("{}", plan.format_detail());
    }

    Ok(plans)
}

pub fn show_optimization_plan(repo_path: &PathBuf, id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plans = match artifacts.load_plan_artifacts() {
        Ok(p) => p,
        Err(_) => plan_extractions(repo_path)?,
    };

    if let Some(plan) = plans.iter().find(|p| p.id.0 == id) {
        println!("{}", plan.format_detail());
    } else {
        println!("Plan {} not found.", id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::planning_gaps;
    use ariadne_repo::{
        FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
    };
    use std::path::PathBuf;

    fn record(rel: &str, kind: FileKind, language: SourceLanguage) -> FileRecord {
        let path = PathBuf::from(rel);
        FileRecord {
            id: FileId::from_relative_path(rel),
            path: path.clone(),
            canonical_path: path,
            kind,
            size: 0,
            content_hash: String::new(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language,
        }
    }

    #[test]
    fn planning_gaps_flags_planning_unsupported_language_once() {
        let registry = crate::registry::default_registry();
        let files = vec![
            record("a.cpp", FileKind::Source, SourceLanguage::Cpp),
            record("B.scala", FileKind::ScalaSource, SourceLanguage::Scala),
            record("C.scala", FileKind::ScalaSource, SourceLanguage::Scala),
        ];

        let gaps = planning_gaps(&files, &registry);

        assert_eq!(
            gaps.len(),
            1,
            "Scala (planning=false) is the only gap; C++ plans"
        );
        assert_eq!(gaps[0].language, SourceLanguage::Scala);
        assert_eq!(
            gaps[0].sample_path,
            PathBuf::from("B.scala"),
            "the gap carries a representative discovered path"
        );
    }

    #[test]
    fn planning_gaps_empty_when_every_language_plans() {
        let registry = crate::registry::default_registry();
        let files = vec![record("a.cpp", FileKind::Source, SourceLanguage::Cpp)];

        assert!(planning_gaps(&files, &registry).is_empty());
    }

    #[test]
    fn planning_gaps_skips_unclaimed_languages_without_a_plugin() {
        // `Other` (e.g. a discovered `.md`/`.txt` path) has no registered
        // plugin, so it is not a capability gap — the planner never hears of
        // it. Only a REGISTERED plugin with planning=false is one.
        let registry = crate::registry::default_registry();
        let files = vec![
            record("notes.md", FileKind::Other, SourceLanguage::Other),
            record("data.txt", FileKind::Other, SourceLanguage::Other),
            record("a.cpp", FileKind::Source, SourceLanguage::Cpp),
        ];

        assert!(
            planning_gaps(&files, &registry).is_empty(),
            "a language no plugin claims must not produce a planning rejection"
        );
    }
}
