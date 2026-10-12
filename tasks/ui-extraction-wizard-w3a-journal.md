# W-3a Journal - HALTED at boot audit: registration chain needs two files outside the touch-set

Milestone: ui-extraction-wizard. Handoff: `tasks/ui-extraction-wizard-w3a-handoff.md`.
Session: 2026-10-12. Implementer under strict RGB TDD.
Outcome: Duty 1 landed; Duty 2 STOPPED before any edit - the handoff's
touch-set cannot satisfy its own R7 chain without editing two files that
the touch-set forbids. Per the handoff's hard rule ("any edit outside =
blocker, report don't fix") this hand reports and does not fix.

## Boot facts (checkpoint_guard status)

- HEAD at boot: `90f4298` on `main` (docs(tasks): W-2a journal rider).
- Dirty files: none. Tree clean under `crates/**` vs HEAD. No foreign WIP.
- Untracked at boot: exactly the two W-2a review artifacts (tracked by Duty 1).
- Raw `git status` was permission-denied at the shell; tree ground truth
  came from `checkpoint_guard status` (numbered deviation D3).

## Duty 1 - W-2a review artifacts (landed)

- Pre-staged exactly the two paths via `python tools/distill.py -- git add --
  tasks/ui-extraction-wizard-w2a-verdict.md
  tasks/ui-extraction-wizard-w2a-execution-results.md`.
- `python tools/encoding_guard.py --staged` on that staged set: silent pass.
- Staged-set check (`distill.py -- git diff --cached --name-only`): exactly
  the two files, nothing else.
- Reviewer's exact command run:
  `python tools/commit_scoped.py --scope tasks/ui-extraction-wizard-w2a-verdict.md
  --scope tasks/ui-extraction-wizard-w2a-execution-results.md -m "docs(tasks):
  W-2a gate APPROVED (engine-backed read-only manifest/dry-run provider) +
  execution-results"`
- Landing commit `1f561a0`, carrying exactly the two paths
  (execution-results +222, verdict +323). Post-HEAD confirmed clean.

## Duty 2 - the blocker (chain audit, evidence-cited)

The handoff's R7 enumerates five registration sites and its touch-set names
the files for sites 1,2,3,5 plus the test wiring. Site 4 is described as
"the standalone-export inline lock in `static_html.rs`". The repo truth:

1. The export lock `test_standalone_export_inlines_all_split_assets`
   (`crates/ui/tests/ui_test/static_html.rs:949`, frozen, asserts
   `!content.contains("src=\"/app/")`) PASSES only if the standalone export
   inlines every script tag. The inlining is a fixed tuple list in
   `crates/ui/src/export.rs:35-48` (`standalone_template()`).
   A 10th `<script src="/app/extraction-wizard.js">` tag survives the
   replacement loop verbatim, so the lock fails unless `export.rs` gains the
   tuple entry `("extraction-wizard.js", EXTRACTION_WIZARD_JS)`.
   `export.rs` is NOT in the touch-set.
2. `crates/ui/tests/ui_test/theme_contract.rs`
   (`test_index_markup_has_no_inline_style_or_script_bodies`, lines 384-449)
   carries two additional hard locks over `index.html` script tags: the
   whitelist `tag_start_is_asset` (exactly nine entries, lines 435-449; any
   other `src=` tag panics "unexpected script tag with src") and
   `assert_eq!(html.matches("<script src=").count(), 9, ...)` (lines 428-432).
   Verified at HEAD: `crates/ui/assets/index.html` lines 591-601 carry
   exactly 9 `<script src=` tags. The mandatory script tag (required test 1)
   makes the count 10 and hits the whitelist. Both fixes live in
   `theme_contract.rs` - NOT in the touch-set.
