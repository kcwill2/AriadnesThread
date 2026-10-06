# M28 T2 journal - stage-5 lifecycle (single-copy take/restore, pre-stage-5
index release, bounded passes, boot cap wiring)

Hop id `t2-stage-lifecycle`. Handoff:
`tasks/stage5-collateral-memory-ceiling-t2-handoff.md`.
Session date: 2026-10-06 (UTC). Predecessor: T1 landed `d6640a2` (HEAD at
dispatch - verified via `python tools/distill.py -- git rev-parse HEAD`).

## Preconditions verified at dispatch (by this implementer)

- Precondition 1 SATISFIED: T0a approved, DP-4 evidence review concluded
  (`tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` sec. RATIFICATION
  + ROUND-2 ERRATUM, landings `39da2e9`/`afcd038`). T2 dispatch relay read
  from the board.
- Precondition 2 SATISFIED: HEAD is exactly `d6640a2` (T1 landing). Cap
  literals sourced from THAT commit's landed code
  (`crates/graph/src/load_caches.rs`: `LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP`
  = 512, `LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP` = 128) plus its journal
  justification (`tasks/stage5-collateral-memory-ceiling-t1-journal.md`
  "Hop-fixed cap literals" - hop-fixed, NOT review-ratified, reviewer
  adjudicates). NOT sourced from the evidence doc.
- Precondition 3: non-cap work file-disjoint from T1 (already landed anyway).
- Precondition 4: zero fixture demand - all new tests use in-test temp dirs.
- Working tree: pre-existing orchestrator-side dirty/untracked files
  (ledger drafts, handoff/verdict docs) outside the touch-set, untouched.

## Verified code facts at dispatch

- `ensure_load` 92 lines (92/100 ceiling) - extraction mandatory.
- Boot order `ui.rs::run_pipeline_stages`: include -> index -> reduced ->
  CALLGRAPH (line 150) -> load (163), then `take_include_graph` (250).
  Consequence: the boot path never calls `ensure_index` after `ensure_load`,
  so the (b) release costs boot ZERO reloads (zero G-D timing impact); the
  reload branch is hit only by consumers that ask for the index after the
  load stage (proven by the new RunStats test).
- `pipeline_test.rs::test_cold_then_warm_all_stages` runs callgraph before
  load on both cold and warm pipelines, then asserts stage_reuse records -
  unaffected by the release (release does not record; reload only happens if
  a consumer re-runs `ensure_index`).
- `payload_tail_bench` / `stage5_memory_bench` drive index -> callgraph ->
  load, so the stage-5 timing window excludes any post-load reload.
- `LoadMemo` API: `with_closures_and_capacity` TAKES the closures table by
  value (OnceLock seed); no extraction API exists, and `crates/graph/**` is
  T1-locked for this hop. See DEVIATIONS D1 for the closures-copy consequence.

## DEVIATIONS (declared up front, per handoff discipline)

- D1 (structural, (a)-depth): the single-copy take/restore lands as a full
  MOVE for the graph and the reduced model (their A4 deep clones die), and
  as exactly ONE retained deep copy for the closures table - the memo seed.
  Reason: `LoadMemo::with_closures_and_capacity` consumes the seed table by
  value and the memo cannot hand it back (no extraction API; graph frozen
  by this hop), while `include_cache` MUST hold the full pair again when
  `ensure_load` returns - boot's `take_include_graph` after stage 5
  (`ui.rs:250`, MUST-NOT-TOUCH, green UNEDITED) and
  `internal_tests.rs:118` consume it. The two alternatives are rejected:
  (i) restore by `compute_closures()` recompute = re-pays stage-1 BFS CPU
  inside/after stage 5 (G-D timing + boot wall regression at corpus scale);
  (ii) end-of-pass clone out of the memo = the identical 2-copy peak, moved
  to the save span (the worst place - A7). Net residency during the file
  pass: graph x1 (today x2), reduced x1 (today x2), closures x2 (today x2 -
  zero regression; the memo's seed is the second copy, the original waits
  for restore), index result x0 (today x1), memo caches capped (today
  uncapped). The machine assertion (seam empties the caches - below) pins
  the move semantics of the seam.
