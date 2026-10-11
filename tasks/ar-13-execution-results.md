# AR-13 Execution Results - reviewer acceptance pass

HEAD at ground-truth: `768529f` (branch `shining-enthusiasm`), tracked
tree clean. All gate lines quoted verbatim from my own stdout.
`--baseline` note: testrun `--baseline` is FILE mode and no baseline file
exists in-repo (AR-23 finding), so the workspace census stands in per
that precedent.

## Contract gate commands (fresh, own stdout)

1. `python tools/testrun.py -p ariadne-ui`
   canonical line: `265 passed / 0 failed / 0 ignored / 3 suites`
   (256 baseline + AR-11's 9 oracle tests; count unchanged pre/post split
   and post decomposition; only warning: inherited `unused variable: gen`
   now at routes/api_graph.rs:690 - disclosed, pre-existing at
   server.rs:984).
2. `python tools/distill.py -- python tools/ui_smoke.py`
   digest: `### generic PASS (exit 0)` /
   `boot+route checks clean (cold fixture, attempt 1, 1.2s)` /
   `ui_smoke: PASS`
   (route-table authority; capture `target/distill-capture.txt`).
3. `python tools/hygiene_check.py` (direct run, full output) and
   `python tools/distill.py -- python tools/hygiene_check.py`
   distill: `### generic PASS (exit 0)`, capture 179 lines.
   FULL output (identical content both runs; EVERY line is R3 (warn),
   zero R1/R2 blocking rows, exit 0):

   ```
   crates/build/src/discovery.rs:320: R3 (warn) single-letter local `c`
   crates/cli/src/commands/ui.rs:543: R3 (warn) single-letter local `p`
   crates/cli/src/commands/ui.rs:665: R3 (warn) banned identifier `bar`
   crates/cli/src/commands/ui.rs:673: R3 (warn) single-letter local `p`
   crates/cli/src/commands/ui.rs:681: R3 (warn) banned identifier `bar`
   crates/cli/src/commands/ui.rs:800: R3 (warn) single-letter local `p`
   crates/cli/src/commands/ui/boot_progress_tests.rs:542: R3 (warn) single-letter local `p`
   crates/cli/src/commands/ui/builder_tests.rs:113: R3 (warn) single-letter local `p`
   crates/cli/src/commands/ui/builder_tests.rs:79: R3 (warn) single-letter local `p`
   crates/cli/src/lib.rs:363: R3 (warn) single-letter local `r`
   crates/cluster/src/plan.rs:74: R3 (warn) single-letter local `s`
   crates/graph/src/builder.rs:569: R3 (warn) banned identifier `tmp`
   crates/graph/src/builder.rs:570: R3 (warn) banned identifier `tmp`
   crates/graph/src/builder.rs:572: R3 (warn) banned identifier `tmp`
   crates/graph/src/builder.rs:653: R3 (warn) banned identifier `tmp`
   crates/graph/src/builder.rs:654: R3 (warn) banned identifier `tmp`
   crates/graph/src/builder.rs:657: R3 (warn) banned identifier `tmp`
   crates/graph/src/builder.rs:664: R3 (warn) banned identifier `tmp`
   crates/graph/src/closure_equivalence_tests.rs:179: R3 (warn) single-letter local `e`
   crates/graph/src/closure_equivalence_tests.rs:250: R3 (warn) single-letter local `a`
   crates/graph/src/closure_equivalence_tests.rs:430: R3 (warn) single-letter local `i`
   crates/graph/src/closure_equivalence_tests.rs:431: R3 (warn) single-letter local `j`
   crates/graph/src/closure_equivalence_tests.rs:439: R3 (warn) single-letter local `i`
   crates/graph/src/closure_equivalence_tests.rs:453: R3 (warn) single-letter local `k`
   crates/graph/src/closure_equivalence_tests.rs:461: R3 (warn) single-letter local `i`
   crates/graph/src/closure_equivalence_tests.rs:474: R3 (warn) single-letter local `i`
   crates/graph/src/closure_equivalence_tests.rs:475: R3 (warn) single-letter local `j`
   crates/graph/src/closure_equivalence_tests.rs:501: R3 (warn) single-letter local `k`
   crates/graph/src/load_caches.rs:428: R3 (warn) single-letter local `f`
   crates/graph/src/load_caches.rs:433: R3 (warn) single-letter local `f`
   crates/graph/src/load_justification_tests.rs:384: R3 (warn) single-letter local `m`
   crates/graph/src/load_justification_tests.rs:385: R3 (warn) single-letter local `z`
   crates/graph/src/load_justification_tests.rs:386: R3 (warn) single-letter local `b`
   crates/graph/src/load_justification_tests.rs:546: R3 (warn) single-letter local `y`
   crates/graph/src/load_justification_tests.rs:547: R3 (warn) single-letter local `z`
   crates/graph/src/load_justification_tests.rs:548: R3 (warn) single-letter local `w`
   crates/graph/src/load_justification_tests.rs:549: R3 (warn) single-letter local `v`
   crates/graph/src/reduced.rs:526: R3 (warn) single-letter local `x`
   crates/graph/src/reduced.rs:707: R3 (warn) single-letter local `a`
   crates/graph/src/reduced.rs:708: R3 (warn) single-letter local `b`
   crates/graph/src/reduced.rs:760: R3 (warn) single-letter local `s`
   crates/graph/src/traversal.rs:247: R3 (warn) single-letter local `n`
   crates/graph/src/traversal.rs:406: R3 (warn) single-letter local `n`
   crates/graph/src/traversal.rs:514: R3 (warn) single-letter local `n`
   crates/lang-api/src/catalog.rs:188: R3 (warn) single-letter local `c`
   crates/lang-cpp/src/analyzer.rs:24: R3 (warn) single-letter local `w`
   crates/lang-cpp/src/analyzer.rs:34: R3 (warn) single-letter local `w`
   crates/lang-cpp/src/parser.rs:113: R3 (warn) single-letter local `j`
   crates/lang-cpp/src/parser.rs:299: R3 (warn) single-letter local `j`
   crates/lang-cpp/src/parser.rs:315: R3 (warn) single-letter local `j`
   crates/lang-cpp/src/parser.rs:388: R3 (warn) single-letter local `k`
   crates/lang-cpp/src/symbol_locator.rs:142: R3 (warn) single-letter local `i`
   crates/lang-cpp/src/symbol_locator.rs:164: R3 (warn) single-letter local `i`
   crates/lang-cpp/src/symbol_locator.rs:216: R3 (warn) single-letter local `i`
   crates/lang-cpp/src/symbol_locator.rs:297: R3 (warn) single-letter local `i`
   crates/lang-cpp/src/tokenizer.rs:50: R3 (warn) single-letter local `i`
   crates/lang-cpp/src/tokenizer.rs:53: R3 (warn) single-letter local `c`
   crates/lang-scala/src/constraints.rs:266: R3 (warn) single-letter local `g`
   crates/lang-scala/src/constraints.rs:281: R3 (warn) single-letter local `g`
   crates/repo/src/lib.rs:92: R3 (warn) banned identifier `res`
   crates/repo/src/lib.rs:94: R3 (warn) banned identifier `res`
   crates/repo/src/lib.rs:96: R3 (warn) banned identifier `res`
   crates/scheduler/src/pool.rs:563: R3 (warn) banned identifier `res`
   crates/scheduler/src/pool.rs:564: R3 (warn) banned identifier `res`
   crates/transform/src/engine/mod.rs:135: R3 (warn) single-letter local `u`
   crates/transform/src/engine/mod.rs:152: R3 (warn) single-letter local `n`
   crates/transform/src/engine/mod.rs:170: R3 (warn) single-letter local `u`
   crates/transform/src/engine/tests.rs:131: R3 (warn) single-letter local `u`
   crates/transform/src/engine/tests.rs:198: R3 (warn) single-letter local `u`
   crates/ui/src/dto.rs:532: R3 (warn) single-letter local `s`
   crates/ui/src/dto.rs:559: R3 (warn) single-letter local `s`
   crates/ui/src/dto.rs:728: R3 (warn) single-letter local `w`
   crates/ui/src/routes/api_graph.rs:103: R3 (warn) single-letter local `g`
   crates/ui/src/routes/api_graph.rs:173: R3 (warn) single-letter local `g`
   crates/ui/src/routes/api_graph.rs:208: R3 (warn) single-letter local `r`
   crates/ui/src/routes/api_graph.rs:220: R3 (warn) single-letter local `r`
   crates/ui/src/routes/api_graph.rs:229: R3 (warn) single-letter local `g`
   crates/ui/src/routes/api_graph.rs:233: R3 (warn) single-letter local `r`
   crates/ui/src/routes/api_graph.rs:325: R3 (warn) single-letter local `q`
   crates/ui/src/routes/api_graph.rs:432: R3 (warn) single-letter local `d`
   crates/ui/src/routes/api_graph.rs:766: R3 (warn) single-letter local `h`
   crates/ui/tests/ui_test/graph_api.rs:123: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:124: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:125: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:228: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:229: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:231: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:52: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:53: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:59: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:60: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:66: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:67: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:73: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/graph_api.rs:74: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/justification_builders.rs:541: R3 (warn) single-letter local `i`
   crates/ui/tests/ui_test/justification_builders.rs:569: R3 (warn) single-letter local `i`
   crates/ui/tests/ui_test/justification_builders.rs:572: R3 (warn) single-letter local `i`
   crates/ui/tests/ui_test/justification_golden.rs:248: R3 (warn) single-letter local `j`
   crates/ui/tests/ui_test/justification_golden.rs:261: R3 (warn) single-letter local `m`
   crates/ui/tests/ui_test/server_api.rs:124: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:125: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:136: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:137: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:175: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:176: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:178: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:182: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:397: R3 (warn) single-letter local `i`
   crates/ui/tests/ui_test/server_api.rs:400: R3 (warn) single-letter local `i`
   crates/ui/tests/ui_test/server_api.rs:403: R3 (warn) single-letter local `i`
   crates/ui/tests/ui_test/server_api.rs:52: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:53: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:66: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:67: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:68: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:80: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/server_api.rs:81: R3 (warn) banned identifier `res`
   crates/ui/tests/ui_test/theme_contract.rs:132: R3 (warn) single-letter local `i`
   crates/ui/tests/ui_test/theme_js_zero_hex.rs:29: R3 (warn) single-letter local `i`
   crates/ui/tests/ui_test/theme_js_zero_hex.rs:32: R3 (warn) single-letter local `j`
   crates/validate/src/lib.rs:104: R3 (warn) single-letter local `p`
   crates/worker-core/src/tu_store.rs:54: R3 (warn) banned identifier `tmp`
   crates/worker-core/src/tu_store.rs:61: R3 (warn) banned identifier `tmp`
   crates/worker-core/src/tu_store.rs:62: R3 (warn) banned identifier `tmp`
   tasks/t0c-review-probe.py:24: R3 (warn) single-letter local `k`
   tasks/t0c-review-probe.py:25: R3 (warn) single-letter local `k`
   tasks/t0c-review-probe.py:26: R3 (warn) single-letter local `k`
   tasks/t0c-review-probe.py:68: R3 (warn) single-letter local `l`
   tasks/t0c-review-probe.py:72: R3 (warn) single-letter local `l`
   tasks/t0c-review-probe.py:76: R3 (warn) single-letter local `l`
   tasks/t4-review-probe.py:14: R3 (warn) single-letter local `k`
   tasks/t4-review-probe.py:39: R3 (warn) single-letter local `p`
   tasks/t4-review-probe.py:42: R3 (warn) single-letter local `p`
   tasks/t4-review-probe.py:50: R3 (warn) single-letter local `p`
   tasks/t4-review-probe.py:50: R3 (warn) single-letter local `v`
   tasks/t4-review-probe.py:51: R3 (warn) single-letter local `p`
   tasks/t4-review-probe.py:51: R3 (warn) single-letter local `v`
   tasks/t4-review-probe.py:70: R3 (warn) single-letter local `i`
   tasks/t4-review-probe.py:70: R3 (warn) single-letter local `v`
   tasks/t4-review-probe.py:71: R3 (warn) single-letter local `i`
   tasks/t4-review-probe.py:71: R3 (warn) single-letter local `v`
   tasks/t4-review-probe.py:74: R3 (warn) single-letter local `p`
   tasks/t4-review-probe.py:75: R3 (warn) single-letter local `p`
   tasks/v3-review-debug2.py:29: R3 (warn) single-letter local `i`
   tasks/v3-review-debug2.py:31: R3 (warn) single-letter local `i`
   tasks/v3-review-debug2.py:34: R3 (warn) single-letter local `i`
   tasks/v3-review-debug2.py:36: R3 (warn) single-letter local `i`
   tasks/v3-review-debug2.py:63: R3 (warn) single-letter local `p`
   tasks/v3-review-debug2.py:78: R3 (warn) single-letter local `x`
   tasks/v3-review-debug2.py:80: R3 (warn) single-letter local `x`
   tasks/v3-review-debug2.py:83: R3 (warn) single-letter local `x`
   tasks/v3-review-debug3.py:41: R3 (warn) single-letter local `p`
   tasks/v3-review-debug3.py:56: R3 (warn) single-letter local `k`
   tasks/v3-review-debug4.py:30: R3 (warn) single-letter local `k`
   tasks/v3-review-debug4.py:36: R3 (warn) single-letter local `k`
   tasks/v3-review-final.py:15: R3 (warn) single-letter local `a`
   tasks/v3-review-final.py:17: R3 (warn) single-letter local `x`
   tasks/v3-review-final.py:18: R3 (warn) single-letter local `b`
   tasks/v3-review-final.py:27: R3 (warn) single-letter local `x`
   tasks/v3-review-final.py:32: R3 (warn) single-letter local `x`
   tasks/v3-review-final.py:34: R3 (warn) single-letter local `x`
   tasks/v3-review-final.py:45: R3 (warn) single-letter local `x`
   tasks/v3-review-final.py:47: R3 (warn) single-letter local `x`
   tasks/v3-review-verbatim.py:28: R3 (warn) single-letter local `a`
   tasks/v3-review-verbatim.py:28: R3 (warn) single-letter local `b`
   tasks/v3-review-verbatim.py:48: R3 (warn) single-letter local `c`
   tasks/v3-review-verbatim.py:56: R3 (warn) single-letter local `c`
   tasks/v3-review-verbatim.py:61: R3 (warn) single-letter local `c`
   tasks/v3-review-verbatim.py:67: R3 (warn) single-letter local `i`
   tasks/v3-review-verbatim.py:78: R3 (warn) single-letter local `l`
   tasks/v3-review-verbatim.py:80: R3 (warn) single-letter local `l`
   tasks/v3-review-verbatim.py:80: R3 (warn) single-letter local `x`
   tests/integration/tests/common/mod.rs:218: R3 (warn) single-letter local `i`
   tests/integration/tests/m10_validation_test.rs:24: R3 (warn) single-letter local `p`
   tests/integration/tests/m10_validation_test.rs:35: R3 (warn) single-letter local `p`
   tests/integration/tests/m3_include_graph_test.rs:149: R3 (warn) banned identifier `res`
   tests/integration/tests/m3_include_graph_test.rs:150: R3 (warn) banned identifier `res`
   tests/integration/tests/m5_reduced_model_test.rs:86: R3 (warn) single-letter local `u`
   ```

   Baseline transition verified: BEFORE (journal + every prior verdict) =
   exactly 2 blocking rows `crates/ui/src/server.rs: R2 1172 lines >
   1000` + `crates/ui/src/server.rs:816: R1 fn api_graph_traversal is 161
   lines (> 100)`; AFTER = zero blocking rows, exit 0. The 3
   `crates/worker-core/src/tu_store.rs` rows and the count 179 (vs the
   journal's 178) belong to the foreign mid-session landing `79a90d3`
   (AR-20a), which touches no `crates/ui/**` path; R3 rows formerly at
   server.rs now attach to routes/api_graph.rs with identical
   identifiers (verbatim move, count preserved).

4. `python tools/encoding_guard.py --all` -> silent (no output).
   Via distill for the printed exit code: `### generic PASS (exit 0)`,
   capture 0 lines.
5. `python -m unittest discover -s tools -p "*_tests.py"` ->
   `Ran 219 tests in 22.198s` / `OK` (hygiene_check.py itself untouched
   by AR-13 - sanity confirmed).

## Locks / invariants (fresh)

- m16 UNEDITED: `python tools/testrun.py -p ariadne-integration-tests
  --test m16_traversal_test` -> `2 passed / 0 failed / 0 ignored /
  1 suites` (test_e2e_standalone_html_export_traversal,
  test_e2e_ui_traversal_and_pathfinding).
- m15 UNEDITED: `python tools/testrun.py -p ariadne-integration-tests
  --test m15_graph_ui_test` -> `1 passed / 0 failed / 0 ignored /
  1 suites` (test_m15_graph_ui_command).
  (Both runs needed `PYTHONIOENCODING=utf-8` - known testrun cp1252
  echo crash.)
- `ariadne_ui::server::StatusResponse` resolves through the shell
  re-export: un-edited server_api.rs:71/128 decode it inside the green
  265 suite.

## Acceptance-pass census (fresh-session reviewer substitution for
`--baseline` FILE mode)

