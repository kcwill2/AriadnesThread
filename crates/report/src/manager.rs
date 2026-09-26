use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use ariadne_repo::{Repository, RepositoryDiscoveryResult, FileRecord, build_discovery_result};
use ariadne_build::BuildDiscoveryResult;
use ariadne_index::{CURRENT_ANALYSIS_VERSION, IncrementalStats};

use crate::manifests::*;

/// An artifact that cannot be trusted: missing manifest, mismatched input
/// fingerprint, mismatched repository root, or a different analysis version.
/// Callers treat this as "rebuild the stage", never as a hard failure.
#[derive(Debug, Clone)]
pub struct StaleArtifact {
    pub stage: String,
    pub reason: String,
}

impl std::fmt::Display for StaleArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stale artifact for stage '{}': {}", self.stage, self.reason)
    }
}

impl std::error::Error for StaleArtifact {}

pub fn is_stale(err: &anyhow::Error) -> bool {
    err.downcast_ref::<StaleArtifact>().is_some()
}

pub struct ArtifactManager {
    pub artifact_root: PathBuf,
}

fn stale(stage: &str, reason: impl Into<String>) -> anyhow::Error {
    anyhow::Error::from(StaleArtifact { stage: stage.to_string(), reason: reason.into() })
}

/// Process-unique temp name so concurrent writers of the same artifact never
/// rename away each other's staging file.
fn unique_tmp(path: &Path) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    path.with_file_name(format!(
        ".{}.{}.{}.tmp",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("artifact"),
        std::process::id(),
        id
    ))
}

/// Write bytes to `path` atomically: write a unique temp file next to the
/// target, then rename over the target so a crash mid-write leaves the
/// previous artifact intact.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = unique_tmp(path);
    fs::write(&tmp, bytes)
        .with_context(|| format!("Failed to write temp file at {:?}", tmp))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("Failed to rename {:?} to {:?}", tmp, path))?;
    Ok(())
}

fn write_json_atomic<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .with_context(|| format!("Failed to serialize {:?}", path))?;
    write_atomic(path, json.as_bytes())
}

/// Compare paths ignoring Windows verbatim (`\\?\`) prefixes and casing.
#[cfg(windows)]
fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| -> String {
        p.to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .replace('\\', "/")
            .to_lowercase()
            .trim_end_matches('/')
            .to_string()
    };
    norm(a) == norm(b)
}

