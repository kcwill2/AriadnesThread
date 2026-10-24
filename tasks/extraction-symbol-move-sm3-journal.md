# SM-3 Journal - extraction-symbol-move (plan-id threading + m19/m20 second amendment)

Hand SM-3, milestone `extraction-symbol-move`. Start rev (dispatch head,
verbatim from `git log --oneline -3`): `790fe75 docs(extraction-symbol-move):
SM-2 APPROVED - land verdict + execution-results + ledger SM-2 record (census
1849 verified)`. Every reviewer diff is against it.

## Boot (checkpoint_guard + git status --porcelain via distill)

HEAD `fba3c90` on main before duty-1; dirty exactly:
` M tasks/milestone_ledger.md`, `?? tasks/extraction-symbol-move-sm2-verdict.md`,
`?? tasks/extraction-symbol-move-sm2-execution-results.md`, tolerated
`M extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`. Matched the
dispatch expectation; no STOP.

## Duty-1 (SM-2 doc landing, pre-hand duty)

`encoding_guard --staged` (empty staging area at that moment) and `--all`
both silent + exit 0 (the tool prints only violations by design).
`commit_scoped.py` with the three SM-2 scopes landed duty-1 `790fe75`;
`git show --stat HEAD` proves the 3-file atomicity (execution-results 102+,
verdict 300+, ledger +7/-3). Start rev recorded above.

## BASELINE CORRECTION (dispatch carrier 2 - journal-mandated discrepancy)

The handoff quotes the SM-2 landing workspace census as
`1848 passed / 0 failed / 13 ignored / 119 suites` (lines 12-13, 216-218).
That is a planning-phase off-by-one: the actual anchor at `f6c92fe+` is
`1849 passed / 0 failed / 13 ignored / 119 suites` (matches the SM-2 ledger
record and duty-1 commit message "census 1849 verified"). Reconciliation in
this journal runs from MY measured gate-7 census, not the relayed number;
expected net delta vs the measured SM-2 landing is +0 (riders amend inside
existing tests).

## Mandatory intake (unit-test-audit.md "Deferred findings")

Read before touching the validate seam. Planner intake result confirmed:
zero deferred items name `validate/src/lib.rs` or the strip call-site
(deferred set names transform, index/graph, scheduler, cli, repo). No new
finding discovered while reading the seam.

## Dispatch carrier 1 (M1 pin - observable, named failing test)

Satisfied by the handoff itself: rider 1 (m20
`apply_thing_plan_locks_transaction_outcome_and_disk_effects`) asserts the
BEGIN line EQUALS
`// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-001 symbols=Thing`, an observable pin
naming BOTH the plan id (`plan=PLAN-001`) and the symbol names
(`symbols=Thing`) forwarded through `move_out_top_level_symbols` at the strip
call-site; it is a named Red-failing test in gate 2. No self-authored pin
outside the touch-set was needed.

## R5 statement (shape risk R5; journal-mandated)

Until SM-4 lands, the dry run (cli reader `move_removal_count`) still reports
0 move removals while apply marks - a transient under-report window. Honest
per hand; SM-4 MUST land immediately after SM-3 with no other landing between,
and no release may ship in the window. I did not touch `crates/cli/**`.

## Kickoff section 1 layer-1 / SM-I7 note

The in-transaction rollback already snapshots the original file via the
move-removal paths (`validate/src/lib.rs:173-176`); my threading touches the
snapshot set ZERO bytes; the delegation proof
`crates/validate/tests/move_removal_transaction_test.rs` stays byte-identical
and goes green through SM-1's default forwarder (its fake implements only the
old hook) - that green-through-default IS the SM-I5 + SM-I7 witness.

## Gate ledger (itemized per the 2026-10-06 M28 law; 9 scoped + commit cycle)

- Gate 1 (`testrun --distill -p ariadne-validate`, BEFORE any edit): exit 0,
  `33 passed / 0 failed / 0 ignored / 5 suites`, capture
  `target/testrun-capture.txt`. Clean validate baseline recorded.

## RGB protocol status

