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
