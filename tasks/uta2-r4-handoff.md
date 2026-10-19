# UTA2-R4 Handoff - sweep-B remainder: worker-core drop pin + small crate completions

Binding inputs:
1. This file. 2. docs/specs/unit-test-audit.md "Round 2" (drop pin + rows 1, 8-18). 3. crates/worker-core/src/tu_store.rs:285-322 + worker_backend.rs; crates/scheduler/src/incremental.rs; crates/build/src/discovery.rs:60-120 + measure.rs:160-187; crates/cluster/src/plan.rs:63; crates/artifact/src/manager/*.rs (load_*_checked); crates/transform/src/placement.rs:27; crates/lang/src/extractor.rs:110; crates/validate/src (capture/fingerprint IO-error observation from uta-t4 review). 4. Model: round-1 pin files (tu_store_t5_pins.rs, uta_t6_pins.rs naming).

## Touch-set (hard)
- tasks/uta2-r4-journal.md
- crates/worker-core/src/**, crates/scheduler/src/**, crates/build/src/**, crates/cluster/src/**, crates/artifact/src/**, crates/transform/src/**, crates/lang/src/**, crates/validate/src/**
You MUST NOT touch: crates/repo, crates/lang-api (R3 owns them this round), crates/ui, cli optimize/commands files, graph, lang-cpp, lang-scala. No stash/branch change.

## What to build (ALL PINS unless marked; red-exposed wrong behavior not in spec = pin + report)
1. worker-core edge_destinations: Resolved(id)/Wildcard(ids) with id absent from path_by_id -> silent drop pinned as `pin_edge_destination_drops_unknown_id` + REPORT recommending a surfaced counter (same class as UTA-B6; no fix - shape decision owed). WorkerBackend-for-Arc<T> delegating-spy test.
2. scheduler incremental.rs: save_artifacts IO-failure (read-only dir -> Err surfaces, no manifest written); analyze partial-save state pinned.
3. build: compile-db candidate precedence (root > build/ > out/ with conflicting dbs); malformed JSON -> Err names the file; measure_tu simulation fallback attribution pin (frontend_cpu_time == wall_clock) - honest name.
4. cluster plan.rs format_summary one-shape lock.
5. artifact load_*_checked: one mismatch pin per variant (or confirm+pin shared-helper equivalence with a grep-lock note in the journal).
6. transform is_noop: identical-lists + non-empty preservations -> false row completes the truth table.
7. lang extractor render_unit direct pin (header layout/namespace shape).
8. validate: FileSnapshot::capture dir-as-path bare-IO-error semantics + rolled_back IO-error observation shipped in uta-t4 review - pin exactly.

## RGB, hygiene, locks
Pins only. No behavior change; every touched file keeps its round-1 pin files intact.
Hygiene/encoding exit 0.

## Gates (timeout 600000 ms each) - one distill per touched crate (8 total, in budget)
1-8. `python tools/testrun.py --distill -- -p ariadne-{worker-core,scheduler,build,cluster,artifact,transform,lang,validate}`
9. hygiene + encoding (witnesses)
10. Commit: `python tools/commit_scoped.py --scope "crates/worker-core/**" --scope "crates/scheduler/**" --scope "crates/build/**" --scope "crates/cluster/**" --scope "crates/artifact/**" --scope "crates/transform/**" --scope "crates/lang/**" --scope "crates/validate/**" --scope "tasks/uta2-r4-journal.md" -m "test: uta2-r4 sweep-B residual completion pins"` (truth-adjust).

## Discipline
Write/Edit only, pure ASCII; distill entry points; journal per green crate; STOP on context wall (order: worker-core, scheduler, build, cluster, artifact, transform, lang, validate).

## Report back
Commit hash + stat; per-crate canonical lines (before -> after); rows stopped with reasons; out-of-scope observations.
