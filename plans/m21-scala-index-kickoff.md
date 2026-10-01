# M21 Kickoff — Scala Semantic Index (offline, deterministic)

Status: **IN PROGRESS** · Prepared 2026-10-01 · Base: branch `multi-language-support` @ `77bd0a7` (M20 complete + all reviewer gates approved; workspace 767/0/3, hygiene exit 0, goldens zero drift)

Authoritative spec: `plans/multi-language-support-implementation_plan.md` — §5 "M21" table (lines 403–413) is the per-file change list and acceptance; §6.2 row `lang-scala::parser` the unit-test contract; §6.3 the suite name; §7 R2/R4 the review rules (R4: parser expected to split into submodules). This document adds only milestone state, decisions, and the orchestration protocol.

## Tree reality vs plan table (recon at `77bd0a7`)

Rows of the plan:405–410 table **already satisfied** — do not redo, only witness:

- Registry wiring: `pipeline::build_default_registry` registers Cpp then Scala (both `pipeline/src/lib.rs:49-53` and `cli/registry.rs` share it; `d9_registries_identical` locks identity).
- Per-unit plugin dispatch is complete: `PluginWorkerBackend::analyze_task` (`scheduler/worker.rs:79-102`) and `pipeline::index_unit_with_plugin` (lib.rs:743-765) pick the analyzer via `detect_language` → `plugin_for_id`; there is no C++ default left. A Scala unit today flows the whole index path and yields **empty-but-Scala-stamped** analyses via `stubs::StubAnalyzer` (the documented interim contract, `stubs.rs:7-10`).
- "Combined analysis = own file + files reached by resolved imports" is exactly `scheduler::tu_store::compute_tu_closure_plugin` + `analyze_closure_with` (verified: Scala externals never enter; `analyze_closure_with` stamps `language` from index 0).
- Cache rows: `TuCacheFingerprint` already hashes `language` + `plugin_fingerprint` and `CURRENT_ANALYSIS_VERSION = "2.0.0"` (M18.10 claim verified against `index/src/cache.rs:15,17-33,59-82`; digest-scoping test exists). **The only M21 cache action is the plugin fingerprint bump (D19).** `scheduler/{worker.rs,incremental.rs}` are untouched by M21.
- `handle_index_with_registry_filtered` + CLI `--lang` already exist (plan's "handle_index_with_registry" row is satisfied; no `handle_index` anymore).

Genuinely missing → the four tasks: (T-A) `ScalaSourceParser`; (T-B) `ScalaAnalyzer` + plugin wiring + fingerprint bump; (T-C) index-stage unit shaping under an explicit language filter (the D14 gap: `registry.unit_shaper_for()` is C++-first-wins → `ariadne index --lang scala` on an sbt repo yields **0 units** today, so T-A/T-B alone change nothing end-to-end); (T-D) `m21_scala_semantic_index_test.rs` + final gate.

## Scope — hard boundary

**M21 = offline heuristic semantic index for Scala + the shaping seam + acceptance.** Explicitly OUT:

- `ScalaToolchainAnalyzer` / any `scalac`/`scala-cli`/`sbt` probing → M23 (A3: tests must run with no Scala toolchain on PATH).
- `ScalaSourceExtractor`/`ScalaDependencyRewriter`/`ScalaValidator`, `StubConstraints` replacement (companion/sealed grouping) → M22 (constraints wiring) / M23 (isolation).
- Mixed-language repos: `cli/registry.rs::incremental_backend`'s error + its lock test (`registry.rs:284-297`, "multi-language incremental analysis lands with the language-scoped cache (M21)") stay **unedited**; single-language Scala already passes it. Re-timing the message is M22/M23 territory.
- Marker-based shaper auto-selection for **unfiltered** runs — stays deferred to M22 exactly as the M20 execution notes carry it. M21 only adds filter-driven selection (D15).
- `lang-scala/src/directive_parser.rs` is **frozen** (m20 graph locks lean on it); the new parser owns its own scanner (D18).
- No edits to `index`, `graph`, `scheduler`, `repo`, `lang-cpp`, `lang-api`, `lang`; no golden re-pin; `tests/fixtures/scala-basic/**` frozen (m19's exact 6-record lock).

Touch-set: `crates/lang-scala/**` (T-A/T-B), `crates/pipeline/src/lib.rs` + pipeline tests (T-C, shaper-selection edit only), `tests/integration/tests/m21_scala_semantic_index_test.rs` (T-D). Any other core edit needs justification against §3.1 in the commit body.

## Decisions (orchestrator rulings)

- **D15 — Filter-driven unit shaping (R2/R3-clean, C++ byte-identical).** `Pipeline::compute_inputs` today calls `registry.unit_shaper_for()` regardless of `language_filter`. Change: when `language_filter == Some(lang)`, the shaper comes from `registry.plugin_for_id(lang)?.shaper()` (registry lookup — no `match LanguageId`, no language nouns in pipeline); when `None`, the existing first-wins call is **unmoved**. `--lang scala` therefore chooses "the language explicitly" per plan §6.3; unfiltered Scala auto-detection remains M22.
- **D16 — Reference→edge policy.** The plan:413 acceptance names *edges* OrderService→Order/User, which the parser-table alone (references-only) would not produce. Contract: the parser emits `SemanticEdge` for (1) every `extends`/`with` parent (kind `Complete` iff the target maps to a named import, the same-file namespace set, or a same-package sibling declared in this file; else `Incomplete`) and (2) every type-position / `new X` / named-call reference to a **resolvable** name (named import selector or same-package sibling) with kind `Complete`; a usage fed only by a single wildcard import gets an `Incomplete` edge with best-effort qualified dst (e.g. `ArrayBuffer` → `scala.collection.mutable.ArrayBuffer`) plus a `SymbolReference`; unresolvable names get references only, never edges. `dst_file`/`src_file` on parser edges use the `FileId(0)` sentinel unless mirroring the `lang-cpp` analyzer's convention (T-A must inspect `lang-cpp/src/analyzer.rs` and match; `merge_tu_edges` reads only src/dst symbol + kind, so the sentinel is downstream-safe). Dedup edges by `(src,dst,kind)`; keep every reference occurrence (load metric's `tu_usage` consumes them).
- **D17 — Declaration contract.** Top-level (brace-depth-0) decls only: `class`/`case class`→`Class`; `trait`→`Trait`; `object`/`case object`/`package object`→`Object`; `enum`→`Enum`; `given` (paramlist→`Function`, else `Variable`); top-level `def`→`Function`, `val`→`Variable`. `SymbolRecord.name` = simple identifier; `id` = `<package>.<Name>`, **companion objects take the JVM module-class convention id `<qual>.<Name>$`** so class/object never collide; every record carries exactly one `SymbolDecl` (`is_definition = true`, single-file placement) and `definition = Some(that DeclId)`; `DeclId = <symbol-id>#L<line>`. `package object models` under `package com.example` → id `com.example.models`, kind `Object` (M22's aggregation hook). Reference tie-break: type position (`: T`, `new X`, `extends/with`) → class id; call position `X(` → object id when an object with that simple name exists in scope, else class id.
- **D18 — Scanner fidelity is the parser's own.** New `scanner.rs` in lang-scala: skips `//`, nested `/* */`, `'c'`, `"…"`, `"""…"""`, interpolated `s/f"…"` (splices `${…}` are **not** analyzed — documented limitation + negative test); backtick-escaped identifiers lexed as identifiers (content verbatim). `directive_parser.rs` untouched; known M20 carry-forward scanner quirks (quote-run leak, `given` selectors) belong to this scanner only if they also apply, else they are M23 notes.
- **D19 — Fingerprint bump.** `ScalaPlugin::analysis_fingerprint()` "scala-1" → **"scala-parser-1"** (plan:408) in the same T-B commit that replaces `StubAnalyzer`; update `plugin_identity_and_fingerprint` lock honestly (comment `// bumped with the M21 parser`); C++ `"v1"` must not move (m18 parity). No cache.rs edit — digest machinery already hashes the plugin string.
- **D20 — Mixed-language analyze policy unchanged** (see scope); m21 tests are single-language.
- **D21 — Hit-rate observability.** plan:413 "second run reports 100 % hit-rate … editing reanalyzes only import-dependents" is proved on **content, not vibes**: cross-run comparison of `artifacts/shards/<file_id>.json` envelopes (fingerprint embedded). Expectations (deterministic for the fixture): cold = 5 main-root units analyzed; warm = all 5 reused (envelopes byte-identical); after a content-only edit to `models/Order.scala` (temp-dir copy only) exactly **{Order.scala, OrderService.scala}** get new fingerprints (OrderService's closure contains Order), `{Main.scala, User.scala, all.scala}` reused. If an existing pipeline/progress stat distinguishes analyzed-vs-reused, use it too; a minimal *neutral* counter is pre-authorized in T-D only if the shard witness cannot prove the reuse direction.

## Acceptance (machine-checkable)

1. `cargo test -p ariadne-lang-scala` green — parser unit tests per plan §6.2 (`lang-scala::parser` row: class, trait, object, case class, `given`, companion pairing, `extends`/`with` edges, nested package, comments + string interpolation produce nothing) + analyzer tests (stamps `language=Scala`, `context_name`; fingerprint lock).
2. `cargo test -p ariadne-integration-tests --test m21_scala_semantic_index_test` green per plan:413: index contains `com.example.services.OrderService` with `Complete` edges to `com.example.models.Order` and `com.example.models.User`; cold/warm/edit shard behavior per D21; language-scoped digest witnesses (Cpp-digest recomputes identically with `"v1"`; Scala digest changes iff its own plugin fingerprint does; epoch witness `"2.0.0"`).
3. C++ locks untouched and green: `m00` 2/2 **zero golden drift**, `m18` 22/22, `m3`, `m17` 5/5 (guards: pipeline edit names no language), `m19` 5/5, `m20_scala` 5/5, old-numbering `m20_apply`, `m10`/`m14` witnesses.
4. `cargo test --workspace` green (baseline 767 + new) + `python tools/hygiene_check.py` exit 0.
5. No Scala toolchain invoked (A3): `scalac`/`sbt` remain string literals.

## Orchestration protocol (non-negotiable — M18/M19/M20 lessons)

- Four sequential bounded handoffs, each committing immediately on green scoped commands: **T-A** `tasks/m21-ta-handoff.md` (parser core + scanner) → **T-B** `tasks/m21-tb-handoff.md` (analyzer + plugin + fingerprint) → **T-C** `tasks/m21-tc-handoff.md` (filter-driven shaping) → **T-D** `tasks/m21-td-handoff.md` (m21 suite + final gate). No parallel implementers in this worktree (shared cargo locks + commit serialization). M18 size-risk memory applies: if a handoff stalls, restart fresh from disk inventory — a compacted session with an "empty summary" is unrecoverable.
- Reviewer gate after each commit (APPROVED before the next handoff). Gate focuses: **A** = negative-case fidelity (comments/strings/splices) + no `directive_parser` edits + §6.2 coverage; **B** = fingerprint blast-radius (C++ `"v1"` unmoved, shard keys, lock honesty) + stub-removal completeness; **C** = `filter=None` path byte-identical behavior + pipeline names no language + m00 witness; **D** = plan:413 verbatim + D21 determinism + guard suites.
- Hygiene every commit: `python tools/hygiene_check.py` (fn < 100, file < 1000) + meaningful names; R4 split guidance for the parser is baked into T-A's layout.
- UTF-8 hazard (recorded correction): never script-edit tracked files via `Set-Content`/`Out-File`/`>`; Read/Edit tools only, verify with `git diff --numstat`. Fixture/parsed sources stay ASCII-safe anyway.
- Long cargo runs: command timeout 600000 ms; scoped commands only until T-D's final `--workspace`.

## Execution notes

(to be appended on completion)
