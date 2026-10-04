# M23.5 Phase 3 T3-D Reviewer Verdict (crate: graph)

Reviewer session started 2026-10-04. Skeleton written first (two prior reviewer
sessions died to upstream idle timeouts before writing anything). Sections A-G
are recorded in place, cheapest-first, replacing PENDING as work proceeded.
Gates were run only after A-G were all recorded.

Contract: `tasks/m235-t3d-handoff.md`. Journal: `tasks/m235-t3d-journal.md`
(deviations Dev-1..Dev-5). Predecessor: `tasks/m235-t3c-verdict.md`. Prior
findings routed in: `tasks/m235-t2a-verdict.md` findings 4 and 5.
Plan: `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec1.2
(IV-2/9/10/11), sec1.3, sec3 items 1, 2, 6.

## 0. Ground truth (reviewer-run, before any judgement)

`python tools/checkpoint_guard.py status`:

| Signal | Value |
|---|---|
| HEAD | `aebfa9b` |
| branch | `tarjan-s-scc` |
| DIRTY FILES (`git diff HEAD --numstat`) | **`(none)`** |
| UNTRACKED | `tasks/m235-t3d-verdict.md` (this reviewer's own artifact) |
| RECENT COMMITS | `aebfa9b`, `ed5c467`, `3e1efa6`, `acea7f2`, `adb53a3` |
| census, this worktree | `ahead=24 behind=13 dirty=2` |

The `dirty=2` is the known F12 porcelain-only stat-cache entry on
`tests/fixtures/basic/artifacts/includes/report.json` plus my own untracked
verdict file. Adjudicated in section H below.

---

## SECTION A - D1-B DELETION COMPLETE (acceptance criterion 1)

**RESULT: PASS. `detect_cycles` and `dfs_cycle` are gone as live code
everywhere in the workspace, and `detect_cycles_in_units` is provably a
different function.**

### A.1 My own search, not the journal's

Search 1 - definitions and call shapes, whole `crates/` tree, pattern
`fn detect_cycles|fn dfs_cycle|\.detect_cycles\(|Self::dfs_cycle|::dfs_cycle\(`:

```
crates/graph/tests/cycle_groups_equivalence_test.rs:65   // `Self::dfs_cycle` -> `old_dfs_cycle`   <- DOC COMMENT
crates/transform/src/engine/mod.rs:133                    fn detect_cycles_in_units(...)            <- different fn
```

**Two hits. Zero definitions of `detect_cycles`. Zero definitions of
`dfs_cycle`. Zero call sites of either.**

Search 2 - bare `detect_cycles` across all of `crates/`: 15 hits, exhaustively
classified by me:

| Site | Class | Live code? |
|---|---|---|
| `crates/graph/src/graph.rs` | **ZERO hits - file is clean** | n/a |
| `crates/graph/src/cycle_groups.rs:64` | doc comment contrasting the D-A1 seed universe with `detect_cycles`'s old seed | prose |
| `crates/graph/src/closure_equivalence_tests.rs:67` | fixture **label string** `"graph.rs 2-cycle fixture (detect_cycles)"` | string literal |
| `crates/graph/tests/cycle_groups_test.rs:5/:101/:142` | doc/Red-evidence comments | prose |
| `crates/graph/tests/cycle_groups_equivalence_test.rs` (11 hits) | the **frozen test-local oracle** `old_detect_cycles`/`old_dfs_cycle` + its docs + the label string | IV-6 frozen oracle |
| `crates/transform/src/engine/mod.rs:72/:133` | `detect_cycles_in_units` | **different function** |
| `crates/cli/tests/payload_tail_bench.rs:62/:64` | stale prose (`detect_cycles` "still exists until") | prose, outside touch-set |

Search 3 - `detect_cycles|dfs_cycle` under the workspace-root `tests/`: **no
files found**.

### A.2 Nothing resembling them was conflated

`detect_cycles_in_units` (`crates/transform/src/engine/mod.rs:133-164`) read in
full: it takes `&[GeneratedUnit]`, builds a `BTreeMap<PathBuf, BTreeSet<PathBuf>>`
from `required_deps`, delegates to a **nested free fn `has_cycle`**, and returns
a **`bool`**. Different type domain (`GeneratedUnit` / `PathBuf`, not
`IncludeGraph` / `FileId`), different return type, different crate, different
module. It is not a rename, alias, or surviving relative of the deleted
`IncludeGraph::detect_cycles`. The T3-B handoff's line 71 warning was honoured
correctly. **No conflation.**

### A.3 The frozen oracle cannot be a hidden caller

`old_detect_cycles` (`cycle_groups_equivalence_test.rs:90-114`) takes
`graph: &IncludeGraph` but touches **no method on it**: it reads
`graph.file_paths` (`:96`) and calls its own `old_adjacency(graph)` (`:98`),
which builds its own mirrored map. `old_dfs_cycle` (`:116-143`) is recursive
over that local map. Neither calls the deleted method. The oracle survives
intact (IV-6), as the handoff required.

### A.4 Hidden-caller probe (feature gates / ignored bench targets)

The strongest form of this probe is structural: `detect_cycles` is **not a symbol
in the tree**, so no configuration can reach it. I additionally checked the one
plausible carrier - `crates/cli/tests/payload_tail_bench.rs`, a test target
outside `crates/graph`:

- `#[ignore]`: **zero occurrences** (grep for `#\[ignore\]|cfg\(feature)` in
  that file returns nothing; only two bare `#[test]` at `:442` and `:569`).
- Its timed call already reads `graph.detect_cycle_groups()` at `:322`.
- No `cfg(feature` gate on any cycle API anywhere in `crates/graph`.

**No hidden caller. Not blocking.**

### A.5 The second clause of criterion 1 - `CycleGroup` as the only representation

`CycleGroup` is now the only cycle **detector**. One path-based
cycle **classification** view survives by design and by another unit's ownership:

- `crates/ui/src/dto.rs:604-622` - `build_include_elements_legacy_paths(...,
  cycles: &[Vec<FileId>], ...)`, documented in-source as "Legacy path-based
  entry point (pre-M23.5)".
- `crates/ui/src/cycle_groups_view.rs:3-9` - module doc naming both entry
  points and stating the legacy one is "retired in T3-E".

`tasks/m235-t3e-handoff.md` **exists** and owns exactly that deletion. So
criterion 1's second clause completes at T3-E, not here; T3-D's `crates/graph`
scope could not and did not touch it. **Recorded as a carry-forward to T3-E, not
a finding against this unit.**

### A.6 The atomicity requirement (handoff sec10 audit 1) - satisfied

`git show --stat acea7f2` (the delete commit) carries **all three files in one
commit**:

```
 crates/graph/src/closure_equivalence_tests.rs |  2 +-
 crates/graph/src/cycle_groups.rs              |  5 +-
 crates/graph/src/graph.rs                     | 70 +++-------------------
 3 files changed, 13 insertions(+), 64 deletions(-)
```

There is **no** commit in the range in which `detect_cycles` is absent while any
caller still references it. Half One (`adb53a3`) contains no deletion at all;
Half Two (`acea7f2`) deletes and repoints `:534` and `closure_equivalence_tests.rs:515`
together. **Atomic.**

### A.7 The preflight claim, corroborated independently

The implementer reported preflight live refs at `graph.rs:234/:246/:260/:275/:356/:534`
plus `closure_equivalence_tests.rs:515`. That is **independently confirmed by the
predecessor**, not merely by me: `tasks/m235-t3c-verdict.md:769-772` measured
`pub fn detect_cycles` at `graph.rs:234`, `fn dfs_cycle` at `:260`, and the three
call sites at `:356`, `:534`, `closure_equivalence_tests.rs:515` at its HEAD
`6c0c2b6`. The preflight that authorised the delete was truthful.

---

## SECTION B - THE IV-11 CLASSIFICATION RULE (audited from the code)

**RESULT: PASS on all five probes.**

### B.1 The rule as implemented

`crates/graph/src/graph.rs:246-257`:

```rust
/// A group is informational iff **every** member file's policy is
/// non-`Reject` (M20 D11 ported to partition semantics, IV-11); a
/// mixed-member group is blocking.
fn group_is_informational(
    group: &CycleGroup,
    policy_of: &dyn Fn(FileId) -> CyclePolicy,
) -> bool {
    group
        .member_files
        .iter()
        .all(|&file_id| !matches!(policy_of(file_id), CyclePolicy::Reject))
}
```

Matches plan sec1.2 IV-11 and handoff sec3.1 verbatim, including the mandated
name `group_is_informational` (T2-A deviation E's deferral is now discharged -
the fn is live, called at `graph.rs:315`) and the mandated inherent-associated-fn
shape (it takes no `self` and is called as `Self::group_is_informational`, not
converted to a free fn). `Tolerate` and `TolerateWithinPackage` both satisfy
"non-`Reject`", matching the M20 D11 original.

### B.2 Probe (1) - EMPTY-GROUP VACUITY: **impossible, proven structurally**

`Iterator::all` on an empty iterator returns `true`, so an empty group WOULD
classify informational. The question is whether an empty group can exist. It
cannot, and the proof is two independent barriers in the producer:

1. **`cycle_groups.rs:292-303`** - `group_is_cyclic` matches
   `[] => false`. `filter_cyclic_groups` (`:282-290`) drops any group whose
   member list is empty. This is an **explicit, named rejection of the empty
   case**, not an accident of the algorithm.
2. **`cycle_groups.rs:236-254`** - `emit_group_if_root` cannot produce an empty
   group even in principle: the `loop` pushes `popped` (including the root
   `vertex` itself) and only breaks after pushing `vertex`, so `group.len() >= 1`
   always holds.

A third, independent barrier: `order_reverse_topological` indexes
`groups[index][0]` (`:323`, `:332`) as the heap key and would **panic** on an
empty group, so the empty case is structurally excluded before that point too;
and it short-circuits `group_count == 0` to `Vec::new()` at `:316-318`.

`CycleGroup` has exactly one construction site (`cycle_groups.rs:52-57`), fed
only from `ordered_groups`. **Therefore `group_is_informational` never evaluates
`all()` over an empty iterator. The vacuity has no carrier.**

### B.3 Probe (2) - MIXED group: BLOCKING, and pinned by an assertion

`all()` short-circuits on the first `Reject`, so one rejecting member among
tolerating members yields `false` -> blocking. **Pinned directly, not merely
implied**, by two independent tests:

- `graph.rs:602-628` `double_reporting_scc_with_one_rejecting_member_is_blocking`
  (R2): fixture group `{1,2,3}`, delegate `FileId(1) -> Reject`, everything else
  `-> Tolerate`. Asserts `report.informational_cycle_count == 0` with a message
  naming the exact discriminating case, **and** asserts the serialized JSON does
  not contain `"informational_cycle_count"`. This is the milestone's semantic
  centre and it is asserted head-on.
- `graph.rs:651-663` `cycle_with_one_rejecting_member_is_not_informational`:
  the 2-cycle fixture, `FileId(1) -> Reject`, rest `-> TolerateWithinPackage`,
  asserting `cycle_count == 1` **and** `informational_cycle_count == 0`.

**The fixture genuinely discriminates.** I re-derived the old rule's answer from
the frozen oracle rather than accepting the journal's trace. The oracle
(`cycle_groups_equivalence_test.rs:116-143`) seeds from sorted `file_paths`,
pushes 1 -> 2 -> 3, and on `3`'s sorted neighbours `[1, 2]` finds `1` on
`current_path` at position 0 -> reports `[1,2,3]`, then `2` at position 1 ->
reports `[2,3]`. Under the old per-path rule that is
`informational([1,2,3]) == false` (member 1 rejects) but
`informational([2,3]) == true` (both tolerate), so the old report said
`informational_cycle_count == 1`. The journal's R2 Red quote (`left: 1`,
`right: 0`) is exactly what that derivation predicts. **The test would not
discriminate if the fixture had produced one path; it produces two.**

