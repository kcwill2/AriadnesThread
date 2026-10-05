# V1 Handoff - M26 boot-progress-visibility: first frame honest, dead boot loud

Milestone: `boot-progress-visibility` (M26, ledger row 2026-10-05). Single-crate
handoff (cli). Sizing law held: 1 crate, 6 implementer gate invocations
(<= ~10), ~450-650 expected produced lines (<= ~1200). Workspace-wide gates
(full census, `ui_smoke`, `bench_compare`, `encoding_guard --all`) belong to
the reviewer hop, not to this session.

Why this hop exists (E3, 2026-10-05, PROVEN necessity of I11): the user's UI
served a frozen "loading" row for minutes while its own boot worker lay dead
with a truthful panic line (`tokenizer.rs:331:21 subtract with overflow`) on a
console stderr nobody was watching. The boot `spawn_blocking` has no
`catch_unwind` and drops the JoinHandle (`crates/cli/src/commands/ui.rs:786-805`),
while the watcher (`:635-676`) catches and logs. This hop closes that hole and
makes the first boot frame precede the heavy construction hashing.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
   sec. 2 (I1/I2/I3/I5/I6/I9/I11), sec. 3.1 (mapping + boot error-surface
   contract), sec. 9 (hygiene).
3. `crates/cli/src/commands/ui.rs` lines 76-189 (report type + span block),
   191-268 (`run_pipeline_stages`), 310-317 (`build_ui_payload` head),
   681-726 (`make_bar_progress`, `make_boot_progress`), 780-824 (boot task).
4. `crates/cli/src/commands/progress.rs` - FROZEN, read-only: reuse
   `format_step_bar`/`StepBarRenderer`/`TerminalStepBars` unchanged.
5. Frozen test modules you imitate but never edit:
   `crates/cli/src/commands/ui/progress_contract_tests.rs`,
   `crates/cli/src/commands/ui/boot_progress_tests.rs`.
6. `docs/specs/tool-output-distillation.md` (distilling entry points - the
   only commands you may run).

## Objective and invariants

Three behaviors, all in `crates/cli`, all observability-only:

1. **stage_spans extraction (hygiene, pre-V3).** `crates/cli/src/commands/ui.rs`
   is 839 lines; V3 must not grow it. Extract the stage-span block verbatim
   into `crates/cli/src/commands/ui/stage_spans.rs`: lines 90-189 - the five
   `*_PHASE` consts, `STAGES_PER_RUN`, `RUN_PHASES`, the A4 calibration
   comment + stage-5 milestone consts, `stage_start_percent`,
   `stage_end_percent`, `step_percent`, `mapped_span_percent`,
   `tu_index_percent`, `collateral_file_percent` - as a VERBATIM move (comment
   included; it is the committed calibration evidence). `stage_report`,
   `ProgressReport`, `ProgressFn`, `stage_progress_callback` STAY in `ui.rs`
   (they close over boot machinery). `ui.rs` adds `mod stage_spans;` plus a
   re-export `use stage_spans::{...};` covering EXACTLY the names the frozen
   test modules import via `use super::{...}` (collateral_file_percent,
   stage_end_percent, stage_start_percent, step_percent, tu_index_percent,
   ARTIFACTS_PHASE, ARTIFACTS_SAVED_PERCENT, CALL_GRAPH_PHASE,
   COLLATERAL_FILE_SPAN_END, COLLATERAL_PHASE, JUSTIFICATION_COMPLETE_PERCENT,
   JUSTIFYING_SYMBOLS_PERCENT, PAYLOAD_ASSEMBLED_PERCENT,
   PLANNING_EXTRACTIONS_PERCENT, REDUCTION_PHASE, STAGES_PER_RUN,
   SYMBOLS_ANALYZED_PERCENT, TU_INDEX_PHASE, RUN_PHASES) plus every name
   `ui.rs` production still calls. Also add in `stage_spans.rs`:
   `pub const DISCOVERY_HASHED_SPAN_END: usize = 10;` with a comment
   `// TODO(calibrate M27 T1): replace 10 with the measured stage-1 split
   point from the boot_stages release table (A4 precedent).`
   Invariants I2/I3: this is a pure move - no behavior change, no percent
   value changes, `progress.rs` untouched (I3, DP-1 frozen).

