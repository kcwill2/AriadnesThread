# AR-01 Execution Results - reviewer acceptance pass

Reviewer re-runs against `tasks/ar-01-handoff.md` acceptance criteria and the
audit-remediation plan's workspace gates, 2026-10-09, worktree
`shining-enthusiasm` at HEAD `932041a` (AR-01 landed at `6d7f96b`, 4 commits
back), including uncommitted WIP of the cancelled AR-02 session.
Verdict: `tasks/ar-01-verdict.md` (APPROVED).

## Per-criterion results (canonical lines verbatim)

| # | Criterion (handoff / plan) | Result | Evidence line (verbatim) |
|---|----------------------------|--------|--------------------------|
| 1 | scoped ui suite green (`testrun.py --distill -- -p ariadne-ui`) | PASS | `245 passed / 0 failed / 0 ignored / 3 suites` (journal's 242 correct for 6d7f96b; +3 = uncommitted AR-02 CORS tests in shared worktree; all 8 `source_containment::` locks individually `ok` in capture) |
| 2 | `ui_smoke` PASS (via distiller) | PASS | `ui_smoke: PASS` (`### generic PASS (exit 0)`, capture `target/distill-capture.txt`) |
| 3 | encoding gate (`encoding_guard.py`, handoff `--staged`, reviewer ran `--all`) | PASS | no output; source contract `return 1 if violations else 0` -> exit 0 with zero violations |
| 4 | hygiene gate ("exit 0" per handoff; plan census says FAIL exit 1 pre-known) | PASS on zero-new-violations; gate itself exit 1 | `crates/ui/src/server.rs: R2 1155 lines > 1000` and `crates/ui/src/server.rs:799: R1 fn \`api_graph_traversal\` is 161 lines (> 100)` - exactly the two pre-existing AR-13-owned entries; zero new R1/R2/R3 entries attributable to 6d7f96b (baseline identity verified by full-diff analysis + plan census + arithmetic; a literal run at the parent revision is impossible under read-only git - `hygiene_check.py` has no `--rev`, reviewer checkout denied. NOT RUN as literally instructed; equivalent performed and documented in verdict section 5) |
| 5 | TDD red-first evidence | PASS (as far as read-only permits) | RED `237 passed / 5 failed / 0 ignored / 2 suites`; GREEN `242 passed / 0 failed / 0 ignored / 3 suites`; 5 failed = the 5 of 8 new tests that provably must fail pre-fix; 2-vs-3 suites reconciles via `--tests` passthrough (doc-tests excluded). RED itself not reproducible by a read-only reviewer - NOT INDEPENDENTLY RE-RUN |
| 6 | commit-vs-touch-set EXACT | PASS | `git diff --name-only 6d7f96b~1..6d7f96b` = 6 paths, all inside touch-set union journal-declared deviation files (plan: "(or new mod)") |
| 7 | S1 containment closes `..` / absolute / `local_prefix` escape; contained reads byte-identical | PASS | rejecting, mapping, and byte-identity tests all `ok` in reviewer's fresh run; code audit in verdict section 3 |
| 8 | S3 `<`/U+2028/U+2029 escaped; clean payloads byte-identical | PASS | `test_export_embeds_json_without_raw_script_breakout_chars ... ok`, `test_export_leaves_clean_json_byte_identical ... ok`; single-pass totality independently verified |
| 9 | S4-partial helper ships + wired | PASS | `test_hotspots_asset_ships_attribute_safe_escaper ... ok`; both `data-preview-*` sites switched |

## Workspace acceptance gates (reviewer role, no implementer carries these)

- `python tools/testrun.py --baseline` -> NOT RUN (not runnable: `--baseline`
  requires a FILE argument; no testrun baseline file exists in the tree).
- Full workspace census `python tools/testrun.py --distill` -> FAILED TO BUILD
  (exit 2): `parse format drift: no test result line matched the pinned parse
  spec`. Cause: uncommitted cancelled-AR-02 WIP - `+    expose_opt:
  Option<String>` added to `run_ui` (dirty hunk against HEAD) while committed
  callers pass 6 args: `error[E0061]: this function takes 7 arguments but 6
  arguments were supplied` in `m17_dir_nav_test.rs:11`,
  `m16_traversal_test.rs:17` and `:158`;
  `error: could not compile ariadne-integration-tests`. Commit 6d7f96b touches
  no cli/integration file (proven by commit file list) - not an AR-01 defect.
- Fallback census `python tools/testrun.py --distill -- --workspace --exclude
  ariadne-integration-tests` -> PASS: `1169 passed / 0 failed / 13 ignored /
  58 suites` (13 ignored matches plan anchor; 91-58=33 suites belong to the
  excluded, WIP-broken integration package).
- `python -m unittest discover -s tools -p "*_tests.py"` -> PASS:
  `Ran 219 tests ... OK` (exact plan anchor match).
- `bench_compare`: NOT RUN - not required by this handoff (no perf claim);
  plan schedules bench gates from Wave 2.
- `ui_smoke`: run (row 2 above).

## Explicitly not run / not runnable (statement)

1. `testrun.py --baseline` as written: no baseline file exists (see above).
2. `hygiene_check.py` literally at `6d7f96b~1`: impossible under reviewer
   read-only git (no checkout); parent-equivalent violation-set proof done
   instead (verdict section 5).
3. RED reproduction (needs reverting the fix - reviewer cannot mutate).
4. Full workspace census including `ariadne-integration-tests`: broken by
   foreign uncommitted WIP, quoted verbatim above.
5. Bash probes refused by the global catch-all
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`:
   `python -c` temp extraction, `echo $LASTEXITCODE` compound, `git status
   --short`, `git grep`. No expected gate lost to these denials; substitutions
   documented where used.

## Anchor comparison

Plan census anchor `1280 / 0 / 13 / 91` could not be re-measured intact
because of the foreign WIP build break. Comparable evidence: fallback census
`1169 / 0 / 13 / 58` (everything but the 33 integration suites) fully green,
including AR-01's +8 and AR-02-WIP's +3 tests; tools suites at anchor `219 OK`.
A clean `--workspace` census must be re-run by the orchestrator once the
AR-02 WIP is completed or reverted, per the AR-02/census finding in the verdict.
