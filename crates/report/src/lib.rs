//! Artifact generation and load reporting.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ariadne_repo::{FileOwnership, OptimizationPolicy, Repository, RepositoryDiscoveryResult};

/// Metadata manifest for repository discovery artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryManifest {
    pub repository_root: PathBuf,
    pub total_files: usize,
    pub ownership_summary: HashMap<FileOwnership, usize>,
    pub policy_summary: HashMap<OptimizationPolicy, usize>,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
}
