//! Include-graph, reduced-semantic-model, and call-graph artifact IO.

use anyhow::{Context, Result};
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use ariadne_repo::RepositoryDiscoveryResult;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::{write_atomic, write_json_atomic, ArtifactManager};
use crate::manifests::{
    collect_stage_languages, CallGraphManifest, IncludeManifest, ReducedManifest,
};

impl ArtifactManager {
    /// Saves edges.bin, closures.bin, report.json, and manifest.json into artifacts/includes/
    pub fn save_include_artifacts(
        &self,
        graph: &ariadne_graph::IncludeGraph,
        closures: &HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
        report: &ariadne_graph::IncludeReport,
        input_fingerprint: &str,
    ) -> Result<PathBuf> {
        let includes_artifact_dir = self.artifact_root.join("includes");
        fs::create_dir_all(&includes_artifact_dir).with_context(|| {
            format!(
                "Failed to create includes artifact directory at {:?}",
                includes_artifact_dir
            )
        })?;

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
            languages: collect_stage_languages(graph.edges.iter().map(|e| e.language)),
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
    ) -> Result<(
        ariadne_graph::IncludeGraph,
        HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    )> {
        self.check_manifest("includes", input_fingerprint)?;
        let includes_artifact_dir = self.artifact_root.join("includes");

        let edges_path = includes_artifact_dir.join("edges.bin");
        let edges_bin = fs::read(&edges_path)
            .with_context(|| format!("Failed to read edges.bin at {:?}", edges_path))?;
        let edges: Vec<ariadne_graph::IncludeEdge> =
            bincode::deserialize(&edges_bin).context("Failed to deserialize include edges")?;

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

    /// Saves symbols.bin, edges.bin, tu-usage.bin, file-symbols.bin, and manifest.json into artifacts/reduced/
    pub fn save_reduced_artifacts(
        &self,
        model: &ariadne_graph::ReducedSemanticModel,
        input_fingerprint: &str,
    ) -> Result<PathBuf> {
        let reduced_artifact_dir = self.artifact_root.join("reduced");
        fs::create_dir_all(&reduced_artifact_dir).with_context(|| {
            format!(
                "Failed to create reduced artifact directory at {:?}",
                reduced_artifact_dir
            )
        })?;

        // 1. symbols.bin
        let symbols_path = reduced_artifact_dir.join("symbols.bin");
        let symbols_bin =
            bincode::serialize(&model.symbols).context("Failed to serialize symbols to binary")?;
        write_atomic(&symbols_path, &symbols_bin)?;

        // 2. edges.bin
        let edges_path = reduced_artifact_dir.join("edges.bin");
        let edges_bin =
            bincode::serialize(&model.edges).context("Failed to serialize edges to binary")?;
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
        let symbols =
            bincode::deserialize(&symbols_bin).context("Failed to deserialize symbols")?;

        let edges_path = reduced_artifact_dir.join("edges.bin");
        let edges_bin = fs::read(&edges_path)
            .with_context(|| format!("Failed to read edges.bin at {:?}", edges_path))?;
        let edges = bincode::deserialize(&edges_bin).context("Failed to deserialize edges")?;

        let tu_usage_path = reduced_artifact_dir.join("tu-usage.bin");
        let tu_usage_bin = fs::read(&tu_usage_path)
            .with_context(|| format!("Failed to read tu-usage.bin at {:?}", tu_usage_path))?;
        let tu_usage =
            bincode::deserialize(&tu_usage_bin).context("Failed to deserialize tu usage")?;

        let file_symbols_path = reduced_artifact_dir.join("file-symbols.bin");
        let file_symbols_bin = fs::read(&file_symbols_path).with_context(|| {
            format!("Failed to read file-symbols.bin at {:?}", file_symbols_path)
        })?;
        let file_symbols = bincode::deserialize(&file_symbols_bin)
            .context("Failed to deserialize file symbols")?;

        Ok(ariadne_graph::ReducedSemanticModel {
            repo_root: manifest.repository_root,
            symbols,
            edges,
            tu_usage,
            file_symbols,

            extends_edges: Vec::new(),
            sealed_symbols: Vec::new(),
        })
    }

    pub fn load_reduced_checked(
        &self,
        input_fingerprint: &str,
    ) -> Result<ariadne_graph::ReducedSemanticModel> {
        self.check_manifest("reduced", input_fingerprint)?;
        self.load_reduced_artifacts()
    }

    pub fn save_callgraph_artifacts(
        &self,
        callgraph: &ariadne_graph::CallGraph,
        input_fingerprint: &str,
    ) -> Result<PathBuf> {
        let callgraph_dir = self.artifact_root.join("callgraph");
        fs::create_dir_all(&callgraph_dir).with_context(|| {
            format!(
                "Failed to create callgraph artifact directory at {:?}",
                callgraph_dir
            )
        })?;

        write_json_atomic(&callgraph_dir.join("callgraph.json"), callgraph)?;
        write_atomic(
            &callgraph_dir.join("callgraph.bin"),
            &bincode::serialize(callgraph)?,
        )?;

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
        let binary_payload = fs::read(&bin_path)
            .with_context(|| format!("Failed to read callgraph.bin at {:?}", bin_path))?;
        Ok(bincode::deserialize(&binary_payload)?)
    }

    pub fn load_callgraph_checked(
        &self,
        input_fingerprint: &str,
    ) -> Result<ariadne_graph::CallGraph> {
        self.check_manifest("callgraph", input_fingerprint)?;
        self.load_callgraph_artifacts()
    }
}
