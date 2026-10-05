# T1a journal - M27 tu-index-hang-throughput: RELEASE boot_stages bench + sufficiency evidence

Hop: `tasks/tu-index-hang-throughput-t1a-handoff.md` (implementer, RGB-TDD).
Start revision: `42d4a6e` (M26 CLOSED anchor). Touch-set (hard): this journal,
`crates/pipeline/tests/common/mod.rs`, `crates/pipeline/tests/boot_stages_bench.rs`,
`tasks/tu-index-hang-throughput-t1-measurement.md`. Zero src change.
`load_perf_bench.rs` byte-identical (COPY of its pure generators into
`tests/common/mod.rs`, parameterized 2000/400 per shape sec. 3.3 start values).

## Method decisions (binding-inputs read order followed)

- Shape sec. 3.3 labels taken verbatim; they are T1b's
  `DOCUMENTED_PHASES_BOOT_STAGES` contract - byte-exact, `label | ms`
  grammar mirrored from `payload_tail_bench::print_table` (the grammar
  `bench_compare.parse_bench_table` actually parses; exactly-one-pipe lines
  with non-numeric values raise ParseDrift, so context lines carry no pipe).
- Boot replication via the public surface only (pipeline `src/` untouched):
  rows 1-3 replicate `Pipeline::compute_inputs` (locate+discover,
  BuildDiscoverer + registry marker shaper, StageFingerprints::from_inputs);
  rows 4-8 fresh `Pipeline` instances over temp fixtures; cold = fresh
  artifacts tree; warm rows time construction+stage-load and subtract the
  construction-equivalent cost recomputed as `Pipeline::rehash_inputs`
  (same locate+discover+build+fingerprint walk, no artifact IO) to isolate
  the artifact-load path - method also stated in the bench doc comment.
- W=N = `Pipeline::new(&path, Some(n))`, `n = min(8, available_parallelism)`
  - today's library capability (`Some(workers)`), NOT T3 dispatch. The W=N
  row runs on a SECOND freshly generated fixture: the W=1 cold run wrote
  index artifacts + shards into the first tree, a second cold run needs a
  pristine artifacts tree.
- Debug row: one-TU temp corpus built from numeric byte arrays via
  `String::from_utf8` (pure-ASCII authored source), mirroring the approved
  T0b fixture lines (`tests/fixtures/m27-nonascii-underflow`) exactly:
  `cafe2_cafe2 cafeine = cafe2_cup;` (two U+00E9 in leading word) and the
  line-leading U+20AC punct run with U+00AB mid-run. Post-T2 (landed,
  DP-6 2.1.0) the debug run COMPLETES and prints ms - the label's
  `done-post-T2` branch. The pre-T2 panic line is cited from
  `tasks/tu-index-hang-throughput-t0-evidence.md` sec. 1, never re-captured.

## Launch-profile record (obligation (b)) - facts gathered pre-run

- `AGENTS.md` Build section: "Use `cargo build` to build the project" /
  "Use `cargo build --release` for release builds" - the documented default
  is DEBUG; `--release` is phrased as the exception.
- `tools/ui_smoke.py`: build step is `cargo build --bin tool` (line ~301),
  default `--target-dir target/debug` (line ~95) - the repo's own smoke
  ritual builds AND launches a DEBUG binary.
- `EXTENSIONS.md` + `extensions/ariadne-extension` docs: run lines are
  `cargo run --bin cli --features server` - debug profile (cargo run
  default).
- No Makefile/justfile/`.cargo/config` aliases exist; the only `.ps1`
  (`test-opencode-acp.ps1`) does not build or launch the app.
- E3 (T0 evidence): the user's hang session was DEBUG (overflow-checks ON
  panic). De-facto default profile = DEBUG -> "cheap throughput before any
  machinery" finding recorded in the measurement doc.

## Baseline caveat (orchestrator directive)

No gate in this handoff reads `tools/bench_baseline.json`; the
`bench_compare --bench boot-stages` gate is T1b's, PR-2-bound. Nothing ran
against the stale baseline here (nothing needed REPORT-ONLY treatment).

## Checkpoints

- [x] Binding inputs read (handoff, shape 0/3.3/9, load_perf_bench.rs,
  pipeline lib.rs/stage_index.rs public surface, ui.rs boot path, AGENTS.md,
  ui_smoke.py, distillation spec, T0 evidence, T0b fixture shape).
