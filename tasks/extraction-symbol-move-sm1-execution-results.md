# extraction-symbol-move SM-1 - execution results (reviewer re-gate, 2026-10-24)

Reviewer-run acceptance record for `tasks/extraction-symbol-move-sm1-handoff.md`
(anti-phantom re-gate after PHANTOM-NARRATION-1). Every line below is this
reviewer's own run output, quoted verbatim. Verdict file:
`tasks/extraction-symbol-move-sm1-verdict.md` (APPROVED).

## Ground truth

- `.git/logs/HEAD` line 439 (file has exactly 439 lines), verbatim:
  `e5c49ec6a4891d195d0cbbf14c1ca271a15a1117 b9936000da7827e49e8dbe999f1dd65d65b75704 kcwill2 <kcwill2@gmail.com> 1792819353 -0700	commit: feat(lang): add the neutral move_out_top_level_symbols contract with a delegating default (extraction-symbol-move SM-1)`
- `python tools/checkpoint_guard.py status`: sha `b993600`, branch `main`;
  dirty: `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`
  (pre-existing stray) + `tasks/milestone_ledger.md` (uncommitted planner
  incident row, expected); untracked:
  `tasks/extraction-symbol-move-sm1-handoff-resume.md` (expected; reviewer
  owns disposition -> verdict recommends DELETE at closeout).

## Hand gates (handoff Gates 1-8), reviewer's own fresh runs

| # | Criterion | Reviewer evidence (verbatim) | Result |
|---|---|---|---|
| 1/2 | lang crate green with +2 pins | `python tools/testrun.py --distill -p ariadne-lang` -> `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 61 lines]` + `47 passed / 0 failed / 1 ignored / 2 suites` | MET (baseline 45 + 2; both pins named `... ok` in capture lines 32-33; old pin `... ok` line 39) |
| 3 | Scala zero-diff + green | `python tools/testrun.py --distill -p ariadne-lang-scala` -> `257 passed / 0 failed / 0 ignored / 2 suites` | MET (+ zero `crates/lang-scala` path in range diff) |
| 4 | validate delegation proof UNEDITED | `python tools/testrun.py --distill -p ariadne-validate --test move_removal_transaction_test` -> `2 passed / 0 failed / 0 ignored / 1 suites` | MET (file absent from range diff) |
| 5 | full census, delta attributed | `python tools/testrun.py --baseline` -> `usage error: missing value for --baseline` (superseded FILE form per F9); substitute `python tools/testrun.py --distill` -> `1839 passed / 0 failed / 13 ignored / 119 suites` | MET (1837 anchor +2; +2 both pins named `... ok` in the 3194-line census capture; 13 ignored / 119 suites held) |
| 6 | fresh bench recording, WARNs recorded not attributed | `python tools/distill.py -- python tools/bench_compare.py` -> `### generic PASS (exit 0)`; closing line `bench compare: 3 run(s), no regressions past 1.25x baseline`; full 17-line capture read: zero WARN lines | MET (nothing to attribute) |
| 7 | hygiene exit 0 | `python tools/hygiene_check.py` reviewed (R3 warn-only); exit machine-proven via `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)`; capture 173 lines, none under `crates/lang/src`, zero R1/R2 | MET (`rewriter.rs` 291 lines) |
| 8 | commit cycle inside touch-set, journal rides | `python tools/distill.py -- git diff --name-only e5c49ec b993600` -> exactly `crates/lang/src/rewriter.rs` + `tasks/extraction-symbol-move-sm1-journal.md`; `python tools/distill.py -- git show --numstat --format=short b993600` -> `74	4	crates/lang/src/rewriter.rs` / `210	0	tasks/extraction-symbol-move-sm1-journal.md`; commit message matches the handoff's mandated string | MET (`Cargo.lock` absent; ledger row correctly excluded) |

## Frozen locks

- `remove_top_level_symbols` signature/body + all seven pre-existing tests +
  `InertRewriter` + structs + other trait defaults: byte-identical (diff shows
  them only as context; current file read matched the `e5c49ec` blob read).
- Five trait-default pins green UNEDITED in my census capture (lines
  1390-1394, all `... ok`).
- Encoding: `python tools/encoding_guard.py --all` silent; exit proven via
  `python tools/distill.py -- python tools/encoding_guard.py --all` ->
  `### generic PASS (exit 0)` (capture 0 lines).
- Theme-token clause: N/A (no UI/theme files in touch-set; no theme path in
  the range diff).
- goldens/fixtures/tools/tests byte-identical: all absent from the range
  name-list.

## Workspace acceptance pass (reviewer duty beyond the hand)

- Full census: `1839 passed / 0 failed / 13 ignored / 119 suites` (exit 0,
  `--distill` form; `--baseline` FILE form refused with `usage error: missing
  value for --baseline` - recorded as the canonical refusal, substitute used
  per the handoff's own F9 supersession note).
- Tools suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 296 tests in 26.658s` / `OK`.
- UI runtime smoke: `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)` + `boot+route checks clean (cold fixture,
  attempt 1, 0.7s)` + `ui_smoke: PASS`. (Direct `python tools/ui_smoke.py`
  refused by the bash allow-list catch-all: `The user has specified a rule
  which prevents you from using this specific tool call. Here are some of the
  relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
  - same deferred allow-list row as prior milestones; the distill wrapper is
  the contracted generic-command route and was used.)
- Bench: run fresh this hop (see table row 6) - no machine-load WARNs
  occurred to record; zero-warn proof relied upon: the 17-line capture read
  verbatim, closing line quoted above.
- Hygiene + encoding: exit 0 each (wrapper-proven, table rows 7/8).

## Not run / not observable (explicit)

- `--baseline` canonical milestone-gate FILE form: refused by the tool itself
  (`usage error: missing value for --baseline`); not run in file form. The
  handoff pre-declares the `--distill` substitute; used it.
- `theme_token_check --check-added`: not applicable to this touch-set (no
  hex/asset paths in the range diff); not run.
- Exit codes for the two raw direct runs (`hygiene_check.py`,
  `encoding_guard.py --all`) were not directly echoable (allow-list refuses
  `Write-Output`/`python -c` probes with the same catch-all message); both
  exit codes were instead machine-proven through the distill wrapper, which
  prints `### generic PASS (exit 0)`; outputs also match the clean signatures
  (warn-only capture; empty capture).
- The redo's pre-commit work sha256 anchor could not be retroactively
  recomputed; moot because the worktree is clean at HEAD and my gates ran
  against the committed bytes.

## Ledger incident row (item 5 of the re-gate checklist)

Verified the uncommitted planner row (`git diff` unstaged, `cc0672d ->
d6e72ac`) against reality: landing SHA/parent, dirty-state claims, re-gate
status, resume-file ownership, reflog countermeasure - all TRUE. One number
wrong: `+78/-4` appears twice; measured truth is `74 4` (numstat above).
Correct before landing (verdict finding M1). Not committed by this reviewer.
