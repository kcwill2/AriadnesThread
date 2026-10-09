# Verdict - M28 T2 `t2-stage-lifecycle` (stage-5 single-copy lifecycle, pre-stage-5 index release, bounded passes, boot cap-wiring)

Commits under review: `e52a9e9` (fix) and `69138db` (journal proof);
start rev = `d6640a2` (T1 landing, = dispatch HEAD as precondition 2
requires). Reviewer acceptance pass run fresh at HEAD `69138db`
(main, ahead=0, behind=0, dirty=23 foreign paths per checkpoint_guard)
on 2026-10-06. Execution-results addendum:
`tasks/stage5-collateral-memory-ceiling-execution-results.md`
("T2 reviewer acceptance pass" section).

## VERDICT: APPROVED

0 blockers. 1 relay finding (timing adjudication to T0b), 1 accepted
interim deviation with a mandated future-touch follow-up, 2 journal
errata, 1 witness-strength note. D1-D9 adjudicated below; hop-fixed
caps 512/128 consumed exactly as T1's verdict relayed them to this hop.

## 1. Commit-vs-touch-set - PASS, exact

`git diff --name-only d6640a2..HEAD` = exactly five paths:
`crates/pipeline/src/lib.rs` (+66/-0), `crates/pipeline/src/stage_load.rs`
(+225/-72), `crates/pipeline/src/stage_load_tests.rs` (+163/-0, NEW),
`crates/pipeline/tests/load_stage_lifecycle_test.rs` (+203/-0, NEW),
the journal (+317 across both commits). The NEW in-crate test file is
inside the touch-set by the handoff's own clause ("src internal_tests.rs
adjacency if in-crate visibility requires it") and needed in-crate
visibility (private `include_cache`/`index_cache` fields) - the journal
states this and names it as an explicit `commit_scoped --scope`.
Per-commit, oldest-first: `e52a9e9` = the five paths (journal +282);
`69138db` = journal only (+35). Zero diff in range for `internal_tests.rs`,
`stage_index.rs`, the whole progress family, `crates/cli/**` incl. `ui.rs`,
`crates/graph/**`, `crates/report/**`, `tests/**` incl. fixtures/goldens,
`tools/**`, `cache.rs`, `bench_baseline.json`, the ledger, every
`*.toml`/`*.lock`. Journal claims verified: `load_caches.rs` blob identity
across the range (`9519a44` == `9519a44`).

## 2. Lifecycle correctness (central duty) - VERIFIED from committed code

(a) Sole memo owner dropped before save: `build_stage5_analyzer`
constructs `LoadMemo::with_closures_and_capacity(closures.clone(),
SEMANTIC_CLOSURE_BOOT_CAP, PHYSICAL_EXPANSION_BOOT_CAP)` through
`LoadAnalyzer::with_memo` (`load.rs:201`) - the Arc has exactly one
owner. `drop(analyzer)` at `stage_load.rs:138` precedes
`restore_stage5_inputs` (:142) and `save_load_report_artifacts` (:149).
Cap literals cited to `d6640a2` at the call site, labeled hop-fixed;
verified present in the landed graph code (`load_caches.rs:117` = 512,
`:125` = 128) - not invented, not from the evidence doc.
(b) `IndexResult` released pre-pass: `release_index_result_for_load`
(`lib.rs:536`) called at `stage_load.rs:80` (cold, after the input
ensures, before take/passes/save) AND `:67` (warm branch, D4).
(c) graph/reduced deep clones dead: `take_graph_and_reduced_for_load`
`take()`s both caches (move); the analyzer only borrows `&graph`/
`&reduced`; the two `cached_*()` clone calls at old :43-44 are gone
(diff-verified). Restored by move before return - boot's
`take_include_graph` (`ui.rs:250`, diff-EMPTY) gets its pair:
witnessed live by `ui_smoke: PASS` and by the 3 take_include_graph
internal tests green UNEDITED inside gate 2's `12 passed`.
(d) Warm-reuse semantics behavior-preserving for ALL consumers:
`ensure_reduced` (:544-547) and `ensure_callgraph` (:579-582)
early-return on their own caches BEFORE their internal `ensure_index`,
so post-release consumers pay zero reload unless a genuine rebuild is
needed; `ui.rs` has no index consumer after `ensure_load` (grep of the
diff-EMPTY file), and callgraph precedes load in boot (:150 before
:163) - boot pays ZERO reloads as the journal claims. The
fingerprint-fresh disk reload is locked by the new public RunStats
witness and covered for equality by the report-equality + warm-reuse
oracles, green UNEDITED: gate 2 `12 passed / 0 failed / 0 ignored / 6
suites`, `internal_tests.rs` absent from every diff (3.3 budget
unspent).
Extraction hygiene: `ensure_load` = 94 lines (61..154, counted fresh);
all five extracted helpers well under 100 (largest ~37);
`collect_file_metrics` signature UNCHANGED (pin green UNEDITED);
`stage_load.rs` 363/400 budget; verbatim-move discipline holds in the
helper bodies (`try_reuse_persisted_load` keeps the pre-existing
`ok()`-swallow rebuild-on-doubt; `save_load_report_artifacts` keeps
the discarded-PathBuf contract).

