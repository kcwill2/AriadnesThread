# M23.5 T3-E1 Handoff - Retire the Path-Based Include-Element DTO (crate:ui)

**Milestone:** M23.5, Phase 3, unit **E1** of 7 (A, B, C, D, D2, E1, E2).
**First of two replacement units for the SUPERSEDED `tasks/m235-t3e-handoff.md`.**
That contract was adjudicated **unsatisfiable** and its implementer stopped at
preflight with zero commits (`tasks/m235-t3e-journal.md`, BLOCKER section). Do
not read it as authority; read the journal for the executed proof of why.
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec1.3 line 69
(canonical public API), sec2 Phase 3 T3.2.
**Prerequisites, all APPROVED and landed:** A (`31513b9..ea82422`), B
(`9d831bf..91967c9`), D2 (`42bb44e` + `4d0f245` + `b0a91d2`). Unit D is
independent. **Dispatch base: `b0a91d2`**; HEAD must be `b0a91d2` at preflight.
**Journal:** `tasks/m235-t3e1-journal.md` (you create it).
**E2** (`tasks/m235-t3e2-handoff.md`) is NOT authorized until E1 is APPROVED.

This handoff is deliberately short. Where the old contract asserted a fact about
the tree, this one either cites the file and line so you can re-read it, or tells
you to run the check that establishes it. **If a line reference disagrees with
the file you read, THE FILE WINS** -- report the drift, do not edit around it.

---

## 0. PREFLIGHT

**0.1** `python tools/checkpoint_guard.py status`. Record HEAD, branch, dirty and
untracked paths. Confirm HEAD `b0a91d2`. If HEAD moved, stop and report.

**0.2 F12 fixture.** `tests/fixtures/basic/artifacts/includes/report.json` has
no `.gitattributes` pin (that file pins only `crates/ui/tests/fixtures/*.json`
and `tests/golden/**/*.json`) and this worktree has `core.autocrlf=true`, so a
full test run re-dirties its **stat entry** with byte-identical content (ledger
row M23.5 F12). Run
`git diff --numstat HEAD -- tests/fixtures/basic/artifacts/includes/report.json`.
**If and only if it prints rows**, the content is dirty: restore byte-safely with
the Kilo `Read` tool then the Kilo `Write` tool. NEVER `Set-Content`,
`Out-File`, `>`, `>>`, `Get-Content | -replace`; `git checkout --` /
`git restore` are mutating git and DENIED. If it prints no rows, do nothing and
say so. That path must appear in **NO** commit of either E unit.

