# W-3a Verdict - ui-extraction-wizard: asset registration + stepper shell + steps 0-1

Verdict: APPROVED (with 2 minor, non-blocking findings).

Handoff adjudicated as AMENDED: `tasks/ui-extraction-wizard-w3a-handoff.md`
(planner amendment `bf5a1db`, 3-row amendment table). Commits under review:
`49fc843` (landing feat, parent `bf5a1db`) and `c1956e5` (journal rider,
docs-only). Reviewer: fresh full verification, per-commit oldest-first,
independent plan-literal and deviation adjudication. Write-git is denied, so
no finding was fixed here; all are reported.

## 1. Ground truth and commit-vs-touch-set

- HEAD `c1956e5` on `main`; tree clean, no untracked (`checkpoint_guard status`).
- `git diff --name-only "bf5a1db..c1956e5"` = EXACTLY the 13 amended
  touch-set paths: crates/ui/assets/{extraction-wizard.js, hotspots.js,
  index.html, styles.css}; crates/ui/src/{export.rs, lib.rs, routes/api_payloads.rs,
  server.rs}; crates/ui/tests/ui_test/{extraction_wizard_render.rs, main.rs,
  theme_contract.rs}; tools/ui_smoke.py; tasks/ui-extraction-wizard-w3a-journal.md.
  No path outside the touch-set. PASS.
- Frozen-lock byte-identity proven (empty diffs `bf5a1db..c1956e5`):
  - `crates/ui/tests`: only the 3 allowed files changed
    (`extraction_wizard_render.rs` NEW, `main.rs` +1, `theme_contract.rs`
    amendment). `static_html.rs`, `export_order.rs`, `extraction_wizard_api.rs`,
    `extraction_contrast.rs`, and the card-family test files are UNEDITED.
  - Frozen assets byte-identical: preview.js, api.js, graph.js, traversal.js,
    main.js, theme.js, themes.css.
  - `tools/theme_token_baseline.json` and `tests/**` byte-identical.
  - Load-pipeline producer set byte-identical vs `bf5a1db` (crates/cli,
    scheduler, index, pipeline, graph, report, repo, transform, build, validate,
    worker-core) - empty diff, the bench-exemption proof.

## 2. Amendment table executed exactly (no more, no less)

| Row | Plan expectation | Landed hunk | Verdict |
|---|---|---|---|
| 1 | export.rs tuple `("extraction-wizard.js", EXTRACTION_WIZARD_JS),` immediately before `("theme.js", THEME_JS),`; import gains `EXTRACTION_WIZARD_JS` between `API_JS` and `FILE_VIEW_JS` | `+ ("extraction-wizard.js", EXTRACTION_WIZARD_JS),` between main.js and theme.js; import `API_JS, EXTRACTION_WIZARD_JS, FILE_VIEW_JS, ...` | EXACT |
| 2 | theme_contract.rs count `9 -> 10`, message `"exactly nine classic asset scripts" -> "exactly ten classic asset scripts"` | `- 9 "exactly nine..."` / `+ 10 "exactly ten..."` | EXACT |
| 3 | theme_contract.rs whitelist `"/app/extraction-wizard.js",` immediately before `"/app/theme.js",` | inserted after `/app/main.js`, before `/app/theme.js` | EXACT |

Script-tag position (lock `main < wizard < theme`): `<script
src="/app/extraction-wizard.js">` sits immediately before
`<script src="/app/theme.js">` and after `/app/main.js` (index.html +35 hunk).
`main.rs` mod line inserted immediately after `mod extraction_wizard_api;`
and before `mod file_view;` - per plan branch.

## 3. Five named Red locks and nothing else (D3-D5 hold under my reading)

The 9 render tests decompose into exactly 4 green-at-Red (registration tests
1,2,9 + asset-discipline test 8) and 5 red-at-Red (tests 3,4,5,6,7). The 5
panics equal the handoff's named set verbatim:
`index.html must contain the wizard modal shell`; `extraction-wizard.js must
declare the wizard state reset`; `step Per-file manifest must be declared in
the wizard step list order` (D4: the asset header mentions Overview and
"Files to create", so Per-file manifest is the true first missing id); `the
proposal card must wire the Open-wizard handler`; `the S1 step must read the
payload truncation flag for caps honesty`. No theme_contract or static_html
failure at Red (the two amendment locks rode the registration chain). The
single squashed landing commit cannot be re-run at Red, but the Green code
structure is consistent with exactly these five failing on the protocol's
pre-Green state. The docstring `2 suites` at Red / `3 suites` at Green is the
documented cargo behavior when a test target fails (doc-tests skipped).