- Red: both rider hunks authored BEFORE the threading; single gate-2 capture
  covers both riders (two scoped `--test` invocations in one command).
- Green: threading + hook switch + three declared doc lifts.
- Refactor: none expected; re-run if anything moves.

## Gate ledger continued

- Harness note (gate 2 form): the literal gate-2 command (two `--test` flags
  in ONE invocation) made cargo run only the LAST named target
  (capture showed a single `Running ...m19...exe` line; census
  `4 passed / 1 failed / 1 suites` = m19's own five tests). This is a cargo
  flag-selection quirk (duplicate `--test` keeps the last), not a semantic
  deviation; the handoff's RGB protocol section prescribes the resolution:
  "One Red capture, TWO SCOPED INVOCATIONS (gate 2)." I ran each scoped
  invocation; both failures captured below.
- Gate 2a (m19 scoped, `--test m19_optimize_characterization_test`): exit 1,
  `4 passed / 1 failed / 0 ignored / 1 suites`; failing test =
  `optimize_writes_all_extracted_headers_and_rewrites_consumer_includes`,
  verbatim panic line:
  `thread 'optimize_writes_all_extracted_headers_and_rewrites_consumer_includes' (11024) panicked at tests\integration\tests\m19_optimize_characterization_test.rs:172:5:` +
  `extraction-symbol-move: the optimize verb moves identically (one hook fixes all verbs) - the header gained a marker (prefix-only pin; m20 owns the PLAN-001 byte)`
  - the BEGIN presence assert, pre-thread no marker exists. Expected reason.
- Gate 2b (m20 scoped, `--test m20_apply_migrate_characterization_test`):
  exit 1, `5 passed / 1 failed / 0 ignored / 1 suites`; failing test =
  `apply_thing_plan_locks_transaction_outcome_and_disk_effects`, verbatim
  panic line:
  `thread 'apply_thing_plan_locks_transaction_outcome_and_disk_effects' (50044) panicked at tests\integration\tests\m20_apply_migrate_characterization_test.rs:263:5:` +
  `assertion 'left != right' failed: the original file moved: apply must retire the moved declarations in include/thing.h, not leave it byte-identical (extraction-symbol-move)`
  - the `thing_header_after != thing_header_before` assert, pre-thread the
  strip still offers the old hook which answers `None` for C++, so
  include/thing.h is byte-identical. Expected reason.
- Red verification: EXACTLY the two named tests failed across the two scoped
  invocations; every other test in both files passed; no assertion weakened.
  Working-tree checkout of the fixture is CRLF (visible in the m20 panic
  left-value) - all marker proofs are EOL-safe (`.lines()`,
  `split_inclusive('\n')` + terminator trim), never hardcoded full-file bytes.
- Gate 3 (validate, post-thread): exit 0,
  `33 passed / 0 failed / 0 ignored / 5 suites` = the gate-1 baseline exactly
  (+0, threading adds no test).
- Gate 4a (m20 scoped, post-thread): exit 0,
  `6 passed / 0 failed / 0 ignored / 1 suites` - the m20 rider went GREEN.
  The threading's core effect is machine-witnessed on the apply verb:
  include/thing.h carries exactly ONE balanced BEGIN/END pair bound to
  `PLAN-001 symbols=Thing`; uncomment reconstructs the original byte-exactly;
  no live `class Thing {` survives; the reworded consumer-side message holds
  with its assertion byte-identical.
- Gate 4b (m19 scoped, post-thread): exit 1,
  `4 passed / 1 failed / 0 ignored / 1 suites` - NOT green. Failing test is
  the named rider at the SAME panic site as Red
  (`m19_optimize_characterization_test.rs:172:5`, same BEGIN-presence
  message): the optimize verb marks nothing even after the plan-id
  threading. Mechanical root cause below.

## STOP finding (SM-I4 triple-lock - the m19 rider Green is machine-unsatisfiable inside this milestone's frozen surface)

The handoff's rider-2 premise "the optimize verb moves identically - one
hook fixes all verbs" (item 4) is contradicted by the milestone's own
frozen locks. The optimize payload never yields a move-removal CANDIDATE,
so no hook - old or new, plan-id or not - is ever offered for it:

1. `crates/cli/src/optimize.rs:320` (`relocate_extraction_to_generated`,
   read-only evidence): the map feeding the migration relocates every
   generated-unit path to `generated/include/<name>_extracted.h`.
2. `crates/transform/src/placement.rs:107-121,146`
   (`IncludeRewriter::compute_rewrite`/`build_preview`, read-only evidence):
   `ConsumerRewrite.new_includes` are those relocated paths - the
   `generated/` prefix, exactly the spelling divergence m19's own CW-1
   comment records ("the optimize payload's `generated/`-prefixed
   new_includes spelling is unchanged").
3. `crates/validate/src/lib.rs:360` (`collect_move_removals`, byte-frozen by
   shape SM-I4 "The derivation is NOT touched"): the candidate predicate is
   `rewrite.new_includes.contains(&unit.path)` while `apply_with` receives
   the UN-relocated `extraction` (paths WITHOUT the `generated/` prefix -
   `optimize.rs:407` passes the pre-relocation `&extraction`). Prefixed
   `new_includes` never contain the unprefixed `unit.path`, so under the
   optimize verb the candidate set is empty and `strip_moved_symbols` has
   nothing to offer. Under the apply/transform verb (m20/PLAN-001) both
   spellings agree, which is why m20 went green and m19 cannot.

The three surfaces that could carry a fix are all outside my touch-set and
explicitly zero-diff for the whole milestone: `collect_move_removals`
(SM-I4: "ZERO-DIFF"; kickoff section 4: "broadening the derivation is a
pipeline change with its own amendment surface, out" of the milestone),
`IncludeRewriter` (SM-I4 zero-diff), `crates/cli/src/optimize.rs`
(handoff must-not: "crates/cli/** ... every other crate in no diff").
The closed amendment table authorizes ONLY the two test-hunk amendments; the
dispatch's own unsatisfiable-mandate rule and the handoff's STOP-and-report
clause bind: do not improvise a fix, do not widen the amendment surface,
never weaken an assertion to pass. This is the F14-class relay-error class
the SM-2 handoff's "PLANNER VERDICT ON THE FIXTURE OUTCOME" section exists
for: a relayed claim ("one hook fixes all verbs", kickoff section 2
composition paragraph, which speaks of the transform-verb transaction) not
verified against the derivation's machine truth. Only the planner can
adjudicate: re-author SM-3 without the m19 marker claim, register the
derivation-broadening as its own amendment hand, or another disposition.

No landing: committing the tree would land a red test on main (the hand's
one-scoped-landing law presumes gates 3-7 green). Working tree preserved as
the successor/reviewer base (disk-state inventory below); successor runs
`python tools/checkpoint_guard.py resume --spec
tasks/extraction-symbol-move-sm3-handoff.md --scope crates/validate --scope
tests/integration/tests --cmd ...`.

## SECOND STOP finding (required test 1 - the R5 window is a TEST failure, not just a dry-run under-report)

Gate 7 (full census via `--distill`) stops fail-fast at the ariadne-cli lib
binary: `259 passed / 1 failed / 0 ignored / 3 suites` (suites reached:
artifact 58 ok, build 31 ok, cli 170 ok + 1 FAILED). The failing test is
PRE-EXISTING, UNEDITED, outside the touch-set:
`commands::ui::extraction_apply::tests::test_a_valid_token_applies_the_plan_end_to_end`
(`crates/cli/src/commands/ui/extraction_apply/tests.rs:300`), verbatim panic:
`the apply must not modify any file outside the approved dry-run's consumer_file set: modified=["...include\\thing.h", "...src\\thing.cpp", "...tests\\consumer.cpp"]`.
Mechanics: the sanctioned threading marks include/thing.h on the UI apply
path (dry-run spelling matches there, as in m20) - exactly the milestone's
decision of record - while this required-test-1 lock still forbids any
modification outside the consumer_file set. The shape itself assigns that
truth's amendment to SM-4 (SM-I6: "the one changed test file is required
test 1's module tree"; module map SM-4 row owns `extraction_apply/tests.rs`
+ `tests/move_proof.rs` grown oracle; C5: "the original file becomes a
modified file"). SM-3's handoff text treats the inter-hand window as merely
a dry-run under-report ("R5 ... the dry run reports 0 while apply marks -
honest per hand") - it does not anticipate that required test 1 locks the
no-outside-set-modification as an absolute assertion. This makes the SERIAL
SM-3-alone landing impossible without leaving main red, independent of the
m19 finding above. Touching `crates/cli/**` is forbidden to me (handoff
must-not + shape module map). Report, do not fix.

Planner-side remedy candidates (planner's call, not mine): the shape
section 6 DECLINED merge option - "a validated merge of SM-3+SM-4 (validate
change as a declared rider in the cli hand) is legal IF the merged hand
still fits the law" - resolves finding 2 (required test 1 + its grown
oracle ship atomically with the threading) and finding 1 only by dropping
or deferring the m19 marker claim, since the derivation-broadening that
finding 1 needs is milestone-out by kickoff section 4's own words.

## Gates 5-9 witness ledger (all on the threaded tree)

- Gate 5 (delegation proof, scoped): exit 0,
  `2 passed / 0 failed / 0 ignored / 1 suites` - the fake implements the OLD
  hook only, green THROUGH SM-1's default forwarder = the SM-I5 + SM-I7
  proof. Empty-diff proof: `git diff --numstat 790fe75 -- crates/validate/tests`
  returned 0 lines (capture `target/distill-capture.txt (0 lines)`).
- Gate 6 (five scoped invocations, see D-row): m10
  `5 passed / 0 failed / 0 ignored / 1 suites`; m14
  `3 passed / 0 failed / 0 ignored / 1 suites`; m17
  `10 passed / 0 failed / 0 ignored / 1 suites`; m18
  `23 passed / 0 failed / 0 ignored / 1 suites` (parity preview bytes
  survive); m23 `7 passed / 0 failed / 0 ignored / 1 suites` (Scala spine
  deletion via the default delegation un-wobbled). All UNEDITED.
- Gate 7 (full census): exit 1 (two findings attributed above):
  `259 passed / 1 failed / 0 ignored / 3 suites` (fail-fast stop at the cli
  lib binary; the m19 integration failure is downstream of the stop and
  independently witnessed at gate 4b). Reconciliation vs my measured
  baseline: NOT at landing state; the only workspace failures on the
  threaded tree are (1) the m19 rider test (unsatisfiable claim, my
  amendment hunk) and (2) required test 1 (SM-4's table truth). All other
  suites that ran are green; `--no-fail-fast` is not a testrun-sanctioned
  flag (rejects unknown flags), so the full-tree sweep stops at finding 2 by
  design.
- Gate 8 (bench_compare fresh recording via distill): exit 1, recording
  fresh; WARN recorded, NOT attributed (PR-2 persists):
  `WARN get_or_build_include_graph clone (warm): 1.3 ms vs baseline 1.0 ms (+30%)`.
  The exit 1 travels with the WARN form of this tool's output.
- Gate 9 (hygiene_check): exit 0 (warn-only R3 entries, all PRE-EXISTING;
  zero R1/R2 lines: the amended m20 test stays within the 100-line ceiling
  via the handoff's own "marker proofs into the local fn" hygiene
  instruction; my two new local fns are each ~11 lines; all files < 1000).
- encoding_guard --all at stop state: silent + exit 0 (the three doc-lift
  hunks and rider hunks are pure-ASCII authored; the em-dash bytes removed
  from the lifted doc lines are deletions inside the declared doc-lift
  clause; every other pre-existing legitimate character untouched).

## Amendment clause quoting (byte-level, per kickoff section 7 item 2)

Authorizing clause (verbatim, kickoff section 7 item 2): "m19
`optimize_writes_all_extracted_headers_and_rewrites_consumer_includes` and
m20
`apply_thing_plan_locks_transaction_outcome_and_disk_effects`: SECOND
named amendment, additive original-file proofs + the stale "not a move"
comment wording; the validate hand carries the rider; every
consumer-side assertion from CW-1's amendment survives byte-identical."
Shape C7 row 2 mirrors it: "additive original-file marker proofs + the
stale 'not a move' comment; consumer-side CW-1 assertions byte-identical."

- m20: ONE edited line-region (the declared message hunk). Deleted bytes
  (start rev m20 :249-250, verbatim):
  `        "X5 copy-only semantics: the flip is an include rewrite, not a move - \`
  `         declarations survive in the source"`
  New bytes (verbatim):
  `        "extraction-symbol-move: the consumer's definition bytes survive; it is the \`
  `         ORIGINAL file include/thing.h where declarations are commented out in place \`
  `         under the marked ARIADNE-EXTRACT-OUT region"`
  The assertion expression line
  `        thing_impl_after.contains("Thing::Thing(ThingConfig config)"),`
  is byte-identical and stayed true throughout (src/thing.cpp is a
  consumer, never a move source). Everything else in the test body is pure
  insertion (before-capture beside :183-184; marker proof block after the
  CW-1 block). Consumer-side CW-1 block (:222-246 bytes), manifest/outcome/
  tier/generated-list block (:192-220 bytes) and every other test in the
  file: zero bytes changed (reviewer re-verifies against start rev 790fe75).
- m19: PURE ADDITIONS (no deleted or edited line): before-capture line
  beside :113-114; proof block between the post-CW-1 report-count asserts
  (start-rev bytes :156-157 unchanged, surviving) and `Ok(())`. Report
  counts and test counts unchanged in both files (5 tests m19, 6 tests m20
  post-RED-verified census lines above), keeping census-by-name
  reconciliation mechanical.

## D-row: gate-budget overshoot (declared per the 2026-10-06 M28 law; silence is the deviation)

D-2026-10-24 SM-3 overshoot: declared budget 9 scoped invocations + commit
cycle; measured 16 + duty-1 trio: G1 x1, G2 x2 (cargo duplicate-`--test`
quirk forced the two-scoped-invocation form the RGB protocol itself names),
G3 x1, G4 x2 (same quirk; both named riders must be witnessed individually),
G5 x1, G6 x5 (same quirk; five named UNEDITED suites each need their own
invocation - the handoff's single-command form would have sampled only the
last), G7 x1, G8 x1, G9 x1, plus `git diff --numstat` witness (G5 proof) and
`encoding_guard --all` at stop state. Root cause: cargo keeps only the LAST
duplicate `--test` flag (captured, verbatim, above); every multi-test gate
command in this handoff under-samples as written. Recommend the planner
re-state multi-test gates as per-test invocations milestone-wide.

## Deliberately NOT done (scope discipline)

- No edit to `collect_move_removals`, `IncludeRewriter`,
  `relocate_extraction_to_generated`, `crates/cli/**`, or any validate test:
  the two STOP findings' fixes live exclusively on frozen/out-of-touch-set
  surfaces; touching them would be an improvised fix outside the handoff.
- No weakening/removal of any assertion to force green.
- No landing commit for the code; duty-1 (`790fe75`) is this session's only
  commit (sanctioned pre-hand duty).
- No workspace-wide gate beyond the handoff-named gate 7 (`--distill`
  census); no `git stash`, no branch change, Cargo.lock in no diff.

## Disk-state inventory at STOP (for successor `checkpoint_guard resume`)

- main at `790fe75` (this session's duty-1 landing; reflog below).
- Uncommitted (working tree), all inside the SM-3 touch-set:
  `M crates/validate/src/lib.rs` (threading + three declared doc lifts),
  `M tests/integration/tests/m20_apply_migrate_characterization_test.rs`
  (rider 1 - green under the threading),
  `M tests/integration/tests/m19_optimize_characterization_test.rs`
  (rider 2 - red; unsatisfiable claim per STOP finding 1),
  `?? tasks/extraction-symbol-move-sm3-journal.md` (this file).
  Pre-existing tolerated: `M extensions/ariadne-extension/...vsix`.
- If the planner instead re-runs SM-3 minus the m19 rider: rider 1 + the
  threading go green as witnessed (gates 3/4a/5/6), BUT STOP finding 2
  (required test 1) still forbids landing until its SM-4 truth ships - the
  merge option or a same-session SM-4 adjacency is the landing path.

## AMENDMENT-1 execution (resumed session, 2026-10-24)

Contract of record: `tasks/extraction-symbol-move-sm3-handoff.md` section
"AMENDMENT-1 (dated 2026-10-24, planner)" (handoff lines 287-494), read in
full; it supersedes RIDER 2 (item 4) and the gate-7 landing expectation.

### Resume boot (amendment landing-sequence step 1) + DISCREPANCY DISCLOSED

- Literal amendment invocation (`--cmd python tools/testrun.py ...`
  unquoted) fails parsing mechanically: `ERROR: unknown resume option:
  tools/testrun.py. usage: ... resume --spec <handoff.md> [--scope <glob>]...
  [--cmd <line>]...` - `--cmd` takes ONE line token. Retried with the same
  command as one quoted line (sanctioned form per the tool's own usage).
  Deviation declared: quoting the `--cmd` line; content unchanged.
- Resume verdicts (tool ran them): the quoted m19 gate ran PRE-EDIT and came
  back `red (exit 1)` - the old working-tree unsatisfiable block still in
  place (expected, documented); the resume tool additionally parsed a
  commit_scoped command VERBATIM from the handoff markdown and ran it
  truncated (`usage: commit_scoped.py ... error: argument --scope`, exit 2,
  usage error) - NO commit occurred: `git log --oneline -3` re-verified HEAD
  `790fe75` immediately after. Resume report:
  `tasks/extraction-symbol-move-sm3-handoff-resume.md` (self-generated
  artifact, left untracked).
- Disk state at resume: HEAD `790fe75` (main); dirty `M
  crates/validate/src/lib.rs`, `M tests/integration/tests/m19_...rs`, `M
  tests/integration/tests/m20_...rs` (the three in-scope edits), tolerated
  `M ...vsix`, `?? tasks/extraction-symbol-move-sm3-journal.md` - PLUS three
  extra modified files the amendment-step-1 inventory did not list:
  `M tasks/extraction-symbol-move-sm3-handoff.md` (the AMENDMENT-1 text
  itself), `M tasks/extraction-symbol-move-sm4-handoff.md`, `M
  tasks/milestone_ledger.md` (the amendment's "ledger FOLLOW-ON INTAKE row,
  2026-10-24"). Adjudication to proceed rather than STOP: each extra is the
  ratified amendment's own issuance artifact by the planner's no-bash write
  pattern (planner artifacts land via an implementer's scoped duty - duty-1
  is this hand's own precedent); none is a foreign live hand; none can ride
  my landing (commit_scoped file-scopes to the four declared paths, proven
  post-landing by `git show --stat`); none affects the three re-run gates.
  These three files REMAIN uncommitted for their own commit duty - I do not
  widen my scope to land them.

### RIDER 2 re-author (step 2): working-tree proof block -> DEFERRAL lock

Edited ONLY `tests/integration/tests/m19_optimize_characterization_test.rs`
(Edit tool, never Set-Content/Out-File/`>`). Authorization: amendment
section "RIDER 2 disposition (item 4) - RE-AUTHOR to a machine-true DEFERRAL
lock (withdrawal rejected)": "Byte-exact working-tree replacement (Edit tool
only; replace EXACTLY this block - the current file's lines 160-185 ... the
`uncomment_marked_regions` local fn is REMOVED from m19 - an unused local
fn is a dead-code warning, and m20 keeps its own separate copy)".
Old-bytes verified byte-identical against the file before replacing.
- REMOVED (working-tree-only bytes; NEVER reached git history - the
  reviewer's per-commit diff of the landing vs start rev `790fe75` sees ONLY
  the NEW block): the 26-line proof block - after-capture, the 11-line
  `uncomment_marked_regions` local fn, the BEGIN-presence `assert!`, the
  begin/end marker collections, the balance assert, the reconstruction
  assert (full text quoted in the amendment's OLD bytes block, handoff
  lines 340-367).
- ADDED (amendment NEW bytes block, handoff lines 371-387, 15 lines,
  byte-exact): the after-capture line + the DEFERRAL-lock `assert_eq!`
  pinning `thing_header_after == thing_header_before` with the message
  naming STOP finding 1's mechanics (optimize.rs:320 relocation vs
  UN-relocated extraction at :407; frozen `collect_move_removals`
  `crates/validate/src/lib.rs:362` SM-I4 ZERO-DIFF yields ZERO candidates)
  and the unfreeze tripwire ("If this assert ever FAILS, the
  ledger-registered follow-on 'optimize-verb move parity' ... has landed:
  replace this lock with the marked-retention proofs m20 witnesses ... never
  delete it silently").
- KEPT: before-capture :115, report-counts :157-158, blank :159, `Ok(())` +
  closing brace. Test-count law holds: m19 5 tests, m20 6 tests, SM-3 delta
  +0. Against start rev `790fe75` the m19 file diff remains PURE ADDITIONS
  (16 net lines: before-capture + the 15-line block minus... exact stat in
  the landing commit).

### Gates re-run (step 3; amended budget: ONE scoped testrun + commit trio,
   plus the two mandated re-proofs requested by my dispatch)

- Gate 4b' (m19 scoped): exit 0, `5 passed / 0 failed / 0 ignored / 1
  suites`, capture `target/testrun-capture.txt` (30 lines) - deferral lock
  GREEN. RGB adjudication per amendment: Red = gates 2a + 4b (the
  machine-false claim failing, witnessed above); Green = this re-run;
  Green-at-birth is the machine-truth characterization of the deferral,
  strengthened by the unfreeze tripwire, never a weakening.
- Gate 4a re-proof (m20 scoped): exit 0, `6 passed / 0 failed / 0 ignored /
  1 suites` - the m19 edit changed nothing in m20's behavior.
- Gate 3 re-proof (validate): exit 0, `33 passed / 0 failed / 0 ignored / 5
  suites` - identical to the gate-1 baseline and the earlier gate 3.

### HARD-CHAIN statement (verbatim from AMENDMENT-1, authorized)

"the planner AUTHORIZES a designed HARD CHAIN SM-3 -> SM-4 (precedent:
ui-extraction-card-playbook T-A/T-B designed-red window, O1 disclosed):
SM-3 lands validate threading + m20 rider + m19 deferral lock carrying
EXACTLY that one workspace red, disclosed in this amendment, the journal,
and the ledger; SM-4's hop discharges the workspace gates for BOTH hands
(gate-1 names the known red; the C5 growth discharges it at authoring; gate
6 full census `1852/0/13/119` proves both; the SM-4 verdict names both
landing commits). Window law: no release between; SM-4's code landing is the
immediate next landing (duty-1 doc landings allowed in between; reviewer
adjacency check)."
Restated: main stays red on the single named test
(`test_a_valid_token_applies_the_plan_end_to_end`) until SM-4's landing
discharges it.

### R5 carried forward (unchanged law)

Until SM-4's reader flip lands, the dry run reports 0 while apply marks -
and (finding 2) required test 1 fails on main by designed authorization.
SM-4 MUST be the immediate next code landing; NO RELEASE may ship in the
window.

### Landing (step 5): amended commit_scoped, four scopes, journal rides

Form used: `encoding_guard --staged` then `--all` (both silent + exit 0,
my established precedent), then the amendment's revised commit_scoped
command verbatim (four scopes; never `git add .`). Landing SHA and `git
show --stat` recorded in the final report; post-landing `git status
--porcelain` must show ONLY: tolerated .vsix + the planner's three
uncommitted artifacts (their own commit duty, disclosed above) + the
self-generated resume report - NOT claimed clean; reported verbatim.




