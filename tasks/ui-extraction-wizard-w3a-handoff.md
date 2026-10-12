# W-3a Handoff - ui-extraction-wizard: new asset registration + stepper shell + steps 0-1

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
   `crates/ui/src/routes/api_payloads.rs` (the asset handlers), `crates/ui/assets/index.html:591-601`
   (script tags), `crates/ui/tests/ui_test/static_html.rs` (the
   standalone-export inline lock), `tools/ui_smoke.py:58-60` and its
   `ROUTE_ASSERTIONS` (:61-71).
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
- R7: the asset registration chain is five sites - (1) the
  `include_str!` const in `lib.rs`, (2) the route in `server.rs` + the handler
  in `routes/api_payloads.rs`, (3) the `index.html` script tag, (4) the
  standalone-export inline lock in `static_html.rs`, (5) the `tools/ui_smoke.py`
  probe row. A missing site fails `ui_smoke` or the export lock.
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
   route, the handler, the script tag. This lands the asset on the wire.
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
a weak Red.
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
delta. `python tools/hygiene_check.py` exit 0.

## Frozen locks

- `preview.js`, `api.js`, `graph.js`, `traversal.js`, `main.js`, `theme.js`,
  `themes.css`, `tools/theme_token_baseline.json` byte-identical; zero added
  hex, proven by the added-scan below.
- `test_order_and_preview_locks` and `test_details_collapsed_by_default`
  green UNEDITED (W10); the card-family locks survive except by W-3b's C5
  table (this hand amends NONE of them).
- `tests/fixtures/**`, `tests/golden/**` frozen.

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
   crates/ui/src/routes/api_payloads.rs --scope crates/ui/assets/index.html
   --scope crates/ui/assets/hotspots.js --scope crates/ui/assets/styles.css
   --scope tools/ui_smoke.py --scope
   crates/ui/tests/ui_test/extraction_wizard_render.rs --scope
   crates/ui/tests/ui_test/main.rs --scope tasks/ui-extraction-wizard-w3a-journal.md
   -m "feat(ui): extraction-wizard asset registered + stepper shell with steps 0-1"`
   (never `git add .`)
10. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 10 (at the bound; the commit-cycle trio is gates 8-10 and the
ui_smoke pair 6-7 exists because this hand edits the tool - all itemized in
the journal per the M28 T0b finding-1 law).

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
