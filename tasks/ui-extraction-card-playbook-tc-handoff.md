# T-C Handoff - ui-extraction-card-playbook: collapsed summary + click-expand playbook renderer

Milestone: `ui-extraction-card-playbook`. Third and final hand of the HARD
CHAIN T-A -> T-B -> T-C. REQUIRES BOTH landed: T-A (DTO fields) and T-B (the
feed actually emits `target_files`, so the `ui_smoke` playbook render is
real). Verify both in HEAD before mutating; if absent, report, do not
start-guess.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. Shape `tasks/ui-extraction-card-playbook-shape.md` sections C3 (markup
   skeleton), C5 (disclosure mechanics), C6 (token map), C7 (help copy),
   P4-P11 (oracles) - binding.
3. Kickoff `tasks/ui-extraction-card-playbook-kickoff.md` (invariants 3-10).
4. `crates/ui/assets/hotspots.js` lines 271-344 (helpers + the five-beat
   builder you restructure) and 346-373 (`renderProposalsSection` cache +
   `syncExtractionProposalSurfaces` - BYTE-IDENTICAL).
5. `crates/ui/tests/ui_test/extraction_card_story.rs` (all 212 lines - the 9
   UNEDITABLE locks; read test 8 `test_beats_appear_in_story_order`
   :169-192 with the survival box below).
6. `crates/ui/tests/ui_test/static_html.rs` lines 847-944 +
   `static_html_sections.rs` :200-207 (C4/P7 keep-literals incl. the preview
   button).
7. `crates/ui/assets/styles.css`: locate the contiguous `.extraction-*`
   block (starts at the `/* PROPOSED EXTRACTION CARDS */` comment, ends
   before `.hotspot-row`; consolidated at `ef9d170`) - your ONLY CSS site.
   `.just-*` rules (around 800-823) are read-only neighbors, byte-identical.
8. `crates/ui/assets/index.html`: the Hotspots help `<ul>` inside
   `id="helpTab-legend"` (content-anchored; the prior milestone appended
   three `<li>` there - yours append at most three more, same list).
9. `crates/ui/tests/ui_test/help_center_readability.rs` (`css_rule_body` and
   `tab_slice` idioms to copy locally; wall-of-prose + copy locks).
10. `crates/cli/src/lib.rs` lines 112-133 and 201-219 (READ-ONLY honesty
    check for the command verbs/flags your copy prints - `apply`, `migrate`,
    `--plan`, positional repo root).

## Objective and invariants

Restructure `renderExtractionProposalCard` into: always-visible SUMMARY
(summary button with plan_id chip + target title, N symbols, guarded saving,
risk badge) + collapsed DETAIL (`<div class="extraction-details" hidden>`
holding the five existing beats UNCHANGED in order, plus the two new PLAYBOOK
rows: per-file destination manifest and the honest `tool apply ...` recipe),
preview button row UNCHANGED and OUTSIDE details. Add
`toggleExtractionCard(event, this)` (stopPropagation, DOM-walk toggle of
`hidden` + `aria-expanded` within the same `.extraction-card`, NO element
ids). Help `<li>`s per C7.

Invariants preserved (full oracles in shape P4-P11):
- P4 collapsed by default (template literal has `hidden`); P5 keyboard +
  pointer semantics (real button, `aria-expanded`, stopPropagation so card
  click still selects the hotspot).
- P6: `extraction_card_story.rs` is NOT in your touch set and its 9 tests
  stay GREEN UNEDITED.
- P7: every C4 keep-literal survives verbatim (the shape lists them; the
  consumer tests prove it).
- P8: no `fetch(`/`XMLHttpRequest`/`EventSource` ever appears in
  hotspots.js (verified absent today); renderer reads only `p.*`.
- P9 command honesty: exactly `tool apply &lt;repo-root&gt; --plan
  <plan_id>` + `tool migrate &lt;repo-root&gt; --plan <plan_id> --preview`
  (`<repo-root>` is a literal placeholder token; pulling `/api/repo-info`
  per card is REJECTED overreach - shape P9); help states the UI never
  mutates the repo and apply validates in three tiers with automatic
  rollback.
