# SM-2 Journal - extraction-symbol-move: C++ move hook (ARIADNE-EXTRACT-OUT grammar)

Implementer session, 2026-10-24. Anti-phantom rules bind: every claim below
cites the command that produced it; capture paths quoted where the distiller
prints one.

## Duty-1 (landing SM-1's gated artifacts) - COMPLETE

Start rev of this session (measured, `git status --porcelain` via distill):

- HEAD `b993600` (SM-1 landing). Dirty exactly: ` M
  extensions/ariadne-extension/ariadne-extension-0.1.0.vsix` (tolerated),
  ` M tasks/milestone_ledger.md`, `?? tasks/extraction-symbol-move-sm1-verdict.md`,
  `?? tasks/extraction-symbol-move-sm1-execution-results.md`,
  `?? tasks/extraction-symbol-move-sm1-handoff-resume.md`. Match; no STOP.

Deletion (reviewer re-gate disposition DELETE - its disk-state section is
false post-`b993600`):

- Refused forms (catch-all deny boilerplate, journal-permitted): raw
  `Remove-Item`; `python tools/distill.py -- python -c "...unlink()..."`.
- Sanctioned form used:
  `python tools/distill.py -- git clean -f -- tasks/extraction-symbol-move-sm1-handoff-resume.md`
  digest: `Removing tasks/extraction-symbol-move-sm1-handoff-resume.md`
  (exit 0). Confirmation: `git status --porcelain` afterwards shows no
  `resume` line.

Landing commit `0db2680c34515e03d4589d101dca010fd564f86d` via
`python tools/commit_scoped.py --scope tasks/milestone_ledger.md --scope
tasks/extraction-symbol-move-sm1-verdict.md --scope
tasks/extraction-symbol-move-sm1-execution-results.md -m "docs(extraction-symbol-move):
SM-1 re-gate APPROVED - land verdict + execution-results + incident ledger
row (PHANTOM-NARRATION-1 closed)"`.

- `python tools/encoding_guard.py --staged` exit 0 (no output).
- `python tools/encoding_guard.py --all` exit 0 (no output) per precedent.
- Atomicity (`git show --stat HEAD` via distill): 3 files changed, 348
  insertions(+), 1 deletion(-); execution-results 90+, verdict 250+, ledger
  9/1. Reflog cross-check: `HEAD@{0}: commit: docs(extraction-symbol-move):
  SM-1 re-gate APPROVED...` at `0db2680`.
- Memory: key `ledger.milestone.extraction-symbol-move` corrected (SM-1
  LANDED + re-gate APPROVED); key `orchestration.reflog_sha_cross_check`
  added (anti-phantom countermeasure). Both confirmed by tool output.

## SM-2 boot

- `python tools/distill.py -- python tools/checkpoint_guard.py status`:
  HEAD `0db2680`, main `dirty=1` (the tolerated `.vsix`), no anomalies
  beyond it. Start rev for reviewer diffs: `0db2680`.
- Mandated intake reads done: kickoff sec. 1 + sec. 5; shape C2, invariants
  SM-I1/I2/I9, risk R2, sec. 8; `docs/specs/unit-test-audit.md` "Deferred
  findings (closeout ledger sync, 2026-10-19)" - planner intake confirmed on
  disk: zero deferred items name `dependency_rewriter.rs`, the move-hook
  seam, or a `move_retention` module (lang-cpp items name
  detector/planning/shaper/backend/parser surfaces). No new finding
  discovered so far.
- SM-1 hook verified live in `crates/lang/src/rewriter.rs:118-125` (default
  delegates to `remove_top_level_symbols`).

## Gate 1 - clean baseline BEFORE any edit

Command: `python tools/testrun.py --distill -p ariadne-lang-cpp`

Canonical line (verbatim): `205 passed / 0 failed / 0 ignored / 2 suites`
(exit 0; capture `target/testrun-capture.txt`, 282 lines).

## RGB - Red

Harness fixes before the valid Red (not semantic failures): the first gate-2
run exited 2 (parse drift) on an `out of range hex escape` - Rust `\xNN`
escapes cap at 0x7f; the non-ASCII plan-id specimen is now `"PLAN-\u{e9}"`
(still an escape, never a literal). Test 2's reconstruction helper was
simplified pre-Red (dead duplicate block).

Gate 2: `python tools/testrun.py --distill -p ariadne-lang-cpp`
Canonical line (verbatim): `205 passed / 10 failed / 0 ignored / 1 suites`
(exit 1; capture `target/testrun-capture.txt`, 353 lines). EXACTLY the ten
named tests fail (capture `failures:` block lines 274-283, verbatim):

