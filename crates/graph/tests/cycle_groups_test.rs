//! M23.5 T2-A -- CycleGroup contract suite (crate:graph,
//! plan 1790821308681 Phase 2, handoff tasks/m235-t2a-handoff.md).
//!
//! T2.1-R1 (IV-7) and T2.1-R2 (IV-3) were authored against the CURRENT
//! public API `detect_cycles()` and observed failing (evidence in
//! tasks/m235-t2a-journal.md); both are now re-pointed at
//! `detect_cycle_groups()` and must stay green without weakened assertions.
//! The witness verifier is the T2.2 Red (IV-4): it fails against any group
//! whose `representative_cycle` is not an edge-backed closed walk.

use ariadne_graph::{
    CycleGroup, IncludeEdge, IncludeGraph, IncludeKind, LanguageId, SourceLocation,
};
use ariadne_repo::FileId;
use std::collections::HashMap;
use std::path::PathBuf;

/// Small stack chosen by the M23.5 plan for the stress thread: deep
/// recursion cannot survive it, an iterative machine can (IV-7). Mirrors
/// `SMALL_STACK_BYTES` in crates/cluster/tests/scc_iterative_parity.rs.
const SMALL_STACK_BYTES: usize = 64 * 1024;
const CHAIN_FILE_COUNT: u64 = 100_000;

// ---------------------------------------------------------------------------
// Fixture helpers copied from crates/graph/src/closure_equivalence_tests.rs:22-46
// ---------------------------------------------------------------------------

type FixtureShape = (u64, Option<u64>);

fn edge_with_context(source: u64, destination: Option<u64>, context: &str) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: destination.map(FileId),
        context_name: context.to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: format!("f{source}.h"),
        resolved_path: None,
        language: LanguageId::cpp(),
    }
}

fn edge(source: u64, destination: Option<u64>) -> IncludeEdge {
    edge_with_context(source, destination, "M23.5")
}

fn graph_from_shapes(files: &[u64], shapes: &[FixtureShape]) -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    for &file in files {
        graph.register_file(FileId(file), PathBuf::from(format!("f{file}.h")));
    }
    for &(source, destination) in shapes {
        graph.add_edge(edge(source, destination));
    }
    graph
}

/// Deliberate MIRROR of the pub(crate) `IncludeGraph::adjacency()` at
/// crates/graph/src/graph.rs:83-95. An integration test sees only the
/// public API, so the witness verifier rebuilds the deduplicated
/// source -> sorted-destinations map here (skip `destination: None`, group
/// by source, sort, dedup). This is a test-side mirror of that contract,
/// not a second production adjacency loop.
fn test_adjacency(graph: &IncludeGraph) -> HashMap<FileId, Vec<FileId>> {
    let mut adjacency: HashMap<FileId, Vec<FileId>> = HashMap::new();
    for include_edge in &graph.edges {
        if let Some(destination) = include_edge.destination {
            adjacency
                .entry(include_edge.source)
                .or_default()
                .push(destination);
        }
    }
    for destinations in adjacency.values_mut() {
        destinations.sort();
        destinations.dedup();
    }
    adjacency
}

fn resolved_edge_pairs(graph: &IncludeGraph) -> Vec<(FileId, FileId)> {
    graph
        .edges
        .iter()
        .filter_map(|include_edge| {
            include_edge
                .destination
                .map(|dest| (include_edge.source, dest))
        })
        .collect()
}

fn same_group(groups: &[CycleGroup], first: FileId, second: FileId) -> bool {
    groups.iter().any(|group| {
        group.member_files.contains(&first) && group.member_files.contains(&second)
    })
}

// ---------------------------------------------------------------------------
// T2.1-R1 re-pointed (IV-7): 100 000-node fused chain on a 64 KiB stack.
// Red evidence (aborted binary against detect_cycles) lives in the journal.
// ---------------------------------------------------------------------------