**A vacuity mode I found and closed:** R2 in isolation is susceptible to passing
for the wrong reason - if `detect_cycle_groups()` returned **zero** groups on this
fixture, then `informational_cycle_count == 0` and the JSON-key-absence
assertion would BOTH hold vacuously. **R1 closes this**: it uses the identical
`double_reporting_scc()` fixture and asserts `report.cycle_count == 1`
(`graph.rs:594-599`), so zero groups would fail R1. R1 and R2 are adjacent,
both in the lib target, both green in gate 3. The pair is non-vacuous.

### B.4 Probe (3) - no assertion silently re-expressed into a weaker condition

Decisive evidence: `git diff -U0 6c0c2b6..aebfa9b -- crates/graph/src/graph.rs`
returns **exactly twelve hunks** for the whole range. Every one accounted for:

| # | Hunk | Truth-condition effect |
|---|---|---|
| 1 | `+use crate::cycle_groups::CycleGroup;` | none |
| 2 | `adjacency()` doc expansion (F4) | comment only |
| 3 | **DELETE** `detect_cycles` + `dfs_cycle` (56 lines) | the mandated change |
| 4 | `cycle_is_informational` -> `group_is_informational` | the mandated change |
| 5 | report doc rewrite | comment only |
| 6 | `let detected_cycles = self.detect_cycles();` -> `detected_groups = self.detect_cycle_groups()` | the mandated change |
| 7 | `.filter(\|cycle\| Self::cycle_is_informational(cycle, ..))` -> `\|group\| Self::group_is_informational(group, ..)` | **same predicate, renamed binding** |
| 8 | `cycle_count: detected_cycles.len()` -> `detected_groups.len()` | the mandated IV-9 change |
| 9 | `let cycles = graph.detect_cycles();` -> `let cycle_groups = graph.detect_cycle_groups();` | rename only |
| 10 | `!cycles.is_empty(),` -> `!cycle_groups.is_empty(),` | **same predicate, renamed binding** |
| 11 | **+71 lines**: `double_reporting_scc()`, R1, R2, suite header comment | new coverage |
| 12 | test 1's one-line `assert_eq!` rewrapped, message reworded | **values identical** |

Only two predicates changed shape (hunks 7, 10) and both are the *same* predicate
on a renamed value. Not one assertion anywhere in the range was made easier to
satisfy. No fixture weakened, no gate loosened, no operator changed.

### B.5 Probe (4) - the six ported `informational_cycle_count` tests

Proven by the same zero-context diff: **exactly one line inside the six-test
range changed** (hunk 12), and it is test 1's assertion **message**:

```
-        assert_eq!(report.cycle_count, 1, "IV-2: cycle_count keeps counting all cycles");
+        assert_eq!(
+            report.cycle_count, 1,
+            "IV-9: cycle_count keeps counting every entangled group"
+        );
```

The compared values (`report.cycle_count`, `1`) are byte-identical; only the
string literal and the wrapping changed. Tests 2-6 **do not appear in the diff
at all** - they are untouched bytes. Verified each stance in the tree at HEAD
(`graph.rs:639-710`):

| # | Test | Line | Stance | Verdict |
|---|---|---|---|---|
| 1 | `cycle_with_all_members_tolerating_is_informational` | `:639` | `cycle_count == 1`, `informational == 1` | **matches** reported `1/1`; values unmoved |
| 2 | `cycle_with_one_rejecting_member_is_not_informational` | `:651` | `cycle_count == 1`, `informational == 0` | **matches** reported `1/0`; untouched |
| 3 | `plain_tolerate_also_counts_as_informational` | `:665` | `cycle_count == 1`, `informational == 1` | **matches** reported `1/1`; untouched |
| 4 | `default_report_never_classifies_cycles_as_informational` | `:674` | `cycle_count == 1`, `informational == 0` | **matches** reported `1/0`; untouched |
| 5 | `informational_cycle_count_key_is_skipped_from_json_when_zero` | `:681` | key absent when 0; literal `"informational_cycle_count":1` present when non-zero | **matches**; byte-for-byte lock preserved |
| 6 | `all_reject_policy_json_is_byte_identical_to_default_report` | `:699` | `assert_eq!(default_json, all_reject_json)` | **matches**; byte-for-byte lock preserved |

**The reword did not weaken the assertion it describes.** `assert_eq!`'s truth
condition is `left == right` on the two compared expressions; both are
character-identical across the change. The new message is also **more accurate
than the old one**: it says "every entangled group" (true under the ported
semantics) where the old said "all cycles" (now false), and it relabels the
invariant from IV-2 (cycle-iff-nontrivial) to IV-9 (the `cycle_count` name
carrying group count), which is the invariant the assertion now actually
witnesses. This is the opposite of a reword-for-cover.

### B.6 Probe (5) - IV-9 naming, and `types.rs` untouched

- **`cycle_count` carries GROUP count while keeping its NAME.** `graph.rs:325`
  reads `cycle_count: detected_groups.len(),` inside the `IncludeReport` literal.
  Field name unchanged; `usize` unchanged; type unchanged.
- **`types.rs` genuinely untouched.** `git diff --name-only 6c0c2b6..aebfa9b --
  crates/graph/src/types.rs` is **empty**, and the per-commit stat lists contain
  no `types.rs` path. Read at HEAD to confirm the lock survives rather than
  inferring it: `types.rs:56` `pub cycle_count: usize`;
  `types.rs:61-62` `#[serde(default, skip_serializing_if = "informational_cycles_absent")] pub informational_cycle_count: usize`;
  `types.rs:69-71` `fn informational_cycles_absent(count: &usize) -> bool { *count == 0 }`.
  **Skip-when-zero serde shape is byte-intact**, so test 5 and test 6's locks
  remain meaningful.
- **`generate_report()` stays the all-`Reject` wrapper**: `graph.rs:242-244`
  `self.generate_report_with_cycle_policies(&|_file_id| CyclePolicy::Reject)`.
  Untouched by the diff, which is why every C++ artifact stays byte-stable.
- **Callback signature frozen**: `generate_report_with_cycle_policies(&self,
  policy_of: &dyn Fn(FileId) -> CyclePolicy)` at `graph.rs:266-269` - not in the
  diff.

### B.7 A stale doc comment I found in a byte-locked file

`crates/graph/src/types.rs:57-60` still documents
`informational_cycle_count` as "**Cycles** whose every member file tolerates",
and `:66-68` as "no **cycle** is informational". After this unit the counted unit
is the **group**. `types.rs` is byte-locked out of this unit's touch-set
(handoff sec2), so the implementer was right not to edit it.
**Report-only, owner = whoever next touches `types.rs`.**

---

## SECTION C - IV-9 FALSIFIABILITY

**RESULT: PASS. The predecessor's green-for-the-wrong-reason defect is closed,
and the untouched test is still byte-identical.**

### C.1 The defect the predecessor identified, and its closure

`tasks/m235-t3c-verdict.md` finding 5 recorded that at the prior HEAD
`m3_include_graph_test` was green **for the wrong reason**, because
`graph.rs:369` still read `cycle_count: detected_cycles.len()` - so the
assertion witnessed a path count, not a group count.

At HEAD, `graph.rs:325` reads:

```rust
cycle_count: detected_groups.len(),
```

fed by `let detected_groups = self.detect_cycle_groups();` at `graph.rs:312`.
The line moved from `:369` to `:325` purely because 44 lines of recursion were
deleted above it (hunks 3, 4, 5 of the zero-context diff). **The semantic
half of IV-9 is now live.** (The journal cites the pre-change number `:369`;
the post-change line is `:325`. A stale citation in the journal, not a code
defect.)

### C.2 The test is byte-identical - blob-level proof

`git diff b950963..aebfa9b -- tests/integration/tests/m3_include_graph_test.rs`
-> **empty output**. Independently corroborated at blob level:

