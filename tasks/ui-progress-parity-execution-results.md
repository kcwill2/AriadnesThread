# Execution results - ui-progress-parity (addendum, opened at T1 adjudication)

Opened by the T1 reviewer hop, 2026-10-04; closed out by the T3 HOP-3
re-adjudication the same day. FINAL HANDOFF STATE: T1 `5eff824`
APPROVED, T2 `587c409` APPROVED, T3 `17e2299` APPROVED after three
reviewer hops (hop-1 blocker: legacy text rows not hidden under rows,
fixed `9d96272`; hop-2 blocker N1/N2: error/ready text invisible after
trio hiding, fixed by Variant A single restore site in `updateProgress`,
plus R2 `body_slice` compaction verified deletion-free; hop-3 APPROVED).
Ledger deferred rows (a) stage-5 frame-count errata and (b) kickoff G7
`--baseline` misconfiguration appended by the reviewer in `cad6efb`
(ledger-only scope). Gates: G1 scoped cli `96 passed / 0 failed /
1 ignored / 5 suites` + scoped ui `184 passed / 0 failed / 0 ignored /
3 suites`; G2 frozen oracle (progress.rs EMPTY diff e276910..HEAD); G4
theme hex check silent vs main AND vs e276910; G5 hygiene exit 0
(R3 warn-only) + encoding_guard --all silent; G6 `Ran 188 tests in
19.895s` + `OK`; G7 workspace census `1168 passed / 0 failed / 4
ignored / 85 suites` (the literal `--baseline FILE` form is unrunnable -
no committed baseline, see ledger row (b); census taken at `1bc50b8`;
rework ranges added only UI assets/tests/docs and the scoped ui suite
was re-proved green at `a17952a` and `24ecb70`). NOT RUN in any agent
session (sandbox refusal, verbatim texts in the T3 verdict): G3 `python
tools/ui_smoke.py` - REQUIRED in a fresh human/orchestrator session -
and `python tools/bench_compare.py` (not a kickoff gate). Milestone
closeable: NO until G3 runs green; all code handoffs are approved and
landed on `main`.

## T1 - ServerProgress additive step-rows DTO (commits 5eff824, 000238b)

Verdict: APPROVED. See `tasks/ui-progress-parity-t1-verdict.md` for findings.

| Acceptance criterion (T1 handoff) | Result | Evidence |
|---|---|---|
| Gate 2 green: scoped ui suite | PASS | reviewer's own `python tools/testrun.py --distill -p ariadne-ui` exit 0: `184 passed / 0 failed / 0 ignored / 3 suites` (capture `target/testrun-capture.txt`) |
| Gate 3 hygiene | PASS | `python tools/hygiene_check.py` exit 0, R3 warn-only; zero warnings in the new-code line ranges |
| Gate 4/encoding | PASS | `python tools/encoding_guard.py --all` exit 0, clean |
| Commit-vs-touch-set | PASS with 1 adjudicated deviation | `git diff --name-only e276910..HEAD` = 4 touch-set files + `tasks/milestone_ledger.md` (accepted, verdict finding 1) |
| Per-commit review oldest-first | PASS | `git show "5eff824"` then `git show "000238b"`; DTO matches shape 89-119 verbatim; server.rs hunks are the two literals only; test diff is additions + one `use` widening |
| Frozen locks (I9 et al.) | PASS | `git diff e276910..HEAD` empty for `crates/pipeline`, `crates/cli` (whole crate incl. `commands/progress.rs`), `crates/ui/assets`, `themes.css`, `theme.js`, `tests/fixtures`, `tests/golden`, `tools/theme_token_baseline.json`, `static_html.rs`, `common.rs` |
| I4 readiness contract | PASS | `test_status_route_locks_ready_response_shape` byte-untouched in diff; green run passes it |
| I5 additive serde | PASS | `test_server_progress_from_legacy_four_field_json_gains_empty_step_fields` present, asserts defaults + legacy round-trip; green |
| Mid-boot wire fixture | PASS | `test_status_route_locks_mid_boot_step_rows_shape` locks shape lines 125-137 field-for-field incl. lowercase wire strings; green |
| I6 error freeze | PASS | `test_error_arm_write_pattern_freezes_step_rows` mirrors `ui.rs:720-722` (writes only `state`/`message`), Vec-equality of rows; green |
| ServerProgress literal sites | PASS | grep confirms ONLY `server.rs:60` (`new_loading`) and `server.rs:84` (`new_loaded`) construct the struct |
| Serde serialization of fresh state gains keys | PASS | `test_progress_serialization_gains_empty_step_fields`; no `skip_serializing_if` in DTO |
| Hygiene ceilings | PASS | `dto.rs` 880 lines (<1000), `server.rs` 792 lines; new functions far <100 lines |
| RGB protocol fidelity | PASS | Red evidence + honest compile-error notation in journal (verdict finding 3) |

