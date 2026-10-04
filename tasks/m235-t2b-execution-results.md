# M23.5 T2-B Execution Results - reviewer acceptance addendum

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Range:** `7c94af3..b950963`, HEAD `b950963`, branch `tarjan-s-scc`
**Verdict:** `tasks/m235-t2b-verdict.md` - APPROVED, 0 blocking / 10 non-blocking

Written to `tasks/m235-t2b-execution-results.md`, NOT to
`tasks/m235-execution-results.md`, following the T2-A precedent: the latter path
holds the Phase 1 acceptance record and was committed intact in `08741b8`.

---

## 1. Canonical workspace line, verbatim

```
1169 passed / 0 failed / 4 ignored / 88 suites
```

Baseline as corrected by T2-A: `1165 / 0 / 4 / 87`.

Delta: **+4 passed, +0 failed, +0 ignored, +1 suite.** Expected delta was
exactly +1 suite and +4 tests, so the measured delta equals the prediction with
**zero unattributed residual.** This is the discipline that caught the Phase 1
F11 skew, and it is clean here: the only new target in the range is
`crates/graph/tests/cycle_groups_equivalence_test.rs` with four `#[test]`
functions, all four named in the gate-2 capture.

Note for the ledger (verdict finding 7): the M23.5 milestone row still carries
`1156 / 0 / 4 / 86` and "Phase 2 ... not started". It needs T2-A's
`1165 / 0 / 4 / 87` and this unit's `1169 / 0 / 4 / 88` before T4.1 re-pins.

## 2. Handoff acceptance criteria, one by one

### Sec 5 Step 1 - four `#[test]` functions with the exact pinned names

MET. All four present in `crates/graph/tests/cycle_groups_equivalence_test.rs`
and all four named as passing in the gate-2 capture:
`test_group_membership_equals_old_cycle_node_set_on_registered_corpus`,
`test_old_cycle_members_are_subset_of_a_single_group`,
`test_new_groups_never_miss_a_file_the_old_enumeration_called_cyclic`,
`test_unregistered_endpoint_cycle_is_found_by_groups_but_not_by_old_paths`.
Evidence line: `4 passed / 0 failed / 0 ignored / 1 suites`.

### Sec 5 Step 1 - corpus regime-mix assertion

MET. `test_new_groups_never_miss_a_file_the_old_enumeration_called_cyclic`
asserts both `saw_acyclic_regime` and `saw_nontrivial_group` over
`full_corpus()`. Non-vacuous: the edgeless fixtures and the PureDag seeded
graph supply the acyclic regime, the 2-cycle fixtures supply the nontrivial one.

### Sec 5 Step 1 - exact-equality anti-vacuity floor

MET and measured. `exact_cases >= 20`; I enumerated the sweep independently and
the measured value is 23 of 25 graphs in `fixture_corpus() + seeded_corpus()`,
the two exclusions being acyclic characterization shapes.

### Sec 5 Step 2 - mutation-proven Red (the IV-5 tie-break reversal)

MET, verified by causal proof rather than re-execution (verdict section 5,
finding 1). Reported failure: `3 passed / 1 failed / 0 ignored / 1 suites`, panic
at `cycle_groups_equivalence_test.rs:702:5` - which is exactly where the
`assert_eq!` token sits in the committed file - message matching lines 705-706
verbatim, `left: [[3, 4], [1, 2]]` which is precisely what `ordered.reverse()`
produces on two incomparable sinks.

### Sec 5 Step 3 - clean Green after revert

MET. `cycle_groups.rs` byte-equal to `7c94af3` (`git diff` empty). Gates 1 and 2
reproduce their pinned lines exactly.

### Sec 5 Step 4 - gap fixes

MET, with the required explicit record. **No gap was found.** No test failed
against unmutated production code, so `crates/graph/src/cycle_groups.rs` carries
no T2-B diff at all and the commit scope contains no production path. The
journal states this outcome explicitly rather than leaving it ambiguous.

### Sec 4 - the one legitimate divergence, named not filtered

MET. Exactly one genuine old-vs-new divergence exists in the corpus and it is
named in four places (module doc, graph-constructor doc, test name, journal).
The registered-endpoint gate excludes three graphs; I checked each and only the
named one actually diverges - the two characterization shapes are acyclic with
empty old and new sets. The divergence graph is carried inside `full_corpus()`
so tests 2 and 3 assert over it rather than skipping it.

### Sec 3.1 / Sec 7 - the oracle is a frozen verbatim copy

MET. `git diff 2708425 HEAD -- crates/graph/src/graph.rs` is empty, so the
checkout equals the pinned pre-migration blob. Field-by-field comparison of
`:234-287` against the test copy at `:90-143` shows every statement identical
and zero locals renamed. The three declared test-locality adaptations are each
inert (receiver rename; associated-fn to free-fn where the body never touches
`self`; a line-for-line mirror of the `pub(crate)` `adjacency()`).

### Sec 3.3 - corpus copied, not reinvented; seeds locked

MET. All 16 existing fixture shapes reproduced label-for-label and edge-for-edge.
`InlineLcg` constants and all five `RANDOM_FIXTURES` seeds / counts /
topologies / names byte-identical to `closure_equivalence_tests.rs:373-416`.
`cyclic_chain` and the tangle constants byte-identical. Three shapes added, none
of the 16 altered.

