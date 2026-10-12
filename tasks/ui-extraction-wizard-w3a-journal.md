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
