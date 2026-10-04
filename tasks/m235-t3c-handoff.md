# M23.5 T3-C Handoff - Scala Integration Consumers + IV-9 Acceptance Gate (tests/integration)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 3 of 4, unit C of 5. Plan task: **T3.1 consumer migration (IV-12)**
+ the explicit **IV-9 acceptance gate**.
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 3
(T3.1), sec1.2 IV-9 / IV-11 / IV-12, sec3 items 2 and 6
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`
**Journal:** `tasks/m235-t3c-journal.md` (you create it; record your own SHAs in it)

Every line reference was read out of the tree at `b950963`. **If a line
reference disagrees with the file you read, the FILE wins** -- report the drift.

---

## 0. PREFLIGHT

**Step 0.1** - `python tools/checkpoint_guard.py status`; record HEAD, branch,
dirty files, untracked files.

**Step 0.2** - the unpinned CRLF fixture. `tests/fixtures/basic/artifacts/
includes/report.json` is not covered by any `.gitattributes` pin and this
worktree has `core.autocrlf=true`, so a full test run re-dirties its **stat
entry** even when content is byte-identical (ledger deferred row **M23.5 F12**;
measured three times: T2-A finding 9, T2-B finding 10). If and only if
`git diff --numstat HEAD -- <path>` shows rows: restore with the Kilo `Read`
tool followed by the Kilo `Write` tool. NEVER `Set-Content`, `Out-File`, `>`,
`>>`, `Get-Content | -replace`. `git checkout --` / `git restore` are mutating
git and are DENIED.

**This unit runs the full C++ pipeline over two real fixture repos, so the
stat-entry re-dirty is LIKELY here.** Plan for it. Every `commit_scoped.py`
carries an EXPLICIT `--scope`, and that path must appear in NO commit.

**Step 0.3** - you have NO prerequisite unit. `detect_cycle_groups` already
exists and is already public at `b950963`. This unit may run at any point in the
Phase 3 order.

---

## 1. Objective and Invariants

### Objective

Migrate the two Scala integration consumers from the path representation to the
group representation, and turn `m3_include_graph_test`'s `cycle_count > 0`
assertion into an **explicit machine-checked gate** rather than an expectation.

### The two call sites, re-verified at `b950963`

**S1 - `tests/integration/tests/m20_scala_dependency_graph_test.rs:154`**, inside
`fixture_cycle_is_informational_under_tolerate_policy` (`:150-192`):

```rust
let cycles = graph.detect_cycles();
assert!(
    cycles.len() >= 1,
    "the deliberate models <-> services fixture cycle is detected"
);
```

**S2 - `tests/integration/tests/m22_scala_load_analysis_test.rs:442`**, inside
`auto_shaped_graph_keeps_the_cycle_informational` (`:437-454`):

```rust
let cycles = graph.detect_cycles();
assert!(
    !cycles.is_empty(),
    "the deliberate fixture cycle is present on the auto-shaped lane"
);
```

### The finding that makes this unit small

**Every other assertion in both tests survives the migration VERBATIM.** Do not
edit them. Read them and confirm this yourself before you touch anything:

- `m20:160-169` -- `generate_report_with_cycle_policies(&|_| TolerateWithinPackage)`,
  then `informational_cycle_count >= 1` and
  `informational_cycle_count == cycle_count`. Under group semantics every group's
  every member tolerates, so every group is informational: still equal.
- `m20:171-179` -- the all-`Reject` delegate yields
  `informational_cycle_count == 0`, and `blocking.cycle_count ==
  tolerated.cycle_count` (both are the group count).
- `m20:181-189` -- the skip-when-zero serde shape: the zero-count JSON must NOT
  contain `informational_cycle_count`; the non-zero JSON must.
- `m22:447-453` -- the same all-tolerate equality plus `> 0`.

This is the plan's IV-11 stance-preservation requirement, and it holds because
the ported classification rule (unit D) is **"a group is informational iff every
member file's policy is non-`Reject`"** -- verbatim the old per-path rule on the
natural domain. **The only change either test needs is the one call.**

### Invariants this unit establishes

- **IV-9 `cycle_count` naming survives.** `m3_include_graph_test:118-121` asserts
  `report.cycle_count > 0` on its own fixture, where `include/a.h` and
  `include/b.h` include each other -- one nontrivial SCC, so exactly one group,
  so `cycle_count == 1`. **That test must pass WITHOUT EDIT.** It is gate 1.
- **IV-12 Scala paths keep working.** M20 and M22 are pure consumers of the
  graph API; there is no Scala-side change. No file under `crates/lang-scala/**`
  is in scope.
- **The C++ artifacts stay byte-stable.** `m20:181-189` is the byte-stability
  lock for `informational_cycle_count`; `m23_symbol_isolation_test.rs:205`
  asserts `cycle_count == 0` on an acyclic fixture and must stay unedited and
  green.

### Invariants this unit must NOT disturb

- `tests/golden/**` and `tests/fixtures/**` are **byte-locked**. See sec7.
- No production code anywhere. This unit is two call sites and nothing else.

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

- `tests/integration/tests/m20_scala_dependency_graph_test.rs` (S1 only)
- `tests/integration/tests/m22_scala_load_analysis_test.rs` (S2 only)
- `tasks/m235-t3c-journal.md` (**new**, yours)

You MUST NOT touch: any other file under `tests/integration/**`, any other
directory under `tests/` (in particular `tests/golden/**` and
`tests/fixtures/**`), `crates/**`, `tools/**`, `.kilo/plans/**`, `Cargo.toml`,
`Cargo.lock`, `.gitattributes`.

**No `git add .`.** Every `commit_scoped.py` carries an EXPLICIT `--scope`, and
`tests/fixtures/basic/artifacts/includes/report.json` must appear in NO commit.

---

## 3. Green - the minimal implementation bound

S1 becomes:

```rust
let cycle_groups = graph.detect_cycle_groups();
assert!(
    !cycle_groups.is_empty(),
    "the deliberate models <-> services fixture cycle is detected as one entangled group"
);
```

S2 becomes:

```rust
let cycle_groups = graph.detect_cycle_groups();
assert!(
    !cycle_groups.is_empty(),
    "the deliberate fixture cycle is present on the auto-shaped lane as a group"
);
```

That is the whole unit. Use `is_empty()` rather than a `len()` comparison in S1:
the old assertion was `cycles.len() >= 1`, and under group semantics the count
drops from "number of reported paths" to "number of entangled groups", so the
stance to preserve is *non-empty*, not *at least one*. Say that reasoning in
your journal -- it is the one place where "preserve the asserted stance" means
re-expressing the comparison, and the reviewer will check that you understood
why rather than mechanically editing `cycles` to `groups`.

Where a name reads better than `cycles` for the surrounding prose, you MAY update
a local assertion message so it says "group" instead of "cycle". You may NOT
change an assertion's truth condition, and you may NOT change any assertion
listed in sec1.

---

## 4. Red - none, and you must say so rather than fabricate one

This unit is a two-line rewiring with **no new behaviour to drive Red**: both
call sites' replacements (`detect_cycle_groups`) already exist, are already
public, and are already exercised by `crates/graph`'s own two suites.

The handoff **pre-authorises you to state that plainly in your journal** --
"no Red for T3-C; the API it consumes is already green in
`crates/graph/tests/cycle_groups_test.rs` (9 tests) and
`cycle_groups_equivalence_test.rs` (4 tests), both reviewer-verified APPROVED at
`b950963`". That is the same disclosure Phase 2's T2-A handoff pre-authorised for
its own unobservable Red (deviation C), and its reviewer ACCEPTED it as the
correct behaviour over fabricating a Red.

**What you must NOT do:** do not invent a Red by breaking a fixture, weakening a
gate, or editing an assertion to fail. Do not delete `detect_cycles` to force a
compile error -- that is unit D's job and it belongs to a different scope.

**Your Red-equivalent evidence** is the gate set in sec8, in particular gate 5
(the whole `ariadne-integration-tests` crate), which must show **exactly the same
passed count before and after your change**. Your change adds and removes no
test, so any non-zero delta is a finding, not progress. Measure the crate line
BEFORE your first edit and record it; that pair is your evidence.

---

## 5. Required tests

No new tests. Both edited files keep their exact test counts: `m20` carries
**5** `#[test]` functions at `b950963` (`:85`, `:120`, `:149`, `:194`, `:236`)
and `m22` carries **6** (`:119`, `:183`, `:247`, `:300` `#[tokio::test]`,
`:413`, `:436`). Any other count is a finding.

---

## 6. Refactor and hygiene targets

- Nothing to refactor. Functions < 100 lines, files < 1000 lines
  (`m20_scala_dependency_graph_test.rs` is 258, `m22_scala_load_analysis_test.rs`
  is 455 -- **neither approaches the ceiling and your change makes both
  smaller**). No `tmp`/`res`/`foo`/`bar`/`test1`, no single-letter locals in code
  you author, no source line >= 2000 characters.
- `python tools/hygiene_check.py`: zero R1/R2.

---

## 7. The golden lock - a re-pin here is a FINDING, not a chore

Plan T3.1 says "re-pin any golden containing concrete cycle lists". **I verified
against the tree that none exists.** Both golden reports,
`tests/golden/basic/includes_report.json:2` and
`tests/golden/expensive/includes_report.json:2`, read exactly:

```
  "cycle_count": 0,
```

`IncludeReport` (`crates/graph/src/types.rs:56-66`) carries a count, not paths --
there is no field anywhere that serialises a concrete cycle list. Both golden
fixtures are acyclic, so 0 groups == 0 paths and the byte is unchanged.

Therefore:

- `tests/golden/**` is a **frozen lock** in this unit, not a work item.
- Run `UPDATE_GOLDENS=1` **never**. Never regenerate a golden.
- Gate 2 (`m00_golden_baseline_test`) must be green with the committed goldens
  **byte-unchanged**.
- **If gate 2 fails, that is a finding, not a re-pin.** Stop, report it, and do
  not regenerate. The correct fix is in unit D (or is a genuine defect), never
  an artifact rewrite.

The same applies to `tests/fixtures/**`: no new files may be added inside
`tests/fixtures/basic` or `tests/fixtures/expensive`, and no existing file may
change.

---

## 8. Acceptance - gates, exact commands, expected verdict lines

Every gate in DIRECT form. **Never write a gate as
`python tools/distill.py -- <argv>`** unless `<argv>` is a `cargo build`.

**Never write a bare-word test filter** -- `testrun.py ... -- <word>` passes
`<word>` to cargo as a TEST-NAME filter and selected zero tests in Phase 1,
returning `0 passed / 0 failed / 0 ignored / 7 suites` with exit 0 (Phase 1
verdict finding 2). Use `--test <target>`.

**9 invocations.**

```
1. python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m3_include_graph_test
2. python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m00_golden_baseline_test
3. python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m20_scala_dependency_graph_test
4. python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m22_scala_load_analysis_test
5. python tools/testrun.py --distill --timeout 3600 -p ariadne-integration-tests
6. python tools/hygiene_check.py
7. python tools/encoding_guard.py --staged
8. python tools/checkpoint_guard.py status
9. python tools/commit_scoped.py --scope 'tests/integration/tests/m20_scala_dependency_graph_test.rs' --scope 'tests/integration/tests/m22_scala_load_analysis_test.rs' --scope 'tasks/m235-t3c-journal.md' -m "test(integration): migrate Scala cycle consumers to cycle groups"
```

**Gate 1 is the IV-9 gate the plan requires.** `m3_include_graph_test.rs` is NOT
in your touch-set and you will NOT edit it. It asserts `report.cycle_count > 0`
at `:118-121` and it passes because `include/a.h` and `include/b.h` form one
nontrivial SCC. **"Passes without edit" is the proof; an edit is a blocking
finding.**

**Expected verdict lines:**

| Gate | Expected verbatim line |
|---|---|
| 1 | `3 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `2 passed / 0 failed / 0 ignored / 1 suites` |
| 3 | `5 passed / 0 failed / 0 ignored / 1 suites` |
| 4 | `6 passed / 0 failed / 0 ignored / 1 suites` |
| 5 | **identical to the line you measured before your first edit** (see below) |
| 6 | zero R1/R2 lines; R3 warns, if any, are legacy files only |
| 7 | clean, no output |

**The arithmetic behind gates 1-4** (counted out of the tree at `b950963`, so you
can check the gate rather than trust it):

- `m3_include_graph_test.rs` -- `#[test]` at `:14`, `:79`, `:146` -> **3**.
- `m00_golden_baseline_test.rs` -- `#[test]` at `:39`, `:44` -> **2**.
- `m20_scala_dependency_graph_test.rs` -- `#[test]` at `:85`, `:120`, `:149`,
  `:194`, `:236` -> **5**.
- `m22_scala_load_analysis_test.rs` -- `#[test]` at `:119`, `:183`, `:247`, `:413`,
  `:436` plus one `#[tokio::test]` at `:300` -> **6**.

Each of these is one test target, so each gate is exactly one suite. None of the
files carries an `#[ignore]`.

**Gate 5 -- read this carefully, it is not a number I can pre-pin.** The
`ariadne-integration-tests` crate carries **139** `#[test]`/`#[tokio::test]`
functions in total at `b950963` (counted across every target under
`tests/integration/tests/`, including the five `m18_cpp_plugin_parity/*`
sub-modules that `m18_apply_migrate_characterization_test.rs` declares). I am not
giving you a guessed `139 / 0 / x / y` line, because a guess is exactly the defect
that made Phase 1's gates unusable. Instead:

> **Measure gate 5 verbatim BEFORE your first edit and again after. The two lines
> must be IDENTICAL.** This unit adds and removes no test. Record both in your
> journal. Any non-zero delta is a finding you must report, not a number to
> explain away.

Run gate 5 before your edit as your step-0 measurement, then again as gate 5.

**Effect on the canonical workspace line:** none from this unit's own targets
(counts unchanged). The reviewer's acceptance pass will read
`1169/0/4/88` plus whatever units A and B added -- do not predict it; quote what
you measured.

---

## 9. Frozen locks - each with a proving command

| Lock | Proving command | Expected |
|---|---|---|
| **IV-9: `m3_include_graph_test` unedited and green** | gate 1; reviewer's `git diff --name-only` | `3/0/0/1`; the path absent from every commit |
| `tests/golden/**` byte-unchanged, no `UPDATE_GOLDENS` | gate 2; reviewer's `git diff --name-only` | `2/0/0/1`; absent |
| `tests/fixtures/**` byte-unchanged, no new fixture files | reviewer's `git diff --name-only`; `git status` | absent |
| `m23_symbol_isolation_test.rs:205` `cycle_count == 0` unedited | gate 5 | green; path absent from commits |
| m20's assertions `:160-189` unedited (IV-11 stance + serde shape) | reviewer's `git diff` | only the `:154` call and its local `assert!` changed |
| m22's assertions `:447-453` unedited | reviewer's `git diff` | only the `:442` call and its local `assert!` changed |
| m20 (5) and m22 (6) test counts | gates 3 and 4 | `5/0/0/1` and `6/0/0/1` |
| no production code touched | reviewer's `git diff --name-only` | no `crates/**` path in any commit |
| `crates/lang-scala/**` untouched (IV-12 is plugin-side-free) | reviewer's `git diff --name-only` | absent |
| IV-8 planner byte-identity suites unaffected | reviewer's `git diff --name-only` | `m00_facts_path_parity_test.rs` absent |
| R1/R2 hygiene | gate 6 | no R1/R2 |
| UTF-8 mojibake | gate 7 | clean; authored content pure ASCII |
| touch-set | reviewer's per-commit `git diff --name-only` | exactly the three in-set paths |
| the unpinned fixture path | reviewer | in no commit (ledger F12) |

Workspace-wide gates are NOT yours: `python tools/testrun.py --baseline` (BROKEN,
ledger F9), `python -m unittest discover -s tools -p "*_tests.py"`,
`python tools/ui_smoke.py`, `python tools/bench_compare.py` (both REFUSED, ledger
F10). Do not work around any of them; never route a refused command through
`distill.py`.

---

## 10. Reviewer gate reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
sec2. The reviewer will audit:

1. **IV-9 is a gate, not an expectation.** `m3_include_graph_test.rs` must appear
   in NO commit and gate 1 must have been run and quoted. This is the single most
   important check in this unit.
2. **Assertion preservation.** The reviewer diffs both files and confirms that
   the ONLY changes are the two call sites and their immediate local assertions.
   Any changed truth condition in `:160-189` or `:447-453` is a finding.
3. **The `len() >= 1` -> `is_empty()` reasoning.** Confirm the journal explains
   that the count changed MEANING (paths -> groups), not value, and that the
   preserved stance is non-emptiness.
4. **The golden no-op.** The goldens must be untouched and gate 2 green. If a
   golden was regenerated, that is a blocking finding regardless of the diff.
5. **The honest no-Red disclosure** rather than a manufactured Red.
6. **Gate 5's before/after identity**, quoted with both lines.

**Report back (final message):** commit SHAs (also in your journal); gate 1's
verbatim line as the IV-9 proof; gate 5's before line and after line; the
`m3_include_graph_test` no-edit confirmation; every deviation with its reason;
out-of-scope observations, report-only. Never paste raw cargo or git output.

---

## 11. Discipline clauses

- **Command execution.** `bash` is an allowlist restricted to the distilling
  entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED.
- **Reading a refusal.** A refused command ALWAYS reports the catch-all deny
  rule, whatever the real cause. Treat it as boilerplate. Attempt each gate
  INDEPENDENTLY; never drop a gate because its first invocation was refused. If a
  gate cannot run, record the VERBATIM refusal in your journal rather than
  silently narrowing the acceptance set.
- **Strict TDD.** Measure gate 5 -> edit the two call sites -> re-run every gate
  -> compare. Never weaken an assertion to make it pass. Never commit a Red state.
- **Code hygiene.** No `tmp`/`res`/`foo`/`bar`/`test1`, no single-letter locals in
  code you author, no source line >= 2000 characters, functions < 100 lines.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>`, or
  `Get-Content | -replace` on a tracked file. Kilo Read/Edit/Write tools, or
  byte-level `open(path, "wb")`. Pure-ASCII authored content. Quote whole git
  arguments: `git show "rev^:path"`.
- **Scope.** Touch only sec2 -- two files. Out-of-scope breakage: report it, do
  not fix it.
- **Journal.** Append a 1-3 line checkpoint after EACH Green gate, including the
  gate-5 before/after pair. Record your own SHAs in it.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A
  dead session is never resumed -- a successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3c-handoff.md --scope ... --cmd ...`.

---

## 12. Sizing check (handoff law, stated explicitly)

| Metric | Bound | This unit |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 top-level module directory | `tests/integration` only -- **1 top-level module directory under `tests/`** |
| Gate commands | ~10 | **9** |
| Produced lines (implementation + test diff) | ~1200 | **~85** (two call sites +2/-2, two local assertions +4/-4, journal ~70) |

**Single handoff unit -- fits on all three axes, with the produced-lines figure
at 7% of the bound.**

The unit is deliberately tiny and deliberately separate. It cannot join unit B
(`crates/cli`) or unit A/E (`crates/ui`) or unit D (`crates/graph`) without
breaking the one-scope law, because `tests/integration` is its own top-level
module directory under `tests/`. It has no prerequisite, so it is the natural
parallel unit.

**Parallel safety: this unit is file-disjoint from units A, B and E.** The four
paths it touches appear in no other Phase 3 unit's touch-set. It may therefore
run **concurrently with unit A or unit B** -- but only in a SEPARATE WORKTREE.
Two live implementer sessions in this shared worktree on one branch would race
on `commit_scoped.py` staging, which is exactly the shared-worktree hazard the
scoped-commit tooling exists to prevent. In this worktree, dispatch sequentially.

---

## 13. Authorization and ordering

This handoff, `tasks/m235-kickoff.md`, and the M23.5 milestone row in
`tasks/milestone_ledger.md` (Phase 1 APPROVED, Phase 2 APPROVED and CLOSED, HEAD
`b950963`, canonical workspace line `1169 / 0 / 4 / 88`) are the sole
authorization for this unit. **No prerequisite unit.**

Unit D (`tasks/m235-t3d-handoff.md`) -- the atomic delete of
`detect_cycles`/`dfs_cycle` -- requires this unit AND unit B to have landed,
because after both, `detect_cycles` has no caller outside `crates/graph`. Do not
delete `detect_cycles` or `dfs_cycle`, do not touch `crates/graph/**`, and do not
begin Phase 4 (T4.1), which remains UNAUTHORIZED.