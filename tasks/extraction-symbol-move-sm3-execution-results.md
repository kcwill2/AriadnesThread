# SM-3 Execution Results - extraction-symbol-move (reviewer's fresh measurement)

Companion addendum to `tasks/extraction-symbol-move-sm3-verdict.md`, written
by the SM-3 reviewer hop 2026-10-24. Contract of record: the SM-3 handoff
INCLUDING AMENDMENT-1 (2026-10-24), which exempts the full-workspace census
for this landing (designed HARD CHAIN SM-3 -> SM-4) and prescribes the
substitution duties discharged below. Every number is from THIS reviewer's
own runs; nothing here is relayed.

## Ground truth (before any gate)

- `python tools/checkpoint_guard.py status`: HEAD `21cf779`, branch `main`;
  dirty (numstat vs HEAD): `.vsix` tolerated, `tasks/extraction-symbol-move-sm3-handoff.md` 209/0, `tasks/milestone_ledger.md` 5/2, `tasks/extraction-symbol-move-sm4-handoff.md` 62/0; untracked: `tasks/extraction-symbol-move-sm3-handoff-resume.md`. Worktree census: main + 6 pool/temp/branch worktrees, none holding milestone work.
- `git rev-parse HEAD` = `21cf779c599fb7b38fe935303cec20b27dd8d7e0`.
- `.git/logs/HEAD` (Read tool, 444 lines total): line 443 = duty-1
  `fba3c90... -> 790fe75900c041347d68a97a9bb7b36bfc3bc848 ... commit: docs(extraction-symbol-move): SM-2 APPROVED - land verdict + execution-results + ledger SM-2 record (census 1849 verified)`;
  line 444 = landing `790fe75900c041347d68a97a9bb7b36bfc3bc848 -> 21cf779c599fb7b38fe935303cec20b27dd8d7e0 ... commit: feat(validate): bind the plan id into the move hook at the strip call; m20 marked-retention rider + m19 optimize-deferral lock; hard-chained to SM-4 (extraction-symbol-move SM-3)`. File ENDS at 444 - no stray commits.

## Acceptance criterion 1: threading lands the milestone's core effect

PASS. Landing diff (`git show "21cf779" -- crates/validate/src/lib.rs`):
`strip_moved_symbols` gains `plan_id: &str`; strip call passes
`&migration.plan_id`; hook offer is
`rewriter.move_out_top_level_symbols(&content, names, plan_id)` - names AND
plan id forwarded observably at the source; m20's disk-level BEGIN-byte
equality (`plan=PLAN-001 symbols=Thing`) is the observable pin (SM-2 verdict
M1 CLOSED). Three doc lifts only, exactly as licensed; `collect_move_removals`
BODY, snapshot, rollback, ordering all byte-identical (SM-I4/SM-I7).

Machine-witness claims read directly: `crates/cli/src/optimize.rs:320`
(`PathBuf::from("generated").join(&unit.path)`), `optimize.rs:407`
(`apply_with(..., &extraction, ...)`), `crates/validate/src/lib.rs:362`
(`!rewrite.new_includes.contains(&unit.path)`). The deferral lock's claim
("the optimize verb marks NO original file") is TRUE under the frozen
derivation; the lock is a pure-addition strengthening with an unfreeze
tripwire; `git log -S uncomment_marked_regions -- m19` empty (withdrawn block
never reached history).

## Acceptance criterion 2: scoped gates, reviewer's own runs

All via `python tools/testrun.py --distill ...` (capture
`target/testrun-capture.txt` per run), canonical lines verbatim:

| gate | canonical line | exit |
| --- | --- | --- |
| `-p ariadne-validate` | `33 passed / 0 failed / 0 ignored / 5 suites` | 0 |
| `--test m19_optimize_characterization_test` | `5 passed / 0 failed / 0 ignored / 1 suites` | 0 |
| `--test m20_apply_migrate_characterization_test` | `6 passed / 0 failed / 0 ignored / 1 suites` | 0 |
| `-p ariadne-validate --test move_removal_transaction_test` | `2 passed / 0 failed / 0 ignored / 1 suites` | 0 |
| `--test m10_validation_test` | `5 passed / 0 failed / 0 ignored / 1 suites` | 0 |
| `--test m14_end_to_end_test` | `3 passed / 0 failed / 0 ignored / 1 suites` | 0 |
| `--test m17_plugin_seam_test` | `10 passed / 0 failed / 0 ignored / 1 suites` | 0 |
| `--test m18_cpp_plugin_parity_test` | `23 passed / 0 failed / 0 ignored / 1 suites` | 0 |
| `--test m23_symbol_isolation_test` | `7 passed / 0 failed / 0 ignored / 1 suites` | 0 |
| `-p ariadne-lang` | `47 passed / 0 failed / 1 ignored / 2 suites` | 0 |
| `-p ariadne-lang-cpp` | `215 passed / 0 failed / 0 ignored / 2 suites` | 0 |

Each target its own invocation (the cargo duplicate-`--test` keeps-last quirk
is real; my single-target samplings corroborate it). validate-tests zero-diff
proof: `git diff --numstat 790fe75..21cf779 -- crates/validate/tests` printed
NOTHING (the delegation proof is byte-identical and green THROUGH the SM-1
default forwarder - SM-I5/SM-I7 witnesses).

## Acceptance criterion 3: HARD-CHAIN single-red proof (substitution for the exempted census)

