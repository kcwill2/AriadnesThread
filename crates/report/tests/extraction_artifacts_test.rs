//! Relocated (M23 T3, D-M23-10) from `crates/report/src/manager/tests.rs`
//! verbatim: `save_extraction_artifacts` is driven with the real C++ plugin
//! extractor, so this test lives outside the guard walk scope
//! (`crates/*/src`).

use ariadne_report::ArtifactManager;
use ariadne_repo::Repository;
use tempfile::tempdir;

#[test]
fn test_save_extraction_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let sym1 = ariadne_transform::ExtractedSymbol {
        name: "MyClass".to_string(),
        kind: ariadne_index::types::SymbolKind::Class,
        body: "// body1".to_string(),
    };
    let header1 = ariadne_transform::GeneratedUnit {
        path: std::path::PathBuf::from("h1.h"),
        symbols: vec![sym1],
        required_deps: vec![],
        primary_source_text: None,
    };

    let result = ariadne_transform::ExtractionResult {
        plan_id: "PLAN-1".to_string(),
        headers: vec![header1],
        errors: vec![],
    };

    let output_dir = mgr.artifact_root.join("generated");
    mgr.save_extraction_artifacts(&result, &output_dir, &ariadne_lang_cpp::CppSourceExtractor)
        .unwrap();

    assert!(output_dir.join("manifest.json").exists());
    assert!(output_dir.join("headers").join("h1.h").exists());
}
