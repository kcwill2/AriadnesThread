# Tool Output Distillation (spec)

Status: ACCEPTED 2026-10-02 (full workflow). Scope generalized per user
direction: the digest+capture pattern applies to EVERY non-trivial tool
output, and enforcement lives at the tool-output layer, not in agent
delegation prose (I8). Successor to the discarded untracked prototype
`tools/distill_tool_output.py` (deleted 2026-10-02, never committed — lessons
below, do not resurrect its failure modes).

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
- **I8 — Output-layer enforcement.** Empirical finding (2026-10-02):
  implementer subagents do NOT observably delegate to tool-runner; raw output
  lands in implementer context and implementers sometimes overflow (M18.11
  lineage). Context protection therefore lives in the TOOLS, not in skill
  prose: every non-trivial output reaching an agent comes from a distilling
  entry point — a pinned distiller or `tools/distill.py` generic mode. Bare
  runs of known high-volume commands inside an implementer session are a
  reviewer blocker; reports must show digest-shaped evidence (verdict line +
  capture path), never raw pastes. tool-runner delegation is optional
  architecture, never the load-bearing protection.
- **I9 — Action-completeness.** A digest is valid only if the implementer can
  act from it without opening the capture in the common case: failing test
  name + `panicked at file:line:col` + assertion text (tests); error code +
  message + `-->` location (builds); exception type + message (generic).
  Truncation must self-declare (`... and K more`), never silent.

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
def digest_generic(text, exit_code, max_lines=50) -> Digest  # fallback
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

### `digest_generic` — deterministic fallback for commands without a pinned parser

Verdict := child exit code only (I2/I8); text NEVER flips the verdict. Excerpt
selection in strict priority order, hard cap 50 digest lines total:

1. Python unittest `^(FAIL|ERROR): <name>` lines (max 5).
2. Traceback blocks: `^Traceback (most recent call last):` through the final
   non-indented line (max 2 blocks, max 10 lines each).
3. Exception/error summary lines matching `^[A-Za-z_.]*(Error|Exception)[: ]`
   or `^error:` or `^ERROR` (max 5).
4. Nothing matched: first 3 + last 12 lines (anchored head/tail — never a
   silent empty digest).

