# T-R Handoff - ui-degree-opacity-ramp: graph.js budget relief by verbatim relocation (lands FIRST)

Authored 2026-10-09 by the planner role. Unregistered post-M28 track per
kickoff: NO M-number; `tasks/milestone_ledger.md` stays numbering authority.
This is installment 1 of TWO SERIALIZED handoffs (Option B, amendment
2026-10-09). The ramp itself (T1) starts only on this handoff's green
landing commit.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract; zero verbal briefing needed).
2. `tasks/ui-degree-opacity-ramp-kickoff.md` - the Amendment 2026-10-09
   section SUPERSEDES the preconditions/sizing rows above it.
3. `tasks/ui-degree-opacity-ramp-shape.md` - the Amendment 2026-10-09
   section SUPERSEDES earlier rows; the Decided-contract section belongs to
   T1, not to T-R.
4. This handoff's dated errata (E1-E3, at the bottom) are binding where they
   contradict the shape: the shape was authored against coordinates its own
   re-anchor table got wrong; the tree wins.

## Dispatch precondition (blocking)

- Clean tree at the recorded start rev `TRACK_BASE` = CURRENT `main` HEAD
  (the `5b4da58`+ lineage). The design baseline was `20e3fb8` ("fixed
  traversal view"), but `ui-traversal-back-button` T1 LANDED APPROVED
  mid-registration 2026-10-09 (landing `9c0cd0c` + journal rider `5b4da58`,
  its TRACK_BASE `20e3fb8`; see its ledger row) - the design order holds,
  just shifted: T-R is first AMONG THE RAMP TRACK'S handoffs and rides after
  back-button. `9c0cd0c`'s touch-set excluded every file and anchor region
  this handoff reads or edits (its I3/I7 locks held graph.js/main.js/
  pre-existing tests diff-EMPTY), so all coordinates below remain valid.
  Before mutating, match every before-string byte-verbatim against the
  committed blobs; any mismatch -> STOP and report (do not silently rebase
  onto moved content).
- Known-red base is EXPECTED (erratum E4; see Step 0). At HEAD:
  `crates/ui/assets/graph.js` is 1051 lines - RED on the budget assert
  (`crates/ui/tests/ui_test/static_html_sections.rs:214-217`, `<= 998`,
  stale "(cap 990)" message intact), RED on the R2 1000-line hygiene
  finding, PLUS TWO findings T-R does NOT own and must NOT touch: the
  inherited test red `static_html_sections::test_inspector_panel_is_wider_
  and_collapsible` (deferred row "ui-traversal-back-button F1": the
  `onclick="toggleInspector()"` control vanished from `index.html` in the
  operator WIP; handler orphaned at `main.js:388`; operator disposition)
  and `hygiene_check` findings `server.rs R2 1120 > 1000` +
  `server.rs:764 R1 api_graph_traversal 161 > 100` (deferred row "F2";
  `crates/ui/src/**` is outside this touch-set). T-R's acceptance is
  name-identity shrinkage: the budget red and the graph.js R2 finding are
  GONE BY FIX; the F1 test red and the server.rs hygiene pair remain,
  byte-for-byte inherited.

## Serialization (hard; ledger row records it)

- NEVER parallel with any other handoff touching `crates/ui` (tests or
  assets): exactly one `crates/ui` landing commit in flight on `main` at a
  time. The `ui-traversal-back-button` T1 handoff landed APPROVED
  (`9c0cd0c` + rider `5b4da58`) BEFORE this handoff is dispatched - the
  order constraint is already satisfied, no action needed, and that track
  is closed to this handoff (do not touch, dispatch, or re-open it from
  here). Any FUTURE `crates/ui` landing - notably the PLANNED
  `ui-extraction-card-clarity` T-A/T-B (shares `index.html`/`styles.css`
  and the `ui_test/main.rs` mod roster) - serializes after this handoff's
  landing and re-anchors on its own start rev. Ramp T1
  (`tasks/ui-degree-opacity-ramp-t1-handoff.md`) dispatches ONLY from THIS
  handoff's APPROVED landing commit.

## Objective and invariants

Bring `graph.js` back under the 998/998-line locks by MOVING (not rewriting)
two cohesive blocks into `main.js`, and relocate the line-budget assert into
a dedicated, findable lock test with an honest message. This discharges the
ledger Deferred-findings row "graph.js line-count budget lock hides inside
`test_extraction_impact_preview_overlay`".

Preserve, falsifiably:
- V1 Behavior byte-identical: the moved blocks change no byte of logic.
  Moved content must be diff-EMPTY against the `TRACK_BASE` blob (proof in
  the Gates section).
- V2 Roster zero: `index.html`, `server.rs`, `export.rs`, `theme_contract.rs`
  untouched; `main.js` is already routed (`index.html:599`);
  `tools/ui_smoke.py` byte-identical (route table `:61-70` unchanged).
- V3 Call-time resolution: classic-script globals resolve when clicked, not
  when parsed; `main.js` loads after `graph.js` (`index.html:593/:599`) and
  every moved caller (`onclick="jumpToCrumb(i)"`,
  `onclick="illuminatePath(...)"` at `graph.js:641`, `graph.js:102/:775/:846`
  `renderThreadBar()`, `main.js:361`, `index.html:98`) runs post-load. The
  `main.js` -> `graph.js` `renderThreadBar()` call exists TODAY
  (`main.js:361`); the move only reverses that direction's host. `main.js`
  already owns `let threadHistory = []` (`main.js:23`).
- V4 Zero hex / token-safe: moved bodies carry no 3/6-hex runs
  (`&#8250;` is a 4-digit run, rejected by `theme_js_zero_hex.rs`); all
  `var(--...)` refs already declared. `theme_js_zero_hex.rs` unedited.
- V5 No hunk inside the degree-bug territory: `filterAndExpandSubgraph`
  (`graph.js:895-1023`), `stepDegree`, `toggleAllCapped`, `setNodeCap` stay
  byte-identical; `ui.traversal_view_degree_setting_bug` stays OPEN.
- V6 New budget lock caps at 990 with a message that names its own number.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ui-degree-opacity-ramp-tr-journal.md` (checkpoint journal; create
  it; always in-set)
- `crates/ui/assets/graph.js` (deletions ONLY - see erratum E1 ranges)
- `crates/ui/assets/main.js` (insertion ONLY, one site)
- `crates/ui/tests/ui_test/static_html.rs` (3 assertion rewrites, net-zero
  lines - see erratum E3)
- `crates/ui/tests/ui_test/static_html_sections.rs` (assert relocation +
  2 assertion rewrites)
- `crates/ui/tests/ui_test/node_verdict_spine.rs` (3 assertion rewrites)
- `crates/ui/tests/ui_test/edge_verdict_story.rs` (1 assertion rewrite)

You MUST NOT touch: `crates/ui/assets/index.html`, `styles.css`,
`hotspots.js`, `traversal.js`, `api.js`, `preview.js`, `file-view.js`,
`provenance.js`, `theme.js`, `themes.css`, any file under `crates/**/src/**`
(assets ride `include_str!` - zero Rust edits), `tools/**`, `tests/**`
outside `crates/ui/tests`, any other test file, `Cargo.toml`/`Cargo.lock`.
No `git stash`, no branch change, no rebase.

## Step 0 - red-baseline capture (BEFORE any edit; this is the red baseline)

1. Record `TRACK_BASE` (`git rev-parse HEAD`, or have the orchestrator
   report it if your session's permissions deny git) in the journal.
   Dirty tree on touch-set files -> STOP.
2. Anchor-match: every before-string in this file must be found byte-verbatim
   in the committed tree (line numbers are advisory; before-strings bind).
   Any mismatch -> STOP.
3. Run `python tools/testrun.py --distill -- -p ariadne-ui` (timeout
   600000 ms). Record VERBATIM in the journal: canonical summary line,
   digest header, capture path, and the FULL failing-test-name set read
   from `target/testrun-capture.txt`. Expected at HEAD (erratum E4):
   exactly `ui_test::static_html_sections::test_extraction_impact_preview_
   overlay` (T-R's target) and `ui_test::static_html_sections::
   test_inspector_panel_is_wider_and_collapsible` (inherited deferred row
   F1 - NOT T-R's to fix). Record whatever you MEASURE (standing ledger
   errata - never reconcile against guessed numbers). Label it
   `BASELINE FAILING SET (red base; budget test = T-R's target, inspector
   test = inherited F1)`.
4. Run `python tools/hygiene_check.py`. Expected exit NONZERO with three
   findings: `graph.js R2 1051 > 1000` (T-R's target), plus the inherited
   pair `server.rs R2 1120 > 1000` and `server.rs:764 R1
   api_graph_traversal 161 > 100` (deferred row F2; out of touch-set).
   Journal the finding names. T-R's acceptance removes the `graph.js`
   finding BY FIX; the `server.rs` pair must survive name-for-name
   untouched.

## What to build

### C1 Verbatim move (M18 verbatim-move precedent; ranges per erratum E1)

Block 1 - heat-style cluster, `graph.js` lines 445-499 (content), deleting
lines 445-500 INCLUDING the trailing blank 500:
- 445-447 comment `// Tri-state classification of a reachable include
  dependency ...` (the three comment lines open verbatim with
  `    // Tri-state classification of a reachable include dependency against the`)
- `function classifyDependency(depId, collateralCount, j)` (448-456)
- blank + comment + `function appendVerdictSpine(el, tier)` (458-473)
- blank + comment + `function styleDependencyCard(el, tier, collateral, maxCollateral)` (475-490)
- blank + comment + `function verdictBadgeColor(tier, tierColor)` (492-499)

NOT part of the move: lines 501-502 (`// BFS over the full (unfiltered) edge
dataset ...` + `// Temperature/heat must use this set, ...`) - that comment
belongs to `computeFullDependencies`, which STAYS in `graph.js`. (Erratum E1:
the shape's "445-502" swallowed it; the tree wins.)

Block 2 - thread/path block, `graph.js` lines 671-755 (content), deleting
lines 671-756 INCLUDING the trailing blank 756:
- `function recordCrumb(id, label)` (671-680)
- blank + `function renderThreadBar()` (682-695)
- blank + `function jumpToCrumb(i)` (697-702)
- blank + `function findIncludePath(fromId, toId)` (704-720)
- blank + `function illuminatePath(fromId, toId)` (722-755)

After deletion, `graph.js` reads `... } else { restoreStandardLegend(); } }`
(blank) `    // BFS over the full (unfiltered) edge dataset...` at the Block-1
site, and `selectNode`'s close + one blank + `function clearInspector() {` at
the Block-2 site. No double blanks.

Insertion (single site, `main.js`): after `restoreStandardLegend`'s close
(line 230 `    }` immediately after `restoreStandardLegend()`'s
`.join('');` line; before the blank preceding
`    function showProgress(stageText, percent) {`) append: one blank, Block 1
content (55 lines), one blank, Block 2 content (85 lines), one blank. Moved
lines keep their exact 4-space house indentation.

Post-move budget math (verify, don't assume): `graph.js` 1051 - 142 = 909
(<= 990 lock, < 1000 R2); `main.js` 604 + 142 = 746 (< 1000). If your
measured `graph.js` count exceeds 990 or `main.js` hits 1000, STOP.

### C2 Budget-lock relocation (DISCHARGES the deferred ledger row)

In `static_html_sections.rs::test_extraction_impact_preview_overlay`
(fn at :153): delete the assert currently at :214-217 verbatim:

```rust
    assert!(
        ariadne_ui::GRAPH_JS.lines().count() <= 998,
        "graph.js must stay within the spine-plan line budget (cap 990)"
    );
```

Trim its comment (207-209) down to the sentence that still describes the
`ariadnePreview.clearPreview()` assert that REMAINS at :210-213; the
line-budget provenance moves into the new test's own comment.

Add a dedicated test right after `test_extraction_impact_preview_overlay`
(closes at :224), exactly:

```rust
#[test]
fn test_graph_js_line_budget() {
    // Relocated 2026-10-09 out of test_extraction_impact_preview_overlay
    // (it was a hidden lock in an unrelatedly-named test whose message
    // disagreed with its own number - deferred-ledger-row discharge,
    // ui-degree-opacity-ramp T-R). Cap 990: R2 hygiene wall is 1000 lines
    // for .js; this in-suite cap keeps headroom for the ramp feature.
    assert!(
        ariadne_ui::GRAPH_JS.lines().count() <= 990,
        "graph.js must stay within its line budget (cap 990; R2 hygiene wall is 1000)"
    );
}
```

### C3 Declared test amendments (exactly 9 assertion rewrites + 1
relocation - erratum E2 supersedes the shape's "exactly 3")

Each is a containment-object move `GRAPH_JS -> MAIN_JS` (or a slice
terminator); zero assertion weakening, zero line-count change in
`static_html.rs` (erratum E3: `static_html.rs` is 999 lines, AT the R2
ceiling - every edit there stays net-zero lines; put provenance in the
message string and the journal, not in added comment lines). The other three
test files have headroom and MAY carry a trailing `// relocated with
<fn> to main.js (T-R 2026-10-09)` comment.

| # | File:line | Before (substring of the assert) | After |
|---|---|---|---|
| 1 | static_html.rs:238 (+msg :239) | `GRAPH_JS.contains("function classifyDependency(")` | `MAIN_JS.contains("function classifyDependency(")`; message: `missing classifyDependency predicate helper (main.js after T-R move)` |
| 2 | static_html.rs:245 (+msg :246) | `GRAPH_JS.contains("appendVerdictSpine(")` | `MAIN_JS.contains("appendVerdictSpine(")`; message += ` (main.js after T-R move)` |
| 3 | static_html.rs:367-368 | `.find("function recordCrumb(")` / `.expect("recordCrumb function")` | `.find("function clearInspector(")` / `.expect("clearInspector function")` |
| 4 | node_verdict_spine.rs:25 | `graph.contains("appendVerdictSpine(")` | `ariadne_ui::MAIN_JS.contains("appendVerdictSpine(")` |
| 5 | node_verdict_spine.rs:29 | `graph.contains("verdict-spine")` | `ariadne_ui::MAIN_JS.contains("verdict-spine")` |
| 6 | node_verdict_spine.rs:33 | `graph.contains("heat-tier-")` | `ariadne_ui::MAIN_JS.contains("heat-tier-")` |
| 7 | edge_verdict_story.rs:130 | `graph.contains("appendVerdictSpine(")` | `ariadne_ui::MAIN_JS.contains("appendVerdictSpine(")` |
| 8 | static_html_sections.rs:310 | `GRAPH_JS.contains("heat-tier-")` | `MAIN_JS.contains("heat-tier-")` |
| 9 | static_html_sections.rs:314 | `GRAPH_JS.contains("heat-critical")` | `MAIN_JS.contains("heat-critical")` |

Machine-verified basis for E2: in `graph.js` the literals
`verdict-spine`, `heat-tier-`, `heat-critical` occur ONLY inside the moved
bodies (`:466`, `:486`, `:487`), and `appendVerdictSpine(` has exactly one
caller (`:488`, inside `styleDependencyCard`, co-moving). The shape counted
them as survivors; they break. Survivors CONFIRMED unedited (do not touch):
`node_verdict_spine.rs:45` `verdictBadgeColor(` (caller stays at
`graph.js:407`); `edge_verdict_story.rs:119/:126`; all `common::page_content()`
locks (union of all assets - breadcrumb test `:720/:724/:728/:732/:748`,
`illuminatePath` `:456`, page-level CSS checks); `clearSelection`/
`switchTab`/`setupCanvasPanZoom`/`renderSvg` slices (`function_body`'s
`\n    }` terminator bounds them to the untouched hosts);
`hotspot_direction.rs` (incl. `test_graph_delegates_inbound_heat_before_outbound_block`
- the `renderSvg` slice `:374-443` is untouched); `theme_js_zero_hex.rs`;
the asset byte-serve roster (`static_html.rs:806` area);
`test_script_tag_order_preserves_original_execution_order`;
`test_ui_noticeable_fading_elements` (that is T1's declared amendment, not
yours).

## Verbatim-move proof (gate + reviewer MUST)

Byte-equality of moved content against the base blob. Recipe (byte-level,
PowerShell-text-pipe forbidden):

```
python -c "import subprocess as sp
base=sp.run(['git','show','<TRACK_BASE>:crates/ui/assets/graph.js'],capture_output=True).stdout.splitlines(keepends=True)
cur=open('crates/ui/assets/main.js','rb').read().splitlines(keepends=True)
moved=b''.join(base[444:499])+b''.join(base[670:755])
assert moved in b''.join(cur), 'MOVED BLOCKS NOT BYTE-IDENTICAL'"
```

(`base[444:499]` = lines 445-499, `base[670:755]` = lines 671-755.) Journal
the PASS; the reviewer re-proves independently (erratum 2026-10-09: line
numbers shift under T1/T-R adjacency, content anchors and the blob diff are
binding). If your session cannot run python/git, report it - the proof
routes to the reviewer via `git show "<TRACK_BASE>:crates/ui/assets/graph.js"`.

## RGB protocol

1. Red (tests first): apply C2 + C3 WITHOUT moving any asset code. Run
   `python tools/testrun.py --distill -- -p ariadne-ui`. Expected failing
   set = Step 0 set MINUS `test_extraction_impact_preview_overlay` (its
   assert is gone) PLUS, each with its own first-failing message:
   - `static_html_sections::test_graph_js_line_budget` - "cap 990..." fires
     with 1051 > 990;
   - `static_html::test_ui_heat_map_tri_state_justification` - :238 fails
     ("missing classifyDependency predicate helper (main.js after T-R move)");
   - `node_verdict_spine::test_graph_js_builds_spine_and_flat_tiers` - :25;
   - `edge_verdict_story::test_edge_verdict_graph_wiring` - :130;
   - `static_html_sections::test_ui_heat_tier_outlines_are_flat` - :310.
   Amendment 3 (:367) is Red-neutral pre-move (the slice still contains the
   body; `clearInspector` already exists) and becomes load-bearing AT the
   move - it must land in this test commit, never with the move. Capture the
   set verbatim (must equal BASELINE - overlay + these five).
   A failing test must fail on assertions - compile is mandatory.
2. Green: perform C1 (deletions + single insertion). Re-run the same scoped
   command: failing set must equal exactly
   `{test_inspector_panel_is_wider_and_collapsible}` (the inherited F1 red
   rides through UNTOUCHED, same assert, same message - by-name identity
   against Step 0; everything else green); passed = Step 0 passed + 1 (the
   new budget test); quote the canonical summary line verbatim. Verify the
   move math (C1) and the verbatim-move proof.
3. Refactor: none permitted inside moved code (V1). Doc-comment polish only
   OUTSIDE moved ranges would violate diff-EMPTY - so refactor = journal +
   hygiene pass only. Re-run scoped suite (same expectation as Green). Run
   `python tools/hygiene_check.py`: expect exit NONZERO with EXACTLY the
   inherited `server.rs` R2 + R1 pair - the `graph.js` finding must be
   GONE (that is this handoff's hygiene acceptance; `exit 0` is unattainable
   while deferred row F2 stands and is NOT this handoff's job).

## Hygiene (review-enforced)

`python tools/hygiene_check.py`: the `graph.js` R2 finding must be GONE at
the end (it is part of the red base - the target); the run overall still
exits NONZERO carrying ONLY the inherited `server.rs` pair (erratum E4,
deferred row F2). Post-state: `graph.js` 909 < 1000,
`main.js` 746 < 1000, `static_html.rs` stays 999 (E3),
`static_html_sections.rs` 847 + ~14 < 1000. No renamed identifiers; moved
code keeps its names verbatim.

## Frozen locks

| Lock | Proof |
|---|---|
| Moved content byte-identical | Verbatim-move proof (Gates 6) against TRACK_BASE blob |
| `index.html`, `tools/ui_smoke.py`, `themes.css`, `theme.js`, `tools/theme_token_baseline.json`, `tests/fixtures/**`, all `crates/**/src/**`, `tools/**`, all assets except graph.js/main.js byte-identical | `git diff <TRACK_BASE>..HEAD -- <path>` EMPTY (reviewer); scoped commit scopes below make it structural |
| 998-assert removal is the ONLY test semantic relaxation and it LOWERS the cap to 990 | reviewer `git show` per commit: the deleted 4 lines vs the new dedicated test |
| Zero added hex; token locks green | `python tools/theme_token_check.py --check-added --base <TRACK_BASE>` == exactly the single sanctioned `#8250` phantom line in `main.js` (E5/F3a); `theme_js_zero_hex.rs` unedited (V4) |
| No mojibake; pure-ASCII authored content | `python tools/encoding_guard.py --staged` exit 0 before each commit (`--all` on the reviewer hop) |
| Degree-bug territory untouched (V5) | reviewer per-commit `git show` oldest-first, hunk-header check |
| Roster frozen (V2) | `git diff` EMPTY on V2's files; `ui_smoke: PASS` on unchanged table |

## Gates (exact commands; implementer hop; timeout 600000 ms; distilling
entry points ONLY; journal itemizes the count - declare any overshoot as a
dated D-row per the M28 T0b law)

1. Step 0 capture: `python tools/testrun.py --distill -- -p ariadne-ui`
   (journal: BASELINE FAILING SET + hygiene red-base exit code, before any
   edit).
2. Red capture: same command (failing set = BASELINE - overlay + the five
   named in RGB step 1).
3. Green capture: same command (failing set == exactly
   `{test_inspector_panel_is_wider_and_collapsible}` (inherited F1);
   canonical line verbatim;
   digest header + capture path).
4. `python tools/hygiene_check.py` - NONZERO exit carrying EXACTLY the two
   inherited `server.rs` findings; the `graph.js` finding must be gone
   (erratum E4; `exit 0` is unreachable while deferred row F2 stands).
5. `python tools/distill.py -- python tools/ui_smoke.py` - reports
   `ui_smoke: PASS` on the unchanged route table.
6. Verbatim-move proof (python recipe above or report-to-reviewer) - PASS.
7. `python tools/theme_token_check.py --check-added --base <TRACK_BASE>` -
   expected output: EXACTLY ONE sanctioned phantom line naming
   `main.js: new hex literal #8250` - the entity `&#8250;` in the moved
   `renderThreadBar` thread-separator line. `HEX_LITERAL` in
   `tools/theme_token_check.py:42-47` has no `&#\d+;` exemption and the
   per-file added-scan fires because base `main.js` lacks the string while
   base `graph.js` had it (misread mechanism deferred row "F3a"; the tool
   baseline already sanctions four such entity misreads). It is a numeric
   character reference, not a color. Journal the single line verbatim;
   ANY other violation line, any other file, or any other literal =
   blocker (erratum E5).
8. `python tools/encoding_guard.py --staged` - exit 0 before committing
   (commit_scoped owns staging; `--all` is the reviewer hop).
9. Commit ONE landing commit:
   `python tools/commit_scoped.py --scope crates/ui/assets/graph.js --scope crates/ui/assets/main.js --scope crates/ui/tests/ui_test/static_html.rs --scope crates/ui/tests/ui_test/static_html_sections.rs --scope crates/ui/tests/ui_test/node_verdict_spine.rs --scope crates/ui/tests/ui_test/edge_verdict_story.rs --scope tasks/ui-degree-opacity-ramp-tr-journal.md -m "refactor(ui): move heat-style and thread/path blocks to main.js; dedicated graph.js line-budget lock"`
   (never `git add .`; if your permissions deny `commit_scoped.py`/`git`,
   STOP and report - commit duty routes to the orchestrator per the standing
   allowlist errata row).

Gate count: 9 invocations (<= 10). If 3 is insufficient because a flake or
lock forces a re-run, declare the overshoot (dated D-row), do not hide it.

## Sizing verification (planner, re-verified against the tree 2026-10-09)

- Scope: 1 crate dir (`crates/ui`), 4 asset/test files + 2 test files, 6
  paths + journal. Within the law.
- Produced lines: moved bytes x2 = 142 deletions + 142 insertions = 284;
  test rewrites ~34 (9 sites x ~2 lines + comment trim + new test body);
  new lock function ~14; journal ~25. Total ~357 vs the ~1200 ceiling
  (shape estimate ~360 - confirmed).
- Gates: 9 invocations vs the ~10 ceiling.
- Post-state margins: graph.js 909 (81 under the new 990 lock, 91 under
  R2); main.js 746 (254 headroom).

## Dated errata (2026-10-09, planner authoring-time adjudication; per the
ui-extraction-card-clarity precedent the handoff is binding where the shape
is wrong)

- E1: shape T-R Block 1 says "graph.js:445-502". Lines 501-502 are the
  2-line comment of `computeFullDependencies` (which stays). Binding Block-1
  content range: 445-499 (delete 445-500 incl. trailing blank).
- E2: shape says "declared amendments (exactly three)" and lists the
  survivor locks. Machine enumeration: 9 assertion rewrites + 1 relocation
  across 4 files (table C3). Sole-occurrence literals inside moved bodies
  (`appendVerdictSpine(`, `verdict-spine`, `heat-tier-`, `heat-critical`)
  break the shape's survivor claim at static_html.rs:245,
  node_verdict_spine.rs:25/:29/:33, edge_verdict_story.rs:130,
  static_html_sections.rs:310/:314. All are object moves, not weakenings.
- E3: shape says each amendment carries "a rationale comment".
  `static_html.rs` is 999 lines, AT the R2 1000 wall (its own comment at
  :753-757 records why).   In THAT file rationale lives in the assert message
  strings and the journal - net-zero lines. The other three files may carry
  trailing comments.
- E4 (added at registration 2026-10-09, concurrent-landing adjudication):
  while this handoff was being authored, `ui-traversal-back-button` T1
  landed APPROVED (`9c0cd0c` + rider `5b4da58`, own TRACK_BASE `20e3fb8`)
  and its verdict registered deferred rows F1/F2/F3a. Consequences binding
  here: the red base at T-R's actual dispatch is TWO test reds (the budget
  assert - T-R's target - plus `test_inspector_panel_is_wider_and_
  collapsible`, F1, inherited) and THREE hygiene findings (graph.js -
  T-R's target - plus the `server.rs` pair, F2, inherited and out of
  touch-set); acceptance is therefore name-identity shrinkage, not empty
  sets (Step 0 / RGB / Hygiene / Gates / Reviewer hop amended accordingly).
  The kickoff/shape "base statically RED on both gates" wording predates
  F1/F2 being named and is superseded by those ledger rows.
- E5 (2026-10-09, F3a interaction): the theme gate cannot be "silent" for
  a verbatim move - `&#8250;` moving into `main.js` is a sanctioned phantom
  per deferred row F3a (Gate 7 spells the exact accepted line). Verbatim
  move (V1) wins over tool cosmetics; do NOT "fix" the entity.

## Discipline clauses

- UTF-8: never `Set-Content`/`Out-File`/`>`/`Get-Content -replace`;
  Read/Edit/Write tools only; authored content pure ASCII (AGENTS.md
  File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`, raw workspace census)
  are a reviewer blocker in an implementer session - route through the
  distilling entry points (`docs/specs/tool-output-distillation.md`).
- After EACH gate, append a 1-3 line checkpoint to
  `tasks/ui-degree-opacity-ramp-tr-journal.md`: step + commit hash, the
  recorded red sets verbatim, any deviation, any anchor mismatch.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ui-degree-opacity-ramp-tr-handoff.md --scope crates/ui --cmd "python tools/testrun.py --distill -- -p ariadne-ui"`.

## Reviewer hop (fresh session; adjudicate via `review-gate`; re-run FRESH)

1. Scoped ui suite fresh: failing set == exactly
   `{test_inspector_panel_is_wider_and_collapsible}` (the inherited F1 red,
   UNTOUCHED by name + message - fixing it is the operator's F1 disposition,
   not this track's). The budget red must be gone BY FIX, not by
   deletion-of-test: the only deleted assert is the :214-217 pair and it
   reappears as the dedicated `test_graph_js_line_budget` capped at 990.
2. `git diff --name-only TRACK_BASE..HEAD` set-equality against the
   touch-set minus the journal; V2 file list diff-EMPTY.
3. Verbatim-move proof re-run independently against `TRACK_BASE`; moved
   ranges byte-equal.
4. Per-commit `git show` oldest-first: V5 hunk check; commit-vs-touch-set;
   E2 enumeration honored (exactly 9 rewrites + 1 relocation - a 10th
   semantic test edit is a blocker).
5. `python tools/hygiene_check.py` NONZERO carrying ONLY the inherited
   `server.rs` pair (graph.js finding gone - deferred rows F2/E4);
   `python tools/distill.py -- python tools/encoding_guard.py --all` exit 0;
   `theme_token_check --check-added --base <landing-parent>` == exactly the
   sanctioned single `#8250` phantom line in `main.js` (E5/F3a) - anything
   else is a blocker;
   `ui_smoke: PASS` (distill route).
6. Ledger/track note: if approved, the closeout runner archives the deferred
   row "graph.js line-count budget lock hides inside
   test_extraction_impact_preview_overlay" to `tasks/ledger_archive.md` with
   a dated discharge note pointing at the landing commit (the row names the
   heat/legend extraction as its target - this handoff IS that
   installment per the kickoff amendment 2026-10-09).

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Step 0 baseline sets (test-run +
   hygiene) + Red set + Green canonical line (verbatim) + capture path.
3. Verbatim-move proof verdict + final graph.js/main.js line counts.
4. Deviations + gate-count itemization (declare overshoots as D-rows).
5. Out-of-scope observations (report-only). Never paste raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the Touch-set above, which
therefore lists every file this task may legitimately change. T1
(`tasks/ui-degree-opacity-ramp-t1-handoff.md`) dispatches ONLY on this
handoff's APPROVED landing commit on main.
