# T4b Rider Handoff - M28 stage5-collateral-memory-ceiling: bench_compare growth-gate demotion to tripwire

Hop id: `t4b-g-tripwire`. Small `tools/`-only rider of user decision 3
(2026-10-06): the growth gate G <= 2.0 is DEMOTED to a tripwire. Acceptance
is the absolute 5200 MiB peak bars (both profiles) + the Corpora field boot;
G keeps being computed and printed and NEVER fails a gate. Rationale (the
T0c verdict sec. 7, approved): the fixture is Theta(N^2) in edges by
construction - ratio 4.0-4.1 IS the quadratic payload shape, identical under
cap-0 ablation; RSS scaling with work is not a regression.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/stage5-collateral-memory-ceiling-t0c-verdict.md` sec. 7
   (G-gate grounding) and sec. 5 of the measurement doc (the unclosable-
   against-this-fixture reading) - the decision's evidence base.
3. `tools/bench_compare.py` - the exact sections you edit: exit-code
   docstring :30-37; `STAGE5_MEMORY_GROWTH_GATE_RATIO` + comment :185-186;
   `stage5_memory_bar_verdict` :397-418 (the growth branch :410-417 sets
   `regressed = True` today - that is the FAIL you remove);
   `_run_stage5_memory_gate` verdict_tail :761-763; USAGE_TEXT :229
   ("1 regression or ratified-bar exceedance" wording).
4. `tools/bench_compare_tests.py` - `stage5_memory_canonical_values`
   docstring :344-351 (mentions "under the ratified gate G = 2.0"); the
   peak-bar quadrant test (:595-605 region) and
   `test_rated_growth_gate_boundary_passes_and_exceeding_regresses`
   (:606-619) - your Red carriers.
5. `tasks/stage5-collateral-memory-ceiling-t0b-handoff.md` sec. "RATIFIED
   gate contract" + `tasks/stage5-collateral-memory-ceiling-t0b-verdict.md`
   decisions (bars enforced compare-path-only, pre-PR-2 exit-0 no-baseline
   hint, the 1.50 choice) - the semantics you PRESERVE.
6. `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` sec.
   RATIFICATION + ROUND-2 ERRATUM, esp. the `RATIFIED` growth-gate line
   (:298-299) - the line your appended note supersedes (note discipline
   below).

## Objective and invariants

Objective: the stage5-memory compare path no longer FAILS on G. The
growth-ratio median exceeding 2.0 prints a loud TRIPWIRE line and does not
contribute to the exit code. Everything else is UNCHANGED:
- The 5200 MiB peak bar still fails the compare path (exit 1 + the RATIFIED
  WARN line) exactly as landed at `ac0eb7e`.
- The timing mode (`stage5-timing`, 1.25x) is byte-behavior-identical.
- The memory mode's 1.50x per-row-vs-baseline regression semantics
  (including for the ratio row as a BASELINE row) are unchanged - the
  demotion touches only the ABSOLUTE gate.
- Sampling law unchanged: median-of->=5 accepted, >8% operand rejection both
  operands both scales, <5 accepted -> exit 2 `insufficient samples`,
  multiset peak parsing.
- Pre-PR-2 shape preserved: absent baseline -> exit 0 + the no-baseline
  hint; the bars/tripwire fire independently of baseline values.
- `tools/bench_baseline.json` untouched (operator PR-2 only).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/stage5-collateral-memory-ceiling-t4b-journal.md` (checkpoint
  journal; always in-set; create alongside)
- `tools/bench_compare.py` (sections 3 above; keep
  `STAGE5_MEMORY_GROWTH_GATE_RATIO` as the TRIPWIRE threshold constant -
  rename is allowed if the diff stays honest, the value 2.0 stays)
- `tools/bench_compare_tests.py` (invert the G-quadrant tests red-first)
- `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` - APPEND-ONLY
  supersession note (exact contract below). You must not rewrite, reorder,
  or reflow ANY existing line of this file: prove by
  `git diff --numstat` showing 0 deletions on the file.

