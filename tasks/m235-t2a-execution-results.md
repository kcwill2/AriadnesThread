# M23.5 Execution Results - Phase 2 unit A (T2.1 + T2.2, crate:graph)

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Range:** `e2a05e2..7c94af3` (branch `tarjan-s-scc`, worktree
`.kilo/worktrees/tarjan-s-scc`)
**Verdict:** `tasks/m235-t2a-verdict.md` - APPROVED, 0 blocking / 10
non-blocking findings
**Scope note:** T2-B (T2.3, equivalence-to-old oracle suite) and all Phase 3 /
Phase 4 work were neither demanded nor credited.

> Written to a distinct path from `tasks/m235-execution-results.md` on purpose:
> that file holds the Phase 1 acceptance record, which this pass was told to
> confirm intact and not disturb. Phase 1's own addendum lives at
> `tasks/m235-execution-results.md`; Phase 2 unit A's lives here.

---

## 1. Canonical summary line (verbatim)

Workspace, reviewer-run, `python tools/testrun.py --distill --timeout 3600`:

```
1165 passed / 0 failed / 4 ignored / 87 suites
```

Ledger-measured baseline `1156 / 0 / 4 / 86`.
**Delta: +9 passed, 0 failed, 0 ignored, +1 suite.**

The delta is fully attributed: exactly one new integration target
(`crates/graph/tests/cycle_groups_test.rs`) carrying exactly nine `#[test]`
functions. Unlike Phase 1's residual (ledger F11, still open), there is no
unattributed remainder.

## 2. Handoff acceptance criteria, one by one

| # | Criterion | Outcome | Evidence |
|---|-----------|---------|----------|
| 1 | `CycleGroup` + `detect_cycle_groups` added additively to `crate:graph`, plan sec1.3 contract verbatim, no redesign | MET | `crates/graph/src/cycle_groups.rs:19-31` reproduces the struct field-for-field with the plan's derives and doc text; `:39` is `pub fn detect_cycle_groups(&self) -> Vec<CycleGroup>` as an inherent `impl IncludeGraph` block. The handoff's `[c0, 0]` doc typo was resolved toward the plan's `[c0, c0]` (`:26`). |
| 2 | IV-2 cycle iff nontrivial | MET (partially unlocked - finding 1) | `filter_cyclic_groups` / `group_is_cyclic` (`:281-302`) keep size>=2 groups and exactly those singletons with a self-loop in the deduplicated adjacency. Seed universe `tarjan_seed_universe` (`:67-78`) unions `file_paths` keys + edge sources + resolved destinations, matching `graph.rs:124-130`, sorted ascending. No test exercises the unregistered-endpoint case. |
| 3 | IV-3 every intra-group edge cyclic (the headline wart) | MET in its mandated form (finding 3) | `test_diamond_with_chord_reports_every_intra_group_edge_as_cyclic` asserts plan sec1.3's rule "class `cycle` iff both endpoints are in the same group" over every resolved edge pair, plus `d` in no group. Red independently confirmed by tracing the frozen `dfs_cycle` oracle (`graph.rs:260-287`) against the fixture. |
| 4 | IV-4 witness is an edge-backed closed walk | MET (Red undemonstrated - finding 2) | `closed_walk_witness` (`:450-488`) audited directly: consecutive pairs are real adjacency entries, closure exact, length >= 2, all nodes members. `assert_witness_is_edge_backed` (`cycle_groups_test.rs:471-501`) verifies closed / len>=2 / edge-backed / in-group against a `graph.edges` rebuild, over 8 locked-seed fixtures and a 200-node tangle. |
| 5 | IV-5 determinism and tie-break | MET | Kahn over the condensation with a `Reverse<(FileId, usize)>` min-heap keyed on `groups[index][0]` (`:309-337`). Determinism is measured, not asserted: the test calls `detect_cycle_groups()` twice in-process, and Rust's per-instance `RandomState` gives the two adjacency maps different iteration orders. Incomparable-group tie-break pinned by the connector-free sub-fixture. |
| 6 | IV-7 no unbounded recursion; 100 000-node chain at 64 KiB | MET | `test_deep_include_chain_completes_on_default_stack` runs `detect_cycle_groups()` on `Builder::stack_size(64 * 1024)`. Gate 1 capture shows the target completing in 17.87 s, i.e. the chain executed at 64 KiB. Both the Tarjan machine and the witness walk keep frames in `Vec`s. |
| 7 | IV-10 closure parity untouched | MET | Gate 2 fresh: `112 passed / 0 failed / 0 ignored / 4 suites`. Capture names passing `graph::closure_equivalence::*` cases; `closure_equivalence_tests.rs:515` still calls `graph.detect_cycles()`; that file is byte-unchanged in the range. |
| 8 | IV-11 cycle-policy seam untouched | MET | `cycle_is_informational` (`graph.rs:299`) and the six `informational_cycle_count` unit tests are byte-unchanged; `graph.rs` unchanged at 704 lines. |
| 9 | Additive-only; `detect_cycles` + `dfs_cycle` survive; no third representation; no premature rewiring | MET | `detect_cycles` at `graph.rs:234`, `dfs_cycle` at `graph.rs:260`, both intact at their pinned lines. `detect_cycle_groups` has one production definition and ZERO production call sites workspace-wide. `generate_report_with_cycle_policies` untouched. No pre-existing file was edited, so no pre-existing behaviour could be altered to accommodate the new code. |
| 10 | No cross-crate dependency; `Cargo.toml` byte-unchanged | MET | `crates/graph/Cargo.toml` unchanged, no `ariadne-cluster` edge; `crates/cluster/Cargo.toml` unchanged. The word `cluster` appears in `cycle_groups.rs` only in doc comments (lines 6, 83, 221). The ~60-line frame-machine duplication is the deliberate accepted shape sharing of D-A7. |
| 11 | D-A6 lowlink update points: emit-if-root FIRST on pop, then propagate | MET | `retire_frame` (`:226-230`): `emit_group_if_root` is the first statement, `finish_child` the second, on the single pop path. Push (`:157-167`), advance (`:173-185`), pop all audited against `crates/cluster/src/scc.rs:100-241` and against the recursive Tarjan body. Tree-edge propagation deferred to pop; cross edges to completed SCCs correctly ignored. |
| 12 | Hygiene: functions < 100 lines, files < 1000, meaningful names, no dead code | MET | `python tools/hygiene_check.py`: zero R1/R2 workspace-wide, and zero R3 in either new file. `cycle_groups.rs` 527 lines, `cycle_groups_test.rs` 564, `lib.rs` 20, `graph.rs` 704. Longest function ~39 lines. Gate 3 fresh: `ok (exit 0, 0 warnings emitted)`. `group_is_informational` confirmed absent by grep (zero matches across `crates/`). |
| 13 | Touch-set: every commit inside the declared set | MET | `git diff --name-only e2a05e2..7c94af3` returned exactly 7 in-set paths; per-commit sets verified separately. `tests/fixtures/basic/artifacts/includes/report.json` appears in NEITHER commit. |
| 14 | Preflight step 0: fixture restored to LF and byte-equal to HEAD | MET | Path absent from the `git diff HEAD --numstat` dirty section at session start and after this pass. `git ls-files --eol` -> `i/lf w/lf attr/`. No `\r` byte in the file. `git diff --numstat HEAD -- <path>` returns no rows. |
| 15 | Nine tests, exact names | MET | All nine `#[test]` functions present in `cycle_groups_test.rs` with the handoff's exact names; gate 1 selects 9 tests (not vacuously zero). |
| 16 | UTF-8 mojibake guard | MET | `python tools/encoding_guard.py --all` -> no output. `--staged` -> no output. Authored content pure ASCII. |
| 17 | Report-evidence discipline (digest-shaped, no raw dumps) | MET | Journal quotes canonical lines, exit codes, and capture paths; it explicitly flags that the R1 capture path was transient and overwritten by a later run rather than citing it as retained. Two refusals logged verbatim with the boilerplate read correctly. |

