# AR-20b Verdict - lang-cpp switch + scheduler dep removal + transitive guard GREEN-once (A4 part 2)

Reviewer: review-gate skill, fresh full verification, per-commit `git show`
oldest-first, commit-vs-touch-set enforcement against
`tasks/ar-20b-handoff.md`. Landing `dcfc813`, addendum `f49faeb`,
base (dispatch anchor) `4dc7237`, AR-20a anchor `0861db0`.

## VERDICT: APPROVED

A4 finding CLOSED machine-locked. AR-26 scheduler-side reparent hold LIFTED
(its precondition guard is machine-true in history and in-tree). Lang-family
queue may drain: AR-24 dispatch next (then AR-25/AR-21/AR-31).

## 1. Ground truth (checkpoint_guard status)

HEAD `f49faeb`; branch `shining-enthusiasm`; recent chain matches the journal
exactly (`768529f` -> `0861db0` -> `4dc7237` -> `dcfc813` -> `f49faeb`).
Dirty tracked files are exclusively `crates/ui` (AR-15 implementer in flight -
foreign, not adjudicated here). Worktree census clean-formal; no anomaly.

## 2. Commit-vs-touch-set - PASS

`dcfc813` numstat: `Cargo.lock 1/1`, `crates/lang-cpp/Cargo.toml 7/6`,
`crates/lang-cpp/src/backend.rs 3/3`, `tasks/ar-20b-handoff-journal.md 182/0`,
`tests/integration/tests/m17_plugin_seam_test.rs 207/0`. All five paths inside
the declared touch-set (Cargo.lock named precisely because cargo regenerates
it; journal always in-set; m17 the single named test exception).
`f49faeb`: `tasks/ar-20b-handoff-journal.md 24/0` only - the journal-only
second commit explicitly admitted by the handoff (AR-16/AR-19 precedent).
Zero paths outside the touch-set in either commit.

## 3. GREEN-once (invariant 5) - PASS, both forms

- `git log -S plugin_crates_transitive_tree --oneline -- tests/` -> EXACTLY
  ONE line: `dcfc813 feat(lang-cpp): consume ariadne-worker-core directly;
  drop scheduler dep; land transitive guard GREEN (AR-20b)`.
- Whole-repo form adds two hits, both proven docs-only:
  - `d63c0d0` - full numstat is `138 0 tasks/ar-20-handoff-journal.md` only;
    `git show "d63c0d0" --stat -- tests/integration/tests/m17_plugin_seam_test.rs`
    is EMPTY (m17 byte-unchanged there; the mention is the cited AR-20
    journal's description text - matches the journal's claim exactly).
  - `f49faeb` - the AR-20b addendum itself (journal-only).
  So the guard's test code exists in exactly one commit's tree-history and was
  never RED in committed history.

## 4. Guard body audit (GREEN-by-construction weakness scan) - PASS

- `TRANSITIVE_FORBIDDEN_DEPS` (:462-469): exactly the six names, no typos:
  scheduler/report/cluster/graph/transform/validate.
- Walker `collect_reachable_local_deps` (:567-590): real DFS descent into each
  local-path dep manifest (:583-588); cycle-safe visited set keyed by
  `normalize_manifest_path` (textual `..` collapse, backslash-normalized).
- `parse_local_path_deps` (:596-630): reads ONLY `[dependencies]` /
  `[build-dependencies]`; `[dev-dependencies]`, target-scoped tables, and all
  other sections skipped - per the handoff's documented description; dev
  exclusion is LOCKED BY ASSERTION in the self-test (:541-544), not silent.
