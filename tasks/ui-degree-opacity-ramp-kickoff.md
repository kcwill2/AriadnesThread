# Kickoff - ui-degree-opacity-ramp (graph canvas graduated fade)

Track key: `ui-degree-opacity-ramp`. UNREGISTERED post-M28 change track,
following the `ui-hotspot-canvas-direction` precedent: no M-number (the ledger
is numbering authority; do not guess one). The milestone-lifecycle kickoff
runner registers the live ledger row; this doc is the architect artifact.
Quick-mode sized: exactly ONE handoff, one module directory (`crates/ui`),
no split.

## Intent

Replace the uniform heavy fade on non-highlighted graph canvas nodes/edges
with a graduated opacity ramp keyed to undirected hop distance (degree) from
the selected node. User report (verbatim): "These are too transparent.
Transparency should increase gradually by degree from the selected node."

Observed today: the heat overlay pass (`crates/ui/assets/graph.js:387-437`,
working tree at kickoff time) fades every node outside the selected node's
reachable cone at a flat `opacity: 0.12`
(`crates/ui/assets/styles.css:437-441`) and every unrelated edge at a flat
`0.06` (`styles.css:494-496`). The far field reads as dead; proximity to the
selection carries no signal.

Behavior change: closer off-story nodes render nearly opaque and each
additional hop becomes progressively more transparent down to a floor equal
to today's values. The root/selected node stays fully opaque. Justified /
reachable heat-tinted cards keep their current treatment unchanged (they are
never faded today; they stay never-faded).

Blast radius: 3 asset files (`graph.js`, `hotspots.js`, `styles.css`), 1 new
Rust test module, 1 `mod` line, and a declared 2-line amendment of one
existing test assertion. No Rust product code, no payload/API surface, no
theme layer, no new asset scripts.

Underlying why: the heat map asks "which files does this header pull in?".
The off-closure crowd is not noise proportional to distance - a file one hop
away is structurally adjacent to the story. The ramp restores distance as a
visual quantity and keeps the far field at exactly today's dimness, so
nothing becomes *harder* to see than before.

## Scope in

- `crates/ui/assets/styles.css`: the two `.faded` opacity rules become
  custom-property reads with today's values as fallbacks.
- `crates/ui/assets/graph.js`: hop-distance machinery + ramp application at
  the outbound heat overlay pass (`:387-437` working tree); leak-guard
  rider in `illuminatePath` (`:735-755`).
- `crates/ui/assets/hotspots.js`: ramp application at the inbound heat lens
  pass `renderInboundHeatView` (`:337-386` working tree) - it is the twin of
  the outbound pass and the same complaint applies to the 'in' lens.
- `crates/ui/tests/ui_test/`: one new lock module `hop_fade_ramp.rs` +
  `mod` line in `main.rs`; declared amendment of the two opacity literals in
  `static_html.rs::test_ui_noticeable_fading_elements` (`:155-156`).

## Scope out (with reasons)

- The open bug "degree setting not respected when traversal view activates"
  (corrections.md row `ui.traversal_view_degree_setting_bug`) and the
  in-flight collateral-traversal-view work (dirty working tree: new
  `traversal.js`, edits to `graph.js`/`hotspots.js`/`styles.css`/`index.html`
  and others). Reason: separate track, separate landing commit. The ramp is
  defined so that bug neither breaks it (distance is computed from
  `selectedNodeId` over the FULL `tabAllEdges` dataset, never from the
  degree/cap-filtered `activeNodes`/`activeEdges`) nor gets silently fixed
  by it (no hunk of this handoff touches `filterAndExpandSubgraph`,
  `stepDegree`, `toggleAllCapped`, or `setNodeCap`).
- `illuminatePath` ramping (`graph.js:735-755`). Reason: path illumination
  is a binary story (on-path vs off-path); a gradient would dilute it. It
  keeps flat 0.12/0.06 - guaranteed by the CSS fallbacks plus a
  `removeProperty` rider that clears leftover inline ramp properties.
- `traversal.js` and any fade application the traversal track may add.
  Reason: foreign in-flight code; the CSS fallback keeps it flat 0.12
  unless that track opts in itself.
