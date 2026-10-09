# T3 Handoff - M28 stage5-collateral-memory-ceiling: profile-lever rider (ui_smoke release flag)

Hop id: `t3-profile-lever-rider`. The ledger's named small hop, finally
named. DP-1 (USER-RESOLVED 2026-10-05, binding): the release-launch profile
lever SHIPS as this rider handoff (`tools/` + `AGENTS.md`) - it may NEVER
substitute for the memory fix (the I7 both-profile-columns gate stands;
release-only survival is not sufficiency, shape sec. 1/R7/R8).

State this hop starts from (verified 2026-10-05, do not redo it):
- The doc-routes half of the ledger's OPEN profile-lever obligation landed
  APPROVED at `5b517e6` (M27 T5: `[profile.release] overflow-checks = true`
  in root `Cargo.toml` + release launch lines in `AGENTS.md` Build section,
  `EXTENSIONS.md`, the extension README). Verdict
  `tasks/tu-index-hang-throughput-t5-profile-lever-verdict.md` (:173-176)
  deliberately left `tools/ui_smoke.py` debug and named "ui_smoke profile"
  as an on-ledger decision. DP-1 answered that decision: ship the smoke-side
  release route as THIS hop.
- The remaining OPEN half (ledger deferred row 67, retargeted 2026-10-05):
  `ui_smoke.py` builds (`cargo build --bin tool`, :299-304) and launches
  `target/debug` (:95-96) with NO release form.

Dispatch precondition: NONE. Dispatchable immediately, IN PARALLEL with T0a
(touch-sets disjoint: this hop `tools/ui_smoke*` + `AGENTS.md` + journal;
T0a `crates/cli/tests` + evidence doc + journal; T0a's file does not exist
before its hop lands and vice versa - two handoffs touching one file are
never parallel, these touch none in common). No dependency on T0 tables or
ratification: DP constants never touch this hop.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. Shape `.kilo/plans/1791236997466-stage5-collateral-memory-ceiling-shape.md`
   sec. 3.5 (rider contract), sec. 2 I8 (rider invariant + the honest
   ceiling sentence).
3. `tasks/tu-index-hang-throughput-t5-profile-lever-verdict.md` :160-200
   (what landed, what was deliberately left, discharge readiness).
4. `tools/ui_smoke.py` (full flag surface :77-100; `build_binary` :299-304;
   `resolve_exe_path` :307-314; usage docstring :9-12).
5. `tools/ui_smoke_tests.py` (existing plumbing-test style to extend).
6. `AGENTS.md` Build + Testing sections (current release bullet landed at
   `5b517e6` - read it; do not duplicate it).

## Objective and invariants

Objective (shape 3.5 literal): `ui_smoke.py` gains the ratified release-flag
form - `--release`: builds `cargo build --release --bin tool` and launches
`target/release/tool.exe`, same cold/warm routes and the same route +
`Cache-Control: no-store` assertions. `AGENTS.md` gains the one dated line
that documents the smoke's release form (Build section already says normal
human launch uses the RELEASE build - your line completes it for the smoke
route; minimal surface, no duplication). A tools unit test asserts the flag
plumbing - the honest ceiling for a doc/flag change: a launch test cannot
check what humans type (shape I8), and the I8 check that MATTERS (release
boot end-to-end) is the reviewer's `ui_smoke --release` run.

Invariants:
- DEFAULT BEHAVIOR UNTOUCHED: no flag -> exact today's behavior (`cargo
  build --bin tool`, `target/debug`). This keeps the PR-2 / bench-baseline
  profile binding (M27 T5 I2 regime) intact for every existing workflow:
  the release route is opt-in, so nothing that runs today changes.
- `--exe` keeps overriding `--target-dir` keeps overriding the profile
  default (`--release` alone -> `target/release`; `--exe` wins over both).
- `--no-build --release` reads `target/release` without building (mirrors
  today's `--no-build` semantics).
- Zero Rust change; stage-5 bench columns unchanged; `bench_compare.py`,
  `bench_baseline.json` untouched; no new files inside
  `tests/fixtures/basic` / `tests/fixtures/expensive` (zero fixture demand;
  any fixture pressure routes through the documented frozen-set approval
  path, M27 T0b dated-row precedent).
- I1-I4 frozen floors: untouched (tools/docs diff); reviewer proves by diff
  scope. On APPROVED, this hop DISCHARGES ledger row 67 (planner archives
  the row verbatim per the archive rule).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/stage5-collateral-memory-ceiling-t3-journal.md` (checkpoint
  journal; always in-set; create alongside)
- `tools/ui_smoke.py` (the `--release` flag + build-command/target wiring +
  usage docstring line)
- `tools/ui_smoke_tests.py` (plumbing tests)
- `AGENTS.md` (ONE dated line, Testing section next to the `ui_smoke`
  command line, e.g.: release-form smoke invocation; pure ASCII)

You MUST NOT touch: `Cargo.toml` (T5's stanza landed - verify-absent-edit,
not verify-absent); `crates/**`; `tests/**`; `tools/bench_compare.py`,
`bench_compare.py`'s baseline, `tools/**` beyond the two named files;
`EXTENSIONS.md` / extension README (already release-pointed by T5);
`tasks/milestone_ledger.md`. No `git stash`, no branch change.

## Frozen locks

