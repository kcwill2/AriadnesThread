# UTA-T5 Handoff - scheduler + worker-core + index: fingerprint hardening, swallow surfacing, boundary pins

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. docs/specs/unit-test-audit.md (UTA-B2, UTA-B6, dead-knob report rows).
3. crates/index/src/fingerprint.rs + cache.rs:61-111 (the AR-05 model fix),
   crates/scheduler/src/engine.rs:250-290, pool.rs (full), worker.rs:80-100,
   shard.rs, queue.rs, task/tu_store in crates/worker-core/src/**.
4. Model: crates/index/src/cache.rs InjectedUnserializableContext test seam.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/uta-t5-journal.md (journal; always in-set)
- crates/scheduler/src/**, crates/worker-core/src/**, crates/index/src/**
You MUST NOT touch: crates/pipeline or crates/cli (callers), Cargo.toml/lock.
No `git stash`, no branch change.

## What to build
FIXES (red test first, each):
1. UTA-B2 index/fingerprint.rs contexts_material: replace `unwrap_or_default()`
   with `serde_json::to_string(&(contexts, tus)).unwrap_or_else(|e|
   format!("SERIALIZE-FAILED:{}", e))`. Deterministic given same inputs;
   differs from honest empty (fail-closed: stale -> rebuild). Red test: seam
   type whose serialization fails -> material starts with SERIALIZE-FAILED and
   fingerprint != honest-empty-material fingerprint; two different failing
   states produce different materials. Mirror cache.rs test-seam style.
2. worker-core tu_store save tmp-name: include process id (+ atomic counter)
   so concurrent savers cannot collide; red test: two threads saving the same
   FileId -> final bytes parse as one complete checkpoint (no mixed tmp reuse).
3. scheduler engine.rs:268-275: foreign `tu_file_id` (not claimed by this
   shard run) must not be checkpointed with an empty fingerprint - skip it
   with a detectable signal (journal the chosen shape: e.g. count surfaced in
   return/log line). `let _ = save_shard(..)`: surface failures (journal the
   chosen minimal shape; at minimum a non-silent channel + a red/green test).
4. pool.rs fatal path: with two permanently-failing tasks + successes assert
   first error wins, non-failing tasks still ran, queue restored with residue
   (fix only if current behavior deviates; red exposes it).

REQUIRED TESTS (pins):
5. TuTask::cmp: equal-cost table incl. file_id then task_id descent; PartialEq
   includes tu; estimated_cost=0 boundary; heap-pop permutation order.
6. format_memory/format_elapsed boundary table (1023/1024 B/KB/MB/GB; 59.99s vs 60s);
   peak.max(1MB) floor via empty backend.
7. HashPartitioner::new(0)/ShardedReducer::new(0) clamp; 100-key range invariant.
8. estimate_tu_cost: empty file -> 1, directory path cost; propagation through
   from_translation_units_with_context with real temp files.
9. run_worker_pool empty queue -> Ok(empty) + queue empty; num_workers:0 -> max(1) drains.
10. tu_store: truncated JSON -> miss; non-UTF-8 shard -> miss; create_dir_all failure path pinned.
11. PluginWorkerBackend body-only arm missing file -> current empty-Ok pinned
    `pin_body_only_missing_file_analyzes_empty`; report.
12. edge_destinations: Wildcard fan-out id order; External/Unresolved on-disk fallback incl. -I dirs (mock resolver).
13. fingerprint_scoped_context: forced_includes in-root rewrite vs system verbatim; equal fingerprint across copied roots.
14. index types: IndexResult::new two TUs shared symbol dedup + incomplete_edges tally + empty state.
15. REPORT ONLY (no tests, no edits): scheduler memory_limit_bytes dead knob;
    index cache.rs IncrementalCache zero callers (dead public API).

## RGB protocol
Per item: Red (capture), Green, Refactor. NEVER weaken assertions.

## Hygiene (review-enforced)
<100-line functions, <1000-line files, banned-name ban, no single-letter locals;
`python tools/hygiene_check.py` exit 0.

## Frozen locks
Existing tests are contracts; edit only bug-locking tests for items 1-4, journal-justified.

## Gates (exact commands, timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-index` (red then green)
2. `python tools/testrun.py --distill -- -p ariadne-worker-core`
3. `python tools/testrun.py --distill -- -p ariadne-scheduler`
4. `python tools/hygiene_check.py` -- exit 0.
5. `python tools/encoding_guard.py --staged` -- exit 0.
6. Commit: `python tools/commit_scoped.py --scope "crates/scheduler/**" --scope "crates/worker-core/**" --scope "crates/index/**" --scope "tasks/uta-t5-journal.md" -m "fix(index,scheduler,worker-core): uta-t5 fingerprint hardening + checkpoint race/error surfacing"`

## Discipline clauses
- UTF-8: Write/Edit only, pure-ASCII.
- Touch only the touch-set; out-of-scope: report.
- No bare cargo; distilling entry points only.
- 1-3 line journal checkpoint per Green gate to tasks/uta-t5-journal.md.
- Context exhaustion imminent: STOP + disk-state inventory (order: index, worker-core, scheduler).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence. 3. Green distill headers +
canonical lines + capture path. 4. Deviations. 5. Out-of-scope observations.
Never paste raw tool output.
