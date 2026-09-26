use crate::commands::plan::handle_plan;
use anyhow::{bail, Result};
use ariadne_build::BuildDiscoverer;
use ariadne_repo::Repository;
use ariadne_report::ArtifactManager;
use ariadne_transform::{GeneratedHeader, IncludeRewriter};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub fn handle_transform_extract(repo_path: &PathBuf, plan_id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plans = artifacts.load_plan_artifacts()?;
    let plan = match plans.iter().find(|p| p.id.0 == plan_id) {
        Some(p) => p,
        None => bail!("Plan {} not found", plan_id),
    };

    let model = artifacts.load_reduced_artifacts()?;
    let extractor = ariadne_transform::HeaderExtractor::new(&model);
    let result = extractor.extract(plan);

    let output_dir = artifacts.artifact_root.join("generated");
    artifacts.save_extraction_artifacts(&result, &output_dir)?;

    println!(
        "Extracted {} headers from plan {}",
        result.headers.len(),
        plan_id
    );
    println!("generated/");

    let mut files = result
        .headers
        .iter()
        .map(|h| h.filename.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    files.sort();

    for (i, f) in files.iter().enumerate() {
        if i == files.len() - 1 {
            println!("└── {}", f);
        } else {
            println!("├── {}", f);
        }
    }

    Ok(())
}

pub fn handle_migrate(repo_path: &PathBuf, plan_id: &str, preview: bool) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    // 1. Load the plan
    let plan = load_plan_for_action(&artifacts, repo_path, plan_id)?;

    // 2-3. Build symbol->header map and generated headers list from the plan
    let (symbol_name_to_header, generated_headers) = plan_rewrite_targets(&plan);

    // 4. Find consumers: TUs that include the target file (heuristic: check source file content)
    let consumers = find_plan_consumers(&repository, &plan)?;

    // 5. Build preview
    let rewriter = IncludeRewriter::new(symbol_name_to_header, generated_headers);
    let preview_result = rewriter.build_preview(plan_id.to_string(), &consumers);

    // 6. Print diff
    if preview {
        println!("Migration preview for {}:", plan_id);
        println!();
        if preview_result.rewrites.is_empty() {
            println!(
                "  No consumers found for target: {}",
                plan.target_file.display()
            );
        } else {
            println!("{}", preview_result.format_diff());
            println!("No files changed.");
        }
    }

    // 7. Save artifacts
    artifacts.save_migration_artifacts(&preview_result)?;

    println!(
        "Migration preview: {} effective rewrites",
        preview_result.effective_rewrites()
    );

    Ok(())
}

pub fn handle_apply(repo_path: &PathBuf, plan_id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plan = load_plan_for_action(&artifacts, repo_path, plan_id)?;

    let model = artifacts.load_reduced_artifacts()?;

    let extractor = ariadne_transform::HeaderExtractor::new(&model);
    let extraction = extractor.extract(&plan);

    let (symbol_name_to_header, generated_headers) = plan_rewrite_targets(&plan);

    let consumers = find_plan_consumers(&repository, &plan)?;

    let rewriter =
        ariadne_transform::IncludeRewriter::new(symbol_name_to_header, generated_headers);
    let migration = rewriter.build_preview(plan_id.to_string(), &consumers);

    println!("Applying {}...\n", plan_id);

    let (snapshot, new_paths) =
        ariadne_validate::TransactionApplier::apply(&repository.root, &extraction, &migration)?;

    commit_or_rollback(
        &artifacts,
        plan_id,
        &repository,
        &extraction,
        &migration,
        &snapshot,
        &new_paths,
    )
}

fn load_plan_for_action(
    artifacts: &ArtifactManager,
    repo_path: &PathBuf,
    plan_id: &str,
) -> Result<ariadne_cluster::plan::OptimizationPlan> {
    let plans = artifacts
        .load_plan_artifacts()
        .or_else(|_| handle_plan(repo_path).map(|_| vec![]))
        .unwrap_or_default();
    let plans = if plans.is_empty() {
        artifacts.load_plan_artifacts()?
    } else {
        plans
    };
    plans
        .into_iter()
        .find(|p| p.id.0 == plan_id)
        .ok_or_else(|| anyhow::anyhow!("Plan {} not found", plan_id))
}

