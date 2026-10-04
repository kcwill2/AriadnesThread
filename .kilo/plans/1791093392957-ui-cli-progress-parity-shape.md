# Plan - UI/CLI Boot Progress Parity (shape complete, ready for planning)

Role boundary: this is the architect's shape, not a task list. The two
authoritative artifacts are:

- `tasks/ui-progress-parity-kickoff.md` - intent, scope in/out, invariants,
  frozen ledger locks, milestone acceptance gates.
- `tasks/ui-progress-parity-shape.md` - checkable invariants I1-I9, exact
  Rust/JSON/JS contracts, module map, risks with oracles, hygiene ceilings,
  split seams for the planner.

## Chosen contract (core design decision)

Widen the progress snapshot the CLI already writes to `UiState.progress`, so
the browser receives the same per-step frames the terminal commits.
`ServerProgress` grows additively (`steps: Vec<ProgressStepView>`,
`total_stages`), where each row carries `{stage, label, percent (own step's
0-100), message, status: pending|active|done}`. A new `BootStepSnapshot` in
`crates/cli/src/commands/ui/boot_progress.rs` mirrors the report stream that
already drives `TerminalStepBars` (seeded with all `[k/N]` labels at boot,
rows commit to done exactly when the renderer commits a terminal line).
`/api/status`, the 150 ms poll, and the readiness state machine are
unchanged; the frontend gains one `renderProgressSnapshot` that draws
per-step 20-cell bars with `floor(20 * percent / 100)` - the CLI's own
arithmetic - with a legacy single-bar fallback when `steps` is empty. The
parity oracle is an equivalence test replaying report streams against the
unchanged `StepBarRenderer`/`format_step_bar` and the snapshot.

## Open for the user

- DP-1 pending rows dim vs hidden (recommend dim).
- DP-2 cell-glyph bar vs CSS width fill (recommend cell bar).
- DP-3 done rows accent-only, no verdict green (recommend as shaped).

## Next step

Planner sizes handoffs from the split seams (ui backend, ui frontend, cli
producer; suggested order 1 -> 3 -> 2), each within the Planning Law.
