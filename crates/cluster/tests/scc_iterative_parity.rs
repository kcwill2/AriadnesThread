//! M23.5 T1.1 -- frozen parity oracle for the symbol SCC path (IV-6, IV-7).
//!
//! `recursive_tarjan_oracle` is a verbatim, test-only copy of the recursive
//! `strongconnect` machinery in `crates/cluster/src/scc.rs` as it stood at the
//! Phase 1 kickoff tree (post-merge `f9064f4`). It locks the exact output
//! contract the iterative frame machine must preserve: same seed-skip rule,
//! same SCC emission order, same intra-SCC sort.
//!
//! The oracle stays here; production `scc.rs` must never carry a copy of
//! itself behind `#[cfg(test)]`. The chain stress case additionally proves the
//! de-recursion claim (IV-7): the production path completes a 100 000-node
//! chain on a 64 KiB thread stack (Red today: the recursive path overflows
//! that stack), while the recursive oracle needs a dedicated large-stack
//! thread to finish the same input. The two outputs must be identical.

use ariadne_cluster::scc::{compute_symbol_sccs_from_adj, SymbolScc};
use ariadne_index::types::SymbolId;
use std::cmp::min;
use std::collections::{HashMap, HashSet};

/// Small stack chosen by the M23.5 plan (sec 2, T1.1) for the stress thread:
/// deep recursion cannot survive it, an iterative machine can (IV-7).
const SMALL_STACK_BYTES: usize = 64 * 1024;
/// Stack given to the recursive oracle on the chain stress input. The
/// handoff (m235-t1-handoff.md sec 3, T1.1) suggested 2 MiB; measured on
/// this tree, the oracle still overflows 2 MiB at 100 000 recursion levels
/// (capture: STATUS_STACK_OVERFLOW on the oracle thread), so the baseline
/// gets a reserve sized to complete. Only the SMALL_STACK side proves IV-7;
/// the oracle stack is a comparison baseline with no behavioral contract.
const ORACLE_STACK_BYTES: usize = 64 * 1024 * 1024;
const CHAIN_NODE_COUNT: usize = 100_000;
const SEEDED_GRAPH_COUNT: usize = 200;
/// Documented seed for the 200-digraph parity sweep (plan sec 2, T1.1).
/// Graph `ordinal` uses `SEEDED_SWEEP_BASE_SEED ^ ordinal`, so the sweep is
/// reproducible across processes and machines (LCG, no platform RNG).
const SEEDED_SWEEP_BASE_SEED: u64 = 0x0DA7_ED5E_ED12_3456;

#[test]
fn test_fixtures_parity() {
    for fixture in existing_unit_fixtures() {
        assert_fixture_parity(&fixture);
    }
}

#[test]
fn test_seeded_random_parity() {
    let mut rng = DeterministicLcg::with_seed(SEEDED_SWEEP_BASE_SEED);
    for ordinal in 0..SEEDED_GRAPH_COUNT {
        let graph = seeded_digraph(ordinal, &mut rng);
        assert_digraph_parity(&graph);
    }
}

#[test]
fn test_100k_chain_parity() {
    let production_chain =
        run_chain_on(SMALL_STACK_BYTES, ChainShape::OpenChain, compute_symbol_sccs_from_adj);
    let oracle_chain =
        run_chain_on(ORACLE_STACK_BYTES, ChainShape::OpenChain, recursive_tarjan_oracle);
    assert_eq!(
        production_chain, oracle_chain,
        "open 100k chain: production output diverged from the frozen oracle"
    );
    assert_plain_chain_shape(&production_chain);

    let production_fused =
        run_chain_on(SMALL_STACK_BYTES, ChainShape::FusedCycle, compute_symbol_sccs_from_adj);
    let oracle_fused =
        run_chain_on(ORACLE_STACK_BYTES, ChainShape::FusedCycle, recursive_tarjan_oracle);
    assert_eq!(
        production_fused, oracle_fused,
        "chain + back edge s99999->s0: production output diverged from the frozen oracle"
    );
    assert_single_fused_scc_shape(&production_fused);
}

// ---------------------------------------------------------------------------
// Chain stress scaffolding
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum ChainShape {
    /// s0 -> s1 -> ... -> s99999
    OpenChain,
    /// Open chain plus the back edge s99999 -> s0 (one giant SCC).
    FusedCycle,
}

