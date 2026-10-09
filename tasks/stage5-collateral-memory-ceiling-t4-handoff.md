# T4 Handoff - M28 stage5-collateral-memory-ceiling: load-artifact payload reshape + streaming save path + epoch bump 2.2.0

Hop id: `t4-payload-reshape`. DISPATCHED 2026-10-07 (user decision 2026-10-06,
second evidence review: the artifact payload term ranks top per the approved
T0c review). This hop eliminates the addressable part of the A7 save-span
transient. It states MEMORY-LAW acceptance; the representation design belongs
to the implementer inside the locks.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/stage5-collateral-memory-ceiling-t0c-verdict.md` - APPROVED
   evidence base: sec. 7 (fixture quadratic grounding), sec. 8 (the
   decomposition: resident base 4371.5 MiB = 51.6% + A7 save-span transient
   4098.0 = 48.4%, memo residual ~0), sec. 9 (findings 1-5, planner relays -
   nothing here re-opens them).
3. `tasks/stage5-collateral-memory-ceiling-t0c-measurement.md` - ablation
   arithmetic sec. 4; DP-2 ranking reading sec. 5 (ADJUDICATED YES by the
   user decision below: `8469.5 - 4098.0 = 4371.5 <= 5200` - fixing A7 alone
   closes the release peak bar).
4. Shape `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`
   sec. 0 (A7 + the exhaustive `symbol_metrics` consumer read), sec. 3.4
   (T4 artifact-shape contract - GIVEN, do not redesign), sec. 4 (module map
   + verified line counts), R6 (old-blob risk; round-trip + rejection tests
   are the mandated oracle).
5. USER DECISIONS (binding, dated 2026-10-06):
   a. T4 DISPATCHED (decision 1).
   b. `CURRENT_ANALYSIS_VERSION` bumps 2.1.0 -> 2.2.0 IN THE SAME COMMIT as
      the payload change (decision 2; DP-6-class decision; the 2.1.0 freeze
      is released by the user for this registered exception only).
   c. Growth gate G <= 2.0 DEMOTED to tripwire (decision 3): acceptance is
      the absolute 5200 MiB peak bars (both profiles) + the Corpora field
      boot; G keeps being computed and printed, NEVER fails a gate (the
      fixture is Theta(N^2) in edges by construction per t0c-verdict sec. 7;
      RSS scaling with work is not a regression).
6. `tasks/stage5-collateral-memory-ceiling-t0b-handoff.md` sec. "RATIFIED
   gate contract" (the sampling law your in-handoff curve gate implements)
   and `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` sec.
   RATIFICATION (bars/scales/protocol constants - never invent numbers).
7. `tasks/stage5-collateral-memory-ceiling-t0c-handoff.md` gates 0, 4, 5, 8
   - copy those command shapes verbatim for the disk witness, preflight,
   release invocations, and the `--medians` pass (helper:
   `tasks/t0c_run_invocation.py`, committed and ratified; you may NOT edit
   it - it is not in your touch-set).
8. Contract code reads (all verified at authoring):
   - `crates/report/src/manifests.rs:158-163` - `LoadReport { repository_root,
     symbol_metrics: Vec<LoadMetric>, file_metrics: Vec<LoadMetric> }`.
   - `crates/report/src/manager.rs:70-86` - the writer conventions
     (`write_atomic` unique-tmp+rename; `write_json_atomic` via
     `serde_json::to_string_pretty` -> String -> write_atomic);
     `:135-181` - `check_manifest` (analysis_version mismatch = StaleArtifact
     = rebuild, the invalidation machinery you lock, never re-invent).
   - `crates/report/src/manager/metrics_io.rs` - whole file: `:11-19` both
     whole-payload serializations (JSON String + `bincode::serialize` Vec -
     the two concurrent byte strings ARE the transient); `:25-50`
     `build_load_manifest` (the scalar sums: `total_collateral_load` =
     symbol sum + file sum, `total_analyzed_symbols` = symbol count);
     `:67-97` the save span (rayon::join over the two writers); `:99-111`
     the load path (`fs::read` whole Vec + whole-bincode deserialize).
   - `crates/report/src/manager/tests.rs` - `:274-310` round-trip pin
     (asserts per-symbol detail survives save/load - your Red carrier);
     `:592-735` the three FNV byte-locks (`LOCK_REPORT_JSON_HASH` :594,
     `LOCK_REPORT_BIN_HASH` :595, `LOCK_MANIFEST_JSON_HASH` :601, with the
     resync-lineage comment precedent :596-600) + the determinism/
     threading locks :723+ (re-run UNEDITED).
   - `crates/cli/src/commands/measure.rs:22-31` - the artifact-side consumer:
     loads `report.bin` and sums `symbol_metrics[].collateral_load`; migrates
     to the persisted scalar summary; the printed number must stay identical
     by construction, locked by a cross-check test (shape sec. 3.4).
   - `crates/cli/src/commands/analyze.rs:259-303` - producer of its own
     hand-built report; stays the producer (in-memory builds unchanged).
   - `crates/pipeline/src/stage_load.rs:134-151` - the call site (builds the
     report, saves, hands out the in-memory `Arc<LoadReport>`). NO edit
     planned here.
   - `crates/index/src/cache.rs:10-21` (epoch const + lineage doc) + `:336`
     (assert).
   - `tests/integration/tests/m12_incremental_analysis_test.rs:93-97`
     (pinned `analysis_version` literal + lineage comment);
     `tests/integration/tests/m21_scala_semantic_index_test.rs:499`
     (epoch witness); `tests/integration/tests/m27_nonascii_corpus_test.rs`
     (verification only - no version literal exists in it).
   - `tests/integration/tests/m00_golden_baseline_test.rs` (14 goldens incl.
     `load_report` -> `load/report.json` and `load_manifest`) +
     `tests/integration/tests/common/mod.rs` normalization rule 5:
     `analysis_version` and `input_fingerprint` are MASKED in goldens -
     the bump itself moves zero golden bytes; only the payload reshape
     moves `load_report` goldens.
   - `tests/integration/tests/m6_load_analysis_test.rs:178` reads
     `report.symbol_metrics.len()` - determine cold-handle vs loaded-artifact
     and keep it green (see STOP law).
9. `docs/specs/tool-output-distillation.md`; AGENTS.md File-Edit Discipline.

## Objective and invariants

Objective (memory law, not implementation): the N=1280 stage-5 save span must
stop growing peak RSS ~4098 MiB above its own resident tail. Two addressable
terms per the ratified decomposition (t0c-verdict sec. 8):

1. Payload term: drop the never-consumed per-symbol metric detail from the
   PERSISTED load artifact (both `report.json` and `report.bin`). Scalar
   sums stay (symbol count, symbol collateral sum, file count, file
   collateral sum - whatever the manifest and `tool measure` need is exactly
   what survives). Consumer evidence that the detail is dead weight:
   shape sec. 0's exhaustive grep (manifest sums; `tool measure`'s scalar
   sum; UI boot reads ONLY `file_metrics`; zero detail consumers of a loaded
   artifact) + T0a Block-4 data showing the entry mass. `file_metrics`
   stays in full - it IS consumed (UI routes `ui.rs:283,298,300`).
2. Materialization term: the save path must not hold whole-payload byte
   strings; serialize streaming/segmented within the crate's existing writer
   conventions (`manager.rs:70-86` family: unique-tmp + atomic rename stays;
   no new runtime dependency without a written justification note).
   The load-side read path (whole-Vec read) is the warm-boot twin of the
   same class - address it only if your design makes it natural; the gate
   below does not measure warm boot (ui_smoke's warm+cold does).

The in-memory hand-out (`Arc<LoadReport>` with full detail, `stage_load.rs:151`)
is the RESIDENT pass base - the T0c reading says it is the fixture-inherent
quadratic payload and NOT this hop's target (user decision 3 says RSS
scaling with work is not a regression). Do not try to evict it here; the
deferred `LoadMemo` seed-extraction follow-up (ledger row M28 T2 D1) stays
open for a graph-touching hop.

Invariants (each independently falsifiable):
- IV1 Acceptance arithmetic: post-fix release peak@1280 median <= 5200 MiB.
  The saving is 4371.5 + (transient -> ~0) + slack; the G row reads ~4x and
  is RECORDED as tripwire only (user decision 3).
- IV2 `tool measure`'s number (and every manifest total) stays identical:
  `total_analyzed_symbols`, `total_analyzed_files`, `total_collateral_load`
  byte-equal except `analysis_version` (which the m00 normalizer masks
  anyway). Cross-check test required (shape sec. 3.4).
- IV3 Non-regression law: m00/m18 parity suites GREEN with their TEST FILES
  UNEDITED; all goldens byte-identical EXCEPT the two registered
  `load_report.json` resyncs; plan-stage and every other stage's outputs
  unchanged (diff-proof, gate 4).
- IV4 Old-artifact invalidation is EXPLICIT (bincode is strict; the
  fingerprint chain hashes content, not code): the 2.2.0 bump IS the
  invalidation, locked by a round-trip test + an old-blob-rejection test
  (shape sec. 3.4 / R6). A 2.1.0-era `report.bin` must reach `StaleArtifact`
  -> rebuild, never a serde/bincode error surface.
- IV5 Frozen floors unchanged: I1-I3 (progress floors), fixtures,
  `tests/golden/**` except IV3's registered pair,
  `crates/graph/tests/load_memo_capacity_invariance_test.rs` (I5, forever-
  locked, re-run UNEDITED), M26/M27 appended tests UNEDITED.
- IV6 Pipeline equivalence oracle: `crates/pipeline/src/internal_tests.rs`
  report-equality/warm-reuse pins (:122-185, 216-228, 333-337, 418-439, 441)
  stay green UNEDITED under the preferred in-memory-API-preserving shape.
  If your representation makes any of them un-greenable-UNEDITED: STOP and
  report (D-row) - budget-edit authority belongs to the landing reviewer,
  never to a silent edit. This file is NOT in your touch-set.

## Preconditions (hard - enforced at dispatch)

1. Disk gate 0 BEFORE any cargo that can write an N=1280 artifact:
   `python tools/distill.py --kind generic -- python tasks/t0c_run_invocation.py
   --disk-witness --min-free-gib 16` - exit 0 + one `DISK C: free <n> GiB ...
   PASS` line. Exit 1 = STOP and report (operator duty; `os error 112`
   history). Re-run at the reviewer hop.
2. Any invocation dying on `os error 112` mid-save: STOP + report -
   capacity, not code (T1 verdict sec. 7 truth statement).
3. HEAD carries the T0c landing `e9a4ac1` (bench labels, caps 512/128,
   T0b mode grammar all present).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/stage5-collateral-memory-ceiling-t4-journal.md` (checkpoint
  journal; always in-set; create alongside)
- `crates/report/src/manifests.rs` (the payload struct: drop/compact the
  per-symbol detail into the persisted scalar summary)
- `crates/report/src/manager/metrics_io.rs` (save/load paths: streaming or
  segmented serialization; incremental aggregation so manifest totals are
  built without the detail)
- `crates/report/src/manager.rs` (ONLY if the streaming writer variant
  belongs in the `write_atomic` family; prefer metrics_io-local helpers and
  leave this file diff-EMPTY when possible)
- `crates/report/src/manager/tests.rs` (byte-lock resync WITH written
  justification + lineage comments per the :596-600 precedent; round-trip
  contract rewrite; new lock tests). Ceiling note: 841 lines, margin 159 -
  if new tests would not fit, put them in a NEW module
  `crates/report/src/manager/payload_reshape_tests.rs` registered in
  `manager.rs` (that file joins the touch-set only if you create it).
- `crates/cli/src/commands/measure.rs` (read the persisted scalar summary;
  number identical by construction + cross-check test citation)
- `crates/index/src/cache.rs` (epoch const :21 -> "2.2.0"; assert :336;
  lineage doc-comment :10-21 gains the 2.1.0 -> 2.2.0 entry: payload reshape
  changed the persisted bincode layout; bump is the registered DP-2
  invalidation; user decision 2026-10-06)
- `tests/integration/tests/m12_incremental_analysis_test.rs` (:97 pin +
  lineage comment extension)
- `tests/integration/tests/m21_scala_semantic_index_test.rs` (:499 witness)
- `tests/golden/basic/load_report.json` and
  `tests/golden/expensive/load_report.json` - resync ONLY via the documented
  `UPDATE_GOLDENS=1` path scoped to `m00_golden_baseline_test` (its header
  documents the form), justified in the commit message: the registered DP-2
  payload change. These two are the ONLY goldens allowed to move.
- `Cargo.lock` ONLY if cargo regenerates it (expected diff-EMPTY; if it
  moves, the journal must justify why - no new deps planned).

You MUST NOT touch: `crates/cli/tests/**` - `stage5_memory_bench.rs` is
995 lines with a 5-line ceiling margin and its 26 labels are the ratified
tuple (label drift is an evidence-review event, not a T4 edit);
`crates/pipeline/**` (incl. `internal_tests.rs`, `stage_load.rs` - STOP
law IV6); `crates/graph/**` (I5 forever-locked; the T2 D1 seed-extraction
follow-up is NOT this hop - the deferred row stays open for a graph-touching
hop); `crates/cli/src/commands/analyze.rs`, `ui.rs`, the progress floors;
`tests/fixtures/**`; `tests/golden/**` beyond the registered pair;
`tools/**` incl. `bench_baseline.json` (operator PR-2 only) and
`bench_compare*.py` (T4b's files - even though T4b can run in parallel,
its files are never yours); `tasks/t0c_run_invocation.py` (ratified helper);
`tasks/milestone_ledger.md`. No `git stash`, no branch change.

## Version bump procedure (user decision 2; per the M27 T2 finding-3 erratum)

Five dependent sites, updated IN THE SAME COMMIT as the payload change:
1. `crates/index/src/cache.rs:21` const -> "2.2.0" (plus assert :336 and the
   lineage comment).
2. `tests/integration/tests/m12_incremental_analysis_test.rs:97` pin.
3. `tests/integration/tests/m21_scala_semantic_index_test.rs:499` witness.
4. `crates/report/src/manager/tests.rs:601` `LOCK_MANIFEST_JSON_HASH`
   resync (manifest embeds `analysis_version`, bytes legitimately shift;
   report json/bin locks move too here - but because of the PAYLOAD, with
   their own justification sentences; keep the prior-value lineage comments).
5. m27 verification site: `tests/integration/tests/m27_nonascii_corpus_test.rs`
   must stay UNEDITED and its completion test stays UN-IGNORED (the M27 T2
   un-ignore is permanent; its +1 passed is baked into the census-anchor
   arithmetic `1250/0/13/91` - re-ignoring silently breaks every future
   census reconciliation).

PROOF IS A GREEN-TREE FULL-WORKSPACE PROBE, NOT THE OLD TWO-SITE GREP (the
erratum: `grep the constant across crates/` finds only literal definitions
and let the M27 T2 bump go RED twice; the byte-lock carries NO version
string): the scoped package runs (gate 3) plus the full-workspace census at
the reviewer hop, quote counts from GREEN-tree or the
`--no-fail-fast`-pass-through run only (row-65 erratum). Grep is permitted
as a pre-mutation inventory, never as the lock-row proof.

m27/ratified-bars interaction note: until T4b (`tasks/stage5-
collateral-memory-ceiling-t4b-handoff.md`) lands, `bench_compare.py
--bench stage5-memory` compare-path STILL FAILS growth past G. Do not run
it as your gate; your in-handoff curve gate is the t0c helper over the
captures (gates 5-7). If the live gate is run by anyone before T4b lands
and reads exit 1 on the ratio row, that is the expected pre-T4b state -
record honestly, do not re-litigate, do not "fix" it.

## Frozen locks (ENUMERATE BEFORE MUTATING - this is the enumeration)

Every byte-lock covering load-artifact bytes, with its expected verdict:
- `LOCK_REPORT_JSON_HASH` (`manager/tests.rs:594`): RESYNC expected
  (payload reshape changes report.json bytes). Commit message carries the
  justification sentence; lineage comment keeps the prior value.
- `LOCK_REPORT_BIN_HASH` (`manager/tests.rs:595`): RESYNC expected (same
  reason, bincode side).
- `LOCK_MANIFEST_JSON_HASH` (`manager/tests.rs:601`): RESYNC expected
  (version bump embeds `analysis_version`).
- `tests/golden/{basic,expensive}/load_report.json`: RESYNC expected via
  the documented `UPDATE_GOLDENS=1` path (registered DP-2 output change -
  shape sec. 3.4 names it as the user-visible consequence the DP approved).
- ALL OTHER 13 golden artifacts x 2 fixtures (repository/build/includes/
  index/reduced/plans manifests + reports + plans.json, plus both
  `load_manifest.json` - the normalizer masks `analysis_version`):
  BYTE-IDENTICAL. Any parity byte difference in the non-registered set =
  STOP-AND-REPORT. STOP beats the UPDATE_GOLDENS reflex - no reflexive
  golden regeneration, ever.
- The determinism/threading locks (`manager/tests.rs:723+`): green UNEDITED
  - they become the byte-stability proof for the streaming writer once the
  FNV locks are resynced (consecutive saves byte-identical; single-thread
  vs global pool identical).
- `tests/fixtures/**`: frozen, zero-diff (reviewer proves by diff scope).
- m18 parity (`23/0/0/1` form) + m00 form `2 passed` + m17 seam + m6 + m12
  (except :97 pin) + m21 (except :499) + m27 + I5 graph file: UNEDITED and
  green.
- Kickoff sec. 5 lock table applies verbatim (progress.rs, boot contract,
  M26/M27 appended tests, `bench_baseline.json` operator-only).

## What to build (behavior law; representation is the implementer's)

- The persisted `load/report.json` and `load/report.bin` carry: repository
  root; the symbol-summary scalars (count + collateral-load sum); the full
  `file_metrics` detail; and whatever minimal version/fingerprint metadata
  today's contract has. The per-symbol detail sections disappear from
  report.json (that user-visible output change IS the registered DP-2
  decision - do not paper over it, do not keep a hidden copy).
- The manifest totals stay exactly today's numbers (IV2). `tool measure`
  computes its scalar from the persisted summary and prints the SAME number
  by construction.
- `load_load_artifacts`/`load_load_checked` keep working; loaded reports
  expose the summary (detail absent) and full file_metrics.
- Save writes atomically as today (unique-tmp + rename), streaming per the
  materialization term.
- `analyze.rs` keeps hand-building its in-memory report and saving through
  the same reshaped path (its own scalars persist; its console output
  "Analyzed N symbols" stays - it counts its in-memory vec, unchanged).
- Prefer the representation where the IN-MEMORY `LoadReport` API is
  unchanged (detail survives in RAM for the process-lifetime hand-out, the
  persisted form drops it) - it keeps `internal_tests.rs` UNEDITED (IV6).
  A custom serialization for the persisted form (summary + streamed
  file_metrics) within serde_json/bincode as already in the dependency tree
  is the in-scope machinery. This is guidance, not mandate; any shape that
  passes the gates and locks is admissible - IV6's STOP law is the guard
  either way.

## RGB protocol

1. Red (compile-clean failing tests first; NEVER todo!()/unimplemented
   skeletons as "Red"; capture the failing list):
   R1 (in `manager/tests.rs` or the new module): after
   `save_load_artifacts(fixture)`, parse the written `report.json` via
   `serde_json::Value` and assert it has NO `symbol_metrics` key and DOES
   carry the summary scalars. Expected failure today: the key is present
   with the detail array (assertion text: "persisted report.json must not
   carry the never-consumed per-symbol detail: found key symbol_metrics").
   R2: epoch-rejection pin: write a load `manifest.json` declaring
   analysis_version "2.1.0" with a fresh fingerprint in a temp repo, then
   `load_load_checked` must return a StaleArtifact (rebuild path). Expected
   failure TODAY: the "2.1.0" blob is accepted fresh (pre-bump the epoch
   still is 2.1.0) - goes green exactly when the bump lands. This is the
   bump's own Red carrier.
   R3 (measure parity cross-check, IV2): manifest total vs summary-derived
   total equality on the fixture. Green-by-design pin is admissible HERE
   (it locks "identical by construction"); journal labels it as a pin, not
   a Red, per the T0c D2 last-write-wins-class reading.
   Capture all expected-failure lines in the journal before any Green.
2. Green: minimal change per "What to build". NEVER weaken an assertion to
   pass. The byte-lock resyncs are contract changes, not weakenings - each
   carries its justification comment (lineage precedent :596-600).
3. Refactor: honest names (summary struct name says it summarizes), dedupe
   the two writers' streaming without merging their files, doc comments on
   the new contract at the struct and at `save_load_artifacts_with_languages`.
   Re-run.

## Required tests

- R1 persisted-detail-absence (json) + scalar round-trip through
  `load_load_checked` (bin side): loaded summary == saved summary; loaded
  file_metrics == saved file_metrics.
- R2 old-blob (2.1.0 manifest) rejection -> StaleArtifact, not a decode
  error (message wording may mention rebuild/stale).
- R3 totals-parity cross-check pin (IV2).
- Manifest bytes: fixture manifest equals today's fields with only
  `analysis_version` moved (proves "manifest totals survive" + nothing else
  drifted).
- Determinism/threading locks UNEDITED stay green (streaming byte-stability
  rides them post-resync).
- Measure-side: the cross-check is R3; `measure.rs` keeps its printing
  (integration-level number parity is the reviewer's Corpora-boot duty).
- Existing suites green UNEDITED (except the enumerated pins/locks/goldens):
  `-p ariadne-report`, `ariadne-integration-tests` (m00/m6/m12/m17/m18/m21/
  m27), `-p ariadne-pipeline` (internal_tests oracle IV6),
  `-p ariadne-graph` incl. the I5 file.

## Gates (exact commands; timeout 600000 ms except stated; ALL via the
distilling entry points - subagent sessions cannot invoke gate commands
directly, ledger row 61)

0. Disk witness (BEFORE any cargo): the command in Preconditions 1; exit 0
   + the PASS line quoted verbatim in the journal.
1. Red: `python tools/testrun.py --distill -p ariadne-report` - quote the
   R1/R2 failing lines (R3 pin line green is expected and journal-noted).
2. Green: same command after implementation - canonical line + digest
   header verbatim.
3. Integration scoped: `python tools/testrun.py --distill -p
   ariadne-integration-tests` - run the `UPDATE_GOLDENS=1` m00 resync
   scoped per the m00 header BEFORE this run; this run must read `0 failed`
   with the two resynced goldens.
4. Golden-scope proof (before committing):
   `python tools/distill.py --kind generic -- git diff --name-only -- tests/golden`
   - output is EXACTLY the two `load_report.json` paths. Anything else =
   STOP-and-report (IV3).
5. Preflight release boot (codegen + disk smoke): t0c gate-4 shape -
   `python tools/distill.py --timeout 3600 --capture target/t4-preflight.txt
   -- python tasks/t0c_run_invocation.py --files 1280 -- cargo test
   --release -p ariadne-cli --test stage5_memory_bench -- --exact
   stage5_rss_scale_runner --ignored --nocapture --test-threads=1` - exit 0;
   capture FULLY READ immediately after (E1 law); quote the `STAGE5_BENCH
   profile=release files=1280` context + peak row.
6. Curve group: k = 1..5, up to 8 on rejections -
   `python tools/distill.py --timeout 3600 --capture target/t4-postfix-k.txt
   -- cargo test --release -p ariadne-cli --test stage5_memory_bench --
   --ignored --nocapture --test-threads=1` (t0c G1 form verbatim; one boot
   per process, `--test-threads=1`, foreground, never parallel; each
   capture FULLY READ right after its run; journal 1-3 lines per accepted
   invocation).
7. Law pass: `python tools/distill.py --kind generic -- python
   tasks/t0c_run_invocation.py --medians target/t4-postfix-1.txt ...`
   (explicit listed paths). Required lines: `accepted 5 of N` (>= 5);
   `MEDIAN(accepted) stage5 peak RSS (release, N=1280) | <= 5200.0` is the
   acceptance reading; the `stage5 RSS growth ratio (release, N=640->1280)`
   median is recorded in the journal explicitly as `TRIPWIRE (never a
   gate, user decision 2026-10-06)` - never PASS/MISS wording for it.
   Fewer than 5 accepted after 8 invocations = STOP + report the bimodality
   evidence; NEVER pad the group.
8. `python tools/hygiene_check.py` - exit 0.
9. `python tools/encoding_guard.py --staged` - exit 0.
10. Commit: `python tools/commit_scoped.py` with --scope for EVERY
    touch-set path (journal, the report files incl. any new test module,
    measure.rs, cache.rs, m12, m21, the two goldens, Cargo.lock if moved)
    - message contains: the DP-2 payload justification sentence, the
    golden-resync justification, the bump lineage sentence. ONE landing
    commit (bump + payload + locks + goldens together is the
    same-commit discipline of decision 2).

Declared gate-invocation ledger (journal-count law, ledger row 73; T0c
D3/D6 + verdict finding 4 precedent): base scoped gates ~10 + disk witness
1 + preflight 1 + curve 5-8 + medians 1 + golden resync run 1 = declare up
FRONT-loaded in the journal D-row (~21 worst case). The ratified >=5-accepted
protocol floor is arithmetically incompatible with the ~10 count ceiling
for a fix+measure hop; the DECLARATION is the law, the overshoot is
expected and acceptable, silence is the violation.

## Hygiene / ceilings at risk (pre-checked at authoring)

`manager/tests.rs` 841/1000 (margin 159 - the real R2 risk; route overflow
to the new module), `metrics_io.rs` 172, `manifests.rs` 222, `manager.rs`
182, `cache.rs` 384, `measure.rs` 65, m12 207, m27 314. Functions < 100
(the streaming helper family), files < 1000, no `tmp`/`res`/`foo`/`bar`
names, intention-revealing names, one responsibility per function.
`hygiene_check.py` exit 0. Pure ASCII (byte-escape fixtures if a test ever
needs non-ASCII; none planned). Sizing: one primary crate + the named
5-site bump riders; est. ~650-1100 produced lines (report src+tests
~450-650; goldens shrink by the dropped detail sections; bump ~20;
measure ~15) - within the ~1200 bound.

## Discipline clauses

- UTF-8: Write/Edit tools only; no Set-Content/Out-File/redirect/
  Get-Content-replace; pure-ASCII authored content (AGENTS.md).
- Touch only the touch-set; out-of-scope breakage = report, never fix.
- No bare high-volume commands (raw cargo test/build/clippy, full
  unittest discover, ui_smoke, bench_compare) in this session; every gate
  above is already distill-routed. The LIVE `--bench stage5-memory` run is
  NOT yours (pre-T4b G-fail expectation, see the bump note).
- Journal: 1-3 lines per Green step + per accepted curve invocation + the
  declared-count D-row + every lock finding as a dated D-row. The journal
  is the dead session's decision log.
- Context wall: STOP, return a disk-state inventory; a dead session is
  NEVER resumed - successor runs `python tools/checkpoint_guard.py resume
  --spec tasks/stage5-collateral-memory-ceiling-t4-handoff.md --scope
  crates/report --cmd ...`.
- STOP-and-report beats any reflexive-regression-instinct: on IV3 golden
  drift beyond the registered pair, IV6 oracle breakage, or R6 rejection
  not landing - stop, journal, report. The landing reviewer owns
  adjudication; you own nothing outside the touch-set.

## Acceptance (machine-checkable verdict lines)

- Gate 2 canonical line: report-scoped `N passed / 0 failed / ...` (N =
  prior + new cases, quoted from the GREEN run, row-65 law).
- Gate 3: integration-scoped `0 failed` (m00 with resynced goldens;
  m12/m21 green at "2.2.0"; m6/m17/m18/m27 UNEDITED green).
- Gate 4: golden diff = exactly the two `load_report.json` paths.
- Gate 7: `accepted 5 of N` and the peak median line `<= 5200.0`; the
  tripwire line present with its citation (no verdict word attached to G).
- cache.rs:21/:336 assert green at "2.2.0"; lineage comment extended.
- Gates 0/8/9 exit 0; `git show --stat <landing>` == touch-set exactly
  (Cargo.lock diff-EMPTY unless justified); every byte-lock resync carries
  a lineage comment with the prior value.
- Reviewer owns (NOT implementer gates): workspace census reconcile from
  the `1250 passed / 0 failed / 13 ignored / 91 suites` anchor (+ report
  test cases; m27's +1 already inside the anchor); `ui_smoke` PASS (warm +
  cold boot = the load-path field proof); `encoding_guard --all`; per-
  commit `git show` oldest-first vs touch-set; independent median recompute
  from your captures; `git diff` proofs for fixtures/IV5; the live
  `--bench stage5-memory` run stays PR-2-bound and pre-T4b G-fail is the
  expected shape; the Corpora field boot (G-H) stays the terminal
  milestone acceptance (user/operator duty).

Reviewer-gate reference: adjudicated via the `review-gate` skill; its
commit-vs-touch-set check enforces this file. The reviewer additionally
adjudicates: the lock-resync justifications, the golden-exception scope
(exactly two files), the IV6 oracle outcome, the bump's green-tree probe
(the erratum-compliant proof form), and the curve group's accounting
(rejections loud, medians recomputed independently).

Deferred findings this hop does NOT own (rows stay open; do not absorb):
M28 T2 D1 `LoadMemo` seed-extraction (next graph-touching hop); I7c
implicit-counter tripwire row; testrun `_echo` non-cp1252 row; row-72
allowlist/pin (next manifest touch - your commit does NOT touch manifests,
keep it that way).