- Absent-manifest `continue` in the guard (:487-488) is documented ("asserts
  the moment a plugin crate appears"); both subjects exist at landing, so the
  walk is non-vacuous. Machine twin: my fresh `cargo tree` matches the
  assertion exactly (section 7).
- Failure message (:496-499) reproduces the handoff-pinned shape
  `{plugin_crate} transitively compiles against core-pipeline crates
  {found:?} (plan rule R1 must hold transitively)` - matches the journal's
  verbatim RED quote modulo the found list, in forbidden-list order.

## 5. Planted-edge self-test non-vacuity (invariant 9) - PASS

`transitive_guard_detects_planted_scheduler_edge` (:512-545): injects only a
manifest-reader seam; root manifest -> `planted-intermediate` -> 
`ariadne-scheduler = { path = "../scheduler" }` two hops down. Detection runs
through the SAME `reachable_local_crate_names` / `collect_reachable_local_deps`
/ `parse_local_path_deps` code the guard uses - not a hardcoded expectation.
Mutation coherence: with the descent call removed, `reachable` =
`["planted-intermediate"]` only, so the assert fails with exactly the
journal-quoted message `planted scheduler edge not detected through closure
descent: ["planted-intermediate"]` - the committed format string reproduces
the trail verbatim. The committed file (the restored state) contains the
descent at :583-588. Second assert pins dev-skip. Pattern mirrors
`guard_neutral_vocab_stripping_cannot_mask_plugin_names` (:402-420) per
contract.

## 6. Switch completeness (plan-literal) - PASS

- backend.rs: EXACTLY 3 reference edits, 3/3 - module-doc :19, import :24,
  module-qualified engine call :180 (`ariadne_worker_core::tu_store::
  analyze_tu_full_with_plugin(` with `_fingerprint` discard kept), all three
  byte-exact per handoff items 1-3; module-doc header :3-10 untouched; zero
  logic edits.
- Manifest: `ariadne-scheduler = { path = "../scheduler" }` GONE; no scheduler
  entry in ANY table (full 28-line file read: dependencies + dev-dependencies
  only; dev-deps exactly serde_json/tempfile); `ariadne-worker-core =
  { path = "../worker-core" }` in its slot; note block rewritten byte-for-byte
  to the handoff's ASCII block (6 lines, verified line-by-line against
  handoff lines 69-74; the old en-dash prose left with the deleted lines).
- Cargo.lock: the single lang-cpp edge `ariadne-scheduler` -> 
  `ariadne-worker-core` - nothing else.
- Workspace grep `crates/lang-cpp`: no `use ariadne_scheduler` and no code
  reference anywhere; two residual hits are pre-existing module-doc HISTORY
  prose (backend.rs:3 M18.11 relocation - ruled kept; lib.rs:8 historical
  relocation, file outside touch-set, correctly untouched).

## 7. Frozen locks - PASS

- `git diff --stat "0861db0..HEAD" -- crates/worker-core crates/scheduler`
  EMPTY (shims byte-frozen since AR-20a landing).
- `git show "dcfc813" -- crates/lang-api tests/golden
  tasks/ar-20-handoff-journal.md ...m18/m18_persistence paths` -> EMPTY; and
  `--name-only -- tests/integration/tests/` -> ONLY `m17_plugin_seam_test.rs`,
  so m18/m18_persistence/m00 are UNEDITED in the landing commit; `f49faeb`
  touches only the journal.
- `tasks/ar-20-handoff-journal.md`: tracked, not in the dirty list -
  porcelain-empty.
- m18_persistence `:226` shim lock resolving through the 20a module alias:
  green (7/0/0/1), file byte-identical.

## 8. Deviation adjudication (independent, on merits)

- D1 (planted fixtures as `concat!` single-line literals): ACCEPTED. The
  scanner misparse is real tooling behavior (the Rust hygiene scanner resets
  in-string state at physical EOL, so brace-bearing continuation lines parse
  as code -> false blocking `unclosed fn`). Semantics unchanged: `concat!`
  concatenation is compile-time, the runtime manifest bytes asserted are
  identical; suite green; mutation trail message still exact. My fresh hygiene
  run confirms the effect: m17 appears in ZERO rows. Root cause goes to the
  ledger as a tooling row (section 10, F2). No revert required.
- D2 (`--staged` run after the scoped commit, `--all` silent before):
  ACCEPTED on substance. Raw `git add` off the session allowlist makes
  pre-stage `--staged` mechanically infeasible with atomic `commit_scoped`;
  the committed bytes were independently verified by MY fresh
  `encoding_guard.py --all` (silent) on the landing+addendum tree - the
  superset oracle over exactly the content `--staged` would have covered.
