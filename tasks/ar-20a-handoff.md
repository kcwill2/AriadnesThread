# AR-20a Handoff - worker-core extraction, additive move + re-export shims (A4 part 1)

Successor of the blocked AR-20 (journal `tasks/ar-20-handoff-journal.md`:
zero code landed; `d63c0d0`/`7adbeca` are journal-only). Shape ruling
`tasks/ar-20-shape.md` (C-lite) is binding: do not redesign it. Sibling:
`tasks/ar-20b-handoff.md` (switch + guard) lands AFTER this one is APPROVED.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ar-20-shape.md` sections 1, 3, 4.1, 4.2, 4.4, 8, 9 (facts,
   invariants, worker-core contract, shim contract, module map, hygiene).
3. `tasks/ar-20-kickoff.md` (invariants 1-4, 7, 8 + frozen locks).
4. Mirror files: `crates/scheduler/src/lib.rs` (module style to mirror),
   `crates/build/Cargo.toml:16-21` (dev-dep cycle precedent note style),
   `crates/scheduler/Cargo.toml:19-25` (existing dev-dep comment).

## Objective

Extract the worker vocabulary from `ariadne-scheduler` into a NEW neutral
crate `ariadne-worker-core` (`crates/worker-core`) - via BYTE-SAFE rename/move,
zero signature edits - and keep every scheduler public path resolving via
re-export shims so ALL external consumers compile UNEDITED. This handoff is
additive: `crates/lang-cpp` still points at scheduler re-exports after 20a
(zero consumer edits), the m17 guard is UNTOUCHED, runtime behavior is zero-delta.

Invariants preserved (shape 3, numbered there):
1. m18 byte-parity UNEDITED (`m18_cpp_plugin_parity_test`, baseline
   `23 passed / 0 failed / 0 ignored / 1 suites` - re-measure at dispatch HEAD).
2. m00 golden UNEDITED (`4 passed / 0 failed / 0 ignored / 2 suites`;
   `git status --porcelain -- tests/golden` empty).
3. Public-path compatibility: `rg "scheduler::(tu_store|queue|worker)::"`
   hit-set identical pre-move/post-move; every hit compiles; specifically
   module-qualified `ariadne_scheduler::tu_store::ShardCheckpoint` at
   `tests/integration/tests/m18_persistence_incremental_test.rs:226` keeps
   resolving UNEDITED. No file outside worker-core+scheduler+2 manifests edits.
4. Census conservation: scheduler-suite + worker-core-suite counts equal the
   pre-move `34 passed / 0 failed / 0 ignored / 2 suites` total with ZERO tests
   authored or deleted; the 8 `#[test]` fns in `tu_store.rs:438`'s test module
   TRAVEL (attribute-move, not add/remove - phantom-test tool family is the
   cross-check, `#[test]` attribute count diff).
8. Byte-faithful move: `git diff --find-renames` records `tu_store.rs` as a
   rename with ZERO content hunks (R100); `encoding_guard --all` silent.
