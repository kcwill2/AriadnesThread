use ariadne_repo::Repository;
use ariadne_report::ArtifactManager;
use std::path::PathBuf;

#[test]
fn test_m0_fixture_location_and_artifacts() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Navigation to tests/fixtures/basic relative to tests/integration
    let fixture_dir = manifest_dir.join("../fixtures/basic");

    let repo = Repository::locate(&fixture_dir).expect("Should locate basic C++ test fixture");
    assert!(repo.root.exists());
    assert!(repo.root.join("include/thing.h").exists());
    assert!(repo.root.join("include/thing_types.h").exists());
    assert!(repo.root.join("include/heavy.h").exists());

    let temp_workspace =
        tempfile::tempdir().expect("Should create temp directory for artifact test");
    let artifact_mgr =
        ArtifactManager::init(temp_workspace.path()).expect("Should initialize artifact root");
    assert!(artifact_mgr.artifact_root.exists());
    assert_eq!(
        artifact_mgr.artifact_root,
        temp_workspace.path().join("artifacts")
    );
}
