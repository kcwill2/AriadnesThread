# M23.5 T3-B Execution Results - CLI Consumer Rewiring + Stage-5 Bench Re-Pin (crate:cli)

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Handoff:** `tasks/m235-t3b-handoff.md`
**Range:** `ea82422..91967c9` (base `ea82422` = T3-A tip; HEAD `91967c9`; branch `tarjan-s-scc`; worktree `.kilo/worktrees/tarjan-s-scc`)
**Commits:** `9d831bf`, `52f8aa2`, `a55091c`, `9194c25`, `9f30c9a`, `91967c9`
**Verdict:** `tasks/m235-t3b-verdict.md` - **APPROVED**, 0 blocking / 8 non-blocking
**Discharged deferred item:** T3-A verdict sec 8 / finding 1 (two-process export
byte-stability) - owner reassigned to unit B. **DISCHARGED** (section 5 of the verdict).

This addendum is the execution-results half of the review. Everything below was
**measured by the reviewer in this session**, not relayed from the implementer's
journal. Where the implementer's report and the reviewer's measurement differ,
the difference is stated.

---

## A. Ground truth

`python tools/checkpoint_guard.py status` at reviewer session start:

- HEAD `91967c9`, branch `tarjan-s-scc`
- DIRTY FILES (`git diff HEAD --numstat`): `(none)`
- UNTRACKED FILES: `(none)`
- WORKTREE CENSUS: 8 worktrees; `.kilo/worktrees/tarjan-s-scc` `ahead=16 behind=13 dirty=1`

The single `dirty=1` is the F12 stat-cache entry for
`tests/fixtures/basic/artifacts/includes/report.json` - adjudicated in section D.

Base note: the handoff pins line references at `b950963`, but the unit's actual
base is `ea82422` (T3-A's tip). Confirmed by handoff sec13 (an APPROVED T3-A is
a precondition), sec0 step 0.3 (`build_include_elements_from_groups` must
exist), and the commit graph (`ea82422` immediately precedes `9d831bf`).

---

## B. Acceptance criteria, criterion by criterion