- The deferred graph.js line-budget relocation (ledger deferred row: move the
  `<= 998` assert out of `test_extraction_impact_preview_overlay`), and the
  heat/legend extraction that row belongs to. Reason: named follow-up
  handoff owns it; this track must not absorb an extraction.
- Any change to `themes.css`, `theme.js`, `tools/theme_token_baseline.json`,
  `index.html`, the `ui_smoke.py` route roster, or `tools/**`.

## Preconditions (before the handoff may start)

1. Clean tree at `main`: the collateral-traversal-view track has LANDED (its
   files overlap this handoff's touch set; `commit_scoped` cannot separate
   foreign hunks inside a shared file) and its landing commit's scoped ui
   suite + `hygiene_check` are green.
2. `crates/ui/assets/graph.js` is under the 1000-line R2 wall at that
   landing commit. Working tree at shaping time is 1051 lines against the
   `ariadne_ui::GRAPH_JS.lines().count() <= 998` lock
   (`crates/ui/tests/ui_test/static_html_sections.rs:215`) - foreign
   landing debt. If the debt is not resolved by that track, this handoff
   does not start; it must not raise the 998 cap or the R2 wall on top of
   foreign debt. The placement decision rule in the shape artifact then
   fixes where the new JS lives.

## Invariants (numbered; the checkable versions + oracles are in the shape)

1. Far field is never worse than today: every faded node/edge with no
   finite hop distance renders exactly 0.12 / 0.06 (fallback + table floor).
2. Ramp is monotone: node hop opacity strictly decreasing from 0.85 at
   hop 1; edge hop opacity non-increasing from at most 0.30 (strictly below
   the 0.35 `.edges-has-selection` dim so a faded edge can never outshine a
   kept one).
3. Distance is hop count from `selectedNodeId` over the full undirected
   `tabAllEdges` dataset - cap/ALL-WITH-CAP/nodeCap/seeds/degree settings
   cannot influence it.
4. The SET of elements receiving `.faded` is unchanged in every pass; the
   ramp only adds an inline custom property where `.faded` is already added.
5. Heat-root, reachable heat-tinted cards, verdict spines/badges, and the
   root's tri-colored edges are untouched by this change.
6. `illuminatePath` and hotspots-flat paths still render flat 0.12/0.06.
7. Zero new hex literals; theme layer byte-identical.
8. No hunk touches the degree-filter code path (non-fixing of the open bug).
9. No new `/app/*.js` script; `ui_smoke.py` route table byte-identical.

## Frozen locks inherited (with proving command)

| Lock | Proof |
|---|---|
| `tests/fixtures/**` frozen; no new files in `tests/fixtures/basic|expensive` | `git diff <start>..HEAD -- tests/fixtures` empty |
| `tools/theme_token_baseline.json`, `themes.css`, `theme.js` byte-identical | `git diff <start>..HEAD -- <each>` empty + `python tools/theme_token_check.py --check-added --base <start>` silent |
| graph.js `<= 998` budget lock (`static_html_sections.rs:215`) stays at 998 | scoped ui suite green, that test unedited |
| Hygiene R1/R2 (R2 scans `.js`: files < 1000 lines) | `python tools/hygiene_check.py` exit 0 |
| Mojibake/non-UTF-8 gate; authored content pure ASCII; no PowerShell file rewrites | `python tools/encoding_guard.py --all` exit 0; byte-safe edit tools only |
| Scoped commits only | `python tools/commit_scoped.py --scope ...` (never `git add .`) |
| Census discipline | reviewer hop reconciles from live tree; literal `testrun.py --baseline` FILE form remains unrunnable (ledger errata row) - use `--distill` substitute |
| Distillation contract (`docs/specs/tool-output-distillation.md`) | every gate command run via the distilling entry points |

## Acceptance gate for the milestone (single handoff)

- G1 scoped ui suite green INCLUDING the new `hop_fade_ramp.rs` locks and
  the amended `test_ui_noticeable_fading_elements`:
  `python tools/testrun.py --distill -- -p ariadne-ui` (counts quoted
  verbatim; expected +1 suite).
- G2 runtime gate: `python tools/distill.py -- python tools/ui_smoke.py`
  reports `ui_smoke: PASS` (route roster unchanged, 8-route assertions).
- G3 `python tools/theme_token_check.py --check-added --base <start-rev>`
  silent (zero added hex).
