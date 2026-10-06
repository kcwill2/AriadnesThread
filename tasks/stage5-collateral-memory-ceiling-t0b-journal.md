# T0b journal - M28 stage5-collateral-memory-ceiling: `bench_compare --bench stage5-memory`

Hop id `t0b-gate-mode`. Touch-set honored exactly: `tools/bench_compare.py`,
`tools/bench_compare_tests.py`, this journal. Nothing else was edited.
Inputs read in handoff order: handoff, shape sec. 3.1 + 6
(`.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`),
evidence doc (RATIFICATION + ROUND-2 ERRATUM, "Remain PROPOSED", E1/E3),
`crates/cli/tests/stage5_memory_bench.rs`, `tools/bench_compare.py` + tests,
distillation contract.

## Ground truth at dispatch

- HEAD `69138db` (T1 `d6640a2` and T2 `e52a9e9` already landed; the bench
  labels are T0a-stable and unchanged by T1/T2).
- `tools/bench_baseline.json` contains NO stage5-memory keys: the pre-PR-2
  exit-0 `no baseline for machine class` hint is the expected live shape and
  is pinned by unit test.
- M27 T1b has NOT landed: the `boot-stages` choice is not implemented here;
  a test asserts `--bench boot-stages` is a usage error (exit 2), so this
  hop does not preempt T1b's touch of the shared file.
- Existing `bench_compare_tests.py` case count: 21 (all green at baseline).

## RED (file-order per handoff "RGB protocol" 1a-1d)

New tests appended to `tools/bench_compare_tests.py` (24 new methods;
existing 21 UNEDITED). Gate 1 RED digest (verbatim):

    ### generic FAIL (exit 1)
    Ran 45 tests in 0.282s
    FAILED (failures=4, errors=22)
    capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (340 lines)

Failure causes matched the handoff's predicted class exactly:
- (a) parse tests: `AttributeError: module 'bench_compare' has no attribute
  'parse_bench_table_multiset'`; mode selection -> `ERROR unknown argument
  '--bench'` exit 2 ("unknown --bench choice -> usage exit 2 (the mode does
  not exist yet)").
- (b) ratio tests: AttributeError on `STAGE5_MEMORY_REGRESSION_RATIO`,
  `STAGE5_MEMORY_PEAK_BAR_MIB`, `STAGE5_MEMORY_GROWTH_GATE_RATIO`.
- (c) tuple test: AttributeError `DOCUMENTED_PHASES_STAGE5_MEMORY`.
- (d) protocol tests: AttributeError `STAGE5_MEMORY_DIVERGENCE_FRACTION`,
  `stage5_memory_baseline_key` (baseline-key tests share this class).
One new test passed already at RED by design:
`test_default_parse_keeps_strict_last_write_wins_contract` pins the
UNCHANGED default-mode contract (it must pass before and after).

## GREEN

Implementation in `tools/bench_compare.py` (additive + parameterized):
- `DOCUMENTED_PHASES_STAGE5_MEMORY`: 46 concrete debug labels built from
  the 20 scale templates x {640, 1280}, the 26th template's growth-ratio
  row at default scales, and the 5 warm templates (26 distinct templates =
  25 `format!("... {suffix}")` grep stems + the ratio template; exactness
  pinned by grep against the bench source).
- `parse_bench_table_multiset` shares `_parse_pipe_row` (identical drift
  messages/grammar) with the unchanged-behavior `parse_bench_table`.
- RATIFIED protocol: `reject_divergent_peak_invocations` = per-slot
  provisional medians over every peak sample, wholesale invocation
  rejection on >8% operand divergence (both operands, both scales,
  inflation AND deflation), loud `REJECT ...` lines +
  `stage5-memory sample accounting: accepted K of N invocations` line,
  medians recomputed over accepted invocations only.
- Gate 1 GREEN digest (verbatim):

    ### generic PASS (exit 0)
    Ran 45 tests in 0.308s
    OK
    capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (5 lines)

