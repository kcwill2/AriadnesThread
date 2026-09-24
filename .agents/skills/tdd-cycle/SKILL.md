---
name: tdd-cycle
description: Run one full TDD cycle for a given feature or task
---

When executing a TDD cycle for a feature or task:

1. **Manage Task** (Orchestrator):
   - Create entries for the task and its subtasks in `todo.md`.
   - Decompose feature into smallest testable units.

2. **Delegate Unit TDD Cycle** (Orchestrator $\rightarrow$ `implementer`):
   - Invoke `implementer` sub-agent (`invoke_subagent("implementer", ...)`).
   - `implementer` writes the failing test (RED).
   - `implementer` invokes `tool-runner` to build and verify expected RED failure.
   - `implementer` writes minimal implementation code to pass (GREEN).
   - `implementer` invokes `tool-runner` to build and verify GREEN passage.

3. **Advance & Report** (Orchestrator):
   - Review `implementer` report.
   - Update `todo.md`.
   - Advance to next unit or report task completion to the user.
