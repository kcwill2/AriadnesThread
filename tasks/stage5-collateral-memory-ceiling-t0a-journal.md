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

`{p}` is `debug` or `release`; `{n}` is the actual file scale (defaults 640, 1280).
RSS and size rows are MiB; wall rows are seconds; memo rows are counts.

- `stage5 peak RSS ({p}, N={n})`
- `stage5 RSS growth ratio ({p}, N=640->1280)`
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

- (recorded below after creation)
