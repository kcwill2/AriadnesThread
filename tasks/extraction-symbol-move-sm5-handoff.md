# SM-5 Handoff - extraction-symbol-move: move-truth copy (S3/S4/S5/S6) + the S5 keying amendment + the marked-retention no-rot lock

Milestone: `extraction-symbol-move` (UNREGISTERED post-M28 track, no
M-number). Hand 5 of 5, strictly serial: SM-1 -> SM-2 -> SM-3 -> SM-4 -> SM-5.
SM-2/SM-3 shipped marked-retention apply, SM-4 made the dry run compute it
honestly; your copy must say what now ships - and your no-rot lock makes the
old sentence impossible to reintroduce. Author rev `2177a85` on `main`
(planner-verified 2026-10-23; your baseline is the SM-4 landing state:
workspace `1851 passed / 0 failed / 13 ignored / 119 suites`; YOUR landing is
the milestone census expectation `1853 passed / 0 failed / 13 ignored /
119 suites`). Start rev: your dispatch head - journal it verbatim; every
reviewer diff is against it.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract - every replacement below is byte-exact; copy
   drift turns your own lock red, the CW-3 lesson).
2. `tasks/extraction-symbol-move-kickoff.md` section 1 layer 3 (post-commit
   FULL reversibility is explicitly NOT promised: restore = uncomment +
   revert consumer include rewrites, or git; the copy must never claim a
   one-click undo), section 7 item 4 (YOUR amendment: the S5 keying clause).
3. `tasks/extraction-symbol-move-shape.md` C6 (the four lines that CHANGE,
   the lines that STAY - state them in your journal so you do not "fix" a
   line the audit proved true), C7 item 4 (your amendment), invariants
   SM-I8/SM-I9, risk R6 (uncomment-redefinition confusion - the restore
   procedure ships IN the sentence).
4. `docs/specs/unit-test-audit.md` section "Deferred findings" - MANDATORY
   intake read. Planner intake result 2026-10-24: zero deferred items name the
   wizard asset or `extraction_wizard_render.rs` copy locks. Carry any new
   finding; report, do not fix.
5. Your touch sites, current bytes (planner-verified 2026-10-24):
   `crates/ui/assets/extraction-wizard.js` (606 lines) -
   `COPY_VS_MOVE_SENTENCE` constant :219 (stays byte-identical - true on the
   `move_removals == 0` fallback branch), the pending/unavailable/copy
   branches :353-357 (untouched), S3 move branch :359, `wizardCopyOnly`
   :306-309 (untouched - its lock reads the body), S4 keyed block :388-392
   (only :392's move branch changes), the S4 truncation note :386 (STAYS -
   CW-3's no-rot lock pins its string), S5 return concat :426-432 (only the
   :430 sentence line becomes the keyed block), the S6 artifacts line :575
   (one clause). `crates/ui/tests/ui_test/extraction_wizard_render.rs`
   (890 lines) - `COPY_VS_MOVE_SENTENCE` const :331, the amended test
   `test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path`
   :400-429 (existing asserts byte-identical; you APPEND), the fetch-site
   bound lock (never restore `count == 1`; you add zero fetches), the ASCII
   locks, the A-5 S5 locks :672-693 (green UNEDITED - S5 keeps
   `COPY_VS_MOVE_SENTENCE`, the three verbs, and `escapeHtml(planId || '')`).
6. Amendment-table item 5 CLOSED as NO-OP (planner grep-verified 2026-10-24):
   NO existing lock anywhere under `crates/ui/tests` pins "strips", "This
   plan", "as it writes", or "in place for move languages" - the :575 reword
   is a plain copy edit (shape C7 item 5 anticipated a possible W-3b pin;
   there is none). If any pre-existing lock nonetheless fires on your copy
   edit: STOP-and-report (X8/R8-class; do NOT amend it).
7. `python tools/checkpoint_guard.py status` - bootstrap before any mutation.
   Expected clean tree at the SM-4 landing except the pre-existing dirty
   `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`; anything
   beyond it is a STOP-and-report.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/extraction-symbol-move-sm5-journal.md (checkpoint journal; create it)
- crates/ui/assets/extraction-wizard.js (the ONE new constant + the four copy
  sites below - zero code change beyond the keyed S5 block's ternary; no new
  `fetch(`, no class-name change, no function rename)
