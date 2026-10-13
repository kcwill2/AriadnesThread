# A-5 Handoff - extraction-auto-apply: S5 approval surface, S6 live outcome, the help-copy flip, C8 amendments

Depends on A-3 AND A-4 landed (the execute step needs the live route and a
real applier behind it).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-auto-apply-shape.md` sections 2 (A7 and its oracle, A9
   and A10), 3 (C6 post-apply rebuild, C7 front-end steps verbatim, C8 lock
   amendment table verbatim), 4 (scope boundary), 5 (module map, A-5 row).
   `tasks/extraction-auto-apply-kickoff.md` sections 3 (items 7-8), 5
   (items 7 and 10), 10 (item 5).
3. `crates/ui/assets/extraction-wizard.js` (all 446 lines): the wizard state
   (:19-26), `resetWizardState` (:40-47), `showWizardStep` (:64-78),
   `fetchWizardRoute` (:222-244, note the blanket 409 reset), 
   `enterWizardStep` (:263-286), `renderWizardStepExecute` (:397-405),
   `renderWizardStepAfterApply` (:409-416), the payload-swap reset hook
   (:424-446).
4. `crates/ui/assets/index.html` :548 and :551 (the two sentences to flip)
   and :543-552 (their neighbours).
5. `crates/ui/tests/ui_test/extraction_wizard_render.rs` :543-581 (the W5
   verb-count lock), :585-609 (the execution-boundary lock and the
   raw-interpolation ban), :20-23 (the `include_str!` constants).
6. `crates/ui/tests/ui_test/extraction_card_playbook.rs` :190-222
   (`test_apply_recipe_is_honest`) and :295-319 (`test_help_playbook_copy`).

## Objective and preserved invariants

Turn the wizard's Execute step (S5) into the approval surface and its
After-apply step (S6) into live outcome reporting, and flip the help copy
from "the UI never applies" to the claim of record: "the UI applies with a
dry-run first, validation in three tiers and automatic rollback".

Invariants this hand must preserve:

- **A7 approval-surface honesty.** The apply control is enabled ONLY when
  `wizardState.dryRun.apply_token` is non-empty. Without one the step
  renders the dry-run-first instruction and no apply request is possible.
  Locked by a renderer assertion on the enable condition.
- **W5 verb-count lock stays satisfiable.** `renderWizardStepExecute` keeps
  the three-verb recipe in its current order and introduces NO new
  `tool ` string anywhere in the asset: the lock is
  `named_verbs == honest_verbs` over the whole file, so a phrase like "the
  CLI tool" detonates it. Every new sentence avoids the word "tool".
- **Escape discipline.** Every interpolated value - the tier detail, the
  verdict, the created-file paths, the verbatim `output_text`, the refusal
  detail - passes through `escapeHtml`. The raw-interpolation ban at
  `extraction_wizard_render.rs:596-608` keeps scanning the asset; keep its
  forbidden list satisfiable and do not add raw `${outcome.` /
  `${applying` interpolations.
- **C7 state discipline.** The wizard state gains `applying` and `outcome`
  and every existing reset path clears both (different plan, 409 stale
  generation, payload swap): an outcome belongs to the generation that
  produced it.
- **C6 honesty.** No rebuild claim the client cannot verify: the S6 copy
  states that the background watcher re-analyzes and the wizard resets when
  the new analysis lands, and that the outcome record also lands under
  `artifacts/transaction`.
- **A8/A9.** No bind widening (this hand touches no route or handler) and
  every rendered string is pure ASCII.
- **A10 lock survival.** Exactly the C8 amendments below; 16 of the 18
  card-family locks survive byte-identical and `extraction_wizard_api.rs`
  stays entirely UNEDITED.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/extraction-auto-apply-a5-journal.md (checkpoint journal)
- crates/ui/assets/extraction-wizard.js (S5 approval surface, S6 outcome,
  the two state fields and their resets)
- crates/ui/assets/index.html (the copy flip at :548 and :551, plus the
  neighbouring sentence that becomes false - :549)
- crates/ui/assets/styles.css (ONE contiguous apply/outcome block,
  net-capped, existing theme tokens only, zero added hex)
- crates/ui/tests/ui_test/extraction_wizard_render.rs (C8 amendment 3 plus
  the new A7/A9 locks)
- crates/ui/tests/ui_test/extraction_card_playbook.rs (C8 amendments 1 and
  2, each with a rationale comment in its diff)

You MUST NOT touch: `crates/ui/src/**` (routes, DTOs, state, server, lib),
`crates/ui/tests/ui_test/main.rs` (this hand adds NO test module),
`crates/ui/tests/ui_test/extraction_wizard_api.rs`,
`crates/ui/tests/ui_test/extraction_card_story.rs`, every other ui test
file, `crates/ui/assets/hotspots.js`, `crates/ui/assets/main.js`,
`crates/ui/assets/api.js`, `crates/ui/assets/preview.js`,
`crates/ui/assets/graph.js`, `crates/ui/assets/traversal.js`,
`crates/ui/assets/file-view.js`, `crates/ui/assets/provenance.js`,
`crates/ui/assets/themes.css`, `crates/ui/assets/theme.js`,
`tools/theme_token_baseline.json`, `tools/**`, `crates/cli/**`, `tests/**`,
`Cargo.toml`, `Cargo.lock`. No `git stash`, no branch change.

## Frozen locks

- The C8 amendment table is the COMPLETE set of card-family and wizard-era
  edits. Any further card-lock edit discovered mid-hand is a
  STOP-and-report (shape risk R7), never a silent fix.
- 16 of the 18 card-family locks survive byte-identical, including all 9 in
  `extraction_card_story.rs` (that file is not in this touch-set at all).
- `extraction_wizard_api.rs` stays UNEDITED (its GET-only and no-write
  locks, and its `sample_dry_run_dto`, which A-3 already amended by exactly
  one line).
- `tools/theme_token_baseline.json` byte-locked; zero added hex proven by
  the combined `--check-added --base <start-rev>` gate.
- `tests/golden/**` and `tests/fixtures/**` frozen; the boot payload bytes
  unchanged (this hand adds no payload field).

### The C8 amendments (exact before/after; the reviewer diffs each hunk)

| # | Test | Site | Before | After | Why no keep-literal path |
|---|---|---|---|---|---|
| 1 | `extraction_card_playbook::test_apply_recipe_is_honest` | `:214-217` | `legend.contains("never applies")` with the message `C7: the reworded sentence must still state that the UI never applies anything` | assert the flipped claim instead: the legend contains the new sentence's key phrase (the copy of record is "the UI applies with a dry-run first, validation in three tiers and automatic rollback", so the lock asserts a stable substring of it and keeps the `extraction-auto-apply` and verb-count assertions in the same test) | mechanically opposed to the mandated copy flip: the sentence this lock pins is the one the milestone deletes |
| 2 | `extraction_card_playbook::test_help_playbook_copy` | `:303-306` | `legend.contains("never applies")` with the message `C7: help must state the UI never applies anything itself` | the same flipped claim, keeping the "three tiers", "rollback", "collapsed summary", "Proposed extractions section" and "Preview after extraction" assertions untouched | same reason; the flip is the milestone's headline behavior |
| 3 | `extraction_wizard_render::test_wizard_help_copy_names_the_execution_boundary` | `:588-591` | `legend.contains("never mutates the repository")` with the message `the Hotspots help copy must state that the UI never mutates the repository in this version` | the flipped claim, keeping the `extraction-auto-apply` assertion at :592-595 byte-identical | same reason; the `extraction-auto-apply` key must survive as provenance |

Each amendment carries a one-line rationale comment naming this milestone
and the C8 row. No other assertion in those three tests moves.

## What to build

1. **The copy flip** (`index.html`): replace the "never applies" / "never
   mutates the repository" clauses with the claim of record while KEEPING
   the milestone key `extraction-auto-apply` as provenance (two key-presence
   assertions depend on it) and keeping the "three tiers", "rollback",
   "collapsed summary", "Proposed extractions section" and "Preview after
   extraction" phrases. Reword the now-false neighbouring sentence at :549
   ("Apply runs in the CLI: ...") so it describes the shared
   dry-run-then-apply path instead. No new script tag, no new element, so
   the asset-count and script-order locks are untouched.
2. **S5 `renderWizardStepExecute`** (`extraction-wizard.js`): keep the
   three-verb recipe and the W4 sentence verbatim, drop the sentence "The
   UI shows this recipe and never runs it", and add:
   - an apply `<button>` whose `disabled` state is driven by whether
     `wizardState.dryRun.apply_token` is non-empty (A7); the disabled state
     carries the dry-run-first instruction;
   - an in-flight state that disables the control and shows progress copy
     while the request is open;
   - the response mapping below.
3. **The apply request** `applyExtractionPlan()`: a POST to
   `/api/extraction/<plan id>/apply` with the JSON body
   `{"apply_token": <the dry-run's token>}`, through its own fetch site (NOT
   the blanket-409 `fetchWizardRoute`, because the apply's 409 has two
   meanings that must be told apart):
   - 200: store `wizardState.outcome`, clear `applying`, render S6;
   - 409 whose body's `error` is `stale_generation`: ride the EXISTING
     stale-generation reset (the same copy and reset the GET path uses);
   - 409 whose body's `error` is `apply_in_progress`: show "another apply is
     running; try again in a moment";
   - 428 `dry_run_required`: show "re-open the dry run to refresh the
     approval";
   - 500 `apply_failed`: show the escaped `detail`;
   - 503 `analysis_loading`: show a copy stating the analysis is reloading;
   - a network failure: the existing unreachable-copy idiom.
4. **S6 `renderWizardStepAfterApply`**: when `wizardState.outcome` exists,
   render the three tier rows (tier name, PASS/FAIL/SKIPPED plus the reason
   for a fail or skip), the committed-vs-rolled-back verdict line, the
   created files, and the verbatim `output_text`, all through `escapeHtml`.
   With no outcome the step keeps its current descriptive copy, so it stays
   enterable before an apply.
5. **State**: add `applying` and `outcome` to `wizardState`, initialize
   both in `resetWizardState` (the single reset point), and clear them on
   every existing reset path.
6. **`styles.css`**: one contiguous block for the apply control, its
   disabled state, the in-flight state and the outcome rows, using existing
   theme tokens only. `.css` is invisible to R2, so the cap is the
   discipline: net-capped, no more than ~40 added lines.
7. **The renderer locks** in `extraction_wizard_render.rs`: the new A7 and
   A9 tests below, plus C8 amendment 3.

## RGB protocol (per module, in this file order)

Red: apply the C8 amendments and author the new renderer locks FIRST, then
run gate 2 BEFORE touching the asset. Expected failures, verbatim shape:

- `test_execute_step_apply_control_requires_an_approval_token`:
  `renderWizardStepExecute must gate its apply control on the dry-run apply_token`
- `test_execute_step_shows_the_dry_run_first_instruction_without_a_token`:
  `the execute step must instruct the user to open the dry run first when no token is present`
- `test_after_apply_step_renders_the_live_outcome_rows`:
  `renderWizardStepAfterApply must render the outcome tiers, verdict and output_text`
- the three C8 amendments fail on the un-flipped `index.html`:
  `the Hotspots help copy must state ...` (the amended assertions' own
  messages)
- `test_wizard_help_copy_names_the_execution_boundary` fails with its
  amended message

The W5 verb-count lock and the raw-interpolation ban pass at Red by
construction - the asset is untouched yet. A Red run that shows a failure in
any OTHER test file means the touch-set was violated: stop and report.

Green: implement the asset changes and re-run until exit 0.

Refactor: keep each renderer small; the apply fetch lives in one function;
`extraction-wizard.js` stays well under the line ceiling.

## Required tests (added to `crates/ui/tests/ui_test/extraction_wizard_render.rs`)

1. `test_execute_step_apply_control_requires_an_approval_token` (A7): the
   body of `renderWizardStepExecute` reads `wizardState.dryRun.apply_token`
   (or the equivalent local) and the apply control's enabled state is keyed
   off it.
2. `test_execute_step_shows_the_dry_run_first_instruction_without_a_token`
   (A7): the execute step's copy contains the dry-run-first instruction
   when the token is absent.
3. `test_execute_step_keeps_the_three_verb_recipe` (W5 survival after the
   edit): the three verbs, their order and the `escapeHtml(planId || '')`
   interpolation survive - a re-assertion so the S5 edit cannot silently
   break the W5 lock.
4. `test_after_apply_step_renders_the_live_outcome_rows` (A5/S6): the body
   of `renderWizardStepAfterApply` renders tier rows keyed off the outcome,
   the committed/rolled-back verdict, the created files and
   `output_text`, and keeps the descriptive copy when no outcome exists.
5. `test_every_outcome_interpolation_is_escaped` (A9/A11): the outcome
   renderer interpolates no value without `escapeHtml`, and the asset adds
   no raw `${outcome.` / `${applying` form.
6. `test_apply_response_mapping_covers_every_refusal` (the C7 mapping):
   the asset names each refusal code's copy - the 428 approval refresh, the
   `apply_in_progress` sentence, the 500 detail, the 503 reloading copy -
   and the 200 path stores the outcome and advances to S6.
7. Lock: the asset stays ASCII-only (no byte above 0x7F) - the existing
   ASCII lock idiom in this file, extended to the new code paths.
8. Lock: `styles.css`'s new block introduces no hex literal (zero added hex,
   the same assertion style the theme locks use).

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names, no
single-letter locals, intention-revealing names (shape section 9's names are
binding: `renderWizardStepOutcome`, `applyExtractionPlan`). Measured at
HEAD `023558c`: `extraction-wizard.js` 446 -> ~560; `index.html` 640 ->
~645; `extraction_wizard_render.rs` 609 -> ~700;
`extraction_card_playbook.rs` 319 -> ~325; `styles.css` is over the
1000-line ceiling by an existing dated deferred row and this hand's block is
net-capped at ~40 lines. `python tools/hygiene_check.py` exit 0.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-ui` (baseline canonical
   line recorded verbatim in the journal)
2. `python tools/testrun.py --distill -p ariadne-ui` (Red: exit 1; exactly
   the named failing tests, verbatim panic lines, in the journal)
3. `python tools/testrun.py --distill -p ariadne-ui` (Green: exit 0;
   canonical line = baseline + the new tests, no other movement)
4. `python tools/distill.py -- python tools/ui_smoke.py` (exit 0:
   `ui_smoke: PASS` - the runtime proof the boot still serves the flipped
   asset; the distill route is the sanctioned form per the standing
   contract)
5. `python tools/theme_token_check.py --check-added --base <start-rev>`
   (silent, exit 0: the combined form is canonical; the bare form is a
   usage error per the ledger)
6. `python tools/hygiene_check.py` (exit 0)
7. `python tools/encoding_guard.py --staged` (exit 0)
8. `python tools/commit_scoped.py --scope crates/ui/assets/extraction-wizard.js
   --scope crates/ui/assets/index.html --scope crates/ui/assets/styles.css
   --scope crates/ui/tests/ui_test/extraction_wizard_render.rs --scope
   crates/ui/tests/ui_test/extraction_card_playbook.rs --scope
   tasks/extraction-auto-apply-a5-journal.md -m "feat(ui): wizard execute
   step applies behind the approval token with live outcome reporting +
   help-copy flip"` (never `git add .`)
9. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 9 (the commit-cycle trio is gates 7-9; gate 4 is the runtime
proof this hand's asset edit requires). Bench stance: this hand touches
`crates/ui/assets` only - no path under `crates/*/src` - so the dated
DOCUMENTED_PHASES row's zero-diff producer proof is trivially satisfied
(an empty producer-crate diff, recorded in the journal) and NO timing
number is attributed to this hand. Do not run `bench_compare.py`.

## Acceptance (machine-checkable)

1. The S5 apply control is enabled only with a non-empty
   `wizardState.dryRun.apply_token`, and the dry-run-first instruction
   renders otherwise (required tests 1-2).
2. S6 renders the tier rows, the verdict, the created files and the verbatim
   `output_text`, all escaped (required tests 4-5), and keeps its
   descriptive copy when no outcome exists.
3. Every refusal code has its own copy and the 200 path advances to S6
   (required test 6).
4. The help copy carries the flipped claim and still names
   `extraction-auto-apply` in both tabs that assert it.
5. `python tools/distill.py -- python tools/ui_smoke.py` reports
   `ui_smoke: PASS`.
6. `python tools/theme_token_check.py --check-added --base <start-rev>`
   exits 0 with no added hex.
7. The C8 table is the COMPLETE amendment set: the reviewer's diff shows
   exactly three amended tests across exactly two files, each with its
   rationale comment, and `extraction_card_story.rs` +
   `extraction_wizard_api.rs` appear in no diff.
8. `python tools/hygiene_check.py` exit 0; `python
   tools/encoding_guard.py --all` exit 0.
9. The commit's touch-set is exactly the six files in gate 8 (reviewer
   commit-vs-touch-set enforcement).

## Discipline clauses

- UTF-8: Write/Edit tools only; never `Set-Content`/`Out-File`/`>`/
  `Get-Content | -replace`; pure-ASCII authored content; no em dashes; use
  `&rarr;` / `&#8592;` entities where a glyph is wanted.
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, direct `ui_smoke.py`, `bench_compare.py`) are a
  reviewer blocker: distilling entry points only.
- Do NOT add a `tests/ui_test` module, so do NOT touch `main.rs` (the
  kickoff's "A-3 and A-5 each add a mod line" is superseded by the shape's
  module map; the A7 oracle lives in the existing renderer file).
- Report-only observation to carry into the journal: the card's apply hint
  in `hotspots.js` (`renderExtractionApplyHint`, the comment at :521-522)
  still says "the UI states the command, it never runs anything". That file
  is outside this touch-set, no lock constrains the sentence, and the stale
  phrase is a report-only relay for the next card-touching hop.
- Checkpoint the journal after each Green gate (module, commit hash,
  deviation, lock finding).
- Context exhaustion: STOP with a disk-state inventory; successor resumes
  via `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-auto-apply-a5-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (the named failing
   tests with verbatim panic lines). 3. Green evidence (distill digest
   header line + canonical line + capture path; the `ui_smoke: PASS` line;
   the theme-token gate's silence). 4. Deviations + reasons. 5. The
   report-only `hotspots.js` relay. No raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly.
