# Shape - ui-degree-opacity-ramp (graph canvas graduated fade)

Coordinates below are from the WORKING TREE at shaping time (dirty with the
in-flight collateral-traversal-view track). All handoff edits are anchored
by content (the house `function_body` / `js_fn_body` string idiom), not by
line number. Kickoff preconditions gate the start; every decision the
planner needs is already made here.

## Decided contract (shape questions answered)

Q1 - Distance metric: undirected BFS hop distance from `selectedNodeId`
over the FULL `tabAllEdges` dataset. Direction: traverse BOTH `adjOut` and
`adjIn` (same adjacency-build pattern as `filterAndExpandSubgraph`,
`graph.js:946-952` working tree; the per-tab dataset reassignment is the
existing `tabAllEdges` global). Justification: faded nodes are exactly the
ones outside the selected node's OUT-closure - a direction-following
distance would be infinite for every faded node and the ramp would
degenerate to today's flat floor. The undirected neighborhood is the only
metric with signal, and it matches the user's mental model of "near". The
existing forward BFSes (`computeFullDependencies` at `graph.js` outbound
pass, `computeFullConsumers` in `hotspots.js`, `findIncludePath` at
`graph.js:704-720`) keep their semantics; this is a new, purely visual
walk. Visited-set BFS is cycle-safe.

Q2 - Ramp contract: per-element inline CSS custom property, values from two
frozen tables. Mechanism choice: custom property beats hop classes - no
class-explosion, one-line removal for the leak guard, and the CSS fallback
keeps every non-adopting call site pixel-identical with zero edits.
Theme-safe: opacity decimals are not hex literals, so
`theme_token_check.py --check-added` stays silent and `theme_js_zero_hex.rs`
(locks hex in `graph.js`) stays green; no touch of `themes.css`/`theme.js`.

```
/* styles.css .node-group.faded (working tree :437-441) - only the opacity
   line changes; the grayscale/brightness filter and the transitions stay
   uniform for every hop (a hopped-down card stays the same KIND of thing,
   just less attention): */
.node-group.faded {
  opacity: var(--fade-opacity, 0.12) !important;
  filter: grayscale(80%) brightness(0.65);
  transition: opacity 0.25s ease, filter 0.25s ease;
}
/* styles.css .edge-path.faded (working tree :494-496): */
.edge-path.faded {
  opacity: var(--edge-fade-opacity, 0.06) !important;
}
```

Frozen oracle tables (the implementation IS the table - a Rust test can
regex the literal out and recheck monotonicity/floor without executing JS):

```
/* hop index: table[hopper-clamped-to-1..6][d - 1]; d >= 6, Infinity, NaN,
   or undefined resolve to the LAST entry (today's values exactly). */
const HEAT_RAMP_NODE_OPACITIES = [0.85, 0.55, 0.36, 0.23, 0.15, 0.12];
const HEAT_RAMP_EDGE_OPACITIES = [0.3, 0.28, 0.18, 0.12, 0.08, 0.06];
```

