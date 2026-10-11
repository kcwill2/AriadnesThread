# AR-20 Shape - worker-core extraction (C-lite), additive-then-switch

> **Recommendation (verbatim ruling):** Adopt C-lite: extract the worker
> vocabulary - the `tu_store` closure-merge engine, `TuTask`, and the
> `WorkerBackend` trait (plus its `Arc` blanket impl) - into a new neutral
> crate `ariadne-worker-core` (`crates/worker-core`), depending only on
> `repo`/`build`/`index`/`lang` + `anyhow`/`serde`/`serde_json`. `scheduler`
> keeps scheduling (queue, pool, dispatch backends, incremental, engine) and
> re-exports every moved name so all external consumers compile UNEDITED.
> `lang-cpp` then repoints three references at worker-core and drops its
> `ariadne-scheduler` dependency; the m17 transitive guard lands in that same
> commit, GREEN - never in history as RED. Split into AR-20a (additive move +
> shims, all suites green, guard untouched) and AR-20b (switch + dep removal +
> guard), the AR-18/AR-19 pattern. Option B is rejected on a machine fact, not
> taste: `scheduler` deps `ariadne-graph` directly, so exempting the scheduler
> edge still leaks `graph` into the plugin cone even after AR-26. Option A is
> feasible but strictly inferior (details below). The enabling fact the
> journal missed: `tu_store.rs` has ZERO imports from scheduler - the engine
> was never scheduler logic, only scheduler-located, so the plugin->engine
> call becomes a legal downward call once the engine lives below both.

## 1. Verified facts (on-disk, this session)

- `crates/scheduler/src/tu_store.rs`: 705 lines. Imports exactly:
  `anyhow`, `serde`, `std`, `ariadne_build::{CompilationContext, TranslationUnit}`,
  `ariadne_index::{TuAnalysis, TuCacheFingerprint, CURRENT_ANALYSIS_VERSION}`,
  `ariadne_lang` resolver traits, `ariadne_repo`. No `ariadne_scheduler`, no
  `ariadne_graph`, no `ariadne_report`. Its test module uses
  `ariadne_lang_cpp::CppPlugin` (8 `#[test]` fns).
- `crates/lang-cpp/src/backend.rs:24` (`use ariadne_scheduler::{TuTask, WorkerBackend};`)
  and `:180` (`ariadne_scheduler::analyze_tu_full_with_plugin(...)`, returned
  fingerprint discarded with `_`) are the ONLY scheduler references in the
  plugin crate. `TuAnalysis` already arrives via `ariadne_lang` (lang-api
  re-export), so no `index` edge is needed for the analysis type.
- `crates/scheduler/src/worker.rs:92-93` discards the fingerprint too;
  fingerprint consumers are scheduler/pipeline-internal (`incremental.rs:9-12`,
  `engine.rs:17-20`, `pipeline/stage_index.rs:19-22`).
- `crates/scheduler/src/queue.rs:8-47`: `TuTask` struct + `PartialEq/Eq/PartialOrd/Ord`;
  the queue machinery (`PrioritizedTuQueue`, `estimate_tu_cost`) is separate.
- `crates/scheduler/src/worker.rs:37-41`: `impl<T: WorkerBackend + ?Sized> WorkerBackend for Arc<T>` -
  a blanket impl over a foreign generic; the orphan rule forces it to live in
  the trait's defining crate wherever the trait goes.
- `crates/scheduler/src/lib.rs`: `pub mod`+glob for queue/worker/tu_store -
  the flat surface external code uses; `m18_persistence_incremental_test.rs:226`
  uses the module-qualified `ariadne_scheduler::tu_store::ShardCheckpoint`.
- Manifest cones: `index -> {repo, build, lang-api}`; `build -> {repo, lang-api, lang}`
  (+ dev-dep lang-cpp, cycle precedent documented at `crates/build/Cargo.toml:16-21`);
  `lang -> {repo, lang-api}`; `lang-api -> {repo}`. NONE contains a forbidden
  crate, so a neutral crate over `{repo, build, index, lang}` is guard-clean.
  `scheduler -> graph` DIRECT (shard.rs/engine.rs/incremental.rs) and
  `scheduler -> report` direct; `report -> {graph, index, cluster, transform, validate}`.
  `lang-scala -> {repo, lang-api, lang}` - transitive-clean already.
