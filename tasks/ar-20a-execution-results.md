# AR-20a Execution Results — fresh reviewer gates at `4dc7237`

Reviewer-session run 2026-10-11 (fresh session, worktree tracked-tree clean
at both ground-truth reads). All canonical lines quoted verbatim from own
stdout. Capture: `target/testrun-capture.txt` (shared - capture-race rule:
lines below are from the run itself, not the capture file).

## Ground truth

`python tools/checkpoint_guard.py status`: HEAD `4dc7237`, branch
`shining-enthusiasm`, tracked tree clean (untracked = tasks/docs only).
Range `768529f..4dc7237` = exactly `0861db0` + `4dc7237`. Worktree census:
10 worktrees, no foreign tracked dirt in this worktree (re-verified at end).

## Scoped handoff gates (reviewer re-runs, verbatim)

| # | command | canonical line (own stdout) | pinned | verdict |
|---|---|---|---|---|
| 1 | `python tools/testrun.py -p ariadne-worker-core --timeout 500` | `8 passed / 0 failed / 0 ignored / 2 suites` | 8/0/0/2 (dispatch) | MATCH |
| 2 | `python tools/testrun.py -p ariadne-scheduler --timeout 500` | `26 passed / 0 failed / 0 ignored / 2 suites` | 26/0/0/2 | MATCH |
| 3 | `python tools/testrun.py -p ariadne-lang-cpp --timeout 500` | `153 passed / 0 failed / 0 ignored / 2 suites` | 153/0/0/2 UNEDITED | MATCH |
| 4 | `... --test m18_cpp_plugin_parity_test` | `23 passed / 0 failed / 0 ignored / 1 suites` | 23/0/0/1 UNEDITED | MATCH |
| 5 | `... --test m18_persistence_incremental_test` (via `--distill`) | `7 passed / 0 failed / 0 ignored / 1 suites` | 7/0/0/1 UNEDITED | MATCH |
| 6 | `... --test m00_golden_baseline_test --test m00_facts_path_parity_test` (via `--distill`) | `4 passed / 0 failed / 0 ignored / 2 suites` | 4/0/0/2 UNEDITED | MATCH |
| 7 | `... --test m16_clang_worker_test` (via `--distill`) | `3 passed / 0 failed / 0 ignored / 1 suites` | pinned 2 (stale) | GREEN + UNEDITED (D1 -> ledger row bank) |

Conservation equation (invariant 4), reviewer-verified: **8 + 26 = 34** =
pre-move `34 passed / 0 failed / 0 ignored / 2 suites` (journal Phase-0 +
independent AR-16 board landing line). Roster partition verified by blob-hash
identity of `tu_store.rs` (all 8 test fns identical), zero `#[test]`
add/remove in the 3 modified scheduler file diffs, and all other scheduler
files byte-identical; the 8 engine test names run under worker-core and zero
of them remain in the scheduler run (per-test output inspected).

Gate-1 note: my first bare-mode m18_persistence attempt crashed inside
`testrun.py` bare echo (`UnicodeEncodeError` cp1252, `tools/testrun.py:262`)
- reviewer-harness console issue, gate re-run green via the contract's
`--distill` form. Recorded as verdict finding 4.

## R100 / touch-set / locks (commands + results)

- `git rev-parse` triple: worker-core blob at `4dc7237`, scheduler blob at
  `0861db0^`, worker-core blob at `0861db0` - all
  `202df67913c63b5eb7bfccf242bd89309b9db834`. R100 authentic.
- `git diff --name-status -M "768529f" "4dc7237"` - exactly 12 paths, all in
  the declared touch-set, tu_store as `R100`.
- `git diff --name-only "768529f" "4dc7237" -- crates/lang-cpp crates/lang-api
  tests/ crates/index crates/build crates/lang crates/pipeline crates/cli
  crates/ui crates/report` - EMPTY (m17 guard, m18 harnesses, m00 goldens all
  byte-empty in both commits).
- `git rev-parse "0861db0:crates/scheduler/src/tu_store.rs"` ->
  `fatal: path ... does not exist in '0861db0'` (D7 landing complete).
- `git rev-parse "0861db0^"` -> `768529f` (parent = dispatch HEAD).
- Hit-set grep `scheduler::(tu_store|queue|worker)::` = the journal's
  post-move set exactly (m18_persistence:226 compiled lock + 2 doc hits + the
  new shim comment).

## Workspace-wide acceptance pass

- `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests in 20.723s` / `OK` (exit 0).
- `python tools/testrun.py --distill --timeout 550` (workspace census, first
  attempt) -> `946 passed / 1 failed / 8 ignored / 60 suites` (exit 1; cargo
  fail-fast truncation). Failure pre-existing and foreign:
  `internal_tests::test_load_milestones_absent_on_cache_hit` panicked on
  `ARIADNE_LOAD_MEM_CAPS ... got "3,x"` - env-window race in the M28 T0c
  cap-ablation harness (`stage_load_tests.rs`, from `f312dba`; pipeline
  byte-empty in AR-20a range).
- `python tools/testrun.py --distill -p ariadne-pipeline --timeout 550` ->
  `40 passed / 0 failed / 4 ignored / 9 suites` (standalone green).
- `python tools/testrun.py --distill --timeout 550` (workspace census, retry)
  -> **`1372 passed / 0 failed / 13 ignored / 101 suites` (exit 0)** -
  canonical workspace census line.
- `python tools/testrun.py --baseline`: NOT RUN - not runnable: `--baseline`
  requires a FILE argument (`testrun.py:206,119-128`) and no repo baseline
  file exists (glob for `*baseline*` finds only
  `tools/theme_token_baseline.json`, `tools/bench_baseline.json`, and the m00
  test source). Bare-form usage error documented; substituted canonical
  workspace census line above (same disposition AR-03/AR-10 reviewers).
- `python tools/hygiene_check.py` -> R3 warn-only rows across the repo, ZERO
  R1/R2 blocking rows (post-AR-13 baseline held); touched new files
  (`worker-core/src/{lib,task,worker_backend}.rs`) produce zero rows; moved
  `tu_store.rs` rows are the 3 pre-existing `tmp` warns traveling verbatim.
- `python tools/encoding_guard.py --all` -> no output, exit 0 (silent).
- `python tools/ui_smoke.py` / `python tools/bench_compare.py`: NOT RUN -
  not demanded by the AR-20a contract (pure verbatim-move refactor, no UI or
  runtime path edits; zero-delta carried by m18/m00 parity locks + R100 blob
  identity + the green workspace census).

## Verdict

APPROVED. AR-20b (switch + dep removal + m17 guard GREEN-once at
`8 passed / 0 failed / 0 ignored / 1 suites` per the planner's census
correction) is UNBLOCKED. See `tasks/ar-20a-verdict.md` for numbered
findings (all minor: pipeline env-race flake row, commit_scoped tooling row,
stale m16/suite-count pins, testrun bare-echo console bug row,
fixture-regeneration note, reviewer allowlist observation).
