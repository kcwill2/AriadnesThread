# AR-23 Verdict - analyze incremental path consumes `Pipeline::for_incremental` (A6 part 2)

**APPROVED**

Commit: `d8559ae` (single commit; parent `d63c0d0`).
Reviewed at HEAD `7adbeca` (the child `7adbeca` is AR-20 journal docs only, out of this lane).
Artifacts: `tasks/ar-23-handoff.md`, `tasks/ar-23-handoff-journal.md`, commit
`d8559ae` (analyze.rs 20+/94-, new lock `tests/integration/tests/ar23_analyze_incremental_parity_test.rs`
+236, journal +284). Fresh gates in `tasks/ar-23-execution-results.md`.

## 1. Commit-vs-touch-set: EXACT

`git diff --name-only d8559ae^ d8559ae` = exactly the 3 handoff-declared paths
(`crates/cli/src/commands/analyze.rs`, the new integration lock, the journal).
`crates/pipeline/**` is absent from the file list - consumption-only honored.
`tests/golden/**` and the m00 lock files absent - frozen locks byte-untouched
(both m00 suites re-ran green UNEDITED).

## 2. Mirror TRULY DEAD

Base blob `d8559ae^:crates/cli/src/commands/analyze.rs` (read whole, quoted
whole-rev) verified: the locate/discover/shaper/restrict sequence sits at
:32-46 (`Repository::locate` at :32, `artifacts_are_authoritative` read at
:42-46), the guard at :27-30, the helper pair + the self-admitted mirror
comment ("mirrors `Pipeline`'s `language_filter.is_none()` index guard") at
:105-121. The handoff's `~:32-56` / `~:110-114` anchors are honest approxima-
tions and the journal's "as-is" claim holds.

Committed code: the sequence, BOTH helpers, the mirror comment, and the now-
unused imports (`BuildDiscoverer`, `Repository`) are all deleted; the cli-side
`ensure_language_registered` call is removed with the error text preserved -
the ctor's guard message (`for_incremental.rs:67`) is verbatim-equal to
`crates/cli/src/registry.rs:29` ("language plugin not registered: {language:?}").
The remaining path is `Pipeline::for_incremental(repo_path, language)?` ->
authority from `pipeline.incremental_view` -> `registry::incremental_backend`
over `&pipeline.build` (analyze.rs:30-39) - the AR-22-journaled cli-side
pairing surface, exactly as AR-22's D5 disposition prescribes. Grep confirms
zero residual `restrict_and_validate_units` / `artifacts_are_authoritative` /
`BuildDiscoverer` / `Repository::locate` in analyze.rs (remaining hits are
other command modules, other hands). The persist/print blocks are byte-
identical (only the `save_artifacts` argument changed to `&pipeline.repo`).

## 3. PARITY AUTHENTICITY (central) - ACCEPTED

- **Literals, not tautology.** The lock asserts hardcoded captured values:
  the 15-file authority file set incl. both shard ids, the shard-only 2-file
  set, stats data `0/2/0.0`, `2/0/100.0`, `0/2/0.0`, `2/0/100.0` - no expected
  value is re-derived from the same run. The C3 persist-nothing check is a
  genuine cross-run size snapshot (file set + sizes before vs after the
  filtered run), not a self-comparison.
- **Capture authenticity.** The pre-change-green claim cannot be re-observed
  post-commit, but it is corroborated on three axes: (a) the base blob is the
  untouched mirror at the exact anchors the journal cites (HEAD `0d7ecd3` =
  AR-19+AR-22, blob `19818fd` = the one AR-22 diffed); (b) AR-22's verdict
  independently established the ctor's verbatim equivalence to that mirror
  (same sequence, same bail texts, same authority rule), so mirror-side
  behavior on the locked observables is structurally the same code; (c) the
  four expectations are exactly the C1-C4 captures and all four are green
  post-switch, expectations UNCHANGED.
