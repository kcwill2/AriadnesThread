# Systems Layer — Concepts, Relations & Lenses ("Engine Room") for AriadnesThread

**Emphasis: smooth, fluid, intuitive graph navigation and view-editing.** The Experience section below states the strategies and measurable gates that make this real — treat them as acceptance criteria, not aspirations.

The user authors a **conceptual model** of the analyzed repo: concepts (subsystems containing components, mapped to code), relations between concepts whose *meaning the user decides*, notes, and named processes. The tool renders this as a zoomable engine room over the existing graph UI and reconciles it against real code — declared vs evidenced vs leaked. Five default **lenses** (structure, boundaries, dependency, flow, process) are configurations of one renderer; **user-defined views** are first-class saved lens configs.

## Confirmed decisions (interview)

1. **Integrated semantic zoom** — one canvas/camera; Systems layer ↔ node graph crossfade on zoom threshold / dbl-click; selection opens existing inspector + source drawer.
2. **Concepts, not components**: user-defined nodes mapped to code (file/symbol globs). A **subsystem is just a concept with children** (`parent =`) — composition comes from the hierarchy, not a special type.
3. **User-defined relations**: authored concept→concept; kind = curated seed vocabulary + free text (user determines meaning); evidence reconciliation against real code edges is the Boundaries/Flow payoff.
4. **No extractor work in v1**: derived channels consumed read-only (`includes`, semantic `Complete|Incomplete`, `calls`, `extends_edges`). Known gap, documented: C++ emits no `extends_edges` (`crates/lang-cpp/src/analyzer.rs:52`; only the Scala path fills it) — C++ `inherits` is declared-only in v1. Deriving C++ base classes or field-type composition is a separate pipeline milestone (bincode edge layout frozen, `crates/lang-api/src/types.rs:13-19`; additive parallel-channel discipline required).
5. **Processes = named code traces**: root (entry symbol/file) + relation (calls|includes) + optional depth, saved as markdown; selecting animates a glowing thread through the concepts the trace passes. Candidates seeded from the existing `/api/graph/entrypoints`.
6. **Heuristic suggestions v1, AI later**: include-graph community detection seeds concept suggestions; schema carries `source = heuristic | llm`.
7. **Visuals**: engine-room language, **tokens only** — three themes exist (`themes.css`, `data-theme` dark/light/dark-steampunk); `systems.css` uses `var(--token)` exclusively so it inherits all themes; reduced-motion respected.
8. **Authoring in-browser** via POST; git-tracked markdown is the single source of truth (read-modify-write preserves bodies). Standalone export stays read-only.

## Navigation fluidity & view-editing strategies (concrete)

### Navigation — engineering guarantees

- **S1 Retained-mode layers.** `graph.js` wipe-rebuilds all SVG per state change; the Systems layer must NOT copy this. `systems.js` builds its SVG once per data change; camera moves and layer crossfades only ever mutate the `viewportGroup` transform and layer `opacity`/`pointer-events`. Crossfade starts only when the target layer is `ready` (built); until then a skeleton placeholder renders — **no blank frames, ever**.
- **S2 Camera service.** One rAF driver; primitives `focusOn(box)` / `focusNode(id)` / `setZoomAt(point, k)`, easeInOutCubic 220–300 ms, **interruptible**: a new gesture captures the current interpolated transform (never snap). Wheel zoom stays direct-manipulation (untweened) to preserve feel; tweens are for programmatic moves (dive, Esc, search-focus, gear dbl-click). 10 % zoom hysteresis around the layer threshold kills thrash.
- **S3 Continuity invariants.** Selection id + breadcrumb survive every layer/tab switch; gear↔member correspondence is bidirectional and live: selecting a concept highlights its mapped files in the node layer, selecting a file highlights owning gears. Zoom round-trip restores camera within 2 px / 2 %.
- **S4 Input polish.** 4 px click-vs-drag threshold; invisible hit-pad (+8 px) around gear glyphs; `pointer-events` gated by active layer; keyboard map identical on both layers: `←/→` cycle concepts, `Enter` dive, `Esc` step out, `/` search, `v` view picker, visible focus rings.
- **S5 Perf budgets.** Ambient motion uses only `transform`/`opacity`/`stroke-dashoffset`; gears spin via CSS, single rAF (S2 only); ≤150 animated gears (past cap: static + dim, never paused-then-pop); layer transition ≤300 ms; `prefers-reduced-motion` → instant cuts, static gears + badge.
- **S6 Unfurl & Thread moments.** Dive = target members scale 0.9→1 (≤50 ms stagger, cap 8) + one cyan `stroke-dashoffset` thread drawn (~400 ms) from gear center into the graph; gear select = 90 ms scale pulse; RPM ∝ fan-in/closure load, steam on hottest decile, ghost belts (dashed) for declared-without-evidence relations.

