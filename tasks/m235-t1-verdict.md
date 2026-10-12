# M23.5 T1 Verdict - Symbol SCC De-recursion (crate:cluster)

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Handoff:** `tasks/m235-t1-handoff.md` (T1.1 + T1.2, Phase 1)
**Range reviewed:** `2708425..e2a05e2` (base `2708425`, HEAD `e2a05e2`, branch `tarjan-s-scc`)
**Commits:** `dd3db11` (oracle + frame machine), `e2a05e2` (journal)

## VERDICT: APPROVED

0 blocking findings. 12 non-blocking findings: 4 defects in the handoff
artifact itself (planner), 3 implementer-record gaps, 1 planner tool-wording
defect, 1 stale workspace baseline, 1 configuration gap that narrows this
reviewer's acceptance pass, and 1 tracked-artifact dirtying caused by
pre-existing test behaviour and surfaced by this pass.

Phase 2 (CycleGroup, crate:graph) was not reviewed and is not authorized.

---

## 1. Ground truth

`python tools/checkpoint_guard.py status`: HEAD `e2a05e2`, branch
`tarjan-s-scc`, worktree clean, no untracked files. The only commit on `main`
absent from this branch is `e276910` (a Python tooling fix); no Rust surface
differs, so the recorded post-merge cargo baseline applies to this tree.

## 2. Commit-versus-touch-set check (PASS)

Declared touch-set: `crates/cluster/src/scc.rs`,
`crates/cluster/tests/scc_iterative_parity.rs`, `tasks/m235-t1-journal.md`.

`git diff --name-only 2708425..e2a05e2` returns exactly those three paths. No
path outside the touch-set. Per-commit:

| Commit | Paths | Inside touch-set |
|--------|-------|-----------------|
| `dd3db11` | `crates/cluster/src/scc.rs`, `crates/cluster/tests/scc_iterative_parity.rs` | yes |
| `e2a05e2` | `tasks/m235-t1-journal.md` | yes |

## 3. Per-commit review, oldest first

### `dd3db11` - oracle freeze + iterative frame machine

`crates/cluster/src/scc.rs` (+188 / -69, 455 lines total). The diff is
confined to the region strictly above `#[cfg(test)] mod tests`; the fixture
test module does not appear in the diff at all, so it is byte-unchanged. The
recursive `strongconnect` and the inlined traversal in
`compute_symbol_sccs_from_adj` are replaced by `TarjanFrame`, `TarjanRun`,
`run_tarjan`, and the `TarjanRun` methods. Public API, `SymbolScc`,
`build_symbol_adjacency`, and the `compute_symbol_sccs` wrapper are untouched.

`crates/cluster/tests/scc_iterative_parity.rs` (new, 425 lines). Contains the
frozen recursive oracle, 9 mirrored unit fixtures, a 200-graph seeded sweep,
and the 100 000-node chain stress in open and fused-cycle shapes.

Oracle authenticity (the "frozen before the swap" requirement): the oracle was
compared field-by-field against the pre-swap blob read from
`git show "dd3db11^:crates/cluster/src/scc.rs"`. The entry head, the neighbor
loop, both `min` update rules, the root test, the pop-through-vertex loop, the
lexicographic intra-SCC sort, and the seed-skip rule are all identical. The only
differences are semantics-preserving renames (`strongconnect` ->
`oracle_strongconnect`, `v`/`w`/`scc` -> `vertex`/`popped`/`group`) and the
`#[allow(clippy::too_many_arguments)]` attribute the 8-argument shape needs.
The oracle is the pre-swap implementation, not a re-derivation.

### `e2a05e2` - journal

Adds `tasks/m235-t1-journal.md` only. Digest-shaped evidence table with
canonical lines, exit codes, and capture paths; no raw output dumps; two
verbatim refusals recorded. Compliant with the distillation contract.

## 4. Audit focus (handoff section 4)

