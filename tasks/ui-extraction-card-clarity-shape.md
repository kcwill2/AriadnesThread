# Milestone Shape — ui-extraction-card-clarity

Companion to `tasks/ui-extraction-card-clarity-kickoff.md` (key verbatim there).
Read-only survey at HEAD; the ratios in this file are HAND-COMPUTED with the
WCAG formula (no bash in the shaping session; tolerance ~ +/-0.15). The oracle
of contract C4 is the authority; T-A's first RED run replaces this table with
measured values in its journal.

## 1. Invariants (each independently checkable)

- I1 Disclosure model intact: `class="extraction-details" hidden>` +
  `aria-expanded="false"` + `toggleExtractionCard(event, this)` contract stay;
  oracle = `extraction_card_playbook::test_details_collapsed_by_default` and
  `test_disclosure_button_contract` green UNEDITED.
- I2 Body ink on cards: for every theme, ratio(--text-muted, --bg-card) >= 4.5
  and ratio(--text-main, --bg-card) >= 7.0. Oracle = `extraction_contrast.rs`
  (new, T-A).
- I3 On-card ink tier ban: no `.extraction-*` styles.css rule that paints the
  card surface may declare `color: var(--text-sub)` (measured 2.97:1 on
  --bg-card in dark — the "cloudy" mechanism). Oracle = clarity test
  css_rule_body assertion (T-B).
- I4 Mass floor: for every theme,
  ratio(--bg-card, --bg-surface) >= 1.12 AND
  ratio(--border-light, --bg-card) >= 1.35 AND
  ratio(--border-light, --bg-surface) >= 1.50.
  Oracle = `extraction_contrast.rs`.
- I5 Per-file story: detail renders one `.extraction-file-block` per
  `target_files` entry, payload order, heading `code.extraction-file-path`
  (escaped), symbols only via `extractionChips(t.symbol_names)`, `+K more` and
  "more files not listed" and `<em>no file plan in payload</em>` fallbacks
  intact. Oracle = `renderExtractionFilePlan` body lock + new clarity block
  order lock + `test_file_plan_rows_markup` (survives via frozen names/literals).
- I6 Commands: only `tool apply` and `tool migrate` named; the verb-count
  equality assert stays. Oracle = `test_apply_recipe_is_honest` UNEDITED.
- I7 No new data: `renderExtractionProposalCard` reads payload fields only;
  DTO byte-identical (no `crates/*/src` diff). Oracle = `test_no_new_data_sources`
  UNEDITED + `proposals_payload.rs` green UNEDITED + empty-diff proof.
- I8 Summary is exactly the five scan fields: plan id, target path, N symbols,
  guarded saving, risk badge. Oracle = clarity test on
  `renderExtractionProposalCard` body (five class tokens, nothing else visible)
  + `test_summary_meta_and_risk_visible` UNEDITED.
- I9 Hex discipline: zero new hex in rule files; `themes.css` edits change
  VALUES only (baseline JSON diff-empty; count freeze unchanged). Oracle =
  `theme_token_check --check-added` silent + `theme_contract` tests green.
- I10 Order: `proposals.map(renderExtractionProposalCard)` payload order, no
  re-sort. Oracle = `test_order_and_preview_locks` UNEDITED.
- I11 No comma blobs; chips only. Oracle = `test_no_runon_join_blobs` UNEDITED.
- I12 Escape discipline: every payload string through `escapeHtml` /
  `extractionChips`; plan id escaped before command interpolation. Oracle =
  `test_playbook_escape_discipline` UNEDITED.
- I13 Four-theme coverage: the contrast oracle iterates the `THEMES` registry
  from theme.js (parsed, never hardcoded) — a fifth theme added later is forced
  into the same pair table automatically. Oracle = `extraction_contrast.rs`
  panics on any registry id without a themes.css block; a missing pair row is a
  test bug by construction (table loops the registry).

## 2. Contracts

### C1 Function-name freeze (survival mechanism for existing string locks)

In `hotspots.js` keep every name and locked substring exactly:
`toggleExtractionCard`, `extractionChips`, `extractionSavingText`,
`renderExtractionRiskTag`, `extractionSummaryMeta`, `renderExtractionFilePlan`
(renamed semantics: now emits BLOCKS; keeps signature `(targetFiles, truncated)`
and locked substrings `escapeHtml(t.path`, `extractionChips(t.symbol_names)`,
`Files to create (${files.length})`, `more files`, fallback `<em>`),
`renderExtractionApplyHint` (markup byte-faithful),
`renderExtractionProposalCard`, `renderProposalsSection`,
`proposals.map(renderExtractionProposalCard)`, `before > 0`, `Math.round`,
`100 / before`, `consumer TU(s) need re-validation`, `toLowerCase`, `[^a-z]`,
`getElementById`-free toggle. The 4-space top-level function indent inside the
IIFE must not change (playbook `function_body` terminates on `"\n    }"`).

