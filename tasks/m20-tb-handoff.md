# T-B Handoff — Import edge kind + External outcome (M20.2 kind plumbing)

Kickoff: `plans/m20-scala-graph-kickoff.md` (D10, D10b). Base: T-A's commit. Bounded session: touch-set, scoped commands, commit on green.

## 1. Intent & Invariants

**Objective:** land the two builder-neutral seams so a resolved Scala import lands as an `Import` edge and an out-of-repo import as an `External` edge — with **zero C++ behavior change** (m18 22/22 + m00 byte-identical are the proof).

**Invariants:**
- IV-1 `DependencyResolver` gains a default method:
  ```rust
  fn directive_edge_kind(&self) -> Option<IncludeKind> { None }
  ```
  C++ does **not** override it (keeps today's `ownership_kind`/`style_kind`). Scala overrides → `Some(IncludeKind::Import)`.
- IV-2 `IncludeKind::Import` is the **last** variant (bincode index 3). Every exhaustive `match` on `IncludeKind` gains a deliberate arm; the report-kind tally in `graph.rs::generate_report` (~290-294) counts `Import` as a distinct kind; any C++-facing tally keeps its current numbers because C++ never produces `Import`.
- IV-3 In `builder.rs`, `directive_edge_kind()` is consulted **only** for `ResolveOutcome::Resolved` and `ResolveOutcome::Wildcard` edges. `Some(kind)` → use it; `None` → existing `ownership_kind(record)` path unchanged. The on-disk-fallback and `style_kind` paths are NOT touched.
- IV-4 **External split (D10b):** `ResolveOutcome::External` no longer shares the `Unresolved` on-disk-fallback branch. `External` ⇒ emit an edge with `destination: None`, `kind: IncludeKind::External`, `resolved_path: None`, and **do not** run `first_ondisk_include_candidate`. `Unresolved` keeps its exact current behavior (on-disk fallback → `style_kind`). Justification of C++ inertness (put in the code comment): `CppDependencyResolver::resolve_target` returns only `Resolved`/`Unresolved`; `External` is reserved for the Scala unresolved-package case (verified `lang-cpp/src/resolver.rs`).
- IV-5 No language word (`scala`, `import`, `.scala`, `cl.exe`) appears in `builder.rs` / `graph.rs` non-comment code (R2). The builder still selects plugins via `registry.detect_language` only.
- IV-6 Serde/bincode shape: `Import` is purely additive; no existing variant is reordered or renamed. `IncludeEdge.language` continues to carry `SourceLanguage::Scala` on Scala edges (already populated in M17-M18) — assert a Scala edge round-trips with `kind: Import`, `language: Scala`.

## 2. API contracts

```rust
// crates/lang/src/resolver.rs — one additive default method on DependencyResolver
fn directive_edge_kind(&self) -> Option<IncludeKind> { None }

// crates/lang-scala/src/resolver.rs (from T-A) — override
fn directive_edge_kind(&self) -> Option<IncludeKind> { Some(IncludeKind::Import) }

// crates/lang-api/src/types.rs — IncludeKind gains last variant
pub enum IncludeKind { Local, System, External, Import }
```

`builder.rs::edge_from_outcome` (~136), `resolved_edge` (~177), `style_kind` (~217), `ownership_kind` (~227): thread the resolver handle (it is already reachable from the loop's plugin) so `resolved_edge` can ask `directive_edge_kind()`; add the `External` branch before the `Unresolved` fallback.

## 3. Plan-Task (RGB)

- **Red:**
  - `builder.rs` unit test: a synthetic Scala-shaped catalog + `ScalaImportResolver` (or a minimal in-test resolver implementing `directive_edge_kind → Some(Import)`) drives `build_dependency_graph_with_unit_shaper` (D14 seam lands here — see §3.5) so a `import pkg.B` yields an edge `kind == Import`, `language == Scala`. A resolver returning `ResolveOutcome::External` yields `kind == External`, `destination == None`, and the on-disk fallback is NOT consulted (assert via a raw target that WOULD hit disk if fallback ran).
  - `graph.rs` unit test: `generate_report` tallies an `Import`-kind edge under a new/existing counter without changing `Local/System/External` counts for a C++-only graph.
  - Bincode/serde shape guard: a C++-only graph's `graph.bin`/`report.json` bytes are unchanged (m00 does the real check; a crate-local test asserting `Import`'s discriminant is 3 and old variants 0/1/2 guards the index discipline cheaply).
- **Green:** implement IV-1..IV-4 minimally. Prefer reusing the existing plugin handle in the BFS loop over threading a new argument through the whole builder; keep helper fns < 100 lines.
- **D14 seam (this task's builder entry change):** add `pub fn build_dependency_graph_with_unit_shaper(repo_path: &Path, registry: &LanguageRegistry, shaper: &dyn UnitShaper) -> Result<IncludeGraph>`; refactor `build_dependency_graph` to delegate with `registry.unit_shaper_for()` so the C++ call path is byte-identical. This is the hook the m20 suite uses to inject `ScalaUnitShaper` without any language name in `graph`.
- **Refactor / Hygiene:** the External split should read as two clear arms, not a flag; no `match kind` with a wildcard swallow that could hide a future variant — every `match` names `Import` explicitly. `python tools/hygiene_check.py` exit 0.

## 4. Touch-set

`crates/lang/src/resolver.rs` · `crates/lang-api/src/types.rs` (variant + any exhaustive match the variant forces there) · `crates/lang-scala/src/resolver.rs` (override) · `crates/graph/src/builder.rs` (kind apply, External arm, `build_dependency_graph_with_unit_shaper`) · `crates/graph/src/graph.rs` (report tally arm) · **no** changes to `cluster`/`transform`/`validate`/`pipeline`.

## 5. Scoped commands

```powershell
cargo test -p ariadne-lang -p ariadne-lang-api -p ariadne-lang-scala
cargo test -p ariadne-graph
cargo test -p ariadne-integration-tests --test m18_cpp_plugin_parity_test
cargo test -p ariadne-integration-tests --test m00_golden_baseline_test
cargo test -p ariadne-integration-tests --test m3_include_graph_test
cargo test -p ariadne-integration-tests --test m17_plugin_seam_test
python tools/hygiene_check.py
```

## 6. Clauses

- **m00 golden drift = STOP.** Any byte change to `tests/golden/**` means a C++ behavior slipped — halt and report, do not re-pin.
- `Import` must be the LAST enum variant (index 3); verify with the discriminant guard test before committing.
- Do not touch cycle detection here (T-C owns it). Targeted diffs; commit on green (`feat(m20.2): Import kind + External outcome`).
