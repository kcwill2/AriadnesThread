# M23.5 Execution Results - Phase 1 (T1.1 + T1.2, crate:cluster)

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Range:** `2708425..e2a05e2` (branch `tarjan-s-scc`, worktree
`.kilo/worktrees/tarjan-s-scc`)
**Verdict:** `tasks/m235-t1-verdict.md` - APPROVED, 0 blocking / 12 non-blocking
findings
**Scope note:** Phase 2 (CycleGroup, crate:graph) is not authorized and was not
reviewed, built, or demanded.

---

## 1. Canonical summary line (verbatim)

Workspace, reviewer-run:

```
1156 passed / 0 failed / 4 ignored / 86 suites
```

Recorded post-merge baseline for comparison: `1147 / 0 / 4 / 85`.
Delta: +9 passed, 0 failed, 0 ignored, +1 suite. Of the +9, +1 suite and +3
tests are the new `scc_iterative_parity` target. The residual +6 is not
attributable to this range (test counts are compile-time fixed; this range adds
one new test file with three test functions and the `scc.rs` diff is confined
above `#[cfg(test)]`), so the recorded baseline is stale - verdict finding 11.

## 2. Kickoff acceptance criteria (section 6), one by one

| # | Criterion | Outcome | Evidence |
|---|-----------|---------|----------|
| 1 | `crates/cluster/tests/scc_iterative_parity.rs` exists with oracle + equivalence assertions (fixtures + 200 seeded digraphs + 100k chain) | MET | File present, 425 lines. `python tools/testrun.py --distill --timeout 1100 -p ariadne-cluster --test scc_iterative_parity` -> `3 passed / 0 failed / 0 ignored / 1 suites`. Contains 9 mirrored unit fixtures, `SEEDED_GRAPH_COUNT = 200` over documented seed `0x0DA7_ED5E_ED12_3456`, and `CHAIN_NODE_COUNT = 100_000` in open and fused-cycle shapes. |
| 2 | Iterative frame machine passes the oracle on all fixtures + stress at default stack (64 KiB) | MET | Both production chain runs use `SMALL_STACK_BYTES = 64 * 1024` on a `Builder::stack_size` thread and complete; `3 passed / 0 failed / 0 ignored / 1 suites`. Oracle comparison runs at `ORACLE_STACK_BYTES = 64 MiB`, not the 2 MiB the handoff names - verdict finding 3, D2 accepted. |
| 3 | `cargo test -p ariadne-cluster` + `planner_characterization` + `planner_facts_parity` + `m00_facts_path_parity_test` all green, unedited | MET | `-p ariadne-cluster` -> `54 passed / 0 failed / 0 ignored / 8 suites`. `--test planner_characterization` -> `11 passed / 0 failed / 0 ignored / 1 suites`. `--test planner_facts_parity` -> `5 passed / 0 failed / 0 ignored / 1 suites`. `-p ariadne-integration-tests --test m00_facts_path_parity_test` -> `2 passed / 0 failed / 0 ignored / 1 suites`. Unedited is proven by the touch-set check: none of the three files appear in `git diff --name-only 2708425..e2a05e2`. |
| 4 | No R1/R2 hygiene violations; `encoding_guard.py --all` clean | MET | `python tools/hygiene_check.py` -> zero R1/R2 across the whole workspace; R3 warn-only. Two R3 warns are new, both in the frozen oracle (`crates/cluster/tests/scc_iterative_parity.rs:380`, `:411`, single-letter `w`). `python tools/encoding_guard.py --all` -> no output. `python tools/encoding_guard.py --staged` -> no output. |
| 5 | Planner output byte-identical (IV-8) | MET | All three byte/characterization locks green unedited (criterion 3 evidence). Intra-SCC sort (`first.0.cmp(&second.0)`), SCC emission order, and the seed-skip rule are textually identical to the pre-swap implementation read from `git show "dd3db11^:crates/cluster/src/scc.rs"`. |

## 3. Plan section 3 acceptance criteria, Phase 1 slice

| # | Criterion | Outcome | Evidence |
|---|-----------|---------|----------|
| 4 (partial) | Deep-chain stress (100 000-node) passes at default stack in both crates; equivalence oracles green on fixtures + 200 seeded digraphs each | MET for cluster; graph crate deferred to Phase 2 | cluster side green at 64 KiB (criterion 2 above). The graph-crate half is T2.1/T2.3 and is not authorized. |
| 5 | Planner output byte-identical; closure equivalence green; export byte-stable | MET | Planner locks green unedited. Closure equivalence and export/golden suites are inside the workspace run: `1156 passed / 0 failed / 4 ignored / 86 suites`, zero failures. |
| 8 | Workspace green + hygiene gate + encoding guard + reviewer APPROVED | MET with a recorded exception | Workspace green; hygiene and encoding clean; APPROVED. Exception: no UI smoke and no bench comparison could be run - section 5 below. |

Plan criteria 1, 2, 3, 6, 7 are Phase 2/3/4 concerns and were neither reviewed
nor demanded.

## 4. Workspace-wide acceptance pass (reviewer duty)

