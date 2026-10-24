# SM-5 Execution Results - extraction-symbol-move (reviewer fresh runs, 2026-10-24)

Reviewer hop for SM-5, the final hand. Every figure below is THIS reviewer's
own fresh run at HEAD `522a770f68abda4ca5cf0dd4604108bd285de1a1` on `main`;
nothing relayed. These workspace gates double as the milestone-final
acceptance census for `extraction-symbol-move`.

## Against the handoff's acceptance criteria (SM-5)

| # | Criterion | Result | Evidence line (verbatim) |
|---|---|---|---|
| 1 | Red: exactly two named failures | PASS (journal evidence, layout-consistent) | journal: `370 passed / 2 failed / 0 ignored / 2 suites` exit 1; panic sites `:434:5` and `:916:5` match the authored file (`:434` is the `wizardCopyOnly(` clause); `370+2 = 372` = final count |
| 2 | Green scoped suite | PASS | `python tools/testrun.py --distill -p ariadne-ui` -> `### cargo test PASS (exit 0)`, `372 passed / 0 failed / 0 ignored / 3 suites` (= baseline 371 + the new lock, amended test count-neutral) |
| 3 | `ui_smoke` served-bytes change, boot holds | PASS | `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture, attempt 1, 0.7s)` / `ui_smoke: PASS` |
| 4 | theme tokens zero added hex | PASS | `python tools/distill.py -- python tools/theme_token_check.py --check-added --base e717ba0` -> `### generic PASS (exit 0)`, silent, capture 0 lines |
| 5 | census `1853/0/13/119` exactly | PASS | `python tools/testrun.py --distill` -> `### cargo test PASS (exit 0)`, `1853 passed / 0 failed / 13 ignored / 119 suites` |
| 6 | bench fresh recording, WARNs not attributed | PASS (clean) | `bench compare: 3 run(s), no regressions past 1.25x baseline`, exit 0 - implementer WARN NOT reproduced (see below) |
| 7 | hygiene exit 0 | PASS | `### generic PASS (exit 0)`; R3 warns all pre-existing, none in the hand's files |
| 8 | encoding guard | PASS | `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`, capture 0 lines |
| 9 | commit-vs-touch-set | PASS | `git diff --name-only fd96b09..522a770` -> exactly the 3 touch-set paths; asset 16/4, render 57/0, journal 156/0; no Cargo.lock |
| 10 | reflog SHA law | PASS | `.git/logs/HEAD` 447=`fd96b0947d6124ac84902070a9a1f37bbbd38b40`, 448=`82cf789efa92c7c311bf16367313ead4000f8f12`, 449=`522a770f68abda4ca5cf0dd4604108bd285de1a1`=HEAD |

## Against the milestone acceptance gate (kickoff sec. 8, whole-milestone)

| # | Gate item | Result | Evidence |
|---|---|---|---|
| 1 | Fixture apply marks `include/thing.h` per grammar, m19/m20 green with byte-identical remainder | PASS (SM-2/SM-3 gates + this census green; grammar re-checked in-file) | `move_retention.rs:31-32` `// ARIADNE-EXTRACT-OUT BEGIN plan=` / `... END plan=`; SM-3 ledger record: m20 rider pins `plan=PLAN-001 symbols=Thing` at the strip call; m19/m20 green inside `1853/0/13/119` |
| 2 | Grown faithfulness oracle green through the UI applier | PASS (SM-4 discharges; intact at HEAD) | required-test-1 widening = hard-chain discharge at `e717ba0`; C5.5 parity measured 1 == 1 (ledger SM-4 row); census green |
| 3 | Copy truth: no "strips" claim, amended W4 lock green, CW-3 locks + claim of record green UNEDITED | PASS | grep: `strips` zero occurrences in `extraction-wizard.js`; `one-click` only in negated copy (`:223-224`); amended W4 lock keyed (verdict sec. 4) and green in the 372-run; CW-3 locks + hotspots claim-of-record UNEDITED (outside all five hunks) |
| 4 | Consumer-side zero regression | PASS | `git diff --numstat e717ba0..HEAD -- crates/cli crates/lang crates/lang-cpp crates/validate crates/transform tests/golden tests/fixtures tools Cargo.lock` -> EMPTY; SM-1..SM-4 records held the per-hand zero-diffs |
| 5 | Rollback still covers the original file | PASS | `move_removal_transaction_test.rs` UNEDITED green inside the census (SM-I7 route, per-hand ledgers) |
| 6 | Workspace gates exit 0 at every hand and at the close; commit-vs-touch-set exact | PASS | every hand's verdict records its gate lines; THIS close: all lines below; SM-5 touch-set exact (table row 9 above) |
| 7 | ASCII throughout | PASS | `encoding_guard --all` exit 0 silent at HEAD; asset carries its ASCII locks green |

