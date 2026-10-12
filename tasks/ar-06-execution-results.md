# AR-06 Execution Results - reviewer gate run (shared with AR-07)

Reviewer-fresh gates, 2026-10-10, own stdout (capture-race ruling: canonical
lines quoted from my run output, not the shared capture file). HEAD `70e1699`.
Same session ran the AR-07 gates - see `tasks/ar-07-execution-results.md`.

## Acceptance criteria vs evidence (verbatim lines)

1. Scoped graph suite - criterion: green, expected 116/0/0/5.
   `python tools/testrun.py --distill -- -p ariadne-graph`
   `### cargo test PASS (exit 0)` ...
   `116 passed / 0 failed / 0 ignored / 5 suites` MET.
   Includes the 7 AR-06 parity tests (bytes = `8ea8d0d` blob = working tree).
2. Mandated bench gate - criterion: no regression past 1.25x.
   `python tools/distill.py -- python tools/bench_compare.py`
   `### generic PASS (exit 0)` ...
   `bench compare: 3 run(s), no regressions past 1.25x baseline` MET.
   Truth rows (my run): `get_or_build_include_graph clone (cold) | 1.6`,
   `(warm) | 1.2`, `ensure_load total | 4617.1`,
   `build_file_justification_with_memo | 8528.6`,
   `OptimizationPlanner::plan() facts+closures | 1749.6`,
   `symbol_pass+prep (combined, <= first event) | 1561.6`.
   Clone rows vs 1.3/1.0 baseline = 1.23x/1.20x (pass; cold near-ratio).
   Implementer's direct `bench_compare.py` run was permission-refused
   (boilerplate catch-all deny); I hit the same refusal pattern on a chained
   `git log` command once and on a piped `unittest` invocation, and every
   allowlisted gate ran normally - the wrapper route is valid evidence.
3. Byte-order lock (handoff invariant "output edge order byte-identical"):
   `edges_vec_is_byte_identical_to_reference_interleave_fixture` asserts
   serde-bytes equality; fresh suite green. MET.
4. m00 parity / graph-bytes consumers UNEDITED + green (shared with AR-07):
   `m00_facts_path_parity_test`: `2 passed / 0 failed / 0 ignored / 1 suites`;
   `m00_golden_baseline_test`: `2 passed / 0 failed / 0 ignored / 1 suites`.
   Both files untouched in the wave range (`git log`: last edit `f9064f4`,
   pre-wave). MET.
5. Encoding: `python tools/encoding_guard.py --all` - silent, exit 0. MET.
6. Hygiene: `python tools/hygiene_check.py` output shows exactly the 2
   pre-existing blocking rows (`crates/ui/src/server.rs: R2 1155 lines`,
   `R1 api_graph_traversal 161 lines`); no rows for `graph.rs` or the parity
   test; all other rows R3 warn-only (warn-only by ledger policy). MET.
7. Workspace census (shared gate, attributable):
   `python tools/testrun.py --distill` (bare = `--workspace`):
   `1326 passed / 0 failed / 13 ignored / 94 suites` exit 0.
   +4 vs post-AR-02 census `1322/0/13/94` = foreign UNCOMMITTED AR-16 WIP
   (`crates/scheduler/src/worker.rs` +547, adds tests, green). No AR-06/AR-07
   attribution. MET.
8. Tools CI suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
   `Ran 219 tests in 19.110s` / `OK`. MET.
9. Byte-identity of the swept test file:
   `git diff 8ea8d0d..70e1699 -- crates/graph/tests/ar06_edge_dedup_test.rs`
   -> EMPTY. MET (no silent evolution).
10. Report evidence: journal digest-shaped, distilling entry points, capture
    paths cited, bench FAIL/PASS digest rows quoted. MET.

## Not run / not possible (explicit)

- `python tools/testrun.py --baseline` bare form: tool contract requires a
  FILE (`usage error: missing value for --baseline`), and no repo baseline
  file exists (known environment finding from the AR-03 verdict). Bare
  workspace census run instead (item 7).
- `python tools/ui_smoke.py`: not run - UI runtime is untouched by these
  graph-only hands and the milestone did not require it for them (AR-02's
  APPROVED review smoke-tested the current UI commit).
- Raw exit codes for hygiene/encoding could not be echoed (chained `echo`/
  `$LASTEXITCODE` off the allowlist, per AR-02 finding 3 precedent); the
  printed outputs above are complete and were adjudicated instead.
- `cargo check --workspace --all-targets`: not re-run - the fresh full
  workspace compile-and-test (item 7) subsumes compile-level fallout.
