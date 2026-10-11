# AR-16 Verdict - Worker dispatch contract tests (T3)

**APPROVED**

Range reviewed: `70e1699..a48c018` (commits `ea4c576` tests, `a48c018`
journal hash record). Base of AR-16 session was `602c176`; `70e1699` is a
foreign graph-package commit that landed between session start and this
handoff's first commit - not part of AR-16's scope, excluded from judgment.

## Verification (numbered, all measured fresh)

1. **Touch-set EXACT.** `git log --oneline 70e1699..a48c018` returns exactly
   the two claimed commits. `git diff --name-only 70e1699..a48c018` returns
   exactly `crates/scheduler/src/worker.rs` and
   `tasks/ar-16-handoff-journal.md` - both declared. `git show --stat`
   confirms `ea4c576`: worker.rs 547+/0-, journal 155+/0-; `a48c018`:
   journal only, 11+/5-. No foreign file in either commit.
2. **Production bytes untouched.** The `ea4c576` worker.rs diff is ONE hunk,
   `@@ -106,3 +106,550 @@`: three context lines (the pre-existing trailing
   comment, lines 106-108), then 547 `+` lines, zero `-` lines. The addition
   opens with `#[cfg(test)]` + `mod dispatch_contract_tests {`. File goes
   108 -> 655 lines; every pre-M18 production line is byte-intact.
3. **Missing-language lock locks, does not redefine.** Test 1 uses the real
   `LanguageRegistry::new().register(Box::new(ariadne_lang_cpp::CppPlugin::new()))`
   plus a positive `.cpp` control. Its assertions -
   `contains("unhandled language")`, `contains("solver.f90")`,
   `!contains("Cpp")` - match the real integration locks verbatim:
   `tests/integration/tests/m24_language_identity_test.rs:379` (contains
   "unhandled language") and `:383` (never "Cpp"), and
   `m18_cpp_plugin_parity/worker_backend_parity.rs:57` (same phrase),
   against production `worker.rs:84` `unhandled language for {path:?}`.
4. **Unknown-task lock exercises real dispatch.** `SelfRenamingIdPlugin`
   (unstable `id()`: claims `mocklang` once, then `mocklang-retired`) is fed
   through production `PluginWorkerBackend::new(&registry).analyze_task` -
   the mock is the plugin, not dispatch. Locks production `worker.rs:88`
   text `no plugin registered for language` plus zero analyzer calls (no
   silent fall-through). The journal's unreachability rationale is honest:
   `detect_language` derives the id FROM the claiming plugin, so
   `plugin_for_id` cannot miss with a consistent registry - the renaming-id
   mock is the only observable shape of that arm, and it pins the arm's
   contract, not a real plugin's behavior (documented as such in-module).
5. **Cancel lock asserts both branches.** Body-only task analyzes exactly
   `[root]`; the same task with a store context analyzes exactly
   `[root, header]` in closure order with the analysis stamped by the unit's
   file id - disjoint observable analyzer-call sets separating the body-only
   branch from the plugin-closure merge arm (`worker.rs:89-95`,
   `analyze_tu_full_with_plugin`).
6. **Mutation never landed; mutation-proof is credible.** Committed
   `worker.rs:84` carries the ORIGINAL wording `unhandled language for`
   (clean-checkout read; census confirms scheduler package clean vs HEAD);
   zero occurrences of the mutation string `dispatch refused` anywhere in
   the file. Both named wording locks assert
   `contains("unhandled language")` directly from the committed diff, so the
   logged mutation would necessarily turn exactly those two red - consistent
   with the journal's quoted 2-failure digest.
7. **Fresh gates pass** - verbatim lines in `tasks/ar-16-execution-results.md`.
8. **Hygiene rows attributed.** `hygiene_check.py` exits 1 with three R1/R2
   rows, none from this handoff: `server.rs` R2 1172 (= committed 1155 + 17
   net lines of foreign uncommitted `crates/ui` WIP per checkpoint census
   +26/-9), `server.rs:816` R1 161 (pre-existing, AR-13-owned, matches AR-01/
   AR-05 records), and `m15_graph_ui_test.rs:9` R1 104 (foreign uncommitted
   integration WIP; committed function is 99 lines = 104 minus the 5 net
   in-function additions visible in the uncommitted diff, i.e. under the cap
   at commit time). ZERO worker.rs rows of any kind, as the journal claimed.
9. **Report evidence.** Journal is digest-shaped: canonical summary lines,
   exit codes, capture paths (`target/testrun-capture.txt`), gates routed
   through `testrun.py --distill` / `distill.py`; no raw dumps.

## Declared deviations - dispositions

- **D1 (group-name mapping: unknown-task -> `plugin_for_id` arm; cancel ->
  store-context arm): ACCEPTED.** `worker.rs` contains exactly one dispatch
  (`analyze_task`) with exactly three arms, which the three handoff items
  map onto; there is no other "unknown-task" or "cancel" concept in the file.
  The alternative reading (pool break/fatal) is out of touch-set AND already
  characterized in `pool.rs` (verified: `test_worker_pool_failure_recovery`,
  `test_worker_pool_max_retries_exceeded`, fatal-retry-budget test).
- **D2 (all four locks green-first, justified as characterization):
  ACCEPTED.** These lock pre-existing behavior, so no red phase exists by
  construction; non-vacuity is evidenced for the finding-named contract by
  the logged mutation (2 named locks red, reverted byte-safe), and tests 3/4
  are non-vacuous by structure (they assert disjoint behaviors of distinct
  production arms).
- **D3 (`encoding_guard --staged` vacuous post-commit, `--all` as the
  substantive form): ACCEPTED** - disclosed, and `--all` verified exit 0
  over the committed content by this review.

## Not run (explicit)

- `python tools/testrun.py --baseline`: not run. The review-gate procedure
  assigns workspace-level milestone gates to the orchestrator, and four
  foreign lanes currently hold uncommitted WIP in `crates/graph`,
  `crates/ui`, `crates/pipeline`, `tests/integration` - a baseline now would
  measure their in-flight state, not this handoff. Per AGENTS.md sizing
  rule, workspace-wide final gates run only in a fresh/clean session or the
  orchestrator's bounded own run.
- `python tools/ui_smoke.py`, `python tools/bench_compare.py`: not run; not
  required by this handoff (test-only change, zero production/runtime bytes
  touched).
- Refused command forms (recorded verbatim where they gate a check): any
  piped command (`git show ... | Select-Object`, `git show > file`,
  `python ... 2>&1 | Select-Object`) and inline `python -c` were refused
  with `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  Substitutes were byte-safe (plain `git show`, Read-tool checkout reads,
  committed-diff arithmetic); none weakened a check.

## Follow-up dispositions

- The `m15_graph_ui_test.rs` R1 row (104 > 100) belongs to the active
  integration-lane WIP, not AR-16; it must be cleared (or recorded) by that
  lane before its own handoff, else it will join the AR-13 server.rs rows at
  milestone closeout. Routed to orchestrator attention via this verdict; no
  ledger entry is due from AR-16.
