# AR-08 Execution Results (reviewer acceptance pass)

Reviewer run time: 2026-10-10 ~05:33-05:55 UTC. Head reviewed: `73f2160`
(branch `shining-enthusiasm`, ahead 14). Ground truth via
`python tools/checkpoint_guard.py status`: foreign dirt confined to
`crates/graph/src/{traversal,graph}.rs` (AR-09 WIP),
`crates/graph/tests/ar09_adjacency_equivalence_test.rs` (AR-09 untracked),
and `tests/integration/tests/m1*/m21` files (AR-17 WIP). All gates below
were run fresh by the reviewer with own stdout.

## Acceptance criteria vs evidence

| # | Criterion | Result | Evidence line (verbatim) |
|---|-----------|--------|--------------------------|
| 1 | Scoped pipeline gate | PASS, exactly the expected line | `35 passed / 0 failed / 4 ignored / 8 suites` (`### cargo test PASS (exit 0)`, capture `target\testrun-capture.txt`) |
| 2 | m00 parity + golden suites UNEDITED green | PASS - `m00_golden_baseline_test` + `m00_facts_path_parity_test`; UNEDITED proven twice (empty `git diff 3a26a9a..73f2160 -- tests ...`; not in the worktree dirty set) | `4 passed / 0 failed / 0 ignored / 2 suites` |
| 3 | bench_compare, no regressions; symbol-metrics rows 0.971x/0.965x | PASS with attribution note (verdict row 1 of verdict file). Journal's three runs all PASS; reviewer rerun A (contended) FAIL with 4 non-stage-5 WARNs; reruns B and C each FAIL on exactly one 1 ms clone-row WARN (flapping cold<->warm), targeted row BELOW baseline both times | journal x3: `bench compare: 3 run(s), no regressions past 1.25x baseline`; targeted row: journal AFTER runs 1539.6/1529.9 vs BEFORE 1586.0; reviewer reruns `symbol_pass+prep` 1565.3 and 1576.0 vs baseline 1585.6; rerun B: `WARN get_or_build_include_graph clone (cold): 1.7 ms vs baseline 1.3 ms (+31%)`; rerun C: `WARN get_or_build_include_graph clone (warm): 1.4 ms vs baseline 1.0 ms (+40%)` |
| 4 | encoding_guard --all | PASS | silent (no findings over the whole tracked tree) |
| 5 | hygiene_check, foreign rows only | PASS | Blocking rows, all foreign: `crates/ui/src/server.rs: R2 1172 lines > 1000`, `crates/ui/src/server.rs:816: R1 fn \`api_graph_traversal\` is 161 lines (> 100)` (AR-13), `tests/integration/tests/m15_graph_ui_test.rs:9: R1 fn \`test_m15_graph_ui_command\` is 104 lines (> 100)` (AR-17, m15 still dirty). Zero rows on `crates/pipeline/src/stage_load.rs` or `crates/pipeline/tests/load_symbol_metrics_lock_test.rs`; remaining rows R3 warn-only |
| 6 | Commit vs touch-set | PASS | `git diff --name-only 3a26a9a..73f2160` -> 3 paths, all inside handoff globs; `git show --stat` per commit: `7f7df89` = stage_load.rs + lock test + journal; `73f2160` = journal only. Zero `crates/graph` paths |
| 7 | Tools unit suites | PASS | `Ran 219 tests in 19.349s` / `OK` |

## Byte-identity / central-claim checks (not a gate command, but gate #2 of the hop)

- Legacy scan read at `crates/graph/src/load.rs:226-232`; model map is
  `BTreeMap<SymbolId, ReducedSymbolRecord>` (`reduced.rs:86`); name entry
  delegates to `analyze_symbol_load_by_id` (load.rs:563-568). Resolution
  table's first-wins rule shown equivalent by order-preserving vacant-slot
  insertion over both arms (verdict, "central claim" section, items 1-2).
- Scala shared-name fixture PRESENT in the lock test with the legacy-quirk
  literal (both lines carry `a/widget.scala`/`a.Widget`); non-vacuity
  mutation evidenced: journal lines `1/0/0/1` (capture), `3/0/0/1` (green
  pre-change), `2/0/0/1 -> 1 failed` (naive-by-id mutation, restored).
- Quadratic-removal probe table recorded in journal (93.9 vs 1.4 ms at
  S=2160; 13558.7 vs 16.3 ms at S=24000); probe never committed (post-deletion
  suite line identical `35 passed / 0 failed / 4 ignored / 8 suites`).
  Reviewer did not re-run the probe, per dispatch.

## NOT run (explicit)

- `python tools/testrun.py --baseline` (whole-workspace final gate): not run
  here. Dispatch explicitly restricted the acceptance scope to the five
  listed gates due to three concurrent agents mid-flight, and the repo
  planning law assigns whole-workspace final gates to a fresh session or the
  orchestrator's own bounded run. A full-workspace result in this shared
  worktree would compile AR-09/AR-17 WIP and cannot be attributed to AR-08.
- `python tools/ui_smoke.py`: not in this milestone task's gate list and not
  requested by the dispatch; not run.
- `bench_compare --bench stage5-memory` (RSS mode): not in the handoff gate
  list (journal states why: 5-invocation RATIFIED protocol, separate
  expensive gate); not run.
- Refusals encountered (boilerplate catch-all-deny): `git grep <rev>`,
  `python -c ...`, piped/redirected invocations. None removed an expected
  gate; blob-equality was established via the dirty-file census plus Read/grep
  on the proven-untouched files, and the tools suite ran bare.

## Attribution caveat on reviewer bench numbers

Reviewer bench runs compiled the dirty shared worktree, i.e. HEAD +
AR-09's uncommitted `crates/graph` changes; their numbers measure
HEAD+WIP, which is why the journal's clean-tree PASS runs carry the
ratio claim (0.971x/0.965x) and the reviewer reruns serve only to show no
stage-5 regression attributable to AR-08 (targeted row below baseline in
both clean reruns).
