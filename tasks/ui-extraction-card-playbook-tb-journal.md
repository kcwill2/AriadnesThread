# T-B Journal - ui-extraction-card-playbook: feed wiring (cli)

Handoff: `tasks/ui-extraction-card-playbook-tb-handoff.md`.
Start rev (HEAD at session start): `af11113` (contains T-A `df1e21e`).
Scope: `crates/cli` only - fill `target_files` / `target_files_truncated`
in `extraction_proposal_dto` from the plan's per-symbol `target_header`.

## Ground truth at boot

- `crates/cli` did not compile at start: the exhaustive
  `ExtractionProposalDto` literal at `ui.rs:524-536` lacked the two T-A
  fields. Restoring green was part of this hand.
- Carry-over ruling from T-A review honored: authored content pure ASCII,
  no U+2014 em dashes, in every file touched here.

## What was done

- `crates/cli/src/commands/ui/target_files_tests.rs` (NEW): the 5 P1
  feed-truth oracle tests on constructed `OptimizationPlan`s, reusing the
  `builder_tests.rs` construction idiom via local helpers (`extraction`,
  `plan_extracting`, `one_symbol_per_file`); each test under 40 lines;
  imports come from `super`, existing test modules untouched.
- `crates/cli/src/commands/ui.rs`:
  - NEW pure helper `target_file_groups` (doc-commented caps + BTreeMap
    path-asc determinism; per-group `symbol_id` sort mirrors the existing
    `symbol_names` sort): groups `proposed_extractions` by `target_header`,
    caps names at `EXTRACTION_PROPOSAL_MAX_NAMES` with uncapped
    `symbol_count`, caps rows at `EXTRACTION_PROPOSAL_MAX_TARGET_FILES`
    with `truncated` iff group count exceeds the cap; `path` is the
    lossy-rendered PathBuf.
  - `extraction_proposal_dto` (now ~38 lines) fills the two DTO fields from
    the helper; all 11 pre-existing fields byte-faithful (P11).
  - ONE new `#[cfg(test)] mod target_files_tests;` line after
    `boot_progress_tests`.
  - No import-line change (helper uses fully-qualified `std::collections::
    BTreeMap` / `std::path::PathBuf`, matching the file's existing style),
    so the ui.rs diff stays confined to the mapping region + the mod line.
- ui.rs lands at 839 lines (< 1000); helper 33 lines, dto 38 (< 100).

## What was deliberately NOT done

- No planner run, no planner hook, no I/O in the DTO mapping.
- No edits to `crates/cluster/**`, `crates/lang-cpp/**`, `crates/ui/**`,
  `crates/cli/src/lib.rs`, `commands/{plan,transform}.rs`, `optimize.rs`,
  the existing `commands/ui/*` test modules, `tests/**` (incl.
  `m00_facts_path_parity_test.rs`), fixtures, goldens, `tools/**`.
- No assertion weakened; no `UPDATE_GOLDENS`; no ui_smoke (NIL value here
  per the handoff; runs at T-C); no bench_compare (reviewer hop).

## RGB evidence

- Red (gate 1): `python tools/testrun.py --distill -p ariadne-cli` ->
  `97 passed / 4 failed / 0 ignored / 1 suites`, exit 1; failing list =
  exactly the 4 P1 grouping/cap tests
  (`test_target_files_group_by_path_ascending`,
  `test_symbol_file_assignment_preserved`,
  `test_per_file_name_cap_and_uncapped_count`,
  `test_file_row_cap_sets_truncated_flag`), all failing on empty
  `target_files` / missing truncated flag - the predicted semantic reason,
  not a compile error. `test_existing_fields_unchanged_by_the_fill`
  already passed (it pins the pre-change behavior of the old fields).
- Green (gate 2): same command -> `101 passed / 0 failed / 1 ignored /
  5 suites`, exit 0 (includes untouched `builder_tests`,
  `progress_contract_tests`, `boot_progress_tests`).
- Gate 3 consumer side: `python tools/testrun.py --distill -p ariadne-ui`
  -> `200 passed / 0 failed / 0 ignored / 3 suites` (proves no ui-side
  edit was needed).
- Gate 4 parity lock: `python tools/testrun.py --distill -p
  ariadne-integration-tests --test m00_facts_path_parity_test` ->
  `2 passed / 0 failed / 0 ignored / 1 suites`, exit 0, file UNEDITED.
- Gate 5 hygiene: `python tools/hygiene_check.py` exit 0.
- Gate 6 encoding: `python tools/encoding_guard.py --staged` exit 0
  before committing.
- Gate 7 commit: single `commit_scoped.py` commit with the three handoff
  globs (ui.rs, target_files_tests.rs, this journal); the landing hash is
  the commit this journal ships in (visible via `git show`).

## Deviations (numbered for the reviewer)

1. Red scaffold fill: to reach the handoff's Red state ("compiles +
   fails"; the literal at :524 did not compile at all after T-A), the Red
   step temporarily filled the literal with `vec![]` / `false` so the 4 P1
   assertions failed semantically. Green replaced the fill with the real
   grouping in the same touch region. No extra commit; final diff has no
   trace of the scaffold.
2. None beyond D1. Fully-qualified `std::collections::BTreeMap` in the
   helper is a style choice to keep the diff out of the import block,
   consistent with existing fully-qualified usage in the file (e.g.
   `std::path::Path` at `proposal_basename`).

## Observations

- The Red run's digest showed `1 suites` vs the Green run's `5 suites`:
  in the failing Red run the canonical-line parser counted only the suite
  that reported failures; totals reconcile (97+4 = 101+1 ignored across
  Green). Not a product issue; recorded for census reconciliation.
- `cpp-brace-depth-underflow` landed at `af11113` (raw-string terminator
  fix) before this hand booted - file-disjoint as the kickoff predicted;
  no test here pins payload bytes, nothing chased.
