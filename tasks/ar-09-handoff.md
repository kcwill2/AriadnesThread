# AR-09 Handoff - Adjacency-driven scans (P3 HIGH + P4 MED-HIGH)

Wave 2 / Algorithmic Perf. Bench-gated. Crate (`graph`). Depends: AR-06 landed
(same-file region serialization).

## Objective

Back `direct_includes` (graph.rs) and the BFS in `traversal.rs`
(`traverse_neighborhood`, `shortest_include_path`) and `callgraph.rs` with the
existing memoized adjacency (`LoadMemo::file_adjacency_map`, `load_caches.rs`)
or a lazily built adjacency index on the graph. Keep result order identical.

## Impact

- `/api/graph/traversal`, `/api/graph/traverse`, `/api/graph/path` click cost
  O(k*E)/O(V_visited*E) -> O(k*deg). On a 1k-node/20k-edge graph a typical
  neighborhood click drops ~10^7 comparisons to ~10^4.
- `frontier.rs` and `load_explain.rs` callers inherit the win.

## Touch-set (hard)

- `crates/graph/src/graph.rs`
- `crates/graph/src/traversal.rs`
- `crates/graph/src/callgraph.rs`
- `crates/graph/src/load_caches.rs` (only if reusing its memo)
- `crates/graph/tests/**` — equivalence locks vs current scans
- `tasks/ar-09-handoff-journal.md`

## Constraints

- TDD: red-first equivalence locks BEFORE changing scans.
- Result order byte-identical (equivalence tests vs current scans on fixtures).
- Pure ASCII only.
- Commit via `python tools/commit_scoped.py`; gates via distilling entry points.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-graph`
2. `python tools/bench_compare.py` (no regression past 1.25x)
3. `python tools/encoding_guard.py --all`
4. `python tools/hygiene_check.py` (zero NEW violations)

## Invariants / locks

- Equivalence locks: adjacency-driven results identical (nodes sorted, edges
  sorted) to the current edge-list scans on all existing fixtures + new ones.
- m00 parity and goldens UNEDITED.
