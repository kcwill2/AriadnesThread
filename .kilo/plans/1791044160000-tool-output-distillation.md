# Tool Output Distillation (spec)

Status: PROPOSED (awaiting mode + go decision). Successor to the discarded
untracked prototype `tools/distill_tool_output.py` (deleted 2026-10-02, never
committed — lessons below, do not resurrect its failure modes).

## 1. Intent Statement

Make every tool result an agent consumes *bounded by construction*: a compact,
verdict-correct digest instead of raw multi-thousand-line logs, while the full
capture remains on disk for targeted reads. Today the canonical testrun line
distills the verdict, but `testrun.py` still echoes the full cargo log
(`_echo(cargo_text)`, line 257), and the tool-runner skill pastes raw build/
clippy output by contract. Both defeat context economy.

**Why:** context engineering rules (ed0f95a) and the ledger archive discipline
made session-context growth a first-class cost. Re-reading a raw workspace
test log to find 2 panics is the single largest recurring context tax the
workflow pays.

## 2. Invariants (must hold before and after)

- **I1 — Canonical line sacred.** `N passed / M failed / K ignored / S suites`
  remains the LAST stdout line of every testrun invocation and is quoted
  verbatim in reports. Default (non-distill) testrun behavior is byte-for-byte
  unchanged (CI, `--baseline`, `--replay` unaffected).
- **I2 — Verdict honesty.** Pass/fail verdict derives ONLY from the child exit
  code plus pinned-shape parses. Never from substring matching of log text
  (`"FAIL" in line` heuristics are banned — they were prototype defects).
- **I3 — Drift loud.** A `test result:` line deviating from the pinned shape
  exits 2 with `parse format drift`. Distill mode must not mask or swallow
  drift; no zero-result output may print PASS.
- **I4 — Digest bounded.** Worst-case digest <= ~60 lines: at most 5 failure
  excerpts x at most 8 excerpt lines for tests; at most 10 error blocks for
  build/clippy; a trailing `... and K more` line when capped.
- **I5 — Full capture preserved.** Distill mode writes the complete raw output
  to a capture file and prints its path in the digest; deep-dives are targeted
  file reads, never context re-runs. Default capture path: `target/`
  (gitignored).
- **I6 — UTF-8 discipline.** All authored content pure ASCII; writes byte-safe;
  `python tools/encoding_guard.py --all` clean. No PowerShell content-pipe
  rewrites.
- **I7 — Hygiene.** Functions < 100 lines, files < 1000 lines, descriptive
  names, no ambiguous abbrevs; `tools/hygiene_check.py` R1/R2 clean.

## 3. Architecture & Contract

One new shared digest module + one flag on the existing canonical command + one
new entry point for non-test commands. The grammar of cargo test output stays
owned by `testrun.py` (single source of truth — `distilled.py` imports it,
never re-implements it).

### `tools/distilled.py` (new module, pure functions + one writer)

```python
FailureExcerpt = namedtuple("FailureExcerpt", "name panic_line lines")
BuildErrorBlock = namedtuple("BuildErrorBlock", "code message location lines")
Digest = namedtuple("Digest", "verdict lines capture_path")

def digest_cargo_test(text, counts, exit_code,
                      max_blocks=5, max_block_lines=8) -> list[str]
def digest_cargo_build(text, exit_code, max_blocks=10) -> Digest
def write_capture(text: str, capture_path) -> Path   # binary-mode write
```

Pinned cargo-test failure grammar (fixtures must cover all):
- Failure detail blocks start `---- <test_name> stdout ----` and end at the
  next `---- ` header, `^failures:$`, or `^test result:` line.
- Preferred excerpt line: the one containing `panicked at` (carries
  `file:line:col`); plus up to 2 lines following it; plus the `failures:`
  summary tail count `K` for the `... and K more` cap line.
- ANSI stripping reuses the existing pattern idea but is applied per line
  before matching.

Clean-pass digest is <= 4 lines: counts line, `no failures`, capture path,
canonical line (testrun mode only).

### `tools/testrun.py` (extend, behavior-preserving)

- `--distill` joins `OWN_SWITCH_FLAGS`.
- In distill mode only, `_echo(cargo_text)` is replaced by
  `digest_cargo_test(...)` + `write_capture(...)`; capture path printed inside
  the digest.
- Exit-code contract unchanged: 0 clean / 1 failed-or-timeout / 2 drift-or-usage.
- `--replay` works in distill mode (this is the unit-test seam; no cargo needed
  in tests).

### `tools/distill.py` (new CLI, build/clippy/generic commands)

```
python tools/distill.py --format markdown cargo clippy -- -D warnings
python tools/distill.py cargo build --release
python tools/distill.py --input-file capture.txt     # offline parse (test seam)
```

