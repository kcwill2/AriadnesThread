# T-B Handoff - ui-extraction-card-clarity: per-file instruction blocks + restyle

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract). T-A must be LANDED on main before you boot
   (hard dependency: its contrast floors are the target of this restyle).
2. `tasks/ui-extraction-card-clarity-shape.md` sections 1 (I1, I3, I5, I6-I12),
   2 (C1 freeze list, C2 markup tree, C3 token-choice map + scale, C7
   amendment list AS AMENDED BY THIS FILE - see the Amended Test Table below).
3. `crates/ui/assets/hotspots.js` lines 276-440 (the extraction builders you
   restructure) and `crates/ui/assets/styles.css` lines 988-1177 (the
   PROPOSED EXTRACTION CARDS block you restyle).
4. Model locks for the new suite: `crates/ui/tests/ui_test/extraction_card_playbook.rs`
   (css_rule_body / function_body idiom, local copies per house practice).

## Objective and preserved invariants

Restructure the extraction card detail from stacked beats into per-file
instruction blocks (path heading / what it provides / which symbols), demote
stats to a footnote strip, keep the commands block last, slim the summary to
the five scan fields, and restyle the section so cards, header, and surfaces
stop reading as one mass - in all four themes.
Preserved verbatim: I1 disclosure model (`class="extraction-details" hidden>`
+ `aria-expanded="false"` + `toggleExtractionCard(event, this)` - UNEDITED
strings), I5 per-file story, I6 command honesty (only `tool apply`/`tool
migrate`, verb-count assert), I7 no new data (DTO byte-identical, zero
`crates/*/src` diff), I8 summary = exactly plan id, target path, N symbols,
guarded saving, risk badge, I10 payload order (no sort), I11 no comma blobs,
I12 escape discipline.

## Contract tensions resolved at authoring (planner decisions - plan-literally
binding; reviewer adjudicates against THIS section, not the raw shape)
1. Amended-test count: shape C7 says "3 amended, 13 survive" but the two test
   files hold 18 functions total (9 story + 9 playbook) and one more surviving
   lock is structurally incompatible with C2. This hand amends FOUR, fourteen
   survive byte-identical, zero die (evidence: `test_file_plan_rows_markup`
   locks `class="extraction-file"` with its closing quote, which cannot
   coexist with C2's `class="extraction-file-block"`; C1's own frozen
   substring list for the helper deliberately omits it, and C7's
   styles-selector amendment already introduces `.extraction-file-block {`).
2. provides-line sourcing: C2's line "Provides N symbols moved out of
   <target_file>" needs the card-level source name, while C1 freezes the
   helper signature `(targetFiles, truncated)`. Resolution: the card builder
   pre-maps enriched entries - `renderExtractionFilePlan((p.target_files ||
   []).map(t => ({ ...t, from: p.target_file || "" })), p.target_files_truncated)`
   - helper signature stays byte-identical, helper reads `t.from` and escapes
   it; no new payload field, no new data source (I7 holds: `p.target_file` is
   an already-read payload field).
3. Section chrome: the shape's contingent T-C is FOLDED into this hand
   (measured estimate stays under both sizing ceilings; chrome without the
   renderer restructure would ship a half-built seam). The scoped
   section-header rule lands INSIDE the extraction block:
   `#proposedExtractionsSection .link-list-title { border-bottom: 1px solid
   var(--border); padding-bottom: 8px; }` plus list margin - the shared
   `.link-list-title` rule elsewhere in styles.css stays UNTOUCHED.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/ui-extraction-card-clarity-tb-journal.md (create it; FIRST entry
  records start rev + baseline scoped-ui test line; start rev = T-A landing)
- crates/ui/assets/hotspots.js (extraction builders only, lines 276-440
  region; C1 frozen names/substrings/indent stay)
- crates/ui/assets/styles.css (edits restricted to the PROPOSED EXTRACTION
  CARDS block currently at :988-1177, including the scoped section-chrome
  rule added inside that block)
