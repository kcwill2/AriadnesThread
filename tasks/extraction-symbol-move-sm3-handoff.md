# SM-3 Handoff - extraction-symbol-move: plan-id threading into the strip call + the m19/m20 SECOND named amendment (DECLARED riders)

Milestone: `extraction-symbol-move` (UNREGISTERED post-M28 track, no
M-number). Hand 3 of 5, strictly serial: SM-1 -> SM-2 -> SM-3 -> SM-4 -> SM-5.
SM-2's hook is live but nothing calls it at apply yet (the strip call still
offers the OLD deletion hook, which answers `None` for C++ - so apply today is
still copy-only); your ~10-line threading lands the milestone's core effect,
and your two riders are its machine witnesses. SM-4 MUST land immediately
after you (risk R5: until SM-4 the dry run reports 0 while apply marks -
honest per hand, journal the R5 statement, no release may ship in the window).
Author rev `2177a85` on `main` (planner-verified 2026-10-23; your baseline is
the SM-2 landing state: workspace `1848 passed / 0 failed / 13 ignored /
119 suites`). Start rev: your dispatch head - journal it verbatim; every
reviewer diff is against it.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-symbol-move-kickoff.md` section 7 item 2 (YOUR closed
   amendment clause: m19/m20 SECOND named amendment - additive original-file
   proofs + the stale "not a move" comment wording; every consumer-side CW-1
   assertion survives byte-identical), section 1 layer 1 (rollback already
   snapshots the original file - your threading must not weaken that).
3. `tasks/extraction-symbol-move-shape.md` C3 (your contract), invariants
   SM-I1/SM-I4/SM-I7, risk R4 (second amendment of a twice-touched frozen
   lock - the sharpest repeat risk of this milestone), module map SM-3 row.
4. `tasks/extraction-symbol-move-sm2-handoff.md` "PLANNER VERDICT ON THE
   FIXTURE OUTCOME" - the fixture marks exactly ONE merged region
   (`class Thing`, lines 6-13, bound to `PLAN-001`); your riders pin THAT,
   never the shape's relayed "4 pairs".
5. `docs/specs/unit-test-audit.md` section "Deferred findings" - MANDATORY
   intake read before touching the validate seam. Planner intake result
   2026-10-24: zero deferred items name `validate/src/lib.rs` or the strip
   call. Carry any new finding you discover; report, do not fix.
6. Your production sites (planner-verified today): `crates/validate/src/lib.rs`
   - `write_all_targets` already holds `migration: &MigrationPreview`
   (:212, and `MigrationPreview.plan_id` is a public String - lib.rs :580
   test construction proves it); `strip_moved_symbols` is called at :273 and
   offered via `remove_top_level_symbols` at :390; `collect_move_removals`
   (:345-370) is NOT touched; the snapshot (:173-176) is NOT touched (SM-I7).
7. Your delegation proof, UNEDITED, green: `crates/validate/tests/
   move_removal_transaction_test.rs` (174 lines) - the fake implements ONLY
   the old hook (:44-54), so after your switch it reaches the strip through
   SM-1's default forwarder; `seen_names` (:133-135) still records the moved
   names; rollback still restores (:138-144). That file is the SM-I5 + SM-I7
   witness - you change ZERO bytes in it.
8. Your two rider tests in full (post-CW-1 state, planner-verified today):
   `tests/integration/tests/m20_apply_migrate_characterization_test.rs`
   (289 lines; amend ONLY `apply_thing_plan_locks_transaction_outcome_and_disk_effects`
   :178-253; the stale "not a move" assertion+message sits at :247-251) and
   `tests/integration/tests/m19_optimize_characterization_test.rs` (227
   lines; amend ONLY `optimize_writes_all_extracted_headers_and_rewrites_consumer_includes`
   :109-159).
9. `python tools/checkpoint_guard.py status` - bootstrap before any mutation.
   Expected clean tree at the SM-2 landing except the pre-existing dirty
   `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`; anything
   beyond it is a STOP-and-report.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/extraction-symbol-move-sm3-journal.md (checkpoint journal; create it)
- crates/validate/src/lib.rs (the plan-id threading + the THREE named doc
  sentences below)
- tests/integration/tests/m20_apply_migrate_characterization_test.rs
  (DECLARED RIDER 1: hunks ONLY inside
  `apply_thing_plan_locks_transaction_outcome_and_disk_effects`)
- tests/integration/tests/m19_optimize_characterization_test.rs
  (DECLARED RIDER 2: hunks ONLY inside
  `optimize_writes_all_extracted_headers_and_rewrites_consumer_includes`)

You MUST NOT touch: every other file in `crates/validate` (esp. every test:
`move_removal_transaction_test.rs`, `mid_apply_rollback_test.rs`,
`transaction_apply_test.rs`, `orchestration_tests.rs`), `crates/cli/**`
(the reader still reports 0 until SM-4 - R5, journal it, do not "fix" it),
`crates/transform/**`, `crates/lang/**`, `crates/lang-cpp/**`, `crates/ui/**`,
every other test in m19/m20, `tests/fixtures/**` (byte-frozen), `tools/**`.
No `git stash`, no branch change. Cargo.lock must not appear in your diff.

## What to build (shape C3 + the closed second amendment)

1. Production threading (~10 lines): `strip_moved_symbols` gains a
   `plan_id: &str` parameter; the call at :273 passes `&migration.plan_id`;
   the hook offer at :390 becomes
   `rewriter.move_out_top_level_symbols(&content, names, plan_id)`.
   NOTHING else moves: no signature change on `apply_with`/`rollback`/
   `verify_restored`; the snapshot set, the phase ordering (rewrites :246-271
   THEN strip :273), `collect_move_removals`, per-unit atomicity semantics
   all byte-identical (SM-I4). validate still names no language (I2).
2. Declared doc-lift clause (the three sentences that will become untrue the
   moment you switch the call - named here so the reviewer's SM-I4 "ZERO-DIFF"
   is adjudicated as BEHAVIOR zero-diff, doc-truth lift excepted; same class
   as SM-1's I5 doc lift that the shape itself ordered):
   - `apply_with` doc :142-149: "`rewriter.remove_top_level_symbols` decides
     the strip" -> the move hook (`move_out_top_level_symbols`) decides it;
     `None` (a refusal or a copy-only language) leaves the file
     byte-identical (the copy fallback survives); keep the D-M23-5/I2/I5
     references; pure ASCII.
   - `collect_move_removals` doc :334-344: the first sentence's hook name ->
     the move hook; every other word byte-identical.
   - `strip_moved_symbols` doc :379-381: "the plugin's removal hook" -> the
     plugin's move hook, naming `move_out_top_level_symbols` and the plan-id
     marker binding.
3. RIDER 1 - m20 `apply_thing_plan_locks_transaction_outcome_and_disk_effects`
   ONLY. Additive + the one declared message:
   - capture `let thing_header_before =
     fs::read_to_string(repo_root.join("include").join("thing.h"))?;` beside
     the existing before-captures (:183-184);
   - after `apply_transform_plan`, add a local fn `uncomment_marked_regions`
     (remove exactly the BEGIN/END marker lines, strip one leading `//` from
     every remaining region line; \r-tolerant) and the marker proofs:
     `thing_after != thing_before` stated as "the original file moved";
     BEGIN-count == END-count (balanced); exactly ONE BEGIN line and it
     EQUALS `// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-001 symbols=Thing`
     (planner-verified fixture truth per binding input 4 - message cites the
     merged-region derivation, not the shape's relayed 4-pair count);
     the END line EQUALS `// ARIADNE-EXTRACT-OUT END plan=PLAN-001`;
     prefix-stripping reconstructs the ORIGINAL file exactly:
     `uncomment_marked_regions(&thing_after) == thing_before` (kickoff
     section 1 layer-2 reversibility, machine-witnessed; EOL-safe line
     comparison, no hardcoded full-file bytes);
     no live declaration survives: `!thing_after.contains("\nclass Thing {")`
     (the region line carries the `//` prefix).
   - REWORD the :247-251 message ONLY (the assertion expression
     `thing_impl_after.contains("Thing::Thing(ThingConfig config)")` is
     byte-identical and stays true - `src/thing.cpp` is a consumer, never a
     move source; the message states the new truth: the consumer's definition
     bytes survive; it is the ORIGINAL file `include/thing.h` where
     declarations are commented out in place under the marked region
     (extraction-symbol-move); pure ASCII; drop the "not a move" wording).
   - The manifest/outcome/tier/generated-list assertions (:192-220) and the
     consumer-side CW-1 block (:222-246) byte-identical (closed policy).
4. RIDER 2 - m19
   `optimize_writes_all_extracted_headers_and_rewrites_consumer_includes`
   ONLY. Additive: capture `thing_header_before` beside :113-114; after the
   optimize call add the local `uncomment_marked_regions` (duplicated per the
   CW-1 no-cross-file precedent - separate test binaries) and the proofs:
   the header gained a marker (positive: `contains("// ARIADNE-EXTRACT-OUT
   BEGIN plan=")`), BEGIN/END counts balanced, uncommented reconstruction ==
   `thing_header_before` (the optimize verb moves identically - one hook
   fixes all verbs). Do NOT pin the literal plan id in m19 (report-only
   prefix check; m20 owns the `PLAN-001` byte). Report counts + test counts
   unchanged in both files so census-by-name reconciliation stays mechanical.

## Frozen-lock amendment clause (kickoff section 7 item 2 - binding)

You are the ONLY hand that may EVER amend m19/m20, second and FINAL
amendment, only these two tests, only the hunks above, only after capturing
the Red. Proof obligations the reviewer enforces per-commit against your
start rev:
- Every consumer-side CW-1 assertion byte-identical (m20 :224-246, m19
  :135-157 except none - m19's hunks are pure additions; the m19 :156-157
  report counts survive), the m20 manifest/outcome/tier/generated-list block
  byte-identical, every other test in both files byte-identical.
- The amended assertions must first FAIL on the pre-thread tree for the
  documented reason (captured Red below).
- The new truth is independently witnessed, never only by your amendments:
  SM-2's ten unit tests stand under it, and SM-4's grown oracle lands as the
  end-to-end witness (never amend a rider to protect it).
- Discovery of any OTHER pre-existing lock wanting an edit: STOP-and-report
  (R8 / CW-3 discipline precedent). Do not widen the amendment table.

## RGB protocol (single Red capture covering both riders)

1. Red: author BOTH riders' hunks BEFORE the threading. One Red capture, two
   scoped invocations (gate 2): exit 1, EXACTLY the two named tests failing -
   m20 on `thing_after != thing_before` (pre-thread, the strip still offers
   the old hook -> `None` -> `include/thing.h` byte-identical), m19 on the
   BEGIN presence assert. Journal the two panic lines verbatim. NEVER weaken
   an assertion to pass.
2. Green: thread the plan id + switch the hook offer + the three doc lifts;
   re-run gates 3-6 until green.
3. Refactor: none expected (threading + riders); re-run.

## Hygiene (review-enforced)
Functions < 100 lines (m20's amended test grows but the local helper keeps
the body honest; if the test body threatens the ceiling, the marker proofs go
into the local fn - the file has room: m20 289 -> ~330, m19 227 -> ~260,
`validate/src/lib.rs` 847 -> ~857, all < 1000); no banned names
(`uncomment_marked_regions`, `thing_header_before`); authored bytes pure
ASCII. `python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- The delegation + rollback proof green UNEDITED: gate 5
  (`move_removal_transaction_test.rs` - its fake implements the OLD hook
  only; green-through-default IS the SM-I5 + SM-I7 proof; file in no diff).
- `crates/validate` suites green UNEDITED (gate 3): `mid_apply_rollback_test`,
  `transaction_apply_test` byte-parity, orchestration tests.
- m10/m14/m17/m18/m23 green UNEDITED: gate 6 (m18 parity keeps the
  `- #include "include/thing.h"` DISPLAY bytes; m23 is the Scala spine -
  deletion via the default delegation must not wobble).
- Snapshot/rollback machinery byte-identical (:157-201 minus doc hunks, :283
  -331; SM-I7). `collect_move_removals` BODY byte-identical (:345-370; only
  its doc sentence lifts).
- goldens/fixtures byte-locked; `tools/**` operator-law; `crates/cli/**` and
  every other crate in no diff; Cargo.lock in no diff.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only;
raw cargo is a reviewer blocker)
1. `python tools/testrun.py --distill -p ariadne-validate` - clean baseline
   BEFORE any edit; journal the canonical line verbatim.
2. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m20_apply_migrate_characterization_test --test
   m19_optimize_characterization_test` - Red: exit 1, EXACTLY the two amended
   tests failing (journal the two panic lines).
3. `python tools/testrun.py --distill -p ariadne-validate` - Green: exit 0,
   passed = baseline (+0: threading adds no test), everything green.
4. same command as gate 2 - Green: exit 0.
5. `python tools/testrun.py --distill -p ariadne-validate --test
   move_removal_transaction_test` - exit 0 UNEDITED (the named SM-I5/I7
   delegation proof; journal the capture lines + the empty `git diff --numstat
   <start rev> HEAD -- crates/validate/tests` proof).
6. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m10_validation_test --test m14_end_to_end_test --test m17_plugin_seam_test
   --test m18_cpp_plugin_parity_test --test m23_symbol_isolation_test` -
   exit 0 UNEDITED.
7. `python tools/testrun.py --distill` - full workspace census; reconcile from
   your journal baseline and attribute every delta (expect +0 passed - the
   riders amend inside existing tests, so passed holds at the SM-2 landing
   `1848 passed / 0 failed / 13 ignored / 119 suites`, 13 ignored / 119
   suites held; anchor of record `1837 passed / 0 failed / 13 ignored /
   119 suites @ b201aa6`; the `--baseline` FILE form stays superseded by this
   `--distill` substitute).
8. `python tools/distill.py -- python tools/bench_compare.py` - fresh
   recording (kickoff sec. 5 mandates it per hand); WARNs recorded, NOT
   attributed (PR-2 persists).
9. `python tools/hygiene_check.py` - exit 0.
10. Commit cycle (itemized per the 2026-10-06 M28 law):
    `python tools/encoding_guard.py --staged` (or the `--all` superset if the
    staging seam refuses - journal the form used) then
    `python tools/commit_scoped.py --scope crates/validate/src/lib.rs --scope
    tests/integration/tests/m20_apply_migrate_characterization_test.rs --scope
    tests/integration/tests/m19_optimize_characterization_test.rs --scope
    tasks/extraction-symbol-move-sm3-journal.md -m "feat(validate): bind the
    plan id into the move hook at the strip call; second m19/m20
    marked-retention amendment (extraction-symbol-move SM-3)"`. Never `git add
    .`; the journal rides inside the landing commit.

Declared gate budget (2026-10-06 M28 law): 9 scoped invocations + the commit
cycle - inside `<= ~10 + commit-cycle trio`; itemize per-gate counts in the
journal and declare any overshoot as a dated D-row (silence is the deviation).

## Discipline clauses
- UTF-8: never Set-Content/Out-File/`>`/`Get-Content | -replace`; Write/Edit
  tools only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
  Your reword DELETES the pre-existing message bytes at :249-250 and adds
  only ASCII - the encoding guard must stay silent.
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Amendment policy (kickoff section 7 item 2) and the STOP-and-report clause
  are the highest-priority contract after this file.
- After EACH Green gate append a 1-3 line checkpoint to the journal (module +
  rev, any deviation, any lock finding).
- Context exhaustion looming: STOP, return a disk-state inventory; a dead
  session is never resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-symbol-move-sm3-handoff.md --scope crates/validate --scope
  tests/integration/tests --cmd ...`.
- Never pin warning counts; never state a count without the command that
  produces it (M23.5 F14 standing rules).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence: the two named failing
tests with verbatim first-assert panic lines. 3. Green evidence: distill
digest headers + canonical lines for gates 3-7 + capture paths, the empty
validate-tests diff proof, and the R5 statement (reader still reports 0 until
SM-4 - land SM-4 next, no release between). 4. Any deviation + reason.
5. Out-of-scope observations (report-only). Never paste raw tool output; cite
command + canonical line + exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill. Extra MUSTs for this hand: per-commit diff shows hunks ONLY in the four
touch-set files; inside the two rider files the hunks sit ONLY inside the two
named tests (m20 additionally carries the declared message-only hunk at
:247-251 - its assertion line must be byte-identical); `crates/validate/tests`
and `crates/cli` diff-EMPTY; `collect_move_removals` body byte-identical;
census +0 reconciled.

Sizing (planner, itemized from the mandated bodies - M23.5 rule 4): threading
+ doc lifts ~+18/-8; m20 rider ~+38/-5 (helper + proofs + message hunk); m19
rider ~+22/-0; journal ~15 - total ~95-140 produced lines vs the ~1200 bound;
9 scoped gates + commit cycle vs ~10 + trio; one crate dir plus the TWO
DECLARED rider files (A-3 rider precedent). The split rationale for keeping
the riders with the threading (shape section 6 merge option DECLINED at
authoring): the rider hunks ARE the statement of the threading's truth - any
seam separating them forces a half-built tree (threading-without-riders ships
the milestone's core effect un-witnessed; riders-without-threading cannot go
Green), and the merged SM-3+SM-4 option would force the rider Red to be
re-choreographed around the reader flip, which the serial order gives free.

## AMENDMENT-1 (dated 2026-10-24, planner) - supersedes RIDER 2 (item 4); authorizes a designed HARD-CHAIN landing; adjudicates both STOP findings

Authority: kickoff section 7 closed-amendment policy (the planner owns
amendment issuance; the implementer correctly STOPped per the STOP-and-report
clause - textbook CW-3 discipline, zero improvisation). Trigger: the SM-3
journal's two machine-witnessed STOP findings, both verified by the planner
against the named files on 2026-10-24 before issuance. Everything above this
section stands EXCEPT as amended here. Reviewer: adjudicate the landing
against THIS section where it conflicts with the original text.

### Erratum (milestone-truth scope)

The rider-2 premise carried in item 4 - "the optimize verb moves identically
(one hook fixes all verbs)" - is machine-FALSE for the optimize verb under
this milestone's frozen SM-I4 derivation:
`relocate_extraction_to_generated` (crates/cli/src/optimize.rs:320) spells
the migration with the `generated/` prefix while `apply_with` receives the
UN-relocated extraction (`optimize.rs:407`), so `collect_move_removals`
(crates/validate/src/lib.rs:347-370, predicate :362 - NOT
`crates/transform/src/lib.rs` as one relay stated; planner-verified by grep
and read) matches ZERO candidates and the move hook is never offered for the
optimize payload. The kickoff/shape composition paragraphs speak of the
transform-verb transaction - the decision of record is UNCHANGED and the
milestone's core effect is PROVEN for the apply verb: the m20 rider went
GREEN under the threading, witnessing exactly ONE balanced
`ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-001 symbols=Thing` pair, uncomment
reconstructing the original byte-exactly (journal gate 4a). Optimize-verb
move parity is NOT scope of this milestone: it is a registered follow-on
(ledger FOLLOW-ON INTAKE row, 2026-10-24; unfreezing the path-derivation is
"out with a reason" per kickoff section 4).

### RIDER 2 disposition (item 4) - RE-AUTHOR to a machine-true DEFERRAL lock (withdrawal rejected)

Withdrawal (plain `git restore` of m19) is rejected: the closed clause
(kickoff section 7 item 2) authorizes exactly "additive original-file proofs"
inside this one named test - a deferral lock about `include/thing.h` stays
inside that authorized hunk surface, and a withdrawal would document the
deferral nowhere in the machine. The re-authored hunk stays pure additions
against start rev `790fe75` (no start-rev byte deleted; the current working
tree's proof-block bytes NEVER reached git history, so the reviewer's
per-commit diff of the landing vs `790fe75` sees only the NEW block).

Byte-exact working-tree replacement (Edit tool only; replace EXACTLY this
block - the current file's lines 160-185, from the after-capture through the
reconstruction assert, inside
`optimize_writes_all_extracted_headers_and_rewrites_consumer_includes`; the
before-capture line 115 `let thing_header_before = ...` STAYS, lines
157-159 (report counts + blank) STAY, `Ok(())` and the closing brace STAY;
the `uncomment_marked_regions` local fn is REMOVED from m19 - an unused
local fn is a dead-code warning, and m20 keeps its own separate copy):

OLD bytes (current working tree, verbatim):

```rust
    let thing_header_after = fs::read_to_string(repo_root.join("include").join("thing.h"))?;
    fn uncomment_marked_regions(text: &str) -> String {
        let mut restored = String::new();
        let mut inside_region = false;
        for line in text.split_inclusive('\n') {
            let line_body = line.trim_end_matches(['\r', '\n']);
            if line_body.starts_with("// ARIADNE-EXTRACT-OUT BEGIN ") { inside_region = true; continue; }
            if line_body.starts_with("// ARIADNE-EXTRACT-OUT END ") { inside_region = false; continue; }
            restored.push_str(if inside_region { line.strip_prefix("//").unwrap_or(line) } else { line });
        }
        restored
    }
    assert!(
        thing_header_after.contains("// ARIADNE-EXTRACT-OUT BEGIN plan="),
        "extraction-symbol-move: the optimize verb moves identically (one hook fixes all \
         verbs) - the header gained a marker (prefix-only pin; m20 owns the PLAN-001 byte)"
    );
    let begin_markers: Vec<&str> = thing_header_after.lines().filter(|line| line.starts_with("// ARIADNE-EXTRACT-OUT BEGIN ")).collect();
    let end_markers: Vec<&str> = thing_header_after.lines().filter(|line| line.starts_with("// ARIADNE-EXTRACT-OUT END ")).collect();
    assert_eq!(begin_markers.len(), end_markers.len(), "the marked regions are balanced BEGIN/END pairs");
    assert_eq!(
        uncomment_marked_regions(&thing_header_after),
        thing_header_before,
        "kickoff section 1 layer-2 reversibility: uncommenting the marked region \
         reconstructs the original file byte-exactly"
    );
```

NEW bytes (verbatim - pure ASCII, 15 lines):

```rust
    let thing_header_after = fs::read_to_string(repo_root.join("include").join("thing.h"))?;
    assert_eq!(
        thing_header_after,
        thing_header_before,
        "extraction-symbol-move DEFERRAL lock (SM-3 AMENDMENT-1, 2026-10-24; machine-true \
         per STOP finding 1): the optimize verb marks NO original file - \
         relocate_extraction_to_generated (crates/cli/src/optimize.rs:320) spells the \
         migration with the generated/ prefix while apply_with receives the UN-relocated \
         extraction (:407), so the frozen derivation (collect_move_removals, \
         crates/validate/src/lib.rs:362, SM-I4 ZERO-DIFF) yields ZERO move candidates and \
         the move hook is never offered. If this assert ever FAILS, the ledger-registered \
         follow-on 'optimize-verb move parity' (path-derivation unfreeze) has landed: \
         replace this lock with the marked-retention proofs m20 witnesses for the apply \
         verb - never delete it silently"
    );
```

Test-count law holds: m19 keeps 5 tests, m20 keeps 6 - census-by-name
reconciliation stays mechanical and the SM-3 delta stays +0.

RGB adjudication (reviewer MUST read): the deferral lock is a
machine-truth CHARACTERIZATION (Green-at-birth), not a new-truth Red: its
Red is witnessed by gate 2a (the original unsatisfiable claim failing) plus
gate 4b (post-thread, the header unmarked), its Green is the re-run below;
justified-Green-at-birth precedent is SM-1's justified-Red-skip clause and
SM-4's declared trio. The "never weaken an assertion" law is satisfied: the
replaced claim was machine-false (witnessed by the milestone's own frozen
locks), the replacement asserts witnessed machine truth AND is an
unfreeze-tripwire that FAILS when the follow-on lands - a strengthening,
never a weakening. If the re-run goes red for ANY reason other than this
lock's own message: STOP-and-report, do not touch it.

### Reduced gates for the landing under this amendment

Gates 1, 2a, 2b, 3, 4a, 5, 6, 8, 9 and `encoding_guard --all` are WITNESSED
in the journal and stand as evidence (the reviewer independently re-runs
them fresh). Amended expectations:
- Gate 4b' (new): `python tools/testrun.py --distill -p
  ariadne-integration-tests --test m19_optimize_characterization_test` -
  exit 0, `5 passed / 0 failed / 0 ignored / 1 suites` (deferral lock
  green).
- Gate 7 (full workspace census): EXEMPTED from this landing by this
  amendment - hard chain, see below. The journal's gate-7 stop witness
  (`259 passed / 1 failed / 0 ignored / 3 suites`, fail-fast at the ariadne-cli
  lib binary) is the designed-red record of reference.
- Resumed-session gate budget: ONE scoped testrun invocation + the commit
  cycle trio. The journal's D-2026-10-24 overshoot row (16 measured vs 9
  declared) stands and travels to the ledger dated sub-block at closeout;
  the implementer's recommendation (re-state multi-test gates as per-test
  invocations milestone-wide) is ACCEPTED for all future hands.

### HARD CHAIN SM-3 -> SM-4 - AUTHORIZED (designed red, disclosed)

Finding 2 is machine-true: the pre-existing UNEDITED lock
`crates/cli/src/commands/ui/extraction_apply/tests.rs:300-304` (required
test 1, `test_a_valid_token_applies_the_plan_end_to_end`) asserts modified
inside the approved dry-run consumer set; the sanctioned threading marks
`include/thing.h` on the UI apply path (same spelling as m20), so the test
fails with verbatim panic (journal gate 7): `the apply must not modify any
file outside the approved dry-run's consumer_file set: modified=
["...include\\thing.h", "...src\\thing.cpp", "...tests\\consumer.cpp"]`.
The shape assigns the grown-oracle truth to SM-4 (SM-I6: "the one changed
test file is required test 1's module tree"; C5: "the original file becomes
a modified file"); kickoff section 7 item 3's relay phrase "tests 1-4
UNEDITED" is SUPERSEDED for that clause by shape C5 + SM-I6 (planner
adjudication, recorded). `crates/cli/**` stays SM-3-forbidden.

Authorization wording: the planner AUTHORIZES a designed HARD CHAIN
SM-3 -> SM-4 (precedent: ui-extraction-card-playbook T-A/T-B designed-red
window, O1 disclosed): SM-3 lands validate threading + m20 rider + m19
deferral lock carrying EXACTLY that one workspace red, disclosed in this
amendment, the journal, and the ledger; SM-4's hop discharges the workspace
gates for BOTH hands (gate-1 names the known red; the C5 growth discharges
it at authoring; gate 6 full census `1852/0/13/119` proves both; the SM-4
verdict names both landing commits). Window law: no release between; SM-4's
code landing is the immediate next landing (duty-1 doc landings allowed in
between; reviewer adjacency check).

MERGE SM-3+SM-4 REJECTED on the sizing law: the merged hand would carry 17+
scoped gate invocations against the ~10 cap - shape section 6's own "IF the
merged hand still fits the law" condition fails - and the rider Red would
have to be re-choreographed around the reader flip, which the serial order
gives for free (shape section 6, paraphrased: "the m19/m20 rider's Red must
then run BEFORE the reader flip lands"). One-crate-per-hand scope
discipline also argues against shipping validate inside a cli hand.

### Landing commit - revised message (scopes unchanged)

```
python tools/commit_scoped.py --scope crates/validate/src/lib.rs --scope
tests/integration/tests/m20_apply_migrate_characterization_test.rs --scope
tests/integration/tests/m19_optimize_characterization_test.rs --scope
tasks/extraction-symbol-move-sm3-journal.md -m "feat(validate): bind the
plan id into the move hook at the strip call; m20 marked-retention rider +
m19 optimize-deferral lock; hard-chained to SM-4 (extraction-symbol-move
SM-3)"
```

### Exact landing sequence (resumed SM-3 implementer)

1. `python tools/checkpoint_guard.py resume --spec
   tasks/extraction-symbol-move-sm3-handoff.md --scope crates/validate
   --scope tests/integration/tests --cmd python tools/testrun.py --distill
   -p ariadne-integration-tests --test m19_optimize_characterization_test`
   - bootstrap; journal the boot lines + confirm the disk-state inventory
   matches the journal (HEAD `790fe75`; dirty exactly validate/m19/m20 +
   this journal + tolerated .vsix).
2. Edit ONLY `tests/integration/tests/m19_optimize_characterization_test.rs`:
   replace the OLD block quoted above with the NEW block, byte-exactly
   (Edit tool; never Set-Content/Out-File/`>`).
3. Gate 4b' (the one scoped invocation above): expect exit 0,
   `5 passed / 0 failed / 0 ignored / 1 suites`.
4. Journal: append the AMENDMENT-1 execution record (amendment reference,
   gate 4b' canonical line + capture path, R5 statement carried forward,
   and the HARD-CHAIN statement: main stays red on the single named test
   until SM-4's landing discharges it).
5. `python tools/encoding_guard.py --staged` (or the `--all` superset if the
   staging seam refuses - journal the form used), then the revised
   commit_scoped command above. Never `git add .`; the journal rides in the
   landing commit.
6. Report back per the handoff's "Report back" section with two deltas:
   gate 7 omitted by amendment (hard chain), and Red evidence = gates 2a/2b
   + gate 4b stop witness + gate 4b' Green.
