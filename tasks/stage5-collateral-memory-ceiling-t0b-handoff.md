# T0b Handoff - M28 stage5-collateral-memory-ceiling: `bench_compare --bench stage5-memory` gate mode

Hop id: `t0b-gate-mode`. Pure `tools/` hop: teaches `bench_compare.py` the
new memory-gate mode so T0a's measured curve becomes a machine-checkable
regression gate. The RSS curve gate is LOCAL-gate-only (DP-3, user-resolved
2026-10-05): GitHub-runner RSS is incomparable; the CI accompaniment is the
deterministic memo-counter cap test (I7c), which lives in T1 - no wall-time
or RSS number ever runs in CI.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. Shape `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`
   sec. 3.1 (the exact grammar: values MiB, profile IN the label, ratio rows
   first-class, per-profile rows keyed like today's machine-class keys, own
   ratio) and sec. 6 (the label/grammar table; the ratification OUTCOME is
   now in the evidence doc, item 3).
3. `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` - THE SUPPLIER
   (canonical name per the evidence-doc header and T0a round-1 verdict
   finding 2; the T0a handoff's `...-t0-evidence.md` spelling never existed
   on disk - pointer errata, planner-corrected 2026-10-06): the RATIFIED
   gate constants + protocol are read from sec. RATIFICATION + ROUND-2
   ERRATUM (2026-10-06, landing `39da2e9`/`afcd038`); the verbatim 26-label
   tuple from sec. "Verbatim label set (T0b row contract)"; the parse-shaping
   multiset consequence from sec. E1. THE GATE RATIO IS EXPLICITLY NOT
   RATIFIED (sec. "Remain PROPOSED": the 1.15x cross-machine tolerance is
   owned by T0b gate design, pending PR-2) - the hop chooses the ratio with
   a written justification note and the landing reviewer adjudicates.
   Do NOT invent labels.
4. `crates/cli/tests/stage5_memory_bench.rs` (landed T0a) - the literal
   labels the tuple must mirror, run-for-run.
5. `tools/bench_compare.py` (full file: existing mode,
   `DOCUMENTED_PHASES` :43-58, parse :88-109, baseline keying +
   `--update-baseline`, exit-code contract :10-14) and
   `tools/bench_compare_tests.py` (test style to extend).
6. `docs/specs/tool-output-distillation.md` (digest shapes you cite).

## Preconditions (hard - encoded here, enforced at dispatch)

1. SATISFIED 2026-10-06. T0a landed APPROVED (round-1 REJECTED at `8d34a3c`
   - blocker: release growth-ratio row not decomposable; rework
   `407fe66`/`eec5646`/`69885ce` - round-2 APPROVED at `69885ce`, verdict
   `tasks/stage5-collateral-memory-ceiling-t0a-verdict.md`). Its bench
   labels are STABLE and fixed verbatim: 26 labels, evidence-doc sec.
   "Verbatim label set (T0b row contract)"; any rework that changes labels
   re-triggers the evidence review first.
2. SATISFIED 2026-10-06 with one carve-out. The DP-4 written evidence
   review CONCLUDED and its RATIFICATION landed (`39da2e9`/`afcd038`,
   evidence-doc sec. RATIFICATION + ROUND-2 ERRATUM). Ratified for this
   hop: N0 = 1280 fixture files (scales {640, 1280}); peak bars
   `stage5 peak RSS` <= 5200 MiB per profile at N=1280; growth gate
   `stage5 RSS growth ratio` <= 2.0; protocol median-of->=5 isolated
   invocations with >8% per-slot-median rejection on BOTH operands at
   BOTH scales. CARVE-OUT: the regression ratio is NOT ratified (the
   1.15x cross-machine tolerance stayed PROPOSED) - THIS hop owns the
   ratio choice with a written justification note; the landing reviewer
   adjudicates it.
3. REMAINING PRECONDITION (OPEN, operator-side): operator PR-2 -
   `bench_compare.py --update-baseline` on VERIFIED-IDLE - must land
   before the LIVE `--bench stage5-memory` gate row carries meaning;
   pre-PR-2 the no-baseline exit-0 hint is the EXPECTED shape (invariant
   above). The mode itself may be built and unit-tested BEFORE PR-2 -
   dispatch timing is the orchestrator's call, not an implementer STOP
   for the unit-testable work.
