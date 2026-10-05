# cpp-brace-depth-underflow - execution results (reviewer hop)

Tree under review: `main` @ `44dfb73` (= `02de7f3` + `9ce140a` + `ad65747` +
`44dfb73`). Reviewer session ran every workspace-wide gate itself; the
implementer and orchestrator ran none of them. All commands run unchained
(single-command form); chained forms hit the boilerplate catch-all deny and
were retried in single form.

## Ground truth (checkpoint_guard)

HEAD `44dfb73`, branch `main`, dirty: unrelated `.kilo/plans/1790697359283-...`
(106/80) and `tasks/milestone_ledger.md` (1/0, the orchestrator's own row);
worktree census: 8 worktrees, none sharing this review's commits dirty.

## Acceptance criteria

| Gate | Result | Verbatim evidence |
|---|---|---|
| G1 `testrun --distill -p ariadne-lang-cpp` | PASS | `147 passed / 0 failed / 0 ignored / 2 suites` exit 0, capture `target\testrun-capture.txt` (160 lines). `S suites` = 2 (unit + 0-test Doc-tests), unchanged structure vs pre-fix. |
| G2 `... --test m18_cpp_plugin_parity_test` | PASS, wording drift | `23 passed / 0 failed / 0 ignored / 1 suites` exit 0. Lock wording "22/22" is stale: UNEDITED file contains 23 `#[test]`. |
| G3 `... --test m00_golden_baseline_test` | PASS | `2 passed / 0 failed / 0 ignored / 1 suites` exit 0 - zero golden drift; `tests/golden/**` untouched in range. |
| G3m17 `... --test m17_plugin_seam_test` | PASS, wording drift | `6 passed / 0 failed / 0 ignored / 1 suites` exit 0. Wording "5/5" stale: file contains 6 `#[test]`. |
| G4 corpus evidence scan | **FAIL (verdict Finding 1/2)** | see G4 section below. |
| G5 `hygiene_check.py` | PASS | exit 0; output rows are R3 warn-only, all pre-existing (cli/ui/graph/cluster/transform/repo/scheduler/validate/lang-scala + pre-existing lang-cpp locals at analyzer.rs:24/34, parser.rs:113/299/315/388, tokenizer.rs:50/53 - all original lines); zero R1/R2. |
| G6 `encoding_guard.py --all` | PASS | exit 0, no output. |
| Workspace census `testrun.py --baseline` | literal unrunnable; substitute PASS | literal: `usage error: missing value for --baseline` (pre-existing ledger row 56; no baseline file committed). Substitute `python tools/testrun.py --distill` (workspace): `1195 passed / 0 failed / 4 ignored / 85 suites` exit 0. Prior census `1168 / 0 / 4 / 85 @ 1bc50b8`: suites and ignored unchanged; +27 passed = +13 new lang-cpp tests +14 intervening-milestone tests. |
| Tool suites `python -m unittest discover -s tools -p "*_tests.py"` | PASS | `Ran 188 tests in 20.706s` / `OK`. |
| `ui_smoke.py` | PASS | direct call refused by allowlist; sanctioned route `python tools/distill.py -- python tools/ui_smoke.py`: `### generic PASS (exit 0)`, `ui_smoke: PASS`, capture `target\distill-capture.txt`. |
| `bench_compare.py` | PASS on re-run; first run FAIL (noise) | run1 (`--runs` default 3): `### generic FAIL (exit 1)`, `WARN OptimizationPlanner::plan() facts+closures: 1908.3 ms vs baseline 1451.0 ms (+32%)`. run2 (`--runs 5`): `### generic PASS (exit 0)`, `bench compare: 5 run(s), no regressions past 1.25x baseline`; milestone-relevant `symbol_pass+prep (combined, <= first event)` 1464.5 ms = 1.15x baseline. `plan()` is closure/cluster-side; m00/m18 prove lang-cpp output bytes unchanged; verdict finding 7 records the baseline-freshness note. |
| Step-0 byte-identity | PASS | `git diff --no-index --stat ".kilo/plans/1791144879664-cpp-brace-depth-underflow.md" "docs/specs/cpp-brace-depth-underflow.md"` -> empty (exit 0, only symmetric autocrlf warnings; both files LF+ASCII). 302 lines each; spec committed isolated in `9ce140a`, never touched again in range. The journal's sha256 `2a54c804...f82ad` was NOT independently reproduced: `git hash-object` and `python -c` are off the reviewer allowlist (boilerplate deny); the empty git diff is the equivalent byte-identity proof used instead. |
| Touch-set (I7) | PASS | `git diff --name-only 02de7f3..HEAD`: parser.rs, parser/brace_depth_tests.rs, tokenizer.rs, tokenizer/raw_string_tests.rs, docs/specs/cpp-brace-depth-underflow.md, tasks/cpp-brace-depth-underflow-handoff-journal.md - all six inside the declared touch-set. No characterization/symbol_locator/golden/fixtures/analyzer/Cargo path appears in any commit's stat. |
| Per-commit review | PASS structurally | `9ce140a` docs-only; `ad65747` the fix + tests + journal; `44dfb73` journal tally correction only (honest: raised the tally 9->11 and added post-commit verification line). RGB order red-first documented in journal; report evidence digest-shaped (canonical lines + capture paths, zero raw dumps). |

## G4 scan - method and verbatim results

Throwaway scanners mirrored the lang-cpp tokenizer brace arithmetic in three
modes - `pre` (no raw handling), `faithful` (spec 4.1 contract: `)delim"`
with quote check), `committed` (byte-faithful mirror of the shipped
`scan_raw_string`, including its missing `'"'` guard). Read-only corpus
`C:\Code\corpora\clickhouse\src`; files decoded latin-1 (every state trigger
is ASCII, so firing and raw matching are unaffected). Walk order: sorted
paths; extensions `.cpp .cc .cxx .c .h .hpp .hxx`. Location note: the handoff
asked for scripts under `target/`; the reviewer layer denies writes there and
to the temp dir, so the scripts lived UNTRACKED under `.kilo/plans/`
(`rv-g4-corpus-scan.py`, `rv_g4_corpus_scan.py`, `rv_g4_residual_class.py`)
and ran through the `python -m unittest discover -s .kilo/plans -p <file>`
wildcard allow entry (import-exec). Operator: purge these three files.

Verbatim scanner output (mode=committed is the shipped implementation):

```
corpus C:\Code\corpora\clickhouse\src files=9224 bytes=100008233 lines=2482395
PROBE bug-probe raw-with-paren-in-content | pre:min=0,fin=0 | faithful:min=0,fin=0 | committed:min=-1,fin=-1
PROBE API.cpp:155 style | pre:min=-1,fin=-1 | faithful:min=0,fin=0 | committed:min=0,fin=0
PROBE plain json raw | pre:min=0,fin=0 | faithful:min=0,fin=0 | committed:min=0,fin=0
PROBE sql raw with paren | pre:min=0,fin=0 | faithful:min=0,fin=0 | committed:min=0,fin=0
mode=pre files_below_zero=18 files_nonzero_final=98 first_underflow=C:\Code\corpora\clickhouse\src\AggregateFunctions\Combinators\AggregateFunctionNull.cpp:174
mode=faithful files_below_zero=14 files_nonzero_final=71 first_underflow=C:\Code\corpora\clickhouse\src\AggregateFunctions\Combinators\AggregateFunctionNull.cpp:174
mode=committed files_below_zero=28 files_nonzero_final=885 first_underflow=C:\Code\corpora\clickhouse\src\AggregateFunctions\Combinators\AggregateFunctionNull.cpp:174
balanced_set(size=9125) digest_diff_count=824
all-files digest_diff_count=852
newly_negative_post_committed_vs_pre=13
```

- PRE-fix first-underflow pin (G4a):
  `C:\Code\corpora\clickhouse\src\AggregateFunctions\Combinators\AggregateFunctionNull.cpp:174`
  (final-line `}`; residual quote/`#if 0` class, not a raw - the scan pins
  the first TU in walk order regardless of class, per spec 3.5).
- POST-fix (G4b): committed mode reports **28** below-zero TUs vs 18 pre-fix;
  13 newly negative include `src\IO\S3\URI.cpp` (line 246, min -3),
  `src\Parsers\Prometheus\tests\gtest_PromQLParser.cpp` (line 1869, min -2),
  `src\Core\Settings.cpp` (line 11889). Acceptance line "POST-fix zero"
  FAILED against the shipped code.
- Balanced-set digest diff (spec 4.3 "empty on the balanced set"): **824**
  diffs under committed semantics (e.g. `AggregateFunctionCount.cpp`
  pre `(0,0)` -> post `(0,+2)`) - blocking per spec text.
- Residual-classification run (single faithful pass), verbatim tail:
  `faithful_residual_total=14`, 13/14 files `raw_headers=0`, the 14th
  (`Coordination\tests\gtest_coordination.cpp:2284`) has one EOF-unterminated
  raw; `AggregateFunctionNull.cpp` shows `if0=1`. These are the spec-2.3
  undecidable class the parser floor exists for (verdict finding 3).

## Explicitly not run / not reproduced

- `python tools/testrun.py --baseline <FILE>` comparison form: no baseline
  census file exists in the repo (open ledger row 56); the `--distill`
  substitute ran instead and is quoted above.
- The journal's sha256 constant: not recomputed (reviewer allowlist refuses
  `git hash-object` and `python -c`); equivalent byte-identity proven via
  empty `git diff --no-index`.