#[cfg(not(windows))]
fn same_path(a: &Path, b: &Path) -> bool {
    a == b
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

    pub fn repo_root(&self) -> PathBuf {
        self.artifact_root
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.artifact_root.clone())
    }

    /// Verify a stage manifest declares the expected input fingerprint,
    /// repository root, and analysis version. Any unreadable/corrupt manifest
    /// counts as stale so the stage is rebuilt rather than trusted.
    pub fn stage_is_fresh(&self, stage: &str, expected_fingerprint: &str) -> bool {
        self.check_manifest(stage, expected_fingerprint).is_ok()
    }

    /// Verify a stage manifest declares the expected input fingerprint,
    fn check_manifest(&self, stage: &str, expected_fingerprint: &str) -> Result<()> {
        let manifest_path = self.artifact_root.join(stage).join("manifest.json");
        let content = fs::read_to_string(&manifest_path)
            .map_err(|_| stale(stage, format!("no manifest.json at {:?}", manifest_path)))?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| stale(stage, format!("unreadable manifest.json: {}", e)))?;

        let stored_fp = value
            .get("input_fingerprint")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if stored_fp.is_empty() {
            return Err(stale(stage, "artifact has no input fingerprint"));
        }
        if stored_fp != expected_fingerprint {
            return Err(stale(stage, "input fingerprint mismatch"));
        }

        let stored_version = value
            .get("analysis_version")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if stored_version != CURRENT_ANALYSIS_VERSION {
            return Err(stale(stage, format!(
                "analysis version mismatch: {} != {}",
                stored_version, CURRENT_ANALYSIS_VERSION
            )));
        }

        if let Some(root) = value.get("repository_root").and_then(|v| v.as_str()) {
            if !same_path(Path::new(root), &self.repo_root()) {
                return Err(stale(stage, format!(
                    "repository root mismatch: {} != {}",
                    root,
                    self.repo_root().display()
                )));
            }
        }

        Ok(())
    }

    /// Saves files.json, files.bin, and manifest.json into artifacts/repository/
    pub fn save_repository_artifacts(&self, discovery: &RepositoryDiscoveryResult, input_fingerprint: &str) -> Result<PathBuf> {
        let repo_artifact_dir = self.artifact_root.join("repository");
        fs::create_dir_all(&repo_artifact_dir)
            .with_context(|| format!("Failed to create repository artifact directory at {:?}", repo_artifact_dir))?;

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
        };
        write_json_atomic(&repo_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(repo_artifact_dir)
    }

    pub fn load_repository_artifacts(&self) -> Result<Vec<FileRecord>> {
        let bin_path = self.artifact_root.join("repository").join("files.bin");
        let content = fs::read(&bin_path)
            .with_context(|| format!("Failed to read files.bin at {:?}", bin_path))?;
        let records = bincode::deserialize(&content)
            .context("Failed to deserialize file records")?;
        Ok(records)
    }

    pub fn load_repository_checked(&self, input_fingerprint: &str) -> Result<RepositoryDiscoveryResult> {
        self.check_manifest("repository", input_fingerprint)?;
        let files = self.load_repository_artifacts()?;
        Ok(build_discovery_result(self.repo_root(), files))
    }

    /// Saves contexts.json, commands.json, translation-units.bin, and manifest.json into artifacts/contexts/
    pub fn save_build_context_artifacts(&self, build_result: &BuildDiscoveryResult, input_fingerprint: &str) -> Result<PathBuf> {
        let contexts_artifact_dir = self.artifact_root.join("contexts");
        fs::create_dir_all(&contexts_artifact_dir)
            .with_context(|| format!("Failed to create contexts artifact directory at {:?}", contexts_artifact_dir))?;

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
        };
        write_json_atomic(&contexts_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(contexts_artifact_dir)
    }

    pub fn load_build_context_checked(&self, input_fingerprint: &str) -> Result<BuildDiscoveryResult> {
        self.check_manifest("contexts", input_fingerprint)?;
        let contexts_path = self.artifact_root.join("contexts").join("contexts.json");
        let contexts_json = fs::read_to_string(&contexts_path)
            .with_context(|| format!("Failed to read contexts.json at {:?}", contexts_path))?;
        let contexts = serde_json::from_str(&contexts_json)
            .context("Failed to deserialize compilation contexts")?;

        let tus_path = self.artifact_root.join("contexts").join("translation-units.bin");
        let tus_bin = fs::read(&tus_path)
            .with_context(|| format!("Failed to read translation-units.bin at {:?}", tus_path))?;
        let translation_units = bincode::deserialize(&tus_bin)
            .context("Failed to deserialize translation units")?;

        Ok(BuildDiscoveryResult {
            repo_root: self.repo_root(),
            contexts,
            translation_units,
        })
    }

    /// Saves edges.bin, closures.bin, report.json, and manifest.json into artifacts/includes/
    pub fn save_include_artifacts(
        &self,
        graph: &ariadne_graph::IncludeGraph,
        closures: &HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
        report: &ariadne_graph::IncludeReport,
        input_fingerprint: &str,
    ) -> Result<PathBuf> {
        let includes_artifact_dir = self.artifact_root.join("includes");
        fs::create_dir_all(&includes_artifact_dir)
            .with_context(|| format!("Failed to create includes artifact directory at {:?}", includes_artifact_dir))?;

        // 1. edges.bin
        let edges_path = includes_artifact_dir.join("edges.bin");
        let edges_bin = bincode::serialize(&graph.edges)
            .context("Failed to serialize include edges to binary")?;
        write_atomic(&edges_path, &edges_bin)?;

        // 2. closures.bin
        let closures_path = includes_artifact_dir.join("closures.bin");
        let closures_bin = bincode::serialize(closures)
            .context("Failed to serialize include closures to binary")?;
        write_atomic(&closures_path, &closures_bin)?;

        // 3. report.json
        let report_path = includes_artifact_dir.join("report.json");
        let report_json = serde_json::to_string_pretty(report)
            .context("Failed to serialize include report to JSON")?;
        write_atomic(&report_path, report_json.as_bytes())?;

        // 4. manifest.json
        let manifest = IncludeManifest {
            repository_root: self.repo_root(),
            total_edges: report.total_edges,
            unique_headers: report.unique_headers,
            local_headers_count: report.local_headers_count,
            system_headers_count: report.system_headers_count,
            external_headers_count: report.external_headers_count,
            unresolved_includes_count: report.unresolved_includes_count,
            cycle_count: report.cycle_count,
            input_fingerprint: input_fingerprint.to_string(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };
        write_json_atomic(&includes_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(includes_artifact_dir)
    }

    /// Load a fingerprint-checked include graph. File registrations are
    /// rebuilt from the current repository discovery so paths stay fresh.
    pub fn load_include_checked(
        &self,
        input_fingerprint: &str,
        discovery: &RepositoryDiscoveryResult,
    ) -> Result<(ariadne_graph::IncludeGraph, HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>)> {
        self.check_manifest("includes", input_fingerprint)?;
        let includes_artifact_dir = self.artifact_root.join("includes");

        let edges_path = includes_artifact_dir.join("edges.bin");
        let edges_bin = fs::read(&edges_path)
            .with_context(|| format!("Failed to read edges.bin at {:?}", edges_path))?;
        let edges: Vec<ariadne_graph::IncludeEdge> = bincode::deserialize(&edges_bin)
            .context("Failed to deserialize include edges")?;

        let closures_path = includes_artifact_dir.join("closures.bin");
        let closures_bin = fs::read(&closures_path)
            .with_context(|| format!("Failed to read closures.bin at {:?}", closures_path))?;
        let closures = bincode::deserialize(&closures_bin)
            .context("Failed to deserialize include closures")?;

        let mut graph = ariadne_graph::IncludeGraph::new();
        for file_record in &discovery.files {
            graph.register_file(file_record.id, file_record.path.clone());
        }
        for edge in edges {
            graph.add_edge(edge);
        }

        Ok((graph, closures))
    }

    /// Saves tu_analyses.bin, index.json, and manifest.json into artifacts/index/
    pub fn save_index_artifacts(
        &self,
        index_result: &ariadne_index::IndexResult,
        input_fingerprint: &str,
        reuse: Option<&IncrementalStats>,
    ) -> Result<PathBuf> {
        let index_artifact_dir = self.artifact_root.join("index");
        fs::create_dir_all(&index_artifact_dir)
            .with_context(|| format!("Failed to create index artifact directory at {:?}", index_artifact_dir))?;

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

    pub fn load_index_checked(&self, input_fingerprint: &str) -> Result<ariadne_index::IndexResult> {
        self.check_manifest("index", input_fingerprint)?;
        let tu_analyses_path = self.artifact_root.join("index").join("tu_analyses.bin");
        let tu_analyses_bin = fs::read(&tu_analyses_path)
            .with_context(|| format!("Failed to read tu_analyses.bin at {:?}", tu_analyses_path))?;
        let tu_analyses: Vec<ariadne_index::TuAnalysis> = bincode::deserialize(&tu_analyses_bin)
            .context("Failed to deserialize tu analyses")?;
        Ok(ariadne_index::IndexResult::new(self.repo_root(), tu_analyses))
    }

    /// Saves symbols.bin, edges.bin, tu-usage.bin, file-symbols.bin, and manifest.json into artifacts/reduced/
    pub fn save_reduced_artifacts(&self, model: &ariadne_graph::ReducedSemanticModel, input_fingerprint: &str) -> Result<PathBuf> {
        let reduced_artifact_dir = self.artifact_root.join("reduced");
        fs::create_dir_all(&reduced_artifact_dir)
            .with_context(|| format!("Failed to create reduced artifact directory at {:?}", reduced_artifact_dir))?;

        // 1. symbols.bin
        let symbols_path = reduced_artifact_dir.join("symbols.bin");
        let symbols_bin = bincode::serialize(&model.symbols)
            .context("Failed to serialize symbols to binary")?;
        write_atomic(&symbols_path, &symbols_bin)?;

        // 2. edges.bin
        let edges_path = reduced_artifact_dir.join("edges.bin");
        let edges_bin = bincode::serialize(&model.edges)
            .context("Failed to serialize edges to binary")?;
        write_atomic(&edges_path, &edges_bin)?;

        // 3. tu-usage.bin
        let tu_usage_path = reduced_artifact_dir.join("tu-usage.bin");
        let tu_usage_bin = bincode::serialize(&model.tu_usage)
            .context("Failed to serialize tu usage to binary")?;
        write_atomic(&tu_usage_path, &tu_usage_bin)?;

        // 4. file-symbols.bin
        let file_symbols_path = reduced_artifact_dir.join("file-symbols.bin");
        let file_symbols_bin = bincode::serialize(&model.file_symbols)
            .context("Failed to serialize file symbols to binary")?;
        write_atomic(&file_symbols_path, &file_symbols_bin)?;

        // 5. manifest.json
        let manifest = ReducedManifest {
            repository_root: model.repo_root.clone(),
            total_symbols: model.symbols.len(),
            total_edges: model.edges.len(),
            total_tu_usages: model.tu_usage.len(),
            total_indexed_files: model.file_symbols.len(),
            input_fingerprint: input_fingerprint.to_string(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };
        write_json_atomic(&reduced_artifact_dir.join("manifest.json"), &manifest)?;

        Ok(reduced_artifact_dir)
    }

    pub fn load_reduced_artifacts(&self) -> Result<ariadne_graph::ReducedSemanticModel> {
        let reduced_artifact_dir = self.artifact_root.join("reduced");

        let manifest_path = reduced_artifact_dir.join("manifest.json");
        let manifest_json = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read manifest.json at {:?}", manifest_path))?;
        let manifest: ReducedManifest = serde_json::from_str(&manifest_json)
            .context("Failed to deserialize reduced manifest")?;

        let symbols_path = reduced_artifact_dir.join("symbols.bin");
        let symbols_bin = fs::read(&symbols_path)
            .with_context(|| format!("Failed to read symbols.bin at {:?}", symbols_path))?;
        let symbols = bincode::deserialize(&symbols_bin)
            .context("Failed to deserialize symbols")?;

        let edges_path = reduced_artifact_dir.join("edges.bin");
        let edges_bin = fs::read(&edges_path)
            .with_context(|| format!("Failed to read edges.bin at {:?}", edges_path))?;
        let edges = bincode::deserialize(&edges_bin)
            .context("Failed to deserialize edges")?;

        let tu_usage_path = reduced_artifact_dir.join("tu-usage.bin");
        let tu_usage_bin = fs::read(&tu_usage_path)
            .with_context(|| format!("Failed to read tu-usage.bin at {:?}", tu_usage_path))?;
        let tu_usage = bincode::deserialize(&tu_usage_bin)
            .context("Failed to deserialize tu usage")?;

        let file_symbols_path = reduced_artifact_dir.join("file-symbols.bin");
        let file_symbols_bin = fs::read(&file_symbols_path)
            .with_context(|| format!("Failed to read file-symbols.bin at {:?}", file_symbols_path))?;
        let file_symbols = bincode::deserialize(&file_symbols_bin)
            .context("Failed to deserialize file symbols")?;

        Ok(ariadne_graph::ReducedSemanticModel {
            repo_root: manifest.repository_root,
            symbols,
            edges,
            tu_usage,
            file_symbols,
        })
    }

    pub fn load_reduced_checked(&self, input_fingerprint: &str) -> Result<ariadne_graph::ReducedSemanticModel> {
        self.check_manifest("reduced", input_fingerprint)?;
        self.load_reduced_artifacts()
    }

    pub fn save_callgraph_artifacts(&self, callgraph: &ariadne_graph::CallGraph, input_fingerprint: &str) -> Result<PathBuf> {
        let callgraph_dir = self.artifact_root.join("callgraph");
        fs::create_dir_all(&callgraph_dir)
            .with_context(|| format!("Failed to create callgraph artifact directory at {:?}", callgraph_dir))?;

        write_json_atomic(&callgraph_dir.join("callgraph.json"), callgraph)?;
        write_atomic(&callgraph_dir.join("callgraph.bin"), &bincode::serialize(callgraph)?)?;

        let cycles = callgraph.detect_recursion_cycles().len();
        let manifest = CallGraphManifest {
            total_functions: callgraph.nodes.len(),
            total_calls: callgraph.edges.iter().map(|e| e.call_count).sum(),
            recursion_cycles: cycles,
            input_fingerprint: input_fingerprint.to_string(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };
        write_json_atomic(&callgraph_dir.join("manifest.json"), &manifest)?;

        Ok(callgraph_dir)
    }

    pub fn load_callgraph_artifacts(&self) -> Result<ariadne_graph::CallGraph> {
        let bin_path = self.artifact_root.join("callgraph").join("callgraph.bin");
        let data = fs::read(&bin_path)
            .with_context(|| format!("Failed to read callgraph.bin at {:?}", bin_path))?;
        Ok(bincode::deserialize(&data)?)
    }

    pub fn load_callgraph_checked(&self, input_fingerprint: &str) -> Result<ariadne_graph::CallGraph> {
        self.check_manifest("callgraph", input_fingerprint)?;
        self.load_callgraph_artifacts()
    }

    pub fn save_load_artifacts(&self, report: &LoadReport, input_fingerprint: &str) -> Result<PathBuf> {
        let load_artifact_dir = self.artifact_root.join("load");
        fs::create_dir_all(&load_artifact_dir)
            .with_context(|| format!("Failed to create load artifact directory at {:?}", load_artifact_dir))?;

        // 1. report.json
        write_json_atomic(&load_artifact_dir.join("report.json"), report)?;

        // 2. report.bin
        let report_bin = bincode::serialize(report)
            .context("Failed to serialize load report to binary")?;
        write_atomic(&load_artifact_dir.join("report.bin"), &report_bin)?;

        // 3. manifest.json
        let total_collateral_load: usize = report.symbol_metrics.iter().map(|m| m.collateral_load).sum::<usize>()
            + report.file_metrics.iter().map(|m| m.collateral_load).sum::<usize>();

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
        let report: LoadReport = bincode::deserialize(&report_bin)
            .context("Failed to deserialize load report")?;
        Ok(report)
    }

    pub fn load_load_checked(&self, input_fingerprint: &str) -> Result<LoadReport> {
        self.check_manifest("load", input_fingerprint)?;
        self.load_load_artifacts()
    }

    pub fn save_plan_artifacts(&self, repository_root: &Path, plans: &[ariadne_cluster::plan::OptimizationPlan], input_fingerprint: &str) -> Result<PathBuf> {
        let plan_artifact_dir = self.artifact_root.join("plans");
        fs::create_dir_all(&plan_artifact_dir)
            .with_context(|| format!("Failed to create plans artifact directory at {:?}", plan_artifact_dir))?;

        write_json_atomic(&plan_artifact_dir.join("plans.json"), plans)?;
        write_atomic(&plan_artifact_dir.join("plans.bin"), &bincode::serialize(plans)?)?;

        let total_plans = plans.len();
        let total_targets = plans.len();
        let total_collateral_reduced = plans.iter().map(|p| p.collateral_load_before.saturating_sub(p.collateral_load_after)).sum();

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
        let plans: Vec<ariadne_cluster::plan::OptimizationPlan> = bincode::deserialize(&bin_content)
            .context("Failed to deserialize plans")?;
        Ok(plans)
    }

    pub fn load_plan_checked(&self, input_fingerprint: &str) -> Result<Vec<ariadne_cluster::plan::OptimizationPlan>> {
        self.check_manifest("plans", input_fingerprint)?;
        self.load_plan_artifacts()
    }

    pub fn save_extraction_artifacts(
        &self,
        result: &ariadne_transform::ExtractionResult,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create extraction artifact directory at {:?}", output_dir))?;

        let manifest = ExtractionManifest {
            plan_id: result.plan_id.clone(),
            header_count: result.headers.len(),
            error_count: result.errors.len(),
            input_fingerprint: String::new(),
            analysis_version: CURRENT_ANALYSIS_VERSION.to_string(),
        };

        write_json_atomic(&output_dir.join("manifest.json"), &manifest)?;

        let headers_dir = output_dir.join("headers");
        result.write_to_disk(&headers_dir)
            .context("Failed to write extracted headers to disk")?;

        Ok(output_dir.to_path_buf())
    }

    /// Saves preview.bin, preview.json, and manifest.json into artifacts/migration/
    pub fn save_migration_artifacts(
        &self,
        preview: &ariadne_transform::MigrationPreview,
    ) -> Result<PathBuf> {
        let migration_dir = self.artifact_root.join("migration");
        fs::create_dir_all(&migration_dir)
            .with_context(|| format!("Failed to create migration artifact directory at {:?}", migration_dir))?;

        // 1. preview.bin
        write_atomic(&migration_dir.join("preview.bin"), &bincode::serialize(preview)?)?;

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
            ariadne_validate::ApplyOutcome::RolledBack { failed_tier: _, tiers_passed } => {
                ("RolledBack", tiers_passed.len() + 1, tiers_passed.len())
            }
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

    pub fn save_measurement_artifacts(&self, measurement: &ariadne_build::BuildMeasurement) -> Result<PathBuf> {
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
        write_atomic(&measure_dir.join("measurements.bin"), &bincode::serialize(measurement)?)?;

        Ok(measure_dir)
    }

    pub fn load_measurement_artifacts(&self) -> Result<ariadne_build::BuildMeasurement> {
        let measure_dir = self.artifact_root.join("measure");
        Self::load_measurement_from_path(&measure_dir)
    }

    pub fn load_measurement_from_path<P: AsRef<Path>>(path: P) -> Result<ariadne_build::BuildMeasurement> {
        let path = path.as_ref();

        let bin_path = if path.is_dir() { path.join("measurements.bin") } else { path.to_path_buf() };
        let json_path = if path.is_dir() { path.join("measurements.json") } else { path.to_path_buf() };

        if bin_path.exists() && bin_path.extension().and_then(|s| s.to_str()) == Some("bin") {
            let data = fs::read(&bin_path)?;
            if let Ok(m) = bincode::deserialize(&data) {
                return Ok(m);
            }
        }

        if json_path.exists() && (json_path.extension().and_then(|s| s.to_str()) == Some("json") || path.is_dir()) {
            if let Ok(data) = fs::read_to_string(&json_path) {
                if let Ok(m) = serde_json::from_str(&data) {
                    return Ok(m);
                }
            }
        }

        anyhow::bail!("Could not load measurement artifacts from path {:?}", path);
    }

    pub fn save_comparison_artifacts(&self, comparison: &ariadne_build::MeasurementComparison) -> Result<PathBuf> {
        let measure_dir = self.artifact_root.join("measure");
        fs::create_dir_all(&measure_dir)?;

        write_json_atomic(&measure_dir.join("comparison.json"), comparison)?;
        write_atomic(&measure_dir.join("report.txt"), comparison.format_report().as_bytes())?;

        Ok(measure_dir)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use ariadne_build::{CompilationContext, CompileCommand, TranslationUnit};
    use ariadne_repo::FileId;

    const FP: &str = "test-fingerprint";

    #[test]
    fn test_artifact_manager_creation() {
        let dir = tempdir().unwrap();
        let repo = Repository::locate(dir.path()).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();
        assert!(mgr.artifact_root.exists());
    }

    #[test]
    fn test_write_atomic_replaces_existing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("thing.bin");
        fs::write(&path, b"old").unwrap();
        write_atomic(&path, b"new").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert!(!path.with_extension("bin.tmp").exists());
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

        let repo_dir = mgr.save_repository_artifacts(&discovery, FP).unwrap();
        assert!(repo_dir.join("files.json").exists());
        assert!(repo_dir.join("files.bin").exists());
        assert!(repo_dir.join("manifest.json").exists());

        let loaded = mgr.load_repository_checked(FP).unwrap();
        assert_eq!(loaded.files.len(), discovery.files.len());
    }

    #[test]
    fn test_load_repository_checked_rejects_mismatch() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("include")).unwrap();
        fs::write(root.join("include/test.h"), "#pragma once").unwrap();

        let repo = Repository::locate(root).unwrap();
        let discovery = repo.discover().unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();
        mgr.save_repository_artifacts(&discovery, FP).unwrap();

        let err = mgr.load_repository_checked("other-fingerprint").unwrap_err();
        assert!(is_stale(&err), "fingerprint mismatch must report StaleArtifact: {}", err);
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

        let ctx_dir = mgr.save_build_context_artifacts(&build_result, FP).unwrap();
        assert!(ctx_dir.join("contexts.json").exists());
        assert!(ctx_dir.join("commands.json").exists());
        assert!(ctx_dir.join("manifest.json").exists());

        let loaded = mgr.load_build_context_checked(FP).unwrap();
        assert_eq!(loaded.contexts.len(), 1);
        assert_eq!(loaded.translation_units.len(), 1);
        assert_eq!(loaded.translation_units[0].source_path, root.join("src/main.cpp"));
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

        let inc_dir = mgr.save_include_artifacts(&graph, &closures, &report, FP).unwrap();
        assert!(inc_dir.join("edges.bin").exists());
        assert!(inc_dir.join("closures.bin").exists());
        assert!(inc_dir.join("report.json").exists());
        assert!(inc_dir.join("manifest.json").exists());

        let discovery = repo.discover().unwrap();
        let (loaded, loaded_closures) = mgr.load_include_checked(FP, &discovery).unwrap();
        assert_eq!(loaded.edges.len(), graph.edges.len());
        assert_eq!(loaded_closures.len(), closures.len());
    }

    #[test]
    fn test_save_index_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let index_result = ariadne_index::IndexResult::new(repo.root.clone(), vec![]);

        let index_dir = mgr.save_index_artifacts(&index_result, FP, None).unwrap();
        assert!(index_dir.join("tu_analyses.bin").exists());
        assert!(index_dir.join("index.json").exists());
        assert!(index_dir.join("manifest.json").exists());

        let loaded = mgr.load_index_checked(FP).unwrap();
        assert_eq!(loaded.repo_root, repo.root);
    }

    #[test]
    fn test_save_reduced_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let index_result = ariadne_index::IndexResult::new(repo.root.clone(), vec![]);
        let reduced = ariadne_graph::ReducedSemanticModel::reduce(&index_result);

        let reduced_dir = mgr.save_reduced_artifacts(&reduced, FP).unwrap();
        assert!(reduced_dir.join("symbols.bin").exists());
        assert!(reduced_dir.join("edges.bin").exists());
        assert!(reduced_dir.join("tu-usage.bin").exists());
        assert!(reduced_dir.join("file-symbols.bin").exists());
        assert!(reduced_dir.join("manifest.json").exists());
    }

    #[test]
    fn test_load_reduced_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let index_result = ariadne_index::IndexResult::new(repo.root.clone(), vec![]);
        let reduced = ariadne_graph::ReducedSemanticModel::reduce(&index_result);

        mgr.save_reduced_artifacts(&reduced, FP).unwrap();

        let loaded = mgr.load_reduced_artifacts().unwrap();
        assert_eq!(reduced.symbols, loaded.symbols);
        assert_eq!(reduced.edges, loaded.edges);
        assert_eq!(reduced.tu_usage, loaded.tu_usage);
        assert_eq!(reduced.file_symbols, loaded.file_symbols);

        let checked = mgr.load_reduced_checked(FP).unwrap();
        assert_eq!(reduced.symbols, checked.symbols);
    }

    #[test]
    fn test_fingerprintless_manifest_is_stale() {
        // Artifacts written before fingerprints existed carry no input
        // fingerprint and must never be trusted by checked loaders.
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let reduced_dir = mgr.artifact_root.join("reduced");
        fs::create_dir_all(&reduced_dir).unwrap();
        fs::write(
            reduced_dir.join("manifest.json"),
            serde_json::to_string(&serde_json::json!({
                "repository_root": repo.root,
                "total_symbols": 0,
                "total_edges": 0,
                "total_tu_usages": 0,
                "total_indexed_files": 0
            }))
            .unwrap(),
        )
        .unwrap();

        let err = mgr.load_reduced_checked("whatever").unwrap_err();
        assert!(is_stale(&err));
    }

    #[test]
    fn test_save_and_load_load_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let report = LoadReport {
            repository_root: repo.root.clone(),
            symbol_metrics: vec![
                ariadne_graph::LoadMetric {
                    target_name: "TestSymbol".to_string(),
                    target_file: Some(PathBuf::from("test.cpp")),
                    semantic_load: 10,
                    include_load: 20,
                    collateral_load: 10,
                    total_load: 20,
                    semantic_symbols: vec![],
                    collateral_symbols: vec![],
                    collateral_sources: vec![],
                    largest_collateral_source: None,
                }
            ],
            file_metrics: vec![],
        };

        let load_dir = mgr.save_load_artifacts(&report, FP).unwrap();
        assert!(load_dir.join("report.json").exists());
        assert!(load_dir.join("report.bin").exists());
        assert!(load_dir.join("manifest.json").exists());

        let loaded = mgr.load_load_checked(FP).unwrap();
        assert_eq!(report.symbol_metrics.len(), loaded.symbol_metrics.len());
        assert_eq!(report.symbol_metrics[0].target_name, loaded.symbol_metrics[0].target_name);
    }

    #[test]
    fn test_save_and_load_plan_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let plan = ariadne_cluster::plan::OptimizationPlan {
            id: ariadne_cluster::plan::PlanId("test-plan".to_string()),
            target_file: PathBuf::from("test.cpp"),
            target_file_id: FileId::from_relative_path("test.cpp"),
            problem: "too big".to_string(),
            collateral_load_before: 100,
            collateral_load_after: 58,
            proposed_extractions: vec![],
            optimization_frontier: vec![],
            risk: ariadne_cluster::plan::RiskLevel::Conservative,
            validation_required_tus: vec![],
            rationale: "why not".to_string(),
        };

        let plans = vec![plan.clone()];
        let plan_dir = mgr.save_plan_artifacts(&repo.root, &plans, FP).unwrap();

        assert!(plan_dir.join("plans.json").exists());
        assert!(plan_dir.join("plans.bin").exists());
        assert!(plan_dir.join("manifest.json").exists());

        let loaded = mgr.load_plan_checked(FP).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].collateral_load_before, 100);
        assert_eq!(loaded[0].collateral_load_after, 58);
    }

    #[test]
    fn test_save_extraction_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let sym1 = ariadne_transform::ExtractedSymbol {
            name: "MyClass".to_string(),
            kind: ariadne_index::types::SymbolKind::Class,
            forward_declarable: true,
            body: "// body1".to_string(),
        };
        let header1 = ariadne_transform::GeneratedHeader {
            filename: std::path::PathBuf::from("h1.h"),
            pragma_once: true,
            symbols: vec![sym1],
            required_includes: vec![],
        };

        let result = ariadne_transform::ExtractionResult {
            plan_id: "PLAN-1".to_string(),
            headers: vec![header1],
            errors: vec![],
        };

        let output_dir = mgr.artifact_root.join("generated");
        mgr.save_extraction_artifacts(&result, &output_dir).unwrap();

        assert!(output_dir.join("manifest.json").exists());
        assert!(output_dir.join("headers").join("h1.h").exists());
    }

    #[test]
    fn test_save_migration_artifacts() {
        use ariadne_transform::{ConsumerRewrite, MigrationPreview};
        use std::path::PathBuf;

        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let preview = MigrationPreview {
            plan_id: "PLAN-001".to_string(),
            rewrites: vec![
                ConsumerRewrite {
                    consumer_file: PathBuf::from("src/foo.cpp"),
                    old_includes: vec![PathBuf::from("thing.h")],
                    new_includes: vec![PathBuf::from("thing_types.h")],
                    side_effect_preservations: vec![],
                },
            ],
        };

        let migration_dir = mgr.save_migration_artifacts(&preview).unwrap();
        assert!(migration_dir.join("preview.bin").exists());
        assert!(migration_dir.join("preview.json").exists());
        assert!(migration_dir.join("manifest.json").exists());
    }

    #[test]
    fn test_load_migration_artifacts_roundtrip() {
        use ariadne_transform::{ConsumerRewrite, MigrationPreview};
        use std::path::PathBuf;

        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let preview = MigrationPreview {
            plan_id: "PLAN-042".to_string(),
            rewrites: vec![
                ConsumerRewrite {
                    consumer_file: PathBuf::from("src/bar.cpp"),
                    old_includes: vec![PathBuf::from("thing.h")],
                    new_includes: vec![PathBuf::from("thing_core.h"), PathBuf::from("thing_types.h")],
                    side_effect_preservations: vec![],
                },
                ConsumerRewrite {
                    consumer_file: PathBuf::from("src/init.cpp"),
                    old_includes: vec![PathBuf::from("thing.h"), PathBuf::from("registry.h")],
                    new_includes: vec![PathBuf::from("thing_types.h")],
                    side_effect_preservations: vec!["registry.h".into()],
                },
            ],
        };

        mgr.save_migration_artifacts(&preview).unwrap();
        let loaded = mgr.load_migration_artifacts().unwrap();

        assert_eq!(loaded.plan_id, "PLAN-042");
        assert_eq!(loaded.rewrites.len(), 2);
        assert_eq!(loaded.rewrites[0].consumer_file, PathBuf::from("src/bar.cpp"));
        assert_eq!(loaded.rewrites[1].side_effect_preservations, vec![PathBuf::from("registry.h")]);
    }

    #[test]
    fn test_migration_manifest_content() {
        use ariadne_transform::{ConsumerRewrite, MigrationPreview};
        use std::path::PathBuf;

        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let preview = MigrationPreview {
            plan_id: "PLAN-007".to_string(),
            rewrites: vec![
                ConsumerRewrite {
                    consumer_file: PathBuf::from("src/a.cpp"),
                    old_includes: vec![PathBuf::from("thing.h")],
                    new_includes: vec![PathBuf::from("thing_types.h")],
                    side_effect_preservations: vec![],
                },
                ConsumerRewrite {
                    // noop rewrite
                    consumer_file: PathBuf::from("src/b.cpp"),
                    old_includes: vec![PathBuf::from("thing.h")],
                    new_includes: vec![PathBuf::from("thing.h")],
                    side_effect_preservations: vec![],
                },
            ],
        };

        let migration_dir = mgr.save_migration_artifacts(&preview).unwrap();
        let manifest_content = std::fs::read_to_string(migration_dir.join("manifest.json")).unwrap();
        let manifest: MigrationManifest = serde_json::from_str(&manifest_content).unwrap();

        assert_eq!(manifest.plan_id, "PLAN-007");
        assert_eq!(manifest.total_rewrites, 2);
        assert_eq!(manifest.effective_rewrites, 1); // only a.cpp changes
    }

    #[test]
    fn test_save_transaction_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let outcome = ariadne_validate::ApplyOutcome::Committed {
            tiers_passed: vec![ariadne_validate::TierResult {
                tier: ariadne_validate::ValidationTier::Structural,
                status: ariadne_validate::ValidationStatus::Pass,
            }],
        };

        let tx_dir = mgr.save_transaction_artifacts("PLAN-TX", &outcome).unwrap();
        assert!(tx_dir.join("manifest.json").exists());
        assert!(tx_dir.join("outcome.json").exists());

        let manifest_content = fs::read_to_string(tx_dir.join("manifest.json")).unwrap();
        assert!(manifest_content.contains("PLAN-TX"));
        assert!(manifest_content.contains("Committed"));
    }

    #[test]
    fn test_save_and_load_measurement_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let measurement = ariadne_build::BuildMeasurement {
            repo_root: root.to_path_buf(),
            timestamp: 12345,
            full_build_wall_time: std::time::Duration::from_millis(500),
            total_frontend_cpu_time: std::time::Duration::from_millis(1500),
            total_preprocessing_time: std::time::Duration::from_millis(200),
            peak_memory_bytes: 1024,
            tu_measurements: vec![],
            collateral_load: Some(10),
            affected_tus_count: Some(5),
        };

        let measure_dir = mgr.save_measurement_artifacts(&measurement).unwrap();
        assert!(measure_dir.join("measurements.json").exists());
        assert!(measure_dir.join("measurements.bin").exists());
        assert!(measure_dir.join("manifest.json").exists());

        let loaded = mgr.load_measurement_artifacts().unwrap();
        assert_eq!(measurement.timestamp, loaded.timestamp);
        assert_eq!(measurement.peak_memory_bytes, loaded.peak_memory_bytes);

        // Test load_measurement_from_path with directory
        let loaded_from_dir = ArtifactManager::load_measurement_from_path(&measure_dir).unwrap();
        assert_eq!(measurement.timestamp, loaded_from_dir.timestamp);

        // Test load_measurement_from_path with json file
        let json_path = measure_dir.join("measurements.json");
        let loaded_from_json = ArtifactManager::load_measurement_from_path(&json_path).unwrap();
        assert_eq!(measurement.timestamp, loaded_from_json.timestamp);

        // Test load_measurement_from_path with bin file
        let bin_path = measure_dir.join("measurements.bin");
        let loaded_from_bin = ArtifactManager::load_measurement_from_path(&bin_path).unwrap();
        assert_eq!(measurement.timestamp, loaded_from_bin.timestamp);
    }

    #[test]
    fn test_save_comparison_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let before = ariadne_build::BuildMeasurement {
            repo_root: root.to_path_buf(),
            timestamp: 12345,
            full_build_wall_time: std::time::Duration::from_millis(500),
            total_frontend_cpu_time: std::time::Duration::from_millis(1500),
            total_preprocessing_time: std::time::Duration::from_millis(200),
            peak_memory_bytes: 1024,
            tu_measurements: vec![],
            collateral_load: Some(10),
            affected_tus_count: Some(5),
        };

        let after = ariadne_build::BuildMeasurement {
            repo_root: root.to_path_buf(),
            timestamp: 12346,
            full_build_wall_time: std::time::Duration::from_millis(250),
            total_frontend_cpu_time: std::time::Duration::from_millis(750),
            total_preprocessing_time: std::time::Duration::from_millis(100),
            peak_memory_bytes: 512,
            tu_measurements: vec![],
            collateral_load: Some(5),
            affected_tus_count: Some(5),
        };

        let comparison = ariadne_build::MeasurementComparison::compare(before, after);
        let measure_dir = mgr.save_comparison_artifacts(&comparison).unwrap();
        assert!(measure_dir.join("comparison.json").exists());
        assert!(measure_dir.join("report.txt").exists());
        let report_content = fs::read_to_string(measure_dir.join("report.txt")).unwrap();
        assert!(!report_content.is_empty());
    }

    #[test]
    fn test_save_load_callgraph_artifacts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let repo = Repository::locate(root).unwrap();
        let mgr = ArtifactManager::prepare_for_repo(&repo).unwrap();

        let mut callgraph = ariadne_graph::CallGraph::default();
        let f1 = FileId::from_relative_path("main.cpp");
        let s1 = ariadne_index::types::SymbolId::new("A");
        callgraph.nodes.push(ariadne_graph::CallNode {
            symbol_id: s1.clone(),
            name: "A".into(),
            owning_file: f1.clone(),
            location: None,
        });

        let callgraph_dir = mgr.save_callgraph_artifacts(&callgraph, FP).unwrap();
        assert!(callgraph_dir.join("callgraph.json").exists());
        assert!(callgraph_dir.join("callgraph.bin").exists());
        assert!(callgraph_dir.join("manifest.json").exists());

        let loaded = mgr.load_callgraph_checked(FP).unwrap();
        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].name, "A");
    }
}
