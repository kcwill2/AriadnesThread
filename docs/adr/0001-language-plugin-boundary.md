# ADR-0001: The language plugin boundary

- Status: Accepted
- Date: 2026-10-03
- Applies to: `crates/lang-api`, `crates/lang`, `crates/lang-cpp`, `crates/lang-scala`, and every core crate that receives `&dyn` language capabilities
- Supersedes: nothing
- Spec source: `plans/multi-language-support-implementation_plan.md` sections 3.1-3.4, 3.6, and review rules R1-R3

## Context

Before this work the language question was answered by hardcoded C++ literals scattered
through the pipeline. The coupling table in the spec (section 1, whose rows are named call
sites rather than guesses) places C++-specific behaviour in `crates/build` (translation-unit
discovery keyed on C/C++ extensions, plus a synthesized `cl.exe /c` invocation), `crates/graph`
(the include-closure BFS and its `#include` directive parser), `crates/index` (one in-process
C++ parser), `crates/cluster` (a header-named aggregation query), `crates/transform` (a header
emitter that writes `#include` lines), `crates/validate` (an applier that rewrites `#include`
text), `crates/scheduler` (the worker backends), and `crates/cli`. Adding a language therefore
meant edits in at least six crates plus two binaries (`crates/cli/src/bin/tool.rs` and
`tools/clang-worker/src/main.rs`).

Before this work, the `ariadne-lang` trait surface already existed and was unit-tested, but no
production code called it: `LanguageRegistry` was reachable only from its own tests, so the seam
was dead code while every stage stayed hardcoded. The seam was the right shape and the wrong
wiring. That state is recorded in the spec's section 1 coupling table and is not restated as
current fact anywhere in this ADR.

The goal this record fixes, stated once:

> **new language = 1 new crate + 1 registry entry + fixtures/tests, with 0 edits to the core pipeline crates.**

That sentence is the decision's whole point. Everything below is the machinery that makes it
true, and the last two sections are the honest limits of what has actually been proven by it.

## Decision

### 1. Vocabulary lives in `ariadne-lang-api`

The eleven index value types -- `SymbolId`, `DeclId`, `SymbolKind`, `DependencyKind`,
`FileLocation`, `SymbolDecl`, `SymbolRecord`, `SemanticEdge`, `CallSite`, `SymbolReference`,
`TuAnalysis` -- now live in `crates/lang-api/src/types.rs`. `crates/index/src/types.rs` is
reduced to `pub use ariadne_lang_api::types::*;` plus `IndexResult`, which stayed behind because
it is an index product rather than shared vocabulary. Every existing call path keeps resolving:
`ariadne_index::types::SymbolId`, the crate-root re-export, and `ariadne_index::SymbolId` are
now the *same* type, not a conversion pair. `tests/integration/tests/m17_plugin_seam_test.rs`
asserts that identity directly in `lang_api_types_are_the_same_types_index_re_exports`.

Why the vocabulary had to move rather than the traits: the dependency direction was
`lang -> index`, because the traits took and returned index value types, so pushing the traits
down into `ariadne-index` would have been a permanent cycle. `crates/lang/Cargo.toml` now
depends on `ariadne-lang-api` and no longer on `ariadne-index`.

`crates/lang-api` depends only on `ariadne-repo` and `serde`. It does not invent a second
language enum: `crates/lang-api/src/identity.rs` re-exports the existing
`ariadne_repo::SourceLanguage` as `LanguageId`.

### 2. Plugin crates depend inward; core crates never depend on plugin crates

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
                              ariadne-cli   (delegates plugin enumeration)
