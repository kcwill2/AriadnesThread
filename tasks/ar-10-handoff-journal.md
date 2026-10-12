# AR-10 Journal - Search path: hoist wildcard regex per request (P7 MED)

Owner: implementer (AR-10). Base: 70e1699 (chain: AR-01 6d7f96b and AR-02
b49ed06 both landed and approved; this branch additionally carries AR-04/05/
06/07/08-era commits on top, all pre-existing and untouched here).

## What was done

- New test mod `crates/ui/tests/ui_test/search_characterization.rs`
  (11 tests) pinning the FULL `/api/graph/search` semantics against a
  populated three-graph payload fixture (mixed-case labels/paths, a
  literal `*` in a label, a literal-regex precedence bait node, slash
  probes, a label-less/kind-less node, an edge element, 25 filler nodes
  for ordering/truncation).
- 1-line registration in `crates/ui/tests/ui_test/main.rs` (server_api.rs is
  at the 1000-line ceiling; same new-mod precedent as AR-01/AR-02).
- Refactor of the search region in `crates/ui/src/server.rs`: the wildcard
  glob is compiled ONCE per request via a new small helper
  `build_wildcard_glob` (placed directly above `api_graph_search`), using
  the case-insensitive flag `(?i)` so the per-element `to_lowercase()`
  of label AND path disappears from the wildcard path.
  Per-element cost went from up to N regex compiles + 2N lowercase allocs
  to one compile + zero allocs on that path.

## TDD record (digests verbatim, from own stdout)

- Baseline package pre-touch:
  `245 passed / 0 failed / 0 ignored / 3 suites`
  (capture: target/testrun-capture.txt)
- Characterization module pre-change (behavior lock): first run
  `10 passed / 1 failed` - the failure was MY expectation, not a handler
  defect: the empty query also matches the label-less node 9, so the
  default-limit slot 20 is `f10`, not `f11`. Lock corrected to the observed
  current behavior (this is what characterization means), then
  `11 passed / 0 failed / 0 ignored / 1 suites` pre-change: GREEN, current
  behavior fully pinned.
- Post-refactor scoped: `11 passed / 0 failed / 0 ignored / 1 suites`
  - every lock still green, byte-identical results.
- Gate 1 full package: `256 passed / 0 failed / 0 ignored / 3 suites`
  (= 245 baseline + 11 new locks; existing graph_api.rs search tests
  q=*test*, /^test/, include/ also still green.)
- Gate 2: `python tools/distill.py -- python tools/ui_smoke.py` ->
  `ui_smoke: PASS` (boot+route checks clean, capture printed).
- Gate 3: `python tools/encoding_guard.py --all` -> clean (no output).
- Gate 4: `python tools/hygiene_check.py` -> server.rs carries exactly the
  2 AR-13-owned pre-existing blocking rows: `R2 1172 lines > 1000` and
  `R1 fn api_graph_traversal is 161 lines`. NEW server.rs line count: 1172
  (was 1155 at session start; +17 net before AR-13 splits the file).
  `api_graph_search` itself stayed at its original ~94 lines (under the
  100-line R1 ceiling) thanks to the helper extraction.

## Characterization-lock list

1. test_substring_is_case_insensitive_over_label_and_path - substring
   contains over lowercased label OR path; include->symbol->call order;
   edge labels never scanned (q=renderedge, q=e1 -> empty).
2. test_substring_dot_and_slash_are_literal - no-wildcard queries keep
   `.` and `/` literal (q=widget.cc, q=a/, q=/a).
3. test_wildcard_glob_is_anchored_and_case_insensitive - `*`/`?` produce
   ANCHORED case-insensitive globs (q=*.cc, widget?c?, RENDER_*? *engine,
   WIDGET*).
4. test_wildcard_query_escapes_literal_metacharacters_in_target - query
   metachars escaped; a literal `*` stored in a label is an ordinary char
   (q=xa*by -> node 5 only; q=widget.cc.bak substring -> node 4).
5. test_wildcard_branch_suppresses_substring_fallback - a compiling glob
   never falls through to substring contains (q=a*b -> empty although
   "xa*by" contains "a*b" as text).