/// Runs one Tarjan implementation over the chain fixture on a thread with the
/// requested stack size. A stack overflow on such a thread aborts the whole
/// test process, so a `join` error here means an ordinary panic, not stack
/// exhaustion -- the 64 KiB production runs are the IV-7 demonstration.
fn run_chain_on<F>(stack_bytes: usize, shape: ChainShape, tarjan: F) -> Vec<SymbolScc>
where
    F: for<'seed, 'map, 'key> Fn(
        &'seed [SymbolId],
        &'map HashMap<&'key SymbolId, Vec<&'key SymbolId>>,
    ) -> Vec<SymbolScc>
        + Send
        + 'static,
{
    let spawned = std::thread::Builder::new()
        .stack_size(stack_bytes)
        .spawn(move || {
            let symbols: Vec<SymbolId> = (0..CHAIN_NODE_COUNT)
                .map(|position| SymbolId::new(format!("s{}", position)))
                .collect();
            let adj = build_chain_adjacency(&symbols, shape);
            tarjan(&symbols, &adj)
        })
        .expect("chain stress thread spawns with the requested stack size");
    spawned
        .join()
        .expect("chain stress thread completed without panicking")
}

fn build_chain_adjacency<'key>(
    symbols: &'key [SymbolId],
    shape: ChainShape,
) -> HashMap<&'key SymbolId, Vec<&'key SymbolId>> {
    let last = symbols.len() - 1;
    let mut edge_pairs: Vec<(usize, usize)> =
        (0..last).map(|position| (position, position + 1)).collect();
    if matches!(shape, ChainShape::FusedCycle) {
        edge_pairs.push((last, 0));
    }
    build_adjacency_from_edge_pairs(&edge_pairs, symbols)
}

fn assert_plain_chain_shape(sccs: &[SymbolScc]) {
    assert_eq!(
        sccs.len(),
        CHAIN_NODE_COUNT,
        "an open chain has exactly one SCC per node"
    );
    assert!(
        sccs.iter().all(|scc| scc.symbols.len() == 1),
        "every SCC of an open chain is a singleton"
    );
    // Emission is reverse-topological: the tail is emitted first.
    assert_eq!(
        sccs[0].symbols[0],
        SymbolId::new(format!("s{}", CHAIN_NODE_COUNT - 1)),
        "open chain SCCs must start from the tail s99999"
    );
    assert_eq!(
        sccs[CHAIN_NODE_COUNT - 1].symbols[0],
        SymbolId::new("s0"),
        "open chain SCCs must end at the seed s0"
    );
}

fn assert_single_fused_scc_shape(sccs: &[SymbolScc]) {
    assert_eq!(
        sccs.len(),
        1,
        "the back edge s99999->s0 must fuse the whole chain into one SCC"
    );
    let fused = &sccs[0];
    assert_eq!(fused.symbols.len(), CHAIN_NODE_COUNT);
    // Intra-SCC sort is lexicographic on the symbol string.
    assert_eq!(
        fused.symbols.first().expect("the fused SCC is non-empty"),
        &SymbolId::new("s0"),
        "lexicographically first member of the fused SCC must be s0"
    );
    assert_eq!(
        fused.symbols.last().expect("the fused SCC is non-empty"),
        &SymbolId::new("s99999"),
        "lexicographically last member of the fused SCC must be s99999"
    );
}

// ---------------------------------------------------------------------------
// Fixtures mirroring every unit fixture in crates/cluster/src/scc.rs
// ---------------------------------------------------------------------------

struct SymbolFixture {
    label: &'static str,
    symbol_names: Vec<&'static str>,
    edge_pairs: Vec<(usize, usize)>,
}

