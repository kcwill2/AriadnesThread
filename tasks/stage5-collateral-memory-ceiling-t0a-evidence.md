# stage5 collateral memory ceiling -- T0a evidence table

Canonical path: `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md`
(T0a verdict finding 2: the T0b handoff supplier pointer is being corrected
in parallel by the planner; see journal deviation D6).

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

---

## ERRATUM (append-only; appended after T0a verdict REJECTED at HEAD `8d34a3c`)

### E1 (verdict finding 1, BLOCKER) -- corrected release growth-ratio reading

The `stage5 RSS growth ratio (release, N=640->1280)` row above read 3.6 in the
first work session. That reading was wrong for the gate (I7b). The ratio test
has been made DECOMPOSABLE: `stage5_rss_growth_ratio` now re-emits both
children's `stage5 peak RSS` operand rows verbatim before the ratio row (code:
`crates/cli/tests/stage5_memory_bench.rs` `emit_and_parse_peak_row`), so every
ratio measurement is auditable against the rows it computed from. The ratio row
label itself is unchanged (T0b label contract intact). The peak label now
appears twice per scale in the full-suite run (scale test's child + ratio
test's child): two honest samples of one quantity; T0b must parse peak rows as
a multiset per label.

Corrected release readings (fixed code, >= 3 fully-read isolated invocations,
each capture read immediately after its run):

| invocation (capture: `target/distill-capture.txt`) | half operand | full operand | ratio |
| --- | --- | --- | --- |
| standalone `--exact stage5_rss_growth_ratio` | 2257.5 | 9033.8 | 4.0 |
| suite form, release invocation 1 | 2256.7 | 9042.0 | 4.0 |
| suite form, release invocation 2 | 2255.9 | 9040.2 | 4.0 |
| suite form, release invocation 3 | 2259.3 | 9029.5 | 4.0 |

CORRECTED release growth ratio = **4.0** (median of four). Debug control, fixed
code, suite form: half 2258.6 / full 9048.7 / ratio 4.0 -- debug unchanged and
still consistent with its peaks, so no debug erratum. The proposed growth gate
(`ratio <= 2.0`) and the PROPOSED bars are UNAFFECTED: the corrected value is
4.0, a clean quadratic that still fails a pre-fix gate with a wide margin.

Erratum against the 3.6 rows above: they implied release grew only 3.6x per
2x scale-up while release grew ~4.0x per the peak rows. The 4.0x reading is
correct. The divergent side was the HALF-scale (numerator) operand: the ratio
test's isolated N=640 child read ~2511 MiB (~+12%) in the session that produced
3.6, while its full-scale (denominator) operand stayed at ~9040. All four
corrected half operands land in the in-band ~2256-2259, and the ratio is 4.0.

Mechanism (reviewer finding 1b -- OS working-set behaviour, not a bench
artefact): the anomaly is ambient machine-memory pressure during the very
short (~1.6 s) release N=640 boot, not a code path. Evidence:

- Rebuilt old code (pre-fix, non-emitting parse), same machine minutes later:
  standalone ratio read 4.0 (half 2257.5 / full 9033.8) -- in band. The 3.6
  did not reproduce once memory pressure cleared.
- In that same old-code suite run, the anomaly attached to the HALF-scale
  CURVE test child (peak 2531.4, out of band) while that run's ratio read 4.0
  -- i.e. the inflation follows whichever N=640 child boots under the
  pressured window, NOT the test that spawned it. The reviewer's
  "depends-on-spawning-test" framing is thus refined: it depends on ambient
  pressure at boot time, and the short half-scale boot is the exposed one.
- Full-scale operands never diverged (~9030-9048 across every reading in band):
  the longer full boot lets its working set settle below the transient before
  the final peak read, so trimming does not shave the reported high-water.

Consequence for the printed gate row: with operands now emitted, a pressure-
inflated half operand is visible in the same invocation, and the T0b median-
of->=3 protocol (Block 6) is robust to the rare inflated sample. Nothing here
changes any measured peak; it corrects one derived ratio and its interpretation.

### E2 (verdict finding 4) -- block-2 exemplar and the 4096 transient, recomputed

Block 2 above used debug invocation 4 / release invocation 4 as exemplars; the
release invocation 4 is the same run excluded as the +13% peak outlier, and the
fully-read set is debug invocations 1/2/4 (not "1-3"). In-band, fully-read
exemplars are now available (fixed-code release suite invocation 1: full peak
9035.7, save tail 4939.7; fixed-code debug control: peak 9047.2, save tail
4951.1). The 4096 transient is recomputed using ONLY pairs whose peak AND
save-tail were both transcribed in the same capture, dropping the partial-
invocation peak 9043.3 and any untranscribed save-tail:

| run (both operands transcribed same capture) | peak | save tail | transient |
| --- | --- | --- | --- |
| fixed-code debug control | 9047.2 | 4951.1 | 4096.1 |
| fixed-code release suite 1 | 9035.7 | 4939.7 | 4096.0 |
| earlier fully-read debug | 9034.4 | 4938.4 | 4096.0 |
| earlier fully-read debug | 9033.7 | 4937.6 | 4096.1 |

The constant ~4096.0 +/- 0.1 MiB save-span transient finding STANDS on the
corrected, fully-transcribed pair set. The 4096-rider probe remains PROPOSED.

### E3 (verdict finding 3) -- label count

The line above says "25 table-row labels"; the correct count is **26** (the
journal's "Label set emitted" enumerates 26 and the code emits exactly those).
The label SET was always verbatim-correct; only the count was misstated. For
T0b's tuple-exactness grep: the growth-ratio label is printed with a
parameterized scale `N={half}->{n0}`, which reads `N=640->1280` only at the
default scale.

### E4 (verdict finding 2) -- canonical path and the T0b supplier pointer

This file's canonical path is `tasks/stage5-collateral-memory-ceiling-t0a-
evidence.md` (recorded in the header above). It was NOT renamed. The T0b
supplier pointer at `tasks/stage5-collateral-memory-ceiling-t0b-handoff.md:16`
(citing the non-existent `...t0-evidence.md`) is being corrected by the planner
in parallel; deviation D6 in the journal declares the name mismatch.

### Net status of proposed constants after the rework

N0 (1280), bars (5200 MiB/profile), growth gate G (2.0), tolerance (1.15x) and
the 4096-rider remain PROPOSED only. Zero RATIFIED lines; DP-4 holds.

---

## RATIFICATION + ROUND-2 ERRATUM (2026-10-06; appended after the DP-4
## evidence review; append-only -- nothing above is rewritten)

Authority for this section: user decision at the DP-4 evidence review,
**RATIFY as tabled** (2026-10-06, orchestrator-recorded). This section
SUPERSEDES the "Zero RATIFIED lines" sentence above (append-only discipline:
the earlier sentence records the pre-review state, which was correct when
written).

### RATIFIED

- `RATIFIED:` (2026-10-06, authority: user decision DP-4 evidence review,
  orchestrator-recorded) **N0 = 1280 fixture files; scales {640, 1280}.**
- `RATIFIED:` (same date/authority) **peak-RSS bars: `stage5 peak RSS`
  <= 5200 MiB per profile at N=1280.**
- `RATIFIED:` (same date/authority) **growth gate G: `stage5 RSS growth
  ratio` <= 2.0.**
- `RATIFIED:` (same date/authority) **gate protocol: median-of->=5 isolated
  invocations; ANY invocation whose operands diverge >8% from the per-slot
  median is REJECTED -- binding on BOTH operands at BOTH scales** (the
  round-2 reviewer's condition, carried in as binding per the verdict's
  paper-relay).
- `RATIFIED:` (same date/authority) **the ~4096 MiB save-span transient probe
  as a T0b/T1 rider probe.**

### Remain PROPOSED (explicitly not ratified here)

- The **1.15x cross-machine tolerance** stays PROPOSED: it is owned by T0b
  gate design, pending PR-2.
- Everything else in this document is untouched by this section and remains
  PROPOSED or MEASURED as previously marked.

### ERRATUM to E1 -- the full-scale side DOES diverge too

E1's third mechanism bullet says "Full-scale operands never diverged
(~9030-9048 across every reading in band)". That sentence is **REFUTED** by
the round-2 reviewer's fresh witness: release operands `2260.2 / 7720.5` ->
ratio `3.4` (full operand **-14%**, outlier frequency ~1/7; capture
`target/distill-capture.txt`; recorded in
`tasks/stage5-collateral-memory-ceiling-t0a-verdict.md`). The same claim's
weaker form in E1's "3.6 rows" paragraph (that window's full operand staying
at ~9040) was true for those readings only.

Corrected law: **working-set trimming can inflate OR deflate either operand
at either scale under ambient memory pressure.** The standing defense is not
a claim about which side is safe -- it is the decomposable self-auditing row
plus the now-RATIFIED >8% operand-rejection protocol at both scales.

Amended ratio-gate rationale: the corrected ratio of 4.0 stands on **six
clean decomposed readings** (the four E1-table release operand pairs, the
debug-control pair 2258.6/9048.7, and the debug-control curve pair
9047.2/2263.3): every quotient is 4.00, maximum deviation +0.19%. The value
is unaffected by trimming in either direction: a trimmed full operand
(7720.5) still fails the RATIFIED 5200 MiB bar, and the worst observed
trimming pair (full -14%, half +12%) can pull an apparent ratio from a
genuinely quadratic run down to ~3.07 -- still a wide fail against the
RATIFIED gate G = 2.0.

### ERRATUM to E2 -- table row 4 mislabel and the confirmation count

E2's table row labelled "earlier fully-read debug" for 9033.7 / 4937.6 is a
**mislabel: that pair is release invocation 3** (its N=1280 rows were
transcribed from the partial capture; the row's arithmetic 4937.6 -> 4096.1
is unchanged). The journal's "(5th confirmation ...)" count **overstates the
evidence: the correct statement is the four-pair table**
(4096.0 / 4096.1 / 4096.0 / 4096.1, exact arithmetic, law unchanged). The
row-3 label "earlier fully-read debug" for 9034.4 / 4938.4 is correct
(debug invocation 4).

### SUPERSEDED: growth gate G demoted to a tripwire (2026-10-06 user decision, landed 2026-10-07 by M28 T4b)

The `RATIFIED:` line "**growth gate G: `stage5 RSS growth ratio` <= 2.0**"
(sec. RATIFICATION, above) is **SUPERSEDED**: on 2026-10-06, at the evidence
review, the user decided (decision 3) that G is **DEMOTED to a tripwire**.
Acceptance for the memory gate is the **absolute 5200 MiB peak bars (both
profiles) plus the Corpora field boot**; the growth ratio keeps being
computed and printed -- above 2.0 it prints a `TRIPWIRE` line and **never
fails a gate**. Grounding (`tasks/stage5-collateral-memory-ceiling-t0c-verdict.md`
sec. 7): the fixture is Theta(N^2) in aggregate retained closure by
construction (memo est aggregate refs 742896 -> 3122416 = 4.20 per 2x scale,
save-tail growth 4.10x, ratio rows 4.0-4.1 byte-stable) -- the 4.0-4.1
reading **is the payload shape**, identical under cap-0 ablation; RSS
scaling in response to workload is not a regression, and G <= 2.0 was
unclosable against this fixture by design. Nothing measured or ratified
elsewhere is touched: the 5200 MiB peak bars stand, the >8% rejection and
median-of->=5 protocol stand, the 4.0-4.1 ratio stays on record as MEASURED,
and the constant 2.0 survives as the tripwire threshold
(`STAGE5_MEMORY_GROWTH_TRIPWIRE_RATIO`). The original RATIFIED line above is
kept verbatim (append-only discipline: it records the pre-decision state,
which was correct when written). Landed by
`tasks/stage5-collateral-memory-ceiling-t4b-handoff.md` (implementation
`tools/bench_compare.py`, RED-first in `tools/bench_compare_tests.py`;
journal `tasks/stage5-collateral-memory-ceiling-t4b-journal.md`).



