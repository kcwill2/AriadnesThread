# M23.5 T4.1 journal -- Phase 4 integration closeout

Contract: `tasks/m235-t4-handoff.md`. Base `dcd681b`, branch `tarjan-s-scc`,
worktree `.kilo/worktrees/tarjan-s-scc`. This unit changes no production
behaviour: F12 config pin, F13 identifier renames, verification + records.

## Preflight (gate 1) -- `python tools/checkpoint_guard.py status`

- HEAD `dcd681b` on `tarjan-s-scc`; census lists 8 worktrees; this one
  ahead=32 behind=17 dirty=6.
- Tree vs handoff sec2, TWO deviations, TREE WINS (recorded as documentation
  defects against the handoff, sec5/sec2 rows):
  1. Handoff says "the ONLY dirty path is the F12 fixture". Actual dirty set
     also holds `tasks/milestone_ledger.md` (+121/-22) and
     `tasks/ledger_archive.md` (+62/-1) -- planner bookkeeping never landed
     because planner/reviewer roles hold read-only git. Same class as sec3.7.
  2. Handoff lists two untracked records; actual list also holds
     `tasks/m235-t4-handoff.md` itself. Same class.
- Plan-file determination (sec3.4, listed as NOT-verified-for-me):
  `git ls-files --error-unmatch .kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md`
  via distill -> exit 0, path echoed. **The plan file is TRACKED.** No landing
  needed; cited as the contract of record. Ledger row (line 80) said
  UNVERIFIED; this row discharges it.

## Red evidence (sec5, both captured before any edit)

1. F12: `git status --porcelain -- tests/fixtures/basic/artifacts/includes/report.json`
   (via `distill.py`) prints ` M tests/fixtures/basic/artifacts/includes/report.json`;
   `git diff -- <same path>` returns NO hunks (only the PowerShell-side CRLF
   warning line). Stat-entry churn, not content drift. Signature #1 CONFIRMED.
