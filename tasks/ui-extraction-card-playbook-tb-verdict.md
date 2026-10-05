# Verdict - ui-extraction-card-playbook T-B: APPROVED

Commit under review: `6a48a85` (full `6a48a859e35a48efd9e8ff68a040b47746eee405`),
parent `af11113` (`cpp-brace-depth-underflow` review-fix landing, rode in
pre-boot; file-disjoint). First non-assets Rust hand of the milestone: ALL
workspace gates deferred from T-A's hop were run here and all pass. The
workspace-red window declared in the T-A verdict is CLOSED.

## Gate results (fresh, this hop, verbatim)

| # | Gate | Result |
|---|---|---|
| 1 | `testrun.py --distill -p ariadne-cli` | `101 passed / 0 failed / 1 ignored / 5 suites` exit 0 -- reproduces implementer Green verbatim |
| 2 | `testrun.py --distill -p ariadne-ui` | `200 passed / 0 failed / 0 ignored / 3 suites` exit 0 |
| 3 | `testrun.py --distill -p ariadne-integration-tests --test m00_facts_path_parity_test` | `2 passed / 0 failed / 0 ignored / 1 suites` exit 0, test file byte-identical in diff |
| 4 | census `testrun.py --distill` (unscoped, NORMAL target dir, no lock) | `1204 passed / 0 failed / 4 ignored / 85 suites` exit 0 |
| 5 | `distill.py -- python tools/ui_smoke.py` | `boot+route checks clean (cold fixture, attempt 1, 1.0s)` / `ui_smoke: PASS` exit 0 |
| 6 | `distill.py -- python tools/bench_compare.py` | attempt 1 FAIL, attempt 2 (protocol max-1 retry) PASS: `bench compare: 3 run(s), no regressions past 1.25x baseline` exit 0 |
| 7 | `hygiene_check.py` | R3 warn-only legacy rows (same set shifted +46), zero from T-B lines; exit 0 |
| 8 | `encoding_guard.py --all` | no output, exit 0 |

## Census reconciliation arithmetic (per the deferred-reconcile note)

Anchor: `1195 passed / 0 failed / 4 ignored / 85 suites` recorded at
`44dfb73` in `tasks/cpp-brace-depth-underflow-execution-results.md`.
Delta chain: +3 T-A (`df1e21e`, machine-proven by scoped ui 197->200 at the
T-A hop) +1 rider (`af11113` adds exactly one `#[test]` in
`raw_string_tests.rs`; diff read whole) +5 T-B (the five `target_files_tests`
functions) = **1204**. Ignored held at 4 (the `1 ignored` visible in the
scoped cli digest is one of the four). Suites held at 85 (the new test module
is a `cfg(test) mod` inside an existing binary). Prediction and fresh census
agree exactly: `1204 / 0 / 4 / 85`.

## Bench adjudication (shape risk 8, REQUIRED this hop)

Attempt 1: exit 1, single breach `WARN OptimizationPlanner::plan()
facts+closures: 1918.8 ms vs baseline 1451.0 ms (+32%)`. Mechanically
unattributable to T-B: `crates/cluster/**` is frozen-empty in the diff, the
measured span is the planner, while T-B's O(symbols) grouping runs AFTER the
planner in the DTO mapping; the identical signature (same phase, +32%)
occurred run-1 at the brace-milestone hop (1908.3 ms) whose retry passed, and
whose verdict recorded a baseline-freshness note. Protocol retry (max one):
PASS exit 0, `plan()` 1667.6 ms = 1.15x, boot-adjacent phases 1.08-1.15x --
the grouping overhead the risk row anticipated stays inside the gate. No
baseline touched.

## Commit-vs-touch-set and frozen regions

