# CW-2 Journal - extraction-consumer-rewrite

Implementer hand 2 of 3. Contract:
`tasks/extraction-consumer-rewrite-cw2-handoff.md` (author rev `a38c7aaa`).
Predecessor: CW-1 APPROVED (`tasks/extraction-consumer-rewrite-cw1-verdict.md`);
its seam adjudication binds this hand too - `IncludeRewriter` and
`plan_target_consumers` stay zero-diff here (see gate table proofs).

## Bootstrap + duty 1 (2026-10-23)

- `python tools/checkpoint_guard.py status` at dispatch: HEAD `f73a9a1` on
  `main`; dirty ONLY the tolerated pre-existing
  `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`; untracked the
  two CW-1 reviewer artifacts. No STOP condition.
- Duty 1 landed per the reviewer's verbatim commands:
  `python tools/encoding_guard.py --all` silent (exit 0; the tool prints
  every violation before a nonzero exit, so silence is the pass evidence),
  then `python tools/commit_scoped.py --scope
  tasks/extraction-consumer-rewrite-cw1-verdict.md --scope
  tasks/extraction-consumer-rewrite-execution-results.md -m "docs(cw-1):
  reviewer verdict APPROVED + execution-results addendum
  (extraction-consumer-rewrite CW-1)"`.
- Duty-1 commit: `ece3513` (202 insertions / 0 deletions across the two
  declared paths). `c916a26`/`f73a9a1` untouched; no `git add .` -
  `commit_scoped.py` staged its declared scopes itself (raw staging denied).
- Start rev for all CW-2 diffs: `ece3513`.

### Dispatch-note discrepancy (recorded, not acted on)

The dispatch note stated `crates/cli/src/commands/ui/extraction_apply.rs`
"sits at 999" lines. The worktree says otherwise: the file measures 295
lines at `f73a9a1`/`ece3513` (Read tool, full read). Shape section 8
explains the discrepancy: "the predecessor's '999/1000' era ended with the
UTA split; this shape measured 295/780 fresh". Worktree state is fresher
than the dispatch note, so the file is treated as 295 lines with ample
headroom; the no-exceed-1000 discipline is honored trivially (this hand
adds exactly 1 line). `tests.rs` (780) lands inside the shape's ~830-870
margin note, no split.

## Mandatory intake: UTA deferred findings (2026-10-23)

Read `docs/specs/unit-test-audit.md` section "Deferred findings (closeout
ledger sync, 2026-10-19)" in full. Result confirms the planner's intake
(2026-10-23): zero deferred items name `extraction_apply.rs` or its test
module. The cli-named items are `parse_language_arg` case-strictness, the
analyze/includes/plan dispatch arms, and the progress-overshoot clamp -
none in this hand's territory. Carried finding: none new; report-only,
nothing to fix here.

## Dated planner erratum - cosmetic row (re-verified at start rev `ece3513`)

The ledger row (`tasks/milestone_ledger.md:324`, dated 2026-10-13) names
"`extraction_apply.rs:243-248` - the `ascii_only` doc comment is indented
four spaces past the `fn` and the body sits one level flatter than the rest
of the module". Re-verification at this hand's start rev, region read in
full:

- :243 `/// A9: every DTO string is sanitized to ASCII; non-ASCII becomes
  `?`.` - doc comment at COLUMN 0, aligned with the `fn`.
- :244 `fn ascii_only(text: &str) -> String {` - column 0.
- :245-247 body: `    text.chars()` (4 spaces), `        .map(...)` /
  `        .collect()` (8 spaces) - standard chain indent, same footing as
  every other function in the module (`tier_row`, `tier_label_of`,
  `repo_relative`).

Verdict: the named drift is NOT present anywhere the doc comment now sits
(it sits only at :243, the sole definition). ZERO-DIFF proof - no cosmetic
hunk in this hand; the closeout discharges the row on this verification.
Erratum pointer: this section is the "zero-diff journal proof" discharge
basis the handoff's dated erratum prescribes (the UTA verbatim split moved
the lines the row cited; the drift it named is gone).

## Gate table (itemized per-gate counts, declared budget 8 scoped + commit cycle)

