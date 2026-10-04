# M23.5 -- Cycle Substrate: SCC Condensation + Iterative Tarjan

**Status:** PLANNED -> **READY** -- kickoff gate cleared: multi-language merged to main (`f9064f4`, M24 Part A, 2026-10-03).
**Decisions locked:** D1 = B (replacement, no parallel representations) + representative cycle - D2 = Both recursion sites hardened - D3 = standalone milestone on main, post-merge.

> **Re-evaluation 2026-10-03 (post-merge).** The merge grew a cycle-policy seam the plan did not know about (M20 D11): `CyclePolicy { Reject, Tolerate, TolerateWithinPackage }` in `ariadne_lang_api::modeling`, `generate_report_with_cycle_policies(&dyn Fn(FileId) -> CyclePolicy)`, and `IncludeReport::informational_cycle_count` (serde-skipped when 0, C++ artifacts byte-stable). It classifies **cycles** as blocking vs informational; this milestone replaces cycles with groups, so classification migrates to a group-level rule (new IV-11, section 1.3, T3.1). Everything else survived the merge: both recursions live (`graph.rs:234 dfs_cycle`, `cluster/scc.rs:63 strongconnect` -- scc.rs untouched); call-site set for `detect_cycles()` is 7 sites (graph.rs:356 report + :534 unit test, `closure_equivalence_tests.rs:515`, `cli/commands/ui.rs:310`, `payload_tail_bench.rs:295`, Scala integration tests **m20:154** and **m22:442** -- both new, both must migrate). `build_include_elements` is now 5-arg (D27 language/unit-kind node maps). The UI gained the three-theme token system (semantic colors token-driven; single sanctioned hex home `themes.css`). New byte-lock `m00_facts_path_parity_test.rs` + rayon-parallel `plan()` + `plan_serial_for_test()` now guard IV-8. `graph.rs` grew 498 -> 704 lines.

---

## 1. Intent Specification

### 1.1 Intent Statement

Replace the include graph's concrete-cycle path enumeration with a single canonical cycle representation -- **strongly-connected components with condensation metadata** -- and remove every unbounded recursion from cycle detection in the workspace. This is a behavior-refactoring milestone (one intentional semantic change: cycle semantics) that:

1. Fixes the legend-semantics wart: an include edge **inside** an entangled group that `dfs_cycle` never walked as a back-edge currently renders unstyled between two red-glow nodes. Under SCC semantics this is impossible -- every edge within a nontrivial SCC is provably on a cycle.
2. Stops `cycle_count` from exploding combinatorially on dense entangled clusters (it becomes the number of entangled **groups**, not DFS-accidental paths).
3. Eliminates the stack-overflow hazard class: `cluster::scc::strongconnect` (per-header symbol chains) and `graph::dfs_cycle` (repo-wide include chains) are both recursive with unbounded depth.
4. Gives the UI, reports, and the cycle-policy seam one cycle currency: groups, with a cheap witness cycle per group for the inspector/tooltip story.

**The underlying why:** cycles in a dependency graph are a *partition* property (which files are mutually entangled), not a *path* property. The path representation was an approximation of the partition; this milestone adopts the real thing -- the same substrate the planner already uses for extraction decisions at symbol level, the same one `compute_closures` already uses (SCC-condensation DP, plan 1790608181551), and now the natural domain for the M20 cycle-policy classification, which is coherent on partition semantics ("is this entangled group tolerable?") and awkward on DFS-accidental paths ("does every member of *this* arbitrarily-chosen cycle tolerate?").

### 1.2 Invariants (must hold before and after)

