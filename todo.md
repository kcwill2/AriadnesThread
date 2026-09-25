# Todo List

## Completed Tasks

- [x] **M0 — Project Skeleton & Test Harness**
  - [x] Create workspace `Cargo.toml` with 10 core crates and `clang-worker` sidecar tool crate
  - [x] Implement CLI binary (`ariadne` / `tool`) supporting `--help`, `version`, `analyze <repo>`, and `discover <repo>`
  - [x] Implement `ariadne-report` artifact manager creating and initializing `artifacts/` roots
  - [x] Create synthetic C++ fixture repository in `tests/fixtures/basic` with intentional collateral dependency (`heavy.h` included by `thing.h`)
  - [x] Add integration test `test_m0_fixture_location_and_artifacts` in `tests/integration`
  - [x] Verify all unit and integration tests pass via `cargo test --workspace`
- [x] **M1 — Repository Discovery**
  - [x] Source file enumeration & C++ file type classification (`Header`, `Source`, `Other`)
  - [x] Path canonicalization and deterministic SHA-256 relative path `FileId` generation
  - [x] Directory-name & path rule based `FileOwnership` (`Repository`, `ThirdParty`, `Toolchain`, `Generated`) and `OptimizationPolicy` (`Optimizable`, `Contract`, `Frozen`) classification
  - [x] SHA-256 file content hashing (`content_hash`)
  - [x] CLI `tool discover <repo>` command with formatted summary output
  - [x] Artifact generation: `artifacts/repository/files.json`, `files.bin`, `manifest.json`
  - [x] Unit and integration tests for discovery, policy precedence, determinism, and artifact creation
- [x] **M2 — Build & Compilation Discovery**
  - [x] Unit 1: Define compilation context, compile command, and translation unit models with normalization & determinism (`crates/build`)
  - [x] Unit 2: Implement compilation context discovery engine supporting compilation databases and context configs (`crates/build`)
  - [x] Unit 3: Implement build artifact generator (`artifacts/contexts/contexts.json`, `commands.json`, `manifest.json`) in `crates/report`
  - [x] Unit 4: Add `tool build discover <repo>` CLI command and integration test harness with `DEBUG`/`NDEBUG` fixtures in `tests/integration`
- [x] **M3 — Include Graph**
  - [x] Unit 1: Define include graph types and data structures (`crates/graph/src/types.rs`)
  - [x] Unit 2: Implement `#include` parser and search path resolver (`crates/graph/src/resolver.rs`)
  - [x] Unit 3: Implement `IncludeGraph` engine with transitive closure, cycle detection, consumer TUs, and explain tree (`crates/graph/src/graph.rs`)
  - [x] Unit 4: Implement artifact persistence for `edges.bin`, `closures.bin`, and `report.json` (`crates/report/src/lib.rs`)
  - [x] Unit 5: Implement CLI commands `tool includes` and `tool includes explain` (`crates/cli`) and integration test suite (`tests/integration/tests/m3_include_graph_test.rs`)
- [x] **M4 — Semantic Indexing**
  - [x] Unit 1: Indexing data structures & types (`crates/index/src/types.rs`)
  - [x] Unit 2: C++ Semantic analyzer & parser engine (`crates/index/src/analyzer.rs`, backend abstraction, and worker sidecar `tools/clang-worker`)
  - [x] Unit 3: Index result aggregation & artifact persistence in `crates/report`
  - [x] Unit 4: CLI command `tool index <repo>` in `crates/cli` and integration test suite (`tests/integration/tests/m4_semantic_indexing_test.rs`)

- [x] **M5 — Reduced Semantic Model**
  - [x] Unit 1: Reduced semantic model data structures & reduction engine (`crates/graph/src/reduced.rs`)
  - [x] Unit 2: Reduced model artifact persistence in `crates/report` (`artifacts/reduced/`)
  - [x] Unit 3: Query engine & CLI `tool query` commands (`crates/cli`, `crates/graph`)
  - [x] Unit 4: Integration test suite & artifact determinism test (`tests/integration/tests/m5_reduced_model_test.rs`)

- [x] **M6 — Dependency Load Analysis**
  - [x] Unit 1: Define load analysis metrics and calculation engine with provenance (`crates/graph/src/load.rs`)
  - [x] Unit 2: Implement load artifact persistence and reporting (`crates/report/src/lib.rs`)
  - [x] Unit 3: Implement CLI commands `tool analyze load` and `tool explain load` (`crates/cli`)
  - [x] Unit 4: Integration test suite for load analysis, collateral quantification, and explain provenance (`tests/integration/tests/m6_load_analysis_test.rs`)

