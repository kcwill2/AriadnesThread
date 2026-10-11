# Branding assets — change shape (kickoff + shape digest)

Authoritative artifacts (written 2026-10-11):

- `tasks/branding-assets-kickoff.md` — intent, scope in/out, invariants,
  inherited locks with proof commands, acceptance gate.
- `tasks/branding-assets-shape.md` — decisions D1-D4, falsifiable invariants,
  boundary contracts, module map, risks/mitigations/oracles, hygiene notes,
  sizing input for the planner.

Track: `branding-assets`, unregistered post-M28 track (hotspot-canvas
precedent; ledger row at kickoff, flip at closeout). No implementation in this
plan — this is the frozen shape the planner sizes handoffs from.

## Key decisions

1. **App icon (AT-svg.svg)** — new embedded asset `crates/ui/assets/icon.svg`
   (byte-identical copy), `ICON_SVG` const in `lib.rs`, new route `/icon.svg`
   reusing the `ui_asset()` no-store builder with
   `Content-Type: image/svg+xml`; `<link rel="icon">` in `index.html` head;
   `.logo-badge` bolt SVG replaced by `<img class="brand-mark"
   src="/icon.svg">`. `themes.css` untouched — brand art is
   theme-independent (accepted first; 3-theme constraint preserved via
   untouched theme suites). Inlining the SVG into `index.html` was rejected
   (hex-literal drift guard + SVG id collisions).
2. **Extension image (AT.jpg)** — `extensions/ariadne-extension/media/
   banner.jpg` (byte copy) as README banner; marketplace `"icon"` requires a
   square PNG, so a derived `media/icon.png` + `"icon": "media/icon.png"`
   ships only if a byte-safe rasterizer (Pillow) exists on this machine —
   otherwise banner-only with the PNG deferred; `"icon"` never points at a
   missing file. No webview exists in the extension (verified) — README is the
   only banner surface.
3. **Source of truth** — root `media/` is never edited; downstream copies are
   owned by their handoff (app copy owned by A, extension copies owned by B);
   byte-identity checked by hash equality.
4. **Contract riders in handoff A (declared, touch-set-listed)** —
   `tools/ui_smoke.py` `ROUTE_ASSERTIONS` gains `("/icon.svg",
   "image/svg+xml", True)` (mirror contract); `tools/theme_token_check.py`
   adds `icon.svg` to `TOKEN_FILES` (new brand-art file would otherwise fail
   both the added-literal and frozen-count scans unconditionally).
   `crates/ui/src/export.rs` `standalone_template()` inlines the icon SVG and
   drops the favicon link so the `file://` export stays closed-world.
5. **Sizing** — 2 handoffs: A = `crates/ui` (+2 rider lines in tools/, ~6-8
   gates, ~150 diff lines + 56-line asset); B = `extensions/ariadne-extension`
   (~4-5 gates, ~20 diff lines + binaries). Binary additions declared:
   `icon.svg`, `banner.jpg`, optional `icon.png`.

## Constraints recorded (kickoff §locks)

Byte-safe copies only (never PowerShell pipes, AGENTS.md); pure-ASCII authored
text; "Ariadne's Thread" possessive everywhere; `encoding_guard.py --all` green
(its binary heuristic is NUL-presence — verified risk R1 on the jpg);
`theme_token_check.py` green via the TOKEN_FILES exemption; `ui_smoke` PASS
with the extended route table; hygiene ceilings — `server.rs` already 1120
lines (add ~8 minimally, extraction is a future finding, not this track).

## Open risks (top)

- **R1**: a NUL-free `AT.jpg` would trip `encoding_guard.py`'s "binary"
  heuristic -> STOP-report at first staging; no silent guard edit.
- **R3**: rasterizer availability for `icon.png` is unverified on this host ->
  banner-only fallback defined; PNG follow-up with reason.
- **R4**: `file://` export closedness handled in `export.rs`; oracle is the
  extended `test_standalone_export_inlines_all_split_assets` (zero
  `/icon.svg` references).