- GREEN iteration (fixture, not production): three ratio-row tests failed
  first because the bench prints every value at 1-decimal precision
  (`print_row` `{:>12.1}`); 1.92/1.51/2.01 are not printable. Fixtures now
  move ratio rows on the bench's own 0.1 grid (1.9 canonical; 1.6 and 2.1
  past-boundary). Production code unchanged by the fix.
- Refactor + gates re-run (verbatim):

    Ran 45 tests in 0.325s
    OK

- Gate 2 (verbatim, exit 0, first line of usage):

    usage: python tools/bench_compare.py [--bench {stage5-timing,stage5-memory}] [--runs N] [--timeout SECONDS] [--baseline PATH] [--update-baseline]

## Hop decisions the handoff assigned to this implementer

1. Regression ratio CHOSEN = 1.50 (constant
   `STAGE5_MEMORY_REGRESSION_RATIO` carries the full justification
   comment). Cite, evidence doc sec. "Remain PROPOSED": "The 1.15x
   cross-machine tolerance stays PROPOSED: it is owned by T0b gate design,
   pending PR-2." The handoff demands wider than 1.25x; the observed
   residual RSS envelope (+12% half-scale bimodality, -14% full-side
   trimming per E1 + round-2 erratum) is far outside the ms mode's
   0.06%/0.13% in-band spread, and no second-machine baseline exists
   pre-PR-2. 1.50 is exactly representable in binary floating point, so
   the pinned boundary tests compare exactly. Pre-fix class (4.0 growth,
   ~9040 MiB) fails it by miles. Reviewer adjudicates.
2. <5-accepted path CHOSEN = exit 2 with `ERROR insufficient samples: ...`
   (pinned by two tests). Rationale: the reviewer-note says "never gate on
   <5" -- an exit-0 WARN line would read as a pass to every caller and
   silently soften the ratified floor; exit 2 marks "no verdict" honestly,
   consistent with the tool's existing exit-2 "never substitute zeros"
   philosophy.
3. The two RATIFIED absolute gates (peak 5200 MiB at N=1280 debug, growth
   ratio <= 2.0) are enforced in the COMPARE path (with the baseline
   regression check), so a machine verdict cannot exceed gate law; they
   deliberately do NOT fire in the no-baseline hint path, keeping the
   pinned pre-PR-2 acceptance shape "exit 0 + no-baseline hint" true.
4. Insufficient-samples check precedes any baseline read/write: fewer than
   5 accepted invocations is not a verdict regardless of baseline state
   (pinned by a test that places a valid baseline file and asserts neither
   the hint nor the pass line appears).
5. `--runs` < 5 is NOT rejected at parse time: the honest rejection is the
   insufficient-samples path AFTER accounting (a partial-acceptance run
   still gets loud rejected-sample accounting). Memory mode default runs =
   5; timing default stays 3.

## Default-mode invariance evidence

- All 21 pre-existing tests UNEDITED and green (they exercise the strict
  one-row-per-label parse, ms WARN formatting byte-for-byte, exit codes,
  baseline update merge behavior).
- `test_default_parse_keeps_strict_last_write_wins_contract` adds an
  explicit duplicate-label pin on the default parser.
- `test_explicit_timing_choice_behaves_as_the_default_mode` proves
  `--bench stage5-timing` reaches the unchanged verdict line
  `no regressions past 1.25x baseline`.
- `test_default_mode_never_gates_on_memory_rows`: fed bench-memory output,
  the default mode exits 2 naming its own missing rows, prints no table.
- `--help` is a NEW exit-0 path required by Gate 2; absent `--help`,
  behavior is unchanged (an unknown argument still exits 2 naming it).
- No live bench run by the implementer (handoff: not an implementer gate).

## Deviations

- D1 - Inter-def comment lines in `bench_compare.py`. `hygiene_check.py`'s
  Python span finder does not close a module-level def's span when the
  FOLLOWING `def` line carries the signature's opening paren (multi-line
  def signature); HEAD already tolerates this quirk (max inflated span 53).
  My added multi-line signatures chained the swallow to a recorded 137-line
  span -> R1 BLOCKING. Fix is behavior-neutral: one honest module-level
  comment line before each multi-line def (a column-0 line closes the
  span). `hygiene_check.py` (not in touch-set) untouched.
