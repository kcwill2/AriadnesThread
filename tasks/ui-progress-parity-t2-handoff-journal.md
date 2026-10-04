# T2 Journal - ui-progress-parity: BootStepSnapshot producer + terminal-parity oracle

Handoff: `tasks/ui-progress-parity-t2-handoff.md`. Milestone base (pre-T1):
`e276910`. T1 landed at `5eff824` (+ journal doc `000238b`).

## Checkpoints

- 2026-10-04 session start: baseline `000238b` HEAD verified via
  `python tools/distill.py -- git log --oneline -3`. T1 DTO fields
  (`steps`, `total_stages`) present on `ServerProgress` in
  `crates/ui/src/dto.rs`, re-exported via `pub use dto::*`.
- Red: created `crates/cli/src/commands/ui/boot_progress_tests.rs` with only
  `test_boot_progress_snapshot_writes_active_row` (pre-existing API:
  `UiState::new_loading` + `make_boot_progress` + direct `ProgressReport`),
  plus `#[cfg(test)] mod boot_progress_tests;` in ui.rs.
  First gate attempt hit a stale `target/debug/tool.exe` (PID 42952) holding
  an OS file lock - terminated via distill taskkill (environment fix, no
  repo change). Re-run:
  `python tools/testrun.py --distill -p ariadne-cli` ->
  `86 passed / 1 failed / 0 ignored / 1 suites`, failing test
  `commands::ui::boot_progress_tests::test_boot_progress_snapshot_writes_active_row`
  with exactly `assertion failed: !p.steps.is_empty()` (post-T1 empty-steps
  wire, as the handoff predicted).
- Green: implemented `crates/cli/src/commands/ui/boot_progress.rs`
  (`BootStepSnapshot::seeded/apply` + a read accessor `rows()` - deviation
  note below) and the ui.rs wiring (module decl, `RUN_PHASES` const,
  `run_ui` seed write, `make_boot_progress` widened with
  `Mutex<BootStepSnapshot>`; apply OUTSIDE the write lock; rows join the
  existing single `write_state` pass; globals/error arm/`swap_into_state`
  untouched). Added all required tests: seed DP-1, I1 (pinned stage-5
  sequence + own-stage-end rule), I2 oracle (full run + open-line prefix,
  local MemorySink duplicated from progress.rs tests, positional parsing,
  zero authored glyphs), I3 (+ direct stale-frame test), finalize rule,
  shape-JSON example field-for-field. Journal honesty per the handoff: the
  BootStepSnapshot tests were compile-errors pre-implementation, not
  assertion-Red; only the wiring test rode a true Red.
  Gate: `96 passed / 0 failed / 1 ignored / 5 suites`.
- Refactor: extracted `RUN_PHASES` (dedupes the five phase labels across
  the two wiring sites + test fixtures); stage-5 fixture now reuses the
  calibrated percent constants (`ARTIFACTS_SAVED_PERCENT` etc.) and
  `stage_start_percent(5)`; replay fixtures deduped into
  `stage_one_to_four_frames` / `stage_five_calibrated_frames` /
  `boot_replay_stream` / `replay`. Post-refactor gate:
  `96 passed / 0 failed / 1 ignored / 5 suites` (no warnings in capture).
- Gates: `python tools/distill.py -- python tools/hygiene_check.py` exit 0
  (R1/R2 clean; R3 warn-only - the flagged `bar`/`p` in ui.rs are
  pre-existing names inside `make_boot_progress`/`swap_into_state` style
  blocks, and the handoff's own wiring spec prescribes `p.steps`/
  `p.total_stages`). ui.rs 754 -> 790 lines (+36 <= 60, < 820 stop line).
  boot_progress.rs 115 lines (<= 450).

## Deviations / decisions

1. Added `BootStepSnapshot::rows()` (read accessor). The handoff's DP-1
   `run_ui` seed must write all-`Pending` rows once, but the contract API
   only returned rows from `apply(report)`; faking a first report would
   mark row 1 `Active`. `rows()` is the minimal pure accessor; no
   behavior change.
2. The handoff's "exact 11 reports in run_pipeline_stages order" count
   does not match the source (10 literal `stage_report` calls, the last
   two of which ARE stage-5 frames that duplicate the pinned sequence's
   entries). The I1/I2 replay therefore uses the 8 stage 1..4 start/end
   frames + the pinned 13-frame stage-5 sequence (which already contains
   "Load analysis"@80 and "Collateral mapped"@85), preserving the real
   emission order and keeping percent monotone. Replaying run_pipeline_
   stages' stage-5 frames before the pinned sequence would regress 85->80
   and break I3 by construction.
3. I2 mid-boot prefix cut at frame 20 (one before finalize) so the
   open-line half of the parity contract is exercised. A rewind-style
   prefix (stage-5 frames then a stage-2 frame) is NOT a valid oracle
   input: the renderer commits the stage-5 open line while the snapshot
   ignores the stale frame - the two sides legitimately diverge there,
   which is exactly the defensive-I3 contract, not a parity failure.
4. Post-finalize same-stage frames are ignored by the snapshot
   (mirrors `StepBarRenderer::frame`'s finalized short-circuit, so the
   committed message can never be rewritten). Not spelled out in the
   handoff; required by "no row leaves Done" + commit-line stability.

## Lock/contract findings

- `git diff e276910 -- crates/cli/src/commands/progress.rs` verified EMPTY
  (via distill; G2 precondition: oracle frozen against the unchanged
  renderer).
- No edits outside the touch-set; no Cargo.lock regeneration (no new deps).
- Ledger untouched: the handoff's acceptance names no status change for
  the T2 row.
