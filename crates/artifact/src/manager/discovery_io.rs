//! Repository discovery and build-context artifact IO.

use anyhow::{Context, Result};
use ariadne_build::BuildDiscoveryResult;
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use ariadne_repo::{build_discovery_result, FileRecord, RepositoryDiscoveryResult};
use std::fs;
use std::path::PathBuf;

use super::bincode_options::deserialize_bounded;
use super::{write_atomic, write_json_atomic, ArtifactManager};
use crate::manifests::{
    collect_stage_languages, BuildContextManifest, RepositoryManifest,
};

impl ArtifactManager {
    /// Saves files.json, files.bin, and manifest.json into artifacts/repository/
    pub fn save_repository_artifacts(
        &self,
        discovery: &RepositoryDiscoveryResult,
        input_fingerprint: &str,
    ) -> Result<PathBuf> {
        let repo_artifact_dir = self.artifact_root.join("repository");
        fs::create_dir_all(&repo_artifact_dir).with_context(|| {
            format!(
                "Failed to create repository artifact directory at {:?}",
                repo_artifact_dir
            )
        })?;

        // 1. files.json
        let json_path = repo_artifact_dir.join("files.json");
        let json_content = serde_json::to_string_pretty(&discovery.files)
            .context("Failed to serialize file records to JSON")?;
        write_atomic(&json_path, json_content.as_bytes())?;

        // 2. files.bin
        let bin_path = repo_artifact_dir.join("files.bin");
        let bin_content = bincode::serialize(&discovery.files)
            .context("Failed to serialize file records to binary")?;
        write_atomic(&bin_path, &bin_content)?;

        // 3. manifest.json
        let manifest = RepositoryManifest {
            repository_root: discovery.root.clone(),
            total_files: discovery.files.len(),
            ownership_summary: discovery.ownership_summary.clone(),
            policy_summary: discovery.policy_summary.clone(),
            input_fingerprint: input_fingerprint.to_string(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
            languages: collect_stage_languages(discovery.files.iter().map(|f| f.language.clone())),
        };
        write_json_atomic(&repo_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(repo_artifact_dir)
    }

    pub fn load_repository_artifacts(&self) -> Result<Vec<FileRecord>> {
        let bin_path = self.artifact_root.join("repository").join("files.bin");
        let content = fs::read(&bin_path)
            .with_context(|| format!("Failed to read files.bin at {:?}", bin_path))?;
        let records =
            deserialize_bounded(&content).context("Failed to deserialize file records")?;
        Ok(records)
    }

    pub fn load_repository_checked(
        &self,
        input_fingerprint: &str,
    ) -> Result<RepositoryDiscoveryResult> {
        self.check_manifest("repository", input_fingerprint)?;
        let files = self.load_repository_artifacts()?;
        Ok(build_discovery_result(self.repo_root(), files))
    }

    /// Saves contexts.json, commands.json, translation-units.bin, and manifest.json into artifacts/contexts/
    pub fn save_build_context_artifacts(
        &self,
        build_result: &BuildDiscoveryResult,
        input_fingerprint: &str,
    ) -> Result<PathBuf> {
        let contexts_artifact_dir = self.artifact_root.join("contexts");
        fs::create_dir_all(&contexts_artifact_dir).with_context(|| {
            format!(
                "Failed to create contexts artifact directory at {:?}",
                contexts_artifact_dir
            )
        })?;

        // 1. contexts.json
        let contexts_path = contexts_artifact_dir.join("contexts.json");
        let contexts_content = serde_json::to_string_pretty(&build_result.contexts)
            .context("Failed to serialize compilation contexts to JSON")?;
        write_atomic(&contexts_path, contexts_content.as_bytes())?;

        // 2. commands.json
        let commands_path = contexts_artifact_dir.join("commands.json");
        let commands: Vec<_> = build_result
            .translation_units
            .iter()
            .map(|tu| &tu.compile_command)
            .collect();
        let commands_content = serde_json::to_string_pretty(&commands)
            .context("Failed to serialize compile commands to JSON")?;
        write_atomic(&commands_path, commands_content.as_bytes())?;

        // 3. translation-units.bin (lossless TU list for reload)
        let tus_path = contexts_artifact_dir.join("translation-units.bin");
        let tus_bin = bincode::serialize(&build_result.translation_units)
            .context("Failed to serialize translation units to binary")?;
        write_atomic(&tus_path, &tus_bin)?;

        // 4. manifest.json
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
            input_fingerprint: input_fingerprint.to_string(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
            languages: collect_stage_languages(
                build_result
                    .contexts
                    .iter()
                    .map(|c| c.language.clone())
                    .chain(build_result.translation_units.iter().map(|tu| tu.language.clone())),
            ),
        };
        write_json_atomic(&contexts_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(contexts_artifact_dir)
    }

    pub fn load_build_context_checked(
        &self,
        input_fingerprint: &str,
    ) -> Result<BuildDiscoveryResult> {
        self.check_manifest("contexts", input_fingerprint)?;
        let contexts_path = self.artifact_root.join("contexts").join("contexts.json");
        let contexts_json = fs::read_to_string(&contexts_path)
            .with_context(|| format!("Failed to read contexts.json at {:?}", contexts_path))?;
        let contexts = serde_json::from_str(&contexts_json)
            .context("Failed to deserialize compilation contexts")?;

        let tus_path = self
            .artifact_root
            .join("contexts")
            .join("translation-units.bin");
        let tus_bin = fs::read(&tus_path)
            .with_context(|| format!("Failed to read translation-units.bin at {:?}", tus_path))?;
        let translation_units =
            deserialize_bounded(&tus_bin).context("Failed to deserialize translation units")?;

        Ok(BuildDiscoveryResult {
            repo_root: self.repo_root(),
            contexts,
            translation_units,
        })
    }
}
