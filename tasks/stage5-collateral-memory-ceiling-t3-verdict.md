# Verdict - M28 stage5-collateral-memory-ceiling T3 (profile-lever rider)

Hop: `t3-profile-lever-rider` - `tasks/stage5-collateral-memory-ceiling-t3-handoff.md`
Commits reviewed: `9385acf` (feat(tools): ui_smoke --release launch route),
`9f02645` (docs(tasks): journal post-commit scope proof). Start rev `5169483`.
Reviewed fresh at HEAD `9f02645` (main, ahead=0, behind=0, dirty=18 foreign
paths - checkpoint_guard, never from report).

## VERDICT: APPROVED

## Findings

1. (minor, planner-side, NOT an implementer violation) The handoff's
   reviewer-obligation line pins the full-tools command literal as
   `python -m unittest discover -s tools` with expected `N = 188 + your new
   cases`. The literal is under-constrained: unittest's default discovery
   pattern is `test*.py`, so the literal form discovers only 30 tests
   (`Ran 30 tests in 0.740s` / `OK` - fresh run). The expected value is only
   reachable in the CI form `-p "*_tests.py"`, which I ran: `Ran 194 tests
   in 20.462s` / `OK` (188 + 6 new - exact). The implementer touched no
   handoff line; the gap is in the handoff's command table. Disposition:
   planner fixes the literal at next handoff-spec touch; no action on this hop.
