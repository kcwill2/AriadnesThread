# AR-26a2 Verdict - APPROVED

Handoff: `tasks/ar-26a2-handoff.md` (as amended `69ab3f7`). Commits:
`d32c3fa` (journal bootstrap) -> `0d348ee` (landing) -> `8eec456`
(addendum). Frozen-lock base `4794467`. Chain: hop 2/3 of AR-26
(hop 1 verdict `tasks/ar-26a-verdict.md`). Full evidence:
`tasks/ar-26a2-execution-results.md`.

**APPROVED.** AR-26b may dispatch.

## Central adjudications (explicit verdicts)

1. **VERBATIM AT SCALE - VERIFIED.** `git show --name-status -M 0d348ee`
   reproduces 11/11 R100. Stronger: `git diff --find-renames --raw -M
   4794467 0d348ee` shows every moved pair with IDENTICAL src/dst blob
   hashes - byte identity at hash level, file by file. The ONLY content
   changes in the entire moved-set region are the two pinned shim files
   (`crates/artifact/src/lib.rs` 9 lines, `crates/report/src/lib.rs`
   7 lines, both byte-matching the contract pins) and the ruled 1+1-
   manifests hunk. All 9 `use crate::manifests` sites present, one per
   file, at the new home, proven byte-unchanged by blob identity; the
   `crate::` rebinding (report shim -> same-crate `pub mod manifests`)
   is resolution-time only, as the contract's ruling text specifies.
2. **RIDER TRIO EXECUTED - VERIFIED, WINDOW CLOSED.** Current
   `crates/report/src/lib.rs` = exactly the pinned 7 lines: the rationale
   comment, `#[allow(hidden_glob_reexports, unused_imports)]`, and the
   `pub(crate) use ...::collect_stage_languages;` line are gone; the 12-DTO
   manifests pair and the manager pair stand. `crates/artifact/src/
   manifests.rs:22` reads `pub(crate) fn collect_stage_languages(`.
   Reviewer re-ran the G7 substituted grep: capture = 1 line, exactly the
   `pub(crate)` def, zero report-lib lines. Repo-wide symbol census: the
   fn exists ONLY at the def + the three same-crate consumer files
   (discovery_io, metrics_io, graph_io). Terminal public surface = 12 DTOs
   (flat + module-qualified) + manager exports; the 26a transient-widening
   window is closed. Contract-vs-ruling: the amended contract implements
   ruling board_12d48349 rider 3 exactly (definition revert, enumerated
   trio drop, DTO pair retention, machine-checkable gate 7 per verdict F1);
   the contract's "+ manager exports" surface extension is this hop's own
   mandated shim design, consistent with the ruling (which spoke to the
   manifests surface pre-manager-move). No divergence.