`python tools/testrun.py --workspace -- --no-fail-fast` (with
PYTHONIOENCODING=utf-8) ->
`1372 passed / 0 failed / 13 ignored / 101 suites`
(= the 1372/0/13 at AR-23 acceptance, +2 empty suites: worker-core
unittests + doc-tests from the foreign mid-review landing `79a90d3`;
zero failures; only pre-existing warnings).
Note: `--no-fail-fast` direct-passthrough without `--` errored
`usage error: unknown flag: --no-fail-fast` (testrun arg handling;
tools-ticket family).

## Run / refused / not-run

RUN (each individually, own stdout): checkpoint_guard status; git log /
git show (aaffde0 stat+full -M, -M --find-copies-harder stat, 14e21b1,
768529f) / git diff --name-only; testrun -p ariadne-ui; distill ui_smoke;
hygiene_check (direct + distill); encoding_guard --all (direct +
distill); unittest discover; testrun m16; testrun m15; workspace census;
grep-based pub-surface counts + TraversalQuery single-occurrence proof.
REFUSED (boilerplate catch-all; each attempted then substituted):
- Write tool to the pre-approved temp dir -> `{"permission":"edit","pattern":"*","action":"deny","source":"agent"}`
  (my grant is tasks/** + .kilo/plans/** only - by design).
- heredoc-piped `python -` audit script -> `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
- one compound env+redirect command -> same bash boilerplate (retried as
  single plain commands, which ran).
- `git status --porcelain` -> same bash boilerplate; substituted with
  the git status tool.
Substitutions lost nothing: verbatim fidelity was adjudicated on the
FULL rename-detected diff (complete, not sampled).
NOT RUN: `testrun.py --baseline` (FILE mode, no baseline file in repo -
workspace census substituted per AR-23 precedent); per-commit
compile-green at the intermediate `aaffde0` revision (never checked
out; HEAD content builds green and the journal claims intermediates;
final-state gates are the contract); `bench_compare` / `ui_smoke
--release` (out of this contract); golden_guard (tool absent - T6).
Report-evidence check (distillation contract): journal is digest-shaped,
quotes the canonical lines, names capture paths, uses testrun/distill
entry points, zero raw dumps - PASS.