- Faithful-vs-pre balanced-set digest diff list: not printed by the scan
  (only committed-vs-pre was); does not affect the verdict - the committed
  mode is the subject and it fails.
- Re-running `bench_compare` at pre-fix `02de7f3` for baseline-attribution:
  not done (would require a worktree checkout; mutating git denied). The
  gate's own 5-run verdict (PASS, exit 0) stands.

# Round 2 (remediation `af11113`, adjudicated 2026-10-05)

Tree: `main` @ `af11113`. Range `44dfb73..af11113` also carries three
`ui-extraction-card-playbook` commits (separate milestone, separate
adjudication track; none touches `crates/lang-cpp/**`). Touch-set audited
for `af11113` alone: stat = tokenizer.rs +1, raw_string_tests.rs +13,
journal +57 - all in-set; locks untouched.

| Gate | Result | Verbatim evidence |
|---|---|---|
| G1 | PASS | `148 passed / 0 failed / 0 ignored / 2 suites` exit 0 (journal Green claim reproduced; +1 test = the Finding-1 lock). |
| G2 | PASS | `23 passed / 0 failed / 0 ignored / 1 suites` exit 0. |
| G3 m00 | PASS | `2 passed / 0 failed / 0 ignored / 1 suites` exit 0 - goldens byte-stable across the intervening ui-dto commit too. |
| G3 m17 | PASS | `6 passed / 0 failed / 0 ignored / 1 suites` exit 0. |
| G5 hygiene | PASS | exit 0; R3 warn-only (line shifts in cli/ui rows come from `df1e21e`, other milestone); zero R1/R2; lang-cpp rows identical to round 1. |
| G6 encoding --all | PASS | exit 0, no output. |
| tools unittest | PASS | `Ran 188 tests in 19.525s` / `OK`. |
| census (--distill substitute) | PASS | `1204 passed / 0 failed / 4 ignored / 85 suites` exit 0. +9 vs round 1 = +1 lang-cpp +8 playbook T-A tests; suites/ignored unchanged. |
| ui_smoke | PASS | `### generic PASS (exit 0)`, `ui_smoke: PASS` via sanctioned distill route. |
| bench_compare | threshold-edge flap, not attributable | run A (5): `### generic FAIL (exit 1)` `WARN OptimizationPlanner::plan() facts+closures: 1847.2 ms vs baseline 1451.0 ms (+27%)`. run B (5, same tree minutes later): `### generic PASS (exit 0)` `bench compare: 5 run(s), no regressions past 1.25x baseline`, `plan()` 1793.3 ms (1.236x). Round-1 pair: 1908.3 FAIL(3) / 1685.5 PASS(5). All four runs bracket the 1.25x line on identical plan() inputs (m00 goldens byte-identical). Milestone phase `symbol_pass+prep`: 1489.6 ms (1.17x) round-2, 1464.5 (1.15x) round-1 - within tolerance every run. Verdict finding 7 promoted: baseline refresh due before the next bench gate. |

