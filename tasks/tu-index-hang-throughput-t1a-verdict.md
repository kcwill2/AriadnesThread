# T1a verdict - M27 tu-index-hang-throughput: RELEASE boot_stages bench + sufficiency gate

Reviewer hop (skill `review-gate`), 2026-10-05. Handoff
`tasks/tu-index-hang-throughput-t1a-handoff.md`; commit `bd4fb1a` (single
commit, start `42d4a6e`); journal
`tasks/tu-index-hang-throughput-t1a-journal.md`; evidence
`tasks/tu-index-hang-throughput-t1-measurement.md`. Execution-results
addendum 4: `tasks/tu-index-hang-throughput-execution-results.md`.

## VERDICT: APPROVED

No blockers. Three minor findings (none require an implementer change); both
declared deviations ACCEPTED. The SUFFICIENCY-GATE is formally RATIFIED as
ACCEPT - see the dedicated section; it is this hop's binding output.

## FORMAL SUFFICIENCY-GATE (ratified at this hop)

Bar ratification. Shape sec. 3.3(c) defines the gate question: does
release+single-thread indexing meet the ratified wall-time bar at corpus
scale; shape sec. 11 Q2 defers "does T3 run at all" to this gate. No artifact
(kickoff, shape, ledger, handoffs) fixes a numeric bar - the handoff (lines
107-108, 193-195) places ratification at this evidence review, and this
dispatch delegates the formalization to the reviewer hop. Ratified bar, from
the milestone's own motivating evidence (kickoff S2 + field evidence E1):
release-profile single-thread cold stage-2 wall at the 5744-TU corpus scale
must be minutes-to-tens-of-minutes - an order of magnitude below the
multi-hour DEBUG boot that motivated the milestone - with warm-boot reuse in
seconds.

Measurement basis (medians over 3 runs, all eight medians independently
verified as the exact middle of their three rows): 151.4 ms/TU at W=1 ->
869,547 ms ~= 14.5 min projected cold stage-2 wall (full-precision product
151.3835 x 5744; the doc's printed "151.4 * 5744 = 869,547" is the same value
pre-rounded - finding 1); warm reuse 3176.2 ms ~= 3.2 s. Reviewer's fresh
independent run: W=1 63428.2 ms = 158.6 ms/TU -> 15.2 min; warm 3178.5 ms.
Cross-session run spread: 13.6-16.4 min projection. Bar MET with ~an order of
magnitude margin against E1's multi-hour reality.

T3-immateriality clause: no stricter ratified number would flip the dispatch
answer. T3 is the workers-wiring seam, and the shipped `Some(workers)` path
measures 1.41x SLOWER at n=8 - 60553.4 vs 85681.1 median, reproduced 4/4
runs across two independent sessions (implementer 3/3: 85681.1/83850.5/
101597.2 all > their paired W=1; reviewer 1/1: 86912.5 > 63428.2). T3
dispatch is predicated on a wiring win the data refutes; the measurement doc
correctly names ProductionAnalyzer throughput work, not shaped T3, as the
only sub-bar route. ACCEPT is therefore the dispatch-correct answer at any
plausible bar.

Formal line (verbatim handoff format; orchestrator owes the finalization
append to the measurement doc sec. (d), handoff lines 193-195):

```
SUFFICIENCY-GATE: ACCEPT - release+single-thread meets the ratified
  wall-time bar at corpus scale (T3 NOT dispatched)
```

Rider ratifications: K = 1.0 ACCEPTED (no improvement factor defensible from
a 0.71x row; any future throughput gate takes the T1 W=1 medians as baseline,
parity at best). Auto-workers cap: `min(8, available_parallelism)` stays an
explicit opt-in `--workers` escape hatch ONLY; default boot stays
single-threaded (`None`) on this evidence. Q7 exact cap: moot (T3 not
fired). Q5/DP-4 and the T4 stub: EVIDENCE-CLOSED - stage-1 components total
734.9 ms of the 61.3 s measured stage work (1.2%); generous linear corpus
extrapolation keeps them seconds-to-tens-of-seconds against a 14.5-min
stage-2 wall; repo-hash parallelism cannot move the wall. The DP-4
lang-cpp parse-scope limb is the reserved sub-bar analyzer-throughput route,
moot at the ratified bar. No cause found to rule otherwise.

## Per-commit review (oldest-first; the range has one commit)

`bd4fb1a` - single commit, all-new files, +755/-0. Content reviewed at HEAD
(worktree clean on all four paths per checkpoint_guard dirty list). Adds:
`crates/pipeline/tests/boot_stages_bench.rs` (330 lines, ceiling ~450),
`crates/pipeline/tests/common/mod.rs` (142 lines, ceiling ~180), the
measurement doc, the journal. Doc comment carries the committed MEASUREMENT
TABLE beside the consts (A4 precedent, bench :43-65) - obligation met.
Largest function `boot_stages_release_timing` ~83 lines < 100. PASS.

