# Journal - M28 T4 (load-artifact payload reshape + streaming save + epoch 2.2.0)

Hop `t4-payload-reshape`, spec `tasks/stage5-collateral-memory-ceiling-t4-handoff.md`.
Start rev `e9a4ac1` (T0c landing verified, precondition 3). All gates distill-routed.

## D-row: declared gate-invocation ledger (journal-count law, ledger row 73)

D1 (2026-10-07): Declared up FRONT per the handoff's declared-reserve clause:
baseline witness 1 + disk witness 1 + Red 1 + Green 1 + golden-resync 1 +
integration scoped 1 + golden-diff proof 1 + pipeline scoped 1 + graph scoped
1 + full-workspace census probe 1 + preflight release 1 + curve 5-8 + medians
1 + hygiene 1 + encoding 1 + commit 1 = declared up to ~21 worst case (plus
routed support calls). The ratified >=5-accepted protocol floor is
arithmetically incompatible with the ~10 sizing ceiling for a fix+measure hop
(T0c D3/D6 + verdict finding 4 precedent); the declaration is the law, the
overshoot is expected, silence is the violation.

## Gate 0: disk witness (BEFORE any cargo)

- Command: `python tools/distill.py --kind generic -- python
  tasks/t0c_run_invocation.py --disk-witness --min-free-gib 16` - exit 0.
- Witness line verbatim: `DISK C: free 183 GiB of 1862 GiB - PASS`
  (capture: target/distill-capture.txt).

## Precondition 3: HEAD carries the T0c landing

- `python tools/checkpoint_guard.py status`: HEAD `e9a4ac1`, branch main;
  bench labels/caps 512-128/T0b grammar present via T0c verdict sec. 2.

## Design note (representation owned by the implementer per handoff)

The in-memory `LoadReport` struct stays EXACTLY as-is (fields + types): the
`crates/pipeline/src/internal_tests.rs` oracles, `stage_load.rs` and
`analyze.rs` construct/read it unedited (IV6 + touch-set law - I cannot add a
field without breaking a non-touch-set struct literal). The reshape lives in
the persisted form inside `crates/report/src/manager/metrics_io.rs`:
- report.json: `repository_root` + `symbol_summary` (analyzed_symbols count +
  collateral_load sum) + full `file_metrics`, streamed to the unique-tmp file
  (no whole-payload String). The `symbol_metrics` key disappears (DP-2).
- report.bin: `repository_root` + `symbol_summary_rows` (per-symbol scalar
  rows: name, target file, the four load scalars; detail vectors NOT
  persisted) + full `file_metrics`, streamed via bincode serialize_into (no
  whole-payload Vec).
- Load side reconstructs LoadReport with the summary rows as scalar-only
  `symbol_metrics` entries (detail absent by construction) and full
  `file_metrics`. This keeps the UNEDITED floors green: m6:178 reads
  `symbol_metrics.len()` from the loaded artifact (verified loaded-artifact
  path, not a cold handle - `manager.load_load_checked`), so the loaded rows
  must carry the count; internal_tests warm-reuse (:416-462) asserts event
  silence only; the sequential-reference/determinism oracles (:112-188,
  :325-340) run cold and read the in-memory full-detail report.
- `measure.rs` sums the persisted scalar rows: identical by construction;
  cross-checked by the R3 pin in the report crate.

## Baseline + Red captures

- Baseline green witness at start rev: `python tools/testrun.py --distill -p
  ariadne-report` -> `36 passed / 0 failed / 0 ignored / 3 suites` exit 0.