- P10 escape discipline (`escapeHtml(t.path`, each manifest symbol name) +
  zero added hex (C6 tokens only).
- P11: `proposals.map(renderExtractionProposalCard)`,
  `graphData.extraction_proposals || []`, `proposals.length === 0`, cache
  logic + `syncExtractionProposalSurfaces` byte-identical; disclosure state
  is per-DOM and resets to collapsed on re-render (documented C5 behavior).
- Legacy payloads (`target_files` absent/empty): render the honest
  `<em>no file plan in payload</em>` line - never a fabricated path.

### Story-test-8 survival rules (hard, derived from the source-first-index lock)

`test_beats_appear_in_story_order` asserts the FIRST occurrence of
`extraction-head`, then `extraction-problem`, `extraction-cost`,
`extraction-chips`, `extraction-tu` in ascending source index across the
WHOLE file - helpers count. Therefore:
1. The five beat substrings keep appearing first inside the builder's return
   template, in that order (the current template order already satisfies it;
   keep it).
2. Any NEW helper defined BEFORE the builder (`extractionSummaryMeta`,
   `renderExtractionFilePlan`, `renderExtractionApplyHint`,
   `toggleExtractionCard`) must not contain any of the five substrings in
   its own string literals. `renderExtractionFilePlan` emits
   `.extraction-file`, `.extraction-file-path`, singular `.extraction-chip`
   (via the existing `extractionChips`) - the `.extraction-chips` wrapper is
   composed by the BUILDER below the `extraction-head` line, per the C3
   skeleton.
3. If Green evidence contradicts this analysis, STOP: story test 8 is
   amended ONLY via journal + verdict written justification (the sanctioned
   path, shape P6) - never by an implementer edit.
