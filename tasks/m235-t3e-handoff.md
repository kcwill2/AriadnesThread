# M23.5 T3-E Handoff - Retire the Path-Based Include-Element DTO (crate:ui)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 3 of 4, unit E of **6** (A, B, C, D, D2, E) -- **the last Phase 3
unit**. Completes D1-B for the UI cycle currency.
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 3
(T3.2), sec1.3, sec3 item 1
**Prerequisites (all THREE must be APPROVED and landed):** unit A
`tasks/m235-t3a-handoff.md`, unit B `tasks/m235-t3b-handoff.md`, and unit D2
`tasks/m235-t3d2-handoff.md` (verdict `tasks/m235-t3d2-verdict.md`; commits
`42bb44e` + `4d0f245` + `b0a91d2`). Unit D (`crates/graph`) is independent of
this unit -- see sec12.
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`
**Journal:** `tasks/m235-t3e-journal.md` (you create it; record your own SHAs in it)

**AMENDED 2026-10-04, before any implementer was dispatched.** Unit D2 landed
between this handoff's first draft and now, so every count, caller enumeration
and line reference below was re-derived from the tree at `b0a91d2`. What
changed, and why: F1a-F1d of `tasks/m235-t3d2-verdict.md` (four stale counts and
citations, itemised in sec13.1); F2 of the same verdict (unit D2's parity test
P1 becomes a TAUTOLOGY after this unit's rename, so sec5.0-5.3 replace the old
"you add and remove no test" pin with the ONE sanctioned conversion); and
sec13.2-13.4, three further defects found while re-deriving that the unit's
reviewer had not caught -- a required `from_paths` deletion whose file was
outside the touch-set (which made gate 3's zero-warning build unreachable), four
under-specified import lists, and a doc sentence that becomes
self-referential after the rename. Nothing else in this contract changed.

Every line reference was re-read out of the tree at **`b0a91d2`** (this handoff
was first written against `b950963`, before unit D2 landed). **If a line
reference disagrees with the file you read, the FILE wins** -- report the drift
and do not edit around it.

---

## 0. PREFLIGHT

**Step 0.1** - `python tools/checkpoint_guard.py status`; record HEAD, branch,
dirty files, untracked files.

**Step 0.2** - the unpinned CRLF fixture. `tests/fixtures/basic/artifacts/
includes/report.json` has no `.gitattributes` pin and this worktree has
`core.autocrlf=true`, so a full test run re-dirties its **stat entry** even when
content is byte-identical (ledger deferred row **M23.5 F12**; measured **seven**
times: T2-A, T2-B, T3-A, T3-B, T3-C, T3-D, T3-D2 -- the pre-amendment text said
"three", which was stale). If and only if `git diff --numstat HEAD -- <path>`
shows rows: restore
with the Kilo `Read` tool then the Kilo `Write` tool. NEVER `Set-Content`,
`Out-File`, `>`, `>>`, `Get-Content | -replace`. `git checkout --` / `git restore`
are mutating git and are DENIED. Every `commit_scoped.py` carries an EXPLICIT
`--scope`, and that path must appear in NO commit.

**Step 0.3 - the ordering precondition, with the full census.** Grep for
`build_include_elements(` -- note the OPEN PAREN, which is what separates a call
from a prose mention -- and you must find **zero** call sites outside
`crates/ui/**`. If any real call remains in `crates/cli/**` (including
`crates/cli/tests/payload_tail_bench.rs`), **STOP and report**: unit B has not
landed (or missed one of its four sites) and deleting the old entry point now
would break the workspace build.

Three non-code hits legitimately contain that string. They are **not** call
sites: do not stop for them, and do not edit them (all are outside your
touch-set):

1. `crates/cli/tests/payload_tail_bench.rs:389` -- the bench row LABEL
   `("build_include_elements", include_elements_ms)`, a string literal whose
   timed call is `build_include_elements_from_groups` at `:325` (T3-B
   finding 4). **This unit makes that label accidentally CORRECT**, because the
   surviving entry point is then named `build_include_elements`. Still
   `crates/cli/**`: out of scope, report-only, do not touch it.
2. `crates/cli/tests/payload_tail_bench.rs:41` and
   `crates/cli/src/commands/ui.rs:103` -- prose in doc/`//` comments.
3. `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md:69` -- plan text
   describing the 5-arg shape. `.kilo/plans/**` is not yours.

The expected caller census is **NINE**, not seven: the seven pre-existing sites
listed in sec1 plus the two unit-D2 sites `export_order.rs:310` (P2) and
`export_order.rs:381` (P1). If you find a **tenth** real call, STOP and report
rather than widening scope.

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

### The NINE in-crate call sites, re-verified at `b0a91d2`

All nine are in `crates/ui/tests/ui_test/` (one test target; `main.rs` declares
19 modules, `mod export_order;` at `main.rs:5`). Sites 1-4, 7-9 are the
pre-existing set; sites 5 and 6 are the two unit D2 added at `42bb44e`.

| # | File:line (the CALL) | Third argument | Note |
|---|---|---|---|
| 1 | `dto_builders.rs:15` | `&cycles`, bound by `let cycles = vec![];` at `dto_builders.rs:13` | the only site with a named binding |
| 2 | `graph_api.rs:136` | `&[]` (at `:139`) | 5-arg, fully qualified `HashMap::new()` maps |
| 3 | `export_order.rs:91` | `&[]` (at `:94`) | test body opens at `:88`; the CALL is `:91` |
| 4 | `export_order.rs:108` | `&[]` (at `:111`) | test body opens at `:105`; the CALL is `:108` |
| 5 | `export_order.rs:310` | `&[]` (at `:313`) | **unit D2, P2** `the_parity_comparison_detects_a_cycle_classification_difference` (body from `:297`) |
| 6 | `export_order.rs:381` | `&[]` (same line) | **unit D2, P1** -- **consumed by the sec5 conversion**, which replaces this call with ONE call to the group builder |
| 7 | `d27_node_provenance.rs:141` | `&[]` | one-line call, the longest shape |
| 8 | `d27_node_provenance.rs:152` | `&[]` (at `:155`) | |
| 9 | `common.rs:157` | `&[]` (at `:160`) | `payload.include_graph = build_include_elements(...)` |

The pre-T3-D2 text of this handoff cited `dto_builders.rs:14`,
`export_order.rs:88/:105` and `d27_node_provenance.rs:128/:139`. Those were
`#[test]`/`let` lines, not calls: site 1's binding is `:13`, sites 3/4 were
citing their `#[test]` lines, and sites 7/8 were 13 lines stale. **Use this
table.**

**Every one of the nine passes an EMPTY currency** -- a bare `&[]` at eight
sites, `&cycles` from `vec![]` at site 1 -- so the element type is inferred from
the parameter at all nine and **no type annotation changes anywhere**: only the
callee name. (The old text said "the four sites"; the true figure is nine of
nine.) Site 1's `let cycles = vec![];` binding may be renamed to say
`cycle_groups` where that reads better; that is a one-token local rename inside
an in-set file, and nothing else in `dto_builders.rs` may move.

**Four import lists carry BOTH names and must collapse to one.** The old sec3.4
said only "any import list that named the old function is updated", which
under-specifies the de-duplication and leaves a duplicate-import compile error:

| File:line | Today | After this unit |
|---|---|---|
| `export_order.rs:15` | `{build_include_elements, build_include_elements_from_groups, GraphExportPayload, ...}` | drop `build_include_elements_from_groups` |
| `dto_builders.rs:4` | `{build_call_elements, build_include_elements, build_include_elements_from_groups, ...}` | drop `build_include_elements_from_groups` |
| `graph_api.rs:4` | same two-builder shape as `dto_builders.rs:4` | drop `build_include_elements_from_groups` |
| `d27_node_provenance.rs:9` | `{build_include_elements, build_include_elements_from_groups, build_symbol_elements}` | drop `build_include_elements_from_groups` |
| `common.rs:3` | `{build_call_elements, build_include_elements, GraphExportPayload, ...}` | **no edit** -- it names only the survivor |

### Invariants this unit preserves

- **IV-9 / D27 / determinism:** unchanged. This is a rename plus a deletion; the
  body, the 5-arg shape, the canonical `(path asc, file_id asc)` emission, the
  D27 OMITS rule and the node-data keys all carry over untouched.
- **Zero new hex literals:** you touch no CSS, no JS, no template. Semantic
  verdict colors unchanged across light, dark and dark-steampunk by construction.
- **IV-11 / IV-12:** untouched. No report code, no Scala plugin code.
- **Exactly one cycle entry point in `crates/ui`** after this unit, and
  **exactly one cycle-currency constructor**: `IncludeCycleClass::from_paths`
  goes with it (sec3.1). Leaving a second currency constructor in the tree is
  the D1-B dual representation this milestone exists to remove, so "delete the
  function but keep the view" is not an option.

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

- `crates/ui/src/dto.rs` (delete the legacy entry point; rename the survivor)
- `crates/ui/src/cycle_groups_view.rs` (**newly in-set, and only for the two
  bounded edits in sec3.1** -- delete `IncludeCycleClass::from_paths` at
  `:32-53` with its doc comment at `:26-31`, and amend the module doc at
  `:1-9`, which names it at `:4`. Without this file in the touch-set the unit
  cannot reach gate 3's zero-warning build at all; see sec13.2)
- `crates/ui/tests/ui_test/dto_builders.rs` (site 1; import list `:4`)
- `crates/ui/tests/ui_test/graph_api.rs` (site 2; import list `:4`)
- `crates/ui/tests/ui_test/export_order.rs` (sites 3, 4, 5, the P1 conversion in
  sec5, and the resulting single group-builder call; import list `:15`)
- `crates/ui/tests/ui_test/d27_node_provenance.rs` (sites 7, 8; import list `:9`)
- `crates/ui/tests/ui_test/common.rs` (site 9)
- `tasks/m235-t3e-journal.md` (**new**, yours)

That is **eight paths**, all in one crate plus your journal.

You MUST NOT touch: `crates/ui/src/lib.rs` (`pub use dto::*;` at **`:6`** -- the
renamed function is exported automatically; note the pre-amendment text said
`:5`, which was stale), any OTHER file under `crates/ui/src/` (notably
`cycle_groups_view.rs` outside the two edits sec3.1 names, and `export.rs`,
`server.rs`), any other file under `crates/ui/tests/ui_test/`, any file under
`crates/ui/assets/`, `crates/cli/**`,
`crates/graph/**`, `crates/report/**`, `tests/**`, `tools/**`,
`.kilo/plans/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`.

No `git stash`, no branch change, no `git add .`.

---

## 3. Green - the minimal implementation bound

Do these in this order. The order is load-bearing in two places: 5.4's capture
must happen while both builders still exist, and 3.5 must precede the final gate
runs because it is the ONE sanctioned test change.

### 3.1 Retire the path-based currency (`crates/ui/src/cycle_groups_view.rs`)

`IncludeCycleClass::from_paths` (`cycle_groups_view.rs:32-53`, doc comment
`:26-31`) has exactly ONE caller in the whole tree:
`crates/ui/src/dto.rs:618`, inside the function 3.2 deletes. A repo-wide grep
for `from_paths` returns that call, the definition, the module doc at `:4`, and
`tasks/**` records only -- no test, no other crate.

**Delete it in this unit, or gate 3 cannot pass.** `cycle_groups_view` is a
`pub(crate)` module (`crates/ui/src/lib.rs:1`), so the moment its last caller is
gone `from_paths` is unreachable code and `cargo build -p ariadne-ui` emits a
`dead_code` warning. Gate 3 is pinned at `0 warnings emitted`. Before this
amendment `cycle_groups_view.rs` was outside the touch-set, so the unit as
written was REQUIRED to finish with a warning: it would have deleted the caller
while forbidding the deletion of the callee's only user, and it would have left
a second cycle-currency constructor in the tree -- the D1-B dual representation
the milestone exists to remove. Keeping it behind `#[allow(dead_code)]` is not
an alternative; that is the same dual representation with a silencing
attribute.

Also amend the module doc at `cycle_groups_view.rs:1-9`, which names
`from_paths` at `:4` and says "retired in T3-E". A doc comment naming a deleted
function is exactly the documentation defect recorded in the ledger row for
`build_include_elements_legacy_paths`. Keep `:1-3` (module purpose) and `:5-9`
(the shared-tail rationale, which is still true) and restate `:4` so it
describes the single surviving currency.

**Everything else in that file is frozen:** `from_groups` (`:63-88`), the four
query methods `node_is_cycle`/`edge_is_cycle`/`group_size`/`witness_labels`
(`:91-110`), `group_index_by_member` (`:115-123`), `endpoints_share_group`
(`:127-136`), `witness_path_labels` (`:142-149`). Do not reformat, reorder or
"tidy" any of them.

### 3.2 Delete the legacy entry point (`crates/ui/src/dto.rs`)

Delete `build_include_elements` (`dto.rs:608-622`) **and its legacy doc
paragraph** (`:604-607`, "Legacy path-based entry point (pre-M23.5) ... New
callers must use `build_include_elements_from_groups`"). Keep the D27 paragraph
at `dto.rs:599-602` -- it is the D27 OMITS contract for the entry point, not
for the retired one -- and move it onto the surviving function's doc comment,
which already states the OMITS rule for the two group keys at `:635-636`.

**There is no legacy-only helper left to hunt, and that is a fact, not a
search.** Both entry points call the ONE private tail `emit_include_elements`
(`dto.rs:659`, called from `:615` and `:644`), and `from_paths` (3.1) was the
only legacy-exclusive code in the crate. So the production deletion is 19 lines
in `dto.rs` and 28 in `cycle_groups_view.rs`. **If your diff shows any other
deletion in either file, STOP and report it** -- that would mean the two entry
points were not the shape this handoff re-derived.

### 3.3 Rename the survivor, and fix the doc that renames into nonsense

Rename `build_include_elements_from_groups` (`dto.rs:637`) to
`build_include_elements`; its third parameter becomes
`cycle_groups: &[ariadne_graph::CycleGroup]` (`:640`). Three doc obligations:

- **`dto.rs:625-626` is MANDATORY to fix.** It reads "the group-based sibling of
  `build_include_elements`", which after the rename names the function ITSELF
  as its own sibling. Replace it with wording stating that this is the only
  include-element entry point in `crates/ui`, the path-based one having been
  retired in T3-E. Leaving that sentence is a finding: it is the same
  self-referential class of defect as 3.1's module doc.
- Keep `:627-636` (the IV-2/IV-3/IV-5 semantics and the D27 OMITS rule for the
  group keys) and add the D27 paragraph moved in from 3.2.
- Add the short history note this handoff has always required: the third
  parameter was `&[Vec<FileId>]` (reported DFS path enumeration) before M23.5
  and is now `&[CycleGroup]` (the SCC partition), so the next reader does not
  "restore" the old type.

### 3.4 Rename the call sites -- NINE textual sites, callee name only

Apply the sec1 table. **Eight** legacy sites change only the callee name (sites
1, 2, 3, 4, 5, 7, 8, 9); the ninth is the single call P1 keeps after the sec5
conversion, which at conversion time names `build_include_elements_from_groups`
and is renamed by this step like any other. No type annotation changes (sec1).
Then collapse the four two-name import lists per the sec1 table.

`crates/ui/tests/ui_test/main.rs` and `crates/ui/src/lib.rs` need **no** edit.

If after 3.4 the crate still compiles with a dangling reference to the old name
anywhere, that is a finding: report it, do not widen your scope.

### 3.5 Convert P1 -- the ONE sanctioned test change (sec5)

Do it between 3.1 and 3.4 if you prefer; it compiles either way, because both
builders exist until 3.2. Its full post-conversion shape is sec5.3, and the
expectation must be captured first per sec5.4.

---

## 4. Red - three executed Reds, none of them fabricated

There is no production-behaviour Red to drive, and you must say so plainly: the
entry point this unit retires was proven by unit A's
`intra_group_non_tree_edge_carries_the_cycle_class` and its four siblings, and
it has been in production use since unit B wired the CLI to it.

The handoff **pre-authorises you to state that in your journal**: "no
production-behaviour Red for T3-E; the entry point it retires was proven in unit
A and put into production use by unit B, both reviewer-APPROVED." That is the
same disclosure Phase 2's T2-A handoff pre-authorised for an unobservable Red
(deviation C), and its reviewer ACCEPTED it over a fabricated Red.

What replaces it is **three real, executed demonstrations**. Each one is a gate
invocation you actually run; each one's canonical line goes in your journal.

| # | What you do | The line you must record |
|---|---|---|
| R1 | finish 3.1-3.4 but leave **exactly one** of the nine sites still calling the deleted name; run gate 1 | a compile FAILURE, quoted verbatim |
| R2 | fix that site, run gate 1 | `186 passed / 0 failed / 0 ignored / 1 suites` |
| R3 | with R2 green, mutate **one character** of the frozen expectation (this table's R3); run gate 1 | **`185 passed / 1 failed / 0 ignored / 1 suites`** |

- **R1 proves the site set is complete and the deletion is real.** A stale
  reference to the deleted `build_include_elements` is only expressible AFTER
  3.2, so the old text ("before you delete, deliberately leave one stale
  reference") was sequenced impossibly; do it after 3.2 and before the last
  rename. Any one of the nine sites serves; this handoff nominates
  `export_order.rs:91`, because its body is a bare call, so the error names one
  line and nothing else. Record the `error[E0xxx]` code, the message, and the
  file:line verbatim. A rename-and-delete with no evidence of completeness is
  this unit's whole risk, so if your journal lacks R1 the unit is not done.
- **R2 is the count identity.** `186 passed / 0 failed / 0 ignored / 1 suites` --
  exactly what unit D2 left at `b0a91d2`. **The count must not move**, because
  the sec5 conversion rewrites one test body and adds, removes and renames
  nothing (sec5.1).
- **R3 proves the converted lock can fail.** A frozen-expectation test that has
  never been observed failing is the old vacuity in new clothes, so mutate it
  once and observe it. Choose a substring that occurs **exactly once** in the
  literal (count before you change it), change exactly one character inside it,
  and keep the line count identical so no later line number moves. The failure
  must be AT the frozen-expectation assertion, not at a guard assert and not a
  compile error -- say which in your journal. Quote the `panicked at` and
  `assertion ... failed:` lines verbatim; the two blobs are ~2.5 KB each and
  would themselves breach the 2000-character source-line rule, so quote them by
  capture path and identifying key, exactly as unit D2's Dev-6 did.
- **Never commit a Red state.** R1 and R3 exist only in your working tree.

**Never write a single line anywhere that reports `0 failed` while describing a
failing test.** The T3-B reviewer recorded exactly that self-contradiction as a
planner defect (`tasks/m235-t3b-verdict.md` finding 1).

---

## 5. Required tests

### 5.0 The decision this section encodes (T3-D2 verdict finding F2)

After 3.3 the name `build_include_elements` denotes the group-based builder, so
unit D2's P1 would invoke ONE function twice with the same `&[]` third argument
and assert that a thing equals itself: permanently green, asserting nothing.
That is the vacuity this milestone exists to prevent, so P1 cannot ship as
written.

**Decision: P1 is CONVERTED, not deleted.** It becomes a comparison of the
surviving builder against a **frozen recorded expectation**, so that after the
deletion it still pins the real builder's emitted bytes and stays a genuine
regression lock. Rationale, and the alternatives weighed against it:

- **Rejected -- delete P1 and treat its committed history as the record.** The
  history is real: `tasks/m235-t3d2-journal.md` Checkpoints 2-3 hold
  `185 passed / 1 failed / 0 ignored / 1 suites` with the byte lock failing at
  `export_order.rs:433`, then `186/0/0/1` after a revert proven byte-for-byte by
  two identical SHA256 values, and `tasks/m235-t3d2-verdict.md` section B
  re-derives that failure from source rather than believing the journal. What
  deletion costs is concrete: `crates/ui` loses its ONLY full-payload byte lock
  on the acyclic, empty-currency case. `graph_api.rs:741-789` locks determinism,
  not payload content; `dto_builders.rs` locks semantics key by key; nothing
  else in the crate compares emitted bytes for this case. A lock that cannot
  fail is not a record, it is a comment.
- **Chosen -- convert.** At `b0a91d2` both builders emit the SAME bytes on this
  input, proved by the committed byte lock rather than assumed, so recording
  those bytes pins exactly what the parity lock proved. The conversion is not a
  weaker claim than the test it replaces: it is the same claim with the second,
  now-redundant operand removed and replaced by a recorded value -- and unlike
  the deleted version it keeps failing when emission drifts.
- **Rejected -- keep the double call.** Tautology by construction.
- **Rejected -- demote P1 to a decoration-reachability test** (keep the guards,
  drop the payload comparison). Keeps a test, loses the lock, and leaves a test
  whose name no longer states any payload claim: the same vacuity, cheaper.
- **Rejected -- a new golden file.** See the last paragraph of 5.2.

**P2 is untouched by this decision.** It passes `&[]` at `:310` and `&groups`
at `:317` to a single name, so after the rename it is still a real comparison
with teeth and keeps its `assert_ne!` verbatim. Only its callee names change.

### 5.1 The pin - exactly ONE test changes

**The old pin ("no new tests, no renamed tests, no changed assertions ... You
add and remove no test") is WITHDRAWN**, because obeying it is precisely what
would have shipped the tautology. In its place:

- You add **no** test, remove **no** test, and change **no** test other than
  `export_order.rs`'s P1. Its body is rewritten in full per 5.3, and it is
  RENAMED to the exact name in 5.3. That rename is part of the same single
  sanctioned change, not a second one: the old name
  (`acyclic_include_elements_are_byte_identical_under_both_cycle_currencies`)
  asserts a two-builder property that stops being true at this commit, so
  leaving it would put a false claim in the test list.
- Every other test in `crates/ui/tests/ui_test/` -- the 179 pre-existing, unit
  A's 5, and unit D2's P2 -- must be **byte-identical** in name and body except
  for the callee name at the sites sec3.4 enumerates.
- Changing a second test, adding a third assertion family, or touching any
  fixture is a blocker, not a judgement call.

Count: `crates/ui/tests/ui_test/` carries exactly **186**
`#[test]`/`#[tokio::test]` functions at `b0a91d2` -- 179 measured at `b950963`,
plus 5 added by unit A, plus 2 added by unit D2 at `42bb44e`; zero `#[ignore]`
in that directory; 19 modules declared in `main.rs`. **186 at entry, 186 at
exit.** A body rewrite and a rename move neither count:
186 - 0 + 0 = 186.

### 5.2 Where the frozen expectation lives, and why not a golden file

**Inline**, in `crates/ui/tests/ui_test/export_order.rs`, as a module-scope
constant immediately above the converted test:

    const ACYCLIC_EMPTY_CURRENCY_PAYLOAD: &str = concat!(
        "...", "...", "...",
    );

**No new file enters any golden or fixture set.** Each candidate was weighed:

- `tests/golden/**` is a BYTE-LOCKED set (ledger, Byte-locked sets) whose
  regeneration protocol is `UPDATE_GOLDENS=1` plus a manifest hash resync, and
  whose workspace guard is specced-but-unimplemented (ledger row T6). It is
  LF-pinned by `.gitattributes`; adding a file to an LF-pinned, hash-locked set
  is exactly the shape that has been a late gate casualty in this repo. Every
  Phase 3 handoff, this one included, freezes `tests/golden/**` byte-unchanged,
  so a new golden would breach this unit's own lock table.
- `crates/ui/tests/fixtures/**` is byte-locked too ("changes need explicit
  review approval"), so a new fixture file needs an out-of-band waiver -- the
  same defect F6 records for reviewer records.
- A new data file inside `crates/ui/tests/ui_test/` would be UNPINNED by
  `.gitattributes` under `core.autocrlf=true`, which is the F12 trap (stat-cache
  re-dirty, stat-only diffs) in a set whose entire purpose is byte exactness.
- So: no new file. The const ships inside the test file it locks.

The const is about 2.5 KB (unit D2's Dev-6 measured the serialized payload at
roughly that size), so it **must** be split into `concat!` chunks of at most 96
characters, one chunk per line: a single-line literal would breach the
2000-character source-line rule. The serialization contains no newline, so no
chunk spans a line break, and every interior `"` must be escaped as `\"`.

### 5.3 The converted P1, in full

Name it exactly:

    acyclic_include_elements_match_the_frozen_acyclic_payload

Body, in this order, with the module-scope const of 5.2 directly above it:

1. `let graph = shuffled_file_graph();` then the unchanged acyclicity assert
   (`graph.detect_cycle_groups().is_empty()`, message naming why).
2. The three input maps, byte-identical to today's `export_order.rs:364-377`:
   `closures` covering ids 5 and 1, `file_languages` covering 1 and 3,
   `file_unit_kinds` covering 1.
3. **EXACTLY ONE builder call:**

       let elements = build_include_elements_from_groups(
           &graph,
           &closures,
           &[],
           &file_languages,
           &file_unit_kinds,
       );

   ONE call, not two. That single-invocation rule IS the anti-tautology
   invariant: a comparison whose second operand is another invocation of the
   same function asserts nothing, which is the exact failure mode this
   conversion removes.
4. The non-vacuity guards, unchanged in substance, now reading from the single
   `elements` vector, keeping today's messages from `:392-426`: the vector is
   non-empty; at least one node element (kind `HEADER` or `SOURCE`) carries
   `data["closure"].as_u64().unwrap_or(0) > 0`; at least one carries a
   `language` key; and strictly fewer than `node_elements.len()` carry it, which
   is the D27 OMITS case.
5. `let payload = serde_json::to_string(&elements).expect("elements
   serialize");` then
   `assert_eq!(payload, ACYCLIC_EMPTY_CURRENCY_PAYLOAD, "...")`, the message
   naming the property: on acyclic input with empty cycle currency the
   surviving builder's payload must equal the expectation recorded at
   `b0a91d2`.
6. **Nothing else.** Today's structural lock at `export_order.rs:442-445` is
   DELETED, not ported: with a single invocation it would become
   `assert_eq!(x, x)`, which is the tautology. If you feel the need to "add
   something" to make the test stronger, you are reintroducing the defect this
   section exists to prevent.

Replace today's header comment (`export_order.rs:333-349`) with one that states,
in your own words: (a) what the test now locks -- the surviving builder's
emitted bytes on an acyclic graph with empty cycle currency, against a recorded
expectation; (b) WHERE the expectation came from -- the committed parity lock at
`42bb44e`, whose byte lock proved `legacy_json == grouped_json` on this exact
input, so the recorded bytes are the agreed payload of BOTH builders and this
unit's deletion is behaviour-preserving ON THIS INPUT; (c) what it still does
NOT lock -- cross-process emission determinism, which lives in
`graph_api.rs:741-789` and in the CLI two-process lock
`export_bytes_are_identical_across_two_separate_processes` in
`crates/cli/tests/payload_tail_bench.rs`; and (d) that the expectation is
frozen deliberately, so a diff here means emission changed and a HUMAN
re-approves the expectation -- never the implementer.

Do not amend the module doc at `export_order.rs:1-8`: its caution about
double-build equality remains true as a caution and is outside this unit.

### 5.4 How to obtain the expectation, honestly

You cannot type it from this handoff and you must not invent it.

1. With the tree untouched at `b0a91d2` and gate 1 green at 186, add ONE
   temporary line inside P1, immediately after today's `:432` (once
   `grouped_json` is computed):
       std::fs::write(std::env::temp_dir().join("t3e_p1_payload.json"), &grouped_json).expect("capture payload");
   Run gate 1 once. It stays green.
2. Read that file with the Kilo `Read` tool. It holds the exact
   `serde_json::to_string` bytes of the GROUP-based side on the empty-currency
   acyclic input. Today's byte lock at `export_order.rs:433` is what makes those
   bytes also the legacy side's output; that equality is your provenance and
   belongs in your journal.
3. **Delete the temporary line before you commit anything**;
   `git diff --numstat` for `export_order.rs` must never show it. The capture
   file lives in the OS temp directory, never inside the worktree.

Then paste those bytes into the `concat!` const (5.2) and convert the body
(5.3). **If the first run after pasting fails, you mis-transcribed or
mis-escaped the blob**: re-read the capture and re-paste. Never reach green by
editing the expectation to fit a wrong builder output, by loosening the
comparison (`contains`, or comparing `Vec` values instead of the string), or by
deleting a guard assert. A first-run failure here is the expected consequence of
hand-transcribing 2.5 KB; record it in your journal, resolve it by re-pasting,
and never by weakening.

---

## 6. Refactor and hygiene targets

- Functions < 100 lines, files < 1000 lines. **Measured, not estimated:**
  `crates/ui/src/dto.rs` is **901** lines and `crates/ui/src/cycle_groups_view.rs`
  is **149** lines at `b0a91d2`. (The old text said "`dto.rs` was 852 lines at
  `b950963` ... deleting the old entry point (~45 lines)", which was both a stale
  base and a bad estimate: the legacy function is 19 lines with its legacy doc
  paragraph, not 45.) Expect `dto.rs` at roughly **882** and
  `cycle_groups_view.rs` at roughly **121** after 3.1-3.2.
  **Report the exact before/after counts for both files.** If `dto.rs` lands
  above **940** lines, say so and stop rather than extracting a module nobody
  asked for.
- `export_order.rs` is 446 lines at `b0a91d2`. The conversion (5.3) drops the
  second call, the structural lock and the legacy binding, and adds the const
  and its header, so the file should end near **440**. Report the exact count.
  The converted test must stay under the hard 100-line function ceiling; the
  `concat!` const sits at module scope and is not part of the function.
- Every `concat!` chunk is one line of at most 96 characters. No source line you
  author may reach 2000 characters -- which is the whole reason the 2.5 KB
  expectation is split.
- No `tmp`, `res`, `foo`, `bar`, `test1`; no single-letter locals in code you
  author; one responsibility per function.
- `python tools/hygiene_check.py`: zero R1/R2. Every name you author
  intention-revealing. **Zero NEW R3 warn lines in any file you touch** -- unit
  D2 held that line for `export_order.rs` and it is the standard here.
- **Authored content is pure ASCII.** No em dash, no arrow glyph, no section
  sign: use `--`, `->`, "sec".

---

## 7. Acceptance - gates, exact commands, expected verdict lines

Every gate in DIRECT form. **Never write a gate as
`python tools/distill.py -- <argv>`** unless `<argv>` is a `cargo build`.

**Never write a bare-word test filter** -- `testrun.py ... -- <word>` passes
`<word>` to cargo as a TEST-NAME filter and selected zero tests in Phase 1,
returning `0 passed / 0 failed / 0 ignored / 7 suites` with exit 0 (Phase 1
verdict finding 2). Use `--test <target>`.

**8 distinct gates, 12 planned invocations.** Gate 1 runs five times and gate 3
twice, both prescribed rather than deviations: gate 1 carries the capture run
(5.4), the three Red/Green states of sec4 (R1 compile failure, R2 Green, R3
mutation Red) and the final restored Green; gate 3 runs once at the untouched
dispatch base, so that the crate is known warning-free BEFORE any deletion, and
once after 3.1-3.3, so any warning is attributable to this unit's own edits.
Nothing else repeats.

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui
3. python tools/distill.py -- cargo build -p ariadne-ui
4. python tools/theme_token_check.py --check-added --base <base-rev>
5. python tools/hygiene_check.py
6. python tools/encoding_guard.py --staged
7. python tools/checkpoint_guard.py status
8. python tools/commit_scoped.py --scope 'crates/ui/src/dto.rs' --scope 'crates/ui/src/cycle_groups_view.rs' --scope 'crates/ui/tests/ui_test/dto_builders.rs' --scope 'crates/ui/tests/ui_test/graph_api.rs' --scope 'crates/ui/tests/ui_test/export_order.rs' --scope 'crates/ui/tests/ui_test/d27_node_provenance.rs' --scope 'crates/ui/tests/ui_test/common.rs' --scope 'tasks/m235-t3e-journal.md' -m "refactor(ui): retire the path-based include element builder"
```

**Gate 8's scope list is the touch-set, verbatim.** Eight `--scope` flags: seven
code paths plus the journal. A path missing from it is a path that cannot be
committed; a path present that you did not touch is a finding.

**Gate 6 ordering, pre-authorised.** `commit_scoped.py` stages and commits
atomically and raw git staging is denied, so `encoding_guard.py --staged` cannot
interpose between staging and committing. Run the **strictly stronger**
`python tools/encoding_guard.py --all` immediately BEFORE the commit and the
literal `--staged` form immediately AFTER. Disclose this as a pre-authorised
substitution, not as a deviation. Exact precedent: T2-A gate 5, T2-B Dev-E,
T3-A D5, T3-B D5, T3-C Dev-3, T3-D Dev-4, T3-D2 Dev-4.

**Gate 4's `--base`:** use the base revision **you were dispatched from** (the
commit your `checkpoint_guard.py status` preflight reports; expect `b0a91d2`,
unit D2's tip). `--base` must be the revision your work started at so the check
covers only what you added. **Do not use an older revision here** -- `b950963`
would sweep in units A, B, C, D and D2 and is not what the tool is for, and the
pre-amendment text's "normally unit A's or unit D's commit" is stale. **If you
cannot determine your base revision, run the check and record the verdict you
get; do not guess a revision.**

**Expected verdict lines:**

| Gate | Expected verbatim line |
|---|---|
| 1 | `186 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `186 passed / 0 failed / 0 ignored / 3 suites` |
| 3 | `ok (exit 0, 0 warnings emitted)` |
| 4 | no added hex literals (zero findings) |
| 5 | zero R1/R2 lines; R3 warns, if any, are legacy files only, and none in a file you touched |
| 6 | clean, no output |

**The arithmetic, so you can check it rather than trust it.** `crates/ui/tests/
ui_test/` carries exactly **186** `#[test]`/`#[tokio::test]` functions at
`b0a91d2` (counted across all 19 modules declared in `main.rs`;
`#[tokio::test]` expands to one `#[test]`, and there is **no** `#[ignore]` in
that directory): 179 at `b950963` + 5 from unit A + 2 from unit D2 at `42bb44e`.
Unit D2's reviewer MEASURED `186/0/0/1` and `186/0/0/3`; the old `184` in this
table was the pre-D2 number and would have made a correct implementer record a
false gate failure. This unit adds none, removes none, renames none:
186 - 0 + 0 = 186. `crates/ui/src/` carries **zero** `#[test]` functions, so
gate 2's three suites are lib + `ui_test` + doc-tests. **Recompute from
`crates/ui/Cargo.toml` and the capture every time; never copy a count or a suite
total out of the ledger.**

**Gate 3 must report ZERO warnings, and it is now load-bearing for a deletion as
well as a rename.** It is the only proof that (a) the rename left no dangling
import -- the four two-name import lists of sec1 are the trap -- and (b)
`from_paths` was actually deleted, because an un-deleted `from_paths` is
unreachable code in a `pub(crate)` module and the only thing standing between it
and a `dead_code` warning is 3.1. Phase 1's reviewer treated a warning-free
build as evidence, not style.

**Effect on the canonical workspace line: zero, and you must not measure it.**
The last MEASURED line is `1178 passed / 0 failed / 4 ignored / 88 suites` at
`aebfa9b` (the T3-D reviewer). Unit D2 added +2 -- predicted `1180/0/4/88` --
but its reviewer deliberately DEFERRED the workspace-wide measurement to T4.1,
so `1180/0/4/88` is a **prediction, not a measurement**, and the old text here
(`1177/0/4/88`, from a stale +1 attribution to unit B) was wrong twice over.
This unit adds 0, so the line after you is still predicted `1180/0/4/88`.
**You do not run the workspace-wide gate and you must not claim any number here
as measured.**

---

## 8. Frozen locks - each with a proving command

| Lock | Proving command | Expected |
|---|---|---|
| exactly one cycle entry point in `crates/ui` | reviewer's grep for `pub fn build_include_elements` | one definition, third parameter `&[CycleGroup]`, no path-based signature |
| exactly one cycle CURRENCY constructor | reviewer's grep for `from_paths` under `crates/ui/src` | zero hits; `from_groups` still at `cycle_groups_view.rs:63-88` |
| rest of `cycle_groups_view.rs` byte-unchanged apart from `:1-9` and `:26-53` | reviewer's `git diff` | deletions confined to `from_paths` + its doc; no reformat, no reorder |
| `crates/ui/src/lib.rs` byte-unchanged | reviewer's per-commit `git diff --name-only` | absent |
| `crates/ui/tests/ui_test/main.rs` byte-unchanged (still 19 modules) | same | absent |
| the 185 tests other than the sanctioned P1 byte-identical apart from the callee name | reviewer's `git diff` on the FIVE test files | callee-name-only changes; in `export_order.rs` confined to `:91`, `:108`, `:310`, `:317`, the converted P1 block, and the import list at `:15` |
| the converted P1 calls the builder EXACTLY ONCE and holds NO second-computation assert | reviewer's read of the converted body | one call; no `assert_eq!(x, x)`; no `contains`-style loosening |
| test COUNT unmoved: the sanctioned body rewrite and rename are neither an add nor a remove | gate 1 | `186/0/0/1`; gate 2 `186/0/0/3` |
| D27 OMITS rule and canonical emission order | gate 1 (`export_order.rs`, `d27_node_provenance.rs` tests green unedited) | green |
| zero new hex literals, verdict colors intact across all three themes | gate 4 | zero findings |
| `crates/cli/**`, `crates/graph/**`, `crates/report/**` byte-unchanged | reviewer's `git diff --name-only` | absent |
| `tests/fixtures/**`, `tests/golden/**` byte-unchanged, and NO new file in either | reviewer's `git diff --name-only`; no `UPDATE_GOLDENS` run | absent, and zero added paths |
| R1/R2 hygiene | gate 5 | no R1/R2; `dto.rs` and `cycle_groups_view.rs` well under 1000 |
| UTF-8 mojibake | gate 6 | clean; authored content pure ASCII |
| touch-set | reviewer's per-commit `git diff --name-only` | exactly the EIGHT in-set paths |
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

1. **The NINE sites are the complete set.** The journal's deliberate
   stale-reference compile error (sec4 R1) is the evidence that no tenth call
   site exists somewhere the implementer did not grep. If the journal lacks it,
   ask for it: a rename-and-delete with no evidence of completeness is the unit's
   whole risk.
2. **`from_paths` is gone, and gate 3 is why.** Zero hits under `crates/ui/src`,
   `from_groups` intact, nothing else in `cycle_groups_view.rs` touched. An
   un-deleted `from_paths`, or a `#[allow(dead_code)]` on it, is a finding: it
   is the D1-B dual representation wearing a silencing attribute.
3. **The two now-nonsense doc sentences were fixed**, not preserved:
   `cycle_groups_view.rs:4` (named the deleted `from_paths`) and `dto.rs:625-626`
   ("the group-based sibling of `build_include_elements`"). Both are the same
   self-referential documentation defect class as the ledger's
   `build_include_elements_legacy_paths` row.
4. **The four two-name import lists collapsed to one name each**, and
   `common.rs:3` was left alone. A duplicate-name import is a compile error, but
   a silently rewritten import list is a scope finding.
5. **The P1 conversion is a conversion, not a tautology and not a deletion.**
   Exactly one builder invocation; no second-computation assert; the structural
   lock deleted rather than ported; a non-empty, closure-decorated,
   language-partial guard set still in place ahead of the byte comparison; and a
   header that states where the frozen bytes came from.
6. **The expectation was captured, not invented.** The journal records the
   capture procedure, that the temporary line was removed, and that the
   expectation equals the group side's output at `b0a91d2` -- which the committed
   parity lock at `42bb44e` proves is also the legacy side's output. **A
   first-run failure resolved by re-pasting is fine; a first-run failure resolved
   by weakening the assertion is a REJECT.** No new file in `tests/golden/**`,
   `tests/fixtures/**` or `crates/ui/tests/ui_test/`.
7. **R3 was really executed**: the mutated expectation, the verbatim panic, and
   the restore. A frozen lock that has never been seen to fail is the old
   vacuity in new clothes.
8. **No accidental behaviour change elsewhere.** The `git diff` on the four
   non-`export_order` test files must be callee-name-only, and in
   `export_order.rs` confined to the sites sec3.4 lists plus the converted P1
   block. A changed assertion in any other test is a finding even if it "fixes"
   something.
9. **`dto.rs` and `cycle_groups_view.rs` line counts** after the deletions,
   reported honestly against the measured 901 and 149.
10. **Gate 4's `--base` revision** is the implementer's dispatch base, not a
    hard-coded older revision.

**Report back (final message):** commit SHAs (also in your journal); the
verbatim stale-reference compile error from R1; the R3 panic verbatim; Green
evidence quoting the canonical lines and capture paths for gates 1, 2 and 3; the
`dto.rs` and `cycle_groups_view.rs` before/after line counts; the `export_order.rs`
before/after count; how the expectation was captured and whether a first run
failed; the `--base` revision you used for gate 4; every deviation with its
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
- **Strict TDD.** Capture (5.4) -> convert P1 (5.3) -> 3.1 -> 3.2 -> 3.3 ->
  3.4 -> R1 (deliberate stale reference, compile failure) -> R2 (Green) -> R3
  (mutated expectation, Red) -> restore -> final Green -> Refactor. Never
  weaken an assertion. Never commit a state that does not compile. Never commit
  a Red state.
- **The capture line is temporary and is not yours to keep.** The single
  `std::fs::write` line of 5.4 exists only long enough to read the bytes. It is
  removed before any commit, it writes only into the OS temp directory, and it
  must never appear in `git diff --numstat` for `export_order.rs`. Read the
  capture with the Kilo `Read` tool; do not route it through PowerShell.
- **Never edit the frozen expectation to match an output you did not expect.**
  That single rule is the difference between this conversion being a regression
  lock and being a rubber stamp.
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
| Scope | <= 1 Cargo crate OR <= 1 top-level module directory | `crates/ui` only -- **1 crate**, 8 paths |
| Gate commands | ~10 | **8 distinct gates, 12 planned invocations** (gate 1 five times, gate 3 twice; the RGB trail of sec4 is why, and the count is disclosed rather than trimmed) |
| Produced lines (implementation + test diff) | ~1200 | **~210**, itemised below |

**The line estimate, itemised from the bodies this handoff actually mandates** --
not derived from a ledger line and not rounded up to look safe. The standing
rule (ledger row "M23.5 handoff sizing arithmetic") requires exactly this, after
two prior misses: T3-D predicted 679 and landed 734, and T3-D2 predicted 84 and
landed 167. Both misses were additions estimated from anything other than the
mandated test bodies.

| Item | Lines |
|---|---|
| `dto.rs`: delete the legacy function and its legacy doc paragraph | -19 |
| `dto.rs`: rename + three doc obligations (self-referential sibling sentence, moved D27 paragraph, parameter history note) | +6 |
| `cycle_groups_view.rs`: delete `from_paths` + its doc comment | -28 |
| `cycle_groups_view.rs`: amend the module doc | +2 |
| nine call sites, callee name only | +9 / -9 |
| four import lists, one name dropped each | +4 / -4 |
| P1 conversion (sec5.3): new header, guards, single call, const reference | +30 |
| P1 conversion: removed second call, structural lock, legacy binding | -35 |
| `ACYCLIC_EMPTY_CURRENCY_PAYLOAD` const, ~2.5 KB in <=96-char `concat!` chunks | +27 |
| **subtotal, code** | **~-17** |
| journal (5 gate-1 states, 3 Red trails, capture disclosure, deviations) | ~90 |
| **total** | **~210 net, ~260 churn** |

**Single handoff unit -- fits with wide headroom, and it remains deliberately the
smallest Phase 3 unit.**

Why it exists at all: D1-B ("no dual representation") is a plan acceptance
criterion, and unit A could not retire the path-based entry point in its own
commit without breaking `crates/cli`. Two `crates/ui`-scoped commits -- one that
adds (unit A) and one that retires (this unit), with the `crates/cli` rewiring
between them (unit B) -- is the only decomposition under the one-crate law that
keeps every handoff boundary compiling and green.

---

## 12. Authorization and ordering

This handoff (as amended 2026-10-04), `tasks/m235-kickoff.md`, the M23.5
milestone row in `tasks/milestone_ledger.md`, and APPROVED
`tasks/m235-t3a-handoff.md`, `tasks/m235-t3b-handoff.md` **and**
`tasks/m235-t3d2-handoff.md` (verdict `tasks/m235-t3d2-verdict.md`) are the sole
authorization for this unit.

**Runs last.** Ordering across Phase 3, with the dependency that forces it:

```
A   crates/ui           additive DTO entry point        <- done, APPROVED
B   crates/cli          CLI consumers + bench re-pin    <- done, APPROVED
C   tests/integration   Scala consumers + IV-9 gate     <- done, APPROVED
D   crates/graph        IV-11 port + ATOMIC DELETE      <- done, APPROVED
D2  crates/ui           legacy/group parity lock        <- done, APPROVED (`42bb44e`+`4d0f245`+`b0a91d2`)
E   crates/ui           retire the path-based DTO       <- needs A, B and D2   *** this unit
```

**Unit D2 was a HARD EXPIRY on this unit, and it is now discharged.** D2 existed
because the parity lock could only be written while both builders existed and
this unit pins one test change rather than a test addition; it landed before this
handoff was dispatched, and sec5.0 is the consequence its reviewer identified.

**Parallel safety:** this unit and unit D touch **disjoint files**
(`crates/ui/**` vs `crates/graph/**`) and neither depends on the other, so they
may run concurrently **in separate worktrees**. In this shared worktree,
dispatch sequentially. Unit D2 (`crates/ui/tests/ui_test/export_order.rs`) was
**NOT** parallel-safe with this unit -- both touch the same file -- and is now
complete, so nothing is outstanding. No two Phase 3 units other than (A, C),
(B, C) and (D, D2) were parallel-safe: A/B share the entry-point name, B/D share
the delete precondition, A/E share `crates/ui/src/dto.rs`, E/D2 share
`export_order.rs`.

Phase 4 (T4.1 integration closeout) remains UNAUTHORIZED. Do not begin it, do not
run the workspace-wide gates, and do not close ledger rows F9, F10, F11 or F12 --
those are operator actions at closeout. Do not touch `crates/graph/**`,
`crates/cli/**`, `tests/**`, or `tools/**`.

---

## 13. Planner defects recorded here so T4.1 does not inherit them

Every item below is attributed to the **planner**, not to the implementer. They
are recorded rather than fixed in place because each one is a template or
estimation defect, not a statement in this file.

### 13.1 F1a-F1d -- this handoff was stale against the tree unit D2 landed

`tasks/m235-t3d2-verdict.md` findings F1a-F1d, all corrected above. The defect
class is **dispatch-blocking**: an honest implementer following the old text
would have recorded a false gate failure at F1a, stopped and reported at F1c, or
carried a dangling reference to a deleted function at the same place. Standing
rule, now binding on every handoff in this milestone: **a handoff's counts,
caller enumerations and line references are re-derived from the tree at the unit's
dispatch base, in the same pass that writes the contract -- and any unit that
lands between one handoff's authoring and its dispatch forces a re-derivation,
not an edit-around.** T3-A, T3-B, T3-C, T3-D and T3-D2 each landed before the
next handoff was written, and this is the first time one landed AFTER.

### 13.2 Missing deletion whose file was outside the touch-set (NOT in F1)

`IncludeCycleClass::from_paths` (`cycle_groups_view.rs:32-53`) has exactly one
caller, `dto.rs:618`, inside the function this unit deletes. The old sec3 item 1
said "delete ... and every helper that existed only to serve it. Before deleting
a helper, grep for its name" -- but `cycle_groups_view.rs` was NOT in the
touch-set. So the contract simultaneously required a zero-warning build (gate 3)
and forbade the only edit that could produce one: the unit was **required** to
finish with a `dead_code` warning, or to leave a second cycle currency in the
tree, which is D1-B's dual representation. Fixed by 3.1 plus the sec2
addition. Standing rule: **when a touch-set boundary is crossed by a
deletion's blast radius, the file joins the touch-set with a tightly bounded
obligation -- a reviewer who cannot reach gate 3 has been handed a contradiction,
not a task.** F6 is the same shape one level up: an edit a unit genuinely needs,
landing outside the declared touch-set.

### 13.3 Under-specified import lists (NOT in F1)

The old sec3 item 4 said only "any import list that named the old function is
updated". Four files import BOTH names, so after the rename each needs a
DE-DUPLICATION, which that sentence did not say. Now enumerated in sec1.
Standing rule: **"update the call sites" is not a contract; enumerate every line
that carries the name, including `use` lists, and state what each becomes.**

### 13.4 A doc sentence that becomes nonsense under the rename (NOT in F1)

`dto.rs:625-626` reads "the group-based sibling of `build_include_elements`".
After the rename it names the function as its own sibling. The old sec3 item 2
listed two doc obligations and this was not one of them. Now mandatory in 3.3,
with the same reasoning as 3.1's module doc. Standing rule: **when a rename
touches a name that its own documentation references, the documentation is part
of the rename.**

### 13.5 F3 -- sizing arithmetic, second consecutive miss in this milestone

Unit D2's handoff itemised its two mandated test bodies at +84 and **+167**
landed; the T3-D2 reviewer adjudicated it to the planner (Dev-1) as the same
defect class as T3-D's predicted 679 versus measured 734. Sec11 above is
re-derived from the mandated bodies, including the ~2.5 KB frozen expectation,
which is exactly the line an estimate that ignores mandated test content
undercounts. Recommendation for the `handoff-spec` template: **require every
addition estimate to be itemised from the mandated bodies, and require the
author to state the base revision the count was measured at.**

### 13.6 F4 -- a contract that mandated content while forbidding its remedy

Unit D2's sec5.1 mandated six ordered steps inside one test function while its
sec3.2 permitted a helper ONLY if the maps could not be written inline (they
can), producing a 96-line function against a ~40-line guideline. The hard
<100-line ceiling held, so it was never a hygiene finding -- but it was a
planner contradiction, and the implementer had to choose which instruction to
break. Recommendation: **never write a "prefer a helper" guideline into a
section that separately mandates an ordered inline body; either sanction the
helper or drop the guideline.**

### 13.7 F6 -- reviewer records need an out-of-band touch-set waiver (recommendation only)

For **every** unit of this milestone, the orchestrator had to land the unit's
reviewer records and the unit's own handoff through an explicit, out-of-band
`--scope` because the reviewer role holds read-only git and the touch-set did
not name `tasks/**`. `tasks/m235-t3d2-verdict.md` D5 records it for the seventh
time. **Recommendation, not a change made here:** `handoff-spec` should carry a
standing clause that the orchestrator may always land (a) the unit's reviewer
records and (b) the next unit's handoff, by explicit `--scope`, outside the
implementer's touch-set. Until that exists, every unit repeats the waiver and
the commit-vs-touch-set check keeps flagging a shape that is actually correct.
**This handoff does not attempt that change**: the skill is outside this unit's
and this planner's write scope.