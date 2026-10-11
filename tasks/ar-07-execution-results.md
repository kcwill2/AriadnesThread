# AR-07 Execution Results - reviewer gate run (shared with AR-06)

Reviewer-fresh gates, 2026-10-10, own stdout (capture-race ruling applies).
Commits `8ea8d0d` + `602c176`, HEAD `70e1699`. Same reviewer session as
`tasks/ar-06-execution-results.md` - shared lines quoted in both.

## Acceptance criteria vs evidence (verbatim lines)

1. Scoped graph suite - criterion: green, expected 116/0/0/5 (includes the 5
   new inline `symbol_load_by_id` locks + the 7 AR-06 suite tests).
   `python tools/testrun.py --distill -- -p ariadne-graph`
   `### cargo test PASS (exit 0)` ...
   `116 passed / 0 failed / 0 ignored / 5 suites` MET - matches the
   implementer's GREEN line exactly.
2. The 5 locks verified genuine by source reading (not just by count):
   metric parity by-id vs name entry; helper dual-match on fixture
   `ns::Thing` (id) / `Thing` (record name) - id != name, so the two match
   arms are genuinely exercised - and miss -> `None`; the three error strings
   `"Symbol not found: Absent"`, `"Symbol record not found for: Absent"`,
   `"No defining file for symbol"` each asserted, with name/by-id agreement
   on the missing-defining-file miss. MET.
3. Additive purity: name entry delegates; body single-sited; `find_symbol_id`
   alias kept for the out-of-touch-set caller confirmed at
   `crates/graph/src/load_explain.rs:146`. MET (load.rs diff review).
4. Map-redundancy claim verified at source:
   `crates/graph/src/reduced.rs:86` `pub symbols: BTreeMap<SymbolId,
   ReducedSymbolRecord>` - side map would be a redundant second
   representation. MET (deviation accepted; journal mis-citation of a
   "stated fallback" recorded as verdict finding 2).
5. Encoding: `python tools/encoding_guard.py --all` - silent, exit 0. MET.
6. Hygiene: `python tools/hygiene_check.py` - exactly the 2 pre-existing
   blocking `server.rs` rows (R2 1155 lines; R1 `api_graph_traversal`
   161 lines); zero rows on `load.rs`; rest warn-only R3. MET.
7. m00 graph-bytes consumers UNEDITED + green (shared gate):
   `m00_facts_path_parity_test` -> `2 passed / 0 failed / 0 ignored / 1 suites`;
   `m00_golden_baseline_test` -> `2 passed / 0 failed / 0 ignored / 1 suites`;
   last edit of both files pre-wave (`f9064f4`). MET.
8. Workspace census: `python tools/testrun.py --distill` ->
   `1326 passed / 0 failed / 13 ignored / 94 suites` exit 0 (+4 vs post-AR-02
   `1322/0/13/94` = foreign uncommitted AR-16 `worker.rs` WIP, green; no
   AR-06/AR-07 attribution). MET.
9. Tools CI suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
   `Ran 219 tests in 19.110s` / `OK`. MET.
10. Scope-leak containment: swept `ar06_edge_dedup_test.rs` byte-identical
    `8ea8d0d..70e1699` (empty `git diff` on the path) and green in item 1's
    run; disclosure trail (journal + `602c176` + board INFO/RESULT) landed
    before first review. MET as disclosed deviation (verdict finding 1).
11. Report evidence: journal digest-shaped with RED DRIFT line + capture path,
    GREEN canonical line, provenance statement per capture-race ruling,
    distilling entry points throughout. MET.

## Not run / not possible (explicit)

- `python tools/testrun.py --baseline` bare form: requires a FILE (tool
  contract; `usage error: missing value for --baseline`), no repo baseline
  file exists (AR-03-documented environment state). Bare workspace census run
  instead (item 8).
- `bench_compare`: not in AR-07's handoff gates (handoff claims no runtime
  change); nevertheless run fresh in the shared session and PASSED -
  `bench compare: 3 run(s), no regressions past 1.25x baseline` - confirming
  the additive seam costs nothing.
- `python tools/ui_smoke.py`: not run - UI runtime untouched by these hands;
  not milestone-required for them.
- Raw exit codes for hygiene/encoding uncapturable (chained-echo off
  allowlist, AR-02 finding 3 precedent); printed outputs complete and
  adjudicated.
- One reviewer command (`git log` with `;` chaining + `echo`) hit the
  boilerplate catch-all deny once; retried as separate calls and the git MCP
  tool, all succeeded - no gate skipped.
