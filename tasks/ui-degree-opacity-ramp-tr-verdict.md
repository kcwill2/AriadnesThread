# Verdict - ui-degree-opacity-ramp T-R (graph.js budget relief, verbatim relocation)

Reviewer: review-gate skill, fresh session, all gates re-run independently.
Commits adjudicated oldest-first: `3c426eb23a704f569454bd9ccc9cba16385305de`
(landing), `497ac61a14da4f379372810f7e78acdd3abff6d2` (docs rider).
TRACK_BASE: `5b4da58616129b2ef57f19e2bc81d28094f67a22`.

## VERDICT: APPROVED

Zero blockers. T1 (`tasks/ui-degree-opacity-ramp-t1-handoff.md`) may dispatch
from this track's landing lineage (HEAD `497ac61`; T1 re-anchors its own
before-strings against this tree per its contract).

## Ground truth (step 1)

`python tools/checkpoint_guard.py status`: HEAD `497ac61`, branch main,
dirty=none, worktree census unchanged from track baseline (all other
worktrees behind/detached, none touching this range). Untracked: exactly the
two foreign `ui-traversal-back-button` docs the journal declared report-only.

## Per-commit review (steps 3-4, oldest-first)

### 3c426eb - landing

- `git show --stat`: graph.js 0/142 (deletions ONLY), main.js 142/0
  (insertion ONLY, single site), edge_verdict_story 1/1, node_verdict_spine
  3/3, static_html 6/6 (net-zero, E3 held - file measured 999 at HEAD),
  static_html_sections 16/9, journal +170 (created, in-set). Name-only set ==
  touch-set + journal, EXACT.
- Hunk audit: Block 1 deleted at 445-500 (BFS comment of `computeFull
  Dependencies` survives - E1 honored); Block 2 at 671-756; seams match the
  handoff post-state verbatim (`}` blank `// BFS...`; selectNode close blank
  `function clearInspector() {`; `    }` blank B1 blank B2 blank
  `function showProgress`). No double blanks.
- V5: no hunk anywhere near `filterAndExpandSubgraph`/`stepDegree`/
  `toggleAllCapped`/`setNodeCap`; degree-bug territory byte-identical.
- E2 enumeration: EXACTLY the 9 table rewrites + 1 relocation. Row-by-row
  verified against the C3 table incl. message strings
  (`(main.js after T-R move)` wording matches). Containment objects move to
  MAIN_JS - no assertion weakening. The only additional test-content edit is
  the C2-declared comment trim. No 10th semantic test edit.
- C2 relocation plan-literal: deleted 4-line 998-assert matches the handoff's
  quoted block verbatim; the stale `(cap 990)` message dies with it; new
  `test_graph_js_line_budget` matches the handoff's pinned block verbatim
  (cap 990, message names its own number, provenance comment intact).
  The 998-removal is the ONLY semantic relaxation and it LOWERS the cap.
