# W-3b Verdict - ui-extraction-wizard: steps 2-4 live + help copy + C5 amendments

Reviewer gate over `tasks/ui-extraction-wizard-w3b-handoff.md`.
Commits under review: `24c2f7e` (landing) and `b8597f1` (journal rider),
range base `a7a3e2a`.

**Verdict: APPROVED.** No finding touches the touch-set, the C5 amendment
table, the copy_only keying proof, a frozen lock, or a gate.
Two non-blocking documentation nits are recorded as findings 1 and 2.

## 1. Ground truth and range

`python tools/checkpoint_guard.py status` at review start: HEAD `b8597f1` on
branch `main`, working tree clean, no untracked files.
`git log --oneline "a7a3e2a..b8597f1"` = exactly two commits (`24c2f7e`,
`b8597f1`).

## 2. Commit-vs-touch-set, per commit, oldest first

- `24c2f7e` (landing): `git show --stat` = 7 files, +904/-38
  (extraction-wizard.js +288, extraction_wizard_render.rs +322, styles.css
  +92/-7, extraction_card_playbook.rs +47/-6, extraction_card_story.rs
  +35/-12, index.html +3/-1, journal +154). Every path is inside the
  handoff's declared touch-set.
- `b8597f1` (rider): one file, `tasks/ui-extraction-wizard-w3b-journal.md`,
  +13/-0, docs-only (the landing-record section). Inside the touch-set.
- Range check: `git diff --name-only "a7a3e2a" "b8597f1"` = EXACTLY the seven
  declared paths. Nothing else.
- `git diff --name-only "a7a3e2a" "b8597f1" -- crates` lists only the three
  assets and the three ui test files; `git diff --stat` over
  `crates/ui/src crates/cli/src crates/transform/src crates/validate/src
  crates/cluster/src crates/graph/src` is empty. Zero `crates/*/src` diff,
  zero `crates/cli/**`, zero `tools/**`, zero `tests/**`.

## 3. C5 amendment table, executed exactly

All five rows landed, each with a rationale comment in its diff, zero
deletions, and no sixth amendment:

1. `extraction_card_story::test_beats_appear_in_story_order` - the beat list
   is now the seven `renderWizardStep*` declarations in S0..S6 C1 order
   (Overview, Files, Manifest, Impact, DryRun, Execute, AfterApply), read from
   `EXTRACTION_WIZARD_JS`. Rationale comment present.
2. `extraction_card_story::test_risk_tier_badge_sanitized` - base rule now
   expects `var(--text-muted)`; the matching swap landed in `styles.css`
   `.extraction-risk` with a rationale comment. Zero hex added or removed.
3. `extraction_card_playbook::test_file_plan_rows_markup` - the expectation
   moved to `EXTRACTION_WIZARD_JS.contains("class=\"extraction-file-block\"")`,
   matching the new `.extraction-file-block` rule in the wizard CSS block.
   Rationale comment present.
4. `extraction_card_playbook::test_playbook_styles_declared_tokens` - selector
   list extended with `.extraction-file-block {`, `.wizard-file-row {`,
   `.wizard-file-path {`, `.wizard-rewrite-row {`, all of which exist in
   `styles.css`. Rationale comment present.
5. `extraction_card_playbook::test_apply_recipe_is_honest` - verb list grows by
   `tool transform extract` only (asserted present in the wizard asset, and
   the asset-wide verb-count invariant now spans HOTSPOTS_JS +
   EXTRACTION_WIZARD_JS), and the never-applies sentence now demands both the
   frozen "never applies" phrase and the follow-on key `extraction-auto-apply`
   in the legend tab. Rationale comment present.

Row-for-row diff comparison against shape section 3 C5 and the handoff item 5:
exact. No amendment outside the table, no lock deleted.

## 4. The 13-survivor byte-identity claim (D6), verified test by test

The card family holds 18 locks (9 in `extraction_card_story.rs`, 9 in
`extraction_card_playbook.rs`). Exactly five bodies changed (the C5 rows
above). The other 13 bodies are byte-identical under my own per-file diffs:

- story survivors: `test_why_beat_visible_in_body`,
  `test_plan_id_identity_chip`, `test_saving_computed_with_zero_guard`,
  `test_no_runon_join_blobs`, `test_names_and_headers_render_as_chips`,
  `test_order_and_preview_locks`, `test_tu_sentence_and_help_copy`.