fn existing_unit_fixtures() -> Vec<SymbolFixture> {
    let [symbol_a, symbol_b, symbol_c, symbol_d, symbol_e] = ["A", "B", "C", "D", "E"];
    vec![
        SymbolFixture {
            label: "acyclic A->B->C (test_acyclic_sccs)",
            symbol_names: vec![symbol_a, symbol_b, symbol_c],
            edge_pairs: vec![(0, 1), (1, 2)],
        },
        SymbolFixture {
            label: "cycle A->B->C->A with tail D->C (test_cyclic_sccs)",
            symbol_names: vec![symbol_a, symbol_b, symbol_c, symbol_d],
            edge_pairs: vec![(0, 1), (1, 2), (2, 0), (3, 2)],
        },
        SymbolFixture {
            label: "empty input (test_empty_input)",
            symbol_names: vec![],
            edge_pairs: vec![],
        },
        SymbolFixture {
            label: "disconnected symbols (test_disconnected_components)",
            symbol_names: vec![symbol_a, symbol_b],
            edge_pairs: vec![],
        },
        SymbolFixture {
            label: "self loop A->A (test_self_loop)",
            symbol_names: vec![symbol_a],
            edge_pairs: vec![(0, 0)],
        },
        SymbolFixture {
            label: "two disjoint SCCs (test_multiple_disjoint_sccs)",
            symbol_names: vec![symbol_a, symbol_b, symbol_c, symbol_d, symbol_e],
            edge_pairs: vec![(0, 1), (1, 0), (2, 3), (3, 4), (4, 2)],
        },
        SymbolFixture {
            label: "shared fixture with E->A (from_adj_matches_wrapper_on_shared_fixture)",
            symbol_names: vec![symbol_a, symbol_b, symbol_c, symbol_d, symbol_e],
            edge_pairs: vec![(0, 1), (1, 0), (1, 2), (2, 3), (3, 4), (4, 2), (4, 0)],
        },
        SymbolFixture {
            label: "shared fixture minus E->A (from_adj_resolves_two_disjoint_sccs)",
            symbol_names: vec![symbol_a, symbol_b, symbol_c, symbol_d, symbol_e],
            edge_pairs: vec![(0, 1), (1, 0), (1, 2), (2, 3), (3, 4), (4, 2)],
        },
        SymbolFixture {
            label: "connected two-SCC graph (test_connected_multi_scc_graph)",
            symbol_names: vec![symbol_a, symbol_b, symbol_c, symbol_d],
            edge_pairs: vec![(0, 1), (1, 0), (1, 2), (2, 3), (3, 2)],
        },
    ]
}

fn assert_fixture_parity(fixture: &SymbolFixture) {
    let symbols: Vec<SymbolId> = fixture
        .symbol_names
        .iter()
        .map(|name| SymbolId::new(*name))
        .collect();
    let adj = build_adjacency_from_edge_pairs(&fixture.edge_pairs, &symbols);
    assert_eq!(
        compute_symbol_sccs_from_adj(&symbols, &adj),
        recursive_tarjan_oracle(&symbols, &adj),
        "production output diverged from the frozen oracle on fixture '{}'",
        fixture.label
    );
}

// ---------------------------------------------------------------------------
// Seeded random digraph sweep
// ---------------------------------------------------------------------------

struct DeterministicLcg {
    state: u64,
}

impl DeterministicLcg {
    fn with_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Knuth MMIX LCG constants -- identical on every platform.
    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % (bound as u64)) as usize
    }
}

/// One reproducible random digraph over symbols n0..n{symbol_count-1}.
/// Edge order is generation order, so neighbor order is part of the fixture.
struct RandomDigraph {
    ordinal: usize,
    symbol_count: usize,
    edge_pairs: Vec<(usize, usize)>,
}

fn seeded_digraph(ordinal: usize, rng: &mut DeterministicLcg) -> RandomDigraph {
    // Re-seed per graph so one graph's shape cannot shift the next one's.
    rng.state = SEEDED_SWEEP_BASE_SEED ^ (ordinal as u64).wrapping_mul(6364136223846793005);
    let symbol_count = 2 + rng.below(30);
    let edge_count = rng.below(2 * symbol_count + 1);
    let edge_pairs: Vec<(usize, usize)> = (0..edge_count)
        .map(|_| (rng.below(symbol_count), rng.below(symbol_count)))
        .collect();
    RandomDigraph {
        ordinal,
        symbol_count,
        edge_pairs,
    }
}

