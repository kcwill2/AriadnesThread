# UI Theming: Full Token Sweep + Three Themes

## Goal

Make the graph UI themeable. Ship **three** themes — `light`, `dark` (the current "slate" look), and `dark-steampunk` — behind an explicit user-facing selector, by first converting every hardcoded color in `crates/ui/assets/` into a declared CSS custom property.

The token architecture is N-theme from the start: one token block per theme, a parity test that iterates the set, a selector with N options. Steampunk is a third palette block plus a third `<option>` — not a redesign.

## Decisions (agreed)

| # | Decision | Rationale |
|---|---|---|
| D1 | **Full sweep, canvas included** | `gui_scrollbar_style.rs:111` currently locks graph hexes out of scope. Overturned: its stated reason ("they render on the canvas") is an argument *for* tokenizing, since the canvas is the surface that must follow the theme. A light theme with dark node cards is unusable. |
| D2 | **Named bg/text token pairs for tints** | Dark-specific tints (`#450a0a`, `#052e16`, `#082f49`) are surface-dependent and cannot be shared. Explicit named tokens per theme are statically testable; `color-mix` is not. |
| D3 | **Explicit selector, all three themes in one change** | Toolbar `<select>`; `localStorage` persistence; blocking `<head>` script applies before first paint (no flash); parity test asserts every palette defines every token. |
| D4 | **Selector in toolbar-right as `.select-dark`** | Reuses the existing control idiom, inheriting the focus/hover tests at `gui_scrollbar_style.rs:87`. |
| D5 | **New `assets/theme.js` for theme logic** | `graph.js` is at 990/1000 lines — 10 from the R2 hygiene gate. New logic must not land there. |
| D6 | **Steampunk retints chrome, locks verdicts** | Steampunk's signature color (brass/amber) collides with the semantic amber verdict. Retinting verdicts would make "amber = partly needed" indistinguishable from "amber = brass". So: decorative layer goes brass/copper/copper-patina/aged brass; `--green`/`--amber`/`--red` verdicts and the 5-tier heat ramp keep their semantics, retuned only for contrast on the warmer surface. |
| D7 | **Steampunk reuses `--cyan` as its brass accent token** | `--cyan` is the existing decorative accent (focus rings, `[ROOT]` badge, logo, scrollbar hover, ~19 `var()` uses). Rather than introduce `--accent` and rewrite 19 call sites for no visual gain, steampunk declares `--cyan` as brass/gaslight-gold. Verdicts are unaffected because they use `--green`/`--amber`/`--red`. |
| D8 | **No `prefers-color-scheme` auto-detect** | A light theme silently repainting a dark-first analysis tool is unwanted; user chooses explicitly. |

### Semantic vs decorative, per theme

The two layers must be reasoned about separately when authoring the steampunk palette:

| Layer | Tokens | dark | light | dark-steampunk |
|---|---|---|---|---|
| **Decorative** | `--cyan` accent, `--brand-grad-*`, `--indigo`, `--purple`, borders, shadows, scrims, scrollbars | cyan / blue | deep blue (needs >4.5:1 on white) | **brass / copper / patina** |
| **Semantic** | `--green` `--amber` `--red`, `--heat-0..4-*`, preview verdicts | unchanged | retuned for contrast on light tints | **semantically locked**, retuned only for contrast on warm surfaces |

A steampunk cycle card must still read "cycle" via rust-red, not "brass". The heat ramp must still read cool→hot.

## Constraints discovered

- `tools/hygiene_check.py:19` — `R2_SCANNED_SUFFIXES = {".rs", ".py", ".html", ".js"}`. **`.css` is exempt**, so `styles.css` (1422 lines) is not gate-bound, but do not grow it further; put theme palettes in `themes.css`.
- `graph.js` 990 lines, `index.html` 449, `main.js` 513, `preview.js` 354. `graph.js` is the binding one.
- Assets are `include_str!`'d into the binary (`lib.rs:9-16`) and inlined into the standalone export (`export.rs:9-23`). A new asset needs: `lib.rs` const, `server.rs` route + handler, `index.html` `<script>`, and `export.rs` inlining tuple. **The `export.rs` tuple must be updated or the standalone report silently loses the theme bootstrap.**
- `server.rs:127` serves `/styles.css`; the new `themes.css` needs a parallel route.
- `var()` in SVG presentation attributes already works here (`graph.js:346` uses `fill="var(--cyan)"`). No `getComputedStyle` plumbing needed.
- `index.html:206-224` — the 7 SVG `<marker>` defs have hardcoded `fill` values. These are the arrowheads and **must** be tokenized or light/steampunk arrows stay dark/cyan.
- `.select-dark` is a **class name that will now be a lie in 2 of 3 themes** (`index.html:125,133,155,184`). Decide in T7 whether to rename it (touches `gui_scrollbar_style.rs:87-108` selectors) or accept the stale name. Recommended: accept the name in this change and note it — the control is already fully token-driven, so the rename is cosmetic churn across tests.
- Steampunk is the only theme where a *dark* surface must host *warm* tints; the existing dark tints are all cool (`#082f49` blue, `#1e1b4b` indigo, `#2e1065` violet). The steampunk `-bg` tints must be genuinely warm or node cards will float on the warm canvas as cool patches.