| # | Handoff / plan criterion | Result | Evidence line / measurement |
|---|---|---|---|
| 1 | sec2 touch-set: exactly `ui.rs`, `includes.rs`, `payload_tail_bench.rs`, the journal, plus orchestrator-declared records | **MET** | `git diff --name-only ea82422..HEAD` = exactly those 6 paths (3 production + 3 records). Per-commit `--stat` name lists in verdict sec 2. |
| 2 | sec0 step 0.4: every commit carries an explicit `--scope`; forbidden fixture in NO commit | **MET** | Six per-commit path lists, none containing any `tests/` path. Fixture absent from all six. |
| 3 | sec3.1 B1/B2: `detect_cycle_groups` + `build_include_elements_from_groups` with the same 5 args; `node_maps::build` kept | **MET** | `ui.rs:322` `let cycle_groups = include_graph.detect_cycle_groups();`, `:328` `build_include_elements_from_groups(` carrying `&include_graph, &closures, &cycle_groups, &node_maps.languages, &node_maps.unit_kinds`; `:323` `let node_maps = node_maps::build(` is a diff CONTEXT line in `a55091c`, i.e. byte-untouched at column 0. |
| 4 | sec3.2 B5: label becomes `Cycle Groups Detected` with the two leading spaces | **MET** | `includes.rs:83` `"  Cycle Groups Detected"`; print at `:121` `println!("{}: {}", cycle_groups_label(vocabulary.as_ref()), report.cycle_count);` - same shape as neighbours `:93`-`:120`. |
| 5 | sec3.3 B6: `ui.rs:103` names `detect_cycle_groups` and carries the measured figure | **MET** | `ui.rs:103` `// generate_report+detect_cycle_groups 5.6ms ...`. No stale `detect_cycles`, no surviving bare `4.7ms` beside it - `4.7` survives only inside the provenance note explaining its retirement. |
| 6 | sec3.4 bench docs `:20` / `:39` / `:361` all name `detect_cycle_groups` and the measured number | **MET** | `:20` phase note, `:40` table row `generate_report+detect_cycle_groups (combined)   \|      5.6`, `:388` emitted column label. (One UNDECLARED-BY-SEC3.4 sibling staleness remains at `:389` - verdict finding 4, non-blocking, planner.) |
| 7 | sec4: genuine compiling Red on the label, helper extracted with the CURRENT bytes first | **MET** | `9d831bf` adds the helper returning the OLD `"  Cycles Detected"` with NO test; `52f8aa2` flips the literal and adds the test. Ordering provable from history. Red's verbatim location `includes.rs:220:9` matches HEAD's first `assert_eq!` at 8-space indent (column 9). |
| 8 | sec6: the three existing `includes.rs` tests pass unedited | **MET** | `includes.rs:177 / :197 / :225` all appear as diff CONTEXT lines in `9d831bf` and `52f8aa2`, never as `+`/`-`. Gate-1 capture lines 8, 9, 19 name all three `... ok`. |
| 9 | sec8 gate 1 expected `87 passed / 0 failed / 1 ignored / 3 suites` | **DEVIATION (D1+D3), accepted** | Reviewer-measured: `88 passed / 0 failed / 1 ignored / 5 suites`. Both halves of the delta attributed: +1 test (D1) and +2 suites from two `[[bin]]` targets (D3). Verdict sections 4 and 9. |
| 10 | sec8 gate 2 expected `1 passed / 0 failed / 0 ignored / 1 suites` | **MET** | Reviewer-measured verbatim: `1 passed / 0 failed / 0 ignored / 1 suites` (exit 0, capture 26 lines). Required the D2 doubled-`--` invocation shape; the CONTRACT's single-`--` line is the defect. |
| 11 | sec5: measurement must be `--release`; a debug number is not comparable | **MET** | Capture line 25 `Finished \`release\` profile [optimized] target(s)`, line 26 `target\release\deps\payload_tail_bench-....exe`. |
| 12 | sec5/sec8: gate 2 must not be vacuous; bench is `#[ignore]`d so it must be proven to have run | **MET** | Capture: `running 1 test`, `payload_tail_phase_timing ... TAIL_BENCH headers=520 tus=40 reduced_symbols=2160 file_metrics=560 justification_entries=8960 plans=520 conservative_rows=520`, `ok`, `1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 13.22s`. |
| 13 | sec5: UNVERIFIED sentence in the journal AND in the file comments; do not run or evade `bench_compare.py` | **MET** | Journal Checkpoint 2; `payload_tail_bench.rs:79-85`; `ui.rs:114-118`. Reviewer independently confirms `bench_compare.py` is still REFUSED (section C). |
| 14 | sec7: zero R1/R2 hygiene; authored content pure ASCII; no source line >= 2000 chars | **MET** | `python tools/hygiene_check.py` - zero R1/R2 workspace-wide, all output is legacy R3 warns. `python tools/encoding_guard.py --all` and `--staged` both silent. |
| 15 | sec9 frozen locks (14 rows) | **ALL HOLD** | Verdict section 7 table, each with its proving command. |
| 16 | sec10 audit 2: "a handoff that implies reviewer-verified perf evidence is a blocking finding" | **NOT TRIGGERED** | No perf verification is claimed anywhere; the tree states at both sites that no such evidence exists. Verdict section 4. |
| 17 | sec10 audit 4: no over-claim on the payload change | **MET** | Journal Checkpoints 1 and 5 both attribute the chord-classification change to unit A and claim only byte-STABILITY here. |
| 18 | T3-A verdict sec 8 / finding 1 (assigned to this unit): two-process export byte-stability lock | **DISCHARGED** | Verdict section 5. All four audited properties CONFIRMED; the test is committed in `a55091c` and GREEN in the reviewer gate-1 capture (line 106). |
| 19 | plan:128 / plan:147 / IV-5 (plan:29): two-process export byte-stability is a milestone requirement | **NOW SATISFIED, with a declared residual** | Verdict sections 5.1-5.5. Residual: two samples (accepted, section 5.5). Companion parity test remains open with a HARD EXPIRY at unit E - verdict finding 3. |

---

## C. Gates: what ran, what could not

### Scoped gates (this unit's contract), reviewer-run

