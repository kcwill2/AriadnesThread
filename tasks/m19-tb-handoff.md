# T-B Handoff — M19.3 + D9: register Scala, resolve registry duplication, fixture + m19 suite

Precondition: T-A landed `crates/lang-scala` (ScalaPlugin/ScalaDetector/ScalaUnitShaper) — verified green. Read `tasks/m19-ta-handoff.md` §"What to build" for its contracts (do not re-implement or modify that crate).

Binding inputs (targeted reads): this file; `plans/m19-scala-discovery-kickoff.md` (D9, acceptance, protocol); plan §5 M19 table lines 377–387 (`plans/multi-language-support-implementation_plan.md`); `crates/pipeline/src/lib.rs` lines 42–52; `crates/cli/src/registry.rs` (whole, 91 lines); `crates/cli/src/commands/analyze.rs` lines 20–55; `crates/cli/src/commands/transform.rs` sole_plugin sites (~30, ~92, ~136, ~263); `crates/cli/src/optimize.rs` sites (~368, ~379); `crates/repo/src/discovery.rs` line 53; `crates/build/src/discovery.rs` lines 38–65 + 200–265 (shaper plumbing); `tests/integration/tests/m18_cpp_plugin_parity/mod.rs` (isolated-repo pattern, lines 40–70).

