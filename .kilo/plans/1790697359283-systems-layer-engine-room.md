# Systems Layer — Architecture ("Engine Room") View for AriadnesThread

**Emphasis: smooth, intuitive, and fun user experience.** Every feature below is judged first on feel: continuous camera motion, always-know-where-you-are wayfinding, and delight animation driven by *real analysis metrics* (no fake gimmicks).

Add a zoomable architectural layer over the existing graph UI: 30,000-foot view of user-defined **components** rendered as an animated engine room, zooming down into the existing include/symbol graph and source drawer. Deep search spans symbols, files, components, and notes. Components + markdown notes are user-authored, git-tracked files; the live server can create/edit them via new POST endpoints.

## Confirmed decisions (interview)

1. **Integrated semantic zoom** — same canvas/camera; Systems layer ↔ existing graph crossfade on zoom threshold / double-click; selection opens existing inspector + source drawer.
2. **Storage**: git-tracked `architecture/` dir at the analyzed repo root (never under `artifacts/`, which is fingerprint-invalidated cache). One markdown file per component with **TOML frontmatter** (`+++` fences) + body = attached doc.
3. **Auto-mapping v1 = heuristics only** (label-propagation communities + directory seeds). Schema leaves room for future LLM-generated suggestions; no LLM in v1.
4. **Visuals**: engine-room language (gear glyphs, pipe edges, heat) rendered **exclusively with the existing configured token palette** (`--bg-*`, `--cyan`, `--indigo`, `--amber`, `--red`…). No new palette, no new render library.
5. **Authoring**: in-browser via new POST endpoints; the `.md` files remain the single source of truth (read-modify-write preserves note bodies). Standalone exported HTML stays read-only.

## Experience contract (Smooth · Intuitive · Fun)

Concrete, verifiable requirements — each maps to acceptance checks in the task list.

**Smooth**
- One camera for both layers. All zoom/pan/layer changes route through a tiny `tweenCamera()` helper in `systems.js` (easeInOutCubic, 220–300 ms) — no instant jumps, ever. Wheel zoom keeps existing direct manipulation (no tween fight; tweens are for programmatic moves only).
- Layer switch = 250 ms opacity crossfade + gear→file "unfurl": component's member nodes scale 0.9→1 with ≤50 ms stagger (cap 8 staggered, rest instant). Crossfade is one-way per gesture — no flicker on threshold hover (10 % hysteresis band on zoom scale).
- Animate only compositor-cheap properties: `transform`, `opacity`, `stroke-dashoffset`, CSS vars. Gears spin via CSS `animation`, **zero JS timers/rAF loops for ambience**; one rAF is permitted solely inside `tweenCamera`. Animations `animation-play-state: paused` when their layer is hidden.
- `prefers-reduced-motion: reduce` → all transitions become instant cuts, gears render static with a "static engine" badge. Target: no dropped frames with 150 gears + 400 pipes (fade gears past 150).

**Intuitive**
- Location is never ambiguous: breadcrumb extends to `Systems ▸ <Component> ▸ <File>` (reuses thread bar pattern); Esc steps *out* one level (file → component → Systems), matching existing Esc semantics.
- Affordances telegraph: gear hover = 2 px lift + `--cyan-glow` + tooltip (name, member count, load, note-exists dot); `zoom-in` cursor + pulsing pinion-hole hint on dbl-clickability; pipes glow their source gear on component hover to show "what drives what".
- Selection actions: with file(s) selected anywhere, an **Assign to component…** action (existing/new) appears in the inspector — no hidden gestures.
- Empty Systems state teaches by doing: 3-step micro-guide ("select files → name it → watch it spin") + one-click **Generate starter engine from suggestions** (top suggestions, cap 10). Hooks into the existing tour/help-center welcome pattern.
- Search groups results by type with badges `SYS | DOC | SYM | FILE`, systems/docs ranked first; quick-open covers components + notes with the same keyboard flow as files.

**Fun (data-driven, no gimmicks)**
- Engine start-up: first entry into Systems ramps gear rotation 0→metric speed over ~1.2 s, once per session.
- Gear RPM ∝ real fan-in/closure load; hottest decile puffs a few steam sprites; pipe dash-flow speed ∝ coupling edge count — the machine visibly "works harder" where the build hurts.
- **Ariadne's Thread moment**: on zoom-into-component, one cyan thread stroke draws (~400 ms, `stroke-dashoffset`) from the clicked gear's center into the emerging file graph — the product metaphor made literal. Selecting a gear gives a 90 ms scale pulse ("clunk").
- Out of scope: sound (browser autoplay hostility + noise), mascot, confetti.

## Ground facts (verified)

