# M23.5 T4.1 Handoff -- Phase 4 integration closeout

Base `dcd681b`, branch `tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`.
Every implementation unit of this milestone is APPROVED. This unit changes no
production behaviour; it closes the milestone's open bookkeeping and runs the
plan's Phase 4 verification (plan `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md`
sec2 Phase 4).

## 0. OPERATOR PREREQUISITES -- the closeout CANNOT discharge these

Three obligations belong to no role, including you. **M23.5 may not be declared
CLOSED until all three are done**, and an "every unit APPROVED" reading is not
"done":

1. **F10 -- amend the permission allowlist, then run the two refused gates.**
   `python tools/ui_smoke.py` and `python tools/bench_compare.py` are refused
   verbatim by the catch-all rule. Until an operator amends the allowlist and
   runs both, there is ZERO runtime evidence that the UI cold-boots, routes,
   serves `Cache-Control: no-store` and returns a group-count report, and NO
   verified perf evidence after T3-D changed what `cycle_count` means. **Do not
   route either through `distill.py` to evade the denial.**
2. **The kilo-memory mirror of `tasks/milestone_ledger.md`.** The ledger's own
   authority rule: memory MIRRORS the ledger and must move in the same commit.
   Not delegable to implementer, reviewer or planner.
3. **The worktree census disposition.** This milestone ran on branch
   `tarjan-s-scc` in worktree `.kilo/worktrees/tarjan-s-scc` and is **NOT
   merged to `main`**. It must end merged, or abandoned with a ledger row.

A fourth closeout item, also not yours: `milestone-lifecycle` requires a
committed walkthrough on this branch and `walkthroughs/**` holds **no M23.5
deck** (planner-verified: only `ui-theming-`, `m21-`, `m19-`). Generate it via
`walkthrough-commit` so it cannot be lost.

## 1. Objective and invariants

**Objective:** close the milestone's remaining bookkeeping -- the F12 config
fix, the F13 stale-identifier rename, the golden and plan-file confirmations,
the record landing, and the bench timing capture -- without touching a byte of
production behaviour.

**Invariants you must preserve:**

- `crates/graph`, `crates/report`, `crates/pipeline`, `crates/cluster`:
  **byte-unchanged.** This milestone's cycle substrate is frozen; you have no
  reason to be in them.
- Exactly one public include-element builder: `crates/ui/src/dto.rs:621`
  `pub fn build_include_elements`, third param `&[CycleGroup]`; the tail
  `emit_include_elements` (`:643`) is private; `crates/ui/src/lib.rs` stays a
  bare `pub use dto::*;`. Do not add an alias, shim or deprecation re-export.
- `crates/graph/tests/cycle_groups_equivalence_test.rs` keeps its frozen
  `old_detect_cycles` (`:90`) and `old_dfs_cycle` (`:116`) oracle. **Deleting
  it deletes the evidence that the SCC path reproduces DFS semantics.** It is
  the one deliberate dual representation in the workspace, by design.
- The `UNVERIFIED` perf paragraphs stay verbatim and unabridged:
  `crates/cli/tests/payload_tail_bench.rs:80-87` and
  `crates/cli/src/commands/ui.rs:115-119`.
- Test counts do not move: `--test ui_test` stays `186/0/0/1`,
  `-p ariadne-ui` stays `186/0/0/3`, the workspace line stays
  `1180/0/4/88`. Your rename in sec3 moves a test's *name*, not its count.

## 2. Verified starting state

Every line below was measured by the planner at `dcd681b`. Do not re-derive
them; do not contradict them without evidence.

| claim | how it was verified |
|---|---|
| HEAD is `dcd681b` on branch `tarjan-s-scc` | `git log` / `git status` |
| the ONLY dirty path is `tests/fixtures/basic/artifacts/includes/report.json` | `git status` |
| that fixture's diff is EMPTY -- stat-entry churn, not content drift | `git diff` returned no hunks |
| the ONLY untracked files are `tasks/m235-t3e2-verdict.md` and `tasks/m235-t3e2-execution-results.md` | `git status` |
| `.gitattributes` is 7 lines with exactly two pins: `crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json`, both `text eol=lf` | read the file |
| `crates/ui/src/dto.rs:621` is the only `pub fn build_include_elements`; `:643` `fn emit_include_elements` has no `pub` | grep |
| `crates/ui/tests/ui_test/export_order.rs` stale identifiers: `legacy_elements` `:311`, `grouped_elements` `:318`, test name `the_parity_comparison_detects_a_cycle_classification_difference` `:298` | grep |
| `crates/graph/tests/cycle_groups_equivalence_test.rs:90` / `:116` hold the frozen oracle; module doc `:4` says so | grep |
| `crates/transform/src/engine/mod.rs:133` `detect_cycles_in_units` is private, called at `:72`, unrelated subsystem | grep |
| `tests/golden/**` holds 30 files, incl. `basic/includes_report.json` and `expensive/includes_report.json` | glob |
| export determinism is already locked twice, both NON-ignored: `payload_tail_bench.rs:571` `export_bytes_are_identical_across_two_separate_processes`, and `crates/ui/tests/ui_test/graph_api.rs:742` `include_elements_export_is_stable_across_two_calls_with_cycle_fixtures` | read + grep |
| the 520-header fixture (`payload_tail_bench.rs:107` `HEADER_COUNT = 520`, `FANOUT_BASE = 10`) is cycle-bearing by construction -- the +10 stride mod 520 closes after 52 steps -- and is exercised ONLY by `payload_tail_phase_timing`, which is `#[ignore]` (`:444`) and asserts payload sizes, NOT determinism | read |

