# T0c measurement - post-fix live stage-5 RSS curve + release cap-0 ablation ranking

Hop `t0c-postfix-curve-ablation` (M28 `stage5-collateral-memory-ceiling`).
MEASUREMENT DATA for the T4/DP-2 written evidence review - NOT a gate
verdict. The `--bench stage5-memory` GATE still waits for operator PR-2;
nothing here asserts one. Sampling law: the RATIFIED median-of->=5 isolated
-invocation protocol with >8% strict per-slot-median rejection on BOTH peak
operands (both scales, symmetric, loud accounting - T0b handoff sec.
"RATIFIED gate contract"). All numbers are transcribed from the cited
captures (each fully read immediately after its run per the evidence-doc
sec. E1 law); every run was distill-routed.

## Headline table (accepted medians + rejected-sample accounting + ablation)

| label (bench grammar, MiB unless wall/ratio) | G1 release | G2 debug | G3 release cap-0 |
| --- | --- | --- | --- |
| stage5 peak RSS (N=1280) | 8469.5 | 8470.9 | 8479.2 |
| stage5 peak RSS (N=640) | 2088.8 | 2091.2 | 2086.7 |
| stage5 RSS growth ratio (N=640->1280) | 4.1 | 4.0 | 4.1 |
| RSS checkpoint prep (N=1280) | 171.5 | 173.9 | 171.8 |
| RSS checkpoint symbol-pass end (N=1280) | 3515.6 | 3517.7 | 3520.4 |
| RSS checkpoint file-pass tail (N=1280) | 4372.0 | 4375.6 | 4380.5 |
| RSS checkpoint save tail (N=1280) | 4371.5 | 4375.6 | 4381.0 |
| RSS checkpoint post-return (N=1280) | 4371.5 | 4375.6 | 4381.0 |
| RSS checkpoint save tail (N=640) | 1065.2 | 1068.0 | 1063.3 |
| ensure_load wall seconds (N=1280) | 13.6 | 61.1 | 16.2 |
| boot stages 1-4 wall seconds (N=1280) | 3.1 | 13.7 | 3.1 |
| accounting | accepted 5 of 6 | accepted 5 of 5 | accepted 5 of 5 |

Rejected-sample accounting (never silent):
- G1 invocation 3 REJECTED: `stage5 peak RSS (release, N=640)` sample
  2363.9 vs slot median 2088.8 = 13.2% (>8% strict, ratio-test child; the
  evidence-doc E1 half-scale inflation class recurring post-fix; its
  growth-ratio row read 3.6 while every accepted invocation reads 4.1).
- G2: zero rejections. G3: zero rejections.

Ablation ranking rows (release, N=1280 medians; `t0c`-suffixed labels so
they can never be confused with the 26-label gate tuple):

```
t0c transient landed (release, N=1280)          | 4098.0
t0c transient cap-0 (release, N=1280)           | 4098.2
t0c memo residual (release, N=1280)             | -9.7
t0c memo contribution uncapped (release, N=1280)| 560.8
t0c resident after return landed (release, N=1280) | 4371.5
t0c resident after return cap-0 (release, N=1280)  | 4381.0
t0c peak bar gap (release, N=1280 vs 5200)      | 3269.5
```

## 1. Provenance

- Start rev: `dc294bb` (T0b landing; T1 `d6640a2` and T2 `e52a9e9` verified
  ancestors). Hop commits: `f312dba` (route + helper + feature chain),
  `60c88c7` (medians-parse fix, journal D5), `bf23e84` (this document +
  journal). The document's own landing record commit follows `bf23e84` and
  is enumerated in `tasks/stage5-collateral-memory-ceiling-t0c-journal.md`
  (a document cannot contain the hash of its own commit; journal practice
  per the T0a precedent).
- Gate 0 disk witness (verbatim, exit 0, run before any cargo):
  `DISK C: free 161 GiB of 1862 GiB - PASS`
- STAGE5_BENCH context lines (per group, from each capture; all invocations
  printed the identical trio):
  - `STAGE5_BENCH profile=release files=1280 symbols=5024 file_metrics=1280`
  - `STAGE5_BENCH profile=release files=640 symbols=2464 file_metrics=640`
  - `STAGE5_BENCH profile=debug  files=1280 symbols=5024 file_metrics=1280`
    (and the debug 640 twin; spacing as printed)
  - preflight: `target/t0c-preflight.txt` (release N=1280 boot, exit 0,
    peak 8476.0, save tail 4380.0, `ensure_load wall` 16.4 s).
