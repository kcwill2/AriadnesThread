# M23.5 T1 Handoff - Symbol SCC De-recursion (crate:cluster)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 1 of 4 (independent, shippable)
**Tasks:** T1.1 + T1.2
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 1
**Kickoff:** `tasks/m235-kickoff.md`
**Worktree:** `.kilo/worktrees/tarjan-s-scc`

---

## 1. Intent & Invariants

### Objective
Replace the recursive `strongconnect` in `crates/cluster/src/scc.rs` with an **iterative frame machine** that has identical observable behavior (IV-8: planner output byte-identical) and eliminates the unbounded-stack hazard (IV-7).

### Invariants (must hold before and after)
- **IV-8 Symbol-level behavior frozen:** `compute_symbol_sccs_from_adj` public signature + output identical (order, intra-SCC sort). Wrapper `compute_symbol_sccs` unchanged. Frame state `Send + Sync`-compatible for rayon planner (adjacency read-only).
- **IV-6 Parity oracle:** iterative Tarjan produces SCC sets identical to current recursive on all existing fixtures + 200 seeded random digraphs + 100k chain.
- **No unbounded recursion:** after T1.2, the detection path does not grow OS stack with graph depth.

---

## 2. API / Interface Contracts

**Unchanged (public):**
```rust
// crates/cluster/src/scc.rs
pub fn compute_symbol_sccs_from_adj(
    symbol_ptrs: &[SymbolPtr],
    adj: &[Vec<usize>],
) -> Vec<SymbolScc>;

pub struct SymbolScc {
    pub members: Vec<SymbolPtr>,
    pub is_cyclic: bool,      // true for nontrivial SCC (size>=2) or self-loop
}

// Internal helper (used by planner, unchanged):
pub fn compute_symbol_sccs(...) -> Vec<SymbolScc>;
```

**New (test-only, T1.1):**
```rust
// crates/cluster/tests/scc_iterative_parity.rs
fn recursive_tarjan_oracle(
    symbol_ptrs: &[SymbolPtr],
    adj: &[Vec<usize>],
) -> Vec<SymbolScc>;  // frozen copy of current recursive strongconnect
```

**Implementation target (T1.2):**
```rust
// crates/cluster/src/scc.rs - replace strongconnect body only
fn strongconnect_iterative(
    symbol_ptrs: &[SymbolPtr],
    adj: &[Vec<usize>],
) -> Vec<SymbolScc>;  // explicit Vec<TarjanFrame>, lowlink on frame pop
```

---

## 3. Plan-Task for Implementer

### T1.1 - Freeze the Parity Oracle (Red)

| Item | Specification |
|------|---------------|
| **File** | `crates/cluster/tests/scc_iterative_parity.rs` (new) |
| **Red** | Add test module with: <br>- `recursive_tarjan_oracle` - verbatim copy of current recursive `strongconnect` logic (test-local, `#[cfg(test)]`) <br>- `test_fixtures_parity` - asserts `strongconnect_iterative` (not-yet-existent, returns empty) != oracle on all fixtures <br>- `test_seeded_random_parity` - 200 seeded LCG digraphs (seed documented), same assertion <br>- `test_100k_chain_parity` - 100k-node chain `s0->s1->...->s99999` + back-edge variant `s99999->s0`; chain runs on **default 64 KiB stack** (crashes recursive = Red demonstrates overflow class); oracle runs on 2 MiB thread for comparison |
| **Green** | T1.2 makes all assertions pass |
| **Refactor/Hygiene** | Oracle stays test-only; production `scc.rs` carries NO `#[cfg(test)]` copy of itself. Functions < 100 lines, file < 1000 lines. |

**Gate command:**
```bash
python tools/testrun.py --distill -p ariadne-cluster -- scc_iterative_parity
```

---

### T1.2 - Iterative `strongconnect` Frame Machine (Green)

| Item | Specification |
|------|---------------|
| **File** | `crates/cluster/src/scc.rs` - replace `strongconnect` function body |
| **Red** | T1.1 suite is red |
| **Green** | Implement explicit-stack frame machine: <br>- `struct TarjanFrame { symbol_ptr: SymbolPtr, neighbor_idx: usize, ... }` <br>- Stack: `Vec<TarjanFrame>` <br>- **Lowlink propagation on frame POP** (not call return) - this is the classic trap the design structurally avoids <br>- Index counter, on_stack bitset, lowlink array - same state as recursive <br>- Output contract identical: same SCC order, same intra-SCC sort, `Send + Sync`-compatible frame state (adjacency read-only) |
| **Refactor** | Decompose frame machine into predicate helpers < 100 lines each: <br>- `run_tarjan` - main loop <br>- `advance_frame` - visit next neighbor or pop <br>- `finish_child` - lowlink update on child completion <br>- `emit_group_if_root` - SCC emission when `index == lowlink` <br>- `representative_cycle_for` - (stub, T2.2) <br>Keep `build_symbol_adjacency` untouched. |
| **Hygiene** | `python tools/hygiene_check.py` clean; `python tools/encoding_guard.py --staged` clean. |