## Fresh workspace gate lines (verbatim, this hop)

- `python tools/testrun.py --baseline` -> `usage error: missing value for --baseline` (FILE-form refusal expected; `--distill` substitute governs)
- scoped: `372 passed / 0 failed / 0 ignored / 3 suites` (exit 0; `target/testrun-capture.txt`, 409 lines)
- FULL CENSUS (milestone final): `1853 passed / 0 failed / 13 ignored / 119 suites` (exit 0; `target/testrun-capture.txt`, 3213 lines)
- `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 28.046s` + `OK`
- `ui_smoke: PASS` (exit 0; distill capture 2 lines)
- `bench compare: 3 run(s), no regressions past 1.25x baseline` (exit 0)
- hygiene: `### generic PASS (exit 0)` (R1/R2 clean; R3 warns pre-existing only)
- encoding: `### generic PASS (exit 0)` (capture 0 lines)
- theme tokens: `### generic PASS (exit 0)` (silent)

Census reconciliation (from the anchor): `1837 passed / 0 failed / 13 ignored /
119 suites @ b201aa6` + SM-1 +2 (1839) + SM-2 +10 (1849) + SM-3 +0 (exempt,
one designed red, discharged by SM-4) + SM-4 +3 (1852) + SM-5 +1 (1853) =
EXACTLY the milestone expectation. Zero foreign residue; suites count steady
119 (test appended in-file, no new suite).

## Bench stance (milestone-close position)

Implementer gate 7 recorded exit 1 with one WARN
(`WARN get_or_build_include_graph clone (warm): 1.3 ms vs baseline 1.0 ms
(+30%)`) plus producer zero-diff proof. This reviewer's fresh run: exit 0,
warm clone median 1.0 ms, no regressions past 1.25x. NOT reproduced;
attribution = PR-2 per-machine baseline class (SM-4 precedent), NOT
attributed to SM-5 (its diff touches only ui asset/test/journal; producer
area empty-diff proven above). All five hands recorded fresh benches;
milestone ships zero attributable performance regressions.

## Not run / refused (explicit statement)

- Nothing from the required gate list was skipped. Refused probes (allow-list
  gaps, boilerplate catch-all `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
  `git status --short`, a compound `git rev-parse; git status | Select-Object`
  command, and `python -c` one-liners. Sanctioned routes used instead
  (ledger SM-2 finding M5): porcelain via `checkpoint_guard status`, reflog
  via Read on `.git/logs/HEAD`, exit codes via `distill.py`. Byte-equality of
  the mandated sentences: machine-proven for asset==test by the green
  `contains(MOVE_RETENTION_SENTENCE)` lock; handoff-vs-ship verified by
  verbatim Read comparison (temp-script byte check itself denied by the
  reviewer write-grant boundary - disclosed, minor).
- `bench_compare --bench stage5-memory` variant and `ui_smoke --release`:
  not mandated by this handoff or the milestone gate - not run.
- Golden guard tooling (`golden_guard.py`): does not exist in `tools/` -
  handoff's prose byte-checks used as prescribed.
