# Plan: ui-degree-opacity-ramp (graph canvas graduated fade)

Architect artifacts (binding): `tasks/ui-degree-opacity-ramp-kickoff.md` +
`tasks/ui-degree-opacity-ramp-shape.md`. Quick-mode track: ONE handoff, one
module dir (`crates/ui`), unregistered post-M28 track (no M-number; register
the ledger row at milestone-lifecycle kickoff).

## Decided contract

- Distance: undirected BFS hop distance from `selectedNodeId` over the FULL
  `tabAllEdges` dataset (both adjacencies, pattern `graph.js:946-952`
  working tree). Direction-following would be infinite for every faded node
  (they are outside the out-closure); undirected is the only metric with
  signal and is immune to degree/cap/seeds by construction.
- Ramp mechanism: inline CSS custom properties read with today's values as
  fallbacks: `.node-group.faded { opacity: var(--fade-opacity, 0.12)
  !important; }` (styles.css:437-441, filter/transition stay uniform) and
  `.edge-path.faded { opacity: var(--edge-fade-opacity, 0.06) !important; }`
  (:494-496). No hex added; theme layer untouched.
- Frozen oracle tables (implementation IS the table):
  NODE `[0.85, 0.55, 0.36, 0.23, 0.15, 0.12]` (0.85 at hop 1, ~0.65/hop,
  floor == today's 0.12 at hop >= 6 / unreachable);
  EDGE `[0.3, 0.28, 0.18, 0.12, 0.08, 0.06]` (min(0.30, node/2); 0.30 cap
  keeps faded edges below the 0.35 `.edges-has-selection` dim; floor 0.06).
  JS: `computeHopDistances(rootId)`, `hopFadeOpacity(hop)`,
  `edgeFadeOpacity(hop)`.
- Edges: adopted - a faded edge keys `--edge-fade-opacity` to its FARTHER
  endpoint (`max(hop(src), hop(dst))`, missing = floor). Fade conditions
  byte-identical; property lines added inside existing add-faded branches.
- Call sites: ramp adopted at BOTH heat passes - outbound
  `graph.js:387-437` and inbound lens twin `hotspots.js:337-386`.
  `illuminatePath` (:735-755) stays flat, with a 2-line `removeProperty`
  leak-guard rider; `traversal.js` and the degree-setting bug
  (`ui.traversal_view_degree_setting_bug`) are out of scope and structurally
  non-interfering (ramp never reads filter state; no hunk in the degree
  code path).
- Reachable/justified heat cards: unchanged treatment (never faded, no
  properties); root stays fully opaque.

## Touch set (complete)

`graph.js`, `hotspots.js`, `styles.css`, NEW
`crates/ui/tests/ui_test/hop_fade_ramp.rs`, `main.rs` (+1 mod), declared
2-line amendment of `static_html.rs::test_ui_noticeable_fading_elements`
(opacity literals -> var-form substrings, rationale comments). Everything
else byte-identical, incl. `themes.css`, `theme.js`, baseline, `ui_smoke.py`
roster, `index.html`.

## Preconditions (blocking)

1. Collateral-traversal-view track (current dirty tree) lands first; scoped
   ui + hygiene green on a clean tree (shared-file hunks cannot be separated
   by `commit_scoped`).
2. graph.js under the 1000 R2 wall at that landing; placement rule decides
   whether the ~34 lines of helpers land in `graph.js` or `hotspots.js`
   (shape "Placement decision rule"); STOP branch if no headroom - the
   deferred heat/legend extraction precedes, do not raise the 998 lock.

## Gates

G1 scoped ui green (new locks + amendment):
`python tools/testrun.py --distill -- -p ariadne-ui`;
G2 `python tools/distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS`
(roster byte-identical); G3 `theme_token_check.py --check-added --base
<start>` silent; G4 `hygiene_check.py` + `encoding_guard --all` exit 0;
G5 empty diffs on all lock paths; G6 journal manual-smoke lines (outbound
ramp, inbound lens, flat illuminatePath, floor far field, degree/cap
independence). Reviewer hop re-runs unedited + commit-vs-touch-set.
