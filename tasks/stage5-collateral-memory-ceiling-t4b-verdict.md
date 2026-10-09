# Verdict: stage5-collateral-memory-ceiling T4b (growth-gate G demotion to tripwire)

- **Verdict: APPROVED**
- Reviewer: reviewer subagent (review-gate skill), 2026-10-07
- Range: base `e9a4ac1` (parent of `1df4ce4`, = T0c landing) -> HEAD
  `373a160`; commits `1df4ce4` (feat), `777653f` (docs), `373a160`
  (journal hashes), reviewed oldest-first, each judged alone
- Authority accepted as given (user decision 2026-10-06, decision 3): G is
  a tripwire; acceptance = 5200 MiB peak bars (both profiles) + Corpora
  field boot. Nothing re-ratified.

## 1. Commit-vs-touch-set and locks

- `git diff --name-only e9a4ac1..373a160` = EXACTLY the four touch-set
  paths: `tools/bench_compare.py` (35/21), `tools/bench_compare_tests.py`
  (31/7), `tasks/...t0a-evidence.md` (**26/0**),
  `tasks/...t4b-journal.md` (164/0).
- Per-commit: `1df4ce4` = the two tools files; `777653f` = evidence doc
  + journal; `373a160` = journal only. Every commit inside the touch-set.
- `tools/bench_baseline.json` diff-EMPTY across the range AND
  worktree-clean at review; **zero `crates/**` paths in the range** (the
  dirty `crates/report/**` et al. are UNCOMMITTED, in-flight T4 work,
  never in this range - finding 3 relay). `Cargo.lock` untouched (pure
  Python + docs hop).

## 2. Semantics verification (code read + fresh runs)

- Growth branch: keeps computing, keeps printing, **never sets
  `regressed`** - the `regressed = True` line is deleted; strict `>`
  boundary preserved (`growth_ratio > STAGE5_MEMORY_GROWTH_TRIPWIRE_RATIO`
  :424).
- Tripwire line built from the label constant -> tuple-identical label;
  prints `TRIPWIRE stage5 RSS growth ratio (debug, N=640->1280): <v>
  exceeds the demoted growth gate 2.0 - tripwire only, never a gate
  (user decision 2026-10-06; the fixture is Theta(N^2) by construction,
  see t0c-verdict sec. 7)` - lands in the WARN/verdict tail region.
- **Live witnesses (fresh, reviewer scratch, temp-dir baselines,
  `bench_baseline.json` never referenced)** - capture 9 lines, exit 0:
  - A ratio 4.00 in-band peak: `exit=0` + the TRIPWIRE line verbatim
    above (names the user decision).
  - B peak 5300: `exit=1` + `WARN stage5 peak RSS (debug, N=1280):
    5300.0 MiB exceeds the RATIFIED 5200 MiB peak bar (N=1280)` - the
    bar string verbatim, byte-identical to the T0b witness.
  - C/C2 ratio exactly 2.0: `exit=0`, **no TRIPWIRE line** (strict `>`).
  - D ratio 4.00, ratio row absent from baseline: `exit=0` + TRIPWIRE
    present - tripwire independent of baseline presence.
- Frozen literals: `STAGE5_MEMORY_PEAK_BAR_MIB = 5200.0` (:187) and
  `STAGE5_MEMORY_REGRESSION_RATIO = 1.50` (:221) appear in NO diff hunk
  and read at current values; the peak-bar WARN branch is unchanged diff
  context; the 1.50x per-row-vs-baseline semantics incl. the ratio row as
  a BASELINE row are preserved (`test_growth_ratio_row_regression_is_
  first_class` green UNEDITED); timing mode + sampling law + pre-PR-2
  hint carry no hunks.
- Grep-proofs: `GROWTH_GATE` identifier GONE from all of `tools/`; old
  growth `WARN ... RATIFIED growth gate` string gone (historical comment
  prose only at :189); peak-bar WARN intact; USAGE + verdict_tail name
  peak bar + tripwire, growth number kept visible in the pass tail.
