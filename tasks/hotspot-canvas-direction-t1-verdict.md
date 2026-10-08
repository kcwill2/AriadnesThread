# Verdict — ui-hotspot-canvas-direction T1 (handoff `tasks/hotspot-canvas-direction-t1-handoff.md` incl. AMENDMENT A1)

Verdict: **APPROVED** at `c17d182facfa34b822fa18db1e174a6052921000` (parent `7036c9e`, track baseline `c00576d`).
Reviewer hop: 2026-10-08, via `review-gate` skill. All scoped gates re-run fresh; workspace-wide gates
run at this hop per handoff ("Reviewer hop (workspace-wide — NOT implementer gates)").

## 1. Commit-vs-touch-set — PASS

Single commit `c17d182`; `git show --stat` and `git diff --name-only 7036c9e..c17d182` give exactly the
seven declared scopes, nothing else:

```
crates/ui/assets/graph.js                       7+/1-   (net +6, cap +8)
crates/ui/assets/hotspots.js                    146+/1-
crates/ui/assets/index.html                     4+/0
crates/ui/tests/ui_test/hotspot_direction.rs    216+/0  (new)
crates/ui/tests/ui_test/main.rs                 1+/0
crates/ui/tests/ui_test/static_html_sections.rs 1+/1-   (AMENDMENT A1)
tasks/hotspot-canvas-direction-journal.md       163+/0  (new)
```

No foreign paths swept in: the shared worktree's ~40 untracked M28 artifacts plus modified
`tasks/milestone_ledger.md`, `tasks/ledger_archive.md`, `.kilo/plans/1790697359283-*`,
`tools/bench_baseline.json` (all foreign-owned per the handoff's discipline clause) remain
uncommitted. `git add .`/`-A` discipline held.

Per-commit review (oldest-first; one commit): full `git show` per file adjudicated below.

### AMENDMENT A1 line — exact

The `static_html_sections.rs` diff is exactly the authorized one line at :215
(hunk `@@ -212,7 +212,7 @@`, changed line = :215):
`ariadne_ui::GRAPH_JS.lines().count() <= 990` -> `<= 998`. The :216 assert-message literal
"(cap 990)" is untouched — stale **by authorization** (A1), owned by the deferred ledger row;
not counted as a finding. No other line in the file, and no other pre-existing test file,
appears in any diff. The A1 carve-out was consumed exactly once.

## 2. Invariant / decision adjudication (I1-I8, D1-D3) — PASS

Verified against the committed files, not the report:

- **I1** `hotspotMetricDirection` (hotspots.js:14-19) is the contract verbatim:
  burden/fanin->`'in'`, fanout/closure->`'out'`, cycle->`null` (hotspots.js:17-18 comment names D3).
- **I2** `selectHotspotRow` (hotspots.js:289-294) matches the contract verbatim; the bare
  `selectNode(id)` literal survives (hotspots.js:293); the lens travels via `pendingHeatLens` only.
  Both call sites route through it: hotspot row hotspots.js:274, extraction card hotspots.js:536.
  Pre-existing `assert_hotspot_burden_and_list_rules` green UNEDITED in the fresh run.
- **I3** Lens is consumed as the first two statements of `selectNode` (graph.js:474-475, comment
  excluded from statement order); `dirSelect` onchange nulls BOTH flags before
  `filterAndExpandSubgraph()` (hotspots.js:7-10). Manual direction changes therefore land in the
  `} else if` branch = today's byte-identical behavior; every other selection path (canvas click,
  inspector links graph.js:542/:556, breadcrumb graph.js:644) inherits a null lens because
  `pendingHeatLens` is armed ONLY inside `selectHotspotRow` and cleared on every consumption.
- **I4** `paid = nid => ((graphData.file_collateral || {})[nid] || {})[selectedNodeId] || 0`
  (hotspots.js:322) — consumer-first lookup, contract literal; no `computeFullDependencies` in the
  view (own reverse BFS `computeFullConsumers`, hotspots.js:299-311, mirror of graph.js:457-470
  with `e.target === curr` / enqueue `e.source` flip).
