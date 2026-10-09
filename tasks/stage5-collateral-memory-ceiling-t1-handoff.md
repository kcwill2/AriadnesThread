# T1 Handoff - M28 stage5-collateral-memory-ceiling: bounded-capacity LoadMemo + capacity-invariance oracle (frozen BEFORE the swap)

Hop id: `t1-bounded-memo`. The milestone's swap oracle hop: the
capacity-invariance equality suite (I5) lands RED-first against the CURRENT
append-only behavior, stays green forever, and only then does the eviction
policy ship. Query results must be identical for every capacity - eviction
may trade time, never bytes.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. Shape `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`
   sec. 2 (I5/I7c wording), sec. 3.2 (the additive constructor - FROZEN
   literal), sec. 0 A1/A3 (what grows and why caps are safe at boot).
3. `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` - THE SUPPLIER:
   sec. RATIFICATION + ROUND-2 ERRATUM (2026-10-06, landing `39da2e9`/
   `afcd038`) supplies the ratified milestone constants: N0 = 1280 fixture
   files, scales {640, 1280}; peak bars `stage5 peak RSS` <= 5200 MiB per
   profile at N=1280; growth gate `stage5 RSS growth ratio` <= 2.0; gate
   protocol median-of->=5 isolated invocations with >8% per-slot-median
   operand rejection (binding on BOTH operands at BOTH scales); the
   ~4096 MiB save-span transient probe as a T0b/T1 rider probe. PER-CACHE
   CAPS AND THE I7c PIN CONSTANT ARE NOT IN THE RATIFIED SET (relay
   2026-10-06): this hop fixes them as literals (`HOP_CAP` / I7c pin) with
   a written justification paragraph in the journal, and the landing
   reviewer adjudicates them - cite them as hop-fixed, NEVER as a review
   ratification (DP-4 honesty: no invented constant silently).
4. `crates/graph/src/load_caches.rs` (full file; doc-dlock-discipline block
   :7-19 stays TRUE after your change; `new` :98, `with_closures` :119,
   counters :301-311, both cached paths :189-209 and :378-403).
5. `crates/graph/src/load.rs` :535-580 only if the I5 suite needs an
   analyzer seam (default: untouched).
6. `crates/pipeline/src/internal_tests.rs` :122-185 (report-equality oracle
   you must keep green UNEDITED - you must not touch the file at all).
7. `crates/graph/tests/m22_marker_unit_shaper_test.rs` (test-file style +
   `*_test.rs` naming), `docs/specs/tool-output-distillation.md`.

## Preconditions (hard - encoded here, enforced at dispatch)

1. SATISFIED 2026-10-06. T0 landed APPROVED (T0a round-1 REJECTED at
   `8d34a3c` - blocker: ratio row not decomposable; round-2 APPROVED at
   `69885ce`, verdict `tasks/stage5-collateral-memory-ceiling-t0a-verdict.md`)
   and the WRITTEN DP-4 EVIDENCE REVIEW CONCLUDED 2026-10-06 (evidence doc
   sec. RATIFICATION + ROUND-2 ERRATUM, landing `39da2e9`/`afcd038`). Per
   the review's dispatch relay T1 is FIRED and DISPATCHABLE NOW: the
   measured table names the symbol-pass retention / permanent-memo growth
   class (A1/A3) among the dominant contributors, which is exactly this
   hop's axis. Per-cache caps + the I7c pin are NOT review-ratified - this
   hop fixes them (item 3, journal justification, reviewer adjudicates).
   The measure-before-machinery law is satisfied by the completed,
   ratified table; no constant is invented silently.
2. Dispatched in parallel with T2 only under the shape's condition (sec. 1:
   crate-disjoint FILES - graph vs pipeline; the boot-memo cap WIRING is
   T2's obligation, so if the review wired cap usage into T2's budget, T2's
   cap-wiring clause additionally preconditions on THIS hop landing).
3. Zero fixture demand: the I5 suite builds its models in-code / temp dirs.
   Nothing new inside `tests/fixtures/basic` / `tests/fixtures/expensive`;
   fixture demand routes through the documented frozen-set approval path
   (M27 T0b dated-row precedent) - not implemented inside this hop.

## Objective and invariants

