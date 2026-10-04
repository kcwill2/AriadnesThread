# T3 Handoff - ui-progress-parity: overlay step-row renderer (cell-glyph bars, dim pending)

Milestone key `ui-progress-parity`. Execution slot 3 of 3. Predecessors:
T1 (DTO fields + `server_api.rs` fixtures) and T2 (`run_ui` seeds rows from
boot) must be merged. Browser ships last so no intermediate state ever has
a half-renderer. Reviewer adjudicates via the `review-gate` skill.

USER DECISIONS (resolved 2026-10-04 - these ARE the spec, no verbal briefing
needed, do not relitigate):
- DP-1: all five step rows are visible from boot; not-yet-started rows are
  DIM (`step-pending`: muted text, no message text). Never hide rows.
- DP-2: bars are literal 20-cell block-glyph bars in the browser -
  `[████░░░░░░] pp%` look - with the CLI's own arithmetic
  `filled = Math.floor(20 * percent / 100)`, remainder cells light glyphs,
  total always 20 cells. NOT a smooth CSS width fill.
- DP-3: `done` rows are accent-colored only (the `--cyan` token family).
  No green checkmarks, no verdict green/amber/red on progress rows -
  semantic verdict colors stay locked to verdicts.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-progress-parity-shape.md` lines 170-192 (frontend contracts:
   renderer name, legacy fallback, ids, classes) and line 66-73 (I7 bar
   arithmetic, I8 theme safety).
3. `crates/ui/assets/main.js` lines 232-252 (`showProgress`/`updateProgress`/
   `hideProgress` - names kept, static_html asserts them, standalone callers
   rely on them).
4. `crates/ui/assets/api.js` lines 7-15 (standalone export synthetic branch)
   and lines 19-75 (server poll branch; line 32 is the exact call you
   replace).
5. `crates/ui/assets/index.html` lines 26-29 (overlay container +
   `loadingProgress` bar + `loadingText`/`loadingDetail`).
6. `crates/ui/tests/ui_test/static_html.rs` lines 512-528
   (`test_ui_loading_overlay_and_progress` - the needles you keep AND
   extend), plus `common::page_content()` (the bundled single-file page:
   html+css+js all count).
7. `crates/cli/src/commands/progress.rs` line 74 - the cell pattern you are
   imitating: 20 cells of U+2588 fill + U+2591 remainder.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/ui-progress-parity-t3-handoff-journal.md (checkpoint journal; create
  alongside)
- crates/ui/assets/index.html (overlay gains `loadingSteps` container div;
  keep `loadingOverlay`/`loadingProgress`/`loadingText`/`loadingDetail` as
  the legacy fallback UI)
- crates/ui/assets/main.js (add `renderProgressSnapshot`; keep the three
  legacy functions' names and behavior)
- crates/ui/assets/api.js (poll branch + standalone branch feed the one
  renderer)
- crates/ui/assets/styles.css (~40 token-only lines, appended step-row
  block; `styles.css` is 1422 and invisible to the R2 size gate - that is a
  known ledger finding, do NOT "fix" it, just do not grow anything else)
- crates/ui/tests/ui_test/static_html.rs (update the overlay test; add
  formula/width/token-only assertions)
- Cargo.lock (only if regenerated; no new deps)

You MUST NOT touch: `themes.css`, `crates/ui/assets/theme.js`,
`crates/ui/src/**` (T1 merged; DTO is consumed by wire shape only),
`crates/ui/tests/ui_test/server_api.rs` (T1's), any `crates/cli/**`,
`crates/pipeline/**`, `tests/fixtures/**`, `tests/golden/**`,
`tools/theme_token_baseline.json`, the verdict/heat CSS token definitions,
`index.html` inline styles beyond the one container div.
No `git stash`, no branch change.

## What to build

`main.js` - add `renderProgressSnapshot(progress)`:
- `!progress.steps || progress.steps.length === 0` -> legacy single-bar
  path: delegate to the existing `loadingProgress` width behavior (the
  honest fallback; standalone exports and legacy fixtures land here).
- else draw one `.step-row` per step into `#loadingSteps` (create it once):
  chip `[stage/total_stages]`, `.step-label`, 20-cell `.step-bar`,
  right-aligned `.step-percent` (`pp%`), `.step-detail` (the message).
- Bar cells, exact arithmetic (I7, mirrors `format_step_bar`'s integer
  math): `var filled = Math.floor(20 * percent / 100);` with
  `percent = Math.min(percent, 100)`, filled cells = 20 - remainder,
  total cells ALWAYS 20. Glyphs are written as JS unicode escapes so the
  authored file stays pure ASCII: `"\u2588"` (█ fill), `"\u2591"` (░
  remainder). Wrap the bar in `[` `]` to read `[████░░░░░░░░░░░░░░░░░░]`.
  Monospace font (`.step-bar`).
- Status classes per row: `step-done`, `step-active`, `step-pending`
  (from `step.status`). Pending rows render the row dimmed with EMPTY
  detail (DP-1). Done rows: accent text only (DP-3).
- `renderProgressSnapshot` composes DOM via `textContent` (no innerHTML
  with untrusted message strings).

`index.html` - add `<div id="loadingSteps"></div>` inside the existing
overlay panel; the legacy bar/text elements remain; when rows render, hide
the legacy bar (`display: none` on the legacy bar row) and `hideProgress`
keeps working for both paths (clear `#loadingSteps` content on hide).

`api.js`:
- server poll branch: replace the single line 32 call
  `updateProgress(status.progress.percent, status.progress.phase + ': ' + status.progress.message)`
  with `renderProgressSnapshot(status.progress)` (the surrounding
  `status.progress.state === 'loading'` needle stays - static_html locks
  it). The `ready`/`error` branches are unchanged.
- standalone export branch (lines 7-15): build a same-shaped local
  snapshot `{state:'loading', phase:'', percent, message:'',
  total_stages: 3, steps:[...]}` with the 3 synthetic steps (25/65/100
  per the existing calls), statuses pending/active/done as the sequence
  advances, and route it through `renderProgressSnapshot` - ONE renderer,
  no second one.

`styles.css` - appended block only, `var()` tokens ONLY, zero hex literals
(I8): `.step-row`, `.step-index`, `.step-label`, `.step-bar` (monospace,
letter-spacing 0), `.step-percent` (right-aligned tabular width),
`.step-detail`, plus `.step-pending` (muted via `var(--text-muted)`),
`.step-active` (chrome via existing `--cyan`-family), `.step-done` (accent
via existing tokens; DP-3 - no green/amber/red, no verdict tokens).

`static_html.rs` - keep every existing needle in
`test_ui_loading_overlay_and_progress` (lines 516-527 all stay true:
legacy ids + three legacy function names + both poll needles), and extend:
- `html.contains("id=\"loadingSteps\"")`
- `html.contains("function renderProgressSnapshot")`
- `html.contains("renderProgressSnapshot(status.progress)")`
- `html.contains("Math.floor(20 * percent / 100)")` (I7 formula source
  lock, exact needle)
- `html.contains("total_stages")` and glyph escapes present as source:
  `html.contains("\\u2588")` and `html.contains("\\u2591")` (literal
  backslash-u needles in the bundle source)
- token-only assertion for the new CSS block: extract the appended
  step-row rules and assert no `#`-hex color appears in them (mirror the
  style of the existing token assertions in this file).

## RGB protocol
1. Red: FIRST rewrite `test_ui_loading_overlay_and_progress` with the
   extension list above. It compiles (string asserts on the bundled page)
   and MUST fail today at
   `assertion failed: html.contains("id=\"loadingSteps\"")` (first new
   needle). All pre-existing needles pass at Red time. Capture the failure.
2. Green: edit the four assets; re-run until green. NEVER weaken an
   existing needle to pass.
3. Refactor: extract the cell-builder into one named helper (e.g.
   `buildStepBarCells`), honest docs, re-run.

## Hygiene (review-enforced)
Functions < 100 lines (`renderProgressSnapshot` and its cell helper stay
far under); `main.js` 513 -> ~560, `index.html` 562 -> ~575 - container
div only, no new inline styles beyond it; no single line anywhere near
2000 chars (step rows compose element-by-element); meaningful names, no
banned names. `python tools/hygiene_check.py` exit 0.

## Frozen locks
- `themes.css` / `theme.js` / `tools/theme_token_baseline.json` byte-
  identical: `python tools/theme_token_check.py --check-added --base main`
  reports zero added hex (I8; G4 of the milestone acceptance).
- Verdict/heat semantic colors untouched: reviewer eyeballs the styles.css
  diff touches no `--green`/`--amber`/`--red`/heat-ramp rule (DP-3).
- Legacy overlay contract: `loadingOverlay`/`loadingProgress`/
  `loadingText`/`loadingDetail` ids and the three legacy JS function names
  remain (static_html + `graph.js`/standalone callers).
- `/api/*` routes unchanged (this handoff is assets+test only); readiness
  contract (I4) untouched; `crates/pipeline/**` and
  `crates/cli/src/commands/progress.rs` byte-identical (reviewer diff).
- `tests/fixtures/**` frozen.

## Gates (exact commands, timeout 600000 ms each; scoped only)
1. `python tools/testrun.py --distill -p ariadne-ui` -- Red run: the
   rewritten overlay test fails at the named needle; canonical line cited.
2. `python tools/testrun.py --distill -p ariadne-ui` -- Green run:
   canonical line `0 failed`.
3. `python tools/theme_token_check.py --check-added --base main` -- zero
   added hex literals (I8).
4. `python tools/hygiene_check.py` -- exit 0.
5. `python tools/encoding_guard.py --staged` -- exit 0 before committing.
6. Commit with
   `python tools/commit_scoped.py --scope "crates/ui/assets/index.html" --scope "crates/ui/assets/main.js" --scope "crates/ui/assets/api.js" --scope "crates/ui/assets/styles.css" --scope "crates/ui/tests/ui_test/static_html.rs" --scope "tasks/ui-progress-parity-t3-handoff-journal.md" -m "feat(ui): boot overlay step-rows with CLI cell bars (ui-progress-parity T3)"`
   (never `git add .`).

## Discipline clauses
- UTF-8: Write/Edit tools only; authored content pure ASCII - the block
  glyphs exist ONLY as `\u2588`/`\u2591` escapes in JS source, never as raw
  bytes in any file you author (encoding_guard must stay green).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`) are a reviewer blocker;
  use distilling entry points (`docs/specs/tool-output-distillation.md`).
  `ui_smoke` (milestone G3) is the orchestrator's closeout run, not yours.
- Manual three-theme eyeball (R5): reviewer notes steampunk legibility is
  token-guarded, not taste-guarded; flag in report what you visually
  checked if you ran the UI locally.
- After EACH per-module Green gate append a 1-3 line checkpoint to
  `tasks/ui-progress-parity-t3-handoff-journal.md`.
- If context exhaustion looms: STOP, return a disk-state inventory; never
  resume a dead session - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ui-progress-parity-t3-handoff.md --scope crates/ui --cmd "python tools/testrun.py --distill -p ariadne-ui"`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (needle that failed).
3. Green evidence (distill digest header + canonical line + capture path;
   theme_token_check exit 0 line). 4. Deviations + reasons. 5. Out-of-scope
observations (report-only). Never paste raw tool output.

Reviewer-gate reference: adjudicate via `review-gate`; this handoff feeds
milestone gates G1/G4/G5 (G3 `ui_smoke` and G7 `--baseline` are fresh-
session/orchestrator closeout runs per the sizing law).
