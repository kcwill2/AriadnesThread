//! Header-extraction and migration-preview artifact IO.

use anyhow::{Context, Result};
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use std::fs;
use std::path::{Path, PathBuf};

use super::{write_atomic, write_json_atomic, ArtifactManager};
use crate::manifests::{ExtractionManifest, MigrationManifest};

impl ArtifactManager {
    pub fn save_extraction_artifacts(
        &self,
        result: &ariadne_transform::ExtractionResult,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "Failed to create extraction artifact directory at {:?}",
                output_dir
            )
        })?;

        let manifest = ExtractionManifest {
            plan_id: result.plan_id.clone(),
            header_count: result.headers.len(),
            error_count: result.errors.len(),
            input_fingerprint: String::new(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };

        write_json_atomic(&output_dir.join("manifest.json"), &manifest)?;

        let headers_dir = output_dir.join("headers");
        result
            .write_to_disk(&headers_dir)
            .context("Failed to write extracted headers to disk")?;

        Ok(output_dir.to_path_buf())
    }

    /// Saves preview.bin, preview.json, and manifest.json into artifacts/migration/
    pub fn save_migration_artifacts(
        &self,
        preview: &ariadne_transform::MigrationPreview,
    ) -> Result<PathBuf> {
        let migration_dir = self.artifact_root.join("migration");
        fs::create_dir_all(&migration_dir).with_context(|| {
            format!(
                "Failed to create migration artifact directory at {:?}",
                migration_dir
            )
        })?;

        // 1. preview.bin
        write_atomic(
            &migration_dir.join("preview.bin"),
            &bincode::serialize(preview)?,
        )?;

        // 2. preview.json
        write_json_atomic(&migration_dir.join("preview.json"), preview)?;

        // 3. manifest.json
        let manifest = MigrationManifest {
            plan_id: preview.plan_id.clone(),
            total_rewrites: preview.rewrites.len(),
            effective_rewrites: preview.effective_rewrites(),
            input_fingerprint: String::new(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };
        write_json_atomic(&migration_dir.join("manifest.json"), &manifest)?;

        Ok(migration_dir)
    }

    /// Loads migration preview from artifacts/migration/preview.bin
    pub fn load_migration_artifacts(&self) -> Result<ariadne_transform::MigrationPreview> {
        let bin_path = self.artifact_root.join("migration").join("preview.bin");
        let bin_content = fs::read(&bin_path)
            .with_context(|| format!("Failed to read preview.bin at {:?}", bin_path))?;
        let preview: ariadne_transform::MigrationPreview = bincode::deserialize(&bin_content)
            .context("Failed to deserialize migration preview")?;
        Ok(preview)
    }
}