### Sec 7 - locks

| Lock | Measured | Result |
|---|---|---|
| `graph.rs` byte-unchanged | `git diff 2708425 HEAD -- graph.rs` empty | HOLDS |
| `cycle_groups.rs` byte-unchanged vs `7c94af3` | `git diff 7c94af3 HEAD` empty | HOLDS |
| `closure_equivalence_tests.rs` byte-unchanged | `git diff` empty | HOLDS (IV-10) |
| `cycle_groups_test.rs` byte-unchanged | `git diff` empty | HOLDS |
| `lib.rs` and `Cargo.toml` byte-unchanged | `git diff` empty | HOLDS, zero new deps |
| R1/R2 hygiene, every fn < 100 lines, file < 1000 | `hygiene_check.py` zero R1/R2, zero R3 in the new file; 833 lines; longest fn 82 | HOLDS |
| UTF-8 mojibake | `encoding_guard.py --all` and `--staged` both silent | HOLDS |
| touch-set | six in-set paths, nothing else | HOLDS |
| fixture path in neither commit | path-scoped `git log` empty | HOLDS |
| no `#[cfg(test)]` copy of the old algorithm in production | grep finds the old algorithm only in `graph.rs` and the test oracle | HOLDS (no dual representation) |

## 3. Workspace-wide acceptance pass

| Gate | Command | Reviewer-measured result |
|---|---|---|
| baseline | `python tools/testrun.py --baseline` | `usage error: missing value for --baseline` (exit 2) - ledger F9, recorded once, NOT a gate failure, NOT worked around |
| full workspace (what `--baseline` wraps) | `python tools/testrun.py --distill --timeout 3600` | `1169 passed / 0 failed / 4 ignored / 88 suites`, exit 0, capture `target/testrun-capture.txt` 2069 lines |
| tool unit suites | `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 184 tests in 39.029s` - `OK` |
| encoding, all tracked content | `python tools/encoding_guard.py --all` | no output |
| encoding, staged | `python tools/encoding_guard.py --staged` | no output |
| hygiene | `python tools/hygiene_check.py` | zero R1/R2; every printed line a legacy-file R3 warn |
| UI runtime smoke | `python tools/ui_smoke.py` | REFUSED - verbatim: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` |
| load-perf regression | `python tools/bench_compare.py` | REFUSED - verbatim: identical text |
| graph crate build | `python tools/distill.py -- cargo build -p ariadne-graph` | `ok (exit 0, 0 warnings emitted)`, capture `target/distill-capture.txt` |

`ui_smoke` and `bench_compare` were each attempted independently, once, and
NEITHER was routed through `distill.py` to evade the denial. Both are ledger
M23.5 F10; the refusal is recorded as standing because T2-B is test-only and
touches no production perf surface. The residual risk is unchanged from T2-A
and must be closed before T3.2 re-pins `payload_tail_bench.rs`.

Also refused with the identical catch-all boilerplate, each attempted separately
and each replaced by a form that ran: `;`-chained git invocations,
`git ls-files --eol <path>`, `git status --porcelain -- <path>`. Third instance
of the recorded allowlist trap. Substitutes that ran: `git diff --numstat HEAD --
<path>` and `git log --oneline <base>..HEAD -- <path>`.

## 4. Explicitly NOT run

- The two mutation re-executions (Red A, Red B). The reviewer write grant is
  confined to `tasks/**` and `.kilo/plans/**`; `crates/**` is read-only for this
  role, so mutating production to reproduce a Red is impossible, not skipped.
  Both were instead adjudicated by exact-location, exact-message, exact-value
  causal proof plus an exhaustive proof that each reported failure set is
  complete (verdict section 5).
- No phase-3 or phase-4 work was demanded or credited. No perf measurement of
  `detect_cycle_groups` exists in this review (F10).

## 5. Fixture-path post-run state

`python tools/checkpoint_guard.py status` at session start AND after my full
workspace test run: DIRTY FILES `(none)`, UNTRACKED FILES `(none)`.
`tests/fixtures/basic/artifacts/includes/report.json` is absent from the
numstat dirty section in both states; `git diff --numstat HEAD -- <path>`
returns no rows in both; `git log --oneline 7c94af3..HEAD -- <path>` returns
nothing. The only output on that path is git's autocrlf warning
(`LF will be replaced by CRLF the next time Git touches it`), which is exactly
T2-A finding 9's `core.autocrlf=true` plus unpinned-path mechanism and produces
stat-cache churn, not content dirt. The implementer's report that no restoration
was needed is CONFIRMED. F12's remaining action (pin the path in
`.gitattributes`, or make the artifact writer emit LF) is still open before
T4.1.

## 6. Worktree state left behind

HEAD `b950963`, branch `tarjan-s-scc`, DIRTY FILES `(none)`, UNTRACKED FILES
`(none)`. Worktree census unchanged (8 worktrees; this one `ahead=7 behind=6
dirty=1` - the `dirty=1` count is the autocrlf stat-cache entry above, not a
content change). This review added two untracked files under `tasks/` by design:
`tasks/m235-t2b-verdict.md` and this addendum. They are mine to land, not the
implementer's; nothing else in the tree was modified.
