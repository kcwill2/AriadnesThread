use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use tempfile::tempdir;
use serde::Deserialize;

use ariadne_cli::handle_index;
use ariadne_index::DependencyKind;

#[derive(Deserialize)]
struct IndexManifest {
    pub total_translation_units: usize,
    pub total_symbols: usize,
    pub total_edges: usize,
    pub complete_edges: usize,
    pub incomplete_edges: usize,
}

#[test]
fn test_m4_semantic_indexing() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");
    let result = handle_index(&repo_path)?;
    
    assert!(result.total_symbols > 0, "Should have found some symbols");
    assert!(result.total_edges > 0, "Should have found some edges");
    
    // Check manifest file
    let artifacts_dir = repo_path.join("artifacts/index");
    assert!(artifacts_dir.join("manifest.json").exists(), "manifest.json should exist");
    assert!(artifacts_dir.join("index.json").exists(), "index.json should exist");
    assert!(artifacts_dir.join("tu_analyses.bin").exists(), "tu_analyses.bin should exist");

    let manifest_content = fs::read_to_string(artifacts_dir.join("manifest.json"))?;
    let manifest: IndexManifest = serde_json::from_str(&manifest_content)?;

    assert_eq!(manifest.total_translation_units, result.tu_analyses.len());
    assert_eq!(manifest.total_symbols, result.total_symbols);
    assert_eq!(manifest.total_edges, result.total_edges);
    assert_eq!(manifest.complete_edges, result.complete_edges);
    assert_eq!(manifest.incomplete_edges, result.incomplete_edges);

    Ok(())
}

#[test]
fn test_m4_semantic_critical_patterns() -> Result<()> {
    let dir = tempdir()?;
    let repo_path = dir.path().to_path_buf();
    
    fs::create_dir_all(repo_path.join("src"))?;
    fs::write(repo_path.join("src/main.cpp"), r#"
        class B1;
        class A1 { B1* ptr; };

        class B2 {};
        class A2 { B2 val; };

        class B3 {};
        class A3 : public B3 {};

        class B4 {};
        class A4 { void f() { sizeof(B4); } };

        class B5;
        class A5 { B5 make_b(); };
    "#)?;

    // We also need a dummy CMakeLists.txt or something so that build context discovery can find it?
    // Let's just create a basic compile_commands.json
    fs::write(repo_path.join("compile_commands.json"), serde_json::json!([
        {
            "directory": repo_path.to_string_lossy().into_owned(),
            "command": "c++ -c src/main.cpp -o src/main.o",
            "file": "src/main.cpp"
        }
    ]).to_string())?;

    let result = handle_index(&repo_path)?;
    assert_eq!(result.tu_analyses.len(), 1);

    let tu = &result.tu_analyses[0];
    
    let get_edge_kind = |src: &str, dst: &str| {
        tu.edges.iter().find(|e| e.src_symbol.0 == src && e.dst_symbol.0 == dst).map(|e| e.kind)
    };

    assert_eq!(get_edge_kind("A1", "B1"), Some(DependencyKind::Incomplete));
    assert_eq!(get_edge_kind("A2", "B2"), Some(DependencyKind::Complete));
    assert_eq!(get_edge_kind("A3", "B3"), Some(DependencyKind::Complete));
    assert_eq!(get_edge_kind("A4", "B4"), Some(DependencyKind::Complete));
    assert_eq!(get_edge_kind("A5", "B5"), Some(DependencyKind::Incomplete));

    Ok(())
}