## Architect rulings baked into this handoff (verify, don't re-litigate)
1. **sole_plugin blast radius:** adding Scala to `default_registry()` makes `sole_plugin()` → `None`, which would break `m10`/`m12`/`m14`/`m20` (they call `apply_transform_plan` / `build_migration_preview` / `run_incremental_analysis`, which route through it). Fix = capability-gated backend selection (below) — Scala's `false` capability literals (plan:381) keep C++ the sole text backend, byte-identically for C++ repos.
2. **D9 direction:** cli depends on pipeline, never the reverse (cycle). The shared builder therefore LIVES IN PIPELINE; cli delegates.
3. **repo discovery filter:** `crates/repo/src/discovery.rs:53` `kind.is_source_or_header()` never records `.scala` (plan:387's "repo needs no functional change" predates M17 defaults). Ruled: generalize to repo's own neutral `kind.is_analyzable()` (already includes `ScalaSource`, excludes `.sc`/BuildDefinition — exactly plan:387's ".scala files listed").
4. **`lib/` boundary wording deviation (accepted, documented):** shipped `policy.rs` has NO `lib` rule and traversal SKIPS `target/` dirs entirely, so "lib/+target/ → ThirdParty/Generated via existing policy.rs" cannot mean discovery classification. The m19 suite locks the REAL boundary behavior + the M20 marker lock (spec below). Do NOT edit `crates/repo/src/policy.rs`.

## Touch-set
- `crates/pipeline/src/lib.rs`, `crates/pipeline/Cargo.toml` (lang-scala dep — D9 helper mandate)
- `crates/cli/src/registry.rs`; `crates/cli/Cargo.toml` (only: remove now-dead `ariadne-lang-cpp` dep after delegation — verify `rg ariadne_lang_cpp crates/cli/src` is empty first; keep `ariadne-lang`)
- **Pre-justified §3.1 exceptions (commit body must carry this justification):** `crates/cli/src/commands/transform.rs`, `crates/cli/src/optimize.rs`, `crates/cli/src/commands/analyze.rs` — the M18.8b `sole_plugin()` seam documented itself as transitional ("per-language split lands with M19", `crates/lang/src/registry.rs:124–129`); the capability literals of plan:381 are the plan's own gating mechanism; handlers stay plugin-type-free, `registry.rs` remains the only site naming plugins.
- `crates/repo/src/discovery.rs` (ONE filter term + one unit test; justification ruling 3)
- `tests/fixtures/scala-basic/**` (new), `tests/integration/Cargo.toml` (dev-dep lang-scala), `tests/integration/tests/m19_scala_discovery_test.rs` (new)
- `Cargo.lock`
Nothing else. `tests/golden/**`, `tests/fixtures/basic|expensive`, the m18 parity suite + captures are FROZEN byte-identical.

## Implementation plan (RGB)

### Step 1 — D9 red
Extend `crates/cli/src/registry.rs` tests FIRST: rename `default_registry_enums_cpp_only` → `default_registry_enums_cpp_then_scala` (`languages() == [Cpp, Scala]`, `.scala` detect is now `Some`); `ensure_language_registered_accepts_cpp_rejects_scala` → `accepts_cpp_and_scala_rejects_other`; add `registries_enumerate_identical_languages` (cli vs `ariadne_pipeline::default_registry()` — same language vec AND same `unit_shaper_for()` build_markers). Run `cargo test -p ariadne-cli registry` → RED (missing Scala). Capture.

### Step 2 — shared builder (pipeline)
`crates/pipeline/Cargo.toml`: `ariadne-lang-scala = { path = "../lang-scala" }`. In `lib.rs`:
```rust
/// The single plugin enumeration site (D9 consolidation): C++ first, then Scala.
/// Registration ORDER is a contract — it drives `unit_shaper_for()` and
/// `plugin_for()` precedence so existing C++ discovery stays byte-identical.
/// Scala discovery selects its shaper explicitly (M19.3) — automatic
/// marker-based shaper selection is the deferred M18.5 gap.
pub fn build_default_registry() -> LanguageRegistry {
    LanguageRegistry::new()
        .register(Box::new(ariadne_lang_cpp::CppPlugin::new()))
        .register(Box::new(ariadne_lang_scala::ScalaPlugin::new()))
}
pub fn default_registry() -> Arc<LanguageRegistry> { Arc::new(build_default_registry()) }
```
Update the stale doc comments (pipeline lines 42–47, 168–172). `crates/cli/src/registry.rs::default_registry()` → `ariadne_pipeline::build_default_registry()`; rewrite the module doc to the delegation story. Run `cargo test -p ariadne-pipeline`, `cargo test -p ariadne-cli` → green.

### Step 3 — capability-gated CLI backends (replace every `sole_plugin()` consumer)
In `crates/cli/src/registry.rs` add:
```rust
/// Stage backend: the one registered plugin whose capabilities enable `capability`.
/// `Some` only on exactly-one claim; ambiguous/none → `None` (callers keep their
/// existing error/passthrough semantics). Language-neutral: no `match LanguageId` (R3).
pub fn capability_backend<'a>(
    registry: &'a LanguageRegistry,
    capability: fn(&ariadne_lang::LanguageCapabilities) -> bool,
) -> Option<&'a dyn ariadne_lang::LanguagePlugin>
```
(iterate `registry.languages()` → `plugin_for_id` → filter → exactly-one). Unit-test it on `default_registry()`: `extraction`, `rewriting`, `validation` each resolve to the C++ plugin (Scala's false literals); empty registry → None.
Swap sites (behavior must be identical for C++-only registries):
- transform.rs ~30 → `capability_backend(&registry, |c| c.extraction).map(|p| p.extractor())` (same error fallback)
- transform.rs ~92 & ~136, optimize.rs ~368 → `|c| c.rewriting`
- transform.rs ~263 & optimize.rs ~379 → `|c| c.validation` (None → passthrough, unchanged)
- analyze.rs ~45–47: replace `sole_plugin().ok_or_else(...)` with a `registry.rs` helper `incremental_backend(&registry, &build_discovery) -> Result<&dyn LanguagePlugin, String>`: distinct `tu.language` set over `build_discovery.translation_units` → map `plugin_for_id` (missing plugin ⇒ "unhandled language" error, never a C++ default); exactly one ⇒ that plugin; ZERO units ⇒ fall back to `registry.sole_plugin()` (empty-repo behavior unchanged); >1 distinct ⇒ explicit error "multi-language incremental analysis lands with the language-scoped cache (M21)". Update the stale error comment. Green: `cargo test -p ariadne-cli`.

### Step 4 — repo discovery filter
`discovery.rs:53` → `if kind.is_analyzable()`. Grep `is_source_or_header` callers first (only this site may change; elsewhere leave untouched). Add a unit test: tempdir with `a.cpp`, `b.h`, `c.scala`, `d.sc`, `build.sbt` → records == {a.cpp, b.h, c.scala}; `c.scala` has `language: Scala`. Green: `cargo test -p ariadne-repo`.

### Step 5 — fixture + suite (red → green)
`tests/integration/Cargo.toml` dev-deps += `ariadne-lang-scala = { path = "../../crates/lang-scala" }` (`ariadne-lang` already present).
`tests/fixtures/scala-basic/` (pure ASCII): `build.sbt` (name/scalaVersion/version lines), `project/build.properties` (`sbt.version=1.10.1`), `src/main/scala/com/example/Main.scala` (`package com.example`, object Main), `models/{User.scala,Order.scala,all.scala}` (last is `package com.example; package object models {}`), `services/OrderService.scala` (`package com.example.services`, `import com.example.models.{Order, User}`, class OrderService), `src/test/scala/com/example/OrderServiceSpec.scala`, `lib/legacy.jar` (ASCII sentinel bytes). Check root `.gitignore` for `target`: if ignored (likely), do NOT commit any `target/**` — the suite creates `target/scala-2.13/classes/Generated.scala` inside its temp copy at runtime instead.
`m19_scala_discovery_test.rs` (self-contained recursive-copy helper; 5 tests, names must show up in the run log):
1. `discover_lists_scala_sources` — records == exactly the 6 `.scala` paths (5 main + 1 test spec); all `kind: ScalaSource`, `language: Scala`, `ownership: Repository`, `optimization_policy: Optimizable`; no `build.sbt`/`project/build.properties`/`lib/` records.
2. `external_frozen_boundary_lock` — after injecting `target/scala-2.13/classes/Generated.scala` into the copy: zero records under `target/` or `lib/`; direct `classify_ownership_and_policy("target/scala-2.13/classes/Generated.scala") == (Generated, Frozen)`; `ScalaPlugin::new().modeling().external_path_markers` contains `lib` and `target` (M20 enforcement lock). Comment records the kickoff-wording deviation (ruling 4).
3. `shaper_yields_scalac_units` — `BuildDiscoverer::new(root).with_repo_discovery(&disc).with_unit_shaper(Some(&ScalaUnitShaper)).discover()`: exactly 1 context `{ name: "root", compiler: "scalac", language: Scala }`; exactly 5 translation units == the `src/main/scala` files, `language: Scala`; no `cl.exe`/C++-flavored context anywhere (compile_commands path not entered).
4. `plugin_contract_in_default_registry` — `ariadne_cli::registry::default_registry()`: `languages() == [Cpp, Scala]`; Scala capabilities `assert_eq!` against the FULL plan:381 literal (blocker); `detect_language` Scala/Cpp paths; `unit_shaper_for()` markers == C++ markers (registration-order lock).
5. `d9_registries_identical` — cli vs pipeline: identical `languages()` and `unit_shaper_for()` markers (belt-and-braces on the Step 1 unit lock, cross-crate view).
NOTE: dev-deps available in the package include repo/build/cli/pipeline/lang/lang-scala/tempfile — if you need `ariadne-lang` types in this test the dev-dep already exists (plan §6.3 wording about "gains ariadne-lang" was satisfied earlier).

### Step 6 — witness gate (this is the blast-radius proof, run ALL)
`cargo test -p ariadne-integration-tests --test m19_scala_discovery_test` then witnesses:
`--test m17_plugin_seam_test --test m00_golden_baseline_test --test m18_cpp_plugin_parity_test --test m10_validation_test --test m12_incremental_analysis_test --test m14_end_to_end_test --test m20_apply_migrate_characterization_test`
(multiple `--test` flags in one command; fall back to per-suite if rejected). m17 must report **5 passed**; m00 **2 passed**; m18 **22 passed**. `python tools/hygiene_check.py` exit 0. `git diff --numstat` vs HEAD before committing: ONLY touch-set files.

### Step 7 — commit
`git add` exactly the touched files + `Cargo.lock`; message:
`feat(m19.3,d9): Scala registration via shared pipeline registry builder; capability-gated CLI stage backends; scala-basic fixture + m19 discovery lock`
Commit body MUST include: the §3.1 justification for the cli-handler edits (ruling 1 + the "lands with M19" registry.rs:124–129 quote), the repo-filter justification (ruling 3), and the lib/-boundary wording deviation note (ruling 4).

## Blockers — STOP and report, never improvise
- Any witness suite failing for a reason outside these rulings; any m00 golden drift; any file outside the touch-set needing edits; the m17 guard flipping on lang-scala's manifest.

## File-edit discipline (kickoff §Orchestration, non-negotiable)
Write/Edit tools only — NEVER `Set-Content`/`Out-File`/`>` or PowerShell text rewriting (UTF-8 corruption correction `tooling.powershell_utf8_file_edits`). All new content pure ASCII. No `scalac`/`sbt` invocation ever (A3 offline).

## Context discipline
- Targeted diffs/greps over full-file reads; read only the files listed at top.
- Checkpoint report beats context overflow: if context is running out, stop and return a disk-state inventory (files changed, gates run + results, steps remaining) — a dead session is never resumed; the orchestrator restarts fresh from the inventory.
- NEVER `cargo test --workspace` (that is the orchestrator's final gate). Scoped commands above, timeouts 600000 ms.

## Report back
1. Commit hash + `git show --stat`. 2. Red evidence per step. 3. Green + witness test counts (m17 5/5, m00 2/2, m18 22/22 explicitly). 4. Every deviation from this spec + why. 5. Observations outside touch-set (report-only).