| Revision | `git rev-parse <rev>:tests/integration/tests/m3_include_graph_test.rs` |
|---|---|
| `b950963` (pre-Phase-3 tip) | `b6a54d5881dcf3d82ae8fcf56885fabd7e892a23` |
| `aebfa9b` (HEAD) | `b6a54d5881dcf3d82ae8fcf56885fabd7e892a23` |

**Identical blob SHA-1, and zero differing bytes - not one.** The T3-C reviewer
measured the same blob SHA at its HEAD `6c0c2b6`, so the file has been
byte-stable from before Phase 3, through unit C, through unit D. Three
independent methods (empty path diff, identical blob hash, absence from the
range `--name-only` list and from every per-commit stat) agree. **If one byte
differed this would be blocking; none does.**

### C.3 The falsification I was asked to construct

**Would `m3_include_graph_test` actually fail if the classification reverted to
paths?** No - and that is *correct and required*, not a weakness. The assertion
is `report.cycle_count > 0` over the fixture `include/a.h` includes `"b.h"` and
`include/b.h` includes `"a.h"`. Under path semantics that is one DFS path;
under group semantics it is one SCC, one group. **1 > 0 and 1 > 0.** The
fixture cannot distinguish the two representations, which is exactly why it is
the right *contract* witness and the wrong *semantic* witness - and exactly why
the semantic witness had to be added inside `graph.rs` as R1/R2.

The falsification the plan actually asks for is IV-9's **count** half, and R1
supplies it on a fixture that *can* distinguish: `double_reporting_scc()` yields
two old paths and one group, so a reversion to path semantics at
`graph.rs:325` turns R1 red with `left: 2, right: 1`. **The milestone now has a
falsifiable semantic lock, and the milestone-level field-name lock is still the
byte-untouched `m3` test.** The two halves are covered by the right instruments.

### C.4 The one re-pin the predecessor asked for

T3-C finding 5 also asked this unit to re-pin the four absolute
`assert_eq!(report.cycle_count, 1, ...)` at `graph.rs:617/:631/:640/:647`. **Done
- and correctly so:** they are now `graph.rs:645/:661/:670/:677` (shifted by the
deletion). All four still assert `== 1`, and all four are on `mutual_cycle_graph()`
where one path equals one group, so they needed no value change - only their
wording. Confirmed against the zero-context diff: only one of the four (test 1)
changed at all.

---

## SECTION D - IV-10 AT `closure_equivalence_tests.rs:515`

**RESULT: PASS. One line changed. Nothing was loosened, dropped, or swallowed.**

### D.1 The complete diff for that file across the range

`git diff 6c0c2b6..aebfa9b -- crates/graph/src/closure_equivalence_tests.rs`
returns **one hunk, one changed line**:

```diff
@@ -512,7 +512,7 @@ fn cyclic_chain(
     fn test_random_digraphs_with_cycles_match_bfs_oracle() {
         for &(seed, file_count, topology, name) in RANDOM_FIXTURES.iter() {
             let graph = random_graph(seed, file_count, topology);
-            let cyclic = !graph.detect_cycles().is_empty();
+            let cyclic = !graph.detect_cycle_groups().is_empty();
```

Checked explicitly, all four loosening mechanisms: **no `||` introduced; no
`unwrap_or(true)` introduced; no bound loosened; no assertion dropped.** The
file is 535 lines at HEAD and the diff is 1 line.

### D.2 The rest of the function, read at HEAD - intact

`closure_equivalence_tests.rs:511-535`:

- `Topology::PureDag => assert!(!cyclic, ...)` - operator and message unchanged.
- `_ => assert!(cyclic, ...)` - operator and message unchanged.
- `assert_closures_match_oracle(&graph, name);` (`:526`) - the suite's **actual**
  equivalence proof, untouched.
- `assert_eq!(graph.compute_closures(), from_module, ...)` (`:529-533`) - the
  module-entry parity lock, untouched.

**A suite that keeps passing while proving less would be blocking. This suite
proves exactly what it proved before**: the repointed `cyclic` binding is a
**corpus regime-sanity check** ("this corpus contains both regimes"), not the
closure-equivalence claim. The claim is `assert_closures_match_oracle`, and that
line is not in the diff.

### D.3 The named divergence was NOT swallowed - and why structurally

Phase 2 deliberately carried **one** named old-vs-new divergence for the
*unregistered* endpoint universe, in
`crates/graph/tests/cycle_groups_equivalence_test.rs`.

- **That file is byte-unchanged across this range**: `git diff --name-only
  6c0c2b6..aebfa9b -- crates/graph/tests/cycle_groups_equivalence_test.rs` is
  **empty**. The divergence therefore *cannot* have been swallowed.
- It is still **named in eight places** in that file, including the module doc
  `:10-11`, the fixture `named_divergence_graph()` at `:469`, the corpus push
  `"named-divergence"` at `:505`, and the dedicated test
  `test_unregistered_endpoint_cycle_is_found_by_groups_but_not_by_old_paths` at
  `:791` - which asserts the old `file_paths`-only enumeration finds nothing
  (`:802`), that exactly one cyclic group is found (`:810`), that the D-A1 union
  seeding locates the unregistered 401 <-> 402 cycle (`:815-816`), that new is a
  **strict superset** of old there (`:824`), and that the dangling unregistered
  node 403 is in **no** group (`:830-831`).

### D.4 The equality argument, verified from the tree rather than accepted

The repoint is legal only if `!detect_cycles().is_empty()` and
`!detect_cycle_groups().is_empty()` have the same truth value on this corpus.
I checked the corpus generator at `closure_equivalence_tests.rs:418-427`:
`random_graph` registers **every** file `1..=file_count` (`:420-424`), and the
`link` closure only ever connects `files[from]` to `files[to]` - both inside the
registered set. **So the entire corpus at `:515` lies inside the
registered-endpoint universe where Phase 2's IV-2 equality is proven.** The one
divergence requires an unregistered cycle endpoint, which this corpus
structurally cannot contain.

**Conclusion: the repoint is a same-predicate re-expression on this corpus, the
named divergence is untouched in a byte-locked file, and the suite's proof is
untouched. IV-10 holds.**

---

## SECTION E - LOWLINK AND DETERMINISM SURVIVED

**RESULT: PASS. Push / advance / pop all correct; no cross-crate dependency.**

### E.1 Frame pop order: emit-if-root FIRST, then propagate

`cycle_groups.rs:227-231`:

```rust
fn retire_frame(&mut self) {
    let frame = self.frames.pop().expect("retire_frame runs with a live frame");
    self.emit_group_if_root(frame.vertex);   // EMIT FIRST
    self.finish_child(frame.vertex);         // THEN PROPAGATE
}
```

Correct, and the ordering is **load-bearing**: emitting after the propagate would
let a child's lowlink contaminate the root test `lowlink[v] == index[v]`
(`:237`) and split one SCC into several. The in-source justification at `:222-226`
names the recursive-body correspondence (the callee emits before the caller
applies the min after the call).

### E.2 The propagate itself is the required min

`finish_child` (`cycle_groups.rs:259-274`): if no parent frame remains (the seed's
own frame retired) it returns; otherwise it takes the new top frame's vertex as
the parent and computes `*parent_lowlink = min(*parent_lowlink, child_lowlink)`.
**That is `min(lowlink[parent], lowlink[child])`, exactly as required.**

### E.3 Push and advance

- **PUSH** (`enter_vertex`, `:158-168`): `indices[v] = lowlink[v] = next_index`,
  increment, push onto `tarjan_stack`, mark `on_stack`, open a frame at neighbour
  0. All five effects of the recursive head are present.
- **ADVANCE** (`advance_frame`, `:174-186`): exhaustion -> `NeighborsExhausted`;
  unindexed neighbour -> `enter_vertex` + `Descended`, with **tree-edge
  propagation explicitly DEFERRED to pop** (documented at `:171-172`); indexed
  neighbour still `on_stack` -> `relax_lowlink_toward_index` =
  `lowlink[v] = min(lowlink[v], indices[w])` (`:210-220`).

All four lowlink update points (push, tree-edge-on-pop, back-edge-on-advance,
parent propagation) are individually present and in the classic iterative-Tarjan
order. **No dual representation retained**: `cycle_groups.rs` holds no path
enumeration, no `rec_stack`, no `current_path`.

### E.4 Determinism (IV-5) - no hash-iteration order reaches the output

- Seeds: `tarjan_seed_universe` sorts then dedups (`:75-76`).
- Adjacency: `graph.rs:95-98` sorts then dedups destinations.
- Members: `emit_group_if_root` sorts ascending (`:252`).
- Group order: `order_reverse_topological` is Kahn's algorithm on the
  condensation DAG, emitting **sinks first** (= reverse-topological), with a
  `BinaryHeap<Reverse<(FileId, usize)>>` keyed on `groups[index][0]` - the
  group's **smallest member**, since members are sorted. Because groups are
  pairwise disjoint (IV-1) those keys are **unique**, so the tie-break is a
  **total** order, not merely deterministic-by-accident.
- Two `HashMap`/`HashSet` iteration sites exist - `condensation_edges:359` and
  `external_degree_pairs:392` - and **neither can perturb the output**:
  out-degrees are counts of distinct successors (commutative), and the
  `predecessors` vectors are consumed only to decrement counters and push onto a
  total-order heap. `external_degree_pairs` accumulates into fixed indices.
- `representative_cycle_for` seeds at `members[0]` and walks sorted adjacency;
  `closed_walk_witness` uses its `HashSet`s for membership only, never iteration.

**Determinism holds.**

### E.5 No cross-crate dependency introduced

- `git diff --name-only 6c0c2b6..aebfa9b -- crates/graph/Cargo.toml` -> **empty**.
  `Cargo.toml` is byte-unchanged; zero new deps. The range-level `--name-only`
  list contains no `Cargo.toml` and no `Cargo.lock` path.