- **I5** Outbound heat block byte-identical: the whole graph.js diff is two hunks; the 7+/1-
  is fully accounted for (leading comment + 2 new branch lines + `} else if` rewrite, minus the
  old condition line; selectNode comment + 2 statements). `const reachableDependencies`
  (graph.js:331) through `updateHeatLegend(...)` (:393) and `else { restoreStandardLegend(); }`
  (:394-396) are unchanged diff context — git's blob comparison is the byte proof.
- **I6** Inside `renderInboundHeatView` the only `graphData.<field>` reads are `file_collateral`
  and `file_justification` (hotspots.js:322-323); regex lock test 7 enforces, plus my read-through
  (`tabAllNodes`/`tabAllEdges` are pre-existing state globals, not payload fields).
- **I7** No new asset: index.html diff adds one `<li>` only, zero `<script>` changes; 8-script
  roster + export order green via pre-existing theme_contract/export_order locks (UNEDITED, in
  the 217) and the clean `ui_smoke` route table (F1 form below).
- **I8** Help copy verbatim contract (index.html:511-514), inserted directly after the :510 anchor
  item, sentence-per-line, ASCII, `&larr;`/`&rarr;` entities; contains `natural direction` and names
  the cycle exception. "Hotspots Triage" tour step: main.js not in the commit at all.
- **D1** Metric `<select>` onchange is `renderHotspots()` only (index.html:221) — never touches
  `dirSelect`; the burden->fanin no-collateral fallback (hotspots.js:252-255) rewrites the metric
  to another `'in'` metric, coherent.
- **D2** Inbound legend owns its copy: summary `${payers.size} payer(s) - ${total} symbols paid`,
  rows "Payer fully needs it" / "Payer partly needs it or no verdict data" /
  "Payer uses none of it - include can go"; gating idiom mirrors `updateHeatLegend`
  (main.js:186-200: green gated on justified, red on unused, amber unconditional);
  `updateHeatLegend` itself untouched; spine rows via `verdictCardLegendRows`, heat rows via
  `HEAT_LEGEND`. `updateHeatLegend` verbatim reuse is nowhere present.
- **D3** cycle -> mapping returns `null` -> `dirSelect` untouched, `pendingHeatLens = null` ->
  current Direction + outbound view kept (code chain above).
- Mirrors vs frozen block: root badge idiom byte-mirrored (x=32/y=36/`var(--cyan)`/`[ROOT]`),
  `styleDependencyCard` return consumed as `hc.border` exactly like graph.js:356-361;
  `cycle-edge` guard (hotspots.js:376) mirrors graph.js:375; `url(#arrow-entering)` for justified
  mirrors graph.js:382; tooltip argument order swapped to consumer-first
  (`appendVerdictTip(el, nodeLabel(nid), rootLabel, ...)`) exactly as the shape requires.
- Degradation contract: absent justification -> `classifyDependency` returns `'neutral'`
  (graph.js:403) -> `+N col` badge, no tip; true-zero pays `+0 col`; every map read on the
  `|| {}` guarded idiom; no fetch, no new throw sites.

graph.js: 990 -> 996 (<=998 lock and <=996 shape target both held; HARD CAP +8 respected at +6).
`renderInboundHeatView` = hotspots.js:320-414 = 95 lines (claim exact; <100 R1).

## 3. Fresh gate re-runs (reviewer, at HEAD c17d182)

| Gate | Result |
| --- | --- |
| 1 `python tools/testrun.py --distill -p ariadne-ui` | `### cargo test PASS (exit 0)` / `217 passed / 0 failed / 0 ignored / 3 suites` (suites 3 at GREEN as A1 requires) |
| 2 `python tools/hygiene_check.py` | exit 0; zero R1/R2 rows; R3 warns exclusively pre-existing in foreign files (journal claim confirmed) |
| 3 `python tools/theme_token_check.py --check-added --base c00576d4a574052e00c88ad4e36ca2f1b3aeefa0` | silent (zero added hex literals) |
| 4 encoding | commit landed, so `--staged` is not reproducible post-commit; ran the superset `python tools/encoding_guard.py --all` -> silent, exit 0 |
| 5 `python tools/distill.py -- git diff HEAD -- crates/cli` | `### generic PASS (exit 0)` / capture 0 lines — DP-1 intact |