3. **CONSERVATION RECONCILED.** PASSED `40 + 7 = 47` holds in own runs.
   Suite attribution (own captures): artifact 2 = lib 40 + Doc-tests 0;
   report 4 = lib 0 + bincode 6 + extraction 1 + Doc-tests 0. Pre-hop pair
   was 4+2=6 suites @47; post-hop 2+4=6 suites @47 - targets relocated,
   none lost. Gate 6 decomposition named: m00_facts_path_parity 2 +
   m00_golden_baseline 2 + m23_symbol_isolation 7. Census own run
   `1398 passed / 0 failed / 13 ignored / 105 suites` = exact anchor;
   suites moved 105 -> 105, NO drift (the empty report lib target persists
   as a suite, as the contract's census clause itself anticipated). The
   gate-3 header's projected "3 suites" contradicts the contract's own
   4-target enumeration; measured 4 with PASSED = 7. Reviewer-pass
   authority exercised under the contract's stated tolerance clause
   (authority = PASSED conservation): ACCEPTED; findings 1-2 record the
   contract-text defects for correction, not the land.
4. **G7 SUBSTITUTION - ACCEPTED-EQUIVALENT.** The pinned `|`-alternation
   form was refused in MY run with the same catch-all boilerplate,
   confirming the implementer's logged refusal was environmental, not
   selective. `rg -n -e a -e b -e c <same paths>` is the OR of the same
   literal terms over the same targets - semantically identical to
   `rg -n "a|b|c"`. Output matched the pinned expectation exactly (one
   `pub(crate)` def line, zero report lines). Refusal verbatim recorded in
   both journal and execution results.
5. **TOUCH-SET EXACT + D7 + MUTATION REVERT.** All three commits land
   inside the declared touch-set (d32c3fa/8eec456 journal-only; 0d348ee =
   17 named paths, Cargo.lock explicitly sanctioned). D7 independently
   re-proven: `git ls-files` refused in my run (allowlist), substituted
   `git show "HEAD:crates/report/src/manager.rs"` -> fatal not-in-HEAD +
   disk census `crates/report/src/**` -> only `lib.rs`; deletion+addition
   in the ONE commit, no stray follow-up. Frozen locks: tests +
   report/tests diff EMPTY vs 4794467; m17 EMPTY; golden porcelain empty.
   Mutation E0433 trail mechanically sound: `crates/scheduler/src/
   incremental.rs:23` really is `ariadne_report::ArtifactManager::...`;
   revert byte-complete - the committed report lib.rs blob equals the
   pinned shim and the addendum commit touches only the journal.
   `#[test]` recount: 47 both sides, per-file distribution intact; ZERO
   test edits. Cargo.toml additions = exactly the contract's dependency
   list, nothing extra.

## Deviation dispositions

- Declared deviations: NONE - and none found independently.
- G3 suite-count note (declared as note, not deviation): ACCEPTED per
  adjudication 3.
- G7 command-shape substitution: ACCEPTED-EQUIVALENT per adjudication 4.

## Findings

1. **(minor - contract text, planner-owned, not the implementer)**
   `tasks/ar-26a2-handoff.md:69` states the census anchor as
   `1398 passed / 0 failed / 0 ignored / 105 suites`; the AR-26a
   reviewer-measured anchor (and this contract's own line 104) is
   `13/ignored`. Measured truth: 13 ignored. Fix the anchor row at next
   touch (ledger/closeout candidate).
2. **(minor - contract text, planner-owned, not the implementer)**
   Gate-3 header pins `7 passed / 0 failed / 0 ignored / 3 suites` while
   the same sentence enumerates FOUR targets (bincode 6 + extraction 1 +
   empty lib 0 + Doc-tests). Measured 4. Correct the projection to 4 at
   next touch so hop-3-style reuse does not re-import the miscount.
3. **(informational - configuration, not the implementer)** Reviewer
   allowlist refusals in this run, all substituted with outputs, none a
   skipped gate: G7 pinned `|` form, `git ls-files`, one compound git
   chain, bare `rg` (distiller required), piped `unittest discover`.
   Refusal boilerplate quoted verbatim in the execution results.

## Acceptance pass - canonical lines (own stdout)

- Gate 1: `ok (exit 0, 31 warnings emitted)`; zero warning attribution to
  `ariadne-report`/`ariadne-artifact` in own capture.
- Gate 2: `40 passed / 0 failed / 0 ignored / 2 suites`
- Gate 3: `7 passed / 0 failed / 0 ignored / 4 suites`
- Gate 4: `26 passed / 0 failed / 0 ignored / 2 suites`
- Gate 5: `118 passed / 0 failed / 8 ignored / 7 suites`
- Gate 6: `11 passed / 0 failed / 0 ignored / 3 suites` + golden porcelain
  empty
- Gate 7 (substituted): `crates/artifact/src/manifests.rs:22:pub(crate) fn
  collect_stage_languages(` - sole line
- Hygiene: R3 (warn) rows only, ZERO blocking, no report/artifact rows
- Encoding: `encoding_guard.py --all` silent (exit 0)
- Tools: `Ran 219 tests in 21.079s` / `OK`
- CENSUS: `1398 passed / 0 failed / 13 ignored / 105 suites` - exact
  anchor, suites 105 -> 105, fully attributed (see reconciliation table)
- NOT run: `--baseline` (errata: `usage error: missing value for
  --baseline`, file-mode/no repo baseline), `ui_smoke`, `bench_compare`
  (out of contract; closeout duty).

## Follow-up dispositions (recorded, never dropped)

- Findings 1-2 to the orchestrator/planner for contract-text correction
  (or ledger deferred-findings row at closeout).
- Standing deferrals unchanged: report/Cargo.toml orphaned-deps cleanup
  (later AR owns it), m17/cone guard update (AR-26b owns it).
