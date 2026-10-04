# M23.5 T3-C Verdict - Scala Integration Consumers + IV-9 Acceptance Gate (tests/integration)

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Handoff:** `tasks/m235-t3c-handoff.md` (Phase 3 unit C of 5, plan T3.1 consumer
migration IV-12 + the explicit IV-9 acceptance gate)
**Journal audited:** `tasks/m235-t3c-journal.md` (declared deviations Dev-1..Dev-3)
**Base:** `91967c9` (T3-B tip), per the journal and the brief
**Range reviewed:** `91967c9..6c0c2b6`, branch `tarjan-s-scc`, worktree
`.kilo/worktrees/tarjan-s-scc`
**Commits (oldest first):** `b774362`, `d11a819`, `6c0c2b6`
**Execution-results addendum:** `tasks/m235-t3c-execution-results.md`

This file is written INCREMENTALLY, cheap gates first. Sections 1-6 are final;
section 7 onward is appended as each gate completes.

---

## 1. Ground truth

`python tools/checkpoint_guard.py status`, reviewer-run at session start:

- HEAD `6c0c2b6`, branch `tarjan-s-scc`
- DIRTY FILES (`git diff HEAD --numstat`, worktree + index vs HEAD): **`(none)`**
- UNTRACKED FILES: **`(none)`**
- RECENT COMMITS (last 5): `6c0c2b6`, `d11a819`, `b774362`, `91967c9`, `9f30c9a`
- WORKTREE CENSUS: 8 worktrees; this one `ahead=19 behind=13 dirty=1`

**Base identification: `91967c9` is correct and independently confirmed.**
`git log` shows `91967c9` ("docs(tasks): M23.5 T3-B journal ...") as the commit
immediately preceding `b774362`, and the T3-B verdict's own range header ends at
`91967c9`. So the reviewed range is exactly the three declared commits and
nothing else. (Note for the successor: the T3-C **handoff** pins its line
references at `b950963`, an earlier rev, while the **journal** correctly bases
itself at `91967c9` - see finding 6.)

**The `dirty=1` census entry is adjudicated in section 6.**

---

## 2. Commit-versus-touch-set check (PASS, 3 of 3 commits)

**Enforced touch-set.** Handoff sec2 declares exactly three paths:

- `tests/integration/tests/m20_scala_dependency_graph_test.rs`
- `tests/integration/tests/m22_scala_load_analysis_test.rs`
- `tasks/m235-t3c-journal.md` (new)

Plus the **orchestrator-declared records** carried by the dispatch task message
(`tasks/m235-t3b-verdict.md` + `tasks/m235-t3b-execution-results.md`; reviewers
cannot write git, so the adjudication record must land under an implementer's
scoped commit - same mechanism T3-B adjudicated at `9f30c9a`).

### Range-level name list, reviewer-measured

`git diff --name-only 91967c9..HEAD` - exactly five paths:

```
tasks/m235-t3b-execution-results.md
tasks/m235-t3b-verdict.md
tasks/m235-t3c-journal.md
tests/integration/tests/m20_scala_dependency_graph_test.rs
tests/integration/tests/m22_scala_load_analysis_test.rs
```

`git diff --numstat 91967c9..HEAD`:

| Added | Removed | Path |
|---|---|---|
| 212 | 0 | `tasks/m235-t3b-execution-results.md` |
| 847 | 0 | `tasks/m235-t3b-verdict.md` |
| 238 | 0 | `tasks/m235-t3c-journal.md` |
| **3** | **3** | `tests/integration/tests/m20_scala_dependency_graph_test.rs` |
| **3** | **3** | `tests/integration/tests/m22_scala_load_analysis_test.rs` |

**Negative touch-set checks, reviewer-measured, each EMPTY (no output):**
`-- crates`, `-- tests/golden`, `-- tests/fixtures`, `-- tools`.
Also empty: `git diff 91967c9..HEAD -- tests/integration/tests/m3_include_graph_test.rs`.

Therefore, absent from every commit of this range: **`crates/**` (so
`crates/lang-scala/**` per IV-12, and `crates/graph/**` per sec13), `tools/**`,
`tests/golden/**`, `tests/fixtures/**`, `.kilo/plans/**`, `Cargo.toml`,
`Cargo.lock`, `.gitattributes`, and every other `tests/integration/**` path
(including `m3_include_graph_test.rs`, `m23_symbol_isolation_test.rs`,
`m00_facts_path_parity_test.rs`).** Sec2's "you MUST NOT touch" list holds in
every direction, and the touch-set is enforced without being over-broad.

### Per-commit review, oldest first, each judged alone

| # | SHA | Subject | Paths (+/-) | In touch-set? |
|---|---|---|---|---|
| 1 | `b774362` | `test(integration): migrate Scala cycle consumers to cycle groups` | the two touch-set test files only, +3/-3 each | **yes** (sec2) |
| 2 | `d11a819` | `docs(tasks): land M23.5 T3-B reviewer records (verdict, execution results)` | `tasks/m235-t3b-verdict.md` (+847/-0), `tasks/m235-t3b-execution-results.md` (+212/-0) | **yes** (orchestrator-declared) |
| 3 | `6c0c2b6` | `docs(tasks): M23.5 T3-C journal -- SHAs b774362/d11a819, IV-9 gate, gate-5 identity pair` | `tasks/m235-t3c-journal.md` (+238/-0) | **yes** (sec2) |

Commit 1's full diff is section 4's subject matter and is reproduced there in
full; it is `3 added / 3 removed` per file and touches nothing else. Commits 2
and 3 are **pure additions** (0 deletions in both), so neither can have modified
a single existing byte anywhere.

**The forbidden fixture.** `tests/fixtures/basic/artifacts/includes/report.json`
appears in **NO** commit. Verified two ways: (a) `-- tests/fixtures` is empty
across the range; (b) the per-commit stat lists in the table above contain no
`tests/` path at all in any of the three commits.

**Scope verdict: PASS.** Every commit lands inside the declared touch-set plus
the orchestrator-declared records. This is the check that catches an over-broad
touch-set, and it is clean in both directions: enforced AND not over-broad.

---

## 3. STEP 1 gates, reviewer-run

| Gate | Command | Reviewer-measured result |
|---|---|---|
| hygiene | `python tools/hygiene_check.py` | **ZERO R1/R2 lines workspace-wide.** Every printed line is `R3 (warn)`. |
| encoding, all tracked | `python tools/encoding_guard.py --all` | **no output**, exit 0 |
| encoding, staged | `python tools/encoding_guard.py --staged` | run at section 7 |
| fixture state | `python tools/checkpoint_guard.py status` | run at section 1, re-run at section 6 |

### The R3 claim, independently checked (not relayed)

**Zero of the ~120 printed lines are in either file this unit edited.**
`tests/integration/tests/m20_scala_dependency_graph_test.rs` and
`m22_scala_load_analysis_test.rs` appear NOWHERE in the hygiene output, so the
zero is a measurement, not a skip: the scanner demonstrably reaches
`tests/integration/**` - it flags `tests/integration/tests/common/mod.rs:218`,
`tests/integration/tests/m5_reduced_model_test.rs:86`, and
`tests/integration/tests/m3_include_graph_test.rs:149`/`:150` (banned identifier
`res`) in the same tree this unit's commits live in. Those last two are on a
file this unit was forbidden to touch and did not touch, so they are pre-existing.

R3 is warn-only per `AGENTS.md`, so none of this is blocking; it is recorded
because the journal claims "every printed line an R3 warn in files this unit does
not touch" and that claim is literally true.

### Encoding

`--all` silent over the whole tracked tree: authored content is ASCII-clean and
no double-encoded UTF-8 entered through any of the three commits. Consistent with
Dev-3 (the implementer ran `--all` pre-commit and `--staged` post-commit, both
silent).

---

## 4. THE SCALA MIGRATIONS (IV-12) - independent verdict

**VERDICT: CORRECT. Neither assertion was weakened; the change is exactly what
the plan and the handoff specified, and nothing else.**

### The complete diff, reviewer-measured at zero context

`git diff -U0 91967c9..HEAD -- <the two files>` returns **exactly four hunks in
the entire range**, 3 added / 3 removed per file:

| File | Line | Removed | Added |
|---|---|---|---|
| `m20_scala_dependency_graph_test.rs` | 154 | `let cycles = graph.detect_cycles();` | `let cycle_groups = graph.detect_cycle_groups();` |
| | 156-157 | `cycles.len() >= 1,` / old message | `!cycle_groups.is_empty(),` / `"...is detected as one entangled group"` |
| `m22_scala_load_analysis_test.rs` | 442 | `let cycles = graph.detect_cycles();` | `let cycle_groups = graph.detect_cycle_groups();` |
| | 444-445 | `!cycles.is_empty(),` / old message | `!cycle_groups.is_empty(),` / `"...on the auto-shaped lane as a group"` |

**At zero context there is nothing else.** In particular the following are
outside every hunk - they are not even CONTEXT lines, they are untouched bytes:

- `m20:160-189` in full: the `TolerateWithinPackage` delegate
  (`&\|_file_id| CyclePolicy::TolerateWithinPackage`), `informational_cycle_count
  >= 1` (`:163`), `informational_cycle_count == cycle_count` (`:166-169`), the
  all-`Reject` delegate `generate_report()` (`:171`), `informational_cycle_count
  == 0` (`:172-175`), `blocking.cycle_count == tolerated.cycle_count`
  (`:176-179`), and the skip-when-zero serde pair (`:181-190`).
- `m22:447-453` in full: the same all-tolerate delegate, `informational_cycle_count
  == cycle_count` (`:449-452`), `informational_cycle_count > 0` (`:453`).

I read all of those lines in the tree at HEAD as well (m20 `:145-199`, m22
`:430-455`) and they are present, in that order, with those exact operators.

### Answering the three specific questions

**1. Was any assertion WEAKENED to make it pass? NO - and this is not a
judgement about intent, it is structural.** The only assertion whose truth
condition changed is the one at the call site, and the change is
`cycles.len() >= 1` -> `!cycle_groups.is_empty()` (m20) and
`!cycles.is_empty()` -> `!cycle_groups.is_empty()` (m22). Those two predicates are
**logically identical** in each file. In m22 the local binding was merely renamed;
the predicate is character-for-character the same test on a renamed value. In
m20 `len() >= 1` and `!is_empty()` are the same predicate on any finite
collection, including the empty one. **Not one assertion anywhere in either file
was made easier to satisfy.** No fixture was broken, no gate was loosened, and no
number was adjusted; the only edit to a numeric expression anywhere is
`>= 1` becoming `!is_empty()`, which is a re-expression, not a relaxation.

**2. Does m22's `informational_cycle_count == cycle_count` under all-tolerate
still mean the same thing at group granularity (IV-11), or has it become
accidentally true?** **Same meaning. The claim survives the representation
change and it is NOT accidental - and the mechanism is checkable in the source
rather than argued.**

The predicate delegate at `m22:448` is `&|_file_id| CyclePolicy::TolerateWithinPackage`
- a **constant function** that discards its `FileId` argument and always returns
`TolerateWithinPackage`, which is not `Reject`. So "every member file's policy is
non-`Reject`" holds for every candidate unit of classification - every reported
path today, and every entangled group under the ported rule - **by construction of
the delegate, identically in both representations**. The equality therefore
continues to assert exactly what it asserted before: *the classifier neither drops
nor duplicates anything*. It would fire if unit D's port returned fewer groups
than it counted as informational, double-counted, or filtered groups on some
other predicate. That is a real, falsifiable property, and it is preserved with
equal force under the new rule.

What the assertion is **not**: it is not a test of *per-file policy resolution*,
because a constant delegate cannot exercise that. That weakness is
**pre-existing and unchanged by this unit** - the same assertion, against the
same constant delegate, in the old path semantics had exactly the same blind
spot. The all-`Reject` counterweight that *does* pin the other direction lives in
m20 (`:171-179`, `informational_cycle_count == 0` plus
`blocking.cycle_count == tolerated.cycle_count`), and it is also untouched. So
the pair is preserved with no loss of strength. This is the plan's IV-11
"assertions preserved where meaningful" satisfied literally.

**3. Was any plugin-side change needed or made?** **NO - verified from the
commits, not from the journal.** `git diff --name-only 91967c9..HEAD -- crates`
returns **empty**, so `crates/lang-scala/**` appears in **no** commit of this
range; nor does any other `crates/**` path. And the necessity question is settled
from the tree: both Scala tests are **pure consumers of the graph API** -
`m20:154` and `m22:442` call `IncludeGraph::detect_cycle_groups()` and read back
`IncludeReport` fields; neither constructs a cycle policy by language plugin or
inspects an `Import` edge kind in the changed region. `IncludeKind::Import` (the
IV-12 tally the plan names) is unaffected because no Scala source, fixture, or
plugin rule is in the diff. **IV-12 is satisfied with zero plugin-side change, as
the plan requires.**

### The `len() >= 1` -> `!is_empty()` reasoning - correctly understood

The journal (Checkpoint 2) states it exactly right: the count changed **MEANING
(paths -> groups)**, not value, so raw-number comparison across representations is
meaningless, and the stance each test actually asserts is **non-emptiness** - "a
deliberate cycle exists" - which is IV-2-invariant ("a fixture has a cycle iff it
has at least one nontrivial SCC"). It then says the right consequence: because the
old predicate was already `is_empty()`-shaped in m22 and `>= 1` in m20, and both
are the non-emptiness predicate, `is_empty()` expresses the preserved stance
directly. **This is the correct reasoning and not a mechanical rename of `cycles`
to `groups`.** I verified it independently: the m20 predicate changed *shape*
only because `len() >= 1` and `!is_empty()` are the same predicate, and the
implementer chose the form that names the surviving invariant.

