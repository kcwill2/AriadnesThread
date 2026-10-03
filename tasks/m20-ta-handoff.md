# T-A Handoff — ScalaImportResolver + FileCatalog suffix lookup (M20.1)

Kickoff: `plans/m20-scala-graph-kickoff.md` (D12, fingerprint ruling, protocol). Base: commit after the M20 docs commit. You are one bounded implementer session: touch-set below, scoped commands, commit on green.

## 1. Intent & Invariants

**Objective:** `ScalaImportResolver: DependencyResolver` replaces `stubs::StubResolver` inside `ScalaPlugin`: `resolve_edges` recognizes Scala `package` clauses, `import a.b.C`, `import a.b.{C, D}`, `import a.b._`, bare `import a.b`, and Scala 3 `export`; `resolve_target` maps dotted package paths to repository files via a new deterministic suffix lookup on `FileCatalog`, with the plugin's external-boundary markers winning before any lookup.

**Invariants:**
- IV-1 A `package` clause is scope context, never a dependency edge. `resolve_edges` emits edges only for `import`/`export` directives.
- IV-2 Grouped selector `import a.b.{C, D}` yields one `RawDepEdge` per selected name (`raw_target = "a.b.C"`, `"a.b.D"`), `is_wildcard = false`. Wildcard `import a.b._` yields exactly one edge with `raw_target = "a.b"`, `is_wildcard = true`. Bare `import a.b` (no selector, not a rename/`given` clause — those may be ignored as noise, document the choice) is treated as wildcard `a.b`.
- IV-3 Every Scala edge carries `DirectiveStyle::Unspecified`, true `line`/`column` of the directive keyword, `source_file`/`source_rel_path` from the arguments.
- IV-4 Directives inside `//` line comments, `/* */` block comments, string literals, and string interpolation (`s"...$x..."` with braces) produce nothing. Comments may span lines; block comments nest? Scala block comments **do nest** — implement nesting depth.
- IV-5 `resolve_target`: marker first (`lib`/`target`/`.ivy2`/`.cache` as the **first** segment of `raw_target` ⇒ `External` without lookup); then suffix lookup on `FileCatalog` — 1 match ⇒ `Resolved`; >1 matches ⇒ `Wildcard(sorted)`; 0 matches ⇒ `External`; a matched record whose rel-path contains a marker component ⇒ `External`.
- IV-6 `EXTERNAL_PATH_MARKERS: &[&str] = &["lib", "target", ".ivy2", ".cache"]` is defined once (e.g. `lang-scala/src/boundary.rs`), consumed by `plugin.rs::modeling()` (the M19 lock `modeling_matches_plan_section_3_4_scala_column` + `external_frozen_boundary_lock` in `m19_scala_discovery_test.rs` must stay green with identical values) and by the resolver. No new dependency crates.
- IV-7 `analysis_fingerprint()` on `ScalaPlugin` returns `"scala-1"` (was trait default `"v1"`; resolver = new analysis behavior). Keep/adjust any test that asserts fingerprint values with an honest comment; **C++'s fingerprint must not move** (m18 parity).
- IV-8 Stub retirement is resolver-only: `StubAnalyzer`/`StubConstraints`/`StubExtractor`/`StubRewriter`/`StubValidator` stay, `stubs.rs` module doc updated (it currently says "the analyzer and resolver stubs …"); do not touch capabilities.
- IV-9 The resolver is stateless (`Clone, Copy, Default` like `CppDependencyResolver`); no filesystem access — everything goes through `FileCatalog` + `ResolutionContext` (search paths may serve as source-root hints; do not require them).

## 2. API contracts

```rust
// crates/lang-api/src/catalog.rs — additive
/// Deterministic (sorted-by-module-key) records whose module path ENDS WITH the
/// given dotted/slash module path. `com.example.models.User` matches
/// `src/main/scala/com/example/models/User.scala`; `com.example/models` matches
/// all files under that package dir. Zero matches = empty Vec.
pub fn by_module_path_suffix(&self, module_path: &str) -> Vec<FileId>;

// crates/lang-scala/src/resolver.rs — new
#[derive(Debug, Clone, Copy, Default)]
pub struct ScalaImportResolver;
impl DependencyResolver for ScalaImportResolver {
    fn resolve_edges(&self, file_id: FileId, rel_path: &Path, source: &str,
                     ctx: &ResolutionContext<'_>) -> Vec<RawDepEdge>;
    fn resolve_target(&self, edge: &RawDepEdge, catalog: &FileCatalog,
                      ctx: &ResolutionContext<'_>) -> ResolveOutcome;
}

// crates/lang-scala/src/lib.rs — re-export ScalaImportResolver (public for plugin + tests)
```