4. Shared-file serialization: M27 T1b
   (`tasks/tu-index-hang-throughput-t1b-handoff.md`, still PR-2-bound at
   authoring) touches the SAME files (`bench_compare.py`,
   `bench_compare_tests.py`, boot-stages tuple). The orchestrator MUST
   sequence this hop and T1b one at a time against a common tree; this hop
   adds ONLY the `stage5-memory` mode and its own tuple - it does not
   pre-author, rename, or reserve the boot-stages tuple. If T1b landed
   first, rebase your diff on its outcome; if you land first, T1b rebases
   on yours.

## Objective and invariants

Objective: `python tools/bench_compare.py --bench stage5-memory` runs the
ignored `stage5_memory_bench` (debug profile per today's default run
mechanics; the RELEASE column of the table is evidence-review data, not a
gate run the machine re-executes unless the ratified gate contract (below)
says otherwise),
parses `label | MiB` rows against a new `DOCUMENTED_PHASES_STAGE5_MEMORY`
tuple, takes medians, and compares per-row against per-machine-class
baseline entries with THIS HOP'S regression ratio, chosen with a written
justification note (wider than 1.25x for RSS noise/fragmentation; the 1.15x
cross-machine tolerance stayed PROPOSED at the DP-4 review and is owned by
T0b gate design per the evidence doc's "Remain PROPOSED" clause - cite the
clause in the journal; the landing reviewer adjudicates the choice). The
growth-ratio rows (I7b: `RSS(N0)/RSS(N0/2)`) are checked as first-class
rows, under the ratified sampling protocol (see RATIFIED gate contract
below). Unit tests pin the parse, the unit declaration, the ratio math, the
per-slot-median + >8% divergence-rejection protocol, and the new tuple's
exactness against the bench source labels.

## RATIFIED gate contract (2026-10-06 DP-4 review; from the evidence doc
sec. RATIFICATION + ROUND-2 ERRATUM, landing `39da2e9`/`afcd038`)

- `RATIFIED:` N0 = 1280 fixture files; scales {640, 1280}.
- `RATIFIED:` peak-RSS bars: `stage5 peak RSS` <= 5200 MiB per profile at
  N=1280.
- `RATIFIED:` growth gate G: `stage5 RSS growth ratio` <= 2.0.
- `RATIFIED:` gate protocol: median-of->=5 isolated invocations; ANY
  invocation whose operands diverge >8% from the per-slot median is
  REJECTED - binding on BOTH operands at BOTH scales. Implementing this
  means: per-slot medians over >= 5 ACCEPTED invocations; a >8%
  divergence check on every ratio operand and every peak slot; loud
  rejected-sample accounting in the digest (never silent exclusion); and
  fewer than 5 accepted invocations reading as exit 2 OR an explicit
  not-enough-samples WARN line - implementer picks one and pins it in
  tests.
- MULTISET parsing (RATIFIED-era consequence, round-2 reviewer finding 3
  relay): `stage5 peak RSS` labels occur TWICE per scale in suite runs
  (the curve test's child + the ratio test's child - two honest samples
  of one quantity). The memory mode's per-label parse collects a MULTISET
  per label and feeds every sample to the per-slot median; a duplicate
  label is NOT parse drift there. The existing default mode keeps its
  strict one-row-per-label contract byte-identical (invariant above).
- Ratio-row label is parameterized: `stage5 RSS growth ratio (release,
  N=640->1280)` at the default scales - grep the landed bench for the
  tuple; 26 verbatim table-row labels = the tuple source (evidence-doc
  "Verbatim label set (T0b row contract)" sec.).
