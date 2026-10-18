# UTA-T3 Handoff - lang + lang-api + lang-scala: trait defaults, path pins, scanner edge lexers

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. docs/specs/unit-test-audit.md (lang-scala doc-claim adjudication row).
3. crates/lang/src/resolver.rs, registry.rs:100-130, rewriter.rs:60-120;
   crates/lang-api/src/catalog.rs:160-200; crates/lang-scala/src/scanner.rs (doc
   lines 1-30, 195-320), parser/declarations.rs, parser/occurrences.rs,
   parser/directives.rs, imports.rs, directive_parser.rs, dependency_rewriter.rs,
   locator/removal.rs, planning.rs, shaper.rs, resolver.rs, constraints.rs.
4. Model: existing byte-exact locks in crates/lang-scala/src/locator/tests.rs.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/uta-t3-journal.md (journal; always in-set)
- crates/lang/src/**, crates/lang-api/src/**, crates/lang-scala/src/** (doc-comment
  fix in scanner.rs allowed per item 1; test modules everywhere in-set)
You MUST NOT touch: crates/lang-cpp, crates/ui, m19-m24 integration oracles (if a
pin red-flips one, report; do not edit). No `git stash`, no branch change.

## What to build
DOC FIX (no behavior change):
1. scanner.rs:3-4 and 198-199 claim unterminated single-quoted strings "run to
   EOF"; actual scan_string_literal stops at newline. Fix the doc text to
   reality; add pin `pin_scanner_unterminated_string_stops_at_newline` AND
   `pin_directive_parser_unterminated_string_runs_to_eof` (two scanners
   genuinely disagree - pin both real behaviors, report divergence, do not
   unify).

REQUIRED TESTS - lang-scala (pins):
2. char-literal false path: `kinds("'x end") == [Punct('), Ident(x), Ident(end)]`.
3. Scala-2 `class Box[T]` vs `<T>` angle-skip equivalence (skip_balanced_if_angle).
4. `private[p] sealed class C` -> sealed modifier recognized.
5. emit_parent_edge: `class X extends a.b.Base` (incomplete-to-full) and
   import-matched parent key assertions.
6. skip_rename_suffix via parser path: `import a.b.{C => D}` + D usage -> no
   reference; C usage -> edge to a.b.C.
7. dependency_rewriter on non-ASCII source (multibyte before touched import) -> byte-exact.
8. scan_numeric_literal: `1.toString + 2.0f` -> [Literal, Punct, Literal].
9. consume_group truncated `{`: bounded result pinned for rewriter scan.
10. unicode idents + `$`-lead: `class Ünïcödé $priv` tokenizes as Ident.
11. locator removal under CRLF: remove a class from CRLF source, byte-exact.
12. parser-path group wildcard `import a.b.{X, _}` -> wildcard_base a.b.
13. rewriter: export survivor keyword; two old_deps hitting one statement.
14. dedupe_extends fold: `class X extends Base with Base` -> one pair.
15. companion_groups permutation invariance; path_contains_components mixed-case.
16. removal prelude nested block comment + unbalanced `*/` bail.
17. sanitize_symbol_stem edges (non-ASCII -> "symbol" collision pinned, `:::`, `_x_`).
18. resolver helpers: trailing-dot, whitespace-only, leading-dot targets pinned.
19. directive selectors `_*` and `given` targets; is_member_access suppression;
    skip_blanks `*/ a` and line-comment-at-EOF-no-newline.

REQUIRED TESTS - lang (pins):
20. first_ondisk_include_candidate: Unspecified behaves Quoted (includer-dir first);
    absolute in-root strip vs out-of-root verbatim.
21. LanguageRegistry::detect multi-plugin first-non-Other; root_entries_end_with
    nonexistent root -> false.
22. Neutral trait defaults: consumer_discovery_targets ("a/b.h" -> ["b.h"],
    "." -> []), consumer_rewrite_spec, render_diff neutral, locate_symbol_bodies
    empty, places_generated_units_in_place false, directive_edge_kind None.

REQUIRED TESTS - lang-api (pins):
23. normalize_relative_path: `/a/../../b` -> `/` (no escape above root), dot-edge unify/strip edges.

## RGB protocol
Per item: Red, Green, Refactor. Pins that go RED against current behavior and
expose unlisted wrong behavior: report, do not fix (outside listed fixes).

## Hygiene (review-enforced)
Functions < 100 lines, files < 1000 lines, banned-name ban, no single-letter
locals. `python tools/hygiene_check.py` exit 0.

## Frozen locks
Existing tests are contracts; corpus no-panic tests may be KEPT but never
weakened. m19-m24 Scala oracles: report-only on conflict.

## Gates (exact commands, timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-lang-scala` (red then green)
2. `python tools/testrun.py --distill -- -p ariadne-lang`
3. `python tools/testrun.py --distill -- -p ariadne-lang-api`
4. `python tools/hygiene_check.py` -- exit 0.
5. `python tools/encoding_guard.py --staged` -- exit 0.
6. Commit: `python tools/commit_scoped.py --scope "crates/lang/**" --scope "crates/lang-api/**" --scope "crates/lang-scala/**" --scope "tasks/uta-t3-journal.md" -m "test(lang,lang-api,lang-scala): uta-t3 scanner/path/def-impl pins + doc correction"`

## Discipline clauses
- UTF-8: Write/Edit only, pure-ASCII; multibyte specimens via byte escapes.
- Touch only the touch-set; report out-of-scope breakage.
- No bare cargo; distilling entry points only.
- 1-3 line journal checkpoint per Green gate to tasks/uta-t3-journal.md.
- Context exhaustion imminent: STOP + disk-state inventory (work crate order:
  lang-scala first, then lang, then lang-api).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence. 3. Green distill headers +
canonical lines + capture path. 4. Deviations. 5. Out-of-scope observations.
Never paste raw tool output.
