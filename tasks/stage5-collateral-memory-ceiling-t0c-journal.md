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

## Commit 1 (gate 11): `f312dba`

`git show --stat` = exactly the six scoped paths (journal, helper,
stage_load.rs, stage_load_tests.rs, pipeline/cli Cargo.toml); `Cargo.lock`
absent; `tools/**` absent. Staged scoped via distill-routed `git add`,
`encoding_guard --staged` exit 0 before the commit.

## Measurement groups (commit-2 data; every run isolated, foreground,
sequential, distill-routed, capture fully read immediately after the run)

Gate 4 preflight (`target/t0c-preflight.txt`, exit 0): release N=1280
`--exact` boot: peak 8476.0, save tail 4380.0 (transient 4096.0),
ensure_load 16.4 s, no os error 112; recompiled graph+pipeline in the
release bench build - direct empirical proof the new dev-dep feature edge
reaches the bench link graph.

G1 release curve, 6 invocations (`target/t0c-g1-release-k.txt`, k=1..6,
all exit 0, 57.98-60.74 s suite wall):
- k1 peaks 8469.3/8471.2 @1280, 2092.1/2089.2 @640; ratio 4.1. ACCEPTED.
- k2 peaks 8465.9/8469.0, 2093.2/2088.1; ratio 4.1. ACCEPTED.
- k3 peaks 8465.7/8465.4, 2086.9/2363.9; ratio 3.6 - the ratio-child half
  operand +13.2% (E1 half-scale inflation class, post-fix recurrence,
  exactly the working-set-pressure mechanism the ratified law exists for).
  REJECTED at gate 8: `REJECT invocation 3: stage5 peak RSS (release,
  N=640) sample 2363.9 vs slot median 2088.8 = 13.2%`.
- k4 8464.9/8471.0, 2087.6/2088.2; ratio 4.1. ACCEPTED.
- k5 8469.6/8467.7, 2089.9/2089.0; ratio 4.1. ACCEPTED.
- k6 8471.3/8473.7, 2088.5/2086.6; ratio 4.1. ACCEPTED.
- Gate 8 G1: `accepted 5 of 6` (`target/t0c-g1-medians.txt`). Medians:
  peak@1280 8469.5, peak@640 2088.8, save tail@1280 4371.5, ratio 4.1,
  ensure_load 13.6 s.

Gate 8 helper bug + fix (D5): the FIRST gate-8 G1 pass read
`accepted 6 of 6` - invalid. Cause: cargo `--nocapture` prints
`test <name> ... ` without a newline, so a test's FIRST printed table row
carries a harness prefix (the growth-ratio test's re-emitted half-operand
row); my parse treated it as a non-peak label, hiding the +13.2% outlier
from the peak-slot rejection. Fixed in the helper (in touch-set) via
`strip_harness_prefix`; gate 8 G1 re-run produced the `accepted 5 of 6`
above; the first pass is void/superseded (counted in D6). G2/G3 parses ran
with the fixed helper (their prefixed first rows are STAGE5_BENCH context
lines - no pipe - so no median values were affected in those groups).

G2 debug curve, 5 invocations (`target/t0c-g2-debug-k.txt`, all exit 0;
k1 208.56 s incl. the debug rebuild, k2-5 209.59-210.56 s - well under the
T0a 361-389 s class post-T2):
- k1 8468.9/8458.5, 2088.6/2089.2; ratio 4.0. ACCEPTED.
- k2 8471.6/8464.1, 2092.0/2091.7; ratio 4.0. ACCEPTED.
- k3 8470.2/8473.1, 2091.2/2088.4; ratio 4.1. ACCEPTED.
- k4 8471.6/8467.0, 2093.1/2091.3; ratio 4.0. ACCEPTED.
- k5 8474.9/8473.4, 2093.9/2090.1; ratio 4.1. ACCEPTED.
- Gate 8 G2: `accepted 5 of 5` (`target/t0c-g2-medians.txt`). Medians:
  peak@1280 8470.9 (release parity: profile-independence reproduced),
  peak@640 2091.2, save tail 4375.6, ratio 4.0, ensure_load 61.1 s
  (T0a was 112.8 s - wall rows for ledger rows 69/72).

G3 release cap-0 ablation, 5 invocations (`target/t0c-g3-cap0-k.txt`, all
exit 0, 65.65-73.29 s; every capture carries the loud witness line
`load-memo caps override active: semantic=0 physical=0` at line 68 - the
harness-process boot; children's witnesses are swallowed by `.output()` as
designed, child-side liveness proven by the ensure_load CPU rows):
- k1 8479.0/8472.3, 2087.3/2086.7; ensure_load 17.1 s (vs G1 13.8).
- k2 8477.1/8479.5, 2087.4/2087.5; ensure_load 16.2 s.
- k3 8485.0/8496.2, 2087.5/2086.7; ensure_load 19.2 s.
- k4 8472.5/8494.7, 2085.1/2086.7; ensure_load 14.2 s.
- k5 8476.4/8488.3, 2083.1/2086.7; ensure_load 15.0 s.
- Gate 8 G3: `accepted 5 of 5` (`target/t0c-g3-medians.txt`). Medians:
  peak@1280 8479.2, peak@640 2086.7, save tail@1280 4381.0, ratio 4.1,
  ensure_load 16.2 s (+2.6 s recompute tax vs G1).
