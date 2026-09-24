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

## Upcoming Milestones

- [ ] **M2 — Build & Compilation Discovery**
- [ ] **M3 — Include Graph**
- [ ] **M4 — Semantic Indexing**
- [ ] **M5 — Reduced Semantic Model**
- [ ] **M6 — Dependency Load Analysis**
- [ ] **M7 — Optimization Planning**
- [ ] **M8 — Header Extraction**
- [ ] **M9 — Consumer Rewriting**
- [ ] **M10 — Validation & Transactions**
- [ ] **M11 — Measurement**
- [ ] **M12 — Incremental Analysis**
- [ ] **M13 — Production-Scale Execution**
- [ ] **M14 — End-to-End Optimization**
