# AR-16 Handoff Journal - Worker dispatch tests (T3 MED)

- Handoff: `tasks/ar-16-handoff.md` (Wave 5 test hardening, scheduler crate)
- Worktree: `shining-enthusiasm`, starting HEAD `602c176`
- Result: four characterization tests in the `worker.rs` test module only;
  zero production-code change (diff is insertions-only, verified below).

## Handoff item mapping (interpretation, recorded per duty)

The original audit T3 finding (recovered verbatim from the audit session
transcript) reads: "scheduler worker dispatch has zero direct tests ...
Direct unit test for missing-language dispatch." The handoff expands this to
three groups; each maps to a real dispatch arm of
`PluginWorkerBackend::analyze_task` (the only dispatch in `worker.rs`):

| Handoff wording | Dispatch arm under lock | Test(s) |
|---|---|---|
| missing-language dispatch (M24 third-language error contract) | `detect_language(...) -> None` arm, worker.rs:81-84 | 1, 2 |
| unknown-task path | `plugin_for_id(...) -> None` arm (worker.rs:85-88): dispatch answered a language it cannot produce a plugin for | 3 |
| cancel path | store-context arm (worker.rs:89-95): a task carrying a store context CANCELS the body-only analysis branch and must run the plugin closure merge instead | 4 |

Notes:

- Error wording was LOCKED AGAINST the existing integration locks, not
  redefined: `m24_language_identity_test` (contains "unhandled language",
  never "Cpp") and `m18_cpp_plugin_parity::worker_backend_parity` (same
  phrase). The tests assert the same substrings the integration suites pin.
- The `plugin_for_id` arm is unreachable with a consistent registry by
  construction (`detect_language` derives the language id FROM the claiming
  plugin, which `plugin_for_id` then finds). It is characterized through
  `SelfRenamingIdPlugin`, the only observable shape of that disagreement.
- There is no handle-abort cancellation inside a single `analyze_task` call;
  the pool-level break/fatal paths already have their own characterization in
  `pool.rs` and were left untouched. The store-context arm is the cancel
  semantic available at worker dispatch and is what test 4 locks.

## Tests added (`mod dispatch_contract_tests` in `crates/scheduler/src/worker.rs`)

1. `unclaimed_third_language_task_errors_as_unhandled_and_never_default_analyzes`
   - Real `CppPlugin` registry; `.f90` unit errors containing
     "unhandled language" + the path, never naming "Cpp"; `.cpp` positive
     control through the same backend proves the verdict is the language,
     not the harness.
2. `shared_and_arc_delegate_backends_carry_the_unhandled_language_error_verbatim`
   - Borrowed, `SharedPluginWorkerBackend`, and `Arc<dyn WorkerBackend>`
     variants produce byte-identical error text (the documented
     "just delegates" claim; how the backend is constructed/dispatched).
