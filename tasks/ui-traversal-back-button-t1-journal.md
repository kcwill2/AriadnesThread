# T1 Journal - ui-traversal-back-button (traversal panel Back control)

Handoff: `tasks/ui-traversal-back-button-t1-handoff.md`. Implementer session
started 2026-10-09 ~00:33 UTC. Fresh session; bootstrap via
`python tools/checkpoint_guard.py status`.

## Checkpoints

### Step 0 - baseline (pre-implementation)

TRACK_BASE 20e3fb832263e0459ec05510c142dfc9129d74fc
(branch main, tree clean: dirty=0, untracked=0). The previously uncommitted
traversal-view WIP landed as `20e3fb8 "fixed traversal view"` (operator
update 2026-10-09 ~00:31 UTC). No touch-set file is dirty; all verbatim
before-strings (C1-C4) verified byte-exact against the landed tree
(index.html:72/:388, traversal.js:398-401/:406, hotspots.js:619,
styles.css:1793-1805). No anchor mismatch -> dispatch proceeds.

Gate 1 command: `python tools/testrun.py --distill -- -p ariadne-ui`
Digest header:
`### cargo test FAIL (exit 1) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 268 lines]`
Canonical summary line: `215 passed / 2 failed / 0 ignored / 2 suites`
Capture path: `target/testrun-capture.txt`

BASELINE FAILING SET (pre-existing):
- `static_html_sections::test_extraction_impact_preview_overlay`
  (assert `GRAPH_JS.lines().count() <= 998`, message "(cap 990)")
