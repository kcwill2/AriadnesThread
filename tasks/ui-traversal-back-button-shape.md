# Shape: UI traversal back button (key `ui-traversal-back-button`)

Companion to `tasks/ui-traversal-back-button-kickoff.md` (intent, scope,
frozen locks, acceptance gates live there).
Baseline: the tree after the uncommitted traversal-view WIP lands or is
dispositioned (kickoff Precondition). All line numbers are working-tree
facts-at-kickoff, and edits must be content-anchored.

## Chosen back semantics (the core decision)

`Back` = one-level return to the tab that was active when traversal was last
entered; default `'include'` when no other entry was captured.
Rejected alternatives:
- Relabel-only (keep `switchTab('include')` behind a Back label): dishonest
  for entries from the symbol/call tabs; the user asked for intuitive, not
  for a sticker.
- Full navigation history: over-engineering for a 4-tab app whose tab bar is
  always visible; one level covers every documented entry path.
Entry points that must capture the origin (survey-verified as exhaustive):
`index.html:72` tab-button onclick and `hotspots.js::openTraversalForRoot`
(which is include-tab-only by construction -- hotspot rows render only when
the hotspot panel is open on the include tab).
Programmatic `switchTab('traversal')` callers added in the future would fall
back to the default `'include'`, which is today's behavior -- a strictly
non-regressive fallback.

## Invariants (each independently checkable; oracle named)

I1 Back control exists and the close contract is dead.
`INDEX_HTML` contains `class="traversal-back"` and
`onclick="backFromTraversal()"` and a `title` starting `Back to`;
`INDEX_HTML` and `TRAVERSAL_JS` contain no `traversal-close`,
no `closeTraversal`, and the traversal head contains no `title="Close"`;
`STYLES_CSS` has no `.traversal-close` rule.
Oracle: new lock test `traversal_back::test_back_control_markup`
(string-level, mirrors existing asset-literal lock style).

I2 Return-to-origin semantics are wired at every entry.
`TRAVERSAL_JS` declares `let traversalReturnTab = 'include'`, defines
`function openTraversalTab()` capturing `currentTab` (guarded
`currentTab !== 'traversal'`) before calling `switchTab('traversal')`, and
`function backFromTraversal()` calling `switchTab(traversalReturnTab)`;
`INDEX_HTML` tab button and `HOTSPOTS_JS::openTraversalForRoot` call
`openTraversalTab()` and neither contains a direct
`switchTab('traversal')` anymore.
Oracle: lock test `traversal_back::test_return_capture_wiring` + the G7
manual-smoke matrix in the landing journal (3 entries x Back: symbol, call,
hotspot-card).

I3 Zero collateral diff.
`crates/ui/assets/graph.js`, `api.js`, `main.js`, `preview.js`,
`provenance.js`, `file-view.js`, `theme.js`, `themes.css`, all of
`crates/ui/src/**`, all of `tools/**` byte-identical to the track baseline.
Oracle: reviewer `git diff --name-only <base>..<head>` set equality against
the module map below (hard part of review-gate).

I4 Degree area untouched (adjacent-open-bug protection).
The degree bug (`ui.traversal_view_degree_setting_bug`, open, prior fix
attempt failed) lives in `graph.js::switchTab` traversal branch +
`parseAndRenderElements` + `filterAndExpandSubgraph` and in the uncommitted
WIP.
This track adds no diff there (implied by I3, checked as its own gate line
so a reviewer cannot wave it past inside a bigger diff).
Oracle: `git diff <base> -- crates/ui/assets/graph.js` empty.
Honest status note: the degree bug stays OPEN after this milestone; no test
pins it green today and this shape does not claim otherwise.

