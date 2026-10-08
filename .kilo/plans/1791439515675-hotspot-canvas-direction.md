# Plan: ui-hotspot-canvas-direction

Full artifacts (authoritative):
- `tasks/hotspot-canvas-direction-kickoff.md` (intent, scope, frozen locks, milestone call)
- `tasks/hotspot-canvas-direction-shape.md` (invariants, contracts, module map, risks, acceptance)
Note: the requested `docs/specs/hotspot-canvas-direction.md` location is
outside this session's write permissions; the orchestrator can copy the two
task artifacts there verbatim.

## Problem (confirmed with user 2026-10-08)

Hotspot list ranks by metric direction; canvas keeps toolbar Direction
(default outbound). Clicking the #1 burden offender (inbound rank) shows its
own tiny outbound includes - list and canvas tell opposite stories. Repo
state note: memory of a previous "direction fix applied" is wrong - staged
main.js/styles.css changes are the boot-overlay step-bar track; no direction
code has landed.

## Chosen behavior contract (decisions D1-D3)

- D1 Selection-driven only: Direction auto-sets on hotspot row / extraction
  card clicks (via `selectHotspotRow`); metric-select changes never touch the
  toolbar.
- Mapping: burden/fanin -> Consumers ("in"); fanout/closure -> Dependencies
  ("out"); cycle -> toolbar untouched (bidirectional rank; flipping would
  reproduce a half-faded mismatch under the downstream-only heat).
- D2 Inbound metrics get an inbound heat view keyed to selection PROVENANCE
  (`hotspotHeatLens === 'in'`), not to dirSelect.value: `[ROOT]` badge stays;
  consumers heat-styled and badged with what THEY pay
  (`file_collateral[consumer][selected]`, verdict via
  `file_justification[consumer][selected]`); non-consumers faded; root's
  incoming edges verdict-tricolored; inbound legend copy "N payer(s) -
  M symbols paid". Manual Direction changes stay byte-identical to today.
- D3 Pending/consumed lens flags: `pendingHeatLens` set by hotspot click,
  consumed as the first statements of `selectNode`; `dirSelect.onchange`
  nulls both flags. `selectNode(id)` call literal preserved (existing test
  lock).

## Hard touch-set (implementer)

- `crates/ui/assets/hotspots.js` (lens globals, onchange reset,
  `hotspotMetricDirection`, `selectHotspotRow`, `computeFullConsumers`,
  `renderInboundHeatView`; 465 -> ~575 lines)
- `crates/ui/assets/graph.js` (2 lines at top of `selectNode`; leading
  delegate branch in `renderSvg` heat section; 990 -> <=996, hard cap +8)
- `crates/ui/assets/index.html` (one help `<li>`, ASCII entities for arrows)
- `crates/ui/tests/ui_test/hotspot_direction.rs` (new; 8 locks) + `mod` line
  in `crates/ui/tests/ui_test/main.rs`
- `tasks/hotspot-canvas-direction-journal.md` (implementer journal)
- Forbidden: main.js, styles.css, themes.css, crates/ui/src/**, tools/**,
  pre-existing tests, fixtures/goldens, ledger; no new asset file.

## Machine-checkable acceptance

- `python tools/testrun.py --distill -p ariadne-ui`: 0 failed; baseline 209
  passed + 8 new tests; pre-existing tests unedited (equivalence oracle for
  "manual direction unchanged").
- New locks: mapping; selectHotspotRow shape (keeps `selectNode(id)`);
  onchange lens clear; renderSvg delegate branch + preserved `} else if`;
  selectNode lens consumption; payer-side read expression; no-new-payload-
  fields regex scan; help-copy lock.
- `python tools/hygiene_check.py` clean; `theme_token_check --check-added`
  silent; `encoding_guard --staged` silent; `ui_smoke.py` clean (unchanged
  route roster); `git diff crates/cli` empty (DP-1).

## Landing recommendation

Not an M28 rider (M28 is operator-tail-only per ledger). Own small post-M28
change track, sequenced after the staged boot-overlay commit lands.
