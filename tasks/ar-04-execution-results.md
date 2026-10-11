# AR-04 Execution Results - Reviewer Acceptance Addendum

Reviewer-run, fresh at HEAD `932041a` (branch `shining-enthusiasm`,
base `3b960bc`), 2026-10-09. Verdict lives in
`tasks/ar-04-verdict.md`.

## Acceptance criteria vs evidence

| # | Criterion | Evidence (verbatim) | Result |
|---|-----------|---------------------|--------|
| 1 | Scoped validate suite green (`testrun.py --distill -- -p ariadne-validate`) | `16 passed / 0 failed / 0 ignored / 5 suites` (exit 0, capture `target/testrun-capture.txt`, 47 lines) | PASS - matches handoff expectation 16/0/0/5 |
| 2 | Encoding gate (`encoding_guard.py --all`) | bare run: no output; distill wrapper: `### generic PASS (exit 0)`, capture `target/distill-capture.txt` (0 lines) | PASS exit 0 |
| 3 | Hygiene (`hygiene_check.py`), zero NEW violations | R1/R2 hits only `crates/ui/src/server.rs` (R2 1155 lines; R1 fn `api_graph_traversal` 161 lines) - untouched by both AR-04 commits, dirty from other concurrent work. Validate crate: only pre-existing warn-only `crates/validate/src/lib.rs:104: R3 (warn) single-letter local 'p'`. New test file + changed regions: no findings | PASS for this handoff |
| 4 | Commit-vs-touch-set EXACT | `git diff --name-only 3b960bc..HEAD` -> `crates/validate/src/lib.rs`, `crates/validate/tests/mid_apply_rollback_test.rs`, `tasks/ar-04-handoff-journal.md` - exactly the declared touch-set | PASS |
| 5 | Success path untouched (transaction tests unedited + green) | `git diff --name-only 3b960bc..HEAD -- ...transaction_apply_test.rs ...move_removal_transaction_test.rs` -> empty; both included in the green 16/0/0/5 run | PASS |
| 6 | Oracles fail mid-sequence after >=1 write; byte-identical restoration asserted | Read `mid_apply_rollback_test.rs`: Lock 1 fails at `lib.rs:223` `create_dir_all` after `lib.rs:243` write landed; Lock 2 fails at `lib.rs:248` `read_to_string` after unit write + genuine consumer mutation; both assert exact bytes + absence + blocker integrity | PASS |
| 7 | Error text `rolled back N files` locked by test | Both tests assert `contains("rolled back 4 files")` on the propagated error | PASS |
| 8 | TDD RED: both oracles failed pre-fix | Journal RED digest `0 passed / 2 failed / 0 ignored / 1 suites` with raw causes `os error 183` / `stream did not contain valid UTF-8`; reviewer independently derived both modes from the code paths | PASS (journal-backed; RED replay needs write access, see below) |
| 9 | Per-commit review oldest-first | `git show --stat "8209a7e"` (3 files, +392/-4) then full `git show "8209a7e" -- crates/validate/src/lib.rs`; `git show --stat "932041a"` (journal only, +19/-4) | PASS |
| 10 | Distillation contract (report evidence) | Journal quotes digest lines + exit codes + capture paths only; gates via `testrun.py --distill` / `distill.py --` | PASS |

## Not run (explicit)

- `python tools/testrun.py --baseline` (workspace), full
  `python -m unittest discover -s tools -p "*_tests.py"`, `ui_smoke.py`,
  `bench_compare.py`: per the AGENTS.md Planning Law these workspace-wide
  final gates belong to the orchestrator/fresh session, and the worktree
  currently carries UNCOMMITTED `crates/cli`/`crates/ui` changes from
  concurrent non-AR-04 work that would contaminate a workspace run.
  The AR-04 adjudication scope (single-handoff gates) is fully covered
  above.
- RED replay of the two oracles: requires reverting the fix; reviewer git
  is read-only. Substituted evidence: journal RED digest cross-checked
  against reviewer's independent derivation of the two failure modes from
  the implementation - consistent.

## Refused commands during review (recorded for completeness)

- `git show "3b960bc:crates/cli/src/optimize.rs" | Select-Object ...`,
  combined-conditional git invocations, and `... ; echo "EXIT=..."`
  chains returned the catch-all deny
  (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`).
  These were boilerplate refusals, not a bash denial: bare
  `git diff`/`git show --stat`, `python tools/testrun.py`,
  `python tools/encoding_guard.py --all`, `python tools/hygiene_check.py`
  and `python tools/distill.py` all ran. Substitutions used per repo
  rules: checkout reads via Read/grep instead of piping blobs through
  PowerShell; exit codes via the `distill.py` wrapper.
