# SM-2 Handoff - extraction-symbol-move: the C++ move hook - ARIADNE-EXTRACT-OUT line-prefix grammar, refusal contract, hazard corpus

Milestone: `extraction-symbol-move` (UNREGISTERED post-M28 track, no
M-number). Hand 2 of 5, strictly serial: SM-1 -> SM-2 -> SM-3 -> SM-4 -> SM-5.
SM-1's neutral hook (trait default delegating to deletion) is the contract you
implement; SM-3 wires your hook at the strip call; SM-4 makes the dry run
compute over it. Author rev `2177a85` on `main` (planner-verified 2026-10-23;
census anchor `1837 passed / 0 failed / 13 ignored / 119 suites @ b201aa6`;
your baseline is the SM-1 landing state: workspace `1839 passed / 0 failed /
13 ignored / 119 suites`). Start rev: your dispatch head - journal it verbatim;
every reviewer diff is against it.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-symbol-move-kickoff.md` section 1 (decision of record,
   three-layer reversibility, cleanup story), section 5 (inherited locks,
   ASCII A9).
3. `tasks/extraction-symbol-move-shape.md` C2 (your contract, binding shape:
   grammar, line-prefix rule, plan-id/names provenance, recognition regexes,
   bottom-up + merge, CRLF, refusal contract, cleanup story), invariants
   SM-I1/SM-I2/SM-I9, risk R2 (the M25 span-safety hazard class - your
   refusals are its mitigation), section 8 (naming + size notes).
4. `docs/specs/unit-test-audit.md` section "Deferred findings (closeout ledger
   sync, 2026-10-19)" - MANDATORY intake read before touching lang front-ends.
   Planner intake result 2026-10-24: zero deferred items name
   `dependency_rewriter.rs`, the move-hook seam, or a `move_retention` module
   (the lang-cpp items name detector/planning/shaper/backend/parser surfaces).
   Carry any new finding you discover; report, do not fix.
5. SM-1's product, landed and frozen: `crates/lang/src/rewriter.rs` - the
   `move_out_top_level_symbols` default (delegates to
   `remove_top_level_symbols`). You OVERRIDE it; you do not edit
   `crates/lang` (frozen this hand).
6. `crates/lang-cpp/src/dependency_rewriter.rs` (390 lines; planner-verified
   today): your hook method lands inside `impl DependencyRewriter for
   CppDependencyRewriter` directly after `consumer_rewrite_spec` (ends :113);
   the four frozen fns (`rewrite` :25, `render_diff` :40,
   `emit_file_preamble` :58, `find_dependency_targets` :79) and
   `consumer_rewrite_spec` stay byte-identical.
7. Your region source, READ ONLY, zero diff: `crates/lang-cpp/src/
   symbol_locator.rs` - `pub(crate) fn locate(source, wanted: &[(String,
   SymbolKind)]) -> Vec<SymbolBody>` (:24); the module doc :11-15 states the
   matching contract you adopt verbatim: comment contents never reach the
   token stream, and a record lookup is a BLESSED answer for a constructor's
   qualified name ("`Thing` answers a `Thing::Thing` constructor lookup").
   The hook NEVER re-slices text the locator did not certify. Import the same
   `SymbolKind` the locator module imports (:18).
8. `crates/lang-cpp/src/lib.rs` (55 lines): private mods declared :26-27
   (`mod symbol_locator; mod tokenizer;`). Your `mod move_retention;` line
   goes directly before `mod symbol_locator;` (:26), keeping that group's
   alphabetical order. PLANNER NOTE (dated 2026-10-24, A-3 compile-required
   precedent): the shape's SM-2 module map omitted this +1-line lib.rs edit;
   it is a compile prerequisite, not a design change - the touch-set below
   carries it and the reviewer accepts it as declared.
9. The fixture your tests must actually mark (planner-verified bytes, 13
   lines): `tests/fixtures/basic/include/thing.h` - `class Thing {` at :6
   through `};` at :13, the ctor declaration at :8 and the method declaration
   at :9 sit INSIDE the record span; plan id `PLAN-001` (m20 :88-91).
10. `python tools/checkpoint_guard.py status` - run it as your bootstrap
    before any mutation. Expected clean tree at the SM-1 landing except the
    pre-existing dirty `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`;
    anything beyond it is a STOP-and-report.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/extraction-symbol-move-sm2-journal.md (checkpoint journal; create it)
- crates/lang-cpp/src/move_retention.rs (NEW: grammar builders, region
  builder, refusal preflight, 10 unit tests + mod tests)
- crates/lang-cpp/src/dependency_rewriter.rs (the one hook method delegating
  to `move_retention`; ~+25 lines, file 390 -> ~415)
- crates/lang-cpp/src/lib.rs (+1 `mod move_retention;` line at :26, the
  declared compile prerequisite above; no other byte moves)

You MUST NOT touch: `crates/lang/**` (SM-1's product is frozen), `crates/lang-scala/**`
(zero-diff rule), `crates/validate/**`, `crates/cli/**`, `crates/ui/**`,
`crates/lang-cpp/src/symbol_locator.rs` + its test modules, `tokenizer.rs`,
`parser.rs`, `source_extractor.rs`, every other file in `crates/lang-cpp`,
every existing test (all 16 `#[test]`s in `dependency_rewriter.rs`
byte-identical), `tests/**` (the riders land with SM-3). No `git stash`, no
branch change. Cargo.lock: no new dependencies (std only), so it must not
appear in your diff; if cargo rewrites it, revert.

## What to build (shape C2, with the two planner refinements below - the
refinements refuse MORE, never less; none changes the mechanism)

1. Grammar (pure ASCII by construction; every generated line well under 2000
   chars). For each located, line-aligned span, emit exactly:

   ```
   // ARIADNE-EXTRACT-OUT BEGIN plan=<plan-field> symbols=<names-field>
   //<original line 1>
   //<original line 2>
   //A
   // ARIADNE-EXTRACT-OUT END plan=<plan-field>
   ```

   - LINE-PREFIX COMMENT ONLY: two bytes `//` prepended to the ORIGINAL line
     bytes (no added space; stripping the prefix restores the original
     exactly; an empty original line becomes bare `//`). Block comments are
     FORBIDDEN as a retention form (C2).
   - `<plan-field>`: the `plan_id` argument rendered by `fn
     plan_field_ascii(plan_id: &str) -> String` replacing every character
     outside printable-ASCII-excluding-space (`0x21..=0x7E`) with `?` - the
     A9 ASCII channel plus the `$`-anchored `\S+` recognition grammar's
     no-space demand. Tests use `PLAN-TEST` and one non-ASCII specimen built
     from byte escapes (AGENTS.md fixture rule), asserting the `?` rendering.
   - `<names-field>`: requested qualified names joined by `;`, each first
     mapped char-by-char: characters outside `[A-Za-z0-9_:]` to `_`
     (documented one-way; names are human provenance, the byte truth lives in
     the region lines, restore does not depend on them).
   - Recognition contract (future purge tooling, kickoff section 6): lines
     matching `^// ARIADNE-EXTRACT-OUT BEGIN plan=(\S+) symbols=(\S+)$` and
     `^// ARIADNE-EXTRACT-OUT END plan=(\S+)$`. Pin both regexes' behavior in
     a test: every produced marker line matches, every region content line
     does NOT.
   - CRLF: the file's per-line terminators are preserved verbatim, marker
     lines use the file's dominant terminator.
2. Span resolution per requested name (the locator is READ ONLY): call
   `locate` with the kind preference order `Class, Struct, Union, Enum,
   Function, TypeAlias` and take the FIRST located body as the certified
   span (the locator doc blesses the record answer for a ctor lookup;
   `Other` is never tried). Then resolve that verbatim body text to whole
   source lines per the two PLANNER REFINEMENTS (both are refusals, each a
   named test):
   - R-C (byte uniqueness): the body must occur exactly ONCE in the source
     outside existing marked regions; multiple byte-identical occurrences =
     cannot safely retire = `None` (retaining the wrong copy would delete the
     wrong declaration - the same refuse-rather-than-guess discipline as
     `consumer_rewrite_spec`'s honest fallback).
   - R-D (line alignment): on the span's first line the bytes BEFORE the body
     must be whitespace only, and on the last line the bytes AFTER the body
     must be whitespace only; otherwise `None`. This is what makes SM-I1's
     "region lines minus the prefix equal the original bytes" literally true
     for WHOLE original lines.
3. Refusal contract (each refusal a named unit test, per C2; a refusal means
   the whole candidate is skipped whole - per-unit atomicity, mirroring the
   `collect_move_removals` T8b comment). `None` when any requested name:
   - R-A: no kind locates it (e.g. `ThingConfig` against the 13-line fixture
     header - journal-verified outcome of that exact pair).
   - R-B: locatable ONLY inside an existing marked region (idempotence; never
     nested marking. Mechanism: the locator's tokenizer cannot see comment
     contents, so re-offering post-mark text is refused by R-A's own route -
     your test pins the OUTCOME, which is what SM-I1 asserts).
   - R-C: body occurrence ambiguous (refinement above).
   - R-D: span not line-aligned (refinement above).
   - R-E: an unterminated block comment (`/*` without a matching `*/`) exists
     in the bytes before the region start (conservative scan of the pre-region
     text; C2 as written).
   - R-F: the line immediately BEFORE the region ends in a backslash (the
     BEGIN marker would be pulled into that line's logical continuation).
   - R-G: the region's LAST line ends in a backslash (the END marker would
     splice into the comment chain and break the balanced-pair grammar).
   Refusal tests are PAIR-FORMED (Red non-vacuity per M23.5 F14 rule 4): each
   asserts the hazard input refuses AND an adjacent clean variant marks
   (without the pairing, the trait default's blanket `None` would satisfy the
   refusal test vacuously and the Red capture would be hollow).
4. Multiple spans in ONE call: adjacent or overlapping spans merge to one
   region; regions apply bottom-up so earlier line indices stay valid (C2).
5. The hook on `impl DependencyRewriter for CppDependencyRewriter` (place
   directly after `consumer_rewrite_spec`): `fn move_out_top_level_symbols(
   &self, source: &str, qualified_names: &[String], plan_id: &str) ->
   Option<String>` delegating to `crate::move_retention::move_out_text(
   source, qualified_names, plan_id)`. Doc comment states (pure ASCII, no em
   dash): marked comment-out retention is C++'s chosen form of the
   extraction-symbol-move decision of record; provenance is the plan id;
   refusals answer `None` so the caller proceeds without the move (copy
   fallback); block comments are never used.
6. Module layout (shape section 8 names, no banned names, no single-letter
   locals): `crates/lang-cpp/src/move_retention.rs` with `pub(crate) fn
   move_out_text(...)`, free fns `region_marker_open`, `region_marker_close`,
   `prefix_out_region`, `plan_field_ascii`, `normalize_marker_name`, and the
   refusal preflight as named helpers; every function well under 100 lines.

## PLANNER VERDICT ON THE FIXTURE OUTCOME (dated 2026-10-24, F14-class shape
relay correction - read before writing any "4 pairs" expectation)

Shape SM-I1 says the basic fixture gets "four pairs (four units, four pairs)".
That count is a relay error - the machine truth, planner-derived today from
`collect_move_removals` (`crates/validate/src/lib.rs:345-370`, one entry per
unit), the `BTreeSet` order (`["Thing"] < ["Thing::Thing"] <
["Thing::get_id"] < ["ThingConfig"]`), the fixture bytes (ctor :8 and method
:9 INSIDE the class record :6-13), the tokenizer's comment invisibility, and
this hand's own R-B refusal: the class entry is offered first and marks ONE
merged region; the ctor and method entries then locate only inside the marked
region and refuse (idempotence); `ThingConfig` is unlocatable in this file and
refuses. Fixture outcome after SM-3: exactly ONE balanced BEGIN/END pair
bound to `PLAN-001`, `symbols=Thing`, region content = the 8 original lines
under the `//` prefix, every other byte of the file unchanged. The mechanism
is unchanged by this correction; SM-3's riders and SM-4's oracle pin the
machine truth (balanced pairs, byte fidelity, plan binding, re-offer refusal)
with the fixture count at one. Journal your measurement; do not reproduce the
shape's 4-pair wording in any test message.

