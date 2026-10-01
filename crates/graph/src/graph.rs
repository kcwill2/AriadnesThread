//! Physical file include graph structure, closure calculation, cycle detection, and visualization.

use crate::types::{IncludeClosure, IncludeEdge, IncludeKind, IncludeReport};
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// Only the test-only scan counter touches atomics; the import stays
// `cfg(test)` so release builds emit no unused-import warning.
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
use std::sync::Arc;

/// Central include graph model managing dependency edges, closures, and analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncludeGraph {
    pub edges: Vec<IncludeEdge>,
    pub file_paths: HashMap<FileId, PathBuf>,
    /// Test-only tally of `find_file_id` O(V) string scans (plan LoadPerf
    /// A2); the by-id analyzer path must keep this at zero. `Arc` so the
    /// existing `Clone` derive still works (a clone shares the tally);
    /// Serde-skipped so artifact bytes never see it (absent in non-test
    /// builds anyway).
    #[cfg(test)]
    #[serde(skip)]
    find_file_id_scans: Arc<AtomicUsize>,
}

impl IncludeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_file(&mut self, file_id: FileId, path: PathBuf) {
        self.file_paths.insert(file_id, path);
    }

    pub fn add_edge(&mut self, edge: IncludeEdge) {
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }
    }

    /// Finds a FileId by matching filename or relative path.
    pub fn find_file_id(&self, path_or_name: &Path) -> Option<FileId> {
        #[cfg(test)]
        self.find_file_id_scans.fetch_add(1, Ordering::Relaxed);
        let target_str = path_or_name.to_string_lossy();
        let target_name = path_or_name
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        for (&id, path) in &self.file_paths {
            if path == path_or_name || path.to_string_lossy() == target_str {
                return Some(id);
            }
            if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                if fname == target_name {
                    return Some(id);
                }
            }
        }
        None
    }

    /// Test-only: tally of `find_file_id` scans on this graph; the by-id
    /// analyzer path (plan LoadPerf A2) must keep this at zero.
    #[cfg(test)]
    pub(crate) fn find_file_id_scan_count(&self) -> usize {
        self.find_file_id_scans.load(Ordering::Relaxed)
    }

    /// Builds a `source -> sorted, deduplicated destinations` map from the edge list.
    /// Matches `direct_includes` semantics; lets whole-graph traversals run in O(V + E)
    /// instead of rescanning the edge list at every visited node. The single
    /// adjacency-build call sites are `compute_closures` and `LoadAnalyzer`'s
    /// `file_adjacency` memo — no other code may duplicate this loop.
    pub(crate) fn adjacency(&self) -> HashMap<FileId, Vec<FileId>> {
        let mut adj: HashMap<FileId, Vec<FileId>> = HashMap::new();
        for edge in &self.edges {
            if let Some(dest) = edge.destination {
                adj.entry(edge.source).or_default().push(dest);
            }
        }
        for dests in adj.values_mut() {
            dests.sort();
            dests.dedup();
        }
        adj
    }

    /// Returns direct outgoing include file IDs for a source file.
    pub fn direct_includes(&self, file_id: FileId) -> Vec<FileId> {
        let mut direct = Vec::new();
        for edge in &self.edges {
            if edge.source == file_id {
                if let Some(dest) = edge.destination {
                    if !direct.contains(&dest) {
                        direct.push(dest);
                    }
                }
            }
        }
        direct.sort();
        direct
    }

    /// Computes direct and transitive include closures for all known files in the graph.
    ///
    /// Delegates to the SCC-condensation DP in `closures` (D1, plan
    /// 1790608181551); the scoped file set is unchanged from the original
    /// per-file BFS: registered files plus edge sources plus resolved
    /// destinations. Byte-identical output is locked by the equivalence
    /// suite in `closure_equivalence_tests.rs` against the frozen BFS oracle
    /// below.
    pub fn compute_closures(&self) -> HashMap<FileId, IncludeClosure> {
        let adj = self.adjacency();

        let mut all_files: HashSet<FileId> = self.file_paths.keys().copied().collect();
        for edge in &self.edges {
            all_files.insert(edge.source);
            if let Some(dest) = edge.destination {
                all_files.insert(dest);
            }
        }

        crate::closures::compute_closures(&adj, &all_files)
    }

    /// Test-only oracle for D1 (plan 1790608181551): the ORIGINAL per-file
    /// BFS body of `compute_closures`, preserved verbatim as the definition
    /// of correct. The SCC-condensation rewrite may never change this body;
    /// it seeds `visited` with the queried file itself, which is what
    /// excludes the seed from `transitive_includes` even inside cycles.
    #[cfg(test)]
    pub(crate) fn compute_closures_bfs_oracle(&self) -> HashMap<FileId, IncludeClosure> {
        let mut closures = HashMap::new();
        let adj = self.adjacency();

        let mut all_files: HashSet<FileId> = self.file_paths.keys().copied().collect();
        for edge in &self.edges {
            all_files.insert(edge.source);
            if let Some(dest) = edge.destination {
                all_files.insert(dest);
            }
        }

        for &file_id in &all_files {
            let direct: Vec<FileId> = adj.get(&file_id).cloned().unwrap_or_default();
            let mut transitive = HashSet::new();
            let mut visited = HashSet::new();
            let mut stack = direct.clone();

            visited.insert(file_id);

            while let Some(current) = stack.pop() {
                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current);
                transitive.insert(current);

                if let Some(next_dests) = adj.get(&current) {
                    for &next_dest in next_dests {
                        if !visited.contains(&next_dest) {
                            stack.push(next_dest);
                        }
                    }
                }
            }

            let mut transitive_vec: Vec<FileId> = transitive.into_iter().collect();
            transitive_vec.sort();

            let closure = IncludeClosure::new(file_id, direct, transitive_vec);
            closures.insert(file_id, closure);
        }

        closures
    }

    /// Maps each aggregation unit (today's C++ header, plan §3.4) to the set
    /// of units that include it directly or transitively — the neutral
    /// aggregation-consumer query (plan M18.5) that package/module-based
    /// languages reuse unchanged; zero logic change from the transitional
    /// header-named alias.
    pub fn map_aggregation_consumers(
        &self,
        tu_file_ids: &[FileId],
    ) -> HashMap<FileId, Vec<FileId>> {
        let closures = self.compute_closures();
        let mut header_consumers: HashMap<FileId, HashSet<FileId>> = HashMap::new();

        for &tu_id in tu_file_ids {
            if let Some(closure) = closures.get(&tu_id) {
                for &inc_header in &closure.transitive_includes {
                    header_consumers
                        .entry(inc_header)
                        .or_default()
                        .insert(tu_id);
                }
            }
        }

        let mut result = HashMap::new();
        for (header_id, tu_set) in header_consumers {
            let mut tu_list: Vec<FileId> = tu_set.into_iter().collect();
            tu_list.sort();
            result.insert(header_id, tu_list);
        }

        result
    }

    /// Detects cycles in the include graph using DFS traversal.
    pub fn detect_cycles(&self) -> Vec<Vec<FileId>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut current_path = Vec::new();

        let mut all_files: Vec<FileId> = self.file_paths.keys().copied().collect();
        all_files.sort();
        let adj = self.adjacency();

        for &node in &all_files {
            if !visited.contains(&node) {
                Self::dfs_cycle(
                    node,
                    &adj,
                    &mut visited,
                    &mut rec_stack,
                    &mut current_path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_cycle(
        node: FileId,
        adj: &HashMap<FileId, Vec<FileId>>,
        visited: &mut HashSet<FileId>,
        rec_stack: &mut HashSet<FileId>,
        current_path: &mut Vec<FileId>,
        cycles: &mut Vec<Vec<FileId>>,
    ) {
        visited.insert(node);
        rec_stack.insert(node);
        current_path.push(node);

        if let Some(neighbors) = adj.get(&node) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    Self::dfs_cycle(neighbor, adj, visited, rec_stack, current_path, cycles);
                } else if rec_stack.contains(&neighbor) {
                    if let Some(pos) = current_path.iter().position(|&n| n == neighbor) {
                        let cycle = current_path[pos..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        rec_stack.remove(&node);
        current_path.pop();
    }

    /// Generates an aggregated summary report of the include graph.
    pub fn generate_report(&self) -> IncludeReport {
        let closures = self.compute_closures();
        let mut unique_headers = HashSet::new();
        let mut local_count = 0;
        let mut system_count = 0;
        let mut external_count = 0;
        let mut unresolved_count = 0;

        for edge in &self.edges {
            if edge.destination.is_none() {
                unresolved_count += 1;
            } else if let Some(dest) = edge.destination {
                unique_headers.insert(dest);
            }

            match edge.kind {
                IncludeKind::Local => local_count += 1,
                IncludeKind::System => system_count += 1,
                IncludeKind::External => external_count += 1,
            }
        }

        let mut closure_sizes: Vec<(FileId, PathBuf, usize)> = closures
            .iter()
            .map(|(&id, closure)| {
                let path = self
                    .file_paths
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| PathBuf::from(format!("{:016x}", id.0)));
                (id, path, closure.closure_size)
            })
            .collect();

        closure_sizes.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(&b.1)));
        let top_headers = closure_sizes.into_iter().take(10).collect();

        IncludeReport {
            total_edges: self.edges.len(),
            unique_headers: unique_headers.len(),
            local_headers_count: local_count,
            system_headers_count: system_count,
            external_headers_count: external_count,
            unresolved_includes_count: unresolved_count,
            cycle_count: self.detect_cycles().len(),
            top_headers_by_closure: top_headers,
        }
    }

    /// Formats a tree visualization for the `explain` command for a given target file.
    pub fn format_explain(&self, root_id: FileId) -> String {
        let mut output = String::new();
        let root_name = self
            .file_paths
            .get(&root_id)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| format!("{:016x}", root_id.0));

        output.push_str(&root_name);
        output.push('\n');

        let mut visited = HashSet::new();
        visited.insert(root_id);

        self.format_explain_node(root_id, "", &mut visited, &mut output);

        output
    }

    fn format_explain_node(
        &self,
        parent_id: FileId,
        prefix: &str,
        visited: &mut HashSet<FileId>,
        output: &mut String,
    ) {
        let children = self.direct_includes(parent_id);
        let count = children.len();

        for (idx, &child_id) in children.iter().enumerate() {
            let is_last = idx == count - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let child_name = self
                .file_paths
                .get(&child_id)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| format!("{:016x}", child_id.0));

            output.push_str(prefix);
            output.push_str(connector);
            output.push_str(&child_name);

            if visited.contains(&child_id) {
                output.push_str(" (cycle)\n");
            } else {
                output.push('\n');
                visited.insert(child_id);
                let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                self.format_explain_node(child_id, &child_prefix, visited, output);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceLocation;

    #[test]
    fn test_include_graph_closure_and_explain() {
        let mut graph = IncludeGraph::new();

        let f_main = FileId(1);
        let f_thing = FileId(2);
        let f_types = FileId(3);
        let f_heavy = FileId(4);

        graph.register_file(f_main, PathBuf::from("src/thing.cpp"));
        graph.register_file(f_thing, PathBuf::from("include/thing.h"));
        graph.register_file(f_types, PathBuf::from("include/thing_types.h"));
        graph.register_file(f_heavy, PathBuf::from("include/heavy.h"));

        // main -> thing.h
        graph.add_edge(IncludeEdge {
            source: f_main,
            destination: Some(f_thing),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "thing.h".into(),
            resolved_path: Some(PathBuf::from("include/thing.h")),
            language: ariadne_repo::SourceLanguage::Cpp,
        });

        // thing.h -> thing_types.h
        graph.add_edge(IncludeEdge {
            source: f_thing,
            destination: Some(f_types),
            context_name: "Default".into(),
            source_location: SourceLocation::new(3, 1),
            kind: IncludeKind::Local,
            raw_path: "thing_types.h".into(),
            resolved_path: Some(PathBuf::from("include/thing_types.h")),
            language: ariadne_repo::SourceLanguage::Cpp,
        });

        // thing.h -> heavy.h
        graph.add_edge(IncludeEdge {
            source: f_thing,
            destination: Some(f_heavy),
            context_name: "Default".into(),
            source_location: SourceLocation::new(4, 1),
            kind: IncludeKind::Local,
            raw_path: "heavy.h".into(),
            resolved_path: Some(PathBuf::from("include/heavy.h")),
            language: ariadne_repo::SourceLanguage::Cpp,
        });

        let closures = graph.compute_closures();
        let main_closure = closures.get(&f_main).unwrap();
        assert_eq!(main_closure.closure_size, 3);
        assert!(main_closure.transitive_includes.contains(&f_thing));
        assert!(main_closure.transitive_includes.contains(&f_types));
        assert!(main_closure.transitive_includes.contains(&f_heavy));

        let thing_closure = closures.get(&f_thing).unwrap();
        assert_eq!(thing_closure.closure_size, 2);

        let tree_explain = graph.format_explain(f_thing);
        assert!(tree_explain.contains("include/thing.h"));
        assert!(tree_explain.contains("heavy.h"));
        assert!(tree_explain.contains("thing_types.h"));
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = IncludeGraph::new();
        let f_a = FileId(1);
        let f_b = FileId(2);

        graph.register_file(f_a, PathBuf::from("a.h"));
        graph.register_file(f_b, PathBuf::from("b.h"));

        // a.h -> b.h
        graph.add_edge(IncludeEdge {
            source: f_a,
            destination: Some(f_b),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "b.h".into(),
            resolved_path: Some(PathBuf::from("b.h")),
            language: ariadne_repo::SourceLanguage::Cpp,
        });

        // b.h -> a.h
        graph.add_edge(IncludeEdge {
            source: f_b,
            destination: Some(f_a),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "a.h".into(),
            resolved_path: Some(PathBuf::from("a.h")),
            language: ariadne_repo::SourceLanguage::Cpp,
        });

        let cycles = graph.detect_cycles();
        assert!(
            !cycles.is_empty(),
            "Cycle must be detected between a.h and b.h"
        );
    }
}

/// D1 equivalence suite (plan 1790608181551): locks the SCC-condensation
/// `compute_closures` rewrite byte-for-byte against the verbatim BFS oracle
/// (`IncludeGraph::compute_closures_bfs_oracle`) over every existing graph
/// test fixture and seeded random digraphs with cycles.
#[cfg(test)]
#[path = "closure_equivalence_tests.rs"]
mod closure_equivalence;