### View editing — trust & fluidity model

- **S7 Optimistic writes + one-step undo.** UI updates immediately from the POST response (no refetch round-trip); every mutation response returns `{ ok, file, prior }`; the write layer keeps the prior file content in-process and `POST /api/architecture/undo` restores it byte-exactly. Undo chip ("Concept created — Undo") for 10 s after every mutation. Deeper history is git's job — say so in UI copy; no CRDT.
- **S8 Never author blind.** Debounced `GET /api/architecture/preview?glob=` returns match count + sample paths while the user types membership globs; the relation composer shows live evidence counts ("3 include edges / 12 calls will back this"); view parameter controls apply **live** to the canvas — "Save view" only persists what is already on screen (WYSIWYG) and is disabled when equal to the base lens.
- **S9 Direct manipulation.** Shift-click accumulate selection → "New concept from selection"; drag a file node onto a gear = add member (drop-hover shows gear lift + `+N` badge); drag gear→gear = relation composer popover (kind chips from registry, free-text with autocomplete over in-use kinds, detail field, evidence preview); inline rename changes `name` only — `id`/slug are frozen forever (identity stability, loader-tested), so renames never break relations, notes, or processes.
- **S10 One validation vocabulary.** The loader's validation table (unknown fields, unresolved globs, dangling relation targets, parent cycles) is the same code the POST validators and form error strings use — an error the UI can report is an error the loader can report.
- **S11 Inspector mounting without touching graph.js.** Systems/inspector sections mount via the proven `provenance.js` pattern (`#inspLabel` mutation mount, `provenance.js:1-5`) so the frozen `graph.js` (990 lines) stays untouched beyond registration hooks.
- **S12 Process traces computed client-side** by one BFS over payload edges (call/symbol/include as configured), used identically in live and standalone mode (payload is embedded both ways); no per-trace API. Depth default 4 (calls) / 2 (includes), cap 6.

### Measurable UX acceptance (verified in Task 5–9 walkthroughs)

- Zoom round-trip within 2 px / 2 %; selection + breadcrumb intact after every transition; zero blank intermediate states.
- Transition at ~60 fps on AT's own repo; ≤2 consecutive dropped frames; budgets of S5 held.
- Every mutation reflected ≤100 ms; undo restores prior bytes in one request; failed writes roll back with the exact field-level reason.
- Cold empty Systems view → selected concept ≤5 keystrokes; concept search ≤2 actions; glob typing → live match count within 300 ms after pause.

## Experience language (unchanged highlights)

Engine start-up ramp ~1.2 s once/session; breadcrumb `Systems ▸ <Lens> ▸ <Concept ▸ …> ▸ <File>`; gear hover = 2 px lift + `--cyan-glow` + tooltip (members, load, leak/relation counts, note dot); restore-tab convention for return paths, **no toolbar toggle buttons** (saved contract); leaks amber, boundary cycles red-glow; no sound/mascot/confetti.

## Storage formats (precise)

Git-tracked at the analyzed repo root (never under `artifacts/`, the fingerprint-invalidated cache):

```
architecture/
  concepts/<slug>.md          concepts/relations (one file per concept)
  symbols/<sanitized-id>.md   symbol-attached notes
  processes/<slug>.md         named traces
  views/<slug>.md             user-defined lenses
```

Concept frontmatter (TOML in `+++` fences; body = markdown = concept doc):

```toml
+++
id = "gateway"                # stable, default = slug; never rewritten by rename
name = "API Gateway"
parent = "edge-tier"          # optional → hierarchy (subsystems are parents)
files = ["services/gateway/**"]            # repo-relative paths/globs → FileIds
symbols = ["Gateway*"]                     # SymbolId-name globs (search/notes aid, not edge source)
[[relations]]
to = "engine"                 # unresolved → visible 'dangling' warning
kind = "feeds"                # registry seed or free text
detail = "pushes decoded jobs onto the queue"
evidence = "calls"            # optional: derived channel to reconcile against
+++
```

