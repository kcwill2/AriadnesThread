use super::*;
use crate::manifests::{LoadReport, MigrationManifest};
use ariadne_build::{BuildDiscoveryResult, CompilationContext, CompileCommand, TranslationUnit};
use ariadne_repo::FileId;
use tempfile::tempdir;

pub(super) const FP: &str = "test-fingerprint";

#[test]
fn test_artifact_manager_creation() {
    let dir = tempdir().unwrap();
    let repo = Repository::locate(dir.path()).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();
    assert!(mgr.artifact_root.exists());
}

#[test]
fn test_write_atomic_replaces_existing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("thing.bin");
    fs::write(&path, b"old").unwrap();
    write_atomic(&path, b"new").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"new");
    assert!(!path.with_extension("bin.tmp").exists());
}

#[test]
fn test_save_repository_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("include")).unwrap();
    fs::write(root.join("include/test.h"), "#pragma once").unwrap();

    let repo = Repository::locate(root).unwrap();
    let discovery = repo.discover().unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let repo_dir = mgr.save_repository_artifacts(&discovery, FP).unwrap();
    assert!(repo_dir.join("files.json").exists());
    assert!(repo_dir.join("files.bin").exists());
    assert!(repo_dir.join("manifest.json").exists());

    let loaded = mgr.load_repository_checked(FP).unwrap();
    assert_eq!(loaded.files.len(), discovery.files.len());
}

#[test]
fn test_load_repository_checked_rejects_mismatch() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("include")).unwrap();
    fs::write(root.join("include/test.h"), "#pragma once").unwrap();

    let repo = Repository::locate(root).unwrap();
    let discovery = repo.discover().unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();
    mgr.save_repository_artifacts(&discovery, FP).unwrap();

    let err = mgr
        .load_repository_checked("other-fingerprint")
        .unwrap_err();
    assert!(
        is_stale(&err),
        "fingerprint mismatch must report StaleArtifact: {}",
        err
    );
}

#[test]
fn test_save_build_context_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let mut ctx = CompilationContext::new("Debug|x64", "cl.exe");
    ctx.defines.push("DEBUG".to_string());

    let cmd = CompileCommand {
        directory: root.to_path_buf(),
        file: root.join("src/main.cpp"),
        command: Some("cl.exe /c /DDEBUG src/main.cpp".into()),
        arguments: vec![
            "cl.exe".into(),
            "/c".into(),
            "/DDEBUG".into(),
            "src/main.cpp".into(),
        ],
        context_name: "Debug|x64".into(),
        output: None,
    };

    let tu = TranslationUnit {
        file_id: FileId::from_relative_path("src/main.cpp"),
        source_path: root.join("src/main.cpp"),
        context_name: "Debug|x64".into(),
        compile_command: cmd,
        language: Default::default(),
    };

    let build_result = BuildDiscoveryResult {
        repo_root: root.to_path_buf(),
        contexts: vec![ctx],
        translation_units: vec![tu],
    };

    let ctx_dir = mgr.save_build_context_artifacts(&build_result, FP).unwrap();
    assert!(ctx_dir.join("contexts.json").exists());
    assert!(ctx_dir.join("commands.json").exists());
    assert!(ctx_dir.join("manifest.json").exists());

    let loaded = mgr.load_build_context_checked(FP).unwrap();
    assert_eq!(loaded.contexts.len(), 1);
    assert_eq!(loaded.translation_units.len(), 1);
    assert_eq!(
        loaded.translation_units[0].source_path,
        root.join("src/main.cpp")
    );
}

#[test]
fn test_save_include_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let graph = ariadne_graph::IncludeGraph::new();
    let closures = graph.compute_closures();
    let report = graph.generate_report();

    let inc_dir = mgr
        .save_include_artifacts(&graph, &closures, &report, FP)
        .unwrap();
    assert!(inc_dir.join("edges.bin").exists());
    assert!(inc_dir.join("closures.bin").exists());
    assert!(inc_dir.join("report.json").exists());
    assert!(inc_dir.join("manifest.json").exists());

    let discovery = repo.discover().unwrap();
    let (loaded, loaded_closures) = mgr.load_include_checked(FP, &discovery).unwrap();
    assert_eq!(loaded.edges.len(), graph.edges.len());
    assert_eq!(loaded_closures.len(), closures.len());
}

