# T0c Handoff - M28 stage5-collateral-memory-ceiling: post-fix live RSS curve + cap-0 ablation ranking (measurement hop)

Hop id: `t0c-postfix-curve-ablation`. MEASUREMENT hop: it produces the
decision-support DATA the orchestrator's written evidence review consumes -
NOT the formal gate verdict. The `--bench stage5-memory` GATE still waits
for operator PR-2 (no `::stage5-memory` baseline key exists today); this
hop's runs are post-fix curve confirmation + the cap-0 ablation ranking
that DP-2 requires for the T4 dispatch decision. The operator freed disk
2026-10-06; gate 0 re-witnesses free space before anything runs.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/stage5-collateral-memory-ceiling-t0b-handoff.md` sec. "RATIFIED
   gate contract" - the sampling law verbatim (median-of->=5 ACCEPTED
   isolated invocations; >8% per-slot-median operand rejection on BOTH
   peak operands at BOTH scales, symmetric; loud rejected-sample
   accounting, never silent exclusion; multiset parsing of the
   twice-per-scale peak labels; 26-label tuple identity).
3. `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` - THE SUPPLIER:
   sec. E1 (decomposable ratio operands; fully-read protocol = read each
   capture in full immediately after its run; the +12% half-scale
   bimodality and -14% full-side trimming witnesses), Block 2 (save-span
   transient ~4.0 GiB at N=1280, profile-independent), Block 4 (uncapped
   memo attribution at N=1280: semantic-closure 3,122,416 + physical-
   expansion 6,328,320 = 9,450,736 aggregate refs), Block 5 (wall
   context: full ignored-suite invocation ~361-389 s debug / ~86-114 s
   release). Uncapped release N=1280 peak class ~9040 MiB per
   `tasks/stage5-collateral-memory-ceiling-t0b-verdict.md` (pre-fix class
   citation).
4. `crates/cli/tests/stage5_memory_bench.rs` (landed, locked UNEDITED):
   the isolated-orchestration architecture (scale runner gated on
   `STAGE5_MEM_BENCH_FILES`, children inherit the harness env), the
   growth-ratio test re-emitting both operand peak rows, the checkpoint /
   attribution / wall row families. Curve tests:
   `stage5_rss_curve_half_scale`, `stage5_rss_curve_full_scale`,
   `stage5_rss_growth_ratio`, `stage5_warm_deserialize_peak` (run as the
   full `--ignored` suite, `--test-threads=1`).
5. `crates/pipeline/src/stage_load.rs` :61-154 (`ensure_load` lifecycle -
   the thing being measured) and `build_stage5_analyzer` (the ONE route
   site this hop may edit); `crates/graph/src/load_caches.rs` :117/:125/
   :134 (`pub const SEMANTIC_CLOSURE_BOOT_CAP: usize = 512`,
   `PHYSICAL_EXPANSION_BOOT_CAP: usize = 128`, `I7C_CACHE_LEN_PIN`) -
   read-only reference, that crate is FROZEN for this hop.
6. `crates/ui/Cargo.toml` :22-26 - the sanctioned duplicate-dev-dep
   feature pattern (`ariadne-graph = { path = "../graph", features =
   ["test-instrument"] }`) to mirror in the cli dev-dep line.
7. `docs/specs/tool-output-distillation.md` - digest shapes, capture-path
   discipline, prohibitions (all runs distill-routed, zero raw pastes).
8. `tasks/stage5-collateral-memory-ceiling-t1-verdict.md` "Follow-up
   dispositions" (cap 0 = pure-recompute ablation, LOCKED; I5 file
   forever-locked UNEDITED) and
   `tasks/stage5-collateral-memory-ceiling-t2-verdict.md` sec. 11 (the
   ~8.6-8.7 GiB post-fix N=1280 projection; transient survives by design).

## Objective and invariants

Objective: produce `tasks/stage5-collateral-memory-ceiling-t0c-measurement.md`
carrying, under the ratified sampling law:
- post-fix RSS curves, release AND debug, scales {640, 1280}, caps active
  (512/128 landed from `d6640a2`/`e52a9e9`), against the ratified bars
  (`stage5 peak RSS` <= 5200 MiB per profile at N=1280;
  `stage5 RSS growth ratio` <= 2.0) - record ACTUALS; the honest
  expectation from the T2 verdict is ~8.6-8.7 GiB = the bar FAILS pending
  the A7 transient fix;
- the release-profile cap-0 ablation variant (memo resident ~= 0 via
  pure recompute) for the per-contributor ranking;
- the DP-2 ranking statement (draft): save-span transient vs memo
  retention residual vs the rest, computed by the arithmetic contract in
  sec. "What to build".

Invariants:
- ZERO production behavior change: the ablation route is
  `#[cfg(feature = "test-instrument")]`-gated only; with the feature off
  (every production build) the code compiles out and defaults are the
  landed consts. The plain (feature-off) pipeline suite must stay
  `32 passed / 0 failed / 4 ignored / 7 suites` byte-behavior-identical.