- Amendment 3 (`recordCrumb` -> `clearInspector` slice terminator): present in
  this commit; Red-neutral pre-move was empirically captured (journal #4) and
  it is load-bearing at the move. See Finding 2 on the handoff's own wording.

### 497ac61 - docs rider

Scope = track planning artifacts only: kickoff, shape, T-R + T1 handoffs,
ledger row, plan file (`.kilo/plans/1791502926950-...md`), journal tail (+75:
D-rows + Close, which necessarily carry the landing hash). Zero code paths.
Declared as the rider's scope in the dispatch; pattern precedent: the closed
back-button track landed as landing + journal rider (`9c0cd0c` + `5b4da58`).
Foreign untracked back-button docs deliberately NOT committed - verified still
untracked. Accepted (see Disposition R below).

## Independent verification (steps 2, 5-7 - fresh runs, this session)

- **Verbatim-move proof (my own, stronger oracle):** blob-vs-blob against
  TRACK_BASE and HEAD (immune to the autocrlf worktree question). Results:
  per-block byte-containment True/True; `graph.js == base minus declared
  deletions ONLY` True; `main.js == base main + single-site verbatim
  insertion` True. The whole-file reconstructions subsume containment and
  additionally prove zero collateral in both assets. Line counts measured
  from blobs: graph.js 1051->909, main.js 604->746. Blobs pure LF. The
  handoff's literal joined recipe is False - confirming D1's claim that it is
  unsatisfiable by construction (it concatenates blocks the insertion spec
  requires to be separated).
- **Scoped ui suite fresh:** canonical line verbatim
  `220 passed / 1 failed / 0 ignored / 2 suites`; failing set == exactly
  {`static_html_sections::test_inspector_panel_is_wider_and_collapsible`},
  message "the inspector collapse/toggle controls must call
  toggleInspector()", fires at :418.
- **F1 inherited-red confirmed:** base blob carries the same assert at
  :411-414 with the byte-identical message (HEAD :418 = pure +7 shift from the
  declared C2 line delta); `index.html` (the failing input) is byte-identical
  base->HEAD (absent from the whole-range name-only diff). The red therefore
  fires identically at TRACK_BASE - not attributable to this track. Its fix
  remains the operator's F1 disposition.
- **Hygiene fresh:** `### generic FAIL (exit 1)`; full 176-line capture
  contains EXACTLY two blocking findings - `crates/ui/src/server.rs: R2 1120
  lines > 1000` and `crates/ui/src/server.rs:764: R1 fn api_graph_traversal
  is 161 lines (> 100)`. The `graph.js` finding is absent from the entire
  capture - GONE BY FIX. `server.rs` is byte-identical base->HEAD, so the
  pair fires identically at base (inherited F2; out of touch-set; exit 0
  unreachable by design).
- **encoding_guard:** `python tools/distill.py -- python tools/encoding_guard.py
  --all` -> exit 0, silent.
- **theme_token_check --check-added --base TRACK_BASE:** output == the
  journal's 4 lines verbatim. Base-relative added-scan == EXACTLY the one
  sanctioned `main.js: new hex literal #8250` phantom (E5/F3a - `&#8250;`
  entity; tool `HEX_LITERAL` has no `&#\d+;` exemption). The other 3 lines
  are the base-INDEPENDENT frozen-count scan (`scan_frozen_counts` reads the
  worktree vs pinned JSON - mechanism verified in the tool source):
  `main.js 1 > 0` is the count-twin of the same sanctioned entity;
  `index.html 6 > 4` and `traversal.js 1 > 0` are byte-inherited (both files
  byte-identical to TRACK_BASE). No other new hex - V4 holds. See Finding 3.
- **ui_smoke:** `ui_smoke: PASS` (exit 0, distill route), route table
  byte-identical (`tools/ui_smoke.py` untouched) - V2 roster frozen.
- **Frozen locks:** every V2/lock-list file (index.html, server.rs, export.rs,
  theme_contract.rs, themes.css, theme.js, baseline JSON, tools/**, all
  `crates/**/src/**`, all assets except graph.js/main.js) absent from the
  whole-range name-only diff -> structurally byte-identical.
  `theme_js_zero_hex.rs` unedited. Zero collateral outside the touch-set.
- **Report evidence (step 8):** journal is digest-shaped throughout - digest
  headers, canonical lines verbatim, capture paths, exit codes, zero raw
  dumps; all gate commands via distilling entry points. Compliant with
  `docs/specs/tool-output-distillation.md`.

## Numbered findings (all minor; no blocker)

1. **[minor - config relay, standing 6th-encounter erratum]** Literal
   `python tools/testrun.py --baseline` unrunnable: `usage error: missing
   value for --baseline` (`--baseline` is FILE-valued, `tools/testrun.py:6,
   :206-207`; no test-count baseline file exists in the repo). Substitute
   workspace census run instead: `1252 passed / 1 failed / 12 ignored /
   70 suites` (single failure = inherited F1). Relay to the ledger owner; the
   substitution is the established precedent (hotspot-canvas-direction
   execution-results:44-57).
2. **[minor - planner copy]** Handoff self-contradiction adjudicated in the
   implementation's favor: RGB step 1 says amendment 3 "must land in this
   test commit, never with the move", while Gate 9 mandates ONE landing
   commit. The implementation followed Gate 9; the substantive intent
   (amendment 3 applied pre-move and Red-neutral there) was captured
   empirically in the Red run. Future handoffs with single-commit mandates
   should drop per-phase commit language.
3. **[minor - deferred-findings relay]** Theme frozen-count drift pair
   `index.html 6 > 4` + `traversal.js 1 > 0` (byte-inherited) plus the
   `main.js` count-twin want a `tools/theme_token_baseline.json`
   reaffirmation decision by its owner; and deferred row F3a (the
   `HEX_LITERAL` `&#\d+;`-entity misread at `tools/theme_token_check.py`)
   stays OPEN - this landing adds a fifth sanctioned phantom to its
   population. No fix in-scope here (E5: verbatim move wins; baseline JSON is
   golden).
4. **[info - closeout relay]** Foreign untracked back-button artifacts
   (`tasks/ui-traversal-back-button-execution-results.md`,
   `tasks/ui-traversal-back-button-t1-verdict.md`) still sit in the tree;
   they belong to the back-button track's own closeout, not to this track.

## Dispositions of declared deviations (step 5)

- **D1 - accepted (accepted substitution, judged on merit):** the handoff's
  joined-blob recipe is provably unsatisfiable against its own insertion
  spec; the implementer's per-block containment + exact whole-file
  reconstruction is strictly stronger and IS what the frozen-locks table
  names. Independently re-proven by this review (blob-level, all True).
- **D2 - accepted:** the 142-line layout is pinned by the handoff's own
  budget math (1051-142=909, 604+142=746) and its no-double-blank rule; the
  declared post-state seams are matched verbatim. The "three added blanks"
  phrasing is the handoff's arithmetic slip (a third blank would create the
  double blank it forbids).
- **D3 - accepted:** added-scan half of E5 satisfied exactly (one sanctioned
  phantom); the frozen-count lines are base-independent by verified tool
  mechanism and two of them byte-inherited. Name-identity logic (E4) applied
  correctly; nothing "fixed" cosmetic; baseline untouched.
- **D4 - accepted:** "passed = Step 0 passed + 1" is internally inconsistent
  with the same paragraph's expectation that the overlay test returns green
  BY FIX; measured +2 is the honest number, name-identity holds (freshly
  confirmed this review: 220/1).
- **D5 - accepted:** overshoot declared per the M28 T0b law. 16 distilling
  invocations itemized; the five named extras are each justified (Step-0.4
  exit-code probe, prescribed refactor re-run, autocrlf diagnostic required
  for a correct byte proof, second `--staged` for the second commit,
  orchestrator-dispatched rider); the two not restated in D5's parenthetical
  (#2 Step-0 raw hygiene, #9 denied direct proof attempt) are itemized in
  their own journal sections. Auditable, honest.
- **D6 - accepted:** `--staged` before the first commit is structurally
  impossible (commit_scoped owns staging); the post-scoped-commit run inspects
  exactly the committed blobs, and this review's `--all` exit 0 subsumes it.
- **Disposition R (docs rider, 497ac61) - accepted:** outside the T-R
  touch-set textually, but named as the rider's scope in the dispatch and
  consistent with the landing + docs-rider convention this repo already uses
  (back-button `5b4da58`). Contains only track planning artifacts + journal
  tail; zero code; foreign docs left untracked as declared.

## Gate execution record (this session)

Ran (fresh, own invocations): checkpoint_guard status; per-commit `git show`
(both commits, full); `git diff --name-only` (range + per commit); independent
reconstruction proof (via `python tools/distill.py -- python <script>`, then
self-removed); `python tools/distill.py -- python tools/testrun.py --distill
-- -p ariadne-ui`; `python tools/testrun.py --distill` (workspace census);
`python tools/testrun.py --baseline` (usage error - see Finding 1);
`python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 219 tests in
18.839s` / `OK`; `python tools/distill.py -- python tools/hygiene_check.py`;
`python tools/distill.py -- python tools/encoding_guard.py --all`;
`python tools/distill.py -- python tools/theme_token_check.py --check-added
--base 5b4da58...`; `python tools/distill.py -- python tools/ui_smoke.py`.
Refusals encountered (verbatim, all the catch-all boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
chained `;` command, raw `python -c` (direct and inside distill),
`Remove-Item`, plus an `edit`-permission refusal for the OS temp dir
(`{"permission":"edit","pattern":"*","action":"deny","source":"agent"}`).
None denied a gate: every expected gate ran (the `--baseline` literal was
unrunnable for its own independent file-valued reason, Finding 1).
`bench_compare` NOT run: justified N/A - zero `crates/*/src` paths in either
commit, this track's gate list omits it, zero-Rust-diff exemption precedent
(hotspot-canvas-direction execution-results item 2).

## Follow-ups recorded (never dropped)

1. Ledger closeout (reviewer-hop item 6): archive deferred row "graph.js
   line-count budget lock hides inside test_extraction_impact_preview_overlay"
   to `tasks/ledger_archive.md` with a dated discharge note pointing at
   `3c426eb` - closeout runner's job.
2. Findings 1, 3, 4 above go to the ledger deferred-findings rows.
3. T1 dispatch unblocked from this lineage; F1 (`test_inspector_panel_
   is_wider_and_collapsible`) and F2 (`server.rs` pair) remain explicitly NOT
   this track's and must keep name-identity through T1's acceptance.
