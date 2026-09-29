# Fix the "Analyzing collateral" tail (stage 5/5 last 25%)

## Context

The `[5/5] Analyzing collateral...` bar maps ONLY the rayon per-file metrics loop (`CollateralFile` events) onto step 0%→75% (`crates/cli/src/commands/ui.rs:125-131, 202-207`). Everything else in stage 5 runs with **zero progress events** and renders as the frozen "last 25%":

1. Symbol pass before the file loop (no events) — `pipeline/src/lib.rs:503-514`.
2. Serial artifact save after the last file event — `pipeline/src/lib.rs:522` (~1.8–2.0 s per `plans/LoadPerfImprovements.md:126`).
3. `get_or_build_load()` deep-clones the whole `LoadReport` (refcount is 2 because the cache keeps one ref; `Arc::unwrap_or_clone` copies) — `pipeline/src/lib.rs:582-588`, called at `ui.rs:236`.
4. `get_or_build_include_graph()` clones graph + full closure HashMap a second time — `pipeline/src/lib.rs:556-562`, called at `ui.rs:233`.
5. **`build_extraction_proposals` re-runs the entire load analysis cold and serial**: `OptimizationPlanner::plan()` builds a cold `LoadAnalyzer::new` (`crates/cluster/src/planner.rs:60`), serially calls path-based `analyze_file_load` per header (`planner.rs:70-87, 128-157`), rebuilds the full SCC adjacency per candidate (`planner.rs:187`, `cluster/src/scc.rs:10-16`), and recomputes include closures inside `map_header_consuming_tus` (`planner.rs:65-67`, `graph/src/graph.rs:186-187`).
6. Serial `build_file_justification_with_memo`, `generate_report`, `detect_cycles`, `build_include_elements` (`ui.rs:238-255`).

Decisions already made with the user:
- Scope = perf fixes **and** honest progress reporting (weighted sub-phase spans + new milestone events).
- Planner reuses the stage's `LoadReport.file_metrics` (no re-analysis) for the UI path; CLI `tool optimize`/`plan` keep the cold path byte-identical.
- Artifact save stays **inline** (async save rejected: watcher `rehash_inputs`/restart-freshness contract). We only report it, don't hide it.
- Payload DTO output must remain identical (characterization tests protect it).

## Tasks (ordered; RGB-TDD — failing test first per task)

### A0 — Measure the tail (bench first, no behavior change)
- New ignored bench `crates/cli/tests/payload_tail_bench.rs` (cli crate depends on all needed crates). Duplicate the synthetic-fixture generator from `crates/pipeline/tests/load_perf_bench.rs` into the bench (test-code duplication is acceptable; do not add a crate dependency).
- Time and print per-phase ms: symbol pass, file pass, `save_load_artifacts`, `get_or_build_load` clone, `get_or_build_include_graph` clone, `OptimizationPlanner::plan()` cold, `build_file_justification_with_memo`, `build_include_elements`.
- Record baseline table in the task notes; these numbers drive the A4 span constants.
- Run: `cargo test -p ariadne-cli --test payload_tail_bench --release -- --ignored --nocapture --test-threads=1`.

### A1 — Planner consumes precomputed facts (kills the cold serial re-analysis)
- Cluster: extend `OptimizationPlanner` with optional analysis inputs (new constructor, keep `new()` cold behavior untouched):
  - candidate screening from a `FileId → (collateral_load, largest_collateral_source)` map built by the caller from `LoadReport.file_metrics` (via `path_to_id`), instead of `load_analyzer.analyze_file_load(&path)` (`planner.rs:128-157`).
  - optional `&HashMap<FileId, IncludeClosure>`: add `IncludeGraph::map_header_consuming_tus_with_closures(tu_ids, &closures)`; existing `map_header_consuming_tus` delegates with `compute_closures()` (`graph.rs:186`).