Values: nodes 0.85 at d=1 (nearly opaque per the user's ask), geometric
~0.65 per hop, floor 0.12 == today. Edges = min(0.30, node/2) frozen
table; the 0.30 cap keeps every faded edge strictly dimmer than the
non-faded `.edges-has-selection` edge (0.35, `styles.css:504-506` -
untouched), so a faded edge can never outshine a kept one; floor 0.06 ==
today. JS signatures (globals; all asset scripts share page scope - the
`classifyDependency` cross-file call graph.js->hotspots.js is the precedent;
classic-script function declarations resolve at call time):

```
function computeHopDistances(rootId)   // Map<nodeId, hop>=1>; root absent
function hopFadeOpacity(hop)           // number -> string, table above
function edgeFadeOpacity(hop)          // number -> string, table above
```

`computeHopDistances`: seed `visited = {rootId}`, queue `{id, dist:0}`; per
pop, push both `adjOut[id]` and `adjIn[id]` neighbors not visited at
`dist+1`; return Map without the root entry. Reads ONLY `rootId`,
`tabAllEdges`, `tabAllNodes` - never `activeNodes`, `activeEdges`, `nodeCap`,
`currentDegree`, `isAllCapped`, search state, or seeds (invariant I3; cap
and seeding therefore cannot affect the ramp - Q5 answered by construction).

Application sites (property set is placed in the SAME branch that already
does `classList.add('faded')`, nothing else changes):

```
/* outbound pass, node loop else-branch (graph.js working tree :412-414),
   `hops` computed once before the node loop as
   computeHopDistances(selectedNodeId): */
el.classList.add('faded');
el.style.setProperty('--fade-opacity', hopFadeOpacity(hops.get(nid)));
/* outbound pass, edge loop faded-branch (:419-420): */
el.classList.add('faded');
el.style.setProperty('--edge-fade-opacity', edgeFadeOpacity(
  Math.max(hops.get(src) ?? Infinity, hops.get(dst) ?? Infinity)));
/* inbound lens mirrors it at hotspots.js working tree :362-364 and
   :370-371, same property names, own hops = computeHopDistances(...) */
```

Q3 - Edges: yes - a faded edge adopts `--edge-fade-opacity` keyed to its
FARTHER endpoint (`max(hop(src), hop(dst))`; missing endpoint = Infinity =
floor). The edge-fade condition itself (`!reachableDependencies.has(src) &&
src !== selectedNodeId` outbound; `dst !== selectedNodeId &&
!payers.has(dst)` inbound) is byte-identical - only the two property lines
are added inside the existing branches.

Q4 - Call sites: ADOPT the ramp at exactly the two heat-overlay passes -
outbound `graph.js:387-437` (the pass from the user's screenshot) and the
inbound twin `hotspots.js::renderInboundHeatView` (`:337-386`, fades at
`:363` and `:371`) - both lenses are "heat mode" and the flat fade there
draws the identical complaint. OUT of scope: `illuminatePath`
(`graph.js:735-755`) keeps its flat binary fade (declared above); any future
`traversal.js` fade keeps flat 0.12 via the fallback. Because DOM is
rebuilt only by `renderSvg` (`innerHTML = ''` at `:243-244`) but
`illuminatePath` mutates the post-heat DOM in place from the inspector
collateral links (`:641`), `illuminatePath` gains a leak-guard rider: in
each of its two loops, `el.style.removeProperty('--fade-opacity')` /
`removeProperty('--edge-fade-opacity')` on the faded branch (2 lines,
guarding its flat look - not a ramp adoption).

Q5 - Cap/ALL-WITH-CAP/nodeCap/seeds: answered by construction (I3). The
filter pipeline (`filterAndExpandSubgraph`, `graph.js:946-1003` working
tree) decides WHICH elements exist; `computeHopDistances` decides how dim
an existing faded one is, always from the selection over the full dataset.
The two never share inputs.

## Invariants (each independently checkable; lock or admission named)

I1. Floor-exactness: faded with no finite hop renders exactly 0.12 (node) /
    0.06 (edge). Locks: `hop_fade_ramp.rs` asserts both tables' last entry
    is 0.12 / 0.06 and `styles.css` contains
    `var(--fade-opacity, 0.12)` and `var(--edge-fade-opacity, 0.06)`
    (STYLES_CSS constant; regex extraction, no JS execution); amended
    `test_ui_noticeable_fading_elements` covers the page level.
I2. Monotonic ramp: node table strictly decreasing, first entry 0.85; edge
    table non-increasing, first entry <= 0.30. Lock: `hop_fade_ramp.rs`
    parses the two array literals (regex) and checks ordering/floors/caps
    numerically.
I3. Distance independence: `function_body(GRAPH_JS, "computeHopDistances")`
    contains `tabAllEdges` and must NOT contain `activeNodes`, `activeEdges`,
    `nodeCap`, `currentDegree`, `isAllCapped`, `seeds`, `isFiltering`.
    Lock: `hop_fade_ramp.rs` negative literals. (The open degree bug is
    likewise structurally excluded - it lives in the filter, which the ramp
    never reads.)
I4. Faded-set equivalence: every `.faded` condition literal stays
    byte-identical at all 4 heat sites and `illuminatePath`; property lines
    appear only inside the existing add-faded branches. Lock:
    `hop_fade_ramp.rs` asserts the 4 condition literals still occur and
    that the count of `setProperty('--fade-opacity'` occurrences is 1 in
    `graph.js` + 1 in `hotspots.js`, `--edge-fade-opacity` likewise.
    Residual: element-set equivalence at runtime is manual-smoke G6.
I5. Reachable treatment untouched: no line in the reachable/verdict branches
    of either heat pass changes (they gain no property, no class). Lock:
    commit-diff hunk review (reviewer hop) + the I4 count assertions
    (a setProperty in a reachable branch would raise the count past 2).
I6. illuminatePath stays flat: its body contains both
    `removeProperty('--fade-opacity')` and `removeProperty('--edge-fade-opacity')`
    and its condition literals are unedited. Lock: `hop_fade_ramp.rs` body
    checks; manual G6 line.
I7. Theme safety: zero added hex; `themes.css`/`theme.js`/
    `theme_token_baseline.json`/`index.html` byte-identical. Locks: G3 + G5.
I8. Bug non-interference (both directions): no diff hunk lands inside
    `filterAndExpandSubgraph`, `stepDegree`, `toggleAllCapped`, `setNodeCap`
    (reviewer: `git show` per commit, oldest-first, hunk-header check);
    I3 keeps the ramp from inheriting the bug's inputs. Explicitly: the
    correction row `ui.traversal_view_degree_setting_bug` stays OPEN after
    this track - this handoff neither fixes nor references it.
I9. Roster frozen: `tools/ui_smoke.py` and `index.html` byte-identical;
    `ui_smoke: PASS` on the unchanged route table. Lock: G2 + G5.

Declared amendment (precedent: hotspot-track AMENDMENT A1):
`static_html.rs::test_ui_noticeable_fading_elements` lines 155-156 lock the
pre-ramp literals `opacity: 0.12` / `opacity: 0.06`, which the `var(...)`
form removes. Amend exactly the two assert substrings to
`--fade-opacity, 0.12` / `--edge-fade-opacity, 0.06`, with a 1-line rationale
comment each; everything else in that test survives byte-identical. Also
verified: `hotspot_direction.rs::test_graph_delegates_inbound_heat_before_outbound_block`
is literal/order-level (`find`-based) and passes unedited - its prose about
byte-identical block bodies describes ITS track's landing, not a perpetual
lock; this track still must not move the lead/delegate literals.

## Module map

Touched (the complete touch set, `commit_scoped --scope` list):
- `crates/ui/assets/graph.js` - tables + 3 helpers (placement below), hops
  compute + 4 property lines in the outbound pass, 2 leak-guard lines in
  `illuminatePath`.
- `crates/ui/assets/hotspots.js` - hops compute + 4 property lines in the
  inbound lens pass.
- `crates/ui/assets/styles.css` - the 2 opacity declarations only.
- `crates/ui/tests/ui_test/hop_fade_ramp.rs` - NEW: I1-I4, I6 locks; copies
  the house `function_body` helper locally (modules do not import across
  each other - `hotspot_direction.rs:19-31` idiom).
- `crates/ui/tests/ui_test/main.rs` - +1 `mod hop_fade_ramp;` line.
- `crates/ui/tests/ui_test/static_html.rs` - the declared 2-line amendment.

Must NOT change (byte-identical): `themes.css`, `theme.js`,
`tools/theme_token_baseline.json`, `tools/ui_smoke.py`, `index.html`,
`crates/ui/assets/traversal.js`, `api.js`, `main.js`, `preview.js`,
`file-view.js`, `provenance.js`, all `crates/**/src/**`, `tests/fixtures/**`,
`tools/**`. (Foreign dirty-tree edits to some of these belong to the
traversal track; per the kickoff precondition they land BEFORE this handoff,
after which these are truly untouched by this track.)

## Placement decision rule (fixed at authoring time, one branch taken)

`headroom = 1000 - lines(graph.js)` at the landing commit of the traversal
track (the R2 wall - files < 1000 lines, `.js` IS scanned by
`hygiene_check.py`; the `<= 998` budget test at `static_html_sections.rs:215`
is a stricter lock that stays at 998 unedited):
- headroom >= 45 -> full placement in `graph.js` (canonical home of canvas
  BFS machinery); ~+40 lines.
- 6 <= headroom < 45 -> tables + 3 helpers (~34 lines) go into
  `hotspots.js` (632 lines now; shared globals make the call legal either
  way - the `classifyDependency` precedent runs the other direction);
  `graph.js` gains only its ~6 inline lines. `hop_fade_ramp.rs` file-
  ownership locks are authored against the branch taken.
- headroom < 6 -> STOP: report blocker; the deferred heat/legend extraction
  must precede. Do NOT raise the 998 lock or split this handoff.
Current working tree: graph.js at 1051 lines violates both - foreign debt,
kickoff precondition 2; shaping-time placement CANNOT be fixed until the
traversal track lands green.

## Risks -> mitigations -> oracle

R1. graph.js over the 998 budget and 1000 R2 wall on the dirty tree; the
    ramp cannot land on foreign debt. Mitigation: kickoff precondition 2 +
    placement rule + STOP branch. Oracle: `hygiene_check.py` exit 0; scoped
    ui (budget test unedited) green.
R2. `test_ui_noticeable_fading_elements` breaks on the `var(...)` form.
    Mitigation: declared 2-line amendment with rationale comments. Oracle:
    G1 green with exactly that test amended (touch-set includes it).
R3. Stale inline property leaking into flat-fade paths (`illuminatePath`
    mutates post-heat DOM without a rebuild). Mitigation: the 2
    `removeProperty` rider lines. Oracle: I6 body lock + manual G6 line.
R4. Faded edge brighter than a kept edge at hop 1. Mitigation: the 0.30 cap
    (strictly < 0.35). Oracle: I2 numeric check in `hop_fade_ramp.rs`.
R5. Custom-property inheritance side effects (properties inherit into SVG
    children). Mitigation: both consumers are element-scoped `.faded`
    opacity rules; nodes and edges live in separate layers so no shared
    ancestor carries the property; nothing else references either name.
    Oracle: source count lock (I4) + G6 visual pass.
R6. The traversal track's landing moves the cited coordinates (both tracks
    edit `graph.js`/`hotspots.js`/`styles.css`). Mitigation: content-anchored
    edits; re-verify anchors at handoff authoring per kickoff precondition 1.
    Oracle: implementer's pre-mutation anchor check (the anchor lines quoted
    above must be found verbatim before editing).
