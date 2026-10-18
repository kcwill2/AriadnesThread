# UTA-T7b Journal - cli dispatch/progress/query/registry behavior pins

Handoff: tasks/uta-t7b-handoff.md. Base: main @ ff47047 (H7a landed).
Order followed: rows 1,2,3,5,6,13 first, then 4,7,8,9,10,11,12.

## Planned seam extractions (each journaled at Green)

- lib.rs: pure route functions for dispatch_analyze/_includes/_plan so the
  repo-path decision table is testable without executing heavy commands.
  Dispatch bodies keep every bail string verbatim.
- query.rs: render-to-String seams for the three query commands.
- includes.rs: fallback FileId seam for the explain normalization.
- index.rs: progress-input extraction for index_progress_cb.
- ui.rs: expose-warning text function split (warn_if_expose_used) and the
  stage-frame skeleton seam (run_pipeline_stages) so the REAL emission
  order can be replayed in tests.
- extraction_apply.rs: test module moved to ui/extraction_apply/tests.rs
  because the file sits at the 999/1000-line ceiling (998 after the move);
  the new row-11 pin cannot fit inline without breaking the ceiling.

## Deviations (appended as they occur)

- (pending row 2) pin name: handoff mandated `pin_progress_overshoot_clamped`
  but `format_progress` is NOT clamped when current > total (pct 150, 30-cell
  bar). Per the ALL-PINS honest-naming rule (same rule the handoff states for
  row 5), the pin is named `pin_progress_overshoot_unclamped`. REPORTED.
- (pending row 5) legacy test `parse_language_arg_accepts_serde_names_case_
  insensitively` overclaims in its NAME (it only feeds lowercase + exact
  PascalCase, and passes); it is left untouched (in-set but not directed;
  its assertions stay true) and the honest full matrix pin is added next to
  it. REPORTED.

## Baseline (pre-change)

- T7a landed canonical line: `147 passed / 0 failed / 8 ignored / 8 suites`.

## Green checkpoints (1-3 lines per row)

(row 1) `pin_build_file_collateral_drops_zeroes_skips_unmapped_and_keys_by_id_string`
  green 1st run in ui/tests.rs; LoadMetric fixture built by hand (no new
  src change; build_file_collateral untouched).
(row 2) three pins green in commands/progress.rs `mod tests`: total==0 ->
  100% full bar; overshoot UNCLAMPED (bar 30 cells, 150%) pinned exact ->
  REPORTED with honest name (handoff name said "clamped"); strip_stage_prefix
  edge table [12]/[1/2]x/[ ]/unclosed pinned.
(row 3) `pin_panic_payload_text_string_and_foreign_payload_arms` green;
  String/&str/foreign(42u32 -> "unknown panic payload") arms pinned.
(row 5) two pins green in registry.rs mod tests. CPP and sCaLa REJECTED
  (only exact lowercase + exact serde names parse) -> REPORTED; the legacy
  `..._case_insensitively` name overclaims, left untouched (its assertions
  are true), honest matrix added beside it. plan_backend both honest errors
  pinned (unclaimed `.rs` names target+stage; `requires_toolchain` is the
  honest capability-off bit - both shipped plugins keep it false).
(row 6) seam extraction (journaled, behavior-preserving): AnalyzeRoute /
  IncludesRoute / PlanRoute + route_analyze/_includes/_plan; dispatchers are
  thin executors, all three bail strings verbatim, single-letter `r`/`w`
  locals replaced while rewriting those bodies. Five pins green
  (cli_command_tests.rs). ASYMMETRY REPORTED: analyze Load compares the two
  paths (`repo_path != load_repo`) while includes/plan drop the positional
  via a wildcard arm WITHOUT comparing; result identical (subcommand repo
  silently wins, no warning anywhere).
  TEST-MODULE EXTRACTION: lib.rs inline `mod tests` moved verbatim to
  src/cli_command_tests.rs (only the two row-13 strengthenings differ)
  because route seams + pins would push lib.rs past the 1000-line ceiling.
  lib.rs 834 -> 550 lines. Test names unchanged (no external referent -
  grep showed only historical report files).
(row 13) test_handle_analyze: tautology replaced by total_tus == 2 (fixture
  ships src/thing.cpp + tests/consumer.cpp), reused+reanalyzed==total,
  tu_analyses.len()==total - all green. test_handle_measure: is_ok-only
  replaced by field assertions - tu_measurements empty, affected_tus_count
  Some(0), collateral_load None (no fabricated zero), repo_root ends with
  the located dir - all green.
Canonical line (scoped --lib): 151 passed / 0 failed / 0 ignored / 1 suites.
(row 4) two pins green in commands/plan.rs mod tests (no src change): empty
  extends edge list == colocated_groups byte-for-byte (non-empty fixture, so
  the equality is not vacuous); per-language edge filtering pinned - a mixed
  scala/cpp fixture with a sealed Shape, intra-Scala extends edges + one
  cross-language edge (cpp Widget -> scala Shape) yields exactly the
  [Circle, Shape, Square] group, and the cross edge ALONE colocates nothing.
  Helpers routing_over/group_id_strings/mixed_scala_cpp_catalog added in the
  test module only.
