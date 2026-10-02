//! M22.2a (D25): additive manifest `languages` fields — serde-shape lock and
//! builder coverage for the four stage manifests fed by the write paths.

use super::tests::{fixture_load_report, manager_in_fresh_tempdir, FP};
use super::ArtifactManager;
use crate::manifests::RepositoryManifest;
use ariadne_build::{BuildDiscoveryResult, CompilationContext, CompileCommand, TranslationUnit};
use ariadne_repo::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, Repository,
    RepositoryDiscoveryResult, SourceLanguage,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn fixture_file_record(rel: &str, language: SourceLanguage) -> FileRecord {
    let path = PathBuf::from(rel);
    FileRecord {
        id: FileId::from_relative_path(rel),
        canonical_path: path.clone(),
        path,
        kind: FileKind::Source,
        size: 0,
        content_hash: String::new(),
        ownership: FileOwnership::Repository,
        optimization_policy: OptimizationPolicy::Optimizable,
        language,
    }
}

fn fixture_discovery(root: &Path, languages: &[SourceLanguage]) -> RepositoryDiscoveryResult {
    RepositoryDiscoveryResult {
        root: root.to_path_buf(),
        files: languages
            .iter()
            .enumerate()
            .map(|(i, lang)| fixture_file_record(&format!("src/unit_{i}.cc"), *lang))
            .collect(),
        ownership_summary: HashMap::new(),
        policy_summary: HashMap::new(),
    }
}

fn read_manifest_json(dir: &Path) -> String {
    fs::read_to_string(dir.join("manifest.json")).unwrap()
}

fn manifest_languages(json: &str) -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(json).unwrap()["languages"].clone()
}

#[test]
fn m22_manifest_serde_shape_lock_key_omitted_when_empty() {
    let manifest = RepositoryManifest {
        repository_root: PathBuf::from("fixture/repo"),
        total_files: 0,
        ownership_summary: HashMap::new(),
        policy_summary: HashMap::new(),
        input_fingerprint: "fp".to_string(),
        analysis_version: "v".to_string(),
        languages: vec![],
    };
    let json = serde_json::to_string(&manifest).unwrap();
    assert!(
        !json.contains("languages"),
        "empty languages must not emit the key (m00 byte lock): {json}"
    );
}

#[test]
fn m22_manifest_serde_shape_lock_key_serialized_when_present() {
    let manifest = RepositoryManifest {
        repository_root: PathBuf::from("fixture/repo"),
        total_files: 0,
        ownership_summary: HashMap::new(),
        policy_summary: HashMap::new(),
        input_fingerprint: "fp".to_string(),
        analysis_version: "v".to_string(),
        languages: vec![SourceLanguage::Scala],
    };
    let json = serde_json::to_string(&manifest).unwrap();
    assert!(
        json.contains("\"languages\":[\"Scala\"]"),
        "present languages must serialize as the variant name: {json}"
    );
}

#[test]
fn m22_manifest_serde_shape_lock_missing_key_deserializes_empty() {
    let pre_m22 = r#"{
        "repository_root": "fixture/repo",
        "total_files": 3,
        "ownership_summary": {},
        "policy_summary": {},
        "input_fingerprint": "fp",
        "analysis_version": "v"
    }"#;
    let manifest: RepositoryManifest = serde_json::from_str(pre_m22).unwrap();
    assert!(
        manifest.languages.is_empty(),
        "pre-M22 JSON without the key must read back empty"
    );
}

#[test]
fn m22_repository_manifest_collects_deduped_non_default_languages() {
    let dir = tempdir().unwrap();
    let repo = Repository::locate(dir.path()).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();
    let discovery = fixture_discovery(
        dir.path(),
        &[
            SourceLanguage::Scala,
            SourceLanguage::Cpp,
            SourceLanguage::Scala,
        ],
    );

    let repo_dir = mgr.save_repository_artifacts(&discovery, FP).unwrap();
    let json = read_manifest_json(&repo_dir);
    assert_eq!(
        manifest_languages(&json),
        serde_json::json!([SourceLanguage::Scala]),
        "default-compare filter + dedupe expected"
    );
}

#[test]
fn m22_repository_manifest_skips_languages_when_all_default() {
    let dir = tempdir().unwrap();
    let repo = Repository::locate(dir.path()).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();
    let discovery = fixture_discovery(
        dir.path(),
        &[SourceLanguage::Cpp, SourceLanguage::Cpp],
    );

    let repo_dir = mgr.save_repository_artifacts(&discovery, FP).unwrap();
    let json = read_manifest_json(&repo_dir);
    assert!(
        !json.contains("languages"),
        "all-default languages must skip the key: {json}"
    );
}

fn fixture_build_result(root: &Path, languages: &[SourceLanguage]) -> BuildDiscoveryResult {
    let mut contexts = Vec::new();
    let mut translation_units = Vec::new();
    for (i, lang) in languages.iter().enumerate() {
        let name = format!("ctx_{i}");
        let mut ctx = CompilationContext::new(name.clone(), "cc");
        ctx.language = *lang;
        contexts.push(ctx);
        let file = format!("src/unit_{i}.cc");
        translation_units.push(TranslationUnit {
            file_id: FileId::from_relative_path(&file),
            source_path: root.join(&file),
            context_name: name.clone(),
            compile_command: CompileCommand {
                directory: root.to_path_buf(),
                file: root.join(&file),
                command: None,
                arguments: vec![],
                context_name: name,
                output: None,
            },
            language: *lang,
        });
    }
    BuildDiscoveryResult {
        repo_root: root.to_path_buf(),
        contexts,
        translation_units,
    }
}

