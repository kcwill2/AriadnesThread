# SM-5 Journal - extraction-symbol-move: move-truth copy + S5 keying amendment + marked-retention no-rot lock

Handoff: `tasks/extraction-symbol-move-sm5-handoff.md` (author rev `2177a85`).
Dispatch head (start rev): `e717ba0` (checkpoint_guard status, verbatim:
`sha: e717ba0`). Dirty at boot exactly:
` M extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`,
` M tasks/milestone_ledger.md`,
`?? tasks/extraction-symbol-move-sm4-execution-results.md`,
`?? tasks/extraction-symbol-move-sm4-verdict.md` - expected set, tolerated vsix.

## Duty-1 (SM-4 doc landing)

Ledger-recorded command run as ONE line:
`python tools/encoding_guard.py --staged` (silent, exit 0) then
`python tools/commit_scoped.py --scope tasks/milestone_ledger.md --scope
tasks/extraction-symbol-move-sm4-verdict.md --scope
tasks/extraction-symbol-move-sm4-execution-results.md -m "docs(extraction-symbol-move):
SM-4 APPROVED - land verdict + execution-results + ledger SM-4 record (census 1852, hard chain closed)"`.
Landed `fd96b09`; `git show --stat HEAD` = exactly the 3 paths;
`encoding_guard --all` silent exit 0; porcelain-after = `.vsix` only.
Hand work bases on `fd96b09`.

## Intake reads

- kickoff sec. 1 layer 3: post-commit FULL reversibility explicitly NOT
  promised; restore = uncomment + revert consumer include flips, or git;
  never a one-click claim. Shipped copy says exactly that.
- kickoff sec. 7 item 4: my amendment - `test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path`
  S5 clause unconditional -> keyed; sentence constant byte-identical.
- shape C6: CHANGE = :359, :392, :430, :575-clause. STAYS list below.
- shape C7 item 5 CLOSED NO-OP: verified at touch -
  `git grep -n "in place for move languages" -- crates/ui/tests` exit 1 (no
  pin). The :575 reword is a plain copy edit.
- unit-test-audit.md "Deferred findings": zero deferred items name the wizard
  asset or `extraction_wizard_render.rs` copy locks (planner intake confirmed;
  nothing new to carry, report-only).
- Touch sites Read-verified against current bytes: :219 constant, :353-357
  branches, :359, :386, :388-392, :426-432, :575, test :331, :400-429,
  :631-693 A-5 locks, :864-890 CW-3 no-rot locks - all as the handoff maps.
- "strips" in the asset occurs exactly at :359 and :392 (both replaced by the
  mandated edits); nowhere in `crates/ui/tests` is "strips"/"This plan"/"as
  it writes"/"in place for move languages" pinned.

## STAYS list (shape C6 - audited true under move truth; do NOT "fix")

- The claim of record "the UI applies with a dry-run first, validation in
  three tiers and automatic rollback" (hotspots.js + its lock :875).
- `COPY_VS_MOVE_SENTENCE` constant bytes (asset :219 + test :331) - true on
  the `move_removals == 0` fallback branch; no hunk touches either.
- The S4 truncation note :386 + its CW-3 no-rot lock (`test_truncation_note_
  states_the_shipped_truth`) - consumer truth unchanged (CW-1).
- The tier/rollback S6 lines :571-574 - the original file IS in the snapshot
  now, truer than before.
- "The dry run changes nothing; tool apply performs the same computation
  before writing." (asset :399).
- `wizardCopyOnly` body :306-309 - its lock reads the body.

## Gate 1 (baseline before edits)

`python tools/testrun.py --distill -p ariadne-ui` ->
`371 passed / 0 failed / 0 ignored / 3 suites` exit 0 (PASS).
Workspace baseline per handoff: `1852 passed / 0 failed / 13 ignored /
119 suites` (SM-4 landing); expectation `1853/0/13/119` after my +1 lock.

## Gate 2 (Red) - asset untouched, test file authored first

`python tools/testrun.py --distill -p ariadne-ui` -> exit 1,
`370 passed / 2 failed / 0 ignored / 2 suites` - EXACTLY the two named
failures. Verbatim panic lines:
- `thread 'extraction_wizard_render::test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path' (23088) panicked at crates\ui\tests\ui_test\extraction_wizard_render.rs:434:5:` / `amendment 4 (extraction-symbol-move): S5 must key its copy sentence off wizardCopyOnly like S4 - the copy sentence is false on the move branch`
- `thread 'extraction_wizard_render::test_move_sentence_states_marked_retention_truth' (12668) panicked at crates\ui\tests\ui_test\extraction_wizard_render.rs:916:5:` / `the asset must carry the exact marked-retention sentence (the value lives contiguous on the asset's ONE-line const, same idiom as the COPY_VS_MOVE_SENTENCE pin)`
Capture: `target/testrun-capture.txt` (422 lines). No assertion weakened.

## Gate 3 (Green)

