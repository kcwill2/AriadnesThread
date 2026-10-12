# W-3a Handoff - ui-extraction-wizard: new asset registration + stepper shell + steps 0-1

AMENDED IN PLACE 2026-10-12 (planner) after the implementer halted pre-Red at
the boot audit (journal `tasks/ui-extraction-wizard-w3a-journal.md`, journal
commit `e22f4a6`, parent `1f561a0`): the R7 registration chain was under-counted.
Two asset-count lock files join the touch-set under the AMENDMENT section below
(playbook C5 / clarity T-B keep-literal discipline). Everything else in this
handoff is unchanged.

Depends on W-1 landed (shared `main.rs` insert discipline and the split
module tree; this hand needs no DTO, so W-2a/W-2b may still be in flight, but
`main.rs` contention keeps the hands serialized on main).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-extraction-wizard-shape.md` sections 1 (capability row 14), 2
   (W5/W10), 3 (C1 stepper states + chrome, C6 volume rules, C7 wizard
   state), 5 (module map), 8 (R7 asset chain); `tasks/ui-extraction-wizard-kickoff.md`
   sections 3, 5, 6.
3. Registration-chain anchors: `crates/ui/src/lib.rs:12-24` (the
   `include_str!` consts), `crates/ui/src/server.rs:48-76` (route table),
   `crates/ui/src/routes/api_payloads.rs` (the asset handlers),
   `crates/ui/assets/index.html:591-601` (script tags),
   `crates/ui/src/export.rs:35-48` (the standalone inline tuple; its lock
   `test_standalone_export_inlines_all_split_assets` lives in
   `crates/ui/tests/ui_test/static_html.rs:949` and stays UNEDITED),
   `crates/ui/tests/ui_test/theme_contract.rs:384-449` (the two asset-count
   locks the AMENDMENT section below updates), `tools/ui_smoke.py:58-60` and
   its `ROUTE_ASSERTIONS` (:61-71).
4. Model asset for structure and doc idiom: `crates/ui/assets/traversal.js`
   (newest single-purpose asset) and the modal infrastructure at
   `index.html:423-585` (`#modalOverlay` + `.modal`).

## Objective and preserved invariants

Register the new `crates/ui/assets/extraction-wizard.js` through every site of
the chain, and build the wizard stepper shell plus steps S0 (Overview) and S1
(Files to create) rendered from the EXISTING boot payload. No provider fetch
yet (steps 2-4 are W-3b), so this hand must be honest about the payload's
display caps.

Invariants this hand must preserve:
- R7: the asset registration chain is FIVE registration sites - (1) the
   `include_str!` const in `lib.rs`, (2) the route in `server.rs` + the handler
   in `routes/api_payloads.rs`, (3) the `index.html` script tag, (4) the
   standalone-export inline tuple in `export.rs` (its lock lives in
   `static_html.rs`), (5) the `tools/ui_smoke.py` probe row. A missing
   registration site fails `ui_smoke` or the export lock. Adding the asset also
   detonates TWO asset-count locks that must be amended in the SAME commit as
   the script tag: the `theme_contract.rs` script-tag count + whitelist
   (AMENDMENT rows 2-3). A missing amendment fails `theme_contract`.
- W10: no sort is introduced; the panel cards render in payload order and
  stay collapsed by default (`test_order_and_preview_locks` and
  `test_details_collapsed_by_default` stay green UNEDITED - the entry button
  adds a control, it does not change disclosure behavior).
- W9: the boot payload grows by nothing.
- C7: one module-level wizard state object
  (`{planId, step, selectedFile, manifest, dryRun, error}`), reset on modal
  open for a different plan, on 409 stale generation (wired in W-3b), and on
  payload swap; wizard-scoped element ids (`wizardStep1` etc.); step
  transitions use the `hidden` attribute (string-lockable).
- ASCII-only labels; Back/Next are real `<button>`s; `Esc` closes via the
  existing `closeModals()` path; no animation (keeps `hidden` and the string
  locks honest); ordered step list with `aria-current="step"`.
- Escape discipline: every interpolated payload value passes through
  `escapeHtml` (house idiom).
