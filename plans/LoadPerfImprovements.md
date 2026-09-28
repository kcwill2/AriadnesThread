---
agent: devin-local
session: spicy-scarecrow
created: 2026-09-28T15:18:12Z
---
# Optimize Collateral Analysis Performance

Identify and implement performance optimizations for the "Analyzing collateral" load step, which currently rebuilds expensive data structures repeatedly for each symbol and file analysis.

## Performance Analysis Summary

The "Analyzing collateral" step (stage 5/5) has significant performance bottlenecks:

### Key Bottlenecks Identified

1. **Symbol adjacency rebuilt N+M times** (load.rs:228-238)
   - `get_semantic_closure_of_symbols()` rebuilds adjacency map from ALL edges on every call
   - Called for every symbol analysis (N times) + every file analysis (M times)
   - Cost: O(E) per call where E = number of semantic edges

2. **Graph closures not reused efficiently** (load.rs:219, 345)
   - `compute_closures()` performs BFS for EVERY file even when closures are precomputed
   - Called in `get_transitive_includes()` and `strict_closure()`
   - Cost: O(V+E) per file where V = files, E = include edges

3. **No caching of semantic closures**
   - Each symbol/file analysis performs fresh BFS through dependency graph
   - Many symbols share common dependency subgraphs
   - Cost: O(V+E) per analysis where V = symbols, E = semantic edges

4. **Redundant sorting** (load.rs:520-521)
   - Both collateral and semantic symbol vectors sorted unconditionally
   - May not be needed for all use cases

## Implementation Plan

### Phase 1: Precompute Symbol Adjacency (High Impact)

**File: `crates/graph/src/load.rs`**

1. Precompute symbol adjacency map in `LoadAnalyzer::new()` and `with_closures()`
2. Store in `RefCell<Option<HashMap<SymbolId, Vec<SymbolId>>>>` instead of rebuilding
3. Remove lazy initialization from `get_semantic_closure_of_symbols()`

**Expected Impact**: Eliminates O(E * (N+M)) work → O(E) one-time cost

### Phase 2: Efficient Closure Caching (High Impact)

**File: `crates/graph/src/load.rs`**

1. Ensure closures passed via `with_closures()` are always used
2. Remove redundant `compute_closures()` calls in `get_transitive_includes()` and `strict_closure()`
3. Add validation that closures match the graph when debugging

**Expected Impact**: Eliminates O((V+E) * M) redundant BFS traversals

### Phase 3: Semantic Closure Memoization (Medium Impact)

**File: `crates/graph/src/load.rs`**

1. Add `RefCell<HashMap<SymbolId, HashSet<SymbolId>>>` for caching semantic closures
2. Cache results of `get_semantic_closure_of_symbols()` by seed symbol
3. Use cache for symbol analysis (single seed) and fall back to computation for multi-seed cases

**Expected Impact**: Reduces redundant traversals for common dependency subgraphs

### Phase 4: Parallel File Analysis (Medium Impact)

**File: `crates/pipeline/src/lib.rs`**

1. Use `rayon` for parallel iteration over files in `ensure_load()`
2. Keep LoadAnalyzer as read-only during analysis (already true)
3. Add feature flag for parallel execution

**Expected Impact**: Utilizes multiple cores for I/O-bound analysis

### Phase 5: Conditional Sorting (Low Impact)

**File: `crates/graph/src/load.rs`**

1. Add parameter to `compute_load()` to control whether sorting is needed
2. Only sort when results will be displayed/serialized
3. Skip sorting for internal computations

**Expected Impact**: Small reduction in per-analysis overhead

## Files to Modify

- `crates/graph/src/load.rs` - Core load analysis logic
- `crates/pipeline/src/lib.rs` - Pipeline orchestration (for parallelization)
- `crates/graph/src/load_characterization_tests.rs` - Update tests if needed
- `crates/graph/src/load_justification_tests.rs` - Update tests if needed

## Verification

- [ ] Run existing load analysis tests to ensure correctness
- [ ] Measure performance on a representative codebase
- [ ] Verify no regressions in collateral analysis results
- [ ] Check that caching doesn't cause memory bloat
- [ ] Test parallel execution doesn't break determinism

## Risks/Considerations

