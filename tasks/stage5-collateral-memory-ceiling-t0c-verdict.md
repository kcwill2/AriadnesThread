# Verdict: stage5-collateral-memory-ceiling T0c (post-fix curve + cap-0 ablation)

- **Verdict: APPROVED**
- Reviewer: reviewer subagent (review-gate skill), 2026-10-06
- Range: base `dc294bb` -> HEAD `e9a4ac1` (commits `f312dba`, `60c88c7`, `bf23e84`, `e9a4ac1`, reviewed oldest-first)
- Artifacts: `tasks/stage5-collateral-memory-ceiling-t0c-handoff.md`, `-t0c-journal.md` (D1-D7), `-t0c-measurement.md`

## 1. Gate evidence (reviewer-fresh, canonical lines verbatim)

| Check | Command (route) | Result line |
|---|---|---|
| Scoped plain pipeline | `python tools/testrun.py --distill -p ariadne-pipeline` | `32 passed / 0 failed / 4 ignored / 7 suites` exit 0 EXACT |
| Scoped instrumented pipeline | `python tools/testrun.py --distill -p ariadne-pipeline -- --features test-instrument` | `35 passed / 0 failed / 4 ignored / 7 suites` exit 0 |
| Workspace census | `python tools/testrun.py --distill` | `1250 passed / 0 failed / 13 ignored / 91 suites` exit 0 |
| UI smoke | `python tools/distill.py -- python tools/ui_smoke.py` | `ui_smoke: PASS` exit 0 |
| bench_compare (default, live) | `python tools/distill.py -- python tools/bench_compare.py` | exit 1, stale-baseline class: `WARN file_pass (event span): 598.5 ms vs baseline 312.5 ms (+92%)`; `ensure_load total | 14787.7` medians recorded honestly |
| Hygiene | `python tools/distill.py --kind generic -- python tools/hygiene_check.py` | exit 0; zero lines in touched files |
| Encoding | `python tools/distill.py --kind generic -- python tools/encoding_guard.py --all` | exit 0, silent |
| Disk witness | `python tasks/t0c_run_invocation.py --disk-witness --min-free-gib 16` | `DISK C: free 162 GiB of 1862 GiB - PASS` |
| Directional re-run | helper `--files 640 -- cargo test --release -p ariadne-cli --test stage5_memory_bench -- --exact stage5_rss_scale_runner --ignored --nocapture --test-threads=1` | exit 0; `stage5 peak RSS (release, N=640) | 2090.4` (vs accepted slot median 2088.8, +0.08%); no witness line -> instrumented binary inert with env unset |

## 2. Touch-set and locks

- `git diff --name-only dc294bb..e9a4ac1` = EXACTLY the seven touch-set paths
  (both Cargo.tomls, `stage_load.rs`, `stage_load_tests.rs`, journal,
  measurement doc, `tasks/t0c_run_invocation.py`). Per-commit `--stat` all
  inside. `Cargo.lock` diff-EMPTY (feature-only dev-dep edge, acceptance
  branch reads EMPTY).
- Absence in range proves simultaneously: bench file UNEDITED,
  `crates/graph/**` + I5 diff-EMPTY, `tools/**` diff-EMPTY,
  `tools/bench_baseline.json` untouched (no `--update-baseline` usurpation),
  ledger untouched. `resolver = "2"` at root `Cargo.toml:22` keeps the prod
  bin feature-unified-free.
- Bench children inherit env (`Command::env`, no `env_clear`) - verified in
  `crates/cli/tests/stage5_memory_bench.rs`; cap-0 routing to the measured
  process is real.

## 3. Anti-fabrication audit (independent recompute from raw captures)

Reviewer probe (`tasks/t0c-review-probe.py`, own regex parser, standalone law
re-implementation) re-derived everything from `target/t0c-*.txt` without the
implementer's helper:

- **Witness census**: `load-memo caps override active: semantic=0 physical=0`
  present in ALL 5 G3 captures (line 68 each), ABSENT in all 6 G1, all 5 G2,
  and the preflight capture. G3 was really run at cap-0; G1/G2 really not.
- **D5 prefix witness**: raw G1-inv3 row literally begins
  `test stage5_rss_growth_ratio ... stage5 peak RSS (release, N=640) | 2363.9`;
  prefix mechanism real, row really was parse-missing pre-fix, and the fix
  moved data INTO the parse (increasing scrutiny: it produced a rejection).
