# Shape (governing plan): `stage5-collateral-memory-ceiling`

Companion kickoff:
`.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-kickoff.md`
(E-rows, scope in/out, frozen-lock commands, acceptance gates G-A..G-H, DP
summary). History, read not restated: the SPENT pair
`.kilo/plans/1791168710915-large-corpus-boot-responsiveness-{kickoff,shape}.md`
(M26 + M27 CLOSED per `tasks/milestone_ledger.md` rows 2026-10-05), including
its sec. 9 hygiene outcome (the stage modules this shape touches) and its T1
sufficiency-gate discipline, which this shape mirrors for memory.

## 0. Verified code facts (all reads 2026-10-05; the OOM's memory lifecycle)

Stage 5 = `ensure_load` cold path, `crates/pipeline/src/stage_load.rs:28-119`.
Boot wiring: `crates/cli/src/commands/ui.rs:246` passes `workers: None`, so
`run_on_configured_pool` (`stage_load.rs:133-148`) runs inline and the
`par_iter` passes execute on the GLOBAL rayon pool - unlike stage 2's W=1,
stage 5 is fully parallel at boot. The death frame `5737/5753 files` is the
`CollateralFile` mapping at `ui.rs:190-195` - the file-metric pass.

Simultaneously alive at the file-pass tail (each a candidate accumulator;
T0 ranks them - this ranking is hypothesis, not verdict):

- A1 `semantic_closure_cache` (`crates/graph/src/load_caches.rs:46-53`),
  append-only `Mutex<HashMap<SymbolId, Arc<HashSet<SymbolId>>>>`, doc says
  plainly: "Memory grows by one `Arc<HashSet<SymbolId>>` per distinct seed
  ever queried; entries are permanent". The symbol pass
  (`stage_load.rs:72-73` -> `collect_symbol_metrics`
  `stage_load.rs:156-169` -> `analyze_symbol_load`
  `crates/graph/src/load.rs:550-580`, memo insert at `load.rs:569` ->
  `load_caches.rs:189-209`) warms it with one full closure PER SYMBOL before
  the first file event. Aggregate ~ Sigma over symbols of |closure(s)| -
  superlinear in corpus size.
- A2 `symbol_metrics: Vec<LoadMetric>` (`stage_load.rs:72,100-104`): one
  `LoadMetric` per symbol (`load.rs:535-547`), each carrying
  `semantic_symbols` + `collateral_symbols` + `collateral_sources[].symbols`
  + `semantic_sources[].symbols` + `largest_collateral_source` clone -
  roughly 3-5 cloned `Vec<SymbolId>` copies of per-symbol sets. Alive from
  the symbol pass through the file pass, the artifact save, and the
  process-lifetime `Arc<LoadReport>` (`stage_load.rs:116`;
  `lib.rs:194,581-588`; shared to the UI at `ui.rs:252`).
- A3 `physical_expansion_cache` (`load_caches.rs:54-64,378-403`), append-only,
  one full `Arc<HashSet<SymbolId>>` (symbols of the whole transitive include
  closure) PER FILE, grown inside the file pass via the single-def-file fast
  path (`load.rs:510-516`). This is the accumulator that grows AT the 5737/5753
  tail - its peak IS the loop end. In boot it is pure single-use bloat: the
  pipeline queries each def-file's expansion exactly once; cross-use exists
  only for post-boot on-demand routes, which run on a SEPARATE fresh memo
  (`ui.rs:289` `LoadMemo::with_closures`).
- A4 stage-5 deep clones: `stage_load.rs:43` `cached_include_graph()` deep-
  clones the entire `(IncludeGraph, closures)` pair (`lib.rs:555-561`) and
  `stage_load.rs:44` `cached_reduced()` deep-clones the whole
  `ReducedSemanticModel` (`lib.rs:569-573`) - both alive while the pipeline
  caches still hold the originals. The A3 take pattern (`take_include_graph`,
  `lib.rs:469-475`, boot usage `ui.rs:250`) proves the fix class exists.
