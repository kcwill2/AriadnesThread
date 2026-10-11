# AR-20 Journal - BLOCKED: handoff infeasible as scoped (zero code landed)

Disposition: the AR-20 move (WorkerBackend + TuTask verbatim into lang-api,
`lang-cpp -> scheduler` Cargo dep removed, guard assert GREEN) cannot compile.
No production file was changed and the dep was NOT removed; the handoff's
compile-green premise fails on a CODE dependency from lang-cpp into
scheduler's analysis engine.
Per the implementer contract ("if the handoff is wrong, report it and stop;
do not improvise a fix") this journal + a board ASK are the deliverables.
Session anchor: HEAD 0d7ecd3 at start (AR-19 chain landed); HEAD adb54ea
(AR-11) at journal commit time.

## Placement choice (answered negatively - no placement exists as scoped)

### WorkerBackend (crates/scheduler/src/worker.rs:50-52)

- Signature: `fn analyze_task(&self, task: &TuTask, repo_root: &Path) ->
  anyhow::Result<ariadne_index::TuAnalysis>`.
- `TuAnalysis` is the same type lang-api owns (index re-export; locked by the
  m17 seam test), so the trait's analysis type is fine.
- The trait needs `anyhow` on lang-api.
  lang-api/Cargo.toml currently has only `ariadne-repo` + `serde`, and
  lang-api's manifest is OUTSIDE the hard touch-set (`crates/lang-api/src/**`
  is src-only; lang-cpp's Cargo.toml is named explicitly, lang-api's is not).

### TuTask (crates/scheduler/src/queue.rs:8-47)

- Fields: `task_id: usize`, `tu: ariadne_build::TranslationUnit`,
  `estimated_cost: usize`, `context: Option<SharedTuStoreContext>`.
- A verbatim move additionally requires lang-api to name `ariadne-build`
  (same out-of-touch-set manifest problem).
- `SharedTuStoreContext` = `Arc<TuStoreContext>` is scheduler-owned
  (crates/scheduler/src/tu_store.rs:71-99).
  Moving TuTask verbatim forces moving TuStoreContext too.
  Its private fields `hash_by_id`/`path_by_id` are read by scheduler's own
  closure engine (tu_store.rs:119, :296, :302), so the move would change
  field visibility - beyond "MOVED VERBATIM" and beyond "re-export shim
  only - call sites UNTOUCHED".

### Hard blocker - lang-cpp -> scheduler is a code dep, not just a type dep

- crates/lang-cpp/src/backend.rs:180: the PRODUCTION `inprocess_cpp_fallback`
  calls `ariadne_scheduler::analyze_tu_full_with_plugin(&task.tu, repo_root,
  ctx, &CppPlugin::new())` whenever the task carries a store context.
- That engine (scheduler/src/tu_store.rs, module-wide) imports
  `ariadne_index::{TuAnalysis, TuCacheFingerprint, CURRENT_ANALYSIS_VERSION}`
  plus ariadne-lang resolver traits.
  lang-api cannot depend on ariadne-index: index re-exports lang-api types
  (m17 seam test `lang_api_types_are_the_same_types_index_re_exports`), so
  the edge would cycle.
  The closure engine therefore CANNOT relocate to lang-api.
- Byte-parity of this path is locked by `m18_cpp_plugin_parity_test`; stubbing
  or bypassing the merge changes locked behavior and is out of contract.
- Consequence: removing the Cargo dep makes `cargo check` fail on backend.rs
  (E0433/E0432 on `ariadne_scheduler`), so "import repoint + dep removal in
  ONE commit, cargo check --workspace green" is unsatisfiable, and the new
  guard assert would be permanently RED.
- The lang-cpp manifest note (crates/lang-cpp/Cargo.toml:11-15) states the
  dependency is needed "because the backend implements scheduler::WorkerBackend
  over scheduler::TuTask" - that note is INCOMPLETE: the tu_store call is the
  load-bearing edge.

## TDD evidence (RED captured, GREEN unreachable, extension reverted)

