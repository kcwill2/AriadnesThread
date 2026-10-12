# AR-07 Handoff - Symbol name->id side map + by-id API (P1 HIGH, part 1)

Wave 2 / Algorithmic Perf. Independent crate (`graph`). Depends: none (additive).

## Objective

Add `symbol_id_by_name` map (build alongside symbol insertion) +
`analyze_symbol_load_by_id` on the load API, mirroring the existing
`analyze_file_load_by_id` precedent. Purely additive; old fn kept.

## Impact

- No runtime change yet -- creates the seam for AR-08.
- Public API additive only.

## Touch-set (hard)

- `crates/graph/src/load.rs` (+ inline test mod)
- `crates/graph/tests/**` (if applicable)
- `tasks/ar-07-handoff-journal.md` — checkpoint journal

## Constraints

- TDD: red-first. Add test for by-id API before implementing.
- Pure ASCII only.
- Commit via `python tools/commit_scoped.py --scope <glob> -m "<msg>"`.
- Gates via distilling entry points only.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-graph` (scoped graph tests)
2. `python tools/encoding_guard.py --staged`
3. `python tools/hygiene_check.py`

## Invariants / locks

- New API `analyze_symbol_load_by_id` returns same data shape as `analyze_file_load_by_id`.
- `symbol_id_by_name` map built alongside symbol insertion.
- Existing API unchanged.
