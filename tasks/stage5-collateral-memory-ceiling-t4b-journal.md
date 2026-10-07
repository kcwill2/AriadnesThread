# M28 T4b checkpoint journal - bench_compare growth-gate demotion to tripwire

Hop id: `t4b-g-tripwire`. Implementer session (same session that authored
T0a + the evidence-doc ratification landing, per the single-session rule).
Handoff: `tasks/stage5-collateral-memory-ceiling-t4b-handoff.md`.
Authority: user decision 2026-10-06 (evidence review, decision 3): the G
growth gate is DEMOTED to a tripwire; acceptance = the absolute 5200 MiB
peak bars + the Corpora field boot. Grounding:
`tasks/stage5-collateral-memory-ceiling-t0c-verdict.md` sec. 7 (fixture is
Theta(N^2) in aggregate retained closure; ratio 4.0-4.1 is the payload
shape, not a regression).

## Inputs read

- Handoff (complete contract).
- `tasks/stage5-collateral-memory-ceiling-t0c-verdict.md` sec. 7 (witness
  numbers: memo est aggregate refs 742896 -> 3122416 = 4.20 per 2x; save
  tail 1065.2 -> 4371.5 = 4.10x; ratio rows 4.0-4.1 byte-stable).
- `tools/bench_compare.py` sections 3 of the handoff (docstring :30-37,
  constant :185-186, `stage5_memory_bar_verdict` :397-418, verdict_tail
  :761-763, USAGE_TEXT :229); `_handle_compare` :660-694 and
  `compare_medians` :496+ confirm: rows absent from the baseline print
  `WARN untracked phase` and NEVER regress; absent class key -> exit 0
  + no-baseline hint.
- `tools/bench_compare_tests.py` :344-351 (canonical docstring), :591-605
  (peak-bar quadrant - MUST STAY UNEDITED), :606-619 (G quadrant - Red
  carrier).
- `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` sec. RATIFICATION
  + ROUND-2 ERRATUM (G-RATIFIED line) - the line this hop's appended note
  supersedes.
- T0b verdict semantics preserved here: bars compare-path-only, pre-PR-2
  exit-0 no-baseline hint, 1.50x choice.

## Plan (RGB order)

1. GREEN-before-quote: scoped discover green run (row-65 file-local count
   law) - prior `Ran N` quoted below before any inversion.
2. RED: invert the G quadrant (exceeding arm -> exit NOT 1 + TRIPWIRE +
   label; exact-2.0 arm -> pass + no tripwire line) and add the one
   tripwire-baseline-independence case; run scoped discover against the
   UNCHANGED bench_compare.py; quote the failure text verbatim.
3. GREEN: `stage5_memory_bar_verdict` growth branch keeps computing +
   printing a `TRIPWIRE ...` line and never sets `regressed`; constant
   comment cites the 2026-10-06 demotion; docstring exit-1 wording names
   the peak bar only; verdict_tail names the RATIFIED peak bar + growth
   tripwire (growth number kept visible); USAGE_TEXT exceedance wording
   names the peak bar; tests' canonical docstring reworded minimally,
   numbers kept.
4. Evidence-doc APPEND-ONLY supersession note (0 deletions, proven by the
   landing numstat).
5. Gates 1-6 per handoff; gates 3-6 distill-routed (session permission law:
   raw `python tools/hygiene_check.py` is allowlist-denied here - wrapped
   via `distill.py --kind generic`, same substitution as the T0a hop and
   the T3 verdict note).

## Gate digests (verbatim)

- Row-65 pre-inversion GREEN (discipline, not a counted gate):
  `Ran 45 tests in 0.339s` / `OK`
  (capture: `target/distill-capture.txt`).
