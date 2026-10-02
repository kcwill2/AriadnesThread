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

Both rows above executed by handoff C (`tasks/tool-distill-c-handoff.md`):
`f3dc53b` (Red-gated ANSI-strip at parser entry; drift never hidden),
`c600fb8` (exit-honesty pin); review-gate APPROVED zero findings at
`c600fb8`; suites `Ran 30 tests ... OK` / discover `Ran 184 tests ... OK`.
