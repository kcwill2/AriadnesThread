//! Collateral-load metrics and optimization-plan artifact IO.

use anyhow::{Context, Result};
use ariadne_graph::LoadMetric;
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use super::{unique_tmp, write_atomic, write_json_atomic, ArtifactManager};
use crate::manifests::{collect_stage_languages, LoadManifest, LoadReport, PlanManifest};

// ---------------------------------------------------------------------------
// M28 T4 (DP-2, user decision 2026-10-06) load-artifact payload shape.
// The IN-MEMORY `LoadReport` keeps its full detail (the process-lifetime
// hand-out needs it); the PERSISTED forms drop the never-consumed per-symbol
// detail and stream to disk. The old writers materialized the whole report
// twice concurrently (a pretty JSON `String` + a bincode `Vec`) - the two
// byte strings WERE the ~4096 MiB save-span transient measured at N=1280
// (tasks/stage5-collateral-memory-ceiling-t0c-measurement.md sec. 4).
// Persisted shapes:
// - `report.json`: `repository_root`, the `symbol_summary` scalars
//   (analyzed_symbols + collateral_load), and the FULL `file_metrics`
//   detail (the UI reads only `file_metrics` - shape sec. 0's exhaustive
//   consumer read). The `symbol_metrics` array is gone (the user-visible
//   change the DP-2 decision registered).
// - `report.bin`: `repository_root`, `symbol_summary_rows` (per-symbol
//   scalar rows - name, target file, the four load numbers; the detail
//   vectors are never written), and the full `file_metrics`.
// Both writers serialize element-by-element into a BufWriter over the
// unique-tmp file, so no whole-payload byte string ever exists in RAM.
// Bincode is strict, so the layout change cannot deserialize old blobs:
// the `CURRENT_ANALYSIS_VERSION` bump 2.1.0 -> 2.2.0 IS the invalidation
// (`check_manifest` rejects the stale epoch before any blob read -
// `stale_prior_epoch_load_manifest_reaches_rebuild_not_decode_error`).
// ---------------------------------------------------------------------------

/// One symbol's persisted summary row: the scalars `tool measure`, the
/// manifest totals, and the loaded-report contract need; no detail vectors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SymbolSummaryRow {
    target_name: String,
    target_file: Option<PathBuf>,
    semantic_load: usize,
    include_load: usize,
    collateral_load: usize,
    total_load: usize,
}

impl SymbolSummaryRow {
    fn from_metric(metric: &LoadMetric) -> Self {
        Self {
            target_name: metric.target_name.clone(),
            target_file: metric.target_file.clone(),
            semantic_load: metric.semantic_load,
            include_load: metric.include_load,
            collateral_load: metric.collateral_load,
            total_load: metric.total_load,
        }
    }

    /// Rebuilds the loaded-report row: the summary scalars survive, the
    /// never-consumed detail vectors stay absent by construction.
    fn into_metric(self) -> LoadMetric {
        LoadMetric {
            target_name: self.target_name,
            target_file: self.target_file,
            semantic_load: self.semantic_load,
            include_load: self.include_load,
            collateral_load: self.collateral_load,
            total_load: self.total_load,
            semantic_symbols: Vec::new(),
            collateral_symbols: Vec::new(),
            collateral_sources: Vec::new(),
            largest_collateral_source: None,
            semantic_sources: Vec::new(),
        }
    }
}

/// The scalar summary `report.json` carries in place of the detail array.
#[derive(Serialize)]
struct SymbolSummary {
    analyzed_symbols: usize,
    collateral_load: usize,
}

fn symbol_summary_of(report: &LoadReport) -> SymbolSummary {
    SymbolSummary {
        analyzed_symbols: report.symbol_metrics.len(),
        collateral_load: report
            .symbol_metrics
            .iter()
            .map(|metric| metric.collateral_load)
            .sum(),
    }
}

/// Borrowed view serialized into the persisted `report.json`. The derive
/// order fixes the key order; every field borrows, nothing clones detail.
#[derive(Serialize)]
struct PersistedLoadReportJson<'a> {
    repository_root: &'a Path,
    symbol_summary: SymbolSummary,
    file_metrics: &'a [LoadMetric],
}

