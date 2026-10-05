# T1 measurement evidence - M27 tu-index-hang-throughput: RELEASE boot stages + sufficiency gate

Hop: `tasks/tu-index-hang-throughput-t1a-handoff.md` (T1a). Companion journal:
`tasks/tu-index-hang-throughput-t1a-journal.md`. Bench:
`crates/pipeline/tests/boot_stages_bench.rs` (ignored; `label | ms` grammar
is T1b's `DOCUMENTED_PHASES_BOOT_STAGES` contract). This doc is the
sufficiency-gate evidence base for whether T3 (workers wiring) ever runs.

Date: 2026-10-05. Machine (per bench run header, identical all runs):
`windows-AMD64-AMD64 Family 26 Model 68 Stepping 0, AuthenticAMD`
(consistent with the E3-recorded 8-core/16-thread Ryzen 9800X3D, 31 GB).
Fixture: synthetic 2000 headers / 400 TUs (shape sec. 3.3 start values,
ratifiable at the evidence review). W=N row width n=8
(`min(8, available_parallelism)` - today's `Pipeline::new(path, Some(n))`
library capability, NOT T3 dispatch).

Idle-ness note: runs were back-to-back on the live dev workstation while
agent sessions were active - idle-ish, NOT verified-idle (the PR-2 rule
binds only the baseline update, T1b's hop). Run-to-run spread is recorded
below so the reviewer can weigh medians accordingly.

## (a) RELEASE median table (3 runs, medians; all values ms)

| stage label (T1b contract)                              | run 1    | run 2    | run 3    | median   |
|----------------------------------------------------------|----------|----------|----------|----------|
| discover+hash total (release)                            |    224.6 |    222.7 |    237.5 |    224.6 |
| build discovery (contexts) total (release)               |      1.1 |      1.0 |      1.0 |      1.0 |
| stage fingerprints total (release)                       |      0.8 |      0.7 |      0.7 |      0.7 |
| include-graph cold total (release)                       |    464.3 |    491.2 |    460.9 |    464.3 |
| include-graph warm load total (release)                  |     51.3 |     37.2 |     44.3 |     44.3 |
| tu-index cold total (W=1, release)                       |  60553.4 |  56563.2 |  68494.9 |  60553.4 |
| tu-index cold total (W=N, release)                       |  85681.1 |  83850.5 | 101597.2 |  85681.1 |
| tu-index warm reuse total (release)                      |   3176.2 |   2730.2 |   3308.1 |   3176.2 |

Captures: `target/distill-capture.txt` per handoff gate 1 (overwritten by
each successive distilling invocation per the capture-file convention; all
three row sets transcribed above while fresh). Test-harness line per run:
`1 passed; 0 failed` (harness totals, not the canonical census).

Method pointers (stated verbatim in the bench module doc): boot path
replicated over the public surface only; cold = fresh artifacts tree; warm
rows subtract a same-code-path `Pipeline::new` ANCHOR (construction cost,
`rehash_inputs`-equivalent) measured back-to-back after a throwaway
construct, and assert `StageReuse::Loaded`.

Key derived facts:

- Stage 2 (TU index) cold is 99% of the measured cold boot work: 60.6 s of
  a ~61.5 s measured stage total; stage-1 components (discover+hash
  224.6 ms, build discovery 1.0 ms, fingerprints 0.7 ms, include-graph cold
  464.3 ms) are ~1.2% - input to the `stage_spans.rs` D-calibration TODO,
  context for review, not this hop's decision.
- Warm TU-index reuse costs 3.18 s (index-artifact reload over the fat
  400-TU closure fixture) vs 60.6 s cold - warm boots already ride it.

## (b) DEBUG context row (E3 shape, non-gate)

Bench command: handoff gate 2 (debug profile). Captured row:

    tu-index non-ASCII repro TU (debug, panic-pre-T2 / done-post-T2) |     12.6

Harness: `test result: ok. 1 passed; 0 failed` - the run COMPLETED under
the debug profile (`debug_assertions=true` in the run header): the
`done-post-T2` label branch, because T2 landed APPROVED (tokenizer both
flush sites + class audit; `CURRENT_ANALYSIS_VERSION` 2.1.0 per DP-6).

PRE-T2 this same run PANICKED (`attempt to subtract with overflow` at
`crates/lang-cpp/src/tokenizer.rs:331:21`) - the E3 self-report. That
panic line is deliberately NOT re-captured here (the fix is landed; the
authoritative captured text is cited at
`tasks/tu-index-hang-throughput-t0-evidence.md` sec. 1). This row is
context only: never in any gate tuple (handoff + shape sec. 3.3; T1b's
`DOCUMENTED_PHASES_BOOT_STAGES` stays the eight release labels).

Corpus: one-TU temp fixture built in-test from numeric byte arrays
(U+00E9 word-run pair + line-leading U+20AC punct run + U+00AB mid-run,
mirroring the approved T0b fixture lines in
`tests/fixtures/m27-nonascii-underflow` - read-only reference, unedited);
never committed, authored bench source is pure ASCII.

## (c) LAUNCH-PROFILE RECORD (obligation (b): how the app is NORMALLY built)

Facts (inspected 2026-10-05 at `42d4a6e`):

- `AGENTS.md` Build section: "Use `cargo build` to build the project";
  "Use `cargo build --release` for release builds" - the documented
  default is DEBUG; `--release` is phrased as the special case.
- `tools/ui_smoke.py`: build step is `cargo build --bin tool` (no profile
  flag) and the default `--target-dir target/debug` - the repo's own
  smoke ritual builds AND launches a DEBUG binary.
- `EXTENSIONS.md` and `extensions/ariadne-extension` docs tell users to
  start the app with `cargo run --bin cli --features server` - `cargo run`
  defaults to DEBUG.
- No Makefile / justfile / `.cargo/config` build aliases exist; the only
  tracked `.ps1` (`test-opencode-acp.ps1`) neither builds nor launches the
  app.

FINDING (de-facto profile): DEBUG is the de-facto default an actual user
gets through every documented route. E3 independently proved the user's
hang session was a DEBUG build (overflow-checks ON panic), and E1's
observed ~3.1 s/TU was therefore measured under that profile. This is
cheap throughput before any machinery: shipping a release-profile launch
route (doc line or smoke/smoke-adjacent script flag) is a release-
independent lever worth more than any plausible T3 win on today's data
(see (d): W=N is currently SLOWER, factor 0.71). Route to the orchestrator.

## (d) SUFFICIENCY VERDICT (measured numbers only; formal gate = review)

    release ms/TU (W=1) = 60553.4 / 400 = 151.4 ms/TU
    projected stage-2 cold wall at corpus scale = 151.4 * 5744
      = 869,547 ms ~= 870 s ~= 14.5 min
    W=N speedup factor = W=1 median / W=N median
      = 60553.4 / 85681.1 = 0.71  (W=N at n=8 is 1.41x SLOWER)
    RECOMMENDATION: ACCEPT - release+single-thread projects ~14.5 min cold
      stage 2 at 5744-TU scale (vs multi-hour observed DEBUG reality), with
      warm reuse already at ~3 s; T3 machinery must not be dispatched on
      evidence that its own workers path is 0.71x.
    SUFFICIENCY-GATE: ACCEPT - release+single-thread meets the ratified
      wall-time bar at corpus scale (T3 NOT dispatched)

Finalization note: the ratified wall-time BAR is the orchestrator+user
decision at the evidence review (shape sec. 11 Q2/Q3); the ACCEPT line
above is the implementer's recommendation in the verbatim format and the
review note under this section finalizes it (swap to the FAIL variant
verbatim if the bar demands sub-14.5-min cold boots).

If the bar does demand it, the W=N row says the win is not in wiring: the
production (`Some(workers)`) path is measurably slower at n=8 on this
fixture, so the sub-bar route is a ProductionAnalyzer throughput fix first,
not the T3 dispatch as shaped.

Q3/Q7 proposals (ratify at review):

- K (improvement target): K = 1.0 - no improvement factor is defensible
  from the W=N row (0.71x); a dispatched T3-phase gate should require the
  affected `boot_stages` phase <= T1 W=1 median / 1.0 (parity at best)
  until the workers path independently proves faster.
- Auto-workers cap N: keep the `min(8, available_parallelism)` cap only as
  an explicit opt-in escape hatch (`--workers`); the DEFAULT boot stays
  single-threaded (`None`) on this evidence. Q7's exact cap matters only
   if a fixed workers path ever ships.

### (d-final) FORMAL RATIFICATION - orchestrator finalization append, 2026-10-05

Source: `tasks/tu-index-hang-throughput-t1a-verdict.md` (VERDICT: APPROVED;
FORMAL SUFFICIENCY-GATE section; the ACCEPT line quoted verbatim from its
lines 52-55; ratification owned by the reviewer hop per handoff lines
193-195; landing commit `bd4fb1a`, start `42d4a6e`).

Ratified bar: release-profile single-thread cold stage-2 wall at the
5744-TU corpus scale must be minutes-to-tens-of-minutes - an order of
magnitude below the multi-hour DEBUG boot that motivated the milestone -
with warm-boot reuse in seconds. Bar MET with ~an order of magnitude
margin (cross-session projection 13.6-16.4 min; warm ~3.2 s; reviewer
fresh independent run 15.2 min).

    SUFFICIENCY-GATE: ACCEPT - release+single-thread meets the ratified
      wall-time bar at corpus scale (T3 NOT dispatched)

Rider ratifications: K = 1.0 ACCEPTED (future throughput gates take the T1
W=1 medians as baseline, parity at best); the min(8, available_parallelism)
cap stays an explicit opt-in `--workers` escape hatch ONLY - default boot
stays single-threaded (`None`); Q7 exact cap moot (T3 not fired); Q5/DP-4
and the T4 stub EVIDENCE-CLOSED (stage-1 = 1.2% of measured boot work;
repo-hash parallelism cannot move the wall); the DP-4 lang-cpp parse-scope
limb stays the reserved sub-bar analyzer-throughput route, moot at the
ratified bar.

Arithmetic errata (verdict finding 1; display-only, no re-run, results
unchanged): the printed ":110" line "151.4 * 5744 = 869,547" above is
pre-rounded display - 869,547 is the full-precision product
151.3835 x 5744 = 869,546.8 (the rounded factor 151.4 alone evaluates to
869,641.6); and the ":48-49" "~61.5 s measured stage total" is 61.29 s.

## Deviations recorded

1. Warm-row subtraction: a plain `Pipeline::rehash_inputs` subtraction ran
   NEGATIVE (-92.9 ms, first Green capture) because the anchor hashed
   cold-cache while the boot measure hashed warm on the fast index-reuse
   row. The shipped method (anchored same-code-path `Pipeline::new`,
   throwaway construct first) is stated in the bench module doc and keeps
   the handoff's "construction time subtracted" requirement honest.
2. Release bench invocations: 5 (1 Red partial-label, 1 Green with the
   pre-fix warm method, 3 medians on final code) instead of 3, because the
   negative-anchor bug invalidated the first Green run. Handoff ceiling of
   <= 10 gate invocations held (total implementer distill invocations: 10).

## (e) Post-flag release cost (T5, 2026-10-05)

Method (one line): ONE release run of the T1a bench under the newly landed
`[profile.release] overflow-checks = true` (workspace root, same commit),
same invocation as T1a handoff gate 1
(`python tools/distill.py -- cargo test --release -p ariadne-pipeline
--test boot_stages_bench -- boot_stages_release_timing --ignored
--nocapture --test-threads=1`), machine header identical to sec. (a),
single-run values transcribed from the fresh capture
`target/distill-capture.txt` per the sec. (a) overwrite convention.

| label                                     | post-flag ms | T1a median | delta % |
|-------------------------------------------|--------------|------------|---------|
| discover+hash total (release)             |        232.3 |      224.6 |   +3.4% |
| build discovery (contexts) total (release)|          1.0 |        1.0 |    0.0% |
| stage fingerprints total (release)        |          0.7 |        0.7 |    0.0% |
| include-graph cold total (release)        |       1240.3 |      464.3 | +167.1% |
| include-graph warm load total (release)   |         46.1 |       44.3 |   +4.1% |
| tu-index cold total (W=1, release)        |      58389.0 |    60553.4 |   -3.6% |
| tu-index cold total (W=N, release)        |      86201.9 |    85681.1 |   +0.6% |
| tu-index warm reuse total (release)       |       3082.6 |     3176.2 |   -3.0% |

STOP-bar outcome: NOT A TRIP under the handoff adjudication rule. The
include-graph cold single value exceeded the +10% bar on the first run
(1240.3 ms, which was also the first run of the freshly rebuilt release
binaries under the new profile); the ONE confirming re-run printed
478.3 ms - INSIDE the sec. (a) recorded run spread for that stage
(460.9-491.2). Both numbers recorded; proceeding. Re-run context values
(non-gating): discover+hash 217.2, build discovery 1.0, fingerprints 0.7,
warm load 42.0, W=1 61274.2, W=N 78005.8, warm reuse 2718.9. The other
seven first-run labels sit within -3.6%..+4.1% of their medians; the
flag cost is uniform and within bar. Final flag acceptance is the
reviewer's call per the T5 handoff.
