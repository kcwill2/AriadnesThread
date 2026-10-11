# AR-20a Verdict — worker-core extraction (additive move + re-export shims)

**APPROVED.** Commits `0861db0` (landing, 12 paths) + `4dc7237` (journal hash
record), parent = dispatch HEAD `768529f` (verified via
`git rev-parse "0861db0^"`). Verdict per `review-gate` skill; fresh gate
evidence in `tasks/ar-20a-execution-results.md`.

## 1. Commit-vs-touch-set — EXACT

`git diff --name-status -M "768529f" "4dc7237"` returned exactly the 12
declared paths: 6 M (`Cargo.lock`, root `Cargo.toml`, `crates/scheduler/
{Cargo.toml,src/lib.rs,src/queue.rs,src/worker.rs}`), 5 A
(`crates/worker-core/{Cargo.toml,src/lib.rs,src/task.rs,
src/worker_backend.rs}`, `tasks/ar-20a-handoff-journal.md`), and 1 R100
(`crates/scheduler/src/tu_store.rs` -> `crates/worker-core/src/tu_store.rs`).
Nothing outside the handoff touch-set. `4dc7237` alone touches only the
journal (AR-16/AR-19 precedent honored).

## 2. R100 authenticity — machine-proven (central check)

`git rev-parse` three-way, my own run:

- `4dc7237:crates/worker-core/src/tu_store.rs` -> `202df67913c63b5eb7bfccf242bd89309b9db834`
- `0861db0^:crates/scheduler/src/tu_store.rs` -> `202df67913c63b5eb7bfccf242bd89309b9db834`
- `0861db0:crates/worker-core/src/tu_store.rs` -> `202df67913c63b5eb7bfccf242bd89309b9db834`

Same blob at pre-move path, landing path, and HEAD; `git show --stat -M` and
`git diff --name-status -M` both record `R100 ... | 0` - zero content hunks.
Not a content edit dressed as a rename. `git rev-parse
"0861db0:crates/scheduler/src/tu_store.rs"` -> `fatal: path ... does not
exist in '0861db0'` - landed commit complete (tu_store present only in
worker-core; D7 resolved on the merits).

## 3. Shim set — matches shape 4.2/4.4 exactly

The three source shims, verbatim against the shape list:

- `crates/scheduler/src/lib.rs`: `pub mod tu_store; pub use tu_store::*;`
  replaced by `pub use ariadne_worker_core::tu_store;` +
  `pub use ariadne_worker_core::tu_store::*;` (+ compat-contract doc comment,
  as the contract requires it "in the lib.rs move hunk").
- `crates/scheduler/src/queue.rs`: `TuTask` block (was :8-47) deleted,
  `pub use ariadne_worker_core::task::TuTask;` added.
- `crates/scheduler/src/worker.rs`: trait + `Arc` blanket impl deleted,
  `pub use ariadne_worker_core::worker_backend::WorkerBackend;` added; local
  impls `SharedPluginWorkerBackend`/`PluginWorkerBackend` stay (orphan-rule
  correct).

`scheduler::tu_store::ShardCheckpoint` resolves: `m18_persistence_
incremental_test.rs:226` is UNEDITED in the range and the suite is green
(gate 6). Mutation proof journaled (E0432 `incremental.rs:9`, E0433
`worker.rs:88`), committed tree carries the alias uncommented at lib.rs:24
(read at HEAD). `engine.rs`/`incremental.rs`/`pool.rs`/`shard.rs` untouched
(name-status confirms).

New-crate files match shape 4.1: manifest deps `anyhow`/`serde`/`serde_json`
workspace + `repo`/`build`/`index`/`lang` path, dev-deps `tempfile` workspace
+ `ariadne-lang-cpp` path with the `build/Cargo.toml:16-21` precedent cited;
`lib.rs` = the 3 mod+glob pairs; `task.rs` = `TuTask` verbatim (byte-
comparable to the deletion hunk incl. the 4 impls); `worker_backend.rs` =
trait verbatim + blanket impl verbatim (doc comment included) +
`use ariadne_index::TuAnalysis;`. scheduler manifest keeps the lang-cpp
dev-dep; root manifest +1 member; `Cargo.lock` = pure cargo regeneration
(16 added lines: the `ariadne-worker-core` package + scheduler edge).

## 4. Conservation (invariant 4) — verified with names

