//! Artifact generation and load reporting.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ariadne_repo::{FileOwnership, OptimizationPolicy, Repository, RepositoryDiscoveryResult};
use ariadne_build::BuildDiscoveryResult;

/// Metadata manifest for repository discovery artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryManifest {
    pub repository_root: PathBuf,
    pub total_files: usize,
    pub ownership_summary: HashMap<FileOwnership, usize>,
    pub policy_summary: HashMap<OptimizationPolicy, usize>,
}

/// Metadata manifest for compilation context artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildContextManifest {
    pub repository_root: PathBuf,
    pub total_contexts: usize,
    pub total_translation_units: usize,
    pub context_names: Vec<String>,
}

pub struct ArtifactManager {
    pub artifact_root: PathBuf,
}

impl ArtifactManager {
    pub fn init<P: AsRef<Path>>(root: P) -> Result<Self> {
        let artifact_root = root.as_ref().join("artifacts");
        fs::create_dir_all(&artifact_root)
            .with_context(|| format!("Failed to create artifact directory at {:?}", artifact_root))?;
        Ok(Self { artifact_root })
    }

    pub fn prepare_for_repo(repo: &Repository) -> Result<Self> {
        Self::init(&repo.root)
    }

    /// Saves files.json, files.bin, and manifest.json into artifacts/repository/
    pub fn save_repository_artifacts(&self, discovery: &RepositoryDiscoveryResult) -> Result<PathBuf> {
        let repo_artifact_dir = self.artifact_root.join("repository");
        fs::create_dir_all(&repo_artifact_dir)
            .with_context(|| format!("Failed to create repository artifact directory at {:?}", repo_artifact_dir))?;

        // 1. files.json
        let json_path = repo_artifact_dir.join("files.json");
        let json_content = serde_json::to_string_pretty(&discovery.files)
            .context("Failed to serialize file records to JSON")?;
        fs::write(&json_path, json_content)
            .with_context(|| format!("Failed to write files.json at {:?}", json_path))?;

        // 2. files.bin
        let bin_path = repo_artifact_dir.join("files.bin");
        let bin_content = bincode::serialize(&discovery.files)
            .context("Failed to serialize file records to binary")?;
        fs::write(&bin_path, bin_content)
            .with_context(|| format!("Failed to write files.bin at {:?}", bin_path))?;

        // 3. manifest.json
        let manifest_path = repo_artifact_dir.join("manifest.json");
        let manifest = RepositoryManifest {
            repository_root: discovery.root.clone(),
            total_files: discovery.files.len(),
            ownership_summary: discovery.ownership_summary.clone(),
            policy_summary: discovery.policy_summary.clone(),
        };
        let manifest_content = serde_json::to_string_pretty(&manifest)
            .context("Failed to serialize repository manifest")?;
        fs::write(&manifest_path, manifest_content)
            .with_context(|| format!("Failed to write manifest.json at {:?}", manifest_path))?;

        Ok(repo_artifact_dir)
    }

    /// Saves contexts.json, commands.json, and manifest.json into artifacts/contexts/
    pub fn save_build_context_artifacts(&self, build_result: &BuildDiscoveryResult) -> Result<PathBuf> {
        let contexts_artifact_dir = self.artifact_root.join("contexts");
        fs::create_dir_all(&contexts_artifact_dir)
            .with_context(|| format!("Failed to create contexts artifact directory at {:?}", contexts_artifact_dir))?;

        // 1. contexts.json
        let contexts_path = contexts_artifact_dir.join("contexts.json");
        let contexts_content = serde_json::to_string_pretty(&build_result.contexts)
            .context("Failed to serialize compilation contexts to JSON")?;
        fs::write(&contexts_path, contexts_content)
            .with_context(|| format!("Failed to write contexts.json at {:?}", contexts_path))?;

        // 2. commands.json
        let commands_path = contexts_artifact_dir.join("commands.json");
        let commands: Vec<_> = build_result
            .translation_units
            .iter()
            .map(|tu| &tu.compile_command)
            .collect();
        let commands_content = serde_json::to_string_pretty(&commands)
            .context("Failed to serialize compile commands to JSON")?;
        fs::write(&commands_path, commands_content)
            .with_context(|| format!("Failed to write commands.json at {:?}", commands_path))?;

        // 3. manifest.json
        let manifest_path = contexts_artifact_dir.join("manifest.json");
        let context_names: Vec<String> = build_result
            .contexts
            .iter()
            .map(|c| c.name.clone())
            .collect();

        let manifest = BuildContextManifest {
            repository_root: build_result.repo_root.clone(),
            total_contexts: build_result.contexts.len(),
            total_translation_units: build_result.translation_units.len(),
            context_names,
        };
        let manifest_content = serde_json::to_string_pretty(&manifest)
            .context("Failed to serialize build context manifest")?;
        fs::write(&manifest_path, manifest_content)
            .with_context(|| format!("Failed to write manifest.json at {:?}", manifest_path))?;

        Ok(contexts_artifact_dir)
    }

