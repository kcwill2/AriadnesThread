# UTA-T2 Journal - lang-cpp directive boundary fix + parser/locator/backend coverage

Handoff: tasks/uta-t2-handoff.md. Base: main @ 994e5c3 (UTA-T1 approved head).
Implementer session started 2026-10-18.

## Pre-flight (before any edit)

- Fixture/oracle dependence scan (handoff-mandated, item 1): grep across
  `tests`, `crates`, `docs`, `tools` for `#\s*include[A-Za-z0-9_]` (the
  non-boundary shapes) - hits only in docs/specs/unit-test-audit.md and
  tasks/uta-t2-handoff.md (the claim texts themselves). No fixture, no
  vendored m18 parity capture, no inline test specimen contains an
  `#include`-prefixed keyword run. Safe to proceed; nothing to force.
- Read all binding inputs per handoff read-list; model suites
  tokenizer/underflow_fix_tests.rs + parser/brace_depth_tests.rs inspected
  (byte-escape `specimen()` idiom adopted).

## Deviations (with reasons)

1. UTA-B3 "fix" not committed: the directed red never materialized (see
   checkpoint 1); the guard would be a behavior no-op and TDD forbids
   code without red. Boundary behavior pinned instead. Owner call needed to
   downgrade UTA-B3 in the audit ledger (CONFIRMED -> NOT REPRODUCIBLE).
2. Commit message wording: handoff text said "boundary fix"; the commit
   carries locks, no logic change, so "fix" was changed to "locks" for
   truthfulness. Scope and everything else verbatim.
3. Gate 2 command as written (`--testfilter m18` on `-p ariadne-lang`) is
   not valid cargo; the m18 parity suite lives in the
   `ariadne-integration-tests` package. Ran the handoff intent as
   `python tools/testrun.py --distill -- -p ariadne-integration-tests m18`
   -> 30 passed / 0 failed / 0 ignored / 37 suites (parity holds; hand is
   test-additive with zero src behavior change).
4. Encoding-gate sequencing: commit_scoped stages internally, so
   `encoding_guard --staged` cannot observe the window. Ran
   `encoding_guard --all` (superset, CI form) exit 0 pre-commit, plus
   `--staged` post-commit. Mirrors the UTA-T1 accepted sequencing.

## Report-flagged observations (report-only, no behavior touched)

- ParserCtx self-edge quirk: a definition at token index > 0 whose previous
  token is punctuation (e.g. `::f()` at stream start) records
  `f -> f Complete` via handle_variable_decl firing after
  handle_global_function set current_function. Pinned verbatim.
- Ctor-shape double record: `A a();` in a class yields BOTH
  Holder->A and Holder::a->A Incomplete edges (member-signature misfire).
  Pinned verbatim.
- symbol_locator `#else` after a disabled `#if 0` masks the #else branch
  too (compiler would see it). Conservative miss; pinned, owner decision.
- pin_backend_missing_source_analyzes_empty pins the spec-adjudicated
  backend.rs:189 empty-Ok swallow (loud name per handoff).
- analyze_tu (analyzer) on a missing path is Err, NOT empty-Ok - no pin
  of a swallow there; the lock documents the asymmetry with the backend.

## Gate record

1. `python tools/testrun.py --distill -- -p ariadne-lang-cpp`:
   red-attempt run `159 passed / 0 failed / 0 ignored / 2 suites` (predicted
   red absent), green `193 passed / 0 failed / 0 ignored / 2 suites`;
   capture C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt
2. m18 parity: `30 passed / 0 failed / 0 ignored / 37 suites` (see
   deviation 3 for command shape).
3. `python tools/hygiene_check.py` exit 0 (R1/R2 clean; the emitted R3
   warns are all pre-existing files, none from this hand's new modules).
4. `python tools/encoding_guard.py --all` exit 0 pre-commit; `--staged`
   exit 0 after staging (deviation 4).
5. `python tools/commit_scoped.py --scope "crates/lang-cpp/**" --scope
   "tasks/uta-t2-journal.md"` -> commit 55262e8 (message per deviation 2),
   18 files, +811/-0, all inside the touch-set.

## Checkpoints (one per green gate)

- Item 1 (RED ATTEMPT - did not materialize): added directive_parser/
  boundary_tests.rs (3 locks: `#includemy "a.h"` -> 0, `#include_next <x>` ->
  0, normal quoted/angled/tight controls parse). Scoped run on UNFIXED code:
  `159 passed / 0 failed / 0 ignored / 2 suites` - predicted red absent.
  Root cause of non-reproduction: line 28 slices `after_hash[7..]` and the
  remainder must start (after trim_start) with `"`/`<`; a word char at the
  boundary can never satisfy that, so UTA-B3's phantom include is
  UNREACHABLE in current code. The directed boundary guard is therefore a
  behaviorally-provable no-op; per handoff "report instead of forcing" +
  TDD no-code-without-red, NO src change to directive_parser.rs logic was
  made. Tests kept as regression locks + mod declaration only.
- Items 2-16 GREEN: added 9 test modules (34 new tests):
  parser/handle_branch_tests.rs (items 2,3,8,9,13), symbol_locator/
  scan_branch_tests.rs (5,6,7), tokenizer/literal_position_tests.rs (10),
  backend/flag_and_source_pins.rs (4,11), planning/sanitize_pin_tests.rs (15),
  analyzer/tu_io_pin_tests.rs (16), directive_parser/column_tests.rs (14),
  extension_case_matrix_tests.rs (12), plus item-1 boundary locks. Only src
  edits: test mod declarations. One pin adjusted to observed behavior
  mid-loop: `A a();` ctor shape ALSO yields a `Holder::a` return-type edge
  (member-signature misfire, report-flagged quirk) - pinned verbatim, not
  weakened. Final green: `193 passed / 0 failed / 0 ignored / 2 suites`.
