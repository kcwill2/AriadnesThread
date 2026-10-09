# T-R Journal - ui-degree-opacity-ramp: graph.js budget relief (verbatim relocation)

Handoff: `tasks/ui-degree-opacity-ramp-tr-handoff.md` (binding contract).
Track: `ui-degree-opacity-ramp` (unregistered post-M28; ledger is numbering
authority). Installment 1 of 2 serialized handoffs; T1 starts only on this
landing commit.

## Step 0 - pre-implementation checkpoint (before any edit)

- TRACK_BASE = `5b4da58616129b2ef57f19e2bc81d28094f67a22` (`git rev-parse HEAD`
  via `python tools/distill.py -- git rev-parse HEAD` - raw git denied by
  permission; distilling entry point used for all git observation).
- Dirty tree at start: ONLY the track's planning artifacts
  (`tasks/milestone_ledger.md` live row, `tasks/ui-degree-opacity-ramp-kickoff.md`,
  `tasks/ui-degree-opacity-ramp-shape.md`, both ramp handoff files,
  `.kilo/plans/1791502926950-degree-opacity-ramp.md`) plus two foreign
  untracked `ui-traversal-back-button` docs (execution-results, t1-verdict).
  ZERO dirty files on the touch-set; no STOP. Foreign docs are NOT mine to
  commit (report-only; declared D3 below).
- Anchor-match PASS: every before-string in the handoff found byte-verbatim in
  the committed tree (touch-set paths clean, working == committed). Verified:
  graph.js Block 1 445-499 (+ blank 500; BFS comment 501-502 stays), Block 2
  671-755 (+ blank 756; `function clearInspector() {` at 757), main.js
  `restoreStandardLegend` close `    }` at 230 / blank 231 / `showProgress` 232,
  static_html.rs :238/:245/:367-368 (file 999 lines, AT the R2 ceiling - E3
  net-zero applies), static_html_sections.rs fn@153, comment 207-209,
  clearPreview assert 210-213, budget assert 214-217, fn close 224, :310/:314,
  node_verdict_spine.rs :25/:29/:33 (+ survivor :45), edge_verdict_story.rs
  :130 (+ survivors :119/:126).
- E2 machine re-verified: in graph.js `verdict-spine` occurs only at :466,
  `heat-tier-` only at :486, `heat-critical` only at :487, `appendVerdictSpine(`
  def :460 + single caller :488 (all inside moved bodies); survivors
  `verdictBadgeColor(` caller :407 stays; `renderThreadBar()` callers
  :102/:524/:775/:846 + main.js:361 stay; breadcrumb locks
  static_html.rs:720-748 read `common::page_content()` (union) - cross-file
  moves survive; `static_html_sections.rs:775` is a `js_fn_body`
  `clearSelection` slice lock - host stays in graph.js.
- graph.js line count at base: 1051.

## Gate log (itemized; declared cap 9)

(append one checkpoint per gate; counts totalled at close)

### Step 0 capture - BASELINE FAILING SET (red base; budget test = T-R's
### target, inspector test = inherited F1)

