# M22 Kickoff — Scala analysis outputs: aggregation, load metric, reports, Graph UI

Spec: `plans/multi-language-support-implementation_plan.md` §M22 (lines 417–428) + checklist M22.1–M22.3 (576–578).
Baseline: `f09869f` (M21 closed, 842/0/3, four reviewer gates APPROVED).
Carried-forward into M22 (from M20/M21 execution notes): marker-based shaper auto-selection (the "M18.5 gap"), Scala `ClusteringConstraints` (`stubs.rs:11` "owned by M22"), heuristic labelling of Scala output (plan risk row line 509, "for M21–M22"), UI declared-vs-defined (§3.6.1, "M22").

## Scope — hard boundary

**M22 = Scala analysis outputs.** Per plan:421–428 + the deferred list:

1. Scala planning surface (package-object aggregation filter, target naming, companion constraints) — `lang-scala` only.
2. Capability-gated `PlanRejection` (planning=false never returns silently-empty).
3. Marker-based shaper auto-selection so unfiltered pipeline runs on an sbt repo shape Scala (D15's "unfiltered Scala shaping is M22").
4. Additive `languages` in report manifests (skip-when-empty ⇒ C++ goldens byte-identical).
5. `LanguageModeling`-sourced load/explain wording + "heuristic" label in CLI output for Scala.
6. UI: `language` + `unit_kind` on file nodes; path-resolved `defining_file` + `declaring_files` on symbol nodes; inspector declared-vs-defined rendering.
7. `m22_scala_load_analysis_test.rs` + golden re-check + workspace green.

Explicitly OUT: M23 (any transform/rewrite/isolation execution, `ScalaSourceExtractor`, real bodies, cross-file same-package resolution), M23.5 (cycle substrate — lives on main), mixed-language `incremental_backend` error, LoadMetric struct fields from §3.5 (ruling D25: manifest-level only), `is_source_or_header` retirement, pipeline default_registry duplication.

## Decisions (orchestrator rulings)

- **D22 — Marker-based shaper auto-selection (neutral, C++ byte-identical).** New `LanguageRegistry::unit_shaper_for_repo(&Path) -> Option<&dyn UnitShaper>`: first plugin **in registration order** whose `shaper().build_markers()` has any member present at the repo root; fallback = today's first-wins `unit_shaper_for()`. `unit_shaper_for()` itself is **unmoved** (m19 registration-order lock + d9 stay green). Switch the call sites that hold a repo root: `graph/builder.rs::build_dependency_graph` (D14 twin already injects), pipeline `compute_inputs` + `select_unit_shaper` `None` arm (D15 filtered arm unmoved), CLI `discover`/`measure`/`transform`/`analyze` incremental build-discovery sites. Safety proof: selection differs from today **only** where a non-first plugin's markers are present and the first plugin's are absent — impossible for every existing C++ fixture (no `build.sbt`), so m00/fps/index bytes are unchanged. Zero language nouns (registry lookup only, R2/R3).
- **D23 — Capability-gated planning rejection.** `RejectionReason` gains an additive variant naming the language *by data* (`LanguageId` from lang-api — no literal in cluster). Planner gains an optional injected planning-gaps input (setter, not an 9th ctor arg) and a new `plan_with_rejections() -> (Vec<OptimizationPlan>, Vec<PlanRejection>)`; `plan()` delegates unchanged (existing callers byte-identical). One explanatory rejection per repository-present language whose `capabilities().planning == false`, **independent of candidate pass**. CLI `plan.rs` computes gaps via discovery `FileRecord.language` ∘ `registry.plugin_for_id(..).capabilities()` (R3-legal) and prints them (`exit 0` — output is never silently empty). `PlanManifest` untouched unless the impl proves an additive skip-when-empty field is free; goldens byte-identical.
- **D24 — Scala planning surface.** `lang-scala/src/planning.rs`: aggregation filter = "file declares a top-level `package object`" (scanner-backed detection through the M21 parser family — comments/strings cannot fire it; NOT filename `all.scala`); `scala_target_path_for` = `{sanitized_symbol}.scala` beside the aggregation file (same sanitize discipline as lang-cpp). `ScalaClusteringConstraints` replaces `StubConstraints` in the plugin: companion object↔class pairing via the D17 `$`-id convention within the given symbol batch; sealed-trait↔subtype pairing only what the per-batch `&[SymbolRecord]` input can see, cross-file limitation documented + test-locked as such. `planning=false` means production never runs these — wiring proven by unit tests + the planner-injection lane.
- **D25 — Additive manifest `languages` fields.** `RepositoryManifest`, `BuildContextManifest`, `IncludeManifest`, `LoadManifest` gain `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub languages: Vec<SourceLanguage>` holding only languages **≠ `SourceLanguage::default()`** (default-comparison, not a language noun; D11 skip-when-zero precedent). C++ repos ⇒ empty ⇒ skipped ⇒ **goldens byte-identical**; Scala ⇒ `["Scala"]`. §3.5's LoadMetric-struct fields deferred.
- **D26 — Vocabulary + caveat in `LanguageModeling`.** Additive `pub dependency_vocabulary: DependencyVocabulary` (lang-api: `directive_label` "Include"/"Import", `directive_passive_verb` "included"/"imported", `aggregation_noun` "header"/"package aggregation") and `pub analysis_caveat: Option<String>` (C++ `None`, Scala `Some("heuristic …")`). `LoadMetric::format_summary()` and `LoadExplanation::format_explanation()` keep their current bytes and gain `*_with(vocabulary)` variants; CLI analyzes picks the vocabulary via registry for the analyzed language(s) — C++ call path emits today's exact strings (m6 locks them). Heuristic caveat printed by CLI (stdout) when any analyzed language carries one; **not** in manifests (kept out per D25). Plugin modeling-lock tests updated honestly in the same commit.
- **D27 — UI surface + fixture evolution.** File nodes: additive `language` (from `FileRecord.language`) + `unit_kind` (from plugin modeling via CLI payload builder — cli is language-legal). Symbol nodes: `defining_file` becomes the **resolved repo-relative path** (today a numeric FileId string — that is a defect, §3.6.1 intent), plus additive `declaring_files` (paths, deduped). `path` on symbol nodes = defining path when resolvable (enables the source drawer). graph.js inspector renders "Defined in …" / "Declared in …" rows; file inspector shows the language. No UI goldens exist (verify; if a snapshot test exists, update + record).
  **Fixture evolution (T-7 only):** content-only edit of `OrderService.scala` — grouped import → `import com.example.models._` — so the graph closure reaches `all.scala` (package-object file, owns the symbol that becomes the collateral). m19's 6-record lock survives (no file changes). m20/m21 expectation updates allowed **only** where empirically forced, recorded in the commit body.

## Gates

- Gate-A (after T-A/T-B/T-C), Gate-B (after T-D/T-E/T-F), Gate-C (after T-G + full workspace). Each: reviewer re-runs the suites independently, byte-scans the diff (no PowerShell mojibake, no golden edits, R1/R2 token scan), hygiene exit 0, verdict APPROVED/REJECTED.

## Tasks

- **T-A (M22.1a)** Scala planning surface — `crates/lang-scala/**` only.
- **T-B (M22.1b)** Capability-gated `PlanRejection` — `crates/cluster/src/{plan.rs,planner.rs}` + `crates/cli/src/commands/plan.rs`.
- **T-C (M22.1c)** Marker-based shaper auto-selection (D22) — `crates/lang/src/registry.rs`, `crates/graph/src/builder.rs`, `crates/pipeline/src/lib.rs`, CLI build-discovery call sites.
- **T-D (M22.2a)** Manifest `languages` fields (D25) — `crates/report/**` + write sites.
- **T-E (M22.2b)** Vocabulary + caveat (D26) — `crates/lang-api/src/modeling.rs`, `crates/graph/src/{load.rs,load_explain.rs}`, `crates/lang-cpp`/`lang-scala` modeling, CLI analyze/includes output.
- **T-F (M22.2c)** UI surface (D27) — `crates/ui/src/{dto.rs, server.rs?}`, `crates/cli/src/commands/ui.rs`, `crates/ui/assets/**` (graph.js), ui tests.
- **T-G (M22.3)** `m22_scala_load_analysis_test.rs` + fixture evolution + golden re-check + `cargo test --workspace` final gate.

## Acceptance (plan:428)

1. `ariadne load --symbol com.example.services.OrderService` on scala-basic (auto-selected shaper, no `--lang`): collateral load **non-zero**, largest collateral source `all.scala`.
2. `ariadne explain` renders the import chain with import-flavored wording.
3. `ariadne plan` prints the capability-gated Scala rejection (exit 0, no fake header invented — plans empty).
4. C++ report/UI golden output unchanged (m00 byte-identical; m6/m10/m12/m14/m18/m20_apply unedited and green).
5. `m22_scala_load_analysis_test.rs` green; `cargo test --workspace` green; `python tools/hygiene_check.py` exit 0.

## Execution notes

(to be appended at close)
