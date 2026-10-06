# stage5 collateral memory ceiling -- T0a evidence table

Milestone `stage5-collateral-memory-ceiling` (M28), task T0a (`t0a-rss-bench`).
Producer: implementer, 2026-10-06 UTC. Bench: `crates/cli/tests/stage5_memory_bench.rs`.
Status of every constant here: MEASURED or PROPOSED. Nothing in this document is
RATIFIED; ratification happens at evidence review (kickoff DP-4). DP-5: the fixture
is synthetic; no real-corpus content was measured or captured.

## Measurement protocol (read this before using any number)

- Peak RSS is a process-lifetime monotonic quantity, so every curve/warm row is
  produced by an isolated child process (one boot per scale, `--exact` filter).
  Medians therefore live at this evidence layer across gate invocations, not as
  median-of-N inside one boot (journal deviation D2).
- Gate command (debug): `python tools/distill.py --timeout 3600 -- cargo test
  -p ariadne-cli --test stage5_memory_bench -- --ignored --nocapture
  --test-threads=1`. Release adds `--release`.
- Debug: 4 invocations, all exit 0, ~361-389 s each; invocations 1, 2, 4 fully
  read. Release: 4 invocations, all exit 0, ~86-114 s each; invocations 1, 2, 4
  fully read; invocation 3's capture was overwritten before full read and only
  its N=1280 rows (peak 9033.7, checkpoints 172.1/4093.9/4936.7/4937.6/4551.3)
  are used. Journal deviations D2/D3 record the protocol.
- Machine: the operator's win32 workstation, normal background dev load
  (DP-3 reserves VERIFIED-IDLE for baseline writes; there are none here).
- Units: RSS/size rows are MiB, wall rows seconds, memo rows counts.

## Block 1 -- peak RSS curve and growth ratio (the oracle rows)

| row (defaults N0=1280, N0/2=640) | debug | release |
| --- | --- | --- |
| stage5 peak RSS (debug/release, N=1280) | 9038.0 | 9045.4 |
| stage5 peak RSS (debug/release, N=640) | 2261.0 | 2257.2 |
| stage5 RSS growth ratio (N=640->1280) | 4.0 | 3.6 |

Per-invocation peak values, fully-read invocations: debug N=1280
9039.8 / 9038.0 / 9034.4 (partial-invocation row 9043.3 also recorded); debug
N=640 2261.0 / 2258.3 / 2261.7; ratios 4.0 / 3.9 / 4.0. Release N=1280
9045.4 / 9040.6 / 10218.1 (partial row 9033.7); N=640 2254.1 / 2257.2 / 2258.2;
ratios 3.6 / 3.6 / 3.6.

Outlier note: release invocation 4 printed N=1280 peak 10218.1, +13% over the
9033.7-9045.4 band from invocations 1-3, while its N=640 row stayed in band;
median-of-3+ is robust to it, max-of-N is not. This motivates the PROPOSED
gate protocol below.

Verdict-class reading (review ratifies): the curve reproduces the E1
accumulation class -- near-quadratic retained growth (ratio 3.6-4.0, i.e. ~2N
per 2N scale-up) at both profiles, with debug and release peaking within 0.1%
of each other at the same scale. RSS here is heap-dominated and
profile-independent; wall time is not (Block 5).

## Block 2 -- checkpoint series and phase deltas (which stage dominates)

Fully-read single invocation (debug invocation 4; release invocation 4; the
debug medians across invocations 1-3 are within 0.5% of these):

| row (N=1280) | debug | release |
| --- | --- | --- |
| RSS checkpoint prep | 173.7 | 172.2 |
| RSS checkpoint symbol-pass end | 4098.6 | 4103.8 |
| RSS checkpoint file-pass tail | 4937.6 | 4944.2 |
| RSS checkpoint save tail | 4938.4 | 4945.1 |
| RSS checkpoint post-return | 4554.8 | 4546.7 |
| symbol-pass RSS delta | 3924.9 | 3931.7 |
| file-pass RSS delta | 839.0 | 840.4 |
| save-tail RSS delta | 0.9 | 0.9 |
| post-return RSS delta | -383.6 | -398.4 |

Reading: the symbol pass (A1/A2 caches) dominates retained growth, ~3.92 GiB
of the ~4.9 GiB resident working set; the file pass adds a further ~0.84 GiB;
the save tail moves almost nothing at checkpoint granularity; post-return
reclaim is small (~-0.39 GiB), so resident-after-return is ~4.55 GiB.

The 4096 MiB transient (finding for T1/T0b): peak minus save-tail checkpoint
is 4096.0 +/- 0.1 MiB in EVERY fully-read debug invocation and in release
invocation 3 (9034.4-4938.4; 9039.8-4943.8; 9043.3-4947.2; 9033.7-4937.6),
i.e. an exactly ~4.0 GiB transient between adjacent checkpoint samples -- most
consistent with a single ~4 GiB-scale allocation/realloc in the save/serialize
span (A7). Event-checkpoint granularity cannot localize it further; a finer
probe around artifact serialization is PROPOSED as a T0b/T1 rider.
(release invocation 4 is excluded from this arithmetic: it is the +13% outlier.)

