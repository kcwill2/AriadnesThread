# W-3b Handoff - ui-extraction-wizard: steps 2-4 live + honest help copy + C5 amendments

Depends on W-3a landed (the wizard asset, shell and steps 0-1) AND W-2a
landed (the provider: steps 2-4 fetch live data).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-extraction-wizard-shape.md` sections 2 (W1/W3/W4/W5/W6/W9), 3
   (C1 rows S2-S6, C3 error shapes, C4 dry-run claims, C5 amendment table,
   C6 volume rules, C7 state), 4 (auto-extract boundary); `tasks/ui-extraction-wizard-kickoff.md`
   sections 3, 5, 7 (clarity disposition - C5 is inherited from clarity T-B).
3. `crates/ui/assets/extraction-wizard.js` as W-3a landed it; the W-2b routes
   and DTOs; the W-2a provider behavior.
4. The C5 carrier files (read all five tests before amending any):
   `crates/ui/tests/ui_test/extraction_card_story.rs` and
   `crates/ui/tests/ui_test/extraction_card_playbook.rs`; the token site at
   `crates/ui/assets/styles.css` `.extraction-risk` base rule (verified at
   HEAD `97dd836`: `color: var(--text-sub);`, measured 2.97:1 in dark per C5).

## Objective and preserved invariants

Make steps S2 (per-file manifest), S3 (impact and risk) and S4 (dry run)
render live provider data with the documented caps and honest degradation;
wire the wizard reset paths; state the exact CLI recipe (S5) and what the
apply validates (S6); write the help copy that names the execution boundary;
and apply the five C5 card-lock amendments - the complete amendment table,
zero deletions, no amendment outside it.

Invariants this hand must preserve:
- W3: every `required_includes` entry rendered comes from the route payload;
  when `includes_unavailable` is true the renderer prints the honest fallback
  line, never a fabricated path.
- W4 copy-vs-move: step S3 (and S5) carry the exact locked sentence stating
  the C++ backend writes the new file and leaves the source declarations in
  place; the dry-run panel restates it when `copy_only` is true.
- W5 command honesty: only `tool apply <repo-root> --plan <id>`,
  `tool migrate <repo-root> --plan <id> --preview` and
  `tool transform extract <repo-root> --plan <id>` are named, with the plan
  id interpolated (the real flag surface; `plan` is a positional repo root,
  `--plan` is the flag).
- W6 staleness: a 409 response resets the wizard step state (C7); the reset
  call is string-locked.
- W9 / C6 volume: per-step fetches only; the boot payload is untouched
  (proved at W-2a); caps `EXTRACTION_MANIFEST_MAX_FILES = 64`,
  `EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE = 200`,
  `EXTRACTION_DRY_RUN_MAX_REWRITES = 200` with `files_truncated` /
  `rewrites_truncated` driving honest copy and the `just-more` `+K more` chip
  idiom; `symbol_count` carries the truth; no symbol body is ever rendered.
- W7: the 18 card-family locks survive EXCEPT the five C5 rows below; no lock
  is deleted; every amendment carries a rationale comment in its diff.
- Help copy states the execution boundary: the UI never mutates the
  repository in this version, and the follow-on milestone
  (`extraction-auto-apply`) is named as where execution will live.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/ui-extraction-wizard-w3b-journal.md (checkpoint journal)
- crates/ui/assets/extraction-wizard.js (steps S2-S4 fetch + render, the
  reset paths, S5 recipe, S6 description)
- crates/ui/assets/index.html (the help copy in the Hotspots help section)
- crates/ui/assets/styles.css (the C5 item-2 token swap in the
  `.extraction-risk` base rule plus the dry-run diff rules; net-capped)
- crates/ui/tests/ui_test/extraction_wizard_render.rs (extend with the S2-S6
  locks)
- crates/ui/tests/ui_test/extraction_card_story.rs (C5 items 1-2)
- crates/ui/tests/ui_test/extraction_card_playbook.rs (C5 items 3-5)

You MUST NOT touch: `crates/ui/src/**` (routes/state/DTOs are frozen after
W-2b), `crates/cli/**`, `tools/**`, `themes.css`, `theme.js`, other assets,
`tests/**`. No `git stash`, no branch change.

## What to build

1. Steps S2-S4 in `extraction-wizard.js`: fetch
   `/api/extraction/{plan_id}/manifest` and `/api/extraction/{plan_id}/dry-run`
   per step (C6: per-step fetching, never at boot); render the per-file
   manifest (symbol names under the cap + `symbol_count` + the includes list
   or the `includes_unavailable` fallback), the impact panel
   (`cycle_error`, consumer TU count, copy-vs-move sentence), and the dry-run
   panel (the new-file paths plus every consumer rewrite old -> new with the
   side-effect preservations, `rewrites_truncated` honest). Render directive
   spelling through the payload as the route returns it.
2. Reset paths (C7): 409 stale generation -> reset wizard state; payload swap
   (the existing re-render path) -> reset; opening the wizard for a different
   plan id -> reset. Each reset call is string-lockable.
3. S5/S6: the exact recipe with the interpolated plan id and the
   `--preview`-first step; S6 describes the three validation tiers, the
   rollback and where the artifacts land, with no UI surface.
4. Help copy in `index.html`: the wizard is read-only in this version; the UI
   never mutates the repository; the apply recipe is the path to execution;
   the follow-on `extraction-auto-apply` is named as the boundary.
5. The C5 amendments, exactly these five, each with a rationale comment in its
   diff:
   1. `extraction_card_story::test_beats_appear_in_story_order` - the beat
      list becomes the wizard-step order; it survives as an order lock over
      the wizard asset's step list (S0..S6 in C1 order).
   2. `extraction_card_story::test_risk_tier_badge_sanitized` - the base-rule
      token expectation becomes `var(--text-muted)`; the matching swap lands
      in `styles.css` `.extraction-risk` (measured 2.97:1 dark at HEAD).
   3. `extraction_card_playbook::test_file_plan_rows_markup` - the
      `class="extraction-file"` literal cannot coexist with the wizard's
      `.extraction-file-block`; the selector moves with the block into the
      wizard asset.
   4. `extraction_card_playbook::test_playbook_styles_declared_tokens` - the
      selector list is extended with the wizard rules.
   5. `extraction_card_playbook::test_apply_recipe_is_honest` - the verb list
      grows by `tool transform extract` only, and the "UI never applies"
      sentence is reworded to name the follow-on boundary.

## RGB protocol (per module, in this file order)

Red: extend `extraction_wizard_render.rs` with the S2-S6 locks (manifest
fetch path and render, includes fallback line, copy-vs-move sentence exact
text, dry-run rewrite rows, the three reset calls, cap + truncation chips,
the recipe verb list, the ASCII/escape discipline) AND amend the five C5 card
locks to their new expectations. Run Red before the wizard content exists.
Expected failures (verbatim shape): each new S2-S4 lock panics on the missing
markup or fetch call by name (`extraction-wizard.js must fetch the manifest
for the selected file`, the copy-vs-move sentence lock reports the missing
sentence, the reset lock reports the missing 409 handler, the recipe lock
reports the missing verb); each amended card lock fails on its new
expectation (the wizard-step order, `var(--text-muted)`,
`.extraction-file-block`, the extended selector list, the third verb).
Green: implement the wizard steps, help copy and style edits; re-run until
exit 0.
Refactor: keep each renderer small and the fetch/reset handling in one place;
every function under 100 lines.

## Required tests (extends)

In `extraction_wizard_render.rs`:
1. Lock: S2 fetches `/api/extraction/` + plan id + `/manifest` per step (never
   at boot; no manifest fetch in the boot path).
2. Lock: the includes fallback line exists and is rendered when
   `includes_unavailable` (W3); no include path is hardcoded anywhere.
3. Lock: the exact copy-vs-move sentence (W4; the sentence text is part of
   the contract - read shape section 2 W4 for its required claims).
4. Lock: S4 fetches the dry-run route and renders rewrite rows with the
   `&rarr;` entity between old and new include sets.
5. Lock: the 409 stale-generation response resets the wizard state; the reset
   function is called from the fetch error path (C7).
6. Lock: caps - the per-file list renders at most
   `EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE` rows with the `+K more` chip
   and `symbol_count` beside them; the rewrites list honors
   `EXTRACTION_DRY_RUN_MAX_REWRITES` with `rewrites_truncated` copy.
7. Lock: recipe verb list is exactly the three W5 verbs with the plan id
   interpolated; no fourth verb string exists in the asset.
8. Lock: the help copy names the execution boundary and the follow-on
   milestone key; the asset stays ASCII-only with `escapeHtml` on every
   interpolated value.

In the card files: the five C5 amendments as listed, byte-exact in their new
expectations, rationale comment per amendment.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names, no
single-letter locals, intention-revealing names. `extraction-wizard.js` must
stay under 1000 lines after this hand (W-3a sized the shell to leave the
room). `python tools/hygiene_check.py` exit 0.

## Frozen locks

- Everything not in the C5 table: the other 13 card-family locks byte-identical
  (W7; reviewer diffs both card test files per commit).
- `proposals_payload.rs`, `static_html.rs`, `static_html_sections.rs`,
  `help_center_readability.rs`, `export_order.rs`, `dto_builders.rs`,
  `theme_contract.rs`, `theme_dark_grey_palette.rs`, `theme_js_zero_hex.rs`,
  `extraction_contrast.rs`, `preview_hypothetical.rs`, `icon_asset.rs`,
  `asset_injection_hardening.rs` byte-identical.
- `preview.js`, `api.js`, `graph.js` byte-identical; `theme.js`,
  `themes.css`, `tools/theme_token_baseline.json` byte-identical (the token
  swap changes a VALUE reference only - no hex literal is added or removed,
  so the added-scan and the baseline both hold).
- `tests/fixtures/**`, `tests/golden/**` frozen.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-ui` (baseline canonical line)
2. `python tools/testrun.py --distill -p ariadne-ui` (Red: exit 1; named
   failing tests verbatim, including the five amended locks)
3. `python tools/testrun.py --distill -p ariadne-ui` (Green: exit 0;
   canonical line = baseline, no new test count - the count only moves if an
   assertion is added)
4. `python tools/theme_token_check.py --check-added --base <start-rev>`
   (silent; combined form is canonical)
5. `python tools/hygiene_check.py` (exit 0)
6. `python tools/distill.py -- python tools/ui_smoke.py` (exit 0:
   `ui_smoke: PASS` - the help-copy edit changed the served page; boot proof)
7. `python tools/encoding_guard.py --staged` (exit 0)
8. `python tools/commit_scoped.py --scope crates/ui/assets/extraction-wizard.js
   --scope crates/ui/assets/index.html --scope crates/ui/assets/styles.css
   --scope crates/ui/tests/ui_test/extraction_wizard_render.rs --scope
   crates/ui/tests/ui_test/extraction_card_story.rs --scope
   crates/ui/tests/ui_test/extraction_card_playbook.rs --scope
   tasks/ui-extraction-wizard-w3b-journal.md -m "feat(ui): wizard steps 2-4
   live on the provider routes + execution-boundary help copy"` (never
   `git add .`)
9. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 9 (commit-cycle trio is gates 7-9, itemized per the M28 T0b
finding-1 journal law). This hand touches `crates/ui/assets` and ui tests
only: zero `crates/*/src` diff, so the bench gate rides the documented
assets-only stance, proven by an empty `crates/*/src` range diff in the
report.

## Discipline clauses

- UTF-8: Write/Edit tools only; pure-ASCII authored content; no em dashes;
  JS glyphs as `&rarr;` / `&#8592;` entities.
- Touch only the touch-set; out-of-scope breakage: report, do not fix. The C5
  table is COMPLETE: any further card-lock edit discovered mid-hand is a
  STOP-and-report, not an amendment.
- Bare high-volume commands are a reviewer blocker: distilling entry points
  only.
- Checkpoint the journal after each Green gate.
- Context exhaustion: STOP with a disk-state inventory; successor resumes via
  `python tools/checkpoint_guard.py resume --spec
  tasks/ui-extraction-wizard-w3b-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (named failing tests,
  verbatim panic lines). 3. Green evidence (digest header + canonical line +
  capture path; the ui_smoke PASS line). 4. Deviations + reasons. 5.
Out-of-scope observations (report-only). No raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly.
