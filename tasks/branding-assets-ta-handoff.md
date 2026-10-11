# T-A Handoff - branding-assets: served brand icon in the web app (crates/ui)

Authored 2026-10-11 by the planner role. Unregistered post-M28 track per
kickoff: NO M-number; `tasks/milestone_ledger.md` stays numbering authority
(ledger registration row exists; the hotspot-canvas live-row pattern is the
precedent).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract; zero verbal briefing needed).
2. `tasks/branding-assets-kickoff.md` (intent, invariants 1-7, inherited
   frozen locks, track acceptance, human-eyeball rider).
3. `tasks/branding-assets-shape.md` (decisions D1-D4, invariants I1-I9,
   contracts, risks R1-R8, hygiene notes, sizing).
4. Content anchors (verbatim, verified against the working tree 2026-10-11):
   `crates/ui/src/server.rs` `build_router` route roster (asset routes at the
   top of the chain), the `ui_asset()` builder + `asset_styles_css`/
   `asset_themes_css` handlers; `crates/ui/src/lib.rs` const list (ends at
   `THEME_JS`); `crates/ui/src/export.rs` `standalone_template()` and its
   `use crate::{...}` import list; `crates/ui/assets/index.html` head
   stylesheet links (lines ~21-22) and the `.brand` header block
   (`.logo-badge` with the bolt `<svg>` child, lines ~37-42);
   `crates/ui/assets/styles.css` `.logo-badge` and `.logo-badge svg` rules
   (lines ~63-77); `tools/ui_smoke.py` `ROUTE_ASSERTIONS` tuple table;
   `tools/theme_token_check.py` `TOKEN_FILES` + its comment block.
5. Lock/test patterns to mirror: `crates/ui/tests/ui_test/d27_node_provenance.rs`
   `d27_provenance_route_serves_the_asset` (raw route assertion via
   `common::request_raw_body`); `crates/ui/tests/ui_test/hotspot_direction.rs`
   (asset-string lock doc style); `crates/ui/tests/ui_test/static_html.rs`
   `test_ui_asset_routes_serve_bundled_content_with_no_store` (READ-ONLY
   pattern source - the file itself is frozen, see Frozen locks).
6. Literal sources: brand asset `media/AT-svg.svg` (55 lines, pure ASCII,
   256x256 viewBox, ids `neonGradient`/`neonGlow`); `media/` is read-only
   source of truth.

## Dispatch preconditions (blocking)

- HOLD dispatch until the in-flight server.rs structural split (AR-13,
  dispatched 2026-10-11 by the orchestrator) has landed or is otherwise
  dispositioned: this handoff's `server.rs` anchors (route roster + `ui_asset`)
  live inside the region that split moves. If AR-13 landed, re-anchor every
  verbatim before-string below at the new location before any edit; the
  content strings are the contract, the line numbers are not.
- If at session start the working tree is dirty on any touch-set file, or any
  verbatim before-string below does not match the current file exactly, STOP
  and report; do not silently rebase onto moved content (shape R1).
- `python tools/hygiene_check.py` is expected NOT exit-0 at baseline: it
  carries the inherited `crates/ui/src/server.rs` R2 finding pair (>1000
  physical lines; pre-existing, tracked in the ui-degree-opacity-ramp closeout
  row). Step 0 records the exact baseline finding set; if AR-13 landed and the
  server.rs finding is gone, the baseline is simply the (smaller) actual set.

## Objective and invariants

Serve the canonical brand icon (`media/AT-svg.svg`) as a first-class UI asset
and use it as the product's face in the app: favicon + header brand mark.
Preserve, falsifiably:

- I1 (kickoff invariant 1) `GET /icon.svg` -> 200, `Cache-Control: no-store`,
  `Content-Type: image/svg+xml`, body = the embedded asset. Comes from the
  shared `ui_asset()` builder; no new serving machinery.
- I2 (kickoff invariant 2) `python tools/ui_smoke.py` stays green AND its
  `ROUTE_ASSERTIONS` table carries the `("/icon.svg", "image/svg+xml", True)`
  row (mirror contract: the table mirrors `build_router` + the page's loads;
  a stale table breaks it - shape D4).
