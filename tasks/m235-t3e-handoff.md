# M23.5 T3-E Handoff - Retire the Path-Based Include-Element DTO (crate:ui)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 3 of 4, unit E of 5 -- **the last Phase 3 unit**. Completes D1-B for the
UI cycle currency.
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 3
(T3.2), sec1.3, sec3 item 1
**Prerequisites (BOTH must be APPROVED and landed):** unit A
`tasks/m235-t3a-handoff.md` and unit B `tasks/m235-t3b-handoff.md`. Unit D
(`crates/graph`) is independent of this unit -- see sec12.
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`
**Journal:** `tasks/m235-t3e-journal.md` (you create it; record your own SHAs in it)

Every line reference was read out of the tree at `b950963`. **If a line reference
disagrees with the file you read, the FILE wins** -- report the drift.

---

## 0. PREFLIGHT

**Step 0.1** - `python tools/checkpoint_guard.py status`; record HEAD, branch,
dirty files, untracked files.

**Step 0.2** - the unpinned CRLF fixture. `tests/fixtures/basic/artifacts/
includes/report.json` has no `.gitattributes` pin and this worktree has
`core.autocrlf=true`, so a full test run re-dirties its **stat entry** even when
content is byte-identical (ledger deferred row **M23.5 F12**; measured three
times). If and only if `git diff --numstat HEAD -- <path>` shows rows: restore
with the Kilo `Read` tool then the Kilo `Write` tool. NEVER `Set-Content`,
`Out-File`, `>`, `>>`, `Get-Content | -replace`. `git checkout --` / `git restore`
are mutating git and are DENIED. Every `commit_scoped.py` carries an EXPLICIT
`--scope`, and that path must appear in NO commit.

**Step 0.3 - the ordering precondition.** `grep` for
`build_include_elements(` across the workspace. You must find **zero** call sites
outside `crates/ui/**`. If any remain in `crates/cli/**` (including
`crates/cli/tests/payload_tail_bench.rs`), **STOP and report**: unit B has not
landed (or missed one of its four sites) and deleting the old entry point now
would break the workspace build. The expected remaining callers are exactly the
seven inside `crates/ui/tests/ui_test/`.

---

## 1. Objective and Invariants

### Objective

Delete the path-based `build_include_elements` and rename
`build_include_elements_from_groups` back to `build_include_elements`, so the UI
payload layer has exactly **one** cycle entry point and the milestone's D1-B
(no dual representation) holds on the UI side too.

This unit exists because the handoff law permits one crate per unit. Unit A had to
**add** a sibling entry point rather than change the existing signature, because
changing it would have broken `crates/cli` at the A/B boundary. Unit B migrated
the CLI. Only now can a `crates/ui`-only commit retire the old one. **Do not
"simplify" this by folding the retirement into unit A or unit B -- that would
break the workspace at a handoff boundary, which is exactly what the sequencing
constraint forbids.**

### The seven in-crate call sites, re-verified at `b950963`

All are in `crates/ui/tests/ui_test/` (one test target, `main.rs` declares 19
modules):

| # | File:line | Shape |
|---|---|---|
| 1 | `dto_builders.rs:14` | `let cycles = vec![];` then `build_include_elements(&include_graph, &closures, &cycles, ...)` |
| 2 | `graph_api.rs:136` | same 5-arg shape |
| 3 | `export_order.rs:88` | `build_include_elements(&graph, &HashMap::new(), &[], ...)` |
| 4 | `export_order.rs:105` | same |
| 5 | `d27_node_provenance.rs:128` | `build_include_elements(&one_file_graph(), &HashMap::new(), &[], &languages, &unit_kinds)` |
| 6 | `d27_node_provenance.rs:139` | 5-arg shape |
| 7 | `common.rs:157` | `payload.include_graph = build_include_elements(...)` |

The four sites that pass a bare `&[]` or `vec![]` infer their element type from
the parameter, so they need **no** type annotation change -- only the callee
name. Site 1's `let cycles = vec![];` binding should be renamed to say
`cycle_groups` where that reads better; the sites at `d27_node_provenance.rs:128`
and `export_order.rs:88/:105` pass an empty slice on one line and need nothing
but the name.

### Invariants this unit preserves

- **IV-9 / D27 / determinism:** unchanged. This is a rename plus a deletion; the
  body, the 5-arg shape, the canonical `(path asc, file_id asc)` emission, the
  D27 OMITS rule and the node-data keys all carry over untouched.
- **Zero new hex literals:** you touch no CSS, no JS, no template. Semantic
  verdict colors unchanged across light, dark and dark-steampunk by construction.
- **IV-11 / IV-12:** untouched. No report code, no Scala plugin code.
- **Exactly one cycle entry point in `crates/ui`** after this unit.

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

- `crates/ui/src/dto.rs` (delete the old function; rename the new one)
- `crates/ui/tests/ui_test/dto_builders.rs` (site 1)
- `crates/ui/tests/ui_test/graph_api.rs` (site 2)
- `crates/ui/tests/ui_test/export_order.rs` (sites 3, 4)
- `crates/ui/tests/ui_test/d27_node_provenance.rs` (sites 5, 6)
- `crates/ui/tests/ui_test/common.rs` (site 7)
- `tasks/m235-t3e-journal.md` (**new**, yours)

You MUST NOT touch: `crates/ui/src/lib.rs` (`pub use dto::*;` at `:5` needs no
edit -- the renamed function is exported automatically), any other file under
`crates/ui/tests/ui_test/`, any file under `crates/ui/assets/`, `crates/cli/**`,
`crates/graph/**`, `crates/report/**`, `tests/**`, `tools/**`,
`.kilo/plans/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`.

No `git stash`, no branch change, no `git add .`.

---

## 3. Green - the minimal implementation bound

1. In `crates/ui/src/dto.rs`, **delete** `build_include_elements` and every helper
   that existed only to serve it. Before deleting a helper, grep for its name --
   if the renamed entry point uses it too, it stays.
2. **Rename** `build_include_elements_from_groups` to `build_include_elements`,
   and fix its doc comment: the "D27: `file_languages`/`file_unit_kinds` ...
   (locked contract, no fabricated default)" note stays, and the `cycles`
   parameter becomes `cycle_groups: &[ariadne_graph::CycleGroup]`. Leave a short
   comment recording that the third parameter was `&[Vec<FileId>]` (DFS path
   enumeration) before M23.5 and is now `&[CycleGroup]` (SCC partition), so the
   next reader does not "restore" the old type.
3. Update the seven call sites to the new name. Change **nothing else** in those
   files -- no assertion, no fixture, no test name.
4. Any import list that named the old function is updated; `crates/ui/tests/
   ui_test/main.rs` and `crates/ui/src/lib.rs` need **no** edit.

If after step 3 the crate still compiles with a dangling reference to the old
name anywhere, that is a finding: report it, do not widen your scope.

---

## 4. Red - none, and why that is correct here

This is a **deletion plus a rename**. There is no behaviour to drive Red: the
new function was proven by unit A's `intra_group_non_tree_edge_carries_the_cycle_class`
and its four siblings, and it has been in production use since unit B wired the
CLI to it.

The handoff **pre-authorises you to state that plainly in your journal**: "no Red
for T3-E; the entry point it retires was proven in unit A and put into production
use by unit B, both reviewer-APPROVED." That is the same disclosure Phase 2's
T2-A handoff pre-authorised for an unobservable Red (deviation C), and its
reviewer ACCEPTED it over a fabricated Red.

**Your Red-equivalent evidence is the compiler plus the test counts:**

- **The deletion is proved by a compile failure.** Before you delete, deliberately
  leave one stale reference, run gate 1, and record the resulting compile error
  verbatim in your journal. Then fix it. This is not a fabricated Red -- it is a
  real, executed demonstration that the seven sites in sec1 are the complete set,
  which is the actual risk in a rename-and-delete unit. (Recommended: pick the
  LAST site, `common.rs:157`, and prove it.)
- **The rename is proved by gate 3's zero-warning build** and by gate 1's
  unchanged count.
- **The count identity:** gate 1 must read `184 passed / 0 failed / 0 ignored /
  1 suites`, exactly what unit A left. You add and remove no test.

---

## 5. Required tests

No new tests, no renamed tests, no changed assertions. All 179 pre-existing plus
unit A's 5 must be **byte-identical** in name and body except for the callee name
at the seven call sites.

`crates/ui/tests/ui_test/` count: **184** at entry (179 measured at `b950963` +
5 added by unit A). It must be **184** at exit.

---

## 6. Refactor and hygiene targets

- Functions < 100 lines, files < 1000 lines. `crates/ui/src/dto.rs` was 852 lines
  at `b950963` and unit A took it up by ~140; deleting the old entry point
  (~45 lines) and its now-unused helpers brings it back down. **Report the exact
  before/after line counts** -- if the file is still above ~940 lines after the
  deletion, say so and stop rather than extracting a module nobody asked for.
- No `tmp`, `res`, `foo`, `bar`, `test1`; no single-letter locals in code you
  author; one responsibility per function; no source line >= 2000 characters.
- `python tools/hygiene_check.py`: zero R1/R2. Every name you author
  intention-revealing.
- **Authored content is pure ASCII.**

---

## 7. Acceptance - gates, exact commands, expected verdict lines

Every gate in DIRECT form. **Never write a gate as
`python tools/distill.py -- <argv>`** unless `<argv>` is a `cargo build`.

**Never write a bare-word test filter** -- `testrun.py ... -- <word>` passes
`<word>` to cargo as a TEST-NAME filter and selected zero tests in Phase 1,
returning `0 passed / 0 failed / 0 ignored / 7 suites` with exit 0 (Phase 1
verdict finding 2). Use `--test <target>`.

**8 invocations.**

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui
3. python tools/distill.py -- cargo build -p ariadne-ui
4. python tools/theme_token_check.py --check-added --base <base-rev>
5. python tools/hygiene_check.py
6. python tools/encoding_guard.py --staged
7. python tools/checkpoint_guard.py status
8. python tools/commit_scoped.py --scope 'crates/ui/src/dto.rs' --scope 'crates/ui/tests/ui_test/dto_builders.rs' --scope 'crates/ui/tests/ui_test/graph_api.rs' --scope 'crates/ui/tests/ui_test/export_order.rs' --scope 'crates/ui/tests/ui_test/d27_node_provenance.rs' --scope 'crates/ui/tests/ui_test/common.rs' --scope 'tasks/m235-t3e-journal.md' -m "refactor(ui): retire the path-based include element builder"
```

**Gate 4's `--base`:** use the base revision **you were dispatched from** (the
commit your `checkpoint_guard.py status` preflight reports, normally unit A's or
unit D's commit). `--base` must be the revision your work started at so the check
covers only what you added. Do not use `b950963` here -- that would sweep in
unit A's and unit B's diffs and is not what the tool is for. **If you cannot
determine your base revision, run the check and record the verdict you get; do
not guess a revision.**

**Expected verdict lines:**

| Gate | Expected verbatim line |
|---|---|
| 1 | `184 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `184 passed / 0 failed / 0 ignored / 3 suites` |
| 3 | `ok (exit 0, 0 warnings emitted)` |
| 4 | no added hex literals (zero findings) |
| 5 | zero R1/R2 lines; R3 warns, if any, are legacy files only |
| 6 | clean, no output |

**The arithmetic, so you can check it rather than trust it:** `crates/ui/tests/
ui_test/` carries exactly **179** `#[test]`/`#[tokio::test]` functions at
`b950963` (counted across all 19 modules declared in `main.rs`; `#[tokio::test]`
expands to one `#[test]`, and there is **no** `#[ignore]` in that directory).
Unit A added exactly 5 -> 184. This unit adds none. `crates/ui/src/` carries
**zero** `#[test]` functions, so gate 2's three suites are lib + `ui_test` +
doc-tests.

**Gate 3 must report ZERO warnings.** It is the load-bearing evidence that the
deletion left no orphaned helper and the rename left no dangling import. Phase 1's
reviewer treated a warning-free build as evidence, not style.

**Effect on the canonical workspace line:** none. The reviewer's acceptance pass
reads the same line as after unit D: `1177 passed / 0 failed / 4 ignored /
88 suites` (`1169` + 5 from A + 1 from B + 0 from C + 2 from D + 0 from here).
You do not run the workspace-wide gate; do not claim that number as measured.

---

## 8. Frozen locks - each with a proving command

| Lock | Proving command | Expected |
|---|---|---|
| exactly one cycle entry point in `crates/ui` | reviewer's grep for `build_include_elements` | one definition, no path-based signature |
| `crates/ui/src/lib.rs` byte-unchanged | reviewer's per-commit `git diff --name-only` | absent |
| `crates/ui/tests/ui_test/main.rs` byte-unchanged | same | absent |
| unit A's 5 tests byte-unchanged apart from the callee name | reviewer's `git diff` on the four test files | name-only changes |
| all 184 test names unchanged | gate 1's count; reviewer's diff | `184/0/0/1` |
| D27 OMITS rule and canonical emission order | gate 1 (`export_order.rs`, `d27_node_provenance.rs` tests green unedited) | green |
| zero new hex literals, verdict colors intact across all three themes | gate 4 | zero findings |
| `crates/cli/**`, `crates/graph/**`, `crates/report/**` byte-unchanged | reviewer's `git diff --name-only` | absent |
| `tests/fixtures/**`, `tests/golden/**` byte-unchanged | reviewer's `git diff --name-only`; no `UPDATE_GOLDENS` run | absent |
| R1/R2 hygiene | gate 5 | no R1/R2; `dto.rs` well under 1000 |
| UTF-8 mojibake | gate 6 | clean; authored content pure ASCII |
| touch-set | reviewer's per-commit `git diff --name-only` | exactly the seven in-set paths |
| the unpinned fixture path | reviewer | in no commit (ledger F12) |

Workspace-wide gates are NOT yours: `python tools/testrun.py --baseline` (BROKEN,
ledger F9), `python -m unittest discover -s tools -p "*_tests.py"`,
`python tools/ui_smoke.py`, `python tools/bench_compare.py` (both REFUSED, ledger
F10). Do not work around any of them; never route a refused command through
`distill.py`.

---

## 9. Reviewer gate reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
sec2. The reviewer will audit:

1. **The seven sites are the complete set.** The journal's deliberate
   stale-reference compile error is the evidence that no ninth call site exists
   somewhere the implementer did not grep. If the journal lacks it, ask for it --
   a rename-and-delete with no evidence of completeness is the unit's whole risk.
2. **No helper orphaned.** Every function removed from `dto.rs` must have been
   used only by the deleted entry point; gate 3's zero warnings is the check.
3. **No accidental behaviour change.** The `git diff` on the four test files must
   be name-only. A changed assertion is a finding even if it "fixes" something.
4. **The doc comment on the renamed function** records the parameter's type
   history (`&[Vec<FileId>]` -> `&[CycleGroup]`) so nobody restores the old type.
5. **The dual representation is gone.** After unit E, `crates/ui` has exactly one
   cycle entry point; combined with unit D, plan sec3 item 1 is fully discharged.
6. **`dto.rs` line count** after the deletion, reported honestly.
7. **Gate 4's `--base` revision** is the implementer's dispatch base, not a
   hard-coded older revision.

**Report back (final message):** commit SHAs (also in your journal); the verbatim
stale-reference compile error from sec4; Green evidence quoting the canonical
lines and capture paths for gates 1, 2 and 3; the `dto.rs` before/after line
counts; the `--base` revision you used for gate 4; every deviation with its
reason; out-of-scope observations, report-only. Never paste raw cargo or git
output.

---

## 10. Discipline clauses

- **Command execution.** `bash` is an allowlist restricted to the distilling entry
  points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED.
- **Reading a refusal.** A refused command ALWAYS reports the catch-all deny rule,
  whatever the real cause. Treat it as boilerplate. Attempt each gate INDEPENDENTLY;
  never drop a gate because its first invocation was refused. If a gate cannot
  run, record the VERBATIM refusal in your journal rather than silently narrowing
  the acceptance set.
- **Strict TDD.** Prove the seven-site set with a deliberate stale reference ->
  delete -> rename -> update the sites -> Green -> Refactor. Never weaken an
  assertion. Never commit a state that does not compile.
- **Code hygiene.** Functions < 100 lines, files < 1000 lines, no
  `tmp`/`res`/`foo`/`bar`/`test1`, no single-letter locals in code you author, no
  source line >= 2000 characters.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>`, or
  `Get-Content | -replace` on a tracked file. Kilo Read/Edit/Write tools, or
  byte-level `open(path, "wb")`. Pure-ASCII authored content. Quote whole git
  arguments: `git show "rev^:path"`.
- **Scope.** Touch only sec2. Out-of-scope breakage: report it, do not fix it.
- **Journal.** Append a 1-3 line checkpoint after EACH Green gate. Record your
  own SHAs in it.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A
  dead session is never resumed -- a successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3e-handoff.md --scope ... --cmd ...`.

---

## 11. Sizing check (handoff law, stated explicitly)

| Metric | Bound | This unit |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 top-level module directory | `crates/ui` only -- **1 crate** |
| Gate commands | ~10 | **8** |
| Produced lines (implementation + test diff) | ~1200 | **~135** (`dto.rs` ~-50/+8, seven call sites ~+14/-14, journal ~70) |

**Single handoff unit -- fits with wide headroom, and it is deliberately the
smallest Phase 3 unit.**

Why it exists at all: D1-B ("no dual representation") is a plan acceptance
criterion, and unit A could not retire the path-based entry point in its own
commit without breaking `crates/cli`. Two `crates/ui`-scoped commits -- one that
adds (unit A) and one that retires (this unit), with the `crates/cli` rewiring
between them (unit B) -- is the only decomposition under the one-crate law that
keeps every handoff boundary compiling and green.

---

## 12. Authorization and ordering

This handoff, `tasks/m235-kickoff.md`, the M23.5 milestone row in
`tasks/milestone_ledger.md`, and APPROVED `tasks/m235-t3a-handoff.md` **and**
`tasks/m235-t3b-handoff.md` are the sole authorization for this unit.

**Runs last.** Ordering across Phase 3, with the dependency that forces it:

```
A  crates/ui           additive DTO entry point        <- first, lands green
B  crates/cli          CLI consumers + bench re-pin    <- needs A's entry point
C  tests/integration   Scala consumers + IV-9 gate     <- needs nothing (parallel-safe with A/B in a separate worktree)
D  crates/graph        IV-11 port + ATOMIC DELETE      <- needs B and C
E  crates/ui           retire the path-based DTO       <- needs A and B   *** this unit
```

**Parallel safety:** this unit and unit D touch **disjoint files**
(`crates/ui/**` vs `crates/graph/**`) and neither depends on the other, so they
may run concurrently **in separate worktrees**. In this shared worktree,
dispatch sequentially. No two Phase 3 units other than (A, C) and (B, C) are
parallel-safe: A/B share the entry-point name, B/D share the delete precondition,
A/E share `crates/ui/src/dto.rs`.

Phase 4 (T4.1 integration closeout) remains UNAUTHORIZED. Do not begin it, do not
run the workspace-wide gates, and do not close ledger rows F9, F10, F11 or F12 --
those are operator actions at closeout. Do not touch `crates/graph/**`,
`crates/cli/**`, `tests/**`, or `tools/**`.