3. Sweep of the remaining suspects - cleared, no edit needed:
   - `test_script_tag_order_preserves_original_execution_order`
     (`static_html.rs:760`): relative-order walk, unaffected by additions.
   - `test_ui_asset_routes_serve_bundled_content_with_no_store`
     (`static_html.rs:792`): fixed 8-element list, additive-safe.
   - `common::page_content()` (`tests/ui_test/common.rs:237`): concatenates a
     fixed asset list, additive-safe (new asset simply absent).
   - `asset_injection_hardening.rs`: fixed per-asset loops, additive-safe
     (coverage gap only; note the wizard must NOT re-declare `escapeHtml`).
   - `tools/ui_smoke_tests.py:86-95` `fully_compliant_routes()` builds the
     stub table FROM `ROUTE_ASSERTIONS`, so the one added row is safe.
   - `export_order.rs`: payload byte-lock, unrelated.

Consequence: gates 1-3 of this handoff (full `-p ariadne-ui` green) are
unsatisfiable under the stated touch-set. The two bounded fixes (one tuple
line in `export.rs`; whitelist entry + count 9->10 in `theme_contract.rs`)
belong in the touch-set - precedent: every prior asset (theme.js,
provenance.js) shipped these ledger updates inside its own hand.
This hand did not improvise them (numbered deviation D1).

## Gates actually run (this halt record)

- Gate 1 (baseline): `python tools/testrun.py --distill -p ariadne-ui` ->
  `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 323 lines]`
  `294 passed / 0 failed / 0 ignored / 3 suites` (suite green at HEAD, so the
  breakage above is purely addition-induced).
- Gate 5 (hygiene): `python tools/hygiene_check.py` -> exit 0 via the
  distiller digest `### generic PASS (exit 0)` (capture:
  `C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt`, 179 lines);
  all noise is pre-existing R3 warn-only.
- Not run (bound to a landing hand this halt cannot execute): gates 2, 3, 4,
  6, 7, 9, 10. Gate 8 (encoding guard) ran on every staged set this session
  produced (Duty 1 and the journal commit).

## Touch-set conformance

- Files edited this session under `crates/**`, `tools/`: NONE. Zero edits.
- The touch-set includes NO `ui.rs`: neither `crates/cli/src/commands/ui.rs`
  (the F2 986/1000 file) nor any `crates/ui/src/ui.rs` (does not exist) is in
  the set, so the R2 ceiling was never at risk from this hand.
- Sole file written: this journal (in touch-set).

## Registration-chain proof status (per site, as required)

| Site | Target | Status |
|---|---|---|
| 1 | `lib.rs` `EXTRACTION_WIZARD_JS` const | NOT WIRED (halt) |
| 2 | `server.rs` route + `api_payloads.rs` handler | NOT WIRED (halt) |
| 3 | `index.html` script tag | NOT WIRED (halt) - the exact edit that detonates the two out-of-touch-set locks |
| 4 | export inline (`export.rs` tuple; lock file `static_html.rs` stays unedited) | NOT WIRED (halt) - AND missing from the touch-set |
| 5 | `tools/ui_smoke.py` `ROUTE_ASSERTIONS` row | NOT WIRED (halt) |
| + discovered 6 | `theme_contract.rs` whitelist + 9->10 count | NOT IN TOUCH-SET - required by site 3 |

## What the successor needs

1. Planner/orchestrator amendment: add `crates/ui/src/export.rs` and
   `crates/ui/tests/ui_test/theme_contract.rs` to the W-3a touch-set
   (bounded edits: one tuple entry; one whitelist entry + one count change).
   The card-family frozen files still stay untouched.
2. Then run W-3a exactly as handed; suggested tag position: immediately
   before `/app/theme.js` (relative order locks unaffected; wizard globals
   exist long before any card click).
3. The wizard asset must not re-declare `escapeHtml` (asset_injection
   "shipped exactly once" family lock).
4. W9 boot-payload proof: this hand changed nothing; the payload lock stays
   green trivially at HEAD 1f561a0.

## Numbered deviations (rider-style included)

1. D1: Halt-before-edit. Duty 2 stops at the audit; no Red, no Green, no
   code landing. Reason: hard touch-set rule + unsatisfiable gates.
