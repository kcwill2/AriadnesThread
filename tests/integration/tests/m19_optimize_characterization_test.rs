use anyhow::Result;
use ariadne_cli::{run_end_to_end_optimize, EndToEndOptimizer, OptimizationReport};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn provision_basic_repo_copy() -> Result<(TempDir, PathBuf)> {
    let guard = tempfile::tempdir()?;
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/basic");
    let repo_root = guard.path().join("basic");
    copy_dir_all(&fixture_root, &repo_root)?;
    Ok((guard, repo_root))
}

fn list_relative_files(root: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                names.push(
                    path.strip_prefix(root)?
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    names.sort();
    Ok(names)
}

#[test]
fn optimize_on_basic_fixture_locks_report_counts() -> Result<()> {
    let (_repo_guard, repo_root) = provision_basic_repo_copy()?;

    let report = EndToEndOptimizer::new(&repo_root).optimize_repository()?;

    assert_eq!(report.total_files, 5);
    assert_eq!(report.total_tus, 2);
    assert_eq!(report.total_symbols, 7);
    assert_eq!(report.total_semantic_edges, 19);
    assert_eq!(
        report.collateral_opportunities,
        vec![
            ariadne_cli::CollateralOpportunity {
                header_name: "heavy.h".to_string(),
                collateral_symbols: 9,
            },
            ariadne_cli::CollateralOpportunity {
                header_name: "thing.cpp".to_string(),
                collateral_symbols: 6,
            },
            ariadne_cli::CollateralOpportunity {
                header_name: "thing.h".to_string(),
                collateral_symbols: 1,
            },
            ariadne_cli::CollateralOpportunity {
                header_name: "thing_types.h".to_string(),
                collateral_symbols: 1,
            },
        ],
        "opportunities must stay sorted by descending collateral symbols, then name"
    );
    assert_eq!(report.safe_plans_available, 3);
    assert_eq!(report.affected_consumer_files, 2);
    assert_eq!(report.validation_total_tus, 2);
    assert_eq!(report.validation_passed_tus, 2);
    assert!(report.build_passed);
    assert_eq!(report.committed_transformations, 3);
    assert_eq!(report.residual_collateral_load, 9);
    Ok(())
}

#[test]
fn optimize_writes_report_artifact_equal_to_returned_report() -> Result<()> {
    let (_repo_guard, repo_root) = provision_basic_repo_copy()?;

    let report = EndToEndOptimizer::new(&repo_root).optimize_repository()?;

    let report_json = fs::read_to_string(
        repo_root
            .join("artifacts")
            .join("optimize")
            .join("report.json"),
    )?;
    let persisted: OptimizationReport = serde_json::from_str(&report_json)?;
    assert_eq!(
        persisted, report,
        "artifacts/optimize/report.json must round-trip identical to the returned report"
    );
    Ok(())
}

#[test]
fn optimize_writes_all_extracted_headers_and_leaves_consumer_sources_unchanged() -> Result<()> {
    let (_repo_guard, repo_root) = provision_basic_repo_copy()?;

    let consumer_before = fs::read_to_string(repo_root.join("tests").join("consumer.cpp"))?;
    let thing_impl_before = fs::read_to_string(repo_root.join("src").join("thing.cpp"))?;
    assert!(
        consumer_before.contains("#include \"thing.h\""),
        "fixture precondition"
    );
    assert!(!repo_root.join("generated").exists(), "precondition");

    let report = EndToEndOptimizer::new(&repo_root).optimize_repository()?;

    let generated_files = list_relative_files(&repo_root.join("generated"))?;
    assert_eq!(
        generated_files,
        vec![
            "include/ThingConfig_extracted.h",
            "include/Thing_Thing_extracted.h",
            "include/Thing_extracted.h",
            "include/Thing_get_id_extracted.h",
            "src/ThingConfig_extracted.h",
            "src/Thing_Thing_extracted.h",
            "src/Thing_extracted.h",
            "src/Thing_get_id_extracted.h",
            "tests/ThingConfig_extracted.h",
            "tests/Thing_extracted.h",
            "tests/main_extracted.h",
        ],
        "every plan's extracted headers land under repo/generated/<target dir>/, even for plans with zero consumers"
    );

    let consumer_after = fs::read_to_string(repo_root.join("tests").join("consumer.cpp"))?;
    let thing_impl_after = fs::read_to_string(repo_root.join("src").join("thing.cpp"))?;
    assert_eq!(
        consumer_before, consumer_after,
        "current behavior: committed transforms never edit consumer text because the old-include \
         replacement uses the full relative path (include/thing.h) while sources contain the bare \
         name (thing.h) — latent vacuous-rewrite bug, characterization only"
    );
    assert_eq!(thing_impl_before, thing_impl_after);
    assert_eq!(report.committed_transformations, 3);
    assert_eq!(report.affected_consumer_files, 2);
    Ok(())
}

#[test]
fn optimize_missing_repo_path_reports_locate_failure() -> Result<()> {
    let missing = std::env::temp_dir().join("ariadne-m19-does-not-exist-9f3a2b");
    let error = EndToEndOptimizer::new(&missing)
        .optimize_repository()
        .expect_err("optimize on a missing path must fail");
    let message = format!("{error:#}");
    assert!(
        message.contains("Failed to locate repository path"),
        "unexpected error text: {message}"
    );
    Ok(())
}

#[test]
fn handle_optimize_matches_direct_optimize_and_locks_output_header() -> Result<()> {
    let (_repo_guard, repo_root) = provision_basic_repo_copy()?;
    let direct = EndToEndOptimizer::new(&repo_root).optimize_repository()?;

    let (_second_guard, second_root) = provision_basic_repo_copy()?;
    let dispatched = run_end_to_end_optimize(&second_root)?;
    let rendered = dispatched.format_m14_output();

    assert_eq!(dispatched.total_files, direct.total_files);
    assert_eq!(dispatched.total_tus, direct.total_tus);
    assert_eq!(dispatched.total_symbols, direct.total_symbols);
    assert_eq!(dispatched.total_semantic_edges, direct.total_semantic_edges);
    assert_eq!(
        dispatched.collateral_opportunities,
        direct.collateral_opportunities
    );
    assert_eq!(dispatched.safe_plans_available, direct.safe_plans_available);
    assert_eq!(
        dispatched.affected_consumer_files,
        direct.affected_consumer_files
    );
    assert_eq!(
        dispatched.validation_passed_tus,
        direct.validation_passed_tus
    );
    assert_eq!(dispatched.validation_total_tus, direct.validation_total_tus);
    assert_eq!(dispatched.build_passed, direct.build_passed);
    assert_eq!(
        dispatched.committed_transformations,
        direct.committed_transformations
    );
    assert_eq!(
        dispatched.residual_collateral_load,
        direct.residual_collateral_load
    );

    assert!(rendered.contains("Repository:\n    5 files\n    2 translation units"));
    assert!(rendered.contains("Analysis:\n    7 symbols\n    19 semantic edges"));
    assert!(rendered.contains("Plan:\n    3 safe transformations available"));
    assert!(rendered.contains("Preview:\n    2 consumer files affected"));
    assert!(rendered.contains("Validation:\n    2 / 2 affected TUs passed"));
    assert!(rendered.contains("Build:\n    PASS"));
    assert!(rendered.contains("Committed:\n    3 transformations"));
    assert!(rendered.contains("Residual collateral load:\n    9 symbols"));
    Ok(())
}
