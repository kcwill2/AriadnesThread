# AR-26b Handoff - Scheduler reparent to `ariadne-artifact` + m17 cone-guard extension (A3 chain, hop 3/3)

Successor of `tasks/ar-26a-handoff.md` (crate + manifests) and
`tasks/ar-26a2-handoff.md` (atomic manager move); dispatch ONLY after both
are APPROVED and landed. Evidence base: `tasks/ar-26-handoff-journal.md`
F5 (scheduler consumption is ONE call site, `incremental.rs:23`) and F6
(guard family file). Do not re-measure what that journal numstat-proves.

> RESOLVED 2026-10-11 - ARCHITECT SHAPE RULING `tasks/ar-26b-shape-ruling.md`
> (read it first; binding for this dispatch). Option A is the ONLY
> dispatchable shape (Option B killed: DTO relocation/feature-gating/finer
> splits all violate VERBATIM or the m17 walker's semantics). This hop
> machine-locks EXACTLY (Ruling 1):
> `"ariadne-report" NOT-IN Cone(scheduler)` (m17
> `reachable_local_crate_names` walk) AND `"ariadne-report" NOT-IN
> Tree(scheduler)` (`cargo tree -p ariadne-scheduler -e normal`), both
> mechanisms required, exact-token grep (precision note P1, gate 6).
> The guard wording below is RATIFIED verbatim (Ruling 4):
> `SCHEDULER_CONE_FORBIDDEN: [&str; 1] = ["ariadne-report"]` + planted-edge
> non-vacuity + guard-proper RED before the swap. `ariadne-cluster`,
> `ariadne-transform`, `ariadne_validate` are EXPECTED cone members through
> `scheduler -> artifact` (P2): describe them as expected in the journal,
> NEVER assert them absent in any const, gate, or amendment - cone content
> is byte-identical to HEAD (report dragged them before; artifact does
> after). Artifact's cluster/transform/validate path deps are MANDATORY
> (Ruling, evidence section): no implementer may "clean up" or
> feature-hide them to strengthen any assertion. The 4-crate exclusion is
> NOT a follow-up AR (P3, Ruling 3): the residual is a DEFERRED LEDGER ROW
> at orchestrator closeout carrying the AR-26c promotion charter verbatim;
> audit A3 is PARTIALLY satisfied (Ruling 2). Dispatchable once 26a and
> 26a2 are APPROVED and landed.

## Objective

Reparent `ariadne-scheduler` from `ariadne-report` to `ariadne-artifact`:
swap the ONE call site and the ONE manifest edge, and make the m17 guard
honest in AR-20b's own style - extend it FIRST (RED while scheduler still
pulls report), machine-lock the scheduler cone, GREEN after the swap.
Also close the two guard-scope gaps the extraction opened: walk
`crates/artifact` under the R2 core-crates guard, and add
`ariadne-artifact` to the plugin transitive-forbidden list. ZERO test
edits: scheduler's 26 and worker-core's 8 tests conserve exactly.

## Baselines (dispatch-HEAD = AR-26a2 landing; re-measure and journal BEFORE
any mutation)

- scheduler `26/0/0/2`; ariadne-worker-core `8/0/0/2`; report `7/0/0/3`
  (post-26a2); artifact `40/0/0/2`; m17 `8/0/0/1`; m00 `4/0/0/2`;
  cli `118/0/8/7`; lang-cpp `155/0/0/2`.
- hygiene ZERO blocking rows. Census post-26a2 state (reviewer pass;
  passed totals conserved at `1397`).

## Per-hop reconciliation (what this hop reconciles)

- m17: `8 -> 10` passed (`8/0/0/1 -> 10/0/0/1`): +1 cone guard, +1 planted
  non-vacuity self-test. Census passed total `1397 -> 1399`, suite count
  unchanged (same test binary).
- scheduler `26/0/0/2` and worker-core `8/0/0/2` conserve UNEDITED - the
  conservation equation for the whole chain closes here.
- No suite relocates this hop; moved suites already reconcile under
  `-p ariadne-artifact` (see 26a2).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ar-26b-handoff-journal.md` (checkpoint journal; always in-set)
- `tests/integration/tests/m17_plugin_seam_test.rs` (guard extension ONLY:
  the two new tests + two const-array additions below; every existing test
  body byte-untouched)
- `crates/scheduler/src/incremental.rs` (ONE line - `:23` import swap, see
  exact edit)
- `crates/scheduler/Cargo.toml` (remove line 16
  `ariadne-report = { path = "../report" }`, add
  `ariadne-artifact = { path = "../artifact" }`; comment in the swap's
  place: `# AR-26b: artifact-store IO consumed directly (was via report).`)
