# UTA2-R5 journal -- sweep-A remainder pins

Base: main at 59abbf0 (uta2-r4 tip), clean tree. Handoff:
tasks/uta2-r5-handoff.md. Discipline: pins only, zero src-logic changes,
pure ASCII (byte escapes), distilling entry points throughout.

## Gate 1 -- ariadne-graph (GREEN)

Red/Green note: pins for a zero-coverage but existing function; the pin
contract predicts green-on-run (a red would expose unlisted wrong behavior --
none did). Four pins, new file
`crates/graph/tests/uta2_r5_aggregation_consumers_test.rs`:

- `transitive_consumer_listing_covers_the_whole_chain` -- 3-file chain
  tu->hdr->mid: mid lists the TU although the TU only reaches it
  transitively; map size exactly 2.
- `consumer_vectors_are_sorted_by_file_id_regardless_of_input_order` -- two
  TUs, descending input, ascending vectors (the internal HashSet iteration
  order is hash-seed-random; the sort is the only stabilizer).
- `tu_id_missing_from_the_closure_map_is_skipped` -- unknown id (no edges, no
  registration) contributes nothing: map equals the without-unknown map, id
  surfaces neither as key nor in any consumer vector.
- `empty_input_maps_to_empty_output` -- edge-present graph and empty graph
  both map `[] -> {}`.

Canonical line: `174 passed / 0 failed / 0 ignored / 12 suites`
(capture: target/testrun-capture.txt). Suite 12 = previous 11 + the new file.

## Gate 2 -- ariadne-lang-cpp (GREEN)

Two new inline test modules (src changes: two `#[cfg(test)] mod` declarations
only; no logic touched). Missing-arm lock
(`probe_of_absent_binary_is_missing_and_graceful`) byte-untouched.

`crates/lang-cpp/src/tokenizer/raw_string_multibyte_tests.rs` (5 pins):

- `raw_string_open_reads_byte_offsets_through_a_multibyte_prefix` -- handoff
  specimen `...e-acute...R"( { } )"` built from byte escapes (`0xC3 0xA9`):
  raw open reads byte offsets through the two-byte char; glued
  e-acute+R variant proves continuation byte 0xA9 upstream disturbs nothing;
  leading guard pinned directly: `raw_string_open("\"abc", 0) == false`
  (no predecessor index).
- `multibyte_before_raw_string_leaks_no_brace_tokens_and_keeps_char_columns`
  -- exact token stream (no `punct:{`/`punct:}` leak) and trailing token at
  line 1 COLUMN 19 -- a byte-counted column would say 20; this is the
  char-column pin.
- `glued_multibyte_prefix_run_is_dropped_into_the_raw_string_wholesale` --
  documented pending-run drop rule: the `e-acute + R` run disappears into the
  raw opener; honest pin, quirks preserved not unified.
- `multiline_multibyte_raw_string_keeps_line_and_char_columns` -- line 2,
  char column after multi-line raw content with the multi-byte char upstream.
- `header_guard_of_scan_raw_string_with_fallback_falls_back_to_the_plain_swallow`
  -- the handoff marked the guard REPORT-only "unless directly testable":
  it IS directly reachable from this inline module (the sole production route
  pre-validates the header via `raw_string_open`, so the guard is deliberate
  paranoia), so it gets a pin. Position landing (byte=6, char=6, line=1,
  column=7) equals the plain `scan_string_literal` swallow of `"x"`. The
  owner's debug_assert-vs-delete call stays open; the pin just locks
  whichever fallback semantics the decision must preserve.

`crates/lang-cpp/src/plugin/plugin_probe_arm_tests.rs` (3 pins):

- `available_arm_reports_the_first_nonempty_line_of_a_real_banner` --
  `cmd.exe --version` prints its banner to stdout, exit 0; probe must carry
  exactly the first trimmed non-empty line (expected value recomputed from an
  independent spawn, no machine-dependent literal).
- `a_runnable_binary_with_empty_stdout_falls_back_to_the_binary_string` --
  `findstr.exe --version` (empty stdout, complaint on stderr only, exit 2):
  spawn-Ok + empty banner => `Available("findstr.exe")`. Pinned honestly --
  never Missing, never a fabricated version.
- `probing_the_running_test_binary_lands_available_via_the_empty_stdout_fallback`
  -- DEVIATION (handoff expectation does not reproduce): the handoff expected
  this specimen to be `Available(libtest banner first line)`. On this
  toolchain (rustc 1.99.0) libtest REJECTS `--version`: the child prints
  "error: Unrecognized option: 'version'" to STDERR and panics (exit 101)
  while stdout stays EMPTY (host-verified). The probe never inspects exit
  status, so the spawn succeeds and the test binary lands on the empty-stdout
  fallback => pinned exactly `Available(current_exe_string)`. If a future
  libtest implements `--version` this pin goes red for the owner to re-pin;
  the behavioral invariant (never Missing/panic) holds either way.

Reds seen and disposition: (1) fixture authoring bug -- my first specimen
write dropped the trailing `...` of the handoff's `...e-acute...R"( { } )"`;
fixed the fixture, no semantics involved. (2) module-path, format-string, and
String/&str compile drift; all harness-level, zero src-logic changes.