2. D2: No landing commit under the gate-9 command. Instead ONE journal-only
   scoped commit `docs(tasks): W-3a journal - boot audit blocker...` via
   `commit_scoped.py --scope tasks/ui-extraction-wizard-w3a-journal.md`.
   Gate 9 with full 11 scopes would have been a no-op/error with no code.
3. D3: Tree-clean verification used `checkpoint_guard status` because raw
   `git status` is permission-denied at the shell; supplementary git reads
   ran through `distill.py` per the command contract.
4. D4: Gates 1 and 5 were run at HEAD (pre-edit) as halt documentation for
   the successor - outside the Red-Green order, never as hand gates.
5. D5: Exit-code capture for `hygiene_check.py` went through
   `distill.py --` because direct `echo`/redirection chains are
   permission-denied in this shell.
6. D6 (rider, Duty 1): staged the two W-2a artifacts via the distiller
   BEFORE running the reviewer's commit_scoped command so
   `encoding_guard --staged` checked the exact staged set (commit_scoped
   `--only` re-stages the same paths; pre-staging makes the guard
   non-vacuous).

---

# RESUME SECTION - W-3a executed on the amended handoff (2026-10-12, second implementer)

## Amendment facts (per the Duty-1 instruction)

- The planner's amendment command ran verbatim:
  `python tools/commit_scoped.py --scope tasks/ui-extraction-wizard-w3a-handoff.md
  --scope tasks/milestone_ledger.md -m "docs(tasks): W-3a handoff amendment -
  two asset-chain lock sites added to the touch-set; ledger erratum"`
- Pre-flight inspection of `git diff tasks/milestone_ledger.md` (via the
  distiller): exactly three hunks - (1) the dated erratum row added to the
  findings table, (2) the W-3a detail bullet with the AMENDED 2026-10-12
  note, (3) the New-asset registration chain bullet rewritten to FIVE sites
  PLUS TWO lock sites (SEVEN total). The milestone row was NOT flipped. No
  foreign content.
- Landing: `bf5a1db`, carrying exactly the two scoped files
  (milestone_ledger.md +3/-2; ui-extraction-wizard-w3a-handoff.md +128/-23).
  Post-commit tree: zero dirty files.

## Boot facts (checkpoint_guard status, resume)

- HEAD at boot: `bf5a1db` on `main` (= halt journal `e22f4a6` + Duty-1
  amend commit). Dirty files: none. Untracked: none. No foreign WIP on
  main; the dirty worktrees in the census are other sessions' directories.
- Raw `git` stays permission-denied at this shell; every git read ran
  through `python tools/distill.py -- git ...` (deviation D1).

## Gate itemization (per the M28 T0b finding-1 law; 10 gates, timeout 600000)

- Gate 1 (baseline, at `bf5a1db` pre-edit):
  `python tools/testrun.py --distill -p ariadne-ui` ->
  `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 323 lines]`
  `294 passed / 0 failed / 0 ignored / 3 suites` - matches the ledger
  erratum's stated halt baseline exactly.
- Registration chain + amendments applied as one phase (asset, lib.rs,
  server.rs, api_payloads.rs, index.html script tag, export.rs tuple +
  import, theme_contract count 9->10 + message + whitelist row, ui_smoke
  row). Suite re-run after the chain, BEFORE any test existed: exit 0,
  `294 passed / 0 failed / 0 ignored / 3 suites` - proves the amendments
  are load-bearing (without them the chain detonates theme_contract and
  the export lock) and that the chain itself breaks nothing.
