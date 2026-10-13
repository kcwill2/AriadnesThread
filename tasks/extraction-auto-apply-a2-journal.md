# A-2 Journal - extraction-auto-apply (apply_plan_with transactional core)

Handoff of record: `tasks/extraction-auto-apply-a2-handoff.md`.
Shape binding inputs: `tasks/extraction-auto-apply-shape.md` sections 2
(invariant A1), 3 (C1 verbatim), 5 (module map A-2 row), 8 (hygiene),
9 (name list); kickoff section 3 item 10.
Start rev: `ff93508` (duty 1 = the A-1 reviewer verdict commit landed in
this session as its first act). Branch `main`. Implementer hop
2026-10-13.

## Boot verification

- Pre-duty-1 HEAD: `7d819bb` (A-1 journal rider); working tree carried
  exactly one untracked path: the A-1 verdict file. No foreign dirt under
  `crates/**`; no STOP.
- Duty 1 sequence, verbatim: `python tools/encoding_guard.py --staged`
  silent exit 0 (the index was empty at guard time - the verdict file was
  untracked; its byte-level coverage lands in this hand's gate 9
  `--all` run, which scans it as tracked content);
  `python tools/commit_scoped.py --scope tasks/extraction-auto-apply-a1-verdict.md
  -m "review(extraction-auto-apply): A-1 APPROVED verdict + gate evidence"`
  landed `ff93508`, numstat `473  0  tasks/extraction-auto-apply-a1-verdict.md`.
- `git show --stat HEAD`: exactly one file, 473 insertions, nothing else.
- Post-duty-1 `git status --porcelain`: zero lines - tree fully clean,
  so `crates/**` is clean. Boot rule satisfied.

## F14 caller census at the dispatch base (start rev `ff93508`)

Names this hand adds: `apply_plan_with`, `ApplyTransactionReport`.

- Tree census over `crates/**` and `tests/**` at the start rev: ZERO hits
  for both names (pre-extraction truth, matching the handoff's "At HEAD
  023558c both counts are 0 outside transform.rs"; A-1 never touched
  `transform.rs` or these names).
- `apply_plan_with(` occurrences inside `crates/cli/src/commands/transform.rs`
  at the start rev: 0.
- Expected post-hand census (A1 caller census of record, acceptance 3):
  exactly 2 lines matching `apply_plan_with(` in `transform.rs` - the
  definition plus the wrapper's one call. STOP condition: any occurrence
  outside those two lines, or any hit outside `transform.rs`, forks the
  transactional path and is a STOP.

## Gate log

Canonical lines quoted verbatim; captures at
`target/testrun-capture.txt` / `target/distill-capture.txt` (path printed
by each distilling entry point).

- Gate 1 (cli baseline, `python tools/testrun.py --distill -p ariadne-cli`):
  `130 passed / 0 failed / 8 ignored / 7 suites`, exit 0. Matches the A-1
  reviewer hop's scoped line exactly - the tree is unchanged under
  `crates/cli`.
- Gate 2 (frozen apply suites baseline, `python tools/testrun.py --distill
  -p ariadne-integration-tests --test m10_validation_test --test
  m20_apply_migrate_characterization_test --test m23_symbol_isolation_test`):
  `18 passed / 0 failed / 0 ignored / 3 suites`, exit 0. This line is the
  immovable bar for gate 5 (byte-for-byte).

## Red evidence (gate 3)

`python tools/testrun.py --distill -p ariadne-cli` at the Red tree
(test module authored first, nothing else changed):

- Digest: `### cargo test FAIL (exit 1)`; canonical line
  `124 passed / 1 failed / 0 ignored / 1 suites`. The `1 suites` shape is
  the digest reflecting cargo's abort: the lib test binary failed, so
  cargo did not run the remaining 6 targets (capture ends with
  `error: test failed, to rerun pass -p ariadne-cli --lib`). The lib
  binary itself: 124 passed + 1 failed = 125 = the start-rev lib-suite
  count plus the one new test. NO other failure anywhere in the capture
  (exactly one FAILED line).
