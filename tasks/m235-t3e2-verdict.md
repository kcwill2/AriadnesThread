# M23.5 T3-E2 Reviewer Verdict (public-API rename)

Reviewer: reviewer subagent (read-only with respect to `crates/**`, `tests/**`, `tools/**`).
Worktree: `C:\Users\Kurt\Code\AriadnesThread\.kilo\worktrees\tarjan-s-scc`.
Dispatch base `5addad7` (E1 tip) -> HEAD `dcd681b`. Commits: `378b240`, `4bc3b88`, `dcd681b`.

**Method note (evidence, not excuse).** `rg --version`, `rg -c ... crates tests tools` and
`git grep -c ... 5addad7` were each refused, verbatim and identically:

```
The user has specified a rule which prevents you from using this specific tool call.
Here are some of the relevant rules
{"permission":"bash","pattern":"*","action":"deny","source":"global"}
```

Each was attempted independently, none retried, and **no refused command was routed through
`distill.py`**. This is the catch-all boilerplate, not evidence that bash is inert: `python
tools/checkpoint_guard.py status`, `git show`, `git show --stat`, and `git log --oneline` all
ran unmodified in this same session. Census work was done with the dedicated `grep`/`read`
tools and read-only git MCP tools instead.

## Verdict

**APPROVED.** Zero blocking findings against this unit. Sections A through E each PASS on
independent evidence. Fourteen numbered findings follow; none blocks T3-E2. Nine of the
fourteen are attributed to the **planner**, not to this unit's implementer, and one of those
(the unobservable error-count prediction, Dev-1) is the fifth instance of this milestone's
named defect class F14.

The unit did exactly what the contract asked and nothing else: one identifier renamed at one
definition, 19 call sites and 5 import lists repointed callee-only, 7 mandated doc fixes
(D1-D7), 6 no-edit doc items and 3 `tools/` bench-baseline keys byte-unchanged, no test added
or removed, no behaviour changed, and 19 call sites touched in a commit whose payload I read
line by line.

## Section A -- census closed sum (exhaustiveness proof)

**Verdict: PASS. Independently re-derived; the closed sum holds and the reported wording slip
is in the CONTRACT'S WORDING, not in its arithmetic.**

### A.1 At HEAD, measured directly from the tree

`build_include_elements` over `crates/` returns **29** lines; over `tools/` **3**; over
`tests/` **0**. Decomposition at HEAD:

| category | count | locations (verified line by line) |
|---|---|---|
| definition | **1** | `crates/ui/src/dto.rs:621` `pub fn build_include_elements(` |
| call sites | **19** | `cli/src/commands/ui.rs:328`; `cli/tests/payload_tail_bench.rs:326`; `common.rs:158`; `d27_node_provenance.rs:141/:152/:174`; `dto_builders.rs:15/:470/:501/:588`; `export_order.rs:91/:108/:167/:311/:318/:423`; `graph_api.rs:136/:752/:759` |
| import lists | **5** | `common.rs:3`, `d27_node_provenance.rs:9`, `dto_builders.rs:4`, `export_order.rs:15`, `graph_api.rs:4` |
| doc name-refs | **1** | `export_order.rs:369` |
| prose / label | **3** | `payload_tail_bench.rs:41`, `:390`; `cli/src/commands/ui.rs:103` |
| | **29** | 1 + 19 + 5 + 1 + 3 |

**The delta from the base figure of 30 is exactly one line, and it is explained, not
residual:** `dto.rs:601` spelled the PLAIN name in the D1 sentence and D1 replaced that
sentence, so the token no longer occurs there. Every other line is accounted for.

### A.2 At the dispatch base `5addad7`, reconstructed from commit `4bc3b88`'s removed lines

The diff's deleted side supplies the base text, so the base census is verifiable without
`git grep`: base `dto.rs` held `the group-based sibling of` / `` `build_include_elements` ``
(the plain-name doc ref) and `export_order.rs` held
`` `build_include_elements_from_groups` must emit exactly the recorded bytes `` (the grouped
doc ref). Therefore base = **1 + 19 + 5 + 2 + 3 = 30** in `crates/`, **33** total with the
3 `tools/` keys (`bench_compare.py:53`, `bench_baseline.json:6`, `bench_compare_tests.py:37`,
all three still present at those exact lines at HEAD), **0** in `tests/`.

**The arithmetic closes. Confirmed.**

### A.3 The wording slip -- attributed to the contract, and harmless to the sum

Contract sec0.4's row reads: pattern ``build_include_elements_from_groups`` NOT followed by
`(`, expected **5 import lists + 2 doc name-refs = 7**. The implementer reports **6**. Both are
right, because the pattern and the category disagree: `export_order.rs:368` spelled the
grouped name and matches the pattern (1), the 5 import lists match (5) = **6**, while
`dto.rs:601` spelled the **plain** name and cannot match a pattern that requires the
`_from_groups` suffix.

**Adjudication: the slip is in the row's wording, not in the arithmetic.** The sum that
sec0.4 itself designates as the exhaustiveness proof is expressed over the UNION pattern
`build_include_elements`, under which `dto.rs:601` counts correctly as a doc name-ref. The
implementer's own reconciliation is the right one and is stated in the journal: 10 non-paren
lines = 5 import + 2 doc + 3 prose. No STOP condition should have fired and none did.
**Attribution: planner, non-blocking.** This is the fifth contract claim about the tree that
did not survive contact with it.

## Section B -- rename completeness and behaviour preservation

**Verdict: PASS. Complete, and behaviour-preserving -- confirmed by reading the payloads, not
the report.**

### B.1 Zero surviving occurrences of the old name

`build_include_elements_from_groups` over `crates/`, `tests/` and `tools/`: **zero hits**. It
survives only in `tasks/**` records and `.kilo/plans/**`, which is correct and expected.

### B.2 Exactly one definition, canonical signature

`crates/ui/src/dto.rs:621` `pub fn build_include_elements(` -- **exactly one**. Read at HEAD:
third parameter `cycle_groups: &[ariadne_graph::CycleGroup]` at **:624**; five parameters
total (`graph`, `closures`, `cycle_groups`, `file_languages`, `file_unit_kinds`); return
`Vec<CytoscapeElement>`. Plan line 69's 5-arg shape with groups third is satisfied verbatim.
The body is a single `emit_include_elements(...)` call passing
`cycle_groups_view::IncludeCycleClass::from_groups(cycle_groups, graph)` -- in the diff this
body is pure CONTEXT, unchanged. Function length 15 lines (`:621-635`); `dto.rs` totals 885
lines against the 1000 ceiling.