- Caps-witness citation (anti-fabrication): `load-memo caps override
  active: semantic=0 physical=0` is present in EVERY cited G3 capture,
  exactly once per invocation (the harness-process boot via the in-suite
  `stage5_warm_prepare` branch; the isolated children's stderr is captured
  by `.output()`, so the harness line is the only surface that can carry
  the witness - see journal):
  - `target/t0c-g3-cap0-1.txt` (line 68), `target/t0c-g3-cap0-2.txt` (68),
    `target/t0c-g3-cap0-3.txt` (68), `target/t0c-g3-cap0-4.txt` (68),
    `target/t0c-g3-cap0-5.txt` (68).
  - Absent (verified by grep over `target/t0c-*.txt`): every G1, G2, and
    preflight capture - exactly the landed-curve expectation.
  - Child-side liveness corroboration: G3 `ensure_load wall seconds
    (release, N=1280)` (full-scale CHILD rows) read 14.2-19.2 s vs the G1
    range 12.8-13.9 s - the pure-recompute CPU tax inside the env-inheriting
    children, with the memo-entry rows unchanged (demand counts, not cache
    lengths).
- VERIFIED foreground/idle statement: every group invocation ran
  sequentially in the foreground of this single implementer session with
  `--test-threads=1`; no other cargo/test process was started by this hop
  concurrently; groups were never parallelized (isolation law). Machine:
  the operator's win32 workstation, disk freed 2026-10-06 (not claimed
  VERIFIED-IDLE - the protocol requires that only for baseline writes, and
  no baseline was touched).
- Captures: preflight `target/t0c-preflight.txt`; G1
  `target/t0c-g1-release-{1..6}.txt`; G2 `target/t0c-g2-debug-{1..5}.txt`;
  G3 `target/t0c-g3-cap0-{1..5}.txt`; law passes
  `target/t0c-g1-medians.txt`, `target/t0c-g2-medians.txt`,
  `target/t0c-g3-medians.txt` (helper `tasks/t0c_run_invocation.py
  --medians`, distill-routed).

## 2. Per-group per-label tables (per-invocation values + median)

Peak slots carry two honest samples per invocation (curve-test child +
ratio-test child re-emit, multiset law). Median basis: ACCEPTED
invocations for peak slots, ALL invocations elsewhere (ratified law).

### G1 release curve (caps 512/128 landed; accepted 5 of 6)

| label (release) | inv1 | inv2 | inv3 | inv4 | inv5 | inv6 | median |
| --- | --- | --- | --- | --- | --- | --- | --- |
| stage5 peak RSS (N=1280) sample a | 8469.3 | 8465.9 | 8465.7 | 8464.9 | 8469.6 | 8471.3 | 8469.5 |
| stage5 peak RSS (N=1280) sample b | 8471.2 | 8469.0 | 8465.4 | 8471.0 | 8467.7 | 8473.7 | (above) |
| stage5 peak RSS (N=640) sample a | 2092.1 | 2093.2 | 2086.9 | 2087.6 | 2089.9 | 2088.5 | 2088.8 |
| stage5 peak RSS (N=640) sample b | 2089.2 | 2088.1 | 2363.9 | 2088.2 | 2089.0 | 2086.6 | (above) |
| stage5 RSS growth ratio (N=640->1280) | 4.1 | 4.1 | 3.6 | 4.1 | 4.1 | 4.1 | 4.1 |
| RSS checkpoint prep (N=1280) | 171.6 | 171.1 | 172.0 | 171.9 | 171.4 | 171.1 | 171.5 |
| RSS checkpoint symbol-pass end (N=1280) | 3514.5 | 3514.7 | 3514.5 | 3516.6 | 3517.1 | 3518.0 | 3515.6 |
| RSS checkpoint file-pass tail (N=1280) | 4373.2 | 4369.9 | 4370.9 | 4368.9 | 4373.5 | 4374.7 | 4372.0 |
| RSS checkpoint save tail (N=1280) | 4373.2 | 4369.9 | 4369.6 | 4368.9 | 4373.5 | 4375.3 | 4371.5 |
| RSS checkpoint post-return (N=1280) | 4373.2 | 4369.9 | 4369.6 | 4368.9 | 4373.5 | 4375.3 | 4371.5 |
| symbol-pass RSS delta (N=1280) | 3342.9 | 3343.6 | 3342.4 | 3344.8 | 3345.7 | 3347.0 | 3344.2 |
| file-pass RSS delta (N=1280) | 858.7 | 855.2 | 856.5 | 852.2 | 856.4 | 856.7 | 856.5 |
| save-tail RSS delta (N=1280) | 0.1 | 0.0 | -1.3 | 0.0 | 0.0 | 0.5 | 0.0 |
| post-return RSS delta (N=1280) | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| RSS checkpoint prep (N=640) | 85.8 | 86.0 | 85.9 | 85.8 | 85.6 | 85.9 | 85.8 |
| RSS checkpoint symbol-pass end (N=640) | 857.2 | 856.8 | 853.1 | 854.6 | 854.8 | 854.8 | 854.8 |
| RSS checkpoint file-pass tail (N=640) | 1067.5 | 1068.6 | 1062.9 | 1063.6 | 1065.3 | 1063.9 | 1064.6 |
| RSS checkpoint save tail (N=640) | 1068.1 | 1069.1 | 1062.9 | 1063.6 | 1065.8 | 1064.5 | 1065.2 |
| RSS checkpoint post-return (N=640) | 1068.1 | 1069.1 | 1062.9 | 1063.6 | 1065.8 | 1064.5 | 1065.2 |
| ensure_load wall seconds (N=1280) | 13.8 | 13.4 | 12.8 | 13.4 | 13.9 | 13.9 | 13.6 |
| boot stages 1-4 wall seconds (N=1280) | 3.1 | 3.1 | 3.2 | 3.1 | 3.1 | 3.1 | 3.1 |
| ensure_load wall seconds (N=640) | 1.6 | 1.7 | 1.6 | 1.6 | 1.6 | 1.6 | 1.6 |
| warm deserialize peak | 9.1 | 9.1 | 9.1 | 9.1 | 9.1 | 9.2 | 9.1 |
| warm deserialize delta | 3.7 | 3.7 | 3.7 | 3.7 | 3.7 | 3.7 | 3.7 |