```

Rule: **plugin crates depend inward; core crates never depend on plugin crates.**

The ONLY place plugins are enumerated is `build_default_registry` in
`crates/pipeline/src/registry.rs`, reached through `ariadne_pipeline`. `crates/cli/src/registry.rs`
calls `ariadne_pipeline::build_default_registry()` and enumerates nothing itself. That direction
is forced, not chosen: `ariadne-cli` depends on `ariadne-pipeline`, never the reverse, so the
composition root has to live in the pipeline crate.

Registration order -- C++ first, then Scala -- is a contract, not an accident. It drives
`unit_shaper_for()` fallback and `detect_language` precedence, which is what keeps existing C++
discovery byte-identical. `crates/cli/src/registry.rs` locks the order in
`default_registry_enums_cpp_then_scala`.

The direction is not a convention that review happens to catch; it is asserted mechanically.
See Machine-checkable enforcement below.

### 3. Core crates reach capabilities only as `&dyn`

A core stage never names a plugin type, a compiler binary, a file extension, or a directive
syntax. It receives `&dyn ariadne_lang::LanguagePlugin` (or a narrower `&dyn` view -- one of
`SemanticAnalyzer`, `DependencyResolver`, `SourceExtractor`, `DependencyRewriter`,
`CompilerValidator`, `LanguageDetector`, `ClusteringConstraints`, `UnitShaper`) and asks it for
data through `modeling()`, `capabilities()`, `toolchain_status()`, and `shaper()`.

Routing is per-language, through the registry, with no fallback language.
`capability_backend_for_language` in `crates/cli/src/registry.rs` returns the plugin of *that*
language when that plugin's own capabilities enable the stage, and `None` otherwise;
`plan_backend` resolves a plan's language through the registry detector rather than defaulting.
`incremental_backend` reports "unhandled language" for a unit whose language has no registered
plugin. In every case the answer for an unclaimed language is an error or `None`, never C++.

## The lifecycle mapping (C++ vs Scala)

The spec's table at section 3.4 is written in planned-move tense ("existing -> moves into
`lang-cpp`"). The table below is transposed: rows are the *concepts*, and the two columns say
where each one lives NOW, post-landing. The spec remains the historical record of the intent.
Where the two differ, this table wins, because it describes shipped code.

| Concept | C++ (in `crates/lang-cpp`) | Scala (in `crates/lang-scala`) |
|---|---|---|
| Analysis unit | `UnitKind::TranslationUnit`. `CppUnitShaper` shapes one unit per C/C++ source with a synthesized `cl.exe /c <canonical>` build spec, an `NDEBUG` define, and an `include/` search dir | `UnitKind::SourceFile`. `ScalaUnitShaper` shapes sbt-layout units |
| Build-manifest markers | `CMakeLists.txt`, `compile_commands.json`, `.vcxproj` | `build.sbt`, `build.sc`, `pom.xml` |
| Aggregation artifact | `AggregationKind::Header`, `aggregation_reexports = true` | `AggregationKind::PackageObject`, `aggregation_reexports = false` |
| Duplicate-definition guard | `GuardKind::PragmaOnce`. `CppSourceExtractor` emits the guard plus quoted dependency lines | `GuardKind::None` (JVM plus package uniqueness) |
| Dependency directive | `#include "x.h"` and `<x>`, parsed by `parse_include_directives` in `crates/lang-cpp/src/directive_parser.rs`; edges carry `DirectiveStyle::Quoted` or `DirectiveStyle::Angled` | `import` and `export`, parsed by `parse_import_directives` in `crates/lang-scala/src/directive_parser.rs`; edges carry `DirectiveStyle::Unspecified` |
| External boundary | path markers `third_party` and `vendor`, plus the angle system include, which the resolver knows as directive style rather than as a path marker | `EXTERNAL_PATH_MARKERS` in `crates/lang-scala/src/boundary.rs`: `lib`, `target`, `.ivy2`, `.cache` |
| Cycle policy | `CyclePolicy::Reject` | `CyclePolicy::TolerateWithinPackage` |
| Collateral load | symbols from transitively included aggregation artifacts that are not used | symbols pulled into the compile-time closure by importing a package aggregation |
| Symbol placement | `SymbolPlacement::SplitDeclaration`. In `crates/graph/src/reduced.rs`, `defining_file` is resolved from the definition's declaration location and is kept distinct from `declaring_files` | `SymbolPlacement::SingleFile`. Declaring file and defining file are always the same |
| Isolation unit | the declaration site -- extract from the aggregation artifact, because that is what consumers pay for; definitions stay in their source files | the whole top-level symbol -- single-file placement, so one file becomes its own file |
| Isolation target naming | `cpp_target_path_for` in `crates/lang-cpp/src/planning.rs`, producing `{symbol}_extracted.h` beside the aggregation file | `scala_target_path_for` in `crates/lang-scala/src/planning.rs`, producing `{Symbol}.scala` beside the aggregation file (the package-object file, so the target lands in the package directory) |
| Atomic symbol groups | none beyond ownership and policy: `CppClusteringConstraints::colocated_groups` returns empty | `ScalaClusteringConstraints`: a companion object paired with its class, and a `sealed` trait grouped with its direct subtypes via `colocated_groups_with_edges` |
| Toolchain | optional. `CppCompilerValidator::toolchain_status()` spawns `clang --version`; `ClangWorkerBackend` in `crates/lang-cpp/src/backend.rs` drives the `tools/clang-worker` sidecar. The built-in analyzer runs offline, so `requires_toolchain = false` | optional. `ScalaToolchainValidator` probes `scalac`, `scala-cli`, `sbt` in that order; no usable candidate means `ToolchainStatus::Missing`, which feeds the existing tier-3 `Skipped` mapping in `crates/validate/src/lib.rs`. `requires_toolchain = false` |
| Output wording | `DependencyVocabulary` with directive label `Include`, passive verb `included`, aggregation noun `header`; `analysis_caveat: None` | `DependencyVocabulary` with directive label `Import`, passive verb `imported`, aggregation noun `package aggregation`; `analysis_caveat: Some(...)`, the offline-heuristic-parser disclaimer |