2. **Frame-1-first (I5).** The FIRST statement of `build_ui_payload` becomes
   `progress(stage_report(1, stage_start_percent(1), ARTIFACTS_PHASE,
   "Hashing sources"));` - before `Pipeline::new(repo_path, None)`. Delete the
   now-duplicate first emission from `run_pipeline_stages` (the
   `stage_report(1, stage_start_percent(1), ARTIFACTS_PHASE, "Hashing
   sources")` call at ui.rs:197-202); every other frame stays byte-identical
   in order. The visible frame SEQUENCE is unchanged; only its timing becomes
   honest (the bar exists before the constructor's serial SHA-256 hashing).
   The stage-end frame `(1, 20, "Include graph ready")` stays (I9 span).

3. **Boot error surface (I11).** Extract the boot task body into a testable
   bridge, mirroring the watcher precedent at ui.rs:635-676:
   `fn drive_boot_build(state: &Arc<UiState>,
   fps_tx: tokio::sync::oneshot::Sender<Option<StageFingerprints>>,
   build_fn: impl FnOnce() -> Result<UiBuildResult>)` wraps the call in
   `std::panic::catch_unwind(AssertUnwindSafe(build_fn))`:
   - `Ok(out)` - body as today (print load-complete, `swap_into_state`,
     `fps_tx.send(Some(fps))`).
   - `Err(e)` - as today (state `error`, message = `e.to_string()`,
     `fps_tx.send(None)`).
   - `Err(payload)` (panic) - downcast payload to `&str`/`String` text;
     write `state = "error"`, `message = panic text`, then
     `tracing::error!("UI boot task panicked: {text}")` (R8: never silent),
     then `fps_tx.send(None)`.
   `run_ui` keeps the JoinHandle (`let handle = tokio::task::spawn_blocking(...)`)
   and awaits it in a `tokio::spawn` task; a `JoinError` (double-panic /
   runtime abort) routes to the SAME error-state write (no fps_tx needed there:
   if the closure died before sending, `fps_tx` is dropped and `fps_rx`
   resolves `Err` - the watcher `if let Ok(Some(..))` already no-ops).
   The existing watcher and its catch at 635-676 stay untouched.
   Testability seam: add `pub(crate) fn make_bar_progress_with_sink(sink:
   Arc<Mutex<dyn Write + Send>>)` and `make_boot_progress_with_sink(state,
   sink)`; the existing `make_bar_progress`/`make_boot_progress` delegate with
   stdout. No `crates/ui/**`, no asset, no SSE changes (out of scope, DP
   history `ui-progress-parity`).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/boot-progress-visibility-v1-journal.md` (checkpoint journal; always
  in-set; create it alongside this handoff)
- `crates/cli/src/commands/ui.rs` (extraction to `mod stage_spans;` +
  re-exports; frame-1-first in `build_ui_payload` + duplicate removal in
  `run_pipeline_stages`; `drive_boot_build` + JoinError route;
  `*_with_sink` builders; two appended `#[cfg(test)] mod` lines)
- `crates/cli/src/commands/ui/stage_spans.rs` (NEW - verbatim extraction +
  the one new D const)
- `crates/cli/src/commands/ui/first_frame_tests.rs` (NEW - I5 locks)
- `crates/cli/src/commands/ui/boot_panic_tests.rs` (NEW - I11 locks)

You MUST NOT touch: `crates/cli/src/commands/progress.rs` (I3/DP-1:
byte-identical, any diff = auto-REJECT);
`crates/cli/src/commands/ui/progress_contract_tests.rs`,
`ui/boot_progress_tests.rs`, `ui/builder_tests.rs`, `ui/target_files_tests.rs`
(append-only floor: zero diff at this hop); `crates/pipeline/**` (V3 owns it);
`crates/repo/**`; `crates/ui/**` assets + `server.rs` + `dto.rs`;
`crates/cli/src/commands/index.rs` (V3 rider); `tests/golden/**`,
`tests/fixtures/**`; `tools/**`. No `git stash`, no branch change.

## What to build - exact literals

- Message literal: `"Hashing sources"` (kept verbatim; the V1 lock tests and
  the frozen contract tests both needle it).
- Error-state literal fields: `state = "error"`, `message` = panic text or
  error text (same shape as the existing Err arm at ui.rs:799-802).
- Panic mirror literal: `tracing::error!` (watcher uses `warn!`; the boot
  surface is stricter per R8).
- `DISCOVERY_HASHED_SPAN_END: usize = 10` (used by V3; declared here so V3
  grows `stage_spans.rs`, not `ui.rs`).
- Frozen test imports must resolve UNCHANGED: every name in
  `progress_contract_tests.rs` lines 1-8 and the `use super::...` block of
  `boot_progress_tests.rs` must still compile against `ui.rs`'s namespace via
  your re-exports. Do not edit those import lines.

## Required tests

In `ui/first_frame_tests.rs` (new mod line appended in `ui.rs`):
1. `test_first_frame_precedes_pipeline_construction` - source-text lock like
   `test_planning_milestone_precedes_justifying_in_source`: in the production
   section of `../ui.rs` (before `#[cfg(test)]`), the first occurrence of
   `"Hashing sources"` is strictly before the first occurrence of
   `"Pipeline::new"`. The needle deliberately has no `(` so it stays green
   when V3 renames the call to `Pipeline::new_with_progress`.
2. `test_stage_one_start_frame_emitted_exactly_once` - the production section
   contains `"Hashing sources"` exactly once (dedup lock).
3. `test_cold_boot_first_report_is_stage_one_and_two_states` - I5 behavior:
   build a tiny temp repo (two headers, no build db), run `build_ui_payload`
   with a recording `ProgressFn`; assert reports[0] ==
   `(stage 1, percent 0, ARTIFACTS_PHASE, "Hashing sources")`; at least two
   distinct stage-1 `(percent, message)` states; every stage-1 percent inside
   `[0, 20]` (I9); the stage-1 end frame `(1, 20)` present.
4. `test_boot_snapshot_mirrors_first_frame` - I6 parity: feed the recorded
   reports through `BootStepSnapshot::seeded(...).apply` and assert the
   stage-1 row is Active with the first frame's message (no frozen Pending row
   survives a frame).

