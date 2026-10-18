use ariadne_index::types::{FileLocation, SymbolId};
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallNode {
    pub symbol_id: SymbolId,
    pub name: String,
    pub owning_file: FileId,
    pub location: Option<FileLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallEdge {
    pub caller: SymbolId,
    pub callee: SymbolId,
    pub call_count: usize,
    pub call_locations: Vec<FileLocation>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallGraph {
    pub nodes: Vec<CallNode>,
    pub edges: Vec<CallEdge>,
}

struct CalleeLookup<'m> {
    by_exact_name: std::collections::HashMap<&'m str, &'m crate::ReducedSymbolRecord>,
    by_id_suffix: std::collections::HashMap<&'m str, &'m crate::ReducedSymbolRecord>,
}

impl<'m> CalleeLookup<'m> {
    fn from_model(model: &'m crate::ReducedSemanticModel) -> Self {
        let mut by_exact_name: std::collections::HashMap<&str, &crate::ReducedSymbolRecord> =
            std::collections::HashMap::new();
        let mut by_id_suffix: std::collections::HashMap<&str, &crate::ReducedSymbolRecord> =
            std::collections::HashMap::new();

        for sym in model.symbols.values() {
            by_exact_name.insert(sym.id.0.as_str(), sym);
            by_exact_name.entry(sym.name.as_str()).or_insert(sym);

            let mut current = sym.id.0.as_str();
            while let Some(idx) = current.find("::") {
                current = &current[idx + 2..];
                by_id_suffix.entry(current).or_insert(sym);
            }
        }

        Self {
            by_exact_name,
            by_id_suffix,
        }
    }

    fn resolve(&self, callee_name: &str) -> Option<&'m crate::ReducedSymbolRecord> {
        self.by_exact_name
            .get(callee_name)
            .copied()
            .or_else(|| self.by_id_suffix.get(callee_name).copied())
            .or_else(|| {
                let mut current = callee_name;
                while let Some(idx) = current.find("::") {
                    current = &current[idx + 2..];
                    if let Some(&sym) = self.by_exact_name.get(current) {
                        return Some(sym);
                    }
                }
                None
            })
    }
}

fn call_node(
    symbol_id: SymbolId,
    name: String,
    defining_file: Option<ariadne_repo::FileId>,
) -> CallNode {
    CallNode {
        symbol_id,
        name,
        owning_file: defining_file
            .unwrap_or_else(|| ariadne_repo::FileId::from_relative_path("unknown")),
        location: None,
    }
}

fn record_call_site(
    nodes: &mut std::collections::BTreeMap<SymbolId, CallNode>,
    edges: &mut std::collections::BTreeMap<(SymbolId, SymbolId), CallEdge>,
    model: &crate::ReducedSemanticModel,
    site: &ariadne_index::types::CallSite,
    callee_sym: &crate::ReducedSymbolRecord,
) {
    let caller = site.caller_symbol.clone();
    let callee = callee_sym.id.clone();

    let edge = edges
        .entry((caller.clone(), callee.clone()))
        .or_insert_with(|| CallEdge {
            caller: caller.clone(),
            callee: callee.clone(),
            call_count: 0,
            call_locations: Vec::new(),
        });

    edge.call_count += 1;
    if !edge.call_locations.contains(&site.location) {
        edge.call_locations.push(site.location.clone());
    }

    if !nodes.contains_key(&caller) {
        if let Some(record) = model.symbols.get(&caller) {
            nodes.insert(
                caller.clone(),
                call_node(
                    caller.clone(),
                    record.name.clone(),
                    record.defining_file.clone(),
                ),
            );
        }
    }
    if !nodes.contains_key(&callee) {
        nodes.insert(
            callee.clone(),
            call_node(
                callee.clone(),
                callee_sym.name.clone(),
                callee_sym.defining_file.clone(),
            ),
        );
    }
}