- The landed bench file, the I5 suite
  (`crates/graph/tests/load_memo_capacity_invariance_test.rs`),
  `load_caches.rs`, goldens/fixtures, m00/m18, `progress.rs`, the boot
  contract: UNEDITED (frozen locks below).
- `tools/bench_baseline.json` is NEVER written (operator PR-2 only) and
  `tools/**` is not touched at all (T0b-owned).
- No ledger touches; no gate verdict is asserted from this hop - the doc
  records numbers + DRAFT readings, the written evidence review adjudicates.
- Isolation discipline per the RATIFIED contract: one stage-5 boot per
  process (the bench's own child architecture), `--test-threads=1`, no
  concurrent load, foreground; per-invocation capture files retained at
  distinct `target/t0c-*.txt` paths and FULLY READ immediately after each
  run (evidence doc sec. E1 law).

## Preconditions (hard)

1. Free disk: gate 0 witnesses >= 16 GiB free on C:. Derivation (state it
   in the journal): the T0a evidence records the N=1280 save-span
   transient at ~4.0 GiB (Block 2, profile-independent); the save writes
   JSON+bincode artifacts through `.tmp` files whose serialization buffer
   IS that transient, so the disk-side save is the same order of
   magnitude; 4x for temp + final + fixture + OS headroom = 16 GiB floor.
   os error 112 killed two T1-review attempts at lower headroom. If gate 0
   fails: STOP and report (operator duty), do not run, do not retry.
2. Any invocation that dies on `os error 112` mid-save: STOP + report;
   it is capacity, not code (T1 verdict sec. 7 truth statement).
3. HEAD carries the T0b landing (`dc294bb`) - the mode's label grammar
   exists so the doc's rows stay tuple-compatible.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/stage5-collateral-memory-ceiling-t0c-journal.md` (checkpoint
  journal; always in-set; create alongside)
- `tasks/t0c_run_invocation.py` (the ONE committed measurement helper:
  env-setter for `STAGE5_MEM_BENCH_FILES` / `ARIADNE_LOAD_MEM_CAPS` so the
  ablation env reaches the distill-routed cargo child via plain env
  inheritance - chained `$env:` shell forms hit the global catch-all,
  ledger row 61; `--disk-witness` mode via `shutil.disk_usage`; and
  `--medians` mode implementing the ratified rejection/median law over
  listed capture files. stdout stays a passthrough so the OUTER distill
  wrapper owns capture + digest; child exit code propagates - I2 truth)
- `tasks/stage5-collateral-memory-ceiling-t0c-measurement.md` (the
  artifact; sec. "What to build" is its required structure)
- `crates/pipeline/src/stage_load.rs` (ONLY the ablation route inside
  `build_stage5_analyzer`, `#[cfg(feature = "test-instrument")]`-gated,
  <= ~25 lines + the loud witness line)
- `crates/pipeline/src/stage_load_tests.rs` (the three feature-gated
  route tests; T2 in-crate adjacency precedent)
- `crates/pipeline/Cargo.toml` (`[features] test-instrument =
  ["ariadne-graph/test-instrument"]` - chains the graph counters so the
  route tests can assert cache lengths)