- [x] Red: bench authored with 6-of-8 labels; Gate 1 run captured the
  missing-labels evidence + release build. Digest `ok (exit 0, 0 warnings
  emitted)`, capture `target/distill-capture.txt` (transcribed before
  overwrite): six rows only - `discover+hash 247.3 | build discovery 1.2 |
  fingerprints 0.8 | include-graph cold 476.3 | tu-index W=1 60792.0 |
  W=N 83490.3`; BOTH warm labels absent = label-completeness FAIL as the
  RGB protocol predicted. The W=N row is already > W=1 in the first run.
  The pre-T2 debug panic was NOT re-captured: T2 landed APPROVED, so the
  handoff's PRE-T2 exit-1 evidence does not exist any more; the debug row
  completes (gate 2 below) and the authoritative panic text is cited from
  `tasks/tu-index-hang-throughput-t0-evidence.md` sec. 1.
- [x] GREEN bug found (run "green-1", pre-refactor): naive
  `Pipeline::rehash_inputs` subtraction produced
  `tu-index warm reuse total (release) | -92.9` - anchor hashed cold-cache
  after the second fixture's page churn while the boot measure hashed warm.
  Fixed in the Refactor pass (below); that run's numbers are not used.
- [x] Refactor + method fix: shared `print_row` + anchored
  `measure_warm_load` (throwaway construct, same-code-path
  `Pipeline::new` anchor, `StageReuse::Loaded` assert), stated in the
  module doc. 3 median runs, each digest `ok (exit 0, 0 warnings emitted)`
  (capture overwritten per-run; all rows in the measurement doc sec. (a)):
  medians 224.6 / 1.0 / 0.7 / 464.3 / 44.3 / 60553.4 / 85681.1 / 3176.2;
  `1 passed; 0 failed` per run; all eight labels byte-exact.
- [x] Gate 2 debug-context row: digest `ok (exit 0, ...)`, capture cited;
  row `tu-index non-ASCII repro TU (debug, panic-pre-T2 / done-post-T2) |
  12.6`, run header `profile=debug debug_assertions=true` - done-post-T2
  branch, exit 0 (correct post-T2; the exit-1 pre-T2 condition is moot).
  Debug row is in NO gate tuple.
- [x] Gate 3 scoped suite: `25 passed / 0 failed / 4 ignored / 6 suites`
  (ignored 2 -> 4 = census +2; pipeline suite green UNEDITED).
- [x] Measurement doc `(a)-(d)` complete incl. verbatim sufficiency lines,
  K=1.0 + N-cap proposals for Q3/Q7, deviations (warm-anchor fix, 5 release
  runs). Bench doc comment carries the committed MEASUREMENT TABLE beside
  the consts (A4 precedent).
- [x] Gate 4 hygiene: `python tools/hygiene_check.py` exit 0 - zero
  findings in the touch-set; output is only pre-existing R3 warn-only rows
  in other crates (R3 does not gate). R1/R2 clean.
- [x] Gate 5 encoding: `python tools/encoding_guard.py --staged` exit 0,
  no output (no mojibake/non-UTF-8 in staged content; all authored files
  pure ASCII).
- [x] Gate 6 commit: `commit_scoped.py` with the four touch-set scopes
  (handoff message verbatim); `git show --stat` = touch-set only.

## Frozen-lock proofs (recorded)

- Pipeline `src/` + `load_perf_bench.rs` untouched: `git diff --stat
  42d4a6e..HEAD` lists touch-set files only (proved at review hop too).
- Suite green UNEDITED: `25 passed / 0 failed / 4 ignored / 6 suites`
  (census +2 ignored: the two new `#[ignore]` benches).
- No committed corpus: the E3-shape TU exists only inside a temp dir built
  in-test from numeric byte arrays; `tests/fixtures/**` untouched.
- Debug row excluded from every gate tuple (bench emits it under its own
  ignored test; T1b's documented set is the eight release labels).


## Invocation budget (handoff: <= 10 distill invocations)

Release bench 5 (Red 1 + invalidated Green 1 + medians 3), debug 1, suite
1, hygiene 1, encoding 1, commit 1 = 10. The two extra release runs are
the R9-style honest cost of the negative-anchor bug found in Green; no
scale halving was needed (every run inside the 600 s gate timeout; bench
wall ~155-190 s).