6. test_regex_literal_is_case_sensitive_and_unanchored - `/regex/` matches
   RAW label/path, case-sensitive, unanchored (/Renderer/ vs /renderer/,
   /src/call/ vs /SRC/CALL/, /Main.cc/ vs /main.cc/, /[.]cc$/).
7. test_invalid_regex_falls_back_to_wildcard_glob - "/*a*/" fails as a
   regex and engages the glob instead (escape -> `^/.*a.*/$`).
8. test_slash_boundary_locks - "/" alone is NOT a regex (len>1 guard) ->
   plain substring; "//" IS the empty regex matching every node.
9. test_graph_type_selects_scanned_graphs - include/symbol/call/unknown.
10. test_limit_truncates_in_graph_order - limit=2, limit=0, default 20
    (slot 20 = f10), wildcard + limit interaction.
11. test_missing_label_node_matches_by_path_with_empty_fields - the
    unwrap_or_default fallbacks yield empty label/kind strings.

## Deliberately NOT done

- Plain-substring path (no wildcards, no /regex/) left per-element
  lowercase-contains: the handoff scopes the hoist to the wildcard case
  ("and the one /regex/ literal path as today"). Optimizing it would widen
  semantics drift surface beyond the lock intent.
- `/regex/` path untouched (still case-sensitive, raw targets, same
  compile-once-per-request it already had).
- No result ordering, limit, or graph-selection changes.
- No bench_compare: explicitly out of contract (not a pipeline path). No new
  timing test in the suite either - a timing assertion would be a flaky
  lock. Micro-bench note (analytic, cheap form): a wildcard keystroke over
  N elements used to recompile the identical regex N times and allocate 2N
  lowercase strings; it now compiles once and allocates nothing per
  element. The regex crate compile is the dominant per-call cost, so the
  saving scales linearly with N at the handler's own scan cost floor.
- One documented, untested semantic edge of the (?i) swap: the regex
  crate's case folding is SIMPLE (per-code-point) while `str::to_lowercase`
  is full lowercase mapping, so exotic Unicode full-fold pairs (e.g.
  U+1E9E/german-sharp variants, U+0130) could in principle answer a
  wildcard query differently than the old double-lowercase path. All
  graph labels/paths are ASCII in practice and every ASCII outcome is
  identical (locked); this is the design the handoff directs and is
  recorded here as the only known divergence class.

## Deviations

- The wildcard hoist was extracted into `build_wildcard_glob` instead of
  staying inline: inline the region pushed `api_graph_search` (~94 lines,
  ceiling 100) over the R1 ceiling, which would have ADDED a third
  blocking hygiene row. Extraction keeps the handler at its original
  length. Net produced diff still ~20 lines in server.rs (1155 -> 1172).
- Characterization expectation fix (f11 -> f10) documented above; it
  corrected my prediction to observed pre-change behavior, never the src.

## Foreign state in the shared tree (not staged, not mine)

- Concurrent tests/integration agent: `tests/integration/tests/
  m15_graph_ui_test.rs` grew a NEW `R1 fn ... 104 lines` hygiene row and
  m10 gained R3 warns mid-session; `tests/integration/*`,
  `crates/pipeline/*` and a fixture report.json are dirty but belong to
  that lane. My commit is scoped to server.rs, the ui_test mod dir, and
  this journal only.
- Pre-existing, NOT in the AR-10 gate list: `cargo clippy -p ariadne-ui
  --all-targets -D warnings` fails in `crates/build/src/discovery.rs:411`
  (`type_complexity`, out-of-scope crate, present independent of this
  change). Recorded, not fixed.

## Gates (final, from own stdout)

1. `python tools/testrun.py --distill -- -p ariadne-ui` ->
   `256 passed / 0 failed / 0 ignored / 3 suites`
2. `python tools/distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS`
3. `python tools/encoding_guard.py --all` -> clean
4. `python tools/hygiene_check.py` -> server.rs: exactly the 2 AR-13-owned
   rows (R2 1172 lines; R1 api_graph_traversal 161 lines); zero new
   violations from this change.