#[test]
fn test_save_index_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let index_result = ariadne_index::IndexResult::new(repo.root.clone(), vec![]);

    let index_dir = mgr.save_index_artifacts(&index_result, FP, None).unwrap();
    assert!(index_dir.join("tu_analyses.bin").exists());
    assert!(index_dir.join("index.json").exists());
    assert!(index_dir.join("manifest.json").exists());

    let loaded = mgr.load_index_checked(FP).unwrap();
    assert_eq!(loaded.repo_root, repo.root);
}

#[test]
fn test_save_reduced_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let index_result = ariadne_index::IndexResult::new(repo.root.clone(), vec![]);
    let reduced = ariadne_graph::ReducedSemanticModel::reduce(&index_result);

    let reduced_dir = mgr.save_reduced_artifacts(&reduced, FP).unwrap();
    assert!(reduced_dir.join("symbols.bin").exists());
    assert!(reduced_dir.join("edges.bin").exists());
    assert!(reduced_dir.join("tu-usage.bin").exists());
    assert!(reduced_dir.join("file-symbols.bin").exists());
    assert!(reduced_dir.join("manifest.json").exists());
}

#[test]
fn test_load_reduced_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let index_result = ariadne_index::IndexResult::new(repo.root.clone(), vec![]);
    let reduced = ariadne_graph::ReducedSemanticModel::reduce(&index_result);

    mgr.save_reduced_artifacts(&reduced, FP).unwrap();

    let loaded = mgr.load_reduced_artifacts().unwrap();
    assert_eq!(reduced.symbols, loaded.symbols);
    assert_eq!(reduced.edges, loaded.edges);
    assert_eq!(reduced.tu_usage, loaded.tu_usage);
    assert_eq!(reduced.file_symbols, loaded.file_symbols);

    let checked = mgr.load_reduced_checked(FP).unwrap();
    assert_eq!(reduced.symbols, checked.symbols);
}

#[test]
fn test_fingerprintless_manifest_is_stale() {
    // Artifacts written before fingerprints existed carry no input
    // fingerprint and must never be trusted by checked loaders.
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let reduced_dir = mgr.artifact_root.join("reduced");
    fs::create_dir_all(&reduced_dir).unwrap();
    fs::write(
        reduced_dir.join("manifest.json"),
        serde_json::to_string(&serde_json::json!({
            "repository_root": repo.root,
            "total_symbols": 0,
            "total_edges": 0,
            "total_tu_usages": 0,
            "total_indexed_files": 0
        }))
        .unwrap(),
    )
    .unwrap();

    let err = mgr.load_reduced_checked("whatever").unwrap_err();
    assert!(is_stale(&err));
}

#[test]
fn stale_analysis_epoch_is_rejected_before_blob_read() {
    // §7 risk row + §9 M18.10: `symbols.bin` is bincode, so a v1-era blob must
    // NEVER reach the new struct — the manifest epoch gate must fire first.
    // Scope honesty: constructing a byte-authentic v1 `symbols.bin` requires
    // the pre-M18.9 `ReducedSymbolRecord` layout (changed by the parallel M18.9
    // task), so this test plants an intentionally undecodable blob and locks
    // the ORDER of the checks: a stale epoch yields a StaleArtifact (rebuild
    // trigger), never a bincode/serde error.
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let index_result = ariadne_index::IndexResult::new(repo.root.clone(), vec![]);
    let reduced = ariadne_graph::ReducedSemanticModel::reduce(&index_result);
    mgr.save_reduced_artifacts(&reduced, FP).unwrap();

    let reduced_dir = mgr.artifact_root.join("reduced");
    fs::write(reduced_dir.join("symbols.bin"), b"definitely-not-v1-bincode").unwrap();
    let manifest_path = reduced_dir.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["analysis_version"] = serde_json::Value::String("1.0.0".to_string());
    fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();

    let err = mgr.load_reduced_checked(FP).unwrap_err();
    assert!(
        is_stale(&err),
        "stale epoch must report StaleArtifact (rebuild trigger): {err}"
    );
    assert!(
        err.to_string().contains("analysis version mismatch"),
        "error must name the epoch gate: {err}"
    );
    assert!(
        !err.to_string().contains("eserialize"),
        "blob must never be deserialized once the epoch is stale: {err}"
    );
    assert!(!mgr.stage_is_fresh("reduced", FP));
}