### Workspace-wide gates (this hop's obligation)

- Literal `python tools/testrun.py --baseline`: **usage error `missing value for --baseline`** —
  the flag is FILE-form (`--baseline FILE`, tools/testrun.py:6,206) and no test-count baseline file
  is committed. 5th documented encounter (boot-progress-visibility ledger row; the handoff itself
  states "the literal `--baseline` errata stands"). Standing substitute run fresh:
  `python tools/testrun.py --distill` (unscoped workspace) ->
  **`1263 passed / 0 failed / 13 ignored / 91 suites`**, exit 0, capture
  `target/testrun-capture.txt` (2231 lines). The handoff's `~1221` projection was declared
  "reconcile from the live tree, don't anchor blindly"; 0 failures is the verdict column, and the
  scoped 209+8 -> 217 reconciliation is exact for this hop's eight new tests.
- `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 219 tests in 18.356s` / `OK`.
- `python tools/ui_smoke.py` -> refused by the session bash allowlist twice (verbatim refusal
  F1); executed via the AGENTS.md distilling entry point
  `python tools/distill.py -- python tools/ui_smoke.py` ->
  **`boot+route checks clean (cold fixture, attempt 1, 1.1s)` / `ui_smoke: PASS`**, exit 0.
- `python tools/bench_compare.py` -> **N/A, justified**: the commit touches zero paths under
  `crates/*/src` (pure UI assets + UI-test module + journal; `git show --stat` above), so the
  stage-5 indexing tail it measures is byte-unaffected; the shape's acceptance gate list
  (shape:295-306) does not include bench_compare; precedent: ui-extraction-card-playbook T-C
  "assets+tests EXEMPTION recorded with zero-Rust-diff proof" (ledger row 30).

## 4. Report-evidence check (distillation contract)

Journal is digest-shaped: canonical lines, exit codes, the distill header
(`### cargo test PASS (exit 0)`), gate-invocation itemization (24, declared D3), zero raw dumps.
One evidence gap — finding F3.

## 5. Deviation dispositions (independent adjudication)

- **D1** journal at `tasks/hotspot-canvas-direction-journal.md` — ACCEPTED: handoff :142-144
  pre-rules the shape name WINS; the file matches and the handoff's own touch-set names it.
- **D2** `2 suites` in failing digests (RED 209/8, pre-A1 216/1) — ACCEPTED: A1 documents the
  panic-abort artifact for failing runs only; passed/failed reconciled to 209+8; GREEN reads
  `3 suites` (re-verified here); lineage M27 T2 counting-errata family.
- **D3** 24 gate invocations vs pre-amendment <=10 — ACCEPTED with note: fully itemized and dated
  (M28 T0b law: declared harmless overshoot is acceptable); note that hop-1 alone consumed 11
  against the original budget of 10 — declared, immaterial, no re-run was wasted.
- **D4** `git add -- <seven explicit paths>` to stage for the `--staged` encoding scan, commit
  still via `commit_scoped` — ACCEPTED: the landing commit's content is the verified 7-path set;
  the staging form is commit_scoped's own explicit-path form; boot-overlay precedent cited.
- **D5** `function_body` copy with asset-neutral panic text — ACCEPTED: side-by-side vs
  `extraction_card_playbook.rs:45-56` shows logic byte-equivalent; only the first panic string
  generalizes ("the asset must define"), required because the suite locks graph.js functions too.

Journal-name ruling (no `-t1-` suffix) and the stale "(cap 990)" literal: both pre-authorized —
not findings, as instructed.

## 6. Findings (numbered; none blocking)