Accounting: `REJECT invocation 3: stage5 peak RSS (release, N=640) sample
2363.9 vs slot median 2088.8 = 13.2%`; `accepted 5 of 6`
(`target/t0c-g1-medians.txt`). Deterministic attribution rows (identical in
every G1/G2/G3 invocation, demand not cache-length, I7c-bound):
semantic-closure entries 5024.0 / sampled entry size 621.5 / est aggregate
refs 3122416.0 and physical-expansion entries 1280.0 / sampled entry size
4944.0 / est aggregate refs 6328320.0 at N=1280 (T0a Block 4 values
reproduced exactly); N=640 twins 2464.0/301.5/742896.0 and
640.0/2384.0/1525760.0.

### G2 debug curve (caps landed; accepted 5 of 5, zero rejections)

| label (debug) | inv1 | inv2 | inv3 | inv4 | inv5 | median |
| --- | --- | --- | --- | --- | --- | --- |
| stage5 peak RSS (N=1280) sample a | 8468.9 | 8471.6 | 8470.2 | 8471.6 | 8474.9 | 8470.9 |
| stage5 peak RSS (N=1280) sample b | 8458.5 | 8464.1 | 8473.1 | 8467.0 | 8473.4 | (above) |
| stage5 peak RSS (N=640) sample a | 2088.6 | 2092.0 | 2091.2 | 2093.1 | 2093.9 | 2091.2 |
| stage5 peak RSS (N=640) sample b | 2089.2 | 2091.7 | 2088.4 | 2091.3 | 2090.1 | (above) |
| stage5 RSS growth ratio (N=640->1280) | 4.0 | 4.0 | 4.1 | 4.0 | 4.1 | 4.0 |
| RSS checkpoint prep (N=1280) | 174.1 | 173.5 | 174.0 | 173.9 | 173.9 | 173.9 |
| RSS checkpoint symbol-pass end (N=1280) | 3515.6 | 3517.7 | 3517.5 | 3519.1 | 3520.3 | 3517.7 |
| RSS checkpoint file-pass tail (N=1280) | 4373.5 | 4375.6 | 4374.1 | 4376.5 | 4378.9 | 4375.6 |
| RSS checkpoint save tail (N=1280) | 4372.8 | 4375.6 | 4374.2 | 4375.6 | 4378.9 | 4375.6 |
| RSS checkpoint post-return (N=1280) | 4372.8 | 4375.6 | 4374.2 | 4375.6 | 4378.9 | 4375.6 |
| ensure_load wall seconds (N=1280) | 60.7 | 61.5 | 60.6 | 61.3 | 61.1 | 61.1 |
| boot stages 1-4 wall seconds (N=1280) | 13.7 | 13.7 | 13.7 | 13.7 | 13.7 | 13.7 |
| ensure_load wall seconds (N=640) | 13.2 | 13.2 | 13.3 | 13.2 | 13.2 | 13.2 |
| boot stages 1-4 wall seconds (N=640) | 6.1 | 6.2 | 6.2 | 6.2 | 6.1 | 6.2 |
| RSS checkpoint save tail (N=640) | 1064.6 | 1068.0 | 1067.2 | 1069.1 | 1069.8 | 1068.0 |
| warm deserialize peak | 10.4 | 10.4 | 10.4 | 10.4 | 10.4 | 10.4 |
| warm deserialize delta | 4.5 | 4.5 | 4.5 | 4.5 | 4.5 | 4.5 |

