# AR-09 Journal - Adjacency-driven scans (P3 HIGH + P4 MED-HIGH)

Date: 2026-10-10. Owner: implementer (sole owner of `crates/graph` at handoff time).
Handoff: `tasks/ar-09-handoff.md`. Depends on AR-06 (O(1) `add_edge`,
`seen_edges` COW) and AR-07 (both APPROVED, landed).

## What was done

### 1. `IncludeGraph::direct_includes` (graph.rs) - cached adjacency index

Decision from the handoff's two options: the graph-internal LAZY CACHE, not a
per-call build. The arithmetic: the call sites are all per-visited-node loops
(`format_explain_node` recursion in graph.rs, `cluster::frontier.rs` BFS,
`load_explain.rs` scan loop, four UI server routes). A per-call build would
make every single call O(E) to build the map to serve one lookup - strictly
worse than the retired O(E) scan for isolated calls, and no better for loops.
The cache makes the first query of a generation pay one O(V+E+sort) build and
every later lookup O(deg) clone. Per query sweep over k visited nodes:
before O(k*E), after O(E + k*deg) amortized (O(1) map probe + neighbor clone).
On the handoff's 1k-node/20k-edge example a deep click drops ~2*10^7
comparisons to ~10^5 map/clone work.

Implementation:
- `AdjacencyCache` slot { `validated_edge_count`, `map: Option<Arc<HashMap>>` },
  stored as `#[serde(skip)] adjacency_cache: Mutex<AdjacencyCache>` on
  `IncludeGraph`; built by the sanctioned `IncludeGraph::adjacency()` builder
  (no loop duplication - the builder's doc now names this as its third
  legitimate call site alongside `compute_closures` and the LoadMemo memo).
- Invalidation design (the handoff flagged staleness as a correctness bug):
  the map is VALIDATED against `edges.len()` on every access rather than
  invalidated from mutators. Rationale found during the survey: ui tests
  mutate `graph.edges` by direct `push`, bypassing `add_edge`, so an
  add_edge-hook alone would serve stale adjacency in exactly those places.
  Every include-graph edge mutation in the workspace is append-only (grep
  found no remove/truncate/assign), and an append always changes the count,
  so the count check catches every actual mutation surface. `register_file`
  is deliberately NOT a trigger: `file_paths` is not an input to adjacency
  (this answers the handoff's "invalidation on add_edge/register_file"
  question for the register_file half). A hypothetical length-restoring
  direct edit would already violate the AR-06 `seen_edges` invariant, so the
  cache inherits exactly the mutation contract AR-06 established - nothing
  stronger is expressible while `edges` stays `pub`.
- The `lock().unwrap()` is held only across the validation and the pointer
  swap; the shared map is immutable behind its `Arc`, mirroring the
  `load_caches.rs` lock discipline (poison = prior panic under the same lock,
  run already aborting).
- Invalidation complexity was therefore trivial (a count compare), which is
  what made the cached variant safe; the handoff's fallback reasoning ("if
  invalidation complexity is nontrivial, choose per-call and justify the
  arithmetic") is recorded above as the per-call rejection for THIS call site
  (per-call build is arithmetic-negative for `direct_includes`).

### 2. `traversal.rs` - `IncludeGraph::traverse_neighborhood` + `shortest_include_path`

Both BFSs rescanned `self.edges` for every dequeued node (O(V_visited * E)
per call). Both now consume ONE per-call `DirectionalAdjacency` build (O(E)
per call, out/in halves filled only as the direction demands - the
`Both`/`Incoming` semantics match on destination exactly as the retired scan
did, via the reverse `incoming` map).
- `traverse_neighborhood`: before O(V_visited * E) scan + O(E) induced-edge
  pass; after O(E) build + O(V_visited + deg) BFS + the same unchanged O(E)
  induced-edge pass. Output is set-based and sorted, so neighbor order is
  provably unobservable; the equivalence suite confirms it exhaustively.
- `shortest_include_path`: same asymptotics, but order IS observable - the
  `parent` map is set by first arrival, so iterating a SORTED neighbor list
  would re-parent equal-distance targets and change the returned path. The
  per-call builder therefore preserves edge-list order and duplicates
  (deliberately NOT `adjacency()`, whose sort+dedup is the wrong order for
  path reconstruction; the tie fixture `1 -> {5,3} -> 7` with 5 inserted
  first is locked byte-for-byte against a sorted-adjacency rewrite).
- One shared `DirectionalAdjacency::from_edges` + one shared expansion helper
  serve the include-graph and call-graph neighborhood BFSs (single loop
  implementation, two edge-shape adapters).
- `load_caches.rs` memo NOT reused (handoff guidance): it is generation-scoped
  and memo-carried; graph methods must not require a `LoadMemo`. The memo
  keeps its own OnceLock view; both builders are pure functions of `edges`.

### 3. `callgraph.rs` + CallGraph neighborhood in traversal.rs

- `CallGraph::traverse_neighborhood` (traversal.rs): same directional
  adjacency treatment, O(V_visited * E) -> O(E + V_visited + deg) per call.
- `CallGraph::shortest_call_path`: retired code paid an O(E) `callees_of`
  scan plus a per-node neighbor sort; now ONE pass groups callees by caller
  (borrowed keys, no clones) and sorts each caller's list once. Neighbor
  sequence is identical to the retired per-node sort (including duplicates
  from literal-built graphs, suppressed by `visited` as before), so paths are
  byte-identical.
- CallGraph gets no cache field: it has no mutator API and every route hits
  these methods once per request; per-call build already wins the arithmetic.

## Complexity table (per call)

