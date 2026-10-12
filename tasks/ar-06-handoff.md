# AR-06 Handoff - O(1) edge dedup in `add_edge` (P2 HIGH)

Wave 2 / Algorithmic Perf. Bench-gated. Independent crate (`graph`).

## Objective

Maintain a `HashSet` of edge keys alongside the edge Vec in `FileGraph`.
`builder.rs:129` caller unchanged. This changes `add_edge` from O(E) scan to O(1)
lookup, making graph build go O(E^2) -> O(E).

## Impact

- At 50k edges: ~2.5e9 comparisons removed -- the dominant builder cost.
- Output edge order locked byte-identical (Vec order preserved).

## Touch-set (hard)

- `crates/graph/src/graph.rs` — `add_edge` method (line 42-46), `IncludeGraph` struct
- `crates/graph/src/builder.rs` (if ctor plumbing needs touching)
- `crates/graph/tests/**` — order-parity test
- `tasks/ar-06-handoff-journal.md` — checkpoint journal

## Constraints

- TDD: red-first. Add test for duplicate rejection before implementing.
- Pure ASCII only.
- Commit via `python tools/commit_scoped.py --scope <glob> -m "<msg>"`.
- Gates via distilling entry points only.
- `bench_compare` gate required.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-graph` (scoped graph tests)
2. `python tools/bench_compare.py` (must show no regression)
3. `python tools/encoding_guard.py --staged`
4. `python tools/hygiene_check.py`

## Invariants / locks

- `add_edge` must reject duplicates via O(1) lookup.
- Edge Vec order preserved (byte-identical output).
- Existing `compute_closures`, `direct_includes`, etc. behavior unchanged.