## 4. Asset hygiene and locks

- Wizard CSS: one contiguous, fully `.wizard-*`-scoped block; every color a
  theme token (`var(--...)`); zero hex literals in styles.css (grep confirms).
- Required test 8 enforced: `test_wizard_asset_is_ascii_and_escaped` is green;
  the asset reuses the escapers hotspots.js ships (hotspots.js:225
  `function escapeHtml`, :234 `function escapeAttribute`) and re-declares
  neither; no raw `${proposal.`/`${fileRow.` interpolation.
- Served contract re-proven: render test
  `test_wizard_route_serves_the_registered_const` asserts
  content-type `text/javascript; charset=utf-8` and `Cache-Control: no-store`
  (green), and distiller-routed `ui_smoke` passes with the
  `/app/extraction-wizard.js` row (`True` = expects no-store).
- Pure-ASCII on newly authored lines: `encoding_guard --all` silent; every
  non-ASCII byte in the touched files sits on pre-existing lines (house idiom
  glyphs / doc-comment dashes); the new asset and new test carry no non-ASCII;
  index.html uses `&#10005;`, the asset uses `&rarr;`.
- W9 (boot payload) and W10 (order/preview locks) untouched and green; state.rs
  not edited.

## 5. Fresh acceptance gates (this session)

- Scoped Green: `python tools/testrun.py --distill -p ariadne-ui` ->
  `303 passed / 0 failed / 0 ignored / 3 suites` (baseline 294 + 9).
- `python tools/theme_token_check.py --check-added --base bf5a1db` -> silent.
- `python tools/hygiene_check.py` -> `### generic PASS (exit 0)`; all output
  pre-existing R3 warn-only, none in code from this hand.
- `python tools/encoding_guard.py --all` -> silent (exit 0).
- Scoped ui_smoke tools suite -> `Ran 29 tests in 4.704s` / `OK`.
- Runtime proof: `python tools/distill.py -- python tools/ui_smoke.py` ->
  `boot+route checks clean (cold fixture, attempt 1, 0.4s)` / `ui_smoke: PASS`.
- Workspace census: `python tools/testrun.py --distill` ->
  `1474 passed / 0 failed / 13 ignored / 112 suites`.
- Full CI tools discover: `python -m unittest discover -s tools -p
  "*_tests.py"` -> `Ran 219 tests in 22.113s` / `OK`.

## 6. Bench exemption (documented, proven)

`python tools/distill.py -- python tools/bench_compare.py` -> `### generic
FAIL (exit 2)`, a single missing row: `generate_report+detect_cycles
(combined)`. This is the documented M23.5 label rename: the producer
`crates/cli/tests/payload_tail_bench.rs:389` emits
`generate_report+detect_cycle_groups (combined)` (comment :62-65) while
`bench_compare.py` DOCUMENTED_PHASES still names the old `detect_cycles`.
Exemption recorded WITH proof: the entire producer set is byte-identical to
`bf5a1db` (empty `git diff` over crates/cli + every load-pipeline crate), and
`tools/bench_compare.py` is not in this hand's diff. No timing regression is
attributable to W-3a; the parser correctly refuses to substitute zeros.

## 7. Deviation dispositions

Halt-session set (pre-Red, journal `e22f4a6`):
- D1 halt-before-edit: ACCEPTED (correct given the under-counted chain).
- D2 journal-only commit, no gate-9 landing: ACCEPTED (no code to land).
- D3 tree-clean via checkpoint_guard, raw git denied: ACCEPTED.
- D4 gates 1,5 at HEAD as halt documentation: ACCEPTED (labeled non-hand).
- D5 exit-code via distill.py (echo/redirect denied here): ACCEPTED.
- D6 Duty-1 pre-stage then encoding --staged on the exact set: ACCEPTED.

