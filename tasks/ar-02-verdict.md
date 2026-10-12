# AR-02 Verdict - Localhost default bind + same-origin CORS (S6 HIGH)

VERDICT: APPROVED - landing commit `b49ed06`, zero blockers. Numbered findings
1-3 are minor/informational; no rework on `b49ed06`.

Reviewer-fresh gate lines are quoted verbatim in
`tasks/ar-02-execution-results.md` (own-run stdout per the capture-race ruling).

## Ground truth and commit structure

- Review boot: HEAD `602c176`, branch `shining-enthusiasm`, dirty = only
  `crates/graph/src/graph.rs` (+25/-4, AR-06 in-flight WIP). During the review
  AR-06 landed that file as `70e1699`; the bytes every gate ran against are
  the bytes that commit carries, so AR-02 attribution is unaffected.
- Landing commit `b49ed06` (single commit). ACTUAL parent is `8ea8d0d`, not
  `3b960bc` as the review dispatch brief claimed: `932041a` (AR-04 journal),
  `8209a7e` (validate T1), and `8ea8d0d` (AR-07) landed mid-wave on the shared
  branch. The touch-set check is adjudicated against the real parent
  `8ea8d0d`. Not an implementer fault (finding 1).
- Per-commit review: `git show b49ed06` read whole, per file. `602c176` is
  AR-07's journal addendum, out of AR-02 scope.

## Commit-vs-touch-set: EXACT

`b49ed06` touches 13 paths. Every one is inside the handoff touch-set or a
declared deviation:

| Path | Bucket |
| --- | --- |
| crates/cli/src/lib.rs | touch-set |
| crates/cli/src/commands/ui.rs | touch-set |
| crates/cli/src/commands/ui/tests.rs | touch-set ("or inline" lock option) |
| crates/ui/src/server.rs | touch-set |
| tasks/ar-02-handoff-journal.md | touch-set |
| crates/ui/tests/ui_test/cors_same_origin.rs | deviation D2 (new mod) |
| crates/ui/tests/ui_test/main.rs (+1) | deviation D2 (mod registration) |
| tests/integration/tests/m15_graph_ui_test.rs | deviation D1 |
| tests/integration/tests/m16_traversal_test.rs | deviation D1 |
| tests/integration/tests/m17_dir_nav_test.rs | deviation D1 |
| tests/integration/tests/m18_persistence_incremental_test.rs | deviation D1 |
| tests/integration/tests/m21_run_ui_smoke_test.rs | deviation D1 |
| tests/integration/tests/m22_scala_load_analysis_test.rs | deviation D1 |

No path outside. `crates/graph/**` absent from the commit (AR-06 WIP neither
staged nor reverted); `report.json` CRLF churn not swept.

## Lock verification

- `crates/ui/tests/ui_test/server_api.rs` BYTE-IDENTICAL to pre-AR-02:
  blob `0c813f0c5c8121788f0a8513818ed91916235672` at `3b960bc`, `8ea8d0d`,
  and `b49ed06` (hash equality across revs is byte identity; the path-scoped
  diff is empty by construction). 942 lines, R2 breach reverted as claimed.
- Bind: `resolve_bind_addr(None | blank) -> "127.0.0.1"`; `run_ui` binds
  `format!("{bind_addr}:{port}")` only. `grep bind\(` in `crates/cli` finds
  exactly one production bind (ui.rs:801) plus the lock's own literal;
  `crates/ui/src` has zero binds (router only). No env-var or smoke-only bind
  bypass exists; `ui_smoke.py` launches the real binary argv
  `tool ui <repo> --no-open --no-watch --port N` (no `--expose`) and probes
  `http://127.0.0.1:<port>`, so the PASS proves the production default path.
- CORS: `CorsLayer::permissive()` GONE (remaining `permissive` matches are
  doc/comment text in the new test mod only). Replacement is
  `AllowOrigin::predicate(is_same_origin)`: grants only when Origin equals
  `http(s)://<request Host>` exactly (port-significant); foreign origins
  receive no `access-control-allow-origin` header at all.
- Help/warning: clap help for `--expose` names the 127.0.0.1 default, the
  no-auth LAN exposure and that "this warns"; `warn_if_expose_used` prints
  the warning to stderr on every boot that widens the bind. Locked by
  `test_cli_ui_help_documents_expose_lan_warning`.