- crates/ui/tests/ui_test/extraction_wizard_render.rs (the named test's
  APPENDED clauses + one APPENDED lock test + one appended `const` - no
  existing byte changes)

You MUST NOT touch: `crates/ui/tests/ui_test/main.rs` (NO new test file - the
lock APPENDS to the existing file, the CW-3 ruling), `hotspots.js` (the
claim-of-record block + its lock stay green UNEDITED), `index.html`,
`styles.css`, routes/DTOs/state, other ui_test files, `crates/cli/**`, every
other file. No `git stash`, no branch change. Cargo.lock must not appear.

## What to build (shape C6 items 1-4; every mandated string was Read-verified
contiguous IN THIS FILE'S POST-EDIT FORM by the planner 2026-10-24 - the CW-3
DATED AMENDMENT lesson is pre-baked: no pin below crosses a line wrap)

1. NEW constant, inserted directly below `COPY_VS_MOVE_SENTENCE` (:219), on
   ONE line (the :219 idiom is what makes the lock pin contiguous):

   ```
   // The marked-retention move sentence (extraction-symbol-move): what the C++
   // apply actually does with moved declarations, plus the honest restore
   // procedure (kickoff section 1 layer 3 - never a one-click claim).
   const MOVE_RETENTION_SENTENCE = 'the plugin comments the moved declarations out in place under ARIADNE-EXTRACT-OUT markers that name the plan; restoring means uncommenting those regions AND reverting the consumer include rewrites, or using git - never a one-click undo.';
   ```

2. S3 move branch (:359) - the line becomes EXACTLY:

   ```
       copyMoveLine = `<div class="wizard-impact-line">This plan moves ${escapeHtml(dryRun.move_removals)} declaration(s): ${escapeHtml(MOVE_RETENTION_SENTENCE)}</div>`;
   ```

3. S4 move branch (:392) - the branch line becomes EXACTLY:

   ```
           : `<div class="wizard-impact-line">As it writes, this plan retires ${escapeHtml(dryRun.move_removals || 0)} source declaration(s): ${escapeHtml(MOVE_RETENTION_SENTENCE)}</div>`);
   ```

   (The count's MEANING after SM-4's fold: names the hook retires = names
   bound into markers - SM-I3; the sentence says exactly that.)
4. S5 Execute (amendment 4): replace the single :430 sentence line with a
   keyed block, mirroring S4's structure, null-safe (S5 can render before any
   dry run - a bare S4-clone would throw on null): inside
   `renderWizardStepExecute` add

   ```
     const moveTruthLine = !wizardState.dryRun
       ? '<div class="wizard-impact-fallback">Copy-vs-move pending: the dry run has not been fetched yet (step 5).</div>'
       : wizardCopyOnly(wizardState.dryRun)
         ? `<div class="wizard-impact-line">${escapeHtml(COPY_VS_MOVE_SENTENCE)}</div>`
         : (wizardState.dryRun.includes_unavailable
             ? '<div class="wizard-impact-fallback">Includes unavailable: the provider could not complete this dry run; nothing is claimed about the writes.</div>'
             : `<div class="wizard-impact-line">On apply this plan retires ${escapeHtml(wizardState.dryRun.move_removals)} moved declaration(s): ${escapeHtml(MOVE_RETENTION_SENTENCE)}</div>`);
   ```

   and the return concat line that carried the old :430 template becomes
   `    moveTruthLine +`. The keyed branch keeps `COPY_VS_MOVE_SENTENCE`
   (true on the copy branch), keeps `.copy_only` unread (the existing
   absence lock at :426-428 survives), and keeps every A-5 S5 lock green
   UNEDITED (:631-652, :656-667, :672-693 - all their substrings survive your
   hunk).
5. S6 help line (:575) - ONE clause reword, rest of the line byte-identical:
   `(or in place for move languages)` -> `(or in place for languages that
   stage units in place)`. The line stays ONE line.
6. Lines that STAY (audit-proved true under move truth; journal this list so
   no one "fixes" them): the claim of record "the UI applies with a dry-run
   first, validation in three tiers and automatic rollback";
   `COPY_VS_MOVE_SENTENCE` constant bytes; the S4 truncation note :386 + its
   CW-3 lock; the tier/rollback S6 lines :571-574 (the original file IS in
   the snapshot now - truer than before); "The dry run changes nothing; tool
   apply performs the same computation before writing."; `wizardCopyOnly`
   body.
