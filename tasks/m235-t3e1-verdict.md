# M23.5 T3-E1 Reviewer Verdict

Unit: `m235-t3e1` (scope `crates/ui`). Base `b0a91d2`, tip `5addad7`, branch `tarjan-s-scc`.
Reviewer: fresh adjudication. Read-only with respect to `crates/**` and `tests/**`.
Write grant used: this file and `tasks/m235-t3e1-execution-results.md` only.

**Method note (recorded because it is evidence).** The first bash invocation of this
session (`Test-Path ...`) and a second (`git status --porcelain`) were both refused with the
verbatim catch-all text:

```
The user has specified a rule which prevents you from using this specific tool call.
Here are some of the relevant rules
{"permission":"bash","pattern":"*","action":"deny","source":"global"}
```

Per the standing instruction this is boilerplate and NOT evidence that bash is inert. It is
not: `python tools/hygiene_check.py` ran on the next attempt, unmodified, without any
wrapping. Every gate below was therefore attempted **independently**, and each refusal is
recorded verbatim in "Gates attempted and not run". No refused command was routed through
`distill.py` to evade a denial. Read-only git was obtained through the `git_git_*` MCP tools
(`log`, `show`, `diff`, `status`), never through bash.

## Verdict

**APPROVED** -- **zero blocking findings against this unit.** Sections A through F each PASS on
independent evidence. Seven numbered findings follow; none blocks T3-E1, but **finding 1 is
BLOCKING for the T3-E2 dispatch**, because `tasks/m235-t3e2-handoff.md` is unsatisfiable as
written in the same shape as the E blocker it replaced. That finding is attributed to the
planner, not to this unit's implementer.

Headline results: the census was re-derived independently and **matched** (arithmetic closed at
20 = 19 calls + 1 definition); the converted test has **exactly one** builder invocation, the
structural `x == x` lock was **deleted not ported**, the four guards sit **ahead** of the byte
lock, and the frozen-expectation form is a **strictly stronger** lock than the self-relative one
it replaced; R1 and R2 were **really executed**; `from_paths` is **gone by deletion** with zero
suppression; the workspace-wide line was **measured** at
`1180 passed / 0 failed / 4 ignored / 88 suites`, confirming D2's `+2` prediction and E1's `+0`.

## Section A -- Preflight census and scope

**Verdict: PASS.** Census independently re-derived and it MATCHED. Touch-set clean in both
commits.

### A.1 Census, re-derived from the tree at the tip (= post-E1; base states cross-checked)

Grep over `crates/` for `build_include_elements(_from_groups)?\s*\(` returns **20** hits. The
arithmetic closes exactly, which is the check that matters:

| | count |
|---|---|
| definitions (`pub fn`) | 1 -- `crates/ui/src/dto.rs:618` |
| call sites, all `..._from_groups` | 19 |
| -- in `crates/ui/tests/ui_test/` | 17 (`graph_api.rs` :136/:752/:759; `export_order.rs` :91/:108/:167/:310/:317/:422; `dto_builders.rs` :15/:470/:501/:588; `d27_node_provenance.rs` :141/:152/:174; `common.rs` :158) |
| -- in `crates/cli` | 2 (`src/commands/ui.rs:328`, `tests/payload_tail_bench.rs:325`) |
| legacy-name (`build_include_elements(`) call sites | **0** |

Base -> tip delta: base was 22 = 20 call sites + 2 definitions (9 legacy + 11 grouped call
sites; legacy def `dto.rs:608`, grouped def `dto.rs:637`). E1 repointed the 9 legacy call
sites onto the grouped name (+9 grouped), deleted the legacy definition (-1 definition), and
the P1 conversion removed P1's second, grouped call (-1 grouped call site). 22 - 1 + 9 - 1 =
**29?** No -- 22 is (20 calls + 2 defs); after repointing the call count is unchanged at 20,
then P1's conversion drops one call -> 19 calls; definitions 2 -> 1. **19 + 1 = 20**, which is
exactly what the grep returns. The census is arithmetically closed, not merely plausible.

`from_paths` under `crates/`: **zero hits**. Confirmed by direct grep (Section D).
`pub fn build_include_elements`: **exactly one** hit, `dto.rs:618`. `crates/ui/src/lib.rs:6`
remains the bare `pub use dto::*;`, so no alias or shim exists.
`tests/` and `tools/`: zero hits for either builder name and zero hits for `ariadne_ui`
(it is a declared path dependency of the integration tests but textually unreferenced --
independently confirmed, see E.1).

The census the implementer reported is therefore **confirmed line-for-line**, including the
STOP-condition items (no tenth legacy site; the two `crates/cli` grouped sites exactly at
`:328` and `:325`; no other crate names either function). No STOP condition fired.

### A.2 Touch-set, per commit (oldest first)

Commit `ab59bb4` (records, out of band per sec8.2): 7 paths, **all under `tasks/`**, all
planner records landed byte-as-found. Verified from `git show ab59bb4`: the diff opens on
`tasks/ledger_archive.md`, then the pure additions `tasks/m235-t3d2-verdict.md` (537/0) and
`tasks/m235-t3e-handoff.md`, and the set is the seven sec8.2 scopes. The ledger_archive hunk
is the M23.5 companion-parity row being archived plus the discharging note; the superseded
E contract is landed as the record of what was dispatched and rejected, unmodified.

Commit `5addad7` (code): `git diff ab59bb4..worktree` returns **exactly 8 paths**:

```
crates/ui/src/cycle_groups_view.rs     6/36
crates/ui/src/dto.rs                   4/23
crates/ui/tests/ui_test/common.rs      3/2
crates/ui/tests/ui_test/d27_node_provenance.rs  3/3
crates/ui/tests/ui_test/dto_builders.rs 5/5
crates/ui/tests/ui_test/export_order.rs 80/50
crates/ui/tests/ui_test/graph_api.rs    3/3
tasks/m235-t3e1-journal.md           348/0
```

