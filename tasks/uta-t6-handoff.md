# UTA-T6 Handoff - cluster + artifact + build + pipeline: security decode, manifest, frontier, reuse pins

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. docs/specs/unit-test-audit.md (pin-only adjudication rows for cluster/artifact/build).
3. crates/artifact/src/manager/bincode_options.rs, manager.rs:50-180,
   manager/apply_io.rs, metrics_io.rs, index_io.rs, transform_io.rs;
   crates/cluster/src/frontier.rs, planner.rs:30-50,660-710, scc.rs output order;
   crates/build/src/types.rs:85-105, discovery.rs:60-80,280-300, measure.rs:70-90;
   crates/pipeline/src/stage_load.rs:150-190, stage_index.rs:90-120,
   for_incremental.rs:70-95, lib.rs:80-110,660-690.
4. Model: crates/artifact/src/manager.rs existing stale-epoch test; build
   characterization suite style.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/uta-t6-journal.md (journal; always in-set)
- crates/cluster/src/**, crates/artifact/src/**, crates/build/src/**,
  crates/pipeline/src/**
You MUST NOT touch: crates/cli/crates/ui callers, Cargo.toml/lock. No stash, no branch change.

## What to build
ALL ITEMS ARE PINS unless marked FIX. No behavior changes ship except FIX rows.
1. artifact bincode_options: hand-built blob declaring huge string/collection
   length -> fast Err, no allocation blowup (assert bounded time via immediate
   result, not wall clock). THE security purpose of the module; both variants.
2. artifact manager check_manifest: root-mismatch -> stale; key-absent manifest
   -> pin current skip behavior `pin_manifest_without_root_key_skips_root_check`;
   corrupt manifest ("{not json") -> stale; same_path table (case, `/` vs `\`,
   trailing slash, `\\?\` verbatim).
3. apply_io load_measurement_from_path: corrupt bin + valid json -> pin silent
   fallback as `pin_corrupt_bin_falls_back_to_json`; both-corrupt Err message;
   unknown extension. metrics_io write_streaming_atomic: emit closure Err ->
   tmp removed, prior bytes intact; rename-fail pinned.
4. index_io: non-empty TU roundtrip + reuse Some(IncrementalStats) -> manifest fields + reload.
   transform_io: extraction manifest fingerprint "" policy pinned as
   `pin_extraction_manifest_fingerprint_empty`; report.
5. cluster frontier: all 5 path shapes (no-path [], start==collateral,
   all-optimizable, missing policy entry -> Optimizable, and
   `start -> frozen -> optimizable -> collateral` pinned as
   `pin_frontier_breaks_at_first_frozen`); `_file_ownerships` ignored pinned
   as `pin_frontier_ignores_ownership`; report both.
6. cluster planner: merge_colocated_batches multi-group intersecting merge
   (anchor stays, reverse-append, transitive shared symbol) -> deterministic
   batch order/membership asserted; screening_facts duplicate metrics ->
   stable winner pinned; check_candidate_safety accept + frozen-target pair.
7. cluster scc: cross-SCC output ORDER byte-asserted over fixed fixture,
   repeated runs equal; duplicate seeds in symbols slice pinned.
8. build types normalize: two same-named contexts -> pin whole-context drop as
   `pin_normalize_drops_duplicate_named_context`; report. relative_path_for_source
   verbatim-root vs plain file branch; discover precedence root>build/>out with
   two compile_commands.json; format_num boundaries 0/1_000/100_000/1_000_000
   (note divergence vs cluster/plan.rs impl in journal, no unify).
9. pipeline stage_load try_reuse_persisted_load: corrupt report.bin + fresh
   manifest -> Rebuilt Ok (pin the .ok() swallow contract); stage_index
   `--lang` filtered path: retained units + no full-fp manifest written;
   for_incremental restrict_and_validate_units unhandled-language bail Err;
   cycle_policy_by_file unknown-plugin -> Reject; RunStats::record replace
   semantics + format_summary both branches.
10. REPORT ONLY: planner.rs:477 per-candidate SCC rebuild (perf).

## RGB protocol
Pins: write test, if it goes red against current behavior and the behavior was
NOT adjudicated in the spec, STOP that row, journal it, ship the rest.

## Hygiene (review-enforced)
<100-line functions, <1000-line files, banned-name ban, no single-letter locals;
`python tools/hygiene_check.py` exit 0.

## Frozen locks
Existing tests byte-locks are contracts: do not edit any (no fixes listed here).

## Gates (exact commands, timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-artifact`
2. `python tools/testrun.py --distill -- -p ariadne-cluster`
3. `python tools/testrun.py --distill -- -p ariadne-build`
4. `python tools/testrun.py --distill -- -p ariadne-pipeline`
5. `python tools/hygiene_check.py` -- exit 0.
6. `python tools/encoding_guard.py --staged` -- exit 0.
7. Commit: `python tools/commit_scoped.py --scope "crates/cluster/**" --scope "crates/artifact/**" --scope "crates/build/**" --scope "crates/pipeline/**" --scope "tasks/uta-t6-journal.md" -m "test(cluster,artifact,build,pipeline): uta-t6 security/manifest/frontier/reuse pins"`

## Discipline clauses
- UTF-8: Write/Edit only, pure-ASCII.
- Touch only the touch-set; out-of-scope: report.
- No bare cargo; distilling entry points only.
- 1-3 line journal checkpoint per Green gate to tasks/uta-t6-journal.md.
- Context exhaustion imminent: STOP + disk-state inventory (order: artifact,
  cluster, build, pipeline).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Per-crate distill headers + canonical
lines + capture path. 3. Rows skipped/stopped with reasons. 4. Out-of-scope
observations. Never paste raw tool output.
