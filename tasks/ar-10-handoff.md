# AR-10 Handoff - Search path: hoist regex + single-pass lowercase (P7 MED)

Wave 2 / Perf. Crate (`ui`). Depends: after AR-01 AND AR-02 (server.rs
serialization chain).

## Objective

In `api_graph_search` (`crates/ui/src/server.rs`, region around lines 630-645
at AR-01 base): compile the wildcard `Regex` ONCE per request (case-insensitive
regex replaces both `to_lowercase()` allocs per element).

## Impact

- Per-search over 10k elements stops recompiling up to 10k regexes and doing
  2*10k allocations per keystroke. Removes UI typing lag on the filter box.

## Touch-set (hard)

- `crates/ui/src/server.rs` (search handler region only)
- `crates/ui/tests/ui_test/server_api.rs` OR a new test mod (server_api.rs is
  at ~1000 lines — do NOT push it over; use a new mod if needed)
- `tasks/ar-10-handoff-journal.md`

## Constraints

- TDD: red-first — lock search semantics (wildcard `*`/`?`, `/regex/` literals,
  case-insensitive contains) with characterization tests BEFORE the rewrite.
- Search result ordering and match semantics byte-identical.
- Pure ASCII only.
- Commit via `python tools/commit_scoped.py`; gates via distilling entry points.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-ui`
2. `python tools/distill.py -- python tools/ui_smoke.py`
3. `python tools/encoding_guard.py --all`
4. `python tools/hygiene_check.py` (zero NEW violations; server.rs R1/R2 are
   AR-13-owned pre-existing)

## Invariants / locks

- Characterization locks: wildcard/regex/substring cases produce identical
  results pre/post change.
- ~20-40 line change; anything larger is scope creep.
