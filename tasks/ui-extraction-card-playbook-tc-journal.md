# T-C Journal - ui-extraction-card-playbook: collapsed summary + click-expand playbook renderer

Handoff: `tasks/ui-extraction-card-playbook-tc-handoff.md`.
Start rev (HEAD at session boot): `6a48a85` (T-B; contains T-A `df1e21e`).
Scope: `crates/ui` assets + the new playbook test module. Third hand of the
T-A -> T-B -> T-C chain.

## Ground truth at boot

- Chain verified before mutating: `git log` shows `6a48a85` (T-B feed) on
  top of `af11113` (`cpp-brace-depth-underflow`) on top of `df1e21e` (T-A
  DTO); `target_files` present in `crates/cli/src/commands/ui.rs` and in the
  ui payload tests. No concurrent index.html worker (serialization note in
  the tasking honored: this hand owns index.html/styles.css/hotspots.js).
- T-A reviewer ruling honored: all authored content pure ASCII (no U+2014).
- CLI honesty check (read-only): `crates/cli/src/lib.rs` confirms `Apply
  { repo: PathBuf, #[arg(long)] plan }` (:212-219) and `Migrate { repo,
  plan, #[arg(long)] preview }` (:201-211) - positional repo root, `--plan`
  flag. Card copy uses exactly these verbs/flags.

## What was done

- `crates/ui/tests/ui_test/extraction_card_playbook.rs` (NEW): the 9
  required locks, each under 45 lines, local `css_rule_body` + `tab_slice`
  copies plus a `function_body` slicer so each lock judges exactly the
  helper it names. `test_no_new_data_sources` is the P8 no-regression
  guard, green at Red time.
- `crates/ui/tests/ui_test/main.rs`: ONE line `mod
  extraction_card_playbook;` inserted alphabetically after
  `mod edge_verdict_story;`, before `mod export_order;`.
- `crates/ui/assets/hotspots.js` (395 -> 465 lines):
  - `toggleExtractionCard(event, buttonEl)` next to `selectHotspotRow`
    (doc-commented DOM-walk contract + reset-to-collapsed semantics):
    stopPropagation, `closest`/`querySelector` within the same card, NO
    element ids, flips `hidden` + `aria-expanded`.
  - `extractionSummaryMeta(p)` (6 lines): symbol count + the guarded saving
    REUSED from `extractionSavingText`, wrapped in `.extraction-summary-saving`.
  - `renderExtractionFilePlan(targetFiles, truncated)` (13 lines): one
    `.extraction-file` row per destination with `escapeHtml(t.path)` and
    symbols only via `extractionChips(t.symbol_names)`, `just-more` symbol
    overflow, `more files not listed` marker when truncated, honest
    `<em>no file plan in payload</em>` for legacy payloads.
  - `renderExtractionApplyHint(planId)` (8 lines): `tool apply
    &lt;repo-root&gt; --plan <id>` + note `Validates and rolls back on
    failure; preview first: tool migrate &lt;repo-root&gt; --plan <id>
    --preview`; escaped plan id; the only two `tool ` mentions in the file.
  - `renderExtractionProposalCard` restructured (46 lines): summary button
    (`type="button" class="extraction-disclosure" aria-expanded="false"`)
    carrying head + meta + saving + risk; `<div class="extraction-details"
    hidden>` holding the five beats UNCHANGED in order plus filePlan +
    applyHint; preview row byte-faithful and OUTSIDE the details.
  - Cache logic + `syncExtractionProposalSurfaces` byte-identical; no
    fetch/XHR/EventSource; payload order untouched.
  - Survival-rule compliance: the four new helpers contain none of the five
    beat substrings; the plural `.extraction-chips` wrapper is composed
    only in the builder below the `extraction-head` line, so test 8's
    first-source-index order (head -> problem -> cost -> chips -> tu) is
    preserved. Story file untouched.
- `crates/ui/assets/styles.css` (1731 -> 1795 lines, +64 contiguous inside
  the `.extraction-*` block): `.extraction-disclosure` (+hover),
  `.extraction-summary-meta`, `.extraction-summary-saving`,
  `.extraction-details`, `.extraction-files`, `.extraction-file`,
  `.extraction-file-path`, `.extraction-command`, `.extraction-apply-note`.
  All colors are `var(--token)` from the existing three-theme set; zero hex;
  `.extraction-list`, risk tiers, `.just-*`, `.extraction-card` spine,
  `.extraction-head` untouched; `.just-*` neighbors untouched.
- `crates/ui/assets/index.html`: three `<li>` appended to the Hotspots help
  `<ul>` (collapsed summary + click to open; playbook lists destination
  files + the apply command + the UI never applies/never changes files;
  three tiers + automatic rollback + migrate --preview first). Each under
  400 stripped chars, pure ASCII, `&mdash;`/`&lt;` entities. Locked phrases
  `Proposed extractions section` / `Preview after extraction` untouched.

