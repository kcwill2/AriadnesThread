# AR-13 Handoff Journal - server.rs split + api_graph_traversal decomposition

Session: implementer, branch `shining-enthusiasm`, base HEAD `7adbeca`.
Commits: `aaffde0` (verbatim move) + `14e21b1` (decomposition).

## Objective recap

Split `crates/ui/src/server.rs` (1172 lines at re-anchor) into route-family
modules and decompose `api_graph_traversal` (161 lines, the repo's last
blocking R1 row). This handoff ENDS the standing "2 server.rs rows" hygiene
baseline: after the landing, `hygiene_check.py` exits 0 with ZERO blocking
rows. New standing baseline: ZERO blocking rows; R3 warnings are the only
remaining reported class.

## Module map (post-split sizes)

| File | Lines | Contents |
| --- | --- | --- |
| `crates/ui/src/server.rs` | 78 | shell: `is_same_origin` (CORS), `build_router` (route table byte-identical), pub re-exports |
| `crates/ui/src/state.rs` | 114 | `read_state`, `write_state`, `AnalysisGeneration`, `UiState` + impl |
| `crates/ui/src/routes/mod.rs` | 6 | `pub(crate) mod` declarations for the three route families |
| `crates/ui/src/routes/api_payloads.rs` | 116 | `api_index`, `ui_asset`, 11 asset handlers, `StatusResponse`/`api_status`, `api_export` |
| `crates/ui/src/routes/api_source.rs` | 87 | `SourceQuery`/`api_source`, `RepoInfoResponse`/`api_repo_info` |
| `crates/ui/src/routes/api_graph.rs` | 878 | all `/api/graph/*` handlers, `filter_elements`, search helpers, collateral, traversal + subtree helpers, `api_justification` |

`lib.rs` gained the two declared mod lines (`pub(crate) mod routes;`,
`pub(crate) mod state;`); `pub use server::*;` untouched, so the shell's
re-exports keep the crate surface intact.

Largest new file 878 lines (< 1000); every function < 100 lines.
Moved-line count ~1063 matched code lines - inside the sizing guard; no
re-split (T-a/T-b) needed.

## Verbatim-move statement + evidence

- `git show aaffde0 --stat -M --find-copies-harder` reports the rename
  detection: `crates/ui/src/{server.rs => routes/api_graph.rs}` - the
  single largest family moved as a detected copy.
- Byte-identity audit (temp script `ar13_move_audit.py`, compares
  `aaffde0^:crates/ui/src/server.rs` vs the six new/shell files at
  `aaffde0`, comparing stripped non-blank lines with `pub(crate)` visibility
  normalized): 1069 old code lines, 1063 matched byte-identically; the 6
  old-only lines are ALL fragments of the old top-level `use` tree
  (`use axum::{`, `routing::get,`, `extract::{Query, State},`,
  `Json, Router,`, and the two dto import lines) now re-expressed as
  per-module import headers. Zero body/comment/route-table drift.
- Route table: `build_router` body byte-identical to pre-split (ui_smoke
  authority: PASS below).

## Declared amendments (complete list; no semantic edits)

1. Per-module `use` headers replace the old single import tree (mechanical
   re-expression; the already-unused `ariadne_ui::dto::TraversalQuery`
   import - present at old line 4 with zero uses in the file - was dropped).
2. Visibility uplift to `pub(crate)` only (pub surface unchanged):
   `read_state`, `write_state`; private handlers now reachable from the
   shell (`api_index`, `asset_*` x11, `api_status`, `api_export`,
   `api_source`, `api_repo_info`, `api_graph_include/symbol/call/traverse/
   path/entrypoints/search`); query structs `SourceQuery` and
   `EntrypointsQuery` raised to `pub(crate)` (Rust requires argument
   visibility to match their `pub(crate)` handlers).
3. Shell `pub use` re-exports preserve every previously-public path:
   `AnalysisGeneration`, `UiState`, `build_router` (stays defined here),
   `StatusResponse`, `RepoInfoResponse`, `CollateralQuery`,
   `TraversalApiQuery`, `JustificationQuery`, `api_graph_collateral`,
   `api_graph_traversal`, `api_justification`.
4. `lib.rs`: the two mod lines named in the touch-set.

Test files: ZERO edits. The only server-qualified test references
(`ariadne_ui::server::StatusResponse`, `server_api.rs:71/128`) resolve
through the shell re-export unchanged.

## api_graph_traversal decomposition (commit 14e21b1)

