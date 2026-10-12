# AR-11 Handoff Journal - Escape-every-interpolation + data-id delegation (S2 HIGH / S4 MED)

Lane: sole UI-lane owner. Branch `shining-enthusiasm`, launch base `73f2160`,
finish base HEAD `0d7ecd3` (AR-19/AR-22/AR-01..10 landed between; `crates/ui`
sources confirmed byte-identical across that span via `git diff --stat
73f2160 0d7ecd3 -- crates/ui` -> empty, so every content anchor held).

## TDD red -> green record (canonical digest lines, verbatim)

Baseline before changes (full package, launch base):

    256 passed / 0 failed / 0 ignored / 3 suites

Phase RED (oracle module, scoped): the first scoped run reported
`1 passed / 6 failed / 0 ignored / 1 suites` - but see the syntax-bug
reconciliation below: only 7 of the 9 oracle tests were registered at that
point, and every registered failure was the expected semantic assertion.

Resume reconciliation: a provider stall split this handoff across sessions.
At resume, HEAD had moved to `0d7ecd3`; a full-package probe showed
`257 passed / 6 failed / 0 ignored / 2 suites` = baseline 256 + the one
oracle test already passing, with the suite-count drop an artifact of the
distiller parser on runs containing failures (final clean run shows 3 suites
again). The red set matched the oracle suite exactly; no asset drift.

Oracle-file syntax repair (NOT an assertion amendment): five raw strings
opened `r##"` were closed `"#` instead of `"##` (lines 211/213/239/243/315).
Rust treats `"#` as content, so each string swallowed source until the next
`"##`, silently nesting `test_apijs_declared_sites_escaped_and_delegated` and
`test_traversaljs_declared_option_sites_escaped_without_degree_regression`
inside other items - they compiled but never registered (the "phantom test"
mystery). Repair = closing delimiters only; zero assertion text changed.
After the repair `--list` shows all 9 oracle tests registered. This deviation
is declared here per the "oracle is the lock" rule: the assets were bent to
the oracle, never the reverse.

Interleaved red->green (module-scoped, one asset per cycle):

    2 passed / 7 failed / 0 ignored / 1 suites   (after hotspots.js remainder)
    6 passed / 3 failed / 0 ignored / 1 suites   (after traversal/api/graph)
    9 passed / 0 failed / 0 ignored / 1 suites   (after main.js wiring + file-view dedup)

Final gates (each run AFTER every edit, own stdout):

    python tools/testrun.py --distill -- -p ariadne-ui
      -> 265 passed / 0 failed / 0 ignored / 3 suites      (= 256 + 9)
    python tools/distill.py -- python tools/ui_smoke.py
      -> ui_smoke: PASS  (boot+route checks clean, attempt 1, 0.6s)
    python tools/encoding_guard.py --all
      -> silent (exit 0)
    python tools/hygiene_check.py
      -> blocking rows exactly the two AR-13-owned server.rs rows:
         `crates/ui/src/server.rs: R2 1172 lines > 1000` and
         `crates/ui/src/server.rs:816: R1 fn api_graph_traversal is 161 lines`
         Zero new R1/R2; every R3 warn is pre-existing and none in this
         handoff's files.

Capture paths per the distillation contract: `target/testrun-capture.txt` and
`target/distill-capture.txt` (shared worktree: another lane's runs can
overwrite captures between invocation and inspection; canonical lines above
are quoted from this lane's own command stdout, never re-read from capture).

## Asset-by-asset sweep table

hotspots.js (+42/-9):
- `escapeHtml` QUOTE-EXTENDED: now escapes & < > " ' (S4 contract; single
  shared definition across all shipped JS).
- hotspot row markup: `onclick="selectHotspotRow('${e.node.id}')"` ->
  `data-select-id="${escapeAttribute(e.node.id)}"`.
- traversal button: `onclick="event.stopPropagation();
  openTraversalForRoot('${e.node.id}')"` ->
  `data-traversal-root="${escapeAttribute(e.node.id)}"`.
