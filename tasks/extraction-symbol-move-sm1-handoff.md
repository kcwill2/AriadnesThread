# SM-1 Handoff - extraction-symbol-move: the neutral move-text hook on DependencyRewriter (default delegates; Scala byte-identical by construction)

Milestone: `extraction-symbol-move` (UNREGISTERED post-M28 track, no
M-number). Hand 1 of 5, strictly serial: SM-1 -> SM-2 -> SM-3 -> SM-4 -> SM-5.
SM-2 implements the hook for C++; SM-3 wires it at the strip call; SM-4 makes
the dry run honest; SM-5 makes the copy tell the truth. Author rev `2177a85`
on `main` (planner-verified 2026-10-23; census anchor `1837 passed / 0 failed
/ 13 ignored / 119 suites @ b201aa6`, HEAD `2177a85` docs-only above it, tree
clean except the pre-existing dirty `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`).
Start rev: your dispatch head - journal it verbatim; every reviewer diff is
against it.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-symbol-move-kickoff.md` section 1 (decision of record +
   three-layer reversibility), section 7 item 1 (YOUR amendment: the I5 doc
   sentence, DOC LINE ONLY), section 5 (inherited locks).
3. `tasks/extraction-symbol-move-shape.md` C1 (your contract, binding shape),
   invariants SM-I5 (Scala neutrality by delegation) and SM-I9, section 4
   (plugin split: the trait owns the neutral contract, each language owns its
   retention form).
4. `docs/specs/unit-test-audit.md` section "Deferred findings (closeout ledger
   sync, 2026-10-19)" - MANDATORY intake read before touching lang front-ends.
   Planner intake result 2026-10-23: zero deferred items name
   `crates/lang/src/rewriter.rs` or the move-hook seam (the lang-cpp items name
   detector/planning/shaper/backend/parser surfaces; the lang-scala item names
   the scanner/directive_parser newline seam - Scala files are zero-diff this
   milestone). Carry any new finding you discover; report, do not fix.
5. Your file, current state: `crates/lang/src/rewriter.rs` (221 lines) - the
   I5 doc sentence at :94-98, the `remove_top_level_symbols` default at
   :99-101, the `InertRewriter` fake at :143-149, the pin
   `remove_top_level_symbols_default_is_none_copy_semantics_preserved` at
   :161-168 (byte-identical UNEDITED - it stays).
6. The Scala override you must NOT need to touch:
   `crates/lang-scala/src/dependency_rewriter.rs:109-115` (delegates to
   `crate::locator::removal::removal_text`). The trait default you add
   forwards to the SAME method Scala overrides - that composition IS the
   byte-identity argument (SM-I5).
7. The delegation proof your hand must keep green UNEDITED:
   `crates/validate/tests/move_removal_transaction_test.rs` - its fake
   implements only `remove_top_level_symbols` (:44), so after SM-3 switches
   the strip call to YOUR new hook, the fake still reaches the strip through
   your default. Gate 4 runs it.
8. `python tools/checkpoint_guard.py status` - FIRST HAND OF THE MILESTONE:
   run it as your bootstrap before any mutation (the architect and planner
   hops had no bash). Expected clean tree except the `.vsix` stray; anything
   beyond it is a STOP-and-report.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/extraction-symbol-move-sm1-journal.md (checkpoint journal; create it)
- crates/lang/src/rewriter.rs (the new default hook + the I5 doc lift + two
  pin tests + one fake)

You MUST NOT touch: `crates/lang-scala/**` (zero-diff rule - Scala opts out of
your hook by simply not overriding it), `crates/lang-cpp/**` (SM-2's crate),
`crates/validate/**`, `crates/cli/**`, `crates/ui/**`, every other file in
`crates/lang`, every existing test in `rewriter.rs` (all seven current
`#[test]`s byte-identical, pins included). No `git stash`, no branch change.
Cargo.lock: no new dependencies (std only), so it must not appear in your
diff; if cargo rewrites it, revert.

## What to build (shape C1 - the contract)

1. Replace ONLY the doc block at `rewriter.rs:94-98` (the `remove_top_level_symbols`
   doc). The function signature and body at :99-101 stay byte-identical. The
   new doc must state the truth (pure ASCII, no em dash): the default is still
   `None`; the engine never removes for languages that do not opt in; since
   `extraction-symbol-move` the retention decision lives one level up in
   `move_out_top_level_symbols`, whose default delegates here (deletion), so
   Scala keeps its deletion behavior through this hook and C++ chooses marked
   comment-out. Keep the D-M23-5 reference.
2. Add the new hook to `pub trait DependencyRewriter` (place it directly
   after `remove_top_level_symbols`), exactly this contract:

   ```rust
   /// The post-move source-text contract: return the text that REPLACES
   /// `source` once the named top-level symbols have moved into their
   /// extracted units, or `None` when any name cannot be safely retired -
   /// the caller then proceeds WITHOUT the move (copy fallback). The
   /// RETENTION FORM is the language's choice: Scala deletes via the
   /// inherited parser spans (through the default below); C++ comments
   /// regions out under `ARIADNE-EXTRACT-OUT` markers bound to `plan_id`
   /// (extraction-symbol-move, the user's reversible-move decision of
   /// record). `plan_id` is marker provenance only - a language whose
   /// retention needs no provenance ignores it. The default delegates to
   /// `remove_top_level_symbols` (deletion), so every existing
   /// implementation keeps byte-identical behavior.
   fn move_out_top_level_symbols(
       &self,
       source: &str,
       qualified_names: &[String],
       _plan_id: &str,
   ) -> Option<String> {
       self.remove_top_level_symbols(source, qualified_names)
   }
   ```

   The `_plan_id` underscore on the DEFAULT binding is intentional (the
   default ignores provenance); implementers override with a named binding.
   No rename and no signature change to ANYTHING existing (SM-I5: Scala pins,
   the validate fake, and every caller resolve through the default).
3. Two pin tests + one fake in `mod tests` (house naming, no banned names):
   - fake `struct DeletingRewriter;` implementing `rewrite` (identity) and
     `remove_top_level_symbols` (returns `Some(source.replace("class Drop\n",
     ""))` - a visible byte transform so delegation is observable).
   - test `default_move_out_delegates_byte_identically_to_remove_hook`:
     for the same (source, names) pair assert
     `DeletingRewriter.move_out_top_level_symbols(s, n, "PLAN-TEST")` equals
     `DeletingRewriter.remove_top_level_symbols(s, n)` field-for-byte
     (both `Some`, byte-identical). Message: "SM-I5: the default IS the old
     hook - every existing implementation keeps byte-identical behavior".
   - test `default_move_out_refuses_exactly_where_the_remove_hook_refuses`:
     `InertRewriter.move_out_top_level_symbols("int x;", &["x".to_string()],
     "PLAN-TEST")` is `None`, message naming SM-I5 and the copy fallback.

## Frozen-lock amendment clause (kickoff section 7 item 1 - binding)

You are the ONLY hand that may ever edit the I5 doc sentence, and only the
DOC block above `remove_top_level_symbols` (`:94-98`). Proof obligations the
reviewer enforces per-commit against your start rev:
- The `remove_top_level_symbols` fn signature/body (:99-101) and the pin
  `remove_top_level_symbols_default_is_none_copy_semantics_preserved`
  (:161-168) are byte-identical UNEDITED - the pin's own message keeps its
  historical em dash (pre-existing legitimate UTF-8; do NOT "fix" it, do NOT
  add non-ASCII bytes of your own).
- All six other `#[test]`s + `InertRewriter` + `RewriteSpec` + `DirectiveHit`
  + the `rewrite`/`find_dependency_targets`/`consumer_discovery_targets`/
  `consumer_rewrite_spec`/`render_diff`/`emit_file_preamble` defaults:
  byte-identical.
- Discovery of any OTHER pre-existing item wanting an edit: STOP-and-report
  (CW-3 discipline precedent).

## RGB protocol

1. Red - JUSTIFIED RED-SKIP, declared per the ui-extraction-wizard W-1 +
   extraction-consumer-rewrite CW-2 precedent: no assertion can fail against
   the pre-hand tree WITHOUT the new method existing, and a compile-failing
   test is NOT a Red (M23.5 F14 defect 4 class: never pin what the step
   ordering cannot force observable). Journal the justification with its two
   precedent citations and the falsifiability statement: the two pin tests
   fail if the default returns `None` (left `None`, right `Some(...)`) or
   forwards wrong arguments (byte mismatch); the historical Red of the old
   contract is the existing `remove_top_level_symbols_default_is_none_...`
   pin, which must be green UNEDITED at gate 2 (that pin IS the pre-hand
   truth witness).
2. Green: doc lift + hook + fake + two tests; re-run gates 2-4 until green.
   NEVER weaken an assertion to pass.
3. Refactor: doc wording plain, honest names; re-run.

## Hygiene (review-enforced)
Functions < 100 lines (the default is 3); `rewriter.rs` 221 -> ~290 (< 1000);
no banned names, no single-letter locals; authored bytes pure ASCII (the new
doc block and tests carry NO em dash, NO non-ASCII);
`python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- Scala suites UNEDITED + green: gate 3 (`-p ariadne-lang-scala`) + no diff
  hunk under `crates/lang-scala/` (SM-I5).
- validate's delegation proof green UNEDITED: gate 4
  (`move_removal_transaction_test.rs`, its fake implements the OLD hook only
  - after SM-3 that is THE SM-I7 rollback proof, so your default must stay
  the pure forwarder written above).
- The five trait-default pins (`find_dependency_targets_default_is_empty_...`,
  `remove_top_level_symbols_default_is_none_...`,
  `pin_neutral_consumer_discovery_targets_...`,
  `pin_neutral_consumer_rewrite_spec_default_is_none`,
  `pin_neutral_render_diff_default_uses_bare_marker_grammar`) green UNEDITED:
  inside gates 2 and 5.
- goldens/fixtures byte-locked; `tools/**` operator-law; `tests/**` in no
  diff (SM-1 touches zero test files outside the crate).

## Gates (exact commands, timeout 600000 ms each; distilling entry points only;
raw cargo is a reviewer blocker)
1. `python tools/testrun.py --distill -p ariadne-lang` - clean baseline BEFORE
   any edit; journal the canonical line verbatim.
2. `python tools/testrun.py --distill -p ariadne-lang` - Green: exit 0,
   passed = baseline + 2 (the two pins), the seven pre-existing tests green
   UNEDITED.
3. `python tools/testrun.py --distill -p ariadne-lang-scala` - exit 0 UNEDITED
   (delegation composition witness).
4. `python tools/testrun.py --distill -p ariadne-validate --test
   move_removal_transaction_test` - exit 0 UNEDITED (the fake resolves
   through your default).
5. `python tools/testrun.py --distill` - full workspace census; reconcile
   from your journal baseline and attribute every delta (expect +2 passed,
   0 failed, 13 ignored, 119 suites held; anchor of record
   `1837 passed / 0 failed / 13 ignored / 119 suites @ b201aa6`; the
   `--baseline` FILE form stays superseded by this `--distill` substitute).
6. `python tools/distill.py -- python tools/bench_compare.py` - fresh
   recording (kickoff sec. 5 mandates it per hand); WARNs recorded, NOT
   attributed (PR-2 persists).
7. `python tools/hygiene_check.py` - exit 0.
8. Commit cycle (itemized per the 2026-10-06 M28 law):
   `python tools/encoding_guard.py --staged` (or the `--all` superset if the
   staging seam refuses - journal the form used) then
   `python tools/commit_scoped.py --scope crates/lang/src/rewriter.rs --scope
   tasks/extraction-symbol-move-sm1-journal.md -m "feat(lang): add the neutral
   move_out_top_level_symbols contract with a delegating default
   (extraction-symbol-move SM-1)"`. Never `git add .`; the journal rides
   inside the landing commit.

Declared gate budget (2026-10-06 M28 law): 7 scoped invocations + the commit
cycle - inside `<= ~10 + commit-cycle trio`; itemize per-gate counts in the
journal and declare any overshoot as a dated D-row (silence is the deviation).

## Discipline clauses
- UTF-8: never Set-Content/Out-File/`>`/`Get-Content | -replace`; Write/Edit
  tools only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- After EACH Green gate append a 1-3 line checkpoint to the journal.
- Context exhaustion looming: STOP, return a disk-state inventory; a dead
  session is never resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-symbol-move-sm1-handoff.md --scope crates/lang --cmd ...`.
- Never pin warning counts; never state a count without the command that
  produces it (M23.5 F14 standing rules).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. The justified Red-skip declaration with
its two citations. 3. Green evidence: distill digest header + canonical line
for gates 2-5 + capture path. 4. Any deviation + reason. 5. Out-of-scope
observations (report-only). Never paste raw tool output; cite command +
canonical line + exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill. Extra MUSTs for this hand: the diff hunk in `rewriter.rs` is confined
to (a) the doc block above `remove_top_level_symbols`, (b) the appended hook
method, (c) appended test-module items; `crates/lang-scala/**` in no diff;
the signature line `fn remove_top_level_symbols(&self, _source: &str,
_qualified_names: &[String]) -> Option<String {` byte-identical; the five
default pins named above green UNEDITED; census +2 attributed.

Sizing (planner, itemized from the mandated bodies - M23.5 rule 4): doc lift
+~14/-5; hook method + doc ~22; fake ~12; two pin tests ~40; journal ~12 -
total ~95-140 produced lines vs the ~1200 bound; 7 scoped gates + commit
cycle vs ~10 + trio; one crate dir (`crates/lang`). No hand splits: the doc
lift without the hook ships an untrue sentence, the hook without the doc ships
a trait that contradicts itself - both half-states are the seam this hand
exists to avoid.
