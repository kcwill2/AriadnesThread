# Tool Output Distillation - Standing Contract

Durable policy owned by the tools, cited by `AGENTS.md` and the workflow
skills. This document is law; the plan file
`.kilo/plans/1791044160000-tool-output-distillation.md` is history.
Introduced by the tool-distillation milestone (handoff A gated APPROVED at
`1b7bc3f`; fixes + this contract landed in handoff B).

## Purpose

Every tool result an agent consumes must be *bounded by construction*: a
compact, verdict-correct digest instead of raw multi-thousand-line logs,
while the full capture stays on disk for targeted reads. The single largest
recurring context tax this workflow pays is re-reading a raw workspace test
log to find two panics.

**I8 rationale (why this is law, not advice):** empirical finding
2026-10-02: implementer subagents do NOT observably delegate to a tool-runner;
raw output lands in implementer context and implementers sometimes overflow
(M18.11 lineage). Context protection therefore lives in the TOOLS, not in
skill prose or delegation routing.

## Entry points

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
| timeout | - | `TIMEOUT after Ns -> capture: <path>` | 1 | written |

## Invariants (I1-I9, condensed; full text in the spec)

- **I1** The canonical line `N passed / M failed / K ignored / S suites`
  is sacred: LAST stdout line of every testrun invocation, quoted verbatim
  in reports; default (non-distill) testrun behavior byte-for-byte unchanged.
- **I2** Verdict honesty: pass/fail derives ONLY from the child exit code
  plus pinned-shape parses, never from substring matching of log text.
- **I3** Drift loud: a `test result:` line off the pinned shape exits 2 with
  `parse format drift`; distill mode must not mask drift; zero-result output
  never prints PASS.
- **I4** Digest bounded: worst case <= ~60 lines (5 failure excerpts x 8
  lines; 10 build error blocks; 50-line generic budget; trailing
  `... and K more` when capped).
- **I5** Full capture preserved: distill mode writes complete raw output to
  a capture file and prints its path; deep-dives are targeted file reads.
- **I6** UTF-8 discipline: authored content pure ASCII, byte-safe writes,
  `tools/encoding_guard.py --all` clean, no PowerShell content pipes.
- **I7** Hygiene: functions < 100 lines, files < 1000 lines, descriptive
  names, `tools/hygiene_check.py` R1/R2 clean.
- **I8** Output-layer enforcement (see rationale above): every non-trivial
  output reaching an agent comes from a distilling entry point.
- **I9** Action-completeness: the implementer can act from the digest
  without opening the capture in the common case (failing test name +
  `panicked at file:line:col` + assertion text; error code + message +
  `-->` location; exception type + message); truncation always
  self-declares `... and K more ...`.

## Digest shapes (verbatim literals)

Cargo test (testrun `--distill`): header
`### cargo test PASS (exit 0) [capture: <path>, <L> lines]`, then counts,
then `no failures` on a clean pass; on failures each excerpt is
`---- <test_name> ----` followed by the `panicked at ...` excerpt lines and
a `... and K more failures` cap line; the canonical line prints LAST
(I1). On drift the header is `### cargo test DRIFT (exit 2)` and the line
`parse format drift: no test result line matched the pinned parse spec`.

Cargo build/clippy (`distill.py`): clean run is the one line
`ok (exit 0, <W> warnings emitted) -> capture: <path>`; failing run opens
with `### cargo build FAIL (exit N)`, then deduped `error[E...]: msg` blocks
with `-->` locations and one `=`-note line each, `... and K more errors`
when capped, last line `capture: <path> (<L> lines)`.

Generic fallback (`distill.py --kind generic`): header
`### generic PASS|FAIL (exit N)`, excerpts in priority order (unittest
FAIL/ERROR names, Traceback blocks, exception summary lines, anchored
head/tail), last line `capture: <path> (<L> lines)`. Verdict from the exit
code only; text never flips it.

## Prohibitions

- Bare high-volume commands in agent sessions (raw `cargo test`/`build`/
  `clippy` workspace, full `unittest discover`, `ui_smoke`, `bench_compare`).
- Verdicts from substring matching.
- Raw pastes in reports (cite digest + exit + capture path).
- Re-implementing the cargo-test grammar outside `testrun.py`.

## Evidence rules

- Implementer reports are digest-shaped: digest header line(s) + canonical
  line (quoted verbatim) + `Ran N ... OK` summaries + exit codes + capture
  paths; zero raw output pastes.
- Gate reports cite the distilling command, its digest, and its exit code.
- A review gate treats a missing digest/capture citation or a raw log dump
  in an implementer report as a numbered finding.

## Capture-file convention

Captures live under `target/` (gitignored), one per entry point by default
(`target/testrun-capture.txt`, `target/distill-capture.txt`), overwritten on
every run, path always printed inside the digest. A capture is scratch
evidence for deep-dives, never a deliverable, never committed, never pasted.

## Deployment modes

Running the distilling entry points directly IS the supported mode: an
implementer, tool-runner, or human may invoke `testrun.py --distill` or
`distill.py` itself. tool-runner (when used at all) is the POLICY owner and
report-shape owner - never the load-bearing protection (I8). Nested
delegation is optional architecture; the protection is the tool, not the
routing. Timeouts belong to the tools' own `--timeout`, not to an agent.