- The named test, verbatim panic (capture lines 29/30):
  `test commands::transform::tests::test_apply_plan_with_is_the_shared_transactional_core ... FAILED`
  `thread 'commands::transform::tests::test_apply_plan_with_is_the_shared_transactional_core' (57036) panicked at crates\cli\src\commands\transform.rs:430:9:`
  `C1: transform.rs must define the shared transactional core apply_plan_with`
- The second named panic line (`C1: the core must return
  ApplyTransactionReport, never print it`) is designed for the same test's
  second assert. With BOTH needles absent pre-extraction, `assert!`
  short-circuits and only the first panic is observable in a single run;
  the second message is verbatim in the test source and stays armed
  forever after as the anti-regression lock. Observation O1, not a
  deviation: the handoff's Red wording ("the second assertion in the same
  test panics with ...") describes each assertion's failure text; the
  single-run mask is inherent to a two-assert single test.
- This was a semantic Red: the file compiled, the test read its own file
  via `include_str!("transform.rs")` and failed on the missing contract
  text, per the `format!` needle idiom (no contiguous needle text exists
  in the authored comments/messages, so the test cannot match itself).

## Green evidence (gates 4 and 5)

Extraction per "What to build": wrapper thin, core + struct inserted
between the wrapper and `load_plan_for_action`; `commit_or_rollback` and
`report_rollback` lost their `println!` calls and now return
`Result<ApplyTransactionReport>`; artifact-record writes stay in place
(save-then-return order unchanged; in `report_rollback` the removed print
sat before the rollback and its removal preserves the rollback-then-save
artifact order). Every moved comment (M23 T1/T7, T8b, M18.10) moved with
the code. The core uses `plan.id.0` where the old code used the caller's
`plan_id` - the same string by `load_plan_for_action`'s id match, so the
transaction manifest `plan_id` content is unchanged (m20 asserts it).

- Gate 4 (`python tools/testrun.py --distill -p ariadne-cli`):
  `### cargo test PASS (exit 0)`; canonical line
  `131 passed / 0 failed / 8 ignored / 7 suites` = gate 1's baseline
  exactly +1 passed (the one mandated test), suites unchanged.
- Gate 5 (frozen suites command identical to gate 2):
  `### cargo test PASS (exit 0)`; canonical line
  `18 passed / 0 failed / 0 ignored / 3 suites` - IDENTICAL byte for byte
  to gate 2. Acceptance 4 MET. The three frozen suites passed UNEDITED,
  proving: m10 success path + `Committed` transaction artifact, m20
  `Plan PLAN-999 not found` error text + `Failed to locate repository
  path` + full disk-effect characterization (generated files, consumer
  bytes, manifest tiers 3/3), m23 Scala end-to-end apply with offline
  tiers, moved-unit staging and stripped source.
- Success-path stdout by inspection of the wrapper: exactly
  `println!("Applying {}...\n", plan_id)` then
  `println!("{}", report.outcome.format_output())` - the same two prints,
  same text, same order as the start rev. The rollback path prints the
  same `format_output()` text through the same single wrapper print.
  Error-path delta (contract-sanctioned): extraction/consumer/apply
  errors now surface AFTER the `Applying` line because the print moved
  before the core; `Plan {} not found` and the missing-path error still
  print NOTHING first (they fire before the print) - m20 green proves
  both error texts.
- Core prints nothing: the apply family (`apply_plan_with`,
  `commit_or_rollback`, `report_rollback`) contains zero `println!`
  (grep-verified); the remaining `println!` hits are all in the
  untouched `extract_plan_headers` / `build_migration_preview` families.

## Refactor

`apply_plan_with` is 62 lines (162-223), `commit_or_rollback` 60,
`report_rollback` 17, the wrapper 16 - all under 100; no helper
extraction needed (the tier sequence did not grow). No duplication was
introduced (the extraction IS the deduplication); no banned identifiers,
no single-letter locals in the new code (`repo_path`, `reduced_model`,
`source_reader`, `text_extractor`, `header_output_dir`, `report` - shape
section 9 names honored; `p`/`h`/`f`/`i` hits in the capture are
pre-existing lines outside the moved families - gate 6 enforces).

