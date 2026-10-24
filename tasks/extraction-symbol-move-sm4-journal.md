# SM-4 Journal - extraction-symbol-move (dry-run honesty via the strip-fold mirror + grown faithfulness oracle + generation-swap relay RECORD)

Hand SM-4, milestone `extraction-symbol-move`. Start rev (dispatch head,
verbatim): `a33f620 docs(extraction-symbol-move): SM-3 APPROVED - land verdict
+ execution-results + ledger SM-3 record + SM-3/SM-4 amendments` (this
session's duty-1 landing). Every reviewer diff is against it.

## Boot (checkpoint_guard + git status --porcelain via distill)

HEAD `21cf779` on main; dirty exactly the dispatch-listed set:
` M tasks/milestone_ledger.md`, ` M tasks/extraction-symbol-move-sm3-handoff.md`,
` M tasks/extraction-symbol-move-sm4-handoff.md`, `?? tasks/extraction-symbol-move-sm3-verdict.md`,
`?? tasks/extraction-symbol-move-sm3-execution-results.md`,
`?? tasks/extraction-symbol-move-sm3-handoff-resume.md`, tolerated
` M extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`. Exact match;
no STOP.

## Duty-1 (SM-3 approval doc landing, pre-hand duty)

- Sanctioned clean (Remove-Item is catch-all-denied precedent):
  `python tools/distill.py -- git clean -f -- tasks/extraction-symbol-move-sm3-handoff-resume.md`
  -> `Removing tasks/extraction-symbol-move-sm3-handoff-resume.md`.
- `encoding_guard --staged` (empty staging area at that moment): silent,
  exit 0 (the tool prints only violations by design; the exit-code echo form
  `; Write-Output ...` was refused by the catch-all deny - boilerplate - so
  the guard was verified via the distiller's exit reporting instead).
- Duty-1 commit (single line, run exactly as given): landed `a33f620`,
  `git show --stat HEAD` = 5 files changed, 708 insertions(+), 2 deletions(-):
  sm3-execution-results 147+, sm3-handoff 209+, sm3-verdict 283+, sm4-handoff
  62+, milestone_ledger +7/-2 - `.vsix` ABSENT (atomic).
- `encoding_guard --all` via distiller: PASS exit 0.
- Porcelain after duty-1: only the tolerated `. M extensions/...vsix`.

## AMENDMENT-1 carrier baseline correction (handoff AMENDMENT-1 CARRIER, binding)

The handoff header's `1848` quotes are planning-phase off-by-one relays
(already journal-corrected by SM-3). Measured anchors of record: SM-2 landing
`1849 passed / 0 failed / 13 ignored / 119 suites`; SM-3 delta +0; MY dispatch
baseline `1849/0/13/119` but NOT green (one designed red); landing expectation
`1852/0/13/119`; anchor `1837/0/13/119 @ b201aa6`.

## Mandatory intake (unit-test-audit.md "Deferred findings")

Read before touching the reader/oracle seams. Planner intake result confirmed:
zero deferred items name `extraction_reader.rs` or `extraction_apply/**`
(the cli items name the transform/optimize/measure/discover payload surfaces,
frozen here). No new finding discovered while reading the seams.

## Gate 1 (HARD-CHAIN BASELINE WITNESS - NOT a clean baseline by design)

`python tools/testrun.py --distill -p ariadne-cli` - exit 1,
`170 passed / 1 failed / 0 ignored / 1 suites`, capture
`target/testrun-capture.txt`. The EXACTLY one failing test is the designed
SM-3 hard-chain red, verbatim panic line:
`thread 'commands::ui::extraction_apply::tests::test_a_valid_token_applies_the_plan_end_to_end' (37428) panicked at crates\cli\src\commands\ui\extraction_apply\tests.rs:300:9:` +
`the apply must not modify any file outside the approved dry-run's consumer_file set: modified=["\\\\?\\...basic\\include\\thing.h", "\\\\?\\...basic\\src\\thing.cpp", "\\\\?\\...basic\\tests\\consumer.cpp"]`
Per AMENDMENT-1 CARRIER section 2 this is the baseline witness, NOT a STOP;
nothing beyond it appeared (the run's fail-fast stops at the cli lib binary
by design). The `.vsix` tolerated file does not affect tests.

## RGB - Red (gate 2, capture twice; second capture is the witness)

Authored first (before any production change): required test 5 amendment
(rename + move-truth flip) in `extraction_reader.rs`; the required-test-1 C5
growth in `extraction_apply/tests.rs` (mod line + the two declared helper
call sites + the widened :300 subset clause + widened audit helper); the NEW
`extraction_apply/tests/move_proof.rs` submodule (helpers + the trio, so
everything COMPILES at Red); the declared `pub(crate)` visibility raise of
`move_removal_candidates` (mechanical, zero behavior) rides this Red step.

First capture (mine - journal as an intermediate): exit 1, test 5 failed as
designed, but test 1 failed at the CW-2 sub-claim with `left: []` - MY
authoring bug: I placed the `expected_modified_consumers` computation at the
audit call site, which runs POST-apply, where the rewritten spellings never
hit. C5.1 says both sets read PRE-apply. Fixed by computing `expected_consumers`
once before apply and passing it to both uses (no claim weakened).

Gate 2 witness capture (`python tools/testrun.py --distill -p ariadne-cli`):
exit 1, `172 passed / 2 failed / 0 ignored / 1 suites`, capture
`target/testrun-capture.txt` (266 lines). The two named failures with verbatim
first-assert panic lines:
1. MY named Red - required test 5, pre-fold the reader offers the old hook
   per candidate -> all `None` -> `move_removals == 0`:
   `thread 'commands::ui::extraction_reader::tests::test_dry_run_reports_move_truth_on_the_cpp_fixture' (37140) panicked at crates\cli\src\commands\ui\extraction_reader.rs:795:9:` +
   `the C++ move hook retires the fixture's locatable top-level declaration(s) under ARIADNE-EXTRACT-OUT markers - the dry run must report the move, got move_removals=0`
2. The SM-3 designed hard-chain red, MOVED not discharged: the :300-304
   subset clause went GREEN at authoring (the C5 widening IS the authorized
   discharge per AMENDMENT-1 CARRIER 2 + shape C5 + SM-I6), but the same
   named test then fails the NEW C5.5 parity claim, which pre-fold states the
   R5 under-report itself (dry run 0 vs one name bound):
   `thread 'commands::ui::extraction_apply::tests::test_a_valid_token_applies_the_plan_end_to_end' (16220) panicked at crates\cli\src\commands\ui\extraction_apply\tests\move_proof.rs:315:5:` +
   `assertion `left == right` failed: SM-I3 parity (C5.5): the dry-run move claim must equal the total symbol names bound into the written markers (fixture: 1 == 1)` +
   `  left: 1` + ` right: 0`
- DEVIATION D-2026-10-24a (declared): the AMENDMENT-1 CARRIER predicted
  "EXACTLY ONE named failure" at gate 2 with test 1 GREEN at authoring.
  Machine outcome: two failures in the SAME two named tests of the original
  handoff's RGB protocol ("EXACTLY two named failures" - test 5 pre-fold,
  test 1's grown equality). The carrier's prediction is F14-class
  optimistic-relay (same class as the shape's 4-pair and the original
  gate-2 "in actual NOT" relays): the SM-I3 parity claim CANNOT be green
  pre-fold - the pre-fold parity IS the R5 window. The carrier's substance
  holds: the :300-304 designed-red clause went green at authoring, and the
  remaining failure is discharged by the fold at Green. Not a STOP: both
  failures are named, documented-reason Red witnesses; no STOP-and-report
  lock is involved. No assertion weakened.
- Declared Green-at-birth trio (SM-1 justified-Red-skip precedent):
  `move_proof::move_removal_union_helper_matches_the_fixture_derivation`,
  `move_proof::assert_marked_regions_rejects_an_unbalanced_pair`,
  `move_proof::assert_marked_regions_rejects_tampered_region_bytes` - they
  cannot exist before their helpers; the two rejects are pure synthetic-byte
  self-checks of the proof helpers (non-vacuity guards, not behavior claims);
  the union test pre-apply witnesses the derivation. All three ok in the
  gate-2 capture's passed set (172 = 170 baseline + 3 new - 2 named fails).
- Test-count arithmetic witness: gate 1 171 lib-binary tests -> gate 2 174
  (the +3 trio); both captures 1 suites (fail-fast at the lib binary).

## Green checkpoint (gate 3)

`python tools/testrun.py --distill -p ariadne-cli` - exit 0,
`182 passed / 0 failed / 8 ignored / 8 suites`, capture
`target/testrun-capture.txt` (274 lines). The fold + hook switch + doc lift
landed; the amended pair green (`test_dry_run_reports_move_truth_on_the_cpp_fixture ... ok`,
`test_a_valid_token_applies_the_plan_end_to_end ... ok`), the trio green,
tests 2-11 + honest-absent pin (`pin_degraded_dry_run_is_honest_absent_and_never_cached ... ok`)
+ W9 green UNEDITED (their hunks: none - required test 5 is the only reader
test touched). Gate-3 +3 reconciliation is mechanical at the gate-6 full
census (gate 1/2 lines are fail-fast lib-binary counts; gate 3 reached all
8 suites of the package).

## M3 terminator-oracle rule (carrier 3, binding, honored)

`uncomment_regions` reconstructs byte-exactly; the SM-2 deviation-4 rule (a
marked region ending on a file's final UNTERMINATED line gains the dominant
terminator, `crates/lang-cpp/src/move_retention.rs` render path) is documented
in the module header and the reconstruction message; the fixture files are
terminated so equality holds as written; both synthetic specimens are
AUTHORED terminated (carrier option "author the specimen terminated").

## M4 awareness (carrier 4, informational)

R-G backslash-continuation refusal can never fire alone under R-C/R-D (R-D
also refuses); the pair test pins the OUTCOME. The fold only mirrors the
strip's offer order/text flow and never re-implements a refusal branch, so
no action taken.

## Generation-swap relay (RECORD, NOT execute - kickoff section 4)

ledger 445 generation-swap trigger re-fired at SM-4 via
`extraction_apply/tests.rs` touch; NOT executed per kickoff section 4;
closeout appends the dated relay. Byte proof:
`git diff --numstat a33f620 HEAD -- crates/cli/src/commands/ui/extraction_apply.rs`
EMPTY (the 295 lines stay; recorded below at landing).

## D-rows (gate budget, 2026-10-06 M28 law)

Declared 8 scoped invocations + commit cycle. D-2026-10-24b (declared
overshoot, cause inherited + self-caused): gate 1 x1, gate 2 x2 (my PRE-apply
bug forced a re-capture of the SAME gate - honest witness kept as separate
invocation), gate 3 x2 (Green witness + refactor re-run), gate 4 x1, gate 5
x3 (the handoff's two-`--test` single invocation hits the cargo
last-flag-keeps-one quirk - SM-3 D-row precedent, planner-accepted remedy
"re-state multi-test gates as per-test invocations": ran m18 and m00 as
separate scoped invocations; plus the m00 target-name re-run - the handoff
relayed `m00_facts_path_parity` but the manifest target is
`m00_facts_path_parity_test`, cargo rejected the short name: `error: no test
target named ...`; first parallel attempt of gate 5a/5b additionally DRIFTED
(exit 2) from two concurrent testrun runs racing the shared capture file +
cargo target lock - serialized re-run is the fix, journaling the rule),
gate 6 x2 (discharge witness + post-refactor final), gate 7 x1, gate 8 x2
(first run caught my R1 breach, see the refactor row; exit-code witness via
the sanctioned distiller form `python tools/distill.py -- python tools/
hygiene_check.py` after the plain form's compound exit-echo variant was
catch-all-refused). Measured 15 scoped testrun/invocations + bench + hygiene
x2 - overshoot declared, root causes: my re-captures (gate 2) and the two
inherited tool-surface quirks. Silence is the deviation; this is the row.

## Gates 4-5 (frozen locks, UNEDITED)

- Gate 4 `testrun --distill -p ariadne-validate`: exit 0,
  `33 passed / 0 failed / 0 ignored / 5 suites` - identical to the SM-3
  journal's validate baseline; `move_removal_transaction_test` green UNEDITED
  inside it (SM-I7 condition 6, not re-proven here per handoff item 4).
- Gate 5a `-p ariadne-integration-tests --test m18_cpp_plugin_parity_test`:
  exit 0, `23 passed / 0 failed / 0 ignored / 1 suites` (consumer-half
  byte-truth SM-I4, UNEDITED).
- Gate 5b `--test m00_facts_path_parity_test` (manifest target name, see
  D-row): exit 0, `2 passed / 0 failed / 0 ignored / 1 suites` (boot parity
  on the UNAPPLIED fixture; R9: post-apply boots differ by design - stated,
  not locked further).

## Gate 6 - THE DISCHARGE LINE

`python tools/testrun.py --distill` - exit 0,
`1852 passed / 0 failed / 13 ignored / 119 suites`, capture
`target/testrun-capture.txt` (3214 lines; final post-refactor re-run 3213
lines, same canonical line). Reconciliation: dispatch baseline `1849/0/13/119`
(SM-2 landing; SM-3 delta +0) + my +3 (the trio) = `1852/0/13/119` = the
carrier's landing expectation EXACTLY; anchor `1837/0/13/119 @ b201aa6` plus
SM-1 +2, SM-2 +10, SM-3 +0, SM-4 +3 = 1852. THIS CENSUS DISCHARGES THE HARD
CHAIN SM-3->SM-4 FOR BOTH HANDS: SM-3's threading is proven green
workspace-wide (the designed red at tests.rs:300 is discharged by the C5
widening + the fold) and my gate 2 residual (SM-I3 parity 1 != 0) is closed
by the fold. The SM-4 verdict hop must name BOTH landing commits (SM-3
`a33f620`-recorded code landing and this hand's) and cite the AMENDMENT-1
carrier.

## Gate 7 (fresh bench recording, kickoff sec. 5)

`python tools/distill.py -- python tools/bench_compare.py` - exit 1 (the
tool's WARN form exits 1 - same witness class as the SM-3 journal gate 8
row); recording captured fresh. WARN recorded, NOT attributed (PR-2
persists): `WARN get_or_build_include_graph clone (cold): 1.8 ms vs baseline
1.3 ms (+38%)`. This hand adds no production hot path (the reader fold runs
once per cached dry-run, in-memory, replacing an equivalent read+offer loop).

## Gate 8 (R3 ceiling oracle) + R1 refactor row

First run: `python tools/hygiene_check.py` surfaced ONE blocking finding -
`crates/cli/src/commands/ui/extraction_apply/tests.rs:240: R1 fn
test_a_valid_token_applies_the_plan_end_to_end is 103 lines (> 100)` (my
growth) plus the PRE-EXISTING R3 warn-only set (zero of my lines).
Refactor (shape section 8: the growth belongs in the submodule): the three
PRE-apply bindings collapsed into `move_proof::expectation_pre_apply`
returning the `PreApplyExpectation { fold, consumers, union }` triple - the
same three declared claims, computed by the same helpers, just hoisted.
Gate 3 re-run: exit 0, same `182 passed / 0 failed / 8 ignored / 8 suites`.
Gate 8 re-run (distiller form): `### generic PASS (exit 0)` - R1/R2 clean;
the capture holds 173 lines, all PRE-EXISTING R3 warns (none in
`move_proof.rs`, none new in `tests.rs`/`extraction_reader.rs`).
Ceiling numbers at landing (measured with the Read tool): `tests.rs` 914
lines (watch ceiling 918, net +11 vs 903; R1 test 1 body <= 100 - hygiene
saw it at 103 pre-refactor, exit 0 post), `extraction_reader.rs` 926 lines
(< 1000, margin noted per shape section 8: 899 -> ~910 estimate, actual 926,
no split owed), `move_proof.rs` 495 lines (handoff sizing said ~300-340 -
overshoot declared as D-2026-10-24c: the module header's invariant prose,
granular per-claim helpers (needed for the non-vacuity pair to be
synthetic-testable), and the fuller union-witness test account for the
delta; R2 < 1000 with wide margin, every fn well under 100).

## Measured fixture move count (handoff item 2, journal duty)

MEASURED: 1 (`move_removals == 1`, `copy_only == false` on the fixture).
Machine witnesses: gate 2 parity panic shows the marker-bound total is 1
(`left: 1`) while the pre-fold claim was `right: 0`; gate 3/6 green holds
`test_dry_run_reports_move_truth_on_the_cpp_fixture` (> 0, not copy_only)
and the C5.5 parity (`dry_run.move_removals == bound names == 1`) plus the
trio-A derivation (exactly {include/thing.h} retirable, the ONE merged
`class Thing` entry per the SM-2 PLANNER VERDICT; ctor/method refuse via
idempotence, ThingConfig unlocatable - I pinned the machine truth, never the
shape's relayed "4 pairs").

## Refactor record (RGB step 3)

Plain docs, honest names, one hoist (above) + the gate-2 PRE-apply fix;
gates 3 + 6 re-run green after the refactor; nothing else moved.

## Deliberately NOT done / out of scope (report-only)

- Generation-swap test: NOT added (kickoff section 4 decision of record;
  RECORD-only relay above; `extraction_apply.rs` untouched - diff-EMPTY
  proof recorded below).
- `transform.rs`, every other `crates/cli` file, `crates/ui/**`,
  `crates/validate/**`, `crates/lang*/**`, `crates/transform/**`, DTOs
  (`copy_only: move_removals == 0` construction UNEDITED - SM-I3 keeps its
  computed meaning), token/mutex/epoch machinery, W9 boot lock, fixtures,
  tools: zero hunks (porcelain witness: the tree carried ONLY the four
  touch-set paths + tolerated .vsix before commit).
- `tests.rs` helpers `expected_modified_consumers` (survives, called as the
  sub-claim input) and `assert_modified_faithfully_applied` (the DECLARED
  hunk per handoff item 4: claim shapes kept, inputs widened - expected
  consumers + union + repo_root; message cites SM-I2 + the surviving
  sub-claim; the CW-2 second assert keeps its message body with the sub-claim
  attribution; the A-4 no-deletions claim byte-identical). Binding input 8's
  "byte-identical" for that helper yields to the declared hunk + the 15-line
  ceiling: the two phrases conflict; the amendment clause ("the one changed
  test file is required test 1's module tree") and item 4's explicit
  "declared hunk" are the operative text; journaled as the reading.
- M4 carrier: awareness only, no action (fold mirrors offer order/text flow,
  never refusal branches).
- Report-only observation: the ui crate carries two PRE-EXISTING unused-code
  warnings (`crates/ui/src/export.rs` `INDEX_HTML`, `api_graph.rs:755` `gen`)
  visible in gate captures; untouched (out of touch-set), not attributed.

## Adjacency law (R5 + red-test law, reviewer check)

Duty-1 doc landing `a33f620` -> MY code landing is the immediate next
landing (this commit); nothing else landed between (reflog witness in the
final report); no release in the window. The R5 window closes here: from
this commit the dry-run claim equals what apply binds (SM-I3 witnessed).

## Landing (commit cycle per handoff gate 9, itemized)

Form used: `encoding_guard --staged` (silent + exit 0 per SM-3 precedent)
then the four-scope `commit_scoped.py` verbatim. SHA + `git show --stat` +
adjacency witnesses live in the final report; the empty
`extraction_apply.rs` numstat vs `a33f620` was measured pre-commit (0 lines,
capture `target/distill-capture.txt (0 lines)`) - it is the generation-swap
relay's byte proof too (ledger 445: "next extraction_apply.rs test touch"
re-fired via the tests module; NOT executed; closeout appends the dated
relay).

