# Code Hygiene Refactor Plan

Re-surveyed 2026-09-26 at commit `1ce7954 persistent indexes` (clean tree). The persistence commit already fixed several violations: `cli/src/lib.rs` 1401→572 lines, `report/src/lib.rs` 1181→7-line facade, `run_ui` 166→<100 lines, and it shipped tests (`pipeline` unit tests + `m18_persistence_incremental_test.rs`).

## Goal
Enforce and hold: functions <=100 lines, files <=1000 lines, meaningful names, single responsibility, fewer fragile spots — across all Rust crates + integration tests, `crates/ui/assets/index.html`, Python helpers, and `tools/clang-worker`. Every behavior-preserving change is test-protected (TDD: characterization test first, then refactor, commit green).

## Decisions (confirmed with user)
- Baseline: index-persistence work committed as `1ce7954`; hygiene starts from this green baseline.
- Scope: all Rust crates + integration tests, `index.html` (3270 lines), Python helpers (`modify.py`, `parse.py`), `tools/clang-worker`.
- Enforcement: CI workflow + repo check script.
- Public cross-crate APIs: free to rename anything; update call sites in the same commit.

## Rules (definition of done for the codebase)
- R1: No function/method >100 lines (excl. blank/comment-only lines optionally, but measure consistently in the check script).
- R2: No source file >1000 lines.
- R3: Names describe purpose; no `data`/`tmp`/`handle_*` vagueness where a precise name exists.
- R4: One responsibility per type/function; split when a function mixes orchestration with detail work.
- R5: Fragility: production `unwrap()` outside `#[cfg(test)]` must be justified (replace with `?`/`Context` in `anyhow` paths where feasible).

## Known violations (current inventory at 1ce7954)
- Files >1000: `crates/report/src/manager.rs` (1327 = ~773 production + `#[cfg(test)]` block from line 774 with all 106 `unwrap()`s test-only).
- Files near limit: `crates/ui/tests/ui_test.rs` (861), `crates/transform/src/lib.rs` (839), `crates/index/src/analyzer.rs` (819), `crates/pipeline/src/lib.rs` (684 — newest file, watch it).
- UI asset: `crates/ui/assets/index.html` (3270).
- Functions >100 lines: `EndToEndOptimizer.optimize` (168, cli/optimize.rs:116), `ShardedReducer.reduce` (136, scheduler/shard.rs:27), `ParserCtx.handle_global_function` (131, index/analyzer.rs:334), `tokenize` (122, index/analyzer.rs:18), `OptimizationPlanner.plan` (117, cluster/planner.rs:35), `handle_apply` (112, cli/commands/transform.rs:133), `parse_args` (111, build/discovery.rs:259), `WorkerPool.run` (104, scheduler/worker.rs:184), `modify_html` (122, ui/assets/modify.py). Borderline exactly-100 (legal, fix opportunistically when the file is touched): `HeaderExtractor.extract` (transform/lib.rs:210), `ClangWorkerBackend.execute` (scheduler/worker.rs:46), `cli::run` (cli/lib.rs:250).
- Fragility: `pipeline/src/lib.rs` has 9 **production** `unwrap()`s (lines 257–488, `self.<stage>_cache.clone().unwrap()` on stage caches — new R5 target from persistence work). Other production-unwrap hotspots to classify against `#[cfg(test)]` during Pass 4: `cli/lib.rs` (38), `cluster/plan.rs` (22), `ui/server.rs` (23), `repo/*` (33), `scheduler/*` (tu_store 21, incremental 17).
- Untested hotspots (highest risk first): `api_graph_entrypoints`/`api_graph_search`/`api_graph_traverse` (ui/server.rs), `optimize`, `handle_apply`/`handle_migrate`, `ClangWorkerBackend.execute`, `BuildDiscoverer.parse_compile_commands_json`, `ParserCtx.handle_global_function`, `tokenize`. (Persistence stages `ensure_*` now covered by pipeline unit tests + m18; `run_ui` shrank and needs only light characterization.)

## Pass 0 — Baseline (no code changes)
1. Verify green baseline at `1ce7954`: `cargo build --workspace`, `cargo test` (workspace + `tests/integration`), fix any failures.
2. Record clippy baseline: `cargo clippy --workspace --all-targets` (goal is no *new* warnings during hygiene passes).