- playbook survivors: `test_details_collapsed_by_default`,
  `test_disclosure_button_contract`, `test_summary_meta_and_risk_visible`,
  `test_playbook_escape_discipline`, `test_no_new_data_sources`,
  `test_help_playbook_copy`.

The only non-test-body edits in either file are the shared `use` lines, which
gain `EXTRACTION_WIZARD_JS` (required by amendments 1 and 3-5). Same shape as
the accepted W-3a `theme_contract.rs` amendment (file edited, unamended locks
untouched). W7 holds.

Frozen-lock byte-identity, all diff-empty in the range:
`proposals_payload.rs`, `static_html.rs`, `static_html_sections.rs`,
`help_center_readability.rs`, `export_order.rs`, `dto_builders.rs`,
`theme_contract.rs`, `theme_dark_grey_palette.rs`, `theme_js_zero_hex.rs`,
`extraction_contrast.rs`, `preview_hypothetical.rs`, `icon_asset.rs`,
`asset_injection_hardening.rs`; assets `preview.js`, `api.js`, `graph.js`,
`theme.js`, `themes.css`; `tools/theme_token_baseline.json`;
`tests/fixtures/**` and `tests/golden/**`.

## 5. The copy_only / includes_unavailable keying proof, re-proved

Read from the asset and the lock, not from the journal:

- `wizardCopyOnly(dryRun)` (asset lines 299-302) gates
  `dryRun.includes_unavailable` FIRST and returns false, then judges
  `dryRun.move_removals === 0`. That is exactly the required order.
- The literal `.copy_only` never appears in the asset. The word `copy_only`
  appears only inside comments (lines 210 and 296), never as a field read.
  The lock `test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path`
  asserts `!EXTRACTION_WIZARD_JS.contains(".copy_only")`, so the absence is
  machine-enforced.
- The degraded answer is real, not hypothetical: the provider's
  `honest_absent_dry_run` answers `move_removals: 0, copy_only: false,
  includes_unavailable: true` (`crates/cli/src/commands/ui/extraction_reader.rs`
  :416-425), while the computed path is `copy_only: move_removals == 0`
  (same file :189). Under that degraded payload:
  - S4 takes the `dryRun.includes_unavailable` branch and renders
    "Includes unavailable: the provider could not complete this dry run;
    nothing is claimed about the writes." (asset lines 383-385).
  - S3 takes the `dryRun.includes_unavailable` branch and renders
    "Includes unavailable: the provider could not complete the dry run for
    this generation, so no copy-vs-move claim is made." (asset lines 347-348).
  - S2 keys its own fallback off the per-file `includes_unavailable` flag
    (asset lines 321-324): "Includes unavailable: the engine could not resolve
    the dependencies for this new file; nothing is fabricated here."
  The degradation line therefore renders in every panel; the forced-false
  `copy_only` flag can never produce a false copy-only claim.

## 6. Steps 2-4 content fidelity, caps, and the W9 boot payload

Against shape C1 rows and C6:

- S2 (lines 307-332): the per-file manifest for the selected file: capped
  `symbol_names` (at `EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE = 200`) with
  `symbol_count` beside them as the truth, the shared `wizardCapChip`
  `+K more` chip, `files_truncated` copy, and the includes block (engine
  `required_includes` rows or the honest fallback). Symbols AND includes, both
  from the route payload. No symbol body is ever rendered.
- S3 (lines 338-359): consumer TU count, required headers, the engine
  `cycle_error` when present, and the copy-vs-move statement keyed through the
  proven gate.
- S4 (lines 364-393): new-file paths capped at
  `EXTRACTION_MANIFEST_MAX_FILES = 64`, every consumer rewrite old -> new
  separated by the `&rarr;` entity, `side_effect_preservations` rendered as
  kept-for-side-effects, rewrites capped at
  `EXTRACTION_DRY_RUN_MAX_REWRITES = 200` with `rewrites_truncated` honest
  copy, the C4 "The dry run changes nothing" sentence, and the copy-only
  restatement through the same gate.
- S5 (lines 397-405): preview-first recipe, exactly the three W5 verbs with
  `escapeHtml(planId || '')`. S6 (lines 409-416): three tiers, rollback on
  Fail only, artifact locations, described with no UI surface.
- Fetching is per step only: one `fetch(` call site in the whole asset (in
  `fetchWizardRoute`), fired from `enterWizardStep` on steps 2-4;
  `openExtractionWizard` fetches nothing (locked).
