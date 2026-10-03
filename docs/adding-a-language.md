# Adding a language to Ariadnes

This page is for the engineer adding a third language; every name and path below
was read out of the tree. When this page and the code disagree, the code wins.

## Before you start

A new language is 1 new crate, 1 registration line, 1 fixture, 1 suite, and 0
edits to `crates/index`, `crates/graph`, `crates/cluster`, `crates/transform`,
`crates/validate`, `crates/report`, or `crates/scheduler`.

That promise is machine-checked, not merely intended: two guard tests in
`tests/integration/tests/m17_plugin_seam_test.rs` read your manifest and the core
crates' source as text and fail the build when you break the rules.

Two honest qualifiers. The registration line is an edit to a file inside a
guarded crate -- `crates/pipeline/src/registry.rs` is exempt from the neutrality
guard by exact path because enumerating the plugin set is its single duty -- so
the accurate claim is "zero edits to core stage logic". And the CLI, outside that
guard's walk scope, does route per language: `crates/cli/src/commands/plan.rs`
matches a unit's `SourceLanguage` to pick the planner hook pair, so expect to add
an arm there.

Your plugin crate depends inward only: `ariadne-repo`, `ariadne-lang-api`, and
`ariadne-lang` and nothing else, and never `ariadne-graph`, `ariadne-cluster`,
`ariadne-transform`, or `ariadne-validate`.

## The five steps

### Step 1 -- Create the plugin crate

`crates/lang-<x>/`, with a manifest modelled on `crates/lang-scala/Cargo.toml`:

```toml
[package]
name = "ariadne-lang-<x>"
version.workspace = true
edition.workspace = true
authors.workspace = true

# Plugin crate: depends inward only. Deliberately NOT depended on: ariadne-build
# / ariadne-scheduler / ariadne-graph / ariadne-cluster / ariadne-transform /
# ariadne-validate.
[dependencies]
ariadne-repo = { path = "../repo" }
ariadne-lang-api = { path = "../lang-api" }
ariadne-lang = { path = "../lang" }

[dev-dependencies]
tempfile.workspace = true
```

ADR-0001 explains why those three and no more: the repository spine, the neutral
vocabulary, and the plugin traits. Then add `"crates/lang-<x>"` to the `members`
list in the root `Cargo.toml` -- the only core-adjacent file you touch here.

### Step 2 -- Implement the trait surface

Read this against `crates/lang/src/registry.rs` (trait at line 23) and
`crates/lang/src/lib.rs`: `LanguagePlugin` has 12 methods, 10 of them required.
Seven required accessors return a `&dyn` capability trait you also implement, and
that trait's own required methods are:

| accessor | trait (module in `crates/lang/src/`) | required methods |
|---|---|---|
| `detector()` | `LanguageDetector` (`detector.rs`) | `name(&self) -> &str`, `detect(&self, path: &Path) -> SourceFileKind` |
| `analyzer()` | `SemanticAnalyzer` (`analyzer.rs`) | `analyze(&self, file_id: FileId, source: &str, context: &str) -> TuAnalysis` |
| `resolver()` | `DependencyResolver` (`resolver.rs`) | `resolve_edges(&self, file_id: FileId, rel_path: &Path, source: &str, ctx: &ResolutionContext<'_>) -> Vec<RawDepEdge>` |
| `constraints()` | `ClusteringConstraints` (`constraints.rs`) | `colocated_groups(&self, symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>>`, `frozen_symbols(&self, symbols: &[SymbolRecord]) -> HashSet<SymbolId>` |
| `extractor()` | `SourceExtractor` (`extractor.rs`) | `extract(&self, symbols: &[SymbolBody], namespace: &str, required_deps: &[String]) -> GeneratedSourceFile` |
| `rewriter()` | `DependencyRewriter` (`rewriter.rs`) | `rewrite(&self, source: &str, spec: &RewriteSpec) -> String` |
| `validator()` | `CompilerValidator` (`validator.rs`) | `validate_compile(&self, source_files: &[PathBuf], compiler_args: &[String]) -> Result<(), String>` |

