# UTA2-R1 Handoff - ui: traversal projection fixes + endpoint coverage

Binding inputs:
1. This file (complete contract).
2. docs/specs/unit-test-audit.md "Round 2" section (UTB-1/2/3 fixes, UTB-4 decision row, audit rows 1-13).
3. crates/ui/src/routes/api_graph.rs (traversal endpoint ~495-914, entrypoints 208-300, traverse/path arms 28-207), crates/ui/src/routes/api_source.rs, crates/ui/src/routes/api_payloads.rs:138.
4. Model: existing ui_test fixture style in tests/ui_test/ (loaded-generation fixtures, etag/golden patterns).

## Touch-set (hard)
- tasks/uta2-r1-journal.md (journal; always in-set)
- crates/ui/src/routes/api_graph.rs, api_source.rs, api_payloads.rs (+ minimal other src only if a seam demands it - journal first)
- crates/ui/tests/** (tests/ui_test/)
You MUST NOT touch: crates/graph (consumers of its payloads), assets JS (traversal.js behavior is the locked known-open degree bug - DO NOT touch JS), Cargo files. No stash/branch change.

## What to build
FIXES (red test first):
1. UTB-2 sort determinism: subtree nodes and edges arrays sorted by FileId (u64 numeric order) before emission; red: two builds of the same payload in-process are equal AND order is numeric, not HashSet-arbitrary.
2. UTB-1 orphan edges: induced-edge semantics - the excluded dep must be filtered from edges BOTH as source and as target (edges live on the node set); red: root->dep fixture where dep is excluded: no edge references the dep id.
3. UTB-3 repo-info paths: file_paths emits repo-relative path strings (values), not numeric FileIds; grep ALL consumers first (extension host under extensions/, ui assets, ui_test) and journal the contract; red: loaded state asserts a real path string.
4. UTB-4 hub order: inspect traversal.js consumption read-only; if it re-sorts or ignores order, re-sort top_hubs by rank order (fix, red test); if it depends on emission order, pin current + report loudly.

REQUIRED TESTS (pins):
5. Full traversal endpoint matrix: loaded fixture root->dep: every payload field of necessary/actual/collateral subtrees (total_symbols, file_count, decoys), 400 non-numeric root/dep, 404 unknown file, 400 dep-not-direct-include, empty-metric early return, symbols= override vs justified set (resolve_traversal_symbols), HEADER/SOURCE classification, file_symbols counts.
6. Rename/join the misnamed test_server_traversal_routes into the matrix (it asserts only 200-on-empty; strengthening it is authorized, journal-justify).
7. api_graph traverse symbol/call branches + direction param + filter_elements edge-id format verification against real payload ids.
8. api_graph_path symbol/call arms: exact id sequences on a symbol chain.
9. api_graph_symbol/api_graph_call loaded-state pass-through bodies.
10. current_export_snapshot refresh_key invalidation without epoch bump (mutate stage_status.last_refresh; assert re-serialize + counter bump).
11. asset_traversal_js HTTP serve test added to the asset table; is_same_origin https-arm + missing-Host arm; state.rs read_state/write_state poison arms (documented behavior pin only).

## RGB, hygiene, locks
Red-first per item; never weaken an assertion. <100-line functions, <1000-line files (ui.rs-style ceilings and the asset byte-lock family stay green), banned-name ban. Frozen locks: all existing ui_test byte/etag/golden locks pass verbatim; if the UTB-3 payload change flips an existing lock or the extension host, journal BEFORE changing it (consumer contract decision).

## Gates (timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-ui` (red then green; quote digest + canonical line)
2. `python tools/ui_smoke.py` (real boot assertion of changed endpoints)
3. `python tools/hygiene_check.py`; `python tools/encoding_guard.py --all` (pre-commit) and `--staged` (witness)
4. Commit: `python tools/commit_scoped.py --scope "crates/ui/**" --scope "tasks/uta2-r1-journal.md" -m "fix(ui): uta2-r1 traversal subtree determinism + induced edges + repo-info paths; test(ui): traversal endpoint matrix"` (adjust message to truth).

## Discipline
Write/Edit tools only, pure ASCII. No bare cargo - distilling entry points. 1-3 line journal checkpoint per Green gate. Out-of-scope: report. Context wall: STOP + disk-state inventory.

## Report back
Commit hash + git show --stat; red evidence per fix; green distill header + canonical line + capture path + ui_smoke line; deviations; out-of-scope observations. Never paste raw output.
