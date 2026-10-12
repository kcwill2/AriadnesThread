# W-2b Verdict - ui-extraction-wizard: reader trait, wizard DTOs, two GET routes

Handoff: `tasks/ui-extraction-wizard-w2b-handoff.md`
Journal: `tasks/ui-extraction-wizard-w2b-journal.md`
Range under review: `7b2238c..27cb9de` (landing `f5a7d80`, parent `7b2238c`;
journal rider `27cb9de`)

## Verdict: APPROVED

Zero blockers. Six numbered findings, all minor or relay; none blocks the
landing. The frozen seam W-2a plugs into is byte-complete and independently
provable from an external crate, the error contract is uniform across both
routes, and the workspace census reconciles to the ledger anchor with zero
residual.

## Ground truth (before judging)

`python tools/checkpoint_guard.py status`:

- HEAD `27cb9de`, branch `main`, ahead 0 / behind 0, dirty 0, untracked 0.
- Commits: `27cb9de` (docs rider), `f5a7d80` (landing), `7b2238c` (W-1 verdict
  tracking, outside this handoff).
- Worktree census: 5 worktrees, none owned by this milestone; the only dirty
  entries are foreign-track files in `mergetest` (272) and
  `tarjan-s-scc` (2) / `shining-enthusiasm` (1), all pre-existing.

## Fresh verification (re-run by the reviewer; nothing trusted)

| Gate | Command | Result |
|---|---|---|
| V1 scoped ui | `python tools/testrun.py --distill -p ariadne-ui` | `### cargo test PASS (exit 0) [capture: ...testrun-capture.txt, 323 lines]` / `294 passed / 0 failed / 0 ignored / 3 suites` / `no failures` |
| V2 F14 cli | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` | `ok (exit 0, 0 warnings emitted) -> capture: ...distill-capture.txt` |
| V3 hygiene | `python tools/distill.py -- python tools/hygiene_check.py` | `### generic PASS (exit 0)`; 179 capture lines, every row R3 warn-only in untouched files; **zero R3 rows in any of the six touched paths** (verified by sweeping the capture for `crates/ui/src/routes/api_extraction.rs`, `state.rs`, `dto/extraction.rs`, `server.rs`, `lib.rs`, `tests/ui_test/extraction_wizard_api.rs` - no match) |
| V4 encoding | `python tools/distill.py -- python tools/encoding_guard.py --all` | `### generic PASS (exit 0)`; capture 0 lines |
| V5 theme tokens | `python tools/distill.py -- python tools/theme_token_check.py --check-added --base 7b2238c` | `### generic PASS (exit 0)`; capture 0 lines (silent, zero added hex) |
| V6 encoding staged | `encoding_guard.py --staged` | not reproducible post-hoc (staging empty; `commit_scoped.py` owns staging - W-1 verdict gate-6 precedent); superseded by V4 `--all` |
| V7 workspace census | `python tools/testrun.py --distill` | `1458 passed / 0 failed / 13 ignored / 112 suites` exit 0, capture 2767 lines (see the execution-results artifact for the arithmetic) |
| V8 tools suites | `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 219 tests in 23.039s` / `OK` |
| V9 ui_smoke | `python tools/distill.py -- python tools/ui_smoke.py` | `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture, attempt 1, 1.0s)` / `ui_smoke: PASS` |
| V10 bench | `python tools/distill.py -- python tools/bench_compare.py` | `### generic FAIL (exit 2)` / `ERROR parse format drift: documented phase row(s) missing from bench output (never substituting zeros):` / `  missing documented phase: generate_report+detect_cycles (combined)` - the known stale-`DOCUMENTED_PHASES` failure, exempted with the zero-diff proof below |
| V11 baseline FILE form | `python tools/testrun.py --baseline` | `usage error: missing value for --baseline` (exit 2) - the 8th encounter of the standing ledger errata row; `--baseline` is a FILE value flag and no baseline census file is committed. Superseded by V7 |