## RGB protocol

1. Red: author ALL of `move_retention.rs`'s ten tests + the `mod
   move_retention;` line FIRST, without the hook override (the tests call
   `CppDependencyRewriter`'s hook, which at Red resolves to SM-1's trait
   default -> `None` -> every pair-formed test fails on its POSITIVE marking
   assertion; expect messages name `extraction-symbol-move`). Gate 2 captures
   EXACTLY ten named failures; journal the failing-test list verbatim. A
   compile-failing test is NOT a Red; these compile against the SM-1 default.
2. Green: implement `move_retention` + the hook override; re-run gates 3-6
   until green. NEVER weaken an assertion to pass.
3. Refactor: honest names, plain doc wording, no duplication between the
   refusal helpers; re-run.

## Required tests (exact; in `move_retention.rs` `mod tests`, called through
`CppDependencyRewriter`'s hook so the delegation is exercised by all ten)
1. `move_out_marks_line_aligned_class_region_verbatim_under_prefix` - the
   13-line fixture header, name `Thing`, plan `PLAN-TEST`: exact BEGIN/END
   bytes, all 8 region lines prefix-stripped equal the original lines
   byte-exactly, every byte outside the region unchanged.
2. `move_out_marks_multiple_spans_bottom_up_and_merges_adjacent_regions` - a
   namespaced class + a template + a multi-line-signature function (SM-I1
   hazard corpus), two adjacent names merge to one region, non-adjacent
   regions keep their own pairs, earlier line indices valid (bottom-up).
3. `move_out_preserves_crlf_terminators_in_regions_and_markers` - CRLF file:
   region lines keep `\r\n`, marker lines use it, prefix-strip reconstructs
   the originals including `\r`.
4. `move_out_binds_marker_provenance_to_plan_id_and_normalizes_names` - a
   name containing characters outside `[A-Za-z0-9_:]` renders `_` in the
   symbols field; a plan id carrying a non-ASCII byte (built via `"\xe9"`)
   renders `?`; recognition regexes hold.
5. `move_out_refuses_unlocatable_name` (R-A pair-formed; the `ThingConfig`
   pair above).
6. `move_out_refuses_name_already_locatable_only_inside_marked_region`
   (R-B: mark, then re-offer the same name answers `None`; re-offer a
   different un-marked name in the same file still marks - pair).
7. `move_out_refuses_body_ambiguous_or_not_line_aligned` (R-C + R-D pair).
8. `move_out_refuses_region_start_inside_unterminated_block_comment` (R-E
   pair).
9. `move_out_refuses_backslash_continuation_on_the_line_before_the_region`
   (R-F pair; the macro-with-trailing-backslash hazard, SM-I1 corpus).
10. `move_out_refuses_backslash_continuation_at_the_region_last_line`
    (R-G pair).

## Hygiene (review-enforced)
Functions < 100 lines (the refusal preflight splits until true);
`move_retention.rs` ~450-520 (< 1000), `dependency_rewriter.rs` 390 -> ~415;
no banned names, no single-letter locals; authored bytes pure ASCII (no em
dash anywhere); `python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- The four frozen rewriter fns + `consumer_rewrite_spec` + all 16 existing
  `#[test]`s byte-identical (SM-I4): reviewer per-commit diff (no hunk in
  their regions) + gate 3 green.