Fresh own-stdout: worker-core `8 passed / 0 failed / 0 ignored / 2 suites` +
scheduler `26 passed / 0 failed / 0 ignored / 2 suites` = **34** = pre-move
`34 passed / 0 failed / 0 ignored / 2 suites` (journal Phase 0; independently
corroborated by the AR-16 board RESULT line at `ea4c576` over the same file
state). Roster partition: the 8 moved names are the `tu_store.rs` test-module
fns (`test_shard_envelope_roundtrip_and_old_format_miss`,
`m23_fingerprint_bump_blocks_pre_change_scala_blobs`,
`test_closure_tracks_header_change`, `test_fingerprint_survives_repo_copy`,
`test_closure_merges_header_symbols`, `test_unreadable_header_sentinel`,
`test_missing_tu_source_analyzes_as_miss`,
`test_non_utf8_tu_source_analyzes_as_miss`) - all 8 run under worker-core,
none remain under scheduler; the 26 scheduler test names are all scheduling
tests (queue/pool/worker-dispatch/shard/incremental/engine/lib). Name loss
is structurally excluded: the blob is identical (hash), the 3 modified
scheduler files delete zero `#[test]` lines (full diffs inspected), all
other scheduler files are byte-identical.

## 5. Frozen locks — byte-empty (verified, not trusted)

`git diff --name-only "768529f" "4dc7237" -- crates/lang-cpp crates/lang-api
tests/ crates/index crates/build crates/lang crates/pipeline crates/cli
crates/ui crates/report` -> EMPTY (covers m17 guard, m18 harnesses, m00
goldens). Worktree tracked-tree clean at both ground-truth reads (session
start + session end). Hit-set identity (invariant 3): my own grep finds
exactly the journal's post-move set - `m18_persistence:226` (compiled lock),
`backend.rs:19` + `external_boundary_parity.rs:5` (docs), + the new shim
comment in `scheduler/src/lib.rs:20` (disclosed).

## 6. Deviation dispositions (independent adjudication)

- **D1 (m16 3 vs pinned 2): ACCEPTED.** Fresh run `3 passed / 0 failed / 0
  ignored / 1 suites`; suite UNEDITED in range (frozen-lock diff empty). The
  pin is stale, predating AR-20a. -> ledger row bank (finding 3).
- **D2 (queue.rs dead `use std::cmp::Ordering;` dropped): ACCEPTED.** Its
  sole consumer was the moved `Ord` impl; the edit is one line inside a
  touch-set file, byte-local, declared. Undeclared it would be a finding;
  declared and mechanically forced by the move, it is the minimal complete
  deletion. Re-check green with 0 warnings.
- **D3 (worker-core 2 suites vs pinned 1): ACCEPTED.** The extra suite is the
  empty doc-test binary - the same mechanism that made scheduler measure 2
  pre-move; test count 8 exact; conservation unaffected. Pin correction ->
  ledger (finding 3).
- **D4 (hygiene tmp-warn line attribution :54/:62 -> :54/:61/:62): ACCEPTED.**
  Blob identity is hash-proven, so content drift is impossible; my hygiene
  run lists exactly :54/:61/:62. Checker attribution, warn-only both sides.
- **D5 (`report.json` fixture touched by suite run, restored via
  `git checkout --`): ACCEPTED.** Restore-to-HEAD of autocrlf-normalized
  bytes, zero content delta, frozen lock then read empty; my own range diff +
  end-of-session tree state confirm clean. The fixture-regeneration-by-tests
  behavior itself is a pre-existing repo property -> ledger candidate
  (finding 5).
- **D6 (`"clang++"` fallback lands in neutral crate): NOTED, no action.**
  Shape 7 note; R2 guard 7-crate walk unchanged (worker-core out of scope,
  confirmed - my hygiene/encoding runs show no new violation class); A8 owns
  neutralizing, worker-core is its future home. -> ledger row bank.
- **D7 (commit_scoped cannot carry a staged rename deletion): ACCEPTED.**
  Adjudicated on the landed artifact, not the story: commit `0861db0` is
  rename-complete (section 2), exactly the 12 in-set paths, no foreign
  content swept, discarded attempts (`79a90d3`/`d4630d4`) are not in the
  range (`git log --oneline "768529f..4dc7237"` = exactly 2 commits); a
  self-owned seconds-old unpushed commit removal by `reset --soft` violates
  neither the no-stash nor no-branch-change clause. No tool was edited
  (touch-set honored); recommendation carried to the tooling owner
  (finding 2).