| Gate | Ran? | Result |
|------|------|--------|
| `python tools/testrun.py --baseline` | attempted, could not run | `usage error: missing value for --baseline` (exit 2). `--baseline` requires a FILE and no `N/M/K/S` baseline file exists in the repo. Substituted the invocation it wraps: `python tools/testrun.py --distill --timeout 2400` -> `1156 passed / 0 failed / 4 ignored / 86 suites`. |
| workspace test run (substitute for the above) | yes | `1156 passed / 0 failed / 4 ignored / 86 suites`; capture `target/testrun-capture.txt`, 2044 lines, 86 result lines, 69 test binaries + 17 doc-test groups. |
| `python -m unittest discover -s tools -p "*_tests.py"` | yes | `Ran 184 tests in 19.677s` then `OK`. |
| `python tools/hygiene_check.py` (workspace) | yes | No R1/R2. R3 warn-only; ~120 pre-existing warns plus the 2 new oracle warns. |
| `python tools/encoding_guard.py --all` | yes | clean, no output. |
| `python tools/encoding_guard.py --staged` | yes | clean, no output (tree clean, so vacuous by construction). |
| `python tools/theme_token_check.py --check-added --base 2708425` (frozen lock) | yes | clean, no output. No UI file is in the touch-set. |
| `python tools/checkpoint_guard.py status` | yes | HEAD `e2a05e2`, branch `tarjan-s-scc`, clean, no untracked files. |
| `python tools/ui_smoke.py` | NO - refused | Two independent direct forms attempted. Verbatim both times: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` |
| `python tools/bench_compare.py` | NO - refused | Verbatim: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` |

### Explicit statement of what was not run

- `python tools/ui_smoke.py` did not run.
- `python tools/bench_compare.py` did not run.
- `python tools/testrun.py --baseline` did not run in its documented form; the
  workspace test run it wraps was run instead.

Root cause for the two refusals is a configuration gap, established by reading
the reviewer agent definition: its bash allowlist covers `testrun.py`,
`distill.py`, `hygiene_check.py`, `encoding_guard.py`, `theme_token_check.py`,
`commit_scoped.py`, `checkpoint_guard.py`, `python -m unittest*` and read-only
git, but not `ui_smoke.py` or `bench_compare.py`, while the same file instructs
the reviewer to run them. Neither command was routed through `distill.py` to
evade the denial. Consequence: this pass provides no UI-runtime evidence and no
perf-regression evidence. Verdict finding 10.

Two reviewer-side commands were also refused and are recorded for completeness;
neither is a gate. `git status --short` was refused (the `git_git_status` tool
was used instead and reported a clean tree), and a PowerShell
`git diff ... | Select-String` pipeline was refused (the plain `git diff` form
was used instead).

### Worktree side effect of this acceptance pass (important for the next commit)

The worktree was clean at session start (`checkpoint_guard.py status`: no dirty
files, no untracked files). After the workspace run it is not:
`git status` reports `tests/fixtures/basic/artifacts/includes/report.json` as
modified, while `git diff` and `git diff --numstat` show zero content hunks and
git emits `LF will be replaced by CRLF the next time Git touches it`. An
integration test that uses `tests/fixtures/basic` as its repo root regenerated
that artifact with CRLF terminators; `.gitattributes` pins `text eol=lf` only
for `crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json`, so this path
is unpinned. Content is identical after normalization, so this is not a
behavioral regression and it is not caused by this handoff (the touch-set is
three cluster files). Operational consequence: the next `commit_scoped.py` in
this worktree must carry an explicit `--scope` so the file cannot be swept into
a Phase 2 commit, and the LF pin set should be extended. Verdict finding 12.
The reviewer did not repair it - outside both the edit grant and the touch-set.

The only other worktree change is this reviewer's two untracked deliverables,
`tasks/m235-t1-verdict.md` and `tasks/m235-execution-results.md`.

## 5. Per-commit execution record

| Commit | Files | Change | Reviewer verification |
|--------|-------|--------|-----------------------|
| `dd3db11` | `crates/cluster/src/scc.rs`, `crates/cluster/tests/scc_iterative_parity.rs` | +613 / -69; scc.rs 455 lines, parity test 425 lines | Full `git show` diff read. Diff confined above `#[cfg(test)] mod tests`; fixture module untouched, so no dual representation. Pre-swap blob read via `git show "dd3db11^:crates/cluster/src/scc.rs"` and compared field-by-field against the frozen oracle: only semantics-preserving renames. Lowlink push/advance/pop audited against the recursive body (verdict 4.1). No `Rc`/`RefCell`/`Cell`/`thread_local`/`static mut` in production. All functions and both files far under the 100-line / 1000-line locks. |
| `e2a05e2` | `tasks/m235-t1-journal.md` | +132 | Journal evidence is digest-shaped (canonical lines, exit codes, capture paths), zero raw dumps, two verbatim refusals logged. Gaps recorded as verdict finding 8 (own SHAs not recorded; cited Red capture path overwritten by later runs). |

Touch-set enforcement: `git diff --name-only 2708425..e2a05e2` returned exactly
the three declared paths and nothing else. No commit landed outside the
declared touch-set.

## 6. Deviations adjudicated

D1 accepted, D2 accepted, D3 accepted. Full basis in verdict section 7. The
reviewer reproduced D3 directly: the handoff's literal gate form returns
`0 passed / 0 failed / 0 ignored / 7 suites` (exit 0), i.e. it selects nothing.

## 7. Milestone state

Phase 1 is APPROVED and independently shippable. The ledger row and the T2
handoff remain the planner's next action. Six handoff/plan/tooling defects and
two configuration gaps are routed as follow-ups in verdict section 8; none of
them requires reopening Phase 1.

The recorded workspace baseline `1147 / 0 / 4 / 85` must be re-pinned to a
measured value (1156 / 0 / 4 / 86 at this commit) before T4.1 closeout.