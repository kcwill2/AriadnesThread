# Verdict - M28 `stage5-collateral-memory-ceiling` T0b (t0b-gate-mode)

Reviewer: review-gate skill, fresh verification at HEAD `dc294bb` (main,
ahead=0, behind=0; 24 foreign dirty paths, none inside the touch-set;
ledger/plans dirt is orchestrator-owned). Commits under review,
oldest-first: `ac0eb7e` (mode + 24 tests + journal), `dc294bb` (journal
hash record). Base (dispatch HEAD): `69138db`. Suppliers honored:
handoff RATIFIED section, evidence doc sec. RATIFICATION + ROUND-2
ERRATUM / "Remain PROPOSED" / E1-E3, T0a round-2 verdict conditions.

## VERDICT: APPROVED

No blockers. The mode implements the RATIFIED contract as written, the
default mode is behaviorally untouched, and the three hop-owned choices
are defensible on the record. 3 findings: 1 minor process finding, 2
informational relays; none requires a rework.

## Per-commit scope verdict

- `ac0eb7e`: `git show --stat` = journal +190/-0, `tools/bench_compare.py`
  +495/-56, `tools/bench_compare_tests.py` +475/-0. INSIDE touch-set
  exactly. The tests-file numstat deletion count is ZERO - independently
  verified: the 21 pre-existing cases are byte-identical (pure-insertion
  diff).
- `dc294bb`: journal only (+20/-6). INSIDE.
- Range proof: `git diff --name-only 69138db..dc294bb` = the three
  touch-set paths, nothing else. `git diff 69138db..dc294bb --
  tools/bench_baseline.json` = EMPTY (exit 0, no output). I1-I4,
  goldens/fixtures, m00/m18, `progress.rs`, `CURRENT_ANALYSIS_VERSION`,
  every `crates/**` path: zero diff by scope, byte-identical.

## RATIFIED-contract fidelity (central duty) - clause by clause

Each RATIFIED clause traced to code AND a pinning test, all green in a
fresh scoped run:

1. Median-of->=5 ACCEPTED invocations: `reject_divergent_peak_invocations`
   (bench_compare.py:348) rejects first, `stage5_memory_medians`
   (:380) computes verdict medians over accepted runs only; memory-mode
   default `--runs` = 5 (live probe witness: `default runs (memory): 5`;
   `test_memory_mode_defaults_to_five_invocations`).
2. >8% operand-divergence rejection binding on BOTH operands at BOTH
   scales: `_divergent_operand` (:338) tests every sample of every
   `stage5 peak RSS` label at both scales with `abs(sample - median) >
   0.08 * median` - symmetric (inflation AND deflation), wholesale
   invocation rejection. Pinned: `test_operand_divergence_rejects_
   invocation_in_all_four_quadrants` (half/full x up/down) and
   `test_operand_divergence_of_exactly_eight_percent_is_accepted` (the
   strict `>` matches the ratified word "diverge >8%" - exactly-8% is
   accepted). Independent reviewer witness: my scratch scenario C (one
   invocation +25% on the half-scale slot) printed `REJECT invocation 4:
   stage5 peak RSS (debug, N=640) sample 3000.0 MiB diverges +25.0% from
   slot median 2400.0 MiB (limit 8%)` and `accepted 5 of 6`.
3. Loud rejected-sample accounting: `_report_sample_accounting` (:625)
   always prints REJECT lines + `stage5-memory sample accounting:
   accepted K of N` - never silent exclusion; tally prints even with
   zero rejections (scenario A witness: `rejected 0`).
4. <5 accepted is not a verdict: pinned exit-2 path
   `ERROR insufficient samples: ...` (:745), AFTER loud accounting,
   BEFORE any baseline read/write (journal decision 4 verified in code
   order); pinned by `test_fewer_than_five_accepted_invocations_exits_2`
   (with a VALID baseline placed, asserts neither hint nor pass line
   appears) and `test_requested_runs_below_five_cannot_reach_a_verdict`.
