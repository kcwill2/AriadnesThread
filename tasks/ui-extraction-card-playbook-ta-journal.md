# T-A Journal - ui-extraction-card-playbook: additive DTO target-file plan fields

Handoff: `tasks/ui-extraction-card-playbook-ta-handoff.md` (contract followed literally).
Start rev: `4a65b6f` (parent chain: `f2e1790` planning artifacts, `44dfb73` brace-underflow journal).
Scope: `crates/ui` DTO only. Read-only playbook milestone; this hand is the DTO feed.

## What was done (RGB)

Red (gate 1, compile-clean by design): in `proposals_payload.rs` the key-stability
lock `test_proposal_dto_key_names_are_stable` was amended to the 13-key union
(old 11 + exactly `target_files`, `target_files_truncated`; the amendment IS the
sanctioned de-thaw per P3), and ONE value-level test
`test_target_file_keys_emitted_when_fields_set` was appended using JSON `Value`
indexing only - no reference to not-yet-existing types or fields.
Digest verbatim: `196 passed / 2 failed / 0 ignored / 2 suites` (exit 1; capture
`target/testrun-capture.txt`, 233 lines). Failing list = exactly the two named
tests; the key assert showed the 11-old-key left vec vs the 13-key expectation;
`target_files` absent. Every semantic reason matched the prediction.

Green (gate 2): added the C1 block to `crates/ui/src/dto.rs` -
`EXTRACTION_PROPOSAL_MAX_TARGET_FILES = 8`, `ProposedTargetFileDto`
(`path`/`symbol_names`/`symbol_count`, derives per shape C1 verbatim), and the two
`ExtractionProposalDto` fields `target_files: Vec<ProposedTargetFileDto>` +
`target_files_truncated: bool`, both `#[serde(default)]`, inserted after `risk`.
Compelled struct-literal fill-ins (`target_files: vec![]`,
`target_files_truncated: false`) in `proposals_payload.rs::sample_proposal` and
`dto_builders.rs::d2_sample_proposal`. Digest verbatim:
`198 passed / 0 failed / 0 ignored / 3 suites` (exit 0; capture
`target/testrun-capture.txt`, 218 lines).

Green step 3 (same commit): appended the typed locks
`test_legacy_payload_without_target_file_keys_decodes` (keys removed from the
serialized row; decodes to `vec![]` / `false` - P2) and
`test_target_files_round_trip_preserves_content` (one `ProposedTargetFileDto`,
`symbol_count` 3 vs 2 capped names, payload round-trip equality via `PartialEq`).
Refactor (step 4): doc comments on the new const/type/fields were authored WITH
the green edit (repo-relative `path`, lossy-rendered like the plan PathBuf,
cap semantics); the step-4 re-run then confirmed no regression.
Final digest verbatim: `200 passed / 0 failed / 0 ignored / 3 suites` (exit 0;
capture `target/testrun-capture.txt`, 220 lines). Totals reconcile: 197 prior +
1 Red-phase + 2 typed = 200.

Outer payload-level `skip_serializing_if` on `extraction_proposals`
(dto.rs:223-226 region) untouched; its two key-stability tests green UNEDITED.

## Gates (invocation census)

1. Red `python tools/testrun.py --distill -p ariadne-ui` - exit 1, 2 named failures.
2. Green same command - `198 passed / 0 failed`.
3. Step-3/4 re-run same command - `200 passed / 0 failed`.
4. `python tools/hygiene_check.py` - exit 0 (R1/R2 clean; R3 warn-only lines all
   pre-existing code, zero from this hand's lines; exit contract read from the
   tool: violations->1, scanner errors->2, R3 never gates).
5. `python tools/encoding_guard.py --staged` - exit 0 before the commit.
6. Commit via `commit_scoped.py` (single), scopes exactly the touch-set.
Testrun invocations: 3 (handoff steps 1-4 mandate red, green, and a step-4
re-run); total 6 <= 10 bound. See deviation D3.

## Commit

Landing: `feat(ui): additive ExtractionProposalDto target-file plan fields
(ui-extraction-card-playbook T-A)` via
`python tools/commit_scoped.py --scope crates/ui/src/dto.rs --scope
crates/ui/tests/ui_test/proposals_payload.rs --scope
crates/ui/tests/ui_test/export_order.rs --scope
crates/ui/tests/ui_test/dto_builders.rs --scope
tasks/ui-extraction-card-playbook-ta-journal.md`.
Landing hash is reported to the orchestrator in the return report (the single
commit rule forbids a self-hash append commit; the story-milestone precedent
`4900e04` tracked the landing rev from the outside). Parent = this journal's
start rev `4a65b6f`.

## Deliberately NOT done

- No re-export line in `crates/ui/src/lib.rs` - `pub use dto::*;` (lib.rs:5)
  already re-exports `ProposedTargetFileDto`; verified, build agrees.
- No producer/feed edit (`crates/cli/**`), no assets, no fixtures, no tools.
- No assertion weakened anywhere; existing tests byte-identical except the
  sanctioned P3 amendment and the compelled mechanical fill-ins (plus the
  compelled `use` line gaining `ProposedTargetFileDto` in proposals_payload.rs).
- dto.rs module extraction NOT performed (shape 9 defers it to a future
  milestone; dto.rs lands 907/1000).

## Deviations (numbered, for the reviewer)

D1. `export_order.rs` needed NO fill-in: the handoff/shape expected a struct
    literal at :163, but that line is payload-level
    `extraction_proposals: vec![]` with no `ExtractionProposalDto` literal in
    the file. File left byte-identical; it therefore carries no diff and does
    not appear in `git show --stat` despite being in the commit scopes
    (acceptance 2 reads as "exactly the touched files"; an unchanged file
    cannot appear in a diff). Scope was passed per the handoff's literal gate 5.
D2. Allowlist wall: direct `git` and piped shell forms are denied in this
    session (catch-all boilerplate; same signature as the open deferred ledger
    row). `git add` through `distill.py` was also denied, so staged-set
    verification used: clean-index census pre-commit (porcelain via distill,
    first column empty), commit_scoped's internal `commit --only` glob scoping,
    and post-commit `git show --stat` set-equality. Duty-1 `encoding_guard
    --staged` accordingly checked the empty index (vacuously clean) for
    doc-allowlisted paths; the T-A pre-commit `--staged` run DID check the real
    Rust blobs (crates/ is not allow-listed).
D3. Gate-count: 6 invocations vs the handoff's planned 5 - the handoff's own
    step 4 ("re-run") plus step-3 tests require a third testrun; all within the
    <= 10 sizing-law bound.
D4. Doc comments authored in the green edit rather than a separate refactor
    commit step; step 4 executed as its mandated verification re-run.

## Out-of-scope observations (reported, not fixed)

O1. `crates/cli/src/commands/ui.rs:524` constructs `ExtractionProposalDto` with
    an exhaustive struct literal (no `..` rest), so `crates/cli` does not
    compile between this landing and T-B. This is the designed hard-chain cost
    (T-B owns those field fills in its own touch-set); the scoped
    `-p ariadne-ui` gate is unaffected, and this commit keeps `crates/cli`
    byte-identical as required.
O2. Scope-adjacent observation for T-B/C only: none beyond O1. Census suites
    counted 2 in Red vs 3 in Green runs (same cosmetic drift as story-milestone
    D4; totals reconcile).
