# Shape — branding-assets

Artifacts produced by the Architect; the planner sizes tasks from this. No
implementation here.

## Assets (read on disk, verified)

- `media/AT-svg.svg` — 55 lines, pure ASCII, 256x256 viewBox, dark rounded
  square `#0A0F1D`, neon thread gradient (`#FF2A54` -> `#FF9B26`), ids
  `neonGradient`/`neonGlow`. Square — suitable as app favicon, header mark, and
  marketplace-icon source.
- `media/AT.jpg` — photographic wide banner (~16:9) with the wordmark "ARIADNE'S
  THREAD — CODE DEPENDENCY ANALYZER". Not square — README/marketplace banner
  material, NOT a marketplace icon.

## Decisions (the shape of the change)

### D1 — App side: icon.svg as served brand asset

The UI embeds every asset via `include_str!` (crates/ui/src/lib.rs) and serves
them with no-store (server.rs `ui_asset()` contract; ui_smoke mirrors the route
table). The icon joins that pattern as a text asset:

- New file `crates/ui/assets/icon.svg` — byte-identical copy of
  `media/AT-svg.svg` (byte-level copy only; never PowerShell text pipes).
- `lib.rs`: `pub const ICON_SVG: &str = include_str!("../assets/icon.svg");`
- `server.rs`: route `/icon.svg` -> handler calling the existing
  `ui_asset(crate::ICON_SVG, "image/svg+xml")`. No-store + 200 come free from
  the shared builder. No new serving machinery.
- `index.html` head: `<link rel="icon" type="image/svg+xml" href="/icon.svg">`.
- `index.html` header: inside `.brand`, replace the `.logo-badge` inline bolt
  `<svg>...</svg>` child with `<img class="brand-mark" src="/icon.svg"
  alt="Ariadne's Thread logo">`. No hex literals enter the HTML.
- `styles.css`: adjust `.logo-badge` (fixed ~32px rounded box; the bolt-only
  `.logo-badge svg` rule replaced by a `.logo-badge img { width:100%;
  height:100% }`-style rule). The `.logo-badge` background gradient rule MAY
  stay (hidden behind the opaque icon) or go; either way no new hex literals
  and `themes.css` is not touched. If the gradient rule is removed,
  `--brand-grad-a/b` become defined-but-unused in themes.css — accepted: the
  D7 ledger note forbids deleting shared var names here; orphan cleanup is a
  follow-up.
- Rejected alternative: inlining the full SVG into `index.html`. It would add
  ~12 hex literals to a scanned asset (theme_token_check violation), risk
  `neonGradient`/`neonGlow` id collisions with existing page SVGs, and bloat
  the page. Serving it keeps one copy, theme-independent, cache-honest.

### D2 — Extension side: jpg banner now, PNG icon decision gated

- `extensions/ariadne-extension/media/banner.jpg` — byte-identical copy of
  `media/AT.jpg`. `README.md` gains a banner image near the top
  (`![Ariadne's Thread](media/banner.jpg)`, ASCII alt text). `.vscodeignore`
  has no media-excluding entry — verified — so it packages.
- VS Code marketplace `"icon"` in package.json MUST be a square PNG (>=128px).
  A JPG fails vsce packaging. Resolution: commit a derived
  `extensions/ariadne-extension/media/icon.png` (128x128 recommended) and set
  `"icon": "media/icon.png"`. Source for the PNG: the jpg's square logo
  region (left band) or the SVG rasterized — but the implementer must produce
  it byte-safely (Python/Pillow script in one shot, not PowerShell), and the
  Pillow availability is an OPEN QUESTION. If Pillow (or another
  already-installed rasterizer) is unavailable, handoff B lands the banner +
  README only and records the PNG conversion as a deferred follow-up with the
  reason; `"icon"` field is NOT set without a committed PNG (a dangling
  `"icon"` path breaks packaging).

### D3 — Source-of-truth and ownership

`media/` (repo root) is the source of truth and is never modified. Downstream
copies: `crates/ui/assets/icon.svg` owned by the UI handoff (A),
`extensions/ariadne-extension/media/{banner.jpg,icon.png}` owned by the
extension handoff (B). Ownership rule: to re-brand, edit `media/` then re-sync
copies (a sync check is I3; no build-time sync machinery — YAGNI at 2 copies).

### D4 — Contract mirrors that must change atomically (declared riders)

- `tools/ui_smoke.py`: `ROUTE_ASSERTIONS` gains `("/icon.svg",
  "image/svg+xml", True)`. The file's own docstring says the table mirrors
  `build_router` + the page's loads; leaving it stale breaks the mirror
  contract. One appended tuple.
