# M21 T-C Handoff — Filter-driven unit shaping (index stage, D15)

Depends on: T-B merged. Decision ref: D15. Kickoff: `plans/m21-scala-index-kickoff.md`. **Read `crates/pipeline/src/lib.rs` around `compute_inputs` (~line 264–286), `rebuild_index_single_threaded` (~434–478), and `LanguageRegistry::unit_shaper_for` (`crates/lang/src/registry.rs:120-122`) before touching anything.**

## Problem (verified at `77bd0a7`)

`Pipeline::compute_inputs` calls `registry.unit_shaper_for()` unconditionally — first registered shaper wins ⇒ C++ always. On an sbt repo the C++ shaper finds no markers ⇒ 0 translation units ⇒ `ariadne index --lang scala` produces an empty index even after T-A/T-B land. M20 worked around this for the graph stage via an injected `&dyn UnitShaper` (D14); the index stage must select the shaper from the **already-explicit language filter** instead (D15).

## Touch-set (nothing else)

- EDIT `crates/pipeline/src/lib.rs` — `compute_inputs` only.
- Pipeline unit tests (wherever `pipeline_tests.rs` lives; colocated is fine).
- **Not**: `crates/lang` (the registry API already has `plugin_for_id`), CLI (`build_source_index_filtered` already threads the filter), scheduler, build, graph. **Zero language nouns** in any added line (R2: *unit, directive, dependency, external boundary* vocabulary only) — the m17 `core_crates_do_not_name_languages` guard plus a new pipeline-side review focus enforce this.

## Contract

```rust
// compute_inputs, replacing the current `.with_unit_shaper(registry.unit_shaper_for())` line:
let shaper: Option<&dyn ariadne_lang::UnitShaper> = match self.language_filter {
    Some(lang) => registry.plugin_for_id(lang).and_then(|p| p.shaper()),
    None => registry.unit_shaper_for(),   // existing first-wins call, byte-identical path
};
```

Exact binding shape is yours (lifetimes via a local `Option<Arc<...>>`-free borrow); the invariants are:

1. `language_filter == None` ⇒ **the old code path runs unchanged** (C++ fixtures and every existing suite must not observe a difference — m00 goldens are the witness).
2. `Some(lang)` with a plugin that has no shaper ⇒ `None` shaper (today's effective behavior for filtered runs anyway — document it).
3. No `match LanguageId` anywhere (R3). Registry lookups only.
4. If `Some(lang)` selects a shaper whose `shape_units` yields units of other languages too, leave that alone — per-unit language filtering already happens downstream (`plugin_for_path` + closure language scoping); do NOT add a second filter here.

## RGB plan

- **Red** (pipeline test module): build a temp repo copying `tests/fixtures/scala-basic` (pattern: m19's `isolated_scala_repo`); `Pipeline::with_registry(root, None, build_default_registry()).language_filter(Some(SourceLanguage::Scala))` → assert `get_or_build_index()` returns **5 tu_analyses** (fails today: 0). Counterpart negative-neighbor: `language_filter(None)` on a copy of `tests/fixtures/basic` (C++) still yields the same TU count as before your change (assert current expected count explicitly — read today's number, don't guess).
- **Green**: the match above. **Refactor**: extract `fn select_unit_shaper(&self, registry) -> Option<&dyn UnitShaper>` if `compute_inputs` drifts toward the 100-line hygiene ceiling.

## Commands & gates

- `cargo test -p ariadne-pipeline` green; scoped witnesses: `cargo test -p ariadne-integration-tests --test m00_golden_baseline_test` (**zero golden drift**) `--test m17_plugin_seam_test --test m18_cpp_plugin_parity_test` (run m18 too — it locks the default-registry dispatch shape).
- `python tools/hygiene_check.py` exit 0. No `--workspace`.
- Commit: `feat(m21): language-filtered index stage selects the plugin's unit shaper (D15)`.
- Report: SHA, the exact diff of the shaper-selection edit, confirmation m00 report.json/tu-counts byte-identical, m17 guard green with the new pipeline line (no language nouns).