5. MULTISET parsing (twice-per-scale peak labels):
   `parse_bench_table_multiset` (:288) shares `_parse_pipe_row` with the
   unchanged-behavior default parser (identical grammar + identical
   drift messages); `test_peak_label_duplicate_within_one_run_keeps_
   both_samples` pins both samples feed one slot, no parse drift, no
   last-write-wins in memory mode; default mode's strict contract stays
   (explicit pin `test_default_parse_keeps_strict_last_write_wins_
   contract` + all 21 old tests green).
6. 26-label tuple validation: 26 templates (20 scale x 2 scales + 1
   ratio + 5 warm = 46 concrete debug labels). My own grep of
   `crates/cli/tests/stage5_memory_bench.rs` finds exactly the 25
   `format!("... {suffix}")` label stems (20 scale + 5 warm) and the
   ratio template at :835 - one-to-one with the tuple.
   `Stage5MemoryTupleExactnessTests` re-greps the real bench source at
   test time (set equality, 25/20/5/46/40/1 counts, no release rows).
7. 4096-rider probe tolerance (no bench-side row this hop): unknown
   probe row class parses, gets a median row, `WARN untracked phase` -
   `test_unknown_probe_row_is_reported_not_fatal` exit 0. Report, not
   crash - as the RATIFIED section requires absent a touch-set amendment.

Adjudication note (faithfulness, not a finding): the rejection baseline
is a provisional per-slot median over the full sample pool (median is
robust to the ~1/7 outlier frequency); the final per-label medians are
recomputed over accepted invocations only. The RATIFIED sentence does
not define an iterate-to-fixed-point; the single-pass reading + the
recompute-over-accepted are the honest literal implementation, and
every boundary is pinned in tests. ACCEPTED as plan-faithful.

## Hop-owned decision adjudications

1. Regression ratio 1.50: ACCEPTED. The handoff assigns this hop the
   choice with a written note "wider than 1.25x", citing the evidence
   doc's "Remain PROPOSED" clause - the note exists verbatim at the
   constant (:198-213) and in the journal, and satisfies the plan
   literal. On the merits: the RSS envelope (+12% half-scale
   bimodality, -14% full-side trimming, ~1/7 outlier frequency) is not
   the ms mode's 0.06%/0.13% spread; no second-machine baseline exists
   pre-PR-2; and the 1.25x ms gate flapped +25..+39% on the SAME tree
   on this machine (ledger row 62) - a tighter ratio would bake the
   row-61 flapping failure into RSS mode from day one. Tightness is
   preserved because the two RATIFIED absolute bars fire in the same
   compare path INDEPENDENTLY of the baseline ratio (reviewer scenario
   B: median 5300 exits 1 naming the bar, though 1.50x relative is
   nowhere near), and the pre-fix class (~9040 MiB, ratio 4.0) misses
   the bars by miles. 1.50 is exactly representable in binary FP, so
   the boundary tests compare exactly. CONDITION (recorded, not a
   blocker): re-adjudicate this ratio at the post-PR-2 timing re-judge
   (row 69) and tighten once the baseline epoch + a second machine
   settle the cross-machine uncertainty; 1.15x stays PROPOSED and was
   NOT silently ratified here (the constant's comment says
   `CHOSEN BY THIS HOP - NOT a ratified constant`).
2. <5 accepted -> exit 2 `ERROR insufficient samples`: ACCEPTED. The
   ratified clause allowed exit-2 OR an explicit WARN; adjudicated on
   merits (not rubber-stamped): an exit-0 WARN reads as clean to every
   exit-code consumer (the distiller itself digests `ok (exit 0)`),
   which would soften "never gate on <5" into a soft pass; exit 2
   matches the tool's standing "never substitute zeros" exit-2 class
   and is documented in the docstring/usage. Pinned in two tests; loud
   accounting still prints before the exit.
