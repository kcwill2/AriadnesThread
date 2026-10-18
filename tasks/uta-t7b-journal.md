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
(row 12) seam extraction (journaled, behavior-preserving): index_progress_inputs
  -> Option<(done,total)> selecting TuIndex|TuStarted with the verbatim pair;
  index_progress_cb keeps the sole print_progress call (grep lock intact).
  Two pins: selection table exact (0,0 and 15/10 pass through unnormalized)
  + every non-TU event (CollateralFile, both LoadMilestones, both
  BootMilestones) yields None.
(row 7) seam extraction (journaled, behavior-preserving): render_not_found /
  file_path_map / render_symbol_report / render_dependencies_report /
  render_users_report; commands keep lookup order (file_map built only after
  a hit - no new ArtifactManager::init side effect on a miss) and print the
  rendered lines unchanged. Five pins: not-found single-line text, full
  report scaffold order with (none) placeholders, unmapped FileId 16-hex raw
  rows (definition + users), named-vs-dangling dependency rows, users
  mapped/unmapped/none arms.
(row 8) seam extraction (journaled, behavior-preserving):
  explain_fallback_file_id + resolve_explain_target (lookup, then fallback,
  then registration check, then verbatim bail; emission order untouched).
  Three pins: backslash spelling hashes as the normalized forward-slash form
  (and provably not as the raw string), verbatim bail text on an empty graph,
  and resolution order - registered exact/basename wins, fallback id only
  ever wins when it itself is registered (constructed crossed fixture).
Canonical line (scoped --lib, rows 4+7+8+12 landed): 160 passed / 0 failed /
  0 ignored / 1 suites (filter runs: pin_with_edges 2/0, includes 9/0).
(row 9) seam extractions (journaled, behavior-preserving):
  - expose_warning_text split (warning bytes out of the eprintln call site;
    warn_if_expose_used stays the single emitter, "warning:"/"--expose"
    literals kept inside the ui.rs production section for the S6 grep lock).
  - drive_stage_frames(ProgressFn, &mut dyn FnMut(StageCall)->StageOutcome):
    nine-frame emission order + per-call short-circuit moved behind a stub
    advance; run_pipeline_stages is a thin adapter (the five ensure_*/
    get_or_build_reduced calls, byte-identical frames).
  - boot_progress_tests: seam_stage_frames() drives the REAL seam once with
    the pinned Load-arm events; stage_one_to_four_frames and
    stage_five_calibrated_frames both derive from it (hand-copied frames
    gone; the 13-globals stage-5 assert now checks derived data).
  - pins: exact nine-frame shape + five-call order + model passthrough;
    short-circuit at every stage (frames left + calls consumed); expose
    warning verbatim bytes (0.0.0.0 + trimmed 10.0.0.5) and the two None
    arms; proposal_basename whole-path fallback ("", "..", ".").
  DEVIATION (forced): the frozen ratchet
    boot_progress.rs::test_ui_rs_is_under_the_extraction_ceiling demands
    ui.rs <= 880 lines (base 877). The seam therefore lives in
    ui/boot_progress.rs (the sanctioned extraction target per the W-2a F2
    family-move + stage_spans precedents), re-exported into ui via the
    existing boot_progress use line. ui.rs 877 -> 842 lines.
  Small cleanup: moved test file cli_command_tests.rs lost one stale `mut`
  (pre-existing warning, surfaced post-move).
Canonical line (scoped --lib, rows 1-9,12): 167 passed / 0 failed / 0
  ignored / 1 suites.
(row 11) TEST-MODULE MOVE: extraction_apply.rs sat at 999/1000 - its inline
  `mod tests` (703 body lines) moved to ui/extraction_apply/tests.rs
  (byte-safe script, per-line terminators preserved; `mod tests;` now
  2 lines, source file 999 -> 295 lines). `super`/`super::super` semantics
  unchanged. Deviations inside the move: the two include_str! paths gained
  one `../` (file-relative), and one pre-existing unused-`state` warning
  became `_state` (same one-char cleanup class as the cli_command_tests
  mut). New pin green 1st run: `pin_failed_transaction_burns_the_token_and_
  reports_failed_text` - a `.rs` target fails the core routing (matches the
  row-5 unclaimed-backend family), Failed carries non-empty ASCII text, the
  token is burned at consumption, and the same token retries as NotApproved.
  Frozen greps intact: extraction_apply.rs 1 `apply_plan_with(`, transform.rs 2.
(row 10) one pin green in extraction_reader.rs mod tests (no src change):
  `pin_degraded_dry_run_is_honest_absent_and_never_cached` - reader over an
  uncanonicalizable repo root: unknown plan -> None (404), shipped plan ->
  the exact W3 honest-absent DTO (claim-free, copy_only false,
  includes_unavailable true, no token, epoch verbatim), and the discovery
  counter moves 1 -> 2 across two calls (never cached, R3 retryable).
Canonical line (scoped --lib, rows 1-12): 169 passed / 0 failed / 0
  ignored / 1 suites.
