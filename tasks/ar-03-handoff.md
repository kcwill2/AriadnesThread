# AR-03 Handoff - Bounded bincode decode (S5 MED)

Wave 0 / Security Hotfix. Independent crate — can run in parallel with AR-01.

## Objective

Add a decode-options helper (`bincode 1.3 options().with_limit(...)`) sized from
observed payloads + margin, used by all `bincode::deserialize` sites in
`report/src/manager/*.rs`. Add a regression fixture: blob declaring huge collection
length fails fast with an error, not allocation abort.

## Impact

- A corrupt or attacker-authored `artifacts/` blob can no longer abort the process via
  declared-length allocation.
- Error surfaces as a normal cache-miss/error instead of process abort.
- Honest reads unchanged.

## Touch-set (hard)

- `crates/report/src/manager/bincode_options.rs` (NEW) — helper function
- `crates/report/src/manager/*` — replace `bincode::deserialize` with bounded helper
- `crates/report/tests/**` — regression fixture test
- `crates/report/src/manager/tests.rs` — if applicable
- `tasks/ar-03-handoff-journal.md` — checkpoint journal

## Constraints

- TDD: red-first. Add failing test for huge-length blob before implementing fix.
- Pure ASCII only.
- Measure actual payload sizes and record in journal.
- Commit via `python tools/commit_scoped.py --scope <glob> -m "<msg>"`.
- Gates via distilling entry points only.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-report` (scoped report tests)
2. `python tools/encoding_guard.py --staged`
3. `python tools/hygiene_check.py`

## Invariants / locks

- All `bincode::deserialize` sites in `report/src/manager/` must use the bounded helper.
- A blob declaring a huge collection length must fail fast, not abort.
- Honest reads byte-identical.
- The limit value should be sized from observed payload sizes + margin.
