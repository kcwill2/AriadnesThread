//! M23.5 T2-B -- equivalence-to-old safety net (crate:graph,
//! plan 1790821308681 Phase 2 T2.3, handoff tasks/m235-t2b-handoff.md).
//!
//! `old_detect_cycles` / `old_dfs_cycle` below are a FROZEN COPY of the old
//! recursive path enumeration (`crates/graph/src/graph.rs:234-287`, blob
//! `2708425`) and are the definition of correct (IV-6). They may never be
//! edited to accommodate `detect_cycle_groups`; the copy is test-local so the
//! suite keeps holding the old semantics after T3.1 deletes the live method.
//!
//! The NAMED divergence case (handoff sec 4) is
//! `test_unregistered_endpoint_cycle_is_found_by_groups_but_not_by_old_paths`:
//! old seeds DFS from `file_paths` keys only (`graph.rs:240-241`), the new
//! seed universe is the union of registered files plus every edge source
//! plus every resolved destination (D-A1). A cycle whose endpoints were
//! never `register_file`'d is unreachable from old seeds, so old misses it
//! and new finds it -- the old miss is the IV-2 bug, so this case carries NO
//! equality claim; every other corpus graph has all edge endpoints
//! registered and is asserted under exact set equality.

use ariadne_graph::{
    CycleGroup, IncludeEdge, IncludeGraph, IncludeKind, LanguageId, SourceLocation,
};
use ariadne_repo::FileId;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

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
    edge_with_context(source, destination, "M23.5-T2B")
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

// ---------------------------------------------------------------------------
// The frozen oracle: verbatim copy of graph.rs:234-287 at blob 2708425.
// Forced test-locality adaptations (logged in tasks/m235-t2b-journal.md):
//   - receiver `self.` -> parameter `graph.` (free fn, no impl block)
//   - `Self::dfs_cycle` -> `old_dfs_cycle` (private associated fn -> free fn)
//   - `self.adjacency()` -> `old_adjacency(&graph)` (pub(crate) call site)
// No local inside either body was renamed; the bodies are otherwise verbatim.
// ---------------------------------------------------------------------------