fn plan_rewrite_targets(
    plan: &ariadne_cluster::plan::OptimizationPlan,
) -> (BTreeMap<String, PathBuf>, Vec<GeneratedHeader>) {
    // Build symbol_name_to_header map from plan's proposed_extractions
    let mut symbol_name_to_header: BTreeMap<String, PathBuf> = BTreeMap::new();
    // Generated headers list (from proposed_extractions, conservative: no required_includes)
    let mut header_map: BTreeMap<PathBuf, GeneratedHeader> = BTreeMap::new();
    for ext in &plan.proposed_extractions {
        symbol_name_to_header.insert(ext.symbol_name.clone(), ext.target_header.clone());
        header_map
            .entry(ext.target_header.clone())
            .or_insert_with(|| GeneratedHeader {
                filename: ext.target_header.clone(),
                pragma_once: true,
                symbols: vec![],
                required_includes: vec![],
            });
    }
    (symbol_name_to_header, header_map.into_values().collect())
}

fn find_plan_consumers(
    repository: &Repository,
    plan: &ariadne_cluster::plan::OptimizationPlan,
) -> Result<Vec<(PathBuf, Vec<PathBuf>, Vec<String>)>> {
    let repo_discovery = repository.discover()?;
    let build_discoverer =
        BuildDiscoverer::new(&repository.root).with_repo_discovery(&repo_discovery);
    let build_result = build_discoverer.discover()?;

    let target_include = plan
        .target_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let all_symbol_names: Vec<String> = plan
        .proposed_extractions
        .iter()
        .map(|e| e.symbol_name.clone())
        .collect();

    let mut consumers: Vec<(PathBuf, Vec<PathBuf>, Vec<String>)> = Vec::new();
    for tu in &build_result.translation_units {
        let abs_path = repository.root.join(&tu.source_path);
        if let Ok(content) = fs::read_to_string(&abs_path) {
            // Check if this TU includes the target header
            if content.contains(&format!("#include \"{}\"", target_include))
                || content.contains(&format!("#include <{}>", target_include))
            {
                consumers.push((
                    tu.source_path.clone(),
                    vec![plan.target_file.clone()],
                    all_symbol_names.clone(), // conservative: all symbols
                ));
            }
        }
    }
    Ok(consumers)
}

fn commit_or_rollback(
    artifacts: &ArtifactManager,
    plan_id: &str,
    repository: &Repository,
    extraction: &ariadne_transform::ExtractionResult,
    migration: &ariadne_transform::MigrationPreview,
    snapshot: &ariadne_validate::FileSnapshot,
    new_paths: &[PathBuf],
) -> Result<()> {
    let mut tiers_passed = Vec::new();

    let t1 = ariadne_validate::TierValidator::validate_structural(extraction);
    if t1.status == ariadne_validate::ValidationStatus::Pass {
        tiers_passed.push(t1);
    } else {
        return report_rollback(artifacts, plan_id, t1, tiers_passed, snapshot, new_paths);
    }

    let header_output_dir = repository.root.join("generated");
    let t2 =
        ariadne_validate::TierValidator::validate_generated_headers(extraction, &header_output_dir);
    if t2.status == ariadne_validate::ValidationStatus::Pass {
        tiers_passed.push(t2);
    } else {
        return report_rollback(artifacts, plan_id, t2, tiers_passed, snapshot, new_paths);
    }

    let t3 = ariadne_validate::TierValidator::validate_affected_tus(migration, &header_output_dir);
    if t3.status == ariadne_validate::ValidationStatus::Pass {
        tiers_passed.push(t3);
    } else {
        return report_rollback(artifacts, plan_id, t3, tiers_passed, snapshot, new_paths);
    }

    let outcome = ariadne_validate::ApplyOutcome::Committed { tiers_passed };
    println!("{}", outcome.format_output());
    artifacts.save_transaction_artifacts(plan_id, &outcome)?;

    Ok(())
}

fn report_rollback(
    artifacts: &ArtifactManager,
    plan_id: &str,
    failed_tier: ariadne_validate::TierResult,
    tiers_passed: Vec<ariadne_validate::TierResult>,
    snapshot: &ariadne_validate::FileSnapshot,
    new_paths: &[PathBuf],
) -> Result<()> {
    let outcome = ariadne_validate::ApplyOutcome::RolledBack {
        failed_tier,
        tiers_passed,
    };
    println!("{}", outcome.format_output());
    ariadne_validate::TransactionApplier::rollback(snapshot, new_paths)?;
    artifacts.save_transaction_artifacts(plan_id, &outcome)?;
    Ok(())
}
