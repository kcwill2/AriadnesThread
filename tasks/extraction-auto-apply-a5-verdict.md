# A-5 Verdict - extraction-auto-apply: S5 approval surface, S6 live outcome, the help-copy flip, C8 amendments

**Verdict: APPROVED.**

Reviewer hop, read-only with respect to source. Review range `42ccff5..4518e34`,
HEAD `4518e34`, branch `main`, worktree clean at start and at end
(`python tools/checkpoint_guard.py status`: HEAD `4518e34`, dirty 0, untracked
none; worktree census unchanged, none of the registered worktrees is this
hand's).

This is the milestone's final hand and it lands behind a browser-facing write
path, so the adjudication leans hardest on four things: the A7 approval gate
(no token, no request), the two-409 disambiguation at the asset's own fetch
site, the escape discipline on every new innerHTML path, and the honesty of
the copy now that the UI really does apply. All four hold.

## 1. Ground truth and commit-vs-touch-set (oldest first)

Range: `git log --oneline "42ccff5..4518e34"` = `4518e34`, `f89c901` (two
commits; the base `42ccff5` is duty 1, the already-adjudicated A-4 verdict,
verified by `git show` to be one file, `tasks/extraction-auto-apply-a4-verdict.md`,
387 lines, and it sits below the base so it is not judged here again).

**`f89c901` (parent `42ccff5`)** - judged alone. `6 files changed, 724
insertions(+), 22 deletions(-`:
- `crates/ui/assets/extraction-wizard.js` `168/8`
- `crates/ui/assets/index.html` `3/3` (three sentences replaced in place, net 0
  lines, so the asset-count, script-order and readability locks are untouched)
- `crates/ui/assets/styles.css` `26/0`
- `crates/ui/tests/ui_test/extraction_card_playbook.rs` `10/4`
- `crates/ui/tests/ui_test/extraction_wizard_render.rs` `249/7`
- `tasks/extraction-auto-apply-a5-journal.md` `268/0`

**`4518e34` (parent `f89c901`)** - journal only, `1 file changed, 31
insertions(+), 2 deletions(-)`, the landing record plus gates 7-9 and the
producer zero-diff proof. Nothing outside `tasks/**` (the W-1 D5 / A-2 D2 /
A-4 D7 rider precedent).

`git diff --name-only "42ccff5..4518e34"` = exactly the six declared touch-set
paths. **No commit lands outside the declared touch-set.** `commit_scoped.py`
discipline held; `git add .` never appears.

Frozen locks, byte-identity over the whole range. `git diff --stat 42ccff5
4518e34 -- crates/ui/src crates/ui/tests/ui_test/main.rs
crates/ui/tests/ui_test/extraction_card_story.rs
crates/ui/tests/ui_test/extraction_wizard_api.rs` returns empty output:
`crates/ui/src/**` (routes, DTOs, state, server, lib), `main.rs` (no test
module added), `extraction_card_story.rs` (all 9 of its card-family locks) and
`extraction_wizard_api.rs` (its GET-only and no-write locks, its amended
`sample_dry_run_dto`) are byte-identical. Producer zero-diff proof:
`git diff --name-only 42ccff5..HEAD -- crates` lists only
`crates/ui/assets/extraction-wizard.js`, `crates/ui/assets/index.html`,
`crates/ui/assets/styles.css`, `crates/ui/tests/ui_test/extraction_card_playbook.rs`
and `crates/ui/tests/ui_test/extraction_wizard_render.rs` - **zero paths under
any `crates/*/src`**. No `Cargo.toml`/`Cargo.lock` change, no `tools/**`
change, `tests/golden/**` and `tests/fixtures/**` untouched,
`tools/theme_token_baseline.json` diff-empty, the boot payload bytes unchanged
(no payload field added), and the A8 bind set is in no diff because no route or
handler was touched.

Pure ASCII: `extraction-wizard.js` is locked ASCII-only by the new test 7
(`EXTRACTION_WIZARD_JS.is_ascii()`, ok in the fresh run), and
`encoding_guard.py --all` is silent exit 0 (fresh, below).

## 2. D1, the headline deviation - ACCEPTED

The handoff carries two clauses in tension:

- **Clause 1 (frozen locks, :93-95):** "The C8 amendment table is the COMPLETE
  set of card-family and wizard-era edits. Any further card-lock edit
  discovered mid-hand is a STOP-and-report (shape risk R7), never a silent
  fix." Acceptance 7 (:283-286) restates it: "the reviewer's diff shows
  exactly three amended tests across exactly two files".
- **Clause 2 ("What to build" item 3, :137-141):** the apply request is "a POST
  to `/api/extraction/<plan id>/apply` with the JSON body
  `{"apply_token": <the dry-run's token>}`, through its own fetch site (NOT
  the blanket-409 `fetchWizardRoute`, because the apply's 409 has two meanings
  that must be told apart)".

**Determination: Clause 2 is authoritative for implementation; Clause 1 is a
preservation guard and does not reach this edit.** Five reasons, each checked
rather than assumed:

1. **The clauses are mutually unsatisfiable, so one must yield.** Clause 2
   requires a second `fetch(` in the asset; the existing lock
   `test_wizard_step_manifest_fetches_per_step` asserted
   `EXTRACTION_WIZARD_JS.matches("fetch(").count() == 1`. There is no tree
   that satisfies both as written.
2. **No compliant alternative exists under Clause 1's literal reading.**
   Keeping `count == 1` forces the apply POST through `fetchWizardRoute`,
   which Clause 2 prohibits in terms, and which would collapse the two 409
   meanings (the blanket reset fires on `apply_in_progress` as if the
   generation were stale) - directly breaking acceptance 3 ("Every refusal
   code has its own copy", both 409 branches named). A re-land that preserves
   the old lock cannot implement the handoff's own mandate.
3. **Clause 1's enforcement trigger names card locks.** "Any further
   **card-lock** edit" is what triggers STOP-and-report, and shape risk R7 is
   a card-family risk. The byte-identity duty in the preserved invariants
   (A10, :61-63) is enumerated over card-family locks: "16 of the 18
   card-family locks survive byte-identical". The fetch-count lock is a
   wizard-era lock: it is in neither the C8 table nor shape section 9's freeze
   set, and its file is in this hand's touch-set ("plus the new A7/A9 locks").
4. **The relaxation is minimal, tight and intent-preserving.** The count
   assertion became `matches("fetch(").count() <= 2` (actual count 2 at
   Green, 1 at Red), with the rationale comment naming the milestone and the
   handoff directive (:344-348). The GET-side assertion - "the single fetch
   call site must live in fetchWizardRoute" - is byte-identical and still
   scoped to the fetcher body, and every other assertion in that test (the
   open path fetches nothing, per-step entry drives the fetches, navigation
   enters the step) is untouched. The lock's intent - fetch handling stays in
   named places, bounded - is preserved; the new test 6 additionally pins the
   apply site's shape (`extractionRouteUrl('/apply')`, `method: 'POST'`,
   `apply_token`) and that it never rides `fetchWizardRoute`.
5. **It was declared, not silent.** Journal D1 names the file and lines,
   quotes Clause 2, argues the disposition, and routes it to the reviewer
   while holding the Green set stable (the assertion holds at Red 1 <= 2 and
   Green 2 <= 2, so the Red/Green rosters are exactly as the handoff
   predicts). That is the STOP-and-report duty discharged in the only form
   that does not breach Clause 2.

**A7 backstop, verified independently in the asset** (not via the report).
`applyExtractionPlan` (extraction-wizard.js:447-449) opens
`const applyToken = wizardState.dryRun ? (wizardState.dryRun.apply_token || '') : '';
if (applyToken === '' || wizardState.applying) return;` - with no token the
function returns before any fetch, so **no token, no request**. The control is
`disabled` when `applyDisabled = approvalMissing || inFlight` (:419) and the
disabled state carries the dry-run-first instruction (:410, :420); the
delegated click handler checks `!applyControl.disabled` (:507) as a belt on
top of the render gate. The token is read from exactly two sites, both
`wizardState.dryRun.apply_token` (:416, :448), i.e. only ever from the GET
dry-run DTO; the POST body is `JSON.stringify({ apply_token: applyToken })`
(:457) to `extractionRouteUrl('/apply')` (:454) from the asset's own fetch
site, never through `fetchWizardRoute`.

**Red evidence.** The Red phase is not re-runnable at HEAD under a read-only
grant (re-creating it requires reverting the asset, which is mutation), so the
accepted evidence is the strongest obtainable without touching the tree:
(a) the three mandated panic strings are present verbatim in the landed locks
- "renderWizardStepExecute must gate its apply control on the dry-run
apply_token", "the execute step must instruct the user to open the dry run
first when no token is present", "renderWizardStepAfterApply must render the
outcome tiers, verdict and output_text" - and the Green run proves those
assertions now pass against the shipped asset; (b) the arithmetic reconciles
exactly: scoped baseline 327 -> Red `327 passed / 8 failed / 0 ignored / 2
suites` (335 tests ran; 3 of the 8 new locks Red-green by design - required
tests 3, 5, 7 - plus the 3 amended help-copy tests and required tests 1, 2, 4,
6, 8) -> Green `335 passed / 0 failed / 0 ignored / 3 suites` = baseline + 8,
zero movement; (c) my fresh workspace census `1521 passed / 0 failed / 13
ignored / 112 suites` = the 1513 anchor + exactly 8, all eight new tests
confirmed `... ok` by name in the capture.

## 3. Deep adjudication of the named items

### 3.1 D2 - Red roster superset: ACCEPTED

The handoff's Red roster names required tests 1, 2 and 4 plus the three C8
amendments (six failures); the observed set is eight. Required tests 6 (the
apply mapping locked over the named `applyExtractionPlan`) and 8 (the
`styles.css` block behind its marker) cannot simultaneously be strong Green
locks and pass against an asset that does not yet carry what they lock; their
panics name their own purpose ("the apply request must be one named function
applyExtractionPlan (shape section 9 binding name)", "styles.css must carry
the contiguous A-5 apply/outcome block behind its marker comment"). The
handoff's stop clause targets failures in any OTHER test file; all eight
failures are inside the two touched test files. The superset is forced by the
handoff's own required-tests list and is declared (D2). Accepted.

### 3.2 D3 - the 500 detail through `textContent`: ACCEPTED

Verified in the asset: `renderWizardError` (:61-66) sets
`slot.textContent = wizardState.error || ''` - the wizard error slot is
`textContent`, never `innerHTML`. The 500 `apply_failed` detail is therefore
rendered through a structurally injection-proof channel, strictly stronger
than an `escapeHtml` wrap and identical to the house idiom every existing
refusal copy already uses (`fetchWizardRoute`'s network, 409 and generic
refusals all set `wizardState.error`). The escape invariant binds the S6
innerHTML path, where every value is escape-wrapped and locked by required
tests 4-5. Compliant in substance; the letter of the invariant ("the refusal
detail passes through escapeHtml") is satisfied a fortiori. No weakening.

### 3.3 D4 - 404 copy authored; the watcher-off clause: ACCEPTED

The handoff's mapping list omits 404 `unknown_plan`, but acceptance 3 demands
"Every refusal code has its own copy" and the route answers 404 (the A-4
adjudication verified `UnknownPlan` is checked before the lock). The asset
carries a dedicated branch plus sentence (:493-494), locked by required test 6
(`apply.contains("status === 404")`). The enumerated mapping is read as a
minimum, and acceptance 3 is the governing clause. The S6 watcher sentence
(:530) states the mechanism and both states - "The background watcher
re-analyzes the repository and this wizard resets when the new analysis lands;
with watching disabled the analysis refreshes at the next UI start. The
outcome record also lands under artifacts/transaction." - which is exactly
the C6 invariant's demand plus the honest fallback, because the front end
cannot observe the watcher setting. Accepted.

### 3.4 D5 - the report-only relays, untouched: CONFIRMED

`crates/ui/assets/hotspots.js:522` still carries the comment "the command, it
never runs anything" (verified present at HEAD; the file is in no diff of the
range). `extraction-wizard.js:386`'s S4 truncation note "the apply rewrites
every consumer" is pre-existing and unchanged (the diff's nearest hunk starts
below it). Neither is locked, neither is in the touch-set, and neither was
touched. Both are carried into the closeout list below.

### 3.5 D6 - the CSS one-line shape: ACCEPTED

`styles.css` gained one contiguous block appended at EOF behind the marker
comment `=== EXTRACTION AUTO-APPLY (extraction-auto-apply A-5)`, 26 lines net
against the ~40 cap, single-property rules as disciplined one-liners to honor
the cap on the over-ceiling file (whose dated deferred row stands; `.css` is
invisible to R2). Zero `#` hex literals, colors from existing theme tokens
only (text-muted, amber, green, red, text-main, bg-card, border), verified in
the file and locked by required test 8, and proven workspace-wide by the
fresh `theme_token_check --check-added --base 42ccff5` (silent exit 0).
Accepted.

### 3.6 Acceptance 7 - the C8 amendment diff hunks

Exactly three C8 amendments across exactly two files, each with its one-line
rationale comment naming the milestone and the C8 row, and no fourth C8
amendment:

1. `extraction_card_playbook::test_apply_recipe_is_honest` :213-224 - the
   `legend.contains("never applies")` assertion (message `C7: the reworded
   sentence must still state that the UI never applies anything`) replaced by
   the flipped claim with the `A-5 C8 amendment 1: ...` message; the
   `extraction-auto-apply` key assertion at :221-224 and the P9 verb census at
   :202-212 untouched.
2. `extraction_card_playbook::test_help_playbook_copy` :306-312 - the same
   replacement with the `A-5 C8 amendment 2: ...` message; "collapsed summary",
   "three tiers", "rollback", "Proposed extractions section" and "Preview
   after extraction" all survive (:302-324).
3. `extraction_wizard_render::test_wizard_help_copy_names_the_execution_boundary`
   :596-599 - `legend.contains("never mutates the repository")` (message `the
   Hotspots help copy must state that the UI never mutates the repository in
   this version`) replaced by the flipped claim; the `extraction-auto-apply`
   key assertion at :600-603 byte-identical; the W-3b raw-interpolation ban
   at :604-616 byte-identical.

The fourth amended test in the range is D1's fetch-site bound (section 2) -
not a C8 amendment, adjudicated there. `extraction_card_story.rs` and
`extraction_wizard_api.rs` appear in no diff. Acceptance 7's literal "exactly
three amended tests" is superseded by D1 and recorded as finding 1 below.

### 3.7 Escaping discipline and the locks

- Every dynamic value on the new innerHTML paths passes through `escapeHtml`:
  `instructionText`, `APPLY_INFLIGHT_COPY` (:424, :431), `tier.tier`,
  `statusText`, `tier.detail`, `filePath`, `outcome.output_text`, `verdictText`
  (:541-560). The apply button label and disabled attribute interpolate only
  static strings. The outcome renderer is locked by required test 4 (the five
  `escapeHtml(...)` forms asserted) and the asset-wide ban is locked by
  required test 5 (no `${outcome.` / `${applying`).
- The asset does not re-declare `escapeHtml` (no declaration anywhere in the
  file; the load-order contract at :10-12 documents that the escaper ships
  once in hotspots.js).
- W5 verb-count lock intact: `named_verbs == honest_verbs` over the whole
  asset, and a fresh grep finds exactly four `tool ` occurrences
  (extraction-wizard.js:399, :427, :428, :429) - all pre-existing, none added
  by this hand; 4 named = 4 honest (tool apply x2, tool migrate, tool
  transform extract). The three-verb recipe and the W4
  `COPY_VS_MOVE_SENTENCE` interpolation are byte-identical (:427-430), and
  "The UI shows this recipe and never runs it" is gone.
- ASCII-only authored content: locked by required test 7 over the whole asset
  and by `encoding_guard.py --all` (silent exit 0, fresh).

### 3.8 Honesty (D6 latent-bug consistency)

No sentence authored in this hand claims consumer rewrites. The flipped help
copy claims exactly what ships: "the UI applies with a dry-run first,
validation in three tiers and automatic rollback" (index.html:548, :551), the
reworded neighbour describes the shared dry-run-then-apply path (:549), the
approved instruction writes "applying writes the new files, validates in three
tiers with automatic rollback, and reports the outcome" (:411), the verdict
lines state only committed/rolled-back disk truth (:549-550), and the S6
watcher sentence states the mechanism plus the watching-disabled fallback
rather than an unverifiable rebuild (:530). The only pre-existing sentence
touching the D6 claim is the S4 truncation note (report-only, section 3.4).
The D6 latent defect itself remains A-4's deferred row (section 8).

## 4. Deviation dispositions

| Deviation | Disposition | Reasoning |
|---|---|---|
| D1 - one-fetch-site lock relaxed to the documented bound | **ACCEPTED** | Section 2. Clause 2 (own fetch site, two 409 meanings) is authoritative and unsatisfiable against the old lock; the bound is tight (2), the GET-side lock is byte-identical, the intent is preserved, and it was declared with rationale rather than silent. |
| D2 - Red set superset (8 failures, not 6) | **ACCEPTED** | Section 3.1. Required tests 6 and 8 must fail at Red by construction; all failures are inside the two touched test files, which is the only stop condition the handoff names. |
| D3 - 500 detail via `textContent`, not `escapeHtml` | **ACCEPTED** | Section 3.2. The error slot is textContent (verified :61-66), a stronger posture than escapeHtml and the house idiom; the innerHTML paths are fully escaped and locked. |
| D4 - 404 copy authored; watcher-off clause in S6 | **ACCEPTED** | Section 3.3. Acceptance 3 ("every refusal code has its own copy") governs over the mapping list's enumeration; the watcher sentence covers both observability states honestly. |
| D5 - report-only relays untouched | **CONFIRMED** | Section 3.4. hotspots.js:522 and the S4 note are present, pre-existing, unlocked and unchanged; carried to closeout. |
| D6 - CSS one-line shape (26 net vs ~63 in house shape) | **ACCEPTED** | Section 3.5. Contiguous, net-capped, zero hex, tokens only; the cap is the discipline on an over-ceiling file. |
| D7 - journal-only rider commit | **ACCEPTED** | Precedent W-1 D5, A-2 D2, A-4 D7; the rider is journal-only and inside `tasks/**`. |

## 5. Findings

All findings below are minor and non-blocking; none requires a re-land.

1. **MINOR (record) - acceptance 7's literal text is superseded by D1.**
   `extraction_wizard_render.rs:349-352`. The handoff's acceptance 7 says the
   reviewer's diff shows "exactly three amended tests across exactly two
   files"; with D1 the range amends four (the three C8 rows plus the
   fetch-site bound). This is the accepted disposition of section 2, not a
   contract breach, but the planner must record the new bound - the wizard
   asset keeps its fetch handling to the GET fetcher plus exactly one
   sanctioned apply POST site - as the wizard-era invariant of record, so a
   later hand does not "restore" `count == 1` and detonate the apply path.
