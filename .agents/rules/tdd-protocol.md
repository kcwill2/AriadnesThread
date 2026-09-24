---
trigger: always_on
---

# TDD Protocol (Mandatory)

All feature work MUST follow the TDD cycle defined in `.agents/skills/tdd-cycle/SKILL.md`.
When the user describes a task, immediately enter the TDD cycle as the **Orchestrator** without waiting for a slash command.

Delegation Requirements:
- You (as Orchestrator) own architecture, planning, and task state.
- If the `invoke_subagent` tool is available in your active toolset:
  - Delegate all code implementation tasks to the `implementer` sub-agent (`invoke_subagent("implementer", ...)`).
  - Delegate all test execution, compilation, and shell commands to the `tool-runner` sub-agent (`invoke_subagent("tool-runner", ...)`).
- If `invoke_subagent` is NOT available in your active toolset:
  - Perform the Red-Green-Refactor TDD cycle directly in sequence (Write test -> Verify RED failure -> Write code -> Verify GREEN passage -> Commit).
