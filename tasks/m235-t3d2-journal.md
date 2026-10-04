# M23.5 T3-D2 Journal - Legacy/Group Include-Builder Parity Lock (crate:ui)

Contract: `tasks/m235-t3d2-handoff.md`. Reviewer: `review-gate` at adjudication.
Dispatch base (my `checkpoint_guard.py status` preflight HEAD): `aebfa9b`,
branch `tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`.

## Checkpoint 0 - preflight (sec0)

- `checkpoint_guard.py status`: HEAD `aebfa9b`, branch `tarjan-s-scc`.
  DIRTY FILES (content numstat): `tasks/milestone_ledger.md` only (7/5) -- a
  pre-existing planner edit adding the D2 authorization rows; outside my
  touch-set, left untouched, in none of my commits. UNTRACKED:
  `tasks/m235-t3d-verdict.md`, `tasks/m235-t3d-execution-results.md`,
  `tasks/m235-t3d2-handoff.md` -- orchestrator-declared into this unit's
  commit scope (dispatch message); landed in commit `4d0f245` (pure additions,
  +2353/-0), exact precedent T3-B `9f30c9a` / T3-D `3e1efa6`.
- F12 fixture `tests/fixtures/basic/artifacts/includes/report.json`: NOT in
  the numstat DIRTY section at preflight nor before commit; content
  byte-equal, so the Read/Write restore was never needed and none was
  attempted. Path appears in NO commit of this unit. No `.gitattributes`
  change. Census `dirty=5`/`dirty=6` entries are the known F12 stat-cache
  plus the ledger edit and untracked records, not content deltas.
- Step 0.3 shape grep `pub fn build_include_elements` under `crates/ui/src`:
  exactly TWO public entry points, as the shape-based preflight predicted:
  - `crates/ui/src/dto.rs:608` `pub fn build_include_elements(graph,
    closures, cycles: &[Vec<FileId>], file_languages, file_unit_kinds)` --
    the legacy path-based entry;
  - `crates/ui/src/dto.rs:637` `pub fn build_include_elements_from_groups(
    graph, closures, cycle_groups: &[CycleGroup], file_languages,
    file_unit_kinds)` -- the group-based sibling.
  Both are 5-arg and share the ONE emission tail `emit_include_elements`
  (`dto.rs:659`). Premise live; no STOP required.
- Doc defect recorded (tree wins, per handoff sec0.3/line-reference rule):
  `tasks/m235-t3d-verdict.md` sec A.5 names the legacy builder
  `build_include_elements_legacy_paths`. That name does not exist in the
  tree at `aebfa9b`; the legacy builder is `build_include_elements` at
  `dto.rs:608`. Correcting source: shape grep of `crates/ui/src` at
  `aebfa9b` (this journal, Checkpoint 0). No function was invented, no
  rename performed. Handoff sec0.3 deliberately grepped by shape and was
  right; the verdict text is what is wrong.
- Entry counts recomputed from the tree, not from any ledger line:
  184 `#[test]`/`#[tokio::test]` across the 19 modules declared in
  `main.rs`; zero `#[ignore]` under `ui_test/`; `crates/ui/Cargo.toml` (26
  lines) declares no `[[bin]]` and no `[[test]]`, so cargo auto-discovers
  lib + `tests/ui_test/main.rs` + doc-tests = 3 suites for gate 2.

## Checkpoint 1 - R1 (P2 only): Green at 185

Gate 1 `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui
--test ui_test`:

    185 passed / 0 failed / 0 ignored / 1 suites

(capture `target/testrun-capture.txt`, 198 lines). P2
`the_parity_comparison_detects_a_cycle_classification_difference` appended;
P1 not yet added.

## Checkpoint 2 - R2 (P1 added + mutation): the Red

P1 `acyclic_include_elements_are_byte_identical_under_both_cycle_currencies`
added green (empty `&[]` currency on both sides), then mutated: the group
builder's third argument replaced in one line (same line count, all line
numbers below unchanged) from `&[],` to
`&cyclic_shuffled_file_graph().detect_cycle_groups(),` -- a real, executed
mutation passing a NON-EMPTY `&[CycleGroup]`. Gate 1 in the mutated state:

    185 passed / 1 failed / 0 ignored / 1 suites

The failing test is P1 itself and the failing assertion is the step-5 BYTE
LOCK (not a precondition), verbatim:

    thread 'export_order::acyclic_include_elements_are_byte_identical_under_both_cycle_currencies' (29552) panicked at crates\ui\tests\ui_test\export_order.rs:433:5:
    assertion `left == right` failed: parity: on acyclic input with empty cycle currency the group-based and legacy builders must emit byte-identical elements

`left` (group-based) carries for nodes 5/11/2 the keys
`"cycle_group_size":3` and `"cycle_witness":["bravo.h","alpha.h","alpha.h",
"bravo.h"]`, `"is_cycle":true` and `"classes":"cycle"` (and edge 5-2 flips to
`is_cycle:true`); `right` (legacy) carries `is_cycle:false`, NEITHER group
key, and no `classes` for the same nodes (capture line 197). The full
serialized blobs exceed the 2000-character source-line rule, so they are
quoted by their identifying keys here and lived in the (ephemeral) capture at
`target/testrun-capture.txt` lines 196-197 while it existed.
P2 passed in this same run; per handoff sec4, this R2 run IS P2's teeth
evidence -- P2's `assert_ne!` and P1's mutated `assert_eq!` are the same
comparison over the same fixture family with opposite expectations. No
independent Red is claimed for P2, and none exists.