- A5 cross-stage residency: nothing drops `index_cache` (`lib.rs:191`) after
  stage 3, so the full `IndexResult` sits through stage 5; stage 5 itself
  never reads it (`stage_load.rs:41-44` needs include + reduced only).
- A6 rayon transients: per file call, the multi-seed union allocates an owned
  `HashSet` (`load_caches.rs:163-183`), `syms` clones the file's symbol set
  (`load.rs:605-610`), and `compute_load` builds two partition vecs plus two
  `contributing_sources` groupings (`load.rs:528-533,276-307`). With the
  global pool, ~N_LOGICAL_CORES corpus-scale transients are concurrent.
- A7 save/cache tail: the full `LoadReport` is serialized to JSON and to
  bincode CONCURRENTLY (`crates/report/src/manager/metrics_io.rs:81-91`),
  each building the whole serialized byte string in RAM. Warm boot reads all
  `report.bin` bytes into one `Vec` and deserializes the whole report
  (`metrics_io.rs:99-105`) - old artifacts of the E1 shape will OOM warm
  boots too, and `tool analyze` shares the save path
  (`crates/cli/src/commands/analyze.rs:259-303`).

Who consumes `LoadReport.symbol_metrics` (verified by exhaustive grep of all
crates): the manifest aggregate (`metrics_io.rs:30-44` - a scalar sum and a
length), `tool measure`'s scalar sum from artifacts
(`crates/cli/src/commands/measure.rs:22-28`), `tool analyze`'s own
hand-built report, and benches/tests. The UI boot consumes ONLY
`file_metrics` (`ui.rs:283,298,300`); no UI route, payload builder, or boot
path reads `symbol_metrics`. The persisted per-symbol detail has ZERO
detail consumers - only derived sums - which is what opens DP-2.

No memory oracle exists anywhere: `tools/bench_compare.py` parses only the
`label | ms` timing grammar (`tools/bench_compare.py:88-109`, phase tuple
`:43-58`); grep for RSS / WorkingSet / peak / memory across `tools/` is
empty. The `Peak memory` line in `measure.rs:49` measures the instrumented
BUILD process, not the analyzer. A fix without an RSS oracle is unfalsifiable
- the oracle is a core hop, not a rider.

Current fixture scale vs failing scale: the stage-5 benches generate 520
headers / 40 TUs (`crates/cli/tests/payload_tail_bench.rs:23-26`; generator
constants `crates/pipeline/tests/load_perf_bench.rs:29-35` `HEADER_COUNT =
520`, `TU_COUNT = 40`); the failing corpus has 5753 collateral files.
Superlinearity is invisible below the cliff, which is why every existing
gate was green (E6).

Test-pinned surfaces a fix must survive or budget:
`crates/pipeline/src/internal_tests.rs` pins report equality against a
direct-analyze reference (`:122-185,216-228,333-337`), warm reuse
(`:418-439`), and `collect_file_metrics`' signature/behavior (`:472-515`).
These are pre-M26 tests - not on the M26/M27 unedited floors - but they are
the equivalence oracle; the shape's preference is to keep them UNEDITED
green and budget edits (with per-test justification at review-gate) only for
a signature change T2 cannot avoid.

## 1. Topology and sequencing verdict

One milestone, six candidate hops, two profiles in every table (E5). The
shape does NOT presume the accumulation site: A1-A7 are code-verified
candidates, and the T0 table - not this document - assigns the fix budget.