- `crates/cli/Cargo.toml` (`[dev-dependencies]` line
  `ariadne-pipeline = { path = "../pipeline", features =
  ["test-instrument"] }` - the ui/graph pattern at binding input 6)
- `Cargo.lock` (only if cargo regenerates the dev-dep edge)

You MUST NOT touch: `crates/cli/tests/stage5_memory_bench.rs` (the curve
mechanism is measured UNEDITED; env passes through by inheritance -
`run_isolated_runner_child` sets only the scale var and inherits the
rest); `crates/graph/**` incl. `load_caches.rs` and the I5 file;
`crates/report/**` (T4/DP-2 territory); `tools/**` incl.
`bench_compare*.py` and `bench_baseline.json`; `tests/**`;
`tasks/milestone_ledger.md`; any other `tasks/*`. No `git stash`, no
branch change.

## What to build

Route slice (commit 1, TDD):
- Env literal: `ARIADNE_LOAD_MEM_CAPS`, value `"a,b"` where each token is
  a non-negative integer or `max` (-> `usize::MAX` = uncapped control).
- Inside `build_stage5_analyzer` under `#[cfg(feature = "test-instrument")]`:
  if the env is set, build `LoadMemo::with_closures_and_capacity` with the
  parsed caps and emit exactly once per process, stderr, the witness line
  `load-memo caps override active: semantic=<n> physical=<n>`; if unset,
  the landed consts (512/128) - default path byte-identical. Malformed
  value = PANIC loudly (no silent zeros, bench env style). Under
  `#[cfg(not(feature = "test-instrument"))]` the env is not read at all.
- Helper `tasks/t0c_run_invocation.py`: modes `--files N` / `--caps a,b`
  (set env, then subprocess the trailing `-- argv`, passthrough output,
  propagate exit), `--disk-witness --min-free-gib F` (exit 0 iff free >=
  F, prints one line `DISK C: free <n> GiB of <total> GiB - PASS/STOP`),
  `--medians <capture...>` (per-label multiset parse of `label | value`
  rows; ratified law: provisional per-slot medians over the peak-label
  pool, wholesale REJECT any invocation with any peak sample diverging
  >8% strict from the provisional slot median, print
  `REJECT invocation k: ...` + `accepted K of N`, then per-label medians
  over ACCEPTED invocations; non-peak labels: medians over all; <5
  accepted => `GROUP INCOMPLETE (K of N)` line, exit 1 - never a
  verdict). Pure ASCII, <100-line functions.

Groups (commit 2, measurement):
- G1 post-fix release curve: >= 5 ACCEPTED full-suite invocations (the
  full `--ignored` suite: half-scale + full-scale + growth-ratio + warm,
  children isolated per scale).
- G2 post-fix debug curve: >= 5 ACCEPTED, same form, debug.
- G3 release cap-0 ablation (`--caps 0,0`): >= 5 ACCEPTED, same suite +
  the `ARIADNE_LOAD_MEM_CAPS` env set via the helper. Release-only by
  design (stated scope decision, journal D-row): the launch profile per
  AGENTS.md is release; RSS growth is profile-INDEPENDENT per the
  evidence doc Block 5 reading (only wall time is); debug cap-0 would
  multiply the ~20x-slow debug passes by mass recomputation for zero
  additional bar data.

Measurement doc required content (structure it exactly):
1. Provenance: start rev, both commit hashes, `STAGE5_BENCH` context
   lines, disk-witness line, the `load-memo caps override active` witness
   cited per G3 accepted invocation, capture paths, VERIFIED foreground/
   idle statement.
2. Per-group per-label table: per-accepted-invocation values + median
   column + accounting line (`accepted K of N`); rejected invocations
   listed with their >8% divergence reasons, never dropped silently.
3. Bar comparison: `stage5 peak RSS (release, N=1280)` median vs 5200 MiB
   and `(debug, N=1280)` median vs 5200 MiB; `stage5 RSS growth ratio
   (<profile>, N=640->1280)` medians vs G <= 2.0. Print
   `BAR: PASS` / `BAR: MISS` DRAFT markers only - the evidence review
   adjudicates; annotate the expected ~8.6-8.7 GiB FAIL context.