7. The locks - APPEND to `extraction_wizard_render.rs`:
   - amendment 4, appended INSIDE
     `test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path`
     (all six existing asserts byte-identical), with the test-file-side
     sentence const appended beside `COPY_VS_MOVE_SENTENCE` :331:

     ```
     const MOVE_RETENTION_SENTENCE: &str = "the plugin comments the moved declarations out in place under ARIADNE-EXTRACT-OUT markers that name the plan; restoring means uncommenting those regions AND reverting the consumer include rewrites, or using git - never a one-click undo.";
     ```

     appended clauses:

     ```
         assert!(
             execute.contains("wizardCopyOnly("),
             "amendment 4 (extraction-symbol-move): S5 must key its copy sentence off wizardCopyOnly like S4 - the copy sentence is false on the move branch"
         );
         assert!(
             execute.contains("MOVE_RETENTION_SENTENCE"),
             "S5's move branch must carry the marked-retention sentence"
         );
     ```

   - the new no-rot lock `test_move_sentence_states_marked_retention_truth`:
     positive `EXTRACTION_WIZARD_JS.contains(MOVE_RETENTION_SENTENCE)` (the
     value lives contiguous on the asset's ONE-line const - planner-verified
     idiom, same as the :403 pin); `wizard_function_body` carries of
     `MOVE_RETENTION_SENTENCE` in `renderWizardStepImpact`,
     `renderWizardStepDryRun`, `renderWizardStepExecute`; positive
     `contains("ARIADNE-EXTRACT-OUT")`; absences
     `!contains("strips")` (SM-I8 - the retired word, lock it out) and
     `!contains("in place for move languages")`; positive
     `contains("in place for languages that stage units in place")`.

## RGB protocol

1. Red: author the test file FIRST (const + appended clauses + new lock),
   asset untouched. Gate 2 shows EXACTLY two named failures: the amended test
   (its new `wizardCopyOnly(` clause panics - pre-hand S5 body carries no
   such call, planner-verified) and the new lock (positive sentence absent).
   Journal both panic lines. NEVER weaken an assertion to pass.
2. Green: apply the asset edits EXACTLY as mandated; re-run until green.
3. Refactor: none expected (copy + two lock sites); re-run.