### 4.1 Lowlink update points - CORRECT

Audited against the recursive body read from the pre-swap blob.

- **Push** (`enter_vertex`): assigns `index` and `lowlink` from the shared
  counter, increments it, pushes to the Tarjan stack, sets `on_stack`, opens a
  frame at neighbor 0. Mirrors the recursive head exactly.
- **Advance** (`advance_frame`): unindexed neighbor -> `enter_vertex` and
  return `Descended` (the tree edge, whose propagation is deferred to pop);
  indexed neighbor on the stack -> `relax_lowlink_toward_index` applies
  `lowlink[v] = min(lowlink[v], index[w])`. Mirrors both loop arms.
- **Pop** (`retire_frame` -> `emit_group_if_root` then `finish_child`): emission
  first, then `lowlink[parent] = min(lowlink[parent], lowlink[child])`. The
  recursive body emits inside the callee and applies `min` in the caller after
  the call returns; the event ordering is identical.
- Frame discipline: frames are pushed only from `advance_frame` on the top
  frame and popped LIFO, so `frames.last()` after a pop is always the
  recursed-into parent. A vertex below a descendant on the Tarjan stack cannot
  be popped by that descendant's emission, so `finish_child`'s parent is always
  still on the stack.
- Root test uses `lowlink.get(v) != indices.get(v)`, the same comparison the
  recursive body used.
- The failure mode the handoff names (propagating on call return instead of
  frame pop) is structurally absent: the only writer of a parent's lowlink from
  a child is `finish_child`, reached solely from `retire_frame`.

### 4.2 Tie-break determinism - VERIFIED

Parity is asserted as full `Vec<SymbolScc>` equality, so both SCC emission
order and intra-SCC member order are locked, not just set membership. Seed
`0x0DA7_ED5E_ED12_3456` is documented in the test, re-seeded per graph ordinal,
using Knuth MMIX constants (platform-independent, no RNG).

### 4.3 No dual representation - CONFIRMED

Production `scc.rs` contains no `#[cfg(test)]` copy of itself. The diff proves
the pre-existing `mod tests` (public-API fixture tests, not an oracle) is
untouched, and the oracle exists only in the integration test crate.

### 4.4 Send + Sync - CONFIRMED

No `Rc`, `RefCell`, `Cell`, `thread_local`, or `static mut` anywhere in
production `scc.rs` (the only textual match is the doc comment asserting their
absence). All frame state is plain owned or borrowed collections
(`HashMap`/`HashSet`/`Vec` of `&'a SymbolId` and `usize`). Adjacency is reached
only through `&HashMap` and `HashMap::get`; there is no interior mutability, so
no lock is needed and the type is `Send + Sync` by construction. Frame state is
function-local and never crosses a thread boundary; the rayon call site
`propose_extractions(&self, ...)` requires `Planner: Sync` and compiles.

### 4.5 Stack-size verification

- Production side: both 100k runs use `SMALL_STACK_BYTES = 64 * 1024` (65536,
  i.e. 64 KiB) on a `Builder::stack_size` thread. Verified in source and by a
  green run: a 100 000-deep traversal cannot complete in 64 KiB by recursion
  (100 000 frames at even 16 bytes each is ~1.6 MB), so the passing test is
  itself the IV-7 proof.
- Oracle side: `ORACLE_STACK_BYTES = 64 MiB`, not the 2 MiB the handoff and plan
  specify. See finding 3; D2 accepted.
- The pre-swap abort at 64 KiB is asserted by the implementer; its capture was
  overwritten by later runs and is not on disk. See finding 8.

## 5. Fresh verification run by the reviewer

