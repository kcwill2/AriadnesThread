# Multi-Language Architecture — Implementation Plan

Status: **DRAFT for approval** (no source changed yet)
Date: 2026-09-24 · Refined 2026-09-29 (symbol-isolation + placement modeling, §0.1/D6/D7) · Baseline: `664e014` + uncommitted WIP · `cargo check --workspace --all-targets` = green (warnings only)

---

## 0. Inputs, assumptions, decision points

**Confirmed:** reference second language = **Scala**. The tree is already partly prepared: `crates/lang` (untracked WIP) contains unit-tested plugin traits (`LanguagePlugin`, `LanguageRegistry`, `LanguageDetector`, `DependencyResolver`, `SemanticAnalyzer`, `SourceExtractor`, `DependencyRewriter`, `CompilerValidator`, `ClusteringConstraints`), and `crates/repo/src/types.rs` already has `FileKind::{ScalaSource, ScalaScript, BuildDefinition}` plus `SourceLanguage { Cpp, Scala, Other }` / `from_file_kind` / `FileKind::language()`.

**Architectural refinement:** Rename "Include Graph" to "Import Graph" since superfluous imports affect Java, Scala, and other languages, not just C++. This makes the architecture truly language-agnostic and relevant for all modern languages with import systems.

### 0.1 Refinements (2026-09-29, confirmed by user — see D6/D7)

C++ is special among target languages: a symbol's **declaration** lives in a header while its **definition** may live in a different source file; most modern languages (Scala, Go, TS, Python) are **single-file placement** — one file per top-level symbol, declaration and definition inseparable. Two consequences are folded into this plan:

1. **The split is modeled explicitly, never assumed** (§3.2 `SymbolPlacement`, §3.4, D7): `LanguageModeling` gains `symbol_placement`; the reduced model resolves a *true* `defining_file` from the definition's declaration location and records `declaring_files`, instead of equating them via `owning_file` ("usually" — `crates/graph/src/reduced.rs:141`).
2. **Extraction is generalized into symbol isolation** (§3.6, D6): the language-neutral primitive is "move symbol S out of shared file F into its own file F_S; consumers that use only S point at F_S". For C++ that shrinks the include closure; for single-file languages it shrinks import granularity and makes the symbol→file relation (and therefore the per-file load metric) honest. `transform` becomes a neutral isolation engine in M18; M23 is re-framed from "Scala transform" to "symbol isolation — shared engine (C++ + Scala)".

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

**Already language-neutral — do not "generalize" it:** `crates/repo/src/types.rs` (`FileKind::ScalaSource`/`ScalaScript`/`BuildDefinition`, `SourceLanguage { Cpp, Scala, Other }`, `SourceLanguage::from_file_kind`, `FileKind::language()`, `is_analyzable()` which already includes `ScalaSource`), `crates/repo/src/policy.rs`, `crates/graph/src/reduced.rs::reduce` (ownership comes from `owning_file`/definition, no extension test — *except* the placement leak at line 141 `defining_file = owning_file` "usually", fixed by §3.6.1, not generalized), `crates/graph/src/load.rs::compute_load` (works on graph closure + symbol closure only), `crates/cluster/src/scc.rs`.

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
- "Minimal change rule" enforced by the M17 guard tests `m17_plugin_seam_test.rs::core_crates_do_not_name_languages` + `::plugin_crates_do_not_depend_on_core_pipeline` (§6.3).
- Non-comment production code in `crates/{index,graph,cluster,transform,validate,report}/src` contains **0** matches for `clang` / `g++` / `#include` / `".h"` after M18 (locked by `core_crates_do_not_name_languages`; D5-frozen identifiers like `include_load` are exempt by design).

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