| Gate | Command | Reviewer-measured canonical line |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-cli` | `88 passed / 0 failed / 1 ignored / 5 suites` |
| 2 | `python tools/testrun.py --distill --timeout 3600 --release -p ariadne-cli --test payload_tail_bench -- -- --ignored --nocapture --test-threads=1` | `1 passed / 0 failed / 0 ignored / 1 suites` |

### Workspace-wide gates

| Gate | Command | Result |
|---|---|---|
| baseline | `python tools/testrun.py --baseline` | **DID NOT RUN (broken).** Verbatim: `usage error: missing value for --baseline`. Ledger M23.5 F9. Attempted ONCE by the reviewer, recorded once, NOT treated as a gate failure, NOT worked around, NOT routed through `distill.py`. |
| full workspace (what `--baseline` wraps) | `python tools/testrun.py --distill --timeout 3600` | **`1176 passed / 0 failed / 4 ignored / 88 suites`**, exit 0, capture 2065 lines |
| tool unit suites | `python -m unittest discover -s tools -p "*_tests.py"` | **`Ran 184 tests in 18.444s`** then `OK` |
| encoding, all tracked | `python tools/encoding_guard.py --all` | no output, exit 0 |
| encoding, staged | `python tools/encoding_guard.py --staged` | no output, exit 0 |
| hygiene | `python tools/hygiene_check.py` | zero R1/R2; every printed line a legacy-file R3 warn |
| theme tokens | `python tools/theme_token_check.py --check-added --base ea82422` | no output, exit 0 = zero added hex literals |
| fixture state | `python tools/checkpoint_guard.py status` | run 3x: at session start, after gate 2, after the workspace run |
| UI runtime smoke | `python tools/ui_smoke.py` | **DID NOT RUN - REFUSED.** Verbatim: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` |
| load-perf regression | `python tools/bench_compare.py` | **DID NOT RUN - REFUSED.** Verbatim: identical text. |

### Why the denials changed nothing about the acceptance set

Both refusals were attempted **independently, directly, once each**, and neither
was retried through `distill.py` or any other wrapper. A refusal always reports
the catch-all deny rule regardless of the real cause; here the real cause is
that neither script is on the reviewer bash allowlist (ledger F10), not that
bash is unavailable. **After the two refusals the reviewer still ran hygiene,
`encoding_guard --all`, `encoding_guard --staged`, `theme_token_check`,
`checkpoint_guard status` (three times), scoped gate 1, scoped gate 2, the
full-workspace run, and the Python tool suite** - all of them succeeded.

**Two further refusals occurred on reviewer-shaped commands and are recorded for
the next reviewer's benefit:** (a) a compound `git log ...; echo ...; git diff
--name-only ...` with `2>&1` was refused, and re-issuing the queries as plain
single commands (and via the dedicated git tools) succeeded; (b) `git status
--porcelain <path>` was refused, and the dedicated status tool returned the same
information. In both cases the refusal was self-inflicted by command shape, and
in both cases the acceptance set was not narrowed as a result.

### Not run at all, stated plainly

- `testrun.py --baseline` (broken, F9) - the wrapped invocation was run instead.
- `ui_smoke.py` (refused, F10) - **no runtime UI evidence exists for this
  milestone**, which matters more after this unit than after any earlier M23.5
  unit (verdict finding 6).
- `bench_compare.py` (refused, F10) - **no verified perf-regression evidence
  exists for `detect_cycle_groups`.** The pinned `5.6` is a single local
  measurement and says so at both sites.
- The raw Red capture - not recoverable. The Red revision was correctly never
  committed and `target/` is ephemeral; the verbatim failure text survives in
  the committed journal.
- No commit was made by this reviewer (write git is denied to this role).

---

## D. Fixture state and the F12 mechanism

| | at reviewer session start | after reviewer gate 2 (520-header run) | after reviewer workspace run |
|---|---|---|---|
| DIRTY FILES (numstat) | `(none)` | **`(none)`** | **`(none)`** |
| UNTRACKED | `(none)` | `tasks/m235-t3b-verdict.md` | `tasks/m235-t3b-verdict.md` |
| census `dirty=` | `1` | `2` (+1 = my untracked verdict file) | `2` |

`git status` names exactly one tracked modification across all three readings:
`modified: tests/fixtures/basic/artifacts/includes/report.json`.

**Adjudication: known F12 mechanism, fourth identical reproduction** (T2-A
finding 9, T2-B finding 10, T3-A sec 7, now T3-B). numstat clean + porcelain
dirty + content byte-equal + path in no commit. Cause: the path is unpinned by
`.gitattributes` and this worktree has `core.autocrlf=true`, so a run that
rewrites the artifact invalidates the stat-cache entry while `git diff`
normalises line endings and finds no content delta. No CRLF bytes entered the
repository through this unit. No fix attempted - `git checkout --` / `git
restore` are denied to every role, and rewriting a byte-equal file would be a
content change with no delta to make.

**Standing closeout obligation, still open:** pin the path in `.gitattributes`
or make the artifact writer emit LF. Must land before T4.1.

---

## E. Workspace delta, attributed

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| before this unit (T3-A measured at `ea82422`) | 1174 | 0 | 4 | 88 |
| **after this unit (`91967c9`)** | **1176** | **0** | **4** | **88** |
| handoff sec8 prediction | 1175 | 0 | 4 | 88 |
| **delta** | **+2** | +0 | +0 | **+0** |

**+2 tests, fully attributed, zero unattributed residual:**

| New `#[test]` | Location | Target | +1 |
|---|---|---|---|
| `includes_summary_labels_report_cycle_groups_not_cycle_paths` | `crates/cli/src/commands/includes.rs:216` | `unittests src/lib.rs` | +1 |
| `export_bytes_are_identical_across_two_separate_processes` | `crates/cli/tests/payload_tail_bench.rs:569` | `tests\payload_tail_bench.rs` | +1 |