`crates/ui/src/lib.rs` needed no edit and got none: the bare `pub use dto::*;` re-exports the
new name automatically, and `lib.rs` appears in **no** commit.

### B.3 Rename plus comments only -- no behavioural diff

Every hunk of `4bc3b88`, read individually:

- `cli/src/commands/ui.rs`: one line, callee token only. The 5 arguments are CONTEXT.
- `cli/tests/payload_tail_bench.rs`: one callee-token line, plus the D7 doc clause.
- `ui/src/dto.rs`: the definition's identifier, plus three doc comments (D1/D4, D2, D3). The
  signature's parameter lines and the entire body are CONTEXT.
- the five `ui_test` files: 5 import lists and 19 call sites, callee token only, plus D5 and
  D6 doc comments.

**No argument, no type, no line break, no emission order, no key spelling and no assertion
moved.** The 5-argument shape, the canonical `(path asc, file_id asc)` emission order and the
D27 OMITS rule are untouched (they are CONTEXT lines in the diff). This is a rename.

## Section C -- the compile Red, and the unobservable error-count prediction

**Verdict: PASS on the Red. Dev-1 is a CONTRACT DEFECT of the fifth kind, attributed to
planner. The implementer's substitute completeness argument is SOUND -- I say so explicitly,
because the alternative reading is that the rename shipped unproven.**

### C.1 Was the Red real? YES, and its decisive property is independently corroborated

The contract required R1's gate 3 to name a caller **outside `crates/ui`**. Reported
verbatim: `error[E0425]: cannot find function `build_include_elements_from_groups` in crate
`ariadne_ui`` at `crates\cli\src\commands\ui.rs:328:40`, with `cargo build FAIL (exit 101)`.

I cannot re-execute this Red (it requires editing `dto.rs`, which my write grant forbids), so
I corroborated it structurally, and the corroboration is unusually tight:

1. The diff proves that **before** the rename `ui.rs:328` read
   `let include_elements = ariadne_ui::build_include_elements_from_groups(` and that the
   definition in `dto.rs` was `pub fn build_include_elements_from_groups(`. Renaming the
   definition alone therefore leaves exactly that call resolving against a crate that no
   longer exports the name -- which is E0425 with that exact message shape.
2. **The quoted column is exact.** I counted it: `ui.rs:328` is 4 spaces + `let ` (cols 5-8) +
   `include_elements` (cols 9-24) + ` = ` (25-27) + `ariadne_ui` (28-37) + `::` (38-39), so the
   callee identifier begins at **column 40**. The report's `328:40` is correct to the
   character, which is not what a paraphrase or a reconstruction from the handoff produces.
3. The named caller is the **production** call in `build_ui_payload`, i.e. precisely the
   out-of-crate caller whose absence from the predecessor E blocker's gate set was the
   milestone-level regression. **The Red did the job the contract designed it to do.**

I accept R1 as real, correctly aimed, and correctly reported. The digest was also reported
honestly: a compile failure emits no test result line, so `testrun.py` returned its parse-miss
`DRIFT (exit 2)`, recorded verbatim and never dressed as `0 failed`. That matches the T3-E1
Dev-5 precedent and avoids the T3-B self-contradiction.

### C.2 Is the predicted error-count identity a contract defect? YES -- planner

Contract sec4 predicted gate 1's Red would enumerate the in-crate callers and that
**17 in-crate + 2 cli = 19** would close. Observed instead: **5 errors, all `E0432`
`unresolved import`**, one per import-list file, closing with `could not compile ariadne-ui
(test "ui_test") due to 5 previous errors`.

**This is structurally unobtainable, not merely miscounted.** Every in-crate caller reaches
the builder through a `use ariadne_ui::{...}` import list, so the first resolution failure is
the import itself; rustc then reports the import errors and stops, never reaching the call
sites. For the predicted 17 `E0425`s to appear, the import lists would first have to be
repointed -- but sec4 R1 mandates "rename the DEFINITION ONLY. Repoint nothing." **The
contract specified a procedure whose own step ordering makes its own acceptance criterion
unobservable.** That is a contract defect, not an implementer failure, and it is the fifth
instance of F14 ("an unverified claim about the tree") extended from claims about the tree to
a claim about compiler output. **Attribution: planner.** Remedy is F14's own written remedy:
state every predicted output with the command that establishes it, or do not pin it.

### C.3 Is the substitute completeness argument sufficient? YES

The implementer does not rely on the impossible count. It substitutes two arguments, and both
hold independently:

1. **File-granularity corroboration.** The 5 `E0432` files are *exactly* the census's 5
   import-list files (`common.rs`, `d27_node_provenance.rs`, `dto_builders.rs`,
   `export_order.rs`, `graph_api.rs`). A missed repoint in any of those files would have
   produced an error that is not in the list; a missed repoint in a file with no import list
   would have been the class of defect the E blocker shipped, and is excluded by (2).
2. **The final Green, which is the real proof.** Gate 1 green at `186 passed / 0 failed / 0
   ignored / 1 suites` **with the frozen byte constant** `ACYCLIC_EMPTY_CURRENCY_PAYLOAD`
   asserted against live output, plus gate 3 green at zero warnings and gate 5 green at zero
   warnings. A single unrepointed call site is a hard compile error; a single mis-repointed
   one changes emitted bytes and fails P1. There is no silent-degradation path. **This
   argument is strictly stronger than the identity the contract asked for**, because the
   identity would only have shown the error *count* matched, whereas the Green shows every
   one of the 19 sites both compiles and emits the locked bytes.

**No stronger demonstration is required, and I record that as a judgement rather than a
hedge.** The only thing the contract's identity would have added is a per-call-site
enumeration that the compiler declines to produce.

### C.4 The byte-for-byte revert