## 3. RGB truth - ACCEPTED

Pre-fix Red states are capture-evidenced through the T1-D5 precedent
(D8): totals internally consistent - R1 `27 passed / 5 failed / 4
ignored / 7 suites` (4 semantic reds + the I6 capture run, each failing
for the handoff-predicted message quoted in the journal), R2 `28/4`
after the literal paste (floor green-first on the UNCHANGED production
tree), R3 `32/0` post-change; total test count 36 in all three. I6
literal honesty: the 8 frames match the fixture's shape (5 symbols,
5 files) and the stage-5 emission order the 13-frame replay pins
(`progress_contract_tests.rs:148-154`, diff-EMPTY, green inside the
cli `111 passed`); the notify call sites are byte-identical in the
diff (comment reflow only), and chunk width 512 > fixture size keeps
the pass structure identical - three agreeing observations
(R1/R2/R3), byte-equal post-change (acceptance line satisfied).

## 4. Deviation dispositions (independent adjudication)

- D1 (closures keep ONE memo-seed copy, x2 during pass) - ACCEPTED as
  interim; follow-up mandated (finding 2). Merit: verified zero
  regression - the pre-fix path ALSO held two closure copies during the
  pass (cache original + the :43 clone feeding `with_closures`), so the
  net is graph x2->x1, reduced x2->x1, closures x2->x2 with the second
  copy now DROPPED before the save span instead of resident through it.
  The mechanism claim is code-true: `with_closures_and_capacity`
  consumes the seed (`OnceLock::from(closures)`, no extraction API) and
  graph is T1-locked for this hop. Rejected alternatives are sound:
  (i) recompute re-pays stage-1 BFS CPU (G-D/boot wall), (ii) end-of-
  pass clone relocates the identical peak to the save span (worst base
  for the A7 transient).
- D2 (no unit red for (c)) - ACCEPTED per the handoff's own E6 clause;
  G-C discrimination is T0b's gate job.
- D3 (chunk 512, shared constant) - ACCEPTED; equality locked by the
  UNEDITED sequential-reference oracles + the I6 literal.
- D4 (release on BOTH stage-5 branches) - ACCEPTED; verified in code
  (:67, :80); warm-reuse oracle green UNEDITED.
- D5 (fixture duplication over visibility edit) - ACCEPTED; the
  sanctioned replication pattern; keeps the equivalence oracle at its
  preferred UNEDITED state.
- D6 (`--lang` filtered-view shard-rebuild edge) - ACCEPTED; CPU-only,
  documented at the seam.
- D7 (residency removed from save span, transient itself left to
  T4/DP-2) - ACCEPTED and independently witnessed: drop-before-save
  verified in code; my fresh N=640 release witness shows save-tail RSS
  delta 0.5 MiB while the serialization transient survives
  (peak-minus-save-tail 1024.0 vs pre-fix 1023.9 at N=640) - exactly
  the "residency base removed, transient intact" claim.
- D8 (single commit; Red proof = R1/R2/R3 capture pair) - ACCEPTED per
  the T1-D5 precedent.
- D9 (14 gate invocations vs <=10) - ACCEPTED (finding 3): the sizing
  law's real axis is gates x output volume; diff ~850 < 1200, all
  captures distilled, the overage is 2 self-caught compile drifts +
  1 fail-fast capture + 1 independent exit-code verification, fully
  itemized in the journal per the "journal, don't hide" clause.

## 5. Fresh gate lines (verbatim, run by this reviewer)