Every path is inside the sec2 touch-set plus the journal. Verified ABSENT from both commits:
`crates/cli/**` (all), `crates/graph/**`, `crates/report/**`, `crates/pipeline/**`,
`tests/**` (so `tests/golden/**` and `tests/fixtures/**` are absent and zero added paths),
`tools/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`, `crates/ui/src/lib.rs`,
`crates/ui/tests/ui_test/main.rs`, and any `crates/ui/assets/**` path (which makes "zero new
hex literals" structural, not merely asserted -- see the `theme_token_check.py` adjudication
in the gate section).

Worktree state corroborates the split: `git status` reports no uncommitted change to any
`crates/**` or `tasks/**` path, so HEAD accounts for the whole of both commits. The only
modification is the F12 fixture (see gate 6 / Section D.4) and the only untracked path is
this verdict file.

## Section B -- The anti-tautology invariant

**Verdict: PASS.** The conversion is a conversion, not a tautology and not a deletion, and the
frozen-expectation form is a STRONGER lock than the structural form it replaced.

### B.1 Exactly one builder invocation -- verified by reading the body

`acyclic_include_elements_match_the_frozen_acyclic_payload` at `export_order.rs:391`. Inside
its body (`:391-476`) `build_include_elements_from_groups(` occurs **once**, at `:422`. The
anti-tautology invariant holds: there is no second computation left to compare against.

### B.2 The structural builder-to-builder assert was DELETED, not ported

The pre-E1 step 6 was `assert_eq!(legacy_elements, grouped_elements, "parity as structured
elements, ...")`. The `5addad7` diff removes it and ports nothing in its place. A grep of the
converted body finds **no** `assert_eq!` over two builder results anywhere in the file, so no
`x == x` survives. Nothing was added to "strengthen" the test: the diff contains no new
assertion, no `contains`-style loosening, and no third test.

### B.3 The four non-vacuity guards sit AHEAD of the byte comparison

Order in the body is: fixture acyclicity assert (`:395-398`), input maps (`:404-417`), single
builder call (`:422-428`), then the **four guards** -- `!elements.is_empty()` (`:432-436`),
`nodes_with_closure > 0` (`:450-453`), `nodes_with_language > 0` (`:458-461`),
`nodes_with_language < node_elements.len()` (`:462-465`) -- and only then the byte lock
(`:471-475`). The guards precede the comparison, so "empty payload, therefore equal" cannot
satisfy the lock. Messages were re-worded to the single-vector subject, as sec5.2 step 4
required ("unchanged in substance ... keeping today's messages"); the substance and the
ordering are intact, and the re-wording is the minimum needed to stop the message naming a
builder that no longer exists.

### B.4 P2 survives as a real comparison

`the_parity_comparison_detects_a_cycle_classification_difference` (`:296-331`) keeps its
`assert_ne!` at `:326` and its discriminating shape: `:310` passes `&[]` and `:317` passes
`&groups` to the *same* surviving name, on the *cyclic* fixture. The diff shows exactly one
change to it -- the callee token at `:310`. It remains a genuine cross-currency comparison,
not a self-comparison. This discharges D2 verdict finding F2.

### B.5 Is the frozen-expectation lock STRONGER than what it replaced? Yes -- and here is why

This is the substantive question, so it is answered structurally, not by assertion.

The replaced assertion compared **two live computations** of the same function. Its reach was
exactly the set of inputs both sides agreed on, and it was blind by construction to any change
that moved both sides together: a key rename in emission, a changed `classes` spelling, a
dropped `cycle_group_size` on the ungrouped path, a changed sort key, a changed D27 default --
all of which a self-relative comparison cannot see, because both sides move. Its header said so
itself, and D2's reviewer recorded the same limitation as an observation.

The new assertion compares **one live computation against a recorded constant**. A
single-sided regression now fails it: mutate the emission and the left side moves while the
right side does not. The only class of change it cannot see is one that moves the expectation
and the payload together -- i.e. someone editing `ACYCLIC_EMPTY_CURRENCY_PAYLOAD`, which is
exactly what the frozen-expectation header forbids and what this review verifies.

I verified the constant is a faithful capture rather than an invention, from three independent
directions, none of which trusts the journal:

1. **Internal consistency against the fixture.** `shuffled_file_graph()` builds 10 files and 3
   edges. The constant contains exactly 10 node elements in canonical `(path asc, file_id asc)`
   order -- alpha.h/5, alpha.h/11, bravo.h/2, charlie.h/9, delta.h/1, echo.h/7, foxtrot.cc/3,
   golf.h/8, hotel.h/4, india.cc/6 -- followed by exactly the 3 edges `5-2`, `1-9`, `3-6` in
   source order. Degree tallies agree with the three edges (`in_degree`/`out_degree` per node),
   and edge elements carry only `id`/`is_cycle`/`kind`/`source`/`target`.
2. **Input-map fidelity.** `closures` covers ids 5 and 1 and the constant shows
   `"closure":2` on id 5 and `"closure":1` on id 1, 0 elsewhere. `file_languages` covers 1 and
   3 and the constant shows `"language":"cpp"` on exactly delta.h(1) and foxtrot.cc(3).
   `file_unit_kinds` covers 1 only and the constant shows `"unit_kind":"translation-unit"` on
   delta.h(1) only -- so the D27 OMITS rule is represented on both the present and absent side
   *inside the locked bytes*, which is what makes the lock worth having.
3. **Execution.** Gate 1 green at `186/0/0/1` in this review means the constant equals the live
   output byte-for-byte at the tip. A hand-typed or drifted constant could not pass.

So a real emission regression cannot slip past it. **Caveat, stated rather than hidden:** the
lock is single-input (one fixture, one input shape, empty currency). It does not lock
cross-process determinism, does not cover the cyclic/currency-present path, and does not
replace the two-process CLI lock. The header at `:365-389` says exactly this, names both
determinism homes (`graph_api.rs:741-789` and the CLI
`export_bytes_are_identical_across_two_processes`), and states the human-re-approval rule.
Provenance is stated honestly too: the bytes came from a capture, and the *parity* claim rests
on the committed lock at `42bb44e`, not on the capture.

