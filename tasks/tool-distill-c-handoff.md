# Handoff C — Tool Output Distillation: deferred findings (O1 ANSI-strip entry, B-gate exit-pin test)

Status: COMPLETE 2026-10-02 — C1 landed as `f3dc53b`; C2 test-only exit-pin landed green at base; see tasks/tool-distill-c-handoff-journal.md.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract). Base rev: HEAD at authoring (post-`bc8b89f`).
2. `tasks/milestone_ledger.md` deferred rows: O1 row (source tool-distill A
   gate) and the load-error-pin row (source tool-distill B gate).
3. `tools/testrun.py` (parse_cargo_output loop ~lines 70-91: SUITE_HEADER_LINE
   + TEST_RESULT_LINE matching, ANY_RESULT_MARKER drift sentinel) and
   `tools/testrun_tests.py` (existing fixtures + byte-identity lock; ANSI
   fixtures must be byte-constructed, never literals).
4. `tools/distilled.py` (ANSI stripping already lives there; `testrun.py`
   already imports `distilled` at top level — the single allowed direction;
   distilled must not import testrun at top level).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/tool-distill-c-handoff-journal.md (journal; always in-set)
- tools/testrun.py, tools/testrun_tests.py (both fixes)
- tools/distilled.py ONLY if a shared strip helper is needed (report if so)
- tasks/tool-distill-c-handoff.md (status lines only)
You MUST NOT touch: ledger/archive rows (orchestrator executes them at
closeout), `docs/**`, `.kilo/**`, `AGENTS.md`, `crates/**`, `tests/**`.
Frozen locks: none. No `git stash`, no branch change.

## What to build (two modules, two commits)

### C1 = O1: ANSI-strip at the raw pinned-parser entry (fix)
Red (in `testrun_tests.py`, fixtures built via byte escapes per UTF-8 rule):
1. Colorized PASS fixture (ANSI wrap on `test result: ok. ...` and `Running`
   header lines) -> `parse_cargo_output` returns correct counts, suites
   counted, NO `None`/drift. Must fail at base (drift).
2. Colorized FAILED variant -> counts parsed, exit verdict path correct.
3. Colorized DRIFT-shaped `test result:` line -> still drift: stripping must
   never HIDE drift (exit 2 preserved).
Green: strip ANSI per line at the matching entry inside `parse_cargo_output`
(both SUITE_HEADER_LINE and TEST_RESULT_LINE/ANY_RESULT_MARKER paths), reusing
the existing single-owner strip machinery from `distilled` (top-level import
from testrun to distilled is the sanctioned direction; do NOT duplicate the
regex). Default-mode stdout byte-identity lock must stay untouched and green;
echo remains raw bytes.
Refactor: one helper, honest name; no behavior change for plain text.
Commit: `fix(tools): ANSI-strip at pinned parser entry, colorized cargo no
longer trips drift (deferred O1)`.

### C2 = B-gate hardening test (test-only commit)
4. Direct pin: distill mode + `--baseline` pointing at a MISSING file ->
   header line shows `(exit 2)` AND process return code 2 together (tmp-path
   replay seam). Expect GREEN at base (mechanism already shipped in B); if it
   is unexpectedly Red, that is a finding: stop and report, do not redesign.
Commit: `test(tools): pin distill-mode baseline-load-error header/rc agree
(exit 2) (B-gate hardening)`.

## Gates (exact commands, timeout 600000 ms)
1. `python tools/testrun_tests.py` -- Red names first (C1), then `Ran N ...
   OK`; then again for C2.
2. `python -m unittest discover -s tools -p "*_tests.py"` -- `Ran N ... OK`.
3. Live: `python tools/testrun.py --distill -p ariadne-lang` -- quote digest
   header + canonical line + exit only.
4. `python tools/hygiene_check.py` exit 0; `python tools/encoding_guard.py
   --staged` exit 0 before EACH commit (ASCII everywhere, journal included).
5. `python tools/commit_scoped.py --scope <touch-set glob> -m "<message
   above>"` per module; never `git add .`.

## Discipline clauses
- Write/Edit tools only; pure ASCII; dogfood: cargo only via the gate-3
  distill command or replay seams.
- Journal 1-3 line checkpoint after each module Green gate.
- Context pressure: STOP early, return disk-state inventory.

## Report back (final message)
Commit hashes + `git show --stat` one-liners; C1 Red failing-test names then
`Ran N ... OK` lines; C2 green-at-base confirmation (or stop-and-report);
gate-3 digest header + canonical line quoted; deviations + reasons;
out-of-scope observations report-only. Digest-shaped evidence only, no raw
pastes.