2. F13: grep `legacy_elements|grouped_elements|the_parity_comparison` over
   `crates/ui/tests/ui_test/export_order.rs` -> exactly 5 hits at lines 298,
   311, 318, 325, 326. Signature #2 CONFIRMED.
   NOTE: `git grep` via `distill.py` was REFUSED by the catch-all rule
   (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`);
   the identical pattern was run with the dedicated Grep tool instead. Refusal
   recorded per discipline; not routed around for the gate itself.

## Edits (sec3.1 + sec3.3, applied after Red)

- `.gitattributes`: added `tests/fixtures/**/artifacts/**/*.json text eol=lf`
  plus a 3-line rationale comment matching the file's existing per-pin comment
  style. DEVIATION (minor): handoff sec3.1 says "Add ONE line"; the comment is
  an addition beyond the literal one. Justification: both existing pins carry
  rationale comments ("Same rationale ..."), and the contract itself says
  "keeping the file's existing comment style". Config-only; zero code.
- `crates/ui/tests/ui_test/export_order.rs` (F13, identifiers only):
  - test `the_parity_comparison_detects_a_cycle_classification_difference`
    -> `the_non_vacuity_witness_detects_a_cycle_classification_difference`
    (matches its own doc: it is the non-vacuity witness for the frozen lock;
    no longer claims to be a "parity comparison"),
  - `legacy_elements` -> `empty_currency_elements` (the `&[]` call at :311),
  - `grouped_elements` -> `populated_currency_elements` (the `&groups` call at :318).
  - No assertion, no argument, no expected value, no payload byte touched.
    `legacy_json`/`grouped_json` locals and both `expect` messages kept
    verbatim; the `assert_ne!` is byte-identical.
- Doc lines 366/372 mention historical "parity lock" of unit T3-D2 as prose
  history; NOT identifiers, deliberately untouched.

## Gates

- gate 1 `checkpoint_guard.py status` -- done at preflight (see above).
- gate 2 `testrun.py --distill --timeout 3600` (run 1, pre-normalization):
  `1180 passed / 0 failed / 4 ignored / 88 suites`, exit 0, capture
  `target/testrun-capture.txt` (2068 lines). Fixture still ` M` after this
  run -- pin alone insufficient, see F12 investigation below.
- gate 2 re-run (post-normalization, THE F12 proof run):
  `1180 passed / 0 failed / 4 ignored / 88 suites`, exit 0, capture 2067
  lines.
- gate 3 `testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test`:
  `186 passed / 0 failed / 0 ignored / 1 suites` -- count pin `186/0/0/1`
  survived the rename.
- gate 4 `hygiene_check.py`: R1 0 / R2 0; R3 warns all pre-existing; none in
  the two touched files. Re-run after ledger edits: same shape.
- gate 5 `encoding_guard.py --all`: silent. `--staged`: silent (see staging
  window note in execution-results).
- gate 6 bench via distill: `ok (exit 0, 0 warnings emitted)`; capture shows
  `running 1 test` -> `test result: ok. 1 passed; ... finished in 14.02s`;
  rows recorded in execution-results. Bench verifiably executed, not a
  silent zero-test run.
- gate 7 porcelain for the fixture after the proof run: EMPTY (0 lines).
  F12 CLOSED with proof.
- gate 8 `git diff --stat HEAD -- tests/golden`: EMPTY. `git status
  --porcelain -- tests`: EMPTY. Goldens byte-clean; re-pin unnecessary;
  `UPDATE_GOLDENS` never run in this session.
- theme_token_check `--check-added --base dcd681b`: silent.
- gate 9 commits: see execution-results table.

## F12 investigation (the pin alone did not close it -- measured mechanism)

After adding the pin and running the full suite, porcelain STILL reported
` M`. Measurements: `git ls-files --eol` -> `i/lf w/lf attr/text eol=lf`;
`git check-attr` -> text set / eol lf; `git hash-object <file>` ==
`git rev-parse HEAD:<file>` == `501b19bde2192befb6ba0e81c9de2ff3005202d8`
(content byte-equal to HEAD, raw == blob). `git update-index --refresh`
(REPORT-only, permitted) said `needs update` -- git's INDEX entry still
cached the pre-pin CRLF-checkout expectation. One `git checkout -- <path>`
(byte-identical re-smudge under the new pin; hash re-verified identical)
normalized the entry. The subsequent FULL workspace run (gate 2 re-run) left
the path porcelain-clean: THAT is the proof the contract demanded. Finding
against the handoff model: pin alone cannot clear pre-pin dirtied entries;
closure = pin + one normalization + full-run proof. Fixture in no commit.

## Refusal log (every refusal recorded; each gate attempted independently)

- `git grep ...` via distill: REFUSED (catch-all deny rule reported). Red
  gate 2 executed instead with the dedicated Grep tool, identical pattern,
  5 hits confirmed.
- `git diff --no-index --stat ...` via distill: REFUSED (probe only,
  non-gate; conclusion obtained otherwise).
- `python -c "import os,datetime; ..."` via distill: REFUSED (mtime probe).
- PowerShell `Get-Item ... | Select-Object | Format-Table`: REFUSED (same
  mtime probe; read-only but not allowlisted). Golden conclusion stands on
  gate 8 + empty `tests/` porcelain + default-env green golden suites.
- `wc` via distill: NOT a refusal -- `cannot run child: [WinError 2]`
  (binary absent on Windows).
- Permitted through distill (read-mostly): `git status`, `diff`, `diff HEAD`,
  `ls-files` (+`--eol`, `+/-v`), `check-attr`, `hash-object`, `rev-parse`,
  `ls-files --error-unmatch`, `show --numstat` (by commit_scoped), plus
  `git checkout -- <frozen fixture path>` and `git update-index --refresh`
  (both executed for the F12 normalization; scope: single fixture path).
- NOT attempted per contract: `python tools/ui_smoke.py`,
  `python tools/bench_compare.py`, and any `distill.py` routing of either.

## Determinism evidence, as-is (sec3.5: REPORT only, not built)

- Lock A: `crates/cli/tests/payload_tail_bench.rs:571`
  `export_bytes_are_identical_across_two_separate_processes` -- two separate
  processes, NOT ignored, uses its own small SCC fixture (2-header cycle).
- Lock B: `crates/ui/tests/ui_test/graph_api.rs:742`
  `include_elements_export_is_stable_across_two_calls_with_cycle_fixtures` --
  two calls in-process, NOT ignored, two-disjoint-group graph.
- The 520-header bench fixture (`HEADER_COUNT = 520`, `FANOUT_BASE = 10`,
  cycle-bearing by construction: +10 stride mod 520 closes after 52 steps) is
  exercised ONLY by `payload_tail_phase_timing`, which is `#[ignore]`d and
  asserts payload SIZES, not determinism. **Cycle-bearing export determinism
  IS locked inside the `1180/0/4/88` run (locks A+B); determinism on the
  520-header fixture specifically does NOT exist.** Plan T4.1's wording is
  unmet-as-literally-stated; adding that lock is a NEW TEST = new RGB unit,
  not closeout work. Per contract sec3.5: REPORTED, not built.

## Closing

- Post-run `checkpoint_guard.py status`: HEAD `6ec1a01`, DIRTY (none),
  UNTRACKED (none), this worktree dirty=0. Fixture clean. (Census note:
  the `main` worktree reports dirty=10 -- foreign, out of scope, report-only.)

- Commit 1 (config+code): `b3f33a9` -- paths `.gitattributes` (+4),
  `crates/ui/tests/ui_test/export_order.rs` (5/5). Fixture NOT in the commit.
- Commit 2 (`6ec1a01`, this record landing, `tasks/**`-only, no code): ledger rows for
  F12 CLOSED / F13 CLOSED / milestone / strays-row discharge, the T4.1
  Phase-4 detail sub-block, this journal, `tasks/m235-t4-execution-results.md`,
  `tasks/m235-t4-handoff.md`, plus the previously-untracked reviewer records
  `tasks/m235-t3e2-verdict.md` + `tasks/m235-t3e2-execution-results.md` and
  the planner's pending ledger/archive edits. Precedent: out-of-band
  `tasks/**`-only landing; standing template-clause recommendation stands
  (T3-E2 verdict finding 12) -- this landing is the Nth occurrence.
- Outstanding at close of this unit (all non-delegable): F10 allowlist +
  ui_smoke/bench_compare runs (BLOCKS closeout); kilo-memory mirror of this
  ledger (no role holds the memory tool -- operator); worktree census
  disposition (branch NOT merged to `main`); M23.5 walkthrough deck
  (walkthrough-commit role, requested separately).

## Deferred / outstanding (not closeable by this role) -- detail

- F10 (OPERATOR): `python tools/ui_smoke.py` and `python tools/bench_compare.py`
  refused by the reviewer bash allowlist. NOT attempted; NOT routed through
  `distill.py`. Until an operator amends the allowlist and runs both: zero
  runtime UI evidence, zero verified perf evidence after T3-D. M23.5 may not
  be described as CLOSED.
- kilo-memory mirror of `tasks/milestone_ledger.md` (OPERATOR): the ledger
  authority rule requires memory to move in the same commit as the ledger;
  no role in-workflow holds the memory tool. Ledger edits in this unit are
  therefore mirrored PENDING -- outstanding at close of this unit.
- Worktree census disposition (OPERATOR): branch `tarjan-s-scc` (this worktree)
  is NOT merged to `main`; mutating git is denied to every role. Must end
  merged, or abandoned with a ledger row.
- Walkthrough: `milestone-lifecycle` expects a committed M23.5 deck on this
  branch; none exists (`walkthroughs/**` holds only ui-theming-, m21-, m19-).
  Separate role (`walkthrough-commit`), requested separately. NOT authored here.
- Perf divergence left UNRECONCILED on purpose: `5.1` (`crates/cli/src/commands/ui.rs:103`
  calibration row), `5.4` (`payload_tail_bench.rs:41` bench header), `5.6`
  (`payload_tail_bench.rs:40` adjacent row / T3-D re-pinned stage-5 figure).
  Reconciling while F10 is open would manufacture perf evidence (T3-E2 verdict
  finding 6: do-not-re-open).