| Gate | Result |
|------|--------|
| `-p ariadne-cluster --test scc_iterative_parity` | `3 passed / 0 failed / 0 ignored / 1 suites` |
| handoff literal gate form `-- scc_iterative_parity` | `0 passed / 0 failed / 0 ignored / 7 suites` (vacuous; confirms D3) |
| `-p ariadne-cluster` (full crate) | `54 passed / 0 failed / 0 ignored / 8 suites` |
| `-p ariadne-cluster --test planner_characterization` | `11 passed / 0 failed / 0 ignored / 1 suites` |
| `-p ariadne-cluster --test planner_facts_parity` | `5 passed / 0 failed / 0 ignored / 1 suites` |
| `-p ariadne-integration-tests --test m00_facts_path_parity_test` | `2 passed / 0 failed / 0 ignored / 1 suites` |
| workspace (`testrun.py --distill`) | `1156 passed / 0 failed / 4 ignored / 86 suites` |
| `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 184 tests ... OK` |
| `python tools/hygiene_check.py` | no R1/R2; R3 warn-only |
| `python tools/encoding_guard.py --all` | clean |
| `python tools/encoding_guard.py --staged` | clean |
| `python tools/theme_token_check.py --check-added --base 2708425` | clean |
| `python tools/testrun.py --baseline` | could not run; usage error (finding 7) |
| `python tools/ui_smoke.py` | could not run; refused (finding 9) |
| `python tools/bench_compare.py` | could not run; refused (finding 9) |

UNEDITED requirement: `planner_characterization`, `planner_facts_parity`, and
`m00_facts_path_parity_test` were not in the touch-set, so unedited is proven by
the touch-set check, and all three are green as measured.

## 6. Findings

All non-blocking. Severity labels: NB = non-blocking.

1. **NB, planner** - `tasks/m235-t1-handoff.md` section 2 publishes API
   signatures that do not exist in the tree: `SymbolPtr`, `&[Vec<usize>]`,
   `SymbolScc { members, is_cyclic }`, and
   `strongconnect_iterative(...) -> Vec<SymbolScc>`. The real API is
   `SymbolId`, `HashMap<&SymbolId, Vec<&SymbolId>>`, `SymbolScc { symbols }`.
   **D1 accepted.** Preserving the real signature is what IV-8 and the kickoff
   locks require, and the byte-locks confirm it. The handoff text should be
   corrected so a future reader does not read section 2 as an interface-change
   order.

2. **NB, planner** - the T1.1/T1.2 gate
   `python tools/testrun.py --distill -p ariadne-cluster -- scc_iterative_parity`
   is vacuous. Reproduced independently: `0 passed / 0 failed / 0 ignored /
   7 suites`, exit 0. The trailing argument is a test-name filter, and no test
   name contains the file name. **D3 accepted**; the implementer logged the
   vacuous run and switched to `--test scc_iterative_parity`, which selects the
   3 real tests. Consequence to record: as written, this gate could never have
   observed the Red phase either, so it was never a real gate.

3. **NB, planner** - the 2 MiB oracle-stack constant in the handoff (section 3)
   and the plan (section 2, T1.1) is arithmetically impossible at 100 000
   recursion levels: the oracle frame is 64 bytes of arguments plus live
   `usize` locals and a slice iterator, so 100 000 levels need roughly 6-13 MB.
   **D2 accepted.** `ORACLE_STACK_BYTES` was raised to 64 MiB. The
   IV-7-bearing production constant is unchanged at 64 KiB, and the oracle
   stack carries no behavioral contract, so nothing observable moved. Recommend
   the T2 handoff not repeat the 2 MiB figure for a 100k-deep oracle.