Every gate the handoff declared and every workspace gate this hop owes has
evidence above; nothing was skipped silently. Refusals are itemized in
Finding 6.

## Commit-vs-touch-set (enforced)

`git diff --name-status "7b2238c" "HEAD"` -> exactly nine paths:

```
M       crates/ui/src/dto/extraction.rs
M       crates/ui/src/lib.rs
A       crates/ui/src/routes/api_extraction.rs
M       crates/ui/src/routes/mod.rs
M       crates/ui/src/server.rs
M       crates/ui/src/state.rs
A       crates/ui/tests/ui_test/extraction_wizard_api.rs
M       crates/ui/tests/ui_test/main.rs
A       tasks/ui-extraction-wizard-w2b-journal.md
```

All nine are inside the declared touch-set, which names exactly these paths.
Per-commit, oldest first:

- `f5a7d80` (landing, parent `7b2238c`): the nine paths above, `808
  insertions(+)`, zero deletions - dto/extraction.rs 98, lib.rs 4,
  routes/api_extraction.rs 103 [NEW], routes/mod.rs 1, server.rs 3, state.rs
  25, extraction_wizard_api.rs 371 [NEW], main.rs 1, journal 202. Nothing
  else. No `Cargo.lock` regeneration.
- `27cb9de` (rider): `tasks/ui-extraction-wizard-w2b-journal.md` alone,
  `54 insertions(+), 2 deletions(-)`. Docs-only, inside the touch-set.

No commit lands outside the touch-set. The nine-path listing is the machine
proof for every frozen lock: `tools/theme_token_baseline.json`, `tests/
golden/**`, `tests/fixtures/**`, assets, `themes.css`, all 18 card-family
locks (`extraction_card_story.rs`, `extraction_card_playbook.rs` and the
other 16) and all of `crates/cli/**` appear in no diff of this hand.
Independently confirmed: `git diff --numstat "7b2238c" "HEAD" -- crates/cli
tools/bench_compare.py tools/bench_baseline.json crates/pipeline` returns
empty (positive control on the same command: `25 0
crates/ui/src/state.rs`), so the bench exemption rests on zero-diff proof,
not on a silent pass.

## Plan-literal compliance

- Trait (handoff lines 70-79, shape C2): landed verbatim at
  `routes/api_extraction.rs:28-34` - `pub trait
  ExtractionManifestReader: Send + Sync + 'static` with `epoch() -> u64`,
  `manifest(&self, plan_id: &str) -> Option<ExtractionManifestDto>`,
  `dry_run(&self, plan_id: &str) -> Option<ExtractionDryRunDto>`. Object-safe
  (proven by `Arc<dyn ExtractionManifestReader>` compiling in both the
  handler and an out-of-crate test). No dual definition anywhere in the tree
  (a workspace grep for the eight new names finds one definition site each,
  the rest are call sites, locks and docs).
- Caps (handoff lines 87-89): `dto/extraction.rs:79/84/87` = `64` / `200` /
  `200`, exact, all `pub const`, re-exported to `ariadne_ui::*` through
  `dto.rs:10` (`pub use extraction::*;`) and `lib.rs:8` (`pub use
  dto::*;`).
- DTO shape (handoff lines 81-84, shape C2 list): all four structs land with
  the exact field names and order, every field `#[serde(default)]`, all
  serde-plain types. No cluster/transform/validate type crosses the boundary.
- State seam (handoff lines 92-94): one field
  `extraction_reader: RwLock<Option<Arc<dyn ExtractionManifestReader>>>`
  (`state.rs:97-98`) with `install_extraction_reader` (`:191`) and the
  `extraction_reader()` accessor (`:197`). `new_loading` (`:102`) and
  `new_loaded` (`:124`) keep their signatures and initialize the field to
  `None` (`:120`, `:148`) - no existing call site moved, which is what V2
  proves.
