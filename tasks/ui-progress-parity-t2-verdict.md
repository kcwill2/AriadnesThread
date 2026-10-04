# Verdict - ui-progress-parity T2: BootStepSnapshot producer + terminal-parity oracle

Commit under review: `587c409` (single scoped commit; base `000238b`,
milestone base `e276910`). Reviewer hop 2026-10-04, via `review-gate`.

## VERDICT: APPROVED

Ground truth (`python tools/checkpoint_guard.py status`): HEAD `587c409`
branch `main`, worktree clean (only untracked `tasks/`/`.kilo/plans/` docs,
none in the touch-set), census unchanged from T1 hop.

## Gate evidence (reviewer's own fresh runs)

- `python tools/testrun.py --distill -p ariadne-cli` (exit 0, capture
  `target/testrun-capture.txt`):
  `96 passed / 0 failed / 1 ignored / 5 suites`
  — matches the journaled green and post-refactor lines exactly.
- `python tools/testrun.py --distill -p ariadne-ui` (exit 0) - T1 open
  item #4 census check: `184 passed / 0 failed / 0 ignored / 3 suites`
  — still 3 suites. **T1 verdict open item #4: CLOSED.**
- `python tools/hygiene_check.py` - R3 warn-only, no R1/R2; harness reports
  any non-zero exit as command failure and reported none, so exit 0 (an
  explicit `$LASTEXITCODE` echo was refused - see refused-gate log). New-code
  warnings are exactly the journaled `p` locals + `bar` in `ui.rs`, the two
  `bar` names and `p` seed locals being prescribed by the handoff itself
  (lines 92-96, 623-655 context), and one `p` in the new test file.
- `python tools/encoding_guard.py --all` - clean, exit 0 (no output).
- Frozen locks vs `e276910`:
  - `git diff e276910 -- crates/cli/src/commands/progress.rs` -> EMPTY.
    The I2 oracle is therefore frozen against the UNCHANGED renderer
    (milestone gate G2 precondition MET).
  - `git diff e276910 --stat -- crates/pipeline` -> EMPTY (I9).
  - `git diff --name-only e276910..HEAD` -> only T1-approved files
    (`crates/ui/src/dto.rs`, `crates/ui/src/server.rs`,
    `crates/ui/tests/ui_test/server_api.rs`, T1 journal, ledger touched by
    `5eff824`) + T2's 4 files. `progress_contract_tests.rs`, `crates/ui/**`,
    `themes.css`, `theme.js`, `tests/fixtures/**`, `tests/golden/**`,
    `tools/theme_token_baseline.json`: all untouched (G2 accepted).
