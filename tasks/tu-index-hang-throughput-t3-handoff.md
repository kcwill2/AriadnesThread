# T3 Handoff - M27 tu-index-hang-throughput: `--workers` wiring + completion-count percents

*** STATUS: DISPATCH-BLOCKED *** Do NOT dispatch, estimate against, or
pre-emptively implement until BOTH conditions are recorded in writing:

1. The T1 SUFFICIENCY GATE FAILED:
   `tasks/tu-index-hang-throughput-t1-measurement.md` carries the evidence
   review's finalized `SUFFICIENCY-GATE: FAIL - release+single-thread does
   not meet the ratified wall-time bar at corpus scale (T3 dispatch
   required)` line with the ratified wall-time bar, improvement target K,
   worker setting N, and the auto-workers cap filled in (shape sec. 3.5:
   "The shape below is then the ready contract, not assumed work";
   DP-3 governs the policy WHEN the day comes, measurement decides whether
   the day comes).
2. M26 V3 LANDED on `main` (hard V3-before-T3 ordering on
   `crates/pipeline/src/lib.rs` + the extracted `stage_index.rs` /
   `stage_load.rs` - shape R6; the inheritance record: `TuStarted`, the
   path-in-message lock, the boot tests, append-only contract tests exist
   on `main` and may not be re-opened or edited by this hop).

If T1's gate says ACCEPT, this file stays a dormant ready-contract and the
throughput mandate is honored by measuring, not by assuming parallelism.

Milestone: `tu-index-hang-throughput` (M27). Because the full contract
exceeds one scope once emission plumbing is required, it is pre-split into
two hops along the seam that lets the FIRST one ship independently (an
optional, default-off scheduler emission hook that changes nothing for
existing callers); T3b consumes it. DP-2/DP-3 bind the content verbatim.

Binding decisions (USER-RESOLVED - do not re-open):
- DP-3: `tool ui --workers <n>` flag + auto default; the live-refresh
  watcher inherits the same setting.
- DP-2 (BINDING): percent = completion COUNT, monotonic clamp under every
  arrival order.

Binding inputs (read at dispatch time, targeted reads only):
1. This file (complete contract; fill the bracketed [RATIFY AT DISPATCH]
   constants from the T1 evidence review note FIRST).
2. Shape sec. 3.5 (contract sketch, verbatim below), sec. 2 (I1'/I4/I7/I12/
   I13), sec. 9 (T3 adds into the extracted modules, not `lib.rs`), sec. 1
   (inheritance record).
3. `tasks/tu-index-hang-throughput-t1-measurement.md` (medians, ratified
   K/N/cap, FAIL verdict line).
4. `crates/cli/src/lib.rs` lines 139-145 (the `analyze --workers` flag
   precedent: `#[arg(long)] workers: Option<usize>`) and 238-256 + 442-460
   (`Ui` variant + `dispatch_ui` + `run_ui` arg chain).
5. `crates/cli/src/commands/ui.rs` post-M26 (boot + watcher build sites of
   `build_ui_payload`; `stage_progress_callback`'s stage-2 arms).
6. `crates/pipeline/src/stage_index.rs` post-V3 (`ensure_index` workers
   branch; `ProductionAnalyzer` wiring; the `collect_file_metrics`
   `AtomicUsize` completion-count precedent in the pipeline crate) and
   `crates/scheduler/src/engine.rs` (`ProductionAnalyzer::analyze` loop -
   the per-TU completion point).
7. `docs/specs/tool-output-distillation.md`.

## T3a (first hop): scheduler emission hook - ships independently

Scope: 1 crate (`crates/scheduler`); ~5 gates; ~250-400 lines.
Objective: `ProductionAnalyzer` gains an OPTIONAL progress sink emitting,
per TU START, `TuStarted { done: <completion count via fetch_add>, total,
source: repo-relative path }` and, when the pool path finishes, the terminal
`TuIndex { done: total, total }` - through a neutral
`&mut dyn FnMut(&ProgressEvent)`-style callback owned by the caller, NOT a
pipeline dependency inversion (define the callback over the scheduler's own
event struct if `ProgressEvent` lives above it; pipeline maps/adapts in
T3b - decide against the actual dependency graph at dispatch, journal the
choice). Completion count semantics: `AtomicUsize::fetch_add(1, Relaxed)` on
the shared counter (the `collect_file_metrics` precedent, pipeline
`collect_file_metrics` ~lib.rs:941 pre-V3 / `stage_load.rs` post-V3); the
"last starter wins" active-TU cell mirrors the shape's ArcSwap note (an
`Arc<RwLock<PathBuf>>` cell is acceptable - no new deps without approval).
Backward-compat: hook `None` => byte-identical behavior (all existing
callers); `--workers Some(1)` through the pool path must equal the
single-threaded shard outputs (existing production-path parity suites, I1').
Tests (scheduler unit + existing suites UNEDITED green): hook-off no-op;
hook-on emits exactly one start event per TU (parallel arrival => event
ORDER may shuffle, `done` values are exactly 0..total, each used ONCE);
terminal frame once; `Some(1)` parity. Gates: `python tools/testrun.py
--distill -p ariadne-scheduler` red/green, hygiene, encoding,
distill-wrapped `commit_scoped`.

## T3b (second hop): cli+pipeline wiring (+ compile-coupled mapping tests)

Scope: `crates/cli` + `crates/pipeline` (declared two-crate exception: the
callback adaptation + `run_ui` arg chain are compile-coupled; same pattern
as M26 V3); ~7 gates; ~350-550 lines. After T3a AND M26 V3.

Contract (shape sec. 3.5, DP-3 verbatim):
```rust
// crates/cli/src/lib.rs Commands::Ui gains (mirroring the Analyze flag):
#[arg(long)] workers: Option<usize>,  // None => auto (resolved DP-3)
```
- `dispatch_ui`/`run_ui` gain `workers: Option<usize>` and thread it to
  BOTH boot and `spawn_watcher` (watcher inherits - DP-3); `--export` path
  behavior-neutral (its own call site, journal the choice).
- Auto default (None): `resolved_workers = [AUTO_WORKERS_CAP ratified at the
  T1 evidence review] .min(std::thread::available_parallelism())`, floor 1;
  `Some(1)` reproduces today's single-thread path exactly (escape hatch +
  A/B); the PIPELINE LIBRARY default stays `None`-means-single-threaded for
  all other callers.