- Check order (handoff lines 41-43): `reader_ready_for_request`
  (`routes/api_extraction.rs:68-78`) implements exactly no reader -> 503,
  reader epoch != `analysis_epoch()` -> 409 with the live epoch, then the
  handler's None -> 404 / Some -> 200. Both handlers route through the one
  shared helper, so the order is uniform by construction, not by duplication.
- Routes (handoff lines 96-98): `server.rs:78-79` registers
  `/api/extraction/:plan_id/manifest` and `/api/extraction/:plan_id/dry-run`,
  each with `get(...)` only. Plan id is read as a plain `Path<String>`; no
  path parsing.
- Module wiring: `lib.rs:14` one `pub use
  routes::api_extraction::ExtractionManifestReader;`; `routes/mod.rs:4` one
  `pub(crate) mod api_extraction;`; `server.rs:15` one `use` line;
  `main.rs:11` one mod line inserted immediately after
  `mod extraction_contrast;` and before `mod file_view;` - the exact site the
  handoff fixed for review-diff stability.
- Test count and arithmetic: 11 new tests (4 manifest route shapes + 4
  dry-run route shapes + 3 string locks), matching the handoff's required
  list 1-8 exactly. 283 baseline + 11 = 294, reproduced fresh at V1.

## Frozen-seam audit for W-2a (the chain check)

W-2a's handoff expectations against what W-2b actually landed:

| W-2a expectation | Landed | Verdict |
|---|---|---|
| `ariadne_ui::ExtractionManifestReader` implementable by a cli type | `lib.rs:14` exports it; trait is `pub`, object-safe, `Send + Sync + 'static` | PASS |
| Return the wizard DTOs by exact field name | all 18 field names present and `pub` under `ariadne_ui::*` | PASS |
| Enforce the three caps | `EXTRACTION_MANIFEST_MAX_FILES`, `..._MAX_SYMBOLS_PER_FILE`, `EXTRACTION_DRY_RUN_MAX_REWRITES` all `pub const` at the root | PASS |
| `UiState::install_extraction_reader(&self, Arc<dyn ExtractionManifestReader>)` | `state.rs:191`, `pub` | PASS |
| `epoch()` returns the generation the reader was built for | trait method present, no constraint that would block W-2a's construction | PASS |
| W-2a's own surface untouched | `crates/cli/**` diff-empty over the range | PASS |
| Out-of-crate implementor proof | `tests/ui_test/extraction_wizard_api.rs` implements the trait and installs it from an external crate (`impl ExtractionManifestReader for ScriptedReader` + `state.install_extraction_reader(Arc::new(...))`) and all four shapes pass - this is the same call W-2a will make | PASS |

Seam drift found: none. Observation recorded as Finding 4.

## Acceptance items (handoff "Required tests" 1-8), one by one

| # | Item | Verdict | Evidence |
|---|---|---|---|
| 1 | Manifest: no reader installed -> 503 `analysis_loading` | PASS | `test_manifest_route_returns_503_analysis_loading_while_no_reader ... ok` (fresh capture line 1983); body asserted equal to `{"error":"analysis_loading"}` and ASCII-checked |
| 2 | Manifest: reader for the live epoch, known plan -> 200 with the DTO JSON | PASS | `test_manifest_route_returns_200_with_reader_manifest ... ok` (line 1982); body asserted equal to the reader's serialized DTO |
| 3 | Manifest: same reader, unknown plan id -> 404 `unknown_plan` with the id | PASS | `test_manifest_route_returns_404_unknown_plan ... ok` (line 1981); echoed id asserted, ASCII-checked |
| 4 | Manifest: epoch != live -> 409 `stale_generation` with the live epoch; body ASCII | PASS | `test_manifest_route_returns_409_stale_generation ... ok` (line 1984); `"epoch":1` substring asserted, ASCII-checked. Byte order of the body is Finding 1 |
| 5 | Dry-run: the same four shapes (503/200/404/409) | PASS | lines 1973 (200), 1974 (404), 1978 (409), 1979 (503), all `ok` |
| 6 | Lock: `server.rs` registers both paths with `get(` only | PASS | `test_server_registers_extraction_routes_with_get_only ... ok` (line 1980) |
| 7 | Lock: module contains no `fs::write`, `fs::remove_file`, `fs::create_dir`, `ArtifactManager` | PASS | `test_extraction_route_module_has_no_write_calls ... ok` (line 1977) |
| 8 | Lock: module contains no `post(`, `put(`, `delete(`, `patch(` | PASS | `test_extraction_route_module_has_no_mutation_route_registrations ... ok` (line 1972) |

