# SM-4 Handoff - extraction-symbol-move: dry-run honesty via the strip-fold mirror + the grown faithfulness oracle (tests/move_proof.rs) + the generation-swap relay (record, NOT execute)

Milestone: `extraction-symbol-move` (UNREGISTERED post-M28 track, no
M-number). Hand 4 of 5, strictly serial: SM-1 -> SM-2 -> SM-3 -> SM-4 -> SM-5.
You land IMMEDIATELY after SM-3 (risk R5: between SM-3 and you the dry run
under-reports while apply marks - closing that window is your first duty).
Your dry run and SM-3's apply must agree by the SM-I3 parity; your grown
oracle is the milestone's end-to-end machine witness (shape C5) and the
independent witness standing under SM-3's riders (R4). Author rev `2177a85` on
`main` (planner-verified 2026-10-23; your baseline is the SM-3 landing state:
workspace `1848 passed / 0 failed / 13 ignored / 119 suites`). Start rev: your
dispatch head - journal it verbatim; every reviewer diff is against it.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-symbol-move-kickoff.md` section 4 (the GENERATION-SWAP
   decision of record: the touch trigger ARRIVES at your hand and is
   DELIBERATELY NOT EXECUTED - your journal records the relay; the closeout
   appends the dated row), section 7 item 3 (YOUR amendment: reader required
   test 5 flips to move truth; tests 1-4, 6-10 UNEDITED).
3. `tasks/extraction-symbol-move-shape.md` C4 + C5 (your contracts), invariants
   SM-I2/SM-I3/SM-I6, risk R3 (the `tests.rs` 903-line ceiling - your growth
   goes to the submodule, `tests.rs` gains ~1 line of mod + minimal call
   sites), risk R5, section 8 (the watch this milestone owns).
4. `tasks/extraction-symbol-move-sm2-handoff.md` "PLANNER VERDICT ON THE
   FIXTURE OUTCOME" - the fixture marks exactly ONE merged region
   (`class Thing`, `symbols=Thing`, `PLAN-001`); your oracle pins THAT, never
   the shape's relayed "4 pairs".
5. `docs/specs/unit-test-audit.md` section "Deferred findings" - MANDATORY
   intake read. Planner intake result 2026-10-24: zero deferred items name
   `extraction_reader.rs` or `extraction_apply/**` (the cli items name the
   transform/optimize payload surfaces, frozen here). Carry any new finding;
   report, do not fix.
6. Your production sites (planner-verified today): `crates/cli/src/commands/
   ui/extraction_reader.rs` (899 lines) - `move_removal_count` :232-262 with
   the old-hook offer at :254-256 and its doc :225-231;
   `fn move_removal_candidates` :356 (private; mirrors validate's
   `collect_move_removals` exactly - doc :350-355); the DTO construction
   :174-192 (`copy_only: move_removals == 0` - UNEDITED, no DTO field moves);
   the degraded honest-absent shape :420-425 + its pin tests :877-886
   (UNEDITED); required test 5 :779-796.
7. PLANNER NOTE (dated 2026-10-24, F14-class shape relay correction, binding):
   shape C4 says the dry run "offers the SAME hook" - offering per candidate
   independently breaks SM-I3 for overlapping spans (pre-apply, ALL FOUR
   fixture candidates answer `Some`; post-apply only ONE region exists - the
   ctor/method refusals fire only against the already-marked text). Your
   contract: `move_removal_count` becomes a FOLD that mirrors
   `strip_moved_symbols` exactly: iterate the candidates in `BTreeSet` order,
   keep a per-path running-text cache, offer the CURRENT fold text to
   `move_out_top_level_symbols(&current, &names, &plan.id.0)`, on `Some`
   replace the cached text and add `names.len()`. The fold is still "computed,
   not asserted" (W4) and read-only (nothing is written); it equals exactly
   what the apply will bind. Fixture outcome of your fold: `move_removals = 1`
   (the class entry), `copy_only = false`. Your doc lift says the fold mirrors
   the strip order so the dry-run move claim equals the markers apply binds
   (SM-I3).
8. Your oracle sites (planner-verified today): `crates/cli/src/commands/ui/
   extraction_apply/tests.rs` (903 lines; `#[cfg(test)] mod tests;` from
   `extraction_apply.rs`, so a child `mod move_proof;` resolves to
   `extraction_apply/tests/move_proof.rs` - the `symbol_locator.rs` + `mod
   tests;` tree idiom is your in-repo precedent) - required test 1
   `test_a_valid_token_applies_the_plan_end_to_end` :233-331 (before/after
   walks :251/:258, expected set :253, audit call :306-308, outcome DTO block
   :310-330 UNEDITED); helpers `expected_modified_consumers` :345-380 and
   `assert_modified_faithfully_applied` :392-421; `walked_files`,
   `modified_relative`, `written_relative`, `canonicalized_lossy`,
   `is_artifact_tree` - all stay put and byte-identical.
9. `python tools/checkpoint_guard.py status` - bootstrap before any mutation.
   Expected clean tree at the SM-3 landing except the pre-existing dirty
   `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`; anything
   beyond it is a STOP-and-report.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/extraction-symbol-move-sm4-journal.md (checkpoint journal; create it;
  carries the generation-swap relay line - see "Generation-swap relay")
- crates/cli/src/commands/ui/extraction_reader.rs (the fold + hook switch +
  doc lift + the declared `pub(crate)` visibility raise of
  `move_removal_candidates` + required test 5 amendment)
- crates/cli/src/commands/ui/extraction_apply/tests.rs (+1 `mod move_proof;`
  line + the MINIMAL required-test-1 call-site changes; `tests.rs` growth
  capped at +15 lines total - anything bigger belongs in the submodule; the
  903-line watch this milestone owns)
- crates/cli/src/commands/ui/extraction_apply/tests/move_proof.rs (NEW: the
  C5 oracle helpers + 3 non-vacuity tests; ~300-340 lines, well under 1000)

You MUST NOT touch: `crates/cli/src/commands/ui/extraction_apply.rs` (the 295
lines stay - your growth is test-side; that file's touch trigger for the
generation-swap row RELAYS, it does not EXECUTE), `crates/cli/src/commands/
transform.rs` (frozen), every other file under `crates/cli`, `crates/ui/**`,
`crates/validate/**`, `crates/lang*/**`, `crates/transform/**`, DTO bytes
(no new DTO field - the X3-class frozen display survives), token/mutex/epoch
machinery, W9 boot-payload lock, required tests 2-11, the honest-absent shape
+ pin, `tests/fixtures/**` (byte-frozen), `tools/**`. No `git stash`, no
branch change. Cargo.lock must not appear in your diff.

## What to build (shape C4 + C5, with the binding fold refinement from
binding input 7)

1. Reader (C4 + SM-I3): rewrite `move_removal_count` as the fold above; doc
   lift states the mirror + plan-id marker provenance + "computed, not
   asserted"; `plan.id.0` renders the SAME `PLAN-001` the marker binds
   (m20's byte truth). Raise `move_removal_candidates` to `pub(crate) fn`
   (declared, mechanical, zero behavior - your submodule reads the same
   derivation validate runs inside apply; SM-I6: no public surface moves).
2. Required test 5 amendment (kickoff section 7 item 3): rename
   `test_dry_run_reports_copy_only_honesty_on_the_cpp_fixture` ->
   `test_dry_run_reports_move_truth_on_the_cpp_fixture` (declared amendment -
   the old name states the superseded truth, the m19 CW-1 rename precedent)
   and flip the body: `assert!(dry_run.move_removals > 0, "...the C++ move
   hook retires the fixture's locatable top-level declaration(s) under
   ARIADNE-EXTRACT-OUT markers...")` + `assert!(!dry_run.copy_only, ...)`;
   journal your MEASURED count (planner-verified expectation: 1). The
   honest-absent pin's `move_removals == 0 && !copy_only` shape (:881-882)
   stays UNEDITED. Tests 1-4, 6-10 UNEDITED.
3. `move_proof.rs` helpers (shape section 8 names): `strip_marker_prefix`,
   `uncomment_regions`, `assert_marked_regions`,
   `expected_modified_with_move_sources` (consumers-with-hits UNION retirable
   move sources - both sets read PRE-apply, host-canonical via
   `canonicalized_lossy`, sorted; retirable = `move_out_top_level_symbols`
   answers `Some` on the PRE-apply text over the reader's
   `move_removal_candidates` mirror). Marker proof per marked original file
   (C5.4, from the before/after walks): balanced BEGIN/END pairs, pair count
   == hook-success calls, every marker line carries the approved plan id,
   prefix-stripping each region reconstructs the pre-apply span bytes
   byte-exactly, every marker byte ASCII, re-offering the hook over the
   POST-apply text with the same names answers `None` (the region is inert;
   SM-I1's no-live-declaration condition).
4. Required test 1 growth (C5 conditions 1-5; inline edits in `tests.rs`
   MINIMAL, helpers do the work):
   - condition 1/2: `expected_modified :=
     move_proof::expected_modified_with_move_sources(...)` (consumers-only
     list from the existing `expected_modified_consumers` survives as the
     sub-claim input); audit gains
     `move_proof::assert_modified_faithfully_applied`-style claims:
     `modified == union` (no silent drop, nothing outside); the fixture
     equality `modified == approved_consumers UNION
     {canonicalized_lossy(include/thing.h)}`; the consumer-only equality
     `expected consumers == approved_consumers` survives as the sub-claim -
     the :399-410 asserts keep their claim shapes, their inputs widen
     (declared hunk, message cites SM-I2 + the surviving sub-claim);
   - condition 3: `written == approved_new_files` and the no-deletions claim:
     byte-identical UNEDITED claims (:260-271 and the third audit claim);
   - condition 4: `move_proof::assert_marked_regions(...)` over the walk pair
     for `include/thing.h` (fixture truth: ONE merged pair, `symbols=Thing`,
     `plan=PLAN-001`);
   - condition 5 (SM-I3 parity): `dry_run.move_removals ==` the total symbol
     names bound into the written markers (fixture: 1 == 1);
   - condition 6 is NOT re-proven here: rollback coverage is
     `move_removal_transaction_test.rs` green UNEDITED (validate's suite,
     gate 4).
5. Three new tests in `move_proof.rs` (your +3):
   `move_removal_union_helper_matches_the_fixture_derivation` (PRE-apply
   state: union == approved consumers UNION {canonical thing.h} - derived,
   nothing applied), `assert_marked_regions_rejects_an_unbalanced_pair`
   (synthetic), `assert_marked_regions_rejects_tampered_region_bytes`
   (synthetic; prefix-strip reconstruct must fail the byte equality).
6. Generation-swap relay (RECORD, NOT execute): the ledger row 445 trigger
   ("next `extraction_apply.rs` test touch") fires at your touch of
   `extraction_apply/tests.rs`. You do NOT add any generation-swap test
   (kickoff section 4: out, by decision; the touch is to the tests module,
   not `extraction_apply.rs`, and the relay records that too). Journal line
   (verbatim, with your hashes): "ledger 445 generation-swap trigger re-fired
   at SM-4 via `extraction_apply/tests.rs` touch; NOT executed per kickoff
   section 4; closeout appends the dated relay."

## Frozen-lock amendment clause (kickoff section 7 item 3 - binding)

You may edit required test 5 (rename + body flip) and required test 1's own
module tree (the C5 growth + the two helper call sites) - nothing else in the
existing test tree. Reviewer per-commit MUSTs:
- Required tests 2-11, the honest-absent pin, W9 boot-payload lock: no hunk
  (SM-I6; the one changed test file is required test 1's module tree).
- The amended assertions must first FAIL on the pre-hand tree for the
  documented reason (captured Red below).
- The grown oracle is the machine witness for SM-I2/SM-I3; SM-3's riders are
  NOT re-touched by you (report, do not fix, if they wobble - STOP-and-report).
- Discovery of any OTHER pre-existing lock wanting an edit: STOP-and-report
  (R8 / CW-3 precedent).

## RGB protocol

1. Red: author the required-test-5 amendment and the required-test-1 growth
   FIRST (call sites pointed at `move_proof::` helpers; the submodule exists
   at Red containing helpers + the three non-vacuity tests so everything
   COMPILES - a test that fails to compile is not a Red). The `pub(crate)`
   raise lands with this Red commit (mechanical visibility, zero behavior).
   Expected, for the documented reason: required test 5 fails (pre-fold the
   reader offers the old hook per candidate -> all `None` -> `move_removals
   == 0`); required test 1 fails the grown `modified == union` equality (the
   pre-fold dry-run and the un-folded expectation: thing.h is in expected
   because the hook already exists, in actual NOT - apply binds it, and
   `assert_modified_faithfully_applied`'s widened equality catches the
   consumer-only reality). EXACTLY two named failures; journal the panic
   lines. Declared Green-at-birth trio: the three `move_proof.rs` tests
   cannot exist before their helpers (pure synthetic-byte self-checks of the
   proof helpers - non-vacuity guards, not behavior claims); declare them in
   the journal with this justification (SM-1's justified-Red-skip clause is
   the precedent). NEVER weaken an assertion to pass.
2. Green: the fold + hook switch + doc lift; re-run gates 3-6 until green.
3. Refactor: plain docs, honest names; re-run.

## Hygiene (review-enforced)
`tests.rs` 903 -> <= 918 (the ceiling WATCH: mod line + call sites only,
marker assertions NEVER inline - shape section 8 names this the watch this
milestone owns); `move_proof.rs` ~300-340; `extraction_reader.rs` 899 -> ~910
(margin noted, no split owed); functions < 100 lines (split `move_proof`
helpers until true); no banned names; authored bytes pure ASCII.
`python tools/hygiene_check.py` exit 0 (it IS the R3 oracle).

## Frozen locks (commands that prove them)
- Required tests 2-11 + honest-absent pin + W9 boot lock green UNEDITED:
  inside gates 1/3/6; reviewer per-commit diff (no hunk outside the touch-set
  regions).
- DTO bytes + `copy_only: move_removals == 0` construction UNEDITED (SM-I3's
  "keeps its computed meaning"): `crates/ui/src/dto/**` in no diff.
- `transform.rs` frozen (shape section 0): no hunk. `extraction_apply.rs` 295
  lines stay: `git diff --numstat <start rev> HEAD --
  crates/cli/src/commands/ui/extraction_apply.rs` EMPTY - journal it (this is
  the generation-swap relay's byte proof too).
- Consumer half byte-truth (SM-I4): m18 parity green UNEDITED (gate 5).
- Rollback coverage (SM-I7): validate suites green UNEDITED (gate 4).
- goldens/fixtures byte-locked; `tools/**` operator-law; Cargo.lock in no diff.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only;
raw cargo is a reviewer blocker)
1. `python tools/testrun.py --distill -p ariadne-cli` - clean baseline BEFORE
   any edit; journal the canonical line verbatim.
2. `python tools/testrun.py --distill -p ariadne-cli` - Red: exit 1, EXACTLY
   the two named amended tests failing (journal the two panic lines + the
   three at-birth tests listed green with their declaration).
3. `python tools/testrun.py --distill -p ariadne-cli` - Green: exit 0,
   passed = baseline + 3 (the three `move_proof` tests), the amended pair
   green, tests 2-11 + honest-absent + W9 green UNEDITED.
4. `python tools/testrun.py --distill -p ariadne-validate` - exit 0 UNEDITED
   (includes `move_removal_transaction_test` - SM-I7 condition 6).
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m18_cpp_plugin_parity_test --test m00_facts_path_parity` - exit 0 UNEDITED
   (R9: boots/parity run on the UNAPPLIED fixture, post-apply boots differ by
   design - stated, not locked further).
6. `python tools/testrun.py --distill` - full workspace census; reconcile from
   your journal baseline and attribute every delta (expect +3 passed, 0
   failed, 13 ignored, 119 suites held; upstream of record: SM-3 landing
   `1848 passed / 0 failed / 13 ignored / 119 suites`; anchor
   `1837 passed / 0 failed / 13 ignored / 119 suites @ b201aa6`; the
   `--baseline` FILE form stays superseded by this `--distill` substitute).
7. `python tools/distill.py -- python tools/bench_compare.py` - fresh
   recording (kickoff sec. 5 mandates it per hand); WARNs recorded, NOT
   attributed (PR-2 persists).
8. `python tools/hygiene_check.py` - exit 0 (the R3 tests.rs-ceiling oracle -
   journal the `tests.rs` line count it saw).
9. Commit cycle (itemized per the 2026-10-06 M28 law):
    `python tools/encoding_guard.py --staged` (or the `--all` superset if the
    staging seam refuses - journal the form used) then
    `python tools/commit_scoped.py --scope
    crates/cli/src/commands/ui/extraction_reader.rs --scope
    crates/cli/src/commands/ui/extraction_apply/tests.rs --scope
    crates/cli/src/commands/ui/extraction_apply/tests/move_proof.rs --scope
    tasks/extraction-symbol-move-sm4-journal.md -m "feat(cli): dry-run move
    honesty via the strip-fold mirror; grown move-proof oracle; required test
    5 move truth (extraction-symbol-move SM-4)"`. Never `git add .`; the
    journal rides inside the landing commit.

Declared gate budget (2026-10-06 M28 law): 8 scoped invocations + the commit
cycle - inside `<= ~10 + commit-cycle trio`; itemize per-gate counts in the
journal and declare any overshoot as a dated D-row (silence is the deviation).

## Discipline clauses
- UTF-8: never Set-Content/Out-File/`>`/`Get-Content | -replace`; Write/Edit
  tools only; authored content pure ASCII (AGENTS.md File-Edit Discipline);
  every marker specimen byte ASCII (A9).
- Touch only the touch-set; out-of-scope breakage: report, do not fix. Any
  pre-existing lock discovered wanting an edit: STOP-and-report (CW-3
  precedent).
- The generation-swap relay is RECORD-ONLY (kickoff section 4): adding a
  generation-swap test is OUT-OF-SCOPE and a touch-set violation.
- After EACH Green gate append a 1-3 line checkpoint to the journal.
- Context exhaustion looming: STOP, return a disk-state inventory; a dead
  session is never resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-symbol-move-sm4-handoff.md --scope crates/cli --cmd ...`.
- Never pin warning counts; never state a count without the command that
  produces it (M23.5 F14 standing rules).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence: the two named failures
with verbatim first-assert panic lines + the declared Green-at-birth trio.
3. Green evidence: canonical lines gates 3-6 + capture paths, the empty
`extraction_apply.rs` diff proof, the `tests.rs` line count, your measured
fixture move count (expect 1). 4. The generation-swap relay line as journaled.
5. Any deviation + reason. 6. Out-of-scope observations (report-only). Never
paste raw tool output; cite command + canonical line + exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill. Extra MUSTs for this hand: `extraction_apply.rs` diff-EMPTY; `tests.rs`
hunks confined to the mod line + required test 1's tree (<= 15 net added
lines); no DTO hunk; required tests 2-11 + honest-absent + W9 diff-EMPTY;
the fold mirrors `strip_moved_symbols` order (read `validate/src/lib.rs:382
-395` against the reader's loop in the diff - this is the SM-I3 adjudication);
census +3 attributed; the R5 window is closed at your landing (journal shows
SM-3 then SM-4 with nothing landed between - reviewer checks adjacency).

Sizing (planner, itemized from the mandated bodies - M23.5 rule 4): reader fold
+ doc + switch ~+40/-20; test 5 amendment ~+10/-8; `tests.rs` ~+15;
`move_proof.rs` ~300-340 (helpers ~170, 3 tests ~130-160); journal ~18 -
total ~380-430 produced lines vs the ~1200 bound; 8 scoped gates + commit
cycle vs ~10 + trio; one crate dir. No hand split: the fold without the grown
oracle ships honesty no one proves; the oracle without the fold pins an
equality (SM-I3) that fails - both half-states re-open R5's window.