4. No new CSS selector in styles.css may be placed such that... (NOT needed:
   test 8 scans HOTSPOTS_JS only) - CSS edits are free within their own
   block, subject to C6/hex/`.just-*` locks.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/ui-extraction-card-playbook-tc-journal.md` (journal; always in-set)
- `crates/ui/assets/hotspots.js` (builder restructure + 4 helpers +
  `toggleExtractionCard`; helpers above the builder subject to survival rule
  2; cache logic byte-identical)
- `crates/ui/assets/styles.css` (ONLY within the contiguous `.extraction-*`
  block; growth <= ~70 lines; `var(--token)` only; `.extraction-list`, risk
  tier rules, `.just-*` rules, `.extraction-card` spine unchanged)
- `crates/ui/assets/index.html` (ONLY the Hotspots help `<ul>`; <= 3 new
  `<li>`, each < 400 stripped chars, pure ASCII, `&rarr;`-style entities;
  locked phrases `Proposed extractions section`, `Preview after extraction`
  untouched)
- `crates/ui/tests/ui_test/extraction_card_playbook.rs` (NEW)
- `crates/ui/tests/ui_test/main.rs` (+1 `mod extraction_card_playbook;`
  line, alphabetical: after `mod edge_verdict_story;`, before
  `mod export_order;`)
You MUST NOT touch: `crates/ui/tests/ui_test/extraction_card_story.rs`
(the 9 locks), `static_html*.rs`, `help_center_readability.rs`,
`proposals_payload.rs`, `dto_builders.rs`, `preview_hypothetical.rs`,
`common.rs`; `api.js`, `preview.js`, `graph.js`, `main.js`, `themes.css`,
`theme.js`; anything under `crates/ui/src/**`, `crates/cli/**`,
`crates/cluster/**`; `tools/**`; `tests/fixtures/**`;
`tools/theme_token_baseline.json`. No `git stash`, no branch change.

## Frozen locks

| Lock | Proof |
|---|---|
| 9 story tests GREEN UNEDITED | present in the scoped green count + file byte-identical in `git diff --name-only` |
| P7 keep-literals (static_html.rs :847-944, static_html_sections :200-207) | those files green UNEDITED; preview button markup string byte-faithful |
| Preview contract + graph.js E-I6 cap | `static_html_sections.rs::test_extraction_impact_preview_overlay` + `preview_hypothetical.rs` green UNEDITED; `api.js`/`preview.js`/`graph.js` absent from diff |
| Help copy locks + no wall of prose | `help_center_readability.rs` green UNEDITED |
| Zero added hex | `python tools/theme_token_check.py --check-added --base <start-rev>` silent; `themes.css` absent from diff |
| DTO/feed | `crates/ui/src/**` + `crates/cli/**` absent from diff |
| `tests/fixtures/**` byte-frozen | touch-set + status census |
| Mojibake gate / hygiene | `encoding_guard.py --staged` / `hygiene_check.py` exit 0 |

## RGB protocol

1. Red (compiles; runtime assert failures only): author
   `extraction_card_playbook.rs` FIRST (test list below) + its `mod` line.
   The P4/P5/P6-family locks fail (no `extraction-disclosure`, no
   `hidden` details, no `tool apply` literal, no `extraction-file-path`, no
   help playbook phrases); the P7/P8 order/preview/no-fetch locks are
   already green and stay green (no-regression guards). Capture the failing
   list.
2. Green: implement per C3/C5/C6/C7 + survival rules; re-run scoped until
   `0 failed` - that count MUST include all 9 story tests passing. NEVER
   weaken an assertion; keep-literals win on collision.
3. Refactor: helpers <= 30 lines, builder <= ~80 (100 ceiling), doc comments
   on the disclosure DOM-walk contract and reset-to-collapsed semantics;
   re-run.

## Required tests (`extraction_card_playbook.rs`, house string idiom:
`ariadne_ui::HOTSPOTS_JS` / `STYLES_CSS` / `INDEX_HTML`, local
`css_rule_body` + `tab_slice` copies; <= 45 lines each)

1. `test_details_collapsed_by_default` - HOTSPOTS_JS contains
   `extraction-details` with `hidden` in the same template segment and
   `aria-expanded="false"` as the initial button state (P4).
2. `test_disclosure_button_contract` - contains `<button type="button"
   class="extraction-disclosure"`, `toggleExtractionCard(event, this)`,
   `stopPropagation`, and the toggle acts on `hidden` + `aria-expanded`
   (e.g. `classList.toggle` or property flip strings) via DOM walk;
   `toggleExtractionCard` does NOT use `getElementById` (P5, C5).
3. `test_summary_meta_and_risk_visible` - contains
   `extraction-summary-meta` with `symbols`, summary saving comes from
   `extractionSavingText` (reuse), summary includes `renderExtractionRiskTag`
   output (shape C3).
4. `test_file_plan_rows_markup` - contains `Files to create`,
   `extraction-file`, `extraction-file-path`, `target_files_truncated`
   handling (`more files` overflow marker string), and the legacy fallback
   literal `no file plan in payload` (P1/C3 honest fallback).
5. `test_apply_recipe_is_honest` - contains
   `tool apply &lt;repo-root&gt; --plan` with the plan_id interpolation,
   `tool migrate &lt;repo-root&gt; --plan` and `--preview`, plus the
   validate/rollback note string; contains NO other `tool ` verb (P9).
6. `test_playbook_escape_discipline` - contains `escapeHtml(t.path` (or the
   loop-variable equivalent used) and escaped symbol interpolation inside
   the file-plan helper (P10).
7. `test_playbook_styles_declared_tokens` - `css_rule_body` checks:
   `.extraction-disclosure`, `.extraction-details`, `.extraction-file-path`,
   `.extraction-command`, `.extraction-apply-note` rule bodies contain only
   `var(--` colors from the C6 map (no hex - belt over the token gate).
8. `test_no_new_data_sources` - `!HOTSPOTS_JS.contains("fetch(")`,
   `!contains("XMLHttpRequest")`, `!contains("EventSource")` (P8; green at
   Red time - no-regression guard).
9. `test_help_playbook_copy` - legend-tab slice (via `tab_slice`) contains
   the collapsed-summary/click-to-open phrase, the never-mutates statement
   (`never` + `applies` worded per C7) and the three-tiers/rollback sentence;
   locked phrases `Proposed extractions section`, `Preview after extraction`
   still present (C7; caps stay with the unedited wall test).

## Hygiene (review-enforced)

Builder ~70-80 lines; 4 new helpers <= 30 each; hotspots.js lands ~520-570
of 1000; styles.css growth <= ~70 in-block (R2 skips `.css` - the deferred
ledger row owns the split decision, not you); every authored line ASCII and
far from 2000 chars; no banned identifier names.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. Red: `python tools/testrun.py --distill -p ariadne-ui` - digest +
   canonical line verbatim, exit 1, failing list = the new-beat tests.
2. Green: same command - `0 failed`, exit 0, 9 story tests + C4 consumers +
   payload suites green UNEDITED.
3. `python tools/distill.py -- python tools/ui_smoke.py` - digest must show
   `### generic PASS (exit 0)` and `ui_smoke: PASS` (live boot renders the
   playbook markup; real `target_files` come from the T-B feed).
4. `python tools/theme_token_check.py --check-added --base <start-rev>` -
   zero added hex.
5. `python tools/hygiene_check.py` - exit 0.
6. `python tools/encoding_guard.py --staged` - exit 0 before committing.
7. Commit (single):
   `python tools/commit_scoped.py --scope crates/ui/assets/hotspots.js
   --scope crates/ui/assets/styles.css --scope crates/ui/assets/index.html
   --scope crates/ui/tests/ui_test/extraction_card_playbook.rs --scope
   crates/ui/tests/ui_test/main.rs --scope
   tasks/ui-extraction-card-playbook-tc-journal.md -m "feat(ui): collapsed
   summary + click-expand extraction playbook card (ui-extraction-card-
   playbook T-C)"` (never `git add .`).
Total implementer gate invocations: 7 (<= 10). Allowlist refusal route:
distill wrapper (open deferred ledger row).

## Dependency edges (gate bookkeeping)

- Proves RENDERING: gates 1-3 + 7 here. Feed proof = T-B's
  `-p ariadne-cli` suite; DTO proof = T-A's key/serde tests. Do not re-run
  T-A/T-B proofs here; they ride as the scoped ui suite (`export_order`,
  `dto_builders`, `proposals_payload` stay green UNEDITED).
- `ui_smoke` (gate 3) is the integration proof that T-B's feed reaches the
  card: without T-B landed, playbook rows would render the legacy `<em>`
  fallback - that is why this hand is last.

## Discipline clauses

- UTF-8: Write/Edit tools only, pure ASCII.
- Touch only the touch-set. The story test file and every existing
  card/section/help test are UNEDITABLE: if one breaks, you violated a
  keep-literal or a survival rule - fix your markup, not the test (or STOP
  and report if the shape itself is wrong).
- Raw cargo / unscoped census / full discover / bench_compare in this
  session = reviewer blocker; distilling entry points only.
- Journal checkpoint after Green (module + commit + deviations).
- Context exhaustion: STOP, disk-state inventory; successor boots via
  `checkpoint_guard.py resume --spec
  tasks/ui-extraction-card-playbook-tc-handoff.md ...`.

## Acceptance (machine-checkable)

1. `-p ariadne-ui` -> `0 failed`, exit 0, with
   `extraction_card_story.rs` + all C4/P7 consumer files byte-identical in
   the diff.
2. `ui_smoke: PASS` verbatim in the report; token check silent on added hex.
3. `git show --stat` = exactly the touch-set files.
4. hygiene + encoding exits 0.

Reviewer hop (`review-gate`): commit-vs-touch-set + per-commit `git show`;
story-file absence from diff is machine-proved; disclosure interaction
spot-check (button toggles only the enclosing card; card click still
selects); command literals spot-diffed against `crates/cli/src/lib.rs`
`apply`/`migrate` defs; C6 token map vs rules; help char caps; workspace
census reconcile at milestone closeout (1181 + T-A/T-B/T-C new tests;
`cpp-brace-depth-underflow` drift reconciled, not chased).

## Report back

Commit hash + `git show --stat`; Red failing list; Green canonical line +
capture path + ui_smoke lines; deviations; out-of-scope observations. No raw
output pastes.
