# Shape (governing plan): `large-corpus-boot-responsiveness` -> TWO milestones

Rework 3 (2026-10-05T04:44Z): folds the CONFIRMED root cause from the live
specimen's self-reported panic (E3, sec. 0) -- T0's diagnosis is landed by
field evidence, T2 gains content, and a data-integrity flag plus a build-
profile question join milestone 2. Supersedes rework 2 in place. Companion kickoff:
`.kilo/plans/1791168710915-large-corpus-boot-responsiveness-kickoff.md`.
Milestones: (M1) `boot-progress-visibility`, (M2) `tu-index-hang-throughput`.
References (read, do not restate): `tasks/milestone_ledger.md` LIVE rows
(`ui-progress-parity` CLOSED row: frozen oracle + its DP-1..DP-3;
M25 `cpp-brace-depth-underflow` CLOSED row; dated 2026-10-04 bench-refresh
ESCALATED row; baseline-file errata row; subagent bash-denial row),
kilo-memory correction `m25.cpp_brace_depth_underflow_closed`,
`tasks/ui-progress-parity-shape.md`, `docs/specs/tool-output-distillation.md`.

## 0. Verified code facts and field evidence

Orchestrator claims VERIFIED this pass (with corrections where noted):

- `crates/pipeline/src/lib.rs:16` `use rayon::prelude::*;` -- confirmed,
  but the rayon machinery lives in the LOAD stage (stage 5): the
  "under rayon"/Mutex comments are ~lines 639-664, the pool builder
  `run_on_configured_pool` 726-741, the value-iterator note ~893-897.
  CORRECTION that changes hypothesis ranking: none of this is on the
  UI-boot stage-2 path. `build_ui_payload` passes `workers: None`
  (`crates/cli/src/commands/ui.rs:314`), and `ensure_index` with
  `workers: None` selects `rebuild_index_single_threaded`
  (`crates/pipeline/src/lib.rs:400-423, 488-532`). At UI boot, stage 2 has
  NO rayon workers and NO Mutex -- it is one thread writing one shard per
  completed TU.
- `crates/cli/src/commands/ui.rs:276-281`: stage-2 message is exactly
  `format!("{done}/{total} TUs")`. `crates/cli/src/commands/index.rs:33-37`:
  `index_progress_cb` renders counts only via `print_progress`. CONFIRMED:
  the current TU is never named anywhere in stage-2 progress.
- `crates/lang-cpp/src/parser.rs:56-70`: the M25 floor guard on
  `handle_brace_close` LANDED (round-2 `+1 guard`). The 2026-10-05 memory
  correction `m25.cpp_brace_depth_underflow_closed` still warns the fix
  "does not fully cover subtraction underflow" and records a runtime panic
  during TU indexing observed on 2026-10-05. Cited as precedent for
  parser-pathological inputs on this exact path -- as history, not proof.
- Material addition found by re-reading the boot path: the UI boot's
  `tokio::task::spawn_blocking` (`ui.rs:786-805`) has NO `catch_unwind`
  and drops the JoinHandle (the background watcher at 636-673 DOES catch
  and log). A panic mid-build therefore leaves: progress frozen at the
  last frame, state stuck `"loading"`, error arm never reached, watcher
  never started, server serving forever with zero build CPU. A dead boot
  is INDISTINGUISHABLE from a hung boot today -- symptom S1 and the E2
  escalation meet at this one hole.

