# T2 Handoff - ui-progress-parity: BootStepSnapshot producer + terminal-parity oracle

Milestone key `ui-progress-parity`. Execution slot 2 of 3. Predecessor: T1
(`tasks/ui-progress-parity-t1-handoff.md`) must be merged - this handoff
consumes `ariadne_ui::dto::ProgressStepView` / `ProgressStepStatus`.
This handoff ships BEFORE the browser (T3) on purpose: the I2 replay oracle
freezes snapshot-vs-terminal equivalence while the only live consumer is the
terminal itself. Reviewer adjudicates via the `review-gate` skill.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-progress-parity-shape.md` lines 140-168 (exact
   `BootStepSnapshot` contract + wiring spec), lines 25-49 (invariants
   I1/I2/I3), lines 122-138 (the mid-boot JSON rows your output must be
   able to produce).
3. `crates/cli/src/commands/ui.rs`: lines 63-93 (`ProgressReport`,
   `ProgressFn`, the five `*_PHASE` consts, `STAGES_PER_RUN`), lines 141-157
   (`stage_start_percent`/`stage_end_percent`/`step_percent`), lines 179-256
   (`run_pipeline_stages` start/end report stream), lines 623-655
   (`make_bar_progress`, `make_boot_progress` - the closure you widen),
   lines 688 and 720-724 (seed point in `run_ui`; the error arm you do NOT
   change).
4. `crates/cli/src/commands/progress.rs` lines 44-84 (`STEP_BAR_WIDTH`,
   `strip_stage_prefix`, `format_step_bar` - byte-format frozen; your
   oracle parses its output by field position), lines 86-147
   (`StepBarRenderer::frame` finalize/commit semantics you copy the
   finalize rule from), lines 149-230 (`TerminalStepBars::with_sink` +
   `render`), lines 334-355 (`MemorySink` test pattern to reuse).
5. `crates/cli/src/commands/ui/progress_contract_tests.rs`
   `test_stage_five_milestone_sequence_is_monotonic_and_contained` - the
   calibrated stage-5 percent sequence your I1 replay must reuse verbatim.
6. `crates/cli/src/commands/ui/node_maps.rs` - the submodule pattern to
   mirror (`mod node_maps;` in ui.rs; `boot_progress.rs` sits beside it).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/ui-progress-parity-t2-handoff-journal.md (checkpoint journal; create
  alongside)
- crates/cli/src/commands/ui/boot_progress.rs (NEW: `BootStepSnapshot`)
- crates/cli/src/commands/ui/boot_progress_tests.rs (NEW: I1/I2/I3 tests +
  the parity oracle)
- crates/cli/src/commands/ui.rs (module declarations + wiring ONLY, see
  What to build; target growth <= ~60 lines)
- Cargo.lock (only if cargo regenerates it; no new deps)

You MUST NOT touch: `crates/cli/src/commands/progress.rs` (the model being
imitated - byte-identical), `crates/pipeline/**`, `crates/ui/**` (T1's
merged DTO is consumed, not edited), `crates/cli/src/commands/ui/
progress_contract_tests.rs`, `crates/cli/src/commands/ui/tests.rs`,
`crates/cli/src/commands/ui/builder_tests.rs`, `crates/ui/assets/**`,
`themes.css`, `theme.js`, `tests/fixtures/**`, `tests/golden/**`.
No `git stash`, no branch change.

## What to build

`crates/cli/src/commands/ui/boot_progress.rs` - exact shape from shape
lines 142-155:

- `pub struct BootStepSnapshot` holding `total_stages` and one slot per
  stage.
- `pub fn seeded(total_stages: usize, labels: &[&'static str]) -> Self` -
  all rows `Pending`, `percent 0`, `message ""`, labels stored with
  `strip_stage_prefix` already applied (call sites pass the `*_PHASE`
  consts). DP-1 decision (user, 2026-10-04): every step row exists from
  boot; not-yet-started rows are `Pending` (the browser dims them; that is
  T3's job - you only emit the statuses).
- `pub fn apply(&mut self, report: &ProgressReport) -> Vec<ProgressStepView>`
  - pure in-memory update returning the full row list:
  - rows with stage < `report.stage` become `Done`, `percent 100`, keeping
    the message of that stage's last observed frame (a stage whose final
    frame was already 100 keeps that message - e.g. "Include graph ready");
  - row == `report.stage` becomes `Active` with `percent =
    step_percent(report.stage, report.percent)` (the existing private fn,
    via `super::step_percent`) and `message = report.message`;
  - the finalize rule copied from `StepBarRenderer::frame` (progress.rs
    line 134): stage == total_stages && step-percent >= 100 => that row is
    `Done` at 100, not Active;
  - frames with stage < the active stage are ignored (defensive, I3);
  - `label` per row = the seeded stripped label; row order = stage order.
- The returned `Vec<ProgressStepView>` is written into
  `state.progress.steps` alongside `total_stages = STAGES_PER_RUN`.

Wiring in `crates/cli/src/commands/ui.rs` (touched, not restructured):
- declare `mod boot_progress;` next to `mod node_maps;` (line ~12) and
  `#[cfg(test)] mod boot_progress_tests;` next to the other test-module
  declarations (lines ~747-754).
- `run_ui` (after `UiState::new_loading`, line ~688, before
  `make_boot_progress` is used): write the seeded snapshot into
  `state.progress` once - rows 1..5 `Pending` with labels
  `strip_stage_prefix(ARTIFACTS_PHASE .. COLLATERAL_PHASE)`,
  `total_stages = STAGES_PER_RUN`. (DP-1: rows visible from boot.)
- `make_boot_progress` (lines 643-655): own a
  `Mutex<BootStepSnapshot>` beside the captured `bar` closure; per report,
  compute rows via `apply` OUTSIDE the write lock (pure), then in the SAME
  existing single `write_state(&state.progress)` pass add
  `p.steps = rows; p.total_stages = STAGES_PER_RUN;` to the existing four
  global writes - the globals stay written exactly as today (I4). Then call
  `bar(report)` exactly as today.
- The `run_ui` error arm (line 720: writes only `state`/`message`) and
  `swap_into_state`: UNCHANGED - rows freeze by construction (I6).
- The watcher-refresh `quiet` plain-line path (line 595): UNTOUCHED.

## RGB protocol
1. Red: create `boot_progress_tests.rs` containing
   `test_boot_progress_snapshot_writes_active_row` using ONLY pre-existing
   API: build `UiState::new_loading`, call `make_boot_progress(state)`,
   feed `stage_report(2, 26, TU_INDEX_PHASE, "6/24 TUs")`-equivalent
   (construct `ProgressReport` directly), assert the progress read back has
   a non-empty `steps` whose stage-2 row is `Active` at step percent
   `step_percent(2, 26)`. This COMPILES after T1 and MUST fail today with
   `assertion failed: !p.steps.is_empty()` (steps is the empty vec -
   nothing wrote rows yet). Capture the failing test list.
2. Green: implement `boot_progress.rs` + wiring; re-run until green. Add
   the required tests below in the same phase (pre-implementation they are
   compile-errors on `BootStepSnapshot`, not assertion-Red - journal that
   honestly). NEVER weaken an assertion to pass.
3. Refactor: doc comments, dedupe replay helpers into one fixture fn;
   re-run.

## Required tests (all in boot_progress_tests.rs unless noted)
- `test_seeded_snapshot_is_all_pending` - DP-1: `seeded(5, &labels)` -> 5
  rows, all `Pending`, percent 0, message empty, labels equal the phases
  minus `[k/5] ` prefixes.
- I1 per-step scale: replay the stage-1..4 start/end frame stream (the
  exact 11 reports in `run_pipeline_stages` order) plus the pinned stage-5
  calibrated sequence from
  `progress_contract_tests.rs::test_stage_five_milestone_sequence_is_monotonic_and_contained`
  (globals 80, 81, 81x5, 82, 85, 85, 85, 86, 99, 100) and assert the
  stage-5 row's percents are exactly 0, 5, 5x5, 10, 25, 25, 25, 30, 95,
  100 (shape lines 31-34), and every earlier row reaches 100 only at its
  own stage-end frame.
- I2 TERMINAL-PARITY ORACLE (the milestone's central bet - most important
  test): replay the SAME streams into BOTH
  (a) `TerminalStepBars::with_sink(MemorySink)` (pattern reused from
  progress.rs `step_bar_tests` - duplicate the small `MemorySink` locally,
  do not touch progress.rs) and
  (b) `BootStepSnapshot::seeded(...).apply(...)` per report. Then parse
  each terminal line committed by the renderer's newline logic by FIELD
  POSITION from `format_step_bar` output
  (`[{stage}/{total}] {label} [{20 cells}] {pp:>3}%  {message}`) and
  assert equality with the snapshot's `done` row `(stage, label,
  step_percent, message)` at that moment; the final open line equals the
  `active` row. Parsing is positional extraction (split on the bar
  brackets / percent field), never substring luck.
- I3 row monotonicity: across each replay every row's percent is
  non-decreasing and no row leaves `Done`; plus one direct test that a
  stale frame (stage < active) is ignored.
- finalize rule: stage-5 frame at global 100 yields row 5 `Done` at 100
  (mirrors `test_final_step_finalizes_once_without_duplicate_commits`).
- The T1-shape JSON row example (shape lines 125-137) must be reproducible
  from a seeded snapshot + the stage-1 end and stage-2 mid frames: assert
  the five resulting rows field-for-field equal that example.

## Hygiene (review-enforced)
Functions < 100 lines; `ui.rs` 754 + <=60 wiring stays < 1000 (if wiring
pushes it past 820, STOP and report - extraction pressure belongs to the
ledger, not to a surprise refactor here); `boot_progress.rs` <= ~450 lines
impl+docs (tests live in their own file); meaningful names, no banned
names, no single-letter locals. `python tools/hygiene_check.py` exit 0.

## Frozen locks
- `crates/cli/src/commands/progress.rs` byte-identical (the oracle imitates
  it; proof: `git diff <milestone-base> -- crates/cli/src/commands/progress.rs`
  empty). G2 of the milestone acceptance: the oracle passes against this
  UNCHANGED renderer.
- `crates/pipeline/**` byte-identical (I9); `stage_progress_callback` stays
  the single mapping point.
- `crates/cli/src/commands/ui/progress_contract_tests.rs` unchanged (its
  pinned sequence is your input, not your output).
- `crates/ui/**`, `themes.css`, `theme.js`, `tests/fixtures/**`,
  `tools/theme_token_baseline.json` untouched.
- `swap_into_state` and the `run_ui` error arm unchanged (I6 rows freeze
  by construction; I4 ready globals identical).

## Gates (exact commands, timeout 600000 ms each; scoped only)
1. `python tools/testrun.py --distill -p ariadne-cli` -- Red run: the named
   wiring test fails as specified; capture failing list + canonical line.
2. `python tools/testrun.py --distill -p ariadne-cli` -- Green run:
   canonical line `0 failed` (oracle included).
3. `python tools/testrun.py --distill -p ariadne-cli` -- post-refactor
   re-run, `0 failed`.
4. `python tools/hygiene_check.py` -- exit 0.
5. `python tools/encoding_guard.py --staged` -- exit 0 before committing.
6. Commit with
   `python tools/commit_scoped.py --scope "crates/cli/src/commands/ui/boot_progress.rs" --scope "crates/cli/src/commands/ui/boot_progress_tests.rs" --scope "crates/cli/src/commands/ui.rs" --scope "tasks/ui-progress-parity-t2-handoff-journal.md" -m "feat(cli): BootStepSnapshot producer + terminal-parity oracle (ui-progress-parity T2)"`
   (never `git add .`).

Authored content pure ASCII: the block glyphs live ONLY in progress.rs
string literals (already there, untouched); your Rust adds no glyphs.

## Discipline clauses
- UTF-8: Write/Edit tools only; pure-ASCII authored content (AGENTS.md
  File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`, runs beyond
  `-p ariadne-cli`) are a reviewer blocker; use distilling entry points
  (`docs/specs/tool-output-distillation.md`).
- After EACH per-module Green gate append a 1-3 line checkpoint to
  `tasks/ui-progress-parity-t2-handoff-journal.md` (module + commit hash,
  deviations, lock/contract findings).
- If context exhaustion looms: STOP, return a disk-state inventory; never
  resume a dead session - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ui-progress-parity-t2-handoff.md --scope crates/cli --cmd "python tools/testrun.py --distill -p ariadne-cli"`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence. 3. Green evidence
(distill digest header + canonical line + capture path). 4. Deviations +
reasons. 5. Out-of-scope observations (report-only). Never paste raw tool
output; cite command + canonical line + exit code.

Reviewer-gate reference: adjudicate via `review-gate`; this handoff feeds
milestone gates G1/G2 (G2 = oracle passes vs unchanged renderer - verify the
progress.rs diff is EMPTY as part of the acceptance pass).
