# Plan: ui-degree-opacity-ramp (graph canvas graduated fade) - AMENDED 2026-10-09

Binding artifacts: `tasks/ui-degree-opacity-ramp-kickoff.md` +
`tasks/ui-degree-opacity-ramp-shape.md` (both carry dated Amendment
2026-10-09 rows that supersede their originals where they conflict).
Unregistered post-M28 track (no M-number until the kickoff runner registers
the ledger row).

## Amendment headline (HEAD 20e3fb8 "fixed traversal view", tree clean)

- Traversal track landed by the operator, but graph.js is 1051 lines at
  HEAD: statically RED on the 998 budget assert
  (`static_html_sections.rs:214-217`, stale "(cap 990)" message intact)
  AND the 1000-line R2 hygiene wall. Nothing lands green on this base.
- All ramp anchor regions are byte-identical to the shaping-time reads
  (heat passes, both .faded rules, filter tail, illuminatePath); the
  distance metric survives unchanged; the degree-setting bug is still
  OPEN and stays out of scope (neither broken nor fixed by the ramp).
- DECISION: Option B. TWO serialized handoffs on main, never parallel:

### T-R - budget relief (lands first; discharges the deferred ledger row)
1. Verbatim-move graph.js:445-502 (classifyDependency, appendVerdictSpine,
   styleDependencyCard, verdictBadgeColor) + graph.js:671-755 (recordCrumb,
   renderThreadBar, jumpToCrumb, findIncludePath, illuminatePath) into
   main.js after restoreStandardLegend (~:230, where the verdict/legend
   helpers already live; roster-zero move).
2. Relocate the line-budget assert out of test_extraction_impact_preview_overlay
   into dedicated `test_graph_js_line_budget()` (cap 990, honest message).
3. Declared amendments (exactly 3): static_html.rs:238 def-lock -> MAIN_JS;
   static_html.rs:367 slice terminator -> "function clearInspector(";
   the :214-217 relocation. All other tests unedited (verbatim-proof oracle:
   moved ranges byte-equal to `git show 20e3fb8:crates/ui/assets/graph.js`).
4. Gates: scoped ui green, hygiene exit 0, ui_smoke PASS (route table
   byte-identical), encoding exit 0, theme-check silent. ~360 lines,
   ~6-7 gates. Session-start known-red (budget test + R2) declared, turned
   green = acceptance.

### T1 - the ramp (starts on T-R's green landing commit; contract unchanged)
- Undirected BFS hops from `selectedNodeId` over full `tabAllEdges`
  (`computeHopDistances`); frozen tables NODE [0.85,0.55,0.36,0.23,0.15,
  0.12] / EDGE [0.3,0.28,0.18,0.12,0.08,0.06]; inline `--fade-opacity`/
  `--edge-fade-opacity` with 0.12/0.06 CSS fallbacks; adopted at BOTH heat
  passes (graph.js:374-443 outbound faded sites :412-414 and :419-420;
  hotspots.js:362-364 and :370-371 inbound twin); edges keyed to farther
  endpoint; illuminatePath (now in main.js) stays flat + 2 removeProperty
  rider lines; reachables/root unchanged.
- Full placement in graph.js (post-T-R ~896 + ~40 < 990 lock, < 1000 R2);
  hotspots.js fallback if measured > 950 (re-based rule).
- New `crates/ui/tests/ui_test/hop_fade_ramp.rs` (+1 mod line at main.rs
  :11/:12 boundary) + declared 2-line amendment of
  static_html.rs::test_ui_noticeable_fading_elements opacity literals.
- Gates G1-G6 per kickoff (scoped ui; ui_smoke via distill; theme-check;
  hygiene+encoding; byte-identical locks incl. tools/ui_smoke.py; manual
  smoke lines). ~200-300 lines, ~7 gates.

## Start rules (both implementers)
Clean tree at the designated start commit; anchor-match rule before every
mutation (line numbers advisory, before-strings binding); STOP on any red
base (T1) or undeclared red; scoped commits via commit_scoped; distilling
entry points for every gate.

## Relay (out of both scopes, for the ledger)
tools/ui_smoke.py ROUTE_ASSERTIONS omit /app/traversal.js (served at
server.rs:136, loaded at index.html:594) and /app/provenance.js - roster
parity unenforced (pre-existing class); next kickoff runner owns it.
