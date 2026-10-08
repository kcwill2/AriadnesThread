# T1 Handoff — ui-hotspot-canvas-direction: hotspot row click follows the metric's natural direction (inbound heat view for burden/fan-in)

Unregistered post-M28 change track (no M-number; ledger row `ui-hotspot-canvas-direction`
records the track). ONE handoff for the whole change; do not split, do not extend.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/hotspot-canvas-direction-shape.md` — invariants I1-I8, exact JS/HTML
   contracts, degradation contract, risks R1-R9 (authoritative; this handoff
   passes its contracts through verbatim).
3. `tasks/hotspot-canvas-direction-kickoff.md` — intent, frozen-lock table,
   acceptance gate (a)-(e).
4. Anchor files with start-of-tree line refs (verified against `c00576d`):
   - `crates/ui/assets/hotspots.js` (:2 one-line `dirSelect` onchange;
     :232-236 `renderHotspots` no-collateral fallback burden->fanin;
     :256 row `onclick="selectHotspotRow(...)"`; :267-269 current bare
     `selectHotspotRow` body; :391 extraction-card `onclick="selectHotspotRow(...)"`)
   - `crates/ui/assets/graph.js` (:327 outbound heat `if (currentTab ===
     'include' && selectedNodeId) {` through :390 `updateHeatLegend(...)` and
     :392 `restoreStandardLegend();`; :454-467 `computeFullDependencies`;
     :469 `selectNode(id, skipFilter = false, skipCrumb = false)`; file is
     990 lines)
   - `crates/ui/assets/index.html` (:510 the `<li>Clicking a row selects the
     file and renders its heat map live beside the open list.</li>` anchor)
   - `crates/ui/tests/ui_test/main.rs` (mod list :1-22; the new mod line sorts
     alphabetically after `mod help_center_readability;` (:11), before
     `mod justification_builders;` (:12))
   - Model file to mirror for test style: `crates/ui/tests/ui_test/extraction_card_playbook.rs`
     (source-level shape locks via its local `function_body` helper — copy the
     helper into the new module; do not import across modules).
   - Field names/literals: `graphData.file_collateral`, `graphData.file_justification`,
     `tabAllEdges`, helpers `getHeatColor`, `classifyDependency`,
     `styleDependencyCard`, `verdictBadgeColor`, `verdictBadgeText`,
     `classifyEdgeVerdict`, `appendVerdictTip`, `HEAT_LEGEND`,
     `verdictCardLegendRows`, `escapeHtml` — all existing, all UNMODIFIED
     globals (main.js / graph.js).

## Objective and invariants

Clicking a hotspot row (or extraction card) auto-sets the toolbar Direction to
the metric's natural direction before selecting the file, and for inbound
metrics the canvas paints an inbound heat view (consumers badged with what THEY
pay) instead of fading every consumer under the outbound-only heat.

Invariants preserved (falsifiable defs in the shape; locks below):
- I1 mapping: burden/fanin -> `'in'`; fanout/closure -> `'out'`; cycle -> `null`.
- I2 auto-set happens ONLY on hotspot row / extraction-card clicks; the
  `selectNode(id)` call literal survives inside `selectHotspotRow` unchanged.
- I3 lens is selection PROVENANCE (`hotspotHeatLens === 'in'`), never
  `dirSelect.value`; consumed as the FIRST statements of `selectNode`;
  manual Direction changes stay byte-identical to today (their lens is null).
- I4 inbound payment lookup is consumer-first:
  `((graphData.file_collateral || {})[nid] || {})[selectedNodeId]` — the
  consumer's payment, never the root-side map.
- I5 the outbound heat block stays byte-identical (leading branch + `} else if`
  rewrite only).
- I6 no new payload surface: inside `renderInboundHeatView` the only
  `graphData.<field>` reads are `file_collateral` and `file_justification`.
- I7 no new asset: the 8-script roster, export order, routes unchanged.
- I8 help copy documents the direction follow AND names the cycle exception;
  the "Hotspots Triage" tour step stays untouched in `main.js`.

Decisions (do not relitigate; shapes in the shape doc):
- D1 selection-driven only — the metric `<select>` changing NEVER touches the
  toolbar Direction.
- D2 inbound heat mirrors the heat view keyed to provenance, with its own
  legend copy ("N payer(s) - M symbols paid"; rows reused from `HEAT_LEGEND` /
  `verdictCardLegendRows`); reusing `updateHeatLegend` verbatim is forbidden
  (it prints the outbound story backwards).
- D3 cycle keeps the current toolbar Direction (bidirectional rank; no flip,
  no new bidirectional heat view).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/hotspot-canvas-direction-journal.md` (checkpoint journal; always in-set;
  create it at start with the start HEAD rev + baseline line)