- `static_html_sections::test_inspector_panel_is_wider_and_collapsible`
  (assert at `static_html_sections.rs:411`: "the inspector collapse/toggle
  controls must call toggleInspector()")

DISCREPANCY vs recorded precondition: the handoff expected the baseline to
contain at minimum the graph.js budget red; it does, but the tree ALSO
carries a second pre-existing red on the inspector-toggle lock, introduced
by the landed WIP (`20e3fb8`). The planner's noted graph.js red still
exists unchanged. Per operator instruction the ACTUAL measured set above is
this track's machine-checkable baseline; the inspector-toggle red is
out-of-scope (report-only; fixing it would require touching files outside
the touch-set). Both carriers (`crates/ui/assets/graph.js`,
`crates/ui/tests/ui_test/static_html_sections.rs`) stay byte-identical.

### Red - traversal_back.rs lock module

Gate 2 command: `python tools/testrun.py --distill -- -p ariadne-ui`
Digest header:
`### cargo test FAIL (exit 1) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 289 lines]`
Canonical summary line: `215 passed / 5 failed / 0 ignored / 2 suites`
File compiled (constants already existed) - semantic Red, not a compile error.
The 3 NEW assertion failures (first failing assert per test, verbatim):
- `traversal_back::test_back_control_markup` ->
  "I1: the traversal head must carry the traversal-back control class"
  (missing `class="traversal-back"` containment on INDEX_HTML)
- `traversal_back::test_return_capture_wiring` ->
  "I2: traversal.js must default the return tab to 'include'"
  (missing `let traversalReturnTab = 'include'` containment on TRAVERSAL_JS)
- `traversal_back::test_back_css_token_only` ->
  "I6: styles.css must define the .traversal-back rule"
  (missing `.traversal-back` rule on STYLES_CSS)
Failing set == BASELINE set UNION the three, confirmed by name.

### Green - C1-C4 asset edits

### Green - C1-C4 asset edits

Gate 3 command: `python tools/testrun.py --distill -- -p ariadne-ui`
Digest header:
`### cargo test FAIL (exit 1) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 271 lines]`
Canonical summary line: `218 passed / 2 failed / 0 ignored / 2 suites`
Failing set == BASELINE FAILING SET by name identity (both pre-existing reds
unchanged, graph.js budget test still failing with the same "(cap 990)"
message - untouched, not this track's). Passed = 215 baseline + 3 new
traversal_back tests, all green. No assertion weakened.

### Refactor + hygiene

Gate 4 (refactor re-run) command: `python tools/testrun.py --distill -- -p
ariadne-ui` -> `218 passed / 2 failed / 0 ignored / 2 suites`, failing set
name-identical to BASELINE (same verdict as Green). Refactor pass found
nothing further to change: the C2 doc comments were authored in place at
Green, the three JS functions are 2-3 lines each, `css_rule_body` exists
once in `traversal_back.rs` (no copied duplication), no banned names in any
touched file.

DISCREPANCY 2 (Gate 5, pre-existing tree red):
`python tools/hygiene_check.py` exits 1 (via
`python tools/distill.py -- python tools/hygiene_check.py` ->
`### generic FAIL (exit 1)`), with exactly three R1/R2 findings:
`crates/ui/assets/graph.js: R2 1051 lines > 1000`,
`crates/ui/src/server.rs: R2 1120 lines > 1000`,
`crates/ui/src/server.rs:764: R1 fn api_graph_traversal is 161 lines`.
All three carriers are proven byte-identical to TRACK_BASE
(`git diff TRACK_BASE -- crates/ui/assets/graph.js
crates/ui/tests/ui_test/static_html_sections.rs crates/ui/src` -> empty,
distill capture 0 lines), i.e. this red was introduced by the landed WIP
(`20e3fb8`) and rides through unchanged; the handoff's "stays exit 0" was
already false at baseline. Zero findings in any touch-set file
(`traversal.js` is 416 lines; `styles.css` is not in R2_SCANNED_SUFFIXES;
`traversal_back.rs` clean). Fixing it is explicitly out of scope (deferred
ledger rows + frozen locks).

DISCREPANCY 3 (Gate 6, pre-existing + contract-internal collision):
`python tools/theme_token_check.py --check-added --base TRACK_BASE` prints
three violations (exit 1):
- `crates/ui/assets/traversal.js: 1 hex literals exceeds frozen baseline
  count 0` - PRE-EXISTING: byte probe shows the match `#128161` is the
  detector's regex reading the WIP's decimal entity `&#128161;` (the
  lightbulb emoji) at base; my added lines contain no `#`.
- `crates/ui/assets/index.html: 6 hex literals exceeds frozen baseline
  count 4` - PRE-EXISTING 5>4 at base (emoji entities `&#128293;`,
  `&#128161;` x2, `&#9888;`, `&#128279;` all match the `#hex{3,4,6}`
  regex) PLUS one match from this track (see next).
- `crates/ui/assets/index.html: new hex literal #8592 added` -
  attributable to this track: the C1 contract-mandated `&#8592;` numeric
  entity is read by the detector as hex `#8592`. It is a numeric character
  reference, pure-ASCII source, NOT a color literal; the handoff mandates
  this exact markup (C1 "after (exact)") while also demanding Gate 6
  silence - the two clauses cannot both hold given the tool's regex.
  Resolution: keep the exact contract markup; do NOT update the frozen
  baseline (forbidden path); substantive I6 is proven by
  `test_back_css_token_only` (rule body token-only, no `#`), by
  `scan_frozen_counts` showing `styles.css` base/current both `[]` (zero
  added hex in CSS), and by C1/C4 being the only diffs. Flagged for
  reviewer adjudication (kickoff G3).
Probe evidence (distill capture 13 lines): base/current hex-regex matches
per touched asset - index.html base 5 matches all from `&#...;` entities,
current adds `#8592` only; traversal.js base and current identical
(`#128161`); hotspots.js and styles.css `[]` at both.

### Gates + landing

I3/I4 zero-diff proofs (distiller, all empty where expected):
- `git diff --name-only TRACK_BASE` -> exactly the 5 tracked touch-set
  files (index.html, traversal.js, hotspots.js, styles.css,
  ui_test/main.rs) + new files `traversal_back.rs` and this journal;
  nothing outside the touch-set.
- `git diff TRACK_BASE -- graph.js static_html_sections.rs crates/ui/src`
  -> 0 lines (graph budget red and the inspector-toggle red ride through
  unchanged; Rust-source zero-diff G5/R5 holds).

G7 manual smoke: NOT executed - no browser in this session. Manual smoke
pending operator. Matrix to run at review hop: enter via tab button from
symbol tab -> Back lands on symbol; from call tab -> Back lands on call;
enter via hotspot card -> Back returns to include with the hotspot panel
restored; degree behavior unchanged from baseline (known-open bug, not
worsened).

Gate invocation itemization (ledger law): Gate 1 baseline testrun x1;
Gate 2 Red testrun x1; Gate 3 Green testrun x1; Gate 4 refactor testrun x1;
Gate 5 hygiene x2 (1 direct output capture + 1 distill rerun for the exit
code); Gate 6 theme_token_check x1; Gate 7 encoding_guard --staged see
landing below; Gate 8 commit_scoped x1 (+1 journal rider). D-row
2026-10-09: gate-command overshoot 11 > ~10 - extra hygiene invocation ran
through the distiller solely to surface its exit code in a digest header
(the direct invocation prints findings without a verdict line).

Encoding-guard form note (Gate 7): `commit_scoped.py` stages atomically
(`git add` + `git commit --only` in one invocation), leaving no seam for a
pre-commit `--staged` scan of this track's content. Executed instead:
`--staged` post-commit (index clean, exit 0, vacuous) AND `--all`
byte-level scan of all tracked working content (exit 0, includes every
committed touch-set file) as the non-vacuous evidence; all authored
content is pure ASCII by construction (Read/Edit/Write tools only, `&#8592;`
entity not the byte). `--all` is nominally the reviewer hop; run here
additionally as substitute evidence for the vacuous `--staged`, not as
adjudication.
