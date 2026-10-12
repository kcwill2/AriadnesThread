# AR-04 Handoff - Rollback on mid-apply failure (T1 HIGH)

Wave 1 / Correctness. Independent crate (`validate`).

## Objective

`apply_with` captures pre-mutation copies per file and restores all written targets before
propagating any internal error. Mirror the existing tier-failure rollback at
`lib.rs:792-834`. Add a failing-writer oracle test that fails on the second write and
asserts byte-identical tree afterwards.

## Impact

- The advertised transactional guarantee becomes true for IO/hook failures too.
- No partial-repo mutation can escape an `apply` that returned Err (locked by regression test).
- Error text gains a "rolled back N files" line; success path byte-identical.

## Touch-set (hard)

- `crates/validate/src/lib.rs` — modify `apply_with` to capture + restore
- `crates/validate/tests/**` — failing-writer oracle test
- `tasks/ar-04-handoff-journal.md` — checkpoint journal

## Constraints

- TDD: red-first. Add failing test for mid-apply rollback before implementing fix.
- Pure ASCII only.
- Commit via `python tools/commit_scoped.py --scope <glob> -m "<msg>"`.
- Gates via distilling entry points only.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-validate` (scoped validate tests)
2. `python tools/encoding_guard.py --staged`
3. `python tools/hygiene_check.py`

## Invariants / locks

- `apply_with` must capture pre-mutation copies of all touched files BEFORE writing.
- On ANY error during write/extract/rewriter phases, ALL previously-written files restored.
- Success path byte-identical to current behavior.
- Regression test: failing writer oracle — simulate write failure on second file, assert tree restored.
