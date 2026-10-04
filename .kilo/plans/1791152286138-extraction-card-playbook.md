# Plan pointer - ui-extraction-card-playbook

Milestone key: `ui-extraction-card-playbook`.

Architect artifacts (the shape the planner sizes from):

- `tasks/ui-extraction-card-playbook-kickoff.md` - intent, scope in/out,
  10 must-hold invariants, frozen ledger locks, acceptance gate,
  concurrency/serialization record.
- `tasks/ui-extraction-card-playbook-shape.md` - auto-extract inventory
  (CLI spine with file:line), propose-time vs apply-time data map,
  read-only boundary decision, invariants P1-P11 with oracles, contracts
  C1-C7 (DTO extension, feed, markup skeleton, disclosure, tokens, help),
  module map, 3-handoff split (T-A DTO -> T-B feed -> T-C assets), risks,
  hygiene notes.

Decision summary: the card becomes a collapsed summary + click-to-expand
playbook (destination paths + per-file manifests, real data from
`ProposedExtraction.target_header` via an additive DTO extension); "one-click
apply from the UI" is deferred (apply mutates repos transactionally - no UI
mutation routes exist). Card points at the real spine:
`tool apply <repo-root> --plan <id>` (`crates/cli/src/commands/transform.rs:129`).