In `ui/boot_panic_tests.rs`:
5. `test_panic_in_boot_task_surfaces_error_state` - fail-first task fn:
   construct `UiState::new_loading(tempdir)`, a oneshot, call
   `drive_boot_build` with a closure that
   `panic!("injected boot failure")`; assert terminal `progress.state ==
   "error"`, `message` contains `"injected boot failure"`, receiver got
   `Ok(None)`.
6. `test_error_result_surfaces_error_state` - same harness, closure returns
   `Err(anyhow!("injected build error"))`; assert `state == "error"`, message
   contains the error text, `Ok(None)`.
7. `test_panicked_boot_commits_the_open_bar_line` - build the boot progress
   closure via `make_boot_progress_with_sink` onto a memory sink; render one
   stage-2 frame; run `drive_boot_build` with a panicking closure using that
   progress; assert the sink holds the last live bar followed by a committed
   `"\n"` (guard-drop commits, error text starts on a fresh line).

Red note (valid Red, no todo!()): author tests first; they compile against
current `ui.rs` ONLY if you first stage the API seam - stub the seam by the
minimal honest wrongness: declare `drive_boot_build` as a thin wrapper that
calls `build_fn()` with NO catch and NO sink builders yet - then tests 5-7
fail at runtime (panic escapes / no error state), tests 1-2 fail (frame order
/count), test 3 may already pass (order-only part) - capture the failing list
verbatim. Extraction (part 1) lands in the same commit sequence BEFORE the
behavior commits so the frozen test imports stay green at every commit.

## RGB protocol (per module, in this file order)
1. Red: tests + seam stubs compile, assertions fail; capture failing list.
2. Green: extraction, frame-first, bridge; re-run until green. NEVER weaken an
   assertion; NEVER edit the frozen test modules to pass.
3. Refactor: honest names, doc comments on `stage_spans.rs` groups; re-run.

## Hygiene (review-enforced)
Functions < 100 lines (every extracted fn already is; `drive_boot_build`
target <= ~50); `ui.rs` must LAND SMALLER than 839 lines (that is the point
of the extraction); `stage_spans.rs` <= ~140 lines; no banned names; authored
content pure ASCII (AGENTS.md File-Edit Discipline).
`python tools/hygiene_check.py` exit 0.

## Frozen locks (each with its proving command)

| Lock | Proof |
|---|---|
| `progress.rs` byte-identical (I3, DP-1) | `git diff --stat <start-rev>..HEAD` lists no `progress.rs` |
| stage-5 13-frame replay UNEDITED (I2) | `progress_contract_tests.rs` absent from diff; `test_stage_five_milestone_sequence_is_monotonic_and_contained` green inside the scoped run |
| boot contract tests APPEND-ONLY (DP-1) | `boot_progress_tests.rs`, `progress_contract_tests.rs`, `builder_tests.rs`, `target_files_tests.rs` absent from diff |
| boot parity oracle green UNEDITED (I6) | `boot_progress_tests` green in the scoped `-p ariadne-cli` run |
| goldens/fixtures frozen; `crates/ui/**` untouched | diff lists touch-set files only; reviewer status census |
| mojibake gate | `python tools/encoding_guard.py --staged` exit 0 pre-commit |
| hygiene | `python tools/hygiene_check.py` exit 0 |