## Checkpoint 3 - G (revert + Green) and the revert proof

Revert performed as the exact inverse single-line Edit. Byte-for-byte proof
(content-based, computed twice through the distilling entry point
`python tools/distill.py -- certutil -hashfile ... SHA256`, capture
`target/distill-capture.txt`):

- SHA256 of the green file immediately BEFORE the mutation:
  `35e9ab19bd4e72dec58a8ca9c7b1d46e126247fa4d06a05cad02d5bc0d053000`
- SHA256 of the file immediately AFTER the revert:
  `35e9ab19bd4e72dec58a8ca9c7b1d46e126247fa4d06a05cad02d5bc0d053000`

Identical, so the reverted file is byte-identical to the pre-mutation state
that commit `42bb44e` carries. Gate 1 after the revert:

    186 passed / 0 failed / 0 ignored / 1 suites

Non-vacuity from the green capture: both new tests are named PASSING
(`acyclic_include_elements_are_byte_identical_under_both_cycle_currencies
... ok` line 23; `the_parity_comparison_detects_a_cycle_classification_
difference ... ok` line 25) and `test result: ok. 186 passed; 0 failed;
0 ignored; 0 measured; 0 filtered out` (line 190) -- zero filtered.

## Gates - all verbatim, per handoff sec7

| # | Command | Measured |
|---|---|---|
| 1 (R1/G) | `testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `185 passed / 0 failed / 0 ignored / 1 suites`, then `186 passed / 0 failed / 0 ignored / 1 suites` (R2 mid-state: `185 passed / 1 failed / 0 ignored / 1 suites`) |
| 2 | `testrun.py --distill --timeout 1200 -p ariadne-ui` | `186 passed / 0 failed / 0 ignored / 3 suites`; capture suite headers = lib(0) + ui_test(186) + doc-tests(0), every result line `0 filtered out` |
| 3 | `distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` |
| 4 | `theme_token_check.py --check-added --base aebfa9b` | no output (zero added hex literals); structurally: no `crates/ui/assets/**` path in my diff |
| 5 | `hygiene_check.py` | ZERO R1/R2 lines; every printed line is a pre-existing R3 warn in files I did not touch; ZERO warns in `export_order.rs` (no new R3) |
| 6 | `encoding_guard.py --all` immediately BEFORE each commit; literal `--staged` immediately AFTER | all silent, exit 0 (see Dev-4 ordering disclosure) |
| 7 | `checkpoint_guard.py status` | preflight + before commits + after; before-commit numstat under `crates/**`: `168 1 crates/ui/tests/ui_test/export_order.rs` only |

Capture paths: test gates `target/testrun-capture.txt` (overwritten by each
run; the final copy is gate 2's, whose suite headers are quoted above); build
`target/distill-capture.txt`.

Gate 4 `--base` = `aebfa9b`, my dispatch base per sec7.

## Scope confirmation

- Touch-set: `crates/ui/tests/ui_test/export_order.rs` (the only code file
  touched) + this journal. `crates/ui/src/**` byte-untouched -- confirmed by
  the pre-commit content numstat (only one `crates/**` path listed) and by
  absence from every commit. `dto.rs`, `cycle_groups_view.rs`,
  `graph_api.rs`, `dto_builders.rs`, `main.rs` and every other test module:
  absent from all three commits. The other 184 tests byte-identical: the
  only edit to an existing line is the permitted `use ariadne_graph::{...}`
  list (see Dev-2); everything else is additive hunks.
- Edits to existing lines in `export_order.rs`: exactly the import-list line
  at `:10` (wrapped, see Dev-2). `:12-15` unchanged -- both builders were
  already imported and `CycleGroup` was never named (groups are taken from
  `detect_cycle_groups()`, so no extra import was needed).

## Line counts and sizing (sec6/sec11, reported honestly)

- `export_order.rs`: 279 lines at `aebfa9b` -> 446 lines now (+168/-1 =
  net +167 per numstat; the -1 is the wrapped import line). Far under the
  1000-line ceiling.
- P2 function `:297-~340` (header doc `:282-295`); P1 function `:351-446` =
  96 lines including its 17-line header and closing brace. P1 is under the
  hard 100-line function ceiling. See Dev-3 on the ~40-line readability
  guideline.
- Actual test-code growth is +167 against sec11's itemized +84. Deviation
  Dev-1 with decomposition below.

## Deviations

- Dev-1 (attributed to planner sizing, same defect class T3-D finding 1):
  sec11 priced the two mandated test bodies at +84; the landed diff is
  +167. Decomposition: import wrap +2; P2 54 lines (13-line header the
  contract itself mandates for the shared-teeth statement + 41 body with
  two mandated fixture-sanity asserts and the mandated message-bearing
  `assert_ne!`); P1 114 lines (17-line header mandated by sec5.2 to state
  what P1 does NOT lock + the six sec5.1 steps: acyclicity assert, three
  non-empty strict-subset input maps at rustfmt width, five separate
  non-vacuity asserts with self-explaining messages, byte lock, structural
  lock). Every line is mandated body or a mandated message; zero scope
  creep: exactly two `#[test]` functions, exactly one permitted import-line
  edit, zero helpers. The estimate under-priced the mandated assertion
  messages and the multi-line rustfmt shape of the mandated maps.
- Dev-2 (form, no semantic effect): the `:10` import list became a 3-line
  wrapped block; a single line with `IncludeClosure` is 106 characters and
  would exceed the repo's 100-column rustfmt width (same situation as T3-D
  Dev-2). This is the only edit to any existing line.
- Dev-3 (judgement, disclosed): sec3.2 permits "at most two small fixture
  helpers" but only "if sec5.1's maps cannot be written inline". They can
  be, so none were extracted. P1 is consequently 96 lines, above sec6's
  ~40-line readability guideline; the excess is the mandated step-1..6
  content (acyclicity assert, three input maps, five non-vacuity asserts,
  two locks), none of which belongs in a helper, and sec6's own rule is
  <100 lines, which holds. Extracting the maps would not move any
  comparison out of the test body; the hard ceiling is honored and the
  mutation-trail bytes remain identical to the committed bytes.
- Dev-4 (pre-authorised ordering, not a deviation per sec7): `commit_scoped.
  py` stages and commits atomically and raw git staging is denied, so the
  literal `--staged` form cannot interpose between staging and committing.
  Ran the strictly stronger `encoding_guard.py --all` immediately BEFORE
  each commit and the literal `--staged` immediately AFTER. Precedents:
  T2-A gate 5, T2-B D-E, T3-A D5, T3-B D5, T3-C Dev-3, T3-D Dev-4.
- Dev-5 (orchestrator-declared, not an implementer scope choice): the
  third commit `4d0f245` carries the two T3-D reviewer records plus this
  unit's handoff -- declared into scope by the dispatch message because
  reviewers cannot commit; pure additions, each with an explicit `--scope`.
- Dev-6 (journal fidelity, forced by hygiene): the R2 panic's `left`/`right`
  serialized blobs are ~2.5 KB each; pasting them verbatim would itself
  violate the <2000-character source-line rule, so they are quoted by
  identifying keys plus the capture path and line numbers. The `panicked
  at` and `assertion ... failed:` lines ARE verbatim.

## Out-of-scope observations (report-only, per sec2 -- NOT fixed)

1. `crates/cli/tests/payload_tail_bench.rs:389` still emits the bench row
   label `"build_include_elements"` while the timed call is the group-based
   builder (T3-B finding 4). Verified present at this HEAD. Owner: unit E.
2. `crates/graph/src/types.rs:57-60` / `:66-68` still document
   `informational_cycle_count` in terms of cycles, not groups (T3-D finding
   4). Verified present at this HEAD. Owner: closeout.
3. Pre-existing content-dirty `tasks/milestone_ledger.md` (planner's D2
   authorization rows) at dispatch time; left untouched, in no commit.
4. The two-sample limit on the CLI two-process export lock was NOT re-opened
   anywhere in this unit (settled ACCEPTED by T3-B sec5.5; obligation
   sec5.4 respected -- this journal records no re-argument).

## Refusals encountered

None. Every gate ran on its first independent attempt, including
`certutil` via `distill.py` for the revert hashes. No refused command
occurred in this session; nothing was routed to evade a denial. The
workspace-wide gates (`--baseline`, `unittest discover`, `ui_smoke.py`,
`bench_compare.py`) were NOT run by design -- they are not mine (sec7), and
`ui_smoke.py`/`bench_compare.py` are ledger-refused (F10); F9/F10/F11/F12
stay open for closeout.

## Commits (sec7 two-phase form, explicit `--scope` on every one)

| SHA | Scope | Subject |
|---|---|---|
| `42bb44e` | `crates/ui/tests/ui_test/export_order.rs` only | test(ui): lock legacy/group include-builder parity on acyclic graphs |
| `4d0f245` | the three declared records | docs(tasks): land M23.5 T3-D reviewer records and T3-D2 handoff (orchestrator-declared) |
| (this file) | `tasks/m235-t3d2-journal.md` only | docs(tasks): M23.5 T3-D2 journal |

Canonical workspace effect: PREDICTED `+2` passed (`1180 passed / 0 failed /
4 ignored / 88 suites`); NOT measured here -- the workspace-wide pass belongs
to the reviewer's acceptance pass, per sec7.

## Why this journal cannot contain its own commit SHA

The journal commit (8b) is the commit that carries THIS file, so its SHA
does not exist at the moment the file content is finalized; a commit cannot
contain its own hash (the hash covers the content it would be written into).
The commit list above therefore names `42bb44e` and `4d0f245`, both
predating 8b; the 8b SHA and the unit's range tip are obtained after the
fact with `git log -1`. No amend was performed and none is needed.