- **Independent law recompute** matches the doc exactly: G1 `REJECT
  invocation 3: release|640 sample 2363.9 vs 2088.8 = 13.2%`, `accepted 5 of
  6`, peak medians `8469.5` / `2088.8`, ratio median `4.1`; G2 5/5
  `8470.9` / `2091.2` ratio `4.0`; G3 5/5 `8479.2` / `2086.7` ratio `4.1`;
  save tails `4371.5` / `4375.6` / `4381.0`.
- **Label census**: unknown-label set EMPTY in every group - no hidden,
  merged, or renamed metric rows after prefix normalization.
- **Implementer helper re-run** on the G1 captures reproduces
  `REJECT invocation 3 ... = 13.2%` / `accepted 5 of 6` and reproduces the
  doc tables line-by-line (incl. memo rows byte-equal to T0a Block 4).
- **Ablation arithmetic** exact: transient landed `4098.0`; transient cap-0
  `4098.2`; memo residual `-9.7`; uncapped-class contribution `560.8`
  (cross-epoch class value, caveated in doc); A7-alone `4371.5 <= 5200`.
- Basis note (disclosed in doc sec. 2, handoff-specified): non-peak medians
  run over ALL invocations; on an accepted-only basis G1's save tail would be
  `4373.2` -> transient `4096.3` - same ~4096 class, no conclusion moves.
- Reviewer's own fresh release N=640 invocation lands in-band (2090.4),
  independently corroborating the accepted G1 half-scale slot.

## 4. Deviation adjudications (independent, on the merits)

- **D1** (`pub(crate)` raise of `build_stage5_analyzer`): ACCEPTED. Inside the
  touch-set file, route region only, private module, T2 sanctioned-raise
  precedent; with the feature off the default branch is the byte-landed path
  (production-invariance witness: the `32/0/4/7` EXACT line, reviewer-fresh).
- **D2** (green-by-design pin at Red): ACCEPTED. A feature-gated route must
  pin default-inertness; the two genuinely-Red tests failed at Red with the
  predicted reasons, the pin line green - journal reads the handoff's
  "three expected RGB lines" consistently with the T0b last-write-wins class.
- **D3/D6** (declared gate-invocation overshoot, final 47 vs ~25-30):
  ACCEPTED. The ratified median-of->=5 protocol floor (5/5/5 + one rejection
  retry) is arithmetically incompatible with the ~10-gate sizing law; the
  T0b-finding-1 law requires itemization, which exists and is internally
  consistent (44 + 3 landing support = 47; declared overshoot sources: 6th G1
  invocation, one void + re-run pass from the helper prefix bug, 12 routed
  support calls). Silence would have been the violation. See finding 4 (relay).
- **D4** (G3 release-only): ACCEPTED. Release is the launch profile per
  AGENTS.md; profile-independence of RSS growth is measured evidence
  (G1 release vs G2 debug: ratio 4.1 vs 4.0, peaks within 1.4 MiB); debug
  cap-0 adds ~20x wall for zero bar data.
- **D5** (`strip_harness_prefix`, own commit `60c88c7` between code and doc):
  ACCEPTED. Measurement-tooling bug fixed before any doc depended on it;
  captures untouched, same captures re-parsed; fix direction verified on raw
  rows (sec. 3); committed scoped and single-file.
- **D7** (frozen-lock proofs): ACCEPTED - all claims independently re-derived
  by reviewer (sec. 2); the journal's command list matches what was run.

## 5. Plan-literal checks

- Env name `ARIADNE_LOAD_MEM_CAPS`, value law `a,b` (non-negative int or
  `max`), loud panic on malformed (incl. empty / third token), once-per-
  process stderr witness line, cfg `feature = "test-instrument"` gating only:
  all verbatim per handoff sec. route; witness string matched verbatim by the
  reviewer probe against raw captures.
- Helper spec flags (`--files N -- cmd`, `--caps`, `--disk-witness
  --min-free-gib`, `--medians`) present and behave as specced (env-setter
  child inherits stdio, exit propagates; `<5 accepted -> GROUP INCOMPLETE`,
  exit 1).
- The optional `<=` pin case at `512,128` was NOT added, and the journal says
  so (handoff makes it conditional on adding it) - compliant.
- DRAFT discipline: measurement doc asserts actuals only, states it is "data,
  not gate verdict"; no bar verdict anywhere in the doc or journal.

## 6. Report-evidence shape (distillation contract)

Journal is digest-shaped throughout: canonical lines, exit codes, capture
paths, zero raw dumps; every gate command routed through `testrun.py
--distill` or `distill.py`. Compliant.

