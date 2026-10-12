# AR-19 Handoff - Dispatch planning via registry; drop cpp hardwires (A1 part 2)

Wave 6. Crate (`cli` + lang-cpp removal + integration lock). Depends: AR-18
(APPROVED pending — verify landed `d937ac0` + verdict before starting).

## Objective

Replace the hardwired C++ planning imports with plugin dispatch through the
AR-18 seam:
1. `crates/cli/src/optimize.rs` (~:102 `use ariadne_lang_cpp::...`) — import
   removed; planner consumes `aggregation_filter`/`target_path_for` from the
   registry-resolved plugin.
2. `crates/cli/src/commands/ui.rs` `build_extraction_proposals` (~:390-400) —
   same switch (the positional cpp fn references go away).
3. `crates/lang-cpp/src/plan.rs` — if it becomes dead after the switch, remove
   (journal the dead-code determination; if still referenced elsewhere, keep
   and say so).
4. `plan.rs` 2-arm language match deleted (per plan: the CLI plan path's
   cpp/scala arms collapse into registry dispatch).

## Impact

`tool optimize` and UI extraction proposals become CORRECT for any
declared-planning language — Scala stops silently yielding empty/incorrect
proposals. First user-visible functional fix of Wave 6.

## Touch-set (hard)

- `crates/cli/src/optimize.rs`, `crates/cli/src/commands/ui.rs`
- `crates/cli/src/registry.rs` if a helper is needed for hook resolution
- `crates/lang-cpp/src/plan.rs` (removal or justified keep)
- NEW integration lock: `tests/integration/tests/m24a_scala_plan_path/...` or
  sibling mod — Scala repo fixture demonstrates a NON-EMPTY, correct plan via
  the plugin path (m24-adjacent naming; check tests/integration/tests/
  common for scala fixture availability; lang-scala tests have scala sources
  to model a mini-repo on)
- `tasks/ar-19-handoff-journal.md`

## Constraints

- TDD: Scala plan-path lock RED first (demonstrates the silent-empty bug
  through the CLI/registry path), GREEN after dispatch switch. C++ behavior
  byte-identical — m00 parity + extraction proposals goldens UNEDITED green.
- STRICT COMPILE-GREEN (shared worktree): switch call sites atomically per
  file; cargo check before tests.
- Pure ASCII; commit_scoped: cli + lang-cpp (if removed) + integration +
  journal.

## Gate commands

1. `-p ariadne-cli` (baseline 118/0/8/6 + yours)
2. m00 golden + facts-path parity UNEDITED
3. new scala plan-path suite green; m18 parity + m24 identity UNEDITED
4. `python tools/testrun.py --distill -- -p ariadne-lang-cpp` IF plan.rs was
   removed (expect test count drop matching removal, journal reconciles)
5. `encoding_guard --all`; `hygiene_check.py` (only server.rs 2 rows)

## Invariants / locks

- C++ `tool optimize` output byte-identical (m00 parity is the authority).
- `ariadne_lang_cpp::cpp_aggregation_filter`/`cpp_target_path_for` may be
  REMOVED from cli's dependency graph for planning purposes — check whether
  the Cargo dep survives for other uses (journal either way).