Process: `root`, `relation = calls|includes`, optional `depth`. View: `lens = structure|boundaries|dependency|flow|process`, `level = concept|node`, `edges = declared|derived|both`, `heat = fanin|fanout|closure|collateral|crossings|none`, `filter` globs/concept-ids. Symbol-note filenames: SymbolId with `[^\w.-]`→`_`, `--<crc32>` on clash. Semantics: concepts are **lenses, not partitions** (multi-membership allowed, unmapped files dimmed); deletion re-parents children and orphans others' relations with warnings; `parent` cycle → cycle badge, no crash.

## Relation-kind registry (server + UI single source)

- Derived channels (read-only from artifacts): `includes`, `uses`/`refers` (Complete/Incomplete), `calls`, `inherits` (extends_edges; empty for C++ — documented gap).
- Declared seeds (free text always allowed, autocomplete over in-use): `inherits, implements, part-of, composes, aggregates, associates` · `calls, drives, feeds, guards` · `owns, mirrors, replaces`.
- Reconciliation per declared relation with `evidence` bound: evidenced when ≥1 derived edge of that channel crosses resolve(A)→resolve(B) **in the declared direction** (undirected channels: `includes` counted either direction); outcomes: solid pipe (declared+evidenced), ghost belt (declared, no evidence), **leak** (crossing derived edge with no declared relation either way). Registry is a const table feeding composer chips + legend.

## Lenses (defaults + user views, one renderer path)

| Lens | Edges | Layout/motion |
|---|---|---|
| Structure | `parent` hierarchy + mapping coverage | parent gear centered, children on concentric rings (deterministic angles by slug order), recursion |
| Boundaries | derived cross-concept edges | gear graph; leaks amber, ghost belts dashed, cycle badge |
| Dependency | concept-aggregated + existing node tabs beneath | column layout (existing) |
| Flow | declared kinds + call evidence | directional dash-flow; pathfinder-aware |
| Process | BFS trace closure (S12) | glowing thread on both layers; gear lit when trace touches a member |

Include/Symbol/Call tabs stay as-is (Dependency lens at node level). New **Systems tab** hosts the lens picker (5 defaults + user views after a separator) and the concept↔node zoom continuum (lens config `level`). Save-current-view captures lens+params into `views/<slug>.md`.

## Server-side design

New module `crates/ui/src/architecture/` (server.rs = 792 lines — routes go here):

- `model.rs` / `loader.rs` — parse (per-file isolation: broken → `Broken` entry, rest load), glob→FileId re-derivation, relation resolution + reconciliation, `ArchitectureSnapshot`; re-run on boot, analysis swap, POST, watcher mtime.
- `registry.rs` — kind registry + free-text normalization (shared by composer, legend, validation).
- `render.rs` — markdown → HTML via pulldown-cmark, raw HTML **disabled** (XSS test required).
- `write.rs` — CRUD for all four file kinds; read-modify-write preserving bodies; atomic tmp+rename (copy `crates/scheduler/src/tu_store.rs`); strict slug/path validation (no `..`/absolute, confined); returns `prior` for undo (S7).
- `routes.rs` — sub-Router: `GET /api/architecture`; `GET /api/architecture/preview?glob=` (S8); `POST .../concept`, `/concept/<id>/membership|note|relation|delete`, `/process`, `/view`, `/symbol-note` (+ delete variants); `POST /api/architecture/undo` (S7). Mutations reload the `RwLock<ArchitectureSnapshot>`. Standalone export has no POST surface by construction.
- Snapshot embeds into `GraphExportPayload` with `#[serde(default)]` (pinned-Fixture regen commit; sorted-map serializer keeps byte stability).
- Suggestions: `crates/graph/src/community.rs` — deterministic label propagation over include adjacency (FileId-order seeds, bounded, stable ties, drop <3 files), name = dominant dir prefix, cap 10, `source` field for future LLM.
- `/api/graph/search` extension: `kind = concept|note|process|view` + snippet; identical behavior when `architecture/` absent.
- New deps: `toml`, `pulldown-cmark` (local pin, matches `regex = "1.10"` style).

## Frontend design (crates/ui/assets)

New files only (graph.js 990, styles.css over budget — grow neither; inspector extensions via S11):

- `systems.js` — retained-mode renderer (S1), camera service (S2), unfurl/thread/pulse (S6), lens picker + live view params (S8), relation composer + drag interactions (S9), optimistic mutations + undo chip (S7), process BFS overlay (S12), concept inspector sections.
- `systems.css` — engine-room styling/keyframes, tokens only (verified by `python tools/theme_token_check.py --check-added`), reduced-motion block.
- `main.js` / `graph.js` / `hotspots.js` — registration hooks only: Systems tab, zoom-state callback feeding S2, search corpus (`SYS/DOC/PROC/VIEW` badges into `compileMatcher` + quick-open).
- Register in `lib.rs` `include_str!`, asset routes, `index.html`; `#systemsRestoreTab` per convention.

