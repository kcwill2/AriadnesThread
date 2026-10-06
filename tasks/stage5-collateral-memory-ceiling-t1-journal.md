# M28 T1 journal - bounded-capacity LoadMemo + capacity-invariance oracle

Hop id `t1-bounded-memo`. Handoff:
`tasks/stage5-collateral-memory-ceiling-t1-handoff.md`.
Session date: 2026-10-06 (UTC).

## Preconditions verified at dispatch (by this implementer)

- HEAD `afcd038` on `main`; the ratified evidence-review landings
  `39da2e9` (RATIFICATION + ROUND-2 ERRATUM written) and `afcd038`
  (journal hash row) are ancestors. T0a verdict `69885ce` APPROVED.
  Measure-before-machinery law satisfied by the ratified table.
- `git diff --stat HEAD -- crates/graph crates/pipeline crates/cli tools tests`
  diff-EMPTY at kickoff (via `python tools/distill.py --`).
- No dirty edits in any locked path; worktree carries only pre-existing
  untracked/modified orchestrator-side files (ledger drafts etc.) outside
  this hop's touch-set, untouched by this session.

## Baseline green truth (pre-dispatch tree)

`python tools/testrun.py --distill -p ariadne-graph`
-> `103 passed / 0 failed / 0 ignored / 3 suites` (exit 0, capture
`target/testrun-capture.txt`). Suite deltas below will be quoted against
this line.

## Plan and ordering (per handoff RGB protocol)

1. First compile-green slice (handoff-sanctioned): add the additive
   `LoadMemo::with_closures_and_capacity` accepting the caps but delegating
   to the uncapped behavior. Zero observable behavior change; the only
   legal way to make the capacity axis compile-observable (the handoff
   names "compile-red is not red" as the alternative to an in-test
   zero-memo harness).
2. Author the frozen I5 suite + I7c cap test + cap-0 ablation test in
   `crates/graph/tests/load_memo_capacity_invariance_test.rs` and capture
   RED on the cap-0 axis.
3. Green: enforce caps (first-N admission policy) in `load_caches.rs`.
4. Refactor: guard-shape audit quoting the doc-dlock block verbatim.
5. Gates + scoped commits (stub slice first, then suite+policy+journal).

## Fixture models (precondition 3: in-code, zero fixture demand)

Deviation - see DEVIATIONS D1: the handoff's parenthetical
"basic/expensive fixture models via the public builders" does not exist in
`crates/graph` (no public fixture-model builder; the real
`tests/fixtures/basic|expensive` trees need pipeline-level indexing, which
graph must not depend on). The suite replicates the crate's LOCKED fixture
SHAPES in-code instead, mirroring (enumerate per handoff):

- `chain_fixture`     - mirrors `setup_chain_fixture`
  (`crates/graph/src/load_perf_tests.rs:13`, same shape as
  `load_justification_tests.rs:12`): 4 headers, 7 symbols, chain + collateral.
- `seed_fixture`      - mirrors `setup_seed_fixture`
  (`crates/graph/src/load_perf_tests.rs:104`): 3 headers, 6 symbols,
  empty file_symbols (empty-file path).
- `concurrency_fixture` - mirrors `setup_concurrency_fixture`
  (`crates/graph/src/load_perf_tests.rs:260`): 9 files, 17 symbols, two
  include sub-trees.
- `impact_fixture`    - mirrors `setup_impact_fixture(&[], false)`
  (`crates/graph/src/load_impact_tests.rs:73`): main.cpp TU + hub fan-in.
- `hub_scale_fixture` - NEW in-code pipeline-scale sweep model: 41 files,
  243 symbols, hub-heavy (39 headers include `hub.h`, `main.cpp` includes
  all) so semantic closures are corpus-shaped relative to the fixture.
- `adversarial_wide_fixture` - NEW in-code demand model: 700 independent
  files/symbols, deliberately > both hop caps, so the counter cap is
  falsifiable at all (see D4 for why this must exist).

## DEVIATIONS