## Section C -- The mutation Red is real

**Verdict: PASS. Executed, not fabricated.**

- **Genuine one-token change.** Target `translation-unit` -> `transmation-unit` inside the
  constant. I independently confirm the precondition the contract required: `translation-unit`
  occurs **exactly once** in the constant (`:348`), and the second occurrence in the file is at
  `:417`, which is the *input map* (`file_unit_kinds.insert(1, "translation-unit"...)`), not the
  expectation. The count was therefore verifiable before mutating, and the mutation could not
  have hit a second site.
- **Failure at the byte lock, not at a guard and not at compile.** Reported verbatim:
  `panicked at crates\ui\tests\ui_test\export_order.rs:471:5: assertion `left == right` failed:
  frozen payload: on acyclic input with empty cycle currency the surviving builder must emit
  the bytes recorded at b0a91d2`. Independent corroboration from the committed tree, the same
  construction D2's reviewer used: **line 471 column 5 in the reverted, committed file IS the
  byte lock `assert_eq!`**. That coincidence is only possible if the mutation preserved line
  count and line offsets, so the panic line number is itself proof the mutation was confined to
  the constant and that `:471` is the lock. The message names the frozen-payload contract.
  The four guards at `:432-465` sit upstream and all passed under the mutation -- correct, not
  suspicious: they inspect presence/counts of keys, not their spelling.
- **Revert is byte-for-byte.** SHA256 `34ef1356...e2ae2f`, recorded pre-mutation and
  post-revert. I do not need to reproduce it to accept it: the mutation changed one ASCII
  letter inside a string literal, the revert was the exact inverse edit, and gate 1 is green at
  `186/0/0/1` in this review, which means the committed constant equals the live output. A
  partial revert would fail gate 1.
- **R1 is real and is the completeness proof.** `error[E0425]: cannot find function
  `build_include_elements` in this scope` at `export_order.rs:91:20`, with
  `could not compile ariadne-ui (test "ui_test") due to 1 previous error`. Exactly **one**
  error. That is the whole point: after the deletion, with eight of nine sites repointed and one
  deliberately left stale, the compiler named no other unresolved reference. No tenth legacy
  call site exists anywhere in `crates/`, and the deleted definition left no dangling in-crate
  use. R1 was never committed.
- **Post-state locks, independently confirmed at the tip:** `from_paths` -> **zero hits** under
  `crates/`; `pub fn build_include_elements` -> **exactly one** definition, `dto.rs:618`, third
  parameter `cycle_groups: &[ariadne_graph::CycleGroup]` at `:621`. (The handoff predicted
  `:637`; post-deletion drift of 19 lines is expected and the contract's own line references
  are pre-deletion. Tree wins, as instructed.)
- **Digest-shape honesty.** R1's gate reports `DRIFT (exit 2)` because a compile failure emits
  no test result line. The journal records that verbatim rather than dressing it as parse
  drift, and no line anywhere claims `0 failed` while describing a failing state (the T3-B
  finding-1 self-contradiction is absent).

## Section D -- Deletion completeness and the zero-warning requirement

**Verdict: PASS**, with one non-blocking stale-comment finding (D.2).

### D.1 `from_paths` is gone, by deletion, with nothing suppressed

`from_paths` -> zero hits under `crates/`. The `5addad7` diff shows the function, its six-line
doc comment, and the now-redundant blank line removed from `cycle_groups_view.rs`, and
**introduces no `#[allow(dead_code)]`, no `#[allow(unused)]`, no `#![allow]` crate attribute and
no other suppression anywhere in the diff**. That is the right resolution and the contract's
reasoning for mandating deletion over suppression is correct: keeping the view behind an
allow-attribute would leave D1-B's dual representation wearing a silencing attribute.
`from_groups` (`:33`) and everything below it are untouched -- the diff's only other hunk in
that file is the module doc.

### D.2 The module doc no longer names a deleted function

`cycle_groups_view.rs:1-8` now reads "One cycle currency reaches the payload layer since T3-E
retired the legacy reported-path enumeration: the milestone's `CycleGroup` membership
(`from_groups`, ...)". No `from_paths`, no "Two cycle currencies", no "Both entry points". The
plan citation and the byte-shape rationale survive verbatim. Dev-3 (restating across `:3-9`
rather than only `:4`) was **correct and necessary**: `:6-7`/`:9` asserted "Both entry points
in `dto.rs`" and "the two entry points", which 3.1's deletion makes false. Restating only `:4`
would have left two false sentences -- precisely the `build_include_elements_legacy_paths`
defect class the ledger exists to stop. The edit stayed inside the `:1-9` region sec3.2
designates. **Attribution: planner** for the contract's incomplete doc inventory (it named
only `:4` as needing restatement while its own quoted text at `:6-7`/`:9` was already false).

### D.3 Orphaned doc references the deletion left behind -- ONE found, not named by the contract

`crates/ui/src/dto.rs:634-639`, the doc comment on `emit_include_elements`:

- `:634` "Shared emission tail for **both include-element entry points**"
- `:636-638` "The only input differing between the **entry points** is the cycle-classification
  view"

After E1 there is exactly ONE entry point, so both sentences are false, in a file inside the
touch-set, describing the two-representation duality this unit exists to remove. This is the
same defect class as the module-doc obligation the contract *did* name, and the contract's doc
inventory missed it. Non-blocking (a comment, no behavioural or lock consequence), but it should
be carried by E2 alongside the `dto.rs:600-601` obligation E2 already owns. Recorded as finding
3.

