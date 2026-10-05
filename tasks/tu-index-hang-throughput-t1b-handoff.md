# T1b Handoff - M27 tu-index-hang-throughput: `bench_compare --bench boot-stages` gate mode

Milestone: `tu-index-hang-throughput` (M27, ledger row 2026-10-05). Single
module-dir handoff (`tools/`). Sizing law held: 1 module dir, 6 implementer
gate invocations (<= ~10), ~180-300 expected produced lines (<= ~1200).
Workspace-wide gates belong to the reviewer hop. PR-2 (operator
`python tools/bench_compare.py --update-baseline` on a VERIFIED-IDLE machine,
dated 2026-10-04 escalation row) gates the LIVE gate run and baseline ROWS -
you never run `--update-baseline`, never touch `bench_baseline.json`.

Why: T1a landed the release bench binary + labels; the regression gate must
be able to run it (`bench_compare --bench boot-stages`) with the same
median/compare protocol, while the DEFAULT `payload_tail_bench` stage-5 gate
stays behavior-identical (its baseline and semantics are M25-era frozen
behavior).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
   sec. 3.3 (gate-mode contract: default stays `payload_tail_bench`; new
   `DOCUMENTED_PHASES_BOOT_STAGES`; same per-machine class; baseline rows
   only via `--update-baseline`).
3. `tools/bench_compare.py` (FULL read - 350 lines: `BENCH_CARGO_ARGS`,
   `DOCUMENTED_PHASES`, `_parse_arguments`, `_handle_update`,
   `_handle_compare`, `main(argv, run_bench_impl)` test seam).
4. `tools/bench_compare_tests.py` (existing unit idiom; test seam usage).
5. `tasks/tu-index-hang-throughput-t1-measurement.md` (the eight labels
   T1a shipped - byte-exact source of your tuple).
6. `docs/specs/tool-output-distillation.md`.

## Objective and invariants

Extend `tools/bench_compare.py`:

1. New flag `--bench {payload-tail,boot-stages}` (default `payload-tail`;
   unknown value = usage error exit 2, same grammar as today).
2. `BENCH_CARGO_ARGS_BOOT_STAGES`:
   `["cargo", "test", "--release", "-p", "ariadne-pipeline", "--test",
   "boot_stages_bench", "boot_stages_release_timing", "--", "--ignored",
   "--nocapture", "--test-threads=1"]` (RELEASE profile explicit - the E3
   lesson: name the profile you measured). `--runs N` semantics unchanged.
3. `DOCUMENTED_PHASES_BOOT_STAGES` tuple = EXACTLY the eight labels from
   T1a's measurement doc (verify byte-exact; do not "fix" their wording).
   Selection of tuple/args/missing-row report keyed off `--bench`.
