# M23.5 T2-A Verdict - CycleGroup core + Iterative Graph Tarjan (crate:graph)

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Handoff:** `tasks/m235-t2a-handoff.md` (T2.1 + T2.2)
**Range reviewed:** `e2a05e2..7c94af3` (base `e2a05e2`, HEAD `7c94af3`, branch
`tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`)
**Commits:** `34f5fe4` (production + tests + journal), `7c94af3` (planning
artifacts + journal SHA fill-in)
**Execution-results addendum:** `tasks/m235-t2a-execution-results.md`

## VERDICT: APPROVED

0 blocking findings. 10 non-blocking findings: 3 carry-forward test-coverage
gaps that belong to the next unit, 3 handoff/plan artifact defects, 2
implementer-record defects, 1 configuration finding that narrows this
acceptance pass, and 1 measured pre-existing-worktree hazard.

Scope discipline held: T2-B (the equivalence-to-old oracle suite) and Phase 3
work were neither demanded nor credited.

---

## 1. Ground truth

`python tools/checkpoint_guard.py status` at session start: HEAD `7c94af3`,
branch `tarjan-s-scc`, DIRTY FILES `(none)`, untracked
`tasks/m235-execution-results.md` + `tasks/m235-t1-verdict.md` only.

The handoff's preflight step 0 requirement is MET and still holds:

