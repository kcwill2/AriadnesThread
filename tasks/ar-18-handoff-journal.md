# AR-18 Journal - Planning hooks onto LanguagePlugin (A1 part 1)

Implementer session 2026-10-10. Handoff: `tasks/ar-18-handoff.md`.
Multi-crate intentional (indivisible seam): `crates/lang`, `crates/lang-cpp`,
`crates/lang-scala`.

## What landed

- `crates/lang/src/registry.rs` - two new `LanguagePlugin` trait methods with
  default "planning unsupported" implementations:
  - `aggregation_filter(&self, repo_root: &Path, path: &Path) -> Option<bool>`
  - `target_path_for(&self, aggregation_path: &Path, symbol_name: &str) -> Option<PathBuf>`
  - Defaults return `None`. No generic trait methods; trait object stays
    dyn-compatible (witnessed by every parity test calling through
    `&dyn LanguagePlugin`).
- `crates/lang-cpp/src/plugin.rs` - `CppPlugin` overrides the hooks,
  delegating to `crate::planning::{cpp_aggregation_filter,
  cpp_target_path_for}`.
- `crates/lang-scala/src/plugin.rs` - `ScalaPlugin` overrides the hooks,
  delegating to `crate::planning::{scala_aggregation_filter,
  scala_target_path_for}`.
- Tests: default-`None` lock (lang), override-parity locks (cpp, scala).
- Free functions stay exported and untouched; cli/cluster/planner call sites
  UNEDITED (AR-19 consumes the seam later).

## Signature normalization (curried vs direct)

Verified real signatures at the consumers before designing:

- `cpp_aggregation_filter(path: &Path) -> bool` - direct, extension-only;
  consumed positionally as `&dyn Fn(&Path) -> bool + Sync` at
  `crates/cli/src/commands/ui.rs` (build_extraction_proposals, :395) and
  `crates/cli/src/optimize.rs` (:102 ff).
- `cpp_target_path_for(aggregation_path: &Path, symbol_name: &str) -> PathBuf`
  - direct; planner contract `&dyn Fn(&Path, &str) -> PathBuf + Sync`
  (`crates/cluster/src/planner.rs:82`).
- `scala_aggregation_filter(repo_root: &Path) -> impl Fn(&Path) -> bool + '_`
  - CURRIED: the repo root is closure-captured because the predicate reads
  `repo_root.join(path)` content (a depth-0 `package object` declaration).
- `scala_target_path_for(aggregation_path: &Path, symbol_name: &str) ->
  PathBuf` - direct; purely lexical (no repository access).

Normalization decision: `repo_root` is a per-call PARAMETER on the trait's
`aggregation_filter`, not a closure capture. That is the only shape both
languages fit without adapters - the C++ override ignores the parameter
(its rule is extension-only), the Scala override forwards it into the
curried free function (`scala_aggregation_filter(repo_root)(path)`), so the
behavior is byte-identical to today's CLI closure construction.
`target_path_for` mirrors C++'s real arity `(aggregation_path, symbol_name)`
as the handoff directed; Scala's naming is lexical, so it needs no root
either.

## Default-shape decision: Option, not an error variant

`None` = "planning unsupported". Chosen per the trait's existing style:
`shaper()` already defaults to `Option<&dyn UnitShaper> { None }` with the
same "unsupported unless overridden" meaning, and the trait carries no error
enum to grow. An explicit error variant would introduce a new public error
type into `ariadne-lang` for a signal that is already advisory elsewhere -
`capabilities().planning` remains the stage-gate bit, so the hook's `None`
is a dispatch fact, not a failure. Locked by
`planning_hooks_default_to_unsupported_without_an_override` (registry.rs,
MockPlugin, which declares `planning: false` and never overrides).

## TDD evidence (digest lines verbatim)

Compile discipline (trait + defaults applied as one atomic edit BEFORE any
test run, per the UI-lane serialization rule):

- `python tools/distill.py -- cargo check --workspace` ->
  `ok (exit 0, 0 warnings emitted)`

Red (post-trait-edit; assertion-red only, never compile-red):

- `python tools/testrun.py --distill -- -p ariadne-lang` ->
  `35 passed / 0 failed / 1 ignored / 2 suites` (default lock green on
  arrival - it locks the default shape, which shipped with the seam edit)
