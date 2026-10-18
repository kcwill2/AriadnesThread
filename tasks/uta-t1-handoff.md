# UTA-T1 Handoff - graph: traversal surface, deterministic lookup, iterative cycles

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. docs/specs/unit-test-audit.md (sections: UTA-B4, UTA-B5, adjudicated pins, known-open).
3. crates/graph/src/traversal.rs (ReducedSemanticModel surface), graph.rs:135-155,
   callgraph.rs:80-240, closures.rs:1-60 (frame-machine pattern to mirror),
   cycle_groups.rs (100k-chain stack test at tests/cycle_groups_test.rs:105).
4. Model files to mirror: crates/graph/src/closures.rs iterative explore_frames;
   crates/graph/tests/* oracle style.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/uta-t1-journal.md (checkpoint journal; always in-set)
- crates/graph/src/** (find_file_id, detect_recursion_cycles + test modules)
- crates/graph/tests/** (new test files only if cleaner than inline)
You MUST NOT touch: crates/cli (its analyze_file_load/explain callers), any other
crate, Cargo.toml/lock. No `git stash`, no branch change.

## What to build
FIXES (red pin test first, each):
1. UTA-B4 `find_file_id` (graph.rs:135): make resolution deterministic -
   full-path match wins over any filename match (two passes), and among equal
   filename matches pick the smallest FileId (FileId: Ord). Red fixture:
   register `a/util.h` and `b/util.h`; assert repeated calls and re-registered
   orderings return the same id; assert full path `b/util.h` resolves to the
   b entry even when `a/util.h` was registered first.
2. UTA-B5 `detect_recursion_cycles` (callgraph.rs:205): replace recursive DFS
   with the closures.rs frame-machine pattern (no recursion). Add fixtures:
   two-SCC graph (A<->B, C<->D, B->C) asserts ALL cycles enumerated once each
   in normalized order; 100k-deep chain must complete on a 64 KiB thread
   (mirror cycle_groups_test.rs:105 style).

REQUIRED TESTS (behavior pins, no code change unless red exposes a listed bug):
3. traverse_neighborhood: chain+self-loop+isolate fixture; direction x depth 0-2 snapshot.
4. shortest_dependency_path: A->C path, X==X -> [X], isolate -> None.
5. record_call_site with caller absent from model.symbols: pin current
   edge-without-caller-node shape as `pin_call_edge_without_caller_node`; report.
6. extraction_impact guard rows: root==source / empty justified set / unregistered -> None.
7. generate_report top-headers: 12 files with tied sizes + one edge-only id
   -> assert (size desc, path asc), take(10), {:016x} fallback.
8. format_explain_node: literal snapshot for a 2-cycle tree + 2-child node ((cycle) marker, prefixes).
9. explain_symbol_load/explain_file_load source_filter hit+miss; both "No ..." error strings.
10. get_entrypoints x3 (IncludeGraph, ReducedSemanticModel, CallGraph): tie-order,
    truncation, all-cycle graph (zero roots), unresolved-only -> leaf;
    parallel-directive degree inflation: pin as `pin_parallel_directive_edges_count_toward_degree`.
11. reduce(): extends_edges + sealed_symbols side channels asserted (sealed modifier fixture).
12. load.rs include_load==total_load: pin `pin_include_load_equals_total_load`; report.
13. closures: 100k-node cycle compute_closures on 64 KiB stack.
14. builder.rs:89-92 unreadable-file continue: report-only, no test required (needs diagnostic policy decision).
15. merge_tu_edges non-Complete-kind precedence both orders; find_symbol duplicate-name
    first-key pin; cycle_groups self-loop group external degrees; IncludeClosure::new empty/direct-only.

## RGB protocol (per numbered item, in this file order)
Red first (capture failing list), Green, Refactor. NEVER weaken an assertion to pass.

## Hygiene (review-enforced)
Functions < 100 lines, files < 1000 lines, no banned names (tmp/res/foo/bar/test1),
no single-letter locals. `python tools/hygiene_check.py` exit 0.

## Frozen locks
Existing tests are contracts: do not edit any existing test except one that
literally locks the OLD behavior of item 1 or 2 (justify in journal, reviewer adjudicates).

## Gates (exact commands, timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-graph` (red, then green; digest header + canonical line verbatim in report)
2. `python tools/hygiene_check.py` -- exit 0.
3. `python tools/encoding_guard.py --staged` -- exit 0 before committing.
4. Commit: `python tools/commit_scoped.py --scope "crates/graph/**" --scope "tasks/uta-t1-journal.md" -m "test(graph): uta-t1 traversal/callgraph coverage + deterministic lookup"`

## Discipline clauses
- UTF-8: Write/Edit tools only, pure-ASCII content (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- No bare cargo/clippy; distilling entry points only (docs/specs/tool-output-distillation.md).
- After each Green gate append a 1-3 line checkpoint to tasks/uta-t1-journal.md.
- If context exhaustion looms: STOP, return disk-state inventory; session is never resumed.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (first failing tests).
3. Green evidence (distill digest header + canonical line + capture path).
4. Deviations + reasons. 5. Out-of-scope observations (report-only).
Never paste raw tool output.
