//! Collateral-load metrics and optimization-plan artifact IO.

use anyhow::{Context, Result};
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use std::fs;
use std::path::{Path, PathBuf};

use super::{write_atomic, write_json_atomic, ArtifactManager};
use crate::manifests::{LoadManifest, LoadReport, PlanManifest};

impl ArtifactManager {
    pub fn save_load_artifacts(
        &self,
        report: &LoadReport,
        input_fingerprint: &str,
    ) -> Result<PathBuf> {
        let load_artifact_dir = self.artifact_root.join("load");
        fs::create_dir_all(&load_artifact_dir).with_context(|| {
            format!(
                "Failed to create load artifact directory at {:?}",
                load_artifact_dir
            )
        })?;

        // 1. report.json
        write_json_atomic(&load_artifact_dir.join("report.json"), report)?;

        // 2. report.bin
        let report_bin =
            bincode::serialize(report).context("Failed to serialize load report to binary")?;
        write_atomic(&load_artifact_dir.join("report.bin"), &report_bin)?;

        // 3. manifest.json
        let total_collateral_load: usize = report
            .symbol_metrics
            .iter()
            .map(|m| m.collateral_load)
            .sum::<usize>()
            + report
                .file_metrics
                .iter()
                .map(|m| m.collateral_load)
                .sum::<usize>();

        let manifest = LoadManifest {
            repository_root: report.repository_root.clone(),
            total_analyzed_symbols: report.symbol_metrics.len(),
            total_analyzed_files: report.file_metrics.len(),
            total_collateral_load,
            input_fingerprint: input_fingerprint.to_string(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };
        write_json_atomic(&load_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(load_artifact_dir)
    }

    pub fn load_load_artifacts(&self) -> Result<LoadReport> {
        let report_bin_path = self.artifact_root.join("load").join("report.bin");
        let report_bin = fs::read(&report_bin_path)
            .with_context(|| format!("Failed to read report.bin at {:?}", report_bin_path))?;
        let report: LoadReport =
            bincode::deserialize(&report_bin).context("Failed to deserialize load report")?;
        Ok(report)
    }

    pub fn load_load_checked(&self, input_fingerprint: &str) -> Result<LoadReport> {
        self.check_manifest("load", input_fingerprint)?;
        self.load_load_artifacts()
    }

    pub fn save_plan_artifacts(
        &self,
        repository_root: &Path,
        plans: &[ariadne_cluster::plan::OptimizationPlan],
        input_fingerprint: &str,
    ) -> Result<PathBuf> {
        let plan_artifact_dir = self.artifact_root.join("plans");
        fs::create_dir_all(&plan_artifact_dir).with_context(|| {
            format!(
                "Failed to create plans artifact directory at {:?}",
                plan_artifact_dir
            )
        })?;

        write_json_atomic(&plan_artifact_dir.join("plans.json"), plans)?;
        write_atomic(
            &plan_artifact_dir.join("plans.bin"),
            &bincode::serialize(plans)?,
        )?;

        let total_plans = plans.len();
        let total_targets = plans.len();
        let total_collateral_reduced = plans
            .iter()
            .map(|p| {
                p.collateral_load_before
                    .saturating_sub(p.collateral_load_after)
            })
            .sum();

        let manifest = PlanManifest {
            repository_root: repository_root.to_path_buf(),
            total_plans,
            total_targets,
            total_collateral_reduced,
            input_fingerprint: input_fingerprint.to_string(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };
        write_json_atomic(&plan_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(plan_artifact_dir)
    }

    pub fn load_plan_artifacts(&self) -> Result<Vec<ariadne_cluster::plan::OptimizationPlan>> {
        let bin_path = self.artifact_root.join("plans").join("plans.bin");
        let bin_content = fs::read(&bin_path)
            .with_context(|| format!("Failed to read plans.bin at {:?}", bin_path))?;
        let plans: Vec<ariadne_cluster::plan::OptimizationPlan> =
            bincode::deserialize(&bin_content).context("Failed to deserialize plans")?;
        Ok(plans)
    }

    pub fn load_plan_checked(
        &self,
        input_fingerprint: &str,
    ) -> Result<Vec<ariadne_cluster::plan::OptimizationPlan>> {
        self.check_manifest("plans", input_fingerprint)?;
        self.load_plan_artifacts()
    }
}