## Token inventory

Existing `:root` (`styles.css:1-20`): 17 tokens. `--indigo` and `--purple` are **dead** (zero `var()` references) — repurpose or delete.

New tokens needed, as bg/text pairs (D2):

| Token | Current dark value | Used by |
|---|---|---|
| `--node-header-bg` / `--node-header-text` | `#082f49` / `#38bdf8` | `getNodeColors('HEADER')` |
| `--node-source-bg` / `--node-source-text` | `#1e1b4b` / `#818cf8` | `getNodeColors('SOURCE')` |
| `--node-class-bg` / `--node-class-text` | `#2e1065` / `#c084fc` | `getNodeColors('CLASS'\|'STRUCT')` |
| `--node-function-bg` / `--node-function-text` | `#052e16` / `#4ade80` | `getNodeColors('FUNCTION')` |
| `--node-other-bg` / `--node-other-text` | `#1e293b` / `#94a3b8` | `getNodeColors` default |
| `--node-cycle-bg` / `--node-cycle-text` | `#3b0d0d` / `#fca5a5` | cycle cards |
| `--heat-0-bg` … `--heat-4-bg` (+ `-text`) | `#06283d`/`#063721`/`#382a04`/`#431407`/`#450a0a` | `getHeatColor` 5 tiers |
| `--preview-eliminated-bg` / `-text` | `#450a0a` / `#fca5a5` | `preview-eliminated` |
| `--preview-kept-bg` / `-text` | `#052e16` / (green) | `preview-kept` |
| `--preview-still-bg` / `-text` | `#422006` / (amber) | `preview-still-reachable` |
| `--preview-proposed-bg` / `-text` | `rgba(30,27,75,.85)` / `#e9d5ff` | proposed header card |
| `--brand-grad-a` / `--brand-grad-b` | `#0284c7` / `#4f46e5` | `.logo-badge` gradient |
| `--overlay-scrim` | `rgba(0,0,0,0.5)` / `0.7` | modal + tour overlays |
| `--shadow-ambient` | `rgba(0,0,0,0.5)` | tour card shadow |

Plus per-verdict border/accent tokens already partially present (`--green`/`--amber`/`--red`/`--cyan`/`--purple`).

## Tasks

Order matters: T1 is a Red-Green refactor of the *tests* only, so every later task starts from a token-asserting baseline.

### T1 — Rewire the hex-asserting tests to accept tokens
Convert assertions to the `var(--x) || #hex` escape-hatch pattern already used at `node_verdict_spine.rs:54`. No production change in this task.

- `gui_scrollbar_style.rs:111-125` — **invert** `test_graph_chrome_hex_rules_untouched_by_token_sweep`: assert `.legend-card` and `.badge-expand` now use `var(...)`, with a comment recording D1 and why the original rationale was wrong.
- `preview_hypothetical.rs` (9 asserts, incl. exact-substring `:279`) — accept tokens.
- `static_html_sections.rs:232,250,264` — accept tokens.
- `edge_verdict_story.rs:36`, `node_verdict_spine.rs:54-64` — accept tokens.
- Preserve `gui_scrollbar_style.rs:59-84` (`#0d1117`/`#161b22` GitHub-palette ban) as-is; it is a palette-fidelity guard, not a token check.
- Extract the shared `var || hex` helper into `tests/ui_test/common.rs` rather than repeating it 18 times.

### T2 — `themes.css` with the full token contract
New `assets/themes.css`, three blocks:
1. **default** (no attribute selector) — the current `:root` values plus all new tokens from the inventory. This is today's slate dark, byte-for-byte where possible.
2. `[data-theme="light"]`
3. `[data-theme="dark-steampunk"]`

