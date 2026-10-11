# AR-15 Journal - Payload Arc clone + blocking-handler offload (P5 + P6 MED)

Session: AR-15 implementer, branch `shining-enthusiasm`, base HEAD `4dc7237`
(post-AR-13 layout). Touch-set honored: `crates/ui/src/**` (state + routes
only), `crates/ui/assets/api.js`, new ui lock file + its `main.rs` registration
line, this journal. Zero edits outside: `git diff HEAD --stat --
tests/integration crates/cli crates/graph crates/report` -> EMPTY (captured
via distill).

## What landed

1. **P5 snapshot serving** (`state.rs` + `routes/api_payloads.rs`):
   - `ExportSnapshot { epoch, refresh_key, bytes: axum::body::Bytes, etag }`
     held in `UiState::export_snapshot` (RwLock<Option<Arc<..>>>).
     `Bytes::clone` is a refcount bump over one immutable buffer - this IS
     the Arc-shared pre-serialized payload; literal `bytes::Bytes` chosen
     over `Arc<Vec<u8>>` so `Body::from(bytes)` is zero-copy AND no
     Cargo.toml change is needed (touch-set is src-only; `axum::body::Bytes`
     is the re-export).
   - Validation pair `(epoch, refresh_key)`: `analysis_epoch` (AtomicU64,
     bumped ONLY by `install_analysis` - the one generation swap used by
     cli boot, watcher rebuild, and the ui rebuild-sim locks) plus
     `stage_status.last_refresh` read per request. cli `swap_into_state`
     order is install(:539) -> call_graph -> payload(:541) -> stage_status
     (:542), so a swap-in-flight can poison at most the microsecond window
     between the epoch bump and the stage-status write - the same bounded
     torn-swap window `swap_into_state` documents for direct payload reads.
     A payload write that signals neither (epoch or last_refresh) keeps
     serving the old snapshot; the sole production writer signals both, and
     the standing direct-write lock (`server_api.rs:88`) installs an epoch
     bump immediately after its write.
   - `api_export`: hit = Arc bump; miss = ONE `serde_json::to_vec` over the
     shared payload REFERENCE (no deep clone; the only buffer is the wire
     bytes) + publish under seqlock re-check. ETag = FNV-1a/64 over exactly
     the served bytes, format `"<16 lowercase hex>"` (strong validator;
     spec documented on `payload_bytes_etag`, test carries an independent
     reference implementation). 304 ONLY on exact `If-None-Match` match -
     lists/`*`/weak tags get the full byte-identical 200 (first-request path
     untouched: status, `content-type: application/json`, bytes).