#[test]
fn test_save_and_load_load_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let report = LoadReport {
        repository_root: repo.root.clone(),
        symbol_metrics: vec![ariadne_graph::LoadMetric {
            target_name: "TestSymbol".to_string(),
            target_file: Some(PathBuf::from("test.cpp")),
            semantic_load: 10,
            include_load: 20,
            collateral_load: 10,
            total_load: 20,
            semantic_symbols: vec![],
            collateral_symbols: vec![],
            collateral_sources: vec![],
            largest_collateral_source: None,
            semantic_sources: vec![],
        }],
        file_metrics: vec![],
    };

    let load_dir = mgr.save_load_artifacts(&report, FP).unwrap();
    assert!(load_dir.join("report.json").exists());
    assert!(load_dir.join("report.bin").exists());
    assert!(load_dir.join("manifest.json").exists());

    let loaded = mgr.load_load_checked(FP).unwrap();
    assert_eq!(report.symbol_metrics.len(), loaded.symbol_metrics.len());
    assert_eq!(
        report.symbol_metrics[0].target_name,
        loaded.symbol_metrics[0].target_name
    );
}

#[test]
fn test_save_and_load_plan_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let plan = ariadne_cluster::plan::OptimizationPlan {
        id: ariadne_cluster::plan::PlanId("test-plan".to_string()),
        target_file: PathBuf::from("test.cpp"),
        target_file_id: FileId::from_relative_path("test.cpp"),
        problem: "too big".to_string(),
        collateral_load_before: 100,
        collateral_load_after: 58,
        proposed_extractions: vec![],
        optimization_frontier: vec![],
        risk: ariadne_cluster::plan::RiskLevel::Conservative,
        validation_required_tus: vec![],
        rationale: "why not".to_string(),
    };

    let plans = vec![plan.clone()];
    let plan_dir = mgr.save_plan_artifacts(&repo.root, &plans, FP).unwrap();

    assert!(plan_dir.join("plans.json").exists());
    assert!(plan_dir.join("plans.bin").exists());
    assert!(plan_dir.join("manifest.json").exists());

    let loaded = mgr.load_plan_checked(FP).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].collateral_load_before, 100);
    assert_eq!(loaded[0].collateral_load_after, 58);
}

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

#[test]
fn test_save_migration_artifacts() {
    use ariadne_transform::{ConsumerRewrite, MigrationPreview};
    use std::path::PathBuf;

    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let preview = MigrationPreview {
        plan_id: "PLAN-001".to_string(),
        rewrites: vec![ConsumerRewrite {
            consumer_file: PathBuf::from("src/foo.cpp"),
            old_includes: vec![PathBuf::from("thing.h")],
            new_includes: vec![PathBuf::from("thing_types.h")],
            side_effect_preservations: vec![],
        }],
    };

    let migration_dir = mgr.save_migration_artifacts(&preview).unwrap();
    assert!(migration_dir.join("preview.bin").exists());
    assert!(migration_dir.join("preview.json").exists());
    assert!(migration_dir.join("manifest.json").exists());
}

#[test]
fn test_load_migration_artifacts_roundtrip() {
    use ariadne_transform::{ConsumerRewrite, MigrationPreview};
    use std::path::PathBuf;

    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let preview = MigrationPreview {
        plan_id: "PLAN-042".to_string(),
        rewrites: vec![
            ConsumerRewrite {
                consumer_file: PathBuf::from("src/bar.cpp"),
                old_includes: vec![PathBuf::from("thing.h")],
                new_includes: vec![
                    PathBuf::from("thing_core.h"),
                    PathBuf::from("thing_types.h"),
                ],
                side_effect_preservations: vec![],
            },
            ConsumerRewrite {
                consumer_file: PathBuf::from("src/init.cpp"),
                old_includes: vec![PathBuf::from("thing.h"), PathBuf::from("registry.h")],
                new_includes: vec![PathBuf::from("thing_types.h")],
                side_effect_preservations: vec!["registry.h".into()],
            },
        ],
    };

    mgr.save_migration_artifacts(&preview).unwrap();
    let loaded = mgr.load_migration_artifacts().unwrap();

    assert_eq!(loaded.plan_id, "PLAN-042");
    assert_eq!(loaded.rewrites.len(), 2);
    assert_eq!(
        loaded.rewrites[0].consumer_file,
        PathBuf::from("src/bar.cpp")
    );
    assert_eq!(
        loaded.rewrites[1].side_effect_preservations,
        vec![PathBuf::from("registry.h")]
    );
}