- Pipeline scoped: `32 passed / 0 failed / 4 ignored / 7 suites`
- Internal oracles (gate 2): `12 passed / 0 failed / 0 ignored / 6 suites`
- Cli (boot contract + 13-frame replay + ui.rs:250 witness):
  `111 passed / 0 failed / 8 ignored / 6 suites`
- m00: `2 passed / 0 failed / 0 ignored / 1 suites` (UNEDITED)
- m18: `23 passed / 0 failed / 0 ignored / 1 suites` (UNEDITED)
- Graph plain / instrumented (T1 suite UNEDITED green):
  `104 passed / 0 failed / 0 ignored / 4 suites` /
  `107 passed / 0 failed / 0 ignored / 4 suites`
- Census (`--distill` substitute, row-59 errata):
  `1247 passed / 0 failed / 13 ignored / 91 suites`
- Tools CI-form: `Ran 194 tests in 20.878s` / `OK`
- Smoke: `boot+route checks clean (cold fixture, attempt 1, 1.0s)` /
  `ui_smoke: PASS`
- Hygiene exit 0 (zero rows on any touched path; warn-only R3 foreign
  rows only). Encoding `--all` exit 0, silent.
- Frozen locks: `progress.rs`, `cache.rs`, `bench_baseline.json`,
  goldens/fixtures, ledger, all manifests diff-EMPTY across the range.

## 6. Census reconciliation - EXACT

T1-review `1240 passed / 0 failed / 13 ignored / 90 suites` -> this run
`1247 passed / 0 failed / 13 ignored / 91 suites`: +7 passed = the 4
in-crate `stage_load_tests` + 3 integration lifecycle tests, +1 suite =
the new test binary, ignored unchanged. Anchor `1235/0/6/88 @ bd4fb1a`
+ T1 (+4/+1, +7 ignored) + T2 (+7/+1) all accounted for.

## 7. bench_compare - RUN (pipeline/src changed; exemption void) - FAIL, stale-baseline caveat (finding 1)

`### generic FAIL (exit 1)`; medians (median-of-3): `symbol_pass+prep`
1653.7 (baseline 1276.4, +30% WARN), `file_pass` 643.5 (baseline 312.5,
+106% WARN), `save_load_artifacts+cache` 14457.6, `ensure_load total`
16760.4. The T1 reviewer's framing applies verbatim: the baseline is
stale (ledger escalation row: same-tree medians swinging 1667->2016 in
a day; PR-2 `--update-baseline` on VERIFIED-IDLE remains outstanding)
and "a stale-baseline FAIL is the known flapping gate, not this hop"
(handoff, G-D section; row-61). BUT the +106% `file_pass` is NEW since
the same-day T1 run (349.1), and it is structurally attributable: T2 is
the hop that WIRED the caps, and the capped memo trades time for bytes
by design (T1 I5) - demand at bench scale (640-1280 physical-expansion
seeds vs cap 128) means most queries recompute. Corroborated by my N=640
release RSS witness: `ensure_load wall` 2.5 s vs the pre-fix release
table's 1.6-1.7 s, while boot stages 1-4 stayed at 1.5-1.6 s (zero
reload cost for boot, as claimed). Not a blocker here; finding 1 relays
the timing adjudication to T0b on a refreshed baseline.

## 8. RSS directional witness (one invocation, release N=640) - RUN

`STAGE5_MEM_BENCH_FILES=640 --exact stage5_rss_scale_runner` (distill
route, exit 0): `stage5 peak RSS (release, N=640) | 2090.2` - BELOW the
pre-fix release band 2254.1-2258.2 (T0a) and below the T1 tree's 2258.0
same-scale reading: **-168 MiB (~-7.4%)**. Checkpoints: symbol-pass end
854.9 (pre-fix 1026.3), file-pass tail 1065.7 (1233.2), save tail
1066.2 (1234.1), post-return 1066.2 (1144.8). The memo `entries` rows
printed 2464/640 are DEMAND counts (`report.symbol_metrics.len()`,
bench :566), not cache lengths - no cap violation; lengths are bound by
the admission invariant pinned by T1's I7c suite (107 green). The N=640
serialization transient survives (1024.0 vs pre-fix 1023.9) - by design
(D7). N=1280 NOT attempted (the machine still cannot hold the artifact
save - T1 recorded os error 112 twice; dispatch authorizes one 640-scale
invocation only; nothing outside temp touched). Single-invocation,
machine not VERIFIED-IDLE: DIRECTIONAL ONLY per the RATIFIED
median-of->=5 protocol. Full post-fix N=1280 curve = T0b/operator duty.