2. **P6 spawn_blocking** (`routes/api_graph.rs`, five handlers wrapped in
   place - no new fns, file stays flat, 914 lines < 1000):
   - `api_graph_traverse` (depth-BFS x3 graph families + payload filter)
   - `api_graph_path` (shortest-path BFS x3)
   - `api_graph_collateral` (memoized `analyze_file_load_by_id`)
   - `api_graph_traversal` (memo closure + extraction-impact projection)
   - `api_justification` (memoized justification recomputation)
   Bodies moved VERBATIM into `tokio::task::spawn_blocking` closures
   (precedent: `tokio::task::spawn_blocking` in `crates/cli/src/commands/
   ui.rs:575` watcher and `:830` boot - the re-anchored plan lines 567/798).
   Decomposed traversal helpers (`resolve_traversal_file_pair`,
   `ensure_traversal_edge`, `empty_traversal_response`,
   `resolve_traversal_symbols`, `project_traversal_subtrees`) are
   byte-identical and called exactly as before. Query extractors run before
   the wrap, so 400s keep their order. JoinError (blocking-task panic) maps
   to 500 - documented in the wrap comments; panics were never an expected
   path today either.
   NOT migrated (declared): `api_graph_entrypoints` (degree-scan, not BFS/
   memo per `get_entrypoints`), `api_graph_search` (payload array scan, and
   its 91-line body is a P7 R1-near case), `api_graph_include/symbol/call`
   (payload slice clones - per-slice serving is a P5 extension outside this
   handoff's "export payload" scope; ledger candidate).
3. **api.js ETag/delta-aware poll fetch** (`crates/ui/assets/api.js`):
   - Boot fetch UNCHANGED text (`fetch('/api/export').catch(() => null)` -
     the `static_html.rs` literal lock survives); it now only additionally
     records `exportEtag = exportRes.headers.get('ETag')`.
   - Live-refresh tick sends `If-None-Match: <exportEtag>` when held;
     `304` -> export-derived fields stay untouched (delta path), fresh 200
     -> validator + fields update as before. `justificationCache` clear and
     `applyLoadedData()` stay unconditional (graph datasets may change while
     the export snapshot is stable).
   - **Latent bug fixed in scope (deviation, disclosed):** `lastSeenRefresh`
     was `let`-bound inside `init()`'s else-block while the top-level
     `startLiveRefresh` loop READ it on every tick. Once the server reported
     any `last_refresh`, that read was a `ReferenceError` swallowed by the
     loop's catch - the delta branch was dead in shipped JS, so an ETag
     delta living there would have been unobservable. Both
     `lastSeenRefresh` and the new `exportEtag` are now declared at api.js
     top level (the only scope both sites share). Boot-path assignments
     (`lastSeenRefresh = status.last_refresh || 0`) keep their text.
   - **Escape preservation statement:** the api.js diff adds ZERO markup
     interpolation, ZERO inline handlers, ZERO escaper definitions or
     nestings - only fetch options, header reads, and one status branch.
     AR-11's `asset_injection_hardening.rs` (9 tests) and the `static_html`
     assign-literals (`graphData.file_collateral =`,
     `graphData.file_justification =`, `graphData.extraction_proposals =`)
     are UNEDITED and green in the suite runs.

## Characterization lock (pre/post payload hash)

- Fixture: deterministic payload (fixture include graph + sorted-emission
  collateral/justification maps + one proposal + stats) in
  `crates/ui/tests/ui_test/payload_etag.rs`.
- PRE-change (RED run): served `/api/export` body bytes ==
  `serde_json::to_vec(&payload)` (axum 0.7 `Json` is compact
  `serde_json::to_writer` + `application/json` - verified against the
  axum-0.7.9 source), FNV-1a/64 of those bytes =
  **993e43a23e1c6fe5** -> locked as a literal.
- POST-change: the same test green on the snapshot path - the ETag is
  computed over exactly the locked bytes; first-request path byte-identical.

## ETag semantics table (new locks)

| request to /api/export                     | status | body                  | ETag header           |
|--------------------------------------------|--------|-----------------------|-----------------------|
| first (no If-None-Match)                   | 200    | locked bytes (1x)     | `"<fnv1a64(bytes)>"`  |
| unchanged payload, exact If-None-Match     | 304    | EMPTY                 | same validator        |
| changed payload (1 byte), old If-None-Match| 200    | fresh bytes           | NEW validator != old  |
| mismatched tag / `*`                       | 200    | full locked bytes     | current validator     |
| payload absent (loading)                   | 503    | (unchanged Err path)  | none                  |

## Mutation non-vacuity (executed, never committed)

- Mutation A: 304 branch disabled (`if false && ...`) ->
  `6 passed / 1 failed` - exactly
  `test_export_unchanged_payload_serves_304_for_exact_if_none_match` red
  (`left: 200`). Reverted byte-exact.
- Mutation B: `payload_bytes_etag` returns a constant tag ->
  `4 passed / 3 failed`:
  `test_export_etag_header_locks_the_served_bytes` (tag no longer hashes the
  body), `test_export_etag_changes_when_one_byte_changes_and_304_is_not_served`
  (changed payload WRONGLY served 304 - the contract mutation "bump a byte
  -> 304 not served" is exactly what this lock kills), and
  `test_export_if_none_match_mismatch_serves_full_body` (constant tag made a
  "stale" tag match). Reverted byte-exact; suite re-greened.

## Lock inventory (new, payload_etag.rs, registered by 1 line in ui_test main.rs)

1. `test_export_payload_bytes_match_locked_serialization_and_hash` (byte + hash characterization)
2. `test_export_etag_header_locks_the_served_bytes` (etag == spec over served bytes)
3. `test_export_unchanged_payload_serves_304_for_exact_if_none_match`
4. `test_export_if_none_match_mismatch_serves_full_body`
5. `test_export_etag_changes_when_one_byte_changes_and_304_is_not_served` (mutation)
6. `test_etag_hash_is_single_byte_mutation_sensitive` (spec-level injectivity)
7. `test_steady_export_requests_serve_snapshot_clones_not_reserialization`
   (no-deep-clone lock via the new instrumentation accessor
   `UiState::export_serialization_count()` - 1 serialization per generation,
   flat across steady polls (incl. 304 ticks), exactly +1 per rebuild.
   Deviation: instrumentation is `pub` (external ui_test lane cannot see
   pub(crate)); surface +1 documented getter, no AR-13 pub-count test exists.)

## Design deviation: "on rebuild" -> rebuild-epoch-validated snapshot

The literal "serialize inside cli `swap_into_state`" is IMPOSSIBLE from this
touch-set (writer lives in `crates/cli` - out of bounds). The landing is
behaviorally equivalent from the outside: the rebuild invalidates the
snapshot atomically-enough via the epoch bump (inside `install_analysis`,
which IS in the ui crate and on every rebuild path); the first post-rebuild
request performs the one serialization over the shared reference and every
later request - every 2 s poll tick - is served from the Arc'd snapshot.
Impact targets met: per-request payload deep clone GONE (peak 2x -> wire
bytes only on the one miss, then ~0 extra); poll ticks near-free (304 with
empty body when unchanged, Arc bump when a 200 is needed).

## Gates (own stdout, verbatim; capture paths cited)

- RED (new locks vs pre-change code):
  `2 passed / 4 failed / 0 ignored / 2 suites` - failures exactly the
  predicted semantic ones (no ETag header; capture hash printed
  993e43a23e1c6fe5). capture: target/testrun-capture.txt
- Baseline pre-change ui: `265 passed / 0 failed / 0 ignored / 3 suites`
  capture: target/testrun-capture.txt
- ui suite after all changes: `272 passed / 0 failed / 0 ignored / 3 suites`
  (265 + 7 new locks; ALL pre-existing locks UNEDITED green incl. the 9 AR-11
  oracle tests, memo-counter reuse locks, rebuild-swap lock).
  capture: target/testrun-capture.txt
- Integration UNEDITED: `-p ariadne-integration-tests --test
  m15_graph_ui_test --test m16_traversal_test --test m24a_scala_plan_path_test`
  -> `5 passed / 0 failed / 0 ignored / 3 suites` (m15 1 + m16 2 + m24a 2;
  m24a is the /api/export consumer run as extra assurance).
  capture: target/testrun-capture.txt
- `python tools/distill.py -- python tools/ui_smoke.py`:
  `boot+route checks clean (cold fixture, attempt 1, 0.7s)` /
  `ui_smoke: PASS` capture: target/distill-capture.txt
- `python tools/distill.py -- python tools/bench_compare.py`:
  `bench compare: 3 run(s), no regressions past 1.25x baseline`
  stage rows (median ms): symbol_pass+prep 1530.9 (below baseline band
  ~1585.6), file_pass 618.9, save_load_artifacts+cache 2415.4, ensure_load
  total 4635.4, generate_report+detect_cycles 82.8, build_include_elements
  65.2, path_to_id+file_collateral 204.1, planner 896.7,
  build_file_justification_with_memo 8426.7.
  Attribution: the chronic clone rows land at 1.4/1.2 ms (cold/warm) -
  inside the pre-existing 1.20-1.23x near-ratio band carried since
  AR-06/AR-08/AR-09; this handoff touches no code behind
  `get_or_build_include_graph` (graph crate byte-untouched - see the
  `git diff --stat` line above). capture: target/distill-capture.txt (17 lines)
- `python tools/distill.py -- cargo check --workspace --all-targets`:
  `ok (exit 0, 21 warnings emitted)` - none from this diff; the two
  ui-referencing lines are the AR-13-INHERITED unused-`gen` warning on
  `build_subtree_nodes` (api_graph.rs:719, a helper byte-untouched here,
  declared inherited in AR-13's landing) and the pre-existing
  `cli/src/lib.rs:804` unused_mut. capture: target/distill-capture.txt
- Hygiene BEFORE: R3-warn-only, ZERO blocking rows. AFTER: identical name
  set (api_graph rows shifted line numbers only), ZERO blocking rows - no
  new R1/R2/R3 entries introduced.
- `python tools/encoding_guard.py --all`: silent, exit 0 (pre-commit).
- `python tools/encoding_guard.py --staged`: run after staging - see board
  RESULT for the final line (post-commit addenda below if any).
- `python tools/theme_token_check.py --check-added --base HEAD~1`: see
  addendum after commit.

## Capture paths (this session)

- target/testrun-capture.txt (each scoped run overwrites; numbers above are
  quoted from my OWN stdout per the capture-race rule, not re-read from disk)
- target/distill-capture.txt (same rule)

## Deliberately NOT done

- `api_graph_include/symbol/call` still deep-clone their payload slice per
  request (out of "export payload" scope; ledger candidate for a future P5
  rider if wanted).
- `api_graph_entrypoints` / `api_graph_search` stay on the async worker
  (degree/array scans, not BFS/memo; handoff named BFS/memo).
- No Cargo.toml change (bytes re-exported via axum); no m15/m16 edits; no
  hotspots.js touch (doc-level listener topology untouched); no theme token
  or golden fixture changes; no traversal.js diff.
- ui_smoke/bench rows NOT re-baselined (bench_compare passed neutral-or-better).

## Memory mirror owed (orchestrator)

- ar15.snapshot.serving: "/api/export serves a generation-validated
  ExportSnapshot (Bytes + FNV-1a/64 ETag, exact-match 304) from
  UiState::export_snapshot; epoch bump lives in install_analysis, second
  key is stage_status.last_refresh; payload_etag.rs locks the byte hash
  993e43a23e1c6fe5 for the AR-15 fixture."