Record start rev (= HEAD before your first commit) in the journal at
bootstrap; all `--base` args use it.

## Gates (exact commands, timeout 600000 ms each; scoped only)
All gates are distilling entry points - this session cannot run bare
`cargo`/`git log` (ledger row 58; `docs/specs/tool-output-distillation.md`).
Workspace-wide gates belong to the reviewer hop.

1. Red: `python tools/testrun.py --distill -p ariadne-cli` - digest header +
   canonical line verbatim in the report; failing list = tests named above;
   exit 1.
2. Green: same command - `0 failed`, exit 0.
3. Refactor re-run (same command) only if refactor touched behavior.
4. `python tools/hygiene_check.py` - exit 0.
5. `python tools/encoding_guard.py --staged` - exit 0 before committing.
6. Commit (one or two commits; journal carries Red evidence):
   `python tools/distill.py -- python tools/commit_scoped.py --scope
   crates/cli/src/commands/ui.rs --scope crates/cli/src/commands/ui/stage_spans.rs
   --scope crates/cli/src/commands/ui/first_frame_tests.rs --scope
   crates/cli/src/commands/ui/boot_panic_tests.rs --scope
   tasks/boot-progress-visibility-v1-journal.md -m "feat(cli): boot frame-1-first,
   boot panic-to-error surface, stage-span extraction to ui/stage_spans.rs
   (boot-progress-visibility V1)"` (never `git add .`).

Total implementer gate invocations: 6 (<= 10).

## Refactor and hygiene targets (named ceilings)
- Ceiling at risk: `ui.rs` 839 lines (1000 cap, V3 grows it next) - land
  <= ~700. Function ceiling 100 lines: none near it after extraction; keep
  `build_ui_payload` body growth <= the one frame statement.
- I9 honesty: no new percent values at this hop (D const is declared, not
  used yet).

## Acceptance (machine-checkable; implementer hop)
1. `python tools/testrun.py --distill -p ariadne-cli` -> `0 failed` exit 0,
   all pre-existing cli tests green UNEDITED.
2. `python tools/hygiene_check.py` -> exit 0.
3. `python tools/encoding_guard.py --staged` -> exit 0.
4. `git show --stat` of landing commit(s) lists exactly the touch-set.
Reviewer hop (NOT yours): scoped `-p ariadne-cli` re-run; at milestone
closeout G1-2 locks diff-EMPTY (incl. m00/m18 parity UNEDITED), G1-3 oracles,
G1-4 `python tools/ui_smoke.py` via distill PASS, G1-5 bench NOT RUN with the
written no-op exemption (behavioral no-op: frame reorder + error surface only;
zero diff under any stage-5 producer path), full census,
`encoding_guard --all`.

## Discipline clauses
- UTF-8: never Set-Content/Out-File/`>`/Get-Content-replace; Write/Edit tools
  only; pure-ASCII content (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`, full census) in this
  session are a reviewer blocker; use the distilling entry points.
- After EACH per-module Green gate, append a 1-3 line checkpoint to
  `tasks/boot-progress-visibility-v1-journal.md` (module + commit hash,
  deviations, lock/contract findings).
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/boot-progress-visibility-v1-handoff.md --scope ... --cmd ...`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (first failing test list).
3. Green evidence (distill digest header + canonical summary line + capture
path). 4. Any deviation + reason. 5. Out-of-scope observations (report-only).
Never paste raw tool output: cite command + canonical line + exit code.

## Reviewer-gate reference
Adjudicated via the `review-gate` skill; its commit-vs-touch-set check
enforces the touch-set. Extra reviewer checks: frozen modules +
`progress.rs` diff-EMPTY against the start rev; `ui.rs` line count DOWN;
re-export list covers every frozen-test `use super::` name (compile proof);
re-exported `stage_spans` values byte-equal the pre-move block (I1/I9);
panic bridge mirrors the watcher precedent AND adds `tracing::error!` +
`fps_tx.send(None)`; JoinHandle kept.

## Sequencing note
Parallel-safe with `boot-progress-visibility-v2` (repo crate, disjoint files).
NOT parallel-safe with V3 (same files: `ui.rs`, `stage_spans.rs`): V3
dispatches only after this lands. Never parallel with any M27 hop touching
`crates/cli` (T3b; it is V3-after anyway).