- **Peak-bar quadrant test UNEDITED**: diff shows zero edited lines in
  the :591-605 region (appears only as diff context); read at HEAD
  confirms :592-605 intact. Sampling-protocol and mode-isolation suites
  UNEDITED (all deletions confined to the two authorized hunks - the
  canonical-docstring reword keeping every number, and the G-quadrant
  inversion + one new case; 45->46 = exactly +1 case; row-65 prior `Ran
  45 tests in 0.339s / OK` quoted in journal BEFORE inverting).

## 3. Evidence-doc append-only proof

- Landing numstat `26 0` on the evidence doc; full diff is one pure-append
  hunk `@@ -349,4 +349,30 @@` with ZERO `-` lines.
- The superseded line survives VERBATIM at :298-299 (`RATIFIED:` (same
  date/authority) **growth gate G: `stage5 RSS growth ratio` <= 2.0.**),
  above the note.
- Note: dated 2026-10-06 decision / landed 2026-10-07; cites decision 3,
  cites `t0c-verdict sec. 7` grounding with witness numbers, cites this
  handoff as landing contract; states the demotion, the acceptance
  (5200 bars + Corpora field boot), the never-fails guarantee, and
  "Nothing measured or ratified elsewhere is touched" (peak bars +
  sampling protocol named explicitly; N0/scales/4096-rider covered by the
  blanket clause). Substantively compliant with the note contract.
- The journal's self-caught pre-commit violation (first write `27 1` via
  a stray leading space on a preserved line; fixed before staging to
  `26 0`) is exactly what the landed diff shows - honest errata-on-sight;
  the violation never existed in git history.

## 4. Journal and gate-count adjudication

- Journal gate digests 1-6 match my fresh runs at every readable point
  (`Ran 46 ... OK` = journal's exact line; --help exit-1 wording verbatim;
  hygiene zero rows for touched files; RED `AssertionError: 1 != 0` text
  arithmetically consistent with the pre-change code's exit-1 behavior -
  Red states verified by consistency per the T2 precedent, read-only).
- Deviation dispositions:
  - Constant rename `..._GROWTH_GATE_RATIO` -> `..._GROWTH_TRIPWIRE_RATIO`
    with value 2.0: **ACCEPTED** - handoff explicitly permits ("rename is
    allowed if the diff stays honest"); diff is honest; (the journal's
    claim that the old NAME survives in the comment is over-cautious: the
    comment keeps prose, not the identifier - grep-proven gone).
  - Commit split code/docs (`feat(tools)` / `docs(tasks)`): **ACCEPTED**
    - repo convention with landed T0b precedent; the single-session rule
    is honored (same session, evidence-doc note authored with the code in
    this hop). Process cost feeds finding 2.
  - Self-caught append-only fix: **ACCEPTED** (sec. 3).

## 5. Fresh gate lines (canonical, verbatim)

| Gate | Result |
|---|---|
| Scoped discover | `### generic PASS (exit 0)` / `Ran 46 tests in 0.305s` / `OK` |
| CI-form discover | `Ran 219 tests in 18.056s` / `OK` (= 218 prior + 1) |
| `--help` | `### generic PASS (exit 0)`, both `--bench` choices |
| Hygiene | `### generic PASS (exit 0)`; 160 lines, ZERO rows on any touched path |
| `encoding_guard --all` | `### generic PASS (exit 0)`, capture 0 lines |
| `ui_smoke` | `boot+route checks clean (cold fixture, attempt 1, 0.9s)` / `ui_smoke: PASS` |
| Census (live re-run) | BLOCKED by in-flight T4 (finding 3): `305 passed / 2 failed / 8 ignored / 19 suites`, both failures = golden suites mid-T4-reshape, fail-fast truncation |
| Census (contract proof) | handoff's own clause: "Python-only hop - zero cargo delta is the proof" - PROVEN (range diff sec. 1); inputs at HEAD byte-identical to `e9a4ac1` whose census was `1250 passed / 0 failed / 13 ignored / 91 suites` at the T0c review; ledger M28 anchor = `1250` (accepted as 2026-10-07 decision) |

