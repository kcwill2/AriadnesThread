# Kickoff — branding-assets (unregistered post-M28 track)

Track key: `branding-assets`. No M-number: per the `ui-hotspot-canvas-direction`
precedent (ledger row, CLOSED 2026-10-08), cosmetic UI/extension tracks run as
unregistered tracks with a live ledger row until closeout; the ledger stays the
numbering authority.

## Intent

Product branding (always "Ariadne's Thread", possessive — correction key
`product.name.possessive`):

- `media/AT-svg.svg` (the icon, 256x256 rounded-square brand art, pure ASCII)
  becomes the web app's branding: favicon + header brand mark in the UI served
  by `crates/ui`.
- `media/AT.jpg` (the image, ~1000x563 photographic banner) becomes branding
  inside the VS Code extension `extensions/ariadne-extension` (README banner;
  square icon art derived for the marketplace `icon` field).

Underlying why: both surfaces present the product name with generic or no art;
the user supplied canonical brand assets and wants them used.

## Scope in

- `crates/ui`: new embedded asset `icon.svg`, route `/icon.svg`, favicon link,
  header brand-mark swap (bolt SVG -> brand icon), standalone-export handling.
- `extensions/ariadne-extension`: `media/` banner copy + square icon, README
  banner, `package.json` `icon` field (PNG required by vsce).
- Two tools files ride along as declared contract mirrors (see shape R5):
  `tools/ui_smoke.py` route row, `tools/theme_token_check.py` brand-art
  exemption.
- Ledger registration row at kickoff, closeout flip at end (milestone-lifecycle
  ritual, same commit as memory mirror).

## Scope out (with reasons)

- `.vsix` rebuild/publish — packaging is an operator duty; the tracked
  `ariadne-extension-0.1.0.vsix` is a stale artifact, not rebuilt by this track.
- `themes.css` changes — the icon is theme-independent brand art; the
  three-theme token set is untouched (ledger D7 shared-var list includes
  `--brand-grad-*`; deleting it is a separate decision).
- Extension webview — verified: `src/extension.ts` has no webview; the graph
  viewer is the browser UI. There is no in-extension HTML surface for the
  banner beyond the README.
- Per-theme brand art, `--brand-grad-*` orphan cleanup, `favicon.ico` bitmap
  fallback — noted as follow-up candidates, not required by intent.
- `crates/ui/src/server.rs` extraction — pre-existing 1120-line ceiling breach;
  this change adds ~8 lines and records a finding, does not refactor.

## Invariants (must hold; falsifiable forms in the shape doc)

1. `/icon.svg` serves 200, `Cache-Control: no-store`, `image/svg+xml`.
2. `python tools/ui_smoke.py` stays green, and its route table gains an
   `/icon.svg` row (mirror contract: smoke mirrors server routes).
3. `crates/ui/assets/icon.svg` is byte-identical to `media/AT-svg.svg`;
   `extensions/ariadne-extension/media/banner.jpg` byte-identical to
   `media/AT.jpg`; `media/` remains the source of truth and is never edited.
4. `theme_token_check.py --check-added` and frozen-count scan exit 0;
   `themes.css` zero-diff; no new hex literals in `index.html`/`styles.css`.
5. `encoding_guard.py --all` exits 0 with the new SVG + JPG blobs tracked.
6. Exported standalone HTML contains no `/icon.svg` reference.
7. Authored text is pure ASCII; product name spelled "Ariadne's Thread".

## Frozen locks inherited (with proof commands)

- UTF-8 / byte-safety discipline (incident E2): never write files via
  PowerShell text pipelines; binary copies must be byte-level (Python
  `read_bytes`/`write_bytes`). Proof: `python tools/encoding_guard.py --all`.
- Three-theme set light/dark/dark-steampunk (constraint
  `ui.three_theme_set`): proof: `cargo test -p ariadne-ui` theme suites
  (`theme_contract.rs`, `theme_js_zero_hex.rs`, `theme_dark_grey_palette.rs`)
  UNEDITED.
- Asset no-store contract (incident E3): proof: `python tools/ui_smoke.py`
  and `crates/ui/tests/ui_test/static_html.rs`
  `test_ui_asset_routes_serve_bundled_content_with_no_store` extended, not
  weakened.
- Hex-literal drift guard (T13 three-theme plan): proof:
  `python tools/theme_token_check.py --check-added --base <track-baseline>`.
- Goldens/fixtures/standalone-export order locks: `static_html.rs`
  `test_script_tag_order_preserves_original_execution_order` and
  `test_standalone_export_inlines_all_split_assets` UNEDITED (new assertions
  appended in NEW tests only).

## Acceptance gate (track-level)

- Handoff A (app side) and handoff B (extension side) each gated APPROVED via
  review-gate with commit-vs-touch-set EXACT.
- Fresh-hop gates at final commit: scoped `cargo test -p ariadne-ui` via
  `python tools/testrun.py --distill` (0 failed); `python tools/ui_smoke.py`
  PASS; `python tools/theme_token_check.py --check-added --base <baseline>` 0;
  `python tools/encoding_guard.py --all` 0; `python tools/hygiene_check.py` 0.
- Human-eyeball rider (journal line, per hotspot precedent): header shows the
  brand icon at ~32px; browser tab shows favicon; each of the 3 themes renders
  the brand mark acceptably (brand art is fixed-color by design — no themed
  variant is produced).
