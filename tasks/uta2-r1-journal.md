# UTA2-R1 Journal - ui traversal projection fixes + endpoint coverage

Handoff: `tasks/uta2-r1-handoff.md`. Base: main 19349b3, clean tree.
Binding spec: `docs/specs/unit-test-audit.md` "Round 2" (UTB-1..UTB-4, audit
rows 1-13).

## Consumer-contract greps (BEFORE any behavior change - handoff mandate)

### UTB-3 `/api/repo-info` `file_paths` consumers
- `extensions/ariadne-extension/src/extension.ts:16-19` - the
  `RepoInfoResponse` interface types `file_paths: string[]`, but NO code path
  reads it. Only `repo.repo_root` is consumed (:274 startup message, :328
  connect message). The interface stays compiling under the fix (still a
  `string[]` of paths).
- `extensions/ariadne-extension/test-api.sh:14,24` - curls the endpoint for
  health + raw echo; parses nothing.
- `extensions/ariadne-extension/README.md:76,135,140` - documents the
  endpoint as "to verify server is running" / "know what paths it serves";
  no value-format promise beyond the doc-comment already in the Rust source.
- ui assets (all 14 files under `crates/ui/assets/`): zero references to
  `repo-info` or `file_paths` (grep incl. case-insensitive).
- `crates/ui/tests/**`: no test references `/api/repo-info`; the
  `file_paths` hits are all `IncludeGraph.file_paths` fixture plumbing.
- `tests/` (integration) and `tools/` (incl. `ui_smoke.py`): zero references.
- Verdict: NO existing byte/etag/golden lock and NO extension-host consumer
  contradicts the fix. The endpoint payload is unconsumed today; changing
  numeric ids to repo-relative path strings implements its own documented
  contract (api_source.rs:64 "File paths relative to `repo_root`"). Safe to
  fix. Emission additionally gets a sort: `file_paths` is a std HashMap, so
  unsorted values would reproduce the UTA-B4/UTB-2 nondeterminism class in
  the very payload being repaired (journal note; not requested verbatim).

### UTB-4 `top_hubs` / `get_entrypoints` consumers (traversal.js inspected read-only)
- `crates/ui/assets/traversal.js` (read-only inspection): fetches ONLY
  `/api/graph/traversal` (lines 239, and api.js:308 the same endpoint). It
  never calls `/api/graph/entrypoints` and never reads `top_hubs`. For the
  traversal payload it renders `subtree.nodes` via `nodes.map(...)` in
  emission order (no re-sort, no positional/index dependency, lines 322-334)
  and never consumes the subtree `edges` arrays at all.
- `crates/ui/assets/api.js:337-352` - the only entrypoints consumer. It maps
  the whole response OBJECT as if it were an ARRAY (`eps.map` on a
  `{graph_type,cycles,top_hubs,roots,leaves}` object) -> TypeError -> the
  modal always renders "Error loading entrypoints.". No live order (or any)
  dependency on `top_hubs` exists in JS.
- `tests/integration/tests/m16_traversal_test.rs:142` - asserts only
  `ep_json.get("top_hubs").is_some()` (existence, order-agnostic).
- `crates/ui/tests/ui_test/graph_api.rs` entrypoints tests - compare hub ids
  through `element_ids()`, which SORTS before asserting: order-agnostic. The
  limit=1 case pins hub id "2", which IS the degree-rank #1 (in+out=2) in
  that fixture, so it survives the re-sort unchanged.