- API is axum, currently **GET-only** (`crates/ui/src/server.rs:124-149`); assets embedded via `include_str!` (`crates/ui/src/lib.rs:9-16`), served `no-store`.
- `SymbolId(pub String)` is a plain symbol-name string (`crates/index/src/types.rs:6`); `FileId` re-derives from a relative path (`crates/repo/src/types.rs`) → stable keys for user files.
- No component/subsystem concept exists anywhere; no user-writable persistence today. Atomic tmp+rename write pattern to copy: `crates/scheduler/src/tu_store.rs:26-63`.
- Server boot/watcher: `run_ui` → `build_ui_payload` → `UiState::install_analysis` generation-swap (`server.rs:102`); watcher loop in `crates/cli/src/commands/ui.rs` (`spawn_watcher`).
- Client search is payload-local (`graph.js:794-955`, `hotspots.js:5-178`); `/api/graph/search` (`server.rs:573-666`) serves the VS Code extension.
- Hygiene: files < 1000, functions < 100 lines. `server.rs` = 773, `graph.js` = 990 (no room), `styles.css` = 1422 (already over — do NOT grow). CI gate: `tools/hygiene_check.py`.
- New deps needed (none in workspace): `toml`, `pulldown-cmark` — local pin in `crates/ui/Cargo.toml`, matching `regex = "1.10"` style.
- Export payload is byte-stable with pinned LF fixtures (commit 085a74b) → payload schema changes need a fixture-regeneration commit.

## File formats (precise)

```
<analyzed-repo>/architecture/
  components/<slug>.md                # slug = kebab-case; stable id = frontmatter id (default: slug)
  symbols/<sanitized-symbol-id>.md    # optional symbol-attached notes
```

```toml
+++
id = "ui-server"
name = "UI Server"
files = ["crates/ui/**", "services/gateway/*.h"]   # repo-relative paths or globs
symbols = ["UiState", "Graph*"]                     # SymbolId-name globs
+++
```
Body = free markdown = the component's attached doc. `symbols/<id>.md` frontmatter: `symbol = "<exact SymbolId>"`; filename = SymbolId with `[^\w.-]`→`_`, `--<crc32>` suffix on clash.

Semantics: components are **lenses, not partitions** — multi-membership allowed; unassigned files render dimmed. Unresolved patterns are reported, never silently dropped.

## Server-side design

New module `crates/ui/src/architecture/` (keeps `server.rs` under budget):

- `model.rs` — `ComponentSpec` (serde TOML), `ResolvedComponent { id, name, file_ids, symbol_ids, unresolved_patterns, stats }`, `SymbolNote`, `ArchitectureSnapshot`.
- `loader.rs` — scan + parse (per-file isolation: broken file → `Broken` entry with parse error, rest load); resolve globs via path→FileId re-derivation; re-run on boot, analysis swap, POST, and watcher mtime.
- `render.rs` — notes → HTML via pulldown-cmark with raw HTML disabled (repo content is untrusted in-browser; XSS test required).
- `write.rs` — create/update/delete `.md`: read-modify-write preserving body, atomic tmp+rename (copy `tu_store`); strict slug/path validation (no `..`, no absolute, stays inside `architecture/`).
- `routes.rs` — sub-`Router` merged in `build_router`:
  - `GET /api/architecture` → components + suggestions + symbol notes (`note_html` + `note_markdown`, 64 KiB/note cap with `truncated` flag).
  - `POST /api/architecture/component` (create from selection or accepted suggestion; prefills glob from LCP directory + file list)
  - `POST /api/architecture/component/<id>/membership` | `/note` | `/delete`; `POST /api/architecture/symbol-note` (upsert).
  - Every mutation reloads `UiState`'s `RwLock<ArchitectureSnapshot>`. Standalone export has no POST surface by construction.

DTO additions ride `GraphExportPayload` with `#[serde(default)]` so standalone reports embed architecture + rendered notes (fixture-regeneration commit).

Suggestions (deterministic): `crates/graph/src/community.rs` — label propagation over undirected include adjacency (seed order = FileId order, bounded iterations, stable tie-break), drop communities < 3 files; name = dominant directory prefix; score = internal-edge density. Surfaced as `suggestions[]`; "Accept" = a component POST.

`/api/graph/search` extension: results may be `kind = component | note` with label + note snippet, so the extension and tests get architecture hits; unchanged behavior when `architecture/` is absent.

## Frontend design (crates/ui/assets)

New files only (existing budget is spent):

