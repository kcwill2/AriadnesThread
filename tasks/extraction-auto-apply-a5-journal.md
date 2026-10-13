# A-5 Journal - extraction-auto-apply: S5 approval surface, S6 live outcome, help-copy flip

Implementer hop under strict RGB-TDD. Handoff:
`tasks/extraction-auto-apply-a5-handoff.md` (read end-to-end first; fully
self-contained). Shape: `tasks/extraction-auto-apply-shape.md` sections 2
(A7/A9/A10), 3 (C6/C7/C8), 4, 5 (A-5 row); kickoff sections 3 (7-8), 5
(7/10), 10 (5).

## Boot state (before any mutation)

- Duty 1 landed the A-4 verdict: `42ccff5`
  `review(extraction-auto-apply): A-4 APPROVED verdict + gate evidence`
  (387 lines, one file: `tasks/extraction-auto-apply-a4-verdict.md`;
  encoding_guard --staged clean before the commit).
- Start rev = `42ccff5`; `git status --porcelain` empty (via the distilling
  entry point, capture 0 lines). No dirt I did not create.
- Census anchor of record (workspace): `1513 passed / 0 failed / 13 ignored
  / 112 suites`.

## Gate 1 - baseline scoped census (clean tree at `42ccff5`)

Command: `python tools/testrun.py --distill -p ariadne-ui`

    327 passed / 0 failed / 0 ignored / 3 suites
    no failures

Canonical line quoted verbatim. All later per-hand reconciliation is from
this scoped baseline; the workspace anchor is the reviewer hop's to
reconcile.

## Red phase (locks first, asset untouched)

Authored in `crates/ui/tests/ui_test/extraction_card_playbook.rs` (C8
amendments 1 and 2, each with its one-line rationale comment) and
`crates/ui/tests/ui_test/extraction_wizard_render.rs` (C8 amendment 3,
the one fetch-count lock disposition D1 below, and the eight new
renderer locks). No asset or css touched before the Red run.

(Section completed after gate 2 below.)

## Gate 2 - Red run

Command: `python tools/testrun.py --distill -p ariadne-ui` (distill digest
header `### cargo test FAIL (exit 1)`, capture 414 lines). Canonical
line: `327 passed / 8 failed / 0 ignored / 2 suites` (the 3rd baseline
suite is the doc-test target, which cargo skips once the failing test
target is hit; the ui_test binary itself ran 335 tests = the 327 baseline
plus the 8 new locks, 3 of them Red-green by design: tests 3, 5, 7).

Exactly eight failures, zero in any other test file:

1. `test_apply_recipe_is_honest` - `A-5 C8 amendment 1: the reworded
   sentence must state the UI applies with a dry-run first, validation in
   three tiers and automatic rollback`
2. `test_help_playbook_copy` - `A-5 C8 amendment 2: help must state the UI
   applies with a dry-run first, validation in three tiers and automatic
   rollback`
3. `test_wizard_help_copy_names_the_execution_boundary` - `the Hotspots
   help copy must state that the UI applies with a dry-run first,
   validation in three tiers and automatic rollback (extraction-auto-apply
   C8 amendment 3)`
4. `test_execute_step_apply_control_requires_an_approval_token` -
   `renderWizardStepExecute must gate its apply control on the dry-run
   apply_token` (mandated verbatim)
5. `test_execute_step_shows_the_dry_run_first_instruction_without_a_token`
   - `the execute step must instruct the user to open the dry run first
   when no token is present` (mandated verbatim)
6. `test_after_apply_step_renders_the_live_outcome_rows` -
   `renderWizardStepAfterApply must render the outcome tiers, verdict and
   output_text` (mandated verbatim)
7. `test_apply_response_mapping_covers_every_refusal` - `the apply request
   must be one named function applyExtractionPlan (shape section 9 binding
   name)` (deviation D2)
8. `test_apply_outcome_style_block_adds_no_hex` - `styles.css must carry
   the contiguous A-5 apply/outcome block behind its marker comment`
   (deviation D2)

Failures 7-8 are required tests 6 and 8: strong Green locks cannot pass
against an asset that does not yet carry what they lock; their panics
name their own purpose. The W5 verb-count lock, the raw-interpolation ban
and the fetch-count disposition (D1) all passed at Red by construction,
as the handoff predicts.

