# Kickoff: `stage5-collateral-memory-ceiling`

Key: `stage5-collateral-memory-ceiling`. M-number and dates assigned by
milestone-lifecycle kickoff at registration - do not guess. Planner registers
the ledger row; nothing here edits `tasks/milestone_ledger.md`.
Companion shape: `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`
(governing plan; file:line facts live there, not restated here).
Governing history: `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-{kickoff,shape}.md`
is SPENT - both its milestones (M26 `boot-progress-visibility`,
M27 `tu-index-hang-throughput`) are CLOSED on `main` (ledger rows 2026-10-05).
This is a new topology decision, not a rework of that pair.

## 1. Intent, blast radius, the why

- Intent: make the stage-5 "Analyzing collateral" pass (pipeline load stage +
  its memo caches + its `LoadReport` output) complete on corpora of thousands
  of files without exhausting system RAM, and lock peak memory with a
  machine-checkable oracle so a regression is a failed gate, not a field OOM.
- Blast radius: `crates/graph` (load memo caches), `crates/pipeline`
  (`stage_load.rs` lifecycle), `crates/cli/tests` (RSS bench) + optionally
  `tools/bench_compare.py` (new gate mode), `tools/`+docs only for the
  profile-lever rider, `crates/report` only if the artifact-shape DP answers
  YES. UI assets, `progress.rs`, and the progress-event contract stay untouched.
- The why: field OOM 2026-10-05T21:46Z (E1 below) on `C:\Code\Corpora`
  (5753 collateral files). Stages 1-4 complete, stage 5 dies at 5737/5753
  (~99.7% of the file loop) after using all system RAM. The symptom class is
  superlinear/unbounded accumulation, not one pathological file (a single late
  file cannot explain the system-wide exhaustion; a small 800 KiB allocation
  was merely the messenger). This is the boot-responsiveness series' memory
  sequel: M26/M27 fixed visibility and stage-2 throughput; nothing in this
  repository has ever measured, gated, or bounded stage-5 memory.

## 2. Field evidence (E-rows; motivating measurements, not instructions)

- E1 2026-10-05T21:46Z verbatim symptom lines (user-reported):

  ```
  AriadnesThread Visualizer loading at http://localhost:7742
  [1/5] Loading artifacts... 100%  Include graph ready
  [2/5] Indexing translation units... 100%  Index complete
  [3/5] Reducing semantic model... 100%  Model reduced
  [4/5] Building call graph... 100%  Call graph ready
  [5/5] Analyzing collateral... [5%]  5737/5753 filesmemory allocation of 819216 bytes failed
  error: process didn't exit successfully: `target\debug\tool.exe ui C:\Code\Corpora` (exit code: 0xc0000409, STATUS_STACK_BUFFER_OVERRUN)
  ```

- E2 Same report: "It used up all of the system's RAM." The failed allocation
  is 819,216 bytes (~800 KiB) - small; the process died because the system was
  already exhausted. Consistent with per-file accumulation reaching the cliff
  at loop tail (5737/5753), not with one oversized allocation.
- E3 Exit-code reading: `memory allocation of 819216 bytes failed` is Rust's
  out-of-memory message, and `0xc0000409` is what Rust's abort path
  (`handle_alloc_error` -> `__fastfail`) reports on Windows. This is an
  allocation-failure abort, NOT a stack-buffer-overrun bug.
- E4 Stages 1-4 all completed: the M27 tokenizer fix (byte-vs-char column
  subtraction) holds; this is NOT the E1/E2/E3-era hang/panic incident
  recurring. The counter `5737/5753 files` is the stage-5 `CollateralFile`
  frame (ui.rs per-file mapping) - the death is inside the file-metric pass.
- E5 Profile fact: launch was `target\debug\tool.exe` - the DEBUG profile,
  which the ledger's still-OPEN row "M27 profile-lever landing" (deferred
  findings, last row) records as the de-facto default through EVERY documented
  route. Debug inflates runtime (~20x at TU scale, M27 T1a finding) and
  allocation overhead; M27's E3 lesson is that the profile confounds any
  measurement - every RSS table in this milestone carries debug AND release
  columns. This shape dispositions the profile lever explicitly (sec. 8 DP-1);
  it does not silently assume release is the norm.
