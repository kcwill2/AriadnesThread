# M23.5 T2-B Handoff - Equivalence-to-Old Safety Net (crate:graph)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 2 of 4, unit B of 2 (T2.3 only). Unit A is `tasks/m235-t2a-handoff.md` (T2.1 + T2.2).
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 2 (T2.3)
**Prerequisite:** `tasks/m235-t2a-handoff.md` APPROVED and landed
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`
**Journal:** `tasks/m235-t2b-journal.md` (you create it; record your own SHAs in it)

Every line reference in this file was read out of the tree at `e2a05e2`. If a
line reference disagrees with the file you read, the FILE wins -- report the
drift in your journal.

---

## 1. Objective and Invariants

### Objective

Freeze the OLD recursive `detect_cycles` path enumeration as a test-local
oracle and prove, over the existing fixture corpus plus a seeded random
digraph corpus, that the new `detect_cycle_groups` output agrees with it about
**which files are cyclic** -- and that no old reported cycle straddles two new
groups. This is the migration safety net the plan makes mandatory BEFORE the
swap (plan sec1.2 IV-6, sec4).

Plan T2.3's Green is literally "fixes any gaps found in T2.1/T2.2". So this unit
may carry a production fix -- but ONLY one that this suite proves is needed.

### Invariants this unit proves

- **IV-2 both directions:** on the corpus, a file is on some old reported cycle IFF it is in some new group's `member_files`. This is the load-bearing claim of the whole milestone.
- **Path containment:** every old reported cycle's member set is a SUBSET of some single new group's `member_files` -- old DFS paths never straddle two groups.
- **IV-6 Parity oracle:** the frozen oracle is the definition of correct and may never be edited to accommodate the new implementation.
- **IV-1 Disjoint partition** (implied by the assertion above, asserted directly).

### Invariants this unit must NOT disturb

Everything T2-A established. In particular `detect_cycles` still exists at
`crates/graph/src/graph.rs:234` and `dfs_cycle` at `:260-287` -- T2-A was
strictly additive (see T2-A sec2.1 for why), and T3.1 deletes both.

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

- `crates/graph/tests/cycle_groups_equivalence_test.rs` (**new** -- the suite)
- `crates/graph/src/cycle_groups.rs` (**existing from T2-A** -- editable ONLY to fix a violation this suite proves; see sec5)
- `tasks/m235-t2b-journal.md` (**new**, yours)

You MUST NOT touch: `crates/graph/src/graph.rs`,
`crates/graph/src/closure_equivalence_tests.rs`, `crates/graph/src/lib.rs`,
`crates/graph/tests/cycle_groups_test.rs`, `crates/cluster/**`, `crates/cli/**`,
`crates/ui/**`, `crates/report/**`, `tests/fixtures/**`, `tests/golden/**`,
`tools/**`, `.kilo/plans/**`, `Cargo.toml`, `Cargo.lock`.

No `git stash`, no branch change, no `git add .`.

**Every commit in this handoff carries an explicit `--scope` (gate 6).** The
worktree also carries an unrelated dirty path, `tests/fixtures/basic/artifacts/includes/report.json`
(CRLF terminators, Phase 1 verdict finding 12). T2-A restores it in its preflight;
confirm it is clean via gate 5 before you commit. It must never appear in your
commit, and you must not re-dirty it.

---

## 3. API Contracts (frozen -- read, do not redesign)

The behavior under test, verbatim from plan sec1.3:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleGroup {
    pub member_files: Vec<FileId>,
    pub representative_cycle: Vec<FileId>,
    pub external_in_count: usize,
    pub external_out_count: usize,
}

impl IncludeGraph {
    pub fn detect_cycle_groups(&self) -> Vec<CycleGroup>;
}
```

Tie-break decisions D-A1 (seed universe = `file_paths` keys plus every edge
`source` plus every resolved `destination`, sorted) through D-A5 (witness is
existence, not exactness) are pinned in `tasks/m235-t2a-handoff.md` sec4 and are
in force. Read them; they determine the expected results you assert.

### 3.1 The frozen oracle source -- read it from the tree, copy it verbatim

| Item | Location |
|---|---|
| `pub fn detect_cycles(&self) -> Vec<Vec<FileId>>` | `crates/graph/src/graph.rs:234-258` |
| `fn dfs_cycle(...)` (6 params, private) | `crates/graph/src/graph.rs:260-287` |

Copy BOTH bodies into your test file as `old_detect_cycles` /
`old_dfs_cycle`, plus a test-local adjacency builder. Renaming a local inside
the copied body is a deviation (Phase 1's reviewer accepted
semantics-preserving renames D1-style, but "verbatim" is the contract and
preserves the evidence). If you must rename, log it.

**Why the oracle cannot just call the live method.** It must be a COPY, frozen
before the swap, for the same reason `crates/cluster/tests/scc_iterative_parity.rs`
carries `oracle_strongconnect`: when `detect_cycles` is deleted in T3.1, the
suite must still hold the old semantics. Calling the live method would make the
suite tautological.

### 3.2 Test-side adjacency mirror

`IncludeGraph::adjacency()` is `pub(crate)` (`crates/graph/src/graph.rs:83`),
so an integration test cannot call it. Build the oracle's adjacency from
`graph.edges`, mirroring `graph.rs:83-95` exactly: skip `destination: None`,
group by `source`, sort each destination list, dedup. Put a comment citing
`graph.rs:83-95` so the next reader sees a deliberate mirror, not the forbidden
duplicate of the production loop. Copy the `edge()` / `graph_from_shapes()`
shape from `crates/graph/src/closure_equivalence_tests.rs:24-46`.

### 3.3 Reuse precedents -- copy, do not reinvent

| Precedent | Location |
|---|---|
| `InlineLcg` (Knuth MMIX, multiply-add only, no RNG dependency) | `closure_equivalence_tests.rs:373-393` |
| `RANDOM_FIXTURES` locked-seed table (5 topologies) | `closure_equivalence_tests.rs:395-416` |
| `random_graph(seed, file_count, topology)` incl. `PureDag`, `BigCycle`, `TwoSccsBridge`, `SelfLoops`, `FullBlob` | `closure_equivalence_tests.rs:418-490` |
| seeded-sweep test body | `closure_equivalence_tests.rs:511-535` |
| `use super::IncludeGraph` + `crate::types::{...}` import shape | `closure_equivalence_tests.rs:16-19` |

**Use the SAME seed values** as `closure_equivalence_tests.rs:405-416`. Changing
them changes the shared fixture corpus and would silently un-verify the
existing closure suite's coverage.

---

## 4. The one place old and new legitimately differ

`detect_cycles` seeds its DFS from `self.file_paths` ONLY (`graph.rs:240-241`).
`detect_cycle_groups` seeds from the union universe (D-A1: `file_paths` plus
every edge `source` plus every resolved `destination`). So when an include
endpoint was never `register_file`'d, the old path enumeration can MISS an
entire cycle that the new one finds. That divergence is CORRECT under IV-2 --
the old behavior is the bug -- but an unconditional equality assertion would
fail on it, and a suite that quietly skips such graphs would be weaker than it
looks.

Therefore the assertions are scoped:

- **Exact equality** (old node set == new node set) on every graph whose edge
  endpoints are ALL registered. This covers every fixture in
  `closure_equivalence_tests.rs:59-176` (they all register their files) and the
  whole seeded corpus built by `random_graph`, which registers `files[1..=count]`
  before linking (`:421-424`).
- **One explicit divergence case** with no equality claim, asserting the
  direction that is safe: every file the OLD enumeration called cyclic is still
  in some new group (new is never a subset of old), and the graph used has at
  least one unregistered endpoint so the case is real.

Do not paper over this by filtering the divergence out of the corpus. Name it.

---

## 5. RGB Protocol for a test-only safety net

T2.3 has no new production behavior to drive Red, so its Red is
**mutation-proven**: the suite must be shown capable of failing before it is
trusted. This is the honest form of Red for an equivalence suite, and the
reviewer will check for it because Phase 1 shipped a gate that silently selected
zero tests (verdict finding 2).

**Step 1 - write the suite.** Four `#[test]` functions, exact names:

| # | Name | Asserts |
|---|---|---|
| 1 | `test_group_membership_equals_old_cycle_node_set_on_registered_corpus` | IV-2, exact equality both directions, over every existing fixture shape plus the seeded corpus |
| 2 | `test_old_cycle_members_are_subset_of_a_single_group` | path containment, per old reported cycle |
| 3 | `test_new_groups_never_miss_a_file_the_old_enumeration_called_cyclic` | new is never a subset of old; also covers the unregistered-endpoint divergence of sec4 |
| 4 | `test_unregistered_endpoint_cycle_is_found_by_groups_but_not_by_old_paths` | the named divergence case, documented, no equality claim |

Plus an `assert!(...)` that the corpus actually exercises both regimes: at
least one fixture with zero cyclic files and at least one with a nontrivial
group. A corpus that is all-DAG or all-cycle proves nothing -- assert the mix.

**Step 2 - prove the suite bites (the Red).** Temporarily mutate ONE production
invariant in `cycle_groups.rs`, re-run gate 1, record the failure verbatim, then
REVERT. Recommended mutation, chosen because it is the subtlest thing T2-A could
get wrong and the easiest to ship broken:

> reverse the IV-5 tie-break -- emit groups in the opposite order from the
> Kahn min-heap.

Expected failure: test 1 fails on a graph whose condensation has two
INCOMPARABLE groups, with a message naming the two groups and their orderings.
If your mutation does NOT produce a failure, your assertions are too weak --
that is the finding, and you must strengthen them before committing. Do not
proceed on a suite that survived its own mutation.

**Step 3 - run clean.** Revert the mutation, re-run gate 1, and confirm the
canonical line.

**Step 4 - gap fixes, if and only if the suite proves one.** If a test fails
against unmutated production code, that failure IS a real defect in T2-A and
this is your Green. You may edit `crates/graph/src/cycle_groups.rs` -- and
ONLY that file -- to fix it. Requirements: the journal records the failing
assertion verbatim BEFORE the fix; the fix is the minimum change that turns that
assertion green; no assertion is weakened; no new API appears. Any defect whose
fix requires touching `graph.rs`, the report layer, or another crate is OUT OF
SCOPE: report it and stop, do not fix it here.

Plan T2.3's Green is "fixes any gaps found in T2.1/T2.2". Finding none is a
legitimate, recordable outcome -- say so explicitly in the journal rather than
leaving it ambiguous.

---

## 6. Gates (exact commands -- 6 invocations)

Every gate is written in DIRECT form. **Never write a gate as
`python tools/distill.py -- <argv>`** -- the distiller is an argv pass-through
and that form bypasses the allowlist.

**Never write a bare-word test filter.** `python tools/testrun.py ... -- <word>`
passes `<word>` to cargo as a TEST-NAME filter: Phase 1's gate written that way
selected zero tests and returned `0 passed / 0 failed / 0 ignored / 7 suites`
with exit 0, so it could never have observed a Red either (verdict finding 2).
Use `--test <target>`, which `testrun.py` forwards to cargo as `--test <target>`
and which selects a real binary.

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_equivalence_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-graph
3. python tools/distill.py -- cargo build -p ariadne-graph
4. python tools/hygiene_check.py
5. python tools/checkpoint_guard.py status
6. python tools/commit_scoped.py --scope 'crates/graph/tests/cycle_groups_equivalence_test.rs' --scope 'crates/graph/src/cycle_groups.rs' --scope 'tasks/m235-t2b-journal.md' -m "test(graph): freeze old detect_cycles oracle and prove group parity"
```

**Expected verdict line (gate 1, clean):**
`4 passed / 0 failed / 0 ignored / 1 suites`

**Expected verdict line (gate 2, clean):**
`116 passed / 0 failed / 0 ignored / 5 suites`

`116 / 5` is arithmetic, not a guess: `-p ariadne-graph` measured
`103 / 0 / 0 / 3` at `e2a05e2`; T2-A added one target with nine tests
(`112 / 0 / 0 / 4`); this unit adds one target with four tests. Gate 2 also
re-proves T2-A's nine tests and the `closure_equivalence` suite in one run.

**Gate 3 must report zero warnings.** An unused oracle helper is a finding, not
a style note -- if your oracle body needs a helper that the compiler sees as
unused, delete the helper rather than suppressing the warning.

---

## 7. Frozen Locks

| Lock | Proving command | Expected |
|---|---|---|
| the oracle is a FROZEN COPY, never edited to fit | reviewer's field-by-field diff against `git show "2708425:crates/graph/src/graph.rs"` region `:234-287` | only semantics-preserving renames, each logged |
| `detect_cycles` / `dfs_cycle` still present and unmodified | gate 2; `git diff` shows no edit to `graph.rs` | green; `graph.rs` byte-unchanged |
| IV-10 closure equivalence green | gate 2 | green; `closure_equivalence_tests.rs` byte-unchanged |
| T2-A's nine tests unchanged | gate 2 | green; `cycle_groups_test.rs` byte-unchanged |
| seeded corpus matches `closure_equivalence_tests.rs:405-416` | reviewer's seed comparison | identical seed constants |
| no new cross-crate dependency | gate 3; `crates/graph/Cargo.toml` byte-unchanged | zero new deps |
| R1/R2 hygiene | gate 4 | no R1/R2; every fn < 100 lines; new file < 1000 lines (expected ~285) |
| UTF-8 mojibake | `python tools/encoding_guard.py --staged` | clean, no output; authored content pure ASCII |
| touch-set | reviewer's per-commit `git diff --name-only` | exactly the in-set paths |

Workspace-wide gates are NOT yours: `python tools/testrun.py --baseline`,
`python -m unittest discover -s tools`, `python tools/ui_smoke.py`, and
`python tools/bench_compare.py` belong to the reviewer's acceptance pass.
`--baseline` is currently BROKEN (ledger deferred row M23.5 F9: `usage error:
missing value for --baseline`) and `ui_smoke` / `bench_compare` are currently
REFUSED by the reviewer allowlist (M23.5 F10). Do not work around either, and
never route a refused command through `distill.py`.

---

## 8. Reviewer Gate Reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
sec2. The reviewer will audit:

1. **Oracle authenticity.** Field-by-field comparison of the frozen copy against the pre-T2-A blob of `crates/graph/src/graph.rs:234-287` -- entry head, neighbor loop, both `min` update rules, the `rec_stack` back-edge test, the `current_path.iter().position(...)` extraction, and the `rec_stack.remove` / `current_path.pop` epilogue. This is the same check the Phase 1 reviewer ran and it is the only evidence that the oracle is the OLD algorithm and not a re-derivation.
2. **Mutation evidence.** The journal must contain the verbatim failure from the tie-break mutation, proving the suite can fail. A suite that never went red is a finding, not a pass.
3. **IV-2 both directions**, and that the equality scope in sec4 is honest -- the divergence case is named, asserted, and not filtered away.
4. **Path containment.** Each old reported cycle's members sit inside ONE group.
5. **Corpus regime mix.** The suite asserts it covers both a cyclic and an acyclic regime.
6. **No oracle leakage.** The oracle exists only in the test target. Production `cycle_groups.rs` must contain no `#[cfg(test)]` copy of the old algorithm -- the "no dual representation" audit from Phase 1 (verdict 4.3) applies here too.
7. **Gap fixes, if any.** Every production edit is justified by a quoted Red, minimal, and assertion-preserving.
8. **Hygiene and locks.** Line counts, R1/R2 clean, `Cargo.toml` unchanged, `graph.rs` byte-unchanged.

**Report back (final message):** commit SHAs (also in your journal -- Phase 1's
implementer omitted them and the reviewer had to recover them from `git log`,
verdict finding 8); the mutation-Red evidence verbatim; Green evidence quoting
the canonical lines and capture paths for gates 1, 2, and 3; an explicit
statement of whether a gap was found and fixed; every deviation with its
reason; out-of-scope observations, report-only. Never paste raw cargo or git
output.

---

## 9. Discipline Clauses

- **Command execution.** `bash` is an allowlist restricted to the distilling entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED. There is no `tool-caller` in this topology and no delegation for command execution.
- **Reading a refusal.** A refused command ALWAYS reports the catch-all deny rule (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`), whatever the real cause -- including "that command is simply not on the allowlist". Treat that text as boilerplate. Attempt each gate INDEPENDENTLY. Never infer from one refusal that command execution is unavailable, and NEVER drop a gate because its first invocation was refused. If a gate genuinely cannot run, record the VERBATIM refusal in your journal rather than silently narrowing the acceptance set.
- **Strict TDD.** Suite first, mutation-proven Red, then clean Green, then Refactor. Never weaken an assertion to make it pass. Never commit a mutated state or a Red.
- **Code hygiene.** Functions < 100 lines, files < 1000 lines, no `tmp`/`res`/`foo`/`bar`/`test1`, no single-letter locals in code you author, one responsibility per function, no source line >= 2000 characters.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>`, or `Get-Content | -replace` on a tracked file. Use the Kilo Read/Edit/Write tools, or byte-level `open(path, "wb")` in a script. Authored content is pure ASCII; build any non-ASCII test signature from byte escapes (`b"\xc3\x82\xc2\xa7"`), never a literal. When quoting git content, quote the WHOLE argument: `git show "rev^:path"`.
- **Scope.** Touch only sec2. On discovering out-of-scope breakage: report it, do not fix it.
- **Journal.** Append a 1-3 line checkpoint after each Green gate: module, commit SHA, any deviation, any lock or contract finding. Record your own SHAs in it.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A dead session is never resumed -- a successor runs `python tools/checkpoint_guard.py resume --spec tasks/m235-t2b-handoff.md --scope ... --cmd ...` (skill `resume-checkpoint`).

---

## 10. Sizing Check (handoff law, stated explicitly)

| Metric | Bound | This unit |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 top-level module directory | `crates/graph` only -- 1 crate |
| Gate commands | ~10 | **6** |
| Produced lines (implementation + test diff) | ~1200 | **~310** (suite ~285, possible `cycle_groups.rs` gap fix 0-25, journal ~60) |

**Single handoff unit -- fits, with headroom on all three axes.**

Split from T2-A because plan T2.3 is not parallel work: its Green is "fixes any
gaps found in T2.1/T2.2", i.e. it is the verifier for the preceding unit and must
run against a landed, reviewed T2-A. See `tasks/m235-t2a-handoff.md` sec11 for
the full split rationale and the measurement that triggered it (all of plan
Phase 2 estimated ~1030 lines and exactly 10 gates -- 100% of the gate bound with
zero slack).

---

## 11. Authorization

This handoff, together with `tasks/m235-kickoff.md`, the M23.5 milestone row in
`tasks/milestone_ledger.md`, and an APPROVED `tasks/m235-t2a-handoff.md`, is the
sole authorization for T2.3.

Phase 3 (T3.1 report / policy-seam / CLI / consumer rewiring, T3.2 UI DTO +
payload) and Phase 4 (T4.1 integration closeout) remain UNAUTHORIZED. Do not
begin them. Do not delete `detect_cycles` or `dfs_cycle`, do not repoint
`graph.rs:356` / `graph.rs:534` / `closure_equivalence_tests.rs:515`, and do not
touch `crates/cli/src/commands/ui.rs:310` or
`crates/cli/tests/payload_tail_bench.rs` -- all of that is Phase 3.