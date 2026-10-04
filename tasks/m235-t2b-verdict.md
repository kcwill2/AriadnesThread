# M23.5 T2-B Verdict - Equivalence-to-Old Safety Net (crate:graph, test-only)

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Handoff:** `tasks/m235-t2b-handoff.md` (T2.3)
**Range reviewed:** `7c94af3..b950963` (base `7c94af3`, HEAD `b950963`, branch
`tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`)
**Commits:** `08741b8` (suite + four record files), `b950963` (journal)
**Execution-results addendum:** `tasks/m235-t2b-execution-results.md`

## VERDICT: APPROVED

0 blocking findings. 10 non-blocking findings: 2 reviewer/process limits, 2
planner-or-handoff defects, 3 implementer observations accepted as-is, 1 ledger
bookkeeping gap, 2 standing configuration items.

Scope discipline held: Phase 3 work was neither demanded nor credited.

---

## 1. Ground truth

`python tools/checkpoint_guard.py status` at session start: HEAD `b950963`,
branch `tarjan-s-scc`, DIRTY FILES `(none)`, UNTRACKED FILES `(none)`.

Re-run AFTER my own full workspace test run: HEAD `b950963`, DIRTY FILES
`(none)`, UNTRACKED FILES `(none)`. `git diff --numstat HEAD --
tests/fixtures/basic/artifacts/includes/report.json` returns no rows (only the
autocrlf warning), i.e. the path is byte-equal to HEAD before and after.
`git log --oneline 7c94af3..HEAD -- <that path>` returns nothing. The
implementer's claim that no restoration was needed is CONFIRMED, not assumed.

## 2. Commit-versus-touch-set check (PASS)

Authorized touch-set: `crates/graph/tests/cycle_groups_equivalence_test.rs`,
`tasks/m235-t2b-journal.md`, plus the four orchestrator-declared record files
`tasks/m235-t1-verdict.md`, `tasks/m235-execution-results.md`,
`tasks/m235-t2a-verdict.md`, `tasks/m235-t2a-execution-results.md`.

`git diff --name-only 7c94af3..b950963` returned exactly six paths, all in set:

```
crates/graph/tests/cycle_groups_equivalence_test.rs
tasks/m235-execution-results.md
tasks/m235-t1-verdict.md
tasks/m235-t2a-execution-results.md
tasks/m235-t2a-verdict.md
tasks/m235-t2b-journal.md
```

`tests/fixtures/basic/artifacts/includes/report.json` appears in NEITHER
commit (per-commit `--stat` and the path-scoped `git log` above).
`crates/graph/src/graph.rs`, `crates/graph/src/cycle_groups.rs`,
`crates/graph/src/lib.rs`, `crates/graph/src/closure_equivalence_tests.rs`,
`crates/graph/tests/cycle_groups_test.rs`, and `crates/graph/Cargo.toml` all
return an EMPTY `git diff 7c94af3..HEAD -- <path>`. The test-only property is
structural, not asserted.

## 3. Fresh gate results (reviewer-run, not journal-relayed)

| Handoff gate | Command | Reviewer-measured result |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_equivalence_test` | `4 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph` | `116 passed / 0 failed / 0 ignored / 5 suites` |
| 3 | `python tools/distill.py -- cargo build -p ariadne-graph` | `ok (exit 0, 0 warnings emitted)` |
| 4 | `python tools/hygiene_check.py` | zero R1/R2; zero R3 in the new file |
| 5 | `python tools/checkpoint_guard.py status` | fixture absent from the dirty section, pre- and post-run |
| 6 | `python tools/encoding_guard.py --staged` | clean, no output |
| 7 | `python tools/encoding_guard.py --all` | clean, no output |
| 8 | commit | both commits in set (section 2) |

Both pinned verdict lines reproduce EXACTLY. Neither is vacuous. Gate 2's
capture (`target/testrun-capture.txt`, 147 lines) shows the target order
lib (101) -> `cycle_groups_equivalence_test` (4) -> `cycle_groups_test` (9,
17.25 s) -> `m22_marker_unit_shaper_test` (2) -> doc-tests (0), and names all
four new `#[test]` functions passing. The 17.25 s is the 100 000-node chain
doing real work at 64 KiB, so the run is not silently filtering.

