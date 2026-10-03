# M21 T-A Handoff — `ScalaSourceParser` (offline heuristic parser)

Depends on: base `77bd0a7` + kickoff docs commit. Decision refs: D16, D17, D18 (kickoff `plans/m21-scala-index-kickoff.md`). **Read the kickoff §"Scope" and D15–D21 before writing any code.**

## Touch-set (nothing else)

- NEW `crates/lang-scala/src/scanner.rs` — token cursor with Scala lexical fidelity.
- NEW `crates/lang-scala/src/parser.rs` (split further, e.g. `parser/decl.rs` + `parser/reference.rs` under a `parser/` dir, if either concern would push a file toward ~700 lines — R4).
- EDIT `crates/lang-scala/src/lib.rs` — module decls + re-export `ScalaSourceParser` only.
- Colocated `#[cfg(test)] mod tests` in the new files.

Frozen: `stubs.rs`, `plugin.rs`, `directive_parser.rs` (any diff here = stop, report), everything outside `crates/lang-scala`. No plugin wiring in this task — the analyzer still returns the stub; that is fine because nothing selects the parser yet.

## Contract

```rust
pub struct ScalaSourceParser; // stateless
impl ScalaSourceParser {
    /// Hand-written offline scanner; deterministic; never panics on invalid input.
    pub fn parse(&self, file_id: ariadne_repo::FileId, rel_path: &std::path::Path, source: &str)
        -> ariadne_lang::TuAnalysis;
}
```

Use `rel_path` where honest (e.g. it is NOT needed for output; keep the plan's signature, document non-use in a doc comment — do not invent usage). Output per plan:407:

1. **Package clause**: consecutive leading `package <qualifier>` lines set the namespace; `package object <name>` → one symbol id `<ns>.<name>` kind `Object` (D17). Brace form `package x {` and Scala-3 `package x:` are documented graceful limitations — must not crash and must not fabricate symbols; negative test each.
2. **Top-level declarations** (brace-depth 0) and kind mapping per D17, incl. `case class`/`case object`/`given`/top-level `def`/`val`. Skip type-parameter lists `<...>` and value parameter lists `(...)` preceding `{`/`=`. Each symbol: exactly one `SymbolDecl` (`is_definition=true`, `location` line/column of the name token), `definition=Some(decl.id)`, `owning_file=file_id`, `name`=simple identifier, `id`=qualified; companion object id suffix `$` per D17. `DeclId = <symbol-id>#L<line>`.
3. **Body spans**: brace-match each top-level body (tracking depth correctly *through* the scanner's comment/string rules) to (a) end the top-level region correctly, (b) know the enclosing symbol for references/call-sites, (c) keep nested decls out of symbol output. Span *text* is not stored (M23 concern) — do not add fields to shared types.
4. **Imports map** (for reference resolution, D16): parse `import a.b.C`, `import a.b.{C, D}`, `import a.b._` occurrences lexically (this is the *parser's* import scan, not `directive_parser`); selector `C` → qualified `a.b.C`; wildcard records base `a.b` (at most one active wildcard attribution per D16). `export` treated like `import`.
5. **Edges/references/call-sites per D16**: `extends`/`with` parents → edge (Complete iff resolvable via import-map ∪ same-file namespace ∪ same-package siblings declared in this file; else Incomplete). Type positions (`: T`, `new X`), `X(` calls, and same-package/in-file uses → Complete edge (dedup by `(src,dst,kind)`) + `SymbolReference` at every occurrence (incl. inside bodies). Single-wildcard usages → Incomplete edge with qualified guess + reference. Unresolvable → reference-with-raw-name only or nothing (pick one rule, document, test it). Call-sites for `new X` and `X(` with `caller_symbol` = enclosing tracked symbol. src/dst file ids: mirror `lang-cpp/src/analyzer.rs`'s SemanticEdge convention if it resolves files, else `FileId(0)` sentinel (verify downstream: `merge_tu_edges` ignores files; `IndexResult::new` counts by kind only).
6. **Scanner (D18)**: `//` line, nested `/* */`, `'c'` (incl. `'\n'`, `'''` char-of-symbol edge cases), `"…"`, `"""…"""` (run-to-run leak acceptable ONLY on invalid input, same concession as M20), interpolated `s"…"`/`f"…"` with `${…}` **not analyzed** (limitation + test: fixture's `s"${user.name} ordered item ${order.id}"` yields zero symbols/edges/references from the splice), backtick identifiers.
7. **Output determinism**: source order; dedup stable.

## RGB plan

- **Red first** (must fail before implementation — compile error counts): unit tests per plan §6.2 row `lang-scala::parser`, each on minimal inline sources: class / trait / object / case class / given (fn and value forms) / companion pair (class+object, ids `X` vs `X$`, tie-break rules) / `extends` local ⇒ Complete / `extends` unresolvable ⇒ Incomplete / `with` / `new X` reference+callsite / named+grouped imports feeding a Complete edge / wildcard ⇒ Incomplete edge to qualified guess / same-package sibling reference without import (mirror of `models/Order.scala`→`User`) / nested `package` graceful-negative / commented-out decl, block-commented decl, decl inside `"…"`, `"""…"""`, `s"…${…}…"` ⇒ zero symbols/edges/references from the region. Plus a **fixture-corpus test** (read the 5 main-root `.scala` files from `tests/fixtures/scala-basic` via `env!("CARGO_MANIFEST_DIR")` — read-only) with exact expected symbol/edge sets for `OrderService.scala` (expect: class OrderService + Complete edges to `com.example.models.Order`/`…User` + ArrayBuffer Incomplete edge + refs incl. `User`/`Order` occurrences; assert the interpolated-splice negatives) and `Order.scala`, `User.scala`, `Main.scala`, `all.scala` (package-object id/kind).
- **Green**: minimal implementation. **Refactor**: scanner/parser separation, functions < 100 lines, predicate-name helpers (`is_definition_keyword`, …), no `tmp`/`res`.

## Commands & gates

- `cargo test -p ariadne-lang-scala` (600 s timeout) + `python tools/hygiene_check.py` exit 0. No `--workspace`.
- Commit only on green: `feat(m21.1): ScalaSourceParser - package/decl/extends edge + reference extraction with offline-fidelity scanner`. Body: decision refs + any documented limitations.
- Report back: commit SHA, test count delta, list of every documented parser limitation, any contract ambiguity you resolved (I will fold rulings into the kickoff execution notes).
