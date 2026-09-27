use anyhow::Result;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use ariadne_cli::{
    build_source_index, query_symbol_by_name, query_symbol_dependencies, query_symbol_users,
};
use ariadne_graph::ReducedSemanticModel;

/// The real reduced-stage fingerprint, so this fixture's reduced artifact
/// carries byte-identical content no matter which test writes it last.
fn reduced_fingerprint(repo_path: &std::path::Path) -> Result<String> {
    let repo = ariadne_repo::Repository::locate(repo_path)?;
    let discovery = repo.discover()?;
    let build = ariadne_build::BuildDiscoverer::new(&repo.root)
        .with_repo_discovery(&discovery)
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
struct ReducedManifest {
    pub total_symbols: usize,
    pub total_edges: usize,
    pub total_tu_usages: usize,
    pub total_indexed_files: usize,
}

fn hash_file(path: &PathBuf) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

#[test]
fn test_m5_reduced_model_aggregation_and_queries() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");

    let artifacts_dir = repo_path.join("artifacts/reduced");

    // Explicitly rebuild instead of removing directories to avoid Windows os error 5
    let index_result = build_source_index(&repo_path)?;
    let model = ReducedSemanticModel::reduce(&index_result);

    let repo = ariadne_repo::Repository::locate(&repo_path)?;
    let artifacts = ariadne_report::ArtifactManager::prepare_for_repo(&repo)?;
    let reduced_fp = reduced_fingerprint(&repo_path)?;
    artifacts.save_reduced_artifacts(&model, &reduced_fp)?;

    // 1. Thing exists exactly once in the reduced model
    // Definition points to include/thing.h
    // tu_usage contains both src/thing.cpp and tests/consumer.cpp
    // Dependency edges are deduplicated

    let thing_sym = model
        .find_symbol("Thing")
        .expect("Thing symbol should exist");
    assert_eq!(thing_sym.name, "Thing");

    let def_file = thing_sym
        .defining_file
        .as_ref()
        .expect("Thing should have a defining file");

    // We expect the file ID to match include/thing.h
    let expected_def_id = ariadne_repo::FileId::from_relative_path("include/thing.h");
    assert_eq!(
        *def_file, expected_def_id,
        "Definition should point to include/thing.h"
    );

    let users = model.get_users(&thing_sym.id);
    let mut users_set = HashSet::new();
    for u in users {
        users_set.insert(u.clone());
    }

    let expected_src = ariadne_repo::FileId::from_relative_path("src/thing.cpp");
    let expected_tests = ariadne_repo::FileId::from_relative_path("tests/consumer.cpp");

    assert!(
        users_set.contains(&expected_src),
        "tu_usage should contain src/thing.cpp"
    );
    assert!(
        users_set.contains(&expected_tests),
        "tu_usage should contain tests/consumer.cpp"
    );

    // 2. Query verification
    query_symbol_by_name(&repo_path, "Thing")?;
    query_symbol_dependencies(&repo_path, "Thing")?;
    query_symbol_users(&repo_path, "Thing")?;

    // 3. Artifact verification
    assert!(artifacts_dir.join("symbols.bin").exists());
    assert!(artifacts_dir.join("edges.bin").exists());
    assert!(artifacts_dir.join("tu-usage.bin").exists());
    assert!(artifacts_dir.join("file-symbols.bin").exists());
    assert!(artifacts_dir.join("manifest.json").exists());

    let manifest_content = fs::read_to_string(artifacts_dir.join("manifest.json"))?;
    let manifest: ReducedManifest = serde_json::from_str(&manifest_content)?;

    assert_eq!(manifest.total_symbols, model.symbols.len());
    assert_eq!(manifest.total_edges, model.edges.len());
    assert_eq!(manifest.total_tu_usages, model.tu_usage.len());
    assert_eq!(manifest.total_indexed_files, model.file_symbols.len());

    Ok(())
}

#[test]
fn test_m5_reduced_model_determinism() -> Result<()> {
    let repo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/basic");
    let artifacts_dir = repo_path.join("artifacts/reduced");

    let run_and_hash = || -> Result<Vec<String>> {
        let index_result = build_source_index(&repo_path)?;
        let model = ReducedSemanticModel::reduce(&index_result);

        let repo = ariadne_repo::Repository::locate(&repo_path)?;
        let artifacts = ariadne_report::ArtifactManager::prepare_for_repo(&repo)?;
        let reduced_fp = reduced_fingerprint(&repo_path)?;
        artifacts.save_reduced_artifacts(&model, &reduced_fp)?;

        let mut hashes = Vec::new();
        for file in &[
            "symbols.bin",
            "edges.bin",
            "tu-usage.bin",
            "file-symbols.bin",
            "manifest.json",
        ] {
            hashes.push(hash_file(&artifacts_dir.join(file))?);
        }
        Ok(hashes)
    };

    let hashes1 = run_and_hash()?;
    let hashes2 = run_and_hash()?;

    assert_eq!(
        hashes1, hashes2,
        "Hashes should match identically between runs"
    );

    Ok(())
}
