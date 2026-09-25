# Include Hotspots Panel — ranked problem areas in the include graph

## Goal

Add a "Hotspots" discovery feature to the Web UI include tab: a docked toggle panel listing worst-offender files ranked by a switchable metric, so users can *find* problem include paths instead of having to select nodes first to see the existing per-selection heat map.

## Context (verified against source)

- UI is a single vanilla-JS page: `crates/ui/assets/index.html` (~2799 lines). No framework, no build step.
- Include-graph nodes already carry `in_degree`, `out_degree`, `closure`, `is_cycle`, `path`, `kind` (built in `crates/ui/src/dto.rs:128` `build_include_elements`).
- `graphData.file_collateral` is a full `rootFileId -> { headerFileId: symbolCount }` map for **every** file, already shipped in both live mode (`/api/export`) and standalone export; built in `crates/cli/src/commands/ui.rs:23-38` (live) and `:117-136` (export).
- Existing selection-driven heat map (`applyHeatAndFade`-style logic at index.html:1772+), heat badges, collateral breakdown, thread-bar path illumination all key off `selectNode(id)` and `selectedNodeId`.
- Directory scope (`dirScope`, chip in header) gates include-tab search/quick-open; `tabAllNodes` holds all include nodes with `/`-relative `path`.
- Existing help center section for the include tab (~index.html:1225) documents heat map workflow.
- UI tests assert on HTML string content (e.g. `crates/ui/tests/ui_test.rs:671` asserts `html.contains("file_collateral")`).

**No backend, DTO, or CLI changes are needed.** Everything the panel ranks on is already in the browser payload.

## Aggregation (client-side)

Compute once per payload load, cache, invalidate on payload reload (index.html ~1439 where `graphData.file_collateral` is assigned):

- **Collateral burden** (default): `burden[dep] = Σ over all roots of file_collateral[root][dep]`; also keep `affectedRoots[dep]` (count) for the row tooltip. If `file_collateral` is empty (older export JSON), fall back to Fan-in and show a one-line hint in the panel.
- **Fan-in**: node `in_degree`.
- **Fan-out**: node `out_degree`.
- **Closure burden**: node `closure`.
- **Cycle participation**: nodes with `is_cycle`, ranked by `in_degree + out_degree` desc.

## Decisions (agreed with user)

1. Ranked hotspot panel with **switchable metrics**; all 5 metrics, default Collateral burden.
2. **Docked toggle panel**, not a modal — the graph stays visible so clicking rows renders the heat map live behind the open list.
3. Feature is **include-tab only**; hidden on symbol/call tabs (same gating pattern as quick-open).

## Tasks (ordered)

1. **Markup** — in the secondary toolbar (next to Pathfinder/Entrypoints, index.html:990), add `<button id="btnHotspots">🔥 Hotspots</button>` (visible only when `currentTab === 'include'`, toggle behavior). Add floating panel `<div id="hotspotPanel">` inside the graph area: header with `<select id="hotspotMetric">` (5 options) + close ✕, scope hint line, scrollable `<div id="hotspotList">`.
2. **CSS** — `.hotspot-panel`: absolute, top-left of graph area (mirror `.legend-box` glass style at index.html:694, `z-index: 25`), width ~300px, `max-height: calc(100% - 40px)` so it never collides with the bottom-left legend; `.hotspot-row` with metric value + horizontal bar normalized to the top score; `.hotspot-row.active` for the selected node.
3. **JS** —
   - `computeHotspots(metric)`: single pass over `tabAllNodes` (rank ALL files, not just the visible capped subset); burden map aggregated once and cached in a module-level var, invalidated when `graphData.file_collateral` is reassigned.
   - `renderHotspots()`: top 25 rows; each row shows rank, file label, path (dim), value, bar; tooltip = `N file(s) pay M symbols` for burden metric. Respect `dirScope` when set (filter by `path` prefix; show scoped-path in the hint line).
   - Row click → `selectNode(id)` (existing flow at index.html:1773+ applies fade/heat/legend/inspector); mark clicked row `.active`; panel stays open for click-through browsing. `filterAndExpandSubgraph` already seeds from `selectedNodeId` (index.html:2290), so rows outside the current view still become visible — verify this.
   - Wire-up: toggle button, metric `<select>` onchange, `switchTab` (hide + re-show button per tab), `setDirScope`/`clearDirScope` (re-render), payload reload (invalidate cache + re-render if open).
4. **Help center** — add a Hotspots paragraph + workflow step to the include-tab help section (index.html ~1225-1230), matching existing tone.
5. **Tests** — follow the repo's string-assertion style in `crates/ui/tests/ui_test.rs`: assert the embedded HTML contains `hotspotPanel`, `computeHotspots`, `file_collateral` usage in the aggregation, and the `btnHotspots` toggle; assert hotspot gating on `currentTab === 'include'` is present.

## Edge cases / risks

- **Standalone export mode**: export HTML embeds the same `index.html` with an inlined payload, so the panel works there for free — but only if `file_collateral` was serialized (it is skipped when empty; fallback path covers it).
- **Old exported JSON without `file_collateral`**: default metric silently degrades to Fan-in with an inline hint; no error state.
- **Large repos**: burden aggregation is one pass over the already-downloaded map; results cached; list capped at 25 rows.
- **Rows for nodes not in the capped active view**: verify `selectNode` + re-expansion brings the node on-screen; if it doesn't, force `filterAndExpandSubgraph()` after selection.
- **index.html size**: already 2799 lines, above the 1500-line guidance in AGENTS.md; this adds ~150 lines. Splitting the UI file is out of scope here — flag as follow-up, don't mix a refactor into this feature.

## Validation

1. `cargo test -p ariadne-ui` (string-assertion tests + existing suite must stay green).
2. Manual against the include fixture (repo with `include/heavy.h` per earlier fixtures): `cargo run` UI on the fixture, open Hotspots —
   - default ranking = collateral burden, `heavy.h` near top with plausible bar lengths;
   - switching each of the 5 metrics re-ranks correctly (spot-check fan-in against a node's in-inspector degree);
   - click a row → heat colors, fading, badge counts, and inspector collateral breakdown render behind the open panel;
   - set a dir scope chip → list re-filters to that subtree; clear chip → full list back;
   - `ariadne ui --export out.html`, open `out.html` → panel works from inlined payload; delete `file_collateral` from the inlined JSON → Fan-in fallback hint shows.
3. Switch to Symbol/Call tab → button and panel hidden; switch back → state restored.

## Open questions

None — all design decisions resolved with the user.