- Gate 1 Red (2nd/3rd invocation of the scoped route; the 1st was a
  compile-error, NOT Red - usize/u64 assert fix, journal-honest):
  `python tools/testrun.py --distill -p ariadne-report` exit 1,
  `37 passed / 3 failed / 0 ignored / 1 suites` (a failing suite truncates
  the later binaries; canonical totals quoted from the GREEN run only,
  row-65 law). Capture: target/testrun-capture.txt (85- then 73-line
  variants). Expected-failure lines (verbatim, all three predicted):
  1. R1 `persisted_json_drops_symbol_detail_and_carries_summary_scalars`:
     `persisted report.json must not carry the never-consumed per-symbol
     detail: found key symbol_metrics` - the key is present today.
  2. R2 `stale_prior_epoch_load_manifest_reaches_rebuild_not_decode_error`:
     `a 2.1.0-era manifest must never be trusted after the 2.2.0 epoch bump:
     LoadReport { ... }` on the expect_err of an Ok - the "2.1.0" blob is
     accepted fresh pre-bump (pre-bump epoch still 2.1.0). The Debug dump
     shows the full per-symbol detail the persisted bin carries today.
  3. Round-trip contract rewrite `manager::tests::
     test_save_and_load_load_artifacts`: `loaded rows must not carry
     semantic_symbols detail (payload reshape)` - the old pin asserted the
     detail SURVIVED; the fixture now seeds non-empty detail vectors so the
     absence asserts are falsifiable.
  Pins green-by-design at Red (journal-label, T0c D2 class):
  `loaded_artifact_round_trips_summary_scalars_and_full_file_detail`
  (scalar round-trip today trivially true),
  `manifest_totals_equal_summary_derived_totals` (IV2 cross-check),
  `load_manifest_json_fields_are_exactly_totals_plus_current_version`.
- Red-phase harness fixes (compile failures are not Red; both honest):
  (a) `files.len()` usize vs `as u64` comparison fixed;
  (b) checked-load tests re-root the shared fixture at the temp manager
  (check_manifest compares repository_root; first Red run failed R2/pins on
  a root-mismatch reason, not the predicted reason).

## Green steps

- G1 metrics_io.rs: persisted payload reshape + streaming writers/readers
  (SymbolSummaryRow / SymbolSummary / PersistedLoadReportJson /
  PersistedLoadReportBin / PersistedLoadReport + write_streaming_atomic in
  the unique-tmp + rename family; manager.rs diff stays EMPTY except the one
  `#[cfg(test)] mod payload_reshape_tests;` registration the handoff admits
  when the new module is created).
- G2 epoch bump landed in-worktree, all five dependents: cache.rs:21 const
  "2.2.0" + lineage doc entry + assert :336; m12:97 pin + lineage; m21:499
  witness; measure.rs consumer comment (m27 stays UNEDITED - no version
  literal exists in it, grep-verified). Same-commit discipline: all five
  land in the ONE landing commit with the payload.