## Commit-vs-touch-set

`git diff --name-only 42d4a6e..bd4fb1a` -> exactly the four declared touch-set
paths; `git show --stat bd4fb1a` -> `4 files changed, 755 insertions(+)`,
zero deletions. No path outside the touch-set. PASS.

## Frozen locks (each reviewer-proven)

- pipeline `src/` diff-EMPTY over the range; `load_perf_bench.rs` diff-EMPTY
  and still exactly 866 lines; `pipeline_test.rs`, `crates/cli/**`,
  `tools/**` (incl. `bench_baseline.json`), `tests/golden/**`,
  `tests/fixtures/**` all diff-EMPTY (per-path `git diff --stat`, empty).
- Pipeline suite green UNEDITED: scoped census identical in passed count to
  the V3 anchor (`25/0/2/5` -> `25/0/4/6`: +2 ignored = the two new
  `#[ignore]` benches, +1 suite = the new binary; zero existing tests
  touched).
- No committed corpus: E3-shape TU built in-test from numeric byte arrays
  (`write_e3_shape_fixture`, bench :287-309); `tests/fixtures` diff-EMPTY.
- Debug row in no gate tuple: emitted only by its own ignored test; T1b's
  documented set remains the eight release labels.

## Plan-literal checks (all PASS)

- Eight labels byte-exact vs shape sec. 3.3 lines 300-304 (bench :271-278;
  byte-identical emitted rows in the reviewer capture). T1b's
  `DOCUMENTED_PHASES_BOOT_STAGES` can be built from the bench source verbatim.
- Debug label byte-exact vs shape :305 (bench :97-98); emitted row byte-exact.
- W=N = `Pipeline::new(&path, Some(n))`, `n = min(8,
  available_parallelism)` (bench :122-127, :250-251) - today's library
  capability, production path proven via `production_stats` assert
  (:186-191). NOT new machinery.
- Consts 2000/400 (bench :88-90) = shape start settings; second pristine
  fixture for the W=N row is methodologically correct (pristine artifacts
  tree) and does not leak fixture-gen time into the row.
- `common/mod.rs` is a faithful parameterized COPY of
  `load_perf_bench.rs:40-141` (bodies identical; FANOUT_BASE 10,
  FANOUT_WIDTH 4, TU_ROOT_STRIDE 103, TU_ROOT_COUNT 5 identical) - shape
  sec. 9 duplication allowance honored.
- Launch-profile record (obligation b) verified against the live tree:
  AGENTS.md default `cargo build`, `ui_smoke` build step + `target/debug`
  default, extension docs `cargo run` - DEBUG de-facto finding CONFIRMED.
  E3 DEBUG proof verified at `tasks/tu-index-hang-throughput-t0-evidence.md`
  :32 (panic text, site `tokenizer.rs:331:21`) and :76-78.

## Independent measurement verification

- All eight table medians re-derived: each equals the middle of its three
  runs. Speedup 60553.4/85681.1 = 0.7067 -> 0.71; inverse 1.4150 -> 1.41x
  slower. ms/TU and projection verified (finding 1 for the display nit).
  "99% of measured cold boot work" re-computed: 60553.4/61288.3 = 98.8%.
