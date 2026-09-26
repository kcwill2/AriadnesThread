use ariadne_repo::{FileOwnership, OptimizationPolicy, Repository};
use ariadne_report::{ArtifactManager, RepositoryManifest};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_m1_repository_discovery_on_basic_fixture() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("../fixtures/basic");

    let repo = Repository::locate(&fixture_dir).expect("Should locate basic C++ test fixture");
    let discovery = repo
        .discover()
        .expect("Repository discovery should succeed");

    assert_eq!(
        discovery.files.len(),
        5,
        "Should discover exactly 5 source/header files"
    );

    let file_paths: Vec<String> = discovery
        .files
        .iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();

    assert!(file_paths.contains(&"include/heavy.h".to_string()));
    assert!(file_paths.contains(&"include/thing.h".to_string()));
    assert!(file_paths.contains(&"include/thing_types.h".to_string()));
    assert!(file_paths.contains(&"src/thing.cpp".to_string()));
    assert!(file_paths.contains(&"tests/consumer.cpp".to_string()));

    // Verify Ownership counts
    assert_eq!(
        *discovery
            .ownership_summary
            .get(&FileOwnership::Repository)
            .unwrap_or(&0),
        5
    );
    assert_eq!(
        *discovery
            .ownership_summary
            .get(&FileOwnership::ThirdParty)
            .unwrap_or(&0),
        0
    );
    assert_eq!(
        *discovery
            .ownership_summary
            .get(&FileOwnership::Toolchain)
            .unwrap_or(&0),
        0
    );
    assert_eq!(
        *discovery
            .ownership_summary
            .get(&FileOwnership::Generated)
            .unwrap_or(&0),
        0
    );

    // Verify Policy counts
    assert_eq!(
        *discovery
            .policy_summary
            .get(&OptimizationPolicy::Optimizable)
            .unwrap_or(&0),
        5
    );
    assert_eq!(
        *discovery
            .policy_summary
            .get(&OptimizationPolicy::Contract)
            .unwrap_or(&0),
        0
    );
    assert_eq!(
        *discovery
            .policy_summary
            .get(&OptimizationPolicy::Frozen)
            .unwrap_or(&0),
        0
    );

    // Verify Artifact Generation
    let temp_workspace =
        tempfile::tempdir().expect("Should create temp directory for artifact test");
    let artifact_mgr =
        ArtifactManager::init(temp_workspace.path()).expect("Should initialize artifact root");
    let artifact_dir = artifact_mgr
        .save_repository_artifacts(&discovery, "m1-test-fingerprint")
        .expect("Should save discovery artifacts");

    let files_json = artifact_dir.join("files.json");
    let files_bin = artifact_dir.join("files.bin");
    let manifest_json = artifact_dir.join("manifest.json");

    assert!(files_json.exists(), "files.json must be created");
    assert!(files_bin.exists(), "files.bin must be created");
    assert!(manifest_json.exists(), "manifest.json must be created");

    // Read and validate manifest JSON
    let manifest_str = fs::read_to_string(&manifest_json).expect("Should read manifest.json");
    let manifest: RepositoryManifest =
        serde_json::from_str(&manifest_str).expect("Should deserialize manifest.json");
    assert_eq!(manifest.total_files, 5);
}

#[test]
fn test_m1_discovery_determinism() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("../fixtures/basic");

    let repo = Repository::locate(&fixture_dir).unwrap();
    let discovery1 = repo.discover().unwrap();
    let discovery2 = repo.discover().unwrap();

    assert_eq!(
        discovery1.files, discovery2.files,
        "Discovery output must be deterministic"
    );
}