## What each crate may know

Review rule R2, in substance: **core crates may not name a language, a compiler binary, a file
extension, or a directive syntax.** Neutral nouns are used instead. The neutral vocabulary is
this set, and every one of them resolves to a concrete type in `crates/lang-api`:

| Neutral noun | What it means | Where it is spelled |
|---|---|---|
| unit | the file-shaped thing an analyzer runs on | `UnitKind`, `UnitDescriptor` (`crates/lang-api/src/unit.rs`) |
| aggregation | what owns symbols and is re-loaded by consumers | `AggregationKind`, `DependencyVocabulary::aggregation_noun` (`crates/lang-api/src/modeling.rs`) |
| directive | the source-text spelling of one dependency edge | `DependencyVocabulary::directive_label` and `directive_passive_verb` (`crates/lang-api/src/modeling.rs`) |
| dependency | a typed edge between two symbols | `DependencyKind` (`crates/lang-api/src/types.rs`), `RawDepEdge` (`crates/lang/src/resolver.rs`) |
| external boundary | the frozen, out-of-scope region | `LanguageModeling::external_path_markers` (`crates/lang-api/src/modeling.rs`), `ResolveOutcome::External` (`crates/lang-api/src/catalog.rs`) |
| symbol placement | whether one symbol's declaration and definition live in different files | `SymbolPlacement` (`crates/lang-api/src/modeling.rs`) |
| isolation unit | what symbol isolation moves into a file of its own | derived from `SymbolPlacement`: the declaration site when placement is split, the whole symbol when placement is single-file |

Only C++ vocabulary lives in `crates/lang-cpp`; only Scala vocabulary lives in
`crates/lang-scala`. Two exceptions are cited here rather than buried, because both are places
where the boundary is deliberately thin:

- `ToolchainStatus` (`crates/lang-api/src/capability.rs`) and `ResolveOutcome`
  (`crates/lang-api/src/catalog.rs`) live in `ariadne-lang-api` so that core crates can match
  on results without importing a plugin. `ToolchainStatus` is what makes a tier-3 `Skipped`
  expressible in `crates/validate` without the validator crate naming any compiler; `ResolveOutcome`
  is what a plugin resolver hands back so `crates/graph` can classify a resolved target from a
  `FileRecord` without touching a plugin type.
- Three artifact field names are frozen for compatibility and keep C++-flavored spelling:
  `TuAnalysis::tu_file_id` (`crates/lang-api/src/types.rs`), `LoadMetric::include_load`
  (`crates/graph/src/load.rs`), and `ProposedExtraction::target_header`
  (`crates/cluster/src/plan.rs`). These are serde field names in stored artifacts; renaming any
  of them breaks every artifact already on disk and every suite that deserializes one. They are
  recorded compatibility debt, not permission to add more.

## Machine-checkable enforcement

This is the section that makes the record worth writing.

Two guard tests in `tests/integration/tests/m17_plugin_seam_test.rs` give the boundary teeth:

- **`plugin_crates_do_not_depend_on_core_pipeline`** reads `crates/lang-cpp/Cargo.toml` and
  `crates/lang-scala/Cargo.toml` as text and asserts that neither declares a dependency on
  `ariadne-graph`, `ariadne-cluster`, `ariadne-transform`, or `ariadne-validate`. It parses the
  plugin manifests.
