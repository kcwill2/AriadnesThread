# M23.5 T2-B Implementer Journal -- equivalence-to-old safety net

**Unit:** T2.3 (`tasks/m235-t2b-handoff.md`), crate:graph tests only.
**Branch/worktree:** `tarjan-s-scc`, `.kilo/worktrees/tarjan-s-scc`, base HEAD `7c94af3`.
**Implementer session:** 2026-10-04.

## Commits (recorded here per Phase 1 verdict finding 8)

| Commit | Scope | Content |
|---|---|---|
| `08741b8` | `crates/graph/tests/cycle_groups_equivalence_test.rs`, `tasks/m235-t1-verdict.md`, `tasks/m235-execution-results.md`, `tasks/m235-t2a-verdict.md`, `tasks/m235-t2a-execution-results.md` | test(graph): freeze old detect_cycles oracle and prove group parity (gate 6 verbatim; numstat 833/132/326/139/487 added, 0 removed) |
| child of `08741b8` (its SHA is the next row in `python tools/checkpoint_guard.py status` RECENT COMMITS -- a commit cannot carry its own SHA) | `tasks/m235-t2b-journal.md` | docs(tasks): T2-B journal with mutation-Red evidence + SHA fill-in (this commit; verifiable one hop away in the commit graph, honestly not self-attested, same precedent T2-A verdict adjudicated ACCEPTED for its row 2) |

## Preflight step -- report.json state (F12 / verdict finding 9)

- BEFORE: `python tools/checkpoint_guard.py status` at session start showed
  DIRTY FILES `(none)` -- `tests/fixtures/basic/artifacts/includes/report.json`
  absent from the `git diff HEAD --numstat` dirty section, i.e. byte-equal to
  HEAD. No restoration needed and none performed (no Write touch).
- AFTER all test runs (gate 5): DIRTY FILES `(none)` again -- the path never
  entered the dirty section and appears in no commit. `git status --porcelain`
  ` M` stat-cache churn (finding 9) is expected and NOT a content dirt; the
  handoff's numstat-based criterion is the one satisfied.

## Test file

`crates/graph/tests/cycle_groups_equivalence_test.rs`, 833 lines (< 1000
ceiling; over the ~285 sizing estimate because the suite copies the full
16-shape fixture corpus verbatim -- handoff sec 3.1/3.3 mandate copy-not-
reinvent -- plus the locked-seed generator). Every function < 100 lines.
Four `#[test]` functions with the exact sec5 names. Zero R1/R2/R3 findings on
the file (gate 4 prints none for it).

## Frozen oracle -- authenticity and logged adaptations

`old_detect_cycles` / `old_dfs_cycle` are copied from
`crates/graph/src/graph.rs:234-287`. NO local inside either body was renamed
(`node`, `adj`, `visited`, `rec_stack`, `current_path`, `cycles`, `pos`, `n`
all verbatim), so no sec3.1 rename deviation exists. Forced test-locality
adaptations only, all mechanical:

1. receiver: `impl IncludeGraph` method -> free fn taking `graph: &IncludeGraph`
   (`self.` -> `graph.`);
2. `Self::dfs_cycle` -> `old_dfs_cycle` (private associated fn cannot be a
   method here; recursion shape identical);
3. `self.adjacency()` -> `old_adjacency(&graph)` -- a documented MIRROR of the
   pub(crate) `adjacency()` loop citing `graph.rs:83-95` (handoff sec3.2
   prescribes exactly this; `adjacency()` is pub(crate), invisible to an
   integration test).

IV-6 posture: the oracle is test-local, never edited to fit, and survives the
T3.1 deletion of the live `detect_cycles`. Production `cycle_groups.rs`
carries no `#[cfg(test)]` copy of the old algorithm (reviewer audit 6 -- none
added).

## Corpus

- All 16 shapes from `closure_equivalence_tests.rs:59-176`, copied verbatim.
- Three T2-B-added shapes (deviation D-C below): the two-INCOMPARABLE-groups
  tie-break pin, the T2-A diamond-with-chord shape, a self-loop singleton.
- The locked-seed corpus with the SAME seed constants as
  `closure_equivalence_tests.rs:405-416` (verdict lock: identical seed values);
  loop locals renamed following the sanctioned T2-A precedent (deviation D-D).
- The 200-node tangle (same seed/constants as `cycle_groups_test.rs:526-529`).
- The named divergence graph (test 4).
- Exact equality is scoped to registered-endpoint graphs (sec 4):
  `all_edge_endpoints_registered` gates the equality loop, and a floor assert
  (`exact_cases >= 20`; 23 of 25 non-divergence graphs pass the gate) proves
  the sweep is real. Test 3 asserts new-is-never-subset-of-old over EVERY
  graph including the two unregistered-endpoint closure shapes.
- Regime mix asserted in test 3: corpus contains at least one all-acyclic
  graph and at least one >= 2-member group.

## THE NAMED DIVERGENCE CASE (sec 4)

Named in code, test, and this journal:
`test_unregistered_endpoint_cycle_is_found_by_groups_but_not_by_old_paths`,
graph `named_divergence_graph()`: registered {1,2} with edge 1 -> 2;
NEVER-registered 401 <-> 402 cycle plus dangling 402 -> 403.