- W=N-slower claim REPRODUCED on a reviewer-fresh release run via the
  handoff's invocation form (`python tools/distill.py -- cargo test
  --release ... boot_stages_release_timing --ignored --nocapture
  --test-threads=1`, exit 0, 179.88 s): all eight labels, W=1 63428.2 /
  W=N 86912.5 (ratio 0.73), inside the implementer run spread, direction
  stable. Machine header identical. The no-work decision on T3 is defensible
  on 4/4 runs across two sessions.
- Debug context row re-run fresh (gate 2 form, exit 0):
  `profile=debug debug_assertions=true`, row `... | 10.6` - the
  `done-post-T2` branch; pre-T2 panic correctly NOT re-captured, authoritative
  text cited from t0-evidence sec. 1 (verified present).
- Report-evidence check (distillation contract): journal + doc digest-shaped,
  capture paths cited, zero raw dumps, gate commands via distilling entry
  points. PASS.

## Deviation dispositions

- Deviation 1 (warm-row subtraction method swap after the -92.9 ms
  negative-anchor bug): ACCEPTED. Intent fidelity holds under independent
  check. The handoff literal ("re-running `rehash_inputs`-equivalent
  construction cost") is not the letter shipped, but the handoff itself
  equates the two (construction time == rehash_inputs-equivalent), and the
  literal naive form measured NEGATIVE - physically invalid, cache-state
  skew. The shipped anchored form subtracts the SAME-code-path construction
  cost measured back-to-back with the boot measure (settled page cache via
  throwaway construct), adds a `StageReuse::Loaded` assert that makes
  silent-rebuild contamination impossible - strictly stronger isolation of
  the artifact-load path, the handoff's stated intent; and the
  "state the subtraction method in the doc comment" requirement is honored
  (bench :24-36, doc :40-44). The invalidating run is excluded from medians.
- Deviation 2 (5 release runs vs 3; gate ceiling held at 10): ACCEPTED. The
  3-median requirement is met on the final code; the 2 extra runs are the
  Red run (the handoff's own RGB protocol) and the invalidated first Green
  (Deviation 1's artifact) - honest cost, disclosed in both journal and doc,
  total distill invocations 10 = ceiling, not over.

## Findings (numbered; no blockers, no required implementer changes)

1. MINOR - `tasks/tu-index-hang-throughput-t1-measurement.md:110`: the
   printed arithmetic is not self-consistent - "151.4 * 5744" evaluates to
   869,641.6, while the stated 869,547 is the full-precision
   151.3835 x 5744 = 869,546.8. The RESULT is correct; the displayed product
   is pre-rounded. Same class: :48-49 "~61.5 s measured stage total" is
   61.29 s. Suggested fix belongs in the orchestrator's finalization append,
   not a re-run.
2. MINOR (config, not the implementer) - reviewer gate surface: compound
   multi-pathspec `git diff -- a b c`, inline `python -c`, and PowerShell
   `Get-Content` were refused with the boilerplate global catch-all
   (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`);
   single-pathspec git, `tools/*.py`, and Read-tool substitutes completed
   every intended check. Recurrence of the open config-row family (ledger
   line 59 precedent). No gate skipped; verbatim refusals in the
   execution-results addendum.
3. MINOR (informational) - the distill capture-overwrite convention makes
   the 3-run medians un-re-verifiable from captures after the fact; the doc
   states the convention and the transcription promise, and the reviewer
   closed the gap with a fresh independent run landing inside the spread.
   Accepted as the established convention (T2-hop precedent).

## Follow-up dispositions (routed, not dropped)

- Launch-profile finding (DEBUG de-facto): route to a NAMED landing - a
  small doc/flag hop or T1b rider shipping a release-profile launch route
  (doc line / smoke flag). The M27 story "profile lever + fix" is not closed
  by recording the finding alone.
- Orchestrator: append the ratified bar + final ACCEPT line (verbatim above)
  under measurement doc sec. (d).
- Ledger: M27 row - T1a LANDED APPROVED; T3 status BLOCKED -> DORMANT BY
  EVIDENCE (gate ACCEPT ratified this hop); T4/DP-4 EVIDENCE-CLOSED; new
  census anchor `1235 passed / 0 failed / 6 ignored / 88 suites` @ `bd4fb1a`;
  memory mirror in the same commit.
- Operator purge queue unchanged: 5 pre-existing `tasks/v3-review-*.py`
  throwaways (M26 residual); this hop added none.

## M27 closeout-readiness

Remaining before M27 can close: (1) T1b - `bench_compare --bench boot-stages`
gate + `DOCUMENTED_PHASES_BOOT_STAGES` tuple over the eight byte-verified
labels; PR-2-bound (operator `--update-baseline` on a VERIFIED-IDLE machine
BEFORE T1b's live gate; implementers never run it) - dispatch CLEARED by this
verdict; (2) the profile-lever landing named above; (3) ledger flips + memory
mirror per follow-ups; (4) orchestrator's measurement-doc finalization append;
(5) milestone-lifecycle closeout ritual: execution-results final addendum,
walkthrough-builder hop (none exists yet), worktree census clean, operator
purge; (6) the standing user duty (re-index non-ASCII corpora post-T2 via the
DP-6 2.1.0 invalidation) remains in the ledger. With T1a landed APPROVED and
the gate ACCEPTED: T3 stays dormant by evidence, T4/DP-4 evidence-closed, and
M27's throughput story closes at "profile lever + fix" once (1) and (2) land.

## Gates run by this reviewer (verbatim digests)

See `tasks/tu-index-hang-throughput-execution-results.md` addendum 4 for the
full evidence; canonical lines quoted there verbatim: scoped
`25 passed / 0 failed / 4 ignored / 6 suites`; workspace
`1235 passed / 0 failed / 6 ignored / 88 suites`; tools `Ran 188 tests`;
`ui_smoke: PASS`; hygiene/encoding `### generic PASS (exit 0)`; release-bench
and debug-row re-runs exit 0 with the captures transcribed above.
`bench_compare`: NOT RUN - PR-2-bound to T1b (handoff lines 7-9, 224-226);
exemption stated in the addendum.