- **`core_crates_do_not_name_languages`** walks every `.rs` file under `src/` in
  `crates/index`, `crates/graph`, `crates/cluster`, `crates/transform`, `crates/validate`,
  `crates/report`, and `crates/pipeline`, and asserts that no source line contains any of the
  seven tokens in `GUARD_FORBIDDEN`: `clang`, `g++`, `#include`, `".h"`, `Cpp`, `lang_cpp`,
  `ariadne-lang-cpp`. Exactly one file is exempt, by exact path: `crates/pipeline/src/registry.rs`,
  the named composition root whose single duty is enumerating the plugin set. Doc-comment and
  whole-line-comment lines are exempt, and `GUARD_NEUTRAL_VOCAB` strips the exact string
  `SourceLanguage::Cpp` -- spine language-taxonomy data owned by `crates/repo`, which is outside
  the walk scope by design, because the repository spine must be able to name languages.
  `guard_neutral_vocab_stripping_cannot_mask_plugin_names` locks that this stripping cannot be
  used to hide a plugin identifier sharing a line with spine data.

The consequence, stated plainly: **rule R1 is verifiable, not merely asserted**, because the
plugins are compile-time crates whose `Cargo.toml` manifests are files sitting in the repository
that a test can read. "Did this plugin crate reach into the core pipeline?" is a question the
repository itself can answer, on every machine, with no toolchain installed.

That is exactly the property a runtime-loaded plugin host would take away. A dylib or WASM
plugin's dependency surface is decided at load time by the host, not declared in a manifest that
lives beside the source, so the same question stops being answerable from the repository. The
boundary is auditable only while plugins are statically linked into the binary. Any move to
dynamic loading has to replace the enforcement mechanism, not just the loading mechanism, which
is why it needs its own decision rather than an amendment to this one.

### Known exception: per-language planning hooks in the CLI

One place in the CLI dispatches on language identity outside `lang-api` and outside the registry
builder: `crates/cli/src/commands/plan.rs`, function `planning_hooks` (lines 62-88). It builds the
two boxed closures of a `PlanningHooks` pair, and each closure is a `match` on the candidate
path's `SourceLanguage` with a `SourceLanguage::Scala` arm and a `_` fallback arm. The Scala arm
calls `ariadne_lang_scala::scala_aggregation_filter` and
`ariadne_lang_scala::scala_target_path_for`; the fallback arm calls
`ariadne_lang_cpp::cpp_aggregation_filter` and `ariadne_lang_cpp::cpp_target_path_for`, by name.
The production caller is the `plan_extractions` path at line 210, and its behavior is locked by a
test named `planning_hooks_route_scala_by_language_and_keep_cpp_hook_bytes`.

This is recorded rather than defended, for three reasons.

It sits against the rule the CLI states about itself. The module doc comment at the top of
`crates/cli/src/registry.rs` says that module "stays the CLI's only registry-facing seam - no
other CLI code may reference a concrete plugin type". `plan.rs` names two plugin crates directly
and bypasses the registry for this stage. Read strictly, that sentence forbids a concrete plugin
*type*, and `plan.rs` calls plugin-crate *functions* rather than constructing a plugin, so the
literal wording is arguable. The intent it states -- CLI code reaches plugins through the
registry rather than by name -- is not arguable, and this is the site where the intent is not met.
The same file is inconsistent with itself here: its clustering-constraints dispatch a few lines
further down routes through the registry (`LanguageRoutingConstraints { registry,
language_of_file }`), so the planner hooks are the odd site out.

It is a live review rule R3 exception. R3 requires a `match` on language identity outside
`lang-api` and `cli/src/registry.rs` to be covered by an ADR in `docs/adr/`. No such ADR exists.
The code is explicit that the dispatch was chosen (its call site is annotated as the D-M23-7 i
"per-language planning dispatch replaces the pre-M23 C++ hardcode"), so this is a decision that
was made without the ADR the rules require, not an accidental leak. This subsection records the
exception. It does not approve it, and it should be read as outstanding work rather than as
precedent.

The guard structurally cannot see it. `GUARD_CORE_CRATES` in
`tests/integration/tests/m17_plugin_seam_test.rs` is a fixed seven-entry list:
`crates/index`, `crates/graph`, `crates/cluster`, `crates/transform`, `crates/validate`,
`crates/report`, `crates/pipeline`. `crates/cli` is not among them, so
`core_crates_do_not_name_languages` never walks this file. This is the same walk-scope exclusion
Consequences already records, and the exclusion is defensible for a crate that only routes
through the registry. `plan.rs` shows the CLI does not only route; it also dispatches.

The `plan.rs` closures are the only sites that *match* on language identity, but they are not the
only sites that hardcode a plugin's hooks. An exhaustive search of `crates/cli/src` for the four
hook-pair symbols (`cpp_aggregation_filter`, `cpp_target_path_for`, `scala_aggregation_filter`,
`scala_target_path_for`) returns 16 hits, which classify as follows.

