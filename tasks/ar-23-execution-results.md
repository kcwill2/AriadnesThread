# AR-23 Execution Results (reviewer acceptance pass)

Workspace: `shining-enthusiasm` worktree, HEAD `7adbeca` at adjudication
(AR-23 commit `d8559ae`, child `7adbeca` = AR-20 journal docs only).
Ground truth via `python tools/checkpoint_guard.py status`: branch
`shining-enthusiasm`, committed tree CLEAN (dirty files: none; untracked = the
foreign `tasks/ar-*` doc lane files + one plan file). No lane dirt could
contaminate any measurement. AR-20 lane idle pending re-scope (journal-only
commits); ui lane has AR-11 committed (`adb54ea`); the AR-11 reviewer ran its
gates concurrently but caused no build contention and no retry was needed.

## Gate commands - canonical lines verbatim (own stdout, fresh runs)

1. `python tools/testrun.py -- -p ariadne-cli` ->
   `114 passed / 0 failed / 8 ignored / 6 suites` (exit 0). Matches the
   expected line exactly (113 lib + 1 bench-suite `ok`; 1 + 7 ignored).
   Reconciles with the journal's `117 baseline - 3 helper-named tests
   deleted + 0 new in-crate`.
2. `python tools/testrun.py -- -p ariadne-pipeline` ->
   `40 passed / 0 failed / 4 ignored / 9 suites` (exit 0; exact match; the
   5-test `for_incremental_parity_test` all green; `crates/pipeline` absent
   from the commit's file list = UNEDITED).
3. `python tools/testrun.py -- --test m00_golden_baseline_test` ->
   `2 passed / 0 failed / 0 ignored / 1 suites`;
   `python tools/testrun.py -- --test m00_facts_path_parity_test` ->
   `2 passed / 0 failed / 0 ignored / 1 suites`. Both files OUTSIDE the
   commit's 3-path list (UNEDITED confirmed by `git diff --name-only d8559ae^
   d8559ae`).
4. `python tools/testrun.py -- --test ar23_analyze_incremental_parity_test`
   -> `4 passed / 0 failed / 0 ignored / 1 suites` (all four C1-C4 locks green
   in the listing).
5. `python tools/encoding_guard.py --all` -> silent (exit 0).
   `python tools/hygiene_check.py` -> exit 1 solely from the two known
   pre-existing `crates/ui/src/server.rs` blocking rows
   (`crates/ui/src/server.rs: R2 1172 lines > 1000` and
   `crates/ui/src/server.rs:816: R1 fn \`api_graph_traversal\` is 161 lines`).
   Zero rows of any kind from `crates/cli/src/commands/analyze.rs`,
   `tests/integration/tests/ar23_analyze_incremental_parity_test.rs`, or the
   journal; all remaining rows warn-only R3 in foreign/pre-existing files.

## Acceptance pass (workspace-wide; not carried by any implementer handoff)

- `python tools/testrun.py -- --workspace --no-fail-fast` ->
  `1372 passed / 0 failed / 13 ignored / 99 suites` (exit 0).
  FIRST census attempt (default fail-fast) recorded ONE flake, retried once:
  `internal_tests::test_load_report_matches_sequential_reference` panicked
  with `ARIADNE_LOAD_MEM_CAPS must be '...', got "3,x"`
  (`crates/pipeline/src/stage_load.rs:223`), truncating the run at
  `946 passed / 1 failed / 8 ignored / 60 suites`. Root cause is pre-existing
  and foreign: the `caps_env_ablation` mod (landed `f312dba`, M28 T0c)
  mutates the process-global env var under a mutex shared only among its own
  three tests; parallel siblings in the same pipeline lib-test binary read it
  when workspace feature unification compiles `test-instrument`. The value
  `"3,x"` is that mod's own malformed-input literal
  (`stage_load_tests.rs:350`). `crates/pipeline` is byte-untouched by AR-23,
  and the scoped pipeline binary (gates 1-2, green twice, incl. 19/19 in the
  first census) does not compile the caps mod. Verdict finding 1; ledger
  deferred-findings candidate at closeout. The retry-once re-run (this section
  header line) is fully green with all 99 suite binaries executed.
