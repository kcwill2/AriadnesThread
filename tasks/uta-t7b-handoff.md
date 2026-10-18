# UTA-T7b Handoff - cli: dispatch, progress, query, registry behavior pins

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. docs/specs/unit-test-audit.md (cli rows; build on H7a commits - tree at its landed state).
3. crates/cli/src/ui.rs:120-140,350-380,600-660,730-760 (build_file_collateral,
   proposal_basename, warn_if_expose_used, run_pipeline_stages),
   progress.rs (full), boot_progress.rs:90-110, plan.rs:90-110, registry.rs
   (full), query.rs (full), includes.rs:130-150, index.rs:25-45,
   lib.rs:340-410, extraction_reader.rs:290-320,410-430,
   extraction_apply.rs:255-290.
4. Model: existing LoadMetric fixture helpers in crates/cli (grep LoadMetric).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/uta-t7b-journal.md (journal; always in-set)
- crates/cli/src/** EXCEPT optimize.rs/transform.rs/measure.rs/discover.rs
  src logic (H7a owns those; test-module additions there also OUT to keep hands disjoint)
- crates/cli/tests/**
You MUST NOT touch: other crates, Cargo.toml/lock. No `git stash`, no branch change.
Minimal pure-fn extraction (e.g. render-to-String, residual of a tick body) is
allowed in the listed files; journal each extraction; no behavior changes.

## What to build (ALL PINS; red row = report, do not fix)
1. build_file_collateral: LoadMetric fixture - symbol_count==0 dropped,
   target_file None skipped, key = id string.
2. progress.rs: format_progress total==0 -> 100% + full bar; current>total
   bounds (bar width + cap at 100%) pinned as `pin_progress_overshoot_clamped`
   (if unclamped today, pin exactly + report); strip_stage_prefix edge table
   ("[12]", "[1/2]x", "[ ]", unclosed).
3. boot_progress panic_payload_text: String payload and non-string payload arms.
4. plan.rs colocated_groups_with_edges: per-language edge filtering; empty
   edges byte-equal to colocated_groups.
5. registry.rs: parse_language_arg case matrix pinning EXACT accepted set
   ("CPP"/"sCaLa" outcomes named explicitly - test name must not claim
   case-insensitive if it is not); plan_backend both honest errors
   (`.rs` target unclaimed; capability-off message).
6. lib.rs dispatch table (4 cases each of dispatch_analyze/_includes/_plan):
   sub-repo-over-positional precedence + (None,None) bail. The asymmetric
   precedence pins as `pin_subrepo_positional_precedence_silent`; report.
7. query.rs: extract render-to-String seams; not-found prints + Ok(()), empty
   deps, unmapped-FileId arms.
8. includes.rs explain fallback: `sub\\a.h` normalizes to graph id; unknown
   file bails.
9. ui.rs: proposal_basename component-less fallback; warn_if_expose_used
   text-fn split - blank value no warning, value warns; run_pipeline_stages:
   seam so the REAL function is replayed (current test replays a hand-written
   copy) - refactor the test, journal.
10. extraction_reader dry_run degraded path: inject discovery failure ->
    honest-absent shape + second call re-runs discovery (never cached).
11. extraction_apply Failed arm: failing transaction seam -> ExtractionApplyRefusal::Failed
    text, token already burned.
12. index.rs index_progress_cb: TuStarted and TuIndex events render identically.
13. Non-asserting test cleanup: lib.rs test_handle_analyze tautology and
    test_handle_measure is_ok-only - strengthen with real asserts or journal
    why the seam cannot support them (editing these two tests is in-set).

## RGB protocol
Pins: write test; if red reveals unlisted wrong behavior, pin exact current
behavior + report; do not fix.

## Hygiene (review-enforced)
<100-line functions, <1000-line files, banned-name ban, no single-letter locals;
`python tools/hygiene_check.py` exit 0.

## Frozen locks
Grep-lock tests (source-text asserts) must keep passing verbatim; if a seam
extraction moves locked text, report BEFORE moving.

## Gates (exact commands, timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-cli`
2. `python tools/hygiene_check.py` -- exit 0.
3. `python tools/encoding_guard.py --staged` -- exit 0.
4. Commit: `python tools/commit_scoped.py --scope "crates/cli/**" --scope "tasks/uta-t7b-journal.md" -m "test(cli): uta-t7b dispatch/progress/query/registry behavior pins"`
   (Note: scope covers crates/cli/** but commit ONLY after H7a has landed;
   stage selectively with commit_scoped - it commits only matching paths not
   yet committed.)

## Discipline clauses
- UTF-8: Write/Edit only, pure-ASCII.
- Touch only the touch-set; out-of-scope: report.
- No bare cargo; distilling entry points only.
- 1-3 line journal checkpoint per Green gate to tasks/uta-t7b-journal.md.
- Context exhaustion imminent: STOP + disk-state inventory (order: 1,2,3,5,6,13 first).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Per-row distill header + canonical line
+ capture path. 3. Rows stopped/pinned-with-report. 4. Out-of-scope observations.
Never paste raw tool output.
