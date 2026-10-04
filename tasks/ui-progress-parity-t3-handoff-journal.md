# T3 Journal - ui-progress-parity overlay step-row renderer

Handoff: `tasks/ui-progress-parity-t3-handoff.md`. Base HEAD: 587c409 (T2).
Predecessors landed: T1 DTO `5eff824`, T2 producer `587c409`.

## Checkpoints

- [Red start] Rewrote `test_ui_loading_overlay_and_progress` with the T3
  extension needles (loadingSteps id, renderProgressSnapshot, formula lock,
  glyph escapes, total_stages, token-only CSS block). Legacy needles kept
  verbatim. Running gate 1 to capture the predicted failure.
- [Red confirmed] Gate 1: `183 passed / 1 failed / 0 ignored / 2 suites`,
  sole failure `assertion failed: html.contains("id=\"loadingSteps\"")` at
  static_html.rs:540 - exactly the predicted first new needle.
- [Green confirmed] Gate 2: `184 passed / 0 failed / 0 ignored / 3 suites`.
  Four assets edited: index.html (container div), main.js
  (`renderProgressSnapshot` + hideProgress clears rows), api.js (both
  branches feed the one renderer), styles.css (token-only step-row block).
- [Refactor] Extracted `buildStepBarCells(percent)` (clamp + floor + 20-cell
  remainder, glyphs as ASCII escapes); scoped re-run stayed green:
  `184 passed / 0 failed / 0 ignored / 3 suites`.
- [Gates] theme_token_check --check-added --base main: silent (tool prints
  only violations/ERROR; silent run == zero added hex, exit 0 per source).
  hygiene_check: R3 warn-only lines only, all pre-existing and none in the
  touched files; exit 0 per source contract (exit 1 only on R1/R2).
- [Committed] Gate 5 encoding_guard --staged: silent (violation-only tool;
  silent == exit 0). Gate 6 commit_scoped: `e9cf10d`, 6 files, 202+/4-,
  exactly the declared scopes. Red needle, Green line and gate evidence are
  in this journal; `ui_smoke` (G3) and `--baseline` (G7) stay with the
  orchestrator closeout per the sizing law.
- [Not done deliberately] No changes to crates/ui/src, server_api.rs,
  themes.css, theme.js, fixtures, or the ledger (frozen / out of touch-set).
  Deviation: theme_token_check/hygiene/encoding exit codes confirmed via
  their silent-on-pass source contracts because compound echo wrappers were
  refused by the bash permission matcher; the gate commands themselves ran
  verbatim and unrefused.

## Hop 2 - verdict remediation (verdict: tasks/ui-progress-parity-t3-verdict.md)

- [Finding 3 format correction] Testrun gates in this hop 1 ran through
  `--distill` with capture `C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt`
  (path printed by the digest; omitted from hop-1 lines - corrected here).
  Gates 3/5 exit-0 lines, stated as exit lines:
  `theme_token_check --check-added --base main` -> exit 0 (silent; tool
  prints violations only, `return 1 if violations else 0`, lines 211-213).
  `encoding_guard.py --staged` -> exit 0 (silent; identical shape, lines
  171-173). Reviewer verified both claims TRUE independently (verdict
  finding 3); no result changed, format only.
- [Rework Red] Extended `test_ui_loading_overlay_and_progress` with the
  legacy-trio needles; predicted first failure hit:
  `183 passed / 1 failed / 0 ignored / 2 suites`,
  `assertion failed: html.contains("function setLegacyFallbackVisible")` at
  static_html.rs:561. Capture: `target/testrun-capture.txt`.
- [Rework Green] main.js: added `setLegacyFallbackVisible(visible)` hiding
  the whole bar-row/loadingText/loadingDetail trio; rows path calls
  `(false)`; empty-steps legacy branch and `hideProgress` (inside the fade
  setTimeout) call `(true)`; rewrote the buildStepBarCells doc comment
  pure-ASCII (raw U+2588/U+2591 removed - verdict finding 2; glyph scan of
  crates/ui/assets now finds zero block-glyph bytes). Gate:
  `184 passed / 0 failed / 0 ignored / 3 suites`, exit 0. Capture:
  `target/testrun-capture.txt`.
- [Rework gates] `theme_token_check --check-added --base main`: exit 0,
  silent (zero added hex). `hygiene_check`: exit 0 (R3 warn-only, all
  pre-existing, none in touched files). `encoding_guard --staged`: exit 0,
  silent (before committing, below).
- [G3/bench attempt - REFUSED in this session too, verbatim] Requested by
  the orchestrator this hop; `python tools/ui_smoke.py` and
  `python tools/bench_compare.py` both refused with:
  `The user has specified a rule which prevents you from using this
  specific tool call. Here are some of the relevant rules
  {"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  Not fabricated, not run; G3 stays open for a fresh/orchestrator session
  per the sizing-law carve-out (verdict findings 8 + scoreboard G3).
- [Rework committed] `9d96272` via commit_scoped with only the three
  touched scopes (main.js, static_html.rs, journal; index.html/api.js/
  styles.css untouched this hop, omitted per verdict). Follow-up docs-only
  commit `fe29dfc` fixes the T2 journal "124 lines" -> verified 115
  (verdict remediation step 5 carry-over).

## Hop 3 - verdict N1/N2 (boot-failure invisibility), Variant A

- [Context] Hop-2 verdict REJECTED on new blocker N1: the frozen error
  branch (api.js:92) writes 'Error: ...' into `loadingDetail` while the
  row path keeps it hidden - invisible error text. Variant A (preferred,
  single restore site) applied; the frozen api.js branches stay untouched.
- [N1 Red] New source-lock needle `update_body.contains(
  "setLegacyFallbackVisible(true)")` failed as predicted:
  `183 passed / 1 failed / 0 ignored / 2 suites`, capture
  `C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt`.
- [N1 Green] `updateProgress` now opens with `setLegacyFallbackVisible(true)`
  (restores the trio before writing bar width + detail text; covers the
  error branch N1 and the ready-phase 90%/100% calls N2 at one site):
  `184 passed / 0 failed / 0 ignored / 3 suites`, exit 0, capture as above.
- [N1 gates] `theme_token_check --check-added --base main`: exit 0, silent
  (zero added hex). `encoding_guard.py --staged`: exit 0, silent.
- [R2 self-inflicted, fixed] My hop-2+hop-3 needles pushed
  `static_html.rs` to 1008 lines (> 1000, R2 blocking). Compacted my own
  hop-2/hop-3 blocks into a shared `body_slice` helper (every needle
  semantics preserved verbatim, no weakening) -> 997 lines; scoped gate
  re-green `184 passed / 0 failed / 0 ignored / 3 suites`; `hygiene_check`
  exit 0 with only the pre-existing R3 warn rows (none in touched files).
