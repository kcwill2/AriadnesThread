# AR-13 Handoff - server.rs split + api_graph_traversal decomposition (Q1/Q2/Q3)

Wave 4. Crate (`ui`). LAST in the server.rs serial chain (AR-01 `6d7f96b` ->
AR-02 `b49ed06` -> AR-10 `3a26a9a` all landed). Dispatch only after AR-11
APPROVED (UI hands never parallel — AR-11 owns assets + ui_test registration
until then).

## Objective

Split `crates/ui/src/server.rs` (currently 1172 lines — re-anchor) into
route-family modules: `routes/api_source.rs`, `routes/api_graph.rs`,
`routes/api_payloads.rs`, `state.rs`, plus a `server.rs` shell re-exporting
the SAME pub surface. Decompose `api_graph_traversal` (~161 lines, the R1
row) into seed-resolution / closure-computation / projection functions.
VERBATIM MOVES + declared amendments only; route table byte-identical.

## Impact

Clears BOTH live hygiene violations — hygiene_check exits 0 for the first
time (the standing "2 server.rs rows" baseline DIES here; journal declares
the new zero-blocking-rows baseline for downstream contracts). Future
containment/security edits land in small reviewable files.

## Touch-set (hard)

- `crates/ui/src/**` (+ `lib.rs` mod lines)
- test files ONLY for declared path amendments (tests referencing
  `crate::server::` internals); zero assertion edits
- `tasks/ar-13-handoff-journal.md`

## Constraints

- SIZING CEILING: ~1000-1200 moved lines is the max. If you sense drift,
  STOP and report for re-split as T-a (state + source routes) / T-b (graph
  routes) rather than shipping a drift-prone mega-move.
- ui_smoke asserts routes — they must be byte-identical.
- STRICT COMPILE-GREEN (shared worktree; AR-19 cli lane may be active).
- Pure ASCII; commit_scoped ui + journal; single move commit where feasible
  so `git show` diff-move detection works.

## Gate commands

1. `-p ariadne-ui` (expected 256 + AR-11's asset_injection_hardening count —
   reconcile against AR-11's landing count, UNEDITED tests except path fixes)
2. `python tools/distill.py -- python tools/ui_smoke.py` (PASS)
3. `python tools/hygiene_check.py` — NEW BASELINE: exit 0, ZERO blocking rows
   (quote verbatim; this is the acceptance criterion)
4. `python tools/encoding_guard.py --all`

## Invariants / locks

- Pub surface of the ui crate unchanged (grep pub before/after).
- Route table identical (ui_smoke authority).
- api_graph_traversal decomposition preserves output exactly: existing
  traversal characterization locks (m15/m16) UNEDITED green.
