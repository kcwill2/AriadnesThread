# M23.5 T3-E2 Handoff - Rename the Surviving Include-Element Builder to the Canonical Name

**Milestone:** M23.5, Phase 3, unit **E2**. **Dispatch base: `5addad7`** (E1's tip).
**Prerequisite: E1 APPROVED** (`tasks/m235-t3e1-verdict.md`).
**Journal:** `tasks/m235-t3e2-journal.md` (you create it).

**Objective.** Rename `build_include_elements_from_groups` to
`build_include_elements` (plan line 69's canonical name, 5-arg shape, groups
third), repoint every caller, and fix the documentation the rename falsifies.
Nothing else changes.

**THE TREE WINS.** Every line reference here was read at `5addad7`. If one
disagrees with the file you open, the FILE is right: record it as a
documentation defect and continue. Never improvise around a mismatch.

---

## 0. Claims the planner verified at `5addad7`, and how

All by direct grep/read of the working tree (bash was refused -- catch-all
boilerplate, see sec9 -- so the `grep`/`read` tools were used instead):

| Claim | How verified |
|---|---|
| Exactly ONE definition, `dto.rs:618`, third param already `&[CycleGroup]` | grep `pub fn build_include_elements` |
| 19 call sites, 5 import lists, 2 doc name-refs, 3 prose refs in `crates/` | grep `build_include_elements` over `crates/` -- **30 hits, arithmetic closed** (0.4) |
| Zero references under `tests/` | grep `build_include_elements\|ariadne_ui` over `tests/` -- no hits |
| Exactly TWO reverse dependencies | grep `ariadne-ui` over `Cargo.toml` -- 3 hits: the package decl plus `crates/cli/Cargo.toml:34` and `tests/integration/Cargo.toml:25` |
| `crates/ui/src/lib.rs:6` is a bare `pub use dto::*;` | read; no alias, no shim |
| Line counts well under 1000 | read each file's total: `dto.rs` 882, `export_order.rs` 476, `dto_builders.rs` 660, `graph_api.rs` 790, `d27_node_provenance.rs` 293, `common.rs` 227, `ui.rs` 766, `payload_tail_bench.rs` 621 |
| `.gitattributes` pins only `crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json` | read the 7-line file -- so the F12 fixture path is unpinned |

**Not run by the planner** (bash denied): every cargo gate. The test counts in
sec7 are E1's reviewer-MEASURED at this exact tip; you re-derive them.

---

## 0.4 PREFLIGHT -- census ABORT GATE

Run these greps over `crates/`, `tests/` and `tools/`. **ABORT on any mismatch.**

| Pattern | Expected at `5addad7` |
|---|---|
| `build_include_elements` | **33 total = 30 in `crates/`, 3 in `tools/`, 0 in `tests/`** |
| `build_include_elements_from_groups\s*\(` | **19**, all call sites |
| `pub fn build_include_elements_from_groups` | **1** |
| `build_include_elements_from_groups` NOT followed by `(` | **5** import lists + **2** doc name-refs |
| `ariadne_ui` | 0 under `tests/`; 2 under `crates/cli` (`src/commands/ui.rs`, `tests/payload_tail_bench.rs`) |
| `ariadne-ui` in any `Cargo.toml` | 3: package decl + the two dependents |

**The 30 in `crates/` MUST decompose as `1 + 19 + 5 + 2 + 3`.** That closed
arithmetic -- not any list in this file -- is the exhaustiveness proof. A
partial enumeration is how this milestone shipped a blocker; if the sum does not
close, or any pattern's total differs, **STOP and report.** Do not widen scope.

**The 3 in `tools/` are bench-baseline KEYS** (`bench_compare.py`,
`bench_baseline.json`, `bench_compare_tests.py`) matching the row label the
bench EMITS. That is why the bench label must not be hand-edited (sec3.3).

Do NOT trust any per-file, per-line call-site list you find in an earlier
handoff, verdict or journal -- including the appendix below. Re-derive from the
tree. The appendix is a CONVENIENCE, not a lock.

<details><summary>Census output at 5addad7 (convenience only)</summary>

```
crates/ui/src/dto.rs:601  doc "group-based sibling of build_include_elements"
crates/ui/src/dto.rs:618  pub fn build_include_elements_from_groups(          <- definition
crates/ui/tests/ui_test/common.rs:3,158
crates/ui/tests/ui_test/d27_node_provenance.rs:9,141,152,174
crates/ui/tests/ui_test/dto_builders.rs:4,15,470,501,588
crates/ui/tests/ui_test/export_order.rs:15,91,108,167,310,317,368,422
crates/ui/tests/ui_test/graph_api.rs:4,136,752,759
crates/cli/src/commands/ui.rs:103,328
crates/cli/tests/payload_tail_bench.rs:41,325,389
tools/: bench_compare.py:53, bench_baseline.json:6, bench_compare_tests.py:37
```
</details>

**0.5 F12 fixture.** `tests/fixtures/basic/artifacts/includes/report.json` is
unpinned by `.gitattributes` under `core.autocrlf=true`, so a test run
re-dirties its stat entry with byte-identical content (ledger F12). Run
`git diff --numstat HEAD -- <path>`. **Only if it prints rows** is content
dirty: restore with the Kilo `Read` then Kilo `Write` tool. NEVER
`Set-Content`/`Out-File`/`>`/`>>`/`Get-Content | -replace`; `git checkout --` /
`git restore` are DENIED. That path appears in **NO** commit.

---

## 1. Touch-set (hard; an edit outside is a blocker -- report, do not fix)

1. `crates/ui/src/dto.rs` -- the definition; doc obligations D1-D4.
2. `crates/ui/tests/ui_test/export_order.rs` -- import list + 6 calls; D5, D6.
3. `crates/ui/tests/ui_test/dto_builders.rs` -- import list + 4 calls.
4. `crates/ui/tests/ui_test/graph_api.rs` -- import list + 3 calls.
5. `crates/ui/tests/ui_test/d27_node_provenance.rs` -- import list + 3 calls.
6. `crates/ui/tests/ui_test/common.rs` -- import list + 1 call. **This file IS
   in the touch-set.** A predecessor contract froze it on the false premise
   that its import already named the survivor; E1's contract-forced edit left
   it naming `build_include_elements_from_groups`, so the rename makes that
   import DANGLING. Obeying a freeze here yields a non-compiling `ui_test`.
7. `crates/cli/src/commands/ui.rs` -- the one production call; no prose edit.
8. `crates/cli/tests/payload_tail_bench.rs` -- the one bench call; D7 only.
9. `tasks/m235-t3e2-journal.md` -- new, yours.

**MUST NOT touch:** `crates/ui/src/lib.rs` (the bare glob re-exports the new
name automatically), `crates/ui/src/cycle_groups_view.rs`, any other
`crates/ui/src/` or `crates/ui/tests/ui_test/` file (incl. `main.rs`),
`crates/ui/assets/**`, any other `crates/cli/**` file, `crates/graph/**`,
`crates/report/**`, `crates/pipeline/**`, `tests/**`, `tools/**`,
`.kilo/plans/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`. No `git stash`,
no branch change, no `git add .`.

**Exception to the one-crate law, declared and justified.** This unit crosses
into `crates/cli`. The boundary is sound in BOTH directions and that was proved
by EXECUTION, not argument: with the definition renamed to a throwaway name and
nothing else touched, `cargo check -p ariadne-cli --all-targets` exited 101 with
`error[E0425]: cannot find function ... in crate ariadne_ui` at
`crates/cli/src/commands/ui.rs` (`tasks/m235-t3e-journal.md`, BLOCKER). E1's
Red below re-executes that probe in the mandated direction.

---

## 2. Green

1. **Rename the definition** in `dto.rs`. The signature is ALREADY canonical --
   third parameter `cycle_groups: &[ariadne_graph::CycleGroup]` -- so only the
   identifier changes. Body, the `emit_include_elements` call and the
   `from_groups` view: byte-unchanged. No reformat of the parameter list.
2. **Repoint every call and every import list.** Callee token / imported name
   only. No argument, no line break, no reformat, no type annotation. The five
   import lists each lose the dead name; nothing else in them moves.
3. **Doc obligations D1-D7** (sec3). All are comments.

**Invariants preserved:** IV-9 determinism, the 5-arg shape, canonical
`(path asc, file_id asc)` emission order, the D27 OMITS rule, node-data keys,
zero new hex literals. **Out of scope, entirely:** any behaviour change, new
test, signature change, emission change, or timing number.

---

## 3. Doc and comment obligations -- by SYMBOL

D1-D7 are MANDATORY. D8-D13 are **NO EDIT**; touching one is a scope finding.

| # | Where (by symbol) | Disposition |
|---|---|---|
| D1 | `dto.rs`, doc on `build_include_elements`: the "group-based **sibling** of `build_include_elements`" sentence | **FIX.** After the rename it names the function as its own sibling. Say it is the ONE include-element entry point in `crates/ui`, the path-based one retired in T3-E by E1; keep the 5-arg shape claim. |
| D2 | `dto.rs`, doc on `emit_include_elements`: "Shared emission tail for **both** include-element entry points" | **FIX.** One entry point survives. |
| D3 | same doc: "The only input differing between the **entry points** is the cycle-classification view" | **FIX.** False. Restate: the view is the layer's only cycle-classification input. Keep the no-duplicated-loop rationale. |
| D4 | `dto.rs`, doc on `build_include_elements` | **ADD** one short line: the third parameter was `&[Vec<FileId>]` (reported DFS path enumeration) before M23.5 and is now `&[CycleGroup]` (the SCC partition), so the next reader does not "restore" the old type. |
| D5 | `export_order.rs`, header of `acyclic_include_elements_match_the_frozen_acyclic_payload` | **FIX** the sentence naming the builder as `build_include_elements_from_groups`. |
| D6 | `export_order.rs`, doc on `the_parity_comparison_detects_a_cycle_classification_difference` | **FIX** it: it names the FORMER P1 name `acyclic_include_elements_are_byte_identical_under_both_cycle_currencies`, which no longer exists, and calls the test below a "parity lock" when P1 is now a frozen-EXPECTATION lock (one live call vs a recorded constant, no second builder). Repoint to the current P1 name and stop calling it a parity lock. Its historical R2 narrative stays -- it is accurate as history. |
| D7 | `payload_tail_bench.rs`, the M23.5 T3.2 re-pin note | **FIX** the clause "`detect_cycles` still exists until the crates/graph unit D deletes it". It is FALSE: unit D landed (`acea7f2`, approved range `adb53a3..aebfa9b`) and deleted the public `detect_cycles()`. State that, name the commit. Touch **no** timing number and no other clause. |
| D8 | `payload_tail_bench.rs`, the emitted bench row LABEL (the `("build_include_elements", ...)` tuple) | **NO EDIT.** It names the timed call and becomes CORRECT by the rename. It is also the key `tools/bench_baseline.json` and `bench_compare.py` match on -- editing it would silently orphan a bench baseline. |
| D9 | `payload_tail_bench.rs`, the header baseline table row `build_include_elements \| 5.4` | **NO EDIT.** Becomes correct by the rename. |
| D10 | `payload_tail_bench.rs`, the sentence naming the `generate_report+detect_cycles` column as the one that "became `generate_report+detect_cycle_groups`" | **NO EDIT.** Both names are correct there: the first is what the column WAS. Historical narration, and true. |
| D11 | `crates/cli/src/commands/ui.rs`, the stage-5 calibration comment row naming `build_include_elements` | **NO EDIT.** Becomes correct by the rename (plan line 129 asked for exactly this). Do NOT reconcile its figure against the bench header's -- that is a perf claim, not a rename. Report the divergence as an observation. |
| D12 | `crates/cli/src/commands/ui.rs`, the `UNVERIFIED against bench_compare.py` paragraph | **MUST SURVIVE VERBATIM.** If your diff removes or weakens it, that is a REJECT. |
| D13 | `cycle_groups_view.rs` module doc; `export_order.rs` / `graph_api.rs` generic "include-element builder" prose | **NO EDIT.** Already accurate; no name referenced; out of touch-set. |

---

## 4. Red -- ONE executed mechanical Red, then revert

**There is NO production-behaviour Red for a rename.** State that plainly in
your journal. Do not manufacture one.

**R1 (the executed Red).** Rename the DEFINITION ONLY. Repoint nothing.
Record `dto.rs`'s SHA256 before the edit.

- Gate 3 MUST FAIL with `error[E0425]`-class evidence naming at least one
  caller **OUTSIDE `crates/ui`** -- the production call in `build_ui_payload`
  and the bench call. **This is the point of the Red:** the predecessor unit
  held only because its gates never built the crate referencing the name it
  deleted. Quote what you actually observe, with its error code and file:line.
  Do not assume the message; run it.
- Gate 1 MUST FAIL too, and its error count is the completeness identity:
  the compiler enumerates the in-crate callers. **`17` in-crate + `2` in
  `crates/cli` must equal the census's `19`.** A different number means the
  census is wrong: THE TREE WINS, report it as a documentation defect, and
  use the compiler's list as the authoritative caller set.
- A compile failure emits no test result line, so `testrun.py` reports its
  parse-miss digest. Record that verbatim; never dress it as `0 failed`.

**R1 revert, byte-for-byte.** Restore `dto.rs` to the pre-rename bytes and
prove it with the SHA256 (identical digests, recorded). A partial revert is
caught by the final Green.

**Never commit a Red state.** Never report `ok` / `0 failed` while describing a
failing state.

---

## 5. Tests

**You change no test.** No test added, removed, renamed, or edited; the five
`crates/ui` test files receive identifier-only edits inside existing bodies.
E1's converted P1 keeps its frozen `ACYCLIC_EMPTY_CURRENCY_PAYLOAD`: its single
call is repointed by identifier only, which is exactly the class of change that
must not alter emitted bytes.

**Frozen on P1:** exactly ONE builder invocation, no `assert_eq!(x, x)`, guards
still AHEAD of the byte comparison, expectation unchanged.

**Counts do not move:** `186 - 0 + 0 = 186`. Re-derive both from your own
capture; never copy a count out of a ledger or another verdict.

---

## 6. Refactor and hygiene

Functions < 100 lines; files < 1000 (sec0 measured all of them; the largest,
`dto.rs` at 882, has ~118 lines of headroom and this unit adds ~10). Report
before/after counts for the five `crates/ui` files and the two `crates/cli`
files. No `tmp`/`res`/`foo`/`bar`/`test1`; no single-letter locals; no
reformatting of lines you touch. `hygiene_check.py`: zero R1/R2 and zero NEW R3
warn lines in any file you touch. **Authored content is pure ASCII** -- use
`--`, `->`, "sec"; never an em dash, arrow glyph or section sign.

---

## 7. Acceptance -- 8 gate commands, 15 planned invocations

Direct form. Never wrap a gate in `distill.py` unless the argv is `cargo build`
or `cargo check`. Never use a bare-word test filter -- `testrun.py ... --
<word>` passes `<word>` to cargo as a NAME filter and selects zero tests; use
`--test <target>`.

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test
2. python tools/testrun.py --distill --timeout 1200 -p ariadne-ui
3. python tools/distill.py -- cargo check -p ariadne-cli --all-targets
4. python tools/distill.py -- cargo check -p ariadne-integration-tests --all-targets
5. python tools/distill.py -- cargo build -p ariadne-ui
6. python tools/hygiene_check.py
7. python tools/encoding_guard.py --all      (immediately BEFORE the commit)
   python tools/encoding_guard.py --staged  (immediately AFTER the commit)
8. python tools/commit_scoped.py --scope ... -m ...
```

**Invocations:** gate 1 x3 (base, R1's compile failure, final), gate 2 x1,
gate 3 x3 (base, R1's compile failure, final), gate 4 x2 (base, final), gate 5
x2 (base, final), gate 6 x1, gate 7 x2, gate 8 x1 = **15 of 8 distinct**. The
overage is prescribed evidence -- the Red, and the base runs that make a final
failure attributable. Every one returns a one-line digest. Do not add; do not
drop.

**Gates 3 and 4 are the systemic requirement.** A unit that renames a public
function must build its REVERSE DEPENDENCIES, not just the crate it edits. The
superseded E built only `crates/ui`, never type-checked `crates/cli`, and would
have shipped a workspace-breaking rename all-green inside a unit. `--all-targets`
is required, not optional: it is what covers the bench target that calls the
renamed function. Gate 4 is a declared path dependency with ZERO textual
references, so it cannot fail on this rename -- run it anyway (ledger F14/2),
and if it cannot complete, record the VERBATIM failure; do not reason that it
"provably cannot fail".

**Gate 5 must report ZERO warnings** -- the proof that no import or reference
was missed.

**Gate 7 ordering is pre-authorised:** `commit_scoped.py` stages and commits
atomically and raw git staging is denied, so `--staged` cannot interpose. Run
the stronger `--all` BEFORE the commit and the literal `--staged` AFTER.
Disclose as a pre-authorised substitution (precedent: T3-D2 Dev-4 and six
others).

**Expected verdict lines**

| Gate | Expected verbatim |
|---|---|
| 1 | `186 passed / 0 failed / 0 ignored / 1 suites` (R1: a compile FAILURE) |
| 2 | `186 passed / 0 failed / 0 ignored / 3 suites` |
| 3 | `ok (exit 0)` at base and final, PLUS R1's `error[E0425]` failure naming an out-of-crate caller |
| 4 | `ok (exit 0)` x2 |
| 5 | `ok (exit 0, 0 warnings emitted)` x2 |
| 6 | zero R1/R2; any R3 warns are legacy files, none in a file you touched |
| 7 | clean, no output |

**Not vacuous:** gate 1 executes 186 with 0 ignored and 1 suite; gate 2's three
suites are lib + `ui_test` + doc-tests; gate 3/4's `--all-targets` covers the
bench. Confirm each from your own capture.

**Effect on the workspace line: zero, and you must NOT measure it.** The last
workspace-wide measurement is `1180/0/4/88` at this tip. Do not run the
workspace-wide gate and do not report any workspace number as measured.
Workspace-wide gates are NOT yours: `testrun.py --baseline` (BROKEN, F9),
`unittest discover -s tools`, `ui_smoke.py`, `bench_compare.py` (REFUSED, F10).
Never route a refused command through `distill.py`.

---

## 8. Commit -- ONE commit, explicit `--scope` on every path

```
python tools/commit_scoped.py \
  --scope 'crates/ui/src/dto.rs' \
  --scope 'crates/ui/tests/ui_test/export_order.rs' \
  --scope 'crates/ui/tests/ui_test/dto_builders.rs' \
  --scope 'crates/ui/tests/ui_test/graph_api.rs' \
  --scope 'crates/ui/tests/ui_test/d27_node_provenance.rs' \
  --scope 'crates/ui/tests/ui_test/common.rs' \
  --scope 'crates/cli/src/commands/ui.rs' \
  --scope 'crates/cli/tests/payload_tail_bench.rs' \
  --scope 'tasks/m235-t3e2-journal.md' \
  -m "refactor(ui,cli): rename the group-based include element builder to the canonical name"
```

Nine scopes, matching sec1 exactly. A path missing from the list cannot be
committed; a listed path you did not touch is a finding. `git add .` is
forbidden; the F12 fixture must never appear.

**Frozen locks, each with a proving command:** exactly one definition, third
param `&[CycleGroup]` (grep `pub fn build_include_elements`); the old name GONE
(grep it over `crates/` `tests/` -- zero hits); `lib.rs` and
`cycle_groups_view.rs` byte-unchanged and absent from `git diff --name-only`;
no test changed (reviewer reads the diff -- identifier-only); counts unmoved
(gates 1, 2); `crates/graph/**`, `crates/report/**`, `crates/pipeline/**`,
`tests/**`, `tools/**` byte-unchanged and absent (this is what keeps the bench
baseline keys valid, D8); D12's paragraph verbatim; touch-set exact per commit;
F12 fixture in NO commit.

---

## 9. Discipline

- **Command execution.** `bash` is an allowlist restricted to the distilling
  entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED.
- **Reading a refusal.** A refused command always reports the catch-all deny
  rule whatever the real cause; treat it as boilerplate. Attempt each gate
  INDEPENDENTLY, never drop a gate because some earlier invocation was refused,
  and record any gate that cannot run as a VERBATIM refusal.
- **Abort, do not judge.** Census mismatch in 0.4 = STOP and report.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>` or
  `Get-Content | -replace` on a tracked file. Kilo Read/Edit/Write, or byte-level
  `open(path, "wb")`. Quote whole git arguments: `git show "rev^:path"`.
- **Scope.** Touch only sec1. Out-of-scope breakage: report it, do not fix it.
- **Journal.** A 1-3 line checkpoint after EACH Green gate; your own SHA in it.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory; a
  successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3e2-handoff.md --scope ... --cmd ...`.

---

## 10. Reviewer gate reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
sec1 and sec8. The reviewer will audit:

1. **The census closed at 1+19+5+2+3 = 30 and the Red's identity closed at
   17+2 = 19.** A rename with no evidence of completeness is this unit's whole
   risk. If the journal lacks R1, ask for it.
2. **R1's compile error names a caller OUTSIDE `crates/ui`**, and the revert is
   byte-for-byte by SHA256.
3. **Gates 3 and 4 ran green at both ends**, gate 5 green with ZERO warnings.
4. **D1-D7 done; D8-D13 untouched.** In particular the bench row label, the
   header table row, and `ui.rs`'s calibration row were NOT hand-edited, and
   D12's `UNVERIFIED against bench_compare.py` paragraph survived verbatim.
5. **The signature did not change** -- third param still `&[CycleGroup]`, 5
   args, body byte-identical. A "helpful" tidy-up is a scope finding.
6. **No test changed; counts did not move.**
7. **The commit contains exactly the nine in-set paths** and no fixture.

**Report back (final message):** the commit SHA; R1's verbatim compile error(s)
and the in-crate error count; the revert SHA256 pair; gates 1-5 verdict lines
with capture paths; before/after line counts for the seven files; every
deviation with its reason; out-of-scope observations, report-only. Never paste
raw cargo or git output.

---

## 11. Authorization and ordering

Authorization: this file, `tasks/m235-kickoff.md`, the M23.5 milestone row in
`tasks/milestone_ledger.md`, plan lines 69 and 129, and APPROVED
`tasks/m235-t3a-handoff.md`, `tasks/m235-t3b-handoff.md`,
`tasks/m235-t3d-handoff.md` + `tasks/m235-t3d-verdict.md`,
`tasks/m235-t3d2-handoff.md` + `tasks/m235-t3d2-verdict.md`, and
`tasks/m235-t3e1-verdict.md`.

```
A   crates/ui           additive DTO entry point     <- done, APPROVED
B   crates/cli          CLI consumers + bench re-pin <- done, APPROVED
C   tests/integration   Scala consumers + IV-9 gate  <- done, APPROVED
D   crates/graph        IV-11 port + ATOMIC DELETE   <- done, APPROVED
D2  crates/ui           legacy/group parity lock     <- done, APPROVED
E1  crates/ui           retire the path-based DTO    <- done, APPROVED (5addad7)
E2  crates/ui + cli     rename to the plan's name    <- THIS UNIT
T4.1                    integration closeout         <- NOT authorized
```

E1 and E2 are **not** parallel-safe: both touch `dto.rs` and `export_order.rs`,
and E2's census is only true after E1 landed. Dispatch sequentially.

Do not begin Phase 4, do not run the workspace-wide gates, and do not close
ledger rows F9, F10, F11, F12 or F14 -- those are operator actions at closeout.