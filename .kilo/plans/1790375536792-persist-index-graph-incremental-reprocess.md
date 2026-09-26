# Persist index + graph, reprocess on code change

## Goal
Every analysis stage persists to `artifacts/` with an **input fingerprint**; every command (`analyze`, `index`, `includes`, `plan`, `analyze load`, `optimize`, `ui`) loads a stage if its fingerprint still matches the code, otherwise rebuilds that stage (and its dependents) and re-persists. The UI additionally live-refreshes via a polling re-hash watcher.

## Decisions (agreed)
- **Scope:** all commands unified behind one artifact-first pipeline.
- **Granularity:** TU-level reuse via existing content-hash fingerprints (`crates/index/src/cache.rs`); downstream stages (include graph, reduced, call graph, load, UI payload) rebuild wholesale when their input fingerprint mismatches. No per-edge incremental patching.
- **UI:** fresh-check at startup + **live refresh via polling re-hash loop** (no `notify` crate, no new deps). Serve the stale payload while a background rebuild runs; swap atomically.
- **Breaking artifact change accepted:** all existing artifacts/shards become invalid once (no fingerprint ⇒ stale), triggering a one-time full rebuild.

## Current gaps this plan fixes
- `artifacts/index/` and `artifacts/includes/{edges,closures}.bin` are write-only — no loaders exist.
- `reduced/` is trusted blindly (existence == validity); no manifest carries any fingerprint (all are pure counters, `crates/report/src/manifests.rs`).
- Shard checkpoints (`artifacts/shards/*.json`, `scheduler/src/engine.rs:122-134`) are keyed only by FileId — edits never invalidate them; `analyze --workers`/`optimize` reuse stale analyses forever.
- `tool ui` re-runs the entire pipeline on every startup and never reads/persists artifacts (`crates/cli/src/commands/ui.rs`).
- Two producers write incompatible content to `artifacts/index/`: `handle_index` merges full include-closure analyses; `IncrementalAnalyzer` miss-path parses only the TU body.
- Files outside repo discovery hash to `""` (`incremental.rs:80-85`) — changes to them never invalidate dependents.

## Architecture
New crate **`crates/pipeline` (`ariadne-pipeline`)** depending on `repo, build, graph, index, report, scheduler`; consumed by `cli` and `ui` (no cycles: cli→pipeline, ui→pipeline).

- `Fingerprint::inputs(files: sorted (FileId, content_hash)) → SHA-256`, extended with upstream stage fingerprint + `CURRENT_ANALYSIS_VERSION` ("1.0.0").
- Reuse `TuCacheFingerprint::compute` (`index/src/cache.rs:22-66`) as the per-TU key everywhere.
- `Pipeline::get_or_build_*` per stage returns `(value, StageReuse::{Loaded, Rebuilt})`; collect `RunStats { per-stage reuse, reused_tus, reanalyzed_tus }` for logging + `/api/status`.
- All artifact writes go through `ArtifactManager` with **atomic save** (write `*.tmp`, rename) so a crash mid-write leaves the old fingerprint-checked artifact intact.

## TU store unification (cache + shards → one store)
One persistent, content-keyed TU analysis store; retire the duplication:
- Shard file format becomes an envelope: `ShardCheckpoint { fingerprint: String /*combined TuCacheFingerprint hash*/, analysis: TuAnalysis }` at `artifacts/shards/{file_id.0}.json`. Missing/mismatched fingerprint ⇒ miss. Old plain-`TuAnalysis` shards fail deserialization ⇒ miss (one-time rebuild).
- All paths read/write this store first: single-thread `analyze` (replacing `artifacts/cache/cache.bin`), `index`, workers (`engine.rs` resume + new results), UI. Delete `save/load_cache_artifacts` usage and `artifacts/cache/` producers; keep `CacheManifest` fields folded into the index manifest reuse stats.
- **Canonical TU analysis semantics** = what `handle_index` produces today (TU + transitive closure parse, merged) so every producer writes identical content. Update `IncrementalAnalyzer` miss-path to use the shared closure-merge function.
- Fix the empty-hash hole: hash any file readable on disk in the closure (via its resolved path), not only repo-discovered files; mark unreadable files with a sentinel hash so they invalidate when they appear.
- Keep closure-keyed (not per-file-keyed) TU granularity this round — it exists, is tested by m12; per-file analysis granularity is noted as future work.

## Tasks (ordered)