## 9. Findings

1. (minor, RELAY to T0b) G-D timing adjudication deferred: `file_pass`
   643.5 ms = 2.06x the stale baseline and ~1.84x the same-day T1 run
   (349.1), consistent with cap-driven recomputation at bench scale
   (also `ensure_load` wall 1.6-1.7 s -> 2.5 s at N=640 release). After
   the operator's PR-2 `--update-baseline` on VERIFIED-IDLE, T0b must
   re-judge the stage-5 timing columns; if `file_pass` still sits past
   1.25x on the fresh baseline, adjudicate the
   `PHYSICAL_EXPANSION_BOOT_CAP=128` economics (demand ~10x the cap)
   against the boot-time budget. Not a T2 blocker: the handoff declares
   the stale-baseline FAIL a known flapping gate, and I7b/RATIFIED
   protocol makes single runs directional only.
2. (accepted interim, follow-up REQUIRED) D1's retained closures copy:
   a future graph-touching hop must add a seed-extraction API to
   `LoadMemo` (`take_closures`/`into_parts` class) and drop the last
   deep copy so closures reach x1 through the stage-5 span. Disposition:
   `tasks/milestone_ledger.md` deferred-findings (relay to orchestrator;
   candidate T4/DP-2 territory since that hop touches report and can
   pick up the graph seam serially).
3. (accepted) D9 over-bound gate invocations - 14 vs <=10, itemized,
   zero behavior risk, law's volume axis satisfied (~850 < 1200 diff
   lines). No action beyond this adjudication.
4. (minor, journal errata) Journal states `lib.rs` 714 lines (margin
   > 280); the committed file is 700 lines (margin 300). Both satisfy
   the 1000 ceiling; number should be corrected at next journal touch.
   Also gate-3's line initially pastes the pipeline command before its
   own replacement form - self-corrected in the same paragraph.
5. (minor, witness strength) `test_warm_ensure_load_releases_index_result`
   cannot distinguish the warm branch from a fall-through cold rebuild
   (both release post-fix, both resident pre-fix), so its red/green says
   nothing specifically about the warm branch. The D4 warm-branch claim
   stands instead on direct code inspection (`stage_load.rs:67`), the
   UNEDITED warm-reuse oracle, and the green public reload witness. No
   change required; recorded so the coverage is not over-claimed.

## 10. Report-evidence check (distillation contract) - PASS

Journal is digest-shaped throughout: verdict/digest lines, canonical
`testrun` lines, exit witnesses, capture paths; all gates via distilling
entry points; zero raw pastes. Red messages quoted per handoff RGB step-1
requirement (the (a) red names the :43-44 clone path as mandated).

## 11. What this hop means for the user's Corpora boot

Post-T1+T2 the stage-5 boot peak loses the second graph copy, the
second reduced copy, the whole `IndexResult`, and uncapped memo growth
(512/128 admission caps), with the memo's seeded closures table dropped
BEFORE the artifact save. Measured directionally at N=640 release:
peak -168 MiB (-7.4%), resident-after-return -79 MiB, boot stages 1-4
untimed-identical (zero reloads), at a small stage-5 CPU cost from
cap recomputation (~+0.9 s at N=640, release). At N=1280 the
constant ~4096 MiB save-span transient (A7, `crates/report` - T4/DP-2)
still dominates the peak: expect the post-T2 N=1280 peak around
~8.6-8.7 GiB - residency relief real but the ratified 5200 MiB bar NOT
yet closable until the transient itself is fixed. For the real Corpora
(~5753 files) the residency removed scales up ~4.5x and matters
proportionally more, but the same conclusion holds: bar closure awaits
T4/DP-2; the numbers are T0b's to measure under the RATIFIED
median-of->=5 protocol on an idle machine with disk freed.

## Follow-up dispositions (relay to the orchestrator, not dropped)

- Finding 1 -> T0b acceptance criteria (stage-5 timing re-judge on the
  PR-2-refreshed baseline; cap-economics adjudication if >1.25x).
- Finding 2 -> ledger deferred-findings: `LoadMemo` seed-extraction API,
  then closures x1 (couples naturally with T4/DP-2).
- Finding 4 -> journal errata at next touch.
- Operator duties unchanged from T1: free disk before any N=1280 run;
  `--update-baseline` on VERIFIED-IDLE (G-D becomes binding post-PR-2).