- Commit-vs-touch-set: `git diff --name-only 000238b..HEAD` = `ui.rs`,
  `boot_progress.rs`, `boot_progress_tests.rs`, journal = exactly the 4
  claimed paths; no Cargo.lock (journal's "no regeneration" claim true);
  scopes match gate command 6 verbatim.
- Per-commit review: single commit, whole diff read; 6 hunks, every one in
  prescribed wiring regions (module decls, `RUN_PHASES`, `make_boot_progress`
  widening, `run_ui` seed, test-mod decl). Error arm, `swap_into_state`,
  watcher `quiet` path: no hunks -> unchanged (I4/I6).
- Hygiene ceilings: `ui.rs` 754 -> 790 (+36 <= 60, < 820 stop line);
  `boot_progress.rs` 115 lines (<= 450); test file 561; all functions far
  under 100 lines.

## Plan-literal checks (handoff acceptance section)

- `boot_progress.rs` shape (shape lines 142-155): `seeded`/`apply` names,
  signatures, and every rule verified against source - Done@100 keeping last
  message, Active via `super::step_percent`, finalize rule (stage==total &&
  step-percent>=100), stale-frame ignore, label = `strip_stage_prefix` at
  seed, row order = stage order.
- Wiring literal `p.steps = rows; p.total_stages = STAGES_PER_RUN;` inside
  the SAME existing single `write_state` pass, `apply` outside the lock
  (lock discipline verified in diff), `bar(report)` after, unchanged.
- All 6 required tests present and named as specified (seed DP-1, I1 pinned
  sequence + own-stage-end rule, I2 oracle full + mid-boot, I3 + direct
  stale-frame, finalize rule, shape-JSON example field-for-field).
- I2 oracle fidelity verified against the frozen renderer byte semantics
  (`progress.rs` `render`: `\n` before new-stage frame, `\r` redraw per
  frame, `\n` on finalize): `committed_lines_of`/`open_line_of` model the
  sink exactly; parsing is positional (`STEP_BAR_WIDTH` cell skip, percent
  field, two-space message separator), zero authored glyphs, MemorySink
  duplicated locally - progress.rs untouched. Committed-line parity is
  checked "at that moment" (apply precedes comparison), finalize line
  included; the open-line half is genuinely exercised via the `[..20]`
  prefix (ends "Symbols justified"@99 -> stage-5 Active @95, no finalize).
- I1/I3 are NOT weakened: stage-5 row asserted equality to the full
  per-frame percent vector `[0,5,5,5,5,5,10,25,25,25,30,95,100]`, earlier
  rows asserted `<100` strictly before AND `==100` from their own end frame
  across all 21 frames; I3 runs over full stream + 12-frame prefix +
  mid-start single frame.
- Red protocol: journaled Red `86 passed / 1 failed / 0 ignored / 1 suites`
  with the exact predicted `assertion failed: !p.steps.is_empty()`; the
  compile-error-not-assertion-Red honesty note is present as required.
  Red is not re-executable against landed code; evidence shape accepted.
- Report-evidence (distillation contract): journal is digest-shaped -
  canonical lines quoted verbatim, exit codes, zero raw dumps, distilling
  entry points only.

## Journaled deviations - independent dispositions

1. **`rows()` accessor - ACCEPTED.** `apply` always marks `report.stage`
   Active/Done (`update_current_row`), so an all-`Pending` DP-1 seed cannot
   come from `apply` without a fake report. The only in-contract alternative
   would be abusing the out-of-range-stage early-return as a side effect;
   a named pure accessor is strictly better and is tested. Additive within
   the NEW touch-set file; no behavior change.
2. **"11 reports" count - ACCEPTED, and it is the handoff that is wrong.**
   Verified: `run_pipeline_stages` contains exactly 10 literal
   `stage_report` calls (2 per stage), the last two of which ARE stage-5
   frames already contained in the pinned 13-frame sequence. The replay
   (8 stage-1..4 frames verbatim-in-order + the pinned sequence rebuilt
   through the REAL `stage_report`/`stage_progress_callback` machinery, with
   an internal `assert_eq!` pinning the 13 globals `80,81,81,81,81,81,82,
   85,85,85,86,99,100`) mirrors the true cold-run emission order, including
   that "Collateral mapped"@85 follows the pipeline's "Load artifacts
   saved"@85. Including the two stage-5 frames separately would regress
   85->80 and break I3 by construction. Related: the shape/handoff
   abbreviation "80, 81, 81x5, 82, ..." (14 entries) double-counts an
   81-frame vs the binding source sequence (13 frames, 81x5); the binding
   input #5 rule "reuse verbatim" from `progress_contract_tests.rs` governs
   and the implementation follows the source. See finding 2.
3. **Mid-boot prefix cut + rewind divergence - ACCEPTED, documented
   contract, not an oracle hole.** The stale-frame ignore is prescribed by
   shape I3 (lines 46-48: "defensive; global percent and stage are monotone
   in `build_ui_payload`"), i.e. the I2 parity domain is stage-monotone
   producer streams by contract. The renderer keys the commit on
   `open != stage`, so a rewind frame legitimately makes the two models
   diverge; demanding parity there would require the snapshot to mimic the
   renderer and would contradict I3. The `[..20]` cut is required to
   exercise the open-line assertion at all (the full run ends with no open
   line since the finalize commits it) - verified live in the model.
4. **Post-finalize frames ignored - ACCEPTED.** Exact mirror of
   `StepBarRenderer::frame`'s finalized short-circuit (progress.rs
   126-132), required by "no row leaves Done" plus committed-line message
   stability; directly tested (repeated 100% frames inert, late message
   cannot rewrite "Payload assembled").

Undeclared deviation scan: none found. `RUN_PHASES` const extraction is a
prescribed wiring-site dedupe, not a restructure - accepted.

## Findings (numbered; none blocking)

1. minor - journal line 50: "boot_progress.rs 124 lines" vs committed blob
   115 lines. Provenance drift from a pre-refactor count; both well under
   450. Fix on the journal's next touch (T2 session continues).
2. minor (doc errata) - `tasks/ui-progress-parity-shape.md` lines 33-34 and
   this handoff's lines 127-128 abbreviate the global sequence as
   "80, 81, 81x5, 82, ..." (14 values) while the binding source sequence
   has 13 frames (81 appears 5x). Implementation is correct (source
   verbatim); the errata belongs to `tasks/milestone_ledger.md`
   deferred-findings / T3 closeout - shape doc is in no implementer
   touch-set. Disposition recorded here; orchestrator to carry it.
3. observation - Red evidence cites "1 suites" vs green "5 suites" for the
   same `-p ariadne-cli` package; not re-executable, and the handoff's Red
   requirement (failing-test list + exact canonical line + predicted
   assertion) is met. Flagged for awareness only.
4. observation - journal cites no capture path explicitly; the pinned path
   `target/testrun-capture.txt` was confirmed live by the reviewer's fresh
   runs (digest header prints it).

## Refused-gate log (this hop)

Verbatim refusal text on every refusal below: `The user has specified a
rule which prevents you from using this specific tool call. Here are some
of the relevant rules {"permission":"bash","pattern":"*","action":"deny",
"source":"global"}`

- Combined lock-diff one-liner (`git diff ...; if ($?) { ... }; echo ...`)
  - refused; re-run as two individual `git diff` calls - both succeeded.
- `hygiene_check.py; echo $LASTEXITCODE` and `encoding_guard.py --all; echo
  ...` - refused; re-run individually - both succeeded.
- `cmd /c "... && echo ..."` and `... > $null; if ($?) {...}` exit-code
  probes - refused; exit 0 for both gates established from harness
  non-zero-is-failure semantics instead.
- `python -c` blob line-count probe - refused; parent `ui.rs` = 754
  established arithmetically (790 current minus the commit's 36 pure
  insertions).

No gate in the T2 scope was left unexecuted for permission reasons.

## Follow-up dispositions

- Finding 1: implementer fixes the journal count on next touch.
- Finding 2: to `tasks/milestone_ledger.md` deferred-findings; adjudicated
  at T3 closeout with the rest of the milestone addendum.
- T1 verdict open item #4 (ariadne-ui suite census): CLOSED - `184 passed /
  0 failed / 0 ignored / 3 suites`.
- Workspace-wide gates remain deferred to the T3 hop per instruction.