## Hygiene (review-enforced)
Asset is invisible to R2 but carries the ASCII locks (:258/:804 - your bytes
are pure ASCII, no hex patterns for the token guard);
`extraction_wizard_render.rs` 890 -> ~935 (< 1000, margin noted); no new
files; no banned names. `python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- All existing wizard/card locks green UNEDITED inside gates 1/3/6: fetch
  bound `<= 2` (you add zero fetches), the S4 truncation-note lock, the P9/P10
  hotspots locks, ASCII locks, theme contract, the six surviving asserts of
  the amended test, the three A-5 S5 locks.
- `COPY_VS_MOVE_SENTENCE` bytes (asset :219 + test :331): no hunk (amendment
  policy).
- Boot payload / served bytes: `ui_smoke: PASS` (gate 4 - served asset bytes
  change). IF any byte-exact asset/served-bytes lock fires, that falsifies
  the kickoff's grep-proof: STOP and report, do not amend it (X8/R8).
- goldens/fixtures; theme tokens: zero added hex (gate 5 silent);
  `tools/**` operator-law; `crates/cli/**` in no diff.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only
for high-volume commands)
1. `python tools/testrun.py --distill -p ariadne-ui` - clean baseline before
   edits; journal the canonical line verbatim.
2. `python tools/testrun.py --distill -p ariadne-ui` - Red: exit 1, EXACTLY
   the two named failures (verbatim panic lines journaled).
3. `python tools/testrun.py --distill -p ariadne-ui` - Green: exit 0, +1
   passed vs baseline (the new lock; the amended test is count-neutral), every
   existing lock green UNEDITED.
4. `python tools/distill.py -- python tools/ui_smoke.py` - `ui_smoke: PASS`
   (the served asset changes bytes; boot + routes + no-store must hold).
5. `python tools/theme_token_check.py --check-added --base <your start rev>` -
   silent (zero added hex; combined form is the canonical gate - the bare
   form is a usage error by design).
6. `python tools/testrun.py --distill` - full workspace census; reconcile from
   your journal baseline and attribute every delta (expect +1 passed; YOUR
   landing line is the MILESTONE expectation `1853 passed / 0 failed /
   13 ignored / 119 suites`, reconciling from the anchor `1837 passed /
   0 failed / 13 ignored / 119 suites @ b201aa6` through SM-1 +2, SM-2 +10,
   SM-3 +0, SM-4 +3, SM-5 +1; attribute any foreign residue in the journal;
   the `--baseline` FILE form stays superseded by this `--distill`
   substitute).
7. `python tools/distill.py -- python tools/bench_compare.py` - fresh
   recording (kickoff sec. 5 mandates it per hand); WARNs recorded, NOT
   attributed (PR-2 persists); journal the empty
   `git diff --numstat <start rev> HEAD -- crates/cli/tests` proof (producer
   zero-diff, CW-3 gate-7 precedent).
8. `python tools/hygiene_check.py` - exit 0.
9. Commit cycle (itemized per the 2026-10-06 M28 law):
   `python tools/encoding_guard.py --staged` (or the `--all` superset if the
   staging seam refuses - journal the form used; the asset carries the
   ASCII-only lock) then `python tools/commit_scoped.py --scope
   crates/ui/assets/extraction-wizard.js --scope
   crates/ui/tests/ui_test/extraction_wizard_render.rs --scope
   tasks/extraction-symbol-move-sm5-journal.md -m "fix(ui): move-truth copy -
   marked retention sentences, keyed S5, marked-retention no-rot lock
   (extraction-symbol-move SM-5)"`. Never `git add .`; the journal rides
   inside the landing commit.

Declared gate budget (2026-10-06 M28 law): 8 scoped invocations + the commit
cycle - inside `<= ~10 + commit-cycle trio` (shape estimated 6-7; bench
recording + hygiene carried for the milestone gate list - still inside the
law, CW-3 precedent); itemize per-gate counts in the journal and declare any
overshoot as a dated D-row (silence is the deviation).

## Discipline clauses
- UTF-8: Write/Edit tools only; authored content pure ASCII (AGENTS.md
  File-Edit Discipline); both replacement texts are byte-exact from this
  handoff - a reworded sentence is a string lock, copy drift turns your own
  lock red.
- Touch only the touch-set; zero new `fetch(` occurrences (the bound is the
  invariant of record); out-of-scope breakage: report, do not fix. Any
  pre-existing lock firing on the copy edit: STOP-and-report (binding input
  6 closed the conditional as NO-OP - a firing lock falsifies the grep-proof).
- After EACH Green gate append a 1-3 line checkpoint to the journal.
- Context exhaustion looming: STOP, return a disk-state inventory; a dead
  session is never resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-symbol-move-sm5-handoff.md --scope crates/ui --cmd ...`.
- Never pin warning counts; never state a count without the command that
  produces it (M23.5 F14 standing rules).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence: the two named failures
with verbatim panic lines. 3. Green evidence: canonical lines gates 3-6 +
`ui_smoke: PASS` line + capture paths. 4. The STAYS-list as journaled. 5. Any
deviation + reason. 6. Out-of-scope observations (report-only). Never paste
raw tool output; cite command + canonical line + exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill. Extra MUSTs: diffs touch ONLY the new constant + four named copy sites
(asset) and appended test items (render file); `COPY_VS_MOVE_SENTENCE` and
`wizardCopyOnly` in no hunk; the amended test's six surviving asserts
byte-identical; the three A-5 S5 locks green UNEDITED; fetch bound green
UNEDITED; zero added hex; census +1 attributed; your landing IS the milestone
census expectation (`1853 passed / 0 failed / 13 ignored / 119 suites`) - the
closeout reconciles the full chain through you.

Sizing (planner, itemized from the mandated bodies - M23.5 rule 4): asset
~+18/-5 (constant block 4, three sentence lines 1:1, S5 keyed block +10/-1,
:575 1:1); test file ~+45 (const 1, amendment clauses ~10, lock test ~34);
journal ~14 - total ~80-110 produced lines vs the ~1200 bound; 8 gates +
commit cycle vs ~10 + trio; one module dir (`crates/ui`). No hand split: the
copy without the lock re-opens the D6-class gap the lock exists to pin; the
lock without the copy ships a Red tree - one hand, one seam.
