use anyhow::Result;
use ariadne_cli::run_end_to_end_optimize;
use ariadne_cli::EndToEndOptimizer;
use ariadne_repo::Repository;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

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

fn setup_temp_repo(fixture_name: &str) -> Result<(tempfile::TempDir, PathBuf)> {
    let temp_dir = tempdir()?;
    let current_dir = env::current_dir()?;
    let workspace_root = current_dir.parent().unwrap().parent().unwrap();
    let fixtures_dir = workspace_root.join("tests").join("fixtures");
    let src_repo = fixtures_dir.join(fixture_name);

    if !src_repo.exists() {
        return Err(anyhow::anyhow!("Fixture not found at {:?}", src_repo));
    }

    let repo_path = temp_dir.path().join(fixture_name);
    copy_dir_all(&src_repo, &repo_path)?;

    Ok((temp_dir, repo_path))
}

fn hash_dir(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};

    let mut entries = Vec::new();
    let mut walk_dir = vec![path.to_path_buf()];
    while let Some(dir) = walk_dir.pop() {
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.into_iter().filter_map(Result::ok) {
                let p = entry.path();
                if p.is_dir() {
                    let name = p.file_name().unwrap().to_string_lossy();
                    if name != "artifacts" && name != ".ariadne" {
                        walk_dir.push(p);
                    }
                } else {
                    entries.push(p);
                }
            }
        }
    }
    entries.sort();

    let mut hasher = Sha256::new();
    for p in entries {
        hasher.update(p.to_string_lossy().as_bytes());
        if let Ok(content) = fs::read(&p) {
            hasher.update(&content);
        }
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[test]
fn test_m14_end_to_end_optimization_flow() -> Result<()> {
    let (_temp, repo_path) = setup_temp_repo("basic")?;

    let optimizer = EndToEndOptimizer::new(&repo_path);
    let report = optimizer.optimize_repository()?;

    assert!(report.total_files > 0);
    assert!(report.total_tus > 0);
    assert!(report.total_symbols > 0);

    let output = report.format_m14_output();
    assert!(output.contains("Repository:"));
    assert!(output.contains("Analysis:"));
    assert!(output.contains("Plan:"));
    assert!(output.contains("Preview:"));
    assert!(output.contains("Validation:"));
    assert!(output.contains("Build:"));
    assert!(output.contains("Measurement:"));
    assert!(output.contains("Committed:"));
    assert!(output.contains("Residual collateral load:"));

    let report_path = repo_path.join("artifacts/optimize/report.json");
    assert!(report_path.exists());

    Ok(())
}

#[test]
fn test_m14_rollback_on_failed_validation() -> Result<()> {
    let (_temp, repo_path) = setup_temp_repo("basic")?;
    let initial_hash = hash_dir(&repo_path)?;

    // Test rollback identity manually via TransactionApplier to simulate rollback
    use ariadne_transform::{ExtractionResult, MigrationPreview};
    use ariadne_validate::TransactionApplier;

    let repo = Repository::locate(&repo_path)?;

    // Dummy extraction
    let mut extraction = ExtractionResult {
        plan_id: "test_plan".into(),
        headers: vec![],
        errors: vec![],
    };
    extraction.errors.push("Simulated error".to_string());

    let migration = MigrationPreview {
        plan_id: "test_plan".into(),
        rewrites: vec![],
    };

    let (snapshot, new_paths) = TransactionApplier::apply_with(
        &repo.root,
        &extraction,
        &migration,
        &ariadne_lang_cpp::CppDependencyRewriter,
        &ariadne_lang_cpp::CppSourceExtractor,
    )?;

    // Rollback
    TransactionApplier::rollback(&snapshot, &new_paths)?;

    let final_hash = hash_dir(&repo_path)?;
    assert_eq!(initial_hash, final_hash);

    Ok(())
}

#[test]
fn test_m14_cli_optimize_output() -> Result<()> {
    let (_temp, repo_path) = setup_temp_repo("basic")?;

    let result = run_end_to_end_optimize(&repo_path);
    assert!(result.is_ok());

    Ok(())
}