#[test]
fn test_migration_manifest_content() {
    use ariadne_transform::{ConsumerRewrite, MigrationPreview};
    use std::path::PathBuf;

    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let preview = MigrationPreview {
        plan_id: "PLAN-007".to_string(),
        rewrites: vec![
            ConsumerRewrite {
                consumer_file: PathBuf::from("src/a.cpp"),
                old_includes: vec![PathBuf::from("thing.h")],
                new_includes: vec![PathBuf::from("thing_types.h")],
                side_effect_preservations: vec![],
            },
            ConsumerRewrite {
                // noop rewrite
                consumer_file: PathBuf::from("src/b.cpp"),
                old_includes: vec![PathBuf::from("thing.h")],
                new_includes: vec![PathBuf::from("thing.h")],
                side_effect_preservations: vec![],
            },
        ],
    };

    let migration_dir = mgr.save_migration_artifacts(&preview).unwrap();
    let manifest_content = std::fs::read_to_string(migration_dir.join("manifest.json")).unwrap();
    let manifest: MigrationManifest = serde_json::from_str(&manifest_content).unwrap();

    assert_eq!(manifest.plan_id, "PLAN-007");
    assert_eq!(manifest.total_rewrites, 2);
    assert_eq!(manifest.effective_rewrites, 1); // only a.cpp changes
}

#[test]
fn test_save_transaction_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let outcome = ariadne_validate::ApplyOutcome::Committed {
        tiers_passed: vec![ariadne_validate::TierResult {
            tier: ariadne_validate::ValidationTier::Structural,
            status: ariadne_validate::ValidationStatus::Pass,
        }],
    };

    let tx_dir = mgr.save_transaction_artifacts("PLAN-TX", &outcome).unwrap();
    assert!(tx_dir.join("manifest.json").exists());
    assert!(tx_dir.join("outcome.json").exists());

    let manifest_content = fs::read_to_string(tx_dir.join("manifest.json")).unwrap();
    assert!(manifest_content.contains("PLAN-TX"));
    assert!(manifest_content.contains("Committed"));
}

#[test]
fn test_save_and_load_measurement_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let measurement = ariadne_build::BuildMeasurement {
        repo_root: root.to_path_buf(),
        timestamp: 12345,
        full_build_wall_time: std::time::Duration::from_millis(500),
        total_frontend_cpu_time: std::time::Duration::from_millis(1500),
        total_preprocessing_time: std::time::Duration::from_millis(200),
        peak_memory_bytes: 1024,
        tu_measurements: vec![],
        collateral_load: Some(10),
        affected_tus_count: Some(5),
    };

    let measure_dir = mgr.save_measurement_artifacts(&measurement).unwrap();
    assert!(measure_dir.join("measurements.json").exists());
    assert!(measure_dir.join("measurements.bin").exists());
    assert!(measure_dir.join("manifest.json").exists());

    let loaded = mgr.load_measurement_artifacts().unwrap();
    assert_eq!(measurement.timestamp, loaded.timestamp);
    assert_eq!(measurement.peak_memory_bytes, loaded.peak_memory_bytes);

    // Test load_measurement_from_path with directory
    let loaded_from_dir = ArtifactManager::load_measurement_from_path(&measure_dir).unwrap();
    assert_eq!(measurement.timestamp, loaded_from_dir.timestamp);

    // Test load_measurement_from_path with json file
    let json_path = measure_dir.join("measurements.json");
    let loaded_from_json = ArtifactManager::load_measurement_from_path(&json_path).unwrap();
    assert_eq!(measurement.timestamp, loaded_from_json.timestamp);

    // Test load_measurement_from_path with bin file
    let bin_path = measure_dir.join("measurements.bin");
    let loaded_from_bin = ArtifactManager::load_measurement_from_path(&bin_path).unwrap();
    assert_eq!(measurement.timestamp, loaded_from_bin.timestamp);
}

