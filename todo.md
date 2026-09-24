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

## Upcoming Milestones
- [ ] **M12 — Incremental Analysis**
- [ ] **M13 — Production-Scale Execution**
- [ ] **M14 — End-to-End Optimization**