Handoff gates 1-8: all discharged by the implementer and independently
corroborated - G1 `283 passed / 0 failed / 0 ignored / 3 suites`, G2 exit 1
with the six named failures, G3 `294 passed / 0 failed / 0 ignored / 3
suites` (reproduced fresh at V1), G4 `ok (exit 0, 0 warnings emitted)` (V2),
G5 hygiene exit 0 (V3), G6 via the V4 superset, G7 landing `f5a7d80`,
G8 `--all` exit 0 (V4).

## Pure-ASCII check on newly authored lines

- `crates/ui/src/routes/api_extraction.rs` (103 new lines): zero non-ASCII
  matches.
- `crates/ui/tests/ui_test/extraction_wizard_api.rs` (371 new lines): zero
  non-ASCII matches.
- dto/extraction.rs additions (lines 70-166), state.rs additions (lines 4,
  92-98, 120, 148, 186-199), lib.rs additions (11-14), server.rs additions
  (15, 78-79), routes/mod.rs addition (line 4), main.rs addition (line 11):
  zero non-ASCII matches. The only non-ASCII in the two touched files sits in
  pre-existing regions (dto/extraction.rs:30/44, the W-1 module doc;
  state.rs:16/18/23/34/40/88/157, pre-W-2b).
- `tasks/ui-extraction-wizard-w2b-journal.md`: zero non-ASCII matches.

The W-1 lesson is honoured: nothing newly authored carries a glyph.

## Deviation adjudication (independent)

- **D1 (Red doc-hygiene) - ACCEPTED.** Verified independently: the first Red
  run really would have tripped the new lock, because the module doc quoted
  the literal `ArtifactManager` while the lock at the bottom of the same file
  asserts the module source does not contain it. The reworded doc
  (`routes/api_extraction.rs:8-9`) states the rule without quoting the
  token; the lock is unweakened (it still asserts absence of the token over
  the whole module source), and the module genuinely contains no
  `ArtifactManager`. Red then reproduced with exactly the six predicted
  failures, matching the handoff's own predicted shape. No scope impact.
- **D2 (path-parameter syntax) - ACCEPTED.** `axum = "0.7"` is the workspace
  pin (`Cargo.toml:42`), and axum 0.7 registers path parameters with `:name`.
  The requested URLs `/api/extraction/PLAN-002/manifest` and
  `/api/extraction/PLAN-002/dry-run` are identical to the documented form,
  and the trait/DTO/state seams W-2a plugs into are untouched. The new test
  lock and the module doc both carry the note, so the documentation form and
  the compile-forced syntax are reconciled in-tree. Compile-forced syntax,
  not a contract change.
