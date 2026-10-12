# AR-16 Execution Results - Worker dispatch contract tests (T3)

Reviewer-run, fresh, against committed range `70e1699..a48c018`
(`ea4c576` tests, `a48c018` journal hash record), worktree
`shining-enthusiasm` at HEAD `a48c018`. Scheduler package verified clean vs
HEAD (checkpoint census dirt confined to `crates/ui` and
`tests/integration` - foreign active lanes).

## Acceptance criteria vs what actually happened

### 1. Gate command 1 - scoped scheduler suite (distilling entry point)

Command: `python tools/testrun.py --distill -- -p ariadne-scheduler`

Own stdout (verbatim):

```text
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\.kilo\worktrees\shining-enthusiasm\target\testrun-capture.txt, 47 lines]
34 passed / 0 failed / 0 ignored / 2 suites
no failures
34 passed / 0 failed / 0 ignored / 2 suites
```

Canonical line: `34 passed / 0 failed / 0 ignored / 2 suites` - matches the
journal's claimed line exactly (34 = prior 30 + 4 new dispatch-contract
tests). MET.

### 2. Gate command 2 - encoding

Command: `python tools/encoding_guard.py --all`

Own stdout: empty (documented pass form) with explicit exit code
`exit=0`. MET. Pure-ASCII constraint honored.

### 3. Gate command 3 - hygiene

Command: `python tools/hygiene_check.py` -> `exit=1`.

Zero `crates/scheduler/src/worker.rs` rows of any kind (journal's claim
verified). The exit-1 is entirely foreign/pre-existing. R1/R2 rows verbatim:

```text
crates/ui/src/server.rs: R2 1172 lines > 1000
crates/ui/src/server.rs:816: R1 fn `api_graph_traversal` is 161 lines (> 100)
tests/integration/tests/m15_graph_ui_test.rs:9: R1 fn `test_m15_graph_ui_command` is 104 lines (> 100)
```

Attribution: row 1 = pre-existing AR-13-owned 1155-line file plus 17 net
lines of foreign UNCOMMITTED `crates/ui/src/server.rs` WIP (census +26/-9);
row 2 = pre-existing AR-13-owned row as recorded by AR-01/AR-05 execution
results; row 3 = foreign UNCOMMITTED integration WIP - the committed function
is 99 lines (104 minus the 5 net in-function additions present only in the
uncommitted diff), i.e. under the cap at commit time. All remaining rows are
warn-only R3. MET for AR-16 (zero new rows from this handoff; the 104-row is
routed to the integration lane, see verdict follow-up).

### 4. Touch-set + production-byte invariants

- `git diff --name-only 70e1699..a48c018` = exactly
  `crates/scheduler/src/worker.rs`, `tasks/ar-16-handoff-journal.md`. EXACT.
- `ea4c576` worker.rs diff: single hunk `@@ -106,3 +106,550 @@`, 547
  insertions, 0 deletions, addition begins at `#[cfg(test)] mod
  dispatch_contract_tests`. Zero production lines changed. MET.
- Committed `worker.rs:84` carries the original contract wording
  `unhandled language for {:?}` (mutation never landed); zero residual
  mutation string in the file. MET.

### 5. Lock non-vacuity (per journal claims, independently checked)

- Missing-language lock: real registry + real `CppPlugin`; assertions match
  the real integration locks verbatim
  (`m24_language_identity_test.rs:379` contains-`"unhandled language"`,
  `:383` not-`"Cpp"`; `worker_backend_parity.rs:57` same phrase) - locked,
  not redefined. Both wording locks assert the phrase directly, so the
  logged wording mutation provably turns exactly those two red. MET.
- Unknown-task lock: production `analyze_task` dispatch driven through
  `SelfRenamingIdPlugin` (mock is the plugin, not dispatch); locks the
  production `no plugin registered for language` arm + zero analyzer calls;
  unreachability-with-consistent-registry rationale verified against the
  dispatch structure and honestly recorded. MET.
- Cancel lock: body-only analyzes exactly [unit]; store-context sibling
  analyzes [unit, header] in closure order stamped with the unit id -
  asserts body-only branch AND plugin-closure merge. MET.

### 6. Workspace tooling suites

Command: `python -m unittest discover -s tools -p "*_tests.py"`

Own stdout (verbatim tail):

```text
Ran 219 tests in 18.237s

OK
exit=0
```

MET.

## NOT RUN (explicit statement)

- `python tools/testrun.py --baseline` - not run, NOT as a dodge: the
  review-gate procedure assigns the workspace-level milestone gate to the
  orchestrator, and four foreign lanes currently hold uncommitted WIP in
  `crates/graph`, `crates/ui`, `crates/pipeline`, `tests/integration`; a
  baseline here would measure their in-flight code, not this handoff. Per
  AGENTS.md, workspace-wide final gates run only in a fresh/clean session or
  the orchestrator's bounded own run. Orchestrator owes this line at
  milestone closeout.
- `python tools/ui_smoke.py`, `python tools/bench_compare.py` - not run; not
  required by AR-16 (test-only change, zero production/runtime bytes).
- `python tools/golden_guard.py` - not run; does not exist yet (skill names
  it as "once it lands"); no golden fixtures in AR-16's scope, and the
  byte-identity claim was instead proven from the insertions-only hunk plus
  committed-wording reads.

## Refused command forms (verbatim, configuration finding not implementer
finding)

Piped command forms and inline `python -c` were refused with:

```text
{"permission":"bash","pattern":"*","action":"deny","source":"global"}
```

Affected attempts: `git show --stat "rev" | Select-Object ...`,
`git show "rev:path" > tempfile`, `python -m unittest ... 2>&1 |
Select-Object`, multi-line and one-line `python -c` measurements. None was a
genuine tool unavailability: plain `git show`/`git log`/`git diff`,
`python tools/*`, and unpiped `python -m unittest` all ran; byte-safe
substitutes (Read on clean checkout, committed-diff arithmetic) replaced the
refused forms without weakening any check.
