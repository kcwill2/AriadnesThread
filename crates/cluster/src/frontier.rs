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
            curr = *parents
                .get(&curr)
                .expect("BFS records a parent for every visited node before start_file");
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
            language: ariadne_lang::LanguageId::cpp(),
        });
        graph.add_edge(IncludeEdge {
            source: f_mid1,
            destination: Some(f_mid2),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "mid2.h".into(),
            resolved_path: Some(PathBuf::from("mid2.h")),
            language: ariadne_lang::LanguageId::cpp(),
        });
        graph.add_edge(IncludeEdge {
            source: f_mid2,
            destination: Some(f_col),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "col.h".into(),
            resolved_path: Some(PathBuf::from("col.h")),
            language: ariadne_lang::LanguageId::cpp(),
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

#[cfg(test)]
mod path_shape_pins {
    //! UTA-T6 item 5 (pin-only): the five path shapes of `find_frontiers`
    //! and the two adjudicated smells from
    //! `docs/specs/unit-test-audit.md` (break-at-first-non-optimizable at
    //! frontier.rs:71-75, `_file_ownerships` ignored at frontier.rs:20).
    //! Pins, not fixes: the owner decision on both smells is pending.

    use super::*;
    use ariadne_graph::types::{IncludeEdge, IncludeKind, SourceLocation};

    fn chain_edge(source: FileId, destination: FileId) -> IncludeEdge {
        IncludeEdge {
            source,
            destination: Some(destination),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: String::new(),
            resolved_path: None,
            language: ariadne_lang::LanguageId::cpp(),
        }
    }

    /// Registers `names` as FileId(1..=n) and chains each file to the next
    /// (`names[i] -> names[i+1]`), so the FIRST is the start and the LAST is
    /// the collateral root.
    fn registered_chain(names: &[&str]) -> IncludeGraph {
        let mut graph = IncludeGraph::new();
        for (index, name) in names.iter().enumerate() {
            graph.register_file(FileId(index as u64 + 1), PathBuf::from(name));
        }
        for index in 0..names.len().saturating_sub(1) {
            graph.add_edge(chain_edge(
                FileId(index as u64 + 1),
                FileId(index as u64 + 2),
            ));
        }
        graph
    }

    fn frontiers(
        graph: &IncludeGraph,
        start: FileId,
        collateral: FileId,
        policies: &HashMap<FileId, OptimizationPolicy>,
        ownerships: &HashMap<FileId, FileOwnership>,
    ) -> Vec<PathBuf> {
        FrontierAnalyzer::new().find_frontiers(start, collateral, graph, policies, ownerships)
    }

    fn all_optimizable(count: usize) -> HashMap<FileId, OptimizationPolicy> {
        (1..=count as u64)
            .map(|id| {
                (
                    FileId(id),
                    OptimizationPolicy::Optimizable,
                )
            })
            .collect()
    }

    #[test]
    fn shape_no_path_returns_empty() {
        // start -> mid exists; the collateral root is a different, detached
        // file the BFS can never reach.
        let graph = registered_chain(&["start.h", "mid.h"]);
        let policies = all_optimizable(2);
        let frontiers = frontiers(
            &graph,
            FileId(1),
            FileId(9),
            &policies,
            &HashMap::new(),
        );
        assert!(
            frontiers.is_empty(),
            "no BFS path to the collateral root must answer []"
        );
    }

    #[test]
    fn shape_start_equals_collateral_returns_empty() {
        let graph = registered_chain(&["solo.h"]);
        let frontiers = frontiers(
            &graph,
            FileId(1),
            FileId(1),
            &all_optimizable(1),
            &HashMap::new(),
        );
        assert!(
            frontiers.is_empty(),
            "pin: start == collateral finds the target on the first pop, the \
             reconstruction path is [start] and the scan breaks before any \
             optimizable is recorded, so the answer is []"
        );
    }

    #[test]
    fn shape_all_optimizable_frontier_is_the_file_before_collateral() {
        let graph = registered_chain(&["start.h", "mid.h", "col.h"]);
        let frontiers = frontiers(
            &graph,
            FileId(1),
            FileId(3),
            &all_optimizable(3),
            &HashMap::new(),
        );
        assert_eq!(
            frontiers,
            vec![PathBuf::from("mid.h")],
            "with every header optimizable the frontier is the deepest \
             optimizable header before the collateral root (collateral's own \
             policy is never consulted - the scan breaks on it)"
        );
    }

    #[test]
    fn shape_missing_policy_entry_treated_as_optimizable() {
        // The lookup defaults missing entries to Optimizable
        // (frontier.rs:68-70): an UNMAPPED file is optimizable by default.
        let graph = registered_chain(&["start.h", "unmapped.h", "col.h"]);
        let only_col_mapped = HashMap::from([(
            FileId(3),
            OptimizationPolicy::Frozen,
        )]);
        let frontiers = frontiers(&graph, FileId(1), FileId(3), &only_col_mapped, &HashMap::new());
        assert_eq!(
            frontiers,
            vec![PathBuf::from("unmapped.h")],
            "the unmapped middle header is treated as Optimizable and wins \
             the frontier (the collateral root's own Frozen entry never gates)"
        );
    }

    #[test]
    fn pin_frontier_breaks_at_first_frozen() {
        // ADJUDICATED SMELL (frontier.rs:71-75): the scan walks start->...
        // and BREAKS at the first non-Optimizable entry. On
        // start -> frozen -> optimizable -> collateral the optimizable
        // header CLOSER to the collateral is never reached, so the frontier
        // is start.h itself. Pinned, not endorsed.
        let graph = registered_chain(&["start.h", "frozen.h", "opt.h", "col.h"]);
        let policies = HashMap::from([
            (FileId(1), OptimizationPolicy::Optimizable),
            (FileId(2), OptimizationPolicy::Frozen),
            (FileId(3), OptimizationPolicy::Optimizable),
            (FileId(4), OptimizationPolicy::Optimizable),
        ]);
        let frontiers = frontiers(&graph, FileId(1), FileId(4), &policies, &HashMap::new());
        assert_eq!(
            frontiers,
            vec![PathBuf::from("start.h")],
            "pin: the scan stops at frozen.h, never seeing the optimizable \
             opt.h behind it"
        );
    }

    #[test]
    fn pin_frontier_ignores_ownership() {
        // ADJUDICATED SMELL (frontier.rs:20): the parameter is literally
        // `_file_ownerships` - no ownership value (ThirdParty, Toolchain,
        // whatever) can veto a frontier. Pinned: the answer is byte-equal
        // with an empty ownership map and with an all-foreign one.
        let graph = registered_chain(&["start.h", "mid.h", "col.h"]);
        let policies = all_optimizable(3);
        let without_ownerships = frontiers(&graph, FileId(1), FileId(3), &policies, &HashMap::new());
        let all_foreign = HashMap::from([
            (FileId(1), FileOwnership::ThirdParty),
            (FileId(2), FileOwnership::ThirdParty),
            (FileId(3), FileOwnership::Toolchain),
        ]);
        let with_foreign_ownerships =
            frontiers(&graph, FileId(1), FileId(3), &policies, &all_foreign);
        assert_eq!(
            without_ownerships, with_foreign_ownerships,
            "pin: ownership is never consulted - the frontier answer is \
             identical with and without foreign ownership"
        );
        assert_eq!(without_ownerships, vec![PathBuf::from("mid.h")]);
    }
}