/// Borrowed view serialized into the persisted `report.bin`. The rows vec is
/// small by construction (scalar-only); `file_metrics` borrows.
#[derive(Serialize)]
struct PersistedLoadReportBin<'a> {
    repository_root: &'a Path,
    symbol_summary_rows: Vec<SymbolSummaryRow>,
    file_metrics: &'a [LoadMetric],
}

/// Owned counterpart deserialized from `report.bin`.
#[derive(Deserialize)]
struct PersistedLoadReport {
    repository_root: PathBuf,
    symbol_summary_rows: Vec<SymbolSummaryRow>,
    file_metrics: Vec<LoadMetric>,
}

impl PersistedLoadReport {
    fn into_load_report(self) -> LoadReport {
        LoadReport {
            repository_root: self.repository_root,
            symbol_metrics: self
                .symbol_summary_rows
                .into_iter()
                .map(SymbolSummaryRow::into_metric)
                .collect(),
            file_metrics: self.file_metrics,
        }
    }
}

/// Streaming sibling of `write_atomic`: run `emit` through a BufWriter over
/// a process-unique temp file, then rename over the target, so a crash
/// mid-write leaves the previous artifact intact AND no whole-payload byte
/// string sits in RAM. Same unique-tmp + rename discipline as
/// `write_atomic` (no new runtime dependency; no fsync, matching the
/// existing writers' durability contract). On emit failure the staged file
/// is removed and the error propagated.
fn write_streaming_atomic<F>(path: &Path, emit: F) -> Result<()>
where
    F: FnOnce(&mut BufWriter<File>) -> Result<()>,
{
    let staged_temp_path = unique_tmp(path);
    let emit_outcome = (|| -> Result<()> {
        let staged_file = File::create(&staged_temp_path).with_context(|| {
            format!("Failed to create temp file at {:?}", staged_temp_path)
        })?;
        let mut writer = BufWriter::new(staged_file);
        emit(&mut writer)?;
        writer
            .flush()
            .with_context(|| format!("Failed to flush temp file at {:?}", staged_temp_path))
    })();
    match emit_outcome {
        Ok(()) => {
            fs::rename(&staged_temp_path, path).with_context(|| {
                format!("Failed to rename {:?} to {:?}", staged_temp_path, path)
            })
        }
        Err(emit_error) => {
            let _ = fs::remove_file(&staged_temp_path);
            Err(emit_error)
        }
    }
}

fn write_load_report_json(path: &Path, report: &LoadReport) -> Result<()> {
    let persisted = PersistedLoadReportJson {
        repository_root: report.repository_root.as_path(),
        symbol_summary: symbol_summary_of(report),
        file_metrics: &report.file_metrics,
    };
    write_streaming_atomic(path, |writer| {
        let mut serializer = serde_json::Serializer::with_formatter(
            writer,
            serde_json::ser::PrettyFormatter::new(),
        );
        persisted
            .serialize(&mut serializer)
            .map_err(|err| {
                anyhow::Error::new(err).context(format!("Failed to serialize {:?}", path))
            })?;
        Ok(())
    })
}

fn write_load_report_binary(path: &Path, report: &LoadReport) -> Result<()> {
    let persisted = PersistedLoadReportBin {
        repository_root: report.repository_root.as_path(),
        symbol_summary_rows: report
            .symbol_metrics
            .iter()
            .map(SymbolSummaryRow::from_metric)
            .collect(),
        file_metrics: &report.file_metrics,
    };
    write_streaming_atomic(path, |writer| {
        bincode::serialize_into(&mut *writer, &persisted).map_err(|err| {
            anyhow::Error::new(err).context("Failed to serialize load report to binary")
        })?;
        Ok(())
    })
}