Canonical line: `201 passed / 0 failed / 0 ignored / 2 suites`
(capture: target/testrun-capture.txt; 193 pre-existing + 8 new pins, all
green).

## Gate 3 -- ariadne-lang-scala (GREEN)

New inline module `crates/lang-scala/src/boundary/boundary_pin_tests.rs`
(boundary.rs gains one `#[cfg(test)] mod` declaration only):

- `external_path_markers_are_the_frozen_four_in_declaration_order` -- exact
  content AND order `["lib", "target", ".ivy2", ".cache"]`, no silent fifth.
- `every_frozen_marker_recognizes_its_exact_component` -- all four true.
- `near_miss_components_never_classify_as_external_boundary` -- 14-row false
  table incl. the handoff's named rows (`targets`, `LIB`) plus case variants,
  superstrings/truncations, dot-absence (`cache`, `ivy2`), and whitespace
  padding.

Canonical line: `257 passed / 0 failed / 0 ignored / 2 suites`
(capture: target/testrun-capture.txt; 254 pre-existing + 3 new pins, green on
first run -- no unlisted wrong behavior exposed).

## Gate 4 -- ariadne-cli (row-5 verification) -- VERDICT: PARTIAL COVERAGE, test added

Verification survey evidence (read-only):

- Action arms in `crates/cli/src/commands/transform.rs`:
  `extract_plan_headers` (:31), `build_migration_preview` (:81),
  `apply_transform_plan` (:129).
- H7a/T7b seam coverage found: `cli_command_tests.rs` covers the three arms'
  CLAP PARSE shapes only (`test_cli_transform_extract`,
  `test_cli_migrate_preview`, `test_cli_apply`); T7b's route-seam pins route
  analyze/includes/plan dispatch -- no transform arm; T7b's transaction pins
  drive the shared core (`apply_plan_with`) through the UI route.
- Behavioral Err-arm coverage exists for two arms in the m20 integration
  characterization (outside H7a/T7b):
  `migrate_unknown_plan_id_reports_not_found`,
  `apply_unknown_plan_id_reports_not_found`,
  `apply_and_migrate_missing_repo_path_report_locate_failure`,
  `migrate_regenerates_missing_plan_artifacts_and_succeeds`.
- `extract_plan_headers`: ZERO behavioral coverage anywhere (repo-wide grep:
  only the lib.rs dispatch call site) -- no success or Err-arm test touches
  it.

Verdict: the arms are NOT already covered per the handoff's condition (the
third arm has no behavioral coverage at all, and the m20 rows live in a
different crate's suite). Per the handoff's "if no" branch, added ONE
table-driven arm test in the transform.rs inline test module:
`every_action_arm_reports_honest_err_without_a_repository_or_a_plan` drives
all three arms through two scenarios: (1) unlocatable repo path -> frozen
"Failed to locate repository path" text per arm (mirrors the m20 contract,
extract now joins it); (2) locatable artifact-free temp repo -> honest Err
with a real message per arm, never a vacuous Ok. Test-module-only; no src
logic.

Canonical line: `178 passed / 0 failed / 8 ignored / 8 suites`
(capture: target/testrun-capture.txt; 177 = T7b closeout count, +1 new test;
the 8 ignores are the pre-existing bench fixtures).
Compile drift fixed first: `Option::unwrap_or_else` closure arity (harness).

## Closeout

- Deliberately NOT done: no src-logic changes anywhere (STOP branch never
  needed -- every expectation pinned existing behavior once fixtures matched
  the handoff specimens); no existing lock touched (all four crate diffs are
  pure additions: new files + `#[cfg(test)] mod` declarations + one appended
  test fn in transform.rs' inline module); no m20 integration rows edited
  (outside this hand's touch-set -- observation, not duty).
- Deviations (both reported, zero logic): (1) the handoff's libtest-banner
  expectation for the current_exe probe does not reproduce on rustc 1.99.0
  (see Gate 2) -- pinned exactly as landed; (2) the raw-multibyte specimen
  was briefly authored missing its trailing `...` -- fixture fixed to the
  handoff text, token pins unchanged.
- Out-of-scope observations for the owner: (a) `scan_quote` drops the WHOLE
  pending alphanumeric run before a raw open, so a glued `e-acute R"( ...`
  loses the e-acute too (documented rule; pinned, not unified). (b)
  `probe_compiler_status` never inspects exit status -- a crashing binary
  with stdout still reports Available (by documented contract; the
  findstr/current_exe pins make this visible). (c) `extract_plan_headers` has
  no replan fallback (direct `load_plan_artifacts()?`), unlike
  `load_plan_for_action` -- honest Err but asymmetric arm behavior, worth an
  owner look someday.
- Gate witnesses (verbatim): graph
  `174 passed / 0 failed / 0 ignored / 12 suites`; lang-cpp
  `201 passed / 0 failed / 0 ignored / 2 suites`; lang-scala
  `257 passed / 0 failed / 0 ignored / 2 suites`; cli
  `178 passed / 0 failed / 8 ignored / 8 suites`. Hygiene: R1/R2 blocking
  clean, all R3 warns pre-existing outside this hand's files. Encoding:
  authored content pure-ASCII (byte-scan witnesses); `encoding_guard` exit
  records in the closeout commit.
