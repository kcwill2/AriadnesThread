---
name: milestone-lifecycle
description: >-
  Run milestone kickoff and closeout against tasks/milestone_ledger.md: the
  ledger is the source of truth for milestone state (memory mirrors it). Use
  when starting a milestone (generate kickoff doc, audit graduation backlog)
  or finishing one (execution-results addendum, ledger+memory sync, walkthrough
  committed, worktree census clean).
---

**Authority rule:** `tasks/milestone_ledger.md` is the source of truth for
milestone state. kilo-memory entries MIRROR it, updated in the same commit;
on conflict, trust the ledger and fix the memory.

## Kickoff (start of a milestone)

1. Read `tasks/milestone_ledger.md` + `python tools/checkpoint_guard.py
   status` (census first -- E16: never start on top of unaccounted worktrees).
2. Generate the kickoff doc `tasks/<milestone>-kickoff.md` from the governing
   plan section + ledger: milestone objective, acceptance commands, frozen
   locks (ledger byte-locked sets), inherited deferred findings (ledger
   table -- copy the rows targeting this milestone IN, or record an explicit
   "carried, not now" with reason).
3. **Audit the graduation backlog** (D4): for every warn-only row (R3 naming,
   clippy `-D warnings`, bench CI, theme-token Phase B), either set a dated
   "flip at/before milestone M" row or record why it stays warn-only. Flips
   NEVER happen automatically and no row may be silently ignored.
4. Author task handoffs via the `handoff-spec` skill (template), referencing
   the ledger's lock list -- not restating them.
5. Commit kickoff + ledger changes together (`chore(tasks): <milestone>
   kickoff`).

## Closeout (end of a milestone)

1. Append an **execution-results section** to the timestamped plan file that
   governed the milestone (`.kilo/plans/<ts>-*.md`): what shipped, hash list,
   gate summary (canonical testrun.py line), deviations, findings split
   (fixed-now vs deferred-to-which-milestone). This replaces ad-hoc addenda
   (E9).
2. **Walkthrough checklist items** (E10/E16 loss prevention -- enforcement
   lives HERE, not in the global walkthrough-guide skill):
   - [ ] `walkthroughs/<feature>-walkthrough.html` EXISTS and is COMMITTED on
         the milestone branch (verify with `git ls-tree -r HEAD --name-only`,
         not with a directory listing). If missing: generate and commit per
         the `walkthrough-commit` skill.
   - [ ] **Worktree census clean:** every participating worktree is merged, or
         abandoned WITH a ledger row. Feed: `python tools/checkpoint_guard.py
         status` census output. A detached-HEAD worktree holding the only copy
         of verified work = STOP (that is how the theming implementation was
         lost).
3. Update the ledger milestone row (status + HEAD commit) and any deferred /
   graduation rows; **update the mirroring kilo-memory entries in the same
   commit**.
4. **Ledger archive + memory GC (context hygiene, 2026-10-01):** apply the
   ledger's Archive rule (census rows with completed disposition and closed
   deferred findings move to `tasks/ledger_archive.md`); then prune the
   kilo-memory mirror -- `forget` superseded entries and any entry the ledger
   now fully owns, so the injected startup context carries only live state
   plus the pointer "state lives in `tasks/milestone_ledger.md`". A record
   marked SUPERSEDED that still states the CURRENT contract is kept in exactly
   one category (corrections preferred), never two.
5. Commit plan addendum + ledger + walkthrough + final code together (per
   `repo.tracks_plan_files`), e.g. `git add .kilo/plans/<file> tasks/
   walkthroughs/ && git commit -m "docs(<milestone>): closeout"`. Use
   `python tools/commit_scoped.py` when a sibling session shares the worktree.
6. Verify no byte-lock drift: handoff lock list unchanged, or an explicitly
   reviewed `UPDATE_GOLDENS=1` commit (`tools/golden_guard.py` once landed).
