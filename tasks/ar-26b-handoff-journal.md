# AR-26b Handoff Journal - Scheduler reparent to ariadne-artifact + m17 cone guard

Implementer lane (sole live lane). Contract: `tasks/ar-26b-handoff.md`
as amended at `69ab3f7`, bound by `tasks/ar-26b-shape-ruling.md`
(Option A, Rulings 1-4, precision notes P1-P3).

## Dispatch HEAD + pre-mutation baselines (re-measured, zero mutation yet)

Dispatch HEAD: `8eec456` (docs addendum of AR-26a2 landing `0d348ee`,
which is the frozen-lock diff base named by the dispatch message).

All lines quoted from own stdout, capture paths printed per run
(`target/testrun-capture.txt` is shared-worktree; per capture-race ruling
the digests below are from my run's stdout).

| suite | command | line (verbatim) |
| --- | --- | --- |
| m17 | `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test` | `8 passed / 0 failed / 0 ignored / 1 suites` |
| scheduler | `python tools/testrun.py --distill -p ariadne-scheduler` | `26 passed / 0 failed / 0 ignored / 2 suites` |
| worker-core | `python tools/testrun.py --distill -p ariadne-worker-core` | `8 passed / 0 failed / 0 ignored / 2 suites` |
| artifact | `python tools/testrun.py --distill -p ariadne-artifact` | `40 passed / 0 failed / 0 ignored / 2 suites` |
| report | `python tools/testrun.py --distill -p ariadne-report` | `7 passed / 0 failed / 0 ignored / 4 suites` |
| m23 | `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test` | `7 passed / 0 failed / 0 ignored / 1 suites` |

Baseline reconciliation vs the two authority pins:

- m17 `8/0/0/1` matches the contract pin; `8 -> 10` reconciliation applies.
- scheduler `26/0/0/2` and worker-core `8/0/0/2` match; conservation closes
  here with ZERO test edits (the chain's conservation equation).
- artifact `40/0/0/2` matches.
- report measured `7/0/0/4`: matches the orchestrator dispatch pin
  (`7/0/0/4`); the handoff's own baseline row still says `7/0/0/3`
  (line 47) - STALE HANDOFF LINE, measurement is the authority
  (report-only observation; the handoff is outside my touch-set, not fixed).
- m23 `7/0/0/1` matches the dispatch pin; file UNEDITED (proven by the
  frozen-lock diff gate below).

Single-consumption-site re-proof at dispatch HEAD (grep tool, not git grep
- git grep is allowlist-denied, logged): searching `crates/scheduler` for
`ariadne[-_]report` returns exactly TWO hits -
`Cargo.toml:16` (the manifest edge) and `src/incremental.rs:23` (the ONE
call site), tests included. Matches journal F5 of `tasks/ar-26-handoff-journal.md`.

## Gate 6 evidence - BEFORE side (pre-swap tree)

Command: `python tools/distill.py -- cargo tree -p ariadne-scheduler -e normal`
Digest: `ok (exit 0, 0 warnings emitted) -> capture: target\distill-capture.txt`

Exact-token grep of the capture (Grep tool, pattern `ariadne-report`):
ONE match, at package-name position:

```
line 71: [box-drawing] ariadne-report v0.1.0 (...crates\report)
```

BEFORE state confirmed: `ariadne-report` IN Tree(scheduler), dragging
`ariadne-artifact` (capture line 73) and the expected cone members
`ariadne-cluster` / `ariadne-transform` / `ariadne-validate`
(capture lines 76/96/113 and the `(*)` repeats). Per P2 these three are
EXPECTED members and are never asserted absent.

Checkpoint: baselines clean, no mutation yet. Next: m17 guard extension
(RED carrier) BEFORE any scheduler edit.

## Gate 2 - RED capture (guard extension landed, scheduler UNTOUCHED)

m17 extension applied exactly per contract: guard-proper + planted
self-test appended as section 5; the two in-place const edits
(`GUARD_CORE_CRATES` 7->8 + `"crates/artifact"`,
`TRANSITIVE_FORBIDDEN_DEPS` 6->7 + `"ariadne-artifact"`); all existing
test bodies byte-untouched (non-ASCII glyph census unchanged: 13 hits,
all pre-existing lines 1-440; section 5 pure ASCII).

Command: `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test`
Capture: `target/testrun-capture.txt` (40 lines), quoted from own stdout.

Verbatim RED (never committed):

```
9 passed / 1 failed / 0 ignored / 1 suites
---- scheduler_cone_excludes_report ----
thread 'scheduler_cone_excludes_report' (5128) panicked at tests\integration\tests\m17_plugin_seam_test.rs:708:5:
scheduler must not transitively compile against "ariadne-report" - consume ariadne-artifact directly (AR-26b); reachable from "C:\\Users\\Kurt\\Code\\AriadnesThread\\.kilo\\worktrees\\shining-enthusiasm\\crates/scheduler/Cargo.toml": ["ariadne-repo", "ariadne-build", "ariadne-repo", "ariadne-lang-api", "ariadne-repo", "ariadne-lang", "ariadne-repo", "ariadne-lang-api", "ariadne-index", "ariadne-repo", "ariadne-build", "ariadne-lang-api", "ariadne-graph", "ariadne-repo", "ariadne-build", "ariadne-index", "ariadne-lang-api", "ariadne-lang", "ariadne-report", "ariadne-repo", "ariadne-build", "ariadne-graph", "ariadne-index", "ariadne-cluster", "ariadne-repo", "ariadne-index", "ariadne-graph", "ariadne-lang", "ariadne-transform", "ariadne-index", "ariadne-graph", "ariadne-cluster", "ariadne-repo", "ariadne-lang", "ariadne-validate", "ariadne-transform", "ariadne-cluster", "ariadne-lang", "ariadne-lang", "ariadne-artifact", "ariadne-build", "ariadne-graph", "ariadne-index", "ariadne-lang", "ariadne-repo", "ariadne-cluster", "ariadne-transform", "ariadne-validate", "ariadne-lang", "ariadne-worker-core", "ariadne-repo", "ariadne-build", "ariadne-index", "ariadne-lang"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
9 passed / 1 failed / 0 ignored / 1 suites
```

Failure attribution (exactly ONE failing test, as mandated):
offender edge = `crates/scheduler/Cargo.toml:16`
(`ariadne-report = { path = "../report" }`) - the reachable set lists
`ariadne-report` from the scheduler root manifest.
The other 9 tests pass immediately: the planted self-test
(`scheduler_cone_guard_detects_planted_report_edge`) proves walker
non-vacuity; `core_crates_do_not_name_languages` green with
`crates/artifact` in the R2 walk (walk-continuity claim holds - the moved
files were walked inside `crates/report` through M23/D-M23-10);
`plugin_crates_transitive_tree_excludes_core_pipeline` green with
`ariadne-artifact` forbidden (artifact unreachable from plugin cones).
RED matches the contract's predicted line verbatim.

Next: the reparent (gate 1 check, gate 3 re-green, gates 4-7 conservation).

## GREEN - the reparent + gate record (all lines own stdout)

Reparent edits (exactly the contract literals):

- `crates/scheduler/src/incremental.rs:23`:
  `ariadne_report::ArtifactManager::prepare_for_repo(repo)?` ->
  `ariadne_artifact::ArtifactManager::prepare_for_repo(repo)?` (ONE line).
- `crates/scheduler/Cargo.toml`: removed
  `ariadne-report = { path = "../report" }` (was line 16); added at the
  swap's place:
  `# AR-26b: artifact-store IO consumed directly (was via report).` +
  `ariadne-artifact = { path = "../artifact" }`.
- `Cargo.lock`: regenerated by cargo (1 insertion / 1 deletion - the
  scheduler package edge swap).

Gates (verbatim, capture paths printed per run; testrun capture =
`target/testrun-capture.txt`, distill capture = `target/distill-capture.txt`):

1. `python tools/distill.py -- cargo check --workspace --all-targets` ->
   `ok (exit 0, 11 warnings emitted)`; capture shows ZERO warnings from
   scheduler or artifact (the 11 are pre-existing families: AR-21
   `IncludeResolver` deprecations, golden-helper dead code, unused test
   locals).
3. m17: `10 passed / 0 failed / 0 ignored / 1 suites` (8 -> 10 per
   reconciliation; same binary, no suite relocation).
4. scheduler: `26 passed / 0 failed / 0 ignored / 2 suites` UNEDITED
   (zero test edits).
5. worker-core: `8 passed / 0 failed / 0 ignored / 2 suites` UNEDITED.
6. `python tools/distill.py -- cargo tree -p ariadne-scheduler -e normal`
   -> exit 0. Exact-token grep (Grep tool, pattern `ariadne-report`) of
   the capture: ZERO matches (BEFORE: ONE match at capture line 71).
   `ariadne-artifact` now sits at a direct normal edge (capture line 3);
   `ariadne-cluster`, `ariadne-transform`, `ariadne-validate` remain
   present as EXPECTED cone members through `scheduler -> artifact`
   (P2: described, never asserted absent - the planted self-test covers
   non-vacuity of the one locked predicate).
   Ruling 1 fully machine-locked: NOT-IN Cone (mechanism a, test) AND
   NOT-IN Tree (mechanism b, this grep), both green at the same commit.
7. m00 golden pair + facts-path parity:
   `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED.
   m23 re-checked unedited via the frozen-lock diff (measured `7/0/0/1`
   pre-mutation).
8. `python tools/hygiene_check.py`: R3 warn rows only (all pre-existing,
   none in touched files); ZERO blocking R1/R2 rows. m17 file at 740
   lines - under the 1000 ceiling, no test-body growth beyond the two
   new tests. (Superseded mid-write figure 744: actual final file length
   is 740, re-measured post-commit.)

Frozen-lock evidence (vs base `0d348ee`):

- `python tools/distill.py -- git diff 0d348ee -- crates/report
  crates/artifact crates/cli crates/pipeline tests/golden
  crates/lang-cpp` -> capture 0 lines = EMPTY diff.
- `git diff 0d348ee --stat` (working scope): Cargo.lock 2, scheduler
  Cargo.toml 3, incremental.rs 2, m17 +81, this journal - plus
  `tasks/ar-26a2-handoff-journal.md +56` which is the HEAD addendum
  commit `8eec456` content, not this lane. Foreign
  `tests/fixtures/basic/artifacts/includes/report.json` LF/CRLF churn
  left unstaged and uncommitted (not golden; golden dir clean).
- bincode wire bytes unchanged by construction: no moved code this hop;
  m00/m23 suites and `tests/golden/**` UNEDITED.

Disposition per Ruling 2/3 (journal of record): audit A3 PARTIALLY
satisfied; the cluster/transform/validate residual is a DEFERRED LEDGER
ROW at orchestrator closeout carrying the AR-26c promotion charter
verbatim - not a follow-up AR.

## Commit record + D7 clause + deviations (post-commit addendum)

Commits (this lane):

- `3064b77` docs(ar-26b): pre-mutation checkpoint (journal only).
- `8e08600` refactor(scheduler): reparent to ariadne-artifact, drop
  scheduler->report; m17 scheduler-cone guard (AR-26b) - the contract's
  single scoped commit verbatim; numstat: Cargo.lock 1/1,
  scheduler/Cargo.toml 2/1, incremental.rs 1/1, journal 104/0,
  m17 79/2 (the 2 deletions are the two const size lines themselves;
  every existing test body byte-intact).
- (this addendum commit carries the 740-line correction above.)

D7 clause (`python tools/distill.py -- git show --stat -M HEAD`): EXACTLY
the 5 touch-set paths; no deletes/adds (no `git mv` this hop - the D7
rename logic is inert by design); no stale copies; no path outside the
touch-set. Lingering-edge spot-check (`git ls-files` is allowlist-denied;
substituted per capture-race-adjacent discipline): Grep over
`crates/scheduler` for `ariadne[-_]report` -> ZERO matches (post-swap;
was 2 at dispatch HEAD); Cargo.lock scheduler package deps:
`ariadne-artifact` present, `ariadne-report` absent.

Encoding gates: `--staged` and `--all` both silent (clean). Raw exit
codes are not echoable in this shell (chained echo / `python -c` are
allowlist-denied) - adjudicated on the printed contract, same precedent
as AR-02/AR-08/AR-18 verdicts.

Refused-command log (verbatim shape, substituted, gates never skipped):

- raw `git log --oneline -5; git status --porcelain` -> refused
  (catch-all deny); rerun as two `python tools/distill.py -- git ...`
  invocations.
- `git grep`, `git ls-files`, `python -c`, chained `echo` not attempted
  as gates; substitutes used (Grep tool, Read tool) and named above.

Deviations:

- D1 NONE against the contract's code shape: guard wording, panic
  message, const arrays, planted self-test, `-e normal`, RED-first
  protocol, and single-commit bundling (RED captured in this journal,
  never committed) all per Ruling 4 verbatim.
- D2 report suite measured `7/0/0/4` vs the handoff baseline row's
  `7/0/0/3` (line 47): STALE HANDOFF LINE, the orchestrator dispatch pin
  and my measurement agree; report-only (handoff is outside touch-set).
- D3 the module doc still reads "the seven core pipeline crates" while
  `GUARD_CORE_CRATES` is now 8: deliberately NOT restyled - the contract
  limits in-place edits to the two const arrays; section 5's doc
  comment records the walk-scope extension. Report-only observation.
- D4 two mis-targeted Edit calls aimed at a nonexistent
  `tasks/placeholder-noop.md` during the m17 append; the tool refused
  them ("File ... not found"), so nothing was created or modified; the
  real m17 edits landed correctly and are in `8e08600`. Disclosed for
  trail completeness.
- D5 gate 6 BEFORE-side tree capture run in addition to the mandated
  AFTER-side (dispatch final report requires before/after evidence);
  extra distill invocation, same mandated command, journal-only.

Out-of-scope observations (report-only, nothing fixed):

- The stale handoff baseline row (D2) and the stale "seven core crates"
  doc phrase (D3) are one-line corrections for whoever next owns those
  files (planner/ledger lane).
- Foreign `tests/fixtures/basic/artifacts/includes/report.json`
  LF/CRLF churn persists unstaged (pre-existing across the whole AR-26
  chain per 26a/26a2 journals); left untouched.

Terminal-hold note for the reviewer's acceptance pass (contract's
Reviewer-gate reference): fresh workspace census expected
`1400 / 0 / 13 / 105` (m17 8->10; passed total authority); census/gate
ownership per Planning Law keeps `--baseline` with the orchestrator's
own bounded run - not run in this lane per dispatch restriction.