- **Deviation "parity-first replaces red-stub" - ACCEPTED on merits.** For a
  byte-identity refactor, capture -> lock-green-pre -> switch -> same-
  expectations-post is the stronger discipline; a red stub would only re-
  encode the same captured observables. Non-vacuity is proven by mutation and
  I verified the mutation argument structurally: hardcoding `authoritative`
  to `true` leaves the unfiltered C1/C2 runs untouched (already authoritative)
  and makes both filtered runs call `save_artifacts` - red on exactly C4
  (shard-only vs 15-file set) and C3 (size snapshot grows). The journal's
  `2 passed / 2 failed` on exactly the persist-skip locks is the forced
  outcome, not a curated subset. Reverted state matches the committed green
  (my fresh gate: `4 passed / 0 failed / 0 ignored / 1 suites`).
- **Byte-identity of stdout.** The println text is provably untouched (diff
  shows the print/persist blocks unchanged); the data the lines print is
  locked by the green suite. The journal's post-change binary double-run
  reproducing C1/C2 (reuse 2/0/100.0%), C3, C4 verbatim is journal evidence
  for the stdout half; the data half I re-verified myself.

## 4. Carried dispositions from the AR-22 verdict - both HONORED

- **Finding 2 (eager `create_dir_all`):** the journal distinguishes it by
  making the observable the artifact FILE SET, never directory existence, and
  its soundness argument holds on this path: C4's PRE-change capture (old
  binary, filtered run, zero authority persistence) still ends with
  `artifacts/shards/*.json` - the mirror's own run always materializes
  `artifacts/` via the shard store on every successful run, so no successful
  scenario ends without the directory on either side. The ctor additionally
  runs `prepare_for_repo` AFTER locate/restrict (`for_incremental.rs:123-127`),
  so the residual empty-dir delta narrows further than the journal claims -
  to ctor-success-then-error paths, invisible to every file-set and stdout
  observation, already adjudicated harmless in AR-22. ACCEPTED.
