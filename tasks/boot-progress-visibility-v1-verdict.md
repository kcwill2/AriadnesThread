# Verdict - M26 boot-progress-visibility V1 (APPROVED)

Reviewer hop: `review-gate` skill. Handoff
`tasks/boot-progress-visibility-v1-handoff.md`; journal
`tasks/boot-progress-visibility-v1-journal.md`. Range `7aeee44..f325bee`
(start rev matches the journal's recorded bootstrap HEAD). Commits
adjudicated, oldest first: `79f3403` (extraction), `f325bee` (frame-1-first
+ boot panic bridge + tests). Ground truth: `checkpoint_guard.py status` ->
HEAD `f325bee` on main, no dirty or untracked files inside the touch-set
(the one dirty file and the four untracked files are unrelated
`.kilo/plans`/prior-milestone docs).

VERDICT: APPROVED. No blockers. Minor findings below; follow-ups to the
ledger. V2 dispatch cleared; V3 dispatch gate open (see Sequencing).

## 1. Gate evidence (canonical lines verbatim)

- Scoped re-run, committed tree:
  `python tools/testrun.py --distill -p ariadne-cli` ->
  `108 passed / 0 failed / 1 ignored / 5 suites`, exit 0. Exact match to
  the MUST line. (Plain non-distill form crashed on the known cp1252 echo
  defect; see finding 2.)
- Workspace census, distill form `python tools/testrun.py --distill` ->
  `1226 passed / 0 failed / 4 ignored / 86 suites`, exit 0 = ledger anchor
  `1219 passed / 0 failed / 4 ignored / 86 suites` (census @ 9ceb8e3,
  carried by doc-only 7aeee44) plus exactly the 7 tests this hop added;
  suite count unchanged. No-fail-fast form (full workspace, 2105-line
  capture).
- Tool unit suites: `Ran 188 tests in 20.655s / OK`, exit 0 (via the
  distill wrapper).
- `python tools/hygiene_check.py` -> exit 0 (distill verdict PASS). Output
  is R3 warn-only; zero R3 lines from `stage_spans.rs`,
  `first_frame_tests.rs`, `boot_panic_tests.rs`; the `bar`/`p` warns in
  `ui.rs:636/644/652` sit on pre-existing verbatim lines.
- `python tools/encoding_guard.py --all` -> exit 0, zero output.
- `python tools/distill.py ... -- python tools/ui_smoke.py` ->
  `ui_smoke: PASS`, exit 0 (`boot+route checks clean (cold fixture,
  attempt 1, 0.9s)`).
- `bench_compare`: NOT RUN under the handoff's written exemption; the diff
  was audited for hot-path changes and has none (sec. 5).

## 2. Commit-vs-touch-set

`git diff --name-only 7aeee44..f325bee` -> exactly
`crates/cli/src/commands/ui.rs`, `ui/boot_panic_tests.rs`,
`ui/first_frame_tests.rs`, `ui/stage_spans.rs`,
`tasks/boot-progress-visibility-v1-journal.md`. All five inside the hard
touch-set; every per-commit `git show --stat` lands inside it as well.
PASS.

## 3. Frozen locks (handoff table, each proven)

- `progress.rs` byte-identical (I3/DP-1):
  `git diff --stat 7aeee44..f325bee -- crates/cli/src/commands/progress.rs`
  -> empty. The `*_with_sink` builders delegate onto the FROZEN
  `TerminalStepBars::with_sink` (`progress.rs:168` - verified in the
  frozen file), so no diff was needed. PASS.
- Stage-5 13-frame oracle + planning needle UNEDITED and green:
  `progress_contract_tests.rs` zero-diff across the range;
  `test_stage_five_milestone_sequence_is_monotonic_and_contained` and
  `test_planning_milestone_precedes_justifying_in_source` `... ok` in the
  workspace capture (lines 93-94).
- Boot parity oracles (I2) UNEDITED and green: `boot_progress_tests.rs`
  zero-diff; all 10 `boot_progress_tests` incl.
  `test_i2_terminal_parity_mid_boot_prefix` / `test_i2_terminal_parity_full_boot_run`
  `... ok`.
- Append-only floor: `builder_tests.rs`, `target_files_tests.rs` zero-diff.
- Goldens/fixtures, `crates/pipeline/**`, `crates/repo/**`, `crates/ui/**`,
  `tools/**` untouched (sec. 2 name list).

## 4. Extraction commit 79f3403 - VERBATIM move, independently proven

Byte-safe route only (no PowerShell redirection): blobs captured via
distill-captured `git show` and a mechanical git blob-vs-blob diff
(`git diff "7aeee44:...ui.rs" "79f3403:...ui/stage_spans.rs"`, capture
`target/blobdiff-verbatim-proof.txt`). Across old lines 90-189 the diff
shows ONLY: 21 `-const`/`-fn` vs `+pub const`/`+pub fn` pairs differing in
the `pub ` prefix alone; the new module doc header; and the authorized
`DISCOVERY_HASHED_SPAN_END` block. Everything else - the A4 calibration
comment (with its `·`/`—` characters), every doc comment, every function
body - is context, i.e. byte-identical per git. The journal's
"non-blank compare, 92 lines" claim is consistent. Values: no percent
changed (I2/I9 pure move). D-const comment carries the required
`// TODO(calibrate M27 T1): replace 10 with the measured stage-1 split
point from the boot_stages release table (A4 precedent).` verbatim
(info: the handoff literal omits the closing paren; the implementation's
correction is accepted). `#[allow(dead_code)]` (f325bee, +1 line) is
attribute-only; the const line stays byte-identical, 117 lines <= ~140.
Re-export coverage: every frozen `use super::{...}` name resolves
(compile-proven by the green suites with the frozen import lines zero-diff).
PASS.

## 5. Behavior commit f325bee - plan-literal compliance

- Frame-1-first (I5): first statement of `build_ui_payload` is
  `progress(stage_report(1, stage_start_percent(1), ARTIFACTS_PHASE,
  "Hashing sources"));` (ui.rs:223) before `Pipeline::new` (:225); the
  duplicate at the head of `run_pipeline_stages` deleted; every later
  frame byte-identical in order; I9 end frame `(1, 20, "Include graph
  ready")` retained (ui.rs:116). Production occurrence of
  `"Hashing sources"`: exactly one. PASS.
- Panic bridge (I11): `drive_boot_build(state: &Arc<UiState>, fps_tx,
  build_fn)` exactly as specified; `catch_unwind(AssertUnwindSafe(...))`;
  Ok arm unchanged in body; Err arm writes `state = "error"`,
  `message = e.to_string()`, `fps_tx.send(None)`; panic arm downcasts
  `&str`/`String`, writes the SAME error state with the panic text, then
  `tracing::error!("UI boot task panicked: {text}")` (stricter than the
  watcher's `warn!`, per R8), then `send(None)`. The `&*payload` unbox fix
  is documented with probes and was caught by test 5 during Green - not an
  assertion weakening. Verified semantically live: tests 5-7 green.
- JoinHandle kept: `let boot_handle = tokio::task::spawn_blocking(...)`;
  awaited in a `tokio::spawn`; JoinError routes to the same
  `surface_boot_failure` on the same shared `Arc<UiState>` (`join_state`,
  `boot_state`, `state` verified clones of one Arc) plus an `error!`
  mirror; watcher block (fps_rx `if let Ok(Some(..))`) untouched. PASS.
- Sink builders: `make_bar_progress_with_sink` /
  `make_boot_progress_with_sink` are `pub(crate)`, delegate over the frozen
  `TerminalStepBars::with_sink`; the original `make_bar_progress` /
  `make_boot_progress` delegate with stdout. No `crates/ui/**`, asset, or
  SSE change. PASS.
- Hot-path audit for the bench exemption: range touches only cli `ui.rs`,
  the new cli files, and the journal. No stage-5 producer path, no
  pipeline/repo/graph change; `run_pipeline_stages` frames byte-identical
  minus the relocated start emission; `catch_unwind` wraps the one-shot
  boot call, not throughput paths. The written no-op exemption stands.
- `ui.rs` 839 -> 751 -> 832: smaller (hard gate met); all functions < 100
  (R1-enforcing hygiene exit 0); `drive_boot_build` ~25 lines <= ~50.

## 6. Deviation adjudications (independent, not rubber-stamped)

1. Red-note misprediction (tests 1-4 green at Red): ACCEPTED. Adjudicated
   on the merits: at the start rev the sole legacy emission sat at
   ui.rs ~:197, textually before `Pipeline::new` (~:314) and unique, so the
   two source-text locks + tests 3-4 cannot be Red pre-change - the
   handoff's note was wrong about the pre-change file layout. Tests 1-2 are
   transition locks (they guard plausible wrong Green states: post-construction
   emission, or duplication), which is a legitimate test class. The real
   semantic Red is tests 5-7 (panic escapes stub, no error state), captured
   verbatim; Red counts are arithmetically consistent (105+3=108); the
   journal also records the demonstration run where tests 1-2 did fail
   (early `#[cfg(test)]` token). Handoff errata -> ledger (finding 5).
2. `*_with_sink` builders staged at Red: ACCEPTED. The handoff's Red recipe
   ("NO sink builders yet") contradicts its own required test 7, which
   calls `make_boot_progress_with_sink` - the tests could not compile at
   Red without it. The builders landed as pure delegation (no hidden
   wrongness); this is the only self-consistent reading. Handoff errata ->
   ledger (finding 5).
3. `#[allow(dead_code)]` on the D const: ACCEPTED - attribute-only; value
   and comment byte-identical; "declared, not used yet" is the handoff's
   own intent and the attribute documents it.
4. `#[cfg(test)]`-gated `COLLATERAL_FILE_SPAN_END` re-export placed below
   all production code: IN-SPEC, no rework. The handoff pins namespace
   coverage ("EXACTLY the names the frozen modules import + every name
   production calls") and unedited frozen imports, not statement position.
   Placement was forced: the frozen (un-editable)
   `test_planning_milestone_precedes_justifying_in_source` cuts the
   production section at the FIRST `#[cfg(test)]` token - an early gated
   use breaks it (journal: first placement did), and an ungated top-level
   use of a test-only name is an unused-import warning. The chosen form
   keeps every frozen import resolving unedited, zero warnings, all locks
   green. Note for V3: the explanatory comment at ui.rs:809 itself contains
   a backticked `#[cfg(test)]` token, so the production-section cut lands
   inside that comment - still strictly after all production code, hence
   harmless (finding 6).
5. `ui.rs` "land <= ~700" target missed at 832: ACCEPTED. Independent
   arithmetic confirms the target is unreachable inside the handoff's own
   constraints: the handoff mandates ~+100 lines of additions to `ui.rs`
   (2 sink builders, bridge, panic helper, JoinError task, 2 mod decls)
   while the extraction yields -88, and the only further reductions (a new
   module file, or moving boot machinery into `stage_spans.rs`) are
   forbidden by the same handoff's touch-set and the 140-line module cap.
   The hard requirement - land smaller than 839 - is met (832), and the
   ceiling's stated purpose (V3 grows `stage_spans.rs`, not `ui.rs`, via
   the declared D const) is preserved. Deviation was declared, not silent.
   Ledger follow-up in finding 7.
6. Gate-invocation volume ~24 vs planned 6 (ceiling ~10): ACCEPTED WITH
   WARNING (finding 3). Sizing-law spirit (bounded context) held - every
   named gate ran in distilling form, report is digest-shaped with
   canonical lines + exit codes, zero raw dumps; the overflow came from
   legitimate iterations (Red capture + demonstration run, R1 fix re-run,
   downcast probes) rather than sloppy loops. The numeric contract was
   still broken; recorded so planners budget iterations.

## 7. Findings

All non-blocking.

1. [minor, environment - not implementer fault] Reviewer-session deny
   surface blocked scripted byte-proof conveniences; each refusal was
   boilerplate `{"permission":"bash","pattern":"*","action":"deny",
   "source":"global"}`: (a) multi-line `python -c` (direct and distill-
   wrapped), (b) inline python spawning `git` via subprocess (anti-bypass),
   (c) inline python file I/O (`open(...)`), (d) PowerShell compound forms
   (`Measure-Object`, `if ($?)`). No required gate was lost - the
   verbatim proof ran via distill-captured git blob-diff and exit codes via
   distill verdicts. Recorded per reviewer duty.
2. [minor, known family - deferred row] Plain `python tools/testrun.py
   -p ariadne-cli` crashed: `UnicodeEncodeError: 'charmap' codec can't
   encode characters in position 2017-2036` in `_echo` (cp1252 console).
   Confirms the implementer's report as the testrun echo defect family,
   not implementer fault. `--distill` form works; reviewer quote above is
   from it.
3. [minor, process] Implementer gate invocations ~24 > planned 6 and > the
   sizing-law's ~10 ceiling; digest discipline verified intact. Disposition
   6; ledger follow-up: budget Red-capture + hygiene-fix iterations in
   future handoff sizing.
4. [minor, process - accepted] Landing commits used an explicit
   `git add -- <touch-set paths>` route (distill-wrapped per journal), not
   the named `commit_scoped.py --scope` gate command; the board shows an
   established orchestrator ruling for the direct commit route under this
   allowlist. Effect verified: both commits' `git show --stat` hit exactly
   touch-set files. Accepted.
5. [info, handoff errata -> ledger deferred rows] Two handoff-internal
   contradictions surfaced by this hop: Red-note misprediction on tests
   1-2, and "no sink builders at Red" vs test 7 requiring the sink seam.
6. [info] ui.rs:809 comment embeds a literal `#[cfg(test)]`; production-
   section cuts in source-text locks land inside that comment (still after
   all production code). Harmless today; V3 must keep any `#[cfg(test)]`
   occurrences below ALL production code.
7. [info, V3 heads-up - accepted from the implementer] `ui.rs` at 832 and
   `run_ui` ~89 lines: V3 must land additions in `stage_spans.rs` per plan;
   if V3 growth pushes `ui.rs` back toward the 1000 cap, the V3 handoff
   should own a boot-machinery extraction (out of V1's touch-set).

## 8. Sequencing disposition

V1 landed on main at `f325bee`, APPROVED. V2 dispatch (repo crate,
disjoint files) CLEARED - parallel-safety was never gated on V1 and the
touch-set audit confirms disjointness. V3 dispatch gate ("only after this
lands") is now open; V1's out-of-scope observations are fully discharged
above, none outstanding.