## Caller census after the hand (acceptance 3, A1 of record)

- `apply_plan_with(` in `transform.rs`: 2 lines - the definition (:162)
  and the wrapper's single delegation (:139). No third call site;
  `TransactionApplier::apply_with` is still called exactly once in the
  file (inside the core); `crates/validate/**` untouched.
- `ApplyTransactionReport`: exists only in `transform.rs` (definition,
  core return, both tier returns, struct literals, the test needle);
  zero occurrences elsewhere in `crates/**` and `tests/**`.

## Hygiene (gate 6)

`python tools/hygiene_check.py` exit 0. Every finding is R3 warn-only,
all pre-existing rows in other files; `transform.rs` contributes ZERO
rows (the moved code carried its identifiers verbatim and the detector's
population is unchanged - `transform.rs` was absent from the warning
population at the start rev and remains absent). R1/R2 zero: file 475
lines (start 415; band "~470" respected), functions 16/62/60/17 lines.

## Machine checks (acceptance 2 and 3, pre-commit)

- Acceptance 2: `git grep -n "pub fn apply_transform_plan" ff93508 -- crates/cli/src/commands/transform.rs`
  (via distiller) prints `ff93508:...:129:pub fn apply_transform_plan(repo_path: &PathBuf, plan_id: &str) -> Result<()> {`
  - the current file carries the byte-identical line at the same `:129`.
- Acceptance 3: `git grep -c "apply_plan_with(" -- crates/cli/src/commands/transform.rs`
  (via distiller) prints `crates/cli/src/commands/transform.rs:2` -
  definition + wrapper call, the A1 census of record. No third call site
  exists; no second `TransactionApplier::apply_with` was authored;
  `crates/validate/**` untouched (byte-identical, it appears in no
  working-tree change).

## Bench stance (producer zero-diff proof, exemption with proof)

This hand touches `crates/cli/src`, so the DOCUMENTED_PHASES dated-row
stance applies. The timing gate is deterministically blocked at
`tools/bench_compare.py:84` (stale tuple label vs the producer's printed
phase name - the A-1 verdict Part 7.4 records the line-level proof); no
timing number is attributable to A-2, and `bench_compare.py` was NOT run
in this hop (the reviewer hop owns it).

Producer zero-diff proof over the landing range (`ff93508..landing`):
the hand's full change set is two paths - `crates/cli/src/commands/transform.rs`
and this journal. `git diff --numstat "ff93508..HEAD" -- crates/cli/tests`
is recorded in the landing record (rider) as EMPTY after the landing
commit: the bench producer set (`crates/cli/tests/payload_tail_bench.rs`
et al.) is byte-identical across the range. What the core changes is the
call structure, not the algorithm: every statement in `apply_plan_with`
moved verbatim from `apply_transform_plan` (same engine, same backends,
same tier sequence, same artifact writes); the only added work on the CLI
path is one additional idempotent `Repository::locate` +
`ArtifactManager::prepare_for_repo` pair (directory preparation, no
indexing), inside non-timed command plumbing.

## Deviations (numbered, with reasons; never silent)

- D1 (gate 7 staging route): the two handoff gate commands are `encoding_guard
  --staged` followed by `commit_scoped.py`; with an empty index the guard
  would scan nothing and the gate would be vacuous. I first staged
  EXACTLY the two touch-set paths via
  `python tools/distill.py -- git add -- crates/cli/src/commands/transform.rs tasks/extraction-auto-apply-a2-journal.md`
  (explicit paths, never `.`, no glob), then ran the guard so it
  inspects the blobs actually landing, then ran gate 8 verbatim
  (`commit_scoped.py` re-adds its resolved scopes and commits with
  `--only`). Precedent: playbook T-C D2 "distill staging route"
  ACCEPTED. Scope discipline unchanged.
- D2 (journal rider commit): the landing record (landing hash, gate 9
  result, post-commit acceptance proofs) cannot exist inside its own
  commit. Gate 8's commit carries the journal as-is; a second scoped
  commit, journal-only, lands the landing record. Precedents: W-1 `82e5ffe`
  (D5 APPROVED) and A-1 `7d819bb` (D6 APPROVED). The 9-gate list is
  discharged verbatim; the rider adds no source change.