- **Finding 3 (no fps-value parity):** grep of journal + lock: every `fps`
  mention is an explicit disclaimer ("I claim FILE-SET and stdout parity only
  - not fps-value parity"; lock header line 21). No fps-VALUE parity claimed
  anywhere. ACCEPTED.

## 5. Baseline reconciliation - INDEPENDENTLY VERIFIED

`git show d8559ae` deletion hunk: exactly 3 `#[test]` fns deleted -
`unrestricted_full_unit_set_is_authoritative` (`artifacts_are_authoritative(None, 8, 8)`),
`active_language_filter_is_never_authoritative` (`... (Some(Cpp), 8, 8)`),
`unit_set_narrowed_below_discovery_is_never_authoritative` (`... (None, 5, 8)`).
All three call ONLY the deleted helper - helper-unit-tests whose subject no
longer exists; the rule they pinned is now the ctor's, locked by the 5-test
pipeline parity suite (green inside 40/0/4/9) and its analyze-path consequence
by the new integration suite. Arithmetic: measured baseline 117 (the journal
honestly corrects the handoff's stale 118; AR-19's own landing line was 117),
`117 - 3 = 114`, confirmed by my fresh gate. Honest reconciliation, accepted.

## 6. Declared deviations - individual dispositions

- D1 parity-first ordering (no red stub): **accepted**, adjudicated on merits
  in section 3 (mutation proof is the non-vacuity instrument).
- D2 lock location (`tests/integration/tests/` rather than cli tests):
  **accepted** - the handoff touch-set itself offers "NEW parity integration
  lock"; a fixture-run observable belongs in the m-series tradition; the
  choice is journaled.
- D3 deletion of the 3 helper-named unit tests: **accepted** - forced by the
  helper deletion; rule survives on the new owner + behavioral integration
  lock (section 5).
- D4 removal of the incremental path's duplicate `ensure_language_registered`
  call: **accepted** - ctor guard first with verbatim message; the cli helper
  stays live for production paths (its registry test green in gate 1).
- D5 capture tooling detours (robocopy via distiller; allowlist refusals):
  **accepted** - no semantic impact; captures quoted from own stdout per the
  capture-race ruling.

## Findings (numbered; none blocking)

1. **Minor (foreign workspace flake, ledger deferred-findings candidate).**
   `cargo test --workspace` is order-flaky independent of AR-23: pipeline
   `internal_tests::test_load_report_matches_sequential_reference` panicked
   once with `ARIADNE_LOAD_MEM_CAPS ... got "3,x"` (`stage_load.rs:223`).
   Root cause: the `caps_env_ablation` mod (landed `f312dba`, pipeline UNEDITED
   since) mutates the process-global env var under a mutex shared only by its
   own 3 tests; parallel siblings in the same lib-test binary read it when
   workspace feature unification compiles the binary with `test-instrument`.
   My re-run (retry-once) was clean: `1372 passed / 0 failed / 13 ignored / 99
   suites`. Also note cargo fail-fast truncates a census at the first failing
   binary (60/94 suites observed) - use `--no-fail-fast` for census lines.
   Not attributable to `d8559ae` (the scoped pipeline binary does not compile
   the caps mod; gate 2 was green twice).
2. **Minor (report-evidence precision).** Journal: `cargo check -p
   ariadne-cli --all-targets` -> `ok (exit 0, 0 warnings emitted)`. My gate
   compilations show `ariadne-cli (lib test) generated 1 warning` (unused_mut,
   `crates/cli/src/lib.rs:804`, introduced by `b49ed06`, lib.rs untouched
   since - foreign to this commit but visible to `--all-targets`). The quoted
   digest line was imprecise; next handoff quote warning counts exactly.
3. **Minor (tooling environment).** `tools/testrun.py` raw-echo mode crashes
   on a cp1252 console (`UnicodeEncodeError` at testrun.py:262); gates were
   run with `PYTHONIOENCODING=utf-8`. Also `--baseline` is unusable as
   dispatched (it requires a baseline FILE; none exists in the repo), so the
   workspace acceptance line is the default `testrun.py` (`cargo test
   --workspace --no-fail-fast`) census. Both are foreign tooling notes, not
   AR-23 defects.
4. **Informational.** The switch constructs a second `default_registry()` at
   analyze.rs:37 although `pipeline.registry` is public; D9-locked identical
   enumeration means zero behavior impact (cosmetic only, no action owed).

## Gate commands (fresh, own stdout; canonical lines verbatim)

- `python tools/testrun.py -- -p ariadne-cli` ->
  `114 passed / 0 failed / 8 ignored / 6 suites`
- `python tools/testrun.py -- -p ariadne-pipeline` ->
  `40 passed / 0 failed / 4 ignored / 9 suites` (UNEDITED)
- `python tools/testrun.py -- --test m00_golden_baseline_test` ->
  `2 passed / 0 failed / 0 ignored / 1 suites`;
  `python tools/testrun.py -- --test m00_facts_path_parity_test` ->
  `2 passed / 0 failed / 0 ignored / 1 suites` (both UNEDITED)
- `python tools/testrun.py -- --test ar23_analyze_incremental_parity_test` ->
  `4 passed / 0 failed / 0 ignored / 1 suites`
- `python tools/encoding_guard.py --all` -> silent (exit 0).
  `python tools/hygiene_check.py` -> exit 1 solely from the two known
  `crates/ui/src/server.rs` rows (`R2 1172 lines > 1000`; `R1 fn
  api_graph_traversal is 161 lines`); zero rows from analyze.rs, the lock, or
  the journal; all other rows warn-only R3 foreign.
- Acceptance pass: `python tools/testrun.py -- --workspace --no-fail-fast` ->
  `1372 passed / 0 failed / 13 ignored / 99 suites`;
  `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 219 tests in
  24.247s` / `OK`. Details, refusal record and not-run statements:
  `tasks/ar-23-execution-results.md`.

## Follow-up dispositions

- Finding 1 (caps-env test isolation flake + census truncation note) and
  Finding 3 (testrun cp1252 echo; no baseline FILE) go to
  `tasks/milestone_ledger.md` deferred-findings at orchestrator closeout.
  Recorded here, not dropped.

## Git constraints honored

Read-only git only (`diff --name-only`, `show` with whole-quoted revs, `log
-S`/`-- <path>`, `rev-parse` via checkpoint_guard). No source, test, or journal
file was modified by this review; only `tasks/ar-23-verdict.md` and
`tasks/ar-23-execution-results.md` were written.