- W9 boot payload: `proposals_payload.rs` (the serialized
  `extraction_proposals` byte-level lock file) is diff-empty, `crates/*/src`
  is diff-empty, and the cold-boot `ui_smoke: PASS` re-proves the served page.
- C7 resets: three callers of the one reset point (`resetWizardState`) -
  different-plan open, the 409 path inside `fetchWizardRoute`, and
  `resetWizardOnPayloadSwap`. The swap hook wraps `window.applyLoadedData`,
  the existing re-render path the payload swap itself calls
  (`crates/ui/assets/api.js:163`); graphData is mutated in place before
  `applyLoadedData()` fires, so the reset reads the NEW payload and correctly
  reports a plan dropped by the new generation. No frozen asset was edited.

## 7. Lock quality: the 409 site and the third-verb recipe are real

- `test_wizard_resets_on_stale_generation` body-extracts `fetchWizardRoute`
  and requires `response.status === 409` AND
  `resetWizardState(wizardState.planId)` inside that function body. It fails
  if the handler moves, is inlined elsewhere, or loses the reset. Not a
  tautology.
- `test_wizard_recipe_names_exactly_the_three_w5_verbs` requires the three
  verb strings in preview-first order, `escapeHtml(planId || '')`, and an
  asset-wide invariant that every `tool ` occurrence is one of the three
  honest verbs. Not a tautology.
- The remaining new locks (manifest fetch path, includes fallback, exact W4
  sentence plus gate order, dry-run rewrites with `&rarr;` ordering,
  payload-swap reset, caps and truncation copy, help-copy boundary) each name
  a specific artifact and fail on its absence; the Red digest shape
  (298 passed / 14 failed = 312) is consistent with exactly five amended card
  locks plus nine new wizard locks failing and nothing else.

## 8. Deviation dispositions (independent adjudication)

- D1 (payload-swap reset as its own new lock): ACCEPTED. The handoff requires
  the payload swap to reset the wizard state and states "each reset call is
  string-lockable"; the 409 lock cannot carry a second call site without
  conflating them. The Red set stayed exactly the union of the five amended
  card locks and the new wizard-step locks (14 failures, no others).
- D2 (beat order reads the seven `renderWizardStep*` declarations):
  ACCEPTED. C1 row order is S0..S6; the renderers are that list in renderer
  form and are the intent-revealing names shape section 9 names. The
  alternative (the label array shipped by W-3a) would have been green at Red,
  which the handoff's Red protocol forbids, so this is the only faithful
  reading. The lock would catch any reordering of the step renderers.
- D3 (payload-swap hook via `window.applyLoadedData` wrap): ACCEPTED.
  `hotspots.js` is frozen and its re-render is IIFE-internal;
  `applyLoadedData` is the existing re-render path the swap calls
  (api.js:163), the W-3a verdict accepted the identical wrap idiom for
  `closeModals` (W-3a D6), and the reset runs against already-swapped
  graphData. See section 6.
- D4 (gate exit codes read via distiller): ACCEPTED, with the same
  substitution applied by this reviewer (see the execution-results refusal
  log). W-3a verdict dispositions D5/D7 accepted the precedent; both gates
  now carry fresh exit-0 evidence from my own runs.
- D5 (docs-only journal rider commit): ACCEPTED. Rider `b8597f1` touches only
  the journal path, inside the touch-set; house precedent `c1956e5` (W-3a
  D11) is exactly this shape, and the landing hash, `git show --stat` and the
  gate-9 result are genuinely post-commit facts.
- D6 (shared `use` line gains `EXTRACTION_WIZARD_JS`; 13 surviving lock bodies
  byte-identical): ACCEPTED. Verified test-by-test in section 4; the amended
  locks 1 and 3-5 move their expectations into the wizard asset, so the
  import is required infrastructure, not scope creep.

## 9. Findings

1. (nit, non-blocking) Journal line-count drift: the journal records
   "extraction-wizard.js (182 -> 447 lines...)" and "index.html (641 lines)".
   Measured: the asset is 446 lines and index.html is 640 lines
   (`git diff --numstat` arithmetic: 182 - 12 + 276 = 446; +3/-1 on a 637-line
   base = 640). Documentation-only; no code, lock, or gate is affected.