**Explicitly NOT verified -- you must measure it, do not assume it:** whether
the plan file is git-tracked; whether any golden needs re-pinning; the bench
timings; whether the F12 pin actually works.

## 3. What to do

**3.1 F12 -- extend the `.gitattributes` pin set (the one real edit).**
Add ONE line, keeping the file's existing comment style:

```
tests/fixtures/**/artifacts/**/*.json text eol=lf
```

Do not touch the artifact writer and do not touch any other pin. **Proof
required:** after sec4 gate 2 (a full test pass), `git status --porcelain` for
`tests/fixtures/basic/artifacts/includes/report.json` must return **empty**.
That is the whole point of the gate: the churn is *caused* by a full test run,
so only a full test run can disprove the fix.

**3.2 Goldens -- verify, do NOT re-pin.** Run the gate, confirm
`git diff --stat HEAD -- tests/golden` is empty and that both pins in
`.gitattributes` are intact after your 3.1 edit. The reviewers established the
goldens are **cycle-free**, so a re-pin should be UNNECESSARY. **Touching a
golden is a finding, not a fix.** If a golden does differ, STOP and report it.

**3.3 F13 -- rename P2's three stale identifiers** in
`crates/ui/tests/ui_test/export_order.rs` (one file, identifier-only edits):
`legacy_elements` -> a name that says what it now holds (the `&[]` call at
`:311`), `grouped_elements` -> the group-bearing call at `:318`, and the test
name at `:298` off the word "parity" (P1 is a frozen-expectation lock now, not
a parity comparison). **Edit no assertion, no argument, no expected value, and
no other file.** The count pin `186/0/0/1` must survive the rename.

**3.4 Plan file -- confirm and land if needed.** Repo convention
(`repo.tracks_plan_files`) is that plan files are tracked, and the plan file is
absent from `git status`'s untracked list. Confirm it is tracked and commit it
alongside the code if it is not. Cite it as the contract of record.

**3.5 Export determinism -- REPORT, do not build.** The plan asks for a
"determinism re-check on the SCC-dense 520-header bench fixture". Measured
reality: the two determinism locks that exist use *different* fixtures (a
2-header SCC and a two-disjoint-group graph), and the 520-header fixture has
**no determinism assertion at all**. Cycle-bearing export determinism is
already locked inside the `1180/0/4/88` run. Adding a new cross-process lock on
the 520-header fixture is a NEW TEST and therefore a new RGB unit, not closeout
work. **Report the discrepancy as a finding; do not write the test.**

**3.6 Bench timings.** Run the ignored bench (gate 6) and record what it prints.
Keep the `UNVERIFIED` label and its surrounding text intact and unabridged,
including the refusal sentence. **Do NOT reconcile `5.1` (`ui.rs:103`), `5.4`
(`payload_tail_bench.rs:41`) and `5.6` (`:40`).** Reconciling them would
manufacture unverified perf evidence while F10 is open. Report the numbers you
observed next to those three and change nothing.

**3.7 Land the untracked records.** `tasks/m235-t3e2-verdict.md` and
`tasks/m235-t3e2-execution-results.md` are untracked because planners and
reviewers hold read-only git. Land both in a `tasks/**`-only commit -- an
out-of-band `tasks/**`-only landing is the standing, precedent-backed exception
to the touch-set; **the first one that carries a code path is a blocker.**

## 4. Touch-set (hard; any edit outside = blocker, report don't fix)

In scope:

- `.gitattributes`
- `crates/ui/tests/ui_test/export_order.rs` (one file, identifiers only)
- `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` (only if untracked)
- `tasks/m235-t4-journal.md` (checkpoint journal, always in-set)
- `tasks/m235-t4-execution-results.md`, `tasks/m235-t3e2-verdict.md`,
  `tasks/m235-t3e2-execution-results.md` (record landing, 3.7)

