//! D1 equivalence suite (plan 1790608181551): the SCC-condensation +
//! reverse-topological DP rewrite of `compute_closures` must reproduce the
//! BFS oracle's closure tables EXACTLY — same key set, same sorted/deduped
//! `direct_includes` and `transitive_includes` element order (I3), same
//! `closure_size`. The oracle is `IncludeGraph::compute_closures_bfs_oracle`,
//! the verbatim pre-D1 per-file-BFS body; it is the definition of correct
//! and may never be edited to accommodate the rewrite.
//!
//! Only the include-graph SHAPE of each existing fixture is reproduced here
//! (the semantic model is irrelevant to `compute_closures`), covering every
//! graph test fixture in the crate: the closure/explain and 2-cycle fixtures
//! from `graph.rs`, the chain/justification/seed/tiebreak/ei9 shapes, the
//! concurrency shape, the extraction-impact base plus its trap variants, and
//! the characterization shapes (single-hop, unregistered edge-only ids).

use super::IncludeGraph;
use crate::types::{IncludeEdge, IncludeKind, LanguageId, SourceLocation};
use ariadne_repo::FileId;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

type FixtureShape = (u64, Option<u64>);

fn edge(source: u64, destination: Option<u64>) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: destination.map(FileId),
        context_name: "D1".to_string(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: format!("f{source}.h"),
        resolved_path: None,
        language: LanguageId::cpp(),
    }
}

fn graph_from_shapes(files: &[u64], edges: &[FixtureShape]) -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    for &file in files {
        graph.register_file(FileId(file), PathBuf::from(format!("f{file}.h")));
    }
    for &(source, destination) in edges {
        graph.add_edge(edge(source, destination));
    }
    graph
}

/// Every existing include-graph fixture shape in this crate, labeled.
fn existing_fixture_graphs() -> Vec<(&'static str, IncludeGraph)> {
    core_fixture_shapes()
        .into_iter()
        .chain(characterization_fixture_shapes())
        .map(|(name, files, edges)| (name, graph_from_shapes(&files, &edges)))
        .collect()
}

/// Load-analysis suite shapes: the graph.rs fixtures, the chain/seed/
/// tiebreak/justification/ei9 shapes, concurrency, and the impact variants.
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

/// Characterization-suite shapes plus the unresolved-edge cases: single-hop,
/// same-file collateral, unregistered edge-only ids, pathless host.
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

/// Mirrors the file-set `compute_closures` scopes its BFS over: registered
/// files plus every edge source plus every RESOLVED destination (unresolved
/// destinations are not files for closure purposes).
fn scoped_files(graph: &IncludeGraph) -> HashSet<FileId> {
    let mut all: HashSet<FileId> = graph.file_paths.keys().copied().collect();
    for e in &graph.edges {
        all.insert(e.source);
        if let Some(dest) = e.destination {
            all.insert(dest);
        }
    }
    all
}

fn assert_closures_match_oracle(graph: &IncludeGraph, case: &str) {
    let expected = graph.compute_closures_bfs_oracle();
    let actual = graph.compute_closures();
    assert_eq!(
        actual.len(),
        expected.len(),
        "{case}: closure table size differs (expected {}, got {})",
        expected.len(),
        actual.len()
    );
    for file_id in expected.keys() {
        let want = &expected[file_id];
        let Some(got) = actual.get(file_id) else {
            panic!("`{case}`: file {file_id:?} missing from compute_closures output");
        };
        assert_eq!(
            got.direct_includes, want.direct_includes,
            "`{case}`: direct_includes mismatch for {file_id:?} (expected {:?}, got {:?})",
            want.direct_includes, got.direct_includes
        );
        assert_eq!(
            got.transitive_includes, want.transitive_includes,
            "`{case}`: transitive_includes mismatch for {file_id:?} (expected {:?}, got {:?})",
            want.transitive_includes, got.transitive_includes
        );
        assert_eq!(
            got.closure_size, want.closure_size,
            "`{case}`: closure_size mismatch for {file_id:?}"
        );
    }
}

#[test]
fn test_compute_closures_matches_bfs_oracle_on_all_existing_fixtures() {
    for (name, graph) in existing_fixture_graphs() {
        assert_closures_match_oracle(&graph, name);
    }
}

