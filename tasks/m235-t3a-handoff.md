# M23.5 T3-A Handoff - Group-Based Include-Element DTO (crate:ui, additive)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 3 of 4, unit A of 5. Plan task: **T3.2 first half** (UI DTO + node
data + edge rule + `ui_test` suites). Units B..E: `tasks/m235-t3b-handoff.md`,
`-t3c-`, `-t3d-`, `-t3e-handoff.md`.
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 3
(T3.2), sec1.3 contracts, sec1.2 IV-3/IV-9
**Prerequisites:** Phase 1 APPROVED (`dd3db11`, `e2a05e2`); Phase 2 APPROVED and
CLOSED -- T2-A `34f5fe4`+`7c94af3`, T2-B `08741b8`+`b950963`
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`, HEAD `b950963`
**Journal:** `tasks/m235-t3a-journal.md` (you create it; record your own SHAs in it)

Every signature and line number in this file was read out of the tree at
`b950963`. **If a line reference disagrees with the file you read, the FILE
wins** -- report the drift in your journal, do not silently adapt.

---

## 0. PREFLIGHT - before writing a single line of code

**Step 0.1 - ground truth.** Run `python tools/checkpoint_guard.py status`
(record it in your journal: HEAD, branch, dirty files, untracked files).

**Step 0.2 - the CRLF fixture.** `tests/fixtures/basic/artifacts/includes/
report.json` is UNPINNED by `.gitattributes` (which sets `text eol=lf` only for
`crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json`) and this
worktree has `core.autocrlf=true` (measured). A full test run therefore
re-dirties its **stat entry** even though the content is byte-identical
(ledger deferred row **M23.5 F12**; measured three times: T2-A finding 9,
T2-B finding 10 -- `git ls-files --eol` stays `i/lf w/lf`, no `\r` byte is
present, `git diff --numstat HEAD -- <path>` returns no rows).

If and only if `git diff --numstat HEAD -- <path>` shows rows for that path:
read it with the Kilo `Read` tool and write it back with the Kilo `Write` tool
so the terminators are LF. **NEVER** PowerShell `Set-Content`, `Out-File`, `>`,
`>>`, or `Get-Content | -replace` on a tracked file -- those are the documented
cause of this repo's mojibake incidents (`AGENTS.md` File-Edit Discipline).
A byte-level `open(path, "wb")` inside a Python snippet is the only other
sanctioned writer. `git checkout --` / `git restore` are mutating git and are
DENIED to every role.

**Step 0.3 - prove it.** Re-run `python tools/checkpoint_guard.py status` after
your full test run at the end of the unit; the numstat dirty section must not
list that path.

**Step 0.4 - every commit carries an EXPLICIT `--scope`** (gate 8 is written
with `--scope`). That path must appear in NO commit you make.

---

## 1. Objective and Invariants

### Objective

Add the milestone's new cycle currency to the UI payload layer as a **new,
additive, public API**: a group-based sibling of `build_include_elements` that
takes `&[CycleGroup]` instead of `&[Vec<FileId>]`, classes nodes and edges by
GROUP MEMBERSHIP (which is what kills the unstyled-edge-between-two-red-nodes
wart), and exposes two new optional node-data keys for the inspector story.

**No existing API is removed or altered in this unit.** The path-based
`build_include_elements` stays, byte-compatible, because its only production
caller (`crates/cli/src/commands/ui.rs:316`) is migrated in unit B and deleting
the old entry point needs a `crates/ui`-only commit afterwards (unit E).

### Invariants this unit establishes

- **IV-3 Edge cyclicity (the milestone's headline fix, at the DTO layer):** an
  include edge `u -> v` is classed `cycle` **iff `u` and `v` are in the SAME
  group**. Consequence: **no edge inside a group may ever be missing from the
  class**, which is exactly the edge the old path representation left unstyled.
- **IV-2 Cycle iff nontrivial (node layer):** a node is classed `cycle` iff it
  is in some group's `member_files`.
- **D27 unchanged:** the 5-argument shape survives; a missing
  `file_languages` / `file_unit_kinds` entry still **OMITS** the key (no
  fabricated default). The new `cycle_group_size` / `cycle_witness` keys follow
  the same OMITS-rather-than-default rule.
- **Determinism:** the canonical `(path asc, file_id asc)` node emission and the
  `graph.edges`-order edge emission are untouched. `cycle_witness` path labels
  must be emitted in a fixed order (ascending `FileId`, matching
  `representative_cycle` order), never in hash order.

### Invariants this unit must NOT disturb

- **IV-9 `cycle_count` naming survives.** This unit touches **no report-layer
  code**. `crates/report/src/manifests.rs:79` and
  `crates/report/src/manager/graph_io.rs:59` are byte-locked out of scope.
- **Zero new hex literals.** You touch no CSS, no JS, no template. Semantic
  verdict colors are unchanged across light, dark and dark-steampunk by
  construction; gate 5 proves no literal was added.
- **IV-11 seam, IV-12 Scala paths:** untouched here.
- **`crates/graph` is read-only in this unit.** `detect_cycle_groups` already
  exists and is already public.

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

- `crates/ui/src/dto.rs` (the new function + the shared element-emission helper)
- `crates/ui/tests/ui_test/dto_builders.rs` (new tests 1, 2, 3)
- `crates/ui/tests/ui_test/graph_api.rs` (new test 4)
- `crates/ui/tests/ui_test/export_order.rs` (new test 5)
- `crates/ui/tests/ui_test/d27_node_provenance.rs` (D27 OMITS-key lock extended)
- `tasks/m235-t3a-journal.md` (**new**, yours)

You MUST NOT touch: `crates/ui/src/lib.rs` (nothing to do -- `pub use dto::*;`
at `lib.rs:5` already exports every public item in `dto.rs`),
`crates/ui/tests/ui_test/main.rs` (do NOT add a module -- extend the existing
files, so `main.rs` stays byte-identical), any other file under
`crates/ui/tests/ui_test/`, any file under `crates/ui/assets/`,
`crates/cli/**`, `crates/graph/**`, `crates/cluster/**`, `crates/report/**`,
`tests/**`, `tools/**`, `.kilo/plans/**`, `Cargo.toml`, `Cargo.lock`,
`.gitattributes`.

No `git stash`, no branch change, no `git add .`.

---

## 3. API contracts - real signatures, read out of the tree at `b950963`

The thing you are extending, verbatim:

```rust
// crates/ui/src/dto.rs:602-608
pub fn build_include_elements(
    graph: &ariadne_graph::IncludeGraph,
    closures: &std::collections::HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    cycles: &[Vec<ariadne_repo::FileId>],
    file_languages: &std::collections::HashMap<u64, String>,
    file_unit_kinds: &std::collections::HashMap<u64, String>,
) -> Vec<CytoscapeElement>
```

The type you consume, verbatim (`crates/graph/src/cycle_groups.rs:21-31`):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleGroup {
    pub member_files: Vec<FileId>,          // ascending by FileId
    pub representative_cycle: Vec<FileId>,  // closed walk [c0..ck, c0]
    pub external_in_count: usize,
    pub external_out_count: usize,
}
```

It reaches you as `ariadne_graph::CycleGroup`: `crates/ui/Cargo.toml:18` already
depends on `ariadne-graph`, and `crates/graph/src/lib.rs:15` re-exports
`cycle_groups::*`.

### 3.1 The signature you must add (name and shape are fixed)

```rust
pub fn build_include_elements_from_groups(
    graph: &ariadne_graph::IncludeGraph,
    closures: &std::collections::HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
    cycle_groups: &[ariadne_graph::CycleGroup],
    file_languages: &std::collections::HashMap<u64, String>,
    file_unit_kinds: &std::collections::HashMap<u64, String>,
) -> Vec<CytoscapeElement>
```

Exact spelling: `build_include_elements_from_groups`. Unit B calls it by that
name; unit E renames it. Do not choose a different name.

### 3.2 What the current body does, and what changes (read `dto.rs:609-696` first)

Today, `dto.rs:612-623` derives two sets from reported PATHS:

```rust
for cycle in cycles {
    for w in cycle.windows(2) { cycle_nodes.insert(w[0]); cycle_nodes.insert(w[1]); cycle_edges.insert((w[0], w[1])); }
    if let (Some(&first), Some(&last)) = (cycle.first(), cycle.last()) {
        cycle_nodes.insert(first); cycle_nodes.insert(last); cycle_edges.insert((last, first));
    }
}
```

Your version derives membership from GROUPS instead:

- `cycle_nodes` = every `FileId` appearing in any `group.member_files`.
- `cycle_edges` is **not** a set of reported path steps. An edge is cyclic iff
  **both** of its endpoints are members of **one and the same** group. Build a
  `HashMap<FileId, usize>` (file -> index of the group containing it) and test
  membership on both endpoints plus equality of the indices. IV-1 makes groups
  disjoint, so the "same group" test is unambiguous.
- A self-loop `u -> u` is cyclic iff `u` is in some group.

Everything from `dto.rs:625` onward (in/out degree tallies, the canonical
`(path asc, file_id asc)` sort at `:638-640`, HEADER/SOURCE kind at `:648-652`,
element emission at `:674-694`) is **shared** and must behave identically for
both entry points. Factor the shared tail into one private helper and have both
public functions call it -- one responsibility, no duplicated emission loop.
Keep every function under 100 lines.

### 3.3 New node data keys

Inside the existing `serde_json::json!({...})` at `dto.rs:658-667`, and only for
grouped nodes:

- `"cycle_group_size": usize` -- the member count of the node's group.
- `"cycle_witness": Vec<String>` -- the `representative_cycle` file paths, in
  `representative_cycle` order.

Both follow the D27 OMITS rule: a node in no group gets **neither key** (not
`null`, not `0`). The existing keys `"is_cycle"`, `"id"`, `"label"`, `"path"`,
`"kind"`, `"closure"`, `"in_degree"`, `"out_degree"` are unchanged, and their
JSON insertion order is unchanged (`serde_json::json!` preserves it), so node
serialization stays byte-comparable.

---

## 4. Red - two compiling Reds, both real

**A test that fails to compile is NOT Red.** Both Reds below are written
against `build_include_elements`, which exists today, and both fail on a real
assertion. Neither is a `todo!()`, a skeleton, or an unimplemented signature.

### R1 (Red) - IV-3: the chord-blind intra-group edge

Add to `crates/ui/tests/ui_test/dto_builders.rs`:

```
intra_group_non_tree_edge_carries_the_cycle_class
```

Fixture: three registered files with include edges `a -> b`, `b -> c`,
`c -> a`, plus the chord `a -> c`. Call `build_include_elements` with the cycles
argument set to the single reported path `vec![vec![a, b, c]]` (what the old
representation reports for this graph) and assert that the edge element whose
`source == a` and `target == c` has `data["is_cycle"] == true` and
`classes == Some("cycle")`.

**Expected failure reason:** assertion (2) fails and names the pair
`(a, c)`. The old body only ever inserts `cycle_edges` from reported path steps
plus the `(last, first)` closing pair, so `(a, c)` is absent and the edge emits
`is_cycle: false` with `classes: None` -- the unstyled edge between two nodes
that both carry the `cycle` class. This is plan sec1.1 item 1, reproduced at the
DTO layer.

### R2 (Red) - the inspector node data

Add to the same file:

```
node_cycle_data_reports_group_size_and_witness_only_for_grouped_nodes
```

Same fixture. Assert the node element for `b` has
`data["cycle_group_size"] == 3` and
`data["cycle_witness"]` equal to the three member paths in ascending `FileId`
order; and that the node element for an ungrouped fourth file `d` (edge
`c -> d`, on no cycle) has **neither** key present at all.

**Expected failure reason:** `data["cycle_group_size"]` is
`serde_json::Value::Null` because the key does not exist in today's `json!({})`;
the assertion message names the missing key and prints the actual node `data`
map so the failure is self-explaining.

### Green

Implement `build_include_elements_from_groups` (sec3.1-3.3) with the shared
emission helper. Then **re-point R1 and R2 from `build_include_elements` to
`build_include_elements_from_groups`**, changing the cycles argument from
`vec![vec![a,b,c]]` to one `CycleGroup { member_files: vec![a,b,c], .. }`
(a real group, built from `graph.detect_cycle_groups()` on the fixture graph --
prefer that over a hand-built literal, so the test exercises the real producer).

**Log this call-site switch as an explicit journal step.** It is the one place
where a Red written against the old API becomes a Green against the new one, and
the reviewer will look for it. This is the same pattern Phase 2's T2-A handoff
prescribed and its reviewer accepted.

---

## 5. Required tests - exact names, exact count

`crates/ui/tests/ui_test/` must gain **exactly five** `#[test]` functions. The
target's count at `b950963` is **179**; after this unit it must be **184**.

| # | Name | File | Locks |
|---|---|---|---|
| 1 | `intra_group_non_tree_edge_carries_the_cycle_class` | `dto_builders.rs` | IV-3 (was R1) |
| 2 | `node_cycle_data_reports_group_size_and_witness_only_for_grouped_nodes` | `dto_builders.rs` | sec3.3 (was R2) |
| 3 | `grouped_nodes_and_edges_are_classed_cycle_from_membership_not_from_reported_paths` | `dto_builders.rs` | IV-2 + IV-3 node/edge rule |
| 4 | `include_elements_export_is_stable_across_two_calls_with_cycle_fixtures` | `graph_api.rs` | determinism incl. `cycle_witness` ordering |
| 5 | `grouped_nodes_keep_the_canonical_emission_order_and_edge_order` | `export_order.rs` | canonical `(path asc, file_id asc)` + `graph.edges` order preserved |

Plus one **extension** to the existing test
`d27_file_node_decoration_is_omitted_when_map_entry_absent` in
`crates/ui/tests/ui_test/d27_node_provenance.rs` (`:135-148`, the locked D27
OMITS-default lock): add a case where that node IS in a cycle group and assert it
still OMITS `language` and `unit_kind` when the D27 maps have no entry for it, and
that it carries `cycle_group_size` / `cycle_witness`. **Extend that existing test;
do not add a sixth `#[test]`** (the count must be 184).

Test 3 is the one that cannot pass by accident: assert that a file which is in
some group's `member_files` but appears in NO reported path of the old
representation is still classed `cycle`, and that an edge between a grouped file
and an ungrouped file is NOT classed `cycle`.

Test 5 is the plan's "byte-stable export" requirement in its in-process,
testable form: call the builder twice on the same graph and assert the emitted
`id` sequence AND every node's `data` map are identical, and that the node array
still follows canonical path-then-id order. The **cross-process** half of that
lock is not yours: `tests/golden/**` is byte-locked out of your scope and the
`--export` byte-stability check belongs to the reviewer's acceptance pass. Say
so in your journal rather than claiming you proved it.

---

## 6. Refactor and hygiene targets

- Functions < 100 lines. Files < 1000 lines. `crates/ui/src/dto.rs` is 852 lines
  today -- **this is the ceiling at risk.** Your additions are ~165 lines; if
  you push the file past 1000, `python tools/hygiene_check.py` R2 fails
  blocking. If your final `dto.rs` exceeds ~940 lines, extract the
  group-membership helpers into a new `crates/ui/src/cycle_groups_view.rs` plus a
  one-line `pub(crate) mod cycle_groups_view;` in `crates/ui/src/lib.rs` -- and
  if you take that branch you MUST declare `crates/ui/src/lib.rs` in your
  journal as a deliberate touch-set extension with your reason (one line, no
  other file changes). Do not silently exceed the ceiling.
- No `tmp`, `res`, `foo`, `bar`, `test1`; no single-letter locals in code you
  author; one responsibility per function; no source line >= 2000 characters.
- The shared emission helper must not become a parameter soup. If you find
  yourself passing five booleans, that is the signal to introduce a small private
  struct.
- `python tools/hygiene_check.py`: zero R1/R2. R3 is warn-only, but every name
  YOU author must be intention-revealing.

---

## 7. Acceptance - gates, exact commands, expected verdict lines

Every gate is written in DIRECT form. **Never write a gate as
`python tools/distill.py -- <argv>`** unless `<argv>` is a `cargo build`; the
distiller is an argv pass-through, so that form runs whatever `<argv>` names.

**Never write a bare-word test filter.** `python tools/testrun.py ... -- <word>`
passes `<word>` to cargo as a TEST-NAME filter. Phase 1 wrote a gate that way,
it selected zero tests, and it returned `0 passed / 0 failed / 0 ignored /
7 suites` with exit 0 -- it could never have observed a Red either (Phase 1
verdict finding 2). Use `--test <target>`, which selects a real binary.

**8 invocations -- the whole allowance.**

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui
3. python tools/distill.py -- cargo build -p ariadne-ui
4. python tools/hygiene_check.py
5. python tools/theme_token_check.py --check-added --base b950963
6. python tools/encoding_guard.py --staged
7. python tools/checkpoint_guard.py status
8. python tools/commit_scoped.py --scope 'crates/ui/src/dto.rs' --scope 'crates/ui/tests/ui_test/dto_builders.rs' --scope 'crates/ui/tests/ui_test/graph_api.rs' --scope 'crates/ui/tests/ui_test/export_order.rs' --scope 'crates/ui/tests/ui_test/d27_node_provenance.rs' --scope 'tasks/m235-t3a-journal.md' -m "feat(ui): add group-based include element builder with cycle group data"
```

**Expected verdict lines (Green):**

| Gate | Expected verbatim line |
|---|---|
| 1 | `184 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `184 passed / 0 failed / 0 ignored / 3 suites` |
| 3 | `ok (exit 0, 0 warnings emitted)` |
| 4 | zero R1/R2 lines; R3 warns, if any, are legacy files only |
| 5 | no added hex literals (zero findings) |
| 6 | clean, no output |

**The `184 / 179` arithmetic, so you can check it rather than trust it:**
`crates/ui/tests/ui_test/` contains exactly **179** `#[test]` / `#[tokio::test]`
functions at `b950963` (counted across all 19 modules declared in
`crates/ui/tests/ui_test/main.rs`; `#[tokio::test]` expands to one `#[test]`, and
there is **no** `#[ignore]` anywhere in that directory). This unit adds exactly
**five** -> 184. `crates/ui/src/` contains **zero** `#[test]` functions, so the
lib target contributes 0; gate 2's three suites are lib + `ui_test` +
doc-tests.

**Gate 3 must report ZERO warnings.** Phase 1's reviewer treated a warning-free
build as load-bearing evidence; an unused helper is a finding, not a style note.

**Effect on the canonical workspace line:** the reviewer's acceptance pass should
read **`1174 passed / 0 failed / 4 ignored / 88 suites`** -- `1169 + 5`, suites
unchanged because `ui_test` is an existing target. Do not claim that number: you
do not run the workspace-wide gate. It belongs to the reviewer.

---

## 8. Frozen locks - each with a proving command

| Lock | Proving command | Expected |
|---|---|---|
| `crates/ui/src/lib.rs` byte-unchanged | reviewer's per-commit `git diff --name-only` | absent from the commit |
| `crates/ui/tests/ui_test/main.rs` byte-unchanged | same | absent from the commit |
| Path-based `build_include_elements` still present and unedited | gate 1 (`common.rs:157` and 6 other in-crate call sites still compile against it) | green |
| IV-9 `cycle_count` field name + `crates/report/**` untouched | reviewer's `git diff --name-only`; no `crates/report` path in any commit | absent |
| zero new hex literals, verdict colors intact across all three themes | gate 5 | zero findings |
| R1/R2 hygiene | gate 4 | no R1/R2; `dto.rs` < 1000 |
| UTF-8 mojibake | gate 6 | clean, no output; authored content pure ASCII |
| no cross-crate dependency change | reviewer's `git diff` on `crates/ui/Cargo.toml` | byte-unchanged |
| touch-set | reviewer's per-commit `git diff --name-only` | exactly the six in-set paths |
| `tests/fixtures/**`, `tests/golden/**` | reviewer's `git diff --name-only` | absent |
| `detect_cycles`/`dfs_cycle` still present (unit D owns the delete) | reviewer's grep | both still at `crates/graph/src/graph.rs:234` and `:260` |
| no third cycle representation | reviewer | `detect_cycle_groups` and `detect_cycles` are the only two |

Workspace-wide gates are NOT yours: `python tools/testrun.py --baseline`,
`python -m unittest discover -s tools -p "*_tests.py"`,
`python tools/ui_smoke.py`, `python tools/bench_compare.py`. Note that
`--baseline` is currently BROKEN (ledger F9: `usage error: missing value for
--baseline`) and `ui_smoke` / `bench_compare` are REFUSED by the reviewer
allowlist (ledger **F10**). Neither is your problem, and you must NOT route a
refused command through `distill.py` to evade the denial.

---

## 9. Reviewer gate reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
sec2. The reviewer will audit:

1. **IV-3 at the DTO layer.** R1 must have been genuinely Red against the path
   representation, with the failure naming the chord pair, and Green only after
   the re-point. An implementer who cannot show the Red has not shown the fix.
2. **Group-identity correctness.** The edge rule must be "both endpoints in the
   SAME group" (index equality), not "both endpoints in SOME group". A file in
   group 1 and a file in group 2 must produce an UNCLASSED edge. Test 3 is the
   lock; verify it actually exercises that case.
3. **Shared emission, not duplicated emission.** Exactly one node-emission loop
   and one edge-emission loop in `dto.rs`. Two copies is a finding.
4. **D27 OMITS rule.** Grouped nodes must still omit absent `language` /
   `unit_kind` keys, and must omit `cycle_group_size` / `cycle_witness` when
   ungrouped -- never `null`, never `0`.
5. **Additive-only.** `build_include_elements` unedited, `crates/cli` unedited,
   `crates/graph` unedited. The coexistence is the plan-sanctioned temporary
   state; unit E removes the old entry point.
6. **Line-count discipline** on `dto.rs` (852 today) and the honesty of any
   declared touch-set extension.
7. **Deviation adjudication.** Any deviation you self-flag is adjudicated on its
   evidence, not accepted because it was declared.

**Report back (final message):** commit SHAs (also in your journal -- Phase 1's
implementer omitted them and the reviewer had to recover them from `git log`,
verdict finding 8); Red evidence for R1 and R2 with the verbatim failure
messages; Green evidence quoting the canonical lines and capture paths for gates
1, 2 and 3; the before/after `dto.rs` line count; every deviation with its
reason; out-of-scope observations, report-only. Never paste raw cargo or git
output -- cite the command, the canonical line, the exit code, and the capture
path. If a capture path will be overwritten by a later run, say so; do not cite
it as retained evidence.

---

## 10. Discipline clauses

- **Command execution.** `bash` is an allowlist restricted to the distilling
  entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED.
  There is no `tool-caller` in this topology and no delegation for commands.
- **Reading a refusal.** A refused command ALWAYS reports the catch-all deny
  rule (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`),
  whatever the real cause -- including "that command is simply not on the
  allowlist". Treat that text as boilerplate. Attempt each gate INDEPENDENTLY.
  Never infer from one refusal that command execution is unavailable, and NEVER
  drop a gate because its first invocation was refused. If a gate genuinely
  cannot run, record the VERBATIM refusal in your journal rather than silently
  narrowing the acceptance set. Phase 1's implementer shipped a Red they never
  saw because one invocation was refused; do not repeat that.
- **Strict TDD.** R1, observe, R2, observe, then Green, then re-point both Reds
  to the new API, then Refactor. Never weaken an assertion to make it pass.
  Never commit a Red state.
- **Code hygiene.** Functions < 100 lines, files < 1000 lines, no
  `tmp`/`res`/`foo`/`bar`/`test1`, no single-letter locals in code you author,
  one responsibility per function, no source line >= 2000 characters.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>`, or
  `Get-Content | -replace` on a tracked file. Use the Kilo Read/Edit/Write tools,
  or byte-level `open(path, "wb")` in a script. Authored content is pure ASCII;
  build any non-ASCII test signature from byte escapes (`b"\xc3\x82\xc2\xa7"`),
  never a literal. When quoting git content, quote the WHOLE argument:
  `git show "rev^:path"`.
- **Scope.** Touch only sec2. On discovering out-of-scope breakage: report it,
  do not fix it.
- **Journal.** Append a 1-3 line checkpoint after EACH Green gate: what changed,
  commit SHA, any deviation, any lock or contract finding. Record your own SHAs
  in it. The journal is what a successor session reads; without it, recovery
  restores code but loses reasoning.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A
  dead session is never resumed -- a successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3a-handoff.md --scope ... --cmd ...`
  (skill `resume-checkpoint`).

---

## 11. Sizing check (handoff law, stated explicitly)

| Metric | Bound | This unit |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 top-level module directory | `crates/ui` only -- **1 crate** |
| Gate commands | ~10 | **8** |
| Produced lines (implementation + test diff) | ~1200 | **~560** (`dto.rs` +165/-25, four test files ~+300, journal ~70, `dto.rs` net ~+140 -> total diff output ~560) |

**Single handoff unit -- fits on all three axes.**

Why plan Phase 3 could not be one or two units: T3.1+T3.2 span `crates/graph`,
`crates/cli`, `crates/ui` and `tests/integration`, and the deletion of
`detect_cycles` is only legal in a commit that also repoints its last caller.
Plan Phase 3 is therefore split into **five** one-crate units (A, B, C, D, E --
see `tasks/m235-t3a-handoff.md` sec12 and the M23.5 milestone row). This unit
is the additive first half of T3.2 and is the seam that lets T3.2's other half
(unit B, the CLI rewiring) land as a `crates/cli`-only commit: adding a NEW
entry point rather than changing the existing signature is what keeps
`crates/cli` compiling across the boundary.

---

## 12. Authorization and ordering

This handoff, together with `tasks/m235-kickoff.md`, the M23.5 milestone row in
`tasks/milestone_ledger.md` (Phase 1 APPROVED, Phase 2 APPROVED and CLOSED, HEAD
`b950963`, canonical workspace line `1169 / 0 / 4 / 88`), and the Phase 2
verdicts, is the sole authorization for this unit.

**Run it first.** Units B and E depend on the entry point it adds; unit C is
file-disjoint from this unit and from B and may run in parallel **only in a
separate worktree**. Unit D (`tasks/m235-t3d-handoff.md`) -- the one that deletes
`detect_cycles`/`dfs_cycle` -- requires units B and C to have landed, because
after them `detect_cycles` has no caller outside `crates/graph`.

Phase 4 (T4.1 integration closeout) remains UNAUTHORIZED. Do not begin it. Do
not delete `detect_cycles` or `dfs_cycle`, do not repoint `crates/graph/src/graph.rs:356`,
`:534` or `closure_equivalence_tests.rs:515`, do not touch
`crates/cli/src/commands/ui.rs:310`, `crates/cli/src/commands/includes.rs:113`
or `crates/cli/tests/payload_tail_bench.rs` -- all of that is units B and D.