# Load Perf, Second Pass — Memo Reuse, Adjacency Lockless Reads, Artifact Tail

## Intent Statement

The first perf pass (commits 261ee41..ef6b7a0, 69% stage-total reduction) optimized the pipeline's `ensure_load` stage. Measurement of what remains shows the center of gravity has moved: the biggest costs are now (a) the **UI justification paths re-running cold analysis** — the live `/api/justification` route constructs a fresh `LoadAnalyzer` per request, so every click pays a full `compute_closures()` (O(V·(V+E))) plus cold-memo BFS work, (b) **`override_closure` rebuilding the include-edge adjacency map from the full edge list on every call** (twice per `extraction_impact`, i.e. per preview entry), (c) **`find_file_id` O(V) string scans** on the file pass, (d) the **serial artifact tail** that serializes the whole `LoadReport` twice, and (e) **~4 `SymbolId` (String) clones per element** inside `compute_load`'s per-metric loops.

This plan removes all of them without changing any analysis result: every task is locked by equivalence tests (byte-identical metrics, identical artifacts) before implementation.

## Invariants

- **I1 (results identity).** Every `LoadMetric`, `ExtractionImpact`, and `LoadReport` byte-for-byte equals today's output; parallel/serial and warm/cold paths agree (extend the existing warm-vs-cold nets in `load_perf_tests.rs`).
- **I2 (artifact stability).** `report.json`, `report.bin`, and `manifest.json` contents unchanged (they are the diff-stability lock; Phase 5 of the first pass dropped sort-skipping for exactly this reason).
- **I3 (determinism).** Single-pass partitions and memo reuse must produce the same sorted vectors and the same `(count desc, path asc)` / `(path asc, file_id asc)` orders.
- **I4 (lock discipline).** New shared caches follow the established pattern: `OnceLock` for one-time build (lock-free reads after init), `Mutex` only around map lookup/insert with the guard ending before any BFS, no guard held across another lock or a rayon boundary. `LoadAnalyzer` stays `Send + Sync` (compile-asserted).
- **I5 (memory).** Append-only memo entries stay bounded by distinct seeds / def-files; memo extraction must not duplicate the memo per consumer.

## Ruled Out (do NOT re-propose)

- **Dropping per-symbol vectors from the artifact.** The UI consumes them: `dto.rs` `justification_names` reads `ContributingSource.symbols`, `cli/src/commands/ui.rs:240` builds `file_collateral` from `file_metrics`. Vectors stay.
- **Skipping the determinism sorts.** Locked by I2/I3 (first pass Phase 5 disposition).
- **DashMap/lockless-concurrent maps.** Current `Mutex` critical sections are map-lookup-only; the wins here come from *sharing* warm memos and lock-free `OnceLock` adjacency, not from swapping the map primitive.

## Task Breakdown (each is one Red-Green-Refactor cycle)

### A1 — `override_closure`: reuse a lockless include-adjacency (graph crate)
- **Problem:** `load.rs:509-538` rebuilds `HashMap<FileId, Vec<FileId>>` from `include_graph.edges` on EVERY call — twice per `extraction_impact` (`kept_subtree` + `root_after`), and `extraction_impact` runs per justification entry (payload caps 16/root; route unbounded).
- **Red:** test that N `extraction_impact` calls perform ≤ 1 adjacency build (add `#[cfg(test)]` `adjacency_builds: AtomicUsize` counter, test-only, mirroring `semantic_bfs_traversals`), plus cold-vs-warm equivalence for `extraction_impact` over the existing impact fixtures.
- **Green:** add `file_adjacency: OnceLock<HashMap<FileId, Vec<FileId>>>` to `LoadAnalyzer` (build identical to `IncludeGraph::adjacency()`, or expose/reuse that); `override_closure` reads the `OnceLock` (plain `&`, no guard during BFS) and branches on `current == source` for the replacement edges. Do NOT hold any other guard during `get_or_init`.
- **Refactor:** keep `override_closure` < 100 lines; the `adjacency()` duplication between `graph.rs` and `load.rs` must be collapsed to one call site.