- E6 Census/scale context: the same 5753-file corpus is the one M26/M27
  closed against; the boot fixture benches run ~520 files (payload_tail_bench
  header) - an order of magnitude below the failing scale, which is why the
  superlinear growth never showed in any existing gate.

## 3. Scope

In scope:
- Stage-5 memory lifecycle: the two append-only `LoadMemo` caches, the
  per-symbol `LoadMetric` vector, the stage-5 deep clones, and the stage-5
  save/cache tail (all ranked by T0 measurement first - the shape must not
  presume the accumulation site; see shape sec. 1).
- A peak-RSS oracle: RSS-vs-file-count curve bench at scaled fixture, debug
  AND release columns, plus a deterministic CI-side bounded-counter
  characterization; gate placement per DP-3.
- The profile-lever rider hop (DP-1) - cheap, ledger OPEN since M27,
  orthogonal to the fix.
- Conditional: load-artifact payload reshaping (DP-2), dispatched only if T0
  ranks persistence (double serialization / warm-boot deserialize) as a top
  contributor AND the user answers YES.

Out of scope, with reason:
- Stage-2 index throughput, progress/visibility semantics, themes, UI assets:
  closed milestones; frozen floors below.
- The rayon pool shape at boot (boot passes `workers: None`; stage 5 runs on
  the global pool): kept as a T0-recorded context row and a candidate T2
  knob, NOT pre-decided - changing concurrency without the RSS/median tables
  is exactly the measure-before-building violation M27's sufficiency gate
  exists to prevent.
- Real-corpus CI fixtures: `C:\Code\Corpora` cannot be committed; the
  committable curve comes from the scaled synthetic generator (hub-heavy
  knob, shape sec. 6).
- Allocator replacement / jemalloc-mimalloc swaps: out until T0 shows the
  residual accumulation is allocator overhead rather than our own retained
  sets; a bounded-retention fix is the causal route either way.

## 4. Invariants at kickoff level

Full numbered list with oracles is shape sec. 2. Headlines:
- All inherited frozen floors (sec. 5) hold UNEDITED at every commit.
- Fix equivalence oracle: on every locked fixture the post-change pipeline
  produces a `LoadReport` structurally equal (`==`) to the pre-change one,
  frozen as an equivalence suite BEFORE any cache-policy swap.