- Guard sources: existing on-disk guard forbids direct `graph`/`cluster`/`transform`/`validate`
  (`m17_plugin_seam_test.rs:269-297`). The reverted extension (journal,
  machine-proven RED `6 passed / 1 failed / 0 ignored / 1 suites`) forbids
  TRANSITIVELY `scheduler`/`report`/`cluster`/`graph`/`transform`/`validate`
  over `[dependencies]`+`[build-dependencies]` local path edges; `index`,
  `build`, `lang`, `repo` are NOT in either forbidden list - confirmed against
  both guard sources. `TuCacheFingerprint` is defined at `crates/index/src/cache.rs:33`
  and its `compute` takes `ariadne_build::CompilationContext`.

## 2. Options adjudicated

- **B (exempt the scheduler edge) - REJECTED.** Machine fact: scheduler deps
  `graph` directly, so the exempted edge keeps `graph` (a first-list forbidden
  crate) in the plugin compile cone - permanently, including post-AR-26. B
  would enshrine a guard that misstates the invariant it names.
- **A (lang-api merger handle + injection) - feasible, rejected as strictly
  inferior.** Verified costs: (i) `TuStoreContext`'s private `hash_by_id`/`path_by_id`
  are read by the engine (`tu_store.rs:119,:296,:302`) which under A stays in
  scheduler, forcing pub-visibility of engine pre-index state inside the
  vocabulary seam; (ii) lang-api's manifest must grow `ariadne-build` +
  `anyhow` (TuTask/`TranslationUnit` + trait `Result`), diluting the thin
  vocabulary role locked by `lang_api_types_are_the_same_types_index_re_exports`
  just as AR-24/AR-27 stack onto that crate; (iii) the merger's return names
  `TuCacheFingerprint` (index/src/cache.rs:33) which lang-api cannot name
  without relocating that type (dragging `build` in via its `compute`
  signature) or changing the engine API shape (dropping the fingerprint) that
  `engine.rs`/`incremental.rs` consume; (iv) injection requires giving the
  unit-struct `ClangWorkerBackend` construction state, editing m16
  integration-suite construction sites - breaking UNEDITED-suite purity
  outside the handoff's own files; (v) the engine ends up split across two
  crates with an adapter closure. A was the journal's least-bad option because
  it assumed the engine cannot move; fact 1 shows it can.
- **C-full (engine into neutral crate, everything else untouched) -
  insufficient:** lang-cpp still names `scheduler::TuTask` +
  `scheduler::WorkerBackend`, so the dep cannot drop and the transitive guard
  stays RED. The vocabulary must travel with the engine.
- **C-lite - CHOSEN.** Neutral crate = engine (`tu_store` module verbatim) +
  `TuTask` + `WorkerBackend` + the `Arc` blanket impl. No injection, no
  runtime state, no lang-api touch, no API shape change; verbatim-move
  discipline preserves every byte-parity lock; scheduler's public surface is
  preserved by re-exports so the blast radius outside worker-core/scheduler/lang-cpp
  is zero.

## 3. Invariants (each independently checkable)

1. **m18 byte-parity UNEDITED:** every file under
   `tests/integration/tests/m18_cpp_plugin_parity/` + its harness has an
   unchanged git blob across the chain; suite green at both landings. Test:
   `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test`
   + `git status --porcelain` empty on that path (baseline `23 passed / 0 failed / 0 ignored / 1 suites`
   from the AR-18 landing row; re-measure at dispatch).
2. **m00 golden UNEDITED:** `git status --porcelain -- tests/golden` empty;
   m00 parity+golden green (baseline `4 passed / 0 failed / 0 ignored / 2 suites`).
3. **Public-path compatibility (20a):** after the move, the grep
   `rg "scheduler::(tu_store|queue|worker)::"` hit-set is identical to
   pre-move, every hit compiles, and no file outside
   worker-core/scheduler (20a) / +lang-cpp (20b) was edited. Locks this
   invariant: `cargo check --workspace --all-targets` at 20a + the UNEDITED
   suites m16 (`2/0/0/1`), m13, m18_persistence (`7/0/0/1`), m18 parity,
   m12, plus pipeline + cli scoped suites.
4. **Census conservation (20a):** scheduler-suite + worker-core-suite test
   counts at landing equal the pre-move scheduler baseline (`34 passed / 0 failed / 0
   ignored / 2 suites`) with zero tests authored or deleted; the 8 `#[test]`
   fns in `tu_store.rs`'s test module travel (diff shows attribute-move, not
   add/remove - the ledger's phantom-test tool family is the cross-check).
5. **Guard never RED in history:** `git log -S plugin_crates_transitive_tree_excludes_core_pipeline`
   shows exactly ONE introducing commit = the 20b landing; m17 suite at 20b:
   `7 passed / 0 failed / 0 ignored / 1 suites`.