`python tools/testrun.py --distill -p ariadne-ui` -> exit 0,
`372 passed / 0 failed / 0 ignored / 3 suites` = gate-1 baseline +1 (the new
lock; amended test count-neutral). Every existing lock green UNEDITED: fetch
bound `<= 2`, S4 truncation note, P9/P10 hotspots, ASCII, theme contract, the
six surviving amended asserts, the three A-5 S5 locks. Byte-pin self-verify:
Read confirmed the pinned sentence contiguous on asset :224 ONE-line const.

## Gates 4-6

- Gate 4 `python tools/distill.py -- python tools/ui_smoke.py` -> exit 0,
  `ui_smoke: PASS` (capture `target/distill-capture.txt`). Served asset bytes
  changed; boot + routes + no-store hold; no byte-exact asset/served lock
  fired (the kickoff grep-proof stands).
- Gate 5 `python tools/theme_token_check.py --check-added --base e717ba0` ->
  silent exit 0 (zero added hex; combined form).
- Gate 6 `python tools/testrun.py --distill` -> exit 0,
  `1853 passed / 0 failed / 13 ignored / 119 suites` =
  journal baseline 1852 (SM-4 landing, duty-1 verified) + my +1 new lock.
  Reconciles from anchor `1837/0/13/119 @ b201aa6`: SM-1 +2, SM-2 +10,
  SM-3 +0, SM-4 +3, SM-5 +1 = 1853 = the MILESTONE expectation. No foreign
  residue; suites count steady at 119 (test appended in-file, no new suite).

## Gates 7-9 + landing

- Gate 7 `python tools/distill.py -- python tools/bench_compare.py` ->
  exit 1, fresh recording with ONE WARN:
  `WARN get_or_build_include_graph clone (warm): 1.3 ms vs baseline 1.0 ms (+30%)`.
  Recorded, NOT attributed (PR-2 persists, handoff mandate; warning counts
  never pinned). Producer zero-diff proof:
  `git diff --numstat e717ba0 HEAD -- crates/cli/tests` -> EMPTY (CW-3 gate-7
  precedent). UI copy hand touches no producer code.
- Gate 8 `python tools/hygiene_check.py` -> exit 0 (distilled run:
  `### generic PASS (exit 0)`; R1/R2 clean; the listed R3 warns are
  pre-existing, none in this hand's files).
- Gate 9 commit cycle, form used (journal per handoff):
  `python tools/encoding_guard.py --staged` (silent, exit 0) then
  `python tools/commit_scoped.py --scope crates/ui/assets/extraction-wizard.js
  --scope crates/ui/tests/ui_test/extraction_wizard_render.rs --scope
  tasks/extraction-symbol-move-sm5-journal.md -m "fix(ui): move-truth copy -
  marked retention sentences, keyed S5, marked-retention no-rot lock
  (extraction-symbol-move SM-5)"` -> landing `82cf789`, exactly the 3
  touch-set paths; `python tools/encoding_guard.py --all` silent exit 0
  after landing; porcelain-after = `.vsix` only.
- Refactor: none needed (copy + two lock sites); gate 3 not re-run beyond
  its Green run because no refactor edits were made.

## Diff audit vs duty-1 (`fd96b09..82cf789`)

- Asset: five hunks = the ONE new constant block + the four named copy sites
  (:359 1:1, :392 1:1, :430 -> keyed `moveTruthLine` block + concat line,
  :575 one clause 1:1). `COPY_VS_MOVE_SENTENCE` constant bytes and the
  `wizardCopyOnly` body appear in no +/- line (constant is context-only).
  Zero new `fetch(`, no class-name change, no rename, no hex.
- Render file: 57 insertions / 0 deletions - the appended const, the two
  amendment clauses inside the named test, the new lock test. All six
  surviving asserts byte-identical; all pre-existing locks untouched.
- Journal +96 (this file, riding the landing commit).

## Gate budget itemization (declared 8 + commit cycle)

Gates 1-8 = 8 scoped invocations (gate 2 red + gate 3 green share the same
gate; hygiene re-run via distill for the exit code = same gate, +1 total
invocation: 9 + commit cycle). Inside the 2026-10-06 M28 law bound
(~10 + commit-cycle trio). No overshoot; no D-row owed.

## Deviations

- None on content: every mandated string copied byte-exact (byte-pin
  self-verified by Read on asset :224 before the lock ran).
- Form note: duty-1's `--staged` ran pre-staging (index empty; SM-4 ledger
  form used verbatim), the `--all` superset additionally run after each
  commit - both silent exit 0.
- bench_compare exit 1 = the WARN form itself (the tool signals WARN);
  recorded per gate-7 mandate, not treated as a gate failure.

## Out-of-scope observations (report-only)

- Audit deferred-findings intake: no item names the wizard asset or this
  file's copy locks; nothing carried, nothing fixed.
- The gate-7 WARN (warm include-graph clone +30% at 1.3 ms vs 1.0 ms
  baseline) is the PR-2 noise class; closeout owns attribution.
