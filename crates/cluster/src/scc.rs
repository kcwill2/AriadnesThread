use ariadne_index::types::{SemanticEdge, SymbolId};
use std::cmp::min;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolScc {
    pub symbols: Vec<SymbolId>,
}

/// Wrapper kept for edge-slice callers: builds the adjacency fresh on every
/// call. Hot loops that reuse one edge set across many symbol seeds (the
/// planner's per-header SCC pass) should build the adjacency once and call
/// [`compute_symbol_sccs_from_adj`] directly instead.
pub fn compute_symbol_sccs(symbols: &[SymbolId], edges: &[SemanticEdge]) -> Vec<SymbolScc> {
    let adj = build_symbol_adjacency(edges);
    compute_symbol_sccs_from_adj(symbols, &adj)
}

/// Adjacency map consumed by [`compute_symbol_sccs_from_adj`], built in edge
/// iteration order exactly as the wrapper does internally.
pub fn build_symbol_adjacency(edges: &[SemanticEdge]) -> HashMap<&SymbolId, Vec<&SymbolId>> {
    let mut adj: HashMap<&SymbolId, Vec<&SymbolId>> = HashMap::new();
    for edge in edges {
        adj.entry(&edge.src_symbol)
            .or_default()
            .push(&edge.dst_symbol);
    }
    adj
}

/// Tarjan SCCs over the seed `symbols`, traversing a prebuilt adjacency map.
/// Output is identical to [`compute_symbol_sccs`] fed edges that build the
/// same adjacency (same neighbor order); the adjacency is only ever read.
///
/// M23.5 T1.2: the traversal is an explicit-stack frame machine (see
/// [`run_tarjan`]), so OS stack depth stays constant regardless of graph
/// depth (IV-7) while the output contract above stays byte-identical (IV-8).
pub fn compute_symbol_sccs_from_adj<'a>(
    symbols: &[SymbolId],
    adj: &HashMap<&'a SymbolId, Vec<&'a SymbolId>>,
) -> Vec<SymbolScc> {
    run_tarjan(symbols, adj)
}

/// One activation record of the Tarjan traversal: the vertex being visited
/// and how far its neighbor list has been consumed. The recursive version's
/// call stack is replaced one-for-one by a `Vec` of these frames.
struct TarjanFrame<'a> {
    vertex: &'a SymbolId,
    next_neighbor: usize,
}

/// All mutable state the recursive `strongconnect` threaded through its
/// arguments, bundled for the iterative machine. Every field is a plain
/// owned or borrowed collection: no `Rc`/`RefCell`, so a `TarjanRun` in
/// flight is `Send + Sync`-compatible for the rayon planner, and `adj`
/// remains read-only (IV-8).
struct TarjanRun<'a> {
    next_index: usize,
    indices: HashMap<&'a SymbolId, usize>,
    lowlink: HashMap<&'a SymbolId, usize>,
    on_stack: HashSet<&'a SymbolId>,
    tarjan_stack: Vec<&'a SymbolId>,
    frames: Vec<TarjanFrame<'a>>,
    sccs: Vec<SymbolScc>,
}

/// Result of consuming one unit of work from the active frame.
enum FrameStep {
    /// A tree edge was taken: a child frame is now the active frame.
    Descended,
    /// A back/cross edge was examined and (if relevant) lowlink relaxed.
    EdgeVisited,
    /// The neighbor list is exhausted: the frame must retire.
    NeighborsExhausted,
}

/// Iterative Tarjan over the seed list: for each unindexed seed, run the
/// frame machine until its frame stack drains. Lowlink propagation happens
/// on frame POP (`retire_frame` -> `finish_child`), never on call return --
/// the classic iterative-Tarjan trap this structure avoids (M23.5 T1.2).
fn run_tarjan<'a>(
    symbols: &[SymbolId],
    adj: &HashMap<&'a SymbolId, Vec<&'a SymbolId>>,
) -> Vec<SymbolScc> {
    let mut run = TarjanRun::new();
    for seed in symbols {
        if !run.indices.contains_key(seed) {
            run.explore_from(seed, adj);
        }
    }
    run.sccs
}

