# AR-26b Handoff Journal - Scheduler reparent to ariadne-artifact + m17 cone guard

Implementer lane (sole live lane). Contract: `tasks/ar-26b-handoff.md`
as amended at `69ab3f7`, bound by `tasks/ar-26b-shape-ruling.md`
(Option A, Rulings 1-4, precision notes P1-P3).

## Dispatch HEAD + pre-mutation baselines (re-measured, zero mutation yet)

Dispatch HEAD: `8eec456` (docs addendum of AR-26a2 landing `0d348ee`,
which is the frozen-lock diff base named by the dispatch message).

All lines quoted from own stdout, capture paths printed per run
(`target/testrun-capture.txt` is shared-worktree; per capture-race ruling
the digests below are from my run's stdout).

| suite | command | line (verbatim) |
| --- | --- | --- |
| m17 | `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test` | `8 passed / 0 failed / 0 ignored / 1 suites` |
| scheduler | `python tools/testrun.py --distill -p ariadne-scheduler` | `26 passed / 0 failed / 0 ignored / 2 suites` |
| worker-core | `python tools/testrun.py --distill -p ariadne-worker-core` | `8 passed / 0 failed / 0 ignored / 2 suites` |
| artifact | `python tools/testrun.py --distill -p ariadne-artifact` | `40 passed / 0 failed / 0 ignored / 2 suites` |
| report | `python tools/testrun.py --distill -p ariadne-report` | `7 passed / 0 failed / 0 ignored / 4 suites` |
| m23 | `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test` | `7 passed / 0 failed / 0 ignored / 1 suites` |

Baseline reconciliation vs the two authority pins:

- m17 `8/0/0/1` matches the contract pin; `8 -> 10` reconciliation applies.
- scheduler `26/0/0/2` and worker-core `8/0/0/2` match; conservation closes
  here with ZERO test edits (the chain's conservation equation).
- artifact `40/0/0/2` matches.
- report measured `7/0/0/4`: matches the orchestrator dispatch pin
  (`7/0/0/4`); the handoff's own baseline row still says `7/0/0/3`
  (line 47) - STALE HANDOFF LINE, measurement is the authority
  (report-only observation; the handoff is outside my touch-set, not fixed).
- m23 `7/0/0/1` matches the dispatch pin; file UNEDITED (proven by the
  frozen-lock diff gate below).

Single-consumption-site re-proof at dispatch HEAD (grep tool, not git grep
- git grep is allowlist-denied, logged): searching `crates/scheduler` for
`ariadne[-_]report` returns exactly TWO hits -
`Cargo.toml:16` (the manifest edge) and `src/incremental.rs:23` (the ONE
call site), tests included. Matches journal F5 of `tasks/ar-26-handoff-journal.md`.

## Gate 6 evidence - BEFORE side (pre-swap tree)

Command: `python tools/distill.py -- cargo tree -p ariadne-scheduler -e normal`
Digest: `ok (exit 0, 0 warnings emitted) -> capture: target\distill-capture.txt`

Exact-token grep of the capture (Grep tool, pattern `ariadne-report`):
ONE match, at package-name position:

```
line 71: [box-drawing] ariadne-report v0.1.0 (...crates\report)
```

BEFORE state confirmed: `ariadne-report` IN Tree(scheduler), dragging
`ariadne-artifact` (capture line 73) and the expected cone members
`ariadne-cluster` / `ariadne-transform` / `ariadne-validate`
(capture lines 76/96/113 and the `(*)` repeats). Per P2 these three are
EXPECTED members and are never asserted absent.

Checkpoint: baselines clean, no mutation yet. Next: m17 guard extension
(RED carrier) BEFORE any scheduler edit.