- Gate 1 RED: `python -m unittest discover -s tools -p
  "bench_compare_tests.py"` -> FAIL, digest named exactly the two inverted
  tests (`test_growth_tripwire_boundary_passes_and_exceeding_trips_without_failing`,
  `test_growth_tripwire_fires_independently_of_baseline_presence`);
  capture 128 lines: `target/distill-capture.txt`.
- Gate 2 GREEN (after implementing in bench_compare.py): `Ran 46 tests in
  0.297s` / `OK` — exactly 45 + 1 added case. Same capture path.
- Gate 3 `python tools/bench_compare.py --help`: exit 0; exit-1 line now
  reads "1 regression past the mode ratio, or (stage5-memory) a RATIFIED
  peak-bar exceedance; the growth tripwire prints but never fails the
  exit code;". Same capture path.
- Gate 4 `python tools/hygiene_check.py` (distill-wrapped, session
  permission law): PASS exit 0; 160-line census is the pre-existing
  R3-warn population; ZERO lines mention bench_compare.py or any touched
  file. Same capture path.
- Gate 5 `python tools/encoding_guard.py --staged`: (filled after run).
- Gate 6 `python tools/commit_scoped.py ...`: (filled after run).

## RED evidence (quoted before touching bench_compare.py)

Both inverted tests against the UNCHANGED bench_compare.py failed for the
predicted semantic reason - the current code exit-1s on ratio > 2.0 and
prints the old `WARN ... RATIFIED growth gate` line, so the demotion
assertions fail at the first assertion of the exceeding arm:

- `AssertionError: 1 != 0 : stage5-memory sample accounting: accepted 5 of
  5 invocations, rejected 0 (> 8% peak-operand divergence from per-slot
  median)` — in
  `test_growth_tripwire_boundary_passes_and_exceeding_trips_without_failing`
  (line 626) and in
  `test_growth_tripwire_fires_independently_of_baseline_presence`
  (line 642). Full outputs (table + old WARN line): capture
  `target/distill-capture.txt` (128 lines).

## Landing evidence

- Evidence-doc append-only proof: `git diff --numstat` on the staged tree
  shows `26 0 tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` —
  ZERO deleted lines. (An intermediate capture showed `27 1`: the first
  note write had introduced a stray leading space on the preserved
  "(debug invocation 4)." line; corrected in-place before staging, final
  diff is a pure append. First hunk starts at file line 347+.)
- Peak-bar quadrant guard proof: `git diff -U0 --
  tools/bench_compare_tests.py` hunk headers are only
  `@@ -348,2 +348,3 @@` (canonical-values docstring),
  `@@ -606,2 +607,9 @@`, `@@ -612,0 +621 @@`, `@@ -617,3 +626,18 @@`
  (the inverted G test + new independence case) — NO hunk covers
  :591-605, so the peak-bar quadrant test is byte-unchanged; capture
  `target/distill-capture.txt`.
- Byte-unchanged semantics: `git diff --cached -U0 --
  tools/bench_compare.py` hunks cover only the docstring (:26, :33), the
  demoted-constant comment+rename (:185), USAGE_TEXT (:229), the
  `stage5_memory_bar_verdict` docstring/growth branch (:398, :412, :415),
  the gate-flow comment (:724), and verdict_tail (:761). NO hunk touches
  the `STAGE5_MEMORY_PEAK_BAR_MIB = 5200.0` definition, the
  `STAGE5_MEMORY_REGRESSION_RATIO = 1.50` definition, or the peak-bar
  WARN branch (only the PASS-line tail wording changed, by design). The
  tripwire keeps the value 2.0 under the honest name
  `STAGE5_MEMORY_GROWTH_TRIPWIRE_RATIO` (rename sanctioned by the
  handoff; old name `STAGE5_MEMORY_GROWTH_GATE_RATIO` now appears only
  in the historical demotion comment).
- Gate 5 `encoding_guard.py --staged`: PASS (exit 0, silent capture).
- Gate 6 `commit_scoped.py`: (filled after run)

## Commits

- (filled after gate 6)