- Cli: `build_extraction_proposals` (`ui.rs:349-362`) passes facts from `load_report.file_metrics` + stage closures.
- Semantics lock tests (must pass BEFORE refactor, against today's cold path on the fixture):
  - plans built from precomputed facts == plans built cold (same order, same `PLAN-001..` ids).
  - headers in `file_paths` but absent from `file_symbols` are filtered on both paths (empty syms ⇒ collateral_load 0).
  - metrics whose `target_file` is unregistered are never candidates.

### A2 — Planner internal cost (benefits CLI too)
- Hoist SCC adjacency construction out of the per-header loop: build once from `mapped_edges`, pass `&adj` into a new `compute_symbol_sccs_from_adj`; keep `compute_symbol_sccs` as a delegating wrapper (`scc.rs:10`).
- Run the per-header loop with rayon `par_iter().filter_map/flat_map` + order-preserving collect over `sorted_header_ids` (same determinism pattern as `collect_file_metrics`, `pipeline/src/lib.rs:611-650`); renumber `PLAN-{:03}` sequentially after collection.
- Lock: characterization test asserts identical `Vec<OptimizationPlan>` output vs current serial implementation.

### A3 — Kill the deep copies
- `cached_load()`/`get_or_build_load()` return `Arc<LoadReport>` (refcount bump, no copy). Update callers (`ui.rs:236`, bench, `pipeline_tests.rs`) and revise the contract test at `pipeline_tests.rs:358-389` ("hand clones" wording, `strong_count` expectations: cache + returned handle = 2).
- Add `Pipeline::take_include_graph()` (moves `(graph, closures)` out of the cache); `build_ui_payload` uses it instead of `get_or_build_include_graph` — safe because the `Pipeline` is single-use per build and the watcher always constructs a fresh one. Test: `take` empties the cache; `cached_include_graph` after `take` returns the documented "stage cache must exist" error; `build_ui_payload` still passes `test_ui_is_artifact_first` needles.

### A4 — Honest stage-5 progress
- Pipeline: add `ProgressEvent::LoadMilestone(LoadMilestone::SymbolsAnalyzed { symbols }| ArtifactsSaved)` emitted inside `ensure_load` (`pipeline/src/lib.rs:503-522`).
- Cli: stage 5's [80,100] global span is split into named sub-span constants: symbol-pass start milestone → symbols-analyzed milestone → file-loop span (replaces the fixed 80→95 mapping, `ui.rs:125-131`) → artifacts-saved milestone → assembly milestones emitted directly by `build_ui_payload`: "Planning extractions" (before `build_extraction_proposals`) and "Payload assembled" (100, terminal, unchanged finalize contract).
- Percent weights chosen from re-measured A0 numbers after A1–A3 land; if unmeasured, conservative fallback documented next to the constants. Monotonic, each report stays inside stage 5's span.
- Update `progress_contract_tests` (`ui.rs:661-806`) for the new milestone set; keep monotonicity + span-containment + label-stability tests green. Terminal-bar finalize contract unchanged (only step 5 at 100% finalizes, `progress.rs:105-134`).

### A5 — File-size split (do BEFORE A4 additions land in ui.rs)
- `crates/cli/src/commands/ui.rs` is at 999/1000 lines. Extract payload-assembly builders (`build_file_collateral`, `build_extraction_proposals`, `conservative_proposals`, DTO helpers + `builder_tests`) into `crates/cli/src/commands/ui_payload.rs`; keep run/progress/watcher orchestration in `ui.rs`.
- Careful: `test_ui_is_artifact_first` and `test_stage_milestones_match_step_labels` grep `include_str!("ui.rs")` needles — point needles at both files after the move.

## Validation

- `cargo test --workspace` green (expect ~543 + new tests, 0 failures).
- `cargo test -p ariadne-cli --test payload_tail_bench --release -- --ignored --nocapture --test-threads=1`: tail-phase table vs A0 baseline; success criterion = planner phase collapses to screening-only cost, report/graph clone phases ≈ 0, total stage-5 window materially reduced (report actual numbers, no invented %).
- `python tools/hygiene_check.py` exit 0 (functions <100 lines, files <1000 — watch planner.rs, pipeline/src/lib.rs, new ui_payload.rs).
- `cargo clippy` warnings net-flat vs baseline.
- Manual smoke: `ariadne ui <fixture-repo>` boot bar visibly advances through the new milestones; `--export` path still produces a byte-stable report.html; background watcher refresh logs one line per new milestone.

## Risks / notes

- **Determinism**: planner output order and PLAN ids must be identical post-parallelization; characterization + parity tests are the gate.
- **A3 API change** is workspace-internal (callers verified: `ui.rs`, bench, pipeline tests).
- **A1 equivalence depends on filtering semantics** (headers without `file_symbols` ⇒ load 0 ⇒ non-candidate); if the parity test disagrees, keep the cold path for the discrepant class and record it — do not change planner semantics.
- Artifact save stays serial-inline (rayon json+bin overlap already exists, `report/src/manager/metrics_io.rs:62-65`); further save speedups are out of scope.
- `build_file_justification_with_memo` parallelization: only if A0/A4 re-measure shows it as a remaining top-3 tail phase; otherwise out of scope.
- Per project convention, this plan file (`.kilo/plans/`) is committed alongside the implementation.