**A question the brief raises, resolved rather than flagged:** why was `>= 1` ->
`!is_empty()` applied at `m20:154` but *not* at `m20:163`
(`informational_cycle_count >= 1`), which is the same non-emptiness stance? Because
they are different kinds of quantity. `:154` compares the length of a
**locally-enumerated collection** (`detect_cycles()`'s return value), whose
representation this unit replaces - so its comparison had to be re-expressed. `:163`
compares a **report field** (`informational_cycle_count`), whose type and name are
retained and whose value unit D re-pins inside `graph.rs` with no edit here. The
handoff's asymmetry is principled, and the implementer followed the explicit
"do not edit :160-189" instruction rather than expanding scope. Correct.

### "ZERO arithmetic re-pin was needed" - CONFIRMED, and the reason is structural

Plan line 123 asks to "preserve the asserted stance, re-pin the arithmetic" for
m22's equality. I enumerate every numeric expression in the two tests and **none
of them pins an absolute non-zero count**: `>= 1`, `== 0`, `> 0`, and two
count-vs-count equalities. There is no absolute group count anywhere to re-pin,
so the instruction is vacuously satisfied. The implementer was right to make no
change rather than manufacture a cosmetic re-pin - and right to say so in the
journal instead of silently doing nothing. (Note for the successor: the *real*
re-pins are `graph.rs:617, 631, 640, 647`, which assert `report.cycle_count == 1`
absolutely; those are unit D's port of the six IV-11 unit tests, not this unit's.)

---

## 5. IV-9: THE HEADLINE PROOF - independent verdict

**VERDICT: GENUINE. `m3_include_graph_test.rs` is byte-identical at this HEAD to
its state at the unit's base commit. The unedited test really is the oracle, and
it really passes.**

### Byte-identity, proved at blob level, not inferred from a diff

| Revision | `git rev-parse <rev>:tests/integration/tests/m3_include_graph_test.rs` |
|---|---|
| `91967c9` (unit base) | `b6a54d5881dcf3d82ae8fcf56885fabd7e892a23` |
| `HEAD` (`6c0c2b6`) | `b6a54d5881dcf3d82ae8fcf56885fabd7e892a23` |

**Identical blob SHA-1.** Independently corroborated by
`git diff 91967c9..HEAD -- tests/integration/tests/m3_include_graph_test.rs`,
which is **empty**, by `--name-only` across the range, which never lists the path,
and by the per-commit stat lists in section 2, which contain no `tests/` path in
any of the three commits. Four independent methods, one conclusion. **If the file
had differed by a single byte this would be blocking; it does not.**

### The assertion itself, and that it is genuinely exercised

`m3_include_graph_test.rs:118-121`:

```rust
assert!(
    report.cycle_count > 0,
    "Cycle between a.h and b.h must be detected"
);
```

It sits inside `test_m3_edge_cases_duplicates_system_missing_cycles` (fn at `:80`,
attribute at `:79`), whose fixture writes `include/a.h` including `"b.h"` and
`include/b.h` including `"a.h"` (`:88-93`). **Gate 1's capture names that exact
test PASSING** - `test_m3_edge_cases_duplicates_system_missing_cycles ... ok` -
so the assertion is not merely present in a file, it is executed and satisfied.
Gate 1's capture also reads `0 filtered out`, which is the exact vacuity mode
handoff sec8 warns about, and it is absent.

The three ways the implementer claims to have verified uneditedness
(never in git's modified/staged list, in none of the three commits, and the gate
ran against the committed blob) are each independently reproducible, and I
reproduced all three above. **No reliance on the journal was needed.**

### Would the assertion still be MEANINGFUL if it HAD been edited? - the counterfactual

The brief asks me to judge this, and the answer is what makes IV-9 a real gate
rather than a formality. Consider the three things a reviewer would have had to
accept had the file been touched:

- **An edit to the operator** (`> 0` -> `>= 0`) would make the assertion
  **vacuous**: `usize >= 0` is a tautology, true even with no cycle at all. The
  original operator is load-bearing.
- **An edit that loosens the comparison to a different field** (e.g.
  `report.cycle_count > 0` -> `!graph.detect_cycle_groups().is_empty()`) would
  destroy the entire point: it would replace the **field-NAME contract** (IV-9:
  the DTO/manifest JSON key `cycle_count` survives and carries group count) with a
  **direct API call**, so the test would keep passing even if the report field were
  renamed to `group_count` or dropped entirely. IV-9 exists precisely because the
  *name* is the contract; a test that bypasses the name cannot witness it.
- **An edit that re-pins the arithmetic** (`> 0` -> `== 1`) would be defensible
  *only after* unit D, once group semantics are live. At this HEAD `cycle_count`
  is still `detected_cycles.len()` (`graph.rs:369`, DFS paths), so `== 1` would
  be pinning the OLD representation - the exact defect this unit was dispatched to
  prevent.

So the untouched assertion is doing real work: `report.cycle_count > 0` is the
only assertion in the tree that simultaneously (a) exercises the field through the
**report/DTO path** by that exact **name**, (b) proves the value is non-zero on a
fixture with a guaranteed 2-file mutual include, and (c) will keep passing
unchanged when unit D swaps the meaning underneath it from paths to groups
(1 group > 0). **Had it been edited, the milestone would have lost its IV-9 proof
without gaining anything.**

### One honest caveat, and it is a sequencing fact rather than a shortfall

At this HEAD, `report.cycle_count` is **still a DFS-path count**: `graph.rs:356`
still reads `let detected_cycles = self.detect_cycles();` and `:369` still sets
`cycle_count: detected_cycles.len()`. So gate 1 as it stands proves "the untouched
test passes on the current implementation", not yet "the untouched test survives
the semantic shift". **The second, stronger half of IV-9 becomes falsifiable only
at unit D**, when the field's meaning actually changes underneath the assertion.
That is inherent to the planner's 5-unit split (plan line 108 and
`tasks/m235-t3b-handoff.md` both state the deletion belongs to the
`crates/graph`-owning unit), and this unit was explicitly forbidden by handoff
sec1/IV-9 from touching report code. **It is contract compliance, not a defect.**
It is recorded as finding 5 so that **unit D's reviewer re-runs gate 1 and treats
it as the real IV-9 milestone-level gate** - the day `cycle_count` starts meaning
groups, that assertion is the witness, and it must be observed then, not now.

---

## 6. `checkpoint_guard status` and the F12 fixture - fifth identical reproduction

Reviewer-measured at session start:

| Signal | Value |
|---|---|
| DIRTY FILES (`git diff HEAD --numstat`) | **`(none)`** |
| UNTRACKED FILES | **`(none)`** |
| `git status` tracked modifications | exactly one: `modified: tests/fixtures/basic/artifacts/includes/report.json` |
| census `dirty=` for this worktree | `1` |

**Adjudication: the known F12 mechanism, CONFIRMED, not a new problem.**
Signature reproduced for the fifth time (T2-A finding 9, T2-B finding 10, T3-A
sec 7, T3-B sec 6, now T3-C): numstat clean, porcelain dirty, census `dirty=1`,
path in no commit. The path is unpinned by `.gitattributes` and this worktree has
`core.autocrlf=true`, so a run that rewrites the artifact invalidates the
stat-cache entry while `git diff` normalises line endings and finds no delta.

This unit runs the full C++ pipeline over two real fixture repos, which handoff
sec0 predicted would make the re-dirty likely; it reproduced, exactly as
predicted. The implementer's distinction - "numstat clean at preflight -> the
Read/Write restore was never needed and none was attempted" versus "porcelain-only
`dirty=1` at the end" - is literally true of my own independent measurement, and
correctly **no fix was attempted** (`git checkout --` / `git restore` are mutating
git and are denied to every role; rewriting a byte-equal file would be a content
change with no delta to make).

**Standing closeout obligation, unchanged and still open:** pin the path in
`.gitattributes` or make the artifact writer emit LF. Must land before T4.1.
**No CRLF bytes entered the repository through this unit** - `-- tests/fixtures`
is empty across `91967c9..6c0c2b6`.

Re-run after my own heaviest run (the full `ariadne-integration-tests` crate,
which executes the C++ pipeline over both fixture repos):

| Signal | at session start | after my gate-5 run |
|---|---|---|
| DIRTY FILES (`git diff HEAD --numstat`) | **`(none)`** | **`(none)`** |
| UNTRACKED | `(none)` | `tasks/m235-t3c-verdict.md` (my own artifact) |
| census `dirty=` for this worktree | `1` | `2` (the +1 is my untracked verdict file) |

The forbidden fixture's content is still byte-equal to HEAD after the heaviest run
of the unit **and** after my own independent run. Unchanged signature, unchanged
conclusion.

---

## 7. STEP 2: the four scoped gates - reviewer-run, nothing relayed

| Gate | Command | Reviewer-measured canonical line |
|---|---|---|
| 1 (IV-9) | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m3_include_graph_test` | `3 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m00_golden_baseline_test` | `2 passed / 0 failed / 0 ignored / 1 suites` |
| 3 | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m20_scala_dependency_graph_test` | `5 passed / 0 failed / 0 ignored / 1 suites` |
| 4 | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m22_scala_load_analysis_test` | `6 passed / 0 failed / 0 ignored / 1 suites` |
| 5 (crate) | `python tools/testrun.py --distill --timeout 3600 -p ariadne-integration-tests` | `139 passed / 0 failed / 0 ignored / 34 suites` |

**All four scoped lines match the handoff sec8 table verbatim.** All four exit 0.

### None of the four is vacuous - the suite decomposition, from the captures

Every scoped capture reads `0 filtered out`, which is the specific vacuity mode
handoff sec8 warns about. And in each capture the **test that carries the
property under audit is named PASSING**:

| Gate | capture evidence |
|---|---|
| 1 | `running 3 tests`; **`test_m3_edge_cases_duplicates_system_missing_cycles ... ok`** (the carrier of `report.cycle_count > 0` at `:118-121`); `3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`; `Running tests\m3_include_graph_test.rs` |
| 2 | `running 2 tests`; `golden_baseline_basic ... ok`; `golden_baseline_expensive ... ok`; `0 filtered out`; `Running tests\m00_golden_baseline_test.rs` |
| 3 | `running 5 tests`; **`fixture_cycle_is_informational_under_tolerate_policy ... ok`** (the migrated call site); `0 filtered out`; `Running tests\m20_scala_dependency_graph_test.rs` |
| 4 | `running 6 tests`; **`auto_shaped_graph_keeps_the_cycle_informational ... ok`** (the migrated call site); `0 filtered out`; `Running tests\m22_scala_load_analysis_test.rs`. The capture also shows the auto-shaped lane genuinely executing the pipeline (`[2/5] Indexing translation units... 100%  5/5 TUs`, `[5/5] Analyzing collateral... 100%  Payload assembled`), so the group count came from a real run, not a stubbed seam. |

**The migrated gates are load-bearing, which is the substantive point.** Gates 3
and 4 do not merely re-confirm that two tests still exist; they assert, at
runtime, that `detect_cycle_groups()` returns a **non-empty** vector on two
different fixtures built by two different lanes (m20's injected shaper seam and
m22's auto-selected lane). Had group detection failed on either lane - no
nontrivial SCC found, group filter too aggressive, `CycleGroup` construction
broken - those two gates would have gone **red**. So "zero arithmetic re-pin was
needed" is not the vacuous claim it would be if the migration had changed nothing
observable; it is the result of two genuine non-emptiness gates being re-verified
against the new representation.

### The pinned test arithmetic, recounted from the tree at HEAD

| File | Attributes reviewer-located | Total | Gate line matches |
|---|---|---|---|
| `m3_include_graph_test.rs` | `#[test]` at `:14`, `:79`, `:146` | 3 | yes |
| `m00_golden_baseline_test.rs` | `#[test]` at `:39`, `:44` | 2 | yes |
| `m20_scala_dependency_graph_test.rs` | `#[test]` at `:85`, `:120`, `:149`, `:194`, `:236` | 5 | yes |
| `m22_scala_load_analysis_test.rs` | `#[test]` at `:119`, `:183`, `:247`, `:413`, `:436` + `#[tokio::test]` at `:300` | 6 | yes |