7. `git status --porcelain -- crates/lang-api` empty (as in 20b, whole chain).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ar-20a-handoff-journal.md` (checkpoint journal; always in-set)
- `crates/worker-core/Cargo.toml` (NEW - shape 4.1 deps: `anyhow`/`serde`/
  `serde_json` workspace + `ariadne-repo`/`ariadne-build`/`ariadne-index`/
  `ariadne-lang` path; dev-deps `tempfile` workspace + `ariadne-lang-cpp`
  path = cargo-legal test-only cycle, cite the build/Cargo.toml:16-21 precedent)
- `crates/worker-core/src/lib.rs` (NEW - `pub mod tu_store; pub use tu_store::*;
  pub mod task; pub use task::*; pub mod worker_backend; pub use
  worker_backend::*;` mirroring scheduler lib.rs style)
- `crates/worker-core/src/tu_store.rs` (MOVED VERBATIM via byte-safe rename -
  see RGB protocol; all pub items + field visibilities unchanged, incl.
  `TuStoreContext` private `hash_by_id`/`path_by_id` - the engine travels with
  its private pre-indexes)
- `crates/worker-core/src/task.rs` (NEW - `TuTask` verbatim from
  `crates/scheduler/src/queue.rs:8-47`: struct + `PartialEq/Eq/PartialOrd/Ord`)
- `crates/worker-core/src/worker_backend.rs` (NEW - `WorkerBackend` trait
  verbatim from `crates/scheduler/src/worker.rs:50-52` PLUS the
  `impl<T: WorkerBackend + ?Sized> WorkerBackend for Arc<T>` blanket impl from
  `worker.rs:37-41`; the orphan rule FORCES the blanket impl to live in the
  trait's defining crate - it cannot stay in scheduler; keeps
  `use ariadne_index::TuAnalysis;` - same type as `ariadne_lang::TuAnalysis`
  via the m17 seam lock)
- `crates/scheduler/src/lib.rs` (`:18-19` shim, see shim list)
- `crates/scheduler/src/queue.rs` (delete `TuTask` block `:8-47`, add shim line)
- `crates/scheduler/src/worker.rs` (delete trait `:50-52` + blanket impl
  `:37-41`, add shim line; local impls `SharedPluginWorkerBackend`/
  `PluginWorkerBackend` STAY - local types satisfy the orphan rule)
- `crates/scheduler/src/tu_store.rs` (removed BY the rename - not hand-deleted)
- `crates/scheduler/Cargo.toml` (+`ariadne-worker-core = { path = "../worker-core" }`;
  KEEP the `ariadne-lang-cpp` dev-dep - worker.rs dispatch tests still name `CppPlugin`)
- `Cargo.toml` (root: +1 member line `"crates/worker-core"`)
- `Cargo.lock` (cargo regenerates it for the new package member)

You MUST NOT touch: `tests/integration/tests/m17_plugin_seam_test.rs` (guard
UNTOUCHED - AR-20b owns it), `crates/lang-cpp/**` (still points at scheduler
re-exports after 20a), `crates/lang-api/**`, `crates/index/**`,
`crates/build/**`, `crates/lang/**`, `crates/pipeline/**`, `crates/cli/**`,
`crates/ui/**`, `crates/report/**`, `tests/golden/**`,
`tests/integration/**` (every suite UNEDITED), `tasks/ar-20-handoff.md`,
`tasks/ar-20-handoff-journal.md`, `tasks/ar-20-shape.md`. No `git stash`, no
branch change.

## Re-export shim rule + exact shim list (carry-forward from shape 4.2/4.4 -
the compat contract; document it in the lib.rs move hunk)

| preserved path | mechanism | consumer lock |
|---|---|---|
| `ariadne_scheduler::tu_store::*` (module + names) | lib.rs module alias + glob | m18_persistence:226 |
| `ariadne_scheduler::{TuStoreContext, SharedTuStoreContext, ShardCheckpoint, TuClosure, ...}` | glob re-export | pipeline stage_index/lib/for_incremental, resolver_parity:338, external_boundary_parity:447 |
| `ariadne_scheduler::{TuTask, WorkerBackend, PrioritizedTuQueue, WorkerPool, SharedPluginWorkerBackend, PluginWorkerBackend}` | queue/worker re-export + kept modules | m16, m13, worker_backend_parity |
| `scheduler::queue::TuTask`, `scheduler::worker::WorkerBackend` (internal + qualified) | `pub use` inside the named modules | worker.rs:5, pool.rs:185/:409, engine.rs:21 |

The three source shims, exactly:
1. `crates/scheduler/src/lib.rs:18-19`: replace `pub mod tu_store;` +
   `pub use tu_store::*;` with `pub use ariadne_worker_core::tu_store;` +
   `pub use ariadne_worker_core::tu_store::*;` (module alias preserves
   `scheduler::tu_store::...` paths; glob preserves the flat names; internal
   `crate::tu_store::` uses in `engine.rs`/`incremental.rs` keep resolving -
   a root-level `pub use` binding is path-resolvable in-crate).
2. `crates/scheduler/src/queue.rs`: add `pub use ariadne_worker_core::task::TuTask;`
   (keeps `crate::queue::TuTask` resolving for `worker.rs:5` and pool.rs).
3. `crates/scheduler/src/worker.rs`: add
   `pub use ariadne_worker_core::worker_backend::WorkerBackend;` (keeps
   `crate::worker::{SharedPluginWorkerBackend, WorkerBackend}` resolving for
   `engine.rs:21`).

`engine.rs`, `incremental.rs`, `pool.rs`, `shard.rs`: UNTOUCHED (paths resolve
through the shims by design; repointing them is out of scope).

## RGB protocol

1. Red (move handoffs carry no new test - census is frozen; Red = falsifiability):
   a. Journal the dispatch HEAD + baseline suite lines (m18 parity,
      m18_persistence, m16, m00, lang-cpp, scheduler) BEFORE touching anything.
   b. Byte-safe move: prefer `git mv crates/scheduler/src/tu_store.rs
      crates/worker-core/src/tu_store.rs` (stage the rename FIRST so
      `--find-renames` sees it); if byte-safe `git mv` is impossible, byte-level
      write of the exact blob (Python `open(path,"wb")` under the repo rules).
      NEVER recreate the file through Edit-tool round-trips (core.autocrlf=true;
      the journal's Edit-round-trip LF-slip precedent).
   c. Shim load-bearingness mutation proof (uncommitted): with shims in place,
      comment out the lib.rs `pub use ariadne_worker_core::tu_store;` alias
      line, run `python tools/distill.py -- cargo check -p ariadne-integration-tests --test m18_persistence_incremental_test`,
      capture the expected failure - `error[E0433]`/`E0432`: cannot find
      `tu_store` in `ariadne_scheduler` - then restore the line byte-exact
      (`git diff` empty on lib.rs). This is the shape's oracle that the shim is
      not decorative. NEVER commit a red state.
2. Green: all gates below pass; canonical lines quoted from own stdout.
3. Refactor: NONE permitted on the moved code (verbatim is the contract).
   Hygiene review applies only to the new manifest/lib.rs/task.rs/worker_backend.rs.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, intention-revealing names, one
responsibility per function. `tu_store.rs` moves whole at 705 lines - under
the ceiling; do NOT grow it (fingerprint/A8 work stays deferred). Post-move
scheduler files SHRINK (`worker.rs` 655 -> ~640, `queue.rs` 236 -> ~190).
Names `worker-core`/`worker_backend`/`task` mirror existing vocabulary; no
`tmp`/`res`/`data` anywhere. `python tools/hygiene_check.py` exit 0 with ZERO
blocking rows - the AR-13 landing (`aaffde0`/`14e21b1`) retired the standing
"2 server.rs rows" baseline; downstream contracts must not re-tolerate R1/R2
rows (AR-13 RESULT, 2026-10-11). Quote the run verbatim; zero new rows from
touched files. Note (shape 7, no action here): the `"clang++"` fallback string at
`tu_store.rs:141` lands in a neutral crate - the R2 guard's 7-crate walk scope
is UNCHANGED (worker-core not in scope), so no new violation; A8 owns it later.

## Frozen locks (verify + quote)

- m17 guard file: `git diff <dispatch-HEAD> -- tests/integration/tests/m17_plugin_seam_test.rs` EMPTY.
- m18 parity/m18 persistence/m16/m00/m13/m12 + all `tests/integration/**` +
  `tests/golden/**`: `git status --porcelain -- tests/` EMPTY at landing.
- serde wire format byte-identical (moved types, zero attribute changes).
- Until `golden_guard.py` lands: manual byte-identical check of the above paths.

## Gates (exact commands, timeout 600000 ms each; scoped only - workspace-wide
final gates belong to the reviewer's acceptance pass)

1. `python tools/distill.py -- cargo check --workspace --all-targets` -- GREEN
   first (also proves the worker-core dev-dep cycle: build/report/scheduler
   already do this three times over). Capture `target/distill-capture.txt`.
2. `python tools/testrun.py --distill -p ariadne-worker-core` -- expect
   `8 passed / 0 failed / 0 ignored / 1 suites` (the 8 traveling tu_store tests).
3. `python tools/testrun.py --distill -p ariadne-scheduler` -- CONSERVATION
   EQUATION: gate 2 + gate 3 sum = the pre-move `34 passed / 0 failed /
   0 ignored / 2 suites` total; journal both lines + the equation; ledger census
   row records the shift.
4. `python tools/testrun.py --distill -p ariadne-lang-cpp` -- `153 passed /
   0 failed / 0 ignored / 2 suites` maintained UNEDITED (proof lang-cpp compiles
   against scheduler re-exports with zero edits).
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` -- `23 passed / 0 failed / 0 ignored / 1 suites` UNEDITED.
6. `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_persistence_incremental_test` -- `7 passed / 0 failed / 0 ignored / 1 suites` UNEDITED (the :226 module-qualified path lock).
7. `python tools/testrun.py --distill -p ariadne-integration-tests --test m16_clang_worker_test` -- `2 passed / 0 failed / 0 ignored / 1 suites` UNEDITED.
8. `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test` -- `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED + `git status --porcelain -- tests/golden` empty.
9. `python tools/hygiene_check.py` -- exit 0, zero blocking rows (post-AR-13
   baseline); `python tools/encoding_guard.py --staged` -- exit 0 before
   committing (+ `--all` in the journal).

Cheap non-gate evidence (journal): the `rg` hit-set identity (invariant 3),
`git diff --find-renames -M --stat` showing `tu_store.rs` as R100 with zero
content hunks (invariant 8), and `git status --porcelain -- tasks/ar-20-handoff-journal.md` empty.

## Discipline clauses (shared with AR-20b)

- STRICT COMPILE-GREEN (shared worktree): the move + all shims + both
  manifests + Cargo.lock land in ONE commit - no intermediate tree is ever
  committed non-compiling. The AR-13 ui lane is ACTIVE during 20a (it edits
  only `crates/ui/**` - disjoint from this touch-set): if a workspace check
  transiently trips on foreign uncommitted dirt, journal the error attribution
  verbatim, verify your own cone checks green, do NOT fix or stage foreign files.
- UTF-8: never Set-Content/Out-File/`>`/`>>`/Get-Content-replace; Write/Edit
  tools only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
  Where an edit abuts pre-existing non-ASCII prose, keep the edit byte-local.
- Capture paths in the journal: every gate run records its canonical line
  quoted from OWN stdout + `target/testrun-capture.txt` /
  `target/distill-capture.txt` path (shared capture races - board-ruled).
- Touch only the touch-set; on discovering out-of-scope breakage (including a
  `crate::`-path inside the moved `tu_store.rs` that would contradict shape
  fact 1 - zero scheduler imports): STOP, report, do not fix, do not improvise.
- No raw high-volume commands in your context: everything through the
  distilling entry points (`docs/specs/tool-output-distillation.md`).
- After EACH Green gate, append a 1-3 line checkpoint to
  `tasks/ar-20a-handoff-journal.md`: module + commit hash, deviations,
  lock/contract findings. If context exhaustion looms: STOP, return a
  disk-state inventory; a dead session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ar-20a-handoff.md ...`.
- Commit: `python tools/commit_scoped.py --scope "crates/worker-core/**"
  --scope "crates/scheduler/**" --scope "Cargo.toml" --scope "Cargo.lock"
  --scope "tasks/ar-20a-handoff-journal.md" -m "refactor(worker-core): extract
  tu_store engine + worker vocabulary; scheduler re-export shims (AR-20a)"`
  (never `git add .`). A second journal-only commit for the hash record is
  the AR-16/AR-19 precedent.

## Acceptance (reviewer re-runs these fresh)

All 9 gate commands above exit green with the pinned canonical lines, plus:
- `git log -1 --stat` touch-set EXACT (rename visible: `git show --stat -M`
  shows `tu_store.rs` as a rename, not delete+add).
- Expected verdict lines: gates 2-8 canonical summary lines verbatim;
  conservation equation `gate2 + gate3 = 34 / 0 / 0 / (2 suites + 1 suite, split journal-verified)`;
  encoding exit 0; hygiene exit 0 zero-blocking (post-AR-13).

## Dependencies / serialization

- Dispatch: AR-13 LANDED clean (`aaffde0`/`14e21b1`/`768529f`, `crates/ui/**`
  only - disjoint from this touch-set) but its reviewer pass may still rework
  ui; no lane edits `crates/scheduler` or `crates/lang**`. Safe to dispatch now.
- AR-20b dispatches ONLY after this handoff is APPROVED and landed.
- AR-26's scheduler-side reparent stays HELD through the whole AR-20 chain
  (shape 7: declare AR-26 rebase-on-20b; shared 1-line surfaces are root
  `Cargo.toml` members + scheduler `Cargo.toml`).

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill (fresh full verification, per-commit `git show` oldest-first,
commit-vs-touch-set enforcement against the list above - it includes
`Cargo.lock` precisely because cargo regenerates it).