- **D3 (Red skeleton handler shape) - ACCEPTED.** The handoff's Red skeleton
  required "handlers whose ONLY implemented branch is the no-reader 503"; the
  implementer used `State(_state)` / `Path(_plan_id)` and returned the 503
  unconditionally rather than an `is_none()` branch deleted at Green. The
  observable prediction the handoff pinned ("every response is
  `503 {"error":"analysis_loading"}`") is what the six failures recorded, and
  the Green handlers carry the full C3 order. Process-internal shape of a
  throwaway skeleton; no observable difference.
- **D4 (hygiene first-invocation form) - ACCEPTED.** The run of record is the
  `distill.py` re-invocation; I re-ran it fresh (V3) and got the same exit 0
  with zero R3 rows in any touched path. Process deviation with a correct
  substitute.
- **D5 (journal rider) - ACCEPTED.** The rider is inside the touch-set (the
  journal path is scope 1), docs-only, and numbered up front per W-1 verdict
  Finding 1's discipline. Direct precedent: W-1's `b27aa46` + `82e5ffe`.

Undeclared deviations found in per-commit review: none.

## Report-evidence check (distillation discipline)

The journal is digest-shaped throughout: gate numbers, the distilling entry
point used, exit codes, canonical lines verbatim, capture paths, and a
completion record. Zero raw output dumps. High-volume commands all route
through `testrun.py --distill` or `distill.py -- ...`; `encoding_guard`
invocations are direct but are silent by design and are the documented
pre-commit/post-commit pair. Staging used
`python tools/distill.py -- git add -- <the nine explicit paths>` (never
`git add .`), with the staged set verified before committing. PASS.

## Findings

1. **Minor** - `crates/ui/src/routes/api_extraction.rs:90` - the 409 error
   body's serialized key order does not match the literal pinned by the
   handoff (line 40) and shape C3 (line 193).
   `serde_json::json!({ "error": "stale_generation", "epoch": live_epoch })`
   builds a `serde_json::Value::Object`, and the workspace pins plain
   `serde_json = "1.0"` with no `preserve_order` feature, so the map is
   `BTreeMap`-backed and serializes keys sorted: the wire bytes are
   `{"epoch":<live>,"error":"stale_generation"}`, not
   `{"error":"stale_generation","epoch":<live>}`. Independent corroboration
   in-tree: `tests/integration/tests/common/mod.rs:9-11` documents exactly
   this BTreeMap behaviour, and `tasks/m235-t3d2-verdict.md:191-199` recorded
   the same nuance as "recorded, not charged". The pinned *content* (key
   names, values, ASCII) is exact, no consumer reads body order (W-3b reads
   by key), and the new test asserts parsed-JSON equality plus the
   order-agnostic substring `"epoch":1`, so it is not evidence of order.
   Non-blocking. Remediation if byte parity is ever wanted: a small
   `#[derive(Serialize)]` error struct with declaration order `error` then
   `epoch`, or an amended shape C3 note that the 409 body is BTreeMap-sorted.
2. **Minor** - `crates/ui/src/routes/api_extraction.rs:82-84` - the 503 now
   carries a body where the crate's existing 503 convention carries none.
   Every pre-existing loading refusal is a bare
   `StatusCode::SERVICE_UNAVAILABLE` with an empty body
   (`routes/api_graph.rs:25, 35, 45, 70, 78, 97, 113, 166, 217, 223, 248`
   and `routes/api_payloads.rs:145`), and the "locked convention" the handoff
   cites (`tests/ui_test/graph_api.rs:268-289`) asserts only the status,
   never a body. The W-2b handoff and shape C3 both pin the body explicitly,
   so the implementation is plan-literal; recorded because W-3b's JS will now
   see two different 503 shapes inside one app, and shape C3's sentence "The
   `503` while-loading shape follows the existing convention" reads as if
   they were the same shape.
3. **Minor (observation)** -
   `crates/ui/tests/ui_test/extraction_wizard_api.rs:148` - the new
   `request_json_body` helper duplicates `common::request_route` /
   `common::request_raw_body` (`tests/ui_test/common.rs:325-354`) rather than
   reusing them. Justified and verified: this module needs all three of
   (status, parsed JSON, raw bytes) from one request, `request_route` returns
   `serde_json::Value::Null` for any non-200 status (`common.rs:336-340`), and
   `request_raw_body` returns no parsed JSON - neither supplies all three, so
   the helper is additive rather than redundant. No change required.
4. **Minor (observation)** - `crates/ui/src/routes/api_extraction.rs:30` -
   the landed trait omits the doc comment
   `/// The analysis generation this reader was built for.` that shape C2
   (`tasks/ui-extraction-wizard-shape.md:123`) places on `epoch()`. The
   W-2b handoff's own quoted trait (lines 70-79) omits it too, so the
   implementer followed the binding contract and the divergence is in the
   shape doc. Zero seam impact.
5. **Relay** - `tools/bench_compare.py:84` pins `DOCUMENTED_PHASES` entry
   `generate_report+detect_cycles (combined)` while the stage-5 bench prints
   `generate_report+detect_cycle_groups (combined)` (M23.5 rename). The gate
   therefore exits 2 with `parse format drift` on every run on this machine,
   before and after W-2b: reproduced fresh at V10 with the verbatim two-line
   error. Zero-diff proof for the exemption this hop: `git diff --numstat
   "7b2238c" "HEAD" -- crates/cli tools/bench_compare.py
   tools/bench_baseline.json crates/pipeline` returns empty, and the
   W-2b range contains no path under `crates/cli/**` or `tools/**`. No bench
   number was silently passed; the gate ran and failed for the known,
   pre-existing, producer-independent reason. The one-line tuple fix plus a
   dated ledger row remains owed to the orchestrator (`tools/**` is forbidden
   to both the implementer and the reviewer), exactly as W-1 verdict Finding
   4 recorded.
6. **Configuration note (session allowlist)** - the following were refused
   this hop with the verbatim catch-all text
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`:
   `python tools/ui_smoke.py` (direct), `python tools/bench_compare.py`
   (direct), `python -c "print('byte-probe')"` (byte-level ASCII proof),
   `git status --porcelain; ... Measure-Object ...`, `git ls-tree -r
   --name-only HEAD -- crates/ui/tests/ui_test`. Consequences and
   substitutions: (a) `ui_smoke` and `bench_compare` were re-run through the
   documented distilling entry point (`distill.py -- ...`), which the kickoff
   names as the fallback for exactly this allowlist case - both produced
   usable evidence (V9, V10); (b) the byte-level `python -c` ASCII proof was
   replaced by a non-ASCII sweep over `crates/ui` and the journal plus
   `encoding_guard --all` exit 0 (stronger for mojibake, and it covers every
   tracked file rather than only the new lines); (c) `git status --porcelain`
   was replaced by the read-only `git_git_status` tool (clean tree) and
   `git ls-tree` by the file listing (35 test modules, `main.rs` declaring
   the new one at line 11). Every gate this hop owes has evidence; nothing
   was skipped silently.

## Follow-up dispositions (recorded, never dropped)

- Deferred ledger row owed (unchanged from W-1): the bench_compare
  `DOCUMENTED_PHASES` one-line tuple fix, `tools/**`, operator/orchestrator
  duty (Finding 5).
- Optional, no owner assigned: if the orchestrator wants shape C3's 409 body
  literal to be byte-true, the error struct fix in Finding 1 is a W-2a or
  W-3a hop, not a W-2b rework.
- Relay to W-3b's author: the two 503 shapes noted in Finding 2 (bodied on
  the two new routes, body-less on every pre-existing route) - the wizard's
  fetch layer must not assume a body exists on the existing 503s.
- Relay to the next kickoff: shape C2's `epoch()` doc comment and the
  `dto.rs:182-184` citation erratum already recorded in W-1 verdict Finding 3
  both remain open (Finding 4).

## Duty 1 for the next implementer hop

The orchestrator should route exactly this command to the next implementer
hop to land this verdict and its execution-results (both files are in the
reviewer's edit grant; the landing commit and rider already landed and are
not in this scope):

```
python tools/commit_scoped.py --scope tasks/ui-extraction-wizard-w2b-verdict.md --scope tasks/ui-extraction-wizard-w2b-execution-results.md -m "docs(tasks): W-2b gate APPROVED (reader trait + wizard DTOs + two GET routes, 503/404/409) + execution-results"
```