**Gate commands:**
```bash
python tools/testrun.py --distill -p ariadne-cluster -- scc_iterative_parity
python tools/testrun.py --distill -p ariadne-cluster
python tools/distill.py -- cargo build -p ariadne-cluster
python tools/hygiene_check.py
python tools/encoding_guard.py --staged
```

---

## 4. Review Gate (Reviewer Audit Focus)

When implementer reports completion, reviewer audits:

1. **Lowlink update points** - push/advance/pop logic against Tarjan invariant (`lowlink[v] = min(lowlink[v], index[w])` for tree edges; `lowlink[v] = min(lowlink[v], index[w])` for back edges to on-stack nodes)
2. **Tie-break determinism** - SCC order matches oracle on seeded digraphs
3. **No dual representation** - only iterative path in production; oracle strictly test-local
4. **Send + Sync compatibility** - frame state contains no `Rc`/`RefCell`/`!Send` types; adjacency remains `&[Vec<usize>]` read-only
5. **Planner byte-identity** - `planner_facts_parity`, `planner_characterization`, `m00_facts_path_parity_test` all green unedited
6. **Hygiene gates** - R1/R2 blocking clean; R3 naming warn-only

**Verdict:** APPROVED / REJECTED + numbered findings

Each numbered finding must be attributed to the role that caused the defect -
architect, planner, implementer, or configuration - and labelled blocking or
non-blocking. A deviation that the implementer self-flagged is adjudicated
independently on the evidence, not accepted because it was declared.

The reviewer also runs the workspace-wide acceptance pass for this milestone,
which no implementer handoff carries: `python tools/testrun.py --baseline` and
`python tools/hygiene_check.py` across the whole workspace, compared against the
`1147 / 0 / 4 / 85` post-merge baseline. Those belong here rather than in the
implementer's gate list.

---

## 5. Discipline Clauses

- **Command execution** - the implementer has `bash` on an allowlist restricted to
  the distilling entry points and `commit_scoped.py`. Raw `cargo` and raw `git`
  are denied. There is no `tool-caller` in this topology and no delegation for
  command execution; if a gate is refused, read the denial carefully (see 5.1)
  before concluding anything.
- **Strict TDD** - T1.1 test written first (Red), then T1.2 implementation
  (Green), then refactor
- **Code hygiene** - fn < 100 lines, file < 1000 lines, meaningful names, single
  responsibility
- **UTF-8 safety** - never use PowerShell `Set-Content`/`Out-File`/`>` for
  tracked files; use Kilo Read/Edit/Write tools
- **Scope boundary** - only `crates/cluster/src/scc.rs` +
  `crates/cluster/tests/scc_iterative_parity.rs`; no changes to `graph`, `cli`,
  `ui`, `report`, or planner

### 5.1 Reading a refusal

A refused command always reports the catch-all deny rule
(`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`),
whatever the real cause - including a command that simply is not on the
allowlist. Treat that text as boilerplate.

Attempt each gate independently. Never infer that command execution is
unavailable from one refusal, and never drop a gate because its first
invocation was refused. If a gate genuinely cannot run, say so in the journal
with the verbatim refusal rather than silently narrowing the acceptance set.

Note that the distilling entry points accept arbitrary arguments, so a gate
expressed as `python tools/distill.py -- <argv>` will run whatever `<argv>` names.
Write gates in the direct form above, where the allowlist actually constrains
what runs.

---

## 6. Handoff Sizing Check

| Metric | Bound | Actual (est.) |
|--------|-------|---------------|
| Scope | <= 1 crate | `cluster` only |
| Gate commands | <= 10 | 5 |
| Output lines | <= 1200 | ~1200 |

**Fits single handoff unit.**

---

**Authorization:** This handoff + `m235-kickoff.md` are the sole authorization for T1 work. Implementer may not proceed to Phase 2 until reviewer returns APPROVED for T1.1+T1.2.