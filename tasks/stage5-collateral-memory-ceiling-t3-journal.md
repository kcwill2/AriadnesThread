# M28 T3 journal - profile-lever rider (ui_smoke release flag)

Handoff: `tasks/stage5-collateral-memory-ceiling-t3-handoff.md`
(hop id `t3-profile-lever-rider`). Start HEAD: `5169483` (verified via
`python tools/distill.py -- git rev-parse --short HEAD`).

## Initial state (verified, not redone)

- Working tree: M28 planning/handoff files + unrelated docs untracked,
  `tasks/milestone_ledger.md` and one plan modified - all OUTSIDE the
  touch-set; none staged, none touched by this hop. `commit_scoped.py`
  with the four declared globs is the only path to history.
- Touch-set files clean at start: `tools/ui_smoke.py` (456 lines),
  `tools/ui_smoke_tests.py` (338 lines), `AGENTS.md` (release-build bullet
  already landed at `5b517e6`; smoke line at Testing section lacks the
  release form).
- DP-1 boundary honored throughout: this rider ships the opt-in `--release`
  smoke route ONLY. It satisfies no I7 clause and no G-C/G-H gate; the
  default (debug) route stays byte-identical in behavior so every existing
  workflow (PR-2/bench baseline regime) is unchanged.

## Plan

- Red: profile-plumbing tests in `tools/ui_smoke_tests.py` (default-argv
  characterization floor + `--release` build-argv, exe-resolution,
  `--no-build`, `--exe`/`--target-dir` precedence), Red captured via the
  distilling entry point.
- Green: minimal wiring in `tools/ui_smoke.py` (`--release` flag,
  `build_binary` profile arg, `resolve_exe_path` profile default, usage
  docstring). No assertion weakened.
- Refactor: docstring + ONE dated `AGENTS.md` line matching behavior
  exactly; hygiene gates.

## Red

- Six tests added in `tools/ui_smoke_tests.py` class
  `ReleaseProfilePlumbingTests`: default-build-argv characterization floor
  (green BEFORE the change, locking today's `cargo build --bin tool` +
  target/debug), `--release` build-argv, `--release` exe resolution,
  `--release --no-build` (release resolution + zero cargo call), `--exe`
  precedence over `--release` + `--target-dir`, `--target-dir` precedence
  over the release default. All offline: `subprocess.run` faked via
  `unittest.mock.patch.object`; no boot, no cargo (handoff: pure flag
  plumbing).
- Gate 1 literal `python tools/distill.py --kind generic -- python -m
  unittest ui_smoke_tests` -> `### generic FAIL (exit 1)`,
  `ERROR: ui_smoke_tests (unittest.loader._FailedTest.ui_smoke_tests)`:
  distill.py pins the child cwd to REPO_ROOT (`distill.py:149`), so the
  handoff's "tools dir as cwd" is not expressible through the distiller.
  DEVIATION (recorded): substituted the proven sibling form from M27 T1b
  handoff (:115) - `python tools/distill.py --kind generic -- python -m
  unittest discover -s tools -p "ui_smoke_tests.py"` - identical suite,
  identical sys.path semantics as CI discover, run from repo root.
- Red capture: `### generic FAIL (exit 1)`; failing list = exactly the 5
  new `--release` tests (ERROR `ui_smoke.UsageError: unrecognized
  arguments: --release` - the strict-parser failure the handoff predicted
  as the honest behavior-red); the characterization floor was green.
  `Ran 29 tests in 4.647s` (23 prior + 6 new). Capture:
  `target/distill-capture.txt` (83 lines).

## Green

- `tools/ui_smoke.py` minimal wiring: `--release` store_true flag placed
  after `--no-build` (temperature-neutral, outside the cold/warm group);
  `--target-dir` default `None` sentinel so precedence is honest
  (`--exe` > `--target-dir` > profile default); `cargo_build_argv(release)`
  helper (single source of truth so the build-failure message names the
  command actually run; default-profile join is byte-identical to the old
  literal message); `build_binary(repo_root, release=False)` - the default
  keeps the old single-arg call form valid, which is what lets the
  characterization floor pin unchanged behavior across both tree states;
  `resolve_exe_path` picks `target/release` iff `--release` and no
  explicit target; usage docstring + help updated. No assertion weakened;
  default route behavior byte-identical (argv, target dir, message text).
- Gate 1 green: `### generic PASS (exit 0)`, `Ran 29 tests in 4.550s` /
  `OK`, exit 0. All 23 pre-existing cases UNEDITED and green. Capture:
  `target/distill-capture.txt` (5 lines).
- Gate 2: `python tools/distill.py --kind generic -- python
  tools/ui_smoke.py --help` -> `### generic PASS (exit 0)`; usage line
  shows `[--release]` and the help lists it. Capture:
  `target/distill-capture.txt` (20 lines).

## Refactor

