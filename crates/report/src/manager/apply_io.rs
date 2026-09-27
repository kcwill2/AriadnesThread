//! Apply-transaction and build-measurement artifact IO.

use anyhow::Result;
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use std::fs;
use std::path::{Path, PathBuf};

use super::{write_atomic, write_json_atomic, ArtifactManager};
use crate::manifests::{MeasureManifest, TransactionManifest};

impl ArtifactManager {
    pub fn save_transaction_artifacts(
        &self,
        plan_id: &str,
        outcome: &ariadne_validate::ApplyOutcome,
    ) -> Result<PathBuf> {
        let transaction_dir = self.artifact_root.join("transaction");
        fs::create_dir_all(&transaction_dir)?;

        let (outcome_str, tiers_run, tiers_passed) = match outcome {
            ariadne_validate::ApplyOutcome::Committed { tiers_passed } => {
                ("Committed", tiers_passed.len(), tiers_passed.len())
            }
            ariadne_validate::ApplyOutcome::RolledBack {
                failed_tier: _,
                tiers_passed,
            } => ("RolledBack", tiers_passed.len() + 1, tiers_passed.len()),
        };

        let manifest = TransactionManifest {
            plan_id: plan_id.to_string(),
            outcome: outcome_str.to_string(),
            tiers_run,
            tiers_passed,
            input_fingerprint: String::new(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };

        write_json_atomic(&transaction_dir.join("manifest.json"), &manifest)?;
        write_json_atomic(&transaction_dir.join("outcome.json"), outcome)?;

        Ok(transaction_dir)
    }

    pub fn save_measurement_artifacts(
        &self,
        measurement: &ariadne_build::BuildMeasurement,
    ) -> Result<PathBuf> {
        let measure_dir = self.artifact_root.join("measure");
        fs::create_dir_all(&measure_dir)?;

        let manifest = MeasureManifest {
            repository_root: measurement.repo_root.clone(),
            total_tus: measurement.tu_measurements.len(),
            full_build_wall_time_ms: measurement.full_build_wall_time.as_millis(),
            total_frontend_cpu_ms: measurement.total_frontend_cpu_time.as_millis(),
            peak_memory_bytes: measurement.peak_memory_bytes,
            input_fingerprint: String::new(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };

        write_json_atomic(&measure_dir.join("manifest.json"), &manifest)?;
        write_json_atomic(&measure_dir.join("measurements.json"), measurement)?;
        write_atomic(
            &measure_dir.join("measurements.bin"),
            &bincode::serialize(measurement)?,
        )?;

        Ok(measure_dir)
    }

    pub fn load_measurement_artifacts(&self) -> Result<ariadne_build::BuildMeasurement> {
        let measure_dir = self.artifact_root.join("measure");
        Self::load_measurement_from_path(&measure_dir)
    }

    pub fn load_measurement_from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<ariadne_build::BuildMeasurement> {
        let path = path.as_ref();

        let bin_path = if path.is_dir() {
            path.join("measurements.bin")
        } else {
            path.to_path_buf()
        };
        let json_path = if path.is_dir() {
            path.join("measurements.json")
        } else {
            path.to_path_buf()
        };

        if bin_path.exists() && bin_path.extension().and_then(|s| s.to_str()) == Some("bin") {
            let binary_payload = fs::read(&bin_path)?;
            if let Ok(m) = bincode::deserialize(&binary_payload) {
                return Ok(m);
            }
        }

        if json_path.exists()
            && (json_path.extension().and_then(|s| s.to_str()) == Some("json") || path.is_dir())
        {
            if let Ok(data) = fs::read_to_string(&json_path) {
                if let Ok(m) = serde_json::from_str(&data) {
                    return Ok(m);
                }
            }
        }

        anyhow::bail!("Could not load measurement artifacts from path {:?}", path);
    }

    pub fn save_comparison_artifacts(
        &self,
        comparison: &ariadne_build::MeasurementComparison,
    ) -> Result<PathBuf> {
        let measure_dir = self.artifact_root.join("measure");
        fs::create_dir_all(&measure_dir)?;

        write_json_atomic(&measure_dir.join("comparison.json"), comparison)?;
        write_atomic(
            &measure_dir.join("report.txt"),
            comparison.format_report().as_bytes(),
        )?;

        Ok(measure_dir)
    }
}
