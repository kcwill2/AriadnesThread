//! M23.5 T2.1/T2.2 -- `CycleGroup`, the milestone's cycle currency for the
//! include graph (plan 1790821308681 sec1.3, handoff tasks/m235-t2a-handoff.md).
//!
//! The detection path is an ITERATIVE Tarjan frame machine (IV-7): the OS
//! stack never grows with graph depth. The machine deliberately mirrors the
//! SHAPE of `crates/cluster/src/scc.rs` (Phase 1, reviewer-audited emit-before-
//! propagate frame-pop ordering, verdict sec 4.1) without any cross-crate
//! dependency (D-A7); the ~60-line duplication between the two crates is
//! pre-accepted.
//!
//! POLICY-FREE posture: no language is named here (R2). Classification of
//! groups against `CyclePolicy` stays in the report layer (IV-11, Phase 3).

use crate::graph::IncludeGraph;
use ariadne_repo::FileId;
use std::cmp::{min, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};

/// A maximal entangled group in the include graph: files mutually
/// reachable via includes. Every edge inside a nontrivial group is on a cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleGroup {
    /// Members of the group, ascending by FileId.
    pub member_files: Vec<FileId>,
    /// Closed-walk witness [c0, c1, ..., ck, c0] over edges inside the group;
    /// [c0, c0] for a self-loop group.
    pub representative_cycle: Vec<FileId>,
    /// Groups condense into the DAG; these degrees describe edges crossing out of/into the group.
    pub external_in_count: usize,
    pub external_out_count: usize,
}

impl IncludeGraph {
    /// Iterative Tarjan SCC over the include adjacency; returns only cyclic
    /// (nontrivial or self-loop) groups, reverse-topological over the
    /// condensation DAG, deterministically tie-broken (IV-5). POLICY-FREE:
    /// classification against CyclePolicy stays in the report layer (IV-11),
    /// so the graph crate keeps its R2 no-language-naming posture.
    pub fn detect_cycle_groups(&self) -> Vec<CycleGroup> {
        // Production must call the single adjacency source (graph.rs:83-95);
        // no adjacency-build loop is duplicated here.
        let adjacency = self.adjacency();
        let seeds = self.tarjan_seed_universe();
        let cyclic_groups = filter_cyclic_groups(run_tarjan(&seeds, &adjacency), &adjacency);
        let ordered_groups = order_reverse_topological(cyclic_groups, &adjacency);
        let external_degrees = external_degree_pairs(&ordered_groups, &adjacency);
        ordered_groups
            .into_iter()
            .zip(external_degrees)
            .map(|(member_files, (external_in_count, external_out_count))| {
                let representative_cycle = representative_cycle_for(&member_files, &adjacency);
                CycleGroup {
                    member_files,
                    representative_cycle,
                    external_in_count,
                    external_out_count,
                }
            })
            .collect()
    }

    /// D-A1 seed universe: registered files PLUS every edge source PLUS every
    /// resolved destination, sorted ascending -- matching `compute_closures`
    /// (graph.rs:124-130) rather than `detect_cycles` (graph.rs:240-241),
    /// whose `file_paths`-only seed silently misses cycles through
    /// never-registered endpoints (IV-2 needs the union).
    fn tarjan_seed_universe(&self) -> Vec<FileId> {
        let mut universe: Vec<FileId> = self.file_paths.keys().copied().collect();
        for include_edge in &self.edges {
            universe.push(include_edge.source);
            if let Some(destination) = include_edge.destination {
                universe.push(destination);
            }
        }
        universe.sort();
        universe.dedup();
        universe
    }
}

// ---------------------------------------------------------------------------
// Iterative Tarjan frame machine (D-A6: shape mirrored from the Phase 1
// cluster machine; emit-first-then-propagate on frame POP is audited there).
// ---------------------------------------------------------------------------

/// One activation record of the Tarjan traversal: the vertex being visited
/// and how far its neighbor list has been consumed. The recursive version's
/// call stack is replaced one-for-one by a `Vec` of these frames.
struct TarjanFrame {
    vertex: FileId,
    next_neighbor: usize,
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

/// All mutable state the recursive strong-connect threaded through its
/// arguments, bundled for the iterative machine.
struct TarjanRun {
    next_index: usize,
    indices: HashMap<FileId, usize>,
    lowlink: HashMap<FileId, usize>,
    on_stack: HashSet<FileId>,
    tarjan_stack: Vec<FileId>,
    frames: Vec<TarjanFrame>,
    groups: Vec<Vec<FileId>>,
}

/// Iterative Tarjan over the seed list: for each unindexed seed, run the
/// frame machine until its frame stack drains. Lowlink propagation happens
/// on frame POP (`retire_frame` -> `finish_child`), never on call return --
/// the classic iterative-Tarjan trap this structure avoids (D-A6).
fn run_tarjan(seeds: &[FileId], adjacency: &HashMap<FileId, Vec<FileId>>) -> Vec<Vec<FileId>> {
    let mut run = TarjanRun::new();
    for &seed in seeds {
        if !run.indices.contains_key(&seed) {
            run.explore_from(seed, adjacency);
        }
    }
    run.groups
}

impl TarjanRun {
    fn new() -> Self {
        Self {
            next_index: 0,
            indices: HashMap::new(),
            lowlink: HashMap::new(),
            on_stack: HashSet::new(),
            tarjan_stack: Vec::new(),
            frames: Vec::new(),
            groups: Vec::new(),
        }
    }

