# Edge Verdict Story — "this include is unnecessary" made intuitive

## Problem

The include heat map paints a root-relative verdict on the dependency's **node card** (heat fill + `+11 col`), while the claim is about the **incoming edge**: "the selected root uses none of what this header supplies". A red node with no visible outgoing edges (e.g. jobserver.h, whose includes are all stdlib) reads as "this file is bloated" — the wrong story. All arrows leaving the selected root are uniform amber (`edge-leaving`), so the arrow making the claim carries zero information. `+N col` is jargon and hides the "0 used" half of the truth.

## Decision (user-approved)

Full edge story: tri-color the root's outgoing arrows + self-explaining `u/t used` badges + hover tooltips + legend/help copy. UI-only change; no Rust graph-crate changes.

## Design spec

All verdicts only when `file_justification[selectedRoot][depId]` has an entry. No entry = "neutral": keep current visuals exactly (no claims from absence of data).

1. **Badges** (in `renderSvg` heat pass, graph.js:343-370). Tiered by `classifyDependency` (graph.js:393), always root-relative:
   - justified: `N/N used` (green; replaces `N sym`)
   - mixed: `{j}/{j+c} used` (heat fill + green underline kept)
   - collateral-only: `{c}/{c}` renders as `0/{c} used` in heat color — the count keeps the drag weight, the `0` is the claim
   - neutral: `+N col` unchanged (also keeps "col" for existing tests)
2. **Edge tri-color** on the root's **direct outgoing, non-cycle, non-self edges** (post-pass loop at graph.js:376-381, which already iterates `.edge-path`):
   - justified → class `edge-justified` (green solid) + `marker-end` set to `url(#arrow-entering)` (green def exists)
   - mixed → class `edge-partial` (solid amber; identical to today's leaving color — the badge carries the fraction) + keep `url(#arrow-leaving)`
   - collateral-only → class `edge-unused` (red, `stroke-dasharray`) + new marker `url(#arrow-unused)` (red def) + `<title>` tooltip on the path itself
   - neutral / cycle / transitive edges → untouched amber leaving style. Per-hop classification of transitive edges is OUT OF SCOPE (payload classifies root→contributor only; badge+tooltip carry those).
3. **Node tooltips** (`<title>` child on dep `.node-group`, same pass as badges; precedent: preview.js:267):
   - with entry: `graph.h uses 0 of 11 symbols from jobserver.h.`
   - collateral-only adds: ` Nothing here is needed — this include is droppable or slimmable.` plus the caveat ` AT graphs project includes only; system/stdlib headers are not analyzed or counted.` (caveat only on the misread-prone tier)
   - justified: `... uses 5 of 5 symbols from dyndep.h — needed.`
4. **Legend** (`updateHeatLegend`, main.js:105): when any entry exists, prepend rows green = "Needed by selected file", red = "Unused — this arrow can go". Fixed muted footnote on the heat legend: `Verdicts relative to selected file · system headers not analyzed`.
5. **Help copy** (index.html:414 paragraph): extend with the arrow vocabulary and the stdlib-scope caveat, ~2 sentences.

## Constraints

- **R2 gate**: `hygiene_check.py` caps .js/.html at 1000 lines; graph.js is at 940. Put the string builders (`verdictBadgeText`, `verdictTooltipText`, `classifyEdgeVerdict`) in **main.js** (home of `getHeatColor`/legend helpers); graph.js delta must stay ≤ ~45 lines.
- **CSS cascade**: insert new edge rules AFTER `.edge-path.edge-leaving` (styles.css:497) but BEFORE the E3 preview block (styles.css:1247+) so `preview-slim-edge`/`preview-proposed-edge` keep winning (equal-spec `!important` rules resolve by order).
- Red dashed = "unnecessary/eliminated" is coherent with the preview's red "eliminated" nodes; cycle edges keep their red solid + glow and are never tri-colored.
- Do NOT claim "0 used" when no entry exists (justified unknown ≠ 0).
- No server/DTO changes: `file_justification` + `file_collateral` already ride both the live `/api/export` poll (api.js:56,112) and standalone report payloads.

## Tasks (TDD — failing asset-content assertion first, following `crates/ui/tests/ui_test/static_html.rs` style)

1. **Marker + CSS**: add `arrow-unused` marker (red, mirrors index.html:211 arrow-cycle geometry) to index.html defs; add `.edge-path.edge-justified` (green solid, glow) and `.edge-path.edge-unused` (#ef4444, `stroke-dasharray: 6 4`, `stroke-width: 2px`) in styles.css at the cascade position above. Tests: `STYLES_CSS` contains both rules with dasharray/stroke; `INDEX_HTML` contains `arrow-unused`.
2. **main.js helpers**: `classifyEdgeVerdict(entryOrNull) -> 'justified'|'partial'|'unused'|null`; `verdictBadgeText(tier, entry)`; `verdictTooltipText(rootLabel, depLabel, tier, j, c)` (caveat text on unused only). Tests assert helper presence + copy fragments (`used`, `droppable`, `system/stdlib headers are not analyzed`).
3. **graph.js wiring**: badge via `verdictBadgeText`; append `<title>` per reachable dep card; extend the edge fade loop to apply class + marker override for the root's direct edges (skip `is_cycle`, `source===target`, null verdict). Keep `dep-justified`/`dep-mixed`/heat behavior unchanged. Tests: `GRAPH_JS` contains `edge-justified`/`edge-unused` application and tooltip title creation; keep existing `classifyDependency`/`dep-justified` assertions green.
4. **Legend + help copy**: `updateHeatLegend` tri-state rows + footnote; index.html:414 help sentence. Tests: HTML/JS contain the new copy anchors.
5. **Verify**: `cargo test` (workspace or `-p ariadne-ui` at minimum), `python tools/hygiene_check.py` exit 0, line counts of graph.js/main.js < 1000, `cargo clippy` clean baseline unchanged. Manual: `cargo build --bin tool && target\debug\tool.exe ui <analysis-project>`; select graph.h — jobserver.h shows red dashed arrow + `0/11 used`; hover shows the tooltip with caveat; dyndep.h arrow turns green; Esc restores; Esc/preview interplay unchanged (preview still overpaints, re-select restores tri-color via renderSvg rebuild).

## Acceptance scenario (the screenshot case)

With graph.h selected: arrow graph.h→jobserver.h is red dashed, badge `0/11 used`, hover states the relation in one sentence including "system/stdlib headers are not analyzed" — a first-time viewer cannot conclude jobserver.h's own includes were judged.
