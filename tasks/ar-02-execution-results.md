# AR-02 Execution Results - reviewer-fresh gates (adjudication of `b49ed06`)

Reviewer run time: 2026-10-09 ~23:35-23:45 UTC, worktree
`shining-enthusiasm`. All canonical lines below are from the reviewer's own
run stdout (capture files race in this shared worktree per the board ruling).
HEAD moved `602c176` -> `70e1699` (AR-06) mid-review; the dirty `graph.rs`
bytes every gate compiled are the bytes `70e1699` committed, so no gate
result changed meaning.

## Dispatch gate lines (verbatim)

1. `python tools/testrun.py --distill -- -p ariadne-ui`
   `### cargo test PASS (exit 0) [capture: ...target\testrun-capture.txt, 283 lines]`
   `245 passed / 0 failed / 0 ignored / 3 suites`  -> MATCHES expected 245/0/0/3.
2. `python tools/testrun.py --distill -- -p ariadne-cli`
   `### cargo test PASS (exit 0) [capture: ...target\testrun-capture.txt, 194 lines]`
   `118 passed / 0 failed / 8 ignored / 6 suites`  -> MATCHES expected 118/0/8/6.
3. Integration compile (arity break cleared):
   - `python tools/testrun.py --distill -- -p ariadne-integration-tests --no-run`
     -> `### cargo test DRIFT (exit 2)` = testrun's pinned-spec contract:
     `--no-run` never prints result lines; NOT a code failure. Capture grep:
     zero `error` lines, and `Executable tests\m15_graph_ui_test.rs`,
     `m16_traversal_test.rs`, `m17_dir_nav_test.rs`,
     `m18_persistence_incremental_test.rs`, `m21_run_ui_smoke_test.rs`,
     `m22_scala_load_analysis_test.rs` all emitted.
   - Formal exit-code claim via journal's form:
     `python tools/distill.py -- cargo test -p ariadne-integration-tests --no-run`
     `ok (exit 0, 21 warnings emitted) -> capture: ...target\distill-capture.txt`
     `EXIT=0`. 21 warnings matches the journal (all pre-existing classes).
4. `python tools/distill.py -- python tools/ui_smoke.py`
   `### generic PASS (exit 0)`
   `boot+route checks clean (cold fixture, attempt 1, 0.6s)`
   `ui_smoke: PASS`
   No weakened path: the smoke spawns the real `tool ui` argv WITHOUT
   `--expose` (tools/ui_smoke.py:185-187) and probes
   `http://127.0.0.1:<ephemeral>` (ui_smoke.py:384); `run_ui`'s only bind is
   `resolve_bind_addr(expose_opt.as_deref())` with no env/bypass. The smoke
   request reaches the production 127.0.0.1 default.
5. `python tools/encoding_guard.py --all` -> silent. Its contract: one line
   per violation, exit 1 on any; silence = zero violations (exit 0).
6. `python tools/hygiene_check.py` -> blocking output is EXACTLY the two
   inherited server.rs rows, zero new:
   `crates/ui/src/server.rs: R2 1155 lines > 1000`
   `crates/ui/src/server.rs:799: R1 fn `api_graph_traversal` is 161 lines (> 100)`
   (ledger deferred row `tasks/milestone_ledger.md:86`, AR-13-owned; count
   drift from the row's 1120 = sanctioned AR-01/AR-02 growth; line drift
   764->799 = insertions above it). All R3 warn rows are pre-existing; the
   new code (cors_same_origin.rs, resolve_bind_addr, warn_if_expose_used,
   is_same_origin) produces no findings.

## Acceptance-pass gates (reviewer-run, no implementer handoff carries these)

- `python -m unittest discover -s tools -p "*_tests.py"`
  `Ran 219 tests in 18.490s` + `OK`
- Workspace census (errata form - see NOT RUN below):
  `python tools/testrun.py --distill`
  `### cargo test PASS (exit 0) [capture: ...target\testrun-capture.txt, 2346 lines]`
  `1322 passed / 0 failed / 13 ignored / 94 suites`
  Full workspace green INCLUDING the concurrent AR-06 graph WIP; zero foreign
  failures to attribute. The E0061 census blocker reported by the AR-01/AR-03
  reviews is cleared by this landing (94 suites compile and run).
- `python tools/hygiene_check.py` and `encoding_guard --all`: as above.
- `python tools/ui_smoke.py`: via distill wrapper, PASS (gate 4 above).

## Static checks (per dispatch)

- Commit-vs-touch-set EXACT incl. D1+D2; table in `tasks/ar-02-verdict.md`.
- server_api.rs byte-identity: blob
  `0c813f0c5c8121788f0a8513818ed91916235672` identical at `3b960bc` /
  `8ea8d0d` / `b49ed06`.
- Bind census: `grep "bind\(" crates/cli` -> 1 production hit (ui.rs:801,
  resolved addr) + 1 lock literal; `grep "bind\(" crates/ui/src` -> none.
- CORS: `permissive` survives only in doc/comment text of the new test mod.
  Lock non-vacuity independently derived: permissive yields header
  `Some("*")` -> all 3 locks fail (matches journal RED `0 passed / 3 failed`,
  `left: Some("*")` x3); bind lock fails on hardcoded `0.0.0.0` with the
  journaled message. RED evidence shape consistent.
- Integration hunks: arg-only `None` additions (m22 hunk sampled;
  assertions byte-untouched). 8 call sites = journal's count.
- `--help`: exposes flag documented ("instead of the 127.0.0.1 default; the
  UI has no authentication and this warns"), locked by
  `test_cli_ui_help_documents_expose_lan_warning`. Runtime stderr warning
  verified in source (`warn_if_expose_used`).

## NOT RUN / could not capture (explicit)

- `python tools/testrun.py --baseline` FILE form: tool requires a baseline
  FILE argument and none exists in-repo; standing 5-encounter ledger errata
  governs this track. Substituted: unscoped `testrun --distill` census
  (`1322 passed / 0 failed / 13 ignored / 94 suites`, exit 0), same form the
  ui-traversal and extraction-card reviews used.
- `bench_compare.py` and `theme_token_check.py`: not required by this
  milestone (no perf claim, zero asset/styling diff).
- Raw shell exit codes for gates 5/6: chained `; echo` forms and
  `python -c` wrappers are off the bash allowlist. Verbatim refusal text on
  every refused call: `{"permission":"bash","pattern":"*","action":"deny",
  "source":"global"}`. Refused calls: `git show --stat "602c176"; git
  ls-files ...` (compound), `python tools/encoding_guard.py --all; echo ...`
  (chained), `python tools/hygiene_check.py; echo ...` (chained), and the
  `python -c "import subprocess..."` exit-code probe. Unchained forms of
  BOTH tools ran successfully; exit semantics were read from the tools'
  documented contracts and hygiene_check.py source
  (`return 1 if violations else 0`) instead. No gate in the dispatch list
  was skipped.