- `Cargo.lock` (cargo regenerates)

You MUST NOT touch: `crates/report/**`, `crates/artifact/**`,
`crates/cli/**`, `crates/pipeline/**`, `crates/lang-cpp/**`, `tests/**`
except the m17 file, `crates/ui/**`, `crates/worker-core/**`,
`tasks/ar-26-handoff.md`, `tasks/ar-26-handoff-journal.md`. No `git stash`,
no branch change.

## What to build (exact literals)

1. m17 guard extension, appended as a new section after section 4, same
   file style (doc comments explaining mechanism + evidence):

   - `#[test] fn scheduler_cone_excludes_report()` - walks
     `reachable_local_crate_names(&repo_root().join("crates/scheduler/Cargo.toml"),
     &filesystem_manifest_reader)` (helpers already in the file, reuse -
     do not fork them), filters against
     `const SCHEDULER_CONE_FORBIDDEN: [&str; 1] = ["ariadne-report"];`
     (Ruling 1 binding: exactly this one-element set;
     cluster/transform/validate are EXPECTED cone members and must never
     enter this const), and
     on offenders panics with the reachable set plus
     `"scheduler must not transitively compile against {offender:?} - consume
     ariadne-artifact directly (AR-26b)"`.
   - `#[test] fn scheduler_cone_guard_detects_planted_report_edge()` -
     non-vacuity mirror of `transitive_guard_detects_planted_scheduler_edge`:
     in-memory planted reader returns, for a path ending
     `planted-scheduler/Cargo.toml`, the manifest text
     `"[dependencies]\nariadne-report = { path = \"../report\" }\n"`;
     assert the walker reports `ariadne-report` reachable from
     `repo_root().join("crates/planted-scheduler/Cargo.toml")`. Self-test
     is green immediately (mechanism proof); the guard test proper is the
     RED carrier.
   - `GUARD_CORE_CRATES: [&str; 7]` -> `[&str; 8]` + `"crates/artifact"`
     (the R2 neutrality walk must not lose the moved code: these exact
     files walked green inside `crates/report` through M23/D-M23-10, so the
     addition is green on arrival; if any hit fires, STOP-report - it
     would falsify the walk-continuity claim).
   - `TRANSITIVE_FORBIDDEN_DEPS: [&str; 6]` -> `[&str; 7]` +
     `"ariadne-artifact"` (plugin crates must not reach the artifact store;
     green on arrival - artifact is reachable only through
     report/scheduler, both already forbidden).