- The card summary keeps the summary-card-first interaction model: the entry
  control opens the wizard; the card itself stays collapsed by default.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/ui-extraction-wizard-w3a-journal.md (checkpoint journal)
- crates/ui/assets/extraction-wizard.js (NEW)
- crates/ui/src/lib.rs (one `include_str!` const, in the existing const block)
- crates/ui/src/server.rs (one route + the use line)
- crates/ui/src/routes/api_payloads.rs (one asset handler)
- crates/ui/assets/index.html (script tag + the wizard modal shell + the
  entry button hook)
- crates/ui/src/export.rs (ONE tuple entry in `standalone_template` + the
  matching const on the `use crate::{...}` import - AMENDMENT row 1)
- crates/ui/tests/ui_test/theme_contract.rs (ONE whitelist row + the
  script-tag count 9 -> 10 and its message - AMENDMENT rows 2-3)
- crates/ui/assets/hotspots.js (the entry button on the proposal card and its
  open call - bounded; the renderer's existing structure stays)
- crates/ui/assets/styles.css (one contiguous wizard chrome block, net-capped
  per the shape's volume rule; note `.css` is invisible to R2, the cap is the
  discipline)
- tools/ui_smoke.py (one `ROUTE_ASSERTIONS` row for
  `/app/extraction-wizard.js`)
- crates/ui/tests/ui_test/extraction_wizard_render.rs (NEW)
- crates/ui/tests/ui_test/main.rs (one mod line: insert
  `mod extraction_wizard_render;` immediately after
  `mod extraction_wizard_api;` if present, else after `mod extraction_contrast;`

  , always before `mod file_view;`)

You MUST NOT touch: `crates/ui/src/dto*/**`, `crates/ui/src/state.rs`,
`crates/ui/src/routes/api_extraction.rs`, `crates/cli/**`, the card test
files, `themes.css`, `theme.js`, `preview.js`, `api.js`, `graph.js`,
`traversal.js`, `main.js`, `tools/theme_token_baseline.json`, `tests/**`.

## What to build

1. Registration chain first (compile prerequisite, not the Red): create
   `extraction-wizard.js` with its module header doc, the C7 wizard state
   object, and the open/close plumbing (real minimal code); add the const, the
   route, the handler, the script tag. This lands the asset on the wire. The
   script tag MUST land in the same commit as the two lock amendments of the
   AMENDMENT section below (the `export.rs` tuple entry and the
   `theme_contract.rs` count/whitelist) - they are part of the registration
   chain, NOT part of the Red: without them the suite is red before any Red
   test exists. Suggested tag position: immediately before `/app/theme.js`
   (relative-order locks unaffected; wizard globals exist long before any card
   click).
2. `extraction-wizard.js` stepper: the ordered step list (S0..S6 labels per
   C1), `aria-current="step"`, step sections toggled by `hidden`, Back/Next
   buttons, `Esc` -> `closeModals()`. Step renderers:
   `renderWizardStepOverview` (S0: plan id, target file, collateral
   before -> after via `&rarr;`, saving, risk tier, consumer TU count, total
   symbols - payload as-is) and `renderWizardStepFiles` (S1: one row per
   destination file from the payload's capped `target_file_groups` - real
   path, symbol count, the honest cap note when `target_files_truncated` is
   set, selectable rows feeding C7 state).
3. The card entry: an "Open wizard" button on each proposal card in
   `hotspots.js` whose handler opens the modal for that plan id (resetting the
   wizard state per C7).
4. `styles.css`: one contiguous wizard block (modal steps, step list, file
   rows, the `just-more` chip reuse for `+K more`).
5. The `ui_smoke.py` row. Do NOT "fix" the known route-table drift (the
   probe list predates `traversal.js`/`provenance.js`/`theme.js` and its
   docstring says "six /app/*.js scripts"; that drift is a deferred relay
   row, out of scope here): add exactly the one wizard row.

## RGB protocol (per module, in this file order)

Red: after the registration chain compiles, author
`crates/ui/tests/ui_test/extraction_wizard_render.rs` (locks below) and run
Red BEFORE the stepper content exists. Expected failures (verbatim shape):
the modal-shell and step locks panic on the missing wizard markup
(`index.html must contain the wizard modal shell`,
`extraction-wizard.js must declare the wizard state reset`, the step-order
lock names the first missing step id, the entry-button lock reports the
missing Open-wizard handler, the caps lock reports the missing truncation
handling). The registration locks (script tag, const, route, handler,
ui_smoke row) pass at Red by construction - that is the documented order, not
a weak Red. The two AMENDMENT locks (`export.rs` tuple,
`theme_contract.rs` count/whitelist) are amended with the registration chain
and are also green at Red; a Red run that shows a `theme_contract` or
`static_html.rs` failure means the amendment was not applied - fix the
amendment, do not weaken the new test file to hide it.
Green: author the stepper shell, steps S0/S1, the entry button and the style
block; re-run until exit 0.
Refactor: keep each renderer small; the wizard state transitions live in one
place; `extraction-wizard.js` stays far under the line ceiling (the asset is
new; shape budgets it so it stays under 1000 even after W-3b).

## Required tests (in extraction_wizard_render.rs)

1. Lock: `index.html` loads `/app/extraction-wizard.js` (script tag present).
2. Lock: `server.rs` registers `get(` for `/app/extraction-wizard.js` and the
   handler serves `ariadne_ui::EXTRACTION_WIZARD_JS` verbatim (route + const
   chain).
3. Lock: the wizard modal shell exists in `index.html` (the `.modal` panel
   with the ordered step list) and uses `hidden` for step switching.
4. Lock: the step list order in the asset is S0 overview, S1 files, S2
   manifest, S3 impact, S4 dry run, S5 execute, S6 after apply (C1 order,
   string order lock).
5. Lock: `aria-current="step"` appears in the stepper chrome; Back/Next are
   `<button>` elements.
6. Lock: the entry button exists on the proposal card path (the handler name
   and the reset-on-different-plan call).
7. Lock: caps honesty - S1 reads `p.target_files_truncated` (or the wizard
   manifest truncation flag once W-2b's DTOs land; at W-3a the payload flag)
   and renders the `+K more` chip idiom (`just-more` precedent).
8. Lock: the new asset is ASCII-only (no byte > 0x7F) and every interpolated
   payload value passes through `escapeHtml`.
9. Lock: `ui_smoke.py` `ROUTE_ASSERTIONS` contains the wizard route row.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names, no
single-letter locals, intention-revealing names (shape section 9's renderer
names are binding: `renderWizardStepFiles`, `renderWizardStepManifest`).
`hotspots.js` is 680 lines at HEAD: the entry button is a bounded delta.
`index.html` is 603 lines: the modal shell plus the script tag is a bounded
delta. The two amended files are far under the ceilings at HEAD:
`theme_contract.rs` is 450 lines (+~3 from the amendment), `export.rs` is 82
(+~2 from the amendment). `python tools/hygiene_check.py` exit 0.

## Frozen locks

- `preview.js`, `api.js`, `graph.js`, `traversal.js`, `main.js`, `theme.js`,
  `themes.css`, `tools/theme_token_baseline.json` byte-identical; zero added
  hex, proven by the added-scan below.
- `test_order_and_preview_locks` and `test_details_collapsed_by_default`
  green UNEDITED (W10); the card-family locks survive except by W-3b's C5
  table (this hand amends NONE of them).
- Deliberate carve-out (this hand's ONLY test-file amendment): the AMENDMENT
  table below amends the `theme_contract.rs` script-tag count and its asset
  whitelist, plus the `export.rs` inline tuple. `static_html.rs` and every
  other UI test file stay byte-identical (`static_html.rs:949`'s export lock
  survives unedited - see the AMENDMENT evidence).
- `tests/fixtures/**`, `tests/golden/**` frozen.

## AMENDMENT (2026-10-12, planner; playbook C5 / clarity T-B keep-literal discipline)

Trigger: the W-3a implementer halted pre-Red at the boot audit (journal
`tasks/ui-extraction-wizard-w3a-journal.md`, commit `e22f4a6`, parent
`1f561a0`). The audit found the registration chain under-counted: adding the
mandatory site-3 script tag detonates two locks whose files were outside the
touch-set, making gates 1-3 unsatisfiable. The chain is FIVE registration
sites plus TWO asset-count lock sites. Precedent: every prior asset
(`theme.js`, `provenance.js`) shipped these same updates inside its own hand.

The amendment table (exact before/after; the reviewer diffs each hunk against
this table, same discipline as the C5 card-lock table):

| # | Lock | File and site | Before (HEAD `e22f4a6`) | After | Why no keep-literal path |
|---|---|---|---|---|---|
| 1 | standalone-export inline tuple | `crates/ui/src/export.rs:35-48` (`standalone_template`) + the `use crate::{...}` import at :2-5 | tuple ends `("main.js", MAIN_JS)`, `("theme.js", THEME_JS)`; no `EXTRACTION_WIZARD_JS` in the import | add `("extraction-wizard.js", EXTRACTION_WIZARD_JS),` immediately before `("theme.js", THEME_JS),` (mirrors script order); import gains `EXTRACTION_WIZARD_JS` between `API_JS` and `FILE_VIEW_JS` | the tuple IS the inlining mechanism: a 10th `<script src="/app/extraction-wizard.js">` tag survives the replacement loop verbatim, so the export lock `!content.contains("src=\"/app/")` (`static_html.rs:992`) fails. Additive by design; the loop replaces tags by name |
| 2 | script-tag count | `crates/ui/tests/ui_test/theme_contract.rs:428-432` | `assert_eq!(html.matches("<script src=").count(), 9, "exactly nine classic asset scripts")` | `... .count(), 10, "exactly ten classic asset scripts")` | MECHANICALLY OPPOSED to the required site-3 script tag: `index.html:591-601` carries exactly 9 `<script src=` tags at HEAD; the new asset makes the count 10. The assertion is a literal count of shipped asset tags, so NO keep-literal path exists - the lock's intent (count == shipped asset scripts) is preserved only by moving the number with the asset |
| 3 | asset whitelist | `crates/ui/tests/ui_test/theme_contract.rs:435-449` (`tag_start_is_asset`) | 9-entry list ending `"/app/theme.js",` | add `"/app/extraction-wizard.js",` immediately before `"/app/theme.js",` (mirrors script order) | the whitelist panics on any other `src=` tag (`"unexpected script tag with src: {tag}"`); it is a membership list whose intended extension mechanism IS an added entry. Not a plan-literal test - it cannot enumerate an asset that did not exist when written |

Why the amended tests cannot be plan-literal, stated plainly: rows 2 and 3 are
locks over the SHIPPED asset set, not over behavior. Row 2's value 9 is
arithmetically derived from the current `index.html` script tags; the hand's
own required test 1 mandates the tag that makes it 10, so keeping the test
literal and satisfying required test 1 simultaneously is impossible. Row 3's
list is the whitelist's extension mechanism. Neither amendment weakens a
behavioral contract: both move in lockstep with the asset exactly as the
`theme.js` and `provenance.js` landings did.

Claim verification (all read 2026-10-12, not assumed):

- `static_html.rs:949` `test_standalone_export_inlines_all_split_assets`
  SURVIVES UNEDITED. Its body (lines 949-998) is the generic no-survivor lock
  `!content.contains("src=\"/app/")` plus per-asset presence markers (named
  functions such as `function compileMatcher`); it enumerates no count and no
  asset list, so a 10th inlined asset breaks nothing. The fix for site 4 is
  the `export.rs` tuple entry alone.
- Further consumers of the export tuple entry - each swept CLEAN, no
  amendment row needed: `export_order.rs` (payload emission-order byte lock;
  no asset surface at all); `asset_injection_hardening.rs` (fixed 9-element
  arrays; additive-safe - and the wizard must NOT re-declare `escapeHtml`,
  which required test 8 enforces); `d27_node_provenance.rs:280`
  (provenance-specific presence/absence locks); `static_html.rs:760`
  (relative script-order walk) and `static_html.rs:792` (fixed 8-element
  serve-route list); `tools/ui_smoke_tests.py:86-95` `fully_compliant_routes`
  (builds its stub table FROM `ROUTE_ASSERTIONS`, so the one added row is
  safe); `crates/cli/tests/payload_tail_bench.rs:571` (two-process byte
  equality - both exports carry the new asset identically); `tests/integration`
  m15/m16/m17 export tests (presence markers only); `tests/golden/**` and
  `tests/fixtures/**` (no HTML-export goldens; stay frozen); `ui_smoke.py`'s
  "six /app/*.js scripts" docstring drift (the known deferred relay row -
  NOT fixed here, exactly one route row added).
- The wizard script tag position: immediately before `/app/theme.js`
  (after `/app/main.js`). The relative-order walk
  (`test_script_tag_order_preserves_original_execution_order`) is unaffected
  by any insertion point, and the wizard globals must merely exist before a
  card click, which happens after every script has evaluated.

Sizing re-check after the amendment (planner, 2026-10-12): scope still ONE
module (`crates/ui` assets + src + tests/ui_test) plus the `tools/ui_smoke.py`
route row - inside the one-module-dir law. Gates stay at 10 (the amendment
folds into gates 3 and 9; no new gate is required because gates 1-3 already
run the whole `-p ariadne-ui` suite that carries all three amended locks).
Diff grows by ~10 lines (three bounded lock edits) inside the W-3a estimate.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-ui` (baseline canonical line)
2. `python tools/testrun.py --distill -p ariadne-ui` (Red: exit 1; named
   failing tests verbatim)
3. `python tools/testrun.py --distill -p ariadne-ui` (Green: exit 0;
   canonical line = baseline + the new tests)
4. `python tools/theme_token_check.py --check-added --base <start-rev>`
   (silent; the combined form is the canonical gate - the bare form is a
   usage error per the ledger)
5. `python tools/hygiene_check.py` (exit 0)
6. `python -m unittest discover -s tools -p "ui_smoke_tests.py"` (REQUIRED:
   this hand edits `tools/ui_smoke.py`'s route table; the CI discover pattern
   for the full run is `-p "*_tests.py"`, the scoped form is this one)
7. `python tools/distill.py -- python tools/ui_smoke.py` (exit 0:
   `ui_smoke: PASS` - the runtime proof the new asset route boots and the
   route table mirrors `build_router`)
8. `python tools/encoding_guard.py --staged` (exit 0)
 9. `python tools/commit_scoped.py --scope crates/ui/assets/extraction-wizard.js
    --scope crates/ui/src/lib.rs --scope crates/ui/src/server.rs --scope
    crates/ui/src/routes/api_payloads.rs --scope crates/ui/src/export.rs
    --scope crates/ui/assets/index.html --scope crates/ui/assets/hotspots.js
    --scope crates/ui/assets/styles.css --scope tools/ui_smoke.py --scope
    crates/ui/tests/ui_test/extraction_wizard_render.rs --scope
    crates/ui/tests/ui_test/theme_contract.rs --scope
    crates/ui/tests/ui_test/main.rs --scope tasks/ui-extraction-wizard-w3a-journal.md
    -m "feat(ui): extraction-wizard asset registered + stepper shell with steps 0-1"`
    (never `git add .`; the 2026-10-12 amendment adds the `export.rs` and
    `theme_contract.rs` scopes to this original command)
 10. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 10 (at the bound; the commit-cycle trio is gates 8-10 and the
ui_smoke pair 6-7 exists because this hand edits the tool - all itemized in
the journal per the M28 T0b finding-1 law). The 2026-10-12 amendment added no
gate: the three lock edits ride inside gates 1-3's suite run and gate 9's
scope list; gate 6 (the scoped tools discover) stays mandatory unchanged.

## Discipline clauses

- UTF-8: Write/Edit tools only; pure-ASCII authored content; no em dashes; in
  the JS asset use `&rarr;` / `&#8592;` entities where a glyph is wanted.
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands are a reviewer blocker: distilling entry points
  only (`ui_smoke` goes through `distill.py` per the standing contract).
- Checkpoint the journal after each Green gate.
- Context exhaustion: STOP with a disk-state inventory; successor resumes via
  `python tools/checkpoint_guard.py resume --spec
  tasks/ui-extraction-wizard-w3a-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (named failing tests,
  verbatim panic lines). 3. Green evidence (digest header + canonical line +
  capture path; the ui_smoke PASS line). 4. Deviations + reasons. 5.
Out-of-scope observations (report-only, including the ui_smoke route-table
drift if it grows). No raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly.
