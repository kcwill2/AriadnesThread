# CW-2 Handoff - extraction-consumer-rewrite: A-4 apply proof upgraded to the expected-modified faithfulness oracle (+ the dated cli one-liners)

Milestone: `extraction-consumer-rewrite`. Hand 2 of 3; dispatch only after CW-1
landed (its hook makes the rewrites real; your oracle asserts them). Serial
house default: CW-3 follows you. Author rev `a38c7aaa` (planner-verified
2026-10-23); start rev = your dispatch head (post-CW-1), journal it.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-consumer-rewrite-kickoff.md` section 2 (the four
   ingressed deferred rows + the arriving one-liners: A9 carve-out row,
   cosmetic row, A-4 observations row) and section 6 (amendment policy - you
   may amend ONE pre-existing test: A-4 required test 1).
3. `tasks/extraction-consumer-rewrite-shape.md` C4 (the oracle, binding form)
   + C5 (the one-liners) + invariants X1/X6/X8 + section 8 (margin note:
   `tests.rs` grows ~780 -> ~830-870, under 1000 with headroom, no split).
4. `docs/specs/unit-test-audit.md` section "Deferred findings (closeout ledger
   sync, 2026-10-19)" - MANDATORY intake (the milestone touches cli apply
   territory). Planner intake result 2026-10-23: zero deferred items name
   `extraction_apply.rs` or its test module (the cli items name
   `parse_language_arg`, dispatch arms, transform action-arm verification -
   none here). Carry any new finding; report, do not fix.
5. Your files, current state: `crates/cli/src/commands/ui/extraction_apply.rs`
   (295 lines; `outcome_dto` :146-170; `ascii_only` :243-248) and
   `crates/cli/src/commands/ui/extraction_apply/tests.rs` (780 lines; required
   test 1 at :233-323 with the D6 comment block :280-290; helpers
   `walked_files`/`modified_relative`/`written_relative`/`is_artifact_tree`/
   `canonicalized_lossy` at :140-212; required test 3 at :378 shows the
   `crate::registry::default_registry()` + `crate::registry::plan_backend(...)`
   idiom you reuse).
6. Contract literals: `crates/ui/src/dto/extraction.rs:159-172`
   (`ExtractionRewriteDto` - `old_includes` at :167, `consumer_file` :165; the
   DTO stays path data by contract - you read it, you do not change it);
   `crates/cli/src/commands/transform.rs:279-302`
   (`plan_consumer_discovery_targets`, the public piece you call; the frozen
   payload push at :338-342 stays); `crates/lang/src/rewriter.rs:54-75` (the
   discovery probe + targets contract your helper reads text through).
7. CW-1's landing (verdict/journal) - the fix your equality witnesses.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/extraction-consumer-rewrite-cw2-journal.md (create it)
- crates/cli/src/commands/ui/extraction_apply.rs (the `plan_id` sanitizer line;
  the cosmetic region ONLY if still drifted - see dated erratum below)
- crates/cli/src/commands/ui/extraction_apply/tests.rs (required test 1
  upgrade + the no-deletions assertion + one new unit test + the helper)

You MUST NOT touch: `crates/cli/src/commands/transform.rs` (the payload push at
:340 is frozen - `plan_target_consumers` is frozen by the shape's seam
adjudication: path payloads are preview/DTO-locked and load-bearing for the
move-removal derivation `crates/validate/src/lib.rs:345-370`, and the
Scala-path move keeps needing them), `crates/cli/src/optimize.rs`,
`crates/cli/src/commands/ui/extraction_reader.rs` (frozen), `crates/ui/**`
(DTO/preview bytes are X3 locks), `crates/lang-cpp/**`, `crates/transform/**`,
`crates/validate/**`, required tests 2-11 and the UTA pin test
(`tests.rs` hunks live ONLY inside required test 1 + its new helper + the new
test), the token/mutex/epoch machinery (`extraction_apply.rs` changes by the
ONE sanitize line - caller census stays 3, X6, test 11 self-locks it).
No `git stash`, no branch change. Cargo.lock must not appear.

## What to build

### C5 one-liners (the dated rows that arrive at this touch)
1. `outcome_dto`: change `plan_id: plan_id.to_string(),` to
   `plan_id: ascii_only(plan_id),` (the A9 carve-out row's named half; the
   `new_files` lossy half stays frozen by field-by-field parity with the
   reader - do NOT sanitize `new_files`).
2. New unit test (this hand's Red):
   `test_outcome_dto_sanitizes_a_non_ascii_plan_id` -
   `standalone_applier(9, Vec::new())` + a minimal committed
   `ApplyTransactionReport` (mirror required test 10's `committed_report`
   shape); call `applier.outcome_dto("PLAN-\u{e9}N-001", &report)` (the
   non-ASCII rides a `\u{e9}` ESCAPE - authored source stays pure ASCII,
   AGENTS.md); assert `dto.plan_id == "PLAN-?N-001"` and
   `assert_all_strings_ascii(&dto, "the non-ASCII plan_id mapping")`.
   The e2e fixture is ASCII-only, which is exactly why route-side test 10
   alone could not prove this.
3. Cosmetic row, DATED PLANNER ERRATUM (2026-10-23, the clarity-track pattern):
   the ledger row (~line 323 of `tasks/milestone_ledger.md`) says the
   `ascii_only` doc comment sits four spaces past the `fn` at
   `extraction_apply.rs:243-248`. Verified at author rev `a38c7aaa`: the named
   region reads clean (doc comment at column 0, standard chain indent) - the
   UTA verbatim split moved it (`tests.rs` header lines 1-11) and the drift
   named by the row is not present at this line number. Duty: re-verify at
   your start rev. If any indentation drift of that doc comment persists
   ANYWHERE it now sits, fix it inside your commit and journal the before/
   after; if clean (expected), journal the zero-diff verification - the closeout
   discharges the row on that proof. Either way journal this handoff's erratum
   line so the closeout pointer is findable.

### C4 the faithfulness oracle (upgrade required test 1 - the ONE declared
pre-existing test amendment; kickoff section 6: same hand, reviewer per-commit
diff, everything else in `tests.rs` byte-identical)

In `tests.rs`, required test 1
`test_a_valid_token_applies_the_plan_end_to_end` (name unchanged):
- Use the plans (`_plans` -> `plans`).
- Add a private helper `expected_modified_consumers(...)`: inputs = the
  approved `dry_run.rewrites` rows, the plans (find the plan whose `id.0`
  equals the plan id), returns a sorted `Vec<String>` of exactly those
  `consumer_file` strings whose on-disk (PRE-apply - read before, or probe
  before) text has a directive hit. Computed with the public pieces:
  spellings = `crate::commands::transform::plan_consumer_discovery_targets(
  &crate::registry::default_registry(), &plan)` (expect Ok);
  prober = `crate::registry::plan_backend(&registry, &plan.target_file,
  |caps| caps.rewriting, "rewriting")?.rewriter()`;
  per row: read `std::fs::read_to_string(std::path::Path::new(&row.consumer_file))`,
  hit iff any spelling gives a non-empty
  `rewriter.find_dependency_targets(&text, spelling)`.
- DATED PLANNER NOTE (why spellings come from the plugin, not the row's
  `old_includes` strings): the deferred row's "an `old_includes` spelling" is
  realized as the plugin's discovery spellings because post-fix the rewrite
  matches exactly those (CW-1's hook maps paths through the same bare-name
  rule) - discovery's predicate and the rewrite's matcher agree BY
  CONSTRUCTION (X1), and the DTO rows stay honest path data (X3). The row's
  DTO citation (`crates/ui/src/dto/extraction.rs:167`) is honored: the
  consumer set the oracle compares against IS the dry-run DTO's rows.
- Replace the "UNSATISFIABLE / journal D6" comment block (:280-290) with a
  fixed-claim comment naming `extraction-consumer-rewrite` (house provenance
  idiom): the subset-parity clause was the D6 workaround; equality is the
  deferred row restored.
- Assertions after the real apply (all of the surviving clauses KEPT:
  `written == approved_new_files` field-by-field, `!written.is_empty()`,
  no-modification-outside-approved, `dto_new_files == written`, `committed`,
  `output_text.ends_with("\nCommitted.")`, epoch/plan round trip) PLUS:
  `assert_eq!(modified, expected_modified, "faithfulness oracle
  (extraction-consumer-rewrite, ex-D6 deferred row): the modified set equals
  the expected-modified set - every rewrite was faithfully attempted, none
  silently dropped")` and
  `assert_eq!(expected_modified, approved_consumers, "on this fixture the
  expected-modified set IS the approved dry-run's consumer set - the deferred
  row's restore-equality claim, machine-witnessed")` (sort both sides).
- NEW one-liner from the A-4 observations row (ledger ~327): no file present
  before is absent after, restricted away from the artifact trees:
  `assert!(before.keys().filter(|relative| !is_artifact_tree(relative)).all(
  |relative| after.contains_key(relative)), "the approved apply must not
  delete any pre-existing file")`.
- The host-canonical string equality between `modified` (via
  `canonicalized_lossy`) and `consumer_file` (already host-canonical - the
  fact recorded at :204-212) is the same footing the current subset clause
  stands on; keep using it.

## RGB protocol
1. Red (genuine, one test): land the plan_id sanitizer TEST first - it fails
   pre-fix: `outcome_dto` passes `plan_id` through unsanitized, so
   `assert_all_strings_ascii` panics with
   `the non-ASCII plan_id mapping: DTO string must be ASCII, got "PLAN-<e-acute>N-001"`
   (the raw char renders at panic time from the escape; your source stays ASCII).
   Capture the failing test verbatim in the journal.
2. The C4 upgrade + no-deletions assertion are DECLARED green-hardening,
   justified Red-skip (precedent: ui-extraction-wizard W-1's justified
   Red-skip): post-CW-1 the equality holds, so the strengthened assertions
   cannot be made red by this hand without regressing CW-1 (which is a
   STOP-and-report, not a Red). Their falsifiability is real - the equality
   fails if any rewrite silently drops, any file changes outside the expected
   set, or any file gets deleted; and the historical Red of this claim is
   captured on record by CW-1's amended m20/m19 Red (pre-hook vacuity) and
   the D6 block you are replacing. Journal this justification with those two
   citations.
3. Green: the sanitizer line (+ conditional cosmetic fix); re-run until green.
   NEVER weaken an assertion to pass.
4. Refactor: honest helper name (`expected_modified_consumers`), doc comment
   naming the invariant; re-run.

## Hygiene (review-enforced)
Functions < 100 lines (upgraded test 1 stays under - factor the helper, not a
second test body); `tests.rs` lands ~830-870 < 1000 (margin note, no split);
`extraction_apply.rs` +1 line. No banned names. `python tools/hygiene_check.py`
exit 0.

## Frozen locks (commands that prove them)
- Required tests 2-11 + UTA pin test + all helpers byte-identical UNEDITED:
  reviewer per-commit diff (hunks only inside required test 1's region + the
  appended test/helper); gate 4 green runs them.
- A-4 gate/mutex/epoch/caller-census proofs green UNEDITED (X6): gate 3.
- m20/m19 characterization green UNEDITED-by-you (CW-1 owns them; they must
  appear in NO diff of yours): gate 4 + reviewer diff.
- lang-cpp suites green UNEDITED (your witness that CW-1's fix stands under
  your oracle): gate 5.
- Preview/dry-run DTO bytes (X3): `crates/ui/**`, `transform.rs`,
  `extraction_reader.rs` in no diff.
- goldens/fixtures; `tools/**` operator-law; Cargo.lock absent.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only;
raw cargo is a reviewer blocker)
1. `python tools/testrun.py --distill -p ariadne-cli` - clean baseline before
   edits; journal the canonical line verbatim.
2. `python tools/testrun.py --distill -p ariadne-cli --
   test_outcome_dto_sanitizes_a_non_ascii_plan_id` - Red: exit 1, exactly that
   test failing, verbatim panic line in the journal.
3. `python tools/testrun.py --distill -p ariadne-cli` - Green: exit 0, 12
   apply-module tests + the upgraded test 1 (the canonical line shows +1
   passed vs baseline).
4. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m20_apply_migrate_characterization_test --test
   m19_optimize_characterization_test` - exit 0 UNEDITED by you.
5. `python tools/testrun.py --distill -p ariadne-lang-cpp` - exit 0 UNEDITED.
6. `python tools/testrun.py --distill` - full census; reconcile + attribute
   (expect +1 passed, suites/ignored held; anchor
   `1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b`; `--distill`
   substitute governs, FILE-form errata stands).
7. `python tools/distill.py -- python tools/bench_compare.py` - fresh
   recording; WARNs recorded not attributed (PR-2); `crates/cli/tests` in no
   diff (cite the empty `git diff --numstat`).
8. `python tools/hygiene_check.py` - exit 0.
9. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`
   superset, journal the form) then
   `python tools/commit_scoped.py --scope
   crates/cli/src/commands/ui/extraction_apply.rs --scope
   crates/cli/src/commands/ui/extraction_apply/tests.rs --scope
   tasks/extraction-consumer-rewrite-cw2-journal.md -m "fix(cli): expected-modified
   faithfulness oracle + A9 plan_id sanitizer + A-4 no-deletions check
   (extraction-consumer-rewrite CW-2)"`. Never `git add .`; journal rides
   inside the landing commit.

Declared gate budget (2026-10-06 M28 law): 8 scoped invocations + commit
cycle - inside the law; itemize per-gate counts in the journal.

## Discipline clauses
- UTF-8: Write/Edit tools only; authored content pure ASCII (the `\u{e9}`
  escape is how a non-ASCII byte enters a test WITHOUT non-ASCII source).
- Touch only the touch-set; amend ONLY required test 1; any further
  pre-existing-test need is a STOP-and-report (X8/R8).
- After EACH Green gate: 1-3 line journal checkpoint.
- Context exhaustion: STOP, disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-consumer-rewrite-cw2-handoff.md --scope
  crates/cli/src/commands/ui/extraction_apply --cmd ...`.
- No count claims without the producing command (M23.5 F14 rules).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (the one named failing
test, verbatim panic line) + the green-hardening declaration with its two
witness citations. 3. Green evidence (distill digest headers + canonical lines
for gates 3-6 + capture path). 4. Deviations + reasons. 5. Out-of-scope
observations (report-only). Never paste raw output.

Reviewer-gate reference: adjudicate via `review-gate`. Extra MUSTs: hunks in
`tests.rs` confined to required test 1 + appended helper/test;
`extraction_apply.rs` diff is the one sanitizer line (+ conditional cosmetic
hunk with journal erratum); required tests 2-11 byte-identical; caller census
still 3; the closeout-discharge set from your journal: A9 plan_id half,
cosmetic row (fix or zero-diff proof), A-4 no-deletions observation.

Sizing (planner, itemized from the mandated bodies): sanitizer line 1;
conditional cosmetic hunk ~0-4; helper ~28; test-1 assertion/comment changes
~+40/-14; no-deletions 4; plan_id test ~24; journal ~15 - total ~110-200
produced lines vs ~1200; 8 gates + commit cycle; one crate dir (`crates/cli`).
