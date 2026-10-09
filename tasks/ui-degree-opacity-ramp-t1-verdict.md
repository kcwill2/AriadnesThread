# Verdict - ui-degree-opacity-ramp T1 (graduated hop-distance opacity ramp)

Reviewer: review-gate skill, fresh session, all gates re-run independently.
Commits adjudicated oldest-first: `d97cf905f5c56bdf80c83459cc864d5a9adcc45d`
(landing), `7547dea3b03f8f266d8c5e52cf565029cacd33c9` (journal rider).
Lineage parent `497ac61` (T-R docs rider); TRACK_BASE `5b4da58`.

## VERDICT: APPROVED

Zero blockers. This closes the track's development phase (installment 2 of
2). Remaining work is ledger closeout + the operator's four manual-smoke
lines + the standing operator dispositions F1/F2 (Finding 6).

## Ground truth (step 1)

`python tools/checkpoint_guard.py status` at review start: HEAD `7547dea`,
branch main, dirty none, untracked exactly the four known docs (two foreign
back-button + this track's execution-results + tr-verdict) - none in the
touch-set. Worktree census unchanged from track baseline. Re-checked after
review writes: same.

## Per-commit review (steps 3-4, oldest-first)

### d97cf90 - landing

- `git show --stat`: graph.js +52/-0, hotspots.js +4/-0, main.js +2/-0,
  styles.css +2/-2, hop_fade_ramp.rs +217/-0 (new), main.rs +1/-0,
  static_html.rs +2/-2, journal +173. 8 paths, all inside the touch-set.
- **B1 styles.css**: exactly the 2 opacity declarations become
  `var(--fade-opacity, 0.12)` / `var(--edge-fade-opacity, 0.06)`; the
  grayscale/brightness filter and transitions stay byte-identical; blob
  line count unchanged 2105 base->HEAD (own `git cat-file` counts).
- **B2 verbatim contract block** inserted exactly between
  `computeFullDependencies`'s close and `selectNode`: comment wording, both
  frozen tables `[0.85, 0.55, 0.36, 0.23, 0.15, 0.12]` /
  `[0.3, 0.28, 0.18, 0.12, 0.08, 0.06]`, `computeHopDistances` (reads only
  `tabAllNodes`/`tabAllEdges`; visited-set cycle-safe), `hopFadeOpacity` /
  `edgeFadeOpacity` (Infinity/NaN/undefined/>=len -> last entry) match the
  handoff block line-for-line. PRIMARY branch honored (tables/helpers in
  graph.js).
- **B3 outbound adoption**: `const hops = computeHopDistances(selectedNodeId)`
  once per pass after the `nodeLabel` anchor; node property line inside the
  existing `add('faded')` else; edge line keyed to the FARTHER endpoint via
  `Math.max(hops.get(src) ?? Infinity, hops.get(el.dataset.edgeTarget) ??
  Infinity)` - E2's `el.dataset.edgeTarget` substitution implemented exactly
  as binding (the shape's `dst` would be a ReferenceError there).
  Reachable/verdict branches gained nothing (I5).
- **B4 inbound twin** in hotspots.js `renderInboundHeatView`: same 4 lines;
  edge uses in-scope `src`/`dst` as the contract specifies.
- **B5 main.js rider**: EXACTLY 2 `removeProperty` lines inside
  `illuminatePath` (node else + edge else), no other main.js change - flat
  0.12/0.06 behavior preserved with the leak guard (I6, E1 touch-set
  honored, diff capped at 2 lines as the locks table demands).
- **B6 static_html.rs**: exactly the 2 substrings at :155-156 amended to the
  var-form assertions with same-line rationale (E3 net-zero); rest of the
  test byte-identical; file still 999 lines (blob count).
- **B7 hop_fade_ramp.rs**: all five tests present with the handoff's exact
  expectations; house `function_body` idiom copied locally; mod line lands
  alphabetically between `help_center_readability` and `hotspot_direction`
  (`mod traversal_back;` tail untouched).

### 7547dea - journal rider

Journal-only, +41/-0, zero code paths. D4 declared BEFORE creation; repo
precedent (`5b4da58`, `497ac61`). Accepted (Disposition D4).

## Commit-vs-touch-set (step 4)

`git diff --name-only 497ac61..HEAD` == EXACTLY the eight touch-set paths
(graph.js, hotspots.js, styles.css, main.js, hop_fade_ramp.rs, main.rs,
static_html.rs, journal). Zero collateral. Every frozen-lock file (index.html,
themes.css, theme.js, traversal.js, api.js, preview.js, file-view.js,
provenance.js, `tools/**` incl. ui_smoke.py + theme_token_baseline.json, all
`crates/**/src/**`, every other test file incl. static_html_sections.rs and
hotspot_direction.rs) is absent from the whole-range name-only diff ->
structurally byte-identical. `crates/ui/tests` subset == exactly the 3
declared files.

## Independent verification (steps 2, 5-8 - fresh runs, this session)

- **Scoped ui fresh (testrun, distill form):** canonical line verbatim
  `225 passed / 1 failed / 0 ignored / 2 suites` - exactly the handoff's
  expectation; failing set == {`static_html_sections::
  test_inspector_panel_is_wider_and_collapsible`} by name identity (assert
  at :418, message unchanged). 220 start + 5 new greens = 225.
- **F1 inherited-red unchanged vs 497ac61 (structural proof):**
  `static_html_sections.rs` and `index.html` (the failing input) are both
  absent from the 497ac61..HEAD name-only diff -> byte-identical -> the red
  fires identically at the start rev. Not attributable to T1.
- **Workspace census (reviewer-hop item 5):**
  `1257 passed / 1 failed / 12 ignored / 70 suites` - passed == T-R's live
  census 1252 + 5 (the five hop_fade_ramp tests), failing set == {F1} by
  name identity. Reconciled from live tree; never against a guessed number.
- **Tools suites (CI form):** `python -m unittest discover -s tools -p
  "*_tests.py"` -> `Ran 219 tests in 29.851s` / `OK`.
- **Blob line counts (own byte-level `git cat-file` via the distiller):**
  graph.js 909 (at 497ac61) -> 961 (HEAD): INSIDE the dedicated <=990 budget
  lock (29 lines of headroom; `test_graph_js_line_budget` green in the
  scoped run) AND under the 1000 R2 wall (hygiene capture carries no
  graph.js finding). hotspots.js 636, main.js 748, static_html.rs 999
  (E3 net-zero held), styles.css 2105 = base. All blobs LF.
- **Hygiene fresh:** exit 1; full 175-line capture grep for R1|R2 returns
  EXACTLY the inherited `server.rs` pair (`R2 1120 lines > 1000` + `server.rs:764
  R1 api_graph_traversal 161 > 100`, capture lines 1-2). ZERO new finding
  names; nothing in graph.js/hotspots.js/main.js/styles.css/hop_fade_ramp.rs/
  static_html.rs. All other lines pre-existing warn-only R3 (E4 acceptance
  "no NEW finding names" MET; exit 0 unreachable track-wide by design).
- **encoding_guard:** `python tools/distill.py -- python tools/encoding_guard.py
  --all` -> exit 0, capture 0 lines. Pure-ASCII lock held.
- **theme_token_check --check-added --base 497ac61:** output == exactly the
  three known base-INDEPENDENT frozen-count lines the T-R verdict
  adjudicated row-by-row (`index.html 6>4` + `traversal.js 1>0` byte-inherited
  - both files absent from the whole-range diff; `main.js 1>0` - the
  sanctioned T-R `&#8250;` phantom count-twin; main.js added lines are the
  two ASCII rider lines, no hex). The base-RELATIVE added-scan half is
  SILENT - zero new hex from T1, E4's prediction confirmed.
- **ui_smoke:** `python tools/distill.py -- python tools/ui_smoke.py` ->
  `boot+route checks clean (cold fixture, attempt 1, 0.4s)` /
  `ui_smoke: PASS` (exit 0). Route table + index.html byte-identical (I9).
- **I8 hunk audit:** T1 graph.js hunks land at @@383/@@411/@@418/@@459
  (+54 max -> ends ~:517); the degree-bug territory sits at graph.js
  :805 `filterAndExpandSubgraph`, :935 `stepDegree`, :942 `toggleAllCapped`,
  :949 `setNodeCap` (function-definition lines measured on the clean HEAD
  checkout). No hunk anywhere near them; hotspots hunks all inside
  `renderInboundHeatView`; correction row `ui.traversal_view_degree_setting_bug`
  stays OPEN, neither fixed nor referenced.
- **I4 faded-set equivalence:** all four condition literals survive verbatim
  in the diff context and are additionally locked in
  `test_faded_sites_and_property_counts`; setter counts 1+1 per property per
  file locked.
- **Test-quality adjudication (machine-checkable acceptance):** the tables
  are locked NUMERICALLY (regex-parsed to f64, full-array equality against
  the frozen contract, ordering + floor + 0.30-cap guards) - not string
  presence. BFS distance BEHAVIOR is locked at source level only (reads
  `tabAllEdges`, all 7 foreign inputs negatively excluded) plus verbatim
  adoption of the plan-pinned block (which this review verified line-for-line
  against the handoff). This matches the track's declared policy (shape:272-
  276: assets suite never executes JS; visuals ride the manual-smoke lines),
  so it IS the handoff's machine-checkable acceptance - the residual
  behavioral risk transfers to the pending operator smoke (Finding 1), not a
  gap against the contract.
- **No retained dual representation:** the flat CSS literals are replaced
  (var-form with exact-value fallback), the static_html literals were amended
  not duplicated, and `illuminatePath`'s flat fade is the contractually
  mandated flat path with the leak guard, not a shadow copy of the ramp.
- **Report evidence (step 8):** journal is digest-shaped throughout -
  canonical lines verbatim, capture paths + line counts, exit codes, zero
  raw dumps; all implementer gate commands via distilling entry points.
  Compliant with `docs/specs/tool-output-distillation.md`.

## Numbered findings

1. **[minor - operator relay, NOT a blocker]** All four journal manual-smoke
   lines are honestly declared "MANUAL SMOKE PENDING OPERATOR" (outbound
   gradient + floor far field; inbound ramp; post-heat collateral click keeps
   off-path cards flat 0.12 (leak-guard); degree/cap toolbar changes leave
   faded opacity keyed to selection not filter). `ui_smoke` PASS is route
   assertion only. The handoff sanctions this deferral explicitly; the visual
   acceptance transfers to the operator. Run before or alongside the ledger
   flip.
2. **[minor - planning-copy relay]** D2 upheld: planner's ~950 estimate
   undercounted the 48-line verbatim B2 block (landed 961, +52 not +41).
   Accepted on merit (see disposition), but the ~950 figure is stale in the
   planning docs - the next `graph.js`-touching handoff must re-anchor
   against the measured 961 (29 lines of headroom to the 990 lock).
3. **[info - config relay, now 7th encounter]** Literal
   `python tools/testrun.py --baseline` remains unrunnable:
   `usage error: missing value for --baseline` (FILE-valued flag; no
   baseline file in repo). Substituted with the workspace census per the
   standing errata (T-R Finding 1). Ledger owner's call.
4. **[info - deferred-rows relay, unchanged]** Theme frozen-count drift pair
   (`index.html 6>4` + `traversal.js 1>0`) + `main.js` count-twin still want
   a `theme_token_baseline.json` reaffirmation by its owner; deferred row F3a
   (`&#\d+;` entity misread in `tools/theme_token_check.py`) stays OPEN - T1
   added zero candidates to its population (added-scan silent).
5. **[info - closeout relay]** Foreign untracked docs still in the tree:
   `tasks/ui-traversal-back-button-execution-results.md` +
   `tasks/ui-traversal-back-button-t1-verdict.md` (back-button closeout's) -
   deliberately not committed by this review.
6. **[info - operator dispositions stay open]** F1
   (`test_inspector_panel_is_wider_and_collapsible`) and F2 (server.rs R1/R2
   pair) remain explicitly NOT this track's; name-identity through T1's
   acceptance re-verified fresh (scoped 225/1, hygiene capture lines 1-2,
   census 1257/1). Correction row
   `ui.traversal_view_degree_setting_bug` stays OPEN per I8.
7. **[info - I9 parity gap carried]** `tools/ui_smoke.py` ROUTE_ASSERTIONS
   (:61-70) still omit `/app/traversal.js` + `/app/provenance.js`; the
   handoff orders this relayed to the NEXT kickoff runner, not fixed here.

## Dispositions of declared deviations (step 5)

- **D1 - accepted:** PC-4 measurement route substitution (Read-tool line
  enumeration after `python -c` refusals) is substance-neutral - this review
  independently measured 909 at 497ac61 from the blob; branch choice PRIMARY
  is what the handoff ordered at <= 950.
- **D2 - accepted on merit:** the 11-line overshoot is a planner arithmetic
  slip against a "~" estimate, not implementer drift (the +52 block IS the
  handoff's own verbatim B2 + contractual inline lines). STOP rule is
  `> 990` post-edit only; landed 961 satisfies both limits (own blob count;
  dedicated budget test green; hygiene silent on graph.js). See Finding 2
  relay.
- **D3 - accepted:** 18 invocations itemized vs the 8-9 typical; every extra
  is a precondition read, the distill-routed `git add` staging enabler (D6-
  class structural constraint - commit_scoped owns staging), post-commit
  proof reads, or the D4 rider. Declared per the M28 T0b law; auditable,
  honest.
- **D4 - accepted:** the landing hash cannot be inside the commit it names;
  landing + journal-rider is the established convention (`5b4da58`,
  `497ac61`); rider verified journal-only, zero code paths.

Errata E1-E4 are the handoff's own binding amendments, not deviations; each
was verified as IMPLEMENTED (E1 main.js rider site + 2-line cap, E2
`el.dataset.edgeTarget`, E3 net-zero trailing comments, E4 name-identity
baselines).

## Gate execution record (this session)

Ran (fresh, own invocations): `checkpoint_guard status` (twice); per-commit
`git show` (both commits, full, both diffs reviewed hunk-by-hunk);
`git diff --name-only 497ac61..HEAD`; blob line counts via byte-level
`git cat-file` inside Python (routed through `python tools/distill.py --
python tasks/<throwaway>.py`; script self-removed via the distiller - tree
left clean); `python tools/testrun.py --distill -- -p ariadne-ui`;
`python tools/testrun.py --distill -- --workspace`; `python tools/testrun.py
--baseline` (usage error - Finding 3); `python -m unittest discover -s tools
-p "*_tests.py"`; `python tools/distill.py -- python tools/hygiene_check.py`
(x2) + capture grep; `python tools/distill.py -- python
tools/encoding_guard.py --all`; `python tools/theme_token_check.py
--check-added --base 497ac61`; `python tools/distill.py -- python
tools/ui_smoke.py`.
Refusals encountered (verbatim boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
`python -c` (bare and inside the distiller), direct `python tasks/*.py`,
`Remove-Item`, `git status --porcelain` (other raw read-only git - `show`,
`diff`, `log` - ran directly; `checkpoint_guard status` covers tree state).
NOT a single gate was lost to a refusal: every declared gate body and every
acceptance-pass gate ran.
`python tools/bench_compare.py` NOT run, justified N/A: zero `crates/*/src`
paths in either commit (assets + UI tests + docs only), track gate list omits
it, zero-Rust-diff exemption precedent (T-R execution-results NOT RUN 2).
`ui_smoke --release` NOT run: opt-in lever, not required by the track.

## Ledger closeout duties remaining for the track row (closeout runner's job)

1. Flip the `ui-degree-opacity-ramp` row in `tasks/milestone_ledger.md` to
   CLOSED/APPROVED @ `d97cf90` (+ rider `7547dea`), with kilo-memory mirror
   in the SAME commit (standing convention).
2. Discharge the T-R deferred row (graph.js line-count budget lock relocated)
   to `tasks/ledger_archive.md` with a dated note pointing at `3c426eb`
   (T-R verdict Follow-up 1 - still pending).
3. Carry forward into deferred-findings: this verdict's Findings 1, 3, 4, 5,
   6, 7 (operator smoke, baseline-file errata 7th encounter, theme baseline
   + F3a, foreign docs, F1/F2 + degree-bug dispositions, ui_smoke roster
   parity gap to the next kickoff runner).
4. Track docs to commit at closeout (precedent `b4e2bb5`): kickoff, shape,
   both handoffs + journals (already tracked `497ac61`/`d97cf90`/`7547dea`)
   plus the untracked `tasks/ui-degree-opacity-ramp-tr-verdict.md` and
   `tasks/ui-degree-opacity-ramp-execution-results.md` (now with the T1
   addendum) and this verdict file.

## Execution-results addendum

Written to `tasks/ui-degree-opacity-ramp-execution-results.md` (T1 section,
this session's verbatim evidence lines + NOT-RUN ledger).