- Source lock `test_run_ui_binds_resolved_addr_never_hardcoded_wildcard`
  forbids `0.0.0.0` in the production section and requires the resolved bind
  + warn path.

## Deviation dispositions

- D1 (6 integration files beyond the literal touch-set): ACCEPTED.
  `run_ui` gained a 7th parameter; the touch-set made breaking these builds
  inevitable the moment the contract's own constraint ("bind 127.0.0.1 by
  default") was implemented. Main's board INFO explicitly mandated "fix ALL
  run_ui call sites". Every hunk verified arg-only: a `None` seventh arg +
  comment at 8 call sites (m15x2/m16x2/m17/m18/m21/m22); m22 hunk sampled
  line-by-line - assertion semantics untouched. Full integration compile
  exit 0 + full workspace census exit 0 prove no site was missed.
- D2 (CORS locks in new mod `cors_same_origin.rs` + 1-line main.rs
  registration; server_api.rs restored byte-identical): ACCEPTED.
  The WIP pushed server_api.rs to exactly 1000 lines (R2 requires < 1000);
  AR-01 precedent `source_containment.rs` + `+1 main.rs` is the sanctioned
  pattern in this repo and was accepted by the AR-01 verdict. Byte identity
  independently hash-verified above.
- WIP reconciliation (aborted 1st dispatch completed in place for
  lib.rs/commands/ui.rs/ui tests.rs/server.rs): ACCEPTED. Committed content
  matches the journal per file, including the journal's own numstat claims
  (+78/-1, +26/-1, +40, +22/-2 pattern; server_api.rs revert verified).
- RED-by-revert methodology (2nd dispatch inherited complete fix+test WIP):
  ACCEPTED. The literal "test-before-fix" ordering was unreconstructable;
  the journal demonstrated RED by temporarily reverting each production fix.
  Independent non-vacuity check: under `CorsLayer::permissive()` tower-http
  answers `Some("*")`, so lock 1 (`== None`) fails, lock 2 fails, and lock 3
  fails because permissive answers `*` not the mirrored value - exactly
  `0 passed / 3 failed` with `left: Some("*")` x3 as journaled. The bind RED
  likewise matches the lock's own first assertion message. The tests
  genuinely fail on the vulnerable build; not vacuous.

## Findings (all minor; no rework)

1. Minor/attribution (review-brief, not code): the dispatch's "parent should
   be `3b960bc`" is wrong; real parent `8ea8d0d`. Adjudication used the real
   parent. Relay to the wave closeout so census reconciliation uses the right
   base. (`8ea8d0d` separately carries AR-07's disclosed glob-scope leak of
   AR-06's untracked test file - AR-07/AR-06 verdicts own that, not AR-02.)
2. Minor (defer candidate): with `--expose`, the boot banner still prints
   `loading at http://localhost:<port>` and `open::that` still opens the
   localhost URL (crates/cli/src/commands/ui.rs, end of `run_ui` else-branch).
   The LAN user sees an under-describing URL, though the stderr warning
   carries the bound address. Handoff did not mandate a change; suggest a
   `tasks/milestone_ledger.md` deferred-findings row (natural carrier: the
   AR-13 server.rs/CLI hygiene hop).
3. Informational (environment): `encoding_guard --all` ran silent (its
   contract: silence = zero violations = exit 0) and `hygiene_check` printed
   exactly the 2 inherited server.rs rows; raw exit codes could not be
   echoed because chained commands and `python -c` are off the bash
   allowlist (verbatim refusals recorded in execution-results). The hygiene
   exit-1-by-construction (`return 1 if violations`) from the 2 printed
   blocking rows matches the AR-13 deferred row `tasks/milestone_ledger.md:86`
   (same rule ids, same fn name; line drift 764->799, count drift 1120->1155
   = AR-01/AR-02 sanctioned growth). Zero NEW violations is the operative
   gate and it holds.

## Follow-ups for the orchestrator

- Finding 2 -> ledger deferred-findings row if the wave closeout agrees.
- No bench/theme gates required: no perf claim, zero styling diff.
