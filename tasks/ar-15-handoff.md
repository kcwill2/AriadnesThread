# AR-15 Handoff - Payload Arc clone + blocking-handler offload (P5 + P6 MED)

Wave 4. Crate (`ui`). Dispatch ONLY after AR-13 APPROVED (paths in this
contract refer to the POST-SPLIT layout — re-anchor routes/state module names
from AR-13's landing).

## Objective

1. Keep the export payload pre-serialized as `Arc<Bytes>` on rebuild; serve
   clones of the Arc (no per-request deep clone) with `ETag` + `304` support
   on the 2 s poll route.
2. Move BFS/memo handlers to `spawn_blocking` (locate existing precedent via
   grep `spawn_blocking` in the ui/cli launch path — plan cites
   `ui.rs:567,:798`, re-anchor).
3. `crates/ui/assets/api.js` poll fetch (~:121-142) becomes ETag/delta-aware.

## Impact

Removes a full deep-clone of the graph payload per request (2x payload peak
-> ~zero); unblocks tokio workers so rebuild no longer queues behind request
serialization; poll ticks become near-free when nothing changed.

## Touch-set (hard)

- `crates/ui/src/**` (post-split routes/state only)
- `crates/ui/assets/api.js`
- ui locks (new ETag/304 + no-deep-clone locks; existing locks UNEDITED)
- `tasks/ar-15-handoff-journal.md`

## Constraints

- Payload BYTES identical to today's serialization (ETag computed over the
  same bytes the deep-clone path produced — characterization lock first).
- api.js edits must preserve AR-11's escaping discipline (escaper calls stay
  at every interpolation; hardening locks UNEDITED green).
- 304 only when `If-None-Match` matches exactly; first-request path
  byte-identical.
- STRICT COMPILE-GREEN; pure ASCII; commit_scoped ui + journal.

## Gate commands

1. `-p ariadne-ui` (post-AR-13/AR-11 count + new locks, reconcile)
2. `python tools/distill.py -- python tools/ui_smoke.py` (PASS)
3. `python tools/distill.py -- python tools/bench_compare.py` (stage rows
   neutral-or-better; poll-route improvement journaled)
4. m15/m16 traversal locks UNEDITED
5. `python tools/theme_token_check.py --check-added --base <rev>` (api.js/api
   css-adjacent rows) — silent
6. `encoding_guard --all`; `hygiene_check.py` (post-AR-13 zero-blocking
   baseline)

## Invariants / locks

- ETag on unchanged payload -> 304 (new lock); changed payload -> fresh 200
  (mutation: bump a byte -> lock goes non-vacuous).
- spawn_blocking move keeps handler output byte-identical.