#[test]
fn test_save_comparison_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let before = ariadne_build::BuildMeasurement {
        repo_root: root.to_path_buf(),
        timestamp: 12345,
        full_build_wall_time: std::time::Duration::from_millis(500),
        total_frontend_cpu_time: std::time::Duration::from_millis(1500),
        total_preprocessing_time: std::time::Duration::from_millis(200),
        peak_memory_bytes: 1024,
        tu_measurements: vec![],
        collateral_load: Some(10),
        affected_tus_count: Some(5),
    };

    let after = ariadne_build::BuildMeasurement {
        repo_root: root.to_path_buf(),
        timestamp: 12346,
        full_build_wall_time: std::time::Duration::from_millis(250),
        total_frontend_cpu_time: std::time::Duration::from_millis(750),
        total_preprocessing_time: std::time::Duration::from_millis(100),
        peak_memory_bytes: 512,
        tu_measurements: vec![],
        collateral_load: Some(5),
        affected_tus_count: Some(5),
    };

    let comparison = ariadne_build::MeasurementComparison::compare(before, after);
    let measure_dir = mgr.save_comparison_artifacts(&comparison).unwrap();
    assert!(measure_dir.join("comparison.json").exists());
    assert!(measure_dir.join("report.txt").exists());
    let report_content = fs::read_to_string(measure_dir.join("report.txt")).unwrap();
    assert!(!report_content.is_empty());
}

#[test]
fn test_save_load_callgraph_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let repo = Repository::locate(root).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let mut callgraph = ariadne_graph::CallGraph::default();
    let f1 = FileId::from_relative_path("main.cpp");
    let s1 = ariadne_index::types::SymbolId::new("A");
    callgraph.nodes.push(ariadne_graph::CallNode {
        symbol_id: s1.clone(),
        name: "A".into(),
        owning_file: f1.clone(),
        location: None,
    });

    let callgraph_dir = mgr.save_callgraph_artifacts(&callgraph, FP).unwrap();
    assert!(callgraph_dir.join("callgraph.json").exists());
    assert!(callgraph_dir.join("callgraph.bin").exists());
    assert!(callgraph_dir.join("manifest.json").exists());

    let loaded = mgr.load_callgraph_checked(FP).unwrap();
    assert_eq!(loaded.nodes.len(), 1);
    assert_eq!(loaded.nodes[0].name, "A");
}

// --- Task E1: load-artifact byte-stability lock (I2) ---------------------

const LOCK_REPORT_JSON_HASH: u64 = 0x70c9_1d07_9946_f7d9;
const LOCK_REPORT_BIN_HASH: u64 = 0x7a12_1e91_0cbf_2e00;
// Resynced for the M18.10 cache-epoch bump ("1.0.0" → "2.0.0"): the load
// manifest embeds `analysis_version`, so its bytes legitimately shift while
// the report json/bin locks stay pinned (they carry no version).
const LOCK_MANIFEST_JSON_HASH: u64 = 0xddc3_f9ee_566b_5ea6;

fn fnv1a64_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn fixture_contributing_source(
    header: &str,
    qualified_prefix: &str,
    symbol_names: &[&str],
) -> ariadne_graph::ContributingSource {
    ariadne_graph::ContributingSource {
        file_id: FileId::from_relative_path(header),
        path: PathBuf::from(header),
        symbol_count: symbol_names.len(),
        symbols: symbol_names
            .iter()
            .map(|name| ariadne_index::types::SymbolId::new(format!("{qualified_prefix}{name}")))
            .collect(),
    }
}