- **IV-1 Disjoint partition:** `CycleGroup.member_files` sets are pairwise disjoint; every file in any group is exactly in one group.
- **IV-2 Cycle iff nontrivial:** a file appears in some `CycleGroup` **iff** it lies on at least one include cycle. Group membership iff membership in a nontrivial SCC (size >= 2) or a self-loop include.
- **IV-3 Edge cyclicity:** an include edge `u -> v` is cyclic **iff** `u` and `v` are in the same nontrivial group (or a self-loop). Consequence: no unstyled/amber edge may ever join two nodes carrying the `cycle` class -- Red-testable from the legend contract.
- **IV-4 Representative cycle is a witness:** each group's `representative_cycle` is a closed walk `[c0, c1, ..., ck, c0]` where every consecutive pair is a real include edge within the group; a cycle exists iff a representative exists (nontrivial SCC guaranteed, self-loops yield `[c0, c0]`).
- **IV-5 Determinism:** group order is reverse-topological over the condensation DAG, with a deterministic tie-break (smallest `FileId` of group members ascending); members within a group sorted ascending by `FileId`; export stays byte-stable across processes (canonical `(path asc, file_id asc)` emission contract preserved).
- **IV-6 Parity oracle:** the iterative Tarjan (both sites) produces SCC sets identical to the current recursive implementations on all existing fixtures -- frozen as equivalence tests before the swap, retained afterward.
- **IV-7 No unbounded recursion:** after this milestone, neither detection path grows the OS stack with graph depth.
- **IV-8 Symbol-level behavior frozen:** `cluster` planner output (proposal order, IDs, screening) is byte-identical to pre-milestone on `planner_facts_parity`, `planner_characterization` (renamed post-merge from `planner_characterization_tests`), and the new production byte-lock `m00_facts_path_parity_test` -- the cluster swap is pure refactor. Parallel `plan()` requires the SCC internals keep `Send + Sync` shareability (adjacency is read-only -- preserved by construction).
- **IV-9 `cycle_count` naming survives:** the DTO/manifest field name `cycle_count` is retained (JSON/UI contract), now carrying group count. `m3_include_graph_test`'s `cycle_count > 0` assertion keeps passing unchanged.
- **IV-10 Closure parity untouched:** `compute_closures` SCC-DP and its BFS-oracle equivalence suite (`closure_equivalence_tests`) must stay green; `closure_equivalence_tests.rs:515`'s use of `detect_cycles()` as a cyclicity predicate migrates to `!detect_cycle_groups().is_empty()` (same truth value by IV-2 -- the equivalence suite's own lock catches drift).
- **IV-11 Cycle-policy seam survives and migrates cleanly:** the per-file policy callback `&dyn Fn(FileId) -> CyclePolicy` and the `informational_cycle_count` **field name/serde shape** (skip-when-zero; absent from all C++ artifacts) are preserved. Classification rule migrates verbatim in spirit: **a group is informational iff every member file's policy is non-`Reject`** (the old per-path "all members tolerate" rule on the natural domain -- partition instead of path). The six `informational_cycle_count` unit tests in `graph.rs` (~:614-687) are ported to group semantics; `m20:154`/`m22:442` migrate to `detect_cycle_groups()` with assertions preserved where meaningful.
- **IV-12 Scala paths keep working:** the Scala include/import flows (M20/M22 tests, `IncludeKind::Import` tally) are pure consumers of the graph API; they gain correct group semantics with no plugin-side change.

### 1.3 Contract & API Signatures

**New in `crates/graph`** (replaces `detect_cycles`; `dfs_cycle` private helper deleted):

