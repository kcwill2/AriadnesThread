# Help Center Readability Handoff — ad-hoc UI task: make #helpCenterModal scannable

Binding inputs (read in this order, targeted reads only):
1. This file, `tasks/help-center-readability-handoff.md`.
2. `crates/ui/assets/index.html` — the `#helpCenterModal` block starting line 423 (working tree is DIRTY from an earlier uncommitted session; anchors may drift a few lines, re-anchor by id/class not by these numbers). NOTE: the mega-paragraph (currently line 461, 2350 chars) is TRUNCATED by the Read tool at 2000 chars. Its verified tail from char 1700 onward is quoted in full in section C below, so you do not need bash to recover it — but verify with a bounded dump if you wish: `python tools/distill.py --kind generic -- python -c "import io; l=io.open('crates/ui/assets/index.html',encoding='utf-8').read().splitlines(); [print(len(x), x[1700:]) for x in l if 'Collateral Heat Map:' in x]"`.
3. `crates/ui/assets/main.js` lines 469-511 (`openHelpCenter`, `closeHelpCenter`, `openHelpCenterTab`, keyboard handler) — READ ONLY, do not modify.
4. `crates/ui/assets/styles.css` lines 1228-1275 (`.modal`, `.modal-overlay`, `.modal-header`, `.modal-close`) and lines 1-30 (global themed-scrollbar rules) — model new rules on this style; existing tokens only.
5. Contract lock sources you must keep green (read the assertion lists):
   - `crates/ui/tests/ui_test/static_html.rs` `test_onboarding_and_help_content` (~line 461) plus every `INDEX_HTML`/`page.contains` assertion in that file (e.g. "Include Justification breakdown", "Proposed extractions section", "updateHeatLegend"|"Heat Map", "illuminatePath"|"findIncludePath").
   - `crates/ui/tests/ui_test/static_html_sections.rs` — INDEX_HTML locks "Preview after extraction"; page locks "After extraction:" and "beside the open list".
   - `crates/ui/tests/ui_test/edge_verdict_story.rs` `test_edge_verdict_legend_and_help_copy` — html locks: "system/stdlib headers are not analyzed", "red dashed" AND "0/N used".
   - `crates/ui/tests/ui_test/theme_contract.rs` — index.html must contain NO `<style` block and NO `rgb(`/`rgba(` literals; inline `<script>` bodies unchanged.
   - `crates/ui/tests/ui_test/gui_scrollbar_style.rs` — global scrollbars already themed; do not regress; do not add scrollbar rules.
6. `crates/ui/tests/ui_test/main.rs` (mod declarations) and `common.rs` (page_content helper + `css_rule_body` idiom in gui_scrollbar_style.rs) — mirror test style.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/help-center-readability-handoff.md (this file, created first)
- tasks/help-center-readability-journal.md (checkpoint journal; always in-set)
- crates/ui/assets/index.html (ONLY the `#helpCenterModal` block — no other element, id, or script tag)
- crates/ui/assets/styles.css (append a help-center rule block + `kbd` rule)
- crates/ui/tests/ui_test/help_center_readability.rs (new)
- crates/ui/tests/ui_test/main.rs (one `mod help_center_readability;` line)
You MUST NOT touch: main.js, graph.js, themes.css, api.js, AGENTS.md, any other test file, `.roomodes`, or anything outside `crates/ui` + the two tasks/ files. No `git stash`, no branch change. NOTE the working tree already carries UNCOMMITTED pre-existing edits (index.html, styles.css, api.js, AGENTS.md, .roomodes deletion) from a prior unreviewed session — do not revert or stash them; your scoped commit of index.html/styles.css will necessarily carry their hunks too; list any pre-existing hunks you notice in those two files in your report so the reviewer can adjudicate them.

## What to build

### A. Container: scroll + breathing room
- Modal markup gets an added class: `class="modal help-center"` (keep `id="helpCenterModal"`, keep JS-toggled `display:none` inline or via class — JS sets `style.display='block'`, so the modal must NOT depend on `display:flex` from CSS).
- styles.css: `.help-center { width: 720px; max-width: 92vw; max-height: 84vh; }` (replaces the inline width/max-width style attr).
- Tab rail + content wrapped in `.help-layout` (flex row; move the existing inline flex styles into the class).
- Right content pane gets `class="help-body"`; styles.css: `.help-body { overflow-y: auto; max-height: 70vh; font-size: 13px; line-height: 1.65; }`. Global themed scrollbars apply automatically.

