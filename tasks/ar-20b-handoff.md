# AR-20b Handoff - lang-cpp switch + scheduler dep removal + transitive guard GREEN-once (A4 part 2)

Successor of `tasks/ar-20a-handoff.md` (additive move + shims - must be
APPROVED and landed before dispatch; journal its landing hash). Shape ruling
`tasks/ar-20-shape.md` (C-lite) is binding: do not redesign it. Mechanism
falsification evidence (pre-existing, cite don't re-create): the AR-20 journal
RED run `6 passed / 1 failed / 0 ignored / 1 suites` naming all six offenders,
with the tree restored byte-identical (porcelain-empty) in `d63c0d0`/`7adbeca`
(`tasks/ar-20-handoff-journal.md` - READ-ONLY evidence, never edit it).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ar-20-shape.md` sections 3 (invariants 5, 6, 9), 4.3, 4.4, 6, 8.
3. `tasks/ar-20-kickoff.md` (invariants 5, 6; frozen locks).
4. `tasks/ar-20-handoff-journal.md` "TDD evidence" section - the guard's
   documented description + exact failure message to re-add byte-for-byte in
   spirit, and the RED citation for the before-evidence.
5. Mirror pattern: `tests/integration/tests/m17_plugin_seam_test.rs:402-421`
   (`guard_neutral_vocab_stripping_cannot_mask_plugin_names` - the non-vacuity
   self-test pattern to mirror).

## Objective

Repoint `crates/lang-cpp` at `ariadne-worker-core` (3 references), DELETE the
`ariadne-scheduler` dependency from lang-cpp, and land the transitive guard
`plugin_crates_transitive_tree_excludes_core_pipeline` + its planted-edge
self-test GREEN - in the SAME commit as the switch. The guard's only history
is your green landing: it must NEVER exist RED in history. Zero behavior
change; runtime call chain identical (backend -> worker-core function, static).

Invariants preserved (shape 3, numbered there):
5. Guard never RED in history: `git log -S plugin_crates_transitive_tree_excludes_core_pipeline`
   shows exactly ONE introducing commit = your landing; m17 suite green
   (see CENSUS CORRECTION below - expected `8 passed / 0 failed / 0 ignored / 1 suites`).
6. End-state cone: `python tools/distill.py -- cargo tree -p ariadne-lang-cpp`
   output contains none of `ariadne-scheduler` `ariadne-report` `ariadne-cluster`
   `ariadne-graph` `ariadne-transform` `ariadne-validate` (dev edges excluded;
   lang-cpp dev-deps are `serde_json`/`tempfile` only, so none of the six can
   appear anywhere in the output at all). Quote from own stdout (capture-race rule).
9. Non-vacuity: new self-test `transitive_guard_detects_planted_scheduler_edge`
   - the closure walker applied to an in-memory planted manifest string
   containing `ariadne-scheduler = { path = "../scheduler" }` must report the
   edge; green in-tree, deterministic; substitutes for the unobservable
   in-tree RED (whose evidence is the journal's `6 passed / 1 failed` run at
   `d63c0d0`/`7adbeca`, cited in your before-evidence).
1/2. m18 byte-parity + m00 golden UNEDITED byte-locks, green at landing
   (`23/0/0/1` and `4/0/0/2` baselines - re-measure at dispatch).
7. `git status --porcelain -- crates/lang-api` empty.

### CENSUS CORRECTION (planner note - machine-checked, adjudicate by enumeration)

Shape invariant 5 / kickoff acceptance pin `7 passed / 0 failed / 0 ignored /
1 suites` - that row counts the guard addition ALONE (the journal RED run
`6 passed / 1 failed` had 7 tests = 6 pre-existing + guard-only, authored
before invariant 9's self-test existed). This handoff lands TWO new tests, so
the green total is EIGHT: 6 pre-existing
(`m17_plugin_seam_test.rs` `#[test]` attrs at :44/:64/:238/:268/:402/:422) +
`plugin_crates_transitive_tree_excludes_core_pipeline` +
`transitive_guard_detects_planted_scheduler_edge`. The enumeration rule
governs: all eight named tests registered, 0 failed, 0 ignored. NEVER delete
or weaken a test to match 7; if the observed total differs, report.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ar-20b-handoff-journal.md` (checkpoint journal; always in-set)
- `crates/lang-cpp/Cargo.toml` - delete the `ariadne-scheduler = { path = "../scheduler" }`
  line (:23); add `ariadne-worker-core = { path = "../worker-core" }`; rewrite
  the M18.11 note (lines 11-15) to exactly this pure-ASCII block:
  `# AR-20 note: the backend implements ariadne_worker_core::WorkerBackend`
  `# over ariadne_worker_core::TuTask and calls the closure-merge engine`
  `# (ariadne_worker_core::tu_store) directly - the worker-core dep is why`
  `# the m17 transitive guard cone stays free of scheduler/report/cluster/`
  `# graph/transform/validate. Acyclic: worker-core never depends on any`
  `# plugin crate outside its own dev-deps (see crates/build/Cargo.toml).`
- `crates/lang-cpp/src/backend.rs` - EXACTLY three reference edits, zero logic
  edits, all else byte-unchanged:
  1. `:24`: `use ariadne_scheduler::{TuTask, WorkerBackend};` ->
     `use ariadne_worker_core::{TuTask, WorkerBackend};`
  2. `:180`: `ariadne_scheduler::analyze_tu_full_with_plugin(` ->
     `ariadne_worker_core::tu_store::analyze_tu_full_with_plugin(`
     (module-qualified, mirroring today's shape; the `_fingerprint` discard stays)
  3. module-doc `:19`: ``(`scheduler::tu_store::analyze_tu_full_with_plugin`)`` ->
     ``(`ariadne_worker_core::tu_store::analyze_tu_full_with_plugin`)``
  The module-doc header `:3-10` (M18.11 relocation history, incl.
  `scheduler::WorkerBackend`/`scheduler::TuTask` mentions) stays BYTE-UNCHANGED:
  it states the M18.11-era requirement and remains truthful as history; the
  manifest note rewrite above is the current-state account. Journal this choice.
- `tests/integration/tests/m17_plugin_seam_test.rs` - append
  `plugin_crates_transitive_tree_excludes_core_pipeline` (re-add of the
  journal's extension per its documented description: walks the local
  path-dependency closure of `crates/lang-cpp/Cargo.toml` and
  `crates/lang-scala/Cargo.toml` over the `[dependencies]` /
  `[build-dependencies]` tables, dev edges skipped, cycle-safe visited set,
  asserting none of scheduler/report/cluster/graph/transform/validate is
  reachable; mechanism mirrors `cargo tree` restricted to compiled edges) AND
  `transitive_guard_detects_planted_scheduler_edge` (invariant 9). File grows
  455 -> ~540 lines, well under the 1000 ceiling; keep helpers in-file.
- `Cargo.lock` (cargo regenerates it: the scheduler edge drops from lang-cpp)

You MUST NOT touch: `crates/scheduler/**`, `crates/worker-core/**` (byte-frozen
since the AR-20a landing - journal the 20a hash and prove
`git diff <20a-landing-commit> -- crates/worker-core crates/scheduler` EMPTY at
your landing), `crates/lang-api/**`, `crates/index/**`, `crates/build/**`,
`crates/lang/**`, `crates/pipeline/**`, `crates/cli/**`, `crates/ui/**`,
`crates/report/**`, `tests/golden/**`, `tests/integration/tests/**` except the
m17 file (m16/m13/m18*/m00/m12 + `common/**` are UNEDITED byte-locks),
`tasks/ar-20-handoff.md`, `tasks/ar-20-handoff-journal.md` (cite it; read-only),
`tasks/ar-20-shape.md`, `tasks/ar-20-kickoff.md`. No `git stash`, no branch change.

## Re-export shim rule (carry-forward - unchanged by this handoff, restated
so it cannot drift out of the chain)

AR-20a's shims are the compat contract and stay EXACTLY as landed:
`scheduler::tu_store::*` via lib.rs module alias + glob (lock:
m18_persistence_incremental_test.rs:226); flat names via glob; `queue::TuTask`
via `pub use ariadne_worker_core::task::TuTask;` (locks worker.rs:5, pool.rs);
`worker::WorkerBackend` via `pub use ariadne_worker_core::worker_backend::WorkerBackend;`
(lock engine.rs:21); `ariadne_scheduler::{TuTask, WorkerBackend, PrioritizedTuQueue,
WorkerPool, SharedPluginWorkerBackend, PluginWorkerBackend}` via the queue/worker
re-exports + kept modules (locks m16, m13, worker_backend_parity). After this
handoff lang-cpp is the ONLY consumer that names worker-core directly; every
other consumer still compiles against scheduler's preserved surface UNEDITED.

## RGB protocol (guard-green-once: history sees only GREEN)

1. Red (UNCOMMITTED, transient): append guard + self-test to the m17 file
   FIRST, with lang-cpp NOT yet switched. Run
   `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test`
   -> expect `7 passed / 1 failed / 0 ignored / 1 suites`: the self-test
   passes; the guard fails naming the six offenders, message mirroring the
   journal: `crates/lang-cpp transitively compiles against core-pipeline
   crates ["ariadne-scheduler", "ariadne-report", "ariadne-cluster",
   "ariadne-graph", "ariadne-transform", "ariadne-validate"] (plan rule R1
   must hold transitively)`. This reproduces the journal RED - capture quote +
   capture path in the journal. NEVER commit this state; the shared worktree
   must never see a red m17 suite (a red suite in a shared worktree breaks
   every lane's census - journal precedent).
2. Green: apply the three backend.rs repoints + the manifest swap in the SAME
   session; m17 now `8 passed / 0 failed / 0 ignored / 1 suites` (enumeration
   rule); all gates below green. Guard + self-test + dep removal + repoints
   are ATOMIC - they land in ONE commit whose tree is green; that commit is
   the guard's only history (shape 6 verdict).
3. Refactor: walker helper naming honest (`local_path_closure`, not `walk2`);
   helpers all in-file; doc comments on both new tests; re-run gates.

Red-equivalent for the guard's vacuity class (mutation proof, uncommitted):
temporarily restrict the walker to direct deps only (no closure descent) ->
the planted-edge self-test must FAIL ("planted scheduler edge not detected"
class message) -> restore byte-exact (`git diff` empty on the m17 file vs the
authored version) before committing. Mirrors the `:403` stripping-guard pattern.

## Hygiene (review-enforced)

Functions < 100 lines (guard + self-test + walker helper each well under);
files < 1000; m17 file 455 -> ~540; no banned names (`tmp`/`res`/`foo`/`bar`,
no single-letter locals); intention-revealing names; one responsibility per
function. `python tools/hygiene_check.py` exit 0 with ZERO blocking rows
(post-AR-13 baseline: the "2 server.rs rows" standing exception died at AR-13
landing `14e21b1`; zero new from your files).

## Frozen locks (verify + quote)

- m18 byte-parity UNEDITED: m18 suite files `git status --porcelain` empty;
  suite `23 passed / 0 failed / 0 ignored / 1 suites` green.
- m00 golden UNEDITED: `git status --porcelain -- tests/golden` empty;
  `4 passed / 0 failed / 0 ignored / 2 suites` green.
- m17 seam type-identity lock `lang_api_types_are_the_same_types_index_re_exports`
  green untouched (inside the same m17 run).
- worker-core/scheduler byte-frozen at the AR-20a landing (diff empty, above).
- `tasks/ar-20-handoff-journal.md` porcelain-empty.

## Gates (exact commands, timeout 600000 ms each; scoped only - workspace-wide
final gates belong to the reviewer's acceptance pass)

1. `python tools/distill.py -- cargo check --workspace --all-targets` -- green
   before any suite (STRICT COMPILE-GREEN; capture `target/distill-capture.txt`).
2. `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test`
   -- red trail first (`7 passed / 1 failed / 0 ignored / 1 suites`,
   uncommitted), then green `8 passed / 0 failed / 0 ignored / 1 suites` per
   the enumeration rule.
3. `python tools/testrun.py --distill -p ariadne-lang-cpp` -- `153 passed /
   0 failed / 0 ignored / 2 suites` maintained, zero test edits.
4. `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` -- `23 passed / 0 failed / 0 ignored / 1 suites` UNEDITED.
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test` -- `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED.
6. `python tools/distill.py -- cargo tree -p ariadne-lang-cpp` -- stdout
   contains NONE of the six forbidden crate names (invariant 6); quote from
   own stdout + journal the capture path.
7. `git log -S plugin_crates_transitive_tree_excludes_core_pipeline --oneline`
   -- exactly ONE line, = your landing commit (run AFTER committing; journal it).
8. `python tools/hygiene_check.py` -- exit 0, zero blocking rows (post-AR-13
   baseline); `python tools/encoding_guard.py --staged` -- exit 0 before
   committing (+ `--all` in the journal).

Cheap non-gate evidence (journal): the mutation proof above;
`git status --porcelain -- crates/lang-api crates/scheduler crates/worker-core`
empty; lang-scala manifest untouched (guard's second subject must already pass).

## Discipline clauses (shared with AR-20a)

- STRICT COMPILE-GREEN (shared worktree): the switch + dep removal + guard are
  ONE commit - no intermediate tree is ever committed non-compiling or with a
  red suite. If a workspace check transiently trips on foreign uncommitted
  dirt, journal the error attribution verbatim, verify your own cone green,
  do NOT fix or stage foreign files.
- UTF-8: never Set-Content/Out-File/`>`/`>>`/Get-Content-replace; Write/Edit
  tools only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
  backend.rs and its manifest note contain pre-existing non-ASCII prose -
  keep every edit byte-local to its target line (the journal's
  Edit-round-trip LF-slip precedent). `encoding_guard --all` is the oracle.
- Capture paths in the journal: every gate run records its canonical line
  quoted from OWN stdout + `target/testrun-capture.txt` /
  `target/distill-capture.txt` path (shared capture races - board-ruled).
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix.
- No raw high-volume commands in your context: everything through the
  distilling entry points (`docs/specs/tool-output-distillation.md`).
- After EACH Green gate, append a 1-3 line checkpoint to
  `tasks/ar-20b-handoff-journal.md`: module + commit hash, deviations,
  lock/contract findings (incl. the RED trail + guard failure message verbatim).
  If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ar-20b-handoff.md ...`.
- Commit: `python tools/commit_scoped.py --scope "crates/lang-cpp/**"
  --scope "tests/integration/tests/m17_plugin_seam_test.rs" --scope "Cargo.lock"
  --scope "tasks/ar-20b-handoff-journal.md" -m "feat(lang-cpp): consume
  ariadne-worker-core directly; drop scheduler dep; land transitive guard GREEN
  (AR-20b)"` (never `git add .`). A second journal-only commit for the hash
  record is the AR-16/AR-19 precedent.

## Acceptance (reviewer re-runs these fresh)

All 8 gate commands exit green with the pinned canonical lines, plus:
- `git log -S <guard fn name>` shows exactly ONE introducing commit
  (= the landing) - invariant 5, machine-verified.
- `cargo tree -p ariadne-lang-cpp` free of all six names - invariant 6.
- m17 green line matches the 8-test enumeration (CENSUS CORRECTION section);
  planted-edge self-test present, green, and mutation-proven load-bearing
  (journal trail).
- m18/m00/lang-api/scheduler/worker-core porcelain-empty vs their anchors.
- Expected verdict lines: gates 2-5 canonical summary lines verbatim;
  gate 7 single commit line; encoding exit 0; hygiene exit 0 zero-blocking.

## Dependencies / serialization

- Dispatch ONLY after AR-20a is APPROVED and landed (journal its hash).
- AR-26's scheduler-side reparent handoff stays HELD until this handoff
  lands (shape 7, stated both directions: AR-20b's guard is the precondition
  for AR-26's "smaller plugin cone after AR-20" compounding claim; declare
  AR-26 rebase-on-20b - shared 1-line surfaces are root `Cargo.toml` members
  and scheduler `Cargo.toml`).
- The lang-family queue serializes AFTER this landing: AR-24 ->
  AR-25/AR-21/AR-31 (lang-cpp is this lane's until your commit lands).
- Do not touch `crates/lang/**` or `crates/scheduler/**` - reserved for the
  AR-26 rebase lane.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill (fresh full verification, per-commit `git show` oldest-first,
commit-vs-touch-set enforcement against the list above - it includes
`Cargo.lock` precisely because cargo regenerates it; the reviewer also
re-derives the 8-test enumeration and treats the CENSUS CORRECTION as
governing over the shape's stale `7 passed` line).