#[test]
fn test_closures_module_matches_public_compute_closures() {
    for (name, graph) in existing_fixture_graphs() {
        let from_module =
            crate::closures::compute_closures(&graph.adjacency(), &scoped_files(&graph));
        let from_public = graph.compute_closures();
        assert_eq!(
            from_public, from_module,
            "`{name}`: public compute_closures must equal the closures module entry point"
        );
    }
}

#[test]
fn test_two_cycle_member_closure_contains_sibling_and_excludes_self() {
    // A <-> B: the oracle seeds `visited` with the queried file, so
    // closure(A) contains B but never A itself. Pinned against BOTH the
    // oracle (convention source) and compute_closures (I1).
    let graph = graph_from_shapes(&[1, 2], &[(1, Some(2)), (2, Some(1))]);
    for closures in [
        graph.compute_closures_bfs_oracle(),
        graph.compute_closures(),
    ] {
        let a = &closures[&FileId(1)];
        assert_eq!(a.direct_includes, vec![FileId(2)]);
        assert_eq!(a.transitive_includes, vec![FileId(2)]);
        assert!(!a.transitive_includes.contains(&FileId(1)));
        assert_eq!(a.closure_size, 1);
        assert_eq!(closures[&FileId(2)].transitive_includes, vec![FileId(1)]);
    }
}

#[test]
fn test_three_cycle_member_closure_lists_both_siblings_without_self() {
    let graph = graph_from_shapes(&[1, 2, 3], &[(1, Some(2)), (2, Some(3)), (3, Some(1))]);
    let expected: HashMap<FileId, Vec<FileId>> = HashMap::from([
        (FileId(1), vec![FileId(2), FileId(3)]),
        (FileId(2), vec![FileId(1), FileId(3)]),
        (FileId(3), vec![FileId(1), FileId(2)]),
    ]);
    for closures in [
        graph.compute_closures_bfs_oracle(),
        graph.compute_closures(),
    ] {
        for (file_id, transitive) in &expected {
            assert_eq!(
                closures[file_id].transitive_includes, *transitive,
                "3-cycle closure of {file_id:?}"
            );
        }
    }
    assert_closures_match_oracle(&graph, "3-cycle");
}

#[test]
fn test_self_loop_appears_in_direct_but_never_in_transitive() {
    // The edge model admits resolved self-edges (add_edge does not reject
    // source == destination). The oracle's seed-previsit makes the self-loop
    // show up in direct_includes while the BFS skips the already-visited
    // seed, so transitive excludes it.
    let graph = graph_from_shapes(&[1, 2], &[(1, Some(1)), (1, Some(2))]);
    for closures in [
        graph.compute_closures_bfs_oracle(),
        graph.compute_closures(),
    ] {
        let a = &closures[&FileId(1)];
        assert_eq!(a.direct_includes, vec![FileId(1), FileId(2)]);
        assert_eq!(a.transitive_includes, vec![FileId(2)]);
        assert_eq!(closures[&FileId(2)].transitive_includes, Vec::new());
    }
    assert_closures_match_oracle(&graph, "self-loop");
}

#[test]
fn test_unresolved_edges_never_enter_closures_but_register_sources() {
    // `destination: None` edges are skipped by adjacency(); their resolved
    // -destination siblings are not. A source with ONLY unresolved edges
    // still gets an empty closure entry.
    let graph = graph_from_shapes(
        &[10],
        &[
            (10, None),
            (10, Some(11)),
            (11, None),
            (20, None),
            (20, Some(10)),
        ],
    );
    for closures in [
        graph.compute_closures_bfs_oracle(),
        graph.compute_closures(),
    ] {
        assert_eq!(closures[&FileId(10)].direct_includes, vec![FileId(11)]);
        assert_eq!(closures[&FileId(10)].transitive_includes, vec![FileId(11)]);
        assert_eq!(
            closures[&FileId(20)].transitive_includes,
            vec![FileId(10), FileId(11)]
        );
        assert_eq!(closures[&FileId(11)].transitive_includes, Vec::new());
    }
    // unregistered source with unresolved edge only:
    let lone = graph_from_shapes(&[], &[(70, None)]);
    let closures = lone.compute_closures();
    assert_eq!(closures.len(), 1);
    assert_eq!(closures[&FileId(70)].direct_includes, Vec::new());
    assert_eq!(closures[&FileId(70)].transitive_includes, Vec::new());
    assert_closures_match_oracle(&lone, "lone unresolved source");
    assert_closures_match_oracle(&graph, "unresolved mix");
}

