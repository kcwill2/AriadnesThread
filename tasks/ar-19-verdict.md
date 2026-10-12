# AR-19 Verdict — Registry-dispatched planning hooks; scala plan path correct (A1 part 2)

Reviewer gate per `.kilo/skills/review-gate`. Handoff `tasks/ar-19-handoff.md`,
journal `tasks/ar-19-handoff-journal.md` (+ addendum). Commits: `5de3884`
(nine paths, main), `0d7ecd3` (journal addendum). AR-18 seam base `d937ac0`;
intervening ancestors `bcc08fc`/`15ffbfd` are AR-22 landings, not AR-19 scope.

## Verdict: APPROVED

## Ground truth (checkpoint_guard)

HEAD `0d7ecd3`, branch `shining-enthusiasm`. Tree dirt attributed to foreign
lanes: `crates/ui/tests/ui_test/main.rs`, `crates/ui/assets/hotspots.js`,
untracked `asset_injection_hardening.rs` (AR-11 UI WIP); `AGENTS.md`,
`tasks/milestone_ledger.md`, `tasks/ledger_archive.md` (orchestrator/docs).
Zero dirt in any AR-19 path. `ui_smoke`/`--baseline` NOT run by instruction
(UI lane dirty); no AR-19 commit touches `crates/ui`.

## Gate lines (verbatim, reviewer's own fresh runs, exit 0 each)

| # | Command | Canonical line |
|---|---------|----------------|
| 1 | `python tools/testrun.py --distill -- -p ariadne-cli` | `117 passed / 0 failed / 8 ignored / 6 suites` |
| 2 | `... -- -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test --test m18_cpp_plugin_parity_test --test m24_language_identity_test` | `35 passed / 0 failed / 0 ignored / 4 suites` |
| 3 | `... -- -p ariadne-integration-tests --test m24a_scala_plan_path_test` | `2 passed / 0 failed / 0 ignored / 1 suites` |
| 4 | `python tools/testrun.py --distill -- -p ariadne-lang-cpp` | `153 passed / 0 failed / 0 ignored / 2 suites` |
| 5 | `python tools/encoding_guard.py --all` | silent, exit 0 (explicit `$LASTEXITCODE` -> `0`) |
| 6 | `python tools/hygiene_check.py` | blocking rows ONLY the 2 inherited AR-13 rows (`crates/ui/src/server.rs` R2 1172; `api_graph_traversal` R1 161); everything else warn-only R3; exit 1 from the inherited rows |
| 7 | `python tools/distill.py -- cargo check -p ariadne-cli` | `ok (exit 0, 0 warnings emitted)` — production-shape (lib+bin, dev-deps unlinked) |

Arithmetic: gate 3 + gate 2 = `37 / 0 / 0 / 5` — the journal's combined
gate-3 line reconciles. Gate 1 reconciles the journal's 118/0/8/6 baseline:
5 plan.rs tests deleted, 1 plan-lane witness + 3 registry seam locks added:
118 − 5 + 1 + 3 = 117 (verified against the rename map below, not assumed).

## Checks (numbered per review request)

1. **Touch-set.** `5de3884` = EXACTLY 9 paths: `crates/cli/Cargo.toml`,
   `crates/cli/src/commands/plan.rs`, `crates/cli/src/commands/ui.rs`,
   `crates/cli/src/commands/ui/builder_tests.rs`, `crates/cli/src/optimize.rs`,
   `crates/cli/src/registry.rs`, `crates/lang-cpp/src/plugin.rs`,
   `tasks/ar-19-handoff-journal.md`,
   `tests/integration/tests/m24a_scala_plan_path_test.rs`. `0d7ecd3` =
   journal only. All 9 land inside the contract's own commit scope line
   ("cli + lang-cpp + integration + journal"). Four sit beyond the bullet
   list; each is named elsewhere or forced — see findings 1–2 and the
   dispositions. D1 read independently: the `builder_tests.rs` hunk adds
   only the two mandated args (`&registry`, `Path::new(".")`) to
   `test_planner_wiring_on_empty_inputs_yields_no_proposals`; the assertion
   (`out.is_empty()`) is untouched — test semantics unchanged.
2. **Hardwires gone.** `optimize.rs` has no
   `use ariadne_lang_cpp::...planning` import (diff removes it; workspace
   grep confirms). `build_extraction_proposals` and `plan_safe_extractions`
   feed `hooks.aggregation`/`hooks.target` from
   `crate::registry::planning_hooks` (trait dispatch). Workspace grep of
   `\.aggregation_filter(` / `\.target_path_for(`: 11 hits, ALL trait
   receivers — registry seam (2), optimize per-file dispatch on
   `plugin_for_catalog_file`'s `&dyn` (1), lang MockPlugin parity locks (2),
   cpp/scala `as_plugin` parity locks (4), comment text (2). Zero concrete
   dot-calls. Workspace grep of the four free hook functions: production
   hits only inside `lang-cpp`/`lang-scala` (definitions + trait
   delegation); everything else is test oracles (planner characterization,
   UNEDITED m00/m18 suites, `payload_tail_bench.rs`), m24 text guards, and
   prose. No seam bypass survives.