## Green phase

Implemented in file order per the RGB protocol; every authored byte is
pure ASCII (Write/Edit tools only, no shell content writes):

1. `crates/ui/assets/index.html` - the copy flip at the two legend
   sentences (playbook sentence and wizard sentence now carry the claim of
   record verbatim: "the UI applies with a dry-run first, validation in
   three tiers and automatic rollback"), both keeping the key
   `extraction-auto-apply` as provenance; the now-false neighbour sentence
   ("Apply runs in the CLI: ...") reworded to the shared
   dry-run-then-apply path. Net line change: 0 (three lines replaced in
   place), so the asset-count, script-order, and 400-char-per-block
   readability locks are untouched.
2. `crates/ui/assets/extraction-wizard.js` - state gains `applying` and
   `outcome`, both cleared by the single reset point (which the three C7
   reset paths already call); S5 becomes the approval surface (apply
   control disabled unless `wizardState.dryRun.apply_token` is non-empty,
   disabled state carries the dry-run-first instruction, in-flight state
   disables and shows progress copy, the never-runs sentence dropped, the
   three-verb recipe and W4 sentence kept byte-identical); `applyExtractionPlan`
   is one named async function with its own fetch site POSTing
   `{"apply_token": ...}` to the plan's `/apply` route, mapping every
   refusal code to its own copy (409 disambiguated by the body's `error`:
   `stale_generation` rides the existing stale reset copy, `apply_in_progress`
   gets its mandated sentence; 428 approval refresh; 500 the refusal's
   detail; 503 reloading copy; 404 its own sentence; network the existing
   unreachable idiom; 200 stores the outcome, clears applying, advances to
   S6); S6 dispatches to `renderWizardStepOutcome` (tier rows with
   PASS/FAIL/SKIPPED plus reasons, committed-vs-rolled-back verdict,
   created files, verbatim `output_text` in a pre block, the honest C6
   watcher sentence) and keeps its descriptive copy when no outcome
   exists. Every interpolated value passes through `escapeHtml`; no new
   template-interpolates a payload object. 446 -> 606 lines (ceiling
   1000).
3. `crates/ui/assets/styles.css` - one contiguous block appended at EOF
   behind a marker comment: +26 lines net (cap ~40), existing theme tokens
   only (text-muted, amber, green, red, text-main, bg-card, border), zero
   hex. Single-property rules are disciplined one-liners to honor the
   net-cap on this over-ceiling file (its dated deferred row stands; .css
   is invisible to R2, so the cap is the discipline).

## Gate 3 - Green run

Command: `python tools/testrun.py --distill -p ariadne-ui`

    335 passed / 0 failed / 0 ignored / 3 suites
    no failures

Canonical line verbatim: baseline 327 + exactly the 8 new locks, zero
movement elsewhere, suites back at 3. All eight new tests confirmed by
name in the capture, all ok. The W5 verb-count lock, the raw-interpolation
ban, the fetch-count disposition, the 400-char readability lock and all 16
surviving card-family locks (including the 9 in `extraction_card_story.rs`)
green UNEDITED.

## Refactor

The apply fetch lives in one function; the S5 re-render is one helper
(`renderWizardExecuteStepContent`) used by the step entry and every state
flip; the tier helpers are two 4-line mappings; `renderWizardStepOutcome`
is ~30 lines. No further extraction warranted; the scoped suite re-run of
gate 3 was already over the final tree (the helper dedup shipped inside
the same Green pass), so no post-refactor regression run was needed.

## Gates 4-9

- Gate 4: `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)`, `boot+route checks clean (cold fixture,
  attempt 1, 1.0s)`, `ui_smoke: PASS`. Runtime proof the boot serves the
  flipped assets.
- Gate 5: `python tools/theme_token_check.py --check-added --base 42ccff5`
  -> silent, exit 0 (combined form; the bare form is a usage error per the
  ledger). Zero added hex over the whole hand.
- Gate 6: `python tools/hygiene_check.py` -> exit 0 (fresh via the
  distilling entry point: `### generic PASS (exit 0)`). R1/R2 zero; the
  R3 rows are the pre-existing warn-only baseline, zero rows name any
  file this hand touched.
- Gate 7: `python tools/encoding_guard.py --staged` -> exit 0 (see the
  staging note below).
- Gate 8: `python tools/commit_scoped.py --scope crates/ui/assets/extraction-wizard.js
  --scope crates/ui/assets/index.html --scope crates/ui/assets/styles.css
  --scope crates/ui/tests/ui_test/extraction_wizard_render.rs --scope
  crates/ui/tests/ui_test/extraction_card_playbook.rs --scope
  tasks/extraction-auto-apply-a5-journal.md -m "feat(ui): wizard execute
  step applies behind the approval token with live outcome reporting +
  help-copy flip"` (never `git add .`).
- Gate 9: `python tools/encoding_guard.py --all` -> exit 0 (recorded
  after the landing commit below).

Staging note: `commit_scoped.py` stages its explicit paths itself
(`git add -- <paths>` then `git commit --only`), so the pre-commit
`--staged` run checks the staged state at its invocation and the real
byte-proof for the landed content is the post-commit `--all` run. No
content was ever staged by hand and `git add .` was never invoked.

## Gate 3 checkpoint (post-commit state)

- Module: A-5 complete (S5 approval surface, S6 live outcome, apply
  request, state resets, copy flip, styles block, C8 amendments 1-3 plus
  the D1 lock disposition, 8 new renderer locks).
- Commit hash: `f89c901` (parent `42ccff5`), exactly the six touch-set
  files: `extraction-wizard.js` 168/8, `index.html` 3/3, `styles.css`
  26/0, `extraction_card_playbook.rs` 10/4, `extraction_wizard_render.rs`
  249/7, this journal.
- Deviations: D1-D6 as tabled above; none silent, all reported for the
  reviewer's frozen-locks adjudication.
- Lock finding: the exactly-one-fetch-site assertion (D1) is the only
  unlisted lock this hand had to touch; `extraction_card_story.rs`,
  `extraction_wizard_api.rs`, `crates/ui/tests/ui_test/main.rs` and
  everything under `crates/ui/src` appear in no diff of the range.
- Gate 9 (post-commit): `python tools/encoding_guard.py --all` silent
  exit 0.
- Producer zero-diff proof (bench stance, post-commit):
  `git diff --numstat 42ccff5..HEAD -- crates/cli/src crates/ui/src
  crates/transform crates/validate crates/artifact crates/cli/tests` ->
  capture 0 lines, and `git diff --name-only 42ccff5..HEAD --
  "crates/*/src"` -> capture 0 lines. The empty producer-crate diff: this
  hand's whole range is `crates/ui/assets` + two ui test lock files + the
  journal. The reviewer hop carries the DOCUMENTED_PHASES stance;
  `bench_compare.py` was not run.
- Range census for the reviewer hop (workspace-wide gates belong there):
  expected delta on the ariadne-ui scope is exactly +8 tests
  (`335 passed / 0 failed / 0 ignored / 3 suites` scoped); the workspace
  anchor `1513 passed / 0 failed / 13 ignored / 112 suites` should
  reconcile to 1521 passed at closeout (8 new tests, no other movement).

## Bench stance - producer zero-diff proof

This hand touches `crates/ui/assets` and the ui test lock files only.
`git diff --numstat 42ccff5..HEAD -- crates/*/src` and
`git diff --numstat 42ccff5..HEAD -- crates/*/tests tests` record the
producer crates byte-identical across the landing range (an empty
producer-crate diff); the deterministic DOCUMENTED_PHASES blocker at
`tools/bench_compare.py:84` stays the documented OPERATOR stance for the
reviewer hop and NO timing number is attributed to this hand.
`bench_compare.py` was NOT run (per the handoff: do not run it).

## Deviation dispositions (numbered; final table at the bottom)

1. **D1 - the one-fetch-site lock disposition (unlisted wizard-era lock
   edit; reported, not silent).** `extraction_wizard_render.rs:344-348`
   (`test_wizard_step_manifest_fetches_per_step`) asserts
   `matches("fetch(").count() == 1` over the wizard asset. The handoff's
   "What to build" item 3 mandates the apply POST ride *its own fetch
   site* ("NOT the blanket-409 fetchWizardRoute, because the apply's 409
   has two meanings"), so a second `fetch(` is required by the handoff
   itself; the C8 table never lists this lock. The lock is NOT in shape
   section 9's freeze set, not a card-family lock (R7's STOP clause names
   card-lock edits), and its file is in this hand's touch-set ("plus the
   new A7/A9 locks"). Disposition: the count assertion flips to a
   documented bound (at most two sites: the GET fetcher plus exactly one
   sanctioned apply site), the "single call site lives in
   fetchWizardRoute" assertion stays for the GET family, and a rationale
   comment names the milestone and the handoff directive. The assertion
   holds at Red (1 <= 2) and Green (2 <= 2), so the Red/Green sets stay
   as the handoff predicts. Reviewer adjudicates per the frozen-locks
   clause; nothing here was silent.
2. **D2 - Red set superset.** The handoff's Red roster names the four
   new locks (1, 2, 4) plus the three amended help-copy tests. Required
   tests 6 (apply mapping over `applyExtractionPlan`) and 8 (the styles
   block) cannot simultaneously be strong Green locks and pass against
   an asset that does not yet contain what they lock, so they fail at
   Red with their own purpose-naming panic messages. All failures stay
   inside the two touched test files (the stop-clause targets failures in
   any OTHER test file). Verbatim observed set recorded under gate 2.
3. **D3 - "show the escaped detail" for the 500 row.** The 500
   `apply_failed` detail reaches the user through the wizard error slot,
   which renders via `textContent` (never innerHTML): a stronger XSS
   posture than an `escapeHtml` wrap, and the house idiom every existing
   refusal copy already uses. The `escapeHtml` discipline of the C7
   invariant binds the S6 innerHTML path, where every value (tier name,
   status, reason, verdict, created paths, `output_text`) is
   escape-wrapped and locked by required tests 4-5.
4. **D4 - 404 copy authored; watcher-off clause.** The handoff's
   mapping list omits 404 `unknown_plan`, but acceptance 3 says every
   refusal code has its own copy, and the route answers it; the asset
   carries a dedicated sentence, locked by required test 6. Shape C6
   additionally wants the watching-disabled truth; the front end cannot
   observe the watcher setting, so the S6 watcher sentence covers both
   states ("with watching disabled the analysis refreshes at the next UI
   start") instead of claiming a rebuild happened.
5. **Report-only relay, untouched:** `hotspots.js`
   `renderExtractionApplyHint` (comment :521-522) still states "the UI
   states the command, it never runs anything". The file is outside the
   touch-set, no lock constrains the sentence; it is a stale phrase for
   the next card-touching hop. Same category: the S4 truncation note
   "the apply rewrites every consumer" (`extraction-wizard.js:379`)
   predates the D6 ledger finding (consumers are not rewritten by
   today's engine on the current fixture family; it renders only when
   the rewrite list is truncated at 200). No new copy in this hand
   claims consumer rewrites anywhere.
6. **CSS net-cap shape.** The styles block reaches ~63 lines in the
   file's house multi-line shape; the handoff caps the block at ~40 net,
   so single-property rules are written as disciplined one-liners (26
   lines net). The block stays contiguous, token-driven, zero-hex; the
   marker comment records the choice.

## Honesty relay from the A-4 review (bound on this copy)

The help flip uses the ledger's verbatim text; every sentence authored in
this hand claims exactly what the shipped path does (new files written
through the transactional applier, three-tier validation with automatic
rollback, outcome reported). No sentence authored here claims consumer
rewrites; the D6 latent defect (plan-target spelling vs on-disk include
spelling, consumers not rewritten on the current fixture family) is the
A-4 verdict's finding 7 statement for the closeout ledger row, and the
only pre-existing copy that touches the claim is report-only above.

## Landing

`f89c901` carries the implementation and this journal as committed; the
post-commit gate records (gate 9 and the zero-diff proof above) ride the
journal-only rider commit per the W-1 D5 / A-2 D2 / A-4 D7 precedent.