- `python tools/testrun.py --distill -p ariadne-cli --lib extraction_apply`:
  FAIL (exit 1), `12 passed / 1 failed / 0 ignored / 1 suites`. The single
  failure: `commands::ui::extraction_apply::tests::test_a_valid_token_applies_the_plan_end_to_end`
  panicking at `crates\cli\src\commands\ui\extraction_apply\tests.rs:300:9`
  with verbatim message: `the apply must not modify any file outside the approved dry-run's consumer_file set: modified=["...include\\thing.h", "...src\\thing.cpp", "...tests\\consumer.cpp"]`.
  Command ADAPTED from the dispatch example (`--test extraction_apply` names
  no target - the lock is a lib unit test; `--lib` + name filter is the
  sanctioned pass-through). The failing predicate (tests.rs:295-304,
  PRE-EXISTING UNEDITED) plus the modified set show the violation is limited
  to the marking of the approved plan's ORIGINAL file - the other two paths
  are the dry-run's own consumer set.
- Corroborator over the whole failing binary:
  `python tools/testrun.py --distill -p ariadne-cli --lib` - FAIL (exit 1),
  `170 passed / 1 failed / 0 ignored / 1 suites`. Exactly one red in the
  entire cli lib binary; matches the journal gate-7 stop witness
  (`259 passed / 1 failed / 0 ignored / 3 suites`).
- Conclusion: the landing carries EXACTLY the one workspace red AMENDMENT-1
  authorized. No additional reds in every suite this hop measured.
  Workspace-wide both-hands proof is discharged by SM-4 gate 6 at
  `1852/0/13/119` (carrier + ledger, quoted in the verdict).

## Acceptance criterion 4: hygiene, encoding, touch-set, frozen locks

- `python tools/hygiene_check.py`: printed ONLY warn-only R3 lines
  (single-letter/banned-name warns, ALL pre-existing; none in the three
  touched source regions); ZERO blocking R1/R2 lines -> pass (R3 flips to
  blocking only via a dated ledger row - none exists; tool exit code not
  echoed: chained `echo` refused by the allow list, recorded honestly).
- `python tools/encoding_guard --all` form used: `python tools/encoding_guard.py --all`:
  SILENT (the tool prints violations only) -> exit 0.
- ASCII: added regions pure ASCII (grep `[^\x00-\x7F]` found only the
  pre-existing legit em dash at m20:111, outside every hunk); no trailing
  whitespace in the m19 block or the amendment's NEW-bytes fence (grep ` +$`:
  none) - the byte-identity comparison between m19's committed block and the
  amendment's quoted block rests on full-text line-by-line match + these
  greps (inline byte-compare script impossible: `python -c` refused, edit
  grant limited to tasks/).
- Touch-set: `git diff --name-only 790fe75..21cf779` = EXACTLY the four
  declared paths; `git diff --numstat 790fe75..21cf779 -- crates/validate/tests crates/cli crates/transform crates/lang crates/lang-cpp crates/ui tests/fixtures tools` printed NOTHING; Cargo.lock absent. m19 hunks pure additions (17/0) inside the one named test; m20 deletions are EXACTLY the two declared message bytes, assertion expression line byte-identical.
- Planner's three uncommitted docs verified OUTSIDE the landing commit; their
  content matches their claimed roles (AMENDMENT-1 text, SM-4 carrier, ledger
  rows).

## Workspace-wide acceptance gates - EXPLICIT STATEMENT OF WHAT WAS NOT RUN

- `python -m unittest discover -s tools -p "*_tests.py"` (CI form): RUN,
  verbatim: `Ran 296 tests in 27.087s` then `OK`.
- Full-workspace census (`python tools/testrun.py --distill`) and
  `python tools/testrun.py --baseline`: NOT RUN by this hop. Reason, on the
  record: AMENDMENT-1 exempts the census from the SM-3 landing (designed
  HARD CHAIN; the milestone's own `--baseline` FILE form is superseded by the
  `--distill` substitute per the handoff, and fail-fast would halt at the
  authorized red - `--no-fail-fast` is not a sanctioned flag). The journal's
  gate-7 stop witness (`259 passed / 1 failed / 0 ignored / 3 suites`) is the
  record of reference; SM-4's gate 6 discharges both hands at
  `1852/0/13/119`.
- `python tools/ui_smoke.py`: NOT RUN (not an SM-3 hand gate - this landing
  touches no UI/asset bytes; smoke rides the milestone closeout and SM-5's
  hand).
- `python tools/bench_compare.py`: NOT re-run by the reviewer (the journal's
  gate-8 fresh recording stands as this hand's obligation: exit 1 with the
  standing, NOT-to-be-attributed PR-2 WARN
  `WARN get_or_build_include_graph clone (warm): 1.3 ms vs baseline 1.0 ms (+30%)`
  - never pin warning counts, rule unchanged).

## Refused commands this hop (allow-list specificity; all boilerplate catch-all denies)

Verbatim refusal text on each: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`

- `git status --porcelain` (plain `git status` and `git rev-parse HEAD` run
  fine), chained multi-command forms with `echo`/`;`, `python -c` (inline
  probe and scripted byte-compare), `git grep -c ... HEAD -- <path>`
  (substituted by checkpoint_guard numstat + commit name-only + `git log -S`
  proofs). Same class as the SM-2 verdict M5 allow-list deferred row -
  re-confirmed, no new orchestration finding.

## Reflog/SHA cross-check law

Every SHA claimed by the handoff, journal, amendment, ledger, and dispatch
was verified against `.git/logs/HEAD` and `git rev-parse HEAD` BEFORE any
gate was trusted (PHANTOM-NARRATION-1 countermeasure). All match; nothing
narrated here is unobserved.