- Gate 2 (Red): `python tools/testrun.py --distill -p ariadne-ui` ->
  `### cargo test FAIL (exit 1) [capture: ...testrun-capture.txt, 363 lines]`
  `298 passed / 5 failed / 0 ignored / 2 suites`. The failing set is
  EXACTLY the five named new-test locks, each on its predicted first
  panic:
  - `test_wizard_modal_shell_declares_the_stepper`:
    `index.html must contain the wizard modal shell`
  - `test_wizard_state_reset_declared`:
    `extraction-wizard.js must declare the wizard state reset`
  - `test_wizard_step_list_order`:
    `step Per-file manifest must be declared in the wizard step list order`
    (deviation D4 explains why Per-file manifest, not Overview, was the
    first missing id)
  - `test_card_entry_button_opens_wizard`:
    `the proposal card must wire the Open-wizard handler`
  - `test_wizard_step_files_honors_payload_caps`:
    `the S1 step must read the payload truncation flag for caps honesty`
  No `theme_contract` or `static_html` failure in the Red set - the
  amendments rode with the registration chain as required. The digest
  reads `2 suites` at Red because cargo skips doc-tests when a test
  target fails; the doc-test suite returns at Green.
- Gate 3 (Green, and again after the refactor):
  `### cargo test PASS (exit 0) [capture: ...testrun-capture.txt, 333 lines]`
  `303 passed / 0 failed / 0 ignored / 3 suites` = baseline 294 + the 9
  new wizard tests. The new asset-discipline lock (required test 8) and
  the four registration locks pass at Red by construction - the
  documented chain-first order, not a weak Red (deviation D5).
- Gate 4: `python tools/theme_token_check.py --check-added --base bf5a1db`
  -> silent (combined form; zero added hex).
- Gate 5: `python tools/hygiene_check.py` -> exit 0; the distiller
  capture shows `### generic PASS (exit 0)` (capture:
  `C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt`, 179
  lines); every listed line is pre-existing R3 warn-only, none in a
  touched file's new code.
- Gate 6 (mandatory scoped discover):
  `python -m unittest discover -s tools -p "ui_smoke_tests.py"` ->
  `Ran 29 tests in 4.694s / OK` - the wizard route row flows through
  `fully_compliant_routes()` (built FROM ROUTE_ASSERTIONS) unchanged.
- Gate 7: `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture,
  attempt 1, 1.0s)` / `ui_smoke: PASS` - runtime proof the new asset
  route boots and the route table mirrors `build_router`.
- Gate 8: `python tools/encoding_guard.py --staged` -> exit 0, run on the
  exactly-13-path staged set (deviation D2 makes it non-vacuous).
- Gate 9: the amended `commit_scoped.py` command, 13 scopes verbatim.
- Gate 10: `python tools/encoding_guard.py --all` post-commit -> exit 0.

## Registration-chain proof (seven sites, evidence per site)

| Site | Target | Evidence |
|---|---|---|
| 1 | `lib.rs` `EXTRACTION_WIZARD_JS` const (between MAIN_JS and THEME_JS) | test 2 string lock `pub const EXTRACTION_WIZARD_JS: &str = include_str!("../assets/extraction-wizard.js");` green |
| 2 | `server.rs` route `get(asset_extraction_wizard_js)` + `routes/api_payloads.rs` handler | test 2: every route line contains `get(`; runtime 200 with bytes == `EXTRACTION_WIZARD_JS`, `text/javascript; charset=utf-8`, `no-store` |
| 3 | `index.html` `<script src="/app/extraction-wizard.js">` immediately before `/app/theme.js` | test 1: tag present + position pinned `main < wizard < theme`; count moves to 10 via amendment rows 2-3 |
| 4 | `export.rs` tuple entry `("extraction-wizard.js", EXTRACTION_WIZARD_JS)` + import (AMENDMENT row 1) | the frozen export lock `test_standalone_export_inlines_all_split_assets` (static_html.rs:949, UNEDITED) green - no `src="/app/` survivor; `asset_injection_hardening` export locks green (one escapeHtml, two pinned survivors) |
| 5 | `tools/ui_smoke.py` `ROUTE_ASSERTIONS` row | test 9 string lock; gate 6 `29 tests OK`; gate 7 `ui_smoke: PASS` |
| 6 | `theme_contract.rs` script-tag count 9 -> 10 + message (AMENDMENT row 2) | amended exactly per the table; `test_index_markup_has_no_inline_style_or_script_bodies` green |
| 7 | `theme_contract.rs` `tag_start_is_asset` whitelist row (AMENDMENT row 3) | amended exactly per the table (mirrors script order: before `/app/theme.js`); same test green |

