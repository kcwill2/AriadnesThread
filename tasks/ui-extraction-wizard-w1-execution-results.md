# W-1 Execution Results - ui-extraction-wizard: dto.rs module split

Reviewer-hop addendum for the workspace-wide gates (AGENTS: they do not run
in the implementer's session). Verdict: `tasks/ui-extraction-wizard-w1-verdict.md`
(APPROVED). Range: `5cc4ac3..82e5ffe`; all evidence below was produced fresh
in the reviewer session at HEAD `82e5ffe`, tree clean.

## Per-acceptance-criterion outcomes (with the evidence line)

### Scoped ui suite (handoff acceptance items 1-2)

- Start-rev baseline: journal record
  `283 passed / 0 failed / 0 ignored / 3 suites` exit 0. Corroboration
  without the overwritten capture: `git diff --numstat 5cc4ac3:crates/ui/tests
  HEAD:crates/ui/tests` returns EMPTY - the entire ui test tree is
  byte-identical, so the start and post-move suites are the same suite.
- Post-move (reviewer-fresh):
  `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 312 lines]`
  `283 passed / 0 failed / 0 ignored / 3 suites`
  `no failures`
  `283 passed / 0 failed / 0 ignored / 3 suites`
  Command: `python tools/testrun.py --distill -p ariadne-ui`.
  IDENTICAL to the baseline line. PASS.

### F14 reverse-dependency compile proofs (handoff acceptance items 3-4)

- `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`
  -> `ok (exit 0, 0 warnings emitted) -> capture: ...target\distill-capture.txt`.
  PASS - public-path proof: the live reverse dependency compiles against the
  unchanged `ariadne_ui::` surface.
- `python tools/distill.py -- cargo check -p ariadne-integration-tests --all-targets`
  -> `ok (exit 0, 21 warnings emitted) -> capture: ...target\distill-capture.txt`.
  PASS - all 21 warnings pre-existing in untouched files (golden-helper dead
  code, AR-21 `IncludeResolver` deprecation, `unused variable: gen` at
  `crates/ui/src/routes/api_graph.rs:719`); none names `dto.rs` or
  `dto/extraction.rs`.
- Live consumers of every moved public name (grep over `crates/`):
  `crates/cli/src/commands/ui.rs` (10 sites), `crates/cli/src/commands/ui/target_files_tests.rs`
  (3), `crates/ui/tests/ui_test/{proposals_payload,payload_etag,dto_builders}.rs`
  - all green through the glob re-export.

### Pure-move proof (handoff acceptance item 5)

- `git diff --numstat "5cc4ac3:crates/ui/src/dto.rs" "82e5ffe:crates/ui/src/dto/extraction.rs"`
  -> `12	944	crates/ui/src/{dto.rs => dto/extraction.rs}` (53 shared content
  lines byte-identical, 12 new header lines).
- `git diff --numstat "97dd836:crates/ui/src/dto.rs" "HEAD:crates/ui/src/dto.rs"` -> `7	54`.
- dto.rs = 953 lines (Read to EOF), extraction.rs = 68 lines.
- One formatting blank collapsed outside the moved block (Finding 2 of the
  verdict); no content edit inside the block. PASS.

### Hygiene R1/R2 (the point of W-1)

- `python tools/hygiene_check.py` -> exit 0
  (`### generic PASS (exit 0)` via the distiller; 179 capture lines, all R3
  warn-only).
- dto.rs at 953 lines: R2 wall discharged (was exactly 1000). extraction.rs
  at 68 lines: inside every file rule. Zero R1 findings (no function over 100
  lines) in either file; the three dto.rs R3 single-letter `s` warnings
  (lines 398, 486, 513) are pre-existing and shifted only by the move.
- `python tools/encoding_guard.py --all` -> exit 0
  (`### generic PASS (exit 0)`, capture 0 lines - silent).
- `python tools/theme_token_check.py --check-added --base 5cc4ac3` (extra
  frozen-lock check) -> `### generic PASS (exit 0)`, silent: zero added hex.

### Touch-set / frozen locks

- `git diff --name-only 5cc4ac3..HEAD` -> exactly `crates/ui/src/dto.rs`,
  `crates/ui/src/dto/extraction.rs`, `tasks/ui-extraction-wizard-w1-journal.md`.
- `git diff --name-only 97dd836 HEAD` -> no `tests/golden/**`, no
  `tests/fixtures/**`, no `tools/theme_token_baseline.json` anywhere in the
  milestone docs range.
- `git diff --numstat 5cc4ac3 HEAD -- crates/ui/tests` (tree form) -> empty.

## Workspace-wide gates (reviewer hop)

### 1. Canonical test census

- `python tools/testrun.py --baseline` (literal form, as dispatched) ->
  `usage error: missing value for --baseline`. This is the standing ledger
  errata (8 encounters before this one): `--baseline` is a FILE value flag
  (tools/testrun.py:59-61, 197-207) and no baseline census file is
  committed. The documented substitute was used.
- `python tools/testrun.py --distill` (census substitute):
  `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2756 lines]`
  `1447 passed / 0 failed / 13 ignored / 112 suites`
  `no failures`
  `1447 passed / 0 failed / 13 ignored / 112 suites`
- Reconciliation against the ledger anchor `1447 passed / 0 failed / 13 ignored
  / 112 suites` @ `59a99c4`: EXACT MATCH on all four numbers. W-1 adds zero
  tests and the refactor is behavior-preserving, so the counts carry with the
  anchors as expected; no drift, no baseline file edit required.

### 2. Tools unit suites

- Documented CI form `python -m unittest discover -s tools -p "*_tests.py"`:
  `...........................................................................................................................................................................................................................`
  `----------------------------------------------------------------------`
  `Ran 219 tests in 22.786s`
  (blank line)
  `OK`
  Matches the ledger's standing `Ran 219 tests` + `OK` (10 tool suites:
  ui_smoke, theme_token_check, testrun, hygiene_check, encoding_guard,
  distill, distilled, commit_scoped, bench_compare, checkpoint_guard).