Related, and correctly left alone: `dto.rs:599-601` still says "the group-based sibling of
`build_include_elements`", naming a function E1 deleted. The contract sec3.1 **expressly
forbade** fixing it here and assigned it to E2, and the implementer obeyed and disclosed it.
The contract's stated *reason* ("still TRUE after E1, because the two names are still
distinct") is wrong -- after the deletion there is only one name -- but the disposition is
unchanged because E2 rewrites that clause anyway. **Attribution: planner**, non-blocking.

### D.4 No other stale comment; F12 handled correctly

`cycle_groups_view.rs` retains no reference to the removed view constructor. In `dto.rs` the
legacy doc paragraph was deleted with the function and the D27 paragraph was **moved** onto the
survivor (`:599-606`), as sec3.1 required; the moved paragraph is byte-identical in substance.
F12: `git status` shows `tests/fixtures/basic/artifacts/includes/report.json` as modified
**not staged** -- i.e. F12's documented stat-cache churn, content byte-equal (see gate 6 for the
numstat evidence). It appears in **NO** commit of either unit, which is the lock that matters,
and the implementer's preflight correctly did nothing when the numstat printed no rows.

## Section E -- Reverse-dependency discipline

**Verdict: PASS on the enumeration and on gate 3 at both ends. One finding: E2 cannot land as
written (finding 1, blocking for E2, not for E1).**

### E.1 Enumeration, re-derived from every `Cargo.toml`

Grepping `Cargo.toml` for `ariadne-ui` across the workspace returns exactly **three** hits: the
package declaration itself (`crates/ui/Cargo.toml:2`) and **two** dependents --
`crates/cli/Cargo.toml:34` (`ariadne-ui = { path = "../ui" }`) and
`tests/integration/Cargo.toml:25` (`ariadne-ui = { path = "../../crates/ui" }`). No third
consumer exists. `ariadne-cli` has live textual references (lib `:328`, bench `:325`);
`tests/**` has **zero** `ariadne_ui` references, independently confirmed. The implementer's
enumeration is **exactly correct**: two reverse dependencies, one with references, one without.

### E.2 Was deferring `ariadne-integration-tests` to E2 safe?

On the merits, yes, and the reasoning is sound rather than convenient: a crate with zero
textual references to a deleted symbol cannot break from that deletion. `crates/cli` -- the
dependency that *does* reference the surface -- was gated at both ends, which is the systemic
lesson of the E blocker, and gate 3 green twice is the proof.

Against that, standing rule **F14/2** in `tasks/milestone_ledger.md:130-137` says a unit that
changes a public function must gate `-p ariadne-cli --all-targets` **AND**
`-p ariadne-integration-tests --all-targets`, and it names the integration-tests crate
explicitly. E1 did change the public surface (it deleted a `pub fn` reachable through the bare
glob `pub use dto::*;`). The contract did not ignore the rule -- sec7 states the deferral and
its reason in writing -- so this is a disclosed, reasoned narrowing, not an omission. Two
consequences: (a) I ran the integration-tests check myself, in this acceptance pass, so the unit
is not landed on an ungated reverse dependency; (b) F14/2's trigger wording is ambiguous about
*deletions* and should be restated to "adds, removes or renames a public function". Recorded as
finding 4 (planner, non-blocking for E1).

### E.3 Does E1 leave E2 able to land? NO -- `tasks/m235-t3e2-handoff.md` is unsatisfiable

This is the most material result of this review, and it is the same shape as the blocker E1
replaced: **the contract freezes the only repair for damage the previous unit's resolution
makes necessary.** Three concrete defects, all verified against the tree at this tip:

1. **`common.rs` is frozen against a dangling import.** E2 sec2 says "You MUST NOT touch:
   `crates/ui/tests/ui_test/common.rs` (its import at `:3` names only `build_include_elements`,
   which after this rename correctly denotes the group builder -- no edit)", and sec8 freezes
   it "byte-unchanged / absent". That reason is **false at the E1 tip**: E1's Dev-1 resolution
   (correct, and contract-forced -- see F.1) changed `common.rs:3` to name
   `build_include_elements_from_groups`. E2 renames the definition to `build_include_elements`,
   so that import names a symbol that no longer exists. Obeying E2's touch-set yields a
   non-compiling `ui_test`. (`crates/ui/src/lib.rs` has no such problem: the bare glob
   re-exports automatically.)
2. **The 0.3 census count and table are wrong by 8 call sites.** E2 expects "**11**
   `..._from_groups(` call sites"; the true count at the E1 tip is **19** (17 in `ui_test` + 2
   in `crates/cli`) -- I measured it in Section A.1. Its table lists
   `export_order.rs` `:167, :317, :382`; the file actually has six grouped calls at
   `:91, :108, :167, :310, :317, :422`. `:382` does not exist (P1's call moved to `:422` when
   the 25-chunk constant was inserted above it), and `:91/:108/:310/:422` are absent from the
   table. E2's STOP condition ("any count or location differing from the table: STOP") would
   fire on a correct tree.
3. **The sec2 touch-set under-specifies the repoint, systematically, because it inherits E1's
   inverted-name error.** E2 names 3 call sites in `export_order.rs`, 3 in `dto_builders.rs`
   (omitting `:15`), 2 in `graph_api.rs` (omitting `:136`), 1 in `d27_node_provenance.rs`
   (omitting `:141`/`:152`), and none in `common.rs`. The nine sites E1 repointed now carry the
   grouped name and equally need the rename. Obeying sec2 literally leaves six of them calling
   a deleted name. E2's 0.4 "reverse dependencies" enumeration is by contrast **correct**.

Good news for the planner's schedule: all three defects are ordinary compile failures that
E2's own gate 1 and gate 3 would catch immediately, so they cannot reproduce the silent
milestone-level regression of the E blocker. They will, however, cost E2 a full implementer
session to diagnose if not repaired first. **Recommended repair, minimal:** add
`crates/ui/tests/ui_test/common.rs` to E2's touch-set (`:3` import only) and delete the
"byte-unchanged" lock for it; re-derive the 0.3 table to the 19 measured call sites with a
STOP condition matching; extend sec2 item lists to all 19.