3. RATIFIED bars enforced in the compare path only, pre-PR-2 exit-0
   hint preserved: ACCEPTED. Verified the hint can never be mistaken
   for a PASS: the no-baseline path (:671) prints `WARNING no baseline
   for machine class '...'; regression compare skipped.` - no verdict
   line exists on that path, and
   `test_absent_memory_baseline_exits_0_with_hint_and_namespaced_snapshot`
   pins exit 0 + hint + the `::stage5-memory`-namespaced snapshot. And
   a landed-baseline run WILL enforce the bars: `stage5_memory_bar_
   verdict` (:397) is wired as `extra_verdict` into `_handle_compare`
   and fires independently of the relative comparison (baseline value
   cannot buy bar headroom) - pinned by
   `test_rated_peak_bar_boundary_passes_and_exceeding_regresses` and
   `test_rated_growth_gate_boundary_passes_and_exceeding_regresses`,
   and witnessed independently by my scratch scenario B (exit 1,
   `WARN stage5 peak RSS (debug, N=1280): 5300.0 MiB exceeds the
   RATIFIED 5200 MiB peak bar (N=1280)`). The PASS line itself names
   the bars: `no regressions past 1.50x baseline and the RATIFIED bars
   (peak 5200 MiB, growth 2.0)`.

## Default-mode non-regression proof

- 21 pre-existing cases UNEDITED (range numstat +475/-0 on the tests
  file) and green inside my fresh `Ran 45` run.
- `--bench stage5-timing` reaches the unchanged verdict line
  `bench compare: 3 run(s), no regressions past 1.25x baseline`
  (`test_explicit_timing_choice_behaves_as_the_default_mode`, fresh run).
- Default mode fed memory-format output exits 2 naming ITS OWN missing
  ms rows and prints no table (`test_default_mode_never_gates_on_
  memory_rows`).
- Default-mode duplicate-label semantics stay last-write-wins
  (explicit pin, passing before AND after - the only test green at Red
  by design).
- Line-by-line audit of all 56 deleted `bench_compare.py` lines: every
  deletion is a parameterization with byte-identical defaults
  (`_parse_pipe_row` is the moved original parse body, drift messages
  verbatim; `compare_medians` ms formatting and WARN/verdict f-strings
  unchanged for the timing defaults; `run_bench` defaults to the old
  `BENCH_CARGO_ARGS`/parser; timing default runs still 3 - live probe
  witness `default runs (timing): 3`; `DOCUMENTED_PHASES` untouched in
  the diff). The ONE intentional behavior addition - `--help` exits 0 -
  is plan-mandated (handoff Gate 2 literally requires "exit 0 showing
  --bench {stage5-timing,stage5-memory}") and shown live.
- Live default-mode bench run (reviewer obligation): ran and produced
  the pre-T0b WARN byte-shape against the stale baseline (see findings
  relay below) - exit 1 as the row-61/62 stale-baseline class predicts.

## M27 T1b serialization

`DOCUMENTED_PHASES` semantics untouched (diff-EMPTY region);
`--bench boot-stages` remains a usage error - live witness: `ERROR
unknown --bench choice 'boot-stages'; expected one of: stage5-timing,
stage5-memory` (exit 2). The memory rows live in a namespaced baseline
key, witnessed live: `Windows-AMD64-AMD64_Family_26_Model_68_Stepping_
0_AuthenticAMD::stage5-memory`. M27 T1b is not preempted.

## Findings

1. [minor, process - ACCEPTED with note] Undeclared gate-invocation
   overshoot: handoff gates section caps implementer invocations at 6;
   the journal records ~9-10 (gate 1 ran RED, GREEN, fixture-fix
   re-run, refactor re-run, committed-tree final; gate 2; gate 3 in
   two forms - the extra form IS declared as D2; gate 4; plus the
   CI-form workspace discover, which is a reviewer-form obligation the
   implementer ran early). Harmless (no context wall, no scope drift),
   and precedent exists (M28 T2 D9 declared a 14>10 overshoot and was
   accepted). Finding = the UNDECLARED part (review-gate step 5:
   undeclared deviation). No code action; next-hop journals declare
   overshoot counts as dated D-rows.
2. [informational, planner relay] The handoff's reviewer-obligation
   census anchor `1236 passed / 0 failed / 13 ignored / 89 suites` was
   stale at dispatch - superseded by the T2 review's
   `1247 passed / 0 failed / 13 ignored / 91 suites`; my fresh census
   matches the NEW anchor exactly and the +24 T0b tests land in the
   Python discover (194 -> 218), zero cargo delta, as expected. Fix
   the anchor line in the handoff at next touch (planner-owned file;
   not an implementer action).
3. [informational, operator purge] `tasks/t0b-review-probe.py` - this
   review's scratch probe (row-67 class; `Remove-Item` denied to
   reviewers). Add to the reviewer-throwaway purge queue.