Objective: bounded entry-count caps on the two append-only memo caches
(`semantic_closure_cache`, `physical_expansion_cache`) behind an additive
constructor; cap value `0` = pure recompute (disables that cache - doubles
as T0b's ablation mode). Eviction policy is implementer-chosen (the shape
declares policy implementer's own), bounded by these invariants:

- I5 (the oracle): every `analyze_*` result is `==`-identical across memo
  capacities `{0, 1, HOP_CAP, usize::MAX}` on every locked fixture
  (`HOP_CAP` = the hop-fixed boot cap, item 3) -
  authored as the RED suite BEFORE any policy swap and kept forever. The
  capacity-0-vs-capacity-MAX equality makes the oracle self-referential: no
  pre-swap snapshot, and it falsifies exactly the axis the fix moves.
- I7c (CI clause): `semantic_closure_cache_len()` /
  `physical_expansion_cache_len()` (existing test-instrument counters,
  `load_caches.rs:301-311` - keep them truthful, do not repurpose them)
  are `<=` each cache's cap at all times; a committed small-fixture,
  non-ignored test asserts the counters stay `<=` the hop-fixed I7c pin
  constant (item 3: NOT review-ratified - hop-fixed literal with journal
  justification, landing reviewer adjudicates; relay 2026-10-06) after a
  full pipeline-scale query run. Deterministic,
  machine-independent, runs inside `cargo test` - this is the ONLY memory
  clause that may run in CI (DP-3).
- Lock discipline (doc :7-19) stays TRUE verbatim: no guard held across
  another acquisition, no guard across a rayon job boundary. Whatever the
  eviction scheme, `Mutex` guards stay single-statement-shaped.
- Append-only doc comments (:46-64 "entries are permanent") get their
  wording updated HONESTLY to describe cap behavior; the immutable-inputs
  staleness argument (:21-24) is unchanged and its doc stays.
- Every existing caller unchanged: `new()`, `with_closures()`,
  `Default` keep exact today's behavior (usize::MAX-equivalent semantics -
  uncapped). `crates/cli` ui C1 memo lock and pipeline internal_tests pass
  UNEDITED.