- `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests in 24.247s` / `OK` (exit 0).

## Acceptance criteria -> observed

- Touch-set EXACT (3 paths): PASS (`git diff --name-only d8559ae^ d8559ae` =
  analyze.rs, new lock, journal; pipeline/golden/fixtures absent).
- Mirror truly dead: PASS (base blob read whole: sequence :27-46..56 region,
  helpers + mirror comment :79-121; committed file has none of it; grep for
  `restrict_and_validate_units` / `artifacts_are_authoritative` /
  `BuildDiscoverer` / `Repository::locate` in analyze.rs: no incremental-path
  hits; `Pipeline::for_incremental` is the construction path; the
  `incremental_backend` pairing survives cli-side over `&pipeline.build`,
  analyze.rs:37-39; guard text verbatim vs `crates/cli/src/registry.rs:29`).
- Parity authenticity: PASS (lock asserts captured literals - 15-file set
  with both shard ids, 2-file shard-only set, stats `0/2/0.0` and `2/0/100.0`
  - no re-derivation; C3 persist-nothing is a cross-run size snapshot;
  mutation `2 passed / 2 failed` on exactly the C3+C4 persist-skip locks is
  the structurally forced outcome of `authoritative := true`; expectations
  byte-identical pre/post per the journal's ordering; print/persist blocks
  provably unchanged by the diff).
- Carried dispositions: PASS (Finding 2 via file-set observability - sound,
  C4 pre-change capture proves the mirror always ends with `artifacts/` on
  successful runs; ctor creates the dir only after locate/restrict, further
  shrinking the residual delta; Finding 3 - every fps mention in
  journal/lock is an explicit NO-CLAIM disclaimer, grep-verified).
- Baseline reconciliation: PASS (git show deletions: exactly 3 helper-named
  unit tests, each calling the deleted `artifacts_are_authoritative` with
  literal args; `117 - 3 = 114` matches the fresh gate line).
- Compile-green: PASS (every gate compiled cli + integration tests without
  errors; only pre-existing foreign warnings).

## Refusal record (gate-execution integrity)

Two invocations were refused with the catch-all boilerplate
(`The user has specified a rule which prevents you from using this specific
tool call. Here are some of the relevant rules
{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
(a) a chained `git diff ... ; echo ...; git show --stat ...` line - both
constituents then ran individually and succeeded; (b) `python -c` byte-dump of
the base blob - substituted by direct `git show "d8559ae^:...path..."` to
stdout (blob is pure ASCII under the encoding gate). Neither refusal indi-
cates a skipped gate; every expected gate ran with full own-stdout output.

## Explicitly NOT run

- `python tools/testrun.py --baseline` in its literal form: UNRUN -
  `--baseline` requires a baseline FILE argument and no test-baseline file
  exists in the repo (consistent with the AR-03 precedent note). Substituted,
  with more coverage, by the default workspace census (`cargo test
  --workspace --no-fail-fast`) - the canonical line for that is quoted above.
- `python tools/ui_smoke.py`, `python tools/bench_compare.py`: not required by
  the AR-23 handoff (no UI surface, no perf claim on this path); not run.
- No source mutation performed by the reviewer; the mutation proof was
  verified structurally against the committed code (only C3/C4 depend on the
  authority bit) plus the reported 2p/2f run and my green re-run of the
  reverted final state.

## Gate-execution environment note

`tools/testrun.py` raw-echo mode crashes on the cp1252 console
(`UnicodeEncodeError` in `_echo`, testrun.py:262) whenever cargo emits
non-ASCII; all gates above were run with `$env:PYTHONIOENCODING='utf-8'`.
Foreign tooling note (verdict finding 3), no bearing on any canonical line.