## Declared deviations - dispositions

- D1 (module-level comment lines closing the hygiene span-finder's
  multi-line-def swallow): ACCEPTED. Behavior-neutral, inside the
  touch-set, hygiene exit 0 with zero rows on both touched files
  (verified in the 154-line capture).
- D2 (gate 3 additionally through distill for the exit witness):
  ACCEPTED; matches the session's permission reality.
- D3 (`bench_baseline.json` untouched, verified by diff): ACCEPTED and
  independently re-proved by me (range diff EMPTY + live JSON read
  shows only the ms machine-class key, zero `::stage5-memory` keys).

## What the LIVE gate does the moment operator PR-2 lands

Operator runs `python tools/bench_compare.py --bench stage5-memory
--update-baseline` on VERIFIED-IDLE (row 70 disk-free prerequisite
included - 5 debug suite invocations, N=1280 temp dirs). That writes
the machine-class-namespaced `...::stage5-memory` entry (runs =
accepted count). From then on every live `--bench stage5-memory` run:
5 isolated debug suite invocations -> multiset parse -> wholesale >8%
divergence rejection with loud `REJECT ...` accounting -> fewer than 5
accepted = exit 2 `insufficient samples` (no verdict, loud accounting
still printed) -> per-label medians over accepted -> exit 1 if ANY row
median exceeds 1.50x the landed baseline, OR the debug N=1280 peak
median exceeds 5200 MiB, OR the debug growth-ratio median exceeds 2.0
(bars fire independently of baseline values); else exit 0 with the
pass line naming 1.50x + both bars. Pre-PR-2 the expected live shape
remains the pinned exit-0 no-baseline hint - and it is provably today's
shape: the baseline JSON contains no `::stage5-memory` key.

## Relays - open, not discharged by this landing (checked)

- Row 62 (stale/flapping ms baseline): live default-mode run today
  exit 1 - `WARN symbol_pass+prep (combined, <= first event): 1646.0 ms
  vs baseline 1276.4 ms (+29%)` and `WARN file_pass (event span):
  666.4 ms vs baseline 312.5 ms (+113%)`; PR-2 still outstanding.
- Row 69 (cap economics, post-PR-2 timing re-judge): today's
  same-machine witness is CONSISTENT with the T2 trigger - `file_pass`
  median 666.4 ms (T2 review: 643.5; same-day T1: 349.1 -> today
  ~1.91x vs the row's ~1.84x), `symbol_pass+prep` 1646.0 (T2: 1653.7;
  +29% vs +30%). Consistent -> recorded as the witness, no new finding;
  the deferred re-judge row stays OPEN.
- Row 70 (operator disk): C: free measured 1.4 GiB at review time -
  still unmet; the live `--bench stage5-memory` run stays blocked on
  it (see Explicitly-NOT-run). Row stays OPEN.
- PR-2 itself: OPEN (no `::stage5-memory` key present, live witness).

## Report-evidence check (tool-output-distillation)