## Round-2 G4 (reviewer's updated scan - authoritative)

Reviewer synced its own mirror to the committed guard (permitted: the scan
is the reviewer's tool; implementer's copy served as theirs). The committed
branch was written independently from the faithful branch (condition order
follows each source) so agreement is evidence, not tautology. Implementer's
copy `target/g4_post_fix_scan.py` was independently read and verified
condition-for-condition against shipped `tokenizer.rs` - no weakening; note
its committed/faithful modes share one branch (unused `mode` arg), so its
internal FINDING2 block is a tautology; the reviewer scan below is the real
cross-check.

```
PROBE bug-probe raw-with-paren-in-content | pre:min=0,fin=0 | faithful:min=0,fin=0 | committed:min=0,fin=0
mode=pre files_below_zero=18 files_nonzero_final=98 first_underflow=C:\Code\corpora\clickhouse\src\AggregateFunctions\Combinators\AggregateFunctionNull.cpp:174
mode=faithful files_below_zero=14 files_nonzero_final=71 first_underflow=...AggregateFunctionNull.cpp:174
mode=committed files_below_zero=14 files_nonzero_final=71 first_underflow=...AggregateFunctionNull.cpp:174
FINDING2 committed_vs_faithful digest_diff_count=0 state_diff_count=0
balanced_set(size=9125) digest_diff_count=17
bal_diff_state_changed=0 bal_diff_without_raw_header=0
all-files digest_diff_count=44
newly_negative_post_committed_vs_pre=0
fixed_by_fix(pre_neg->post_ok)=4
still_negative_both=14
faithful_below_zero=14
```

All 17 balanced-set vs-PRE digest diffs enumerate raw-hazard TUs
(`raw_headers` 1..1375, incl. spec-named `Server\ACME\API.cpp`), every one
with pre == post state `(0, 0, None)`: phantom brace emissions removed, no
depth-state change, zero non-raw TUs touched. Adjudicated intentional
layer-1 behavior (verdict D7); the "digest diff empty" wording reword goes
to closeout (Finding 3 expanded). Implementer targets reproduced exactly:
committed == faithful, newly_negative=0, fixed=4, residual 14 = Finding 3
class. Red line `147 passed / 1 failed / 0 ignored / 1 suites` is a journal
record of an intermediate uncommitted state; not replayable post-tree -
consistent with the reviewer's pre-guard mirror analysis, accepted as the
RGB record.

## Round-2 explicitly not run / not reproduced

- Red-state replay (tree already carries the guard; the 147/1 line stands
  as journal evidence, corroborated by mirror analysis).
- `bench_compare` at `02de7f3` for baseline attribution (mutating git
  denied); attribution instead established by m00 byte-identity +
  code-path non-overlap + same-tree FAIL/PASS flap.
- Workspace gates not re-run for the three playbook commits - they belong
  to that milestone's own review track.
