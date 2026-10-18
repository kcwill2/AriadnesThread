# UTA-T4 Handoff - transform + validate: provenance bug fix, write containment, tier/rollback coverage

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. docs/specs/unit-test-audit.md (UTA-B1, write-containment row, validate findings).
3. crates/transform/src/placement.rs (full, esp. 95-160 and call sites in
   engine/), crates/transform/src/engine/mod.rs:120-160 + tests.rs:200-230,
   crates/transform/src/engine/write.rs (full), crates/validate/src/lib.rs
   (full; esp. 283-370, 400-430).
4. Model: existing rollback tests in validate/src (find the snapshot test to mirror).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/uta-t4-journal.md (journal; always in-set)
- crates/transform/src/**, crates/validate/src/**
You MUST NOT touch: crates/cli (its transform/validate callers), pipeline,
Cargo.toml/lock. No `git stash`, no branch change.

## What to build
FIXES (red test first, each):
1. UTA-B1 placement.rs compute_rewrite (107-116): an old_include is "covered"
   only by provenance, not blanket. Investigate the real call sites in
   engine/ to find the reachable symbol->origin mapping (GeneratedUnit symbols,
   plan data). Minimal honest fix: only mark old_inc covered when provenance
   shows the referenced symbol came from it; unmapped old includes land in
   side_effect_preservations. Red fixture: consumer references mapped
   `ThingConfig` + unmapped `Other` over old includes [legacy.h, other.h] ->
   legacy.h/other.h each preserved iff no provenance. If provenance is NOT
   derivable from available data shapes, STOP: journal BLOCKER with the
   analysis and proposed data-shape change; ship the other items.
2. engine/write.rs: reject unit.path that is absolute or escapes output_dir
   (contains `..` above dir) -> Err with the offending path. Red tests both shapes.

REQUIRED TESTS (pins, no behavior change):
3. Fix the disjunctive assert in engine/tests.rs:211: A<->B cycle fixture
   asserts errors == ["Cycle detected in generated headers"]; diamond
   (A->B,A->C,B->D,C->D) -> no error. (Edit to existing test allowed here:
   the disjunction itself is the defect; journal-justify.)
4. TierValidator::validate_structural / validate_generated_headers: empty/errs
   ExtractionResult -> Pass/Fail table; dep present under dir / under parent /
   nowhere (missing -> Fail, no-parent PathBuf::new() branch).
5. collect_move_removals guard table: old_dep==consumer skip, generated-target
   skip (self-strip guard), missing-file skip, empty-symbols skip, multi-unit-per-file.
6. rolled_back_error when rollback itself fails (RO file) carries original cause;
   verify_restored mismatch -> Err("State mismatch") (mutate snapshot file externally).
7. write_to_disk mid-batch IO failure: partial-write report behavior pinned.
8. ConsumerRewrite::is_noop order-sensitivity pinned as
   `pin_is_noop_is_order_sensitive` (report; semantics decision owed).
9. detect_cycles 3-unit cycle; dep-not-a-unit edge case.

## RGB protocol
Per item: Red (capture), Green, Refactor. NEVER weaken assertions.

## Hygiene (review-enforced)
<100-line functions, <1000-line files, banned-name ban, no single-letter locals;
`python tools/hygiene_check.py` exit 0.

## Frozen locks
Existing tests are contracts. Only item 3 and item 1/2's bug-locking tests may
edit existing tests, journal-justified; reviewer adjudicates.

## Gates (exact commands, timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-transform` (red then green)
2. `python tools/testrun.py --distill -- -p ariadne-validate`
3. `python tools/hygiene_check.py` -- exit 0.
4. `python tools/encoding_guard.py --staged` -- exit 0.
5. Commit: `python tools/commit_scoped.py --scope "crates/transform/**" --scope "crates/validate/**" --scope "tasks/uta-t4-journal.md" -m "fix(transform): uta-t4 include provenance + write containment; test(validate): tier/rollback coverage"`

## Discipline clauses
- UTF-8: Write/Edit only, pure-ASCII.
- Touch only the touch-set; out-of-scope: report.
- No bare cargo; distilling entry points only.
- 1-3 line journal checkpoint per Green gate to tasks/uta-t4-journal.md.
- Context exhaustion imminent: STOP + disk-state inventory.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence. 3. Green distill header +
canonical line + capture path. 4. Deviations. 5. Out-of-scope observations.
Never paste raw tool output.