- D2 - Gate 3 was additionally run through `distill.py --kind generic` to
  capture the exit-code line (`### generic PASS (exit 0)`); the direct
  invocation was also run; both list only pre-existing R3 warnings in
  foreign files -- none in this hop's files.
- D3 - `bench_baseline.json` untouched (verified by `git diff` empty in the
  landing diff); rows land only via operator PR-2 `--update-baseline`
  against the new `::stage5-memory` key.

## Deliberately NOT done

- No `boot-stages` mode / no `--bench boot-stages` acceptance (M27 T1b's
  pending work on the same files).
- No bench-side 4096 MiB rider probe row (would edit `crates/**`,
  outside touch-set). Absent a planner touch-set amendment, the mode
  tolerates an unknown extra probe row class: it parses, gets a median
  table row, and the baseline comparison warns `WARN untracked phase`
  (pinned by `test_unknown_probe_row_is_reported_not_fatal`).
- No 1.15x tolerance plumbing (stays PROPOSED per the evidence doc).
- No refusal logic on `--update-baseline` recording a bar-violating run:
  out of handoff scope; the operator flow (VERIFIED-IDLE, post-fix tree)
  is the protection, and the reviewer sees the accounting lines.
- No automatic re-runs when samples are rejected: the operator raises
  `--runs`; the gate only reports honestly.
- No CI wiring (DP-3: RSS gate is LOCAL-only; I7c memo-counter cap test
  lives in T1, already landed).

## Gated on operator PR-2 + free disk

- The LIVE verdict line (`PASS-or-WARN honest line` after baseline) needs
  operator `bench_compare.py --bench stage5-memory --update-baseline` on
  VERIFIED-IDLE (PR-2), which also needs disk for the N=1280 fixture temp
  dirs across >= 5 isolated invocations. Pre-PR-2 the live form is the
  pinned exit-0 no-baseline hint.
- Reviewer-side obligations (live gate run, workspace census, full CI-form
  discover, ui_smoke, encoding_guard --all) per handoff "REVIEWER hop
  obligations" - not implementer gates.

## Gates (verbatim digests) + commits

- Gate 3: `python tools/hygiene_check.py` and distill form ->
  `### generic PASS (exit 0)` (capture: target/distill-capture.txt).
- Gate 4: `python tools/encoding_guard.py --staged` (distill form) ->
  `### generic PASS (exit 0)` (capture empty: no findings).
- Gate 1 final (committed tree, content-identical to the GREEN run):
  `### generic PASS (exit 0)` / `Ran 45 tests in 0.325s` / `OK`.
- Gate 2: `python tools/distill.py --kind generic -- python tools/bench_compare.py --help`
  -> `### generic PASS (exit 0)`, first usage line shows
  `--bench {stage5-timing,stage5-memory}`.
- CI-form workspace discover (`-p "*_tests.py"`, reviewer form, run on the
  landing tree): `### generic PASS (exit 0)` / `Ran 218 tests in 20.830s`
  / `OK` -- 194 prior workspace cases + 24 new = 218.
- Scope proofs: `git show --stat ac0eb7e` = exactly the three touch-set
  files (journal 190/0, bench_compare.py 495/56, bench_compare_tests.py
  475/0 -- ZERO deleted test lines proves the 21 existing cases are
  UNEDITED); `git diff ac0eb7e^..ac0eb7e -- tools/bench_baseline.json` is
  empty (exit 0, no output).
- Commits (via `tools/commit_scoped.py`, scopes = touch-set):
  - `ac0eb7e` - `feat(tools): bench_compare --bench stage5-memory gate
    mode + unit-locked ratio math (M28 T0b)` -- mode + 24 tests + this
    journal (RED->GREEN lands as one unit; the handoff's acceptance is the
    paired file set).
  - `<this commit>` - journal records the landing hash (this file, alone).