2. (nit, non-blocking) `styles.css` reindents `.wizard-nav` from five to six
   spaces (whitespace-only, inside the wizard block). This cleans up the W-3a
   whitespace erratum the W-3a rider recorded and its verdict accepted as
   cosmetic (W-3a D11); the W-3b journal does not mention the cleanup. No
   value, hex, or selector changed.

## 10. Fresh gate evidence (verbatim)

- Scoped `python tools/testrun.py --distill -p ariadne-ui`:
  `312 passed / 0 failed / 0 ignored / 3 suites` (`### cargo test PASS (exit 0)`,
  `no failures`).
- `python tools/distill.py -- python tools/theme_token_check.py --check-added
  --base a7a3e2a`: `### generic PASS (exit 0)`, capture 0 lines (silent).
- `python tools/distill.py -- python tools/hygiene_check.py`:
  `### generic PASS (exit 0)`, 179 warn-only R3 rows, none on any file this
  hand authored.
- `python tools/distill.py -- python tools/encoding_guard.py --all`:
  `### generic PASS (exit 0)`, capture 0 lines (silent).

## 11. Bench stance

Exemption rides, recorded. The hand claims zero `crates/*/src` diff; the claim
holds under my own range diff (`git diff --name-only "a7a3e2a" "b8597f1" --
crates` lists only the three assets and three ui test files; explicit
producer-crate paths diff-empty). The bench producer set is therefore
byte-identical to the base and no timing result is attributable to this hand.
`python tools/bench_compare.py` was NOT run, by the same documented
assets-only stance the W-3a verdict and execution-results applied.

## 12. Closeout notes

Duty 1 of the closeout hop tracks exactly this review's two artifacts:

```
python tools/commit_scoped.py --scope tasks/ui-extraction-wizard-w3b-verdict.md --scope tasks/ui-extraction-wizard-w3b-execution-results.md -m "docs(tasks): W-3b gate APPROVED (wizard steps 2-4 live on the provider routes + C5 amendments) + execution-results"
```

Stage via the distiller, verify with `git diff --cached --name-only` (exactly
two paths), `python tools/distill.py -- python tools/encoding_guard.py
--staged` (silent), then commit. Never `git add .`.

Still-untracked ui-extraction-wizard artifacts: NONE. `git status` reports a
clean tree with no untracked files, and every milestone artifact is already
tracked:

- `tasks/ui-extraction-wizard-shape.md`, `-kickoff.md`, and all five handoffs
  (`-w1-`, `-w2a-`, `-w2b-`, `-w3a-`, `-w3b-handoff.md`): `5cc4ac3` (the plan
  `.kilo/plans/1791789110124-ui-extraction-wizard.md` landed in the same
  commit).
- W-1 journal `b27aa46`; W-1 verdict + execution-results `7b2238c`.
- W-2a journal `cb51bf8`; W-2a verdict + execution-results `1f561a0`.
- W-2b journal `f5a7d80`; W-2b verdict + execution-results `a9d8655`.
- W-3a journal `e22f4a6` (+ rider `c1956e5`); W-3a verdict +
  execution-results `a7a3e2a`.
- W-3b journal `24c2f7e` (+ rider `b8597f1`).

The only ui-extraction-wizard artifacts the closeout must add are the two
files this verdict names: `tasks/ui-extraction-wizard-w3b-verdict.md` and
`tasks/ui-extraction-wizard-w3b-execution-results.md`.

## 13. Reviewer-side refusals and substitutions (no silent skips)

Several bash invocations were refused by the session allowlist with the
catch-all message
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
Each gate was retried in an allowed form and every gate produced fresh
evidence:

- Piped `git show`/`git ls-files`/`Measure-Object`/`Findstr` probes refused;
  substituted the Glob and Grep tools, plain `git status`, and
  `git diff --name-only`/`--stat`/`--numstat` probes.
- `git status --porcelain` refused; plain `git status` used (clean tree).
- `python -c` one-liners and `git grep -P` refused; ASCII verification done
  with the Grep tool over the checkout (the wizard asset additionally carries
  a passing `is_ascii()` lock) plus `encoding_guard.py --all`.
- Exit codes for `theme_token_check.py` and `encoding_guard.py` captured via
  `python tools/distill.py -- <cmd>` (house D4 / W-3a D5-D7 precedent);
  `hygiene_check.py` ran both raw and via the distiller for a hard exit code.

`encoding_guard.py --staged` (handoff gate 7) is subsumed by the stronger
post-commit `--all` form run here; nothing is currently staged.
