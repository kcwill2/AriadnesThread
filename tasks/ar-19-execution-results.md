# AR-19 Execution Results — registry-dispatched planning hooks (A1 part 2)

Reviewer-run fresh verification against `tasks/ar-19-handoff.md` acceptance
criteria. HEAD under review `0d7ecd3` (main commit `5de3884`), base seam
`d937ac0`. Every gate below is the reviewer's own run with own stdout;
canonical lines quoted verbatim.

## Acceptance criteria — measured

| Criterion (handoff) | Result | Evidence line (verbatim) |
|---|---|---|
| Gate 1: `-p ariadne-cli` (baseline 118 + yours) | MET — 117/0/8/6; 118 − 5 deleted plan.rs tests + 1 plan-lane witness + 3 registry seam locks = 117, rename map independently verified (old names have zero residue) | `117 passed / 0 failed / 8 ignored / 6 suites` |
| Gate 2: m00 golden + m00 facts-path + m18 parity + m24 identity UNEDITED | MET — no test/golden/fixture path appears in either commit; suites re-run green | `35 passed / 0 failed / 0 ignored / 4 suites` |
| Gate 3: new Scala plan-path suite green (both production lanes) | MET — `m24a_scala_plan_path_test`: optimize lane (collateral + >=1 safe plan + persisted agreement) and UI `/api/export` lane (non-empty proposals, `LegacyReport.scala` target, zero `_extracted.h`) | `2 passed / 0 failed / 0 ignored / 1 suites` (journal's combined 37/0/0/5 = 35+2 reconciles) |
| Gate 4: `-p ariadne-lang-cpp` (conditional on plan.rs removal — N/A, run anyway for the rider) | MET — count byte-equal to AR-18's pinned 153 at `d937ac0`: inherent deletion lost zero tests | `153 passed / 0 failed / 0 ignored / 2 suites` |
| Gate 5a: `encoding_guard --all` | MET — silent, exit 0 | `$LASTEXITCODE` -> `0` |
| Gate 5b: `hygiene_check` (only server.rs 2 rows) | MET — blocking rows exactly the 2 inherited AR-13 rows; new/edited files add no rows; the journal's transient `build_ui_payload` R1 is absent from final state | `crates/ui/src/server.rs: R2 1172 lines > 1000` + `api_graph_traversal is 161 lines` (exit 1 from these inherited rows; all else R3 warn-only) |
| TDD: RED first through production lanes, GREEN after switch | ACCEPTED with evidence caveat — journal RED `0 passed / 2 failed / 0 ignored / 1 suites` witnesses match the lock file's own panic templates; reviewer cannot re-execute RED (source frozen, mutation writes out of reviewer scope); structural argument: cpp-only dispatch cannot produce Scala-named targets or non-empty `.scala` proposals | journal RED/GREEN lines; lock read in full (265 lines) |
| Invariant: C++ `tool optimize` byte-identical (m00 parity authority) | MET — m00 goldens + facts-path parity UNEDITED green (gate 2); registry seam locks assert cpp free-function byte-identity through the trait | gate 2 line + registry.rs `planning_hooks_keep_cpp_hook_bytes_through_the_plugin_trait` green in gate 1 |
| Invariant: cpp hook symbols may leave cli's planning dependency graph; dep status journaled | MET — moved to `[dev-dependencies]`; production-shape compile proves plugin-free src: `cargo check -p ariadne-cli` (lib+bin only, dev-deps unlinked, cfg(test) excluded — the exact shape where E0433 would fire) | `ok (exit 0, 0 warnings emitted)` |
| Runtime plugin registration for the binaries | MET — `cli::default_registry()` delegates to `ariadne_pipeline::build_default_registry` (registers CppPlugin+ScalaPlugin, pipeline/registry.rs:25-26); m24a drives both lanes through the real `run_ui`/`EndToEndOptimizer`; D9 identity lock green | gate 1 + gate 3 lines |
| AR-18 verdict finding 3 (inherent-vs-trait reconciliation) | MET — journal records the determination (trait consumed, inherent dead, deleted per "remove or delegate"); independent proof: zero concrete dot-calls workspace-wide; trait override delegates (`plugin.rs:257-264`); lang-cpp count unchanged | gate 4 line; dot-call audit (11 hits, all `&dyn`/test/comment) |
| Objective 4: plan.rs 2-arm match deleted | MET — `PlanningHooks` struct, 2-arm closures, `language_of_path` removed from `commands/plan.rs`; single dispatch is `crate::registry::planning_hooks`; no retained dual representation | `git show 5de3884` hunks |
| Commit-vs-touch-set | MET within the contract's commit scope ("cli + lang-cpp + integration + journal"): `5de3884` = exactly 9 paths, `0d7ecd3` = journal only; 4 paths beyond the bullet list each named by another contract clause or compile-mandated (findings 1-2 in `ar-19-verdict.md`) | `git show` path lists |

## Explicitly NOT run

- `python tools/testrun.py --baseline`, `python -m unittest discover -s tools`,
  `python tools/ui_smoke.py`, `python tools/bench_compare.py` — withheld by
  dispatch instruction: the AR-11 UI WIP is dirty in this shared worktree
  (`crates/ui/assets/hotspots.js`, `crates/ui/tests/ui_test/main.rs`,
  untracked `asset_injection_hardening.rs` — attributed foreign dirt; zero
  `crates/ui` paths in AR-19's commits) and the sizing law reserves these
  for the orchestrator's fresh session.
- A cargo RELEASE-profile check — the plain (non-test-shape) check is the
  dependency-graph-complete proof for the dev-dep move; release profile adds
  no dep-graph evidence.
- Mutation re-execution of the m24a RED phase (would require source edits
  outside reviewer scope); RED accepted on journal evidence + structural
  impossibility argument + template-match of RED witnesses to test panics.

## Verdict

APPROVED. Findings 1-4 (all minor/observation level) and the five deviation
dispositions are recorded in `tasks/ar-19-verdict.md`.
