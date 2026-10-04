# T-B Handoff - ui-extraction-card-playbook: wire the propose-time target_header data into the feed

Milestone: `ui-extraction-card-playbook`. Second hand of the HARD CHAIN
T-A -> T-B -> T-C. REQUIRES T-A landed (the `target_files` /
`target_files_truncated` fields and `ProposedTargetFileDto` compile in
`ariadne_ui`); start only if HEAD contains the T-A commit.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. Shape `tasks/ui-extraction-card-playbook-shape.md` sections C2, P1, P11 +
   section 2 (propose-time vs apply-time: `target_header` IS a plan field at
   `crates/cluster/src/plan.rs:35-45`; DO NOT touch producers).
3. Kickoff `tasks/ui-extraction-card-playbook-kickoff.md` (invariant 1, the
   m00 parity lock row, concurrency note re `cpp-brace-depth-underflow`).
4. `crates/cli/src/commands/ui.rs` lines 447-496 (the boot-path planner +
   `EXTRACTION_PROPOSAL_MAX` cap, whose parity with `plan_extractions` is
   locked by `m00_facts_path_parity` - the planner itself stays untouched)
   and 503-537 (`extraction_proposal_dto` - the mapping you extend).
5. `crates/cli/src/commands/ui/builder_tests.rs` - the
   `OptimizationPlan`/`ProposedExtraction` construction idiom your new tests
   reuse (`plan_with` :9, `ProposedExtraction` literals with
   `symbol_name`/`symbol_id`/`target_header`).
6. `crates/ui/src/dto.rs` lines 179-198 + the T-A additions (field shapes you
   are filling).

## Objective and invariants

Inside `extraction_proposal_dto` (pure mapping, no planner, no I/O): group
`plan.proposed_extractions` by `target_header` into a
`BTreeMap<PathBuf, Vec<&ProposedExtraction>>` (path-asc determinism), sort
each group's symbol names by `symbol_id` (mirroring the existing
`symbol_names` sort at :509), cap per-file names at
`ariadne_ui::EXTRACTION_PROPOSAL_MAX_NAMES`, cap file rows at
`ariadne_ui::EXTRACTION_PROPOSAL_MAX_TARGET_FILES` (8) setting
`target_files_truncated` iff the group count exceeds the cap. `path` is the
lossy-rendered PathBuf. Extract the grouping into a small named pure helper
so `extraction_proposal_dto` stays under 100 lines (shape section 9).

Invariants preserved:
- P1 feed truth: emitted `target_files` grouping equals the plan's
  symbol->`target_header` mapping (path asc, symbol_id asc, caps + flag
  exact). Oracle = your new unit tests on constructed plans.
- P11: payload order / existing fields / cap behavior byte-faithful - only
  the two new fields change output.
- `m00_facts_path_parity` (payload planner == CLI plan output): UNEDITED and
  green - you map existing plans, the planner and `OptimizationPlan`
  serialization are untouched (`plans_plans.json` semantics unchanged).
- No new planner run, no new planner hook, no I/O in the DTO mapping.
- Existing ui.rs code paths (progress percentages, boot sequence) untouched.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/ui-extraction-card-playbook-tb-journal.md` (checkpoint journal;
  create alongside; always in-set)
- `crates/cli/src/commands/ui.rs` (the grouping helper + the two-field fill
  in `extraction_proposal_dto` + ONE `#[cfg(test)] mod target_files_tests;`
  declaration after the `boot_progress_tests` block at :786-790)
- `crates/cli/src/commands/ui/target_files_tests.rs` (NEW - the P1 oracle)
You MUST NOT touch: `crates/cluster/**`, `crates/lang-cpp/**`,
`crates/cli/src/lib.rs`, `crates/cli/src/commands/{plan,transform}.rs`,
`crates/cli/src/optimize.rs`, `crates/ui/**` (T-A landed it), the existing
`crates/cli/src/commands/ui/{tests,builder_tests,boot_progress*,
progress_contract_tests}.rs` files (byte-identical - import from `super`
instead), `tests/**` incl. `m00_facts_path_parity_test.rs`,
`tests/fixtures/**`, `tools/**`. No `git stash`, no branch change.

## Frozen locks

| Lock | Proof |
|---|---|
| `m00_facts_path_parity` green UNEDITED | scoped `-p ariadne-integration-tests --test m00_facts_path_parity_test` (see gate 4) + byte-identical in diff |
| Boot progress contract (`ui-progress-parity` oracle) | `crates/cli/src/commands/progress.rs` + pipeline byte-identical; `progress_contract_tests.rs` untouched, scoped ui-boot tests green in the `-p ariadne-cli` run |
| Existing ui.rs feed fields/caps/order | `builder_tests.rs` + `export_order.rs` green UNEDITED inside the scoped runs |
| `tests/golden/**` byte-locked (`UPDATE_GOLDENS=1` FORBIDDEN) | no producer serialization change; reviewer hop confirms absent from diff |
| Mojibake/UTF-8 gate | `python tools/encoding_guard.py --staged` exit 0 pre-commit |
| Hygiene (R1/R2) | `python tools/hygiene_check.py` exit 0 (ui.rs lands ~830 of 1000) |

## RGB protocol

1. Red (compiles + fails): author `target_files_tests.rs` FIRST (fields
   exist from T-A, so this compiles; the feed still emits `vec![]`, so the
   grouping assertions fail). Capture the failing list.
2. Green: implement the pure grouping helper + fill; re-run until green.
   NEVER weaken an assertion.