### B. Tab rail: visible active state
Add to styles.css:
- `.help-tab-btn { background: transparent; border: none; text-align: left; color: var(--text-sub); font-size: 12px; padding: 6px 8px; border-radius: 6px; cursor: pointer; }`
- `.help-tab-btn:hover { color: var(--text-main); background: var(--bg-card-hover); }`
- `.help-tab-btn.active { color: var(--cyan); background: var(--cyan-wash-soft); font-weight: 600; }`
- `.help-tab-btn.active:hover { background: var(--cyan-wash); }`
Remove the inline `style="text-align:left; border:none; background:transparent;"` from the four tab buttons. KEEP: `onclick` handlers, `help-tab-btn`/`help-tab-content` classes, `#helpTab-<name>` ids, and button labels unchanged — main.js matches the active button via `textContent.toLowerCase().includes(tabName.slice(0,4))`, so labels must keep the prefixes tour/keyb/lege/reci. Header row: reuse existing `.modal-header`/`.modal-close` classes where they fit; keep `closeHelpCenter()` onclick.

### C. Legend tab restructure — structure only, zero copy loss
Verified FULL tail of the mega-paragraph from char 1700 (ASCII source uses &mdash; entities):
"...ed</code>) are partially needed. Grey arrows join two dependencies inside the selection's include closure: they are transitive dependencies of the selection, but no verdict is stated against it &mdash; verdicts exist only on the selection's own outgoing arrows. Each card's spine and its arrow carry the same verdict. These verdicts are root-relative and cover project includes only &mdash; system/stdlib headers are not analyzed or counted, so an arrow's badge never describes what the dependency itself includes. A collapsed side panel (Node Inspector or Source Code) leaves a vertical restore tab on the canvas' right edge that brings it back.</p>"
Re-emit the legend content as scannable sections: `<h5 class="help-section-title">` headings + short paragraphs + `<ul>/<li>` one fact per line. Suggested sections (adapt to actual text):
- "Collateral Heat Map" — tiers (Cold/Teal, Green, Amber, Orange, Red), fading hides unrelated nodes, heat badges (+XX col).
- "Card verdict spines" — green spine = every supplied symbol used (needed); amber = partly needed; red = supplies nothing, the include can go; no spine = no verdict data; card fill stays pure collateral heat (flat tier color, never a glow; glow marks selection and the root only).
- "Selection verdict arrows" — green solid N/N used; red dashed 0/N used (droppable or slimmable); amber J/T used (partly needed); grey arrows join two dependencies inside the selection's include closure — transitive dependencies, no verdict vs selection; verdicts exist only on the selection's own outgoing arrows; each card's spine and its arrow carry the same verdict; verdicts are root-relative and cover project includes only (the stdlib caveat sentence, verbatim lock below); an arrow's badge never describes what the dependency itself includes.
- "Include Justification breakdown" — inspector expands the top contributing header of the selected file: symbols it is included for, headers those symbols need, collateral symbols dragged along.
- "Preview after extraction" — marks what would stop including (red dashed), what still arrives via other paths (amber dashed), what the slim edge keeps (green); spines hide while previewing; hypothetical after-state called out by warning banner + amber dashed canvas frame; extracted result is a violet dashed PROPOSED header node with placeholder name, proposed edges fan out violet, slimmed consumer edge turns amber dashed.
- "Path Illumination" — keeps its heading.
- "Hotspots" — keeps its heading; text must retain "beside the open list" and "Proposed extractions section" verbatim, and the existing fire/lightbulb emoji phrases byte-identical.
- "Restore tab" — a collapsed side panel (Node Inspector or Source Code) leaves a vertical restore tab on the canvas' right edge that brings it back. (Preserved from the old paragraph's tail.)
- Also keep the short existing paragraphs: Node Colors, Theme (Light/Dark/Dark (Steampunk), persistence, semantic colors keep meaning in all three themes), Edges (solid vs dashed), Closure, Degree.
HARD COPY LOCKS (tests assert on full INDEX_HTML; must survive verbatim): "Collateral Heat Map", "Cold/Teal", "Include Justification breakdown", "Preview after extraction", "red dashed", "0/N used", "system/stdlib headers are not analyzed", "Path Illumination", "beside the open list", "Proposed extractions section", "ALL-WITH-CAP", "Direction", "Finding & Decoupling Header Bloat", "Isolating Focused Subgraphs".
Density target: NO text block between `</p>`/`</li>`/`</h5>` exceeds 400 chars. OPTIONAL: verdict rows may use swatch spans `<span class="help-swatch justified"></span>` etc. with styles.css `.help-swatch { display: inline-block; width: 10px; height: 10px; border-radius: 2px; }`, `.help-swatch.justified { background: var(--green); }`, `.help-swatch.mixed { background: var(--amber); }`, `.help-swatch.unused { background: var(--red); }`. NO new hex literals, no new colors (verdict colors are semantically locked across themes).
Also fix the hygiene violation itself: every authored HTML source line must stay under 2000 chars (split the rewritten content across many short lines — this clears the outstanding >2000-char line finding).