Plus a leading comment stating the D6 split (decorative vs semantic) so an editor knows which tokens are free to retint and which are not.

Design constraints per palette:
- **light** — tints must be *light*-surface tints (e.g. a red tint near `#fee2e2`, not `#450a0a`); text tokens must clear 4.5:1 on their own tint; verdicts survive, do not get "tuned" toward the theme.
- **dark-steampunk** — surfaces are warm and dark (near-black brown/charcoal, not blue-black); every `-bg` tint must be warm (brass/oxide/rust/patina) so node cards don't float as cool patches; `--cyan` = brass/gaslight-gold; `--brand-grad-*` = copper→brass; decorative indigos/purples → patina/oxidized-copper. Semantic `--green`/`--amber`/`--red` and the heat ramp keep their hues, adjusted only for contrast on the warm surface.

The **default** (no `data-theme` set) must be the current dark slate look, so a stale attribute or a JS failure degrades to today's appearance rather than an unstyled page.
Move the existing `:root` out of `styles.css` into `themes.css` so there is exactly one definition site.
Link order: `themes.css` first (tokens), then `styles.css` (rules). State and test this order.

### T3 — Server + export wiring
- `lib.rs`: `pub const THEMES_CSS: &str = include_str!("../assets/themes.css");`
- `server.rs`: `.route("/themes.css", get(asset_themes_css))` + handler mirroring `asset_styles_css` (`:174`).
- `export.rs:13-20`: add `("themes.css", ...)` — this is the tuple that must not be missed.
- `index.html`: `<link rel="stylesheet" href="/themes.css">`.
- Test: extend `static_html_sections.rs` to assert the standalone template contains the theme link replaced with the inlined `<style>`, mirroring the existing styles.css inlining assertion.

### T4 — Token sweep: `styles.css`
Replace all 42 remaining hexes outside `:root` with tokens. Special cases:
- `:269-283` `.cycle-btn` — `rgba(239,68,68,0.15)` etc. need an alpha-capable token (use `color-mix(in srgb, var(--red) 15%, transparent)` or a dedicated `--red-wash` token; pick one and use it consistently).
- `:1263` `.modal-overlay` and `index.html:356` tour overlay — `--overlay-scrim`.
- `:84` logo gradient — `--brand-grad-a`/`-b`.
- SVG marker fills in `index.html:206-224` (7 markers) — token each (`--text-sub`, `--cyan`, `--red`, `--green`, `--amber`, `--purple`, `--red`).
- `.badge-expand` (`:1266`) and `.legend-card` (`:939`) — the D1 exceptions, now tokenized.

### T5 — Token sweep: JS color tables
- `main.js:71-86` `getNodeColors` — return `var(--node-*)` strings.
- `main.js:91-98` `getHeatColor` — `var(--heat-N-*)`.
- `main.js:51-68` `legends` and `:100-106` `HEAT_LEGEND` — `color:` fields become `var(--*)`.
- `main.js:512-525` `insight.style.borderColor/background` inline rgba strings — token or `color-mix`.
- `preview.js:103-105,113,299` — tokens.
- `graph.js:311` text fill `#f8fafc` → `var(--text-main)`; `graph.js:552` `#ef4444` → `var(--red)`.
- Add a test asserting **zero** raw hexes remain outside `themes.css` in `main.js`/`graph.js`/`preview.js`.

### T6 — `theme.js`: selector, persistence, no-flash apply
New `assets/theme.js`:
- Single source of truth: `const THEMES = ['light', 'dark', 'dark-steampunk'];` plus `const DEFAULT_THEME = 'dark';`. The `<select>` options, the inline anti-flash snippet, and the parity test must all agree with this list — hardcoding the names in three places is how a fourth theme gets half-added.
- **Anti-flash:** a minimal inline `<script>` in `index.html:3-8` (before the stylesheet) sets `document.documentElement.dataset.theme` synchronously. `theme.js` (deferred) owns the selector wiring + persistence. This split is the reason `theme.js` must be a separate asset from the inline snippet. The inline snippet must be a *self-contained duplicate* of the read-and-apply logic (it cannot depend on `theme.js`, which loads at the end of `<body>`) — accept this duplication and comment both copies to say so, or the two will drift.
- `applyTheme(name)`: validate against `THEMES`, set attribute, persist to `localStorage['ariadne_theme']`, update `<select>` value, call `renderSvg()` — **required**, because node `fill`/`stroke` are set as attributes and the heat palette is recomputed per render.
- Unknown/stale stored value (e.g. a theme removed later) → fall back to `DEFAULT_THEME` and rewrite the key; do not leave a bogus `data-theme` on `<html>` that matches no block and silently yields unstyled tokens.
- Wire into `lib.rs`, `server.rs` route `/app/theme.js`, `index.html` script tag, `export.rs` inlining tuple.