6. **End-state cone (20b):** `python tools/distill.py -- cargo tree -p ariadne-lang-cpp`
   output contains none of `ariadne-scheduler` `ariadne-report`
   `ariadne-cluster` `ariadne-graph` `ariadne-transform` `ariadne-validate`
   (dev edges excluded), quoted from own stdout per the capture-race rule.
7. **lang-api untouched:** `git status --porcelain -- crates/lang-api` empty
   across the whole chain.
8. **Byte-faithful move:** `git diff --find-renames` (20a) records
   `tu_store.rs` as a rename with zero content hunks;
   `python tools/encoding_guard.py --all` silent (CRLF/mojibake oracle - the
   journal's Edit-round-trip LF slip precedent).
9. **Non-vacuity (20b):** new self-test
   `transitive_guard_detects_planted_scheduler_edge` - the closure walker
   applied to an in-memory planted manifest string containing
   `ariadne-scheduler = { path = "../scheduler" }` must report the edge;
   mirrors the `guard_neutral_vocab_stripping_cannot_mask_plugin_names`
   pattern. Green in-tree; substitutes for the unobservable in-tree RED
   (whose evidence is the journal's `6 passed / 1 failed` run at `d63c0d0`/`7adbeca`).

## 4. Contracts

### 4.1 worker-core crate (new; name `ariadne-worker-core`, dir `crates/worker-core`)

`Cargo.toml` `[dependencies]`: `anyhow`, `serde`, `serde_json` (workspace),
`ariadne-repo`, `ariadne-build`, `ariadne-index`, `ariadne-lang` (path deps).
`[dev-dependencies]`: `tempfile` (workspace), `ariadne-lang-cpp` (path; the
moved test module runs `CppPlugin` through the seam - cargo-legal test-only
cycle, the documented `crates/build/Cargo.toml:16-21` precedent).

`src/lib.rs` mirrors scheduler's existing style (so paths read naturally):
`pub mod tu_store; pub use tu_store::*; pub mod task; pub use task::*;
pub mod worker_backend; pub use worker_backend::*;`

- `src/tu_store.rs` - MOVED VERBATIM (all pub items unchanged:
  `ShardCheckpoint { pub fingerprint: String, pub analysis: TuAnalysis }`,
  `shard_dir`, `load_shard`, `save_shard`, `TuStoreContext { pub discovery,
  pub contexts, hash_by_id: private, path_by_id: private }`,
  `SharedTuStoreContext`, `TuClosure { pub files, pub fingerprint }`,
  `analyze_closure_with`, `analyze_tu_full_with_plugin`,
  `compute_tu_closure_plugin`). Field visibilities UNCHANGED - the engine
  travels with its private pre-indexes. No signature edits.
- `src/task.rs` - `TuTask` verbatim from `queue.rs:8-47` (struct +
  `PartialEq/Eq/PartialOrd/Ord` impls).
- `src/worker_backend.rs` - `pub trait WorkerBackend: Send + Sync { fn
  analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis>; }`
  verbatim (keeps `use ariadne_index::TuAnalysis;` - same type as
  `ariadne_lang::TuAnalysis` via the m17 seam lock) + the
  `impl<T: WorkerBackend + ?Sized> WorkerBackend for Arc<T>` blanket impl
  (orphan rule; must live here).

### 4.2 scheduler shims (20a) - the only scheduler source edits

- `src/lib.rs`: replace `pub mod tu_store; pub use tu_store::*;` with
  `pub use ariadne_worker_core::tu_store; pub use ariadne_worker_core::tu_store::*;`
  (module alias preserves `scheduler::tu_store::...` paths incl. the m18_persistence
  use at :226; glob preserves the flat names; internal `crate::tu_store::`
  uses in `engine.rs`/`incremental.rs` keep resolving).
- `src/queue.rs`: delete `TuTask` block, add `pub use ariadne_worker_core::task::TuTask;`
  (keeps `crate::queue::TuTask` resolving for `worker.rs:5` and `pool.rs:185,:409`).
- `src/worker.rs`: delete the trait + `Arc` blanket impl, add
  `pub use ariadne_worker_core::worker_backend::WorkerBackend;` (keeps
  `crate::worker::{SharedPluginWorkerBackend, WorkerBackend}` resolving for
  `engine.rs:21`; local impls `SharedPluginWorkerBackend`,
  `PluginWorkerBackend` stay - local types satisfy the orphan rule).
- `Cargo.toml`: add `ariadne-worker-core = { path = "../worker-core" }`;
  KEEP the `ariadne-lang-cpp` dev-dep (worker.rs dispatch tests still name
  `CppPlugin`).
- Root `Cargo.toml`: +1 member line `crates/worker-core`.
- `engine.rs`, `incremental.rs`, `pool.rs`, `shard.rs`: UNTOUCHED (paths
  resolve through shims by design).

### 4.3 lang-cpp switch (20b)

- `Cargo.toml`: remove `ariadne-scheduler` line; add
  `ariadne-worker-core = { path = "../worker-core" }`; rewrite the manifest
  note (lines 11-15) to: backend implements `ariadne_worker_core::WorkerBackend`
  over `ariadne_worker_core::TuTask` and calls the engine directly - the
  worker-core dep is why the guard cone is clean.
- `src/backend.rs`: `:24` -> `use ariadne_worker_core::{TuTask, WorkerBackend};`
  `:180` -> `ariadne_worker_core::tu_store::analyze_tu_full_with_plugin(...)`
  (module-qualified, mirroring today's shape); module-doc `:19` mention
  repointed. Three reference edits; zero logic edits.
- `tests/integration/tests/m17_plugin_seam_test.rs`: re-add the journal's
  extension byte-for-byte from its documented description (path-closure walk
  over `[dependencies]`/`[build-dependencies]`, dev edges skipped,
  cycle-safe) + the planted-edge self-test of invariant 9. Lands only here
  (verdict, section 6).

### 4.4 Re-export shims summary

| preserved path | mechanism | consumer lock |
|---|---|---|
| `ariadne_scheduler::tu_store::*` (module + names) | lib.rs module alias + glob | m18_persistence:226 |
| `ariadne_scheduler::{TuStoreContext, SharedTuStoreContext, ShardCheckpoint, TuClosure, ...}` | glob re-export | pipeline stage_index/lib/for_incremental, resolver_parity:338, external_boundary_parity:447 |
| `ariadne_scheduler::{TuTask, WorkerBackend, PrioritizedTuQueue, WorkerPool, SharedPluginWorkerBackend, PluginWorkerBackend}` | queue/worker re-export + kept modules | m16, m13, worker_backend_parity |
| `scheduler::queue::TuTask`, `scheduler::worker::WorkerBackend` (internal + qualified) | `pub use` inside the named modules | worker.rs:5, pool.rs, engine.rs:21 |

## 5. Handoffs (sizing per Planning Law; planner authors them)

- **AR-20a - additive move.** Scope: worker-core (new, ~825 created lines of
  which 705 are a byte-identical rename - real content diff ~150-200 lines) +
  scheduler 3 src files + 2 manifests. <=1 new crate dir + shims; produced
  lines within the ~1200 law because the rename is detected (mandate byte-safe
  move: `git mv` preferred, else byte-level write so `--find-renames` sees
  R100; journal's Edit-round-trip CRLF precedent makes byte-safety a gate,
  invariant 8). Guard test NOT touched. All gates of kickoff-20a.
- **AR-20b - switch + guard.** Scope: lang-cpp manifest + `backend.rs`
  (3 reference lines + module doc), m17 guard extension + self-test, journal.
  ~80-140 diff lines, ~10 gate commands. One commit: dep removal and guard are
  atomic - neither can exist without the other being true.
- Split rather than one handoff: the touch set spans 4 dirs (worker-core,
  scheduler, lang-cpp, tests/integration) violating the 1-crate law if
  combined, and the guard's GREEN-atomicity wants the smallest possible
  landing. AR-18/AR-19 precedent: additive before switching.

## 6. Guard RED re-land - explicit verdict

The extension is NOT re-committed as RED anywhere, on any branch. Its
falsification already exists as machine evidence: the journal run
`6 passed / 1 failed / 0 ignored / 1 suites` naming all six offenders, with
the tree restored byte-identical (porcelain-empty) in `d63c0d0`/`7adbeca`.
AR-20b lands guard + self-test + dep removal + repoints in ONE commit whose
tree is GREEN (`7 passed / 0 failed / 0 ignored / 1 suites`); that commit is
the guard's only history. Non-vacuity for a guard whose in-tree RED is
unobservable is carried by invariant 9's planted-edge self-test (green,
deterministic) plus the journal citation in the 20b handoff's before-evidence.

## 7. Risks -> mitigation -> oracle

- **Re-export drift** (a name forgotten) -> glob re-exports make name-by-name
  forgetting structurally impossible for `tu_store::*`; queue/worker names are
  the only enumerated shims. Oracle: `cargo check --workspace --all-targets`
  + the UNEDITED consumer suites (invariant 3).
- **Byte drift on the move (CRLF/mojibake; core.autocrlf=true, journal
  precedent)** -> byte-safe move mandate. Oracle: `--find-renames` R100 +
  `encoding_guard --all` silent (invariant 8).
- **Census shift misread as test loss** -> journal quotes both suite lines +
  the 34-total equation; ledger census row records the shift. Oracle:
  invariant 4.
- **Dev-dep cycle worker-core <-> lang-cpp breaks builds** -> cargo supports
  test-only cycles; already true three times over (build, report, scheduler).
  Oracle: `--all-targets` check at 20a.
- **Guard extension vacuous on manifest-table shapes** -> planted-edge
  self-test (invariant 9); walker restricted to the same tables the journal's
  RED proved.
- **AR-26 interaction (ordering constraint, stated both directions):**
  REQUIRED: AR-20a+20b land BEFORE AR-26's scheduler-side reparent handoff.
  Reasons: (i) the plan already prefers AR-20 first and 26's "smaller plugin
  cone after AR-20" compounding claim is only measurable once 20b's guard is
  true; (ii) file-disjointness is engineered: 20a edits scheduler
  `lib.rs/queue.rs/worker.rs`, AR-26 edits scheduler call sites
  (`incremental.rs:23` ArtifactManager etc.) - disjoint; shared 1-line
  surfaces are root `Cargo.toml` members and scheduler `Cargo.toml` - trivial
  rebase, declare 26 rebase-on-20b. (iii) If AR-26 landed first anyway, 20a's
  rename must rebase over the smaller scheduler - doable, but the guard's
  end-state cone fact would cite a moving scheduler tree; hold 26's kickoff.
- **`"clang++"` fallback string lands in a neutral crate** (`tu_store.rs:141`) -
  R2 guard's 7-crate walk scope is UNCHANGED (worker-core not in scope), so
  no new violation; A8 (deferred Low) owns neutralizing it, and worker-core
  is its correct future home. Note for planner: dated ledger candidate row,
  no code change here.
- **Injection/lock-injection runtime risk from option A** - eliminated by
  construction: C-lite adds zero runtime state; the call chain is static
  (backend -> worker-core function), identical lock behavior to today.
- **Future contributor repoints `engine.rs`/`incremental.rs` around the
  shims** - harmless; explicitly out of scope; shims are documented as the
  compat contract in scheduler's lib.rs move hunk.

## 8. Module map

| file | action | handoff |
|---|---|---|
| `crates/worker-core/{Cargo.toml, src/lib.rs, src/tu_store.rs, src/task.rs, src/worker_backend.rs}` | NEW (tu_store via byte-safe rename) | 20a |
| `crates/scheduler/src/{lib.rs, queue.rs, worker.rs}` | shim edits only | 20a |
| `crates/scheduler/src/tu_store.rs` | removed by rename | 20a |
| `crates/scheduler/Cargo.toml`, root `Cargo.toml` | +1 dep / +1 member | 20a |
| `crates/lang-cpp/{Cargo.toml, src/backend.rs}` | dep drop + 3 reference repoints + note rewrite | 20b |
| `tests/integration/tests/m17_plugin_seam_test.rs` | transitive guard + self-test, GREEN | 20b |
| MUST NOT touch: `crates/lang-api/**`, `crates/index/**`, `crates/build/**`, `crates/lang/**`, `crates/pipeline/**`, `crates/cli/**`, `crates/ui/**`, `crates/report/**`, `tests/golden/**`, m16/m13/m18*/m00/m12 suites, `tests/integration/tests/common/**` | - | both |

## 9. Hygiene notes

- `tu_store.rs` moves whole at 705 lines - under the 1000 ceiling, no split
  required; do NOT grow it (fingerprint/A8 work stays deferred).
- Post-move scheduler files all shrink: `worker.rs` 655 -> ~640, `queue.rs`
  236 -> ~190; `worker.rs`'s 545-line dispatch test mod stays its largest
  region - watch on next touch, not blocking.
- `m17_plugin_seam_test.rs` 455 -> ~540 at 20b (guard + self-test): well
  under ceiling; keep the walker helper `#[cfg(test)]`-free only if reuse
  demands, else all in-file.
- No line anywhere near 2000 chars; prose wraps sentence-per-line in the
  moved/added docs as authored; all authored content pure ASCII (repo rule).
- Names: `worker-core`/`worker_backend`/`task` chosen to mirror existing
  module vocabulary; no `tmp`/`res`/`data` introduced anywhere in the chain.