## Section F -- Seven deviations adjudicated

Attribution is stated per deviation. Three of them are cases where the CONTRACT was wrong.

**Dev-1 -- `common.rs:3` DOES need an edit. ACCEPTED. Contract defect -> planner.**
The contract asserted twice (sec2 `:125-126`, sec3.4 last row) that `common.rs:3` needs no
edit "because it already imports only `build_include_elements`, which after E1 denotes the
survivor". That is false: E1 does not rename (sec1 says so explicitly), so after E1 the sole
surviving name is `build_include_elements_from_groups` and the import named a deleted symbol.
The implementer's resolution is **correct and minimal** -- one import token, rewrapped to stay
inside the repo's 100-column style (226 -> 227 lines) -- and it is independently proven by R1's
compile error and by gate 1 green. The planner's premise ("after E1 denotes the survivor")
silently assumed the rename lands in E1; sec1 forbids exactly that. **Planner defect,
non-blocking for E1.**
*One imprecision in the implementer's own disclosure:* Dev-1's closing sentence -- "The claim
becomes true only at E2, where `common.rs` is indeed frozen" -- is wrong, and it is the sentence
that hides finding 1. The claim does not become true at E2; at E2 the import is *more* wrong,
and E2's freeze is what makes it unsatisfiable. The implementer identified the edit correctly
and mis-predicted its downstream consequence. Recorded, not charged.

**Dev-2 -- sec3.4's "After E1" column is INVERTED. ACCEPTED. Contract defect -> planner.**
The table says to "drop the grouped name" from the four two-name import lists. Following it
literally would delete the survivor from all four lists and break all grouped call sites. The
implementer dropped the **legacy** name instead, in all four (`export_order.rs:15`,
`dto_builders.rs:4`, `graph_api.rs:4`, `d27_node_provenance.rs:9`) -- verified in the diff,
each list now names exactly one builder. **Correct resolution; planner defect.** The column is
literally true for E2 (after the rename the grouped name is the dead one), which is how the
error survived: a table written for the end state was placed in the unit that does not reach it.
This is the same inversion, and it is what produced E2 defect 3 in E.3.

**Dev-3 -- module doc restated across `:3-9`, not only `:4`. ACCEPTED. Contract defect ->
planner.** Adjudicated in D.2: necessary, not over-reach, and confined to the `:1-9` region the
contract designates.

**Dev-4 -- gate 6 ordering (`--all` before each commit, `--staged` once after commit 2).
ACCEPTED, pre-authorised.** sec7 pre-authorises this substitution with seven named precedents,
`commit_scoped.py` stages and commits atomically, raw git staging is denied, and `--all` is
strictly stronger than `--staged`. Correctly disclosed as a substitution rather than dressed up
as compliance.

**Dev-5 -- R1 reported as `DRIFT (exit 2)`. ACCEPTED.** A compile failure emits no test result
line, so `testrun.py` takes its parse-miss branch. The journal records the digest verbatim,
names the real cause, and points at the 9-line capture holding the error. Reporting a digest
faithfully instead of paraphrasing it into a clean-looking line is correct.

**Dev-6 -- the escaping/splitting helper re-attempted through `distill.py` after a raw script
invocation was refused. ACCEPTED -- legitimate use, NOT an evasion. No finding.**
Judged on the merits and on the letter of the rules:

- *Legitimate.* `distill.py` is documented as an argv pass-through whose purpose is to bound
  output volume (ledger row: "distill.py is an argv pass-through so it bounds OUTPUT not
  command scope"). The helper read one temp file and printed ~25 lines of escaped chunks --
  exactly the output-bound case the distiller exists for. It wrote nothing into the worktree, is
  not in any commit, and touched no tracked file.
- *Not an evasion.* The prohibition is on routing a **refused gate** through `distill.py` to
  get around a denial. The refused command here was not a gate: it was an ad-hoc helper
  invocation, and the contract itself authorises `distill.py` as an entry point and devotes
  sec10 to the distinction. The implementer re-attempted **independently** and through the
  **sanctioned** entry point, and disclosed the refusal verbatim -- the opposite of evasion.
- *Substance is verified independently of the mechanism.* Whether the chunks were produced by a
  helper or typed by hand, the correctness proof is the same and it holds three ways: the
  constant is internally consistent with the fixture and the input maps (B.5), it contains
  `translation-unit` exactly once (C), and gate 1 is green at the tip, which means the
  concatenated literal equals the live emitted bytes. Every chunk line is well inside both the
  96-character chunk bound and the 2000-character source-line rule; the largest is 92
  characters by the implementer's count and the file is 476 lines. The 2000-char observability
  hazard this unit was most exposed to is genuinely discharged.

**Dev-7 -- line counts landed at `dto.rs` 882 (predicted ~886) and `cycle_groups_view.rs` 119
(predicted ~123). ACCEPTED.** I measured `dto.rs` = 882 and `cycle_groups_view.rs` = 119 from
the tree. Both are far under the 1000 ceiling, `dto.rs` nowhere near the 940 stop line, and no
module was extracted. The residual is doc separators and the blank line sec3.2's `-19/-28`
spans did not itemise. **Attribution: planner** (prediction error, third instance of the
sizing-arithmetic class the ledger already tracks). Non-blocking.

**Also recorded as sanctioned, not deviations** (and I agree): the P1 rename as part of the one
sanctioned conversion; the `dto_builders.rs:13` `cycles` -> `cycle_groups` local rename that
sec3.3 expressly permits; the two doc obligations; and commit 1's out-of-band `tasks/**` landing
under sec8.2, which is planner-declared, has T3-D2 Dev-5 precedent, and was necessary because
the reviewer role holds read-only git.

## Scoped gates

Every gate run **directly**, in its own turn, in the form the contract prescribes. All six
pinned lines are confirmed. None is vacuous.

| # | Command | Result, verbatim | Pin | Vacuity check |
|---|---|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `186 passed / 0 failed / 0 ignored / 1 suites` | `186/0/0/1` | not vacuous: 186 executed, **0 ignored**, 1 suite, and `main.rs` is byte-unchanged so `mod export_order;` compiles the converted test |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui` | `186 passed / 0 failed / 0 ignored / 3 suites` | `186/0/0/3` | not vacuous: same 186 across 3 suites (lib + `ui_test` + doc-tests; `crates/ui/src/` carries zero `#[test]`) |
| 3 | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` | `ok (exit 0, 0 warnings emitted)` | `ok (exit 0)` | not vacuous: this is the reverse dependency the E blocker never type-checked; `--all-targets` covers lib, bin and the bench target that calls the survivor at `payload_tail_bench.rs:325` |
| 4 | `python tools/distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` | `ok (exit 0, 0 warnings)` | not vacuous: **the** proof that `from_paths` is really gone (un-deleted it would be `dead_code` in a `pub(crate)` module) and that no dangling import survived the five import-list edits |
| 5 | `python tools/hygiene_check.py` | R1: **0 lines**. R2: **0 lines**. Only R3 warns. | zero R1/R2 | verified structurally, see below |
| 6 | `python tools/encoding_guard.py --all` then `--staged` | no output, both forms | silent | not vacuous: `--all` scans all tracked content, so it covers the pure-ASCII requirement on the new constant and every authored doc line |

**Gate 5 detail.** Rather than trust a count, I verified the property the contract actually
demanded -- "zero NEW R3 warn lines in any file you touch". Not one R3 line names
`export_order.rs`, `cycle_groups_view.rs`, `common.rs`, `dto_builders.rs` or
`d27_node_provenance.rs`. The R3 lines that do land in touched files are `dto.rs:330/:418/:445`
(single-letter `s`) and `graph_api.rs` (`res`), and they are provably pre-existing rather than
merely asserted so: `dto.rs`'s every deletion is at `:604`+, so lines 330/418/445 cannot have
shifted, and `graph_api.rs` is `+3/-3`, so its line numbers are unshifted too. Same line numbers
as the pre-unit tree means same pre-existing warnings. All three files are far under the
1000-line ceiling and the converted test is 86 lines, under the 100-line function bound.

**F12 fixture (ledger row M23.5 F12), re-measured.** `git status` shows
`tests/fixtures/basic/artifacts/includes/report.json` as modified-not-staged, and
`git diff --numstat HEAD -- <path>` prints **zero content rows** -- only
`warning: in the working copy of '...', LF will be replaced by CRLF the next time Git touches it`.
That is F12's documented mechanism exactly (unpinned by `.gitattributes` under
`core.autocrlf=true`), measured again. The lock that matters holds: the path is in **NO** commit
of either unit. No restore is warranted, and per "if and only if it prints rows" none was due.

**`theme_token_check.py` -- the implementer's reasoning HOLDS, and I confirmed it rather than
accepting it.** The contract deliberately omits this gate because the touch-set contains no
`crates/ui/assets/**` path, which makes "zero new hex literals" structural. Two independent
confirmations: the commit diff contains no assets path at all (Section A.2), and running
`python tools/theme_token_check.py --check-added --base b0a91d2` myself returns **no output** --
zero added hex literals across the whole range. The gate was not skipped to save time; it was
skipped because it could not fail, and running it confirms that rather than assuming it.

**One extra gate I added, and why.** `python tools/distill.py -- cargo check
-p ariadne-integration-tests --all-targets` -> `ok (exit 0, 21 warnings emitted)`. E1's contract
deferred this to E2 (Section E.2), but standing rule F14/2 mandates it and E1 did change the
public surface, so the acceptance pass runs it. It is **green**: the deferral was safe in fact,
not merely argued. The 21 warnings are pre-existing in `tests/integration/**`, which E1's
touch-set cannot reach (its only paths are seven `crates/ui` files and a journal); E2's own pin
for this gate is `ok (exit 0)`, not zero warnings.

## Workspace-wide gates

Unlike the earlier test-only units, this one deleted **production** code (`pub fn
build_include_elements`, `IncludeCycleClass::from_paths`) and changed a crate's public surface,
so the workspace-wide pass is NOT deferrable here. It ran. **Verbatim canonical line:**

```
1180 passed / 0 failed / 4 ignored / 88 suites
```

(`### cargo test PASS (exit 0)`, capture `target/testrun-capture.txt`, 2070 lines.)

### Measured delta against the ledger baseline -- and it closes a deferral

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| baseline, measured at `aebfa9b` | 1178 | 0 | 4 | 88 |
| **measured at the E1 tip `5addad7`** | **1180** | **0** | **4** | **88** |
| delta | **+2 / +0 / +0 / +0** | | | |

The arithmetic closes with **zero residual**, and it resolves a standing deferral:

- D2's `+2` was recorded throughout this milestone as a **PREDICTION**, never a measurement
  (`1180/0/4/88`), because D2's reviewer deliberately deferred the workspace measurement to
  T4.1. This run **converts that prediction into a measurement**: 1180 is exactly `1178 + 2`.
  D2's two new tests are confirmed to exist and run in the workspace line, not just in the
  crate-scoped line.
- E1's own contribution is **+0**, as predicted: one test was rewritten and renamed, none added
  or removed. `failed`, `ignored` and `suites` are all unmoved from baseline, which is the
  signal that the deletion removed no test and the conversion removed no suite.
- Ledger row **F11**'s unattributed `+6` Phase-1 residual remains confined to Phase 1 and is
  untouched by this measurement.

Canonical line from the tools suites (what CI runs):

```
Ran 184 tests in 18.423s

OK
```

That matches the `discover Ran 184 tests ... OK` figure recorded in `tasks/ledger_archive.md`.

## Per-commit evidence

Oldest first, each judged alone. Base `b0a91d2`, tip `5addad7`.

**1. `ab59bb4fecaff4d1b7add247b5a008815982515f` -- "docs(tasks): land superseded T3-E
contract, its preflight-stop journal, T3-D2 verdict, and the E1/E2 replacement handoffs"**
Seven paths, **every one under `tasks/`**, and zero outside. Verified from `git show`: the diff
opens on `tasks/ledger_archive.md` (the M23.5 companion-parity row archived verbatim plus the
note discharging it), then the pure additions `tasks/m235-t3d2-verdict.md` (537/0) and
`tasks/m235-t3e-handoff.md`, then `tasks/milestone_ledger.md`, `tasks/m235-t3e-journal.md`,
`tasks/m235-t3e1-handoff.md` and `tasks/m235-t3e2-handoff.md` -- the seven sec8.2 `--scope`s
exactly. Zero code, zero fixture, zero lockfile, zero `crates/**` path.
Commit-vs-touch-set: these seven are **outside** the sec2 code touch-set, but sec8.2 declares
them as planner-declared records landing out of band in their own commit, with T3-D2 Dev-5 as
precedent; the implementer had no choice, since the reviewer role holds read-only git and no
other actor can land them. **Judged APPROVED as a planner-declared action.** One structural
note carried forward as finding 5: this is the third unit in a row needing an out-of-band
`tasks/**` waiver, which is the over-broad-touch-set shape the review-gate commit check exists
to catch; the fix belongs in the handoff template, not in this unit.

Byte-as-found, spot-verified inside the commit: the superseded `tasks/m235-t3e-handoff.md` is
landed as the record of what was dispatched and rejected (its amendment text and the sec13
defect list intact, not rewritten); `tasks/ledger_archive.md`'s trailing sentence still reads
"E is AUTHORIZED, repaired in place, dispatch-pending", which F14 expressly says to leave and
correct by ledger row rather than by rewriting archive text; and `tasks/milestone_ledger.md`
carries the line-width rule at `:17-19`, the updated M23.5 milestone row at `:34`, the
superseded-E disposition at `:43`/`:47` and deferred row F14 at `:86` with its detail sub-block
from `:89`. No ledger line approaches the 2000-character observability limit.

**2. `5addad7e6808e84dcb7025da6a07bfe0ec07016a` -- "refactor(ui): retire the path-based include
element builder"**
Eight paths: the seven in-set code files plus `tasks/m235-t3e1-journal.md` (348/0), matching
the eight sec8.2 `--scope`s exactly. Numstat `6/36 + 4/23 + 3/2 + 3/3 + 5/5 + 80/50 + 3/3`.
Commit-vs-touch-set: **CLEAN** -- no path outside the declared touch-set, no `crates/cli/**`,
no `tests/**`, no `tools/**`, no `Cargo.toml`/`Cargo.lock`/`.gitattributes`, no `assets/**`, no
`crates/ui/src/lib.rs`, no `crates/ui/tests/ui_test/main.rs`, and **no new file** in
`tests/golden/**`, `tests/fixtures/**` or `crates/ui/tests/ui_test/**`.
Every hunk is accounted for: the legacy function and its doc paragraph deleted; the D27
paragraph moved onto the survivor; `from_paths` and its doc deleted; the module doc restated;
five import lists de-duplicated; nine call sites repointed callee-name-only; one local renamed
under the sec3.3 allowance; the constant added at module scope; P1 converted, renamed, its
structural lock deleted and its guards kept upstream of the byte lock. No Red state committed,
no amend, the journal's own SHA correctly absent from its own commit table. **Judged APPROVED.**

**Cross-cutting frozen locks, all verified from the two commit path lists:** exactly one cycle
entry point in `crates/ui` (`dto.rs:618`, third param `&[CycleGroup]`); exactly one cycle
currency constructor (`from_paths` zero hits, `from_groups` intact); `crates/cli/**`,
`crates/graph/**`, `crates/report/**`, `tests/**`, `tools/**` byte-unchanged; the F12 fixture in
neither commit; no `crates/ui/assets/**` path, so zero new hex literals is structural (and
confirmed by running the tool); `crates/cli` still compiles against `ariadne-ui` at the tip; the
D27 OMITS rule and canonical emission order untouched (`export_order.rs` and
`d27_node_provenance.rs` green with only callee-token edits); test counts unmoved at
`186/0/0/1` and `186/0/0/3`.

## Numbered findings

**APPROVED with zero blocking findings against T3-E1.** Findings below are attributed to the
role that caused them. Finding 1 is blocking for the **E2 dispatch**, not for this unit.

1. **BLOCKING for E2 dispatch -- planner. `tasks/m235-t3e2-handoff.md` is unsatisfiable as
   written, in the same shape as the E blocker it replaced.** Three verified defects, detailed
   in E.3: (a) it freezes `crates/ui/tests/ui_test/common.rs` "byte-unchanged" on the false
   premise that its import at `:3` already names `build_include_elements`, when E1's Dev-1
   resolution (contract-forced) left it naming `build_include_elements_from_groups` -- so E2's
   rename would leave a dangling import behind an explicit "MUST NOT touch"; (b) its 0.3 census
   pins **11** `..._from_groups(` call sites and a table listing `export_order.rs:167/:317/:382`
   where the true count is **19** and the true lines are `:91/:108/:167/:310/:317/:422`
   (`:382` does not exist), which trips its own STOP condition on a correct tree; (c) its sec2
   touch-set names only 9 of the 19 call sites needing the rename, because it inherited E1's
   inverted-name error -- it assumes E1's nine repointed sites already carry the plain name.
   All three are ordinary compile failures that E2's own gates 1 and 3 would catch, so they
   cannot reproduce the silent milestone-level regression of the E blocker, but they will cost
   a session. Minimal repair: add `common.rs:3` to E2's touch-set and drop its byte-unchanged
   lock; re-derive the 0.3 table to 19 sites with a matching STOP condition; extend the sec2
   per-file call-site lists to all 19.

2. **Non-blocking for E1, planner. `crates/ui/src/dto.rs:634-639`, the doc comment on
   `emit_include_elements`, is an orphaned reference the deletion left behind** -- "Shared
   emission tail for **both include-element entry points**" and "The only input differing
   between the **entry points** is the cycle-classification view". Both sentences are false
   after E1. The contract's doc inventory named only `cycle_groups_view.rs:4`, so this was not
   in the implementer's scope and its absence is a planner omission, not implementer negligence.
   Carry it into E2 alongside the `dto.rs:600-601` clause E2 already owns. Note also that
   sec3.1's stated reason for deferring `:600-601` ("still TRUE after E1, because the two names
   are still distinct") is itself wrong -- after the deletion there is only one name -- though
   the disposition is unaffected because E2 rewrites that clause anyway.

3. **Non-blocking, planner. Three contract assertions about the tree were false**, all three
   correctly detected by the implementer and disclosed rather than absorbed (Dev-1, Dev-2,
   Dev-3): `common.rs:3` does need an edit; sec3.4's "After E1" column is inverted for E1 (it
   describes E2's end state); and the module doc had to be restated across `:3-9`, not just
   `:4`, because sec3.2's own quoted `:6-7`/`:9` text ("Both entry points", "the two entry
   points") was already false. This is the **fourth** instance of the milestone's named defect
   class (F14, "an unverified claim about the tree"), and the second in this unit alone. The
   implementer's handling is the correct template: re-read the tree, follow it, disclose with a
   correcting source. The planner-side fix is standing rule F14's already-written remedy --
   state every claim about the tree with the command that establishes it -- applied to
   documentation and import-table claims, not only to counts.

4. **Non-blocking, planner. Standing rule F14/2 (ledger `:130-137`) is ambiguous about
   deletions and E1 read it narrowly.** The rule mandates gating both `-p ariadne-cli
   --all-targets` and `-p ariadne-integration-tests --all-targets` for "a unit that changes a
   public function name"; E1 deleted a `pub fn` reachable through the bare glob `pub use
   dto::*;` and deferred the integration-tests gate to E2 on the reasoning that only a *rename*
   can break a crate with zero textual references. That reasoning is **correct on the merits**
   and I confirmed it by running the gate (`ok (exit 0, 21 warnings emitted)`), but the rule's
   trigger wording should read "adds, removes or renames a public function" so the next
   implementer is not left to make that call.

5. **Non-blocking, planner/template. The out-of-band `tasks/**` landing is now the third
   occurrence** (T3-B `9f30c9a`, T3-D `3e1efa6`, T3-D2 `4d0f245`, now `ab59bb4`). It remains
   necessary -- the reviewer role holds read-only git -- but a recurring waiver is a recurring
   hole in the touch-set discipline. Recommend the `handoff-spec` template carry the standing
   clause so no unit needs an ad-hoc exception. This repeats T3-D2 finding F6 unchanged.

6. **Non-blocking, implementer (journal precision, not code).** Dev-1's closing sentence --
   "The claim becomes true only at E2, where `common.rs` is indeed frozen" -- is incorrect, and
   it is the sentence that concealed finding 1: at E2 the import is *more* wrong, not less, and
   E2's freeze is precisely what makes it unsatisfiable. The implementer identified the required
   edit correctly and mis-predicted only its downstream consequence. Recorded because the
   mis-prediction is what a successor reading only the journal would inherit.

7. **Non-blocking, configuration (unchanged).** F12 measured an eighth time: zero content
   numstat rows, stat-cache churn only, path in no commit. F9 reproduced verbatim. F10 refused
   verbatim twice and **remains the most material unclosed gate in this milestone** -- there is
   still zero runtime evidence that the UI cold-boots, routes, serves `Cache-Control: no-store`
   and returns a group-count report, and still no `bench_compare.py` evidence for the meaning
   of the payload timing after T3-D. Both must close before T4.1.

## Gates attempted and not run

**Ran (11 invocations, 9 distinct commands, all direct form, each in its own turn):**
gate 1 `--test ui_test`; gate 2 `-p ariadne-ui`; gate 3 `-p ariadne-cli --all-targets`;
gate 4 `cargo build -p ariadne-ui`; gate 5 `hygiene_check.py`; gate 6 `encoding_guard.py --all`
and `--staged`; `theme_token_check.py --check-added --base b0a91d2` (my addition);
`cargo check -p ariadne-integration-tests --all-targets` (F14/2, my addition);
workspace-wide `testrun.py --distill --timeout 3600`; `unittest discover -s tools`;
`git diff --numstat` for the F12 lock. Plus the session's opening `git status`/`git log`/
`git show`/`git diff` via read-only git MCP tools.

**Refused -- verbatim, catch-all, boilerplate, recorded per instruction:**

- `ui_smoke.py`: `The user has specified a rule which prevents you from using this specific
  tool call. Here are some of the relevant rules
  {"permission":"bash","pattern":"*","action":"deny","source":"global"}` (F10)
- `bench_compare.py`: identical verbatim text (F10)

Neither was retried, worked around, or routed through `distill.py`. Neither was dropped after a
first refusal of some *other* command.

**Broken, not refused -- F9:** `python tools/testrun.py --baseline` -> `usage error: missing
value for --baseline`. Not attempted again, not fixed, not worked around; the workspace pass was
run without `--baseline` and compared by hand against the ledger's baseline chain, which is
exactly the protocol F9 prescribes.

**Also refused at session start, recorded because it is the reason the method note exists:**
`Test-Path -LiteralPath "tasks"; Test-Path -LiteralPath "tasks\m235-t3e1-verdict.md"` and
`git status --porcelain` were both refused with the same catch-all text. Neither caused a gate to
be skipped: `python tools/hygiene_check.py` ran unmodified on the next attempt, and read-only
git was obtained through the `git_git_*` MCP tools. The allowlist is narrow, not inert.