### A2 — `analyze_file_load_by_id`: stop re-deriving FileIds (graph + pipeline)
- **Problem:** `collect_file_metrics` has each `FileId` in hand, then calls `analyze_file_load(path)` → `find_file_id` O(V) string scan per file (graph.rs:32-50). `api_justification` (server.rs:670) does the same round-trip (`get` path → `analyze_file_load` → `find_file_id`).
- **Red:** unit test: `analyze_file_load_by_id(file_id)` == `analyze_file_load(path)` for every fixture file, and a scan-counter test locking `find_file_id` call count (or assert the by-id path never calls it — simplest: add `#[cfg(test)]` counter in `find_file_id`).
- **Green:** add `pub fn analyze_file_load_by_id(&self, file_id: FileId) -> Result<LoadMetric>`; `analyze_file_load` becomes resolve-then-by-id (no behavior change); pipeline `collect_file_metrics` calls the by-id variant (still gating on `file_paths.get` for the "unregistered id → skip" contract — verify current flatten/skip semantics match: today unregistered ids yield `None` and are skipped).
- **Refactor:** `ui.rs` `api_justification` also uses by-id (lands with A4; A2 must not regress it).

### A3 — `compute_load`: single-pass partition (graph crate)
- **Problem:** load.rs:717-726 walks `physical_symbols` twice (filter+contains+clone per pass), then `contributing_sources` clones every element again into per-file buckets — ~4 String clones/element/metric × 2160 symbols + 560 files.
- **Red:** characterization: for the justification/concurrency fixtures, warm-vs-cold + a `partition_by_closure` helper test asserting the two vectors equal today's filter-pair output including sort order (I1/I3). Count clones only via behavior (no counter needed — output identity suffices).
- **Green:** one `for s in physical_symbols { if semantic_closure.contains(s) {...} else {...} }` partition; both vectors sorted afterward (sort keeps I3 regardless of hash iteration order). Do not merge the two `contributing_sources` calls yet — grouping per side stays, but each side now feeds from a pre-sorted vector.
- **Refactor:** `compute_load` stays under 100 lines; if it grows past, extract the partition into `fn partition_symbols(...)`.

### B1 — Extract shared `Arc<LoadMemo>`; make analyzer construction trivial (graph crate)
- **Problem:** every UI consumer builds a **cold** analyzer (`dto.rs:205` payload; `server.rs:669` route). The route additionally builds via `LoadAnalyzer::new` — no closures — so its first closure query runs a full `compute_closures()` **per request**, then cold-memo BFS per entry. This is the single biggest remaining user-facing cost.
- **Design:** extract the cache fields (`closures`, `symbol_adjacency`, `semantic_closure_cache`, `physical_expansion_cache`, test counters) into a `load_caches.rs` sibling type `LoadMemo` (struct with the same OnceLock/Mutex fields; methods moved verbatim). `LoadAnalyzer<'a>` becomes `{ include_graph, model, memo: Arc<LoadMemo> }` — a cheap handle. Public API: `LoadAnalyzer::new(...)` builds a fresh `Arc<LoadMemo>` (unchanged behavior); add `LoadAnalyzer::with_memo(graph, model, Arc<LoadMemo>)` and `LoadAnalyzer::memo(&self) -> Arc<LoadMemo>` (clone = refcount bump, thread-safe by construction — same I4 discipline lives inside `LoadMemo`). This also executes the first pass's deferred hygiene item ("load.rs 991/1000 lines — next touch: extract cache fields to `load_caches.rs`"), pulling load.rs back under the cap.
- **Red:** (a) compile-assert `LoadMemo: Send + Sync`; (b) two analyzers sharing one memo: warm the first, second adds ZERO semantic BFS / physical expansions / closure computations (extend the existing traversal-count tests); (c) concurrent shared-memo equivalence reusing `test_concurrent_shared_analyzer_matches_sequential` shape with two handles.
- **Green:** mechanical extraction; no logic changes. Pipeline `ensure_load` unchanged internally (fresh memo per run — still correct, still the I5 bound).
- **Refactor:** `load.rs` line-count relief is a hard gate (must end below 1000 with margin; verify `tools/hygiene_check.py` exit 0).