fn assert_digraph_parity(graph: &RandomDigraph) {
    let symbols: Vec<SymbolId> = (0..graph.symbol_count)
        .map(|position| SymbolId::new(format!("n{}", position)))
        .collect();
    let adj = build_adjacency_from_edge_pairs(&graph.edge_pairs, &symbols);
    assert_eq!(
        compute_symbol_sccs_from_adj(&symbols, &adj),
        recursive_tarjan_oracle(&symbols, &adj),
        "seeded digraph ordinal {} ({} symbols, {} edges) diverged from the oracle",
        graph.ordinal,
        graph.symbol_count,
        graph.edge_pairs.len()
    );
}

/// Adjacency built in edge-pair order exactly as `build_symbol_adjacency`
/// does for edge slices: one entry pushed per edge, duplicates preserved.
fn build_adjacency_from_edge_pairs<'key>(
    edge_pairs: &[(usize, usize)],
    symbols: &'key [SymbolId],
) -> HashMap<&'key SymbolId, Vec<&'key SymbolId>> {
    let mut adj: HashMap<&'key SymbolId, Vec<&'key SymbolId>> = HashMap::new();
    for (src, dst) in edge_pairs {
        adj.entry(&symbols[*src]).or_default().push(&symbols[*dst]);
    }
    adj
}

// ---------------------------------------------------------------------------
// Frozen recursive oracle: verbatim copy of the pre-T1.2 production code
// (compute_symbol_sccs_from_adj + strongconnect at the kickoff tree).
// ---------------------------------------------------------------------------

fn recursive_tarjan_oracle<'a>(
    symbols: &[SymbolId],
    adj: &HashMap<&'a SymbolId, Vec<&'a SymbolId>>,
) -> Vec<SymbolScc> {
    let mut index = 0;
    let mut indices: HashMap<&SymbolId, usize> = HashMap::new();
    let mut lowlink: HashMap<&SymbolId, usize> = HashMap::new();
    let mut on_stack: HashSet<&SymbolId> = HashSet::new();
    let mut stack: Vec<&SymbolId> = Vec::new();
    let mut sccs: Vec<SymbolScc> = Vec::new();

    for sym in symbols {
        if !indices.contains_key(sym) {
            oracle_strongconnect(
                sym,
                adj,
                &mut index,
                &mut indices,
                &mut lowlink,
                &mut on_stack,
                &mut stack,
                &mut sccs,
            );
        }
    }

    sccs
}

#[allow(clippy::too_many_arguments)]
fn oracle_strongconnect<'a>(
    v: &'a SymbolId,
    adj: &HashMap<&SymbolId, Vec<&'a SymbolId>>,
    index: &mut usize,
    indices: &mut HashMap<&'a SymbolId, usize>,
    lowlink: &mut HashMap<&'a SymbolId, usize>,
    on_stack: &mut HashSet<&'a SymbolId>,
    stack: &mut Vec<&'a SymbolId>,
    sccs: &mut Vec<SymbolScc>,
) {
    indices.insert(v, *index);
    lowlink.insert(v, *index);
    *index += 1;
    stack.push(v);
    on_stack.insert(v);

    if let Some(neighbors) = adj.get(v) {
        for w in neighbors {
            if !indices.contains_key(w) {
                oracle_strongconnect(
                    w, adj, index, indices, lowlink, on_stack, stack, sccs,
                );
                let new_lowlink = min(
                    *lowlink
                        .get(v)
                        .expect("strongconnect inserts v's lowlink on entry"),
                    *lowlink
                        .get(w)
                        .expect("recursed w has its lowlink set by its own strongconnect"),
                );
                lowlink.insert(v, new_lowlink);
            } else if on_stack.contains(w) {
                let new_lowlink = min(
                    *lowlink
                        .get(v)
                        .expect("strongconnect inserts v's lowlink on entry"),
                    *indices
                        .get(w)
                        .expect("on_stack membership implies indices contains w"),
                );
                lowlink.insert(v, new_lowlink);
            }
        }
    }

    if lowlink.get(v) == indices.get(v) {
        let mut scc = Vec::new();
        loop {
            let w = stack
                .pop()
                .expect("v is on the stack until popped here, so the stack is non-empty");
            on_stack.remove(w);
            scc.push(w.clone());
            if w == v {
                break;
            }
        }
        // To make deterministic, sort inside SCC if needed, but not strictly required
        // unless test demands it. We just leave it as is, or we could sort by symbol name.
        scc.sort_by(|a, b| a.0.cmp(&b.0));
        sccs.push(SymbolScc { symbols: scc });
    }
}