Half-scale (N=640, debug invocation 4): prep 87.7, symbol-pass end 1031.5,
file-pass tail 1236.8, save tail 1237.7, post-return 1145.3; deltas 943.8 /
205.2 / 0.9 / -92.4. Release invocation 4: 85.7 / 1026.3 / 1233.2 / 1234.1 /
1144.8; deltas 940.6 / 206.9 / 1.0 / -89.3.

## Block 3 -- capacity-0 ablation: DEFERRED

Not produced this session. Pre-T1 there is no capacity knob to zero; a scratch
patch would touch `crates/graph/**` (forbidden by the touch-set) and would not
reproduce post-T1 code. Per the handoff's sanctioned read-path, block 3 is
DEFERRED to the post-T1 T0b rerun. The quadratic signature is instead
established by Block 1 (ratio 3.6-4.0) and Block 4 (refs scaling).

## Block 4 -- memo attribution (deterministic; identical every invocation)

| row | N=640 | N=1280 | 640->1280 factor |
| --- | --- | --- | --- |
| memo semantic-closure entries | 2464 | 5024 | 2.04 |
| memo semantic-closure sampled entry size | 301.5 | 621.5 | 2.06 |
| memo semantic-closure est aggregate refs | 742896 | 3122416 | 4.20 |
| memo physical-expansion entries | 640 | 1280 | 2.00 |
| memo physical-expansion sampled entry size | 2384.0 | 4944.0 | 2.07 |
| memo physical-expansion est aggregate refs | 1525760 | 6328320 | 4.15 |

Reading: entry counts double while per-entry size also doubles, so aggregate
retained refs grow ~4.2x -- quadratic, matching the RSS ratio 3.6-4.0. The
physical-expansion memo carries the larger aggregate (~6.33M vs ~3.12M
sampled-scale refs at N=1280): it is the first eviction candidate for T1, with
the semantic-closure memo second. Sampled entry size = mean SymbolId count per
cached entry over a stride-32 sample through the public cache API (journal
deviation D1: no test-instrument dev-dependency was added; no Cargo.toml
change exists).

## Block 5 -- wall-context rows (profile choice for T0b)

| row (medians, fully-read invocations) | debug | release |
| --- | --- | --- |
| boot stages 1-4 wall seconds (N=1280) | 18.7 | 3.5 |
| ensure_load wall seconds (N=1280) | 112.8 | 17.2 |
| boot stages 1-4 wall seconds (N=640) | 8.3 | 1.6 |
| ensure_load wall seconds (N=640) | 25.5 | 1.7 |
| full ignored-suite invocation wall | ~361-389 s | ~86-114 s |

Reading: stage-5 wall time is ~6.6x debug-over-release at N=1280, while peak
RSS is profile-independent. Per AGENTS.md the human launch profile is release;
T0b gate rows should therefore be recorded on release with debug kept as
diagnostic context.

## Block 6 -- warm deserialize rows (A5/A6 read side, 40-file warm fixture)

| row | debug | release |
| --- | --- | --- |
| warm deserialize peak | 10.4 | 9.2 |
| warm deserialize delta | 4.5 | 3.7 |
| warm deserialize wall seconds | 0.0 | 0.0 |
| warm report.bin size | 0.9 | 0.9 |
| warm report.json size | 2.0 | 2.0 |

Reading: deserialize delta is ~4-5x the report.bin size at this scale.
Synthetic fixture only; real-corpus blob sizes were NOT measured (DP-5).
N0-scale warm rows are PROPOSED for the T0b rerun if review wants them.

## PROPOSED constants and protocol for T0b (all to be ratified at review; zero RATIFIED)

- PROPOSED N0 = 1280 fixture files, scales {640, 1280}; both curves complete
  in one distiller invocation (max ~389 s, under the 3600 s timeout).
- PROPOSED growth gate: `stage5 RSS growth ratio` <= 2.0. Measured pre-fix:
  3.6-4.0 (both profiles) -> fails now; a bounded-cache fix should land near
  linear and pass.
- PROPOSED bars: `stage5 peak RSS` (each profile) <= 5200 MiB at N=1280.
  Rationale: file-pass-tail residency is ~4938-4944 MiB, so the bar passes if
  EITHER dominant contributor is fixed (the ~4096 MiB save-span transient, or
  the ~3.9 GiB symbol-pass retention) while today's ~9040 MiB peak fails with
  >70% margin.
- PROPOSED gate protocol: compare per-machine medians over >= 3 isolated
  invocations (never max); observed run spread is 0.06% debug / 0.13% release
  in-band, with a 13% single-invocation outlier -> PROPOSED cross-machine
  tolerance 1.15x on peak rows, revisit after a second machine's baseline.
- PROPOSED rider probe: finer-grained RSS probe around artifact
  save/serialize to localize the constant 4096 MiB transient before T2 touches
  that span.

## Verbatim label set (T0b row contract)

As enumerated in `tasks/stage5-collateral-memory-ceiling-t0a-journal.md`
("Label set emitted", 25 table-row labels); every table row carries exactly
one pipe and one finite float value, every context line carries no pipe, and
the profile is embedded in every label.
