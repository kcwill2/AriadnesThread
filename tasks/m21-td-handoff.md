# M21 T-D Handoff — `m21_scala_semantic_index_test.rs` + final gate

Depends on: T-A/T-B/T-C merged. Decision refs: D16/D17 (expected shapes), D19 (fingerprint witnesses), D21 (hit-rate observability). Kickoff: `plans/m21-scala-index-kickoff.md`; acceptance text: plan:413 verbatim.

## Touch-set

- NEW `tests/integration/tests/m21_scala_semantic_index_test.rs` **only**. No fixture edits (m19's 6-record lock), no golden re-pin (m00 zero drift is a gate), no src edits — if acceptance cannot be met from test-visible artifacts, STOP and report the gap; a minimal neutral counter (D21) requires my ruling first.
- `tests/integration/Cargo.toml` dev-deps already cover everything (ariadne-{cli,pipeline,index,lang,lang-scala} all present) — verify, do not add blind.

## Bootstrap (copy the house pattern)

`fixture_path` + `isolated_scala_repo(label)` helpers exactly as in m19 (`m19_scala_discovery_test.rs:38-52`) / m20 — every test gets its own tempdir copy; all edits happen in the copy. Registry: `ariadne_cli::registry::default_registry()`. Index entry: `ariadne_cli::commands::index::build_source_index_filtered(&copy, Some(SourceLanguage::Scala))` — this is the `--lang scala` CLI path through T-C's seam; that IS the milestone's product path.

## Required tests (all deterministic, no wall-clock/mtime comparisons)

1. **plan:413 literal** — index contains `SymbolId("com.example.services.OrderService")`, kind `Class`, with `Complete` semantic edges to `com.example.models.Order` and `com.example.models.User` (assert on `IndexResult.tu_analyses[*].edges` — src/dst/kind; `merge`-independent). Assert `Main`/`models.User`/`models.Order`/`models.all`(package-object ⇒ id `com.example.models`) symbols exist with D17 kinds; assert total `tu_analyses.len() == 5` (main-root units only; test root stays excluded per M19 shaper).
2. **Cold / warm (D21)** — first run writes `artifacts/shards/*.json` for the 5 units. Re-run identical → all five shard envelopes **byte-identical** AND (stronger, if observable) zero analyzer invocations: instrument via a counting `SemanticAnalyzer` injected through a bespoke registry (`LanguageRegistry::new().register(ScalaPlugin::new())` wrapped analyzer if feasible without src change — else shard-byte identity is the accepted witness; state which you used).
3. **Edit reanalysis** — append a comment line to the copy's `models/Order.scala` only, re-run: shard envelopes for `Order.scala` and `services/OrderService.scala` have new fingerprints; `Main.scala`, `models/User.scala`, `models/all.scala` envelopes unchanged. (OrderService's closure contains Order via the M20 import edge — this is "only import-dependents".)
4. **Language-scoped cache (plan:413 last clause)** — via `ariadne_index::cache::TuCacheFingerprint::compute`: same inputs with `(Cpp, "v1")` vs `(Scala, "scala-parser-1")` ⇒ different `combined_hash`; the `(Cpp, "v1")` digest is unaffected by what Scala's fingerprint is (recompute with a hypothetical `"scala-other"` to prove non-coupling); epoch witness `CURRENT_ANALYSIS_VERSION == "2.0.0"`.
5. **Offline determinism (A3)** — parse `OrderService.scala` source directly twice via `ScalaSourceParser` through `ariadne_cli::registry::default_registry()`'s plugin ⇒ byte-identical `serde_json::to_string` of both `TuAnalysis` values. And assert no test shells out: none may spawn `scalac`/`sbt` (true by construction; call it out in a comment).

## Final gate (this task owns it)

`cargo test --workspace` (600 s+ timeout; expect 767/0/3 baseline + all new M21 tests, **zero pre-existing suite edited** — `git diff --stat` must show only your new test file since the T-C commit) + `python tools/hygiene_check.py` exit 0 + m00 **zero golden drift** + m17 guards green.

Commit: `test(m21): Scala semantic-index acceptance suite - symbols, complete edges, shard-scoped incremental behavior` — body lists which D21 observability you used and any expectations you had to adjust, each justified against D16/D17.

Report: SHA, per-test pass table, total suite counts, golden-diff proof, deviations.
