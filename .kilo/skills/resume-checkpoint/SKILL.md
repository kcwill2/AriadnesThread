---
name: resume-checkpoint
description: >-
  Deterministic recovery from a dead or uncompacting implementer session
  (incidents E1/E16). Use when starting a fresh session to continue work from
  a handoff file, when a previous session's compaction summary was empty, or
  before any mutation in a resumed session. Bootstraps from on-disk state via
  tools/checkpoint_guard.py, never from remembered intent.
---

THE RULE: Sessions whose compaction summary was empty are NEVER resumed.
Recover only from disk state and the handoff file. This guard report is the
fresh-session bootstrap.

Follow this order exactly:

1. Run `python tools/checkpoint_guard.py status` FIRST, before any mutation.
   Read every section: HEAD, dirty files (numstat), untracked files, recent
   commits, and the WORKTREE CENSUS. Treat any worktree flagged DETACHED or
   ORPHAN as suspect state that needs triage before you trust it.

2. Bootstrap from the handoff:
   `python tools/checkpoint_guard.py resume --spec tasks/<handoff>.md`
   The handoff markdown parse is best-effort only. If the guard reports the
   parse is ambiguous or the handoff is old, pass explicit args and do not
   let the guard guess:
   `resume --spec tasks/<handoff>.md --scope "<crate/dir>/**" --cmd "<command line>"`
   The guard is report-only: it runs the scoped commands and classifies edits
   in-scope vs stray; it never fixes anything.

3. Run the guard suite before continuing mutations:
   - `python tools/testrun.py <scoped gates from the handoff>`
   - `python tools/encoding_guard.py --staged`
   - `python tools/hygiene_check.py`
   Do not proceed while any of these is red.

4. Treat the emitted `tasks/<handoff-basename>-resume.md` as the bootstrap
   checklist: verify its Disk state matches the `status` report, act on the
   Command verdicts (fix reds first), confirm the In-scope edits are yours,
   triage every Stray edit (out of scope) before touching unrelated files,
   then work the Remaining handoff steps.

Exit codes everywhere: 0 clean, 1 violation or red verdict, 2 internal error.

Never resume intent from memory or a broken compaction summary; never
reverse-engineer what the dead session "probably meant". If state and handoff
disagree, print both, stop, and ask.
