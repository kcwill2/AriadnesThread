# Milestone Ledger Archive

Verbatim rows discharged from `tasks/milestone_ledger.md` per its archive
rule (worktree-census rows whose disposition was carried out; deferred
findings executed or dropped). Live ledger stays lean; this file is the
audit trail. Discharge evidence lives in the referenced milestone rows /
gates.

## Discharged worktree-census rows

| Worktree | State at 2026-10-01 | Disposition |
|---|---|---|
| `.kilo/worktrees/load-tail-phase-performance` | merged via `7a58e1f` | abandon with row |

Row moved verbatim 2026-10-04 (Phase 4 closeout): its disposition -- merged --
was carried out, so the row is discharged. The three rows still live in
`tasks/milestone_ledger.md` (`troubled-concrete`, `linen-mice`, `jeweled-neon`,
plus M23.5's own `tarjan-s-scc` added at closeout) have NOT been carried out.

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

### Discharged 2026-10-04 (Phase 4 closeout): M23.5 unit E, SUPERSEDED

Moved VERBATIM from the M23.5 detail sub-block in `tasks/milestone_ledger.md`.
Its obligation is discharged: unit E was never started, it was replaced by E1
and E2, and **both are APPROVED** (`5addad7` and `378b240`+`4bc3b88`+`dcd681b`).
Its "NOT started" clauses record the state at authoring time and are superseded
by those commits. Retained for audit; **do not re-open and do not treat any
count or line reference below as current** -- the live sub-block carries the
current state and the live F14 row carries the supersession of record.

- **Unit E (`tasks/m235-t3e-handoff.md`): SUPERSEDED 2026-10-04. NOT started,
  zero commits, nothing landed.** Its implementer stopped at preflight with an
  executed probe: renaming `dto.rs:637` alone made
  `cargo check -p ariadne-cli --all-targets` fail (exit 101,
  `error[E0425]: cannot find function build_include_elements_from_groups in crate
  ariadne_ui` at `crates/cli/src/commands/ui.rs:328:40`) while the contract
  froze `crates/cli/**` byte-unchanged and `crates/ui/src/lib.rs:6` is a bare
  `pub use dto::*;`. The contract shipped unsatisfiable -- see deferred row
  **F14** and its sub-block. **Replaced by two units, both NOT started:**
  - **E1** `tasks/m235-t3e1-handoff.md` -- crate:ui only. Deletes the legacy
    `build_include_elements` (`dto.rs:608`) and its only helper
    `IncludeCycleClass::from_paths`, repoints the NINE in-crate call sites,
    collapses four two-name import lists, converts parity test P1 to a
    frozen-expectation byte lock. It carries P1's conversion **because it
    deletes the legacy builder**: once that builder is gone, P1 compares the
    survivor against itself. Adds `cargo check -p ariadne-cli --all-targets`
    as a gate. Sizing: 1 crate, 7 code paths + journal, 7 gate commands /
    15 invocations (itemised in the handoff), ~250 produced lines.
  - **E2** `tasks/m235-t3e2-handoff.md` -- renames the survivor to the plan's
    canonical public name (plan line 69). The only unit with a declared
    2-crate exception; touches `crates/cli/src/commands/ui.rs:328` and
    `crates/cli/tests/payload_tail_bench.rs:325` (plus one stale comment clause
    at `:64`); gates BOTH reverse dependencies (`-p ariadne-cli --all-targets`,
    `-p ariadne-integration-tests --all-targets`). Sizing: 2 crates, 7 code
    paths + journal, 8 gate commands / 14 invocations, ~70 produced lines.
  - Split rationale: the deletion and the rename are separable across a
    compile-green boundary (verified in both directions), and only the rename
    needs `crates/cli`. E1 cannot carry the rename, because after E1 the sole
    surviving name is the one `crates/cli` calls.
  - **Superseded contract disposition: SUPERSEDED, retained byte-unchanged on
    disk, not archived and not deleted.** E1 commit 1 lands it as the record of
    what was dispatched and rejected; this sub-block and row F14 are the
    supersession of record.
  - D2 findings **F1** (stale counts/census) and **F2** (P1 tautology) stay
    discharged: F1 by E1's re-derived caller census, F2 by E1's P1 conversion.
  - Test counts move in NEITHER unit: `--test ui_test` stays `186 / 0 / 0 / 1`
    and `-p ariadne-ui` stays `186 / 0 / 0 / 3` (186 - 0 + 0 in both). D2's
    `+2` to the workspace line remains a PREDICTION, never a measurement.

**Outcome (for orientation only, not part of the verbatim row):** the
prediction held. D2 landed `+2`, E1 and E2 each landed `+0`, and the chain ends
`1180/0/4/88` at `dcd681b`, reviewer-measured. The single public builder is now
`crates/ui/src/dto.rs:621`, so every `dto.rs` line number in the verbatim row
above is historical.