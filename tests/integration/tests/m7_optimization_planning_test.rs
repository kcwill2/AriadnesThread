use anyhow::Result;
use std::path::PathBuf;

use ariadne_cli::{build_source_index, plan_extractions, show_optimization_plan};
use ariadne_cluster::checker::ConstraintChecker;
use ariadne_cluster::plan::RejectionReason;
use ariadne_report::ArtifactManager;

#[test]
fn test_m7_plan_generation_basic() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");
    let repo = ariadne_repo::Repository::locate(&repo_path)?;
    // Ensure prerequisites are built (index -> reduce)
    build_source_index(&repo_path)?;

    // Generate plans
    let plans = plan_extractions(&repo_path)?;

    // Validate plan properties for thing.h
    let plan = plans
        .iter()
        .find(|p| {
            p.target_file
                .to_string_lossy()
                .replace("\\", "/")
                .ends_with("thing.h")
        })
        .expect("Plan for thing.h should exist");

    // Check frontier
    assert!(plan
        .optimization_frontier
        .iter()
        .any(|p| p.to_string_lossy().replace("\\", "/").ends_with("thing.h")));
    assert_eq!(plan.collateral_load_before, 2);

    // Check problem description
    assert!(plan.problem.contains("symbols of collateral load"));

    // Check validation required TUs
    assert!(!plan.validation_required_tus.is_empty());

    Ok(())
}

#[test]
fn test_m7_plan_artifacts_and_determinism() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");
    let repo = ariadne_repo::Repository::locate(&repo_path)?;

    let plans_first = plan_extractions(&repo_path)?;

    let artifacts = ArtifactManager::prepare_for_repo(&repo)?;
    let plan_dir = artifacts.artifact_root.join("plans");

    // Verify artifacts exist
    assert!(plan_dir.join("plans.json").exists());
    assert!(plan_dir.join("plans.bin").exists());
    assert!(plan_dir.join("manifest.json").exists());

    // Determinism
    let plans_second = plan_extractions(&repo_path)?;
    assert_eq!(plans_first.len(), plans_second.len());
    for (a, b) in plans_first.iter().zip(plans_second.iter()) {
        assert_eq!(a.id.0, b.id.0);
        assert_eq!(a.target_file, b.target_file);
        assert_eq!(a.optimization_frontier, b.optimization_frontier);
    }

    Ok(())
}

#[test]
fn test_m7_plan_show_formatting() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");

    // Generate plans to ensure they are created
    let plans = plan_extractions(&repo_path)?;

    // Use a plan that targets a header (has validation_required_tus)
    let plan = plans
        .iter()
        .find(|p| {
            let s = p.target_file.to_string_lossy();
            let s = s.replace('\\', "/");
            s.ends_with(".h")
        })
        .unwrap_or_else(|| plans.first().unwrap());

    let detail = plan.format_detail();

    assert!(detail.contains(&plan.id.0), "plan id missing from detail");
    assert!(detail.contains("Target:"));
    assert!(detail.contains("Problem:"));
    assert!(detail.contains("Proposed extraction:"));
    assert!(detail.contains("Optimization frontier:"));
    assert!(detail.contains("Expected result:"));
    assert!(detail.contains("Risk:"));

    // Only assert Validation required if this plan actually has TUs
    if !plan.validation_required_tus.is_empty() {
        assert!(detail.contains("Validation required:"));
    }

    // handle_plan_show should succeed for this plan's ID
    show_optimization_plan(&repo_path, &plan.id.0)?;

    Ok(())
}

#[test]
fn test_m7_rejection_cases() -> Result<()> {
    use ariadne_index::types::SymbolId;
    use ariadne_repo::OptimizationPolicy;
    use std::collections::{HashMap, HashSet};

    let target_file = PathBuf::from("target.h");
    let symbol = SymbolId("MyClass".to_string());

    let mut file_policies = HashMap::new();
    file_policies.insert(target_file.clone(), OptimizationPolicy::Frozen);
    let checker = ConstraintChecker::new(file_policies, HashSet::new());

    let err = checker
        .validate_extraction(&symbol, &target_file, false, false, None)
        .unwrap_err();
    assert!(matches!(err.reason, RejectionReason::FrozenFile(_)));

    let mut file_policies = HashMap::new();
    file_policies.insert(target_file.clone(), OptimizationPolicy::Contract);
    let checker = ConstraintChecker::new(file_policies, HashSet::new());

    let err = checker
        .validate_extraction(&symbol, &target_file, false, false, None)
        .unwrap_err();
    assert!(matches!(err.reason, RejectionReason::ContractViolation(_)));

    let mut non_movable = HashSet::new();
    non_movable.insert(symbol.clone());
    let checker = ConstraintChecker::new(HashMap::new(), non_movable);

    let err = checker
        .validate_extraction(&symbol, &target_file, false, false, None)
        .unwrap_err();
    assert!(matches!(
        err.reason,
        RejectionReason::NonMovableDeclaration(_)
    ));

    let checker = ConstraintChecker::new(HashMap::new(), HashSet::new());

    let err = checker
        .validate_extraction(&symbol, &target_file, false, false, Some("dep".to_string()))
        .unwrap_err();
    assert!(matches!(
        err.reason,
        RejectionReason::UnresolvedSemanticDependency(_)
    ));

    let err = checker
        .validate_extraction(&symbol, &target_file, false, true, None)
        .unwrap_err();
    assert!(matches!(err.reason, RejectionReason::CycleCreation(_)));

    Ok(())
}