- `python tools/testrun.py --distill -- -p ariadne-lang-cpp` ->
  `152 passed / 1 failed / 0 ignored / 1 suites` - failed test
  `planning_hooks_override_the_defaults_with_free_function_parity`,
  `assertion left == right failed: aggregation-filter parity for a.h;
  left: None` - the predicted missing-override reason (default `None` vs
  `Some(cpp_aggregation_filter(...))`).
- `python tools/testrun.py --distill -- -p ariadne-lang-scala` ->
  `220 passed / 1 failed / 0 ignored / 1 suites` - same test name,
  `left: None` on the `all.scala` parity case; same predicted reason.

Green (after overrides):

- `-p ariadne-lang-cpp` -> `153 passed / 0 failed / 0 ignored / 2 suites`
- `-p ariadne-lang-scala` -> `221 passed / 0 failed / 0 ignored / 2 suites`
- `-p ariadne-lang` -> green per the run above (file unchanged since; the
  run preceded only the other two crates' overrides).

Parity evidence: both override locks compare hook output against the free
function on identical inputs - cpp: full header-extension accept/reject set
plus no-extension and uppercase `.H`, target cases incl. `std::vector`,
`Thing<T>`, `A::B`, `:::`; scala: package-object / plain-class / missing-file
filter corpus (incl. the unreadable-path case) and target cases incl. the
qualified `com.example.models.Order`, `Foo$`, `Thing[T]`, `:::`. The Scala
lock builds the free-function oracle as
`scala_aggregation_filter(root)` and the hook as
`plugin.aggregation_filter(root, path)` - the parity assertion IS the
curried-vs-direct normalization lock.

## Named integration suites (UNEDITED green)

- `-p ariadne-integration-tests --test m18_cpp_plugin_parity_test` ->
  `23 passed / 0 failed / 0 ignored / 1 suites`
- `-p ariadne-integration-tests --test m24_language_identity_test` ->
  `8 passed / 0 failed / 0 ignored / 1 suites`
- Invariant "m17 seam guard green" additionally verified:
  `--test m17_plugin_seam_test` -> `6 passed / 0 failed / 0 ignored / 1 suites`

## Hygiene / encoding

- `python tools/encoding_guard.py --all` -> clean (no findings). Staged-run
  form not separately invocable: `commit_scoped.py` stages+commits
  atomically, and `--all` covers the same bytes.
- `python tools/hygiene_check.py` -> blocking rows are ONLY the 2 pre-existing
  `crates/ui/src/server.rs` AR-13 rows (R2 1172 lines; R1
  `api_graph_traversal` 161 lines). Zero new R1/R2/R3 rows in any touched
  file; all three touched files stay far under the 1000-line ceiling and
  every new function is a 1-3 line delegation or one test.
- All authored content pure ASCII; no hex literals introduced (no UI files
  touched).

## Deliberately NOT done

- No edits to `crates/cli` (the A1 hardwire at `optimize.rs:102` and the
  `ui.rs` positional pass stay exactly as-is; AR-19 switches consumers to
  the seam and removes the free-function consumers later).
- Free functions `cpp_aggregation_filter`, `cpp_target_path_for`,
  `scala_aggregation_filter`, `scala_target_path_for` remain exported -
  the parity locks call them as the oracle; deleting them is AR-19 scope.
- `CppPlugin`'s inherent M18.6 hook methods (`aggregation_filter(&self,
  path)`, `target_path_for(&self, aggregation_path, symbol_name)`) kept
  byte-identical: they are pre-existing public API. Note the Rust name
  resolution nuance: dot-calls on the CONCRETE type still resolve to the
  inherent methods (arity 1 / arity 2 returning `bool`/`PathBuf`); the trait
  hooks are reached through `&dyn LanguagePlugin` or qualified syntax, which
  is exactly how AR-19's registry dispatch will call them. The parity tests
  call via `&dyn LanguagePlugin` to exercise the dispatch that matters.
- No workspace-wide gates run (ui/cli untouched lanes under a concurrent
  agent); only the three scoped crate runs and the three named integration
  binaries, per the handoff's gate list.

## Deviations

- None from the touch-set or gates. One addition: ran `m17_plugin_seam_test`
  because the handoff's invariants row names the m17 seam guard green (cheap,
  same package, read-only).