- D2 ((c) has no unit red): per the handoff's own clause - at 5-file/520
  file fixture scale the superlinearity is invisible (E6), so the chunk
  bound has no honest machine-checkable pre-state claim at scoped scale;
  the discrimination run is T0b's G-C gate on the pre-fix tree. Journaling
  the reasoning instead of faking a green-able red, as the handoff directs.
  The chunk width is machine-pinned by the I6 literal + the report-equality
  oracles staying green UNEDITED (chunking must not move one element or one
  event).
- D3 (chunk width 512, one shared constant for both passes): power-of-two,
  >= 16x a typical logical-core count (the scoped/global pool stays
  saturated per chunk), <= ~9% of the failing corpus file count (5753) so
  per-chunk collect intermediates stay sub-dominant. Results equality vs the
  unchunked pass is locked by the sequential-reference oracles (green
  UNEDITED); event equality by the I6 literal.
- D4 ((b) release applies at BOTH stage-5 entry branches): the cold rebuild
  and the on-disk-load return both "run stage 5" (the A5 residency exists
  on both; warm boots are the user's repeat path). Warm-path `ensure_index`
  reload behavior is green (new warm test + RunStats test).
- D5 (fixture duplication, not shared helpers): the new test files re-author
  the temp-dir fixture (same tree as `internal_tests.rs::setup_repo`)
  instead of editing internal_tests visibility - keeps the equivalence
  oracle UNEDITED (the preferred 3.3 outcome); in-code replication is the
  T1-D1 / `payload_tail_bench.rs` sanctioned pattern.
- D6 (`--lang` edge): a filtered run has no persisted full-index artifact,
  so post-release `ensure_index`/`ensure_callgraph` rebuild through the
  shard store (TU-level reuse intact) - correct results, CPU-only on the
  filtered-view edge; documented at the seam.
- D7 (4096 save-span transient - mechanism + witness honesty): the
  in-T2 mechanism is residency removed from the save span: the analyzer and
  its memo (seeded closures table + capped boot caches) drop BEFORE the
  artifact save, the index result is released pre-pass, and the
  graph/reduced stack copies no longer double the caches. The serialization
  transient itself (A7, `crates/report/**`) is out of this touch-set (T4/
  DP-2 territory) and stays untouched. No scoped-scale RSS witness is
  honest (E6: superlinearity invisible below scale); the measured before/
  after is the operator/reviewer `--bench stage5-memory` local gate (DP-3)
  with the RATIFIED median protocol. The in-scope machine witness is the
  structural one: stats-proven index non-residency + empty-cache seam hand-
  out + I6/oracle equality.
- D8 (single scoped commit + capture pair as Red proof): same as the T1-D5
  precedent - `commit_scoped` commits whole files, so the Red state is
  evidenced by the R1/R2/R3 capture lines over an identical frozen test
  file, not by an intermediate commit.

## Expected RED messages (handoff RGB step 1 requirement)

- (a) seam test `test_take_graph_and_reduced_for_load_empties_stage_caches`
  (in-crate, on the clone-semantics stub seam):
  "single-copy stage-5 lifecycle violated: the stage caches still hold the
  include graph / closures / reduced model while the hand-out exists - the
  A4 deep-clone path at stage_load.rs:43-44 keeps a second live copy
  resident through stage 5".
- (b) `test_cold_ensure_load_releases_index_result` (in-crate):
  "index result still resident through stage 5".
- (b-warm) `test_warm_ensure_load_releases_index_result` (in-crate):
  "warm ensure_load must release the index result too" - red because today
  the loaded-from-disk IndexResult stays resident.
- (b-public) `test_cold_stage_five_releases_index_and_reloads_fresh`
  (integration): stats still record the stage-2 Rebuilt because the later
  `ensure_index` hit the resident in-memory cache instead of the
  fingerprint-fresh reload: "index result still resident through stage 5".
- I6 recording test: NOT a red - a characterization FLOOR. R1 is the
  capture run (placeholder literal `vec![]` deliberately mismatching so the
  actual stream is printed into the capture); the literal is then pasted
  from `target/testrun-capture.txt` and the test goes green on the SAME
  unchanged production tree in R2. Floor status: green-first.
- per-file event-count test: FLOOR (characterizes today's contract - one
  `CollateralFile` per file + terminal `done == total`), green-first.

## RGB log

(append per step)

### R1 - Red slice on the pre-fix tree (capture + reds)

`python tools/testrun.py --distill -p ariadne-pipeline -- --no-fail-fast`
-> `27 passed / 5 failed / 4 ignored / 7 suites` (capture
`target/testrun-capture.txt`). Two earlier invocations of the same form
first hit (i) a test-only compile error (`ProgressEvent` has no
`PartialEq` - the event enum is read-only material for this hop, so the
I6 lock compares frames as Debug strings instead of adding a derive), then
cargo fail-fast (`13 passed / 3 failed / 0 ignored / 1 suites`) which
suppressed the integration binaries; `--no-fail-fast` is the RED-capture
form (the final gate runs the handoff's exact command, no extra flags).
Failures (each for the predicted semantic reason, per handoff RGB step 1):
- `stage_load_tests::test_take_graph_and_reduced_for_load_empties_stage_caches`
  - "single-copy stage-5 lifecycle violated: include_cache still holds the
  graph/closures while the stage-5 hand-out exists - the A4 deep-clone
  path ... keeps a second live copy resident through stage 5" (names the
  clone path, as the handoff requires; the seam stub carries today's clone
  semantics verbatim - T1's compile-green-slice pattern).
- `stage_load_tests::test_cold_ensure_load_releases_index_result` -
  "index result still resident through stage 5".
- `stage_load_tests::test_warm_ensure_load_releases_index_result` -
  warm-branch analogue.
- `test_cold_stage_five_releases_index_and_reloads_fresh` -
  `left: Some(Rebuilt)`: the post-load `ensure_index` hit the resident
  in-memory cache instead of the fingerprint-fresh reload.
- `test_stage_five_event_stream_matches_recorded_literal` - CAPTURE RUN
  only (placeholder literal, the handoff's recording ritual): the failure
  line printed the pre-fix stream verbatim; the 8-frame literal was pasted
  from this capture into the test file.

Floors already green on the pre-fix tree: `test_exactly_one_collateral_
event_per_file_plus_terminal_frame`,
`test_cold_ensure_load_restores_stage_inputs`.

### R2 - I6 floor green on the UNCHANGED production tree

Same command -> `28 passed / 4 failed / 4 ignored / 7 suites`: the I6
literal now matches on the pre-fix tree (floor landed green-first,
journal-recorded), the 4 semantic reds stand. The recorded literal (from
R1, stable at R2 and at post-change R3 - three agreeing observations):
`LoadMilestone(SymbolsAnalyzed { symbols: 5 })`, five per-file
`CollateralFile { done: 0..4, total: 5 }` in order, terminal
`CollateralFile { done: 5, total: 5 }`, `LoadMilestone(ArtifactsSaved)`.

### R3 - Green (all behavior landed)

Gate 1, handoff exact command `python tools/testrun.py --distill
-p ariadne-pipeline` -> `32 passed / 0 failed / 4 ignored / 7 suites`.
Baseline `25 / 0 / 4 / 6` (post-T1a truth from the handoff) + 7 new tests
+ 1 new suite, every pre-existing test UNEDITED and green. The I6 literal
is byte-equal AFTER the take/restore + chunking + release change (acceptance
line); the sequential-reference report-equality oracles, the
`collect_file_metrics` pin, and the warm-reuse contract are green UNEDITED.
(One intermediate invocation hit a compile error:
`save_load_artifacts_with_languages` returns `Result<PathBuf>`, the
extracted save helper discards it with `.map(|_artifact_root| ())` - the
pre-existing call site already discarded it.)

Gate 2 `python tools/testrun.py --distill -p ariadne-pipeline -- internal`
-> `12 passed / 0 failed / 0 ignored / 6 suites` - all 12 internal_tests
(report equality x2, warm reuse, Arc-sharing, take_include_graph x3,
milestone order, `collect_file_metrics` pin, missing-cache, hash helper)
green UNEDITED - the equivalence oracle stands on the new lifecycle with
zero edits; no 3.3 budget was spent (internal_tests.rs is UNTOUCHED).

### Gates

- Gate 1 (above): `32 passed / 0 failed / 4 ignored / 7 suites`.
- Gate 2 (above): `12 passed / 0 failed / 0 ignored / 6 suites`.
- Gate 3 `python tools/testrun.py --distill -p ariadne-pipeline`
  replacement witness `python tools/testrun.py --distill -p ariadne-cli`
  -> `111 passed / 0 failed / 8 ignored / 6 suites` - boot contract,
  13-frame replay, and every cli test green UNEDITED (no cli file appears
  in this hop's diff); equals the post-T1a truth quoted by the T0a
  verdict session.
- m00 parity `python tools/testrun.py --distill
  -p ariadne-integration-tests --test m00_golden_baseline_test`
  -> `2 passed / 0 failed / 0 ignored / 1 suites` (frozen row: 2 passed,
  UNEDITED).
- m18 parity `python tools/testrun.py --distill
  -p ariadne-integration-tests --test m18_cpp_plugin_parity_test`
  -> `23 passed / 0 failed / 0 ignored / 1 suites` (frozen row exact,
  UNEDITED).
- Gate 4 `python tools/hygiene_check.py` - exit 0 (verified independently
  via `python tools/distill.py -- python tools/hygiene_check.py` ->
  `generic PASS (exit 0)`; R1/R2 clean; zero rows on touched paths - only
  pre-existing warn-only R3 rows elsewhere).
- Gate 5 `python tools/encoding_guard.py --staged` - result quoted in the
  commit record below.
- `ensure_load` final line count: 94 lines (61..154 of stage_load.rs) -
  under the 100 ceiling, with `try_reuse_persisted_load`,
  `save_load_report_artifacts`, `build_stage5_analyzer`,
  `analyze_symbol_chunk`, `analyze_file_chunk` extracted.
  `stage_load.rs` final 363 lines (budget <= 400); `lib.rs` 714 (margin
  > 280).

### What was implemented (Green step, (a)-(e))

- (a) `Pipeline::take_graph_and_reduced_for_load` + `restore_stage5_inputs`
  (lib.rs): the graph and reduced model MOVE through the stage-5 span
  (A4 clones die); closures keeps ONE retained deep copy as the memo seed
  (deviation D1, memo consumes its seed and cannot return it).
- Cap wiring at `build_stage5_analyzer`:
  `LoadMemo::with_closures_and_capacity(closures.clone(),
  LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP, PHYSICAL_EXPANSION_BOOT_CAP)`
  through the existing `LoadAnalyzer::with_memo` seam - literals cited to
  T1 landing `d6640a2` at the call site (hop-fixed, not review-ratified);
  NOT from the evidence doc.
- (b) `Pipeline::release_index_result_for_load` called on both stage-5
  entry branches (D4) before the passes/save.
- Save-span attack (the ~4096 MiB rider): the analyzer - sole owner of the
  memo with its seeded closures table and capped caches - now
  `drop`s BEFORE `save_load_artifacts_with_languages`, and the index
  result is no longer resident under the serialization transient. The
  transient itself lives in `crates/report` (T4/DP-2 scope, untouched
  here). At scoped scale no honest RSS witness exists (E6); the measured
  before/after is the T0b/operator `--bench stage5-memory` local gate with
  the RATIFIED median-of->=5 protocol. The in-scope witnesses are
  structural: stats-proven reload, empty-cache seam hand-out, I6/oracle
  equality.
- (c) `LOAD_PASS_CHUNK = 512` (D3): both passes collect per chunk; chunk
  loop sequential + order-preserving `collect` => output and event stream
  identical to the single full collect (locked UNEDITED + literal).
- (d) `collect_file_metrics` signature KEPT - the pin test is green
  UNEDITED; the 3.3 budget was never spent.
- (e) all helpers + `ensure_load` < 100 lines (printed above).

### Invocation log (sizing-law accounting)

Testrun/distiller invocations this session: 2 git reads, R1 compile-error
DRIFT, R1 fail-fast red, R1b capture (no-fail-fast), R2 floor, R3
compile-error DRIFT, R3 gate-1 green, gate-2, gate-3, m00, m18, hygiene
direct, hygiene exit-code verify = 14 total. Above the <=10 guidance by 4:
two self-inflicted compile drifts (each caught and fixed by its own
capture, zero behavior risk), the cargo fail-fast capture iteration, and
the independent hygiene exit-code verification. Produced diff ~850 lines
remains inside the sizing law; recorded per "journal, don't hide"
(departure D9).

### Commit record

(single scoped commit, T1-D5 precedent - the Red proof is the R1/R2/R3
capture pair over the identical frozen test files, D8)

