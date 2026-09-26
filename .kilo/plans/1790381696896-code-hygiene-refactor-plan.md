# Code Hygiene Refactor Plan

## Goal
Enforce and hold: functions <=100 lines, files <=1000 lines, meaningful names, single responsibility, fewer fragile spots — across all Rust crates + integration tests, `crates/ui/assets/index.html`, Python helpers, and `tools/clang-worker`. Every behavior-preserving change is test-protected (TDD: characterization test first, then refactor, commit green).

## Decisions (confirmed with user)
- Commit in-flight index-persistence work first as a clean baseline.
- Scope: everything listed above, including `index.html` (3227 lines).
- Enforcement: CI workflow + repo check script.
- Public cross-crate APIs: free to rename anything; update call sites in the same commit.

## Rules (definition of done for the codebase)
- R1: No function/method >100 lines (excl. blank/comment-only lines optionally, but measure consistently in the check script).
- R2: No source file >1000 lines.
- R3: Names describe purpose; no `data`/`tmp`/`handle_*` vagueness where a precise name exists.
- R4: One responsibility per type/function; split when a function mixes orchestration with detail work.
- R5: Fragility: production `unwrap()` outside tests and `#[cfg(test)]` must be justified (replace with `?`/`Context` in `anyhow` paths where feasible).

## Known violations (starting inventory)
- Files >1000: `crates/cli/src/lib.rs` (1401), `crates/report/src/manager.rs` (1316, 106 `unwrap()`), `crates/report/src/lib.rs` (1181).
- Files near limit: `crates/ui/tests/ui_test.rs` (861), `crates/transform/src/lib.rs` (839), `crates/index/src/analyzer.rs` (819).
- UI asset: `crates/ui/assets/index.html` (3227).
- Functions >100 lines: `EndToEndOptimizer.optimize` (171, cli/optimize.rs:118), `run_ui` (166, cli/commands/ui.rs:6), `ShardedReducer.reduce` (136, scheduler/shard.rs:27), `ParserCtx.handle_global_function` (131, index/analyzer.rs:334), `tokenize` (122, index/analyzer.rs:18), `OptimizationPlanner.plan` (117, cluster/planner.rs:35), `handle_apply` (112, cli/commands/transform.rs:133), `parse_args` (111, build/discovery.rs:259), `WorkerPool.run` (104, scheduler/worker.rs:184), `HeaderExtractor.extract` (100, transform/lib.rs:210), `ClangWorkerBackend.execute` (100, scheduler/worker.rs:46), `cli::run` (100, cli/lib.rs:244), `modify_html` (122, ui/assets/modify.py).
- Untested hotspots (highest risk first): `run_ui`, `optimize`, `handle_apply`/`handle_migrate`, `api_graph_entrypoints`/`api_graph_search`/`api_graph_traverse` (ui/server.rs), `ClangWorkerBackend.execute`, `BuildDiscoverer.parse_compile_commands_json`, `ParserCtx.handle_global_function`.

## Pass 0 — Baseline (no code changes)
1. Verify the in-flight work is complete: `cargo build --workspace`, `cargo test` (workspace + `tests/integration`), fix any failures.
2. Commit the in-flight index-persistence work as its own commit (pipeline crate, fingerprint.rs, tu_store.rs + modified files).
3. Record baseline: `cargo clippy --workspace --all-targets -- -D warnings` output (allow existing warnings as a baseline file if noisy; goal is no *new* warnings).

## Pass 1 — Shore up tests (characterization, before any refactor)
TDD: for each hotspot below, write a failing-now characterization test (assert current observable behavior), then it must stay green through all later refactors.
1. UI server API handlers (`api_graph_*`, index/file routes) — extend `crates/ui/tests/ui_test.rs`; this pass will likely push it past 861 lines, so split it into `tests/ui_test/*.rs` modules (server_api, graph_api, file_view) as part of this pass (satisfies R2 early).
2. `run_ui` (cli/commands/ui.rs): extract-free tests via existing integration harness (m15/m17 tests pattern); assert route wiring and progress behavior at process level.
3. `EndToEndOptimizer.optimize` + `handle_apply`/`handle_migrate`: fixture-driven integration tests capturing current outputs (use `tests/fixtures/basic`).
4. `ClangWorkerBackend.execute` + `WorkerPool.run`: unit tests with fake worker process (tempdir stdin/stdout), covering success, malformed JSON, and process-failure paths.
5. `tokenize` + `ParserCtx.handle_global_function`: unit tests locking token classification and declaration extraction edge cases (macros, templates, nested namespaces).
6. `parse_args` + `parse_compile_commands_json`: table-driven tests for flag parsing edge cases (`-D` with/without value, quoted paths, response files).
Gate: full suite green + committed before Pass 2.