impl<'a> TarjanRun<'a> {
    fn new() -> Self {
        Self {
            next_index: 0,
            indices: HashMap::new(),
            lowlink: HashMap::new(),
            on_stack: HashSet::new(),
            tarjan_stack: Vec::new(),
            frames: Vec::new(),
            sccs: Vec::new(),
        }
    }

    /// Frame machine for one seed: enter the seed, then step the active
    /// frame until every frame (the seed's included) has retired.
    fn explore_from(
        &mut self,
        seed: &'a SymbolId,
        adj: &HashMap<&'a SymbolId, Vec<&'a SymbolId>>,
    ) {
        self.enter_vertex(seed);
        while !self.frames.is_empty() {
            if let FrameStep::NeighborsExhausted = self.advance_frame(adj) {
                self.retire_frame();
            }
        }
    }

    /// Entry effect of the recursive `strongconnect` head: assign index and
    /// lowlink, push onto the Tarjan stack, and open a frame at neighbor 0.
    fn enter_vertex(&mut self, vertex: &'a SymbolId) {
        self.indices.insert(vertex, self.next_index);
        self.lowlink.insert(vertex, self.next_index);
        self.next_index += 1;
        self.tarjan_stack.push(vertex);
        self.on_stack.insert(vertex);
        self.frames.push(TarjanFrame {
            vertex,
            next_neighbor: 0,
        });
    }

    /// Consume exactly one neighbor slot of the active frame, mirroring one
    /// iteration of the recursive neighbor loop.
    fn advance_frame(
        &mut self,
        adj: &HashMap<&'a SymbolId, Vec<&'a SymbolId>>,
    ) -> FrameStep {
        let Some((vertex, neighbor)) = self.claim_next_edge(adj) else {
            return FrameStep::NeighborsExhausted;
        };
        if !self.indices.contains_key(neighbor) {
            self.enter_vertex(neighbor);
            return FrameStep::Descended;
        }
        if self.on_stack.contains(neighbor) {
            self.relax_lowlink_toward_index(vertex, neighbor);
        }
        FrameStep::EdgeVisited
    }