- [x] **M7 — Optimization Planning**
  - [x] Unit 1: Define OptimizationPlan, PlanId, ProposedExtraction, RiskLevel, and RejectionReason types and formatters (`crates/cluster/src/plan.rs`)
  - [x] Unit 2: Implement SCC analysis, movability checks, and constraint/rejection validator (`crates/cluster/src/scc.rs`, `crates/cluster/src/checker.rs`)
  - [x] Unit 3: Implement FrontierAnalyzer and OptimizationPlanner engine generating deterministic explainable plans (`crates/cluster/src/frontier.rs`, `crates/cluster/src/planner.rs`)
  - [x] Unit 4: Implement plan artifact persistence and loader in `crates/report` (`artifacts/plans/plans.json`, `plans.bin`, `manifest.json`)
  - [x] Unit 5: Implement CLI commands `tool plan <repo>` and `tool plan show <repo> --id <ID>` in `crates/cli`
  - [x] Unit 6: Integration test suite for optimization planning, golden plan fixtures, determinism, and rejection cases (`tests/integration/tests/m7_optimization_planning_test.rs`)

- [x] **M8 — Header Extraction**
  - [x] Unit 1: Define `ExtractedSymbol`, `GeneratedHeader`, and `ExtractionResult` data types (`crates/transform/src/lib.rs`)
  - [x] Unit 2: Implement `HeaderExtractor` engine — groups proposed extractions by target header, resolves symbol kinds from the reduced model, and detects include cycles (`crates/transform/src/lib.rs`)
  - [x] Unit 3: Implement `ExtractionResult::write_to_disk` — writes `#pragma once`, predecessor `#include`s, and symbol bodies to generated header files
  - [x] Unit 4: Implement `ArtifactManager::save_extraction_artifacts` in `crates/report` (`artifacts/generated/`)
  - [x] Unit 5: Implement CLI command `tool transform extract <repo> --plan <ID>` in `crates/cli`

- [x] **M9 — Consumer Rewriting**
  - [x] Unit 1: Define `ConsumerRewrite` and `MigrationPreview` data types with diff formatting and noop/effective-rewrite counting (`crates/transform/src/lib.rs`)
  - [x] Unit 2: Implement `IncludeRewriter::compute_rewrite` — maps referenced symbol names to generated headers and adds required predecessor includes (`crates/transform/src/lib.rs`)
  - [x] Unit 3: Side-effect guard — includes with zero referenced symbols are placed in `side_effect_preservations` and never silently removed (`crates/transform/src/lib.rs`)
  - [x] Unit 4: Implement `ArtifactManager::save_migration_artifacts` and `load_migration_artifacts` in `crates/report` (`artifacts/migration/preview.bin`, `preview.json`, `manifest.json`)
  - [x] Unit 5: Implement CLI command `tool migrate <repo> --plan <ID> --preview` in `crates/cli`

- [x] **M10 — Validation & Transactions**
  - [x] Unit 1: Define `ValidationTier`, `TierResult`, `ValidationStatus`, and `ApplyOutcome` types in `crates/validate/src/lib.rs`
  - [x] Unit 2: Implement `FileSnapshot` — capture per-file content hashes before apply so rollback can restore the exact pre-apply state
  - [x] Unit 3: Implement `TransactionApplier` — write generated headers and consumer rewrites to disk (actually modifying files)
  - [x] Unit 4: Implement `TierValidator` — Tier 1 (structural: acyclic graph, no lost symbols), Tier 2 (generated headers self-containment check), Tier 3 (affected TU stub validation)
  - [x] Unit 5: Implement `Rollback` — restore all snapshotted files, remove any newly created files, verify repository hash matches pre-apply state
  - [x] Unit 6: Add `ArtifactManager::save_transaction_artifacts` in `crates/report` (`artifacts/transaction/`)
  - [x] Unit 7: Implement CLI command `tool apply <repo> --plan <ID>` in `crates/cli` with full lifecycle output (Applying…, tier statuses, Committed. / Rolling back… / Repository restored.)
  - [x] Unit 8: Integration test suite in `tests/integration/tests/m10_validation_test.rs` — success path, intentional-failure-then-rollback (repository hash identity), and multi-tier validation