- Runs the child with `subprocess` argv (no `shell=True`), inherits repo cwd,
  timeout passthrough (default 900 s). Verdict = child exit code (I2).
- Cargo build/clippy pinned shape: `^error(\[E\d+\])?:` opens a block, `-->`
  location line, one `=`-note line; `^warning: .* generated` tail line yields
  the warning summary; dedupe identical `code+message` blocks.
- Clean run digest is ONE line: `ok (exit 0, <N> warnings emitted) -> <capture>`.
- Exit code propagated: child 0 -> 0; child nonzero -> 1; usage error -> 2.
- Timeout -> `TIMEOUT after Ns -> <capture>` + exit 1 (testrun precedent).

## 4. Adoption (a tool nothing calls does not exist)

- `AGENTS.md` Testing section: agents run tests via
  `python tools/testrun.py --distill [scoped args]`; raw mode remains for humans.
- `.kilo/skills/tool-runner/SKILL.md`: output contract becomes
  command + exit code + digest + capture path; pasting raw full output is
  replaced by "quote the digest; read the capture file only when the digest
  names a location you must see in full."
- `.kilo/skills/handoff-spec/SKILL.md` + `resume-checkpoint/SKILL.md`: green
  evidence line = distill-mode run; the canonical line is still what gets
  quoted verbatim (I1 keeps their existing contract valid).
- Ledger row + kilo-memory mirror in the same commit (authority rule).

## 5. Tasks (TDD; T2 and T3 parallel after T1)

### T1 — `tools/distilled.py` cargo-test digest (Red first)
- Red: `tools/distilled_tests.py` fixtures (byte-level `b"..."` for any ANSI)
  covering: clean multi-suite pass; 3-failure output with `---- name stdout
  ----` blocks + panicked-at lines; 9-failure output proving cap + `... and 4
  more`; ANSI-colored variant; drift-shaped `test result:` line must NOT be
  digested to PASS.
- Green: implement `digest_cargo_test` + `write_capture` + `Digest` types.
- Refactor: keep every function < 40 lines; parse loop extracts helpers.

### T2 — `testrun.py --distill` (depends T1)
- Red: extend `tools/testrun_tests.py` via `--replay` seam: distill mode omits
  full echo (assert pass-suite lines absent), includes failure excerpts,
  capture file created with full text, canonical line is last line, exit codes
  identical to raw mode for pass/fail/timeout/drift cases; default mode output
  byte-identical to current behavior (regression lock).
- Green: flag plumbing, `_echo` branch.
- Refactor: no duplication of excerpt logic — module owns it.

### T3 — `tools/distill.py` build/clippy CLI (depends T1 scaffold only)
- Red: `tools/distill_tests.py` using `--input-file` fixtures: two-error
  compile output (error[E0308] + location `-->`), 15-warning clean build,
  clippy failure, timeout path (injected fake), usage error exit 2. Verdicts
  assert I2 (exit-code-driven, never substring).
- Green: argv runner (no shell=True), pinned build-shape parser, markdown/
  one-line digests, capture write.
- Refactor: share excerpt cap constants with T1 module.

### T4 — Adoption & governance (after T2, T3 review)
- Edits: AGENTS.md, tool-runner SKILL.md, handoff-spec SKILL.md,
  resume-checkpoint SKILL.md; ledger row + memory mirror; scoped commit via
  `tools/commit_scoped.py` (touch-set below).

**Touch-set:** `tools/distilled.py`, `tools/distilled_tests.py`,
`tools/testrun.py`, `tools/testrun_tests.py`, `tools/distill.py`,
`tools/distill_tests.py`, `AGENTS.md`, the three `.kilo/skills/*/SKILL.md`
files, `tasks/milestone_ledger.md`.

## 6. Acceptance gates

- `python -m unittest discover -s tools -p "*_tests.py"` — all green (quote
  canonical count in handoff).
- Live smoke: `python tools/testrun.py --distill -p ariadne-lang` — digest <=
  10 lines on a passing scoped run, canonical line last, capture path printed.
- Live smoke: `python tools/distill.py cargo check` on workspace — one-line
  digest + capture path, exit 0.
- `python tools/hygiene_check.py` R1/R2 clean; `python tools/encoding_guard.py
  --all` clean.
- Code-reviewer verdict APPROVED with commit-vs-touch-set check.

## 7. Rejected alternatives

- **Standalone parallel test runner with its own test parser** — duplicates the
  pinned cargo grammar -> drift risk against E4 canonical owner.
- **Substring/keyword verdicts** (prototype design) — false PASS/FAIL; banned by
  I2.
- **Flipping testrun default to distill** — breaks human expectation and any
  existing raw-output consumers; opt-in flag + agent-guidance update is lower
  risk.