- If the boot passes need a capped ANALYZER handle (shape 3.2's boot-memo
  cap is consumed by T2, not you), add the matching ADDITIVE
  `LoadAnalyzer` constructor in `load.rs` delegating to your memo ctor;
  the journal records why the seam was needed. If unused downstream after
  the review, still add it ONLY with that citation - no speculative API.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/stage5-collateral-memory-ceiling-t1-journal.md` (checkpoint
  journal; always in-set; create alongside)
- `crates/graph/src/load_caches.rs` (caps + additive `with_closures_and_capacity`
  + honest doc updates)
- `crates/graph/src/load.rs` ONLY the additive analyzer-constructor seam
  described above (expected ~20 lines; cite the need in the journal)
- `crates/graph/tests/load_memo_capacity_invariance_test.rs` (NEW - the I5
  suite + I7c cap test; extend the same file rather than scatter)

You MUST NOT touch: `crates/graph/src/load_explain.rs`; graph builders;
`crates/pipeline/**` (boot-memo WIRING is T2's hop - you ship the knob, T2
turns it); `crates/cli/**` (ui C1 lock stays green UNEDITED - that is the
proof your additive API did not shift behavior); `tests/**` incl.
fixtures; `tools/**`; `tasks/milestone_ledger.md`. No `git stash`, no
branch change. `Cargo.toml` unchanged (no new deps; `test-instrument`
feature already exposes the counters you need).

## Frozen locks

Kickoff sec. 5 table verbatim (goldens/fixtures; m00 parity `2 passed`
UNEDITED; m18 parity `23 passed / 0 failed / 0 ignored / 1 suites`
UNEDITED; `progress.rs` byte-identical; stage-5 13-frame replay UNEDITED;
boot contract append-only + M26/M27 appended tests UNEDITED;
`CURRENT_ANALYSIS_VERSION` 2.1.0; `bench_baseline.json` operator-only;
census anchor `1235 passed / 0 failed / 6 ignored / 88 suites @ bd4fb1a`).
Plus this hop's own locks: `crates/pipeline/src/internal_tests.rs`
diff-EMPTY (report-equality oracle green UNEDITED on top of the new policy -
that IS the acceptance witness); the I5 suite file, once landed, is a
forever-lock: later hops' reviewers re-run it UNEDITED. Proving commands:
reviewer `git diff` scope + the scoped suites below.

## RGB protocol

1. Red (the oracle FIRST, against the UNCHANGED cache code): author
   `load_memo_capacity_invariance_test.rs`. For each locked in-repo fixture
   the graph crate can already build (basic/expensive fixture models via
   the public builders - enumerate them in the journal), run the analyze
   path with capacities {0, 1, HOP_CAP, usize::MAX} and assert pairwise
   `==` of the `LoadReport`-level results, plus the counter-cap assertion
   (`cache_len() <= cap` after a full query sweep, the cap literals
   hop-fixed per item 3 with their journal justification). At this point `with_closures_and_capacity` does not
   exist, so Red = the capacity axis: run the SAME fixture with cap-0
   behavior simulated via a tiny in-test zero-memo harness, OR accept
   "compile-red is not red" and land instead the FIRST compile-green slice:
   add the constructor delegating to uncapped behavior (a no-op stub is
   allowed ONLY as the shape-legal additive seam because it changes no
   observable behavior - journal the rationale), then the I5 suite goes
   RED on the cap-0 axis (results equal but counters show memoization
   still on: `semantic_closure_cache_len() > 0` under cap 0 - expected
   failure message names the cap-0 case). Capture the failing test list.
   A test that fails to COMPILE is not Red.
2. Green: implement the eviction/cap policy minimally (0 = skip
   memoization entirely, results computed fresh per query; N = keep the cap
   invariant on every insert; policy choice + rationale in the doc comment
   in <= 5 lines). Re-run the suite to green. NEVER weaken an assertion.
3. Refactor: guard-shape discipline audit against doc :7-19 (quote the doc
   back in the journal and point at every guard you touched); honest names
   (`semantic_closure_cap`, `physical_expansion_cap`); no `tmp`/`res`.

## Required tests

- I5 equality matrix (per fixture, all capacity pairs) - non-ignored.
- I7c cap test (counters `<=` the hop-fixed I7c pin, item 3, journal
  justification cited) -
  non-ignored (it is the CI clause).
- Cap-0 recompute test: zero-cap memo answers identically AND
  `physical_expansion_count()` grows on repeats (memo off = every query is
  a real expansion - the ablation contract T0b reuses).
- Existing graph suite + `m22_marker_unit_shaper_test.rs` UNEDITED green.

## Gates (exact commands, timeout 600000 ms each; all via distilling entry
points - subagent sessions cannot invoke gate commands directly: ledger row
60; proven route `python tools/distill.py -- python tools/ui_smoke.py`)

1. `python tools/testrun.py --distill -p ariadne-graph` - the RED capture
   (cap-0 assertion failing) first, then GREEN; canonical line verbatim in
   the report; baseline green truth per the pre-dispatch tree, journal the
   exact line + your suite delta.
2. `python tools/testrun.py --distill -p ariadne-pipeline` - UNEDITED
   internal_tests report-equality oracle green (behavior witness; MUST
   lines quoted from GREEN-tree runs only - row-65 erratum).
3. `python tools/hygiene_check.py` - exit 0.
4. `python tools/encoding_guard.py --staged` - exit 0.
5. Commit: `python tools/commit_scoped.py --scope
   crates/graph/src/load_caches.rs --scope crates/graph/src/load.rs
   --scope crates/graph/tests/load_memo_capacity_invariance_test.rs
   --scope tasks/stage5-collateral-memory-ceiling-t1-journal.md -m
   "feat(graph): bounded-capacity load memo + capacity-invariance oracle
   (M28 T1)"` (drop the load.rs scope if the seam was not needed - the
   commit scope must match the diff exactly).

Total implementer invocations: <= 8 counting red/green re-runs inside gate 1.

REVIEWER hop obligations (NOT implementer gates): `python tools/testrun.py
--distill -p ariadne-cli` (ui C1 test-instrument memo lock green UNEDITED);
workspace census `python tools/testrun.py --distill` reconciling from the
`bd4fb1a` anchor (`1235 passed / 0 failed / 6 ignored / 88 suites`) with
deltas = your new tests only (the `--baseline` FILE form is unrunnable -
standing errata row 59: `--distill` census substitute); full tools
discover CI form `python tools/distill.py --kind generic -- python -m
unittest discover -s tools -p "*_tests.py"` (the `-p` pattern is REQUIRED:
the default pattern `test*.py` discovers only 30 tests, so the expected
`Ran 188+N ... OK`-class counts are reachable ONLY in the CI form - M28 T3
verdict finding 1); `python tools/distill.py --
python tools/ui_smoke.py`; `encoding_guard --all`; per-commit
`git show` oldest-first vs touch-set; the stage-5 13-frame replay +
m00/m18 proofs. The `--bench stage5-memory` live run and the post-T1
ablation re-run of the curve (shape sec. 6 item 3, "T0b's curve may re-run
it post-T1") are operator/reviewer duties on an idle machine - LOCAL gate
only (DP-3), never implementer commands, never CI.

## Hygiene / ceilings at risk

`load_caches.rs` 410 -> ~550 lines (ceiling 1000 - margin 590 today);
functions < 100 (the two cached paths stay guard-shaped); no new file over
800 lines (budget the I5 matrix as data-driven loops, not copy-paste
per-fixture bodies). Pure ASCII; house-style names.

## Discipline clauses

- UTF-8: Write/Edit tools only; pure ASCII; no PowerShell content pipes.
- Touch only the touch-set; out-of-scope: report, do not fix.
- No bare cargo/`ui_smoke`/`bench_compare`; distilling entry points only;
  cite digest header + canonical line + capture path; zero raw pastes (R9).
- Journal after each Green step (1-3 lines). Context wall: STOP +
  disk-state inventory; successor: `python tools/checkpoint_guard.py
  resume --spec tasks/stage5-collateral-memory-ceiling-t1-handoff.md
  --scope crates/graph --cmd …`.

## Acceptance (machine-checkable verdict lines)

- I5 suite green UNEDITED post-swap (its pre-swap Red capture in the
  report proves the oracle ran against the old behavior - review-gate
  checks Red came first per this file's order).
- I7c cap test green with the hop-fixed pin literal and its journal
  justification present (reviewer adjudicates the chosen caps/pin).
- Gate 2 pipeline canonical line shows UNEDITED internal_tests green.
- `git show --stat` = touch-set exactly; `internal_tests.rs` and every
  locked path absent from all diffs.
- Reviewer: census reconciles from the `bd4fb1a` anchor; ui C1 lock green;
  `ui_smoke: PASS` via distill.
Reviewer gate: `review-gate` skill; commit-vs-touch-set enforces this file;
the reviewer additionally checks the guard-discipline journal quote against
doc :7-19 and that the eviction policy never crossed I5/I7c.