Former 161-line body replaced by handler + 5 functions (all < 70 lines),
same check order and status codes; m15/m16 traversal characterization locks
UNEDITED and green:

- `resolve_traversal_file_pair(&TraversalApiQuery) ->
  Result<(FileId, FileId), StatusCode>` - seed parse, 400 on unparsable.
- `ensure_traversal_edge(&AnalysisGeneration, FileId, FileId) ->
  Result<(), StatusCode>` - 404 missing file, 400 non-direct dep.
- `empty_traversal_response(FileId, FileId) -> TraversalResponse` - the two
  shared empty payloads.
- `resolve_traversal_symbols(&str, &FileJustificationResponse, FileId) ->
  Vec<SymbolId>` - explicit comma list wins; else justified symbols of the
  root->dep edge.
- `project_traversal_subtrees(Option<ExtractionImpact>,
  &AnalysisGeneration, FileId) -> (TraversalSubtree, TraversalSubtree,
  TraversalSubtree, TraversalSummary)` - closure computation + projection
  (necessary / actual / collateral + summary; None -> all-empty).

## Pub-surface lock (grep before/after)

- BEFORE (`git grep -n "pub " -- crates/ui/src/server.rs` at 7adbeca): 39
  pub declarations (items + pub fields). Capture replayed this session;
  set recorded in `target/distill-capture.txt` (39 lines).
- AFTER (same grep over `server.rs`, `state.rs`, `routes/`): 39 item
  declarations, 1:1 name-identical across the four extraction files + the
  shell `build_router`; plus 5 new shell lines (1 doc + 4 `pub use`) that
  are re-export wiring, not surface. Identical set.

## Gates (canonical digest lines, own stdout)

1. `-p ariadne-ui`: baseline (pre-change) AND after move AND after
   decomposition all report:
   `265 passed / 0 failed / 0 ignored / 3 suites` / `no failures`
   (capture: `target/testrun-capture.txt`).
2. `python tools/distill.py -- python tools/ui_smoke.py`:
   `boot+route checks clean (cold fixture, attempt 1, 0.7s)` /
   `ui_smoke: PASS` (capture: `target/distill-capture.txt`).
3. `python tools/hygiene_check.py`:
   - BEFORE (7adbeca tree):
     `crates/ui/src/server.rs: R2 1172 lines > 1000`
     `crates/ui/src/server.rs:816: R1 fn `api_graph_traversal` is 161 lines (> 100)`
     plus R3 warns - 2 blocking rows (the standing baseline).
   - AFTER (14e21b1 tree): exit 0, ZERO blocking rows. Quoted verbatim:
     the report contains NO R1/R2 lines; every reported row is
     `R3 (warn)` (178 lines, capture: `target/distill-capture.txt`).
     The former R3 server.rs rows now attach to `routes/api_graph.rs`
     with identical identifiers (verbatim move). NEW STANDING BASELINE:
     zero blocking rows; downstream contracts must not re-baseline
     R1/R2 rows as tolerated.
4. `python tools/encoding_guard.py --all`: silent, exit 0.
   `--staged` note: `commit_scoped.py` stages+commits atomically, so the
   substantive content gate here is `--all` (exit 0) over the committed
   tree; `--staged` after commit reports a clean index.
5. `python tools/distill.py -- cargo check --workspace` green at every
   intermediate state (strict compile-green for the shared worktree;
   `ok (exit 0, 0 warnings emitted)` - repeated digests replay cached
   warnings).

## Inherited (not introduced, not fixed - verbatim discipline)

- `unused variable: gen` in `build_subtree_nodes` (old server.rs:984 ->
  routes/api_graph.rs): the parameter was already unused pre-split.
- R3 single-letter-local warnings moved with their code (server.rs ->
  routes/api_graph.rs), count unchanged.

## Deviations (each with reason)

- The handoff names one commit message; landed TWO scoped commits
  (`aaffde0` pure moves, `14e21b1` decomposition with the named
  `(Q1/Q2/Q3)` tag) so `git show` diff-move detection works on the pure
  move and every intermediate state stays compile-green.
- Dropped the pre-existing unused `TraversalQuery` import (would otherwise
  re-declare a dead import in every new module header).

## Not done (deliberately)

- No edits under `crates/ui/assets/**` (AR-11 lock; zero touched).
- No golden fixtures, theme baselines, or test assertions touched.
- No rename of inherited single-letter locals (R3 warn-only; verbatim).
