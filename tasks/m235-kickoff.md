# M23.5 Kickoff - Cycle Substrate: SCC Condensation + Iterative Tarjan

**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` (re-pinned post-merge `f9064f4`)
**Worktree:** `.kilo/worktrees/tarjan-s-scc`
**Default branch:** `main` @ `f9064f4`
**Milestone status:** PLANNED -> READY (kickoff gate cleared)

---

## 1. Frozen Locks (carried from ledger + plan)

| Lock | Source | Enforcement |
|------|--------|-------------|
| IV-1..IV-12 | Plan sec1.2 | Test gates + reviewer audit |
| R1/R2 hygiene (fn<100, file<1000) | `python tools/hygiene_check.py` | blocking |
| UTF-8 mojibake guard | `python tools/encoding_guard.py --all` | blocking |
| Zero new hex literals in UI | `python tools/theme_token_check.py --check-added --base <rev>` | blocking |
| Deterministic export (LF pins) | `tests/golden/**` + `.gitattributes` | byte-stable |
| Planner byte-identity | `planner_facts_parity`, `planner_characterization`, `m00_facts_path_parity_test` | unedited green |
| Cycle-policy seam | `generate_report_with_cycle_policies`, `informational_cycle_count` skip-when-zero serde | stance preserved |

---

## 2. Phase Order & Dependencies

```
Phase 1: Symbol SCC de-recursion (crate:cluster)  - INDEPENDENT
    T1.1 Freeze parity oracle (test-only)
    T1.2 Iterative strongconnect frame machine
Phase 2: CycleGroup core in crate:graph
    T2.1 CycleGroup type + iterative graph Tarjan
    T2.2 Representative-cycle witness
    T2.3 Equivalence-to-old suite
Phase 3: Replacement rewiring (D1-B)
    T3.1 Report + policy seam + CLI + consumers
    T3.2 UI DTO + payload
Phase 4: Integration closeout
    T4.1 Full-workspace verification + hygiene
```

**Phase 1 is self-contained and independently shippable** - start here.

---

## 3. T1 Handoff Scope (this kickoff authorizes T1.1 + T1.2)

| Task | Scope | Gate commands (est.) | Output bound |
|------|-------|---------------------|--------------|
| T1.1 | `crates/cluster/tests/scc_iterative_parity.rs` | 2 (test run + distill) | ~400 lines |
| T1.2 | `crates/cluster/src/scc.rs` (replace `strongconnect`) | 3 (test run x2 + distill) | ~800 lines |

**Handoff file:** `tasks/m235-t1-handoff.md` (authored alongside this kickoff)
**Journal:** `tasks/m235-t1-journal.md` (implementer writes)

---

## 4. Delegation Protocol

Five-role topology. Every hop is a `task` spawn from the orchestrator, and the
orchestrator passes artifact **paths**, never contents.

- **Orchestrator** routes only. It holds `edit` at `ask` and `bash: deny`, and
  cannot run a gate. This is deliberate: a parent's `edit: deny` is inherited by
  every child as a hard ceiling, which would strip `write` and `edit` from all
  five roles and make artifact production impossible.
- **Architect** writes this kickoff plus `tasks/m2355-shape.md`. Its edit grant
  is scoped to `tasks/**` and `.kilo/plans/**`; it has `bash: deny`.
- **Planner** reads the shape artifact, treats it as given, and writes
  `tasks/m235-t1-handoff.md`. It also owns `tasks/milestone_ledger.md`.
- **Implementer** executes one handoff under strict TDD, touching only the
  declared touch-set, and commits with
  `python tools/commit_scoped.py --scope <glob> -m <msg>`. It runs the distilling
  entry points directly - `python tools/testrun.py --distill`, `hygiene_check.py`,
  `encoding_guard.py --staged`. There is no `tool-caller`; that agent no longer
  exists and command execution is not delegated.
- **Reviewer** adjudicates via the `review-gate` skill and returns
  APPROVED/REJECTED with numbered findings, each attributed to the causing role.
  It also runs the workspace-wide acceptance pass.

On REJECTED, the orchestrator resumes the **same** implementer session with the
findings rather than spawning a fresh one, so the implementer keeps its context.

Note on gate listings below: write them in the direct form
(`python tools/testrun.py --distill ...`). The distiller accepts arbitrary
arguments, so a gate written as `python tools/distill.py -- <argv>` will run
whatever `<argv>` names, bypassing the allowlist's intent.

---

## 5. Worktree Census (from ledger)

| Worktree | Disposition at closeout |
|----------|-------------------------|
| `tarjan-s-scc` | merge to main (this milestone) |
| `linen-mice` | M19 complete - merge or abandon with row |
| `jeweled-neon` | abandon with row |
| `troubled-concrete` | `git worktree remove` once confirmed nothing to salvage |

---

## 6. Acceptance Criteria (Plan sec3, mirrored for T1)

1. `crates/cluster/tests/scc_iterative_parity.rs` exists with oracle + equivalence assertions (fixtures + 200 seeded digraphs + 100k chain)
2. Iterative `strongconnect` frame machine passes oracle on all fixtures + stress cases at default stack (64 KiB)
3. `cargo test -p ariadne-cluster` + `planner_characterization` + `planner_facts_parity` + `m00_facts_path_parity_test` all green
4. No R1/R2 hygiene violations; `encoding_guard.py --all` clean
5. `planner` output byte-identical (IV-8)

---

## 7. Risk Register (Plan sec4, T1-relevant)

| Risk | Mitigation |
|------|------------|
| Iterative Tarjan lowlink bug | Frozen recursive oracle (T1.1) before swap; reviewer audits update points |
| 100k chain at default stack | Test runs on 64 KiB thread; oracle runs on 2 MiB thread for comparison |
| Algorithm duplication graph/cluster | Deliberate - no cross-crate dep; reviewer signs off or directs shared internal crate |

---

## 8. Next Actions

1. Architect confirms the shape artifact at `tasks/m2355-shape.md` (this kickoff
   carries the intent; the shape carries the contracts and invariants in the form
   the planner consumes)
2. Planner decomposes Phase 1 into `tasks/m235-t1-handoff.md`, sized to the
   handoff law: one crate, at most ten gates, at most about 1200 lines
3. Implementer runs T1.1 (Red: parity test with recursive oracle), then T1.2
   (Green: iterative frame machine), then refactor
4. Reviewer adjudicates and runs the workspace acceptance pass
5. On APPROVED, planner writes the ledger row and the T2 handoff

---

**Authority:** This kickoff + `m235-t1-handoff.md` are the sole authorization for Phase 1 work. No other work is in scope for this worktree until Phase 1 is APPROVED.