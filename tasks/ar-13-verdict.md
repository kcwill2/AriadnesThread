# AR-13 Verdict - APPROVED

Handoff: server.rs split + api_graph_traversal decomposition (Q1/Q2/Q3).
Contract `tasks/ar-13-handoff.md`; journal `tasks/ar-13-handoff-journal.md`.
Range `7adbeca..768529f` (branch `shining-enthusiasm`), per-commit review
oldest-first: `aaffde0` (verbatim split), `14e21b1` (decomposition),
`768529f` (journal). Ground truth via `tools/checkpoint_guard.py status`:
HEAD `768529f`, zero tracked-dirty at session start.

## Verdict

APPROVED. All contract gates re-run fresh with own stdout (addendum
`tasks/ar-13-execution-results.md`). No undeclared semantic edit found.
Zero blocker findings. The acceptance transition is verified and the new
standing hygiene baseline is declared below and inherited by all
downstream hands.

## Commit-vs-touch-set

`git diff --name-only 7adbeca..HEAD` = 8 paths, all inside the hard
touch-set: `crates/ui/src/{lib.rs,server.rs,state.rs}`,
`crates/ui/src/routes/{mod.rs,api_payloads.rs,api_source.rs,api_graph.rs}`,
+ `tasks/ar-13-handoff-journal.md`. Zero test-file edits. `crates/ui/assets/**`
byte-untouched (AR-11 locked territory - no asset path in any commit).

## Per-commit judgments (oldest-first)

1. `aaffde0` - pure move. Stat with `-M --find-copies-harder` reports the
   copy pair `crates/ui/src/{server.rs => routes/api_graph.rs}` exactly as
   the journal cites. VERBATIM FIDELITY verified by full in-context
   comparison of every deleted server.rs line against the added lines of
   state.rs (114), routes/api_payloads.rs (116), routes/api_source.rs (87),
   routes/mod.rs (6), the shell (78), lib.rs (+2 mod lines) and the full
   826-line api_graph.rs as committed: every code and comment line
   matches, with visible delta limited to (a) per-module `use` headers,
   (b) module doc headers, (c) `pub(crate)` prefixes on formerly-private
   items, (d) the four `pub use` re-export statements plus the module doc.
   SECURITY SPOT CHECKS byte-intact: the AR-01 S1 containment block
   (canonicalize BOTH ends, `starts_with`, 403 FORBIDDEN / 404 NOT_FOUND /
   500 mapping, full S1 comment) appears verbatim in api_source.rs;
   `is_same_origin` and the `CorsLayer` route wiring are unchanged
   context lines in the shell diff; the `build_router` route table shows
   as unchanged context (zero +/- lines inside it) and ui_smoke is the
   independent route authority (PASS below).
   Declared exceptions: the 6 old-only lines are the 4 `use axum::{` tree
   fragments (`use axum::{`, `extract::{Query, State},`, `routing::get,`,
   `Json, Router,`) plus the 2 dto block lines that change content when
   `GraphExportPayload` (moved to payloads/state headers) and the unused
   `TraversalQuery` are redistributed; `};` terminators multiset-balance.
   All 6 genuinely import-syntax. Zero body/comment/route-table drift
   found.
2. `14e21b1` - decomposition only (single file, +112/-60). Check order and
   status codes preserved: parse 400 -> analysis 503 -> existence 404 ->
   direct-include 400 -> metric-Err 200-empty -> empty-seed 200-empty ->
   impact projection -> final payload; both early-return payloads are the
   exact deleted struct literals behind `empty_traversal_response`; the
   projection branches (incl. `saturating_sub(1)`, decoy terms) are moved
   byte-identically. Handler `api_graph_traversal` = api_graph.rs:473-531
   (48-49 non-blank lines; well under the 100 ceiling). 5 helpers with the
   journal's exact names; largest (`project_traversal_subtrees`) ~70
   lines.
3. `768529f` - journal only, matches the landed shape.

## Pub surface + route table