## Tasks (ordered, RGB-TDD: failing test first for every Rust task)

1. **Model + loader** (ui): fixtures — concepts (tree, globs→FileIds, multi-membership, unresolved/dangling/broken isolation, parent cycle, rename keeps id, missing dir → empty), processes, views, symbol-note sanitize/collision, validation table exported for reuse (S10). `toml` dep.
2. **Registry + reconciliation** (ui): directed/undirected crossing rules per channel; solid/ghost/leak classification incl. C++-no-inherits; free-text normalization.
3. **Snapshot DTO + GET + preview + boot/watcher wiring + export embed**: contract tests (shape, 64 KiB note cap + `truncated`, XSS inert, glob preview), serde-default back-compat; pinned export fixture regen in its own commit.
4. **Community suggestions** (graph): cliques+bridge split, byte-deterministic, min-size, disconnected.
5. **Camera service + retained-mode renderer** (`systems.js`): S1–S6 mechanics (tween capture, hysteresis, readiness gating, hit-pads, keyboard map, budgets) + Structure and concept-Dependency lenses. **UX gate:** acceptance list items 1–2 on AT's own repo.
6. **Boundaries + Flow + Process lenses + lens picker + live view params** (S8 WYSIWYG save). **UX gate:** leak count in ≤2 actions; process thread visible on both layers.
7. **Wayfinding + concept inspector**: breadcrumb, Esc step-out, concept dive scoping (files + ghosted 1-hop externals), restore tab, gear↔file highlight correspondence (S3), hover affordances. **UX gate:** location assertable at every step; round-trip camera within budget.
8. **Search**: client corpus + badges + quick-open; route tests (concept/note/process hits, regression parity, absent-dir parity). **UX gate:** ≤2 actions; `SYS/DOC/PROC/VIEW` grouping.
9. **Write path + authoring flows**: tests — CRUD all four file kinds, body preservation, prior/undo byte-exact, traversal/slug rejection, preview endpoint; UI — optimistic mutations, undo chips, shift-select concept creation, file→gear drop membership (live `+N`), gear→gear relation composer with evidence preview, empty-state 3-step guide + "Generate starter from suggestions", note editor. **UX gate:** cold empty → subsystem+2 concepts+1 relation ≤8 clicks; mutation visible ≤100 ms; S9 keyboard path ≤5 keystrokes.
10. **Hygiene + validation sweep**: `cargo test`; `cargo clippy` warning-free; `hygiene_check.py`; `theme_token_check.py`; Systems-layer manual pass in all three themes (steampunk showcase, light-theme steam/glow contrast); UI copy notes `architecture/` is meant to be committed.

## Risks / edge cases

- Renamed files → visible unresolved-glob warnings, never silent loss; dangling relation targets and orphaned relations surfaced (S10).
- Export payload growth → 64 KiB/note cap + byte-stable fixture regen commit.
- Write races → read-modify-write + atomic rename; one-step in-process undo is best-effort (lost on server restart — UI says "Undo unavailable after restart"); deeper history = git.
- XSS → server-rendered, HTML disabled; UI inserts only renderer output.
- Mega-community repos → suggestion cap 10 + hand-authoring path.
- Two layers one viewportGroup → readiness gating + pointer-events by active mode; hysteresis.
- Optimistic-write divergence (watcher reload lands during a pending POST) → reload preserves the last mutation's `prior` and merges by file mtime; mutation response carries post-write mtime to reconcile.

## Out of scope (v1)

Extractor changes (C++ extends, field composition) — separate pipeline milestone; LLM assistance (`source` reserved); VS Code extension surfaces; freeform diagram canvas; runtime/OS process semantics; sound; multi-step undo history (git covers).

## Validation summary

`cargo test` green; `cargo clippy` warning-free; `hygiene_check.py` + `theme_token_check.py` pass; tests in `crates/ui/tests/architecture_*.rs`, `crates/graph/tests/community_proposal.rs`. Manual walkthrough on AT's own repo in all three themes executing every **UX gate** above; solid/ghost/leak verified against real include/call crossings; `<script>` note probe inert; `--export` standalone boots offline read-only with working process overlays; reduced-motion path fully usable.