Reported SHA256 `804680f3cf2e728d456b9730691cc4b61929489e58afd0a9efc4547092b15d94` for
`crates/ui/src/dto.rs`, identical pre-Red and post-revert. **Accepted, and it does not need
reproducing to be believed:** the R1 edit was one identifier on one line and the revert its
exact inverse, and the final gate 1 is green against a frozen byte constant, which a partial
revert could not survive. The digest is recorded in full in the journal in both positions.
I do note the implementer reached `certutil` **through `distill.py`** -- a sanctioned entry
point used for output bounding on a non-gate helper, disclosed, and not a refused gate routed
around a denial. **No finding.**

## Section D -- "do not edit" list respected; contested judgments

**Verdict: PASS. All six NO-EDIT items are byte-unchanged, the three `tools/` keys are
untouched, and both contested judgments are correct. This is the section where the trap was
and it was not sprung.**

### D.1 The NO-EDIT list, verified byte-unchanged

Proven by absence from `4bc3b88`'s diff plus a direct read at HEAD. None of these lines
appears as a `+`/`-` anywhere in the unit's three commits:

| item | site | evidence |
|---|---|---|
| D8 bench row LABEL | `payload_tail_bench.rs:390` `("build_include_elements", include_elements_ms)` | not in diff; read at HEAD, byte-identical (it sat at `:389` at base; the +1 is D7's rewrap above it) |
| D9 header table row | `payload_tail_bench.rs:41` `//! build_include_elements                           \|      5.4` | not in diff; read at HEAD |
| D10 `generate_report+detect_cycles` sentence | `payload_tail_bench.rs:62-63` | not in diff; read at HEAD, both names intact as history |
| D11 stage-5 calibration row | `cli/src/commands/ui.rs:103` `generate_report+detect_cycle_groups 5.6ms · build_include_elements 5.1ms ·` | the only `ui.rs` hunk in the commit is `:325`; read at HEAD |
| D12 UNVERIFIED paragraph | `payload_tail_bench.rs:80-87` **and** `cli/src/commands/ui.rs:115-119` | neither hunk touches either range; both read at HEAD |
| D13 generic prose + `cycle_groups_view.rs` | module doc, `export_order.rs`/`graph_api.rs` prose | file absent from all three commits |

**D12 is the load-bearing one** and it survived at **both** sites, including the
`bench_compare.py` refusal and the "not routed through `distill.py` to evade the denial"
sentence. Given that F10 is still open, a weakened D12 would have removed the only in-tree
statement that this milestone has no verified perf evidence. It is intact.

### D.2 The `tools/` bench-baseline keys -- untouched, which is what protects the bench

No `tools/**` path appears in **any** of the three commits (`378b240` = 3 `tasks/` paths;
`4bc3b88` = the 8 code paths + journal; `dcd681b` = journal only). The three keys are still at
`bench_compare.py:53`, `bench_baseline.json:6`, `bench_compare_tests.py:37`. **Because D8 was
left alone, the emitted label and the keys that match on it remain in agreement**; editing
the label would have orphaned a baseline silently, with exit 0 and no failing gate in this
unit. The trap was real and was not sprung.

### D.3 The items that DID require fixing -- all seven done, and D7's citation is TRUE

D1 (`dto.rs:599-604`, the "group-based sibling" sentence -> "the ONE include-element entry
point in `crates/ui` (the path-based one was retired in T3-E by E1)", 5-arg claim kept); D2
(`:637`, "both include-element entry points" -> "the one include-element entry point"); D3
(`:639`, "The only input differing between the entry points is..." -> "The
cycle-classification view is this layer's only cycle-classification input", no-duplicated-loop
rationale kept); D4 (`:602-604`, the third parameter "was `&[Vec<FileId>]` ... and is now
`&[CycleGroup>` ... must not be \"restored\""); D5 (`export_order.rs:369`, header names the
builder by its current name); D6 (P2's doc at `:283-296`, former P1 name -> current P1 name,
"parity lock" -> "frozen-expectation lock"/"the frozen expectation honest", historical R2
narrative byte-unchanged); D7 (`payload_tail_bench.rs:64-65`).

**D7's commit citation is independently verified and is CORRECT**, which is worth recording
because contract claims about the tree have been wrong four times already:
`git show --stat acea7f2` returns `refactor(graph)!: delete detect_cycles/dfs_cycle, repoint
last callers` touching `crates/graph/src/graph.rs`, and `git log --oneline adb53a3..aebfa9b`
confirms the approved range. The D7 replacement text also **balances its parentheses**: the
opening `(` before "the SCC replacement" is closed by the retained trailing `)`, and the
inserted `detect_cycles()` is self-balanced. No timing number moved (`5.6`, `5.8`, `4.7` all
CONTEXT).

### D.4 The two contested judgments -- I AGREE WITH BOTH

**`:62` needs no edit -- correct.** `payload_tail_bench.rs:62-63` reads "the
`generate_report+detect_cycles` combined column became `generate_report+detect_cycle_groups`".
Both names are correct in that sentence: the first is what the column **was**. It is
historical narration of a re-pin, and rewriting it would destroy the only record that the
column was ever named anything else. Untouched. ✓

**`:63-64` must be fixed -- correct, and this is the sharper half of the pair.** The adjacent
clause asserted "`detect_cycles` **still exists until** the crates/graph unit D deletes it".
Unit D landed; `detect_cycles()` is **gone** (verified: `acea7f2` deletes it, and the diff at
`ui.rs:322` shows the crate calling `detect_cycle_groups()`). The clause is not merely stale
narration, it is a **live false statement about the present**, and it would have licensed a
future reader to restore `detect_cycles`. Fixed with a verifiable citation. ✓

**`ui.rs:103` needs no edit -- correct.** It names `build_include_elements`, which the rename
makes **true**; nothing about it is falsified by this unit. The residual `5.1` vs the bench
header's `5.4` for the same phase is a perf claim, and reconciling a perf claim inside a
rename with F10 still open would manufacture exactly the unverified timing evidence this
milestone has been disciplined about refusing. Correctly reported as observation O-1 and left
alone. **I agree, and I would have rejected a "helpful" reconciliation.**

## Section E -- no test added/removed; P1 lock and P2 assert survive

**Verdict: PASS. A rename is exactly where a lock gets quietly weakened; these were not.**

### E.1 No test count moved, structurally

The diff of `4bc3b88` contains **no `#[test]` line added or removed and no `fn` test
declaration added, removed or renamed** in any of the five `ui_test` files. The test names
`acyclic_include_elements_match_the_frozen_acyclic_payload` and
`the_parity_comparison_detects_a_cycle_classification_difference` are unchanged -- and sec5
forbids renaming a test, so leaving `the_parity_comparison_...` alone despite its now-stale
"parity" wording was **required**, not an oversight. Confirmed by measurement in the gate
section: `186` at base and `186` at final, `186 - 0 + 0 = 186`.

### E.2 P1's frozen lock survived intact -- all four properties

1. **The constant is untouched.** `ACYCLIC_EMPTY_CURRENCY_PAYLOAD` (`export_order.rs:338`) is
   pure CONTEXT in the diff. Not one byte of the frozen expectation changed.
2. **Single-builder-invocation holds.** Inside P1's body the builder occurs **once**, at
   `:423`. There is no second computation to compare against, so the anti-tautology invariant
   is intact -- no `assert_eq!(x, x)` was reintroduced.
3. **Guards remain AHEAD of the byte lock.** `:433` `!elements.is_empty()`, `:451`
   `nodes_with_closure > 0`, then `nodes_with_language`, then the byte comparison at `:471`.
   "Empty payload, therefore equal" still cannot satisfy the lock.
4. **The input maps are unchanged**, including `file_unit_kinds.insert(1,
   "translation-unit")` at `:418`, so the D27 OMITS rule is still represented on both the
   present and absent side inside the locked bytes.

### E.3 P2's `assert_ne!` survived and is still a real comparison

`assert_ne!` intact at `:327`, with the same discriminating shape: `:311` passes `&[]` and
`:318` passes `&groups` to the *same* surviving name on the cyclic fixture. Both calls were
repointed callee-only. It remains a genuine cross-currency comparison, not a self-comparison.

### E.4 The one internal tension in the contract, correctly resolved

sec5 says "no test ... edited"; sec3 D5/D6 mandate comment edits *inside* `export_order.rs`.
D5/D6 are MANDATORY and sec5's own words are "identifier-only edits inside existing bodies",
so the mandatory doc obligations govern. The implementer did D5/D6 and disclosed them under
"sanctioned, not deviations". **Correct reading.** No behavioural assertion was touched.

## Deviation adjudication (Dev-1..Dev-5)

Attribution per deviation. **Four of the five are cases where the CONTRACT was wrong.**

**Dev-1 -- the sec4 error-count identity is unobservable. ACCEPTED. Contract defect ->
planner.** Adjudicated in C.2/C.3. The contract mandated "rename the DEFINITION ONLY" and
then pinned an output that only a fully-repointed tree can produce. The implementer executed
the mandated procedure, observed the structural result, refused to fake the count, and
substituted a **stronger** completeness argument (frozen byte lock + zero warnings on both
reverse-dependency gates). **Planner defect, non-blocking.** This is the fifth F14 instance.

**Dev-2 -- gate 4's warning tally differs between its two runs (21 then 9, both exit 0).
ACCEPTED.** Both runs are green; every warning in both is in `tests/integration/**`, which is
outside the touch-set and which this commit cannot reach. The count moves because cargo
replays cached diagnostics only for targets it re-checks, so the digest's tally tracks cache
state, not the edit. See the gate section for the adjudication on whether the contract should
have pinned only the exit status. **ACCEPTED; planner-side wording recommendation.**

**Dev-3 -- gate 7 ordering (`--all` before each commit, `--staged` once after the code
commit). ACCEPTED, pre-authorised.** sec7 pre-authorises exactly this substitution,
`commit_scoped.py` stages and commits atomically, raw git staging is denied, and `--all` is
strictly stronger than `--staged`. Correctly disclosed as a substitution rather than dressed
up as compliance. Precedents T3-D2 Dev-4 and T3-E1 Dev-4.

**Dev-4 -- two commits, not one. ACCEPTED.** sec8's ONE commit carries exactly the nine in-set
paths; commit 1 is a `tasks/**`-only landing of planner records that no other role can commit
(reviewers hold read-only git). Precedent `ab59bb4`, `4d0f245`, `9f30c9a`. Verified: commit 1
contains **only** three `tasks/` paths and zero code.

**Dev-5 -- the dispatch's land-list did not match the tree. ACCEPTED.** The dispatch named
`tasks/m235-t3d2-verdict.md`; the tree has it already committed in `ab59bb4`. What the tree
actually owed was the E1 verdict, the E1 execution results (untracked) and the rewritten
E2 contract (modified) -- exactly the three landed. Landing the executing contract is the same
class as `ab59bb4`, which landed two contracts. **Correct, tree-wins resolution.**

**Also recorded as sanctioned, not deviations, and I agree:** the identifier-only repoints in
the five `ui_test` files (sec1 item 6 puts `common.rs` in-set, which is the repair the E1
verdict's finding 1 demanded); the D1-D7 doc edits; and the D6/D7 rewraps.

## Scoped gate lines (verbatim canonical)

Every gate run **directly**, in its own turn, in the contract's prescribed form. All seven
pinned lines confirmed. None is vacuous.

| # | Command | Result, verbatim | Pin | Vacuity check |
|---|---|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `186 passed / 0 failed / 0 ignored / 1 suites` | `186/0/0/1` | **not vacuous**: 186 executed, **0 ignored**, 1 suite; `### cargo test PASS (exit 0)`, capture `target/testrun-capture.txt`, 193 lines |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui` | `186 passed / 0 failed / 0 ignored / 3 suites` | `186/0/0/3` | **not vacuous**: same 186 across 3 suites (lib + `ui_test` + doc-tests; `crates/ui/src/` carries zero `#[test]`); capture 205 lines |
| 3 | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` | `ok (exit 0, 0 warnings emitted)` | `ok (exit 0)` | **not vacuous**: the reverse dependency the E blocker never type-checked; `--all-targets` covers lib, bin and the bench target that calls the renamed builder at `payload_tail_bench.rs:326` |
| 4 | `python tools/distill.py -- cargo check -p ariadne-integration-tests --all-targets` | `ok (exit 0, 21 warnings emitted)` | `ok (exit 0)` | **not vacuous**: the declared path dependency with zero textual references; exit 0 at both ends. See the Dev-2 adjudication below on the tally |
| 5 | `python tools/distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` | `ok (exit 0, 0 warnings)` | **not vacuous**: **this** is the proof that no import or reference was missed -- a surviving dangling import or unrepointed reference is a hard error or a warning, and the count is zero |
| 6 | `python tools/hygiene_check.py` | R1 **0** lines, R2 **0** lines, all output R3 warns | zero R1/R2 | verified structurally, below |
| 7 | `python tools/encoding_guard.py --all` then `--staged` | **no output**, both forms | silent | `--all` is the load-bearing form: it scans all tracked content, covering the pure-ASCII requirement on every authored doc line. `--staged` is **necessarily silent at review time** because the index is clean after the commits, so it carries no independent evidence here -- correctly disclosed as Dev-3 |

**Count identity, re-derived from my own capture: `186 - 0 + 0 = 186`**, base and final
identical, matching the implementer's report line-for-line. No test was added or removed.

**Gate 6 detail -- "zero NEW R3 in any file you touch", verified structurally rather than
counted.** The R3 warns landing in touched files are `crates/ui/src/dto.rs:330/:418/:445`
(single-letter `s`), `crates/cli/src/commands/ui.rs:557/:659/:732` (single-letter `p`),
`:656/:665` (`bar`), and `crates/ui/tests/ui_test/graph_api.rs` (`res` at `:52-:231`). Every
one is provably pre-existing: this unit's edits are **one line by one line** in every file it
touched except the three that carry mandated doc rewraps, so no line number in any of these
files shifted; and `dto.rs`'s three warns sit at 330/418/445, far **above** the unit's first
edit at `:599`, while `ui.rs`'s sit **below** its only edit at `:328`. `export_order.rs`,
`dto_builders.rs`, `d27_node_provenance.rs`, `common.rs` and `payload_tail_bench.rs` carry
**no** R3 line at all. This is the same reasoning the implementer used, reached
independently.

### Dev-2 adjudicated: gate 4's warning tally is NOT a pinnable quantity

**My own run of gate 4 returns `21 warnings emitted` at the same commit where the
implementer's final run returned `9`.** That is not a discrepancy in anyone's reporting -- it
is the proof that the tally is a function of cargo's diagnostic cache, not of the tree. Cargo
replays cached warnings only for the crates it actually re-checks, so after a partial
rebuild the same source yields a different count, with the same `exit 0`.

**Adjudication: the contract should have pinned only the exit status for gate 4, and pinning
a warning tally for a crate whose warnings live entirely outside the touch-set was a
category error.** The implementer's handling was right on both counts: it met the contracted
pin (`ok (exit 0)`) at both ends, it did not launder the 21-vs-9 difference into a
"no change" claim, and it attributed the cause correctly and disclosed it as Dev-2. The
zero-warning requirement belongs to gate 5, which is the gate that can actually fail on a
missed repoint, and gate 5 reported `0 warnings emitted`.

**Standing recommendation for the template (planner, non-blocking):** for any gate whose
warnings all originate outside the touch-set, pin `exit 0` and pin the warning *count only*
when the gate re-checks the whole crate from cold. Otherwise the pin manufactures a
non-deterministic gate line that a correct implementer must report as a deviation every time
the cache moves -- which is a false-positive generator, and this milestone now has four of
them behind it.

## Workspace-wide acceptance pass

The contract forbade the implementer to measure this (sec7) and it did not; Phase 3's last
implementation unit makes it the reviewer's to run, and **this measurement is the milestone's
pre-closeout state.** Unlike the earlier test-only units, this one renamed a `pub fn`
reachable through the bare glob `pub use dto::*;` and repointed its production caller, so the
pass is **not** deferrable.

### Canonical line, verbatim

```
1180 passed / 0 failed / 4 ignored / 88 suites
```

(`### cargo test PASS (exit 0)`, capture `target/testrun-capture.txt`, 2070 lines.)

### Measured delta against the last measurement

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| last measurement, at the E1 tip `5addad7` (E1 reviewer) | 1180 | 0 | 4 | 88 |
| **measured here at the T3-E2 tip `dcd681b`** | **1180** | **0** | **4** | **88** |
| delta | **+0 / +0 / +0 / +0** | | | |

**Delta is exactly zero, and zero is the correct answer.** E2's contract sec5 said
`186 - 0 + 0 = 186` crate-scoped and forbade it to claim a workspace number; the workspace
line is unmoved on all four axes, which is the independent confirmation that **no test was
added, removed, renamed or made to stop running** anywhere in the workspace -- not merely in
`crates/ui`. Had a single test been dropped, `suites` or `passed` would have moved. This
closes the arithmetic with **zero residual**, and it is the same protocol F9 prescribes for a
broken `--baseline` flag: measure without the flag and compare by hand against the ledger's
baseline chain.

### Tools suites (what CI runs)

```
Ran 184 tests in 20.778s

OK
```

Matches the E1 reviewer's `Ran 184 tests` and the `discover Ran 184 tests ... OK` figure
recorded in `tasks/ledger_archive.md`. **Not moved.**

### Workspace hygiene

`python tools/hygiene_check.py` (the same invocation as scoped gate 6 -- it is workspace-wide
by construction, scanning every crate plus `tests/integration/**`): **R1 0 lines, R2 0 lines**,
output is entirely R3 warns, none newly introduced in any file this unit touched. See the
gate section for the structural proof.

## Gates refused / not run

**Ran (12 invocations, 10 distinct commands, all direct form, each in its own turn):**
`checkpoint_guard.py status`; scoped gate 1 (`--test ui_test`); scoped gate 2 (`-p
ariadne-ui`); scoped gate 3 (`-p ariadne-cli --all-targets`); scoped gate 4 (`-p
ariadne-integration-tests --all-targets`); scoped gate 5 (`cargo build -p ariadne-ui`);
scoped gate 6 (`hygiene_check.py`, workspace-wide); scoped gate 7 (`encoding_guard.py --all`
and `--staged`); the workspace-wide `testrun.py --distill --timeout 3600`;
`python -m unittest discover -s tools -p "*_tests.py"`. Plus read-only git: `show --stat` on
each of the three commits, `show` on `acea7f2`, `log --oneline` on two ranges, `log` via MCP.

**Refused -- verbatim, catch-all boilerplate, each attempted INDEPENDENTLY exactly once, none
retried, none fixed, none routed through `distill.py`:**

- `rg --version` and `rg -c --no-heading "build_include_elements" crates tests tools`:
  `The user has specified a rule which prevents you from using this specific tool call. Here
  are some of the relevant rules
  {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
- `git grep -c -e "build_include_elements" 5addad7 -- crates tests tools` and
  `git grep -c -e "build_include_elements_from_groups" HEAD -- crates tests tools`: identical
  verbatim text. (Census was completed with the dedicated `grep` tool and the commit diff's
  deleted side instead -- see A.2.)
- `python tools/ui_smoke.py` (F10): identical verbatim text.
- `python tools/bench_compare.py` (F10): identical verbatim text.

**No gate was skipped because some other invocation was refused.** The census work, which is
this unit's entire exhaustiveness proof, was completed by other means rather than abandoned.

**Broken, not refused -- F9:** `python tools/testrun.py --baseline` -> `usage error: missing
value for --baseline`. Reproduced verbatim, not attempted again, not fixed, not worked around;
the workspace pass ran without the flag and was compared by hand, which is F9's own protocol.

**F10 remains the most material unclosed gate in this milestone, and this review does NOT close
it.** There is still **zero runtime evidence** that the UI cold-boots, serves its routes,
returns `Cache-Control: no-store` and produces a group-count report, and still **no
`bench_compare.py` evidence** for what the payload timing means after T3-D's SCC swap.
`ui_smoke.py` and `bench_compare.py` are **operator actions at closeout**, and until they run,
milestone acceptance is not complete no matter how many compiler-level gates are green. The
rename does not make this safe to defer: the one surviving public builder's output is exactly
what `ui_smoke.py` would exercise end-to-end, and D12's `UNVERIFIED` paragraph is still the
only in-tree statement of the gap.

## M23.5 acceptance criteria status for Phase 4

**Criterion 1 -- "`CycleGroup` is the workspace's ONLY cycle representation, and one public
builder name survives in the UI layer": FULLY DISCHARGED for production code, with two
precise caveats the closeout must record rather than gloss.**

Verified at HEAD, independently:

- `from_paths` -> **zero** hits under `crates/`. `build_include_elements_legacy_paths` ->
  **zero**. Both deleted by E1.
- The public `IncludeGraph::detect_cycles()` -> **zero production hits**; it survives only as
  (a) historical prose at `payload_tail_bench.rs:62-65` and `crates/graph/src/cycle_groups.rs:64`,
  (b) Red-evidence comments in `crates/graph/tests/cycle_groups_test.rs:5/:101/:142`, and (c) the
  frozen oracle below. Deleted from `crates/graph/src/graph.rs` by `acea7f2`.
- **Exactly one public include-element builder in the UI layer**: `pub fn
  build_include_elements` at `dto.rs:621`, third parameter `&[CycleGroup]`. The emission tail
  `emit_include_elements` is **private** (`:643`, no `pub`), so there is exactly one public
  path to include-element bytes. `crates/ui/src/lib.rs` is the bare `pub use dto::*;`, so the
  new name is exported automatically and **no alias, shim or deprecated re-export survives**.
- `CycleGroup` reaches the payload through `IncludeCycleClass::from_groups` only.

**Caveat 1 (intended, and it is evidence, not debt):**
`crates/graph/tests/cycle_groups_equivalence_test.rs` retains `old_detect_cycles` /
`old_dfs_cycle` as an explicitly **frozen copy of the old implementation**, used as the
equivalence oracle proving the SCC path reproduces DFS semantics. So a second cycle
representation *does* still exist in the workspace -- inside a test, frozen, labelled, and
unreachable from production. **The closeout should record criterion 1 as "discharged for
production, with one deliberate test-local oracle retained by design",** because deleting it
would delete the evidence that the replacement is equivalence-preserving. This is the correct
place for a dual representation, and it is not the defect the milestone was chartered to remove.

**Caveat 2 (out of scope, noted so nobody re-opens it):** `crates/transform/src/engine/mod.rs:72/:133`
has an unrelated `detect_cycles_in_units` over *generated units*. It is a different subsystem's
guard, not the include-graph cycle currency, and M23.5 does not reach it.

**No other acceptance criterion is judged unmet by this unit.** The one criterion that
remains **unmet at the milestone level** is the F10 runtime/perf evidence above, which no
implementation unit can discharge and which this review explicitly does not close.

## Per-commit evidence (commit vs touch-set)

Oldest first, each judged alone. Base `5addad7`, tip `dcd681b`.

**1. `378b240c8492fb990e3dff1f7917678f57f5478a` -- "docs(tasks): land the rewritten T3-E2
contract and the T3-E1 reviewer records"**

Three paths, **every one under `tasks/`**, zero code:
`tasks/m235-t3e2-handoff.md` (283/405 -- the planner's rewritten contract, the very document
this unit executed), `tasks/m235-t3e1-verdict.md` (699/0, pure addition) and
`tasks/m235-t3e1-execution-results.md` (151/0, pure addition). Total 1133 insertions, 405
deletions.

**Commit-vs-touch-set: technically OUTSIDE sec8's nine `--scope`s, adjudicated APPROVED** as a
planner-declared out-of-band record landing, on four grounds that I verified rather than
accepted: (a) it contains **zero** `crates/**`, `tests/**` or `tools/**` path, so it cannot
damage the unit under review; (b) the dispatch expressly waived it, and the implementer
disclosed it as Dev-4/Dev-5 with named precedents (`ab59bb4`, `4d0f245`, `9f30c9a`); (c) no
other role can land it -- the reviewer holds read-only git, so leaving it unlanded strands the
contract; (d) landing the executing contract is the same class as `ab59bb4`, which landed two
contracts as records. **Recorded as finding 12** -- this is the fourth-plus occurrence of the
same waiver and the standing template fix, unchanged from T3-D2 finding F6 and E1 finding 5.
The boundary I will hold to: an out-of-band commit is benign **only** while it is
`tasks/**`-only; the first one that carries a code path is a blocker.

**2. `4bc3b882676ae58c3813a40398e46a909252570b` -- "refactor(ui,cli): rename the group-based
include element builder to the canonical name"**

Nine paths, **exactly** the nine sec8 `--scope`s: the eight in-set code files plus
`tasks/m235-t3e2-journal.md`. Numstat `2 + 7 + 17 + 4 + 8 + 10 + 29 + 8` over the code files
and `324/0` for the journal; 369 insertions, 40 deletions in total.

**Commit-vs-touch-set: CLEAN.** Verified ABSENT from the commit: `tests/fixtures/basic/artifacts/includes/report.json`
(F12), any other `tests/**`, any `tools/**`, `crates/graph/**`, `crates/report/**`,
`crates/pipeline/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`, `crates/ui/src/lib.rs`,
`crates/ui/tests/ui_test/main.rs`, any `crates/ui/assets/**` path (which makes "zero new hex
literals" structural -- no assets path exists to carry one), and any new file anywhere.

Every hunk accounted for, none behavioural: 1 definition identifier; 19 call sites repointed
callee-token-only with every argument a CONTEXT line; 5 import lists each losing exactly the
dead name; D1/D2/D3/D4 on `dto.rs`; D5/D6 on `export_order.rs`; D7 on `payload_tail_bench.rs`.
No Red state committed, no amend, no `.gitattributes` or lockfile churn, and the journal's own
SHA correctly absent from its own commit table (it is recorded in commit 3). **Judged
APPROVED.**

**3. `dcd681bcbf8a9de973c0c2106b7cc42c7d50bbde` -- "docs(tasks): M23.5 T3-E2 journal -- record
the code commit SHA and gate invocation counts"**

One path, `tasks/m235-t3e2-journal.md`, 12 insertions / 9 deletions. Read in full: it adds
commit 2's SHA `4bc3b88` to the Commits table (which sec9 requires and a commit body cannot
contain for itself), corrects the gate-7 invocation count to `--all` x2 + `--staged` x1, and
corrects `checkpoint_guard.py status` from x2 to x3.

**Commit-vs-touch-set: outside sec8's one-commit instruction, adjudicated APPROVED** -- same
class as commit 1, `tasks/**`-only, and the content is **two self-corrections in the
implementer's favour against its own record** (it had under-reported its own gate invocation
counts). An implementer amending its own journal to make its evidence more accurate, in a
separate `tasks/`-only commit, is not a scope breach. Worth stating plainly because it is the
kind of edit a reviewer should be suspicious of; here the diff makes the record *more* accurate
and adds no code. **No finding.**

**Cross-cutting locks, all verified from the three commit path lists plus the tree:** exactly
one `pub fn build_include_elements`, third parameter `&[CycleGroup]`, 5-arg shape; old name
zero hits in `crates/`, `tests/`, `tools/`; `lib.rs` and `cycle_groups_view.rs` byte-unchanged
and absent; `crates/graph/**`, `crates/report/**`, `crates/pipeline/**`, `tests/**`,
`tools/**` byte-unchanged and absent -- which is precisely what keeps the three bench-baseline
keys valid (D8); D12's `UNVERIFIED` paragraph verbatim at **both** sites; D8/D9/D10/D11/D13
byte-unchanged; F12 fixture in **no** commit; test counts unmoved at `186/0/0/1` and
`186/0/0/3`; workspace line unmoved at `1180/0/4/88`.

## Numbered findings

**APPROVED with zero blocking findings against T3-E2.** Attributed to the causing role. Nine
are planner-attributed, which is the substantive result of this review: the unit's
implementation was clean and its **contract** was wrong five separate times.

1. **Non-blocking, planner. Contract sec4's Red acceptance criterion is structurally
   unobtainable.** It mandated "rename the DEFINITION ONLY. Repoint nothing" and then pinned
   "17 in-crate + 2 cli = 19". Every in-crate caller reaches the builder through a `use` list,
   so rustc emits the 5 `E0432` import errors and stops: **5 errors, not 17, and never
   `E0425`-class.** The procedure's own step ordering makes its own acceptance criterion
   unobservable. Attribution **planner**; adjudicated in C.2. Remedy is F14's written remedy:
   state every predicted output with the command that establishes it, or do not pin it.
2. **Non-blocking, planner. Contract sec0.4's census row is imprecise in wording.** The row
   ``build_include_elements_from_groups`` NOT followed by `(` predicts "5 import lists + 2
   doc name-refs = 7"; it returns **6**, because `dto.rs:601` spelled the **plain** name and
   cannot match a pattern requiring the `_from_groups` suffix. **The closed sum is unaffected**
   (it is expressed over the union pattern, under which `dto.rs:601` counts correctly), so the
   slip is in the wording, not the arithmetic. Attribution **planner**; adjudicated in A.3.
3. **Non-blocking, planner. Contract sec0.4's `crates/cli` row conflates files with token
   hits.** It says "2 under `crates/cli`" naming `src/commands/ui.rs` and
   `tests/payload_tail_bench.rs`. True as *files* and as *builder references* (`:328`, `:325`);
   false as raw `ariadne_ui` token hits, which the implementer measured at **17**. Correctly
   disclosed in the journal rather than absorbed. Attribution **planner**.
4. **Non-blocking, planner. Contract sec7 pins a non-deterministic quantity.** Gate 4's
   warning tally is a function of cargo's diagnostic cache, not of the tree: **my own run of
   the identical command at the identical commit returns `21 warnings emitted` where the
   implementer's final run returned `9`**, both `exit 0`. Pin only the exit status for a gate
   whose warnings all originate outside the touch-set. Attribution **planner**; adjudicated in
   the Dev-2 section. The implementer's Dev-2 disclosure was correct and complete.
5. **Non-blocking, implementer (cosmetic, disclosure adequate). The D7 rewrap left a ragged
   orphan line** at `payload_tail_bench.rs:66` (`//! row was re-pinned above to `5.6` ms`),
   a 40-column line between two full-width lines. The sentence's meaning, its parentheses
   balance and every timing number are correct and `D7`'s citation of `acea7f2` /
   `adb53a3..aebfa9b` verifies against git. Not a defect; recorded so the next editor of that
   header does not read it as damage.
6. **Non-blocking, implementer (correctly declined). The residual `5.1` vs `5.4` split**
   between `ui.rs:103` and `payload_tail_bench.rs:41` for the same phase is a **perf claim**,
   and F10 is still open. Reconciling it inside a rename would have manufactured exactly the
   unverified timing evidence this milestone has been disciplined about refusing. **D11 was
   obeyed and O-1 was reported instead. I agree; a "helpful" reconciliation would have been a
   finding.**
7. **Non-blocking, planner (carried, unchanged). `common.rs` was in the touch-set and had to
   be.** This is the repair the E1 reviewer's blocking finding 1 demanded, and it was applied:
   sec1 item 6 names the file, states *why* (E1's contract-forced edit left `:3` naming the
   now-dead symbol), and drops the old byte-unchanged freeze. **The E1 finding is fully
   discharged**, and no predecessor contract's shape recurred in this unit. Recorded as
   positive closure, not a new defect.
8. **Non-blocking, planner. F14/2's trigger wording is still ambiguous about renames'**
   operational scope. This unit gated **both** reverse dependencies at **both** ends, which is
   the correct and complete reading, so nothing is owed -- but the rule should read "adds,
   removes or renames a public function" so the next implementer does not have to re-derive
   it. Repeats E1 finding 4.
9. **Non-blocking, ledger/operator. `tasks/milestone_ledger.md:85` (deferred row F13) still
   describes two builders at `dto.rs:608`/`:637`.** Both names and both line numbers are
   E1-era and now wrong. Ledger rows are operator-owned; the implementer correctly reported it
   (O-4) and did not edit it. **Needs a ledger edit before closeout**, since F13's text
   currently describes a state this milestone has deliberately destroyed.
10. **Non-blocking, planner. P2's locals remain `legacy_elements` / `grouped_elements` and its
    test name remains `the_parity_comparison_detects_a_cycle_classification_difference`**
    (`export_order.rs:298/:311/:318`). Both are now stale-identifier artifacts. **Neither may
    be fixed in this unit**: sec5 forbids renaming a test and permits only callee-only body
    edits, so O-3's refusal is correct. F13's disposition owns this. Recorded so it is not
    mistaken for an oversight.
11. **Non-blocking, implementer (positive, recorded for the milestone). The `certutil` hash
    pair was taken through `distill.py`.** Judged on the merits and it is a legitimate use,
    not an evasion: a non-gate helper whose output volume is bounded by the sanctioned
    distilling entry point, on a file the reviewer has since verified structurally, with no
    refused gate involved and full disclosure. **No finding** -- recorded because the
    implementer's own journal asserted "Refusals encountered: None", and I confirm that for
    its own session while recording that *my* session did hit refusals.
12. **Non-blocking, planner/template. The out-of-band `tasks/**` landing is now the fourth+
    occurrence** (`9f30c9a`, `3e1efa6`, `4d0f245`, `ab59bb4`, now `378b240` and `dcd681b`).
    It remains necessary -- the reviewer role holds read-only git -- but a recurring waiver is
    a recurring hole in the touch-set discipline. Recommend `handoff-spec` carry the standing
    clause so no unit needs an ad-hoc exception. Repeats T3-D2 F6 and E1 finding 5 unchanged.
    **Standing boundary: an out-of-band commit is benign only while it is `tasks/**`-only.**
13. **Non-blocking, configuration (unchanged). F12 measured again:** the fixture
    `tests/fixtures/basic/artifacts/includes/report.json` is **not even dirty** at HEAD in this
    session (`checkpoint_guard.py status` reports `(none)` dirty), and it appears in **no**
    commit of this unit. The lock holds.
14. **Non-blocking, configuration. F10 remains open and is the most material unclosed gate in
    this milestone.** `ui_smoke.py` and `bench_compare.py` both refused verbatim in this
    review. **This review does not close F10**, and milestone acceptance is not complete until
    an operator runs them at closeout. See the gates section for why a green compiler-level
    pass does not substitute.

---

## Reviewer digest (for the Phase 4 closeout planner)

**Verdict APPROVED.** Base `5addad7` -> tip `dcd681b`; three commits, all adjudicated.

- **A PASS** -- census re-derived independently: 33 total at base = 30 `crates/` + 3 `tools/`
  + 0 `tests/`, closing as `1 + 19 + 5 + 2 + 3 = 30`; at HEAD 29 = `1 + 19 + 5 + 1 + 3`, the
  one-line delta being `dto.rs:601` rewritten by D1. The reported wording slip is in the
  CONTRACT's row, not its arithmetic.
- **B PASS** -- old name **zero** hits in `crates/`/`tests/`/`tools/`; exactly one
  `pub fn build_include_elements` at `dto.rs:621`, third param `&[CycleGroup]`, 5 args; rename
  plus comments only, every argument a CONTEXT line.
- **C PASS** -- the Red is real; `ui.rs:328:40` is exact to the character and names the
  out-of-crate PRODUCTION caller, which is the E blocker's blind spot. The predicted 17+2
  identity is a **planner** contract defect (5th F14 instance) and is structurally
  unobtainable. The implementer's substitute argument is **sound and stronger** than the
  identity the contract asked for. Revert SHA256 pair identical.
- **D PASS** -- all six NO-EDIT items byte-unchanged; D12's `UNVERIFIED` paragraph intact at
  **both** sites; three `tools/` bench keys untouched, so the label and its baseline still
  agree; D1-D7 done and D7's `acea7f2` / `adb53a3..aebfa9b` citation verifies against git.
  Both contested judgments **agreed**.
- **E PASS** -- no test added/removed/renamed; `ACYCLIC_EMPTY_CURRENCY_PAYLOAD` byte-unchanged;
  P1's single invocation and upstream guards intact; P2's `assert_ne!` intact and still a real
  cross-currency comparison.
- **Gates:** `186/0/0/1`, `186/0/0/3`, `ok (exit 0, 0 warnings)` x2 (gates 3 and 5), gate 4
  `ok (exit 0, 21 warnings emitted)`, hygiene R1 0 / R2 0, encoding `--all` and `--staged`
  silent.
- **Workspace pass:** `1180 passed / 0 failed / 4 ignored / 88 suites` -- **delta +0/+0/+0/+0**
  against the E1 tip, which is the independent proof that no test moved anywhere. Tools:
  `Ran 184 tests in 20.778s` / `OK`.
- **Criterion 1 discharged** for production: one public builder, one cycle currency, no alias
  or shim. Two caveats recorded (the deliberate test-local `old_detect_cycles` oracle; the
  unrelated transform-engine `detect_cycles_in_units`).
- **NOT closed:** **F10** (`ui_smoke.py`, `bench_compare.py`, both refused) -- required before
  closeout. **F9** reproduced broken. Ledger row **F13** still describes a destroyed state and
  needs an operator edit.
