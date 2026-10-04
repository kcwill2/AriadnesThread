# Kickoff - Extraction Card Story (UI readability)

Milestone key: `ui-extraction-card-story`. The ledger row (M-number and dates)
is assigned by the `milestone-lifecycle` kickoff; this artifact does not guess
a number. Artifacts: this kickoff, `tasks/ui-extraction-card-story-shape.md`
(the shape the planner sizes from).

## Intent

**Behavior change.** The "Proposed Extractions" card (shared builder
`renderExtractionProposalCard` in `crates/ui/assets/hotspots.js`, used by the
hotspot-panel section `renderProposalsSection` and the inspector tie-in
`renderInspectorProposalCard` in `crates/ui/assets/api.js`) must tell the
story of the extraction instead of dumping same-weight field rows. The story
order is fixed: **why -> what moves -> what it costs -> who is affected ->
preview**. Concretely:

1. WHY: the `problem` string becomes a visible one-line sentence in the card
   body. Today it is reachable only through the card's `title` tooltip -- the
   only text explaining the proposal is hidden behind hover. (The string is
   templated at the producer as `"{N} symbols of collateral load"`,
   `crates/cluster/src/planner.rs:659`, so the visible line plus the delta
   line must carry the explanation, not the raw fragment alone.)
2. WHAT MOVES: symbol names stop being a comma run-on; they render as chips
   with the `+N more` truncation marker kept.
3. WHAT IT COSTS: the `300 -> 40 collateral` delta gains an explaining label
   ("collateral = symbols every consumer drags") and a computed saving
   (`-260 (-87%)`), replacing bare jargon.
4. WHO IS AFFECTED: `consumer_tu_count` renders as "N consumer TU(s) need
   re-validation" (it is `validation_required_tus.len()`,
   `crates/cli/src/commands/ui.rs:534`), and `risk` renders as a tiered badge
   whose color carries meaning, with required headers as chips too.
5. IDENTITY: `plan_id` -- the payload's stable identity, never shown today --
   appears as a small monospace chip in the card header.
6. HIERARCHY: labels, values, and accents get distinct visual weights; the
   current design is six same-weight rows with an amber title only.

**Blast radius.** `crates/ui` frontend assets (`hotspots.js`, `styles.css`,
`index.html` help copy) and `crates/ui/tests/ui_test/`. Nothing else. No
Rust source, no DTO, no producer change.

**Why (the underlying user pain).** Verbatim complaint: "The proposed
extraction cards are hard to read and do not tell the story of the
extraction." A user scanning the hotspot panel cannot answer "what is wrong
with this header, what would the plan do, is it safe, what breaks, how do I
see it" without memorizing field abbreviations.

## Scope

In:

- `crates/ui/assets/hotspots.js`: rewrite `renderExtractionProposalCard`
  (+ small pure helpers in the same file) to the five-beat narrative; keep
  the function name, both call sites, the cache logic and the preview button
  markup of `renderProposalsSection` intact.
- `crates/ui/assets/styles.css`: restyle the `.extraction-*` block (approx.
  lines 990-1035) and add chip/label/badge classes, all token-based, all
  within the extraction block; do not restyle `.just-*` classes (shared with
  justification rows in `api.js`).
- `crates/ui/assets/index.html`: help-center copy only -- extend the Hotspots
  help list to define "collateral", the risk tiers, and the consumer-TU
  count in reader terms. No container/marker changes.
- Tests: add `crates/ui/tests/ui_test/extraction_card_story.rs` (+ one `mod`
  line in `main.rs`) for the new story invariants. Existing literals asserted
  by `static_html.rs` / `static_html_sections.rs` / `help_center_readability.rs`
  are KEPT (see shape contract C4), so existing tests are not edited.

Out (with reasons):

- `crates/ui/src/dto.rs` and the serde contract: every field the narrative
  needs already rides the payload (`plan_id`, `problem`, both loads, caps);
  the derived saving/percent is display arithmetic. The key-stability test
  `proposals_payload.rs::test_proposal_dto_key_names_are_stable` stays green
  untouched.
- `crates/cli/src/commands/ui.rs`, `crates/cluster/**`: producers. The
  templated `problem` string and the always-`Conservative` risk are honest
  display inputs; rewording them at the source would re-gate golden/parity
  tests for zero readability gain. The card copy must not claim more than the
  producer guarantees.