```
    move_retention::tests::move_out_binds_marker_provenance_to_plan_id_and_normalizes_names
    move_retention::tests::move_out_marks_line_aligned_class_region_verbatim_under_prefix
    move_retention::tests::move_out_marks_multiple_spans_bottom_up_and_merges_adjacent_regions
    move_retention::tests::move_out_preserves_crlf_terminators_in_regions_and_markers
    move_retention::tests::move_out_refuses_backslash_continuation_at_the_region_last_line
    move_retention::tests::move_out_refuses_backslash_continuation_on_the_line_before_the_region
    move_retention::tests::move_out_refuses_body_ambiguous_or_not_line_aligned
    move_retention::tests::move_out_refuses_name_already_locatable_only_inside_marked_region
    move_retention::tests::move_out_refuses_region_start_inside_unterminated_block_comment
    move_retention::tests::move_out_refuses_unlocatable_name
```

First-assert panic lines (capture verbatim, the five the distiller showed +
`test result: FAILED. 205 passed; 10 failed; ...`): each pair-formed test
panics on its POSITIVE marking `.expect("extraction-symbol-move SM-2: ...")`
at move_retention.rs lines 70/114/140/159/280 - the trait default still
answers `None`. Red fails for the predicted semantic reason, compiles clean.

## RGB - Green

Green pass: `move_retention.rs` production (grammar builders
`region_marker_open`/`region_marker_close`, `prefix_out_region`,
`plan_field_ascii`, `normalize_marker_name`; `SourceLines` line model with
recognition-grammar `marked_interiors` + R-D alignment; span resolution via
`locate` with the kind preference order Class, Struct, Union, Enum,
Function, TypeAlias; refusal preflight `refuses_region_boundary`
(R-E/R-F/R-G named branches) + R-C `unique_unmarked_occurrence`;
`merge_regions` + bottom-up `render_marked_source`) and the hook override in
`dependency_rewriter.rs` directly after `consumer_rewrite_spec` (single hunk
`@@ -111,6 +111,22 @@`, capture `target/distill-capture.txt`). lib.rs rider:
one `mod move_retention;` line before `mod symbol_locator;` (alphabetical).

Gate 3: `python tools/testrun.py --distill -p ariadne-lang-cpp`
`215 passed / 0 failed / 0 ignored / 2 suites` (exit 0; capture
`target/testrun-capture.txt`, 292 lines) = baseline 205 + the ten named
tests; the 16 pre-existing rewriter tests UNEDITED.

Checkpoint (measured fixture-marking behavior; the "ONE merged region"
planner verdict): test 1 marks the 13-line `thing.h` header for `Thing`
into ONE balanced BEGIN/END pair - BEGIN bytes
`// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=Thing`, region = the 8
lines :6-:13 under the `//` prefix (ctor :8 and method :9 inside), all 5
outer lines byte-identical. The shape's "4 pairs" wording is NOT reproduced
in any test (relay correction honored; per-unit re-offer belongs to SM-3).

Gate 4: `python tools/testrun.py --distill -p ariadne-lang-scala`
`257 passed / 0 failed / 0 ignored / 2 suites` (exit 0 UNEDITED; SM-I5).

Gate 5: `python tools/testrun.py --distill -p ariadne-integration-tests
--test m18_cpp_plugin_parity_test`
`23 passed / 0 failed / 0 ignored / 1 suites` (exit 0 UNEDITED; generated-
byte parity).

Gate 6: `python tools/testrun.py --distill`
`1849 passed / 0 failed / 13 ignored / 119 suites` (exit 0). Reconciliation:
my gate-1 lang-cpp baseline `205`; SM-1-landing workspace census of record
`1839 passed / 0 failed / 13 ignored / 119 suites`; delta +10 attributed
exactly to the ten named SM-2 tests (gate 3: lang-cpp 205 -> 215), 0 failed,
13 ignored, 119 suites held. Anchor `1837/0/13/119 @ b201aa6`.

Refactor pass: simplified the test-side terminator slice, removed a dead
`let _ = source;` and an entangled alignment expression, fixed the test-4
symbols expectation to `release:v2_beta::Thing` (`:` is INSIDE the allowed
set `[A-Za-z0-9_:]` - one `_` only for `-`). The gate-3 line above is the
re-run after refactoring.

## Gates 7-8

Gate 7: `python tools/distill.py -- python tools/bench_compare.py` - fresh
recording, exit 0, digest tail: `bench compare: 3 run(s), no regressions
past 1.25x baseline` (capture `target/distill-capture.txt`, 17 lines). WARNs:
none surfaced by the distiller this run; nothing attributed (PR-2 stance).