- `grep "cluster"` across all of `crates/graph` returns **4 hits, all prose**:
  `cycle_groups.rs:6` (module doc naming `crates/cluster/src/scc.rs` as the
  shape being mirrored, explicitly "without any cross-crate dependency
  (D-A7)"), `cycle_groups.rs:83` (section comment), `cycle_groups.rs:222` (doc
  comment citing `cluster scc.rs:190-198`), and
  `tests/cycle_groups_test.rs:20` (doc comment citing a cluster test constant).
- **Zero `use ariadne_cluster`, zero `ariadne_cluster::`, zero `cluster::` code
  references in `crates/graph`.** The ~60-line frame-machine duplication is the
  pre-accepted, documented posture (D-A7) and it is intact.

### E.6 The single adjacency source, no second loop

Complete `.adjacency()` call-site census in `crates/graph` (7 sites + the
definition, exactly the handoff F4 table):

| Site | Kind |
|---|---|
| `graph.rs:88` | definition |
| `graph.rs:127` (`compute_closures`) | production |
| `cycle_groups.rs:42` (`detect_cycle_groups`) | production |
| `load_caches.rs:339` (LoadAnalyzer memo) | production |
| `graph.rs:148` (`compute_closures_bfs_oracle`) | `#[cfg(test)]` |
| `closure_equivalence_tests.rs:231`, `:528` | `#[cfg(test)]` |

`detect_cycle_groups` still calls the single source (`cycle_groups.rs:42`);
no adjacency-build loop was added. `old_adjacency` in the frozen oracle is the
IV-6 frozen test-local copy, pre-existing and required.

---

## SECTION F - CARRIED FINDINGS GENUINELY RESOLVED

**RESULT: PASS on both, by resolution and not by restatement.**

### F.1 T2-A finding 4 - the understated `adjacency()` doc comment: RESOLVED

Read at HEAD, `graph.rs:79-87`:

> Builds a `source -> sorted, deduplicated destinations` map ... **The production
> adjacency-build call sites are `compute_closures` (here),
> `detect_cycle_groups` (`cycle_groups.rs`) and `LoadAnalyzer`'s
> `file_adjacency` memo (`load_caches.rs`) - no other code may duplicate this
> loop. The `#[cfg(test)]` callers (`compute_closures_bfs_oracle` here and
> `closure_equivalence_tests.rs`) are separate, deliberate consumers of this same
> contract, not a licence to add a loop.**

Every clause checked against my own census in E.6: **all three** post-delete
production sites are named (previously 2 of 5 were named, and the named pair
omitted `detect_cycle_groups` entirely); **both** test families are named;
**the prohibition is kept verbatim** ("no other code may duplicate this loop")
and is now reinforced for the test side ("not a licence to add a loop"). The old
em-dash was replaced with an ASCII hyphen, consistent with the pure-ASCII rule.

**This is a resolution, not a restatement**: the previous text was *false*
(understated the call sites); the new text is *true* and I verified it against
the tree.

### F.2 T2-A finding 5 - `FrameStep` boolean-in-disguise: RESOLVED (preferred form)

The exact four-line three-arm match the handoff prescribed is in place
(`cycle_groups.rs:147-152`):

```rust
while !self.frames.is_empty() {
    match self.advance_frame(adjacency) {
        FrameStep::Descended | FrameStep::EdgeVisited => continue,
        FrameStep::NeighborsExhausted => self.retire_frame(),
    }
}
```

All three variants are now load-bearing at the enum's one call site. No deferral
was needed, so the handoff's three-element deferral requirement is moot.

**Behaviour is provably identical.** The old form was
`if let FrameStep::NeighborsExhausted = step { self.retire_frame(); }`.
`FrameStep` has exactly three unit variants and carries no data, so:

- `NeighborsExhausted` -> `retire_frame()` in both forms. Identical.
- `Descended` / `EdgeVisited` -> the old form fell through the `if let` with an
  empty body and reached the end of the `while` body, i.e. the next iteration;
  the new form's `continue` targets the same `while`. **Identical.**
- No arm is missing (`FrameStep` is exhaustively matched, no `_`), so no new
  panic path and no silently swallowed variant.

The whole diff for the file is this hunk: `+3/-2`, nothing else. Combined with
gate 4's zero-warning result, the dead-code concern that motivated the finding is
closed as well.

---

## SECTION G - LINE COUNT WENT THE WRONG WAY

**RESULT: verified, and adjudicated ACCEPTABLE - non-blocking, attributed to the
planner's sizing arithmetic rather than to the implementer.**

### G.1 The count, verified two ways

- `Read` on `crates/graph/src/graph.rs` at HEAD: **total 734 lines**.
- `git diff --numstat 6c0c2b6..aebfa9b`: `105` added / `75` removed for
  `graph.rs`, net **+30**. Base **704** (independently measured at `b950963` by
  the T2-C verdict, the T2-B verdict, and handoff sec3.5). 704 + 30 = **734**.

**The implementer's reported 704 -> 734 is exact.**

### G.2 The decomposition - why it grew

The zero-context diff accounts for every line:

| Contribution | Lines |
|---|---|
| DELETE `detect_cycles` + `dfs_cycle` (production recursion) | **-56** |
| RENAME + rewrap `group_is_informational` (net) | -3 / +5 |
| `+use crate::cycle_groups::CycleGroup;` | +1 |
| `adjacency()` doc expansion (F4) | +4 |
| report doc rewrite | +2 / -4 |
| repointed bindings + `cycle_count` | -2 / +2 |
| **R1 + R2 + `double_reporting_scc()` + fixture doc + suite header** | **+71** |
| test 1's `assert_eq!` rewrap | +3 / -1 |
| **net** | **+30** |

**The growth is 71 lines of mandated ported unit tests and ~7 lines of mandated
doc work, against 56 lines of deleted production recursion.** Handoff sec4 put
both Reds inside `graph.rs`'s existing `mod tests` and sec5 mandated *exactly*
two `#[test]` functions plus one fixture helper there. The implementer built
exactly what was specified and no more - the spec count is respected precisely.

### G.3 Judgement: acceptable, and the estimate is what is wrong

The ceiling holds with **real** margin, not token margin:

| File | Lines | Headroom to 1000 |
|---|---|---|
| `crates/graph/src/graph.rs` | **734** | **266** |
| `crates/graph/src/cycle_groups.rs` | **528** | 472 |

Both far under the R1 ceiling. All authored functions are well under 100 lines
(the largest, `generate_report_with_cycle_policies`, spans `graph.rs:266-329` =
64 lines including its doc-free body). No source line approaches 2000 characters.

**The file grew during a de-recursion milestone, and that is acceptable here,
for a specific reason: what grew is test surface, not production surface.** The
milestone's production goal in this file was met unambiguously - 56 lines of
recursive DFS gone, one `CycleGroup`-based classifier in their place,
`mod tests` now ~46% of the file. The plan's own expectation (the brief cites a
drop to ~649) and handoff sec3.5's arithmetic (704 - 55 + ~30 = ~679) are both
simply wrong: the `~+30` addition estimate undercounts the two mandated Reds and
their fixture, which alone are 71 lines. **Attribution: planner / handoff-spec,
not implementer.** The file is not near the ceiling and no hygiene rule is
breached; the defect is a sizing estimate that would mislead unit E's sizing if
left uncorrected.

One real observation for T3-E's benefit: `mod tests` occupying ~336 of 734 lines
means the next unit that adds to `graph.rs`'s tests should consider whether the
test module belongs in `crates/graph/tests/` instead. Not this unit's problem -
the handoff sec4 explicitly required the Reds in the in-file `mod tests`.

---

## SECTION H - COMMIT-vs-TOUCH-SET, PER-COMMIT, FIXTURE STATE

Range `6c0c2b6..aebfa9b` (base = T3-C tip per the journal and the T3-C verdict's
own range header).

### H.1 Range-level name list

`git diff --name-only 6c0c2b6..aebfa9b` - six paths:

```
crates/graph/src/closure_equivalence_tests.rs
crates/graph/src/cycle_groups.rs
crates/graph/src/graph.rs
tasks/m235-t3c-execution-results.md
tasks/m235-t3c-verdict.md
tasks/m235-t3d-journal.md
```

`git diff --numstat 6c0c2b6..aebfa9b`:

| Added | Removed | Path | In touch-set? |
|---|---|---|---|
| 1 | 1 | `crates/graph/src/closure_equivalence_tests.rs` | **yes** (sec2) |
| 3 | 2 | `crates/graph/src/cycle_groups.rs` | **yes** (sec2) |
| 105 | 75 | `crates/graph/src/graph.rs` | **yes** (sec2) |
| 240 | 0 | `tasks/m235-t3c-execution-results.md` | declared records |
| 1142 | 0 | `tasks/m235-t3c-verdict.md` | declared records |
| 283 | 0 | `tasks/m235-t3d-journal.md` | **yes** (sec2) |

### H.2 Negative touch-set checks, reviewer-measured

| Check | Result |
|---|---|
| `-- tests` (whole directory, incl. `tests/golden` and `tests/fixtures`) | **empty** |
| `-- Cargo.toml` | **empty** |
| `-- Cargo.lock` | **empty** |
| `.gitattributes` | absent from the range name list and every per-commit stat (a dotfile-path-scoped `git diff` was refused by the allowlist with the catch-all text; the range list is dispositive) |
| `crates/lang-scala/**` | absent from the range list |
| `crates/report/**` | absent from the range list |
| `crates/cluster/**` | absent from the range list (IV-8 byte-identity holds) |
| `crates/graph/src/types.rs` | **empty** (explicit check) |
| `crates/graph/src/lib.rs`, `closures.rs`, `load*.rs`, `crates/graph/Cargo.toml` | **empty** (explicit check) |
| `crates/graph/tests/cycle_groups_test.rs`, `cycle_groups_equivalence_test.rs` | **empty** (explicit check) |
| `tools/**`, `.kilo/plans/**` | absent from the range list |

