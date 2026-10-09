# T1 Handoff - ui-degree-opacity-ramp: graduated hop-distance fade on the graph canvas

Authored 2026-10-09 by the planner role. Unregistered post-M28 track per
kickoff: NO M-number; `tasks/milestone_ledger.md` stays numbering authority.
Installment 2 of TWO SERIALIZED handoffs (Option B, amendment 2026-10-09).
The frozen ramp contract is UNCHANGED by the amendment; only its file
anchors moved (see errata).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract; zero verbal briefing needed).
2. `tasks/ui-degree-opacity-ramp-kickoff.md` - Intent, Scope in/out,
   Invariants 1-9, the Frozen-locks table, and the Amendment 2026-10-09
   section (which SUPERSEDES the preconditions/sizing rows above it).
3. `tasks/ui-degree-opacity-ramp-shape.md` - "Decided contract" (Q1-Q5),
   Invariants I1-I9, and the Amendment 2026-10-09 section (which SUPERSEDES
   earlier anchor/placement rows).
4. This handoff's dated errata (E1-E3, at the bottom) are binding where the
   shape/kickoff are wrong against the tree or against T-R's landing.
5. T-R landing state: `tasks/ui-degree-opacity-ramp-tr-handoff.md` (what it
   moved, and why `illuminatePath` now lives in `main.js`).

## Dispatch precondition (blocking; all four, verified at session start)

- PC-1: T-R (`tasks/ui-degree-opacity-ramp-tr-handoff.md`) is LANDED
  APPROVED on `main`. Start rev `TRACK_BASE` = T-R's landing commit. Clean
  tree required; dirty tree on any touch-set file -> STOP.
- PC-2: ANOTHER red base is a STOP, not background noise (kickoff
  amendment, risk R10; erratum E4): run `python tools/testrun.py --distill
  -- -p ariadne-ui` at start - the failing-test-name set must equal
  EXACTLY `{test_inspector_panel_is_wider_and_collapsible}` (the inherited
  deferred-row-F1 red that rides through T-R's green landing, untouched by
  design) or be EMPTY if the operator has since disposed of F1. Anything
  else -> STOP and report. Record the canonical line + set verbatim in the
  journal as `BASELINE FAILING SET (inherited)`; every later gate compares
  against it by name.
- PC-3 (anchor-match rule, PC-2 of the kickoff, risk R9): before ANY
  mutation, every before-string below must match the current committed
  blobs byte-verbatim (line numbers are advisory; before-strings bind).
  All were byte-verified against `20e3fb8` on 2026-10-09; T-R does not edit
  any of them (its contract moves `illuminatePath`'s text into `main.js`
  unchanged). An operator `graph.js`/`hotspots.js`/`styles.css`/`main.js`
  edit between T-R's landing and this dispatch -> STOP.