```rust
/// A maximal entangled group in the include graph: files mutually
/// reachable via includes. Every edge inside a nontrivial group is on a cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleGroup {
    /// Members of the group, ascending by FileId.
    pub member_files: Vec<FileId>,
    /// Closed-walk witness [c0, c1, ..., ck, c0] over edges inside the group;
    /// [c0, c0] for a self-loop group.
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

- `generate_report_with_cycle_policies` classifies **groups**: `cycle_count` = `detect_cycle_groups().len()` (field name kept, IV-9); `informational_cycle_count` = informational **group** count (field name/serde kept, IV-11). `generate_report()` stays the all-`Reject` wrapper (C++ artifact shape unchanged).
- `crates/cli/src/commands/includes.rs:113` label: `"Cycle Groups Detected: {}"`.
- `crates/ui/src/dto.rs::build_include_elements(graph, closures, cycle_groups: &[CycleGroup], file_languages, file_unit_kinds)` -- 5-arg shape preserved; the `cycles` parameter is replaced in place by groups, D27 node maps untouched:
  - node `is_cycle` / class `cycle` iff the node is in some group's `member_files`;
  - edge `is_cycle` / class `cycle` iff both endpoints are in the **same** group;
  - node data gains `"cycle_group_size": usize | null` and `"cycle_witness": [path labels] | null` for the inspector story; any styling of these stays token-driven (zero new hex literals, `tools/theme_token_check.py` gate).
- **New in `crates/cluster/src/scc.rs`:** `compute_symbol_sccs_from_adj` keeps its exact public signature and output (IV-8); `strongconnect` recursion is replaced by an explicit-stack frame machine. New stress test asserts identical output between old (frozen as test-local oracle) and new on fixtures + seeded random digraphs + a 100 000-node chain.

**Explicitly out of scope:** UI visual redesign (class contract unchanged, so stylesheet unaffected beyond new optional data fields); renaming `CyclePolicy` variants or its registry-side derivation (ADR `34e21ea`: planner field write-only, report-side registry-derived -- untouched here); Scala cycle semantics beyond graph API consumption; big-stack-thread fallback (`stacker` dependency rejected -- no tail-recursive Tarjan exists, TCO not guaranteed in Rust).

### 1.4 Hygiene Architecture Constraints

- All functions < 100 lines (the iterative Tarjan frame machine decomposes: `run_tarjan`, `advance_frame`, `finish_child`, `emit_group_if_root`, `representative_cycle_for` -- none may swallow into a monolith).
- All files < 1000 lines. `graph.rs` is at **704** post-merge; expected new code (Tarjan machine + witness + classification port + migrated unit tests) lands it near the ceiling -> **extract `crates/graph/src/cycle_groups.rs` proactively**, following the `closure_equivalence_tests` `#[path]` mod precedent. The merge already burned the repo on a >1000 R2 casualty (`static_html.rs` 1010) -- do not repeat it.
- Predicate booleans and revealing names mandatory: `member_files`, `representative_cycle`, `external_out_count`, `on_stack`, `child_completed`, `group_is_informational` -- no `tmp`/`res`/`scc_tmp`.
- New equivalence suites follow the existing `closure_equivalence_tests.rs` pattern (oracle module + `#[path]` mod pin, seeded random digraphs).

---

## 2. RGB-TDD Task Breakdown

**Phase order:** Phase 1 (cluster swap) is self-contained and independently shippable; Phases 2-4 are the D1-B replacement. Within a phase all tasks are parallel-safe; across phases, respect dependencies noted.

### Phase 1 -- Symbol SCC de-recursion (pure refactor, crate: cluster)

**T1.1 Freeze the parity oracle.**
- **Red:** none new to production -- add `crates/cluster/tests/scc_iterative_parity.rs` containing a *test-local copy* of the current recursive `strongconnect` (`recursive_tarjan_oracle`) plus equivalence assertions for a not-yet-iterative `compute_symbol_sccs_from_adj`. Fixtures: (a) every existing unit fixture, (b) 200 seeded random digraphs (seeded LCG, documented seed), (c) a 100 000-node chain `s0 -> s1 -> ... -> s99999` plus back-edge variant `s99999 -> s0` (one giant SCC). The chain case runs on a thread with the **default small stack** (64 KiB): it crashes/fails today (Red demonstrates the overflow class), passes once iterative (Green) with output equal to the oracle run on a 2 MiB thread.
- **Green:** T1.2.
- **Refactor/hygiene:** oracle stays test-only; production `scc.rs` must not carry a `#[cfg(test)]` copy of itself.

**T1.2 Iterative `strongconnect` frame machine.**
- **Red:** T1.1 suite is red.
- **Green:** replace recursion with explicit `Vec<TarjanFrame { symbol_ptr, neighbor_position, ... }>`; lowlink propagation happens on **frame pop**, not on call return -- the classic trap this design structurally avoids. Output contract identical (IV-8): same order, same intra-SCC sort, wrapper `compute_symbol_sccs` unchanged, frame state `Send + Sync`-compatible for the rayon planner (adjacency remains read-only).
- **Refactor:** one predicate helper per frame-machine step, each < 100 lines; keep `build_symbol_adjacency` untouched.
- **Review gate:** Reviewer audits lowlink update points specifically (push/advance/pop) against the Tarjan invariant; then `cargo test -p ariadne-cluster` + `planner_characterization` + `planner_facts_parity` + **`m00_facts_path_parity_test`** (production byte-lock) all green.

### Phase 2 -- `CycleGroup` core in crate:graph