fn seed_uncalled_function_nodes(
    nodes: &mut std::collections::BTreeMap<SymbolId, CallNode>,
    model: &crate::ReducedSemanticModel,
) {
    for (sym_id, record) in &model.symbols {
        if record.kind == ariadne_index::types::SymbolKind::Function && !nodes.contains_key(sym_id)
        {
            nodes.insert(
                sym_id.clone(),
                call_node(
                    sym_id.clone(),
                    record.name.clone(),
                    record.defining_file.clone(),
                ),
            );
        }
    }
}

/// Runs one root's explicit-stack walk to completion (UTA-B5, hand H1). A
/// frame is `(node, cursor into adj[node])`; consuming one edge slot
/// reproduces the retired recursive body's three steps verbatim -
/// on-path neighbor -> the current path cut is a cycle; unvisited
/// neighbor -> enter and descend (the parent keeps its cursor, so siblings
/// resume after the child, exactly like call return); exhausted frame ->
/// retire and pop the path. No recursion: deep call chains cannot overflow
/// the OS stack. Mirrors the `explore_frames` machine in `closures.rs`.
#[allow(clippy::too_many_arguments)]
fn explore_call_frames(
    frames: &mut Vec<(SymbolId, usize)>,
    adj: &std::collections::HashMap<SymbolId, Vec<SymbolId>>,
    visited: &mut std::collections::HashSet<SymbolId>,
    path: &mut Vec<SymbolId>,
    path_position: &mut std::collections::HashMap<SymbolId, usize>,
    cycles: &mut Vec<Vec<SymbolId>>,
) {
    while let Some(frame) = frames.last_mut() {
        let node = frame.0.clone();
        let cursor = frame.1;
        let neighbors = adj.get(&node).map(Vec::as_slice).unwrap_or(&[]);
        if cursor < neighbors.len() {
            frame.1 += 1;
            let neighbor = neighbors[cursor].clone();
            if let Some(&position) = path_position.get(&neighbor) {
                cycles.push(path[position..].to_vec());
                continue;
            }
            if visited.insert(neighbor.clone()) {
                path_position.insert(neighbor.clone(), path.len());
                path.push(neighbor.clone());
                frames.push((neighbor, 0));
            }
            continue;
        }
        frames.pop();
        let retired = path
            .pop()
            .expect("frames and path are entered and retired together");
        path_position.remove(&retired);
    }
}

impl CallGraph {
    pub fn build_from_index(
        index_result: &ariadne_index::IndexResult,
        model: &crate::ReducedSemanticModel,
    ) -> Self {
        let mut nodes: std::collections::BTreeMap<SymbolId, CallNode> =
            std::collections::BTreeMap::new();
        let mut edges: std::collections::BTreeMap<(SymbolId, SymbolId), CallEdge> =
            std::collections::BTreeMap::new();

        let lookup = CalleeLookup::from_model(model);

        for tu in &index_result.tu_analyses {
            for site in &tu.call_sites {
                if let Some(callee_sym) = lookup.resolve(site.callee_name.as_str()) {
                    record_call_site(&mut nodes, &mut edges, model, site, callee_sym);
                }
            }
        }

        seed_uncalled_function_nodes(&mut nodes, model);

        Self {
            nodes: nodes.into_values().collect(),
            edges: edges.into_values().collect(),
        }
    }

    pub fn callers_of(&self, symbol: &SymbolId) -> Vec<&CallEdge> {
        self.edges.iter().filter(|e| e.callee == *symbol).collect()
    }

    pub fn callees_of(&self, symbol: &SymbolId) -> Vec<&CallEdge> {
        self.edges.iter().filter(|e| e.caller == *symbol).collect()
    }