- `crates/ui/assets/hotspots.js` (lens globals, onchange reset,
  `hotspotMetricDirection`, new `selectHotspotRow`, `computeFullConsumers`,
  `renderInboundHeatView`; 465 -> ~575 lines)
- `crates/ui/assets/graph.js` (2 statements at top of `selectNode`; leading
  delegate branch in the `renderSvg` heat section; 990 -> <=996, HARD CAP +8)
- `crates/ui/assets/index.html` (one help `<li>` after the :510 anchor item,
  copy contract below)
- `crates/ui/tests/ui_test/hotspot_direction.rs` (new; the 8 locks, ~200 lines)
- `crates/ui/tests/ui_test/main.rs` (exactly one `mod hotspot_direction;` line)
- `crates/ui/tests/ui_test/static_html_sections.rs` (SCOPED EXCEPTION — AMENDMENT
  A1 below, dated 2026-10-08: exactly ONE line, the :215 line-count assert
  raised `<= 990` -> `<= 998`; nothing else in the file)

You MUST NOT touch: `crates/ui/assets/main.js`, `crates/ui/assets/styles.css`,
`crates/ui/assets/themes.css`, `crates/ui/assets/theme.js`,
`crates/ui/src/**` (including `static_html.rs` — it sits at 999/1000, any new
lock there is an R2 blocking failure; that is why the new module exists),
`tools/**`, EVERY pre-existing test file (sole exception: the AMENDMENT A1
one-line edit to `static_html_sections.rs` :215), `tests/fixtures/**`, goldens,
`Intent.md`/`Plan.md`/`Spec.md`, `tasks/milestone_ledger.md`,
`tasks/ledger_archive.md`; NO new asset file under any name (no `heat.js`).
No `git stash`, no branch change.

## AMENDMENT A1 (dated 2026-10-08, mid-hop; orchestrator-authorized, Option A)

Provable self-contradiction in the original sizing, planner erratum (recorded
in the ledger track row the same day): the pre-existing frozen lock
`crates/ui/tests/ui_test/static_html_sections.rs:215` —
`assert!(ariadne_ui::GRAPH_JS.lines().count() <= 990, "graph.js must stay
within the spine-plan line budget (cap 990)")` — lives inside the
unrelatedly-named `test_extraction_impact_preview_overlay` (:153) and caps
graph.js at 990 = its exact `c00576d` baseline size, zero headroom, while the
mandated verbatim contract shape needs net +6 (measured: graph.js 990 -> 996).

- REJECTED alternatives (do not pursue): statement-fusing to net-zero and any
  weakening of the I3 lock (Option B); reshaping the contract to fit 990.
- AUTHORIZED edit (exhaustive for the whole hop): in
  `crates/ui/tests/ui_test/static_html_sections.rs`, exactly ONE line — :215
  `ariadne_ui::GRAPH_JS.lines().count() <= 990` ->
  `ariadne_ui::GRAPH_JS.lines().count() <= 998` (this handoff's own +8 HARD
  CAP). Nothing else in that file, including the :216 assert-message literal
  "(cap 990)", which stays stale BY AUTHORIZATION; its correction belongs to
  the deferred ledger row for the heat/legend extraction follow-up (with the
  finding that the lock should move to a dedicated, findable test or be
  dropped in favor of the R2 hygiene gate).
- Completeness: this is the suite's ONLY machine lock on graph.js line count
  (planner-verified 2026-10-08: the only `GRAPH_JS.lines()` occurrence in
  `crates/ui/tests/` is :215; the "R2 cap 990" string at
  `d27_node_provenance.rs:185` is assert-message text — that assert checks
  `MutationObserver` content, not line count).
- Oracle rewording: "all pre-existing `ariadne-ui` tests pass UNEDITED" reads
  "…UNEDITED except the A1 one-line cap raise". The reviewer's commit-vs-
  touch-set check treats that single line as the ONLY permitted pre-existing
  test edit; any other line in that file, or any other pre-existing test
  file, remains a blocker.