## Pass 2 — Break up >1000-line files (pure moves only, no behavior change)
Each split = one commit; `cargo test` before/after; moves must not alter logic (compiler + tests verify).
1. `crates/cli/src/lib.rs` (1401) → `cli/src/` modules: `cli.rs` (arg types), `run.rs` (dispatch), `output.rs`/`report_print.rs`, keep `lib.rs` <200 lines of re-exports.
2. `crates/report/src/manager.rs` (1316) → split by responsibility: `manager.rs` (ReportManager lifecycle), `builder.rs` (report assembly), `formats.rs` (serialization); move `#[cfg(test)]` blocks into sibling `tests` modules when files still exceed 1000.
3. `crates/report/src/lib.rs` (1181) → `types.rs`, `templates.rs` (HTML/rendering), keep `lib.rs` as module facade.
4. `crates/ui/assets/index.html` (3227) → `index.html` (markup only) + `styles.css` + JS split into feature files (`graph.js`, `file-view.js`, `hotspots.js`, `api.js`, `main.js`). Current serving is `include_str!("../assets/index.html")` in `crates/ui/src/lib.rs:9` (no static-dir serving), so add one `include_str!` const per asset plus explicit GET routes (`/styles.css`, `/app/*.js`) with correct MIME types in `server.rs`; add `Cache-Control: no-store` to those routes too (existing index-route rule applies to all UI assets).
5. `crates/transform/src/lib.rs` (839) → `extractor.rs`, `placement.rs`, `lib.rs` facade (do now, before it crosses 1000 during renames).
6. `crates/index/src/analyzer.rs` (819) → `tokenizer.rs`, `parser.rs` (ParserCtx), `analyzer.rs` facade.

## Pass 3 — Break up >100-line functions (extract method, TDD per function)
Work hottest-first. Each: run characterization tests, extract named private functions with single responsibility, keep public signature stable within the pass, commit per function group (one commit per file).
1. `cli/commands/ui.rs::run_ui` → separate: index-build stage, graph-build stage, server-setup, background-task wiring.
2. `cli/optimize.rs::optimize` → stage methods (`plan_stage`, `apply_stage`, `verify_stage`).
3. `scheduler/shard.rs::reduce` → shard-partition + merge steps.
4. `index/analyzer.rs::handle_global_function`, `tokenize` → per-state machine functions (`scan_string`, `scan_comment`, `classify_token`, etc.).
5. `cluster/planner.rs::plan`, `cli/commands/transform.rs::handle_apply`, `build/discovery.rs::parse_args`, `scheduler/worker.rs::run`/`execute`, `transform/lib.rs::extract`, `cli/lib.rs::run` → extract per-phase helpers; CLI mains keep thin orchestration.
6. Python: `modify_html` (modify.py) — if the script is still used after Pass 2 HTML split, refactor into helpers; otherwise delete it (verify no references first).

## Pass 4 — Naming + single-responsibility sweep
Per-crate audit passes (one commit per crate), each kept green:
1. Rename vague names (`run`, `handle_*`, `optimize`, `process`) to purpose names, e.g. `handle_apply` → `apply_transform_plan`; `run` variants → `run_worker_pool`, `run_analysis`. Cross-crate renames: update call sites same commit.
2. SRP: flag any type touching >1 concern (e.g. ReportManager doing IO + formatting + validation — align with Pass 2 split); functions that both compute and print: split compute vs render.
3. Fragility sweep: production `unwrap()` in `cli/lib.rs` (38), `pipeline/lib.rs` (48), `cluster/plan.rs` (22), `ui/server.rs` (23), `repo/*` (33) → convert to `?`/`map_err`/typed errors where a real failure path exists; keep `unwrap()` only on provably-invariant internals with a comment-free justification (prefer `debug_assert`).

## Pass 5 — Enforcement (final commit set)
1. `tools/hygiene_check.py` (Python, matches existing parse.py style): walks repo, flags any source file >1000 lines (`.rs`, `.py`, `.html`, `.js`) and any top-level/impl `fn`/`def` >100 lines by brace heuristics; exit 1 on violation. Keep its own logic under limits.
2. `.github/workflows/ci.yml`: on push/PR run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (+ integration), `python tools/hygiene_check.py`.
3. Update `AGENTS.md` note: hygiene_check must pass; rules R1–R5 are hard gates.

## Validation (every pass)
- `cargo build --workspace`
- `cargo test` (workspace + integration tests)
- `cargo clippy --workspace --all-targets` (no new warnings)
- `python tools/hygiene_check.py` (from Pass 5; run it ad hoc from Pass 2 onward)
- UI pass: `cargo build --bin tool` + `target\debug\tool.exe ui <analysis-project-path>` smoke-check that graph/file-view/hotspots panels still load.

## Risks
- `index.html` split is the only change with real runtime-failure risk (asset serving/paths); mitigated by ui_test + manual smoke check before committing.
- Renaming cross-crate APIs in Pass 4 touches many files; keep one crate per commit so `git revert` is cheap.
- Clang worker subprocess tests may be flaky; use short timeouts and tempdirs.

## Out of scope
- Behavior changes, new features, dependency upgrades, `report.html`/fixture artifacts, and the C++ test fixtures.
