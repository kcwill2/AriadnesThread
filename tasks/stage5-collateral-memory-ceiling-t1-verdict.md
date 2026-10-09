# Verdict - M28 T1 `t1-bounded-memo` (bounded-capacity LoadMemo + capacity-invariance oracle)

Commit under review: `d6640a2` (start rev = parent `afcd038`). Reviewer
acceptance pass run fresh at HEAD `d6640a2` on 2026-10-06.
Execution-results addendum: `tasks/stage5-collateral-memory-ceiling-execution-results.md`
("T1 reviewer acceptance pass" section).

## VERDICT: APPROVED

Zero blockers. Three findings, all minor/informational. Deviations D1-D5
all ACCEPTED on independent merit; the guard-shape relaxation declared in
the journal's guard audit (not D-numbered) is likewise ACCEPTED. The
hop-fixed caps and the I7c pin are adjudicated ACCEPTED on Block-4
evidence (this adjudication is the reviewer's, per DP-4 honesty; they
remain hop-fixed, not review-ratified, and T2 must cite them as such).

## 1. Commit-vs-touch-set - PASS, exact

`git diff --name-only d6640a2^..d6640a2` =
`crates/graph/src/load_caches.rs` (+138/-48),
`crates/graph/tests/load_memo_capacity_invariance_test.rs` (+639/-0, NEW),
`tasks/stage5-collateral-memory-ceiling-t1-journal.md` (+280/-0).
Exactly the handoff touch-set with the `load.rs` scope DROPPED as
gate 5 pre-authorizes ("drop the load.rs scope if the seam was not
needed"). Sufficiency of the drop verified independently:
`LoadAnalyzer::with_memo` is public at `load.rs:201` and takes
`Arc<LoadMemo>`; the new suite itself wires
`with_closures_and_capacity` through that seam (test file :423-428), so
the seam is demonstrably sufficient for T2's boot-cap consumption and a
third constructor would be the speculative API the handoff forbids.
Frozen-lock scan `afcd038..d6640a2`: diff-EMPTY for `progress.rs`,
`crates/index/src/cache.rs` (`CURRENT_ANALYSIS_VERSION` untouched),
`bench_baseline.json`, `tests/fixtures`, `tests/golden`,
`progress_contract_tests.rs`, `boot_progress_tests.rs`, m00/m18 parity
files, `internal_tests.rs`. Wider-range note: root `Cargo.toml` appears
in `bd4fb1a..HEAD` solely via `5b517e6` (M27 T5 profile-lever rider,
landed and adjudicated upstream) - zero Cargo delta in T1's own commit.

## 2. RGB truth - the oracle froze BEFORE the swap, and it pins OUTPUT equality

RED (journal, instrumented scoped form, uncapped/stub tree):
`103 passed / 2 failed / 0 ignored / 2 suites`, failures =
`cap_zero_memo_is_pure_recompute_ablation` (semantic memo len 7 != 0 -
names the cap-0 case as the handoff requires) and
`counters_stay_capped_when_demand_exceeds_every_cap` (len 700 > 512),
with the equality matrix PASSING on the uncapped tree. Arithmetic
independently verified: baseline instrumented green = 103 + 4 new = 107;
RED fail-fast ran lib (101) + new binary (2 pass/2 fail) = 103/2, suites
2 - fully consistent, no missing-suite dodge. "compile-red is not red"
was handled via the handoff-sanctioned no-op stub slice (zero behavior
change), so the equality matrix is a pre-swap witness by construction.
Suite content verified to pin RESULT equality, not counters: the
counter-free test compares `Vec<LoadMetric>` cold AND warm sweeps against
the `LoadAnalyzer::new` uncapped reference across capacity points
{0, 1, hop caps, usize::MAX} on 6 fixtures including an over-cap demand
model (700 > 512/128); `LoadMetric` derives plain
`PartialEq, Eq` (`load.rs:38-52`) - full structural equality, every
symbol vector compared. No retained dual representation; the old
append-only insert is fully replaced by `admit_entry`.

GREEN (fresh, both forms): see section 4.

## 3. Cap + pin adjudication (hop-fixed literals, reviewer's own merits)

ACCEPT all three:
- `SEMANTIC_CLOSURE_BOOT_CAP = 512`, `PHYSICAL_EXPANSION_BOOT_CAP = 128`:
  arithmetic against Block 4 re-verified exactly (512/5024 = 10.2%;
  128/1280 = 10.0%; retained refs 318,208 + 632,832 = 951,040 vs
  3,122,416 + 6,328,320 = 9,450,736 - a >9.9x retention cut at N=1280);
  the tighter relative cut lands on the memo Block 4 names "the first
  eviction candidate", honoring the ratified table's ranking.
- First-N admission vs a least-value policy: ACCEPTED - no quality cliff.
  Boot access is single-use per key (shape A1/A3), so NO admission order
  can convert into boot hit-rate; post-boot consumers keep a separate
  fresh uncapped memo (`ui.rs:289`). Result equality under
  demand>capacity is forever-locked by I5 (over-cap fixture, cold+warm);
  the residual cost is recomputation time only, guarded by the
  bench_compare stage-5 medians and the post-T2 live I7a gate. LRU/
  least-value would buy zero measurable boot win while adding bookkeeping
  and a staleness surface. Admitted entries are permanent - zero
  invalidation logic, zero staleness surface.
- `I7C_CACHE_LEN_PIN = 512` machine-independence: SOUND. The counter is
  admission-capped by construction (`admit_entry` holds `len >= cap`
  inside the same critical section), the assert is pure arithmetic
  (counters <= pin == largest cap), no wall/RSS/machine quantity enters.
  The over-demand falsifier's `== cap` assertion is order-independent
  (convergence to the cap is exact for 700 distinct keys regardless of
  iteration order). CI-safety per DP-3: deterministic, inside
  `cargo test`.
- Cap 0 = pure recompute verified in code (`len() >= 0` always true) and
  locked by the ablation test T0b reuses.

## 4. Fresh gate lines (verbatim, all reviewer-run)

- graph plain (gate-1 exact): `104 passed / 0 failed / 0 ignored / 4 suites`
- graph instrumented: `107 passed / 0 failed / 0 ignored / 4 suites`
- pipeline (UNEDITED oracle): `25 passed / 0 failed / 4 ignored / 6 suites`
- cli (ui C1 lock + stage-5 replay): `111 passed / 0 failed / 8 ignored / 6 suites`
- m00 parity: `2 passed / 0 failed / 0 ignored / 1 suites`
- m18 parity: `23 passed / 0 failed / 0 ignored / 1 suites`
- tools CI form: `Ran 194 tests in 24.855s` / `OK`
- ui_smoke (distill): `boot+route checks clean (cold fixture, attempt 1, 0.7s)` / `ui_smoke: PASS`
- hygiene: `### generic PASS (exit 0)` - all rows pre-existing R3 warn-only
  (zero rows in the new test file; `load_caches.rs:428/433` `for f` are
  pre-existing loop vars in untouched context, as the journal states)
- `encoding_guard --all`: `### generic PASS (exit 0)`, silent

## 5. Workspace census and the D3 coverage truth

Fresh unscoped: `1240 passed / 0 failed / 13 ignored / 90 suites` (exit 0).
Reconciles from the T0a anchor `1236 / 0 / 13 / 89` at exactly +4 passed /
+1 suite / ignored-and-locked held - delta equals this hop's added tests
only. TRUTH STATEMENT: the dispatch note's "+2 counter tests if the
feature unifies" UNDERCOUNTS - three tests are `test-instrument`-gated,
and the census executed ALL THREE plus the I5 matrix: capture lines 364-
367 show `cache_counters_stay_within_hop_caps_and_ci_pin`,
`cap_zero_memo_is_pure_recompute_ablation`,
`counters_stay_capped_when_demand_exceeds_every_cap`,
`analyze_results_are_identical_across_every_memo_capacity` all `ok`
inside the unscoped `cargo test --workspace` build. The unification via
`crates/ui/Cargo.toml:26` (dev-dep `features = ["test-instrument"]`) is
EMPIRICALLY CONFIRMED, not asserted: the I7c CI clause runs in the normal
census. No coverage finding. The unification is nevertheless implicit -
see finding 2.

## 6. bench_compare - RUN (exemption revoked by `crates/graph/src` diff) - PASS

`bench compare: 3 run(s), no regressions past 1.25x baseline` (exit 0,
first attempt). Medians of interest: `symbol_pass+prep` 1418.7;
`file_pass` 349.1; `save_load_artifacts+cache` 14765.6; `ensure_load
total` 16523.3 (1.146x vs the 14413 baseline); `plan() facts+closures`
1733.9 (1.195x vs the 1451.0 baseline). HONEST STALENESS CAVEAT: the
ledger's dated escalation row records `plan()` straddling 1.25x on the
SAME tree (1.236x-1.273x; medians 1667 -> 2016 ms across one day) and
`ensure_load total` at 17.1-17.4k on a machine-proved zero-Rust-diff
hand; today's `plan()` 1733.9 sits BELOW that escalation range, so this
PASS is genuine but baseline-staleness-dependent, and the operator
`--update-baseline` on VERIFIED-IDLE remains outstanding. Judged on the
milestone hot path: `symbol_pass+prep` 1418.7 is inside the
1.15x-1.17x same-machine band the ledger records every recent run.
Structurally, d6640a2 cannot move these columns: the uncapped path adds
one `len()` comparison per cache miss; all uncapped constructors keep
byte-identical behavior (ui C1 + pipeline oracle green UNEDITED).

## 7. RSS directional witness - NOT RUN (environment-blocked), honest statement

Attempted twice: `python tools/distill.py --timeout 3600 -- cargo test
--release -p ariadne-cli --test stage5_memory_bench -- --exact
stage5_rss_growth_ratio --ignored --nocapture --test-threads=1`. Both
attempts: the N=1280 isolated child died in the artifact-save span with
`Failed to write temp file ... \.report.json.<pid>.<seq>.tmp` /
`There is not enough space on the disk. (os error 112)` (exit 101). This
is disk capacity, not code: the identical invocation ran four-plus times
on this machine during T0a the same day. One real data point survived:
attempt 2's N=640 peak `stage5 peak RSS (release, N=640) | 2258.0` -
inside the pre-fix T0a band 2256-2259, i.e. the T1 tree reproduces the
pre-fix curve at half scale, AS EXPECTED BY DESIGN: T1 ships the knob;
the BOOT memo is still uncapped (the cap WIRING is T2's obligation), and
every production constructor is uncapped, so T1 alone must NOT move RSS.
Any N=1280 number - fail or pass - would be evidence for T0b/T2, not for
T1; per the RATIFIED protocol a single invocation is directional-only
anyway (median-of->=5 + >8% operand rejection binding both operands).
Cleanup of my two failed runs' temp dirs was permission-denied
(`Remove-Item` refused). No I7a inference is claimed for T1.

## 8. Deviation dispositions

- D1 fixture-model parenthetical: ACCEPTED. Precondition 3 demands in-code
  models / zero fixture demand; the replication mirrors the four locked
  shapes and enumerates them as the handoff requires.
- D2 `load.rs` seam dropped: ACCEPTED. Pre-authorized verbatim by gate 5;
  sufficiency proven (section 1).
- D3 `test-instrument`-gated counter tests: ACCEPTED; coverage empirically
  confirmed in the census (section 5); see finding 2 for the implicitness
  note.
- D4 `adversarial_wide_fixture` over-demand falsifier inside the same
  locked file: ACCEPTED - it is what makes the I7c "at all times" clause
  falsifiable, and "extend the same file rather than scatter" is the
  handoff's own instruction.
- D5 single commit: ACCEPTED. Handoff gate 5 specifies scopes, not commit
  count; the red-first proof the review-gate consumes (identical frozen
  test file: RED capture pre-policy, GREEN post) is intact and arithmetic-
  consistent; `commit_scoped` whole-file semantics make the stub state
  uncommittable once the policy filled the same file.
- Guard-shape relaxation (journal guard audit, undeclared as a D-row):
  ACCEPTED. `admit_entry`'s single critical section spans `len` + `insert`
  only - no nested acquisition, no BFS, no rayon boundary; the check-and-
  insert MUST be atomic or the cap overshoots under concurrent misses,
  which is the I7c "at all times" clause. The header doc :7-19 stays
  verbatim true; the doc at :477-483 states the shape honestly.

## 9. Findings (numbered)

1. (minor, journal) `tasks/...-t1-journal.md:190-191` promises the gate-4
   `encoding_guard --staged` result is "quoted in the commit section
   below", but the Commit record section (:238-253) never quotes it -
   dangling report-evidence reference (review-gate step 8). Substance
   cleared by the reviewer's fresh `--all` exit 0. Required action: append
   the one-line witness at next journal touch; not a merge blocker.
2. (informational, planner/ledger) The census execution of the three
   counter tests rests on an implicit feature unification through
   `crates/ui/Cargo.toml:26`; nothing pins it. Detection net: any future
   hop whose census reconciles `-3` passed at `90 suites` (or whose touch
   set narrows/removes that dev-dep feature). Suggested ledger row: "I7c
   counter tests ride ui's dev-dep `test-instrument` feature unification -
   census count is the tripwire." Not actionable inside T1: Cargo.toml
   was frozen by the handoff and the implementer declared the mechanism
   under D3.
3. (operator duty, pre-existing condition surfaced) The machine's C: free
   space currently cannot hold an N=1280 artifact save - the release RSS
   curve died twice on `os error 112`. Before T0b's post-T1 curve re-run
   or any post-T2 I7a live gate, free disk (and note the working-set
   protocol already demands VERIFIED-IDLE for baseline writes).

## Follow-up dispositions (relay to the orchestrator, not dropped)

- T2 MUST cite `d6640a2` for cap wiring via
  `LoadMemo::with_closures_and_capacity` + existing `with_memo`
  (`load.rs:201`); caps cite as hop-fixed: 512 / 128, pin 512 (reviewer
  adjudicated at this hop - say "reviewer-adjudicated hop-fixed", never
  "review-ratified"). Cap 0 = T0b ablation lever, locked.
- The I5 file `crates/graph/tests/load_memo_capacity_invariance_test.rs`
  is forever-locked: every later hop's reviewer re-runs it UNEDITED;
  run the scoped graph suite BOTH forms (plain + `--features
  test-instrument`), or let the unscoped census prove the counter tests.
- Finding 3 (disk) is a prerequisite for the T0b curve re-run; findings 1
  and 2 are ledger/next-touch items.