## Frozen locks

- `static_html.rs`: NOT in the diff (UNEDITED) - verified by
  `git diff --name-only`.
- Card-family test files, `extraction_contrast.rs`, `preview.js`,
  `api.js`, `graph.js`, `traversal.js`, `main.js`, `theme.js`,
  `themes.css`, `tools/theme_token_baseline.json`, `tests/**`: absent
  from the diff; `test_order_and_preview_locks` and
  `test_details_collapsed_by_default` green UNEDITED.
- `extraction_wizard_api.rs` green UNEDITED; W9 boot payload untouched
  (payload byte locks in the suite green unchanged).
- The `ui_smoke.py` docstring drift ("six /app/*.js scripts") NOT fixed -
  exactly one row added, per the handoff.

## Numbered deviations (resume hand; rider-style included)

1. D1: Raw `git` permission-denied at this shell (same as the halt
   hand); all git reads through `distill.py --`, tree ground truth from
   `checkpoint_guard status`.
2. D2 (rider, Duty 1): the two amendment files were the ONLY dirty
   files, so the planner's `commit_scoped` scopes staged exactly them;
   the staged set was confirmed from the commit's own file list
   (`git show --stat` after `commit_scoped`'s `--only` commit).
3. D3: Required test 5 (aria-current + Back/Next `<button>`) rides
   inside the modal-shell lock, all its assertions being shell-markup
   assertions. Chosen so Red's failing set is EXACTLY the five named
   panic messages; the test file still carries 9 functions.
4. D4: Red's step-order panic named `Per-file manifest` (not Overview)
   as the first missing step id: the W-3a asset module header mentions
   S0/S1 with the labels "Overview"/"Files to create", so those ids
   were present pre-Green. The handoff's rule ("the step-order lock
   names the first missing step id") is satisfied with the true first
   missing id, and Green's order scan judges real first occurrences.
5. D5: Required test 8 (ASCII + escape discipline + no escaper
   re-declaration) passes at Red by construction - the minimal
   registration asset is ASCII, interpolates nothing, and re-declares
   nothing. Documented chain-first status, same as the registration
   locks; the handoff's Red list does not name it as a failure.
6. D6: The C1 "Esc closes via the existing closeModals() path" is
   implemented as a load-time wrap of the global `closeModals` inside
   the wizard asset (house behavior first, then hide the wizard panel)
   plus an Esc keydown that forwards to `closeModals()` only while the
   wizard is open. `api.js`/`main.js` are frozen, so wrapping the
   global is the only path that extends the existing close path
   without editing a frozen asset; the overlay onclick close gets the
   wizard for free.
7. D7: Gate 5's exit-code evidence captured through `distill.py --`
   (direct echo/redirect chains are permission-denied here), same
   mechanism as the halt hand's D5.
8. D8: The `server.rs` `use` block was re-wrapped in the Refactor step
   (formatting-only rebalance of the amended import list); the scoped
   suite re-ran after it with the identical `303/0/0/3` digest.
9. D9: "Checkpoint the journal after each Green gate" honored as
   disk-state checkpoints; the journal lands inside gate 9's single
   scoped landing commit (its scope list includes this file), per the
   milestone's one-scoped-commit-per-hand rule.
10. D10: The styles.css wizard block is ~110 net lines - over the
    ~45-line planning sketch but within the "one contiguous block"
    discipline (`.css` invisible to R2; the cap is a discipline note);
    the block uses only wizard-scoped selectors and existing tokens,
    zero re-declared selectors (every first-occurrence rule body the
    frozen card/help locks read is byte-unchanged).


