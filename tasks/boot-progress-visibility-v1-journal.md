# V1 Journal - M26 boot-progress-visibility (implementer log)

Handoff: `tasks/boot-progress-visibility-v1-handoff.md`.
Start rev (HEAD before first commit): `7aeee44ab1a3006e9e752fde4a4ed7aa5f05d19d`.
All `--base` / diff reconciliation args use this rev.

## Bootstrap findings (targeted reads)

- `TerminalStepBars::with_sink` already exists in the FROZEN `progress.rs`
  (`crates/cli/src/commands/progress.rs:168`): the `*_with_sink` builders
  delegate without touching the frozen renderer. No `progress.rs` diff needed.
- Frozen-test import surface verified against
  `progress_contract_tests.rs:1-8` and `boot_progress_tests.rs:1-8`:
  `stage_report`, `ProgressReport`, `ProgressFn`, `stage_progress_callback`,
  `read_state` stay in `ui.rs`; the percent/phase/span names move to
  `stage_spans.rs` and are re-exported into `ui.rs`'s namespace so the
  frozen `use super::{...}` lines resolve unchanged.
- `pub mod ui` (commands/mod.rs:10); `stage_spans` is a private child module,
  so moved items get `pub` on the move (module-boundary visibility, values
  byte-identical). Child test modules see parent-private `use` bindings,
  which is what the frozen imports rely on today.
- `UiState::new_loading(PathBuf)` and `ServerProgress` fields confirmed in
  `crates/ui/src/server.rs:54`, `crates/ui/src/dto.rs:27` (read-only).

## Deviations register

- (closed) `ui.rs` landing size: landed 832 < 839 (hard gate MET); the
  "~700" target unreachable inside the touch-set - see [landing] row.

## Checkpoints

(append 1-3 lines per per-module Green gate below)

- [extraction] commit `79f3403`: `ui.rs` 839->751 lines; `stage_spans.rs`
  116 lines; moved block byte-equal to pre-move lines 90-189 (scripted
  non-blank-line compare, PASS 92 lines; forced `pub` prefixes stripped,
  new D-const block excised). Scoped run:
  `101 passed / 0 failed / 1 ignored / 5 suites` (exit 0), frozen test
  modules UNEDITED and green. `encoding_guard --staged` exit 0 pre-commit
  (paths staged via `git add -- <touch-set>` through the distill wrapper).
- [red] seam stage: thin no-catch `drive_boot_build` stub + `*_with_sink`
  builders (delegating via the frozen `TerminalStepBars::with_sink`); new
  test mods `first_frame_tests`, `boot_panic_tests`. Red run
  `python tools/testrun.py --distill -p ariadne-cli` -> exit 1,
  `105 passed / 3 failed / 0 ignored / 1 suites` (count truncated by
  fail-fast; canonical MUST line comes from the green run). Failing list
  verbatim: `commands::ui::boot_panic_tests::test_panic_in_boot_task_surfaces_error_state`
  (panic escapes stub), `...::test_error_result_surfaces_error_state`
  (state left "loading"), `...::test_panicked_boot_commits_the_open_bar_line`
  (panic escapes stub). Tests 1-4 PASSED at Red (deviation from the
  handoff's Red note: in the pre-change source layout the single legacy
  "Hashing sources" emission already precedes `Pipeline::new` textually and
  is unique, so the source locks pass; they guard the plausible wrong
  Green states - emission after construction, or undeduplicated. The first
  Red capture, before a `cfg(test)` import was relocated below production
  code, demonstrated their failure mode: 6 failed incl. both).
- [red] hygiene: D const got `#[allow(dead_code)]` (declared for V3,
  unused at this hop); `COLLATERAL_FILE_SPAN_END` re-export is
  `#[cfg(test)]`-gated AND must stay below all production code because the
  frozen source-text locks cut the production section at the FIRST
  `#[cfg(test)]` token (first placement at the top broke
  `test_planning_milestone_precedes_justifying_in_source`; relocated).
- [green] frame-1-first + bridge landed: `build_ui_payload` now emits the
  sole "Hashing sources" frame before construction (emission deleted from
  `run_pipeline_stages`, every later frame byte-identical in order);
  `drive_boot_build` = catch_unwind bridge (Ok/Err/panic arms per handoff:
  error-state write, `tracing::error!` for panics - stricter than the
  watcher's warn, `fps_tx.send(None)`); `run_ui` keeps the JoinHandle and
  routes JoinError to the same error-state write + error log;
  `make_bar_progress_with_sink`/`make_boot_progress_with_sink` delegate
  over the FROZEN `TerminalStepBars::with_sink` (no progress.rs diff).
  Panic-payload fix: `panic_payload_text(&*payload)` - a bare `&payload`
  on the Box unsizes to a `dyn Any` view OF the box, so both downcasts
  miss (proved by standalone + `--test` probes 2026-10-05); caught by
  test 5 during Green, not an assertion weakening.
- [green] canonical MUST line (green tree, no fail-fast truncation):
  `108 passed / 0 failed / 1 ignored / 5 suites` exit 0; frozen suites
  green UNEDITED incl. `test_stage_five_milestone_sequence_is_monotonic_and_contained`
  and all `boot_progress_tests` I2 parity oracles. Refactor re-run after
  the R1 fix: identical line, exit 0.
- [green] hygiene round: first run flagged R1 `build_ui_payload` 104>100
  (the 6-line frame emission); compacted the emission to one
  rustfmt-stable line -> function back under 100, suite re-run green;
  final `hygiene_check.py` exit 0 (remaining lines are R3 warn-only in
  verbatim-moved/frozen bodies, matching pre-existing repo style).
- [landing] `ui.rs` 839 -> 832 lines (smaller: hard requirement MET);
  `stage_spans.rs` 117 lines (<= ~140). DEVIATION: the "~700" target is
  not reachable inside the touch-set - the seam (2 sink builders, bridge,
  JoinError task, 2 test mod decls) adds ~100 lines and the handoff
  forbids both new module files (outside touch-set) and moving boot
  machinery into `stage_spans.rs` (wrong module + its 140 cap). Arithmetic
  at commit-1 state 751 + seam/bridge/tests ~= +81 net of trims.