- Generated-byte parity UNEDITED: gate 5 (`m18_cpp_plugin_parity_test`).
- Scala zero-diff + green: gate 4 + `crates/lang-scala/**` in no diff (SM-I5).
- `symbol_locator.rs`, `tokenizer.rs`, `parser.rs` in no diff (READ ONLY,
  SM-I4; R2: no refusal path may weaken the locator).
- goldens/fixtures byte-locked; `tools/**` operator-law; `tests/**` in no
  diff; Cargo.lock in no diff.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only;
raw cargo is a reviewer blocker)
1. `python tools/testrun.py --distill -p ariadne-lang-cpp` - clean baseline
   BEFORE any edit; journal the canonical line verbatim.
2. `python tools/testrun.py --distill -p ariadne-lang-cpp` - Red: exit 1,
   EXACTLY the ten named tests failing (journal the failing-test list).
3. `python tools/testrun.py --distill -p ariadne-lang-cpp` - Green: exit 0,
   passed = baseline + 10, the 16 pre-existing tests green UNEDITED.
4. `python tools/testrun.py --distill -p ariadne-lang-scala` - exit 0 UNEDITED.
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m18_cpp_plugin_parity_test` - exit 0 UNEDITED.
6. `python tools/testrun.py --distill` - full workspace census; reconcile from
   your journal baseline and attribute every delta (expect +10 passed, 0
   failed, 13 ignored, 119 suites held; upstream of record: SM-1 landing
   `1839 passed / 0 failed / 13 ignored / 119 suites`, anchor
   `1837 passed / 0 failed / 13 ignored / 119 suites @ b201aa6`; the
   `--baseline` FILE form stays superseded by this `--distill` substitute).
7. `python tools/distill.py -- python tools/bench_compare.py` - fresh
   recording (kickoff sec. 5 mandates it per hand); WARNs recorded, NOT
   attributed (PR-2 persists).
8. `python tools/hygiene_check.py` - exit 0.
9. Commit cycle (itemized per the 2026-10-06 M28 law):
   `python tools/encoding_guard.py --staged` (or the `--all` superset if the
   staging seam refuses - journal the form used) then
   `python tools/commit_scoped.py --scope crates/lang-cpp/src/move_retention.rs
   --scope crates/lang-cpp/src/dependency_rewriter.rs --scope
   crates/lang-cpp/src/lib.rs --scope
   tasks/extraction-symbol-move-sm2-journal.md -m "feat(lang-cpp): implement
   move_out_top_level_symbols with the ARIADNE-EXTRACT-OUT line-prefix
   grammar and refusal contract (extraction-symbol-move SM-2)"`. Never
   `git add .`; the journal rides inside the landing commit.