**T2.1 `CycleGroup` type + iterative graph Tarjan (groups only).**
- **Red:** new `crates/graph/tests/cycle_groups_test.rs`: two-cycle fixture (mirrors `m3` and graph.rs's own unit fixture) -> exactly one group `{a,b}`, witness length 3 closing `[a,b,a]`, `external_in/out` counts; diamond-with-chord fixture (a->b->c->a, a->d<-c) -> one 3-member group, `d` in no group, **every** intra-group edge cyclic (IV-3 -- the unstyled-edge-between-reds regression lock); self-include `a->a` -> group `{a}`, witness `[a,a]`; two disjoint groups + DAG connector -> reverse-topological group order (IV-5); empty/acyclic graph -> empty vec.
- **Green:** `detect_cycle_groups` in **new `crates/graph/src/cycle_groups.rs`** (proactive extraction, section 1.4) -- iterative Tarjan (share the frame-machine *shape* from T1.2; do **not** add a cross-crate dependency -- graph and cluster stay decoupled, ~60-line duplication accepted, noted for reviewer); filter nontrivial/self-loop SCCs; compute external degrees from adjacency crossing counts.
- **Refactor (AMENDED 2026-10-04 against the built tree at `b950963`):** `graph.rs` is at **704 lines and stayed at 704 through all of Phase 2** (T2-A and T2-B reviewers both measured it unchanged; the new machine lives in the extracted `crates/graph/src/cycle_groups.rs`, 527 lines). `graph.rs` does NOT return toward ~550 in T2.1. The `dfs_cycle`/`detect_cycles` deletion belongs to **Phase 3 (T3.1)**, not T2.1, and lands in the Phase 3 unit that owns `crates/graph` -- `tasks/m235-t3d-handoff.md` -- where it is atomic with its own last repoint: three of the seven call sites live OUTSIDE `crate:graph` (`crates/cli/src/commands/ui.rs:310`, `crates/cli/tests/payload_tail_bench.rs:295`, and the two Scala integration tests), so no `crate:graph`-scoped unit may delete the method without breaking the workspace build. The proactive `cycle_groups.rs` extraction (above) is what discharges the 1000-line ceiling pressure (plan sec1.4); the deletion is not.

**T2.2 Representative-cycle witness (closed walk per group).**
- **Red:** seeded random digraphs suite (pattern copied from `closure_equivalence_tests`): every returned witness verified edge-by-edge against `graph.edges` (IV-4); a group with a *known* 2-cycle embedded in a 200-node random tangle must return a witness (existence, not exactness -- exactness is nondeterministic-by-design under DFS choice).
- **Green:** iterative within-group closed-walk finder: path-tracking DFS restricted to group members (structurally the old `current_path`/`rec_stack` walk but bounded to the group and iterative), extract `[pos..] + [first]` on first back-edge; self-loop special case.
- **Refactor:** the deleted `dfs_cycle` semantics survive only here, test-scoped.

**T2.3 Equivalence-to-old suite (migration safety net).**
- **Red:** test-local recursive oracle of the old `detect_cycles` path enumeration; for each seeded digraph: assert (old union new) node sets equal -- a file is on an old path iff it is in some new group (IV-2 both directions); assert each old reported cycle's members are a subset of a single group (old paths never straddle groups).
- **Green:** fixes any gaps found in T2.1/T2.2 before downstream rewiring.

### Phase 3 -- Replacement rewiring (D1-B, no dual representation)

**T3.1 Report + policy seam + CLI + consumers.**
- Red->Green: `cycle_count` = group count; **policy classification migrates to group level** -- port the six `informational_cycle_count` unit tests (graph.rs ~:614-687, incl. the serde skip-when-zero locks) to group semantics with identical stances (`Reject` -> 0 informational + key absent; all-tolerate -> equals group count); `generate_report()` all-`Reject` wrapper unchanged (C++ artifacts byte-stable). Delete `detect_cycles`/`dfs_cycle`; repoint graph.rs:356/:534 and `closure_equivalence_tests.rs:515`.
- Call-site migrations: `cli/commands/includes.rs:113` label -> "Cycle Groups Detected"; **`m20_scala_dependency_graph_test.rs:154`** and **`m22_scala_load_analysis_test.rs:442`** -> `detect_cycle_groups()` (IV-12; m22's `informational_cycle_count == cycle_count` under all-tolerate now means *groups* -- preserve the asserted stance, re-pin the arithmetic).
- **Goldens:** re-pin any golden containing concrete cycle lists; write LF terminators (`tests/golden/**` is LF-pinned via `.gitattributes` post-merge). `m3_include_graph_test` passes **without edit** (IV-9 proof). `m23_symbol_isolation_test`'s `generate_report_with_cycle_policies` call stays compiling (signature untouched).
- **IV-10 note:** the closure-equivalence suite is the independent witness that the migration did not change cyclicity truth values.

**T3.2 UI DTO + payload.**
- Red: `dto_builders`/`graph_api`/`export_order`/`justification_*` suites extended -- intra-group non-tree edge now carries `cycle` class (the old bug left it unstyled); byte-stable export test across two processes with cycle fixtures present; `cycle_group_size`/`cycle_witness` node data present for grouped nodes, absent/null otherwise; D27 language/unit-kind map behavior untouched (missing entry still OMITS the key).
- Green: `build_include_elements` 3rd parameter becomes `&[CycleGroup]` (5-arg shape kept); update `cli/src/commands/ui.rs:310` (keep `node_maps::build` + 5-arg call) and the `:103` stage-5 timing comment; `payload_tail_bench.rs` (column label -> `detect_cycle_groups`; verify combined timing vs the 4.7 ms `generate_report+detect_cycles` baseline -- SCC is linear, expect parity; record the bench number for the reviewer).
- **Reviewer check:** theme-token contract -- zero new hex literals in any touched UI file (`tools/theme_token_check.py`); semantic verdict colors unchanged across all three themes (steampunk palette lock: green/amber/red semantics + heat ramp preserved; only *coverage* of red edges grows correctly).

### Phase 4 -- Integration closeout

**T4.1 Full-workspace verification + hygiene.**
- `python tools/testrun.py --distill` -> quote the canonical `N passed / M failed / K ignored / S suites` line verbatim (compare against the `1147 / 0 / 4 / 85` post-merge baseline; expect net-positive from new suites); golden suite byte-clean (LF pins intact); `python tools/hygiene_check.py` (R1/R2 blocking -- watch the 1000-line ceiling), `python tools/encoding_guard.py --all`; export determinism re-check on the SCC-dense 520-header bench fixture; `payload_tail_bench` timings recorded.
- Commit this plan file alongside the code (repo.tracks_plan_files).
- **Reviewer handoff:** diffs, T1.1/T2.3 oracle suites, bench numbers, goldens diff, m20/m22 migration diffs. Focus asks: lowlink update points, tie-break determinism, absence of any retained dual cycle representation, policy-seam stance preservation (IV-11).

---

## 3. Acceptance Criteria

1. `detect_cycles` no longer exists; `CycleGroup` is the workspace's only cycle representation.
2. `cycle_count` reports entangled-group count; `m3_include_graph_test` green without edit.
3. No unstyled/amber edge connects two `cycle`-classed nodes in any fixture or golden (IV-3).
4. Deep-chain stress (100 000-node) passes at default stack in both crates; equivalence oracles green on fixtures + 200 seeded digraphs each.
5. Planner output byte-identical (IV-8) -- `planner_facts_parity`, `planner_characterization`, **`m00_facts_path_parity_test`** all green unedited; closure equivalence suite green (IV-10); export byte-stable.
6. Cycle-policy seam preserved (IV-11): callback signature, `informational_cycle_count` name + skip-when-zero serde shape, all-`Reject` C++ artifact byte-stability; m20/m22 migrated with stance preserved.
7. No new hex literals in UI files; semantic verdict colors intact across light/dark/dark-steampunk.
8. Workspace green + hygiene gate + encoding guard + reviewer APPROVED.

## 4. Risks & Mitigations

- **Iterative-Tarjan lowlink bug** -> frozen recursive oracles before swap (T1.1, T2.3); reviewer audits update points.
- **Policy-seam stance drift** (cycle->group classification is the one semantic port) -> ported six unit tests + serde-shape locks in T3.1; m22 Scala tolerance flow as a real-input witness.
- **Export golden churn** -> T3.1 isolates re-pinning; LF pins respected; determinism test guards.
- **Merge skew vs multi-language branch -> RESOLVED 2026-10-03:** branch merged (`f9064f4`); plan re-pinned to post-merge tree above. Residual: if further main churn touches graph.rs cycle code before kickoff, re-run this re-pin.
- **Graph/cluster algorithm duplication** (deliberate, no cross-crate dep) -> reviewer signs off or directs a shared internal crate.
- **1000-line ceiling on graph.rs** -> proactive `cycle_groups.rs` extraction (T2.1), not a late R2 casualty.