| site | before | after |
|---|---|---|
| `direct_includes` | O(E) | O(1) probe + O(deg) clone; one O(V+E) build per generation |
| a k-node sweep of the above (frontier/explain/UI loops) | O(k*E) | O(E + k*deg) once per generation |
| `traverse_neighborhood` (include) | O(V_visited*E) | O(E) build + O(V_visited + deg) |
| `shortest_include_path` | O(V_visited*E) | O(E) build + O(V_visited + deg) |
| `CallGraph::traverse_neighborhood` | O(V_visited*E) | O(E) build + O(V_visited + deg) |
| `shortest_call_path` | O(V_visited*E + V_visited*deg*log) | O(E + sum deg*log deg) once + O(V_visited + deg) |

## TDD record (red-by-characterization per handoff)

New `crates/graph/tests/ar09_adjacency_equivalence_test.rs` (13 tests):
frozen verbatim copies of every current scan as oracles, cross-checked
exhaustively (5 include fixtures x 7 probe ids x depths 0..4 x 3 directions;
all-pairs shortest paths; callgraph fixture x 5 centers x depths x directions;
all-pairs call paths), plus hand-derived snapshot literals for node order,
edge order and paths, plus cache-staleness guards (`add_edge` after a query,
clone cross-mutation, serialized shape).
Fixtures: chain, diamond (with the AR-06 distinct-metadata twin edge), cycle,
disconnected (island + isolated file + unresolved edge + self-loop), tie
(edge-insertion-order path tie), callgraph literal (unsorted edges, cycle,
self-call, in-only node D, isolated center).

- Pre-change run: 2 of my hand-derived literals were wrong, not the code
  (Both propagates through an "incoming-only" node; induced edge sets carry
  any edge with BOTH endpoints visited, e.g. (2,3) at depth 1 via 1->{2,3}).
  Corrected the literals to the current behavior - characterization
  discipline: the current scan defines the contract. Oracle cross-checks
  confirmed the corrections were behavior, not drift.
- Baseline digest (pre-change, locks green):
  `13 passed / 0 failed / 0 ignored / 1 suites`
- Package baseline before any change:
  `116 passed / 0 failed / 0 ignored / 5 suites`
- After all three scan changes: locks unmoved.
  `129 passed / 0 failed / 0 ignored / 6 suites`

## Deliberately NOT done

- `ReducedSemanticModel` traversal/shortest scans in traversal.rs: out of
  scope per handoff ("do NOT touch reduced_model scans ... unless trivially
  shared"); not shared (symbol edge shape differs) and untouched.
- `callers_of`/`callees_of` and `get_entrypoints`: single O(E) passes per
  call, not per-node scans; no win to take, scope discipline kept.
- `load_caches.rs` memo reuse for graph methods: kept the graph
  self-contained per handoff note.
- `compute_closures`/`detect_cycles`: already single-adjacency-build O(V+E)
  consumers of the sanctioned builder; left as-is.
- Golden fixtures / m00: unedited (gate run green).

## Deviations

- Hand-written `impl Clone for IncludeGraph` replacing the derived one:
  `std::sync::Mutex` is not `Clone`, so the derive cannot coexist with the
  cache field. The hand-written impl is value-identical to the former derive
  for all pre-existing fields (refcount bumps keep `seen_edges` COW and the
  test tally shared) and gives each clone its OWN validation slot while
  sharing the immutable `Arc` map - so a post-clone `add_edge` cannot
  invalidate the parent's cache (or vice versa) beyond its own count check.
  Two pre-existing field comments updated from "existing Clone derive" to
  "hand-written Clone" (comment-only).
- `adjacency()` doc updated to name the cache as its third build call site;
  the "no other code may duplicate this loop" prohibition stands (the
  traversal builder is an order-PRESERVING loop with different semantics,
  documented why it cannot reuse the sorted map).

## Gates (verbatim digests)

1. `python tools/testrun.py --distill -- -p ariadne-graph`
   `129 passed / 0 failed / 0 ignored / 6 suites` (prior 116 + 13 locks)
2. `python tools/distill.py -- python tools/bench_compare.py` (median-of-3):
   `bench compare: 3 run(s), no regressions past 1.25x baseline`
   Truth rows (median ms):
   symbol_pass+prep 1782.4 | file_pass 674.1 | save_load_artifacts+cache
   2446.2 | ensure_load total 4913.3 | include_graph clone (cold) 1.6 |
   load hand-out (cold) 0.0 | include_graph clone (warm) 1.2 | load hand-out
   (warm) 0.0 | generate_report+detect_cycles 78.3 | build_include_elements
   60.0 | path_to_id+file_collateral maps 178.3 | OptimizationPlanner::plan()
   921.8 | conservative filter pass 0.0 | build_file_justification_with_memo
   8786.7.
   The AR-06 clone rows are 1.6/1.2 ms with a clean "no regressions" verdict
   (not borderline), so the quieter-rerun clause was not triggered; single
   run recorded. Capture: `target/distill-capture.txt`.
3. `python tools/encoding_guard.py --all` - silent pass.
4. `python tools/hygiene_check.py` - R1/R2 rows only at the known baseline:
   server.rs file length + `api_graph_traversal`; the m15_graph_ui_test.rs
   R1 row is pre-existing HEAD state (file untouched this task, confirmed by
   `git status`). Zero NEW blocking rows; the two R3 (warn) rows my test file
   briefly added (`c`, `d` locals) were renamed away and are gone.
5. `python tools/testrun.py --distill -- --test m00_golden_baseline_test`
   `2 passed / 0 failed / 0 ignored / 1 suites` - m00/golden UNEDITED green.
   Plus workspace-wide `cargo check --workspace --all-targets` exit 0
   (pre-existing warnings only, none from crates/graph).
   Capture: `target/testrun-capture.txt` / `target/distill-capture.txt`.