Every line number matches handoff sec8's arithmetic **exactly - zero line drift**,
which is worth recording because handoff sec0 invites drift reporting and there is
none. Each file is one test target, hence one suite each. None of the four
carries an `#[ignore]`, so no test in any scoped gate is hidden from the count.

---

## 8. THE GOLDEN DETERMINATION - confirmed, and I found the one gap in the argument

**VERDICT: CORRECT. No re-pin was needed, none was made, and the reasoning holds
- with one gap in the stated argument that I close here.**

### The grep, run by me

`cycle_count` over `tests/golden/**` returns **exactly four matches**, all of the
form `"cycle_count": 0,`:

| File | Line | Content |
|---|---|---|
| `tests/golden/basic/includes_report.json` | 2 | `  "cycle_count": 0,` |
| `tests/golden/expensive/includes_report.json` | 2 | `  "cycle_count": 0,` |
| `tests/golden/basic/includes_manifest.json` | 3 | `  "cycle_count": 0,` |
| `tests/golden/expensive/includes_manifest.json` | 3 | `  "cycle_count": 0,` |

Matches the journal's claim and handoff sec7's claim, exactly.

### Byte-identity, proved at blob level

| Revision | `git rev-parse <rev>:tests/golden/basic/includes_report.json` |
|---|---|
| `91967c9` | `dff8018df38d9fe42d540a1dbc882eb263b4d091` |
| `HEAD` | `dff8018df38d9fe42d540a1dbc882eb263b4d091` |

Identical. And `git diff --name-only 91967c9..HEAD -- tests/golden` is **empty**,
so **all four** goldens are byte-identical, not just the one I hashed.

### `UPDATE_GOLDENS=1` was never run

The regeneration switch is a documented env var read at
`tests/integration/tests/common/mod.rs:63-65`
(`matches!(std::env::var("UPDATE_GOLDENS").as_deref(), Ok(v) if v == "1")`), and
`m00_golden_baseline_test.rs:9` documents the mode. Two independent confirmations
that it was not used:

1. **Byte-identity.** If the mode had run on either fixture, the written golden
   would be the freshly-serialised report; `-- tests/golden` being empty across
   the range means the committed blobs are the pre-existing ones, untouched.
2. **Gate 2 is green against those untouched blobs**, so the freshly-computed
   reports and the committed goldens still agree byte-for-byte - which is what
   makes this a genuine lock rather than a self-fulfilling rewrite.

**No file was added or changed under `tests/fixtures/basic` or
`tests/fixtures/expensive`** - `-- tests/fixtures` is empty across the range, and
numstat DIRTY is `(none)` after the heaviest run.

### Closing the gap: could a golden hide a NONZERO cycle value that a field-name search would miss?

The brief asks the sharpest form of this question, and it deserves a sharp answer
rather than "the grep found four hits".

**The risk would be a golden that serialises a cycle LIST under a field name
other than `cycle_count`** - a `cycles: [...]` array, a `cycle_paths` array, a
representative-cycle witness - which the substring search for `cycle_count` would
not reveal. **That risk is structurally impossible here, and I verified the
structure rather than assuming it.** `IncludeReport`
(`crates/graph/src/types.rs:47-64`) is the serialised report type and its complete
field list is:

```
total_edges, unique_headers, local_headers_count, system_headers_count,
external_headers_count, unresolved_includes_count, cycle_count,
informational_cycle_count, top_headers_by_closure
```

**There is no cycle-list field anywhere in the type.** `cycle_count` and
`informational_cycle_count` are both `usize`; `top_headers_by_closure` is
`Vec<(FileId, PathBuf, usize)>`, closure sizes, not cycles. A report type with no
cycle-list field **cannot** serialise a concrete cycle list, so the premise of the
risk - that some other key hides a cycle enumeration - has no carrier. `CycleGroup`
(`crates/graph/src/cycle_groups.rs`) is **not** reachable from `IncludeReport` at
all at this HEAD, and the goldens are `includes_report.json` /
`includes_manifest.json`, both projections of that type.

So the two candidate failure modes are both closed:

- A golden with a **nonzero `cycle_count`**: the grep would have revealed it (it
  searched the field name, and no other key can carry the count). None exists.
- A golden with a **cycle list**: impossible by the type's shape, above.

**And the direction of change is safe in the strong sense**: both golden fixtures
are **acyclic**, so the count is 0 under BOTH representations - 0 paths implies 0
nontrivial SCCs by IV-2, hence 0 groups. The byte `"cycle_count": 0` is
representation-**invariant**, not merely equal today. No future group-semantics
change can perturb it either. **The determination is not merely correct, it is
stable.**

### The one thing the stated argument did not cover

Handoff sec7 and the journal both argue from "there is no field that serialises a
concrete cycle list". That argument is correct, but it is made about
`IncludeReport` only; it does not explicitly rule out a cycle list in the
**manifest** (`includes_manifest.json`), which is a different artefact type. In
this instance it does not matter - the manifests are generated by the same
`IncludeReport` projection and read `"cycle_count": 0` - but the gap is closed by
the grep plus the type-level argument above rather than by the journal's text, and
a successor should not inherit "the type has no list field" as sufficient on its
own for a manifest whose generating code it has not read. Recorded as non-blocking
finding 4 so the reasoning is not over-generalised later.

---

## 9. GATE 5 IDENTITY PAIR - the actual evidence constraining a pure migration

**VERDICT: SOUND, and the delta is not merely reported but arithmetically forced.**

### The pair

| | Line | Capture |
|---|---|---|
| BEFORE first edit (implementer, Checkpoint 1) | `139 passed / 0 failed / 0 ignored / 34 suites` | `target/testrun-capture.txt`, 709 lines |
| AFTER the edit (implementer, gate 5) | `139 passed / 0 failed / 0 ignored / 34 suites` | 698 lines |
| **AFTER the edit (reviewer, re-run)** | **`139 passed / 0 failed / 0 ignored / 34 suites`** | **698 lines** |

The implementer's handoff deliberately refused to pre-pin this line ("I am not
giving you a guessed `139 / 0 / x / y` line, because a guess is exactly the defect
that made Phase 1's gates unusable"). I re-ran it and reproduced the AFTER line
**verbatim**, including the 698-line capture length - so the AFTER state I measure
is the same state the implementer measured. Good discipline on both sides: the
handoff refused to guess, and the implementer then measured.

### Does the pair GENUINELY BRACKET the change?

This is the question that decides whether the device is real. **Yes - and I can
prove the "no test added or removed" half independently, so the pair's credibility
does not rest on the implementer's word about when they pressed enter.**

The proof is the zero-context diff (section 4). Across the entire range the two
test files change in exactly four hunks: two call lines, and two
comparison-plus-message pairs. **Not one `#[test]` or `#[tokio::test]` attribute
appears on any `+` or `-` line, and no test function signature appears on one.** A
diff that touches only those six lines cannot add a test, remove a test, or rename
one. Therefore the crate's test inventory is **provably identical** before and
after, and `139 == 139` is a consequence of the diff rather than of a claim about
it.

The only thing a call migration inside two existing test bodies can change is
**pass/fail status**, and that is exactly what the pair measures: 0 failed both
times, with both carriers named PASSING (section 7). The pair's discriminating
power is intact.

Had the BEFORE been measured after the first edit, the pair would have been vacuous
- it would assert `X == X` over one state. I cannot replay the implementer's
session, so I state the limit plainly: **the temporal ordering of the implementer's
BEFORE measurement is the one element of this unit I take on report**, corroborated
but not independently reproducible by a read-only reviewer. It is consistent with
two supporting facts: the journal records it before the first edit and quotes a
709-line capture (differing from the AFTER 698-line capture, as a pre-edit state
should), and the structural proof above shows the *delta* is 0 whether or not the
ordering claim is true. **If the BEFORE had in fact been mis-taken, the finding
would be a reporting-process defect, not an unverified behaviour claim** - because
the test inventory cannot have changed.