So absent from **every** commit: the frozen oracle, `types.rs`, all `Cargo.toml`
files, `Cargo.lock`, `crates/cluster/**` (IV-8), `crates/report/**`,
`crates/lang-scala/**`, `tests/golden/**`, `tests/fixtures/**`, `tools/**`,
`.gitattributes`, `.kilo/plans/**`. **The touch-set is enforced AND not
over-broad** - both directions clean, which is the property that matters.

### H.3 Per-commit, oldest first, each judged alone

| # | SHA | Subject | Paths (+/-) | In touch-set? |
|---|---|---|---|---|
| 1 | `adb53a3` | `refactor(graph): count and classify entangled groups in the report seam (IV-9, IV-11)` | `graph.rs` only, +96/-14 | **yes** (sec2) |
| 2 | `acea7f2` | `refactor(graph)!: delete detect_cycles/dfs_cycle, repoint last callers (D1-B, IV-7, IV-10); FrameStep three-arm match (F5)` | `closure_equivalence_tests.rs` +1/-1, `cycle_groups.rs` +3/-2, `graph.rs` +9/-61 - **all three in ONE commit** | **yes** (sec2) |
| 3 | `3e1efa6` | `docs(tasks): land M23.5 T3-C reviewer records (verdict, execution results)` | the two T3-C records, +1382/-0 | declared records |
| 4 | `ed5c467` | `docs(tasks): M23.5 T3-D journal -- SHAs ..., Red verbatims, abort-grep, F4/F5, deviations` | `tasks/m235-t3d-journal.md`, +277/-0 | **yes** (sec2) |
| 5 | `aebfa9b` | `docs(tasks): M23.5 T3-D journal -- definitive SHA record (...)` | `tasks/m235-t3d-journal.md`, +9/-3 | **yes** (sec2) |

Commit 1 is the seam port with **no deletion**; commit 2 is the atomic delete
with all three repoints and both carried findings. **No commit leaves the build
broken.** Commits 3 and 4 are pure additions (0 deletions) and therefore cannot
have modified any existing byte. Commit 5 is a 12-line self-reference repair to
this unit's own journal (the SHA-record paradox T3-C adjudicated as Dev-2).

### H.4 The forbidden F12 fixture

`tests/fixtures/basic/artifacts/includes/report.json` appears in **NO** commit:
`-- tests/fixtures` is empty across the range, `-- tests` is empty across the
range, and **none of the five per-commit stat lists contains any `tests/` path at
all**. Two independent methods, one conclusion.

### H.5 Fixture state and the F12 mechanism

Reviewer-measured `python tools/checkpoint_guard.py status` at session start:
DIRTY FILES **`(none)`**, UNTRACKED only my own verdict file, census `dirty=2`.
**That is the known F12 signature** - unpinned path plus `core.autocrlf=true`,
where a run invalidates the stat-cache entry while `git diff` normalises line
endings and finds no delta. The implementer's distinction ("numstat clean at
preflight, so the Read/Write restore was never needed and none was attempted")
is literally true of my own independent measurement, and **correctly no fix was
attempted** - `git checkout --` / `git restore` are mutating git and denied to
every role. **Adjudicated: known F12, not a new problem.** Standing closeout
obligation (pin the path or make the writer emit LF) unchanged and still open.
**No CRLF bytes entered the repository through this unit.**

---

## GATES - COMPLETE

### 1. Scoped gates, reviewer-run, nothing relayed

| Gate | Command | Reviewer-measured canonical line | Matches pin? |
|---|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_test` | `9 passed / 0 failed / 0 ignored / 1 suites` | **yes** |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_equivalence_test` | `4 passed / 0 failed / 0 ignored / 1 suites` | **yes** |
| 3 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph` | `118 passed / 0 failed / 0 ignored / 5 suites` | **yes** |
| 4 | `python tools/distill.py -- cargo build -p ariadne-graph` | `ok (exit 0, 0 warnings emitted)` | **yes** |
| 5 (IV-9) | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m3_include_graph_test` | `3 passed / 0 failed / 0 ignored / 1 suites` | **yes** |
| 6 | `python tools/distill.py -- cargo build -p ariadne-cli -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` | **yes** |
| 7 | `python tools/hygiene_check.py` | **zero R1/R2 lines workspace-wide** | **yes** |
| 8 | `python tools/encoding_guard.py --staged` and `--all` | **no output, exit 0** each | **yes** |
| 9 | `python tools/checkpoint_guard.py status` | see section H.5 and 3.4 below | **yes** |

**All six pinned lines reproduced verbatim. Gate 3's arithmetic is confirmed from
the capture: 103 (lib) + 4 + 9 + 2 + 0 (doc-tests) = 118, five suites, zero
unattributed residual.** Lib 103 = 101 at the base + the 2 mandated Reds.

### 2. Non-vacuity of the scoped gates, from the captures

Every scoped capture reads **`0 filtered out`** - the specific vacuity mode
handoff sec8 warns about - and every one names its load-bearing carrier PASSING:

| Gate | capture evidence |
|---|---|
| 3 (149-line capture) | **`graph::tests::cycle_count_counts_entangled_groups_not_reported_paths ... ok`** (`:14`, R1); **`graph::tests::double_reporting_scc_with_one_rejecting_member_is_blocking ... ok`** (`:18`, R2); all six IV-11 tests named ok (`:15`, `:16`, `:19`, `:20`, `:23`, `:28`) - including **both serde byte-stability locks**; **`graph::closure_equivalence::test_random_digraphs_with_cycles_match_bfs_oracle ... ok`** (`:87`, the IV-10 witness with the repointed `:515` predicate) plus all 8 other `closure_equivalence` tests; `test_cycle_detection ... ok` (`:22`); `test_acyclic_and_empty_graphs_yield_no_groups ... ok` (`:122`); `test_deep_include_chain_completes_on_default_stack ... ok` (`:128`, 18.81s, the IV-7 stress); the frozen oracle's **`test_unregistered_endpoint_cycle_is_found_by_groups_but_not_by_old_paths ... ok`** (`:111`, the named divergence) |
| 5 (10-line capture) | `running 3 tests`; **`test_m3_edge_cases_duplicates_system_missing_cycles ... ok`** (`:5`) - the exact carrier of `report.cycle_count > 0` at `m3_include_graph_test.rs:118-121`; `0 filtered out`. **This is the first run in the milestone where that assertion was witnessed over a group count.** |
| 1, 2 | 16-line and 11-line captures, `9 passed` / `4 passed`, `0 filtered out` |

**These gates are load-bearing, not confirmatory.** Gates 3 and 5 positively
witness that the *new* group-based report path executes and returns a non-zero
group count on two independent fixtures built by two independent lanes; R1/R2
positively witness that the ported classifier classifies a mixed group as
blocking where the old per-path rule did not.

### 3. Workspace-wide acceptance pass