| Hop | Crate/module | Content | Dispatch |
|---|---|---|---|
| T0a | `crates/cli/tests` only | New ignored RSS-curve bench (sec. 6): peak-RSS-vs-file-count at scales {N0/2, N0} on a hub-heavy scaled fixture, DEBUG and RELEASE columns, per-phase RSS checkpoints, memo-len x entry-sample attribution (existing `test-instrument` counters `load_caches.rs:301-311`), warm-deserialize row | IMMEDIATE |
| T0b | `tools/` only | `bench_compare.py --bench stage5-memory` gate mode (sec. 3.1 grammar) + `*_tests.py` + baseline rows via `--update-baseline` (PR-2, operator on VERIFIED-IDLE) | after T0a labels stable |
| T1 | `crates/graph` | Bound/scope the two append-only memo caches (sec. 3.2): capacity knob, per-query result equality regardless of capacity; equivalence-first RED tests BEFORE the policy swap | gated on T0 table |
| T2 | `crates/pipeline` | Stage-5 lifecycle: kill A4 clones, release `index_cache` before stage 5 with idempotency proof, chunk/stream the two passes to bound A2 + A6 transients while keeping the exact event stream (I6); extract helpers - `ensure_load` is 92 lines (`stage_load.rs:28-119`) and must not cross the 100-line function ceiling | gated on T0 table |
| T3 | `tools/` + `AGENTS.md` | Profile-lever rider (DP-1): the ledger's named small hop - `ui_smoke` release flag or smoke-adjacent flag + the documented launch line - discharging the OPEN obligation that survived M27 closeout | IMMEDIATE (orthogonal to the fix) |
| T4 | `crates/report` (+ `measure.rs` consumer) | Conditional: load-artifact payload reshape (drop/compact per-symbol detail, sec. 3.4) | ONLY on T0 ranking AND DP-2 YES |

Sequencing law, mirroring M27: measure before building machinery. No T1/T2
commit predates the ratified T0 table. T3 is the one fix-adjacent hop that
dispatches now because it shrinks the measurement confound and the ledger
prices it "worth more than any plausible T3 win on current data" (deferred
findings row 67, OPEN). The profile lever never substitutes for the fix:
release-only survival is not sufficiency, because A1-A3 growth is
profile-independent; the E1 corpus would die later in release, not never.
I7 therefore gates BOTH columns.

Handoff sizing (Planning Law): every hop is <= 1 crate or 1 top-level module
directory, <= ~10 scoped gates, <= ~1200 produced lines. T0a's bench file is
new and self-contained (generator duplication is the sanctioned pattern -
`payload_tail_bench.rs:8-10` precedent); T0b is a pure tools hop; T1/T2 are
crate-disjoint and may run in parallel ONLY after the T0 table names both
their budgets (touch sets otherwise overlap on behavior, not files).

## 2. Invariants, numbered, each independently falsifiable

- I1 (frozen floor) The stage-5 13-frame replay
  `crates/cli/src/commands/ui/progress_contract_tests.rs:141`
  `test_stage_five_milestone_sequence_is_monotonic_and_contained` passes
  UNEDITED at every commit; the file appears in zero diffs.
- I2 (frozen floor) `crates/cli/src/commands/progress.rs` diff-EMPTY.
- I3 (frozen floor) Boot contract tests APPEND-ONLY; M26/M27 appended tests
  (`boot_progress_tests.rs` and kin) UNEDITED and green (I13 lineage).
- I4 (frozen floors) `tests/fixtures/**` zero-diff; m00 parity + golden
  baseline `2 passed` UNEDITED; m18 parity `23/0/0/1` UNEDITED; m17 seam
  UNEDITED; `CURRENT_ANALYSIS_VERSION` stays `2.1.0` unless DP-2 YES names
  the bump in its own registered decision.
- I5 (fix equivalence - the swap oracle) Every `analyze_*` result is
  `==`-identical across memo capacities `{0, 1, ratified cap, usize::MAX}` on
  every locked fixture - authored as the RED suite BEFORE the policy swap and
  kept forever. Capacity-0-vs-capacity-max equality makes the oracle
  self-referential: it needs no pre-swap snapshot and it falsifies exactly
  the axis the fix moves. Existing suites (internal_tests report equality,
  m00/m18 parity) stay green UNEDITED on top of it.
- I6 (event-stream equality) For a given fixture, the recorded
  progress-event stream (kinds, order, payloads) from `ensure_load` is
  byte-equal pre/post fix - chunking must not add, drop, or reorder
  `CollateralFile`/`SymbolsAnalyzed`/`ArtifactsSaved` events, and the
  per-file completion count stays exactly one event per file
  (`AtomicUsize` precedent `stage_load.rs:82,199`). Oracle: recording test
  mirroring the bench's `track_collateral` stamps + I1 green UNEDITED.
