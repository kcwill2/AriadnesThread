# stage5 collateral memory ceiling -- T0a implementer journal

Task: T0a (`t0a-rss-bench`), milestone `stage5-collateral-memory-ceiling` (M28).
Role: implementer (strict RGB-TDD). Work window: 2026-10-05 ~22:15 UTC to
2026-10-06 ~01:55 UTC (spans midnight).
Rule carried all the way: the bench measures and PROPOSES; it never ratifies
(kickoff DP-4). All gate commands ran through distilling entry points only.

## Inputs read

- `tasks/stage5-collateral-memory-ceiling-t0a-handoff.md` (touch-set, gates,
  prohibitions: no `bench_baseline.json`, no `tasks/milestone_ledger.md`, no
  planning/handoff files, no `git add .`).
- `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`
  (sec. 0 accumulators A1-A7, sec. 3.1 row grammar, sec. 6 six table blocks,
  sec. 7 hygiene and isolation law).
- `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-kickoff.md`
  (DP-4 propose-not-ratify, DP-5 synthetic-fixture-only).
- Evidence format precedent: `tasks/tu-index-hang-throughput-t0-evidence.md`.
- API/behavior facts from `crates/pipeline/src/stage_load.rs` (stage-5 event
  milestones SymbolsAnalyzed / ArtifactsSaved) and
  `crates/graph/src/load_caches.rs` (memo entry-key contracts).

## What was built

`crates/cli/tests/stage5_memory_bench.rs` (994 lines, one new file, R2-safe):

- RSS probe: Windows `K32GetProcessMemoryInfo` via raw
  `extern "system"` FFI (`#[link(name = "psapi")]`, pseudo-handle
  `(-1isize)`); Linux `VmHWM`/`VmRSS` parsed from `/proc/self/status`;
  unsupported platforms return `None` with an explicit skip. CI is
  ubuntu-linux, so the procfs arm is exercised by the workspace run.
- Live (non-ignored) probe test `peak_working_set_bytes_tracks_allocation`.
- Deterministic synthetic hub-heavy fixture generator (DP-5: no real-corpus
  capture; nothing corpus-shaped is committed). Scale from
  `STAGE5_MEM_BENCH_FILES`, defaulting to the PROPOSED N0 = 1280.
- Event-checkpoint RSS tracker over stage-5 progress events
  (prep / symbol-pass end / file-pass tail / save tail / post-return).
- Isolated child-process scale runner: every curve or warm row is produced by
  `current_exe --exact <test> --ignored --nocapture` in a fresh process; the
  in-process runners skip with a no-pipe note when their env vars are unset.
- Row printer enforcing shape sec. 3.1 grammar: table rows carry exactly one
  pipe and one finite float value; context lines carry no pipe.

## Red-Green-Refactor trace

**Red.** The first draft shipped a stub RSS probe (OnceLock static snapshot,
always zero move). The probe test failed for exactly the predicted semantic
reason:

- digest: `### cargo test FAIL (exit 1) ... 0 passed / 1 failed / 0 ignored / 1 suites`
- panic: `peak working set must move by >= 209715200 bytes after touching a
  268435456 byte slab, moved 0 bytes (probe is a stub or not live)`

**Green.** Real probes (both platform arms), fixture generator, checkpoint
tracker, row printer. Probe test then passed through the distiller.

**Refactor 1 (isolation law, semantic fix).** The first curve design ran the
fixture boots in-process and printed rows directly. That produced
contaminated rows -- the half-scale peak printed identical to the full-scale
peak -- because a process's peak working set is lifetime-monotonic and never
decreases. Refactor: every curve/warm producer became an isolated child
process; `stage5_rss_scale_runner` and `stage5_warm_measure` became
orchestrated skippers unless their env vars are set; the growth-ratio test
spawns one child per scale and parses the child's peak row. Re-ran scoped
tests after the refactor; rows separated cleanly (2260 vs 9040 debug).
Contaminated rows were never committed.

**Refactor 2.** Negative-zero normalization in wall-row formatting; capture
line de-duplication in the child re-emitter. Scoped re-run clean.

## Gate digests (verbatim through distillers)

- Baseline cli census at anchor `9385acf`:
  `110 passed / 0 failed / 1 ignored / 5 suites`
- Post-implementation cli census (`python tools/testrun.py --distill -p ariadne-cli`):
  `111 passed / 0 failed / 8 ignored / 6 suites`
  (+1 passed = probe test, +7 ignored = bench tests, +1 suite = new file)
