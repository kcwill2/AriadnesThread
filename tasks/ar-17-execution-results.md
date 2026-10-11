# AR-17 Execution Results (reviewer-owned fresh gates on HEAD 55c93c8)

Tree at review: branch `shining-enthusiasm`, HEAD `55c93c8`, worktree dirty
only with AR-11 WIP (`crates/ui/assets/hotspots.js`, `crates/ui/tests/ui_test/
main.rs` +1 line) and untracked task docs - no attribution of any suite line
to AR-11 was needed (no asset-content assertion failure occurred).

## Acceptance criteria vs. what actually happened

| # | Criterion | Result | Evidence (verbatim) |
|---|-----------|--------|---------------------|
| A1 | m10 suite green | PASS | `5 passed / 0 failed / 0 ignored / 1 suites` |
| A2 | m15 suite green | PASS | `1 passed / 0 failed / 0 ignored / 1 suites` |
| A3 | m16 suite green | PASS | `2 passed / 0 failed / 0 ignored / 1 suites` |
| A4 | m17 suite green | PASS | `1 passed / 0 failed / 0 ignored / 1 suites` |
| A5 | m18_cache_epoch green | PASS | `2 passed / 0 failed / 0 ignored / 1 suites` |
| A6 | m18_persistence_incremental green | PASS | `7 passed / 0 failed / 0 ignored / 1 suites` |
| A7 | m21 suite green | PASS | `1 passed / 0 failed / 0 ignored / 1 suites` |
| A8 | encoding gate | PASS | `python tools/encoding_guard.py --all` -> zero output; silent output is exit-0 per `tools/encoding_guard.py:173` (`return 1 if violations else 0`) |
| A9 | hygiene gate: blocking rows exactly the 2 AR-13 server.rs rows | PASS | `crates/ui/src/server.rs: R2 1172 lines > 1000` / `crates/ui/src/server.rs:816: R1 fn \`api_graph_traversal\` is 161 lines (> 100)` - no other blocking row; m15 fix holds |
| A10 | Touch-set exact | PASS | `git diff --name-only "55c93c8^..55c93c8"` -> 8 files, all inside the declared set (journal + m10/m15/m16/m17/m18_cache/m18_persistence/m21) |
| A11 | T4 alien-CWD independence (reviewer reproduction) | PASS | committed binary run via `distill.py` (`cwd=REPO_ROOT`): `### generic PASS (exit 0)`; capture `target/reviewer-m17-alien-cwd.txt`: `test result: ok. 1 passed; 0 failed; 0 ignored; ...` |
| A12 | T5 budgets verbatim, bounded loops | PASS | pre/post blobs: `0..120` x 250 ms in all three sites; ready predicate identical; only sleep position moved (leading -> between); terminal asserts unchanged in diff context |
| A13 | T6 hard locks + gating integrity | PASS | m21:345 `.expect("live route probe must fire...")`, m21:356 `.expect("trace must contain a loading snapshot...")`, 503 assert at m21:253-255; `git diff --stat "14abc55..55c93c8" -- crates/ui/src/server.rs` empty; HEAD server.rs:283 `.ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)` intact |
| A14 | m15 functions < 100 lines | PASS | helper 18 lines; test fn 88 lines (file 33-120); no m15 hygiene row |
| A15 | Tool unit suites | PASS | `Ran 219 tests in 19.119s` / `OK` (`python -m unittest discover -s tools -p "*_tests.py"`) |

## Explicitly NOT run (and why)

- `python tools/testrun.py --baseline` (full-workspace baseline): excluded
  by the orchestrator's adjudication gate list for AR-17 and the Planning
  Law (workspace-wide final gates belong to the orchestrator's fresh
  session, not a handoff adjudication).
- `python tools/ui_smoke.py`, `python tools/bench_compare.py`: milestone-level
  gates; AR-17 is test-hardening with no runtime/benchmark claim; not in the
  handoff gate list.
- Mutation re-run (weaken `api_graph_include` to force m21 RED): impossible
  under the reviewer's read-only grant (source edits denied). Replaced by
  independent code-reading proof of non-vacuity, byte-exact server.rs
  integrity across the commit range, and line-number consistency of the
  journal's RED panic with the committed assert (m21:253).
- Pre-fix alien-CWD FAIL re-execution: would require building the superseded
  revision (write-git denied). Post-fix alien-CWD PASS was reproduced
  directly; pre-fix blob inspected and consistent with the journal.
- `hygiene_check.py` process exit code: `$LASTEXITCODE` capture forms are
  refused by the bash allow-list; the blocking-row set was captured verbatim
  instead (see A9).

## Refused reviewer commands (boilerplate `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`)

`Get-ChildItem`, `python -c ...`, a `git status --short`+`echo` chain,
`python tools/distill.py --cmdlist` probe, `$LASTEXITCODE` wrapper rerun.
None corresponded to an expected gate; all expected gates ran (see
`tasks/ar-17-verdict.md`).