- **Five production dispatch sites, across three files.** The two `match` closures in `plan.rs`
  described above; `crates/cli/src/optimize.rs` line 247, where `analyze_collateral_opportunities`
  applies `cpp_aggregation_filter` unconditionally inside a `.filter()` over the collateral-sources
  map; and `crates/cli/src/optimize.rs` lines 272-281 together with
  `crates/cli/src/commands/ui.rs` lines 359-368, each of which constructs an `OptimizationPlanner`
  with the C++ hook pair and `CyclePolicy::Reject` written in directly (in `plan_safe_extractions`
  and `build_extraction_proposals` respectively). These five sites account for nine of the sixteen
  hits: one each for the `optimize.rs` filter and for each `OptimizationPlanner::new` call, two
  each for the two `plan.rs` closures.
- **One production non-dispatch reference.** The top-level
  `use ariadne_lang_cpp::{cpp_aggregation_filter, cpp_target_path_for};` at
  `crates/cli/src/optimize.rs` line 102, which is what feeds the two `optimize.rs` dispatch sites.
- **Six test-side hits.** All six remaining hits are inside `crates/cli/src/commands/plan.rs`'s
  `mod tests`, which begins at line 275: the assertions and assertion messages at lines 392, 403,
  408, 409, 413, and 414 that pin the current dispatch behavior.

Nine dispatch hits, one import, and six test hits is the sixteen. Only the first group is dispatch.
The three sites outside `plan.rs` never match on language identity, so they are not R3 violations;
they are still places where the C++ aggregation and naming rules are applied unconditionally, and a
new language has to be taught about every one of them. Note that the `optimize.rs` filter at line
247 and the `OptimizationPlanner::new` call at lines 272-281 are two different functions -- one
builds the collateral-opportunity list, the other cuts the plans -- and both are live on the
`optimize` path.

Separately, and not a dispatch site at all: the user-visible error string at
`crates/cli/src/registry.rs` lines 51-52 reads "unknown language '{raw}'; expected cpp, scala, or
other" and names the accepted languages outright. It branches on nothing, so it does not change the
count above, but a new language would want that message updated.

One further item, deliberately typed differently from the five above: a **modelling literal, not a
dispatch site.** `crates/cli/src/commands/plan.rs` line 231, inside the same production
`plan_extractions` that calls `planning_hooks`, passes `ariadne_lang::CyclePolicy::Reject` to the
planner for every language. The modelling table transposed above gives C++ `CyclePolicy::Reject`
and Scala `CyclePolicy::TolerateWithinPackage`, because mutually recursive classes are legal in
Scala, so at this line the planner receives the C++ stance for every language rather than the
plugin's declared `LanguageModeling::cycle_policy`, and report-side cycle classification therefore
uses the C++ stance for every language. The inline comment at lines 227-230 is explicit that this is
not an oversight: the field is informational to the planner, every policy shares the SCC
co-proposal path (locked by the test `cycle_policy_variants_share_the_scc_coproposal_path_for_now`
in `crates/cluster/tests/planner_injection.rs`), and the stance only steers report-side cycle
classification. It is recorded, then, the way the rest of this subsection records its items: as
language-declared behaviour the CLI currently hardcodes, and so as something a new language needs
to be told about -- the same class of thing as the five dispatch sites, and equally not a defect
claim. A third language would find its declared cycle policy overridden here just as it would have
to be taught the five sites above.

Two further locks sit alongside the guards, both in `crates/cli/src/registry.rs`:

- `registries_enumerate_identical_languages` in `crates/cli/src/registry.rs` asserts that the
  CLI-facing registry and the pipeline-facing registry enumerate the same language set and select
  the same unit shaper.
- `default_registry_enums_cpp_then_scala` in the same file asserts the C++-then-Scala
  registration order.

Nothing else substitutes for the two guards. `tools/hygiene_check.py` bounds file and function
size and knows nothing about languages; its `R2_SCANNED_SUFFIXES` set is `.rs`, `.py`, `.html`,
`.js`, so it never even sees a Markdown file.

## Consequences

**Positive.**

- Adding a language no longer requires touching stage logic in any core crate. It is one plugin
  crate, one `.register(...)` line in the named composition root, and fixtures plus tests. Read
  the goal sentence in Context literally, with one honest qualifier: the registry entry *is* an
  edit to a core crate's file, because `crates/pipeline/src/registry.rs` lives in the guard-exempt
  composition root by design. The accurate claim is "zero edits to core stage logic", not "zero
  edits to any core file".
