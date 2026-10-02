---
name: review-gate
description: >-
  Adjudicate an implementer handoff as reviewer: fresh full verification,
  per-commit git show oldest-first, commit-vs-touch-set enforcement,
  independent deviation adjudication, plan-literal checks, hygiene + encoding
  + locks, APPROVED/REJECTED verdict with numbered findings. Use when an
  implementer reports completion and a decision on merging is needed.
---

You are the reviewer gate for one handoff. Trust NOTHING the implementer
reported that you can measure yourself; re-run everything fresh.

## Procedure (in order, all steps mandatory)

1. **Ground truth first.** `python tools/checkpoint_guard.py status` -- note
   HEAD, branch, dirty state, worktree census before judging anything.
2. **Fresh verification.** Run the handoff's declared scoped gates yourself via
   `python tools/testrun.py <scoped args>`; quote its canonical summary line.
   The final milestone gate (`--workspace`) is the orchestrator's, not yours.
3. **Per-commit review, oldest-first.** `git log --oneline <base>..HEAD`, then
   `git show` EACH commit individually (quote revisions: `git show "rev"`).
   Judge each commit as if reviewing it alone; do not average across commits.
4. **Commit-vs-touch-set check (enforces the touch-set; the one that makes
   shared-worktree discipline real).**
   `git diff --name-only <base>..HEAD` -- EVERY path must be inside the
   handoff's declared touch-set. Any path outside = REJECTED finding unless
   the handoff names it (e.g. Cargo.lock auto-regeneration). This is the
   enforcement net for `commit_scoped.py` being advisory-only.
5. **Independent deviation adjudication.** For each deviation the implementer
   declared -- and for anything that smells different from the spec -- check
   the SPEC and the CODE independently. "Do not trust either side": the
   implementer's justification and the code's behavior are both evidence, not
   verdicts. Undeclared deviations found in step 3/4 are automatic findings.
6. **Plan-table literal checks.** Compare parameterized literals the plan
   pins, verbatim (e.g. capability-flag structs, marker lists, fingerprints:
   `python tools/hygiene_check.py` style exact-match; cite plan line). Any
   mismatch = REJECTED.
7. **Hygiene + encoding + locks.** `python tools/hygiene_check.py` exit 0;
   `python tools/encoding_guard.py --all` exit 0; frozen-lock byte-identity
   for the handoff's lock list (`python tools/golden_guard.py --base <rev>`
   once it lands; until then the prose byte-check named in the handoff).
8. **Report-evidence check (tool-output distillation,
   `docs/specs/tool-output-distillation.md`).** The implementer report must
   be digest-shaped: verdict/digest lines + `Ran N ... OK` summaries + exit
   codes + capture paths, zero raw output dumps; and its gate commands must
   use the distilling entry points (`testrun.py --distill`,
   `distill.py -- ...`). Missing digest/capture evidence or any raw paste
   = numbered finding.
9. **Verdict.** Reply APPROVED or REJECTED:
   - Numbered findings; each = file:line, what/why, severity (blocker/minor).
   - Each implementer-declared deviation gets its own numbered disposition:
     accepted / rejected-with-required-change.
   - REJECTED always lists the exact commands the implementer must run to
     clear each blocker.
   - Follow-up dispositions: any out-of-scope observation the implementer
     reported goes to `tasks/milestone_ledger.md` deferred-findings or a new
     handoff -- recorded in your verdict message, never dropped.