### Independent confirmation of the counts

**Suite count 34, reviewer-measured, decomposed.** The capture names exactly 34
suite headers: `Running unittests src/lib.rs`, 32 `Running tests\*.rs` targets
(`m00_facts_path_parity_test`, `m00_golden_baseline_test`, `m0_skeleton_test`,
`m10_validation_test`, `m11_measurement_test`, `m12_incremental_analysis_test`,
`m13_production_scale_test`, `m14_end_to_end_test`, `m15_graph_ui_test`,
`m16_clang_worker_test`, `m16_traversal_test`, `m17_dir_nav_test`,
`m17_plugin_seam_test`, `m18_cache_epoch_test`, `m18_cpp_plugin_parity_test`,
`m18_persistence_incremental_test`, `m19_optimize_characterization_test`,
`m19_scala_discovery_test`, `m1_discovery_test`,
`m20_apply_migrate_characterization_test`, `m20_scala_dependency_graph_test`,
`m21_run_ui_smoke_test`, `m21_scala_semantic_index_test`,
`m22_scala_load_analysis_test`, `m23_symbol_isolation_test`,
`m24_language_identity_test`, `m2_build_discovery_test`, `m3_include_graph_test`,
`m4_semantic_indexing_test`, `m5_reduced_model_test`, `m6_load_analysis_test`,
`m7_optimization_planning_test`), and `Doc-tests ariadne_integration_tests`.
**1 + 32 + 1 = 34**, matching 34 `test result:` lines. This is the same
"count the lib/bin/doc targets the planner forgets" arithmetic that produced T3-B
finding 2 - and here the handoff's figure is **correct**.

**Test count 139, reviewer-measured by decomposition, zero unattributed residual.**
The capture has exactly 34 `test result:` lines and **every one of the 34 reads
`0 filtered out`** - nothing silently filtered, in any suite. Summing the `passed`
fields of all 34 gives **139**, matching the canonical line with nothing left over.
Two of them legitimately read `0 passed` (the empty `src/lib.rs` target and one
zero-test target), which is why a 34-suite decomposition with a 139-test total is
coherent rather than suspicious.

The crate's per-target counts agree with the four scoped gates read out of the same
capture: `m3_include_graph_test` = `3 passed` (line 122), `m00_golden_baseline_test`
= `2 passed` (line 18), `m20_scala_dependency_graph_test` = `5 passed` (line 34),
`m22_scala_load_analysis_test` = `6 passed` (line 203). **No contradiction between
the scoped gates and the whole-crate run.**

**IV-8 planner byte-identity suites unaffected, as sec9 requires.**
`Running tests\m00_facts_path_parity_test.rs` is present (header line 666) and green
(`2 passed`, line 11); the path appears in no commit of the range.
`m23_symbol_isolation_test.rs` (whose `:205` `cycle_count == 0` lock sec9 names) is
also present (header line 690) and green, and is in no commit.

---

## 10. THE MISSING RED - adjudicated

**VERDICT: DECLINING TO FABRICATE WAS THE CORRECT CALL. The disclosure is
honest, the reasoning is sound, and the substituting evidence is sufficient.**

### Why "no genuine failing-first moment exists" is true, not convenient

The implementer's argument (journal Checkpoint 1, pre-authorised by handoff sec4)
is that this unit is a **two-line rewiring of call sites onto an API that already
exists, is already public, and is already green in its own suites**. I verified
every premise independently:

- `detect_cycle_groups` is real, public, and reachable from a test:
  `crates/graph/src/cycle_groups.rs:39 pub fn detect_cycle_groups(&self) -> Vec<CycleGroup>`.
- It is already exercised by `crates/graph/tests/cycle_groups_test.rs` and
  `cycle_groups_equivalence_test.rs` (reviewer-verified APPROVED at `b950963`,
  per the T3-A / T3-B records).
- The old call still compiles and still passes until unit D deletes it - which the
  **plan itself confirms** at line 108: `detect_cycles`/`dfs_cycle` "cannot be
  deleted without breaking the workspace build" from any non-`graph`-scoped unit,
  because three of seven call sites live outside `crate:graph` - **two of which are
  exactly this unit's two call sites.**

So the old API cannot be removed to force a failure (sec4 forbids it and the plan
independently forbids it), no new behaviour is introduced, and no fixture can be
honestly broken. **There is no failing-first state that is not a lie.** The handoff
recognised this in advance, pre-authorised the disclosure, and cited the identical
precedent T2-A's unobservable Red (deviation C), whose reviewer accepted it. That
is a consistent institutional position, not a case-by-case excuse.

### Is the substitute evidence sufficient?

**Yes - and it is stronger than the implementer's own framing suggests**, because
the substitution is not merely "no delta in counts":

1. **The gate-5 identity pair** (section 9) shows the whole 139-test crate behaves
   identically across the change, with `0 filtered out` in all 34 suites, and the
   inventory delta proven structurally 0.
2. **The two migrated carriers are named PASSING** (section 7). This is the part
   the journal understates: it is not merely that no test broke, it is that the
   **new** code path was *executed and returned a non-empty result* on two distinct
   fixtures. If `detect_cycle_groups()` were broken, wrong, or returned empty on
   either lane, gates 3 and 4 would be red. **The migration is positively
   witnessed, not merely unfalsified** - a stronger form of evidence than a Red
   would have been for a mechanical rewiring, because a Red would only have shown
   that a typo was detectable.
3. **The six `informational_cycle_count` unit tests in `graph.rs` (`:614-687`) are
   green** - untouched by this unit, but they pin the report-side classification the
   migrated tests depend on, and they are inside the workspace line measured at
   section 13.

### The limit, stated so it is not mistaken for more than it is

What is genuinely unverified by this unit is what it was **not** dispatched to
verify: that the *report* field has been re-based onto group semantics. At this HEAD
`graph.rs:369` still reads `cycle_count: detected_cycles.len()` over DFS paths, so
**no test anywhere in the tree yet observes `cycle_count` carrying a group count.**
Criterion 2's first clause ("`cycle_count` reports entangled-group count") is not in
scope for this unit - it is unit D's deliverable, and unit D's handoff carries the
re-pins of `graph.rs:617/:631/:640/:647`. **So: the consumer-side migration is
fully witnessed; the provider-side semantic shift remains unobserved until unit D,
which is correct sequencing rather than a gap here.** Recorded as finding 5 for
unit D's reviewer.

---

## 11. Item 4 was NOT placed here, and that is CORRECT

**VERDICT: the dispatch's conditional was genuinely false. Declining the item was
the right call, and no work was dodged.**

The dispatch conditionally assigned `closure_equivalence_tests.rs:515` to this unit
- "**If the handoff places that repoint in your unit**". I evaluated the condition
against the tree and the contract, not against the implementer's account:

1. **The file is not in this unit's scope.** A glob for `**/closure_equivalence*`
   across the worktree returns **exactly one** path:
   `crates/graph/src/closure_equivalence_tests.rs` - under `crates/`, specifically
   `crates/graph/src/`, **not** under `tests/integration/`.
2. **It is absent from the handoff's sec2 touch-set.** Sec2 lists exactly three
   paths (the two Scala test files and the journal). Nothing under `crates/**` is
   listed, and the handoff states: "You MUST NOT touch: ... `crates/**`".
3. **Sec13 forbids it by name.** "Do not delete `detect_cycles` or `dfs_cycle`, do
   not touch `crates/graph/**`". Touching it would have been a **blocking** scope
   violation, not a permitted extra.
4. **The call site is untouched, as it must be.**
   `crates/graph/src/closure_equivalence_tests.rs:515` still reads
   `let cyclic = !graph.detect_cycles().is_empty();` - the old predicate, intact.
   `-- crates` is empty across `91967c9..6c0c2b6`, so the file is in **no** commit.
5. **Plan line 123 assigns the repoint to the same unit as the deletion:**
   "Delete `detect_cycles`/`dfs_cycle`; repoint graph.rs:356/:534 **and**
   `closure_equivalence_tests.rs:515`" - and plan line 108 assigns both to the
   `crates/graph`-owning unit. T3-B's verdict sec 5.6/7 records the same ownership.

**Conclusion: the condition is false, the file is out of scope and untouched, and
the next unit repoints it atomically with the deletion - which is correct, because
splitting a method's deletion from its last repoint is exactly how a workspace build
breaks.** The journal's note ("per the dispatch's own conditional, the condition is
false") is literally true, and it correctly notes that the IV-2-equivalence licence
the T3-B reviewer established is unit D's to spend. **No misreading, no avoided
work: the correct action was inaction.**

