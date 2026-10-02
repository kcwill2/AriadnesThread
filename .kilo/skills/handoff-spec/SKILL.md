---
name: handoff-spec
description: >-
  Author a milestone-task handoff document (M19 T-A/T-B shape) so implementer
  sessions need zero verbal briefing. Use when creating an implementer or
  reviewer handoff: objective + invariants, hard touch-set, frozen locks,
  machine-checkable acceptance, discipline clauses, reviewer-gate reference.
---

Fill this template into `tasks/<milestone>-<part>-handoff.md` on the milestone
branch. Nothing load-bearing may live only in chat or agent memory. The M19
handoffs (`tasks/m19-ta-handoff.md`, `tasks/m19-tb-handoff.md` on
`multi-language-support`) are the worked example.

```markdown
# <Task> Handoff — <milestone>: <one-line title>

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. <exact contract files + line ranges the implementer must read>
3. <model files to mirror in structure/doc style>
4. <exact field names / literals sources>

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/<handoff-basename>-journal.md (checkpoint journal; always in-set)
- <glob or file> (<why>)
...
You MUST NOT touch: <explicit list>. No `git stash`, no branch change.

## What to build
<behavior spec; EXACT literals called out (enum variants, flag structs,
marker lists) with plan-line citations>

## RGB protocol (per module, in this file order)
1. Red: compile-failing-assertion state first (NEVER todo!()/unimplemented
   skeletons as "red"); capture the failing test list.
2. Green: implement; re-run until green. NEVER weaken an assertion to pass.
3. Refactor: dedupe, honest names, doc comments; re-run.

## Required tests
<explicit per-behavior test list, exact expected values>

## Hygiene (review-enforced)
Functions < 100 lines, files < 1000 lines, no banned identifier names
(tmp/res/foo/bar/test1) and no single-letter locals, intention-revealing
identifiers, one responsibility per function.
`python tools/hygiene_check.py` must stay exit 0.

## Frozen locks
<list, or "none this task">; verify with
`python tools/golden_guard.py --base <rev>` once it lands (until then:
manual byte-identical check of <paths> -- do NOT touch these paths).

## Gates (exact commands, timeout 600000 ms each; scoped only until final gate)
1. `python tools/testrun.py --distill -p <crate>` -- canonical summary line
   and digest header must appear verbatim in the report; red first, then
   green.
2. <next scoped command>
3. `python tools/hygiene_check.py` -- exit 0.
4. `python tools/encoding_guard.py --staged` -- exit 0 before committing.
5. Commit with `python tools/commit_scoped.py --scope <touch-set glob> …
   -m "<type>(<scope>): …"` (never `git add .`).

## Discipline clauses
- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools
  only; pure-ASCII content (see AGENTS.md File-Edit Discipline).
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`) in an implementer
  session are a reviewer blocker; run them through the distilling entry
  points (`docs/specs/tool-output-distillation.md`).
- After EACH per-module Green gate, append a 1-3 line checkpoint to
  `tasks/<handoff-basename>-journal.md` (created alongside this handoff):
  module + commit hash, any deviation, any lock/contract finding. The journal
  is the dead session's decision log; without it, recovery restores code (git)
  but loses reasoning (context hygiene, 2026-10-01).
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed -- successor runs `python tools/checkpoint_guard.py
  resume --spec tasks/<this file> --scope … --cmd …`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (first failing tests).
3. Green evidence (distill-mode digest header line + canonical line +
   capture path). 4. Any deviation +
   reason. 5. Out-of-scope observations (report-only).
Never paste raw tool output (cargo/git dumps) into the report: cite the
command + the canonical summary line + exit code (2026-10-01). The reviewer
re-runs gates against the commits; pasted output is context rot.
```

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces this touch-set, so the touch-set
must list EVERY file the task may legitimately change (including Cargo.lock
when cargo regenerates it).
