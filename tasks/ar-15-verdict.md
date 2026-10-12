# AR-15 Verdict - Payload Arc clone + ETag/304 + spawn_blocking offload (P5/P6)

Reviewer: fresh-session adjudication of cd875de (7 paths) + cefbd6f (journal
hash record), base f49faeb (AR-20b journal; parent-of-landing). Milestone
context: post-AR-13 layout, branch `shining-enthusiasm`.

## VERDICT: APPROVED

## Ground truth (step 1)

`checkpoint_guard status`: HEAD cefbd6f, branch shining-enthusiasm, dirty
(none), untracked = docs/handoff artifacts only. AR-15 range is
`f49faeb..cefbd6f`; the implementer's journal base note (4dc7237) precedes the
AR-20b mid-wave landings (dcfc813/f49faeb) - wave reconciliation, anchors hold.

## Touch-set (step 4): EXACT

`git diff --name-only f49faeb..cefbd6f` = exactly the 7 paths the journal
claims: `crates/ui/src/state.rs`, `crates/ui/src/routes/api_payloads.rs`,
`crates/ui/src/routes/api_graph.rs`, `crates/ui/assets/api.js`,
`crates/ui/tests/ui_test/payload_etag.rs` (new), `crates/ui/tests/ui_test/main.rs`
(1 line `mod payload_etag;` - AR-01/AR-02/AR-10 registration precedent),
`tasks/ar-15-handoff-journal.md`. Assets change is api.js only.
`asset_injection_hardening.rs` is NOT in the changed-path set = UNEDITED, and
all 9 of its oracles are green in my own run. m15/m16/m24a UNEDITED (absent
from the path set). api_graph.rs = 914 lines < 1000 (verified).

## Per-commit review (step 3, oldest-first)

cd875de: one scoped commit, all 7 paths. cefbd6f: journal-only, 14 added
lines (post-commit gate record). No intermediate states to judge separately.

### P5 - ETag truthfulness (central audit)

- Snapshot construction (`api_payloads.rs::current_export_snapshot`):
  ONE `serde_json::to_vec(payload_ref)` over the shared payload reference on
  a miss; `etag = payload_bytes_etag(&bytes)` computed over exactly the served
  buffer; steady hits serve `Body::from(snapshot.bytes.clone())` - refcount
  bump, no re-serialization per request. No retained dual representation:
  the removed `Json(p.clone())` path is gone from the file.
- First-request byte-identity: pre-change served `Json(p.clone())`; axum
  0.7.9 `Json::into_response` (registry json.rs:190-201) serializes with
  compact `serde_json::to_writer` + `content-type: application/json` -
  byte-identical to `to_vec` + the new literal header. Verified against the
  pinned vendored source, not just the claim.
- Characterization lock: body == `to_vec(&fixture)` AND literal hash
  `993e43a23e1c6fe5`, green in my fresh runs. The hash is self-consistent
  with the byte lock (both over the same served bytes).
- Non-vacuity by structure: 304 lock asserts NOT_MODIFIED + empty body +
  repeated validator (kills a disabled 304 branch); constant-ETag mutation
  structurally trips the header-vs-body hash lock, the one-byte-304-not-served
  lock, AND the mismatch lock. Lock 7 (serialize counter = 1/generation, flat
  across polls, +1 per rebuild) closes the re-serialization hole lock 1
  cannot see. Mutation results cited in the journal match these structures.
- 304 only on exact match: `if_none_match == Some(etag)` string equality;
  lists/`*`/weak fall to the byte-identical 200 (locked).

### P5 - invalidation soundness (pair key)

Workspace-wide grep: `UiState::payload` has exactly TWO write sites -
cli `swap_into_state` (ui.rs:541) and the `new_loaded` constructor
(state.rs:126, epoch bumped at :143 during construction). `swap_into_state`
ALWAYS moves both keys: `install_analysis` (epoch, :539) precedes the payload
write; `stage_status` with `last_refresh: Some(now_ms())` (:542, :69) follows.
`rebuilding` toggles (ui.rs:588/:612) mutate stage_status content but not
`last_refresh` - correctly non-invalidating (export content unchanged). The
direct-write test precedent (server_api.rs:88->103) bumps the epoch right
after its payload write. Conclusion: NO production content mutation moves
neither key. A stale ETag cannot outlive a rebuild: any served snapshot whose
pair mismatches the live pair is refused by `export_snapshot_matching`.

### P6 - spawn_blocking

Exactly 5 wrap sites (api_graph.rs :58/:160/:427/:502/:873) matching the five
named handlers. The diff is 5 hunks, ALL inside handler regions (:50, :146,
:407, :475, :839); the five AR-13 decomposition helpers show ZERO diff hunks
(called exactly as before). Bodies verbatim (pure +4-space re-indent; no
semantic line differs). Query extractors run pre-wrap, so 400 ordering is
unchanged. JoinError -> 500 documented at each wrap.
Exclusions verified: `api_graph_entrypoints` = degree-HashMap scan
(`get_entrypoints`, traversal.rs:219/:381/:489 - no BFS/memo),
`api_graph_search` = payload array scan (`search_in`). Handoff said "BFS/memo
handlers" - the exclusions are plan-literal, and declared.

### api.js