- crates/ui/tests/ui_test/extraction_card_story.rs (2 functions ONLY:
  `test_beats_appear_in_story_order`, `test_risk_tier_badge_sanitized`)
- crates/ui/tests/ui_test/extraction_card_playbook.rs (2 functions ONLY:
  `test_file_plan_rows_markup`, `test_playbook_styles_declared_tokens`)
- crates/ui/tests/ui_test/extraction_card_clarity.rs (new, ~8-10 locks)
- crates/ui/tests/ui_test/main.rs (+1 mod line ONLY: insert
  `mod extraction_card_clarity;` immediately BEFORE
  `mod extraction_card_playbook;` per shape C7; do NOT fix the mod-order
  erratum region beyond that insertion)

You MUST NOT touch: index.html, api.js, theme.js, main.js, preview.js,
graph.js, file-view.js, provenance.js, themes.css, index assets,
crates/*/src/**, tools/** (incl. baseline JSON + theme_token_check.py),
tests/fixtures/**, theme_contract.rs, theme_dark_grey_palette.rs,
theme_js_zero_hex.rs, help_center_readability.rs, static_html*.rs,
proposals_payload.rs. No `git stash`, no branch change.

## What to build

Renderer (C1/C2; C1 freezes every name, the 4-space top-level indent, and
these substrings: `toggleExtractionCard`, `extractionChips`,
`extractionSavingText`, `renderExtractionRiskTag`, `extractionSummaryMeta`,
`renderExtractionFilePlan` with signature `(targetFiles, truncated)` and
`escapeHtml(t.path`, `extractionChips(t.symbol_names)`,
`Files to create (${files.length})`, `more files`, fallback `<em>`;
`renderExtractionApplyHint` markup byte-faithful;
`proposals.map(renderExtractionProposalCard)`, `before > 0`, `Math.round`,
`100 / before`, `consumer TU(s) need re-validation`, `toLowerCase`, `[^a-z]`,
getElementById-free toggle):
```
.extraction-details (hidden)
  .extraction-problem            one-line why (escaped p.problem)
  .extraction-file-blocks        intro "Files to create (N)" + truncated marker
    .extraction-file-block       per target_files entry, payload order
      code.extraction-file-path  escaped real path
      .extraction-file-provides  "Provides N symbols moved out of <escaped
                                 source path>" (derived via `from` key, tension 2)
      .extraction-chips          extractionChips(t.symbol_names) + `+K more`
  .extraction-footnote           one strip: .extraction-cost (before -> after
                                 + guarded saving), .extraction-tu sentence,
                                 "New file includes" header chips
  .extraction-apply              commands block, markup UNCHANGED (P9)
```
Card-level `p.symbol_names` chip row and the old Impact/headers stacked rows
are deleted from the builder (their content now lives in blocks + footnote);
summary row keeps exactly the five fields (head chip, title, meta, saving,
risk badge). Preview button row stays outside the detail, byte-faithful.

Styles (C3 map, verbatim token choices; zero new hex):
.extraction-card -> opaque `background: var(--bg-card)`, border
`var(--border-light)`, 3px amber left spine kept, `padding: 10px 12px`;
hover `var(--bg-card-hover)`; .extraction-title `var(--text-main)` 13px/700;
.extraction-details `color: var(--text-muted)`, line-height 1.45, 10px top
spacing (NO `--text-sub` anywhere in the block - I3 ban replaces current
usages at :1047/:1084/:1088/:1142/:1175); .extraction-label 10px
`var(--text-muted)`; .extraction-file-path/.extraction-command `var(--bg-sub)`
+ `var(--border-light)` + `var(--code-text)` 12px; .extraction-chip
`var(--bg-card-hover)` + `var(--border-light)`; .extraction-footnote
border-top 1px `var(--border)` 10px `var(--text-muted)`; .extraction-list
gap 10px; section chrome per tension 3. Scale: card padding 10x12, block gap
12px, heading 13px, body 11px, footnote/chip 10px.

## Amended Test Table (exhaustive; each amendment carries its justification
comment IN the diff; anything beyond this table is amendment creep = blocker)
| test | edit | rationale (goes into the diff comment) |
|---|---|---|
| story `test_beats_appear_in_story_order` | beats list becomes `["extraction-head","extraction-problem","extraction-file-block","extraction-chips","extraction-cost","extraction-tu"]` | user mandate: per-file story precedes stats; survives as order lock in the new canonical order |
| story `test_risk_tier_badge_sanitized` | base-rule assert `var(--text-sub)` -> `var(--text-muted)` | unknown-tier badge now paints the card surface; --text-sub there measured 2.97:1 (dark); I3 ban |
| playbook `test_file_plan_rows_markup` | `class=\"extraction-file\"` -> `class=\"extraction-file-block\"` (single substring) | C2 renames the per-file row to block; C1's frozen list already omits the old literal (planner resolution 1) |
| playbook `test_playbook_styles_declared_tokens` | selector list += `.extraction-file-block {`, `.extraction-footnote {`, `.extraction-file-provides {`; per-selector asserts unchanged | the three new C2 classes must be token-declared like their siblings |
ALL 14 other test functions survive byte-identical (proof: gate 4).
New `extraction_card_clarity.rs` locks (~8-10, implementer-authored
messages): C2 block order incl. footnote-before-apply; I8 five-field summary
(nothing else visible in the summary); C3 rule-body token locks (card
background `var(--bg-card)`, details `var(--text-muted)`, path/command
`var(--bg-sub)`+`var(--code-text)`, chip `var(--bg-card-hover)`); I3 ban: no
`.extraction-*` rule in the block declares `color: var(--text-sub)`; scale
substrings `gap: 10px` and `padding: 10px 12px`; the section-chrome scoped
rule exists. At Red, each assertion is EITHER failing now (true Red) OR
journal-listed as an already-green no-regression lock.

## RGB protocol (per module, in this file order)
1. Red: write the 4 amendments + the full clarity suite FIRST; scoped ui run
   fails ONLY on the named amendments (missing block class / --text-muted
   base / new selectors / reordered beats) and the structural clarity locks
   (`.extraction-file-block` absent, gap 6px, --text-sub ban). Compile-clean;
   a non-compiling test is not Red. Journal captures the failing list verbatim
   + the baseline line.
2. Green: restructure renderer + restyle block; re-run until exit 0. NEVER
   weaken an assertion or C1 frozen substring to pass; the disclosure strings
   stay byte-identical.
3. Refactor: delete the dead beat rules (net growth cap), reuse the escape
   paths (one chips helper), update the stale doc comment above
   renderExtractionFilePlan (rows -> blocks), honest names.

## Frozen locks (with proving command)
- The 14 surviving test functions byte-identical; the 4 test functions
  OUTSIDE the named four byte-identical too:
  `python tools/distill.py -- git diff <start-rev> -- crates/ui/tests/ui_test/extraction_card_story.rs crates/ui/tests/ui_test/extraction_card_playbook.rs`
  - every changed hunk must lie inside one of the four named functions;
  journal lists the hunks.
- Disclosure model: playbook `test_details_collapsed_by_default` +
  `test_disclosure_button_contract` green UNEDITED (inside the scoped run).
- `git diff <start-rev> -- crates/*/src tools tests/fixtures crates/ui/assets/index.html crates/ui/assets/api.js crates/ui/assets/theme.js`
  empty (I7 DTO + bench exemption + zero-touch surfaces).
- themes.css UNEDITED in this hand (T-A landed its fixes): appears in the
  empty-diff above via `git diff <start-rev> -- crates/ui/assets/themes.css`
  (start rev = T-A landing).
- `python tools/theme_token_check.py --check-added --base <start-rev>` silent;
  baseline JSON diff-empty.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only)
1. `python tools/testrun.py --distill -p ariadne-ui` (baseline, BEFORE edits)
2. `python tools/testrun.py --distill -p ariadne-ui` (Red - failing list
   journal-verbatim)
3. `python tools/testrun.py --distill -p ariadne-ui` (Green - exit 0,
   delta vs baseline = clarity test count, zero otherwise)
4. `python tools/distill.py -- git diff <start-rev> -- crates/ui/tests/ui_test/extraction_card_story.rs crates/ui/tests/ui_test/extraction_card_playbook.rs` (survival proof)
5. `python tools/theme_token_check.py --check-added --base <start-rev>` (silent)
6. `python tools/hygiene_check.py` (exit 0)
7. `python tools/encoding_guard.py --staged` (exit 0)
8. `python tools/commit_scoped.py --scope crates/ui/assets/hotspots.js --scope crates/ui/assets/styles.css --scope crates/ui/tests/ui_test/extraction_card_clarity.rs --scope crates/ui/tests/ui_test/extraction_card_story.rs --scope crates/ui/tests/ui_test/extraction_card_playbook.rs --scope crates/ui/tests/ui_test/main.rs --scope tasks/ui-extraction-card-clarity-tb-journal.md -m "feat(ui): per-file extraction instruction blocks + section restyle"`
   (single landing commit; never `git add .`)
~9-10 invocations with Green re-runs. Overshoot is declared as a dated
D-row in the journal (M28 T0b finding 1 law).

## Hygiene (review-enforced) - ceilings at risk named
- `renderExtractionProposalCard` must stay < 100 lines: extract the footnote
  strip builder (e.g. `renderExtractionFootnote(p)`) rather than inlining;
  every helper keeps the frozen 4-space IIFE indent (`function_body` locks
  terminate on `"\n    }"` - R7 brittleness).
- styles.css net growth cap +150 lines (1807 -> <= 1957; old beat rules are
  DELETED as block rules land; the 1000-line ceiling decision belongs to the
  dated deferred ledger row - do NOT split or "fix" styles.css structure here).
- hotspots.js 465 -> ~600; no monolith builder (blocks map reuses helpers).
- Functions < 100 lines, no banned names, `python tools/hygiene_check.py`
  exit 0.

## Discipline clauses
- UTF-8: Write/Edit tools only; authored content pure ASCII (escape entities
  like `&rarr;` are the house idiom); never PowerShell file rewrites
  (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; C1 violation suspected -> report, don't improvise.
- Raw `cargo test`/`clippy`/`ui_smoke`/census in this session = reviewer
  blocker; distilling entry points only. Workspace-wide gates (census,
  ui_smoke, tools discover) belong to the reviewer's acceptance pass, NOT
  your commits.
- After EACH Green step, journal a 1-3 line checkpoint. If context looms:
  STOP, return disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ui-extraction-card-clarity-tb-handoff.md`.