2. **MINOR (record) - the network-failure copy hardcodes the route name.**
   `extraction-wizard.js:461`. The apply's network copy is the literal
   `'the extraction route /apply is unreachable'` where the fetcher
   interpolates its suffix (`'the extraction route ' + routeSuffix + ' is
   unreachable'`, :234). The idiom and the "is unreachable" lock shape are
   preserved and nothing depends on it; recorded for the next touch of this
   file, no change required.

## 6. Fresh gate results (reviewer hop, run now)

- `python tools/testrun.py --distill` (full workspace census) -
  `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2838 lines]`
  **`1521 passed / 0 failed / 13 ignored / 112 suites`**
  `no failures`
  Canonical line quoted verbatim. Reconciled against the anchor
  `1513 passed / 0 failed / 13 ignored / 112 suites` = exactly **+8** from
  A-5, the eight new renderer locks, suites held at 112, zero movement
  elsewhere. All eight confirmed by name in the capture, all `... ok`.
- `python tools/testrun.py --distill -p ariadne-ui` (the handoff's scoped
  gate) - `### cargo test PASS (exit 0) [capture: ... 364 lines]`
  **`335 passed / 0 failed / 0 ignored / 3 suites`**
  `no failures`
  = the journal's gate-1 baseline `327 passed / 0 failed / 0 ignored / 3
  suites` + exactly the 8 new locks, agreeing with the implementer's gate-3
  line verbatim.
- `python -m unittest discover -s tools -p "*_tests.py"` - **`Ran 219 tests in
  22.783s`** / **`OK`**.
- `python tools/distill.py -- python tools/ui_smoke.py` - `### generic PASS
  (exit 0)`, `boot+route checks clean (cold fixture, attempt 1, 1.0s)`,
  **`ui_smoke: PASS`**. The runtime proof the boot serves the flipped assets.
- `python tools/distill.py -- python tools/bench_compare.py` -
  `### generic FAIL (exit 2)`,
  `ERROR parse format drift: documented phase row(s) missing from bench output
  (never substituting zeros):`
  Deterministically blocked by the `DOCUMENTED_PHASES` tuple at
  `tools/bench_compare.py:84`; the ledger already carries this as an OPERATOR
  blocker and `tools/**` is forbidden to this hop, so it was not edited and no
  timing number is attributed to this hand. The hand's exemption stance is
  discharged by the recorded empty producer diff (section 1: zero paths under
  `crates/*/src`).