- **Memory usage**: Caching adjacency and closures increases memory footprint
- **Complexity**: Memoization adds complexity; need to ensure cache invalidation is correct
- **Determinism**: Parallel execution must maintain consistent ordering in results
- **Testing**: Need to verify optimizations don't change analysis results
- **Incremental builds**: Ensure cached data is correctly persisted/loaded

## Performance Targets

- **Goal**: Reduce collateral analysis time by 50-70% on large codebases
- **Baseline**: Current implementation with 1000+ symbols and 500+ files
- **Measurement**: Time `ensure_load()` stage in isolation

---

## Execution Results (2026-09-28, executed via IDD/TDD orchestration)

**Fixture**: synthetic 520 headers / 40 TUs / 2160 reduced symbols / 5520 semantic edges / 560 file metrics — meets the 1000+ symbols / 500+ files baseline target. Bench: `cargo test -p ariadne-pipeline --test load_perf_bench --release -- --ignored --nocapture --test-threads=1` (cold `ensure_load`, best-of-two, 16 cores).

| Metric | Baseline HEAD | Final | Reduction |
|---|---|---|---|
| `ensure_load` stage total | 7145 ms | 2204 ms | **69.2%** ✅ |
| Analysis phases only (symbol+file loops, the UI "Analyzing collateral" step) | ≈4218–4445 ms | ≈413 ms | **≈91%** ✅ |
| Artifact save (`save_load_artifacts`, serial) | ≈1.5–1.6 s | ≈1.8–2.0 s | unchanged (Amdahl floor) |
| In-memory report deep copy into pipeline cache | ≈0.7–0.9 s | 0 ms (Arc move) | 100% |

**Phase dispositions:**
- **Phase 1 (precompute symbol adjacency) & Phase 2 (closure reuse): ALREADY PRESENT at plan authoring time.** The plan's bottleneck claims #1/#2 were stale — lazy `get_or_insert_with` caching + `with_closures()` already eliminated repeated builds. Superseded by `OnceLock` (contention-free reads) during Phase 4.
- **Phase 3 (semantic closure memoization): DONE + EXTENDED.** Per-seed `Arc<HashSet<SymbolId>>` memo in `LoadAnalyzer`; multi-seed queries served as unions of cached single-seed closures (zero BFS once seeds are warm — the symbol pass warms all seeds before the file pass). Locked by BFS-count tests in `crates/graph/src/load_perf_tests.rs`.
- **Phase 4 (parallel file analysis): DONE.** `LoadAnalyzer` made `Send + Sync` (OnceLock/Mutex caches); `ensure_load` runs both symbol and file passes via rayon `par_iter` with order-preserving collection; scoped `ThreadPoolBuilder` honors `Pipeline::workers`. Deliberate deviation: no cargo feature flag (avoids an untested dual path); `workers` is the parallelism knob. Extra wins discovered during measurement: `compute_load` borrows the memoized closure (killed per-call set clone) and a per-def-file `physical_expansion_cache`.
- **Phase 5 (conditional sorting): DROPPED — rationale.** Every `compute_load` result is serialized into the load artifact; the sorts are the artifact-determinism lock (diff stability across runs). Internal consumers (`extraction_impact`, `required_headers_for_symbols`) never call `compute_load`, so the claimed savings were unreachable. Removing the sort would make JSON artifacts unstable.
- **Bonus fix (found by measurement):** the stage-total bottleneck was not analysis but the serial artifact tail — `ensure_load`'s `report.clone()` deep copy (~0.9 s) plus artifact save. `load_cache` now holds `Arc<LoadReport>` (no deep copy); `cached_load()` unchanged signature.

**Verification:** `cargo test --workspace` green (graph 63, pipeline 12 + 1 ignored bench); `python tools/hygiene_check.py` exit 0; clippy zero warnings in load*/pipeline/bench files; load.rs 991/1000 lines (next touch: extract cache fields to `load_caches.rs` sibling — 9 lines from cap). Memory tradeoff (plan risk item): caches are append-only per analyzer — one closure per distinct seed symbol and one expansion per distinct def-file; documented in field docs, entry counts locked by `test_cache_entry_counts_match_hand_computed_fixture`.

**Review:** three Code-reviewer passes — memoization APPROVED; rayon parallelization APPROVED; holistic final pass over all layers APPROVED (remaining nits deferred: direct unit test for the multi-def-file `compute_load` fallback — dead-ish code, equivalence-by-construction; load.rs module split on next growth).