- Guard located: tests/integration/tests/m17_plugin_seam_test.rs (path as
  handoff required; existing direct-dep guard
  `plugin_crates_do_not_depend_on_core_pipeline`).
- Extension added: `plugin_crates_transitive_tree_excludes_core_pipeline` -
  walks the local path-dependency closure over `[dependencies]` /
  `[build-dependencies]` tables (dev edges skipped; cycle-safe visited set),
  asserting none of scheduler/report/cluster/graph/transform/validate is
  reachable from lang-cpp or lang-scala.
  Mechanism mirrors `cargo tree` restricted to compiled edges.
- RED run (own stdout; capture path target/testrun-capture.txt, quoted from
  stdout per the shared-worktree capture-race rule):

  `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test`
  `6 passed / 1 failed / 0 ignored / 1 suites`
  failure: `crates/lang-cpp transitively compiles against core-pipeline
  crates ["ariadne-scheduler", "ariadne-report", "ariadne-cluster",
  "ariadne-graph", "ariadne-transform", "ariadne-validate"] (plan rule R1
  must hold transitively)`

- The guard mechanism is thus machine-proven to work.
  GREEN is unreachable as scoped (see blocker), so the extension was REVERTED
  to keep the shared worktree green for lanes AR-11/AR-23:
  `git status --porcelain -- tests/integration/tests/m17_plugin_seam_test.rs`
  is EMPTY (byte-identical restore verified; the Edit round-trip had left one
  LF-only line under core.autocrlf=true - restored to pristine checkout bytes
  via a byte-level write under target/, deleted after use).

## cargo tree evidence (pre-move baseline; the new assert locks its removal)

- `python tools/distill.py -- cargo tree -p ariadne-lang-cpp` -> ok, capture
  target/distill-capture.txt (overwritten by later distill runs - lines
  quoted from own stdout):
  top-level edge `ariadne-scheduler` (line 50); under it `ariadne-graph`
  (53) and `ariadne-report` (74); under report: `ariadne-cluster` (77),
  `ariadne-transform` (97), `ariadne-validate` (114).
  All six forbidden crates are in lang-cpp's compiled tree today.

## Gates status

- Gates 1-4 not run: zero code change; baselines preserved (lang-cpp
  153/0/0/2 and scheduler 34/0/0/2 stand from the AR-18/AR-19 landings;
  m17 guard 6/0/0/1 green as of this session's RED run's 6-passed sibling).
- `encoding_guard --all` and `hygiene_check.py` run for this journal commit
  (results in the board ASK; expected unchanged: encoding silent, hygiene =
  the 2 pre-existing server.rs rows per the handoff's pre-AR-13 state).

## Deliberately NOT done

- No type relocation, no Cargo dep removal, no partial commit of the RED
  guard (a red suite in a shared worktree breaks every other lane's census).
- No dependency-inversion redesign of TuStoreContext/backend.rs - that is
  architecture work for the orchestrator, not an implementer improvisation.
- No touch of crates/ui (AR-11 lane), crates/cli (AR-23 lane), fixtures, or
  golden files.

## Options for the orchestrator (pick one; AR-20 then re-dispatchable)

- A (recommended): predecessor seam handoff - make the closure-merge reachable
  through a lang-api-owned abstraction (TuStoreContext + an opaque merger
  handle living in lang-api; scheduler injects its engine at backend/registry
  construction; backend.rs:180 invokes the handle).
  Then the verbatim move + dep removal + GREEN guard all hold, and the
  A4 invariant stays intact.
- B: weaken the invariant lock (guard forbids report/cluster/transform/
  validate transitively, scheduler edge explicitly exempted for lang-cpp).
  Cheapest but abandons part of A4's claim - needs an explicit ruling.
- C: extract scheduler's tu_store engine into its own neutral crate - large
  surgery; likely violates handoff sizing.

## Deviations

- None from the touch-set: the ONLY landed file is this journal.
- Journal committed alone via commit_scoped; commit hash is reported on the
  board (journal cannot contain its own hash).
