# AR-04 Journal - Rollback on mid-apply failure (T1 HIGH)

Date: 2026-10-09. Branch worktree: `shining-enthusiasm`.

## What was done

### Red

New lock file `crates/validate/tests/mid_apply_rollback_test.rs` with two
failing-writer oracle tests, run via
`python tools/testrun.py --distill -- -p ariadne-validate --test mid_apply_rollback_test`:

- Digest (RED): `0 passed / 2 failed / 0 ignored / 1 suites`, capture
  `target/testrun-capture.txt`. Both failures were the predicted semantic
  ones: the propagated errors were the raw IO causes (`Cannot create a file
  when that file already exists. (os error 183)` and `stream did not contain
  valid UTF-8`) with no rollback line and the pre-failure writes left on disk.

Oracle design (deliberate, see "Deliberately not done" for rejected options):

- Lock 1 (first write phase): the second generated unit's parent-directory
  slot is occupied by a plain file staged by the test, so the second unit's
  `create_dir_all`/`write` fails deterministically on any platform AFTER the
  first unit's write landed. Asserts: `Err`, error text contains
  `rolled back 4 files`, the first unit's file is gone, consumer and
  fixture-blocker bytes are byte-identical.
- Lock 2 (second write phase, "all previously written files"): the generated
  unit is created and the FIRST consumer is genuinely rewritten
  (`OLD-DEP` -> `NEW-DEP`), then the SECOND consumer fails to decode because
  it holds invalid UTF-8 staged via byte escapes (`b"PRE \xff\xfe post\n"`).
  Asserts: `Err`, `rolled back 4 files`, created unit removed, mutated first
  consumer restored to its original bytes, failing consumer untouched.

### Green

`crates/validate/src/lib.rs`:

- `apply_with` now runs the write/rewriter/strip phases through a private
  `write_all_targets` (phases moved verbatim - success path unchanged), and
  on `Err` routes through a private `rolled_back_error` which calls the SAME
  `TransactionApplier::rollback(&snapshot, &new_paths)` the caller-side
  tier-failure pattern uses (cli `optimize.rs` / `transform.rs`), then
  rethrows the original cause with a `rolled back N files` context line.
- N is defined as the number of paths the rollback pass processes:
  `snapshot.entries.len() + new_paths.len()` (documented at
  `rolled_back_error`, locked exactly by both tests).
- If rollback itself fails, the rollback error propagates with the original
  cause as context - an unrestored tree is never reported as rolled back.
- `apply_with` doc comment updated to state the now-true guarantee.

Digests (verbatim, capture `target/testrun-capture.txt` each time):

- Scoped new locks after fix: `2 passed / 0 failed / 0 ignored / 1 suites`.
- Handoff gate 1, full scoped suite: `16 passed / 0 failed / 0 ignored / 5 suites`.

### Refactor

The moved phases are verbatim (no behavior deltas); the two new private
helpers keep every function well under 100 lines. Re-runs stayed green.

## Deliberately not done

- Did not change `rollback`, `FileSnapshot`, or `apply_with`'s public
  signature: callers (cli optimize/transform, integration tests) keep the
  external tier-rollback pattern; it remains correct (rollback of an
  already-restored snapshot is idempotent on the success paths exercised).
- Did not add rollback for failures BEFORE the snapshot (path derivation,
  capture): nothing has been mutated there, so restore would be a no-op.
- Rejected failing-writer oracles: NUL-byte paths (Windows `CreateFileW`
  embedded-NUL truncation semantics are not a reliable failure), replacing a
  target with a directory mid-apply (breaks rollback's own re-write, so the
  fix could not restore), read-only attributes (root/elevation-dependent on
  Unix CI). Chosen oracles are deterministic cross-platform IO errors that
  leave no state blocking restoration.
- Did not touch golden fixtures, byte-locked manifests, or theme baselines.
- Did not modify the existing transaction tests; they pass unmodified,
  which is part of the success-path byte-identity evidence.

## Deviations

- The handoff cites the "tier-failure rollback at lib.rs:792-834"; at HEAD
  those lines are tier tests. The actual pattern to mirror
  (`if fail { rollback(&snapshot, &new_paths) }`) lives at the callers
  (`crates/cli/src/optimize.rs:408-412`, `crates/cli/src/commands/transform.rs`
  rollback call). Mirrored that pattern inside `apply_with` as specified.
- Error text uses anyhow `.context()` so the "rolled back N files" line
  leads the message and the original cause stays in the chain.

## Gates (verbatim results)

- `python tools/testrun.py --distill -- -p ariadne-validate` ->
  `16 passed / 0 failed / 0 ignored / 5 suites` (PASS, final state; capture
  `target/testrun-capture.txt`)
- `python tools/hygiene_check.py` -> ran; zero R1/R2 hits in the touch set.
  All reported R1/R2 hits are pre-existing `crates/ui` violations outside
  this handoff's scope; the only validate-crate line is a pre-existing
  warn-only R3 at `crates/validate/src/lib.rs:104` (untouched
  `FileSnapshot::capture`). The new test file has no findings.
- `python tools/encoding_guard.py --staged` -> exit 0, no findings, after
  staging exactly the three touch-set files (staged via the distilling
  entry point `python tools/distill.py -- git add ...`).

## Commits

- `8209a7e` `fix(validate): rollback on mid-apply failure (T1)` via
  `python tools/commit_scoped.py` with per-file scopes:
  `crates/validate/src/lib.rs` (+65/-4),
  `crates/validate/tests/mid_apply_rollback_test.rs` (+233),
  `tasks/ar-04-handoff-journal.md` (+94).
