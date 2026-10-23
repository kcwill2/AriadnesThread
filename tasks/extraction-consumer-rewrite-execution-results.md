# extraction-consumer-rewrite - Execution Results (through CW-1 review hop)

Written by the CW-1 reviewer hop, 2026-10-23, against
`tasks/extraction-consumer-rewrite-cw1-handoff.md` acceptance and the
workspace-wide gates the reviewer owns. HEAD at gate time: `f73a9a1` on `main`
(landing `c916a26` + journal rider), start rev `2e5b86f`. All commands run
fresh in the reviewer session; nothing is inherited from the implementer's
journal without a re-run.

## Handoff acceptance criteria (CW-1)

| Criterion | Result | Evidence (verbatim) |
|---|---|---|
| lang-cpp green with 4 new tests (gate 4) | PASS | `python tools/testrun.py --distill -p ariadne-lang-cpp` -> `205 passed / 0 failed / 0 ignored / 2 suites`, exit 0 (201 baseline + 4 new) |
| Amended riders green (gate 5) | PASS | `python tools/testrun.py --distill -p ariadne-integration-tests --test m20_apply_migrate_characterization_test --test m19_optimize_characterization_test` -> `11 passed / 0 failed / 0 ignored / 2 suites`, exit 0 |
| m10/m14/m17/m18/m23 green UNEDITED (gate 6) | PASS | same-form command -> `48 passed / 0 failed / 0 ignored / 5 suites`, exit 0; none of the five files appears in `git diff --name-only 2e5b86f..HEAD` |
| Scala green UNEDITED (gate 7) | PASS | `python tools/testrun.py --distill -p ariadne-lang-scala` -> `257 passed / 0 failed / 0 ignored / 2 suites`, exit 0; `crates/lang-scala/**` in no diff |
| Full census +4 vs anchor (gate 8, `--distill` substitute per handoff) | PASS | `python tools/testrun.py --distill` -> `1834 passed / 0 failed / 13 ignored / 119 suites`, exit 0; anchor `1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b` -> exactly +4 passed, failed/ignored/suites held |
| Frozen lock pins green in census capture | PASS | capture (`target/testrun-capture.txt`, reviewer's own census run): line 1388 `test rewriter::tests::pin_neutral_consumer_rewrite_spec_default_is_none ... ok`; line 2639 `test test_transaction_apply_and_rollback ... ok` |
| `crates/cli/tests` zero-diff (producer proof for CW-2 attribution) | PASS | `git diff --numstat 2e5b86f..HEAD -- crates/cli/tests` -> empty |
| Touch-set containment | PASS | `git diff --name-only 2e5b86f..HEAD` -> exactly the 4 declared paths; Cargo.lock and all MUST-NOT-touch crates absent |
| Rider hunk containment (handoff reviewer-gate reference) | PASS | m19: hunk on the sanctioned `fn` rename line + one body hunk inside the named test; m20: single hunk inside the named test (base blobs read via git MCP, byte-safe) |
| Hygiene (gate 10) | PASS | `python tools/hygiene_check.py` -> R1/R2 silent; only pre-existing R3 warn-only rows, none in touched files |
| Encoding | PASS | `python tools/encoding_guard.py --all` -> silent (tool prints every violation then exits nonzero; pass form per its design) |

## Workspace-wide gates (reviewer-owned)

Canonical lines, quoted verbatim from the reviewer's own fresh runs:

- Full census: `1834 passed / 0 failed / 13 ignored / 119 suites`
  (`python tools/testrun.py --distill`, exit 0, capture
  `target/testrun-capture.txt`, 3179 lines)
- Tools suite: `Ran 296 tests in 26.475s` + `OK`
  (`python -m unittest discover -s tools -p "*_tests.py"`) - matches the
  ROUND-2 anchor `Ran 296` OK
- UI smoke: `ui_smoke: PASS` / `boot+route checks clean (cold fixture,
  attempt 1, 1.1s)` via `python tools/distill.py -- python tools/ui_smoke.py`
  (exit 0). The direct form `python tools/ui_smoke.py` was REFUSED by the
  sandbox with the boilerplate catch-all deny
  (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`);
  the distilling wrapper form ran unmodified.
- Timing gate (NOT exempt - this hand touched `crates/lang-cpp`):
  `bench compare: 3 run(s), no regressions past 1.25x baseline` via
  `python tools/distill.py -- python tools/bench_compare.py` (exit 0, capture
  `target/distill-capture.txt`, 17 lines). The `f20ad35` label fix holds: the
  gate ran to completion, 3-run median table emitted, no machine-class WARN
  rows surfaced this run beyond the reported no-regression verdict. Attribution
  for CW-2's oracle is clean: `crates/cli/tests` in no diff.

## Explicitly NOT run

- `python tools/testrun.py --baseline` (FILE form): NOT run. The CW-1 handoff
  gate 8 declares it superseded for this milestone ("the `--baseline` FILE
  form stays superseded by this `--distill` substitute"), and the dispatch for
  this hop assigned the `--distill` full census as the census of record. The
  `--distill` census above is the substituted evidence; no weaker evidence was
  used (identical count semantics, exit codes + capture proven).
- Exit-code echo gates (`python -c`, `cmd /c`, `echo`, `; Write-Output`
  chains) were refused by the sandbox allowlist; where a canonical line or
  digest header with `PASS (exit 0)` came from the tool itself, that is the
  exit evidence (see verdict finding 1 for the config item).

## Red evidence status

Red (4 unit + 2 integration failures) is evidenced by the journal's verbatim
capture lines, which match the committed assert messages word-for-word, and is
mechanically corroborated at HEAD (frozen trait default returns `None`;
reviewer could not re-run Red without mutating the tree, which a reviewer must
not do). Disposition: accepted - see verdict "Red capture adjudication".

## Milestone status

CW-1: gates green, ACCEPTED (see `tasks/extraction-consumer-rewrite-cw1-verdict.md`).
CW-2 (cli faithfulness oracle) and CW-3 (copy relay) pending; this file will
gain per-hand sections as each hop adjudicates.