- `RATIFIED:` the ~4096 MiB save-span transient probe as a T0b/T1 rider
  probe. Bench-side arrival of a probe row would edit
  `crates/cli/tests/**` - OUTSIDE this touch-set - so it can reach T0b
  only via a dated planner touch-set amendment at dispatch; absent that
  amendment, the gate mode simply tolerates an unknown extra probe row
  class per its parse contract (report, don't crash).

Invariants:
- Existing default mode is byte-behavior-identical: same grammar, same
  `DOCUMENTED_PHASES` semantics, same exit codes (0 clean / 1 regression /
  2 parse-drift-or-usage). `--bench stage5-memory` selects a separate mode;
  absent the flag, output is unchanged. The parse contract stays exactly
  `label | number` (one pipe) - identity lives in labels, units are owned by
  the gate mode (MiB declared in the mode's docstring and output header).
- `tools/bench_baseline.json` is FROZEN to this hop: baseline rows land only
  via operator `bench_compare.py --update-baseline` on VERIFIED-IDLE
  (PR-2), never hand-edited. ESCALATION WARNING inherited (ledger deferred
  row 61, bench-baseline stale/flapping, twice escalated): the reviewer
  MUST expect the new `stage5-memory` rows to be absent (exit 0
  "no baseline for machine class" hint stands) or freshly keyed; do NOT let
  the implementer "fix" staleness by editing the JSON, and do NOT treat a
  first-run no-baseline exit 0 as failure.
- Fixtures: zero fixture demand - the gate mode shells the same ignored
  cargo bench (temp dirs only). No new files inside `tests/fixtures/basic`
  or `tests/fixtures/expensive`; anything fixture-shaped routes through the
  documented frozen-set approval path (M27 T0b dated-row precedent).
- I1-I4 frozen floors: untouched by a `tools/` diff; reviewer proves by diff
  scope.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/stage5-collateral-memory-ceiling-t0b-journal.md` (checkpoint
  journal; always in-set; create alongside)
- `tools/bench_compare.py` (add the mode, tuple, per-profile baseline
  keying, the hop-chosen ratio constant and the protocol params
  (>=5-sample window, 8% divergence) each WITH a justification comment -
  ratio: the hop's written choice citing the evidence doc's "Remain
  PROPOSED" clause; params: citing the RATIFICATION sec.)
- `tools/bench_compare_tests.py` (parse / ratio / tuple-exactness / mode-
  isolation tests)

You MUST NOT touch: `tools/bench_baseline.json` (operator PR-2 only);
`crates/**` incl. `stage5_memory_bench.rs` (label drift = evidence-review
event, not a T0b edit); `tools/ui_smoke.py` (T3's file); `tests/**`;
`tasks/milestone_ledger.md`; any other `tools/*`. No `git stash`, no branch
change.

## Frozen locks

Kickoff sec. 5 lock table verbatim (goldens/fixtures, m00/m18 UNEDITED,
`progress.rs` byte-identical, stage-5 13-frame replay UNEDITED, boot
contract append-only, M26/M27 appended tests UNEDITED,
`CURRENT_ANALYSIS_VERSION` 2.1.0, `bench_baseline.json` operator-only).
Proving commands are the reviewer's: diff-scope vs touch-set;
`git diff <start>..HEAD -- tools/bench_baseline.json` EMPTY; existing-mode
isolation proven by the mode-isolation unit tests + the UNEDITED existing
`bench_compare_tests` cases staying green.

## RGB protocol

1. Red, in this file order:
   a. Parse tests for the new grammar rows (e.g. a row
      `stage5 peak RSS (debug, N=640) | 4210.0` parses under the new mode's
      phase tuple; a row missing the pipe raises the same ParseDrift path
      with exit 2). Expected failure: unknown `--bench` choice -> usage
      exit 2 (the mode does not exist yet).
   b. Ratio-math tests: a row at chosen-ratio x baseline passes at
      boundary, regresses past it; the growth-ratio row regression is
      first-class (exit 1). Expected failure: attribute/tuple missing.
   c. Tuple-exactness test: `DOCUMENTED_PHASES_STAGE5_MEMORY` equals the
      literal labels grepped from `stage5_memory_bench.rs` (same trick the
      existing tuple tests use). Expected failure: NameError on the new
      tuple.
   d. Protocol tests (RATIFIED contract): duplicate `stage5 peak RSS`
      labels within one run parse as a MULTISET feeding one slot (both
      samples kept; no parse drift, no last-write-wins); an operand
      diverging >8% from its per-slot median REJECTS that invocation's
      sample and the rejection is reported; fewer than 5 accepted
      invocations takes the pinned not-enough-samples path. Expected
      failure: attribute/tuple missing (same NameError class).
    A test that fails to COMPILE is not Red; state the expected assertion
    text in the journal.
 2. Green: minimal mode implementation - own phases tuple, own phase->ratio
    table (hop-chosen ratio constant + protocol params, each with its
    justification comment per the touch-set clause), own baseline
   key namespace so memory rows never collide with ms rows on the same
   machine class; medians-of-N reuses the existing runner. NEVER weaken an
   assertion to pass.
3. Refactor: keep the file under the 1000-line ceiling
   (`bench_compare.py` is 350 today - margin wide), each function < 100
   lines, names honest (`stage5-memory`, `DOCUMENTED_PHASES_STAGE5_MEMORY`,
   no abbreviations), no `tmp`/`res`.

## Required tests (in `bench_compare_tests.py`)

- New-mode parse of every literal label shape the bench prints (MiB values).
- Mode isolation: default mode ignores memory labels entirely; memory mode
  ignores ms labels; missing-row handling per mode.
- Ratio boundary math for: a peak-RSS row, a phase-delta row, and the
  growth-ratio row (I7b as first-class row).
- MULTISET parsing: duplicate peak labels within one run's output (curve
  child + ratio child, twice per scale) both feed one per-label slot; no
  parse drift, no silent last-write-wins; default mode's strict contract
  unchanged.
- Divergence rejection: an operand >8% from the per-slot median REJECTS
  that invocation's sample - both operands, both scales, inflation AND
  deflation directions (the ERRATUM's corrected law: trimming can hit
  either side); rejected samples counted loudly; fewer than 5 accepted
  invocations -> the pinned not-enough-samples path.
- Baseline keying: memory rows read under their own key; absent baseline ->
  exit 0 + the existing `no baseline for machine class` hint (T0b lands
  BEFORE any operator baseline exists for this mode - that exit-0 shape is
  the expected acceptance behavior, pin it).
- Existing tests UNEDITED and green.

## Gates (exact commands, timeout 600000 ms each; all via distilling entry
points - subagent sessions cannot invoke gate commands directly: ledger row
60; proven route `python tools/distill.py -- python tools/ui_smoke.py`)

1. `python tools/distill.py --kind generic -- python -m unittest
   discover -s tools -p "bench_compare_tests.py"` (CI discover form scoped
   to the one file - the module-name literal is NOT expressible through the
   distiller: `distill.py:149` pins the child cwd to the repo root, so
   "tools dir as cwd" cannot be run; accepted M28 T3 D1 precedent) -
   `Ran N ... OK` with N = the file's prior case count + your new cases
   (the 188+N workspace total belongs to the reviewer's full CI-form
   discover, below).
2. Existing-mode regression: `python tools/distill.py --kind generic --
   python tools/bench_compare.py --help` exit 0 (usage shows the new
   `--bench` choice); NO live bench run is an implementer gate (see below).
3. `python tools/hygiene_check.py` - exit 0.
4. `python tools/encoding_guard.py --staged` - exit 0.
5. Commit: `python tools/commit_scoped.py --scope tools/bench_compare.py
   --scope tools/bench_compare_tests.py --scope
   tasks/stage5-collateral-memory-ceiling-t0b-journal.md -m "feat(tools):
   bench_compare --bench stage5-memory gate mode + unit-locked ratio math
   (M28 T0b)"`.

Total implementer invocations: <= 6.

REVIEWER hop obligations (NOT implementer gates): the LIVE gate run
`python tools/distill.py -- python tools/bench_compare.py --bench
stage5-memory` (LOCAL gate, DP-3 - never CI; runs the ignored debug bench;
expect exit 0 with the no-baseline hint until the operator PR-2 lands, and
expect a PASS-or-WARN honest line after it); workspace census
`python tools/testrun.py --distill` reconciling from anchor `1247 passed /
 0 failed / 13 ignored / 91 suites` (T2-review anchor; erratum applied
 2026-10-06 per the T0b verdict finding 2 relay - it supersedes the
 `1236 passed / 0 failed / 13 ignored / 89 suites` this line carried and
 the older `1235 passed / 0 failed / 6 ignored / 88 suites @ bd4fb1a`; a
 Python-only hop must show zero cargo delta - tools CI-form discover
 carries the +N) (the `--baseline` FILE form is
unrunnable - standing errata row 59: `--distill` census substitute); full
tools discover CI form `python tools/distill.py --kind generic -- python -m
unittest discover -s tools -p "*_tests.py"` (the `-p` pattern is REQUIRED:
the default pattern `test*.py` discovers only 30 tests, so the expected
`Ran 188+N ... OK`-class counts are reachable ONLY in the CI form - M28 T3
verdict finding 1); `python tools/distill.py -- python tools/ui_smoke.py`; `encoding_guard
--all`; commit-vs-touch-set per commit. Baseline refresh for the new rows:
OPERATOR ONLY - `bench_compare.py --update-baseline` on VERIFIED-IDLE
(PR-2), carrying the row-61 stale-baseline ESCALATION: a refreshed or
freshly-keyed baseline is what the reviewer expects to see; a hand-edited
JSON is a REJECT.

Reviewer-note (RATIFIED 2026-10-06 at the DP-4 evidence review -
supersedes this line's PROPOSED status at authoring; landing
`39da2e9`/`afcd038`): the T0a implementer's rework protocol addendum for
this gate is now gate LAW, with the round-2 reviewer's both-scales
condition carried in as binding (verdict finding 1 witness: a -14% FULL
operand made an in-invocation-inflated 3.4 look acceptable). Source
lineage: the evidence doc sec. E1 (decomposable ratio operands emitted per
invocation; fully-read protocol = each capture `target/distill-capture.txt`
read immediately after its run; the +12% short-boot half-scale bimodality
mechanism), proposal recorded at
`tasks/stage5-collateral-memory-ceiling-t0a-journal.md` :244-249. The
live `--bench stage5-memory` run therefore applies: release half-scale
(N=640) peak rows as median-of->=5 isolated invocations, and any
invocation with a >8% per-slot-median operand divergence REJECTED -
binding on both operands at both scales. Fewer than 5 accepted
invocations is not a verdict: report insufficient samples (pinned path),
never gate on <5. Ratio-row parsing accommodates it: operands emitted,
peak labels parsed as a multiset per E1.

## Hygiene / ceilings at risk

`bench_compare.py` 350 -> ~500 lines (ceiling 1000 - wide margin; if your
diff pushes it near 800, extract helpers rather than grow functions).
Functions < 100. The hop-chosen ratio constant and the protocol params
(>=5-sample window, 8% divergence) must each carry their justification
comment (ratio: the hop's written note + the evidence doc's "Remain
PROPOSED" clause; params: the RATIFICATION sec.) - no bare magic numbers.
Pure ASCII.

## Discipline clauses

- UTF-8: Write/Edit tools only; pure ASCII; no PowerShell pipes.
- Touch only the touch-set; report out-of-scope breakage, never fix it.
- No bare high-volume commands; digest header + exit + capture path cited,
  zero raw pastes.
- Journal 1-3 lines per Green step. Context wall: STOP + disk-state
  inventory; successor: `python tools/checkpoint_guard.py resume --spec
  tasks/stage5-collateral-memory-ceiling-t0b-handoff.md --scope tools --cmd …`.

## Acceptance (machine-checkable verdict lines)

- Gate 1: `Ran N tests ... OK` (N = `bench_compare_tests` file-local count:
  prior cases + yours; the 188+N workspace total belongs to the reviewer's
  full CI-form discover `-p "*_tests.py"` - M28 T3 verdict finding 1; exact
  deltas per the suite-count discipline - quote from GREEN-tree
  or `--no-fail-fast` runs only, row-65 lesson).
- Gate 2 exit 0 showing `--bench {stage5-timing,stage5-memory}` (exact
  choice spellings per implementation, shown in journal).
- Gates 3-4 exit 0; `git show --stat <landing>` = touch-set exactly;
  `bench_baseline.json` absent from every diff.
- Reviewer live run exits 0 (no-baseline-hint form) pre-PR-2.
Reviewer gate: `review-gate` skill; its commit-vs-touch-set check enforces
this file; the reviewer additionally adjudicates the label-exactness vs the
evidence doc's "Verbatim label set (T0b row contract)" sec. (26 labels +
the peak-label multiset consequence) and this hop's ratio-justification
note (the 1.15x tolerance is NOT a ratified value - it stayed PROPOSED),
and owns the PR-2 expectation statement.
