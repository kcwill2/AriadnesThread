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