## 7. Findings

All minor; none require rework on `0861db0`.

1. **[minor, environment/ledger-row-bank] Pre-existing pipeline env-race
   flake in workspace censuses.** First fresh census: `946 passed / 1 failed
   / 8 ignored / 60 suites` (cargo fail-fast truncation) -
   `internal_tests::test_load_milestones_absent_on_cache_hit`
   (`crates/pipeline/src/internal_tests.rs:424`) panicked at
   `crates/pipeline/src/stage_load.rs:223` with
   `ARIADNE_LOAD_MEM_CAPS ... got "3,x"`. Mechanism: the M28 T0c cap-ablation
   harness (`crates/pipeline/src/stage_load_tests.rs:166-187`, introduced
   `f312dba`, long pre-AR-20a) sets process-global `ARIADNE_LOAD_MEM_CAPS`
   under a mutex that serializes ONLY its own three tests; unrelated tests in
   the same binary can read it mid-window. Attribution outside AR-20a:
   `crates/pipeline/**` is byte-empty across the range, tree clean at both
   ground truths. My re-runs: pipeline standalone `40 passed / 0 failed / 4
   ignored / 9 suites`; second full census
   **`1372 passed / 0 failed / 13 ignored / 101 suites` exit 0**. Fix owner:
   widen the env mutex to all tests reaching `build_stage5_analyzer` under
   `test-instrument`, or set/remove the caps env inside a process-isolated
   harness.
2. **[minor, tooling] `commit_scoped.py` cannot carry an already-staged
   rename deletion** (journal D7 root cause: `resolve_scopes` enumerates from
   `git ls-files` + `--others`, missing staged deletions; `git commit --only`
   then excludes the old path). Row for the tooling owner: resolve
   candidates also from `git ls-files -d`, or detect staged deletions under a
   directory scope.
3. **[minor, ledger-row-bank] Stale census pins to correct at next ledger
   touch:** m16 pins `2 passed` vs measured `3 passed` (green+UNEDITED both
   sides); AR-20a census row should record the shift 34 -> scheduler 26 +
   worker-core 8 with the 2-suite (empty doc-test) explanation for worker-core.
4. **[minor, tooling] `testrun.py` bare-mode echo crashes on
   `UnicodeEncodeError` (cp1252 console)** when cargo output carries
   non-ASCII (`tools/testrun.py:262` `_echo`; hit on my bare
   m18_persistence run - the harness died, the gate itself was never reached;
   `--distill` mode is immune and is the declared entry point anyway). Row
   for the tooling owner: coerce the echo stream (e.g.
   `errors="backslashreplace"` wrapper) or route bare mode through the same
   byte-safe writer as the capture.
5. **[minor, note-only] Tests regenerate tracked fixtures**
   (`tests/fixtures/basic/artifacts/includes/report.json`, journal D5) - a
   standing census-purity hazard for every frozen-lock handoff; candidate
   ledger row, no action here.
6. **[note, reviewer environment] Reviewer allowlist refusals** (verbatim
   catch-all `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
   `git grep`, `git status`, `git ls-tree`, `python -c`, `python -m unittest`
   with a `Select-Object` pipe, and temp-dir writes were each refused;
   every required gate was still run (refusals were command-scope, not
   bash-wide). Roster/lock checks were substituted with `git diff`/
   `git rev-parse`/`grep`-tool equivalents, all sound; config observation
   only.

## 8. Report-evidence check (distillation contract)

Journal is digest-shaped: canonical lines quoted from own stdout per gate,
capture paths recorded (`target/testrun-capture.txt`,
`target/distill-capture.txt`), distilling entry points used for all heavy
commands, no raw output dumps (the 3-line mutation-proof error block is
targeted load-bearing evidence). PASS.

## 9. Verdict

**APPROVED** - no blockers, no rework required on `0861db0`/`4dc7237`.

**AR-20b is UNBLOCKED**: dispatch per `tasks/ar-20b-handoff.md` - lang-cpp
switch + scheduler-dep removal + m17 transitive guard landing GREEN-once at
`8 passed / 0 failed / 0 ignored / 1 suites` per the planner's census
correction (guard + planted-edge self-test; never RED in history). The
worker-core cone and shim contract it builds on are verified sound above;
AR-26 stays held until 20b lands.
