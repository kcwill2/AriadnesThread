# AR-18 Verdict — Planning hooks on LanguagePlugin (A1 part 1)

Reviewer gate per `.kilo/skills/review-gate`. Handoff `tasks/ar-18-handoff.md`,
journal `tasks/ar-18-handoff-journal.md`, commit `d937ac0` (parent `55c93c8`).

## Verdict: APPROVED

## Gate lines (verbatim, reviewer's own fresh runs)

| # | Command | Canonical line |
|---|---------|----------------|
| 1 | `python tools/testrun.py -- -p ariadne-lang` | `35 passed / 0 failed / 1 ignored / 2 suites` |
| 2 | `python tools/testrun.py -- -p ariadne-lang-cpp` | `153 passed / 0 failed / 0 ignored / 2 suites` |
| 3 | `python tools/testrun.py -- -p ariadne-lang-scala` | `221 passed / 0 failed / 0 ignored / 2 suites` |
| 4 | `... --test m18_cpp_plugin_parity_test` | `23 passed / 0 failed / 0 ignored / 1 suites` |
| 5 | `... --test m24_language_identity_test` | `8 passed / 0 failed / 0 ignored / 1 suites` |
| 6 | `... --test m17_plugin_seam_test` | `6 passed / 0 failed / 0 ignored / 1 suites` |