You MUST NOT touch: `crates/graph/**`, `crates/report/**`,
`crates/pipeline/**`, `crates/cluster/**`, `crates/cli/**`,
`crates/ui/src/**`, `crates/ui/tests/**` other than `export_order.rs`,
`tests/golden/**`, `tests/fixtures/**`, `tools/**`, `Cargo.toml`, `Cargo.lock`,
`walkthroughs/**`. No `git stash`, no branch change, no amend.

Commit with an EXPLICIT `--scope` on every `commit_scoped.py` (never
`git add .`); a sloppy scope silently stages the F12 fixture.

## 5. Red, then Green

**Red (both must fail before you change anything; report the exact output):**

1. F12: `git status --porcelain -- tests/fixtures/basic/artifacts/includes/report.json`
   returns a ` M` line, while `git diff` for the same path returns no hunks.
   That combination is the churn signature, not a content change.
2. F13: a grep for `legacy_elements|grouped_elements|the_parity_comparison`
   over `crates/ui/tests/ui_test/export_order.rs` returns **5** hits
   (lines 298, 311, 318, 325, 326).

**Green:** 3.1 through 3.7 complete, and BOTH Red signatures gone: the porcelain
line is empty after a full test run, and the grep returns zero hits.

## 6. Gates (9 invocations, each in its own turn)

1. `python tools/checkpoint_guard.py status` -- census + the dirty set, before
   any edit.
2. `python tools/testrun.py --distill --timeout 3600` -- **the one
   workspace-wide gate in this handoff, and it is here for a specific reason:
   the F12 churn is caused by a full test run, so only this gate can prove or
   disprove 3.1.** Expect `1180 passed / 0 failed / 4 ignored / 88 suites`.
3. `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test`
   -- expect `186 passed / 0 failed / 0 ignored / 1 suites`. This is the suite
   your 3.3 rename lives in; a moved count means you broke something.
4. `python tools/hygiene_check.py` -- zero R1/R2 lines.
5. `python tools/encoding_guard.py --all`, then `--staged` before committing --
   both silent.
6. `python tools/distill.py -- cargo test -p ariadne-cli --test payload_tail_bench --release -- --ignored --nocapture --test-threads=1`
   -- the bench run for 3.6. By far the most expensive gate; it is REQUIRED,
   not optional, and the timing row is meaningless without it.
7. `git status --porcelain -- tests/fixtures/basic/artifacts/includes/report.json`
   -- **must print nothing.** If bash refuses this read, report it as a
   deviation; do not route it around.
8. `git diff --stat HEAD -- tests/golden` -- **must print nothing** (3.2).
9. `python tools/commit_scoped.py --scope <touch-set glob> ... -m "chore(m23.5): ..."`
   -- one commit for config + code, one for the `tasks/**` record landing.

Report the canonical `testrun.py` line verbatim. Never paste raw cargo or git
output; cite command + summary line + exit code.

## 7. Hygiene, sizing, discipline

Functions < 100 lines, files < 1000 lines, meaningful names, no banned
identifiers, no single-letter locals, no source line over 2000 characters.
Authored content is pure ASCII.

**Sizing (this unit, measured against the handoff law):**

| bound | limit | this unit |
|---|---|---|
| scope | 1 crate or 1 module dir | **1 crate** (`crates/ui`, one TEST file) + repo-root config; **0 production files** |
| gates | ~10 scoped | **9 invocations**, 1 of them workspace-wide for the F12 proof |
| output | ~1200 lines | **~40 lines of diff** (1 config line, 3 identifier renames, records) |

**UTF-8:** never `Set-Content` / `Out-File` / `>` / `Get-Content -replace`. Use
the Read/Edit/Write tools only.

**Journal:** append a 1-3 line checkpoint to `tasks/m235-t4-journal.md` after
each gate. A dead session is never resumed -- a successor runs
`python tools/checkpoint_guard.py resume --spec tasks/m235-t4-handoff.md --scope ... --cmd ...`.

**If any claim in this file turns out to be false:** STOP at preflight, report
it with the command and its verbatim output, and change nothing. That is a
success, not a failure -- a predecessor unit held an entire unit at preflight on
exactly that basis and was approved for it.

## 8. Report back

1. Commit hashes + `git show --stat` per commit.
2. Red evidence: the two Red signatures, verbatim.
3. Green evidence: digests + canonical lines + capture paths, especially the
   `186/0/0/1` count surviving your rename.
4. Deviations + reason.
5. Findings, split into fixed-now vs deferred, and specifically: the 3.5
   determinism discrepancy, and whether the plan file was tracked.
6. Out-of-scope observations, report-only.

## 9. Reviewer gate reference

Acceptance is adjudicated by the `review-gate` skill. Its commit-vs-touch-set
check enforces sec4, so the touch-set must list every path you legitimately
changed. The reviewer's acceptance pass -- not this handoff -- owns the
workspace-wide sweep beyond gate 2, and will re-run every gate in sec6 against
your commits.