IV-10 is proven directly: `closure_equivalence_tests.rs:515` still calls
`graph.detect_cycles()`, that file is byte-unchanged in the range, and the
capture names `graph::closure_equivalence::*` passing inside the same run that
builds the new module.

## 4. The frozen oracle - authenticity (IV-6)

`git diff 2708425 HEAD -- crates/graph/src/graph.rs` is EMPTY, so the current
`graph.rs` is byte-identical to the pre-migration blob the handoff pins; reading
the checkout is byte-faithful and I did so (no `git show` piped through
PowerShell). Region `:234-287` compared field-by-field against the test copy at
`cycle_groups_equivalence_test.rs:90-143`:

- entry head `let mut cycles = Vec::new()` + `visited`/`rec_stack`/
  `current_path` init: identical (235-238 -> 91-94)
- seed `file_paths.keys().copied().collect()` then `.sort()`: identical
  (240-241 -> 96-97)
- outer loop `for &node in &all_files { if !visited.contains(&node) }`: identical
  (244-255 -> 100-111)
- `dfs_cycle` body: `visited.insert` / `rec_stack.insert` / `current_path.push`,
  the neighbour loop, the `rec_stack.contains` back-edge test, the
  `current_path.iter().position(|&n| n == neighbor)` extraction,
  `current_path[pos..].to_vec()`, and the `rec_stack.remove` /
  `current_path.pop` epilogue: identical line-for-line (268-286 -> 124-142)
- NO local renamed. `node`, `adj`, `visited`, `rec_stack`, `current_path`,
  `cycles`, `pos`, `n` are all verbatim. D-B's "zero renames" claim is TRUE.

The three declared forced test-locality adaptations were each audited for
semantic drift and each is inert:

1. receiver `self.` -> `graph.`: same object, no semantic content.
2. `Self::dfs_cycle` -> `old_dfs_cycle`: the original body (268-286) never
   touches `self`, so associated-fn -> free-fn is a pure signature change.
3. `self.adjacency()` -> `old_adjacency(&graph)`: `old_adjacency` (76-88) is a
   line-for-line mirror of `graph.rs:83-95` (same `if let Some(dest)` skip, same
   `entry().or_default().push(dest)`, same `sort()` + `dedup()`, same return),
   with the citing comment handoff sec3.2 required. Proven equivalent because
   `adjacency()` is `pub(crate)` and an integration test cannot call it.

IV-6 posture holds. The oracle is test-local, calls nothing on the live method,
and therefore survives T3.1's deletion of `detect_cycles` - the suite cannot be
tautological and cannot be broken by the swap.

## 5. Mutation experiments - independent verdict

**I could not re-execute either mutation.** My write grant is confined to
`tasks/**` and `.kilo/plans/**`; `crates/**` is read-only for this reviewer and
the role forbids trying (finding 1). What follows is therefore source-level
causal proof plus falsification tests, not execution. Each reported failure is
nonetheless pinned down to the exact assertion, and each failure SET is
arithmetically complete.

### Red A - reverse the IV-5 tie-break (`ordered.reverse()` in `order_reverse_topological`)

VERDICT: REAL. Consequence of the mutation, proven, not inferred.

- The mutation site exists as described: `order_reverse_topological`
  (`cycle_groups.rs:309-337`) pushes `Reverse((groups[index][0], index))` onto a
  `BinaryHeap`, pops into `ordered`, and returns `ordered` (336). Appending
  `ordered.reverse();` before that return is a legal one-line mutation.
- The pinned fixture is two INCOMPARABLE sinks with no condensation edges, so
  `ordered` is exactly the heap pop sequence keyed on smallest member: `{1,2}`
  (key 1) then `{3,4}` (key 3). Reversed gives `[[3,4],[1,2]]` - the journal's
  quoted `left` value, character for character, including the `Vec<Vec<u64>>`
  Debug spacing. Deterministic; no HashMap iteration order can change it.