## What was deliberately NOT done

- No edits to `extraction_card_story.rs`, `static_html*.rs`,
  `help_center_readability.rs`, `proposals_payload.rs`, `dto_builders.rs`,
  `preview_hypothetical.rs`, `common.rs`, `api.js`, `preview.js`,
  `graph.js`, `main.js`, `themes.css`, `theme.js`, `crates/ui/src/**`,
  `crates/cli/**`, `tools/**`, fixtures. No git stash, no branch change.
- No new fetch surface (`/api/repo-info` per card rejected per shape P9);
  no CSS-animated height; no element ids; no one-click apply.
- No assertion weakened at any point; the one Green-phase fix moved the
  `--tile-bg` token into the `.extraction-disclosure` base rule rather than
  relaxing the token test.

## RGB evidence

- Red (gate 1): `python tools/testrun.py --distill -p ariadne-ui` ->
  `201 passed / 8 failed / 0 ignored / 2 suites`, exit 1. Failing list =
  exactly the 8 new-beat tests (`test_details_collapsed_by_default`,
  `test_disclosure_button_contract`, `test_summary_meta_and_risk_visible`,
  `test_file_plan_rows_markup`, `test_apply_recipe_is_honest`,
  `test_playbook_escape_discipline`, `test_playbook_styles_declared_tokens`,
  `test_help_playbook_copy`); every failure a runtime assert panic naming
  the missing markup/helper, not a compile error.
  `test_no_new_data_sources` already green (predicted). Capture:
  `target/testrun-capture.txt`.
- Green (gate 2): same command -> `209 passed / 0 failed / 0 ignored /
  3 suites`, exit 0. All 9 `extraction_card_story::test_* ... ok` visible
  in the capture, story file byte-identical (absent from the diff); C4/P7
  consumers (`static_html`, `static_html_sections`, `help_center_
  readability`, `proposals_payload`, `dto_builders`, `export_order`,
  `preview_hypothetical`) green UNEDITED.
- Gate 3: `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)` + `ui_smoke: PASS` (live boot renders the
  playbook markup; T-B feed provides real `target_files`).
- Gate 4: `python tools/theme_token_check.py --check-added --base 6a48a85`
  - silent (zero added hex).
- Gate 5: `python tools/hygiene_check.py` exit 0 (digest-verified
  `### generic PASS (exit 0)`; output is warn-only R3 rows, all
  pre-existing, none in touched files; no R1/R2 rows).
- Gate 6: `python tools/encoding_guard.py --staged` exit 0 (silent) on the
  real staged blobs; staging itself went through the sanctioned distill
  wrapper because raw `git add` is allowlist-denied this session - see D2.
- Gate 7: single `commit_scoped.py` commit with the six handoff globs; the
  landing hash is the commit this journal ships in (visible via
  `git show`).

## Deviations (numbered for the reviewer)

1. Per-file chip wrapper: the C3 markup skeleton shows a
   `.extraction-chips` wrapper inside each `.extraction-file` row, but
   survival rule 2 (binding, story-test-8 first-source-index) forbids the
   plural substring inside any helper defined before the builder. Per rule
   2's own instruction the wrapper is composed ONLY in the builder
   (Extract/needs rows); file-plan chips render directly inside the flex
   `.extraction-file` row (the singular `.extraction-chip` comes via the
   existing `extractionChips` escape path). Green evidence confirms rule
   2's analysis; no story test was amended.
2. Gate 6 staging route: raw `git add` is allowlist-denied in this session
   (boilerplate deny-rule refusal), so staging used the handoff's sanctioned
   refusal route, `python tools/distill.py -- git add -- <the six touch-set
   paths>`, then `encoding_guard.py --staged` on the real staged blobs
   (exit 0), then the handoff's `commit_scoped.py` (which re-adds no-ops and
   commits `--only` the six paths). All authored content went through
   Write/Edit tools only and is pure ASCII.

## Observations

- Red digest shows `2 suites` vs Green `3 suites`: the same canonical-line
  parser artifact T-B recorded (failing runs count only the suite reporting
  failures); totals reconcile (201+8 = 209).
- hotspots.js lands at 465 of 1000 - below the handoff's ~520-570 estimate
  (the estimate assumed longer helpers); all per-function caps met.
- Disclosure interaction: the summary button toggles only the enclosing
  card's details (stopPropagation keeps the card's own click on
  `selectHotspotRow`); both surfaces (hotspot list, inspector) share the
  builder with no ids. String-level locks only - no browser-driver oracle
  exists, as the shape states plainly; reviewer spot-check is the oracle.