Last digest line is ALWAYS `capture: <path> (<N> lines)`.

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
python tools/distill.py cargo build --release
python tools/distill.py -- cargo clippy -- -D warnings
python tools/distill.py --kind generic python -m unittest discover -s tools
python tools/distill.py --input-file capture.txt --child-exit 3   # offline (test seam)
```

- `--kind build|generic` (default `auto`: first child argv token `cargo` ->
  build, otherwise generic). `--capture PATH` override; default
  `target/distill-capture.txt`. `--child-exit N` supplies the verdict when
  parsing offline with `--input-file` (default 0) — the test seam.

- Runs the child with `subprocess` argv (no `shell=True`), inherits repo cwd,
  timeout passthrough (default 900 s). Verdict = child exit code (I2).
- Cargo build/clippy pinned shape: `^error(\[E\d+\])?:` opens a block, `-->`
  location line, one `=`-note line; `^warning: .* generated` tail line yields
  the warning summary; dedupe identical `code+message` blocks.
- Clean run digest is ONE line: `ok (exit 0, <N> warnings emitted) -> <capture>`.
- Exit code propagated: child 0 -> 0; child nonzero -> 1; usage error -> 2.
- Timeout -> `TIMEOUT after Ns -> <capture>` + exit 1 (testrun precedent).

## 4. Adoption (a tool nothing calls does not exist)

- **Standing contract doc**: `docs/specs/tool-output-distillation.md` — the
  durable policy AGENTS.md and skills cite (digest shapes per family, capture
  paths, evidence rules). The plan file is history; the contract doc is law.
- `AGENTS.md` Testing section: agents run tests via
  `python tools/testrun.py --distill [scoped args]`; raw mode remains for humans.
- `.kilo/skills/tool-runner/SKILL.md`: rewritten as the distillation POLICY
  owner, not a paste-through proxy. When invoked it returns exactly
  `command + exit code + digest + capture path`; for commands lacking a pinned
  distiller it runs them under `tools/distill.py`. Deployment note (I8):
  nested subagent delegation is optional — the implementer running distilling
  entry points directly is a fully supported mode; the protection is the
  tool, not the routing. The `<tool output>` paste field is deleted.
- `.kilo/skills/implementer` guidance via handoff discipline: bare runs of
  known high-volume commands (`cargo test/build/clippy` without a distiller,
  full `unittest discover`, `ui_smoke`, `bench_compare`) in an implementer
  session are a reviewer blocker; gate evidence in reports must be digest +
  capture path.
- `.kilo/skills/review-gate/SKILL.md`: add report-evidence check — missing
  digest/capture evidence or raw log pastes in the implementer report =
  numbered finding.
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
- Green: implement `digest_cargo_test` + `digest_generic` + `write_capture`
  + `Digest`/namedtuple types. `digest_generic` fixtures: unittest FAIL/ERROR
  names; 3 Tracebacks (only 2 excerpted); exception-summary line; empty-match
  output (head/tail fallback, verdict from exit code — the anti-prototype
  test); 50-line budget never exceeded in any shape.
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
- Red: `tools/distill_tests.py` using `--input-file`/`--child-exit` fixtures:
  two-error compile output (error[E0308] + location `-->`), 15-warning clean
  build, clippy failure, generic-kind fallback delegation, timeout path
  (injected fake), usage error exit 2. Verdicts assert I2 (exit-code-driven,
  never substring) — a FAIL-exit fixture with no recognizable text must still
  yield FAIL.
- Green: argv runner (no shell=True), pinned build-shape parser, markdown/
  one-line digests, capture write.
- Refactor: share excerpt cap constants with T1 module.

### T4 — Adoption & governance (after T2, T3 review; separate handoff B)
- Edits: new `docs/specs/tool-output-distillation.md` (standing contract),
  AGENTS.md, tool-runner SKILL.md (policy rewrite + paste-field deletion),
  review-gate SKILL.md (report-evidence check), handoff-spec SKILL.md +
  resume-checkpoint SKILL.md (distill-mode evidence lines); ledger row +
  memory mirror; scoped commit via `tools/commit_scoped.py`.

**Touch-set (handoff A, code):** `tools/distilled.py`,
`tools/distilled_tests.py`, `tools/testrun.py`, `tools/testrun_tests.py`,
`tools/distill.py`, `tools/distill_tests.py`, the handoff + journal files,
this plan file (execution-results addendum only).
**Touch-set (handoff B, governance):** `docs/specs/tool-output-distillation.md`,
`AGENTS.md`, `.kilo/skills/tool-runner/SKILL.md`,
`.kilo/skills/review-gate/SKILL.md`, `.kilo/skills/handoff-spec/SKILL.md`,
`.kilo/skills/resume-checkpoint/SKILL.md`, `tasks/milestone_ledger.md`.

## 6. Acceptance gates

- `python -m unittest discover -s tools -p "*_tests.py"` — all green (quote
  canonical count in handoff).
- New digest suites: `digest_cargo_test` (pass/2-fail/cap-9/ANSI/drift),
  `digest_generic` (unittest names/tracebacks capped/fallback head-tail/
  budget <= 50/exit-code verdict with unrecognizable text), build parser
  (2-error/warning-only/15-error cap), testrun `--distill` (5 replay-mode
  tests incl. default-mode byte-identity regression lock), distill.py CLI
  (offline verdict propagation, usage exit 2).
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

## 8. Execution results (handoff A, code tasks T1-T3, 2026-10-02)

- T1 `tools/distilled.py` + tests: commit 4227407 (10 tests).
- T2 `testrun.py --distill` (+`--capture` override) + 5 replay-mode tests:
  commit 2b0e8ef (suite `Ran 25 OK`).
- T3 `tools/distill.py` CLI + tests: commit 776cba4 (8 tests).
- Gates: tools discovery `Ran 176 tests OK`; live `testrun.py --distill
  -p ariadne-lang` -> 3-line digest, canonical last, exit 0; live
  `distill.py cargo check` -> one-line ok digest + capture path, exit 0;
  `hygiene_check.py` exit 0; `encoding_guard.py --staged` exit 0 per commit.
- Declared deviations (detail in `tasks/tool-distill-a-handoff-journal.md`):
  optional trailing `capture_path=None` kwarg on the three digest functions
  (contract shapes require the path inside emitted lines); testrun
  `--capture PATH` override (needed by the tmp-capture test design).
- Follow-up commits: 98fef86 (addendum + journal), 74da996 (style rename).
- T4 (adoption/governance) remains open: handoff B scope.
