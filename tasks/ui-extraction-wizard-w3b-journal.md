# W-3b Journal - ui-extraction-wizard: steps 2-4 live + execution-boundary help copy + C5 amendments

Handoff: `tasks/ui-extraction-wizard-w3b-handoff.md`. Base: HEAD `a7a3e2a`
(`c1956e5` + the Duty-1 docs commit made at the start of this session).
Boot: `checkpoint_guard status` clean (no dirty, no untracked); tree census
shows only unrelated detached/stale worktrees outside this checkout.

## Duty 1 (pre-hand)

Staged exactly the two W-3a review artifacts via the distiller, verified via
`git diff --cached --name-only` (exactly two paths), `encoding_guard --staged`
silent, committed via `commit_scoped.py`: `a7a3e2a` (verdict +194,
execution-results +80, zero deletions).

## Boot audit

- `crates/cli/src/commands/ui.rs` measured at 986/1000 lines - confirmed NOT
  in the W-3b touch-set (handoff forbids `crates/cli/**`); no edit risked.
- Routes/DTOs read (W-2b `api_extraction.rs`, W-2a `extraction_reader.rs`):
  route paths `/api/extraction/{plan_id}/manifest|dry-run`; degraded
  `honest_absent_dry_run` answers `move_removals: 0, copy_only: false,
  includes_unavailable: true` (reader :416-427); the computed path is
  `copy_only = move_removals == 0` (:189). The renderer must key the
  copy-only sentence off `move_removals === 0` gated by
  `includes_unavailable`, never reading `copy_only` (locked by
  `test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path`,
  which additionally locks the ABSENCE of `.copy_only` in the asset).

## Red (gate 2)

Digest: `298 passed / 14 failed / 0 ignored / 2 suites` (exit 1). Failing set
is EXACTLY the 5 C5-amended card locks + the 9 new wizard-step locks; every
panic names a missing fetch/verb/sentence/renderer/reset by name or the new
expectation:

1. `test_apply_recipe_is_honest` - third verb `tool transform extract` missing (W5).
2. `test_file_plan_rows_markup` - `.extraction-file-block` absent from the wizard asset.
3. `test_beats_appear_in_story_order` - `function renderWizardStepManifest(` missing (step order).
4. `test_playbook_styles_declared_tokens` - `.extraction-file-block {` rule missing from styles.css.
5. `test_risk_tier_badge_sanitized` - base rule not yet `var(--text-muted)`.
6. `test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path` - locked sentence missing (W4).
7. `test_wizard_dry_run_rewrites_render` - `'/dry-run'` fetch missing (C2).
8. `test_wizard_includes_fallback_when_unavailable` - `renderWizardStepManifest` undefined (W3).
9. `test_wizard_step_caps_and_truncation_are_honest` - `EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE = 200` absent (C6).
10. `test_wizard_help_copy_names_the_execution_boundary` - help copy absent.
11. `test_wizard_recipe_names_exactly_the_three_w5_verbs` - `renderWizardStepExecute` undefined.
12. `test_wizard_resets_on_payload_swap` - payload-swap reset absent (C7).
13. `test_wizard_step_manifest_fetches_per_step` - "extraction-wizard.js must fetch the manifest for the selected file".
14. `test_wizard_resets_on_stale_generation` - `fetchWizardRoute` (the 409 handler site) undefined.

Capture: `target/testrun-capture.txt` (via `testrun.py --distill`).

## Green

Digest: `312 passed / 0 failed / 0 ignored / 3 suites` (exit 0) = the 303
baseline + exactly the 9 new step locks; the five amended locks were amended
in place (count unchanged). Capture: `target/testrun-capture.txt`.

Landed content:
- `extraction-wizard.js` (182 -> 447 lines, pure ASCII, max line 308):
  one fetcher (`fetchWizardRoute`) as the single `fetch(` site;
  `ensureWizardManifest`/`ensureWizardDryRun` cache into the C7 state;
  `enterWizardStep` fires the per-step fetches on step entry only (steps
  0-1 render from the boot payload; openExtractionWizard fetches nothing);
  S2 renders the `.extraction-file-block` with capped names + `symbol_count`
  + `+K more` + the W3 includes-or-fallback block; S3 renders consumers,
  required headers, `cycle_error` and the copy-vs-move statement keyed via
  the `includes_unavailable` gate then the computed `move_removals === 0`
  path (`.copy_only` is NEVER read anywhere in the asset - locked); S4
  renders new-file paths and rewrite rows old `&rarr;` new with the
  side-effect preservations, `rewrites_truncated` honest copy, the C4
  changes-nothing sentence, and the copy-only restatement through the same
  gate; S5 the three-verb recipe (preview first) with `escapeHtml(planId)`
  and the W4 sentence; S6 the honest three-tier/rollback/artifact
  description (validate lib semantics: structural, generated-header
  existence, TU-include existence with the toolchain-skip honesty, rollback
  on Fail only, `artifacts/transaction` outcome record).
- Reset paths (C7): different-plan open (W-3a, preserved), 409 in
  `fetchWizardRoute` (reset + honest error + step view returns to S0), and
  `resetWizardOnPayloadSwap` via the load-time `window.applyLoadedData`
  wrap (the existing re-render path the live-refresh swap calls).
