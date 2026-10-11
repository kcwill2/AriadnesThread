# AR-25 Handoff - CLI consumes typed seam errors (A7 riders)

Wave 6. Crate (`cli` + small `report`). Dispatch after AR-24 APPROVED AND
AR-19/AR-23 landed (registry.rs / analyze.rs cli serialization — line cites
below are pre-shift; RE-ANCHOR).

## Objective

Drop the `.map_err(anyhow::Error::msg)` re-wraps at
`crates/cli/src/registry.rs` (~:105) and `crates/cli/src/commands/analyze.rs`
(~:49) so AR-24's typed errors flow as typed anyhow sources; fold the
bespoke `StaleArtifact` reporting onto the shared error pattern where
trivial (where non-trivial, journal why and leave it).

## Impact

Removes the double-wrapping layer: error chains stay intact end-to-end
(context/source() navigable for debugging). No user-visible string change.

## Touch-set (hard)

- `crates/cli/src/registry.rs`
- `crates/cli/src/commands/analyze.rs` (re-wrap sites only)
- `crates/report/src/manager.rs` (StaleArtifact fold ONLY if trivial)
- cli/report lock tests (new chain-navigation lock)
- `tasks/ar-25-handoff-journal.md`

## Constraints

- CLI-visible error STRINGS byte-identical (AR-24's display lock stays the
  authority; m00/m18 error-text locks UNEDITED).
- New lock: `anyhow::Error::downcast_ref::<TypedSeamError>` (or
  `source()` chain walk) succeeds on a forced error where today the
  re-wrap made it impossible — proves the layers actually dropped
  (RED before fix, GREEN after).
- STRICT COMPILE-GREEN; pure ASCII; commit_scoped cli (+report) + journal.

## Gate commands

1. `-p ariadne-cli` (baseline 118/0/8/6 + AR-19/AR-23 deltas reconciled)
2. `-p ariadne-report` (47/0/4i/suites reconciled — AR-03 baseline)
3. m00 golden + facts-path UNEDITED
4. `encoding_guard --all`; `hygiene_check.py` (current zero-blocking or
   2-row baseline — quote state)

## Invariants / locks

- Every error printed to stderr during forced-failure runs matches
  pre-change capture (journal: same forced scenario before/after).
- No new `.map_err(anyhow::Error::msg)` on seam paths (grep lock).