- `systems.js` — Systems layer in a second `<g>` inside `#viewportGroup`: gear glyphs (teeth ∝ members clamped, radius ∝ members, spin duration ∝ load), pipe edges aggregating inter-component includes (existing path style + dash-flow), dag-column-style layout (own copy; do not refactor graph.js at 990), `tweenCamera()` helper, threshold crossfade with hysteresis, unfurl animation, thread-draw transition, tooltips/hover states, inspector Systems section (members + jump, unresolved warnings, rendered note, edit/delete/`+file`).
- `systems.css` — all engine-room styling + keyframes; **only `var(--token)` usage**; reduced-motion block.
- `main.js` / `graph.js` / `hotspots.js` — minimal hooks only: mode registration, zoom-state callback, search-corpus injection (`SYS`/`DOC` entries into `compileMatcher` + quick-open). Anything beyond a hook lives in `systems.js`.
- Register in `lib.rs` `include_str!`, `server.rs` `ui_asset` fixed routes, `index.html` tags. Return-to-Systems uses a `#systemsRestoreTab` (restore-tab convention; **no toolbar toggle buttons** — per saved contract).

## Tasks (ordered, RGB-TDD: failing test first for every Rust task)

1. **Loader + model** (ui): fixtures in `crates/ui/tests/` — frontmatter parse, id/slug defaults, multi-membership, glob→FileId resolution, unresolved reporting, broken-file isolation, missing dir → empty snapshot, symbol-note filename sanitize + collision. Red → `model.rs`/`loader.rs` + `toml` dep → Green.
2. **Community detection** (graph): `crates/graph/tests/community_proposal.rs` — two cliques + bridge → correct split; determinism (byte-identical repeat); min-size filter; disconnected input. Implement `community.rs`.
3. **Snapshot DTO + boot wiring + GET route**: contract test (shape, truncation cap, XSS escaped — no HTML passthrough); `UiState` wiring + re-resolve on swap/watcher; export embedding with `#[serde(default)]` back-compat; regenerate pinned export fixture (separate commit).
4. **Systems layer rendering + motion core**: `systems.js`/`systems.css` — camera tween helper, gear/pipe rendering, layout, heat mapping, crossfade + hysteresis, engine start-up, thread-draw, unfurl, reduced-motion. **UX acceptance:** zoom round-trip feels continuous on AT's own repo; no jank with fixture-scale graphs; reduced-motion = instant, static.
5. **Wayfinding + component scoping**: breadcrumb levels + Esc step-out, dbl-click → existing include graph filtered to component files + ghosted 1-hop externals, restore tab, inspector Systems section + note panel + gear hover tooltip/lift/pulse. **UX acceptance:** user can state their location at all times mid-navigation; each transition has an obvious back path.
6. **Search integration**: client corpus + badges + grouped results + quick-open; server-route tests (component hit, note snippet hit, regression parity, absent-dir parity). **UX acceptance:** typing a component name reaches it in ≤2 actions.
7. **Write endpoints + authoring flows**: `architecture_write.rs` tests (create, membership add/remove, note save preserves untouched body, delete, traversal/absolute/slug rejection); UI: "Create component from selection", "Accept suggestion", empty-state 3-step guide + "Generate starter engine", note edit textarea + save. **UX acceptance:** first component exists within 3 clicks from a cold empty Systems view.
8. **Hygiene + sweep**: `cargo test`; `cargo clippy` warning-free baseline; `python tools/hygiene_check.py` (allow-list only if fixtures demand); UI empty-state text notes `architecture/` is meant to be committed.

## Risks / edge cases

- **Stale membership after refactor**: paths are keys → renamed files surface as visible unresolved-pattern warnings, never silent loss.
- **Payload growth in exports**: per-note 64 KiB cap + sorted-map serializer keeps byte-stability; fixture update commit.
- **Write races** (user edits while server POSTs): read-modify-write + atomic rename; last-writer-wins acceptable v1 (documented).
- **XSS**: notes untrusted → server-rendered with HTML disabled; UI inserts only renderer output.
- **Cycle-heavy repos**: label propagation may yield one mega-community; min-size + suggestion cap 10; hand-authoring always available.
- **Camera conflict (two layers, one viewportGroup)**: pointer-events toggled by active mode; hysteresis prevents threshold thrash.

## Out of scope (v1)

LLM-assisted mapping (`suggestions[].source = heuristic | llm` reserved); VS Code extension surfaces (server route already provides hits); freeform diagram canvas; sound/mascot/confetti; light-theme palette work (owned by the pending token-sweep plan — `systems.css` inherits it for free by consuming only tokens).

## Validation summary

`cargo test` green; `cargo clippy` warning-free; `python tools/hygiene_check.py` passes; tests in `crates/ui/tests/architecture_*.rs` + `crates/graph/tests/community_proposal.rs`. Manual UX walkthrough on AT's own repo: seed 3–4 components via the empty-state guide, then — zoom round-trip continuous and jank-free; breadcrumb/Esc always locates; search finds a component in ≤2 actions; note renders (and a `<script>` probe stays inert); create-from-selection round-trips to disk byte-sensibly; standalone `--export` boots offline with read-only Systems layer; reduced-motion path fully usable.