The handoff's `+1` is exactly "the Red's test only" - which is what a handoff
authored at `b950963`, before the T3-A verdict reassigned the export lock,
would predict. `+0 suites` is correct because both tests live in pre-existing
targets. That is the same tree-shape fact that makes the *scoped* gate-1 suite
count **5** rather than 3 while the *workspace* suite count stays **88**:
`crates/cli/Cargo.toml:7-13` declares two `[[bin]]` targets
(`ariadne`, `tool`), and cargo runs a zero-test unittests binary for each.

---

## F. Findings and dispositions

**APPROVED. 0 blocking / 8 non-blocking.** Full text in
`tasks/m235-t3b-verdict.md` section 10.

| # | Severity | Attributed to | Summary | Disposition |
|---|---|---|---|---|
| 1 | non-blocking | planner | Handoff sec4's Red line is internally impossible (`0 failed` + "one failing"); its `3 suites` also inherits finding 2 | Correct before the text is inherited |
| 2 | non-blocking | planner | Handoff sec8 and the ledger's `-p ariadne-cli = 86/0/1/3` row omit the two `[[bin]]` unittests suites; truth is 5 | Correct the ledger row |
| 3 | non-blocking | planner | Companion parity test left "open, unassigned"; it **expires at unit E** (E deletes the legacy builder) and unit E forbids adding tests | New `crates/ui`-only unit between D and E; ledger row annotated "expires at unit E" |
| 4 | non-blocking | planner | sec3.4's site list omits `payload_tail_bench.rs:389`, whose emitted row label still reads `build_include_elements` after B4 moved to the from-groups builder | Rename in unit E with the entry-point rename |
| 5 | non-blocking | planner / architect | The CLI label says "Cycle Groups Detected" while `report.cycle_count` is still a DFS-path count (`graph.rs:356`/`:369`); self-closes at unit D | Do not read the Phase 3 label as group-accurate until unit D lands |
| 6 | non-blocking | configuration (F10) | `ui_smoke` + `bench_compare` refused; no runtime UI evidence for a unit that changed the served bytes; no verified perf evidence | Close F10 before T4.1 (operator/allowlist action) |
| 7 | non-blocking | configuration (F9) | `testrun.py --baseline` broken (`usage error: missing value for --baseline`) | Fix at closeout; the wrapped run was executed instead |
| 8 | non-blocking | implementer | Two-process lock samples two runs | Accepted (verdict 5.5); keep finding 3 alive until unit E |

### Deviation dispositions D1-D6

All six **ACCEPTED**. Full adjudication in verdict section 9. Summary of
attribution where the CONTRACT is at fault: **D2 and D4 and the D3 count are
planner/handoff-spec defects, not implementer shortfalls** - and in D4's case the
implementer reported the contradiction instead of papering over it, which is the
behaviour the gate exists to reward.

---

## G. Follow-ups owed (recorded, not dropped)

| Item | Owner | Due |
|---|---|---|
| Companion parity test (`from_groups` vs legacy builder byte-equality, acyclic) | planner - author a `crates/ui`-only unit | **before unit E is dispatched (hard expiry)** |
| `payload_tail_bench.rs:389` emitted row label rename | unit E (with the entry-point rename) | unit E |
| Correct handoff sec4's Red line and sec8's/ledger's suite count | planner / milestone closeout | closeout |
| F10: `ui_smoke.py` + `bench_compare.py` off the reviewer allowlist | operator / configuration | **before T4.1** |
| F9: `testrun.py --baseline` argument bug | maintainer | closeout |
| F12: pin `tests/fixtures/basic/artifacts/includes/report.json` in `.gitattributes` (or emit LF from the writer) | maintainer | **before T4.1** |
| T3-A verdict findings 4, 5 (plan/handoff text corrections), 3 (`witness_path_labels` unregistered-member policy) | planner / architect | unit E or T4.1, per the T3-A verdict |
| Milestone ledger: mark the two-process export-stability deferred row DISCHARGED, citing `tasks/m235-t3b-journal.md` and commit `a55091c` | milestone-lifecycle step | closeout |