1. **Manifest fingerprints + atomic writes (`crates/report`)**
   - Add `input_fingerprint: String` + `analysis_version: String` to every manifest struct in `manifests.rs`; add the missing `manifest.json` for `includes/` (new `IncludeManifest`).
   - All `save_*` take the fingerprint; all `load_*` return `None`/error-stale when `repository_root` or fingerprint mismatch.
   - Atomic tmp+rename in every save path.

2. **Include-graph loader (`crates/graph`)**
   - `load_include_graph(manager) -> Option<(IncludeGraph, closures)>` reading `edges.bin` + `closures.bin` with manifest fingerprint check; `build_include_graph` persists both + manifest on rebuild.

3. **`crates/pipeline` crate**
   - Workspace member; fingerprint helpers; `Pipeline` with `get_or_build_repository / build_contexts / include_graph / index / reduced / callgraph / load` wired to `ArtifactManager` loaders/savers and the shard store.
   - Index stage: per-TU shard-store lookup, parse misses with closure-merge semantics, parallel option (delegate to `scheduler::ProductionAnalyzer` when `--workers N`).
   - Reduced stage: reuse `get_or_build_reduced_model` logic, now fingerprint-checked.

4. **TU store unification (`scheduler`, `index`, cli)**
   - Envelope format in `engine.rs` resume/save; workers consult/update shard store; cleanup shards on successful reduce+persist.
   - Remove `artifacts/cache/` producers; repoint m12 tests to shard-store behavior.
   - Shared closure-merge `analyze_tu_full()` used by `handle_index`, `IncrementalAnalyzer`, workers.

5. **Rewire commands (`crates/cli`)**
   - `analyze`, `analyze --workers`, `index`, `includes`, `plan`, `analyze load`, `optimize` call `Pipeline::get_or_build_*`; print reuse stats ("N TUs reused, M reanalyzed; stages loaded: graph, reduced; rebuilt: index").
   - `optimize`/`plan` no longer unconditionally rebuild the include graph.

6. **UI artifact-first startup + watcher (`crates/ui`)**
   - On start: discovery hashes → fingerprint check → load fresh stages (skip parse entirely when warm); rebuild only stale stages with existing progress UI; persist callgraph + load results (callgraph save exists, never called).
   - Watcher task: tokio interval (default 2 s, `--watch-interval`, `--no-watch`) re-hashes discovered files; unchanged ⇒ nothing. Changed ⇒ recompute affected TU fingerprints (closures from persisted `closures.bin`), rebuild stale stages in background, swap `UiState` payload atomically under the existing `RwLock` (stale payload keeps serving meanwhile).
   - `/api/status` gains per-stage `loaded|rebuilt` + `last_refresh` so the frontend can show a "refreshing" indicator.

7. **Tests**
   - New `tests/integration/tests/m18_persistence_incremental_test.rs`: cold run persists; no-change rerun loads every stage with zero parses (assert reuse stats); edit one header ⇒ only dependent TU re-parsed, graph/reduced rebuilt; edit TU body ⇒ one TU re-parsed; old-format shard ⇒ treated stale, rebuilt; corrupted artifact (bad fingerprint) ⇒ rebuilt, not trusted.
   - Extend m12 for the unified store; update fixture expectations (fixture `artifacts/` become fingerprintless ⇒ tests rebuild once, no fixture regeneration needed unless asserts compare bytes).
   - UI test: server start with fresh artifacts reports all stages `loaded` via `/api/status`.

8. **Verify**: `cargo test --workspace`, `cargo clippy --workspace`; rebuild `cargo build --bin tool` and smoke-run `tool ui` on a fixture (warm start must skip parsing).

## Risks
- Breaking artifact format: one-time full rebuild everywhere; no migration needed (stale ⇒ rebuild is the migration).
- Large repos: watcher re-hash is O(repo bytes) every interval — mitigated by interval knob and skipping hashing while no clients are connected (optional, decide during impl).
- `ReducedSemanticModel::reduce` and `handle_index` must stay byte-compatible or fingerprint checks mask drift — enforced by the canonical shared analysis function (task 4).
- Watcher during active rebuild: guard with a "rebuild in progress" flag; collapse queued change detections into one pass.

## Out of scope
- True per-edge incremental patching of graph/reduced/load.
- Event-based (`notify`) watching; git-diff-driven fast path.
- Per-file (rather than per-TU) analysis cache granularity.
- Enabling the clang-worker backend (still `DefaultWorkerBackend`).