1. **F1 (minor — reviewer-profile config relay).** `python tools/ui_smoke.py` (and its
   `tools\ui_smoke.py` variant) refused with the catch-all
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}` even though it is this
   hop's mandated gate. Same class as the recorded incident (ledger row 29: "permission denials
   always report the catch-all rule"). The mandated result was still obtained via the documented
   distilling entry point (PASS above). Disposition: add direct `ui_smoke` to the reviewer
   profile's allowlist at next tooling touch. Evidence: this hop's two refusal returns.
2. **F2 (minor — config relay, 5th occurrence).** `testrun.py --baseline` literal unrunnable
   (FILE flag; no committed baseline file). Verbatim: `usage error: missing value for --baseline`.
   Substitute census applied exactly as the handoff prescribes. Disposition: the standing ledger
   errata row already tracks the `--baseline` decision — this recurrence counts toward it.
3. **F3 (minor — report evidence).** The journal's gate-1 citations omit the distill capture path
   (`target/testrun-capture.txt`) though header, canonical line, and exit code are present
   (journal:85-89); gate 5 cites the capture line-count but not its path (journal:99-100).
   Per `docs/specs/tool-output-distillation.md` evidence rules the capture path belongs in the
   citation. No action required: the reviewer's fresh run re-captured it (236 lines).
4. **F4 (relay to orchestrator).** Ledger track row `milestone_ledger.md:35` still reads
   `PLANNED` (mid-hop text updated to carry A1 + the GREEN landing, but the flip to
   LANDED/APPROVED with landing hash `c17d182` + the kilo-memory mirror
   `ledger.milestone.ui-hotspot-canvas-direction` is the closeout duty and remains outstanding);
   the deferred-findings row for the buried 990-lock relocation + stale "(cap 990)" message IS
   recorded (milestone_ledger.md:79, routed to the heat/legend extraction follow-up named in
   shape R1) but sits in the **uncommitted** worktree diff (+2/-1, verified via
   `git diff --unified=0 -- tasks/milestone_ledger.md`). The track's planning docs
   (handoff/shape/kickoff/plan) are likewise untracked. All are orchestrator duties — the
   implementer's touch-set FORBADE ledger edits, so this is a relay, not a blocker.
5. **F5 (observation, no action).** hotspots.js landed 610 lines vs the shape's "~575" estimate
   (shape:228, tilde-prefixed); ceiling 1000 unbreached; graph.js remains the binding file.

## 7. R8 interactive-click acceptance — adjudicated from code path + smoke (stated)

No DOM harness exists in this repo (kickoff scope-out, shape R8 — a stated limit of the acceptance
design, not a silent gap), and this reviewer environment offers no browser, so the live click was
NOT executed. The chain is deterministic and line-verified: burden row click (hotspots.js:274) ->
`selectHotspotRow` sets `dirSelect.value='in'` -> toolbar option label is
"Consumers (&larr;)" (index.html:160) -> `pendingHeatLens='in'` -> `selectNode` consumes the lens
first (graph.js:474-475) -> `renderSvg` (:606) sees `hotspotHeatLens==='in'` and delegates to
`renderInboundHeatView` (:328-329) -> `filterAndExpandSubgraph` (:609) with `dir==='in'` expands
consumers (graph.js:904, `branchDir='in'` over `adjIn` :891-894) -> final same-frame state:
consumers expanded, payer cards heat-styled with `+N col`/k-over-m badges
(hotspots.js:344-366), non-consumers faded, root `[ROOT]` + `heat-root`. `ui_smoke` boot+route
PASS corroborates the unchanged roster the view renders into. Acceptance (e)'s manual-smoke line
is present at journal end (journal:157-163). The transient first render inside the frame is R4,
ACCEPTED per shape and recorded in the journal.

## 8. Census / worktree status (attributable to this track)

`python tools/checkpoint_guard.py status` at hop start: HEAD `c17d182` on `main`, ahead=0/behind=0,
dirty=42 — every dirty/untracked entry is foreign-owned (the handoff's named foreign set, prior
tracks' untracked docs/probe scripts) or this hop's own end-of-session artifacts. 8 worktrees;
`tarjan-s-scc` carries the 2 foreign dirty lines documented in the M26 closeout row. No stray
worktree or dirty surface is attributable to ui-hotspot-canvas-direction; all seven track paths
are committed at `c17d182`.

## 9. Kickoff acceptance gate (a)-(e)

(a) frozen locks green (§1-§3) — PASS; (b) 8 new locks green (§2, §3) — PASS;
(c) full suite green, zero pre-existing test edits except the A1 line (§1, §3) — PASS;
(d) ui_smoke boots clean on unchanged roster (§3, F1 form) — PASS;
(e) journal manual-smoke line (§7) — PASS.