## 3. Plan-Task (RGB)

- **Red (write first, watch fail):**
  - `crates/lang-api/src/catalog.rs` `#[cfg(test)]`: suffix lookup hits with `/`↔`\`↔`.` unification (`com/example/models/User`, `com.example.models.User`), package-dir prefix hits (3 fixture-shaped files incl. `all.scala`), non-match ⇒ empty, deterministic order across two constructions.
  - `crates/lang-scala/src/resolver.rs` `#[cfg(test)] mod tests` (fixture-shaped catalogs): simple import ⇒ 1 edge Unspecified/pos; grouped `{Order, User}` ⇒ 2 named edges; `._` ⇒ 1 wildcard edge target `a.b`; bare `import a.b` ⇒ wildcard; `export a.b.C` ⇒ edge; `package com.example.models` alone ⇒ **0 edges**; `// import x.y`, `/* import x.y */` (incl. nested `/* /* */ */` and multi-line block), `val s = "import x.y"`, `val t = s"${x}"` containing `import` ⇒ 0 edges; line/column correctness.
  - resolve_target tests: `com.example.models.Order` ⇒ `Resolved(id)`; `com.example.models._` ⇒ `Wildcard([User, Order, all…sorted])`; `scala.collection.mutable` ⇒ `External`; `lib.legacy.Foo` ⇒ `External` (marker, no lookup); synthetic catalog containing a record at `lib/jars/Foo.scala`, target `jars.Foo`? — construct so the record's rel-path component `lib` trips marker ⇒ `External`; 0-match ⇒ `External`; multi-match named ⇒ `Wildcard`.
  - plugin test: `ScalaPlugin::new().resolver()` downcast/type is `ScalaImportResolver` behavior — at minimum `resolve_edges` on real fixture text `tests/fixtures/scala-basic/src/main/scala/com/example/services/OrderService.scala` (read from disk) yields the two named edges; fingerprint test for `"scala-1"`.
- **Green:** minimal hand-written scanner — line loop with state for line-comment / block-comment (depth counter) / string / interpolation; per logical line extract at most one directive (Scala allows one import per line; multiple directives on one line is out of scope — document). Split: `directive_parser.rs` (pure text → `Vec<ParsedImport { raw_target, is_wildcard, line, column }>`) + `resolver.rs` (edges + target resolution) so neither file is a monolith. Reuse `ariadne_lang_api` normalization helpers where they fit; do not duplicate `module_key`.
- **Refactor / Hygiene:** every fn < 100 lines (scanner loop helper `scan_directive_line` decomposed); files < 1000; predicate-bool names (`is_build_definition` style already in detector); no `tmp`/`res`/`data`; `python tools/hygiene_check.py` exit 0.

## 4. Touch-set

`crates/lang-scala/**` (new `resolver.rs`, optional `directive_parser.rs` + `boundary.rs`, `plugin.rs` swap + fingerprint, `stubs.rs` doc, `lib.rs` re-export, `Cargo.toml` unchanged deps) · `crates/lang-api/src/catalog.rs` (one additive fn + tests) · **nothing else.**

## 5. Scoped commands (never `--workspace`)

```powershell
cargo test -p ariadne-lang-api
cargo test -p ariadne-lang-scala
cargo test -p ariadne-integration-tests --test m19_scala_discovery_test   # marker + capability locks
python tools/hygiene_check.py
```

## 6. Clauses

- Targeted diffs over full-file reads; commit immediately on green (`feat(m20.1): ScalaImportResolver …`); checkpoint report to disk beats context overflow.
- UTF-8 discipline per kickoff; verify `git diff --numstat` after any scripted touch.
- If a pre-existing test contradicts an IV, STOP and report (do not silently edit m17/m18/m19 suites).