- Expected lines, updated: RED `209 passed / 8 failed / 0 ignored /
  3 suites`; GREEN `217 passed / 0 failed / 0 ignored / 3 suites` (unchanged).
  The suites column may read `2` instead of `3` in any run where a panicking
  test aborts a test binary — documented pre-existing testrun artifact
  (implementer journal D2; lineage: M27 T2 counting errata family, ledger).
  The passed/failed columns must still reconcile to 209+8, and suites must
  read `3` on GREEN. The observed intermediate `216 passed / 1 failed`
  (sole failure: the cap-990 assert) is the PREDICTED pre-A1 state; applying
  the A1 line closes it — it is not a blocker and not a license to reshape
  graph.js.
- Journal-path ruling: the accepted name is `tasks/hotspot-canvas-direction-journal.md`
  (shape module map); where it deviates from the handoff-spec default
  `<handoff-basename>-journal.md`, the shape name WINS. No rename.

## What to build (exact contracts — from the shape, verbatim)

In `hotspots.js`, module-private lens state declared at the top of the file,
above the `dirSelect` handler (script load order api, graph, file-view,
hotspots, preview, provenance, main, theme guarantees evaluation before any
renderSvg/selectNode call):

```js
let hotspotHeatLens = null;    // null | 'in' - provenance of current selection
let pendingHeatLens = null;    // set by hotspot clicks, consumed by selectNode
```

Replace the current one-line onchange (hotspots.js:2) with:

```js
document.getElementById('dirSelect').onchange = () => {
  hotspotHeatLens = null;
  pendingHeatLens = null;
  filterAndExpandSubgraph();
};
```

New pure mapping (I1):

```js
function hotspotMetricDirection(metric) {
  if (metric === 'burden' || metric === 'fanin') return 'in';
  if (metric === 'fanout' || metric === 'closure') return 'out';
  return null; // cycle: bidirectional rank - toolbar keeps the user's pick
}
```

Replace the 3-line `selectHotspotRow` (hotspots.js:267-269); the lens travels
via `pendingHeatLens`, NOT via arguments — the literal `selectNode(id)` must
survive exactly (a pre-existing `static_html.rs` lock greps for it):

```js
function selectHotspotRow(id) {
  const dir = hotspotMetricDirection(document.getElementById('hotspotMetric').value);
  if (dir) document.getElementById('dirSelect').value = dir;
  pendingHeatLens = dir === 'in' ? 'in' : null;
  selectNode(id);
}
```

In `graph.js` `selectNode` (:469): two statements added at the very top,
signature UNCHANGED (net <= +3 lines with the comment):

```js
hotspotHeatLens = pendingHeatLens;
pendingHeatLens = null;
```

In `graph.js` `renderSvg` heat section (:327): leading branch only (net <= +3
lines); everything from `const reachableDependencies` (:328) through
`updateHeatLegend(...)` (:390) and the trailing `else { restoreStandardLegend(); }`
(:392) stays BYTE-IDENTICAL (I5):

```js
if (currentTab === 'include' && hotspotHeatLens === 'in' && selectedNodeId) {
  renderInboundHeatView(nodesLayer, edgesLayer);
} else if (currentTab === 'include' && selectedNodeId) {
  ...existing block, byte-identical (I5)...
} else {
  restoreStandardLegend();
}
```

In `hotspots.js`: `computeFullConsumers(nodeId)` — reverse BFS mirroring
`computeFullDependencies` (graph.js:454), same full-dataset scan semantics over
`tabAllEdges`, direction flipped (`e.target === curr`, enqueue `e.source`).
Per the shape contract verbatim.

In `hotspots.js`: `renderInboundHeatView(nodesLayer, edgesLayer)` — the inbound
mirror of the graph.js:327-393 heat pass, < 100 lines, reusing the unmodified
globals listed in Binding inputs. Follow the shape's contract comment block
exactly: `payers = computeFullConsumers(selectedNodeId)`;
`paid(nid) = ((graphData.file_collateral || {})[nid] || {})[selectedNodeId] || 0`;
`just(nid) = ((graphData.file_justification || {})[nid] || {})[selectedNodeId]`;
selected node `heat-root` + `[ROOT]` badge (same idiom); payer cards tiered via
`classifyDependency`, `styleDependencyCard(el, tier, paid, maxCollateral)`,
badge `tier === 'neutral' ? '+' + paid + ' col' : verdictBadgeText(tier, just)`,
tip via `appendVerdictTip(el, consumerLabel, rootLabel, verdict, just)` (reads
"<consumer> uses k of m symbols from <selected>"); all other nodes `faded`;
edges faded when `dst !== selectedNodeId && !payers.has(dst)`; incoming
`dst === selectedNodeId && src !== dst` edges verdict-tricolored
(`edge-justified` / `edge-partial` / `edge-unused` + `arrow-unused` marker +
tip) from `classifyEdgeVerdict(just(src))`; own legend copy — legendViewLabel
'Heat Map', summary `payers.size + ' payer(s) - ' + total + ' symbols paid'`,
green 'Payer fully needs it', amber partly/no-data row, red 'Payer uses none
of it - include can go', same verdict-tier gating idiom as `updateHeatLegend`,
card spine rows via `verdictCardLegendRows`, heat tier rows via `HEAT_LEGEND`.
Token-only colors — no hex literals anywhere.