## T2 - BootStepSnapshot producer + terminal-parity oracle (commit 587c409)

Verdict: APPROVED. See `tasks/ui-progress-parity-t2-verdict.md` for findings
and deviation dispositions. Reviewer hop 2026-10-04.

| Acceptance criterion (T2 handoff) | Result | Evidence |
|---|---|---|
| Gate 2/3 green scoped cli | PASS | reviewer's own `python tools/testrun.py --distill -p ariadne-cli` exit 0: `96 passed / 0 failed / 1 ignored / 5 suites` (capture `target/testrun-capture.txt`), exactly the journaled green/post-refactor line |
| Gate 1 Red (historical) | ACCEPTED evidence-shape only | journal: `86 passed / 1 failed / 0 ignored / 1 suites`, failing `...boot_progress_tests::test_boot_progress_snapshot_writes_active_row` with the predicted `assertion failed: !p.steps.is_empty()`; not re-executable without a revert |
| Gate 4 hygiene | PASS | `python tools/hygiene_check.py` exit 0 (harness non-zero-is-failure semantics; explicit `$LASTEXITCODE` echo refused); R3 warn-only, new-code warns exactly the handoff-prescribed `p`/`bar` names + one test-local `p` |
| Gate 5/encoding | PASS | `python tools/encoding_guard.py --all` exit 0, clean |
| Commit-vs-touch-set | PASS | `git diff --name-only 000238b..HEAD` = exactly the 4 claimed files; no Cargo.lock; scopes match gate 6 verbatim |
| Per-commit review oldest-first | PASS | single commit `587c409`, 6 hunks, all in prescribed wiring regions |
| G2 frozen renderer | PASS | `git diff e276910 -- crates/cli/src/commands/progress.rs` EMPTY; oracle passes against it; `crates/pipeline` diff EMPTY (I9); `progress_contract_tests.rs` untouched |
| I1 per-step scale | PASS | stage-5 row percents asserted equal to `[0,5,5,5,5,5,10,25,25,25,30,95,100]`; earlier rows `<100` before / `==100` from own end frame across all 21 frames |
| I2 terminal-parity oracle | PASS | replay through unchanged `TerminalStepBars`+local MemorySink vs `BootStepSnapshot`; positional field extraction; committed + finalize lines and open-line (via `[..20]` prefix) all exercised; oracle model verified against render byte semantics |
| I3 monotonicity + stale ignore | PASS | full-stream/prefix/mid-start property loops + direct stale-frame test; finalize inertness test |
| DP-1 seed / I4 / I6 wiring | PASS | seed write after `new_loading`; globals unchanged in same single write pass; error arm + `swap_into_state` + quiet path have zero hunks |
| Shape JSON example | PASS | field-for-field reproducible rows asserted from seed + stage-1-end + stage-2-mid frames |
| Hygiene ceilings | PASS | `ui.rs` 754 -> 790 (+36 <= 60, < 820); `boot_progress.rs` 115 (<= 450); functions << 100 lines |
| T1 open item #4 (suite census) | CLOSED | `python tools/testrun.py --distill -p ariadne-ui` exit 0: `184 passed / 0 failed / 0 ignored / 3 suites` |
| Deviations 1-4 | ACCEPTED (all 4) | independent adjudication in verdict; deviation 2 confirms the HANDOFF's "11 reports" is the wrong count (source has 10 stage_report calls; 2 are stage-5 frames subsumed by the pinned 13-frame sequence) |