#[test]
fn test_deep_include_chain_completes_on_default_stack() {
    let spawned = std::thread::Builder::new()
        .stack_size(SMALL_STACK_BYTES)
        .spawn(detect_deep_chain_groups)
        .expect("deep-chain stress thread spawns with the requested stack size");
    let groups = spawned
        .join()
        .expect("deep-chain stress thread completed without panicking");
    assert_eq!(
        groups.len(),
        1,
        "the fused 100 000-node chain must yield exactly one group (IV-2)"
    );
    assert_eq!(
        groups[0].member_files.len() as u64,
        CHAIN_FILE_COUNT,
        "every chain node lies on the giant cycle and belongs to the group"
    );
}

/// Builds f0 -> f1 -> ... -> f99999 -> f0 (one giant cycle) and runs the
/// new group detection. Runs on the caller's thread so the test controls
/// the stack size (IV-7).
fn detect_deep_chain_groups() -> Vec<CycleGroup> {
    let mut graph = IncludeGraph::new();
    for position in 0..CHAIN_FILE_COUNT {
        graph.register_file(FileId(position), PathBuf::from(format!("f{position}.h")));
    }
    for position in 0..CHAIN_FILE_COUNT {
        let next = (position + 1) % CHAIN_FILE_COUNT;
        graph.add_edge(edge(position, Some(next)));
    }
    graph.detect_cycle_groups()
}

// ---------------------------------------------------------------------------
// T2.1-R2 re-pointed (IV-3): the chord-blind edge.
// Red evidence (assertion naming a -> c against detect_cycles) in journal.
// ---------------------------------------------------------------------------

/// a -> b -> c -> a with the chord a -> c, plus a hanging node d off c.
fn diamond_with_chord_graph() -> IncludeGraph {
    graph_from_shapes(
        &[1, 2, 3, 4],
        &[
            (1, Some(2)),
            (2, Some(3)),
            (3, Some(1)),
            (1, Some(3)),
            (3, Some(4)),
        ],
    )
}

