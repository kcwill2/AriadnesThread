# AR-07 Journal - Symbol name->id side map + by-id API (P1 HIGH, part 1)

## What was done

- `crates/graph/src/load.rs` (only implementation file touched):
  - New public `symbol_id_by_name(&self, &str) -> Option<SymbolId>`: the
    name->id resolution helper, carrying the original scan body verbatim
    (matches map-key string `id.0` OR record `name`, first key-ascending
    match wins).
  - New public `analyze_symbol_load_by_id(&self, &SymbolId) -> Result<LoadMetric>`
    mirroring the `analyze_file_load_by_id` precedent: takes the metric body
    (record lookup, defining-file lookup, memoized single-seed closure,
    `compute_load`) so callers that already hold ids skip the O(S) scan.
  - `analyze_symbol_load` (name entry) now resolves via the helper and
    delegates to the by-id entry - the exact `analyze_file_load` /
    `analyze_file_load_by_id` split pattern; the two entries cannot drift.
  - Private `find_symbol_id` kept as a one-line delegating alias because
    `load_explain.rs` (child module, out of touch-set) calls it at its
    line 146.
- New inline `#[cfg(test)] mod symbol_load_by_id` in `load.rs` (RED-first):
  5 locks - full `LoadMetric` parity between entries, helper dual-match
  (id-string vs record-name; fixture symbol `ns::Thing`/`Thing` proves both)
  and miss -> `None`, exact error strings for unknown name
  (`"Symbol not found: Absent"`), unregistered id
  (`"Symbol record not found for: Absent"`, new API), and missing defining
  file (`"No defining file for symbol"`) with name/by-id agreement.

## Map-is-redundant finding (handoff option branch)

The `symbol_id_by_name` side map was NOT built. Verification:
`ReducedSemanticModel::symbols` (`crates/graph/src/reduced.rs:86`) is already
`BTreeMap<SymbolId, ReducedSymbolRecord>`, so id->record resolution is an
O(log S) `BTreeMap::get` - the by-id entry point gets it for free and a
forward map would be redundant for that lookup.
A name->id HashMap would only serve the legacy name entry (the AR-08
consumer is expected to iterate ids and will bypass name resolution
entirely), and it cannot be built honestly within this touch-set: there is
no symbol insertion inside `LoadAnalyzer` (it is a read-only handle), the
model struct lives in `reduced.rs` (out of touch-set, its `symbols.bin`
serialization layout is frozen), and per-analyzer memoization would live in
`load_caches.rs` (also out of touch-set) while interior mutability on the
handle would break its documented `Send + Sync` contract.
Ponytail principle applied: deliverable is the by-id entry + public
resolution helper, per the handoff's stated fallback.

## Deliberately NOT done

- No edits to `stage_load.rs` (AR-08), `graph.rs`, `traversal.rs`,
  `callgraph.rs`, `reduced.rs`, `load_caches.rs`, `load_explain.rs`.
- No removal of `find_symbol_id` (child-module caller); no change to any
  existing function's reachable public behavior or error strings.

## Deviations / notes

- RED phase presented as `error[E0599]: no method named
  'analyze_symbol_load_by_id' / 'symbol_id_by_name'` (test module written
  first; in Rust the predicted failure for a missing API is the compile
  error itself, confirmed at `target/testrun-capture.txt` before any impl).
- One error message on a dead branch differs by construction: old
  `analyze_symbol_load` formatted "Symbol record not found for: {query}",
  new via delegation formats "{symbol_id.0}". That branch required
  `model.symbols.get` to fail on an id just found by iterating the same map
  - unreachable before and after; not locked by any existing test.
- Test suite is inline in `load.rs` rather than a sibling `#[path]` file
  because the handoff commit scopes cover only `load.rs`,
  `crates/graph/tests/**`, and this journal; `load.rs` stays under 1000
  lines.

## Gate results (verbatim digest lines)

- RED: `### cargo test DRIFT (exit 2) [capture: ...target\testrun-capture.txt, 118 lines]` (E0599 on the two new methods only).
- GREEN: `116 passed / 0 failed / 0 ignored / 5 suites` (re-run after refactor review: identical).
- `python tools/encoding_guard.py --all` - clean (exit 0, re-run confirmed).
  `--staged` was not separately observable: `commit_scoped.py` stages and
  commits atomically with no stage-only mode, and `--all` (a superset of the
  scoped bytes) was run clean against the exact committed content.
- `python tools/hygiene_check.py` - zero new findings from load.rs; all reported R1/R2/R3 rows are pre-existing (ui/server.rs, builder.rs, etc.).

## Commit-scope incident (disclosed)

- Commit `8ea8d0d` carries one foreign file: the handoff-mandated scope glob
  `crates/graph/tests/**` swept in the AR-06 worker's UNTRACKED in-flight
  file `crates/graph/tests/ar06_edge_dedup_test.rs` (created in this shared
  worktree before my commit; I never read or edited it).
- Disclosure path taken: raw `git reset`/history rewrite is permission-denied
  by repo rules (correctly - concurrent workers are landing commits into the
  same branch during this session), `commit_scoped.py` has no unstage mode,
  and deleting the file would destroy another worker's WIP. So the commit
  stands and the leak is posted to the board for main + AR-06 disposition.
- Containment evidence: the retained full-workspace capture shows
  `Running tests\ar06_edge_dedup_test.rs` passing with zero failed result
  lines, so the committed bytes compile and are green; the AR-06 worker's
  working-tree copy is untouched and future edits commit normally.
- Gate-line provenance: all digest lines above are from my own run stdout
  per the board ruling that `target/testrun-capture.txt` is clobbered by
  concurrent runs.
