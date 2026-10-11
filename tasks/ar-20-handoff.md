> SUPERSEDED 2026-10-11: replaced by the C-lite ruling in `tasks/ar-20-shape.md` and implemented as `tasks/ar-20a-handoff.md` (additive move + shims) -> `tasks/ar-20b-handoff.md` (switch + guard). The lang-api relocation below is mechanically infeasible (journal blocker, `tasks/ar-20-handoff-journal.md`); retained as the blocked-run's record - do not dispatch.

# AR-20 Handoff - WorkerBackend/TuTask move to lang-api (A4 MED)

Wave 6. Crates (`lang-api` + `lang-cpp` + `scheduler`) — indivisible seam
move, per-crate scoped gates (AR-18 multi-crate precedent). Dispatch after
AR-19 APPROVED (lang-cpp serialization; AR-19 owns plan.rs + Cargo churn
until landed).

## Objective

Relocate the worker-api types/traits (`WorkerBackend`, `TuTask` — locate in
`crates/scheduler` today) into `lang-api` (new `worker-api` module or types
module — journal the placement choice). Repoint `crates/lang-cpp/src/backend.rs`
imports; REMOVE the `lang-cpp -> scheduler` Cargo dep; `scheduler` re-exports
the moved types (callers keep compiling). Make the m17 guard honest: ADD a
transitive-dependency assert to the existing guard test so plugin-crate
isolation is machine-checked, not aspirational.

## Impact

Plugin crates stop transitively compiling against
report/cluster/transform/validate; the isolation the m17 guard *claims*
becomes real. Plugin builds get faster.

## Touch-set (hard)

- `crates/lang-api/src/**` (new module + re-exports)
- `crates/lang-cpp/src/backend.rs`, `crates/lang-cpp/Cargo.toml`, `lib.rs` imports
- `crates/scheduler/**` (re-export shim only — call sites UNTOUCHED)
- the existing m17 dependency-guard test (tests/integration/tests/ — locate
  via grep for the existing guard; journal its path)
- `tasks/ar-20-handoff-journal.md`

## Constraints

- TDD: guard test extended FIRST — RED while lang-cpp still depends on
  scheduler (assert no scheduler in lang-cpp's transitive tree), GREEN after
  dep removal.
- Scheduler re-export keeps every existing `scheduler::` worker-type path
  compiling — zero scheduler call-site edits.
- Type definitions MOVED VERBATIM (incl. serde derives — wire format frozen;
  m00 parity + bincode locks are authority).
- STRICT COMPILE-GREEN: land import repoint and dep removal in ONE commit;
  cargo check --workspace before tests.
- Pure ASCII; commit_scoped lang-api + lang-cpp + scheduler + integration +
  journal (AR-18 single cross-crate commit precedent — one commit, no lane
  sweep).

## Gate commands

1. `-p ariadne-lang-api` (baseline + moved-type doc tests if any)
2. `-p ariadne-lang-cpp` (153/0/0/2 exact)
3. `-p ariadne-scheduler` (34/0/0/2 exact)
4. m17 guard suite (now +1 transitive assert) + m00 golden + facts-path UNEDITED
5. `python tools/testrun.py --distill -- cargo check -p ariadne-lang-cpp --all-targets` shape
   check OR cargo check --workspace ok (0 warnings)
6. `encoding_guard --all`; `hygiene_check.py` (post-AR-13: zero blocking rows;
   pre-AR-13: only 2 server.rs rows — quote either state)

## Invariants / locks

- serde wire format byte-identical (moved types, zero attribute changes).
- `cargo tree -p ariadne-lang-cpp` shows NO scheduler/report/cluster
  (the new guard assert locks this).
