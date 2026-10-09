# T2 Handoff - M28 stage5-collateral-memory-ceiling: stage-5 lifecycle (clone elimination, pre-stage-5 release, bounded passes)

Hop id: `t2-stage-lifecycle`. The pipeline-side fix hop: stage 5 runs with
ONE live copy of graph/closures/reduced, WITHOUT the index result resident,
and with its two passes chunked so the A2 vector and A6 transients stay
bounded - while the progress event stream stays byte-equal (I6) and
`ensure_load` stays under the 100-line function ceiling.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. Shape `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`
   sec. 0 A2/A4/A5/A6 + :43-44 deep-clone sites, sec. 2 I6, sec. 3.3 (files
   and outcomes, not code), sec. 4 (module map: `stage_load.rs` 210 lines,
   `lib.rs` 634, `ensure_load` 92/100).
3. `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` - THE SUPPLIER
   for your budget: sec. RATIFICATION + ROUND-2 ERRATUM (2026-10-06,
   landing `39da2e9`/`afcd038`) concluded the DP-4 review - milestone
   constants ratified there (N0/bars/G/protocol/4096-rider); this hop's
   scope stands as shape sec. 3.3 (A4/A5 in scope by shape; A2/A6
   chunking depth per the measured per-phase deltas in blocks 2-4), and
   the review ratified NO per-cache caps (relay 2026-10-06): IF your
   budget wires boot-memo caps, the cap literals come from T1's landed
   choices (cite T1's landing commit + journal justification), never from
   invented numbers.
4. `crates/pipeline/src/stage_load.rs` (full 210 lines; the take/restore
   callback precedent :81-94; `AtomicUsize` completion precedent :82,:199;
   `collect_file_metrics` signature used by internal_tests).
5. `crates/pipeline/src/lib.rs` :191 (`index_cache` residency), :469-475
   (`take_include_graph` move-out precedent), :555-573 (the two deep-clone
   accessors you will thread over).
6. `crates/cli/src/commands/ui.rs` :250 (boot's `take_include_graph` after
   stage 5 - must stay green UNEDITED; the file itself is MUST NOT TOUCH).
7. `crates/pipeline/src/internal_tests.rs` :122-185, :216-228, :333-337
   (report equality), :418-439 (warm reuse), :472-515
   (`collect_file_metrics` pin) - the equivalence oracle: green UNEDITED is
   the preferred outcome; sec. 3.3 of the shape is the ONLY thing that can
   budget an edit, per-test, with written justification.
8. `crates/cli/src/commands/ui/progress_contract_tests.rs` (the 13-frame
   replay - read to know what you must NOT disturb).
9. `docs/specs/tool-output-distillation.md`.

## Preconditions (hard - encoded here, enforced at dispatch)

1. SATISFIED 2026-10-06. T0 landed APPROVED (T0a round-1 REJECTED at
   `8d34a3c` -> round-2 APPROVED at `69885ce`, verdict
   `tasks/stage5-collateral-memory-ceiling-t0a-verdict.md`) and the WRITTEN
   DP-4 EVIDENCE REVIEW CONCLUDED 2026-10-06 (evidence doc sec. RATIFICATION
   + ROUND-2 ERRATUM, landing `39da2e9`/`afcd038`). Per the review's dispatch
   relay T2 is FIRED and DISPATCHABLE NOW (no fix commit predates the table -
   the table exists, is ratified, and its measured phase deltas are your
   A2/A6/A7 attribution). Boot-memo cap WIRING was NOT ratified with cap
   literals (relay): wire it ONLY per precondition 2.
2. Boot-memo cap wiring (the pipeline is where the boot memo instance is
   built, `stage_load.rs:62` `LoadAnalyzer::with_closures`): SERIALIZED
   BEHIND T1'S LANDING - T1 must be landed so the additive capped
   constructor exists to call, and the cap literals are T1's landed
   choices (T1 landing commit + its journal justification, cited in your
   code comment). Until T1 has landed, ship stages (a)-(e) WITHOUT cap
   wiring; do not block this hop on T1 for the non-cap work, and do not
   invent cap numbers.
3. Parallel with T1 is allowed for the NON-cap work only (files disjoint:
   `crates/pipeline` vs `crates/graph`; behavior overlap was the shape's
   named risk, resolved by the table); the cap-wiring clause of
   precondition 2 stays serialized behind T1's landing.
4. Zero fixture demand: all tests run on in-repo fixtures / temp dirs.
   Nothing new inside `tests/fixtures/basic` / `tests/fixtures/expensive`;
   fixture demand routes through the documented approval path (M27 T0b
   dated-row precedent), never inside a hop.

## Objective and invariants

Objective (shape sec. 3.3, outcomes not code):
(a) Kill the A4 double-residency: replace the `cached_include_graph()` /
`cached_reduced()` deep clones at `stage_load.rs:43-44` with a take/restore
over `self` (precedents in-repo: the `self.notify` take/restore at
`stage_load.rs:81-94`; the `take_include_graph` move-out at
`lib.rs:469-475`) so stage 5 runs with ONE live copy of each.
(b) Release `index_cache` before stage 5 executes (A5: stage 5 never reads
it, `stage_load.rs:41-44` needs include+reduced only - verified by the
shape's reads), WITHOUT breaking warm reuse: `internal_tests.rs:418-439`
green UNEDITED and fingerprint-fresh `ensure_index` reload behavior green.
Prove (b) with a new assertion-style test (fixture-scale RSS delta clause
OR a size-counter check - implementer's pick, journal the choice).
(c) Bound the A2/A6 transients: chunk or stream the symbol and file passes
so peak concurrent owned transients scale with chunk size, not corpus size,
keeping the EXACT event stream (I6): same kinds, same order, same payloads;
per-file completion stays exactly one `CollateralFile` event per file
(`AtomicUsize` precedent `stage_load.rs:82,199`); `SymbolsAnalyzed` and
`ArtifactsSaved` appear exactly where they do today.
(d) Keep `collect_file_metrics`' signature if at all possible (preferred:
`internal_tests.rs:472-515` green UNEDITED). An unavoidable signature
change is BUDGETED per shape 3.3: per-test justification in the journal +
verdict, never silent.
(e) `ensure_load` (92 lines today) and every extracted helper stay < 100
lines - extraction is MANDATORY for any growth, planned hygiene not a
discovered failure.

Invariants: I1 (13-frame replay green UNEDITED - your diff must not make
that file appear anywhere), I2 (`progress.rs` diff-empty), I3 (boot
contract append-only; M26/M27 appended tests UNEDITED), I4 (fixtures/
goldens/m00/m18 UNEDITED; `CURRENT_ANALYSIS_VERSION` untouched), I6 as
above, I7a bars are NOT yours to satisfy locally - they are T0b's gate +
operator runs; your obligation is the structural outcome the evidence table
attributes to A2/A4/A5/A6.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/stage5-collateral-memory-ceiling-t2-journal.md` (checkpoint
  journal; always in-set; create alongside)
- `crates/pipeline/src/stage_load.rs` (take/restore, chunked passes,
  extracted helpers)
- `crates/pipeline/src/lib.rs` (take/restore seams, `index_cache` release -
  margin 366 lines)
- `crates/pipeline/src/internal_tests.rs` ONLY if you invoke the 3.3 budget
  - each edited test needs its own written justification; if you cannot
  state one, the edit is out of touch-set
- NEW pipeline test files for the I6 event-stream recording test + the (b)
  release assertion, under `crates/pipeline/tests/` (or `src/
  internal_tests.rs` adjacency if in-crate visibility requires it - same
  budget rules)

You MUST NOT touch: `crates/pipeline/src/stage_index.rs` (hot path, D3
lock); `crates/cli/**` incl. `ui.rs` (853 lines, protected) and every
progress-contract file; `crates/graph/**`; `crates/report/**`; `tests/**`
incl. fixtures; `tools/**`; `tasks/milestone_ledger.md`. No `git stash`, no
branch change. No progress-event changes: the event enum is read-only
material here.

## Frozen locks

Kickoff sec. 5 table verbatim (m00 `2 passed` / m18 `23 passed / 0 failed /
0 ignored / 1 suites` UNEDITED; `progress.rs` byte-identical; 13-frame
replay UNEDITED green; boot contract append-only + M26/M27 appended tests
UNEDITED; goldens/fixtures frozen; `CURRENT_ANALYSIS_VERSION` 2.1.0;
`bench_baseline.json` operator-only; census anchor `1235 passed / 0 failed
/ 6 ignored / 88 suites @ bd4fb1a`). Hop-specific: the warm-reuse contract
(`internal_tests.rs:418-439`) green UNEDITED; boot's `take_include_graph`
after stage 5 green UNEDITED (`ui.rs:250` - you cannot touch it, so it
proves your take/restore left the post-stage-5 state contract intact);
`bench_stages`/`payload_tail_bench` rows stay parseable (timing gate G-D
runs post-PR-2 at the reviewer hop).

## RGB protocol

1. Red, in this order (all compile-green first - a test that fails to
   compile is not Red; state each expected assertion message in the journal):
   a. I6 recording test: capture the full `ensure_load` event stream on a
   locked fixture into a vector (kinds+order+payloads) and assert against a
   pre-recorded literal captured from the CURRENT tree (journal records the
   capture run). It lands GREEN as a characterization lock FIRST - this
   one is a floor, not a red: record it as such in the journal.
   b. (b)-release red: an assertion-style test that after a cold
   `ensure_load` on a fixture the pipeline's stage-5 span does not hold the
   index result (RSS-delta or size-counter form) - expected failure:
   "index result still resident through stage 5" (or your chosen counter's
   analogous line) because A5 residency exists today.
   c. (a)-single-copy red: a structural assertion that during the file
   pass only one copy of the closures table is alive - honest machine form
   (e.g. a drop-count / borrow-window witness test around the take/restore
   seam) - expected failure names the clone path at :43-44.
   d. (c)-chunk red only if your chosen bound has a machine-checkable pre-
   state claim at fixture scale; at 520-file fixture scale superlinearity
   is invisible (E6) - if the honest red needs the scaled fixture, that is
   T0b's gate job (G-C discrimination on the pre-fix tree), NOT a unit
   red; journal the reasoning instead of faking a green-able red.
2. Green: implement (a)-(e) minimally. Re-run the I6 recording test after
   EVERY behavior-touching step - it is your leash. NEVER weaken an
   assertion to pass; if the recording test's literal disagrees, the EVENT
   STREAM is wrong, not the literal (unless a shape-3.3-budgeted change
   says otherwise - it does not cover event streams: I6 has zero budget).
3. Refactor: helper extraction with honest names (`run_symbol_pass`,
   `run_file_pass`, `take_graph_and_reduced_for_load`-class naming - final
   names implementer's, no `tmp`/`res`/single-letter); doc comments state
   the take/restore invariant at each seam; `ensure_load` final line count
   printed in the journal.

## Required tests

- I6 event-stream recording test (per locked fixture, exact stream literal).
- Per-file event-count test: exactly one `CollateralFile` per file,
  `done == total` terminal frame (I6 clause, `AtomicUsize` precedent).
- (b) index-release assertion (fixture-scale RSS-delta or size-counter).
- (a) single-copy structural witness.
- Warm-reuse internal_tests green UNEDITED (your gate, not a new test).
- Existing pipeline suite green UNEDITED except budgeted edits.

## Gates (exact commands, timeout 600000 ms each; all via distilling entry
points - subagent sessions cannot invoke gate commands directly: ledger row
60; proven route `python tools/distill.py -- python tools/ui_smoke.py`)

1. `python tools/testrun.py --distill -p ariadne-pipeline` - Red captures
   (1b/1c failure messages) then GREEN; canonical line verbatim in the
   report; pre-dispatch green truth `25 passed / 0 failed / 4 ignored /
   6 suites` (post-T1a tree; if T1 landed first, reconcile graph-driven
   deltas per its verdict - quote from GREEN-tree runs, row-65 erratum).
2. `python tools/testrun.py --distill -p ariadne-pipeline -- --test
   internal` (or the per-test `--` filter forms) - the three oracle groups
   green.
3. `python tools/testrun.py --distill -p ariadne-cli` - boot contract +
   13-frame replay + `ui.rs:250` path green UNEDITED (you never touch cli
   files; this is the witness).
4. `python tools/hygiene_check.py` - exit 0 (function-ceiling is the named
   risk: 92 -> must stay < 100 with extraction).
5. `python tools/encoding_guard.py --staged` - exit 0.
6. Commit: `python tools/commit_scoped.py --scope
   crates/pipeline/src/stage_load.rs --scope crates/pipeline/src/lib.rs
   --scope crates/pipeline/tests --scope
   tasks/stage5-collateral-memory-ceiling-t2-journal.md [-scope
   crates/pipeline/src/internal_tests.rs ONLY if budgeted] -m "perf(pipeline):
   single-copy stage-5 lifecycle, pre-stage-5 index release, bounded passes
   (M28 T2)"`.

Total implementer invocations: <= 10 counting the red/green iterations
inside gates 1-2. If you need an 11th, you have over-scoped: STOP, journal,
return the disk-state inventory (sizing law is per-handoff, not per-will).

REVIEWER hop obligations (NOT implementer gates): workspace census
`python tools/testrun.py --distill` reconciling from the `bd4fb1a` anchor
`1235 passed / 0 failed / 6 ignored / 88 suites` (the `--baseline` FILE
form is unrunnable - standing errata row 59: `--distill` census substitute);
m00/m18 parity proofs; full tools discover CI form
`python tools/distill.py --kind generic -- python -m unittest discover -s
tools -p "*_tests.py"` (the `-p` pattern is REQUIRED: the default pattern
`test*.py` discovers only 30 tests, so the expected `Ran 188+N ... OK`-class
counts are reachable ONLY in the CI form - M28 T3 verdict finding 1);
`python tools/distill.py -- python tools/ui_smoke.py` (boots warm
AND cold - your R5 witness); `encoding_guard --all`; the G-D timing check
`python tools/distill.py -- python tools/bench_compare.py` (PR-2 baseline
refresh must precede it - row-61 ESCALATION; a stale-baseline FAIL is the
known flapping gate, not this hop); the `--bench stage5-memory` LOCAL gate
run + G-C discrimination evidence (operator/reviewer on idle machine -
DP-3, never CI); per-commit `git show` oldest-first vs touch-set;
internal_tests edit-by-edit adjudication of any 3.3-budgeted change.

## Hygiene / ceilings at risk

`ensure_load` 92/100 - THE ceiling at risk: extraction is mandatory
(section 3.3/7). `lib.rs` 634/1000 margin 366; `stage_load.rs` 210 -> budget
<= 400. Functions < 100 including all extracted helpers. Pure ASCII; no
banned names.

## Discipline clauses

- UTF-8: Write/Edit tools only; pure ASCII; no PowerShell pipes.
- Touch only the touch-set; internal_tests edits ONLY under the 3.3 budget
  with per-test justification; out-of-scope breakage: report, don't fix.
- No bare cargo/ui_smoke/bench_compare; distilling entry points only;
  digest + canonical line + capture path cited; zero raw pastes (R9).
- Journal after each Green step. Context wall: STOP + disk-state inventory;
  successor: `python tools/checkpoint_guard.py resume --spec
  tasks/stage5-collateral-memory-ceiling-t2-handoff.md --scope
  crates/pipeline --cmd …`.

## Acceptance (machine-checkable verdict lines)

- Gates 1-3 canonical lines `0 failed`; I6 recording test green UNEDITED
  post-change (byte-equal stream literal).
- Journal prints final `ensure_load` line count < 100.
- Gate 4 exit 0; `git show --stat` = touch-set exactly; progress-family
  files appear in zero diffs; budgeted internal_tests edits (if any) each
  carry their justification paragraph.
- Reviewer: census reconciles from `bd4fb1a`; `ui_smoke: PASS` warm+cold;
  stage-5 timing gate within tolerance post-PR-2 (G-D).
Reviewer gate: `review-gate` skill; commit-vs-touch-set enforces this file;
the reviewer owns the 3.3-budget adjudication and the G-D acceptance.