Why old and new legitimately differ there: `detect_cycles` seeds its DFS from
`self.file_paths` keys ONLY (`graph.rs:240-241`), so seeds 1 and 2 never reach
401/402 and the old path enumeration reports NOTHING -- yet 401/402 lie on a
real include cycle. That miss IS the IV-2 bug. The new D-A1 seed universe
(registered + every edge source + every resolved destination, sorted) reaches
them and emits group {401,402}. The assertion therefore claims only the safe
direction (old subset-of new, STRICT here) plus the concrete new result; no
equality claim, and the case is carried in the suite rather than filtered out
of the corpus.

## Carry-forward findings from tasks/m235-t2a-verdict.md -- how each is closed

1. **Union seed universe unlocked.** CLOSED by the named divergence case:
   401/402/403 are endpoints NOT in `file_paths` on a real cycle. Collapsing
   `tarjan_seed_universe` back to `file_paths`-only seeding leaves all nine
   T2-A tests and all equality assertions green (every other fixture
   registers its endpoints) but emits NO group on this graph, failing the
   test-4 assertion which states this lock in its message. Test 3 additionally
   sweeps the two unregistered-endpoint closure shapes in the safe direction.
2. **Witness Red never demonstrated (T2-A deviation C).** CLOSED by mutation
   experiment B below: the witness finder was mutated and the failure of the
   real witness-verifying tests was OBSERVED and recorded verbatim -- the
   milestone now contains a demonstrated Red for witness behavior.
3. **IV-3 asserted only as membership iff.** CLOSED by
   `assert_intra_group_edges_lie_on_group_cycles`: a PER-EDGE assertion over
   every resolved edge with both endpoints in one group of every registered
   corpus graph (all fixtures + seeded digraphs + tangle): the destination
   must reach its source back inside the group (within-group BFS reachability,
   `reaches_within_group`). An over-merging Tarjan defect puts two SCCs in one
   group, which leaves a member pair without a return path and trips this
   assertion directly.

## Mutation experiments (the Reds) -- verbatim evidence

Mutations were made in `crates/graph/src/cycle_groups.rs`, run, and reverted
via the Kilo Edit tool. Nothing mutated was ever committed.

### Red A (handoff sec5 Step 2, mandated): reverse the IV-5 tie-break

Mutation: appended `ordered.reverse();` to `order_reverse_topological`.
Gate 1 verbatim digest:

```
### cargo test FAIL (exit 1) [capture: ...\target\testrun-capture.txt, 33 lines]
3 passed / 1 failed / 0 ignored / 1 suites
---- test_group_membership_equals_old_cycle_node_set_on_registered_corpus ----
thread 'test_group_membership_equals_old_cycle_node_set_on_registered_corpus' (4292) panicked at crates\graph\tests\cycle_groups_equivalence_test.rs:702:5:
assertion `left == right` failed: IV-5 tie-break: the two INCOMPARABLE groups {1,2} and {3,4} must emit smallest-member-first -- expected orderings [[1,2],[3,4]]
  left: [[3, 4], [1, 2]]
3 passed / 1 failed / 0 ignored / 1 suites
```

Exactly the predicted failure shape: test 1, the two INCOMPARABLE groups, both
orderings named. Capture file transient (overwritten by the next run), path as
printed by the digest.

### Red B (verdict finding 2): break the closed-walk witness

Mutation: dropped `walk.push(neighbor)` (the closing step) in
`closed_walk_witness`. Gate 2 verbatim digest (failing line trimmed to the
three failure headers; full text in the capture at the printed path):

```
### cargo test FAIL (exit 1) [capture: ...\target\testrun-capture.txt, 171 lines]
111 passed / 3 failed / 0 ignored / 3 suites
---- test_two_cycle_yields_one_group_with_closed_witness ----
... assertion `left == right` failed: witness is a closed walk of length 3 closing [a, b, a] (IV-4)  left: [FileId(1), FileId(2)]
---- test_seeded_digraphs_witnesses_are_edge_backed_closed_walks ----
... group big-cycle witness [FileId(2), ..., FileId(9)] must be a closed walk with first == last (IV-4)
---- test_known_two_cycle_survives_in_random_tangle ----
... group two-cycle-in-200-node-tangle witness [FileId(101), FileId(102)] must be a closed walk with first == last (IV-4)
111 passed / 3 failed / 0 ignored / 3 suites
```

Deviation note D-F: the failing digest shows `3 suites`/114 total because
cargo fail-fast skips the remaining suites once one binary fails (lib 101 +
cycle_groups 9 + equivalence 4 = 114); clean runs show the full 5 suites/116.

### Revert proof (byte-for-byte)

After BOTH reverts, `python tools/checkpoint_guard.py status` reported
`== DIRTY FILES (git diff HEAD --numstat: worktree + index vs HEAD) ==
(none)` -- numstat is content-based, so `crates/graph/src/cycle_groups.rs`
absent from that section proves the file is byte-equal to HEAD `7c94af3`;
`detect_cycles`/`dfs_cycle` in `graph.rs` likewise never touched (dirty
section stayed `(none)` for the whole session; `graph.rs` byte-unchanged).
Post-revert gates 1 and 2 reproduce the pinned green lines exactly.