- O1 (observation, not a deviation): the Red's second named panic is
  masked by `assert!` short-circuit in the same test while both needles
  are absent - inherent to the mandated single-test two-assert shape;
  documented under "Red evidence".

## Landing record (journal rider, D2)

- Landing commit: `fdfe950` (parent `ff93508`), message verbatim from
  gate 8. `commit_scoped.py` numstat digest:
  `74  14  crates/cli/src/commands/transform.rs` and
  `222  0  tasks/extraction-auto-apply-a2-journal.md`.
- Gate 7 (with D1 staging route): the two paths staged via
  `python tools/distill.py -- git add -- crates/cli/src/commands/transform.rs tasks/extraction-auto-apply-a2-journal.md`
  (`### generic PASS (exit 0)`; git's informational LF/CRLF warning on the
  journal's working copy only - the staged blob is what the guard
  scanned), then `python tools/encoding_guard.py --staged` silent exit 0.
- Gate 8: `python tools/commit_scoped.py --scope crates/cli/src/commands/transform.rs
  --scope tasks/extraction-auto-apply-a2-journal.md -m "refactor(cli):
  extract the apply_plan_with transactional core shared by CLI and UI"`
  - the digest above; never `git add .`.
- Gate 9: `python tools/encoding_guard.py --all` silent exit 0 post-commit.
  This also discharges the byte-level coverage of duty 1's verdict file,
  which was untracked at duty-1 guard time (it was scanned as tracked
  content here, clean).
- Acceptance 1: `git show --stat HEAD` at the landing commit shows
  exactly `crates/cli/src/commands/transform.rs` (88 +- lines changed,
  +74/-14) plus this journal (222 insertions). 2 files, 296 insertions,
  14 deletions. Nothing else.
- Acceptance 7: `git diff --name-only "ff93508..HEAD"` = exactly the two
  touch-set paths. The m10/m20/m23 files appear in NO hunk of this
  hand's range; ditto `tests/golden/**`, `tests/fixtures/**`,
  `crates/validate/**`, `crates/transform/**`, `crates/artifact/**`,
  assets, `Cargo.toml`, `Cargo.lock`, `tools/**`.
- Producer zero-diff proof (bench stance): `git diff --numstat
  "ff93508..HEAD" -- crates/cli/tests` returned EMPTY (capture 0 lines) -
  the bench producer set is byte-identical across the landing range.
  Combined with the DOCUMENTED_PHASES deterministic blocker at
  `tools/bench_compare.py:84` (exit 2 before any timing comparison), NO
  timing number is attributable to A-2; the reviewer hop carries the
  exemption with this proof. `bench_compare.py` was not run in this hop.
- Acceptance 3 end-run (mandated re-run "at preflight AND at the end"):
  `git grep -c "apply_plan_with(" HEAD -- crates/cli/src/commands/transform.rs`
  prints `HEAD:crates/cli/src/commands/transform.rs:2`.

### Acceptance table (final)

1. `git show --stat` = transform.rs + journal, exactly - MET.
2. `apply_transform_plan` signature byte-identical to the start rev
   (`pub fn apply_transform_plan(repo_path: &PathBuf, plan_id: &str) -> Result<()>`
   at `:129` both sides) - MET.
3. Census `apply_plan_with(` = 2 (definition + the wrapper's one call),
   re-run at the end - MET.
4. Gate 5 canonical line identical byte-for-byte to gate 2
   (`18 passed / 0 failed / 0 ignored / 3 suites`) - MET.
5. `python tools/hygiene_check.py` exit 0 - MET.
6. `python tools/encoding_guard.py --all` exit 0 - MET.
7. m10/m20/m23 in no hunk (landing range = the two touch-set paths) - MET.

Out-of-scope observations: none new. The pre-existing R3 warn population
and the `tools/bench_compare.py:84` DOCUMENTED_PHASES defect are
unchanged and owned elsewhere (ledger dated rows).