Three more methods are required and return data: `fn id(&self) -> LanguageId`,
`fn modeling(&self) -> LanguageModeling`, and
`fn capabilities(&self) -> LanguageCapabilities`. Two more are optional:
`fn analysis_fingerprint(&self) -> String`, default `"v1"` -- bump it when your
parser changes, since it feeds every cache fingerprint; and
`fn shaper(&self) -> Option<&dyn UnitShaper>`, default `None`, meaning the built-in
CMake/VS/compile_commands shaper stays. Overriding it means implementing
`UnitShaper` in `crates/lang/src/unit.rs`: `build_markers`, `default_compiler`,
`shape_units` required, `context_name_for_defines` defaulted.

`modeling()` and `capabilities()` are the two highest-leverage decisions.
`modeling()` returns `LanguageModeling` from `crates/lang-api/src/modeling.rs`:
ten fields -- `language`, `unit` (`UnitKind`), `aggregation` (`AggregationKind`),
`guard` (`GuardKind`), `symbol_placement` (`SymbolPlacement`),
`aggregation_reexports`, `cycle_policy` (`CyclePolicy`), `external_path_markers`,
`dependency_vocabulary`, `analysis_caveat`. This is where the `.h` and TU
assumptions hardcoded in core crates became data.

One trap. `symbol_placement` is a declared, tested record, not a dispatch switch:
no production code branches on it. What makes your language isolate differently
is the hook pair the planner receives -- `OptimizationPlanner`'s
`aggregation_filter` and `target_path_for` fields in
`crates/cluster/src/planner.rs`, supplied as
`ariadne_lang_cpp::{cpp_aggregation_filter, cpp_target_path_for}` and
`ariadne_lang_scala::{scala_aggregation_filter, scala_target_path_for}`. Declare
your placement honestly AND supply both; ADR-0002 Decision 3 is the record.

`capabilities()` returns `LanguageCapabilities` from
`crates/lang-api/src/capability.rs`: `discovery`, `dependency_graph`,
`semantic_index`, `load_metric`, `planning`, `extraction`, `rewriting`,
`validation`, `requires_toolchain`, deciding which stages your language enables.
Leaving a flag `false` produces an honest rejection, never a silent empty
result: `plan_with_rejections` emits one rejection per planning gap, and the CLI
reports that your plugin lacks the requested capability for the target. A unit
no plugin claims is an "unhandled language" error, never a C++ default.

### Step 3 -- Register the plugin

Add exactly one line to `build_default_registry` in `crates/pipeline/src/registry.rs`:

```rust
.register(Box::new(ariadne_lang_x::XPlugin::new()))
```

That function is the single plugin enumeration site, and the CLI's
`default_registry()` in `crates/cli/src/registry.rs` just returns it.

**Registration order is a contract.** The first registered plugin wins unit-shaper
fallback and language-detection precedence, which is why C++ is registered first
and has to stay first. Before you append, check that your detector does not claim
paths the C++ detector claims: a later plugin claiming them changes detection
precedence and can alter C++ results. The one legitimate exception is
`unit_shaper_for_repo`, where a later plugin wins when its markers sit at the
repo root and every earlier plugin's are absent.

### Step 4 -- Add a fixture repository

Put it under `tests/fixtures/`, mirroring `tests/fixtures/scala-basic/`: two
build-manifest files, five sources, a package object, a test-side file, and
`lib/legacy.jar` as the external boundary. Yours needs the same four properties:
a build-manifest file your shaper's `build_markers()` recognises; sources under
your conventional source root; a test-side file the shaper must exclude; and an
external or generated directory exercising the external/frozen boundary. Keep it
small and deterministic -- the suite must pass with no toolchain installed.

### Step 5 -- Add the test suite

Add one file in `tests/integration/tests/` that copies the fixture into a
throwaway `tempfile::TempDir`, drives the CLI handlers on it, and asserts on the
returned models plus the `artifacts/**/manifest.json` files. Copy the pattern of
`m19_scala_discovery_test.rs` (discovery, external boundary, unit shaping),
`m20_scala_dependency_graph_test.rs` (import/export edges),
`m21_scala_semantic_index_test.rs` (index stage, cold and warm),
`m22_scala_load_analysis_test.rs` (load metric), and
`m23_symbol_isolation_test.rs` (plan, preview, apply, rollback).