- G4 `python tools/hygiene_check.py` and
  `python tools/distill.py -- python tools/encoding_guard.py --all` exit 0.
- G5 lock diffs empty: `themes.css`, `theme.js`, `theme_token_baseline.json`,
  `tools/ui_smoke.py`, `index.html`, `traversal.js`, all `crates/*/src/**`.
- G6 journal manual-smoke lines (hotspot-track precedent): outbound heat
  shows the hop gradient with the floor far field; inbound ('in' lens) heat
  shows the same ramp; clicking an inspector collateral link after a heat
  render keeps off-path cards at flat 0.12 (leak-guard proof); degree/cap
  toolbar changes leave each visible faded node's opacity keyed to the
  selection, not to the filter.
- Reviewer hop (separate session per review-gate): fresh G1-G5 re-run
  unedited, commit-vs-touch-set enforcement, numbered verdict.

Sizing law check: 1 crate dir; implementer gate invocations <= 8; expected
produced lines ~200-300 (of which ~150 Rust test) vs the ~1200 bound.

## Amendment 2026-10-09 (post-operator-commit; supersedes preconditions + sizing rows above)

Operator landed the collateral-traversal-view WIP themselves as `20e3fb8`
("fixed traversal view"). Tree clean at that commit. Verified against the
committed tree:

1. Precondition 1 MET (landed). Precondition 2 FAILED as feared: `graph.js`
   is 1051 lines at `20e3fb8`; the budget lock is still
   `<= 998` with the stale "(cap 990)" message
   (`static_html_sections.rs:214-217`); `.js` IS scanned by hygiene R2
   (limit 1000). So the base is statically RED on two blocking gates:
   `test_extraction_impact_preview_overlay` (budget assert) and
   `hygiene_check.py` R2. No reviewer can land anything green on this base.
2. The committed heat passes / fade conditions / filter tail are
   BYTE-IDENTICAL to the shaping-time anchors (see shape amendment table);
   the ramp contract survives unchanged. `traversal.js` adds no `.faded`
   site (calls `selectNode` only). The degree-setting bug remains OPEN
   (filter tail `:1014-1019` unchanged); the ramp neither breaks nor fixes
   it, as designed.
3. Handoff count AMENDED: the original "exactly one handoff, no split" no
   longer holds. **Option B chosen: TWO serialized handoffs** -
   **T-R (budget relief)** lands first (verbatim relocation of two
   graph.js blocks into `main.js` + the deferred lock relocation/fix;
   ~360 produced lines, ~6-7 gates), then **T1 (the ramp, unchanged
   contract)** starts from T-R's green landing commit. Rationale against
   folding (Option A): combined ~13-16 gate invocations across two
   work-packages in one verdict, and the ledger's deferred row reserves
   the lock relocation for an extraction-style handoff - T-R IS that
   installment and discharges the deferred row with a dated archival note
   at its close. Repo precedent serializes move-vs-feature pairs
   (clarity T-A/T-B; dark-grey then extraction-story).
4. Kickoff preconditions REPLACED by: (PC-1) clean tree at T-R's landing
   commit with scoped ui + hygiene green; (PC-2) anchor-match rule: before
   mutating, every before-string cited in the shape must match the current
   committed blob (line numbers are advisory); (PC-3) session-start red
   set for T-R itself is the budget test + R2 violation - the T-R journal
   must declare them, turning them green is T-R's acceptance.
5. New relay (out of both scopes): `tools/ui_smoke.py` ROUTE_ASSERTIONS
   (:58-70) still omit `/app/traversal.js` although `server.rs:136` serves
   it and `index.html:594` loads it (provenance.js similarly unprobed -
   pre-existing class). ui_smoke still PASSes; roster parity is
   unenforced. G2 of both handoffs keeps "route table byte-identical +
   PASS". Ledger relay row for the next kickoff runner.
6. T-R gates: scoped ui green (including the 3 declared amendments and the
   relocated budget lock at cap 990), `hygiene_check.py` exit 0,
   `ui_smoke: PASS` (via distill), verbatim-move proof (moved ranges
   byte-equal to `git show 20e3fb8:crates/ui/assets/graph.js`),
   encoding/hygiene exit 0, `theme_token_check --check-added` silent.