---

## 12. Frozen locks that must not have moved - reviewer-verified

| Lock | How verified | Result |
|---|---|---|
| `detect_cycles` still EXISTS | grep `fn detect_cycles` workspace-wide -> `crates/graph/src/graph.rs:234 pub fn detect_cycles(&self) -> Vec<Vec<FileId>>` | **HOLDS** |
| `dfs_cycle` still EXISTS | same grep -> `crates/graph/src/graph.rs:260 fn dfs_cycle(` | **HOLDS** |
| both still CALLED (the unit-D precondition) | grep `detect_cycles()` -> **three** call sites survive: `graph.rs:356` (`let detected_cycles = self.detect_cycles();`), `graph.rs:534` (`let cycles = graph.detect_cycles();`), `closure_equivalence_tests.rs:515` (`let cyclic = !graph.detect_cycles().is_empty();`) | **HOLDS - all three intact** |
| no *external* `detect_cycles` caller left behind | the same grep finds **zero** `detect_cycles()` occurrences under `tests/**`; the only other mentions are doc-comment references in `crates/graph/tests/cycle_groups_test.rs:5` | **HOLDS - this unit's migration is complete, which is precisely what unit D's delete-gate needs** |
| `detect_cycle_groups` public and reachable | `crates/graph/src/cycle_groups.rs:39 pub fn detect_cycle_groups(&self) -> Vec<CycleGroup>` | **HOLDS** (exercised successfully by both migrated tests, gates 3 and 4) |
| IV-9 `cycle_count` field name RETAINED | `crates/graph/src/types.rs:56 pub cycle_count: usize`; `crates/graph/**` in no commit | **HOLDS** |
| IV-9's `cycle_count` byte at the CLI | `crates/cli/src/commands/ui.rs:390` untouched (no `crates/**` path in any commit) | **HOLDS** |
| IV-11 `informational_cycle_count` name + **skip-when-zero serde shape** | `types.rs:61-62` `#[serde(default, skip_serializing_if = "informational_cycles_absent")]` over `types.rs:69-71 fn informational_cycles_absent(count: &usize) -> bool { *count == 0 }`, all untouched | **HOLDS - predicate byte-intact** |
| IV-11's six graph.rs unit tests unedited | `graph.rs:617/:618/:631/:632/:640/:641/:647/:648` all present; `crates/graph/**` in no commit | **HOLDS** |
| policy callback signature untouched | `generate_report_with_cycle_policies(&dyn Fn(FileId) -> CyclePolicy)` still called from `m20:161` and `m22:448`; `crates/graph/**` in no commit | **HOLDS** - and plan line 124's "`m23_symbol_isolation_test`'s call stays compiling" is satisfied (target green in gate 5) |
| `m3_include_graph_test.rs` unedited | identical blob SHA-1 at base and HEAD (section 5) | **HOLDS** |
| `m00_facts_path_parity_test.rs` (IV-8 planner lock) unedited | in no commit; target present and green in the gate-5 capture | **HOLDS** |
| `m23_symbol_isolation_test.rs` unedited | in no commit; target present and green in the gate-5 capture | **HOLDS** |
| `crates/lang-scala/**` untouched (IV-12 plugin-side-free) | `-- crates` empty across the range | **HOLDS** |
| `tests/golden/**` byte-unchanged | `-- tests/golden` empty; blob SHA-1 identical (section 8) | **HOLDS** |
| `tests/fixtures/**` byte-unchanged, no new files | `-- tests/fixtures` empty; numstat `(none)` after the heaviest run | **HOLDS** |
| the unpinned F12 fixture path | in no commit (sections 2 and 6) | **HOLDS** |
| R1/R2 hygiene | `python tools/hygiene_check.py` | **zero R1/R2** |
| UTF-8 mojibake | `python tools/encoding_guard.py --all` and `--staged`, both | **no output, exit 0 each** |
| touch-set | per-commit `--stat` name lists (section 2) | **exactly the in-set paths + declared records** |
| journal report is digest-shaped | canonical lines, exit codes, capture line counts; **zero raw cargo/git output pasted**; the one refusal quoted as the catch-all text | **HOLDS** |

---

## 13. Workspace-wide acceptance pass

### 13.1 Every workspace-wide gate: ran, could not run, and why

Each known-broken / known-refused item was attempted **INDEPENDENTLY, exactly
once**, by this reviewer, directly. **None was retried, and none was routed
through `distill.py` or any other wrapper to evade a denial.**

| Gate | Command | Result |
|---|---|---|
| baseline | `python tools/testrun.py --baseline` | **BROKEN.** Verbatim: `usage error: missing value for --baseline`. Ledger M23.5 F9, reproduced. NOT treated as a gate failure, NOT worked around, NOT routed through `distill.py`. |
| full workspace (what `--baseline` wraps) | `python tools/testrun.py --distill --timeout 3600` | see 13.2 - **COMPLETED** |
| tool unit suites | `python -m unittest discover -s tools -p "*_tests.py"` | see 13.3 - **COMPLETED** |
| hygiene | `python tools/hygiene_check.py` | **zero R1/R2**; every printed line a legacy-file R3 warn (section 3) |
| encoding, all tracked | `python tools/encoding_guard.py --all` | **no output**, exit 0 |
| encoding, staged | `python tools/encoding_guard.py --staged` | **no output**, exit 0 |
| fixture state | `python tools/checkpoint_guard.py status` | run at session start, after my gate-5 run, and after my workspace run (sections 1, 6) |
| UI runtime smoke | `python tools/ui_smoke.py` | **REFUSED.** Verbatim: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` |
| load-perf regression | `python tools/bench_compare.py` | **REFUSED.** Verbatim: identical text. |

**On the two refusals.** A refusal ALWAYS reports the catch-all deny rule whatever
the real cause - here the real cause is that neither script is on the reviewer bash
allowlist (ledger M23.5 F10), not that bash is unavailable. Both were attempted
directly and once each; neither was retried through a wrapper. **The denials did
not cause any gate to be skipped**: hygiene, `encoding_guard --all`,
`encoding_guard --staged`, `checkpoint_guard status` (three times), all four scoped
gates, the whole-crate integration run, the full-workspace run and the Python tool
suite all ran. **This unit makes no UI change and no perf-pinned number, so the two
refusals are recorded as standing, not as new exposure** - the `ui_smoke` gap
escalated by T3-A (this HEAD is the first that serves group currency in the UI
payload) is unchanged by this unit and remains open at finding 6.

**Two reviewer-shaped refusals also occurred, recorded for the next reviewer's
benefit.** (a) A compound command (`git diff ... --stat ... ; echo ...; git diff
--name-only ...`) was refused with the same catch-all text. (b) A
`python -c "..."` inline summariser, and a `git grep -e/-c` with multiple `-e`
flags, were each refused with the same text. In every case the real cause was my
own command shape, not an unavailable gate; I re-issued each query as a plain
single command or via the dedicated tools and all of them succeeded. No acceptance
gate was narrowed as a result.

### 13.2 The canonical workspace line - COMPLETED

`python tools/testrun.py --distill --timeout 3600`, exit 0, capture
`target/testrun-capture.txt` (2064 lines). Verbatim:

```
1176 passed / 0 failed / 4 ignored / 88 suites
```

Compared against the ledger baseline the brief named:

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| ledger baseline / expected at this unit (`6c0c2b6`) | 1176 | 0 | 4 | 88 |
| **reviewer-measured (`6c0c2b6`)** | **1176** | **0** | **4** | **88** |
| **Delta** | **+0** | **+0** | **+0** | **+0** |

**The delta is exactly +0 in all four columns, as the brief predicted** (this unit
added no test), and the line is identical to the one the T3-B reviewer measured at
`91967c9` - which is the expected outcome, since the T3-C diff adds and removes no
test anywhere. **Zero unattributed residual.**

Non-vacuity, reviewer-measured: the capture contains exactly **88 suite headers**
(74 `Running ...` target lines + 14 `Doc-tests ...` lines), matching `88 suites`,
and **zero** `test result: FAILED` or `failures:` lines, matching `0 failed`. The
decomposition independently reconfirms the T3-B suite-arithmetic finding at line
1979-1980 of this capture, which show `Running unittests src\main.rs (ariadne-...)`
**and** `Running unittests src\bin\tool.rs (tool-...)` as separate targets - so
`-p ariadne-cli` is **5 suites, not the 3** the ledger row records (finding 2).

### 13.3 Tool unit suites - COMPLETED

`python -m unittest discover -s tools -p "*_tests.py"`:

```
Ran 184 tests in 18.152s

