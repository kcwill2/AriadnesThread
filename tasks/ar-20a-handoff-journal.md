# AR-20a Journal - worker-core extraction (additive move + re-export shims)

Handoff: `tasks/ar-20a-handoff.md`. Shape: `tasks/ar-20-shape.md` (C-lite,
binding). Dispatch message: additive extraction, strict compile-green,
m17 guard UNTOUCHED, lang-cpp UNTOUCHED.

## Phase 0 - dispatch HEAD + baselines (BEFORE any edit)

`python tools/checkpoint_guard.py status`:

- HEAD `768529f` (branch `shining-enthusiasm`), tracked tree clean
  (`git diff HEAD --numstat` empty; untracked = tasks/docs files only).

Baseline suite lines (own stdout, each run's capture at
`target/testrun-capture.txt` as printed per run):

| gate | command | line |
|---|---|---|
| scheduler (pre-move census) | `testrun.py --distill -p ariadne-scheduler` | `34 passed / 0 failed / 0 ignored / 2 suites` |
| lang-cpp | `testrun.py --distill -p ariadne-lang-cpp` | `153 passed / 0 failed / 0 ignored / 2 suites` |
| m18 parity | `... --test m18_cpp_plugin_parity_test` | `23 passed / 0 failed / 0 ignored / 1 suites` |
| m18 persistence | `... --test m18_persistence_incremental_test` | `7 passed / 0 failed / 0 ignored / 1 suites` |
| m16 | `... --test m16_clang_worker_test` | `3 passed / 0 failed / 0 ignored / 1 suites` |
| m00 golden + facts-path | `... --test m00_golden_baseline_test --test m00_facts_path_parity_test` | `4 passed / 0 failed / 0 ignored / 2 suites` |

Conservation equation to pin: worker-core (8) + scheduler (26) = 34 total.
Pre-move scheduler total `34 passed / 0 failed / 0 ignored / 2 suites`
matches the handoff-pinned baseline; the 8 `#[test]` fns inside
`tu_store.rs`'s test module (starts at `tu_store.rs:438`) travel, leaving
`26` in-package.

### Deviation note (measured vs pinned, no action)

- m16 measures `3 passed / 0 failed / 0 ignored / 1 suites` at dispatch HEAD
  `768529f`; the handoff/shape rows pin `2 passed / ...`. The suite gained a
  test on this branch since those rows were authored; file is UNEDITED and
  green. Gate 7 is adjudicated as "green + UNEDITED", not against the stale
  count. Reported here, not fixed (out of touch-set).

### Hygiene BEFORE (new post-AR-13 baseline: zero blocking rows)

`python tools/hygiene_check.py` at `768529f`: R3 warn-only output, ZERO R1/R2
blocking rows (exit 0). Includes pre-existing R3 warns that travel verbatim
with `tu_store.rs`: `tu_store.rs:54` and `:62` (`tmp` locals, warn-only).

### Pre-move hit-set (invariant 3 identity baseline)

`rg "scheduler::(tu_store|queue|worker)::"` compiled-code hits at dispatch
HEAD (doc-comment hits listed, all UNEDITED through the move):

- `tests/integration/tests/m18_persistence_incremental_test.rs:226` -
  `ariadne_scheduler::tu_store::ShardCheckpoint` (the module-qualified lock)