Accounting: `accepted 5 of 5` (`target/t0c-g2-medians.txt`). Debug peaks
match release within 0.1% (the evidence-doc profile-independence reading
reproduces post-fix); debug wall rows are the profile-cost context.

### G3 release cap-0 ablation (`ARIADNE_LOAD_MEM_CAPS=0,0`; accepted 5 of 5)

| label (release, caps 0,0) | inv1 | inv2 | inv3 | inv4 | inv5 | median |
| --- | --- | --- | --- | --- | --- | --- |
| stage5 peak RSS (N=1280) sample a | 8479.0 | 8477.1 | 8485.0 | 8472.5 | 8476.4 | 8479.2 |
| stage5 peak RSS (N=1280) sample b | 8472.3 | 8479.5 | 8496.2 | 8494.7 | 8488.3 | (above) |
| stage5 peak RSS (N=640) sample a | 2087.3 | 2087.4 | 2087.5 | 2085.1 | 2083.1 | 2086.7 |
| stage5 peak RSS (N=640) sample b | 2086.7 | 2087.5 | 2086.7 | 2086.7 | 2086.7 | (above) |
| stage5 RSS growth ratio (N=640->1280) | 4.1 | 4.1 | 4.1 | 4.1 | 4.1 | 4.1 |
| RSS checkpoint prep (N=1280) | 171.8 | 171.8 | 171.9 | 171.5 | 171.9 | 171.8 |
| RSS checkpoint symbol-pass end (N=1280) | 3527.1 | 3518.2 | 3525.2 | 3518.8 | 3520.4 | 3520.4 |
| RSS checkpoint file-pass tail (N=1280) | 4383.0 | 4380.5 | 4388.5 | 4375.9 | 4379.8 | 4380.5 |
| RSS checkpoint save tail (N=1280) | 4383.0 | 4381.0 | 4388.1 | 4376.5 | 4380.4 | 4381.0 |
| RSS checkpoint post-return (N=1280) | 4383.0 | 4381.0 | 4388.1 | 4376.5 | 4380.4 | 4381.0 |
| ensure_load wall seconds (N=1280) | 17.1 | 16.2 | 19.2 | 14.2 | 15.0 | 16.2 |
| RSS checkpoint save tail (N=640) | 1063.3 | 1063.4 | 1063.5 | 1061.1 | 1059.0 | 1063.3 |

