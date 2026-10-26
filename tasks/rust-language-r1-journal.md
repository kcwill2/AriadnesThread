# R-1 Journal - rust-language spine (FileKind::RustSource + LanguageId::Rust)

Implementer: R-1 hand of the rust-language milestone. Contract:
`tasks/rust-language-r1-handoff.md`.

## Bootstrap (binding input 5)

- `python tools/checkpoint_guard.py status` - FIRST HAND OF THE MILESTONE.
- Result: clean tree (no dirty files, no untracked files) at HEAD `bc0c11c`
  on branch `balsam-basil`. Start rev (dispatch head, journalled verbatim):
  `bc0c11c chore(tasks): rust-language handoff set (plan + 10 handoffs +
  ledger flip; memory mirror same-commit)`.
- Worktree census: this worktree `balsam-basil` dirty=0; other worktrees
  untouched by this hand. Nothing unexpected - no STOP condition.

## Gate 1 - clean baseline BEFORE any edit

- `python tools/testrun.py --distill -p ariadne-repo` exit 0.
- Canonical line: `38 passed / 0 failed / 0 ignored / 5 suites`.

## Gate 2 - Red (first mutation: matrix pin :103 only)

- Edit: row :103 `("a.rs", FileKind::Other),` removed from the cases vec and
  replaced by the test-body red assertion
  `assert_ne!(FileKind::from_path(Path::new("a.rs")), FileKind::Other, "R-1 red: ...")`.
  Row :114 (`a.h.rs`) left untouched at this step.
- `python tools/testrun.py --distill -p ariadne-repo` exit 1 (non-zero).
- Canonical line: `30 passed / 1 failed / 0 ignored / 1 suites`.
- The ONE expected failure: `types::file_kind_matrix_tests::pin_file_kind_lowercase_extension_matrix`
  - `assertion \`left != right\` failed`, `left: Other` (right: Other),
  at `crates\repo\src\types.rs:87:9`. Capture:
  `target/testrun-capture.txt` (54 lines at this run).

## Checkpoints

- GREEN (gate 3): `python tools/testrun.py --distill -p ariadne-repo` exit 0,
  `39 passed / 0 failed / 0 ignored / 5 suites` - baseline 38 + the one new
  pin `rust_source_kind_carries_the_rust_identity`. Items 1-7 all applied;
  the red assertion was REPLACED by the honest `("a.rs", FileKind::RustSource)`
  row, never weakened; matrix rows :103 and :114 both flipped; `a.RS` case
  row added; `is_source_or_header` untouched (Scala precedent); frozen trio
  (`WireLanguageForm`, `language_transparent_frame`, `legacy_rank`) untouched.
- Gate 4 (cli, amendments A1-A4 GREEN UNEDITED):
  `python tools/testrun.py --distill -p ariadne-cli` exit 0,
  `182 passed / 0 failed / 8 ignored / 8 suites`. No cli file was touched;
  the enum/identity edits turn nothing red (INV-2 first-wins unchanged,
  plugin set untouched).
- Gate 5 (full workspace census): `python tools/testrun.py --distill` exit 0,
  `1854 passed / 0 failed / 13 ignored / 119 suites`. Reconcile: anchor
  `1853 passed / 0 failed / 13 ignored / 119 suites @ 522a770` -> delta +1,
  fully attributed to the single new test
  `identity::tests::rust_source_kind_carries_the_rust_identity`; suites flat;
  ignored 13 unchanged. Capture: `target/testrun-capture.txt` (3335 lines).
- Refactor pass: diff audited line by line (`git diff` both files via
  `tools/distill.py`). Doc wording plain and honest; names honest; enum hunk
  is a pure append (`RustSource` last, no reordered line - L11);
  identity.rs hunks contain ONLY the RUST const, associated RUST, Rust const,
  `rust()` fn, `from_file_kind` arm, appended pin test - the frozen trio
  appears in NO hunk (L1). Zero changes resulted from the refactor pass, so
  the gate-3 Green run stands; no re-run consumed.
- Gate 6 (hygiene): `python tools/hygiene_check.py` exit 0
  (`python tools/distill.py -- python tools/hygiene_check.py` ->
  `### generic PASS (exit 0)`; capture 173 lines). R1/R2 blocking clean; every
  printed line is warn-only R3, all pre-existing; NO line names
  `crates/repo/src/types.rs` or `crates/repo/src/identity.rs`.

## Gate inventory (itemized, per the declared budget)

- Gate 1 baseline: 1 invocation of `testrun.py --distill -p ariadne-repo`.
- Gate 2 Red: 1 invocation of `testrun.py --distill -p ariadne-repo`.
- Gate 3 Green: 1 invocation of `testrun.py --distill -p ariadne-repo`.
- Gate 4 cli: 1 invocation of `testrun.py --distill -p ariadne-cli`.
- Gate 5 census: 1 invocation of `testrun.py --distill`.
- Gate 6 hygiene: 1 invocation of `hygiene_check.py`.
- Refactor re-runs consumed: 0 (refactor produced no code change).
- Total scoped invocations: 6 - inside the declared budget.
- Commit-cycle trio: `encoding_guard --staged` + `commit_scoped` (journal
  rides inside; never `git add .`).
- Deviations: none. `Cargo.lock` untouched (std-only edit; cargo rewrote
  nothing - `git status` at the commit cycle shows only the touch-set).

## Lock proofs (audited pre-commit on the working diff; re-run at the
## `bc0c11c..HEAD` commit range in the landing report)

- L1/L11: `git diff -- crates/repo/src/identity.rs` reviewed: ONLY the six
  allowed additions; frozen trio in no hunk; enum diff is a pure append
  after `Other`.
- L2/L4/L7-L9: `git diff --stat` shows only the two source files; the journal
  is the only new path (touch-set).
- L12: `python tools/encoding_guard.py` - gate-7 form `--staged` first per
  the handoff; `--all` superset as the L12 proof.

(to be appended per Green gate)

## Gate inventory (itemized per-hand counts, M28 law)

- Gate 1 baseline: 1 invocation.