    /// Enumerates the call-graph cycles, normalized (each cycle rotated so
    /// its smallest `SymbolId` leads, list sorted and deduped).
    ///
    /// UTA-B5 (hand H1): the walk runs on an explicit frame stack mirroring
    /// the `explore_frames` machine in `closures.rs` - no recursion, so deep
    /// call chains cannot overflow the OS stack (IV-7 lock: the 100 000-node
    /// fused chain fixture completes on a 64 KiB thread). The traversal
    /// semantics are the retired recursive DFS's verbatim (same root order,
    /// same neighbor order, same global-visited skip, same path-cut cycles);
    /// only the call stack was replaced, plus the O(depth) on-path scan
    /// became an O(1) `path_position` lookup, which is why the deep-chain
    /// fixture is now also linear-time.
    pub fn detect_recursion_cycles(&self) -> Vec<Vec<SymbolId>> {
        use std::collections::{HashMap, HashSet};

        let mut adj: HashMap<SymbolId, Vec<SymbolId>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.caller.clone())
                .or_default()
                .push(edge.callee.clone());
        }

        let mut cycles: Vec<Vec<SymbolId>> = Vec::new();
        let mut visited: HashSet<SymbolId> = HashSet::new();
        // The current walk path, plus each member's position on it: the
        // back-edge cut `path[position..]` stays O(cycle length) instead of
        // re-scanning the path per examined edge.
        let mut path: Vec<SymbolId> = Vec::new();
        let mut path_position: HashMap<SymbolId, usize> = HashMap::new();

        let mut roots: Vec<SymbolId> = adj.keys().cloned().collect();
        roots.sort();

        for root in &roots {
            if visited.contains(root) {
                continue;
            }
            // Enter the root exactly as the recursive body's head did for
            // the root call (the path is always empty between roots).
            visited.insert(root.clone());
            path_position.insert(root.clone(), path.len());
            path.push(root.clone());
            let mut frames: Vec<(SymbolId, usize)> = vec![(root.clone(), 0)];
            explore_call_frames(
                &mut frames,
                &adj,
                &mut visited,
                &mut path,
                &mut path_position,
                &mut cycles,
            );
            debug_assert!(path.is_empty(), "each root's walk unwinds fully");
        }

        for cycle in &mut cycles {
            let min_idx = cycle
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.cmp(b))
                .expect("cycles are pushed as path[position..] slices, which are never empty")
                .0;
            cycle.rotate_left(min_idx);
        }
        cycles.sort();
        cycles.dedup();

        cycles
    }

    /// BFS shortest call path from `from` to `to` over outgoing call edges.
    /// AR-09 (plan audit P4): callee lists are grouped in ONE O(E) pass and
    /// each caller's list is sorted once, replacing the retired
    /// `callees_of` O(E) scan + per-node sort (was O(V_visited * E)).
    /// Iterating the grouped lists yields the same neighbor sequence the
    /// retired code sorted to per node - duplicates included (kept in the
    /// lists, suppressed by `visited` as before) - so the parent map and
    /// the reconstructed path are byte-identical (locked in
    /// `ar09_adjacency_equivalence_test::callgraph_shortest_path_snapshot_locks`).
    pub fn shortest_call_path(&self, from: &SymbolId, to: &SymbolId) -> Option<Vec<SymbolId>> {
        use std::collections::{HashMap, HashSet, VecDeque};

        let mut callees_by_caller: HashMap<&SymbolId, Vec<&SymbolId>> = HashMap::new();
        for edge in &self.edges {
            callees_by_caller
                .entry(&edge.caller)
                .or_default()
                .push(&edge.callee);
        }
        for callees in callees_by_caller.values_mut() {
            callees.sort();
        }

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

            if let Some(callees) = callees_by_caller.get(&current) {
                for &callee in callees {
                    if !visited.contains(callee) {
                        visited.insert(callee.clone());
                        parent.insert(callee.clone(), current.clone());
                        queue.push_back(callee.clone());
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReducedSemanticModel;
    use ariadne_index::types::{CallSite, SymbolKind, SymbolRecord, TuAnalysis};
    use std::path::PathBuf;

    fn setup() -> (ariadne_index::IndexResult, ReducedSemanticModel) {
        let f1 = FileId::from_relative_path("main.rs");
        let sym_a = SymbolId::new("A");
        let sym_b = SymbolId::new("B");
        let sym_c = SymbolId::new("C");

        let tu = TuAnalysis {
            tu_file_id: f1.clone(),
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![
                SymbolRecord {
                    id: sym_a.clone(),
                    name: "A".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,

                    modifiers: Vec::new(),
                },
                SymbolRecord {
                    id: sym_b.clone(),
                    name: "B".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,

                    modifiers: Vec::new(),
                },
                SymbolRecord {
                    id: sym_c.clone(),
                    name: "C".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,

                    modifiers: Vec::new(),
                },
            ],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![
                CallSite {
                    caller_symbol: sym_a.clone(),
                    callee_name: "B".into(),
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 10,
                        column: 5,
                    },
                },
                CallSite {
                    caller_symbol: sym_b.clone(),
                    callee_name: "C".into(),
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 20,
                        column: 5,
                    },
                },
                CallSite {
                    caller_symbol: sym_c.clone(),
                    callee_name: "A".into(), // cycle A -> B -> C -> A
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 30,
                        column: 5,
                    },
                },
                CallSite {
                    caller_symbol: sym_c.clone(),
                    callee_name: "C".into(), // self recursion
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 31,
                        column: 5,
                    },
                },
            ],

            extends_edges: Vec::new(),
        };

        let index_result = ariadne_index::IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let model = ReducedSemanticModel::reduce(&index_result);
        (index_result, model)
    }

    #[test]
    fn test_callgraph_build() {
        let (index, model) = setup();
        let graph = CallGraph::build_from_index(&index, &model);

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 4);
    }

    #[test]
    fn test_callers_and_callees() {
        let (index, model) = setup();
        let graph = CallGraph::build_from_index(&index, &model);

        let callers_of_b = graph.callers_of(&SymbolId::new("B"));
        assert_eq!(callers_of_b.len(), 1);
        assert_eq!(callers_of_b[0].caller, SymbolId::new("A"));

        let callees_of_b = graph.callees_of(&SymbolId::new("B"));
        assert_eq!(callees_of_b.len(), 1);
        assert_eq!(callees_of_b[0].callee, SymbolId::new("C"));
    }

    #[test]
    fn test_recursion_cycles() {
        let (index, model) = setup();
        let graph = CallGraph::build_from_index(&index, &model);

        let cycles = graph.detect_recursion_cycles();
        assert_eq!(cycles.len(), 2);
    }

    // UTA-B5 (hand H1) fixtures. Mirrors the M23.5 IV-7 pattern from
    // crates/graph/tests/cycle_groups_test.rs: small-stack thread for the
    // depth stress, directly-built CallGraphs for the enumeration shapes.
    const SMALL_STACK_BYTES: usize = 64 * 1024;
    const CALL_CHAIN_LEN: usize = 100_000;

    fn plain_call_edge(caller: &str, callee: &str) -> CallEdge {
        CallEdge {
            caller: SymbolId::new(caller),
            callee: SymbolId::new(callee),
            call_count: 1,
            call_locations: Vec::new(),
        }
    }

    fn graph_of_calls(calls: &[(&str, &str)]) -> CallGraph {
        CallGraph {
            nodes: Vec::new(),
            edges: calls
                .iter()
                .map(|(caller, callee)| plain_call_edge(caller, callee))
                .collect(),
        }
    }

    #[test]
    fn two_scc_graph_enumerates_every_cycle_once_in_normalized_order() {
        // A<->B and C<->D joined by B->C: two disjoint cycles, neither may
        // be missed nor reported twice, and each must come out rotated to
        // its smallest member with the list sorted.
        let graph = graph_of_calls(&[
            ("A", "B"),
            ("B", "A"),
            ("B", "C"),
            ("C", "D"),
            ("D", "C"),
        ]);

        let cycles = graph.detect_recursion_cycles();
        assert_eq!(
            cycles,
            vec![
                vec![SymbolId::new("A"), SymbolId::new("B")],
                vec![SymbolId::new("C"), SymbolId::new("D")],
            ],
            "the two-SCC graph must enumerate ALL cycles exactly once each, \
             each rotated to its smallest member, list sorted"
        );
    }

    #[test]
    fn deep_call_chain_cycle_detection_completes_on_small_stack() {
        // IV-7 style lock (cycle_groups_test.rs:105): a 100 000-node fused
        // call chain on a 64 KiB thread. The retired recursive DFS
        // stack-overflows it (red evidence: aborted test binary); an
        // iterative frame machine completes.
        let spawned = std::thread::Builder::new()
            .stack_size(SMALL_STACK_BYTES)
            .spawn(detect_deep_chain_cycles)
            .expect("deep-chain stress thread spawns with the requested stack size");
        let cycles = spawned
            .join()
            .expect("deep-chain stress thread completed without panicking");
        assert_eq!(
            cycles.len(),
            1,
            "the fused 100 000-node chain has exactly one cycle"
        );
        assert_eq!(
            cycles[0].len(),
            CALL_CHAIN_LEN,
            "the single cycle covers every chain member"
        );
    }

    /// Builds f0 -> f1 -> ... -> f99999 -> f0 and runs cycle detection.
    /// Runs on the caller's thread so the test controls the stack size.
    fn detect_deep_chain_cycles() -> Vec<Vec<SymbolId>> {
        let mut edges = Vec::with_capacity(CALL_CHAIN_LEN);
        for position in 0..CALL_CHAIN_LEN {
            let next = (position + 1) % CALL_CHAIN_LEN;
            edges.push(plain_call_edge(
                &format!("f{position}"),
                &format!("f{next}"),
            ));
        }
        CallGraph {
            nodes: Vec::new(),
            edges,
        }
        .detect_recursion_cycles()
    }

    #[test]
    fn test_shortest_call_path() {
        let (index, model) = setup();
        let graph = CallGraph::build_from_index(&index, &model);

        let path = graph.shortest_call_path(&SymbolId::new("A"), &SymbolId::new("C"));
        assert!(path.is_some());
        assert_eq!(
            path.unwrap(),
            vec![SymbolId::new("A"), SymbolId::new("B"), SymbolId::new("C")]
        );
    }

    #[test]
    fn test_callgraph_build_qualified_names() {
        let f1 = FileId::from_relative_path("main.rs");
        let sym_caller = SymbolId::new("caller");
        let sym_callee = SymbolId::new("ns::MyClass::callee");

        let tu = TuAnalysis {
            tu_file_id: f1.clone(),
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![
                SymbolRecord {
                    id: sym_caller.clone(),
                    name: "caller".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,

                    modifiers: Vec::new(),
                },
                SymbolRecord {
                    id: sym_callee.clone(),
                    name: "callee".into(), // The name might be just "callee"
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,

                    modifiers: Vec::new(),
                },
            ],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![
                CallSite {
                    caller_symbol: sym_caller.clone(),
                    callee_name: "callee".into(), // Caller just says "callee()"
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 10,
                        column: 5,
                    },
                },
                CallSite {
                    caller_symbol: sym_caller.clone(),
                    callee_name: "MyClass::callee".into(), // Caller uses fully qualified name
                    location: FileLocation {
                        file_id: f1.clone(),
                        line: 11,
                        column: 5,
                    },
                },
            ],

            extends_edges: Vec::new(),
        };

        let index_result = ariadne_index::IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let model = ReducedSemanticModel::reduce(&index_result);

        let graph = CallGraph::build_from_index(&index_result, &model);

        let callers = graph.callers_of(&sym_callee);
        assert_eq!(
            callers.len(),
            1,
            "Should map both call sites to the same callee"
        );
        assert_eq!(callers[0].call_count, 2, "Should have 2 calls");
    }

    #[test]
    fn pin_call_edge_without_caller_node() {
        // UTA-T1 item 5 (hand H1): pin the CURRENT `record_call_site` shape
        // when `site.caller_symbol` is absent from `model.symbols`. The edge
        // is still recorded (and the callee gets a node), but the caller-node
        // insert is gated on `model.symbols.get(&caller)`, so the caller
        // never gets a CallNode - the edge floats without a caller node.
        let main_file = FileId::from_relative_path("main.cpp");
        let ghost_caller = SymbolId::new("GhostCaller");
        let known_callee = SymbolId::new("KnownCallee");

        let tu = TuAnalysis {
            tu_file_id: main_file.clone(),
            context_name: "pin".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![SymbolRecord {
                id: known_callee.clone(),
                name: "KnownCallee".into(),
                kind: SymbolKind::Function,
                owning_file: main_file.clone(),
                declarations: vec![],
                definition: None,

                modifiers: Vec::new(),
            }],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![CallSite {
                caller_symbol: ghost_caller.clone(),
                callee_name: "KnownCallee".into(),
                location: FileLocation {
                    file_id: main_file.clone(),
                    line: 7,
                    column: 1,
                },
            }],

            extends_edges: Vec::new(),
        };
        let index_result = ariadne_index::IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let model = ReducedSemanticModel::reduce(&index_result);

        let graph = CallGraph::build_from_index(&index_result, &model);

        let ghost_outgoing = graph.callees_of(&ghost_caller);
        assert_eq!(
            ghost_outgoing.len(),
            1,
            "the edge is recorded even though the caller is unknown to the model"
        );
        assert_eq!(ghost_outgoing[0].callee, known_callee);
        assert_eq!(ghost_outgoing[0].call_count, 1);
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.symbol_id == known_callee),
            "the resolved callee always gets a CallNode"
        );
        assert!(
            graph
                .nodes
                .iter()
                .all(|node| node.symbol_id != ghost_caller),
            "pin: an unregistered caller gets no CallNode - the edge floats \
             without a caller node (reported for an owner decision)"
        );
    }

    #[test]
    fn test_callgraph_resolution_priority() {
        let f1 = FileId::from_relative_path("main.rs");
        let sym_caller = SymbolId::new("caller");
        // We have two symbols. One matches exactly by name, one matches by suffix.
        let sym_exact = SymbolId::new("exact::method");
        let sym_suffix = SymbolId::new("other::exact::method");

        let tu = TuAnalysis {
            tu_file_id: f1.clone(),
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![
                SymbolRecord {
                    id: sym_exact.clone(),
                    name: "exact::method".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,

                    modifiers: Vec::new(),
                },
                SymbolRecord {
                    id: sym_suffix.clone(),
                    name: "method".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: None,

                    modifiers: Vec::new(),
                },
            ],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![CallSite {
                caller_symbol: sym_caller.clone(),
                callee_name: "exact::method".into(), // exact match for sym_exact by ID/Name
                location: FileLocation {
                    file_id: f1.clone(),
                    line: 10,
                    column: 5,
                },
            }],

            extends_edges: Vec::new(),
        };

        let index_result = ariadne_index::IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let model = ReducedSemanticModel::reduce(&index_result);

        let graph = CallGraph::build_from_index(&index_result, &model);

        // Because of HashMap random order in the old implementation, it could pick sym_suffix
        // since `callee_name` ("exact::method") ends with `::sym.name` ("::method").
        // We want to ensure it strictly picks exact matches first.
        let callers = graph.callers_of(&sym_exact);
        assert_eq!(
            callers.len(),
            1,
            "Should prefer exact match over suffix match"
        );
    }
}