- I3 (kickoff invariant 3) `crates/ui/assets/icon.svg` is byte-identical to
  `media/AT-svg.svg` (machine lock: new test `test_icon_asset_matches_source_of_truth`);
  `media/**` is never edited and stays zero-diff.
- I4 (kickoff invariant 4) `theme_token_check.py --check-added` AND its
  frozen-count scan exit 0; `themes.css` zero-diff; no new hex literal in
  `index.html`/`styles.css` (the icon's hex lives only in the exempted
  brand-art asset, exemption via the `TOKEN_FILES` rider - shape D4/R2).
- I5 (kickoff invariant 5) `encoding_guard.py --staged` exit 0 per commit
  (the SVG is pure ASCII; a mojibake signature in the copy aborts the commit).
- I6 (kickoff invariant 6) standalone export closed-world: exported HTML
  contains zero `/icon.svg` references; brand art is inlined in it (the two
  sanctioned `neonGradient`/`neonGlow` ids are unique in the page); the
  existing `src="/app/` and `href="/styles.css"` zero-occurrence locks still
  hold. The app page keeps the mark as `<img>` - never inline (shape D6
  rationale: keeping the id-collision guarantee cheap on the app page).
- I7 (kickoff invariant 7) all authored text pure ASCII; the new alt-text
  spells `Ariadne's Thread` (possessive - correction key
  `product.name.possessive`).
- I8 zero collateral diff: every file outside the Touch-set below, in
  particular `media/**`, `themes.css`, all other app JS, `main.js`, all
  pre-existing test files, `tools/**` except the two declared rider lines,
  and everything under `extensions/` (handoff B's territory - this handoff
  MUST NOT touch it).

## Scope boundaries (explicit out-of-scope)

- `server.rs` extraction: the 1000-line ceiling breach is pre-existing; this
  change adds minimally (~5-8 lines) and records the finding in the journal.
  Do NOT refactor; do NOT "fix" the hygiene red (shape hygiene note 1;
  asset-handler extraction is a separate future handoff).
- `favicon.ico` bitmap fallback, per-theme brand art, `--brand-grad-*` orphan
  cleanup: out (kickoff scope-out; R6 accepted degradation).
- The `index.html <title>` string "AriadnesThread Graph Visualizer" is
  pre-existing; not in the shape; do NOT change it (report-only observation
  at most - possessive-name correction applies to NEW strings only).
- `extensions/ariadne-extension/**` is handoff B's scope. Disjoint touch-sets;
  this handoff is parallel-safe with B but never edits B's files.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/branding-assets-ta-journal.md` (checkpoint journal; create it;
  always in-set)
- `crates/ui/assets/icon.svg` (ADD - byte-level copy of `media/AT-svg.svg`;
  procedure C0 below; never a PowerShell text pipe)
- `crates/ui/src/lib.rs` (+1 line: `pub const ICON_SVG` at the tail of the
  const list, after `THEME_JS`)
- `crates/ui/src/server.rs` (+1 route line in `build_router`, +3-line handler
  after `asset_themes_css` - content C2)
- `crates/ui/src/export.rs` (+`ICON_SVG` to the `use crate::{...}` list
  between `HOTSPOTS_JS,` and `INDEX_HTML,`; two replaces in
  `standalone_template()` - content C5)
- `crates/ui/assets/index.html` (2 edits: favicon link line; `.logo-badge`
  child swap - content C3; no hex literals)
- `crates/ui/assets/styles.css` (replace the `.logo-badge svg` rule with the
  `.logo-badge img` rule - content C4; no hex literals; the `.logo-badge`
  gradient box rule MAY stay or go, either way no new hex - shape D1)
- `crates/ui/tests/ui_test/icon_asset.rs` (NEW - 4 tests, content C6)
- `crates/ui/tests/ui_test/main.rs` (+1 line: `mod icon_asset;` inserted
  between `mod hotspot_direction;` and `mod justification_builders;` - the
  true alphabetical site; do NOT append at roster tail: the roster-tail site
  is a known erratum trap for names sorting earlier)