I5 Asset transport and script roster unchanged.
`/app/traversal.js` route, `no-store`, `include_str!` constant, export
inline roster entry, and the 9-script roster survive with zero edits.
Relay fact (do not fix here): the byte-equality route test
`static_html.rs::test_ui_asset_routes_serve_bundled_content_with_no_store`
has an 8-entry roster that omits `/app/traversal.js` (the roster exists on
the WIP side); this track neither extends nor amends it.
Oracle: pre-existing tests green UNEDITED (I7) + `ui_smoke` PASS.

I6 Theme safety.
Zero added hex literals; the renamed CSS rules declare only existing
`var()` tokens (`--text-muted`, `--text-main`); every added/changed CSS
token is already theme-resolved, so light/dark/dark-steampunk (and
dark-grey, fourth theme landed) inherit automatically.
Oracle: `python tools/theme_token_check.py --check-added --base <parent>`
silent + lock test asserts the `.traversal-back` rule body contains
`var(--` and no `#` (mirrors the `step_row_css_block` token-only pattern).

I7 Existing-test freeze.
Every pre-existing test file is byte-identical; the only test diffs are the
new `traversal_back.rs` module and one `mod` line in `ui_test/main.rs`.
Oracle: reviewer commit-vs-touch-set + `git diff --name-only` filtered to
`crates/ui/tests`.

## Contracts (exact shapes at every changed boundary)

C1 Button markup (`index.html`, inside `.traversal-head`):
before:
`<button class="traversal-close" onclick="closeTraversal()" title="Close">&#10005;</button>`
after (pure-ASCII source; `&#8592;` is a numeric entity, no literal
non-ASCII byte):
`<button class="traversal-back" onclick="backFromTraversal()" title="Back to the view you came from">&#8592; Back</button>`
No SVG (tab-bar SVG icons stay a tab-bar convention; panel-head controls in
this app are text/entity buttons).

C2 traversal.js declarations (replacing the `closeTraversal` block at the
file tail; classic-script global scope, traversal.js loads at
`index.html:594`, before hotspots.js `:596` and main.js `:599`, so click-time
resolution is safe):

```
let traversalReturnTab = 'include';

function openTraversalTab() {
  if (currentTab !== 'traversal') traversalReturnTab = currentTab;
  switchTab('traversal');
}

function backFromTraversal() {
  switchTab(traversalReturnTab);
}
```

Exports: replace `window.closeTraversal = closeTraversal;` with
`window.openTraversalTab = openTraversalTab;` and
`window.backFromTraversal = backFromTraversal;`.
`window.closeTraversal` has no other caller (repo-wide grep at kickoff:
definition + index.html onclick only), so the rename is total.
`currentTab` is declared in `main.js:4`; the reference inside
`openTraversalTab` resolves at click time, same pattern graph.js already
uses to read it.

C3 Entry-point swaps:
`index.html:72`: `onclick="switchTab('traversal')"` ->
`onclick="openTraversalTab()"` (id/class/svg/label unchanged).
`hotspots.js::openTraversalForRoot`: first statement
`switchTab('traversal');` -> `openTraversalTab();` (the root-select retry
loop below is untouched).

C4 CSS rename (`styles.css:1793-1805`): selectors `.traversal-close` and
`.traversal-close:hover` become `.traversal-back` and
`.traversal-back:hover`; declarations unchanged (already token-only), with
`gap: 4px; align-items: center;` added for the glyph+label pair.
No hex literal appears in the diff.

C5 New lock module `crates/ui/tests/ui_test/traversal_back.rs`
(registered by `mod traversal_back;` in `ui_test/main.rs`).
Why a new file: `static_html.rs` is 999 lines -- at the R2 ceiling -- and
frozen by I7; the hotspot precedent `static_html_sections.rs` shows the
split-file pattern.
Tests: `test_back_control_markup` (I1), `test_return_capture_wiring` (I2),
`test_back_css_token_only` (I6: `.traversal-back` rule body has `var(--`,
no `#`; `.traversal-close` absent).
Reads only `ariadne_ui::{INDEX_HTML, TRAVERSAL_JS, HOTSPOTS_JS, STYLES_CSS}`
constants; no server fixture needed.
Error shapes: none added (no fetch, no API change).

