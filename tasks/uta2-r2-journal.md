# UTA2-R2 Journal - tools: crash-class hardening + package_offline coverage

Handoff: tasks/uta2-r2-handoff.md. Spec: docs/specs/unit-test-audit.md "Round 2"
(UTB-5/6/7). Pure-Python hand: no cargo, gates are the unittest discover suite
plus the two hygiene gates.

## Checkpoint 0 - baseline (session start)

- HEAD: 19349b3 (main). Tree carries foreign-session files in crates/ui (R1
  hand running concurrently); touch-set is tools/*.py + tools/*_tests.py plus
  this journal, so scoped commits are isolated.
- Baseline gate: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 232 tests in 19.723s` / `OK`. This floor must only grow.
- Read: handoff, spec Round 2, bench_compare.py, theme_token_check.py,
  ui_smoke.py, package_offline.py, testrun.py, distill.py, checkpoint_guard.py,
  encoding_guard.py, commit_scoped.py, setup_dev_env.py, hygiene_check.py, and
  all 11 existing tools/*_tests.py for the monkeypatch/fake-subprocess model.

## Plan (RGB order)

1. UTB-5 red: bench_compare corrupt-baseline phase value -> traceback today.
   Green: validate baseline phase values in _handle_compare -> exit 2 + ERROR.
2. UTB-6 red: theme_token_check non-dict baseline JSON ([] today ->
   AttributeError). Green: load_baseline validates JSON object -> GitError ->
   exit 2 ERROR.
3. UTB-7 red: NaN/inf timeouts accepted by bench_compare._positive_float and
   ui_smoke.parse_arguments; poll_until_ready NaN deadline never fires.
   Green: reject non-finite at both CLIs (usage exit 2) + fail-fast guard in
   poll_until_ready (math.isfinite).
4. package_offline.py coverage (zero tests today): new
   tools/package_offline_tests.py with stdlib-only fakes (unittest.mock).
5. Error-path pins per handoff items 5-11 in the existing *_tests.py files.
6. Gates: full discover OK; encoding_guard --all; hygiene_check; scoped
   commits per logical step via commit_scoped.py.

## Checkpoint 1 - UTB-5 + bench_compare NaN half: RED then GREEN

- Red (`python -m unittest discover -s tools -p "bench_compare_tests.py"`):
  `Ran 54 tests ... FAILED (failures=4, errors=1)`. The error IS the audited
  crash: `ValueError: could not convert string to float: 'abc'` escaping
  main() from compare_medians (bench_compare.py:519 pre-fix). The three other
  reds show corrupt/non-finite values passing today (exit 0/1), and NaN/inf
  --timeout being accepted by the old `value <= 0` screen.
- Green: same command -> `Ran 54 tests in 0.348s / OK`. Pre-existing 46 tests
  untouched (bench-label rider 19349b3 era count 46 -> 54).
- Fix shape: new `_unusable_baseline_phase_values` screen called from
  `_handle_compare` after the existing "phases" dict guard (whole-object
  screening, so untracked bogus values cannot hide; float() still accepts
  numeric strings - no behavior drift there). `_positive_float` gained a
  `math.isfinite` gate (NaN<=0 is False; inf>0 - the old screen admitted both).
  Module docstring + USAGE_TEXT exit-2 lines updated to name the new
  internal-error member (handoff: keep exit contracts in the docstring).
- Deliberately NOT done: no validation in load_baseline/--update-baseline read
  (a corrupt foreign-class entry survives an --update-baseline merge; the
  compare path is where the value is consumed and where the crash was).

## Checkpoint 2 - UTB-6 theme_token_check non-dict baseline: RED then GREEN

- Red (`python -m unittest discover -s tools -p "theme_token_check_tests.py"`):
  `Ran 12 tests ... FAILED (failures=2)`, reproducing the audit crash verbatim:
  `AttributeError: 'list' object has no attribute 'get'` (and the `NoneType`
  variant) escaping at scan_frozen_counts:143 via main()'s narrow except
  tuple. Process exit 1 with a traceback where the contract says exit 2.
- Green: same command -> `Ran 12 tests in 2.980s / OK`.
- Fix shape: validate at the boundary -- load_baseline raises GitError("must
  contain a JSON object, got <type>") for a non-dict parse; main()'s existing
  except tuple turns it into `ERROR ...` + exit 2. Chose validation over
  widening the except tuple to catch AttributeError/TypeError so unrelated
  attribute bugs keep crashing loudly. Docstring exit-2 clause extended.

## Checkpoint 3 - UTB-7 ui_smoke NaN/inf half: RED then GREEN

- Red (`python -m unittest discover -s tools -p "ui_smoke_tests.py"`):
  `Ran 38 tests ... FAILED (failures=6)` -- every failure the audited
  acceptance class: parse_arguments kept nan/inf ("UsageError not raised"),
  main ran on past them ("--timeout" absent from the ERROR output), and
  poll_until_ready spun time.sleep on a NaN/inf deadline (loud forbidden-sleep
  fired; the spin is real, the test just refuses to hang for it).
  First red run had 4 harness errors (missing fixture_path attr, set by main()
  not parse_arguments) -- harness fixed, red re-run clean per the protocol.
- Green: same command -> `Ran 38 tests in 5.650s / OK`.
- Fix shape: `math.isfinite` gates at BOTH layers -- parse_arguments raises
  UsageError for non-finite/non-positive timeouts (usage exit 2), and
  poll_until_ready fails closed (returns False) for a non-finite budget as
  defense in depth for direct callers. Docstring + --timeout help updated.
- Same file also ships the audit row 11 pins (BuildFailureTests rc=1 ->
  EXIT_INTERNAL_ERROR + tail; RetryLoopTests exhaustion / mid-loop pass /
  non-retryable break) -- they were green on first run, honest as-is pins of
  existing behavior, so they ride in this commit (disclosed deviation from
  strict per-step staging: commit_scoped scopes whole files under tools/**).
- Deliberately NOT done: no restructure of parse_arguments -> run_smoke
  option plumbing (fixture_path assignment stays main()'s job; pins set the
  attrs the same way main does).

## Checkpoint 4 - package_offline coverage (zero tests -> 23)

- New tools/package_offline_tests.py: stdlib-only fakes (unittest.mock),
  REPO_ROOT/DIST_DIR/exe_suffix/detect_platform patched into temp trees,
  subprocess.run recorded. Covers: run() fail-fast SystemExit on rc!=0;
  rust binary reuse-vs-build branches (incl. set-order-tolerant cargo argv
  and the "build did not produce binary" SystemExit); npm/vsce/npx fallback
  chain incl. all-missing error paths and the node_modules skip; zip vs
  tar.gz platform selection incl. stale-archive replacement; staging tree
  (bin/vsix/installer/README, stale package-root replacement, missing
  installer skipped); main() flag->stage wiring + --no-archive + argparse
  exit 2.
- No production change: package_offline.py ships exactly the contract its
  docstring states; the suite is coverage-only (first tests this file ever
  had), green after fixing my own harness on second run (missed mixin
  setUp; installer copy is a silent copy2 so pinned by file content, not by
  an output line).
- Out-of-scope observation (reported, NOT fixed): create_archive derives the
  archive name with Path.with_suffix on a version-dotted pkg name
  ("ariadne-offline-0.1.0-win-x64" -> stem "ariadne-offline-0.1" ->
  "ariadne-offline-0.1.zip"); works but the dotted suffix truncation is a
  wart. Also `archive_base` in create_archive is computed and never used.
  Tests pin the CURRENT naming by consuming create_archive's return value.

## Checkpoint 5 - error-path pins (handoff items 5-11) + canonical gate

- Row 5 testrun (+3): cargo-missing OSError -> exit 2 with "cannot run cargo"
  on stderr; --capture without --distill parsed-but-ignored pin (no capture
  file written, raw mode verdicts normally).
- Row 6 distill (+1): child OSError -> exit 2 usage class, explicitly not
  the verdict exit 1.
- Row 7 checkpoint_guard (+4): run_scoped_command "red TIMEOUT" / "red
  ERROR" verdict rows; unclosed-fence SWALLOW pin with the closed-fence
  contrast (inside-fence branch never unwraps bullet/backtick forms);
  parse_resume_arguments duplicate --spec last-wins vs --scope/--cmd
  accumulation. All as-is characterization, none a fix request.
- Row 8 encoding_guard (+4): --root without value exit 2; --root missing
  dir / plain-file both usage; outside-a-git-repo -> exit 2 "git plumbing
  failed".
- Row 9 commit_scoped (+3): fnmatch '/'-crossing pinned AS-IS and named
  honestly (single-star crosses directory separators; literal prefix
  isolation survives; two-dir fake repo witness: scope "crates/alpha/*"
  commits ONLY crates/alpha/**). Advisory note for the spec: the crossing
  is safe in practice only because of the prefix, worth an owner decision
  if "**" vs "*" semantics ever get tightened.
- Row 10 setup_dev_env (+6): main() flag->argv wiring table (rustup
  components launched at the shutil.which RESULT path - one expected-value
  correction during red-checking, test-side only), npm ci-vs-install
  lockfile branch, --no-locked, --skip-npm (require_npm False recorded),
  --build appends cargo build --release, problem-list prints and exits 1
  without running anything.
- Row 11 ui_smoke: shipped in commit 89fb908 (build-fail EXIT_INTERNAL_ERROR
  + tail; retry-loop exhaustion/mid-loop-pass/non-retryable-break pins).
- No pins failed on first run EXCEPT the honest expected-value correction
  above; pins pin, they do not fix. Where the pinned behavior is wrong-ish
  (unclosed-fence swallow, single-star crossing) the test names say so.
- Canonical gate: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 296 tests in 21.863s` / `OK`. Floor 232 -> 296, monotonically grown
  across checkpoints (232 -> 251 -> 296), no existing test edited or
  weakened.
- Concurrent-session note: R1 ui hand landed af8a997/64314d7 on main after
  my 89fb908; my scopes (tools/** + this journal) never intersect; no
  index.lock collisions encountered.