Refused-gate log for the T2 hop is recorded in
`tasks/ui-progress-parity-t2-verdict.md`; every substantive gate succeeded
on individual re-attempt.

## T3 - overlay step-row renderer (commits e9cf10d, 1bc50b8)

Verdict: REJECTED (blocker finding 1: legacy text rows not hidden under
step-row rendering, violating shape.md:182-184; plus minor ASCII-comment
finding). See `tasks/ui-progress-parity-t3-verdict.md` for all findings,
deviation adjudication, and the exact remediation list. Reviewer hop
2026-10-04; this hop also ran the workspace-wide gates deferred by T1/T2.

| Acceptance criterion (T3 handoff) | Result | Evidence |
|---|---|---|
| Gate 2 green scoped ui | PASS | reviewer's own `python tools/testrun.py --distill -p ariadne-ui` exit 0: `184 passed / 0 failed / 0 ignored / 3 suites` (capture `target/testrun-capture.txt`), exactly the journaled Green + post-refactor lines |
| Gate 1 Red (historical) | ACCEPTED evidence-shape only | journal: `183 passed / 1 failed / 0 ignored / 2 suites`, failing the predicted `assertion failed: html.contains("id=\"loadingSteps\"")` at static_html.rs:540; not re-executable without a revert; suite-count oddity = verdict finding 6 |
| Gate 3 theme token check (G4) | PASS | `python tools/theme_token_check.py --check-added --base main` silent; re-run substantively `--base e276910` also silent - zero added hex across the whole milestone; silent==exit-0 claim VERIFIED from tool source (prints violations only; `return 1 if violations else 0`) and by reviewer runs |
| Gate 4 hygiene | PASS | `python tools/hygiene_check.py` exit 0 (harness non-zero-is-failure semantics); R3 warn-only, none in any T3-touched file |
| Gate 5/encoding | PASS | `python tools/encoding_guard.py --all` silent = exit 0 (source-verified exit semantics + reviewer run); NOTE: raw block glyphs committed in a main.js:261 comment are valid UTF-8 so the guard cannot catch them - separate rule violation (verdict finding 2) |
| Commit-vs-touch-set | PASS | `git diff --name-only 587c409..HEAD` = exactly the 6 claimed paths; `git show --stat "e9cf10d"` = 202+/4- verified; `1bc50b8` journal-only +11; all 4 deletions on prescribed api.js replacement lines |
| Per-commit review oldest-first | PASS with findings | `git show "e9cf10d"` whole, then `git show "1bc50b8"`; legacy needles all kept (static_html.rs:526-537); DOM composed via textContent only |
| Frozen locks | PASS | `git diff 587c409..HEAD` EMPTY for crates/cli, crates/pipeline, crates/ui/src, server_api.rs, common.rs; `git diff e276910..HEAD` EMPTY for crates/pipeline, progress.rs, themes.css, theme.js, theme_token_baseline.json, tests/fixtures, tests/golden |
| DP-1 dim-from-boot pending rows | PASS | all rows render; step-pending = opacity 0.45 + muted, empty detail (main.js:306, styles.css block) |
| DP-2 literal \u2588/\u2591 20-cell bars | PASS | `Math.floor(20 * percent / 100)` verbatim in `buildStepBarCells`, clamp-then-floor, total 20, `[`/`]` wrap, code uses escapes only, test locks backslash-u + formula needles (raw-glyph comment violation split out as finding 2) |
| DP-3 accent-only done rows | PASS | done rows `var(--cyan)` only; zero green/amber/red or heat tokens in the appended block; all 5 tokens used exist in all three theme blocks (themes.css 51-56/182-187/321-326) |
| Legacy-bar fallback behavior | PARTIAL -> blocker | empty-steps delegation to updateProgress present; three legacy function names+behavior kept; hideProgress clears #loadingSteps. BUT row path hides only the bar wrapper (main.js:284-285): loadingText/loadingDetail stay visible with stale frozen text while rows render - violates shape.md:182-184 "hidden when rows render" (binding input). Verdict finding 1 (blocker) |
| One renderer (both api.js branches) | PASS | poll branch line 56 = `renderProgressSnapshot(status.progress)`; standalone feeds `standaloneBootSnapshot(0/1/2)`; ready/error branches untouched; `state === 'loading'` needle intact |
| Hygiene ceilings | PASS | main.js 513 -> 570; renderer ~38 lines, helper 6; styles.css appended-only; no line near 2000 chars |
| Report evidence (distillation) | PASS with format finding | canonical lines quoted; no capture-path/exit-0 lines for gates 3/5 (claims verified true by reviewer from tool sources) = verdict finding 3 |