OK
```

Identical count to the T3-B reviewer's acceptance pass (`Ran 184 tests`, `OK`) -
correct, since this unit changes no `tools/**` file (`-- tools` empty across the
range).

### 13.4 Fixture state after the workspace run

`python tools/checkpoint_guard.py status`, third run, after the heaviest possible
invocation (full workspace, 2064-line capture): HEAD `6c0c2b6`, **DIRTY FILES
`(none)`**, UNTRACKED only `tasks/m235-t3c-verdict.md` (my own artifact), census
`dirty=2` = 1 x F12 stat-cache entry + my untracked verdict file. **The forbidden
fixture is still absent from the numstat dirty section after my full-workspace
run** (section 6).

### 13.5 Everything that was NOT run

Stated plainly, as required:

- `python tools/testrun.py --baseline` - **could not run**, broken
  (`usage error: missing value for --baseline`, F9). Attempted once, verbatim
  recorded, not worked around. **The invocation it wraps WAS run** (13.2).
- `python tools/ui_smoke.py` - **could not run**, REFUSED (F10). Attempted once,
  verbatim recorded, not retried, not wrapped. **No runtime UI evidence exists for
  this milestone at this HEAD** (finding 6).
- `python tools/bench_compare.py` - **could not run**, REFUSED (F10). Attempted
  once, verbatim recorded. **No verified perf evidence exists for
  `detect_cycle_groups`.** This unit changes no pinned timing, and inherits the
  `5.6` single-local-measurement label from T3-B verbatim; it neither improves nor
  worsens that standing disclosure.
- **The implementer's BEFORE capture for gate 5** - not recoverable. The implementer
  correctly never committed a pre-edit state, and `target/` is ephemeral, so the
  709-line pre-edit capture no longer exists. Its line survives only in the
  committed journal. I re-ran and reproduced the AFTER side exactly (section 9) and
  proved the inventory delta structurally 0, so this is **not** a shortfall in the
  evidence that constrains the change - but the *temporal ordering* of the BEFORE
  measurement is taken on report, as stated plainly in section 9.
- **No commit was made by this reviewer** (write git is denied to this role).

---

## 14. Dev-1..Dev-3, adjudicated independently

Adjudicated on evidence - the spec and the tree, not the implementer's
justification.

| # | Deviation | Disposition | Attributed to |
|---|---|---|---|
| Dev-1 | `d11a819` carries `tasks/m235-t3b-verdict.md` + `tasks/m235-t3b-execution-results.md`, outside the sec2 touch-set | **ACCEPTED** | orchestrator (the declaration), not implementer |
| Dev-2 | handoff sec8 gate 9 lists ONE commit; three were landed | **ACCEPTED** - and required by the handoff's own header | n/a (implementer judgement, contract-conformant) |
| Dev-3 | gate 7's `--staged` form cannot interpose with staging; `--all` (strictly stronger) was run pre-commit | **ACCEPTED** | configuration (`commit_scoped.py` has no stage-only mode) |

**Dev-1 - ACCEPTED; the declaration came from outside the handoff, and the
mechanism is the only one available.** The two records are the T3-B reviewer's
own adjudication, which a reviewer cannot commit (write git is denied to this
role). The dispatch task message declared them into this unit's commit scope, and
the precedent is exact: `9f30c9a` did the same for the T3-A records. Verified: the
commit carries **exactly those two paths, +847/-0 and +212/-0, zero deletions** -
so it cannot have modified any existing byte, and `--name-only` confirms no other
out-of-touch-set path was staged anywhere in the range. **The touch-set was
temporarily widened by a higher authority, explicitly, for a records-only commit.**
That is the correct mechanism and it is a **dispatch/declaration** matter, not an
implementer scope violation. Recorded so the next reviewer does not read
`tasks/m235-t3b-*.md` in the range as an implementer scope breach.

**Dev-2 - ACCEPTED, and arguably REQUIRED rather than discretionary.** Handoff
line 9 instructs: "**Journal:** `tasks/m235-t3c-journal.md` (you create it; record
your own SHAs in it)". A journal cannot contain the SHA of a commit that includes
it - a genuine self-reference. The handoff's own sec8 gate 9 then names a single
commit carrying all three scoped paths, which is arithmetically impossible alongside
its header instruction. The implementer split into (a) the code commit carrying the
exact sec8 gate-9 message text verbatim, (b) the declared records, (c) the journal
last - which is the same resolution T3-B adjudicated as REQUIRED at its D6. **Every
commit carried an explicit `--scope`**, so the touch-set discipline was never
relaxed. The residual defect is in the contract's internal inconsistency, not in the
implementation: **handoff sec8 gate 9 is unsatisfiable together with its own header
line 9** (finding 1).

**Dev-3 - ACCEPTED, same disposition as T2-A gate 5 / T2-B D-E / T3-A D5 / T3-B D5.**
`commit_scoped.py` has no stage-only mode, so `--staged` cannot interpose between
staging and committing in one command. The implementer ran the **strictly
stronger** `encoding_guard.py --all` before committing and the literal `--staged`
form after. **I re-ran both independently: `--all` silent, `--staged` silent, exit
0 each.** Substitution cost is zero. Attributed to configuration.

### Undeclared deviations found by this reviewer

**None.** Every divergence from upstream text that surfaced during the audit
traces to a handoff, plan or ledger sentence and is recorded as a finding below
(1, 2, 3, 7). **No undisclosed implementer choice was found anywhere in the
three commits.** The commit split (Dev-2), the declared records (Dev-1), and the
encoding-gate ordering (Dev-3) were all disclosed before I looked.

---

## 15. Findings

**0 blocking / 7 non-blocking.**

1. **NON-BLOCKING, planner.** Handoff sec8 gate 9 is **internally
   unsatisfiable** alongside the handoff's own header line 9: the header requires
   the journal to record its own unit's commit SHAs, but gate 9 names a single
   commit that would contain the journal - a journal cannot contain the SHA of a
   commit that includes it. The implementer split into three scoped commits, which
   is the correct resolution and preserves the gate-9 message text verbatim on the
   code commit. **Action:** fix `handoff-spec`'s gates template to either drop the
   self-reference or state the three-commit split, so the next unit does not have
   to rediscover it. Credit to the implementer for reporting the tension rather
   than silently landing one commit and omitting its own SHA.
2. **NON-BLOCKING, planner (standing, reconfirmed).** The ledger's crate-scoped row
   `-p ariadne-cli = 86/0/1/3` records the **wrong suite count**; the true value is
   **5**. Independently reconfirmed in this reviewer's workspace capture at lines
   1979-1980, which show `Running unittests src\main.rs (ariadne-...)` and
   `Running unittests src\bin\tool.rs (tool-...)` as two separate targets
   (`crates/cli/Cargo.toml` declares two `[[bin]]` targets). This is the same
   arithmetic error T3-B reported and it is **still unfixed**, so it is inherited by
   every future unit scoped to that crate. **Action:** correct the ledger row at
   M23.5 closeout. *(Contrast: this unit's own arithmetic is CORRECT - the handoff's
   `34` for the integration crate decomposes as 1 lib + 32 targets + 1 doc-tests,
   which I counted by hand from the capture, and its `3/2/5/6` per-file test counts
   match the tree at every one of the eight attribute line numbers.)*
3. **NON-BLOCKING, planner.** The ledger's M23.5 row is **three units stale** and
   `tasks/milestone_ledger.md` is designated the source of truth by `AGENTS.md` and
   the `milestone-lifecycle` skill. It records "Phase 3 ... AUTHORIZED ... **not
   started**", `HEAD b950963`, and a baseline chain ending `1169/0/4/88` at
   `b950963` - but T3-A and T3-B have landed and been APPROVED, T3-C has landed
   (this review), actual HEAD is `6c0c2b6`, and the reviewer-measured workspace
   line is `1176/0/4/88` (measured identically by T3-B's reviewer). The
   implementer **could not** fix this: `tasks/milestone_ledger.md` is outside the
   sec2 touch-set, so this is a planner/orchestrator obligation, **not an
   implementer defect**. **Action:** update the row's Phase 3 status, HEAD, and
   baseline chain before dispatching T3-D, and add the crate-scoped row
   `-p ariadne-integration-tests = 139/0/0/34` that T3-C's own measurement now
   supplies and that the row currently lacks entirely.
4. **NON-BLOCKING, implementer (documentation precision, no code impact).** The
   golden no-op argument is correct, but it is stated from `IncludeReport`'s shape
   ("there is no field that serialises a concrete cycle list") without explicitly
   covering `includes_manifest.json`, which is a different artefact. The gap is
   closed by the grep (four hits, all `"cycle_count": 0`) plus the type-level
   argument (section 8), and the ledger row repeats the same
   `includes_report.json`-only phrasing. **Action:** when this reasoning is reused,
   state that both `includes_report.json` and `includes_manifest.json` are
   projections of the same count-only type. Not a correctness issue - the
   determination is sound and stable, because both fixtures are acyclic so 0 is
   representation-invariant.
5. **NON-BLOCKING, planner (sequencing, for T3-D's reviewer to action).** At this
   HEAD `cycle_count` is **still a DFS-path count**: `graph.rs:356` still calls
   `detect_cycles()` and `:369` still sets `cycle_count: detected_cycles.len()`.
   So IV-9's **first** clause ("`cycle_count` naming survives ... now carrying
   group count") is not yet observable, and no test in the tree yet witnesses
   `cycle_count` carrying a group count. This unit was explicitly forbidden by
   handoff sec1/IV-9 from touching report code, so it is **contract compliance,
   not a defect** - it is inherent to the 5-unit split (plan line 108). **Action:
   unit D's reviewer must re-run `m3_include_graph_test` after the port and treat
   that green-without-edit run as the real milestone-level IV-9 gate**; also re-pin
   the four absolute `assert_eq!(report.cycle_count, 1, ...)` at `graph.rs:617`,
   `:631`, `:640`, `:647`, which are the arithmetic that does change meaning.
6. **NON-BLOCKING, configuration (standing, F10).** `python tools/ui_smoke.py` and
   `python tools/bench_compare.py` each attempted once, directly, each refused with
   identical verbatim catch-all text (13.1). Neither routed through `distill.py`.
   **Impact for THIS unit is unchanged from T3-B** - it makes no UI change and pins
   no timing - but the standing gap remains: at this HEAD the UI serves group
   currency in its payload (T3-A/T3-B's work) with **no runtime evidence** that it
   cold-boots, routes, or serves that payload, and the `Cache-Control: no-store`
   route contract is unverified for it. **Action:** close F10 before T4.1; it is an
   operator/allowlist action, not implementer work.
7. **NON-BLOCKING, configuration (standing, F9).**
   `python tools/testrun.py --baseline` -> `usage error: missing value for
   --baseline`, reproduced verbatim. Attempted once, recorded once, NOT treated as
   a gate failure, NOT worked around. The full-workspace invocation it wraps was run
   and compared by hand (13.2).

---

## 16. VERDICT

# APPROVED

**0 blocking / 7 non-blocking.**

Range `91967c9..6c0c2b6`, three commits, every one inside the declared touch-set
plus the orchestrator-declared records; the forbidden F12 fixture in none of them;
`crates/**`, `tools/**`, `tests/golden/**`, `tests/fixtures/**` and every other
`tests/integration/**` path absent from all three. All frozen locks hold, including
the three the next unit's atomic delete depends on (`detect_cycles` at
`graph.rs:234` and `dfs_cycle` at `graph.rs:260` both still EXIST and are still
CALLED from `graph.rs:356`, `graph.rs:534` and `closure_equivalence_tests.rs:515`),
IV-9's `cycle_count` name, and IV-11's `informational_cycle_count` name with its
skip-when-zero serde predicate byte-intact.

**The three headline judgements, independently:**

- **IV-9 uneditedness: GENUINE.** `m3_include_graph_test.rs` is byte-identical at
  this HEAD to its state at the unit's base - proved by **identical blob SHA-1**
  (`b6a54d5...`) at `91967c9` and `HEAD`, plus an empty path diff, plus absence
  from every per-commit stat list. Gate 1 is green and its capture **names the
  exact test carrying `report.cycle_count > 0` as PASSING**, with `0 filtered out`.
  The counterfactual confirms the assertion is load-bearing: `>= 0` would make it a
  tautology, and a direct `detect_cycle_groups()` call would bypass the field NAME,
  which is the entire IV-9 contract. Section 5 records the one honest limit: the
  *semantic shift* half of IV-9 only becomes falsifiable at unit D (finding 5).
- **The Scala migrations: CORRECT, with zero weakening.** The entire range is
  **four hunks** at zero context - two calls, two comparison-plus-message pairs.
  `m20:160-189` and `m22:447-453` are not even CONTEXT lines; they are untouched
  bytes. The only truth-condition change is `len() >= 1` -> `!is_empty()`, which is
  the *same predicate* in both files, so nothing was made easier to satisfy.
  m22's all-tolerate equality still means the same thing at group granularity
  because the delegate is a **constant** `TolerateWithinPackage` - "every member
  non-`Reject`" holds by construction identically in both representations, so the
  assertion keeps asserting *the classifier drops and duplicates nothing*, and the
  all-`Reject` counterweight at `m20:171-179` is likewise untouched. **`crates/**`
  is empty across the range: zero plugin-side change, as IV-12 requires.** Gates 3
  and 4 are **load-bearing** - they prove at runtime that `detect_cycle_groups()`
  returns non-empty on two distinct lanes, which is what makes "zero arithmetic
  re-pin needed" a result rather than a vacuity.
- **The golden determination: CORRECT and STABLE.** Exactly four `cycle_count`
  occurrences across `tests/golden/**`, all `"cycle_count": 0`; golden blob SHA-1
  identical at base and HEAD; `UPDATE_GOLDENS=1` provably never run; no file added
  or changed under `tests/fixtures/basic` or `tests/fixtures/expensive`. I closed
  the sharp version of the brief's question - a golden **cannot** hide a cycle list
  under another key, because `IncludeReport` (`types.rs:47-64`) has no cycle-list
  field at all and `CycleGroup` is unreachable from it - and both fixtures are
  acyclic, so `0` is representation-**invariant**, not merely equal today.

**Dev-1, Dev-2, Dev-3 all ACCEPTED** (14), with Dev-1 attributed to the
orchestrator's declaration, Dev-2 attributed to an unsatisfiable contract (the
split was required), and Dev-3 to configuration. **No undeclared deviation was
found.**

**Declining to fabricate a Red was the correct call** (10). Every premise of "no
genuine failing-first moment" verified independently, including the plan's own line
108 forbidding the delete-from-elsewhere that would have forced one. The substitute
evidence is stronger than the implementer claims: the migration is **positively
witnessed** (both new call paths executed and returned non-empty), not merely
unfalsified.

**Item 4 was correctly NOT done here** (11): the conditional was genuinely false -
`closure_equivalence_tests.rs` exists at exactly one path in the whole worktree,
`crates/graph/src/`, which is outside the touch-set and forbidden by name in sec13;
the call site at `:515` is untouched and the file is in no commit. The correct
action was inaction.

**Honest limits of this review, stated rather than buried:** the temporal ordering
of the implementer's BEFORE gate-5 measurement is the one element taken on report
(sections 9, 13.5) - though the inventory delta is proven structurally 0, so no
behaviour claim rests on it. And `ui_smoke.py` / `bench_compare.py` could not be
run at all (finding 6), so there is still **no runtime UI evidence and no verified
perf evidence** for this milestone at this HEAD.

### Scoped canonical lines, reviewer-measured, verbatim

| Gate | Line |
|---|---|
| 1 (IV-9) | `3 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `2 passed / 0 failed / 0 ignored / 1 suites` |
| 3 | `5 passed / 0 failed / 0 ignored / 1 suites` |
| 4 | `6 passed / 0 failed / 0 ignored / 1 suites` |
| 5 (crate) | `139 passed / 0 failed / 0 ignored / 34 suites` |
| **workspace** | **`1176 passed / 0 failed / 4 ignored / 88 suites`** |

Workspace delta vs the ledger baseline `1176 / 0 / 4 / 88`: **+0 / +0 / +0 / +0.**
Tool suites: `Ran 184 tests in 18.152s` / `OK`.
Hygiene: zero R1/R2. Encoding: `--all` and `--staged` both silent, exit 0.

### Gates that could not be run

`testrun.py --baseline` (broken, F9); `ui_smoke.py` (REFUSED, F10);
`bench_compare.py` (REFUSED, F10). Each attempted once, verbatim recorded in 13.1,
none retried, none wrapped.

---