    /// Hand out the active frame's next unexamined neighbor and advance its
    /// cursor, or signal exhaustion when the neighbor list is consumed.
    fn claim_next_edge(
        &mut self,
        adj: &HashMap<&'a SymbolId, Vec<&'a SymbolId>>,
    ) -> Option<(&'a SymbolId, &'a SymbolId)> {
        let frame = self.frames.last_mut().expect("advance_frame runs with a live frame");
        let no_neighbors: &[&'a SymbolId] = &[];
        let neighbors = adj
            .get(frame.vertex)
            .map(Vec::as_slice)
            .unwrap_or(no_neighbors);
        if frame.next_neighbor >= neighbors.len() {
            return None;
        }
        let neighbor = neighbors[frame.next_neighbor];
        frame.next_neighbor += 1;
        Some((frame.vertex, neighbor))
    }

    /// Back-edge rule of the recursive body: for a neighbor still on the
    /// Tarjan stack, `lowlink[v] = min(lowlink[v], index[w])`.
    fn relax_lowlink_toward_index(&mut self, vertex: &'a SymbolId, neighbor: &'a SymbolId) {
        let neighbor_index = *self
            .indices
            .get(neighbor)
            .expect("on_stack membership implies indices contains the neighbor");
        let vertex_lowlink = self
            .lowlink
            .get_mut(vertex)
            .expect("a live frame vertex always carries a lowlink");
        *vertex_lowlink = min(*vertex_lowlink, neighbor_index);
    }

    /// Frame pop: emit the vertex's SCC if it became a root, then propagate
    /// its lowlink to the parent frame. Emitting before propagating matches
    /// the recursive body, which emits at the end of the callee before the
    /// caller applies `min(lowlink[v], lowlink[w])` after the call.
    fn retire_frame(&mut self) {
        let frame = self.frames.pop().expect("retire_frame runs with a live frame");
        self.emit_group_if_root(frame.vertex);
        self.finish_child(frame.vertex);
    }

    /// Root rule of the recursive body: when `lowlink == index` at the end
    /// of the vertex's traversal, pop the Tarjan stack through the vertex
    /// and emit the group with the same lexicographic intra-SCC sort.
    fn emit_group_if_root(&mut self, vertex: &'a SymbolId) {
        if self.lowlink.get(vertex) != self.indices.get(vertex) {
            return;
        }
        let mut group: Vec<SymbolId> = Vec::new();
        loop {
            let popped = self
                .tarjan_stack
                .pop()
                .expect("vertex is on the stack until popped here, so the stack is non-empty");
            self.on_stack.remove(popped);
            group.push(popped.clone());
            if popped == vertex {
                break;
            }
        }
        // To make deterministic, sort inside SCC if needed, but not strictly required
        // unless test demands it. We just leave it as is, or we could sort by symbol name.
        group.sort_by(|first, second| first.0.cmp(&second.0));
        self.sccs.push(SymbolScc { symbols: group });
    }

    /// Lowlink propagation on frame POP: the retired frame's vertex plays
    /// the role of the recursed `w`, the new top frame the role of the
    /// caller `v` in the recursive `min` update applied after the call.
    fn finish_child(&mut self, child: &'a SymbolId) {
        let Some(parent_frame) = self.frames.last() else {
            return;
        };
        let child_lowlink = *self
            .lowlink
            .get(child)
            .expect("a retiring vertex always carries a lowlink");
        let parent_lowlink = self
            .lowlink
            .get_mut(parent_frame.vertex)
            .expect("a live frame vertex always carries a lowlink");
        *parent_lowlink = min(*parent_lowlink, child_lowlink);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_index::types::DependencyKind;
    use ariadne_repo::FileId;

    fn edge(src: &str, dst: &str) -> SemanticEdge {
        SemanticEdge {
            src_symbol: SymbolId::new(src),
            dst_symbol: SymbolId::new(dst),
            kind: DependencyKind::Complete,
            src_file: FileId::from_relative_path("test"),
            dst_file: FileId::from_relative_path("test"),
        }
    }

    #[test]
    fn test_acyclic_sccs() {
        let syms = vec![SymbolId::new("A"), SymbolId::new("B"), SymbolId::new("C")];
        // A -> B -> C
        let edges = vec![edge("A", "B"), edge("B", "C")];

        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 3);
        // Each should be in its own SCC, topologically sorted (leaf first) or reverse
        // Tarjan typically returns reverse topological order: C, B, A
        let mut lengths: Vec<usize> = sccs.iter().map(|scc| scc.symbols.len()).collect();
        lengths.sort();
        assert_eq!(lengths, vec![1, 1, 1]);
    }

    #[test]
    fn test_cyclic_sccs() {
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
            SymbolId::new("C"),
            SymbolId::new("D"),
        ];
        // A -> B -> C -> A (Cycle)
        // D -> C (No cycle)
        let edges = vec![
            edge("A", "B"),
            edge("B", "C"),
            edge("C", "A"),
            edge("D", "C"),
        ];

        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 2);

        let mut lengths: Vec<usize> = sccs.iter().map(|scc| scc.symbols.len()).collect();
        lengths.sort();
        assert_eq!(lengths, vec![1, 3]);