## Workspace-wide milestone gates (final T3 hop, reviewer-run)

- G6 `python -m unittest discover -s tools -p "*_tests.py"`: PASS -
  `Ran 188 tests in 19.895s` + `OK`.
- G7 census `python tools/testrun.py --distill` (workspace): PASS -
  `1168 passed / 0 failed / 4 ignored / 85 suites` (exit 0, capture
  `target/testrun-capture.txt`, 2042 lines). The kickoff's literal
  `--baseline` form is unrunnable: `--baseline` requires a FILE argument
  and no baseline census file is committed anywhere in the repo
  (verdict finding 7 - doc/config errata for orchestrator triage).
- G4 `theme_token_check --check-added`: PASS at milestone scope (silent
  vs `--base e276910`).
- G5 hygiene + encoding: PASS (exit 0 both, quoted in T3 table).
- G1/G2: MET (scoped green lines this hop + T2 hop; progress.rs EMPTY
  diff vs e276910 re-proven this hop).
- G3 `python tools/ui_smoke.py`: NOT RUN - refused in the reviewer
  sandbox (2 attempts incl. explicit workdir), verbatim refusal text
  recorded in the T3 verdict finding 8; consistent with the sizing-law
  carve-out to fresh-session/orchestrator runs; orchestrator must run it
  for closeout.
- `python tools/bench_compare.py`: NOT RUN - refused (1 attempt, same
  verbatim refusal). Not among kickoff G1-G7; T3 is assets-only with
  crates/pipeline byte-identical.

## NOT RUN (final state after the T3 hop)

- `python tools/ui_smoke.py` (kickoff G3) - NOT RUN in any hop:
  refused in the reviewer sandbox at the T3 hop; required in a
  fresh/orchestrator session before the milestone can close.
- `python tools/bench_compare.py` - NOT RUN: refused at the T3 hop
  (not a kickoff gate).
- `python tools/testrun.py --baseline FILE` comparison - NOT RUN: no
  baseline file exists in the repo to point at; the workspace CENSUS
  equivalent ran green (`1168 passed / 0 failed / 4 ignored / 85
  suites`).
- Red-phase re-runs (T1/T2/T3): historical Red evidence accepted from
  the journals; Red cannot be re-executed against landed code without a
  revert, which reviewer git constraints forbid.

## Refused-gate log (T1 hop)

Combined shell forms (`cmd; echo $LASTEXITCODE` chains; one `python -c`
probe) were refused with the boilerplate catch-all rule
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
Every substantive gate (testrun --distill scoped, hygiene_check,
encoding_guard --all, checkpoint_guard status, read-only git) succeeded on
individual re-attempt. No gate in the T1 scope was left unexecuted for
permission reasons.