## 7. G-gate grounding (fixture quadratic claim, read-only code + data)

`hub_source` gives each hub ~N/4 round-robin leaf includes plus next/far hub
includes; every leaf includes `hub0.h`; TUs include hub0/hub1 plus three
leaves. Direct include edges are O(N), but every leaf's semantic closure and
physical expansion reach the hub cluster and hence corpus scale: aggregate
retained closure is Theta(N^2). Data witness: `memo semantic-closure est
aggregate refs` 742896 -> 3122416 per 2x scale (factor 4.20) reproduces
byte-identically across all 16 captured invocations, and the measured
save-tail growth 1065.2 -> 4371.5 = 4.10x matches the ratio rows 4.0-4.1.
The growth-ratio-vs-G<=2.0 expectation is dead against this fixture for
reasons the fixture itself dictates - correctly logged as a question for the
evidence review, not adjudicated here.

## 8. Decomposition statement (for the T4/DP-2 evidence review)

The A7-top decomposition holds on the post-fix curve. Cap-0 ablation moves
peak@1280 by -9.7 MiB (8469.5 -> 8479.2 cross-epoch sign-inverted, i.e.
within invocation noise): the two boot-cache memos contribute ~0 to the
full-scale peak once A1/A3/A4 are landed. The save-span transient is
cap-invariant (4098.0 landed vs 4098.2 cap-0) and sits in the save/deserialize
path, not the memo path. Everything above the ~4371.5 MiB save tail is the
resident reduced-model/metrics base, whose 4.1x per 2x scaling is the
fixture's Theta(N^2) closure, not memo growth. So the dominant term at N=1280
is A7 (resident model + file collateral), with a ~4096-class save transient as
the second term and memo as a third term of ~0 - consistent with A7 being the
top target per DP-2, and the 5200 MiB bar reachable only if the A7 base or the
transient is attacked; memo-side levers are dead at full scale.

## 9. Findings (numbered; none blocking)

1. **Informational (planner/ledger relay): census anchor moves.**
   Workspace census is now `1250 passed / 0 failed / 13 ignored / 91 suites`
   (plain form). Mechanism identified from the census capture: the
   `crates/cli/Cargo.toml` dev-dep `ariadne-pipeline`
   `features = ["test-instrument"]` unifies pipeline's new feature into the
   workspace build, and the three `stage_load_tests::caps_env_ablation`
   tests ran in pipeline's lib binary (capture lines 1410/1415/1416). The
   1247 anchor used in reviewer dispatch is stale; update anchors at the next
   ledger touch (same class as T0b finding 2).
2. **Informational (planner/ledger relay): row-72 decision trigger arrived.**
   T0c touched both manifests, and the workspace census now implicitly runs
   SIX feature-gated tests via dev-dep unification (3 graph-counter via the
   ui edge, 3 route tests via the new cli edge). The ledger row 72 deferred
   decision ("explicit allow-list/pin at next tools/manifest touch") is now
   due. Not a T0c violation: the handoff's binding input named this pattern.
3. **Purge-queue relay (row-67 class): reviewer scratch files**
   `tasks/t0b-review-probe.py` and `tasks/t0c-review-probe.py` are untracked
   reviewer scratch (the latter is the only R3-warning source in the hygiene
   capture); operator purge at next cleanup.
4. **Process note (measurement-hop sizing law calibration).** The handoff's
   own ~25-30 invocation estimate is miscalibrated against its mandated
   5/5/5-with-rejection protocol; future measurement hops should estimate
   ~45 or the sizing law should carry a measurement-hop exemption row.
   D3/D6 handling was correct; this is a relay, not a T0c violation.
5. **Record for row 70 / PR-2:** gate-0 disk precondition re-witnessed by the
   reviewer at review time: `DISK C: free 162 GiB of 1862 GiB - PASS`. Row 70
   has discharge evidence; rows 62/69 stay open (the default-mode live run
   above still WARNs `file_pass +92%` against the stale baseline - recorded
   honestly, not re-litigated, per handoff sec. "Groups are NOT gates").

## 10. Disposition

APPROVED. Land state stands: `e9a4ac1` is reviewable as the T0c landing.
Findings 1-5 are relays for the orchestrator/ledger, not implementer action.
The T0c hop discharges its contract: post-fix curve measured under the
ratified sampling law, cap-0 ablation executed with witness-proof, DRAFT
discipline kept, G-gate tension logged as a question, decomposition evidence
ready for the DP-2 evidence review.