2. The reparent (only after gate 2's RED is journalled):
   `crates/scheduler/src/incremental.rs:23`
   `let manager = ariadne_report::ArtifactManager::prepare_for_repo(repo)?;`
   -> `let manager = ariadne_artifact::ArtifactManager::prepare_for_repo(repo)?;`
   plus the Cargo.toml swap above. No other scheduler file references
   `ariadne_report` (grep-verified 2026-10-11: the ONE hit is `:23`,
   tests included).

## RGB protocol

1. Red (RED-first IS mandated here - the ONLY hop where it is meaningful):
   add the guard tests + const extensions, run gate 3's m17 command
   BEFORE any scheduler edit; expected failure line:
   `9 passed / 1 failed / 0 ignored / 1 suites`, failing test
   `scheduler_cone_excludes_report`, message listing `ariadne-report` as
   reachable from `crates/scheduler/Cargo.toml:16`. Journal it verbatim.
   (The planted self-test and both const extensions pass immediately; the
   RED must be exactly that one failure.)
2. Green: the swap; re-run m17 -> `10 passed / 0 failed / 0 ignored / 1
   suites`; then all gates. NEVER weaken an assertion to pass.
3. Refactor: none; the guard additions keep the file's existing helper
   vocabulary (`reachable_local_crate_names`, `filesystem_manifest_reader`).

## Hygiene (review-enforced)

Ceiling at risk: `m17_plugin_seam_test.rs` is 663 lines + ~90 new = ~750,
under the 1000 ceiling but the largest file you touch - no test-body
growth beyond the two new tests. `hygiene_check.py` exit 0, ZERO blocking
rows; the walk-scope addition must not itself be counted as a hygiene
change (it is test code; R-walk is unrelated).

## Frozen locks (verify + quote)

- `python tools/distill.py -- git diff <dispatch-HEAD> -- crates/report crates/artifact crates/cli crates/pipeline tests/golden crates/lang-cpp` -> EMPTY (this hop touches only scheduler + m17 + lock).
- Every test in `tests/integration/**` except the m17 file: UNEDITED;
  scheduler/worker-core test files UNEDITED (conservation `26 + 8` is the
  lock; zero test edits).
- m00 golden pair UNEDITED + `git status --porcelain -- tests/golden` empty.
- bincode wire bytes unchanged by construction (no moved code this hop).

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/distill.py -- cargo check --workspace --all-targets`
   -- GREEN after the swap (STRICT COMPILE-GREEN; proves no other crate
   named the removed edge).
2. m17 RED run (before swap):
   `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test`
   -- `9 passed / 1 failed / 0 ignored / 1 suites` (expected-red captured
   verbatim, NEVER committed).
3. Same command after the swap -- `10 passed / 0 failed / 0 ignored / 1 suites`.
4. `python tools/testrun.py --distill -p ariadne-scheduler` -- `26 passed /
   0 failed / 0 ignored / 2 suites` UNEDITED, zero test edits.
5. `python tools/testrun.py --distill -p ariadne-worker-core` -- `8 passed /
   0 failed / 0 ignored / 2 suites` UNEDITED.
6. `python tools/distill.py -- cargo tree -p ariadne-scheduler -e normal`
   -- machine lock (Ruling 1, precision note P1): grep the distilled
   capture for the EXACT token `ariadne-report` at package-name position;
   the pass condition is ZERO matches (token unambiguous - no other
   workspace package name contains it). `ariadne-cluster`,
   `ariadne-transform`, `ariadne_validate` DO appear as expected cone
   members - never grep for their absence (P2). Journal the grep
   invocation, its zero-match result, and the capture path from your own
   run (`target/distill-capture.txt`).
   `-e normal` is REQUIRED (dev/build edges would show test-only plugin
   edges the guard by design does not walk).
7. `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test` -- `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED.
8. `python tools/hygiene_check.py` -- exit 0, zero blocking rows.
9. `python tools/encoding_guard.py --staged` -- exit 0 before committing.
10. `python tools/encoding_guard.py --all` -- exit 0, journal the run.

Commit (single scoped commit; the m17 RED is never committed - the commit
carries guard extension + swap together, AR-20b precedent, with the RED
capture living in the journal):
`python tools/commit_scoped.py --scope "crates/scheduler/**" --scope "tests/integration/tests/m17_plugin_seam_test.rs" --scope "Cargo.lock" --scope "tasks/ar-26b-handoff-journal.md" -m "refactor(scheduler): reparent to ariadne-artifact, drop scheduler->report; m17 scheduler-cone guard (AR-26b)"`
(never `git add .`).

## Rename-completeness proof clause (AR-20a D7 - included for chain
uniformity)

This hop stages no `git mv`, but the clause still applies as written:
after the commit, `git show --stat -M HEAD` must show EXACTLY the
touch-set paths (deletes+adds where any exist; no stale copies, no path
outside the touch-set); `git ls-files` spot-check that no removed manifest
edge or file lingers. Any stray deletion falls out to an immediate second
scoped commit + journal note.

## Discipline clauses

- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools
  only; authored content pure ASCII (the m17 file's existing box-drawing
  section-header comments: keep your additions pure ASCII, do not
  restyle them).
- ZERO test edits except the m17 APPEND (existing test bodies byte-intact;
  the two const-array edits are the only in-place changes).
- Touch only the touch-set; out-of-scope breakage: STOP, report, do not fix.
- No raw high-volume commands; distilling entry points only; quote
  canonical lines from OWN stdout (capture-race ruling).
- After EACH Green gate: 1-3 line checkpoint to
  `tasks/ar-26b-handoff-journal.md`. Context exhaustion: STOP, return
  disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ar-26b-handoff.md
  --scope ... --cmd ...`.
- Serialization: both extraction hops APPROVED + landed before dispatch;
  no other 26-chain hop runs concurrently (none remains after this one);
  AR-31 may still be live in lang-cpp - file-disjoint, rebase +
  regenerate `Cargo.lock` on collision.

## Report back (final message)

1. Commit hash + distilled `git show --stat -M` + D7 clause output.
2. RED evidence: gate 2 verbatim failing line (first, per RGB).
3. Green evidence: gates 1, 3-7 canonical lines verbatim + gate 6 grep
   proof + capture paths. 4. Any deviation + reason. 5. Out-of-scope
   observations (report-only). Never paste raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the list above. Terminal
chain note for the reviewer's acceptance pass: fresh workspace census
expected `1399/0/13/105` (passed total is the authority), and the chain
closes the original AR-26 contract per `tasks/ar-26b-shape-ruling.md`:
audit A3 PARTIALLY satisfied (Ruling 2); the cluster/transform/validate
residual is a DEFERRED LEDGER ROW registered at orchestrator closeout with
the AR-26c promotion charter verbatim (Ruling 3, P3) - not a follow-up AR;
journal the disposition in the verdict.