- extraction card: `onclick="selectHotspotRow('${escapeHtml(...)}')"` ->
  `data-select-id="${escapeAttribute(String(p.target_file_id))}"`.
- NEW single document-level click listener (file end): traversal-button
  branch returns first (reproduces the inline stopPropagation suppression
  exactly), then row, then card - each calling the SAME function/argument
  the removed inline payload used.

graph.js (6/6 lines changed, line count 961 -> 961; budget cap 990):
- upstream row `onclick="selectNode('${e.source}')"` ->
  `data-select-id="${escapeAttribute(e.source)}"`; name via
  `<span>${escapeHtml(name)}</span>`.
- downstream row `onclick="selectNode('${e.target}')"` ->
  `data-select-id="${escapeAttribute(e.target)}"`; name escaped with the
  cyan span form.
- collateral row `onclick="illuminatePath('${id}', '${entry.id}')"
  title="... ${node.label} ... ${entry.label}"` ->
  `data-path-from/data-path-to` + title via `escapeAttribute` on both labels.
- collateral label `<span>${entry.label}</span>` -> `escapeHtml(entry.label)`.

api.js (3/3):
- path-node text `${n}` -> `${escapeHtml(n)}` (declared site :304).
- entrypoints row (declared :324): `onclick="selectNode('${ep}');
  closeModals();"` + raw `${ep}` ->
  `data-select-id="${escapeAttribute(ep)}"` + `${escapeHtml(ep)}`.
- justification preview button (S4-partial completion):
  `data-preview-root/source` from `escapeHtml(...)` ->
  `escapeAttribute(String(nodeId))` / `escapeAttribute(String(row.id))`.

traversal.js (2/2 lines ONLY - zero-diff constraint honored):
- :44 roots option: `value="${n.id}"` -> `escapeAttribute(n.id)`;
  `(${n.path})` -> `(${escapeHtml(n.path)})`.
- :131 hidden-deps option: same treatment for `d.id`/`d.path`.
- Untouched by demand, re-verified byte-present: the subtree row
  `data-node-id="${n.id}" onclick="selectNode('${n.id}')"` and the
  `dirSelect.value = 'out'` direction block (known-open
  `ui.traversal_view_degree_setting_bug`).

main.js (+26, 748 -> 774 lines):
- NEW `onListClick(containerId, rowSelector, handler)` helper + four wirings
  (exact strings locked by the oracle): inspUpstreamList /
  inspDownstreamList -> `selectNode(row.dataset.selectId)`;
  inspCollateralList -> `illuminatePath(row.dataset.pathFrom,
  row.dataset.pathTo)`; entrypointsList -> `selectNode` + `closeModals()`.
  main.js loads last, so containers and handler globals all exist at wire
  time.

file-view.js (+4/-4):
- Removed the file-local weaker `escapeHtml` duplicate (no quote escaping).
  Load order (index.html and export.rs identical: api, graph, traversal,
  file-view, hotspots, preview, provenance, main, theme) means a second
  global declaration would silently OVERRIDE the quote-safe shared one
  depending on order - the oracle's single-definition lock forbids it. All
  `escapeHtml` calls in file-view.js bind to the hotspots.js global at
  runtime (classic-script globals; file-view calls its escaper only from
  click/render handlers, never at load time).

## Sites reviewed and deliberately NOT changed

- `onclick="jumpToCrumb(${i})"` (main.js thread bar): argument is a numeric
  array index, never repo content. Pinned as the sole main.js survivor by
  the oracle.
- Extraction-card disclosure button `onclick="toggleExtractionCard(event,
  this)"`: no repo data; its `stopPropagation()` still suppresses the card
  select exactly as before (click never reaches the delegated listener).
- Preview buttons: preview.js `bindPreviewControls` owns their click +
  stopPropagation; unchanged (oracle asserts the stopPropagation ships).