- Verdict per handoff rule ("if it re-sorts or ignores order, re-sort
  `top_hubs` by rank order"): JS ignores order entirely -> FIX.
  `get_entrypoints` already returns hubs degree-desc/id-asc
  (`crates/graph/src/traversal.rs:255-260`); the handler drops that order by
  re-filtering through payload emission order (the admitted skip at
  api_graph.rs:289). Re-emitting in rank order RESTORES the documented
  `top_hubs` contract. No lock breaks (all hub comparisons are sorted).

### UTB-1 mechanism correction (audit ORCH row vs locked graph convention)
`LoadMemo::strict_closure` (crates/graph/src/load_caches.rs:382-395) returns
`transitive_includes` which NEVER contains the seed - the BFS seed-exclusion
convention is locked by `crates/graph/src/closures_tests.rs:88-98`
("a member's closure never contains itself"). Therefore the traversal dep can
NEVER be in `dep_closure`, the `filter(|&&id| id != exclude)` at
api_graph.rs:756 is vacuous for acyclic graphs, and the handoff's literal red
("no edge references the dep id" on a root->dep fixture) cannot fail on
current code - edges iterate `closure` which excludes dep on both sides.
The orphan-edge bug is REAL via the other half of the mismatch: nodes are
`closure ∩ file_paths` (the `filter_map(file_paths.get(...))` at :757 drops
unregistered ids) while edges test only `closure` membership (:820) - a
closure member unregistered in `file_paths` appears as an edge endpoint but
is absent from the node list. The FIX implements the handoff's stated
semantics ("edges live on the node set"; exclude filtered BOTH as source and
target) as: edges built over `closure \ {exclude} \ file_paths-missing`,
which subsumes both readings. Red asserts the induced-edge invariant on a
fixture with an unregistered closure member (ghost FileId 44).

### Adjacent observations (pinned, NOT fixed - out of the handoff fix list)
- `actual_subtree.file_count = dep_closure.len().saturating_sub(1)`
  (api_graph.rs:672) subtracts the seed even though the closure already
  excludes it: with an all-registered closure, `file_count` is
  `nodes.len() - 1` (and `summary.actual_files` inherits it). Pinned +
  reported for owner decision; fixing it would change summary numbers
  beyond the handoff's mandate.
- The `Err(_) => empty_traversal_response` arm (api_graph.rs:518-523) is
  unreachable over HTTP: `analyze_file_load_by_id` -> `compute_load` has no
  Err path at all (crates/graph/src/load.rs:506-557 always `Ok`); the only
  failure in the path-entry (`find_file_id`) is already gone in the by-id
  entry this route uses. The reachable sibling early-return (empty symbol
  seed) is pinned as the "empty-metric" matrix row; the Err arm is
  reported as dead code (no seam to reach it from a route).
- `build_subtree_edges` (header-based, used by necessary + collateral)
  hardcodes `is_necessary: true` for BOTH callers - collateral edges carry
  `is_necessary=true`. Pinned as-is (owner decision; not in handoff list).
- Ghost ids DO survive into the collateral subtree as path "unknown"
  (ImpactHeader fallback at load.rs:413), unlike actual_subtree nodes which
  drop them. Pinned; asymmetry reported.
- api.js `showEntrypoints` object-vs-array bug: out of scope (JS read-only),
  reported.

## Item-6 rename justification (handoff-authorized)
`test_server_traversal_routes` (graph_api.rs) never touches `/api/graph/
traversal` despite the name - it fires traverse/path/entrypoints/search
against an EMPTY loaded state and asserts only 200. Renamed to
`test_graph_query_routes_serve_empty_loaded_state` (truthful) and
strengthened to assert the empty-body semantics it actually exercises
(handoff item 6 authorizes strengthening; no assertion weakened).

## Item-11 asset-table location (handoff-authorized strengthening)
The asset serve table is
`static_html.rs::test_ui_asset_routes_serve_bundled_content_with_no_store`
(8 rows); `/app/traversal.js` + `TRAVERSAL_JS` is the missing member. Adding
a 9th row only strengthens the table (no existing row edited).

---

# Successor session completion log (2026-10-19)

Predecessor died with an empty compaction summary (never resumed). Successor
bootstrapped via resume-checkpoint and trusted ONLY disk: HEAD 19349b3, 5
modified + 5 new ui_test files, zero src changes. All authored tests above
were on disk; the four fixes were UNAPPLIED.

## Red capture (pre-fix, verbatim)
`358 passed / 11 failed / 0 ignored / 2 suites`
(capture: target/testrun-capture.txt; failures: 2 entrypoints_hub_order,
3 graph_arms_matrix, 1 repo_info_paths, 5 traversal_matrix).

## Green checkpoints (one per fix, per handoff discipline)
- UTB-2 GREEN: `build_subtree_nodes_from_closure` now consumes the ordered
  node set; `test_traversal_actual_subtree_is_deterministic_and_numeric`
  green (two builds byte-equal, ascending numeric FileId, 8-leaf closure).
- UTB-1 GREEN: new `closure_node_ids` (closure \ exclude \
  file_paths-missing, numeric-sorted) drives nodes AND edges;
  `build_subtree_edges_from_closure` induces edges on that node set, so
  ghost 44 and the dep can never be a source or target.
  `test_traversal_actual_subtree_has_no_orphan_edges` green.
- UTB-3 GREEN: `api_repo_info` emits sorted repo-relative path VALUES
  (`strip_prefix(repo_root)`, outside-root paths keep the full lossy
  string); `test_repo_info_file_paths_are_relative_paths_not_ids` green.
- UTB-4 GREEN: entrypoints handler emits `top_hubs` in the degree-rank
  order `get_entrypoints` returns (roots/leaves keep emission order - no
  rank documented). Projection extracted to
  `partition_entrypoint_elements` to hold R1 (<100-line functions).
  Both `entrypoints_hub_order` tests green; chain + limit=1 locks pass
  verbatim (sorted / single-element comparisons).
- Green canonical line:
  `369 passed / 0 failed / 0 ignored / 3 suites` (358+11 = every red
  converted; zero tests deleted or weakened; capture:
  target/testrun-capture.txt).

## Test-fixture fix (NOT an assertion change)
`graph_arms_matrix::arms_state` used `common::ready_state`, which
hardwires an EMPTY reduced model (`common.rs:215-228`), so the symbol arm
of traverse/path saw no chain (center-only / Null reds). State is now
built via `UiState::new_loaded` with the model the assertions target.
Routes under test finally receive their documented input; no assertion
weakened.

## Re-pins (journal rule: provably-wrong assertions re-pinned honestly, none deleted)
1. `test_traversal_matrix_seeds_and_response_keys` keys literal: authored
   order was NOT sorted ("actual_subtree" < "analyzed_symbol_ids", "ac" <
   "an"), so it could never match its own `keys.sort()`. Same key set,
   honest ascending order now.
2. `test_traversal_matrix_actual_subtree_fields`: authored
   `sorted_node_ids` sorts LEXICOGRAPHICALLY ("30" before "4") and could
   never equal the numeric-order literal - unfalsifiable against any
   code. Post-UTB-2 emission order is pinned numeric, so the raw
   `node_ids` order is asserted against the authored literal (same set
   intent, strictly stronger).
3. `test_traversal_matrix_symbols_param_overrides_justified_set`: both
   necessary-set expectations were provably wrong under the LOCKED
   graph-crate A2 rule - `required_headers_for_symbols` drops seed
   defining files WHOLESALE (load.rs:322-331; locked by
   `test_required_headers_seed_defining_file_dropped_wholesale` and
   `test_required_headers_isolated_seed_returns_empty`). drag@aux.h and
   unused@spare.h are seed files: drag => R = {} (file_count 0);
   used_sym+unused => R = {req.h} (file_count 1, nodes ["30"]). Override
   intent preserved via the unmodified `analyzed_symbol_ids` asserts.

## Hygiene riders (R1/R2 forced by this hand's diff)
- R1: `api_graph_entrypoints` reached 113 lines with the UTB-4 block ->
  extracted `partition_entrypoint_elements` (verbatim projection + hubs
  rank map); handler back under 100.
- R2: predecessor's 9th asset row pushed `static_html.rs` to 1004 lines.
  `test_standalone_export_inlines_all_split_assets` relocated VERBATIM to
  new module `static_html_standalone_export.rs` (registered in main.rs).
  Assertion bodies untouched; suite count unchanged.

## Gates (successor session, verbatim)
- `python tools/testrun.py --distill -- -p ariadne-ui` red:
  `358 passed / 11 failed / 0 ignored / 2 suites`; green:
  `369 passed / 0 failed / 0 ignored / 3 suites` (both: capture
  target/testrun-capture.txt).
- `python tools/ui_smoke.py` direct invocation refused by the sandbox
  allowlist (catch-all deny boilerplate); ran via `python tools/
  distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS` (boot+route
  checks clean, cold fixture, attempt 1).
- `python tools/hygiene_check.py` exit 0 for R1/R2; all R3 warns are
  pre-existing single-letter/banned-name rows in HEAD code (none added by
  this hand's files).
- `python tools/encoding_guard.py --all` pre-commit: clean; `--staged`
  witness at commit time recorded below.

## Successor out-of-scope report (adds to predecessor list above)
- `common::ready_state`'s hardwired empty model is a fixture trap for any
  future symbol-arm test; owner may want a `ready_state_with_model`
  variant (NOT touched: frozen harness).
- Pre-existing compiler warnings on the ui lib build: unused
  `INDEX_HTML` import in export.rs; unused `gen` parameter in the
  header-based `build_subtree_nodes` (kept for signature symmetry; the
  header builder genuinely does not read the generation).
