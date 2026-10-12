# AR-15 Execution Results - acceptance-pass evidence (reviewer, fresh session)

Reviewer run at HEAD cefbd6f (branch shining-enthusiasm), 2026-10-11. All
canonical lines quoted verbatim from my own stdout (capture-race rule).
Concurrent-lane note: the AR-20b reviewer was running lang-cpp/m17/m18/m00
gates in the shared worktree; the ui package and its gates are this review's
lane, and the shared-target contention is recorded under finding 6.

## Acceptance criteria vs actual

| # | Criterion (handoff gate table) | Command | Result (verbatim) |
|---|--------------------------------|---------|-------------------|
| 1 | ui suite, post counts + new locks | `python tools/testrun.py -p ariadne-ui` | `272 passed / 0 failed / 0 ignored / 3 suites` (265 + 7; all 7 payload_etag locks + all 9 AR-11 oracles + memo-reuse + rebuild-swap locks individually ok) |
| 2 | ui smoke | `python tools/distill.py -- python tools/ui_smoke.py` | `boot+route checks clean (cold fixture, attempt 1, 1.0s)` / `ui_smoke: PASS` (no retry needed) |
| 3 | bench neutral-or-better | `python tools/distill.py -- python tools/bench_compare.py` | `bench compare: 3 run(s), no regressions past 1.25x baseline` (clone rows 1.5/1.1 = chronic near-ratio band from AR-06/AR-08/AR-09; graph crate untouched) |
| 4 | m15/m16 (+m24a) UNEDITED | `python tools/testrun.py -p ariadne-integration-tests --test m15_graph_ui_test --test m16_traversal_test --test m24a_scala_plan_path_test` | `5 passed / 0 failed / 0 ignored / 3 suites` (paths absent from the changed-path set = UNEDITED) |
| 5 | theme guard, api.js row | `python tools/theme_token_check.py --check-added --base f49faeb` | ONLY the 3 AR-11-adjudicated informational frozen-count flags (`index.html 6>4`, `main.js 1>0`, `traversal.js 1>0` - all byte-untouched files); api.js raises NO flag; zero ADDED hex from this diff |
| 6a | encoding | `python tools/encoding_guard.py --all` | silent (pass contract; raw exit code not echoable on the allowlist - AR-01/AR-02 precedent) |
| 6b | hygiene | `python tools/hygiene_check.py` | ZERO blocking rows - every reported row is `R3 (warn)` (post-AR-13 zero-blocking baseline holds; touched files state.rs/api_payloads.rs/payload_etag.rs contribute zero rows) |

## Workspace-wide acceptance pass

- `python tools/testrun.py --baseline` -> `usage error: missing value for
  --baseline` - the flag takes a FILE and the repo ships no baseline file
  (finding 4; AR-03/AR-20a/AR-23 substitution precedent). Census run instead:
  first attempt (bare `python tools/testrun.py`) died on shared-target
  artifact races from the concurrent AR-20b reviewer (`error: crate
  'h2' required to be available in rlib format, but was not found in this
  form` + windows_strings/base64/regex_syntax/ariadne_ui, `parse format
  drift`); RETRY attempt 2 (capture-race rule) clean:
  `1381 passed / 0 failed / 13 ignored / 101 suites`
  Reconciles: 1372 @ AR-20a census + 7 AR-15 locks + 2 AR-20b m17 tests
  (guard + planted-edge self-test landed between the censuses).
- `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests OK`

## Deviation/gate friction reproduced (foreign, banked classes)

- `testrun.py:262` cp1252 `UnicodeEncodeError` fired on the m15/m16/m24a run;
  reran with `PYTHONIOENCODING=utf-8` (finding 5).

## Independent source-level verifications (not from the journal)

- axum 0.7.9 `Json::into_response` = compact `serde_json::to_writer` +
  `application/json` (vendored json.rs:190-201) -> first-request byte-identity
  of the replacement path holds against the pinned dependency source.
- Workspace-wide write-site grep for `UiState::payload`: exactly 2 sites
  (cli `swap_into_state` ui.rs:541; `new_loaded` ctor state.rs:126 with epoch
  bump at :143). `swap_into_state` necessarily moves BOTH validation keys
  (install_analysis :539 before; stage_status w/ `last_refresh: Some(now_ms())`
  :542 after). No content mutation path moves neither key.
- api_graph.rs = 914 lines; exactly 5 spawn_blocking wrap sites; the 5 AR-13
  traversal helpers have zero diff hunks; `get_entrypoints` is a degree-HashMap
  scan (traversal.rs:219/:381/:489), `api_graph_search` a payload array scan -
  both exclusions plan-literal ("BFS/memo") and independently confirmed.
- Pre-change api.js blob: `lastSeenRefresh` was `let`-bound inside init()'s
  else-block, read by the top-level `startLiveRefresh` loop -> shipped delta
  branch was ReferenceError-dead once the server reported `last_refresh`
  (short-circuit hid it until then; loop catch swallowed it). The hoist is
  required for the ETag delta to execute at all.
- FNV-1a/64 single-byte injectivity claim: correct (per-step state map is
  xor-bijective then odd-multiplication-bijective, so a one-byte difference
  cannot collapse); lock 6 exercises it.

## NOT run by this review (with attribution)

- `cargo check --workspace --all-targets` as a standalone command - not run;
  the workspace census (attempt 2) compiled all crates including all bins and
  test targets with zero errors and zero warnings attributable to AR-15 paths
  (the two visible warnings - api_graph.rs:719 unused `gen`, cli/lib.rs:804
  unused_mut - are AR-13-inherited and b49ed06-inherited respectively).
- lang-cpp/m17/m18/m00/scheduler package gates - AR-20b reviewer's lane, run
  concurrently there; nothing in the AR-15 range touches those crates
  (changed-path set is crates/ui + tasks only).
- Stage-5 RSS / memory probes - not part of this handoff's gate table.
- `golden_guard.py` - does not exist (T6 open); the prose byte-identity checks
  named in this contract (asset_injection_hardening UNEDITED, m15/m16 UNEDITED)
  were verified by changed-path-set absence + green re-runs instead.
