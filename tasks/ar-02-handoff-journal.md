# AR-02 Handoff Journal - Localhost default bind + same-origin CORS (S6 HIGH)

Session: implementer dispatch (2nd attempt; 1st aborted mid-flight).
HEAD at boot: 932041a (AR-01 6d7f96b present, plus AR-03/AR-04 riders).
Branch: shining-enthusiasm worktree.

## WIP reconciliation (per file, from the aborted dispatch)

`python tools/checkpoint_guard.py status` at boot showed 5 dirty files, all
inside the AR-02 touch-set. The reviewer had warned m16/m17 did not build
(E0061 run_ui arity) and that server_api.rs sat at exactly 1000 lines.

- `crates/cli/src/lib.rs` (WIP +78/-1): COMPLETE - KEEP.
  Adds `expose: Option<String>` to `Commands::Ui` with doc-comment naming the
  127.0.0.1 default and the warning, threads it through `run_cli` ->
  `dispatch_ui` -> `run_ui`, and adds 4 lib tests (flag parses, carries the
  address, help text documents default+warn, absence keeps localhost
  default). `pub use commands::ui::*` makes `resolve_bind_addr` visible to
  the test mod. Verified compiling and passing.
- `crates/cli/src/commands/ui.rs` (WIP +26/-1): COMPLETE - KEEP.
  `resolve_bind_addr(expose)` returns 127.0.0.1 unless `--expose` is
  non-blank; `warn_if_expose_used` prints the LAN-exposure warning to stderr
  on every boot that widens the bind; the hardcoded `0.0.0.0` bind line is
  replaced with the resolved address.
- `crates/cli/src/commands/ui/tests.rs` (WIP +40): COMPLETE - KEEP.
  Three locks: default/tram-blank resolution, honor-override, and a
  production-source lock that forbids the hardcoded wildcard and requires the
  warn path.
- `crates/ui/src/server.rs` (WIP +22/-2): COMPLETE - KEEP.
  `is_same_origin` predicate + `CorsLayer::new().allow_origin(AllowOrigin::
  predicate(is_same_origin))` replaces `CorsLayer::permissive()`. Foreign
  origins receive no `access-control-allow-origin` header at all; only an
  Origin exactly equal to scheme+Host of the request is mirrored.
- `crates/ui/tests/ui_test/server_api.rs` (WIP +58): RELOCATED - REVERTED IN PLACE.
  The WIP pushed this file to exactly 1000 lines, breaching R2 (files must be
  UNDER 1000). Following the AR-01 precedent (new mod `source_containment.rs`
  + 1-line main.rs registration), the CORS block was moved to the new mod
  `cors_same_origin.rs` and server_api.rs was returned byte-identical to HEAD
  (verified: `git diff` empty for it; back to 942 lines).
- `crates/ui/tests/ui_test/cors_same_origin.rs` (NEW): CREATED by relocation.
  Same three tests plus the `allow_origin_header` helper, unchanged content.
- `crates/ui/tests/ui_test/main.rs` (+1): registers `mod cors_same_origin;`.

## Call-site repair (contract item 2: fix ALL call sites)

The WIP changed `run_ui` to a 7-arg signature but updated zero integration
call sites, which is what broke the m15/m16/m17/m18/m21/m22 builds.
Fixed all 8 call sites to pass `None` (no --expose, keep 127.0.0.1):
m15_graph_ui_test (2), m16_traversal_test (2), m17_dir_nav_test (1),
m18_persistence_incremental_test (1), m21_run_ui_smoke_test (1),
m22_scala_load_analysis_test (1).
DEVIATION NOTE: these 6 files are outside the handoff's touch-set list but
the dispatch instruction explicitly assigned "fix ALL call sites so the
integration tests build again"; they are arg-only edits (added `None`).

## TDD evidence (Red-Green-Refactor)

RED was demonstrated by temporarily reverting each production fix under the
already-written tests:

- CORS RED (layer line reverted to `CorsLayer::permissive()`):
  `python tools/testrun.py --distill -- -p ariadne-ui --test ui_test test_cors_`
  digest: `0 passed / 3 failed / 0 ignored / 1 suites`
  All three failed on `left: Some("*")` - the permissive wildcard grant, the
  exact S6 semantic being closed.
- Bind RED (bind line reverted to hardcoded `0.0.0.0:{port}`):
  `python tools/testrun.py --distill -- -p ariadne-cli --lib test_run_ui_binds_resolved_addr_never_hardcoded_wildcard`
  digest: `0 passed / 1 failed / 0 ignored / 1 suites`
  Failure message: "run_ui must bind the --expose-resolved address, not a
  hardcoded host".
- GREEN (fixes restored, same two scoped commands):
  `3 passed / 0 failed / 0 ignored / 1 suites`
  `1 passed / 0 failed / 0 ignored / 1 suites`
- The lib.rs `--expose` parse/help tests are structurally impossible to run
  against HEAD (the field does not exist), so their RED is by construction;
  they pass on the completed implementation (inside the ariadne-cli gate).
- Refactor pass: no duplication found to remove; the relocated block is the
  only structural move. Re-ran scoped suites after - no regression.

## Gate transcripts (canonical lines, verbatim)

1. `python tools/testrun.py --distill -- -p ariadne-ui`
   `245 passed / 0 failed / 0 ignored / 3 suites`
2. `python tools/testrun.py --distill -- -p ariadne-cli`
   `118 passed / 0 failed / 8 ignored / 6 suites`
3. Previously-broken integration builds:
   `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m16_traversal_test`
   `2 passed / 0 failed / 0 ignored / 1 suites`
   `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m17_dir_nav_test`
   `1 passed / 0 failed / 0 ignored / 1 suites`
   Full-package compile check (covers m15/m18/m21/m22 too):
   `python tools/distill.py -- cargo test -p ariadne-integration-tests --no-run`
   exit 0; every test executable emitted. 21 warnings, all pre-existing
   classes (server.rs unused `TraversalQuery` import / unused `gen` - both
   outside this diff; m7/common/mod.rs etc.).
4. `python tools/distill.py -- python tools/ui_smoke.py`
   `boot+route checks clean (cold fixture, attempt 1, 1.1s)` +
   `ui_smoke: PASS` - smoke proves the default 127.0.0.1 bind still serves
   every route to a same-machine client.
5. `python tools/encoding_guard.py --all` - silent (its contract: one line
   per violation, exit 1 on any; silence = zero violations, exit 0).
6. `python tools/hygiene_check.py` - blocking output is exactly the two
   pre-existing AR-13-owned server.rs rows (`R2 1155 lines > 1000`,
   `R1 api_graph_traversal 161 lines`); ZERO new R1/R2. All R3 lines are
   pre-existing (the `ui.rs:792` single-letter `p` is DP-1 code shifted by
   this diff, not authored here). New files/fns (cors_same_origin.rs,
   resolve_bind_addr, warn_if_expose_used, is_same_origin) produce no findings.
Note: hygiene R2 on server.rs predates AR-02 (HEAD was 1137 lines, already
>1000); AR-13 owns the extraction. This change adds ~18 lines to that file
under the dispatch's explicit allowance.

## Deliberately NOT done / left alone

- `crates/graph/src/graph.rs` dirty with a foreign WIP (AR-06 `seen_edges`
  dedup) appeared mid-session; it is another worker's in-flight scope. Left
  untouched, not staged, not reverted.
- `tests/fixtures/basic/artifacts/includes/report.json` shows line-ending
  churn only (LF->CRLF notice, zero content diff); not staged.
- No changes to golden fixtures, byte-locked manifests, or theme baselines.
- No hex literals added (no styling touched at all).
