# Directory Quick-Open Navigation (search-integrated)

## Goal
Let users find files by directory structure in the graph UI, integrated into the existing search box as a quick-open dropdown, with a persistent directory scope chip that constrains graph seeds.

## Resolved decisions
- **Surface**: Dropdown panel anchored under the existing search input (VS Code Ctrl+P style). No sidebar.
- **Directory click**: Applies a **persistent scope** shown as a removable chip in the search bar; later searches only match files inside it.
- **Tab coverage**: Include Graph tab only. Symbol/Call node `path` fields are numeric file ids (`crates/ui/src/dto.rs:171`, `:214`), so the tree/dropdown/chip must hide on those tabs. No backend changes.
- **Scope vs BFS**: Scope restricts **seeds only**; degree expansion may traverse outside the directory.
- Frontend-only: all changes in `crates/ui/assets/index.html` (single embedded asset served via `INDEX_HTML`).

## Current behavior (verify in source before editing)
- Nodes: `tabAllNodes[] = {id,label,path,kind,...}`, paths are repo-root-relative with `/` (e.g. `src/thing.cpp`, `include/heavy.h`); built in `parseAndRenderElements` (index.html:1349).
- `compileMatcher` (index.html:2068) supports `path:`/`kind:` prefixes, `/regex/`, globs; `matchesFilter` (index.html:2096).
- `filterAndExpandSubgraph` (index.html:2105) picks seeds: selected node > search matches > lowest-in-degree nodes, then BFS with degree/direction/nodeCap.
- `handleSearch` (index.html:1935) clears selection on keystroke; `clearSearch` (index.html:2248); Ctrl+K and `/` focus search (index.html:2000).

## Design

### New state (near line 1175)
```js
let dirScope = null;            // e.g. "include/" — always ends with '/'
let qoOpen = false;             // quick-open dropdown visibility
let qoExpanded = new Set();     // expanded dir prefixes
let qoActiveRow = -1;           // keyboard cursor
```

### Tree build
- `buildDirTree(nodes)`: split `node.path` on `/`; nested Map `{dirs: Map<name, subtree>, files: []}`, sorted dirs-then-files alphabetically. Nodes without a real path (path === label) land at root level.
- Rebuild lazily per render from `tabAllNodes` (small repos; no caching needed beyond one rebuild per keystroke).

### Dropdown (`#quickOpenPanel`, absolutely positioned under `.search-wrap`)
- Opens when the search input is focused or typed in, **only when `currentTab === 'include'`**. Closes on outside click, Escape, or row activation.
- **Empty query**: browsable tree; top-level dirs expanded one level, nested collapsed; caret toggles rows.
- **With query**: rows = files passing `matchesFilter(n, compileMatcher(q))` **and** inside `dirScope` (if set); ancestor dirs of matches auto-expanded; each dir row shows match count.
- Row cap ~200 with a trailing "… N more files" row.
- File row click → `selectNode(id)` (existing focal-root behavior), close panel.
- Dir row click (not on caret) → `setDirScope(prefix)`, close panel.

### Dir scope chip
- Chip rendered inside `.search-wrap` before the input (or just left of it): `📁 include/ ✕`. Shown whenever `dirScope` is set.
- `setDirScope(d)` / `clearDirScope()` update chip + call `filterAndExpandSubgraph()`.
- Chip ✕ clears scope. Escape with dropdown closed and scope set → clear scope.

### Integration with existing filter pipeline (minimal edits)
1. `matchesFilter` (index.html:2096): first line — `if (dirScope && !((node.path||'').startsWith(dirScope))) return false;` (scope ANDs with search; `path:` matcher still works inside scope).
2. `filterAndExpandSubgraph` (index.html:2105): when no `selectedNodeId` and no query `q` but `dirScope` set → seeds = all files under `dirScope` (subject to existing nodeCap/capping). When a selected node exists, keep current focal-root precedence (chip still dims non-scope matches via matchesFilter for the match badge).
3. `parseAndRenderElements` (index.html:1349, tab switch): reset `dirScope = null`, close dropdown (scope is an Include-tab concept).
4. Keyboard (extend handler at index.html:2000 + search input `keydown`): ↓ opens/enters dropdown; ↑/↓ move `qoActiveRow`; Enter activates row; ← collapses / → expands a dir row; Esc: close panel first, then fall through to existing behavior. Ctrl+K `/` behavior unchanged.
5. Help text: add one line to shortcuts table (index.html:1136) and update tourSteps "Search & Tools" entry (index.html:2262) to mention the directory navigator.

### Edge cases
- Absolute/foreign paths (canonicalized system headers): split works the same; they simply group under their own prefix segments.
- Query with `path:`/regex + active scope: both must pass (AND semantics).
- Scope dir emptied by tab reload: `parseAndRenderElements` reset covers it.
- Sibling-name matching: always compare with trailing `/` prefix (`startsWith(dirScope)`), never raw substring, so `src/` doesn't match `src_extra/`.

## Tasks (ordered, TDD per .agents/rules/tdd-protocol.md)

1. **RED** — Add `tests/integration/tests/m17_dir_nav_test.rs` modeled on `m15_graph_ui_test.rs`: run `ariadne_cli::run_ui(..., export)` on `tests/fixtures/basic`, assert exported HTML contains the feature markers: `quickOpenPanel`, `dirScope`, `buildDirTree`, `setDirScope`. (JS runtime behavior isn't testable here — string assertions are this repo's precedent, see m15 `content.contains(...)`.) Expect RED.
2. **GREEN** — Implement in `crates/ui/assets/index.html`: CSS block (reuse `--panel`/`--cyan`/`--text-muted` theme vars), dropdown markup after `.search-wrap` (index.html:840-851), JS above, and the 5 integration edits. Keep total JS addition < ~250 lines; functions < 100 lines each (`buildDirTree`, `renderQuickOpen`, `setDirScope`, `qoKeyDown`).
3. **GREEN/verify** — `cargo test -p integration_tests --test m17_dir_nav_test m15_graph_ui_test` (adjust package name to actual), then `cargo test` for regressions; `cargo build`.
4. **Manual validation** — serve UI against `tests/fixtures/basic`:
   - Tree shows `include/`, `src/`, `tests/` dirs with the 5 fixture files.
   - Click `include/` → chip appears, graph seeds = 3 headers, degree expansion still pulls in cross-dir consumers.
   - Type `*heavy*` with scope active → 1 match (`include/heavy.h`); clear chip → matches unchanged for a path with no scope conflict.
   - Click file row → node selected as focal root; tab to Symbol → chip gone, dropdown disabled; back to Include → dropdown works again.

## Risks / notes
- `index.html` is already 2440 lines (> 1500-line guideline in AGENTS.md). Splitting into a separate asset would need a new embed + route in `crates/ui`; out of scope — keep the addition compact and note the debt.
- `handleSearch` clears `selectedNodeId` on keystroke; dropdown file-click calls `selectNode` directly so the focal-root flow is unaffected.
- Exported standalone HTML (`/api/export`) embeds the same page, so the feature ships there for free — verify no server-only assumptions.
