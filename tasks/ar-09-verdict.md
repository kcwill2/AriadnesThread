# AR-09 Verdict - Adjacency-driven scans (P3/P4)

Commit judged: `14abc55` (perf(graph): adjacency-driven direct_includes/traversal/callgraph scans).
Handoff: `tasks/ar-09-handoff.md`. Journal: `tasks/ar-09-handoff-journal.md`.
Reviewer session: 2026-10-10, worktree `shining-enthusiasm`, HEAD `55c93c8`.

## VERDICT: APPROVED

No blockers. Two minor findings (F1, F2) are attributed follow-ups, not
merge conditions for this commit. One environment note (F3).

## Gate commands vs commit (touch-set)

`git diff --name-only 14abc55^..14abc55` -> exactly five paths:

- crates/graph/src/graph.rs          (in touch-set)
- crates/graph/src/traversal.rs      (in touch-set)
- crates/graph/src/callgraph.rs      (in touch-set)
- crates/graph/tests/ar09_adjacency_equivalence_test.rs (in touch-set: tests/**)
- tasks/ar-09-handoff-journal.md     (in touch-set)

`load_caches.rs` untouched (touch-set permits it only for memo reuse; memo was
not reused, as declared and justified). PASS - every commit path inside the
declared hard touch-set. No untracked/dirty file is charged to this commit
(the dirty set is `crates/ui` AR-11 WIP + untracked `tasks/` documents).

## Hazard 1 - stale-cache soundness: SOUND

Claim audited: "all real mutations are append-only". Whole-workspace sweeps
(`*.rs`) for non-append mutation of `IncludeGraph.edges`:

- `.edges.(retain|remove|truncate|clear|sort|sort_by|swap|drain|pop|insert|
  extend|append|splice|dedup|reverse|iter_mut|as_mut)`, index assignment
  `edges[..] =`, direct assignment `edges =`, `&mut ... edges` passing.
- Every `.edges = vec![...]` hit is `ReducedSemanticModel.edges`
  (`Vec<ReducedSemanticEdge>`, no cache field) - type-verified at each site:
  crates/graph/tests/load_memo_capacity_invariance_test.rs:126/157/210/273/345,
  crates/graph/src/load_perf_tests.rs:90/150/341,
  crates/graph/src/load_justification_tests.rs:89/343/393/556,
  crates/graph/src/load_impact_tests.rs:128. None is `IncludeGraph`.
- `.edges.append(&mut ...)` hit (scheduler/tu_store.rs:214) is
  `FileAnalysis.edges` into a local combined vec; not `IncludeGraph`.
- No `&mut` borrow of `IncludeGraph.edges` anywhere; no `iter_mut`, no index
  writes, no `sort`, no clear/truncate.

Production mutation surface = `IncludeGraph::add_edge` (graph.rs:127, the only
`self.edges.push` in src) and `load_include_checked`
(crates/report/src/manager/graph_io.rs:100-104: fresh `IncludeGraph::new()`,
then `register_file`/edge loop - a never-built cache slot, so no staleness
window exists at load). UI test direct `graph.edges.push(...)` sites
(graph_api.rs:485, common.rs:280-281, export_order.rs:51-53,
server_api.rs:373-381, justification_golden.rs:130-154,
justification_builders.rs:408/567-573) are all appends; each changes
`edges.len()`, and `cached_adjacency` evaluates
`cache.validated_edge_count != self.edges.len()` against the live Vec on
every access (graph.rs), so every append trips validation mechanically.
`register_file`/`file_paths` mutation is not an adjacency input - correctly
not an invalidation trigger. `Clone` copies the slot into a FRESH Mutex with
the parent's validated count, so clone/parent appends cannot stale each other
(locked by `clone_shares_data_without_cross_mutation`).

Caveat -> finding F2: no test exercises query-then-direct-push ordering
(all fixture pushes precede the first query), so direct-push staleness is
guarded by mechanism + reasoning, not by a runtime lock. The `add_edge`
staleness lock (`results_refresh_after_add_edge`) is behaviorally equivalent
for the count check (both are length-changing appends).

## Hazard 2 - shortest-path parent order: SOUND

- `DirectionalAdjacency::from_edges` preserves edge-insertion order and keeps
  duplicates (no sort, no dedup); `shortest_include_path` builds it
  Outgoing-only from the `filter_map(destination)` stream, whose order equals
  the retired per-edge scan restricted to `edge.source == curr`. First-arrival
  `parent` setting is therefore unchanged, byte-for-byte.
- Tie fixture `1 -> {5,3} -> 7` (5 inserted before 3) genuinely forces the
  divergence. Independently re-derived both ways: retired scan dequeues 1,
  visits 5 then 3 (edge order), dequeues 5 first, `parent[7] = 5` ->
  `[1,5,7]` (the locked literal). A sorted-adjacency rewrite would visit 3
  first and return `[1,3,7]` - the lock fails loudly. Confirmed: the fixture
  is a regression trap, not decoration.
- `traverse_neighborhood` (include + callgraph): intra-layer visit order can
  differ (old interleaved one neighbor per edge; new drains out-half then
  in-half), but BFS distance labels and the visited set are neighbor-order
  invariant, and output is a sorted node list + sorted induced-edge set, so
  nothing observable can move. Confirmed exhaustively by the oracle
  cross-check (5 fixtures x 7 probes x depths 0..4 x 3 directions, all-pairs
  shortest paths; callgraph 5 centers x depths x directions, all-pairs).
- `shortest_call_path`: old code sorted per-node callee lists; new code sorts
  each caller's grouped list once. `Vec<&SymbolId>` sort == `Vec<SymbolId>`
  sort; duplicates retained and suppressed by `visited` identically. Old
  `callees_of` verified (callgraph.rs:187-189) to be a pure caller filter -
  the oracle's inline filter is semantically identical.
- `Both`-direction self-loop: old scan overwrote a single `next_node` slot
  (one visit attempt); new visits twice from out/in halves, `visited` set
  absorbs the duplicate. Same membership, same queue. Confirmed against the
  disconnected/cycle fixture locks.

## Frozen oracle verification (oracle-before-swap)

The five `oracle_*` functions were diffed against the pre-change blob
(`git show "14abc55^:..."` read whole) - they are verbatim copies of the
retired scans (only the `callees_of` inlining, proven identical above). The
equivalence proof is therefore self-contained and does not depend on trusting
the journal's pre-change run order. Journal red-phase record: package baseline
`116 passed / 0 failed / 0 ignored / 5 suites`, then two self-detected literal
errors corrected, then locks green pre-change `13 passed / 0 failed / 0
ignored / 1 suites`, then `129 passed / 0 failed / 0 ignored / 6 suites`.

## Corrected-literal adjudication (characterization, not wish)

Both corrected expectations independently re-derived from the retired scan's
semantics; both match the committed literals -> corrections were to observed
behavior, contract preserved:

1. "Both propagates through incoming-only node" (callgraph fixture, center D,
   depth 4): retired scan from D expands incoming (B,D) -> B, then from B
   reaches A (via (A,B) incoming match) and C; final visited {A,B,C,D}, induced
   edge set = all five edges (every edge has both endpoints visited).
   Committed literal: nodes [A,B,C,D], `hood.edges.len() == 5`. Matches.
2. "Induced edge set carries (2,3) at depth 1 via 1 -> {2,3}"
   (`results_refresh_after_add_edge`): BFS from 1, depth 1 visits {1,2,3}
   (2 and 3 are depth-1, never expanded); induced pass collects any edge with
   both endpoints visited -> (1,2),(1,3),(2,3). Committed literal matches.

Spot-derived additionally (all match committed literals): tie path [1,5,7];
diamond shortest [1,2,4]; cycle shortest 3->2 = [3,1,2]; cycle hood depth-1
Both nodes [1,2,3] with all three edges; disconnected depth-0 induced {(2,2)};
disconnected Incoming depth-1 nodes [1,2] edges [(1,2),(2,2)].

## Deviation dispositions

- D1 hand-written `impl Clone for IncludeGraph`: ACCEPTED. Forced by
  `Mutex: !Clone`. Field-by-field vs former derive: `edges` Vec clone,
  `file_paths` map clone, `#[cfg(test)] find_file_id_scans` `Arc::clone`
  (derive-shared the tally; hand-written shares it too), `seen_edges`
  `Arc::clone` (AR-06 COW refcount-sharing preserved - `add_edge` still
  forks via `Arc::make_mut`), plus the new cache slot in a fresh Mutex.
  All five prior+new fields mapped; value-identical for pre-existing fields.
  Comment-only edits to the two field docs are honest ("hand-written Clone").
- D2 `adjacency()` doc names the cache as third build call site: ACCEPTED.
  The "no other code may duplicate this loop" prohibition still stands for
  the sorted builder; `DirectionalAdjacency::from_edges` is a genuinely
  different loop (order-preserving, directional, duplicate-keeping) whose
  non-reuse is documented at both ends. Not a retained dual representation:
  the retired edge-list scans are deleted, and the two surviving builders
  implement two different required orders (sorted for closures, insertion
  order for path parents) - collapsing them would break path byte-identity.
- D3 graph-internal lazy cache chosen over per-call build for
  `direct_includes`, per-call for the two BFS sites, no `LoadMemo` reuse:
  ACCEPTED - all three options were handoff-sanctioned and the arithmetic is
  recorded.
- D4 deliberately-not-done list (ReducedSemanticModel scans, callers_of/
  callees_of, get_entrypoints, compute_closures, load_caches memo): ACCEPTED
  as scope discipline; each justified in the journal.

No undeclared deviations found.

## serde / locks

- `#[serde(skip)] adjacency_cache`; `serialized_shape_excludes_adjacency_
  cache` asserts keys == ["edges","file_paths"] after warming the cache -
  green in the 129. m00 goldens UNEDITED verified twice: commit diff carries
  no golden path; working tree dirty set carries none.
- m00 parity run green (see execution results, gate 2).

## Report-evidence (distillation contract)

Journal is digest-shaped: canonical testrun lines, bench truth rows, capture
paths (`target/testrun-capture.txt`, `target/distill-capture.txt`), exit
semantics; gate commands use the distilling entry points; zero raw dumps.
PASS.

## Findings

1. MINOR (attributed, not blocking this commit): `bench_compare.py` exited 1
   both runs - warm clone row reproducible at 1.3 ms vs baseline 1.0 (+30%,
   +0.3 ms), cold clone unstable (1.8 ms run 1 -> 1.4 ms run 2). Every
   stage/build row clean and faster than the implementer's own recorded run
   (ensure_load 4501.5 vs 4913.3; justification 8122.0 vs 8786.7; planner
   825.2 vs 921.8). No mechanism exists for AR-09 to slow clone by 0.3 ms
   (hand-written Clone adds one uncontended mutex acquire + 2-word slot copy;
   the map is Arc-shared, never copied). Attribution: sub-ms quantization on
   a documented-borderline row (1.20-1.23x history) plus concurrent AR-11
   build/edit load. Per the gate instruction this is report-and-attribute,
   not a new regression class. Recommend a quiet-conditions milestone-gate
   rerun (orchestrator) to settle the warm row.
2. MINOR (lock-coverage): staleness is runtime-locked only for `add_edge`;
   no test warms the cache and then direct-pushes `graph.edges` (UI fixtures
   push before any query). Mechanism sound by construction; a follow-up test
   in a future graph handoff should warm-then-direct-push. Noted for
   `tasks/milestone_ledger.md` deferred-findings, not a required change here.
3. ENVIRONMENT (configuration, not a code finding): several read-only probe
   invocations were refused by the catch-all bash deny pattern even though
   read-only git and the gate entry points themselves are permitted. Refused
   (verbatim): "The user has specified a rule which prevents you from using
   this specific tool call. Here are some of the relevant rules
   {\"permission\":\"bash\",\"pattern\":\"*\",\"action\":\"deny\",
   \"source\":\"global\"}" - for (a) `git show ... | Select-String`,
   (b) `git cat-file` via `& {}` pipeline, (c) `cmd; echo "exit: $LASTEXITCODE"`
   chained forms, and (d) first attempt at the unittest discover with a
   `Select-Object` pipe. All five expected gates were then run successfully
   as plain single commands (see execution results). No expected gate was
   skipped; the refusals shaped the command forms, not the coverage.

## Follow-up dispositions

- F1 warm-clone quiet rerun -> milestone closeout gate (orchestrator's run).
- F2 direct-push staleness lock -> ledger deferred-findings / next graph
  handoff's test list.
