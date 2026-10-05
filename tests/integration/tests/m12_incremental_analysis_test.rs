use std::fs;
use tempfile::TempDir;

use ariadne_cli::run_incremental_analysis;
use ariadne_repo::Repository;
use ariadne_scheduler::incremental::IncrementalAnalyzer;

fn setup_test_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();

    // Create directories
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    // Create files
    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(
        root.join("include/header_a.h"),
        "#include \"common.h\"\nstruct ThingA { Common c; };",
    )
    .unwrap();
    fs::write(
        root.join("include/header_b.h"),
        "struct ThingB { double x; };",
    )
    .unwrap();

    fs::write(
        root.join("src/tu1.cpp"),
        "#include \"header_a.h\"\nvoid use_a(ThingA a) {}",
    )
    .unwrap();
    fs::write(
        root.join("src/tu2.cpp"),
        "#include \"header_b.h\"\nvoid use_b(ThingB b) {}",
    )
    .unwrap();

    let root_str = root.to_string_lossy().replace("\\", "/");
    let compile_commands = format!(
        r#"[
    {{
        "directory": "{root_str}",
        "command": "clang++ -Iinclude -c src/tu1.cpp",
        "file": "src/tu1.cpp"
    }},
    {{
        "directory": "{root_str}",
        "command": "clang++ -Iinclude -c src/tu2.cpp",
        "file": "src/tu2.cpp"
    }}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();

    temp_dir
}

#[test]
fn test_m12_incremental_analysis_cold_run() {
    let repo_dir = setup_test_repo();
    let root = repo_dir.path().to_path_buf();

    // First run (Cold Run)
    let result = run_incremental_analysis(&root).expect("Analyze should succeed");

    assert_eq!(result.stats.reused_tus, 0);
    assert_eq!(result.stats.reanalyzed_tus, 2);
    assert_eq!(result.stats.hit_rate, 0.0);

    // Check artifacts: unified shard store holds one fingerprinted envelope per TU
    let shards_dir = root.join("artifacts").join("shards");
    let shard_count = fs::read_dir(&shards_dir)
        .expect("shard store directory should exist")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .count();
    assert_eq!(shard_count, 2, "one shard per translation unit");

    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("artifacts").join("index").join("manifest.json")).unwrap(),
    )
    .unwrap();
    assert!(
        !manifest["input_fingerprint"]
            .as_str()
            .unwrap_or_default()
            .is_empty(),
        "index manifest must carry an input fingerprint"
    );
    // Pinned epoch (M18.10 bump 1.0.0 -> 2.0.0: symbols.bin bincode layout +
    // fingerprint language/plugin inputs; M27 T2 bump 2.0.0 -> 2.1.0 per user
    // DP-6: invalidate release-built shards carrying silently-wrapped token
    // columns): a bump must fail this assert.
    assert_eq!(manifest["analysis_version"].as_str().unwrap(), "2.1.0");
}

#[test]
fn test_m12_incremental_analysis_warm_run() {
    let repo_dir = setup_test_repo();
    let root = repo_dir.path().to_path_buf();

    // First run
    run_incremental_analysis(&root).unwrap();

    // Second run (Warm run)
    let result = run_incremental_analysis(&root).unwrap();

    assert_eq!(result.stats.reused_tus, 2);
    assert_eq!(result.stats.reanalyzed_tus, 0);
    assert_eq!(result.stats.hit_rate, 100.0);
}

#[test]
fn test_m12_incremental_analysis_header_modification() {
    let repo_dir = setup_test_repo();
    let root = repo_dir.path().to_path_buf();

    // First run
    run_incremental_analysis(&root).unwrap();

    // Modify common.h (used by tu1 via header_a.h, but not tu2)
    let common_h = root.join("include/common.h");
    let mut content = fs::read_to_string(&common_h).unwrap();
    content.push_str("\n// modified\n");
    fs::write(&common_h, content).unwrap();

    // Second run
    let result = run_incremental_analysis(&root).unwrap();

    // tu1 should be reanalyzed, tu2 should be reused
    assert_eq!(result.stats.reanalyzed_tus, 1);
    assert_eq!(result.stats.reused_tus, 1);
    assert_eq!(result.stats.hit_rate, 50.0);
}

#[test]
fn test_m12_incremental_analysis_context_invalidation() {
    let repo_dir = setup_test_repo();
    let root = repo_dir.path().to_path_buf();

    // First run
    run_incremental_analysis(&root).unwrap();

    // Modify compilation context by modifying compile_commands.json
    let root_str = root.to_string_lossy().replace("\\", "/");
    let _compile_commands = format!(
        r#"[
    {{
        "directory": "{root_str}",
        "command": "clang++ -DNEW_DEFINE -Iinclude -c src/tu1.cpp",
        "file": "src/tu1.cpp"
    }},
    {{
        "directory": "{root_str}",
        "command": "clang++ -DNEW_DEFINE -Iinclude -c src/tu2.cpp",
        "file": "src/tu2.cpp"
    }}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), _compile_commands).unwrap();

    // Second run
    let result = run_incremental_analysis(&root).unwrap();

    // Context changed, both should be reanalyzed
    assert_eq!(result.stats.reused_tus, 0);
    assert_eq!(result.stats.reanalyzed_tus, 2);
    assert_eq!(result.stats.hit_rate, 0.0);
}

#[test]
fn test_m12_incremental_analysis_version_mismatch() {
    let repo_dir = setup_test_repo();
    let root = repo_dir.path();

    use ariadne_build::BuildDiscoverer;

    // First run (normal)
    let repo = Repository {
        root: root.to_path_buf(),
    };
    let repo_discovery = repo.discover().unwrap();
    let build_discovery = BuildDiscoverer::new(root).discover().unwrap();

    let cpp_plugin = ariadne_lang_cpp::CppPlugin::new();
    let mut analyzer1 =
        IncrementalAnalyzer::new(&repo, &repo_discovery, &build_discovery, &cpp_plugin);
    let result1 = analyzer1.analyze().unwrap();
    result1.save_artifacts(&repo).unwrap();

    // Second run with a version distinct from CURRENT_ANALYSIS_VERSION
    // ("2.1.0" since the M27 T2 DP-6 epoch bump - itself following the
    // M18.10 1.0.0 -> 2.0.0 bump - so any older sentinel value is stale
    // here; any never-current version keeps the mismatch semantics).
    let mut analyzer2 =
        IncrementalAnalyzer::new(&repo, &repo_discovery, &build_discovery, &cpp_plugin)
            .with_analysis_version("9.9.9");
    let result2 = analyzer2.analyze().unwrap();

    assert_eq!(result2.stats.reused_tus, 0);
    assert_eq!(result2.stats.reanalyzed_tus, 2);
    assert_eq!(result2.stats.hit_rate, 0.0);
}