Resume-session set (landing hand):
- D1 raw git denied, reads via distill.py, tree from checkpoint_guard: ACCEPTED.
- D2 amendment staging confirmed from the commit's own file list: ACCEPTED.
- D3 required test 5 folded into the modal-shell lock: ACCEPTED. Verified
  `test_wizard_modal_shell_declares_the_stepper` asserts aria-current
  (:138-140) and Back/Next `<button>`s (:142-150); the lock carries test 5's
  required content so Red's failing set is exactly the five named panics.
- D4 Red step-order names Per-file manifest: ACCEPTED (see section 3).
- D5 test 8 green at Red by construction: ACCEPTED (not one of the five named).
- D6 Esc-close via a load-time wrap of the frozen global closeModals: ACCEPTED.
  It extends the existing close path without editing api.js/main.js and matches
  the asset (lines 165-176).
- D7 gate-5 exit code via distill.py: ACCEPTED.
- D8 server.rs use-block re-wrap: ACCEPTED. Verified formatting-only and
  rustfmt-clean at the import block (fmt drift is at line 77, the pre-existing
  W-2a extraction route wraps, not the import).
- D9 journal checkpoints as disk state, landed inside gate-9's single commit:
  ACCEPTED.
- D10 styles.css wizard block ~110 net lines (over the ~45 sketch): ACCEPTED
  (contiguous, wizard-scoped, zero hex, `.css` invisible to R2).
- D11 styles.css `.step-bar-remainder` whitespace erratum: ACCEPTED (cosmetic;
  no lock reads that rule body - only the token-name list at
  theme_dark_grey_palette.rs:50 and the JS-builder class-name check at
  static_html.rs:561).
- D11 (numbered) journal-only rider commit after landing: ACCEPTED (house
  precedent `90f4298`; post-commit facts cannot self-contain in the landing
  commit; docs-only).

## 8. W-3b forward note (from the W-2a verdict F3) - adjudicated

The degraded `honest_absent_dry_run` answers `copy_only: false` alongside
`move_removals: 0`; the faithful value is computed at
`extraction_reader.rs:189` (`copy_only: move_removals == 0`) and forced false
only in degraded mode (`:424`). The W-3a step-1 (S1 `renderWizardStepFiles`)
shell does NOT read or render `copy_only` or `includes_unavailable` anywhere
(grep confirms these live only in dto/extraction.rs and the reader). It reads
the genuine W-3a boot cap flag `target_files_truncated` (the only honest cap
signal available pre-provider) and computes the per-file overflow
`hiddenSymbols` locally (the compute-locally pattern). The C7 state object
carries a reserved `dryRun` slot for W-3b. Therefore W-3a does not bake in the
degraded-`copy_only:false` trap and does not preempt W-3b: the copy-only
sentence must be keyed off the computed `move_removals` path with
`includes_unavailable` as the degradation signal - which is W-3b's obligation,
already named in its own handoff (W-3b: "when `includes_unavailable` is true
the renderer prints the honest fallback"; "the dry-run panel restates it when
`copy_only` is true"). No W-3a change required; relay to W-3b verbatim.

## 9. Numbered findings

1. (minor, non-blocking) `crates/ui/src/routes/api_payloads.rs`: the new
   `asset_extraction_wizard_js` body
   `ui_asset(crate::EXTRACTION_WIZARD_JS, "text/javascript; charset=utf-8")`
   is one line rustfmt wants wrapped (longer const exceeds `fn_call_width`).
   Compiles green; fmt is not a gate and running `cargo fmt` would blow the
   touch-set. Informational only.
2. (minor, non-blocking) `crates/ui/tests/ui_test/extraction_wizard_render.rs:173`
   the `["planId:", "step:", ...]` array and the trailing blank line near
   `theme_contract.rs:448` are cosmetic rustfmt preferences, consistent with
   the crate's ambient non-rustfmt-clean baseline. Informational only.

Both minors are documented in D8/D10 and do not affect any gate, lock, or the
touch-set. No blocker. Neither requires a fix before the W-3b hop; the next
implementer may adopt rustfmt wrapping on any line it re-touches.

## 10. Disposition

APPROVED. All 5 required registration sites plus the 2 amended lock sites
landed exactly per the amendment table; the 13-path touch-set is exact and all
frozen surfaces are byte-identical; every declared deviation holds; the
forward W-3b note is cleanly handed off. Two cosmetic rustfmt nits recorded
as non-blocking. The bench exit-2 is the pre-existing, documented
DOCUMENTED_PHASES label rename, exempt with the zero-diff producer proof.
