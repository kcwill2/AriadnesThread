use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use ariadne_cli::handle_analyze;
use ariadne_scheduler::incremental::IncrementalAnalyzer;
use ariadne_repo::Repository;

fn setup_test_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();
    
    // Create directories
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    
    // Create files
    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(root.join("include/header_a.h"), "#include \"common.h\"\nstruct ThingA { Common c; };").unwrap();
    fs::write(root.join("include/header_b.h"), "struct ThingB { double x; };").unwrap();
    
    fs::write(root.join("src/tu1.cpp"), "#include \"header_a.h\"\nvoid use_a(ThingA a) {}").unwrap();
    fs::write(root.join("src/tu2.cpp"), "#include \"header_b.h\"\nvoid use_b(ThingB b) {}").unwrap();
    
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
        root_str=root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();
    
    temp_dir
}

#[test]
fn test_m12_incremental_analysis_cold_run() {
    let repo_dir = setup_test_repo();
    let root = repo_dir.path().to_path_buf();
    
    // First run (Cold Run)
    let result = handle_analyze(&root).expect("Analyze should succeed");
    
    assert_eq!(result.stats.reused_tus, 0);
    assert_eq!(result.stats.reanalyzed_tus, 2);
    assert_eq!(result.stats.hit_rate, 0.0);
    
    // Check artifacts
    let cache_dir = root.join("artifacts").join("cache");
    assert!(cache_dir.join("manifest.json").exists());
    assert!(cache_dir.join("cache.bin").exists());
    assert!(cache_dir.join("cache.json").exists());
}

#[test]
fn test_m12_incremental_analysis_warm_run() {
    let repo_dir = setup_test_repo();
    let root = repo_dir.path().to_path_buf();
    
    // First run
    handle_analyze(&root).unwrap();
    
    // Second run (Warm run)
    let result = handle_analyze(&root).unwrap();
    
    assert_eq!(result.stats.reused_tus, 2);
    assert_eq!(result.stats.reanalyzed_tus, 0);
    assert_eq!(result.stats.hit_rate, 100.0);
}

#[test]
fn test_m12_incremental_analysis_header_modification() {
    let repo_dir = setup_test_repo();
    let root = repo_dir.path().to_path_buf();
    
    // First run
    handle_analyze(&root).unwrap();
    
    // Modify common.h (used by tu1 via header_a.h, but not tu2)
    let common_h = root.join("include/common.h");
    let mut content = fs::read_to_string(&common_h).unwrap();
    content.push_str("\n// modified\n");
    fs::write(&common_h, content).unwrap();
    
    // Second run
    let result = handle_analyze(&root).unwrap();
    
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
    handle_analyze(&root).unwrap();
    
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
        root_str=root_str
    );
    fs::write(root.join("compile_commands.json"), _compile_commands).unwrap();
    
    // Second run
    let result = handle_analyze(&root).unwrap();
    
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
    let repo = Repository { root: root.to_path_buf() };
    let repo_discovery = repo.discover().unwrap();
    let build_discovery = BuildDiscoverer::new(root).discover().unwrap();
    
    let mut analyzer1 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_discovery);
    let result1 = analyzer1.analyze().unwrap();
    result1.save_artifacts(&repo).unwrap();
    
    // Second run with different version
    let mut analyzer2 = IncrementalAnalyzer::new(&repo, &repo_discovery, &build_discovery)
        .with_analysis_version("2.0.0");
    let result2 = analyzer2.analyze().unwrap();
    
    assert_eq!(result2.stats.reused_tus, 0);
    assert_eq!(result2.stats.reanalyzed_tus, 2);
    assert_eq!(result2.stats.hit_rate, 0.0);
}