- PC-4 (placement measurement): count `crates/ui/assets/graph.js` lines at
  the start commit (byte-safe: `python -c "print(sum(1 for _ in
  open(r'crates/ui/assets/graph.js','rb')))"`). <= 950 -> PRIMARY branch
  (full placement in `graph.js`; expected ~909 at T-R's landing);
  951-985 -> CONTINGENCY branch (tables + 3 helpers into `hotspots.js`
  instead, ~34-41 lines; `graph.js` gains only the inline property/compute
  lines; shared globals make the cross-file call legal - the
  `classifyDependency` precedent); > 985 -> STOP (the deferred heat/legend
  extraction would have to precede; do NOT raise any cap, do not split this
  handoff). `hop_fade_ramp.rs` file-ownership locks are authored against
  the branch MEASURED. Journal the measurement + branch taken.

## Serialization (hard; ledger row records it)

- NEVER parallel with any other `crates/ui` handoff on `main`: exactly one
  `crates/ui` landing commit in flight at a time. `ui-traversal-back-button`
  T1 landed APPROVED before registration (`9c0cd0c` + rider `5b4da58`,
  own TRACK_BASE `20e3fb8`) - the order constraint against this handoff is
  already satisfied; do not touch or re-open that track from here. Any
  FUTURE `crates/ui` landing (notably PLANNED `ui-extraction-card-clarity`
  T-A/T-B: shared `index.html`/`styles.css` and the `ui_test/main.rs` mod
  roster) serializes one landing at a time and re-anchors on its own start
  rev; if such a landing precedes this dispatch, PC-3's anchor-match rule
  governs (STOP on mismatch).
- The `mod` line here inserts between `mod help_center_readability;` and
  `mod hotspot_direction;` (alphabetical site); `mod traversal_back;`
  already exists at the roster tail (`main.rs:24` since the WIP/operator
  landing) - untouched, no collision.

## Objective and invariants (kickoff invariants 1-9 are binding; condensed)

Replace the uniform heavy fade with a graduated opacity ramp keyed to
undirected hop distance from `selectedNodeId`, down to a floor exactly equal
to today's values. The root/selected node stays opaque; justified/reachable
heat-tinted cards keep today's never-faded treatment.

- I1 Floor-exactness: faded with no finite hop renders exactly 0.12 (node)
  / 0.06 (edge) - table floors + CSS fallbacks (today's exact values).
- I2 Monotonicity: node table strictly decreasing from 0.85; edge table
  non-increasing from <= 0.30 (strictly under the 0.35
  `.edges-has-selection` dim - `styles.css:504-506`, untouched - so a faded
  edge can never outshine a kept one).
- I3 Distance independence: distance = undirected BFS hop count from
  `selectedNodeId` over the FULL `tabAllEdges`/`tabAllNodes` dataset;
  `computeHopDistances` reads NOTHING else (no `activeNodes`, `activeEdges`,
  `nodeCap`, `currentDegree`, `isAllCapped`, `seeds`, `isFiltering`). The
  degree/cap/seeds machinery decides WHICH elements exist; the ramp decides
  how dim an existing faded one is. The open degree-setting bug neither
  breaks nor is fixed by this (I8 below).
- I4 Faded-set equivalence: every `.faded` condition stays byte-identical
  at all 4 heat sites; the ramp only adds a property line INSIDE the
  existing `add('faded')` branches; exactly one `setProperty` per property
  name per file (graph.js, hotspots.js).
- I5 Reachable/verdict branches untouched (no new class/property there).
- I6 `illuminatePath` (NOW IN `main.js`, post-T-R) and any
  fallback-path fade stay flat 0.12/0.06: the 2-line `removeProperty`
  leak-guard rider clears stale inline ramp properties from the post-heat
  DOM.
- I7 Zero new hex; `themes.css`, `theme.js`, `tools/theme_token_baseline.json`
  byte-identical. Opacity decimals are not hex; the CSS `var(--x,
  fallback)` form is exempt from the token-declaration lock by the
  `declared_needing_tokens` carve-out in `theme_js_zero_hex.rs:152-158`
  (which explicitly names JS-set custom properties as the intended use).
- I8 Bug non-interference, both directions: no diff hunk lands inside
  `filterAndExpandSubgraph`, `stepDegree`, `toggleAllCapped`, `setNodeCap`;
  I3 keeps the ramp from inheriting the bug's inputs. Correction row
  `ui.traversal_view_degree_setting_bug` stays OPEN after this handoff -
  neither fixed nor referenced.
- I9 Roster frozen: `tools/ui_smoke.py` and `index.html` byte-identical;
  `ui_smoke: PASS` on the unchanged table (its ROUTE_ASSERTIONS at :61-70
  omit `/app/traversal.js` + `/app/provenance.js` - a pre-existing parity
  gap relayed to the next kickoff runner; keep PASS + byte-identical, do
  NOT fix the roster here).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ui-degree-opacity-ramp-t1-journal.md` (checkpoint journal; create
  it; always in-set)
- `crates/ui/assets/graph.js` (PRIMARY branch: tables + 3 helpers + hops
  compute + 4 property lines; CONTINGENCY branch: hops compute + property
  lines only)
- `crates/ui/assets/hotspots.js` (inbound twin: hops compute + 4 property
  lines; + tables/helpers under the CONTINGENCY branch)
- `crates/ui/assets/styles.css` (exactly the 2 opacity declarations)
- `crates/ui/assets/main.js` (exactly the 2 `removeProperty` rider lines,
  inside `illuminatePath` - erratum E1)
- `crates/ui/tests/ui_test/hop_fade_ramp.rs` (NEW lock module)
- `crates/ui/tests/ui_test/main.rs` (+1 line `mod hop_fade_ramp;` between
  `mod help_center_readability;` and `mod hotspot_direction;`)
- `crates/ui/tests/ui_test/static_html.rs` (exactly 2 lines amended at
  :155-156, net-zero lines - erratum E3)

You MUST NOT touch: `index.html`, `themes.css`, `theme.js`, `traversal.js`,
`api.js`, `preview.js`, `file-view.js`, `provenance.js`,
`tools/theme_token_baseline.json`, `tools/**` (incl. `ui_smoke.py`),
`crates/**/src/**` (assets ride `include_str!` - zero Rust edits),
`tests/**` outside `crates/ui/tests`, any other test file (incl.
`static_html_sections.rs` and `hotspot_direction.rs` - they pass unedited),
`Cargo.toml`/`Cargo.lock`. No `git stash`, no branch change, no rebase.

## What to build (binding contract literals)

### B1 styles.css (net 2 lines, in place; before-strings verified 2026-10-09)

```
    .node-group.faded {
      opacity: 0.12 !important;
```
becomes
```
    .node-group.faded {
      opacity: var(--fade-opacity, 0.12) !important;
```
and
```
    .edge-path.faded {
      opacity: 0.06 !important;
```
becomes
```
    .edge-path.faded {
      opacity: var(--edge-fade-opacity, 0.06) !important;
```
The grayscale/brightness filter and the transitions in the node rule stay
byte-identical (a hopped-down card stays the same KIND of thing).

### B2 Tables + helpers (PRIMARY: graph.js, inserted between
`computeFullDependencies`'s close (`      return reachableDependencies;` +
`    }`) and `    function selectNode(id, skipFilter = false, skipCrumb = false) {`;
CONTINGENCY: hotspots.js, same code, module ownership locks follow):

```
    // Hop-distance heat ramp (track ui-degree-opacity-ramp T1): per-element
    // inline custom properties replace the flat fade for hop-reachable
    // elements. Frozen tables: hop >= table length, Infinity, NaN or
    // undefined resolve to the LAST entry - exactly today's flat values.
    const HEAT_RAMP_NODE_OPACITIES = [0.85, 0.55, 0.36, 0.23, 0.15, 0.12];
    const HEAT_RAMP_EDGE_OPACITIES = [0.3, 0.28, 0.18, 0.12, 0.08, 0.06];

    // Undirected BFS hop distance from the selected node over the FULL
    // tab dataset (never the cap/degree-filtered active view - I3). The
    // adjacency-build idiom mirrors filterAndExpandSubgraph's (house
    // pattern); visited-set makes it cycle-safe. Returns Map<nodeId, hop>=1>;
    // the root is absent (it is never faded).
    function computeHopDistances(rootId) {
      const adjOut = {};
      const adjIn = {};
      tabAllNodes.forEach(n => { adjOut[n.id] = []; adjIn[n.id] = []; });
      tabAllEdges.forEach(e => {
        if(adjOut[e.source]) adjOut[e.source].push(e.target);
        if(adjIn[e.target]) adjIn[e.target].push(e.source);
      });
      const dist = new Map();
      const visited = new Set([rootId]);
      const queue = [{ id: rootId, dist: 0 }];
      while (queue.length > 0) {
        const cur = queue.shift();
        const next = cur.dist + 1;
        (adjOut[cur.id] || []).forEach(id => {
          if (!visited.has(id)) { visited.add(id); dist.set(id, next); queue.push({ id, dist: next }); }
        });
        (adjIn[cur.id] || []).forEach(id => {
          if (!visited.has(id)) { visited.add(id); dist.set(id, next); queue.push({ id, dist: next }); }
        });
      }
      return dist;
    }

    function hopFadeOpacity(hop) {
      const table = HEAT_RAMP_NODE_OPACITIES;
      if (!(hop >= 1) || hop >= table.length) return String(table[table.length - 1]);
      return String(table[Math.floor(hop) - 1]);
    }

    function edgeFadeOpacity(hop) {
      const table = HEAT_RAMP_EDGE_OPACITIES;
      if (!(hop >= 1) || hop >= table.length) return String(table[table.length - 1]);
      return String(table[Math.floor(hop) - 1]);
    }
```

Globals in page scope (classic scripts; the `classifyDependency`
graph.js->hotspots.js call is the precedent - the contingency branch runs it
the other direction). Every function <= ~20 lines.

### B3 Outbound heat pass adoption (graph.js `renderSvg`, verified anchors)

Compute hops once per pass. Before (anchor):
```
        const nodeLabel = id => (tabAllNodes.find(n => n.id === id) || { label: id }).label;

        nodesLayer.querySelectorAll('.node-group').forEach(el => {
```
After:
```
        const nodeLabel = id => (tabAllNodes.find(n => n.id === id) || { label: id }).label;
        const hops = computeHopDistances(selectedNodeId);

        nodesLayer.querySelectorAll('.node-group').forEach(el => {
```
Node faded branch - before (binding; must be the FIRST and ONLY occurrence
of this exact shape in graph.js post-T-R - verify uniqueness, else STOP):
```
          } else {
            el.classList.add('faded');
          }
```
After:
```
          } else {
            el.classList.add('faded');
            el.style.setProperty('--fade-opacity', hopFadeOpacity(hops.get(nid)));
          }
```
Edge faded branch - before (binding):
```
          if (!reachableDependencies.has(src) && src !== selectedNodeId) {
            el.classList.add('faded');
```
After:
```
          if (!reachableDependencies.has(src) && src !== selectedNodeId) {
            el.classList.add('faded');
            el.style.setProperty('--edge-fade-opacity', edgeFadeOpacity(
              Math.max(hops.get(src) ?? Infinity, hops.get(el.dataset.edgeTarget) ?? Infinity)));
```
Edges key to the FARTHER endpoint (`max(hop(src), hop(dst))`; missing
endpoint -> Infinity -> floor). Erratum E2: the shape snippet referenced
`dst` here, but `dst` is declared with `const` inside the sibling
`else if` block only - using it in this branch is a ReferenceError;
`el.dataset.edgeTarget` is the in-scope equivalent. Reachable/verdict
branches gain NOTHING (I5).

### B4 Inbound twin adoption (hotspots.js `renderInboundHeatView`)

Before (anchor):
```
      const nodeLabel = id => (tabAllNodes.find(n => n.id === id) || { label: id }).label;

      nodesLayer.querySelectorAll('.node-group').forEach(el => {
```
After:
```
      const nodeLabel = id => (tabAllNodes.find(n => n.id === id) || { label: id }).label;
      const hops = computeHopDistances(selectedNodeId);

      nodesLayer.querySelectorAll('.node-group').forEach(el => {
```
Node faded branch - before (binding; verify uniqueness in hotspots.js):
```
        } else {
          el.classList.add('faded');
        }
```
After:
```
        } else {
          el.classList.add('faded');
          el.style.setProperty('--fade-opacity', hopFadeOpacity(hops.get(nid)));
        }
```
Edge faded branch - before (binding; `src`/`dst` are both declared above the
condition in this loop, so the shape's `dst` usage is legal HERE):
```
        if (dst !== selectedNodeId && !payers.has(dst)) {
          el.classList.add('faded');
```
After:
```
        if (dst !== selectedNodeId && !payers.has(dst)) {
          el.classList.add('faded');
          el.style.setProperty('--edge-fade-opacity', edgeFadeOpacity(
            Math.max(hops.get(src) ?? Infinity, hops.get(dst) ?? Infinity)));
```

### B5 illuminatePath leak-guard rider (main.js - erratum E1: the shape
knew "graph.js:735-755"; T-R moved the function verbatim to main.js; the
rider's SEMANTICS are unchanged: flat 0.12/0.06, NOT a ramp adoption)

Node loop - before (binding, inside `illuminatePath` in main.js):
```
        } else {
          el.classList.add('faded');
        }
```
(verify this exact 8-space shape occurs exactly ONCE in main.js and is the
`pathNodes.has(nid)` branch's else - else STOP). After:
```
        } else {
          el.classList.add('faded');
          el.style.removeProperty('--fade-opacity');
        }
```
Edge loop - before:
```
        } else {
          el.classList.remove('highlighted');
          el.classList.add('faded');
        }
```
After:
```
        } else {
          el.classList.remove('highlighted');
          el.classList.add('faded');
          el.style.removeProperty('--edge-fade-opacity');
        }
```

### B6 Declared amendment (precedent: hotspot-track AMENDMENT A1)

`static_html.rs::test_ui_noticeable_fading_elements` :155-156 lock the
pre-ramp literals the `var(...)` form removes. Amend exactly the two
substrings, net-zero lines (erratum E3), before:
```
    assert!(html.contains(".node-group.faded") && html.contains("opacity: 0.12"));
    assert!(html.contains(".edge-path.faded") && html.contains("opacity: 0.06"));
```
after:
```
    assert!(html.contains(".node-group.faded") && html.contains("--fade-opacity, 0.12")); // ramp var form; 0.12 stays the floor (T1 2026-10-09)
    assert!(html.contains(".edge-path.faded") && html.contains("--edge-fade-opacity, 0.06")); // ramp var form; 0.06 stays the floor (T1 2026-10-09)
```
Everything else in that test survives byte-identical. Also verified
2026-10-09: `hotspot_direction.rs` (incl.
`test_graph_delegates_inbound_heat_before_outbound_block` and the
`renderInboundHeatView` payload-surface lock) passes UNEDITED - the ramp
adds no `graphData.` read, no `computeFullDependencies` reference, and does
not move the lead/delegate literals. Do not amend it.

### B7 New lock module `crates/ui/tests/ui_test/hop_fade_ramp.rs` (+ mod line)

House idiom: local copy of the `function_body` helper (the
`hotspot_direction.rs:19-31` 4-space-terminator idiom - modules do not
import across each other). Doc-comment style mirrors `hotspot_direction.rs`.
Five tests, exact expectations (PRIMARY-branch file ownership; swap
GRAPH_JS/HOTSPOTS_JS ownership literals under the CONTINGENCY branch):

- `test_floor_exactness` (I1): regex-extract both array literals from the
  owner file and assert last entries are exactly `0.12` / `0.06`;
  `ariadne_ui::STYLES_CSS` contains `var(--fade-opacity, 0.12)` and
  `var(--edge-fade-opacity, 0.06)`. No JS execution.
- `test_monotone_ramp_tables` (I2): parse the two array literals and assert
  FULL equality to the frozen values NODE [0.85,0.55,0.36,0.23,0.15,0.12]
  / EDGE [0.3,0.28,0.18,0.12,0.08,0.06] (the implementation IS the table);
  plus ordering guards: nodes strictly decreasing, nodes[0] == 0.85,
  edges non-increasing, edges[0] <= 0.30.
- `test_distance_independence` (I3): `function_body` of
  `computeHopDistances` (owner file) contains `tabAllEdges`; negative
  literals: must NOT contain `activeNodes`, `activeEdges`, `nodeCap`,
  `currentDegree`, `isAllCapped`, `seeds`, `isFiltering`.
- `test_faded_sites_and_property_counts` (I4/I5): the four condition
  literals still occur verbatim (`ariadne_ui::GRAPH_JS` contains
  `if (!reachableDependencies.has(src) && src !== selectedNodeId) {` and
  `} else if (reachableDependencies.has(nid)) {`;
  `ariadne_ui::HOTSPOTS_JS` contains `} else if (payers.has(nid)) {` and
  `if (dst !== selectedNodeId && !payers.has(dst)) {`);
  `matches("setProperty('--fade-opacity'")` count == 1 in graph.js AND == 1
  in hotspots.js; `setProperty('--edge-fade-opacity'` likewise 1 and 1 (a
  setProperty in a reachable branch would raise a count past 1).
- `test_illuminate_path_stays_flat` (I6): `function_body(MAIN_JS,
  "illuminatePath")` contains `removeProperty('--fade-opacity')` and
  `removeProperty('--edge-fade-opacity')`, contains no `setProperty(`, no
  `HEAT_RAMP`; and its condition literals `if (pathNodes.has(nid)) {` and
  `if (pathEdgeKeys.has(` still occur.

The ramp suite keeps to its own literals: it does NOT lock what T-R moved
(where `findIncludePath`/`illuminatePath` live is T-R's verbatim proof's
business, per the shape amendment).

## RGB protocol

1. Red (tests first, zero asset edits): write `hop_fade_ramp.rs` + the mod
   line + B6. Run `python tools/testrun.py --distill -- -p ariadne-ui`
   (timeout 600000 ms). The module MUST compile (constants exist;
   `function_body` on an absent function panics - that is the Red, not a
   compile failure). Expected failing set = BASELINE set (PC-2: `{F1 test}`
   or empty) UNION exactly these 6:
   the five `hop_fade_ramp` tests (first failures: STYLES_CSS var-form
   containment; table-literal extraction panic "must define
   HEAT_RAMP_NODE_OPACITIES"; `function_body` panic "the asset must define
   function computeHopDistances("; property-count 0 != 1; missing
   `removeProperty`) plus
   `static_html::test_ui_noticeable_fading_elements` (amended literal
   missing). Capture the set verbatim.
2. Green: B1, then B2, then B3, then B4, then B5 (in this order - each
   before-string verified immediately before its edit, PC-3 discipline).
   Re-run the scoped command: failing set == the PC-2 BASELINE set again
   by name identity (F1 rides through UNTOUCHED if still open - fixing it
   is the operator's disposition, not this track's); passed = start-baseline
   passed + 5; canonical line verbatim in the journal. NEVER weaken an
   assertion to pass.
3. Refactor: doc comments, honest names (`hops`, `HEAT_RAMP_*`,
   `edgeFadeOpacity` are contractual; no `tmp`/`res`/`data`, no
   single-letter locals outside verbatim anchors). Only if anything
   changed: re-run scoped suite (same expectation as Green). Then
   `python tools/hygiene_check.py`.

## Hygiene (review-enforced)

`python tools/hygiene_check.py`: NO NEW finding names at any step - the
run exits NONZERO carrying exactly the inherited `server.rs` pair
(deferred row F2, erratum E4; `exit 0` is unreachable track-wide). New JS functions <= ~20
lines each (R1 does not parse `.js`, the discipline holds anyway);
`hop_fade_ramp.rs` stays well under 1000; `static_html.rs` stays 999
(E3 - net-zero); PRIMARY branch lands `graph.js` ~909 + ~41 = ~950 (<= 990
lock, < 1000 R2; if your measured post-edit count exceeds 990, the branch
decision was wrong - STOP back to PC-4); CONTINGENCY lands `hotspots.js`
~673, `graph.js` ~913. `styles.css` grows 0 lines (2 declarations amended;
`.css` is R2-exempt - a known deferred ledger row, not this track's
decision).

## Frozen locks

| Lock | Proof |
|---|---|
| `index.html`, `themes.css`, `theme.js`, `tools/theme_token_baseline.json`, `tools/ui_smoke.py`, `traversal.js`, `api.js`, `preview.js`, `file-view.js`, `provenance.js`, all `crates/**/src/**`, `tests/fixtures/**`, `tools/**` byte-identical | reviewer `git diff TRACK_BASE..HEAD -- <path>` EMPTY |
| `main.js` diff is EXACTLY the 2 rider lines | reviewer `git show` stat + content (T-R moved the rest) |
| `styles.css` diff is EXACTLY the 2 opacity lines | reviewer `git show` |
| graph.js budget lock `<= 990` + R2 stay green UNEDITED (T-R's `test_graph_js_line_budget`) | scoped ui green; `static_html_sections.rs` diff-EMPTY |
| `hotspot_direction.rs`, `static_html_sections.rs` and every other test unedited | `git diff --name-only` filtered to `crates/ui/tests` == exactly `hop_fade_ramp.rs` + `main.rs` + `static_html.rs` |
| Zero added hex; token locks green (I7) | `python tools/theme_token_check.py --check-added --base <TRACK_BASE>` silent; `theme_js_zero_hex.rs` unedited and green (var-with-fallback exemption is in-file, :152-158) |
| No mojibake; pure-ASCII authored content | `python tools/encoding_guard.py --staged` exit 0 before each commit (`--all` reviewer hop) |
| Degree-bug territory untouched (I8); faded-set conditions byte-identical (I4) | reviewer per-commit `git show` oldest-first, hunk-header check + B7 I4 literals |
| Roster frozen (I9) | `ui_smoke: PASS` + `tools/ui_smoke.py`/`index.html` diff-EMPTY |

## Gates (exact commands; implementer hop; timeout 600000 ms; distilling
entry points ONLY; journal itemizes the count - declare overshoots as dated
D-rows per the M28 T0b law)

1. Start-baseline record (PC-2): `python tools/testrun.py --distill --
   -p ariadne-ui` (failing set == `{F1 test}` by name, or EMPTY - else
   STOP) + PC-3 anchor match + PC-4 line measurement (journal, not a gate).
2. Red capture: same command (exactly the 6 named failures).
3. Green capture: same command (failing set == the PC-2 baseline set by
   name identity (F1 rides through if still open); passed = start + 5;
   digest header + canonical line verbatim + capture path).
4. Refactor re-run: same command, ONLY if refactor changed files.
5. `python tools/hygiene_check.py` - NONZERO exit carrying ONLY the
   inherited `server.rs` pair (no new finding names - erratum E4).
6. `python tools/distill.py -- python tools/ui_smoke.py` - `ui_smoke: PASS`
   on the unchanged route table.
7. `python tools/theme_token_check.py --check-added --base <TRACK_BASE>` -
   silent.
8. `python tools/encoding_guard.py --staged` - exit 0 before committing.
9. Commit ONE landing commit:
   `python tools/commit_scoped.py --scope crates/ui/assets/graph.js --scope crates/ui/assets/hotspots.js --scope crates/ui/assets/styles.css --scope crates/ui/assets/main.js --scope crates/ui/tests/ui_test/hop_fade_ramp.rs --scope crates/ui/tests/ui_test/main.rs --scope crates/ui/tests/ui_test/static_html.rs --scope tasks/ui-degree-opacity-ramp-t1-journal.md -m "feat(ui): graduated hop-distance opacity ramp on faded graph nodes and edges"`
   (never `git add .`; if your permissions deny `commit_scoped.py`/`git`,
   STOP and report - commit duty routes to the orchestrator per the standing
   allowlist errata row).

Typical count 8 (9 if refactor touched files) - within the ~10 ceiling.

Journal manual-smoke lines (hotspot-track precedent; check off ONLY after
actually running the built UI - otherwise write "manual smoke pending
operator" and flag it):
- outbound heat shows the hop gradient with the floor far field;
- inbound ('in' lens) heat shows the same ramp;
- clicking an inspector collateral link after a heat render keeps off-path
  cards at flat 0.12 (leak-guard proof);
- degree/cap toolbar changes leave each visible faded node's opacity keyed
  to the selection, not to the filter.

## Sizing verification (planner, re-verified against the tree 2026-10-09)

- Scope: 1 crate dir (`crates/ui`); 4 assets + 1 new test + 2 test-file
  edits. Within the law.
- Produced lines: `hop_fade_ramp.rs` ~150; B2 code ~37; graph.js inline
  ~+4 (+37 under PRIMARY = ~+41); hotspots ~+6 (PRIMARY) / ~+41
  (CONTINGENCY); styles 2; main.js 2; main.rs 1; static_html.rs 2;
  journal ~25. Total ~230 (PRIMARY) vs the ~1200 ceiling and the kickoff
  amendment's 200-300 - confirmed.
- Gates: 8-9 invocations vs the ~10 ceiling - within.
- Post-state margins (PRIMARY): `graph.js` ~950 (<= 990 lock, < 1000 R2),
  `hotspots.js` 638, `main.js` ~748.

## Dated errata (2026-10-09, planner authoring-time adjudication; per the
ui-extraction-card-clarity precedent the handoff is binding where the
shape/kickoff are wrong)

- E1: kickoff scope-in says the leak-guard rider is in `graph.js`
  (`illuminatePath :735-755`). After T-R, `illuminatePath` lives in
  `main.js` (shape amendment 2026-10-09 already re-anchored it). `main.js`
  is therefore ADDED to the touch-set, with its diff capped at exactly the
  2 rider lines.
- E2: the shape's outbound edge application snippet keys with `dst`
  (`Math.max(hops.get(src) ?? Infinity, hops.get(dst) ?? Infinity)`) at a
  site where `dst` is declared only in the sibling `else if` block
  (`const dst` at graph.js:423, block-scoped) - in that branch it is a
  ReferenceError. Binding equivalent: `hops.get(el.dataset.edgeTarget)`
  (B3). The inbound twin declares both before the condition - unchanged
  (B4).
- E3: the shape's declared amendment says "a 1-line rationale comment
  each" for `static_html.rs:155-156`. That file is 999 lines, AT the R2
  1000 wall (its own comment at :753-757 records the ceiling) - two added
  comment lines would break R2. Rationale moves into same-line trailing
  comments (net-zero lines), per the T-R E3 precedent.
- E4 (added at registration 2026-10-09, concurrent-landing adjudication):
  `ui-traversal-back-button` T1 landed APPROVED (`9c0cd0c` + `5b4da58`)
  while this handoff was being authored, registering deferred rows
  F1/F2/F3a. Binding consequences here: T-R's green landing carries the
  inherited F1 test red (`test_inspector_panel_is_wider_and_collapsible`),
  so PC-2's baseline is that name-set (or EMPTY once the operator disposes
  of F1) - a DIFFERENT red is the STOP, not any red; hygiene runs will
  exit NONZERO on the inherited `server.rs` pair (deferred row F2) for the
  whole track lifetime - hygiene acceptance for T1 is therefore "no NEW
  finding names" (graph.js/hotspots.js/main.js/styles.css/test files clean)
  rather than exit 0. Gate 7 ("silent") holds UNCHANGED for T1: none of
  T1's added lines contains a `#`-run candidate, so the F3a entity-misread
  class cannot fire (it cost T-R exactly one sanctioned phantom line).

## Discipline clauses

- UTF-8: never `Set-Content`/`Out-File`/`>`/`Get-Content -replace`;
  Read/Edit/Write tools only; authored content pure ASCII (AGENTS.md
  File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, do not fix. The
  degree bug is the prime trap here (I8).
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`, raw workspace census)
  are a reviewer blocker in an implementer session - route through the
  distilling entry points (`docs/specs/tool-output-distillation.md`).
- After EACH gate, append a 1-3 line checkpoint to
  `tasks/ui-degree-opacity-ramp-t1-journal.md`: step + commit hash, PC-2/3/4
  results verbatim (start rev, PC-2 baseline set, anchor outcomes, measured line
  count + branch), any deviation.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ui-degree-opacity-ramp-t1-handoff.md --scope crates/ui --cmd "python tools/testrun.py --distill -- -p ariadne-ui"`.

## Reviewer hop (fresh session; adjudicate via `review-gate`; re-run FRESH)

1. Scoped ui suite fresh: failing set == the PC-2 BASELINE set by name
   identity (F1 rides through untouched, if still open); the 5
   `hop_fade_ramp` tests green UNEDITED; `test_ui_noticeable_fading_elements`
   amended ONLY at the two locked substrings.
2. `git diff --name-only TRACK_BASE..HEAD` set-equality against the
   touch-set minus the journal; the frozen-locks table diff-EMPTY;
   `main.js` diff == exactly 2 rider lines; `styles.css` diff == exactly
   2 lines; `main.rs` diff == exactly 1 mod line; `static_html.rs` net-zero
   (still 999 lines).
3. I8 hunk check per commit oldest-first: no hunk inside
   `filterAndExpandSubgraph`/`stepDegree`/`toggleAllCapped`/`setNodeCap`;
   I4 condition literals byte-identical in the diff context.
4. `python tools/hygiene_check.py` with NO new finding names beyond the
   inherited `server.rs` pair (E4);
   `python tools/distill.py -- python tools/encoding_guard.py --all` exit 0;
   `theme_token_check --check-added --base <landing-parent>` silent (T1
   adds no `#`-run candidates - E4);
   `ui_smoke: PASS` (distill route) on the byte-identical table.
5. Workspace census `python tools/testrun.py --distill -- --workspace`:
   passed = live baseline passed + 5 and failing set == the PC-2 baseline
   reds by name identity, reconciled from the live tree (the
   `--baseline` FILE form is the standing 5-encounter errata - use
   `--distill`; NEVER reconcile against a guessed number).
6. Journal manual-smoke lines present (or honestly deferred to operator);
   G1-G6 map to kickoff acceptance.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. PC-2 start-green canonical line,
   PC-4 measurement + branch, Red set (6 names) + Green canonical line
   (verbatim) + capture path. 3. Final graph.js/hotspots.js line counts.
4. Deviations + gate-count itemization (declare overshoots as D-rows).
5. Out-of-scope observations (report-only; anything seen about the degree
   bug - do not touch it). Never paste raw tool output: cite command +
   canonical line + exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the Touch-set above, which
therefore lists every file this handoff may legitimately change.