- Scoped bench, non-ignored only (`python tools/testrun.py --distill -p ariadne-cli --test stage5_memory_bench`):
  `1 passed / 0 failed / 7 ignored / 1 suites`
- Debug curve gate, 4 invocations
  (`python tools/distill.py --timeout 3600 -- cargo test -p ariadne-cli --test stage5_memory_bench -- --ignored --nocapture --test-threads=1`):
  all `ok (exit 0, 0 warnings emitted)`, ~361-389 s each. (Invocation 4 was
  added because invocation 3's half-scale row was lost when a hygiene distill
  run overwrote the shared capture before full read -- same hazard as D3.)
- Release curve gate, 4 invocations (`cargo test --release ...` same args):
  all `ok (exit 0, 0 warnings emitted)`, ~86-114 s each.
- Hygiene: `python tools/hygiene_check.py` exit 0 (confirmed via
  `### generic PASS (exit 0)`); R1/R2 clean; no finding names
  `crates/cli/tests/stage5_memory_bench.rs`; pre-existing R3 warnings
  elsewhere untouched.
- Encoding: `python tools/encoding_guard.py --staged` -- see commit section;
  the bench file is pure ASCII by construction.
- Workspace-wide `--baseline` gates were NOT run here: the Handoff Sizing Rule
  reserves them for the orchestrator/fresh session; the census delta to
  reconcile forward is +1 passed / +7 ignored / +1 suite.

## Deviations (each with reason)

- **D1 -- attribution without instrument counters.** The handoff hinted at
  sampling test-instrument counters. The bench instead reports definitional
  entry counts plus stride-32 sampled entry sizes through the public cache
  API. Reason (a): a `test-instrument` dev-dependency would perturb the
  `payload_tail_bench` timing gate, and `Cargo.toml` is outside the touch-set
  anyway. Reason (b): the same no-Cargo.toml constraint made `windows-sys`
  unnameable from the test (it is transitive-only in `Cargo.lock`), so the
  probe uses raw FFI -- the deliberate zero-dependency route.
- **D2 -- medians moved to the evidence layer.** Peak RSS is
  process-lifetime monotonic, so a median-of-N inside one boot is physically
  impossible. The gate runs >= 3 invocations and the medians are computed in
  `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md`.
- **D3 -- release gate ran 4 invocations, not 3.** Invocation 3's distill
  capture was overwritten by a hygiene distill run before it was fully read;
  its N=1280 peak row (9033.7) had already been read and is reported with
  that caveat. Invocation 4 restored three fully-read release invocations.