### C2 Detail markup (new story shape)

```
.extraction-details (hidden)
  .extraction-problem            one-line why (escaped p.problem)      [kept]
  .extraction-file-blocks        intro "Files to create (N)"            [group]
    .extraction-file-block       per target file, payload order
      code.extraction-file-path  heading = real path (escaped)
      .extraction-file-provides  one line: "Provides N symbols moved out of
                                 <target_file>" (derived text, escaped parts)
      .extraction-chips          symbol chips + optional +K more
  .extraction-footnote           stats demoted to one strip:
      .extraction-cost           Collateral drop before -> after + saving
      .extraction-tu             "N consumer TU(s) need re-validation"
      required-headers row       "New file includes" chips (was: second chip row)
  .extraction-apply              commands block, markup UNCHANGED (P9)
```

Disposition of old stacked beats, per item (user verdict licenses this):
- Why: KEPT as the one `.extraction-problem` line (problem text is card-level
  templated; a per-file "why" field does not exist and is scope-out).
- Collateral drop / cost: DEMOTED to footnote strip (scan decision is made by
  the summary saving; the before->after detail is verification material).
- Extract chips: MOVED into the per-file blocks (that is exactly the demanded
  "which symbols").
- "extracted symbols need" (required headers): DEMOTED to footnote row — it
  describes what the new file includes, i.e. still instruction, but after the
  per-file story, not competing with it.
- Impact/consumer-TU line: DEMOTED to footnote strip (re-validation cost
  belongs beside the command that triggers it).
- Apply hint: KEPT last (user: "commands after").

### C3 Token-choice map (styles.css; locked rule-body substrings)