- `crates/ui/assets/preview.js`, `graph.js`: the preview contract and the E-I6
  line-budget asset stay byte-identical; the card button keeps its class,
  label, and data attributes.
- `themes.css`, `theme.js`, `tools/theme_token_baseline.json`: all needed
  tokens (`--green`, `--amber`, `--red`, `--cyan`, `--text-sub`,
  `--text-muted`, `--tile-bg`, `--border`, `--border-light`,
  `--bg-card-hover`) already exist in all three themes; zero new hex.
- Splitting `styles.css` (1656 lines, `.css` invisible to the R2 size gate --
  already a deferred ledger row): a separate operator decision; this
  milestone only bounds its own growth (see hygiene notes).

## Invariants (must hold; checkable forms in the shape)

1. Payload order (delta desc, plan_id asc) is rendered as-is -- the UI never
   re-sorts `extraction_proposals`.
2. Empty/absent payload hides the whole section (`proposals.length === 0`,
   default `|| []`) -- byte-identical hide behavior.
3. Preview button appears ONLY when `window.ariadnePreview.hasImpact(
   selectedRoot, target_file_id)`; its class `just-preview-btn`, label
   "Preview drop from selected file", and `data-preview-root` /
   `data-preview-source` attributes are unchanged; `bindPreviewControls`
   wiring unchanged.
4. One shared builder serves both surfaces; `graph.js` never gains the
   renderer and stays within its 990-line cap.
5. Every card-data interpolation passes through `escapeHtml`.
6. Zero added hex: styling uses `var(--token)` only.
7. Help-center locks survive: the phrase "Proposed extractions section" and
   every other copy lock in `help_center_readability.rs::
   test_legend_copy_locks_survive` remain, and every legend-tab prose block
   stays under 400 chars after the copy additions.
8. DTO frozen: `crates/ui/src/dto.rs` untouched; serde round-trip and
   key-name tests green unedited.
9. The five story beats are present in the card markup with the `problem`
   text in the body (not in any `title` attribute) and `plan_id` visible.

## Frozen locks inherited from the ledger

| Lock | Proof command |
|---|---|
| `tests/fixtures/**` byte-frozen (all branches) | touch-set check + `git_git_status` census at review |
| `tools/theme_token_baseline.json` regenerable only in justified commits | `python tools/theme_token_check.py --check-added --base <rev>` prints zero added hex (file untouched) |
| Mojibake/UTF-8 gate | `python tools/encoding_guard.py --staged` pre-commit, `--all` in CI |
| Hygiene (R1/R2 blocking) | `python tools/hygiene_check.py` exit 0 |
| Canonical test census | `python tools/testrun.py --distill -p ariadne-ui` (scoped); full census only at closeout |
| UI runtime smoke | `python tools/ui_smoke.py` -> PASS |
| Tool suites (untouched but adjacent) | `python -m unittest discover -s tools -p "*_tests.py"` at closeout |
| graph.js E-I6 cap | `static_html_sections.rs::test_extraction_impact_preview_overlay` asserts `<= 990` lines; file must not be touched at all |

## Acceptance gate (milestone as a whole)

1. Scoped suite: `python tools/testrun.py --distill -p ariadne-ui` ->
   `0 failed`, verbatim in the handoff report; includes ALL pre-existing
   card/section/help tests green WITHOUT editing their assertions.
2. New `extraction_card_story.rs` locks I5/I6(no-sort)/I9 of the shape and
   every existing test that today pins card literals (`static_html.rs::
   test_ui_extraction_proposal_cards`, `static_html_sections.rs::
   test_extraction_impact_preview_overlay`, `dto_builders.rs::
   test_standalone_export_embeds_extraction_proposals`, `proposals_payload.rs`)
   stays green unedited.
3. `python tools/theme_token_check.py --check-added --base <parent-rev>`
   silent on added lines (zero new hex; `themes.css` absent from the diff).
4. `python tools/hygiene_check.py` exit 0; `python tools/encoding_guard.py
   --staged` exit 0 at commit; authored content pure ASCII.
5. `python tools/ui_smoke.py` -> `ui_smoke: PASS`.
6. Touch set equals the scope list above exactly (review-gate
   commit-vs-touch-set enforcement).
7. Sizing: single handoff -- one crate, frontend assets + tests only, well
   under 10 gate invocations and ~1200 produced lines. No split needed.
