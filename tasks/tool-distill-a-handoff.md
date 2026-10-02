# Handoff A — Tool Output Distillation: digest module + testrun --distill + distill.py CLI

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791044160000-tool-output-distillation.md` (full spec; invariants
   I1-I9, digest shapes, pinned grammars, CLI shapes). Base rev: `942b7b3`.
3. `tools/testrun.py` (whole file — canonical line format, pinned parse spec,
   OWN_SWITCH_FLAGS, `--replay` seam, exit-code contract 0/1/2, DRIFT_MESSAGE).
4. `tools/testrun_tests.py` (model for test seams: direct import via sys.path,
   `mock.patch.object(testrun, "run_cargo_command", ...)`, `--replay` usage).
5. `.kilo/plans/1790831557705-process-tooling-codification.md` T4 row (why the
   pinned-spec + drift discipline exists — incident E4).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/tool-distill-a-handoff-journal.md (checkpoint journal; always in-set)
- tools/distilled.py, tools/distilled_tests.py (new digest module)
- tools/testrun.py, tools/testrun_tests.py (--distill flag only)
- tools/distill.py, tools/distill_tests.py (new CLI)
- .kilo/plans/1791044160000-tool-output-distillation.md (execution-results
  addendum lines only, appended at close)
You MUST NOT touch: `crates/**`, `tests/**`, `Cargo.*`, `AGENTS.md`,
`.kilo/skills/**`, `docs/**`, `tasks/milestone_ledger.md` (handoff B's scope).
No `git stash`, no branch change. Frozen locks: none (no cargo/golden surface).

## What to build (three modules, three commits)

### M1 = spec T1: `tools/distilled.py`
Namedtuples `FailureExcerpt(name, panic_line, lines)`,
`BuildErrorBlock(code, message, location, lines)`, `Digest(verdict, lines,
capture_path)`. Pure functions:
- `digest_cargo_test(text, counts, exit_code, max_blocks=5, max_block_lines=8)`
  -> list of digest lines. Counts come FROM `testrun.parse_cargo_output`
  (import it; never re-implement the pinned grammar). If counts is None ->
  verdict `DRIFT`, lines cite the drift — never PASS.
- Pinned failure grammar: block starts `---- <name> stdout ----`; ends at next
  `---- ` header, `^failures:$`, or `^test result:`. Excerpt = line containing
  `panicked at` + up to 2 following lines; block capped at max_block_lines.
  ANSI stripped per line before matching. Cap overflow adds
  `... and K more failures`.
- Digest shape (testrun mode, exact order):
  1. `### cargo test <PASS|FAIL|DRIFT> (exit N) [capture: <path>, <L> lines]`
  2. excerpt blocks, or `no failures` on clean pass
  3. `... and K more failures` when truncated (I9)
- `digest_cargo_build(text, exit_code, max_blocks=10)` -> Digest. Block opens
  `^error(\[E\d+\])?: `, includes its `-->` location line and first `=`-note
  line; dedupe on (code, message). Warning tail `^warning: .* generated`
  yields summary count. Clean + exit 0 -> ONE line
  `ok (exit 0, <W> warnings emitted) -> capture: <path>`.
- `digest_generic(text, exit_code, max_lines=50)` -> Digest. Verdict := exit
  code ONLY (0 PASS / nonzero FAIL); text never flips it (I2/I8). Excerpt
  priority: unittest `^(FAIL|ERROR):` names (max 5); Traceback blocks (max 2
  blocks x 10 lines); exception/error summary lines
  `^[A-Za-z_.]*(Error|Exception)[: ]` / `^error:` / `^ERROR` (max 5); nothing
  matched -> first 3 + last 12 lines. Last line ALWAYS
  `capture: <path> (<L> lines)`. Total digest lines <= 50 in every shape.
- `write_capture(text, capture_path)` -> Path. Binary-mode write (encoding
  utf-8, newline-safe), mkdir parents, returns path. Default capture dir
  `target/` for callers.

### M2 = spec T2: `testrun.py --distill`
`--distill` joins OWN_SWITCH_FLAGS. Distill mode replaces `_echo(cargo_text)`
with digest + capture (default `target/testrun-capture.txt`). Canonical line
stays the LAST stdout line (I1). Drift/timeout/usage behavior and exit codes
byte-identical to current. `--replay` works in distill mode (test seam).
Default (no flag) behavior MUST be byte-identical to today (regression lock).

### M3 = spec T3: `tools/distill.py`
CLI: `python tools/distill.py [--kind build|generic] [--capture PATH]
[--timeout SECONDS] [--input-file F --child-exit N] -- <child argv...>`.
- Child runs via subprocess argv (NO shell=True), repo cwd, timeout default
  900 s; verdict = child exit code (I2).
- `--kind auto`: first child token `cargo` -> build, else generic.
- `--input-file` + `--child-exit` = offline parse (unit-test seam; no child).
- Exit: child 0 -> 0; child nonzero/timeout -> 1; usage error -> 2.
- Timeout prints `TIMEOUT after Ns -> capture: <path>` (testrun precedent).

## Required tests (exact behaviors; Red first, capture failing list)
tools/distilled_tests.py:
1. clean multi-suite pass -> digest <= 5 lines, contains `0 failed`, no
   `... ok` suite lines leak.
2. 2-failure fixture with `---- name stdout ----` + `panicked at f.rs:88:5:` ->
   each name + panicked line present; passing lines absent.
3. 9-failure fixture -> exactly 5 blocks + `... and 4 more failures`.
4. byte-constructed ANSI variant (`b"\x1b[31m"`) digests identical to plain.
5. drift-shaped `test result:` line -> verdict DRIFT, no PASS claim.
6. `digest_generic` unittest fixture -> FAIL/ERROR names excerpted, verdict
   from exit code.
7. `digest_generic` 3-Traceback fixture -> only 2 blocks excerpted.
8. `digest_generic` no-match output + child exit 3 -> FAIL + head/tail
   fallback (anti-prototype test).
9. every generic shape <= 50 digest lines; last line `capture: <path> ...`.
10. write_capture byte-exact round-trip + parents created.
tools/testrun_tests.py additions (via --replay + tmp capture):
11. distill pass replay: raw suite lines ABSENT, canonical line LAST, exit 0.
12. distill fail replay: excerpt present, capture file == full replay text,
    exit 1.
13. drift replay in distill mode: `parse format drift`, exit 2 (not masked).
14. timeout path unchanged in distill mode (exit 1).
15. default-mode regression lock: stdout byte-identical to current echo +
    canonical behavior (no --distill).
tools/distill_tests.py (offline seam):
16. error[E0308] fixture: code + message + `-->` location in digest, exit 1.
17. warning-only fixture exit 0: single `ok` line with warning count.
18. 15-error fixture -> 10 blocks + `... and 5 more`.
19. `--kind generic` delegates to digest_generic (unittest names present).
20. usage error -> exit 2; missing child exit value -> exit 2.
21. FAIL-exit, no recognizable text -> verdict FAIL (never PASS) via CLI.

## RGB protocol (per module, in this file order)
1. Red: failing tests first (NEVER stub skeletons as "red"); capture the
   failing test list (names only, not raw output).
2. Green: minimal implementation; re-run until green; NEVER weaken assertions.
3. Refactor: dedupe (excerpt caps shared via module constants), honest names,
   doc comments; re-run. One commit per module.

## Gates (exact commands, timeout 600000 ms; scoped only)
1. `python tools/distilled_tests.py` (then `testrun_tests.py`,
   `distill_tests.py`) -- Red first, then `Ran N tests ... OK` quoted.
2. After M2+M3: `python -m unittest discover -s tools -p "*_tests.py"` --
   quote `Ran N tests / OK`.
3. After M2: live `python tools/testrun.py --distill -p ariadne-lang` --
   digest <= 10 lines, canonical line last, capture path printed (quote the
   canonical line only).
4. After M3: live `python tools/distill.py cargo check` -- one-line ok digest
   + capture path, exit 0.
5. `python tools/hygiene_check.py` -- exit 0.
6. `python tools/encoding_guard.py --staged` -- exit 0 before EACH commit.
7. Commit per module with `python tools/commit_scoped.py --scope <touch-set
   glob> -m "feat(tools): ..."` (never `git add .`).

## Discipline clauses
- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools
  only; pure-ASCII content (AGENTS.md File-Edit Discipline).
- Dogfood the contract yourself: during TDD run cargo ONLY through the
  `--replay`/`--input-file` seams or scoped distill-mode smokes. Your own
  context is the first casualty of raw logs; do not `cargo test --workspace`
  raw for any reason.
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- After EACH module Green gate, append a 1-3 line checkpoint to
  `tasks/tool-distill-a-handoff-journal.md`: module + commit hash, deviations,
  contract findings.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed -- successor runs `python tools/checkpoint_guard.py
  resume --spec tasks/tool-distill-a-handoff.md --scope … --cmd …`.

## Report back (final message)
1. Commit hashes + `git show --stat` per module. 2. Red evidence (failing test
   NAMES + counts, never raw tracebacks). 3. Green evidence (quoted
   `Ran N tests ... OK` per suite + both live smoke verdict lines).
4. Any deviation + reason. 5. Out-of-scope observations (report-only).
Never paste raw tool output (cargo dumps) into the report: cite command +
digest line + exit code. The reviewer re-runs gates against the commits.