Kickoff sec. 5 table verbatim (goldens/fixtures; m00/m18 parity UNEDITED;
`progress.rs` byte-identical; stage-5 13-frame replay UNEDITED; boot
contract append-only + M26/M27 appended tests UNEDITED;
`CURRENT_ANALYSIS_VERSION` 2.1.0; `bench_baseline.json` operator-only;
census anchor `1235 passed / 0 failed / 6 ignored / 88 suites @ bd4fb1a` -
UNCHANGED after this hop: zero cargo tests added or removed; the census
line MUST print identically, mirroring T5's I3 discipline). Proving
commands: `git diff --name-only <start>..HEAD` == touch-set exactly;
census line equality.

## RGB protocol

1. Red: in `ui_smoke_tests.py` (pure flag plumbing - no boot in unit tests,
   mirror existing structure-test style):
   a. default build argv test (locks today's behavior BEFORE you change
      anything - lands green as the characterization floor; journal it),
   b. `--release` build-argv test: expected assertion failure text names
      the missing `--release` in the cargo argv (the flag does not exist
      yet - `SystemExit`/`UsageError` from the strict parser is the honest
      first failure; that IS behavior-red here: the strict parser raising
      UsageError on a flag that must exist is a testable behavior, not a
      compile failure),
   c. `--release` exe-resolution test (`target/release/tool(.exe)`),
   d. `--release --exe X` precedence test. Capture the failing list.
2. Green: minimal wiring: one `parser.add_argument("--release", ...)` in
   the temperature-neutral position, `build_binary(options...)` gains the
   profile arg, `resolve_exe_path` defaults to `target/release` iff
   `--release`. Update the usage docstring. NEVER weaken an assertion.
3. Refactor: docstring/`AGENTS.md` lines match behavior exactly (no claim
   exceeds the plumbing); functions < 100 (file 456 lines today - margin
   wide).

## Required tests

- Default build argv == `cargo build --bin tool` + target/debug resolution
  (unchanged-behavior floor).
- `--release`: `cargo build --release --bin tool` + `target/release` exe.
- `--release` with `--no-build`: no cargo call, `target/release` exe.
- `--exe` precedence over `--release` and over `--target-dir`.
- Existing `ui_smoke_tests` cases UNEDITED and green.

## Gates (exact commands, timeout 600000 ms each; all via distilling entry
points - subagent sessions cannot invoke gate commands directly: ledger row
60; proven route `python tools/distill.py -- python tools/ui_smoke.py`)

1. `python tools/distill.py --kind generic -- python -m unittest
   ui_smoke_tests` (tools dir as cwd, mirroring CI's discover invocation) -
   `Ran N ... OK`, N = prior + your cases, Red captured first, MUST lines
   from GREEN-tree runs only (row-65 erratum).
2. `python tools/distill.py --kind generic -- python tools/ui_smoke.py
   --help` - exit 0 (usage shows `--release`). NOTE: a live `ui_smoke` boot
   run is a workspace-wide gate and lives in the reviewer hop (below) - the
   `--help` form is the only smoke invocation in your budget.
3. `python tools/hygiene_check.py` - exit 0.
4. `python tools/encoding_guard.py --staged` - exit 0.
5. Commit: `python tools/commit_scoped.py --scope tools/ui_smoke.py --scope
   tools/ui_smoke_tests.py --scope AGENTS.md --scope
   tasks/stage5-collateral-memory-ceiling-t3-journal.md -m "feat(tools):
   ui_smoke --release launch route (M28 T3 profile-lever rider)"`.

Total implementer invocations: <= 6 counting red/green iterations.

REVIEWER hop obligations (NOT implementer gates): the I8 rider check itself -
`python tools/distill.py -- python tools/ui_smoke.py --release` (a real
release boot; `ui_smoke: PASS` line expected; first run pays the release
rebuild - generous timeout via the distill `--timeout` knob) AND
`python tools/distill.py -- python tools/ui_smoke.py` (default route still
PASSes - unchanged-behavior witness); workspace census
`python tools/testrun.py --distill` MUST print the anchor form `1235 passed
/ 0 failed / 6 ignored / 88 suites` UNCHANGED (zero cargo-test delta; the
`--baseline` FILE form is unrunnable - standing errata row 59: `--distill`
census substitute); full tools `python tools/distill.py --kind generic --
python -m unittest discover -s tools` (`Ran N ... OK` with N = 188 + your
new cases); `encoding_guard --all`; per-commit `git show` oldest-first vs
touch-set; on APPROVED relay to planner: ledger row 67 DISCHARGED (archived
verbatim) + milestone row update.

## Hygiene / ceilings at risk

`ui_smoke.py` 456 -> ~490 lines (ceiling 1000); functions < 100; flag name
literal `--release` (ratified form - do not rename). Pure ASCII; short
lines; `AGENTS.md` line dated and citing this hop.

## Discipline clauses

- UTF-8: Write/Edit tools only; pure ASCII; no PowerShell pipes
  (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; out-of-scope: report, do not fix.
- No bare high-volume commands; distilling entry points only; digest
  header + exit + capture path cited; zero raw pastes.
- Journal after each Green step. Context wall: STOP + disk-state inventory;
  successor: `python tools/checkpoint_guard.py resume --spec
  tasks/stage5-collateral-memory-ceiling-t3-handoff.md --scope tools --cmd …`.

## Acceptance (machine-checkable verdict lines)

- Gate 1 `Ran N ... OK` (Red-first captured); gate 2 exit 0 listing
  `--release`; gates 3-4 exit 0.
- `git show --stat <landing>` = touch-set exactly.
- Reviewer: BOTH smoke forms `ui_smoke: PASS` (default + `--release`),
  full tools discover OK, census UNCHANGED `1235 passed / 0 failed /
  6 ignored / 88 suites`.
- Rider boundary: this hop satisfies NO I7 clause and no G-C/G-H gate -
  if a closeout ever cites it as sufficiency, that is a finding (R8).
Reviewer gate: `review-gate` skill; commit-vs-touch-set enforces this file;
the reviewer owns the release-boot witness and the row-67 discharge relay.
