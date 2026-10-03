# M21 T-B Handoff — `ScalaAnalyzer` + plugin wiring + fingerprint bump

Depends on: T-A merged at `21b4c4d`, Gate-A **APPROVED** zero-blocking. Decision refs: D18 (analyzer is a thin wrapper), D19 (fingerprint bump), D20. Kickoff: `plans/m21-scala-index-kickoff.md`.

## Gate-A addendum (pre-authorized test-only extension of the touch-set)

Fold these reviewer non-blocking notes into your commit (test additions only — no parser semantics changes): (1) parser-level nested block comment negative `/* /* class Ghost */ */` ⇒ zero symbols/edges/refs; (2) anonymous `given Ordering[Int] with {}` ⇒ zero symbols assertion (currently doc-only); (3) strengthen `decl_text_inside_string_literals_is_invisible` by putting a *callable* leak (`X(` of a named-import `X`) inside `"""…"""` and assert edges/call-sites empty too; (4) parser-level `export` ⇒ Complete edge test; (5) scanner char-literal doc/behavior mismatch (reviewer note 5: doc claims run-to-EOF on unterminated `'\n`, `scan_char_literal` returns false and leaks tail) — align the doc to the behavior; (6) `x.package` dot-guard one-liner in `directives.rs` (reviewer note 6) IS a semantics change — implement it with a test (it was judged blocking-exotic; one guard + negative).

## Touch-set (nothing else)

- NEW `crates/lang-scala/src/analyzer.rs` — `ScalaAnalyzer`.
- EDIT `crates/lang-scala/src/plugin.rs` — swap `StubAnalyzer` → `ScalaAnalyzer`; bump `analysis_fingerprint()` `"scala-1"` → `"scala-parser-1"` (D19); update the `plugin_identity_and_fingerprint` lock in the same commit with a `// bumped with the M21 parser` comment.
- EDIT `crates/lang-scala/src/stubs.rs` — **remove `StubAnalyzer` only**; `StubConstraints`/`StubExtractor`/`StubRewriter`/`StubValidator` stay (M22/M23 own them). Update the module doc (`stubs.rs:7-10`) to record that the analyzer gap is closed as of M21.
- EDIT `crates/lang-scala/src/lib.rs` — `pub mod analyzer; pub use analyzer::ScalaAnalyzer;`.
- Plugin test fixups where the fixture smoke test (added at M20 `75abb8e`) asserts empty analysis — expectation updates are test-only, list each in the commit body.

Frozen: `parser.rs`/`scanner.rs` (fix in place ONLY if a bug blocks wiring — report it), `directive_parser.rs`, `Cargo.toml`, anything outside `crates/lang-scala`.

## Contract

```rust
pub struct ScalaAnalyzer { parser: ScalaSourceParser } // or unit struct delegating
impl ariadne_lang::SemanticAnalyzer for ScalaAnalyzer {
    fn analyze(&self, file_id: FileId, source: &str, context: &str) -> TuAnalysis;
}
```

- Delegates to `ScalaSourceParser::parse(file_id, rel_path-as-sentinel, source)`; then stamps `language = SourceLanguage::Scala` and `context_name = context` (`tu_store.rs:205-207` takes language from index-0 — the real stamp must be Scala). Do NOT re-stamp `tu_file_id` (parser owns it).
- Stateless / `Send + Sync` (trait bound). No allocation churn tricks; keep it three-ish lines of real logic.
- `ScalaPlugin::analyzer()` returns `&ScalaAnalyzer`; the `LanguagePlugin` trait signature is unchanged (no `crates/lang` edit).
- Fingerprint: `fn analysis_fingerprint(&self) -> String { "scala-parser-1".into() }` — C++ `"v1"` must not move anywhere (witness below).

## RGB plan

- **Red**: analyzer unit tests — (1) `analyze` output has `language == Scala`, `context_name == given`, `tu_file_id == given`, and non-empty symbols for a `class Foo` source (fails today: stub returns empty); (2) plugin fingerprint test asserting `"scala-parser-1"` + `id() == Scala` + capabilities unchanged (extend existing lock, don't weaken it); (3) stub-removal is compile-enforced, but add a plugin-level test that `ScalaPlugin::new().analyzer().analyze(...)` on the `OrderService.scala` fixture text yields the `com.example.services.OrderService` symbol with `Complete` edges (end-to-end crate-local witness that wiring, not just types, is live).
- **Green**: wrapper + swap + bump. **Refactor**: delete dead stub code/imports; docs updated.

## Commands & gates

- `cargo test -p ariadne-lang-scala` green.
- Witness (scoped, read-only): `cargo test -p ariadne-index -p ariadne-scheduler` green (dispatch layer untouched; `test_differing_language_or_plugin_fingerprint_changes_digest` proves the bump can't touch C++ digests because `"v1"` is a separate input). If `pipeline` or integration suites compile-fail on `StubAnalyzer` references (grep first — expected: none outside lang-scala), report before editing them.
- `python tools/hygiene_check.py` exit 0. No `--workspace`.
- Commit: `feat(m21.2): ScalaAnalyzer wired into ScalaPlugin; analysis_fingerprint scala-parser-1; StubAnalyzer retired`.
- Report: SHA, any test-expectation updates (each justified), confirmation C++ `"v1"` literal untouched (`git diff` excerpt).
