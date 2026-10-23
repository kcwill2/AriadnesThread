# CW-3 Journal - extraction-consumer-rewrite

Status: **RESOLVED and RESUMED - GREEN, landing in progress.** The hand
was originally BLOCKED at its own Green gate (one mandated assertion
was unsatisfiable with the mandated comment copy; the "Blocker"
section below stays as the preserved mechanical witness). The planner
adjudicated in-handoff: option (b) rewrap, recorded as the DATED
AMENDMENT at the foot of `tasks/extraction-consumer-rewrite-cw3-handoff.md`.
The hand resumed per that erratum's docs-first procedure; see "Erratum
resume (2026-10-23)" below.

## Contract and bootstrap

- Handoff: `tasks/extraction-consumer-rewrite-cw3-handoff.md` (author
  rev `a38c7aaa`). Start rev of record: `d3f6dbc` (dispatch head).
- Boot per `resume-checkpoint`: `python tools/checkpoint_guard.py status`
  reported HEAD `d3f6dbc` on `main`; dirty = the tolerated
  `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix` plus the
  two CW-2 reviewer artifacts (`cw2-verdict.md` untracked,
  `execution-results.md` modified +73/-3). Those two ARE the duty-1
  commit material named by the CW-2 verdict's duty-1 block, so this is
  the expected boot state, not a foreign-dirt STOP (see deviation 5).