3. **Inherent deletion safety.** Independently proven: (a) current dot-call
   audit above shows every caller is a `&dyn LanguagePlugin` receiver —
   none could have resolved to the inherent pair (different arity too:
   inherent `aggregation_filter(&self, path)` vs trait `(repo_root, path)`);
   (b) the trait override EXISTS and delegates —
   `plugin.rs:257-264` returns `Some(crate::planning::cpp_aggregation_filter(path))`
   / `Some(crate::planning::cpp_target_path_for(...))` — so the C++ answer
   survives deletion; (c) `-p ariadne-lang-cpp` = 153/0/0/2, byte-equal to
   the AR-18 verdict's pinned count at `d937ac0` — zero tests lost (the
   journal's HEAD-vs-worktree grep is a weaker claim; this closes it
   independently). AR-18 verdict finding 3 sanctioned "remove or make
   inherent hooks delegate"; the journal's
   "Rider (AR-18 verdict): hook coexistence resolution" section records the
   determination — trait pair is the consumer, inherent pair dead, deleted.
   **Reconcile-by-deletion: accepted.**
4. **Cargo dev-dep move is production-safe.** `cargo check -p ariadne-cli`
   without `--all-targets` compiles lib+bin with cfg(test) code excluded and
   dev-deps NOT linked — it passed `ok (exit 0, 0 warnings)`, so no
   `#[cfg(not(test))]`-reachable production path references lang-cpp or
   lang-scala (that shape is exactly where E0433 would fire). All
   `ariadne_lang_cpp`/`ariadne_lang_scala` references under `crates/cli`
   verified to sit inside `#[cfg(test)]` mods (registry.rs 556-616,
   plan.rs 320-331), the comment at `ui.rs:395`, or the test target
   `tests/payload_tail_bench.rs`. `Cargo.lock` ariadne-cli entry consistent
   (no lock dirt). Runtime registration intact: `cli::registry::
   default_registry()` delegates to `ariadne_pipeline::build_default_registry`
   (registry.rs:16-17), which registers `CppPlugin` + `ScalaPlugin`
   (`crates/pipeline/src/registry.rs:25-26`); m24a passes through the real
   `run_ui` and `EndToEndOptimizer` binary lanes, and the D9
   `registries_enumerate_identical_languages` lock is green in gate 1.
5. **planning.rs alive-not-dead; plan-test relocation faithful.** Handoff
   item 3 names `crates/lang-cpp/src/plan.rs`, a file that never existed
   (finding 2). The real `planning.rs` IS alive: consumed by the trait-hook
   delegation (`plugin.rs:258/264`) and the `lib.rs:33` re-export used as
   the byte-identity oracle by cli tests, `payload_tail_bench.rs`, and the
   UNEDITED m00 facts-path + m18 suites — all green with the file kept.
   Deletion would have broken the handoff's own locked authorities. Relocation
   audit, old→new (names from `git show 5de3884` deletions vs additions;
   grep proves zero residue of old names):
   - `a_third_language_no_longer_silently_receives_the_cpp_planning_hooks`
     + `planning_hooks_give_an_unclaimed_path_no_cpp_hooks` ->
     `planning_hooks_answer_honestly_for_paths_no_plugin_claims`
     (same `solver.f90` third-language path + `unclaimed.rs`, same
     assertions incl. the explicit `assert_ne!` vs `cpp_target_path_for`).
   - `planning_hooks_route_scala_by_language_and_keep_cpp_hook_bytes` ->
     split into `planning_hooks_keep_cpp_hook_bytes_through_the_plugin_trait`
     + `planning_hooks_route_scala_through_the_plugin_trait` + re-expressed
     through the plan lane as
     `plan_lane_hooks_are_the_registry_dispatch_cpp_and_scala_stay_correct`
     (all five original assertions present verbatim or stronger: real
     fixture, real `default_registry`, not test injection).
   - the two `language_of_path` locks died with their function; the
     registry-authority they locked (label never dispatches) is now
     STRUCTURAL at the seam (`plugin_for(path)` — no label map exists) and
     externally locked by the existing `catalog_*` authority test
     (registry.rs ~474), the D9 enumeration lock, and the m24
     forbidden-cpp-fallback text guard (green in gate 2). No coverage hole.