39 BEFORE = 39 AFTER, 1:1 name-identical (my own counts, not the journal's):
`^-\s*pub ` in the aaffde0 diff = 38 old declarations + the untouched
context `pub fn build_router` = 39; `^\s*pub ` over the five server-family
files at HEAD = 43 = 39 item declarations + 4 `pub use` re-export
statements. `ariadne_ui::server::StatusResponse` still resolves:
un-edited `crates/ui/tests/ui_test/server_api.rs:71/128` use it and pass
inside the 265 suite. Route table: ui_smoke PASS + context-identical
build_router body (double evidence).

## Unused-import proof (independent)

`TraversalQuery` occurs EXACTLY ONCE across the whole pre-split server.rs
blob (the import line itself, `-    TraversalQuery, TraversalResponse, ...`);
`TraversalApiQuery` is a distinct identifier (no substring overlap). It was
dead at HEAD; dropping it removes no surface (dto's own `TraversalQuery`
remains exported via the untouched `pub use dto::*;`).

## Traversal locks

m15/m16 UNEDITED (proven by the 8-path touch-set) and re-run green on my
own stdout: m16 `2 passed / 0 failed / 0 ignored / 1 suites`, m15
`1 passed / 0 failed / 0 ignored / 1 suites`.

## Declared deviation dispositions

- D1 named commit message split across `aaffde0`/`14e21b1` (contract
  prefers a single move commit "where feasible so git show diff-move
  detection works"): ACCEPTED on merits. The clause is purpose-bound;
  the split serves the purpose strictly better - the pure-move commit
  keeps the copy pair detected and amendment-free, the decomposition
  lands as a clean logic-only diff, and every intermediate state is
  compile-green (the contract's own STRICT COMPILE-GREEN constraint).
  Both messages carry halves of the handoff title, Q1/Q2/Q3 tagged on
  the decomposition.
- D2 dropped unused `TraversalQuery` import: ACCEPTED (proof above).
- Declared amendments (pub(crate) uplift, per-module `use` headers,
  `pub use` re-exports, 2 lib.rs mod lines): ACCEPTED - verified as the
  ONLY visible edits; no undeclared amendment exists.

## Findings (all minor; no rework required)

1. MINOR (journal wording): "39 item declarations + 5 new shell lines
   (1 doc + 4 pub use)" - the shell module-doc header is 5 lines and the
   4 `pub use` statements span 9 physical lines. Load-bearing claims
   (39<->39 items; 4 re-export statements) verify exactly. Fix wording
   at next touch.
2. MINOR (journal/board numbers): "handler(48)" counts non-blank lines
   (fn spans 473-531); "178 R3 lines" measured the pre-`79a90d3` tree -
   my run reports 179 (see finding 3). Neither is load-bearing; the
   repo baseline should pin "ZERO blocking rows", not an R3 line count.
3. MINOR (environment, not AR-13): the shared worktree moved mid-review -
   AR-20a landed as `79a90d3` (worker-core extraction, Cargo.toml member,
   scheduler reparent; staged deletion of `crates/scheduler/src/tu_store.rs`
   pending) after my ground-truth. `79a90d3` touches zero `crates/ui/**`
   paths, so every ui-scoped gate line remains valid verbatim; hygiene
   (R3-only rows from worker-core) and the census (+2 empty suites) saw
   the foreign material. Recorded for wave reconciliation; AR-20a needs
   its own adjudication.
4. CONFIG (this review's tooling, not the handoff): custom byte-level
   audit scripts were unrunnable - stdin-piped python and `python -c`
   hit the catch-all deny (verbatim texts in the addendum). Verbatim
   fidelity was instead adjudicated by full old-blob-vs-new-blobs diff
   comparison in context, which is complete, not sampled. Refused paths
   named per the attempt-every-gate duty; all contract gates ran.

## Follow-up dispositions (to ledger, not AR-13 debt)

- Inherited `unused variable: gen` in `build_subtree_nodes`
  (routes/api_graph.rs:690, formerly server.rs:984): disclosed, verbatim
  discipline honored; candidate ledger cleanup row.
- testrun.py cp1252 echo crash requires `PYTHONIOENCODING=utf-8`;
  `--no-fail-fast` must be passed after `--` to testrun (usage error
  otherwise): existing closeout tools-ticket family.

## NEW-BASELINE DECLARATION (binding on all downstream hands)

The standing "2 server.rs rows" baseline DIES with this handoff. New
standing hygiene baseline, verified fresh by the reviewer at HEAD
`768529f`:

  `python tools/hygiene_check.py` -> exit 0, ZERO blocking (R1/R2) rows;
  every reported row is `R3 (warn)` (179 reported lines at review time).

Before-state (confirmed): exactly 2 blocking rows -
`crates/ui/src/server.rs: R2 1172 lines > 1000` and
`crates/ui/src/server.rs:816: R1 fn `api_graph_traversal` is 161 lines
(> 100)`. Downstream contracts must NOT re-tolerate R1/R2 rows.

Gate lines, verdict evidence, not-run list, and refusal texts:
`tasks/ar-13-execution-results.md`.
