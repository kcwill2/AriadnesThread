# T1 Journal - ui-progress-parity: ServerProgress additive step-rows DTO

Handoff: `tasks/ui-progress-parity-t1-handoff.md`. Execution slot 1 of 3.

## Checkpoints

- 2026-10-04 session start: journal created; no code touched yet.
  Deviation pre-recorded (user-instructed, not handoff): `tasks/milestone_ledger.md`
  carries the planner's uncommitted `ui-progress-parity` PLANNED row and will be
  included in the scoped commit so ledger + code land together.
- Red (gate 1): `python tools/testrun.py --distill -p ariadne-ui` ->
  `179 passed / 2 failed / 0 ignored / 2 suites`, capture
  `target/testrun-capture.txt`. Both named tests failed as missing-key
  assertions (`steps` absent -> `Value::Null`), the predicted semantic reason.
- Green (gate 2): same command -> `184 passed / 0 failed / 0 ignored / 3 suites`
  (exit 0). DTO + two constructor literals in `server.rs` only; no assertion
  weakened. Commit: this session's scoped commit (hash in final report).
- Refactor: doc comments copied verbatim from shape lines 90-98/113-114 onto
  `ProgressStepStatus`, `ProgressStepView`, and the `steps` field; re-run
  `184 passed / 0 failed / 0 ignored / 3 suites`.

## Findings

- Name deviation (pre-recorded in handoff): shape calls the loaded constructor
  `new_with_data`; real name on this tree is `new_loaded`
  (`crates/ui/src/server.rs:70`). NOT renamed.
- Row-level tests (I5 back-compat, mid-boot fixture, I6 error freeze) were
  added after the DTO landed (per the handoff's own instruction), so their
  pre-DTO state is a COMPILE error (`struct ServerProgress has no field named
  'steps'`), not an assertion-Red: the test code references `progress.steps`
  / `progress.total_stages` which did not exist pre-DTO. Recorded honestly;
  not presented as assertion-Red.
- Workspace-wide grep: `ServerProgress` struct literals exist ONLY in the
  two named `server.rs` constructors (lines ~60/~82); the cli error arm
  assigns fields, never constructs, so no out-of-scope breakage.
- Gate 1 reported "2 suites" while green runs report "3 suites" - parse
  succeeded in both runs (exit semantics as contracted); noted for reviewer.