- the fixture path is absent from the `git diff HEAD --numstat` dirty section
  (both at session start and after this review's full workspace test run);
- `git ls-files --eol tests/fixtures/basic/artifacts/includes/report.json` ->
  `i/lf  w/lf  attr/` - index LF, working copy LF, no `.gitattributes` pin;
- no `\r` byte is present anywhere in the file;
- `git diff --numstat HEAD -- <path>` returns no rows, i.e. content is
  identical to the HEAD blob.

`git status --porcelain` does report ` M` on that path, but only after a test
run has rewritten identical bytes and invalidated the stat cache - see
finding 9. The handoff's success criterion is numstat-based and is satisfied.

## 2. Commit-versus-touch-set check (PASS)

Authorized touch-set: `crates/graph/src/cycle_groups.rs`,
`crates/graph/src/lib.rs`, `crates/graph/tests/cycle_groups_test.rs`,
`tasks/m235-t2a-journal.md`, plus the four orchestrator-declared planning
artifacts (`.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md`,
`tasks/milestone_ledger.md`, `tasks/m235-t2a-handoff.md`,
`tasks/m235-t2b-handoff.md`).

`git diff --name-only e2a05e2..7c94af3` returned exactly seven paths, all in
set:

```
crates/graph/src/cycle_groups.rs
crates/graph/src/lib.rs
crates/graph/tests/cycle_groups_test.rs
tasks/m235-t2a-handoff.md
tasks/m235-t2a-journal.md
tasks/m235-t2b-handoff.md
tasks/milestone_ledger.md
```

Per commit:

- `34f5fe4` -> the four crate:graph + journal paths only.
- `7c94af3` -> the two handoffs + journal SHA fill-in + ledger row only.

`tests/fixtures/basic/artifacts/includes/report.json` appears in NEITHER
commit. Neither `crates/graph/src/graph.rs`,
`crates/graph/src/closure_equivalence_tests.rs`, `crates/graph/Cargo.toml`,
nor any path under `crates/cluster`, `crates/cli`, `crates/ui`,
`crates/report`, `tests/`, or `tools/` appears anywhere in the range. The
additive-only property is therefore structural, not merely asserted.

## 3. Fresh gate results (reviewer-run, not journal-relayed)

| Handoff gate | Command | Result |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_test` | `9 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph` | `112 passed / 0 failed / 0 ignored / 4 suites` |
| 3 | `python tools/distill.py -- cargo build -p ariadne-graph` | `ok (exit 0, 0 warnings emitted)` |
| 4 | `python tools/hygiene_check.py` | zero R1/R2 workspace-wide; zero R3 in either new file |
| 5 | `python tools/encoding_guard.py --staged` | clean, no output |
| 6 | `python tools/checkpoint_guard.py status` | fixture absent from dirty section (section 1) |
| 7 | commit | both commits in set (section 2) |

Both pinned verdict lines reproduce EXACTLY. Neither is vacuous:

- gate 1 selected 9 tests, not zero; the capture shows the
  `cycle_groups_test` binary running 17.87 s, which is the 100 000-node chain
  doing real work at 64 KiB.
- gate 2 selected 112 tests across 4 suites. The capture names passing
  `graph::closure_equivalence::*` cases (including
  `test_compute_closures_matches_bfs_oracle_on_all_existing_fixtures` and
  `test_random_digraphs_with_cycles_match_bfs_oracle`) and shows the split
  101 lib + 9 cycle_groups + 2 m22_marker + 0 doc-tests = 112.

IV-10 is proven directly, not by assertion: `closure_equivalence_tests.rs:515`
still calls `graph.detect_cycles()`, that file is byte-unchanged in the range,
and the suite is green in a run that also builds the new module.

## 4. Workspace-wide acceptance pass

Verbatim canonical line, reviewer-run:

```
1165 passed / 0 failed / 4 ignored / 87 suites
```

Baseline `1156 / 0 / 4 / 86`. Delta: **+9 passed, +0 failed, +0 ignored,
+1 suite** - exactly the predicted T2-A delta, and fully attributed to the one
new `cycle_groups_test` target carrying nine `#[test]` functions. Unlike
Phase 1's residual (ledger F11), there is no unattributed remainder here.

Gates that could not be run, with verbatim results:

- `python tools/testrun.py --baseline` -> `usage error: missing value for
  --baseline` (exit 2). Ledger M23.5 F9, reproduced once, recorded once, not
  treated as a gate failure and not worked around. The full-workspace
  invocation it wraps (`python tools/testrun.py --distill --timeout 3600`) was
  run and compared by hand instead.
- `python tools/ui_smoke.py` -> refused, verbatim:
  `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
- `python tools/bench_compare.py` -> refused, verbatim: identical text.

Both refusals are ledger M23.5 F10 and were attempted independently, once
each. Neither was routed through `distill.py`.

Other workspace gates: `python -m unittest discover -s tools -p "*_tests.py"`
-> `Ran 184 tests ... OK`. `python tools/encoding_guard.py --all` -> no output.
`python tools/hygiene_check.py` -> zero R1/R2.

Two `;`-chained variants (`hygiene_check.py; Write-Output ...`,
`testrun.py ...; Write-Output ...`) were refused with the same catch-all
boilerplate; each plain form was then run separately and did execute. This is
a second instance of the recorded allowlist trap and is folded into finding 9.

## 5. Per-commit verification evidence

### `34f5fe4` - feat(graph): iterative CycleGroup detection with closed-walk witness

+1260 / -0 across 4 files; `cycle_groups.rs` 527 lines,
`cycle_groups_test.rs` 564 lines, `lib.rs` +2, journal +167. Implementation +
test output is 1093 lines against the ~1200 sizing bound - inside the law.
Full diff read from the checkout.

`crates/graph/src/cycle_groups.rs`:

- The frozen plan sec1.3 contract is reproduced verbatim: `CycleGroup` with
  `member_files`, `representative_cycle`, `external_in_count`,
  `external_out_count`, the same derives, the same doc text (with the handoff's
  `[c0, 0]` typo correctly resolved to the plan's `[c0, c0]`), and
  `pub fn detect_cycle_groups(&self) -> Vec<CycleGroup>` as an inherent
  `impl IncludeGraph` block. No signature drift.
- R2 no-language-naming posture holds: the module names no language. The only
  `LanguageId::cpp()` in the unit is in the test file, copied from the
  sanctioned `closure_equivalence_tests.rs` fixture pattern.
- `self.adjacency()` is called at line 42 - production does NOT duplicate the
  adjacency loop, satisfying the `graph.rs:78-82` doc contract. The only other
  rebuild is the test-side `test_adjacency`, which cites `graph.rs:83-95` in a
  doc comment exactly as handoff sec6 required.
- No `Rc`/`RefCell`/`Cell`/`thread_local`/`static mut` in production.

### `7c94af3` - docs(tasks): plan, ledger rows, T2-A/T2-B handoffs

+917 / -4 across the two handoffs, the journal SHA fill-in, and a 5-line ledger
row. Content is planning material and the journal's own SHA record. Nothing out
of set. Finding 8 records that the subject names the plan file, which this
commit does not contain.

## 6. Lowlink update points (D-A6) - the #1 audit ask

Audited against `crates/cluster/src/scc.rs:100-241` (the Phase 1 evidence) and
against the recursive Tarjan body. The graph machine is a faithful shape
mirror: `explore_from`, `enter_vertex`, `advance_frame`, `claim_next_edge`,
`relax_lowlink_toward_index`, `retire_frame`, `emit_group_if_root`,
`finish_child` - same names, same responsibilities, same control flow.

**PUSH** (`enter_vertex`, cycle_groups.rs:157-167): assigns `index` and
`lowlink` from the shared counter, pushes onto the Tarjan stack, sets
`on_stack`, opens a frame at neighbour 0. Matches the cluster machine
(`scc.rs:125-135`) and the recursive head exactly.

**ADVANCE** (`advance_frame`, :173-185): unindexed neighbour -> descend, with
tree-edge propagation DEFERRED to pop; indexed neighbour still `on_stack` ->
`lowlink[v] = min(lowlink[v], index[w])` via
`relax_lowlink_toward_index` (:209-219); indexed neighbour NOT on stack (a
cross edge into a completed SCC) -> no update, which is correct. Matches
`scc.rs:139-154`.

**POP** (`retire_frame`, :226-230):

```rust
let frame = self.frames.pop().expect("retire_frame runs with a live frame");
self.emit_group_if_root(frame.vertex);   // emit FIRST
self.finish_child(frame.vertex);        // then min(lowlink[parent], lowlink[child])
```

Emission precedes propagation, structurally - there is exactly one pop path and
the emit call is the first statement. This is precisely the Phase 1 finding
the handoff ordered not re-derived (`scc.rs:190-198`), and it is preserved.
The classic trap (propagating on call return, or propagating before emitting)
is excluded by construction, not by comment.

`finish_child` (:258-273) reproduces the cluster semantics with a different but
equivalent spelling: the cluster version uses
`let Some(parent_frame) = self.frames.last() else { return };`, the graph
version uses a `child_completed` predicate plus `expect`. Same behaviour -
propagate only when a parent frame exists, otherwise the retiring frame was the
seed root and there is nobody to propagate to.

`emit_group_if_root` (:235-253) reproduces the root rule including the
`lowlink == index` test, the pop-through-vertex loop, `on_stack` removal, and
the ascending intra-group sort (D-A3). This is the one intentional behavioural
difference from the cluster machine, which sorts by `SymbolId::0`; sorting by
`FileId` here is required by D-A3/IV-5.

**Duplication is the accepted shape, not a cross-crate dependency.** The word
`cluster` appears in `cycle_groups.rs` only in doc comments (lines 6, 83, 221).
`crates/graph/Cargo.toml` is byte-unchanged and lists no `ariadne-cluster`
edge; `crates/cluster/Cargo.toml` is likewise untouched in the range. The
~60-line frame-machine duplication is the deliberate, pre-accepted shape
sharing D-A7 mandates. The new frame/walk code is NOT a verbatim copy - it is
re-typed for `FileId` and gains the Kahn ordering, degree computation, and
witness finder - so it is duplication of shape, not of logic.

## 7. IV-5 determinism and the tie-break (D-A2)

`order_reverse_topological` (:309-337) builds the condensation over the emitted
groups, then runs Kahn's algorithm on a `BinaryHeap<Reverse<(FileId, usize)>>`
keyed on `groups[index][0]`, emitting sinks first. `groups[index][0]` is the
smallest member because `emit_group_if_root` sorts each group ascending, so the
key is exactly the tie-break IV-5 names. The `usize` half of the key can never
decide anything: two distinct groups cannot share a smallest member, because
IV-1 makes the groups disjoint.

The order is invariant under HashMap/HashSet iteration order, and this is not
merely argued - it is measured. `condensation_edges` iterates a `HashMap`
(`:358`) and then each per-group `HashSet<usize>` of successors (`:374`), so
the *push* order into the predecessor lists is nondeterministic. It does not
matter: a binary heap's pop sequence is fully determined by its contents, not
by insertion order, and each `out_degree` counter reaches zero exactly once.
`external_degree_pairs` iterates the same `HashMap` but only accumulates
counts, so it is order-independent by construction.

The test suite exercises this for real rather than trivially.
`test_two_disjoint_groups_emit_in_reverse_topological_order` calls
`connected.detect_cycle_groups()` twice and asserts equality. Because Rust's
`RandomState` differs per `HashMap` instance, the two calls build two
adjacency maps with different iteration orders inside a single process - so
this is a genuine cross-instance determinism lock, not a tautology. The same
test also pins the incomparable-group tie-break with a connector-free fixture
(`tied[0] == {1,2}`, `tied[1] == {3,4}`), which is the only case where the
min-heap key actually decides anything.

Adversarial note: my attempt to build an independent executable cross-check
was blocked (see finding 10), so I reasoned about the harder cases directly.
The equal-degree-condensation tie family is the same code path as the pinned
two-group case; what the two-group fixture does not exercise is a >=3-group
incomparable set where two groups become sinks at different times. The
analytical argument above covers it, but it is argued rather than measured.

## 8. IV-3, IV-4, IV-1, IV-2, IV-7 - independent assessment

**IV-3 edge cyclicity - the headline bug fix. Genuinely locked, in the
representation the milestone adopts.** The Red claim is independently
confirmed, not taken on faith. I traced the frozen oracle at
`graph.rs:260-287` against the exact fixture: `detect_cycles` seeds at node 1,
walks `1 -> 2 -> 3`, sees `3 -> 1` with 1 on `rec_stack`, and reports
`[1,2,3]`. It then returns to node 1 and examines `1 -> 3`; by that point 3 has
left `rec_stack`, so the chord produces NOTHING. Old output is therefore one
path `[1,2,3]`, while the chord's two endpoints are both in that reported path
and so both classed `cycle`. The wart is exactly as the handoff describes, and
the R2 assertion provably fails against the old API.

The Green form asserts the group-membership iff instead of per-edge cycle
membership. That is the faithful translation, not a weakening: under the new
representation there is no per-edge cycle report, and plan sec1.3 mandates the
UI edge rule as "class `cycle` iff both endpoints are in the **same** group".
The asserted direction is precisely the one that kills the unstyled-edge wart.
The 100 000-node chain, the two disjoint groups, the external-degree fixture,
the 8 seeded witness fixtures and the 200-node tangle all hold.

**IV-4 witness validity.** I audited `representative_cycle_for` (:416-425) and
`closed_walk_witness` (:450-488) directly rather than relying on the suite's
own helper. Every consecutive pair in the emitted walk is sound: the pairs
inside `path[position..]` were traversed as real adjacency entries by
`claim_next_member_neighbor` (:508-527), and the closing pair is the back-edge
that was just examined in `adjacency`. Closure is exact (`witness[0] ==
witness[position] == witness[last]`), length is >= 2 by construction, and
every node is a member because `claim_next_member_neighbor` filters by
`member_set` and only ever descends into members. A self-loop edge inside a
nontrivial group correctly yields `[u, u]`, which is a valid closed walk on a
real edge.

The `return Vec::new()` fallback (:466-471) is unreachable for a nontrivial SCC:
a strongly connected component with >= 2 members contains a cycle reachable
from any member, and DFS finds a back edge whenever a reachable cycle exists.
The `debug_assert!(false, ...)` documents exactly that. It fires in test/debug
builds and would be a hard failure rather than a silent empty witness, which is
the right failure mode.

**IV-1 / IV-2.** `filter_cyclic_groups` + `group_is_cyclic` (:281-302) keep
every group of size >= 2 and exactly those singletons carrying a self-loop in
the deduplicated adjacency - membership iff nontrivial SCC or self-loop
include, which is IV-2 verbatim. D-A1 seeds over the union universe (:67-78),
matching `compute_closures` at `graph.rs:124-130` rather than
`detect_cycles`'s `file_paths`-only seed, and sorts ascending so the traversal
is deterministic before any tie-break. Groups are disjoint by construction
(Tarjan pops each vertex from the stack exactly once). Finding 1 records the
one thing here that is correct but unlocked.

**IV-7.** `test_deep_include_chain_completes_on_default_stack` builds
100 000 registered files `f0..f99999` with `fi -> f(i+1)` plus `f99999 -> f0`
and runs `detect_cycle_groups()` on a `Builder::stack_size(64 * 1024)` thread.
The production path is fully iterative: the Tarjan machine holds frames in a
`Vec` and the witness walk holds its own `Vec<WalkFrame>`, so neither grows the
OS stack with depth. Gate 1's 17.87 s for that target is the chain executing to
completion at 64 KiB, and gate 2 shows the whole crate green with it.

**Additive-only.** `detect_cycles` (`graph.rs:234`) and `dfs_cycle`
(`graph.rs:260`) both survive at their pinned line numbers;
`generate_report_with_cycle_policies` is untouched; `detect_cycle_groups` has
exactly one production definition (`cycle_groups.rs:39`) and ZERO production
call sites - grep across the workspace finds it only in the module, the test
file, and planning documents. No third representation exists, and the
coexistence is the plan-sanctioned temporary state of handoff sec2.1 with its
removal owned by T3.1. Because no pre-existing file was edited, no pre-existing
graph behaviour could have been altered to accommodate the new code.

## 9. Hygiene, locks, dead code

`python tools/hygiene_check.py` reports zero R1/R2 workspace-wide; every line
it prints is an R3 warn on a legacy file. Zero findings in either new file, so
the implementer's claim of zero R3 in `cycle_groups.rs` and
`cycle_groups_test.rs` is confirmed - and it is stronger than the Phase 1
copied-body precedent required, because the copied `InlineLcg` constants were
kept verbatim (so the seeded corpus is unchanged) while the loop locals were
renamed.

Line counts: `cycle_groups.rs` 527, `cycle_groups_test.rs` 564, `lib.rs` 20,
`graph.rs` 704 (unchanged, as sec2.1 predicted). All far under the 1000-line
ceiling. Longest function is `closed_walk_witness` at ~39 lines; all functions
are far under 100. No source line approaches 2000 characters. No `tmp`, `res`,
`foo`, `bar`, `test1`, and no single-letter locals in authored code.

Gate 3's `0 warnings emitted` is the load-bearing dead-code evidence, and it is
fresh. Deviation E is confirmed genuine: `group_is_informational` returns ZERO
matches across the entire `crates/` tree - it is absent, not present-and-unused.
The pre-existing `cycle_is_informational` at `graph.rs:299` and the six
`informational_cycle_count` unit tests are untouched, so IV-11's seam is intact.

Frozen locks: `graph.rs` byte-unchanged; `crates/graph/Cargo.toml`
byte-unchanged with no `cluster` dependency; `closure_equivalence_tests.rs`
byte-unchanged and its suite green; `tests/golden/**` untouched;
`encoding_guard.py --all` clean and `--staged` clean.

## 10. Findings

No blocking findings.

1. **NON-BLOCKING, handoff-spec / planner.** IV-2's union seed universe (D-A1,
   `cycle_groups.rs:67-78`) is implemented correctly but locked by no test.
   Every one of the nine tests registers every endpoint it uses -
   `graph_from_shapes` registers its `files` list and `random_graph` registers
   `1..=file_count` - so replacing the union with `detect_cycles`'s
   `file_paths`-only seed (`graph.rs:240-241`) would leave all nine green
   while violating IV-2 for a cycle through a never-registered endpoint. The
   handoff's sec6 table pins the exact nine names, so this is not an
   implementer omission. Carry-forward: T2-B, which is test-only, adds a
   fixture with an unregistered edge endpoint on a real cycle.
2. **NON-BLOCKING, implementer.** T2.2's witness Red was never observed
   (deviation C). The production code is sound on independent reading
   (section 8), and the handoff's own escape clause pre-authorised exactly this
   disclosure - the implementer recorded it rather than fabricating a Red, which
   is the correct behaviour. But the witness finder still has no demonstrated
   Red anywhere in the milestone. Carry-forward for T2-B: demonstrate it, or
   let the frozen oracle cross-check it.
3. **NON-BLOCKING, planner.** IV-3 is asserted only in its group-membership
   (iff) form, which is what plan sec1.3 mandates, but the plan's stated
   *consequence* - every intra-group edge lies on some cycle - is discharged by
   SCC correctness rather than by any assertion. Consequence: T2-A's suite
   would not catch a Tarjan defect that over-merges groups, because a too-large
   group still yields edge-backed, in-group witnesses. T2-B is the designated
   verifier; recommend its oracle suite add a per-edge reachability assertion
   (destination reaches source within the group).
4. **NON-BLOCKING, implementer.** `adjacency()`'s doc comment
   (`graph.rs:78-82`) now understates its call sites: it says "the single
   adjacency-build call sites are `compute_closures` and `LoadAnalyzer`'s
   `file_adjacency` memo", and `detect_cycle_groups` is a third. `graph.rs` is
   byte-locked out of T2-A's scope, so this cannot be corrected in this unit
   and was correctly reported rather than fixed. Carry-forward to T3.1.
5. **NON-BLOCKING, implementer.** `FrameStep` is a boolean in disguise:
   `advance_frame` returns `Descended`, `EdgeVisited`, or
   `NeighborsExhausted`, but the sole match at `cycle_groups.rs:148` tests only
   `NeighborsExhausted` and discards the other two. No warning fires because
   they are constructed, and plan sec1.4 mandates the `advance_frame`
   decomposition, so this is an observation, not a defect.
6. **NON-BLOCKING, handoff-spec.** The test name
   `test_deep_include_chain_completes_on_default_stack` is misleading: the test
   spawns an explicit 64 KiB thread, so "default" means the plan's default
   small-stack constant, not the process default stack. Handoff sec5 prescribed
   both the name and the thread size, so the implementer complied.
7. **NON-BLOCKING, handoff-spec.** `tasks/m235-t2a-handoff.md:124` carries the
   doc-comment typo `/// [c0, 0] for a self-loop group.`, contradicting plan
   sec1.3 line 50 and the handoff's own D-A5. The implementer resolved toward
   the plan: `cycle_groups.rs:26` reads `[c0, c0]`, and test 4 pins `[a, a]`.
   The typo should be fixed in the handoff artifact so a future reader does not
   re-derive the drift.
8. **NON-BLOCKING, implementer.** Commit `7c94af3`'s subject claims it lands
   "M23.5 plan, ledger rows, and T2-A/T2-B handoffs", but the plan file is NOT
   in that commit - `--name-only` shows only the two handoffs, the journal SHA
   fill-in, and the ledger row. The plan was already tracked at or before
   `2708425`. Nothing out of scope landed, so the audit is unaffected; but a
   subject naming a file it does not contain weakens message-as-evidence, which
   is the same class of record defect as Phase 1's verdict finding 8.
9. **NON-BLOCKING, configuration.** Ledger F10 confirmed, with a second
   instance of the same trap. `ui_smoke.py` and `bench_compare.py` are both
   refused with the catch-all rule; separately, `;`-chained command variants
   (`hygiene_check.py; Write-Output ...`) are also refused, so a shell chain
   cannot be used to capture an exit code. Both plain forms ran fine. **There is
   still no perf evidence for `detect_cycle_groups`**, which is a real surface
   (`payload_tail_bench.rs` measures the cycle path and T3.2 must re-pin it).
   Also measured here: a full workspace test run invalidates the stat entry of
   `tests/fixtures/basic/artifacts/includes/report.json`, so `git status` shows
   it ` M` again after every run even though `git ls-files --eol` stays
   `i/lf w/lf`, no `\r` byte is present, and `git diff --numstat` stays empty.
   Mechanism is `core.autocrlf=true` (measured) plus the unpinned path. This
   confirms F12's characterisation and makes the "every `commit_scoped.py`
   carries an explicit `--scope`" rule non-optional. Enforcement lives in the
   allowlist, not in agent prose; do not route the refused commands through
   `distill.py`.
10. **NON-BLOCKING, configuration.** I could not execute my own adversarial
    fixtures. This reviewer's write grant is confined to
    `tasks/<milestone>-t<n>-verdict.md` and the execution-results file (writing a
    scratch harness under `tasks/` would pollute T2-B's declared commit scope),
    `python -c` is refused, and `git ls-tree`/`git log -- <path>` forms were
    refused by the allowlist. I therefore substituted source-level
    verification: I read the full production module, the frozen
    `dfs_cycle`/`detect_cycles` oracle, the cluster frame machine, and the whole
    test file, and I traced the chord fixture through the old oracle by hand
    (section 8) to confirm the R2 Red independently. Net effect: the IV-1/2/3/4
    and tie-break conclusions rest on code reading plus the fresh gate runs,
    not on executable adversarial fixtures. Findings 1 and 3 are the two places
    where that substitution matters most, and both are carry-forwards for
    T2-B's test-only scope.

## 11. Deviations adjudicated

| # | Deviation (journal label) | Disposition |
|---|---|---|
| A | report.json was already clean at session start, so preflight step 0's CRLF premise did not hold (D-A) | **ACCEPTED.** Independently confirmed the end state is correct: path absent from the numstat dirty section, `i/lf w/lf`, no `\r`, byte-equal to HEAD, in no commit. The step was executed anyway and cost nothing. Finding 9 records why the dirt recurs. |
| B | abort code is `0xc00000fd STATUS_STACK_OVERFLOW`, not the predicted `0xc0000409` (D-B) | **ACCEPTED.** Same overflow class. `STATUS_STACK_OVERFLOW` is what Windows raises when a thread exhausts its guard page; `STATUS_STACK_BUFFER_OVERRUN` is the fail-fast code for a `/GS` overrun. The IV-7-bearing part of the assertion is unchanged: 100 000 recursive frames cannot fit in 64 KiB, and the abort is genuine. The prediction lived in the handoff, not in code. |
| C | T2.2's empty-witness Red was unobservable because T2.1-G already fills every witness (D-C) | **ACCEPTED with carry-forward (finding 2).** The handoff's sec5 pre-authorised exactly this escape. The disclosure is honest and the code is sound on independent reading, but the Red remains undemonstrated and must be closed in T2-B. |
| D | gate 3 failed twice (exit 101) on the implementer's own typos before going green | **ACCEPTED.** Ordinary Green-phase compile iteration. The final state is what I re-ran: `ok (exit 0, 0 warnings emitted)`. Self-declared, correctly. |
| E | `group_is_informational` deliberately not added, to avoid a `dead_code` warning | **ACCEPTED.** Confirmed genuinely absent: zero matches across `crates/`. Plan sec1.4 lists the name under a naming mandate, not a mandate to create the function, and classification against `CyclePolicy` is Phase 3's (IV-11). Adding it now would fire exactly the Phase 1 finding-5 rejection it avoids. |
| F | `detect_cycles`/`dfs_cycle` retained rather than deleted (orchestrator declaration 1+2) | **ACCEPTED.** Both survive at `graph.rs:234` and `:260`; the workspace builds; `Cargo.toml` untouched. Correctly treated as additive-only per plan D1-B with removal owned by T3.1. |
| G | the R2 Red staged run temporarily dropped the aborting R1 test | **ACCEPTED - legitimate isolation, not a weakening.** Handoff sec5 explicitly says an abort kills the whole test binary so sibling tests report nothing, and orders R1 to be run as its own invocation; running R2 with only R2 staged follows that instruction. Nothing was weakened: the committed test file contains both tests, and the R2 Red was a genuine runtime failure whose validity I confirmed by tracing the frozen `dfs_cycle` oracle against the exact fixture (section 8). The staged file is scratch state, not the committed artifact. |

## 12. Housekeeping (confirmed, not fixed, not committed)

`tasks/m235-t1-verdict.md` (326 lines, Phase 1 APPROVED, 0 blocking / 12
non-blocking) and `tasks/m235-execution-results.md` (132 lines, Phase 1
acceptance record) both exist and are intact. Both remain untracked and are
declared into T2-B's commit scope. I did not commit them.

This review's own execution-results addendum is written to
`tasks/m235-t2a-execution-results.md` rather than overwriting
`tasks/m235-execution-results.md`, because that path holds the Phase 1 record I
was instructed to leave intact. That leaves three untracked files for T2-B to
scope: the two Phase 1 deliverables plus this addendum.

## 13. Milestone state

T2-A is APPROVED and shippable: `crate:graph` now carries the milestone's
`CycleGroup` currency, additive, warning-free, with its own nine-test contract
suite, and the workspace is green at `1165 / 0 / 4 / 87`.

`tasks/m235-t2b-handoff.md` (T2.3, equivalence-to-old) is now unblocked. It
should absorb findings 1, 2, and 3 as required test work. Phase 3 (T3.1
report/policy/CLI rewiring, T3.2 UI DTO) and Phase 4 (T4.1) remain
UNAUTHORIZED. Findings 4, 6, and 7 are handoff/plan artifact text fixes for the
next planning touch; findings 8, 9, and 10 are record and configuration
follow-ups for the ledger.