4. **NB, planner** - the T1.1 Red specification ("asserts
   `strongconnect_iterative` (not-yet-existent, returns empty) != oracle") is
   unimplementable: a test cannot call a function that does not exist, and
   `compute_symbol_sccs_from_adj` is the frozen public name. The implementer
   pointed the parity assertions at the live entry point instead and took the
   Red from the stack-overflow abort. That substitution is the only workable
   form and it did produce a genuine Red, but it should have been logged as a
   deviation rather than described as expected behaviour.

5. **NB, implementer** - `representative_cycle_for` from the handoff's
   five-helper refactor list was not stubbed. Justified and correctly logged:
   it is a Phase 2 / crate:graph concern, and an unused private function would
   emit `dead_code`, contradicting the handoff's own zero-warning build
   evidence. The other four named helpers (`run_tarjan`, `advance_frame`,
   `finish_child`, `emit_group_if_root`) exist verbatim as named.

6. **NB, implementer** - no function named `strongconnect_iterative` exists; the
   role named in handoff section 2 is filled by `run_tarjan` plus the
   `TarjanRun` methods. Covered by the same class as finding 1 and by the
   authoritative decomposition in section 3, but recorded so the ledger carries
   the name divergence.

7. **NB, implementer** - R3 warn: single-letter local `w` at
   `crates/cluster/tests/scc_iterative_parity.rs:380` and `:411`, both inside the
   frozen oracle copy. R3 is warn-only and these are precisely the locals whose
   verbatim preservation is the point of the oracle. All names introduced by
   non-oracle code are intention-revealing.

8. **NB, implementer** - `tasks/m235-t1-journal.md` "Commit list" section says
   the SHAs are recorded elsewhere ("see implementer report"), so the journal
   does not carry the commits it describes; the reviewer had to recover them
   from `git log`. Separately, the journal's Red evidence capture path
   (`target/testrun-capture.txt`) has been overwritten by every subsequent run,
   so the stack-overflow abort it cites is asserted rather than retained. IV-7
   does not depend on it (see 4.5), but the journal should say so and should
   name its own SHAs.

9. **NB, planner / configuration** - the workspace gate
   `python tools/testrun.py --baseline`, as written in `AGENTS.md`, the plan,
   the handoff, and the reviewer agent definition, is not runnable: `--baseline`
   requires a FILE argument and no `N/M/K/S` baseline file exists anywhere in
   the repository. Verbatim: `usage error: missing value for --baseline`
   (exit 2). The reviewer ran the equivalent workspace invocation the flag
   wraps and compared by hand. A baseline file should be committed so the
   gate is runnable and the delta is reviewable in the diff.

10. **NB for this handoff; blocking for milestone closeout - configuration** -
    `python tools/ui_smoke.py` and `python tools/bench_compare.py` could not be
    executed. Verbatim, both times: `The user has specified a rule which
    prevents you from using this specific tool call. Here are some of the
    relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
    Two independent direct forms were attempted for `ui_smoke.py`. Root cause
    established by reading the reviewer agent definition: its bash allowlist
    contains `testrun.py`, `distill.py`, `hygiene_check.py`,
    `encoding_guard.py`, `theme_token_check.py`, `commit_scoped.py`,
    `checkpoint_guard.py`, `python -m unittest*`, and read-only git - but not
    `ui_smoke.py` or `bench_compare.py`, while the same file instructs the
    reviewer to run them. This is a configuration gap, not an implementer
    defect, and it was not worked around by routing the commands through
    `distill.py`. Consequence: this acceptance pass covers no UI smoke and no
    perf regression. `compute_symbol_sccs` sits in the planner's per-header SCC
    hot path, so the iterative machine is a real (small) perf surface; the gate
    designed to measure it could not run here. The orchestrator must add both
    commands to the reviewer allowlist, or run them in an authorized session,
    before Phase 2 proceeds.

11. **NB, planner - stale workspace baseline** - measured workspace result is
    `1156 passed / 0 failed / 4 ignored / 86 suites` against the recorded
    post-merge baseline `1147 / 0 / 4 / 85`: +9 passed, 0 failed, 0 ignored,
    +1 suite. The +1 suite is exactly the new parity target and +3 of the +9 are
    its tests. The remaining +6 cannot have been produced by this handoff: test
    counts are fixed at compile time by `#[test]` attributes, this range adds
    exactly one new test file with exactly three test functions, and the
    `scc.rs` diff is confined strictly above `#[cfg(test)]`. The recorded
    baseline is therefore stale. It must be re-pinned to a measured value before
    T4.1 closeout, and finding 9's baseline file is the mechanism.

12. **NB, pre-existing test behaviour (surfaced by this acceptance pass)** - the
    reviewer's workspace run dirtied a tracked file outside the touch-set:
    `tests/fixtures/basic/artifacts/includes/report.json` is reported modified by
    `git status`, while `git diff` and `git diff --numstat` show zero content
    hunks and git emits `LF will be replaced by CRLF the next time Git touches
    it`. The artifact was regenerated by an integration test that uses
    `tests/fixtures/basic` as its repo root (`graph_io.rs` saves `report.json`
    into `artifacts/includes/`; `m3_include_graph_test` computes that artifact
    dir directly) and written with CRLF terminators. `.gitattributes` pins
    `text eol=lf` only for `crates/ui/tests/fixtures/*.json` and
    `tests/golden/**/*.json`, so this path is unpinned and `core.autocrlf`
    rewrites it on write. The worktree was clean at session start
    (`checkpoint_guard.py status`: no dirty files) and is dirty now. Not caused
    by this handoff - the touch-set is three cluster files and none of them is
    written by any test. Consequences to carry forward: (a) any later
    `commit_scoped.py` in this worktree must be given an explicit scope so this
    file cannot be swept in, (b) if it is ever staged under `autocrlf` its line
    terminators flip, which is the exact failure class the existing two LF pins
    were added to prevent, so the pin set should be extended to
    `tests/fixtures/**/artifacts/**/*.json` or the artifact writer should emit
    LF. The reviewer did not repair it - it is outside the edit grant and
    outside the touch-set.

Note (not a finding): the single-commit delivery of T1.1 + T1.2 means git
history alone cannot show the oracle predating the swap. Substantively this is
satisfied and was verified directly against the pre-swap blob (see section 3),
which is stronger evidence than commit ordering.

## 7. Deviation dispositions

| Deviation | Disposition | Basis |
|-----------|-------------|-------|
| D1 handoff section 2 pseudo-signatures | **accepted** | Real signature preserved; IV-8 byte-locks green unedited |
| D2 oracle stack 2 MiB insufficient, raised to 64 MiB | **accepted** | Production 64 KiB claim unchanged; 2 MiB provably impossible at 100k depth |
| D3 gate filter form vacuous | **accepted** | Vacuity reproduced by the reviewer; substantive gate used and green |
| `representative_cycle_for` stub omitted | **accepted** | Would emit `dead_code`; contradicts the handoff's zero-warning build gate |
| Red-phase shape substituted | **accepted with note** | Only workable form; should have been logged as a deviation (finding 4) |
| R3 single-letter `w` in the oracle | **accepted** | R3 warn-only; verbatim preservation is the requirement |

## 8. Follow-up dispositions

- Findings 1, 2, 3, 4, 9, 11 are handoff/plan/tooling defects. Route them to
  the T2 handoff authoring step and to `tasks/milestone_ledger.md` deferred
  findings; do not reopen Phase 1 for them.
- Finding 10 is a configuration repair for the five-role topology. It gates
  Phase 2 only in the sense that the next reviewer's acceptance pass will
  otherwise be incomplete again.
- Finding 12 is a repository-hygiene defect with an immediate operational
  consequence: this worktree is now dirty on a tracked artifact outside every
  handoff touch-set, so the next `commit_scoped.py` invocation must carry an
  explicit `--scope`. Route the LF-pin extension to
  `tasks/milestone_ledger.md` deferred findings.
- Finding 8 is an implementer journal-hygiene item for the T2 implementer
  brief: record own SHAs; do not cite an overwriteable capture path as
  retained evidence.
- Phase 2 (CycleGroup, crate:graph) remains unauthorized until the ledger row
  is written.