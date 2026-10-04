# T1 Handoff - ui-extraction-card-story: make the extraction card tell its story

Milestone: `ui-extraction-card-story` (M-number and dates assigned by the
`milestone-lifecycle` kickoff; do not guess one). Single sized handoff: the
architect's sizing verdict (one crate, assets + tests, no seam worth
splitting) was verified against the handoff sizing rule at planning time
(7 scoped gate invocations, ~430-490 expected produced lines). Do not split.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-extraction-card-story-shape.md` - read FULLY; invariants S1-S12,
   contracts C1-C6 (C3 markup skeleton, C4 keep-literal list, C5 token map,
   C6 honest copy) are binding.
3. `tasks/ui-extraction-card-story-kickoff.md` - intent, scope in/out,
   acceptance gate.
4. `crates/ui/assets/hotspots.js` lines 271-329 (the card builder to rewrite,
   the section + cache logic to keep byte-identical) and lines 201-203
   (`escapeHtml`).
5. `crates/ui/tests/ui_test/static_html.rs` lines 847-944
   (`test_ui_extraction_proposal_cards` - the keep-literal oracle set you must
   keep green UNEDITED).
6. `crates/ui/tests/ui_test/static_html_sections.rs` lines 200-207 (preview
   button label lock) and line 621 onward (graph.js `<= 990` line cap).
7. `crates/ui/tests/ui_test/help_center_readability.rs` (`css_rule_body`
   idiom to copy locally; `test_legend_copy_locks_survive`;
   `test_legend_tab_has_no_wall_of_prose`).
8. `crates/ui/assets/index.html` lines 506-511 (Hotspots help list - your ONLY
   edit site) and 468-515 (legend tab bounds the wall-of-prose test slices).
9. `crates/ui/assets/styles.css` lines 988-1034 (the extraction block - your
   ONLY edit site) and 800-823 (`.just-tag`/`.just-collateral`/`.just-more` -
   read-only neighbors, byte-identical).
10. `crates/ui/tests/ui_test/main.rs` (mod registration; alphabetical site is
    between `mod export_order;` and `mod file_view;`).
11. `docs/specs/tool-output-distillation.md` (distilling entry points - the
    only commands you may run).

## Objective and invariants

Rewrite `renderExtractionProposalCard` so the card tells the five-beat story
(why -> what moves -> what it costs -> who is affected -> preview) per shape
C3: `problem` visible in the body, `plan_id` identity chip, labeled delta with
a computed saving, name/header chips, tiered risk badge, reader-worded
consumer-TU count - then lock all of it in a new test module.

Invariants this handoff must preserve (full list = shape S1-S12; the
load-bearing ones):
- Payload order rendered as-is; NO re-sort in `hotspots.js` (S6).
- Empty/absent payload hides the section; `graphData.extraction_proposals || []`
  and `proposals.length === 0` byte-identical (S7).
- Preview button markup, class `just-preview-btn`, label `Preview drop from
  selected file`, both `data-preview-*` attributes, `bindPreviewControls`
  call sites unchanged (S8/C2); nothing else in the card carries
  `.just-preview-btn`.
- One shared builder, name `renderExtractionProposalCard(` kept (C1);
  `graph.js` never gains it and is not touched at all.
- Every card-data interpolation passes through `escapeHtml` (S10).
- Zero added hex; `var(--token)` only, from the existing three-theme set (S9).
- DTO frozen; producers frozen; every existing test green UNEDITED via the C4
  keep-literal list.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/ui-extraction-card-story-t1-journal.md` (checkpoint journal; always
  in-set; create it alongside this handoff)
- `crates/ui/assets/hotspots.js` (builder rewrite + helpers
  `extractionChips`, `extractionSavingText`, `renderExtractionRiskTag`;
  `renderProposalsSection` cache logic and `syncExtractionProposalSurfaces`
  byte-identical)
- `crates/ui/assets/styles.css` (ONLY the contiguous extraction block,
  currently the comment at line 988 through `.extraction-row` closing brace
  at line 1034; growth of the whole block <= ~90 added lines)
- `crates/ui/assets/index.html` (ONLY the Hotspots help `<ul>` at lines
  506-511; at most three new `<li>`)
- `crates/ui/tests/ui_test/extraction_card_story.rs` (NEW - the story locks)
- `crates/ui/tests/ui_test/main.rs` (+1 `mod extraction_card_story;` line,
  alphabetical: after `mod export_order;`, before `mod file_view;`)

You MUST NOT touch: `crates/ui/assets/api.js`, `preview.js`, `graph.js`,
`main.js`, `themes.css`, `theme.js` (byte-identical); everything under
`crates/ui/src/**` incl. `dto.rs`; `crates/cli/**`; `crates/cluster/**`;
`tools/**` incl. `tools/theme_token_baseline.json`; `tests/fixtures/**`;
`crates/ui/tests/ui_test/common.rs` and ALL other test files - especially
`static_html.rs`, `static_html_sections.rs`, `help_center_readability.rs`,
`proposals_payload.rs`, `dto_builders.rs` (their assertions are the review
oracles; editing any = REJECT). No `git stash`, no branch change.

## Frozen locks (each with its proving command)

| Lock | Proof |
|---|---|
| `api.js`, `preview.js`, `graph.js`, `main.js`, `themes.css`, `theme.js` byte-identical | touch-set check + `git diff --stat <start-rev>..HEAD` shows only touch-set files; `static_html_sections.rs::test_extraction_impact_preview_overlay` green unedited (also holds the graph.js `<= 990` E-I6 cap) |
| DTO/serde frozen (S12) | `proposals_payload.rs` + `dto_builders.rs` green unedited inside the scoped suite |
| C4 keep-literals (existing tests grep them) | `static_html.rs::test_ui_extraction_proposal_cards` green unedited: `function renderExtractionProposalCard(`, `collateral_load_before`, `collateral_load_after`, `consumer_tu_count`, `extraction-row`, `extracted symbols need`, `just-more`, `selectHotspotRow('`, `graphData.extraction_proposals || []`, `proposals.length === 0`, `Preview drop from selected file` |
| Help copy locks + no wall of prose (S11) | `help_center_readability.rs` both tests green unedited |
| Zero added hex (S9) | `python tools/theme_token_check.py --check-added --base <start-rev>` prints zero added hex; `themes.css` absent from diff |
| `tests/fixtures/**` byte-frozen (all branches) | touch-set check + `git_git_status` census at review |
| Mojibake/UTF-8 gate | `python tools/encoding_guard.py --staged` exit 0 pre-commit (`--all` in CI + reviewer hop) |
| Hygiene (R1/R2 blocking) | `python tools/hygiene_check.py` exit 0 |

Record the start rev (= HEAD before your first commit) in the journal at
bootstrap; all `--base` arguments use it.

## What to build (exact literals)

Follow shape C3 markup skeleton verbatim as the class inventory
(`.extraction-head`, `.extraction-plan-id`, `.extraction-problem`,
`.extraction-label`, `.extraction-cost`, `.extraction-delta`,
`.extraction-saving`, `.extraction-chips`, `.extraction-chip`,
`.extraction-tu`, `.extraction-risk extraction-risk-<tier>`; new classes live
ONLY under `.extraction-*`; `.just-more` is REUSED for `+N more`, never
restyled). Honest copy per C6, exactly:
- TU sentence: `N consumer TU(s) need re-validation` (interpolate
  `${p.consumer_tu_count || 0}` so the keep-literal `consumer_tu_count` stays).
- Risk badge text: `Risk: <tier>`; tier class suffix =
  `risk.toLowerCase().replace(/[^a-z]/g, '')` - unknown values degrade to the
  neutral base class, no raw-string class injection (S5).
- Saving chip text: `-D (-P%)` where `D = before - after`,
  `P = Math.round(D * 100 / before)`; the chip renders ONLY when
  `before > 0 && after < before` (no chip, no division, no negative percent
  otherwise; S3). Delta keeps `${p.collateral_load_before || 0} &rarr;
  ${p.collateral_load_after || 0}` (use the `&rarr;` entity, ASCII file).
- Card `title` becomes the target path (`p.target_file ||
  String(p.target_file_id)`, escaped); `p.problem` moves OUT of every
  `title` and INTO the body as `.extraction-problem` (S1).
- Needs row label stays the locked literal `extracted symbols need`; headers
  render as `.extraction-chip` or keep the italic `no additional headers`
  fallback.
- `extractionChips(items)` maps every item through `escapeHtml` (S10); the
  `+N more` overflow math (`total_symbols - symbol_names.length`) is kept.

CSS per C5 token map, all rules inside the extraction block, `var(--token)`
only; keep `.extraction-list` (the `proposedExtractionsList` container uses
it - flex column, 240px scroll), keep `.extraction-card` tile/border/amber
left spine; drop `.extraction-names` only if the builder truly stops using
it. Do not move rules outside the block; `.just-*` rules (lines 800-823) are
byte-identical.

Help copy (C6): at most three new `<li>` in the Hotspots list, pure ASCII,
each under 400 stripped chars: define collateral ONCE (symbols every consumer
drags along), the risk tiers as safety ordering only (planner proposals are
all Conservative today - true at `planner.rs:664`; do not claim more), and
the consumer-TU count as TUs needing re-validation. Do not disturb the locked
phrases `Proposed extractions section` and `Preview after extraction`.

## RGB protocol (per module, in this file order)

1. Red: author `extraction_card_story.rs` (full required-test list below) +
   its `mod` line FIRST. It compiles (pure string asserts against
   `ariadne_ui::HOTSPOTS_JS` / `STYLES_CSS` / `INDEX_HTML` + local
   `css_rule_body` copy) and FAILS at runtime - never use `todo!()` or
   compile-failure as Red. Capture the failing test list verbatim.
2. Green: implement builder + helpers, CSS block, help `<li>`s; re-run the
   scoped suite until 0 failed. NEVER weaken an assertion to pass; if a
   keep-literal collides with the skeleton, the keep-literal wins.
3. Refactor: dedupe, honest names, doc comments; re-run. Then hygiene,
   token check, encoding check, commit.

Red expectation: the new-beat tests (1,2,3,4,5,6,8,9 below) fail; the
order/preview locks (7 below) already pass today and stay passing. Expected
failure messages: `extraction-problem` / `extraction-plan-id` /
`extractionSavingText` / `extraction-chip` substrings "must" contain fails;
test 4 fails because `.join(', ')` exists today at `hotspots.js:279-280`;
test 6 panics via `css_rule_body` with `styles.css must contain a rule for
.extraction-risk-conservative`; test 9 fails on `consumer TU(s) need
re-validation` absent.

## Required tests (in `crates/ui/tests/ui_test/extraction_card_story.rs`)

1. `test_why_beat_visible_in_body` - HOTSPOTS_JS contains
   `extraction-problem` and `escapeHtml(p.problem`; NOT contains
   `title="${escapeHtml(p.problem`.
2. `test_plan_id_identity_chip` - contains `extraction-plan-id` and
   `escapeHtml(p.plan_id` (S2).
3. `test_saving_computed_with_zero_guard` - contains `extractionSavingText`,
   `Math.round`, and the `before > 0` guard expression; contains no bare
   percent without the guard (S3).
4. `test_no_runon_join_blobs` - `!HOTSPOTS_JS.contains(".join(', ')")`;
   note `.join('')` elsewhere is a different substring and stays.
5. `test_names_and_headers_render_as_chips` - contains `extractionChips`
   and `extraction-chip`; contains `map(escapeHtml` (S10 helper part).
6. `test_risk_tier_badge_sanitized` - HOTSPOTS_JS contains
   `extraction-risk`, `toLowerCase`, `[^a-z]`; STYLES_CSS has rule bodies
   (via local `css_rule_body`) for `.extraction-risk-conservative` with
   `var(--green)`, `.extraction-risk-moderate` with `var(--amber)`,
   `.extraction-risk-aggressive` with `var(--red)`, base `.extraction-risk`
   with `var(--text-sub)` (S5).
7. `test_order_and_preview_locks` - contains
   `proposals.map(renderExtractionProposalCard)`; NOT contains
   `extraction_proposals.sort` (S6). Already green at Red time.
8. `test_beats_appear_in_story_order` - index positions of `extraction-head`,
   `extraction-problem`, `extraction-cost`, `extraction-chips`,
   `extraction-tu` strictly ascending in HOTSPOTS_JS (the five beats, top to
   bottom).
9. `test_tu_sentence_and_help_copy` - HOTSPOTS_JS contains
   `consumer TU(s) need re-validation`; INDEX_HTML legend-tab slice (copy
   the `tab_slice` idiom or use `find` bounds `helpTab-legend` to
   `helpTab-recipes`) contains the collateral definition phrase
   `symbols every consumer drags` and `re-validation` (S11 additions; the
   400-char cap and copy locks stay with the existing unedited suite).

## Gates (exact commands, timeout 600000 ms each; scoped only - workspace-wide
gates are the reviewer's)

1. Red: `python tools/testrun.py --distill -p ariadne-ui` - digest header +
   canonical summary line appear verbatim in the report; the failing list =
   the new tests named above; exit 1.
2. Green: same command - `0 failed`, exit 0.
3. Refactor re-run (same command) only if refactor touched behavior beyond
   naming/comments.
4. `python tools/theme_token_check.py --check-added --base <start-rev>` -
   zero added hex.
5. `python tools/hygiene_check.py` - exit 0.
6. `python tools/encoding_guard.py --staged` - exit 0 before committing.
7. Commit (single commit; Red evidence lives in the journal):
   `python tools/commit_scoped.py --scope crates/ui/assets/hotspots.js
   --scope crates/ui/assets/styles.css --scope crates/ui/assets/index.html
   --scope crates/ui/tests/ui_test/extraction_card_story.rs --scope
   crates/ui/tests/ui_test/main.rs --scope
   tasks/ui-extraction-card-story-t1-journal.md -m "feat(ui): extraction
   proposal card tells the five-beat story (ui-extraction-card-story T1)"`
   (never `git add .`).

Total implementer gate invocations: 7 (<= 10).

## Refactor and hygiene targets

Ceilings at risk, named: `renderExtractionProposalCard` target <= ~70 lines
(100-line function ceiling); helpers <= 20 lines each; `hotspots.js` lands
~430-470 of 1000 (file ceiling); styles.css growth <= ~90 lines IN THE BLOCK
(R2 skips `.css` - the split decision is a deferred ledger row, not yours);
no help `<li>` near 2000 chars; no banned names (`tmp`/`res`/`foo`/`bar`,
single-letter locals); every authored line pure ASCII (arrows as `&rarr;`,
no emoji literals).

## Discipline clauses

- UTF-8: never Set-Content/Out-File/`>`/Get-Content-replace; Write/Edit tools
  only; pure-ASCII content (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`, full census) in this
  session are a reviewer blocker; use the distilling entry points
  (`docs/specs/tool-output-distillation.md`).
- After EACH per-module Green gate, append a 1-3 line checkpoint to
  `tasks/ui-extraction-card-story-t1-journal.md`: module + commit hash,
  deviations, lock/contract findings.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/ui-extraction-card-story-t1-handoff.md --scope ... --cmd ...`.

## Acceptance (machine-checkable)

Implementer hop (all must appear in the report, quoted verbatim):
1. `python tools/testrun.py --distill -p ariadne-ui` -> `0 failed`, exit 0,
   ALL pre-existing card/section/help/payload tests green UNEDITED.
2. `python tools/theme_token_check.py --check-added --base <start-rev>` ->
   zero added hex.
3. `python tools/hygiene_check.py` -> exit 0.
4. `python tools/encoding_guard.py --staged` -> exit 0 (pre-commit).
5. `git show --stat` of the landing commit lists exactly the touch-set files.

Reviewer hop - workspace-wide gates (fresh session or reviewer's own bounded
run; NOT the implementer's):
1. `python tools/testrun.py --distill -p ariadne-ui` -> `0 failed`.
2. Full-workspace census. `--baseline` is a FILE flag and no baseline file is
   committed (open ledger errata row): run
   `python tools/testrun.py --distill` (unscoped) and record the canonical
   line; expect `0 failed`, suites unchanged at 85, passed = prior census
   (1168 @ `1bc50b8`, plus any drift since) + N new tests. If the operator
   has committed a baseline file by then, use
   `python tools/testrun.py --distill --baseline <file>`.
3. `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 188 tests`
   + `OK` (count may have drifted; `OK` is the verdict).
4. `python tools/ui_smoke.py` -> `ui_smoke: PASS` (cold-boot visual sanity of
   the panel routes; card height inside the 240px scroller is the
   reviewer/human eyeball step - no height-locked test exists, stated in the
   shape risks).
5. `python tools/bench_compare.py` -> PASS; or the documented assets-only
   exemption (precedent: ui-progress-parity row - pipeline byte-identical,
   record the exemption sentence in the verdict).
6. `python tools/theme_token_check.py --check-added --base <parent-rev>` ->
   zero added hex; `themes.css` absent from the diff.
7. `python tools/encoding_guard.py --all` -> exit 0.
8. `python tools/hygiene_check.py` -> exit 0.
9. Review-gate per-commit `git show` oldest-first + commit-vs-touch-set
   enforcement + status census (`tests/fixtures/**` untouched; api.js,
   preview.js, graph.js, main.js, themes.css, theme.js byte-identical;
   `crates/ui/src/**` untouched).

Reviewer hop count: 9 (<= 10).

## Reviewer-gate reference

Adjudicated via the `review-gate` skill. Its commit-vs-touch-set check
enforces the touch-set above. Extra reviewer-specific checks: C4 keep-literals
present verbatim in the new `hotspots.js`; C6 claims checked against
`crates/cluster/src/planner.rs:659`/`:664` and
`crates/cli/src/commands/ui.rs:534` (copy must not overclaim); no `.sort(`
over proposals; no `join(', ')` reintroduced anywhere in `hotspots.js`;
`.just-*` rules byte-identical.

## Sequencing note (not parallel-safe)

`tasks/ui-dark-grey-theme-t1-handoff.md` (also PLANNED) touches
`crates/ui/assets/index.html` (theme `<option>` + Help Theme paragraph). Two
handoffs touching the same file are never parallel: the orchestrator must
serialize this milestone against `ui-dark-grey-theme`. Disjoint otherwise
(that handoff excludes `hotspots.js`/`styles.css`; this one excludes
`themes.css`/`theme.js`). Whichever lands second uses the first landing
commit as its `--base` rev.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence: first failing test list.
3. Green evidence: distill-mode digest header + canonical summary line +
   capture path. 4. Any deviation + reason. 5. Out-of-scope observations
   (report-only). Never paste raw tool output: cite command + canonical line
   + exit code.