6. **m24a lock is real.** Read in full (265 lines). Lane (a): production
   `EndToEndOptimizer::optimize_repository` on a temp copy of
   `tests/fixtures/scala-isolation` (fixture verified on disk incl.
   `legacy.scala`) asserts `legacy.scala` collateral with >0 symbols,
   `safe_plans_available >= 1`, AND persisted-report agreement. Lane (b):
   real `run_ui` server on an ephemeral port, `/api/export` payload asserted
   NON-empty, target base-name `legacy.scala`, `target_files` contains
   `src/main/scala/com/example/legacy/LegacyReport.scala` (Scala namer),
   ZERO `_extracted.h` rows (cpp namer must not appear), symbol list
   non-empty. Mutation sanity: the journal's RED `0 passed / 2 failed / 0
   ignored / 1 suites` witnesses (`Opportunities seen: []`; payload keys
   without `extraction_proposals`) are the exact panic templates in this
   file, and a cpp-only dispatch structurally cannot emit Scala-shaped
   targets — a revert cannot pass. RED was frozen BEFORE the swap per the
   journal's TDD ordering (lock-only RED, then switch, then GREEN).

## Plan-literal checks

Gate expectations from the task/handoff, verbatim vs measured: 117/0/8/6 ✓,
35/0/0/4 ✓, 153/0/0/2 ✓, hygiene "only server.rs 2 rows" ✓, encoding silent
✓. Transient R1 claim on `build_ui_payload`: final state shows NO R1 row for
it (hoisted registry + compaction under the 100-line ceiling) — final state
verified clean.

## Findings

1. **(minor, accepted)** `crates/cli/src/commands/ui/builder_tests.rs` is not
   in the handoff's bullet touch-set, yet the handoff hard-mandates the
   `build_extraction_proposals` signature change (objective 2) plus strict
   compile-green (Constraints), leaving no compiling alternative. Disposition:
   **accepted** (D1) — semantics unchanged per the diff read; the durable fix
   is authoring-side: handoffs mandating a signature must pre-list call-site
   files or a glob. Goes to ledger authoring note, not to the implementer.
2. **(minor, handoff authoring defect)** Objective 3 names
   `crates/lang-cpp/src/plan.rs`, a file that never existed, and gate 4 is
   conditional on that phantom. Journal D3 caught and corrected it with the
   correct ALIVE determination. Not implementer fault; record for
   handoff-authoring hygiene.
3. **(minor, non-blocking)** Report-evidence: the journal's TDD/gate lines
   are digest-shaped (canonical lines, zero raw dumps) but declare "own
   stdout" with no `target/testrun-capture.txt` capture paths per run, which
   the tool-output distillation contract expects alongside digests
   (same class as AR-18 verdict finding 1). Verdict unaffected.
4. **(observation)** Journal gate 4's byte-equality claim compares HEAD vs
   worktree (clean-tree tautology, not the pre-deletion claim). Closed
   independently by reviewer gate 4 matching AR-18's pinned 153 at `d937ac0`.

## Deviation dispositions

- **D1** (`builder_tests.rs` call-site): **accepted** — forced by the
  mandated signature change; semantics unchanged (checked in the diff).
- **D2** (5 deleted / 4 added test relocations): **accepted** — full
  old→new assertion mapping verified above; arithmetic reconciles against
  the 118 baseline without trusting the journal.
- **D3** (plan.rs naming): **accepted** — file never existed; the real
  `planning.rs` determination (ALIVE, kept, oracle roles enumerated) is
  correct and independently confirmed.
- **D4** (one shared `default_registry()` in `build_ui_payload`): **accepted**
  — pure per-call constructor, identical value; m00 goldens UNEDITED green;
  fn now under the R1 ceiling (no hygiene row).
- **D5** (collateral filter moved from post-aggregation cpp check to
  per-file plugin dispatch): **accepted** — `cpp_aggregation_filter` is
  extension-only (AR-18 verdict check 3), and a file name and its path share
  the extension, so C++ accept/reject per key is identical; byte authorities
  (m00, m18, m24) green UNEDITED plus the named cli unit lock
  `test_collateral_opportunities_skip_symbols_without_defining_file`
  (optimize.rs:567) green.

## Refused commands (configuration record, not gate failures)

Two bash invocations refused by the catch-all deny rule, verbatim:
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}` —
(1) a combined `git show --stat ...; echo ...; git log ...` call (per-commit
review instead executed via the `git_git_show`/`git_git_log` tools with full
diffs), (2) `python -c "import tomllib..."` for the integration package name
(replaced by reading `tests/integration/Cargo.toml`). No expected gate tool
was lost: all five required gates plus the extra production-shape
`cargo check` ran with own stdout.

## Workspace-wide gates

NOT run by instruction and not claimed: `--baseline`, full `unittest
discover`, `ui_smoke`, `bench_compare` — the AR-11 UI WIP is dirty in this
shared tree (`crates/ui/assets/hotspots.js`, `crates/ui/tests/ui_test/main.rs`
modified; `asset_injection_hardening.rs` untracked); none of it is AR-19
scope (zero `crates/ui` paths in either commit). Reserved for the
orchestrator's fresh session per the sizing law.
