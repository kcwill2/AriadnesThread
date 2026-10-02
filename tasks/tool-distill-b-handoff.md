# Handoff B — Tool Output Distillation: standing contract, skill policy rewrites, ledger, M4 fixes

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791044160000-tool-output-distillation.md` (spec; I1-I9;
   section 4 adoption list). Base rev: HEAD at authoring (post-`1b7bc3f`).
3. `tools/distilled.py` + `tools/testrun.py` + `tools/distill.py` (committed
   digest literals below are verified against these files, not transcript).
4. `.kilo/skills/review-gate/SKILL.md` (verdict findings embedded below are
   the gate outcome for Handoff A; the verdict itself lives nowhere else).
5. `tasks/milestone_ledger.md` (row format + archive rule + deferred rows).

## Handoff A gate outcome (context; already APPROVED at `1b7bc3f`)
Minor findings: F1 import cycle `distilled <-> testrun`; F2 distill-mode
header `(exit N)` can disagree with baseline-mismatch process exit; F3 mixed
module/fix commit (disclosed); F4 journal blobs carried 3 non-ASCII bytes at
M1/M2 (fixed from `98fef86`, HEAD clean). Out-of-scope: O1 raw-mode
`parse_cargo_output` matches unstripped lines (colorized cargo -> false
`parse format drift`; pre-existing); O2 three untracked strays
(`.kilo/plans/1790697359283-systems-layer-engine-room.md`,
`.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md`,
`.kilo/skills/deslop/`) need owner decision: commit or discard.
F3/F4 are closed as process lessons (recorded in the ledger row note below);
F1/F2 are fixed by M4 here; O1/O2 become deferred-findings rows.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/tool-distill-b-handoff-journal.md (checkpoint journal; always in-set)
- tools/distilled.py, tools/testrun.py, tools/testrun_tests.py,
  tools/distill_tests.py, tools/distilled_tests.py (M4 fixes + tests ONLY;
  no digest-shape changes)
- docs/specs/tool-output-distillation.md (new, M5)
- AGENTS.md, .kilo/skills/tool-runner/SKILL.md,
  .kilo/skills/review-gate/SKILL.md, .kilo/skills/handoff-spec/SKILL.md,
  .kilo/skills/resume-checkpoint/SKILL.md (M6/M7 targeted edits)
- tasks/milestone_ledger.md (M7: one milestone row + two deferred rows)
- .kilo/plans/1791044160000-tool-output-distillation.md (append-only close line)
You MUST NOT touch: `crates/**`, `tests/**`, `Cargo.*`, `target/**`, other
skills. No `git stash`, no branch change. Frozen locks: none.

## What to build (four modules, four commits)

### M4 = fix-code (Red first, two findings from the A gate)
F1 import cycle: make neither `distilled` nor `testrun` require the other at
IMPORT time (lazy import inside the function that needs it — reviewer
suggested `_emit_distilled_output` in testrun or `digest_cargo_test`'s use of
`testrun.parse_cargo_output`; choose ONE direction, prefer lazy in
`distilled.digest_cargo_test`). Red tests (in `distilled_tests.py`): fresh
subprocess `python -c "import distilled"` and `python -c "import testrun"`
(cwd `tools/`), each alone must exit 0.
F2 distill header exit honesty: with `--distill` + `--baseline` MISMATCH, the
`### cargo test ... (exit N)` header must show the exit code the process
actually returns (1), not 0. Compute baseline verdict before emitting the
digest. Red test in `testrun_tests.py` via `--replay` + tmp baseline fixture:
assert header `(exit 1)` and return code 1 together.
NEVER weaken existing assertions; the byte-identity default-mode lock (test
15) must stay green.

### M5 = standing contract doc `docs/specs/tool-output-distillation.md`
The durable policy AGENTS.md and skills cite. Structure (~100-140 lines,
ASCII): purpose + I8 rationale (implementer delegation empirically absent);
entry-point table (below, verbatim); invariant list I1-I9 condensed; digest
shapes with verbatim literals; prohibitions; evidence rules for reports and
gates; capture-file convention (`target/`, overwritten per run, path printed;
gitignored, never a deliverable); deployment modes (running distilling entry
points directly IS the supported mode; tool-runner is policy owner, never the
load-bearing protection).

ENTRY-POINT TABLE (must match committed code exactly):
| command family | invocation | digest header | exit codes | capture |
| cargo test | `python tools/testrun.py --distill [args]` |
  `### cargo test PASS|FAIL|DRIFT (exit N) [capture: <path>, <L> lines]` +
  canonical line LAST | 0 / 1 / 2(drift|usage) | `target/testrun-capture.txt`
| cargo build/check/clippy | `python tools/distill.py -- cargo <args>` |
  clean: `ok (exit 0, <W> warnings emitted) -> capture: <path>`; fail: error
  blocks `error[E...]: msg` + `-->` loc, `... and K more errors`,
  `capture: <path> (<L> lines)` | 0 / 1 / 2(usage) |
  `target/distill-capture.txt`
| unittest / tools suites / anything else |
  `python tools/distill.py --kind generic -- <argv>` |
  `### generic PASS|FAIL (exit N)` + bounded excerpts (<= 50 lines) +
  `capture: <path> (<L> lines)` | 0 / 1 / 2 | as above
| timeout | — | `TIMEOUT after Ns -> capture: <path>` | 1 | written |

PROHIBITIONS (list verbatim): bare high-volume commands in agent sessions
(raw `cargo test`/`build`/`clippy` workspace, full `unittest discover`,
`ui_smoke`, `bench_compare`); verdicts from substring matching; raw pastes in
reports (cite digest + exit + capture path); re-implementing the cargo-test
grammar outside `testrun.py`.

### M6 = skill policy rewrites (targeted edits, keep each file's voice)
- `.kilo/skills/tool-runner/SKILL.md`: DELETE the `Output:`/`Errors:`
  raw-paste fields from the output format. New format:
  `Command / Exit code / Verdict (from exit code) / Digest (distiller lines
  verbatim) / Capture path`. Add the entry-point table + the deployment-mode
  note (implementer running distilling entry points directly is fully
  supported; tool-runner owns policy + report shape when invoked). Keep all
  existing "What You Don't Do" boundaries. Timeout section: timeouts belong to
  the tools' own `--timeout`.
- `.kilo/skills/review-gate/SKILL.md`: add report-evidence check as numbered
  step 8 (renumber verdict to 9): implementer report must be digest-shaped
  (verdict lines + `Ran N` summaries + capture paths; zero raw dumps) and its
  gate commands must use distilling entry points; missing evidence or raw
  paste = numbered finding.
- `.kilo/skills/handoff-spec/SKILL.md`: Gates template item 1 ->
  `python tools/testrun.py --distill -p <crate>`; Report-back item 3 ->
  distill-mode evidence (digest header line + canonical line + capture path);
  add discipline clause: bare high-volume commands in an implementer session
  are a reviewer blocker.
- `.kilo/skills/resume-checkpoint/SKILL.md`: `--cmd` examples use the
  distilling forms (`testrun.py --distill ...`, `distill.py ...`).

### M7 = AGENTS.md rows + ledger row
- AGENTS.md Testing bullet: append agent guidance that sessions run
  `testrun.py --distill` (digest + `target/testrun-capture.txt`), raw mode for
  humans; Process Tooling bullets: add
  `python tools/distill.py -- <cmd>` wrapper row and
  `docs/specs/tool-output-distillation.md` pointer. Minimal diff, ASCII.
- `tasks/milestone_ledger.md` Milestones table, one new row:
  `| Tool output distillation (contract + distillers) | COMPLETE (A gated
  APPROVED 1b7bc3f; B @ <this commit>) | spec
  .kilo/plans/1791044160000-tool-output-distillation.md; docs/specs/tool-output-distillation.md
  | none | I8: enforcement lives in tools, not delegation prose; F1/F2 fixed
  in B; F3/F4 closed as process lessons |`
  `<this commit>` = your M7 commit hash; edit the row once more in the same
  commit is impossible, so leave the literal `B (same commit)` — the
  orchestrator finalizes the hash at mirror time.
- Deferred-findings table, two new rows: O1 (`testrun.parse_cargo_output`
  matches unstripped lines; colorized cargo -> false drift; fix = ANSI-strip
  at raw parser entry, red-gated | target: next tooling milestone | source:
  tool-distill A gate) and O2 (three untracked strays need owner decision:
  commit or discard | target: operator | source: tool-distill A gate).
- kilo-memory mirror of the ledger row is the ORCHESTRATOR's job (you cannot
  call memory tools) — do not attempt it; note "memory mirror pending" in
  your report.

## Required tests
M4 adds: (1) import distilled alone exit 0; (2) import testrun alone exit 0;
(3) distill+baseline-mismatch header `(exit 1)` and exit 1 together.
Full suites (see Gates) must stay green, including the testrun default-mode
byte-identity lock.

## RGB protocol
M4 Red-first with captured failing test names; Green minimal; Refactor only
if duplication appears. M5-M7 are documentation/governance: no tests, but
every literal (digest shapes, table rows, ledger row) is review-checked
verbatim against this file and the code. One commit per module:
`fix(tools): break distilled/testrun import cycle + distill header exit
honesty (gate F1, F2)`; `docs(specs): standing tool-output distillation
contract`; `docs(skills): tool-runner distillation policy + gate evidence
checks`; `docs(process): AGENTS distill guidance + ledger row + deferred
O1/O2`.

## Gates (exact commands, timeout 600000 ms)
1. `python tools/distilled_tests.py` then `python tools/testrun_tests.py`
   then `python tools/distill_tests.py` -- Red first (M4), then `Ran N ... OK`
   each.
2. `python -m unittest discover -s tools -p "*_tests.py"` -- `Ran N ... OK`.
3. Live: `python tools/testrun.py --distill -p ariadne-lang` -- quote digest
   header + canonical line only.
4. `python tools/hygiene_check.py` exit 0; `python tools/encoding_guard.py
   --staged` exit 0 before EACH commit (M1/M2 lesson F4: run it every time).
5. Commit per module with `python tools/commit_scoped.py --scope <glob> -m ...`
   (never `git add .`).

## Discipline clauses
- UTF-8: Write/Edit tools only; pure-ASCII content; journal too (F4).
- Touch only the touch-set; out-of-scope: report, do not fix.
- Run cargo only through distilling entry points (dogfood the contract; your
  own gate 3 command already is one).
- After EACH module Green gate, append a 1-3 line checkpoint to
  `tasks/tool-distill-b-handoff-journal.md`.
- If context exhaustion looms: STOP, return disk-state inventory; successor
  uses `python tools/checkpoint_guard.py resume --spec
  tasks/tool-distill-b-handoff.md ...`.

## Report back (final message)
Per-module commit + `git show --stat` line; M4 Red failing-test names +
Green `Ran N ... OK` lines; M5-M7 files changed + one-line confirmation each
that literals were diffed against this handoff; deviations + reasons;
out-of-scope observations; note "memory mirror pending". Never paste raw tool
output; your report is itself evidence-shaped (I8).