Degradation contract (shape): absent justification entry -> `neutral` tier,
`+N col` badge, no spine, no tip; true-zero payer shows `+0 col` (parity with
the outbound view; invent no data-vs-zero special case). No new fetch, no new
throw sites; all map lookups use the `|| {}` guarded idiom.

Copy contract — `index.html`, ONE new `<li>` immediately after the :510 item,
sentence-per-line, ASCII, HTML entities for arrows:

"Clicking a row follows the metric's natural direction: Collateral burden and
Fan-in switch the canvas to Consumers (&larr;) and badge each paying file with
what it pays; Fan-out and Closure burden switch to Dependencies (&rarr;);
Cycle participation keeps your current Direction."

## Required tests — `crates/ui/tests/ui_test/hotspot_direction.rs` (new)

House idiom: source-level shape locks reading the asset text, local
`function_body` helper copied from `extraction_card_playbook.rs`. One falsifier
per invariant; I7 rides the pre-existing theme_contract + export_order roster
locks. EXACT test names (the acceptance line counts them):

1. `test_hotspot_metric_direction_mapping` — I1 mapping values.
2. `test_select_hotspot_row_follows_metric_direction` — I2: body contains the
   `hotspotMetricDirection` call, a `dirSelect` assignment, the
   `pendingHeatLens` assignment, AND the literal `selectNode(id)` with no
   added arguments.
3. `test_select_node_consumes_pending_lens` — I3: `selectNode` body starts
   with `hotspotHeatLens = pendingHeatLens;` then `pendingHeatLens = null;`.
4. `test_direction_change_clears_lens_flags` — I3: the `dirSelect` onchange
   handler nulls BOTH flags before calling `filterAndExpandSubgraph`.
5. `test_graph_delegates_inbound_heat_before_outbound_block` — I5: the leading
   `if (currentTab === 'include' && hotspotHeatLens === 'in' && selectedNodeId)`
   branch delegating to `renderInboundHeatView` precedes, and the shape's
   literal `} else if (currentTab === 'include' && selectedNodeId) {` is
   present.
6. `test_consumer_heat_view_reads_payer_side` — I4: `renderInboundHeatView`
   body contains `((graphData.file_collateral || {})[nid] || {})[selectedNodeId]`
   and does NOT contain `computeFullDependencies`.
7. `test_inbound_heat_reads_no_new_payload_fields` — I6: regex scan of the
   function body; allowed `graphData.` field reads ⊆ {`file_collateral`,
   `file_justification`}.
8. `test_help_copy_documents_direction_follow` — I8: the new `index.html` help
   `<li>` contains `natural direction` and names the cycle exception.

Plus the one `mod hotspot_direction;` line in `main.rs` at the alphabetical
site (:11/:12 above). Do NOT edit any pre-existing test file — the whole
pre-existing `ariadne-ui` suite is the frozen equivalence oracle for "manual
direction changes keep working byte-identically" (sole exception: the
AMENDMENT A1 one-line `static_html_sections.rs` :215 cap raise).

## RGB protocol (in this file order)

1. RED: write `hotspot_direction.rs` (all 8 tests) + the `main.rs` mod line
   FIRST, before any JS/HTML change. Tests must COMPILE and fail on string
   assertions, never on compile errors (no todo!()/unimplemented skeletons).
   Run gate 1; capture the failing list — expect exactly the 8 new tests
   failing, e.g. `test_hotspot_metric_direction_mapping` with an assert
   message citing the missing `function hotspotMetricDirection` in
   hotspots.js, and the canonical line `209 passed / 8 failed / 0 ignored /
   3 suites`.
