# M23.5 T3-E2 Handoff - Rename the Surviving Include-Element Builder to the Plan's Canonical Name

**Milestone:** M23.5, Phase 3, unit **E2** of 7. Second replacement unit for the
SUPERSEDED `tasks/m235-t3e-handoff.md`.
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec1.3
**line 69** -- the canonical public API is
`crates/ui/src/dto.rs::build_include_elements(graph, closures, cycle_groups: &[CycleGroup], file_languages, file_unit_kinds)`,
the 5-arg shape with groups third -- and sec2 Phase 3 T3.2 **line 129**, which
assigns this very re-point: "update `cli/src/commands/ui.rs` ... and the `:103`
stage-5 timing comment; `payload_tail_bench.rs`".
**Prerequisite: E1 APPROVED.** Do not start before then; E1 deletes the legacy
builder and renames the parity test's body, and this unit edits the same lines.
**Dispatch base:** E1's tip commit, whatever it is. Confirm it with
`checkpoint_guard.py status` and report it -- do not assume a revision.
**Journal:** `tasks/m235-t3e2-journal.md` (you create it).

This unit is small and mechanical on purpose: it exists only because the
canonical public name lives in `crates/ui` and its two callers live in
`crates/cli`, and those two facts cannot be reconciled by a one-crate commit.
Everything else the superseded contract did lives in E1. **If a line reference
disagrees with the file you read, THE FILE WINS** -- report the drift.

---

## 0. PREFLIGHT

**0.1** `python tools/checkpoint_guard.py status`. Record HEAD, branch, dirty and
untracked paths.

**0.2 F12 fixture.** `tests/fixtures/basic/artifacts/includes/report.json` is
unpinned by `.gitattributes` under `core.autocrlf=true`, so a full test run
re-dirties its stat entry with byte-identical content (ledger row M23.5 F12).
Run
`git diff --numstat HEAD -- tests/fixtures/basic/artifacts/includes/report.json`.
**If and only if it prints rows**, restore byte-safely with the Kilo `Read` tool
then the Kilo `Write` tool. NEVER `Set-Content`, `Out-File`, `>`, `>>`,
`Get-Content | -replace`; `git checkout --` / `git restore` are DENIED. If it
prints no rows, do nothing and say so. That path appears in **NO** commit of
either E unit.