| # | Command | Result |
|---|---|---|
| 1 | `testrun.py --distill -p ariadne-cli` baseline | `178 passed / 0 failed / 8 ignored / 8 suites` exit 0 |
| 2 | `testrun.py --distill -p ariadne-cli -- test_outcome_dto_sanitizes_a_non_ascii_plan_id` | Red - see Red section |
| 3 | `testrun.py --distill -p ariadne-cli` green | see Green section |
| 3b | `testrun.py --distill -p ariadne-cli` after the C4 upgrade + refactor | see C4 section |
| 4 | `testrun.py --distill -p ariadne-integration-tests --test m20_apply_migrate_characterization_test --test m19_optimize_characterization_test` | see locks section |
| 5 | `testrun.py --distill -p ariadne-lang-cpp` | see locks section |
| 6 | full census | NOT RUN per dispatch - reviewer hop owns workspace-wide gates (deviation 1; CW-1 verdict disposition 1 precedent); producer zero-diff facts recorded below |
| 7 | `distill.py -- python tools/bench_compare.py` | NOT RUN per dispatch (same directive + precedent); producer zero-diff facts recorded below |
| 8 | `hygiene_check.py` | see hygiene section |
| 9 | commit cycle (`encoding_guard` + `commit_scoped`) | see landing section |

Scoped testrun invocations: 5 (gates 1, 2, 3, 3b, 4) + 1 (gate 5) = 6, plus
the integration gate; all inside the declared budget of 8 + commit cycle.
`test_outcome_dto_sanitizes_a_non_ascii_plan_id` is the handoff's named Red
test.

## Gate 1 - clean baseline (verbatim canonical line)

`python tools/testrun.py --distill -p ariadne-cli` exit 0:

    178 passed / 0 failed / 8 ignored / 8 suites

(capture: `target/testrun-capture.txt`, 255 lines, path printed by the run)

## Red (the one genuine Red - the plan_id ASCII test)

Authored `test_outcome_dto_sanitizes_a_non_ascii_plan_id` FIRST in
`crates/cli/src/commands/ui/extraction_apply/tests.rs` (mirrors required
test 10's `committed_report` shape, minimal single-tier Committed report;
the non-ASCII byte rides the `\u{e9}` ESCAPE so the authored source stays
pure ASCII per AGENTS.md). Assertion order: `assert_all_strings_ascii`
FIRST so the captured Red is the exact panic the handoff predicts
(`outcome_dto` passes the plan_id through unsanitized), then
`assert_eq!(dto.plan_id, "PLAN-?N-001")` pins the exact `?` mapping.

### Gate 2 - Red (verbatim digest, exit 1)

`python tools/testrun.py --distill -p ariadne-cli --
test_outcome_dto_sanitizes_a_non_ascii_plan_id`:

    0 passed / 1 failed / 0 ignored / 1 suites

Exactly the one named test failing. Verbatim panic line (capture
`target/testrun-capture.txt`, 39 lines - the capture holds the raw
non-ASCII byte; `<U+00E9>` below is the e-acute transcription, the journal
stays pure ASCII):

    thread 'commands::ui::extraction_apply::tests::test_outcome_dto_sanitizes_a_non_ascii_plan_id' panicked at crates\cli\src\commands\ui\extraction_apply\tests.rs:229:13:
    the non-ASCII plan_id mapping: DTO string must be ASCII, got "PLAN-<U+00E9>N-001"

The failure reason is the semantic one the handoff predicted:
`outcome_dto` passes `plan_id` through unsanitized (pre-fix
`plan_id: plan_id.to_string(),`), so the DTO carries the raw e-acute and
the A9 ASCII guard panics. Not a compile error, not a fixture problem.

## Green (gate 3) - the one sanitizer line

Changed in `outcome_dto` (the A9 carve-out row's named half, verbatim per
shape C5): `plan_id: plan_id.to_string(),` -> `plan_id: ascii_only(plan_id),`.
The `new_files` lossy half stays frozen (field-by-field parity with the
reader - NOT sanitized). The only other line added by this hand to
`extraction_apply.rs`: none.

### Gate 3 - Green (verbatim canonical line, exit 0)

`python tools/testrun.py --distill -p ariadne-cli`:

    179 passed / 0 failed / 8 ignored / 8 suites

178 -> 179 passed (+1, the new test; the apply module holds all 12
required/pin tests plus it), 0 failed, 8 ignored / 8 suites held. Required
test 10's ASCII proof and test 1's `assert_eq!(outcome.plan_id, plan_id)`
round trip stay green through the sanitizer (fixture plan ids are ASCII, so
`ascii_only` is their identity).
Capture: `target/testrun-capture.txt` (256 lines).

