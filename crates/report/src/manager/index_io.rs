//! Semantic index artifact IO.

use anyhow::{Context, Result};
use ariadne_index::{IncrementalStats, CURRENT_ANALYSIS_VERSION};
use std::fs;
use std::path::PathBuf;

use super::bincode_options::deserialize_bounded;
use super::{write_atomic, write_json_atomic, ArtifactManager};
use crate::manifests::IndexManifest;

impl ArtifactManager {
    /// Saves tu_analyses.bin, index.json, and manifest.json into artifacts/index/
    pub fn save_index_artifacts(
        &self,
        index_result: &ariadne_index::IndexResult,
        input_fingerprint: &str,
        reuse: Option<&IncrementalStats>,
    ) -> Result<PathBuf> {
        let index_artifact_dir = self.artifact_root.join("index");
        fs::create_dir_all(&index_artifact_dir).with_context(|| {
            format!(
                "Failed to create index artifact directory at {:?}",
                index_artifact_dir
            )
        })?;

        // 1. tu_analyses.bin
        let tu_analyses_path = index_artifact_dir.join("tu_analyses.bin");
        let tu_analyses_bin = bincode::serialize(&index_result.tu_analyses)
            .context("Failed to serialize tu analyses to binary")?;
        write_atomic(&tu_analyses_path, &tu_analyses_bin)?;

        // 2. index.json
        let index_path = index_artifact_dir.join("index.json");
        let index_json = serde_json::to_string_pretty(index_result)
            .context("Failed to serialize index result to JSON")?;
        write_atomic(&index_path, index_json.as_bytes())?;

        // 3. manifest.json
        let manifest = IndexManifest {
            repository_root: index_result.repo_root.clone(),
            total_translation_units: index_result.tu_analyses.len(),
            total_symbols: index_result.total_symbols,
            total_edges: index_result.total_edges,
            complete_edges: index_result.complete_edges,
            incomplete_edges: index_result.incomplete_edges,
            total_cached_tus: reuse.map(|s| s.total_tus).unwrap_or(0),
            reused_tus: reuse.map(|s| s.reused_tus).unwrap_or(0),
            reanalyzed_tus: reuse.map(|s| s.reanalyzed_tus).unwrap_or(0),
            hit_rate: reuse.map(|s| s.hit_rate).unwrap_or(0.0),
            input_fingerprint: input_fingerprint.to_string(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };
        write_json_atomic(&index_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(index_artifact_dir)
    }

    pub fn load_index_checked(
        &self,
        input_fingerprint: &str,
    ) -> Result<ariadne_index::IndexResult> {
        self.check_manifest("index", input_fingerprint)?;
        let tu_analyses_path = self.artifact_root.join("index").join("tu_analyses.bin");
        let tu_analyses_bin = fs::read(&tu_analyses_path)
            .with_context(|| format!("Failed to read tu_analyses.bin at {:?}", tu_analyses_path))?;
        let tu_analyses: Vec<ariadne_index::TuAnalysis> =
            deserialize_bounded(&tu_analyses_bin).context("Failed to deserialize tu analyses")?;
        Ok(ariadne_index::IndexResult::new(
            self.repo_root(),
            tu_analyses,
        ))
    }
}
