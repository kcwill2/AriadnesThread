# CW-3 Handoff - extraction-consumer-rewrite: copy honesty (apply-hint comment flip, S4 note reword) + the no-rot lock

Milestone: `extraction-consumer-rewrite`. Hand 3 of 3; dispatch only after
CW-1 and CW-2 landed - your copy claims the behavior they shipped. Author rev
`a38c7aaa` (planner-verified 2026-10-23); start rev = your dispatch head,
journal it.

PLANNER AMENDMENT 2026-10-23 (read first): the mandated comment block text in
"What to build" item 1 is SUPERSEDED by the DATED AMENDMENT section at the
foot of this file (the lock spec is UNCHANGED). Red is already captured; the
hand resumes at Green.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-consumer-rewrite-kickoff.md` section 2: the stale
   apply-hint relay row (load-bearing since the A-5 help-copy flip) and the S4
   truncation-note relay row ("with the D6 fix" is this milestone).
3. `tasks/extraction-consumer-rewrite-shape.md` C6 (your contract) +
   invariants X5/X6/X8 + section 5 module map (the lock placement ruling
   below).
4. Your exact touch sites: `crates/ui/assets/hotspots.js:519-530`
   (`renderExtractionApplyHint` + its comment block) and
   `crates/ui/assets/extraction-wizard.js:385-387` (the S4 truncation note;
   it renders only when `rewrites_truncated` at 200 rows).
5. `crates/ui/tests/ui_test/extraction_wizard_render.rs` (851 lines; the file
   you append the lock to - it already imports both asset consts at :15;
   existing locks you must leave byte-identical: the fetch-site bound at
   :344-352 (bound `<= 2`, the dated wizard-era invariant row - never restore
   `count == 1`), the ASCII locks at :258-261 and :804-810).
6. Model: the house string-lock idiom already used in that file.
7. The claim of record (verbatim, from the relay row and the A-5 help-copy
   flip): "the UI applies with a dry-run first, validation in three tiers and
   automatic rollback".

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/extraction-consumer-rewrite-cw3-journal.md (create it)
- crates/ui/assets/hotspots.js (comment block :519-522 ONLY - zero code
  change: function bodies, markup bytes, class names stay byte-identical)
- crates/ui/assets/extraction-wizard.js (the one truncation-note sentence at
  :386 ONLY - COPY_VS_MOVE_SENTENCE at :219 and every other byte stay
  identical, X5)
- crates/ui/tests/ui_test/extraction_wizard_render.rs (APPEND the two lock
  tests; change no existing byte in the file)

You MUST NOT touch: `crates/ui/tests/ui_test/main.rs` (NO new test file - the
lock is APPENDED to `extraction_wizard_render.rs`, the module-map ruling that
also avoids a mod line), `styles.css` (the `.wizard-truncation-note` class
stays), `index.html`, routes/DTOs/state, `tools/**` (ui_smoke is gate 4, not a
touch), themes, other ui_test files, `crates/cli/**`, `crates/lang-cpp/**`.
No `git stash`, no branch change. Cargo.lock must not appear.

## What to build (shape C6)

1. hotspots.js comment flip. Replace comment lines :521-522 (which read
   "... the UI states the command, it never runs anything.") so the four-line
   block reads EXACTLY (ASCII; lines 519-520 stay byte-identical):

       // Honest apply recipe, verbatim verbs from the CLI command surface
       // (apply and migrate: positional repo root, --plan flag). <repo-root>
       // stays a literal placeholder token by design (shape P9); the UI applies
       // with a dry-run first, validation in three tiers and automatic rollback.

   The P9 placeholder sentence survives; the claim of record replaces the
   stale half-sentence. Zero code change: `renderExtractionApplyHint`'s body
   (P9/P10-locked markup) untouched.
2. extraction-wizard.js :386 reword. The note line becomes EXACTLY (the only
   change is the clause after "for volume; "):

       ? `<div class="wizard-truncation-note">Rewrite list truncated at ${EXTRACTION_DRY_RUN_MAX_REWRITES} rows for volume; the apply rewrites every consumer the dry run lists.</div>`

   Shipped-truth rationale for the journal: post-CW-1/CW-2 the modified set
   equals the expected-modified set which is a subset of the dry run's
   consumer rows (X1/C4) - the claim holds; rows truncated at 200 are the
   dry run's own honesty flag, so the note names its own bound.
3. The no-rot lock - APPEND two tests to
   `crates/ui/tests/ui_test/extraction_wizard_render.rs`:
   - `test_card_apply_hint_never_claims_the_ui_runs_nothing`:
     `assert!(!HOTSPOTS_JS.contains("never runs anything"), "the card hint
     must not contradict shipped apply behavior (extraction-consumer-rewrite,
     ex-stale-apply-hint relay row)")`; the same absence assert on
     `EXTRACTION_WIZARD_JS`; and the positive claim pin
     `assert!(HOTSPOTS_JS.contains("the UI applies with a dry-run first,
     validation in three tiers and automatic rollback"), "the card hint
     carries the claim of record")`.
   - `test_truncation_note_states_the_shipped_truth`:
     positive: `EXTRACTION_WIZARD_JS.contains("the apply rewrites every
     consumer the dry run lists")`. Absence:
     `!EXTRACTION_WIZARD_JS.contains("every consumer.</div>")` - pin the old
     FULL clause (`...every consumer.</div>`) as absent, NOT the prefix
     `the apply rewrites every consumer`: the new claim CONTAINS that prefix
     as a substring, so a prefix-absence assert would contradict the positive
     assert (dated planner note: this substring trap is why the lock is
     shaped this way).

## RGB protocol
1. Red: author the two lock tests FIRST. `python tools/testrun.py --distill
   -p ariadne-ui` shows EXACTLY two failures, named above, with their
   assertion messages; capture the two panic lines. (Planner-verified today:
   the only repo occurrences of "never runs anything" are
   `hotspots.js:522` and `tasks/**`/plan history, and of the truncation
   sentence only the asset - nothing existing locks either string.)
2. Green: apply the two copy edits (exact texts above); re-run until green.
   NEVER weaken an assertion to pass.
3. Refactor: none expected (copy + two string locks); re-run.

## Hygiene (review-enforced)
`extraction_wizard_render.rs` 851 -> ~890 (< 1000, no split); no new files;
no banned names; authored bytes pure ASCII (both new texts are ASCII; the
wizard asset carries an ASCII lock at :258/:804 - your note must keep it
green). `python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- Existing wizard/card locks UNEDITED: fetch-site bound `<= 2` (:344-352),
  ASCII locks, playbook `function_body(HOTSPOTS_JS,
  "renderExtractionApplyHint")` locks (P9/P10 - they read the FUNCTION BODY;
  your change sits in the comment ABOVE the declaration, a byte-region they do
  not read - that is why the flip is safe under them), graph.js line budget
  lock, static_html/standalone-export presence locks, theme contract. All run
  inside gates 3 and 6 UNEDITED.
- COPY_VS_MOVE_SENTENCE byte-unchanged (X5): reviewer diff (the sentence line
  appears in no hunk).
- Boot payload / served-bytes behavior: `ui_smoke: PASS` (gate 4); etag/export
  locks are computed, not literal-pinned - IF any byte-exact asset lock
  nonetheless fires on the copy change, that falsifies the kickoff's
  grep-proof: STOP and report, do not amend it (X8/R8).
- goldens/fixtures; `tools/**` operator-law; theme tokens: zero added hex -
  the comment contains no `#`+hex pattern.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only
for high-volume commands)
1. `python tools/testrun.py --distill -p ariadne-ui` - clean baseline before
   edits; journal the canonical line verbatim.
2. `python tools/testrun.py --distill -p ariadne-ui` - Red: exit 1, EXACTLY
   the two new locks failing (verbatim panic lines journaled).
3. `python tools/testrun.py --distill -p ariadne-ui` - Green: exit 0, +2
   passed vs baseline, every existing lock green UNEDITED.
4. `python tools/distill.py -- python tools/ui_smoke.py` - `ui_smoke: PASS`
   (the served assets change bytes; boot + routes + no-store must hold).
5. `python tools/theme_token_check.py --check-added --base <your start rev>` -
   silent (zero added hex; combined form is the canonical gate - the bare form
   is a usage error by design).
6. `python tools/testrun.py --distill` - full census; reconcile + attribute
   (expect +1 passed vs your baseline: the two new locks land on a file whose
   pre-existing count moved by 0, so passed +2 against your baseline and
   `1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b` reconciles
   through CW-1 (+4) and CW-2 (+1) to `1837 passed / 0 failed / 13 ignored /
   119 suites` at your landing if no foreign commits landed - attribute any
   residue in the journal; `--distill` substitute governs).
7. `python tools/distill.py -- python tools/bench_compare.py` - fresh
   recording (shape section 6 mandates it per hand); WARNs recorded not
   attributed (PR-2); journal the `git diff --numstat <range> --
   crates/cli/tests` EMPTY proof.
8. `python tools/hygiene_check.py` - exit 0.
9. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`
   superset, journal the form; the wizard asset carries the ASCII-only lock)
   then `python tools/commit_scoped.py --scope
   crates/ui/assets/hotspots.js --scope
   crates/ui/assets/extraction-wizard.js --scope
   crates/ui/tests/ui_test/extraction_wizard_render.rs --scope
   tasks/extraction-consumer-rewrite-cw3-journal.md -m "fix(ui): flip the
   stale apply-hint comment + S4 truncation note to the shipped truth, add the
   no-rot lock (extraction-consumer-rewrite CW-3)"`. Never `git add .`;
   journal rides inside the landing commit.

Declared gate budget (2026-10-06 M28 law): 8 scoped invocations + commit
cycle; itemize per-gate counts in the journal.

## Discipline clauses
- UTF-8: Write/Edit tools only; authored content pure ASCII; both replacements
  are byte-exact from this handoff (a reworded sentence is a string lock -
  copy drift turns your own lock red).
- Touch only the touch-set; zero new `fetch(` occurrences (the bound is the
  invariant of record); out-of-scope breakage: report, do not fix.
- After EACH Green gate: 1-3 line journal checkpoint.
- Context exhaustion: STOP, disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-consumer-rewrite-cw3-handoff.md --scope crates/ui --cmd ...`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (the two named failing
locks, verbatim panic lines). 3. Green evidence (canonical lines gates 3-6 +
`ui_smoke: PASS` line + capture paths). 4. Deviations + reasons. 5.
Out-of-scope observations (report-only). Never paste raw output.

Reviewer-gate reference: adjudicate via `review-gate`. Extra MUSTs: diffs touch
ONLY the comment block (hotspots), the ONE note line (wizard), and appended
tests (render file); COPY_VS_MOVE_SENTENCE in no hunk; fetch-site bound green
unedited; zero added hex; the closeout discharges the two relay rows against
this commit + the C6 acceptance.

Sizing (planner, itemized from the mandated bodies): comment block 4 lines
(2 replaced); note line 1; two lock tests ~36; journal ~14 - total ~55-75
produced lines vs ~1200; 8 gates + commit cycle (shape estimated 6-7; bench
recording + hygiene added for the milestone gate list - still inside the law);
one module dir (`crates/ui`).

---

## DATED AMENDMENT - planner erratum 2026-10-23 (adjudicated in-handoff,
clarity-track pattern; supersedes ONLY the build-item-1 block text)

**Defect (planner-attributed).** Build-item 1 and the lock spec are
mutually unsatisfiable as written: the mandated four-line comment WRAPS the
claim of record across `hotspots.js:521/522` (bytes between "applies" and
"with" are newline + four spaces + `// `), while the mandated pin
`HOTSPOTS_JS.contains("<full claim>")` matches bytes CONTIGUOUSLY against the
raw `include_str!` blob (`crates/ui/src/lib.rs:23`). The contiguous claim
never existed in `hotspots.js` (grep-proven at the block state; the
contiguous form lives only in `index.html:553/556`, the A-5 help copy). The
implementer's stop was correct: journal
`tasks/extraction-consumer-rewrite-cw3-journal.md` "Blocker" section is the
mechanical witness (gate 3, `370 passed / 1 failed / 0 ignored / 2 suites`,
panic at `extraction_wizard_render.rs:874:5`). My author-time verification
covered the Red-side occurrences, not this pin's Green feasibility - the F14
"unverified claim about the tree" class, planner side.

**Adjudication: option (b) - rewrap the comment; the mandated LOCK stays
byte-identical.** Chosen over (a) (pinning the contiguous tail) because (a)
is strictly weaker against the no-rot intent: a future regression that flips
only the claim's head (e.g. "the UI NEVER applies with a dry-run first, ...")
still contains the tail as a substring, so a tail pin would pass while the
absence locks (which pin only the literal "never runs anything") stay silent;
the full-contiguous-claim pin rejects any negated or paraphrased variant,
because the exact sentence must appear verbatim. And the comment's WRAPPING
is inert copy bytes owned by this hand's touch-set (the P9/P10
`function_body` locks read the function BODY, not the comment - proven inert
by the journal's gate 3), whereas the pin is the lock this hand exists to
ship: amend our copy, never weaken the lock. Option (a) would also pin a
fragment, not the claim of record that relay row ledger:325 names verbatim.

**Corrected mandated text - the four-line block (hotspots.js :519-522) now
reads EXACTLY (lines 519-520 byte-identical to the pre-hand original; pure
ASCII; zero code change):**

    // Honest apply recipe, verbatim verbs from the CLI command surface
    // (apply and migrate: positional repo root, --plan flag). <repo-root>
    // stays a literal placeholder token by design (shape P9);
    // the UI applies with a dry-run first, validation in three tiers and automatic rollback.

Diff vs the pre-hand rev stays +2/-2 (:521 and :522 are the only changed
lines). Diff vs the current blocked disk state: edit :521 to drop the
trailing " the UI applies" and re-prefix :522 with "the UI applies "; net
file length unchanged (691 lines).

**The pin itself is UNCHANGED from the original mandate, verbatim:**
`assert!(HOTSPOTS_JS.contains("the UI applies with a dry-run first, validation in three tiers and automatic rollback"), "the card hint carries the claim of record")`
- scans: the `HOTSPOTS_JS` raw asset blob; the contiguous occurrence is
  `crates/ui/assets/hotspots.js:522` (the comment line above; the claim TEXT
  is byte-identical to the shipped A-5 help copy at `index.html:553/556`,
  the comment's trailing period sits after the pinned string and does not
  affect `contains`).
- Absence locks UNCHANGED (substring-trap ruling intact):
  `!HOTSPOTS_JS.contains("never runs anything")`,
  `!EXTRACTION_WIZARD_JS.contains("never runs anything")`,
  `!EXTRACTION_WIZARD_JS.contains("every consumer.</div>")`.

**Red status: ALREADY CAPTURED, no re-capture.** Gate 2's
`369 passed / 2 failed / 0 ignored / 2 suites` with the two named locks
stands as this hand's Red: under the ORIGINAL comment the amended pin's
contiguous claim was absent too (grep-witness above), so the positive pin
would have failed at Red exactly as captured. The implementer RESUMES AT
GREEN: apply the corrected block, then re-run gate 3 expecting `371 passed /
0 failed / 0 ignored / 3 suites` (baseline 369 + the 2 locks), then gates
4-9 as written.

**Landing procedure (additions, everything else in this handoff unchanged):**
1. BEFORE the landing commit, land this amended handoff as a small docs
   commit: `python tools/commit_scoped.py --scope
   tasks/extraction-consumer-rewrite-cw3-handoff.md -m "docs(cw-3): planner
   erratum - comment rewrap so the claim-of-record pin is satisfiable
   (extraction-consumer-rewrite CW-3)"` (journal declares the extra commit
   cycle per the 2026-10-06 M28 itemization law).
2. Journal edits (yours, in your touch-set): the "Copy edits applied" item 1
   and the closeout-discharge line for ledger:325 must cite the AMENDED
   four-line block (this section) as their byte-exact source; the
   ledger:326 discharge line stands unchanged.
3. Everything else in the handoff - touch-set, S4 note line, lock tests,
   hygiene, frozen locks, gates 1-9, discipline, report-back - unchanged.