- I7 (NEW peak-memory oracle - the milestone's namesake invariant), three
  machine-checkable clauses:
  - I7a absolute bar: at ratified scale N0 on the scaled fixture, the bench
    row `stage5 peak RSS (release)` <= bar_release and
    `stage5 peak RSS (debug)` <= bar_debug, medians-of-N, bars RATIFIED from
    the T0 measured table at evidence review (measured-table method - the
    same discipline as M27's ratified wall-time bar - never a guessed
    constant). Discrimination proof: the pre-fix tree FAILS I7a on at least
    the debug column at the same scale (kickoff G-C).
  - I7b growth cap: `RSS(N0) / RSS(N0/2)` <= G, G < 2 ratified from the T0
    table - the falsifiable statement of "no longer superlinear in this
    window".
  - I7c CI deterministic clause: on the small committed fixture, after a
    full boot, `LoadMemo::semantic_closure_cache_len()` and
    `physical_expansion_cache_len()` (test-instrument counters,
    `load_caches.rs:301-311`) <= a ratified capacity constant -
    machine-independent, no wall-time, runs inside `cargo test`. Placement
    of the RSS gate itself (local vs CI) is DP-3; I7c is CI-safe either way.
- I8 (rider, only if DP-1 YES) The documented human launch route produces a
  RELEASE binary: `ui_smoke` with the ratified flag builds
  `cargo build --release` and launches `target/release/tool.exe`; tools unit
  test asserts the flag plumbing (the honest ceiling for a doc/flag change -
  a launch test cannot check what humans type).
- Hygiene (every hop): `hygiene_check.py` + `encoding_guard.py --all` exit 0;
  zero added hex; authored content pure ASCII; `ensure_load` stays < 100
  lines after T2 extraction.

## 3. Contracts at the changing boundaries

### 3.1 RSS bench + gate-mode grammar (T0a/T0b)

The existing parse contract (`bench_compare.py:88-109`) admits exactly
`label | number` rows and treats them as ms; breaking it exits 2. So the new
bench keeps the single-pipe numeric grammar and carries identity in labels,
while the gate mode owns units and policy:

```
# stage5_memory_bench rows (values in MiB; profile IN the label):
stage5 peak RSS (debug, N=640)                   |   4210.0
stage5 peak RSS (release, N=640)                 |    860.0
stage5 peak RSS (debug, N=1280)                  |  17110.0
stage5 peak RSS (release, N=1280)                |   3290.0
symbol-pass RSS delta (release, N=1280)          |   1540.0
file-pass RSS delta (release, N=1280)            |   1120.0
memo semantic-closure entries (release, N=1280)  | 312400.0
memo physical-expansion entries (release, N=1280)|   1280.0
warm deserialize peak (release)                  |   ...
```

- `--bench stage5-memory`: its own `DOCUMENTED_PHASES_STAGE5_MEMORY` tuple,
  unit declared MiB in the gate mode, per-profile rows keyed like today's
  machine-class keys, its own ratio (wider than 1.25x - RSS noise and
  allocator fragmentation; exact value ratified from T0 variance at
  evidence review), and I7b's growth-ratio rows checked as first-class rows
  (a ratio row is `RSS(N0)/RSS(N0/2)` printed by the bench itself).
- Peak-RSS source: process peak working set (Windows
  `GetProcessMemoryInfo`/`GetProcessWorkingSetSizeEx`), read via the
  smallest surface ALREADY in the dependency tree; implementer checks first,
  else a raw `extern "system"` block in the test file (dev-only). No new
  runtime dependency without a written justification.
- Attribution rule: entry-count x sampled-entry-size from the
  `test-instrument` counters, per-phase RSS deltas, and the capacity-0 A/B
  run (I5's zero-capacity mode doubles as the accumulator ablation) - never
  a profiler's raw dump in a journal (R9).
- Fixture generator: hub-heavy (a few high-fanin headers so closures are
  corpus-scale - the E1 shape), scales N in {N0/2, N0}, N0 proposed 1280
  collateral files and RATIFIED at the T0 evidence review against runtime
  budget, not fixed here. Generator lives in the new bench file (sanctioned
  verbatim-duplication pattern; `load_perf_bench.rs` must-not-grow, sec. 7).
- Optional (DP-5 YES): a labeled real-corpus capture mode run by the USER on
  `C:\Code\Corpora` - the only reading at true failing scale; recorded as an
  E-row, never a gate (uncommittable).

### 3.2 Bounded memo contract (T1; shape-level, policy is implementer's)

```rust
// crates/graph/src/load_caches.rs - additive constructor; today's
// new()/with_closures() keep working (every existing caller unchanged).
impl LoadMemo {
    /// Memo with entry-count caps for the two append-only caches.
    /// `0` disables memoization for that cache (pure recompute).
    /// Query RESULTS are identical for every capacity - eviction may trade
    /// time, never bytes (I5). Entry bound is per cache, not global.
    pub fn with_closures_and_capacity(
        closures: HashMap<FileId, IncludeClosure>,
        semantic_closure_cap: usize,
        physical_expansion_cap: usize,
    ) -> Self;
}
```

The pipeline's boot memo becomes a capped instance with ratified caps; UI
on-demand routes may keep larger caps (they are the caches' real reuse
beneficiary, `ui.rs:286-289`). Eviction policy is implementer-chosen; the
invariants it must not cross are I5 (equality), I7c (counters), and the
existing lock discipline (the `Mutex`-guard-discipline doc comment at
`load_caches.rs:7-19` stays true).

### 3.3 Stage-5 lifecycle contract (T2; files and outcomes, not code)

- `ensure_load` must run stage 5 with ONE live copy each of graph/closures/
  reduced during its own execution: replace the deep clones at
  `stage_load.rs:43-44` with take/restore over `self` (the progress-callback
  take/restore pattern at `stage_load.rs:81-94` is the in-repo precedent for
  threading around the `self.notify` borrow conflict; A3's
  `take_include_graph` for the move-out pattern). The warm-reuse contract
  (`internal_tests.rs:418-439`) and boot's `take_include_graph` after stage
  5 (`ui.rs:250`) must both stay green UNEDITED.
- Release `index_cache` before stage 5 runs (nothing in stage 5 reads it -
  sec. 0 A5); proven by a new assertion-style test that a cold pipeline's
  stage-5 phase does not hold the index result (fixture-scale RSS delta
  clause or a size-counter check - implementer's pick), and warm-path
  `ensure_index` reload behavior green.
- Bound the A2/A6 transients (chunked or streamed passes) with I6 event-
  stream equality and the `collect_file_metrics` signature kept if at all
  possible (internal_tests.rs:472-515 green UNEDITED is the preferred
  outcome; an unavoidable signature change is budgeted with per-test
  justification, never silent).
- `ensure_load` and every extracted helper stay < 100 lines (it is at 92
  now - extraction is mandatory, not optional, for any growth).

### 3.4 Artifact-shape contract (T4, CONDITIONAL on T0 + DP-2)

If the persisted per-symbol detail goes: `LoadReport`
(`crates/report/src/manifests.rs:158-163`) loses/compacts
`symbol_metrics`; manifest totals survive via incremental aggregation at
build time (the `metrics_io.rs:30-44` sum moves to accumulation during the
passes). Hard rules: bincode is strict, so old `report.bin` blobs CANNOT
deserialize into a reshaped struct - invalidation must be explicit, and the
fingerprint chain hashes CONTENT not code (M27's standing lesson), so the
clean lever is a `CURRENT_ANALYSIS_VERSION` bump (breaking the I4 freeze via
its registered DP-2 exception, five dependents - see M27 ledger row errata)
or a load-manifest version check with rebuild-on-mismatch; the design picks
ONE and locks it with a round-trip + old-blob-rejection test.
`measure.rs:22-28` migrates to the manifest total (identical number by
construction, locked by a cross-check test); `analyze.rs` stays the producer
of its own hand-built report. The human-readable `report.json` per-symbol
sections are a user-visible output change - that is precisely why this is a
DP, not a T2 rider.

### 3.5 Rider contract (T3; DP-1)

`ui_smoke.py` gains the ratified release-flag form (builds
`cargo build --release`, launches `target/release/tool.exe`, same route and
no-store assertions); `AGENTS.md` Build section gains the line that the
documented human launch IS the release binary (the debug-everywhere default
recorded by the ledger row finally named); tools unit test covers the flag
plumbing. No Rust change; stage-5 bench columns unchanged.

## 4. Module map (hop -> touches -> must-not-touch) and line counts

Current counts verified by read this pass (R2 ceiling 1000):

| File | Lines | Note |
|---|---|---|
| `crates/pipeline/src/lib.rs` | 634 | margin 366; take/restore helpers land here or stage_load |
| `crates/pipeline/src/stage_load.rs` | 210 | T2's home; room fine, function ceiling is the risk |
| `crates/pipeline/src/stage_index.rs` | 288 | per M26 ledger D3; untouched |
| `crates/graph/src/load.rs` | 673 | untouched unless I5 suite wants a seam |
| `crates/graph/src/load_caches.rs` | 410 | T1's home; margin 590 |
| `crates/cli/src/commands/ui.rs` | 853 | MUST NOT TOUCH (margin 147 - protect it) |
| `crates/cli/tests/payload_tail_bench.rs` | 472 | stage-5 timing gate stays as-is |
| `crates/pipeline/tests/load_perf_bench.rs` | 866 | MUST NOT GROW (M27 rule) |
| `crates/report/src/manifests.rs` | 222 | T4 only |
| `crates/report/src/manager/metrics_io.rs` | 172 | T4 only |

Hop touch sets:

| Hop | Touches | Must NOT touch |
|---|---|---|
| T0a | new `crates/cli/tests/stage5_memory_bench.rs` (+ optional `tests/common` helper) | any src; `load_perf_bench.rs`; `payload_tail_bench.rs` semantics |
| T0b | `tools/bench_compare.py`, `tools/bench_compare_tests.py`, baseline rows (operator PR-2 only) | existing `DOCUMENTED_PHASES` semantics; `bench_baseline.json` by hand |
| T1 | `crates/graph/src/load_caches.rs` (+ new equality/capacity test files in `crates/graph/`) | `load_explain.rs`; graph builders; memo lock discipline doc's guarantees |
| T2 | `crates/pipeline/src/stage_load.rs` (+ `lib.rs` seams as needed; `internal_tests.rs` ONLY if budgeted per sec. 3.3) | `stage_index.rs` hot path; progress events contract; warm-reuse tests; `ui.rs` |
| T3 | `tools/ui_smoke.py` (+ its `*_tests.py`), `AGENTS.md` | all crates src |
| T4 | `crates/report` (manifests, metrics_io, its tests), `crates/cli/src/commands/measure.rs`, invalidation lock | fixtures; goldens; the I4 version freeze except its registered DP-2 exception |

Whole-program must-not-touch: `progress.rs`, `progress_contract_tests.rs`,
`boot_progress_tests.rs`, `tests/fixtures/**`, `tests/golden/**`, UI assets/
themes, `bench_baseline.json` outside PR-2.

## 5. Risks -> mitigation -> oracle

- R1 Fixing the wrong accumulator (shape presumes, field OOMs again).
  Mitigation: T0-first law (sec. 1); capacity-0 ablation runs as part of the
  bench; I7a must FAIL pre-fix and PASS post-fix at the same scale. Oracle:
  G-A ratification + G-C discrimination gate.
- R2 Eviction/zero-cap hurts throughput (the caches exist because LoadPerf
  A1/B1 measured their win). Mitigation: caps ratified WITH the
  `bench_compare` stage-5 medians in hand; per-cache caps (A3's boot memo
  entry is single-use - it can be capped tight while route traffic keeps a
  loose memo). Oracle: existing stage-5 `label | ms` gate within tolerance
  (G-D) after PR-2 refresh; boot_stages rows unchanged.
- R3 RSS gate flapping (allocator fragmentation, AV, background load - the
  stale-timing-baseline disease, ledger row 61, in a new unit). Mitigation:
  medians-of-N inside the bench, per-machine class, ratified WIDE ratio at
  evidence review, local-gate placement (DP-3), VERIFIED-IDLE for baseline
  rows, one sanctioned protocol-retry precedent (`6a48a85`). Oracle: T0b
  unit tests pin the parse + ratio math; the flakiness budget itself is a
  ratified table row.
- R4 internal_tests / collect_file_metrics edits collide with reviewer
  frozen-floor instincts. Mitigation: sec. 3.3 declares exactly which
  pre-M26 tests may move and only with per-test justification; the true
  floors (I1-I4) have zero exceptions. Oracle: review-gate commit-vs-touch-set.
- R5 index_cache release breaks warm paths or idempotency. Mitigation:
  warm-reuse tests green UNEDITED (internal_tests:418-439, pipeline_test
  warm suite); fingerprint-fresh reload is already the loaded-from-disk
  branch. Oracle: those suites + ui_smoke (it boots warm AND cold).
- R6 T4's artifact change silently breaks reload of pre-existing corpora.
  Mitigation: strict bincode + explicit invalidation (sec. 3.4),
  old-blob-rejection test, migration note for the E1 corpus's own artifacts
  (they are stale-by-schema the moment T4 lands - the field-acceptance
  re-index is G-H's own boot). Oracle: round-trip + rejection tests.
- R7 Debug-column temptation: "release boots fine, close it". Mitigation:
  I7a gates BOTH columns; kickoff sec. 7 states release-only survival is
  not sufficiency. Oracle: the two bars, both must pass.
- R8 Profile lever ships and the fix is deprioritized around it. Mitigation:
  T3 is explicitly orthogonal and never satisfies I7; the milestone's
  acceptance is I7 + G-H, not the rider. Oracle: G-C/G-H cannot be met by
  T3 alone (pre-fix tree fails I7a in release too at scale - T0 table shows
  the margin).
- R9 Subagent context wall (M23/M27 precedent): every gate via
  `distill.py`/`testrun.py --distill`; bench prints the bounded table only;
  attribution via counters, never profiler dumps in journals.
- R10 Scaled fixture too slow for routine gates. Mitigation: curve runs are
  `#[ignore]`d behind the gate mode (like today's benches); routine CI keeps
  only the small-fixture I7c counter test. Oracle: gate-mode runbook lines
  in T0b.

## 6. T0 measurement protocol (the confirmation bench, not a diagnosis hop)

The dominant accumulators (A1-A3) ARE identifiable from code reading - the
append-only caches are documented in their own comments as permanent-growth
(`load_caches.rs:46-64`) and the per-symbol metric vector is structurally
`O(symbols x per-symbol sets)` - so diagnosis is NOT a separate hop; T0 is a
CONFIRMATION bench that ranks magnitudes and ratifies bars. Table shape (all
rows both profiles - E5/M27-E3 lesson, profile confounds measurement):

1. RSS-vs-file-count curve, hub-heavy generator, N in {N0/2, N0}: raw
   numbers + growth ratio rows (I7b).
2. Per-phase RSS checkpoints (analyzer prep / symbol-pass end / file-pass
   tail / save tail / post-return): separates A1+A2 (symbol-pass delta) from
   A3+A6 (file-pass delta) from A7 (save tail).
3. Capacity-0 ablation (A/B same tree, memo caps 0 via the I5 mode -
   authored with T1's seam, so T0b's curve may re-run it post-T1; the
   pre-fix tree may use a scratch local build for the ablation reading).
4. Memo entry counters x sampled entry size (attribution arithmetic in the
   table itself).
5. Warm-deserialize row (A7 read side) against a committed
   small-fixture artifact only - real-corpus blob sizes quoted, not shipped.
6. Context rows: debug/release wall medians at N0 (the E5 confound), and if
   DP-5 YES, the user-captured real-corpus rows as E-rows.

The evidence review (user + orchestrator, M27 precedent) then ratifies:
N0, the I7a bars, the I7b cap G, the I7c capacity constant, the gate ratio,
the T0b label set, and the T0 ranking's fix budget (which of T1/T2/T4 fire
and in what dispatch order). Nothing downstream pre-commits those numbers.

## 7. Hygiene notes

- Line counts sec. 4 table - nothing currently sits near the file ceiling on
  a touched path; `ui.rs` (853) and `load_perf_bench.rs` (866) are protected
  by must-not-touch, exactly as M26 protected them by extraction.
- `ensure_load` (92 lines) is near the 100-line function ceiling BEFORE any
  growth; T2's extraction of pass helpers is a planned hygiene move, not a
  discovered failure.
- The RSS bench must keep its phase table to the bounded grammar; no
  2000-char rows, no corpus paths in committed output beyond fixture temp
  dirs.
- New names follow the house style (`stage5_memory_bench`,
  `with_closures_and_capacity`, `DOCUMENTED_PHASES_STAGE5_MEMORY`); no
  `tmp`/`res` locals in gate code (R3 allowlist is legacy-only).

## 8. Open questions - DP rows needing a user answer before planning completes

1. DP-1: ship the profile lever now as T3 (recommended YES - OPEN in the
   ledger past M27 closeout; "worth more than any plausible T3 win on
   current data"; it also shrinks the debug-column bars' confound).
2. DP-2: is the load-artifact payload reshape (T4) in scope at all? It is
   the only fix candidate that touches persisted format; it requires either
   the `CURRENT_ANALYSIS_VERSION` bump (I4 exception) or manifest-version
   invalidation, and changes human-readable report output. Answer due at
   the T0 evidence review (the ranking may make the answer obvious - if the
   save tail A7 is not a top contributor, NO is cheap).
3. DP-3: RSS gate placement - confirm local-gate-only for the RSS curve
   (graduation-backlog: bench is local-gate; GitHub-runner timings, and even
   more RSS, are incomparable), and confirm the CI-accompaniment is the
   deterministic I7c counter test rather than any wall/memory timing.
4. DP-4: ratify (at the T0 evidence review) N0, the I7a bars, G, the I7c
   cap, and the gate ratio - by measured-table method only; AND confirm the
   field boot on `C:\Code\Corpora` (G-H) is the terminal acceptance gate.
5. DP-5: real-corpus RSS capture - allowed as optional user-run evidence, or
   synthetic-only? (Recommended: allowed; it is the only reading at the
   true failing scale.)

## 9. Ready-to-register ledger row (draft; kickoff runner commits, planner
assigns the M-number - do not guess)

| Milestone | Status | Where (commit/branch) | Frozen locks | Notes |
|---|---|---|---|---|
| Stage-5 collateral memory ceiling (key `stage5-collateral-memory-ceiling`; M-number + dates assigned by milestone-lifecycle kickoff - do not guess) | PLANNED (kickoff+shape `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-{kickoff,shape}.md`; hops T0a cli-tests + T3 rider dispatchable immediately; T0b after T0a labels; T1 graph + T2 pipeline gated on T0 table; T4 report conditional on T0 ranking AND DP-2) | plan artifacts; `main` | Kickoff sec. 5 lock table verbatim (goldens/fixtures, m00/m18 UNEDITED, progress.rs byte-identical, stage-5 13-frame replay UNEDITED, boot contract append-only, M26/M27 appended tests UNEDITED, `CURRENT_ANALYSIS_VERSION` 2.1.0 unless DP-2 registered, `bench_baseline.json` operator PR-2 only) | Motivating field evidence E1-E6 2026-10-05: stage 5 died at 5737/5753 files after exhausting system RAM; small failed allocation = messenger; DEBUG profile per the still-OPEN ledger profile-lever row; stages 1-4 clean (M27 tokenizer fix holds). Fix oracle: capacity-invariance equality suite frozen BEFORE any policy swap (I5); NEW peak-memory oracle I7a/b/c bars ratified from the T0 measured table - never guessed. Inherits M27's sufficiency-gate discipline: measure before building machinery; release-only survival is not sufficiency. |