| Gate | Command | Result |
|---|---|---|
| baseline | `python tools/testrun.py --baseline` | **BROKEN.** Verbatim: `usage error: missing value for --baseline`. Ledger F9, reproduced. Not treated as a gate failure, not worked around, not wrapped. **The invocation it wraps WAS run** (3.1). |
| full workspace | `python tools/testrun.py --distill --timeout 3600` | **COMPLETED** (3.1) |
| tool unit suites | `python -m unittest discover -s tools -p "*_tests.py"` | **COMPLETED** (3.2) |
| hygiene | `python tools/hygiene_check.py` | **zero R1/R2** (3.3) |
| encoding, all tracked | `python tools/encoding_guard.py --all` | **no output, exit 0** |
| encoding, staged | `python tools/encoding_guard.py --staged` | **no output, exit 0** |
| fixture state | `python tools/checkpoint_guard.py status` | session start, and again after the workspace run (3.4) |
| UI runtime smoke | `python tools/ui_smoke.py` | **REFUSED.** Verbatim: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` |
| load-perf regression | `python tools/bench_compare.py` | **REFUSED.** Verbatim: identical text. |

Each known-broken / known-refused item was attempted **INDEPENDENTLY, exactly
once, directly**. **None was retried, and none was routed through `distill.py`
or any other wrapper to evade a denial.** On the two refusals: the real cause is
that neither script is on the reviewer bash allowlist (F10), **not** that bash is
unavailable - the same session ran `hygiene_check.py`, `encoding_guard.py` (both
forms), `checkpoint_guard.py status`, `unittest discover`, six scoped
`testrun.py` invocations, a full-workspace `testrun.py`, and two `distill.py`
builds. **The denials caused no gate to be skipped.**

### 3.1 The canonical workspace line - COMPLETED

`python tools/testrun.py --distill --timeout 3600`, exit 0, capture
`target/testrun-capture.txt` (2078 lines). Verbatim:

```
1178 passed / 0 failed / 4 ignored / 88 suites
```

Against the baseline the brief named:

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| baseline (`6c0c2b6`, reviewer-measured by T3-C) | 1176 | 0 | 4 | 88 |
| **reviewer-measured (`aebfa9b`)** | **1178** | **0** | **4** | **88** |
| **Delta** | **+2** | **+0** | **+0** | **+0** |

**The delta is exactly +2 passed and +0 in the other three columns** - the two
mandated Reds, and nothing else. **Zero unattributed residual.**

Note: handoff sec8 predicted `1177`. **The handoff's prediction is off by one** -
its baseline chain used `+1` for unit B, but T3-B actually added **2** (its own
reviewer measured `+2` and attributed the gap to D1's test). The implementer's own
stated expectation of `1178` is the correct one. Recorded as finding 2.

Non-vacuity, reviewer-measured: grep of the capture for
`test result: FAILED|^failures:|panicked at|error\[E` returns **no matches**,
matching `0 failed`. Suite count unchanged at **88**, as predicted. The capture
independently confirms the milestone-level locks that matter here:

- `golden_baseline_basic ... ok` and `golden_baseline_expensive ... ok` -
  **no golden drift.** The goldens' `"cycle_count": 0` still agrees
  byte-for-byte with the freshly-computed report under the new group semantics,
  which is IV-11's "all-`Reject` C++ artifact byte-stability" and the handoff's
  "goldens are a byte no-op" claim discharged end-to-end.
- **`fixture_cycle_is_informational_under_tolerate_policy ... ok`** (m20) and
  **`auto_shaped_graph_keeps_the_cycle_informational ... ok`** (m22) - **both
  Scala consumers green against the ported report layer.** This is the direct
  downstream proof that the IV-11 group port did not break the IV-12
  plugin-side-free consumers; their `informational_cycle_count` assertions now
  mean groups and still hold.
- `planner_characterization`, `planner_facts_parity`, `m00_facts_path_parity_test`
  and `m23_symbol_isolation_test` targets all compiled and ran green - **IV-8
  planner byte-identity holds unedited**.

The two `warning:` lines in the capture (`:1963` 20 warnings in
`m00_facts_path_parity_test`, `:1986` 1 in `m23_symbol_isolation_test`) are
**pre-existing integration-test-target warnings in files this unit did not
touch**; gates 4 and 6 both reported `0 warnings emitted` for every crate this
unit did touch.

### 3.2 Tool unit suites - COMPLETED

`python -m unittest discover -s tools -p "*_tests.py"`:

```
Ran 184 tests in 18.164s

OK
```

Identical count to the T3-B and T3-C reviewers' acceptance passes (`Ran 184
tests`, `OK`) - correct, since this unit changes no `tools/**` file.

### 3.3 Hygiene - zero R1/R2, and the R3 claim independently checked

Every printed line is `R3 (warn)`, which is warn-only per `AGENTS.md`, so none
is blocking. The implementer claimed "zero warns in `graph.rs` and
`cycle_groups.rs`; the `closure_equivalence_tests.rs` warns are untouched
fixture-builder lines, not the one line this unit edited." **That claim is
literally true, and I checked it rather than relaying it:**

- `crates/graph/src/graph.rs` - **zero lines** in the hygiene output.
- `crates/graph/src/cycle_groups.rs` - **zero lines**.
- `crates/graph/src/closure_equivalence_tests.rs` - 10 warns, at `:179`, `:250`,
  `:430`, `:431`, `:439`, `:453`, `:461`, `:474`, `:475`, `:501`. **None of
  them is `:515`**, the single line this unit edited; all are in the
  fixture-builder region (`random_graph`, `cyclic_chain`), untouched bytes.

The zero is a **measurement, not a skip**: the scanner demonstrably reaches all
three files this unit touched, and it flags `crates/transform/src/engine/mod.rs:135`
- a file this unit did not touch - in the same run.

**Zero R1/R2 also disposes of the 1000-line ceiling:** `graph.rs` at 734 and
`cycle_groups.rs` at 528 both pass with real margin.

### 3.4 Fixture state after the workspace run - sixth F12 reproduction

`python tools/checkpoint_guard.py status`, after the heaviest possible
invocation (full workspace, 2078-line capture):

| Signal | at session start | after my workspace run |
|---|---|---|
| HEAD | `aebfa9b` | `aebfa9b` |
| DIRTY FILES (`git diff HEAD --numstat`) | **`(none)`** | **`(none)`** |
| UNTRACKED | `tasks/m235-t3d-verdict.md` (mine) | `tasks/m235-t3d-verdict.md` (mine) |
| census `dirty=` | `2` | `2` |

**The forbidden fixture is still absent from the numstat dirty section after my
full-workspace run.** Same mechanism as the five prior reproductions (T2-A
finding 9, T2-B finding 10, T3-A, T3-B, T3-C): unpinned path plus
`core.autocrlf=true`, so a run that rewrites the artifact invalidates the
stat-cache entry while `git diff` normalises line endings and finds no delta.
**Adjudicated: known F12, not a new problem, and correctly no fix attempted**
(`git checkout --` / `git restore` are mutating git and denied to every role).

### 3.5 What was NOT run, stated plainly

- `python tools/testrun.py --baseline` - **could not run**, broken
  (`usage error: missing value for --baseline`, F9). Attempted once, recorded
  verbatim, not worked around. **The invocation it wraps was run** (3.1).
- `python tools/ui_smoke.py` - **could not run**, REFUSED (F10). Attempted once,
  verbatim recorded, not retried, not wrapped. **No runtime UI evidence exists
  for this milestone at this HEAD.**
- `python tools/bench_compare.py` - **could not run**, REFUSED (F10). Attempted
  once, verbatim recorded. **No verified perf evidence exists for this
  milestone.**
- **The implementer's two Red captures** (`101 passed / 2 failed / 0 ignored /
  1 suites`, and the two panic verbatims) are not independently recoverable -
  `target/` is ephemeral and a Red state was correctly never committed. I
  re-derived the Red's arithmetic from the frozen oracle's source by hand
  (section B.3) and it predicts `left: 2 / right: 1` for R1 and
  `left: 1 / right: 0` for R2, matching the journal's quoted failures exactly.
  So the Reds are corroborated structurally rather than by capture; **the
  temporal ordering of their observation is taken on report**, the same posture
  the T3-C reviewer recorded for its gate-5 BEFORE capture.
- **The "orchestrator-declared" status of `3e1efa6`'s two out-of-touch-set
  records** - I have no dispatch message in this session declaring them, and the
  board carries no such declaration. I take it on report, corroborated by the
  exact T3-C Dev-1 precedent (`d11a819` did the same for the T3-B records, and
  that adjudication was ACCEPTED). The commit is **pure additions (+1382/-0)**,
  so it cannot have modified any existing byte, and the mechanism is the only one
  available (reviewers cannot commit; write git is denied).
- **No commit was made by this reviewer** (write git is denied to this role).

---

## DEVIATIONS Dev-1..Dev-5, adjudicated independently

Adjudicated on the spec and the code, not on the implementer's justification.

| # | Deviation | Disposition | Attributed to |
|---|---|---|---|
| Dev-1 | `graph.rs` landed 734 lines vs the handoff's ~679 estimate | **ACCEPTED** | **planner / handoff-spec** (the estimate is wrong, not the build) |
| Dev-2 | `group_is_informational` signature wrapped across lines | **ACCEPTED** - and required | n/a (implementer judgement, contract-conformant) |
| Dev-3 | R2's second assertion (JSON key absence) not directly observed failing | **ACCEPTED** | n/a (disclosure honesty; substitute evidence sufficient) |
| Dev-4 | `encoding_guard.py --staged` necessarily runs on an empty index | **ACCEPTED** | configuration (`commit_scoped.py` has no stage-only mode) |
| Dev-5 | gate count: 10 invocations run as 5 per half | **ACCEPTED** | n/a (nothing skipped) |

**Dev-1 - ACCEPTED; the arithmetic is the defect, not the code.** I verified the
count two ways (section G.1): `Read` reports 734 lines at HEAD, and
`git diff --numstat` reports +105/-75 = net +30 over a base of 704. The
implementer's reported 704 -> 734 is exact. The decomposition in G.2 shows the
growth is **71 lines of mandated ported tests plus ~7 of mandated doc work
against 56 lines of deleted production recursion** - i.e. the handoff's `~+30`
addition estimate undercounts by more than 40 lines, because sec4/sec5 mandated
two `#[test]` functions plus a fixture helper inside the in-file `mod tests`. The
implementer built exactly what was specified: **exactly two** new `#[test]`
functions and **exactly one** new fixture helper, as sec5 requires. Attributed to
the planner (see finding 1). 266 lines of real headroom; no hygiene rule breached.

**Dev-2 - ACCEPTED, and effectively REQUIRED.** The handoff's one-line spelling
(`fn group_is_informational(group: &CycleGroup, policy_of: &dyn Fn(FileId) -> CyclePolicy) -> bool {`)
is 105 characters and would exceed the repo's 100-column rustfmt width. The body
is `group.member_files.iter().all(...)`, exactly as sec3.1 specifies, and the
mandated inherent-associated-fn shape is preserved (it is called as
`Self::group_is_informational` at `graph.rs:315`, not converted to a free fn).
Nothing was dropped.

**Dev-3 - ACCEPTED.** The first assertion's panic necessarily stops the test
before the JSON check, so the second assertion could not be *observed* failing in
the same Red run - that is a property of `assert_eq!`, not a shortcut. The
implementer disclosed it rather than claiming a Red it did not see, which is the
correct behaviour. **The substitute evidence is sufficient and I verified it
from the code, not from the journal:** (a) I re-derived the old per-path rule's
answer on the fixture from the frozen oracle's source and it gives
`informational_cycle_count == 1`, so under the old rule the JSON **does** carry
the key (non-zero serializes) and assertion 2 fails by the same mechanism; (b)
the *Green* side of assertion 2 is directly witnessed - gate 3's capture names
`double_reporting_scc_with_one_rejecting_member_is_blocking ... ok`, and the
surviving ported test `informational_cycle_count_key_is_skipped_from_json_when_zero`
at `graph.rs:681-697` independently pins the skip-when-zero shape on the same
fixture family. So the property is asserted in the committed tree and green.

**Dev-4 - ACCEPTED** - same disposition as T2-A gate 5 / T2-B D-E / T3-A D5 /
T3-B D5 / T3-C Dev-3. `commit_scoped.py` stages and commits atomically and raw
git staging is denied, so `--staged` cannot interpose. The implementer ran the
**strictly stronger** `encoding_guard.py --all` before committing. **I re-ran
both independently: `--all` silent exit 0, `--staged` silent exit 0.**
Substitution cost is zero. Attributed to configuration.

**Dev-5 - ACCEPTED.** The handoff's sec8 lists 10 gate invocations; the unit
committed in two halves (per the orchestrator's reviewability instruction) and
ran the scoped tests once per half. **Nothing was skipped** - every one of the 10
gates ran and every pinned line is reproduced verbatim in section 1 above, plus
my own workspace acceptance pass.

### Undeclared deviations found by this reviewer

**None.** Every divergence from upstream text that surfaced traces to a handoff,
plan or ledger sentence and is recorded as a finding (1, 2, 3). The two-commit
split, the declared records, and the encoding-gate ordering were all disclosed
before I looked.

---

## FINDINGS

**0 blocking / 13 non-blocking.**

1. **NON-BLOCKING, planner / handoff-spec.** `graph.rs` sizing arithmetic is
   wrong in two places. Handoff sec3.5 states "704 - 55 (deleted) + ~+30 = **~679
   lines**"; the plan's framing implies a drop to roughly 649. The measured
   result is **734**. The `~+30` addition estimate undercounts by more than 40
   lines because it does not price the two mandated `#[test]` functions, their
   fixture helper, or the fixture doc - together 71 lines. **Nothing is wrong
   with the delivered file** (266 lines of headroom, zero R1/R2, every function
   well under 100 lines); the defect is the estimate. **Action:** make
   `handoff-spec` derive addition estimates from the mandated test bodies rather
   than guessing, and correct sec3.5 before T3-E's sizing is computed from it.
2. **NON-BLOCKING, planner.** Handoff sec8's predicted workspace line
   `1177 passed / 0 failed / 4 ignored / 88 suites` is **off by one**. Its
   baseline chain used `+1` for unit B, but T3-B added **2** (its own reviewer
   measured `+2` and attributed the gap to D1's test). Measured at this HEAD:
   **`1178 / 0 / 4 / 88`**. The implementer's own stated expectation (1178) was
   correct; the handoff's was not. **Action:** fix the chain in sec8.
3. **NON-BLOCKING, planner.** The plan's citation of the six IV-11 tests as
   `graph.rs ~:614-687` (plan sec2 T3.1, repeated in handoff sec4) is **stale by
   roughly seven lines**: the real pre-change range was `:612-680`, and at HEAD
   the six tests span `:639-710`. The implementer recorded the drift as handoff
   sec4 instructs, which is correct, but the upstream citation should be fixed.
   Same class of defect Phase 2 found in its own handoff artifacts.
4. **NON-BLOCKING, report-only, `types.rs` owner.** `crates/graph/src/types.rs:57-60`
   still documents `informational_cycle_count` as "**Cycles** whose every member
   file tolerates", and `:66-68` as "no **cycle** is informational". After this
   unit the counted unit is the **group**. `types.rs` is byte-locked out of this
   unit's touch-set (handoff sec2), so the implementer was **right not to edit
   it** - correctly reported, not fixed. **Owner:** the next unit touching
   `types.rs` (T3-E or closeout).
5. **NON-BLOCKING, implementer (journal precision, no code impact).** The
   journal's Checkpoint 2 states "`graph.rs:369` now reads
   `cycle_count: detected_groups.len()`". `369` is the **pre-change** line
   number; at HEAD the line is **`:325`** (it moved up 44 lines because the
   recursion above it was deleted). The journal's other line pins are correct.
6. **NON-BLOCKING, report-only, `crates/cli` owner.**
   `crates/cli/tests/payload_tail_bench.rs:62/:64` prose is now stale - it says
   `detect_cycles` "still exists until" a later unit. It was deleted by
   `acea7f2`. Correctly reported by the implementer; the file is outside this
   unit's touch-set and was correctly left alone.
7. **NON-BLOCKING, report-only, `tools/**` owner.**
   `tools/bench_compare.py:52`, `tools/bench_baseline.json:10` and
   `tools/bench_compare_tests.py:36` still label the metric
   `generate_report+detect_cycles (combined)`. A bench row label, out of scope;
   F10 stands.
8. **NON-BLOCKING, planner / closeout (carry-forward).** Acceptance criterion
   1's **second** clause - "`CycleGroup` is the workspace's only cycle
   representation" - is **not yet complete**, and correctly so:
   `crates/ui/src/dto.rs:604-622` still exposes the legacy path-based entry
   point and `crates/ui/src/cycle_groups_view.rs:3-9` documents it in-source as
   "retired in T3-E". `tasks/m235-t3e-handoff.md` **exists** and owns exactly
   that deletion. Criterion 1's first clause (`detect_cycles` no longer exists)
   **is** discharged by this unit. **Action:** carry criterion 1 forward to T3-E;
   do not close the milestone on it before T3-E lands.
9. **NON-BLOCKING, configuration (standing, F10) - and MORE MATERIAL for this
   unit than for any earlier one.** `python tools/ui_smoke.py` and
   `python tools/bench_compare.py` each attempted once, directly, each refused
   with identical verbatim catch-all text (3.3); neither routed through
   `distill.py`. **Why this unit escalates it:** `ui_smoke.py` is the only gate
   in the repo that cold-boots the UI, asserts its routes, and asserts
   `Cache-Control: no-store` on the payload. This unit changed what
   `IncludeReport::cycle_count` *means* - it is now a group count - and that
   number is exactly what the UI's include view and
   `crates/cli/src/commands/includes.rs`'s "Cycle Groups Detected" label
   display. Before this unit the UI's group-derived classification and the
   report's path-count number were **inconsistent by construction** (two
   representations, never compared); after it they agree by construction, which
   is a genuine improvement. But **no runtime run has ever verified that the UI
   boots and serves a report whose `cycle_count` is a group count**, and the
   legacy path builder that T3-E will delete has also never been exercised at
   runtime against the post-T3-D report. Likewise this unit changed what
   `generate_report()` computes internally (iterative Tarjan instead of DFS path
   enumeration), so the timing `payload_tail_bench.rs:322` measures has changed
   meaning again - with `bench_compare.py` refused, there is still **no verified
   perf evidence for the milestone**. **Action:** close F10 before T4.1; it is an
   operator/allowlist action, not implementer work.
10. **NON-BLOCKING, configuration (standing, F9).**
    `python tools/testrun.py --baseline` -> `usage error: missing value for
    --baseline`, reproduced verbatim. Attempted once, not retried, not worked
    around, **not treated as a gate failure**. The full-workspace invocation it
    wraps was run and compared by hand (3.1).
11. **NON-BLOCKING, configuration (standing, F12).** Sixth identical
    reproduction: numstat clean, porcelain dirty, census `dirty=2`, path in no
    commit. Correctly no fix attempted. **Action:** pin
    `tests/fixtures/basic/artifacts/includes/report.json` in `.gitattributes` or
    make the artifact writer emit LF. Must land before T4.1.
12. **NON-BLOCKING, orchestrator (dispatch).** `3e1efa6` carries
    `tasks/m235-t3c-verdict.md` (+1142) and `tasks/m235-t3c-execution-results.md`
    (+240), both outside the sec2 touch-set. **Pure additions, zero deletions**,
    so the commit cannot have modified any existing byte; `--name-only` confirms
    no other out-of-set path was staged anywhere in the range. The mechanism is
    the only one available (reviewers cannot commit). Exact precedent: T3-C's
    `d11a819`, adjudicated ACCEPTED as its Dev-1. **A dispatch/declaration
    matter, not an implementer scope violation.** Recorded so the next reviewer
    does not read those two paths in the range as a scope breach.
13. **NON-BLOCKING, planner (carry-forward, standing since T3-B).** The T3-B
    reviewer's "companion parity test expires at unit E" is **still open**: unit
    E deletes the legacy path builder and pins no test additions, and units C
    and D could not host it (C is `tests/integration`-scoped, D is
    `crates/graph`-scoped). **A `crates/ui`-only unit between D and E is still
    required.** Owner: planner. **Action:** schedule it, or fold the parity test
    into T3-E's scope.

### Reviewer-shaped refusals this session (for the next reviewer's benefit)

Recorded so no gate is later mistaken for one that was skipped: (a) a
`python -c "..."` inline command was refused with the catch-all text; (b) a
multi-path `git diff --name-only <range> -- A B C D E F` (six paths) was refused;
(c) any `git diff` scoped to a leading-dot path (`.gitattributes`) was refused.
**In every case the real cause was my own command shape, not an unavailable
gate.** Each was re-issued as a single-path or dedicated-tool query and
succeeded; the range `--name-only` list (six paths) is dispositive for `.gitattributes`
in any case. **No acceptance gate was narrowed as a result.**

---

## VERDICT

# APPROVED

**0 blocking / 13 non-blocking.**

Range `6c0c2b6..aebfa9b`, five commits. All four in-touch-set paths land inside
the declared touch-set; the fifth is the orchestrator's records-only commit
(finding 12). The forbidden F12 fixture is in **no** commit. `types.rs`, all
`Cargo.toml` files, `Cargo.lock`, `crates/cluster/**` (IV-8), `crates/report/**`,
`crates/lang-scala/**`, `tests/golden/**`, `tests/fixtures/**`, `tools/**` and
the two frozen oracle files are absent from all five. **The touch-set is enforced
AND not over-broad** - the check that catches an over-broad declaration is clean
in both directions.

**The seven audit sections, each answered from the code rather than the journal:**

- **A - D1-B deletion: COMPLETE.** My own two searches over `crates/**` find
  **zero** `fn detect_cycles`, **zero** `fn dfs_cycle`, **zero** `.detect_cycles(`
  call sites, and **zero** hits under `tests/`. `graph.rs` itself has zero
  occurrences. The 15 surviving `detect_cycles` mentions are exhaustively
  classified: frozen IV-6 oracle, doc prose, one fixture label string, one stale
  `crates/cli` prose line, and `crates/transform::detect_cycles_in_units` -
  which I read in full and which is **provably a different function** (takes
  `&[GeneratedUnit]`, `PathBuf` adjacency, nested `has_cycle`, returns `bool`).
  **Nothing resembling them was conflated.** No hidden caller: the symbol does
  not exist, and the one plausible carrier
  (`payload_tail_bench.rs`) has no `#[ignore]`, no `cfg(feature)`, and already
  calls `detect_cycle_groups()`. The delete is **atomic** with all three repoints
  in `acea7f2` - no commit in the range leaves the build broken. The
  precondition grep is independently corroborated by the T3-C verdict's own
  measurement at the same HEAD.
- **B - IV-11 classification rule: CORRECT, and audited adversarially.**
  **EMPTY-group vacuity is structurally impossible**, proven by two independent
  barriers in the producer: `cycle_groups.rs:297` is an explicit `[] => false`
  arm in `group_is_cyclic`, and `emit_group_if_root` cannot construct an empty
  group (it always pushes the root before breaking) - with a third barrier at
  `groups[index][0]`, which would panic on an empty group. `all()` therefore
  never runs over an empty iterator. **The MIXED-group case is BLOCKING and
  pinned by direct assertion, not by implication** - R2 at `graph.rs:602-628`
  asserts `informational_cycle_count == 0` *and* JSON key absence on a fixture
  I re-derived from the frozen oracle to produce exactly "one blocking path and
  one all-tolerating path", so the test genuinely discriminates. I also found and
  closed a vacuity mode R2 has **in isolation** (zero groups would satisfy all
  three of its assertions): **R1 uses the identical fixture and asserts
  `cycle_count == 1`**, so the pair is non-vacuous. **No assertion was silently
  re-expressed**: the entire range's `graph.rs` diff is twelve hunks at zero
  context, and only two predicates changed shape - both the *same* predicate on a
  renamed binding. The six ported tests keep every value (`1/1`, `1/0`, `1/1`,
  `1/0`, both serde locks byte-for-byte); **exactly one line inside them changed
  at all**, test 1's assertion *message*, with the compared values
  character-identical - and the reword is **more** accurate than the original,
  not less. `cycle_count` carries GROUP count with its NAME intact
  (`graph.rs:325`), and `types.rs` is absent from every commit, with the
  skip-when-zero predicate read at HEAD to confirm survival rather than inferred.
- **C - IV-9 falsifiability: CLOSED.** `graph.rs:325` now reads
  `cycle_count: detected_groups.len()`. `m3_include_graph_test.rs` is
  **byte-identical** to `b950963` - empty diff **and** identical blob SHA-1
  `b6a54d5...`, **zero differing bytes, not one**. Gate 5 is green with the
  exact `report.cycle_count > 0` carrier named PASSING and `0 filtered out`.
  On the falsification I was asked to construct: **no, that test would not fail
  if the classification reverted to paths** - and that is *correct*, because the
  `a.h <-> b.h` fixture yields one path and one group, so it is the right
  contract witness and the wrong semantic witness. The semantic falsifier is R1,
  which fails `left: 2, right: 1` on a fixture that *does* distinguish. The
  milestone now has both halves, each on the right instrument. The predecessor's
  four requested re-pins are present and correctly needed no value change.
- **D - IV-10 at `:515`: NO LOOSENING.** The file's diff across the range is
  **one hunk, one line**. No `||`, no `unwrap_or(true)`, no loosened bound, no
  dropped assertion. The repointed `cyclic` binding is a corpus **regime** check;
  the suite's actual proof, `assert_closures_match_oracle`, is not in the diff,
  nor is the module-entry parity assert. **The named divergence was not
  swallowed**: its file is byte-unchanged across the range, it is named in eight
  places, and its dedicated test passes in gate 3's capture. And I verified the
  equality argument from the corpus generator itself - `random_graph` registers
  every file `1..=file_count` and links only inside that set, so the corpus lies
  entirely inside the registered-endpoint universe where IV-2 equality is
  proven. **A suite that kept passing while proving less would have been
  blocking; this one proves exactly what it proved before.**
- **E - Lowlink and determinism: INTACT.** Frame pop is **emit-if-root FIRST,
  then propagate** (`retire_frame` `:227-231`) - the ordering is load-bearing and
  correct - and the propagation is exactly
  `min(lowlink[parent], lowlink[child])` (`:259-274`). Push assigns index and
  lowlink together; advance defers tree-edge propagation to pop and relaxes
  `lowlink[v] = min(lowlink[v], index[w])` on back edges. All four update points
  present, in the classic order. No dual representation retained in
  `cycle_groups.rs`. Determinism holds: seeds, adjacency and members are all
  sorted and deduped, and the condensation's heap key is the group's smallest
  member, which is **unique** because groups are disjoint (IV-1) - so the
  tie-break is a total order, and the two `HashMap` iteration sites are
  order-independent by construction. **No cross-crate dependency:** `Cargo.toml`
  byte-unchanged, and `cluster` appears in `crates/graph` only in **four prose
  mentions**, zero code references. The pre-accepted ~60-line duplication stands
  as documented.
- **F - Carried findings RESOLVED, not restated.** F4: the `adjacency()` doc now
  names **all three** post-delete production sites (it previously named 2 and
  omitted `detect_cycle_groups` outright), names both `#[cfg(test)]` families,
  **keeps the prohibition verbatim**, and reinforces it for the test side. I
  verified completeness against my own 7-site census. F5: the exact prescribed
  three-arm match is in place, all three variants load-bearing, and behaviour is
  **provably** identical (`continue` in a `while` is the implicit fall-through of
  the old empty `if let` body; `FrameStep` is exhaustively matched with no `_`).
  The file's entire diff is that hunk, +3/-2, and gate 4 reports zero warnings.
- **G - Line count: verified, and ACCEPTABLE.** `704 -> 734`, exact, confirmed
  by `Read` and by `numstat` (+105/-75). The plan's drop-to-~649 expectation and
  the handoff's ~679 are both wrong; the growth is **71 lines of mandated ported
  tests and ~7 of mandated docs against 56 lines of deleted production
  recursion**. `graph.rs` at 734 and `cycle_groups.rs` at 528 both hold **266 and
  472 lines** of real headroom, every function is well under 100 lines, and
  hygiene reports **zero R1/R2** - which disposes of the ceiling question
  outright. A file growing during a de-recursion milestone is acceptable **here**
  because what grew is test surface, not production surface, and the contract
  mandated it. Attributed to the planner's sizing arithmetic (finding 1), not to
  the implementer.

**Dev-1, Dev-2, Dev-3, Dev-4, Dev-5 all ACCEPTED**, with Dev-1 attributed to the
planner's arithmetic, Dev-2 required by rustfmt width, Dev-3 an honest disclosure
whose substitute evidence I verified from the code, Dev-4 to configuration, and
Dev-5 to nothing being skipped. **No undeclared deviation was found.**

**Honest limits of this review, stated rather than buried:** the *temporal
ordering* of the implementer's two Red observations and of the "orchestrator-
declared" status of `3e1efa6` are taken on report - the first is structurally
corroborated (I re-derived both Red failures from the frozen oracle's source and
they match the quoted verbatims exactly), the second by an exact T3-C precedent
on a pure-additions commit. And `ui_smoke.py` / `bench_compare.py` could not be
run at all, so there is **still no runtime UI evidence and no verified perf
evidence** for this milestone - and for this unit specifically that gap is at
its most material, because this unit is the one that changed what
`IncludeReport::cycle_count` means for every consumer that displays it (finding
9).

### Scoped canonical lines, reviewer-measured, verbatim

| Gate | Line |
|---|---|
| 1 (`--test cycle_groups_test`) | `9 passed / 0 failed / 0 ignored / 1 suites` |
| 2 (`--test cycle_groups_equivalence_test`) | `4 passed / 0 failed / 0 ignored / 1 suites` |
| 3 (`-p ariadne-graph`) | `118 passed / 0 failed / 0 ignored / 5 suites` |
| 4 (`cargo build -p ariadne-graph`) | `ok (exit 0, 0 warnings emitted)` |
| 5 (`--test m3_include_graph_test`, after the port) | `3 passed / 0 failed / 0 ignored / 1 suites` |
| 6 (`cargo build -p ariadne-cli -p ariadne-ui`) | `ok (exit 0, 0 warnings emitted)` |
| **workspace** | **`1178 passed / 0 failed / 4 ignored / 88 suites`** |

Workspace delta vs the `1176 / 0 / 4 / 88` baseline: **+2 / +0 / +0 / +0**, zero
unattributed residual. Tool suites: `Ran 184 tests in 18.164s` / `OK`. Hygiene:
zero R1/R2. Encoding: `--all` and `--staged` both silent, exit 0 each.

### Gates that could not be run

`testrun.py --baseline` (broken, F9: `usage error: missing value for --baseline`);
`ui_smoke.py` (REFUSED, F10); `bench_compare.py` (REFUSED, F10). Each attempted
**once, independently, directly**, verbatim recorded in 3.3 and 3.5, none
retried, **none routed through `distill.py`**. The invocation `--baseline` wraps
was run in full.

### Per-commit evidence, oldest first

| # | SHA | Paths (+/-) | In touch-set? | Judged |
|---|---|---|---|---|
| 1 | `adb53a3` | `graph.rs` +96/-14 | yes | seam port, **no deletion** - correct as a standalone commit |
| 2 | `acea7f2` | `closure_equivalence_tests.rs` +1/-1, `cycle_groups.rs` +3/-2, `graph.rs` +9/-61 - **all three together** | yes | **the atomic delete**: repoints `:534` and `:515`, deletes both recursive fns, lands F4 and F5. **No commit in the range breaks the build.** |
| 3 | `3e1efa6` | the two T3-C records +1382/-0 | declared records | pure additions; finding 12 |
| 4 | `ed5c467` | `tasks/m235-t3d-journal.md` +277/-0 | yes | pure additions |
| 5 | `aebfa9b` | `tasks/m235-t3d-journal.md` +9/-3 | yes | the journal's own SHA-record self-reference repair (T3-C Dev-2 pattern) |