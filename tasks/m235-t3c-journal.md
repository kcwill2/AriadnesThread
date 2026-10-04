# M23.5 T3-C Journal - Scala Integration Consumers + IV-9 Acceptance Gate (tests/integration)

**Handoff:** `tasks/m235-t3c-handoff.md` (Phase 3 unit C, plan T3.1 consumer
migration IV-12 + explicit IV-9 acceptance gate).
**Base:** `91967c9` (T3-B tip), branch `tarjan-s-scc`, worktree
`.kilo/worktrees/tarjan-s-scc`.
**Commits made by this unit (oldest first):**
- `b774362` `test(integration): migrate Scala cycle consumers to cycle groups`
  (scopes: the two touch-set test files only)
- `d11a819` `docs(tasks): land M23.5 T3-B reviewer records (verdict, execution
  results)` (orchestrator-declared records, see deviation Dev-1)
- the journal commit itself lands after this file is written, so its own SHA
  cannot appear in its body; it is the last `docs(tasks)` commit on
  `tarjan-s-scc` before review.

---

## Checkpoint 0 - preflight

`python tools/checkpoint_guard.py status` at session start:
HEAD `91967c9`, branch `tarjan-s-scc`, DIRTY FILES (numstat): `(none)`,
UNTRACKED: only `tasks/m235-t3b-verdict.md` and
`tasks/m235-t3b-execution-results.md` (the reviewer records the orchestrator
declared into this unit's commit scope).

The unpinned F12 fixture
(`tests/fixtures/basic/artifacts/includes/report.json`) was NOT content-dirty at
preflight (numstat empty), so the Read/Write restore was never needed and no
restore was attempted.

Line-reference verification against the tree before any edit:
- `m20_scala_dependency_graph_test.rs:154` reads `let cycles =
  graph.detect_cycles();` inside
  `fixture_cycle_is_informational_under_tolerate_policy` - matches handoff S1.
- `m22_scala_load_analysis_test.rs:442` reads the same call inside
  `auto_shaped_graph_keeps_the_cycle_informational` - matches handoff S2.
- `detect_cycle_groups` is public (`crates/graph/src/cycle_groups.rs:39`,
  re-exported by `crates/graph/src/lib.rs:15 pub use cycle_groups::*`).
- `detect_cycles` (`graph.rs:234`) and `dfs_cycle` (`graph.rs:260`) exist and
  were NOT touched - their deletion is unit D's atomic job.
- The only `detect_cycles` call sites under `tests/` at base are exactly S1 and
  S2 (workspace grep).

## Checkpoint 1 - Red: none, stated plainly, not fabricated

Per handoff sec 4 (pre-authorised disclosure): **no Red for T3-C.**
Both replacements already exist, are public, and are already exercised by
`crates/graph/tests/cycle_groups_test.rs` (9 tests) and
`cycle_groups_equivalence_test.rs` (4 tests), both reviewer-verified APPROVED
at `b950963`.
A call-site migration has no failing-first moment because the old call still
compiles and passes until unit D deletes it.
No Red was manufactured: no fixture was broken, no gate weakened, no assertion
edited to fail, `detect_cycles` was not deleted to force a compile error.

**Red-equivalent evidence** = the gate-5 before/after pair below.
Measured BEFORE the first edit:
`139 passed / 0 failed / 0 ignored / 34 suites`
(exit 0, capture `target/testrun-capture.txt`, 709 lines).
Measured AFTER the edit (gate 5):
`139 passed / 0 failed / 0 ignored / 34 suites`
(exit 0, capture 698 lines).
IDENTICAL pair - this unit adds and removes no test; zero delta, zero residual.

## Checkpoint 2 - Green: the two call sites

S1 `m20_scala_dependency_graph_test.rs` (inside
`fixture_cycle_is_informational_under_tolerate_policy`):

```rust
let cycle_groups = graph.detect_cycle_groups();
assert!(
    !cycle_groups.is_empty(),
    "the deliberate models <-> services fixture cycle is detected as one entangled group"
);
```

S2 `m22_scala_load_analysis_test.rs` (inside
`auto_shaped_graph_keeps_the_cycle_informational`):

```rust
let cycle_groups = graph.detect_cycle_groups();
assert!(
    !cycle_groups.is_empty(),
    "the deliberate fixture cycle is present on the auto-shaped lane as a group"
);
```

**The `len() >= 1` -> `!is_empty()` reasoning (reviewer audit 3).**
The old S1 assertion was `cycles.len() >= 1` over DFS-reported *paths*; the new
one is `!cycle_groups.is_empty()` over entangled *groups*.
The count changed MEANING (paths -> groups), not value: under group semantics
the same fixture yields one group where the path enumeration reported a
path count, so comparing raw numbers across the two representations is
meaningless.
The stance each test actually asserts is *non-emptiness* - "a deliberate cycle
exists" - and non-emptiness is the predicate that survives the representation
change: a fixture has a cycle iff it has at least one nontrivial SCC (IV-2),
so ">= 1 path" and ">= 1 group" carry the same truth value on every fixture.
`is_empty()` therefore expresses the preserved stance directly instead of
comparing a quantity whose meaning shifted.

**Assertion preservation (IV-11).**
Under the group classification rule ("a group is informational iff every
member file's policy is non-`Reject`"), every assertion outside the two call
sites survives VERBATIM and none was touched:
- m20 all-tolerate: `informational_cycle_count >= 1` and
  `informational_cycle_count == cycle_count` - every group's every member
  tolerates, so every group is informational; the equality now reads *groups*,
  which is exactly the IV-11 re-pin of meaning without re-pinning any number.
- m20 all-`Reject`: `informational_cycle_count == 0` and
  `blocking.cycle_count == tolerated.cycle_count` (both now the group count).
- m20 serde skip-when-zero shape: zero-count JSON lacks
  `informational_cycle_count`, non-zero JSON contains it - the byte-stability
  lock, untouched.
- m22: same all-tolerate equality plus `informational_cycle_count > 0`.
The committed diff proves it: `git diff b774362^..b774362` is 3 added / 3
removed lines per file - the call, the comparison, and the local message.
Nothing in m20 `:160-189` or m22 `:447-453` appears on a `+`/`-` line.

## Checkpoint 3 - gates, verbatim (all implementer-run)

| Gate | Command | Verbatim line |
|---|---|---|
| 1 (IV-9) | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m3_include_graph_test` | `3 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m00_golden_baseline_test` | `2 passed / 0 failed / 0 ignored / 1 suites` |
| 3 | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m20_scala_dependency_graph_test` | `5 passed / 0 failed / 0 ignored / 1 suites` |
| 4 | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m22_scala_load_analysis_test` | `6 passed / 0 failed / 0 ignored / 1 suites` |
| 5 before | `python tools/testrun.py --distill --timeout 3600 -p ariadne-integration-tests` | `139 passed / 0 failed / 0 ignored / 34 suites` |
| 5 after | same command, post-edit | `139 passed / 0 failed / 0 ignored / 34 suites` |
| 6 | `python tools/hygiene_check.py` | zero R1/R2; every printed line an R3 warn in files this unit does not touch |
| 7 | `python tools/encoding_guard.py --all` (pre-commit) and `--staged` (post-staging, per Dev-3) | no output, exit 0 |

All four scoped gates match the handoff's expected lines verbatim, including
the suite arithmetic (each of the four files is one test target = 1 suite;
m20 carries 5 `#[test]` fns, m22 5 `#[test]` + 1 `#[tokio::test]`, matching
sec 5's declared counts).
Gate 5's suite count (34) was measured, not predicted - the handoff explicitly
refused to pre-pin it.
Gate 5's 139 passed equals the 139 `#[test]`/`#[tokio::test]` fns the handoff
counted at `b950963`; the before/after identity is the no-delta proof.

**IV-9 gate, "passes WITHOUT EDIT" proof.**
`m3_include_graph_test.rs` is NOT in this unit's touch-set, was never opened
for edit, and does not appear in `git status` modified/staged lists at any
point (verified via the read-only git status tool after the heaviest run, and
via the numstat section of `checkpoint_guard status` after both full-crate
runs: only the two migrated files ever appear).
Gate 1 runs its `report.cycle_count > 0` assertion (test file `:118-121`)
against the unedited committed blob and passes: `3/0/0/1`.
The path appears in no commit of this unit.

## Checkpoint 4 - golden determination: provably unnecessary, zero bytes touched

Handoff sec 7 claims the goldens are cycle-free; independently verified against
the tree before deciding anything:
- `tests/golden/basic/includes_report.json:2` and
  `tests/golden/expensive/includes_report.json:2` read `"cycle_count": 0,`.
- `tests/golden/basic/includes_manifest.json:3` and
  `tests/golden/expensive/includes_manifest.json:3` read `"cycle_count": 0,`.
- No golden anywhere serialises a concrete cycle list (grep over
  `tests/golden/**` for `cycle_count` returns exactly those four hits;
  `IncludeReport` carries counts, not paths).
Both golden fixtures are acyclic, so 0 groups == 0 paths and every byte is
representation-invariant.
**Therefore: NO golden was re-pinned, none needed re-pinning, and
`UPDATE_GOLDENS=1` was never run.**
`tests/golden/**` and `tests/fixtures/**` appear in no commit; gate 2 green
with the committed goldens byte-unchanged is the proof.
No files were added to `tests/fixtures/basic` or `tests/fixtures/expensive`.

## Checkpoint 5 - fixture and lock state after the work

`python tools/checkpoint_guard.py status` after the post-edit full-crate run:
numstat DIRTY lists ONLY the two migrated test files;
`tests/fixtures/basic/artifacts/includes/report.json` shows as
porcelain-modified but is ABSENT from the numstat content comparison - the
known F12 stat-entry mechanism (fifth identical reproduction), content
byte-equal, no restore needed or attempted, path absent from every commit.
Locks re-checked at HEAD-after-commits: `detect_cycles` (`graph.rs:234`) and
`dfs_cycle` (`graph.rs:260`) still exist AND are still called by
`graph.rs:356`/`:534`/`closure_equivalence_tests.rs:515` - untouched, as unit D
owns them.

## Scope discipline and handoff-placement notes

- Touched exactly the two touch-set test files plus this journal.
  No crate, tool, plan, fixture, golden, or `.gitattributes` file was edited.
- **Item 4 of the dispatch (`closure_equivalence_tests.rs:515`): NOT placed in
  this unit by the handoff.** That file lives at `crates/graph/src/`
  (not under `tests/integration`), is absent from the sec2 touch-set, and
  sec13 forbids touching `crates/graph/**`.
  Unit D owns it, atomic with the `detect_cycles` deletion, exactly as
  `tasks/m235-t3b-verdict.md` sec 5.6/7 records.
  Per the dispatch's own conditional ("If the handoff places that repoint in
  your unit"), the condition is false, so no repoint was made; the
  IV-2-equivalence licence the reviewer established is unit D's to spend.
- IV-12: pure consumer migration; no file under `crates/lang-scala/**` was
  opened, consistent with both Scala tests being graph-API consumers only.

## Deviations (each with reason)

- **Dev-1: `d11a819` carries `tasks/m235-t3b-verdict.md` and
  `tasks/m235-t3b-execution-results.md`, which are outside the handoff sec2
  touch-set.** Reason: the orchestrator task message explicitly declared these
  untracked reviewer records APPROVED into this unit's commit scope (reviewers
  cannot commit; the adjudication record must become durable).
  Precedent: `9f30c9a` did exactly this in unit B.
  No other out-of-touch-set path was staged.
- **Dev-2: handoff sec8 gate 9 lists ONE commit; this unit landed THREE.**
  Reason 1: the journal must record its unit's commit SHAs (handoff header line
  9), which requires the code/record commits to exist before the journal is
  written - the same self-reference split T3-B adjudicated as REQUIRED (its
  D6).
  Reason 2: Dev-1's records are a different logical change from the test
  migration.
  Every commit carried an explicit `--scope`; gate 9's exact message text is
  preserved verbatim on the code commit `b774362`.
- **Dev-3: gate 7's `--staged` form necessarily runs after staging+committing,
  so `encoding_guard.py --all` (the strictly stronger form) was run BEFORE the
  first commit as well.** Same disposition as T2-A gate 5 / T2-B D-E / T3-A D5
  / T3-B D5: `commit_scoped.py` has no stage-only mode.
  All three encoding runs (`--all` pre-commit, `--staged` after each of the
  two landed commits) produced no output, exit 0.
- **Refusal record (discipline clause, not a skipped gate):** raw
  `git status --porcelain` was refused with the verbatim catch-all text
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  The identical information was obtained via the dedicated read-only git
  status tool; no acceptance gate was narrowed as a result.
  No other command in this unit was refused; every gate 1-9 above ran.

## Context / next-unit handoff

After this unit, every `detect_cycles` caller OUTSIDE `crates/graph` is
migrated: the remaining call sites are `graph.rs:356`, `graph.rs:534` (own
unit test), and `closure_equivalence_tests.rs:515` - all inside unit D's
declared scope, so unit D's atomic delete-gate precondition is satisfied from
the `tests/integration` side.
