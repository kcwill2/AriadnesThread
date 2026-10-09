# T0a Handoff - M28 stage5-collateral-memory-ceiling: RSS-curve confirmation bench (debug + release columns)

Hop id: `t0a-rss-bench`. Measurement hop, not a machinery hop: it lands the
peak-RSS oracle the milestone is named after, before any fix commit exists
(shape sec. 1 sequencing law - measure before building machinery).

USER-RESOLVED DP decisions binding on this hop (2026-10-05, carry verbatim):
- DP-4: N0, the I7a peak bars, the I7b growth cap G, the I7c capacity
  constant, and the gate ratio are RATIFIED from this hop's measured
  debug+release table at a written evidence review - this hop measures and
  proposes, it NEVER ratifies and never invents a gate constant. N0 enters
  this hop as the shape's PROPOSED value only.
- DP-5 = synthetic only: NO real-corpus capture mode, no operator RSS
  capture is requested. The scaled hub-heavy fixture bench is the sole curve
  source (shape sec. 3.1 optional mode is NOT built; DP-5 was answered NO to
  the optional capture).

Dispatch precondition: NONE. Dispatchable immediately, in parallel with the
T3 rider (`tasks/stage5-collateral-memory-ceiling-t3-handoff.md`) - the two
touch-sets are disjoint (this hop: `crates/cli/tests/` + one evidence doc +
journal; T3: `tools/` + `AGENTS.md`). T0b dispatches on THIS hop's label
stability.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. Shape `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`
   sec. 3.1 (row grammar), sec. 6 (the six table blocks to produce), sec. 7
   (bench hygiene rules), sec. 0 A1-A7 (what each row attributes to).
3. Kickoff `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-kickoff.md`
   sec. 2 E-rows (the failure class the curve must reproduce), sec. 5 (locks).
4. `crates/cli/tests/payload_tail_bench.rs` - mirror this file's structure:
   verbatim-duplicate generator precedent (:8-10), `label | value` row
   printer, `#[ignore]` + `--ignored` run form, fixture in a temp dir.
5. `crates/pipeline/tests/load_perf_bench.rs` :26-44 (generator constants
   `HEADER_COUNT = 520`, `TU_COUNT = 40`, fanout scheme to re-scale) -
   MUST NOT GROW (M27 rule); read it, never edit it.
6. `tools/bench_compare.py` :88-109 (the parse grammar you must not break:
   exactly one `|`, finite numeric value) and :43-58 (`DOCUMENTED_PHASES`).
7. `crates/graph/src/load_caches.rs` :301-311 (`semantic_closure_cache_len`,
   `physical_expansion_cache_len` - existing `test-instrument` counters you
   sample for attribution), `crates/graph/Cargo.toml` :7-12 (feature gate).
8. `docs/specs/tool-output-distillation.md` (entry points + digest shapes).

## Objective and invariants

Objective: one new ignored bench `crates/cli/tests/stage5_memory_bench.rs`
that prints, per shape sec. 3.1 grammar, `label | value` rows with values in
MiB and the profile INSIDE the label, produced on a hub-heavy scaled fixture
(a few high-fanin headers so closures are corpus-scale - the E1 shape), run
in BOTH profiles, and transcribe the measured tables into the new evidence
doc `tasks/stage5-collateral-memory-ceiling-t0-evidence.md`. The six table
blocks (shape sec. 6): (1) peak RSS at scales {N0/2, N0} + growth-ratio rows
`RSS(N0)/RSS(N0/2)`; (2) per-phase RSS checkpoints (prep / symbol-pass end /
file-pass tail / save tail / post-return); (3) capacity-0 ablation rows;
(4) memo entry counters x sampled entry size (attribution arithmetic printed
in the table); (5) warm-deserialize peak row (committed small-fixture
artifact only - real-corpus blob sizes quoted, never shipped); (6) context
rows: debug/release wall medians at N0.

Invariants this hop must not cross:
- I1-I4 frozen floors (kickoff sec. 5): stage-5 13-frame replay green
  UNEDITED, `progress.rs` byte-identical, boot contract tests append-only,
  fixtures/goldens/m00/m18 UNEDITED. A measurement hop touching only a new
  test file cannot move them - the reviewer proves it with diff scope.