4. Ablation ranking (release, N=1280 medians):
   `transient_landed  = peak(G1) - RSS checkpoint save tail(G1)`,
   `transient_cap0    = peak(G3) - RSS checkpoint save tail(G3)`,
   `memo_residual     = peak(G1) - peak(G3)` (retained memo term at the
   landed caps), `memo_contribution_uncapped = 9040 MiB class (T0a
   uncapped) - peak(G3)`, plus resident-after-return rows. Printed
   arithmetic rows in the same `label | value` grammar with distinct
   `t0c`-suffixed labels (e.g. `t0c memo residual (release, N=1280)`) so
   they can never be confused with the gate tuple.
5. DRAFT DP-2 ranking sentence: ordered contributor list with the numbers
   from sec. 4 and an explicit answer to "does the per-symbol payload
   (save-span transient class, A7) rank among top post-fix contributors?"
   - marked DRAFT, adjudication belongs to the written evidence review.
6. Explicitly NOT run: no `--update-baseline`, no gate-verdict assertion,
   no debug ablation, no T4/DP-2 work, no N>1280 scales.

## RGB protocol (route slice only - measurement groups have no assertions)

1. Red, in the three-capture order:
   a. `caps_env_override_bounds_both_boot_caches` (feature-gated): set
      `ARIADNE_LOAD_MEM_CAPS=0,0`, run the existing small fixture through
      the analyzer seam, assert the graph `test-instrument` counter
      accessors report cache lengths 0. Expected failure BEFORE the
      route: the env is inert, counters exceed 0 (assert names the
      actual lengths).
   b. `caps_env_absent_keeps_landed_const_caps` - green-by-design at Red
      (the default-inert pin, same class as the T0b last-write-wins pin;
      declare it in the journal).
   c. `caps_env_malformed_panics_loudly` (catch_unwind): expected failure
      BEFORE the route: no panic occurs (assert fails on Ok).
   A test that fails to COMPILE is not Red; the feature-gated tests must
   compile against the current tree (feature chain lands with them).
2. Green: minimal route (sec. "What to build"); never weaken an
   assertion; re-run plain (feature off) to prove default inertness.
3. Refactor: route stays inside `build_stage5_analyzer`, honest names
   (`parse_caps_override_env` or similar), no abbreviation.

## Required tests (in `stage_load_tests.rs`, all feature-gated)

- Env discipline (hard, no flake tolerated): `std::env` vars are
  process-global and the cargo test harness runs threads in parallel -
  all three tests acquire ONE shared static `Mutex` guard (poison-safe
  via `unwrap_or_else(|e| e.into_inner())`) around every
  set-env/run/unset section, and restore the unset state on every exit
  path. Model the fixture + in-crate visibility pattern on the existing
  T2 `stage_load_tests.rs` helpers (fixture duplication precedent D5 -
  zero demand on `tests/fixtures/**`).
- (a) caps override reaches both caches (counters == 0 at 0,0; and if
  you add it, `<=` pin at `512,128`) - non-ignored.
- (b) env unset => caps ARE the landed consts (behavior pin) -
  non-ignored.
- (c) malformed value panics loudly - non-ignored.
- Existing pipeline tests UNEDITED and green.

## Gates (exact commands; all via distilling entry points - ledger row 60;
timeouts as stated because these are live measurement runs)

0. `python tools/distill.py --kind generic -- python tasks/t0c_run_invocation.py
   --disk-witness --min-free-gib 16` - exit 0 + one PASS line. exit 1 =
   STOP (precondition 1). Run FIRST, before any cargo.
1. Red captures (instrumented scoped form below, pre-route) - journal
   quotes the three expected failure lines.
2. `python tools/testrun.py --distill -p ariadne-pipeline -- --features
   test-instrument` - expected `35 passed / 0 failed / 4 ignored /
   7 suites` (32 prior + 3 route tests; adjust the +N honestly in the
   journal if you add the `<=` pin case).
