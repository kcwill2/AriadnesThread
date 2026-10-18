# UTA-T3 Journal - lang + lang-api + lang-scala pins + scanner doc correction

Contract: tasks/uta-t3-handoff.md (binding: docs/specs/unit-test-audit.md,
lang-scala doc-claim adjudication row). Base: main at 47cdf79, clean tree.
Work order: lang-scala, then lang, then lang-api.

## Journal

- 2026-10-18 start: contract + spec + all touch-set sources read. Item map
  resolved to files: item 1 (scanner.rs doc fix + two divergence pins),
  2/8/10 + skip_blanks half of 19 (scanner.rs), 1 + `_*`/given half of 19
  (directive_parser.rs), 3/4 (parser/declarations.rs), 5/6/14 (parser/mod.rs),
  12 (parser/directives.rs), 19 is_member_access half (parser/occurrences.rs),
  9 (imports.rs), 7/13 (dependency_rewriter.rs), 11/16 (locator/removal.rs),
  17 (planning.rs), 15 both halves (constraints.rs + shaper.rs), 18
  (lang-scala resolver.rs), 20/22-resolver (lang/resolver.rs), 21
  (lang/registry.rs), 22-rewriter (lang/rewriter.rs), 22-extractor
  (lang/extractor.rs), 23 (lang-api catalog.rs).

## Red phase

- lang-scala: all 23-item tests written as pins of CURRENT behavior (the
  audit row adjudicates this handoff as pin-only; the only behavior-adjacent
  change is the item-1 doc-comment correction, which changes docs, not code).
  A pin failing therefore means my trace was wrong (fix the test harness) or
  unlisted wrong behavior (pin honestly + report). First-run digest recorded
  below before any expectation was adjusted.

## Green checkpoints (1-3 lines per gate)

- Gate 1 GREEN lang-scala: `254 passed / 0 failed / 0 ignored / 2 suites`
  (testrun --distill; +27 pins, doc fix in scanner.rs). Red evidence: two
  compile harness errors (braces in assert messages, fixed) then exactly one
  pin red: sanitize_symbol_stem("\u{fc}ber") == "ber", not "symbol" - my
  misprediction, behavior matches the function's own docs (leading non-ASCII
  collapses to an underscore that the edge trim removes). Pinned honestly, no
  code change.
- Gate 2 GREEN lang: `44 passed / 0 failed / 1 ignored / 2 suites`
  (testrun --distill; +7 pins in resolver/registry/rewriter/extractor, first
  run green - no red materialized for items 20-22).
- Gate 3 GREEN lang-api: `42 passed / 0 failed / 0 ignored / 5 suites`
  (testrun --distill; +1 pin in catalog.rs normalize_relative_path: root
  clamp, CurDir edges, pure-ParentDir chain - first run green).
- Gate 4 hygiene: exit 0 (distiller-confirmed); R3 warnings are all
  pre-existing rows, none on lines this hand authored.
- Deliberately NOT done: no scanner unification (directive_parser vs
  scanner unterminated-string divergence pinned on both sides, reported);
  no code fixes anywhere - items 2-23 are pins; two_old_deps expansion
  duplicate and sanitize `symbol` collision pinned as current behavior.

## Deviations (with reasons)

1. Harness-only red, not behavior: two new assert messages contained literal
   `{`/`}` (Rust format strings) -> compile drift; reworded. testrun digest:
   `parse format drift: no test result line matched the pinned parse spec`.
2. Genuine pin-red (my misprediction, behavior correct): sanitize_symbol_stem
   with a LEADING non-ASCII run returns "ber" for "\u{fc}ber" - the run
   collapses to one underscore which the edge trim removes; only ALL-illegal
   names reach the `symbol` fallback (the function's own docs say exactly
   this). Re-pinned honestly with the corrected expectation.
3. Item 23 phrasing `/a/../../b -> /`: the true current result is `/b` (the
   second `..` is dropped at RootDir; the path stays re-rooted under `/`).
   Pinned as `/b` - no escape above root is the invariant, not a literal
   slash result.
4. Encoding-gate sequencing mirrors the accepted UTA-T1/T2 shape:
   commit_scoped stages internally, so `encoding_guard --all` (superset, CI
   form) ran exit 0 pre-commit, plus `--staged` after the commit.

## Report-flagged observations (report-only, no behavior touched)

- The item-1 divergence is REAL and stands: M21 scanner stops an
  unterminated single-quoted string at the newline; frozen m20
  directive_parser runs it to EOF and swallows following directives. Both
  pinned (`pin_scanner_unterminated_string_stops_at_newline`,
  `pin_directive_parser_unterminated_string_runs_to_eof`); docs corrected to
  reality; NOT unified per contract.
- dependency_rewriter: `rewrite` with two old_deps hitting one statement
  emits the new deps once per matching dep (duplicate new-import lines) -
  sequential per-dep replay. Pinned (`pin_two_old_deps_on_one_statement_...`).
  Owner may want a merge-once decision.
- imports scan: a truncated `import a.b.{Old` (unclosed brace) absorbs every
  later top-level identifier as a group member of base `a.b` through EOF -
  bounded and total, but the rewriter would rewrite those phantom selectors.
  Pinned; flagged for an owner policy decision (reject vs tolerate).
- planning sanitize: any two all-non-ASCII symbol names collide on the
  `symbol` fallback file stem (same stance as lang-cpp's private twin).
  Pinned; collision disambiguation would be an owner decision.
- resolver target grammar: a leading-dot spelling (`.com.example.X`) resolves
  exactly like the plain one because the catalog trims edge dots - silent
  acceptance of a malformed target. Pinned honestly.
- Companion-object id `X$` filtering in consumer_discovery_targets keeps
  companions out of probes (pre-existing doc'd behavior; the lang-scala
  override already had a test - no new pin needed beyond item 22 coverage).

## Gate record (verbatim digest lines)

- Gate 1 (red): `parse format drift: no test result line matched the pinned
  parse spec` (x1, harness); then `253 passed / 1 failed / 0 ignored /
  1 suites` (sanitize misprediction); GREEN: `254 passed / 0 failed /
  0 ignored / 2 suites`.
- Gate 2 GREEN: `44 passed / 0 failed / 1 ignored / 2 suites`.
- Gate 3 GREEN: `42 passed / 0 failed / 0 ignored / 5 suites`.
- Gate 4 hygiene: exit 0 (`### generic PASS (exit 0)` via distiller; all R3
  warns pre-existing, none on this hand's lines).
- Gate 5 encoding: `--all` pre-commit exit 0 (0 findings); `--staged`
  post-commit exit 0.
- Gate 6 commit: 6531df0 (19 files, +624/-2; the -2 are the two replaced
  scanner doc lines) + this journal-final rider commit.