2. GREEN: implement hotspots.js -> graph.js -> index.html in that order;
   re-run gate 1 until `217 passed / 0 failed / 0 ignored / 3 suites` (apply
   the AMENDMENT A1 one-line :215 raise as part of this step — without it the
   run fails exactly once on the cap-990 assert: `216 passed / 1 failed`, the
   predicted intermediate state). NEVER
   weaken an assertion to pass; the graph.js +8 cap is an assertion, not a
   suggestion — if the delegate pattern cannot fit +8, STOP and report.
3. REFACTOR: dedupe against the mirrored outbound block honestly (reuse the
   named helpers; do not copy-paste the 67-line block), intention-revealing
   names, doc comments; re-run gate 1.

## Gates (exact commands, timeout 600000 ms each; scoped only)

Budget: <= 10 distilling-entry-point invocations for the whole hop. Itemize
per-gate invocation counts in the journal; any overshoot must be declared as a
dated D-row (M28 T0b law — undeclared overshoot is the deviation, harmless
declared overshoot is acceptable).

1. `python tools/testrun.py --distill -p ariadne-ui` — canonical summary line
   and digest header must appear verbatim in the report; RED `209 passed /
   8 failed / 0 ignored / 3 suites` first, then GREEN `217 passed / 0 failed /
   0 ignored / 3 suites` (baseline `209 passed / 0 failed / 0 ignored /
   3 suites` re-verified by the planner at `c00576d`, 2026-10-08). Suites
   caveat (AMENDMENT A1): the suites column may read `2` while any test
   binary is panic-aborted (documented pre-existing testrun artifact);
   passed/failed must still reconcile to 209+8; suites must read `3` at
   GREEN.
2. `python tools/hygiene_check.py` — exit 0; no new R1/R2 rows; graph.js
   still < 1000 (start rev: 990; your commits add <= 8 lines).
3. `python tools/theme_token_check.py --check-added --base <start-rev>` —
   silent (start rev = HEAD you recorded in the journal at hop start).
4. `python tools/encoding_guard.py --staged` — silent, exit 0, before
   committing.
5. `python tools/distill.py -- git diff HEAD -- crates/cli` — empty capture
   (DP-1: CLI byte format untouched).
6. Commit with `python tools/commit_scoped.py` and PER-PATH --scope ONLY
   (exact command under Discipline clauses), never `git add .` / `git add -A`.
   Re-run gate 4 per commit if you split commits.

## Frozen locks (each with its proof; reviewer enforces)

| Lock | Proof |
| --- | --- |
| `dirSelect` default stays `out` (`<option value="out" selected>` untouched) | gate 1 (static_html `test_ui_dependency_tree_direction_preserving`) |
| `selectHotspotRow` routes through the literal `selectNode(id)` | gate 1 (`assert_hotspot_burden_and_list_rules` passes UNEDITED) + required test 2 |
| 8-script asset roster + export order unchanged; no `heat.js` | gate 1 (theme_contract.rs, export_order.rs UNEDITED) + reviewer-hop `ui_smoke` route table |
| CLI byte format (DP-1) untouched | gate 5 -> empty capture |
| Every pre-existing `ariadne-ui` test passes UNEDITED (equivalence oracle) — AMENDMENT A1 carve-out: except the one-line `static_html_sections.rs` :215 cap raise 990 -> 998 | gate 1 GREEN line + reviewer per-commit `git show` shows no pre-existing test file in any diff except that single A1 line |
| No mojibake / non-UTF-8 | gate 4 |
| No new hex literals in added lines | gate 3 |
| Files < 1000 lines, functions < 100 lines | gate 2 |
| graph.js delta HARD CAP +8 (990 -> <=998) | journal records `git diff --numstat` for graph.js; reviewer checks per-commit `git show` |

## Hygiene and refactor targets (named ceilings at risk)

- `graph.js` 990/1000 (R2): the +8 cap is binding; the ~110 new lines live in
  hotspots.js. Heat/legend module extraction is a NAMED follow-up (R1), NOT
  this hop — it drags `server.rs`/`export.rs`/`tools/ui_smoke.py` rosters in.
- `static_html.rs` 999/1000 (R2): stay out; new locks live in the new module.
- `renderInboundHeatView` < 100 lines (R1, mirror of the 67-line block);
  new .rs module ~200 lines; no source line near 2000 chars.
- R3 naming: no tmp/res/foo/bar/test1, no single-letter locals.

## Discipline clauses