    /// Frame machine for one seed: enter the seed, then step the active
    /// frame until every frame (the seed's included) has retired.
    fn explore_from(&mut self, seed: FileId, adjacency: &HashMap<FileId, Vec<FileId>>) {
        self.enter_vertex(seed);
        while !self.frames.is_empty() {
            if let FrameStep::NeighborsExhausted = self.advance_frame(adjacency) {
                self.retire_frame();
            }
        }
    }

    /// PUSH update point: assign index and lowlink from the shared counter,
    /// push onto the Tarjan stack, mark `on_stack`, open a frame at
    /// neighbor 0 -- the entry effect of the recursive body's head.
    fn enter_vertex(&mut self, vertex: FileId) {
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

    /// ADVANCE update point: consume exactly one neighbor slot of the
    /// active frame. Unindexed neighbor -> descend (tree-edge propagation
    /// is DEFERRED to pop); indexed neighbor still `on_stack` ->
    /// `lowlink[v] = min(lowlink[v], index[w])`.
    fn advance_frame(&mut self, adjacency: &HashMap<FileId, Vec<FileId>>) -> FrameStep {
        let Some((vertex, neighbor)) = self.claim_next_edge(adjacency) else {
            return FrameStep::NeighborsExhausted;
        };
        if !self.indices.contains_key(&neighbor) {
            self.enter_vertex(neighbor);
            return FrameStep::Descended;
        }
        if self.on_stack.contains(&neighbor) {
            self.relax_lowlink_toward_index(vertex, neighbor);
        }
        FrameStep::EdgeVisited
    }

    /// Hand out the active frame's next unexamined neighbor and advance its
    /// cursor, or signal exhaustion when the neighbor list is consumed.
    fn claim_next_edge(
        &mut self,
        adjacency: &HashMap<FileId, Vec<FileId>>,
    ) -> Option<(FileId, FileId)> {
        let frame = self.frames.last_mut().expect("advance_frame runs with a live frame");
        let no_neighbors: &[FileId] = &[];
        let neighbors = adjacency
            .get(&frame.vertex)
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
    fn relax_lowlink_toward_index(&mut self, vertex: FileId, neighbor: FileId) {
        let neighbor_index = *self
            .indices
            .get(&neighbor)
            .expect("on_stack membership implies indices contains the neighbor");
        let vertex_lowlink = self
            .lowlink
            .get_mut(&vertex)
            .expect("a live frame vertex always carries a lowlink");
        *vertex_lowlink = min(*vertex_lowlink, neighbor_index);
    }

    /// POP update point (D-A6, audited in Phase 1, cluster scc.rs:190-198):
    /// emit the vertex's group if it became a root FIRST, then propagate
    /// `min(lowlink[parent], lowlink[child])`. Emitting before propagating
    /// matches the recursive body, which emits at the end of the callee
    /// before the caller applies the min after the call.
    fn retire_frame(&mut self) {
        let frame = self.frames.pop().expect("retire_frame runs with a live frame");
        self.emit_group_if_root(frame.vertex);
        self.finish_child(frame.vertex);
    }

    /// Root rule of the recursive body: when `lowlink == index` at the end
    /// of the vertex's traversal, pop the Tarjan stack through the vertex
    /// and emit the group, members ascending by FileId (D-A3, IV-5).
    fn emit_group_if_root(&mut self, vertex: FileId) {
        if self.lowlink.get(&vertex) != self.indices.get(&vertex) {
            return;
        }
        let mut group: Vec<FileId> = Vec::new();
        loop {
            let popped = self
                .tarjan_stack
                .pop()
                .expect("vertex is on the stack until popped here, so the stack is non-empty");
            self.on_stack.remove(&popped);
            group.push(popped);
            if popped == vertex {
                break;
            }
        }
        group.sort();
        self.groups.push(group);
    }

    /// Lowlink propagation on frame POP: the retired frame's vertex plays
    /// the role of the recursed `w`, the new top frame the role of the
    /// caller `v` in the recursive `min` update applied after the call.
    fn finish_child(&mut self, child: FileId) {
        let child_completed = self.frames.last().is_some();
        if !child_completed {
            return;
        }
        let parent_vertex = self.frames.last().expect("child_completed just checked").vertex;
        let child_lowlink = *self
            .lowlink
            .get(&child)
            .expect("a retiring vertex always carries a lowlink");
        let parent_lowlink = self
            .lowlink
            .get_mut(&parent_vertex)
            .expect("a live frame vertex always carries a lowlink");
        *parent_lowlink = min(*parent_lowlink, child_lowlink);
    }
}

// ---------------------------------------------------------------------------
// Cyclic filtering, condensation ordering, external degrees.
// ---------------------------------------------------------------------------

/// IV-2: keep nontrivial groups and self-loop singletons; drop the rest.
fn filter_cyclic_groups(
    groups: Vec<Vec<FileId>>,
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> Vec<Vec<FileId>> {
    groups
        .into_iter()
        .filter(|members| group_is_cyclic(members, adjacency))
        .collect()
}

fn group_is_cyclic(
    members: &[FileId],
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> bool {
    match members {
        [] => false,
        [single] => adjacency
            .get(single)
            .is_some_and(|destinations| destinations.contains(single)),
        _ => true,
    }
}

/// D-A2 (IV-5, structural, checkable): build the condensation over the
/// emitted groups and order it with Kahn's algorithm on a min-heap keyed
/// by each group's smallest member, emitting SINKS first -- sinks-first on
/// the condensation is reverse-topological. The min-heap tie-break decides
/// groups the DAG leaves incomparable.
fn order_reverse_topological(
    groups: Vec<Vec<FileId>>,
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> Vec<Vec<FileId>> {
    let group_of = build_group_lookup(&groups);
    let group_count = groups.len();
    if group_count == 0 {
        return Vec::new();
    }
    let (predecessors, mut out_degree) = condensation_edges(&group_of, group_count, adjacency);
    let mut sink_heap: BinaryHeap<Reverse<(FileId, usize)>> = BinaryHeap::new();
    for index in 0..group_count {
        if out_degree[index] == 0 {
            sink_heap.push(Reverse((groups[index][0], index)));
        }
    }
    let mut ordered = Vec::with_capacity(group_count);
    while let Some(Reverse((_, index))) = sink_heap.pop() {
        ordered.push(groups[index].clone());
        for &predecessor in &predecessors[index] {
            out_degree[predecessor] -= 1;
            if out_degree[predecessor] == 0 {
                sink_heap.push(Reverse((groups[predecessor][0], predecessor)));
            }
        }
    }
    debug_assert_eq!(ordered.len(), group_count, "condensation must be acyclic");
    ordered
}

fn build_group_lookup(groups: &[Vec<FileId>]) -> HashMap<FileId, usize> {
    let mut group_of = HashMap::new();
    for (index, members) in groups.iter().enumerate() {
        for &member in members {
            group_of.insert(member, index);
        }
    }
    group_of
}

/// Predecessor lists and out-degrees of the condensation, over DEDUPED
/// adjacency pairs (D-A4 basis). Members are sorted ascending (D-A3), so
/// `groups[index][0]` is the group's smallest member (the heap key).
fn condensation_edges(
    group_of: &HashMap<FileId, usize>,
    group_count: usize,
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut successor_sets: Vec<HashSet<usize>> = vec![HashSet::new(); group_count];
    for (source, destinations) in adjacency {
        let Some(&source_group) = group_of.get(source) else {
            continue;
        };
        for &destination in destinations {
            if let Some(&destination_group) = group_of.get(&destination) {
                if destination_group != source_group {
                    successor_sets[source_group].insert(destination_group);
                }
            }
        }
    }
    let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); group_count];
    let mut out_degree: Vec<usize> = vec![0; group_count];
    for (source_group, successors) in successor_sets.into_iter().enumerate() {
        out_degree[source_group] = successors.len();
        for successor in successors {
            predecessors[successor].push(source_group);
        }
    }
    (predecessors, out_degree)
}

/// D-A4 external degrees, one count per DISTINCT (source, destination)
/// pair from the deduplicated adjacency: an edge counts as external only
/// when exactly one endpoint is in the group (self-loops and intra-group
/// edges are internal and never counted).
fn external_degree_pairs(
    groups: &[Vec<FileId>],
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> Vec<(usize, usize)> {
    let group_of = build_group_lookup(groups);
    let mut degrees: Vec<(usize, usize)> = vec![(0, 0); groups.len()];
    for (source, destinations) in adjacency {
        for &destination in destinations {
            match (group_of.get(source), group_of.get(&destination)) {
                (Some(&source_group), Some(&destination_group)) => {
                    if source_group != destination_group {
                        degrees[source_group].1 += 1;
                        degrees[destination_group].0 += 1;
                    }
                }
                (Some(&source_group), None) => degrees[source_group].1 += 1,
                (None, Some(&destination_group)) => degrees[destination_group].0 += 1,
                (None, None) => {}
            }
        }
    }
    degrees
}

// ---------------------------------------------------------------------------
// Representative-cycle witness (D-A5): the closed walk from the FIRST
// back-edge of a path-tracking walk restricted to group members, iterative
// like the frame machine. Existence, not exactness, is the contract; a
// self-loop group short-circuits to [c0, c0].
// ---------------------------------------------------------------------------

fn representative_cycle_for(
    members: &[FileId],
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> Vec<FileId> {
    if let Some(self_loop_witness) = self_loop_witness(members, adjacency) {
        return self_loop_witness;
    }
    let member_set: HashSet<FileId> = members.iter().copied().collect();
    closed_walk_witness(members[0], &member_set, adjacency)
}

fn self_loop_witness(
    members: &[FileId],
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> Option<Vec<FileId>> {
    let [single] = members else {
        return None;
    };
    let has_self_loop = adjacency
        .get(single)
        .is_some_and(|destinations| destinations.contains(single));
    has_self_loop.then(|| vec![*single, *single])
}

/// One activation record of the within-group closed-walk walk.
struct WalkFrame {
    vertex: FileId,
    next_neighbor: usize,
}

/// Path-tracking DFS restricted to group members; on the first back-edge
/// u -> x (x on the current path), emit [path[pos(x)..]] + [x] (D-A5). A
/// nontrivial SCC guarantees such a back-edge before its seed's frame
/// drains: every member of a nontrivial SCC lies on a cycle (IV-2/IV-3).
fn closed_walk_witness(
    seed: FileId,
    member_set: &HashSet<FileId>,
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> Vec<FileId> {
    let mut path: Vec<FileId> = Vec::new();
    let mut visited: HashSet<FileId> = HashSet::new();
    let mut on_path: HashSet<FileId> = HashSet::new();
    let mut frames: Vec<WalkFrame> = Vec::new();
    enter_walk_vertex(seed, &mut path, &mut visited, &mut on_path, &mut frames);
    loop {
        let Some(neighbor) = claim_next_member_neighbor(&mut frames, member_set, adjacency) else {
            let retired = frames.pop().expect("loop holds a live frame");
            on_path.remove(&retired.vertex);
            path.pop();
            if frames.is_empty() {
                debug_assert!(
                    false,
                    "a nontrivial group always yields a back-edge before its seed retires"
                );
                return Vec::new();
            }
            continue;
        };
        if visited.contains(&neighbor) {
            if on_path.contains(&neighbor) {
                let position = path
                    .iter()
                    .position(|&candidate| candidate == neighbor)
                    .expect("on_path membership implies path membership");
                let mut walk = path[position..].to_vec();
                walk.push(neighbor);
                return walk;
            }
            continue;
        }
        enter_walk_vertex(neighbor, &mut path, &mut visited, &mut on_path, &mut frames);
    }
}

fn enter_walk_vertex(
    vertex: FileId,
    path: &mut Vec<FileId>,
    visited: &mut HashSet<FileId>,
    on_path: &mut HashSet<FileId>,
    frames: &mut Vec<WalkFrame>,
) {
    visited.insert(vertex);
    on_path.insert(vertex);
    path.push(vertex);
    frames.push(WalkFrame {
        vertex,
        next_neighbor: 0,
    });
}

/// The active walk frame's next neighbor INSIDE the group (non-members are
/// skipped, not merely deferred), or None when its member list is consumed.
fn claim_next_member_neighbor(
    frames: &mut [WalkFrame],
    member_set: &HashSet<FileId>,
    adjacency: &HashMap<FileId, Vec<FileId>>,
) -> Option<FileId> {
    let frame = frames.last_mut().expect("walk loop holds a live frame");
    let no_neighbors: &[FileId] = &[];
    let neighbors = adjacency
        .get(&frame.vertex)
        .map(Vec::as_slice)
        .unwrap_or(no_neighbors);
    while frame.next_neighbor < neighbors.len() {
        let candidate = neighbors[frame.next_neighbor];
        frame.next_neighbor += 1;
        if member_set.contains(&candidate) {
            return Some(candidate);
        }
    }
    None
}
