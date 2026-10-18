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
