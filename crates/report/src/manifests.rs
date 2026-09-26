use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use ariadne_repo::{FileOwnership, OptimizationPolicy};
use ariadne_graph::LoadMetric;

/// Metadata manifest for repository discovery artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryManifest {
    pub repository_root: PathBuf,
    pub total_files: usize,
    pub ownership_summary: HashMap<FileOwnership, usize>,
    pub policy_summary: HashMap<OptimizationPolicy, usize>,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

/// Metadata manifest for compilation context artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildContextManifest {
    pub repository_root: PathBuf,
    pub total_contexts: usize,
    pub total_translation_units: usize,
    pub context_names: Vec<String>,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

/// Metadata manifest for include graph artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeManifest {
    pub repository_root: PathBuf,
    pub total_edges: usize,
    pub unique_headers: usize,
    pub local_headers_count: usize,
    pub system_headers_count: usize,
    pub external_headers_count: usize,
    pub unresolved_includes_count: usize,
    pub cycle_count: usize,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

/// Metadata manifest for semantic index artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexManifest {
    pub repository_root: PathBuf,
    pub total_translation_units: usize,
    pub total_symbols: usize,
    pub total_edges: usize,
    pub complete_edges: usize,
    pub incomplete_edges: usize,
    #[serde(default)]
    pub total_cached_tus: usize,
    #[serde(default)]
    pub reused_tus: usize,
    #[serde(default)]
    pub reanalyzed_tus: usize,
    #[serde(default)]
    pub hit_rate: f64,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

/// Metadata manifest for reduced semantic model artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducedManifest {
    pub repository_root: PathBuf,
    pub total_symbols: usize,
    pub total_edges: usize,
    pub total_tu_usages: usize,
    pub total_indexed_files: usize,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

/// Metadata manifest for load report artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadManifest {
    pub repository_root: PathBuf,
    pub total_analyzed_symbols: usize,
    pub total_analyzed_files: usize,
    pub total_collateral_load: usize,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

/// Metadata manifest for call graph artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphManifest {
    pub total_functions: usize,
    pub total_calls: usize,
    pub recursion_cycles: usize,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadReport {
    pub repository_root: PathBuf,
    pub symbol_metrics: Vec<LoadMetric>,
    pub file_metrics: Vec<LoadMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanManifest {
    pub repository_root: PathBuf,
    pub total_plans: usize,
    pub total_targets: usize,
    pub total_collateral_reduced: usize,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionManifest {
    pub plan_id: String,
    pub header_count: usize,
    pub error_count: usize,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationManifest {
    pub plan_id: String,
    pub total_rewrites: usize,
    pub effective_rewrites: usize,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionManifest {
    pub plan_id: String,
    pub outcome: String,
    pub tiers_run: usize,
    pub tiers_passed: usize,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureManifest {
    pub repository_root: PathBuf,
    pub total_tus: usize,
    pub full_build_wall_time_ms: u128,
    pub total_frontend_cpu_ms: u128,
    pub peak_memory_bytes: u64,
    #[serde(default)]
    pub input_fingerprint: String,
    #[serde(default)]
    pub analysis_version: String,
}