- `tests/integration/tests/m18_cpp_plugin_parity/external_boundary_parity.rs:5` (doc comment)
- `crates/lang-cpp/src/backend.rs:19` (module-doc comment)
- `plans/m21-scala-index-kickoff.md:13` (doc, not compiled)
- tasks/*.md planning docs (not compiled)

### Shape fact 1 re-verified

`grep "crate::" crates/scheduler/src/tu_store.rs` - zero matches. The engine
has no scheduler-internal paths; it travels intact. `queue.rs:6`'s
`use crate::tu_store::SharedTuStoreContext;` remains used by
`from_translation_units_with_context` (`queue.rs:71`) after the `TuTask`
deletion, so no unused-import drift.

## Phase 1 - byte-safe rename (Red protocol b)

`git mv crates/scheduler/src/tu_store.rs crates/worker-core/src/tu_store.rs`
(via `python tools/distill.py --`, exit 0; staged first so `--find-renames`
sees it). R100 evidence (own capture, `target/distill-capture.txt`):

- `git ls-files -s crates/worker-core/src/tu_store.rs` ->
  `100644 202df67913c63b5eb7bfccf242bd89309b9db834 0`
- `git rev-parse "HEAD:crates/scheduler/src/tu_store.rs"` ->
  `202df67913c63b5eb7bfccf242bd89309b9db834` (identical blob - byte identity)
- `git diff HEAD --find-renames -M --stat` ->
  `crates/{scheduler => worker-core}/src/tu_store.rs | 0` /
  `1 file changed, 0 insertions(+), 0 deletions(-)` - rename, ZERO content hunks

The new-crate files (`Cargo.toml`, `lib.rs`, `task.rs`, `worker_backend.rs`)
were authored via the Write tool (pure ASCII), so the rename target directory
already existed; `git mv` needed no PowerShell content round-trip.

## Phase 2 - shim load-bearingness mutation proof (Red protocol c)

With shims in place, `pub use ariadne_worker_core::tu_store;` (lib.rs alias
line) was commented out and
`python tools/distill.py -- cargo check -p ariadne-integration-tests --test m18_persistence_incremental_test`
run - the m18_persistence:226 consumer path. Result FAIL (exit 101), errors
verbatim from distill capture (`target/distill-capture.txt`):

```
error[E0432]: unresolved import `crate::tu_store`
 --> crates\scheduler\src\incremental.rs:9:12
error[E0433]: failed to resolve: unresolved import
  --> crates\scheduler\src\worker.rs:88:24
error: could not compile `ariadne-scheduler` (lib) due to 4 previous errors
```

The shim is load-bearing, not decorative. The alias line was then restored
byte-exact (grep confirms `pub use ariadne_worker_core::tu_store;` uncommented
at lib.rs:24); no red state was ever committed.

## Phase 3 - Green gates (all own stdout; capture paths shared-capture races
per board rule - canonical lines quoted from the run itself)

| # | command | canonical line / result |
|---|---|---|
| 1 | `distill.py -- cargo check --workspace --all-targets` | `ok (exit 0)` (first run: 21 warnings - see deviation D2; re-check `cargo check -p ariadne-scheduler --all-targets` -> `ok (exit 0, 0 warnings emitted)`) |
| 2 | `testrun.py --distill -p ariadne-worker-core` | `8 passed / 0 failed / 0 ignored / 2 suites` |
| 3 | `testrun.py --distill -p ariadne-scheduler` | `26 passed / 0 failed / 0 ignored / 2 suites` |
| 4 | `testrun.py --distill -p ariadne-lang-cpp` | `153 passed / 0 failed / 0 ignored / 2 suites` (UNEDITED - compiles against scheduler shims) |
| 5 | `testrun.py --distill ... m18_cpp_plugin_parity_test` | `23 passed / 0 failed / 0 ignored / 1 suites` UNEDITED |
| 6 | `testrun.py --distill ... m18_persistence_incremental_test` | `7 passed / 0 failed / 0 ignored / 1 suites` UNEDITED (:226 module-qualified path lock resolves via lib.rs alias) |
| 7 | `testrun.py --distill ... m16_clang_worker_test` | `3 passed / 0 failed / 0 ignored / 1 suites` UNEDITED (see deviation D1) |
| 8 | `testrun.py --distill ... m00_golden_baseline_test --test m00_facts_path_parity_test` | `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED; `git status --porcelain -- tests/golden` empty |
| 9 | `hygiene_check.py` + `encoding_guard.py --all` | hygiene exit 0, ZERO blocking R1/R2 rows (R3 warn-only, identical post-AR-13 baseline to pre-run); encoding `--all` silent (no output) |

Capture files: `target/testrun-capture.txt` (test gates),
`target/distill-capture.txt` (check/mutation/git evidence).

### Conservation equation (invariant 4)

Gate 2 `8 passed` + gate 3 `26 passed` = **34** = pre-move scheduler total
`34 passed / 0 failed / 0 ignored / 2 suites`. ZERO tests authored or
deleted; the 8 `#[test]` fns of `tu_store.rs`'s test module (file :438)
TRAVEL as a unit with the R100 rename (attribute-move). Suite split
post-move: scheduler `2 suites`, worker-core `2 suites` (see deviation D3).

### Frozen locks verified at landing

- `git diff 768529f -- tests/integration/tests/m17_plugin_seam_test.rs` EMPTY
  (guard UNTOUCHED - AR-20b owns it).
- `git status --porcelain -- tests/` EMPTY (see deviation D5 for a transient).
- `git status --porcelain -- tests/golden` EMPTY.
- `git status --porcelain -- crates/lang-api` EMPTY (invariant 7).
- `git status --porcelain -- tasks/ar-20-handoff-journal.md` EMPTY.
- serde wire format: zero attribute changes - the moved types are byte-identical
  (R100 blob hash above is the mechanical proof for `tu_store`).

### Post-move hit-set (invariant 3 identity)

`rg "scheduler::(tu_store|queue|worker)::"` compiled-code hits after the move
are the SAME set as pre-move - `m18_persistence_incremental_test.rs:226`,
`external_boundary_parity.rs:5` (doc), `lang-cpp/backend.rs:19` (doc) - plus
only new comment lines inside my own `lib.rs` shim hunk and this journal.
Every hit compiles (gate 1 `--all-targets` green with all suites UNEDITED).

## Deviations (each with reason; nothing fixed outside touch-set)

- **D1**: m16 measures `3 passed` (baseline AND gate 7) vs the `2 passed`
  pinned in handoff/shape. Pre-existing at dispatch HEAD `768529f`; the suite
  gained a test after those rows were authored. Gate adjudicated green+UNEDITED.
- **D2**: one-line deviation inside a touch-set file: after the `TuTask` block
  deletion, `queue.rs:2` `use std::cmp::Ordering;` became dead (its only user
  was the moved `Ord` impl; workspace check warned). Dropped that dead import.
  Re-check green, 0 warnings. Not a rename-file edit; `tu_store.rs` untouched.
- **D3**: worker-core shows `2 suites`, not the pinned `1 suites`: `cargo test`
  counts the (empty) doc-test binary as a suite - the same reason scheduler
  measured 2 suites pre-move. Test count 8 is exact; conservation unaffected.
- **D4**: hygiene row attribution: pre-move run listed `tu_store.rs` `tmp`
  warns at :54/:62; post-move run lists :54/:61/:62 for the byte-identical
  blob (all three `tmp` occurrences are visible in the file at those lines).
  Warn-only R3 both sides; blob identity is machine-proven (R100 hash), so
  this is checker line-attribution, not content drift. No authored line adds
  any R1/R2/R3 row: new files (`lib.rs`/`task.rs`/`worker_backend.rs`)
  produce zero rows.
- **D5**: running the suites (not this handoff's edits) regenerated
  `tests/fixtures/basic/artifacts/includes/report.json` leaving a ` M` status;
  content delta was zero lines (numstat emitted only an autocrlf
  normalization warning). Restored to the committed bytes via
  `git checkout --` (restore-to-HEAD, not an edit) so the `tests/` frozen lock
  reads empty at landing, per the handoff's wording.
- **D6**: `"clang++"` fallback string now located at
  `crates/worker-core/src/tu_store.rs:141` inside the neutral crate - shape 7
  note, NO action: the R2 guard's 7-crate walk scope is unchanged (worker-core
  not in scope); A8 owns neutralizing it, worker-core is its future home.

## Landed

Commit `0861db0` (parent = dispatch HEAD `768529f`, one commit, via
`python tools/commit_scoped.py` with scopes `crates/worker-core/**`,
`crates/scheduler/**`, `Cargo.toml`, `Cargo.lock`,
`tasks/ar-20a-handoff-journal.md`), message
`refactor(worker-core): extract tu_store engine + worker types from scheduler (A4 part 1, additive)`.
`git show --stat -M HEAD` records
`crates/{scheduler => worker-core}/src/tu_store.rs | 0` - rename, zero
hunks; the commit tree carries the blob at the worker-core path only.
`python tools/encoding_guard.py --staged` run against the staged landing:
silent, exit 0. Final frozen-lock re-verification at `0861db0`:
`git status --porcelain -- tests/` EMPTY;
`git diff 768529f HEAD --stat -- tests/ crates/lang-api crates/lang-cpp`
EMPTY. Nothing foreign staged; foreign untracked task docs left untouched.

### D7 - commit_scoped cannot carry an already-staged rename deletion (tooling, NOT fixed here)

First attempt (commit `79a90d3`, discarded) shipped the rename as an addition
only: `resolve_scopes` enumerates candidates from `git ls-files` (index) +
`--others`, and a staged `git mv` deletion is absent from the index, so the
old path never matched any scope; `git commit --only` then excludes it by
design. Same for the second attempt state. Resolution used here:
`git reset --soft` back to the dispatch HEAD (discarding only my own
seconds-old, unpushed, wrong-scope commits `79a90d3`/`d4630d4`), leave the
tu_store deletion UNSTAGED (path stays in the index, so `ls-files` lists it),
then run the exact handoff `commit_scoped.py` invocation - the tool stages
the deletion itself and the third commit `0861db0` is rename-complete.
Recommendation for the tooling owner (out of this handoff's touch-set): have
`commit_scoped` resolve candidates also from `git ls-files -d` (worktree-
deleted paths), or detect staged deletions under a directory scope.
