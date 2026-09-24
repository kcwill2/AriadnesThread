use std::fs;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use ariadne_cli;
use ariadne_validate::{ApplyOutcome, TierResult, ValidationStatus, ValidationTier, TierValidator, FileSnapshot, TransactionApplier};
use ariadne_transform::{ExtractionResult, GeneratedHeader};
use ariadne_report::ArtifactManager;
use ariadne_repo::Repository;

fn hash_dir(path: &Path) -> String {
    let mut entries = Vec::new();
    let mut walk_dir = vec![path.to_path_buf()];
    while let Some(dir) = walk_dir.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let p = entry.path();
            if p.is_dir() {
                walk_dir.push(p);
            } else {
                entries.push(p);
            }
        }
    }
    entries.sort();
    
    let mut hasher = Sha256::new();
    for p in entries {
        hasher.update(p.to_string_lossy().as_bytes());
        if let Ok(content) = fs::read(&p) {
            hasher.update(content);
        }
    }
    format!("{:x}", hasher.finalize())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn setup_fixture() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    let fixture_dir = PathBuf::from("../../tests/fixtures/basic");
    
    copy_dir_all(&fixture_dir, dir.path().join("basic")).unwrap();
    
    dir
}

#[test]
fn test_m10_apply_success_path() {
    let dir = setup_fixture();
    let repo_path = dir.path().join("basic");
    
    let plans = ariadne_cli::handle_plan(&repo_path).unwrap();
    if plans.is_empty() { return; }
    let plan = &plans[0];
    
    ariadne_cli::handle_apply(&repo_path, &plan.id.0).unwrap();
    
    let artifacts = ArtifactManager::init(repo_path.clone()).unwrap();
    let tx_dir = artifacts.artifact_root.join("transaction");
    
    assert!(tx_dir.join("manifest.json").exists());
    let manifest = fs::read_to_string(tx_dir.join("manifest.json")).unwrap();
    assert!(manifest.contains("Committed"));
}

#[test]
fn test_m10_rollback_identity() {
    let dir = setup_fixture();
    let repo_path = dir.path().join("basic");
    
    let plans = ariadne_cli::handle_plan(&repo_path).unwrap();
    if plans.is_empty() { return; }
    let plan = &plans[0];
    
    let repo = Repository::locate(&repo_path).unwrap();
    let artifacts = ArtifactManager::prepare_for_repo(&repo).unwrap();
    
    // Inject deliberately broken extraction
    let mut extraction = ariadne_transform::HeaderExtractor::new(&ariadne_cli::get_or_build_reduced_model(&repo_path).unwrap().1).extract(plan);
    extraction.errors.push("Fake error".to_string()); // Make tier 1 fail

    let migration = ariadne_transform::MigrationPreview {
        plan_id: plan.id.0.clone(),
        rewrites: vec![],
    };
    
    let hash_before = hash_dir(&repo_path);
    let (snapshot, new_paths) = TransactionApplier::apply(&repo.root, &extraction, &migration).unwrap();
    let t1 = TierValidator::validate_structural(&extraction);
    
    let outcome = ApplyOutcome::RolledBack {
        failed_tier: t1,
        tiers_passed: vec![],
    };
    
    TransactionApplier::rollback(&snapshot, &new_paths).unwrap();
    
    let hash_after = hash_dir(&repo_path);
    artifacts.save_transaction_artifacts(&plan.id.0, &outcome).unwrap();
    
    assert_eq!(hash_before, hash_after);
}

#[test]
fn test_m10_tier_validation_structural() {
    let extraction = ExtractionResult {
        plan_id: "P1".into(),
        headers: vec![],
        errors: vec!["Cycle detected".into()],
    };
    let result = TierValidator::validate_structural(&extraction);
    assert_eq!(result.tier, ValidationTier::Structural);
    assert_eq!(result.status, ValidationStatus::Fail("Cycle detected".into()));
}

#[test]
fn test_m10_format_output_committed() {
    let outcome = ApplyOutcome::Committed {
        tiers_passed: vec![
            TierResult { tier: ValidationTier::Structural, status: ValidationStatus::Pass },
            TierResult { tier: ValidationTier::GeneratedHeaders, status: ValidationStatus::Pass },
            TierResult { tier: ValidationTier::AffectedTUs, status: ValidationStatus::Pass },
        ]
    };
    
    let out = outcome.format_output();
    assert_eq!(out, "Structural validation: PASS\nGenerated header validation: PASS\nAffected TU validation: PASS\n\nCommitted.");
}

#[test]
fn test_m10_format_output_rolled_back() {
    let outcome = ApplyOutcome::RolledBack {
        failed_tier: TierResult { tier: ValidationTier::GeneratedHeaders, status: ValidationStatus::Fail("Missing inc".into()) },
        tiers_passed: vec![
            TierResult { tier: ValidationTier::Structural, status: ValidationStatus::Pass },
        ]
    };
    
    let out = outcome.format_output();
    assert_eq!(out, "Structural validation: PASS\nGenerated header validation: FAIL\n\nRolling back...\n\nRepository restored.");
}