- `build_ui_payload` (boot + watcher closures) calls
  `Pipeline::new_with_progress(repo, Some(resolved_workers), cb)`; the
  pipeline `ensure_index` workers branch adapts T3a's hook to
  `ProgressEvent::TuStarted`/`TuIndex` (mapping in the pipeline crate,
  inside `stage_index.rs` - NOT `lib.rs`, hygiene sec. 9).
- I4 (DP-2, BINDING): the stage-2 percent is completion-COUNT based
  (`done` = fetch_add completions) and MONOTONIC NON-DECREASING under every
  arrival order, sequential and shuffled-parallel, on both consumers: add
  the monotonic clamp at the cli mapping boundary - `ui/stage_spans.rs`
  keeps a per-stage max-so-far inside the `stage_progress_callback` closure
  (percent never decreases within a stage run; out-of-order arrival
  plateaus, never regresses). `total_stages` stays 5.
- I13 (inheritance obligation): every M1/M26 append-only test - the stage-5
  replay, `first_frame_tests`, `boot_panic_tests`, `boot_progress_tests`,
  the TuStarted path-in-message lock, the contract tests - runs UNEDITED
  and green under W>1 arrival. If an M1 test cannot stay green unedited:
  STOP, report (that is a contract collision, not a test bug).
- I1' byte parity sequential-vs-parallel (existing production-path parity
  suites re-run UNEDITED); I7 shard envelope fields diff-EMPTY.
- G2-4 numeric target: the affected `boot_stages` phase median <= T1
  median / [K ratified at dispatch] via
  `python tools/distill.py -- python tools/bench_compare.py --bench
  boot-stages` (protocol-retry once with proof if the machine flaps, R2
  precedent); `python tools/distill.py -- python tools/ui_smoke.py` PASS.

Required tests (T3b): I4 table-driven shuffled-arrival test THROUGH the real
mapping fn (arrival orders: sequential, reverse, stride-2 interleave; assert
per-stage percent non-decreasing + final == completion-count table);
workers-flag unit tests (clap parse: absent -> auto-resolve fn, `1` -> 1,
`0` -> reject); watcher-inherits test (resolution fn shared, both call sites
same input); I13 W>1 boot-recording test (temp corpus >= 8 TUs, W=4, assert
M1 locks' expectations - first frame, path-in-row, >=1 event per TU - all
green against UNEDITED assertions). Gates: scoped `-p ariadne-cli` +
`-p ariadne-pipeline` red/green (distill), boot_stages gate run (distill),
hygiene, encoding, distill-wrapped `commit_scoped` - 7 total.

## Touch-sets (per hop; any edit outside = blocker, report don't fix)
- T3a: `tasks/tu-index-hang-throughput-t3-journal.md` (SHARED journal for
  both hops), `crates/scheduler/src/engine.rs`, `crates/scheduler/src/lib.rs`
  (export), `crates/scheduler/tests/**` or inline mods.
- T3b: the shared journal; `crates/cli/src/lib.rs`;
  `crates/cli/src/commands/ui.rs`; `crates/cli/src/commands/ui/stage_spans.rs`
  (clamp); `crates/pipeline/src/stage_index.rs` (adaptation; `lib.rs` only if
  the constructor signature genuinely requires it - journal justification);
  new test mods as above.
- Both hops MUST NOT touch: `crates/cli/src/commands/progress.rs` (DP-1);
  any M26 append-only test file or any pre-existing test as oracle (green
  UNEDITED or stop-and-report); shard envelope FIELDS / `ShardCheckpoint`
  (I7); `CURRENT_ANALYSIS_VERSION` (T2's, done); `tests/golden/**`,
  `tests/fixtures/**`; `crates/ui/**`; `bench_baseline.json`; the secondary
  candidates H1'/H3'/DP-4 (shape sec. 3.5: NOT commitments - they need their
  own T0/T1 ranking and a T4-style handoff; discovering an urge here is a
  report, not a diff).

## Acceptance at dispatch (G2-4 summary)
I12 minimized corpus completes with W>1; I1'/I4/I7 green; affected
`boot_stages` phase <= T1 median / K (gate run, distill); M1 tests UNEDITED
green under W>1 (I13); `ui_smoke` PASS; hygiene/encoding exit 0; census via
`testrun.py --distill` (standing errata). Reviewer hop per `review-gate`
skill; extra checks: auto-cap equals the ratified value; `--workers 1`
byte-equals pre-milestone single-thread output; clamp tested through the
REAL mapping fn.

## Report back
Per hop: commit hash + `git show --stat`; red/green canonical lines +
capture paths; K/N/cap values used; I13 W>1 test output line; deviations.
Never paste raw output.

## Sequencing note
T3a may dispatch on the written FAIL note alone (scheduler is disjoint from
everything). T3b requires FAIL + M26 V3 + T3a landed. Never parallel with
any hop touching `crates/cli/src/commands/ui.rs` or
`crates/pipeline/src/stage_index.rs` (M26 V3 by ordering; T4 if it fires -
serialize).