### T7 — Toolbar selector UI
- `index.html:168-175` `.toolbar-right`: add the `.select-dark` `#themeSelect` with three options — `Light`, `Dark`, `Dark (Steampunk)`. Set the `dark` option `selected` in markup.
- Option labels are human-facing; the `value` attributes are the `THEMES` ids. Assert in test that the three `<option value>`s match the `THEMES` list.
- Test in `gui_scrollbar_style.rs`-style: assert `#themeSelect` uses `.select-dark` so it inherits the focus/hover contract already asserted at `:87-108`.
- Consider adding a `theme.js` step to the guided tour (`main.js:303-320` `tourSteps`) — optional, flag for review.

### T8 — Token-parity test
New test: parse the token names declared in the **default** block and in each of `[data-theme="light"]` and `[data-theme="dark-steampunk"]`; assert all N sets are **equal** to each other and non-empty. Iterating the block set (not a hardcoded two-theme list) is what makes this generalize when a fourth theme lands.
Also assert every `var(--x)` referenced in any asset is defined in the default block (catches typos and dead tokens like `--indigo`).
Also assert every theme id appearing in a `[data-theme="…"]` selector is present in the `THEMES` list in `theme.js`, and vice versa — this is the guard against a palette block with no way to select it.

### T9 — Docs
Note the theme selector in the Help Center (`index.html:386+`, `helpTab-shortcuts` or a new line in `helpTab-legend`) and in the guided tour steps if warranted. Record D1 in a code comment at the rewritten `gui_scrollbar_style.rs:111` test so the reversal is discoverable, and record D6 (steampunk locks verdicts) at the steampunk block in `themes.css` so a future editor doesn't "helpfully" retint the verdict ramp into brass.

## Risks

- **The two new palettes are the real risk.** Nothing statically verifies contrast. The parity test covers *presence*; a human must eyeball each theme on the graph canvas, the legend, and the preview overlay. Call this out in review — do not let "tests green" be read as "the palettes are legible." Two new palettes means twice the eyeball surface, not the same.
- **Steampunk can silently lose its verdicts.** D6 is a discipline, not a mechanism: a future editor sees a brass theme with green/amber/red cards and may retint them for "cohesion," destroying the semantic layer the whole edge-verdict story depends on. The D6 comment at the block plus the risk note are the mitigations; there is no test that can detect it.
- **Missing `renderSvg()` on theme change** leaves stale colors on every card until the next interaction. This is the most likely shipped bug; T6's explicit call is the fix and needs a manual check for all three themes.
- **`data-theme` attribute vs CSS specificity** — the default block must not be written as `:root {…}` if `[data-theme=…]` blocks also target the same specificity on `<html>`, or the default silently wins and all themes render identically. Verify by toggling, not by reading.
- **T3's `export.rs` omission is silent** — the standalone report would boot with no theme control and no error. Covered by the T3 test.
- **`graph.js` at 990 lines** — if T5 requires edits there, verify the line count after; prefer moving code out rather than adding.

## Validation

- `cargo test -p ariadne-ui` — all rewired assertions, 3-way parity test, zero-hex test, export-inlining test.
- `cargo clippy --all-targets` clean.
- `python tools/hygiene_check.py` — confirm `graph.js` and `index.html` are still under 1000 and no new function exceeds 100 lines.
- Manual, for **each** of the three themes: graph canvas, legend, heat map, preview overlay, arrowheads, inspector, source drawer, focus rings, scrollbars all repaint correctly; reload persists with no flash; standalone export works under `file://`.
- Manual, steampunk-specific: confirm the cycle card still reads as *cycle*, the heat ramp still reads cool→hot, and brass/amber chrome is not confusable with the amber verdict spine.

## Out of scope

- `prefers-color-scheme` auto-detection (D8 — explicit only).
- A fourth theme, and a runtime theme *registry* (themes stay a static list in `themes.css` + `theme.js`).
- Renaming the `.select-dark` class (see Constraints — accept the now-stale name).
- Re-planning `--indigo`/`--purple` beyond removing them if still dead after T5.
- Any change to verdict *semantics* — only their rendering moves behind tokens.