/// `LoadReport` carries no language data, so the load-stage call site supplies
/// the stage's raw languages (plan M22 D25); this filters against
/// `SourceLanguage::default()`, dedupes, and sorts. An empty/default input
/// keeps the manifest byte-identical to the pre-M22 shape.
fn build_load_manifest(
    report: &LoadReport,
    languages: &[ariadne_repo::SourceLanguage],
    input_fingerprint: &str,
) -> LoadManifest {
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

    LoadManifest {
        repository_root: report.repository_root.clone(),
        total_analyzed_symbols: report.symbol_metrics.len(),
        total_analyzed_files: report.file_metrics.len(),
        total_collateral_load,
        input_fingerprint: input_fingerprint.to_string(),
        analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        languages: collect_stage_languages(languages.iter().map(ariadne_lang::LanguageId::from)),
    }
}

impl ArtifactManager {
    /// Pre-M22 entry point: emits a load manifest with no `languages` key
    /// (the field is skipped when empty). Kept byte-identical for callers
    /// that have no language payload to supply.
    pub fn save_load_artifacts(
        &self,
        report: &LoadReport,
        input_fingerprint: &str,
    ) -> Result<PathBuf> {
        self.save_load_artifacts_with_languages(report, input_fingerprint, &[])
    }

    /// `save_load_artifacts` plus the stage's raw languages (plan M22 D25):
    /// only values beyond `SourceLanguage::default()` are recorded, so an
    /// all-default stage writes exactly the pre-M22 manifest bytes.
    ///
    /// Persisted payload contract (M28 T4 DP-2, user decision 2026-10-06):
    /// the report files carry the scalar symbol summary (json) / scalar
    /// summary rows (bin) plus the FULL consumed `file_metrics` detail - the
    /// never-consumed per-symbol detail vectors are not persisted, and both
    /// files are streamed (no whole-payload byte string). The manifest
    /// totals are computed from the in-memory report and stay byte-identical
    /// across the reshape; a layout change is invalidated by the
    /// `CURRENT_ANALYSIS_VERSION` epoch (2.2.0), which `load_load_checked`
    /// enforces BEFORE any blob deserialization.
    pub fn save_load_artifacts_with_languages(
        &self,
        report: &LoadReport,
        input_fingerprint: &str,
        languages: &[ariadne_repo::SourceLanguage],
    ) -> Result<PathBuf> {
        let load_artifact_dir = self.artifact_root.join("load");
        fs::create_dir_all(&load_artifact_dir).with_context(|| {
            format!(
                "Failed to create load artifact directory at {:?}",
                load_artifact_dir
            )
        })?;

        // The two report serializations are independent CPU-heavy work
        // writing to distinct files, so overlap them. Each streams straight
        // into its own unique-tmp file; the I2 byte-stability lock after the
        // M28 T4 resync is the UNEDITED determinism/threading suite
        // (`load_artifact_bytes_independent_of_threading`): consecutive
        // saves byte-identical, 1-thread and 4-thread pools identical.
        let report_json_path = load_artifact_dir.join("report.json");
        let report_bin_path = load_artifact_dir.join("report.bin");
        let (json_write, bin_write) = rayon::join(
            || write_load_report_json(&report_json_path, report),
            || write_load_report_binary(&report_bin_path, report),
        );
        json_write?;
        bin_write?;

        let manifest = build_load_manifest(report, languages, input_fingerprint);
        write_json_atomic(&load_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(load_artifact_dir)
    }

    /// Reads the reshaped `report.bin` (scalar summary rows + full
    /// file_metrics) through a BufReader - the whole-`Vec` read this path
    /// used to do is the warm-boot twin of the save-span transient class
    /// (shape sec. 0 A7); only the deserialized structure is materialized.
    /// The summary rows come back as scalar-only `symbol_metrics` entries:
    /// the persisted detail is absent, so it cannot silently reappear.
    pub fn load_load_artifacts(&self) -> Result<LoadReport> {
        let report_bin_path = self.artifact_root.join("load").join("report.bin");
        let report_file = fs::File::open(&report_bin_path)
            .with_context(|| format!("Failed to read report.bin at {:?}", report_bin_path))?;
        let persisted: PersistedLoadReport =
            bincode::deserialize_from(std::io::BufReader::new(report_file))
                .context("Failed to deserialize load report")?;
        Ok(persisted.into_load_report())
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