- `index.html` (641 lines): the never-applies sentence reworded to name the
  boundary (keeps the frozen "never applies" phrase, adds "never mutates
  the repository" + `extraction-auto-apply`); two new wizard bullets, every
  block < 400 chars for the frozen readability lock.
- `styles.css`: `.extraction-risk` base `var(--text-sub)` -> `var(--text-muted)`
  (C5 item 2, measured 2.97:1 dark in the dark-steampunk pairing per the
  C5 table) plus the W-3b dry-run/manifest rows appended inside the
  contiguous wizard block - token-only, zero hex.

## Refactor

No post-Green edits were needed: the fetch handling is one function (count
-locked at exactly one `fetch(` call site), the chip idiom one helper, the
copy-only keying one gate, the reset one function with three named callers.
Re-ran the scoped suite at the Green digest above; all functions under 100
lines; `extraction-wizard.js` 447 lines vs the 1000 ceiling.

## Gates

- G1 baseline: `303 passed / 0 failed / 0 ignored / 3 suites` (PASS).
- G2 Red: above (exit 1, 14 named failures, no others).
- G3 Green: `312 passed / 0 failed / 0 ignored / 3 suites` (exit 0).
- G4 `theme_token_check.py --check-added --base a7a3e2a`: silent, exit 0.
- G5 `hygiene_check.py`: exit 0; all output pre-existing R3 warn-only,
  none on lines this hand authored.
- G6 `distill.py -- python tools/ui_smoke.py`: exit 0,
  `boot+route checks clean (cold fixture, attempt 1, 1.1s)` /
  `ui_smoke: PASS` (the help-copy edit changed the served page; boot proof).
- G7 `encoding_guard.py --staged` after staging exactly the seven touch-set
  paths via the distiller: see commit section.
- G8 `commit_scoped.py` with the seven scopes (never `git add .`).
- G9 `encoding_guard.py --all` post-commit.

Bench-exemption proof (documented assets-only stance): the staged diff
carries zero `crates/*/src` paths (proven by the staged name-list below);
the boot payload byte lock (W-2a's captured-proposals test) is therefore
untouched by construction.

## Landing record (rider per D5)

- Landing commit: `24c2f7e` (full `24c2f7ea279de1241f876a3858ebcc11decb6dc1`),
  parent `a7a3e2a` - the boot HEAD, i.e. `c1956e5` + this session's Duty-1
  docs commit. `git show --stat`: 7 files, 904 insertions / 38 deletions
  (wizard.js +288, render.rs +322 NEW-locks, styles.css +92/-7, playbook
  +47/-6, story +35/-12, index.html +3/-1, journal +154).
- `git diff --name-only a7a3e2a 24c2f7e` = EXACTLY the seven touch-set paths;
  zero `crates/*/src`, zero `crates/cli/**`, zero frozen assets/tests.
- G7 `encoding_guard --staged`: silent (exit 0) on the exactly-seven staged
  set (staged via the distiller, never `git add .`).
- G9 `encoding_guard --all` post-commit: silent, exit 0 (distiller-verified).

## Deviations (numbered, per the W-3a verdict doc nit)

- D1 Gate-2 failing-set decomposition: the handoff's Red protocol names the
  409-reset lock explicitly; C7's payload-swap reset is a required build item
  ("each reset call is string-lockable"), so it rides as its own new
  wizard-step lock (`test_wizard_resets_on_payload_swap`) instead of being
  folded into the 409 lock. Still inside "named C5-amended locks + new
  wizard-step locks"; the Red set stayed exactly that union.
- D2 (planned, Green) C5 item 1 implementation reading: the amended
  `test_beats_appear_in_story_order` beats are the seven `renderWizardStep*`
  function declarations in S0..S6 order - the wizard asset's step list in
  renderer form (shape C5 "survives as an order lock over the wizard asset's
  step list"). The label array alone would have been green at Red (W-3a ships
  it), which the protocol forbids ("each amended card lock fails on its new
  expectation: the wizard-step order"). The renderer names are the shape
  section 9 intent-revealing names for S0..S6.
- D3 (planned, Green) Payload-swap reset hook idiom: hotspots.js
  `renderProposalsSection` is IIFE-internal (unhookable without touching a
  frozen asset), so the reset wraps `window.applyLoadedData` - the existing
  re-render path the payload swap itself calls (`api.js:163`) - via the
  load-time wrap precedent the W-3a verdict accepted for `closeModals` (D6).
- D4 Gates 4 and 5 exit codes read via `distill.py` (raw invocation of
  `theme_token_check.py` refused by the session allowlist; hygiene raw run
  printed output but the wrapper hides the exit code). Precedent: W-3a
  verdict dispositions D5/D7 (accepted). Both reported exit 0.
- D5 A post-landing journal-only rider commit is planned to record the
  landing hash, `git show --stat` and the gate-9 `--all` result - those are
  post-commit facts that cannot self-contain in the landing commit; house
  precedent `c1956e5` (W-3a D11, accepted). Docs-only, scoped to the journal
  path.
- D6 Amended locks 1 and 3-5 additionally read `ariadne_ui::EXTRACTION_WIZARD_JS`
  (import line added in both card files). The 13 surviving locks keep their
  bodies byte-identical; the file-level `use` line is shared infrastructure
  the amendments require, same shape as the accepted W-3a `theme_contract.rs`
  amendment (file edited, unamended locks untouched).