2. (informational) The handoff and the dispatch prompt cite the ledger
   profile-lever obligation as "deferred row 67"; in the current working
   ledger that row sits at file line 68 (planner's 2026-10-05 retargeting
   edit shifted numbering). Row content matches the cited obligation
   verbatim ("row stays OPEN until T3 gates APPROVED, then archived verbatim
   per the Archive rule"); the relay below names the row by content.

## Deviation dispositions (independently adjudicated)

- D1 (gate-1 substitute `discover -s tools -p "ui_smoke_tests.py"` for the
  literal `python -m unittest ui_smoke_tests`): ACCEPTED. Verified the
  factual premise myself: `tools/distill.py:149` pins the child `cwd=str(REPO_ROOT)`,
  so "tools dir as cwd" is not expressible through the distiller at all. The
  substitute is the CI discover form scoped to the one file - identical suite
  membership and sys.path semantics; I re-ran exactly the substituted
  command fresh: `Ran 29 tests in 4.603s` / `OK`, exit 0. Declared in journal
  before commit. Ships on its merits.
- D2 (`build_binary(repo_root, release=False)` default param): ACCEPTED.
  The default exists solely so the characterization floor can call the old
  single-arg form and pin unchanged default behavior across both tree
  states; the production call site `run_smoke` passes `options.release`
  explicitly (`tools/ui_smoke.py:419`), so the default can never steer a
  real run toward release, and the flag default is `store_true` False.
  Stronger floor than a required-param shape, not weaker.

## Per-commit scope verdicts

- `9385acf`: `git diff --numstat` = AGENTS.md +1/-0, journal +140/-0,
  `tools/ui_smoke.py` +32/-8, `tools/ui_smoke_tests.py` +90/-0. All four
  paths == the declared touch-set exactly. PASS.
- `9f02645`: journal-only (`tasks/stage5-collateral-memory-ceiling-t3-journal.md`),
  precedent `4900e04` journal-finalization commit. PASS.
- Range proof: `git diff --name-only 5169483..HEAD` returns exactly the four
  touch-set paths - no path outside the touch-set anywhere in the range.

## Hard checks (reviewer-run, not report-trusted)

- 23 pre-existing `ui_smoke_tests` cases byte-identical: RANGE NUMSTAT shows
  `+90/-0` for the file (pure insertion); per-commit `git show` confirms the
  only touched regions are the import block (2 added lines) and the new
  `ReleaseProfilePlumbingTests` class. PASS.
- Zero-diff locks: `Cargo.toml`, `crates/**` (incl. T0a territory
  `crates/cli/tests`), `tests/**`, `tools/bench_compare.py`,
  `bench_baseline.json`, `tasks/milestone_ledger.md`, M27 artifacts - all
  absent from the 4-path range diff. The dirty `milestone_ledger.md`
  (planner in-flight) was NEVER committed by these commits. PASS.
- `--release` is not the default: `store_true` default False;
  `--target-dir` `None` sentinel with precedence `--exe` > `--target-dir` >
  profile default; default resolution stays `target/debug` (locked by the
  characterization test); live default smoke PASS. PASS.
- Default-profile message byte-identity: `cargo_build_argv(False)` joins to
  `cargo build --bin tool`, reproducing the old literal error-message text.
  Verified by read. PASS.
- RED honesty: base-tree strict parser (`_StrictArgumentParser.error` ->
  `UsageError`, `tools/ui_smoke.py:77-79`) makes `UsageError: unrecognized
  arguments: --release` the predicted behavior-red; journal's Red capture
  matches (5 failing `--release` tests, floor green, `Ran 29`, exit 1).
- Shape 3.5 / I8 plan-literal: flag builds `cargo build --release --bin
  tool`, launches `target/release/tool(.exe)`, same routes + no-store
  assertions; tools unit test asserts plumbing; the one dated `AGENTS.md`
  Testing line completes (does not duplicate) the `5b517e6` Build-section
  bullet. Residual-only verified: the doc half was NOT redone (zero diff on
  Cargo.toml / EXTENSIONS.md / README). PASS.
- R8 boundary: journal + AGENTS.md line state opt-in-only, default stays
  debug; no I7/G-C/G-H sufficiency claim anywhere. PASS.

## Reviewer-obligation gate lines (verbatim)

- Gate 1 substitute (fresh): `### generic PASS (exit 0)` /
  `Ran 29 tests in 4.603s` / `OK`
- Gate 2 (fresh): `### generic PASS (exit 0)`; usage lists `[--release]`,
  help lists `--release  build and launch the release profile`
- Hygiene (fresh, distill): `### generic PASS (exit 0)`, capture 152 lines,
  all rows pre-existing R3 warn-only, ZERO rows for the touched paths
- Encoding (fresh, --all): `### generic PASS (exit 0)`, silent, capture 0 lines
- Full tools discover, CI form (fresh): `Ran 194 tests in 20.462s` / `OK`
- Census (fresh): `### cargo test PASS (exit 0)` [capture:
  `target/testrun-capture.txt`, 2175 lines] /
  `1235 passed / 0 failed / 6 ignored / 88 suites` - identical to anchor
  @ `bd4fb1a`, zero delta to reconcile
- `--baseline` file form: `usage error: missing value for --baseline`
  (standing errata row 59 confirmed; `--distill` substitute used per pre-approval)
- Default-route smoke (fresh): `### generic PASS (exit 0)` /
  `boot+route checks clean (cold fixture, attempt 1, 0.6s)` / `ui_smoke: PASS`
- Release-route smoke (fresh; I8 rider check): `### generic PASS (exit 0)` /
  `boot+route checks clean (cold fixture, attempt 1, 0.3s)` / `ui_smoke: PASS`
  - exit 0 on the release path requires `cargo build --release --bin tool`
  success AND `target/release/tool.exe` present (ui_smoke.py:418-428) AND
  that exe booted clean on all route assertions

## Not run (explicit)

- `bench_compare.py`: NOT RUN. Written exemption per dispatch terms: the
  range diff contains zero `crates/**` paths (stage-5 producers untouched),
  `tools/bench_compare.py` and `bench_baseline.json` are zero-diff, and DP-3
  keeps the `stage5-memory` gate mode dormant until T0b. No column this rider
  ships can move a bench row.
- `theme_token_check.py`: NOT APPLICABLE - zero Rust/UI/JS paths in the
  range diff; zero added hex is exempt-with-proof by the 4-path diff itself.
- `golden_guard.py`: not landed (handoff names the prose byte-check
  substitute - the diff-scope proofs above).

## Gate-execution / permission record

- All gates above ran through the proven distilling route
  (`python tools/distill.py -- ...`) with distinct `--capture` files.
- Direct entry-point execution PARTIALLY works in this session contrary to
  the dispatch premise: `python tools/hygiene_check.py` and
  `python tools/testrun.py --baseline` ran directly. Non-allowlisted shell
  probes were refused with the catch-all boilerplate (verbatim: `The user
  has specified a rule which prevents you from using this specific tool
  call. Here are some of the relevant rules
  {"permission":"bash","pattern":"*","action":"deny","source":"global"}`) -
  for a `Test-Path` probe and a `python -c` probe only; no GATE command was
  refused. No gate was skipped or substituted for permission reasons.

## Follow-up dispositions (relay to planner)

1. Ledger deferred-findings row "M28 T3 ... profile-lever landing" (working-
   ledger line 68): T3 gates APPROVED -> row DISCHARGED; archive verbatim
   per the Archive rule. Update the M28 milestone row: smoke half LANDED at
   `9385acf` (doc half already at `5b517e6`); obligation fully closed.
2. Handoff command-table fix per finding 1.
3. Pre-existing, unchanged: reviewer throwaway `tasks/v3-review-*.py` purge
   queue (ledger line 67 area) still open - operator duty, not this hop's.