| rule | tokens (all existing; zero new hex) |
|---|---|
| .extraction-card | background `var(--bg-card)` (opaque, replaces `--tile-bg`), border `var(--border-light)`, 3px left spine `var(--amber)` kept, padding `10px 12px` |
| .extraction-card:hover | `var(--bg-card-hover)` |
| .extraction-title | `var(--text-main)`, 13px/700 (amber demoted to spine-only accent) |
| .extraction-details | `color: var(--text-muted)`, `line-height: 1.45`, margin/padding-top 10px (NO `--text-sub`, I3) |
| .extraction-label | 10px `var(--text-muted)` (was 9px `--text-sub`) |
| .extraction-file-path, .extraction-command | background `var(--bg-sub)`, border `var(--border-light)`, color `var(--code-text)`, 12px |
| .extraction-chip | background `var(--bg-card-hover)`, border `var(--border-light)`, color inherited body ink |
| .extraction-footnote | border-top 1px `var(--border)`, 10px `var(--text-muted)` |
| .extraction-list | gap 10px (was 6px) |
| section header (.link-list-title in #proposedExtractionsSection scope) | border-bottom 1px `var(--border)`, padding-bottom 8px; list margin-top 10px |

Whitespace/type scale (concrete, not vibes): card padding 10x12; block gap 12px;
heading 13px; body/contents 11px; footnote 10px; chip 10px; section gap 10px.
Panel is 520px wide — density target is 4-line collapsed card, ~4 blocks max
before truncation marker.

### C4 Contrast oracle (`crates/ui/tests/ui_test/extraction_contrast.rs`, T-A)

Parse `THEMES` from theme.js (reuse the copied-helper idiom of
`theme_dark_grey_palette.rs`: strip comments, block body, declared value,
hex runs, WCAG luminance + ratio). Translucent declarations composite over
`--bg-surface` (the hotspot panel; per-channel `a*c + (1-a)*under`); opaque
values pass through. Iterate every registry id; evaluate the pair table:

| pair | floor | class |
|---|---|---|
| --text-main on --bg-card / --bg-sub | 7.0 | body |
| --text-muted on --bg-card / --bg-sub / --bg-surface / --bg-main | 4.5 | body |
| --text-sub on --bg-main | 3.0 | UI/off-card |
| --amber on --bg-card | 4.5 | card body ink (12px bold) |
| --green on --bg-card | 4.5 | verdict-on-card |
| --red-text on --bg-card | 4.5 | verdict-on-card |
| ratio(|--bg-card, --bg-surface|) | 1.12 | surface separation |
| ratio(--border-light, --bg-card) | 1.35 | border separation |
| ratio(--border-light, --bg-surface) | 1.50 | border separation |

The 1.12 background floor is deliberately modest: light-theme luminance
underestimates perceived separation of white-vs-pale surfaces; the two border
floors carry separation, while 1.12 catches collapse (today's dark composite
measures ~1.11 and REDs — good falsifier).

### C5 Named muddy pairs (hand-computed, approximate; oracle is authority)

Current effective card surface = --tile-bg over --bg-surface:
dark rgb(25,34,51), light rgb(235,239,245), grey rgb(39,39,39),
steampunk rgb(52,40,24).

| # | pair (dark / light / grey / steampunk) | measured now | verdict |
|---|---|---|---|
| M1 | card-surface vs panel (mass index) | 1.11 / 1.15 / 1.14 / 1.20 | all at/below the 1.12 floor — the "one gray mass", mechanically |
| M2 | --border vs card surface | 1.20 / 1.29 / 1.42 / 1.32 | invisible seam in all four |
| M3 | --text-sub (ALL detail body + labels) on card | 3.35 / 5.52 / 4.33 / 3.83 | FAIL 4.5 in dark, grey, steampunk — the "cloudy" text |
| M4 | chip bg vs card bg (both --tile-bg) | 1.00 all themes | chips invisible by construction |
| M5 | --green saving on card (light) | ~4.35 | FAIL (dark/grey/steampunk pass ~9) |
| M6 | --text-muted on --bg-card-hover (plan-id chip) | 4.83 / 5.11 / 4.51 / 5.35 | pass, grey borderline — keep, lock against regression |

Projected after C3 remap (opaque --bg-card etc.): --text-muted on --bg-card
5.5/6.2/5.4/6.4 PASS; separation 1.25/1.13/1.25/1.23 and borders
1.37/2.08/1.74/1.96 and 1.71/2.56/2.18/2.41 — all floors hold in every theme.

### C6 Token fix list for themes.css (driven by C4, expected minimal)

- Expected single value fix: light `--green` deepened (candidate `#166534`,
  ~5.7:1 on light --bg-card; fixes M5 and improves every green-on-card surface).
  Justification class: contrast-only retint, hue preserved (D6 allows).
  Fallback if the reviewer rejects touching a semantic token: keep the token,
  restyle the saving line as `--text-strong` with green as accent only —
  decided at T-A review, oracle decides otherwise.
- Expected empty for dark/grey/steampunk values (grey further confined by the
  achromatic spread<=4 and byte-equal-to-default locks; any grey value edit is
  presumptively wrong).
- Value edits only: literal COUNT unchanged => `theme_token_baseline.json`
  stays diff-empty (byte-locked set protocol).

### C7 Test amendments (exact; 3 amended, 13 survive byte-identical, 0 killed)

Amended (T-B, each with its justification comment in the diff):
1. `extraction_card_story::test_beats_appear_in_story_order` — beats list
   becomes `["extraction-head", "extraction-problem", "extraction-file-block",
   "extraction-chips", "extraction-cost", "extraction-tu"]` (footnote after the
   blocks). Dies because the user mandate reorders cost below the per-file
   story; survives as an order lock in the new canonical order.
2. `extraction_card_story::test_risk_tier_badge_sanitized` — base-rule
   assertion `var(--text-sub)` -> `var(--text-muted)` (unknown-tier degradation
   now paints the card surface; --text-sub there is 2.97:1). Tier color
   assertions (--green/--amber/--red) unchanged.
3. `extraction_card_playbook::test_playbook_styles_declared_tokens` — selector
   list extended with `.extraction-file-block {`, `.extraction-footnote {`,
   `.extraction-file-provides {`; per-selector assertions unchanged.
Survive UNEDITED (byte-identical, verified per-commit by reviewer): story
`test_why_beat_visible_in_body`, `test_plan_id_identity_chip`,
`test_saving_computed_with_zero_guard`, `test_no_runon_join_blobs`,
`test_names_and_headers_render_as_chips`, `test_order_and_preview_locks`,
`test_tu_sentence_and_help_copy`; playbook
`test_details_collapsed_by_default`, `test_disclosure_button_contract`,
`test_summary_meta_and_risk_visible`, `test_file_plan_rows_markup`,
`test_apply_recipe_is_honest`, `test_playbook_escape_discipline`,
`test_no_new_data_sources`, `test_help_playbook_copy`.
Everything outside the two files is untouched: `help_center_readability.rs`,
`static_html*.rs`, `proposals_payload.rs`, `theme_contract.rs`,
`theme_dark_grey_palette.rs`, `theme_js_zero_hex.rs` (and the locked index.html
copy already matches the new design: "collapsed summary", "lists the suggested
destination files with the symbols each should hold, plus the exact tool apply
command").
New: `extraction_card_clarity.rs` (~8-10 locks: block order per C2, five-field
summary per I8, C3 token-choice rule bodies, I3 --text-sub ban, whitespace
scale substrings `gap: 10px` / `padding: 10px 12px`, section-chrome rule).
`main.rs`: add `mod extraction_card_clarity;` immediately BEFORE
`mod extraction_card_playbook;` (true alphabetical AND consistent with the
current list order; do NOT "fix" the known mod-order erratum in this hop —
ledger row `ui-extraction-card-playbook` erratum stays fix-at-next-touch).

## 3. Module map

Change: `crates/ui/assets/themes.css` (T-A, value edits only),
`crates/ui/tests/ui_test/extraction_contrast.rs` (new, T-A),
`crates/ui/tests/ui_test/main.rs` (+2 mod lines total across T-A/T-B),
`crates/ui/assets/hotspots.js` (T-B), `crates/ui/assets/styles.css` (T-B,
extraction block lines ~988-1177 only), `extraction_card_story.rs` +
`extraction_card_playbook.rs` (amendments per C7), `extraction_card_clarity.rs`
(new, T-B).
Must-not-touch: index.html, api.js, preview.js, graph.js, main.js, theme.js,
file-view.js, provenance.js, crates/ui/src/**, all other crates (DTO/producer
frozen), tools/** (incl. theme_token_check.py + baseline JSON),
theme_dark_grey_palette.rs, help_center_readability.rs, tests/fixtures/**.

## 4. Risks -> named oracle/mitigation

- R1 Token-pairs green but components pick wrong tokens (oracle/tests diverge):
  C3 rule-body locks in `extraction_card_clarity.rs` bind rule->token.
- R2 Grey theme retint temptation (achromatic lock): T-A fix list expected
  empty for grey; `test_dark_grey_chrome_is_achromatic` + semantic byte-equality
  stay UNEDITED and are the tripwire.
- R3 light --green change affects verdict chrome app-wide: it only deepens
  contrast (hue preserved); no test locks light --green values (verified);
  ui_smoke + unchanged suites; G9 deck shows all four themes.
- R4 styles.css crosses ~1950 lines (already over the 1000 ceiling, invisible
  to R2): growth capped at +150 net (old beat rules deleted as block rules
  land); extraction decision belongs to the dated deferred ledger row, NOT here
  (new asset file = lib.rs export + embed + baseline regen chain).
- R5 Amendment creep (implementer rewrites surviving locks): C7 lists the 3
  allowed; reviewer diffs both test files per commit against start rev.
- R6 Composite-layer ambiguity (inspector underlay differs from panel): made
  moot by the opaque --bg-card choice (C3); oracle composites over --bg-surface
  as the documented convention and the clarity test locks opacity-independence
  by requiring the declaration `background: var(--bg-card)`.
- R7 Lock brittleness: `function_body` terminator `"\n    }"` — indent freeze
  in C1; implementer violation fails locks for the wrong reason (visible,
  reversible).

## 5. Hygiene notes

- hotspots.js 465 -> ~600 lines; every helper well under 100 lines (blocks map
  reuses helpers, no monolith builder).
- styles.css 1807 -> ~1950 (see R4; R2 ignores .css — do not "fix" that here).
- New Rust test files ~180-300 lines each; no source line anywhere near 2000
  chars; authored content pure ASCII (no em dashes / section signs); escape
  entities (`&rarr;`) already the house idiom.
- Names: `extraction-file-block`, `extraction-file-provides`,
  `extraction-footnote` — no tmp/res/data.

## 6. Split, dependencies, concurrency

- T-A `ui-extraction-card-clarity-ta` (crates/ui, ~350-450 lines, ~5-6 gates):
  oracle C4 + themes.css fixes C6 + mod line; journal quotes the first RED
  pair table verbatim. No dependency — starts once concurrency below is clear.
- T-B `ui-extraction-card-clarity-tb` (crates/ui, ~600-800 lines, ~8-9 gates):
  C1/C2/C3 + C7 amendments + clarity locks + mod line. HARD DEPENDENCY T-A
  (its floor table is the target of the restyle; token fixes must be on main
  before the card remap is measured green).
- T-C contingent: if T-B's authored estimate exceeds ~1200 produced lines or
  ~10 gates, split T-C (styles.css section chrome + spacing + whitespace scale
  locks only, no JS) after T-B; T-B then keeps renderer + detail rules.
  Planner decides at authoring; dependency edge T-B -> T-C.
- Ledger concurrency check (2026-10-08): LIVE/OPEN rows = M28 operator tail
  (bench baseline commits + Corpora field boot: tools/bench_compare.py,
  bench_baseline.json, ledger rows — file-disjoint from assets/ui-tests) and
  M27 T1b/PR-2 operator bench duty; CLOSED-but-bookkeeping-pending rows:
  ui-dark-grey-theme (row flip + four-theme memory = docs-only, its closeout
  duty, not ours), ui-extraction-card-story / playbook / M25-M27 all closed and
  gated. M19/M20 Scala live on their own branch (no main asset conflict).
  No live hand touches index.html or crates/ui/assets -> conflict surface is
  effectively zero; serialize T-A then T-B (then optional T-C) sequentially on
  main, single-file commits via commit_scoped, ledger row flip at closeout.