### C1 — UI server: share warm memo + precomputed closures across requests (ui crate)
- **Depends on:** B1.
- **Problem:** `api_justification` pays cold `compute_closures` + cold memos per request (see B1 Problem).
- **Red:** integration test (ui crate, in-memory or fixture-backed): two sequential `api_justification` calls produce identical bodies (existing route tests cover shape; add: second call must not rebuild closures/adjacency — instrument via memo counts exposed through a test-only hook on `UiState`, or assert response identity plus wall-time non-regression is NOT acceptable as a lock — prefer the count hook).
- **Green:** `UiState` stores `Arc<LoadMemo>` (or the shared analyzer handle) alongside `include_graph`/`reduced_model`, built ONCE per analysis rebuild (the watcher path that swaps the graph/model must rebuild the memo atomically with them — memo creation takes `with_closures` of the stage's precomputed closures, same as `ensure_load` does). `api_justification` constructs a per-request `LoadAnalyzer::with_memo` from the stored memo — handle is cheap; memos stay warm for the process lifetime. **Memo lifetime = analysis generation:** when a rebuild replaces graph/model, the old memo is dropped with them (no stale-entry window; the memo keys on FileId/SymbolId which are generation-local — document this).
- **Refactor:** `build_file_justification` (dto.rs payload path) optionally takes `Option<&Arc<LoadMemo>>` so the CLI payload build can seed from the pipeline's warm memo — include only if `ui.rs` plumbing is ≤ ~20 lines; otherwise defer.

### D1 — `compute_closures`: SCC condensation + reverse-topological DP (graph crate)
- **Problem:** graph.rs:86+ runs an independent BFS per file → O(V·(V+E)). One-time per cold pipeline run (builder.rs:78), but it dominates cold `ensure_include_graph` on large graphs and feeds A4/C1's closure payloads.
- **Red:** equivalence test over cycle-heavy fixtures (the bench fixture is already cycle-tolerant/SCC-dense): new implementation's `HashMap<FileId, IncludeClosure>` must equal today's per-file-BFS result exactly — `transitive_includes` and `direct_includes` (sorted/dedup per types.rs:55-67) — for the existing graph test fixtures AND a generated random-digraph fixture with cycles (seeded RNG, small N). Lock `generate_report`'s and `detect_cycles`'s expectations unchanged.
- **Green:** Tarjan SCC (iterative, no recursion-depth cliff on deep include chains) → condensation DAG → process SCCs in reverse topological order, union successor SCC member sets (intra-SCC members get the SCC's full member set plus external reach). Keep `adjacency()`'s sorted/dedup output feeding `direct_includes`.
- **Refactor:** module-level functions < 100 lines each (`scc_condensation`, `closure_dp`); if graph.rs crosses 1000 lines, split closure code to `closures.rs` sibling.
- **Note:** this is the largest task; if Red reveals semantic drift (cycle closure conventions), STOP and re-spec rather than tuning thresholds — I1 is absolute.

### E1 — `save_load_artifacts`: parallelize the double serialization (report crate)
- **Problem:** metrics_io.rs:12-56 serially: JSON the whole report, bincode the whole report, write both. Measured ~1.8–2.0 s of the 2.2 s stage floor. `report.json` has NO code reader (deserialization uses `report.bin`; `report/manager/tests.rs:257` only asserts existence) — but keep it (external/debug contract; deleting it is out of scope for this plan).
- **Red:** `save_load_artifacts` output-byte test: capture both files for a fixed report and assert exact bytes before/after refactor (I2). Ordering-independence: run with rayon available and unavailable (workers knob) — bytes identical.
- **Green:** `rayon::join` (or a scoped thread) the JSON serialize+write and bincode serialize+write; manifest build/write stays on the main path after. Both writes are to distinct files; `write_json_atomic`/`write_atomic` are per-file atomic (temp+rename) — verify no shared temp-name collision pattern before parallelizing.
- **Stretch (separate decision, do NOT bundle):** make the JSON export opt-in via a flag. Present results to the user with E1 measurements first.

## Delegation Plan (phases = parallel-safe sets)

| Phase | Tasks | Files touched | Conflict risk |
|---|---|---|---|
| 1 | A1, A2, A3, E1 | load.rs / graph.rs+lib.rs+pipeline / load.rs / metrics_io.rs | A1 & A3 both touch load.rs — serialize A3 after A1 or hand one Implementer both |
| 2 | B1 | load.rs + new load_caches.rs | hard gate after: workspace green + hygiene ≤ 1000 |
| 3 | C1 (needs B1), A4-by-id route update | server.rs, ui state, cli/ui.rs | — |
| 4 | D1 | graph.rs (+ closures.rs) | run last — largest blast radius, needs full suite |
| 5 | E1 if not done in 1 | — | — |

All Implementer handoffs get: the relevant task section above, "Red first, tool-runner executes", and the invariant list. Reviewer gate per task; D1 additionally gets a holistic review pass (graph-traversal correctness is easy to get subtly wrong on cycles).

## Verification & Measurement

- `cargo test --workspace` after every task; `python tools/hygiene_check.py` exit 0; clippy clean on touched files.
- Extend `load_perf_bench.rs` with a **second scenario**: time `build_file_justification`-style payload assembly and a simulated justification-route call against the 520-header fixture, before/after B1+C1 — the first pass's lesson was that the real floor hides outside the loop being timed.
- Re-run `collateral_load_stage_cold_timing` to confirm E1's save-phase drop and no analysis regression (D1 must not slow the memoized paths).
- Targets: `api_justification` steady-state per-request cost drops from full `compute_closures` to memo-hit lookups (expect >10× on the fixture); save phase ~2× via join; artifact bytes unchanged.

## Risks

- **D1 cycle semantics** — `IncludeClosure`'s convention (seed-excluded transitive set) must reproduce exactly under SCC condensation; the equivalence test against the BFS oracle is the gate, and it is written first.
- **C1 memo staleness** — memo entries keyed by FileId/SymbolId can only go stale if graph/model mutate under a live memo; the rebuild path must swap {graph, model, memo} as one unit. Test the swap (rebuild → new memo cold).
- **A3 partition order** — HashSet iteration order is arbitrary; sort-after keeps determinism, but `largest_collateral_source` (`sources.first()`) depends on the sort, not insertion — covered by characterization.
- **E1 parallel writes** — only if `write_json_atomic` temp names are per-call-unique; verify, else serialize temp-file creation.

---

## Execution Results (2026-09-28/29 IDD/TDD run, uncommitted as of writing)

All 8 tasks implemented Red-Green-Refactor; every phase passed an independent Code-reviewer gate (D1 got the plan-mandated holistic Tarjan pass, cross-edge lowlink handling explicitly verified). Baseline 523 → **543 tests, 0 failed**; `hygiene_check.py` exit 0; clippy net-flat vs 126-warning baseline with zero warnings on all touched files.

- **A1** `file_adjacency: OnceLock` — 1 build per N `extraction_impact` calls, counter-locked; adjacency logic collapsed to the single `IncludeGraph::adjacency()` definition. Cold/warm impact equivalence locked across 4 fixture variants.
- **A2** `analyze_file_load_by_id` — by-id path performs 0 `find_file_id` scans (counter-locked); pipeline `collect_file_metrics` by-id with skip contract test-locked.
- **A3** `partition_symbols` — single walk, sort-after, oracle = legacy filter-pair verbatim.
- **E1** `rayon::join` of JSON+bincode serialize/write; **I2 held**: artifact byte-hashes identical before/after, pinned in `load_artifact_bytes_are_byte_locked` + 1/4-thread ordering-independence test. `unique_tmp` is already pid+counter-unique — no extra lock needed. Save phase ~1.9–2.0 s (first-pass floor) → ~1.3–1.55 s in-bench (~30–35%; the join's floor is max(json,bin)+write, so the plan's ~2× ceiling is unattainable while report.json keeps full serialization — that is the stretch-flag's real value, deliberately deferred).
- **B1** `LoadMemo` extracted to `load_caches.rs` (410 ln); `LoadAnalyzer` = cheap `{graph, model, Arc<LoadMemo>}` handle + `with_memo`/`memo()`; shared-memo zero-rework + concurrent two-handle equivalence tests. **load.rs 991 → 664** (explain surface split to `load_explain.rs` mid-A-phase when A1 additions crossed 1000).
- **C1** `UiState.analysis: RwLock<Option<Arc<AnalysisGeneration{graph,model,memo}>>>` — one swap unit (also removed the old two-guards-across-analyzer-work pattern); boot seeds memo from stage closures (build AND disk-load paths); `api_justification` + `api_graph_collateral` use `with_memo` + by-id; counters exposed via opt-in `test-instrument` feature (dev-dep only; release binaries provably carry no counters). Second-request closure/adjacency/BFS/expansion delta == **exactly 0**; rebuild-swap staleness locked (`Arc::ptr_neq`, cold `(0,0,0,0)`, detached memo untouched). dto seeding landed (~16 ln).
- **D1** `closures.rs` (258 ln): iterative Tarjan SCC + reverse-topological DP; old BFS preserved verbatim as test-only oracle; equivalence locked over all existing fixtures + 5 LCG-seeded random digraphs (DAG/big-cycle/SCC+bridge/self-loops/full-blob). Cycle convention oracle-decided: seed always excluded from own transitive set; self-loops in direct only; unresolved edges never in closures. **compute_closures 40.1 ms → 3.5 ms (~11.5×)** on the 520-header fixture; memoized paths unchanged (counter tests green).

### Justification-route measurement (520-header fixture, release, 1 thread)

| scenario | before | after (B1+C1 shared memo) |
|---|---|---|
| route per-request, cold analyzer/req | ~1524–1672 ms | ~1188–1196 ms avg; last req 1138–1142 ms → **steady-state 1.3–1.4×** |
| payload assembly (24 roots/384 entries) | ~610–660 ms | unchanged (memo-shared; floor is per-entry work) |
| ensure_load total | ~1759–1942 ms | ~1898 ms (noise; save phase 1.3–1.55 s post-E1) |

**Plan-target finding:** the anticipated **>10× per-request drop was NOT met (1.3–1.4× achieved)** — measurement shows the memoizable share is only ~29% of per-request cost; the dominant floor is the route's **unbounded per-entry payload** (520 entries × 520 `extraction_impact` set-ops + unbounded name lists ≈ 12 s of 12.2 s route time). The C1 contract (counter delta = 0, warm/cold body identity) passed exactly as specified; wall-time expectation corrected by evidence. **Next lever (out of this plan's scope, needs its own intent):** cap/partition the justification route's per-entry cost — entry caps, batched impact computation, or lazy per-entry payloads.

### Follow-up hygiene tripwires (logged, not blocking)

- `cli/src/commands/ui.rs` is at exactly **999/1000** lines — any future edit must extract first.
- `build_file_justification_with_memo` at 93/100 lines; `load_perf_tests.rs` at 935.
- D1's real-fixture triple-equality gate lives in the `#[ignore]`d bench — consider a non-ignored reduced-shape variant.
- `explore_frames` carries 8 params (allow-free via struct-bundle would match the bench fix already applied for `report_after_route_comparison`).