4. Baseline layout (shape sec. 3.3: "medians under the same per-machine
   class ... new labels = new phase keys"): boot-stages medians live in the
   SAME machine-class `phases` object of `bench_baseline.json` as additional
   keys. Because `--update-baseline` today REPLACES the class entry, change
   `_handle_update` to MERGE semantics: `phases = {**existing, **medians}`
   (documented behavior change, unit-tested both directions) so a boot-stages
   refresh preserves stage-5 keys and vice versa. `_handle_compare`
   unchanged in logic (boot labels missing from baseline fire today's
   `WARN untracked phase` - correct until the operator lands rows).
5. Verdict lines byte-identical (`bench compare: {runs} run(s), no
   regressions past 1.25x baseline`, `WARNING no baseline for machine
   class ...`, exit codes 0/1/2 grammar unchanged) - the stage-5 gate output
   must not shift by one character.
6. `tools/bench_compare_tests.py`: extend via the `run_bench_impl` seam
   (NO real cargo in unit tests): `--bench boot-stages` selects boot args +
   boot tuple; unknown bench value -> exit 2; missing boot row -> exit 2 +
   missing-row report; compare vs merged baseline dict (boot keys present ->
   normal ratio math; absent -> WARN); update MERGES (run A payload keys
   survive a later boot-stages update and vice versa); default `--bench`
   behavior byte-equal to pre-change tests (all existing tests green
   UNEDITED except where an update-test asserted replace semantics - the
   minimal such assertion may be amended WITH a comment naming this handoff;
   list every amended line in the journal).
7. `bench_baseline.json`: DO NOT TOUCH. The boot-stages rows land ONLY via
   the operator on a VERIFIED-IDLE machine (PR-2):
   `python tools/bench_compare.py --bench boot-stages --update-baseline` in
   the T1 landing window. The landing reviewer records which commit the
   operator's baseline commit is.

Invariants: `ui_smoke`, `testrun`, `distill` and every other tool byte-
identical; existing `DOCUMENTED_PHASES` (stage-5) tuple + semantics frozen;
exit-code grammar frozen.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/tu-index-hang-throughput-t1b-journal.md` (journal; always in-set)
- `tools/bench_compare.py` (flag, boot args const, boot tuple, dispatch,
  merge-update)
- `tools/bench_compare_tests.py` (extended tests; existing tests green
  UNEDITED except the merge-semantics lines above, each journal-listed)

You MUST NOT touch: `tools/bench_baseline.json` (operator-owned, PR-2);
any other `tools/*.py`; `crates/**`; `tests/**`; plan/kickoff docs. No
`git stash`, no branch change.

## RGB protocol
1. Red: extend the test file first; run it - `--bench` unknown-argument
   failures + merge assertions fail (today `--bench` is an
   `unknown argument` -> exit 2, so parse tests fail exactly there);
   capture the failing list.
2. Green: implement; re-run until OK.
3. Refactor: keep the two benches behind one dispatch table (no if-forest);
   re-run.

## Hygiene (review-enforced)
`bench_compare.py` stays < 1000 lines (350 now); functions < 100 lines; no
banned names; pure ASCII; `python tools/hygiene_check.py` exit 0.

## Frozen locks (each with its proving command)

| Lock | Proof |
|---|---|
| stage-5 default gate behavior frozen | all pre-existing bench_compare tests green UNEDITED (or journal-listed merge amendments) inside the scoped unittest run |
| `bench_baseline.json` untouched | absent from `git diff --stat <start-rev>..HEAD` |
| other tools byte-identical | diff lists touch-set only |
| mojibake + hygiene | `encoding_guard.py --staged` / `hygiene_check.py` exit 0 |

## Gates (exact commands, timeout 600000 ms each; scoped only)
Distilling entry points only (ledger row 58). Workspace-wide `unittest
discover` (full `tools/`) belongs to the reviewer hop.

1. Red: `python tools/distill.py -- python -m unittest discover -s tools -p
   "bench_compare_tests.py"` - failing list = the new tests; nonzero exit.
2. Green: same command - `OK`.
3. Refactor re-run (same command) only if behavior was touched.
4. `python tools/hygiene_check.py` - exit 0.
5. `python tools/encoding_guard.py --staged` - exit 0.
6. Commit: `python tools/distill.py -- python tools/commit_scoped.py --scope
   tools/bench_compare.py --scope tools/bench_compare_tests.py --scope
   tasks/tu-index-hang-throughput-t1b-journal.md -m "feat(tools):
   bench_compare --bench boot-stages gate mode + merge-baseline
   (M27 T1b)"`.

Total implementer gate invocations: 6 (<= 10).

## Acceptance (machine-checkable; implementer hop)
1. Scoped unittest run `OK` exit 0. 2. hygiene + encoding exit 0.
3. `git show --stat` = touch-set; `bench_baseline.json` absent from diff.
Reviewer hop (the LIVE gate - requires T1a landed + PR-2 baseline rows):
`python tools/distill.py -- python tools/bench_compare.py --bench
boot-stages` -> exit 0 with the median table; `python tools/distill.py --
python tools/bench_compare.py` (default stage-5) verdict line
byte-comparable to precedent (`bench compare: 3 run(s), no regressions past
1.25x baseline` or the recorded protocol-retry); full `python -m unittest
discover -s tools -p "*_tests.py"` -> `OK`; `encoding_guard --all`;
census. G2-2's stage-5-clean-vs-refreshed-baseline half is verified here.

## Discipline clauses
- UTF-8: Write/Edit only; pure ASCII.
- Touch only the touch-set; if T1a's labels don't match the bench output
  byte-for-byte: report (the tuple copies the MEASUREMENT DOC only after
  they match; drift here is a plan-literal violation in either direction).
- Journal checkpoint after each gate.
- Context wall: STOP + disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/tu-index-hang-throughput-t1b-handoff.md --scope ... --cmd ...`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red failing list. 3. Green `OK` line
+ capture path. 4. Every amended pre-existing test line (journal list) +
reason. 5. Out-of-scope observations. Never paste raw output.

## Reviewer-gate reference
Adjudicated via the `review-gate` skill. Extra checks: boot tuple byte-exact
vs T1a doc; default-bench code path diff-minimal; merge semantics both-way
unit-tested; no real cargo in unit tests; baseline file untouched by the
implementer commit; the LIVE gate + operator baseline commit cited in the
verdict.

## Sequencing note
After T1a (live gate needs the bench binary). T2 is parallel-safe with this
hop (disjoint trees: `tools/` vs `crates/lang-cpp`+`crates/index`+
`tests/integration`). PR-2 precondition applies to the reviewer-hop live
gate and the baseline-rows commit, NOT to your unit gates.