- `AGENTS.md` Testing section: ONE dated line next to the smoke command
  documenting the release form (opt-in; default stays debug; precedence
  chain) - completes the `5b517e6` Build-section bullet without
  duplicating it. Pure ASCII, single line.
- Docstring/help claims checked against plumbing: nothing claims more than
  the flag plumbing does; the live release-boot witness is explicitly left
  to the reviewer hop (shape I8 honest ceiling).
- No behavior touched after the green gate-1 run; file at 480 lines,
  functions < 100.

## Gate log

1. Gate 1 (Red then Green), substitute form (deviation above):
   Red `### generic FAIL (exit 1)` + 5-name failing list +
   `Ran 29 tests in 4.647s` / `FAILED (failures=5)` (capture 83 lines);
   Green `### generic PASS (exit 0)`, `Ran 29 tests in 4.550s` / `OK`
   (capture 5 lines). MUST-lines from the GREEN-tree run only (row-65
   erratum).
2. Gate 2 `python tools/distill.py --kind generic -- python
   tools/ui_smoke.py --help`: `### generic PASS (exit 0)`; usage prints
   `[--release]`, help lists `--release` (capture 20 lines).
3. Gate 3 `python tools/distill.py --kind generic -- python
   tools/hygiene_check.py`: `### generic PASS (exit 0)`; all emitted rows
   are pre-existing R3 warn-only sites (capture 152 lines); grep of the
   capture finds ZERO rows for `tools/ui_smoke.py`,
   `tools/ui_smoke_tests.py`, `AGENTS.md`. Zero R1/R2.
4. Gate 4 staging `python tools/distill.py -- git add -- <four touch-set
   paths>` (exit 0; autocrlf notices are the repo's standing CRLF eol
   convention, blobs stored as usual) then `python tools/distill.py
   --kind generic -- python tools/encoding_guard.py --staged`:
   `### generic PASS (exit 0)`, silent (capture 0 lines).
5. Gate 5 commit via
   `python tools/commit_scoped.py --scope tools/ui_smoke.py --scope
   tools/ui_smoke_tests.py --scope AGENTS.md --scope
   tasks/stage5-collateral-memory-ceiling-t3-journal.md -m "feat(tools):
   ui_smoke --release launch route (M28 T3 profile-lever rider)"`.
   Landing hash recorded in the post-commit section below.

## Scope discipline (self-check before commit)

- Edited ONLY: `tools/ui_smoke.py`, `tools/ui_smoke_tests.py`,
  `AGENTS.md` (one dated line), this journal.
- NOT touched: `Cargo.toml` (T5 stanza untouched - no edit made, so
  verify-absent-edit is trivially met), `crates/**`, `tests/**`,
  `tools/bench_compare.py`, `bench_baseline.json`,
  `tasks/milestone_ledger.md`, `EXTENSIONS.md`/README, fixtures. The
  foreign dirty paths seen at start (M28 planning docs, ledger, unrelated
  plans) were never staged (explicit `git add -- <paths>`, never
  `git add .`) and are excluded from the commit by `commit --only`.
- Rider boundary restated: no I7 clause satisfied here; the default smoke
  route is byte-identical behavior; the live `--release` boot witness and
  the row-67 discharge relay belong to the reviewer/planner hops.

## Post-commit (added after gate 5 lands)

- Landing commit: `9385acf` "feat(tools): ui_smoke --release launch route
  (M28 T3 profile-lever rider)". `commit_scoped.py` numstat shows exactly
  the four touch-set paths (AGENTS.md 1+, journal 140+, ui_smoke.py
  +32/-8, ui_smoke_tests.py 90+); `git diff --name-only 5169483..HEAD` =
  the touch-set exactly. Foreign dirty paths (M28 planning docs, ledger,
  unrelated plans) verified still dirty/untracked, never committed.
- Census anchor preservation: zero `crates/**` paths in the range diff,
  therefore zero cargo-test delta against `1235 passed / 0 failed /
  6 ignored / 88 suites @ bd4fb1a` - the reviewer census proves the
  equality; here it follows from diff scope.
- Frozen locks by diff scope: `Cargo.toml`, `crates/**`, `tests/**`,
  fixtures/goldens, `tools/bench_compare.py`, `bench_baseline.json`,
  `tasks/milestone_ledger.md` all zero-diff.
- Landed-tree witness: gate 1 substitute re-run on `9385acf` ->
  `### generic PASS (exit 0)`, `Ran 29 tests in 4.575s` / `OK`.
- Second commit (journal finalization only): this addendum lands as a
  journal-scoped `commit_scoped.py` commit (precedent `4900e04`); its
  hash is the child of `9385acf` in `git log`.
- Deliberately NOT done (reviewer/planner obligations): live
  `ui_smoke --release` boot witness, default-route live smoke re-run,
  full tools discover, census run, `encoding_guard --all`, ledger row-67
  discharge relay (handoff "REVIEWER hop obligations").
