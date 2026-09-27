use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult};
use ariadne_cli::discover_build_contexts;
use ariadne_repo::Repository;
use ariadne_report::{ArtifactManager, BuildContextManifest};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_m2_build_discovery_determinism_and_contexts() {
    let dir = tempdir().expect("Failed to create tempdir");
    let root = dir.path();

    write_two_variant_repo_fixture(root);

    let repo = Repository::locate(root).expect("Repository locate failed");
    let repo_disc = repo.discover().expect("Repository discover failed");

    let discoverer = BuildDiscoverer::new(root).with_repo_discovery(&repo_disc);
    let res1 = discoverer.discover().expect("Build discovery 1 failed");

    let discoverer2 = BuildDiscoverer::new(root).with_repo_discovery(&repo_disc);
    let res2 = discoverer2.discover().expect("Build discovery 2 failed");

    assert_deterministic_build_discovery(&res1, &res2);
    assert_debug_and_release_contexts(&res1);
    verify_build_context_artifacts(root, &res1);
}

fn write_two_variant_repo_fixture(root: &Path) {
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("include/thing.h"),
        "#pragma once\nclass Thing {};",
    )
    .unwrap();
    fs::write(
        root.join("src/thing.cpp"),
        "#include \"thing.h\"\nint main() { return 0; }",
    )
    .unwrap();

    let compile_commands_json = format!(
        r#"[
            {{
                "directory": "{}",
                "command": "cl.exe /c /Iinclude /DDEBUG /std:c++17 src/thing.cpp",
                "file": "src/thing.cpp"
            }},
            {{
                "directory": "{}",
                "command": "cl.exe /c /Iinclude /DNDEBUG /std:c++17 src/thing.cpp",
                "file": "src/thing.cpp"
            }}
        ]"#,
        root.display().to_string().replace('\\', "/"),
        root.display().to_string().replace('\\', "/")
    );

    fs::write(root.join("compile_commands.json"), compile_commands_json).unwrap();
}

fn assert_deterministic_build_discovery(res1: &BuildDiscoveryResult, res2: &BuildDiscoveryResult) {
    // Acceptance criterion: Running discovery twice against the same build configuration produces equivalent normalized compilation contexts.
    assert_eq!(
        res1.contexts, res2.contexts,
        "Compilation contexts must be deterministic"
    );
    assert_eq!(
        res1.translation_units, res2.translation_units,
        "Translation units must be deterministic"
    );
}

fn assert_debug_and_release_contexts(res: &BuildDiscoveryResult) {
    assert_eq!(
        res.contexts.len(),
        2,
        "Should discover Debug|x64 and Release|x64 contexts"
    );

    let debug_ctx = res
        .contexts
        .iter()
        .find(|c| c.name == "Debug|x64")
        .expect("Debug|x64 context missing");
    assert!(
        debug_ctx.defines.contains(&"DEBUG".to_string()),
        "Debug context must contain DEBUG define"
    );

    let release_ctx = res
        .contexts
        .iter()
        .find(|c| c.name == "Release|x64")
        .expect("Release|x64 context missing");
    assert!(
        release_ctx.defines.contains(&"NDEBUG".to_string()),
        "Release context must contain NDEBUG define"
    );
}

fn verify_build_context_artifacts(root: &Path, res: &BuildDiscoveryResult) {
    // Test artifact generation
    let artifact_mgr = ArtifactManager::init(root).expect("ArtifactManager init failed");
    let ctx_dir = artifact_mgr
        .save_build_context_artifacts(res, "m2-test-fingerprint")
        .expect("save_build_context_artifacts failed");

    assert!(
        ctx_dir.join("contexts.json").exists(),
        "contexts.json must exist"
    );
    assert!(
        ctx_dir.join("commands.json").exists(),
        "commands.json must exist"
    );
    assert!(
        ctx_dir.join("manifest.json").exists(),
        "manifest.json must exist"
    );

    let manifest_str = fs::read_to_string(ctx_dir.join("manifest.json")).unwrap();
    let manifest: BuildContextManifest = serde_json::from_str(&manifest_str).unwrap();
    assert_eq!(manifest.total_contexts, 2);
    assert_eq!(manifest.total_translation_units, 2);
}

#[test]
fn test_m2_cli_handle_build_discover() {
    let dir = tempdir().expect("Failed to create tempdir");
    let root = dir.path();

    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("include/thing.h"),
        "#pragma once\nclass Thing {};",
    )
    .unwrap();
    fs::write(
        root.join("src/thing.cpp"),
        "#include \"thing.h\"\nint main() { return 0; }",
    )
    .unwrap();

    let compile_commands_json = format!(
        r#"[
            {{
                "directory": "{}",
                "command": "cl.exe /c /Iinclude /DDEBUG src/thing.cpp",
                "file": "src/thing.cpp"
            }}
        ]"#,
        root.display().to_string().replace('\\', "/")
    );

    fs::write(root.join("compile_commands.json"), compile_commands_json).unwrap();

    let res = discover_build_contexts(&root.to_path_buf());
    assert!(res.is_ok(), "handle_build_discover should succeed");

    let artifact_manifest = root.join("artifacts/contexts/manifest.json");
    assert!(
        artifact_manifest.exists(),
        "CLI handle_build_discover must save artifact manifest.json"
    );
}