- traversal.js :128 `data-value="${d.id}"` dropdown div: third traversal
  candidate declined - zero-diff constraint pins traversal.js to the two
  declared option lines; ids are server-minted numerics (`FileId.0.to_string()`,
  dto ids String-typed but numeric-valued), and the row already binds via
  addEventListener.
- graph.js `e.kind || 'Direct'` / cycle marker spans: server-enum values,
  no user content; oracle leaves them at zero-diff.
- provenance.js / preview.js interpolations: already escaper-bound or
  numeric-only.

## Delegation mapping (old inline -> new listener)

- hotspot row inline `selectHotspotRow('${e.node.id}')` -> document listener
  branch `.hotspot-row[data-select-id]` -> `selectHotspotRow(id)`.
- traversal-btn inline `event.stopPropagation();
  openTraversalForRoot('${e.node.id}')` -> FIRST document branch
  `.hotspot-traversal-btn[data-traversal-root]` ->
  `openTraversalForRoot(id)` + early return (suppression equivalent).
- extraction card inline `selectHotspotRow(escaped target_file_id)` ->
  document branch `.extraction-card[data-select-id]` ->
  `selectHotspotRow(id)`.
- upstream/downstream inline `selectNode('${e.source|e.target}')` ->
  `onListClick('inspUpstreamList'/'inspDownstreamList', ...)` in main.js ->
  `selectNode(id)`.
- collateral inline `illuminatePath('${id}', '${entry.id}')` ->
  `onListClick('inspCollateralList', ...)` ->
  `illuminatePath(pathFrom, pathTo)`.
- entrypoints inline `selectNode('${ep}'); closeModals();` ->
  `onListClick('entrypointsList', ...)` -> `selectNode(id); closeModals();`.
All six mappings ship as comments beside the code AND are string-locked by
`asset_injection_hardening.rs`.

## Double-escape analysis

The oracle locks zero composed escaper calls
(`test_no_nested_escaper_calls`). Audit of every site touched:
`escapeAttribute`/`escapeHtml` are applied once, directly to the raw repo
value (or `String()` of a numeric id, which is conversion not escaping).
No path escapes an already-escaped value: the old card payload
`selectHotspotRow('${escapeHtml(String(p.target_file_id))}')` (the one true
double-context site - escaper output re-parsed inside a JS string) is
REMOVED entirely by delegation, so escaper output is never re-parsed as
JavaScript anywhere. `escapeAttribute` remains the quoted-attribute escaper;
`escapeHtml` (now 5-char) the text escaper.

## Theme hex guard (declared, not mine)

`python tools/theme_token_check.py --check-added --base HEAD` reports three
frozen-COUNT violations: index.html 6>4, main.js 1>0, traversal.js 1>0.
Attribution probe (`target/ar11-hex-attribution.txt`): every flagged literal
is an HTML numeric entity (`&#128293;`, `&#128161;`, `&#8250;`, `&#9888;`,
`&#8592;`-family - the naive hex regex matches them) and the counts/positions
are BYTE-IDENTICAL between HEAD `0d7ecd3` and the worktree. This lane added
zero hex literals; the baseline (frozen at `d76a559`) predates earlier lanes
that shipped entities without `--update-baseline`. Baseline regeneration is
approval-gated (byte-locked manifest), so it was NOT touched here; flagged
to the orchestrator/reviewer as pre-existing drift needing its own
deliberate commit.

## Invariants honored

- graph.js line budget: 961 <= 990 (dedicated test in the 265/0 run green).
- Click-delegation behavior preserved per the mapping above.
- Extraction-card / traversal / hotspot story tests: unedited and green
  inside `265 passed / 0 failed / 0 ignored / 3 suites`.
- Pure-ASCII authored content; `encoding_guard --all` silent.
- Commit scoped via commit_scoped to crates/ui/assets/**,
  crates/ui/tests/ui_test/**, this journal only (shared worktree kept other
  lanes' dirty files unstaged).
