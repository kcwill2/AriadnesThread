# AR-26b Execution Results — Reviewer Acceptance Pass

Run 2026-10-11 at HEAD `ff526b6` (branch `shining-enthusiasm`), fresh
workspace state: `checkpoint_guard status` reported HEAD `ff526b6`, ZERO
dirty files vs HEAD, untracked = long-standing tasks/*.md + .kilo/plans/*
doc pattern only (no stray or placeholder files). Frozen base `0d348ee`;
dispatch HEAD `8eec456`.

## Ground truth + per-commit review (oldest first)

| commit | content | verdict-side note |
| --- | --- | --- |
| `3064b77` | journal only, +63 | pre-mutation baselines + BEFORE tree; no code |
| `8e08600` | Cargo.lock 1/1, scheduler/Cargo.toml 2/1, incremental.rs 1/1, journal 104/0, m17 79/2 | guard AND swap atomic in one commit |
| `ff526b6` | journal +75/-2 | addendum only |

Commit-vs-touch-set: `git diff --name-only "8eec456..HEAD"` returned
exactly the 5 declared paths — `Cargo.lock`,
`crates/scheduler/Cargo.toml`, `crates/scheduler/src/incremental.rs`,
`tasks/ar-26b-handoff-journal.md`,
`tests/integration/tests/m17_plugin_seam_test.rs`. Nothing outside.

Plan-literal checks: `incremental.rs:23` swap verbatim
(`ariadne_report::` → `ariadne_artifact::`); manifest comment line
`# AR-26b: artifact-store IO consumed directly (was via report).` verbatim;
`SCHEDULER_CONE_FORBIDDEN: [&str; 1] = ["ariadne-report"];` verbatim;
planted manifest text string-identical to the contract literal;
`GUARD_CORE_CRATES` 7→8 + `"crates/artifact"`; `TRANSITIVE_FORBIDDEN_DEPS`
6→7 + `"ariadne-artifact"`. m17 diff deletions = exactly the two const
size lines (+79/−2, matching the claimed stat).

## Gate lines (verbatim, reviewer own stdout)

- Gate 1 compile-green:
  `ok (exit 0, 31 warnings emitted) -> capture: ...target\distill-capture.txt`
  (`python tools/distill.py -- cargo check --workspace --all-targets`;
  capture grep: zero warning sources from scheduler/artifact; the journal
  quoted 11 for its run — warning count is cache-state variance, not a
  contract assertion).
- Gate 3 m17 (GREEN): `10 passed / 0 failed / 0 ignored / 1 suites`
- Gate 4 scheduler: `26 passed / 0 failed / 0 ignored / 2 suites`
- Gate 5 worker-core: `8 passed / 0 failed / 0 ignored / 2 suites`
- Gate 7 m00 pair: `4 passed / 0 failed / 0 ignored / 2 suites`
- m23 (dispatch-named, unedited lock): `7 passed / 0 failed / 0 ignored / 1 suites`
- Report suite (D2 corroboration): `7 passed / 0 failed / 0 ignored / 4 suites`
- Gate 6 re-run myself: `ok (exit 0, 0 warnings emitted)`; exact-token
  grep `ariadne-report` of my own capture = **0 matches** (BEFORE journal
  side: 1 match at capture line 71); `ariadne-artifact` present as a
  direct normal edge at capture line 3; `ariadne-cluster` line 52,
  `ariadne-transform` line 89, `ariadne-validate` line 106 present —
  described per P2, no absence assertion anywhere.
- Scheduler cone cleanliness: grep `ariadne[-_]report` over
  `crates/scheduler/` = 0 matches.
- Frozen locks: `git diff 0d348ee -- crates/report crates/artifact
  crates/cli crates/pipeline tests/golden crates/lang-cpp` → empty output.
  `git diff 0d348ee..HEAD --stat -- tests/integration` → m17 file only.
  `git diff --stat -- tests/golden tests/fixtures` → no content; the
  fixtures `report.json` line is an LF→CRLF *warning* only (foreign,
  chain-long, uncommitted by this lane).
- Hygiene: `python tools/hygiene_check.py` → R3 warn rows only
  (all pre-existing; zero rows in any touched file), ZERO blocking rows.
- Encoding: `python tools/encoding_guard.py --all` → silent (clean).

## Workspace-wide gates (reviewer acceptance pass)

- CHAIN CENSUS — `python tools/testrun.py --distill -- --workspace`:
  `1400 passed / 0 failed / 13 ignored / 105 suites`
  Expected `1400 / 0 / 13 / 105` — EXACT match. = 1398@26a2 + 2 new m17
  tests; suite count 105 reconfirmed (same test binary, no relocation).
  This is the CHAIN CENSUS for closeout. (The handoff's census-anchor
  wording `1398/0/0/105` omits the 13 ignored carried by every full
  workspace run — report-only wording note, verdict finding 3.)
- Tools suites — `python -m unittest discover -s tools -p "*_tests.py"`:
  `Ran 219 tests in 27.699s` / `OK` (219 OK as expected).

## Refused commands (verbatim) and substitutes

1. `git status --porcelain -- tests/golden; git status --porcelain` →
   refused: `The user has specified a rule which prevents you from using
   this specific tool call. Here are some of the relevant rules
   {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
2. `git status --porcelain` (standalone retry) → refused, identical text.
   Substitutes: `checkpoint_guard status` (dirty files: none; full
   untracked census) + `git diff --stat -- tests/golden tests/fixtures`
   (no content). Golden-dir cleanliness proven both ways.
3. `findstr /N /C:"DTO" ... tasks\ar-26a2-verdict.md` → refused, identical
   text (not on allowlist; NOT evidence of bash denial — all other gates
   ran). Substitute: grep tool on the verdict file.

No expected gate was skipped; every refusal is recorded above with its
substitute.

## Not run (explicit)

- `python tools/testrun.py --baseline` — treated as ERRATUM per dispatch
  restriction (census/gate ownership: the `--baseline` form is the
  orchestrator's own bounded run; dispatch required the `--distill
  -- --workspace` census instead, which was run fresh and matches).
- `python tools/ui_smoke.py` — out per dispatch (closeout).
- `python tools/bench_compare.py` — out per dispatch (closeout).
- `encoding_guard --staged` — not applicable to the reviewer lane (nothing
  staged); the commit-time run is the implementer's, and `--all` (the
  stronger whole-tree form) ran clean above.

## Acceptance-criterion ledger

| criterion (contract/verdict) | result | evidence line |
| --- | --- | --- |
| m17 8→10 after swap | MET | `10 passed / 0 failed / 0 ignored / 1 suites` |
| scheduler 26 conserved UNEDITED | MET | `26 passed / 0 failed / 0 ignored / 2 suites` |
| worker-core 8 conserved UNEDITED | MET | `8 passed / 0 failed / 0 ignored / 2 suites` |
| m00 pair UNEDITED | MET | `4 passed / 0 failed / 0 ignored / 2 suites` |
| m23 UNEDITED | MET | `7 passed / 0 failed / 0 ignored / 1 suites` |
| gate 6 tree: 0 exact-token report, artifact direct | MET | exit-0 tree; 0 grep matches; artifact capture line 3 |
| strict compile-green | MET | gate 1 `exit 0` |
| frozen locks EMPTY | MET | diff vs `0d348ee` empty |
| hygiene / encoding | MET | warn-only / silent |
| census 1400/0/13/105 | MET | verbatim line above |
| tools 219 OK | MET | `Ran 219 tests ... OK` |
| guard non-vacuity (forbidden set, planted test, exact-token) | MET | verdict adjudication 1 |
| RED trail authenticity + atomic guard+swap commit | MET | verdict adjudication 2 |
| touch-set exact, no stray files | MET | name-only diff; census |