- D1: fixture-model parenthetical - handled as above (in-code replication
  of locked shapes; precondition 3 explicitly permits in-code models).
- D2: `crates/graph/src/load.rs` seam NOT added and dropped from the
  commit scope (handoff gate 5 pre-authorizes the drop when the seam is
  unneeded). Reason: the existing public `LoadAnalyzer::with_memo` (B1,
  `load.rs:201`) already accepts an external `Arc<LoadMemo>`; the capped
  memo flows through it with zero new API, and the boot-cap CONSUMER is
  T2, which can wire `LoadMemo::with_closures_and_capacity` + the existing
  seam. Adding a third constructor would be the speculative API the
  handoff forbids.
- D3: counter assertions (`semantic_closure_cache_len`,
  `physical_expansion_cache_len`, `physical_expansion_count`,
  `semantic_bfs_traversal_count`) exist only under `cfg(any(test,
  feature = "test-instrument"))` and `Cargo.toml` is frozen, so the
  counter-consuming tests are `#[cfg(feature = "test-instrument")]`-gated.
  The RED/GREEN captures run the scoped form
  `python tools/testrun.py --distill -p ariadne-graph --features
  test-instrument` (a pass-through cargo form the distiller explicitly
  whitelists, `tools/testrun.py` `CARGO_VALUE_FLAGS`), and gate 1's exact
  command additionally runs the counter-free I5 equality matrix. Any
  workspace-wide `cargo test` build also enables the feature via
  `crates/ui/Cargo.toml`'s dev-dependency `features = ["test-instrument"]`
  (feature unification), so the I7c CI clause runs in the census build
  too - the same mechanism the ui C1 lock relies on.
- D4: the hop-fixed pin cannot be CI-meaningful on the small fixtures
  alone (their demand is below every cap, so the assertion could never
  falsify), therefore the suite adds the `adversarial_wide_fixture`
  demand sweep INSIDE the same committed file: counters must stay `<=`
  each cap even when demand is 700 > cap. This is the falsifier for the
  I7c "at all times" clause and doubles as a capacity-invariance witness
  at cap < demand.