- The quoted panic location is `cycle_groups_equivalence_test.rs:702:5`. In the
  committed file, line 702 is the `assert_eq!(` token at 4-space indent = column
  5. A fabricated Red would have to guess that exact line AND column after the
  file was written; the reported location matches the committed revision.
- The quoted message text matches lines 705-706 verbatim, including the ASCII
  `--` and the doubled braces.
- The failure SET is complete: the new suite contains NO `representative_cycle`
  reference anywhere (grep: zero matches), and tests 2, 3 and 4 are set-based
  (containment, new-minus-old, one-group-with-`[401,402]`), all invariant under a
  global order reversal. Hence exactly `3 passed / 1 failed / 0 ignored /
  1 suites`. No `#[ignore]` exists anywhere in `crates/graph`.

### Red B - drop `walk.push(neighbor)` in `closed_walk_witness`

VERDICT: REAL, and the count of exactly 3 is arithmetically forced.

- Causal chain: `closed_walk_witness` (`:474-485`) finds a back-edge to a
  vertex on the current path, computes `walk = path[position..].to_vec()`, then
  `walk.push(neighbor)`, then returns. `walk.push` at `:481` is the ONLY closure
  step. For the two-cycle fixture the walk is `[1,2]` and the push supplies the
  closing `1` -> `[1,2,1]`, which is precisely what
  `cycle_groups_test.rs:210-213` asserts. Dropping the push yields
  `[FileId(1), FileId(2)]` - the journal's quoted `left`, exactly.