## 6. Findings (numbered; none blocking)

1. **Minor - out-of-contract docstring drift.** `bench_compare.py:36-40`:
   the exit-2 docstring line lost `/ timed-out` ("failed/timed-out bench
   run" -> "failed bench run"). The contract named ONLY the exit-1 wording
   for change. Behavior is intact (timeout -> `BenchRunError` :458-462 ->
   exit 2 :749) - the docstring now under-documents a live behavior.
   Restore the word at the next touch.
2. **Minor (process) - undeclared gate-invocation overshoot vs the <= 6
   budget, no itemized final count.** The repo's own convention (code /
   docs / hash-recording = 3 scoped commits, each with staging +
   `encoding_guard --staged` + `commit_scoped`) makes the honest routed
   count ~9-10, not <= 6; the journal itemizes gate digests but no
   final count (T0b finding 1 law; T0c's D6 is the compliant shape).
   The commit-split deviation is accepted on merit - the silence is the
   finding. PLANNER RELAY: this is the third consecutive hop where the
   handoff budget and the commit convention collide (T0b minor, T0c
   declared-and-accepted, T4b undeclared); future hop handoffs should
   budget the commit-split cost or fold the hash record into the doc
   commit.
3. **Informational (environment/coordination) - census blocked by
   in-flight T4 in the shared worktree.** The worktree was clean at
   review-start ground truth (19:31) and by census time (19:5x) carried
   UNCOMMITTED T4 work: `crates/cli/src/commands/measure.rs`,
   `crates/index/src/cache.rs`, `crates/report/src/manager.rs`,
   `crates/report/src/manager/metrics_io.rs`,
   `crates/report/src/manager/tests.rs`, untracked
   `crates/report/src/manager/payload_reshape_tests.rs`,
   `tests/golden/{basic,expensive}/load_report.json`,
   `tests/integration/tests/m12_incremental_analysis_test.rs`,
   `tests/integration/tests/m21_scala_semantic_index_test.rs`. The live
   census compiled mid-edit T4 sources and died on the two golden suites
   (load_report line counts 237 vs 634 / 263 vs 636) with fail-fast
   truncation at 19 suites. None of these paths is in this range
   (sec. 1) - T4b cannot move them; this is T4's declared territory
   (T4b handoff parallelism note names `crates/**`, the golden pair,
   `crates/index`). Relays: (a) the T4b census witness is owed clean in
   a quiet window or at T4 landing; (b) the T4 reviewer must expect
   mid-flight dirt and re-derive its own baseline state - and note
   `crates/index/src/cache.rs` is on the M28 frozen-lock list, so T4's
   lock proof must address its (legitimate, territory-permitted) touch;
   (c) shared-worktree parallelism held at the TOUCH-SET level but not
   at the BUILD level - worth a ledger row if the pattern recurs.
4. **Purge-queue relay (row-67):** reviewer scratch
   `tasks/t4b-review-probe.py` (new) joins `tasks/t0b-review-probe.py`,
   `tasks/t0c-review-probe.py`.

## 7. Disposition

APPROVED. `373a160` stands as the T4b landing. The demotion is exactly
the user decision's shape: G computed, printed loud, never gate-failing;
the 5200 bar untouched and still failing the compare path; the 1.50x
baseline-row semantics preserved; sampling law and pre-PR-2 shape intact;
append-only discipline honored with an honest self-caught pre-commit
erratum. Findings 1-4: 1-2 are fix-at-next-touch relays, 3-4 are
orchestrator/ledger relays. No implementer action required to hold the
APPROVED state.

## Scratch-witness list (this review)

- `tasks/t4b-review-probe.py` (live-shape witnesses A/B/C/C2/D;
  temp-dir baselines only, `tools/bench_baseline.json` never opened)
- capture artifacts: `target/distill-capture.txt` (overwritten per run)
