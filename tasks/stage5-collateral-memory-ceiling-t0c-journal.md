# T0c journal - M28 stage5-collateral-memory-ceiling: post-fix live stage-5 RSS curve + cap-0 ablation ranking

Hop id `t0c-postfix-curve-ablation`. MEASUREMENT hop: decision-support DATA
for the T4/DP-2 evidence review, NOT a gate verdict. Spec:
`tasks/stage5-collateral-memory-ceiling-t0c-handoff.md`. Inputs read in
handoff order: handoff; T0b handoff sec. "RATIFIED gate contract"; T0a
evidence doc (protocol, E1/E2, Blocks 2/4/5, RATIFICATION + ROUND-2
ERRATUM); `crates/cli/tests/stage5_memory_bench.rs` (read UNEDITED);
`crates/pipeline/src/stage_load.rs` + `crates/graph/src/load_caches.rs`
(read; graph FROZEN); `crates/ui/Cargo.toml` :22-26 pattern;
`docs/specs/tool-output-distillation.md`; T1 verdict "Follow-up
dispositions" (cap 0 = pure-recompute ablation lever, LOCKED); T2 verdict
sec. 11 (~8.6-8.7 GiB post-fix N=1280 projection).

## Ground truth at dispatch

- HEAD `dc294bb` (T0b landing) - precondition 3 satisfied; T1 `d6640a2` and
  T2 `e52a9e9` verified ancestors (`git merge-base --is-ancestor d6640a2
  HEAD` -> 0, distill-routed).
- Gate 0 (run FIRST, before any cargo; distill-routed, exit 0, verbatim):
  `DISK C: free 161 GiB of 1862 GiB - PASS`
  (capture: `target/distill-capture.txt`).
- Gate 0 derivation (precondition 1): T0a Block 2 records the N=1280
  save-span transient at ~4.0 GiB profile-independent; the save serializes
  JSON+bincode through `.tmp` files whose serialization buffer IS that
  transient, so the disk-side save is the same order of magnitude; 4x for
  temp + final + fixture + OS headroom = the 16 GiB floor. os error 112
  killed two T1-review attempts at lower headroom (T1 verdict finding 3).
- The witness-line mechanism for G3 (read off the frozen bench): the suite
  form runs `stage5_warm_prepare` standalone in the HARNESS process (its
  `STAGE5_MEM_BENCH_WARM_DIR` is unset -> own-tempdir branch ->
  `cold_boot_fixture` boots stage 5 in-harness). The isolated scale-runner
  children's stderr is captured by `.output()` and only pipe/STAGE5_BENCH
  lines are re-emitted, so the harness-process boot is the only surface
  that can carry the `load-memo caps override active` line into a suite
  capture. G1/G2 runs leave the caps env unset -> no line; G3 sets it ->
  exactly one line per invocation.

## Deviations and declarations

- D1 (route test seam visibility): `build_stage5_analyzer` raised to
  `pub(crate)` so the feature-gated route tests in `stage_load_tests.rs`
  can call the route directly. Inside the touch-set file, route region
  only; zero production-codegen change (private module, no external
  surface); T2 `collect_file_metrics` pub(crate)-raise precedent (file
  header "sanctioned visibility raise"). With the feature off the function
  body is byte-identical to the landed default path.
- D2 (expected-Green pin at Red): `caps_env_absent_keeps_landed_const_caps`
  is green-by-design at Red (the default-inertness pin, the T0b
  last-write-wins class). Only `caps_env_override_bounds_both_boot_caches`
  and `caps_env_malformed_panics_loudly` fail at Red; the handoff's "three
  expected failure lines" reads as the two Red tests' failure lines plus
  the declared green-by-design pin line.
- D3 (gate-invocation count): DECLARED PER T0b FINDING 1 LAW - itemized
  count table at the journal end, deliberately exceeding the sizing law's
  ~10-gate ceiling: the ratified median-of->=5 protocol floor forces
  ~25-30 invocations for a measurement hop. Silence would be the
  violation, not the overshoot.
- D4 (release-only G3): scope decision per handoff sec. "Groups": the
  launch profile per AGENTS.md is release; RSS growth is profile-INDEPENDENT
  per evidence Block 5 (only wall time is profile-dependent); debug cap-0
  would multiply the ~20x-slow debug passes by mass recomputation for zero
  additional bar data.

## Route slice (RGB, commit 1)

RED (gate 1, instrumented scoped form pre-route, distill-routed):
`python tools/testrun.py --distill -p ariadne-pipeline -- --features
test-instrument` -> `### cargo test FAIL (exit 1) [capture:
C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 48 lines]`,
canonical `17 passed / 2 failed / 0 ignored / 1 suites` (cargo fail-fast
stops after the lib binary; the full suite runs at Green). Capture fully
read (48 lines). The three expected RGB lines (verbatim):
- (a) Red: `assertion \`left == right\` failed: ARIADNE_LOAD_MEM_CAPS=0,0
  must reach the semantic-closure cache; actual cache lengths
  semantic=3 physical=2` - fails for the predicted semantic reason (env
  inert, counters exceed 0, assertion names the actual lengths).
- (b) green-by-design at Red (declared, D2):
  `test stage_load_tests::caps_env_ablation::caps_env_absent_keeps_landed_const_caps ... ok`
- (c) Red: `malformed ARIADNE_LOAD_MEM_CAPS must panic loudly (no silent
  zeros, no silent defaults); the route returned Ok - no panic occurred`

GREEN (route = cfg-gated env read + loud panic parse + once-per-process
stderr witness line; default path untouched):
- Gate 2: `python tools/testrun.py --distill -p ariadne-pipeline --
  --features test-instrument` ->
  `### cargo test PASS (exit 0) [capture: target/testrun-capture.txt,
  83 lines]`, canonical EXACT `35 passed / 0 failed / 4 ignored / 7 suites`
  (32 prior + 3 route tests; the optional `<=` pin at `512,128` was NOT
  added - the (b) test already pins the two consts and the admission bound).
- Gate 3 (production-invariance witness): `python tools/testrun.py
  --distill -p ariadne-pipeline` ->
  `### cargo test PASS (exit 0) [capture: target/testrun-capture.txt,
  80 lines]`, canonical EXACT `32 passed / 0 failed / 4 ignored / 7 suites`
  - feature-off prior line byte-identical.
- Gate 9: `python tools/hygiene_check.py` exit 0 (R1/R2 clean; all reported
  lines are pre-existing R3 warns, none in touched files).
- `Cargo.lock` diff-EMPTY: the feature-only dev-dep edge regenerates no lock
  content (acceptance empty-or-dev-dep-edge branch: empty).

## Invocation ledger (incremental; final count table at journal end)

1. gate 0 disk witness (distill generic) - 1
2. ground-truth probes: `git rev-parse HEAD`, `git status --porcelain`,
   `git log --oneline -8`, `git merge-base --is-ancestor d6640a2 HEAD`,
   `git status --porcelain Cargo.lock` (all distill-routed) - 5
3. gate 1 RED testrun - 1
4. gate 2 instrumented testrun - 1
5. gate 3 feature-off testrun - 1
6. hygiene (2 invocations: report + exit-code confirmation),
   encoding_guard --help - 3