- **D4 -- block-3 capacity-0 ablation DEFERRED** to the post-T1 T0b rerun
  (the handoff's sanctioned alternative): no capacity knob exists pre-T1 and
  building one would violate the touch-set.
- **D5 -- no real-corpus capture** (DP-5): the fixture is synthetic,
  generated deterministically from the scale at runtime.

## Deliberately NOT done

- No `Cargo.toml` / `Cargo.lock` changes; no changes under any `crates/*/src`;
  no edits to `tasks/milestone_ledger.md`, `bench_baseline.json`, or any
  planning/handoff file (all forbidden or foreign).
- No corpus fixture blobs committed; regeneration is deterministic.
- No new test runs by default beyond the probe: the seven bench tests are
  `#[ignore]`d, so CI cost is one probe test.
- Nothing ratified: every constant in the evidence doc is marked PROPOSED.

## Label set emitted (verbatim; T0b row contract)

**26 table-row labels** (T0a verdict finding 3: the evidence doc's first pass
said 25; the erratum corrects it; the label SET was always byte-correct).
`{p}` is `debug` or `release`; `{n}` is the actual file scale (defaults 640, 1280).
RSS and size rows are MiB; wall rows are seconds; memo rows are counts.
Since the finding-1 rework, `stage5 peak RSS ({p}, N={n})` appears TWICE per
scale in the full-suite run (the scale test's child + the ratio test's
re-emitted operand); T0b must parse peak rows as a multiset per label.

- `stage5 peak RSS ({p}, N={n})`
- `stage5 RSS growth ratio ({p}, N=640->1280)` (the code prints a
  parameterized `N={half}->{n0}`; the literal above is the byte form only at
  the default scale -- parenthetical added per T0a verdict finding 3, for
  T0b's tuple-exactness grep)
- `RSS checkpoint prep ({p}, N={n})`
- `RSS checkpoint symbol-pass end ({p}, N={n})`
- `RSS checkpoint file-pass tail ({p}, N={n})`
- `RSS checkpoint save tail ({p}, N={n})`
- `RSS checkpoint post-return ({p}, N={n})`
- `symbol-pass RSS delta ({p}, N={n})`
- `file-pass RSS delta ({p}, N={n})`
- `save-tail RSS delta ({p}, N={n})`
- `post-return RSS delta ({p}, N={n})`
- `memo semantic-closure entries ({p}, N={n})`
- `memo semantic-closure sampled seeds ({p}, N={n})`
- `memo semantic-closure sampled entry size ({p}, N={n})`
- `memo semantic-closure est aggregate refs ({p}, N={n})`
- `memo physical-expansion entries ({p}, N={n})`
- `memo physical-expansion sampled files ({p}, N={n})`
- `memo physical-expansion sampled entry size ({p}, N={n})`
- `memo physical-expansion est aggregate refs ({p}, N={n})`
- `boot stages 1-4 wall seconds ({p}, N={n})`
- `ensure_load wall seconds ({p}, N={n})`
- `warm deserialize peak ({p})`
- `warm deserialize delta ({p})`
- `warm deserialize wall seconds ({p})`
- `warm report.bin size ({p})`
- `warm report.json size ({p})`

Context lines (no pipe, parser-ignored): `STAGE5_BENCH profile={p}
files={n} symbols={s} file_metrics={n}` and
`STAGE5_BENCH profile={p} warm_fixture_files=40`, plus the orchestrated-skip
notes on `stage5_rss_scale_runner` / `stage5_warm_measure`.

## Commits (this hop, via commit_scoped.py)

- `7c4ddf4` -- bench file only (994 lines, one file).
- `13162f1` -- evidence table + this journal.
- `python tools/encoding_guard.py --staged` ran clean (exit 0) after staging
  and before both commits. Staging was scoped to exactly the three touch-set
  paths; foreign working-tree files (ledger, plans, other handoffs) confirmed
  untouched via `git status --porcelain` routed through the distiller.
- `8d34a3c` -- journal records the first two commit hashes.

## REWORK after T0a verdict REJECTED (finding-by-finding)

### Finding 1 (BLOCKER) -- decomposable growth-ratio row + mechanism

Behavior change, RED-first per the rework brief. The row-contract assertion
(one pipe, verbatim label, finite positive value) now lives in
`emit_and_parse_peak_row`, which the ratio test calls for BOTH operands; it
echoes each child's peak row verbatim before printing the ratio. The ratio
test previously consumed and discarded the child tables (no operands on the
page). Scope: only `stage5_rss_growth_ratio`, its doc comment, and the
`isolated_peak_mib` helper; `run_isolated_runner_child` and the children are
byte-identical, so the operands are measured under identical conditions by
construction (verdict 1c "identify which side diverges" answered below).

Fresh measurements (fixed code), each capture read immediately after its run
because `target/distill-capture.txt` is single-slot and shared (the D3-class
overwrite hazard -- this is the practice that finding 5 codifies; the paths
cited are live-during-run, read-after-run, overwritten by the next distill):

- release standalone ratio: capture `target/distill-capture.txt` (read after
  run; 45.64 s suite-line) -- `stage5 peak RSS (release, N=640) | 2257.5`,
  `stage5 peak RSS (release, N=1280) | 9033.8`,
  `stage5 RSS growth ratio (release, N=640->1280) | 4.0`.
- release suite invocation 1: same capture path (103.69 s line) -- ratio
  operands 2256.7 / 9042.0, ratio 4.0.
- release suite invocation 2: operands 2255.9 / 9040.2, ratio 4.0.
- release suite invocation 3: operands 2259.3 / 9029.5, ratio 4.0.
- debug control, suite form: operands 2258.6 / 9048.7, ratio 4.0 (no debug
  reading moved; no debug erratum).

Mechanism (verdict 1c: OS working-set behaviour, NOT a bench artefact): the
diverging side is the HALF-scale operand. During the original observation
window (2026-10-05 23:55Z to 2026-10-06 01:55Z, concurrent with parallel build
sessions on this box) the ratio test's N=640 child read ~2511 MiB (+11-12%)
while its full-scale operand stayed in band; 9040/2511 prints as 3.6.
Discriminating experiment (scratch, NOT committed -- reverted in the same
session before any commit): a temporary non-emitting clone of the OLD ratio
path (`silent_peak_mib`, scratch diff below) was rebuilt and run against today
conditions: standalone release ratio -> `| 4.0` (old code now agrees with its
operands), and in its suite run the +12% inflation attached to the CURVE
half-scale child (2531.4, out of band) while that run's ratio read 4.0. The
inflation therefore follows whichever short (~1.6 s) release N=640 boot lands
inside a memory-pressure window, not the spawning test. `PeakWorkingSetSize`
is a working-set high-water; a brief process's recorded peak is sensitive to
trim/standby timing under external pressure, while the longer full-scale boot
always settled in band (~9030-9048 across every reading in and out of the
window). This also explains the release invocation-4 +13% outlier and the
debug window's single mild instance (invocation 2 ratio 3.9 vs its curve
quotient 4.00). The printed ratio rows were arithmetically TRUE for the
peaks their children measured; what was missing was the operands -- emitted
now, so every invocation is self-auditing.

Scratch-experiment disposition: `silent_peak_mib` clone + two-line ratio-body
swap; ran two build/run cycles (standalone + suite); fully reverted
(`grep silent_peak_mib` = zero matches at commit time); never staged, never
committed.

Corrected gate-relevant facts (all still PROPOSED): release ratio is 4.0 on
clean conditions (median of 4 decomposed invocations + variant-A control),
not 3.6. The proposed growth cap G (2.0) stands; the proposed protocol gains
one line: release half-scale peak must be taken as median-of->=5 invocations
until the bimodality is characterized (T0b rider), and ratio-operand
divergence beyond ~8% from the per-slot median should reject the invocation.
Wall rows are context-only: the debug control moved `ensure_load` to 71.3 s
(from a 75-113 s band) within one afternoon with RSS rows unchanged -- wall
time is environment-velocity, RSS is not.

### Finding 2 -- D6 (undeclared filename deviation, now declared)

- **D6**: the evidence doc landed as
  `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md`; the handoff
  literal is `...t0-evidence.md`. Per the rework brief the landed file is NOT
  renamed; the canonical path is recorded in the evidence-doc header; the
  planner updates the T0b supplier pointer (`t0b-handoff.md:16`) in parallel.
  Reviewer's finding-2 disposition ("resolve at evidence review") selects the
  pointer-correction branch.

### Finding 3 -- label count

Corrected to **26** in both docs (verified label-by-label against the code
format strings at `stage5_memory_bench.rs:449-486,497-501,569-603` and the
ratio/warm rows; also confirmed by the emitted tables transcribed above).
Journal parenthetical added under the growth-ratio label for the
`N={half}->{n0}` parameterization.

### Finding 4 -- exemplar + 4096 pair set

Erratum E2 recomputes the transient from pairs with BOTH operands transcribed
in the same capture (fixed-code debug control 9047.2-4951.1=4096.1; fixed-code
release suite 9035.7-4939.7=4096.0; earlier debug 9034.4-4938.4=4096.0 and
9033.7-4937.6=4096.1); the untranscribed citations and the partial-invocation
peak are withdrawn; "invocations 1-3" -> fully-read set 1/2/4; the block-2
release exemplar is replaced by fixed-code invocation 1 (in band). The
~4096.0 +/- 0.1 MiB law stands (5th confirmation: debug control this session).

### Finding 5 -- capture-path citations

Adopted going forward (and used throughout this rework): every digest above
cites its capture path; `target/distill-capture.txt` and
`target/testrun-capture.txt` are single-slot and overwritten by the next run,
so the reading discipline is read-immediately-after-run (this rework never
queued a second distill before reading the previous capture -- the exact
omission that caused D3 in the first pass).

### Rework gate digests (verbatim, with capture paths)

- bench compile + probe (`testrun.py --distill -p ariadne-cli --test
  stage5_memory_bench`): `1 passed / 0 failed / 7 ignored / 1 suites`;
  capture `target/testrun-capture.txt` (16 lines).
- scoped cli census (`testrun.py --distill -p ariadne-cli`):
  `111 passed / 0 failed / 8 ignored / 6 suites`; capture
  `target/testrun-capture.txt`.
- release/debug curve forms: five distiller invocations, all
  `ok (exit 0, 0 warnings emitted)`; captures as cited inline.
- `python tools/hygiene_check.py` via `python tools/distill.py --kind generic`
  -> `### generic PASS (exit 0)`; zero rows for
  `crates/cli/tests/stage5_memory_bench.rs`; capture
  `target/distill-capture.txt` (152 lines, all foreign R3 warnings).

### Rework commits (via commit_scoped.py)

- `407fe66` -- bench rework (finding 1a), +28/-27, one file.
- `eec5646` -- evidence errata + this journal rework append.
- (this hash-record line lands in the follow-up scoped commit; no other code
  or doc changed after `eec5646`)

