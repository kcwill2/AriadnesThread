use crate::{CallGraph, IncludeGraph, ReducedSemanticModel};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraversalDirection {
    Incoming,
    Outgoing,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphSelection<T> {
    pub nodes: Vec<T>,
    pub edges: Vec<(T, T)>,
}

/// AR-09 (plan audit P4): per-call out/in neighbor maps for one
/// directional BFS, built from a single O(E) pass instead of rescanning
/// the whole edge list at every dequeued node.
///
/// Deliberately NOT `IncludeGraph::adjacency()`: the neighbor lists here
/// preserve EDGE LIST ORDER and keep duplicates, because
/// `shortest_include_path`/`shortest_call_path` reconstruct their path
/// through `parent`, which is set by first arrival - iterating sorted
/// neighbors would re-parent equal-distance targets and change the
/// returned path (locked byte-for-byte by
/// `ar09_adjacency_equivalence_test::shortest_include_path_snapshot_locks`).
/// The sorted/deduped `adjacency()` stays the single builder for closure
/// semantics (`IncludeGraph::direct_includes`'s cached view).
#[derive(Debug)]
struct DirectionalAdjacency<N> {
    outgoing: HashMap<N, Vec<N>>,
    incoming: HashMap<N, Vec<N>>,
}

impl<N: Eq + std::hash::Hash + Clone> DirectionalAdjacency<N> {
    /// One pass over the edge stream; fills only the halves the direction
    /// actually consumes. An edge whose both endpoints equal the current
    /// node (self-loop) yields the same single neighbor either way,
    /// matching the retired per-edge scan's overwrite behavior.
    fn from_edges(
        edges: impl Iterator<Item = (N, N)>,
        direction: TraversalDirection,
    ) -> Self {
        let wants_outgoing = direction != TraversalDirection::Incoming;
        let wants_incoming = direction != TraversalDirection::Outgoing;
        let mut adjacency = Self {
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
        };
        for (src, dst) in edges {
            if wants_outgoing {
                adjacency.outgoing.entry(src.clone()).or_default().push(dst.clone());
            }
            if wants_incoming {
                adjacency.incoming.entry(dst).or_default().push(src);
            }
        }
        adjacency
    }
}

/// Shared depth-bounded expansion used by both graph neighborhood BFSs:
/// walks the direction's neighbor lists exactly as the retired
/// per-node edge scans did (same membership; final results are sorted,
/// so neighbor order cannot leak into the output).
fn expand_directional_neighbors<N: Clone + Eq + std::hash::Hash>(
    adjacency: &DirectionalAdjacency<N>,
    direction: TraversalDirection,
    current: &N,
    distance: usize,
    visited: &mut std::collections::HashSet<N>,
    queue: &mut std::collections::VecDeque<(N, usize)>,
) {
    let mut visit = |node: &N| {
        if visited.insert(node.clone()) {
            queue.push_back((node.clone(), distance + 1));
        }
    };
    if direction == TraversalDirection::Outgoing || direction == TraversalDirection::Both {
        if let Some(nexts) = adjacency.outgoing.get(current) {
            for next in nexts {
                visit(next);
            }
        }
    }
    if direction == TraversalDirection::Incoming || direction == TraversalDirection::Both {
        if let Some(nexts) = adjacency.incoming.get(current) {
            for next in nexts {
                visit(next);
            }
        }
    }
}

impl IncludeGraph {
    /// Returns the depth-`depth` neighborhood of `center` in the given
    /// direction. AR-09: expansion is driven by one per-call
    /// [`DirectionalAdjacency`] build (O(E) once) instead of rescanning
    /// `self.edges` for every dequeued node (was O(V_visited * E)); the
    /// induced-edge pass below stays as-is (single O(E) scan). Output was and
    /// is order-normalized (sorted nodes/edges), so the change cannot move
    /// results - locked exhaustively against the frozen scan by
    /// `ar09_adjacency_equivalence_test`.
    pub fn traverse_neighborhood(
        &self,
        center: FileId,
        depth: usize,
        direction: TraversalDirection,
    ) -> SubgraphSelection<FileId> {
        use std::collections::{HashSet, VecDeque};

        let adjacency = DirectionalAdjacency::from_edges(
            self.edges
                .iter()
                .filter_map(|edge| edge.destination.map(|dest| (edge.source, dest))),
            direction,
        );

        let mut visited_nodes = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((center, 0));
        visited_nodes.insert(center);

        while let Some((curr, d)) = queue.pop_front() {
            if d >= depth {
                continue;
            }

            expand_directional_neighbors(
                &adjacency,
                direction,
                &curr,
                d,
                &mut visited_nodes,
                &mut queue,
            );
        }

        let mut edges = HashSet::new();
        for edge in &self.edges {
            if let Some(dest) = edge.destination {
                if visited_nodes.contains(&edge.source) && visited_nodes.contains(&dest) {
                    edges.insert((edge.source, dest));
                }
            }
        }

        let mut nodes: Vec<_> = visited_nodes.into_iter().collect();
        nodes.sort();
        let mut edges_vec: Vec<_> = edges.into_iter().collect();
        edges_vec.sort();

        SubgraphSelection {
            nodes,
            edges: edges_vec,
        }
    }

    /// BFS shortest include path from `from` to `to` (outgoing edges only).
    /// AR-09: neighbors come from one per-call [`DirectionalAdjacency`]
    /// build instead of an O(E) scan per dequeued node (was
    /// O(V_visited * E)). The build preserves EDGE ORDER (no sort, no
    /// dedup), so `parent` is set by the same first arrival the retired
    /// scan produced and the reconstructed path is byte-identical -
    /// including the tie fixture where insertion order (5 before 3) picks
    /// the parent of an equal-distance target.
    pub fn shortest_include_path(&self, from: FileId, to: FileId) -> Option<Vec<FileId>> {
        use std::collections::{HashSet, VecDeque};

        if from == to {
            return Some(vec![from]);
        }

        let adjacency = DirectionalAdjacency::from_edges(
            self.edges
                .iter()
                .filter_map(|edge| edge.destination.map(|dest| (edge.source, dest))),
            TraversalDirection::Outgoing,
        );

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(from);
        visited.insert(from);

        while let Some(curr) = queue.pop_front() {
            if curr == to {
                let mut path = Vec::new();
                let mut current = curr;
                while let Some(&p) = parent.get(&current) {
                    path.push(current);
                    current = p;
                }
                path.push(from);
                path.reverse();
                return Some(path);
            }

            if let Some(nexts) = adjacency.outgoing.get(&curr) {
                for &dest in nexts {
                    if visited.insert(dest) {
                        parent.insert(dest, curr);
                        queue.push_back(dest);
                    }
                }
            }
        }

        None
    }

    pub fn get_entrypoints(&self, limit: usize) -> (Vec<FileId>, Vec<FileId>, Vec<FileId>) {
        use std::collections::HashMap;

        let mut in_degree = HashMap::new();
        let mut out_degree = HashMap::new();
        let mut nodes = std::collections::HashSet::new();

        // Also include all files registered
        for &id in self.file_paths.keys() {
            nodes.insert(id);
        }

        for edge in &self.edges {
            nodes.insert(edge.source);
            if let Some(dest) = edge.destination {
                nodes.insert(dest);
                *out_degree.entry(edge.source).or_insert(0) += 1;
                *in_degree.entry(dest).or_insert(0) += 1;
            }
        }

        let mut all_nodes: Vec<_> = nodes.into_iter().collect();
        all_nodes.sort();

        let mut roots = Vec::new();
        let mut leaves = Vec::new();
        let mut hubs = all_nodes.clone();

        for &n in &all_nodes {
            if *in_degree.get(&n).unwrap_or(&0) == 0 {
                roots.push(n);
            }
            if *out_degree.get(&n).unwrap_or(&0) == 0 {
                leaves.push(n);
            }
        }

        hubs.sort_by(|a, b| {
            let deg_a = in_degree.get(a).unwrap_or(&0) + out_degree.get(a).unwrap_or(&0);
            let deg_b = in_degree.get(b).unwrap_or(&0) + out_degree.get(b).unwrap_or(&0);
            deg_b.cmp(&deg_a).then_with(|| a.cmp(b))
        });

        roots.truncate(limit);
        leaves.truncate(limit);
        hubs.truncate(limit);

        (roots, leaves, hubs)
    }
}

impl ReducedSemanticModel {
    pub fn traverse_neighborhood(
        &self,
        center: &SymbolId,
        depth: usize,
        direction: TraversalDirection,
    ) -> SubgraphSelection<SymbolId> {
        use std::collections::{HashSet, VecDeque};

        let mut visited_nodes = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((center.clone(), 0));
        visited_nodes.insert(center.clone());

        while let Some((curr, d)) = queue.pop_front() {
            if d >= depth {
                continue;
            }

            for edge in &self.edges {
                let mut next_node = None;

                if (direction == TraversalDirection::Outgoing
                    || direction == TraversalDirection::Both)
                    && edge.src_symbol == curr
                {
                    next_node = Some(edge.dst_symbol.clone());
                }

                if (direction == TraversalDirection::Incoming
                    || direction == TraversalDirection::Both)
                    && edge.dst_symbol == curr
                {
                    next_node = Some(edge.src_symbol.clone());
                }

                if let Some(n) = next_node {
                    if !visited_nodes.contains(&n) {
                        visited_nodes.insert(n.clone());
                        queue.push_back((n, d + 1));
                    }
                }
            }
        }

        let mut edges = HashSet::new();
        for edge in &self.edges {
            if visited_nodes.contains(&edge.src_symbol) && visited_nodes.contains(&edge.dst_symbol)
            {
                edges.insert((edge.src_symbol.clone(), edge.dst_symbol.clone()));
            }
        }

        let mut nodes: Vec<_> = visited_nodes.into_iter().collect();
        nodes.sort();
        let mut edges_vec: Vec<_> = edges.into_iter().collect();
        edges_vec.sort();

        SubgraphSelection {
            nodes,
            edges: edges_vec,
        }
    }

    pub fn shortest_dependency_path(
        &self,
        from: &SymbolId,
        to: &SymbolId,
    ) -> Option<Vec<SymbolId>> {
        use std::collections::{HashMap, HashSet, VecDeque};

        if from == to {
            return Some(vec![from.clone()]);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent: HashMap<SymbolId, SymbolId> = HashMap::new();

        queue.push_back(from.clone());
        visited.insert(from.clone());

        while let Some(curr) = queue.pop_front() {
            if curr == *to {
                let mut path = Vec::new();
                let mut current = curr;
                while let Some(p) = parent.get(&current) {
                    path.push(current.clone());
                    current = p.clone();
                }
                path.push(from.clone());
                path.reverse();
                return Some(path);
            }

            for edge in &self.edges {
                if edge.src_symbol == curr {
                    let dest = edge.dst_symbol.clone();
                    if !visited.contains(&dest) {
                        visited.insert(dest.clone());
                        parent.insert(dest.clone(), curr.clone());
                        queue.push_back(dest);
                    }
                }
            }
        }

        None
    }

    pub fn get_entrypoints(&self, limit: usize) -> (Vec<SymbolId>, Vec<SymbolId>, Vec<SymbolId>) {
        use std::collections::HashMap;

        let mut in_degree = HashMap::new();
        let mut out_degree = HashMap::new();
        let mut nodes = std::collections::HashSet::new();

        for id in self.symbols.keys() {
            nodes.insert(id.clone());
        }

        for edge in &self.edges {
            nodes.insert(edge.src_symbol.clone());
            nodes.insert(edge.dst_symbol.clone());
            *out_degree.entry(edge.src_symbol.clone()).or_insert(0) += 1;
            *in_degree.entry(edge.dst_symbol.clone()).or_insert(0) += 1;
        }

        let mut all_nodes: Vec<_> = nodes.into_iter().collect();
        all_nodes.sort();

        let mut roots = Vec::new();
        let mut leaves = Vec::new();
        let mut hubs = all_nodes.clone();

        for n in &all_nodes {
            if *in_degree.get(n).unwrap_or(&0) == 0 {
                roots.push(n.clone());
            }
            if *out_degree.get(n).unwrap_or(&0) == 0 {
                leaves.push(n.clone());
            }
        }

        hubs.sort_by(|a, b| {
            let deg_a = in_degree.get(a).unwrap_or(&0) + out_degree.get(a).unwrap_or(&0);
            let deg_b = in_degree.get(b).unwrap_or(&0) + out_degree.get(b).unwrap_or(&0);
            deg_b.cmp(&deg_a).then_with(|| a.cmp(b))
        });

        roots.truncate(limit);
        leaves.truncate(limit);
        hubs.truncate(limit);

        (roots, leaves, hubs)
    }
}

impl CallGraph {
    /// Depth-`depth` call neighborhood of `center`. AR-09: same treatment
    /// as the include-graph neighborhood BFS - one per-call
    /// [`DirectionalAdjacency`] build (O(E)) replaces the per-dequeued-node
    /// edge scan (was O(V_visited * E)); output is order-normalized, so
    /// results are unchanged (locked in `ar09_adjacency_equivalence_test`).
    pub fn traverse_neighborhood(
        &self,
        center: &SymbolId,
        depth: usize,
        direction: TraversalDirection,
    ) -> SubgraphSelection<SymbolId> {
        use std::collections::{HashSet, VecDeque};

        let adjacency = DirectionalAdjacency::from_edges(
            self.edges
                .iter()
                .map(|edge| (edge.caller.clone(), edge.callee.clone())),
            direction,
        );

        let mut visited_nodes = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((center.clone(), 0));
        visited_nodes.insert(center.clone());

        while let Some((curr, d)) = queue.pop_front() {
            if d >= depth {
                continue;
            }

            expand_directional_neighbors(
                &adjacency,
                direction,
                &curr,
                d,
                &mut visited_nodes,
                &mut queue,
            );
        }

        let mut edges = HashSet::new();
        for edge in &self.edges {
            if visited_nodes.contains(&edge.caller) && visited_nodes.contains(&edge.callee) {
                edges.insert((edge.caller.clone(), edge.callee.clone()));
            }
        }

        let mut nodes: Vec<_> = visited_nodes.into_iter().collect();
        nodes.sort();
        let mut edges_vec: Vec<_> = edges.into_iter().collect();
        edges_vec.sort();

        SubgraphSelection {
            nodes,
            edges: edges_vec,
        }
    }

    pub fn get_entrypoints(&self, limit: usize) -> (Vec<SymbolId>, Vec<SymbolId>, Vec<SymbolId>) {
        use std::collections::HashMap;

        let mut in_degree = HashMap::new();
        let mut out_degree = HashMap::new();
        let mut nodes = std::collections::HashSet::new();

        for node in &self.nodes {
            nodes.insert(node.symbol_id.clone());
        }

        for edge in &self.edges {
            nodes.insert(edge.caller.clone());
            nodes.insert(edge.callee.clone());
            *out_degree.entry(edge.caller.clone()).or_insert(0) += 1;
            *in_degree.entry(edge.callee.clone()).or_insert(0) += 1;
        }

        let mut all_nodes: Vec<_> = nodes.into_iter().collect();
        all_nodes.sort();

        let mut roots = Vec::new();
        let mut leaves = Vec::new();
        let mut hubs = all_nodes.clone();

        for n in &all_nodes {
            if *in_degree.get(n).unwrap_or(&0) == 0 {
                roots.push(n.clone());
            }
            if *out_degree.get(n).unwrap_or(&0) == 0 {
                leaves.push(n.clone());
            }
        }

        hubs.sort_by(|a, b| {
            let deg_a = in_degree.get(a).unwrap_or(&0) + out_degree.get(a).unwrap_or(&0);
            let deg_b = in_degree.get(b).unwrap_or(&0) + out_degree.get(b).unwrap_or(&0);
            deg_b.cmp(&deg_a).then_with(|| a.cmp(b))
        });

        roots.truncate(limit);
        leaves.truncate(limit);
        hubs.truncate(limit);

        (roots, leaves, hubs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{IncludeEdge, IncludeKind, SourceLocation};
    use std::path::PathBuf;

    #[test]
    fn test_include_graph_traversal() {
        let mut graph = IncludeGraph::new();
        let f1 = FileId(1);
        let f2 = FileId(2);
        let f3 = FileId(3);
        graph.register_file(f1, PathBuf::from("a.h"));
        graph.register_file(f2, PathBuf::from("b.h"));
        graph.register_file(f3, PathBuf::from("c.h"));

        graph.add_edge(IncludeEdge {
            source: f1,
            destination: Some(f2),
            context_name: "test".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "b.h".into(),
            resolved_path: None,
            language: crate::types::LanguageId::cpp(),
        });

        graph.add_edge(IncludeEdge {
            source: f2,
            destination: Some(f3),
            context_name: "test".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "c.h".into(),
            resolved_path: None,
            language: crate::types::LanguageId::cpp(),
        });

        let hood = graph.traverse_neighborhood(f2, 1, TraversalDirection::Both);
        assert_eq!(hood.nodes.len(), 3);
        assert_eq!(hood.edges.len(), 2);

        let path = graph.shortest_include_path(f1, f3);
        assert_eq!(path, Some(vec![f1, f2, f3]));

        let (roots, leaves, hubs) = graph.get_entrypoints(1);
        assert_eq!(roots, vec![f1]);
        assert_eq!(leaves, vec![f3]);
        assert_eq!(hubs, vec![f2]); // f2 has degree 2
    }

    // UTA-T1 items 3-4 (hand H1): behavior pins for the ReducedSemanticModel
    // traversal surface. Chain A -> B -> C plus the B self-loop and the
    // edge-free registered symbol D.
    fn symbol(name: &str) -> SymbolId {
        SymbolId::new(name)
    }

    fn dependency(from: &str, to: &str) -> crate::ReducedSemanticEdge {
        crate::ReducedSemanticEdge {
            src_symbol: symbol(from),
            dst_symbol: symbol(to),
            kind: ariadne_index::types::DependencyKind::Incomplete,
        }
    }

    fn chain_selfloop_isolate_model() -> ReducedSemanticModel {
        let mut model = ReducedSemanticModel::default();
        for name in ["A", "B", "C", "D"] {
            model.symbols.insert(
                symbol(name),
                crate::ReducedSymbolRecord {
                    id: symbol(name),
                    name: name.to_string(),
                    kind: ariadne_index::types::SymbolKind::Function,
                    defining_file: None,
                    declarations: Vec::new(),
                    definition: None,
                    declaring_files: Vec::new(),
                },
            );
        }
        model.edges = vec![
            dependency("A", "B"),
            dependency("B", "C"),
            dependency("B", "B"),
        ];
        model
    }

    #[test]
    fn reduced_traverse_neighborhood_snapshots_directions_and_depths() {
        let model = chain_selfloop_isolate_model();
        let center = symbol("B");

        // Every direction at depth 0 yields the center alone - and the
        // center's own self-loop IS an induced edge of that one-node set.
        for direction in [
            TraversalDirection::Outgoing,
            TraversalDirection::Incoming,
            TraversalDirection::Both,
        ] {
            let hood = model.traverse_neighborhood(&center, 0, direction);
            assert_eq!(hood.nodes, vec![symbol("B")], "depth 0 {direction:?}");
            assert_eq!(
                hood.edges,
                vec![(symbol("B"), symbol("B"))],
                "depth 0 {direction:?} keeps the self-loop as an induced edge"
            );
        }

        for depth in [1usize, 2usize] {
            let outgoing =
                model.traverse_neighborhood(&center, depth, TraversalDirection::Outgoing);
            assert_eq!(
                outgoing.nodes,
                vec![symbol("B"), symbol("C")],
                "outgoing depth {depth}"
            );
            assert_eq!(
                outgoing.edges,
                vec![(symbol("B"), symbol("B")), (symbol("B"), symbol("C"))],
                "outgoing depth {depth}"
            );

            let incoming =
                model.traverse_neighborhood(&center, depth, TraversalDirection::Incoming);
            assert_eq!(
                incoming.nodes,
                vec![symbol("A"), symbol("B")],
                "incoming depth {depth}"
            );
            assert_eq!(
                incoming.edges,
                vec![(symbol("A"), symbol("B")), (symbol("B"), symbol("B"))],
                "incoming depth {depth}"
            );

            let both = model.traverse_neighborhood(&center, depth, TraversalDirection::Both);
            assert_eq!(both.nodes, vec![symbol("A"), symbol("B"), symbol("C")]);
            assert_eq!(
                both.edges,
                vec![
                    (symbol("A"), symbol("B")),
                    (symbol("B"), symbol("B")),
                    (symbol("B"), symbol("C")),
                ],
                "both depth {depth}"
            );
        }

        // The isolate is its own closed neighborhood at any direction/depth.
        for direction in [
            TraversalDirection::Outgoing,
            TraversalDirection::Incoming,
            TraversalDirection::Both,
        ] {
            let hood = model.traverse_neighborhood(&symbol("D"), 2, direction);
            assert_eq!(hood.nodes, vec![symbol("D")], "isolate {direction:?}");
            assert!(hood.edges.is_empty(), "isolate {direction:?}");
        }
    }

    #[test]
    fn reduced_shortest_dependency_path_locks_path_identity_and_misses() {
        let model = chain_selfloop_isolate_model();

        let a_to_c = model
            .shortest_dependency_path(&symbol("A"), &symbol("C"))
            .expect("A reaches C through B");
        assert_eq!(a_to_c, vec![symbol("A"), symbol("B"), symbol("C")]);

        // X == X short-circuits to [X] even without any self-edge.
        assert_eq!(
            model.shortest_dependency_path(&symbol("A"), &symbol("A")),
            Some(vec![symbol("A")])
        );

        // The isolate has no path in either direction.
        assert_eq!(
            model.shortest_dependency_path(&symbol("A"), &symbol("D")),
            None
        );
        assert_eq!(
            model.shortest_dependency_path(&symbol("D"), &symbol("A")),
            None
        );
    }

    // UTA-T1 item 10 (hand H1): get_entrypoints pins for all three
    // implementations - tie order (degree desc, then id asc), truncation,
    // the all-cycle graph (zero roots), and the unresolved-only graph
    // (leaf + root, no degrees).

    fn include_edge_file(
        source: FileId,
        destination: Option<FileId>,
        line: u32,
    ) -> IncludeEdge {
        IncludeEdge {
            source,
            destination,
            context_name: "entrypoints".into(),
            source_location: SourceLocation::new(line, 1),
            kind: IncludeKind::Local,
            raw_path: "target.h".into(),
            resolved_path: None,
            language: crate::types::LanguageId::cpp(),
        }
    }

    #[test]
    fn include_graph_entrypoints_lock_tie_order_and_truncation() {
        // 1 and 2 fan into 4 and 5; 3 is a registered isolate. Degrees:
        // 1, 2, 4, 5 all tie at 2 (ascending ids break the tie), 3 has 0.
        let mut graph = IncludeGraph::new();
        for id in 1..=5u64 {
            graph.register_file(FileId(id), PathBuf::from(format!("e{id}.h")));
        }
        graph.add_edge(include_edge_file(FileId(1), Some(FileId(4)), 1));
        graph.add_edge(include_edge_file(FileId(1), Some(FileId(5)), 2));
        graph.add_edge(include_edge_file(FileId(2), Some(FileId(4)), 3));
        graph.add_edge(include_edge_file(FileId(2), Some(FileId(5)), 4));

        let (roots, leaves, hubs) = graph.get_entrypoints(2);
        assert_eq!(roots, vec![FileId(1), FileId(2)], "roots tie-break asc");
        assert_eq!(leaves, vec![FileId(3), FileId(4)], "leaves tie-break asc");
        assert_eq!(hubs, vec![FileId(1), FileId(2)], "hubs: degree desc then asc");
    }

    #[test]
    fn include_graph_unresolved_only_edges_yield_root_and_leaf_without_degree() {
        // An unresolved edge (destination None) contributes its source as a
        // node but no degree: the file is simultaneously root and leaf.
        let mut graph = IncludeGraph::new();
        graph.register_file(FileId(9), PathBuf::from("solo.h"));
        graph.add_edge(include_edge_file(FileId(9), None, 1));

        let (roots, leaves, hubs) = graph.get_entrypoints(10);
        assert_eq!(roots, vec![FileId(9)]);
        assert_eq!(leaves, vec![FileId(9)]);
        assert_eq!(hubs, vec![FileId(9)]);
    }

    #[test]
    fn pin_parallel_directive_edges_count_toward_degree() {
        // Pin (semantics undecided, reported): a.h holds TWO include
        // directives for the same target b.h (distinct lines, so
        // `add_edge` keeps both records) and b.h includes c.h once.
        // `get_entrypoints` counts EDGE RECORDS, not distinct pairs: file 2
        // sits at degree 3 and outranks file 1 (degree 2). Under
        // distinct-pair degrees files 1 and 2 would tie at 2 and the id
        // tie-break would rank 1 first - the exact difference this pin
        // locks.
        let mut graph = IncludeGraph::new();
        for id in 1..=3u64 {
            graph.register_file(FileId(id), PathBuf::from(format!("p{id}.h")));
        }
        graph.add_edge(include_edge_file(FileId(1), Some(FileId(2)), 1));
        graph.add_edge(include_edge_file(FileId(1), Some(FileId(2)), 2));
        graph.add_edge(include_edge_file(FileId(2), Some(FileId(3)), 3));

        let (roots, leaves, hubs) = graph.get_entrypoints(10);
        assert_eq!(roots, vec![FileId(1)]);
        assert_eq!(leaves, vec![FileId(3)]);
        assert_eq!(
            hubs,
            vec![FileId(2), FileId(1), FileId(3)],
            "the duplicate directive inflates file 2's degree above file 1's"
        );
    }

    #[test]
    fn reduced_model_entrypoints_lock_tie_order_truncation_and_all_cycle() {
        // Tie order + truncation: s1/s2 fan into s4/s5, s0 registered but
        // edge-free - all four fanning symbols tie at degree 2.
        let mut model = ReducedSemanticModel::default();
        for name in ["s0", "s1", "s2", "s4", "s5"] {
            model.symbols.insert(
                symbol(name),
                crate::ReducedSymbolRecord {
                    id: symbol(name),
                    name: name.to_string(),
                    kind: ariadne_index::types::SymbolKind::Function,
                    defining_file: None,
                    declarations: Vec::new(),
                    definition: None,
                    declaring_files: Vec::new(),
                },
            );
        }
        model.edges = vec![
            dependency("s1", "s4"),
            dependency("s1", "s5"),
            dependency("s2", "s4"),
            dependency("s2", "s5"),
        ];

        let (roots, leaves, hubs) = model.get_entrypoints(2);
        assert_eq!(roots, vec![symbol("s0"), symbol("s1")]);
        assert_eq!(leaves, vec![symbol("s0"), symbol("s4")]);
        assert_eq!(hubs, vec![symbol("s1"), symbol("s2")]);

        // The all-cycle graph has no roots and no leaves.
        let mut cyclic = ReducedSemanticModel::default();
        for name in ["ca", "cb", "cc"] {
            cyclic.symbols.insert(
                symbol(name),
                crate::ReducedSymbolRecord {
                    id: symbol(name),
                    name: name.to_string(),
                    kind: ariadne_index::types::SymbolKind::Function,
                    defining_file: None,
                    declarations: Vec::new(),
                    definition: None,
                    declaring_files: Vec::new(),
                },
            );
        }
        cyclic.edges = vec![
            dependency("ca", "cb"),
            dependency("cb", "cc"),
            dependency("cc", "ca"),
        ];
        let (roots, leaves, hubs) = cyclic.get_entrypoints(10);
        assert!(roots.is_empty(), "every node has inbound edges");
        assert!(leaves.is_empty(), "every node has outbound edges");
        assert_eq!(hubs, vec![symbol("ca"), symbol("cb"), symbol("cc")]);
    }

    #[test]
    fn call_graph_entrypoints_lock_tie_order_truncation_and_all_cycle() {
        // Tie order + truncation: c1/c2 call t4/t5, node n0 is a registered
        // node with no edges (CallGraph::get_entrypoints seeds from `nodes`).
        let call_node = |name: &str| crate::CallNode {
            symbol_id: symbol(name),
            name: name.to_string(),
            owning_file: FileId(7),
            location: None,
        };
        let call_graph = |calls: &[(&str, &str)], extras: &[&str]| {
            let mut graph = CallGraph {
                nodes: extras
                    .iter()
                    .map(|name| call_node(name))
                    .collect(),
                edges: calls
                    .iter()
                    .map(|(caller, callee)| crate::CallEdge {
                        caller: symbol(caller),
                        callee: symbol(callee),
                        call_count: 1,
                        call_locations: Vec::new(),
                    })
                    .collect(),
            };
            for (caller, callee) in calls {
                graph.nodes.push(call_node(caller));
                graph.nodes.push(call_node(callee));
            }
            graph
        };

        let (roots, leaves, hubs) = call_graph(
            &[("c1", "t4"), ("c1", "t5"), ("c2", "t4"), ("c2", "t5")],
            &["n0"],
        )
        .get_entrypoints(2);
        assert_eq!(roots, vec![symbol("c1"), symbol("c2")]);
        assert_eq!(leaves, vec![symbol("n0"), symbol("t4")]);
        assert_eq!(hubs, vec![symbol("c1"), symbol("c2")]);

        let (roots, leaves, hubs) =
            call_graph(&[("ra", "rb"), ("rb", "rc"), ("rc", "ra")], &[]).get_entrypoints(10);
        assert!(roots.is_empty(), "every caller is also called");
        assert!(leaves.is_empty(), "every callee also calls");
        assert_eq!(hubs, vec![symbol("ra"), symbol("rb"), symbol("rc")]);
    }
}
