# Verdict: stage5-collateral-memory-ceiling T4 (payload reshape + streaming save/load + epoch 2.2.0)

- **Verdict: APPROVED**
- Reviewer: reviewer subagent (review-gate skill), 2026-10-08
- Range: base `373a160` (T4b landing) -> HEAD `fd7c1ec` (one landing
  commit; message-only-with-journal amend of dangling `d43571a`)
- This hop's acceptance curve is the milestone's terminal fix reading;
  the ratified 5200 MiB bars consume it. Nothing re-ratified.

## 1. AMEND TRUTH (duty 1)

- `git show -s --format="%B" d43571a` (dangling, readable): exactly one
  paragraph - the bump-lineage text WITHOUT the 2.2.0 entry; missing the
  DP-2 payload and golden-resync justifications. D7's defect story is
  corroborated in full, including that the repeated-`-m` argparse quirk
  kept only the last drafted paragraph.
- `fd7c1ec` message contains all four gate-10 items: the DP-2 payload
  justification, the golden-resync justification, the bump lineage with
  `2.1.0 -> 2.2.0 (M28 T4 DP-2)`, and the measurement line `release
  N=1280 peak median 4379.7 MiB vs the ratified 5200 bar, accepted 5 of
  5, save-tail delta ~0`.
- `git show --stat fd7c1ec` = exactly the 11 paths (measure.rs, cache.rs,
  manager.rs, metrics_io.rs, payload_reshape_tests.rs NEW, manager/
  tests.rs, journal, the two goldens, m12, m21). `Cargo.lock` diff-EMPTY;
  parent = `373a160` EXACT.
- **The same-tree claim is FALSE at letter**: `git diff d43571a fd7c1ec`
  is NOT empty - it is a 12-insertion, JOURNAL-ONLY delta (the D7 row
  itself was folded into the amend). All ten code/test/golden paths are
  byte-identical (delta confined to one touch-set tasks file).
  Finding 1; substantively harmless, audit-truth-wise false as written.
- The journal does not record the landing hash `fd7c1ec` anywhere (it
  cites `d43571a` + symbolic "HEAD"); the precedent hash-recording commit
  (`dc294bb`/`e9a4ac1`/`373a160` class) has not happened. Fold the
  erratum + hash record into the follow-up docs commit (finding 1's
  required follow-up).

## 2. REGISTERED-EXCEPTION LOCK AUDIT (duty 2)

- `crates/index/src/cache.rs` delta = EXACTLY the registered bump: const
  `"2.1.0" -> "2.2.0"`, assert :347, lineage doc entries (M28 entry with
  user-decision citation, M27 entry preserved). ZERO logic lines.
- Dependents, same lens: m12 = pin :99 + lineage comment only; m21 =
  witness + message only; `LOCK_MANIFEST_JSON_HASH` resync with the
  version-mechanism-only justification and full prior-value lineage
  (`0xbee9_1e36_1bbe_1e4b` kept, itself citing `0xddc3_f9ee_566b_5ea6`).
  JSON/BIN locks resynced with payload justifications + prior values per
  the :596-600 precedent (D2).
- `measure.rs` delta is COMMENT-ONLY (+6): the chosen representation
  makes the consumer's existing sum read the persisted scalar rows -
  identical by construction, pinned by R3. `manager.rs` = the 2-line
  `#[cfg(test)] mod payload_reshape_tests;` registration the handoff
  admits on module creation.
- **Green-tree probe (erratum-compliant, no grep-as-proof)**: my FRESH
  full-workspace census (below) is the green-tree constant proof - all
  five dependent sites green in-tree; no sixth site exists (literal
  inventory: cache.rs const+assert, m12, m21 + historical comment prose;
  the only other `"2.1.0"` code occurrence is R2's deliberate stale-epoch
  fixture). m27 contains NO version literal - its UNEDITED status is
  structural.

## 3. GOLDENS (duty 3)