Gate 8: `python tools/hygiene_check.py` exit 0 (probed via
`> $null 2>&1; $LASTEXITCODE` -> `0`). All emitted rows are pre-existing
R3 warns in files outside the touch-set (zero rows in `move_retention.rs`,
`dependency_rewriter.rs`, `lib.rs`). R1/R2 blocking: clean.

Frozen locks proven mechanically:
- `git diff --stat`: `dependency_rewriter.rs +16`, `lib.rs +1`, plus new
  `move_retention.rs` and this journal; `symbol_locator.rs`, `tokenizer.rs`,
  `parser.rs`, `crates/lang/**`, `crates/lang-scala/**`, `tests/**`,
  Cargo.lock: NO diff (reviewer re-verifies per-commit).
- Pure-ASCII: grep `[^\x00-\x7F]` on `move_retention.rs` -> no matches; the
  hook hunk (capture above) is ASCII by inspection.

## Refused-but-sanctioned forms (journal, kept going)

- Raw `Remove-Item ...resume.md` -> catch-all deny; done via
  `distill.py -- git clean -f -- <path>` (duty-1).
- `python tools/distill.py -- python -c "...unlink()..."` -> catch-all deny;
  a late `python -c` ASCII probe -> catch-all deny. Substituted: grep-tool
  non-ASCII scan + encoding_guard gates.
- `Get-Item ... | Select-Object` -> catch-all deny; substituted the glob
  tool.

## Gate-count accounting (declared overshoot, dated D-row)

- D-2026-10-24 (SM-2): 10 scoped invocations + commit cycle vs the declared
  8: the extras are (1) the gate-2 re-run after the harness-only `\xNN`
  escape fix (first run exited 2 parse drift, no test executed - not a
  semantic Red) and (2) the hygiene exit-code probe (output-only
  re-invocation). Count: testrun 7 + bench 1 + hygiene 2.
  Silence is the deviation; declaring.

## Deviations

- Test-4 expectation corrected during Green authoring (`:` preserved in the
  symbols field; `release:v2-beta::Thing` -> `release:v2_beta::Thing`).
  Reason: allowed-set fidelity.
- Test 4's non-ASCII plan-id specimen written `"\u{e9}"` (an escape, never a
  literal): Rust `\xNN` string escapes cap at 0x7f, so the handoff's
  `"\xe9"` form cannot express a non-ASCII byte in a Rust string literal.
  Intent preserved (escape-built non-ASCII char, `?` rendering asserted).
- Last-line-without-terminator edge: a region content line lacking a
  terminator gets the dominant terminator so the END marker stays on its own
  line (balanced-pair grammar beats verbatim on a line with no terminator
  bytes). Not exercised by any mandated test; documented in code.
- R-G's hazard input is refused by R-D too (a backslash after the body end
  is not whitespace); under the planner refinements R-G is defense in depth.
  The named branch is checked independently in the preflight; the test pins
  the OUTCOME pair, per the handoff's R-B outcome language.

## Out-of-scope observations (report-only)

- None discovered beyond the planner's stated intake zero.

## Landing

Commit `f6c92feed6137c4668de0191486164f8970141f7` (`f6c92fe`) via
`python tools/commit_scoped.py --scope crates/lang-cpp/src/move_retention.rs
--scope crates/lang-cpp/src/dependency_rewriter.rs --scope
crates/lang-cpp/src/lib.rs --scope tasks/extraction-symbol-move-sm2-journal.md
-m "feat(lang-cpp): implement move_out_top_level_symbols with the
ARIADNE-EXTRACT-OUT line-prefix grammar and refusal contract
(extraction-symbol-move SM-2)"`.

- `git show --stat HEAD`: 4 files changed, 875 insertions(+) -
  dependency_rewriter 16, lib.rs 1, move_retention 648, journal 210.
- Commit cycle forms used: `encoding_guard.py --staged` (empty index, exit
  0, no output) + `--all` superset (exit 0) BEFORE, `--all` again AFTER
  (exit 0, now covering the tracked `move_retention.rs`).
- Reflog tail (`git reflog -n 4` via distill): `f6c92fe HEAD@{0}: commit:
  feat(lang-cpp): implement move_out_top_level_symbols...`,
  `0db2680 HEAD@{1}`, `b993600 HEAD@{2}` - landing SHA verifiable in
  `.git/logs/HEAD` (the reflog tool reads that file).
- Start rev for reviewer diffs: `0db2680` (duty-1 landing; measured by
  checkpoint_guard at boot).
