# SM-4 Execution Results - extraction-symbol-move (workspace-wide acceptance pass; reviewer hop, fresh runs, both-hand hard-chain discharge)

Reviewer hop for handoff SM-4 at HEAD `e717ba0866eda1d1f8117e8f6b427ee73171e175`
(`main`, ahead 0/behind 0; dirty = ONLY the tolerated
`extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`; untracked none -
`python tools/checkpoint_guard.py status` at boot). This addendum discharges,
for BOTH hard-chained hands - SM-3 landing `21cf779c599fb7b38fe935303cec20b27dd8d7e0`
and SM-4 landing `e717ba0866eda1d1f8117e8f6b427ee73171e175` - the
workspace-wide gates SM-3's verdict (finding M3) deferred per the SM-4 handoff's
AMENDMENT-1 CARRIER. Adjudication lives in
`tasks/extraction-symbol-move-sm4-verdict.md` (APPROVED); this file records
WHAT RAN, with the evidence line for each, and an explicit statement of what
was not run. Every line is from MY OWN fresh runs at the landing state -
nothing relayed. Anti-phantom: SHAs verified in `.git/logs/HEAD` (total 446
lines, ends at the landing) before any gate was trusted.

## The discharge census (both hands)

- `python tools/testrun.py --distill` - exit 0, capture
  `target/testrun-capture.txt` (3212 lines):
  `1852 passed / 0 failed / 13 ignored / 119 suites`
  Exactly the carrier's landing expectation. This is the SM-3+SM-4 BOTH-HANDS
  workspace-wide proof: the designed hard-chain red
  (`commands::ui::extraction_apply::tests::test_a_valid_token_applies_the_plan_end_to_end`)
  is green - legitimately widened per shape C5 (the planner-authorized
  :300-304 supersession is recorded in `tasks/extraction-symbol-move-sm3-handoff.md`
  AMENDMENT-1, lines 433-437, read by me), with the SM-I3 parity fold closing
  the R5 window at `e717ba0`.
- Delta reconciliation: anchor `1837/0/13/119 @ b201aa6` + SM-1 +2 + SM-2 +10
  + SM-3 +0 + SM-4 +3 (the declared `move_proof` trio) = 1852; suites 119
  held; ignored 13 held; zero failures.
- `--baseline` form: NOT RUNNABLE - refusal verbatim:
  `usage error: missing value for --baseline`
  (the FILE form stays superseded by the standing errata; the `--distill`
  substitute law governs - census above).

## Kickoff section 8 acceptance criteria, criterion by criterion

1. Fixture apply marks `include/thing.h` per grammar, one BEGIN/END pair
   bound to the plan id, regions verbatim-under-prefix: GREEN - witness: my
   fresh scoped cli `182 passed / 0 failed / 8 ignored / 8 suites` exit 0
   including `test_a_valid_token_applies_the_plan_end_to_end ... ok` (the
   C5.4 marker proof: balance, pair-count == hook-success (ONE), plan-id on
   every marker, byte-exact prefix-strip reconstruction, ASCII, inert
   re-offer) and m20's byte-equality pin (full census). m19/m20 green with
   byte-identical remainders inside the census; SM-3 surfaces zero-diff in
   my range (`git diff --numstat a33f620..e717ba0 -- crates/validate
   tests/integration ...` EMPTY).
2. Grown faithfulness oracle green through the UI applier, all C5 truth
   conditions: GREEN - the same named test green (conditions 1-5 wired),
   rollback condition 6 witnessed by validate `33 passed / 0 failed / 0
   ignored / 5 suites` exit 0 UNEDITED (`move_removal_transaction_test`
   inside).
3. Copy truth (C6 ui strings): NOT SM-4 SCOPE - SM-5's hand (ui). The
   reader-side W4 flip is green: `test_dry_run_reports_move_truth_on_the_cpp_fixture
   ... ok` and the honest-absent pin `pin_degraded_dry_run_is_honest_absent_and_never_cached
   ... ok` (zero-move degraded case: `move_removals: 0, copy_only: false` -
   reports zero, claims no copy-only promise, never cached; the fold cannot
   fire on that path).
4. Consumer-side zero regression: GREEN UNEDITED - m18 `23 passed / 0 failed
   / 0 ignored / 1 suites` exit 0; m00 `2 passed / 0 failed / 0 ignored / 1
   suites` exit 0; `crates/ui`, `crates/transform`, `crates/lang*`,
   fixtures, tools ZERO-diff (range proof above); Scala cone inside the
   census.
5. Rollback covers the original file: GREEN - validate suites above,
   byte-identical file (zero-diff proof).
6. Workspace gates exit 0 at this hand: ALL PASS - every line below.
   Commit-vs-touch-set EXACT (4 paths, verdict section).
7. ASCII throughout: `encoding_guard --all` exit 0, 0 lines; every hunk
   visually ASCII; marker asserts check `.is_ascii()` per line.

## Workspace-wide gate lines (verbatim, my own fresh runs)