- Range diff over `tests/golden/**` = EXACTLY `basic/load_report.json` +
  `expensive/load_report.json` (from the landing's 11-path list; all
  other goldens byte-identical by absence). m00 green post-resync UN-
  EDITED (fresh: `golden_baseline_basic ... ok` / `golden_baseline_expensive
  ... ok`), m18 parity green UNEDITED (fresh 23-case population in the
  integration capture).
- Content spot-check vs the DTO: the resynced goldens are exactly the
  new persisted contract - `symbol_metrics` key GONE (grep-verified
  absent across `tests/golden`), `symbol_summary { analyzed_symbols,
  collateral_load }` present, `file_metrics` full UI-consumed detail
  intact; 381/405 deleted lines = the per-symbol sections only.

## 4. ACCEPTANCE CURVE TRUTH (duty 4)

- Ratified helper re-run on the five `target/t4-postfix-*.txt` captures
  (reviewer-fresh, verbatim): `accepted 5 of 5` (zero rejections,
  consistent with the journal's "none" claim);
  `MEDIAN(accepted) stage5 peak RSS (release, N=1280) | 4379.7` (<= 5200.0,
  820.3 MiB slack); `MEDIAN(accepted) stage5 peak RSS (release, N=640) |
  1063.8`; `MEDIAN(all) RSS checkpoint save tail (release, N=1280) |
  4377.7` -> derived save-span transient **2.0 MiB** (from 4098.0: dead
  to checkpoint noise); `MEDIAN(all) stage5 RSS growth ratio (release,
  N=640->1280) | 4.1` - TRIPWIRE-only per user decision 2026-10-06;
  journal carries no PASS/MISS wording for it (checked).
- **Independent recompute** (reviewer probe, own parser, standalone
  multiset law): `accepted 5 of 5 (independent law, multiset operands)`,
  peak medians `4379.7` / `1063.8`, ratio `4.1`, save tail `4377.7`,
  transient `2.0`, `acceptance: 4379.7 <= 5200.0 -> True`. Exact match
  with the helper - no fabrication surface.
- Fresh directional witness (one release N=640 isolated invocation,
  reviewer-run): `stage5 peak RSS (release, N=640) | 1063.0` with save
  tail 1063.0, save-tail delta 0.0, post-return delta 0.0 (pre-T4 same
  form: peak 2090.4 vs tail 1065.2 - the ~1024 half-scale transient is
  GONE); `ensure_load wall | 0.7` s (previously ~1.5); memo rows
  byte-identical. Single invocation - directional, and its shape is
  exactly what the five-invocation group shows.
- LIVE debug group (via the hint-form run below): peak@1280 `4364.3` vs
  save tail `4361.2` (pre-T4 debug peak 8470.9) - BOTH ratified profiles
  now under the 5200 bar.

## 5. FRESH GATES (duty 5, canonical lines verbatim)

| Gate | Result |
|---|---|
| report | `41 passed / 0 failed / 0 ignored / 3 suites` EXACT (baseline `36/0/0/3` + 5 new; row-65 honored) |
| integration | `141 passed / 0 failed / 0 ignored / 35 suites` EXACT (m00 resynced goldens + m18/m6/m17/m27 UNEDITED green) |
| pipeline plain (IV6) | `32 passed / 0 failed / 4 ignored / 7 suites` EXACT |
| internal oracles | `12 passed / 0 failed / 0 ignored / 6 suites` EXACT (UNEDITED) |
| graph plain / instrumented | `104/0/0/4` / `107/0/0/4` EXACT (I5 file diff-EMPTY by scope) |
| tools CI-form | `Ran 219 tests in 17.895s` / `OK` (zero delta) |
| hygiene | exit 0; ZERO rows on any touched path incl. the new module |
| encoding_guard --all | exit 0, capture 0 lines (substantive gate for D5's disclosed vacuous --staged run) |
| ui_smoke debug | `boot+route checks clean (cold fixture, attempt 1, 1.0s)` / `ui_smoke: PASS` (warm+cold load-path field proof) |
| ui_smoke --release | `ui_smoke: PASS` (0.6s) |

## 6. CLEAN-TREE CENSUS (duty 6 - owed since T4b finding 3)

- `### cargo test PASS (exit 0)` [capture 2221 lines] /
  `1255 passed / 0 failed / 13 ignored / 91 suites` - matches the
  implementer's probe line EXACTLY.
- Reconciliation from the `1250` anchor: **+5 = the five
  `manager::payload_reshape_tests` cases** (capture lines 1493-1507:
  persisted_json_absence, round_trip_scalars, stale_epoch, totals_pin,
  manifest_fields). m27's completion test is present (line 879) but
  contributes ZERO to the delta - its +1 is baked into the 1250 anchor
  (handoff bump-procedure site 5). The dispatch's guessed split
  (m27 + 4) was wrong; the journal's split is the correct one. New anchor
  for subsequent hops: `1255 passed / 0 failed / 13 ignored / 91 suites`.

## 7. LIVE BENCH (duty 7)

- `--bench stage5-memory` (pre-PR-2 hint form): `### generic PASS
  (exit 0)` with `stage5-memory sample accounting: accepted 5 of 5
  invocations, rejected 0`; `WARNING no baseline for machine class
  '...::stage5-memory'; regression compare skipped. Ready-to-paste
  baseline JSON:`; `basefile not written` (verified: the on-disk baseline
  diff is unchanged pre/post). No TRIPWIRE line in the hint class is the
  CONSISTENT T0b-era no-entry-no-verdict shape (bars/tripwires are
  compare-path; class-absent = no compare). RECORD ONLY - not a pass.
- Default mode: `### generic PASS (exit 0)` / `bench compare: 3 run(s),
  no regressions past 1.25x baseline` - the first clean timing witness
  in the review lineage; `ensure_load total 4674.9` (was 14787.7 at the
  T4b review - the streaming save also cut wall time),
  `save_load_artifacts+cache 2422.9`, ZERO WARN lines. This PASS is
  measured against an UPDATED baseline - see finding 2.

## 8. JOURNAL AND DEVIATIONS (duty 8)

- D1 declared reserve ~21 / D6 final 26 itemized with named causes
  (compile-not-Red honesty, dispatch-added census probe, landing
  support): **ACCEPTED** - the T0c D3/D6 law was HONORED this hop
  (declare up front, itemize at close).
- D2 lock resyncs (3, with lineage + justification): **ACCEPTED** -
  verified byte-for-byte above.
- D3 golden exception (exactly two): **ACCEPTED** - verified.
- D4 representation + m6 determination (scalar rows keep the loaded
  count; m6:178 is a loaded-artifact read; IV6 oracles green UNEDITED):
  **ACCEPTED** - the preferred in-memory-API-preserving shape landed
  exactly as the handoff predicted; m6 green UNEDITED fresh.
- D5 gate-9 adaptation (staged-run vacuous, `--all` as the substantive
  post-landing gate): **ACCEPTED** - disclosed honestly; reviewer's fresh
  `--all` is silent.
- D6: **ACCEPTED** (above).
- D7 amend: story TRUE, executed correctly in substance, but the
  recorded proof claims are false at letter - **finding 1**.
- Red discipline: three predicted Red carriers + labeled green-by-design
  pins (R3 class, T0c D2 reading); the round-trip-pin rewrite is the
  mandated contract change with falsifiable seeding (non-empty detail
  vectors), not an assertion weakening.

## 9. Findings (numbered)

1. **MAJOR (record correction; substantively non-blocking) - the D7
   same-tree claim is false as written.** `git diff d43571a fd7c1ec` is
   not empty: the amended commit folded the 12-line D7 journal row into
   itself ("tree byte-identical" holds ONLY for the ten non-journal
   paths - which I verified empty individually; the touch-set and
   one-commit discipline make the delta harmless). The journal also
   never records the landing hash `fd7c1ec`. REQUIRED FOLLOW-UP
   (orchestrator/next tasks touch, not my grant): a journal erratum row
   recording (a) `fd7c1ec` as the landing hash, (b) the precise amend
   delta (`12-insertion journal-only; code tree identical; d43571a
   superseded`), and the corrected proof form. This verdict carries the
   evidence until then.
2. **Relay (environment/PR-2 timing hazard) - the baseline update landed
   DURING this review window.** At ground truth (2026-10-08 02:31Z)
   `tools/bench_baseline.json` was clean vs HEAD (recorded 2026-10-01
   values); by the default-mode bench observation (~03:05Z) it carried an
   uncommitted operator update (recorded 2026-10-07, runs 3, post-T4
   profile: `file_pass 612.1`, `ensure_load total 4573.0`,
   `save_load_artifacts+cache 2375.4`). All reviewer results stayed
   internally consistent and green (cargo lock serializes), but a
   `--update-baseline` executing inside a heavy-gate window cannot
   satisfy the PR-2 VERIFIED-IDLE precondition by construction -
   unverifiable from here, flagged as protocol timing hazard. The update
   remains UNCOMMITTED (tracked file, 12/12 vs HEAD); PR-2 completion =
   operator commit/decision (rows 62/69/70 and the row-69 cap-economics
   re-judge stay open exactly as the dispatch states).
3. **Relay (tooling, T0b-era surface, not T4)** - the ready-to-paste
   snapshot hygiene has two blemishes visible in the live capture: the
   harness-prefixed row `test stage5_rss_growth_ratio ... stage5 peak
   RSS (debug, N=640)` enters the table AND the snapshot as an untracked
   key (the T0c D5 prefix class is handled for SLOTTING but not snapshot
   hygiene), and a raw float `4364.299999999999` prints where `:.1f`
   belongs. A direct paste would carry both into the baseline. Fix at
   the next `tools/` touch (or sanitize manually at PR-2).
4. **Dispatch erratum (no action)** - the expected census split "+5 =
   m27 un-ignore + 4 T4 tests" is wrong; actual `+5 = 5 new
   payload_reshape_tests` with m27's +1 already baked into the anchor.
   Journal arithmetic correct; recorded so no future reconciliation
   re-doubts the anchor.
5. **Purge queue (row-67)** - reviewer scratch `tasks/t4-review-probe.py`
   joins `tasks/t0b-review-probe.py`, `tasks/t0c-review-probe.py`,
   `tasks/t4b-review-probe.py`.

## 10. Operator-chain status as this verdict leaves it

- T4 lands APPROVED: `fd7c1ec` stands. Memory-law acceptance is
  MEASURED on the ratified law, both profiles: release 4379.7 (5x
  independent corroboration + directional re-run), debug 4364.3 (live
  gate table). The 4098 transient is dead (2.0 MiB at full scale, 0.0
  deltas at half scale).
- STILL OPEN (terminal milestone tail): PR-2 baseline commit (the
  2026-10-07 update exists on disk, uncommitted - commit/decide under
  VERIFIED-IDLE), the memory-gate baseline paste (snapshot ready;
  finding 3 caveats apply), the FORMAL `--bench stage5-memory` gate run
  (first post-baseline run is the real verdict; TRIPWIRE for G expected,
  never gate-failing), the **Corpora field boot (G-H terminal
  acceptance)**, the journal erratum/hash commit (finding 1), ledger +
  memory sync at closeout (anchor `1255/0/13/91`), walkthrough.
- Deferred rows stay open and were NOT absorbed by this hop: M28 T2 D1
  LoadMemo seed-extraction, I7c tripwire, testrun _echo cp1252, row-72
  allowlist/pin (this commit touched no manifests - Cargo.lock diff-EMPTY
  confirmed).

## Scratch-witness list (this review; purge queue)

- `tasks/t4-review-probe.py` (independent curve recompute; the final
  two debug-print lines raise a cosmetic NameError AFTER all assertions
  print - results are in `target/rv-t4-independent-recompute.txt`)
- reviewer captures: `target/rv-t4-rerun640.txt`,
  `target/rv-t4-medians-rerun.txt`, `target/rv-t4-independent-recompute.txt`,
  `target/rv-t4-memory-hint.txt`
