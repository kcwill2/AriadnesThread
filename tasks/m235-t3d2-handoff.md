# M23.5 T3-D2 Handoff - Legacy/Group Include-Builder Parity Lock (crate:ui)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 3 of 4, **inserted unit D2**. Slots BETWEEN unit D
(`tasks/m235-t3d-handoff.md`) and unit E (`tasks/m235-t3e-handoff.md`).
Phase 3 therefore has SIX units: A, B, C, D, **D2**, E.
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 3
(T3.2), sec3 items 1 and 8
**Origin of this unit:** `tasks/m235-t3b-verdict.md` sec5.6 and finding 3
("companion parity test ... has a HARD EXPIRY at unit E"), re-confirmed still
open as `tasks/m235-t3d-verdict.md` finding 13.
**Prerequisite:** unit D APPROVED (`tasks/m235-t3d-verdict.md`, HEAD `aebfa9b`).
**Hard expiry:** this unit must land BEFORE `tasks/m235-t3e-handoff.md` is
dispatched. See sec12.
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`
**Journal:** `tasks/m235-t3d2-journal.md` (you create it)

Every line reference below was read out of the tree at `aebfa9b`. **If a line
reference disagrees with the file you read, the FILE wins** - report the drift,
do not edit around it.

---

## 0. PREFLIGHT

**Step 0.1** - `python tools/checkpoint_guard.py status`; record HEAD, branch,
dirty files, untracked files. Your dispatch base is that HEAD. It is `aebfa9b`
as of this handoff; if it is not, you were dispatched from something else and
sec7 gate 4 must use YOUR base.

**Step 0.2** - the unpinned CRLF fixture.
`tests/fixtures/basic/artifacts/includes/report.json` has no `.gitattributes`
pin and this worktree has `core.autocrlf=true`, so a full test run re-dirties
its **stat entry** even when content is byte-identical (ledger deferred row
**M23.5 F12**, six reproductions). If and only if
`git diff --numstat HEAD -- <path>` shows rows: restore with the Kilo `Read`
tool then the Kilo `Write` tool. NEVER `Set-Content`, `Out-File`, `>`, `>>`,
`Get-Content | -replace`. `git checkout --` / `git restore` are mutating git
and are DENIED. Every `commit_scoped.py` carries an EXPLICIT `--scope`, and
that path must appear in NO commit.

**Step 0.3 - confirm your expiry is still real, do not assume it.** Grep
`pub fn build_include_elements` under `crates/ui/src`. You must find exactly
TWO public entry points: the legacy path-based one and the group-based sibling.
If only ONE exists, unit A or E already changed the shape and this unit's
premise is void: **STOP and report** rather than writing a test against a
surface that no longer exists. Expected, at `aebfa9b`:
`crates/ui/src/dto.rs:608` and `crates/ui/src/dto.rs:637`.

**Step 0.4 - confirm the fixture is acyclic from the tree, not from this file.**
`crates/ui/tests/ui_test/export_order.rs:34-56` builds a 10-file graph whose
only edges are `5 -> 2`, `1 -> 9`, `3 -> 6`. No cycle is reachable. Step 4 of
sec5 makes that machine-checked anyway; you do not need to trust this sentence.

---

## 1. Objective and Invariants

### Objective

Add ONE parity lock, in `crates/ui`, that the T3-B verdict left explicitly
open: on a graph carrying **no** cycle currency from either representation, the
legacy path-based builder `build_include_elements` and the milestone's
group-based sibling `build_include_elements_from_groups` emit **byte-identical**
elements.

**Why this is the exact obligation.** `tasks/m235-t3b-verdict.md` sec5.6, quoted:
"prove that unit E's deletion of the legacy builder is behaviour-preserving on
acyclic graphs (`build_include_elements_from_groups(g, .., &[], ..)`
byte-equal to `build_include_elements(g, .., &[], ..)`), which is exactly the
evidence unit E needs and is forbidden to produce itself." **Do not substitute
a different obligation.** In particular, see sec5.4.

### What this unit is NOT

- **It is not a replacement for the two-process export lock.** That lock lives
  at `crates/cli/tests/payload_tail_bench.rs:501-621`
  (`export_bytes_are_identical_across_two_separate_processes`) and is
  reviewer-ACCEPTED (`tasks/m235-t3b-verdict.md` sec5, verdict 5.5: the
  two-sample residual is **ACCEPTED**). **This unit must not re-open, weaken,
  extend or re-litigate that decision.** It adds the in-crate parity companion
  only. If you believe the two-sample limit should be revisited, that is a
  report-only observation in your journal - not a code change.
- **It is not a determinism lock.** Hash-iteration-order determinism is already
  locked by `crates/ui/tests/ui_test/graph_api.rs:741-789` (in-process) and by
  the CLI's two-process export lock. See sec5.2 for the honest limit of what
  P1 proves.
- **It is not a migration.** No production file changes. Zero lines of
  `crates/ui/src/**` are edited. See sec3.

### Invariants this unit preserves

- **IV-2 / IV-3 / IV-9 / IV-11 / D27:** untouched. You add no production code,
  so every semantic invariant is preserved by construction.
- **Zero new hex literals:** you touch no CSS, no JS, no template, no
  `crates/ui/assets/**` file. The semantic verdict colors across light, dark and
  dark-steampunk cannot move.
- **The 182 pre-existing `crates/ui` tests stay byte-identical.** You add
  exactly two `#[test]` functions. You edit no existing test name, assertion,
  fixture or import line except as sec2 permits.
- **`crates/ui/src/dto.rs` stays byte-identical**, so unit E's delete-and-rename
  plan is untouched by this unit.

---

## 2. Touch-set (hard; any edit outside is a blocker - report, do not fix)

- `crates/ui/tests/ui_test/export_order.rs` (append two `#[test]` functions plus
  at most two small fixture helpers)
- `tasks/m235-t3d2-journal.md` (**new**, yours)

That is the whole touch-set: **one test file in one crate, plus your journal.**

You MUST NOT touch: `crates/ui/src/**` (including `dto.rs` and
`cycle_groups_view.rs` - they are the subject of the test, not of it),
`crates/ui/tests/ui_test/main.rs`, any other file under
`crates/ui/tests/ui_test/` (`dto_builders.rs`, `graph_api.rs`,
`d27_node_provenance.rs`, `common.rs` and the other fifteen modules),
`crates/ui/tests/**` outside `ui_test/`, `crates/ui/assets/**`,
`crates/ui/Cargo.toml`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`,
`crates/cli/**`, `crates/graph/**`, `crates/report/**`, `crates/cluster/**`,
`crates/lang-scala/**`, `crates/lang/**`, `tests/**`, `tools/**`,
`.kilo/plans/**`, and every other `tasks/**` file.

**Two named out-of-scope observations, so you do not "helpfully" fix them:**

1. `crates/cli/tests/payload_tail_bench.rs:389` still emits a bench row label
   reading `build_include_elements` while the timed call is now
   `build_include_elements_from_groups` (raised as `tasks/m235-t3b-verdict.md`
   finding 4). **Owner: unit E.** Report-only.
2. `crates/graph/src/types.rs:57-60` and `:66-68` still document
   `informational_cycle_count` in terms of cycles rather than groups
   (`tasks/m235-t3d-verdict.md` finding 4). **Owner: closeout.** Report-only.

No `git stash`, no branch change, no `git add .`.

---

## 3. Green - the minimal implementation bound

Append to `crates/ui/tests/ui_test/export_order.rs`. **Nothing else changes.**

1. **Two `#[test]` functions**, exactly as specified in sec5 (P1 and P2). No
   third test. No helper module, no new file, no `main.rs` edit.
2. **At most two small fixture helpers**, and only if sec5.1's maps cannot be
   written inline. Every name intention-revealing; no single-letter locals; no
   `tmp`/`res`/`foo`/`bar`/`test1`.
3. **Imports**: the file already imports both builders at
   `crates/ui/tests/ui_test/export_order.rs:12-15`
   (`build_include_elements`, `build_include_elements_from_groups`). You will
   need `ariadne_graph::IncludeClosure` added to the `:10` import list for the
   closures map, and `CycleGroup` if P2's group is hand-built rather than taken
   from `detect_cycle_groups()`. Those are the **only** permitted edits to
   existing lines.
4. **No production change.** If you find yourself wanting to change
   `crates/ui/src/dto.rs` to make a test pass, that is a finding: report it.
   Never weaken an assertion to reach green.

---

## 4. Red - there is no production-behaviour Red, and the executed Red is a mutation

**State this plainly, do not manufacture a Red.** This unit adds no production
code. Both entry points already exist and are already in production use
(unit A added the group sibling; unit B wired the CLI to it). So there is no
behaviour to drive Red against, exactly as `tasks/m235-t3e-handoff.md` sec4
pre-authorised for unit E - and that precedent was ACCEPTED by its reviewer.

**The executed Red is a mutation of the new lock itself, in three steps.**
Run gate 1 after each and record all three canonical lines in your journal.

| Step | What you do | Gate 1 line you must record |
|---|---|---|
| R1 | add **P2 only** (the teeth check, sec5.3). Do not add P1 yet. | `185 passed / 0 failed / 0 ignored / 1 suites` |
| R2 | add **P1**, then temporarily pass a **NON-EMPTY** `&[CycleGroup]` to its group-builder call instead of `&[]`. Leave everything else in P1 alone. | **`185 passed / 1 failed / 0 ignored / 1 suites`** |
| G | restore `&[]` in P1. | `186 passed / 0 failed / 0 ignored / 1 suites` |

**The R2 line is the Red, and it is a coherent line**: one test failing, 185
passing, 0 ignored, 1 suite. It is a DIFFERENT state from the Green line and
must never be reported as, or confused with, it. Do not write a single line
anywhere that reports `0 failed` while describing a failing test - the T3-B
verdict recorded exactly that self-contradiction as a planner defect
(`tasks/m235-t3b-verdict.md` finding 1).

**What the R2 failure message will say.** The failing test is
`acyclic_include_elements_are_byte_identical_under_both_cycle_currencies`. The
panic comes from P1's byte-equality assertion (sec5.1), reported as
`assertion failed: (left == right)` with

- `left` = the serialization of the **group-based** elements, which for the
  grouped node carries the keys `cycle_group_size` and `cycle_witness` and
  `classes` = `Some("cycle")`;
- `right` = the serialization of the **legacy** elements, which for the same
  node carries `is_cycle` = `false`, **no** `cycle_group_size` key, **no**
  `cycle_witness` key, and `classes` = `None`.

Assert with a message string naming the property, so the panic's own last line
is self-explaining. Record the panic verbatim in your journal.

**Never commit a Red state.** R2 exists only in your working tree; the commit at
sec7 gate 8 carries the restored, green P1.

**P2 has no Red of its own, and its evidence is the R2 run.** P2's assertion and
P1's mutated assertion are the SAME comparison over the SAME fixture family
with OPPOSITE expectations (`assert_ne!` vs `assert_eq!`). So the run in which
P1 fails for a non-empty group currency is simultaneously the run that proves
P2's comparison has teeth. State that in P2's header comment and in your
journal; do not claim two independent Red observations.

---

## 5. Required tests

`crates/ui/tests/ui_test/` carries exactly **184** `#[test]`/`#[tokio::test]`
functions at `aebfa9b` (counted across all 19 modules declared in
`crates/ui/tests/ui_test/main.rs:1-19`; `#[tokio::test]` expands to one
`#[test]`; there is no `#[ignore]` in that directory). **184 at entry, 186 at
exit.** Both new tests land in the existing `ui_test` target, so the suite
count does not move.

### 5.1 P1 - the parity lock (the obligation)

Name it exactly:

```
acyclic_include_elements_are_byte_identical_under_both_cycle_currencies
```

Body, in this order:

1. **Fixture sanity, machine-checked.** Assert
   `shuffled_file_graph().detect_cycle_groups().is_empty()` with a message
   naming why ("the parity fixture must be acyclic, or `&[]` is not the
   currency unit E will delete"). Do not take acyclicity on this handoff's word.
2. **Build the inputs once**, so both calls receive identical arguments:
   - `graph` = `shuffled_file_graph()` (`export_order.rs:34-56`): 10 files, a
     duplicate path on ids 5/11 that exercises the `(path asc, file_id asc)`
     tie-break, and 3 edges;
   - `closures` = a **non-empty** `HashMap<FileId, IncludeClosure>` covering a
     **strict subset** of the fixture's ids, so both a decorated node
     (`closure_size: N`, from `crates/ui/src/dto.rs:696`) and an undecorated
     one are inside the compared bytes. `IncludeClosure` is
     `{ file_id, direct_includes, transitive_includes, closure_size }`
     (`crates/graph/src/types.rs:18-23`); `IncludeClosure::new` exists at
     `types.rs:26` and sorts/dedups for you. **If you cannot construct one,
     report that as an out-of-scope observation and use an empty map - do not
     edit `crates/graph/**`.**
   - `file_languages` and `file_unit_kinds` = **non-empty** maps covering a
     strict subset of ids, so the D27 decoration path
     (`crates/ui/src/dto.rs:599-602`, the OMITS rule) is inside the compared
     bytes for some nodes and absent for others.
3. **Two calls, empty currency on both sides:**
   `build_include_elements(&graph, &closures, &[], &languages, &unit_kinds)`
   and
   `build_include_elements_from_groups(&graph, &closures, &[], &languages, &unit_kinds)`.
4. **Non-vacuity first.** Assert the element vector is non-empty and report its
   length in the message, and assert at least one node element carries a
   non-zero `closure_size` and one carries `language` while another omits it.
   Without this, "both sides are empty, therefore equal" would satisfy the
   byte comparison and the test would prove nothing.
5. **The byte lock.** Serialize each vector with `serde_json::to_string` and
   `assert_eq!` the two strings. This is the literal claim "byte-identical", not
   a structural proxy: `CytoscapeElement` is
   `{ data: serde_json::Value, classes: Option<String> }` with
   `#[derive(Debug, Clone, PartialEq, Eq, ...)]` (`crates/ui/src/dto.rs:53-58`),
   and `serde_json::Map` is a `BTreeMap` in this workspace, so a structural
   `assert_eq!` would compare key/value sets and say nothing about emitted key
   order. The serialization comparison does.
6. **The structural lock, too.** `assert_eq!` the two `Vec<CytoscapeElement>`
   values as well, so a mismatch reports a readable element-level diff rather
   than only a string blob. This mirrors the existing precedent at
   `crates/ui/tests/ui_test/graph_api.rs:786-789`.

### 5.2 P1's header comment - say what it does NOT lock

The file's own module doc (`crates/ui/tests/ui_test/export_order.rs:1-8`) warns
that "a double-build equality test inside ONE test process is vacuously green
for any map-iteration order". **That warning is correct and it is why P1 must
not be mistaken for a determinism lock.** P1 compares two DIFFERENT entry
points on the SAME graph in the SAME process, so any hash-order perturbation
would affect both sides equally: P1 is insensitive to it by construction. P1
locks **classification state**, not emission order across processes. Say this
in P1's header comment in your own words, and cite
`crates/ui/tests/ui_test/graph_api.rs:741-789` and the CLI two-process lock as
where determinism actually lives. **Do not amend the module doc at
`export_order.rs:1-8`** - it is outside the spirit of this unit and its
statement is true as written.

### 5.3 P2 - the teeth check (non-vacuity, and nothing more)

Name it exactly:

```
the_parity_comparison_detects_a_cycle_classification_difference
```

Body:

1. `graph` = `cyclic_shuffled_file_graph()`
   (`crates/ui/tests/ui_test/export_order.rs:127-149`). Assert
   `graph.detect_cycle_groups()` yields **exactly one** group and that its
   `member_files` are `[2, 5, 11]` - the same fixture sanity the neighbouring
   test asserts at `export_order.rs:158-164`. Do not invent a new fixture.
2. Call the legacy builder with `&[]` and the group builder with the real
   `&groups`.
3. `assert_ne!` the two serializations, with a message naming the property: the
   comparison P1 relies on must be able to FAIL, and the group-based side is
   the one that differs.

P2 exists **only** as P1's non-vacuity witness. It does not re-test IV-2, IV-3,
the group keys or the OMITS rule - `crates/ui/tests/ui_test/dto_builders.rs:457`,
`:490` and `:562` already own those, all APPROVED with unit A. **If you find
yourself adding a third test, you have left this unit.**

### 5.4 What this unit explicitly does NOT do

- It does not extend, replace or re-argue the two-process export lock
  (`crates/cli/tests/payload_tail_bench.rs:501-621`). Accepted as-is by
  `tasks/m235-t3b-verdict.md` sec5.5.
- It does not compare the two builders on a graph that HAS groups. That
  comparison is not equal by design, which is the whole point of P2, and the
  grouped semantics are already locked by unit A's five tests.
- It does not run, re-pin or assert anything about `bench_compare.py` or
  `ui_smoke.py`. Both are REFUSED (ledger F10). See sec7.
- It does not touch unit E's delete-and-rename, nor decide when unit E runs.

---

## 6. Refactor and hygiene targets

- Functions < 100 lines, files < 1000 lines. `export_order.rs` is **279** lines
  at `aebfa9b`; you add roughly 90. Report the exact before/after counts.
  **If the file somehow approaches 700 lines, say so and stop** rather than
  extracting a module nobody asked for.
- P1 and P2 must each be readable top-to-bottom in one screen. If either grows
  past ~40 lines, that is a signal you have inlined something that should be a
  named helper - not a licence to split the test's assertions across helpers
  that hide the comparison.
- No `tmp`, `res`, `foo`, `bar`, `test1`; no single-letter locals in code you
  author (the pre-existing `|c|` at `crates/ui/src/dto.rs:696` is not yours and
  must stay); intention-revealing names; one responsibility per function; no
  source line >= 2000 characters.
- `python tools/hygiene_check.py`: **zero R1/R2**. R3 is warn-only; if you
  introduce a NEW R3 warn in `export_order.rs`, fix the name rather than
  declaring it legacy.
- **Authored content is pure ASCII.** No em dash, no arrow glyph, no section
  sign. Use `--`, `->`, "sec".

---

## 7. Acceptance - gates, exact commands, expected verdict lines

Every gate in DIRECT form. **Never write a gate as
`python tools/distill.py -- <argv>` unless `<argv>` is a `cargo build`.**

**Never write a bare-word test filter** - `testrun.py ... -- <word>` passes
`<word>` to cargo as a TEST-NAME filter and selects zero tests while returning
exit 0. Use `--test <target>`.

**8 distinct gates, 10 planned invocations** (gate 1 runs three times across the
RGB trail in sec4; nothing else repeats).

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui
3. python tools/distill.py -- cargo build -p ariadne-ui
4. python tools/theme_token_check.py --check-added --base <your dispatch base>
5. python tools/hygiene_check.py
6. python tools/encoding_guard.py --staged
7. python tools/checkpoint_guard.py status
8. python tools/commit_scoped.py --scope 'crates/ui/tests/ui_test/export_order.rs' --scope 'tasks/m235-t3d2-journal.md' -m "test(ui): lock legacy/group include-builder parity on acyclic graphs"
```

**Gate 8 runs TWICE, in this order** - this is prescribed, not a deviation, and
it resolves the journal-SHA self-reference that the T3-C reviewer adjudicated
(`tasks/m235-t3c-verdict.md` finding 1) and T3-D repaired in `aebfa9b`:

- **8a** the code commit: `--scope 'crates/ui/tests/ui_test/export_order.rs'`
  only, message as above. Nothing else.
- **8b**, after 8a: record 8a's SHA in the journal, then the journal commit:
  `--scope 'tasks/m235-t3d2-journal.md'` only.

**The journal's final line must state, in your own words, that commit 8b's own
SHA cannot appear inside 8b** and that the range tip is obtained with
`git log -1`. Do not attempt to write your own SHA into the file you are
committing, and do not amend. A handoff must never contain a header and a gate
that are mutually unsatisfiable.

**Gate 6 ordering, pre-authorised.** `commit_scoped.py` stages and commits
atomically and raw git staging is denied, so `--staged` cannot interpose between
staging and committing. Run the **strictly stronger**
`python tools/encoding_guard.py --all` immediately BEFORE each of 8a and 8b, and
the literal `--staged` form immediately AFTER. Disclose this in the journal as
a pre-authorised substitution, not as a deviation - exact precedent: T2-A gate
5, T2-B Dev-E, T3-A D5, T3-B D5, T3-C Dev-3, T3-D Dev-4.

**Gate 4's `--base`:** use the revision your `checkpoint_guard.py status`
preflight reported (expect `aebfa9b`), so the check covers only what you added.
**If you cannot determine your base revision, run the check and record the
verdict you get; do not guess a revision.**

### Expected verdict lines

| Gate | Expected verbatim |
|---|---|
| 1 (Green) | `186 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `186 passed / 0 failed / 0 ignored / 3 suites` |
| 3 | `ok (exit 0, 0 warnings emitted)` |
| 4 | no output / zero findings |
| 5 | zero R1/R2 lines; any R3 warn is a pre-existing line |
| 6 | clean, no output, exit 0 |
| 7 | your recorded HEAD and dirty/untracked state |

### The arithmetic, so you can check it rather than trust it

- **`crates/ui/tests/ui_test/` = 184 at entry, 186 at exit.** Counted, not
  estimated: 184 `#[test]`/`#[tokio::test]` across the 19 modules in
  `crates/ui/tests/ui_test/main.rs:1-19`. You add exactly two `#[test]`.
- **Gate 2's `3 suites` = lib + `ui_test` + doc-tests.** `crates/ui/Cargo.toml`
  declares **no** `[[bin]]` and **no** `[[test]]` section (it is 26 lines:
  package, dependencies, dev-dependencies), so cargo auto-discovers exactly
  `src/lib.rs` and `tests/ui_test/main.rs`. `crates/ui/src/**` contains **zero**
  `#[test]` functions, so lib contributes 0 and doc-tests contribute 0:
  `186 + 0 + 0 = 186`.
  **Do not copy a suite count from any ledger line.** The T3-B and T3-C
  reviewers both caught `-p ariadne-cli = 86/0/1/3` recorded where the truth is
  `88/0/1/5`, because `crates/cli/Cargo.toml:7-13` declares **two** `[[bin]]`
  targets and cargo runs a unittests binary for each even at zero tests
  (`tasks/m235-t3b-verdict.md` sec4, `tasks/m235-t3c-verdict.md` finding 2).
  Compute from `Cargo.toml` and the capture, every time.
- **Gate 1's `1 suite`** is unchanged because both tests land in the existing
  `ui_test` target. No new test binary appears.

**Effect on the canonical workspace line: `+2`, predicted, NOT measured by you.**
The reviewer's acceptance pass reads `1180 passed / 0 failed / 4 ignored /
88 suites` (`1178` at `aebfa9b`, plus your two tests; suites unchanged at 88).
**You do not run the workspace-wide gate and you must not claim that number as
measured.** State it as a prediction in your journal and let the reviewer
measure it.

Workspace-wide gates are NOT yours: `python tools/testrun.py --baseline`
(BROKEN, ledger F9 - `usage error: missing value for --baseline`),
`python -m unittest discover -s tools -p "*_tests.py"`,
`python tools/ui_smoke.py` and `python tools/bench_compare.py` (both REFUSED,
ledger F10). Do not work around any of them; **never route a refused command
through `distill.py`** to evade a denial. Attempt each independently, once, and
record any refusal verbatim in your journal rather than silently narrowing the
acceptance set.

---

## 8. Frozen locks - each with a proving command

| Lock | Proving command | Expected |
|---|---|---|
| `crates/ui/src/dto.rs` byte-unchanged (both entry points intact) | reviewer's `git diff --name-only` | absent from every commit |
| `crates/ui/src/cycle_groups_view.rs` byte-unchanged | same | absent |
| both public entry points still exist with their 5-arg shapes | reviewer's grep `pub fn build_include_elements` | exactly two, `dto.rs:608` and `dto.rs:637` |
| `emit_include_elements` still the ONE shared emission tail | reviewer's read of `dto.rs:659` | one definition, both entry points call it |
| the other 182 `crates/ui` tests byte-identical (names, assertions, fixtures) | reviewer's `git diff` on `export_order.rs` | additive hunks only; the two `use` lines at `:10`/`:12-15` are the only permitted edits to existing lines |
| `crates/ui/tests/ui_test/main.rs` byte-unchanged (still 19 modules) | reviewer's `git diff --name-only` | absent |
| no file outside `export_order.rs` under `crates/ui/tests/**` touched | same | absent |
| D27 OMITS rule and canonical `(path asc, file_id asc)` emission order | gate 1: `include_nodes_emit_in_canonical_path_then_file_id_order`, `include_edges_keep_source_vec_order_behind_sorted_nodes`, `grouped_nodes_keep_the_canonical_emission_order_and_edge_order` all green, unedited | green |
| unit A's five group tests untouched and green | gate 1: `intra_group_non_tree_edge_carries_the_cycle_class`, `node_cycle_data_reports_group_size_and_witness_only_for_grouped_nodes`, `grouped_nodes_and_edges_are_classed_cycle_from_membership_not_from_reported_paths` (and its two siblings) green | green |
| IV-5 in-process determinism lock untouched | reviewer's `git diff --name-only`; `graph_api.rs` absent | absent |
| the two-process CLI export lock untouched | reviewer's `git diff --name-only`; `crates/cli/**` absent | absent |
| zero new hex literals; verdict colors intact across all three themes | gate 4 | zero findings; and structurally: no `crates/ui/assets/**` path in your diff |
| `crates/graph/**`, `crates/report/**`, `crates/cluster/**` (IV-8), `crates/lang-scala/**` (IV-12) byte-unchanged | reviewer's `git diff --name-only` | absent |
| `tests/fixtures/**`, `tests/golden/**` byte-unchanged; no `UPDATE_GOLDENS` run | reviewer's `git diff --name-only` | absent |
| R1/R2 hygiene | gate 5 | no R1/R2; `export_order.rs` far under 1000 |
| UTF-8 mojibake | gates 6 / `encoding_guard.py --all` | clean; authored content pure ASCII |
| touch-set | reviewer's per-commit `git diff --name-only` | 8a: exactly `export_order.rs`; 8b: exactly the journal |
| the unpinned fixture path | reviewer | in no commit (ledger F12) |
| `crates/ui/Cargo.toml`, `Cargo.toml`, `Cargo.lock`, `.gitattributes` | reviewer's `git diff --name-only` | absent |

---

## 9. Reviewer gate reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
sec2. The reviewer will audit:

1. **The parity comparison is the obligation T3-B left open** - legacy vs
   group, empty currency on both sides, byte-compared - and not a substitute
   obligation the implementer preferred.
2. **P1 is non-vacuous.** The fixture emits elements, some node carries a
   non-zero `closure_size`, some node carries `language` and another omits it,
   and the acyclicity of the fixture is asserted from
   `detect_cycle_groups()`, not taken on the handoff's word.
3. **The byte claim is a byte claim.** `serde_json::to_string` equality, not
   only a `PartialEq` on `CytoscapeElement` - plus the structural assert for a
   readable diff. Both present.
4. **P2 exists and is not scope creep.** Same fixture family as the
   neighbouring grouped test, `assert_ne!`, no third test.
5. **The two-sample limit on the CLI two-process lock was not re-opened.** No
   change, no re-pin, no re-argument anywhere in the range or the journal. A
   report-only note is acceptable; a code change is a finding.
6. **The Red is the mutation, honestly reported.** Three gate-1 lines recorded
   verbatim - `185/0/0/1`, `185/1/0/1`, `186/0/0/1` - with the R2 panic quoted.
   **No line anywhere reports `0 failed` while describing a failing test**
   (`tasks/m235-t3b-verdict.md` finding 1).
7. **The two-commit form (8a code, 8b journal)**, each with an explicit
   `--scope`, and the journal's final line stating why 8b's own SHA is not in
   8b. No amend, no SHA self-reference paradox.
8. **The forbidden F12 fixture is in no commit**, and `--numstat` was clean or
   the Read/Write restore was performed and disclosed.
9. **Suite counts recomputed from `crates/ui/Cargo.toml` and the capture**, not
   read out of the ledger.
10. **Line counts reported honestly**, and the sizing arithmetic in sec11
    compared against what actually landed. Prior units undercounted mandated
    test bodies and the reviewer caught it (`tasks/m235-t3d-verdict.md`
    finding 1: predicted ~679, measured 734). If you exceed the estimate, say so
    with the decomposition; do not silently ship a different number.

**Report back (final message):** the two commit SHAs (also in your journal);
all three gate-1 canonical lines and the R2 panic verbatim; the gate-2 and
gate-3 lines with capture paths; `export_order.rs` before/after line counts;
the `--base` revision used for gate 4; the pre-authorised encoding-gate ordering
you used; every deviation with its reason; out-of-scope observations,
report-only. **Never paste raw cargo or git output.**

---

## 10. Discipline clauses

- **Command execution.** `bash` is an allowlist restricted to the distilling
  entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED.
- **Reading a refusal.** A refused command ALWAYS reports the catch-all deny
  rule, whatever the real cause. Treat it as boilerplate. Attempt each gate
  INDEPENDENTLY; never drop a gate because its first invocation was refused. If
  a gate cannot run, record the VERBATIM refusal in your journal rather than
  silently narrowing the acceptance set.
- **Strict TDD.** P2 -> P1 -> mutate P1 (Red) -> restore P1 (Green) -> hygiene
  pass. Never weaken an assertion. Never commit a state that does not compile.
  Never commit a Red state.
- **Code hygiene.** Functions < 100 lines, files < 1000 lines, no
  `tmp`/`res`/`foo`/`bar`/`test1`, no single-letter locals in code you author,
  no source line >= 2000 characters.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>`, or
  `Get-Content | -replace` on a tracked file. Kilo Read/Edit/Write tools, or
  byte-level `open(path, "wb")`. Pure-ASCII authored content. Quote whole git
  arguments: `git show "rev^:path"`.
- **Scope.** Touch only sec2. Out-of-scope breakage: report it, do not fix it.
- **Journal.** Append a 1-3 line checkpoint after EACH Green gate. Record your
  own SHAs per sec7 gate 8.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A
  dead session is never resumed - a successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3d2-handoff.md --scope ... --cmd ...`.

---

## 11. Sizing check (handoff law, stated explicitly)

| Metric | Bound | This unit |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 top-level module directory | `crates/ui` only, and within it exactly one test file - **1 crate** |
| Gate commands | ~10 | **8 distinct gates, 10 planned invocations** (gate 1 runs three times across the RGB trail) |
| Produced lines (implementation + test diff) | ~1200 | **~160** |

**The line estimate, itemised from the tests this handoff actually mandates** -
not from production code, because this unit writes none. The T3-D reviewer
recorded the cost of estimating additions from anything other than the mandated
test bodies (`tasks/m235-t3d-verdict.md` finding 1: `graph.rs` predicted ~679,
landed 734, because a `+30` addition estimate ignored 71 lines of mandated
tests).

| Item | Lines |
|---|---|
| two imports added to the existing `use` lists (`:10`, `:12-15`) | +2 |
| closures + language/unit_kind maps for P1 (2 helpers at most) | +14 |
| P1: header comment + 6 ordered steps incl. non-vacuity and both asserts | +42 |
| P2: header comment + fixture sanity + `assert_ne!` | +26 |
| **subtotal, test code** | **+84** |
| journal (2 commits, 3 gate-1 lines, the R2 panic, the disclosures) | ~70 |
| **total** | **~154** |

Rounded to **~160** to leave room for the assertion messages the handoff
requires. **Single handoff unit - fits with wide headroom.**

**Why this unit exists at all rather than being folded into T3-E:** the handoff
law permits one crate per unit, and unit E pins "You add and remove no test"
(`tasks/m235-t3e-handoff.md:161`). Units C and D could not host a `crates/ui`
test - C's touch-set is two files under `tests/integration`, D's is
`crates/graph/src/{graph.rs, closure_equivalence_tests.rs, cycle_groups.rs}` plus
its journal. So the companion parity test has nowhere to live except its own
`crates/ui`-only unit, sequenced before E.

---

## 12. Authorization and ordering

This handoff, `tasks/milestone_ledger.md` (the M23.5 row), APPROVED
`tasks/m235-t3a-handoff.md`, `tasks/m235-t3b-handoff.md`,
`tasks/m235-t3c-handoff.md`, `tasks/m235-t3d-handoff.md` **and**
`tasks/m235-t3e-handoff.md` are the sole authorization for this unit.

**Runs last-but-one.** Ordering across Phase 3, with the dependency that forces
this slot:

```
A   crates/ui           additive DTO entry point                <- done, APPROVED
B   crates/cli          CLI consumers + bench re-pin           <- done, APPROVED
C   tests/integration   Scala consumers + IV-9 gate             <- done, APPROVED
D   crates/graph        IV-11 port + ATOMIC DELETE              <- done, APPROVED
D2  crates/ui           legacy/group parity lock       *** this unit, NEXT
E   crates/ui           retire the path-based DTO                <- blocked on D2
```

**Why D2 must run BEFORE E, and this is a hard expiry, not a preference:**

1. Unit E **deletes** the legacy `build_include_elements`
   (`tasks/m235-t3e-handoff.md:48-49`, `:117-120`) and renames
   `build_include_elements_from_groups` back to `build_include_elements`. After
   E there is exactly ONE entry point, and the comparison this unit is built
   from **cannot be written at all**.
2. Unit E also **cannot host it**: its handoff pins "You add and remove no
   test" (`:161`) and pins its own suite-count gate.
3. Units C and D **cannot host it**: their touch-sets are `tests/integration`
   and `crates/graph` respectively; a `crates/ui` test file is outside both
   (`tasks/m235-t3b-verdict.md` sec5.6; reconfirmed as finding 13 in
   `tasks/m235-t3d-verdict.md`).
4. Deferring to T4.1 does not help: T4.1 is after E, by which point the legacy
   builder is gone.

**Parallel safety.** This unit touches `crates/ui/tests/ui_test/export_order.rs`
and its journal. It is file-disjoint from units B (`crates/cli/**`), C
(`tests/integration/**`) and D (`crates/graph/**`), so it was parallel-safe with
all three. It is **NOT** parallel-safe with unit E: both touch
`crates/ui/tests/ui_test/export_order.rs` (E's sites 3 and 4). In this shared
worktree, dispatch sequentially: D2, then E.

**Phase 4 (T4.1 integration closeout) remains UNAUTHORIZED.** Do not begin it,
do not run the workspace-wide gates, and do not close ledger rows F9, F10, F11
or F12 - those are operator actions at closeout. In particular **F10** (the
refused `ui_smoke.py` / `bench_compare.py`) is at its most material now and
must close before T4.1: T3-D changed what `IncludeReport::cycle_count` MEANS,
and there is still zero runtime evidence that the UI boots and serves a
group-count report. That is closeout work, not yours.
