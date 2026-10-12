//! AR-09 equivalence suite: adjacency-driven scans must reproduce the
//! edge-list-scan results byte-for-byte (node order, edge order, paths).
//!
//! Characterization discipline: the `oracle_*` functions below are verbatim
//! copies of the CURRENT edge-list scans (`direct_includes`,
//! `traverse_neighborhood`, `shortest_include_path`, the CallGraph BFS and
//! `shortest_call_path`) operating on pub fields. They are the definition of
//! correct for this suite: every fixture x query cell is cross-checked
//! against them, and the snapshot tests pin hand-derived literals so a
//! future change to oracle and implementation together cannot silently move
//! the contract. The cache-interaction tests guard the AR-09 lazy adjacency
//! cache against staleness (AR-06 `seen_edges` sibling contract).

use ariadne_graph::{
    CallEdge, CallGraph, CallNode, IncludeEdge, IncludeGraph, IncludeKind, LanguageId,
    SourceLocation, SubgraphSelection, TraversalDirection,
};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use std::path::PathBuf;

const ALL_DIRECTIONS: [TraversalDirection; 3] = [
    TraversalDirection::Outgoing,
    TraversalDirection::Incoming,
    TraversalDirection::Both,
];

fn edge(source: u64, destination: u64, line: u32) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: Some(FileId(destination)),
        context_name: "Default".into(),
        source_location: SourceLocation::new(line, 1),
        kind: IncludeKind::Local,
        raw_path: format!("dep{destination}.h"),
        resolved_path: None,
        language: LanguageId::cpp(),
    }
}

fn unresolved_edge(source: u64, line: u32) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: None,
        context_name: "Default".into(),
        source_location: SourceLocation::new(line, 1),
        kind: IncludeKind::System,
        raw_path: "missing.h".into(),
        resolved_path: None,
        language: LanguageId::cpp(),
    }
}

fn register(graph: &mut IncludeGraph, ids: &[u64]) {
    for &id in ids {
        graph.register_file(FileId(id), PathBuf::from(format!("f{id}.h")));
    }
}

fn chain_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    register(&mut graph, &[1, 2, 3, 4]);
    graph.add_edge(edge(1, 2, 1));
    graph.add_edge(edge(2, 3, 1));
    graph.add_edge(edge(3, 4, 1));
    graph
}

fn diamond_graph() -> IncludeGraph {
    // Includes the AR-06 distinct-metadata twin: 1->3 twice on different
    // lines stays two edges, so adjacency dedup (not edge dedup) is locked.
    let mut graph = IncludeGraph::new();
    register(&mut graph, &[1, 2, 3, 4]);
    graph.add_edge(edge(1, 2, 1));
    graph.add_edge(edge(1, 3, 1));
    graph.add_edge(edge(2, 4, 1));
    graph.add_edge(edge(3, 4, 1));
    graph.add_edge(edge(1, 3, 9));
    graph
}

fn cycle_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    register(&mut graph, &[1, 2, 3]);
    graph.add_edge(edge(1, 2, 1));
    graph.add_edge(edge(2, 3, 1));
    graph.add_edge(edge(3, 1, 1));
    graph
}

fn disconnected_graph() -> IncludeGraph {
    // Two islands, an isolated registered file, an unresolved edge and a
    // self-loop: every shape that must stay invisible/visible identically.
    let mut graph = IncludeGraph::new();
    register(&mut graph, &[1, 2, 3, 4, 5]);
    graph.add_edge(edge(1, 2, 1));
    graph.add_edge(edge(3, 4, 1));
    graph.add_edge(unresolved_edge(2, 2));
    graph.add_edge(edge(2, 2, 3));
    graph
}

fn tie_order_graph() -> IncludeGraph {
    // Byte-identity hazard for shortest-path BFS: node 1 gains edges to 5
    // BEFORE 3, and both reach 7 in one hop. The edge-list scan parents 7
    // under 5 (edge order); a sorted adjacency would parent it under 3.
    // This fixture fails loudly if a rewrite iterates sorted neighbors.
    let mut graph = IncludeGraph::new();
    register(&mut graph, &[1, 3, 5, 7]);
    graph.add_edge(edge(1, 5, 1));
    graph.add_edge(edge(1, 3, 1));
    graph.add_edge(edge(5, 7, 1));
    graph.add_edge(edge(3, 7, 1));
    graph
}