- D5: one scoped commit instead of stub-slice-then-green (see "Commit
  record"): `commit_scoped` commits whole files, so once the policy filled
  the same file the behavior-neutral stub state was no longer committable;
  Red-first is evidenced by the RED/GREEN capture pair over the identical
  frozen test file.

## Hop-fixed cap literals (NOT review-ratified - reviewer adjudicates)

Justification from `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md`
Block 4 (memo attribution, deterministic rows, N=1280):

- semantic-closure memo: 5024 entries, sampled entry size 621.5,
  est aggregate refs 3,122,416.
- physical-expansion memo: 1280 entries, sampled entry size 4944.0,
  est aggregate refs 6,328,320.

Boot query economics (shape sec. 0 A1/A3, ratified reading): the symbol
pass queries each seed once and the file pass each def-file exactly once,
so boot has essentially ZERO cache-to-cache reuse - an entry retained
after its single query is pure bloat (the A3 reading; post-boot UI routes
keep their own fresh uncapped memo via `ui.rs:289`, existing callers
unchanged by this hop). Result equality is guaranteed independently by the
I5 oracle, so the caps are a pure retention budget.

- `LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP = 512` entries: 512/5024 = 10.2%
  of the measured N=1280 uncapped entry count; retained refs <=
  512 x 621.5 ~= 318k = 10.2% of the 3,122,416 Block-4 aggregate.
  Power-of-two, entry-count cap per the frozen constructor contract
  ("bounded entry-count caps").
- `LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP = 128` entries: 128/1280 = 10.0%
  of the measured N=1280 uncapped entry count; retained refs <=
  128 x 4944.0 ~= 633k = 10.0% of the 6,328,320 Block-4 aggregate. Cut
  harder in relative terms because Block 4 names it "the first eviction
  candidate" (larger aggregate: ~6.33M vs ~3.12M sampled-scale refs).
  Boot caps therefore drop the combined memo retention from ~9.45M
  aggregate refs to <= ~0.95M sampled-scale refs at N=1280 (>10x), at
  zero query cost given the single-use boot access pattern.
- `LoadMemo::I7C_CACHE_LEN_PIN = 512`: the largest hop cap is the pin.
  Determinism: counters are admission-capped at SEMANTIC_CLOSURE_BOOT_CAP
  / PHYSICAL_EXPANSION_BOOT_CAP by construction (asserted in the same
  test), so `<= 512` holds on every machine independent of demand; the
  hub-scale sweep's own demand (243 seeds / 41 files) leaves 2.1x margin.

The 4096 MiB save-span transient probe (ratified as a T0b/T1 rider
PROBE): not mandated as a T1 gate by this handoff - gate 1-5 contain no
bench invocation, and the handoff's reviewer-hop obligations classify any
`--bench stage5-memory` live run as operator/reviewer duty on an idle
machine (DP-3, LOCAL gate only). Not executed here; the A7 save-span span
is also untouched by this hop's diff (graph memo only), so the probe
carries no T1 falsification power.

## RGB log

RED (oracle first, against the UNCHANGED cache code — the stub slice
carried zero behavior change):
`python tools/testrun.py --distill -p ariadne-graph --features
test-instrument`
-> `103 passed / 2 failed / 0 ignored / 2 suites` (capture
`target/testrun-capture.txt`). Failing test list (both counter-axis, the
semantic reason the handoff predicted — results themselves stayed EQUAL,
the equality matrix passed):
- `cap_zero_memo_is_pure_recompute_ablation` —
  `assertion left == right failed: cap 0 (fixture chain): semantic-closure
  memo must stay EMPTY - a non-zero length means memoization is still on
  at cap 0; left: 7` (names the cap-0 case, per handoff RGB step 1).
- `counters_stay_capped_when_demand_exceeds_every_cap` —
  `the semantic-closure counter must sit exactly AT the cap, never above;
  left: 700` (700 > hop cap 512 under the uncapped append-only policy).
(cargo fail-fast suppressed the m22 + doc-test binaries after the failing
binary — the 2-suite count is fail-fast, not a missing suite.)

GREEN (caps enforced): same command
-> `107 passed / 0 failed / 0 ignored / 4 suites` — baseline 103 + the 4
new tests, every pre-existing test UNEDITED and green.

Gate 1 exact command `python tools/testrun.py --distill -p ariadne-graph`
-> `104 passed / 0 failed / 0 ignored / 4 suites` (counter-free builds
compile in the equality matrix only, +1 test / +1 suite vs the
`103 / 0 / 0 / 3` baseline — deviation D3).

Gate 2 `python tools/testrun.py --distill -p ariadne-pipeline`
-> `25 passed / 0 failed / 4 ignored / 6 suites` — UNEDITED internal_tests
report-equality oracle green on top of the new policy (behavior witness;
quoted from the GREEN-tree run per row-65 erratum).

Gate 3 `python tools/hygiene_check.py` — R1/R2 clean (exit 0). The only
`load_caches.rs` R3 rows (:428/:433 `for f in def_files`) are PRE-EXISTING
loop variables outside this hop's diff; R3 is warn-only.

Gate 4 `python tools/encoding_guard.py --staged` — result quoted in the
commit section below.

## Policy and eviction (implementer's own, <= 5-line doc in code)

First-N admission: while a cache holds fewer than its cap every miss is
memoized; at the cap later misses recompute fresh and are never stored, so
an admitted entry is PERMANENT (nothing is ever replaced or evicted - no
staleness surface, no invalidation logic, order-independent results) and
only recomputation time is traded. Cap 0 leaves a cache permanently empty:
every query is real work (T0b's ablation mode, locked by
`cap_zero_memo_is_pure_recompute_ablation`). The uncapped constructors pass
`usize::MAX`, whose admission check can never fire - byte-identical
behavior to the pre-hop append-only policy for `new()`,
`with_closures()`, `Default` (the ui C1 memo lock and the pipeline oracle
pass UNEDITED - acceptance witnesses).

## Guard-shape discipline audit (handoff RGB step 3; doc :7-19 quoted)

Header doc (verbatim, unchanged, still TRUE): "No lock is ever held while
acquiring another (`OnceLock::get_or_init` for `symbol_adjacency` is
invoked OUTSIDE any other guard, and the `semantic_closure_cache` guard
ends before the miss-path BFS starts), so the caches cannot deadlock, and
no guard crosses a rayon job boundary."

Guards touched, one by one:
1. `cached_single_seed_closure` lookup guard — UNTOUCHED, still the
   single-statement temporary dropped at the end of the lookup statement.
2. `cached_single_seed_closure` miss-path insert — replaced by
   `admit_entry(...)`: one critical section on the cache's OWN mutex
   covering `len()` + `insert()` only. Between the two there is no lock
   acquisition, no BFS (the BFS completed before the call, guardless,
   exactly as before), no closure OnceLock touch, no rayon boundary. The
   check+insert must be one section or the cap could overshoot under
   concurrent misses ("`<=` cap at all times", I7c).
3. `cached_physical_expansion` lookup guard — UNTOUCHED.
4. `cached_physical_expansion` miss path — expansion (the only work that
   touches the `closures` OnceLock) still runs OUTSIDE every guard, before
   `admit_entry` (same ordering argument as item 2).
5. All `OnceLock`s (`closures`, `symbol_adjacency`, `file_adjacency`) —
   UNTOUCHED.
Deviation from the letter of "single-statement-shaped": the admit critical
section spans a length test + insert (3 statements, no I/O, no nesting).
Chosen deliberately: a check-then-insert across two separate guard
acquisitions would admit races past the cap; the journal records this as
the minimal relaxation that keeps the invariant, and the doc quoted above
stays verbatim TRUE under it.

## Commit record

One scoped commit (deviation D5: single commit instead of stub-then-green;
see DEVIATIONS):
`python tools/commit_scoped.py --scope crates/graph/src/load_caches.rs
--scope crates/graph/tests/load_memo_capacity_invariance_test.rs --scope
tasks/stage5-collateral-memory-ceiling-t1-journal.md -m "feat(graph):
bounded-capacity load memo + capacity-invariance oracle (M28 T1)"`
(load.rs scope DROPPED per deviation D2.)

- D5: single commit instead of the journal plan's two-step (stub slice then
  green): `commit_scoped` commits whole files, so the behavior-neutral
  stub could not be committed after the policy already filled the same
  file; the RED ordering proof is the capture pair above (identical test
  file content, policy absent -> failing; policy present -> passing),
  which is the evidence the review-gate Red-first check consumes.