- `python tools/distill.py -- python tools/theme_token_check.py --check-added
  --base 42ccff5` - `### generic PASS (exit 0)`, capture 0 lines (silent exit
  0). Zero added hex over the whole hand.
- `python tools/distill.py -- python tools/hygiene_check.py` -
  `### generic PASS (exit 0)`. R1/R2 zero. The 179-line R3 list is the
  pre-existing warn-only baseline; zero rows name any file this hand touched
  (`extraction-wizard.js` is a `.js` asset, invisible to the checker; the two
  amended test `.rs` files, `index.html` and `styles.css` carry no rows).
- `python tools/distill.py -- python tools/encoding_guard.py --all` -
  `### generic PASS (exit 0)`, capture 0 lines (silent exit 0).

Refused commands (verbatim catch-all, per the reviewer rule): `git show --stat
42ccff5 | Select-Object -First 12`; `git diff --name-only 42ccff5..HEAD --
"crates/*/src"`; `python tools/hygiene_check.py > $env:TEMP\kilo\hygiene-a5.txt
2>&1; echo "hygiene_exit=$LASTEXITCODE"`; `python tools/hygiene_check.py; echo
"hygiene_exit=$LASTEXITCODE"`. Each refusal text: `The user has specified a
rule which prevents you from using this specific tool call. Here are the
relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
Each was worked around through the dedicated git MCP tools
(`git show`, `git diff` on `42ccff5..HEAD -- crates`), the plain invocation,
and the distilling entry point, which reports exit codes. No gate I was
expected to run was skipped, and no weaker evidence was substituted.

## 7. Execution-results addendum (against the acceptance criteria)

1. **S5 approval surface gated on the token** - MET. `applyDisabled =
   approvalMissing || inFlight` keyed off `wizardState.dryRun.apply_token`
   (extraction-wizard.js:416-419); the dry-run-first instruction renders in
   the disabled state (:410, :420); no token means no request (:449).
   Evidence: `test_execute_step_apply_control_requires_an_approval_token
   ... ok` (capture line 97), `test_execute_step_shows_the_dry_run_first_instruction_without_a_token
   ... ok` (capture line 101).
2. **S6 live outcome, all escaped, descriptive copy kept** - MET. Tier rows
   with PASS/FAIL/SKIPPED plus reasons, committed-vs-rolled-back verdict,
   created files, verbatim `output_text` in a `pre` (:536-562); every
   interpolation escape-wrapped; `renderWizardStepAfterApply` keeps its
   descriptive copy with no outcome (:568-576). Evidence:
   `test_after_apply_step_renders_the_live_outcome_rows ... ok` (capture line
   92), `test_every_outcome_interpolation_is_escaped ... ok` (capture line
   106).
3. **Every refusal code has its own copy; 200 advances to S6** - MET. Seven
   branches plus the generic fallback (:459-497): network, 409
   stale_generation (rides the existing reset with the GET path's
   byte-identical copy), 409 apply_in_progress, 428, 500 detail, 503, 404;
   200 stores the outcome, clears `applying`, `showWizardStep(6)` +
   `enterWizardStep(6)`. Evidence:
   `test_apply_response_mapping_covers_every_refusal ... ok` (capture line
   95).
4. **Help copy flipped; key preserved in both asserting tabs** - MET.
   index.html:548 and :551 carry the claim of record verbatim plus
   `extraction-auto-apply`; :549 reworded to the shared dry-run-then-apply
   path; no "never applies" / "never mutates" / "never runs" remains anywhere
   in index.html. Evidence: `test_apply_recipe_is_honest ... ok`,
   `test_help_playbook_copy ... ok`,
   `test_wizard_help_copy_names_the_execution_boundary ... ok` in the scoped
   run.
5. **`ui_smoke: PASS`** - MET (section 6).
6. **`theme_token_check --check-added --base 42ccff5` silent exit 0** - MET
   (section 6); required test 8 `test_apply_outcome_style_block_adds_no_hex
   ... ok` (capture line 93) is the in-repo twin.
7. **C8 table is the complete amendment set** - MET with the D1 disposition:
   exactly three C8 amendments across the two files, each with its rationale
   comment; `extraction_card_story.rs` + `extraction_wizard_api.rs` in no
   diff; the fourth amended test is D1's documented fetch-site bound
   (section 2, finding 1).
8. **Hygiene exit 0; encoding_guard --all exit 0** - MET (section 6). Function
   and file ceilings: `extraction-wizard.js` 606 lines, `applyExtractionPlan`
   54 lines, `renderWizardStepOutcome` 27 lines, `extraction_wizard_render.rs`
   851 lines, `extraction_card_playbook.rs` 325 lines, `index.html` 640 lines
   - all under the 1000/100 ceilings.
9. **Commit touch-set exactly the six files** - MET for both commits
   (section 1).

Not run in this hop, stated explicitly: the Red phase (not re-runnable at
HEAD under the read-only grant; substituted evidence described in section 2);
`cargo clippy` (no clippy gate exists in this repo and no handoff gate names
it); `bench_compare.py` (deterministically blocked at
`tools/bench_compare.py:84`, operator-blocked, `tools/**` edit-forbidden - the
empty producer diff is the compensating proof); and any `--release` smoke or
bench form, which no handoff for this hand mandates.

## 8. Closeout-relevant statement - what the planner must record in the ledger

All items below are dated rows owed by the closeout write; none blocks this
verdict.

1. **The D6 latent vacuous-rewrite bug (the headline deferred row).** From the
   A-4 verdict, unchanged by this hand: an apply's consumer rewrites are a
   no-op on the basic fixture because consumer discovery reports the plan
   target's spelling `include/thing.h` as the rewrite's `old_includes`
   (`crates/cli/src/commands/transform.rs:330-334`) while the sources hold the
   bare `#include "thing.h"`; locked as characterization only by
   `tests/integration/tests/m20_apply_migrate_characterization_test.rs:179`
   `apply_thing_plan_locks_transaction_outcome_and_disk_effects`. Shape A6's
   consumer half is therefore not satisfiable today and A-4 required test 1
   asserts subset parity plus a non-vacuity guard. Fix owner: `crates/transform`
   (`IncludeRewriter`) plus the consumer-discovery `old_includes` spelling;
   requires amending the m20 characterization.
2. **The D6 follow-up oracle.** Expected-modified = the approved consumers
   whose on-disk text actually contains an `old_includes` spelling (the
   dry-run DTO exposes `old_includes`, `crates/ui/src/dto/extraction.rs:167`);
   attach to the row above (A-4 verdict finding 2).
3. **The A9 path carve-out (A-4 verdict finding 1).**
   `crates/cli/src/commands/ui/extraction_apply.rs` `outcome_dto` sanitizes
   `output_text`, `detail` and the fail/skip reasons but leaves `plan_id` and
   the `repo_relative` rendering of `new_files` unsanitized; the `plan_id`
   half is a one-line `ascii_only(&plan_id)` on any later touch of that file.
4. **The cosmetic A-4 finding 3.** `extraction_apply.rs:243-248`, the
   `ascii_only` doc-comment indentation; next touch of the file.
5. **The `hotspots.js` stale apply-hint relay (now load-bearing).**
   `crates/ui/assets/hotspots.js:521-522` (`renderExtractionApplyHint`) still
   states "the UI states the command, it never runs anything" - as of this
   hand the help copy says the opposite, so the card hint now contradicts
   shipped behavior. The next card-touching hop must flip it to the claim of
   record ("the UI applies with a dry-run first, validation in three tiers and
   automatic rollback").
6. **The S4 truncation-note relay.** `extraction-wizard.js:386` "the apply
   rewrites every consumer" is aspirational against the D6 latent bug
   (consumers are not rewritten on the current fixture family); it renders
   only when the rewrite list truncates at 200. Report-only; reword when the
   D6 row is worked.
7. **The new wizard-era fetch-site invariant.** The wizard asset keeps its
   fetch handling to the GET fetcher plus exactly one sanctioned apply POST
   site (`extraction_wizard_render.rs:349-352`, bound `<= 2`); a later hand
   must not "restore" the old `count == 1` (finding 1).
8. **The two A-4 non-numbered observations.** A-4 required test 1 does not
   assert the absence of deleted files (a one-line before-only-keys check
   would extend the whole-tree guarantee); and no test drives a generation
   swap while a transaction is in flight (shape C5's "not cancelled" claim
   follows from the applier owning its `Arc<AnalysisGeneration>`).
9. **Bench stance unchanged.** `bench_compare.py` remains deterministically
   blocked at `tools/bench_compare.py:84` (DOCUMENTED_PHASES) as an OPERATOR
   row; this hand has an empty `crates/*/src` diff and attributes no timing
   number.

## 9. Handoff back to the planner

A-5 is approved for the ledger as landed at `4518e34` (landing `f89c901` plus
journal rider `4518e34`). With this verdict the milestone's final hand is
adjudicated: the extraction path now runs dry-run-first behind a single-use
approval token from the wizard, reports its live outcome with honest rollback
copy, and the help text finally matches the shipped behavior. The two minor
findings are recorded, not fixed, per the reviewer's read-only grant. The
deferred rows in section 8 go into the milestone closeout write
(`milestone-lifecycle`): the D6 vacuous-rewrite row with its follow-up oracle,
the A-9 carve-out one-liner, the cosmetic note, the `hotspots.js` relay (now
contradicting the flipped copy), the S4 truncation-note relay, the fetch-site
invariant, and the two A-4 observations. The next implementer hop commits this
verdict as its duty 1.