/// Deliberate MIRROR of the pub(crate) `IncludeGraph::adjacency()` at
/// crates/graph/src/graph.rs:83-95 (skip `destination: None`, group by
/// source, sort each destination list, dedup). The oracle needs the map and
/// an integration test cannot call the pub(crate) method, so this rebuilds
/// it from the public `graph.edges` exactly as that loop does -- a test-side
/// mirror of the contract, not a second production adjacency loop.
fn old_adjacency(graph: &IncludeGraph) -> HashMap<FileId, Vec<FileId>> {
    let mut adj: HashMap<FileId, Vec<FileId>> = HashMap::new();
    for edge in &graph.edges {
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

fn old_detect_cycles(graph: &IncludeGraph) -> Vec<Vec<FileId>> {
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut current_path = Vec::new();

    let mut all_files: Vec<FileId> = graph.file_paths.keys().copied().collect();
    all_files.sort();
    let adj = old_adjacency(graph);

    for &node in &all_files {
        if !visited.contains(&node) {
            old_dfs_cycle(
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

fn old_dfs_cycle(
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
                old_dfs_cycle(neighbor, adj, visited, rec_stack, current_path, cycles);
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

// ---------------------------------------------------------------------------
// Corpus: every existing fixture shape (closure_equivalence_tests.rs:59-176),
// three T2-B pins, the locked-seed random corpus
// (closure_equivalence_tests.rs:373-490 -- SAME seed values), a 200-node
// tangle, and the named divergence graph.
// ---------------------------------------------------------------------------

/// Load-analysis suite shapes, copied verbatim from
/// closure_equivalence_tests.rs:59-140.
fn core_fixture_shapes() -> Vec<(&'static str, Vec<u64>, Vec<FixtureShape>)> {
    vec![
        (
            "graph.rs closure/explain fixture",
            vec![1, 2, 3, 4],
            vec![(1, Some(2)), (2, Some(3)), (2, Some(4))],
        ),
        (
            "graph.rs 2-cycle fixture (detect_cycles)",
            vec![1, 2],
            vec![(1, Some(2)), (2, Some(1))],
        ),
        (
            "chain/justification fixture",
            vec![1, 2, 3, 4],
            vec![(1, Some(2)), (1, Some(4)), (2, Some(3))],
        ),
        (
            "seed/required fixture (edgeless registered)",
            vec![1, 2, 3],
            vec![],
        ),
        (
            "tiebreak fixture (edgeless registered)",
            vec![1, 2, 3],
            vec![],
        ),
        ("ghost fixture (single registered file)", vec![50], vec![]),
        (
            "concurrency fixture",
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![
                (1, Some(2)),
                (1, Some(4)),
                (2, Some(3)),
                (5, Some(6)),
                (6, Some(7)),
                (8, Some(9)),
            ],
        ),
        (
            "impact base fixture",
            vec![1, 2, 3, 4, 5, 6],
            vec![
                (1, Some(2)),
                (2, Some(3)),
                (2, Some(4)),
                (4, Some(5)),
                (4, Some(6)),
            ],
        ),
        (
            "impact sibling-trap variant (main->d)",
            vec![1, 2, 3, 4, 5, 6],
            vec![
                (1, Some(2)),
                (2, Some(3)),
                (2, Some(4)),
                (4, Some(5)),
                (4, Some(6)),
                (1, Some(5)),
            ],
        ),
        (
            "impact partial-trap variant (main->c)",
            vec![1, 2, 3, 4, 5, 6],
            vec![
                (1, Some(2)),
                (2, Some(3)),
                (2, Some(4)),
                (4, Some(5)),
                (4, Some(6)),
                (1, Some(4)),
            ],
        ),
        (
            "ei9 fixture (edgeless registered)",
            vec![1, 2, 3, 4, 5],
            vec![],
        ),
    ]
}

/// Characterization-suite shapes, copied verbatim from
/// closure_equivalence_tests.rs:144-172.
fn characterization_fixture_shapes() -> Vec<(&'static str, Vec<u64>, Vec<FixtureShape>)> {
    vec![
        (
            "characterization mock data",
            vec![1, 2, 3],
            vec![(1, Some(2)), (2, Some(3))],
        ),
        (
            "characterization same-file collateral",
            vec![10, 11],
            vec![(10, Some(11))],
        ),
        (
            "characterization unregistered edge-only ids",
            vec![22],
            vec![(22, Some(20)), (20, Some(21))],
        ),
        (
            "characterization pathless '..' host",
            vec![40, 41],
            vec![(40, Some(41))],
        ),
        (
            "unresolved-only source (never registered)",
            vec![71],
            vec![(70, None), (71, Some(72))],
        ),
    ]
}

/// Shapes added by T2-B: the IV-5 tie-break pin (two INCOMPARABLE groups --
/// the mutation bite case of handoff sec5), the T2-A diamond-with-chord
/// shape (IV-3 per-edge reachability on the chord), and a self-loop
/// singleton shape.
fn t2b_added_fixture_shapes() -> Vec<(&'static str, Vec<u64>, Vec<FixtureShape>)> {
    vec![
        (
            "T2-B incomparable pair pin",
            vec![1, 2, 3, 4],
            vec![(1, Some(2)), (2, Some(1)), (3, Some(4)), (4, Some(3))],
        ),
        (
            "T2-B diamond-with-chord shape",
            vec![1, 2, 3, 4],
            vec![
                (1, Some(2)),
                (2, Some(3)),
                (3, Some(1)),
                (1, Some(3)),
                (3, Some(4)),
            ],
        ),
        ("T2-B self-loop fixture", vec![1], vec![(1, Some(1))]),
    ]
}

/// Knuth MMIX LCG: multiply-add constants only, deterministic per seed.
/// Copied from closure_equivalence_tests.rs:373-393.
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

/// Seeds are locked: identical to closure_equivalence_tests.rs:405-416 --
/// changing them changes the shared fixture corpus.
const RANDOM_FIXTURES: [(u64, usize, Topology, &str); 5] = [
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
];

/// Copied from closure_equivalence_tests.rs:418-490; loop locals renamed
/// source_index/destination_index/chain_position following the sanctioned
/// T2-A precedent in cycle_groups_test.rs. The LCG constants and control
/// flow are untouched, so the seeded corpus is identical.
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
            // Strict one-way bridges so the two SCCs stay distinct.
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
    // Unresolved (destination: None) edges are sprinkled into every random
    // fixture so both paths must ignore them identically.
    for _ in 0..3 {
        let source = files[rng.below(file_count)];
        graph.add_edge(edge(source, None));
    }
    graph
}

/// Cycle plus skip chords over `files[start..end]`, used for the two SCCs.
/// Copied from closure_equivalence_tests.rs:493-509 with the same loop-local
/// rename precedent.
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

const TANGLE_SEED: u64 = 0x7A7A_1E00_0000_0009;
const TANGLE_FILE_COUNT: u64 = 200;
const KNOWN_CYCLE_FIRST: u64 = 101;
const KNOWN_CYCLE_SECOND: u64 = 102;

/// Dense 200-node seeded tangle with an embedded known 2-cycle, built as in
/// cycle_groups_test.rs:531-544 (same seed and constants).
fn tangle_graph() -> IncludeGraph {
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
    graph
}

/// THE NAMED DIVERGENCE CASE (handoff sec 4). Files 401/402/403 are NEVER
/// registered; 401 <-> 402 form a real include cycle plus a dangling
/// 402 -> 403. Registered files 1 and 2 cannot reach the cycle, so the old
/// `file_paths`-only seeding (graph.rs:240-241) never walks it and reports
/// nothing, while the D-A1 union-seeded groups find group {401, 402}. The
/// old miss IS the IV-2 bug (401/402 are on a real cycle), so the new result
/// is the correct one and this graph carries no equality claim -- it is the
/// single named exception, asserted in
/// `test_unregistered_endpoint_cycle_is_found_by_groups_but_not_by_old_paths`.
fn named_divergence_graph() -> IncludeGraph {
    graph_from_shapes(
        &[1, 2],
        &[
            (1, Some(2)),
            (401, Some(402)),
            (402, Some(401)),
            (402, Some(403)),
        ],
    )
}

fn fixture_corpus() -> Vec<(&'static str, IncludeGraph)> {
    core_fixture_shapes()
        .into_iter()
        .chain(characterization_fixture_shapes())
        .chain(t2b_added_fixture_shapes())
        .map(|(name, files, shapes)| (name, graph_from_shapes(&files, &shapes)))
        .collect()
}

fn seeded_corpus() -> Vec<(&'static str, IncludeGraph)> {
    let mut corpus: Vec<(&'static str, IncludeGraph)> = RANDOM_FIXTURES
        .iter()
        .map(|&(seed, file_count, topology, name)| (name, random_graph(seed, file_count, topology)))
        .collect();
    corpus.push(("tangle-200", tangle_graph()));
    corpus
}

/// The whole corpus INCLUDING the named divergence graph -- used by the two
/// one-directional assertions (path containment, new-never-subset-of-old),
/// which are safe on the divergence graph because the old set is empty there.
fn full_corpus() -> Vec<(&'static str, IncludeGraph)> {
    let mut corpus = fixture_corpus();
    corpus.extend(seeded_corpus());
    corpus.push(("named-divergence", named_divergence_graph()));
    corpus
}

// ---------------------------------------------------------------------------
// Shared assertions
// ---------------------------------------------------------------------------

/// True iff every edge source and every RESOLVED destination is a registered
/// file -- the exact-equality gate of handoff sec 4. Unresolved destinations
/// are not files for any traversal, so they do not count as endpoints.
fn all_edge_endpoints_registered(graph: &IncludeGraph) -> bool {
    graph.edges.iter().all(|include_edge| {
        graph.file_paths.contains_key(&include_edge.source)
            && include_edge
                .destination
                .map_or(true, |dest| graph.file_paths.contains_key(&dest))
    })
}

fn old_cyclic_node_set(graph: &IncludeGraph) -> HashSet<FileId> {
    old_detect_cycles(graph)
        .into_iter()
        .flatten()
        .collect()
}

fn new_cyclic_node_set(groups: &[CycleGroup]) -> HashSet<FileId> {
    groups
        .iter()
        .flat_map(|group| group.member_files.iter().copied())
        .collect()
}

fn member_orderings(groups: &[CycleGroup]) -> Vec<Vec<u64>> {
    groups
        .iter()
        .map(|group| group.member_files.iter().map(|member| member.0).collect())
        .collect()
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

/// IV-1: no member file may appear in two groups.
fn assert_partitions_disjoint(groups: &[CycleGroup], label: &str) {
    let mut seen: HashSet<FileId> = HashSet::new();
    for group in groups {
        for &member in &group.member_files {
            assert!(
                seen.insert(member),
                "`{label}`: IV-1 violated -- FileId({}) appears in two groups",
                member.0
            );
        }
    }
}

/// Within-group reachability: can `from` reach `target` following adjacency
/// edges whose heads are all members of `member_set`?
fn reaches_within_group(
    adjacency: &HashMap<FileId, Vec<FileId>>,
    member_set: &HashSet<FileId>,
    from: FileId,
    target: FileId,
) -> bool {
    let mut seen: HashSet<FileId> = HashSet::from([from]);
    let mut queue: VecDeque<FileId> = VecDeque::from([from]);
    let no_neighbors: &[FileId] = &[];
    while let Some(current) = queue.pop_front() {
        if current == target {
            return true;
        }
        for &next in adjacency
            .get(&current)
            .map(Vec::as_slice)
            .unwrap_or(no_neighbors)
        {
            if member_set.contains(&next) && seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    false
}

/// IV-3 CONSEQUENCE, asserted per EDGE (T2-A verdict finding 3): every
/// resolved edge whose endpoints lie in one group must have its destination
/// reach its source back inside that group -- i.e. the edge lies on a cycle
/// contained in the group. A Tarjan defect that over-merges two SCCs puts a
/// pair of members in one group with no return path, which membership
/// proxies cannot see but this assertion catches directly.
fn assert_intra_group_edges_lie_on_group_cycles(
    graph: &IncludeGraph,
    groups: &[CycleGroup],
    label: &str,
) {
    let adjacency = old_adjacency(graph);
    let resolved_pairs = resolved_edge_pairs(graph);
    for group in groups {
        let member_set: HashSet<FileId> = group.member_files.iter().copied().collect();
        for (source, destination) in &resolved_pairs {
            if !member_set.contains(source) || !member_set.contains(destination) {
                continue;
            }
            assert!(
                reaches_within_group(&adjacency, &member_set, *destination, *source),
                "`{label}`: IV-3 consequence violated -- intra-group edge FileId({}) \
                 -> FileId({}) of group {:?} has no return path to its source inside \
                 the group, so the edge is on no cycle within the group (over-merged \
                 group or non-SCC partition)",
                source.0,
                destination.0,
                group.member_files
            );
        }
    }
}

/// Exact-equality parity claim for one registered-endpoint graph: IV-2 both
/// directions, IV-1 disjointness, IV-3 per-edge consequence.
fn assert_membership_equals_oracle(graph: &IncludeGraph, label: &str) {
    let old_cyclic = old_cyclic_node_set(graph);
    let groups = graph.detect_cycle_groups();
    let new_cyclic = new_cyclic_node_set(&groups);

    let mut missed: Vec<u64> = old_cyclic
        .difference(&new_cyclic)
        .map(|member| member.0)
        .collect();
    missed.sort();
    assert!(
        missed.is_empty(),
        "`{label}`: IV-2 (old -> new) violated -- files {missed:?} lie on an old \
         reported cycle but are in no new group's member_files",
    );

    let mut extra: Vec<u64> = new_cyclic
        .difference(&old_cyclic)
        .map(|member| member.0)
        .collect();
    extra.sort();
    assert!(
        extra.is_empty(),
        "`{label}`: IV-2 (new -> old) violated -- files {extra:?} are in a new \
         group but lie on no old reported cycle",
    );

    assert_partitions_disjoint(&groups, label);
    assert_intra_group_edges_lie_on_group_cycles(graph, &groups, label);
}

// ---------------------------------------------------------------------------
// Test 1 -- IV-2 exact equality (both directions) on registered-endpoint
// graphs, IV-1, IV-3 per edge, and the IV-5 tie-break order lock that the
// mutation experiment of handoff sec5 must bite.
// ---------------------------------------------------------------------------

#[test]
fn test_group_membership_equals_old_cycle_node_set_on_registered_corpus() {
    let mut exact_cases: usize = 0;
    for (name, graph) in fixture_corpus().into_iter().chain(seeded_corpus()) {
        if all_edge_endpoints_registered(&graph) {
            assert_membership_equals_oracle(&graph, name);
            exact_cases += 1;
        }
    }
    assert!(
        exact_cases >= 20,
        "the exact-equality regime must be genuinely swept, got only {exact_cases} \
         registered-endpoint graphs (a corpus that silently shrinks to zero proves \
         nothing -- Phase 1 verdict finding 2 class)",
    );

    // IV-5 order lock on two INCOMPARABLE groups: the condensation is two
    // sinks, so only the tie-break can decide the order, and IV-5 pins it to
    // smallest member ascending. This is the assertion the reverse-tie-break
    // mutation of handoff sec5 Step 2 must fail.
    let pinned = graph_from_shapes(
        &[1, 2, 3, 4],
        &[
            (1, Some(2)),
            (2, Some(1)),
            (3, Some(4)),
            (4, Some(3)),
        ],
    );
    let orderings = member_orderings(&pinned.detect_cycle_groups());
    assert_eq!(
        orderings,
        vec![vec![1, 2], vec![3, 4]],
        "IV-5 tie-break: the two INCOMPARABLE groups {{1,2}} and {{3,4}} must emit \
         smallest-member-first -- expected orderings [[1,2],[3,4]]"
    );
}

// ---------------------------------------------------------------------------
// Test 2 -- path containment: no old reported cycle straddles two groups.
// ---------------------------------------------------------------------------

#[test]
fn test_old_cycle_members_are_subset_of_a_single_group() {
    for (name, graph) in full_corpus() {
        let groups = graph.detect_cycle_groups();
        for old_cycle in old_detect_cycles(&graph) {
            let hosts_one_group = groups.iter().any(|group| {
                old_cycle
                    .iter()
                    .all(|member| group.member_files.contains(member))
            });
            assert!(
                hosts_one_group,
                "`{name}`: old reported cycle {old_cycle:?} is not a subset of any \
                 single group's member_files -- old DFS paths must never straddle \
                 two new groups (path containment, handoff sec1)",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 3 -- new is never a subset of old (the safe direction on the whole
// corpus, including the unregistered-endpoint divergence regime of sec 4),
// plus the corpus regime-mix lock.
// ---------------------------------------------------------------------------

#[test]
fn test_new_groups_never_miss_a_file_the_old_enumeration_called_cyclic() {
    let mut saw_acyclic_regime = false;
    let mut saw_nontrivial_group = false;
    for (name, graph) in full_corpus() {
        let old_cyclic = old_cyclic_node_set(&graph);
        let groups = graph.detect_cycle_groups();
        let new_cyclic = new_cyclic_node_set(&groups);

        let mut missed: Vec<u64> = old_cyclic
            .difference(&new_cyclic)
            .map(|member| member.0)
            .collect();
        missed.sort();
        assert!(
            missed.is_empty(),
            "`{name}`: the new groups MISS files {missed:?} that the old enumeration \
             called cyclic -- new output must never be a subset of old",
        );

        saw_acyclic_regime |= old_cyclic.is_empty();
        saw_nontrivial_group |= groups
            .iter()
            .any(|group| group.member_files.len() >= 2);
    }
    assert!(
        saw_acyclic_regime,
        "corpus regime mix: no corpus graph had zero cyclic files -- an all-cycle \
         corpus would pass test 1 vacuously (handoff sec5)"
    );
    assert!(
        saw_nontrivial_group,
        "corpus regime mix: no corpus graph produced a group of >= 2 members -- an \
         all-DAG corpus would pass test 1 vacuously (handoff sec5)"
    );
}

// ---------------------------------------------------------------------------
// Test 4 -- THE NAMED DIVERGENCE CASE (handoff sec 4). Old seeds from
// `file_paths` only (graph.rs:240-241); the cycle 401 <-> 402 is unreachable
// from every registered seed, so old finds NOTHING and the union-seeded
// groups find group {401, 402}. Old and new legitimately differ here because
// the old miss is precisely the IV-2 bug the union seed universe (D-A1)
// fixes: 401 and 402 lie on a real include cycle. This is also the seed
// universe lock demanded by T2-A verdict finding 1: collapsing
// `tarjan_seed_universe` back to `file_paths`-only seeding would emit no
// group here and fail the assertion below, while leaving every T2-A test
// green -- every T2-A fixture registers its endpoints.
// ---------------------------------------------------------------------------

#[test]
fn test_unregistered_endpoint_cycle_is_found_by_groups_but_not_by_old_paths() {
    let graph = named_divergence_graph();
    assert!(
        !all_edge_endpoints_registered(&graph),
        "the named divergence case must genuinely contain an unregistered edge \
         endpoint, or it is not the divergence case"
    );

    let old_paths = old_detect_cycles(&graph);
    assert!(
        old_paths.is_empty(),
        "named divergence case: the old file_paths-only enumeration must report no \
         cycle here -- seeds 1 and 2 cannot reach 401/402 (this is the IV-2 bug)",
    );

    let groups = graph.detect_cycle_groups();
    assert_eq!(
        groups.len(),
        1,
        "named divergence case: exactly one cyclic group is expected",
    );
    assert_eq!(
        groups[0].member_files,
        vec![FileId(401), FileId(402)],
        "named divergence case / seed-universe lock (T2-A verdict finding 1): the \
         unregistered cycle 401 <-> 402 must be found by the D-A1 union seeding; \
         file_paths-only seeding would emit no group here"
    );

    let old_cyclic = old_cyclic_node_set(&graph);
    let new_cyclic = new_cyclic_node_set(&groups);
    assert!(
        old_cyclic.is_subset(&new_cyclic) && old_cyclic.len() != new_cyclic.len(),
        "named divergence case: new must be a STRICT superset of old here \
         (old {old_cyclic:?} vs new {new_cyclic:?}) -- the safe direction of the \
        sec 4 scoping, asserted without an equality claim"
    );
    assert!(
        !new_cyclic.contains(&FileId(403)),
        "named divergence case: dangling unregistered node 403 is on no cycle and \
         must be in no group (IV-2 negative on the unregistered universe)"
    );
}