#[test]
fn test_diamond_with_chord_reports_every_intra_group_edge_as_cyclic() {
    let graph = diamond_with_chord_graph();
    let groups = graph.detect_cycle_groups();
    assert_eq!(
        groups.len(),
        1,
        "a, b, c entangle into exactly one group (IV-1)"
    );
    assert_eq!(
        groups[0].member_files,
        vec![FileId(1), FileId(2), FileId(3)]
    );
    assert!(
        !groups
            .iter()
            .any(|group| group.member_files.contains(&FileId(4))),
        "hanging node d must belong to no group (IV-2)"
    );
    for (source, destination) in resolved_edge_pairs(&graph) {
        let source_is_cyclic = groups
            .iter()
            .any(|group| group.member_files.contains(&source));
        let destination_is_cyclic = groups
            .iter()
            .any(|group| group.member_files.contains(&destination));
        if source_is_cyclic && destination_is_cyclic {
            assert!(
                same_group(&groups, source, destination),
                "edge FileId({}) -> FileId({}) (the a -> c chord): both endpoints are \
                 cyclic yet the edge is not intra-group, so it would render unstyled \
                 between two cycle-classed nodes (IV-3)",
                source.0,
                destination.0
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Plan T2.1 fixture list.
// ---------------------------------------------------------------------------

#[test]
fn test_two_cycle_yields_one_group_with_closed_witness() {
    // Mirrors the graph.rs:501-539 unit fixture.
    let graph = graph_from_shapes(&[1, 2], &[(1, Some(2)), (2, Some(1))]);
    let groups = graph.detect_cycle_groups();
    assert_eq!(groups.len(), 1);
    let group = &groups[0];
    assert_eq!(group.member_files, vec![FileId(1), FileId(2)]);
    assert_eq!(
        group.representative_cycle,
        vec![FileId(1), FileId(2), FileId(1)],
        "witness is a closed walk of length 3 closing [a, b, a] (IV-4)"
    );
    assert_eq!(group.external_in_count, 0);
    assert_eq!(group.external_out_count, 0);
}

#[test]
fn test_self_include_yields_single_member_group() {
    let graph = graph_from_shapes(&[1], &[(1, Some(1))]);
    let groups = graph.detect_cycle_groups();
    assert_eq!(groups.len(), 1, "a self-include is a cycle (IV-2)");
    let group = &groups[0];
    assert_eq!(group.member_files, vec![FileId(1)]);
    assert_eq!(
        group.representative_cycle,
        vec![FileId(1), FileId(1)],
        "a self-loop group short-circuits to [c0, c0] (D-A5, IV-4)"
    );
    assert_eq!(group.external_in_count, 0);
    assert_eq!(group.external_out_count, 0);
}

#[test]
fn test_two_disjoint_groups_emit_in_reverse_topological_order() {
    // Connector 2 -> 3 makes {1,2} upstream of {3,4}: the sink group emits
    // first under reverse-topological order (IV-5, D-A2).
    let connected = graph_from_shapes(
        &[1, 2, 3, 4],
        &[
            (1, Some(2)),
            (2, Some(1)),
            (3, Some(4)),
            (4, Some(3)),
            (2, Some(3)),
        ],
    );
    let groups = connected.detect_cycle_groups();
    assert_eq!(
        groups[0].member_files,
        vec![FileId(3), FileId(4)],
        "the condensation sink emits first (IV-5)"
    );
    assert_eq!(groups[1].member_files, vec![FileId(1), FileId(2)]);
    assert_eq!(
        connected.detect_cycle_groups(),
        groups,
        "IV-5 determinism: two runs of the same graph must produce identical groups"
    );
    // Incomparable groups (no connector) exercise the D-A2 min-heap
    // tie-break: smallest member ascending.
    let incomparable = graph_from_shapes(
        &[1, 2, 3, 4],
        &[
            (1, Some(2)),
            (2, Some(1)),
            (3, Some(4)),
            (4, Some(3)),
        ],
    );
    let tied = incomparable.detect_cycle_groups();
    assert_eq!(
        tied[0].member_files,
        vec![FileId(1), FileId(2)],
        "incomparable groups tie-break on the smallest member ascending (D-A2)"
    );
    assert_eq!(tied[1].member_files, vec![FileId(3), FileId(4)]);
}

#[test]
fn test_acyclic_and_empty_graphs_yield_no_groups() {
    let empty = IncludeGraph::new();
    assert!(
        empty.detect_cycle_groups().is_empty(),
        "an empty graph yields no groups"
    );
    let acyclic = graph_from_shapes(&[1, 2, 3], &[(1, Some(2)), (2, Some(3))]);
    assert!(
        acyclic.detect_cycle_groups().is_empty(),
        "an acyclic graph yields no groups (IV-2 negative)"
    );
}

#[test]
fn test_external_degrees_ignore_intra_group_edges_and_self_loops() {
    // Group {1,2}: internal 1 -> 2, 2 -> 1, self-loop 1 -> 1 (all internal,
    // D-A4). External: 8 -> 1 enters, 2 -> 7 leaves, with 7 and 8 ungrouped.
    let mut graph = graph_from_shapes(
        &[1, 2, 7, 8],
        &[
            (1, Some(2)),
            (2, Some(1)),
            (1, Some(1)),
            (8, Some(1)),
            (2, Some(7)),
        ],
    );
    // A second RECORD of the SAME pair (2,7): add_edge (graph.rs:42-46)
    // only rejects exact duplicate records, so D-A4's per-distinct-pair
    // counting must fold it.
    graph.add_edge(edge_with_context(2, Some(7), "DUPLICATE-RECORD"));
    let groups = graph.detect_cycle_groups();
    assert_eq!(groups.len(), 1);
    let group = &groups[0];
    assert_eq!(group.member_files, vec![FileId(1), FileId(2)]);
    assert_eq!(
        group.external_in_count, 1,
        "only the 8 -> 1 pair enters the group (D-A4)"
    );
    assert_eq!(
        group.external_out_count, 1,
        "the duplicate (2,7) record must count once (D-A4)"
    );
}

// ---------------------------------------------------------------------------
// T2.2: seeded witness verifier (IV-4). Pattern copied from
// closure_equivalence_tests.rs:373-416 (InlineLcg + locked-seed table).
// ---------------------------------------------------------------------------

/// Knuth MMIX LCG: multiply-add constants only, deterministic per seed.
#[derive(Clone, Copy)]
struct InlineLcg(u64);

impl InlineLcg {
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }

    fn percent(&mut self, chance: u64) -> bool {
        self.next_u64() % 100 < chance
    }
}

#[derive(Clone, Copy)]
enum Topology {
    PureDag,
    BigCycle,
    TwoSccsBridge,
    SelfLoops,
    FullBlob,
}

/// Seeds are locked: changing them changes the fixture corpus.
const WITNESS_FIXTURES: [(u64, usize, Topology, &str); 8] = [
    (0x0D10_D000_0000_00A6, 20, Topology::PureDag, "pure-dag"),
    (0xC0FF_EE00_0000_0011, 18, Topology::BigCycle, "big-cycle"),
    (
        0xBADC_AFE0_0000_0002,
        21,
        Topology::TwoSccsBridge,
        "two-sccs-bridge",
    ),
    (0x5E1F_10AD_0000_0003, 16, Topology::SelfLoops, "self-loops"),
    (0xB10B_10B0_0000_0004, 12, Topology::FullBlob, "full-blob"),
    (0xC0FF_EE00_0000_0012, 24, Topology::BigCycle, "big-cycle-b"),
    (0x5E1F_10AD_0000_0004, 15, Topology::SelfLoops, "self-loops-b"),
    (0xB10B_10B0_0000_0005, 14, Topology::FullBlob, "full-blob-b"),
];

fn random_graph(seed: u64, file_count: usize, topology: Topology) -> IncludeGraph {
    let mut rng = InlineLcg(seed);
    let files: Vec<u64> = (1..=file_count as u64).collect();
    let mut graph = IncludeGraph::new();
    for &file in &files {
        graph.register_file(FileId(file), PathBuf::from(format!("rnd{file}.h")));
    }
    let link = |graph: &mut IncludeGraph, from: usize, to: usize| {
        graph.add_edge(edge(files[from], Some(files[to])));
    };
    match topology {
        Topology::PureDag => {
            for source_index in 0..file_count {
                for destination_index in (source_index + 1)..file_count {
                    if rng.percent(22) {
                        link(&mut graph, source_index, destination_index);
                    }
                }
            }
        }
        Topology::BigCycle => {
            for source_index in 0..file_count {
                link(&mut graph, source_index, (source_index + 1) % file_count);
            }
            for _ in 0..file_count {
                link(&mut graph, rng.below(file_count), rng.below(file_count));
            }
        }
        Topology::TwoSccsBridge => {
            let first = file_count / 3;
            let second = 2 * file_count / 3;
            cyclic_chain(&mut graph, &mut rng, &files, 0, first);
            cyclic_chain(&mut graph, &mut rng, &files, first, second);
            for chain_position in second..file_count.saturating_sub(1) {
                link(&mut graph, chain_position, chain_position + 1);
            }
            link(&mut graph, 1, first + 1);
            link(&mut graph, first + 1, second);
        }
        Topology::SelfLoops => {
            for source_index in 0..file_count {
                link(&mut graph, source_index, (source_index + 1) % file_count);
                if source_index % 3 == 0 {
                    link(&mut graph, source_index, source_index);
                }
            }
            for _ in 0..file_count {
                link(&mut graph, rng.below(file_count), rng.below(file_count));
            }
        }
        Topology::FullBlob => {
            for source_index in 0..file_count {
                for destination_index in 0..file_count {
                    if source_index != destination_index && rng.percent(35) {
                        link(&mut graph, source_index, destination_index);
                    }
                }
            }
        }
    }
    // Unresolved (destination: None) edges in every fixture, so detection
    // must ignore them exactly like the adjacency contract requires.
    for _ in 0..3 {
        let source = files[rng.below(file_count)];
        graph.add_edge(edge(source, None));
    }
    graph
}

/// Cycle plus skip chords over `files[start..end]`, used for the two SCCs.
fn cyclic_chain(
    graph: &mut IncludeGraph,
    rng: &mut InlineLcg,
    files: &[u64],
    start: usize,
    end: usize,
) {
    debug_assert!(end - start >= 2);
    for chain_position in start..end {
        let next = start + (chain_position - start + 1) % (end - start);
        graph.add_edge(edge(files[chain_position], Some(files[next])));
        if rng.percent(30) {
            let skip = start + (chain_position - start + 2) % (end - start);
            graph.add_edge(edge(files[chain_position], Some(files[skip])));
        }
    }
}

/// IV-4 witness contract, checked edge-by-edge against the graph: closed
/// (`first == last`), at least one step long, every consecutive pair is a
/// real include edge, and no step leaves the group.
fn assert_witness_is_edge_backed(group: &CycleGroup, adjacency: &HashMap<FileId, Vec<FileId>>, label: &str) {
    let witness = &group.representative_cycle;
    assert_eq!(
        witness.first(),
        witness.last(),
        "group {label} witness {:?} must be a closed walk with first == last (IV-4)",
        witness
    );
    assert!(
        witness.len() >= 2,
        "group {label} witness {:?} must span at least one closing step (IV-4)",
        witness
    );
    for step in witness.windows(2) {
        assert!(
            adjacency
                .get(&step[0])
                .is_some_and(|destinations| destinations.contains(&step[1])),
            "group {label} witness step {:?} -> {:?} is not a real include edge (IV-4)",
            step[0],
            step[1]
        );
        assert!(
            group.member_files.contains(&step[0])
                && group.member_files.contains(&step[1]),
            "group {label} witness step {:?} -> {:?} leaves the group's members (IV-4)",
            step[0],
            step[1]
        );
    }
}

#[test]
fn test_seeded_digraphs_witnesses_are_edge_backed_closed_walks() {
    for &(seed, file_count, topology, name) in WITNESS_FIXTURES.iter() {
        let graph = random_graph(seed, file_count, topology);
        let adjacency = test_adjacency(&graph);
        let groups = graph.detect_cycle_groups();
        match topology {
            Topology::PureDag => assert!(
                groups.is_empty(),
                "`{name}` (seed {seed}): a DAG must yield no groups for the corpus \
                 to cover both regimes (IV-2)"
            ),
            _ => assert!(
                !groups.is_empty(),
                "`{name}` (seed {seed}): must contain cycles for the corpus to be meaningful"
            ),
        }
        for group in &groups {
            assert_witness_is_edge_backed(group, &adjacency, name);
        }
    }
}

const TANGLE_FILE_COUNT: u64 = 200;
const TANGLE_SEED: u64 = 0x7A7A_1E00_0000_0009;
const KNOWN_CYCLE_FIRST: u64 = 101;
const KNOWN_CYCLE_SECOND: u64 = 102;

#[test]
fn test_known_two_cycle_survives_in_random_tangle() {
    let mut rng = InlineLcg(TANGLE_SEED);
    let mut graph = IncludeGraph::new();
    for file in 1..=TANGLE_FILE_COUNT {
        graph.register_file(FileId(file), PathBuf::from(format!("t{file}.h")));
    }
    for _ in 0..(4 * TANGLE_FILE_COUNT) {
        let source = 1 + rng.below(TANGLE_FILE_COUNT as usize) as u64;
        let destination = 1 + rng.below(TANGLE_FILE_COUNT as usize) as u64;
        graph.add_edge(edge(source, Some(destination)));
    }
    graph.add_edge(edge(KNOWN_CYCLE_FIRST, Some(KNOWN_CYCLE_SECOND)));
    graph.add_edge(edge(KNOWN_CYCLE_SECOND, Some(KNOWN_CYCLE_FIRST)));

    let groups = graph.detect_cycle_groups();
    let known_group = groups
        .iter()
        .find(|group| {
            group.member_files.contains(&FileId(KNOWN_CYCLE_FIRST))
                && group.member_files.contains(&FileId(KNOWN_CYCLE_SECOND))
        })
        .expect("the embedded 101 <-> 102 pair must share one cycle group (IV-2)");
    assert_witness_is_edge_backed(
        known_group,
        &test_adjacency(&graph),
        "two-cycle-in-200-node-tangle",
    );
    assert!(
        known_group.representative_cycle.len() >= 3,
        "a group containing a real 2-cycle closes over at least two distinct \
         members plus the closing step (existence, not exactness: D-A5)"
    );
}