- D3 (commit message = handoff's verbatim `-m` string): ACCEPTED. Verified
  `git show dcfc813` header matches the contract command verbatim.
- Journal-disclosed estimate drift: handoff said m17 grows to "~540" lines;
  actual 662 (helpers + doc comments). Disclosed in the journal; the binding
  ceiling is <1000; informational only (F1).

## 9. Fresh gates (reviewer's own stdout, verbatim)

1. m17: `8 passed / 0 failed / 0 ignored / 1 suites` (capture
   `target/testrun-capture.txt`). Enumeration re-derived: exactly 8 `#[test]`
   attrs at :44/:64/:238/:268/:402/:422 + guard :481 + self-test :512; the
   CENSUS CORRECTION governs over the shape's stale `7 passed` row - observed
   total MATCHES eight; no test deleted or weakened.
2. `-p ariadne-lang-cpp`: `153 passed / 0 failed / 0 ignored / 2 suites`.
3. m18 parity: `23 passed / 0 failed / 0 ignored / 1 suites`;
   m18_persistence: `7 passed / 0 failed / 0 ignored / 1 suites`;
   m00 pair: `4 passed / 0 failed / 0 ignored / 2 suites` - all UNEDITED.
4. `python tools/distill.py -- cargo tree -p ariadne-lang-cpp` ->
   `ok (exit 0, 0 warnings emitted)`; own-capture grep of the six forbidden
   names: **No files found**; ariadne edge set is exactly build/lang/
   lang-api/repo/worker-core (:50) + index (:53, under worker-core).
5. `-p ariadne-scheduler`: `26 passed / 0 failed / 0 ignored / 2 suites`;
   conservation rider `-p ariadne-worker-core`: `8 passed / 0 failed /
   0 ignored / 2 suites` -> 26 + 8 = 34 unchanged (AR-20a census row).
6. `hygiene_check.py`: output is R3 warn-only over the FULL output - ZERO R1/
   R2/ERROR blocking rows; none of the AR-20b files appears (post-AR-13
   zero-blocking baseline held). `encoding_guard.py --all`: SILENT.
   `python -m unittest discover -s tools -p "*_tests.py"`:
   `Ran 219 tests in 20.081s` / `OK`.
7. Gate 1 (`cargo check --workspace --all-targets`) and the workspace-wide
   baseline/smoke gates: NOT re-run here - the tracked tree carries AR-15's
   in-flight `crates/ui` edits; the review-gate skill assigns the workspace
   final gate to the orchestrator, and the dispatch states ui-class gates are
   outside this contract. This lane's cone is compile-green by gates 1-5
   building and passing. See `tasks/ar-20b-execution-results.md` for the
   explicit not-run ledger.

## 10. Findings (numbered; none blocking)

1. (minor, informational) `tests/integration/tests/m17_plugin_seam_test.rs`
   grew to 662 lines against the handoff's "~540" estimate - disclosed in the
   journal, inside the 1000 ceiling, all new functions far under 100 lines.
   No action.
2. (tooling row -> `tasks/milestone_ledger.md` deferred-findings)
   `tools/hygiene_check.py` Rust scanner misparses EOL-backslash string
   continuations (in-string state resets at physical EOL; brace-bearing
   continuation text parses as code -> false blocking `unclosed fn`). Root
   cause of AR-20b D1; workarounds exist (`concat!`), fix is in the scanner.
3. (minor, informational) The walker skips `[dependencies.<name>]`
   table-syntax and target-scoped tables - plan-literal per the handoff's
   pinned table set and the repo's universal inline-table convention; a future
   manifest written in table syntax would be invisible to the guard. Record,
   do not change (changing it would exceed the contract).
4. (configuration note, not an implementer finding) This reviewer session's
   sandbox allowlist refuses exit-code capture forms (`$LASTEXITCODE`,
   `cmd /c`, `python -c`) with the catch-all deny boilerplate
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`;
   hygiene/encoding EXIT codes are therefore evidenced by full-output
   inspection (zero blocking rows / silent pass signature) rather than a
   captured code. All contract gates themselves ran and are quoted verbatim.

## 11. Report-evidence check (distillation contract)

Journal is digest-shaped: canonical lines quoted from own stdout, capture
paths recorded (`target/testrun-capture.txt`, `target/distill-capture.txt`),
exit codes noted, zero raw dumps; gate commands use `testrun.py --distill` /
`distill.py -- ...`. PASS.

## Dispositions

- A4 (lang-cpp -> scheduler transitive violation): CLOSED, machine-locked by
  the green-once guard + planted-edge self-test + cargo-tree twin.
- AR-26 scheduler-side reparent: hold LIFTED; rebase-on-20b declared; shared
  1-line surfaces (root `Cargo.toml`, scheduler `Cargo.toml`) unchanged here.
- Lang queue: AR-24 dispatch next; AR-25/AR-21/AR-31 follow.
- F2 tooling row: orchestrator to record in the ledger (reviewer may not edit
  the ledger).
