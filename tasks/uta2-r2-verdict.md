# UTA2-R2 Verdict - tools: crash-class hardening + package_offline coverage

**APPROVED** - reviewer 2026-10-19, fresh verification at HEAD `eaeaf2d`.
Handoff: `tasks/uta2-r2-handoff.md`. Journal: `tasks/uta2-r2-journal.md`.
Spec: `docs/specs/unit-test-audit.md` Round 2 (UTB-5/6/7).
Implementer commits (oldest-first): `89fb908`, `536a399`, `7a1ff41`, `eaeaf2d`;
base `19349b3`. Concurrent unrelated ui commits `af8a997`/`64314d7` interleave
in history and were EXCLUDED from this review (only the four named commits'
diffs judged; the journal discloses the interleaving and the scopes never
intersect).

## Gate lines observed (reviewer fresh runs, verbatim)

- `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 22.676s` / `OK`
- `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)` (R3 warn-only; every warn in crates/**, tests/integration/**, or reviewer scratch - none in any file this hand touched or created)
- `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`, capture 0 lines
- Live-CLI probe (reviewer-written, run once via distill, all true process exit codes):
  - `BC-ARGV-nan rc=2 out='ERROR --timeout must be a finite number > 0, got nan'`
  - `BC-ARGV-inf rc=2 out='ERROR --timeout must be a finite number > 0, got inf'`
  - `SMOKE-ARGV-nan rc=2 out='ERROR --timeout must be a positive finite number of seconds'`
  - `SMOKE-ARGV-inf rc=2 out='ERROR --timeout must be a positive finite number of seconds'`
  - `BC-CORRUPT rc=2 ... ERROR baseline entry for 'Windows-AMD64-...' has unusable phase value(s) ...: file_pass (event span): nan is not finite / zz phantom phase: 'abc' is not a number` (temp copy of the REAL `tools/bench_baseline.json` planted with a non-numeric untracked value AND a NaN tracked value; compare path executed with the bench runner injected - no cargo)
  - `HEX-CORRUPT rc=2 out='ERROR baseline file tools/theme_token_baseline.json must contain a JSON object, got list'` (throwaway git repo, `[]` baseline, real subprocess CLI)

## Per-commit review (oldest-first)

1. `89fb908` (fixes + red-first suites + row-11 pins): +437/-13; the 13
   deletions are all inside the three fixed scripts (docstring/contract lines
   and the replaced screens). Production: `_unusable_baseline_phase_values`
   called in `_handle_compare` after the existing phases-dict guard (whole
   object screened; `float()` parity for numeric strings; `math.isfinite`
   rejects NaN/inf) -> ERROR lines + `return 2`; `theme_token_check.load_baseline`
   raises GitError for non-dict parse; `ui_smoke.parse_arguments` +
   `poll_until_ready` isfinite gates. Contract docstrings/USAGE_TEXT name the
   new exit-2 members and match observed behavior (live witness above).
   Test files `+91/-0`, `+29/-0`, `+151/-0` - pure addition.
2. `536a399` (+485/-0): new `tools/package_offline_tests.py` (461 lines,
   23 tests) + journal checkpoint 4.
3. `7a1ff41` (+327/-0): pins only - six `*_tests.py` files, all pure
   insertion; no production file in the commit.
4. `eaeaf2d` (+39/-0): journal closeout only.

## Commit-vs-touch-set

All paths in the four commits: `tasks/uta2-r2-journal.md` + thirteen
`tools/*.py` / `tools/*_tests.py`. Nothing outside; **zero `crates/**` in any
of the four** (verified per-commit via `git show --stat`).

## Growth-only lock (232 -> 296), zero existing tests edited

Per-commit `--numstat` deletions column = 0 for every `*_tests.py` in all
four commits. Production deletions (13) confined to the three fixed scripts.
Reconciliation: 232 + 19 (fix tests: bench 8, theme 2, smoke 9) + 23
(package_offline) + 22 (pins: testrun 3, distill 1, checkpoint_guard 5,
encoding_guard 4, commit_scoped 3, setup_dev_env 6) = 296, matching the fresh
canonical line. File-size lock tail-read verified: bench_compare_tests.py 924
< 1000, ui_smoke_tests.py 579, package_offline_tests.py 461, bench_compare.py
837.

## Directed adjudications (independent, code-read + live witnesses)

1. **UTB-5 screen** catches non-numeric AND non-finite at the compare
   boundary incl. NaN (live `BC-CORRUPT` above - stronger than the suite's
   planted `inf`); untracked bogus values cannot hide (whole-object loop,
   test `..._even_for_untracked_phase`); ERROR text matches the docstring /
   USAGE_TEXT exit-2 clauses verbatim in kind. Prior 46 bench tests:
   `compare_medians` untouched (zero deletions there), screen replicates
   `float()` acceptance so any baseline that behaved sanely before behaves
   identically after; all green in the fresh 296. Deliberate scope call
   (no `--update-baseline` read validation) is disclosed and leaves no crash:
   a merged corrupt foreign entry is rejected exit-2 at compare.
2. **UTB-6 GitError reuse: ACCEPTED on merits.** This module's exit-2 channel
   IS the narrow `except (GitError, json.JSONDecodeError)` tuple (:223), and
   GitError already carries the baseline-missing contract (:162). A dedicated
   BaselineFormatError would land in the same tuple with the same message -
   no behavior difference; reuse keeps the tuple narrow so unrelated
   attribute bugs still crash loudly. ERROR-line message quality verified
   live: names file, requirement, and actual type (`got list`).
3. **UTB-7: both values, both CLIs rejected** (4 live rc=2 witnesses), and
   the poll guard cannot regress bounded behavior: it fires only on
   non-finite/non-positive budgets; a finite > 0 budget falls through to the
   byte-unchanged deadline loop. Guard liveness is provable: removing it
   re-reds `test_nonfinite_poll_deadline_fails_fast_without_sleeping` (fake
   sleep turns any spin into a loud AssertionError against a
   never-ready stub server).
4. **package_offline suite fidelity: PASS - not mock-to-green.** Every
   expectation checked against `package_offline.py` call sites: fail-fast
   `SystemExit` message (`exit 7` + command); reuse witness `calls == []`
   (cargo untouched) vs build argv prefix `["cargo","build","--release"]` +
   set-order-tolerant `--bin` pairs + cwd; reuse-path `shutil.which`
   AssertError-patched (proves which never runs); exact ORDERED argv list
   `npm install -> npm run compile -> vsce package --allow-missing-repository`
   and npx spelling `npx --yes @vscode/vsce package ...`; node_modules skip
   (`argv[1] == ["run","package"]`); all-missing error paths; zip vs tar.gz
   selection with REAL archive round-trips (zipfile/tarfile entry reads) and
   stale-archive byte replacement; real staging content checks (copy2 pinned
   by file content - honestly disclosed); `--no-archive` wiring + argparse
   exit 2. The `with_suffix` dotted-stem naming wart is pinned via the
   returned path and reported, not hidden.
5. **Pins rows 5-10 honest.** Row 9: `scope_matches` verified
   fnmatchcase-based (`commit_scoped.py:94`) - single-star crossing is real;
   test names state the surprise ("...crosses_directory_separators"), the
   two-dir e2e witness records exactly what commits, advisory-to-owner note
   included. Row 7: unclosed-fence swallow matches `checkpoint_guard.py:294-
   302` (inside-fence arm takes only direct command-prefix lines, never
   unwraps bullet/backtick forms) with the closed-fence contrast shipped as
   the control. Rows 5/6/8/10 assertions verified against
   `testrun.py:147`, `distill` OSError class, encoding_guard usage paths, and
   `setup_dev_env.py:70-102,140-155` (which()-resolved rustup argv[0] is the
   true production behavior the correction pins).
6. **Deviations** - all four dispositioned below.

## Deviation dispositions

- **D1 row-11 pins riding `89fb908` (the mixing deviation): ACCEPTED.**
  `ui_smoke_tests.py` had to ship UTB-7 tests in the fix commit anyway;
  commit_scoped scopes whole files, so the same file cannot split across two
  commits without a staging event it does not offer; the pins are pure
  insertion (`+151/-0`) and cannot mask the fix; disclosed in journal
  checkpoint 3 + deviations summary; the handoff's own commit gate says
  "adjust to truth".
- **D2 per-file gate via `discover -p "<file>"` instead of direct
  invocation: ACCEPTED.** The handoff's canonical gate is the discover form
  itself; the `-p` shape is the identical discovery engine; direct
  `python <script>` is off this session's allowlist (refusal re-witnessed in
  this review).
- **D3 setup_dev_env expected-value correction: ACCEPTED.** Code-proven true
  behavior (`which` result is argv[0]); correction was test-side only -
  `setup_dev_env.py` appears in NO commit of this hand; pins must pin truth,
  which is exactly what the red-check did.
- **D4 encoding `--staged` sequencing (post-commit `--all`): ACCEPTED** per
  the established chain (T1-T7b, R1 rider `64314d7`): commit_scoped stages +
  commits internally, so `--all` over tracked content at HEAD is a strict
  superset of `--staged`; reviewer's fresh `--all` at HEAD exit 0 covers every
  committed byte of this hand, including the new test file.

## Findings (numbered; none blocking)

1. **Minor (journal arithmetic).** `tasks/uta2-r2-journal.md` Checkpoint 5
   row 7 says checkpoint_guard "+4" but `7a1ff41` ships **5** new tests there
   (`ScopedCommandVerdictTests` x2 + `HandoffParsePinTests` x3 - the
   closed-fence contrast is its own test function). The totals reconcile
   anyway (19+23+22=64, `Ran 296`), so this is bookkeeping drift only; no
   change required.
2. **Minor (reviewer residue, not the hand's).** Deletion is denied to the
   reviewer in this session: the executed probe scratch
   `tasks/r2-probe.py` + `tasks/r2-probe-driver.py` were neutralized to 2-line
   comments and left UNTRACKED for closeout cleanup, same precedent as
   `tasks/uta-t4-reviewer-verify.py` and the t7b scratch.

## Follow-up dispositions (out-of-scope observations -> ledger deferred-findings at UTA2 closeout)

- `package_offline.create_archive`: `with_suffix` dotted-stem truncation
  ("ariadne-offline-0.1.0-win-x64" -> stem "ariadne-offline-0.1") + dead
  `archive_base` local (package_offline.py:186). Pinned-as-is + reported.
- fnmatch single-star `*` crosses `/` in commit_scoped scopes (safe today via
  the literal prefix; owner decision if `*`/`**` semantics tighten). Pinned +
  advisory.
- checkpoint_guard unclosed-fence swallow. Pinned + reported.
- `--update-baseline` can merge a corrupt foreign-class entry that compare
  then rejects exit-2 (no crash). Left as-is + reported.

## Verdict

**APPROVED.** Four commits inside the touch-set, plan-literal compliance on
all three fixes plus the package_offline coverage contract, growth-only 232 ->
296 with zero existing tests edited, both hygiene gates exit 0, four
declared deviations independently accepted, two minor findings (one journal
arithmetic, one reviewer-side scratch), no required changes.