- Witness discipline verified by grep over `target/t0c-*.txt`: present in
  all five G3 captures, absent in every G1/G2/preflight capture.

STOP conditions: none hit. No os error 112 anywhere; no group needed more
than 8 attempts; no padding (k=3 G1 rejection kept loud in the table).

## D5 (helper parse fix): `strip_harness_prefix` added to
`tasks/t0c_run_invocation.py` after commit 1 (post-measurement tooling
fix; measurement captures untouched - the same captures re-parsed); landed
as its own scoped commit between `f312dba` and the gate-12 doc commit.

## D6 final declared gate-invocation count (T0b finding 1 law)

Pre-measurement / support (distill- or testrun-routed):
1-4. ground-truth git probes - 4
5. gate 0 disk witness - 1
6. gate 1 RED testrun - 1
7. gate 2 instrumented testrun - 1
8. gate 3 feature-off testrun - 1
9-10. hygiene_check (report + exit-code confirm) - 2
11. encoding_guard --help (interface probe) - 1
12. git status Cargo.lock - 1
13. git add (scoped staging, commit 1) - 1
14. encoding_guard --staged (pre commit 1) - 1
15. commit_scoped gate 11 (`f312dba`) - 1
Measurement:
16. gate 4 preflight - 1
17-22. G1 invocations k=1..6 - 6
23. gate 8 G1 FIRST pass (void: helper prefix bug; superseded by 24) - 1
24. gate 8 G1 re-run (valid) - 1
25-29. G2 invocations k=1..5 - 5
30. gate 8 G2 - 1
31-35. G3 invocations k=1..5 - 5
36. gate 8 G3 - 1
Closeout:
37. git add (helper-fix staging) - 1
38. encoding_guard --staged (pre helper-fix commit) - 1
39. commit_scoped (helper fix) - 1
40. hygiene_check final (gate 9 rerun) - 1
41. git add (doc+journal staging) - 1
42. encoding_guard --staged (pre gate 12) - 1
43. commit_scoped gate 12 - 1
44. frozen-lock proof probes post-landing (`git diff --name-only` range) - 1+
TOTAL ~44 invocations vs the ~25-30 protocol estimate. Declared overshoot
sources: the G1 rejection demanded a 6th invocation; the helper prefix bug
cost one void + one re-run pass; git probes/staging routed through the
distiller add 12 support invocations. Silence would be the violation; the
overshoot is itemized here per the M28 T0b-finding-1 law.

## Commits (this hop, via commit_scoped.py)

- `f312dba` route + helper + feature chain (gate 11).
- `60c88c7` helper parse fix (D5 `strip_harness_prefix`, scoped single file).
- `bf23e84` measurement doc + this journal (gate 12).
- (this next landing commit records the three hashes; its own hash is the
  HEAD after it, T0a precedent - the journal cannot contain its own hash.)

## Frozen-lock proof commands (run post-commit-2; results in D7)

## D7 (frozen-lock proofs, post-landing, distill-routed)

- `git diff --name-only dc294bb..HEAD` -> EXACTLY the seven touch-set paths:
  `crates/cli/Cargo.toml`, `crates/pipeline/Cargo.toml`,
  `crates/pipeline/src/stage_load.rs`,
  `crates/pipeline/src/stage_load_tests.rs`,
  `tasks/stage5-collateral-memory-ceiling-t0c-journal.md`,
  `tasks/stage5-collateral-memory-ceiling-t0c-measurement.md`,
  `tasks/t0c_run_invocation.py`.
  Consequences proven simultaneously: `crates/cli/tests/stage5_memory_bench.rs`
  UNEDITED (bench mechanism measured untouched); `crates/graph/**` incl.
  `load_caches.rs` + I5 file diff-EMPTY; `tools/**` incl.
  `bench_compare*.py` diff-EMPTY; `tools/bench_baseline.json` diff-EMPTY AND
  untouched (no PR-2 usurpation); `Cargo.lock` diff-EMPTY (the
  empty-or-dev-dep-edge acceptance branch reads EMPTY); `tests/**`,
  `tasks/milestone_ledger.md`, m00/m18, `progress.rs`, boot contract
  diff-EMPTY.
- `git show --stat` per commit = touch-set exactly (verified per commit:
  f312dba 6 paths, 60c88c7 1 path, bf23e84 2 paths).
- Gate 3 (the plain-suite line `32 passed / 0 failed / 4 ignored / 7
  suites` EXACT, exit 0, run AFTER the route landed) IS the
  production-invariance witness; the instrumented gate 2 line
  `35 passed / 0 failed / 4 ignored / 7 suites` proves the +3 route tests.
- D6 count addendum: the landing commit itself adds 3 support invocations
  (stage, `encoding_guard --staged`, commit_scoped) -> final declared total
  47 distill/testrun/commit-routed invocations, itemized above (D3/D6).

## NOT run / NOT done (measurement-hop boundary)

No `--update-baseline`, no gate verdict, no debug ablation (D4), no N>1280,
no T4/DP-2 work, no ledger touches, no fixture touches. Reviewer-hop
obligations (fresh census incl. the +3-or-+0 unification note, one group's
medians recomputed independently, one release invocation re-run, G3
witness-line audit, bench_compare default-mode run expecting the stale-
baseline exit-1 class per relays row 62, `ui_smoke`, `encoding_guard
--all`, commit-vs-touch-set, ledger relays) are the reviewer's, not
implementer gates.
