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
| M23.5 companion parity test (T3-B companion item) -- **HARD EXPIRY: must land before unit E.** T3-B's two-process export lock is APPROVED and its two-sample residual was explicitly ACCEPTED (do not re-open it), but its reviewer left one companion item open and unassigned: a `crates/ui` test proving `build_include_elements_from_groups(g, .., &[], ..)` is byte-equal to `build_include_elements(g, .., &[], ..)` on an acyclic graph -- i.e. the evidence that E's deletion is behaviour-preserving. **AUTHORIZED 2026-10-04 as inserted unit D2, `tasks/m235-t3d2-handoff.md`, NOT started.** Why it cannot wait: E DELETES the legacy builder (after which the comparison cannot be written at all) and pins "you add and remove no test"; units C (`tests/integration`) and D (`crates/graph`) cannot host a `crates/ui` file; T4.1 is after E | lands with D2; only then may E be dispatched. Archive this row (verbatim) once D2 is APPROVED | T3-B verdict sec5.6 + finding 3; T3-D verdict finding 13 |

The first two rows above executed by handoff C (`tasks/tool-distill-c-handoff.md`):
`f3dc53b` (Red-gated ANSI-strip at parser entry; drift never hidden),
`c600fb8` (exit-honesty pin); review-gate APPROVED zero findings at
`c600fb8`; suites `Ran 30 tests ... OK` / discover `Ran 184 tests ... OK`.

The third row (M23.5 companion parity test) was discharged by inserted unit D2
(`tasks/m235-t3d2-handoff.md`): test commit `42bb44e`, reviewer records
`4d0f245` and `b0a91d2`; review-gate APPROVED with 0 blocking findings
(12 non-blocking) per `tasks/m235-t3d2-verdict.md`. The row's own hard expiry
("Archive this row (verbatim) once D2 is APPROVED") is therefore satisfied,
and unit E's precondition ("only then may E be dispatched") is cleared -- E is
AUTHORIZED, repaired in place, dispatch-pending. The row is reproduced exactly
as written while it was live: its "NOT started" clause records the state at
authoring time and is superseded by the commits cited above. Its ACCEPTED
two-sample residual stays closed -- do not re-open it.