**0.3 Caller census -- grep for the name this unit renames.** The preflight that
failed on the superseded E grepped only the legacy name and so missed the
grouped-name hazard entirely. Run:

    build_include_elements_from_groups\s*\(
    build_include_elements\s*\(
    pub fn build_include_elements

Expected after E1, at the E1 tip: **11 `..._from_groups(` call sites and 0 legacy
call sites** (E1 repointed all nine legacy sites at the survivor), of which 9 are
in `crates/ui/tests/ui_test/`:

| File | `..._from_groups(` call lines | import list naming the name |
|---|---|---|
| `crates/ui/tests/ui_test/export_order.rs` | `:167`, `:317`, `:382` | `:15` |
| `crates/ui/tests/ui_test/dto_builders.rs` | `:470`, `:501`, `:588` | `:4` |
| `crates/ui/tests/ui_test/graph_api.rs` | `:752`, `:759` | `:4` |
| `crates/ui/tests/ui_test/d27_node_provenance.rs` | `:174` | `:9` |
| `crates/cli/src/commands/ui.rs` | `:328` (production `build_ui_payload`) | none (fully qualified) |
| `crates/cli/tests/payload_tail_bench.rs` | `:325` (inside `run_assembly`) | none (fully qualified) |

And exactly ONE definition, `crates/ui/src/dto.rs:637`, exported by the bare glob
`pub use dto::*;` at `crates/ui/src/lib.rs:6` -- no alias, no shim, no
`#[deprecated]` re-export. Verify it. Zero call sites in `crates/report`,
`crates/pipeline`, `crates/graph`, `crates/index`, `tools/**`. Zero textual
references to `ariadne_ui` anywhere under `tests/**` (verified), though
`tests/integration/Cargo.toml:25` declares the path dependency.

**STOP conditions.** Any count or location differing from the table: STOP and
report, do not widen scope. In particular a `..._from_groups(` call in any crate
not listed, or a `build_include_elements(` call still present inside
`crates/ui/tests/ui_test/` (which would mean E1 did not complete).

**0.4 Reverse dependencies, enumerated from the tree.** Crates that can see the
API this unit renames: **`ariadne-cli`** (`crates/cli/Cargo.toml:34`, the only
crate with live textual references -- lib and bench/test targets) and
**`ariadne-integration-tests`** (`tests/integration/Cargo.toml:25`, declared path
dependency, no textual references today). Both are gated in section 7.

---

## 1. Objective and invariants

**Objective.** Rename `build_include_elements_from_groups` to
`build_include_elements` so the public API matches plan line 69, repoint all 11
call sites, and fix the documentation the rename makes false. Zero behaviour
change: a rename plus two doc-clause corrections.

**Exception to the one-crate handoff law, stated and justified.** This unit
touches `crates/ui` (the definition and nine test call sites) **and**
`crates/cli` (`:328` production, `:325` bench). The law permits one crate because
a cross-crate edit usually means one crate's handoff boundary is unbuildable.
Here the boundary is provably sound in BOTH directions, and that was proved by
execution, not by argument: the superseded E's implementer renamed
`dto.rs:637` to a throwaway name with everything else untouched and
`cargo check -p ariadne-cli --all-targets` failed with exit 101 --
`error[E0425]: cannot find function build_include_elements_from_groups in crate
ariadne_ui` at `crates/cli/src/commands/ui.rs:328:40` -- then reverted byte for
byte (`tasks/m235-t3e-journal.md`, BLOCKER). The proof is twofold: the reverse
direction (rename without the two cli repoints) breaks the workspace, and the
forward direction (E1's deletion, name untouched) does not. Two crates with two
lines of cross-boundary edit are therefore the minimum, and section 7's reverse
dependency gates make the failure mode an ordinary gate failure instead of a
silent milestone-level regression.

**Invariants preserved.** IV-9 / determinism, the 5-arg shape, the third
parameter's type, canonical `(path asc, file_id asc)` emission order, the D27
OMITS rule, node-data keys, zero new hex literals: all untouched. Nothing but an
identifier changes.

**Explicitly NOT in this unit:** any behaviour change, any new test, any
signature change, any emission change, and E1's deletions (already landed).

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

1. `crates/ui/src/dto.rs` -- rename the definition at `:637`; the two doc
   obligations in 3.1.
2. `crates/ui/tests/ui_test/export_order.rs` -- import list `:15`; calls `:167`,
   `:317`, `:382`.
3. `crates/ui/tests/ui_test/dto_builders.rs` -- import list `:4`; calls `:470`,
   `:501`, `:588`.
4. `crates/ui/tests/ui_test/graph_api.rs` -- import list `:4`; calls `:752`, `:759`.
5. `crates/ui/tests/ui_test/d27_node_provenance.rs` -- import list `:9`; call `:174`.
6. `crates/cli/src/commands/ui.rs` -- **line 328 only**, the callee token in
   `build_ui_payload`'s single call. The `:103` stage-5 timing comment needs NO
   edit (see 3.3). The odd indentation of `let node_maps` at `:323` is
   pre-existing: leave it, report it if you wish.
7. `crates/cli/tests/payload_tail_bench.rs` -- **line 325** (the callee token in
   `run_assembly`) and **the single stale clause at `:64`** (see 3.3). Nothing
   else in this file, and no timing number anywhere.
8. `tasks/m235-t3e2-journal.md` -- **new**, yours.

**You MUST NOT touch:** `crates/ui/tests/ui_test/common.rs` (its import at `:3`
names only `build_include_elements`, which after this rename correctly denotes the
group builder -- no edit), `crates/ui/src/cycle_groups_view.rs`, any other file
under `crates/ui/src/` or `crates/ui/tests/ui_test/` (including `main.rs`),
anything under `crates/ui/assets/`, any other file under `crates/cli/**` (notably
`crates/cli/src/commands/includes.rs` and every other bench), `crates/graph/**`,
`crates/report/**`, `tests/**`, `tools/**`, `.kilo/plans/**`, `Cargo.toml`,
`Cargo.lock`, `.gitattributes`. No `git stash`, no branch change, no `git add .`.

---

## 3. Green -- the minimal implementation bound

### 3.1 Rename the definition, and fix the doc the rename makes false

`pub fn build_include_elements_from_groups` at `dto.rs:637` becomes
`pub fn build_include_elements`. Its third parameter is already
`cycle_groups: &[ariadne_graph::CycleGroup]` at `:640` -- exactly the plan's
canonical third position -- so the **signature does not change**; only the
identifier does. The body, the `emit_include_elements` call at `:644` and the
`from_groups` view at `:647` are byte-unchanged. Do not reformat the parameter
list.

Two doc obligations, both mandatory, both the same defect class (a document that
references the renamed name):

- **`dto.rs:625-626` MUST change.** It reads "the group-based sibling of
  `build_include_elements`, keeping the exact 5-argument D27 shape", which after
  the rename names the function as its own sibling. Replace it with wording
  stating that this is the only include-element entry point in `crates/ui`, the
  path-based one having been retired in T3-E (by E1), and keep the shape claim.
- **Add a short history note:** the third parameter was `&[Vec<FileId>]`
  (reported DFS path enumeration) before M23.5 and is now `&[CycleGroup]` (the
  SCC partition), so the next reader does not "restore" the old type. E1 already
  moved the D27 paragraph here; leave it where E1 put it.

Keep `:627-636` (the IV-2/IV-3/IV-5 semantics and the D27 OMITS rule for the two
group keys) unchanged. `crates/ui/src/lib.rs` needs **no** edit: `pub use dto::*;`
at `:6` re-exports the new name automatically.

### 3.2 Repoint the eleven call sites -- callee token only

Every site in the 0.3 table changes the callee identifier and nothing else. No
argument, no line break, no reformat, no type annotation. The four import lists
collapse to one name each (drop the grouped name, which no longer exists); a
duplicate-name import is a compile error, but a silently rewritten import list is
a scope finding, so change only the name.

`crates/cli/src/commands/ui.rs:328` and
`crates/cli/tests/payload_tail_bench.rs:325` are fully qualified calls
(`ariadne_ui::build_include_elements_from_groups(`) -- change the identifier
inside the qualification and nothing else on the line.

### 3.3 The three `crates/cli` prose sites -- decision

Verified at the tree, one by one:

- **`payload_tail_bench.rs:389`, the bench row LABEL
  `("build_include_elements", include_elements_ms)`:** the timed call at `:325`
  is the grouped builder, so this label is wrong today and becomes **correct** by
  the rename. **No edit.** This is the T3-B reviewer finding 4 that the
  superseded E predicted "this unit makes that label accidentally correct" --
  do not touch it, and do not let its correctness imply that the number behind it
  was re-verified.
- **`payload_tail_bench.rs:41`, the header table row
  `build_include_elements | 5.4`:** same situation, becomes correct by the
  rename. **No edit.**
- **`crates/cli/src/commands/ui.rs:103`, the stage-5 calibration comment
  `build_include_elements 5.1ms`:** same situation, becomes correct by the
  rename, which is precisely what plan line 129 asked for ("and the `:103`
  stage-5 timing comment"). **No edit, and do not reconcile 5.1 against the
  bench's 5.4** -- those are a 2026-09-29 A4 calibration table and a 2026-10-04
  re-pinned bench table for the same call; reconciling them is a perf claim,
  not a rename, and `bench_compare.py` is REFUSED for this role (ledger F10).
  The `:115-117` paragraph reading "UNVERIFIED against bench_compare.py: that
  tool is off the reviewer allowlist (ledger M23.5 F10) and was not run; 5.6 ms
  is a single local measurement, not bench-compare-verified perf evidence" **must
  survive verbatim.** If your diff removes or weakens that sentence, it is a
  REJECT.
- **The one prose edit you DO make:** `payload_tail_bench.rs:64` reads
  "`detect_cycles` still exists until the crates/graph unit D deletes it". That
  is now **false**: unit D landed and deleted the public `detect_cycles()`
  (approved `adb53a3..aebfa9b`; the only surviving hits of that string in the
  workspace are this comment, a frozen `old_detect_cycles` oracle copy in
  `crates/graph/tests/`, and unrelated `detect_cycles_in_units` in
  `crates/transform`). Fix **that clause only** -- state that unit D deleted it,
  name the commit -- and touch **no** timing number, no other clause, and not the
  `:62` sentence. The surrounding re-pin note keeps its UNVERIFIED labelling
  exactly as it stands; the 5.6 ms figure behind it is still a single local
  measurement and must still read as one.

---

## 4. Red -- two executed demonstrations

There is **no production-behaviour Red** for a rename, and you should state that
plainly in your journal. What replaces it is two real, executed states of
section 7's reverse-dependency gate:

| # | What you do | What you must record |
|---|---|---|
| R1 | With the definition renamed (3.1) and all nine `crates/ui` call sites repointed, deliberately leave **`crates/cli/src/commands/ui.rs:328`** still calling the old name; run gate 3 | the compile FAILURE verbatim, with its `error[E0xxx]` code and file:line |
| R2 | repoint `:328` and `:325`; run gate 3 | `ok (exit 0)` |

**R1 is the superseding contract's blocker, re-executed as a gate.** The
expected message is the one the E implementer captured by probe:
`error[E0425]: cannot find function build_include_elements_from_groups in crate
ariadne_ui` at `crates/cli/src/commands/ui.rs:328:40` -- but **quote what you
actually observe**, and if the error names a different site or a different code,
that is your completeness evidence and you report it verbatim rather than
paraphrasing. Do not assume the message; run it.

- **Never commit a Red state.**
- **Never write a line that reports `0 failed` / `ok` while describing a failing
  state** (T3-B verdict finding 1).
- If R2's `cargo check -p ariadne-cli --all-targets` is green but a *later* gate
  reveals a missed site, that is a finding, not a gate to drop.

---

## 5. Tests

**You change no test.** No test is added, removed, renamed, or edited; the four
`crates/ui` test files receive identifier-only edits inside existing bodies
(callee token, import list). E1's converted parity test
`acyclic_include_elements_match_the_frozen_acyclic_payload` keeps its frozen
`ACYCLIC_EMPTY_CURRENCY_PAYLOAD` expectation: its single call at
`export_order.rs:382` is repointed by identifier only, which is exactly the kind
of change that must not alter emitted bytes.

**Counts do not move:** gate 1 stays `186 passed / 0 failed / 0 ignored /
1 suites` and gate 2 stays `186 passed / 0 failed / 0 ignored / 3 suites` (186 is
D2's reviewer-measured value at `b0a91d2`, re-measured by the E journal preflight
and by E1 at the E1 tip). This unit changes `186 - 0 + 0 = 186`. **Re-derive both
from your own capture; never copy a count out of a ledger or another verdict.**

**Frozen locks on the tests:** the converted P1 still contains exactly ONE
builder invocation, no `assert_eq!(x, x)`, and its guards still precede the byte
comparison. A test-body change of any kind, including "fixing" the import line
beyond dropping the dead name, is a finding.

---

## 6. Refactor and hygiene

- Functions < 100 lines, files < 1000 lines. Nothing you touch should change size
  by more than the two doc lines you add. Report before/after line counts for
  `crates/ui/src/dto.rs`, `crates/ui/tests/ui_test/export_order.rs`,
  `crates/cli/src/commands/ui.rs` and `crates/cli/tests/payload_tail_bench.rs`.
- No `tmp` / `res` / `foo` / `bar` / `test1`; no single-letter locals in code you
  author; no reformatting of the lines you touch.
- `hygiene_check.py`: zero R1/R2, and zero NEW R3 warn lines in any file you
  touch.
- **Authored content is pure ASCII.** Use `--`, `->`, "sec".

---

## 7. Acceptance -- 8 distinct gate commands, 14 planned invocations

Every gate in DIRECT form. Never write a gate as
`python tools/distill.py -- <argv>` unless `<argv>` is a `cargo build` or
`cargo check`. Never use a bare-word test filter; use `--test <target>` (Phase 1
verdict finding 2).

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui
3. python tools/distill.py -- cargo check -p ariadne-cli --all-targets
4. python tools/distill.py -- cargo check -p ariadne-integration-tests --all-targets
5. python tools/distill.py -- cargo build -p ariadne-ui
6. python tools/hygiene_check.py
7. python tools/encoding_guard.py --all        (immediately BEFORE the commit)
   python tools/encoding_guard.py --staged     (immediately AFTER the commit)
8. python tools/commit_scoped.py --scope ... -m ...
```

**Invocation arithmetic, itemised so you can check it rather than trust it:**
gate 1 x2 (green after the rename, final after refactor), gate 2 x1, gate 3 x3
(base, R1's expected failure, R2), gate 4 x2 (base and final), gate 5 x2 (base
and final), gate 6 x1, gate 7 x2, gate 8 x1 = **14 invocations of 8 distinct
gate commands**. The overage above "~10" is prescribed evidence, not padding: the
two Reds and the four base-state runs are what make this rename trustworthy, and
every invocation returns a one-line digest through a distilling entry point (the
sizing axis AGENTS.md names is gate count x output volume). Do not add
invocations; do not drop any.

**Gates 3 and 4 are the systemic requirement, not a formality.** A unit that
changes a public function name must build its **reverse dependencies**, not just
the crate it edits. The superseded E built only `crates/ui`, never type-checked
`crates/cli`, and would therefore have shipped a workspace-breaking rename
all-green inside a unit -- a silent milestone-level regression first visible at
T4.1. The two crates that can see this rename are enumerated in 0.4. Run both at
the untouched base AND after the rename; all four invocations must be `ok (exit 0)`.
If gate 4 cannot complete inside your timeout, record the VERBATIM failure or
timeout in your journal -- do not substitute a narrower target and do not reason
that the gate "provably cannot fail".

**Gate 5 must report ZERO warnings.** It is the proof that no dangling import or
missed reference survived in `crates/ui`.

**Gate 7's ordering is pre-authorised:** `commit_scoped.py` stages and commits
atomically and raw git staging is denied, so `--staged` cannot interpose. Run the
strictly stronger `--all` immediately BEFORE the commit and the literal `--staged`
immediately AFTER. Disclose it as a pre-authorised substitution. Precedent: T2-A
gate 5, T2-B Dev-E, T3-A D5, T3-B D5, T3-C Dev-3, T3-D Dev-4, T3-D2 Dev-4.

**Expected verdict lines**

| Gate | Expected verbatim |
|---|---|
| 1 | `186 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `186 passed / 0 failed / 0 ignored / 3 suites` |
| 3 | `ok (exit 0)` x2, PLUS R1's expected compile FAILURE (recorded as R1, not as a gate failure) |
| 4 | `ok (exit 0)` x2 |
| 5 | `ok (exit 0, 0 warnings emitted)` x2 |
| 6 | zero R1/R2; any R3 warns are legacy files, none in a file you touched |
| 7 | clean, no output |

**Effect on the workspace line: zero, and you must not measure it.** The last
MEASURED line is `1178/0/4/88` at `aebfa9b`; D2's `+2` (`1180/0/4/88`) is a
**prediction**, not a measurement (its reviewer deferred the workspace-wide
measurement to T4.1). E1 and E2 both add 0 tests. Do not run the workspace-wide
gate and do not report any workspace number as measured.

Workspace-wide gates are NOT yours: `python tools/testrun.py --baseline`
(BROKEN, ledger F9), `python -m unittest discover -s tools -p "*_tests.py"`,
`python tools/ui_smoke.py`, `python tools/bench_compare.py` (both REFUSED,
ledger F10). Do not work around any of them; never route a refused command
through `distill.py`.

---

## 8. Frozen locks, each with a proving command

| Lock | Proving command | Expected |
|---|---|---|
| the public API is the plan's canonical 5-arg name | grep `pub fn build_include_elements` | ONE definition at `dto.rs:637`, third parameter `&[CycleGroup]` |
| the old name is GONE from the workspace | grep `build_include_elements_from_groups` over `crates/` and `tests/` | ZERO hits -- including the two `crates/cli` lines |
| `crates/ui/src/lib.rs` byte-unchanged | `git diff --name-only` | absent |
| `crates/ui/src/cycle_groups_view.rs` byte-unchanged | same | absent |
| `crates/ui/tests/ui_test/common.rs` byte-unchanged | same | absent |
| `crates/ui/tests/ui_test/main.rs` byte-unchanged | same | absent |
| no test changed | reviewer's `git diff` on the four `crates/ui` test files | identifier-only edits in the enumerated lines; no assertion, no body, no name changed |
| test COUNT unmoved | gates 1, 2 | `186/0/0/1`, `186/0/0/3` |
| zero new hex literals, verdict colors intact across all three themes | `git diff --name-only` has no `crates/ui/assets/**` path | structural, and reported |
| `crates/cli/**` changed ONLY at `:328` and `:64`/`:325` | reviewer's `git diff` on `crates/cli/**` | exactly three lines |
| the `UNVERIFIED against bench_compare.py` paragraph | reviewer reads `ui.rs:108-119` | verbatim, unweakened |
| `crates/graph/**`, `crates/report/**`, `crates/pipeline/**`, `tests/**`, `tools/**` byte-unchanged | `git diff --name-only` | absent |
| R1/R2 hygiene | gate 6 | no R1/R2 |
| UTF-8 mojibake | gate 7 | clean; authored content pure ASCII |
| touch-set | per-commit `git diff --name-only` | exactly the eight in-set paths |
| the F12 fixture path | reviewer | in NO commit (ledger F12) |

### 8.1 Commit -- ONE commit, explicit `--scope`

```
python tools/commit_scoped.py \
  --scope 'crates/ui/src/dto.rs' \
  --scope 'crates/ui/tests/ui_test/export_order.rs' \
  --scope 'crates/ui/tests/ui_test/dto_builders.rs' \
  --scope 'crates/ui/tests/ui_test/graph_api.rs' \
  --scope 'crates/ui/tests/ui_test/d27_node_provenance.rs' \
  --scope 'crates/cli/src/commands/ui.rs' \
  --scope 'crates/cli/tests/payload_tail_bench.rs' \
  --scope 'tasks/m235-t3e2-journal.md' \
  -m "refactor(ui,cli): rename the group-based include element builder to the canonical name"
```

Eight scopes, matching 2 exactly. A path missing from the list cannot be
committed; a path present that you did not touch is a finding. `git add .` is
forbidden and the F12 fixture must never appear in any commit.

---

## 9. Reviewer gate reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
2 and 8.1. The reviewer will audit:

1. **The eleven call sites were the complete set**, evidenced by R1's verbatim
   compile error. If the journal lacks R1, ask for it: a rename with no evidence
   of completeness is this unit's whole risk.
2. **Gate 3 ran three times (once as R1's expected compile failure, green twice)
   and gate 4 twice, green twice each** -- including
   `ariadne-integration-tests`, which the superseded E's gate set did not even
   name. An unrun reverse-dependency gate is a finding.
3. **The signature did not change** -- third parameter still
   `cycle_groups: &[CycleGroup]` at `dto.rs:640`, 5 args, body and
   `emit_include_elements` call byte-identical. A "helpful" signature or
   formatting tidy-up is a scope finding.
4. **`dto.rs:625-626` was rewritten** (it named the function as its own sibling)
   and the history note was added, while `:627-636` was preserved.
5. **`crates/cli` changed on exactly three lines**: `ui.rs:328`,
   `payload_tail_bench.rs:325`, `payload_tail_bench.rs:64`. The `:389` row label,
   the `:41` header row and `ui.rs:103` were NOT edited -- they become correct
   by the rename, and editing them would be a scope finding. The
   `UNVERIFIED against bench_compare.py` paragraph survived verbatim.
6. **No test changed**, and the counts did not move.
7. **The commit contains exactly the eight in-set paths** and no fixture.

**Report back (final message):** the commit SHA; R1's verbatim compile error; R2's
verdict line; gate 1, 2, 3, 4 and 5 verdict lines with capture paths;
before/after line counts for the four named files; your dispatch base revision;
every deviation with its reason; out-of-scope observations, report-only. Never
paste raw cargo or git output.

---

## 10. Discipline

- **Command execution.** `bash` is an allowlist restricted to the distilling
  entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED.
- **Reading a refusal.** A refused command always reports the catch-all deny rule
  whatever the real cause; treat it as boilerplate. Attempt each gate
  INDEPENDENTLY, never drop a gate because its first invocation was refused, and
  record any gate that cannot run as a VERBATIM refusal in the journal.
- **Strict TDD.** R1 (deliberate un-repointed cli line -> compile failure) -> R2
  (repointed -> `ok`) -> final gates -> Refactor. Never weaken anything. Never
  commit a Red state.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>` or
  `Get-Content | -replace` on a tracked file. Kilo Read/Edit/Write, or byte-level
  `open(path, "wb")`. Pure-ASCII authored content. Quote whole git arguments:
  `git show "rev^:path"`.
- **Scope.** Touch only 2. Out-of-scope breakage: report it, do not fix it.
- **Journal.** Append a 1-3 line checkpoint after EACH Green gate. Record your own
  SHA in it.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A
  dead session is never resumed; a successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3e2-handoff.md --scope ... --cmd ...`.

---

## 11. Sizing (handoff law, stated explicitly)

| Metric | Bound | E2 |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 module directory | **2 crates -- a DECLARED EXCEPTION**, justified in section 1 with the executed probe as evidence. 7 code paths + journal |
| Gate commands | ~10 | **8 distinct gate commands, 14 planned invocations** (itemised in section 7; overage is prescribed Red + reverse-dependency evidence, each a one-line digest) |
| Produced lines (implementation + test) | ~1200 | **~70**, itemised below |

| Item | Lines |
|---|---|
| `dto.rs`: rename definition (1 line) | +1 / -1 |
| `dto.rs`: rewrite doc `:625-626`, add history note | +4 / -2 |
| 11 call sites, callee token only | +11 / -11 |
| 4 import lists, one name dropped each | +4 / -4 |
| `payload_tail_bench.rs:64` stale clause | +2 / -2 |
| **subtotal, code** | **~+22 / -20 (42 churn)** |
| journal | ~50 |
| **total** | **~72 net, ~92 churn** |

Priced from the mandated edits, per the standing ledger rule on sizing
arithmetic: every addition is itemised, and this unit's additions are two doc
sentences plus a two-line comment correction -- which is exactly why splitting it
out of E1 was worth a second unit.

---

## 12. Authorization and ordering

Authorization: this file, `tasks/m235-kickoff.md`, the M23.5 milestone row in
`tasks/milestone_ledger.md`, plan lines 69 and 129, and APPROVED
`tasks/m235-t3a-handoff.md`, `tasks/m235-t3b-handoff.md`,
`tasks/m235-t3d2-handoff.md` (`tasks/m235-t3d2-verdict.md`), and **E1's
reviewer verdict**.

```
A   crates/ui           additive DTO entry point     <- done, APPROVED
B   crates/cli          CLI consumers + bench re-pin <- done, APPROVED
C   tests/integration   Scala consumers + IV-9 gate  <- done, APPROVED
D   crates/graph        IV-11 port + ATOMIC DELETE   <- done, APPROVED
D2  crates/ui           legacy/group parity lock     <- done, APPROVED (b0a91d2)
E1  crates/ui           retire the path-based DTO    <- must be APPROVED first
E2  crates/ui + cli     rename to the plan's name    <- THIS UNIT
T4.1                    integration closeout         <- NOT authorized
```

**E1 and E2 are NOT parallel-safe:** both touch `crates/ui/src/dto.rs` and
`crates/ui/tests/ui_test/export_order.rs`, and E2's 0.3 census is only true after
E1 has landed. In this shared worktree, dispatch strictly sequentially.

Do not begin Phase 4, do not run the workspace-wide gates, and do not close
ledger rows F9, F10, F11, F12 or F14 -- those are operator actions at closeout.
Do not touch `crates/graph/**`, `crates/report/**`, `tests/**` or `tools/**`.