- Boot note: the direct `git log -1` form was refused with the
  boilerplate catch-all deny
  (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`);
  `checkpoint_guard` supplied the ground truth (record, not deviation).

## Duty 1 (before any Red) - CW-2 reviewer artifacts landed

Both commands run verbatim, in order:

- `python tools/encoding_guard.py --all` -> silent, exit 0.
- `python tools/commit_scoped.py --scope
  tasks/extraction-consumer-rewrite-cw2-verdict.md --scope
  tasks/extraction-consumer-rewrite-execution-results.md -m "docs(cw-2):
  reviewer verdict APPROVED + execution-results addendum
  (extraction-consumer-rewrite CW-2)"` -> landed **`701d7d7`**.
- Stat: verdict +342/-0, execution-results +73/-3 - exactly the two
  declared paths. No `git add .`; no amend of
  `ece3513`/`fddb200`/`d3f6dbc`. CW-3 builds on `701d7d7` as its start
  rev (the "d3f6dbc-plus-duty-1" the CW-2 verdict prescribes).

## Gate table (itemized; distilling entry points; timeout 600000 ms)

| # | Gate | Result |
|---|------|--------|
| 1 | `python tools/testrun.py --distill -p ariadne-ui` (clean baseline) | exit 0, `369 passed / 0 failed / 0 ignored / 3 suites` (capture `target/testrun-capture.txt`, 407 lines) |
| 2 | same command (Red) | exit 1, `369 passed / 2 failed / 0 ignored / 2 suites` - EXACTLY the two named locks; panic lines verbatim below |
| 3 | same command (Green attempt after the byte-exact copy edits) | exit 1, `370 passed / 1 failed / 0 ignored / 2 suites` - the truncation lock is GREEN; the card-hint lock advances to :874:5 and fails on the positive pin (Blocker) |
| 4 | `ui_smoke` via distill | NOT RUN - landing gate behind the Blocker (a red scoped suite makes it unattributable) |
| 5 | `python tools/theme_token_check.py --check-added --base 701d7d7` | silent, exit 0 (zero added hex; run as a disk-state fact) |
| 6 | full census `testrun --distill` | NOT RUN - landing gate behind the Blocker |
| 7 | `bench_compare` via distill + `crates/cli/tests` zero-diff proof | NOT RUN - landing gate; the whole worktree delta vs `701d7d7` is the three touch-set files + this journal + the tolerated vsix, so the producer zero-diff fact holds trivially |
| 8 | `python tools/hygiene_check.py` | R1/R2 blocking SILENT; every printed row is a warn-only R3 row in files this hand did not touch; zero rows in the three touched files |
| 9 | commit cycle (`encoding_guard --staged`/`--all` + `commit_scoped`) | NOT RUN at the blocked phase (no landing under a Blocker); `encoding_guard --all` was run fresh on the current disk state: silent, exit 0. RESUMED: the commit cycle ran on the amended state (Erratum resume + Landing record below) |

Routed invocation tally: 3 testrun (gates 1-3) + 2 `encoding_guard`
(duty-1 + disk state) + 1 `theme_token_check` + 1 `hygiene_check` + 1
distill-routed `git diff --numstat` + 1 `commit_scoped` (duty-1) = 9
total, with no gate retried; the blocked CW-3 landing commit was
correctly not spent.

## RGB evidence

Red (gate 2; the stale texts present, locks authored, nothing else):

- canonical: `369 passed / 2 failed / 0 ignored / 2 suites`, exit 1.
- panic 1 (verbatim): `extraction_wizard_render::test_card_apply_hint_never_claims_the_ui_runs_nothing`
  panicked at `crates\ui\tests\ui_test\extraction_wizard_render.rs:866:5`:
  `the card hint must not contradict shipped apply behavior (extraction-consumer-rewrite, ex-stale-apply-hint relay row)`
- panic 2 (verbatim): `extraction_wizard_render::test_truncation_note_states_the_shipped_truth`
  panicked at `crates\ui\tests\ui_test\extraction_wizard_render.rs:882:5`:
  the S4 truncation note must state the shipped truth
  (extraction-consumer-rewrite, ex-S4 truncation-note relay row): the
  apply rewrites every consumer the dry run lists

Green attempt (gate 3, after BOTH mandated copy edits, byte-exact):

- canonical: `370 passed / 1 failed / 0 ignored / 2 suites`, exit 1
  (capture `target/testrun-capture.txt`, 415 lines).
- `test_truncation_note_states_the_shipped_truth`: **GREEN** - positive
  claim and full-clause absence both pass.
- `test_card_apply_hint_never_claims_the_ui_runs_nothing`: both absence
  asserts GREEN (the test moved from :866 to :874); it panics at
  `:874:5` with the handoff's own message:
  `the card hint carries the claim of record`
- Every pre-existing lock green unedited, including the fetch-site
  bound `<= 2` (:344-352), the ASCII locks (:258-261, :804-810), and
  the P9/P10 `function_body(HOTSPOTS_JS, "renderExtractionApplyHint")`
  locks - exactly as the handoff predicted for a comment-region edit.

Refactor: not reached (copy + string locks; the handoff itself said
"none expected").

## Copy edits applied on disk (byte-exact from the handoff)

1. `crates/ui/assets/hotspots.js` :521-522 - the four-line block now
   reads exactly as mandated by the AMENDED handoff (DATED AMENDMENT,
   option b: the rewrap that keeps the claim of record contiguous at
   :522; lines 519-520 byte-identical to the pre-hand original; zero
   code change; `renderExtractionApplyHint`'s P9/P10-locked body
   untouched). Diff vs the pre-hand rev +2/-2; file stays 691 lines.
   The AMENDED block is the byte-exact source of record for this flip:

   ```
   // Honest apply recipe, verbatim verbs from the CLI command surface
   // (apply and migrate: positional repo root, --plan flag). <repo-root>
   // stays a literal placeholder token by design (shape P9);
   // the UI applies with a dry-run first, validation in three tiers and automatic rollback.
   ```
2. `crates/ui/assets/extraction-wizard.js` :386 - the note line is now
   exactly `... rows for volume; the apply rewrites every consumer the
   dry run lists.</div>`. Diff +1/-1; `COPY_VS_MOVE_SENTENCE` (:219)
   appears in no hunk (X5).
3. `crates/ui/tests/ui_test/extraction_wizard_render.rs` - pure
   APPEND +39/-0 (zero deleted lines: every pre-existing line is
   byte-identical by the numstat itself). File 851 -> 890 lines
   (< 1000). No new files, no `main.rs` mod line (the module-map
   ruling). Both new test names exactly as mandated; assertion messages
   exactly as mandated; pure ASCII throughout (hygiene and
   encoding-guard clean above).

Produced diff vs `701d7d7`: +42/-3 across the three code files
(2+1+39 insertions), consistent with the ~55-75 sizing band minus the
~14 journal lines this file carries.

## Blocker (report, do not fix)

The handoff's build-item 1 and its lock spec are mutually
unsatisfiable:

- The four-line comment block must read EXACTLY as given (discipline
  clause: "both replacements are byte-exact from this handoff"). That
  block WRAPS the claim of record across the two new comment lines:
  line 521 ends `... the UI applies` and line 522 is
  `    // with a dry-run first, validation in three tiers and automatic rollback.`
  The bytes between `applies` and `with` are newline + four spaces +
  `// `, not a space.
- The lock must carry verbatim:
  `assert!(HOTSPOTS_JS.contains("the UI applies with a dry-run first, validation in three tiers and automatic rollback"), "the card hint carries the claim of record")`.
  `HOTSPOTS_JS` is the raw `include_str!` blob of `hotspots.js`
  (`crates/ui/src/lib.rs:23`), and Rust `str::contains` matches bytes
  contiguously, so the pin can only pass if the claim sits
  contiguously somewhere in the asset.
- It does not: grep over `crates/ui/assets` for `the UI applies`,
  `dry-run first`, `three tiers`, and `automatic rollback` finds the
  claim only in `index.html` (:553/:556, the A-5 help copy) and near-
  paraphrases in the wizard; `hotspots.js` holds none of them outside
  the wrapped comment. (The planner's verification covered the
  Red-side occurrences, not the Green feasibility of this pin - the
  same dispatch-note staleness class as the CW-2 verdict's finding 2.)
- Mechanical witness: gate 3 above, run with the mandated copy in
  place - the assertion itself panics at :874:5 with its own message.
- Green is reachable only by weakening the assert (forbidden: "NEVER
  weaken an assertion to pass"), drifting the copy off its byte-exact
  mandate (forbidden, and X8's closed list), or touching outside the
  touch-set (forbidden). So this hand stops here.
- Everything else in the hand is sound: both absence locks green with
  the copy as mandated, the whole S4 relay row green including the
  substring-trap shape, and the positive pin's claim TEXT is the
  correct claim of record - only its host string cannot carry it
  contiguously under the mandated wrap.
- Amendment candidates for the planner/reviewer (deliberately NOT
  applied here): (a) reword the pin to the contiguous tail the block
  really carries, e.g. pin
  `HOTSPOTS_JS.contains("with a dry-run first, validation in three tiers and automatic rollback")`;
  or (b) rewrap comment lines 521-522 so the full claim sits on one
  comment line (a comment-only change, but it changes the handoff's
  four-line block text, so it must come from a planner erratum, not
  from here). Either is one line; the choice belongs to the handoff's
  owner.
- **RESOLVED 2026-10-23:** the planner adopted candidate (b) as the
  DATED AMENDMENT in the handoff (rationale: a tail-pin is strictly
  weaker against the no-rot intent; amend our copy, never weaken the
  lock). See "Erratum resume (2026-10-23)". This section stands
  unchanged as the mechanical witness.

## Erratum resume (2026-10-23) - GREEN + commit cycle

The planner adjudicated the Blocker in-handoff: the handoff's DATED
AMENDMENT section chose option (b) - rewrap the comment, keep the
mandated lock byte-identical - superseding ONLY the build-item-1 block
text (rationale there: a tail pin is strictly weaker against the
no-rot intent; the comment's wrapping is inert copy bytes inside this
hand's touch-set, while the pin is the lock this hand exists to ship;
F14 class, planner side). Red was NOT re-captured per the erratum:
under the original comment the amended pin's contiguous claim was
absent too, so gate 2 stands as this hand's Red.

Docs-first per the erratum's landing procedure:

- `python tools/encoding_guard.py --all` -> silent, exit 0.
- `python tools/commit_scoped.py --scope
  tasks/extraction-consumer-rewrite-cw3-handoff.md -m "docs(cw-3):
  planner erratum - comment rewrap so the claim-of-record pin is
  satisfiable (extraction-consumer-rewrite CW-3)"` -> **`90beaef`**
  (+88/-0, exactly the handoff path). The extra docs commit cycle is
  declared here per the 2026-10-06 M28 itemization law.

Rewrap edit (`hotspots.js` :521-522 only, Edit tool, pure ASCII):
:521 dropped the trailing " the UI applies"; :522 re-prefixed with
"the UI applies ", so the full claim sits contiguous at :522. Lines
519-520 stay byte-identical to the pre-hand original; the file stays
691 lines; net diff vs the pre-hand rev stays +2/-2. The block was
verified byte-for-byte against the erratum's quoted text by direct
read before the gate run.

Gate 3 Green on the amended state:

- canonical: `371 passed / 0 failed / 0 ignored / 3 suites`, exit 0
  (capture `target/testrun-capture.txt`, 409 lines) - the erratum's
  mechanical expectation EXACTLY (baseline 369 + the 2 locks).
- `test_card_apply_hint_never_claims_the_ui_runs_nothing`: all three
  asserts green (both absence locks + the full-claim pin).
- `test_truncation_note_states_the_shipped_truth`: green (unchanged
  from the blocked phase).
- Every pre-existing lock green UNEDITED: fetch-site bound `<= 2`
  (:344-352), the ASCII locks, the P9/P10 `function_body` locks, the
  theme contract, the static_html/standalone-export presence locks.

Scoped gates on the amended state (exit codes recorded):

- gate 5 `theme_token_check --check-added --base 701d7d7`: silent,
  exit 0 (zero added hex).
- gate 8 `hygiene_check`: R1/R2 blocking SILENT; every printed row is
  a warn-only R3 row in files this hand does not touch.
- commit-cycle `encoding_guard --all` (superset form of `--staged`,
  form journalled per gate 9's clause): silent, exit 0.
- NOT RUN by this hop per dispatch ruling (reviewer hop owns
  workspace-wide gates): gate 4 `ui_smoke`, gate 6 full census, gate 7
  `bench_compare`.

Producer-side zero-diff facts the reviewer hop needs: recorded
verbatim in the Landing record (journal rider), covering
`crates/cli/tests`, `crates/lang-cpp`, and the characterization files.

## Deliberate not-dos

- No weakened assertion, no copy drift, no self-amendment of the
  handoff, no improvisation past the pin.
- No CW-3 commit: gate 9's landing assumes a green scoped suite.
- Untouched and diff-empty: `crates/ui/tests/ui_test/main.rs`,
  `styles.css`, `index.html`, routes/DTOs/state, `tools/**`,
  `crates/cli/**`, `crates/lang-cpp/**`, themes; `Cargo.lock` appears
  in no diff; zero new `fetch(` occurrences (bound still holds).
- No `git stash`, no branch change.

## Disk-state inventory (pre-landing state after the erratum resume; the landing rides the commit cycle below)

- `M crates/ui/assets/hotspots.js` (+2/-2; the :521-522 comment block
  only - AMENDED rewrap: the full claim sits contiguous at :522, file
  unchanged at 691 lines)
- `M crates/ui/assets/extraction-wizard.js` (+1/-1; the :386 note line only)
- `M crates/ui/tests/ui_test/extraction_wizard_render.rs` (+39/-0 append; 890 lines)
- `?? tasks/extraction-consumer-rewrite-cw3-journal.md` (this file)
- `M extensions/ariadne-extension/ariadne-extension-0.1.0.vsix` (tolerated pre-existing stray, untouched)
- Reproduction history: the blocked witness state ran `370 passed / 1
  failed / 0 ignored / 2 suites` with the single failure at
  `extraction_wizard_render.rs:874:5`. RESOLVED by the erratum: the
  amended tree runs `371 passed / 0 failed / 0 ignored / 3 suites`
  exit 0 (= baseline 369 + the 2 locks); the reviewer hop's gates
  4/6/7 then proceed unchanged.
- Landed before this commit cycle: docs commit **`90beaef`** (the
  amended handoff, +88/-0, exactly the handoff path) per the
  erratum's docs-first procedure.

## Closeout discharge lines (ledger:325 + ledger:326)

Ready for the closeout hop; both cite the CW-3 landing commit (SHA in
the Landing record section, appended by the journal rider). The
planner confirmed these discharges stand, with the AMENDED block as
the byte-exact source for ledger:325 and ledger:326 unchanged:

- **Ledger:325 stale apply-hint relay**: DISCHARGED by CW-3 -
  `hotspots.js:521-522` flipped to the claim of record, byte-exact
  from the AMENDED four-line block (DATED AMENDMENT option b: lines
  519-520 byte-identical to the pre-hand original, the full claim
  contiguous at :522, zero code change). The stale text is gone and
  pinned gone on both assets; the positive pin
  `HOTSPOTS_JS.contains("the UI applies with a dry-run first,
  validation in three tiers and automatic rollback")` is green
  against the contiguous occurrence at :522 - exactly the claim the
  ledger row quotes, verbatim.
- **Ledger:326 S4 truncation-note relay**: DISCHARGED by CW-3 - the
  note now states the shipped truth (`the apply rewrites every
  consumer   the dry run lists`), truthful post-CW-1/CW-2 because the
  expected-modified oracle (CW-2, shipped) makes the modified set a
  subset of the dry run's consumer rows, and rows truncated at 200 are
  the dry run's own honesty flag; the no-rot lock
  `test_truncation_note_states_the_shipped_truth` pins positive and
  full-clause-absent per the substring-trap ruling.
- **Ledger:327 fetch-site invariant (wizard-era row)**: PRESERVED, NOT
  discharged (dispatch ruling) - this hand adds zero fetch sites, and
  the bound lock `<= 2` at `extraction_wizard_render.rs:344-352` ran
  green UNEDITED in every phase of every gate run (Red, blocked
  Green-attempt, final Green).

## Deviations

1. No landing commit - stopped under the Blocker per the handoff's
   report-don't-fix discipline and the implementer role rule (report
   and stop, do not improvise a fix). Full evidence above.
   (2026-10-23 resume: SUPERSEDED - the planner's erratum adjudicated
   the blocker; the landing follows in the Erratum resume and Landing
   record. The stop itself was ratified by the erratum: "The
   implementer's stop was correct.")
2. Gates 4/6/7 not run - landing gates behind the Blocker; CW-1/CW-2
   precedent already moves workspace-wide gates to the reviewer hop.
   (2026-10-23 resume: STANDS for a different reason - the dispatch
   assigns workspace-wide gates to the reviewer hop outright; the
   producer zero-diff facts it needs are recorded in the Landing
   record.)
3. Refactor step not reached - predicted "none expected"; blocked
   upstream of it.
4. Boot refusal record - direct `git log -1` hit the boilerplate
   catch-all deny; `checkpoint_guard` is the sanctioned ground-truth
   path (record, not a deviation).
5. Dispatch-wording note - the dispatch's "tolerated pre-existing
   dirty: vsix only" predates the duty-1 posture: the two CW-2
   reviewer artifacts were on disk BY DESIGN as duty-1's inputs (CW-2
   verdict's duty-1 block); read as "no foreign dirt" and executed.