## Step 4 outcome -- gap fixes

EXPLICIT: NO gap was found. No test failed against unmutated production code;
`crates/graph/src/cycle_groups.rs` was edited ONLY inside the two mutation
experiments and is byte-unchanged at commit time (dirty section `(none)`
above). The commit scope carries no production diff.

## Gate ledger (verbatim results, direct form)

| # | Gate | Result |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_equivalence_test` | pre-mutation green `4 passed / 0 failed / 0 ignored / 1 suites`; under Red A `3 passed / 1 failed / 0 ignored / 1 suites`; post-revert `4 passed / 0 failed / 0 ignored / 1 suites`. Capture `target/testrun-capture.txt` (transient; path printed by each digest) |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph` | under Red B `111 passed / 3 failed / 0 ignored / 3 suites`; post-revert `116 passed / 0 failed / 0 ignored / 5 suites` -- matches the pinned line; re-proves T2-A's nine tests + closure_equivalence (IV-10) in one run |
| 3 | `python tools/distill.py -- cargo build -p ariadne-graph` | `ok (exit 0, 0 warnings emitted)` -> capture `target/distill-capture.txt` |
| 4 | `python tools/hygiene_check.py` | zero R1/R2; every printed line is a legacy-file R3 warn; ZERO R3 findings in `cycle_groups_equivalence_test.rs` despite the verbatim-copied body (deviation D-B: the copied `pos`/`n` spellings were kept per the verbatim contract; the scanner did not flag them) |
| 5 | `python tools/checkpoint_guard.py status` | run at session start, between mutation/revert pairs, and after all test runs: DIRTY FILES `(none)` every time; fixture path never dirty; after both reverts = byte-for-byte revert proof |
| 6 | `python tools/commit_scoped.py --scope ... -m ...` | two scoped commits (table at top); explicit scopes only, never `git add .`; fixture path in NO commit |
| - | `python tools/encoding_guard.py --staged` after staging | clean, no output (see deviation D-E on the staging gate's mechanics) |

## Locks touched / untouched

- `graph.rs` byte-unchanged (dirty `(none)` throughout); `detect_cycles` at
  `:234`, `dfs_cycle` at `:260-287` intact (T3.1 owns deletion).
- `crates/graph/src/cycle_groups.rs` byte-unchanged vs `7c94af3` (mutations
  reverted; no gap needed a fix).
- `crates/graph/tests/cycle_groups_test.rs`, `closure_equivalence_tests.rs`,
  `crates/graph/Cargo.toml`, `tests/fixtures/**`, `tests/golden/**`: untouched.
- No new cross-crate dependency; no new hex literals (no UI file touched).
- Seeded corpus identical to the closure suite's locked seeds.

## Deviations (each with reason)

- **D-A (scope, orchestrator-declared):** the first commit also carries the
  four orchestrator-declared already-approved record files
  (`tasks/m235-t1-verdict.md`, `tasks/m235-execution-results.md`,
  `tasks/m235-t2a-verdict.md`, `tasks/m235-t2a-execution-results.md`) beyond
  the handoff sec2 touch-set, per the orchestrator's T2-B tasking message --
  the planner/reviewers cannot commit and the records must become durable.
  Reviewer's commit-vs-touch-set check should adjudicate against THAT
  declaration; nothing else outside sec2 is in any commit.
- **D-B (verbatim oracle):** zero local renames; only the three forced
  test-locality adaptations listed above. Chosen over T2-A-style renames
  because handoff sec3.1 states "verbatim is the contract"; hygiene accepted
  it (gate 4 zero R3 on the file).
- **D-C (corpus additions):** three shapes added (incomparable pin,
  diamond-with-chord, self-loop). The incomparable pin is REQUIRED by sec5
  Step 2 (the mutation must have a bite case); the other two give IV-3
  per-edge and singleton-self-loop coverage the closure corpus lacks. Additive
  only; none of the 16 closure shapes changed.
- **D-D (copied-generator renames):** `random_graph`/`cyclic_chain` loop
  locals renamed to the T2-A-sanctioned spellings; LCG constants, seeds, and
  control flow byte-identical, so the seeded corpus is unchanged.
- **D-E (staging-gate mechanics):** `commit_scoped.py` stages and commits
  atomically (no stage-only mode), so `encoding_guard.py --staged` cannot
  interpose between them. `encoding_guard.py --all` was therefore run as the
  pre-commit content gate over the whole tracked tree, and `--staged` was run
  at the literal gate position (after staging/commit; empty staged set, clean).
  Same mechanics as T2-A gate 5, which the reviewer accepted.
- **D-F (failing-run suite count):** documented above under Red B -- cargo
  fail-fast, not a suite disappearing.
- **Refusal record:** one attempted `python tools/distill.py -- python -c
  ...` (file line-count helper) was refused with the catch-all boilerplate
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`;
  the line count was obtained via the Read tool instead. No gate was dropped
  for a refusal.