Accounting: `accepted 5 of 5` (`target/t0c-g3-medians.txt`); every capture
carries the loud witness line (sec. 1). Wall context: the cap-0 recompute
cost is the +2.6 s median on `ensure_load` at N=1280 (16.2 vs 13.6) -
small because boot queries each seed/file exactly once, so even the
landed-cap admissions are never re-read (T1's "retained entries past the
first query are pure bloat" economics, now measured).

## 3. Bar comparison (DRAFT markers only - the evidence review adjudicates)

- `stage5 peak RSS (release, N=1280)` median 8469.5 vs bar 5200 ->
  `BAR: MISS` (DRAFT).
- `stage5 peak RSS (debug, N=1280)` median 8470.9 vs bar 5200 ->
  `BAR: MISS` (DRAFT).
- `stage5 RSS growth ratio (release, N=640->1280)` median 4.1 vs G 2.0 ->
  `BAR: MISS` (DRAFT).
- `stage5 RSS growth ratio (debug, N=640->1280)` median 4.0 vs G 2.0 ->
  `BAR: MISS` (DRAFT).

FAIL context (expected, per the T2 verdict sec. 11): the post-fix N=1280
peak was projected ~8.6-8.7 GiB with the ratified 5200 MiB bar explicitly
not closable until the A7 save-span transient is fixed. The measured
landed-caps median is 8469.5 MiB = 8.27 GiB - slightly BETTER than the
projection (the T1+T2 residency relief ran ~0.4 GiB stronger at N=1280
than the N=640-scaled estimate), and still 63% over the bar. The constant
save-span transient reproduces at 4098.0 MiB (vs the T0a finding 4096.0 +/-
0.1: +2.0 MiB drift, within checkpoint-granularity noise) and the
half-scale transient at 1024.0 (peak 2088.8 - save tail 1065.2 = 1023.6;
T2 witness 1024.0).
Growth-ratio context: the ratio stayed ~4.0-4.1 because the RESIDENT PASS
BASE itself scales ~quadratically on this hub-heavy fixture (save tail
1065.2 -> 4371.5 = 4.10x for 2x files): the fixture's per-file payload IS
corpus-scale (the accumulation shape the bench was built to reproduce),
and the cap-0 variant has the identical ratio - memo caching is
demonstrably not the growth driver post-T1/T2.

## 4. Ablation ranking (release, N=1280 medians; arithmetic contract)

`transient_landed = peak(G1) - RSS checkpoint save tail(G1)`
= 8469.5 - 4371.5 = 4098.0
`transient_cap0 = peak(G3) - RSS checkpoint save tail(G3)`
= 8479.2 - 4381.0 = 4098.2
`memo_residual = peak(G1) - peak(G3)`
= 8469.5 - 8479.2 = -9.7
(the retained-memo term at the landed 512/128 caps is INSIDE the run-to-run
noise band: G1 peak range 8464.9-8473.7, G3 range 8472.3-8496.2; the sign
even favors cap-0 paying a slightly higher transient base, so the honest
reading is memo residual ~= 0 with |value| < ~25 MiB)
`memo_contribution_uncapped = 9040 MiB class (T0a uncapped release N=1280
peak) - peak(G3)` = 9040.0 - 8479.2 = 560.8
(READ HONESTLY: this cross-epoch difference mixes the memo uncapping
removed by T1 with the residency T2 removed - second graph/reduced copies,
the IndexResult, the pre-save memo table drop. It is the TOTAL T1+T2 relief
class, not a pure memo term; cited that way.)
Resident-after-return rows: landed 4371.5, cap-0 4381.0 (delta -9.5, noise;
the analyzer-and-memo drop before save already removes all memo residency
from the post-return base at either cap setting).

Printed in the row grammar (t0c-suffixed, distinct from the gate tuple):

```
t0c transient landed (release, N=1280)          | 4098.0
t0c transient cap-0 (release, N=1280)           | 4098.2
t0c memo residual (release, N=1280)             | -9.7
t0c memo contribution uncapped (release, N=1280)| 560.8
t0c resident after return landed (release, N=1280) | 4371.5
t0c resident after return cap-0 (release, N=1280)  | 4381.0
```

## 5. DRAFT DP-2 ranking sentence (adjudication belongs to the evidence review)

DRAFT reading for DP-2 (does the per-symbol payload / save-span transient
class, A7, rank among the top post-fix contributors to the stage-5 peak?):
post-fix release N=1280 peak median 8469.5 MiB decomposes into resident
pass base 4371.5 (51.6%) + save-span transient 4098.0 (48.4%), with the
landed-cap memo residual at ~0 (-9.7, in-band).
Ordered contributors: (1) pass-resident model/metrics base 4371.5 MiB -
the fixture-inherent quadratic payload, not addressable by caching work;
(2) the A7 save-span transient 4098.0 MiB - the LARGEST SINGLE ADDRESSABLE
CONTRIBUTOR, reproduced constant at ~4096 MiB and profile-independent;
(3) memo retention at landed caps ~0 MiB - the memo lever is exhausted for
peak purposes (this is the measured answer to "memo retention residual vs
save-span transient vs the rest": the transient ranks above memo retention
by ~4100 MiB).
Answer to the DP-2 condition as written: YES among addressable
contributors - the A7 transient dominates every remaining cache-side term,
and by the arithmetic `8469.5 - 4098.0 = 4371.5 <= 5200`, fixing A7 alone
closes the release peak bar at N=1280, while no memo-side action can
(further) close it.
The growth gate G 2.0 CANNOT be closed by either lever on this fixture
(ratio 4.0-4.1 is the quadratic payload shape itself; cap-0 has the
identical ratio) - relayed as a question for the review: whether G 2.0 is
meaningfully re-checkable on a hub-heavy synthetic corpus at all.

## 6. Explicitly NOT run / NOT done

- No `--update-baseline`, no `tools/**` touch, no `bench_baseline.json`
  touch (operator PR-2 only); no gate verdict asserted from any number
  above - only DRAFT markers.
- No debug cap-0 ablation (journal D4: release is the launch profile, RSS
  growth is profile-independent per evidence Block 5; debug cap-0 would pay
  the ~20x-slow debug passes x mass recompute for zero additional bar data).
- No N>1280 scales; no T4/DP-2 implementation work; no edits outside the
  touch-set; the bench file, I5 file, `crates/graph/**`, and the boot
  contract stayed diff-EMPTY (proof commands in the journal).
