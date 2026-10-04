# T1 Journal - ui-dark-grey-theme: register the fourth theme `dark-grey`

Companion to `tasks/ui-dark-grey-theme-t1-handoff.md`. Decision log for this
implementer session: checkpoints per gate, deviations with reasons, lock and
contract findings. This file is the recovery artifact if the session dies.

## Session start - precondition verification (before any mutation)

- `python tools/checkpoint_guard.py status`: HEAD `d76a559` on `main`, dirty
  only in foreign files (`.kilo/plans/...systems-layer-engine-room.md`,
  `tasks/milestone_ledger.md` +1) which stay unstaged and untouched.
- T3 precondition MET: HEAD `d76a559` is the ui-progress-parity closeout
  commit (ledger row CLOSED 2026-10-04); T3 commits `e9cf10d`, `9d96272`,
  `17e2299` are ancestors of HEAD; the T3 step-row markup
  `<div id="loadingSteps"></div>` is live in index.html; T3's
  `crates/ui/tests/ui_test/static_html.rs` is present.
- Handoff deviation (report-only, verified, not a stop): the handoff's example
  verifier "crates/ui/src/static_html.rs present" does not exist - the repo
  has no `crates/ui/src/static_html.rs` (shape says the same: `lib.rs`,
  `server.rs`, `export.rs`, `dto.rs` only). The actual T3 surface is
  `crates/ui/tests/ui_test/static_html.rs` (present) plus main.js/index.html
  markup (present). The handoff's own alternative check ("T3 step-row markup
  in index.html") passes, so the tree is a post-T3 tree and work proceeds.
- All content anchors quoted in the handoff verified verbatim:
  themes.css light close `}` -> steampunk comment insertion point (lines
  298-311), default block (lines 40-166), theme.js THEMES literal line 15,
  bootstrap comment + validator line + `#themeSelect` options + Help
  "Theme:" paragraph in index.html, theme_contract.rs export assertion
  (lines 104-107) and helper idiom (lines 178-243), main.rs mod site,
  theme_token_check.py TOKEN_FILES (37-40) + frozen-count scan (136-147).
- Plan: RED (new oracle file + mod line + export assertion extension ->
  expect compile-clean with exactly 5 named failures), GREEN (CSS block,
  theme.js, index.html one pass, baseline regen), hygiene gates, one scoped
  commit carrying block + baseline + justification sentence.

## Grey palette design decision (authored before Green, drives the oracles)

- Chrome ramp: pure achromatic zinc direction, every hex r=g=b, every
  authored rgba r=g=b. bg ladder #141414 -> #171717 -> #1c1c1c -> #262626 ->
  #2e2e2e -> #3a3a3a; borders #3f3f3f/#525252; ink #f5f5f5 main, #a3a3a3
  muted, #8a8a8a sub, #e4e4e4 code. Hand-computed WCAG margins: text-main
  ~12.5:1 on the worst (lightest) tested surface, muted ~7.3:1 on bg-main,
  sub ~5.3:1 on bg-main - all floors (7.0/4.5/3.0) hold with headroom; the
  I8 test is the arbiter in Green.
- Translucent chrome neutralized to the solid ladder equivalents
  (panel 28,28,28 = bg-surface; tile 46,46,46 = bg-card; track 163,163,163 =
  text-muted). Shadows/scrims/grid-dot keep the default black/white alphas
  (already achromatic).
- Everything except the 26 I7 chrome tokens is copied VERBATIM from the
  default block (semantic layer byte-equal per I6; sky-cyan decorative
  family, node-kind identities, and the proposed family per the recorded
  user decisions).

## RED checkpoint (gate 1)

- Command: `python tools/testrun.py --distill -p ariadne-ui`
- Digest: `### cargo test FAIL (exit 1) [capture: target/testrun-capture.txt,
  239 lines]`; canonical line `183 passed / 5 failed / 0 ignored / 2 suites`.
- Exactly the 5 predicted failures, all assertion/panic (compile-clean):
  the 4 `theme_dark_grey_palette::*` tests panic
  `themes.css must contain a block for [data-theme="dark-grey"]`;
  `theme_contract::test_standalone_export_inlines_theme_tokens` fails on the
  extended "all four theme blocks" assertion. No collateral failures -
  the auto-detecting oracles (select/bootstrap/css parity, full-token-set)
  stay green with the surfaces unchanged, as designed.