- `tools/theme_token_check.py`: `icon.svg` is sanctioned brand art; exempt it
  from BOTH scans the same way `themes.css` is exempted (add to `TOKEN_FILES`,
  update the comment: the SVG's literals ARE the fixed brand palette). Without
  this, the frozen-count scan fails the new file (baseline count 0) and the
  added-literal scan fails it unconditionally (new file, empty base text).
  The alternative (adding `crates/ui/assets/icon.svg: N` to the baseline JSON
  via `--update-baseline`) fixes only half: the added-literal scan exempts
  only TOKEN_FILES, so the exemption is required regardless.

These two tools files are handoff-A riders, justified in the handoff touch-set:
each is a one-line mirror of a server.rs contract change; splitting them into
a third handoff invites the drift they exist to prevent. Sizing note for the
planner/reviewer: handoff A spans `crates/ui` + 2 lines in `tools/` — declared
scope deviation on merit, not silence (ledger law from M28 T0b finding 1).

## Invariants (each independently checkable; falsifier named)

1. Router-level: GET `/icon.svg` -> 200, `Cache-Control: no-store`,
   `Content-Type: image/svg+xml`. Falsifier: new test
   `crates/ui/tests/ui_test/icon_asset.rs` (extend
   `static_html.rs::test_ui_asset_routes_serve_bundled_content_with_no_store`
   pattern via `common::request_raw_body`; append-only elsewhere).
2. `python tools/ui_smoke.py` exits 0 AND its `ROUTE_ASSERTIONS` contains the
   `/icon.svg` row. Falsifier: the smoke run + a grep-able table row.
3. Byte-identity: `crates/ui/assets/icon.svg` == `media/AT-svg.svg` and
   `extensions/ariadne-extension/media/banner.jpg` == `media/AT.jpg`
   (byte compare, e.g. `git hash-object` equality). Falsifier: hash mismatch.
4. Theme-token guard green: `theme_token_check.py --check-added --base <rev>`
   exit 0; `themes.css` zero-diff (git diff empty); no new hex literal in
   `index.html`/`styles.css`. Falsifier: guard exit 1 or nonempty themes diff.
5. Encoding guard green with the new blobs: `encoding_guard.py --all` exit 0.
   Falsifier: exit 1 ("not utf-8 decodable" on the jpg, or mojibake signature
   in the SVG).
6. Standalone export closed-world: exported HTML (via
   `export_standalone_html` on a fixture payload) contains zero occurrences of
   `/icon.svg` and still contains zero occurrences of `/styles.css`,
   `/themes.css`, `/app/` (existing lock). Falsifier: extended
   `test_standalone_export_inlines_all_split_assets` (append assertion).
7. Extension packaging validity: `package.json` parses as JSON; IF `"icon"` is
   set, the path exists and the file is a PNG >=128x128 (magic-bytes + IHDR
   check in a tools/ test or gate command); `"icon"` unset iff no PNG
   committed. Falsifier: gate command exits nonzero.
8. Text hygiene: all authored text pure ASCII; every new user-visible string
   spells "Ariadne's Thread" (possessive). Falsifier: encoding guard +
   review-gate literal check.
9. Three-theme behavior: theme suites UNEDITED and green; brand mark renders
   identical across light/dark/dark-steampunk (fixed art by design). Falsifier:
   theme suite edits or failures. (No automated pixel test — journal
   eyeball line per hotspot precedent.)

## Contracts (boundary shapes)

- Route: `GET /icon.svg` -> 200 | headers `Cache-Control: no-store`,
  `Content-Type: image/svg+xml` | body = `ICON_SVG`. Errors: none (embedded
  constant, same infallible builder as all assets).
- Export: `standalone_template()` additionally replaces
  `<img class="brand-mark" src="/icon.svg" ...>` with the inline `<svg>`
  markup (ICON_SVG inlined as a real element; `file://` has no routes) and
  drops the favicon `<link rel="icon" href="/icon.svg">` tag (favicon is
  meaningless under `file://`). Post-condition = invariant 6. Note: inline id
  collision (`neonGradient`/`neonGlow`) is safe in the export because no other
  inline SVG in the page uses those ids — but the app page must keep the icon
  as `<img>` (never inlined) precisely to keep that guarantee cheap.
- `ui_smoke.ROUTE_ASSERTIONS`: tuple row `("/icon.svg", "image/svg+xml",
  expect_no_store=True)`.
- `theme_token_check.TOKEN_FILES`: `("themes.css", "icon.svg")` (comment:
  sanctioned brand-art palette).
- `extensions/ariadne-extension/package.json`: adds `"icon":
  "media/icon.png"` (only when the PNG ships). No other manifest changes.
- README.md: banner paragraph; ASCII only.

## Module map

Touch-set A (app handoff, `crates/ui` + 2 rider lines):
- ADD `crates/ui/assets/icon.svg` (byte copy)
- EDIT `crates/ui/src/lib.rs` (+1 const)
- EDIT `crates/ui/src/server.rs` (route + handler, ~8 lines)
- EDIT `crates/ui/src/export.rs` (+1/−1 replaces in `standalone_template`)
- EDIT `crates/ui/assets/index.html` (head link + `.logo-badge` swap)
- EDIT `crates/ui/assets/styles.css` (`.logo-badge`/`.brand-mark` rules; no hex)
- EDIT `crates/ui/tests/ui_test/static_html.rs` (append-only export assertion)
  or ADD `crates/ui/tests/ui_test/icon_asset.rs` (new route test) — append-only
  discipline either way
- EDIT `tools/ui_smoke.py` (rider: +1 route row)
- EDIT `tools/theme_token_check.py` (rider: TOKEN_FILES + comment)

Touch-set B (extension handoff, `extensions/ariadne-extension` only):
- ADD `extensions/ariadne-extension/media/banner.jpg` (byte copy)
- ADD `extensions/ariadne-extension/media/icon.png` (derived; conditional —
  see D2 fallback)
- EDIT `extensions/ariadne-extension/package.json` (`"icon"`; conditional)
- EDIT `extensions/ariadne-extension/README.md` (banner)

Must-not-touch: `themes.css`, `media/**` (source of truth),
`ariadne-extension-0.1.0.vsix`, `tests/fixtures/**`, `tests/goldens/**`,
`crates/ui/assets/main.js` and all other app JS, `extension.ts` (no webview —
verified), `package-lock.json`, ledger outside closeout ritual, theme suites.

## Risks -> mitigations -> oracle

| # | Risk | Mitigation | Oracle |
|---|------|-----------|--------|
| R1 | `encoding_guard.py` treats "binary" as "contains NUL"; a NUL-free jpg would fail as non-UTF-8 | Verify at first staging; realistic jpeg entropy streams contain 0x00 | `encoding_guard.py --staged`; on failure STOP-report (no silent guard edit; guard change would need its own dated ledger row) |
| R2 | `icon.svg` new hex literals break theme guard | TOKEN_FILES exemption (D4), committed atomically with the asset | `theme_token_check.py --check-added` + `--update-baseline` NOT needed (exempted from frozen counts too) |
| R3 | No image rasterizer on this Windows box for icon.png | Banner-only fallback path defined in D2; PNG deferred with reason; `"icon"` never points at a missing file | gate command: JSON parse + magic-byte/IHDR check script (tools/), or reviewer manual check |
| R4 | Standalone export shows broken icon / dangling route under `file://` | D6 export replacements | extended `test_standalone_export_inlines_all_split_assets` (invariant 6) |
| R5 | tools/ rider lines read as scope creep | Declared in handoff touch-set with the mirror-contract reason; review-gate enforces commit-vs-touch-set EXACT | review-gate step: touch-set diff |
| R6 | Browser without SVG-favicon support shows no tab icon | Accepted; `/favicon.ico` bitmap fallback is an out-of-scope follow-up | none (cosmetic degradation only) |
| R7 | Theme suites assert hidden expectations about `.logo-badge` markup | Verified by grep: no test references `logo-badge`, the bolt path, or `brand-title` markup; only new asserts append | `cargo test -p ariadne-ui` (0 failed) |
| R8 | Stale `.vsix` misleads packaging tests | Out of scope; closeout records "vsix rebuild = operator duty" | n/a |

## Hygiene notes (plan-for-now, not discover-later)

- `crates/ui/src/server.rs` is ALREADY 1120 lines — over the 1000 ceiling
  (pre-existing). This change must add minimally (~8 lines). Do NOT fix here;
  record as a finding for a future server.rs extraction handoff (asset
  handlers are a natural extraction unit: `ui_asset` + handlers ~= 60 lines).
- `static_html.rs` is ~950+ lines and R2-adjacent (its own test caps edited
  files at `<= 998` per the ledger relocation row): prefer a NEW test file
  `icon_asset.rs` over growing it; if the export assertion must append there,
  check the per-test caps first.
- `index.html`/`styles.css`/`themes.css` line budgets unaffected (<30 lines of
  churn). `extension.ts` untouched.
- New tools gate scripts (if B needs a PNG-validity check) must follow tools
  conventions: argparse strict, exit 0/1/2, distilled-run friendly, pure ASCII.

## Sizing (planner input, not a task list)

- Handoff A (app): touch-set A; ~6-8 gate commands (scoped ui testrun --distill,
  ui_smoke, theme_token_check, encoding_guard --staged, hygiene_check, scoped
  commit). Est. <= ~150 diff lines + 56-line asset. Fits one handoff.
- Handoff B (extension): touch-set B; ~4-5 gate commands (JSON parse, encoding,
  optional PNG validity check, optional `npm run compile` sanity, scoped
  commit). Est. <= ~20 diff lines + binary assets. Fits one handoff.
- Binary additions to declare in both handoffs: `icon.svg` (55 lines of text),
  `banner.jpg` (size not measured from the architect seat — implementer records
  the byte size in the journal), optional `icon.png` (small).