## Acceptance (reviewer runs; verdict lines expected)
- Fresh scoped ui run exit 0; delta = clarity test count vs baseline.
- Survival proof: the two test-file diffs contain hunks ONLY in the four
  named functions (amended count 4, byte-identical 14, killed 0).
- Per-commit touch-set enforcement (review-gate set equality); index.html,
  themes.css, all crates/*/src diff-empty; bench EXEMPT with that proof.
- `python tools/theme_token_check.py --check-added --base <start-rev>` silent.
- `python tools/ui_smoke.py` PASS (via `python tools/distill.py -- ...` if
  the session allowlist denies direct runs); workspace census via `--distill`
  reconciles from the reviewer-captured start anchor; `python -m unittest
  discover -s tools -p "*_tests.py"` unchanged `Ran 188 ... OK`;
  hygiene + `encoding_guard --all` exit 0.
- Reviewer gate reference: adjudicated via the `review-gate` skill; plan-
  literal checks cite THIS file including the tension resolutions (raw shape
  C7 count is superseded here; see ledger erratum row dated 2026-10-08).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (failing list verbatim +
already-green lock list). 3. Green evidence (digest header + canonical line +
capture path). 4. Survival-proof hunk inventory. 5. Deviations with dated
D-rows. 6. Out-of-scope observations (report-only).
Never paste raw tool output; cite command + canonical line + exit code.
