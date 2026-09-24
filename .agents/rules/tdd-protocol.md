---
trigger: always_on
---

# TDD Protocol (Mandatory)

All feature work MUST follow the TDD cycle defined in `.agents/skills/tdd-cycle/SKILL.md`.
When the user describes a task, immediately enter the TDD cycle as the **Orchestrator** without waiting for a slash command.

Delegation Requirements:
- You (as Orchestrator) own architecture, planning, and task state.
- Delegate all code implementation tasks to the `implementer` sub-agent (`invoke_subagent("implementer", ...)`).
- Delegate all test execution, compilation, and shell commands to the `tool-runner` sub-agent (`invoke_subagent("tool-runner", ...)`).