## Files touched (final diff shape)

- `crates/graph/src/load_caches.rs` — 410 -> 500 lines (ceiling 1000);
  caps + `with_closures_and_capacity` + `with_memo_capacity` single
  construction path + `admit_entry` + honest doc updates ("entries are
  permanent" -> admitted-permanent past the cap; immutable-inputs
  staleness argument at :21-24 untouched).
- `crates/graph/tests/load_memo_capacity_invariance_test.rs` — NEW, 4
  tests (I5 matrix; I7c pin + caps; over-demand cap falsifier; cap-0
  ablation), data-driven fixtures, no per-fixture copy-paste bodies.
- `crates/graph/src/load.rs` — UNTOUCHED (D2).
- `tasks/stage5-collateral-memory-ceiling-t1-journal.md` — this file.

## Successor / T2 pointer

The T2 implementer cites THIS commit for cap wiring: boot-memo caps are
`LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP` / `LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP`
(hop-fixed literals, reviewer adjudicates — cite as hop-fixed, never as
review-ratified), consumed via `LoadMemo::with_closures_and_capacity` +
the existing `LoadAnalyzer::with_memo` seam (no analyzer constructor was
added; D2). Cap-0 mode = T0b's ablation lever. The I5 suite file is
forever-locked: re-run UNEDITED.

Context-wall resume: `python tools/checkpoint_guard.py resume --spec
tasks/stage5-collateral-memory-ceiling-t1-handoff.md --scope crates/graph
--cmd ...`
