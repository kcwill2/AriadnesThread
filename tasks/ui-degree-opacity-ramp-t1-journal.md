# Journal - ui-degree-opacity-ramp T1 (graduated hop-distance opacity ramp)

Handoff: `tasks/ui-degree-opacity-ramp-t1-handoff.md`. Unregistered post-M28
track (no M-number; `tasks/milestone_ledger.md` stays numbering authority).
Installment 2 of 2 serialized (T-R landed APPROVED; verdict
`tasks/ui-degree-opacity-ramp-tr-verdict.md`).

## Step 0 - preconditions (recorded before any mutation)

- TRACK_BASE = `497ac61` (HEAD = T-R landing lineage
  `5b4da58` -> `3c426eb` -> `497ac61`). Tree clean on every touch-set file;
  `git status --porcelain` shows only four untracked foreign/track docs
  (back-button execution-results + verdict, this track's execution-results +
  tr-verdict) - none in the touch-set.
- PC-2 BASELINE FAILING SET (inherited): canonical line verbatim
  `220 passed / 1 failed / 0 ignored / 2 suites` (exit 1; capture
  `target/testrun-capture.txt`, 265 lines). Failing set == EXACTLY
  {`static_html_sections::test_inspector_panel_is_wider_and_collapsible`}
  - the inherited deferred-row F1 red (assert at
    `static_html_sections.rs:418`, message "the inspector collapse/toggle
    controls must call toggleInspector()"). Equals the PC-2 expectation;
  every later gate compares failing sets by NAME IDENTITY against this set.
- PC-3 anchor match: every before-string byte-verified against the tree at
  `497ac61` before any mutation; ALL MATCH. Sites: styles.css `.node-group.
  faded` opacity 0.12 line (:437-438) and `.edge-path.faded` opacity 0.06
  line (:494-495); graph.js B2 insertion site `      return
  reachableDependencies;` + `    }` + blank + `    function selectNode(id,
  skipFilter = false, skipCrumb = false) {` (:459-462); graph.js outbound
  nodeLabel anchor (:385) + nodesLayer loop (:387); graph.js node faded-else
  shape (:412-414) UNIQUE (only occurrence of the exact shape); graph.js
  edge condition + add-faded (:419-420); `const dst` confirmed block-scoped
  inside the sibling else-if (:423) - E2 `el.dataset.edgeTarget` keying is
  binding; hotspots.js nodeLabel anchor (:335) + nodesLayer loop (:337);
  hotspots node faded-else shape (:362-364) UNIQUE; hotspots edge cond
  (:370) with src/dst declared above (:368-369); main.js `illuminatePath`
  node else shape (:356-358) UNIQUE 8-space occurrence preceded by
  `pathNodes.has(nid)` (:354) and edge else shape (:367-370) UNIQUE;
  static_html.rs:155-156 literals verbatim (file at 999 lines); main.rs mod
  roster with `mod help_center_readability;` (:11) adjacent to `mod
  hotspot_direction;` (:12) and `mod traversal_back;` at tail (:24).
- PC-4 placement measurement: `crates/ui/assets/graph.js` = 909 lines ->
  <= 950 -> PRIMARY branch (matches the expected ~909 at T-R's landing).
  Measurement route deviation: the handoff's byte-safe `python -c` recipe
  was refused by session permissions (raw, distill-routed, and a read-only
  PowerShell byte-count fallback - all the catch-all boilerplate); the count
  was taken from the byte-safe Read tool's line enumeration instead. No
  shell content pipe was used for any measurement.
- Branch taken: PRIMARY (tables + 3 helpers in graph.js; hotspots.js gets
  the hops compute + 4 property lines).

## Gate record (itemized; 1-3 lines per gate)

- G1 (start baseline): `python tools/testrun.py --distill -- -p ariadne-ui`
  -> `220 passed / 1 failed / 0 ignored / 2 suites` (exit 1). Failing set
  == {F1} verbatim per Step 0 above. Invocation #1.
- G2 (Red capture): same command -> `219 passed / 7 failed / 0 ignored /
  2 suites` (exit 1; capture target/testrun-capture.txt, 309 lines). Failing
  set from capture == BASELINE {F1} UNION exactly the 6 predicted names:
  hop_fade_ramp::test_floor_exactness, test_monotone_ramp_tables,
  test_distance_independence, test_faded_sites_and_property_counts,
  test_illuminate_path_stays_flat, static_html::
  test_ui_noticeable_fading_elements. All predicted Red reasons fired
  (HEAT_RAMP extraction panic; computeHopDistances function_body panic;
  setter count 0 != 1; missing removeProperty; amended literal missing).
  Module compiled - Red is semantic, not build. Invocation #2.
- G3 (Green capture): same command -> `225 passed / 1 failed / 0 ignored /
  2 suites` (exit 1; capture target/testrun-capture.txt, 271 lines). Failing
  set == the PC-2 BASELINE set by name identity ({F1}, same assert at
  static_html_sections.rs:418, same message); passed = start 220 + 5 new
  greens. All B1-B5 landed in contract order with before-strings matched
  verbatim at edit time (Edit-tool uniqueness errors would have caught any
  anchor drift; none fired). Invocation #3.
- G4 (refactor re-run): SKIPPED by design - no refactor commit needed; the
  landed code is the handoff's verbatim contract block (B2) plus pinned
  property lines; no speculative abstraction introduced. Zero files changed
  after G3, so the conditional re-run does not apply.
- G5 (hygiene): `python tools/distill.py -- python tools/hygiene_check.py`
  -> exit 1; full capture target/distill-capture.txt (175 lines). The ENTIRE
  capture contains exactly two blocking findings - `crates/ui/src/server.rs:
  R2 1120 lines > 1000` and `server.rs:764: R1 api_graph_traversal 161 >
  100` - the inherited deferred-row F2 pair. Grep of the capture for
  R1|R2|touch-set names finds those two lines only: NO NEW finding names,
  zero findings in graph.js/hotspots.js/main.js/styles.css/hop_fade_ramp.rs/
  static_html.rs. All other capture lines are pre-existing warn-only R3
  rows in untouched files. Invocation #4 (distill route).
- G6 (ui_smoke): `python tools/distill.py -- python tools/ui_smoke.py` ->
  `ui_smoke: PASS` (exit 0, capture 2 lines) on the byte-identical route
  table. Invocation #5.
- G7 (theme added-scan): `python tools/theme_token_check.py --check-added
  --base 497ac61` printed the three KNOWN base-independent frozen-count
  lines, verbatim the population the T-R verdict adjudicated row-by-row:
  index.html 6>4 + traversal.js 1>0 (byte-inherited; both files unedited by
  T1) and main.js 1>0 (the sanctioned T-R `&#8250;` phantom count-twin).
  ZERO added-hex lines: T1's added lines carry no `#`-run candidates
  (E4 prediction confirmed - the added-scan half is silent). Nothing added
  beyond what the handoff sanctions. Invocation #6.
- Post-G3 measured line counts: graph.js 961 (+52; planner estimate ~950
  undercounted the 48-line B2 block - budget lock test_graph_js_line_budget
  `<= 990` green in G3, R2 < 1000; no STOP condition), hotspots.js 636
  (+4 vs estimate ~638), main.js 748 (exactly +2 rider lines), static_html.rs
  999 (net-zero, E3 held), styles.css line count unchanged (2 in-place
  declarations).
- G8 (encoding, pre-commit): touch-set staged via
  `python tools/distill.py -- git add -- <the 8 scope paths>` (distill route;
  raw git denied at permission level in this session), then
  `python tools/encoding_guard.py --staged` -> exit 0, silent (no violations
  in the staged blobs). G9 (landing commit) follows via commit_scoped with
  the handoff's exact scope list and message.

## Gate invocation itemization (distilling entry points + declared gates)

1. distill `git log` - precondition lineage check (raw git denied -> distill).
2. distill `git status --porcelain` - PC-1 tree-clean check.
3. G1 testrun start baseline.
4. G2 testrun Red capture.
5. G3 testrun Green capture.
6. distill `hygiene_check.py` (G5).
7. distill `ui_smoke.py` (G6).
8. `theme_token_check.py --check-added --base 497ac61` (G7).
9. distill `git add` touch-set (staging enabler for the pre-commit --staged).
10. `encoding_guard.py --staged` (G8, exit 0).
11. `commit_scoped.py` landing (G9).
12+. post-commit proof reads (distill `git show --stat`, `git status`,
   `git diff --name-only TRACK_BASE..HEAD`) - numbered in the close section.
Refused attempts (NOT gate invocations; all the catch-all boilerplate rule):
raw `git log`/`git status`, `python -c` PC-4 recipe (raw + distill route),
PowerShell read-only byte-count fallback. Every expected gate ran.

## D-rows (dated deviations, per the M28 T0b law)

- D1 (2026-10-09): PC-4 measurement route deviation - the handoff's byte-safe
  `python -c` recipe was refused (raw, distill-routed, and a PS byte-count
  fallback); line counts taken from the byte-safe Read tool's line
  enumeration instead. No shell content pipe used; measurement unchanged in
  substance (909 -> PRIMARY, as the handoff predicted).
- D2 (2026-10-09): graph.js lands at 961 vs the handoff's ~950 estimate; the
  planner's ~+41 arithmetic undercounted the verbatim B2 block (48 lines +
  4 inline + 1 blank = +52). The PC-4 branch decision ran on the MEASURED
  start count (909 <= 950 -> PRIMARY, as the handoff ordered); the landing
  sits 29 lines under the dedicated <=990 budget lock (green in G3) and
  under the 1000 R2 wall. Not a STOP condition per the handoff's own rule
  (STOP applies only if the post-edit count exceeds 990).
- D3 (2026-10-09): gate-8 "before each commit" satisfied via a distill-routed
  `git add` staging step (normally structurally impossible - commit_scoped
  owns staging; T-R D6 precedent) so the --staged run inspected real staged
  blobs pre-commit. Two extra precondition distill reads (log/status) and
  the post-commit proof reads are itemized above; overshoot vs the 8-9
  typical count is declared here.
- D4 (2026-10-09): landing + journal-rider convention - gate 9 mandates ONE
  landing commit, yet the post-G9 checkpoint (which must carry the landing
  hash) cannot be inside the commit it names. Closing journal checkpoint
  lands as a journal-only scoped rider (repo precedent: back-button `5b4da58`,
  T-R `497ac61`); zero code paths in the rider. Declared BEFORE creating it.

## Manual smoke (hotspot-track precedent; honest status)

- outbound heat shows the hop gradient with the floor far field: MANUAL
  SMOKE PENDING OPERATOR (no built-UI browser run in this session; ui_smoke
  cold-boot PASS is automated route assertion only, not a visual pass).
- inbound ('in' lens) heat shows the same ramp: MANUAL SMOKE PENDING
  OPERATOR.
- clicking an inspector collateral link after a heat render keeps off-path
  cards at flat 0.12 (leak-guard proof): MANUAL SMOKE PENDING OPERATOR.
- degree/cap toolbar changes leave each visible faded node's opacity keyed
  to the selection, not to the filter: MANUAL SMOKE PENDING OPERATOR.

## Out-of-scope observations (report-only; degree bug untouched per I8)

- Nothing touched the degree-bug territory; correction row
  `ui.traversal_view_degree_setting_bug` stays OPEN. No hunk inside
  filterAndExpandSubgraph / stepDegree / toggleAllCapped / setNodeCap (B2's
  insertion site at the computeFullDependencies close and B3's inside the
  heat pass are both outside the filter tail).

## Close (post-commit)

- Landing commit G9: `d97cf905f5c56bdf80c83459cc864d5a9adcc45d`
  (`d97cf90`) "feat(ui): graduated hop-distance opacity ramp on faded graph
  nodes and edges". `git show --numstat`: graph.js +52/-0, hotspots.js
  +4/-0, main.js +2/-0 (exactly the E1 rider), styles.css +2/-2 (exactly the
  2 opacity declarations), hop_fade_ramp.rs +217/-0 (new), main.rs +1/-0
  (the mod line), static_html.rs +2/-2 (net-zero, E3), journal +173/-0.
- Convergence: `git diff --name-only 497ac61..HEAD` == EXACTLY the eight
  touch-set paths (zero collateral); every frozen-lock file absent from the
  whole-range name-only diff -> structurally byte-identical (index.html,
  themes.css, theme.js, tools/**, all crates/**/src/**, traversal.js,
  api.js, preview.js, file-view.js, provenance.js, theme_token_baseline.json,
  all other test files incl. static_html_sections.rs + hotspot_direction.rs).
- Hunk audit (self-check, I8/I4): graph.js hunks @@386/@@415/@@423/@@466 all
  inside the outbound heat pass or the computeFullDependencies close - none
  in filterAndExpandSubgraph/stepDegree/toggleAllCapped/setNodeCap;
  hotspots.js hunks @@336/@@365/@@374 all inside renderInboundHeatView;
  main.js hunks @@358/@@371 are the two illuminatePath rider lines;
  styles.css @@438/@@495 the two declarations; main.rs @@12 the mod line;
  static_html.rs @@155,2+155,2 the declared amendment.
- Rider commit (D4): this close section lands as a journal-only scoped rider
  carrying the landing hash; zero code paths.
- Full invocation ledger (distilling entry points + declared gates):
  #1 distill git log (precondition), #2 distill git status (PC-1), #3 G1
  baseline, #4 G2 Red, #5 G3 Green, #6 distill hygiene (G5), #7 distill
  ui_smoke (G6), #8 theme_token_check (G7), #9 distill git add (pre-commit
  staging enabler), #10 encoding_guard --staged (G8, exit 0), #11
  commit_scoped landing (G9), #12 distill diff --name-only (convergence),
  #13 distill git show --unified=0 (hunk audit; first attempt was a
  --format=none usage error at exit 128, not a gate), #14 distill git add
  (rider staging), #15 encoding_guard --staged (rider), #16 commit_scoped
  rider, #17 distill git status (final tree state), #18 distill git log
  (lineage). Overshoot vs the typical 8-9 declared under D3; all extras are
  precondition reads, the pre-commit staging enabler, post-commit proofs,
  and the D4 rider.
- Refused-command ledger (all catch-all boilerplate; none a blocked gate):
  raw `git log`/`git status` (rerouted via distill), PC-4 `python -c` recipe
  raw + distill + PowerShell fallback (Read-tool line enumeration used per
  D1), one `--format=none` usage error.