### D. Shortcuts tab: real key rows
Keep every shortcut and `<kbd>` label exactly (Ctrl+K, /, down-arrow from Search, up/down, Enter, left/right, ?, Shift+/, F1, Esc, ArrowLeft/ArrowRight + full navigator-row description). Rows: `<div class="help-shortcut-row"><span class="help-keys">...</span><span class="help-desc">...</span></div>`. styles.css:
- `kbd { background: var(--bg-card); border: 1px solid var(--border-light); border-bottom-width: 2px; border-radius: 4px; padding: 1px 5px; font-size: 11px; color: var(--text-main); }`
- `.help-shortcut-row { display: grid; grid-template-columns: 180px 1fr; gap: 12px; align-items: baseline; }`
- `.help-desc { color: var(--text-sub); }`

### E. Recipes tab: steps, not arrows
Four recipes, each a title + `<ol>` of `<li>` steps. Titles locked verbatim: "Finding & Decoupling Header Bloat", "Triage Worst Offenders", "Isolating Focused Subgraphs", "Breaking Circular Dependencies". Split the arrow-chained prose into discrete steps preserving EVERY clause (header-bloat: select a header / inspect Collateral Breakdown / click a heavy contributor to illuminate the include path / refactor or forward-declare; triage: open the Hotspots panel from its left-edge tab / switch metric (Collateral burden, Fan-in, Fan-out, Closure burden, Cycle participation) / click a row to inspect its heat map / optionally set a directory scope chip first; subgraphs: set Direction (Dependencies / Consumers) / step Degree (1 deg, 2 deg) or use ALL-WITH-CAP with safety node caps; cycles: toggle Highlight Cycle / inspect circular links / use Pathfinder to see the loop).

### F. Typography
`.help-body p { margin: 0 0 10px; }`, `.help-body h5.help-section-title { font-size: 12px; font-weight: 600; color: var(--cyan); margin: 16px 0 6px; }`, `.help-body ul, .help-body ol { margin: 0 0 10px; padding-left: 18px; }`, `.help-body li { margin-bottom: 4px; }`.

## RGB protocol
1. Red FIRST: write `crates/ui/tests/ui_test/help_center_readability.rs` + mod line in main.rs; run gate 1; capture failing test names.
   Tests (string-asset style; copy the local `css_rule_body` helper pattern from gui_scrollbar_style.rs into your file; keep a small `fn tab_slice(html, start_marker, end_marker)` helper; every fn < 100 lines):
   - `test_help_body_scrolls`: styles.css `.help-body` body contains `overflow-y: auto` and `max-height`; `.help-center` body contains `max-height`; INDEX_HTML contains `class="modal help-center"` and `class="help-body"`.
   - `test_legend_tab_has_no_wall_of_prose`: slice from `id="helpTab-legend"` to `id="helpTab-recipes"`; strip tags; every text block between `</p>`, `</li>`, `</h4>` or `</h5>` boundaries <= 400 chars; slice contains >= 5 `<h5` headings.
   - `test_legend_copy_locks_survive`: INDEX_HTML contains every HARD COPY LOCK string verbatim (use `\u{2014}` escapes where the locked literal contains an em dash, as edge_verdict_story.rs does).
   - `test_active_tab_state_is_styled`: styles.css contains `.help-tab-btn.active {` whose body contains `var(--cyan)`.
   - `test_kbd_and_shortcut_rows_are_styled`: styles.css contains `kbd {` and `.help-shortcut-row`; the shortcuts slice (from `id="helpTab-shortcuts"` to `id="helpTab-legend"`) contains >= 12 `<kbd`.
   - `test_recipes_are_step_lists`: recipes slice (from `id="helpTab-recipes"` to `quickOpenPanel`) contains >= 4 `<ol` and all four locked titles.
