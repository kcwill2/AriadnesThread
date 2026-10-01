---
name: walkthrough-commit
description: >-
  Land a generated walkthrough so it cannot be lost: write it INTO the repo at
  walkthroughs/, and commit it in the same commit as its finalization via
  commit_scoped. Use whenever the walkthrough-guide skill (global) produces a
  deck -- a deck that exists only in a worktree or only on disk has been lost
  twice in this repo (E10/E16).
---

The global `walkthrough-guide` skill generates the deck but lives outside this
repo; landing is THIS skill's job.

1. The deck must be written to the repo path
   `walkthroughs/<feature>-walkthrough.html` -- use the Write tool with the
   repo-absolute path (never a worktree temp path, never console capture).
2. Commit in the SAME commit that finalizes it, scoped:
   `python tools/commit_scoped.py --scope "walkthroughs/<feature>-walkthrough.html" -m "docs(walkthrough): <feature>"`
   (add the milestone's other closeout files as extra `--scope` args when this
   is the closeout commit; per `repo.tracks_plan_files` plan+ledger+walkthrough
   go together).
3. Verify landed: `git ls-tree -r HEAD --name-only | grep walkthrough` (or
   `git show --stat HEAD`) -- a directory listing is NOT proof.
4. If the worktree is about to be removed/abandoned, the deck must already be
   in git on a surviving branch; milestone-lifecycle closeout re-checks this
   and the worktree census.
