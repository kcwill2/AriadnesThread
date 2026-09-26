use ariadne_graph::IncludeGraph;
use ariadne_repo::{FileId, FileOwnership, OptimizationPolicy};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct FrontierAnalyzer;

impl FrontierAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn find_frontiers(
        &self,
        start_file: FileId,
        collateral_file: FileId,
        include_graph: &IncludeGraph,
        file_policies: &HashMap<FileId, OptimizationPolicy>,
        _file_ownerships: &HashMap<FileId, FileOwnership>,
    ) -> Vec<PathBuf> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parents: HashMap<FileId, FileId> = HashMap::new();

        queue.push_back(start_file);
        visited.insert(start_file);

        let mut found_path = false;

        while let Some(current) = queue.pop_front() {
            if current == collateral_file {
                found_path = true;
                break;
            }

            for next in include_graph.direct_includes(current) {
                if !visited.contains(&next) {
                    visited.insert(next);
                    parents.insert(next, current);
                    queue.push_back(next);
                }
            }
        }

        if !found_path {
            return vec![];
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut curr = collateral_file;
        while curr != start_file {
            path.push(curr);
            curr = *parents.get(&curr).unwrap();
        }
        path.push(start_file);
        path.reverse(); // start_file -> ... -> collateral_file

        // Find the last optimizable header
        let mut last_optimizable = None;
        for file_id in &path {
            if *file_id == collateral_file {
                break; // Stop before the collateral root itself? The requirement says "before reaching ... collateral root headers".
            }
            let policy = file_policies
                .get(file_id)
                .unwrap_or(&OptimizationPolicy::Optimizable);
            if *policy == OptimizationPolicy::Optimizable {
                last_optimizable = Some(*file_id);
            } else {
                break;
            }
        }

        if let Some(frontier_id) = last_optimizable {
            if let Some(path_buf) = include_graph.file_paths.get(&frontier_id) {
                return vec![path_buf.clone()];
            }
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_graph::types::{IncludeEdge, IncludeKind, SourceLocation};

    #[test]
    fn test_frontier_analyzer() {
        let mut graph = IncludeGraph::new();
        let f_start = FileId(1);
        let f_mid1 = FileId(2);
        let f_mid2 = FileId(3);
        let f_col = FileId(4);

        graph.register_file(f_start, PathBuf::from("start.h"));
        graph.register_file(f_mid1, PathBuf::from("mid1.h"));
        graph.register_file(f_mid2, PathBuf::from("mid2.h"));
        graph.register_file(f_col, PathBuf::from("col.h"));

        graph.add_edge(IncludeEdge {
            source: f_start,
            destination: Some(f_mid1),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "mid1.h".into(),
            resolved_path: Some(PathBuf::from("mid1.h")),
        });
        graph.add_edge(IncludeEdge {
            source: f_mid1,
            destination: Some(f_mid2),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "mid2.h".into(),
            resolved_path: Some(PathBuf::from("mid2.h")),
        });
        graph.add_edge(IncludeEdge {
            source: f_mid2,
            destination: Some(f_col),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "col.h".into(),
            resolved_path: Some(PathBuf::from("col.h")),
        });

        let mut policies = HashMap::new();
        policies.insert(f_start, OptimizationPolicy::Optimizable);
        policies.insert(f_mid1, OptimizationPolicy::Optimizable);
        policies.insert(f_mid2, OptimizationPolicy::Frozen); // mid2 is frozen
        policies.insert(f_col, OptimizationPolicy::Optimizable);

        let ownerships = HashMap::new();

        let analyzer = FrontierAnalyzer::new();
        let frontiers = analyzer.find_frontiers(f_start, f_col, &graph, &policies, &ownerships);

        // Frontier should be mid1.h, since mid2 is frozen.
        assert_eq!(frontiers.len(), 1);
        assert_eq!(frontiers[0], PathBuf::from("mid1.h"));
    }
}
