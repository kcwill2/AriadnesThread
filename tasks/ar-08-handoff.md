# AR-08 Handoff - Stage-5 by-id symbol metrics (P1 HIGH, part 2)

Wave 2 / Algorithmic Perf. Bench-gated. Independent crate (`pipeline`). Depends: AR-07.

## Objective

`stage_load.rs:347` collects `(symbol_id, name)` once (or resolves via the AR-07 map)
and calls the by-id API instead of string-comparing.

## Impact

- Kills the O(S^2) string-compare term in the stage-5 tail -- the exact segment
  `bench_compare.py` measures.
- Expected measurable stage-5 median drop (declare observed ratio in journal; REJECT if none).

## Touch-set (hard)

- `crates/pipeline/src/stage_load.rs` (around line 347)
- `crates/pipeline/tests/**`
- `tasks/ar-08-handoff-journal.md` — checkpoint journal

## Constraints

- TDD: red-first. Add test for by-id symbol metrics before refactoring.
- Pure ASCII only.
- Commit via `python tools/commit_scoped.py --scope <glob> -m "<msg>"`.
- Gates via distilling entry points only.
- `bench_compare` gate required.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-pipeline` (scoped pipeline tests)
2. `python tools/bench_compare.py` (must show stage-5 improvement or neutral)
3. `python tools/encoding_guard.py --staged`
4. `python tools/hygiene_check.py`

## Invariants / locks

- Output byte-identical (same metrics computed).
- `bench_compare` must show no regression (1.25x threshold).
- Depends on AR-07 landing first.
