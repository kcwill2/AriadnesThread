# AR-17 Handoff - Integration fixture paths + polling + m21 deterministic lock (T4/T5/T6 MED)

Wave 5 / Test hardening. Directory (`tests/integration`). Parallel-safe with
disjoint crates — but NOT with AR-02 (both can touch m16/m17 call sites);
dispatch only after AR-02 lands.

## Objective

1. Replace `PathBuf::from("../../tests/fixtures/basic")` (and sibling relative
   forms) with `common::fixture_path` in m10/m15/m16/m17 (and m18 where the
   pattern appears).
2. Replace the three fixed 250 ms sleeps with the existing poll-until-ready
   idiom (pattern precedent in m21:129-157 — re-anchor at start).
3. Make the m21 serve-while-loading/gating lock deterministic: force loading
   state via the boot seam OR count-assert the probe fired — the lock must not
   be able to silently stop asserting.

## Impact

- Suites stop depending on process CWD (CI dir-independence).
- Fixed-sleep flake class removed on loaded machines.
- m21 gating guarantee gets a lock that cannot silently become vacuous.

## Touch-set (hard)

- `tests/integration/tests/m10*.rs`, `m15*.rs`, `m16*.rs`, `m17*.rs`,
  `m18*.rs`, `m21*.rs`
- `tests/integration/tests/common/mod.rs` (fixture_path helper if missing)
- `tasks/ar-17-handoff-journal.md`

## Constraints

- Behavior-neutral: assertion count and meaning preserved; deterministic-by-
  construction only.
- Pure ASCII; commit_scoped; distilling gates.

## Gate commands

1. `python tools/testrun.py --distill -- <m10..m21 scoped integration forms>`
   (each touched suite binary once; quote canonical lines)
2. `python tools/encoding_guard.py --all`
3. `python tools/hygiene_check.py` (zero NEW)

## Invariants / locks

- No fixture file moves — helper resolves existing paths only.
- m21 lock post-change: journal must show the negative case (lock fails when
  gating is removed — mutation-proof) or the count-assert equivalent.