All six match the reviewer's pinned expectations. `python tools/encoding_guard.py
--all` -> no findings, exit 0. `python tools/hygiene_check.py` -> blocking rows
ONLY the 2 pre-existing `crates/ui/src/server.rs` AR-13 rows (`R2 1172 lines`;
`R1 fn api_graph_traversal is 161 lines`); zero rows in any AR-18 touched file;
remaining output is R3 warn-only and pre-existing repo-wide.

## Checks (numbered per review request)

1. **Touch-set EXACT.** `git diff --name-only 55c93c8..d937ac0` returns exactly
   the 4 paths: `crates/lang/src/registry.rs`, `crates/lang-cpp/src/plugin.rs`,
   `crates/lang-scala/src/plugin.rs`, `tasks/ar-18-handoff-journal.md`. No
   path outside the declared set.
2. **Seam shape.** `aggregation_filter(&self, _repo_root: &Path, _path: &Path)
   -> Option<bool>` and `target_path_for(&self, _aggregation_path: &Path,
   _symbol_name: &str) -> Option<PathBuf>`, both defaulting to `None` —
   matching the trait's `shaper() -> Option<&dyn UnitShaper> { None }`
   default-`None` idiom (registry.rs:43). No generic methods; trait object
   stays dyn-compatible (compiled `&dyn` storage + explicit dyn witnesses).
   Advisory-gate semantics read and UNCHANGED: `cli/src/commands/plan.rs:30`
   (`if plugin.capabilities().planning { continue; }` against `PlanningGap`
   reporting) and `cli/src/registry.rs:302` untouched by the commit; the trait
   doc states `capabilities().planning` stays the stage-gate bit.
3. **Normalization claim verified independently.** `cpp_aggregation_filter(path:
   &Path) -> bool` (lang-cpp/src/planning.rs:14) is extension-only — takes no
   root at all, so the override's ignored `_repo_root` is genuine, not masked.
   Scala override is `Some(crate::planning::scala_aggregation_filter(repo_root)(path))`
   — closure constructed per call inside the hook from the per-call parameter;
   NO registration-time capture anywhere in plugin.rs.
4. **Parity locks exercise the real dispatch.** Both cpp and scala locks bind
   `let as_plugin: &dyn LanguagePlugin = &plugin;` and `assert_eq!` hook output
   against the FREE functions on identical inputs (`Some(cpp_aggregation_filter(
   path))` etc.); the scala filter oracle is `scala_aggregation_filter(root)`
   built once and compared to `plugin.aggregation_filter(root, path)` — the
   assertion IS the curried-vs-direct normalization lock, not a re-implemented
   expectation. Scala lexical naming parity locked incl. `com.example.models.Order`,
   `Foo$`, `Thing[T]`, `:::`; cpp incl. `std::vector`, `Thing<T>`, `A::B`, `:::`,
   uppercase `.H`, `noext`.
5. **Untouched-claims verified.** Free fns `pub fn` at lang-cpp planning.rs:14/24
   and lang-scala planning.rs:85/103, crate-root re-exports intact
   (`lang-cpp/src/lib.rs:33`, `lang-scala/src/lib.rs:37`). cli hardwires read
   byte-as-is at HEAD: `optimize.rs:102` (`use ariadne_lang_cpp::{cpp_aggregation_filter,
   cpp_target_path_for};`) and `ui.rs:395-396` (positional pass into planner).
   Planner untouched (not in touch-set, no working-tree diff). CppPlugin inherent
   M18.6 hooks (plugin.rs:144-151) byte-identical — outside the commit's hunks.
6. **Default-`None` locked.** `planning_hooks_default_to_unsupported_without_an_override`
   (registry.rs:633) exercises the trait's own defaults via `MockPlugin`, which
   declares `planning: false` (registry.rs:448) and never overrides; green in
   the fresh run.

## Findings

1. **(minor, non-blocking)** Journal TDD evidence cites `testrun.py --distill`
   digest lines but no `target/testrun-capture.txt` capture paths per run; the
   distillation contract (`docs/specs/tool-output-distillation.md`) expects the
   path alongside the digest. Canonical lines are present so verdict unaffected.
2. **(observation, accepted)** The constraint line "commit_scoped with one scope
   per crate" landed as a single `d937ac0` commit spanning three crates + journal.
   The enforceable intent — no foreign-lane sweep in the shared worktree — holds
   exactly (touch-set 4/4). `commit_scoped.py` usage is not reconstructable
   post-hoc; accepted.
3. **(follow-up disposition — AR-19)** CppPlugin retains inherent
   `aggregation_filter`/`target_path_for` (plugin.rs:144/150) alongside the new
   trait overrides: a coexistence the handoff mandates (untouched pre-existing
   public API) and the journal discloses, including the dot-call resolution
   nuance on concrete types. AR-19 MUST reconcile (remove or make inherent
   hooks delegate to the trait) when it switches consumers, or concrete-type
   call sites will silently bypass the seam. Goes to the ledger's deferred
   findings; recorded here, not dropped.

## Deviation dispositions

- Declared addition (`m17_plugin_seam_test` run): **accepted** — the handoff's
  own invariants row names "m17 seam guard green"; the run discharges an
  invariant, it is barely a deviation at all.
- Declared non-invocability of `encoding_guard.py --staged` separate from
  `commit_scoped.py`: **accepted** — `--all` (exit 0) covers the same bytes.

## Refused commands (configuration record, not gate failures)

`git grep` (both calls) and two exit-code helper forms (`echo`, `python -c`)
were refused by the catch-all deny rule, verbatim: `{"permission":"bash",
"pattern":"*","action":"deny","source":"global"}`. No expected gate tool was
lost: all six testrun gates, `encoding_guard.py --all` (exit 0 confirmed via
`python tools/encoding_guard.py --all; $LASTEXITCODE`), and a full successful
`hygiene_check.py` run were executed with own stdout.

## Workspace-wide gates

Not run here by instruction: concurrent AR-11 lane edits `crates/ui/assets` +
`crates/ui/tests/ui_test/main.rs` (foreign dirt, attributed), AR-22 dispatching
mid-review, and the milestone sizing law reserves `--baseline`, full `unittest
discover`, `ui_smoke`, `bench_compare` for a fresh/orchestrator session. The
`ariadne-ui` lib warnings seen during integration-suite builds
(`server.rs:4` unused import, `server.rs:984` unused variable) are foreign-
lane dirt, not AR-18 findings.