**0.3 Caller census -- grep for BOTH names, not one.** This is the preflight that
the superseded contract got wrong (it grepped only the legacy name and so missed
the grouped-name hazard entirely).

    build_include_elements(_from_groups)?\s*\(
    pub fn build_include_elements
    from_paths

Run all three over `crates/`, `tests/` and `tools/` (Grep tool, or
`python tools/distill.py -- rg -n ...` for the shell). Expected at `b0a91d2`,
each verified against the tree:

**20 call sites. 9 legacy-name, all inside `crates/ui/tests/ui_test/`:**
`common.rs:157`; `d27_node_provenance.rs:141`, `:152`; `dto_builders.rs:15`;
`export_order.rs:91`, `:108`, `:310`, `:381`; `graph_api.rs:136`.

**11 grouped-name: 9 in `crates/ui/tests/ui_test/`** -- `d27_node_provenance.rs:174`;
`dto_builders.rs:470`, `:501`, `:588`; `export_order.rs:167`, `:317`, `:382`;
`graph_api.rs:752`, `:759` -- **and 2 in `crates/cli`:**
`crates/cli/src/commands/ui.rs:328` (production `build_ui_payload`) and
`crates/cli/tests/payload_tail_bench.rs:325` (inside `run_assembly`).

**Zero** call sites in `crates/report`, `crates/pipeline`, `crates/graph`,
`crates/index`, `tests/**`, `tools/**`.

**Definitions:** exactly two, `crates/ui/src/dto.rs:608` (legacy, third param
`cycles: &[Vec<FileId>]`) and `crates/ui/src/dto.rs:637` (grouped, third param
`cycle_groups: &[CycleGroup]`). `crates/ui/src/lib.rs:6` is a bare
`pub use dto::*;` -- the only export path, no alias, so a name deleted from
`dto.rs` is gone workspace-wide.

**`from_paths`:** exactly three textual hits -- `cycle_groups_view.rs:4` (module
doc), `cycle_groups_view.rs:32` (definition), `dto.rs:618` (its ONLY caller, and
it sits inside the function E1 deletes).

**STOP conditions.** Any count or location above that differs: STOP and report,
do not widen scope. Specifically, a tenth legacy call site, or either
`crates/cli` grouped site at a line other than `:328` / `:325`, or any other
crate naming either function.

---

## 1. Objective and invariants

**Objective.** Delete the path-based `build_include_elements` and its only
helper, repoint its nine in-crate callers at the surviving group-based builder,
and convert unit D2's parity test P1 so it keeps pinning emitted bytes after the
deletion. After E1 the UI payload layer has exactly one cycle entry point and one
cycle-currency constructor: milestone criterion **D1-B** ("no dual
representation") holds.

**E1 does NOT rename anything.** The survivor stays
`build_include_elements_from_groups`; renaming it to the plan's canonical
`build_include_elements` is unit E2, and it is the only part of the work that
crosses into `crates/cli`. If you are tempted to "finish the rename here",
stop: that is E2, and it is what keeps E1 inside one crate.

**Invariants preserved.**
- IV-9 / determinism: the surviving function's body, 5-arg shape, canonical
  `(path asc, file_id asc)` emission order, D27 OMITS rule and node-data keys
  are untouched. E1 is a deletion plus a name-level repoint, nothing else.
- Zero new hex literals: no CSS, JS or template is touched.
- IV-11 / IV-12: untouched; no report or Scala code.
- `crates/cli/**` byte-unchanged. Verified basis: after E1 the only remaining
  builder name is the one `crates/cli` already calls.

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

1. `crates/ui/src/dto.rs` -- delete the legacy entry point and its legacy doc
   paragraph; move the D27 paragraph onto the survivor.
2. `crates/ui/src/cycle_groups_view.rs` -- **only** the two bounded edits in
   3.2. Without this path in the touch-set gate 4's zero-warning build is
   unreachable (see 6).
3. `crates/ui/tests/ui_test/export_order.rs` -- sites `:91`, `:108`, `:310`,
   the P1 conversion, the import list at `:15`.
4. `crates/ui/tests/ui_test/dto_builders.rs` -- site `:15`, import list `:4`.
5. `crates/ui/tests/ui_test/graph_api.rs` -- site `:136`, import list `:4`.
6. `crates/ui/tests/ui_test/d27_node_provenance.rs` -- sites `:141`, `:152`,
   import list `:9`.
7. `crates/ui/tests/ui_test/common.rs` -- site `:157`.
8. `tasks/m235-t3e1-journal.md` -- **new**, yours.
9. Plus the planner-declared records in 8.2, commit 1 only.

`crates/ui/tests/ui_test/common.rs:3` needs **NO edit** -- it already imports
only `build_include_elements`, which after E1 denotes the survivor.

**You MUST NOT touch:** `crates/ui/src/lib.rs` (the re-export is automatic),
any other file under `crates/ui/src/` (including `export.rs`, `server.rs`),
any other file under `crates/ui/tests/ui_test/` (including `main.rs`), anything
under `crates/ui/assets/`, **`crates/cli/**`**, `crates/graph/**`,
`crates/report/**`, `tests/**`, `tools/**`, `.kilo/plans/**`, `Cargo.toml`,
`Cargo.lock`, `.gitattributes`. No `git stash`, no branch change, no `git add .`.

---

## 3. Green -- the minimal implementation bound, in this order

The order is load-bearing twice: 4.4's capture must run while both builders
exist, and 3.6 (the P1 conversion) must land before the final gates.

### 3.1 Delete the legacy entry point (`dto.rs`)

Delete `pub fn build_include_elements` (`dto.rs:608-622`) **and** its legacy doc
paragraph (`dto.rs:604-607`, "Legacy path-based entry point (pre-M23.5) ...
New callers must use `build_include_elements_from_groups`").

Keep the D27 paragraph at `dto.rs:599-602` and **move it** onto the survivor's
doc comment, which already states the OMITS rule for the two group keys at
`:635-636`. The survivor's own doc (`dto.rs:625-636`) is otherwise unchanged in
this unit: `dto.rs:625-626` ("the group-based sibling of
`build_include_elements`") is still TRUE after E1, because the two names are
still distinct. Fixing that sentence is **E2's** obligation, and doing it here
is a finding.

### 3.2 Delete the orphaned currency constructor (`cycle_groups_view.rs`)

`IncludeCycleClass::from_paths` (`cycle_groups_view.rs:32-53`) with its doc
comment (`:26-31`) had exactly one caller: `dto.rs:618`, inside the function 3.1
deletes (verified, 0.3). `cycle_groups_view` is `pub(crate)`
(`crates/ui/src/lib.rs:1`), so once that caller is gone `from_paths` is
unreachable code and `cargo build -p ariadne-ui` emits a `dead_code` warning.
Gate 4 is pinned at 0 warnings emitted, so **this deletion is mandatory, not
optional** -- the alternative (`#[allow(dead_code)]`) leaves D1-B's dual
representation wearing a silencing attribute, and keeping the view at all is the
same dual representation.

Also amend the module doc at `cycle_groups_view.rs:1-9`, which names the deleted
function at `:4`. Keep `:1-3` (purpose) and `:5-9` (the shared-tail rationale,
still true); restate `:4` to describe the single surviving currency. A doc
comment naming a deleted function is the defect class the ledger's
`build_include_elements_legacy_paths` row exists to stop.

**Everything else in that file is frozen:** `from_groups` (`:63-88`), the five
query methods (`:91-110`), `group_index_by_member` (`:115-123`),
`endpoints_share_group` (`:127-136`), `witness_path_labels` (`:142-149`). No
reformat, no reorder, no "tidying".

**There is no other legacy-exclusive code to hunt, and that is a verified fact,
not a guess.** Both entry points call the ONE private tail
`emit_include_elements` (`dto.rs:659`, called from `:615` and `:644`), and
`from_paths` was the only legacy-exclusive function in the crate. The production
deletion is therefore **19 lines in `dto.rs` and 28 in `cycle_groups_view.rs`**.
If your diff shows any other deletion in either file, STOP and report it.

### 3.3 Repoint the nine in-crate call sites -- callee name only

`build_include_elements(` becomes `build_include_elements_from_groups(` at the
nine sites 0.3 enumerates. Every one passes an **empty** currency (a bare `&[]`
at eight, `&cycles` from `vec![]` at `dto_builders.rs:13`), so the element type
is inferred from the parameter and **no type annotation changes anywhere** --
callee name only. You may rename `dto_builders.rs:13`'s local `cycles` to
`cycle_groups` if it reads better; nothing else in that file may move.

### 3.4 Collapse the four two-name import lists

Each of these files imports BOTH names today, so each needs a DE-DUPLICATION
(drop `build_include_elements_from_groups`), not an "update":

| File:line | Today | After E1 |
|---|---|---|
| `crates/ui/tests/ui_test/export_order.rs:15` | `build_include_elements, build_include_elements_from_groups, ...` | drop the grouped name |
| `crates/ui/tests/ui_test/dto_builders.rs:4` | `build_call_elements, build_include_elements, build_include_elements_from_groups, ...` | drop the grouped name |
| `crates/ui/tests/ui_test/graph_api.rs:4` | same two-name shape | drop the grouped name |
| `crates/ui/tests/ui_test/d27_node_provenance.rs:9` | `build_include_elements, build_include_elements_from_groups, build_symbol_elements` | drop the grouped name |
| `crates/ui/tests/ui_test/common.rs:3` | `build_call_elements, build_include_elements, ...` | **NO edit** -- names only the survivor |

The grouped-name calls already present at `dto_builders.rs:470/:501/:588`,
`export_order.rs:167/:382`, `graph_api.rs:752/:759`,
`d27_node_provenance.rs:174` and `export_order.rs:317` need **no edit** in E1 --
the name they call survives. Do not touch them.

### 3.5 Green, then the three Reds

Compile and run gate 1 (the capture-and-green run of 4.4), then the two Reds of
section 4, then the final green.

### 3.6 Convert P1 -- the ONE sanctioned test change (section 5)

Do it any time after 4.4's capture; it needs both builders to exist only at
capture time.

---

## 4. Red -- two executed Reds plus the capture-and-green run, none fabricated

State plainly in your journal: **no production-behaviour Red for E1**; the entry
point E1 retires was proven by unit A's
`intra_group_non_tree_edge_carries_the_cycle_class` and its four siblings and
put into production use by unit B, both reviewer-APPROVED. That disclosure is
pre-authorised here (same precedent as T2-A deviation C).

| # | What you do | Line you must record |
|---|---|---|
| R1 | Finish 3.1-3.4 but leave **exactly one** of the nine sites still calling the deleted name (this handoff nominates `export_order.rs:91`, whose body is a bare call); run gate 1 | a compile FAILURE, quoted verbatim with its `error[E0xxx]` code and file:line |
| R2 | with the unit green, mutate **ONE CHARACTER** of the frozen expectation (see 5.3); run gate 1 | **`185 passed / 1 failed / 0 ignored / 1 suites`** |

- **R1 proves the site set is complete and the deletion is real.** Do it AFTER
  3.1, because a stale reference to a deleted function is only expressible then.
  A rename-and-delete with no evidence of completeness is this unit's whole risk.
- **The count identity is the FINAL green run**, and it is what R2 reverts to.
  Exactly what D2 left at `b0a91d2`: `186/0/0/1`, and `186/0/0/3` for gate 2.
  The P1 conversion rewrites one body and renames it; it adds, removes and
  renames no test, so **186 - 0 + 0 = 186**. Re-derive from your own gate 1
  output; never copy a count out of a ledger or another verdict. There is no
  separate R2 green run: the single run after you revert R2's mutation is both
  the fix-verification and the final green, and folding them is what keeps gate
  1 at four invocations instead of five.
- **R2 must fail AT the byte lock** -- not at a guard assert and not at compile.
  Say which in your journal. Choose a substring occurring **exactly once** in
  the frozen expectation (count it before editing), change one character inside
  it, keep the line count identical. Quote the `panicked at` and
  `assertion ... failed:` lines verbatim and cite the two ~2.5 KB blobs by
  capture path and identifying key (pasting them would breach the
  2000-character source-line rule; same construction as D2's Dev-6).
- **Never commit a Red state**, and never write a line that reports `0 failed`
  while describing a failing test (T3-B verdict finding 1).

### 4.4 Obtaining the frozen expectation, honestly

You cannot type 2.5 KB from a handoff and you must not invent it.

1. At the untouched `b0a91d2` with gate 1 green, add ONE temporary line in P1
   immediately after today's `export_order.rs:432`:
   `std::fs::write(std::env::temp_dir().join("t3e1_p1_payload.json"), &grouped_json).expect("capture payload");`
2. Run gate 1 once; it stays green. Read the file with the Kilo `Read` tool. It
   holds the exact `serde_json::to_string` bytes of the **group-based** side.
3. **Delete the temporary line before you commit anything**; it must never appear
   in `git diff --numstat` for `export_order.rs`, and it writes only into the OS
   temp directory, never the worktree.

**Provenance, for your journal and for the new test header:** the committed byte
lock at `export_order.rs:433-437` asserts `grouped_json == legacy_json` on this
exact input, so those bytes are the agreed output of BOTH builders and E1's
deletion is behaviour-preserving *on this input*. That committed lock is your
provenance -- do not claim the capture alone proves parity.

---

## 5. Required tests

### 5.0 The decision (D2 verdict finding F2)

After 3.1 the name `build_include_elements` denotes the group-based builder, so
P1 as written would invoke ONE function twice with the same `&[]` and assert a
thing equals itself: permanently green, asserting nothing. **P1 is CONVERTED,
not deleted.** Deleting it would leave `crates/ui` with no full-payload byte lock
on the acyclic, empty-currency case at all (`graph_api.rs:741-789` locks
determinism, not payload content; `dto_builders.rs` locks keys individually).
Keep the double call is tautology; demoting P1 to a reachability test loses the
lock. **P2 is untouched by this decision**: `export_order.rs:310` passes `&[]`
and `:317` passes `&groups` to a single name, so it keeps its `assert_ne!` and
stays a real comparison -- only its callee name changes.

### 5.1 The pin -- exactly ONE test changes

You add **no** test, remove **no** test, and change **no** test other than P1.
Its body is rewritten in full per 5.2 and it is renamed, because the old name
(`acyclic_include_elements_are_byte_identical_under_both_cycle_currencies`)
asserts a two-builder property that stops being true at this commit. That
rename is part of the single sanctioned change, not a second one. Every other
test in `crates/ui/tests/ui_test/` must be byte-identical in name and body
except for the callee name at the sites 3.3 enumerates. Changing a second test
or touching any fixture is a blocker, not a judgement call.

### 5.2 The converted P1, in full

Name it exactly:

    acyclic_include_elements_match_the_frozen_acyclic_payload

Body, in this order, with the module-scope const of 5.3 directly above it:

1. `let graph = shuffled_file_graph();` then the unchanged acyclicity assert
   (today's `export_order.rs:355-358`).
2. The three input maps, byte-identical to today's `export_order.rs:364-377`:
   `closures` covering ids 5 and 1, `file_languages` covering 1 and 3,
   `file_unit_kinds` covering 1.
3. **EXACTLY ONE builder call** -- the surviving `build_include_elements_from_groups`
   with `&[]` third (today's `:382-388`). One call, not two. That
   single-invocation rule IS the anti-tautology invariant.
4. The non-vacuity guards, unchanged in substance, reading the single `elements`
   vector, keeping today's messages (`:392-426`): non-empty; at least one
   `HEADER`/`SOURCE` node element with `data["closure"].as_u64().unwrap_or(0) > 0`;
   at least one carrying a `language` key; strictly fewer than
   `node_elements.len()` carrying it (the D27 OMITS case). Guards stay **ahead**
   of the byte comparison -- without them "both sides empty, therefore equal"
   would satisfy the lock.
5. `let payload = serde_json::to_string(&elements).expect(...)`; then
   `assert_eq!(payload, ACYCLIC_EMPTY_CURRENCY_PAYLOAD, "...")` with a message
   naming the property: on acyclic input with empty cycle currency the surviving
   builder's payload must equal the expectation recorded at `b0a91d2`.
6. **Nothing else.** Today's structural lock (`export_order.rs:442-445`) is
   **DELETED, not ported**: with one invocation it would be `assert_eq!(x, x)`.
   Do not "add something" to make the test stronger; that reintroduces the
   defect 5.0 exists to prevent.

Replace today's header comment (`export_order.rs:333-349`) with one stating, in
your own words: (a) what the test now locks -- the surviving builder's emitted
bytes on an acyclic graph with empty cycle currency against a recorded
expectation; (b) WHERE the expectation came from -- the committed parity lock at
`42bb44e`, whose byte lock proved `legacy_json == grouped_json` on this exact
input, so the recorded bytes are the agreed payload of BOTH builders and E1's
deletion is behaviour-preserving on this input; (c) what it still does NOT lock
-- cross-process emission determinism, which lives in `graph_api.rs:741-789` and
in the CLI two-process lock `export_bytes_are_identical_across_two_separate_processes`
in `crates/cli/tests/payload_tail_bench.rs`; (d) that the expectation is frozen
deliberately, so a diff here means emission changed and a HUMAN re-approves it,
never the implementer.

Do not amend the module doc at `export_order.rs:1-8`; its caution about
double-build equality remains true and is out of scope.

### 5.3 Where the frozen expectation lives

**Inline** in `crates/ui/tests/ui_test/export_order.rs`, module scope,
immediately above the converted test:

    const ACYCLIC_EMPTY_CURRENCY_PAYLOAD: &str = concat!(
        "...", "...", "...",
    );

The expectation is roughly 2.5 KB, so it **must** be split into `concat!`
chunks of at most 96 characters, one chunk per line -- a single-line literal
would breach the 2000-character source-line rule and hide its own tail from every
tool. The serialization contains no newline, so no chunk spans a line break;
every interior `"` must be escaped as `\"`.

**No new file enters any golden or fixture set.** `.gitattributes` pins only
`crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json`; `tests/golden/**`
is byte-locked with a hash-resync protocol and its guard is specced-but-
unimplemented (ledger row T6), `tests/fixtures/**` is byte-locked, and a NEW data
file would be unpinned under `core.autocrlf=true` -- the F12 trap, in a set whose
purpose is byte exactness. So: no new file, in `tests/golden/**`,
`tests/fixtures/**` or `crates/ui/tests/ui_test/**`.

---

## 6. Refactor and hygiene

- Functions < 100 lines; files < 1000 lines. **Measured at `b0a91d2`:**
  `crates/ui/src/dto.rs` **901** lines, `crates/ui/src/cycle_groups_view.rs`
  **149**, `crates/ui/tests/ui_test/export_order.rs` **446**. Report the exact
  before/after count for all three. `dto.rs` should land near **886**;
  `cycle_groups_view.rs` near **123**. If `dto.rs` lands above **940**, say so
  and stop rather than extracting a module nobody asked for.
- The converted test must stay under the hard 100-line function ceiling; the
  `concat!` const is at module scope and is not part of the function.
- Every `concat!` chunk is one line of at most 96 characters. No authored source
  line may reach 2000 characters.
- No `tmp` / `res` / `foo` / `bar` / `test1`; no single-letter locals in code you
  author; intention-revealing names; one responsibility per function.
- `hygiene_check.py`: zero R1/R2, and **zero NEW R3 warn lines in any file you
  touch** (D2 held that line for `export_order.rs`).
- **Authored content is pure ASCII.** Use `--`, `->`, "sec"; never an em dash,
  arrow glyph or section sign.

---

## 7. Acceptance -- 7 distinct gate commands, 15 planned invocations

Every gate in DIRECT form. Never write a gate as
`python tools/distill.py -- <argv>` unless `<argv>` is a `cargo build` or
`cargo check`. Never use a bare-word test filter -- `testrun.py ... -- <word>`
passes `<word>` to cargo as a TEST-NAME filter and selects zero tests (Phase 1
verdict finding 2); use `--test <target>`.

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui
3. python tools/distill.py -- cargo check -p ariadne-cli --all-targets
4. python tools/distill.py -- cargo build -p ariadne-ui
5. python tools/hygiene_check.py
6. python tools/encoding_guard.py --all        (immediately BEFORE each commit)
   python tools/encoding_guard.py --staged     (once, after the second commit)
7. python tools/commit_scoped.py --scope ... -m ...   (two commits, see 8.2)
```

**Invocation arithmetic, itemised so you can check it rather than trust it:**
gate 1 x4 (capture-and-green, R1, R2, final green), gate 2 x1, gate 3 x2 (base
and final), gate 4 x2 (base and final), gate 5 x1, gate 6 x3, gate 7 x2 (two
commits) = **15 invocations of 7 distinct gate commands**. That exceeds the
sizing law's "~10", and the excess is prescribed evidence, not padding: the
RGB trail and the reverse-dependency checks ARE this unit's deliverable, and each
repeat above has a named reason. Every one of them returns a one-line digest via a
distilling entry point (the sizing axis AGENTS.md names is gate count x output
volume, and the volume here is near zero -- never a raw log). Do not add
invocations beyond these; do not drop any.

**Gate 3 is the systemic lesson of the E blocker, and it is not optional.** A
unit that changes a public surface must build its **reverse dependencies**, not
just the crate it edits. The superseded E built only `crates/ui` and never
type-checked `crates/cli`, which is exactly how a workspace-breaking rename
shipped all-green in a unit and would first have surfaced at T4.1. Run gate 3 at
the untouched base AND after 3.1-3.4; both must be `ok (exit 0)`. E1 deletes a
name `crates/cli` never references (verified 0.3: its only builder reference is
the surviving name), so this gate must be green at both ends -- and if it is not,
that is a finding about the unit, not a gate to drop.
**`ariadne-integration-tests` is deliberately NOT in this unit's gates:** it is a
declared path dependency of `ariadne-ui` (`tests/integration/Cargo.toml:25`) but
has **zero** textual references to either name (verified), so the rename of a
public surface -- not E1's deletion -- is what could break it, and that is E2's
gate. The reviewer runs it in its acceptance pass.

**Gate 4 must report ZERO warnings**, and it is load-bearing for a deletion as
well as for hygiene: it is the only proof that (a) no dangling import survived
the four de-duplications and (b) `from_paths` is actually gone (an un-deleted
`from_paths` is unreachable code in a `pub(crate)` module -- 3.2). Phase 1's
reviewer treated a warning-free build as evidence, not style.

**Gate 6 ordering is pre-authorised:** `commit_scoped.py` stages and commits
atomically and raw git staging is denied, so `--staged` cannot interpose between
staging and committing. Run the strictly stronger `--all` immediately BEFORE each
commit, and the literal `--staged` form once after the second commit. Disclose it
as a pre-authorised substitution. Precedent: T2-A gate 5, T2-B Dev-E, T3-A D5,
T3-B D5, T3-C Dev-3, T3-D Dev-4, T3-D2 Dev-4.

**`theme_token_check.py` is NOT in this unit's gates, and that is a decision,
not an omission:** the touch-set contains no path under `crates/ui/assets/`, so
"zero new hex literals" is **structural** here and the reviewer's
`git diff --name-only` proves it. Run `python tools/theme_token_check.py
--check-added --base b0a91d2` only if you find yourself having touched an asset
file, which the contract forbids.

**Expected verdict lines**

| Gate | Expected verbatim |
|---|---|
| 1 | `186 passed / 0 failed / 0 ignored / 1 suites` (R2: `185 / 1 / 0 / 1`; R1: a compile FAILURE) |
| 2 | `186 passed / 0 failed / 0 ignored / 3 suites` |
| 3 | `ok (exit 0)`, twice |
| 4 | `ok (exit 0, 0 warnings emitted)`, twice |
| 5 | zero R1/R2; any R3 warns are legacy files, none in a file you touched |
| 6 | clean, no output |

The `186` is D2's reviewer-measured gate 1 at `b0a91d2` and was independently
re-measured at that same base by the E journal preflight; gate 2's three suites
are lib + `ui_test` + doc-tests because `crates/ui/src/` carries zero
`#[test]` functions. **Recompute both from your own capture every time.**

**Effect on the workspace line: zero, and you must not measure it.** The last
MEASURED line is `1178/0/4/88` at `aebfa9b`. Unit D2 added +2 (`1180/0/4/88`)
but its reviewer deliberately deferred the workspace-wide measurement to T4.1,
so that number is a **prediction, not a measurement**. E1 adds 0. Do not run the
workspace-wide gate and do not report any workspace number as measured.

Workspace-wide gates are NOT yours: `python tools/testrun.py --baseline`
(BROKEN, ledger F9), `python -m unittest discover -s tools -p "*_tests.py"`,
`python tools/ui_smoke.py`, `python tools/bench_compare.py` (both REFUSED,
ledger F10). Do not work around any of them; never route a refused command
through `distill.py`.

---

## 8. Frozen locks, each with a proving command

| Lock | Proving command | Expected |
|---|---|---|
| exactly one cycle entry point in `crates/ui` | grep `pub fn build_include_elements` | ONE definition, third parameter `&[CycleGroup]` |
| exactly one cycle CURRENCY constructor | grep `from_paths` under `crates/**` | zero hits in `crates/ui/src`; `from_groups` still at `cycle_groups_view.rs:63-88` |
| `cycle_groups_view.rs` untouched apart from `:1-9` and `:26-53` | `git diff` | deletions confined to `from_paths` + its doc; no reformat, no reorder |
| `crates/ui/src/lib.rs` and `crates/ui/tests/ui_test/main.rs` byte-unchanged | `git diff --name-only` | absent |
| the converted P1 has EXACTLY ONE builder invocation and NO second-computation assert | reviewer reads the body | one call; no `assert_eq!(x, x)`; no `contains`-style loosening |
| test COUNT unmoved | gates 1, 2 | `186/0/0/1`, `186/0/0/3` |
| D27 OMITS rule and canonical emission order | gate 1 (`export_order.rs`, `d27_node_provenance.rs` green unedited) | green |
| zero new hex literals, verdict colors intact across all three themes | reviewer's `git diff --name-only` | no `crates/ui/assets/**` path can appear -- the touch-set has none |
| **`crates/cli/**`, `crates/graph/**`, `crates/report/**` byte-unchanged** | `git diff --name-only` | absent |
| `crates/cli` still compiles against `ariadne-ui` | gate 3 | `ok (exit 0)` |
| `tests/fixtures/**`, `tests/golden/**` byte-unchanged, NO new file in either | `git diff --name-only`; no `UPDATE_GOLDENS` run | absent, zero added paths |
| R1/R2 hygiene | gate 5 | no R1/R2; all three files well under 1000 |
| UTF-8 mojibake | gate 6 | clean; authored content pure ASCII |
| touch-set | per-commit `git diff --name-only` | exactly the in-set paths of 2 plus 8.2 commit 1 |
| the F12 fixture path | reviewer | in NO commit (ledger F12) |

### 8.2 Commit scope -- TWO commits, both with explicit `--scope`

`commit_scoped.py` needs an explicit `--scope` per path or a path cannot be
committed; `git add .` is forbidden and the F12 fixture must never be committed.

**Commit 1 -- planner-declared records, OUT OF BAND from your code touch-set.**
The superseded E implementer deliberately left these uncommitted and nobody else
can land them. The reviewer role holds read-only git, so this is the only actor
that can. Precedent: T3-D2 Dev-5. Land the working-tree content **as-is**; do
not re-edit these files.

```
python tools/commit_scoped.py \
  --scope 'tasks/milestone_ledger.md' \
  --scope 'tasks/ledger_archive.md' \
  --scope 'tasks/m235-t3d2-verdict.md' \
  --scope 'tasks/m235-t3e-journal.md' \
  --scope 'tasks/m235-t3e-handoff.md' \
  --scope 'tasks/m235-t3e1-handoff.md' \
  --scope 'tasks/m235-t3e2-handoff.md' \
  -m "docs(tasks): land superseded T3-E contract, its preflight-stop journal, T3-D2 verdict, and the E1/E2 replacement handoffs"
```

What each path is, so you commit it knowingly:

| Path | What it is |
|---|---|
| `tasks/m235-t3e-handoff.md` | the **superseded** 949-line E contract, committed BYTE-UNCHANGED on purpose. It is the record of what was dispatched and rejected; the ledger's M23.5 row and `tasks/milestone_ledger.md` deferred row F14 are the supersession of record. Do not "fix" it and do not delete it. |
| `tasks/m235-t3e-journal.md` | the implementer's preflight STOP: the executed probe (`dto.rs:637` renamed to a throwaway name; `cargo check -p ariadne-cli --all-targets` -> exit 101, `error[E0425]: cannot find function build_include_elements_from_groups in crate ariadne_ui` at `crates/cli/src/commands/ui.rs:328:40`; byte-for-byte revert proven by matching `dto.rs` SHA256). The blocker evidence. |
| `tasks/m235-t3d2-verdict.md` | untracked T3-D2 reviewer record. Its findings F1 and F2 are both discharged: F1 by this unit's re-derived census, F2 by 5.0. |
| `tasks/ledger_archive.md` | already-dirty planner archive move (13/1): the M23.5 companion-parity-test row moved verbatim after D2 was APPROVED. Its trailing sentence still says E is "AUTHORIZED, repaired in place, dispatch-pending"; that sentence is superseded by ledger row F14. **Do not rewrite it** -- archive rows and signed reviewer records are verbatim audit trail, corrected by ledger row, same precedent as the existing `build_include_elements_legacy_paths` row. |
| `tasks/milestone_ledger.md` | planner's row correction: M23.5 milestone row updated, deferred row F14 added. |
| `tasks/m235-t3e1-handoff.md`, `tasks/m235-t3e2-handoff.md` | this contract and its E2 successor. |

**Commit 2 -- the code.**

```
python tools/commit_scoped.py \
  --scope 'crates/ui/src/dto.rs' \
  --scope 'crates/ui/src/cycle_groups_view.rs' \
  --scope 'crates/ui/tests/ui_test/dto_builders.rs' \
  --scope 'crates/ui/tests/ui_test/graph_api.rs' \
  --scope 'crates/ui/tests/ui_test/export_order.rs' \
  --scope 'crates/ui/tests/ui_test/d27_node_provenance.rs' \
  --scope 'crates/ui/tests/ui_test/common.rs' \
  --scope 'tasks/m235-t3e1-journal.md' \
  -m "refactor(ui): retire the path-based include element builder"
```

Eight scopes: seven code paths plus your journal.

---

## 9. Reviewer gate reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
2 and 8.2. The reviewer will audit:

1. **The nine legacy sites were the complete set** -- the journal's R1
   stale-reference compile error is the evidence that no tenth call site exists
   somewhere nobody grepped. If the journal lacks R1, ask for it.
2. **`from_paths` is gone and gate 4 is why.** Zero hits under `crates/ui/src`,
   `from_groups` intact, nothing else in `cycle_groups_view.rs` touched. An
   un-deleted `from_paths`, or `#[allow(dead_code)]` on it, is a finding.
3. **Two doc obligations met, and one NOT over-reached:** `cycle_groups_view.rs:4`
   restated (named a deleted function) and the D27 paragraph moved onto the
   survivor -- while `dto.rs:625-626` was left ALONE, because it is still true in
   E1. Fixing it here is a scope finding.
4. **The four two-name import lists collapsed**, and `common.rs:3` was left alone.
5. **The P1 conversion is a conversion, not a tautology and not a deletion.**
   Exactly one builder invocation; the structural lock deleted rather than
   ported; guards still ahead of the byte comparison; header states where the
   frozen bytes came from.
6. **The expectation was captured, not invented.** Capture procedure recorded,
   temporary line removed, bytes equal the group side's output at `b0a91d2` --
   which the committed parity lock at `42bb44e` proves is also the legacy side's.
   A first-run failure resolved by re-pasting is fine; resolved by weakening the
   assertion is a REJECT.
7. **R2 (the mutation) was really executed**, with the verbatim panic and the
   restore.
8. **Gate 3 ran twice and was green twice.**
9. **Commit 1 contained only `tasks/**`, byte-as-found, and commit 2 only the
   eight in-set code paths.** F12 fixture in neither.

**Report back (final message):** both commit SHAs; the verbatim R1 compile error;
the R2 mutation panic verbatim; Green evidence quoting canonical lines and
capture paths
for gates 1, 2, 3, 4; before/after line counts for `dto.rs`,
`cycle_groups_view.rs`, `export_order.rs`; how the expectation was captured and
whether a first run failed; every deviation with its reason; out-of-scope
observations, report-only. Never paste raw cargo or git output.

---

## 10. Discipline

- **Command execution.** `bash` is an allowlist restricted to the distilling
  entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED.
- **Reading a refusal.** A refused command always reports the catch-all deny
  rule whatever the real cause; treat it as boilerplate. Attempt each gate
  INDEPENDENTLY, never drop a gate because its first invocation was refused, and
  record any gate that cannot run as a VERBATIM refusal in the journal.
- **Strict TDD.** Capture (4.4) -> first green -> R1 (deliberate stale reference)
  -> R2 (mutated expectation) -> revert -> final Green. Never weaken an
  assertion. Never commit a state that does not compile; never commit a Red
  state.
- **The capture line is temporary and not yours to keep.** It exists only long
  enough to read the bytes, writes only into the OS temp directory, and must
  never appear in `git diff --numstat` for `export_order.rs`. Read the capture
  with the Kilo `Read` tool; do not route it through PowerShell.
- **Never edit the frozen expectation to match an output you did not expect.**
  That single rule separates a regression lock from a rubber stamp.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>` or
  `Get-Content | -replace` on a tracked file. Kilo Read/Edit/Write, or byte-level
  `open(path, "wb")`. Pure-ASCII authored content. Quote whole git arguments:
  `git show "rev^:path"`.
- **Scope.** Touch only 2 (and 8.2 commit 1). Out-of-scope breakage: report it,
  do not fix it.
- **Journal.** Append a 1-3 line checkpoint after EACH Green gate. Record your own
  SHAs in it.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A
  dead session is never resumed; a successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3e1-handoff.md --scope ... --cmd ...`.

---

## 11. Sizing (handoff law, stated explicitly)

| Metric | Bound | E1 |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 module directory | **`crates/ui` only -- 1 crate**, 7 code paths + journal (+ 7 `tasks/**` in commit 1, out of band) |
| Gate commands | ~10 | **7 distinct gate commands, 15 planned invocations** (itemised in section 7; the overage is prescribed RGB + reverse-dependency evidence, each with a named reason, every one a one-line digest) |
| Produced lines (implementation + test) | ~1200 | **~250**, itemised below |

Itemised from the bodies this contract mandates -- not from a ledger line:

| Item | Lines |
|---|---|
| `dto.rs`: delete legacy fn + legacy doc paragraph | -19 |
| `dto.rs`: D27 paragraph moved onto the survivor | +4 |
| `cycle_groups_view.rs`: delete `from_paths` + its doc | -28 |
| `cycle_groups_view.rs`: module doc `:4` restated | +2 / -2 |
| nine call sites, callee name only | +9 / -9 |
| four import lists, one name dropped each | +4 / -4 |
| P1 conversion (5.2): new header, guards on one vector, single call, byte lock | +33 / -36 |
| `ACYCLIC_EMPTY_CURRENCY_PAYLOAD`, ~2.5 KB in <=96-char `concat!` chunks | +27 |
| **subtotal, code** | **~+79 / -98 (177 churn)** |
| journal (2 gate-1 states, 2 Reds, capture disclosure, deviations, counts) | ~75 |
| **total** | **~175 net, ~250 churn** |

The standing rule from the ledger's sizing row applies and is honoured here:
every addition is priced from the mandated body, including the ~2.5 KB frozen
expectation, because that is exactly what an estimate that ignores mandated test
content undercounts (T3-D predicted 679 / landed 734; T3-D2 predicted 84 /
landed 167 -- both planner misses, same class).

---

## 12. Authorization and ordering

Authorization: this file, `tasks/m235-kickoff.md`, the M23.5 milestone row in
`tasks/milestone_ledger.md`, and APPROVED `tasks/m235-t3a-handoff.md`,
`tasks/m235-t3b-handoff.md`, `tasks/m235-t3d2-handoff.md` (verdict
`tasks/m235-t3d2-verdict.md`).

```
A   crates/ui           additive DTO entry point     <- done, APPROVED
B   crates/cli          CLI consumers + bench re-pin <- done, APPROVED
C   tests/integration   Scala consumers + IV-9 gate  <- done, APPROVED
D   crates/graph        IV-11 port + ATOMIC DELETE   <- done, APPROVED
D2  crates/ui           legacy/group parity lock     <- done, APPROVED (b0a91d2)
E1  crates/ui           retire the path-based DTO    <- THIS UNIT
E2  crates/ui + cli     rename to the plan's name    <- NOT authorized until E1 is APPROVED
T4.1                    integration closeout         <- NOT authorized
```

**E1 and unit D touch disjoint files** and neither depends on the other, so they
are parallel-safe in separate worktrees; in this shared worktree, dispatch
sequentially. E1 and E2 are **not** parallel-safe -- both touch
`crates/ui/src/dto.rs` and `export_order.rs`.

Do not begin Phase 4, do not run the workspace-wide gates, and do not close
ledger rows F9, F10, F11, F12 or F14 -- those are operator actions at closeout.