#[test]
fn test_scc_chain_external_reach_matches_oracle() {
    // {1,2} <-> and {3,4} <-> with the bridge 2->3: each member's closure is
    // (its SCC minus itself) plus the full downstream reach.
    let graph = graph_from_shapes(
        &[1, 2, 3, 4, 5],
        &[
            (1, Some(2)),
            (2, Some(1)),
            (2, Some(3)),
            (3, Some(4)),
            (4, Some(3)),
            (4, Some(5)),
        ],
    );
    let expected: HashMap<FileId, Vec<FileId>> = HashMap::from([
        (FileId(1), vec![FileId(2), FileId(3), FileId(4), FileId(5)]),
        (FileId(2), vec![FileId(1), FileId(3), FileId(4), FileId(5)]),
        (FileId(3), vec![FileId(4), FileId(5)]),
        (FileId(4), vec![FileId(3), FileId(5)]),
        (FileId(5), vec![]),
    ]);
    let closures = graph.compute_closures();
    for (file_id, transitive) in &expected {
        assert_eq!(
            closures[file_id].transitive_includes, *transitive,
            "SCC-chain closure of {file_id:?}"
        );
    }
    assert_closures_match_oracle(&graph, "scc-chain");
}

// ---------------------------------------------------------------------------
// Seeded random digraphs with cycles (inline LCG — no RNG dependency).
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
            for i in 0..file_count {
                for j in (i + 1)..file_count {
                    if rng.percent(22) {
                        link(&mut graph, i, j);
                    }
                }
            }
        }
        Topology::BigCycle => {
            for i in 0..file_count {
                link(&mut graph, i, (i + 1) % file_count);
            }
            for _ in 0..file_count {
                let from = rng.below(file_count);
                let to = rng.below(file_count);
                link(&mut graph, from, to);
            }
        }
        Topology::TwoSccsBridge => {
            let first = file_count / 3;
            let second = 2 * file_count / 3;
            cyclic_chain(&mut graph, &mut rng, &files, 0, first);
            cyclic_chain(&mut graph, &mut rng, &files, first, second);
            for k in second..file_count.saturating_sub(1) {
                link(&mut graph, k, k + 1);
            }
            // Strict one-way bridges so the two SCCs stay distinct.
            link(&mut graph, 1, first + 1);
            link(&mut graph, first + 1, second);
        }
        Topology::SelfLoops => {
            for i in 0..file_count {
                link(&mut graph, i, (i + 1) % file_count);
                if i % 3 == 0 {
                    link(&mut graph, i, i);
                }
            }
            for _ in 0..file_count {
                let from = rng.below(file_count);
                let to = rng.below(file_count);
                link(&mut graph, from, to);
            }
        }
        Topology::FullBlob => {
            for i in 0..file_count {
                for j in 0..file_count {
                    if i != j && rng.percent(35) {
                        link(&mut graph, i, j);
                    }
                }
            }
        }
    }
    // Unresolved (destination: None) edges are sprinkled into every random
    // fixture so the rewritten closure code must ignore them identically.
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
    for k in start..end {
        let next = start + (k - start + 1) % (end - start);
        graph.add_edge(edge(files[k], Some(files[next])));
        if rng.percent(30) {
            let skip = start + (k - start + 2) % (end - start);
            graph.add_edge(edge(files[k], Some(files[skip])));
        }
    }
}

#[test]
fn test_random_digraphs_with_cycles_match_bfs_oracle() {
    for &(seed, file_count, topology, name) in RANDOM_FIXTURES.iter() {
        let graph = random_graph(seed, file_count, topology);
        let cyclic = !graph.detect_cycle_groups().is_empty();
        match topology {
            Topology::PureDag => assert!(
                !cyclic,
                "`{name}` (seed {seed}): must be a DAG for the corpus to cover both regimes"
            ),
            _ => assert!(
                cyclic,
                "`{name}` (seed {seed}): must contain cycles for the corpus to be meaningful"
            ),
        }
        assert_closures_match_oracle(&graph, name);
        let from_module =
            crate::closures::compute_closures(&graph.adjacency(), &scoped_files(&graph));
        assert_eq!(
            graph.compute_closures(),
            from_module,
            "`{name}`: module entry must equal public compute_closures"
        );
    }
}