3. `python tools/testrun.py --distill -p ariadne-pipeline` - expected
   `32 passed / 0 failed / 4 ignored / 7 suites` (feature off = prior
   line EXACT - the default-invariance witness).
4. Preflight (codegen + disk smoke, one release full-scale boot):
   `python tools/distill.py --timeout 3600 --capture target/t0c-preflight.txt
   -- python tasks/t0c_run_invocation.py --files 1280 -- cargo test
   --release -p ariadne-cli --test stage5_memory_bench -- --exact
   stage5_rss_scale_runner --ignored --nocapture --test-threads=1` -
   exit 0; fully read the capture; journal quotes the `STAGE5_BENCH
   profile=release files=1280` context + peak row.
5. G1 invocations k = 1..5+ (up to 8):
   `python tools/distill.py --timeout 3600 --capture target/t0c-g1-release-k.txt
   -- cargo test --release -p ariadne-cli --test stage5_memory_bench --
   --ignored --nocapture --test-threads=1` (k is the invocation number;
   fully read each capture immediately after its run - E1 law).
6. G2 invocations k = 1..5+ (up to 8): same WITHOUT `--release`, capture
   `target/t0c-g2-debug-k.txt`. (First debug invocation may pay a debug
   rebuild - the 3600 s timeout covers it; T0a used the same form.)
7. G3 invocations k = 1..5+ (up to 8): G1 command wrapped as
   `-- python tasks/t0c_run_invocation.py --caps 0,0 -- cargo test
   --release ...` exactly as in gate 4's wrapper form, capture
   `target/t0c-g3-cap0-k.txt`.
8. Median/rejection pass, one per group:
   `python tools/distill.py --kind generic -- python tasks/t0c_run_invocation.py
   --medians target/t0c-g1-release-1.txt ... (explicit listed paths)` -
   the accounting + median table goes into the doc; <5 accepted after 8
   attempts = STOP + report (bimodality evidence, never pad the group).
9. `python tools/hygiene_check.py` - exit 0.
10. `python tools/encoding_guard.py --staged` - exit 0.
11. `python tools/commit_scoped.py --scope tasks/stage5-collateral-memory-ceiling-t0c-journal.md
    --scope tasks/t0c_run_invocation.py --scope crates/pipeline/src/stage_load.rs
    --scope crates/pipeline/src/stage_load_tests.rs --scope crates/pipeline/Cargo.toml
    --scope crates/cli/Cargo.toml --scope Cargo.lock -m "test(pipeline):
    cfg(test-instrument) load-memo caps override route for the M28 T0c
    ablation (default build byte-identical)"`
12. `python tools/commit_scoped.py --scope tasks/stage5-collateral-memory-ceiling-t0c-journal.md
    --scope tasks/stage5-collateral-memory-ceiling-t0c-measurement.md
    -m "docs(tasks): M28 T0c post-fix RSS curve + cap-0 ablation ranking
    measurement (data, not gate verdict)"`

Declared gate-invocation count law (T0b verdict finding 1): the journal
MUST itemize every invocation per gate (the 5/5/5 protocol floor makes
this session ~25-30 total); declare it as a dated D-row - the sizing
law's count ceiling is deliberately exceeded by the ratified protocol
floor for a measurement hop, and silence is the violation, not the
overshoot. Session bash timeouts: use 3600000 ms on gates 0 and 4-8.

Runtime envelope (from evidence Block 5): nominal >= 15 accepted group
runs ~= 5 x ~400 s (G2 debug) + 5 x ~100 s (G1) + 5 x ~100-220 s (G3,
recompute adds CPU) ~= 45-70 min plus builds/preflight; worst case 24
group invocations ~= 100-120 min. Stay under the 2 h envelope.

## Frozen locks