#[test]
fn m22_build_context_manifest_collects_deduped_non_default_languages() {
    let dir = tempdir().unwrap();
    let repo = Repository::locate(dir.path()).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();
    let build = fixture_build_result(
        dir.path(),
        &[
            SourceLanguage::Cpp,
            SourceLanguage::Scala,
            SourceLanguage::Scala,
        ],
    );

    let ctx_dir = mgr.save_build_context_artifacts(&build, FP).unwrap();
    let json = read_manifest_json(&ctx_dir);
    assert_eq!(
        manifest_languages(&json),
        serde_json::json!([SourceLanguage::Scala])
    );
}

#[test]
fn m22_build_context_manifest_skips_languages_when_all_default() {
    let dir = tempdir().unwrap();
    let repo = Repository::locate(dir.path()).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();
    let build = fixture_build_result(dir.path(), &[SourceLanguage::Cpp]);

    let ctx_dir = mgr.save_build_context_artifacts(&build, FP).unwrap();
    let json = read_manifest_json(&ctx_dir);
    assert!(!json.contains("languages"), "all-default must skip: {json}");
}

fn fixture_include_edge(
    source: &str,
    destination: &str,
    language: SourceLanguage,
) -> ariadne_graph::IncludeEdge {
    ariadne_graph::IncludeEdge {
        source: FileId::from_relative_path(source),
        destination: Some(FileId::from_relative_path(destination)),
        context_name: "ctx".to_string(),
        source_location: ariadne_graph::SourceLocation::new(1, 1),
        kind: ariadne_graph::IncludeKind::Local,
        raw_path: destination.to_string(),
        resolved_path: Some(PathBuf::from(destination)),
        language,
    }
}

#[test]
fn m22_include_manifest_collects_deduped_non_default_languages() {
    let dir = tempdir().unwrap();
    let repo = Repository::locate(dir.path()).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let mut graph = ariadne_graph::IncludeGraph::new();
    for rel in ["a.cc", "b.hh", "s.scala", "o.scala"] {
        graph.register_file(FileId::from_relative_path(rel), PathBuf::from(rel));
    }
    graph.add_edge(fixture_include_edge(
        "a.cc",
        "b.hh",
        SourceLanguage::Cpp,
    ));
    graph.add_edge(fixture_include_edge(
        "s.scala",
        "o.scala",
        SourceLanguage::Scala,
    ));
    graph.add_edge(fixture_include_edge(
        "o.scala",
        "s.scala",
        SourceLanguage::Scala,
    ));
    let closures = graph.compute_closures();
    let report = graph.generate_report();

    let inc_dir = mgr
        .save_include_artifacts(&graph, &closures, &report, FP)
        .unwrap();
    let json = read_manifest_json(&inc_dir);
    assert_eq!(
        manifest_languages(&json),
        serde_json::json!([SourceLanguage::Scala])
    );
}

#[test]
fn m22_include_manifest_skips_languages_when_all_default() {
    let dir = tempdir().unwrap();
    let repo = Repository::locate(dir.path()).unwrap();
    let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

    let mut graph = ariadne_graph::IncludeGraph::new();
    for rel in ["a.cc", "b.hh"] {
        graph.register_file(FileId::from_relative_path(rel), PathBuf::from(rel));
    }
    graph.add_edge(fixture_include_edge(
        "a.cc",
        "b.hh",
        SourceLanguage::Cpp,
    ));
    let closures = graph.compute_closures();
    let report = graph.generate_report();

    let inc_dir = mgr
        .save_include_artifacts(&graph, &closures, &report, FP)
        .unwrap();
    let json = read_manifest_json(&inc_dir);
    assert!(
        !json.contains("languages"),
        "all-default must skip: {json}"
    );
}

#[test]
fn m22_load_manifest_uses_supplied_languages() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);

    let load_dir = mgr
        .save_load_artifacts_with_languages(
            &fixture_load_report(),
            FP,
            &[
                SourceLanguage::Scala,
                SourceLanguage::Cpp,
                SourceLanguage::Scala,
            ],
        )
        .unwrap();
    let json = read_manifest_json(&load_dir);
    assert_eq!(
        manifest_languages(&json),
        serde_json::json!([SourceLanguage::Scala]),
        "supplied languages must be filtered vs default, deduped, sorted"
    );
}

#[test]
fn m22_load_manifest_all_default_and_plain_save_skip_languages() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);

    let with_default = mgr
        .save_load_artifacts_with_languages(&fixture_load_report(), FP, &[SourceLanguage::Cpp])
        .unwrap();
    let json = read_manifest_json(&with_default);
    assert!(
        !json.contains("languages"),
        "all-default supplied languages must skip: {json}"
    );

    let plain = mgr.save_load_artifacts(&fixture_load_report(), FP).unwrap();
    let json = read_manifest_json(&plain);
    assert!(
        !json.contains("languages"),
        "pre-M22 save path must keep emitting no key: {json}"
    );
}
