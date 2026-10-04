# Shape - Extraction Card Story (UI readability)

Companion to `tasks/ui-extraction-card-story-kickoff.md`. This is the shape,
not a task list; the planner sizes handoffs from the split seam at the end
(there is one handoff: assets + tests, no seam worth splitting).

## Rendering reality (what the card is today)

`renderProposalsSection` (hotspots.js) and `renderInspectorProposalCard`
(api.js) both feed `graphData.extraction_proposals` items into the single
shared `renderExtractionProposalCard` (hotspots.js). Today's card is six
same-weight rows: an amber title sentence, `300 -> 40 collateral`, a comma
blob of symbol names, a needs row, two bare tags (`23 TU(s)`,
`Conservative`), and a conditional preview button. The `problem` string --
the only why-text -- rides solely in the card's `title` attribute.
`plan_id` is never shown. All payload caps are upstream (names <= 12,
headers <= 8, proposals <= 12 per payload), so the card content is bounded
and the truncation is inferable from `symbol_names.length < total_symbols`.

Producer semantics that bound honest copy: `problem` is templated
`"{collateral_load} symbols of collateral load"` (planner.rs:659); `risk` is
always `Conservative` today (planner.rs:664; tiers Conservative/Moderate/
Aggressive exist in `RiskLevel`); `consumer_tu_count` is
`validation_required_tus.len()` (cli/src/commands/ui.rs:534), i.e. consumer
TUs that must be re-validated after applying the plan.

## Card narrative (the chosen story structure)

Five beats top-to-bottom, each with an uppercase muted label and distinct
weight; the preview beat already exists and stays byte-identical:

1. HEAD (identity): `plan_id` monospace chip + target file title (strongest
   weight, amber). Click selects the target (unchanged `selectHotspotRow`).
2. WHY (visible): `extraction-problem` line rendering `escapeHtml(p.problem)`
   in the body, prefixed by the `Why` label; removed from the `title`
   tooltip (title becomes the target path).
3. WHAT IT COSTS / SAVES: labeled delta `300 &rarr; 40` plus a computed
   saving chip `-260 (-87%)` in the green token, guarded on `before > 0`.
4. WHAT MOVES: `Extract (total_symbols)` + wrapped name chips + `+N more`
   (`just-more` reused); then the locked-label needs row `extracted symbols
   need` with header chips (or the italic no-additional-headers fallback).
5. WHO IS AFFECTED: `N consumer TU(s) need re-validation` + tiered risk
   badge `Risk: <tier>` colored by tier.
6. PREVIEW: existing button, unchanged markup.

## Invariants (numbered, each independently checkable)

S1 **Why is in the body.** The builder interpolates `p.problem` through
`escapeHtml` into an `.extraction-problem` element and no longer puts
`p.problem` into any `title` attribute. Locked by: new
`crates/ui/tests/ui_test/extraction_card_story.rs` source asserts
(`HOTSPOTS_JS.contains("extraction-problem")`, contains `escapeHtml(p.problem`,
and NOT contains the old `title="${escapeHtml(p.problem` pattern). No test
locks this today.

S2 **Identity is visible.** The builder renders `p.plan_id` (escaped) inside
an `.extraction-plan-id` span. Locked by: new test in the same file. No test
locks this today.

S3 **Delta is a sentence, not jargon.** The delta line keeps
`collateral_load_before`/`collateral_load_after` (existing lock in
`static_html.rs::test_ui_extraction_proposal_cards`) and adds a saving
derived only from those two fields, with an explicit zero-guard on
`before` (no saving chip, no division) and no negative percentages. Locked
by: new test asserting the guard expression and `Math.round` percent exist;
string-level like its siblings.

S4 **No run-on blobs.** Names and headers render as `.extraction-chip`
spans; `hotspots.js` contains NO `join(', ')` anywhere after the change
(the pattern exists today only in the card builder, verified). Locked by:
new test `!HOTSPOTS_JS.contains(".join(', ')")`. Falsifiable, survives as a
no-regression guard.

S5 **Risk has semantics.** `renderExtractionRiskTag` emits
`extraction-risk extraction-risk-<tier>` where `<tier>` is
`risk.toLowerCase()` filtered to `[a-z]` (unknown values degrade to the
neutral base class, no raw-string class injection). styles.css defines the
three tier rules with `var(--green)` / `var(--amber)` / `var(--red)` text
and the base rule with `var(--text-sub)`. Locked by: new test using the
`css_rule_body` idiom (same one as `gui_scrollbar_style.rs` /
`help_center_readability.rs`) for each tier rule + the sanitizer assert.

