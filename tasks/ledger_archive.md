# Milestone Ledger Archive

Verbatim rows discharged from `tasks/milestone_ledger.md` per its archive
rule (worktree-census rows whose disposition was carried out; deferred
findings executed or dropped). Live ledger stays lean; this file is the
audit trail. Discharge evidence lives in the referenced milestone rows /
gates.

## Discharged worktree-census rows

| Worktree | State at 2026-10-01 | Disposition |
|---|---|---|

## Discharged deferred findings

| Finding | Target | Source |
|---|---|---|
| `testrun.parse_cargo_output` matches unstripped lines; colorized cargo -> false `parse format drift`; fix = ANSI-strip at raw parser entry, red-gated | next tooling milestone | tool-distill A gate |
| Optional test hardening: distill-mode baseline-load-error direct assertion (header `(exit 2)` + rc 2 together); class currently covered by shared-mechanism + adjacent pins | next tooling milestone | tool-distill B gate |
| Stale UI/tool process holds `target\debug\tool.exe` (`error: failed to remove file ... Access is denied. (os error 5)`): unscoped census and bench attempts die against the normal target (testrun DRIFT exit 2 / cargo bench exit 101); clean census `1181 passed / 0 failed / 4 ignored / 85 suites` was obtained with `CARGO_TARGET_DIR` redirected to a temp dir (code-equivalent). Close the stale process before the next normal-target build, `bench_compare`, or `--baseline` run | operator environment cleanup, before next bench/census | found during ui-extraction-card-story T1 reviewer hop (verdict finding 2) |

The first two rows above executed by handoff C (`tasks/tool-distill-c-handoff.md`):
`f3dc53b` (Red-gated ANSI-strip at parser entry; drift never hidden),
`c600fb8` (exit-honesty pin); review-gate APPROVED zero findings at
`c600fb8`; suites `Ran 30 tests ... OK` / discover `Ran 184 tests ... OK`.

The tool.exe row discharged 2026-10-04 at the ui-extraction-card-playbook
closeout: T-B's reviewer hop ran the unscoped census against the NORMAL
target dir with the verdict noting "no lock" (`1204 passed / 0 failed / 4
ignored / 85 suites` exit 0), and T-C's hop repeated clean normal-target
census (`1213 ...`) + `ui_smoke` (0.7s boot) + bench attempts - the os-error-5
signature never recurred, so the stale process was closed before the T-B hop.
The bench FAILs at those hops are the stale-baseline machine-drift class,
tracked by the dated escalation row, not this lock.