## Module map

Touched (complete; reviewer enforces set equality):
- `crates/ui/assets/index.html` (2 lines: `:72` onclick, `:388` button)
- `crates/ui/assets/traversal.js` (tail block replace + 2 export lines)
- `crates/ui/assets/hotspots.js` (1 line in `openTraversalForRoot`)
- `crates/ui/assets/styles.css` (2 selector renames + 2 declarations)
- `crates/ui/tests/ui_test/traversal_back.rs` (new)
- `crates/ui/tests/ui_test/main.rs` (+1 `mod` line)

Touched-by-embedding-only (zero-edit; content change rides `include_str!`):
`lib.rs`, `server.rs`, `export.rs`.

Must not change (named risk carriers, not a repo list):
`graph.js` (degree path + 998 budget lock `static_html_sections.rs:215`),
`main.js`, `themes.css`, `theme.js`, `theme_token_baseline.json`,
`tools/**`, `dto.rs`, every pre-existing test file,
`docs/specs/collateral-traversal-view.md` (the WIP's own doc), all of
`tests/**` outside `crates/ui/tests`.

## Risks and mitigations (oracle per risk)

R1 WIP collision: the uncommitted traversal/degree WIP mutates the same
files (`index.html`, `traversal.js` untracked, `hotspots.js`, `styles.css`).
Mitigation: dispatch precondition (kickoff), content-anchored edits,
I3/I4 zero-diff gates.
Oracle: reviewer diff set equality at landing; if anchors have moved, the
implementer STOPs and reports rather than rebasing silently.

R2 graph.js budget lock is red in the working tree TODAY (1051 vs `<= 998`
at `static_html_sections.rs:215`) because of the WIP, and the assert message
still says "(cap 990)" (known deferred row).
Mitigation: this track never touches either file; a pre-existing red there
must be recorded as identical before/after, not fixed here (fix belongs to
the WIP's landing or the heat/legend extraction row).
Oracle: scoped ui suite failure-set equality before/after + I3 diff.

R3 Back into a tab that "lost" its state: symbol/call tabs re-parse their
graph on every `switchTab` from the payload; hotspot visibility is derived
from `currentTab` + `hotspotOpen` inside `switchTab`, so returning to include
restores the open hotspot panel for free.
Mitigation: none needed by construction.
Oracle: G7 manual-smoke matrix.

R4 String-lock brittleness on rename: mitigated by the fact that these
literals ARE the contract (class + handler + function names); the lock test
is cheap to amend if a future track renames again.
Oracle: `traversal_back.rs` green.

R5 Theme drift: mitigated by token-only rule (C4) and the zero-added-hex
gate.
Oracle: `theme_token_check --check-added` silent + I6 lock test.

R6 Accidentally absorbing the degree bug (temptation while touching
traversal files): mitigated by I4 as its own gate line and by the touch-set.
Oracle: `git diff <base> -- crates/ui/assets/graph.js` empty.

## Hygiene notes

- `styles.css` is 2103 lines and exempt from the R2 suffix scan (known
  deferred row) -- this change renames in place, net CSS delta ~2 lines.
- `graph.js` at 1051 (working tree) is this track's zero-diff file, so its
  budget landmine is not grown here.
- `static_html.rs` at 999 lines is AT the 1000 ceiling: the reason new
  assertions go to `traversal_back.rs`; any future test growth in that file
  must extract first.
- No function approaches 100 lines in the touched JS; `traversal.js` is 406
  lines and grows by roughly +15.
- Authored content pure ASCII (`&#8592;` entity, not the byte); writes only
  via Read/Edit/Write tools, never PowerShell rewrites; commits via
  `commit_scoped.py` with `encoding_guard --staged` per repo discipline.