S6 **Order is display-faithful.** The section still maps proposals straight
into the builder; `hotspots.js` contains no `.sort(` over proposals
(assert `proposals.map(renderExtractionProposalCard)` substring kept, and no
`extraction_proposals.sort` substring). Locks the inherited payload order
(delta desc, plan_id asc).

S7 **Empty-hide survives untouched.** The `graphData.extraction_proposals
|| []` and `proposals.length === 0` literals stay exactly as-is; existing
`static_html.rs` I4 assertions pass unedited.

S8 **Preview contract is byte-faithful.** Button keeps class
`just-preview-btn`, label `Preview drop from selected file`, both
`data-preview-*` attributes, `bindPreviewControls` call sites unchanged;
`preview.js` and `api.js` are absent from the touch set (review-gate
touch-set check + `static_html_sections.rs::
test_extraction_impact_preview_overlay` passing unedited).

S9 **Zero added hex.** Styling adds only `var(--token)` usages from the
existing three-theme token set; `themes.css`/`theme.js`/baseline untouched.
Oracle: `python tools/theme_token_check.py --check-added --base <parent>`
prints no added hex.

S10 **Escape discipline holds.** `p.problem`, `p.risk`, `p.plan_id`, each
header and each symbol name appear inside `escapeHtml(...)` in the builder
helpers (S1/S2/S5 asserts plus a chips-helper assert). Numeric ids and the
existing `selectHotspotRow('...')` escaping pattern are kept as-is.

S11 **Help center stays readable and locked.** Hotspots help gains at most
three new `<li>` blocks inside the existing list, each under 400 stripped
chars; the copy lock `Proposed extractions section` and all sibling locks
survive. Locked by: existing `help_center_readability.rs::
test_legend_tab_has_no_wall_of_prose` + `test_legend_copy_locks_survive`,
unedited.

S12 **DTO/serde frozen.** `crates/ui/src/dto.rs` absent from the touch set;
`proposals_payload.rs` (round-trip, key names, absent-when-empty) and
`dto_builders.rs` (standalone blob carries the renderer + data) pass
unedited.

## Contracts

C1 **Builder signature.** `renderExtractionProposalCard(p)` stays global in
`hotspots.js`, returns a single `.extraction-card` root element string; both
call sites inject it into their container and then run
`bindPreviewControls(container)`. Function name is load-bearing: the
standalone export blob test greps for `function renderExtractionProposalCard(`.

C2 **Preview descendant.** Any card DOM must expose its preview control as
`button.just-preview-btn[data-preview-root][data-preview-source]` with the
locked label; nothing else in the card may carry `.just-preview-btn`.

C3 **Markup skeleton** (class inventory; all new classes live under the
`.extraction-*` namespace so the shared `.just-*` classes used by
justification rows are never restyled):

```
<div class="extraction-card" onclick="selectHotspotRow('<id>')" title="<path>">
  <div class="extraction-head">
    <span class="extraction-plan-id">PLAN-007</span>
    <span class="extraction-title">graph.h</span>
  </div>
  <div class="extraction-problem"><span class="extraction-label">Why</span>300 symbols of collateral load</div>
  <div class="extraction-cost">
    <span class="extraction-label">Collateral drop</span>
    <span class="extraction-delta">300 &rarr; 40</span>
    <span class="extraction-saving">-260 (-87%)</span>   <!-- only when before > 0 -->
  </div>
  <div class="extraction-row">
    <span class="extraction-label">Extract (12)</span>
    <span class="extraction-chips"><span class="extraction-chip">Thing</span>...
      <span class="just-more">+3 more</span></span>
  </div>
  <div class="extraction-row">
    <span class="extraction-label">extracted symbols need</span>
    <span class="extraction-chips">...chips... | <em>no additional headers</em></span>
  </div>
  <div class="extraction-row">
    <span class="extraction-label">Impact</span>
    <span class="extraction-tu">23 consumer TU(s) need re-validation</span>
    <span class="extraction-risk extraction-risk-conservative">Risk: Conservative</span>
  </div>
  <div class="extraction-row">[preview button, unchanged]</div>
</div>
```

Helper decomposition (all trivially under the 100-line function rule):
`extractionChips(items)`, `extractionSavingText(before, after)`,
`renderExtractionRiskTag(risk)`; the main builder stays under ~70 lines.

C4 **Keep-literal list** (existing tests grep these; the rewrite must keep
each substring verbatim, so NO existing test is edited):

- `hotspots.js`: `function renderExtractionProposalCard(`,
  `collateral_load_before`, `collateral_load_after`, `consumer_tu_count`,
  `extraction-row`, `extracted symbols need`, `just-more`,
  `selectHotspotRow('`, `graphData.extraction_proposals || []`,
  `proposals.length === 0`, `Preview drop from selected file`.
- `api.js` (untouched): `graphData.extraction_proposals =`,
  `target_file_id`, `just-preview-btn`, `Preview after extraction`.