fn include_fixtures() -> Vec<(&'static str, IncludeGraph)> {
    vec![
        ("chain", chain_graph()),
        ("diamond", diamond_graph()),
        ("cycle", cycle_graph()),
        ("disconnected", disconnected_graph()),
        ("tie_order", tie_order_graph()),
    ]
}

fn fixture_probe_ids() -> Vec<u64> {
    vec![1, 2, 3, 4, 5, 7, 99]
}

fn call_graph_fixture() -> CallGraph {
    // Literal-built (BTreeMap order NOT assumed): edges are inserted in a
    // deliberately unsorted order so the `shortest_call_path` neighbor sort
    // and the neighborhood set semantics are both exercised. D is an
    // isolated node; C has a self-call.
    let owning = FileId(1);
    let node = |name: &str| CallNode {
        symbol_id: SymbolId::new(name),
        name: name.into(),
        owning_file: owning.clone(),
        location: None,
    };
    let call = |caller: &str, callee: &str| CallEdge {
        caller: SymbolId::new(caller),
        callee: SymbolId::new(callee),
        call_count: 1,
        call_locations: Vec::new(),
    };
    CallGraph {
        nodes: vec![node("A"), node("B"), node("C"), node("D")],
        edges: vec![
            call("C", "A"),
            call("A", "B"),
            call("C", "C"),
            call("B", "C"),
            call("B", "D"),
        ],
    }
}

fn symbol_names(ids: &[SymbolId]) -> Vec<String> {
    ids.iter().map(|id| id.0.clone()).collect()
}

fn symbol_pairs(pairs: &[(SymbolId, SymbolId)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(src, dst)| (src.0.clone(), dst.0.clone()))
        .collect()
}

fn file_ids(ids: &[FileId]) -> Vec<u64> {
    ids.iter().map(|id| id.0).collect()
}

fn file_pairs(pairs: &[(FileId, FileId)]) -> Vec<(u64, u64)> {
    pairs.iter().map(|(a, b)| (a.0, b.0)).collect()
}

// --- frozen oracles: verbatim current scan semantics ---------------------