3. `answered_language_without_a_producible_plugin_errors_before_any_analysis`
   - Locks "no plugin registered for language <id>" + zero analyzer calls
     (no silent fall-through to another plugin's analysis).
4. `store_context_task_cancels_the_body_only_branch_and_runs_the_closure_merge`
   - Body-only task analyzes ONLY the unit file; the same task with a store
     context instead runs the closure merge (analyzes unit + resolved
     header in closure order - files the body-only branch can never observe),
     analysis stamped with the unit's file id.

## Per-test first-run status (required record)

All four are contract/characterization locks and the contracts already held:

| Test | First-run status |
|---|---|
| unclaimed_third_language_task_errors_as_unhandled_and_never_default_analyzes | PASS (green first run) |
| shared_and_arc_delegate_backends_carry_the_unhandled_language_error_verbatim | PASS (green first run) |
| answered_language_without_a_producible_plugin_errors_before_any_analysis | PASS (green first run) |
| store_context_task_cancels_the_body_only_branch_and_runs_the_closure_merge | PASS (green first run) |

First run (all of `ariadne-scheduler`, capture
`target/testrun-capture.txt`, 49 lines):

```text
34 passed / 0 failed / 0 ignored / 2 suites
```

Green-first is legitimate for these locks per the handoff rule;
non-vacuity is proven by the mutation below (arm 1 wording), which is the
exact contract the finding says was only indirectly locked.

## Mutation-proof (non-vacuity evidence)

Mutation applied LOCALLY to the production error arm in
`crates/scheduler/src/worker.rs` (line 84):

```rust
// before (contract wording):
.ok_or_else(|| anyhow::anyhow!("unhandled language for {:?}", task.tu.source_path))?;
// mutated (wording dropped):
.ok_or_else(|| anyhow::anyhow!("dispatch refused for {:?}", task.tu.source_path))?;
```

Run under mutation (`target/testrun-capture.txt`, 62 lines):

```text
32 passed / 2 failed / 0 ignored / 1 suites
---- worker::dispatch_contract_tests::shared_and_arc_delegate_backends_carry_the_unhandled_language_error_verbatim ----
the borrowed-variant phrase is the contract: dispatch refused for "src/solver.f90"
---- worker::dispatch_contract_tests::unclaimed_third_language_task_errors_as_unhandled_and_never_default_analyzes ----
error must name the failure honestly: dispatch refused for "src/solver.f90"
```

Both wording locks went RED for exactly the semantic reason predicted (the
"unhandled language" phrase disappeared); they are not vacuous. The mutation
was reverted via the byte-safe Edit tool and NEVER committed. Revert proof:
`git diff --stat` (via distiller) after revert reports
`crates/scheduler/src/worker.rs | 547 +++++++++ ... 1 file changed, 547 insertions(+)`
- insertions only, zero deletions: every pre-existing production line is
byte-intact, and no occurrence of "dispatch refused" remains in the file.

Green re-run after revert:

```text
34 passed / 0 failed / 0 ignored / 2 suites
```

## Gates (quoted canonical lines)

1. `python tools/testrun.py --distill -- -p ariadne-scheduler`
   -> `34 passed / 0 failed / 0 ignored / 2 suites` (exit 0; final green;
   capture `target/testrun-capture.txt`, 49 lines)
2. `python tools/encoding_guard.py --all`
   -> PASS, exit 0 (via `python tools/distill.py --` digest:
   `### generic PASS (exit 0)`; bare run printed nothing, which is its
   pass form)
3. `python tools/hygiene_check.py`
   -> zero NEW entries: no `crates/scheduler/src/worker.rs` lines at all.
   Gate itself exits 1 on the two plan-known pre-existing AR-13-owned
   `crates/ui/src/server.rs` rows (R2 1155 lines; R1 `api_graph_traversal`
   161 lines) plus warn-only R3 rows, exactly as recorded by AR-01/AR-05
   execution results; hygiene exit-0 is plan-unreachable until AR-13 lands.
4. `python tools/encoding_guard.py --staged` (post-staging)
   -> see final entry below.

## Deliberately NOT done / deviations

- No production-code change: touch-set honored; diff is test-module-only
  (insertions-only diff proven above).
- The unknown-task and cancel group names were ambiguous against the code
  (the original T3 finding only named missing-language dispatch). Mapping to
  the `plugin_for_id` arm and the store-context arm is recorded above with
  rationale; if the orchestrator intended pool.rs break/fatal paths, those
  already have characterization in `pool.rs` (out of AR-16 touch-set).
- Did not touch `tests/integration/**`, `pool.rs`, golden fixtures, or any
  other agent's files; no `git add .`; raw `git`/`cargo` refused at
  permission level, all commands routed through `tools/distill.py` /
  `tools/testrun.py` distilling entry points.
- Mutation reverted uncommitted; capture of the mutated run survives only in
  `target/testrun-capture.txt` (ignored path).

## Commit

- Scope: `crates/scheduler/src/worker.rs`, `tasks/ar-16-handoff-journal.md`
  via `python tools/commit_scoped.py --scope ... -m
  "test(scheduler): worker dispatch error-path contracts (T3)"`
- `python tools/encoding_guard.py --staged` -> see digest recorded at commit
  time in the commit message review trail (PASS).