- DECLARED RIDER 1 `tools/ui_smoke.py` (+1 tuple line at the tail of
  `ROUTE_ASSERTIONS`: `("/icon.svg", "image/svg+xml", True),` - nothing else;
  mirror contract must change atomically with the route, shape D4; ledger law:
  declared scope deviation on merit, not silence)
- DECLARED RIDER 2 `tools/theme_token_check.py` (`TOKEN_FILES` gains
  `"icon.svg"` + the comment block extended per content C7 - the sanctioned
  brand-art palette exemption for BOTH scans)

The 2 tools/ rider lines are the DECLARED scope deviation: 1 crate
(`crates/ui`) + 2 one-line contract mirrors in `tools/`, justified in
shape D4 ("splitting them into a third handoff invites the drift they exist
to prevent"). The reviewer enforces commit-vs-touch-set EXACT against the
list above INCLUDING the riders - the deviation is declared here, on the
ledger row, and in the journal, never improvised.

You MUST NOT touch: `media/**` (source of truth), `crates/ui/assets/themes.css`,
`crates/ui/assets/main.js` or any other app JS, `tools/theme_token_baseline.json`,
any other file under `tools/` (only the 2 riders), any pre-existing test file
(`static_html.rs` is exactly 999 lines at the R2 ceiling and frozen;
`static_html_sections.rs` carries the graph.js lock; theme suites
`theme_contract.rs`/`theme_js_zero_hex.rs`/`theme_dark_grey_palette.rs` are
frozen UNEDITED by kickoff invariant 4), `extensions/**`, `tests/fixtures/**`,
`tests/goldens/**`, `tasks/milestone_ledger.md`, any `Cargo.toml`/`Cargo.lock`.
No `git stash`, no branch change.

## What to build (contracts C0-C7; before-strings verbatim from the tree 2026-10-11,
re-verified at dispatch per the precondition above)

C0 Byte-level asset copy (binary-safe, AGENTS.md File-Edit Discipline):
run the copy as byte-level Python, never through any PowerShell text pipe:

```
python tools/distill.py -- python -c "from pathlib import Path; Path('crates/ui/assets/icon.svg').write_bytes(Path('media/AT-svg.svg').read_bytes())"
```