**The `mNN` prefix is not the plan milestone.** `m17_dir_nav_test.rs` is a UI
directory-navigation feature, not milestone M17; `m19_optimize_characterization_test.rs`
and `m21_run_ui_smoke_test.rs` likewise have nothing to do with the Scala
milestone. The five above plus `m18_cpp_plugin_parity_test.rs` are the
milestone-evidence names; name yours for the milestone it lands in.

## The guard will check you

Two tests in `tests/integration/tests/m17_plugin_seam_test.rs`:

- `plugin_crates_do_not_depend_on_core_pipeline` reads `crates/lang-cpp/Cargo.toml`
  and `crates/lang-scala/Cargo.toml` as text and asserts neither declares
  `ariadne-graph`, `ariadne-cluster`, `ariadne-transform`, or `ariadne-validate`.
  `PLUGIN_CRATES` is a two-element list: your crate is NOT in it until you add it,
  or the guard silently skips you.
- `core_crates_do_not_name_languages` walks every `.rs` file under `src/` in
  `crates/index`, `crates/graph`, `crates/cluster`, `crates/transform`,
  `crates/validate`, `crates/report`, and `crates/pipeline`, and asserts that no
  source line contains any of the seven tokens in `GUARD_FORBIDDEN`: `clang`,
  `g++`, `#include`, `".h"`, `Cpp`, `lang_cpp`, `ariadne-lang-cpp`. One file is
  exempt by exact path, `crates/pipeline/src/registry.rs`.

The neutral-word rule is enforced against production lines AND in-`src` test
lines: a `#[cfg(test)]` mod inside a core crate is walked, so you need no
production code to turn that test red; only `crates/*/tests/` files are out of
scope. Reach instead for: *unit*, *aggregation*, *directive*, *dependency*,
*external boundary*, *isolation unit*, *symbol placement*.

Before you open a review, run `python tools/hygiene_check.py` and the guard
suite; hygiene bounds size for `.rs`, `.py`, `.html`, and `.js` only, so it never
sees a Markdown file.

## What the plugin may not do

- **Do not depend on a core pipeline crate.** It inverts the dependency
  direction. A plugin reaching into `graph`, `cluster`, `transform`, or `validate`
  stops being swappable and the boundary stops being checkable.
- **Do not make a core crate name your language.** Not in a type, not in a
  string, not in a test fixture. Each leaks it into code that must stay neutral
  for the next language.
- **Do not require a toolchain for the suite to pass.** Set
  `requires_toolchain: false` if you work offline, and report
  `ToolchainStatus::Missing` from `toolchain_status()` so tier 3 says `Skipped`.
- **Do not match on language identity outside `lang-api` and the registry
  builder without an ADR.** `crates/cli/src/commands/plan.rs` does this once,
  under ADR-0002 Decision 3: precedent and ceiling. A second such site needs a
  record of its own.

## Honest limits

This page exists because a promise is only as good as its evidence, and that
evidence is thinner than the promise itself.

- **The C++ "plugin" was not authored against the seam.** It was produced by
  extracting existing core code into `crates/lang-cpp`. That proves the seam is
  neutral -- core code can move out and the core stays neutral -- but it does not
  prove a foreign language can be added without friction. `crates/lang-scala` is
  the only genuinely foreign plugin so far, and rule R1 has never been falsified
  by a language that did not already exist here.
- **The guard has only ever seen two plugins, both added by this team in this
  repository.** The boundary is enforced continuously, which is not the same as
  it having been tested against an adversary. A substring scan over a fixed token
  list and a fixed crate list is a floor, not a proof.
- **Dynamic and out-of-process plugin loading is neither available nor decided.**
  No `libloading`, no WASM host, no plugin ABI is chosen, specified, prototyped,
  or rejected. ADR-0001 parks it and says why: a loaded plugin's dependency
  surface is decided by the host at load time rather than by a manifest beside
  the source, so it would change how rule R1 is enforced and needs its own ADR.

## Where to read more

- `docs/adr/0001-language-plugin-boundary.md` -- the boundary decision and the
  guard tests as actually enforced, plus its own honest-limits section.
- `docs/adr/0002-symbol-placement-and-isolation.md` -- what `symbol_placement`
  declares versus what the `aggregation_filter` and `target_path_for` hooks do.
- `plans/multi-language-support-implementation_plan.md` -- the design those two
  ADRs record; read the section your step touches. When this page and
  `crates/lang/src/registry.rs` disagree, the code wins.