- New peak-memory invariant (I7): at a ratified fixture scale N0, stage-5
  peak RSS stays under a measured-table bar in BOTH profiles, and the
  RSS growth factor between scale N0/2 and N0 is capped (sub-quadratic
  clause) - bar and cap RATIFIED from the T0 table at evidence review,
  never guessed (mirrors M27's sufficiency-gate discipline).

## 5. Frozen locks inherited from the ledger (each with its proving command)

| Lock | Command that proves it |
|---|---|
| `tests/fixtures/**` byte-frozen (changes need documented reviewer approval) | `git diff --stat <start>..HEAD -- tests/fixtures` EMPTY |
| m00 parity UNEDITED (`tests/integration/tests/m00_facts_path_parity_test.rs`, m00 golden baseline) | `python tools/testrun.py --distill -- --test m00_facts_path_parity_test` + file absent from every diff |
| m18 parity UNEDITED (23/0/0/1) | `python tools/testrun.py --distill -- --test m18_cpp_plugin_parity_test` + file absent from diffs |
| `crates/cli/src/commands/progress.rs` byte-identical | `git diff <start>..HEAD -- crates/cli/src/commands/progress.rs` EMPTY |
| Stage-5 13-frame replay `test_stage_five_milestone_sequence_is_monotonic_and_contained` UNEDITED (`crates/cli/src/commands/ui/progress_contract_tests.rs:141`) | test green + `progress_contract_tests.rs` absent from every diff |
| Boot contract tests APPEND-ONLY; M26/M27 appended tests UNEDITED (I13 lineage) | `boot_progress_tests.rs` + M27-added tests absent from fix-hop diffs; suites green |
| `CURRENT_ANALYSIS_VERSION` frozen at 2.1.0 (M27 row, five dependents) | `git diff <start>..HEAD -- crates/index/src/cache.rs` EMPTY unless a registered DP (DP-2 YES) names the bump |
| `bench_baseline.json` operator-only | no diff in any implementer commit; refresh only via operator `bench_compare.py --update-baseline` on VERIFIED-IDLE (PR-2) |
| Census anchor `1235 passed / 0 failed / 6 ignored / 88 suites` @ `bd4fb1a` | `python tools/testrun.py --distill` reconciles forward from the anchor (baseline-FILE form unrunnable - standing errata) |
| Hygiene + encoding | `python tools/hygiene_check.py` and `python tools/encoding_guard.py --all` exit 0 |

## 6. Acceptance gate for the milestone as a whole

- G-A T0 tables landed and ratified: RSS-vs-file-count curve, debug AND
  release columns, component attribution, warm-boot deserialize row, and a
  written sufficiency statement of which accumulators dominate - before any
  fix hop is sized. (Measure before building; no fix commit predates it.)
- G-B I5 equivalence suite green (frozen first, then policy swap).
- G-C I7 peak-memory gate PASSES at ratified scale in BOTH profiles, and at
  the pre-fix tree the same gate FAILS on at least the debug column (the gate
  must demonstrably discriminate the E1 failure class).
- G-D Existing stage-5 bench (`bench_compare`, stage-5 payload-tail phases)
  within tolerance - bounding memory must not buy it with throughput;
  PR-2 baseline refresh precedes the bench gates.
- G-E All frozen floors diff-EMPTY / UNEDITED per sec. 5 at every hop
  (review-gate commit-vs-touch-set).
- G-F Scoped suites green via canonical `testrun.py` lines; census reconciles
  from the `bd4fb1a` anchor; tools `unittest discover` OK; `ui_smoke` PASS;
  hygiene + encoding exit 0; zero added hex.
- G-G Profile-lever rider (if DP-1 YES): the documented launch route builds
  and runs the RELEASE binary (doc line + smoke-adjacent flag), with its own
  tools unit lock.
- G-H Field acceptance (user-routed, terminal): the user boots
  `tool ui C:\Code\Corpora` via the documented route and stage 5 completes.
  The E1 corpus is the acceptance specimen; fixture gates do not replace it.

## 7. Sequencing verdict (dispatchable vs gated; detail shape sec. 1/8)

- Dispatch immediately: T0a RSS curve bench (cli-tests; measurement, not
  machinery) and T3 profile-lever rider (tools/docs; independent of the fix;
  it shrinks the confound, not the curve).
- Gated on T0: T0b bench-compare gate mode (needs T0a's stable labels), T1
  (graph memo bounding), T2 (pipeline stage-5 lifecycle), T4 (report artifact
  reshape - only on T0 ranking AND DP-2 YES).
- The fix must not presume the accumulation site: candidate sites are ranked
  from code reading (shape sec. 0), but only the T0 table assigns the fix's
  budget. If T0 shows one accumulator dominant, T1/T2 collapse accordingly;
  if the table shows the debug profile alone explains the E1 margin, the
  fix still ships - release-only survival is not sufficiency (the growth
  class is profile-independent), so the rider never substitutes for the fix.

## 8. Open user decisions (DP rows; full text shape sec. 8)

- DP-1: ship the ledger-OPEN profile lever as this milestone's cheap rider
  hop now? (Recommended YES - ledger says worth more than any plausible T3
  win on current data, and it has already survived M27's closeout OPEN.)
- DP-2: is load-artifact payload reshaping in scope (drop/compact the
  per-symbol detail that no UI or boot path consumes), with the artifact
  invalidation mechanism it requires? (Gated on T0 ranking; answer due at
  evidence review.)
- DP-3: RSS gate placement - local gate only, and does a CI-side deterministic
  clause (bounded-counter characterization test, not wall-time RSS) accompany
  it? (Recommended: local RSS gate + CI counter-cap test.)
- DP-4: ratify the peak-memory bar by measured-table method at evidence
  review, and confirm G-H field boot on the E1 corpus as the terminal gate.
- DP-5: may T0 also request an optional real-corpus RSS capture (user reruns
  a labeled bench command on `C:\Code\Corpora`) or is the synthetic curve the
  sole oracle? (Recommended: synthetic mandatory, real-corpus optional
  evidence row - it is the only reading at true failing scale.)