    /// Saves edges.bin, closures.bin, and report.json into artifacts/includes/
    pub fn save_include_artifacts(
        &self,
        graph: &ariadne_graph::IncludeGraph,
        closures: &HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
        report: &ariadne_graph::IncludeReport,
    ) -> Result<PathBuf> {
        let includes_artifact_dir = self.artifact_root.join("includes");
        fs::create_dir_all(&includes_artifact_dir)
            .with_context(|| format!("Failed to create includes artifact directory at {:?}", includes_artifact_dir))?;

        // 1. edges.bin
        let edges_path = includes_artifact_dir.join("edges.bin");
        let edges_bin = bincode::serialize(&graph.edges)
            .context("Failed to serialize include edges to binary")?;
        fs::write(&edges_path, edges_bin)
            .with_context(|| format!("Failed to write edges.bin at {:?}", edges_path))?;

        // 2. closures.bin
        let closures_path = includes_artifact_dir.join("closures.bin");
        let closures_bin = bincode::serialize(closures)
            .context("Failed to serialize include closures to binary")?;
        fs::write(&closures_path, closures_bin)
            .with_context(|| format!("Failed to write closures.bin at {:?}", closures_path))?;

        // 3. report.json
        let report_path = includes_artifact_dir.join("report.json");
        let report_json = serde_json::to_string_pretty(report)
            .context("Failed to serialize include report to JSON")?;
        fs::write(&report_path, report_json)
            .with_context(|| format!("Failed to write report.json at {:?}", report_path))?;

        Ok(includes_artifact_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use ariadne_build::{CompilationContext, CompileCommand, TranslationUnit};
    use ariadne_repo::FileId;

    #[test]
    fn test_artifact_manager_creation() {
        let dir = tempdir().unwrap();
        let repo = Repository::locate(dir.path()).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();
        assert!(mgr.artifact_root.exists());
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

        let repo_dir = mgr.save_repository_artifacts(&discovery).unwrap();
        assert!(repo_dir.join("files.json").exists());
        assert!(repo_dir.join("files.bin").exists());
        assert!(repo_dir.join("manifest.json").exists());
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
            arguments: vec!["cl.exe".into(), "/c".into(), "/DDEBUG".into(), "src/main.cpp".into()],
            context_name: "Debug|x64".into(),
            output: None,
        };

        let tu = TranslationUnit {
            file_id: FileId::from_relative_path("src/main.cpp"),
            source_path: root.join("src/main.cpp"),
            context_name: "Debug|x64".into(),
            compile_command: cmd,
        };

        let build_result = BuildDiscoveryResult {
            repo_root: root.to_path_buf(),
            contexts: vec![ctx],
            translation_units: vec![tu],
        };

        let ctx_dir = mgr.save_build_context_artifacts(&build_result).unwrap();
        assert!(ctx_dir.join("contexts.json").exists());
        assert!(ctx_dir.join("commands.json").exists());
        assert!(ctx_dir.join("manifest.json").exists());
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

        let inc_dir = mgr.save_include_artifacts(&graph, &closures, &report).unwrap();
        assert!(inc_dir.join("edges.bin").exists());
        assert!(inc_dir.join("closures.bin").exists());
        assert!(inc_dir.join("report.json").exists());
    }
}