2. Green: implement A-F; re-run gate 1 until green. NEVER weaken an assertion to pass; NEVER modify or delete any existing test.
3. Refactor: group CSS under one `/* Help & Reference Center */` comment block near the modal rules; honest names; no banned identifiers (tmp/res/data/foo); re-run.

## Hygiene
Test fns small; new test file well under 1000 lines. styles.css baseline is 1493+ lines and EXEMPT from R2 (do not split or shrink it). `python tools/hygiene_check.py` must stay exit 0. New HTML authored with one element/section per line, no line > 2000 chars.

## Frozen locks
- All existing tests byte-identical except the one mod line in main.rs.
- main.js, graph.js, themes.css, api.js byte-identical.
- No new `<style`/`<script>` blocks in index.html; no `rgb(`/`rgba(` literals in index.html.
- `python tools/theme_token_check.py --check-added --base d7399cd` — if it flags lines predating your work (pre-existing dirty hunks), report them, do not fix; YOUR added lines must be hex-free.

## Gates (exact commands; timeout 600000 ms each; scoped only)
1. Red, then green: `python tools/testrun.py --distill -p ariadne-ui` (verify package name in crates/ui/Cargo.toml; tests import `ariadne_ui`; adjust scope flag if the name differs and cite what you ran). Quote the digest header + canonical `N passed / M failed / K ignored / S suites` line verbatim in the report, both red and green runs.
2. `python tools/hygiene_check.py` — exit 0.
3. `python tools/theme_token_check.py --check-added --base d7399cd` — exit 0 for your added lines (see frozen locks on pre-existing lines).
4. `python tools/encoding_guard.py --staged` — exit 0 before each commit.
5. Commit: `python tools/commit_scoped.py --scope "tasks/help-center-readability-handoff.md" --scope "tasks/help-center-readability-journal.md" --scope "crates/ui/assets/index.html" --scope "crates/ui/assets/styles.css" --scope "crates/ui/tests/ui_test/help_center_readability.rs" --scope "crates/ui/tests/ui_test/main.rs" -m "fix(ui): restructure help & reference center for readability"` (NEVER `git add .`). One or two commits, all inside the touch-set. api.js/AGENTS.md/.roomodes stay uncommitted — leave them dirty.

## Discipline clauses
- UTF-8 hazard (this repo, PowerShell 5.1): NEVER write files via Set-Content/Out-File/`>`/Get-Content-replace pipelines; use your Write/Edit tools only. Authored content pure ASCII — ASCII entities (`&mdash;`, `&rarr;`, `&#128293;`) for NEW punctuation; DO NOT alter existing non-ASCII bytes (fire/lightbulb/cross emoji in the Hotspots text and the close button) — carry them over byte-identical inside rewritten blocks.
- Touch only the touch-set; on discovering out-of-scope breakage: report, do not fix.
- Journal checkpoint after each Green gate.
- If context exhaustion looms: STOP, return a disk-state inventory (git status, last commit, journal tail); a dead session is never resumed.

## Report back (final message)
1. Commit hash(es) + `git show --stat` stat lines only. 2. Red evidence: first failing test names. 3. Green evidence: distill digest header + canonical summary line verbatim + capture path. 4. Deviations + reason (incl. list of any pre-existing dirty hunks you noticed in index.html/styles.css that rode along in the commit). 5. Out-of-scope observations (report-only). Never paste raw cargo/git dumps.