- Boot fetch `fetch('/api/export').catch(() => null)` byte-kept (boot hunk
  adds only the `exportEtag` capture inside `if (exportRes && exportRes.ok)`);
  static_html `contains("fetch('/api/export')")` lock green in my run.
- 304 branch preserves export-derived fields (empty delta branch; fields only
  updated on fresh 200); `justificationCache = {}` + `applyLoadedData()` stay
  unconditional (graph datasets may move while the export is stable).
- Zero new markup interpolations, zero escaper/handler changes - AR-11
  discipline preserved (all 9 oracles UNEDITED green).
- The `lastSeenRefresh` hoist: pre-change blob independently verified -
  `let lastSeenRefresh` is block-scoped in `init()`'s else-block while the
  top-level `startLiveRefresh` reads it; the `s.last_refresh &&` short-circuit
  hides the ReferenceError until the server reports a refresh, after which
  the loop's catch swallows it - the delta branch was dead in shipped JS.
  The hoist is REQUIRED-BY-FEATURE (an ETag delta living in that branch would
  otherwise ship dead), not scope creep: 2 top-level declarations, boot
  assignment text kept.

## Deviation dispositions (step 5 - judged on merits, not rubber-stamped)

- D1 pair-keyed snapshot (epoch + `stage_status.last_refresh`) instead of the
  literal "serialize inside cli `swap_into_state`": ACCEPTED. The writer is
  genuinely outside `crates/ui/src/**`; invalidation independently proven
  sound above; impact targets machine-verified by lock 7.
- D2 `axum::body::Bytes` over `Arc<Vec<u8>>`/new dep: ACCEPTED. Touch-set is
  src-only; Bytes is the axum re-export; zero-copy `Body::from`. Cargo.toml
  untouched (verified: not in changed paths).
- D3 `lastSeenRefresh`/`exportEtag` hoist (latent ReferenceError fix):
  ACCEPTED as required-by-feature (see independent verification above).
- D4 `pub` instrumentation getter `export_serialization_count()` (surface
  +1): ACCEPTED - external ui_test lane cannot see pub(crate); no standing
  pub-count test exists (AR-13's 39-pub check was prose verification).
- D5 JoinError -> 500: ACCEPTED. Panics were never an expected path;
  documented at each wrap.
- D6 1-line `main.rs` registration: ACCEPTED per AR-01/AR-02/AR-10 precedent.
- D7 report.json CRLF churn restore: ACCEPTED; tree clean at review time.

## Numbered findings

1. MINOR (invariant, not a defect): invalidation pair is observed, not
   enforced. A future payload writer that moves NEITHER key would serve the
   stale snapshot forever. Today impossible (sole production writer signals
   both - grep-proven); invariant is documented in `state.rs` ExportSnapshot
   doc + journal but no lock pins it. Ledger deferred-findings candidate: a
   debug_assert or lock if a second payload writer is ever introduced.
2. MINOR (bounded window, no regression): the publish re-check is not atomic
   against an in-flight swap - a request landing between the epoch bump and
   the stage-status write can publish pre-swap bytes under the new epoch.
   The next request mismatches on the refresh key and refilters, so a STALE
   ETag cannot persist unless the swap rewrote `last_refresh` with an equal
   value - requiring two full pipeline builds inside one millisecond clock
   tick (practically unreachable; rebuilds are seconds apart). The same
   torn-swap window pre-existed for direct payload reads (swap_into_state's
   own doc). No action required.
3. MINOR (surface): +1 pub on UiState (D4) - accepted; recorded here so a
   future pub-surface count reconciles.
4. MINOR (foreign, tooling - already banked): `testrun.py --baseline` takes a
   FILE and no repo baseline exists, so the milestone-gate form is
   unrunnable-as-written; substituted the bare `testrun` (= `cargo test
   --workspace`) census per AR-03/AR-20a/AR-23 precedent. Closeout tools
   ticket reaffirmed.
5. MINOR (foreign, tooling - reproduced again): `testrun.py:262` cp1252 echo
   crash fired on my m15/m16/m24a run; workaround
   `PYTHONIOENCODING=utf-8`. Banked by AR-20a/AR-23.
6. MINOR (foreign, shared-target contention): first census attempt died with
   `crate ... required to be available in rlib format` races against the
   concurrently-running AR-20b reviewer's cargo in the shared target dir;
   retry clean (capture-race class, already banked).

## Follow-up dispositions (never dropped)

- Implementer's declared ledger candidate: `api_graph_include/symbol/call`
  still deep-clone their payload slice per request (P5-extension rider
  candidate). -> closeout deferred-findings row.
- Journal memory-mirror text (`ar15.snapshot.serving`) is owed by the
  orchestrator post-closeout, same pattern as AR-30.

## Gates (own stdout, verbatim) - full evidence in tasks/ar-15-execution-results.md

- ui: `272 passed / 0 failed / 0 ignored / 3 suites` (265 + 7 new locks)
- integration UNEDITED: `5 passed / 0 failed / 0 ignored / 3 suites`
- `ui_smoke: PASS` (attempt 1, via distill)
- `bench compare: 3 run(s), no regressions past 1.25x baseline`
- census: `1381 passed / 0 failed / 13 ignored / 101 suites`
- tools: `Ran 219 tests OK`
- hygiene: zero blocking rows (R3 warn-only); encoding: silent; theme guard:
  only the 3 AR-11-era informational flags, zero ADDED hex.
