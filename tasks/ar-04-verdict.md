# AR-04 Verdict - Rollback on mid-apply failure (T1 HIGH)

Reviewer gate adjudication, 2026-10-09. Range `3b960bc..932041a`
(`8209a7e` fix + new test, `932041a` journal transcript rider).

## VERDICT: APPROVED

No blocker findings. No minor findings against the touch-set.

## Fresh verification (re-run by reviewer, not taken from journal)

- `python tools/testrun.py --distill -- -p ariadne-validate` ->
  `16 passed / 0 failed / 0 ignored / 5 suites` (PASS, exit 0;
  capture `target/testrun-capture.txt`, 47 lines)
- `python tools/encoding_guard.py --all` -> exit 0, zero findings
  (re-run bare and via `python tools/distill.py -- ...`:
  `### generic PASS (exit 0)`, capture 0 lines)
- `python tools/hygiene_check.py` -> zero NEW violations from this handoff.
  The two R1/R2 hits in output are `crates/ui/src/server.rs` (R2 1155
  lines, R1 `api_graph_traversal` 161 lines) - untouched by both commits
  and dirty from other concurrent work; the only validate-crate line is
  the pre-existing warn-only R3 at `crates/validate/src/lib.rs:104`.
  `crates/validate/src/lib.rs` diff regions and the new test file:
  no findings.

## Checks (review-gate procedure)

1. Ground truth: `checkpoint_guard.py status` - HEAD `932041a`,
   branch `shining-enthusiasm`; dirty files are `crates/cli`/`crates/ui`
   work outside this handoff (not judged here).
2. Commit-vs-touch-set EXACT: `git diff --name-only 3b960bc..HEAD` =
   `crates/validate/src/lib.rs`,
   `crates/validate/tests/mid_apply_rollback_test.rs`,
   `tasks/ar-04-handoff-journal.md`. All three inside the declared
   touch-set; nothing undeclared. PASS.
3. Per-commit review, oldest-first:
   - `8209a7e` (+392/-4): fix + oracle test + journal. PASS.
   - `932041a` (+19/-4): journal transcript rider only. PASS.
4. Success path untouched: `git diff --name-only 3b960bc..HEAD --
   crates/validate/tests/transaction_apply_test.rs
   crates/validate/tests/move_removal_transaction_test.rs` -> empty.
   Both suites green inside the fresh 16/0/0/5 run. PASS.
5. Oracle genuineness (read from checkout, `mid_apply_rollback_test.rs`):
   - Lock 1 (lines 115-168): second unit `blocked/second.src` fails at
     `fs::create_dir_all(parent)` (`lib.rs:223`) because the parent slot
     is a plain file staged by the test; the FIRST unit's `fs::write`
     (`lib.rs:243`) has landed and its path is in `new_paths`
     (`lib.rs:225-227`) before the failure. Asserts `Err` (an `Ok` is a
     panic), error contains `rolled back 4 files`, `generated/first.src`
     gone, consumer byte-identical, blocker file intact (proves rollback
     does not over-delete). Non-vacuous. PASS.
   - Lock 2 (lines 178-233): after TWO successful writes (unit created,
     first consumer genuinely mutated `OLD-DEP` -> `NEW-DEP`), the second
     consumer fails at `fs::read_to_string` (`lib.rs:248`) on bytes
     `b"PRE \xff\xfe post\n"` (byte escapes, per repo rule). Asserts
     created unit removed, mutated consumer restored to its exact staged
     bytes, failing consumer untouched. Catches a rollback that only
     deletes created files. Non-vacuous. PASS.
6. Error text lock: both tests assert the exact substring
   `rolled back 4 files`; N = `snapshot.entries.len() + new_paths.len()`
   documented at `rolled_back_error` (`lib.rs:283-295`). Rollback failure
   propagates the rollback error with the original cause as context -
   an unrestored tree is never reported as rolled back. PASS.
7. Plan-literal: handoff invariants held - snapshot captured BEFORE any
   mutation (`lib.rs:178` precedes `write_all_targets`); on ANY phase
   error all written targets restored via the same
   `TransactionApplier::rollback` the callers use; success path moved
   verbatim (diff shows a pure extraction, only reference-form
   adjustments `&gen_dir`->`gen_dir`, `&move_removals`->`move_removals`
   - behavior-identical); no retained dual representation (the inline
   phases are gone from `apply_with`). PASS.
8. TDD RED evidence: journal records `0 passed / 2 failed / 0 ignored /
   1 suites` with the specific raw causes
   (`Cannot create a file when that file already exists. (os error 183)`,
   `stream did not contain valid UTF-8`). Reviewer independently derived
   both exact failure modes from the pre/post code paths, so the RED
   digest is internally consistent with what the injected failures must
   produce. (Replaying RED requires mutating the fix - write git denied -
   so journal evidence + code-derived consistency is the ceiling here.)
9. Report-evidence (distillation contract): journal is digest-shaped -
   quoted digest lines, exit codes, capture paths
   (`target/testrun-capture.txt`), zero raw dumps; gates via distilling
   entry points (`testrun.py --distill`, staging via `distill.py --`).
   PASS.

## Deviation dispositions

1. **Handoff line citation wrong; pattern mirrored from callers instead** -
   ACCEPTED. Verified independently: `lib.rs:792-834` at HEAD is the test
   module (fixture helpers at 790-796, `#[test]` functions at 809-835).
   The tier-failure pattern the handoff meant lives at the callers:
   `crates/cli/src/optimize.rs:410-411`
   (`if !validation_passed { TransactionApplier::rollback(&snapshot,
   &new_paths)?; }`) and `crates/cli/src/commands/transform.rs:412`
   (`report_rollback` calling `TransactionApplier::rollback`). The
   implementer mirrored the real pattern as the objective required.
2. **Error decoration via anyhow `.context()`** - ACCEPTED. The handoff
   only pins "error text gains a rolled back N files line"; the context
   form puts that line on `to_string()` (locked by both tests) while the
   original cause stays in the anyhow chain.
3. **Rejected oracle alternatives (NUL paths, dir-replacement, read-only
   attrs) documented in "Deliberately not done"** - ACCEPTED on merits:
   the chosen injections are deterministic cross-platform IO errors that
   leave no state blocking restoration; the rejected dir-replacement
   oracle would have broken rollback's own re-write path.

## Non-blocking observations (recorded, not findings)

- Callers still run their own rollback after `apply_with` returns `Err`;
  this now double-restores against an already-restored snapshot.
  `rollback` is idempotent (`path.exists()` guards, byte-identical
  rewrites) and the journal records this deliberately. Left as-is.
- `crates/ui/src/server.rs` R1/R2 hygiene hits are pre-existing and
  outside AR-04; they belong to whatever handoff owns the current dirty
  ui work, not to the ledger deferred-findings triggered by this verdict.

## Follow-ups

None required. Scope of this verdict is the AR-04 commits only; the
milestone-wide workspace gate remains the orchestrator's per the
handoff-sizing rule (see `tasks/ar-04-execution-results.md`).