R7. Ramp too strong/weak for the corpus. Mitigation: the tables are the
    whole tuning surface; a retune is a 2-literal amendment + its test edit.
    Oracle: user eyeball on G6.

## Hygiene notes

- `styles.css` is 2103 lines, past the 1000 ceiling, but `.css` is outside
  `R2_SCANNED_SUFFIXES` (known open ledger row; not this track's decision).
  This change is net-zero lines there (2 declarations amended).
- `graph.js` 1051 lines (dirty): near/beyond both ceilings - governed by the
  placement rule and R1, and the extraction belongs to the deferred follow-up
  named in the kickoff scope-out.
- `hotspots.js` 632 lines - the only file with comfortable headroom; fine in
  either placement branch.
- New JS functions are all <= ~20 lines (no `renderSvg` growth beyond ~6
  inline lines). R1 does not parse `.js` functions, but the discipline holds
  anyway. Names follow R3 discipline even though `.js` is not R3-scanned:
  no `tmp`/`res`/`data`; `hops`, `edgeFadeOpacity`, `HEAT_RAMP_*` reveal
  intent.
- Authored content pure ASCII; edits via byte-safe tools only (PowerShell
  UTF-8 hazard row). Every new line far under 2000 chars.

## Test/oracle summary for the planner

Runtime gates: G1 scoped ui (`testrun.py --distill -- -p ariadne-ui`),
G2 `ui_smoke` via distill, G3-G5 lock proofs (kickoff table). JS behavior is
locked at source level (house idiom - the assets suite never executes JS);
the frozen tables make the numeric contract machine-checkable without a JS
engine; everything visual rides the journal's manual-smoke lines (hotspot-
track precedent). Reviewer hop re-runs G1-G5 unedited and enforces
commit-vs-touch-set: exactly the six touched paths, nothing else.