fn oracle_direct_includes(graph: &IncludeGraph, file_id: FileId) -> Vec<FileId> {
    let mut direct = Vec::new();
    for edge in &graph.edges {
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

fn oracle_traverse_neighborhood(
    graph: &IncludeGraph,
    center: FileId,
    depth: usize,
    direction: TraversalDirection,
) -> SubgraphSelection<FileId> {
    use std::collections::{HashSet, VecDeque};

    let mut visited_nodes = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back((center, 0));
    visited_nodes.insert(center);

    while let Some((curr, d)) = queue.pop_front() {
        if d >= depth {
            continue;
        }

        for edge in &graph.edges {
            if let Some(dest) = edge.destination {
                let mut next_node = None;

                if (direction == TraversalDirection::Outgoing
                    || direction == TraversalDirection::Both)
                    && edge.source == curr
                {
                    next_node = Some(dest);
                }

                if (direction == TraversalDirection::Incoming
                    || direction == TraversalDirection::Both)
                    && dest == curr
                {
                    next_node = Some(edge.source);
                }

                if let Some(n) = next_node {
                    if !visited_nodes.contains(&n) {
                        visited_nodes.insert(n);
                        queue.push_back((n, d + 1));
                    }
                }
            }
        }
    }

    let mut edges = HashSet::new();
    for edge in &graph.edges {
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

fn oracle_shortest_include_path(
    graph: &IncludeGraph,
    from: FileId,
    to: FileId,
) -> Option<Vec<FileId>> {
    use std::collections::{HashMap, HashSet, VecDeque};

    if from == to {
        return Some(vec![from]);
    }

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

        for edge in &graph.edges {
            if edge.source == curr {
                if let Some(dest) = edge.destination {
                    if !visited.contains(&dest) {
                        visited.insert(dest);
                        parent.insert(dest, curr);
                        queue.push_back(dest);
                    }
                }
            }
        }
    }

    None
}

fn oracle_callgraph_traverse(
    graph: &CallGraph,
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

        for edge in &graph.edges {
            let mut next_node = None;

            if (direction == TraversalDirection::Outgoing
                || direction == TraversalDirection::Both)
                && edge.caller == curr
            {
                next_node = Some(edge.callee.clone());
            }

            if (direction == TraversalDirection::Incoming
                || direction == TraversalDirection::Both)
                && edge.callee == curr
            {
                next_node = Some(edge.caller.clone());
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
    for edge in &graph.edges {
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

fn oracle_shortest_call_path(
    graph: &CallGraph,
    from: &SymbolId,
    to: &SymbolId,
) -> Option<Vec<SymbolId>> {
    use std::collections::{HashMap, HashSet, VecDeque};

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut parent: HashMap<SymbolId, SymbolId> = HashMap::new();

    queue.push_back(from.clone());
    visited.insert(from.clone());

    while let Some(current) = queue.pop_front() {
        if current == *to {
            let mut path = Vec::new();
            let mut curr = current.clone();
            while let Some(p) = parent.get(&curr) {
                path.push(curr.clone());
                curr = p.clone();
            }
            path.push(from.clone());
            path.reverse();
            return Some(path);
        }

        let mut neighbors: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.caller == current)
            .map(|e| e.callee.clone())
            .collect();
        neighbors.sort();

        for next in neighbors {
            if !visited.contains(&next) {
                visited.insert(next.clone());
                parent.insert(next.clone(), current.clone());
                queue.push_back(next);
            }
        }
    }

    None
}

// --- exhaustive frozen-scan equivalence -----------------------------------

#[test]
fn direct_includes_matches_frozen_scan_on_all_fixtures() {
    for (name, graph) in include_fixtures() {
        for id in fixture_probe_ids() {
            assert_eq!(
                graph.direct_includes(FileId(id)),
                oracle_direct_includes(&graph, FileId(id)),
                "direct_includes({name}, {id}) diverged from the frozen scan"
            );
        }
    }
}

#[test]
fn traverse_neighborhood_matches_frozen_scan_on_all_fixtures() {
    for (name, graph) in include_fixtures() {
        for id in fixture_probe_ids() {
            for depth in 0..=4usize {
                for direction in ALL_DIRECTIONS {
                    assert_eq!(
                        graph.traverse_neighborhood(FileId(id), depth, direction),
                        oracle_traverse_neighborhood(&graph, FileId(id), depth, direction),
                        "traverse_neighborhood({name}, center={id}, depth={depth}, \
                         direction={direction:?}) diverged from the frozen scan"
                    );
                }
            }
        }
    }
}

#[test]
fn shortest_include_path_matches_frozen_scan_on_all_fixtures() {
    for (name, graph) in include_fixtures() {
        for from in fixture_probe_ids() {
            for to in fixture_probe_ids() {
                assert_eq!(
                    graph.shortest_include_path(FileId(from), FileId(to)),
                    oracle_shortest_include_path(&graph, FileId(from), FileId(to)),
                    "shortest_include_path({name}, {from} -> {to}) diverged \
                     from the frozen scan"
                );
            }
        }
    }
}

#[test]
fn callgraph_traversal_matches_frozen_scan() {
    let graph = call_graph_fixture();
    let centers = ["A", "B", "C", "D", "missing"];
    for center in centers {
        let center = SymbolId::new(center);
        for depth in 0..=4usize {
            for direction in ALL_DIRECTIONS {
                assert_eq!(
                    graph.traverse_neighborhood(&center, depth, direction),
                    oracle_callgraph_traverse(&graph, &center, depth, direction),
                    "CallGraph::traverse_neighborhood({center:?}, depth={depth}, \
                     direction={direction:?}) diverged from the frozen scan"
                );
            }
        }
    }
}

#[test]
fn callgraph_shortest_path_matches_frozen_scan() {
    let graph = call_graph_fixture();
    let probes = ["A", "B", "C", "D", "missing"];
    for from in probes {
        for to in probes {
            let (from, to) = (SymbolId::new(from), SymbolId::new(to));
            assert_eq!(
                graph.shortest_call_path(&from, &to),
                oracle_shortest_call_path(&graph, &from, &to),
                "shortest_call_path({from:?} -> {to:?}) diverged from the frozen scan"
            );
        }
    }
}

// --- hand-derived snapshot literals (order byte-locks) --------------------

#[test]
fn direct_includes_snapshot_locks() {
    let chain = chain_graph();
    assert_eq!(file_ids(&chain.direct_includes(FileId(1))), vec![2]);
    assert_eq!(file_ids(&chain.direct_includes(FileId(4))), Vec::<u64>::new());
    assert_eq!(
        file_ids(&chain.direct_includes(FileId(99))),
        Vec::<u64>::new(),
        "missing node must yield the empty list"
    );

    let diamond = diamond_graph();
    assert_eq!(
        file_ids(&diamond.direct_includes(FileId(1))),
        vec![2, 3],
        "duplicate 1->3 edges collapse to one destination, sorted"
    );

    let cycle = cycle_graph();
    assert_eq!(file_ids(&cycle.direct_includes(FileId(3))), vec![1]);

    let disconnected = disconnected_graph();
    assert_eq!(file_ids(&disconnected.direct_includes(FileId(2))), vec![2]);
    assert_eq!(
        file_ids(&disconnected.direct_includes(FileId(4))),
        Vec::<u64>::new(),
        "unresolved edges must not contribute destinations"
    );

    // Sorted output even when edges were ADDED out of id order.
    let tie = tie_order_graph();
    assert_eq!(
        file_ids(&tie.direct_includes(FileId(1))),
        vec![3, 5],
        "direct_includes sorts regardless of edge insertion order"
    );
}

#[test]
fn traverse_neighborhood_snapshot_locks() {
    use TraversalDirection::{Both, Incoming, Outgoing};

    let chain = chain_graph();
    let hood = chain.traverse_neighborhood(FileId(2), 1, Outgoing);
    assert_eq!(file_ids(&hood.nodes), vec![2, 3]);
    assert_eq!(file_pairs(&hood.edges), vec![(2, 3)]);

    let hood = chain.traverse_neighborhood(FileId(2), 1, Incoming);
    assert_eq!(file_ids(&hood.nodes), vec![1, 2]);
    assert_eq!(file_pairs(&hood.edges), vec![(1, 2)]);

    let hood = chain.traverse_neighborhood(FileId(2), 2, Both);
    assert_eq!(file_ids(&hood.nodes), vec![1, 2, 3, 4]);
    assert_eq!(file_pairs(&hood.edges), vec![(1, 2), (2, 3), (3, 4)]);

    let hood = chain.traverse_neighborhood(FileId(1), 0, Both);
    assert_eq!(file_ids(&hood.nodes), vec![1]);
    assert_eq!(file_pairs(&hood.edges), Vec::new(), "depth 0 expands nothing");

    // Cycle: at depth 1 Both all three nodes are visited, so ALL three edges
    // (including the never-expanded 2->3) appear in the induced edge set.
    let cycle = cycle_graph();
    let hood = cycle.traverse_neighborhood(FileId(1), 1, Both);
    assert_eq!(file_ids(&hood.nodes), vec![1, 2, 3]);
    assert_eq!(file_pairs(&hood.edges), vec![(1, 2), (2, 3), (3, 1)]);

    // Disconnected: self-loop is its own in- and out-neighbor and shows up
    // in a depth-0 induced edge set over the single visited node.
    let disconnected = disconnected_graph();
    let hood = disconnected.traverse_neighborhood(FileId(2), 0, Both);
    assert_eq!(file_ids(&hood.nodes), vec![2]);
    assert_eq!(file_pairs(&hood.edges), vec![(2, 2)]);

    let hood = disconnected.traverse_neighborhood(FileId(2), 1, Incoming);
    assert_eq!(file_ids(&hood.nodes), vec![1, 2]);
    assert_eq!(file_pairs(&hood.edges), vec![(1, 2), (2, 2)]);

    let hood = disconnected.traverse_neighborhood(FileId(99), 4, Both);
    assert_eq!(file_ids(&hood.nodes), vec![99], "missing center yields itself");
    assert_eq!(file_pairs(&hood.edges), Vec::new());
}

#[test]
fn shortest_include_path_snapshot_locks() {
    let chain = chain_graph();
    assert_eq!(
        file_ids(&chain.shortest_include_path(FileId(1), FileId(4)).unwrap()),
        vec![1, 2, 3, 4]
    );
    assert_eq!(chain.shortest_include_path(FileId(4), FileId(1)), None);
    assert_eq!(
        file_ids(&chain.shortest_include_path(FileId(2), FileId(2)).unwrap()),
        vec![2]
    );
    assert_eq!(chain.shortest_include_path(FileId(99), FileId(1)), None);
    assert_eq!(chain.shortest_include_path(FileId(1), FileId(99)), None);

    let diamond = diamond_graph();
    assert_eq!(
        file_ids(&diamond.shortest_include_path(FileId(1), FileId(4)).unwrap()),
        vec![1, 2, 4],
        "first enqueued predecessor (edge order 1->2 before 1->3) wins"
    );

    let cycle = cycle_graph();
    assert_eq!(
        file_ids(&cycle.shortest_include_path(FileId(3), FileId(2)).unwrap()),
        vec![3, 1, 2]
    );

    // THE byte-identity lock against sorted-adjacency rewrites: 5 was
    // inserted before 3, so the scan parents 7 under 5 and returns 1->5->7.
    let tie = tie_order_graph();
    assert_eq!(
        file_ids(&tie.shortest_include_path(FileId(1), FileId(7)).unwrap()),
        vec![1, 5, 7],
        "BFS parent must follow EDGE INSERTION order, not id order"
    );

    let disconnected = disconnected_graph();
    assert_eq!(disconnected.shortest_include_path(FileId(1), FileId(4)), None);
    assert_eq!(
        file_ids(&disconnected.shortest_include_path(FileId(2), FileId(2)).unwrap()),
        vec![2],
        "from == to short-circuits to the singleton"
    );
}

#[test]
fn callgraph_traversal_snapshot_locks() {
    use TraversalDirection::{Both, Incoming, Outgoing};

    let graph = call_graph_fixture();
    let center_c = SymbolId::new("C");
    let center_d = SymbolId::new("D");

    // Self-call: the depth-0 induced edge set over {C} already carries C->C.
    let hood = graph.traverse_neighborhood(&center_c, 0, Both);
    assert_eq!(symbol_names(&hood.nodes), vec!["C"]);
    assert_eq!(symbol_pairs(&hood.edges), vec![("C".into(), "C".into())]);

    let hood = graph.traverse_neighborhood(&center_c, 1, Outgoing);
    assert_eq!(symbol_names(&hood.nodes), vec!["A", "C"]);
    assert_eq!(
        symbol_pairs(&hood.edges),
        vec![("C".into(), "A".into()), ("C".into(), "C".into())]
    );

    let hood = graph.traverse_neighborhood(&center_c, 1, Incoming);
    assert_eq!(symbol_names(&hood.nodes), vec!["B", "C"]);
    assert_eq!(
        symbol_pairs(&hood.edges),
        vec![("B".into(), "C".into()), ("C".into(), "C".into())]
    );

    let hood = graph.traverse_neighborhood(&center_c, 1, Both);
    assert_eq!(symbol_names(&hood.nodes), vec!["A", "B", "C"]);
    assert_eq!(
        symbol_pairs(&hood.edges),
        vec![
            ("A".into(), "B".into()),
            ("B".into(), "C".into()),
            ("C".into(), "A".into()),
            ("C".into(), "C".into())
        ]
    );

    let hood = graph.traverse_neighborhood(&center_c, 2, Both);
    assert_eq!(symbol_names(&hood.nodes), vec!["A", "B", "C", "D"]);
    assert_eq!(hood.edges.len(), 5);

    // "Isolated" node D has the incoming (B,D): Both propagates through it
    // and the whole graph is reached; a truly edgeless center stays alone.
    let hood = graph.traverse_neighborhood(&center_d, 4, Both);
    assert_eq!(symbol_names(&hood.nodes), vec!["A", "B", "C", "D"]);
    assert_eq!(hood.edges.len(), 5);

    let hood = graph.traverse_neighborhood(&SymbolId::new("missing"), 4, Both);
    assert_eq!(symbol_names(&hood.nodes), vec!["missing"]);
    assert_eq!(symbol_pairs(&hood.edges), Vec::new());
}

#[test]
fn callgraph_shortest_path_snapshot_locks() {
    let graph = call_graph_fixture();
    let path = |from: &str, to: &str| {
        graph
            .shortest_call_path(&SymbolId::new(from), &SymbolId::new(to))
            .map(|ids| symbol_names(&ids))
    };
    assert_eq!(path("A", "B"), Some(vec!["A".into(), "B".into()]));
    assert_eq!(path("A", "C"), Some(vec!["A".into(), "B".into(), "C".into()]));
    assert_eq!(
        path("C", "D"),
        Some(vec!["C".into(), "A".into(), "B".into(), "D".into()])
    );
    assert_eq!(path("D", "A"), None, "isolated caller reaches nothing");
    assert_eq!(path("missing", "A"), None);
    assert_eq!(path("A", "missing"), None);
    assert_eq!(path("A", "A"), Some(vec!["A".into()]), "from == to");
}

// --- adjacency cache interaction locks (AR-09 staleness guards) -----------

#[test]
fn results_refresh_after_add_edge() {
    // Post-AR-09 this locks the lazy cache against staleness: an add_edge
    // after a query must be visible to every scan immediately.
    let mut graph = chain_graph();
    assert_eq!(file_ids(&graph.direct_includes(FileId(1))), vec![2]);

    graph.add_edge(edge(1, 3, 77));
    assert_eq!(
        file_ids(&graph.direct_includes(FileId(1))),
        vec![2, 3],
        "a query must never serve an adjacency stale to the last add_edge"
    );

    let hood = graph.traverse_neighborhood(FileId(1), 1, TraversalDirection::Outgoing);
    assert_eq!(file_ids(&hood.nodes), vec![1, 2, 3]);
    assert_eq!(
        file_pairs(&hood.edges),
        vec![(1, 2), (1, 3), (2, 3)],
        "traversal must see the edge added after the first query; the induced \
         edge set also carries (2,3) because both endpoints are visited"
    );
}

#[test]
fn clone_shares_data_without_cross_mutation() {
    let mut parent = chain_graph();
    assert_eq!(file_ids(&parent.direct_includes(FileId(1))), vec![2]);

    let mut clone = parent.clone();
    clone.add_edge(edge(9, 1, 5));

    assert_eq!(file_ids(&clone.direct_includes(FileId(9))), vec![1]);
    assert_eq!(
        file_ids(&parent.direct_includes(FileId(9))),
        Vec::<u64>::new(),
        "a mutation on the clone must never leak into the parent's results"
    );
    assert_eq!(file_ids(&parent.direct_includes(FileId(1))), vec![2]);

    parent.add_edge(edge(1, 4, 9));
    assert_eq!(file_ids(&parent.direct_includes(FileId(1))), vec![2, 4]);
    assert_eq!(
        file_ids(&clone.direct_includes(FileId(1))),
        vec![2],
        "a mutation on the parent must never leak into the clone's results"
    );
}

#[test]
fn serialized_shape_excludes_adjacency_cache() {
    // The AR-09 cache index must stay serde-skipped: the graph JSON shape
    // remains exactly {edges, file_paths} (the m00/golden byte contract).
    let graph = chain_graph();
    let _ = graph.direct_includes(FileId(1)); // warm the cache first
    let shape = serde_json::to_value(&graph).expect("serialize graph");
    let keys: Vec<&String> = shape
        .as_object()
        .expect("graph serializes as an object")
        .keys()
        .collect();
    assert_eq!(
        keys,
        vec!["edges", "file_paths"],
        "no adjacency-cache field may appear in the serialized shape"
    );
}