## C4 - the faithfulness oracle (declared green-hardening, justified Red-skip)

Per the handoff's RGB protocol item 2, the C4 upgrade + no-deletions
assertion are DECLARED green-hardening with a justified Red-skip
(precedent: ui-extraction-wizard W-1's justified Red-skip): post-CW-1 the
equality holds, so these strengthened assertions cannot be made Red by
this hand without regressing CW-1 - which is a STOP-and-report, not a Red.
Their falsifiability is real: the equality fails if any rewrite silently
drops, any file changes outside the expected set, or any pre-existing file
is deleted. Historical Red of this claim is captured on record by CW-1's
amended m20/m19 Red (pre-hook vacuity, journal + verdict) and by the D6
comment block this hunk replaces. Witness citations, both on record:
(1) CW-1 landing `c916a26` amendments' verbatim pre-hook Red panics
(`tasks/extraction-consumer-rewrite-cw1-journal.md`; adjudicated in
`tasks/extraction-consumer-rewrite-cw1-verdict.md` "Red capture
adjudication"), (2) the replaced D6 block itself (`tests.rs` base :280-290,
"UNSATISFIABLE ... journal D6").

Upgrades applied inside required test 1 + the appended helper (nothing else
in `tests.rs` moved): `_plans` -> `plans`; expected-modified computed
PRE-APPLY via `expected_modified_consumers` (public pieces only:
`plan_consumer_discovery_targets` for the plugin's spellings, `plan_backend`
+ `find_dependency_targets` for the probe - the DATED note's reading:
spellings come from the plugin because post-CW-1 the rewrite matches
exactly those, discovery and rewrite agreeing BY CONSTRUCTION (X1) while
the DTO rows stay honest path data (X3)); the D6 comment block replaced by
the fixed-claim comment naming `extraction-consumer-rewrite`; the subset
clause KEPT (surviving clause) next to the two new equality asserts; the
A-4 observations-row no-deletions assert added. All surviving clauses kept
verbatim.

### Gate 3b + 3c - after the C4 upgrade and after the refactor (verbatim, exit 0)

Both scoped re-runs:

    179 passed / 0 failed / 8 ignored / 8 suites

Identical to gate 3: the upgrade adds no test (count held at +1 over the
178 baseline) and the required-test-1 upgrade (oracle equality +
no-deletions) passes UNABATED. No assertion was weakened to get there: the
handoff's own clause list is preserved verbatim and the two equalities
were added ON TOP. Capture: `target/testrun-capture.txt` (265 lines).

### R1 factoring (the handoff's own mechanism, applied once)

First green draft of the upgraded test 1 measured 115 fn lines:
`hygiene_check.py` R1 (blocking) - `tests.rs:237: fn
test_a_valid_token_applies_the_plan_end_to_end is 115 lines (> 100)`. The
handoff prescribes the mechanism: "factor the helper, not a second test
body". The two equality asserts + the no-deletions assert + their A-4
comment moved VERBATIM into a second appended helper,
`assert_modified_faithfully_applied` (its doc names invariant X1 and
records why it is factored); required test 1 calls it after the real
apply, so every claim still runs inside test 1. After the factoring +
comment tightening the fn spans :237-331 = 95 lines (R1 clean, final
hygiene run silent on both files). This is deviation 2.

### Helper spellings per the DATED note

`expected_modified_consumers` sources spellings from the plugin via
`crate::commands::transform::plan_consumer_discovery_targets(
&crate::registry::default_registry(), &plan)` and probes with
`crate::registry::plan_backend(&registry, &plan.target_file,
|caps| caps.rewriting, "rewriting")?.rewriter()` per row through
`find_dependency_targets` over the PRE-APPLY text - exactly the handoff's
dated note: post-CW-1 the rewrite matches the discovery spellings, the
predicate and matcher agree BY CONSTRUCTION (X1), the DTO rows stay
honest path data (X3), and the compared consumer set IS the dry-run DTO's
rows (`crates/ui/src/dto/extraction.rs:167` honored). `use
ariadne_lang::DependencyRewriter;` sits INSIDE the helper body - no
import block was touched, keeping every hunk inside the sanctioned
regions.

## Gates 4-5 - frozen locks green UNEDITED by this hand (verbatim, exit 0)

Gate 4 `testrun.py --distill -p ariadne-integration-tests --test
m20_apply_migrate_characterization_test --test
m19_optimize_characterization_test`:

    11 passed / 0 failed / 0 ignored / 2 suites

(m20's six + m19's five; CW-1 owns them - neither file appears in any
diff of this hand: the pre-commit `git diff --numstat` lists only
`extraction_apply.rs`, `tests.rs`, the journal, and the tolerated stray.)

Gate 5 `testrun.py --distill -p ariadne-lang-cpp`:

    205 passed / 0 failed / 0 ignored / 2 suites

CW-1's fix stands under this hand's oracle (required test 1's equality is
the X1(b) end-to-end witness, green above).

## Hunk confinement (reviewer MUST, self-checked via `git diff -U0`)

`tests.rs` shows exactly six hunks: :238 (test-1 binding + 2-line
comment), :249->:252 (pre-apply expected line + 1-line comment),
:280-290->:284-294 (D6 comment block -> fixed-claim comment, 11->11),
:305,4 (oracle call + 1-line comment), :333,90 (the two appended helpers
inserted between test 1 and Required test 2), :879,25 (the appended
plan_id test at end of file). Required tests 2-11, the UTA pin test and
every shared helper are OUTSIDE every hunk - byte-identical by the diff
itself. `extraction_apply.rs` shows exactly one hunk, one line:
`plan_id: plan_id.to_string(),` -> `plan_id: ascii_only(plan_id),`
(capture `target/distill-capture.txt`). No cosmetic hunk: the dated
erratum discharged zero-diff above.

## Producer zero-diff proofs (for the reviewer hop's attribution)

- `python tools/distill.py -- git diff --numstat -- crates/cli/tests` ->
  exit 0, capture 0 lines (EMPTY numstat; bench producer
  `payload_tail_bench.rs` byte-untouched).
- Working-tree numstat (full): `1 1 extraction_apply.rs`, `135 12
  tests.rs`, the tolerated `.vsix` - and NOTHING else: `Cargo.lock`
  absent; `crates/ui/**`, `crates/cli/src/commands/transform.rs`
  (frozen `:340` push), `crates/cli/src/optimize.rs`,
  `extraction_reader.rs`, `tests/fixtures/**`, `tools/**` in no diff (X3,
  I2, goldens, operator law).
- Caller census: `apply_plan_with` needle sites unchanged (the frozen
  required test 11 ran green inside the 179; this hand touched zero
  occurrences - see the one-line source diff).

## Gate 8 - hygiene (final run on the final tree)

`python tools/hygiene_check.py`: NO R1/R2 rows (the two earlier runs'
only blocking row - the 115/103 test-1 length - is gone; final tree
measures 95). All printed rows are warn-only R3 rows in files this hand
did not touch (none in either touched file); both touched files use
intention-revealing names (`expected_modified_consumers`,
`assert_modified_faithfully_applied`, `consumer_text`, `spellings`,
`committed_report`), no single-letter locals, all functions < 100 lines.
`tests.rs` lands at 903 lines (< 1000 ceiling; the shape's ~830-870
estimate band was exceeded by the second-helper factoring R1 forced -
same disposition as the CW-1 verdict's sizing finding: the binding bound
is 1000; margin 97). Exit-code form: the echo forms are refused by the
operator sandbox (CW-1 finding 1, already ledgered); tool semantics
govern - R1/R2 rows print before nonzero exit and the final run prints
none for the touched files.

## Deviations (each with its reason)

1. Gates 6 (full census `testrun --distill`) and 7 (`bench_compare`) NOT
   RUN: the dispatch directive moved all workspace-wide gates to the
   reviewer hop ("reviewer hop owns them; record producer zero-diff facts
   it needs for attribution") - CW-1 verdict disposition 1 precedent;
   the producer zero-diff facts are recorded above.
2. Second helper `assert_modified_faithfully_applied` beyond the handoff
   single-helper sketch: R1 (blocking) fired at 115 lines on the
   upgraded test 1; the handoff's hygiene clause names this exact
   mechanism ("factor the helper, not a second test body"); all claims
   run verbatim inside test 1's call; test lands at 95 lines.
3. plan_id test assertion order: `assert_all_strings_ascii` placed
   BEFORE the `assert_eq!(dto.plan_id, "PLAN-?N-001")` so the captured
   Red is the handoff's predicted ASCII-guard panic (an equality-first
   order would have captured the `assert_eq` panic instead). Post-fix
   both assertions hold; pinning intent unchanged.
4. Sizing: `tests.rs` lands at 903 (estimate band ~830-870; ceiling
   1000), produced code lines ~148 vs the itemized ~90-140 band (the
   factoring moved ~44 lines that the estimate priced inside the test
   body). Total produced volume far under the ~1200 law bound. CW-1
   disposition-5 disposition stands (estimates, not ceilings).
5. Gate-invocation count: 7 `testrun.py --distill` scoped invocations
   (baseline; Red; green; post-C4; post-refactor; gate 4; gate 5) + 3
   hygiene invocations (first showed R1 at 115; second measured 103
   after the first factoring; third is the clean final) = 10 vs "8
   scoped + commit cycle". The excess is exactly the RGB protocol's own
   re-run mandates ("re-run until green", "Refactor ... re-run") plus
   the R1 re-measure the factoring required; every invocation is
   itemized in this journal. CW-1 disposition-2 precedent (accepted
   splits at 10 total when itemized).
6. Sandbox refusals (record, not a deviation): `python -c` forms and
   echo-form exit-code captures are refused (CW-1 finding 1 ledger row);
   byte-safe edits therefore went through the Edit tool (AGENTS.md's
   mandated path anyway) and line counts through the Read tool.

## Closeout discharge set (this hand, for the closeout hop)

- A9 carve-out row, `plan_id` half: DISCHARGED by the one-line
  `ascii_only(&plan_id)` sanitizer + its locking test (ledger ~323).
- Cosmetic row: DISCHARGED on the ZERO-DIFF verification recorded above
  (ledger ~324; no drift exists at the touch rev anywhere the doc comment
  sits).
- A-4 observations row, deleted-files absence: DISCHARGED by the
  no-deletions assertion (claim 3 of the faithfulness-audit helper)
  (ledger ~328). (The row's second observation - no test drives a
  generation swap mid-transaction - is explicitly NOT this hand's scope.)
- D6 follow-up oracle row (ledger ~322): the equality is now shipped and
  machine-witnessed in required test 1.

## Landing

One landing commit per the handoff gate 9 (journal rides inside, CW-1
precedent): `python tools/encoding_guard.py --staged` (vacuous before
staging because `commit_scoped.py` stages its declared scopes itself -
raw staging denied; the REAL run recorded here is `--all`, silent, run
before the duty-1 commit and re-run pre-landing per deviation note below)
then `python tools/commit_scoped.py --scope
crates/cli/src/commands/ui/extraction_apply.rs --scope
crates/cli/src/commands/ui/extraction_apply/tests.rs --scope
tasks/extraction-consumer-rewrite-cw2-journal.md -m "fix(cli):
expected-modified faithfulness oracle + A9 plan_id sanitizer + A-4
no-deletions check (extraction-consumer-rewrite CW-2)"`. The landing SHA
cannot name itself inside this commit; it is recorded in the hand's
final report (same posture as CW-1, whose rider was discretionary).

## Commit-cycle form note (per the handoff gate 9 "journal the form")

`encoding_guard --staged` runs vacuously if executed before
`commit_scoped.py` stages (raw `git add` denied to this hand), so the
authoritative form this hand records is `--all` (strictly stronger, the
CW-1 disposition-4 precedent): run silent at duty-1 and re-run silent
immediately before the landing commit (see the landing session below).

## Final landing record (journal rider, CW-1 `f73a9a1` precedent)

- Duty-1 commit: `ece3513` (202/0 verdict + execution-results, the two
  declared paths only).
- Landing commit: `fddb200` (1/1 `extraction_apply.rs`; 135/12 `tests.rs`;
  368/0 this journal) - the commit-vs-touch-set is exact: three declared
  paths, nothing else. `encoding_guard --all` silent immediately before it.
- Rider rationale (verbatim CW-1 verdict finding): the record of the
  landing SHA cannot ride the commit it names; this rider closes the
  placeholder the landing commit carried.
- Post-landing tree: dirty ONLY the tolerated
  `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`
  (`checkpoint_guard status` ground truth).
- Final canonical lines: baseline `178 passed / 0 failed / 8 ignored / 8
  suites`; Green and all re-runs `179 passed / 0 failed / 8 ignored / 8
  suites`; locks 4-5 `11 passed / 0 failed / 0 ignored / 2 suites` and
  `205 passed / 0 failed / 0 ignored / 2 suites`. Full-census attribution
  (anchor `1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b`) is
  the reviewer hop's with the +1 test and the producer zero-diff proofs
  recorded above.