- C++ behaviour is preserved by a permanent parity suite rather than by assertion.
  `tests/integration/tests/m18_cpp_plugin_parity_test.rs` recomputes directive resolution, worker
  dispatch, generated-file bytes, migration-preview JSON, diff text, and reduced-model provenance
  independently in-test, and compares each against production.
- The C++-first registration order is locked by a unit test rather than left to convention.

**Negative and accepted costs, recorded honestly.**

- Two registry facades must be kept in agreement. Because `crates/cli/src/registry.rs` delegates
  to `build_default_registry`, the two are the same object today, so
  `registries_enumerate_identical_languages` is close to tautological. It is kept anyway: its
  value is that a future re-split of the enumeration site cannot silently diverge without
  turning this test red.
- **The C++ plugin is an extraction of existing core code into `crates/lang-cpp`, not a plugin
  authored independently against the seam.** `crates/lang-scala` is the only genuinely foreign
  language here, and it is the only plugin written fresh against `LanguagePlugin`. Rule R1 has
  therefore never been falsified by a language that did not already exist in the codebase: the
  strongest available claim is that one extracted plugin and one independently authored plugin
  coexist under a single registry and a single neutral vocabulary -- not that the boundary has
  survived an arbitrary third language.
- The neutrality guard has structural limits. It is a substring scan over a fixed token list and
  a fixed crate list. A new core crate is not covered until someone adds it to
  `GUARD_CORE_CRATES`. Whitespace-obfuscated plugin names are out of scope by construction.
  `crates/cli`, `crates/scheduler`, and `crates/build` sit outside the walk scope by design,
  because the invariant is analysis-core neutrality and composition roots are named rather than
  anonymous -- so those three crates' neutrality rests on review, not on this test.
- A new language currently needs more than a crate and a registration. It also needs teaching at
  each of the five production dispatch sites in the known-exception subsection above -- two
  `match` closures in `crates/cli/src/commands/plan.rs`, two hardcodes in
  `crates/cli/src/optimize.rs` (line 247 and lines 272-281), and one in
  `crates/cli/src/commands/ui.rs` -- plus a wording update to the error string in
  `crates/cli/src/registry.rs`. Separately, and as a modelling literal rather than a sixth dispatch
  site, a new language needs to know that the cycle policy handed to the planner at
  `crates/cli/src/commands/plan.rs` line 231 is a hardcoded C++ stance rather than the value the
  plugin declares. Closing those edit sites is outstanding work; this ADR records the
  boundary, not a fix for it.
- The vocabulary is not yet used everywhere it could be. The frozen field names listed above
  keep C++ flavor in stored artifacts, and the isolation engine in `crates/transform` still
  knows the concept of a unit generically while the *spelling* of what it writes has moved into
  the plugins. Each of those is a step already taken or deliberately deferred, not a gap; they
  are listed so a later reader does not mistake the boundary for total language erasure.

## What this ADR does not decide

- **Dynamic or out-of-process plugin loading is NOT decided here.** No `libloading`, no WASM
  host, no plugin ABI is chosen, specified, prototyped, or rejected by this record. It is
  currently parked, and the gate for un-parking it is explicit: a real plugin must need
  something the in-process model cannot give it -- a heavyweight third-party dependency we do
  not want statically linked into the binary, fault isolation from a crashing compiler or
  parser, or an independent release cadence for the plugin. If that gate is crossed it requires
  **its own ADR**, because it changes *how rule R1 is enforced*: manifests on disk stop being
  the source of truth, so the Machine-checkable enforcement section above would have to be
  replaced rather than extended.
- No third language is decided here. The pattern is evidenced for C++ and Scala only.
- No change to `Plan.md` milestones M0-M16, and no change to the milestone record itself. The
  landed state of M15 through M23 is recorded in `Plan.md` Part II and is not restated here.
- `tools/clang-worker` stays what it is: a separate-process sidecar for C++ analysis, reached
  through `ClangWorkerBackend`. This record does not generalize it into a plugin host and does
  not take a position on its JSON protocol.
- The design rationale for symbol placement, symbol isolation, and reduced-model provenance is
  not decided here. Those are recorded in the spec's sections 3.5 and 3.6 and in `Plan.md`
  Part II. What this ADR fixes is only that they are expressed in the boundary's neutral
  vocabulary, so that a future language reaches them without editing core crates.
- How the per-language planning-hook dispatch in the CLI should be removed is not decided here,
  including whether the hook pair should become plugin-supplied rather than language-matched.
  Only the exception is recorded, above.