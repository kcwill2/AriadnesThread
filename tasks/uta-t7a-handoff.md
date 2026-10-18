# UTA-T7a Handoff - cli: error-swallow and residual-arithmetic fixes

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. docs/specs/unit-test-audit.md (UTA-B7, UTA-B8).
3. crates/cli/src/optimize.rs (full: 140-180 commit loop, 440-480),
   crates/cli/src/transform.rs:220-245, crates/cli/src/measure.rs:1-40,
   crates/cli/src/discover.rs:70-100, crates/cli/src/lib.rs:340-410 (dispatch).
4. Model: existing error-propagation style in crates/cli/src/extraction_apply.rs.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/uta-t7a-journal.md (journal; always in-set)
- crates/cli/src/optimize.rs, crates/cli/src/transform.rs,
  crates/cli/src/measure.rs, crates/cli/src/discover.rs (+ their test modules),
  tests in crates/cli/tests/ if needed
You MUST NOT touch: other src files in cli (H7b hand owns them), other crates,
Cargo.toml/lock. No `git stash`, no branch change.

## What to build
FIXES (red test first, each; keep each fix surgical):
1. UTA-B8 optimize.rs residual_collateral_load (457-459): residual must be
   computed from the actually-committed plans' reductions, not
   `take(committed_transformations)` prefix assumption. Track committed plans
   in the commit loop (159-168). Red fixture: 3 plans, middle one fails
   validation -> residual uses plan1+plan3 reductions (current code uses plan1
   only). Extract the residual math into a pure fn if needed (journal).
2. UTA-B7a optimize.rs save_optimization_report (471-473): serde failure and
   fs::write failure must produce Err (or a surfaced warning + documented
   contract) - never a silent Ok(()) with no report written. Red: read-only/
   unwritable target dir -> the fn must not report unqualified success.
   Choose Err; journal.
3. UTA-B7b transform.rs load_plan_for_action (230-233): a failing replan must
   surface its own root-cause error, not be erased to empty vec (context-chain
   the reload error with the replan error text). Red: inject failing replan seam
   -> error message contains the replan cause.
4. UTA-B7c measure.rs:11 / discover.rs:87 `discover().ok()`: a discovery
   failure must NOT yield an empty-file fingerprint. Measure path: propagate
   Err (command fails loudly). discover.rs site: journal what it gates, apply
   the same no-silent-empty rule. Red: corrupt/unreadable root scenario ->
   observable error, not Ok over empty hashes.

## RGB protocol
Per item: Red (capture failing list), Green, Refactor. NEVER weaken assertions.

## Hygiene (review-enforced)
<100-line functions, <1000-line files, banned-name ban, no single-letter locals;
`python tools/hygiene_check.py` exit 0.

## Frozen locks
Existing tests are contracts; edit only tests that literally lock the old
swallow/prefix behavior (journal-justify each). Grep-lock tests (source-text
asserts in progress/boot/extraction files) must keep passing verbatim - if a
fix changes a locked source string, report BEFORE editing the lock.

## Gates (exact commands, timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-cli` (red then green)
2. `python tools/hygiene_check.py` -- exit 0.
3. `python tools/encoding_guard.py --staged` -- exit 0.
4. Commit: `python tools/commit_scoped.py --scope "crates/cli/src/optimize.rs" --scope "crates/cli/src/transform.rs" --scope "crates/cli/src/measure.rs" --scope "crates/cli/src/discover.rs" --scope "crates/cli/tests/**" --scope "tasks/uta-t7a-journal.md" -m "fix(cli): uta-t7a committed-residual arithmetic + surface swallowed errors"`

## Discipline clauses
- UTF-8: Write/Edit only, pure-ASCII.
- Touch only the touch-set; out-of-scope: report.
- No bare cargo; distilling entry points only.
- 1-3 line journal checkpoint per Green gate to tasks/uta-t7a-journal.md.
- Context exhaustion imminent: STOP + disk-state inventory.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence per item. 3. Green distill
header + canonical line + capture path. 4. Deviations. 5. Out-of-scope observations.
Never paste raw tool output.