pub(super) fn fixture_load_report() -> LoadReport {
    let alpha_source =
        fixture_contributing_source("include/alpha.h", "Alpha::", &["value", "helper", "state"]);
    let beta_source = fixture_contributing_source("include/beta.hpp", "Beta::", &["edge"]);
    LoadReport {
        repository_root: PathBuf::from("fixture/repo"),
        symbol_metrics: vec![
            ariadne_graph::LoadMetric {
                target_name: "AlphaStore".to_string(),
                target_file: Some(PathBuf::from("src/alpha_store.cpp")),
                semantic_load: 3,
                include_load: 5,
                collateral_load: 4,
                total_load: 9,
                semantic_symbols: vec![
                    ariadne_index::types::SymbolId::new("AlphaStore"),
                    ariadne_index::types::SymbolId::new("AlphaStore::flush"),
                    ariadne_index::types::SymbolId::new("AlphaStore::read"),
                ],
                collateral_symbols: vec![
                    ariadne_index::types::SymbolId::new("Alpha::value"),
                    ariadne_index::types::SymbolId::new("Alpha::helper"),
                    ariadne_index::types::SymbolId::new("Alpha::state"),
                    ariadne_index::types::SymbolId::new("Beta::edge"),
                ],
                collateral_sources: vec![alpha_source.clone(), beta_source.clone()],
                largest_collateral_source: Some(alpha_source.clone()),
                semantic_sources: vec![alpha_source],
            },
            ariadne_graph::LoadMetric {
                target_name: "BetaView".to_string(),
                target_file: Some(PathBuf::from("src/beta_view.cpp")),
                semantic_load: 1,
                include_load: 1,
                collateral_load: 0,
                total_load: 1,
                semantic_symbols: vec![ariadne_index::types::SymbolId::new("BetaView")],
                collateral_symbols: vec![],
                collateral_sources: vec![],
                largest_collateral_source: None,
                semantic_sources: vec![fixture_contributing_source(
                    "include/beta.hpp",
                    "Beta::",
                    &["view"],
                )],
            },
        ],
        file_metrics: vec![ariadne_graph::LoadMetric {
            target_name: "include/alpha.h".to_string(),
            target_file: Some(PathBuf::from("include/alpha.h")),
            semantic_load: 3,
            include_load: 2,
            collateral_load: 1,
            total_load: 5,
            semantic_symbols: vec![ariadne_index::types::SymbolId::new("Alpha::value")],
            collateral_symbols: vec![ariadne_index::types::SymbolId::new("Beta::edge")],
            collateral_sources: vec![beta_source],
            largest_collateral_source: None,
            semantic_sources: vec![],
        }],
    }
}

fn load_artifact_hashes(load_dir: &Path) -> (u64, u64, u64) {
    (
        fnv1a64_hash(&fs::read(load_dir.join("report.json")).unwrap()),
        fnv1a64_hash(&fs::read(load_dir.join("report.bin")).unwrap()),
        fnv1a64_hash(&fs::read(load_dir.join("manifest.json")).unwrap()),
    )
}

pub(super) fn manager_in_fresh_tempdir(dir: &tempfile::TempDir) -> ArtifactManager {
    let repo = Repository::locate(dir.path()).unwrap();
    ArtifactManager::prepare_for_repo(&repo).unwrap()
}

#[test]
fn load_artifact_bytes_are_byte_locked() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);

    let load_dir = mgr.save_load_artifacts(&fixture_load_report(), FP).unwrap();
    let (json_hash, bin_hash, manifest_hash) = load_artifact_hashes(&load_dir);

    assert_eq!(
        (json_hash, bin_hash, manifest_hash),
        (
            LOCK_REPORT_JSON_HASH,
            LOCK_REPORT_BIN_HASH,
            LOCK_MANIFEST_JSON_HASH
        ),
        "I2 violation: artifact bytes changed (json={json_hash:#018x} bin={bin_hash:#018x} manifest={manifest_hash:#018x})"
    );
}

#[test]
fn load_artifact_bytes_independent_of_threading() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);

    let first = load_artifact_hashes(&mgr.save_load_artifacts(&fixture_load_report(), FP).unwrap());
    let second =
        load_artifact_hashes(&mgr.save_load_artifacts(&fixture_load_report(), FP).unwrap());
    assert_eq!(first, second, "consecutive saves must be byte-identical");

    let single_thread = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    let single_dir = tempdir().unwrap();
    let single_mgr = manager_in_fresh_tempdir(&single_dir);
    let single = single_thread.install(|| {
        load_artifact_hashes(
            &single_mgr
                .save_load_artifacts(&fixture_load_report(), FP)
                .unwrap(),
        )
    });
    assert_eq!(first, single, "1-thread rayon pool must match direct call");

    let multi_thread = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    let multi_dir = tempdir().unwrap();
    let multi_mgr = manager_in_fresh_tempdir(&multi_dir);
    let multi = multi_thread.install(|| {
        load_artifact_hashes(
            &multi_mgr
                .save_load_artifacts(&fixture_load_report(), FP)
                .unwrap(),
        )
    });
    assert_eq!(first, multi, "4-thread rayon pool must match direct call");
}
