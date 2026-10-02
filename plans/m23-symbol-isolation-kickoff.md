# M23 Kickoff — Symbol isolation, shared engine (C++ + Scala)

Spec source: `plans/multi-language-support-implementation_plan.md` §3.4 (Isolation unit row), §3.6 (isolation primitive + division of labor), Deferred-M23 bullets (lines 432–436), carried items from M18.11 notes (line 564) and M21 notes (line 415).
Base: branch `multi-language-support` @ `76f619c` (M22 closed, final gate 916/0/3).
Environment ruling: no Scala toolchain on this machine (`Get-Command scalac, scala, scala-cli, sbt` → empty). Scala tier 3 therefore deterministically `Skipped` here; the probe must degrade the same way in CI.

## 0. Corrections to the plan's premises (verified 2026-10-02)

1. `crates/transform/src/extractor.rs` no longer exists (plan:286/433 cite `extractor.rs:72`). The placeholder is **`crates/transform/src/engine.rs:76`**: `format!("// Symbol: {} ({:?})", ext.symbol_name, kind)`.
2. The `// Symbol:` byte-locks are **`engine.rs:276`** (`test_extract_basic`, exact string) and **`tests/integration/tests/m18_cpp_plugin_parity/isolation_output_parity.rs`** `BASIC_PLAN_LOCKS` (:57–69) / `EXPENSIVE_PLAN_LOCKS` (:82–90) full-file bytes. The `preview_json`/`diff_text` locks (:72–73, :93) carry **no bodies** and must survive untouched — they are the proof the neutral preview path did not change. Recapture tool: `zz_capture_isolation` (same file :407–442).
3. There is no `ValidationTransaction` type; transactional state = `FileSnapshot` + `ApplyOutcome` (serde names locked by `m20` :208 and validate :581–597). Nothing here changes them.
4. `format_diff` (plain) no longer exists — only `format_diff_with(&dyn DependencyRewriter)`.
5. `TransactionApplier::apply_with` **already** routes consumer rewrites through the plugin `DependencyRewriter` (`validate/src/lib.rs:217`). What is NOT yet generalized: generated-file text is hand-rolled inside apply (`emit_file_preamble` + raw `sym.body` pushes, :194–199) — a **second body emitter** that cannot render a Scala package clause. And the 3-arg `apply` shim (:146–153) + `TransitionalDefaultRewriter` (:132–138) survive with exactly three callers: validate in-src test :516, `m10_validation_test.rs:115`, `m14_end_to_end_test.rs:126`.
6. Body/span text exists nowhere in the data model: C++ tokenizer tokens carry no lexeme/offset (`lang-cpp/src/tokenizer.rs:2-6`); `SymbolDecl`/`FileLocation` are line/column only (`lang-api/src/types.rs:56-79`); shards/tu_analyses persist no source text. The Scala parser computes token-index spans internally and discards them by design (`lang-scala/src/parser/declarations.rs:7-8` "Span text is not stored (M23 concern)").
7. CLI `plan` still hardcodes the C++ planner injection (`cli/src/commands/plan.rs:57-60`); Scala planner hooks exist only in tests. Scala's `PlanRejection` is driven solely by `planning: false` (`lang-scala/src/plugin.rs:140`).
8. Capability backends resolve via exactly-one-claim (`cli/src/registry.rs:59-76`); flipping Scala capabilities true makes global C++-only backend resolution ambiguous → backend selection must become language-routed.

## 1. Intent statement

Make the isolation primitive real for both languages without breaking the neutrality that M17–M22 bought: the engine feeds **verbatim symbol text** (not placeholders) through the plugin extractor, the applier emits through the plugin extractor (single emitter), and Scala gains a genuine extractor/rewriter/validator so `plan → preview → apply` produces an isolated symbol file per language. C++ output changes exactly once, deliberately: generated extracted files carry real declaration text. That is the milestone's only sanctioned C++ byte drift, under its own golden refresh + reviewer diff sign-off.

## 2. Invariants

