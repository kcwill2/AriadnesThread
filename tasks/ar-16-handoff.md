# AR-16 Handoff - Worker dispatch tests (T3 MED)

Wave 5 / Test Hardening. Independent crate (`scheduler`).

## Objective

Unit tests in `worker.rs` for:
1. Missing-language dispatch (the M24 third-language error contract)
2. Unknown-task path
3. Cancel path

## Impact

- The cross-language error contract stops being locked only indirectly by C++/Scala parity.
- A refactor of dispatch that drops the error path fails fast in-crate.

## Touch-set (hard)

- `crates/scheduler/src/worker.rs` (test module only)
- `tasks/ar-16-handoff-journal.md` — checkpoint journal

## Constraints

- TDD: tests written and failing first, then verify they pass.
- Pure ASCII only.
- Commit via `python tools/commit_scoped.py --scope <glob> -m "<msg>"`.
- Gates via distilling entry points only.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-scheduler`
2. `python tools/encoding_guard.py --staged`
3. `python tools/hygiene_check.py`

## Invariants / locks

- Tests are additive (test mod only, no production code changes).
- Must cover missing-language dispatch, unknown-task, and cancel paths.
