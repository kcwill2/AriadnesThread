# Plan: Extraction Card Story (UI readability)

Milestone key: `ui-extraction-card-story`. Authoritative artifacts (this plan
does not restate them):

- `tasks/ui-extraction-card-story-kickoff.md` - intent, scope, invariants,
  frozen locks, acceptance gate.
- `tasks/ui-extraction-card-story-shape.md` - story structure, invariants
  S1-S12 with named oracles, contracts C1-C6 (markup skeleton, keep-literal
  list, token map, honest copy), module map, risks, hygiene.

## Intent (one paragraph)

Make the Proposed Extractions card tell its story - why -> what moves ->
what it costs -> who is affected -> preview - by rendering `problem` and
`plan_id` visibly, turning the collateral delta into a labeled before/after
with a computed saving, rendering symbol names and required headers as
chips, and giving the risk tier and consumer-TU count real semantics, all
in token-only styling across the three themes. DTO and producers stay
frozen; this is display-layer work in `crates/ui` assets + ui tests.

## Shape in one screen

1. `renderExtractionProposalCard` (hotspots.js) rewritten to the five-beat
   card of shape contract C3; helpers `extractionChips`,
   `extractionSavingText`, `renderExtractionRiskTag` extracted so every
   function stays small. Section cache logic, click-to-select, and preview
   button markup unchanged.
2. `styles.css`: `.extraction-*` block restyle + new classes, token-only
   (C5), budget <= ~90 added lines, shared `.just-*` rules untouched.
3. `index.html`: Hotspots help gains <= 3 short `<li>` defining collateral,
   risk tiers, and the re-validation count honestly (C6).
4. New `crates/ui/tests/ui_test/extraction_card_story.rs` (+ `mod` line in
   `main.rs`) locks S1-S6/S10; every existing card/section/help test stays
   green UNEDITED thanks to the keep-literal list (C4).

## Handoff sizing

One handoff (assets + tests, one crate, well under 10 gates and ~1200
produced lines). Gate commands: `python tools/testrun.py --distill -p
ariadne-ui`; `python tools/theme_token_check.py --check-added --base <rev>`;
`python tools/hygiene_check.py`; `python tools/encoding_guard.py --staged`;
`python tools/ui_smoke.py`.

## Hard no-touch set

`api.js`, `preview.js`, `graph.js`, `main.js`, `themes.css`, `theme.js`,
`crates/ui/src/**`, `crates/cli/**`, `crates/cluster/**`, `tools/**`,
`tests/fixtures/**`, `tools/theme_token_baseline.json`.

## Out-of-scope decisions kept visible

- `styles.css` exceeds 1000 lines with no gate (R2 skips `.css`) - the
  existing deferred ledger row owns that call, not this milestone.
- Rewording the templated `problem` string at the planner producer was
  rejected: it would re-gate parity tests for zero readability gain.

## Execution results (closeout addendum, 2026-10-04)

Shipped: T1 (single handoff, sizing verdict held) at landing commit
`ef9d170` on `main` (parent = journal start rev `4900e04`; dark-grey T1
`41ec354` landed first exactly as the serialization plan required). The card
now tells the five beats: visible `problem` line + `plan_id` chip, labeled
delta with zero-guarded `-D (-P%)` saving, symbol/header chips with the kept
`+N more`, sanitized tiered risk badge, `N consumer TU(s) need re-validation`,
token-only `.extraction-*` restyle (~+75 CSS lines in-block, budget ~90),
3 new help `<li>`, 9 new locks in `extraction_card_story.rs`; ZERO
pre-existing test edits (C4 keep-literals kept the whole suite green).

Gates: verdict APPROVED - `tasks/ui-extraction-card-story-t1-verdict.md`;
workspace-gate evidence `tasks/ui-extraction-card-story-execution-results.md`.
Canonical lines: scoped `197 passed / 0 failed / 0 ignored / 3 suites`;
census `1181 passed / 0 failed / 4 ignored / 85 suites`; tools discover
`Ran 188 tests in 20.991s` + `OK`; `ui_smoke: PASS` (via distill wrapper);
zero added hex (`--base 4900e04`); hygiene + encoding exit 0;
`bench_compare` sanctioned assets-only exemption (empty diff over all
producer crates is the machine proof).

Deviations D1-D4, all accepted (dispositions in the verdict): D1 shape C5
delta `--text` emphasis unsatisfiable vs the frozen declared-token oracle -
dropped emphasis, stricter oracle wins; D2 the `ui-dark-grey-theme` ledger
flip stays that milestone's reviewer-hop duty; D3 start rev `4900e04` (not
planning-time `d76a559`) per the first-landing-head rule; D4 Red suite-count
cosmetic (cargo fail-fast skips the 0-test doc-test suite).

Findings split: fixed-now = none in code; routed to deferred ledger rows =
reviewer allowlist gap for direct `ui_smoke`/`bench_compare` invocation, and
the stale `target\debug\tool.exe` process lock that forced the census through
a temp `CARGO_TARGET_DIR`. Human eyeball of card height inside the 240px
scroller remains open (no height-locked test, as stated in the shape risks).
Walkthrough deck pending (walkthrough-commit hop).