- The other two quoted messages exist verbatim in `cycle_groups_test.rs:476`
  ("group {label} witness {:?} must be a closed walk with first == last (IV-4)")
  and the labels (`big-cycle`, the 200-node tangle's `[101,102]` group) match
  `RANDOM_FIXTURES` and `KNOWN_CYCLE_FIRST/SECOND`.
- Failure SET completeness, checked exhaustively against every
  `representative_cycle` reference in `cycle_groups_test.rs`:
  `test_two_cycle_...` (:211) fails; `assert_witness_is_edge_backed` called at
  :521 (seeded) and :554 (tangle) fail; `test_self_include_yields_single_member_group`
  (:227, expects `[1,1]`) is IMMUNE because `representative_cycle_for`
  (`:420-421`) short-circuits singletons through `self_loop_witness`, never
  reaching `closed_walk_witness`; `test_deep_include_chain_...` asserts no
  witness; the remaining three assert only membership, order, or degrees.
  Exactly three failures. 101 lib + 4 equivalence + 6 of 9 = 111 passed, and
  cargo aborts before `m22_marker_unit_shaper_test`, giving 3 suites. The
  journal's `111 passed / 3 failed / 0 ignored / 3 suites` is exact.
- Neither failure is pre-existing: gates 1 and 2 are green at this HEAD with
  the mutations reverted.

### Revert proof

`git diff 7c94af3 HEAD -- crates/graph/src/cycle_groups.rs` is EMPTY.
`cycle_groups.rs` is byte-equal to its state at the T2-A head. Nothing mutated
was committed; the commit scope carries no production diff at all.

## 6. The three carry-forward findings - genuinely discharged?

**Finding 1 (union seed universe unlocked): GENUINELY DISCHARGED.** I verified
the mechanism, not the name. `named_divergence_graph()` registers only {1,2} and
adds edges 1->2, 401->402, 402->401, 402->403. Test 4 first asserts
`!all_edge_endpoints_registered(&graph)`, so the case cannot silently become a
registered-endpoint graph. Under a `file_paths`-only seed the universe is {1,2};
1->2 is acyclic; 401/402/403 are unreachable from either seed, so zero groups
are emitted and BOTH `groups.len() == 1` and
`groups[0].member_files == vec![401,402]` fail. Uniqueness of the lock is also
established: I enumerated all 25 graphs in the equality sweep. The 6 seeded
graphs register `1..=file_count` and link only inside that range; the two
characterization graphs excluded by the registered-endpoint gate ("unregistered
edge-only ids" 22->20->21 and "unresolved-only source" 70->None, 71->72) are
both acyclic with empty old and new sets, so neither would fail. No other corpus
graph can catch a seeding collapse.

**Finding 2 (witness Red never demonstrated): GENUINELY DISCHARGED.** Red B is
an executed, causally-verified Red for witness behaviour. The milestone now
holds a demonstrated Red for IV-4 closure, not just for the tie-break.

**Finding 3 (IV-3 asserted only as membership iff): GENUINELY DISCHARGED, with
a stated residual (finding 4).** `assert_intra_group_edges_lie_on_group_cycles`
(`:606-631`) walks EVERY resolved edge of EVERY group of every equality-sweep
graph (11 core fixtures + 5 characterization + 3 T2-B pins + 5 seeded
topologies + the 200-node tangle) and requires within-group BFS reachability
from the destination back to the source. An over-merge that fuses two SCCs
joined by a one-way edge leaves that edge intra-group with no return path, and
the assertion fires directly - a defect that group-membership proxies cannot
see. The bridged topologies in the corpus (TwoSccsBridge, full-blob, the 200-node
tangle) all supply such a crossing edge.

## 7. The named divergence case - exactly one, and not a filter

The equality gate `all_edge_endpoints_registered` excludes 3 of the 26 corpus
graphs: the two characterization shapes above and the named divergence graph.
Only ONE of the three is an actual old-vs-new divergence, and it is the one that
is named, justified in code, in the module doc (`:10-18`), in the graph
constructor's doc (`:460-468`), in the test name, and in the journal. Both
excluded characterization graphs are acyclic with empty old and new sets, so
nothing is being concealed. The divergence graph is additionally carried INSIDE
the corpus for the two one-directional assertions: `full_corpus()` (`:502-507`)
pushes it and both test 2 and test 3 iterate `full_corpus()`. It is therefore
named, not filtered. The anti-vacuity floor is real: `exact_cases >= 20` with
23 of 25 equality-sweep graphs passing the gate, and the regime-mix assertion
requires both a zero-cyclic graph and a >= 2-member group.

## 8. Corpus fidelity

- The 16 existing fixture shapes (11 core + 5 characterization) are reproduced
  label-for-label and edge-for-edge against `closure_equivalence_tests.rs:59-176`.
  I compared both regions in full.
- `InlineLcg` constants (`6364136223846793005`, `1442695040888963407`),
  `below`, `percent`, and all five `RANDOM_FIXTURES` seeds / file counts /
  topologies / names are byte-identical to `:373-416`. The seeded corpus lock
  HOLDS.
- `random_graph` control flow is identical modulo the declared D-D loop-local
  renames. `cyclic_chain` is identical modulo the same rename. The tangle
  constants and edge budget match `cycle_groups_test.rs:526-544` exactly, so the
  tangle is the same graph.
- Three T2-B shapes added (incomparable pin, diamond-with-chord, self-loop).
  Purely additive; none of the 16 changed. The incomparable pin is REQUIRED by
  sec5 Step 2 as the mutation bite case, and it independently pins group COUNT
  (exactly two groups) on the corpus's only bridgeless SCC pair.

## 9. Hygiene, locks, no dual representation

`python tools/hygiene_check.py` reports zero R1/R2 workspace-wide; every printed
line is a legacy-file R3 warn. Zero findings in
`crates/graph/tests/cycle_groups_equivalence_test.rs` - and the scanner does scan
test files (it flags `crates/cluster/tests/scc_iterative_parity.rs:380`), so the
zero is measured, not a skip. The file is 833 lines, under the 1000 ceiling;
its longest function is `core_fixture_shapes` at 82 lines, under the 100-line
rule, confirmed twice (R1/R2 clean plus direct read).

`python tools/encoding_guard.py --all` and `--staged` both produced no output.

Locks: `graph.rs` byte-unchanged since the pre-migration blob `2708425`;
`cycle_groups.rs` byte-unchanged vs `7c94af3`; `lib.rs`,
`closure_equivalence_tests.rs`, `cycle_groups_test.rs`, and `Cargo.toml`
byte-unchanged; `tests/fixtures/**` and `tests/golden/**` untouched; zero new
hex literals (no UI file in the range).

No dual representation: grep for `dfs_cycle|rec_stack|current_path|detect_cycles`
across `crates/` finds the old algorithm in exactly two places - the production
`graph.rs:234-287` (T3.1 owns its deletion) and the test-local oracle.
`cycle_groups.rs` carries NO `#[cfg(test)]` module and no copy of the old
algorithm; its only mention of `detect_cycles` is a doc reference at `:64`.

## 10. Findings

No blocking findings.

1. **NON-BLOCKING, reviewer/process.** I could not re-execute either mutation
   experiment. My write grant is confined to `tasks/**` and `.kilo/plans/**`
   (`crates/**` is read-only for the reviewer role), and mutating production to
   reproduce a Red would violate both. The verdict in section 5 therefore rests
   on exact panic-location/column matching, verbatim message matching, exact
   `left`-value derivation from the mutation, proof that both mutation sites
   exist as described, and an exhaustive proof that each reported failure SET is
   arithmetically complete and non-pre-existing. Same class as T2-A finding 10.
   If the milestone wants executed mutation evidence at review time, the
   orchestrator must grant the reviewer a scratch mutation capability or run the
   two mutations itself.
2. **NON-BLOCKING, handoff-spec / planner.** The sizing estimate is off by
   roughly 2.9x. Handoff sec10 predicted ~285 lines for the suite and ~310
   produced lines for the unit; the actual figures are 833 and 1066 (833 suite +
   0 production + 233 journal). The unit still fits the ~1200 sizing law, but at
   89% of it, so the headroom sec10 claimed does not exist. The cause is
   sec3.3's own copy-not-reinvent mandate over the 16-shape corpus, the LCG, the
   generator, the tangle and the divergence graph. Next unit's sizing estimates
   should be re-derived from this measurement.
3. **NON-BLOCKING, implementer (accepted shape, flagged for the ledger).** The
   fixture corpus is now triplicated: `InlineLcg`, `Topology`, `random_graph`,
   `cyclic_chain` and the 16 shapes exist in `closure_equivalence_tests.rs`,
   `cycle_groups_test.rs` and `cycle_groups_equivalence_test.rs`, and the
   adjacency mirror now exists in three places (`graph.rs:83`,
   `test_adjacency`, `old_adjacency`). The locked-seed contract is now enforced
   by prose in three files instead of one, and 833 lines is 83% of the 1000-line
   ceiling that the next addition to this suite will press against. Consider a
   shared test-support module before any fourth copy.
4. **NON-BLOCKING, reviewer assessment (residual on carry-forward 3).**
   `assert_intra_group_edges_lie_on_group_cycles` cannot catch an over-merge of
   two SCCs that have NO directed edge in either direction: intra-SCC edges still
   have return paths, and IV-2 membership is unaffected. On this corpus that
   residual is covered for the one bridgeless pair by the group-count/order lock
   at `:692-707`, and every bridged topology supplies a crossing edge. Recorded
   so the coverage claim is not overstated.
5. **NON-BLOCKING, implementer (record).** Journal row 2 does not self-attest
   its own SHA and instead points at "the next row in `checkpoint_guard.py
   status`". Honest, and T2-A's row 2 set the precedent this reviewer accepted,
   but it is a recurrence of Phase 1 verdict finding 8's class. Recoverable in
   one hop: the journal commit is `b950963`.
6. **NON-BLOCKING, implementer (undeclared but inert).** Two transcription
   details fall under D-B/D-D without being enumerated: the copied `edge()` uses
   `context_name: "M23.5-T2B"` where the closure suite uses `"D1"`, and
   `random_graph`'s `let from = ...; let to = ...;` statements are inlined into
   the call in the BigCycle and SelfLoops arms. Both are provably inert:
   `add_edge` (`graph.rs:42-46`) dedups on whole-`IncludeEdge` equality, so one
   uniform context string per graph preserves dedup behaviour exactly, and
   `adjacency()` reduces edges to a per-source destination set regardless; Rust
   guarantees left-to-right call-argument evaluation, so the LCG consumption
   order and therefore the seeded corpus are unchanged. Not drift - recorded for
   the next reader.
7. **NON-BLOCKING, planner / milestone lifecycle.** The M23.5 milestone row
   (`tasks/milestone_ledger.md:26`) is stale: it still reads "Phase 2 ... not
   started", cites `HEAD e2a05e2`, and its canonical-baseline note still reads
   `1156 / 0 / 4 / 86` with no entry for T2-A's measured `1165 / 0 / 4 / 87`.
   Neither T2-A nor T2-B synced it. Must be corrected before T4.1 re-pins the
   baseline, otherwise the F11 unattributed-residual discipline is applied
   against a two-generations-old number.
8. **NON-BLOCKING, implementer (scope note).** The new suite exercises only
   `CycleGroup::member_files`; `representative_cycle`, `external_in_count` and
   `external_out_count` are never referenced (grep: zero matches). Handoff sec3
   pins all four fields. T2.3's parity claim is about membership and T2-A's suite
   covers the other three, so this is scope-appropriate - recorded so a future
   reader does not mistake this file for the full `CycleGroup` contract.
9. **NON-BLOCKING, configuration (standing).** F10 reconfirmed: `ui_smoke.py`
   and `bench_compare.py` both refused, verbatim, identical catch-all
   boilerplate; neither was routed through `distill.py`. There is still no
   ui-smoke and no perf evidence for `detect_cycle_groups`. T2-B is test-only
   and adds no production, so it cannot have moved perf - but T3.2 must re-pin
   `payload_tail_bench.rs` against the 4.7 ms baseline without reviewer-run perf
   evidence.
10. **NON-BLOCKING, configuration (F12 / T2-A finding 9, informational).** The
    post-run fixture check reproduces T2-A's mechanism verbatim:
    `git diff --numstat HEAD -- <path>` emits `warning: in the working copy of
    'tests/fixtures/basic/artifacts/includes/report.json': LF will be replaced
    by CRLF the next time Git touches it` and returns no rows. Content is
    byte-equal to HEAD before AND after my full workspace run and the path is in
    neither commit, so no restoration was needed. F12's remaining action (pin
    the path in `.gitattributes`, or make the artifact writer emit LF) is still
    open before T4.1.

## 11. Deviations adjudicated

| # | Deviation (journal label) | Disposition |
|---|---|---|
| D-A | four orchestrator-declared record files committed beyond the sec2 touch-set | **ACCEPTED.** Adjudicated against the orchestrator's explicit declaration, which is the touch-set I was instructed to enforce. All four landed unaltered: line counts match T2-A's own record exactly (326 and 132 for the Phase 1 pair, 487 and 139 for T2-A's own pair). Nothing else outside sec2 appears in any commit. |
| D-B | zero local renames in the oracle; three forced test-locality adaptations only | **ACCEPTED.** Section 4 audits each adaptation and finds no semantic drift; the rename claim is TRUE (all eight locals verbatim). Gate 4 measured zero R3 on the file despite the verbatim contract. |
| D-C | three corpus shapes added | **ACCEPTED.** The incomparable pin is required by sec5 Step 2 as the mutation bite case and additionally pins group count on the corpus's only bridgeless SCC pair; the diamond-with-chord supplies IV-3 per-edge reachability on a chord the closure corpus lacks; the self-loop supplies a singleton case. Purely additive - I verified all 16 closure shapes are unaltered. |
| D-D | copied-generator loop locals renamed, seeds and control flow unchanged | **ACCEPTED.** Seeds, LCG constants, `below`/`percent`, `cyclic_chain` body and both topologies' control flow verified byte-identical. The rename follows the T2-A precedent and is what keeps R3 clean. See finding 6 for two inert details that fall under this label without being enumerated. |
| D-E | `encoding_guard.py --staged` cannot interpose because `commit_scoped.py` has no stage-only mode | **ACCEPTED.** Mechanics are real. I ran BOTH forms fresh: `--all` produced no output over the whole tracked tree, and `--staged` produced no output. `--all` is the stronger gate and it is clean, so the substitution costs nothing. Same precedent as T2-A gate 5. |
| D-F | failing-run digest shows `3 suites` / 114 because cargo fail-fast | **ACCEPTED, and independently re-derived.** Gate 2's capture gives the target order (lib, equivalence, cycle_groups, m22_marker, doc-tests). With `cycle_groups_test` failing, cargo aborts before `m22_marker`: 101 + 4 + 6 = 111 passed, 3 failed, 3 suites - the journal's line exactly. No suite silently vanished and no test was filtered; `0 filtered out` appears in every `test result:` line of the green capture. |

## 12. Gates I could not run, recorded

- `python tools/testrun.py --baseline` -> `usage error: missing value for
  --baseline` (exit 2). Ledger M23.5 F9, reproduced ONCE and recorded once. Not
  treated as a gate failure and not worked around; the full-workspace
  invocation it wraps was run and compared by hand (section 4 of the
  execution-results addendum).
- `python tools/ui_smoke.py` -> refused. Verbatim: `The user has specified a
  rule which prevents you from using this specific tool call. Here are some of
  the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
- `python tools/bench_compare.py` -> refused. Verbatim: identical text.
  Both are F10; each was attempted independently, once, and neither was routed
  through `distill.py`.
- Also refused with the same catch-all boilerplate, each attempted separately
  and each then replaced by a form that ran: `;`-chained git invocations,
  `git ls-files --eol <path>`, and `git status --porcelain -- <path>`. This is a
  third instance of the recorded allowlist trap (T2-A finding 9). The
  equivalent evidence was obtained via `git diff --numstat HEAD -- <path>` and
  `git log --oneline <base>..HEAD -- <path>`, both of which ran.
- The two mutation re-executions (finding 1). Recorded as a capability limit of
  the reviewer role, not as a gate the implementer failed to run.

## 13. Phase 2 closure judgement

**Phase 2 is closed in my judgement.** T2-A and T2-B are both APPROVED.

- IV-1 asserted directly (`assert_partitions_disjoint`, plus Tarjan's
  single-pop construction) and re-asserted per graph in the T2-B sweep.
- IV-2 both directions asserted with exact set equality over 23 registered-
  endpoint graphs, plus the named, justified, unfiltered divergence case for the
  unregistered-endpoint universe.
- IV-3 asserted per edge over intra-group reachability across the whole corpus
  (residual in finding 4), with the diamond-with-chord chord covered.
- IV-4 witnessed: two executed mutation Reds, and the witness is verified on the
  closure-equivalence corpus and the tangle.
- IV-5 tie-break pinned on two INCOMPARABLE groups and mutation-proven.
- IV-6 oracle is a byte-faithful frozen copy of the pre-migration blob, never
  edited, test-local, and it survives the swap.
- IV-10 closure equivalence green with `closure_equivalence_tests.rs`
  byte-unchanged.

No T2-A or T2-B non-blocking finding becomes blocking once Phase 3 deletes
`detect_cycles`. The structural reason is that the oracle is a COPY that calls
nothing on the live method, so T3.1's deletion cannot break it; and T2-B's IV-2
equality is exactly the licence T3.1 needs for the
`closure_equivalence_tests.rs:515` repoint, since
`old_detect_cycles(g).is_empty()` and `detect_cycle_groups(g).is_empty()` are
equal on every registered-endpoint graph in the corpus.

Two items must still land before T4.1, neither of them gating Phase 2:
bookkeeping (finding 7, stale milestone row) and the standing configuration
gaps (finding 9, no ui-smoke or perf evidence for `detect_cycle_groups`; and
F12's unpinned fixture path).