## 3. Plan section 3 acceptance criteria, Phase 2 slice

| # | Plan criterion | Outcome | Note |
|---|----------------|---------|------|
| 3 | No unstyled/amber edge connects two `cycle`-classed nodes (IV-3) | MET in this unit's scope | Locked as the group-membership iff on the chord fixture. The UI-rendered consequence lands in T3.2. Finding 3 carries the stronger per-edge assertion to T2-B. |
| 4 | Deep-chain stress passes at default stack in both crates; equivalence oracles green on fixtures + 200 seeded digraphs each | PARTIAL - correctly so | `graph` deep-chain half MET (this unit; 100 000-node chain at 64 KiB, production code, green). `cluster` half was Phase 1 (`scc_iterative_parity`, 200 seeded digraphs). The `graph` equivalence-to-old oracle is T2.3 and is not authorized yet. Criterion 4 completes at T2-B, not here. |

Criterion 1 ("`detect_cycles` no longer exists") is a Phase 3 criterion and is
**not** expected to hold at T2-A; its non-satisfaction is the plan-sanctioned
coexistence of handoff sec2.1 (D1-B), not a finding.

## 4. Gates run, with results

| Gate | Command | Result |
|---|---|---|
| H1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_test` | `9 passed / 0 failed / 0 ignored / 1 suites` |
| H2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph` | `112 passed / 0 failed / 0 ignored / 4 suites` |
| H3 | `python tools/distill.py -- cargo build -p ariadne-graph` | `ok (exit 0, 0 warnings emitted)` |
| H4 | `python tools/hygiene_check.py` | zero R1/R2; zero R3 in new files |
| H5 | `python tools/encoding_guard.py --staged` | clean |
| H6 | `python tools/checkpoint_guard.py status` | fixture absent from dirty section |
| A1 | `python tools/testrun.py --distill --timeout 3600` | `1165 passed / 0 failed / 4 ignored / 87 suites` |
| A2 | `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 184 tests ... OK` |
| A3 | `python tools/encoding_guard.py --all` | clean |
| A4 | `python tools/hygiene_check.py` (workspace) | zero R1/R2 |