Then prove byte identity in the journal: `python tools/distill.py -- git hash-object media/AT-svg.svg crates/ui/assets/icon.svg` - the two printed hashes MUST be equal (if your session's permissions deny even via the distiller, the new Rust source-of-truth test below is the machine check; note the denial in the journal). Also record `media/AT-svg.svg`'s byte size in the journal.

C1 `lib.rs` const, after the `THEME_JS` line (exact form):

```rust
pub const ICON_SVG: &str = include_str!("../assets/icon.svg");
```

C2 `server.rs` (AR-13 may have relocated these anchors; match content):
in `build_router`, after `.route("/themes.css", get(asset_themes_css))`, add:

```rust
        .route("/icon.svg", get(asset_icon_svg))
```

after the `asset_themes_css` handler, add:

```rust
async fn asset_icon_svg() -> axum::response::Response {
    ui_asset(crate::ICON_SVG, "image/svg+xml")
}
```

No-store + 200 come free from `ui_asset()` (incident E3 contract).

C3 `index.html`. Edit 1 - head, before (exact):

```html
  <link rel="stylesheet" href="/styles.css">
</head>
```

after:

```html
  <link rel="stylesheet" href="/styles.css">
  <link rel="icon" type="image/svg+xml" href="/icon.svg">
</head>
```

Edit 2 - the `.brand` header block, before (exact, 6-/8-space indentation):

```html
      <div class="logo-badge">
        <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
        </svg>
      </div>
```

after (exact; `<img>`, never inline SVG - I6 rationale; ASCII apostrophe):

```html
      <div class="logo-badge">
        <img class="brand-mark" src="/icon.svg" alt="Ariadne's Thread logo">
      </div>
```

C4 `styles.css`, before (exact):

```css
    .logo-badge svg {
      width: 18px;
      height: 18px;
      stroke: var(--on-accent);
    }
```

after (fills the 32px rounded box; art is a fixed-color rounded square; no
hex literal anywhere in the diff; `border-radius` optional at the implementer's
eyeball, matching the existing 8px box radius):

```css
    .logo-badge img {
      width: 100%;
      height: 100%;
      border-radius: 8px;
    }
```

C5 `export.rs` `standalone_template()`: add `ICON_SVG,` to the import list
(alphabetical site above). After the `STYLES_CSS` replace block, before the
JS loop, add:

```rust
    // Brand art under file:// has no routes: the favicon is meaningless and
    // the header mark must arrive as inline markup. Ids neonGradient/neonGlow
    // are unique among inline SVGs here (the app page keeps <img> precisely
    // to keep that guarantee cheap - shape D6).
    html = html.replace(
        "<link rel=\"icon\" type=\"image/svg+xml\" href=\"/icon.svg\">",
        "",
    );
    html = html.replace(
        "<img class=\"brand-mark\" src=\"/icon.svg\" alt=\"Ariadne's Thread logo\">",
        ICON_SVG.trim_end(),
    );
```

The two literal strings MUST be byte-equal to the C3 markup (shared
contract). Post-condition is invariant I6.

C6 New test module `crates/ui/tests/ui_test/icon_asset.rs` + the C6 `mod`
line. Why a new file: `static_html.rs` is exactly 999 lines - AT the R2
ceiling - so the kickoff's "extend" option for
`test_ui_asset_routes_serve_bundled_content_with_no_store` is unsatisfiable
without a NEW R2 breach; PLANNER ADJUDICATION: the shape's preferred option
wins (all new assertions in a NEW test file; the 999-line lock file survives
UNEDITED, nothing weakened). Route-assertion style mirrors
`d27_node_provenance.rs`. Doc-comment style mirrors `hotspot_direction.rs`.
Tests (exact expected behavior):

1. `test_icon_route_serves_brand_art_with_no_store` (I1):
   `ariadne_ui::build_router(common::loading_state())`;
   `common::request_raw_body(app, "/icon.svg")`; assert status 200,
   `Cache-Control == "no-store"`, `Content-Type == "image/svg+xml"`, body
   `== ariadne_ui::ICON_SVG.as_bytes()`.
2. `test_icon_asset_matches_source_of_truth` (I3): first assert
   `Path::new(&format!("{}/../assets/icon.svg", env!("CARGO_MANIFEST_DIR")))
   .exists()` with message naming the brand asset; then byte-compare that
   file against `env!("CARGO_MANIFEST_DIR") + "/../../media/AT-svg.svg"`
   via `std::fs::read`, asserting equality ("the served asset must stay
   byte-identical to the source of truth").
3. `test_page_head_marks_favicon_and_brand_img` (I6/I7 markup half):
   `ariadne_ui::INDEX_HTML` contains the exact favicon line
   `<link rel="icon" type="image/svg+xml" href="/icon.svg">`, contains the
   exact img tag `<img class="brand-mark" src="/icon.svg" alt="Ariadne's Thread logo">`,
   contains NEITHER the retired bolt path `d="M13 10V3L4 14h7v7l9-11h-7z"`
   nor `neonGradient` nor `neonGlow` (ids must live only in the served
   asset, keeping the page collision-free); `STYLES_CSS` contains
   `.logo-badge img` and no longer contains `.logo-badge svg`.
4. `test_standalone_export_closes_icon_references` (I6):
   `ariadne_ui::export_standalone_html(&common::empty_export_payload(), "",
   &out_path)` into a `tempfile::tempdir()`; read the file; assert it
   contains `neonGradient` (brand art inlined) and contains NONE of:
   `/icon.svg`, `<img class="brand-mark"`, `<link rel="icon"`,
   `src="/app/`, `href="/styles.css"`, `href="/themes.css"` (last three are
   the surviving closed-world locks, asserted here so the closed set is
   whole in the NEW test; the locked originals in `static_html.rs` stay
   UNEDITED).

C7 Rider `tools/theme_token_check.py`: replace

```python
TOKEN_FILES = ("themes.css",)
```

with

```python
TOKEN_FILES = ("themes.css", "icon.svg")
```

and extend the comment block above it with (keep the existing sentences):

```python
# icon.svg is sanctioned brand art (source of truth: media/AT-svg.svg): its
# literals ARE the fixed brand palette, exempt from BOTH scans exactly like
# the token file; a re-brand edits media/ then re-syncs the copy (shape D3).
```

Rider 1 `tools/ui_smoke.py`: append the tuple
`("/icon.svg", "image/svg+xml", True),` at the tail of `ROUTE_ASSERTIONS`
(positional `True` = expect no-store, same shape as every existing row).
Nothing else in the file - its docstring is already drifted from the script
roster (known relay; do not "fix" it here).

## RGB protocol (single module, this file order)

1. Red: write `icon_asset.rs` with tests 2-4 ONLY (no test references
   `ariadne_ui::ICON_SVG` yet - a test that fails to compile is not Red)
   + the `mod` line. Run
   `python tools/testrun.py --distill -- -p ariadne-ui` (timeout 600000 ms).
   Expected EXACTLY three new assertion failures on top of the Step 0
   baseline set, each with its own message:
   - `icon_asset::test_page_head_marks_favicon_and_brand_img` - first
     failing assert is the missing favicon line on `INDEX_HTML`;
   - `icon_asset::test_icon_asset_matches_source_of_truth` - first failing
     assert is the missing `crates/ui/assets/icon.svg` existence check;
   - `icon_asset::test_standalone_export_closes_icon_references` - first
     failing assert is the missing `neonGradient` inlining.
   Capture the failing set verbatim in the journal (must equal BASELINE
   UNION these three).
2. Green: apply C0-C5 + rider 2 + riders BEFORE adding test 1 (the route
   test references the new constant). Re-run the same scoped command.
   Add test 1, re-run. NEVER weaken an assertion to pass. Expected: failing
   set is EXACTLY the BASELINE set again (name identity) and passed count =
   Step 0 passed + 4.
3. Refactor: doc comments, honest names, dedupe any helper; re-run the same
   command (same expectation as Green). Then the remaining gates below.

## Required tests

C6 tests 1-4, exactly as specified; no other test diffs anywhere (I8).

## Hygiene (review-enforced)

Functions < 100 lines (largest addition ~30, the new test file stays
~120-140 lines total). Files < 1000: `icon_asset.rs` new and small;
`lib.rs`/`server.rs`/`export.rs` grow minimally; `index.html` ~604 +3;
`styles.css` ~0 delta; do NOT add a line to `static_html.rs` (999 = ceiling).
`server.rs` is ALREADY over the ceiling: its inherited R2 finding rides
through; the finding count in its message must match the journal's line-count
claim; NO new file/rule finding may appear. No banned names
(`tmp`/`res`/`foo`/`bar`/`test1`), no single-letter locals,
intention-revealing identifiers (C-names are contractual), source lines
< 2000 chars. `python tools/hygiene_check.py` baseline finding-set identity
(file+rule pairs) at Green - not a blanket exit-0 claim while the inherited
server.rs pair stands.

## Frozen locks (all inherited; proof commands scoped)

| Lock | Proof |
|---|---|
| `media/**` zero-diff (source of truth, kickoff invariant 3/D3) | `git diff TRACK_BASE -- media` EMPTY; C6 test 2 |
| `crates/ui/tests/ui_test/static_html.rs` UNEDITED (999-line R2 ceiling; carries `test_ui_asset_routes_serve_bundled_content_with_no_store`, `test_script_tag_order_preserves_original_execution_order`, `test_standalone_export_inlines_all_split_assets` - all survive UNEDITED, new assertions live in `icon_asset.rs` only) | `git diff TRACK_BASE -- crates/ui/tests/ui_test/static_html.rs` EMPTY |
| Theme suites `theme_contract.rs` / `theme_js_zero_hex.rs` / `theme_dark_grey_palette.rs` UNEDITED and green (kickoff invariant 4; brand mark renders identical across themes by design - fixed art) | scoped ui suite, failing set = baseline set |
| `themes.css` + all other app assets (JS, `main.js`) + `static_html_sections.rs` zero-diff (I8) | `git diff TRACK_BASE --` each path EMPTY |
| Asset no-store contract (incident E3): shared `ui_asset()` unchanged; smoke + route test extended not weakened (extension lives in `icon_asset.rs` + smoke rider) | scoped ui suite green + gate 5 smoke PASS incl. the `/icon.svg` row |
| Zero added hex in scanned assets; new brand asset exempt via the C7 rider only (`--update-baseline` NOT needed and NOT allowed - shape R2: the exemption covers both scans) | `python tools/theme_token_check.py --check-added --base TRACK_BASE` silent AND `python tools/theme_token_check.py` (frozen-count scan) exit 0 |
| No mojibake; pure-ASCII authored text; possessive product name in the new alt text | `python tools/encoding_guard.py --staged` exit 0 before EACH commit (`--all` is the reviewer hop) |
| Goldens/fixtures, `tools/theme_token_baseline.json`, ledger, extensions: outside touch-set | reviewer `git diff --name-only` set equality against the Touch-set |

## Gates (exact commands; implementer hop; timeout 600000 ms each; scoped only)

1. Step 0 baseline: `python tools/testrun.py --distill -- -p ariadne-ui` -
   journal the digest header, canonical line, capture path, and the FULL
   failing-test-name set from `target/testrun-capture.txt`, labeled
   `BASELINE FAILING SET (pre-existing)`. Record `TRACK_BASE <rev>`.
2. Step 0 hygiene baseline: `python tools/hygiene_check.py` - journal the
   complete finding set verbatim (expect the inherited server.rs R2 pair;
   record whatever you MEASURE - reconcile only against measurement, never
   against guessed numbers, standing ledger errata).
3. Red: same command as gate 1 (expected failing set = BASELINE UNION the
   three named Red tests; capture in the journal).
4. Green (twice: without/with test 1): same command as gate 1 (failing set
   name-identical to BASELINE; passed = baseline + 4; digest header +
   canonical line verbatim in the journal). C0's hash-object equality and
   `media/AT-svg.svg` byte size go in the journal here too.
5. `python tools/distill.py -- python tools/ui_smoke.py` - `ui_smoke: PASS`
   with the new `/icon.svg` row asserted (bounded via the distiller; the
   fresh un-distilled hop is the reviewer's).
6. `python tools/theme_token_check.py --check-added --base TRACK_BASE` -
   silent (exit 0).
7. `python tools/theme_token_check.py` - frozen-count scan exit 0 (proves
   the C7 exemption holds on both scans).
8. `python tools/hygiene_check.py` - finding-set identity vs gate 2
   (inherited server.rs pair only, count grown by exactly the journal's
   server.rs line claim; zero new file/rule findings).
9. `python tools/encoding_guard.py --staged` - exit 0 before committing.
10. Commit with `python tools/commit_scoped.py --scope crates/ui/assets/icon.svg
    --scope crates/ui/src --scope crates/ui/assets/index.html
    --scope crates/ui/assets/styles.css
    --scope crates/ui/tests/ui_test/icon_asset.rs
    --scope crates/ui/tests/ui_test/main.rs --scope tools/ui_smoke.py
    --scope tools/theme_token_check.py
    --scope tasks/branding-assets-ta-journal.md
    -m "feat(ui): serve brand icon at /icon.svg with favicon and brand mark"`
    (one landing commit; never `git add .`; if your session's permissions
    deny `commit_scoped.py`/`git`, STOP and report - commit duty routes to
    the orchestrator per the standing allowlist errata row; do not improvise
    raw git staging).

Journal duty: itemize per-gate invocation counts; DECLARE any overshoot
beyond 10 as a dated D-row. NOT this session's gates (workspace-wide,
reviewer hop): workspace census, `encoding_guard --all`, fresh `ui_smoke`,
`bench_compare` (exempt: no pipeline/Rust hot-path producer changes beyond
the ui crate's own serving; reviewer confirms via diff paths).

## Reviewer hop (fresh session; adjudicate via `review-gate`; re-run FRESH)

1. Scoped ui suite fresh: failing set == journal BASELINE by name identity;
   all four `icon_asset` tests green; passed = baseline + 4 (the
   `--baseline` FILE form is the standing errata - use `--distill`).
2. `git diff --name-only TRACK_BASE..HEAD` set-equality against the Touch-set
   minus the journal - EXACT, riders included, nothing else. Zero-diff
   proofs for the Frozen-lock table rows (`media`, `static_html.rs`, theme
   suites, `themes.css`, other JS).
3. `python tools/ui_smoke.py` PASS fresh (route via
   `python tools/distill.py -- python tools/ui_smoke.py` if needed); grep
   the smoke table carries the `/icon.svg` row.
4. `python tools/theme_token_check.py --check-added --base <landing-parent>`
   silent; frozen-count scan exit 0; `python tools/encoding_guard.py --all`
   exit 0 with the new SVG tracked (shape R1: the SVG is text - if the guard
   ever flags it, STOP-report, never edit the guard);
   `python tools/hygiene_check.py` finding-set identity vs journal baseline.
5. Workspace census `python tools/testrun.py --distill -- --workspace`:
   passed = live baseline passed + 4, reconciled from the live tree.
6. Per-commit `git show` oldest-first; commit-vs-touch-set EXACT; journal
   carries the kickoff eyeball line: header shows the brand icon at ~32px;
   browser tab shows the favicon; each shipped theme renders the brand mark
   acceptably (kickoff enumerates light/dark/dark-steampunk; the tree also
   ships dark-grey since the ui-dark-grey-theme track - eyeball it too; fixed
   art by design). If the implementer could not open a browser, the journal
   says "manual eyeball pending operator" - an unexecuted check is never
   checked off.

## Discipline clauses

- UTF-8 / binary safety (kickoff E2 lock; AGENTS.md File-Edit Discipline):
  never `Set-Content`/`Out-File`/`>`/`Get-Content -replace`; Read/Edit/Write
  tools only; the asset copy is byte-level Python only (C0); authored text
  pure ASCII (the apostrophe in `Ariadne's Thread` is ASCII).
- Touch only the touch-set; on discovering out-of-scope breakage (e.g. AR-13
  moved an anchor): STOP-report, do not fix, do not silently re-anchor onto
  half-landed code.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, raw census,
  undistilled `ui_smoke`, `bench_compare`) in this implementer session are a
  reviewer blocker; everything routes through the distilling entry points
  (`docs/specs/tool-output-distillation.md`).
- After EACH per-module Green gate, append a 1-3 line checkpoint to
  `tasks/branding-assets-ta-journal.md`: module + commit hash, any deviation,
  any lock/contract finding - including the Step 0 baseline sets and any
  anchor mismatch.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/branding-assets-ta-handoff.md --scope crates/ui --cmd "python tools/testrun.py --distill -- -p ariadne-ui"`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence: the 3 new assertion
messages + failing set equal to BASELINE UNION the three. 3. Green evidence:
distill digest header line + canonical summary line verbatim + capture path,
failing set name-identical to BASELINE, passed = baseline + 4; the two
`git hash-object` hashes for C0; ui_smoke PASS line; both theme-token
verdicts; hygiene finding-set identity statement. 4. Any deviation + reason
(declare gate-count overshoot as a D-row). 5. Out-of-scope observations
(report-only). Never paste raw tool output: cite command + canonical line +
exit code. The reviewer re-runs gates against the commits; pasted output is
context rot.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the Touch-set above EXACT,
including the two declared rider lines (shape D4/R5: the deviation is
declared here, on the ledger row, and in the journal - a rider edit that is
NOT one of the two declared lines is a blocker).