/// Does one symbol's declaration and definition live in different files?
/// C++ (header declares, source defines) => SplitDeclaration.
/// Scala/Go/TS/Python (one file per top-level symbol) => SingleFile.
/// Drives symbol-isolation targets (§3.6) and reduced-model provenance (§3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolPlacement { SplitDeclaration, SingleFile }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageModeling {
    pub language: LanguageId,
    pub unit: UnitKind,
    pub aggregation: AggregationKind,
    pub guard: GuardKind,
    pub symbol_placement: SymbolPlacement,                   // NEW (D7)
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
| Symbol ownership | header owns declarations, TU owns definitions — `SymbolPlacement::SplitDeclaration`; reduced model keeps `defining_file` (definition decl's file) distinct from `declaring_files` | file owns top-level `class/trait/object/enum/given`; `package` clause owns the namespace — `SymbolPlacement::SingleFile`; declaring file == defining file always |
| Build manifest | `CMakeLists.txt`, `.vcxproj`, `compile_commands.json` | `build.sbt`, `project/build.properties`, `build.sc`, `pom.xml` |
| Toolchain | `cl.exe` fallback in `infer_default_discovery`, real compiler from `compile_commands.json`, optional `clang-worker` sidecar (JSON request/response protocol unchanged) | `scalac` / `scala-cli` / `sbt` **optional**; absent ⇒ built-in parser + `Skipped` validation tier |
| Atomic symbol groups | none beyond policy | companion object ↔ class, `sealed trait` ↔ direct subtypes (`ClusteringConstraints::colocated_groups`) |
| Extraction target naming | `{symbol}_extracted.h` (template provided by the plugin in M18 — `OptimizationPlanner::target_path_for`) | `{symbol}.scala` in the same package dir (same hook; §3.6) |
| Isolation unit (§3.6) | the **declaration site** — extract from the header, that is what consumers pay for; definitions stay in their source files | the **whole top-level symbol** — single-file placement, one file becomes its own file |

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
| `artifacts/transactions/*.json` | `ValidationStatus` gains `Skipped(String)`; `ValidationTier::GeneratedHeaders` + `ConsumerRewrite.old_includes`/`new_includes` + `MigrationPreview` keep their existing serde names (D5 legacy vocabulary) | `crates/validate/src/lib.rs`, `crates/transform/src/placement.rs` |
| `artifacts/reduced/symbols.bin` | `ReducedSymbolRecord` gains `declaring_files: Vec<FileId>` and `defining_file` is resolved from the definition's `SymbolDecl.location` (fallback `owning_file`) — §3.6.1. **This is bincode, not JSON: `#[serde(default)]` does not help.** Safe *because and only with* the M18 `CURRENT_ANALYSIS_VERSION` `"2.0.0"` bump: `check_manifest` rejects stale epoch and the blob is regenerated. JSON goldens unchanged (manifest stores counts only) | `crates/graph/src/reduced.rs`, `crates/report/src/manager/graph_io.rs` |

### 3.6 Symbol isolation — the language-neutral optimization primitive (D6)

The product's core move is **not** "extract a C++ header". It is: *move symbol S out of shared file F into its own file F_S; consumers that use only S now depend on F_S instead of F.* In C++ this shrinks the include closure; in Scala/Go/TS/Python it shrinks import granularity — and in both cases it makes the **symbol→file relation 1:1**, which is what makes the per-file load metric actionable. `SymbolPlacement` (§3.2) only changes *what* gets isolated (§3.4 "Isolation unit" row), not the pipeline.

**Division of labor (fixed in M18, executed for both languages in M23):**

| Piece | Where | Notes |
|---|---|---|
| Isolation engine: group plan extractions per target file, compute dependency edges among extracted symbols over the reduced model, cycle-check the target graph, write files | `crates/transform` (stays neutral) | `HeaderExtractor` → renamed `IsolationEngine`; `GeneratedHeader{filename, pragma_once, required_includes}` → `GeneratedUnit{path, required_deps}`; `ExtractedSymbol.forward_declarable` moves into the C++ plugin's payload. These three types have **no serde derives** (verified) — rename cost is call-site only (`validate/src/lib.rs`, suites `m10`/`m14`/`m20`) |
| New-file text rendering (pragma + includes / package + imports) | plugin: `lang::SourceExtractor` (already neutral in `crates/lang/src/extractor.rs`) | Today the engine itself emits `#pragma once` + `#include "…"` (`crates/transform/src/extractor.rs:157-163`); M18 moves that byte-emit into `lang-cpp::CppSourceExtractor` reproducing today's bytes exactly |
| Consumer redirect (which old deps a consumer still needs; side-effect preservation) | engine computes `ConsumerRewrite` (neutral logic, serde names frozen per D5); directive *text* rewriting + diff rendering (`#include "…"` lines in `format_diff`, `crates/transform/src/placement.rs:26-43`) move behind the plugin's `DependencyRewriter` | C++ bytes must stay identical (`m14`/`m20` assert on preview JSON + diff text) |
| Target file naming | plugin hook `OptimizationPlanner::target_path_for` (M18) | `{name}_extracted.h` becomes the C++ default, not an engine literal (`crates/cluster/src/planner.rs:97`) |
| Validation tiers | already neutral + M18's `Skipped` | C++ tier 3 = real compile; Scala tier 3 = `Skipped` offline |

**Known behaviour gap (fixed in M23, not M18):** today's engine does not copy real code — symbol bodies are placeholders (`format!("// Symbol: {} ({:?})", …)`, `crates/transform/src/extractor.rs:72`). M23 switches the engine to feed verbatim `SymbolBody` spans (parser span text) into the plugin extractor. This *changes C++ output* (generated files get real bodies), so M23 carries its own golden refresh — M18 stays byte-identical including the placeholders.

#### 3.6.1 Symbol→file provenance fix (D7, lands in M18)

- `ReducedSemanticModel::reduce` builds a `DeclId → FileId` map from each `tu.declarations` (`SymbolDecl.location.file_id` is already recorded) and sets `defining_file` = the defining decl's file, falling back to `owning_file` when the decl id is unknown — replacing the `owning_file` shortcut at `crates/graph/src/reduced.rs:141,148`.
- `ReducedSymbolRecord` gains `declaring_files: Vec<FileId>` (deduped, sorted — the files that host any declaration of the symbol; small even under ODR because each file is analyzed once).
- **Provably a no-op on current fixtures:** today's parser records every decl location in the file being analyzed, and each file in the include closure is analyzed exactly once, so definition-decl file == `owning_file` on both C++ fixtures → `defining_file` values are unchanged; only the added `declaring_files` field changes `symbols.bin` bytes (covered by the epoch bump, §3.5).
- UI (`crates/ui/src/dto.rs`) surfaces **declared-in** vs **defined-in** on symbol nodes (M22) — the meaningfulness of the symbol-file relation the isolation primitive depends on.

---

## 4. Crate / dependency changes

New workspace members (`Cargo.toml` `members` list) and their dependency edges:

| Crate | Package name | Depends on | Purpose |
|---|---|---|---|
| `crates/lang-api` | `ariadne-lang-api` | `ariadne-repo`, `serde`, `anyhow` | Moved index value types + `LanguageId`, `LanguageModeling`, `LanguageCapabilities`, `UnitDescriptor`, `FileCatalog`, `ResolveOutcome`, `ToolchainStatus` |
| `crates/lang-cpp` | `ariadne-lang-cpp` | `lang-api`, `lang`, `repo`, `index`, `serde_json` | `CppPlugin` (`symbol_placement = SplitDeclaration`) + `CppUnitShaper`; the `#include` parser and `IncludeResolver` logic moved out of `crates/graph/src/resolver.rs`; the `CppSemanticAnalyzer` tokenizer/`ParserCtx` moved out of `crates/index/src/analyzer.rs`; `ClangWorkerBackend` moved out of `crates/scheduler/src/worker.rs`; `CppSourceExtractor` (new-file text: `#pragma once` + `#include`) + `CppDependencyRewriter` (consumer redirect + diff text) moved out of `crates/transform` |
| `crates/lang-scala` | `ariadne-lang-scala` | `lang-api`, `lang`, `repo`, `serde`, `anyhow` | `ScalaPlugin` (`symbol_placement = SingleFile`): detector, `ScalaUnitShaper`, import resolver, `ScalaSourceParser` (semantic analyzer), constraints (companion/sealed); `ScalaSourceExtractor`/`ScalaDependencyRewriter` arrive in M23 (§3.6) |

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

> EXECUTION NOTES (M18.1–M18.2, reviewer-APPROVED): (a) `IncludeKind`/`SourceLocation`/`IncludeEdge` moved to `lang-api` (plugin crates may not depend on `graph`; serde/bincode shapes unchanged, graph re-exports original paths — rename deferred to a D5 milestone); (b) `ClangWorkerBackend` stays in `scheduler/worker.rs` until M18.11 — relocation needs `lang-cpp→scheduler` as a *normal* dep while `index→lang-cpp` shim exists (hard cargo cycle); unblocked when shims drop; (c) `lang-cpp` normal-deps `ariadne-build` (not in §4 table; acyclic, guard passes).

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
| `crates/graph/src/reduced.rs` | `merge_tu_symbols` (`defining_file = Some(sym.owning_file)`, line 141/148) | **§3.6.1 provenance fix:** build `DeclId → FileId` from each `tu.declarations`; `defining_file` = defining decl's file (fallback `owning_file`); add `declaring_files: Vec<FileId>` (sorted, deduped). No-op on current fixture values; `symbols.bin` (bincode) byte change is covered by the epoch bump below |
| `crates/graph/src/load.rs` | `compute_load` | **no change** — already language-neutral (verified); vocabulary documented, not renamed |
| `crates/transform/src/{lib.rs,extractor.rs}` | `GeneratedHeader`, `ExtractedSymbol.forward_declarable`, `HeaderExtractor`, `ExtractionResult::write_to_disk` (the `#pragma once` / `#include "…"` emit at extractor.rs:157-163) | neutral isolation engine (§3.6): `GeneratedUnit{path, symbols, required_deps}`; engine keeps grouping / inter-symbol dependency / cycle-check / disk-write, delegates new-file text to `plugin.extractor(): SourceExtractor`; C++ byte-emit → `lang-cpp::CppSourceExtractor` (reproduces today's bytes **including the placeholder bodies** — real verbatim bodies are an M23 change with its own golden refresh) |
| `crates/transform/src/placement.rs` | `IncludeRewriter`, `ConsumerRewrite::format_diff` (`#include` text) | engine keeps the symbol→file redirect logic (serde names frozen, D5); the directive-text rewrite + diff rendering move behind `plugin.rewriter(): DependencyRewriter`; `ConsumerRewrite`/`MigrationPreview` keep their serde shape |
| `crates/validate/src/lib.rs` | `ValidationStatus { Pass, Fail(String) }`, `TierValidator::*` | **M18 adds only** `ValidationStatus::Skipped(String)` + `format_output` handling + `CompilerValidator::toolchain_status()`; `ValidationTier::GeneratedHeaders` keeps its serialized name (D5, asserted by `m20`); `TransactionApplier::apply` (the `#pragma once` / `content.replace("#include …")` block) is generalized in **M23** |
| `crates/index/src/cache.rs` | `CURRENT_ANALYSIS_VERSION`, `TuCacheFingerprint::compute`, `TuCacheEntry::is_valid` | fingerprint gains `language` + `plugin_fingerprint`; epoch bump; `validate_and_get` unchanged |
| `crates/scheduler/src/incremental.rs` | `IncrementalAnalyzer::analyze` (uses `CppSemanticAnalyzer` + `IncludeResolver`), `TuCacheFingerprint::compute` call site | analyzer + resolver come from the registry (`PluginWorkerBackend`, `CppDependencyResolver`); header-dependency invalidation (`test_header_modification`) becomes the C++ plugin's `dependents_of()` contribution |
| `crates/scheduler/src/worker.rs` | `DefaultWorkerBackend::execute` | becomes `PluginWorkerBackend { registry }` (dispatch by `task.tu` language); `ClangWorkerBackend` → `lang-cpp` |
| `crates/cli/src/registry.rs` | *new* | `pub fn default_registry() -> LanguageRegistry` — the **only** place plugins are enumerated (`CppPlugin::new()` now, `ScalaPlugin` in M19) |

Acceptance: golden artifact diff empty for both C++ fixtures; `m18_cpp_plugin_parity_test.rs` proves `CppDependencyResolver::resolve_edges`/`resolve_target` reproduce `parse_include_directives` + `IncludeResolver::resolve_file_includes` output on `tests/fixtures/basic` + `tests/fixtures/expensive`, that `PluginWorkerBackend` reproduces `DefaultWorkerBackend::execute` byte-for-byte, that the isolation engine + `CppSourceExtractor` + `CppDependencyRewriter` reproduce today's generated-file bytes and `MigrationPreview` JSON/diff text (`m10`/`m14`/`m20` untouched and green), and that reduced-model `defining_file` values are unchanged while `declaring_files` is populated (both fixtures); source search for `clang` / `cl.exe` / `#include` / `".h"` in `index|graph|cluster|transform|validate|report` production code = 0 matches.


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
| `crates/ui/src/{dto.rs,export.rs}` | expose `language` + `unit_kind` on file nodes so the Graph UI can distinguish C++ from Scala; expose `defining_file` + `declaring_files` on symbol nodes so a C++ class renders "defined in `.cpp`, declared in `.h`" while a Scala symbol shows a single file (`symbol_placement` from §3.6.1) |

Acceptance (`m22_scala_load_analysis_test.rs`): `ariadne load --symbol com.example.services.OrderService` reports non-zero collateral load attributed to `models/all.scala` (Scala analogue of the C++ "expensive" fixture); `ariadne explain` renders the import chain; `ariadne plan` prints the capability-gated rejection; C++ report/UI golden output unchanged.

### Deferred, specified but not executed here

- **M23 — Symbol isolation, shared engine (C++ + Scala).** The *isolation* primitive (§3.6) is the point of the tool for every language, not just C++; M18 already makes the engine neutral, so M23 is "make it produce real code for both languages", executed under one approval gate.
  - **Fix the C++ placeholder gap:** the engine today emits `// Symbol: {} ({:?})` bodies (`crates/transform/src/extractor.rs:72`), not real code. M23 feeds verbatim `SymbolBody` spans (from `CppSemanticAnalyzer` source ranges) into `CppSourceExtractor`. **This changes C++ output ⇒ its own golden refresh + reviewer sign-off, deliberately separated from the byte-identical M18.**
  - **Scala:** `ScalaSourceExtractor: SourceExtractor` (package clause + minimal imports + real bodies), `ScalaDependencyRewriter: DependencyRewriter` (import-line replacement preserving every other byte — `TransactionApplier::apply` calls `rewrite()` instead of `content.replace("#include …")`), planner target `{symbol}.scala` in the source package dir (`target_path_for`, §3.4), `ClusteringConstraints` enforcement (companion object ↔ class, `sealed trait` ↔ subtypes stay together), tiers 1–2 ported, tier 3 = `Skipped` without a Scala toolchain.
  - `SymbolPlacement` selects the isolation unit per language at plan time (§3.4 "Isolation unit" row): C++ isolates the declaration site (header), Scala isolates the whole top-level symbol.
  - Highest-risk milestone; approve separately.
- **M24 — out-of-process plugin host** (`libloading` / WASM). Worth it only when a plugin is built by another team; the trait boundary here is the precondition, nothing more.


## 6. Test strategy

### 6.1 Regression protection first (before any behaviour moves)

- New `tests/golden/` snapshots produced by `tests/integration/tests/m00_golden_baseline_test.rs`: run `discover → build → graph → index → reduce → load → plan` on `tests/fixtures/basic` and `tests/fixtures/expensive`, normalize (sorted keys, absolute paths → `<ROOT>`, timestamps removed), write to `tests/golden/<fixture>/*.json`.
- `assert_golden_matches(fixture, artifact)` helper in `tests/integration/tests/common/mod.rs` (create the module if missing). It is what makes "behaviour-preserving" verifiable instead of asserted.

### 6.2 New unit tests (co-located `#[cfg(test)] mod tests`)

| Module | Tests |
|---|---|
| `lang-api::identity` | exhaustive `match` over every `FileKind` variant → expected `SourceLanguage` (`Header`/`Source`→`Cpp`, `ScalaSource`/`ScalaScript`/`BuildDefinition`→`Scala`, `Other`→`Other`) |
| `lang-api::modeling` | `SymbolPlacement` round-trips through serde; C++/Scala modeling fixtures carry `SplitDeclaration` / `SingleFile` respectively |
| `lang-api::catalog` | `by_rel_path` / `by_basename` / `by_module_path`; `/` vs `\` normalization on Windows |
| `lang::registry` (extends its existing tests) | `plugin_for_id`, `languages`, `detect_language` precedence when two plugins claim a path, `SourceLanguage::Other` ⇒ `None` |
| `lang-cpp::directive_parser` | port of every case in `crates/graph/src/resolver.rs`'s current tests (quoted vs angle, commented-out, `<>` system, `#  include` with whitespace) |
| `lang-cpp::resolver` | parity table vs `IncludeResolver::resolve_file_includes` on both C++ fixtures |
| `lang-scala::detector` | `.scala`, `.sc`, `build.sbt`, `pom.xml`, and `.java` → `Other` |
| `lang-scala::resolver` | simple / grouped / wildcard imports, `export`, unresolvable ⇒ `External` |
| `lang-scala::parser` | class, trait, object, case class, `given`, companion pairing, `extends`/`with` edges, nested `package`, comments and string interpolation must not create symbols |
| `lang-scala::shaper` | module naming, test-root tagging, empty `src/main/scala` ⇒ 0 units |
| `cluster::planner` | `aggregation_filter` + `target_path_for` defaults reproduce today's plan set on both C++ fixtures; Scala filter selects only package-object files |
| `graph::reduced` | `defining_file` resolved from the definition decl's file; fallback to `owning_file` when the decl id is unknown; `declaring_files` deduped + sorted; on fixture data `defining_file` equals the legacy `owning_file` value (§3.6.1) |
| `transform::engine` | neutral engine + injected `CppSourceExtractor`/`CppDependencyRewriter` reproduce pre-M18 generated-file bytes, `MigrationPreview` JSON, and `format_diff` text byte-for-byte (incl. placeholder bodies); cycle check unchanged |
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
| `#[deprecated]` shims in `graph`/`index` hide leftover C++ assumptions | medium | Shims are the only dual state; they are removed inside M18 itself (exit task M18.11), and the architecture guard excludes only files that carry an explicit `#[allow(deprecated)]` |
| Scala heuristic parser produces wrong symbols → misleading load metrics | high (by design) | Parser never fabricates a symbol from the file stem; `ScalaSourceParser` reports only constructs it actually recognised, and Scala output is labelled heuristic in CLI/report for M21–M22; exact types arrive with the M23 toolchain analyzer |
| Incremental cache poisoned across languages | medium | Cache key carries `language` + `plugin_fingerprint` + epoch bump; test asserts a Scala run cannot invalidate C++ entries and vice versa |
| Refactor stalls half-way with two shapes of the same trait | medium | Every milestone is independently mergeable and green; deprecated shims are deleted in the milestone that introduces them |
| `symbols.bin` is **bincode** — adding `declaring_files` shifts the byte layout and `#[serde(default)]` does **not** rescue it | certain | The M18 `CURRENT_ANALYSIS_VERSION` bump (`"2.0.0"`) is what forces regeneration; `report/.../graph_io.rs::check_manifest` rejects the stale epoch before `load_reduced_artifacts` deserializes, so a v1 blob never reaches the new struct. A test asserts a v1-era `symbols.bin` triggers reindex, not a serde error |
| Isolation engine change silently breaks `m10`/`m14`/`m20` (they consume `transform` types) | medium | `GeneratedHeader`/`ExtractedSymbol`/`ExtractionResult` have no serde derives (verified), so the neutral rename is call-site-only; `ConsumerRewrite`/`MigrationPreview` keep their exact serde field names (D5); byte-identical parity test + those three suites run unmodified as the guard |
| The real-bodies fix (§3.6) would change C++ generated output and blow the "byte-identical" promise | high if mixed into M18 | Deliberately deferred to M23 with its own golden refresh; M18 keeps the current placeholder bodies so the behaviour-preserving invariant holds |
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
| D6 | How far does generalized **symbol isolation** (extract symbol → own file) go in this slice? | **Confirmed 2026-09-29: Option A** — M17/M18 define the neutral contract (§3.6) and prove C++ byte-parity through the seam; M23, re-framed as "symbol isolation — shared engine", executes real isolation for C++ *and* Scala under one gate. Scala isolation is *not* run inside M19–M22 |
| D7 | How explicitly is the C++-specific declaration/definition split modeled? | **Confirmed 2026-09-29: Option A** — `LanguageModeling.symbol_placement` + reduced-model `declaring_files` and true `defining_file` (§3.6.1) + UI declared-vs-defined; existing serde names untouched, bincode risk covered by the epoch bump |

---

## 9. Implementation order (todo list)

**M17 — API seam**
- [x] M17.0 Capture golden artifacts for `tests/fixtures/basic` + `tests/fixtures/expensive` (`m00_golden_baseline_test.rs`, `tests/golden/`)
- [x] M17.1 Create `crates/lang-api`; move the 11 value types out of `crates/index/src/types.rs` (`IndexResult` stays); `index` re-exports them
- [x] M17.2 Add `identity.rs` (`SourceLanguage` alias + `language_of_file_kind`), `modeling.rs`, `capability.rs`, `unit.rs`, `catalog.rs` + unit tests; additive `repo` edits (`SourceLanguage` derives `PartialOrd, Ord, Default`, `FileRecord.language`)
- [x] M17.3 Flip `crates/lang/Cargo.toml` from `ariadne-index` to `ariadne-lang-api`; add `unit.rs` (`UnitShaper`)
- [x] M17.4 Extend `LanguagePlugin` (`id`, `modeling`, `capabilities`, `analysis_fingerprint`, `shaper`) + registry lookups (`plugin_for_id`, `languages`, `detect_language`) + tests
- [x] M17.5 Add `DependencyResolver::resolve_target`, `FileCatalog`, `ResolveOutcome`, `CompilerValidator::toolchain_status`, `ToolchainStatus`
- [x] M17.6 `m17_plugin_seam_test.rs` incl. architecture guards → `cargo test --workspace` green, golden diff empty

**M18 — C++ becomes a plugin**
- [x] M18.1 Create `crates/lang-cpp`; move `crates/graph/src/resolver.rs` (`parse_include_directives`, `IncludeResolver`) → `directive_parser.rs` / `resolver.rs` (`CppDependencyResolver`, `CppUnitShaper`); `#[deprecated]` shims in `graph`
- [x] M18.2 Move `CppSemanticAnalyzer` (tokenizer + `ParserCtx`) from `crates/index/src/analyzer.rs` → `lang-cpp/src/analyzer.rs` with the `lang::SemanticAnalyzer` adapter impl; `index` re-exports; move `ClangWorkerBackend` out of `crates/scheduler/src/worker.rs`
- [x] M18.3 `crates/cli/src/registry.rs::default_registry()` + registry-aware `handle_index_with_registry` / `build_dependency_graph` variants + `--lang` flag
- [x] M18.4 Inject `UnitShaper` into `BuildDiscoverer`; move the `"cl.exe"` / `cpp|cc|cxx|c` / `Default|x64` literals from `infer_default_discovery` into `lang-cpp::CppUnitShaper`
- [x] M18.5 Make `build_include_graph`'s BFS generic over `&dyn DependencyResolver` (`build_dependency_graph(repo, registry)`); neutral alias `map_aggregation_consumers` in `graph.rs` — MUST reconcile documented divergences from M18.2 review: legacy closure includes ThirdParty headers while plugin walk drops `ResolveOutcome::External`; `PluginWorkerBackend` context path must not run hardcoded-C++ `analyze_tu_full` for non-C++ languages; angle style/include-dir fallback unmodeled in `ResolveOutcome`
- [x] M18.6 `OptimizationPlanner` gains `aggregation_filter`, `target_path_for`, `constraints`, `cycle_policy` with C++-identical defaults
- [x] M18.7 Planner: path template from plugin, `ClusteringConstraints` merge of overlapping SCCs, `CyclePolicy` handling on top of `crate::scc`
- [x] M18.8 Neutral isolation engine (§3.6): `GeneratedHeader`→`GeneratedUnit`, drop `forward_declarable`, engine delegates new-file text to `lang-cpp::CppSourceExtractor` and consumer-redirect/diff text to `CppDependencyRewriter`; `ConsumerRewrite`/`MigrationPreview` serde names frozen (D5); byte-parity incl. placeholder bodies
- [x] M18.9 `reduced.rs` provenance fix (§3.6.1): `DeclId→FileId` map from `tu.declarations`, true `defining_file` (fallback `owning_file`), add `declaring_files`; epoch bump invalidates the bincode `symbols.bin`
- [x] M18.10 `validate`: add `ValidationStatus::Skipped(String)` + `format_output`; `CompilerValidator::toolchain_status()`; `TuCacheFingerprint` (`language`, `plugin_fingerprint`) + `CURRENT_ANALYSIS_VERSION` `"1.0.0"` → `"2.0.0"`
- [x] M18.11 `m18_cpp_plugin_parity_test.rs` (resolver + `PluginWorkerBackend` + isolation-engine byte-parity + `MigrationPreview` parity + reduced-model provenance); remove the `#[deprecated]` shims; architecture guard passes; workspace green

> EXECUTION NOTES (M18.11, reviewer-APPROVED 2026-09-30 at 3b06e54): (a) the permanent suite's body-only worker oracle shares `CppSemanticAnalyzer::analyze_source` with the subject, so it locks dispatch/shape only — analyzer-semantics locking lives in the m00 goldens; (b) `core_crates_do_not_name_languages` has two admitted gaps: core-crate *test* code imports plugin type names (`CppDependencyRewriter`/`CppSourceExtractor`) and `"c++20"` literals pass the 4-token FORBIDDEN list — widening tokens (`Cpp`/`lang_cpp`/`c++`) or moving byte-fixture tests out of core src is an M23 item; the `concat!` literal scrub is byte-identical in assertion semantics and judged acceptable; (c) `pipeline::default_registry()` duplicates the `cli::registry::default_registry()` enumeration — keep in sync at M19 or share one helper; (d) relocation edge is `lang-cpp → scheduler` (verified `cargo tree -e normal`: no scheduler→lang-cpp normal edge); validate's 3-arg `apply` shim intentionally survives until M23.

> EXECUTION NOTES (M18.3/M18.5, recorded post-audit 2026-10-01): divergence fixes from the M18.2 review live in code — (i) External-drop closed by feeding the compilation context's include dirs into `ResolutionContext.search_paths` + `on_disk_include_fallback` (`scheduler/tu_store.rs` `edge_destinations`, mirrored by the `graph` BFS incl. catalog-ownership ThirdParty→External classification); (ii) worker dispatch plugin-only (`analyze_tu_full_with_plugin`; empty registry reports unhandled, never defaults to C++); (iii) `DirectiveStyle` + shared `first_ondisk_include_candidate` reproduce legacy candidate order. Post-completion audit (2026-10-01): C++-is-a-plugin intent confirmed, zero behavioral regressions; accepted file-placement drifts — `CppUnitShaper` lives in `lang-cpp/src/shaper.rs` and the analyzer family spans `analyzer.rs`/`parser.rs`/`tokenizer.rs` (symbols/crate placement unchanged); the shaper-less compile_commands path in `build` retains `cl.exe`/`x64`/`language: Cpp` residues per §4 with plugin injection logged as an M19+ item.

**M19–M22 — Scala analysis slice**
- [ ] M19.1 `crates/lang-scala` + `ScalaPlugin` (modeling, capabilities) + workspace members
- [ ] M19.2 `ScalaDetector`, `ScalaUnitShaper` + unit tests
- [ ] M19.3 Register in `cli/src/registry.rs`; add `tests/fixtures/scala-basic/**`; `m19_scala_discovery_test.rs`
- [ ] M20.1 `ScalaImportResolver` (`resolve_edges` + `resolve_target`) + unit tests
- [ ] M20.2 `IncludeKind::Import`; cycle policy in `graph.rs` + `planner.rs`; `m20_scala_dependency_graph_test.rs`
- [ ] M21.1 `ScalaSourceParser` (package, decls, extends/with, references) + unit tests
- [ ] M21.2 `ScalaAnalyzer` + `handle_index_with_registry` + `PluginWorkerBackend` language dispatch; language-scoped cache key; `m21_scala_semantic_index_test.rs`
- [ ] M22.1 Package-object `aggregation_filter` for Scala in `planner.rs`; capability-gated `PlanRejection`
- [ ] M22.2 `language` in report manifests; `crates/cli/src/commands/analyze.rs` wording; `crates/ui` dto `language` / `unit_kind` on file nodes and `defining_file` / `declaring_files` (declared-vs-defined) on symbol nodes
- [ ] M22.3 `m22_scala_load_analysis_test.rs`; golden re-check; workspace green

**Docs**
- [ ] D.1 Append M17–M22 to `Plan.md` using its Proves / Does-not-prove format
- [ ] D.2 `docs/adr/0001-language-plugin-boundary.md` = §3.4 mapping table + §3.6 isolation contract + rules R1–R3
- [ ] D.3 "Adding a language in 5 steps" page (new crate → implement traits → register → fixture → tests)
- [ ] D.4 `docs/adr/0002-symbol-placement-and-isolation.md` — C++ `SplitDeclaration` vs single-file placement; why isolation (symbol → own file) is the language-neutral primitive (D6/D7 rationale)

**Sequencing note:** M17 is mostly mechanical; **M18 is the largest single block (~45 % of the work — the transform neutralization and provenance fix join the original dispatch rewiring)**; M19–M20 are small; M21's parser is the second-largest item; M22 is medium. Each milestone should ship as its own reviewable PR set.

---

*No source files were modified while producing this plan.*






