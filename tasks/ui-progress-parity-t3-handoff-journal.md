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