You MUST NOT touch: `tools/bench_baseline.json`; `crates/**` (the bench
file and its labels are frozen); `tools/bench_compare.py`'s timing mode
semantics; any other `tools/*` (`ui_smoke.py` is T3's landed file);
`tasks/milestone_ledger.md`; any other tasks doc.

Single-session rule (stated per the user decision): the SAME implementer
session that lands this hop authors BOTH the bench_compare change AND the
append-only supersession note on the T0a evidence doc, in one scoped commit.
The evidence doc is the milestone's supplier artifact kept under append-only
discipline (its own RATIFICATION section header: "append-only -- nothing
above is rewritten"); this hop continues that practice as the doc's current
owning session - no separate doc-editing hop, no rewrite of pre-existing
lines. The evidence doc's G-RATIFIED line is superseded by APPENDIX, not by
editing the line itself.

## What to build

1. `stage5_memory_bar_verdict`: the growth-ratio branch keeps computing and
   keeps speaking, but never sets `regressed`. Print line (exact prefix
   law, built from the existing label constant so the label text stays
   tuple-identical):
   `TRIPWIRE stage5 RSS growth ratio (debug, N=640->1280): <median> exceeds
   the demoted growth gate 2.0 - tripwire only, never a gate (user decision
   2026-10-06; the fixture is Theta(N^2) by construction, see t0c-verdict
   sec. 7)`. Value formatting as today (`:.2f`-class). The line lands in
   the WARN/verdict tail region where the old WARN line printed, so live
   runs always surface the reading.
2. Constant + comment: cite the 2026-10-06 user decision where the comment
   today says "RATIFIED growth gate" (no bare magic-number re-justification
   needed - it is a demoted ratified constant; say exactly that, with the
   date and the reason pointer).
3. Docstring/usage/verdict_tail: exit-1 wording names the peak bar only;
   the pass-line tail names "the RATIFIED peak bar (5200 MiB)" and mentions
   the growth tripwire. Do not delete the growth number from the pass line
   - the reading must stay visible.
4. Evidence-doc appended note (after the existing RATIFICATION/ERRATUM
   sections, file end), dated 2026-10-06 decision / 2026-10-07 landing,
   stating verbatim-in-effect: this note SUPERSEDES the `RATIFIED growth
   gate G: stage5 RSS growth ratio <= 2.0` line above (append-only
   discipline: that line recorded the pre-decision state, correct when
   written); the user decision 2026-10-06 DEMOTES G to a tripwire -
   computed and printed, never gate-failing; acceptance = the 5200 MiB
   peak bars (both profiles) + the Corpora field boot; N0, scales, peak
   bars, sampling protocol, and the 4096-rider probe remain exactly as
   ratified; cite `tasks/stage5-collateral-memory-ceiling-t0c-verdict.md`
   sec. 7 as the grounding and this handoff as the landing contract.

## RGB protocol

1. Red (invert the G quadrants FIRST, against current code):
   `test_rated_growth_gate_boundary_passes_and_exceeding_regresses` - the
   exceeding arm must now expect: exit code NOT 1 (0 when the baseline
   comparison itself passes, mirroring the test's fixture) + output
   contains `TRIPWIRE` and the ratio-row label; the boundary/exact-2.0 arm
   stays pass and asserts NO tripwire line (strict-exceed condition, same
   strictness as the old gate). Expected Red failure text today: exit_code
   is 1, the test wants 0; and the `TRIPWIRE` line is absent (today's line
   says `WARN ... RATIFIED growth gate`). Quote both in the journal before
   touching bench_compare.py.
2. Green: implement sections 1-4. NEVER weaken the PEAK-bar quadrant test
   (:595-605 region) - it must remain UNEDITED (byte-identical in the diff;
   the reviewer proves it) and green; it is the guard that the demotion
   touched only G. The canonical-values docstring sentence at :344-351 may
   be reworded minimally (it is a comment), keeping the numbers.
3. Refactor: names honest (`TRIPWIRE`, no abbreviation); `bench_compare.py`
   is 789 lines (ceiling 1000, margin 211 - this hop is a few lines; if
   anything balloons it, you are out of contract); functions < 100.

## Required tests

- Inverted exceeding arm (above) - tripwire printed, exit unaffected.
- No-tripwire-at-or-under arm.
- Tripwire independence from baseline: exceeding ratio median with the
  ratio row absent from the baseline still exits per the baseline rules
  (no-baseline hint class), tripwire line present. (One small case; pin it.)
- Peak-bar quadrant test UNEDITED and green (diff proof + run).
- Sampling-protocol and mode-isolation tests UNEDITED and green.
- File-local count discipline: quote the prior `Ran N` from a GREEN run
  before inverting (row-65 law); the landing line is the same N plus any
  case you ADD (inversions alone keep N constant).

## Gates (exact commands; timeout 600000 ms; distilling entry points only -
ledger row 61)

1. `python tools/distill.py --kind generic -- python -m unittest discover
   -s tools -p "bench_compare_tests.py"` - RED state quoted (accepted M28
   T3 D1 precedent: CI discover form scoped to the one file; child cwd is
   pinned by `distill.py:149`).
2. Same command - `Ran N ... OK` green.
3. `python tools/distill.py --kind generic -- python tools/bench_compare.py
   --help` - exit 0, shows both `--bench` choices; NO live bench run is an
   implementer gate (the live run is PR-2-bound and reviewer-side).
4. `python tools/hygiene_check.py` - exit 0.
5. `python tools/encoding_guard.py --staged` - exit 0.
6. `python tools/commit_scoped.py --scope tools/bench_compare.py --scope
   tools/bench_compare_tests.py --scope
   tasks/stage5-collateral-memory-ceiling-t0a-evidence.md --scope
   tasks/stage5-collateral-memory-ceiling-t4b-journal.md -m "feat(tools):
   bench_compare stage5-memory demotes growth gate to tripwire per user
   decision 2026-10-06 + append-only supersession note (M28 T4b)"`.

Total implementer invocations: <= 6. This hop is a pure-Python + docs hop:
zero cargo delta; the workspace census must not move.

## Parallelism note

T4b is PARALLEL-SAFE with T4 (`tasks/stage5-collateral-memory-ceiling-t4-
handoff.md`): the touch-sets are DISJOINT files (T4: `crates/**`,
`tests/golden` pair, `crates/index`, the pins; T4b: `tools/bench_compare*`,
one tasks doc, the evidence doc). Two handoffs that touch the same file are
never parallel; these share none. If T4b lands first, the T4 reviewer's
live-gate expectation note ("pre-T4b G-fail") no longer applies - record
which order landed.

## Acceptance (machine-checkable verdict lines)

- Gate 2: `Ran N ... OK` (N per the file-local discipline).
- `git show --stat <landing>` == touch-set exactly; `bench_baseline.json`
  absent from every diff; `git diff` of `bench_compare_tests.py` shows the
  peak-bar quadrant test with ZERO edited lines; `git diff --numstat` of
  the evidence doc shows 0 deletions.
- Grep-proof: `TRIPWIRE stage5 RSS growth ratio` reachable in
  `bench_compare.py` source; the old growth `WARN ... RATIFIED growth gate`
  string gone; the peak-bar WARN string intact verbatim.
- Reviewer owns: CI-form tools discover `-p "*_tests.py"` (`Ran 218...`
  prior-total class, M28 T3 verdict finding 1: the `-p` pattern is
  REQUIRED); workspace census UNCHANGED `1250 passed / 0 failed / 13
  ignored / 91 suites` (Python-only hop - zero cargo delta is the proof);
  `encoding_guard --all`; commit-vs-touch-set; and the post-PR-2 live gate
  expectation: when the operator PR-2 + live run happen, G exceedance must
  print TRIPWIRE without exit-1 contribution (peak-bar exceedance still
  exits 1).

Reviewer-gate reference: adjudicated via the `review-gate` skill; its
commit-vs-touch-set check enforces this file; the reviewer additionally
adjudicates the append-only discipline of the evidence-doc note (0
deletions, dated, cites decision + grounding) and the single-session rule
(one commit authoring both changes).