- Tokenizer unit-mismatch CONFIRMED by direct read 2026-10-05 (this
  shape's verified-fact, not a hypothesis): `crates/lang-cpp/src/
  tokenizer.rs:49` tokenizes over `Vec<char>` with `col` advancing per
  CHAR; `flush_word` :311 computes `column: column - word.len()` and
  `flush_punct` :331 computes `column: column - punct.len()` -- BYTE
  length subtracted from a CHAR-COUNTED column, while `end: start + len`
  (:313/:333) correctly uses `word.chars().count()` /
  `punct.chars().count()`; the unit inconsistency is internal to the
  file. Any non-ASCII bytes in a word/punct run near a line start make
  `column < byte_len`: DEBUG (overflow-checks on) PANICS at :311 or :331;
  RELEASE WRAPS SILENTLY to huge columns that flow downstream with no
  signal. Same underflow class as the M25 brace-depth incident
  (`parser.rs`, one-site guards missed the class -- the 2026-10-05
  correction row still warns exactly this).

Field evidence (recorded as motivating measurements):

- E1 2026-10-05T04:11Z: stage 2 ~30% after 90+ minutes on the large
  corpus (5744 TUs). Counter consistent with ~1723/5744 done; mean ~3.1
  s/TU single-threaded. Projection: multi-hour cold boot.
- E2 2026-10-05T04:17Z (ESCALATION): frozen at 1780/5744 for several
  minutes; total CPU ~20%; RAM free; UI server alive, no build CPU.
- E3 2026-10-05T04:22Z (ROOT CAUSE, live specimen self-report): the hung
  session printed
  `thread 'tokio-rt-worker' (55408) panicked at crates\lang-cpp\src\tokenizer.rs:331:21: attempt to subtract with overflow`.
  Per-core Task Manager graph at 04:21 (8-core/16-thread Ryzen 9800X3D,
  ~20% total, 25/31 GB RAM): several cores active, NONE PINNED -- the
  "worker died mid-run, siblings wind down, frozen counter" signature,
  not a mutual deadlock. The thread name is consistent with the boot's
  `spawn_blocking` task thread (blocking-pool threads carry the runtime's
  worker name); the frozen counter fits a panic escaping the build
  closure, which the verified boot hole (no `catch_unwind`, dropped
  JoinHandle) swallows into a permanently-frozen loading row.
  Profile read: the panic proves overflow-checks ON -- the user's run was
  a DEBUG/dev profile build, which also prices into E1's 3.1 s/TU.
- E1/E2/E3 together close T0's question (sec. 6 re-ranking); the dump
  capture protocol (sec. 7) is retained for OPTIONAL post-mortem value
  only -- the panic named file:line itself.

## 1. Topology, inheritance, sequencing verdict

DP-0 RESOLVED by the user: TWO milestones.

- M1 `boot-progress-visibility`: handoffs V1 (cli), V2 (repo),
  V3 (pipeline+cli compile-coupled). No bench gate. Runnable with zero
  dependency on the bench baseline refresh (PR-2 belongs to M2 only).
- M2 `tu-index-hang-throughput`: T0 (hang diagnosis) is LANDED BY FIELD
  EVIDENCE (E3) -- its remaining obligation shrinks to the written report
  (reproduce the panic on a minimized synthetic TU, cite :311/:331, carry
  the decision list). Remaining handoffs: T0b (minimized specimen
  fixture), T1 (measurement bench + gate mode, RELEASE profile, PR-2-
   bound, and the release-already-single-thread sufficiency gate), T2 (the
   now-specified root-cause fix + same-class audit, sec. 3.4), T3
   (throughput wiring per resolved DP-2/DP-3 -- DISPATCHED ONLY IF T1's
   sufficiency gate says release+single-thread is not acceptable; DP-3
   stays the binding policy whenever T3 runs), T4 (conditional, DP-4).
   New user decision opened by E3: DP-6, `CURRENT_ANALYSIS_VERSION` bump
   at T2 landing (sec. 3.4 data-integrity flag).

Inheritance record (M2's kickoff doc must carry it verbatim): M1 landed,
on `main` before M2's T3: frame-1-before-construction; stage-1 sub-events;
`TuStarted` + the path-in-message lock; the boot panic->error bridge;
append-only boot contract tests. M2's parallel index path must EMIT
`TuStarted` too and keep every M1 test UNEDITED and green (acceptance
obligation G2-4) -- M1 is never re-opened.

Sequencing verdict (E3 supersedes the E2-era reading): the specimen
emergency is CLOSED -- the panic self-reported file:line, so nothing
waits on a dump. Milestone 2 priority order is now T0-report (bookkeeping
+ minimized panic repro) -> T1 (RELEASE-profile measurement + launch-
profile record + sufficiency gate) and T2 (fix + audit + fixtures), with
T1 and T2 dispatchable in parallel once the T0 report lands; T3 is
gated by T1's sufficiency verdict, not assumed. M1 dispatches in
PARALLEL and is unchanged in scope: E3 PROVES its I11 lock necessary --
the user's UI sat on a frozen loading row while its own worker lay dead
with a truthful panic line on a console stderr nobody was watching; and
it is M1's path-in-row (`TuStarted`, I10) that will name the unit at
fault the next time anything dies mid-stage. Hard V3-before-T3 ordering
on `crates/pipeline/src/lib.rs` stands.

## 2. Invariants, numbered, assigned, each with its oracle

M1 (visibility):

- I1 Zero analysis-output drift: every artifact, plan output, UI payload
  byte-equals the pre-milestone tree. Oracle: goldens diff-empty; m00
  parity `2 passed` UNEDITED; m18 parity UNEDITED.
- I2 (resolved DP-1, frozen) The stage-5 13-frame replay
  `progress_contract_tests.rs::test_stage_five_milestone_sequence_is_monotonic_and_contained`
  passes UNEDITED at every commit of BOTH milestones.
- I3 (resolved DP-1, frozen) `crates/cli/src/commands/progress.rs`
  diff-EMPTY for both milestones (new frames reuse `format_step_bar` /
  `StepBarRenderer` unchanged).
- I5 Bounded first visible movement: cold boot emits the stage-1 Active
  frame BEFORE `Pipeline` construction completes, and >= 2 distinct
  stage-1 `(percent, message)` states. Oracle: cli builder test with a
  recording `ProgressFn` around a tiny injected repo (today nothing locks
  this -- that absence is S1).
- I6 Terminal/web row parity: `BootStepSnapshot` mirrors every frame; the
  existing `boot_progress_tests.rs` parity oracle green UNEDITED.
- I9 Honest in-span percents: every new frame lands inside its stage's
  `[(k-1)*20, k*20]` and only where work really progressed. Oracle: new
  append-only span test mirroring
  `test_tu_index_events_stay_in_stage_two_span`; message-only progress
  holds percent honestly.
- I10 (binding, from the hang incident) While stage 2 is Active, the row
  message names the current TU's repo-relative path. Oracle: append-only
  contract test -- mapping a `TuStarted { source, .. }` report yields a
  message containing the verbatim relative path; plus the event actually
  fires per TU in `rebuild_index_single_threaded`.
- I11 Dead-boot truth (PROVEN necessary by E3 -- cite in all M1 artifacts):
  a build panic or error sets progress `state = "error"` with the panic
  message; the frozen-loading signature becomes impossible. E3: the user's
  UI served a frozen row while its worker died with a truthful panic line
  on console stderr; only the default panic hook told the truth. Oracle:
  panic-injection test on the boot wrapper (fail-first task fn) asserting
  terminal state error + a committed bar line; no hidden watcher-only
  catch remains (the boot wrapper matches the 636-673 precedent).

M2 (hang + throughput):

- I1' Byte parity between the sequential and the parallel index path
  (existing production-path parity suites) + all M1 I1 locks re-run.
- I4 (resolved DP-2, BINDING) Per-stage percent is completion-COUNT based
  and monotonic non-decreasing under every arrival order, sequential and
  shuffled-parallel, on both consumers. Oracle: table-driven
  shuffled-arrival test through the real mapping fn.
- I7 Shard-store compatibility: post-change shards read by pre-milestone
  code and vice versa; `ShardCheckpoint` fields diff-EMPTY. Envelope
  field changes only via a fresh user DP (none expected). The cache-
  INVALIDATION knob `CURRENT_ANALYSIS_VERSION` is byte-identical UNLESS
  user decision DP-6 answers YES (sec. 3.4 data-integrity flag) -- a YES
  bump is a single commit-local constant change landing WITH the T2 fix,
  not an envelope format change. Oracle: serde round-trip test + struct
  diff; version constant diff checked against the DP-6 answer.
- I12 Cold indexing COMPLETES on the minimized specimen corpus (E3
  reading: the corpus that panicked now finishes), and the offending
  source shape -- non-ASCII inside a word run AND inside a punct run --
  is a locked regression fixture (added via the documented fixtures
  approval path, new sibling dir, never inside `tests/fixtures/basic` /
  `tests/fixtures/expensive`).
- I14 Panic-on-non-ASCII impossible in any profile: tokenize of a line
  whose word/punct runs contain multi-byte characters neither panics in
  debug (overflow-checks on) nor wraps in release; every `RawToken.column`
  stays within `[1, line_char_len + 1]` and `end - start ==
  chars().count()` on the same fixtures under BOTH
  `cargo test -p ariadne-lang-cpp` and `cargo test --release -p
  ariadne-lang-cpp`. This is the shape's falsifiable statement of the
  unit-mismatch kill; the debug panic reproduction (E3 minimized) is its
  RED test.
- I13 (inheritance obligation) M1's appended tests run UNEDITED and green
  under W>1 arrival at M2 closeout.

Hygiene (both): `hygiene_check.py` / `encoding_guard.py --all` exit 0;
zero added hex; authored content pure ASCII.

## 3. Contracts at the changing boundaries

### 3.1 Progress seam (M1, append-only everywhere; resolved DP-1)

```rust
// crates/pipeline/src/lib.rs -- append-only; existing variants untouched
pub enum ProgressEvent {
    TuIndex { done: usize, total: usize },        // unchanged
    CollateralFile { done: usize, total: usize }, // unchanged
    LoadMilestone(LoadMilestone),                 // unchanged
    BootMilestone(BootMilestone),                 // NEW (stage 1)
    TuStarted { done: usize, total: usize, source: std::path::PathBuf }, // NEW (stage 2)
}

pub enum BootMilestone {
    /// Input-computation hashing progressed; total None while the
    /// single-pass walk has not bounded the file set.
    DiscoveryHashed { done: usize, total: Option<usize> },
    /// Include-scan progress of the [1/5] cold rebuild.
    IncludeScan { done: usize, total: usize },
}
```

Mapping contract in `ui.rs` (moved to `ui/stage_spans.rs` in V1 for
hygiene): stage 1 owns [0,20] -- `DiscoveryHashed` onto [0, D] (total None
=> message advances, percent held honestly), `IncludeScan` onto [D, 19],
stage-end frame keeps `(1, 20)`; D calibrated by M2's T1 table when it
lands (A4 precedent, committed measurement table beside the constants;
until then D = 10 with the calibration TODO). Stage 2: `TuStarted` carries
`done` = completed count and `source`; the Active message becomes
`"{done}/{total} TUs  {source}"` (repo-relative verbatim path --
identifiability wins over line length; `index.rs` `index_progress_cb`
adopts the same payload as a small rider). `total_stages` stays 5;
`test_phase_label_stable_within_stage` keeps passing (labels unchanged).
Boot-order contract: `build_ui_payload` first statement is the stage-1
`stage_report(1, 0, ARTIFACTS_PHASE, "Hashing sources")`, then
`Pipeline::new_with_progress`.

```rust
impl Pipeline {
    pub fn new_with_progress(
        repo_path: &Path,
        workers: Option<usize>,
        progress: Box<dyn FnMut(&ProgressEvent) + Send>,
    ) -> Result<Self>; // Pipeline::new delegates with None -- behavior no-op
}
impl Repository {
    /// Additive; per hashed file, (hashed_so_far, discovered_so_far).
    /// discover() delegates no-op. Record ORDER unchanged.
    pub fn discover_with_progress(
        &self, on_hashed: &mut dyn FnMut(usize, usize),
    ) -> Result<RepositoryDiscoveryResult>;
}
```

Boot error-surface contract (V1): the boot `spawn_blocking` closure is
wrapped like the watcher's (636-673 precedent): `catch_unwind` + on panic
write `state = "error"`, `message = panic text`, send `fps_tx.send(None)`;
the JoinHandle is kept and its `JoinError` also routes to the error state.
No UI asset change; the error arm of `updateProgress` already renders
(I2 hop-2 precedent).

Contract-test edits are APPEND-ONLY (resolved DP-1, binding for both
milestones): new tests may be added by M1; the stage-5 replay test,
`emitted_milestones()`' existing rows, and every prior assertion stay
byte-identical; M2 may not edit the file at all (only run it).

### 3.2 Diagnosis artifact contract (M2 T0 -- LANDED by field evidence E3)

`tasks/tu-index-hang-throughput-t0-evidence.md` (tracked) records what E3
already established: the panic trace line, the per-core graph reading
(several cores active, none pinned -- wind-down, not deadlock), the
machine/profile facts (8c/16t Ryzen 9800X3D, overflow-checks ON => debug
dev build), the verified code mechanism (sec. 0: char-counted `col`,
byte-length subtraction at `tokenizer.rs:311`/`:331`, `end` correct at
:313/:333), the M25 same-class precedent, and the hypothesis verdicts
(sec. 6). Its ONE remaining author obligation: a minimized synthetic
RED test that reproduces the `:331` panic in debug on a tiny source
(a word or punct run containing one multi-byte character at line start)
-- authored with T0b/T2, not before. The dump/stack capture protocol
(sec. 7) is OPTIONAL post-mortem only; the counter-and-shard liveness
techniques (sec. 6 discriminators c/e) are retained as reusable T0
methodology for FUTURE stalls, not as outstanding work. T0's decision
list output: T2 = both-site fix + class audit (sec. 3.4); T1 = release-
profile measurement + sufficiency gate (sec. 3.3); DP-6 opened (sec. 3.4);
I11 in M1 stands independent of root cause (truth-surface).

### 3.3 Measurement seam (M2 T1)

New ignored bench `crates/pipeline/tests/boot_stages_bench.rs`, reusing the
`load_perf_bench` generator at a scaled setting (ratify at T1: start 2000
headers / 400 TUs), printing the same `label | ms` grammar. Phase labels
(carry the profile explicitly -- E3 proved the user's 90-minute run was a
DEBUG build, so part of "indexing takes forever" is profile, not
algorithm): `discover+hash total (release)`, `build discovery (contexts)
total (release)`, `stage fingerprints total (release)`, `include-graph
cold total (release)`, `include-graph warm load total (release)`,
`tu-index cold total (W=1, release)`, `tu-index cold total (W=N,
release)`, `tu-index warm reuse total (release)`, plus one
`tu-index non-ASCII repro TU (debug, panic-pre-T2 / done-post-T2)` row
tracking the E3 shape. The DEBUG profile is benched ONCE as the E1
context row, labeled non-gate.
Gate mode: `python tools/bench_compare.py --bench boot-stages`; default
stays `payload_tail_bench` (existing stage-5 gate untouched); new
`DOCUMENTED_PHASES_BOOT_STAGES` tuple; medians under the same per-machine
class in `bench_baseline.json` (new labels = new phase keys). New baseline
rows land ONLY via `--update-baseline` on a verified-idle machine (PR-2) in
the T1 landing commit; implementers never hand-edit it.

T1 evidence obligations added by E3: (a) run the RELEASE-profile column
of the table explicitly (`cargo test --release ... --ignored`); (b)
RECORD how the app is NORMALLY launched/built -- inspect `AGENTS.md`
build lines, the `tools/ui_smoke.py` build step, and any repo-provided
run scripts/aliases -- and state the default profile an actual user gets
(if debug is the de facto norm, that is a routable finding: cheap
throughput before any machinery); (c) the SUFFICIENCY GATE: before any
T3 dispatch, the table must answer whether release+single-threaded
indexing meets the ratified wall-time bar at corpus scale; if yes, T3 is
NOT dispatched (the throughput mandate is honored by measuring, not by
assuming parallelism is needed; DP-3 stays the binding policy for the day
it is).

### 3.4 Root-cause fix seam (M2 T2; content confirmed by E3 -- files, not code)

Confirmed mechanism (sec. 0): `tokenizer.rs` counts `column` in CHARS but
subtracts BYTE length at BOTH flush sites -- `flush_word` :311
(`column - word.len()`) and `flush_punct` :331 (`column - punct.len()`,
the E3 panic site :331:21) -- while the adjacent `end: start + len`
correctly uses `chars().count()`. Non-ASCII bytes in a word/punct run
therefore underflow: DEBUG panics, RELEASE WRAPS SILENTLY to huge column
values that flow downstream unchecked. Same underflow class as M25's
brace-depth incident (`parser.rs`), whose one-site guards the corrections
row still records as not covering the class -- the audit obligation below
exists because that precedent says single-site fixes miss the class.

T2 touch-set sketch (the planner sizes; this is the boundary, not code):
`crates/lang-cpp/src/tokenizer.rs` BOTH flush sites (:311 and :331 -- a
fix that touches one is a plan-literal violation), plus a same-class
AUDIT sweep of every usize subtraction on position math in
`tokenizer.rs` + `crates/lang-cpp/src/parser.rs` (M25 precedent cited in
the handoff), every audit finding either fixed or dispositioned in the
journal. Tests (RED first): the E3 minimized panic repro (:331 shape and
the :311 word shape, non-ASCII in words AND in punctuation runs); I14
release-profile characterization -- assert wrapped-column behavior or
`debug_assert` coverage for both sites under `cargo test --release` so
the silent-wrap path is pinned, not merely gone; unit-level sources via
`include_bytes!`/inline byte escapes (pure-ASCII authored rule), corpus
fixture via T0b (sec. 4) through the documented approval path.

DATA-INTEGRITY FLAG (mandatory disposition text, T2 + closeout): the
user's completed 30% (E1) came from the SAME release-vs-debug mix and any
PRIOR release-profile indexing of corpora containing non-ASCII TUs
carried silently-wrapped columns into shards, index, and stage artifacts
-- those artifacts are SUSPECT CORRUPT, not merely stale; a fingerprint
match will happily reload them post-fix. Disposition: RE-INDEX after T2
lands on every affected corpus. The fingerprint chain hashes CONTENT, not
analyzer CODE, so it will not self-invalidate: the clean invalidation is
the existing `CURRENT_ANALYSIS_VERSION` bump (crates/index) landed WITH
the T2 fix -- but that is a cache-wide version bump and therefore user
decision DP-6 (sec. 11). Candidate validation (NOT a commitment; a
follow-up tool at most): scan shard JSON for token columns exceeding
line length -- wraps are huge, so detection is cheap. The release-
profile characterization test in T2 additionally ensures future wraps are
mathematically impossible rather than merely detectable.

### 3.5 Throughput seam (M2 T3; resolved DP-2/DP-3 binding, CONDITIONAL)

Dispatch precondition: a WRITTEN T1 sufficiency-gate FAIL -- release-
profile, single-threaded indexing at the ratified corpus scale does NOT
meet the wall-time bar. If release+single-thread suffices, T3 is not
dispatched and this seam stays dormant (the DP-2 percent semantics still
apply wherever a pool already exists). The shape below is then the ready
contract, not assumed work.

```rust
// crates/cli/src/lib.rs Commands::Ui gains:
#[arg(long)] workers: Option<usize>,  // None => auto (resolved DP-3)
```

Boot + the spawned watcher build the pipeline with
`Some(resolved_workers)`; `tool ui --workers 1` reproduces today's
single-thread path (escape hatch + A/B). Library default of `Pipeline`
stays `None`-means-single-threaded for all other callers. The parallel
(ProductionAnalyzer) path must: emit `TuStarted` per TU start (last
starter wins, `ArcSwap`-style cell, path + running completion count),
carry completion-COUNT `done` via `fetch_add` on the worker pool (resolved
DP-2; the `collect_file_metrics` `AtomicUsize` precedent at pipeline
lib.rs ~941), and pass every M1 append-only test UNEDITED (I13). Percent
clamp: the stage-span mapping monotonizes per stage, so out-of-order
arrival plateaus but never regresses (I4 table test).

Secondary optimization candidates are NOT commitments; each needs T0/T1
rank: H1' shard serde cost (`tu_store` save path, envelope bytes frozen by
I7), H3' per-TU `fs::canonicalize` hoist in `fingerprint_scoped_context`
(fingerprint bytes asserted identical), DP-4 conditional `crates/repo`
hash parallelism (order-preserving `par_iter`) or `crates/lang-cpp`
per-file parse work.

## 4. Module map (handoff -> crate -> touches -> must-not-touch)

M1 (dispatchable now, parallel with M2-T0):

| Hop | Crate | Touches | Must NOT touch |
|---|---|---|---|
| V1 | cli | `ui.rs` frame-1-first + boot `catch_unwind`/JoinError bridge; extract stage-span constants/mapping to `ui/stage_spans.rs`; builder/boot-progress tests incl. panic-injection | `progress.rs`; contract file beyond append; assets |
| V2 | repo | `discovery.rs` `discover_with_progress` + unit tests (additive, order-preserving) | anything else |
| V3 | pipeline + cli (compile-coupled by exhaustive `match`) | `BootMilestone` + `TuStarted` + `new_with_progress` + emission points (`rebuild_index_single_threaded`, `ensure_include_graph` scan loop, discovery hook wiring); `ui.rs`/`index.rs` message mapping; append-only contract tests; pipeline stage-module extraction (sec. 9) with written justification | goldens, fixtures, `progress.rs`, stage-5 replay rows, assets |

M2:

| Hop | Crate | Touches | Must NOT touch |
|---|---|---|---|
| T0 | none (LANDED by E3) | `tasks/tu-index-hang-throughput-t0-evidence.md` + E3 traces + minimized RED repro authored alongside T2 | ALL source |
| T0b | tests/ | one approved fixture sibling dir: non-ASCII-in-word AND in-punct TUs + corpus-completes characterization | `tests/fixtures/basic` / `.../expensive`; any src |
| T1a | pipeline tests | `boot_stages_bench.rs` RELEASE-profile rows + launch-profile record (shared generator helper; `load_perf_bench.rs` stays < 1000) | src, `load_perf_bench` semantics |
| T1b | tools/ | `bench_compare.py` `--bench boot-stages` + `*_tests.py` + baseline rows via `--update-baseline` (PR-2) | existing `DOCUMENTED_PHASES` semantics |
| T2 | lang-cpp (CONFIRMED by E3) | `tokenizer.rs` :311+:331 fix + class audit incl. `parser.rs` + RED tests + release characterization (+ `CURRENT_ANALYSIS_VERSION` bump only if DP-6 lands YES) | envelope fields, goldens, fixtures (except T0b path) |
| T3 | cli + pipeline (+ scheduler) -- dispatch ONLY on T1 sufficiency-gate FAIL | sec. 3.5 wiring, I4/I13 tests | `progress.rs`, M1 appended tests, envelope |
| T4 | DP-4 (repo or lang-cpp) | conditional, authored post-evidence-review | as scoped at review |

Whole-program must-not-touch: `crates/ui/**` assets + server.rs + dto.rs
(rows generic; no SSE exists), theme assets/baseline, goldens, fixtures
(besides the one T0b approval), `progress.rs`, stage-5 replay rows.

## 5. Risks -> mitigation -> oracle

- R1 (CLOSED by E3) Live specimen loss: moot for the root cause -- the
  worker self-reported file:line before any kill was considered; the
  capture protocol (sec. 7) is retained as reusable methodology.
- R1b (NEW, E3) Silent release-profile corruption already on disk: shards/
  index artifacts from release runs over non-ASCII corpora carry wrapped
  columns and will be RELOADED, not rebuilt, by fingerprint-matching
  post-fix runs. Mitigation: T2 data-integrity disposition (sec. 3.4):
  re-index affected corpora; DP-6 version bump is the clean global
  invalidation; column-vs-line-length scan is a candidate detector only.
  Oracle: after re-index, the affected corpus boots clean AND the I14
  release characterization proves wraps are impossible going forward.
- R2 Stale/escalated bench baseline fires false regressions. Mitigation:
  PR-2 refresh (VERIFIED-IDLE) gates M2 bench only; M1 ships with the
  written no-bench exemption (behavioral no-op proof); T1+ gates run idle,
  one sanctioned protocol-retry with proof. Oracle: quoted median tables.
- R3 Parallel arrival regresses percents/UI rows. Mitigation: resolved
  DP-2 clamp; I4 shuffled-arrival table; I13 M1-tests-green obligation.
- R4 Root cause is inside frozen-ish territory (lang-cpp parser) and
  tempts scope creep. Mitigation: T0 names file:line first, THEN a sized
  T2; DP-4 keeps repo/lang-cpp entry evidence-gated; commit-vs-touch-set
  at review-gate.
- R5 New fixture violates the frozen-set rule. Mitigation: T0b uses the
  documented approval path (reviewer-approved commit + dated ledger note),
  sibling dir only; I12 records it.
- R6 M2 T3 lands before M1 V3 (ordering race on `pipeline/src/lib.rs`).
  Mitigation: hard V3-before-T3 ordering; planner sequence note; review-
  gate touch-set check.
- R7 `TuStarted` message floods terminals (5744 line redraws). Non-risk
  check: single `\r`-line renderer, same rate as today's `TuIndex` events
  (the loop already notifies per TU) -- V3 replaces, not adds, the per-TU
  frame; assert event-count-per-TU stays 1 in the contract test.
- R8 Boot `catch_unwind` masks real bugs. Mitigation: panic text +
  `tracing::error!` mirror the watcher precedent; state=error is surfaced,
  never silent; the root cause still lands in T2.
- R9 Subagent context wall (M23 precedent): all gates via `distill.py`;
  bench scale bounded; dump text excerpts, never raw dumps, in journals.

## 6. Hang hypotheses -- FINAL ledger after E3 (re-ranked, evidence-decided)

- H-B CONFIRMED = ROOT CAUSE (was rank 2): the boot task DIED from a
  swallowed panic at `tokenizer.rs:331:21` (E3); the UI serves a frozen
  last frame while siblings wind down. The E3 core graph (several cores
  active, NONE pinned) matches "worker died mid-run", and excludes mutual
  deadlock. The underlying arithmetic bug (byte-vs-char column
  subtraction, sec. 0) is T2's confirmed content; the SWALLOW that made
  it look like a hang is M1 I11's confirmed necessity. The hang was a
  CONSEQUENCE, never its own bug -- the milestone scope is retitled
  accordingly in both artifacts.
- H-A (one pinned core; pathological TU traps the parser thread):
  KILLED for this incident (no pinned core; the thread no longer
  existed). Retained as the standing signature for a FUTURE one-core
  stall; the parser-pathological class is still live in the wild (M25
  history; T2's audit narrows it).
- H-C (genuinely slow): PARTIALLY CONFIRMED as a throughput contributor,
  not a hang -- E1's 3.1 s/TU was measured in DEBUG profile (E3 proves
  overflow-checks ON), inflating it an unknown factor; whether release+
  single-thread is fast enough is exactly what T1's sufficiency gate
  measures before any parallelism machinery (DP-3) is dispatched.
- H-D (blocking I/O / Defender storm): KILLED for this incident (trace
  supersedes); technique retained for future zero-CPU stalls.
- H-E (Mutex convoy): KILLED twice over -- no Mutex on this path at UI
  boot, AND the path is single-threaded.

Reusable discriminator methodology (for the NEXT stall; NOT outstanding
work for this one): (a) per-logical-core CPU graph -- one pinned core vs
all idle; (b) counter-advance poll (row sampled twice, 5 min apart); (c)
shard-write liveness -- `artifacts\shards` newest-mtime + count sampled
twice (every reanalyzed TU writes a shard: a free progress oracle the
bug report never used; on warm runs shard COUNT exceeds `done` because
reused TUs count done without rewriting, so only MTIME liveness
discriminates); (d) thread stacks + `!runaway` CPU deltas (sec. 7); (e)
console/stderr capture -- E3 proves the default panic hook is today's
ONLY error channel, which M1 I11 replaces.

## 7. Specimen capture protocol (OPTIONAL post-mortem; demoted from PR-0 by E3)

E3's self-reported panic closed the diagnosis without a dump; nothing
below blocks any handoff. Retained verbatim as the standing procedure for
a future stall whose thread DOESN'T self-report (Windows). If the user's
session is still hung at routing time, capture step 3 (dump + stacks) is
still worth taking for post-mortem polish -- but conclusions rest on E3.

Do NOT kill anything until steps 1-4 artifacts exist.

1. Liveness: `Get-Counter '\Processor(_Total)\% Processor Time' -SampleInterval 2 -MaxSamples 15`
   plus Task Manager > Performance > per-core graph (or
   `Get-Counter '\Processor(*)\% Processor Time'`). Save text output.
   Discriminator: one pinned core (H-A pattern) vs near-zero everywhere
   (dead-worker / H-D pattern).
2. Shard liveness, two samples 5 min apart:
   `(Get-ChildItem <repo>\artifacts\shards -Filter *.json).Count` and
   `Get-ChildItem <repo>\artifacts\shards | Sort-Object LastWriteTime -Descending | Select-Object -First 8 Name,LastWriteTime`.
3. Thread evidence BEFORE kill, in order of preference:
   - `procdump -ma <pid> <path>\ui-boot-hang.dmp` (Sysinternals; full
     dump; non-invasive), THEN offline analysis:
     `cdb -z ui-boot-hang.dmp -c "!runaway; ~*e k; q" > stacks.txt`.
   - Or live attach: `cdb -p <pid> -c "!runaway; ~*e k; qd"` (ATTACH MUST
     DETACH with `qd`, never `q`), and repeat `!runaway` 60 s apart for a
     CPU-delta read of WHICH thread burns time.
   - Fallback without debuggers: WER `procdump -h` hang trigger + Task
     Manager thread-level CPU column sorted per thread, screenshotted.
   - Also: capture the CONSOLE/stderr first if a panic line is already
     there -- E3 shows that beats every dump for speed and precision.
4. Scene record: last visible progress count, exact `tool ui` command line,
   repo path, corpus TU count, the two shard listings, artifacts tree
   listing, OS + Defender status, the stacks/`!runaway` outputs.
5. ONLY THEN kill/restart the session. Preserve the dump outside the
   corpus (it survives closeout; reference its path in the incident's
   evidence doc -- do not paste raw dumps into journals, excerpts only, R9).

Offender-TU identification (still useful for the fixture, deterministic
first): the single-threaded loop processes `build.translation_units` IN
ORDER and writes one shard per reanalyzed TU, so the panicking unit is
(first-try) the first TU in that order whose shard file is absent or
stale-fingerprinted, cross-checked as the successor of the newest-mtime
shard; on mismatch fall back to binary search by pruning TU sets in a
repo copy until the panic (not a hang -- E3) reproduces at half size.
The panic message itself carries no path today -- sec. 6(e) + I10 exist
so the NEXT one names its file unaided. T0b then extracts the minimized
corpus as the regression fixture (frozen-set documented approval path,
sec. 5 R5).

## 8. Gates

M1: G1-1 scoped `cargo test -p ariadne-cli` / `-p ariadne-repo` /
`-p ariadne-pipeline` exit 0 (canonical `testrun.py` lines); G1-2 locks
table diff-EMPTY (incl. DP-1 rows, m00/m18 UNEDITED); G1-3 oracles I5/I6/
I9/I10/I11 green as new tests, parity tests UNEDITED; G1-4
`python tools/ui_smoke.py` PASS (distill-routed); G1-5 bench NOT RUN,
written exemption (stage-5 producers unchanged; `Pipeline::new` delegates
a behavioral no-op); one informational `bench_compare` run may be recorded
if a reviewer asks, non-blocking while the stale-baseline row stands.

M2: G2-1 T0 report doc landed (E3 evidence in, minimized RED repro
authored with T2) -- no longer blocks T2, whose content is confirmed;
G2-2 T1: RELEASE-profile bench + gate mode exit 0, pre-fix median table
+ debug-context row + launch-profile record, `bench_compare` (stage-5)
clean vs PR-2 refreshed baseline, sufficiency verdict written
(release+single-thread acceptable? -> T3 dispatched or not); G2-3 T2
(CONFIRMED content): E3-repro RED tests green for BOTH flush sites, I14
release+debug characterization green, class-audit findings dispositioned,
data-integrity disposition executed per DP-6 answer (version bump or
documented re-index instruction), m00/m18 UNEDITED (fix must not alter
ASCII-path tokenization bytes -- if the corrected column value DIFFERS
in any stored output, goldens/parities catch it: expected outcome is
identical output for pure-ASCII corpora, corrected values only for
non-ASCII position fields); G2-4 if T3 dispatched: I12 completes + I1'/
I4/I7 green + affected `boot_stages` phase <= T1 median / K (K ratified
at the evidence review) + M1 tests UNEDITED green under W>1 (I13) +
`ui_smoke` PASS; G2-shared: tools `unittest discover` OK, hygiene/
encoding exit 0, zero added hex, census via `testrun.py --distill`
(standing baseline-file errata).

## 9. Hygiene notes

- `crates/pipeline/src/lib.rs` 959 lines -- AT the ceiling. The V3 commit
  (first pipeline src touch) extracts `src/stage_index.rs` /
  `src/stage_load.rs` verbatim-move with `lib.rs` re-exports (I1-locked);
  T3/T4 then add into the extracted modules, not `lib.rs`.
- `crates/cli/src/commands/ui.rs` 839 lines -- V1 extracts the stage-span
  constants/mapping to `ui/stage_spans.rs` BEFORE V3 grows the file.
- `load_perf_bench.rs` 866 lines -- do not grow it for T1; share the pure
  generator functions into the new bench (duplicate or `tests/common`
  helper), keeping both files < 1000.
- T0b characterization + I4/I10/I11 tests are small files; no existing
  function nears 100 lines on any touched path except via extractions,
  which must keep every extracted fn < 100.

## 10. Ready-to-register ledger rows (kickoff runners commit them, one per milestone)

| Milestone | Status | Where (commit/branch) | Frozen locks | Notes |
|---|---|---|---|---|
| Boot progress visibility (key `boot-progress-visibility`; M-number + dates assigned by milestone-lifecycle kickoff - do not guess) | PLANNED (kickoff+shape `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-{kickoff,shape}.md` rework 3; handoffs V1 cli / V2 repo / V3 pipeline+cli; dispatchable immediately, parallel to `tu-index-hang-throughput`) | plan artifacts; `main` | `tests/golden/**` + `tests/fixtures/**` byte-locked; `m00`/m18 parity UNEDITED; `crates/cli/src/commands/progress.rs` byte-identical + stage-5 13-frame replay UNEDITED (resolved DP-1 frozen floor); boot contract tests APPEND-ONLY; `crates/ui/**` + theme assets untouched | Binding new locks: stage-2 Active row names current TU path (I10); panicked boot surfaces state=error, never frozen loading (I11 -- PROVEN necessary by E3 2026-10-05: user's UI sat on a frozen row while its worker died with a panic line on stderr nobody was watching). Gates: scoped suites + ui_smoke + hygiene/encoding; NO bench gate (written exemption; PR-2 bench refresh belongs to the throughput milestone only). Pipeline touch justified: events + `new_with_progress` + stage-module extraction (959-line ceiling) |
| TU-index panic-consequence + throughput (key `tu-index-hang-throughput`; M-number + dates assigned by milestone-lifecycle kickoff - do not guess; retitled: the hang was a CONSEQUENCE of a swallowed panic, never its own bug) | PLANNED, T0 LANDED BY FIELD EVIDENCE (same plan pair rework 3; root cause CONFIRMED by E3 self-report: `crates/lang-cpp/src/tokenizer.rs:331:21 subtract with overflow` -- byte-vs-char column subtraction at :311/:331, debug panics / release wraps silently; handoffs: T0 report bookkeeping + minimized RED repro / T0b approved specimen fixture (non-ASCII in word AND punct runs) / T1 RELEASE-profile bench+gate+launch-profile-record+sufficiency-gate (PR-2-bound) / T2 CONFIRMED fix: both flush sites + same-class usize audit over tokenizer.rs+parser.rs (M25 precedent) + release-profile characterization / T3 workers per resolved DP-2+DP-3 DISPATCHED ONLY ON T1 sufficiency-gate FAIL / T4 conditional DP-4 OPEN; new user decision DP-6: `CURRENT_ANALYSIS_VERSION` bump at T2 landing) | plan artifacts; `main` | All rows of the visibility milestone's lock table (inherited); PLUS shard envelope fields + `CURRENT_ANALYSIS_VERSION` byte-identical EXCEPT the DP-6 decision; visibility milestone's appended tests UNEDITED and green under parallel arrival (acceptance obligation I13); `bench_baseline.json` touched only by operator `--update-baseline` on VERIFIED-IDLE (PR-2) and T1 landing | Motivating field evidence E1-E3 2026-10-05 (30% at 90 min; frozen 1780/5744; E3 panic self-report + per-core graph: several cores active none pinned = died-mid-run, not deadlock; overflow-checks ON proves DEBUG profile -- part of the slowness is profile, not algorithm). DATA-INTEGRITY DISPOSITION: artifacts/shards from RELEASE runs over non-ASCII corpora (incl. the completed 30%) carry silently-wrapped columns and are RELOADED not rebuilt post-fix -- re-index affected corpora after T2; DP-6 bump is the clean invalidation; column-vs-line-length scan is a candidate detector only. M2 kickoff inherits the T0 report as landed evidence. Binding decisions: DP-2 completion-count percent + monotonic clamp; DP-3 `tool ui --workers` auto default, watcher inherits. Hypothesis ledger closed in shape sec. 6 (H-B confirmed; H-A/H-D/H-E killed; H-C = throughput contributor measured by T1) |

## 11. Open questions (nothing else is presumed)

1. DP-6 (NEW, from E3): bump `CURRENT_ANALYSIS_VERSION` with the T2 fix?
   RECOMMENDED YES -- it is the only reliable invalidation of silently-
   wrapped release-built shards (fingerprint hashes content, not analyzer
   code); a NO obliges a named per-corpus re-index route for every
   affected artifact set. The column-vs-line-length shard scan stays a
   candidate detector either way, not a commitment.
2. Does T3 run at all -- answered by T1's release+single-thread
   sufficiency gate, not by preference (DP-3 then governs the policy).
3. Improvement target K and worker setting N -- ratified from T1's
   RELEASE-profile median table (debug numbers recorded as context only).
4. How the app is NORMALLY launched/built (debug vs release default) --
   recorded by T1 from AGENTS.md / ui_smoke build step / repo run scripts;
   if debug is the de facto norm, that is itself a finding to route
   (cheap throughput without any machinery).
5. DP-4 repo-hash parallelism / lang-cpp parse scope -- decided at the
   T1 evidence review, per the ledger row.
6. Minimized fixture approval (non-ASCII-in-word AND in-punct corpus) --
   routed to the reviewer at T0b per the documented frozen-set path.
7. Exact auto-workers cap -- ratified at T3, only if T3 fires.
8. Whether `tool index` CLI keeps the path-carrying bar (rider assumed
   included; drop silently if the reviewer objects -- behavior-neutral).

CLOSED questions (were open pre-E3, now answered by evidence): T2 content
(both flush sites + class audit, confirmed); the hang mechanism (H-B,
confirmed, ledger in sec. 6); PR-0 specimen urgency (panic self-reported;
protocol demoted to optional post-mortem).