- **I1** C++ consumer-facing bytes are unchanged: `MigrationPreview` JSON, diff text, plan JSON except additive `source_file` (D-M23-3), rewrite substitution bytes. Only generated extracted-file bodies change (placeholder → verbatim).
- **I2** `transform`, `validate`, `cluster`, `graph`, `index`, `report` name no plugin or language in production or in-src test code (guard hardened, D-M23-10).
- **I3** One body emitter: every generated file (engine `write_to_disk` and `apply_with`) is rendered by `SourceExtractor::extract`; the applier never concatenates preamble+bodies itself.
- **I4** Fallback is total: if a body cannot be located (no source content, symbol not found), the engine falls back to today's placeholder string — deterministic, visible, and the placeholder unit tests keep meaning.
- **I5** Scala isolation is a **move**, snapshot/rollback-capable via the existing `FileSnapshot` transaction; no duplicate top-level symbol remains in the source file after apply. C++ stays copy-only (M18 behaviour-preserving rule).
- **I6** No breaking artifact-schema change: all new serde fields are `#[serde(default)]` (additive), epoch untouched (2.0.0) because `symbols.bin` layout does not change.
- **I7** Everything stays offline-deterministic: toolchain probes resolve to `Missing` on this machine and in CI; tier 3 = `Skipped`; no network, no Scala install assumed.
- **I8** Hygiene: functions < 100 lines, files < 1000 lines, predicate/intention-revealing names; `m17` guards pass after the milestone.

## 3. Decisions (D-M23-*)

