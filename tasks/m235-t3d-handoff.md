# M23.5 T3-D Handoff - IV-11 Policy Port + ATOMIC DELETE of detect_cycles/dfs_cycle (crate:graph)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 3 of 4, unit D of 5. Plan task: **T3.1 report + policy seam**, and it
is the unit that carries the milestone's **D1-B delete**.
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 3
(T3.1), sec1.2 **IV-2, IV-9, IV-10, IV-11**, sec1.3, sec1.4, sec3 item 1
**Prerequisites (BOTH must be APPROVED and landed):** unit B
`tasks/m235-t3b-handoff.md` (crates:cli) and unit C
`tasks/m235-t3c-handoff.md` (tests/integration). See sec0 step 0.3.
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`
**Journal:** `tasks/m235-t3d-journal.md` (you create it; record your own SHAs in it)

Every signature and line number was read out of the tree at `b950963`. **If a
line reference disagrees with the file you read, the FILE wins** -- report the
drift in your journal.

---

## 0. PREFLIGHT

**Step 0.1** - `python tools/checkpoint_guard.py status`; record HEAD, branch,
dirty files, untracked files.

**Step 0.2** - the unpinned CRLF fixture. `tests/fixtures/basic/artifacts/
includes/report.json` is not covered by any `.gitattributes` pin (which sets
`text eol=lf` only for `crates/ui/tests/fixtures/*.json` and
`tests/golden/**/*.json`) and this worktree has `core.autocrlf=true`, so a full
test run re-dirties its **stat entry** even when content is byte-identical
(ledger deferred row **M23.5 F12**; measured three times: T2-A finding 9, T2-B
finding 10). If and only if `git diff --numstat HEAD -- <path>` shows rows:
restore with the Kilo `Read` tool then the Kilo `Write` tool. NEVER
`Set-Content`, `Out-File`, `>`, `>>`, `Get-Content | -replace`. `git checkout --
` / `git restore` are mutating git and are DENIED.

**Step 0.3 - the ordering precondition. Do not skip this.** The delete in this
unit is only legal because every caller OUTSIDE `crates/graph` has already been
migrated. Verify by grepping the tree yourself:

```
detect_cycles|dfs_cycle
```

You must find `detect_cycles` / `dfs_cycle` referenced ONLY inside
`crates/graph/src/` -- in `graph.rs` (the definition at `:234`, the two internal
`Self::dfs_cycle` calls at `:246` and `:275`, the `dfs_cycle` definition at
`:260`, the report call at `:356`, the unit-test call at `:534`) and in
`closure_equivalence_tests.rs:515`. If **any** reference remains in
`crates/cli/**`, `crates/ui/**`, `crates/report/**`, `crates/cluster/**` or
`tests/**`, **STOP and report**: units B and C have not landed (or missed a site)
and deleting now would break the workspace build.

**Expected remaining non-production references at this point:** the test-local
frozen oracle in `crates/graph/tests/cycle_groups_equivalence_test.rs`
(`old_detect_cycles` at `:90`, `old_dfs_cycle` at `:116`, plus their doc comments
at `:4`, `:7`, `:65`). **That oracle is a deliberate frozen copy** (IV-6) and
**must survive this delete** -- do NOT delete or rename it. Phase 2's reviewer
confirmed it calls nothing on the live method and therefore cannot be broken by
this swap.

**Step 0.4** - every commit carries an EXPLICIT `--scope` (gate 10). That fixture
path must appear in NO commit.

---

## 1. Objective and Invariants

### Objective

Port the cycle-policy classification from **paths** to **groups** (IV-11 -- the
milestone's one deliberate semantic change), repoint the three in-crate
`detect_cycles` callers, and **delete `detect_cycles` and `dfs_cycle` in the same
commit as the last of those repoints**. After this unit the workspace has exactly
one cycle representation and plan sec3 item 1 is discharged.

### Invariants this unit establishes

- **IV-11 port, concretely:** a group is informational **iff every member file's
  policy is non-`Reject`**. The callback signature
  `&dyn Fn(FileId) -> CyclePolicy` and the field name
  `informational_cycle_count` and its **skip-when-zero serde shape** are
  preserved exactly. `generate_report()` stays the all-`Reject` wrapper, so every
  C++ artifact stays byte-stable.
- **IV-9 `cycle_count` naming survives**, now carrying group count.
- **IV-2 both directions** stay locked by T2-B's frozen-oracle suite, which is
  the licence this repoint rests on.
- **IV-10 closure parity untouched:** `closure_equivalence_tests.rs:515` migrates
  from `!graph.detect_cycles().is_empty()` to
  `!graph.detect_cycle_groups().is_empty()`. The two are equal on every
  registered-endpoint graph in the corpus (that is exactly what T2-B proved), and
  the suite's own green run is the witness.
- **IV-7 no unbounded recursion:** the recursive `dfs_cycle` is gone from the
  include-graph path. The one recursion left in `crate:graph` after this unit is
  `format_explain_node` (`graph.rs:395-427`), which is **out of this milestone's
  scope** -- note it in your journal, do not touch it.

### Invariants this unit must NOT disturb

- **IV-8 planner byte-identity:** `crates/cluster/**` is byte-locked out of
  scope. `planner_facts_parity`, `planner_characterization` and
  `m00_facts_path_parity_test` must stay green unedited.
- **`crates/report/**` byte-locked.** It only forwards `cycle_count`
  (`manifests.rs:79`, `manager/graph_io.rs:59`) and the field name is unchanged.
- **`crates/graph/tests/cycle_groups_test.rs` (9 tests) and
  `cycle_groups_equivalence_test.rs` (4 tests) byte-unchanged.** They are the
  oracles, not this unit's work.
- **`tests/golden/**` and `tests/fixtures/**` byte-unchanged.** The goldens read
  `"cycle_count": 0` on acyclic fixtures, so the report change is a byte no-op
  there. If `m00_golden_baseline_test` ever fails, that is a FINDING -- never
  regenerate a golden.

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

- `crates/graph/src/graph.rs` (the seam, the three repoints, the DELETE, the
  `adjacency()` doc fix, the new tests)
- `crates/graph/src/closure_equivalence_tests.rs` (one line, `:515`)
- `crates/graph/src/cycle_groups.rs` (the `FrameStep` disposition only -- sec7)
- `tasks/m235-t3d-journal.md` (**new**, yours)

You MUST NOT touch: `crates/graph/tests/cycle_groups_test.rs`,
`crates/graph/tests/cycle_groups_equivalence_test.rs` (the frozen oracle),
`crates/graph/src/lib.rs`, `crates/graph/src/types.rs`,
`crates/graph/src/closures.rs`, `crates/graph/src/load*.rs`,
`crates/graph/Cargo.toml`, any other crate, `tests/**`, `tools/**`,
`.kilo/plans/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`.

No `git stash`, no branch change, no `git add .`.

---

## 3. API contracts - the real code you are changing, read at `b950963`

### 3.1 The classification helper that becomes a group rule

```rust
// crates/graph/src/graph.rs:297-303  -- CURRENT, path-based
/// A cycle is informational iff **every** member file's policy is
/// non-`Reject` (M20 D11, IV-3); a mixed-member cycle is blocking.
fn cycle_is_informational(cycle: &[FileId], policy_of: &dyn Fn(FileId) -> CyclePolicy) -> bool {
    cycle
        .iter()
        .all(|&file_id| !matches!(policy_of(file_id), CyclePolicy::Reject))
}
```

Becomes, named exactly as plan sec1.4 mandates:

```rust
// A group is informational iff **every** member file's policy is non-`Reject`
// (M20 D11 ported to partition semantics, IV-11); a mixed-member group is blocking.
fn group_is_informational(group: &CycleGroup, policy_of: &dyn Fn(FileId) -> CyclePolicy) -> bool {
    group
        .member_files
        .iter()
        .all(|&file_id| !matches!(policy_of(file_id), CyclePolicy::Reject))
}
```

**`group_is_informational` is the name Phase 2 deliberately deferred.** T2-A's
deviation E recorded that it was NOT added to avoid a `dead_code` warning, and
its reviewer ACCEPTED that ("confirmed genuinely absent: zero matches across
`crates/`"). This is the unit where it belongs, and this is the call site that
makes it live. Do not re-defer it.

It is an inherent associated fn on `IncludeGraph` today (called as
`Self::cycle_is_informational` at `graph.rs:359`) even though it takes no `self`;
keep that shape -- changing it to a free fn is an unrequested refactor.

### 3.2 The report function

```rust
// crates/graph/src/graph.rs:310-313  -- signature FROZEN, do not touch
pub fn generate_report_with_cycle_policies(
    &self,
    policy_of: &dyn Fn(FileId) -> CyclePolicy,
) -> IncludeReport
```

and the four lines inside it that change:

```rust
// :356-360  CURRENT
let detected_cycles = self.detect_cycles();
let informational_cycle_count = detected_cycles
    .iter()
    .filter(|cycle| Self::cycle_is_informational(cycle, policy_of))
    .count();

// :369-370  CURRENT
cycle_count: detected_cycles.len(),
informational_cycle_count,
```

becomes group-based: `let detected_groups = self.detect_cycle_groups();`, the
filter over `Self::group_is_informational`, and `cycle_count:
detected_groups.len()`. `generate_report()` at `:293-295` is untouched.

### 3.3 The DELETE - `crates/graph/src/graph.rs:233-287`, 55 lines

Remove, in one commit:

| Lines | What |
|---|---|
| `:233-234` | the `/// Detects cycles ... DFS traversal.` doc + `pub fn detect_cycles(&self) -> Vec<Vec<FileId>>` |
| `:235-258` | the body, including `let adj = self.adjacency();` at `:242` and the `Self::dfs_cycle(` call at `:246` |
| `:260-287` | `fn dfs_cycle(...)` including the recursive call at `:275` |

**This is the plan sec3 item 1 discharge.** After this commit `grep` finds zero
production `detect_cycles` / `dfs_cycle` in the workspace, and the only remaining
mentions are the frozen test-local oracle and documentation prose.

### 3.4 The three in-crate repoints

| Line | Current | Replace with |
|---|---|---|
| `graph.rs:356` | `self.detect_cycles()` | `self.detect_cycle_groups()` |
| `graph.rs:534` | `let cycles = graph.detect_cycles();` (in `test_cycle_detection`, `:501-539`) | `let cycle_groups = graph.detect_cycle_groups();` and its `assert!(!cycles.is_empty(), ...)` becomes `assert!(!cycle_groups.is_empty(), ...)`. **The fixture at `:501-539` is `a.h` <-> `b.h` -- one SCC, one group -- so the assertion's truth is unchanged and no number moves.** |
| `closure_equivalence_tests.rs:515` | `let cyclic = !graph.detect_cycles().is_empty();` | `let cyclic = !graph.detect_cycle_groups().is_empty();` |

### 3.5 `graph.rs` size arithmetic (check it, do not assume it)

`graph.rs` is **704 lines** at `b950963` and has been byte-unchanged since the
pre-migration blob `2708425`. After this unit: 704 - 55 (deleted) + ~+30 (the
group helper, the renamed locals, the two new tests and their fixture helper) =
**~679 lines**. Plan sec1.4's ceiling pressure is already discharged by the
extracted `crates/graph/src/cycle_groups.rs` (527 lines); the deletion is not what
discharges it, and the plan's T2.1 refactor note claiming otherwise was corrected
on 2026-10-04.

---

## 4. Red - two compiling Reds, and they are the sharpest evidence in Phase 3

Both Reds live in `crates/graph/src/graph.rs`'s existing `#[cfg(test)] mod tests`
(which starts at `:430`) and both **compile against the tree exactly as it stands
today**. Neither is a `todo!()`, a skeleton, or an unimplemented signature.

### The discriminating fixture - read this twice, it is the whole trick

`mutual_cycle_graph()` (`graph.rs:602-610`) is a 2-cycle: one reported path and
one group, so it **cannot discriminate** path semantics from group semantics.
You need a graph where the old DFS reports MORE THAN ONE PATH for a single SCC.

Fixture `double_reporting_scc()` -- three registered files with include edges:

```
1 -> 2      (a -> b)
2 -> 3      (b -> c)
3 -> 1      (c -> a)
3 -> 2      (the extra chord c -> b)
```

Trace the frozen oracle at `graph.rs:268-286` by hand before you write the test,
and record the trace in your journal:

- DFS seeds `[1,2,3]` sorted. `dfs_cycle(1)`: push 1; neighbour 2 unvisited ->
  `dfs_cycle(2)`: push 2; neighbour 3 unvisited -> `dfs_cycle(3)`: push 3.
- `3`'s neighbours are `[1, 2]`. `1` is on `rec_stack` at `position 0` -> report
  `path[0..] = [1, 2, 3]`. `2` is on `rec_stack` at `position 1` -> report
  `path[1..] = [2, 3]`.
- Unwind; 2 and 3 are now visited, so the outer loop skips them.

**Old `detect_cycles()` returns `[[1, 2, 3], [2, 3]]` -- two cycles for one SCC.
`detect_cycle_groups()` returns ONE group `{1, 2, 3}`.** All three files are
registered, so IV-2 equality holds and the MEMBERSHIP answer is identical; only
the COUNT differs. That isolates the semantic port perfectly.

### R1 (Red) - IV-9 / plan sec1.1 item 2: count groups, not paths

Add to `graph.rs`'s `mod tests`:

```
cycle_count_counts_entangled_groups_not_reported_paths
```

Build `double_reporting_scc()`, call `generate_report()`, and assert
`report.cycle_count == 1`.

**Expected failure reason:** the assertion fails with `left: 2`, `right: 1`. Your
assertion message must name both old reported paths (`[1, 2, 3]` and `[2, 3]`) so
the failure is self-explaining. This is plan sec1.1 item 2 verbatim: `cycle_count`
stops exploding combinatorially on dense entangled clusters.

### R2 (Red) - IV-11: the group is blocking even though one old path was not

Same fixture. Stance: `FileId(1)` -> `CyclePolicy::Reject`, everything else ->
`CyclePolicy::Tolerate`. Assert:

1. `report.informational_cycle_count == 0`, and
2. the serialized JSON does **not** contain `"informational_cycle_count"`.

**Expected failure reason, and this is the one to get right in your journal:**
under the OLD path rule, `cycle_is_informational([1,2,3])` is false (member 1
rejects) but `cycle_is_informational([2,3])` is **true** (both tolerate), so the
old report says `informational_cycle_count == 1` and the JSON **does** carry the
key. Both of your assertions fail. Under the group rule the single group `{1,2,3}`
contains a rejecting member, so it is BLOCKING: 0 informational, key absent.

That is the entire milestone's semantic port in one fixture: a mixed-member
group is blocking even when some DFS path through it happens to be
all-tolerating. This is why the port is necessary and not cosmetic.

### Green

Implement sec3.1-3.4 and re-point R1 and R2 from the path expectation to the
group one -- R1 and R2 already call `generate_report()`, so unlike Phase 2's
T2-A Reds there is **no call-site switch to make**: they are already written
against the report surface, which is exactly what changes underneath them. Say
in your journal that you observed both Reds against the unported report.

### The six existing `informational_cycle_count` tests - what "port" means here

They live at `graph.rs:612-680`:

| # | Name | Line |
|---|---|---|
| 1 | `cycle_with_all_members_tolerating_is_informational` | `:613` |
| 2 | `cycle_with_one_rejecting_member_is_not_informational` | `:622` |
| 3 | `plain_tolerate_also_counts_as_informational` | `:636` |
| 4 | `default_report_never_classifies_cycles_as_informational` | `:645` |
| 5 | `informational_cycle_count_key_is_skipped_from_json_when_zero` | `:652` |
| 6 | `all_reject_policy_json_is_byte_identical_to_default_report` | `:670` |

**Plan sec2 T3.1 cites them as "graph.rs ~:614-687". The real range is
`:612-680`.** The plan's citation is stale by roughly seven lines; record that
drift in your journal (it is the same class of defect Phase 2 found in its own
handoff artifacts).

**All six keep their exact assertion values and you must not change a single
number in them.** Each is built on `mutual_cycle_graph()`, where one reported
path equals one group, so every stance (`Reject` -> 0 informational + key
absent; all-tolerate -> equals the count; the two serde byte-stability locks) is
already group-correct. "Porting" them means:

- update the assertion **messages** and any doc comment that says "cycle" where
  it now means "group", so a reader is not misled about what is counted;
- keep the two serde locks (tests 5 and 6) byte-for-byte in their assertions --
  they are the C++ artifact byte-stability evidence.

If you find yourself wanting to change a number in one of the six, STOP: that is
the signal that you have misread the fixture. The discriminating coverage is R1
and R2, not these six.

`report_keeps_import_edges_out_of_cpp_facing_kind_counters` (`:683`) and
`report_counts_cpp_only_edge_set_currently` (`:581`) are unrelated to cycles --
leave them completely alone.

---

## 5. Required tests - exact names, exact count

`crates/graph/src/graph.rs`'s `mod tests` must gain **exactly two** `#[test]`
functions: `cycle_count_counts_entangled_groups_not_reported_paths` (R1) and
`double_reporting_scc_with_one_rejecting_member_is_blocking` (R2 -- name it this,
so the fixture and the stance are legible from the name). Plus **one** new private
fixture helper `double_reporting_scc()`, modelled on the existing
`mutual_cycle_graph()` at `:602-610`.

**`-p ariadne-graph` must report 118 tests**, not 116: the lib target carried
101 at `b950963`, plus 2 new = 103.

---

## 6. Refactor and hygiene targets

- Functions < 100 lines. Files < 1000 lines; `graph.rs` lands near 679 and
  `cycle_groups.rs` at 527, both far from the ceiling. All functions < 100 lines.
- No `tmp`, `res`, `foo`, `bar`, `test1`; no single-letter locals in code you
  author; one responsibility per function; no source line >= 2000 characters.
- No new adjacency loop. `adjacency()` (`graph.rs:83-95`) is the single source;
  `detect_cycle_groups` already calls it at `cycle_groups.rs:42` and must keep
  doing so.
- No cross-crate dependency: `crates/graph/Cargo.toml` stays byte-unchanged and
  gains nothing. `graph` and `cluster` remain decoupled (D-A7).
- **Authored content is pure ASCII.**
- `python tools/hygiene_check.py`: zero R1/R2; every name you author
  intention-revealing.

---

## 7. Carried findings this unit owns

### F4 (T2-A finding 4) - the `adjacency()` doc comment understates its call sites

`graph.rs:78-82` currently reads, in part:

> "The single adjacency-build call sites are `compute_closures` and `LoadAnalyzer`'s
> `file_adjacency` memo -- no other code may duplicate this loop."

I verified every call site in the tree at `b950963`:

| Site | Kind |
|---|---|
| `graph.rs:122` (`compute_closures`) | production |
| `graph.rs:143` (`compute_closures_bfs_oracle`) | test-only, `#[cfg(test)]` |
| `graph.rs:242` (`detect_cycles`) | production -- **DELETED by this unit** |
| `cycle_groups.rs:42` (`detect_cycle_groups`) | production |
| `load_caches.rs:339` (the LoadAnalyzer memo) | production |
| `closure_equivalence_tests.rs:231`, `:528` | test-only |

So the comment names 2 of 5 production sites and omits both `#[cfg(test)]`
families. **Fix it in this unit** -- this is the unit that touches `graph.rs`:
name every production call site and say plainly that the test-side callers
(`compute_closures_bfs_oracle`, `closure_equivalence_tests`) are separate,
deliberate consumers and not a licence to add another loop. Keep the "no other
code may duplicate this loop" prohibition.

### F5 (T2-A finding 5) - `FrameStep` is a boolean in disguise

`crates/graph/src/cycle_groups.rs:94-102` declares three variants --
`Descended`, `EdgeVisited`, `NeighborsExhausted` -- and the sole consumer at
`:145-152` (`explore_from`) tests only one:

```rust
while !self.frames.is_empty() {
    if let FrameStep::NeighborsExhausted = self.advance_frame(adjacency) {
        self.retire_frame();
    }
}
```

No warning fires because the other two are constructed, and plan sec1.4 mandates
the `advance_frame` decomposition, so Phase 2's reviewer recorded it as an
observation rather than a defect.

**Preferred fix (do this):** make all three variants load-bearing by matching all
three in `explore_from`:

```rust
match self.advance_frame(adjacency) {
    FrameStep::Descended | FrameStep::EdgeVisited => continue,
    FrameStep::NeighborsExhausted => self.retire_frame(),
}
```

This is behaviour-identical, costs four lines, and makes the enum say what it
means at its one call site.

**Pre-authorised fallback:** if you conclude the match reads worse than the
`if let`, leave `explore_from` alone and instead record in your journal a
**stated deferral** naming (a) the finding, (b) that plan sec1.4 mandates the
`advance_frame` decomposition so the enum is not gratuitous, and (c) that the two
unused variants are constructed (so no `dead_code` warning exists -- which is the
reviewer's load-bearing concern, and gate 4 measures it). **A stated deferral with
those three elements is an acceptable outcome; silently ignoring the finding is
not.**

### Findings this unit does NOT own

T2-A findings 1, 2, 3 and T2-B finding 4 were DISCHARGED in T2-B (verdict
section 6). T2-A finding 6 (a misleading test name in
`cycle_groups_test.rs`) is byte-locked here -- report only. T2-A finding 7
(`tasks/m235-t2a-handoff.md:122` carries the doc typo `[c0, 0]` instead of the
plan's `[c0, c0]`; production reads `[c0, c0]` correctly at
`cycle_groups.rs:26`) is an artifact text fix **outside your write grant** --
report it, do not fix it. T2-B finding 3 (the fixture corpus is triplicated) is
recorded in the milestone ledger; a shared test-support module before any fourth
copy is a closeout decision, not this unit's.

---

## 8. Acceptance - gates, exact commands, expected verdict lines

Every gate in DIRECT form. **Never write a gate as
`python tools/distill.py -- <argv>`** unless `<argv>` is a `cargo build`; the
distiller is an argv pass-through, so that form runs whatever `<argv>` names.

**Never write a bare-word test filter** -- `testrun.py ... -- <word>` passes
`<word>` to cargo as a TEST-NAME filter and selected zero tests in Phase 1,
returning `0 passed / 0 failed / 0 ignored / 7 suites` with exit 0 (Phase 1
verdict finding 2). Use `--test <target>`.

**10 gate invocations** (plus the preflight `checkpoint_guard.py status` of
step 0.1 and the step 0.3 grep).

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_equivalence_test
3. python tools/testrun.py --distill --timeout 1200 -p ariadne-graph
4. python tools/distill.py -- cargo build -p ariadne-graph
5. python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m3_include_graph_test
6. python tools/distill.py -- cargo build -p ariadne-cli -p ariadne-ui
7. python tools/hygiene_check.py
8. python tools/encoding_guard.py --staged
9. python tools/checkpoint_guard.py status
10. python tools/commit_scoped.py --scope 'crates/graph/src/graph.rs' --scope 'crates/graph/src/closure_equivalence_tests.rs' --scope 'crates/graph/src/cycle_groups.rs' --scope 'tasks/m235-t3d-journal.md' -m "refactor(graph): classify cycle groups against policy and remove detect_cycles"
```

**Expected verdict lines:**

| Gate | Expected verbatim line |
|---|---|
| 1 | `9 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `4 passed / 0 failed / 0 ignored / 1 suites` |
| 3 | `118 passed / 0 failed / 0 ignored / 5 suites` |
| 4 | `ok (exit 0, 0 warnings emitted)` |
| 5 | `3 passed / 0 failed / 0 ignored / 1 suites` |
| 6 | `ok (exit 0, 0 warnings emitted)` |
| 7 | zero R1/R2 lines; R3 warns, if any, are legacy files only |
| 8 | clean, no output |

**The arithmetic behind gate 3, reviewer-measured and re-derived:** Phase 2's
reviewer measured `-p ariadne-graph` = `116 passed / 0 failed / 0 ignored /
5 suites` at `b950963`, split as lib 101 + `cycle_groups_test` 9 +
`cycle_groups_equivalence_test` 4 + `m22_marker_unit_shaper_test` 2 +
doc-tests 0. This unit adds exactly **2** `#[test]` functions to the lib target
(R1 and R2) -> lib 103, total **118**, suites unchanged at 5. Gates 1 and 2 are
the reviewer-measured per-target counts, carried forward unchanged because both
files are byte-locked by this unit.

**Gate 5 is the IV-9 acceptance gate, and it is why this unit is not
`crate:graph`-shaped in its gates alone.** `m3_include_graph_test.rs:118-121`
asserts `report.cycle_count > 0` and must pass **WITHOUT EDIT**; `include/a.h`
and `include/b.h` form one nontrivial SCC, so the group count is 1 and the
assertion holds. **"Passes without edit" is the proof.** The count is 3
(`#[test]` at `:14`, `:79`, `:146`). Unit C already ran this gate before the
report change; **only a run AFTER the report change discharges IV-9**, which is
why it lives here and not only there.

**Gate 6 is the delete's blast-radius proof.** `detect_cycles` was `pub`, so
cargo cannot warn you about a missed external caller -- only a build failure
can. `cargo build -p ariadne-cli -p ariadne-ui` compiles both dependent crates.
It does NOT compile the `ariadne-integration-tests` targets, so gate 5 (which
does build and run `ariadne-graph` through the pipeline) is the compile proof
for that consumer. If you want a belt-and-braces build of the whole consumer
chain, the reviewer runs it in the acceptance pass -- do not add an eleventh
gate.

**Effect on the canonical workspace line:** `+2`. The reviewer's acceptance pass
should read `1177 passed / 0 failed / 4 ignored / 88 suites` (`1169` + 5 from unit
A + 1 from unit B + 0 from unit C + 2 here), assuming unit E adds none. You do
not run the workspace-wide gate and must not claim that number as measured.

---

## 9. Frozen locks - each with a proving command

| Lock | Proving command | Expected |
|---|---|---|
| **plan sec3 item 1: no `detect_cycles` / `dfs_cycle` left in production** | reviewer's workspace grep | zero production hits; only the frozen test oracle + prose |
| **IV-9: `m3_include_graph_test` unedited and green** | gate 5; reviewer's `git diff --name-only` | `3/0/0/1`; the path absent from every commit |
| **IV-10: `closure_equivalence` suite green with `:515` repointed** | gate 3 | green |
| **IV-6: the frozen oracle untouched** | reviewer's diff against the pre-T2-A blob region `:234-287`; reviewer's `git diff --name-only` on `cycle_groups_equivalence_test.rs` | absent from every commit |
| T2-A's 9 `cycle_groups_test` tests unchanged | gates 1 and 3 | `9/0/0/1`; file byte-unchanged |
| **IV-11: the six policy tests' assertion VALUES unchanged** | reviewer's `git diff` on `graph.rs:612-680` | messages/comments may change; no number moves |
| **IV-11: `informational_cycle_count` name + skip-when-zero serde shape** | reviewer's `git diff` on `crates/graph/src/types.rs` (must be absent) | `types.rs` absent from every commit |
| IV-8 planner byte-identity | reviewer's `git diff --name-only`; no `crates/cluster` path | absent |
| `crates/report/**` byte-unchanged | reviewer's `git diff --name-only` | absent |
| `tests/golden/**`, `tests/fixtures/**` byte-unchanged | reviewer's `git diff --name-only`; no `UPDATE_GOLDENS` run | absent |
| `adjacency()` single-source-of-truth | reviewer's grep + gate 4 | no second adjacency loop in `cycle_groups.rs` |
| no cross-crate dependency | `crates/graph/Cargo.toml` byte-unchanged | zero new deps |
| R1/R2 hygiene | gate 7 | no R1/R2; `graph.rs` ~679 lines |
| UTF-8 mojibake | gate 8 | clean; authored content pure ASCII |
| touch-set | reviewer's per-commit `git diff --name-only` | exactly the four in-set paths |
| the unpinned fixture path | reviewer | in no commit (ledger F12) |

Workspace-wide gates are NOT yours: `python tools/testrun.py --baseline` (BROKEN,
ledger F9: `usage error: missing value for --baseline`),
`python -m unittest discover -s tools -p "*_tests.py"`,
`python tools/ui_smoke.py`, `python tools/bench_compare.py` (both REFUSED, ledger
F10 -- there is still NO perf evidence for `detect_cycle_groups`; unit B's bench
re-pin is explicitly labelled UNVERIFIED for that reason). Do not work around any
of them, and never route a refused command through `distill.py` to evade the
denial.

---

## 10. Reviewer gate reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
sec2. The reviewer will audit:

1. **The delete is atomic with its last repoint.** Per-commit
   `git diff --name-only` must show `graph.rs`, `closure_equivalence_tests.rs`
   and `cycle_groups.rs` all landing in the SAME commit that removes
   `detect_cycles`. A commit that deletes the method while `:515` still calls it
   is a blocking build break even if a later commit fixes it.
2. **The Reds were real.** The journal must carry the verbatim `left: 2,
   right: 1` from R1 and the verbatim R2 failure showing
   `informational_cycle_count == 1` against the old path rule, plus the
   hand-trace of the oracle on `double_reporting_scc()` that shows why the old
   code reports two paths. A Red that was never observed is a finding.
3. **R2's semantics.** The reviewer will re-derive the old rule's answer on the
   fixture. If your fixture does not produce "one blocking path and one
   all-tolerating path", the test does not discriminate and the port is unproven.
4. **The six policy tests kept their numbers.** Any moved assertion value in
   `graph.rs:612-680` is a finding.
5. **F4** -- the `adjacency()` doc names every production call site and still
   states the no-duplicate-loop prohibition. **F5** -- either the three-arm match
   is in place, or the journal carries a deferral with all three required
   elements; gate 4's zero-warning result is the load-bearing evidence either way.
6. **`graph.rs` line count** near 679, and no file near 1000.
7. **IV-9 re-proven after the report change** (gate 5), with the file unedited.
8. **Deviation adjudication** on evidence, not on declaration.

**Report back (final message):** commit SHAs (also in your journal); the verbatim
Red failures for R1 and R2 with the oracle hand-trace; Green evidence quoting the
canonical lines and capture paths for gates 1, 2, 3, 4 and 5; the `graph.rs`
before/after line counts; the F5 disposition (fix or deferral, with its three
elements); the step 0.3 grep result that authorised the delete; every deviation
with its reason; out-of-scope observations, report-only. Never paste raw cargo or
git output -- cite the command, the canonical line, the exit code, and the
capture path.

---

## 11. Discipline clauses

- **Command execution.** `bash` is an allowlist restricted to the distilling
  entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED.
- **Reading a refusal.** A refused command ALWAYS reports the catch-all deny
  rule, whatever the real cause. Treat it as boilerplate. Attempt each gate
  INDEPENDENTLY; never drop a gate because its first invocation was refused. If a
  gate cannot run, record the VERBATIM refusal in your journal rather than
  silently narrowing the acceptance set. **Never narrow gate 5** -- it is the
  IV-9 proof.
- **Strict TDD.** Write R1 -> observe -> write R2 -> observe -> implement the
  port -> observe Green -> port the six messages -> delete -> Refactor. Never
  weaken an assertion to make it pass. Never commit a Red state, and never commit
  a commit in which the method is deleted while a caller still references it.
- **Code hygiene.** Functions < 100 lines, files < 1000 lines, no
  `tmp`/`res`/`foo`/`bar`/`test1`, no single-letter locals in code you author, no
  source line >= 2000 characters.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>`, or
  `Get-Content | -replace` on a tracked file. Kilo Read/Edit/Write tools, or
  byte-level `open(path, "wb")`. Pure-ASCII authored content. Quote whole git
  arguments: `git show "rev^:path"`.
- **Scope.** Touch only sec2. Out-of-scope breakage: report it, do not fix it.
  In particular do not touch `crates/cluster/src/scc.rs`, do not create a shared
  test-support module, and do not add a `.gitattributes` pin.
- **Journal.** Append a 1-3 line checkpoint after EACH Green gate. Record your
  own SHAs in it.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A
  dead session is never resumed -- a successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3d-handoff.md --scope ... --cmd ...`.

---

## 12. Sizing check (handoff law, stated explicitly)

| Metric | Bound | This unit |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 top-level module directory | `crates/graph` only -- **1 crate** |
| Gate commands | ~10 | **10** (plus preflight step 0.1 and the step 0.3 grep, which are not gates) |
| Produced lines (implementation + test diff) | ~1200 | **~250** (`graph.rs` -55 deleted + ~+120 seam/repoints/new tests/doc fix, `closure_equivalence_tests.rs` 1 line, `cycle_groups.rs` ~+6/-4, journal ~90) |

**Single handoff unit -- fits on all three axes.**

**The seam, and why the split lands here.** Plan T3.1 as written spans the report
layer, the policy seam, the CLI label, the delete, and five-plus external call
sites across `crates/graph`, `crates/cli`, `crates/report` (whose only role is
forwarding `cycle_count`, which does not change) and `tests/integration` -- four
top-level scopes, far past the one-crate law. It is therefore split into units A
through E, and **this unit is where the delete lands**, because:

- units B (`crates/cli`) and C (`tests/integration`) migrate every caller outside
  `crate:graph` to `detect_cycle_groups()` first, each landing green;
- that leaves exactly three `detect_cycles` callers, all inside `crates/graph`;
- so the delete becomes a **one-crate** change that is atomic with the repoint of
  its own last caller.

A seam that left the method deleted while a foreign caller still referenced it
would put a broken build between two commits. This seam never does: every one of
the five unit boundaries compiles, tests green, and the workspace line moves only
by the tests each unit adds (5, 1, 0, 2, 0).

---

## 13. Authorization and ordering

This handoff, `tasks/m235-kickoff.md`, the M23.5 milestone row in
`tasks/milestone_ledger.md`, and APPROVED `tasks/m235-t3b-handoff.md` **and**
`tasks/m235-t3c-handoff.md` are the sole authorization for this unit.

Runs fourth. Phase 4 (T4.1 integration closeout) remains UNAUTHORIZED -- do not
begin it, do not run the workspace-wide gates, and do not try to close ledger rows
F9, F10, F11 or F12; those are operator actions at closeout. Do not touch
`crates/ui/**`, `crates/cli/**`, `tests/**`, or `tools/**`.