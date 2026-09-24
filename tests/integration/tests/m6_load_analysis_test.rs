use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use serde::Deserialize;

use ariadne_cli::{build_include_graph, get_or_build_reduced_model, handle_analyze_load, handle_explain_load, handle_index};
use ariadne_graph::{LoadAnalyzer, ReducedSemanticModel};
use ariadne_report::ArtifactManager;

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
    let index_result = handle_index(&repo_path)?;
    let model = ReducedSemanticModel::reduce(&index_result);
    let repo = ariadne_repo::Repository::locate(&repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repo)?;
    artifacts.save_reduced_artifacts(&model)?;
    
    let (_, graph, _closures, _report) = build_include_graph(&repo_path)?;
    let analyzer = LoadAnalyzer::new(&graph, &model);
    let metric = analyzer.analyze_symbol_load("Thing")?;
    
    assert_eq!(metric.semantic_load, 2);
    assert_eq!(metric.include_load, 4);
    assert_eq!(metric.total_load, 4);
    assert_eq!(metric.collateral_load, 2);
    
    // Check that ThingConfig is present in semantic_symbols and NOT in collateral_symbols
    let thing_config = model.find_symbol("ThingConfig").expect("ThingConfig symbol should exist");
    assert!(metric.semantic_symbols.contains(&thing_config.id));
    assert!(!metric.collateral_symbols.contains(&thing_config.id));
    
    // Check largest collateral source
    let largest = metric.largest_collateral_source.expect("Largest collateral source should be Some");
    assert_eq!(largest.path.file_name().unwrap().to_str().unwrap(), "heavy.h");
    assert_eq!(largest.symbol_count, 2);
    
    Ok(())
}

#[test]
fn test_m6_explain_provenance() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");
    
    let (_, graph, _closures, _report) = build_include_graph(&repo_path)?;
    let (_, model) = get_or_build_reduced_model(&repo_path)?;
    let analyzer = LoadAnalyzer::new(&graph, &model);
    
    let explanation = analyzer.explain_symbol_load("Thing", None)?;
    
    let formatted = explanation.format_explanation();
    assert!(formatted.contains("Thing"));
    assert!(formatted.contains("thing.h"));
    assert!(formatted.contains("heavy.h"));
    assert!(formatted.contains("heavy.h is included by thing.h"));
    assert!(formatted.contains("No semantic dependency from Thing reaches symbols owned by heavy.h"));
    assert!(formatted.contains("↓"));
    assert!(formatted.contains("Reason:"));
    
    Ok(())
}

#[test]
fn test_m6_file_load_analysis() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");
    
    let (_, graph, _closures, _report) = build_include_graph(&repo_path)?;
    let (_, model) = get_or_build_reduced_model(&repo_path)?;
    let analyzer = LoadAnalyzer::new(&graph, &model);
    
    let file_path = PathBuf::from("include/thing.h");
    let metric = analyzer.analyze_file_load(&file_path)?;
    
    assert_eq!(metric.semantic_load, 2);
    assert_eq!(metric.collateral_load, 2);
    assert_eq!(metric.total_load, 4);
    
    let largest = metric.largest_collateral_source.expect("Largest collateral source should be Some");
    assert_eq!(largest.path.file_name().unwrap().to_str().unwrap(), "heavy.h");
    
    Ok(())
}

#[test]
fn test_m6_load_artifacts() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");
    
    handle_analyze_load(&repo_path, None, None)?;
    
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