Both pinned handoff verdict lines reproduce EXACTLY (`9 / 0 / 0 / 1` and
`112 / 0 / 0 / 4`), and neither gate is vacuous: gate 1 selected 9 tests, gate 2
selected 112 tests across 4 suites with the per-binary split visible in the
capture (101 lib + 9 cycle_groups + 2 m22_marker + 0 doc-tests).

## 5. What was NOT run, and why

| Command | Verbatim result | Disposition |
|---|---|---|
| `python tools/testrun.py --baseline` | `usage error: missing value for --baseline` | Ledger M23.5 F9, reproduced ONCE and recorded once. Not treated as a gate failure, not fixed. The full-workspace invocation it wraps (A1) was run and compared by hand instead. |
| `python tools/ui_smoke.py` | refused: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` | Ledger M23.5 F10. Attempted independently, once. NOT routed through `distill.py`. Low impact: T2-A touches no UI code. |
| `python tools/bench_compare.py` | refused: identical verbatim text | Ledger M23.5 F10. Attempted independently, once. NOT routed through `distill.py`. **Material impact: there is still no perf evidence for `detect_cycle_groups`**, a real surface that `payload_tail_bench.rs` measures and T3.2 must re-pin. |

Two further refusals are worth recording because they are the same trap in a
new shape: `;`-chained variants (`python tools/hygiene_check.py; Write-Output
...`) and `python -c "..."` are both refused with the same catch-all
boilerplate, so an exit code cannot be captured via a shell chain and no ad-hoc
script can be executed. Each plain, allowlisted form was then run separately and
did execute. Verdict finding 9.

Consequence for this pass's evidence quality: I could not run my own
adversarial fixtures against the compiled implementation (verdict finding 10).
Sections 2 and 3 of this document therefore rest on fresh gate runs plus
source-level verification - the full production module, the frozen
`dfs_cycle`/`detect_cycles` oracle, the cluster frame machine, and the whole
test file were read, and the chord fixture was traced by hand through the old
oracle to confirm the R2 Red. Findings 1 and 3 are where that substitution
matters most; both are carry-forwards for T2-B's test-only scope.

## 6. Per-commit verification evidence

| Commit | Files | Evidence |
|---|---|---|
| `34f5fe4` | `crates/graph/src/cycle_groups.rs` (+527), `crates/graph/src/lib.rs` (+2), `crates/graph/tests/cycle_groups_test.rs` (+564), `tasks/m235-t2a-journal.md` (+167) | Full diff read from the checkout. Implementation+test output 1093 lines against the ~1200 sizing bound. `lib.rs` gained exactly `pub mod cycle_groups;` (:6) and `pub use cycle_groups::*;` (:15), mirroring the `graph` wiring. Frozen plan sec1.3 contract reproduced verbatim. Lowlink push/advance/pop audited against `scc.rs:100-241` with emit-before-propagate preserved. Production calls `self.adjacency()` (:42); the only other rebuild is the cited test-side mirror. Zero production call sites for the new API. No `Rc`/`RefCell`/`Cell`/`thread_local`/`static mut`. |
| `7c94af3` | `tasks/m235-t2a-handoff.md` (+562), `tasks/m235-t2b-handoff.md` (+345), `tasks/milestone_ledger.md` (+5), `tasks/m235-t2a-journal.md` (+9/-4) | Planning material plus the journal's own SHA record. Nothing outside the authorized set. Verdict finding 8: the subject names the plan file, which this commit does not contain (the plan was already tracked at or before `2708425`). |

## 7. Milestone state

T2-A is APPROVED and independently shippable. `crate:graph` now carries the
milestone's `CycleGroup` currency additively and warning-free, with its own
nine-test contract suite; the workspace is green at `1165 / 0 / 4 / 87` with
the delta fully attributed.

`tasks/m235-t2b-handoff.md` (T2.3) is unblocked and should absorb verdict
findings 1, 2, and 3 as required test work. Phase 3 (T3.1 report/policy/CLI
rewiring, T3.2 UI DTO) and Phase 4 (T4.1) remain UNAUTHORIZED.

Findings 4, 6, and 7 are handoff/plan artifact text fixes for the next
planning touch. Findings 8, 9, and 10 are record and configuration follow-ups
for `tasks/milestone_ledger.md`. Ledger rows F9, F10, and F12 all reproduce
exactly as recorded and need no amendment; finding 9 adds the measured
`core.autocrlf=true` mechanism and the stat-cache recurrence to F12.

Housekeeping confirmed and left alone: `tasks/m235-t1-verdict.md` (326 lines)
and `tasks/m235-execution-results.md` (132 lines) both exist and are intact,
both untracked, both declared into T2-B's commit scope. Not committed by this
review. With this addendum, T2-B has three untracked files to scope.