- The dispatched pattern `-p "*_test*_tests.py"` was also run and matches zero
  files: `Ran 0 tests in 0.000s` / `NO TESTS RAN`. The documented form is the
  superset and was used as the gate.

### 3. UI runtime smoke

- `python tools/ui_smoke.py` (direct) was REFUSED by the session allowlist
  (verbatim: `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`).
  Rerouted through the distilling entry point, which kickoff section 9 names
  as the exact fallback:
  `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)`
  `boot+route checks clean (cold fixture, attempt 1, 0.9s)`
  `ui_smoke: PASS`
  `capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (2 lines)`

### 4. Load-perf bench

- `python tools/bench_compare.py` (direct) REFUSED by the session allowlist
  (same verbatim text). `python tools/distill.py -- python tools/bench_compare.py`
  (attempt 1) -> `### generic FAIL (exit 2)`
  `ERROR parse format drift: documented phase row(s) missing from bench output (never substituting zeros):`
  `capture: ... (2 lines)`; the capture carries
  `  missing documented phase: generate_report+detect_cycles (combined)`.
- Protocol retry (attempt 2, same command): IDENTICAL failure.
- Disposition (recorded protocol; not a silent pass): the failure is
  deterministic tool drift, not a timing regression and not caused by W-1.
  `tools/bench_compare.py:84` pins `generate_report+detect_cycles (combined)`
  while the bench source has printed `generate_report+detect_cycle_groups
  (combined)` since the M23.5 rename (`9194c25`, 2026-10-04). Cargo itself
  succeeded (the gate reached the missing-phase check, which happens only
  after three successful bench runs). Zero-diff proof for W-1:
  `git diff --numstat` on `crates/cli/tests/payload_tail_bench.rs`,
  `tools/bench_compare.py` and `tools/bench_baseline.json` (base `5cc4ac3` vs
  HEAD) each return EMPTY, and the W-1 range has no path under `crates/cli/**`
  or `tools/**`. Exemption recorded with that proof; a one-line tuple fix plus
  a dated ledger row is owed (verdict Finding 4).
- NOT RUN: `--bench stage5-memory` (the default stage5-timing mode is the
  documented gate and the drift lives in the timing tuple; the memory mode is
  a separate heavy local gate, 5 debug-profile runs, and is not required by
  this handoff).

## Explicitly not run / not reproducible

1. `python tools/testrun.py --baseline` (literal FILE form): ran, usage
   error by design - standing ledger errata; `--distill` census substitute
   run instead and reconciled exactly against the anchor.
2. `python tools/encoding_guard.py --staged`: not reproducible post-hoc at
   the reviewer hop (the index is empty; `commit_scoped.py` owns staging).
   The `--all` superset was run and passed; the journal records the
   pre-commit `--staged` exit 0 at commit time.
3. `git cat-file` byte-level comparison inside Python, `git grep` byte-pattern
   counts, `cmd /c` exit-code probes, `python -c`, and Write-to-temp: refused
   by the session allowlist (verbatim refusals quoted in the verdict Finding
   6). Substituted with allowlisted blob-vs-blob `git diff` proofs plus the
   Read tool on the checkout.
4. `python tools/ui_smoke.py` and `python tools/bench_compare.py` direct:
   refused by the allowlist; rerouted through the distilling entry point
   (evidence above).
5. `--bench stage5-memory`: not run (see bench section).

## Workspace-gate digests (verbatim, one block)

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 312 lines]
283 passed / 0 failed / 0 ignored / 3 suites
no failures
283 passed / 0 failed / 0 ignored / 3 suites

### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2756 lines]
1447 passed / 0 failed / 13 ignored / 112 suites
no failures
1447 passed / 0 failed / 13 ignored / 112 suites

----------------------------------------------------------------------
Ran 219 tests in 22.786s

OK

### generic PASS (exit 0)
boot+route checks clean (cold fixture, attempt 1, 0.9s)
ui_smoke: PASS
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (2 lines)

### generic FAIL (exit 2)
ERROR parse format drift: documented phase row(s) missing from bench output (never substituting zeros):
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (2 lines)
  missing documented phase: generate_report+detect_cycles (combined)
```

Supporting lock digests: hygiene `### generic PASS (exit 0)` (179 lines,
R3-only), encoding `### generic PASS (exit 0)` (0 lines),
theme_token_check `### generic PASS (exit 0)` (0 lines), F14 cli
`ok (exit 0, 0 warnings emitted)`, F14 integration `ok (exit 0, 21 warnings
emitted)`.
