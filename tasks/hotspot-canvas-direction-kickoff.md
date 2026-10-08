# Hotspot Canvas Direction - kickoff

Feature slug: `ui-hotspot-canvas-direction`
Date: 2026-10-08. Problem diagnosed and confirmed with the user in the live
session of 2026-10-08. Architect artifacts only; no implementation here.
Location note: the requested `docs/specs/<feature>.md` location is outside
this session's edit permissions (only `tasks/**` and `.kilo/plans/**` are
writable); the orchestrator may copy these two files to
`docs/specs/hotspot-canvas-direction-{kickoff,shape}.md` verbatim.

## State reconciliation (read first)

A session digest from 2026-10-08 claims a hotspot-direction fix was applied
to `crates/ui/assets/main.js` + `styles.css`. Current repo state contradicts
it: the staged `main.js`/`styles.css`/`static_html.rs` changes are the
boot-overlay step-bar quick fix (see
`tasks/boot-overlay-clean-track-journal.md`), and the working tree contains
no direction-follow code anywhere (`selectHotspotRow` at
`crates/ui/assets/hotspots.js:267` is still the bare `selectNode(id)`
wrapper). Repo state is fresher than memory: nothing of this change has
landed; these artifacts are the single source of truth for the shape.

## Intent

In the Include tab, the hotspot list ranks files by a metric whose direction
depends on the metric ("Collateral burden" is an INBOUND aggregate:
`burden[dep] = sum over roots of file_collateral[root][dep]`), but the
canvas keeps whatever Direction the toolbar holds (default: Dependencies,
outbound). Clicking the #1 burden offender renders a graph of that file's
own tiny outbound includes - the list answers "who pays for this file"
while the canvas shows "what this file pulls in".

Behavior change: clicking a hotspot row automatically sets the toolbar
Direction to the metric's natural direction before selecting the file, and
for inbound metrics the canvas paints an inbound heat view (consumers
badged with what THEY pay) instead of the downstream-only heat that would
fade every consumer node. Why: list and canvas must tell the same story or
the triage workflow (find a burden offender, see who pays, decouple) is
unusable at its most common entry point.

Blast radius: two frontend asset files (`hotspots.js`, small delegate in
`graph.js`), one help-center copy line in `index.html`, one new test module.
No server, no payload fields, no CLI byte-format, no CSS.

## Scope in

- `crates/ui/assets/hotspots.js`: metric-to-direction mapping, Direction
  auto-set on hotspot row / extraction-card click, inbound lens state,
  consumer heat view, lens reset on manual Direction change.
- `crates/ui/assets/graph.js`: lens consumption at the top of `selectNode`;
  new leading branch in the `renderSvg` heat section delegating to the
  consumer heat view. Existing outbound heat block stays byte-identical.
- `crates/ui/assets/index.html`: one help-center list item documenting the
  direction follow.
- `crates/ui/tests/ui_test/hotspot_direction.rs` (new) + its `mod` line in
  `main.rs`. Why a new file: `static_html.rs` sits at 999/1000 lines (the
  boot-overlay journal's deviation 2 already flagged it at the ceiling); any
  new lock there is an R2 blocking failure.

## Scope out (with reasons)

- New asset module (e.g. `heat.js`) despite `graph.js` at 990/1000: the
  asset roster is locked in three places outside `crates/ui/assets`
  (`server.rs` routes, `export.rs` inline list, `tools/ui_smoke.py` route
  table). A new asset drags `tools/` into the touch set, breaking the
  one-module handoff law for a structural nicety. Instead: delegate pattern,
  `graph.js` delta capped at +8 lines; extraction stays a named follow-up.
- Metric-select auto-flip (Decision D1 in the shape).
- Cycle-metric direction flip (Decision D3 in the shape).
- `main.js` (tour copy, globals, heat-legend helpers): the tour line "click
  a row ... see its heat map" stays literally true (the inbound consumer map
  IS its heat map now); the lens globals live in `hotspots.js` to keep
  `main.js` out of the touch set (it carries the uncommitted boot-overlay
  hunks).
- Runtime DOM test harness (jsdom-style): none exists in this repo; the
  house idiom is source-level shape locks + `ui_smoke` boot probe. Stated as
  the oracle limit, not silently assumed away.
- Esc/clearSelection resetting the toolbar Direction: Direction is visible
  toolbar state owned by the user after the click; auto-reverting it on
  deselect would surprise more than it helps. The lens is inert without a
  selection anyway.
- Milestone ledger edits: recommendation only, below.

## Invariants (full falsifiable definitions + locks: shape I1-I8)

1. Metric direction map: burden/fanin -> Consumers; fanout/closure ->
   Dependencies; cycle -> toolbar untouched.
2. Auto-set happens only on hotspot row / extraction-card clicks.
3. Every other selection path keeps today's behavior byte-identically
   (manual Direction changes, canvas clicks, inspector links, breadcrumbs,
   quick-open, filter fallback).
4. Inbound heat badges use `file_collateral[consumer][selected]` - what the
   consumer pays - never the root-side map.
5. No new payload fields, no new asset routes, no theme hex, no CSS.
6. The static default `<option value="out" selected>` is untouched.

## Frozen locks inherited (each with its proof command)

| Lock | Proof command |
| --- | --- |
| `dirSelect` default stays `out` | `python tools/testrun.py --distill -p ariadne-ui` (static_html.rs `test_ui_dependency_tree_direction_preserving`) |
| `selectHotspotRow` routes through the literal `selectNode(id)` | same suite (`assert_hotspot_burden_and_list_rules`) |
| 8-script asset roster + export order unchanged | same suite (theme_contract.rs, export_order.rs) + `python tools/ui_smoke.py` |
| CLI byte format (DP-1) untouched | `python tools/distill.py -- git diff HEAD -- crates/cli` -> empty capture |
| All pre-existing UI tests pass UNEDITED (equivalence oracle) | `python tools/testrun.py --distill -p ariadne-ui` |
| No mojibake / non-UTF-8 | `python tools/encoding_guard.py --staged` |
| No new hex literals in added lines | `python tools/theme_token_check.py --check-added --base <pre-change rev>` |
| Files < 1000 lines, functions < 100 lines | `python tools/hygiene_check.py` |

## Milestone landing recommendation

NOT an M28 rider. The ledger row for M28 (2026-10-08) reads "ALL DEVELOPMENT
HANDOFFS LANDED-APPROVED ... OPERATOR TAIL ONLY"; adding dev work would
reopen a milestone waiting only on baseline commit + scratch purge. Land this
as its own small post-M28 change track (precedent: the boot-overlay clean
track - own journal, scoped commits, no ledger row); do not register an M29
number for it (the ledger is the numbering authority; M29 appears only in
stale roadmap placeholders). Sequence it AFTER the staged boot-overlay commit
lands to avoid `static_html.rs`/`main.js` hunk contention - this change's
touch set then shares no file with it.

## Acceptance gate (milestone as a whole)

All of: (a) every frozen lock above green; (b) the eight new source locks in
`hotspot_direction.rs` green; (c) the full `-p ariadne-ui` suite green with
ZERO edits to pre-existing tests (the equivalence oracle for "manual
direction changes must keep working unchanged"); (d) `python tools/ui_smoke.py`
boots clean on the unchanged route roster; (e) the implementer journal
records one manual smoke line: clicking a burden row shows the toolbar
reading "Consumers (<-)" with consumer cards badged with their payments,
not faded.
