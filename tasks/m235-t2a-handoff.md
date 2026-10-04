# M23.5 T2-A Handoff - CycleGroup Core + Iterative Graph Tarjan (crate:graph)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 2 of 4, unit A of 2 (T2.1 + T2.2). Unit B is `tasks/m235-t2b-handoff.md` (T2.3).
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 2 (T2.1, T2.2), sec1.2 invariants, sec1.3 contracts
**Kickoff:** `tasks/m235-kickoff.md` (its sec3 authorized Phase 1 only)
**Phase 1 verdict:** `tasks/m235-t1-verdict.md` - APPROVED
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`
**Journal:** `tasks/m235-t2a-journal.md` (you create it; record your own SHAs in it)

Every signature, line number, and constant in this file was read out of the
tree at `e2a05e2`. If any line reference here disagrees with the file you read,
the FILE wins -- report the drift in your journal, do not silently adapt.

---

## 0. PREFLIGHT - do this before writing a single line of code

`tests/fixtures/basic/artifacts/includes/report.json` is currently dirty in
this worktree with CRLF line terminators (Phase 1 verdict finding 12). Its
content is identical to the committed blob after normalization; only the
terminators differ, and the path is NOT covered by any `.gitattributes` pin.

**Step 0.1 - restore it to LF, byte-safely.**
Read it with the Kilo `Read` tool, then write it back with the Kilo `Write`
tool so the terminators are LF. NEVER use PowerShell `Set-Content`, `Out-File`,
`>`, `>>`, or `Get-Content | -replace` on a tracked file -- those are the
documented cause of this repo's mojibake incidents (`AGENTS.md` File-Edit
Discipline). A byte-level `open(path, "wb")` inside a Python snippet is the only
other sanctioned writer.

**Step 0.2 - do NOT try to fix it with git.** `git checkout --` and
`git restore` are mutating git and are DENIED to every role in this topology,
including you. Content restoration through the file tools is the only route.

**Step 0.3 - prove it worked.** Run `python tools/checkpoint_guard.py status`
(gate 6). The dirty section must no longer list that path. If it still does,
say so verbatim in the journal and continue -- do not commit it (step 0.4 makes
that impossible anyway).

**Step 0.4 - every commit in this handoff carries an explicit `--scope`.**
Never rely on a default sweep. Gate 7 is written with `--scope`.

---

## 1. Objective and Invariants

### Objective

Add the milestone's new cycle currency to `crate:graph` as a **new, additive**
public API: the `CycleGroup` type and an **iterative** Tarjan that returns
entangled groups, plus a per-group closed-walk witness. No existing public API
is removed or altered in this unit.

### Invariants this unit must establish

- **IV-2 Cycle iff nontrivial:** a file is in some `CycleGroup` iff it lies on at least one include cycle. Group membership iff membership in a nontrivial SCC (size >= 2) or a self-loop include.
- **IV-3 Edge cyclicity:** an include edge `u -> v` is cyclic iff `u` and `v` are in the same group (or `u == v` is a self-loop). Consequence: **no edge inside a group may ever be missing from a reported cycle** -- this is the unstyled-edge-between-two-red-nodes wart the milestone exists to remove.
- **IV-4 Representative cycle is a witness:** `representative_cycle` is a closed walk `[c0, c1, ..., ck, c0]` whose every consecutive pair is a real include edge inside the group. Self-loop groups yield `[c0, c0]`.
- **IV-5 Determinism:** group order is reverse-topological over the condensation DAG with a deterministic tie-break (smallest `FileId` of group members ascending); members within a group ascending by `FileId`.
- **IV-7 No unbounded recursion:** the include-graph cycle path does not grow the OS stack with graph depth.
- **R2 no-language-naming posture:** `cycle_groups.rs` names no language. Classification against `CyclePolicy` stays in the report layer (IV-11, Phase 3). `graph.rs:4-5` shows the sanctioned posture: only the policy *enum* may be named, and it is not needed here at all.

### Invariants this unit must NOT disturb

- **IV-10 Closure parity untouched.** `compute_closures`, its SCC-DP, and the `closure_equivalence` suite stay green. `crates/graph/src/closure_equivalence_tests.rs:515` still calls `graph.detect_cycles()` and still compiles.
- **IV-11 cycle-policy seam** stays exactly as it is. The six `informational_cycle_count` unit tests at `graph.rs:612-680` are untouched by this unit.
- **IV-1, IV-3 (report layer), IV-6, IV-8, IV-9, IV-12** are Phase 3/4 concerns.

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

- `crates/graph/src/cycle_groups.rs` (**new** -- the proactive extraction, mandatory)
- `crates/graph/src/lib.rs` (add `pub mod cycle_groups;` and `pub use cycle_groups::*;`, mirroring how `closures` and `graph` are wired at `lib.rs:5-6` and `12-18`)
- `crates/graph/tests/cycle_groups_test.rs` (**new** -- the T2.1 + T2.2 Red suite)
- `tasks/m235-t2a-journal.md` (**new**, yours)
- `tests/fixtures/basic/artifacts/includes/report.json` (preflight step 0 restoration ONLY; zero content change; must NOT appear in any commit)

You MUST NOT touch: `crates/graph/src/graph.rs`, `crates/graph/src/closure_equivalence_tests.rs`,
`crates/cluster/**`, `crates/cli/**`, `crates/ui/**`, `crates/report/**`,
`tests/fixtures/**` (beyond the single preflight path above), `tests/golden/**`,
`tools/**`, `.kilo/plans/**`, `Cargo.toml`, `Cargo.lock`.

No `git stash`, no branch change, no `git add .`.

### 2.1 Why `detect_cycles` survives this unit

Plan T2.1's refactor note says `graph.rs` "returns toward <= ~550 lines by
deleting `dfs_cycle`/`detect_cycles`". **Do not do that here.** Plan T3.1 owns
the deletion together with the call-site repointing, and `detect_cycles` has
seven live call sites -- `graph.rs:356` and `graph.rs:534` inside this crate,
and `crates/cli/src/commands/ui.rs:310`, `crates/cli/tests/payload_tail_bench.rs:295`,
plus two Scala integration tests -- three of them outside `crate:graph`.
Deleting it inside a `crate:graph`-only handoff would break the workspace build,
which is exactly the half-built seam the sizing law forbids. This unit is
strictly **additive**; both representations coexist until Phase 3 removes the
old one. That coexistence is temporary by plan decision (D1-B), not the dual
representation Phase 3 must eliminate.

**Expected post-migration size of `graph.rs`: unchanged at 704 lines.** The new
module is registered in `lib.rs`, not `graph.rs`, and `detect_cycle_groups` is
an inherent `impl IncludeGraph` block that lives in `cycle_groups.rs`. The
extraction is what protects the ceiling: `cycle_groups.rs` lands at an estimated
**~390 lines**, and when Phase 3 deletes `detect_cycles`/`dfs_cycle`
(`graph.rs:233-287`, 55 lines) `graph.rs` drops to ~649. Neither file approaches
1000. The ceiling pressure recorded in plan sec1.4 is discharged by the
extraction, not by a premature deletion.

---

## 3. API Contracts (verbatim from plan sec1.3 -- FROZEN, do not redesign)

```rust
/// A maximal entangled group in the include graph: files mutually
/// reachable via includes. Every edge inside a nontrivial group is on a cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleGroup {
    /// Members of the group, ascending by FileId.
    pub member_files: Vec<FileId>,
    /// Closed-walk witness [c0, c1, ..., ck, c0] over edges inside the group;
    /// [c0, 0] for a self-loop group.
    pub representative_cycle: Vec<FileId>,
    /// Groups condense into the DAG; these degrees describe edges crossing out of/into the group.
    pub external_in_count: usize,
    pub external_out_count: usize,
}

impl IncludeGraph {
    /// Iterative Tarjan SCC over the include adjacency; returns only cyclic
    /// (nontrivial or self-loop) groups, reverse-topological over the
    /// condensation DAG, deterministically tie-broken (IV-5). POLICY-FREE:
    /// classification against CyclePolicy stays in the report layer (IV-11),
    /// so the graph crate keeps its R2 no-language-naming posture.
    pub fn detect_cycle_groups(&self) -> Vec<CycleGroup>;
}
```

### 3.1 Real existing API this unit builds on (verified at `e2a05e2`)

These are the actual names. Do not paraphrase them.

| Item | Location | Note |
|---|---|---|
| `pub fn detect_cycles(&self) -> Vec<Vec<FileId>>` | `crates/graph/src/graph.rs:234` | stays; T2.3 freezes a copy of it, T3.1 deletes it |
| `fn dfs_cycle(...)` | `crates/graph/src/graph.rs:260-287` | 6 params; the frozen oracle body for T2-B |
| `pub(crate) fn adjacency(&self) -> HashMap<FileId, Vec<FileId>>` | `crates/graph/src/graph.rs:83-95` | **production code MUST call this.** Its doc comment at `:78-82` says "no other code may duplicate this loop" |
| `pub edges: Vec<IncludeEdge>` / `pub file_paths: HashMap<FileId, PathBuf>` | `graph.rs:21-22` | public; usable from an integration test |
| `pub mod graph;` + `pub use graph::*;` | `crates/graph/src/lib.rs:6`, `:14` | the wiring precedent for your two added lines |
| `#[cfg(test)] #[path = "closure_equivalence_tests.rs"] mod closure_equivalence;` | `graph.rs:702-704` | the `#[path]` precedent named in plan sec1.4 |
| `struct TarjanFrame<'a>` / `struct TarjanRun<'a>` | `crates/cluster/src/scc.rs:48`, `:58` | Phase 1 frame machine -- SHAPE reference only |
| `InlineLcg` (Knuth MMIX, no RNG dependency) | `crates/graph/src/closure_equivalence_tests.rs:373-393` | copy this pattern verbatim into your test file |
| `RANDOM_FIXTURES` locked-seed table | `closure_equivalence_tests.rs:405-416` | same pattern |
| `fn edge(source: u64, destination: Option<u64>) -> IncludeEdge` / `type FixtureShape = (u64, Option<u64>)` / `fn graph_from_shapes(...)` | `closure_equivalence_tests.rs:24`, `:22`, `:37-46` | copy these three into your test file |

`FileId` is `ariadne_repo::FileId` (a newtype over `u64`, as `graph.rs:240` uses
`all_files.sort()` and `graph.rs:348` uses `format!("{:016x}", id.0)`).

### 3.2 Additions this unit must make

Exactly two lines in `lib.rs`:

```rust
pub mod cycle_groups;
...
pub use cycle_groups::*;
```

`pub(crate) mod closures;` (`lib.rs:5`) is the precedent for keeping a helper
module crate-internal; you need `pub` here because `CycleGroup` is a public
contract and the crate re-exports everything.

---

## 4. Decisions the plan left open (planner resolutions -- adjudicate these)

The plan does not specify these. They are pinned here so the tests are
deterministic and the reviewer has something concrete to audit.

**D-A1 Seed universe.** Seed the outer Tarjan loop over the union universe --
`file_paths` keys, plus every edge `source`, plus every resolved (`Some`)
`destination` -- matching `compute_closures` at `graph.rs:124-130`. Do NOT seed
over `file_paths` alone as `detect_cycles` does at `graph.rs:240-241`: an
include endpoint that was never `register_file`'d can sit on a real cycle, and
`detect_cycles` silently misses it. IV-2 requires the union. Sort the seed list
ascending so the traversal is deterministic before any tie-break is applied.
`destination: None` (unresolved) edges never enter the universe.

**D-A2 Group order (IV-5), concretely.** Raw Tarjan emission order is already
*some* reverse-topological order, but IV-5 names a specific tie-break, so make
the order structural rather than incidental: build the condensation over the
emitted groups, then order it with Kahn's algorithm using a min-heap keyed on
the group's smallest member `FileId`. Emit sinks first. This is the only
formulation of IV-5 that is checkable by reading the code, and it is what
`test_two_disjoint_groups_emit_in_reverse_topological_order` asserts.

**D-A3 Intra-group member order.** Each group's `member_files` is sorted
ascending by `FileId` before the group is returned.

**D-A4 External degrees.** Compute both degrees from the deduplicated
`adjacency()` (i.e. one count per distinct `(source, destination)` pair, not per
`IncludeEdge` record -- `add_edge` at `graph.rs:42-46` only rejects exact
duplicate records, so the same pair can appear with different `context_name` or
`source_location`). Count an edge as external only when exactly one endpoint is
in the group; an edge with both endpoints in the group is internal, and so is a
self-loop. `external_in_count` counts pairs entering the group,
`external_out_count` pairs leaving it.

**D-A5 Witness search (T2.2).** The witness is the closed walk from the first
back-edge found by a path-tracking walk restricted to group members, structurally
the old `current_path` / `rec_stack` walk at `graph.rs:268-286` but iterative and
bounded to one group. Emit `[path[pos..]] + [path[pos]]`, i.e. close the walk.
A self-loop group short-circuits to `[c0, c0]`. Existence, not exactness, is the
contract: exactness is nondeterministic by design under DFS choice. Do not
attempt a shortest cycle -- that is a different algorithm and a different cost.

**D-A6 Frame-pop ordering -- DO NOT RE-DERIVE, THIS IS AUDITED EVIDENCE.**
Phase 1's reviewer audited the cluster frame machine and confirmed the ordering
(`crates/cluster/src/scc.rs:190-198`, and the reviewer's verdict section 4.1):

```rust
fn retire_frame(&mut self) {
    let frame = self.frames.pop().expect("retire_frame runs with a live frame");
    self.emit_group_if_root(frame.vertex);   // <-- emit FIRST
    self.finish_child(frame.vertex);          // <-- then min(lowlink[parent], lowlink[child])
}
```

Emission happens BEFORE lowlink propagation, because the recursive body it
replaces emits at the end of the callee before the caller applies
`min(lowlink[v], lowlink[w])`. Your graph-side machine must use the same order
and the same three update points:

- **push** -- assign `index` and `lowlink` from the shared counter, push onto the Tarjan stack, set `on_stack`, open a frame at neighbor 0;
- **advance** -- unindexed neighbor -> descend (tree-edge propagation is DEFERRED to pop); indexed neighbor that is `on_stack` -> `lowlink[v] = min(lowlink[v], index[w])`;
- **pop** -- `emit_group_if_root` FIRST, then `min(lowlink[parent], lowlink[child])`.

The classic trap -- propagating lowlink on call return instead of on frame pop,
or propagating before emitting -- is structurally excluded by emitting first
inside the single pop path. Read `crates/cluster/src/scc.rs:110-241` before you
write yours; reuse the SHAPE, never the code.

**D-A7 No cross-crate dependency.** `graph` and `cluster` stay decoupled. The
~60-line frame-machine duplication between them is deliberate and pre-accepted
by the reviewer for Phase 1; do not plan, add, or request a shared internal
crate. `cargo build` will prove it: if `crates/graph/Cargo.toml` gains a
dependency you did not already see at `Cargo.toml:14-33`, you have crossed the
line.

**D-A8 Hygiene decomposition (plan sec1.4).** All functions < 100 lines. One
predicate helper per frame-machine step. The plan names these roles:
`run_tarjan`, `advance_frame`, `finish_child`, `emit_group_if_root`,
`representative_cycle_for`. Intent-revealing names are mandatory:
`member_files`, `representative_cycle`, `external_out_count`, `on_stack`,
`child_completed`, `group_is_informational`. No `tmp`, `res`, `scc_tmp`, and no
single-letter locals in code you author.

---

## 5. RGB Protocol

Strict Red-Green-Refactor. **Both Reds below compile against the tree exactly
as it stands today.** Neither is a `todo!()`, a skeleton, or an
unimplemented signature -- a test that fails to compile is not Red.

### T2.1-R1 (Red) -- IV-7, deep-chain stack overflow

In `crates/graph/tests/cycle_groups_test.rs`:

```
test_deep_include_chain_completes_on_default_stack
```

Build 100 000 registered files `f0 .. f99999` with edges `fi -> f(i+1)` and
`f99999 -> f0` (one giant cycle), and assert a cycle is found. Run the
traversal on a `std::thread::Builder::new().stack_size(64 * 1024)` thread --
64 KiB is the IV-7-bearing constant, matching `SMALL_STACK_BYTES` in
`crates/cluster/tests/scc_iterative_parity.rs`.

Write this Red calling **`detect_cycles()`** (which exists today).

**Expected failure reason:** the binary ABORTS, it does not report a failed
assertion. The capture shows
`thread '<unknown>' (<pid>) has overflowed its stack` and
`STATUS_STACK_BUFFER_OVERRUN (0xc0000409)`, and `testrun.py` reports
`### cargo test DRIFT (exit 2)` because zero result lines survive the abort.
This is the identical Red signature Phase 1 recorded
(`tasks/m235-t1-journal.md`, T1.1), so you will recognize it.

**Consequence you must plan for:** an abort kills the whole test binary, so
sibling tests in the same target report nothing. Run this Red as its own
invocation and expect a drift-exit; the Green run must return a real
`N passed / 0 failed` line.

**Why the 64 KiB number is not negotiable:** it is the assertion. 100 000
recursive frames need megabytes; 64 KiB cannot hold them, so a pass on that
thread IS the IV-7 proof.

### T2.1-R2 (Red) -- IV-3, the chord-blind edge

In the same file:

```
test_diamond_with_chord_reports_every_intra_group_edge_as_cyclic
```

Fixture, per plan T2.1 Red: `a -> b -> c -> a` plus the chord `a -> c`, and a
hanging node `d` with `c -> d` that is on no cycle. Assert:

1. `d` is in no group;
2. **every** edge whose both endpoints lie in the cyclic node set is itself
   reported on some reported cycle.

Write this Red against **`detect_cycles()`** as well.

**Expected failure reason:** assertion (2) fails, naming the offending chord
`a -> c`. Old DFS from `a` walks `a -> b -> c -> a`, reports the path
`[a, b, c]`, and never reports the chord `a -> c` -- the endpoints are both
`cycle`-classed but the edge carries no cycle, which is precisely the unstyled
edge between two red-glow nodes in plan sec1.1 item 1. Your assertion message
should name the edge pair so the failure is self-explaining.

### T2.1-G (Green) -- the module

Create `crates/graph/src/cycle_groups.rs` implementing sec3 plus D-A1..D-A4,
D-A6, D-A8. Add the two `lib.rs` lines from sec3.2.

Then **re-point R1 and R2 from `detect_cycles()` to `detect_cycle_groups()`**.
R1 turns green because the iterative machine cannot overflow 64 KiB. R2 turns
green because IV-3 is now structurally true: every edge with both endpoints in
one SCC lies on a cycle, since the SCC's mutual reachability plus that edge
gives a closed walk. **Log this call-site switch as an explicit step in your
journal** -- it is the one place where a Red written against the old API becomes
a Green against the new one, and the reviewer will look for it.

No dead stubs. `representative_cycle_for` is NOT stubbed in this step (Phase 1's
reviewer rejected an unused private helper for emitting `dead_code` against its
own zero-warning gate -- finding 5).

### T2.1 fixtures you must also cover (plan T2.1 Red, verbatim requirements)

- two-cycle fixture mirroring `graph.rs:501-539` -> exactly one group `{a, b}`, witness length 3 closing `[a, b, a]`, external degrees;
- diamond-with-chord (above);
- self-include `a -> a` -> group `{a}`, witness `[a, a]`;
- two disjoint groups plus a DAG connector -> reverse-topological group order (IV-5);
- empty and acyclic graphs -> empty vec.

### T2.2-R (Red) -> T2.2-G (Green) -- the witness

Add `representative_cycle_for` per D-A5, then add two tests to the same file:

```
test_seeded_digraphs_witnesses_are_edge_backed_closed_walks
test_known_two_cycle_survives_in_random_tangle
```

- **first:** over a seeded digraph corpus built with the `InlineLcg` pattern
  copied from `closure_equivalence_tests.rs:373-393` and a locked-seed table in
  the style of `:405-416`, verify every returned `representative_cycle`
  edge-by-edge against `graph.edges`: `witness.first() == witness.last()`, length
  >= 2, and every consecutive pair is a real include edge whose endpoints are
  both in the same group's `member_files`.
- **second:** embed a known 2-cycle inside a 200-node random tangle and assert a
  witness comes back for that group. Existence, not exactness (D-A5).

These two assertions are the Red for T2.2 in the meaningful sense: with
`representative_cycle` still empty (as T2.1-G leaves it for groups without the
short-circuit), the closed-walk verifier fails. If you find yourself unable to
observe that intermediate failure because T2.1-G already filled every witness,
say so in the journal rather than pretending to a Red you did not see.

### T2.2 Refactor

- No function >= 100 lines; no file >= 1000 lines. `cycle_groups.rs` target ~390 lines, hard ceiling 1000.
- `graph.rs` stays at 704 lines (sec2.1). If you find yourself editing it, you are out of scope -- report it.
- `python tools/hygiene_check.py`: zero R1/R2. R3 is warn-only; if a warn lands inside a verbatim-copied oracle body, that is acceptable and must be noted (Phase 1 precedent, verdict finding 7), but every name YOU author must be intention-revealing.

---

## 6. Required Tests (exact names, exact counts)

`crates/graph/tests/cycle_groups_test.rs` must contain exactly these nine
`#[test]` functions:

| # | Name | Locks |
|---|---|---|
| 1 | `test_deep_include_chain_completes_on_default_stack` | IV-7 (was R1) |
| 2 | `test_diamond_with_chord_reports_every_intra_group_edge_as_cyclic` | IV-3 (was R2) |
| 3 | `test_two_cycle_yields_one_group_with_closed_witness` | plan T2.1 two-cycle fixture, mirrors `graph.rs:501-539` |
| 4 | `test_self_include_yields_single_member_group` | IV-2 self-loop, witness `[a, a]` |
| 5 | `test_two_disjoint_groups_emit_in_reverse_topological_order` | IV-5, D-A2 |
| 6 | `test_acyclic_and_empty_graphs_yield_no_groups` | IV-2 negative |
| 7 | `test_external_degrees_ignore_intra_group_edges_and_self_loops` | D-A4 |
| 8 | `test_seeded_digraphs_witnesses_are_edge_backed_closed_walks` | IV-4, T2.2 |
| 9 | `test_known_two_cycle_survives_in_random_tangle` | T2.2 existence |

`crates/graph/Cargo.toml` has no `[[test]]` sections and no `autotests = false`,
so Cargo auto-discovers `tests/*.rs`. Adding the file needs NO manifest edit --
and if you find yourself wanting one, that is a signal you are doing something
outside this unit.

**Integration tests see only the PUBLIC API.** `adjacency()` is `pub(crate)`
(`graph.rs:83`), so your test file must build its own adjacency map from
`graph.edges` when it needs one, mirroring `graph.rs:83-95` semantics exactly:
skip `destination: None`, group by source, sort each destination list, dedup.
Put a comment on that helper citing `graph.rs:83-95` so the next reader knows
it is a deliberate mirror and not the forbidden duplicate of the production
loop. Copy the `edge()` / `graph_from_shapes()` shape from
`closure_equivalence_tests.rs:24-46` to build fixtures.

---

## 7. Gates (exact commands -- 7 invocations, the whole allowance)

Every gate is written in DIRECT form. **Never write a gate as
`python tools/distill.py -- <argv>`** -- the distiller is an argv pass-through,
so that form runs whatever `<argv>` names and bypasses the allowlist.

**Never write a bare-word test filter.** `python tools/testrun.py ... -- <word>`
passes `<word>` to cargo as a TEST-NAME filter. Phase 1's gate was written that
way, selected zero tests, and returned `0 passed / 0 failed / 0 ignored /
7 suites` with exit 0 -- it could never have observed a Red either. Verdict
finding 2. Use `--test <target>`, which `testrun.py` passes to cargo as
`--test <target>` and which therefore selects a real binary. Never use it.

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-graph
3. python tools/distill.py -- cargo build -p ariadne-graph
4. python tools/hygiene_check.py
5. python tools/encoding_guard.py --staged
6. python tools/checkpoint_guard.py status
7. python tools/commit_scoped.py --scope 'crates/graph/src/cycle_groups.rs' --scope 'crates/graph/src/lib.rs' --scope 'crates/graph/tests/cycle_groups_test.rs' --scope 'tasks/m235-t2a-journal.md' -m "feat(graph): iterative CycleGroup detection with closed-walk witness"
```

Gate 1 is run in three states: at R1 (expect drift-exit 2 with the stack-overflow
signature), at R2 (expect assertion 2 failing, naming `a -> c`), and at Green
(expect the canonical line below). Quote each verbatim in the journal.

**Expected verdict lines (gate 1, Green):**
`9 passed / 0 failed / 0 ignored / 1 suites`

**Expected verdict line (gate 2, Green):**
`112 passed / 0 failed / 0 ignored / 4 suites`

`112 / 4` is not a guess: `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph`
measured `103 passed / 0 failed / 0 ignored / 3 suites` at `e2a05e2`, and this
unit adds exactly one integration target carrying exactly nine tests. Gate 2 is
the IV-10 proof as well -- `closure_equivalence` runs inside it, so
`closure_equivalence_tests.rs:515`'s `detect_cycles()` call is proven still
compiling and green.

**Gate 3 must report zero warnings.** Phase 1's reviewer treated a warning-free
build as load-bearing evidence; a `dead_code` warn from an unused helper is a
finding.

---

## 8. Frozen Locks (must not change; each has a proving command)

| Lock | Proving command | Expected |
|---|---|---|
| IV-10 closure equivalence green, `detect_cycles` still compiling | gate 2 | `112 / 0 / 0 / 4`, no edit to `closure_equivalence_tests.rs` |
| IV-11 six `informational_cycle_count` unit tests unchanged (`graph.rs:612-680`) | gate 2 | green, `graph.rs` byte-unchanged in the commit |
| `adjacency()` single-source-of-truth (`graph.rs:78-95`) | `git diff` shows no edit to `graph.rs`; grep your own new module for a second adjacency loop | production calls `self.adjacency()` |
| no cross-crate dependency (`cluster` <-> `graph`) | gate 3; `crates/graph/Cargo.toml` byte-unchanged | zero new deps |
| R1/R2 hygiene | gate 4 | no R1/R2; `cycle_groups.rs` < 1000 lines; every fn < 100 lines |
| UTF-8 mojibake | gate 5 | clean, no output; authored content pure ASCII |
| touch-set | reviewer's per-commit `git diff --name-only` | exactly the four in-set paths |
| no dual cycle representation is FINAL | reviewer | `detect_cycles` coexists THIS unit by plan decision (sec2.1); its removal is T3.1. Do not add any THIRD representation, and do not wire `detect_cycle_groups` into `generate_report_with_cycle_policies` |

Workspace-wide gates are NOT yours. `python tools/testrun.py --baseline`,
`python -m unittest discover -s tools`, `python tools/ui_smoke.py`, and
`python tools/bench_compare.py` belong to the reviewer's acceptance pass. Note
that `--baseline` is currently BROKEN (ledger deferred row M23.5 F9:
`usage error: missing value for --baseline`) and `ui_smoke` / `bench_compare`
are currently REFUSED by the reviewer allowlist (M23.5 F10) -- neither is your
problem to work around, and you must NOT route a refused command through
`distill.py` to evade the denial.

---

## 9. Reviewer Gate Reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
sec2. The reviewer will run the workspace-wide acceptance pass and will audit:

1. **Lowlink update points (D-A6).** push / advance / pop against the Tarjan invariant, and specifically that emission precedes propagation on pop -- compared against the Phase 1 evidence at `crates/cluster/src/scc.rs:190-198`. This is the #1 audit ask carried over from plan sec4.
2. **IV-5 tie-break determinism (D-A2).** Two runs of the same graph must produce byte-identical group vectors; the Kahn min-heap tie-break must be present and the incomparable-groups fixture must exercise it.
3. **No dual cycle representation.** `detect_cycle_groups` is the only NEW representation; the coexistence with `detect_cycles` is the plan-sanctioned temporary state of sec2.1; nothing else was added; `generate_report_with_cycle_policies` was not rewired.
4. **IV-3 edge cyclicity.** The chord fixture must fail against old `detect_cycles` and pass against `detect_cycle_groups`, with both Red states evidenced in your journal.
5. **IV-7 at 64 KiB.** The 100 000-node chain completing on a 64 KiB thread, with the pre-switch abort signature retained in your journal.
6. **IV-4 witness soundness.** Every witness edge verified against `graph.edges`; `witness.first() == witness.last()`; self-loop yields `[c0, c0]`.
7. **Hygiene and locks.** `cycle_groups.rs` and `graph.rs` line counts; R1/R2 clean; no forbidden adjacency duplicate; `Cargo.toml` unchanged.
8. **Deviation adjudication.** Any deviation you self-flag is adjudicated on its evidence, not accepted because it was declared. Phase 1's three deviations were all accepted, but two of them (a 2 MiB oracle stack that is arithmetically impossible, a gate that selected nothing) were defects the reviewer had to catch. Do not make me catch them.

**Report back (final message):** commit SHAs (recorded in your journal too --
Phase 1's implementer omitted them and the reviewer had to recover them from
`git log`, verdict finding 8); Red evidence for R1 and R2 (the abort signature
and the naming assertion message); Green evidence quoting the canonical line and
capture path for gates 1, 2, and 3; every deviation with its reason; and
out-of-scope observations, report-only. Never paste raw cargo or git output --
cite the command, the canonical line, the exit code, and the capture path. If a
capture path will be overwritten by a later run, say so; do not cite it as
retained evidence.

---

## 10. Discipline Clauses

- **Command execution.** `bash` is an allowlist restricted to the distilling entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED. There is no `tool-caller` in this topology and no delegation for command execution.
- **Reading a refusal.** A refused command ALWAYS reports the catch-all deny rule (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`), whatever the real cause -- including "that command is simply not on the allowlist". Treat that text as boilerplate. Attempt each gate INDEPENDENTLY. Never infer from one refusal that command execution is unavailable, and NEVER drop a gate because its first invocation was refused. If a gate genuinely cannot run, record the VERBATIM refusal in your journal rather than silently narrowing the acceptance set.
- **Strict TDD.** R1, then Green, then R2's Green, then T2.2, then Refactor. Never weaken an assertion to make it pass. Never commit a Red state.
- **Code hygiene.** Functions < 100 lines, files < 1000 lines, no `tmp`/`res`/`foo`/`bar`/`test1`, no single-letter locals in code you author, one responsibility per function, no source line >= 2000 characters (agent Read/grep truncate there and hide content from every participant).
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>`, or `Get-Content | -replace` on a tracked file. Use the Kilo Read/Edit/Write tools, or byte-level `open(path, "wb")` in a script. Authored content is pure ASCII; build any non-ASCII test signature from byte escapes (`b"\xc3\x82\xc2\xa7"`), never a literal. When quoting git content, quote the WHOLE argument: `git show "rev^:path"`.
- **Scope.** Touch only sec2. On discovering out-of-scope breakage: report it, do not fix it.
- **Journal.** Append a 1-3 line checkpoint after EACH Green gate: module, commit SHA, any deviation, any lock or contract finding. Record your own SHAs in it. The journal is what a successor session reads; without it, recovery restores code but loses reasoning.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A dead session is never resumed -- a successor runs `python tools/checkpoint_guard.py resume --spec tasks/m235-t2a-handoff.md --scope ... --cmd ...` (skill `resume-checkpoint`).

---

## 11. Sizing Check (handoff law, stated explicitly)

| Metric | Bound | This unit |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 top-level module directory | `crates/graph` only -- 1 crate |
| Gate commands | ~10 | **7** |
| Produced lines (implementation + test diff) | ~1200 | **~720** (`cycle_groups.rs` ~390, `cycle_groups_test.rs` ~325, `lib.rs` 2, journal ~60) |

**Single handoff unit -- fits, with headroom on all three axes.**

The whole of plan Phase 2 (T2.1 + T2.2 + T2.3) was estimated at ~1030 produced
lines and **10 gate invocations** -- 86% of the output bound and 100% of the
gate bound with zero slack. Phase 1's own handoff estimated ~1200 and produced
745 (over-estimate, lucky); the gate bound is the one that cannot absorb a
single mis-specified invocation, and Phase 1 already lost one to exactly that
(D3). Phase 2 is therefore split at the seam where T2.3 verifies T2.1+T2.2:

- **This unit (T2-A):** T2.1 + T2.2 -- the production algorithm and its own Red suite. Ships independently: additive, workspace green, nothing deleted.
- **`tasks/m235-t2b-handoff.md` (T2-B):** T2.3 -- the equivalence-to-old safety net. Test-only (plus gap fixes), depends on this unit landing.

The seam is correct because T2.3 is not parallel work -- plan T2.3's own Green
is "fixes any gaps found in T2.1/T2.2", i.e. it is the VERIFIER for the
preceding unit and must run against a landed, reviewed T2-A. Between the two
units nothing is half-built: T2-A leaves a compiling, additive, fully tested new
API in the tree.

---

## 12. Authorization

This handoff, together with `tasks/m235-kickoff.md` and the M23.5 milestone row
in `tasks/milestone_ledger.md` (Phase 1 APPROVED, baseline corrected to
`1156 / 0 / 4 / 86`), is the sole authorization for T2.1 and T2.2.

`tasks/m235-t2b-handoff.md` is authorized only after this unit is APPROVED.
Phase 3 (T3.1 report/policy/CLI rewiring, T3.2 UI DTO) and Phase 4 (T4.1
integration closeout) remain UNAUTHORIZED. Do not begin them. Do not delete
`detect_cycles`, do not repoint `graph.rs:356` or `graph.rs:534`, do not
repoint `closure_equivalence_tests.rs:515`, and do not touch
`crates/cli/src/commands/ui.rs:310` or `crates/cli/tests/payload_tail_bench.rs`
-- all of that is Phase 3.