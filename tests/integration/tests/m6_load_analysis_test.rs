use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use ariadne_cli::{
    analyze_dependency_load, build_source_index, get_or_build_reduced_model,
};
use ariadne_graph::{LoadAnalyzer, ReducedSemanticModel};
use ariadne_pipeline::Pipeline;
use ariadne_report::ArtifactManager;

/// The real reduced-stage fingerprint, matching every other writer of this
/// fixture's reduced artifact so shared-contents races stay harmless.
fn reduced_fingerprint(repo_path: &std::path::Path) -> Result<String> {
    let repo = ariadne_repo::Repository::locate(repo_path)?;
    let discovery = repo.discover()?;
    // M18.4: must match the pipeline's injection so fingerprints agree.
    let build = ariadne_build::BuildDiscoverer::new(&repo.root)
        .with_repo_discovery(&discovery)
        .with_unit_shaper(Some(ariadne_lang_cpp::cpp_default_shaper()))
        .discover()?;
    let fps = ariadne_index::StageFingerprints::from_inputs(
        &repo.root,
        &discovery,
        &build.contexts,
        &build.translation_units,
    );
    Ok(fps.reduced)
}

#[derive(Deserialize)]
struct LoadManifest {
    pub total_analyzed_symbols: usize,
    pub total_analyzed_files: usize,
    pub total_collateral_load: usize,
}

#[test]
fn test_m6_symbol_load_and_collateral_quantification() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");

    // Build index and reduce first
    let index_result = build_source_index(&repo_path)?;
    let model = ReducedSemanticModel::reduce(&index_result);
    let repo = ariadne_repo::Repository::locate(&repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repo)?;
    let reduced_fp = reduced_fingerprint(&repo_path)?;
    artifacts.save_reduced_artifacts(&model, &reduced_fp)?;

    let graph = {
        let mut pipeline = Pipeline::new(&repo_path, None)?;
        pipeline.get_or_build_include_graph()?.0
    };
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let metric = analyzer.analyze_symbol_load("Thing")?;

    assert_eq!(metric.semantic_load, 2);
    assert_eq!(metric.include_load, 6);
    assert_eq!(metric.total_load, 6);
    assert_eq!(metric.collateral_load, 4);

    // Check that ThingConfig is present in semantic_symbols and NOT in collateral_symbols
    let thing_config = model
        .find_symbol("ThingConfig")
        .expect("ThingConfig symbol should exist");
    assert!(metric.semantic_symbols.contains(&thing_config.id));
    assert!(!metric.collateral_symbols.contains(&thing_config.id));

    // Check largest collateral source
    let largest = metric
        .largest_collateral_source
        .expect("Largest collateral source should be Some");
    assert_eq!(
        largest.path.file_name().unwrap().to_str().unwrap(),
        "heavy.h"
    );
    assert_eq!(largest.symbol_count, 2);

    Ok(())
}

#[test]
fn test_m6_explain_provenance() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");

    let graph = {
        let mut pipeline = Pipeline::new(&repo_path, None)?;
        pipeline.get_or_build_include_graph()?.0
    };
    let (_, model) = get_or_build_reduced_model(&repo_path)?;
    let analyzer = LoadAnalyzer::new(&graph, &model);

    let explanation = analyzer.explain_symbol_load("Thing", None)?;

    let formatted = explanation.format_explanation();
    assert!(formatted.contains("Thing"));
    assert!(formatted.contains("thing.h"));
    assert!(formatted.contains("heavy.h"));
    assert!(formatted.contains("heavy.h is included by thing.h"));
    assert!(
        formatted.contains("No semantic dependency from Thing reaches symbols owned by heavy.h")
    );
    assert!(formatted.contains("↓"));
    assert!(formatted.contains("Reason:"));

    Ok(())
}

#[test]
fn test_m6_file_load_analysis() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");

    let graph = {
        let mut pipeline = Pipeline::new(&repo_path, None)?;
        pipeline.get_or_build_include_graph()?.0
    };
    let (_, model) = get_or_build_reduced_model(&repo_path)?;
    let analyzer = LoadAnalyzer::new(&graph, &model);

    let file_path = PathBuf::from("include/thing.h");
    let metric = analyzer.analyze_file_load(&file_path)?;

    assert_eq!(metric.semantic_load, 4);
    assert_eq!(metric.collateral_load, 2);
    assert_eq!(metric.total_load, 6);

    let largest = metric
        .largest_collateral_source
        .expect("Largest collateral source should be Some");
    assert_eq!(
        largest.path.file_name().unwrap().to_str().unwrap(),
        "heavy.h"
    );

    Ok(())
}

/// The real load-stage fingerprint, matching what Pipeline persists with.
fn load_fingerprint(repo_path: &std::path::Path) -> Result<String> {
    let repo = ariadne_repo::Repository::locate(repo_path)?;
    let discovery = repo.discover()?;
    // M18.4: must match the pipeline's injection so fingerprints agree.
    let build = ariadne_build::BuildDiscoverer::new(&repo.root)
        .with_repo_discovery(&discovery)
        .with_unit_shaper(Some(ariadne_lang_cpp::cpp_default_shaper()))
        .discover()?;
    let fps = ariadne_index::StageFingerprints::from_inputs(
        &repo.root,
        &discovery,
        &build.contexts,
        &build.translation_units,
    );
    Ok(fps.load)
}

#[test]
fn test_m6_targeted_load_query_keeps_stage_full() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");

    // Persist the authoritative full-stage report.
    analyze_dependency_load(&repo_path, None, None)?;

    // Targeted single-metric queries must not overwrite it.
    analyze_dependency_load(&repo_path, Some("Thing"), None)?;
    analyze_dependency_load(
        &repo_path,
        None,
        Some(std::path::Path::new("include/thing.h")),
    )?;

    let repo = ariadne_repo::Repository::locate(&repo_path)?;
    let manager = ArtifactManager::prepare_for_repo(&repo)?;
    let fp = load_fingerprint(&repo_path)?;
    let report = manager.load_load_checked(&fp)?;

    assert!(
        report.symbol_metrics.len() > 1,
        "--symbol query must not shrink the persisted load stage to one metric"
    );
    assert!(
        report.file_metrics.len() > 1,
        "--file query must not shrink the persisted load stage to one metric"
    );

    let manifest_content = fs::read_to_string(repo_path.join("artifacts/load/manifest.json"))?;
    let manifest: LoadManifest = serde_json::from_str(&manifest_content)?;
    assert!(manifest.total_analyzed_symbols > 1);

    Ok(())
}

#[test]
fn test_m6_load_artifacts() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");

    analyze_dependency_load(&repo_path, None, None)?;

    let artifacts_dir = repo_path.join("artifacts/load");
    assert!(artifacts_dir.join("report.json").exists());
    assert!(artifacts_dir.join("report.bin").exists());
    assert!(artifacts_dir.join("manifest.json").exists());

    let manifest_content = fs::read_to_string(artifacts_dir.join("manifest.json"))?;
    let manifest: LoadManifest = serde_json::from_str(&manifest_content)?;

    assert!(manifest.total_analyzed_symbols > 0);
    assert!(manifest.total_analyzed_files > 0);

    let repo = ariadne_repo::Repository::locate(&repo_path)?;
    let manager = ArtifactManager::prepare_for_repo(&repo)?;
    let _loaded_report = manager.load_load_artifacts()?;

    Ok(())
}
