# AR-18 Execution Results — Planning hooks on LanguagePlugin (A1 part 1)

Reviewer-run acceptance pass for `tasks/ar-18-handoff.md` at commit `d937ac0`
(worktree `shining-enthusiasm`, HEAD `d937ac0`, branch `shining-enthusiasm`).
Verdict recorded in `tasks/ar-18-verdict.md`: **APPROVED**.

## Acceptance criteria — what actually happened

| Criterion | Result | Evidence line (verbatim) |
|---|---|---|
| Touch-set hard, 4 paths | MET | `git diff --name-only 55c93c8..d937ac0` -> `crates/lang-cpp/src/plugin.rs`, `crates/lang-scala/src/plugin.rs`, `crates/lang/src/registry.rs`, `tasks/ar-18-handoff-journal.md` |
| Trait defaults `None`, dyn-compatible | MET | registry.rs:54-69 defaults; `&dyn LanguagePlugin` witnesses in both parity tests; all suites compile+run |
| `-p ariadne-lang` | MET | `35 passed / 0 failed / 1 ignored / 2 suites` |
| `-p ariadne-lang-cpp` | MET | `153 passed / 0 failed / 0 ignored / 2 suites` |
| `-p ariadne-lang-scala` | MET | `221 passed / 0 failed / 0 ignored / 2 suites` |
| m18 parity suite UNEDITED green | MET | `23 passed / 0 failed / 0 ignored / 1 suites` (file absent from touch-set and working-tree dirt) |
| m24 identity suite UNEDITED green | MET | `8 passed / 0 failed / 0 ignored / 1 suites` |
| m17 seam guard green (invariant) | MET | `6 passed / 0 failed / 0 ignored / 1 suites` |
| Override parity vs FREE functions via `&dyn` | MET | cpp+scala `planning_hooks_override_the_defaults_with_free_function_parity` green; assertions compare hook to `crate::planning::*` free fns on identical inputs (curried oracle `scala_aggregation_filter(root)` vs hook `plugin.aggregation_filter(root, path)`) |
| Default-`None` lock for non-planning languages | MET | `registry::tests::planning_hooks_default_to_unsupported_without_an_override ... ok` (MockPlugin, `planning: false`, no override) |
| Free-function exports intact (AR-19 oracles) | MET | `lang-cpp/src/lib.rs:33` and `lang-scala/src/lib.rs:37` re-export rows present; `pub fn` at lang-cpp planning.rs:14/24, lang-scala planning.rs:85/103 |
| cli hardwires UNEDITED | MET | `optimize.rs:102` `use ariadne_lang_cpp::{cpp_aggregation_filter, cpp_target_path_for};`; `ui.rs:395-396` positional pass — both read byte-as-is at HEAD |
| Planner / CppPlugin inherent M18.6 hooks UNEDITED | MET | commit hunks exclude `cluster/*` and plugin.rs:144-151; `git diff d937ac0 -- crates/cli crates/cluster` empty |
| `encoding_guard.py --all` | MET | no findings; exit 0 (`0` from `$LASTEXITCODE`) |
| `hygiene_check.py` — only the 2 server.rs AR-13 rows blocking | MET | only blocking rows: `crates/ui/src/server.rs: R2 1172 lines > 1000` and `crates/ui/src/server.rs:816: R1 fn \`api_graph_traversal\` is 161 lines (> 100)`; zero rows in AR-18 touched files; rest R3 warn-only |
| No behavior change; callers compile unchanged | MET | additive defaults; workspace test builds green for all touched crates + dependents built during integration suites |

## Refused commands (recorded; all named gates were still run)

- `git grep ...` (2 calls) and helper forms `echo "..."`-suffixed run,
  `python -c "..."`: refused with the catch-all deny, verbatim
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  Content searches went through the grep tool; exit codes came from
  `python tools/encoding_guard.py --all; $LASTEXITCODE` -> `0` and a full
  successful `hygiene_check.py` stdout.

## Explicitly NOT run

- `python tools/testrun.py --baseline` (workspace-wide) — not run: concurrent
  AR-11 agent holds working-tree edits to `crates/ui/assets/hotspots.js` and
  `crates/ui/tests/ui_test/main.rs`; milestone sizing law reserves the
  workspace baseline for a fresh/orchestrator session.
- `python -m unittest discover -s tools -p "*_tests.py"` — not run (same
  reservation; no tools/ files in the touch-set or working dirt).
- `python tools/ui_smoke.py`, `python tools/bench_compare.py` — not run (UI
  lane dirt in flight; neither is an AR-18 criterion).
- `python tools/checkpoint_guard.py status` ran (ground truth); `golden_guard.py`
  not applicable (no golden lock list in this handoff).
- `distill.py` workspace check not re-run by reviewer; the six scoped testruns
  above rebuilt every touched crate and dependent integration binary.

## Foreign dirt attributed (not AR-18 findings)

- Working-tree dirt: `crates/ui/tests/ui_test/main.rs` (+1), `crates/ui/assets/
  hotspots.js` (+14/-8), untracked `crates/ui/tests/ui_test/asset_injection_hardening.rs`
  — concurrent AR-11 lane.
- Build warnings from `ariadne-ui` lib during integration-suite runs
  (`server.rs:4` unused import `TraversalQuery`, `server.rs:984` unused `gen`)
  — same foreign lane.
