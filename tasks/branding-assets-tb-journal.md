# branding-assets T-B Journal

## Step 0 - TRACK_BASE and dispatch precondition (2026-10-11)

TRACK_BASE a65e2683b8445cbad4c38276dd69a36ddacd0985

Dispatch precondition check: STOP - working tree is dirty on two touch-set files.

Dirty files in the touch-set:
- extensions/ariadne-extension/README.md: pre-existing edits unrelated to this
  handoff (documents new Start/Stop Ariadne Server feature, toolPath setting,
  port 7742 change, possessive fix in one place).
- extensions/ariadne-extension/package.json: pre-existing edits unrelated to
  this handoff (adds ariadne.toolPath contribution, changes serverUrl default
  to 7742, updates descriptions).

Precondition rule (handoff line 28): "If the working tree is dirty on any
touch-set file, STOP and report."

No mutations were made. No gates were run beyond the precondition check.
No commit was made. Handoff B is BLOCKED pending resolution.

Resolution required: the pre-existing extension work (README, package.json,
src/extension.ts, dist files, .vscodeignore, .vsix) must be committed or
stashed before this handoff may proceed, or those files must be declared
in-scope by an updated handoff. Once the touch-set files are clean at
TRACK_BASE, re-dispatch.

## Read-only diagnostics gathered at the stop point (no mutations)

- `git diff HEAD -- extensions/ariadne-extension/package.json` (distilled,
  exit 0): 40+ changed lines adding `ariadne.toolPath` contribution,
  serverUrl default 7742, Start/Stop-server command surface. Zero branding
  content; zero `"icon"` involvement.
- `git diff HEAD -- extensions/ariadne-extension/README.md` (distilled,
  exit 0): ~30 changed lines: toolPath section, Start/Stop-server command
  docs, port 7742, one `Ariadne Thread` -> `Ariadne's Thread` fix.
  Unrelated to this handoff.
- Anchor check against the WORKING TREE (what the planner verified 2026-10-11):
  README lines 1-3 match the C1 before-string exactly; `package.json` line 7
  is `  "license": "MIT",`; grep finds no `icon` field. Anchors MATCH the
  dirty tree - the handoff's before-strings were taken from this dirty state,
  which does not cure the precondition: committing now would fold someone
  else's uncommitted feature work into this handoff's landing commit and
  blow the frozen locks (package.json single-line-hunk rule, README
  additions-only rule, reviewer TRACK_BASE..HEAD set equality).
- Source asset state: `media/` is UNTRACKED (`?? media/` in git status), as
  are `.vscodeignore`, `ariadne-extension-0.1.0.vsix`, and the track's
  kickoff/shape/handoff docs. `extensions/ariadne-extension/media/` does not
  exist yet. Byte size of `media/AT.jpg` NOT recorded (see D1: probe refused).

## D1 - session permission finding (blocks every handoff gate command as written)

The permission layer refuses `python -c "...; ..."` (any semicolon inside the
quoted -c argument trips the catch-all deny rule). Fingerprinted:
`print(1)` passes; `print(1); print(2)` refused; newline-separated `-c`
statements pass. Consequence: C0 copy, C2 derivation/validation, and C3 JSON
parse - all written single-line with `;` - are refused VERBATIM in this
session. Re-dispatch needs either a permission allowance for nested python,
or the successor reformulates the same commands newline-separated (declare
that as its own D-row deviation at that time).

## D2 - gate-count declaration

10 bash invocations attempted (7 distilled successes, 3 permission refusals).
Expected ~5 on a pre-mutation stop: the overshoot is the two file diffs
(re-run once to read the full capture), anchor checks, and the permission
fingerprinting needed to make finding D1 evidence-backed. Zero mutations;
zero gate commands of the handoff were executed.