        let cycle_scc = sccs.iter().find(|scc| scc.symbols.len() == 3).unwrap();
        assert!(cycle_scc.symbols.contains(&SymbolId::new("A")));
        assert!(cycle_scc.symbols.contains(&SymbolId::new("B")));
        assert!(cycle_scc.symbols.contains(&SymbolId::new("C")));
    }

    #[test]
    fn test_empty_input() {
        let syms = vec![];
        let edges = vec![];
        let sccs = compute_symbol_sccs(&syms, &edges);
        assert!(sccs.is_empty());
    }

    #[test]
    fn test_disconnected_components() {
        let syms = vec![SymbolId::new("A"), SymbolId::new("B")];
        let edges = vec![];
        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 2);
    }

    #[test]
    fn test_self_loop() {
        let syms = vec![SymbolId::new("A")];
        let edges = vec![edge("A", "A")];
        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].symbols.len(), 1);
        assert_eq!(sccs[0].symbols[0], SymbolId::new("A"));
    }

    #[test]
    fn test_multiple_disjoint_sccs() {
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
            SymbolId::new("C"),
            SymbolId::new("D"),
            SymbolId::new("E"),
        ];
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("C", "D"),
            edge("D", "E"),
            edge("E", "C"),
        ];
        let mut sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 2);
        sccs.sort_by_key(|scc| scc.symbols.len());
        assert_eq!(sccs[0].symbols.len(), 2);
        assert!(sccs[0].symbols.contains(&SymbolId::new("A")));
        assert!(sccs[0].symbols.contains(&SymbolId::new("B")));
        assert_eq!(sccs[1].symbols.len(), 3);
        assert!(sccs[1].symbols.contains(&SymbolId::new("C")));
        assert!(sccs[1].symbols.contains(&SymbolId::new("D")));
        assert!(sccs[1].symbols.contains(&SymbolId::new("E")));
    }

    #[test]
    fn from_adj_matches_wrapper_on_shared_fixture() {
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
            SymbolId::new("C"),
            SymbolId::new("D"),
            SymbolId::new("E"),
        ];
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("B", "C"),
            edge("C", "D"),
            edge("D", "E"),
            edge("E", "C"),
            edge("E", "A"),
        ];
        let adj: HashMap<&SymbolId, Vec<&SymbolId>> = edges.iter().fold(
            HashMap::new(),
            |mut adj: HashMap<&SymbolId, Vec<&SymbolId>>, edge| {
                adj.entry(&edge.src_symbol)
                    .or_default()
                    .push(&edge.dst_symbol);
                adj
            },
        );
        assert_eq!(
            compute_symbol_sccs_from_adj(&syms, &adj),
            compute_symbol_sccs(&syms, &edges),
            "prebuilt-adjacency SCCs must equal the delegating wrapper's"
        );
    }

    #[test]
    fn from_adj_resolves_two_disjoint_sccs() {
        // The shared fixture minus the E->A back edge: {A,B} and {C,D,E}
        // are genuinely two SCCs, asserted on the from_adj entry point.
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
            SymbolId::new("C"),
            SymbolId::new("D"),
            SymbolId::new("E"),
        ];
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("B", "C"),
            edge("C", "D"),
            edge("D", "E"),
            edge("E", "C"),
        ];
        let sccs = compute_symbol_sccs_from_adj(&syms, &build_symbol_adjacency(&edges));
        assert_eq!(
            sccs.len(),
            2,
            "dropping E->A must split the whole graph in two SCCs"
        );
        let ab = sccs
            .iter()
            .find(|scc| scc.symbols.contains(&SymbolId::new("A")))
            .expect("SCC containing A");
        assert_eq!(ab.symbols.len(), 2);
        assert!(ab.symbols.contains(&SymbolId::new("B")));
        let cde = sccs
            .iter()
            .find(|scc| scc.symbols.contains(&SymbolId::new("C")))
            .expect("SCC containing C");
        assert_eq!(cde.symbols.len(), 3);
        assert!(cde.symbols.contains(&SymbolId::new("D")));
        assert!(cde.symbols.contains(&SymbolId::new("E")));
    }

    #[test]
    fn test_connected_multi_scc_graph() {
        let syms = vec![
            SymbolId::new("A"),
            SymbolId::new("B"),
            SymbolId::new("C"),
            SymbolId::new("D"),
        ];
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("B", "C"),
            edge("C", "D"),
            edge("D", "C"),
        ];
        let sccs = compute_symbol_sccs(&syms, &edges);
        assert_eq!(sccs.len(), 2);

        let mut lengths: Vec<usize> = sccs.iter().map(|scc| scc.symbols.len()).collect();
        lengths.sort();
        assert_eq!(lengths, vec![2, 2]);
    }
}