Journal is digest-shaped throughout: `### generic PASS/FAIL (exit N)`
verdict lines, verbatim `Ran N` summaries, capture paths, zero raw
pastes; gate commands used the distilling entry points. PASS.

## Gate lines (verbatim, fresh at dc294bb)

- Scoped: `### generic PASS (exit 0)` / `Ran 45 tests in 0.327s` / `OK`
- CI-form tools discover: `### generic PASS (exit 0)` /
  `Ran 218 tests in 20.601s` / `OK` (194 prior + 24 new, exact)
- Workspace census (`--distill` unscoped, row-59 substitute):
  `### cargo test PASS (exit 0)` [capture: `target/testrun-capture.txt`,
  2213 lines] / `1247 passed / 0 failed / 13 ignored / 91 suites` -
  byte-identical to the ledger M28 anchor @ T2; Python-only hop, zero
  cargo delta, as expected
- Hygiene: `### generic PASS (exit 0)` (154 capture lines, all
  pre-existing R3 warns in foreign paths; zero rows for
  `tools/bench_compare.py` / `tools/bench_compare_tests.py`)
- Encoding: `### generic PASS (exit 0)`, capture 0 lines
- Smoke: `### generic PASS (exit 0)` / `boot+route checks clean (cold
  fixture, attempt 1, 1.0s)` / `ui_smoke: PASS`
- Gate 2 live: `### generic PASS (exit 0)` with usage line
  `[--bench {stage5-timing,stage5-memory}]`
- Boot-stages live: `### generic FAIL (exit 2)` / `ERROR unknown
  --bench choice 'boot-stages'; expected one of: stage5-timing,
  stage5-memory`
- Live default-mode bench: `### generic FAIL (exit 1)` (stale-baseline
  class - relays above)
- Synthetic memory-mode witnesses (my scratch, temp-dir baselines;
  `bench_baseline.json` NEVER touched): scenario A exit 0
  (`accepted 5 of 5, rejected 0` + bars named in the pass line);
  scenario B exit 1 (RATIFIED peak bar at 5300); scenario C wholesale
  rejection + `accepted 5 of 6`.

## Explicitly NOT run

- LIVE `--bench stage5-memory` full form: NOT RUN - environment-blocked
  by the OPEN ledger row 70, not by permissions. Disk free witness
  (fresh): `DISK C: free 1.4 GiB of 1862.0 GiB total`; ledger row 70
  names `the LIVE --bench stage5-memory gate` under `FREE DISK BEFORE`.
  The hint form needs >=5 accepted debug N=1280-suite invocations; the
  N=1280 artifact save fails at this capacity (`os error 112` twice at
  the T1 review); attempting it would burn ~30 minutes, fail mid-save,
  and leak further temp dirs into 1.4 GiB of headroom. The pinned
  no-baseline exit-0 hint is instead proven by its pinning test (green
  fresh) + the live fact that `bench_baseline.json` carries no
  `::stage5-memory` key, so the live form IS the hint today.
- `--baseline` FILE form: unrunnable (standing errata row 59); the
  sanctioned `--distill` census substitute ran and is quoted.
- Golden guard script: not landed; byte-identity proven by diff scope.
- `theme_token_check.py`: NOT APPLICABLE - zero UI paths in the range.

## Permission / gate-route record

Every gate was attempted and ran through allowlisted direct or distill
routes. Refused forms (all the catch-all boilerplate `{"permission":
"bash","pattern":"*","action":"deny","source":"global"}`): compound
git+echo chains, `python -c` probes (two forms, incl. one
distill-wrapped), `Get-PSDrive C`, and a direct (non-distill)
`python tools/bench_compare.py --help` (substituted by the distill
form, which passed); one Write into the external temp dir was refused
by the agent edit grant `{"permission":"edit","pattern":"*","action":
"deny","source":"agent"}` (scratch relocated to `tasks/`, row-67
precedent). No gate was skipped, substituted with weaker evidence, or
judged unrunnable on permissions; the only non-completed obligation was
environment-blocked and is declared above.