Kickoff sec. 5 lock table verbatim, plus this hop's:
`crates/cli/tests/stage5_memory_bench.rs` UNEDITED (proving command:
`git diff <start>..HEAD -- crates/cli/tests/stage5_memory_bench.rs`
EMPTY); I5 file + `crates/graph/**` diff-EMPTY; `bench_baseline.json`
diff-EMPTY AND untouched (no PR-2 usurpation); `tools/**` diff-EMPTY;
m00/m18 parity + boot contract + `progress.rs` diff-EMPTY;
`CURRENT_ANALYSIS_VERSION` 2.1.0. The plain-suite gate 3 line IS the
production-invariance witness.

## REVIEWER hop obligations (NOT implementer gates)

- Fresh census `python tools/testrun.py --distill`: expected
  `1250 passed / 0 failed / 13 ignored / 91 suites` (+3 via the cli
  dev-dep feature unification, the T1 ui/graph precedent) - if the census
  shows +0 the unification did not propagate: journal-note it (the
  scoped instrumented gate remains the canonical proof), not a failure.
- Independently recompute one group's medians/rejections from its listed
  captures; re-run ONE release invocation end-to-end.
- Verify the `load-memo caps override active` witness line is present in
  EVERY G3 capture cited (ablation-not-executed is the failure mode this
  gate exists to kill) and absent from G1/G2 captures.
- bench_compare default-mode run (pipeline/src diff voids the exemption;
  stale-baseline exit-1 class expected - relays row 62, not a blocker);
  `ui_smoke` PASS; hygiene + `encoding_guard --all` exit 0;
  commit-vs-touch-set per commit; ledger relays (census anchor, DP-2
  ranking to the evidence review, cap-economics wall rows to row 69/72).

## Hygiene / ceilings at risk

`stage_load.rs` 363 -> ~390 lines (ceiling 1000; ceiling R2 margin wide,
but the 100-line function ceiling binds `build_stage5_analyzer` - the
route parse goes in a helper fn). Helper < ~150 lines, functions < 100.
Measurement doc is prose+tables: pure ASCII, every number transcribed
from a capture whose path you cite. Total produced diff est. ~700 lines
(< 1200 bound).

## Discipline clauses

- UTF-8: Write/Edit tools only; pure ASCII; no PowerShell pipes; `$env:`
  chained forms hit the global catch-all - ALL env setting goes through
  the helper (that is why it exists).
- Touch only the touch-set; report out-of-scope breakage, never fix it.
- No bare high-volume commands; every run wrapped in `tools/distill.py`
  (or `tools/testrun.py --distill`); digest header + exit + capture path
  cited in the journal, zero raw pastes; captures fully read per E1 law
  (reading is context-cheap relative to the run - budget for it).
- Journal 1-3 lines per accepted invocation + the declared count D-row.
  Context wall: STOP + disk-state inventory; successor:
  `python tools/checkpoint_guard.py resume --spec
  tasks/stage5-collateral-memory-ceiling-t0c-handoff.md --scope tasks --cmd ...`
- Groups are NOT gates: never describe a group median as a PASS of the
  ratified bar in the report-back; only the evidence review adjudicates.

## Acceptance (machine-checkable verdict lines)

- Gate 0: one `DISK C: free <n> GiB ... - PASS` line, exit 0.
- Gate 2 `Ran/canonical` line `35 passed / 0 failed / 4 ignored /
  7 suites` exit 0; Gate 3 `32 passed / 0 failed / 4 ignored / 7 suites`
  exit 0 (EXACT prior - default-invariance proof).
- Each group: `accepted K of N` with K >= 5, and the doc's accounting
  section lists every rejection with its divergence value.
- Doc contains sec. 1-6 with the bar-comparison rows, the t0c-suffixed
  arithmetic rows, and the DRAFT DP-2 ranking sentence.
- `git show --stat` per commit = touch-set exactly;
  `bench_baseline.json` and `tools/**` absent from every diff; the bench
  file diff-EMPTY across the range.
- `git diff <start>..HEAD -- Cargo.lock` empty-or-dev-dep-edge-only.
Reviewer gate: `review-gate` skill; its commit-vs-touch-set check
enforces this file; the reviewer additionally relays (never adjudicates)
the bar outcome and the DP-2 ranking to the written evidence review, and
confirms the caps-witness line discipline in G3.