3. Refactor: name honestly (`target_file_groups` or similar), doc comment
   the caps + determinism choice (BTreeMap path-asc; symbol_id sort mirrors
   :509), re-run.

## Required tests (in `target_files_tests.rs`, reusing the `builder_tests.rs`
construction idiom; each < 40 lines)

1. `test_target_files_group_by_path_ascending` - one plan with extractions
   across 3 distinct `target_header` values (constructed out of path order);
   DTO `target_files` paths ascending, each file's `symbol_names` sorted by
   `symbol_id`, `symbol_count` = uncapped members.
2. `test_symbol_file_assignment_preserved` - each symbol name appears under
   exactly its own `target_header` (invariant 1 of the kickoff).
3. `test_per_file_name_cap_and_uncapped_count` - 15 symbols into one
   `target_header`: `symbol_names.len() == EXTRACTION_PROPOSAL_MAX_NAMES`,
   `symbol_count == 15` (mirrors the `symbol_names` cap semantics).
4. `test_file_row_cap_sets_truncated_flag` - 9 distinct `target_header`s:
   rows.len() == 8, `target_files_truncated == true`; 8 or fewer: full list,
   flag false (iff, both directions).
5. `test_existing_fields_unchanged_by_the_fill` - with one plan, the 11 old
   fields equal what they were pre-change for the same input (compare the
   fields you can pin: names, caps, delta, risk string), guarding P11.

## Hygiene (review-enforced)

`extraction_proposal_dto` + new helper each < 100 lines (helper target
~25); no banned names (`tmp`/`res`/`foo`/`bar`, single-letter locals);
ui.rs < 1000 (~830); pure ASCII.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. Red: `python tools/testrun.py --distill -p ariadne-cli` - digest +
   canonical line verbatim, exit 1, failing list = the P1 tests above.
2. Green: same command - `0 failed`, exit 0 (includes the untouched
   `builder_tests`/`progress_contract_tests`/`boot_progress_tests` modules).
3. Regression sanity on the consumer side: `python tools/testrun.py --distill
   -p ariadne-ui` - `0 failed` (no ui-side edit expected; proves you did not
   need to touch ui).
4. Parity lock: `python tools/testrun.py --distill -p
   ariadne-integration-tests --test m00_facts_path_parity_test` - exit 0,
   file byte-identical in the diff.
5. `python tools/hygiene_check.py` - exit 0.
6. `python tools/encoding_guard.py --staged` - exit 0 before committing.
7. Commit (single):
   `python tools/commit_scoped.py --scope crates/cli/src/commands/ui.rs
   --scope crates/cli/src/commands/ui/target_files_tests.rs --scope
   tasks/ui-extraction-card-playbook-tb-journal.md -m "feat(cli): feed
   per-symbol target_header plan into extraction proposal DTO
   (ui-extraction-card-playbook T-B)"` (never `git add .`).
Total implementer gate invocations: 7 (<= 10). Allowlist refusal route:
`python tools/distill.py -- <cmd>` (open deferred ledger row).

## Dependency edges (gate bookkeeping)

- Proves the FEED: gates 1-2 + 4 here (`target_files_tests.rs` + m00 parity).
  T-C's renderer tests do NOT prove the feed; T-B's scoped cli suite is the
  feed oracle. This hand's `ui_smoke` value is NIL (assets unchanged) -
  smoke runs at T-C.
- T-C's live playbook render (`ui_smoke` cold-boot from a real payload)
  requires THIS commit landed first.
- Concurrency: `cpp-brace-depth-underflow` (PLANNED, `lang-cpp
  tokenizer/parser`) is file-disjoint; if it lands between T-B and T-C
  review, plan/payload CONTENTS may shift (bug fix) - reconcile census
  totals, do not chase, and no test here pins payload bytes (kickoff
  concurrency section).

## Bench stance (reviewer hop, not your session)

T-B is a NON-ASSETS Rust change: the assets-only `bench_compare` exemption
does NOT auto-carry (kickoff section). Reviewer decision: scoped
`bench_compare` PASS on a quiet machine (stale-`tool.exe` deferred row:
close the process or use temp `CARGO_TARGET_DIR`), or the documented
exemption recording (a) empty `crates/pipeline/**` diff and (b) the O(symbols)
grouping on a boot path that already runs the full planner (shape risk 6).
The chosen branch is recorded in the verdict - implementer does not run it.

## Discipline clauses

- UTF-8: Write/Edit tools only, pure ASCII.
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Raw cargo / unscoped census / discover / ui_smoke / bench_compare in this
  session = reviewer blocker; distilling entry points only.
- Journal checkpoint after Green (module + commit + deviations).
- Context exhaustion: STOP, disk-state inventory; successor boots via
  `checkpoint_guard.py resume --spec
  tasks/ui-extraction-card-playbook-tb-handoff.md ...`.

## Acceptance (machine-checkable)

1. `-p ariadne-cli` -> `0 failed` including the 5 P1 tests; `m00` parity test
   exit 0 UNEDITED; `-p ariadne-ui` still `0 failed`.
2. `git show --stat` = exactly the touch-set; diff of ui.rs confined to the
   mapping region + the one `mod` line.
3. hygiene + encoding exits 0.

Reviewer hop (`review-gate`): commit-vs-touch-set; per-commit `git show`;
`git diff <start>..HEAD -- crates/cluster crates/lang-cpp crates/ui/assets
tests fixtures` EMPTY (producers + assets + m00 test untouched); scoped cli
re-run; bench decision recorded (see Bench stance).

## Report back

Commit hash + `git show --stat`; Red failing list; Green canonical line +
capture path; m00 exit code; deviations; observations. No raw output pastes.