## Pass 1 — Shore up tests (characterization, before any refactor)
TDD: for each hotspot below, write a characterization test (assert current observable behavior), then it must stay green through all later refactors.
1. UI server API handlers (`api_graph_*`, index/file routes) — extend `crates/ui/tests/ui_test.rs`; this pass will likely push it past 861 lines, so split it into `tests/ui_test/*.rs` modules (server_api, graph_api, file_view) as part of this pass (satisfies R2 early).
2. `EndToEndOptimizer.optimize` + `handle_apply`/`handle_migrate`: fixture-driven integration tests capturing current outputs (use `tests/fixtures/basic`).
3. `ClangWorkerBackend.execute` + `WorkerPool.run`: unit tests with fake worker process (tempdir stdin/stdout), covering success, malformed JSON, and process-failure paths.
4. `tokenize` + `ParserCtx.handle_global_function`: unit tests locking token classification and declaration extraction edge cases (macros, templates, nested namespaces).
5. `parse_args` + `parse_compile_commands_json`: table-driven tests for flag parsing edge cases (`-D` with/without value, quoted paths, response files).
6. `run_ui` (cli/commands/ui.rs): light process-level smoke test via existing m15/m17 harness (route wiring + progress labels), since it will be re-split in Pass 3.
Gate: full suite green + committed before Pass 2.

## Pass 2 — Break up large files (pure moves only, no behavior change)
Each split = one commit; `cargo test` before/after; moves must not alter logic (compiler + tests verify).
1. `crates/report/src/manager.rs` (1327) → move the `#[cfg(test)]` module (553 lines) to `crates/report/tests/manager_test.rs` (or `manager/tests.rs` submodule), then split production code (~773) by artifact family: `manager.rs` (ArtifactManager lifecycle + save/load core) + `artifacts.rs` or per-domain modules for the `save_*`/`load_*_checked` groups. The persistence work already extracted report data types into `manifests.rs` — extend that pattern.
2. `crates/ui/assets/index.html` (3270) → `index.html` (markup only) + `styles.css` + JS split into feature files (`graph.js`, `file-view.js`, `hotspots.js`, `api.js`, `main.js`). Current serving is `include_str!("../assets/index.html")` in `crates/ui/src/lib.rs:9` (no static-dir serving), so add one `include_str!` const per asset plus explicit GET routes (`/styles.css`, `/app/*.js`) with correct MIME types in `server.rs`; add `Cache-Control: no-store` to those routes too (existing index-route rule applies to all UI assets).
3. `crates/transform/src/lib.rs` (839) → `extractor.rs`, `placement.rs`, `lib.rs` facade (do now, before renames push it past 1000).
4. `crates/index/src/analyzer.rs` (819) → `tokenizer.rs`, `parser.rs` (ParserCtx), `analyzer.rs` facade.
5. Watch-list (no action unless they cross 1000): `pipeline/src/lib.rs` (684) — if Pass 1 additions push it up, split stage impls from tests the same way as manager.rs.

## Pass 3 — Break up >100-line functions (extract method, TDD per function)
Work hottest-first. Each: run characterization tests, extract named private functions with single responsibility, keep public signature stable within the pass, commit per function group (one commit per file).
1. `cli/optimize.rs::optimize` (168) → stage methods (e.g. `plan_stage`, `apply_stage`, `verify_stage`).
2. `scheduler/shard.rs::reduce` (136) → shard-partition + merge steps.
3. `index/analyzer.rs::handle_global_function` (131) and `tokenize` (122) → per-state machine functions (`scan_string`, `scan_comment`, `classify_token`, etc.).
4. `cluster/planner.rs::plan` (117), `cli/commands/transform.rs::handle_apply` (112), `build/discovery.rs::parse_args` (111), `scheduler/worker.rs::WorkerPool.run` (104) → extract per-phase helpers; CLI mains keep thin orchestration.
5. `cli/commands/ui.rs::run_ui` → re-split stage wiring into named helpers (it is already <100 but mixes index-build, graph-build, server-setup, background-task wiring — R4 target).
6. Python: `modify_html` (modify.py) — if the script is still used after Pass 2 HTML split, refactor into helpers; otherwise delete it (verify no references first).
7. Opportunistic (exactly 100, legal under R1): trim `HeaderExtractor.extract`, `ClangWorkerBackend.execute`, `cli::run` when their files are already touched in this pass.

### Pass 3 status — COMPLETE 2026-09-26 (18 commits, `5234f88`..`e83dd71`)
All planned items done plus three leftovers a repo-wide sweep caught that the inventory above missed: `graph/callgraph.rs::build_from_index` (107→22, `609faa3`), `m16_traversal_test` e2e (119→54, `b497364`), `m21_run_ui_smoke_test` (120→33, `e83dd71`). Item 4's `WorkerPool.run` measured 99 (already legal — left untouched); item 5 `run_ui` already <100 (left); `HeaderExtractor.extract` already <100 after Pass 2 split. `execute` 118→~65 and `cli::run` 141→72 handled as mandatory (had grown past 100). New characterization before decomposing: `d2b140d` (ShardedReducer.reduce — locked first-wins definitions w/o Class/Struct precedence, kind/name from first record, `tu.references` excluded from `tu_usage`), `bf4e200` (OptimizationPlanner.plan — locked single-SCC zero-proposal header drop, sanitized stem-collision duplicate targets). `modify.py` deleted (`42b1e4f`, zero refs repo-wide). Gate evidence: workspace 380 green, fmt clean, clippy lines/unique unchanged at every commit. R1/R2 repo-wide sweep at `e83dd71`: zero violations (one fnlen brace-literal false positive verified by reading source — Pass 5 checker must count brace STRING LITERALS correctly).

