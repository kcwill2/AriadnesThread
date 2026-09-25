use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_m17_dir_nav_markers_in_export() {
    let repo_dir = PathBuf::from("../../tests/fixtures/basic");
    let export_dir = tempdir().unwrap();
    let export_file = export_dir.path().join("export.html");

    let result = ariadne_cli::run_ui(
        Some(repo_dir.clone()),
        0,
        true,
        Some(export_file.clone()),
    )
    .await;

    assert!(result.is_ok(), "run_ui failed for export");

    let content = fs::read_to_string(&export_file).unwrap();
    assert!(content.len() > 1024, "Export file too small");
    assert!(
        content.contains("window.__ARIADNE_DATA__"),
        "Export missing data payload"
    );

    for marker in ["quickOpenPanel", "dirScope", "buildDirTree", "setDirScope"] {
        assert!(
            content.contains(marker),
            "Export missing directory-nav feature marker: {}",
            marker
        );
    }
}