## GREEN checkpoint (gates 2 + 3)

- One pass applied: themes.css block (inserted at the verified anchor: after
  light's closing `}`, before the steampunk header comment; 106 tokens in
  default order; semantic/decorative/node/proposed values copied verbatim),
  theme.js THEMES literal, index.html bootstrap comment + validator line
  (kept ONE line, `&&` chain intact) + `<option value="dark-grey">` + Help
  paragraph. Zero rewording of the em-dash region: the comment edits matched
  ASCII-only substrings ("below; the three", "adding a fourth"), so the
  pre-existing `—` bytes were never re-authored.
- Baseline regen (gate 3, BEFORE the green testrun so gate 2 ran "after all
  surfaces + baseline"): `baseline written: 11 files, 286 literals,
  286 locations`. themes.css count 211 -> 281 as predicted (+70 hexes).
- Baseline observation (report-only, no hand deviation): the regeneration
  also rewrites `generated_from_head` (f9064f4 -> d76a559) and shifts the
  `locations` LINE-NUMBER keys for index.html (209->220 etc). Both are
  inherent to the prescribed `--update-baseline` tool: the committed baseline
  was stamped pre-T3, when its `locations` froze pre-T3 line numbers; the
  frozen-count scan (the actual gate) uses only `counts`, and no count except
  themes.css changed. index.html literals and their count (4) are unchanged.
- Command: `python tools/testrun.py --distill -p ariadne-ui`
- Digest: `### cargo test PASS (exit 0) [capture: target/testrun-capture.txt,
  208 lines]`; canonical line `188 passed / 0 failed / 0 ignored / 3 suites`.
- Suite census 2 -> 3 across RED/GREEN explained from the capture: three test
  targets (lib 0 tests, ui_test 188 tests, third 0 tests); the RED run's
  cargo fail-fast stopped before the trailing 0-test target printed. Total
  executed test count identical across runs (183+5 = 188).
- Refactor pass: reviewed the new oracle file; helpers already single-
  responsibility with intention-revealing names (`declared_value`,
  `hex_channel_runs`, `rgba_channel_runs`, `relative_luminance`,
  `contrast_ratio`, `first_hex`); the deliberate helper COPY from
  theme_contract.rs stays (the handoff forbids refactoring common.rs or
  theme_contract.rs beyond the export assertion). No code change after
  GREEN; scoped suite stays green.

## Landing checkpoint (gates 4-7)

- Gate 4 pre-commit: `theme_token_check.py --check-added --base d76a559`
  exit 0, no violations (themes.css TOKEN_FILES-exempt from the added scan;
  the regenerated count is what the frozen-count scan validated).
- Staging: scoped `git add -- <the eight touch-set paths>` via
  `tools/distill.py` (never `git add .`; foreign dirty files - `.kilo/plans`
  and `tasks/milestone_ledger.md` - stayed unstaged, and foreign untracked
  plan/handoff files stayed out of every scope glob).
- Gate 5: `hygiene_check.py` exit 0; all emitted rows are warn-only R3
  pre-existing sites; zero findings in the new oracle file or edits.
- Gate 6: `encoding_guard.py --staged` exit 0 before committing.
- Gate 7: single scoped commit `41ec354` carrying block + baseline +
  justification sentence; stat: index.html +5/-4, theme.js 1/1, themes.css
  +141, main.rs +1, theme_contract.rs +3/-2, new oracle file +307, handoff +
  journal recorded, baseline 147+/77-.
- Post-commit confirmation run: `188 passed / 0 failed / 0 ignored /
  3 suites`, digest `### cargo test PASS (exit 0)`.
- Deviations logged: (1) handoff's example precondition verifier names a
  nonexistent `crates/ui/src/static_html.rs`; the handoff's own alternative
  (T3 step-row markup in index.html) verified the post-T3 tree. (2) Baseline
  regeneration necessarily re-stamps `generated_from_head` and re-keys
  index.html `locations` line numbers (pre-existing pre-T3 drift); counts
  semantics preserved, gate 4 green.