Declared gate budget (2026-10-06 M28 law): 8 scoped invocations + the commit
cycle - inside `<= ~10 + commit-cycle trio`; itemize per-gate counts in the
journal and declare any overshoot as a dated D-row (silence is the deviation).

## Discipline clauses
- UTF-8: never Set-Content/Out-File/`>`/`Get-Content | -replace`; Write/Edit
  tools only; authored content pure ASCII (AGENTS.md File-Edit Discipline);
  corruption specimens built via byte escapes, never literals.
- Touch only the touch-set; out-of-scope breakage: report, do not fix. Any
  pre-existing lock or test discovered wanting an edit: STOP-and-report
  (CW-3 discipline precedent).
- After EACH Green gate append a 1-3 line checkpoint to the journal.
- Context exhaustion looming: STOP, return a disk-state inventory; a dead
  session is never resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-symbol-move-sm2-handoff.md --scope crates/lang-cpp --cmd ...`.
- Never pin warning counts; never state a count without the command that
  produces it (M23.5 F14 standing rules).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence: the ten failing-test
names with first-assert panic lines. 3. Green evidence: distill digest header
+ canonical line for gates 3-6 + capture path. 4. Your measured fixture-marking
behavior in one fixture-shaped test (you do not touch `tests/**`; journal the
`thing.h` byte walkthrough your test 1 uses instead). 5. Any deviation +
reason. 6. Out-of-scope observations (report-only). Never paste raw tool
output; cite command + canonical line + exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill. Extra MUSTs for this hand: the diff is confined to the four touch-set
files; `dependency_rewriter.rs` shows ONE added method region only; the five
frozen fns + the 16 existing tests show no hunk; `symbol_locator.rs` +
tokenizer + parser + `crates/lang/**` + `crates/lang-scala/**` + `tests/**` in
no diff; census +10 attributed to the ten named tests.

Sizing (planner, itemized from the mandated bodies - M23.5 rule 4): production
~170-210 (marker builders ~25, prefix/strip ~20, span resolution + R-C/R-D
~50, refusal preflight ~55, merge/bottom-up ~35, hook method + doc ~25); lib.rs
+2; ten tests ~260-300; journal ~15 - total ~450-530 produced lines vs the
~1200 bound; 8 scoped gates + commit cycle vs ~10 + trio; one crate dir. No
hand split: grammar + refusals + hook in separate hands would force shipping a
grammar without refusals (the R2 hazard class ships live) or refusals without
a grammar (nothing to refuse) - both are the half-built seam this hand exists
to avoid.