## Pass 4 — Naming + single-responsibility sweep
Per-crate audit passes (one commit per crate), each kept green:
1. Rename vague names (`run`, `handle_*`, `optimize`, `process`) to purpose names, e.g. `handle_apply` → `apply_transform_plan`; `run` variants → `run_worker_pool`, `run_analysis`. Cross-crate renames: update call sites same commit.
2. SRP: flag any type touching >1 concern (e.g. ArtifactManager doing IO + validation + manifest bookkeeping — align with Pass 2 split); functions that both compute and print: split compute vs render.
3. Fragility sweep, production code only (classify `unwrap()` against `#[cfg(test)]` first — manager.rs's 106 are all test-only and fine):
   - `pipeline/src/lib.rs` (9 production unwraps at 257–488): convert stage-cache unwraps to explicit errors (`context("stage cache must exist after ensure")`) or restructure so the cache is a parameter, not an `Option`.
   - `cli/lib.rs` (38), `ui/server.rs` (23), `cluster/plan.rs` (22), `scheduler/tu_store.rs` (21), `repo/*` (33), `graph/load.rs` (19), `scheduler/incremental.rs` (17) → convert to `?`/`map_err`/typed errors where a real failure path exists; keep `unwrap()` only on provably-invariant internals (prefer `debug_assert`).

### Pass 4 status — COMPLETE 2026-09-26 (27 commits, `e83dd71..f7ace60`, Code-reviewer ACCEPTED)
Renaming + fragility sweep done, gated green at every commit; plan's baseline inventory was stale in places (`cli/lib.rs`/`tu_store`/`incremental`/`repo/*` already zero production unwraps after Pass 2/3). Fragility: pipeline stage-cache unwraps → `.context(...)?` via private `cached_*` accessors with Err-path test first (`af92d9e`/`a0b6378`); 33 RwLock-poison unwraps in ui+cli → `read_state`/`write_state` `into_inner()` recovery (`8f7dd24`/`c29256e`); 27 infallible `writeln!(String).unwrap()` → `let _` (cluster/build); graph/load.rs 18 sites → `?`-propagated or `"unknown"` placeholder matching load.rs's own precedent, characterized first (`24554b7`/`5362760`) — this pushed load.rs to 1049 (R2 regression), tests extracted pure-move to `load_characterization_tests.rs` (`daa4ed1`, 501 lines); graph-search JSON → total accessors + degenerate-data test (`1575766`); provable invariants → `.expect("invariant msg")` (peek-guarded `next()`, Tarjan/BFS maps, JoinHandle); the ONE sanctioned behavior change `161f028` (optimize skip-on-Err — `analyze_symbol_load` legitimately errors on `defining_file: None`) initially REVIEW-REJECTED for missing regression lock → locked `59ea0a7`. R3: 20 CLI `handle_*` → purpose names (`apply_transform_plan` plan-mandated; `build_migration_preview` deliberately names only what the locked latent-bug body does), `cli::run`→`run_cli`, `WorkerPool::run`→`run_worker_pool`, `WorkerBackend::execute`→`analyze_task`, `EndToEndOptimizer::optimize`→`optimize_repository`, vague locals (`res`/`tmp`/`data`) named; index parser `handle_*` left (state their token target + characterization-name locks). Gate at `f7ace60`: 389 tests green (380@Pass3 +9), fmt clean, fresh-vs-fresh clippy vs `e83dd71` content-identical 125/125, R2 clean (max 815), production `.unwrap()` = 5 sites all R5-annotated (2 axum builder + 3 clang-worker serde). Follow-up ledger from review: 6 pre-existing production fns >100 (reduced::reduce 115, run_worker_pool 115, resolve_directive 106, transform extract 110, build_ui_payload 101, engine::analyze 101 — fn>100 count was 12 incl. tests, not the 10 batch reports claimed; Pass-3's "zero violations" claim held only under its brace-literal blind heuristic); `read_state`/`write_state` duplicated ui↔cli.

## Pass 5 — Enforcement (final commit set)
1. `tools/hygiene_check.py` (Python, matches existing parse.py style): walks repo, flags any source file >1000 lines (`.rs`, `.py`, `.html`, `.js`) and any top-level/impl `fn`/`def` >100 lines by brace heuristics; exit 1 on violation. Keep its own logic under limits.
2. `.github/workflows/ci.yml`: on push/PR run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (+ integration), `python tools/hygiene_check.py`.
3. **IGNORE, AGENTS.MD IS NO LONGER USED** Update `AGENTS.md` note: hygiene_check must pass; rules R1–R5 are hard gates.

### Pass 5 status — COMPLETE 2026-09-27 (17 commits, `609c3ac..421a1b0`, Code-reviewer ACCEPTED)
`tools/hygiene_check.py` R1/R2 gate + 41 stdlib-unittest tests (`0287b66`); `.github/workflows/ci.yml` (`3f685f4`, `d719f97` python→python3) runs fmt `--all --check`, clippy (deliberately WITHOUT `-D warnings`: legacy 125–126-warning baseline intact — reviewer fresh-vs-fresh at HEAD `8c8f0a6`=126 vs Pass-4 tip `f7ace60`=125, unique-message set-diff EMPTY both directions; flip to `-D warnings` is a tracked follow-up once baseline cleared), `cargo test --workspace` (tests/integration is a workspace member), `hygiene_check.py`, checker unit tests. Checker exposed exactly the 12 latent R1 violations the Pass-4 review ledger predicted (6 production + 6 test fns; `shard.rs::reduce` correctly absent ≤100); all decomposed TDD-style, behavior-preserving (reviewer: "verbatim/equivalent" per commit): `867acd2` reduce→`merge_tu_symbols`/`merge_tu_edges`; `ceb465d` resolve_directive→`collect_include_candidates`; `fbf1b82` run_worker_pool→`spawn_queue_drain_workers` (drop(tx) ordering + retry/requeue verified identical); `def0806` analyze→`build_analysis_stats` (elapsed() taken at original point); `a5a2906` extract→`build_generated_header`/`detect_cycle_in_headers`; `8ce6ad2` build_ui_payload→`build_file_collateral`; test fns `1326438` `ad7f1d1` `bb95261` `782a22a` `5752fe5` `8c8f0a6` (assertion counts/semantics preserved, verified). First review REJECTED: 4 checker escape/false-flag vectors on valid code — lowercase `fn` type-params splitting Rust spans; Python wrapped signatures ending at `):` line; column-0 closing `"""` ending def body early; `str.splitlines()` pseudo-breaks (`\f` `\v` `\x85` U+2028/2029) inflating line counts — fixed `adb7878` with per-vector regression tests (plus disclosed `_advance_python_string` absolute-match-position bug that had masked inline strings), physical-line rule locked to `\r\n|\r|\n` via single `split_physical_lines`; p7 MAJOR follow-up (def bracket depth opened on body lines never accumulating) closed `421a1b0` + `__pycache__/` gitignored. Allow-list: exactly `report.html` (regenerated artifact, size varies — no line count in comment), exact-path. Gate at `421a1b0`: 389 workspace tests (×3), fmt clean, clippy zero-new, `hygiene_check.py` exit 0 — first time in repo history with zero R1/R2 violations under a correct gate — 41 checker tests, full adversarial probe corpus green. Open follow-ups (recorded, non-blocking): Python def depth counts only `()` not `[]`/`{}` continuations (narrower than closed p7; formatter-never-emits indentation); ui↔cli `read_state`/`write_state` duplication (Pass-4 ledger); clippy 125-baseline cleanup then `-D warnings` flip.

## Validation (every pass)
- `cargo build --workspace`
- `cargo test` (workspace + integration tests)
- `cargo clippy --workspace --all-targets` (no new warnings)
- `python tools/hygiene_check.py` (from Pass 5; run it ad hoc from Pass 2 onward)
- UI pass: `cargo build --bin tool` + `target\debug\tool.exe ui <analysis-project-path>` smoke-check that graph/file-view/hotspots panels still load.

## Risks
- `index.html` split is the only change with real runtime-failure risk (asset serving/paths/MIME); mitigated by ui_test + manual smoke check before committing.
- `pipeline` stage-cache unwrap conversion (Pass 4) changes failure modes from panic to `Err` — pair it with a test asserting the error path before changing it.
- Renaming cross-crate APIs in Pass 4 touches many files; keep one crate per commit so `git revert` is cheap.
- Clang worker subprocess tests may be flaky; use short timeouts and tempdirs.

## Out of scope
- Behavior changes, new features, dependency upgrades, `report.html`/fixture artifacts, and the C++ test fixtures.