- Sequencing rider (hard): the boot-overlay step-bar track LANDED at `c00576d`
  (touch set `main.js`/`styles.css`/`static_html.rs` — now disjoint from this
  hop; verified by the planner 2026-10-08). If at hop start you find NEW
  staged hunks in `main.js`, `styles.css`, or `static_html.rs`, STOP and
  report — never commit on top of another track's staged track. Stage ONLY via
  the scoped form below — the shared worktree carries ~40 untracked M28
  artifacts plus modified `tasks/milestone_ledger.md`, `tasks/ledger_archive.md`,
  `.kilo/plans/1790697359283-*.md`, `tools/bench_baseline.json` owned by other
  tracks; `git add .` would sweep them all into your landing commit:
  `python tools/commit_scoped.py --scope crates/ui/assets/hotspots.js --scope crates/ui/assets/graph.js --scope crates/ui/assets/index.html --scope crates/ui/tests/ui_test/hotspot_direction.rs --scope crates/ui/tests/ui_test/main.rs --scope crates/ui/tests/ui_test/static_html_sections.rs --scope tasks/hotspot-canvas-direction-journal.md -m "feat(ui): hotspot row click follows metric direction; inbound payer heat view"`
  (the `static_html_sections.rs` scope carries ONLY the AMENDMENT A1 single
  line :215 990 -> 998; run `python tools/distill.py -- git diff --stat --
  crates/ui/tests/ui_test/static_html_sections.rs` before staging and confirm
  it reads `1 insertion(+), 1 deletion(-)` — anything more, STOP)
- UTF-8 (AGENTS.md): never Set-Content/Out-File/>/Get-Content-replace;
  Write/Edit tools only; authored content pure ASCII (`&larr;`/`&rarr;`
  entities in HTML; `->`, `<-`, `-` not arrows/em-dashes in JS/RS text).
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`, `testrun --baseline`) in
  an implementer session are a reviewer blocker; run scoped work through the
  distilling entry points only (`docs/specs/tool-output-distillation.md`).
- R4 is ACCEPTED, do not "fix": all renders are synchronous within one click
  frame; record it if you touch the area.
- After EACH per-module Green (hotspots.js, graph.js, index.html+tests),
  append a 1-3 line checkpoint to the journal: module + commit hash, any
  deviation, any lock/contract finding, and the running gate-invocation count.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed — successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/hotspot-canvas-direction-t1-handoff.md --scope crates/ui --cmd "python tools/testrun.py --distill -p ariadne-ui"`.
- Journal MUST end with the manual smoke line (acceptance (e)): clicking a
  burden row shows the toolbar reading "Consumers (<-)" with consumer cards
  badged with their payments, not faded.

## Acceptance (machine-checkable)

Implementer (all from the Gates section, on the landing commit):
1. Gate 1 GREEN: `217 passed / 0 failed / 0 ignored / 3 suites`, exit 0
   (suites MUST read 3 here — the `2 suites` panic-abort artifact is only
   tolerated in failing runs, AMENDMENT A1), pre-existing tests unedited
   except the A1 one-line `static_html_sections.rs` :215 raise.
2. Gate 2 exit 0, no new R1/R2.
3. Gate 3 silent.
4. Gate 4 exit 0.
5. Gate 5 empty capture.
6. Journal contains the manual smoke line + gate-invocation itemization.

Reviewer hop (workspace-wide — NOT implementer gates): acceptance adjudicated
via the `review-gate` skill; re-run gates 1-5 fresh, plus
`python tools/ui_smoke.py` (boot + route checks clean on the unchanged
roster), per-commit `git show` oldest-first vs the touch-set above (commit
vs touch-set enforcement; no pre-existing test file appears in any diff
except the single AMENDMENT A1 line in `static_html_sections.rs` — verify
that diff shows exactly :215 `<= 990` -> `<= 998` and nothing else),
graph.js numstat <= +8, and the workspace census `python tools/testrun.py
--distill` form (the literal `--baseline` errata stands; expect ~1221 passed
= 1213 @ 816d09d + riders + 8 new, suites may have moved — reconcile from
the live tree, don't anchor blindly).

## Report back (final message)

1. Commit hash + `git show --stat`. 2. RED evidence (the 8 failing test names
+ the canonical RED line). 3. GREEN evidence (distill digest header +
canonical line + capture path). 4. graph.js numstat delta. 5. Any deviation +
reason (declared D-rows). 6. Out-of-scope observations (report-only).
Never paste raw tool output: cite command + canonical line + exit code.