- Gate invocation #1 (Step 0 testrun): `python tools/testrun.py --distill -- -p ariadne-ui`
- Digest header (verbatim): `### cargo test FAIL (exit 1) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 270 lines]`
- Canonical summary line (verbatim): `218 passed / 2 failed / 0 ignored / 2 suites`
- Capture path: `target/testrun-capture.txt` (full failing set read from it):
  - `static_html_sections::test_inspector_panel_is_wider_and_collapsible` (inherited F1 - NOT T-R's)
  - `static_html_sections::test_extraction_impact_preview_overlay` (budget assert, message "graph.js must stay within the spine-plan line budget (cap 990)" - T-R's target)
- Matches erratum E4 expectation exactly: exactly these two reds.
- Step 0 hygiene (invocation #2 raw + #3 exit-code probe via
  `python tools/distill.py -- python tools/hygiene_check.py`):
  `generic FAIL (exit 1)`. Blocking findings (names):
  `crates/ui/assets/graph.js: R2 1051 lines > 1000` (T-R's target),
  `crates/ui/src/server.rs: R2 1120 lines > 1000`,
  `crates/ui/src/server.rs:764: R1 fn api_graph_traversal is 161 lines (> 100)`
  (inherited F2 pair; must survive untouched). R3 rows are warn-only.

### Red - C2 + C3 applied, assets untouched (invocation #4)

- `python tools/testrun.py --distill -- -p ariadne-ui`
- Digest: `### cargo test FAIL (exit 1) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 296 lines]`
- Canonical line: `215 passed / 6 failed / 0 ignored / 2 suites`
- FULL failing set (from capture): `edge_verdict_story::test_edge_verdict_graph_wiring`,
  `node_verdict_spine::test_graph_js_builds_spine_and_flat_tiers`,
  `static_html::test_ui_heat_map_tri_state_justification`,
  `static_html_sections::test_graph_js_line_budget` ("cap 990; R2 hygiene wall is 1000"
  fires with 1051 > 990), `static_html_sections::test_inspector_panel_is_wider_and_collapsible`
  (inherited F1, same message "the inspector collapse/toggle controls must call
  toggleInspector()"), `static_html_sections::test_ui_heat_tier_outlines_are_flat`.
- == BASELINE set MINUS `test_extraction_impact_preview_overlay` PLUS the five
  named in RGB step 1 - exactly as predicted. Assertion failures, compile green.
- `static_html.rs` re-counted: 999 lines (E3 net-zero held).
- Amendment 3 (selectNode slice terminator) verified Red-neutral pre-move: its
  test stayed green; it becomes load-bearing at the move (lands in this test
  state, never with the move - satisfied: C3 committed logic is asset-independent).

### Green - C1 verbatim move performed (invocation #5 testrun)

- Deletions: graph.js 445-500 (Block 1 incl. trailing blank) + 671-756
  (Block 2 incl. trailing blank) = 142 lines out, deletions ONLY.
- Insertion: single site in main.js after `restoreStandardLegend`'s close
  (base line 230): blank + Block 1 (55) + blank + Block 2 (85); the existing
  blank before `showProgress` is the spec's trailing blank (see D2: the
  handoff's literal "one blank, B1, one blank, B2, one blank" = 143 lines,
  its own budget math 604+142=746 and the no-double-blank rule pin the
  142-line layout; resulting seams match the handoff's stated post-state
  verbatim: `}` blank `// BFS over...` at the Block-1 site; selectNode close
  blank `function clearInspector() {` at Block-2; `    }` blank Block1 blank
  Block2 blank `function showProgress` in main.js).
- Move math verified: graph.js 909 (<= 990 lock, < 1000 R2), main.js 746
  (< 1000), both measured from the final bytes.
- `python tools/testrun.py --distill -- -p ariadne-ui`
- Digest: `### cargo test FAIL (exit 1) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 266 lines]`
- Canonical summary line (verbatim): `220 passed / 1 failed / 0 ignored / 2 suites`
- Failing set == exactly {`static_html_sections::test_inspector_panel_is_wider_and_collapsible`}
  - inherited F1 UNTOUCHED (same assert site :418, same message "the inspector
  collapse/toggle controls must call toggleInspector()"; the file shift 411->418
  is pure line-arithmetic from this file's own +7 C2 lines, zero content edit).
- Passed count measures 220 = Step 0's 218 + overlay-by-fix + new budget test.
  (D4: handoff's "passed = Step 0 passed + 1" undercounts by the overlay test it
  simultaneously expects green BY FIX; measured honestly, name-identity holds.)

### Refactor - none permitted inside moved code (V1) (invocation #6 testrun)

- No code edits beyond journal/hygiene; re-run confirms Green exactly:
  `220 passed / 1 failed / 0 ignored / 2 suites`, failing set {F1}.

### Gate 4 - post-state hygiene (invocation #7 via distiller)

- `python tools/distill.py -- python tools/hygiene_check.py` ->
  `### generic FAIL (exit 1)`; full capture grepped: blocking findings are
  EXACTLY the inherited pair `crates/ui/src/server.rs: R2 1120 lines > 1000` +
  `crates/ui/src/server.rs:764: R1 fn api_graph_traversal is 161 lines (> 100)`.
  The `graph.js R2` finding is GONE BY FIX (handoff hygiene acceptance;
  exit 0 unreachable while deferred row F2 stands).

### Gate 5 - ui_smoke (invocation #8)

- `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)`, `ui_smoke: PASS`, route table byte-identical
  (`tools/ui_smoke.py` untouched; V2 holds).

### Gate 6 - verbatim-move proof (invocations #9 denied direct run, #10 newline
### diagnostic, #11 corrected proof; see D5)

- Handoff recipe + reconstruction against the TRACK_BASE blobs
  (`git show 5b4da586...:crates/ui/assets/graph.js` byte-level, no PowerShell
  text pipes), run via `python tools/distill.py -- python <script>`; the
  worktree is normalized CRLF->LF because `core.autocrlf=true` makes that the
  exact byte mapping of what commits (measured: blobs pure LF, worktree pure
  CRLF, zero mixed endings).
- Results: block1 byte-identical containment in main.js: True.
  block2 byte-identical containment: True.
  main.js == base main.js + single-site verbatim insertion: True.
  graph.js == base graph.js minus declared deletions ONLY: True.
  -> Moved content diff-EMPTY against the TRACK_BASE blob: PASS.
- The literal handoff-recipe joined check (`block1+block2 in main.js`) is
  False BY CONSTRUCTION: the recipe concatenates the blocks without the blank
  separator the same handoff mandates between them at the insertion site
  (see D1). The per-block containments + exact-file reconstructions are the
  stronger, equivalent oracle and what the frozen-lock table names
  ("moved content byte-identical against the TRACK_BASE blob").

### Gate 7 - theme_token_check --check-added --base TRACK_BASE (invocation #12)

- Verbatim output (4 lines):
  - `crates/ui/assets/index.html: 6 hex literals exceeds frozen baseline count 4 (update tools/theme_token_baseline.json deliberately in this commit)`
  - `crates/ui/assets/main.js: 1 hex literals exceeds frozen baseline count 0 (update tools/theme_token_baseline.json deliberately in this commit)`
  - `crates/ui/assets/main.js: new hex literal #8250 added since 5b4da58616129b2ef57f19e2bc81d28094f67a22`
  - `crates/ui/assets/traversal.js: 1 hex literals exceeds frozen baseline count 0 (update tools/theme_token_baseline.json deliberately in this commit)`
- Added-literal scan (`git diff <base> -- assets`, the base-relative half E5
  speaks of): EXACTLY the one sanctioned `#8250` phantom line in `main.js` -
  the entity `&#8250;` of the moved `renderThreadBar` separator (E5/F3a). Zero
  other added literals, zero other files: V4 holds.
- The other 3 lines belong to the tool's base-independent frozen-count scan
  (working tree vs pinned `tools/theme_token_baseline.json`): `main.js: 1 > 0`
  is the count-twin of the SAME sanctioned entity (misread travels with the
  verbatim move); `index.html 6 > 4` and `traversal.js 1 > 0` are INHERITED
  drift - both files byte-identical to TRACK_BASE (clean in git status) and
  the scan reads only the working tree, so both lines fire identically at
  base. Name-identity survival per E4 logic. Declared D3.
- Not "fixed" per E5/F3a and the user dispatch ("verbatim-move wins");
  `tools/theme_token_baseline.json` NOT touched (golden baseline; explicit
  approval required; V2 locks it byte-identical).

### Gate 9 - landing commit (invocation #13)

- `python tools/commit_scoped.py --scope crates/ui/assets/graph.js --scope
  crates/ui/assets/main.js --scope crates/ui/tests/ui_test/static_html.rs
  --scope crates/ui/tests/ui_test/static_html_sections.rs --scope
  crates/ui/tests/ui_test/node_verdict_spine.rs --scope crates/ui/tests/
  ui_test/edge_verdict_story.rs --scope tasks/ui-degree-opacity-ramp-tr-
  journal.md -m "refactor(ui): move heat-style and thread/path blocks to
  main.js; dedicated graph.js line-budget lock"` -> commit `3c426eb`.
- numstat: graph.js 0/142 (deletions ONLY), main.js 142/0 (insertion ONLY),
  edge_verdict_story 1/1, node_verdict_spine 3/3, static_html 6/6 (net-zero,
  E3), static_html_sections 16/9, journal in-set. Exactly the touch-set,
  nothing else.
- Gate 8 (invocation #14): `python tools/encoding_guard.py --staged` exit 0,
  silent, over the index the scoped commit leaves behind (D6 re staging
  order). Second `--staged` run (invocation #15) precedes the docs rider
  commit (invocation #16: scope = the track's planning artifacts + journal
  tail; foreign untracked `ui-traversal-back-button` docs deliberately NOT
  committed - report-only).

## Dated D-rows (2026-10-09; declared, not hidden)

- D1: the handoff's verbatim-move recipe concatenates Block1+Block2 with no
  separator, contradicting its own insertion spec (blank line between the
  blocks at the single site); the literal joined-substring check cannot pass
  any spec-conformant landing. Corrected oracle run: per-block byte-containment
  + exact whole-file reconstructions (base blob + declared deletions / +
  declared single-site insertion), all True. Worktree CRLF normalized to the
  LF commit representation (`core.autocrlf=true`; blobs pure LF, worktree pure
  CRLF, measured). Reviewer: re-run equivalently.
- D2: handoff C1 insertion text lists three added blanks (143 lines) while its
  own budget math (604+142=746, 1051-142=909) and no-double-blank rule pin a
  142-line layout; landing follows the math; the post-state the handoff
  itself declares is matched verbatim (seams measured). main.js = 746.
- D3: Gate 7 base-relative added-scan == exactly the sanctioned `#8250`
  phantom (E5/F3a). The tool also fires its base-independent frozen-count
  scan: main.js `1 > 0` (same sanctioned entity's count twin) plus
  index.html `6 > 4` and traversal.js `1 > 0` - inherited drift, byte-for-byte
  present at TRACK_BASE (both files untouched here; scan reads worktree
  only). E5's "EXACTLY ONE line" prediction predates these two; per
  name-identity acceptance (E4 principle) they survive untouched; baseline
  JSON is a golden fixture - not fixed here.
- D4: RGB-Green "passed = Step 0 passed + 1" measures +2 (+220): the new
  budget test plus the overlay test returning green BY FIX, which the same
  acceptance paragraph demands; name-identity of the failing set holds.
- D5: gate-count itemization - declared 9; 16 actual distilling-entry
  invocations: 9 declared gates + hygiene exit-code probe (#3, Step 0.4
  requires the number), refactor re-run (#6, prescribed by RGB step 3),
  newline-style diagnostic (#10, required to run the byte proof correctly
  under autocrlf), second encoding_guard --staged (#15, second commit), docs
  rider commit (#16, orchestrator-dispatched planning-artifact convergence).
  Raw `git` and direct temp-script execution are permission-denied here;
  every git/proof observation routed through `python tools/distill.py --`.
- D6: Gate 8's "--staged before committing" is unexecutable before the FIRST
  commit because commit_scoped owns staging (adds + commits in one call);
  --staged ran immediately after the scoped commit (index intact under
  `commit --only`, so it inspects exactly the committed blobs) and before
  the docs commit.

## Close

- Post-state: graph.js 909 (< 990 lock, < 1000 R2), main.js 746,
  static_html.rs 999 (E3 net-zero), static_html_sections.rs 854 (< 1000).
- Scoped ui suite: `220 passed / 1 failed / 0 ignored / 2 suites`; failing
  set == {inherited F1 `test_inspector_panel_is_wider_and_collapsible`}.
- Hygiene exit 1 carrying ONLY the inherited server.rs pair; graph.js
  finding gone BY FIX. `ui_smoke: PASS`. encoding_guard --staged exit 0.
- Out-of-scope observations (report-only): untracked foreign docs
  `tasks/ui-traversal-back-button-execution-results.md` and
  `tasks/ui-traversal-back-button-t1-verdict.md` left in the tree (closed
  track's artifacts; not this touch-set); the frozen-count drift pair on
  index.html/traversal.js (D3) wants a baseline-reaffirmation decision by
  whoever owns `tools/theme_token_baseline.json`.