- I6 untouched: you add no events and change no producer.
- Existing timing gate intact: `payload_tail_bench.rs` semantics unchanged,
  `bench_compare.py` default-mode parse unaffected (your rows use the same
  single-pipe numeric grammar so the default tuple simply never sees them -
  they live in a different bench file).
- Every RSS table row carries BOTH profile columns (E5/M27-E3: the profile
  confounds any measurement; a debug-only table is an acceptance failure).
- Proposed scale: N0 PROPOSED = 1280 collateral files, curve scales
  {640, 1280} (shape sec. 3.1/6 - proposal for the evidence review to
  ratify against runtime budget). Parameterize the scale via an environment
  variable (suggested `STAGE5_MEM_BENCH_FILES`, default 1280) so the review
  can re-run other scales without an edit. Do not add any other constant
  that smells like a gate threshold.
- Capacity-0 ablation (block 3): the pre-fix tree has no capacity knob (T1
  has not landed - sequencing law). Produce block 3 ONLY via a scratch local
  build (journal the scratch diff, do NOT commit it, do NOT touch
  `crates/graph/**`); if the scratch ablation is not affordable this
  session, mark block 3 "DEFERRED to post-T1 T0b re-run" in the evidence
  doc - the shape sanctions exactly this reading path (sec. 6 item 3).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/stage5-collateral-memory-ceiling-t0a-journal.md` (checkpoint
  journal; always in-set; create alongside)
- `crates/cli/tests/stage5_memory_bench.rs` (NEW - generator + RSS probe +
  row printer; the sanctioned verbatim-duplication pattern,
  `payload_tail_bench.rs:8-10` precedent)
- `crates/cli/Cargo.toml` ONLY if a dev-dependency is unavoidable for the
  RSS probe (see gate budget - prefer zero); any such addition requires a
  written justification paragraph in the journal (shape sec. 3.1)
- `tasks/stage5-collateral-memory-ceiling-t0-evidence.md` (NEW; you create
  it with the MEASURED sections only; see evidence-doc rule below)
- `Cargo.lock` ONLY if and only if a dev-dependency landed (git regenerates
  it; zero-diff is the default expectation)

You MUST NOT touch: any `crates/*/src/**`; `crates/pipeline/**` (incl.
`load_perf_bench.rs` - MUST NOT GROW); `crates/cli/tests/payload_tail_bench.rs`;
`crates/graph/**` (ablation is a SCRATCH build, never an edit); `tools/**`;
`tests/**` incl. `tests/fixtures/**` (ZERO fixture demand this hop - the
generator writes only under a `tempfile` temp dir; if you believe you need a
fixture, STOP and report: new fixtures route through the documented
frozen-set approval path, M27 T0b dated-row precedent, and are not
implemented inside a hop); `tasks/milestone_ledger.md` (planner-owned);
`bench_baseline.json`. No `git stash`, no branch change.

## Frozen locks (each with its proving command - reviewer re-runs these)

Inherited M26/M27 lock table (kickoff sec. 5 verbatim): goldens/fixtures
byte-frozen; m00 parity UNEDITED; m18 parity `23 passed / 0 failed / 0
ignored / 1 suites` UNEDITED; `progress.rs` byte-identical; stage-5 13-frame
replay `test_stage_five_milestone_sequence_is_monotonic_and_contained` green
UNEDITED; boot contract tests APPEND-ONLY with M26/M27 appended tests
UNEDITED; `CURRENT_ANALYSIS_VERSION` 2.1.0; `bench_baseline.json`
operator-only (PR-2); census anchor `1235 passed / 0 failed / 6 ignored /
88 suites @ bd4fb1a`. Proof at this hop: `git diff --name-only
<start-rev>..HEAD` equals the touch-set exactly (commit-vs-touch-set); none
of the locked paths can appear. `bench_compare.py` default mode must still
parse `payload_tail_bench` rows unchanged - your bench never feeds it.

## Evidence-doc rule (supplier discipline, DP-4)

`tasks/stage5-collateral-memory-ceiling-t0-evidence.md` is the SUPPLIER
artifact for every downstream constant: T0b's label set and gate ratio, T1's
caps, T2's pass budget, T4's dispatch decision. You fill the MEASURED
sections (six blocks, both profiles, run-count + idle-ish notes, medians-of
-N >= 3 inside the bench where the shape asks medians) and a
`PROPOSED:` line per ratifiable quantity. You do NOT write any
`RATIFIED:` line - those are appended by user + orchestrator at the written
evidence review (M27 precedent:
`tasks/tu-index-hang-throughput-t0-evidence.md`). From your first write the
doc is APPEND-ONLY.

## RGB protocol

1. Red (the RSS probe first, before any generator): a non-ignored test
   `peak_working_set_bytes_tracks_allocation` in the new file that reads
   peak RSS, allocates a known ~256 MiB slab, and asserts the peak moved by
   at least ~200 MiB. Write the probe as raw `extern "system"` FFI to
   `K32GetProcessMemoryInfo` (psapi) inside the test file - zero new
   dependencies, the shape's named fallback (sec. 3.1). FIRST check whether
   a windows-surface crate is already reachable from this crate's test
   target (transitive lock presence is NOT reachability); only if it is,
   prefer it, and journal the check. Expected first failure: the probe
   panics/asserts a delta of ~0 (probe returns the static peak before the
   slab) - a compile error is NOT red.
2. Green: implement probe + hub-heavy generator (scales parameterized,
   hub-heavy = a small set of high-fanin headers, unlike the 4-wide fanout
   of `load_perf_bench.rs`; the E1 shape means closure sizes ~ corpus
   scale) + the six blocks' row printer + `#[ignore]` on curve tests. Then
   run the DEBUG and RELEASE benches (gates 2-3) >= 3 runs each, transcribe
   medians into the evidence doc. NEVER weaken an assertion to pass.
3. Refactor: bounded table only (R9/R10: no raw dumps, no > 2000-char rows,
   no corpus paths beyond temp dirs); honest names (house style:
   `stage5_memory_bench`, no `tmp`/`res` locals); the bench must print its
   rows even when a checkpoint probe is unavailable, degrading loudly not
   silently.

## Required tests

- `peak_working_set_bytes_tracks_allocation` - non-ignored, cheap, locks the
  oracle itself (fails loudly on any platform where the probe is a stub).
- Ignored curve tests: one per scale x profile block; each prints the exact
  label set of shape sec. 3.1 (e.g. `stage5 peak RSS (debug, N=1280)`,
  `symbol-pass RSS delta (release, N=1280)`,
  `memo semantic-closure entries (release, N=1280)`,
  `warm deserialize peak (release)`) - these literal labels are the INPUT
  to T0b's `DOCUMENTED_PHASES_STAGE5_MEMORY` tuple; the journal MUST list
  them verbatim as the label proposal.
- Warm-deserialize row runs against a small-fixture (e.g. 40-file) artifact
  generated in a temp dir - never a committed blob.

## Gates (exact commands, timeout 600000 ms each unless noted; all via
distilling entry points - subagent sessions CANNOT invoke gate commands
directly: ledger deferred-findings row 60, proven route
`python tools/distill.py -- python tools/ui_smoke.py`)

1. `python tools/testrun.py --distill -p ariadne-cli` - non-ignored suite
   green, `0 failed` (MUST lines from GREEN-tree runs only - row-65
   erratum). Baseline cli truth at dispatch: `110 passed / 0 failed /
   1 ignored / 5 suites` (t1a handoff gate 2, green); after your hop expect
   the ignored/suite deltas of exactly your new benches; journal the exact
   new line.
2. `python tools/testrun.py --distill -p ariadne-cli -- --test stage5_memory_bench` -
   the non-ignored probe test green.
3. Debug curve (each run one invocation; allow `--timeout 3600` on distill
   per the contract's timeout row, and budget a first-run rebuild):
   `python tools/distill.py -- cargo test -p ariadne-cli --test stage5_memory_bench -- --ignored --nocapture --test-threads=1`
   - >= 3 runs.
4. Release curve: same form with `cargo test --release ...` - >= 3 runs.
5. `python tools/hygiene_check.py` - exit 0.
6. `python tools/encoding_guard.py --staged` - exit 0 before committing.
7. Commit (direct route - sanctioned precedent M27 T0b `4ef8412` / T2 D1 /
   T5): `python tools/commit_scoped.py --scope crates/cli/tests/stage5_memory_bench.rs
   --scope tasks/stage5-collateral-memory-ceiling-t0-evidence.md --scope
   tasks/stage5-collateral-memory-ceiling-t0a-journal.md [-scope
   crates/cli/Cargo.toml -scope Cargo.lock if justified] -m "test(cli):
   stage5 RSS-curve bench + debug/release measured table (M28 T0a)"`.

Total implementer invocations: <= 10 counting re-runs inside gates 3/4.

REVIEWER hop obligations (NOT implementer gates; reviewer runs them fresh):
workspace census `python tools/testrun.py --distill` reconciling FORWARD
from anchor `1235 passed / 0 failed / 6 ignored / 88 suites @ bd4fb1a`
(the literal `--baseline` FILE form is unrunnable - standing errata,
ledger row 59: use the `--distill` census substitute); full tools
`python tools/distill.py --kind generic -- python -m unittest discover -s
tools` (`Ran N ... OK`); `python tools/distill.py -- python tools/ui_smoke.py`
-> `ui_smoke: PASS`; `encoding_guard.py --all`; per-commit `git show`
oldest-first vs touch-set; the locked-files byte-diff proofs above. The
`bench_compare --bench stage5-memory` gate does not exist until T0b and is
never an implementer command in this milestone (DP-3: LOCAL-gate-only,
operator-run on VERIFIED-IDLE).

## Hygiene / ceilings at risk

Files < 1000: your new file is the only risk if the generator sprawls -
budget ~600-800 lines including tables; split the row printer into
< 100-line functions. No banned names (R3). RSS numbers as `format!("{:.1}")`
MiB rows, never 2000-char lines. Authored content pure ASCII.

## Discipline clauses

- UTF-8: Write/Edit tools only, never Set-Content/Out-File/pipes; pure
  ASCII (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- No bare `cargo test`/`build` - everything through the distilling entry
  points (`docs/specs/tool-output-distillation.md`); digest header +
  canonical line + capture path cited, zero raw pastes (R9).
- Journal 1-3 lines after EACH Green gate (module + commit, deviations,
  label-proposal confirmations). If context exhaustion looms (M23/M27 wall):
  STOP, return a disk-state inventory; a dead session is NEVER resumed -
  successor runs `python tools/checkpoint_guard.py resume --spec
  tasks/stage5-collateral-memory-ceiling-t0a-handoff.md --scope crates/cli/tests --cmd …`.

## Acceptance (machine-checkable verdict lines)

- Gates 1-2 exit 0 with `0 failed`; gates 3-4 exit 0 with the full literal
  label set present >= 3 runs each, debug AND release rows non-empty
  (a table missing either profile column is REJECTED - E7/R7 discipline).
- Evidence doc: six blocks present (block 3 may be the sanctioned DEFERRED
  line), `PROPOSED:` lines for N0 / bars / G / counter cap, zero
  `RATIFIED:` lines authored by the implementer.
- `git show --stat <landing>` = touch-set exactly; no path under
  `crates/*/src`, `tools/`, `tests/` appears.
- Census reconciles forward from `1235/0/6/88 @ bd4fb1a` with deltas equal
  to your added tests only.
- Downstream gates unblocked by acceptance: T0b (labels stable) and the
  written evidence review (DP-4 ratification; DP-2 dispatch decision for
  T4's placeholder row).

Reviewer gate: adjudicate via `review-gate` skill (fresh full verification,
per-commit `git show` oldest-first, commit-vs-touch-set vs this file,
locks byte-identical). On APPROVED, relay to planner for the ledger hop and
to the orchestrator to convene the evidence review.