- **D-M23-1 Body location = extract-time plugin locator, not analyzer-persisted spans.** Deviation from the plan's literal "feed parser spans from `CppSemanticAnalyzer`" (plan:433): persisting spans/text in `SymbolDecl`/shards/`TuAnalysis` would grow every artifact, force analyzer+tokenizer surgery, and re-open m00 index goldens. Instead, the engine hands the plugin the raw text of each extraction's source file and asks it to locate verbatim declaration blocks (`locate_symbol_bodies`); `lang-cpp` reuses its tokenizer (extended internally with char offsets), `lang-scala` reuses `ScalaSourceParser` spans mapped to offsets via a line-index table. Same observable output, no schema bloat. Logged as a Gate-G1 reviewer deviation.
- **D-M23-2 Namespace seam finally wired.** Engine resolves the unit's namespace through a new default trait hook `SourceExtractor::unit_namespace(source_text) -> String` (C++ default `""`; Scala returns the file's package clause — re-derived, since block/label packages are unsupported by the M21 parser, so file-level leading-`package` re-derivation matches analysis-time qualification) and passes it as the **existing** `namespace` parameter of `extract` (`engine.rs:186` today hardcodes `""`). C++ ignores it (locked by `namespace_argument_is_ignored` — still passes).
- **D-M23-3 Engine body context.** `ProposedExtraction` gains `source_file: Option<FileId>` (`#[serde(default)]`), set by the planner from the candidate file id. `IsolationEngine::extract` gains an injected content provider `sources: &dyn Fn(FileId) -> Option<String>` (CLI/callers read disk; tests pass `|_| None` → placeholder fallback). Per target-file group the engine dedupes source files, calls `extractor.locate_symbol_bodies(text, &[(name, kind)])`, keeps placeholder for anything unresolved.
- **D-M23-4 Single emitter + shim death.** `apply_with` gains an `extractor: &dyn SourceExtractor` parameter and builds generated content exactly like `write_unit` (`extract(bodies, namespace, deps)`); delete the hand-rolled emit, the 3-arg `apply` shim, and `TransitionalDefaultRewriter`; migrate the three frozen call sites together. `write_to_disk` must `create_dir_all` per unit parent (Scala targets are nested package dirs; gap acknowledged at `isolation_output_parity.rs:374-377`).
- **D-M23-5 Scala isolation = move.** For `SymbolPlacement::SingleFile`, apply (a) writes the new `{symbol}.scala` with package clause + verbatim top-level text + only the imports the moved region actually references, and (b) removes the moved region from the source file — both via the plugin, both inside the snapshot/rollback transaction (`DependencyRewriter` gains a default hook `remove_top_level_symbols(source, &[name]) -> Option<String>`; C++ default = `None` → engine never removes, copy semantics preserved; Scala implements via parser spans). Consumer import lines stay valid under package resolution, except aggregate-wide wildcard imports, which the rewriter narrows to the new file's module path when the old aggregate no longer carries the symbol.
- **D-M23-6 Capability flip + probe validator.** `ScalaPlugin::capabilities`: `planning/extraction/rewriting/validation → true`; `requires_toolchain` stays `false`. Replace `StubValidator` with `ScalaToolchainValidator`: `toolchain_status()` probes `scalac`/`scala-cli`/`sbt` (Command::new version probe); `validate_compile` errors only when a toolchain is present and the compile fails — offline path = `Missing` → tier 3 `Skipped` via the existing `TierValidator` handling (`validate/src/lib.rs:339-344`). `StubExtractor`/`StubRewriter` are deleted, not kept behind flags.
- **D-M23-7 Language-routed wiring.** (i) CLI `plan`: build per-language dispatch for `aggregation_filter` / `target_path_for` / `constraints` / `cycle_policy` from the registry + `FileRecord.language` — replaces the C++ hardcode at `plan.rs:57-60` with identical C++ behaviour (single-language C++ repos must plan byte-identically). (ii) Backend selection (`capability_backend` callers in `transform.rs`/`optimize.rs`) resolves per unit/consumer **language** through registry+catalog, fixing the exactly-one-claim ambiguity the flip creates. (iii) Consumer discovery: the raw `#include "<target>"` substring scan (`transform.rs:228`, `optimize.rs:314-346`) moves behind a neutral `DependencyRewriter` hook (`find_dependency_targets(source_text, target) -> Vec<DirectiveHit>`); C++ implementation reproduces today's scan byte-for-byte; Scala scans import directives (reusing `directive_parser`). (iv) `SymbolPlacement` needs no new planner hook: placement semantics are realized by each language's filter/target-pair; add a debug assertion + doc line, not branching.
- **D-M23-8 Sealed constraints.** Parser records a `sealed` modifier per top-level decl; `SymbolRecord` gains additive `modifiers: Vec<String>` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]` → C++ index goldens unchanged). `ClusteringConstraints` gains `colocated_groups_with_edges(&[SymbolRecord], &[(SymbolId, SymbolId)])` with default delegating to `colocated_groups` (C++ untouched); the planner feeds extends edges from the reduced model; Scala groups `sealed trait` ↔ direct subtypes alongside the existing companion pairing. Update `constraints.rs` `sealed_pairing_scope_locked` (it currently locks the gap — rewrite as the positive lock).
- **D-M23-9 Fixture strategy.** New fixture `tests/fixtures/scala-isolation`: one shared file `legacy.scala` in a package holding ≥2 top-level classes plus a package-object aggregation file and a consumer that wildcard-imports the package, sized so the Scala planner proposes ≥1 extraction and a closure win is observable. `scala-basic` is NOT extended (m19 6-record lock, m20/m21 locks stay); its m22 expectation flips from `PlanRejection` to whatever the wired planner produces (assert + document at Gate-G3).
- **D-M23-10 Guard hardening (M18.11 gap (b)).** Move every `ariadne_lang_cpp::*` reference out of core `src/` (`transform/engine.rs:405,437,460,485`, `placement.rs:178-181`, `validate/lib.rs:356,459`, `report/manager/tests.rs:371`) into sibling `crates/*/tests/` files (outside guard scope) or neutral fakes; widen `FORBIDDEN` in `m17_plugin_seam_test.rs:312` to also reject `Cpp`, `lang_cpp`, `ariadne-lang-cpp`; add `crates/pipeline` to the walk scope; `c++20` standard literals (build fixtures, `index/src/cache.rs:194`) stay — they are language-standard data; record this carve-out in the guard's doc comment.
- **D-M23-11 Out of scope (explicit deferrals).** `ScalaToolchainAnalyzer` exact-types analyzer → post-merge milestone with M23.5 (needs a real toolchain; nothing to exercise offline). Package-scoped cycle precision / SCC condensation → M23.5. Marker-based shaper auto-selection → unchanged deferral. UI redesign, third language → unchanged non-goals.
- **D-M23-12 (amendment 2026-10-02, pre-T2) Real compiler invocation deferred out of M23 — for BOTH languages.** The plan's division-of-labor row "C++ tier 3 = real compile" and `validate_affected_tus`'s doc comment ("real compile wiring lands in M23", `validate/src/lib.rs:306-307`) are amended to a follow-up milestone. Evidence: Gate-G1 F3 finding — extracted headers are not self-contained (they reference types like `ThingConfig` outside the extraction's own `required_deps`), so a real `clang++ -fsyntax-only` pass over generated units deterministically FAILS on the current fixtures; wiring the invocation would flip green offline suites (A3) to guaranteed-red wherever a compiler exists, with the only honest fix being include-closure computation at extraction time — a workstream of its own. What M23 actually delivers for the tier-3 row: probe-backed honesty (`Missing ⇒ Skipped`, unchanged neutral logic at `validate/src/lib.rs:339-344`) plus the Scala probe (`ScalaToolchainValidator`, D-M23-6) so the offline path is *reported* honestly for both languages. `TierValidator` never invokes `validate_compile` (verified `validate/src/lib.rs:315-349`: existence walk + status mapping only; `Pass` when status is not `Missing`); the doc comment gets amended to cite this decision, not deleted. The follow-up milestone owns: extraction include-closure + real compile invocation + the `Skipped`-disappears golden shift.

## 4. Contracts (trait deltas, exact)

```rust
// crates/lang/src/extractor.rs — additive defaults only; C++ byte-locks unaffected
pub trait SourceExtractor {
    fn extract(&self, symbols: &[SymbolBody], namespace: &str, required_deps: &[String]) -> GeneratedSourceFile; // unchanged
    /// Namespace the engine should pass back into `extract` for units cut from `source_text`.
    fn unit_namespace(&self, _source_text: &str) -> String { String::new() }
    /// Verbatim bodies for the named symbols located in `source_text`; unresolved names omitted.
    fn locate_symbol_bodies(&self, _source_text: &str, _wanted: &[(String, SymbolKind)]) -> Vec<SymbolBody> { Vec::new() }
}

// crates/lang/src/rewriter.rs — additive
fn find_dependency_targets(&self, _source_text: &str, _target: &str) -> Vec<DirectiveHit>; // default = today's C++ substring scan as neutral default? NO — default empty; C++ implements its scan explicitly
fn remove_top_level_symbols(&self, _source_text: &str, _names: &[String]) -> Option<String> { None } // Scala-only move support

// crates/lang/src/constraints.rs — additive
fn colocated_groups_with_edges(&self, records: &[SymbolRecord], edges: &[(SymbolId, SymbolId)]) -> Vec<Vec<SymbolId>> { self.colocated_groups(records) }

// crates/cluster/src/plan.rs
pub struct ProposedExtraction { …, #[serde(default)] pub source_file: Option<FileId> }

// crates/transform/src/engine.rs
IsolationEngine::extract(&self, plan: &OptimizationPlan, sources: &dyn Fn(FileId) -> Option<String>) -> ExtractionResult
// body precedence: located verbatim (via extractor.locate_symbol_bodies over dedup'd source files) → placeholder fallback (I4)

// crates/validate/src/lib.rs
TransactionApplier::apply_with(&self, …, rewriter: &dyn DependencyRewriter, extractor: &dyn SourceExtractor) -> ApplyOutcome // 3-arg apply + TransitionalDefaultRewriter DELETED
```

`DirectiveHit { line_index_range: (usize, usize), raw_directive: String }` lives in `crates/lang/src/rewriter.rs` (or `lang-api` if engine needs it — prefer `lang`).

## 5. Task plan (TDD; single lane, commit per task)

Gate G1 after T1, G2 after T2+T3, G3 (final) after T8. Reviewer = Code-reviewer subagent per gate; workspace gate = `cargo check --workspace --all-targets` + `cargo test --workspace` + guard suites.

### T1 — C++ verbatim bodies + golden refresh (Gate G1)
- **Red:** `lang-cpp` unit tests on `locate_symbol_bodies` (class-with-braces verbatim incl. trailing `;`, inline function, `namespace`-qualified name lookup, template decl, commented-out occurrence must not match, unresolved name → absent from result); engine test: source content with a real class → body equals it; no content → placeholder (I4); tokenizer offset capture does not change token stream semantics (existing tokenizer characterization green).
- **Green:** D-M23-1/2/3; `source_file` wired in planner (`propose_batch`/`assemble_plan`); engine content provider at CLI call sites (`transform.rs`, `optimize.rs`); placeholder fallback.
- **Golden refresh:** `zz_capture_isolation` recapture → replace `BASIC_PLAN_LOCKS`/`EXPENSIVE_PLAN_LOCKS` gen-bytes; `engine.rs:276` keeps placeholder expectation via no-content context; m00 goldens refresh ONLY where additive `source_file` appears — reviewer signs the diff line-by-line; `preview_json`/`diff_text` locks must be byte-identical (I1).
- **Refactor:** body-location helpers split from extractor; tokenizer offset edit minimal; no function > 100 lines.

### T2 — Single emitter + shim removal (Gate G2)
- **Red:** apply writes a generated file through a fake extractor whose bytes differ from preamble+concat (proves applier delegates); `production_write_path_reproduces_the_same_bytes_*` still green with the new signature; compile-error-driven migration of m10/m14/validate call sites.
- **Green:** D-M23-4; parent-dir creation in `write_to_disk`; delete shim/`TransitionalDefaultRewriter`; `#[allow(deprecated)]` annotations gone.
- **Refactor:** shared `render_unit(extractor, unit, namespace)` helper used by engine and applier (kills the duplication that created this bug class).

### T3 — Guard hardening (Gate G2)
- **Red:** widened `m17` guard first (expect red listing the known offenders), then fix until green; new walk scope includes `crates/pipeline`.
- **Green:** D-M23-10 relocation to `crates/transform/tests/` etc. with neutral-fake or real-plugin dev-deps; carve-out comment for `c++20`.
- **Refactor:** guard helper table-driven (token × crate), each fn < 30 lines.

### T4 — Scala toolchain probe validator
- **Red:** `toolchain_status()` on PATH-stripped probe ⇒ `Missing`; `validate_compile` never runs without toolchain; tier-3 `Skipped` flow asserted through `TierValidator` with the injected validator.
- **Green:** `ScalaToolchainValidator` replaces `StubValidator`; capability field flips land with T7 (validator injectable ahead of the flip).
- **Refactor:** probe helper unit-tested with injectable command runner.

### T5 — ScalaSourceExtractor (bodies + package + minimal imports)
- **Red:** given `legacy.scala` source + wanted symbol, `extract` output = `package <ns>` + referenced imports only (unreferenced import dropped, byte-verbatim referenced lines kept) + verbatim top-level region (braceless `def`/`val` region rule reuses `span_boundaries`); `unit_namespace` on `package a.b`, nested `package a.b.c`, and block-package file (`""`, matching analysis-time); interpolation/comment/string contents must not corrupt the slice (corpus-fixture guard tests).
- **Green:** scanner line/column→offset table; `ResolvedDecl` spans surfaced through a text-locating entry point; `locate_symbol_bodies` + `unit_namespace` impls.
- **Refactor:** extractor module ≤ 300 lines, three helpers (package, imports filter, region slice).

### T6 — ScalaDependencyRewriter (import rewrite, move-removal, diff text)
- **Red:** rewrite of `import a.b.{Old}` → new dep keeps every other byte (test pins whole-file equality); wildcard-aggregate narrowing per D-M23-5; `remove_top_level_symbols` deletes exactly the region incl. its doc comment and leaves neighbours byte-identical; `render_diff` emits `- import a.b.C` / `+ import a.b.C` / `~ … // side-effect preserved` in the C++ diff grammar's shape; `find_dependency_targets` finds named/grouped/wildcard hits, ignores `//`-commented and string-literal occurrences.
- **Green:** built on `directive_parser` + T5 spans; `remove_top_level_symbols` real impl.
- **Refactor:** no duplication with T5 region logic (share the span slicer).

### T7 — Wiring: capabilities, routing, planner dispatch, sealed constraints (D-M23-6/7/8)
- **Red:** m19 registry-backend lock rewritten for language-routed resolution; `plan` on scala-basic now yields a plan/rejection-free result with Scala-shaped targets (`scala_target_path_for` via CLI, not test injection) while a C++ fixture's plan JSON stays byte-identical; sealed-pairing planner test: sealed `Shape` + 2 subtypes in `legacy.scala` land in one batch (constraint merge), unsealed sibling stays separable; consumer discovery for a Scala target found via import scan (not `#include` scan).
- **Green:** capability flip; per-language planner/backend dispatch; `find_dependency_targets` call sites; `colocated_groups_with_edges` + planner edge feed + `modifiers` in the Scala parser (`skip_serializing_if` ⇒ C++ m00 index goldens unchanged — assert).
- **Refactor:** dispatch tables not match-fests; each `plan.rs`/`registry.rs` addition < 40 lines per fn.

### T8 — Acceptance suite + closeout (Gate G3)
- **Red→Green:** `tests/integration/tests/m23_symbol_isolation_test.rs` — style of m22 (`copy_tree_excluding_build_output`): on `scala-isolation` fixture end-to-end discover→build→graph→index→reduce→load→plan→preview→apply: extracted file exists at the package dir with verbatim symbol text + package clause; source file no longer contains the moved symbol (I5); a consumer's closure loses the non-extracted sibling (the load win); untouched files byte-identical; rollback restores everything (dir-hash identity); tier outcomes `[Pass, Pass, Skipped]`. C++ side: `m20_apply_migrate_characterization` + m18 parity green (bodies refreshed at T1).
- **Expectation refresh:** m22 plan-rejection → new asserted behaviour; final `cargo test --workspace` count recorded; kickoff closeout notes (decisions, deviations, ledger) appended here; `git` commit per task throughout; plan files committed alongside code (`repo.tracks_plan_files`).
- **Reviewer:** Gate-G3 full-milestone audit vs this kickoff + plan §3.6; deviations from plan text (D-M23-1) must be explicitly closed.

## 6. Golden / lock inventory at risk

| Lock | Location | Disposition |
|---|---|---|
| placeholder byte | `engine.rs:276` | survives via fallback path |
| gen-file vendored bytes | `isolation_output_parity.rs` BASIC/EXPENSIVE | refreshed at T1 via `zz_capture_isolation`, reviewer diff |
| preview JSON + diff text | same file :72-73,:93 | must stay byte-identical (I1) |
| m00 goldens | `tests/golden/**` | refresh only additive `source_file` (T1) + zero index churn (T7 asserts) |
| D5 serde names (`GeneratedHeaders`, `ConsumerRewrite` keys, `Committed/RolledBack`) | validate + transform | untouched, `status_serde_names_stable_and_skipped_is_additive` must stay green |
| m22 plan-rejection | `m22_scala_load_analysis_test.rs` | evolved at T7/T8 (D-M23-9) |
| m19 capability literals | `cli/src/registry.rs:210-229` tests | rewritten for language routing at T7 |
| m19 scala-basic 6-record discovery lock | `m19_scala_discovery_test.rs:93-120` | untouched (new fixture per D-M23-9) |
| `symbols.bin` epoch 2.0.0 | `index/src/cache.rs:9` | untouched (I6) |

## 7. Risks

- C++ declaration-text location is the riskiest locator (templates, macros, `namespace` blocks) — fallback to placeholder (I4) bounds damage; verbatim-locatable set locked by unit table, not by fixture-only luck.
- Move-semantics removal must never corrupt neighbours — whole-file byte-equality tests except for removed region (T6) are the contract.
- Capability flip widens ambiguity blast-radius (registry, transform CLI, optimize path) — T7 lands all routing in one commit set, Gate-G3 audits the full call graph.
- `m10`/`m14` are rollback-integrity characterizations: their shim migration must not weaken the dir-hash identity assertions (only the call signature moves).