| gate | command | result |
| --- | --- | --- |
| full census | `python tools/testrun.py --distill` | exit 0, `1852 passed / 0 failed / 13 ignored / 119 suites` |
| baseline form | `python tools/testrun.py --baseline` | refused: `usage error: missing value for --baseline` (superseded FILE form; `--distill` substitute law) |
| tools suites | `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 296 tests in 28.305s` / `OK` |
| ui_smoke (plain) | `python tools/ui_smoke.py` | REFUSED by catch-all: `{"permission":"bash","pattern":"*","action":"deny","source":"global"}` |
| ui_smoke (sanctioned) | `python tools/distill.py -- python tools/ui_smoke.py` | `### generic PASS (exit 0)`, `boot+route checks clean (cold fixture, attempt 1, 1.0s)`, `ui_smoke: PASS` |
| bench (fresh, both-hands stance) | `python tools/distill.py -- python tools/bench_compare.py` | `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline`; cold include-graph row `1.5` ms (no WARN - the implementer's transient `1.8 ms (+38%)` WARN did NOT reproduce; attribution: NOT code - the timed span calls zero-diff pipeline code, the bench harness never runs the SM-4 fold; verdict M1/attrib-section) |
| hygiene | `python tools/hygiene_check.py` + distiller exit witness | output warn-only R3; `### generic PASS (exit 0)`; capture 173 lines, ALL pre-existing, zero in authored files, zero R1/R2 |
| encoding | `python tools/encoding_guard.py --all` + distiller exit witness | silent; `### generic PASS (exit 0)`; 0 lines |
| theme drift | `python tools/theme_token_check.py --check-added --base a33f620` | silent (no hex added) |
| scoped cli | `python tools/testrun.py --distill -p ariadne-cli` | exit 0, `182 passed / 0 failed / 8 ignored / 8 suites` |
| validate | `python tools/testrun.py --distill -p ariadne-validate` | exit 0, `33 passed / 0 failed / 0 ignored / 5 suites` |
| m18 parity | `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` | exit 0, `23 passed / 0 failed / 0 ignored / 1 suites` |
| m00 parity | `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_facts_path_parity_test` | exit 0, `2 passed / 0 failed / 0 ignored / 1 suites` |

All gates ran SERIALLY (the D-e DRIFT law: concurrent testrun runs race the
shared capture file + cargo target lock); zero drift encountered.

## Refusals encountered (verbatim, per the attempt-each-gate law)

1. `python tools/ui_smoke.py` (plain form) - refused; boilerplate:
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
   Substituted the sanctioned distiller form: PASS (above). No gate evidence
   weakened: the smoke assertions ran.
2. `python tools/testrun.py --baseline` - tool-level refusal:
   `usage error: missing value for --baseline`; standing supersession;
   substitute ran the FULL census (stronger than the file form).
3. A chained multi-command probe (diff-empty + log + show with `;`/`echo`)
   was refused with the same boilerplate; I retried EVERY component
   individually and all components ran - no gate was skipped because of it.

## Explicitly NOT run (and why)

- `--baseline` FILE-form census: refused by the tool (standing errata); the
  `--distill` FULL census substitutes per the carrier law (ran, exit 0).
- Bench on the SM-3 state (`21cf779`): would require a checkout/worktree
  write - reviewer git is read-only; attribution was instead adjudicated
  structurally (measured span = zero-diff code) + by the two-hands
  different-row WARN pattern + the fresh exit-0 recording at the head.
- m19/m20/m10/m14/m17/m23 as INDIVIDUAL scoped invocations: covered green by
  the full-workspace census (0 failed) and the frozen-surface zero-diff
  range proofs; SM-3's hop ran them scoped with per-suite lines.
- `golden_guard.py` byte-locks: not part of this handoff's lock list (the
  prose byte-checks - `extraction_apply.rs` diff-EMPTY, fixtures zero-diff -
  ran instead, per the skill).

## State of record after this gate

- HEAD `e717ba0866eda1d1f8117e8f6b427ee73171e175` on `main`; reflog line 446
  (the file's last line); tree clean except tolerated `.vsix`.
- Census `1852 passed / 0 failed / 13 ignored / 119 suites` (green, exit 0) -
  the SM-5 dispatch baseline.
- Tools `Ran 296 tests` / `OK`; ui_smoke PASS; bench exit 0, no WARN
  (fresh, this hop); hygiene exit 0; encoding exit 0; theme check silent.
- Worktree census (bootstrap): main worktree + 6 pool/`.kilo` worktrees,
  none interfering; none carry this milestone's branches.
- Uncommitted by design: this hop's two artifacts (verdict + this addendum);
  SM-5 duty-1 must EXPECT FIRST the planner's ledger SM-4-record write, then
  land ledger + both artifacts (plus any SM-5 amendment) in one scoped docs
  commit; boot porcelain must match that set exactly.
- Hard-chain status: CLOSED. The generation-swap relay remains OPEN
  (record-only, still unexecuted - diff-empty re-proved); closeout appends
  its dated row.