`git diff --name-only af11113..6a48a85` = `crates/cli/src/commands/ui.rs`,
`crates/cli/src/commands/ui/target_files_tests.rs` (NEW),
`tasks/ui-extraction-card-playbook-tb-journal.md`. Exactly the touch-set;
single commit in range. Frozen-empty diff PROVEN (explicit path-form
commands): all of `crates/cluster`, `crates/lang-cpp`, `crates/ui`, `tools`,
`tests` (incl. `m00_facts_path_parity_test.rs` and goldens),
`crates/cli/src/lib.rs`, `commands/{plan,transform}.rs`, `optimize.rs`,
`commands/progress.rs` (the boot-progress oracle), and every PRE-EXISTING
`commands/ui/*` test module. The ui.rs diff is confined to the mapping region
(helper inserted before `extraction_proposal_dto`, the two-field fill) plus
ONE `#[cfg(test)] mod target_files_tests;` after `boot_progress_tests` -- no
import-block change, boot sequence untouched.

## Feed audit (P1/P11 literals)

`target_file_groups`: BTreeMap path-asc; per-group `symbol_id` sort mirroring
the existing `:509` sort; names capped at `EXTRACTION_PROPOSAL_MAX_NAMES`
with uncapped `symbol_count`; rows capped at
`EXTRACTION_PROPOSAL_MAX_TARGET_FILES` with `truncated = groups.len() > CAP`
(iff, both directions tested); `path` lossy-rendered. Pure: no planner, no
I/O. All 11 pre-existing fields untouched in the literal (diff-verified);
`test_existing_fields_unchanged_by_the_fill` pins every one of them. Helper
33 lines, dto ~40, ui.rs lands 839/1000. New file pure ASCII -- T-A
carry-over ruling honored (zero U+2014 in any T-B line; the 10 em dashes in
ui.rs are all pre-existing lines outside the hunks).

## Deviation dispositions

1. D1 (Red scaffold `vec![]`/`false` fill): ACCEPTED. After T-A left the
   exhaustive literal uncompilable (the declared chain cost), the scaffold
   was the ONLY route to the handoff's mandated "compiles + fails" Red;
   Green replaced it in the same region and the final diff carries no trace
   (verified by reading the full diff). Single-commit rule honored; the
   failing list (4 grouping/cap tests, semantic) matches the prediction and
   the 5th test passing pre-change is the expected P11 behavior.
2. D2 (fully-qualified `std::collections::BTreeMap` etc.): ACCEPTED. Matches
   the file's existing style (`std::path::Path` in `proposal_basename`) and
   actively serves acceptance 2 by keeping the import block out of the diff.

## Journal Red evidence

`97 passed / 4 failed / 0 ignored / 1 suites` exit 1 is journal-attested
(historical state). Consistency holds: 97+4 = 101, and Green totals 101+1
ignored; the 1-vs-5 suite count is the parser counting only the failing
suite (journal observation, same artifact as story-milestone D4). Not
reproducible post-Green.

## Findings

1. (minor, operational -- not against the commit) Third consecutive hop where
   `bench_compare` run-1 breached ONLY `plan()` at +32% and a retry passed
   (story hop: uniform multi-phase; brace hop: `plan()` +32%, retry PASS;
   this hop: `plan()` +32%, retry PASS). The per-machine baseline
   (`plan()` 1451.0 ms) is demonstrably stale against this machine's current
   noise floor. Recommend an operator `bench_compare.py --update-baseline`
   run on a verified-idle machine, or a dated baseline-refresh ledger row.
   No baseline was touched by this hop.

## Gate coverage / T-C

Everything runnable at this hop was run: scoped cli + ui, m00 parity,
workspace census (normal target, no lock workaround needed), ui_smoke,
bench (PASS via protocol retry), hygiene, encoding. Nothing deferred remains
except milestone-closeout duties (tools discover, ledger/memory sync -- the
tools suites were `Ran 188 ... OK` at the story hop and `tools/**` is
frozen-empty here). T-A's deferred census/smoke/bench are all hereby
discharged.

**T-C MAY START.** Its renderer consumes feed proven green by the P1 oracles;
T-C's own hop re-runs scoped gates + smoke against the playbook render and
the closeout takes the final census.
