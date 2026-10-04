# T-A Handoff - ui-extraction-card-playbook: additive DTO target-file plan fields

Milestone: `ui-extraction-card-playbook` (M-number/dates by
milestone-lifecycle kickoff; do not guess). First hand in a HARD CHAIN
T-A -> T-B -> T-C: T-B consumes the struct this handoff adds; T-C renders
against the feed T-B wires. Nothing here is parallel-safe.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. Shape `tasks/ui-extraction-card-playbook-shape.md` sections C1, C4, P2,
   P3 (DTO extension, serde policy, legacy-decode and key-stability
   invariants) + section 9 hygiene notes.
3. Kickoff `tasks/ui-extraction-card-playbook-kickoff.md` (invariants 1-2,
   frozen locks table).
4. `crates/ui/src/dto.rs` lines 179-198 (the cap consts, the "plain display
   data only" rule at :182-184, `ExtractionProposalDto` to extend) and
   :223-226 (outer `extraction_proposals` `skip_serializing_if`, UNTOUCHED).
5. `crates/ui/tests/ui_test/proposals_payload.rs` (98 lines; `sample_proposal`
   builder :4-18; key-stability test :71-98 - the one amendment this hand
   makes).
6. `crates/ui/tests/ui_test/export_order.rs` around :163 and
   `crates/ui/tests/ui_test/dto_builders.rs` - struct-literal constructors
   needing mechanical fill-ins.

## Objective and invariants

Extend `ExtractionProposalDto` with the per-symbol destination plan as PLAIN
display data: `target_files: Vec<ProposedTargetFileDto>` + `target_files_truncated:
bool`, both `#[serde(default)]`, plus the `ProposedTargetFileDto` struct and
`EXTRACTION_PROPOSAL_MAX_TARGET_FILES = 8` - exactly the shape C1 block.

Invariants preserved:
- P3: `test_proposal_dto_key_names_are_stable` is amended IN THIS COMMIT to
  old-11-keys union `{target_files, target_files_truncated}` (13 sorted keys,
  the amendment IS the de-thaw; reviewer checks exactly-two additions).
- P2: JSON without the new keys deserializes to `vec![]` / `false`; new
  payloads round-trip byte-stable.
- C4: outer payload-level `skip_serializing_if` on `extraction_proposals`
  stays byte-identical; nested rows always emitted when present.
- dto.rs:182-184 respected: new fields are Strings/usize/Vec only - no
  cluster type crosses into the ui crate.
- This is a `crates/ui`-only change; no asset, no cli, no producer edit.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/ui-extraction-card-playbook-ta-journal.md` (checkpoint journal;
  create alongside; always in-set)
- `crates/ui/src/dto.rs` (the C1 extension, purely additive)
- `crates/ui/tests/ui_test/proposals_payload.rs` (P3 amendment to the one
  key list + NEW tests appended; existing tests' assertions otherwise
  byte-identical except compelled struct-literal fill-ins)
- `crates/ui/tests/ui_test/export_order.rs` (+ `crates/ui/tests/ui_test/
  dto_builders.rs`) - MECHANICAL struct-literal fill-ins ONLY
  (`target_files: vec![]`, `target_files_truncated: false`); no assertion
  changes
You MUST NOT touch: `crates/ui/src/lib.rs` (verified at planning:
`pub use dto::*;` at lib.rs:5 already re-exports the new type - shape C1's
"+1 re-export line" is NOT needed; if your build somehow disagrees, STOP and
report, do not add a line), `crates/cli/**`, `crates/ui/assets/**`,
`crates/ui/tests/ui_test/{static_html*.rs,help_center_readability.rs,
extraction_card_story.rs,dto_builders.rs assertions,common.rs,main.rs}`,
`tests/fixtures/**`, `tools/**`. No `git stash`, no branch change.

## Frozen locks

| Lock | Proof |
|---|---|
| Key-stability: exactly 11 old keys + exactly 2 new | the amended test IS the lock; reviewer diffs the amended key vec against old + `target_files`,`target_files_truncated` |
| No other DTO key changes | `static_html.rs` / `dto_builders.rs` (except mechanical fill-ins) / `help_center_readability.rs` / `extraction_card_story.rs` byte-identical in diff |
| Outer `extraction_proposals` key-stability tests (`test_empty_extraction_proposals_key_absent_from_json`, `test_payload_json_without_proposals_key_deserializes_to_empty`) | green UNEDITED |
| `tests/fixtures/**` byte-frozen | touch-set + status census |
| Mojibake/UTF-8 gate | `python tools/encoding_guard.py --staged` exit 0 pre-commit |
| Hygiene (R1/R2) | `python tools/hygiene_check.py` exit 0 (dto.rs lands ~920 of 1000 - closest file on this seam; shape section 9 records the module-extraction plan for a FUTURE milestone - not yours) |

## RGB protocol

1. Red (compiles + fails at runtime, NEVER a compile-failing state as Red):
   in `proposals_payload.rs` FIRST amend `test_proposal_dto_key_names_are_stable`
   to the 13-key union and append ONE value-level test
   `test_target_file_keys_emitted_when_fields_set` that serializes the
   EXISTING sample and asserts
   `json["extraction_proposals"][0].get("target_files").is_some()` (JSON
   `Value` indexing only - no reference to `ProposedTargetFileDto` or the
   field, which do not exist yet). Run scoped; capture the 2-failure list:
   key-vec assert shows 11 vs 13, and `target_files` absent. Everything else
   green.
2. Green: add the C1 block to dto.rs; fix struct-literal fallout in
   `export_order.rs`/`dto_builders.rs`/`sample_proposal` with the mechanical
   fill-ins; re-run until 0 failed. NEVER weaken an assertion.
3. Green, same commit: append the typed locks now that the fields exist -
   `test_legacy_payload_without_target_file_keys_decodes` (JSON object built
   without the two keys -> `target_files == vec![]`,
   `target_files_truncated == false`) and
   `test_target_files_round_trip_preserves_content` (construct one
   `ProposedTargetFileDto`, full round-trip equality via `PartialEq`).
4. Refactor: doc comments on new types (repo-relative `path`, cap semantics),
   re-run.

## Hygiene (review-enforced)

Functions < 100 lines (tests trivially pass); dto.rs < 1000 after +~40 lines;
names per shape C1 verbatim (`ProposedTargetFileDto`,
`EXTRACTION_PROPOSAL_MAX_TARGET_FILES`, `target_files`,
`target_files_truncated`); pure ASCII authored content; `path` is a String
(lossy-rendered like the plan's PathBuf - documented in the doc comment).

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. Red: `python tools/testrun.py --distill -p ariadne-ui` - digest header +
   canonical line verbatim in the report, exit 1, failing list = the 2 named
   tests.
2. Green: same command - `0 failed`, exit 0.
3. `python tools/hygiene_check.py` - exit 0.
4. `python tools/encoding_guard.py --staged` - exit 0 before committing.
5. Commit (single):
   `python tools/commit_scoped.py --scope crates/ui/src/dto.rs --scope
   crates/ui/tests/ui_test/proposals_payload.rs --scope
   crates/ui/tests/ui_test/export_order.rs --scope
   crates/ui/tests/ui_test/dto_builders.rs --scope
   tasks/ui-extraction-card-playbook-ta-journal.md -m "feat(ui): additive
   ExtractionProposalDto target-file plan fields (ui-extraction-card-playbook
   T-A)"` (never `git add .`).
Total implementer gate invocations: 5 (<= 10). If the permission allowlist
refuses a listed gate in your session, the sanctioned route is
`python tools/distill.py -- <cmd>` (open deferred ledger row).

## Dependency edges (for the orchestrator's gate bookkeeping)

- Proves this handoff: scoped `-p ariadne-ui` (the amended key test + new
  serde tests). Feed proof is T-B's `-p ariadne-cli` suite; rendering proof
  is T-C's `extraction_card_playbook.rs` + `ui_smoke`.
- T-B MUST NOT start before this commit lands (its tests compile against the
  new fields). T-C must not consume `target_files` before T-B emits them.

## Discipline clauses

- UTF-8: Write/Edit tools only, pure ASCII (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands (unscoped census, full discover, ui_smoke,
  bench_compare, raw cargo) in this session are a reviewer blocker.
- Journal checkpoint after Green (module + commit + any deviation).
- Context exhaustion: STOP, return disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/ui-extraction-card-playbook-ta-handoff.md --scope ... --cmd ...`.

## Acceptance (machine-checkable)

1. `python tools/testrun.py --distill -p ariadne-ui` -> `0 failed`, exit 0,
   including the 13-key test and all pre-existing tests UNEDITED (except
   P3 + mechanical fill-ins).
2. `git show --stat` lists exactly the touch-set files; `dto.rs` diff purely
   additive; both new fields `#[serde(default)]`.
3. `python tools/hygiene_check.py` exit 0; `encoding_guard --staged` exit 0.

Reviewer hop (`review-gate` skill): commit-vs-touch-set; key-vec spot diff
(union == old + exactly two names); `crates/ui/src/lib.rs` absent from diff;
`git diff <start>..HEAD -- crates/cli crates/ui/assets tests/fixtures tools`
-> EMPTY; scoped suite re-run; `hygiene_check.py` + `encoding_guard --all`
exit 0.

## Report back

Commit hash + `git show --stat`; Red failing-test list; Green canonical line
+ capture path; deviations; out-of-scope observations. Never paste raw tool
output.