- `index.html`: ids `proposedExtractionsSection` /
  `proposedExtractionsList` / `inspProposalSection` / `inspProposalCard`;
  section placed after `hotspotList`; help phrases `Proposed extractions
  section` and `Preview after extraction`.
- `graph.js`: must NOT contain `renderExtractionProposalCard`; stays
  `<= 990` lines (best: untouched).

C5 **Token map for new styling** (all tokens exist in all three themes;
zero additions): `.extraction-card` keeps `--tile-bg`/`--border`/`--amber`
left spine; `.extraction-plan-id` `--text-muted` on `--bg-card-hover`-style
chip background using `--tile-bg` + `--border-light`; `.extraction-title`
keeps `--amber` 700 weight, 12px; `.extraction-label` 9px uppercase
`--text-sub` (mirrors `.just-tag` typography but is a NEW class);
`.extraction-delta` `--text-sub` to `--text` value emphasis;
`.extraction-saving` `--green`; `.extraction-chip` monospace 10px with
`--border-light` outline on tile background; `.extraction-tu`
`--text-sub`; `.extraction-risk` base `--text-sub`, tiers `--green` /
`--amber` / `--red`.

C6 **Honest copy strings.** Saving text is derived (`-D (-P%)`); TU text is
`N consumer TU(s) need re-validation`; risk text is `Risk: <tier>`; help
copy may say the planner's proposals are all Conservative today (true at
planner.rs:664) and must describe tiers as safety ordering only. The word
"collateral" is defined once in help (symbols every consumer drags along)
and never invented anew on the card.

## Module map

| File | Role |
|---|---|
| `crates/ui/assets/hotspots.js` | CHANGED: builder rewrite + 3 helpers; `renderProposalsSection` cache logic untouched |
| `crates/ui/assets/styles.css` | CHANGED: `.extraction-*` block restyle + new classes, contiguous, token-only |
| `crates/ui/assets/index.html` | CHANGED: Hotspots help list gains <= 3 `<li>` |
| `crates/ui/tests/ui_test/extraction_card_story.rs` | NEW: locks S1-S6/S10 |
| `crates/ui/tests/ui_test/main.rs` | CHANGED: one `mod` line |
| `api.js`, `preview.js`, `graph.js`, `main.js`, `themes.css`, `theme.js` | MUST NOT change (byte-identical) |
| `crates/ui/src/**`, `crates/cli/**`, `crates/cluster/**`, `tools/**`, `tests/fixtures/**`, `theme_token_baseline.json` | MUST NOT change |

## Risks (each with named mitigation + oracle)

- **Card grows taller inside the 240px-scrolling list.** Content is capped
  upstream (12 names, 8 headers, 12 proposals), problem is one templated
  short line; chips wrap at 10px font. Oracle: `ui_smoke` boots the panel
  routes; visual check is the reviewer/human step (no height-locked test
  exists -- stated plainly).
- **Collateral damage to justification rows via `.just-*`.** Mitigation:
  new `.extraction-*` namespace only; `.just-tag`/`.just-collateral`/
  `.just-more` rules byte-identical (`.just-more` is reused, not restyled).
  Oracle: diff of styles.css hunks confined to the extraction block +
  `justification_*` tests unedited and green.
- **Copy overclaims risk/TU semantics.** Mitigation: contract C6 pins the
  wording to producer-verifiable facts; reviewer checks claims against
  planner.rs/ui.rs lines cited here.
- **Percent math on degenerate payloads (before = 0 or after > before).**
  Mitigation: S3 zero-guard; after > before shows the raw delta without the
  saving chip. Oracle: new test on `extractionSavingText` source guard.
- **styles.css ungated growth (1656 lines today; R2 skips `.css` per the
  deferred ledger row).** Mitigation: added CSS budget <= ~90 lines in one
  contiguous block; the css-extraction decision stays the ledger's separate
  deferred row -- explicitly out of scope, with the reason recorded in the
  kickoff.
- **Standalone export regression.** Mitigation: C1 keeps the function name;
  `dto_builders.rs` blob tests unedited; oracle is that suite.

## Hygiene notes

- `hotspots.js` 350 -> approx. 430-470 lines: far under the 1000 ceiling;
  builder and helpers stay well under 100 lines each.
- `styles.css` 1656 lines, no size gate: see risk above; growth bounded.
- `index.html` help edits are short `<li>` additions; keep every authored
  line ASCII, arrows as `&rarr;` entities, no line near 2000 chars.
- New test file follows the string-assertion idiom of its siblings
  (`ariadne_ui::HOTSPOTS_JS.contains(...)`, `css_rule_body` copy), which is
  the house oracle style for asset markup in this repo.
