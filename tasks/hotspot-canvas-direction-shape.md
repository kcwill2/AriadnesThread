# Hotspot Canvas Direction - shape

Companion to `tasks/hotspot-canvas-direction-kickoff.md`. Same feature slug
`ui-hotspot-canvas-direction`, 2026-10-08. All JS/HTML snippets are contracts
for the implementer; source-level string locks in the named tests are the
enforcement, per the house idiom (extraction_card_playbook.rs `function_body`
helper pattern).

## Decisions on the three open questions

### D1 - selection-driven only

Direction auto-sets on hotspot row and extraction-card clicks (both route
through `selectHotspotRow`), never when the metric select changes.
Rationale: changing the metric re-ranks the list; the canvas keeps its
current story until the user commits to a row. Auto-setting the toolbar on
dropdown change would hijack a manually chosen Direction while nothing new
is selected - maximum surprise, zero added story. The no-collateral-data
fallback (burden -> fanin, `renderHotspots`) rewrites the metric select to an
inbound metric, so D1 stays coherent under the fallback.

### D2 - inbound heat: mirror the heat view, keyed to selection provenance

With Direction "in" on a burden-selected file the canvas shows:

- the root badged `[ROOT]` with class `heat-root` (unchanged idiom);
- every transitive consumer heat-styled and badged with what THAT consumer
  pays: `file_collateral[consumer][selected]`, tiered and spined by
  `file_justification[consumer][selected]` through the existing
  `classifyDependency` / `styleDependencyCard` / `verdictBadgeText` /
  `appendVerdictTip` helpers (badge reads "k/m used" from the consumer's
  side - the tooltip strings stay honest because `appendVerdictTip(el,
  consumerLabel, rootLabel, ...)` composes "<consumer> uses k of m symbols
  from <selected>");
- every non-consumer node faded; the root's INCOMING edges verdict-tricolored
  (`edge-justified` / `edge-partial` / `edge-unused` + `arrow-unused` marker
  + tip, driven by `classifyEdgeVerdict(file_justification[src][selected])`);
- an inbound legend with its own copy - "N payer(s) - M symbols paid", rows
  reused from `HEAT_LEGEND` and `verdictCardLegendRows` - because reusing
  `updateHeatLegend` verbatim would print "N dep(s)" and "Needed by selected
  file", which is the outbound story told backwards.

This is the minimum that tells the burden story honestly. The alternative
(leave the outbound heat pass running under inbound expansion) fades the
entire expanded neighborhood - the exact defect being fixed. Crucial binding:
the inbound heat is keyed to a selection-PROVENANCE flag
(`hotspotHeatLens === 'in'`), NOT to `dirSelect.value`. That is what keeps
MANUAL Direction changes byte-identical to today (their lens is null). The
lens is consumed - and the pending flag cleared - as the FIRST statements of
`selectNode`, so no other selection path can inherit it.

### D3 - cycle keeps the current Direction

The cycle rank is in+out degree: a bidirectional number with no single
natural direction. Flipping to "both" would expand consumers too, but the
existing downstream-only heat would fade them all, reproducing a half-faded
mismatch that this change exists to eliminate; and inventing a bidirectional
heat view doubles the render surface with no user ask. Keeping the outbound
view is what happens today, and the cycle decoupling workflow (which of MY
includes are unused - the verdict story for cycle members) stays intact.
"Keep current" is the only choice that adds no new mismatch class.

## Invariants (each independently checkable; named lock)

- **I1** `hotspotMetricDirection('burden') === 'in'`, `'fanin' === 'in'`,
  `'fanout' === 'out'`, `'closure' === 'out'`, `'cycle' === null`.
  Lock: `test_hotspot_metric_direction_mapping`.
- **I2** `selectHotspotRow` body contains the mapping call, the `dirSelect`
  assignment, the pending-lens assignment, AND still contains the literal
  `selectNode(id)` with no added arguments (a pre-existing static_html.rs
  lock greps for it). Lock:
  `test_select_hotspot_row_follows_metric_direction`; pre-existing
  `assert_hotspot_burden_and_list_rules` must pass UNEDITED.
- **I3** Lens flags: `selectNode` starts with `hotspotHeatLens =
  pendingHeatLens;` + `pendingHeatLens = null;`, and the `dirSelect`
  onchange handler nulls both flags before calling `filterAndExpandSubgraph`.
  Locks: `test_select_node_consumes_pending_lens`,
  `test_direction_change_clears_lens_flags`.
- **I4** Inbound payment lookup is consumer-first: the `renderInboundHeatView`
  body contains `((graphData.file_collateral || {})[nid] || {})[selectedNodeId]`
  and does NOT contain `computeFullDependencies` (it uses its own reverse BFS
  `computeFullConsumers`). Lock:
  `test_consumer_heat_view_reads_payer_side`.
- **I5** Outbound heat block byte-identical: `renderSvg` gains ONLY a leading
  `if (currentTab === 'include' && hotspotHeatLens === 'in' &&
  selectedNodeId)` branch plus the `} else if` rewrite of the old condition
  line; everything from `const reachableDependencies` through
  `updateHeatLegend(...)` and the trailing `else { restoreStandardLegend(); }`
  is untouched. Lock:
  `test_graph_delegates_inbound_heat_before_outbound_block` (asserts
  `} else if (currentTab === 'include' && selectedNodeId) {` present), plus
  every pre-existing heat/verdict test (node_verdict_spine,
  edge_verdict_story, justification_builders/golden) green UNEDITED.
- **I6** No new payload surface: the only `graphData.<field>` reads inside
  `renderInboundHeatView` are `file_collateral` and `file_justification`.
  Lock: `test_inbound_heat_reads_no_new_payload_fields` (regex scan of the
  function body).
- **I7** No new asset: `INDEX_HTML` still lists exactly the 8 `/app/*.js`
  script tags and contains no `heat.js`. Lock:
  `test_help_copy_documents_direction_follow` keeps copy checks separate;
  roster proven by the pre-existing theme_contract + export_order locks and
  the `ui_smoke` route table.
- **I8** Copy: the new `index.html` help list item contains `natural
  direction` and names the cycle exception; the tour step "Hotspots Triage"
  remains present in untouched `main.js`. Lock:
  `test_help_copy_documents_direction_follow` + pre-existing tour lock.

## Contracts (exact shapes)

```js
// hotspots.js - module-private lens state, declared at the top of the file,
// above the dirSelect handler. Script load order (api, graph, file-view,
// hotspots, preview, provenance, main, theme) guarantees evaluation before
// any renderSvg/selectNode call happens at boot or by user action.
let hotspotHeatLens = null;    // null | 'in' - provenance of current selection
let pendingHeatLens = null;    // set by hotspot clicks, consumed by selectNode

// hotspots.js - replaces the current one-line onchange (line 2):
document.getElementById('dirSelect').onchange = () => {
  hotspotHeatLens = null;
  pendingHeatLens = null;
  filterAndExpandSubgraph();
};

// hotspots.js - new pure mapping (locked by I1):
function hotspotMetricDirection(metric) {
  if (metric === 'burden' || metric === 'fanin') return 'in';
  if (metric === 'fanout' || metric === 'closure') return 'out';
  return null; // cycle: bidirectional rank - toolbar keeps the user's pick
}

// hotspots.js - replaces the current 3-line body; keeps the selectNode(id)
// literal EXACTLY (I2); the lens travels via pendingHeatLens, not arguments:
function selectHotspotRow(id) {
  const dir = hotspotMetricDirection(document.getElementById('hotspotMetric').value);
  if (dir) document.getElementById('dirSelect').value = dir;
  pendingHeatLens = dir === 'in' ? 'in' : null;
  selectNode(id);
}

// graph.js selectNode - two statements added at the very top (net <= +3 lines
// with the comment); signature UNCHANGED:
function selectNode(id, skipFilter = false, skipCrumb = false) {
  hotspotHeatLens = pendingHeatLens;
  pendingHeatLens = null;
  ...unchanged...
}

// graph.js renderSvg heat section - leading branch only (net <= +3 lines):
if (currentTab === 'include' && hotspotHeatLens === 'in' && selectedNodeId) {
  renderInboundHeatView(nodesLayer, edgesLayer);
} else if (currentTab === 'include' && selectedNodeId) {
  ...existing block, byte-identical (I5)...
} else {
  restoreStandardLegend();
}

// hotspots.js - reverse BFS mirroring computeFullDependencies (graph.js:454),
// same full-dataset scan semantics, direction flipped:
function computeFullConsumers(nodeId) {
  const payers = new Set();
  const queue = [nodeId];
  while (queue.length > 0) {
    const curr = queue.shift();
    tabAllEdges.forEach(e => {
      if (e.target === curr && !payers.has(e.source)) {
        payers.add(e.source);
        queue.push(e.source);
      }
    });
  }
  return payers;
}

// hotspots.js - the inbound mirror of the graph.js:327-393 heat pass.
// < 100 lines. Reuses globals (main.js / graph.js), all UNMODIFIED:
// getHeatColor, classifyDependency, styleDependencyCard, verdictBadgeColor,
// verdictBadgeText, classifyEdgeVerdict, appendVerdictTip, HEAT_LEGEND,
// verdictCardLegendRows, escapeHtml.
function renderInboundHeatView(nodesLayer, edgesLayer) {
  // payers = computeFullConsumers(selectedNodeId)
  // paid(nid)  = ((graphData.file_collateral || {})[nid] || {})[selectedNodeId] || 0
  // just(nid)  = ((graphData.file_justification || {})[nid] || {})[selectedNodeId]
  // maxCollateral = max of paid over payers; total = sum of paid
  // selected node: classList.add('heat-root') + '[ROOT]' badge (same idiom)
  // payer nodes: tier = classifyDependency(nid, paid(nid), just(nid));
  //   styleDependencyCard(el, tier, paid(nid), maxCollateral);
  //   badge text = tier === 'neutral' ? '+' + paid + ' col'
  //                                 : verdictBadgeText(tier, just(nid));
  //   verdict = classifyEdgeVerdict(just(nid)); if (verdict)
  //     appendVerdictTip(el, consumerLabel, rootLabel, verdict, just(nid));
  //   (tooltip then reads "<consumer> uses k of m symbols from <selected>")
  // all other nodes: classList.add('faded')
  // edges: faded when dst !== selectedNodeId && !payers.has(dst);
  //   edges with dst === selectedNodeId (and src !== dst) get
  //   edge-justified (+arrow-entering marker, mirroring the outbound block),
  //   edge-partial, or edge-unused (+arrow-unused marker + appendVerdictTip)
  //   from classifyEdgeVerdict(just(src))
  // legend: own copy - legendViewLabel 'Heat Map'; summary line
  //   payers.size + ' payer(s) - ' + total + ' symbols paid';
  //   edge rows: green 'Payer fully needs it', amber partly/no-data row,
  //   red 'Payer uses none of it - include can go' (gated on verdict tiers
  //   present, same gating idiom as updateHeatLegend); card spine rows via
  //   verdictCardLegendRows; heat tier rows via HEAT_LEGEND.
  //   Token-only colors - no hex literals anywhere (theme_token_check).
}
```

Copy contract (`index.html`, inserted as one new `<li>` after the "Clicking
a row selects the file and renders its heat map live beside the open list."
item, sentence-per-line, ASCII, HTML entities for arrows):
"Clicking a row follows the metric's natural direction: Collateral burden
and Fan-in switch the canvas to Consumers (&larr;) and badge each paying
file with what it pays; Fan-out and Closure burden switch to Dependencies
(&rarr;); Cycle participation keeps your current Direction."

Degradation contract: absent justification entry -> `neutral` tier,
`+N col` badge, no spine, no tip - exactly what the outbound view does for
deps without entries. A consumer that pays true-zero shows `+0 col`, the
parity case with the outbound view; no data-vs-zero special case invented.
Error shapes: none new - no fetch, no throw sites; all map lookups are the
`|| {}` guarded idiom already in `getBurdenMaps`/renderSvg.

## Module map

| File | Kind | What |
| --- | --- | --- |
| `crates/ui/assets/hotspots.js` | changes | lens globals, onchange reset, `hotspotMetricDirection`, new `selectHotspotRow`, `computeFullConsumers`, `renderInboundHeatView` (465 -> ~575 lines) |
| `crates/ui/assets/graph.js` | changes | `selectNode` lens consumption (+2-3 lines), `renderSvg` leading branch (+3 lines); 990 -> <=996, HARD CAP +8 |
| `crates/ui/assets/index.html` | edits | one help `<li>` (copy contract above) |
| `crates/ui/tests/ui_test/hotspot_direction.rs` | new | the 8 I-locks in the house idiom (local `function_body` helper copied from extraction_card_playbook.rs) |
| `crates/ui/tests/ui_test/main.rs` | edits | one `mod hotspot_direction;` line |
| `tasks/hotspot-canvas-direction-journal.md` | new | implementer journal (workflow convention) |
| must NOT touch | - | `main.js`, `styles.css`, `themes.css`, `crates/ui/src/**`, `tools/**`, every pre-existing test file, fixtures, goldens, `Intent.md`/`Plan.md`/`Spec.md`, `tasks/milestone_ledger.md`, and the staged boot-overlay files until that commit lands |

## Risks -> mitigation -> oracle

- **R1 graph.js ceiling** (990/1000): delta hard-capped +8 by this shape;
  the ~110 new lines live in hotspots.js (465 -> ~575). Oracle:
  `python tools/hygiene_check.py` R2 in gates. Named follow-up (NOT this
  change): heat/legend module extraction deserves its own handoff planned
  together with the asset-roster cost (server.rs + export.rs + ui_smoke.py).
- **R2 lens leak** (inbound heat on a manual selection would be a new lie):
  consumption as the first statements of `selectNode` + onchange clear.
  Oracle: I3 locks; every pre-existing manual-path test (static_html,
  edge_verdict_story, node_verdict_spine) green UNEDITED.
- **R3 breaking the `selectNode(id)` literal lock** (pre-existing
  static_html.rs assert): the contract forbids passing the lens as a
  `selectNode` argument; the pending-flag handoff keeps the literal. Oracle:
  I2 + the pre-existing lock green.
- **R4 transient wrong heat inside the click** (`selectNode` renders once
  with the previous expansion before `filterAndExpandSubgraph` re-renders):
  all renders are synchronous within one event frame; the browser paints
  only the final state; `renderSvg` rebuilds both layers from scratch per
  call, so no stale DOM. Accepted; recorded because a future async refactor
  must revisit it.
- **R5 legend copy lies in inbound mode** ("N dep(s)", "Needed by selected
  file"): inbound view owns its copy ("payer(s)", "Payer ... needs it");
  `updateHeatLegend` untouched so the outbound path stays byte-identical.
  Oracle: I4/I5 locks + `theme_token_check` + encoding guard (ASCII copy).
- **R6 extraction-card clicks inherit the flip** (cards call
  `selectHotspotRow(target_file_id)`): intended - a plan card clicked under
  the burden metric shows who pays the proposed target, the same story the
  card's collateral numbers tell. Oracle: extraction_card_story /
  extraction_card_playbook locks green unedited.
- **R7 cycle rows keep the list/canvas mismatch**: accepted per D3; the new
  help copy states the exception so the behavior is documented, not hidden.
- **R8 no runtime oracle for the click itself** (no jsdom-style harness
  exists - boot-overlay journal "Deliberately not done"): strongest machine
  oracle is the 8 shape locks + clean `ui_smoke` boot + the journal's
  one-line manual smoke note. Stated limitation of the acceptance design.
- **R9 fallback coherence**: with an empty `file_collateral` payload,
  `renderHotspots` rewrites the metric to fanin (also 'in'); inbound heat
  then shows payers with `+0 col` / no-spine cards - honest emptiness, no
  fabricated numbers. Oracle: degradation contract above + I4/I6 locks.

## Hygiene notes

- `graph.js` 990/1000 - at ceiling; the +8 cap is a binding contract, and
  the extraction follow-up is named in R1 rather than discovered mid-work.
- `static_html.rs` 999/1000 - cannot receive any new lock (boot-overlay
  deviation 2 precedent); hence the new test module. The new module also
  keeps its own file small (~200 lines).
- `hotspots.js` 465 -> ~575; `renderInboundHeatView` must stay < 100 lines
  (mirror of the 67-line block it parallels); no source line near 2000
  chars; authored content pure ASCII (`&larr;`/`&rarr;` entities in HTML;
  badge/legend JS strings ASCII; use `->`, `<-`, `-` not arrows/em-dashes in
  JS/CSS/RS authored text). No file writes through PowerShell
  Set-Content/Out-File per AGENTS.md.
- Handoff sizing: 1 crate dir (`crates/ui`) + tasks journal; 6 scoped gate
  commands; estimated produced lines ~350-450. Fits exactly one handoff.

## Acceptance criteria (machine-checkable)

Gate commands (distilling entry points per AGENTS.md):

1. `python tools/testrun.py --distill -p ariadne-ui` -> `0 failed`;
   baseline `209 passed / 0 failed / 0 ignored / 3 suites` (verbatim from
   the boot-overlay journal) plus the 8 new tests; reviewer confirms via
   per-commit `git show` that no pre-existing test file was edited.
2. `python tools/hygiene_check.py` -> no new R1/R2 rows (graph.js still
   under 1000).
3. `python tools/theme_token_check.py --check-added --base <rev>` -> silent.
4. `python tools/encoding_guard.py --staged` -> silent, exit 0.
5. `python tools/ui_smoke.py` -> boot + route checks clean (roster unchanged).
6. `python tools/distill.py -- git diff HEAD -- crates/cli` -> empty capture.

New test names in `crates/ui/tests/ui_test/hotspot_direction.rs`
(one falsifier per invariant; I7 rides the pre-existing roster locks):
`test_hotspot_metric_direction_mapping`,
`test_select_hotspot_row_follows_metric_direction`,
`test_direction_change_clears_lens_flags`,
`test_graph_delegates_inbound_heat_before_outbound_block`,
`test_select_node_consumes_pending_lens`,
`test_consumer_heat_view_reads_payer_side`,
`test_inbound_heat_reads_no_new_payload_fields`,
`test_help_copy_documents_direction_follow`.

Equivalence oracle: every pre-existing `ariadne-ui` test is the frozen
suite; it must pass UNEDITED. "For every manual selection/direction path
the rendered behavior is identical before and after" is enforced that way -
the only practical stand-in in this repo for canvas-behavior fixtures, given
no DOM harness exists (R8).
