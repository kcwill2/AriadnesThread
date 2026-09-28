# Multi-Language Architecture — Implementation Plan

Status: **DRAFT for approval** (no source changed yet)
Date: 2026-09-24 · Baseline: `664e014` + uncommitted WIP · `cargo check --workspace --all-targets` = green (warnings only)

---

## 0. Inputs, assumptions, decision points

**Confirmed:** reference second language = **Scala**. The tree is already partly prepared: `crates/lang` (untracked WIP) contains unit-tested plugin traits (`LanguagePlugin`, `LanguageRegistry`, `LanguageDetector`, `DependencyResolver`, `SemanticAnalyzer`, `SourceExtractor`, `DependencyRewriter`, `CompilerValidator`, `ClusteringConstraints`), and `crates/repo/src/types.rs` already has `FileKind::{ScalaSource, ScalaScript, BuildDefinition}` plus `SourceLanguage { Cpp, Scala, Other }` / `from_file_kind` / `FileKind::language()`.

**Architectural refinement:** Rename "Include Graph" to "Import Graph" since superfluous imports affect Java, Scala, and other languages, not just C++. This makes the architecture truly language-agnostic and relevant for all modern languages with import systems.

**Assumptions (correct any of these — they are restated as decision points in §8):**

| # | Assumption | Why it is safe to start here |
|---|---|---|
| A1 | Milestone scope = **behaviour-preserving boundary refactor + Scala *analysis* vertical slice** (discovery → build → dependency graph → semantic index → reduced model → load metric → reports / Graph UI). Scala *transform + validation* is specified here but executed in a gated later milestone (M23). | Analysis is the part that must be language-agnostic first; transforms additionally depend on the modeling table §3.4 and on a real Scala toolchain. |
| A2 | Plugins are **compile-time Rust crates** (`crates/lang-cpp`, `crates/lang-scala`) registered statically. No `libloading`/WASM plugin host yet. | Matches build reality (single binary, offline CI); the trait boundary is what makes a dynamic host possible later. |
| A3 | Scala semantic depth = **built-in lightweight Scala parser** (same trust level as today's in-process `CppSemanticAnalyzer`, which is a hand-written tokenizer + `ParserCtx` in `crates/index/src/analyzer.rs`), with an optional toolchain-backed analyzer behind a capability probe (mirroring `check_clang_available()` in `tools/clang-worker/src/main.rs`). | `clang`, `g++`, `scalac`, `sbt` are **not on PATH** in this environment (`Get-Command` returned nothing), so no test may require them. |
| A4 | JSON artifact shapes stay backward compatible: additive fields with `#[serde(default)]`; existing `#include`/`header` vocabulary in manifests is preserved. Neutral vocabulary is introduced only in *new* API surfaces. | The **15** integration suites in `tests/integration/tests/` (`m0…m7`, `m10…m16`) deserialize these manifests. |
| A5 | `ariadne-cli` is the only crate that knows *which* plugins exist; every other crate receives `&dyn` plugin capabilities as parameters. | Keeps language lists out of core crates permanently. |

**Scope guard:** the plan does **not** rewrite `tools/clang-worker` (it becomes the C++ plugin's backend, unchanged protocol) and does **not** change `Plan.md` milestone semantics M0–M16.

---
## 1. Current state: what a new language must touch today

Verified in the working tree (each row is a real call site, not a guess):

| # | Coupling point | Location (verified in the current tree) | Effect for a non-C++ language |
|---|---|---|---|
| 1 | The analysis unit is a C++ translation unit, and the fallback unit shaper only recognises C++ extensions | `crates/build/src/discovery.rs::infer_default_discovery` (lines 165–217: `CompilationContext::new("Default\|x64", "cl.exe")`, `ext == "cpp" \|\| "cc" \|\| "cxx" \|\| "c"`, `/c` in `cmd_args`), `infer_context_name` (`DEBUG`/`_DEBUG`/`NDEBUG`), `crates/build/src/types.rs::TranslationUnit` | A Scala repo yields **0 translation units** ⇒ every later stage has nothing to do |
| 2 | The build database shape is clang's | `crates/build/src/discovery.rs::{ClangCompileCommandEntry, parse_compile_commands_json, parse_args}` (C++ flags `-D`, `-I`, `-std=`) | Nothing reads `build.sbt` / `pom.xml`, so no context is ever produced |
| 3 | The only dependency-directive parser is the `#include` parser | `crates/graph/src/resolver.rs::parse_include_directives`, `RawIncludeDirective`, `IncludeResolver::{resolve_directive, resolve_file_includes}` | `import a.b.C` produces no edges |
| 4 | Graph construction is a BFS over the *include* closure using that resolver | `crates/graph/src/builder.rs::build_include_graph` (free function; `IncludeResolver::new(…).with_repo_discovery(…).with_context(…)`) | No seam to feed non-`#include` edges into `IncludeGraph` |
| 5 | Aggregation-unit query is header-named and has exactly one consumer | `crates/graph/src/graph.rs::map_header_consuming_tus`, called only from `crates/cluster/src/planner.rs:57` | No equivalent for package/module-based languages |
| 6 | Semantic analysis is one hardcoded in-process C++ parser; the worker seam exists but both its implementations are C++ | `crates/index/src/analyzer.rs::CppSemanticAnalyzer::{analyze_source, analyze_tu}` (tokenizer + `ParserCtx`); `crates/scheduler/src/worker.rs::{WorkerBackend, DefaultWorkerBackend, ClangWorkerBackend}` (the latter spawns `clang-worker` with `-D`/`-I` args); `tools/clang-worker/src/main.rs` (WIP, `check_clang_available`) | A `.scala` file can be discovered but never analysed |
| 7 | The index *orchestration* assumes a header closure and merges per-file analyses | `crates/cli/src/commands/index.rs::handle_index` (walks `resolve_file_includes` transitively, then `CppSemanticAnalyzer::analyze_source` per file in the closure) | Scala has no include closure, so the merge loop is meaningless |
| 8 | Planner iterates every graph file and names its output `*_extracted.h` | `crates/cluster/src/planner.rs::plan` (line 57 `map_header_consuming_tus`, line 97 `path.with_file_name(format!("{}_extracted.h", record.name))`) | No plan-generation path for another language |
| 9 | Extraction emits C++ text | `crates/transform/src/lib.rs::HeaderExtractor` (`GeneratedHeader{ pragma_once, required_includes }`, `format!("#include \"{}\"", …)`) | No generated-artifact path for another language |
| 10 | Apply mutates `#include` text; validation only checks file existence and cannot express "toolchain unavailable" | `crates/validate/src/lib.rs::TransactionApplier::apply` (`content.push_str("#pragma once\n\n")`, `content.replace(&format!("#include \"{}\"", …))`), `TierValidator::{validate_structural, validate_generated_headers, validate_affected_tus}`, `ValidationStatus { Pass, Fail(String) }` | No pluggable rewrite; a missing Scala compiler can only be reported as `Fail` |
| 11 | `ariadne-lang` traits exist and are unit-tested, but **no production code calls them** | `crates/lang/src/{lib,registry,detector,analyzer,resolver,extractor,rewriter,validator,constraints}.rs`; `LanguageRegistry::plugin_for` appears only in `crates/lang/src/registry.rs` tests | The scaffold is dead code: every stage is still hardcoded |
| 12 | Dependency direction forbids the naive fix | `crates/lang/Cargo.toml` → `ariadne-index`, because `lang` traits take/return `ariadne_index::{SymbolId, SymbolRecord, TuAnalysis}` defined in `crates/index/src/types.rs` | `index` cannot consume `lang` today ⇒ cycle |

**Already language-neutral — do not "generalize" it:** `crates/repo/src/types.rs` (`FileKind::ScalaSource`/`ScalaScript`/`BuildDefinition`, `SourceLanguage { Cpp, Scala, Other }`, `SourceLanguage::from_file_kind`, `FileKind::language()`, `is_analyzable()` which already includes `ScalaSource`), `crates/repo/src/policy.rs`, `crates/graph/src/reduced.rs::reduce` (ownership comes from `owning_file`/definition, no extension test), `crates/graph/src/load.rs::compute_load` (works on graph closure + symbol closure only), `crates/cluster/src/scc.rs`.

**Consequence today:** adding a language = edits in ≥ 6 crates + 2 binaries.
**Target after this plan:** new language = **1 new crate + 1 registry entry + fixtures/tests**, with **0 edits** to `index`, `graph`, `cluster`, `transform`, `validate`, `report`, `scheduler` (review rule R1, §7).

---

## 2. Goals, non-goals, success criteria

**Goals**
1. `index`, `graph`, `cluster`, `transform`, `validate`, `report`, `scheduler`, `build` contain **no language names** (no `"cl.exe"`, no `.h` suffix tests, no `#include` literals) — the only C++ vocabulary lives in `crates/lang-cpp`.
2. Pipeline stages dispatch through the existing `ariadne-lang` trait layer (`LanguagePlugin` / `LanguageRegistry`), which today is unused.
3. Scala analysis works end-to-end on a fixture repo with **no Scala toolchain installed**.
4. Incremental cache correctness is preserved per language (`CURRENT_ANALYSIS_VERSION` + plugin fingerprint + per-unit fingerprint).

**Non-goals (this plan):** Scala transforms (M23), dynamic plugin loading (M24), a third language, UI redesign, changes to `Plan.md` milestones M0–M16.

**Success criteria (machine-checkable)**
- `cargo test --workspace` green at the end of **every** milestone.
- Byte-identical C++ artifacts: golden snapshot of `artifacts/**.json` for `tests/fixtures/basic` and `tests/fixtures/expensive` is unchanged across M17–M18 (except additive fields listed in §3.5).
- "Minimal change rule" test `m17_plugin_seam_test.rs::new_language_touches_only_plugin_crate` (§6.4).
- `Select-String -Path crates/{index,graph,cluster,transform,validate,report}/src/*.rs -Pattern 'clang|\.h"|include'` returns **0** production matches after M18.

---

## 3. Target architecture

### 3.1 Crate layout and dependency direction (no cycles)

```
                          ariadne-repo            (FileId, FileKind, FileRecord)
                               ^
                          ariadne-lang-api        (NEW: shared neutral vocabulary)
                            /    |     \
                     ariadne-lang  ariadne-index  ariadne-build
                       (traits+registry)   |
        /-------------------/------------------\
   ariadne-graph   ariadne-cluster   ariadne-transform   ariadne-validate
        \                  |               /
                 ariadne-lang-cpp (NEW)   ariadne-lang-scala (NEW, M19+)
                              \      |      /
                              ariadne-cli   (only crate that enumerates plugins)
```

Rule: **plugin crates depend inward; core crates never depend on plugin crates.**

**Breaking coupling #12 (the `lang -> index` cycle) — the key structural move:**
`crates/index/src/types.rs` (265 lines, 12 public types: `SymbolId`, `DeclId`, `SymbolKind`, `DependencyKind`, `FileLocation`, `SymbolDecl`, `SymbolRecord`, `SemanticEdge`, `CallSite`, `SymbolReference`, `TuAnalysis`, `IndexResult`) is split:

- Move `SymbolId`, `DeclId`, `SymbolKind`, `DependencyKind`, `FileLocation`, `SymbolDecl`, `SymbolRecord`, `SemanticEdge`, `CallSite`, `SymbolReference`, `TuAnalysis` → new crate **`ariadne-lang-api`** (`crates/lang-api/src/types.rs`), unchanged serde attributes (artifact JSON is byte-compatible).
- `IndexResult` stays in `ariadne-index` (it is an index product).
- `crates/index/src/types.rs` becomes `pub use ariadne_lang_api::types::*;` + `IndexResult`. All existing paths (`ariadne_index::types::SymbolId`, and the crate-root `pub use types::*`) keep resolving, so **no call-site churn and no serde change**.
- `crates/lang/Cargo.toml`: replace `ariadne-index` with `ariadne-lang-api`; `lang` then legally feeds `index`.

### 3.2 `ariadne-lang-api` — the neutral vocabulary (new crate)

Files: `crates/lang-api/Cargo.toml`, `src/lib.rs`, `src/types.rs` (moved), `src/identity.rs`, `src/modeling.rs`, `src/capability.rs`.

```rust
// crates/lang-api/src/identity.rs — REUSE the enum that already exists, do not invent a second one.
// `SourceLanguage { Cpp, Scala, Other }` + `SourceLanguage::from_file_kind(FileKind)` already live in
// crates/repo/src/types.rs (that file already classifies .scala/.sc/build files).
pub use ariadne_repo::SourceLanguage as LanguageId;

/// Additive changes to `ariadne_repo::SourceLanguage` (the only `repo` edit in the plan):
///   #[derive(..., PartialOrd, Ord, Default)]   // ordering for stable report grouping
///   pub enum SourceLanguage { #[default] Cpp, Scala, Other }
/// Do NOT add #[serde(rename_all)]: existing artifacts already store the PascalCase
/// variant names ("Cpp", "Scala", "Other"), and `#[default] Cpp` makes every
/// pre-existing JSON artifact read unchanged.
pub fn language_of_file_kind(kind: FileKind) -> LanguageId { SourceLanguage::from_file_kind(kind) }

// crates/lang-api/src/modeling.rs  — replaces ".h"/TU assumptions everywhere
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitKind { TranslationUnit, SourceFile }          // C++: TU; Scala: source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationKind { Header, PackageObject, None }  // what "owns" symbols & is re-loaded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardKind { PragmaOnce, IncludeGuard, None }     // was: hardcoded #pragma once

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageModeling {
    pub language: LanguageId,
    pub unit: UnitKind,
    pub aggregation: AggregationKind,
    pub guard: GuardKind,
    /// Does an aggregation artifact physically re-aggregate dependency code (C++ header)
    /// or only re-export/nominalise it (Scala package object / given precedence)?
    pub aggregation_reexports: bool,
    /// How this language must treat dependency cycles for planning.
    pub cycle_policy: CyclePolicy,
    /// Path components that mark the external (frozen) boundary; replaces is_third_party_path(".h").
    pub external_path_markers: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CyclePolicy { Reject, TolerateWithinPackage, Tolerate }

// crates/lang-api/src/capability.rs — gates each pipeline stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageCapabilities {
    pub discovery: bool, pub dependency_graph: bool, pub semantic_index: bool,
    pub load_metric: bool, pub planning: bool, pub extraction: bool,
    pub rewriting: bool, pub validation: bool,
    pub requires_toolchain: bool,      // false => works offline (A3)
}
```


### 3.3 Trait deltas in `ariadne-lang` (small — the scaffold is already neutral)

The existing traits (`detector.rs` `SourceFileKind`+`LanguageDetector`, `resolver.rs` `RawDepEdge`+`ResolutionContext`+`DependencyResolver`, `analyzer.rs` `SemanticAnalyzer`, `extractor.rs` `SymbolBody`+`GeneratedSourceFile`+`SourceExtractor`, `rewriter.rs` `RewriteSpec`+`DependencyRewriter`, `validator.rs` `CompilerValidator`, `constraints.rs` `ClusteringConstraints`) are already language-neutral and already documented for Scala. They are extended, not replaced:

```rust
// crates/lang/src/registry.rs — additions to the existing aggregate trait
pub trait LanguagePlugin: Send + Sync {
    // ... existing: detector/analyzer/resolver/constraints/extractor/rewriter/validator ...

    fn id(&self) -> LanguageId;                                   // NEW  dispatch key
    fn modeling(&self) -> LanguageModeling;                       // NEW  replaces ".h"/TU rules
    fn capabilities(&self) -> LanguageCapabilities;               // NEW  stage gating
    /// Ingredient of every cache fingerprint: bump when the plugin's parser changes.
    fn analysis_fingerprint(&self) -> String { "v1".into() }      // NEW
    /// Unit shaping (build stage). None = use the built-in CMake/VS/compile_commands shaper.
    fn shaper(&self) -> Option<&dyn UnitShaper> { None }          // NEW
}

// crates/lang/src/registry.rs — registry lookups the pipeline needs
impl LanguageRegistry {
    pub fn plugin_for_id(&self, id: LanguageId) -> Option<&dyn LanguagePlugin>;      // NEW
    pub fn languages(&self) -> Vec<LanguageId>;                                       // NEW
    pub fn detect_language(&self, path: &Path) -> Option<LanguageId>;                 // NEW
}

// crates/lang/src/unit.rs — NEW file (UnitDescriptor itself lives in lang-api)
pub trait UnitShaper: Send + Sync {
    /// Build/manifest files this language recognises:
    /// ["CMakeLists.txt", ...] | ["build.sbt", "build.sc", "pom.xml"].
    fn build_markers(&self) -> Vec<&'static str>;
    /// Compiler binary name used for context/measurements; may legitimately be absent.
    fn default_compiler(&self) -> &str;
    /// Language-specific: which discovered files become analysis units, and their context.
    fn shape_units(&self, repo_root: &Path, files: &[FileRecord]) -> Vec<UnitDescriptor>;
}

// crates/lang/src/resolver.rs — one added method: language-specific target resolution
pub trait DependencyResolver: Send + Sync {
    fn resolve_edges(&self, file_id: FileId, rel_path: &Path, source: &str,
                     ctx: &ResolutionContext<'_>) -> Vec<RawDepEdge>;          // existing

    /// Maps a raw directive to a repository file. Default: unresolved.
    /// C++: include-dir + basename search. Scala: package -> directory mapping.
    fn resolve_target(&self, edge: &RawDepEdge, catalog: &FileCatalog,
                      ctx: &ResolutionContext<'_>) -> ResolveOutcome {          // NEW
        let _ = (edge, catalog, ctx); ResolveOutcome::Unresolved
    }
}

// crates/lang/src/validator.rs — toolchain probing so tier 3 can be Skipped, not Fail
pub trait CompilerValidator: Send + Sync {
    fn validate_compile(&self, files: &[PathBuf], args: &[String]) -> Result<(), String>; // existing
    fn toolchain_status(&self) -> ToolchainStatus { ToolchainStatus::Unknown }             // NEW
}

// crates/lang-api/src/catalog.rs — NEW (neutral lookup surface for resolve_target)
pub struct FileCatalog { /* by_rel_path, by_basename, by_module_path */ }
pub enum ResolveOutcome { Resolved(FileId), External, Wildcard(Vec<FileId>), Unresolved }
```

Also new in `lang-api` (`unit.rs`): `UnitDescriptor { file_id, language, unit_kind, context_name, aggregation_target: Option<FileId>, build_spec: Vec<String> }`, and `ToolchainStatus { Available(String), Missing, Unknown }` in `capability.rs`.

Two facts make M18 cheaper than it looks:

1. The existing `lang::analyzer::SemanticAnalyzer::analyze(&self, file_id: FileId, source: &str, context: &str) -> TuAnalysis` has **exactly** the signature of `index::CppSemanticAnalyzer::analyze_source(tu_file_id, source_code, context_name)`, so the C++ plugin adapter is a three-line `impl` — no parser rewrite.
2. `crates/scheduler/src/worker.rs` **already has a backend seam**: `trait WorkerBackend { fn execute(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> }` with `DefaultWorkerBackend` (in-process) and `ClangWorkerBackend` (spawns `clang-worker`). M18 turns `DefaultWorkerBackend` into `PluginWorkerBackend { registry }` that picks the analyzer from `plugin_for_id(language_of_file_kind(...))`, keeping `ClangWorkerBackend` as the C++-specific implementation inside `lang-cpp`. Nothing else in `scheduler` changes.
3. `SourceLanguage::Other` deliberately has **no plugin**: the registry must return `None` for it, and `handle_index` then reports "unhandled language" instead of silently analysing C++ syntax.


### 3.4 Lifecycle mapping — C++ vs Scala (this table is the refactor contract)

| Concept | C++ (existing → moves into `lang-cpp`) | Scala (new `lang-scala`) |
|---|---|---|
| Analysis unit | `TranslationUnit` (`.cpp` + include closure) | one `.scala` file = one unit (`UnitKind::SourceFile`) |
| Aggregation artifact | header `.h` (`AggregationKind::Header`, `aggregation_reexports = true`) | package clause / package object (`PackageObject`, `aggregation_reexports = false`) |
| Duplicate-definition guard | `#pragma once` / include guard | JVM + package uniqueness ⇒ `GuardKind::None` |
| Dependency directive | `#include "x.h"` / `<x>` | `import a.b.C`, `import a.b._`, `export` |
| External boundary | angle system include, `third_party/`, `vendor/` | unresolved package prefix, `lib/`, `target/scala-*/`, managed deps of `build.sbt` |
| Cycle policy | `CyclePolicy::Reject` (TU cycle = defect) | `CyclePolicy::TolerateWithinPackage` (mutually recursive classes are legal) |
| "Collateral load" | symbols from transitively included headers you do not use | symbols pulled into the compile-time closure by importing a package aggregation |
| Symbol ownership | header owns declarations, TU owns definitions | file owns top-level `class/trait/object/enum/given`; `package` clause owns the namespace |
| Build manifest | `CMakeLists.txt`, `.vcxproj`, `compile_commands.json` | `build.sbt`, `project/build.properties`, `build.sc`, `pom.xml` |
| Toolchain | `cl.exe` fallback in `infer_default_discovery`, real compiler from `compile_commands.json`, optional `clang-worker` sidecar (JSON request/response protocol unchanged) | `scalac` / `scala-cli` / `sbt` **optional**; absent ⇒ built-in parser + `Skipped` validation tier |
| Atomic symbol groups | none beyond policy | companion object ↔ class, `sealed trait` ↔ direct subtypes (`ClusteringConstraints::colocated_groups`) |
| Extraction target naming | `{symbol}_extracted.h` | `{symbol}.scala` in the same package dir (planner asks the plugin — M23) |

### 3.5 Artifact & versioning changes (additive only — assumption A4)

| Artifact | Change | File |
|---|---|---|
| `artifacts/index/*.json` | `TuAnalysis` gains `language: SourceLanguage` (`#[serde(default)]` ⇒ old artifacts read as `Cpp`) | moved type `crates/lang-api/src/types.rs` |
| `artifacts/cache/cache.json` | `TuCacheFingerprint` gains `language: SourceLanguage` + `plugin_fingerprint: String` (both feed the existing sha2 digest); `IncrementalCache::entries` stays keyed `(FileId, context_name)` | `crates/index/src/cache.rs` |
| cache epoch | `CURRENT_ANALYSIS_VERSION: &str = "1.0.0"` → `"2.0.0"` — one line, invalidates every mixed-version cache | `crates/index/src/cache.rs:9` |
| `artifacts/repository/manifest.json` | `FileRecord` gains `language: LanguageId` | `crates/repo/src/types.rs` (WIP already adds `FileKind::language()`) |
| `artifacts/build/manifest.json` | `CompilationContext` gains `language: SourceLanguage`; `TranslationUnit` gains `language`; `CompileCommand` unchanged (there is no `BuildContext` type — these are the real names in `crates/build/src/types.rs`) | `crates/build/src/types.rs` |
| `artifacts/includes/manifest.json` | `IncludeEdge` gains `language`; `IncludeKind { Local, System, External }` gains **one** additive variant `Import`; `External` is reused for unresolved/out-of-repo imports (`"Local"`/`"System"` strings unchanged) | `crates/graph/src/types.rs` |
| `artifacts/load/manifest.json` | `LoadMetric` gains `aggregation_target`, `language`; field name `include_load` kept for compatibility | `crates/graph/src/load.rs` |
| `artifacts/plans/*.json` | `OptimizationPlan` gains `language` | `crates/cluster/src/plan.rs` |
| `artifacts/transactions/*.json` | `ValidationStatus` gains `Skipped(String)` | `crates/validate/src/lib.rs` |

---

## 4. Crate / dependency changes

New workspace members (`Cargo.toml` `members` list) and their dependency edges:

| Crate | Package name | Depends on | Purpose |
|---|---|---|---|
| `crates/lang-api` | `ariadne-lang-api` | `ariadne-repo`, `serde`, `anyhow` | Moved index value types + `LanguageId`, `LanguageModeling`, `LanguageCapabilities`, `UnitDescriptor`, `FileCatalog`, `ResolveOutcome`, `ToolchainStatus` |
| `crates/lang-cpp` | `ariadne-lang-cpp` | `lang-api`, `lang`, `repo`, `index`, `serde_json` | `CppPlugin` + `CppUnitShaper`; the `#include` parser and `IncludeResolver` logic moved out of `crates/graph/src/resolver.rs`; the `CppSemanticAnalyzer` tokenizer/`ParserCtx` moved out of `crates/index/src/analyzer.rs`; `ClangWorkerBackend` moved out of `crates/scheduler/src/worker.rs` |
| `crates/lang-scala` | `ariadne-lang-scala` | `lang-api`, `lang`, `repo`, `serde`, `anyhow` | `ScalaPlugin`: detector, `ScalaUnitShaper`, import resolver, `ScalaSourceParser` (semantic analyzer), constraints (companion/sealed), later extractor/rewriter/validator |

Modified `Cargo.toml`s: `crates/lang` (`ariadne-index` → `ariadne-lang-api`), `crates/index` (+ `ariadne-lang-api`), `crates/build` (+ `ariadne-lang-api`, + `ariadne-lang`), `crates/graph` (+ `ariadne-lang-api`, + `ariadne-lang`), `crates/cluster`, `crates/transform`, `crates/validate` (+ `ariadne-lang-api`), `crates/cli` (+ `ariadne-lang`, `ariadne-lang-cpp`, `ariadne-lang-scala`), `crates/scheduler` (+ `ariadne-lang`), `crates/report` (+ `ariadne-lang-api`), root `[workspace] members`.

**Deliberate non-dependency:** `graph`, `cluster`, `transform`, `validate` must **not** depend on `lang-cpp`/`lang-scala`; they receive `&dyn` capabilities from `cli`/`scheduler` (assumption A5). `ToolchainStatus` and `ResolveOutcome` live in `lang-api` so core crates can pattern-match results without importing a plugin.


---

## 5. Milestones, per-file change list, acceptance per milestone

Numbering continues `Plan.md` (M0–M16 exist). Every milestone ends with `cargo test --workspace` green.

### M17 — Language API seam (pure refactor, zero behaviour change)

**Objective:** the plugin vocabulary exists and is wired, while C++ still runs its current code paths.

| File | Change |
|---|---|
| `Cargo.toml` (root) | add `"crates/lang-api"`, `"crates/lang-cpp"` to `members` |
| `crates/lang-api/Cargo.toml`, `src/lib.rs` | new crate (deps: `ariadne-repo`, `serde`) |
| `crates/lang-api/src/types.rs` | **moved verbatim** from `crates/index/src/types.rs`: `SymbolId`, `DeclId`, `SymbolKind`, `DependencyKind`, `FileLocation`, `SymbolDecl`, `SymbolRecord`, `SemanticEdge`, `CallSite`, `SymbolReference`, `TuAnalysis`. Additive: `TuAnalysis.language: SourceLanguage` (`#[serde(default)]`), `SymbolKind::{Trait, Object, Module}` (never emitted by the C++ parser, so existing readers and artifacts are unaffected) |
| `crates/lang-api/src/identity.rs` | `pub use ariadne_repo::SourceLanguage as LanguageId;` + `language_of_file_kind(FileKind)` — **no new enum**: `SourceLanguage { Cpp, Scala, Other }`, `from_file_kind` and `FileKind::language()` already exist in `crates/repo/src/types.rs`. Additive repo edits: `#[derive(PartialOrd, Ord, Default)]` with `#[default] Cpp`, and `FileRecord.language: SourceLanguage` (`#[serde(default)]`) |
| `crates/lang-api/src/{modeling,capability,unit,catalog}.rs` | as specified in §3.2 / §3.3 |
| `crates/index/src/types.rs` | becomes `pub use ariadne_lang_api::types::*;` + the `IndexResult` that stays behind |
| `crates/index/Cargo.toml` | `+ ariadne-lang-api` |
| `crates/lang/Cargo.toml` | `- ariadne-index` `+ ariadne-lang-api` (breaks coupling #12) |
| `crates/lang/src/lib.rs` | `+ pub mod unit; pub use unit::UnitShaper;` and `pub use ariadne_lang_api::*;` so `ariadne_lang::SymbolId` keeps working for existing users |
| `crates/lang/src/registry.rs` | add `id()`, `modeling()`, `capabilities()`, `analysis_fingerprint()`, `shaper()` to `LanguagePlugin`; add `plugin_for_id`, `languages`, `detect_language`; keep `plugin_for`/`detect` (already covered by its tests) |
| `crates/lang/src/resolver.rs` | add `resolve_target` default method |
| `crates/lang/src/validator.rs` | add `toolchain_status()` |
| `crates/lang/src/constraints.rs` | no change (already Scala-aware) |
| `tests/integration/tests/m17_plugin_seam_test.rs` | new (§6.3) |

Acceptance: workspace compiles; M17 tests pass; the 15 pre-existing suites are **unmodified and green**; golden artifact snapshot for `tests/fixtures/basic` (taken as task M17.0) is byte-identical afterwards.


### M18 — C++ becomes a plugin (behaviour-preserving)

**Objective:** delete coupling rows #3–#10 of §1 by dispatching through the registry, with the C++ implementation *relocated* into `ariadne-lang-cpp`.

Relocations (code move, not rewrite):

- `crates/graph/src/resolver.rs` → `crates/lang-cpp/src/{directive_parser.rs, resolver.rs}`: `parse_include_directives` / `RawIncludeDirective` become the directive parser, and `IncludeResolver::{resolve_directive, resolve_file_includes, normalize_path}` becomes `CppDependencyResolver` implementing `lang::DependencyResolver` (`resolve_edges` = parse, `resolve_target` = today's `resolve_directive` search logic). `ariadne-graph` keeps `#[deprecated]` re-export shims so `m3`/`m12`/`m13` suites keep compiling during the transition, and `scheduler/src/incremental.rs:4` (which imports `ariadne_graph::IncludeResolver`) switches to the plugin.
- `crates/index/src/analyzer.rs`: keep the crate neutral. The tokenizer/`ParserCtx`/`CppSemanticAnalyzer` move to `crates/lang-cpp/src/analyzer.rs` with a three-line `impl lang::SemanticAnalyzer for CppSemanticAnalyzer { fn analyze(..) { Self::analyze_source(..) } }`; `index::analyzer` re-exports them as `#[deprecated]` for one milestone. `tools/clang-worker` keeps its JSON `WorkerRequest`/`WorkerResponse` protocol unchanged — it becomes the C++ plugin's optional backend.
- `crates/scheduler/src/worker.rs`: `DefaultWorkerBackend` (which calls `CppSemanticAnalyzer::analyze_source` directly) becomes `PluginWorkerBackend { registry: &LanguageRegistry }` that dispatches on the unit's language; `ClangWorkerBackend` moves to `lang-cpp`. `queue.rs`, `shard.rs`, `engine.rs` are untouched.

Dispatch rewiring (core crates stop containing C++):

| File | Site (current name) | Change |
|---|---|---|
| `crates/build/src/discovery.rs` | `BuildDiscoverer::discover` (compile_commands candidates then `infer_default_discovery`), `infer_default_discovery` (`"cl.exe"`, `cpp/cc/cxx/c`, `Default\|x64`), `infer_context_name` | `with_unit_shaper(Option<&dyn UnitShaper>)`: if a shaper's `build_markers()` exist in the root, units come from `shape_units()`; otherwise today's two paths run unchanged. The C++ literal set moves into `lang-cpp::CppUnitShaper` |
| `crates/build/src/types.rs` | `CompilationContext`, `TranslationUnit`, `CompileCommand` | additive `language: SourceLanguage` (§3.5); no shape change otherwise |
| `crates/cli/src/commands/index.rs` | `handle_index` (closure walk via `resolve_file_includes` + `CppSemanticAnalyzer::analyze_source` merge), `get_or_build_reduced_model` | add `handle_index_with_registry(&path, &registry)`: per unit, `plugin.resolver()` drives the dependency walk and `plugin.analyzer().analyze(..)` produces each file's `TuAnalysis`; `handle_index(..)` keeps its signature and delegates with `default_registry()` so all existing callers/tests are untouched |
| `crates/graph/src/builder.rs` | `build_include_graph(repo_path)` (BFS + `IncludeResolver`) | add `build_dependency_graph(repo_path, &registry)`; the BFS becomes generic over `&dyn DependencyResolver` (`resolve_edges` + `resolve_target(&FileCatalog)`), `External` outcome comes from `ResolveOutcome::External` / `LanguageModeling::external_path_markers`; `build_include_graph` keeps delegating with the C++ resolver |
| `crates/graph/src/graph.rs` | `map_header_consuming_tus` | add neutral alias `map_aggregation_consumers` (thin wrapper, zero logic change) and keep the old name for one milestone |
| `crates/cluster/src/planner.rs` | `plan` (line 57 `map_header_consuming_tus`, line 97 `format!("{}_extracted.h", record.name)`) | new fields on `OptimizationPlanner`: `aggregation_filter: &dyn Fn(&Path) -> bool` (default: every file, as today), `target_path_for: &dyn Fn(&Path, &str) -> PathBuf` (default: `{name}_extracted.h`), `constraints: Option<&dyn ClusteringConstraints>`, `cycle_policy` — `crate::scc` stays as-is |
| `crates/validate/src/lib.rs` | `ValidationStatus { Pass, Fail(String) }`, `TierValidator::*` | **M18 adds only** `ValidationStatus::Skipped(String)` + `format_output` handling + `CompilerValidator::toolchain_status()`; `TransactionApplier::apply` (the `#pragma once` / `content.replace("#include …")` block) is generalized in **M23**, because only Scala transforms need it |
| `crates/index/src/cache.rs` | `CURRENT_ANALYSIS_VERSION`, `TuCacheFingerprint::compute`, `TuCacheEntry::is_valid` | fingerprint gains `language` + `plugin_fingerprint`; epoch bump; `validate_and_get` unchanged |
| `crates/scheduler/src/incremental.rs` | `IncrementalAnalyzer::analyze` (uses `CppSemanticAnalyzer` + `IncludeResolver`), `TuCacheFingerprint::compute` call site | analyzer + resolver come from the registry (`PluginWorkerBackend`, `CppDependencyResolver`); header-dependency invalidation (`test_header_modification`) becomes the C++ plugin's `dependents_of()` contribution |
| `crates/scheduler/src/worker.rs` | `DefaultWorkerBackend::execute` | becomes `PluginWorkerBackend { registry }` (dispatch by `task.tu` language); `ClangWorkerBackend` → `lang-cpp` |
| `crates/cli/src/registry.rs` | *new* | `pub fn default_registry() -> LanguageRegistry` — the **only** place plugins are enumerated (`CppPlugin::new()` now, `ScalaPlugin` in M19) |
| `crates/graph/src/reduced.rs`, `crates/graph/src/load.rs` | `reduce`, `compute_load` | **no change** — both are already language-neutral (verified); only their vocabulary is misleading, which is documented, not renamed, in this milestone |

Acceptance: golden artifact diff empty for both C++ fixtures; `m18_cpp_plugin_parity_test.rs` proves `CppDependencyResolver::resolve_edges`/`resolve_target` reproduce `parse_include_directives` + `IncludeResolver::resolve_file_includes` output on `tests/fixtures/basic` + `tests/fixtures/expensive`, and that `PluginWorkerBackend` reproduces `DefaultWorkerBackend::execute` byte-for-byte; source search for `clang` / `cl.exe` / `#include` / `".h"` in `index|graph|cluster|transform|validate|report` production code = 0 matches.


### M19 — Scala plugin: discovery + build context (first Scala code)

| File | Change |
|---|---|
| `crates/lang-scala/{Cargo.toml,src/lib.rs}` | new crate; `ScalaPlugin: LanguagePlugin` with `id()=Scala`, `modeling()` from §3.4, `capabilities() = {discovery:true, dependency_graph:true, semantic_index:true, load_metric:true, planning:false, extraction:false, rewriting:false, validation:false, requires_toolchain:false}` |
| `crates/lang-scala/src/detector.rs` | `ScalaDetector: LanguageDetector` — `.scala`→`Source`, `.sc`→`Script`, `build.sbt`/`build.sc`/`pom.xml`/`project/build.properties`→`BuildDefinition`; mirrors `FileKind::from_extension` |
| `crates/lang-scala/src/shaper.rs` | `ScalaUnitShaper: UnitShaper` — `build_markers()=["build.sbt","build.sc","pom.xml"]`, `default_compiler()="scalac"`, `shape_units()` = one `UnitDescriptor` per `.scala` under `src/main/scala` (tests under `src/test/scala` tagged), `context_name` = nearest ancestor `*.sbt` module (repo root ⇒ root project name) |
| `crates/cli/src/registry.rs` | `+ .register(Box::new(ScalaPlugin::new()))` — **the only pipeline edit needed to add the language** |
| `tests/fixtures/scala-basic/**` | `build.sbt`, `project/build.properties`, `src/main/scala/com/example/{Main.scala, models/{User.scala,Order.scala,all.scala}, services/OrderService.scala}`, `src/test/scala/com/example/OrderServiceSpec.scala`, plus `lib/legacy.jar` and `target/scala-2.13/` to exercise the external/frozen boundary |

Acceptance (`tests/integration/tests/m19_scala_discovery_test.rs`): `ariadne discover` (i.e. `Repository::discover`) lists the `.scala` files — `FileKind::from_extension` already maps `scala`/`sc`/`sbt`-family paths, so `crates/repo` needs no functional change beyond the additive `FileRecord.language`; `lib/` and `target/` are classified `ThirdParty`/`Generated` + `Frozen` by the existing `crates/repo/src/policy.rs::classify_ownership_and_policy`; `BuildDiscoverer::new(root).with_repo_discovery(..).with_unit_shaper(&ScalaUnitShaper).discover()` yields one unit per `src/main/scala` file with `CompilationContext { language: Scala, compiler: "scalac" }`, and the compile_commands path is not entered (no C++ diagnostics). `tests/integration/Cargo.toml` gains `ariadne-lang` + `ariadne-lang-scala` dev-dependencies.

### M20 — Scala dependency graph (imports → edges)

| File | Change |
|---|---|
| `crates/lang-scala/src/resolver.rs` | `ScalaImportResolver: DependencyResolver` — `resolve_edges` parses the `package` clause and `import a.b.C` / `import a.b.{C,D}` / `import a.b._` / Scala 3 `export`; `resolve_target` maps package → directory via `FileCatalog::by_module_path`, returning `Resolved(FileId)` / `Wildcard(files)` / `External` |
| `crates/graph/src/types.rs` | `IncludeKind` gains **one** additive variant `Import`; unresolved/out-of-repo imports reuse the existing `External` variant (§3.5) |
| `crates/graph/src/graph.rs` | cycle reporting (`generate_report`) and `crates/cluster/src/planner.rs` honour `CyclePolicy::TolerateWithinPackage`: an intra-package import cycle is reported as informational, not as a blocking defect |
| `crates/lang-scala/src/modeling.rs` | `external_path_markers = ["lib","target",".ivy2",".cache"]` |

Acceptance (`m20_scala_dependency_graph_test.rs`): `ariadne graph` emits `OrderService.scala → models/Order.scala` and `→ models/User.scala` with `kind:"Import"`; `import scala.collection.mutable._` becomes an `External` entry; the fixture's deliberate `models ↔ services` cycle does not fail the run (whereas C++ `m3` still asserts cycle reporting).


### M21 — Scala semantic index (offline, deterministic)

| File | Change |
|---|---|
| `crates/lang-scala/src/parser.rs` | `ScalaSourceParser::parse(file_id, rel_path, source) -> TuAnalysis`: `package` clause → namespace; top-level `class / trait / object / enum / case class / given / def / val` → `SymbolRecord` (`SymbolKind::{Class,Trait,Object,Enum,Function,Variable}`); brace-matched body spans; `extends`/`with` → `SemanticEdge { kind: Complete if the target resolves locally else Incomplete }`; `new X`, type refs and call syntax → `SymbolReference` / `CallSite`. Trust level = today's in-process C++ tokenizer + `ParserCtx` heuristics (a hand-written scanner, **no new dependency**) |
| `crates/lang-scala/src/analyzer.rs` | `ScalaAnalyzer: lang::SemanticAnalyzer` wrapping the parser (`fn analyze(&self, file_id, source, context) -> TuAnalysis`, filling `TuAnalysis.language = Scala`); `analysis_fingerprint() = "scala-parser-1"` |
| `crates/cli/src/commands/index.rs` | `handle_index_with_registry` picks `plugin.analyzer()` per unit, so a Scala repo is indexed through `ScalaAnalyzer` with no C++ include-closure walk (a Scala unit's "combined analysis" = its own file plus files reached by resolved imports) |
| `crates/scheduler/src/{worker.rs,incremental.rs}` | `PluginWorkerBackend` resolves the analyzer by unit language; `TuCacheFingerprint::compute` (`crates/index/src/cache.rs`) adds `language` + `plugin_fingerprint`; `CURRENT_ANALYSIS_VERSION` `"1.0.0"` → `"2.0.0"` |
| Deferred to M23 | `ScalaToolchainAnalyzer` (`scalac`/`scala-cli`/`sbt` probe) for exact types — kept out of M21 so CI stays offline-deterministic (A3) |

Acceptance (`m21_scala_semantic_index_test.rs`): index contains `com.example.services.OrderService` with edges to `com.example.models.Order` and `User`; cold run analyses every unit, second run reports 100 % hit-rate, editing `models/Order.scala` reanalyzes only import-dependents; C++ cache entries are **not** invalidated by the Scala fingerprint (language-scoped keys).

### M22 — Scala analysis outputs: aggregation, load metric, reports, Graph UI

| File | Change |
|---|---|
| `crates/cluster/src/planner.rs` | `aggregation_filter` (added in M18) is fed from `plugin.modeling()`: for Scala only a file declaring a `package object` is an aggregation unit (`models/all.scala`); a package without one gets **no** aggregation — no fake header is invented. `ClusteringConstraints` (companion / sealed family) is wired here even while `planning = false` |
| `crates/graph/src/load.rs` | **no algorithm change** (verified neutral): `LoadAnalyzer::analyze_file_load(<package object path>)` already separates semantic from closure-only symbols over the graph closure; only the human label differs |
| `crates/cluster/src/planner.rs` (capability gate) | when `capabilities.planning == false`, emit one explanatory `PlanRejection` per language instead of silently returning `vec![]` |
| `crates/report/src/{manifests.rs,manager.rs}` | additive `language` fields in repository / build / includes / load manifests |
| `crates/cli/src/commands/analyze.rs` (+ `includes.rs`, `query.rs`) | `handle_analyze_load` / `handle_explain_load` wording comes from `LanguageModeling` ("package aggregation" instead of "header") |
| `crates/ui/src/{dto.rs,export.rs}` | expose `language` + `unit_kind` on file nodes so the Graph UI can distinguish C++ from Scala |

Acceptance (`m22_scala_load_analysis_test.rs`): `ariadne load --symbol com.example.services.OrderService` reports non-zero collateral load attributed to `models/all.scala` (Scala analogue of the C++ "expensive" fixture); `ariadne explain` renders the import chain; `ariadne plan` prints the capability-gated rejection; C++ report/UI golden output unchanged.

### Deferred, specified but not executed here

- **M23 — Scala transform + validation.** Needs M18 plus: `ScalaSourceExtractor: SourceExtractor` (package clause + minimal imports + bodies), `ScalaDependencyRewriter: DependencyRewriter` (import-line replacement preserving every other byte — `TransactionApplier::apply` must call `rewrite()` instead of `content.replace("#include …")`), planner target template `{symbol}.scala` in the source package dir, `ClusteringConstraints` enforcement (companion object ↔ class, `sealed trait` ↔ subtypes stay together), tiers 1–2 ported, tier 3 = `Skipped` without a Scala toolchain. Highest-risk milestone; approve separately.
- **M24 — out-of-process plugin host** (`libloading` / WASM). Worth it only when a plugin is built by another team; the trait boundary here is the precondition, nothing more.


## 6. Test strategy

### 6.1 Regression protection first (before any behaviour moves)

- New `tests/golden/` snapshots produced by `tests/integration/tests/m00_golden_baseline_test.rs`: run `discover → build → graph → index → reduce → load → plan` on `tests/fixtures/basic` and `tests/fixtures/expensive`, normalize (sorted keys, absolute paths → `<ROOT>`, timestamps removed), write to `tests/golden/<fixture>/*.json`.
- `assert_golden_matches(fixture, artifact)` helper in `tests/integration/tests/common/mod.rs` (create the module if missing). It is what makes "behaviour-preserving" verifiable instead of asserted.

### 6.2 New unit tests (co-located `#[cfg(test)] mod tests`)

| Module | Tests |
|---|---|
| `lang-api::identity` | exhaustive `match` over every `FileKind` variant → expected `SourceLanguage` (`Header`/`Source`→`Cpp`, `ScalaSource`/`ScalaScript`/`BuildDefinition`→`Scala`, `Other`→`Other`) |
| `lang-api::catalog` | `by_rel_path` / `by_basename` / `by_module_path`; `/` vs `\` normalization on Windows |
| `lang::registry` (extends its existing tests) | `plugin_for_id`, `languages`, `detect_language` precedence when two plugins claim a path, `SourceLanguage::Other` ⇒ `None` |
| `lang-cpp::directive_parser` | port of every case in `crates/graph/src/resolver.rs`'s current tests (quoted vs angle, commented-out, `<>` system, `#  include` with whitespace) |
| `lang-cpp::resolver` | parity table vs `IncludeResolver::resolve_file_includes` on both C++ fixtures |
| `lang-scala::detector` | `.scala`, `.sc`, `build.sbt`, `pom.xml`, and `.java` → `Other` |
| `lang-scala::resolver` | simple / grouped / wildcard imports, `export`, unresolvable ⇒ `External` |
| `lang-scala::parser` | class, trait, object, case class, `given`, companion pairing, `extends`/`with` edges, nested `package`, comments and string interpolation must not create symbols |
| `lang-scala::shaper` | module naming, test-root tagging, empty `src/main/scala` ⇒ 0 units |
| `cluster::planner` | `aggregation_filter` + `target_path_for` defaults reproduce today's plan set on both C++ fixtures; Scala filter selects only package-object files |
| `index::cache` | `TuCacheFingerprint` with differing `language` / `plugin_fingerprint` ⇒ different digest; `TuCacheEntry::is_valid` regression |
| `validate::lib` | `Skipped` propagation + `format_output`; rollback still fires on `Fail` |

### 6.3 New integration tests (`tests/integration/tests/`, package `ariadne-integration-tests`)

`m17_plugin_seam_test.rs`, `m18_cpp_plugin_parity_test.rs`, `m19_scala_discovery_test.rs`, `m20_scala_dependency_graph_test.rs`, `m21_scala_semantic_index_test.rs`, `m22_scala_load_analysis_test.rs`. Style follows the existing suites (e.g. `m6_load_analysis_test.rs`): call `ariadne_cli` handlers directly on a fixture path (`handle_index`, `build_include_graph`, `handle_analyze_load`, `handle_explain_load`, `get_or_build_reduced_model`) and assert on returned models plus `artifacts/**/manifest.json`. Scala suites will need the registry-aware variants introduced in M18/M19 (e.g. `handle_index_with_registry(&path, &registry)`), so the language is chosen by the caller instead of a hidden default.


`m17` additionally contains the **architecture guard**:

```rust
#[test]
fn core_crates_do_not_name_languages() {
    // walk crates/{index,graph,cluster,transform,validate,report}/src
    // assert no production line contains "clang", "g++", "#include", "\".h\""
    // allowlist: doc comments + lines inside #[deprecated] transitional shims
}
#[test]
fn plugin_crates_do_not_depend_on_core_pipeline() {
    // parse crates/lang-cpp/Cargo.toml and crates/lang-scala/Cargo.toml
    // assert neither lists ariadne-graph / ariadne-cluster / ariadne-transform / ariadne-validate
}
```

### 6.4 Commands (Windows PowerShell, repo root)

```powershell
cargo check --workspace --all-targets
cargo test -p ariadne-lang-api -p ariadne-lang                     # M17
cargo test -p ariadne-lang-cpp -p ariadne-index -p ariadne-graph   # M18
cargo test -p ariadne-lang-scala                                    # M19-M21
cargo test --workspace                                              # every milestone
cargo test -p ariadne-integration-tests --test m19_scala_discovery_test  # single suite
Get-Command clang, clang++, clang-worker -ErrorAction SilentlyContinue   # expect empty here: the toolchain-less path must stay green
```

---

## 7. Risks, mitigations, review rules

| Risk | Likelihood | Mitigation |
|---|---|---|
| The M17 type move changes the serde shape of existing artifacts | medium | Move definitions verbatim (same field order + attributes); golden snapshot §6.1 catches any drift; `#[serde(default)]` only on genuinely new fields |
| `#[serde(default)]` on `TuAnalysis.language` requires `Default` | certain | `ariadne_repo::SourceLanguage` gains `#[derive(Default)]` with `#[default] Cpp` — every pre-existing artifact keeps reading correctly, and no `rename_all` is introduced so the stored PascalCase names stay valid |
| `#[deprecated]` shims in `graph`/`index` hide leftover C++ assumptions | medium | Shims are the only dual state; they are removed inside M18 itself (exit task M18.9), and the architecture guard excludes only files that carry an explicit `#[allow(deprecated)]` |
| Scala heuristic parser produces wrong symbols → misleading load metrics | high (by design) | Parser never fabricates a symbol from the file stem; `ScalaSourceParser` reports only constructs it actually recognised, and Scala output is labelled heuristic in CLI/report for M21–M22; exact types arrive with the M23 toolchain analyzer |
| Incremental cache poisoned across languages | medium | Cache key carries `language` + `plugin_fingerprint` + epoch bump; test asserts a Scala run cannot invalidate C++ entries and vice versa |
| Refactor stalls half-way with two shapes of the same trait | medium | Every milestone is independently mergeable and green; deprecated shims are deleted in the milestone that introduces them |
| C++ behaviour silently changes while `cluster`/`transform`/`validate` are generalized | high | The *current* C++ behaviour is injected as the default (`build_include_graph` keeps the `IncludeResolver` BFS, `BuildDiscoverer::discover` keeps compile_commands + `infer_default_discovery` when no shaper is injected, `OptimizationPlanner` defaults to `{name}_extracted.h`); Scala opts in explicitly instead of the other way round |

**Review rules**
- **R1** A "add a language" PR may touch only: `crates/lang-<x>/**`, `crates/cli/src/registry.rs`, `tests/fixtures/**`, `tests/integration/tests/**`. Checked by the `m17` guard test + PR checklist.
- **R2** Core crates may not name a language, a compiler binary, a file extension, or a directive syntax. Neutral nouns instead: *unit*, *aggregation*, *directive*, *dependency*, *external boundary*.
- **R3** Pipeline capabilities are reached only through `LanguageRegistry`; a `match LanguageId` arm outside `lang-api` / `cli/src/registry.rs` needs an ADR in `docs/adr/`.
- **R4** Hygiene from `AGENTS.md` still applies: files < 1500 lines, functions < 100 lines — `lang-scala/src/parser.rs` is expected to split into `parser/decl.rs` + `parser/expr.rs` as it grows.


---

## 8. Decision points needing your confirmation

| # | Question | Recommended default (what this plan assumes) |
|---|---|---|
| D1 | Milestone scope: analysis-only slice now (M17–M22), or Scala transform + validation in the same pass? | **Analysis-only now**; M23 approved separately (it is the only part that rewrites user source) |
| D2 | Where do the neutral value types live? | **New `ariadne-lang-api` crate** — breaks the `lang → index` cycle with zero call-site churn. Alternative: push the traits down into `ariadne-index` (no new crate, but `index` becomes a de-facto API crate and `lang` stays dependent on it, so R1 can never hold) |
| D3 | Scala semantic depth in the first slice? | **Built-in heuristic parser** (offline-deterministic); toolchain-backed analyzer deferred to M23 behind a probe |
| D4 | Should `ariadne plan` emit Scala optimization plans in this slice? | **No** — capability-gated with an explicit rejection message, so output is never silently empty |
| D5 | Keep `#include`-flavoured artifact field names (`include_load`, `tu_file_id`, `target_header`) for compatibility? | **Yes** in this plan (additive-only); renaming is a separate breaking milestone with an artifact-version bump |

---

## 9. Implementation order (todo list)

**M17 — API seam**
- [ ] M17.0 Capture golden artifacts for `tests/fixtures/basic` + `tests/fixtures/expensive` (`m00_golden_baseline_test.rs`, `tests/golden/`)
- [ ] M17.1 Create `crates/lang-api`; move the 11 value types out of `crates/index/src/types.rs` (`IndexResult` stays); `index` re-exports them
- [ ] M17.2 Add `identity.rs` (`SourceLanguage` alias + `language_of_file_kind`), `modeling.rs`, `capability.rs`, `unit.rs`, `catalog.rs` + unit tests; additive `repo` edits (`SourceLanguage` derives `PartialOrd, Ord, Default`, `FileRecord.language`)
- [ ] M17.3 Flip `crates/lang/Cargo.toml` from `ariadne-index` to `ariadne-lang-api`; add `unit.rs` (`UnitShaper`)
- [ ] M17.4 Extend `LanguagePlugin` (`id`, `modeling`, `capabilities`, `analysis_fingerprint`, `shaper`) + registry lookups (`plugin_for_id`, `languages`, `detect_language`) + tests
- [ ] M17.5 Add `DependencyResolver::resolve_target`, `FileCatalog`, `ResolveOutcome`, `CompilerValidator::toolchain_status`, `ToolchainStatus`
- [ ] M17.6 `m17_plugin_seam_test.rs` incl. architecture guards → `cargo test --workspace` green, golden diff empty

**M18 — C++ becomes a plugin**
- [ ] M18.1 Create `crates/lang-cpp`; move `crates/graph/src/resolver.rs` (`parse_include_directives`, `IncludeResolver`) → `directive_parser.rs` / `resolver.rs` (`CppDependencyResolver`, `CppUnitShaper`); `#[deprecated]` shims in `graph`
- [ ] M18.2 Move `CppSemanticAnalyzer` (tokenizer + `ParserCtx`) from `crates/index/src/analyzer.rs` → `lang-cpp/src/analyzer.rs` with the `lang::SemanticAnalyzer` adapter impl; `index` re-exports; move `ClangWorkerBackend` out of `crates/scheduler/src/worker.rs`
- [ ] M18.3 `crates/cli/src/registry.rs::default_registry()` + registry-aware `handle_index_with_registry` / `build_dependency_graph` variants + `--lang` flag
- [ ] M18.4 Inject `UnitShaper` into `BuildDiscoverer`; move the `"cl.exe"` / `cpp|cc|cxx|c` / `Default|x64` literals from `infer_default_discovery` into `lang-cpp::CppUnitShaper`
- [ ] M18.5 Make `build_include_graph`'s BFS generic over `&dyn DependencyResolver` (`build_dependency_graph(repo, registry)`); neutral alias `map_aggregation_consumers` in `graph.rs`
- [ ] M18.6 `OptimizationPlanner` gains `aggregation_filter`, `target_path_for`, `constraints`, `cycle_policy` with C++-identical defaults
- [ ] M18.7 Planner: path template from plugin, `ClusteringConstraints` merge of overlapping SCCs, `CyclePolicy` handling on top of `crate::scc`
- [ ] M18.8 `validate`: add `ValidationStatus::Skipped(String)` + `format_output`; `CompilerValidator::toolchain_status()`; `TuCacheFingerprint` (`language`, `plugin_fingerprint`) + `CURRENT_ANALYSIS_VERSION` `"1.0.0"` → `"2.0.0"`
- [ ] M18.9 `m18_cpp_plugin_parity_test.rs` (resolver + `PluginWorkerBackend` parity); remove the `#[deprecated]` shims; architecture guard passes; workspace green

**M19–M22 — Scala analysis slice**
- [ ] M19.1 `crates/lang-scala` + `ScalaPlugin` (modeling, capabilities) + workspace members
- [ ] M19.2 `ScalaDetector`, `ScalaUnitShaper` + unit tests
- [ ] M19.3 Register in `cli/src/registry.rs`; add `tests/fixtures/scala-basic/**`; `m19_scala_discovery_test.rs`
- [ ] M20.1 `ScalaImportResolver` (`resolve_edges` + `resolve_target`) + unit tests
- [ ] M20.2 `IncludeKind::Import`; cycle policy in `graph.rs` + `planner.rs`; `m20_scala_dependency_graph_test.rs`
- [ ] M21.1 `ScalaSourceParser` (package, decls, extends/with, references) + unit tests
- [ ] M21.2 `ScalaAnalyzer` + `handle_index_with_registry` + `PluginWorkerBackend` language dispatch; language-scoped cache key; `m21_scala_semantic_index_test.rs`
- [ ] M22.1 Package-object `aggregation_filter` for Scala in `planner.rs`; capability-gated `PlanRejection`
- [ ] M22.2 `language` in report manifests; `crates/cli/src/commands/analyze.rs` wording; `crates/ui` dto `language` / `unit_kind`
- [ ] M22.3 `m22_scala_load_analysis_test.rs`; golden re-check; workspace green

**Docs**
- [ ] D.1 Append M17–M22 to `Plan.md` using its Proves / Does-not-prove format
- [ ] D.2 `docs/adr/0001-language-plugin-boundary.md` = §3.4 mapping table + rules R1–R3
- [ ] D.3 "Adding a language in 5 steps" page (new crate → implement traits → register → fixture → tests)

**Sequencing note:** M17 is mostly mechanical; **M18 is the largest single block (~40 % of the work)**; M19–M20 are small; M21's parser is the second-largest item; M22 is medium. Each milestone should ship as its own reviewable PR set.

---

*No source files were modified while producing this plan.*