- G3 lock resync (route: green run #1 showed the registered single failure
  `load_artifact_bytes_are_byte_locked` printing the new bytes
  json=0x1d656f6de2a1343f bin=0xffe60b93421f7d14
  manifest=0x1e97c74050b70c58 -> constants updated with lineage comments
  keeping prior values per the :596-600 precedent; green run #2:
  `python tools/testrun.py --distill -p ariadne-report` ->
  **`41 passed / 0 failed / 0 ignored / 3 suites` exit 0** (Gate 2 canonical
  line; baseline 36 + 5 new cases; capture target/testrun-capture.txt).
  Determinism/threading locks (`load_artifact_bytes_independent_of_threading`)
  green UNEDITED = the streaming writers' byte-stability proof post-resync.
- Green-phase harness fix (honest): missing `std::io::Write` import for
  BufWriter::flush (compile error, not a test failure).

## Golden resync (registered exception)

- Resync run (inside the registered exception, scoped per the m00 header):
  `$env:UPDATE_GOLDENS = "1"; python tools/distill.py --timeout 1200
  --capture target/t4-golden-resync.txt -- cargo test -p
  ariadne-integration-tests --test m00_golden_baseline_test` - exit 0
  (`ok (exit 0, 0 warnings emitted)`). Only the two load_report goldens moved.
- Gate 4 golden-scope proof: `python tools/distill.py --kind generic -- git
  diff --name-only -- tests/golden` -> EXACTLY
  `tests/golden/basic/load_report.json` +
  `tests/golden/expensive/load_report.json` (2 lines, capture printed). No
  other golden moved; the normalizer masks `analysis_version`, so the bump
  itself moved zero golden bytes - only the payload reshape did (IV3 holds).
  No reflexive regeneration touched anything else: STOP clause not reached.

## Scoped suites (all distill-routed, GREEN)

- Gate 3 integration: `python tools/testrun.py --distill -p
  ariadne-integration-tests` -> `141 passed / 0 failed / 0 ignored /
  35 suites` exit 0 (m00 with the two resynced goldens; m12/m21 green at
  "2.2.0" with their in-set pin edits; m6/m17/m18/m27 UNEDITED green;
  m6:178 green by the scalar-rows representation - the loaded rows carry the
  count, detail absent).
- IV6 oracle: `python tools/testrun.py --distill -p ariadne-pipeline` ->
  `32 passed / 0 failed / 4 ignored / 7 suites` exit 0 - EXACT prior line,
  internal_tests.rs UNEDITED and green (no STOP, no budgeted edits).
- I5 floor: `python tools/testrun.py --distill -p ariadne-graph` ->
  `104 passed / 0 failed / 0 ignored / 4 suites` exit 0;
  `crates/graph/**` diff-EMPTY (proven by commit scope).

## Five-dependent probe (M27 T2 finding-3 erratum form)

- GREEN-tree full-workspace probe: `python tools/testrun.py --distill` ->
  **`1255 passed / 0 failed / 13 ignored / 91 suites` exit 0** (capture:
  target/testrun-capture.txt). All five 2.2.0 dependents green in-tree:
  cache.rs:21/:336 (index suite), m12:97, m21:499 (both in the 141 integration
  run + the census), report LOCK_MANIFEST_JSON_HASH (report suite). Grep was
  used only as the pre-mutation inventory (5 literal sites found + historical
  lineage comments + no version literal in m27), never as the lock proof.
- Census reconciliation: anchor `1250 passed / 0 failed / 13 ignored /
  91 suites` (T0c verdict sec. 1) -> `1255` = 1250 + 5 new report-crate
  cases (the 5 payload_reshape tests; no new binaries, suites stay 91).
  m27's un-ignored completion test re-verified inside this census
  (`test_m27_nonascii_corpus_cold_index_completes ... ok` - its +1 is baked
  into the anchor per the handoff's bump-procedure site 5, permanent).

## Curve group (release, isolated, per-invocation journal 1-3 lines)

- Gate 5 preflight (t0c gate-4 shape verbatim): `python tools/distill.py
  --timeout 3600 --capture target/t4-preflight.txt -- python
  tasks/t0c_run_invocation.py --files 1280 -- cargo test --release -p
  ariadne-cli --test stage5_memory_bench -- --exact stage5_rss_scale_runner
  --ignored --nocapture --test-threads=1` exit 0; capture FULLY READ.
  Context `STAGE5_BENCH profile=release files=1280 symbols=5024
  file_metrics=1280`; peak row `stage5 peak RSS (release, N=1280) | 4387.4`;
  save tail 4387.4 (delta 0.0), post-return 4387.4.
- Gate 6 curve: `python tools/distill.py --timeout 3600 --capture
  target/t4-postfix-k.txt -- cargo test --release -p ariadne-cli --test
  stage5_memory_bench -- --ignored --nocapture --test-threads=1` (t0c G1
  form verbatim). k=1..5, sequential foreground, `--test-threads=1`, one
  boot per isolated child, nothing else started concurrently (isolation on
  a shared machine); EVERY capture fully read immediately after its run
  (E1 law). Machine: operator win32 workstation; not claimed
  VERIFIED-IDLE (the protocol requires that only for baseline writes; no
  baseline touched).
  - inv1 (t4-postfix-1.txt): peak 4381.6 / 4377.8; half 1063.6 / 1064.1;
    save-tail delta 0.1/0.0; ratio 4.1; wall ensure_load 11.1 s.
  - inv2 (t4-postfix-2.txt): peak 4369.7 / 4370.6; half 1060.6 / 1067.4;
    delta 0.1/0.0; ratio 4.1; 8.8 s.
  - inv3 (t4-postfix-3.txt): peak 4377.7 / 4382.9; half 1062.5 / 1064.5;
    delta 0.1/0.0; ratio 4.1; 11.0 s.
  - inv4 (t4-postfix-4.txt): peak 4390.4 / 4385.1; half 1062.6 / 1064.2;
    delta 0.1/0.0; ratio 4.1; 12.0 s.
  - inv5 (t4-postfix-5.txt): peak 4369.3 / 4383.7; half 1065.6 / 1062.4;
    delta 0.1/0.0; ratio 4.1; 8.4 s.
  Rejections: none (no >8% operand divergence on either slot at either
  scale); never padded, never silent.
- Gate 7 law pass: `python tools/distill.py --kind generic -- python
  tasks/t0c_run_invocation.py --medians target/t4-postfix-1.txt
  target/t4-postfix-2.txt target/t4-postfix-3.txt target/t4-postfix-4.txt
  target/t4-postfix-5.txt` exit 0 (capture target/distill-capture.txt,
  287 lines). Required lines verbatim:
  - `accepted 5 of 5` (>= 5 - the ratified protocol floor).
  - `MEDIAN(accepted) stage5 peak RSS (release, N=1280) | 4379.7` vs the
    ratified bar `<= 5200.0` -> the acceptance reading: 820.3 MiB of slack.
  - `MEDIAN(accepted) stage5 peak RSS (release, N=640) | 1063.8`.
  - `MEDIAN(all) RSS checkpoint save tail (release, N=1280) | 4377.7` ->
    derived save-span transient 4379.7 - 4377.7 = 2.0 MiB (from the ratified
    4098.0: the A7 transient term is dead to checkpoint-granularity noise).
  - `MEDIAN(all) stage5 RSS growth ratio (release, N=640->1280) | 4.1` -
    recorded explicitly as TRIPWIRE (never a gate, user decision
    2026-10-06); no PASS/MISS wording attached, the fixture is Theta(N^2)
    in edges by construction (t0c-verdict sec. 7) and RSS scaling with
    work is not a regression.

## Locks and findings (dated D-rows)

- D2 (2026-10-07) byte-lock resyncs, all THREE registered, each with its
  lineage comment keeping the prior value:
  - `LOCK_REPORT_JSON_HASH` 0x70c9_1d07_9946_f7d9 -> 0x1d65_6f6d_e2a1_343f
    (payload reshape: symbol_metrics array dropped from report.json in favor
    of symbol_summary scalars + full file_metrics, streamed writer).
  - `LOCK_REPORT_BIN_HASH` 0x7a12_1e91_0cbf_2e00 -> 0xffe6_0b93_421f_7d14
    (same payload change on the bin side: scalar summary rows replace full
    LoadMetric rows; streaming writer).
  - `LOCK_MANIFEST_JSON_HASH` 0xbee9_1e36_1bbe_1e4b ->
    0x1e97_c740_50b7_0c58 (version bump only: the manifest embeds
    `analysis_version` "2.1.0" -> "2.2.0"; every other field locked by
    `load_manifest_json_fields_are_exactly_totals_plus_current_version`).
  The determinism/threading locks (`load_artifact_bytes_are_byte_locked`
  sibling `load_artifact_bytes_independent_of_threading`, manager/tests.rs
  UNEDITED) are the post-resync byte-stability proof of the streaming
  writers: consecutive saves identical; 1-thread and 4-thread pools identical.
- D3 (2026-10-07) golden-exception scope: exactly two goldens moved
  (basic+expensive `load_report.json`). All 13 other artifacts x 2 fixtures
  byte-identical - proof: gate 4 diff lists ONLY the two paths. The two
  `load_manifest.json` goldens did NOT move (normalizer masks
  `analysis_version`; totals are unchanged). No STOP clause reached: no
  non-registered golden drift, no reflexive regeneration.
- D4 (2026-10-07) representation + m6 determination (handoff binding-input
  8): `m6_load_analysis_test.rs:178` is a LOADED-ARTIFACT read (verified:
  `manager.load_load_checked` -> `check_manifest` + `report.bin`
  deserialization - not a cold in-memory handle). The representation that
  keeps it green UNEDITED persists per-symbol SCALAR summary rows in
  report.bin (loaded `symbol_metrics` carries the count, detail vectors
  absent by construction) while report.json keeps only the aggregate
  summary scalars (R1 locks that). IV6 outcome: `internal_tests.rs` green
  UNEDITED (32/0/4/7 EXACT) - no STOP, no budgeted edits.
  `stage_load.rs`, `analyze.rs`, `ui.rs`, `crates/pipeline/**`,
  `crates/graph/**`, `crates/cli/tests/**`: diff-EMPTY (proven by commit
  scope at landing).
- D5 (2026-10-07) gate-9 adaptation (raw git denied at permission level;
  `commit_scoped.py` atomically owns `git add` + `commit --only`):
  `encoding_guard.py --staged` is run BEFORE the landing commit (nothing is
  stageable outside commit_scoped, so its exit-0 is recorded as vacuous-
  true, honestly), and `encoding_guard.py --all` (the CI-form superset
  covering the landed bytes in HEAD) is run immediately AFTER the landing
  commit as the substantive byte gate. Both exit 0 or the hop is not done.
- D6 (2026-10-07) final invocation count (closes D1's declaration):
  report-scoped testrun 7 (1 baseline + 2 compile-fix attempts that are NOT
  Red + 1 semantic Red + 1 compile-fix + 1 hash-reveal + 1 green) +
  golden resync 1 + golden-diff proof 1 + integration 1 + pipeline 1 +
  graph 1 + full-workspace census 1 + preflight 1 + curve 5 (zero
  rejections, no 6th-8th needed) + medians 1 + worktree status proof 1 +
  hygiene 1 + encoding staged 1 + commit 1 + encoding --all 1 + landing
  stat 1 = 26 routed invocations vs the ~21 declared. Overshoot sources,
  itemized per the T0c D3/D6 precedent: 2 extra compile-error report runs
  (compile failures are not Red - declared and honest), 1 census probe run
  added by the dispatch prompt (five-dependent green-tree erratum proof +
  the census reconciliation line; the handoff assigns the census to the
  reviewer hop - the reviewer may re-run), 4 landing-support calls. The
  ratified >=5-accepted floor consumed 11 curve-side invocations (preflight
  + 5 + medians + reads are context, not gate count). Silence would have
  been the violation; this row is the itemization.
- D7 (2026-10-07) workflow finding (landing-message defect, self-caught,
  same-tree fix): `commit_scoped.py` parses `-m` as a SINGLE-value argparse
  flag, so repeated `-m` flags silently keep only the LAST paragraph - the
  first landing attempt (d43571a) carried the bump-lineage paragraph alone,
  missing the DP-2 payload and golden-resync justifications the handoff's
  gate 10 requires. Fixed by `git commit --amend --only -m <single message
  with embedded newlines> -- <the same 11 paths>` (tree byte-identical;
  --only keeps shared-worktree foreign staging out, per the tool's own E5
  doctrine). Future multi-paragraph messages MUST embed newlines in ONE
  `-m` argument; a repeated `-m` through commit_scoped is a silent data-loss
   path. Post-amend proof in the verdict: `git diff d43571a HEAD` is empty
   and `git show --stat HEAD` lists the same 11 paths.
- E1 ERRATUM (2026-10-08) correcting D7's letter-claims, append-only (D7's
  text stands unrewritten; T4 verdict finding 1,
  `tasks/stage5-collateral-memory-ceiling-t4-verdict.md`, APPROVED): D7
  claimed "tree byte-identical" and `git diff d43571a HEAD` "is empty" -
  both false at letter. The true delta is
  `git diff d43571a fd7c1ec` = +12 insertions, JOURNAL-ONLY (this very D7
  row folded into the amended commit itself); the ten non-journal paths
  (all code, tests, and goldens) ARE byte-identical - reviewer-proved in
  finding 1: the delta is confined to one touch-set tasks file and
  `git show --stat fd7c1ec` lists the same 11 paths. A record correction,
  not a rework: the message fix stands in substance and no gate is
  re-run by this row.
- R1 (2026-10-08) landing-hash record: the final T4 landing commit is
  `fd7c1ec` "feat(report): load-artifact payload reshape + streaming
  save/load + epoch 2.2.0 (M28 T4 DP-2)", superseding the dangling first
  landing attempt `d43571a` (message-defective, kept readable as the D7
  defect story). The T4 verdict adjudicated `fd7c1ec` APPROVED; this
  erratum + hash record is the follow-up that finding 1 required.

## Explicitly NOT done (scope honesty)

- No `tools/**` touch (`bench_compare*.py` are T4b's; `bench_baseline.json`
  operator PR-2 only), no `tasks/milestone_ledger.md` touch, no ledger row,
  no `crates/cli/tests/**` (bench labels untouched - the 26-label tuple is
  byte-identical), no I5 file touch, no fixtures touch, no `ui.rs`, no
  `stage_load.rs`, no `internal_tests.rs` touch, no `cargo`/`git` outside
  the distilling entry points, no `--update-baseline`, no live
  `--bench stage5-memory` run (pre-T4b G-fail expectation belongs to
  whoever runs it - recorded, not re-litigated).
- The G growth-ratio row reads 4.1 and is TRIPWIRE-only per user decision
  2026-10-06; this hop ran zero verdict wording on it.
- Deferred rows stay open (not absorbed): M28 T2 D1 LoadMemo
  seed-extraction (next graph-touching hop), I7c counter tripwire, testrun
  _echo cp1252 row, row-72 allowlist/pin (this commit touches NO manifest
  files - Cargo.lock is diff-EMPTY and the two Cargo.tomls are not in the
  landing).
