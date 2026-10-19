# UTA2-R2 Handoff - tools: crash-class hardening + package_offline coverage

Binding inputs:
1. This file (complete contract).
2. docs/specs/unit-test-audit.md "Round 2" section (UTB-5/6/7 fixes; gap rows 1-20).
3. tools/bench_compare.py (contract lines ~30-45, load_baseline ~475-484, compare_medians ~509-536, _positive_float ~556-563, _handle_* paths), tools/theme_token_check.py (load_baseline, main except-tuple), tools/ui_smoke.py (parse_arguments, poll_until_ready), tools/package_offline.py (whole), tools/testrun.py:131-149, tools/distill.py:144-166, tools/checkpoint_guard.py (run_scoped_command, fence parsing), tools/commit_scoped.py (scope_matches), tools/setup_dev_env.py.
4. Model: existing monkeypatch/fake-subprocess patterns in tools/*_tests.py.

## Touch-set (hard)
- tasks/uta2-r2-journal.md (journal; always in-set)
- tools/*.py and tools/*_tests.py
You MUST NOT touch: crates/**, docs of other tools beyond the fixed files' docstrings, .kilo/. No stash/branch change.

## What to build
FIXES (failing unit test first, each):
1. UTB-5 bench_compare: baseline phase value non-numeric (or phases non-dict) -> the module's exit-2 internal-error contract, never a traceback. Validate at load/compare boundary; red test plants {"phases":{"x":"abc"}} and asserts exit 2 + ERROR line.
2. UTB-6 theme_token_check: non-dict baseline JSON -> exit 2 ERROR (AttributeError/TypeError included or validated before .get); red test plants [].
3. UTB-7 NaN/inf args: ui_smoke --timeout nan|inf and bench_compare --timeout nan|inf rejected with usage-error exit 2; poll deadline logic also hardened (math.isfinite). Red tests first.

REQUIRED COVERAGE (package_offline.py currently ZERO tests):
4. Fake-subprocess suite: run() fail-fast SystemExit on non-zero rc; reuse-vs-build branch for the rust binary; npm/vsce/npx fallback chain incl. all-missing error path; zip vs tar.gz platform selection; --no-archive wiring; main arg parsing. stdlib-only fakes (unittest.mock).

REQUIRED PINS (pick from audit rows 5-20 by value; all cheap):
5. testrun cargo-missing OSError -> exit 2 "cannot run cargo"; --capture without --distill accepted-and-ignored pin.
6. distill child-OSError -> exit 2 usage-class (not 1).
7. checkpoint_guard run_scoped_command TIMEOUT and ERROR(OSError) verdict rows; parse_resume unclosed-fence swallow + duplicate --flag last-wins pins.
8. encoding_guard: --root missing/non-dir usage exit 2; run outside git repo -> exit 2 ERROR git plumbing failed.
9. commit_scoped scope_matches: pin the fnmatch '/'-crossing semantics AS-IS (named honestly; behavior advisory in spec) - test asserting which paths commit from a two-dir fake repo.
10. setup_dev_env: flag->argv wiring table + problem-list exit 1 (monkeypatch run/which).
11. ui_smoke: build-fail branch (rc=1 -> EXIT_INTERNAL_ERROR + tail); retry-loop sequence with stubbed _smoke_attempt.

## RGB, hygiene, locks
Red test first per fix; pins must never weaken existing 232 tests (suite may only grow). Keep every script's exit-code contract documented in its docstring where changed. Files stay <1000 lines; new tests may go in a new *_tests.py.

## Gates (timeout 300000 ms each)
1. `python -m unittest discover -s tools -p "*_tests.py"` (red first for each fix class, then all OK; quote the canonical `Ran N ... OK`)
2. `python tools/encoding_guard.py --all`; `python tools/hygiene_check.py` (R3 warns pre-existing only)
3. Commit: `python tools/commit_scoped.py --scope "tools/**" --scope "tasks/uta2-r2-journal.md" -m "fix(tools): uta2-r2 crash-class exit-2 contracts + NaN arg rejection; test(tools): package_offline + error-path coverage"` (adjust to truth).

## Discipline
Byte-safe edits only (Write/Edit tools). Journal checkpoint per gate. Out-of-scope: report.

## Report back
Commit hash + git show --stat; red evidence per fix; canonical unit-test lines before/after (232 -> N); deviations; out-of-scope observations.