- [x] **M11 — Build Performance Measurement**
  - [x] Unit 1: Define `TuMeasurement`, `BuildMeasurement`, and `MeasurementComparison` data types, metrics calculation, and formatting in `crates/build/src/measure.rs`
  - [x] Unit 2: Implement `BuildMeasurer` engine with per-TU time/memory measurement, attribution, and structural load correlation
  - [x] Unit 3: Implement measurement and comparison artifact persistence and loading in `crates/report` (`artifacts/measure/`)
  - [x] Unit 4: Add `tool measure <repo>` and `tool compare <before> <after>` CLI commands in `crates/cli`
  - [x] Unit 5: Controlled fixture with expensive headers and integration test suite (`tests/integration/tests/m11_measurement_test.rs`)

- [x] **M12 — Incremental Analysis**
  - [x] Unit 1: Define cache data structures, fingerprints, and validation rules (`crates/index/src/cache.rs`)
  - [x] Unit 2: Implement cache artifact persistence and loading in `crates/report` (`artifacts/cache/`)
  - [x] Unit 3: Implement `IncrementalAnalyzer` engine in `crates/scheduler` (selective TU invalidation & cache reuse)
  - [x] Unit 4: Integrate CLI command `tool analyze <repo>` with required output format in `crates/cli`
  - [x] Unit 5: Integration test suite for cache hit rate, header modification invalidation, compiler context invalidation, and analysis version mismatch (`tests/integration/tests/m12_incremental_analysis_test.rs`)

- [x] **M13 — Production-Scale Execution**
  - [x] Unit 1: Straggler-mitigating TU scheduling & prioritized work queue (`crates/scheduler/src/queue.rs`)
  - [x] Unit 2: Worker pool with process-level isolation, failure recovery & memory tracking (`crates/scheduler/src/worker.rs`)
  - [x] Unit 3: Shard generation, hash-partitioned shuffle & parallel reducers (`crates/scheduler/src/shard.rs`)
  - [x] Unit 4: Resumability, checkpointing & progress reporting (`crates/scheduler/src/engine.rs`)
  - [x] Unit 5: CLI integration (`tool analyze <repo> --workers <N>`) and production-scale integration test suite (`tests/integration/tests/m13_production_scale_test.rs`)

- [x] **M14 — End-to-End Optimization**
  - [x] Unit 1: Optimization summary report & M14 formatted reporting (`crates/cli/src/optimize.rs`)
  - [x] Unit 2: End-to-end automated optimization pipeline engine (`crates/cli/src/optimize.rs`)
  - [x] Unit 3: CLI command `tool optimize <repo>` integration (`crates/cli/src/lib.rs`)
  - [x] Unit 4: End-to-end integration test suite & failure rollback verification (`tests/integration/tests/m14_end_to_end_test.rs`)

- [x] **M15 — Graphical Interface & Call Graphs**
  - [x] Unit 1: Call Site Extraction & Semantic Indexing (`crates/index`)
  - [x] Unit 2: CallGraph Data Structure & Analysis Engine (`crates/graph`, `crates/report`)
  - [x] Unit 3: Graph DTOs, Web Server & Export Engine (`crates/ui`)
  - [x] Unit 4: Interactive Frontend SPA (`crates/ui/assets`)
  - [x] Unit 5: CLI Integration & End-to-End Integration Suite (`crates/cli`, `tests/integration`)

- [x] **M16 — Interactive Graph Traversal in UI**
  - [x] Unit 1: Traversal & Pathfinding Primitives in `ariadne-graph` (neighborhood extraction, shortest path, entrypoint discovery)
  - [x] Unit 2: UI DTOs & Axum Server Traversal Endpoints in `ariadne-ui` (`/api/graph/traverse`, `/api/graph/path`, `/api/graph/entrypoints`, `/api/graph/search`)
  - [x] Unit 3: Interactive Frontend Traversal, Ariadne's Thread Bar & Pathfinder in `crates/ui/assets/index.html`
  - [x] Unit 4: End-to-End Integration Tests & Standalone HTML Export Verification

## Active Feature Tasks

- [x] **Node Selection Edge Highlighting in UI**
  - [x] Unit 1: Add unit tests in `crates/ui/tests/ui_test.rs` verifying node selection edge highlighting logic, classes (`highlighted`, entering, leaving), and `arrow-selected` marker in `INDEX_HTML` (RED)
  - [x] Unit 2: Implement entering and leaving edge highlighting, dynamic marker selection, edge layering, and visual styling in `crates/ui/assets/index.html` (GREEN & REFACTOR)
