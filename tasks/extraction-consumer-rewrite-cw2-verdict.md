# CW-2 Verdict - extraction-consumer-rewrite

Reviewer hop, 2026-10-23. Contract: `tasks/extraction-consumer-rewrite-cw2-handoff.md`
(author rev `a38c7aaa`, incl. its dated planner erratum on the cosmetic row).
Journal: `tasks/extraction-consumer-rewrite-cw2-journal.md`. Start rev of record:
`f73a9a1` (journal-bootstrapped; every diff adjudicated against it).

## VERDICT: APPROVED

Chain of record: duty-1 `ece3513` (CW-1 verdict + execution-results landing) +
landing `fddb200` (fix, 2 code files + journal inside) + journal rider `d3f6dbc`
(journal-only), on start rev `f73a9a1`. HEAD `d3f6dbc` on `main`; dirty: only the
tolerated pre-existing `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`
(checkpoint_guard ground truth at review start; no untracked files). No STOP
condition.

## Commit-vs-touch-set: PASS

`git diff --name-only f73a9a1..HEAD` returns exactly five paths - the three
CW-2 touch-set paths plus the two CW-1 reviewer artifacts named by the CW-1
verdict's duty-1 commands:

```
1   1   crates/cli/src/commands/ui/extraction_apply.rs
135 12  crates/cli/src/commands/ui/extraction_apply/tests.rs
202 0   tasks/extraction-consumer-rewrite-cw1-verdict.md
388 0   tasks/extraction-consumer-rewrite-cw2-journal.md
75  0   tasks/extraction-consumer-rewrite-execution-results.md
```

`Cargo.lock` in no diff. Every MUST-NOT-touch path is in no diff:
`crates/cli/src/commands/transform.rs` (frozen `:338-342` payload push),
`crates/cli/src/optimize.rs`, `crates/cli/src/commands/ui/extraction_reader.rs`,
`crates/ui/**`, `crates/lang-cpp/**`, `crates/transform/**`, `crates/validate/**`,
`tests/fixtures/**`, `tools/**`.
`git diff --numstat f73a9a1..HEAD -- crates/cli/tests` -> EMPTY (the standing
bench-attribution producer proof, verified by the reviewer, not inherited).

## Duty-1 byte-for-byte: PASS

`git show --stat "ece3513"` = 202 + 75 insertions across exactly the two
declared paths (matching the artifact lengths), and the landing message matches
the CW-1 verdict's verbatim command. `git diff ece3513..HEAD -- <both paths>`
and `git diff HEAD --numstat -- <both paths>` are BOTH empty: the landed blobs
equal the reviewer artifacts byte-for-byte at review time (no commit nor
worktree touch since). `c916a26`/`f73a9a1` untouched; no `git add .` signature.

## Per-commit review (oldest-first)

### ece3513 - docs(cw-1): reviewer verdict + execution-results

Exactly the two declared paths, content byte-equal to the reviewer artifacts
(proof above), message verbatim from the CW-1 verdict duty-1 block. Correct.

### fddb200 - fix(cli): faithfulness oracle + A9 sanitizer + no-deletions

The hand's whole production-source change is ONE hunk of ONE line in
`outcome_dto`: `plan_id: plan_id.to_string(),` -> `plan_id: ascii_only(plan_id),`
(shape C5 item 1, the A9 carve-out row's named half, at the named site). The
`new_files` lossy half stays unsanitized (field-by-field reader parity frozen -
correctly NOT touched). File metrics recorded below (295, not 296 - see finding 2).

`tests.rs` (+135/-12): the 12 deleted lines are exactly the `_plans` binding
line and the 11-line "UNSATISFIABLE / journal D6" comment block - so every
other line in the file is byte-identical BY THE DIFF ITSELF. All hunks land
inside the sanctioned regions: four inside required test 1 (binding with its
CW-2 comment; pre-apply expected line; D6 block -> fixed-claim comment;
factored-audit call), the two appended helpers between test 1 and Required
test 2, and the appended plan_id test at end of file. Required tests 2-11, the
UTA pin test, and every shared helper (`walked_files`, `modified_relative`,
`written_relative`, `is_artifact_tree`, `canonicalized_lossy`,
`assert_all_strings_ascii`, `standalone_applier`) sit in NO hunk.

C4 faithfulness oracle adjudication (the handoff's headline risk):

- `expected_modified_consumers` matches the handoff contract literally: plan
  resolved by `candidate.id.0 == plan_id` with expect; spellings from
  `crate::commands::transform::plan_consumer_discovery_targets(&registry, plan)`
  with expect Ok; prober `crate::registry::plan_backend(&registry,
  &plan.target_file, |caps| caps.rewriting, "rewriting")` + `.rewriter()`;
  per-row `std::fs::read_to_string(std::path::Path::new(&row.consumer_file))`;
  hit iff any spelling yields non-empty `find_dependency_targets`; result
  sorted. Computed PRE-APPLY (`tests.rs:253`, before `applier.apply` at :255).
  The `use ariadne_lang::DependencyRewriter;` sits inside the helper body -
  the import block was not touched, keeping all hunks inside the sanctioned
  regions. The DATED planner note's reading (spellings from the plugin, not
  the row's `old_includes` strings; predicate and matcher agree BY CONSTRUCTION
  per X1; DTO rows stay honest path data per X3; compared set IS the dry-run
  DTO's rows per `dto/extraction.rs:167`) is realized exactly as written.
- The two equality asserts carry the handoff's message literals VERBATIM
  (Rust `\` line-continuation semantics reconcile character-for-character):
  the `modified == expected_modified` faithfulness claim and the
  `expected_modified == approved_consumers` restore-equality claim. Both sides
  sorted (`modified.sort()` :277, `approved_consumers.sort()` :283,
  `expected.sort()` in the helper).
- All surviving A-4 clauses KEPT VERBATIM, none weakened or dropped -
  mechanically proven by the deletion set above: `written == approved_new_files`
  field-by-field (:268), `!written.is_empty()` (:264), non-vacuity
  `!approved_consumers.is_empty()` (:295), no-modification-outside-approved
  (:300), `dto_new_files == written` (:312), `committed` (:316),
  `output_text.ends_with("\nCommitted.")` (:321), `plan_id` round trip (:326),
  epoch (:327). The subset clause survives as belt-and-braces (handoff kept
  "all surviving clauses"; strict strengthening, accepted).
- The D6 stand-in block was replaced exactly per the handoff: the 11-line
  UNSATISFIABLE block is gone, the replacement is the fixed-claim comment
  naming `extraction-consumer-rewrite` with the honest provenance trail (m20
  characterization citation kept, CW-1 credited, falsifiability stated).
- The A-4 observations-row no-deletions assertion is the handoff's literal,
  `is_artifact_tree`-restricted, in the factored helper's claim (3).

Factoring (deviation 2): the three new claims moved VERBATIM into
`assert_modified_faithfully_applied` (doc names invariant X1 and the factoring
reason), called from required test 1 right after the real apply - every claim
still runs inside test 1. Test 1 measures 95 fn lines (:237-331); both helpers
are far under 100. This is the handoff's own prescribed mechanism ("factor the
helper, not a second test body") forced by a real R1 (115 lines, journal
recorded, hygiene then silent).

C5 item 2 (the Red test): `test_outcome_dto_sanitizes_a_non_ascii_plan_id` at
end of file - `standalone_applier(9, Vec::new())`, minimal Committed
`ApplyTransactionReport` mirroring test 10, the non-ASCII byte rides the
`\u{e9}` ESCAPE (authored source pure ASCII), asserts
`assert_all_strings_ascii(&dto, "the non-ASCII plan_id mapping")` and
`assert_eq!(dto.plan_id, "PLAN-?N-001")`. Both present; order swapped per
deviation 3.

Red / green-hardening adjudication: the plan_id Red is journal-only (reviewer
cannot mutate the tree) but mechanically corroborated at HEAD - the base blob
at :163 passed `plan_id.to_string()` through unsanitized, the captured panic
line matches the existing ASCII-guard helper's format string at `tests.rs:229`
character-for-character (`{context}: DTO string must be ASCII, got {value:?}`),
and the panic coordinates (line 229:13) are unchanged between base and Red
state. The C4 green-hardening declaration meets the handoff's RGB protocol
item 2: the justified Red-skip precedent is cited and both required witness
citations are on record (CW-1's amended m20/m19 pre-hook Red panics adjudicated
in `tasks/extraction-consumer-rewrite-cw1-verdict.md` "Red capture
adjudication"; the replaced D6 block itself). Falsifiability is real: any
dropped rewrite, out-of-set change, or deletion trips an equality or the
no-deletions claim.

### d3f6dbc - docs(cw-2): journal landing record

Journal-only (+21/-1, inside the touch-set's journal path). Correct application
of the house rider precedent: the record of `fddb200`'s SHA cannot ride the
commit it names (same posture as CW-1's `f73a9a1`).

## Erratum discharge (cosmetic row): VERIFIED ZERO-DIFF

The dated planner erratum's duty was re-verify at the start rev and journal the
result. The journal carries the full proof (region read line by line, verdict
NOT PRESENT, zero-diff, erratum pointer for closeout). I re-verified at HEAD
with my own eyes: `extraction_apply.rs:243` doc comment at column 0, `:244` fn
at column 0, `:245-247` standard chain indent (4/8) - identical footing to
`tier_row`/`tier_label_of`/`repo_relative`. No drift exists anywhere the doc
comment sits. The closeout discharges ledger row 324 on this proof (journal
section + this verdict).

## Hygiene + file metrics: PASS

- `python tools/hygiene_check.py` (fresh, reviewer-run): R1/R2 blocking silent;
  every printed row is a warn-only R3 row in files this hand did not touch.
  Neither touched file appears in any row.
- **`extraction_apply.rs`: 295 lines at HEAD `d3f6dbc`** (measured, full read).
  The hand's production hunk is a one-line REPLACEMENT (1 insertion / 1
  deletion, net 0) - the file did not grow (see finding 2 on the "999" and the
  "+1 line" phrasings). Margin to the 1000 ceiling: 705.
- `tests.rs`: **903 lines** at HEAD (margin 97). Under 1000; note for the
  ledger/closeout: CW-3 is file-disjoint, and any future hand growing
  `tests.rs` starts from a thin margin - a UTA-style split precedes the next
  growth hand in this module.
- New code names are intention-revealing (`expected_modified_consumers` -
  the shape's own reserved name, `assert_modified_faithfully_applied`,
  `consumer_text`, `spellings`, `committed_report`); no single-letter locals;
  all fns < 100 lines.
- `python tools/encoding_guard.py --all` (fresh): SILENT (pass). Authored
  content pure ASCII; the `\u{e9}` escape keeps the hazard out of source.

## Frozen-seam continuity: PASS

`IncludeRewriter` (`crates/transform/placement.rs` + `lib.rs`),
`plan_target_consumers` (`crates/cli/src/commands/transform.rs`), the CW-1
lang-cpp hook (`crates/lang-cpp/src/dependency_rewriter.rs`), and BOTH
characterization files (`m19_optimize_characterization_test.rs`,
`m20_apply_migrate_characterization_test.rs`) appear in NO diff of this hand
(zero-diff by the touch-set proof above). m20/m19 green UNEDITED AND UNWEAKENED
by the reviewer's fresh run (11/0/0/2 below); lang-cpp 205/0/0/2 below is the
witness that CW-1's fix stands under the new oracle - required test 1's
equality is the X1(b) end-to-end witness, green in my own census capture
(`test commands::ui::extraction_apply::tests::test_a_valid_token_applies_the_plan_end_to_end ... ok`,
capture line 271; caller-census test 11 green at line 199). X3 locks:
`crates/ui/**`, `transform.rs`, `extraction_reader.rs` in no diff.
X6: the sanitizer line touches no token/mutex/epoch machinery; the caller
census test ran green unedited.

## Deviations - independent dispositions

1. **Gates 6/7 not run by the producer - ACCEPTED.** Orchestration assignment
   (dispatch moved all workspace-wide gates to the reviewer hop), CW-1
   disposition-1 precedent. The producer-side half (zero-diff facts) was
   performed AND independently re-verified by me (empty `crates/cli/tests`
   numstat; complete 5-path numstat); I ran the fresh census + tools discover
   + ui_smoke + bench_compare myself. Obligation fully discharged across hops.
2. **Second helper `assert_modified_faithfully_applied` - ACCEPTED.** R1 is
   blocking and fired at 115 fn lines; the handoff's hygiene clause prescribes
   exactly this mechanism ("factor the helper, not a second test body"). I
   verified all three claims verbatim inside the helper, the helper called from
   required test 1, and the test at 95 lines. Amendment scope (X8) intact: still
   the ONE pre-existing test (required test 1); the helpers are new code.
3. **plan_id test assertion order swapped - ACCEPTED.** The handoff pinned the
   two asserts, not their order; the RGB protocol's own demand (capture the
   predicted ASCII-guard Red panic verbatim) requires the guard to fire first.
   Post-fix both assertions hold (green, both orders moot); pinning intent
   unchanged; nothing weakened.
4. **`tests.rs` at 903 vs the ~830-870 estimate band - ACCEPTED.** The binding
   bound is the 1000 ceiling (shape section 8: "under 1000 with headroom" +
   margin note - a note, not a cap); margin 97. The band priced one helper; the
   R1-forced factoring moved ~44 lines plus two doc comments into a second
   helper body - growth the estimate could not have seen, driven by the
   handoff's own hygiene law. Same disposition class as CW-1's finding 5
   (estimates, not ceilings). Produced code volume: 136 insertions / 13
   deletions across the two code files vs the ~1200 law bound.
5. **10 scoped invocations vs "8 + commit cycle" - ACCEPTED.** Itemized 7
   testrun + 3 hygiene in the journal; the excess is exactly the handoff's own
   RGB re-run mandates (green-re-run, post-C4 re-run, "Refactor ... re-run")
   plus the R1 re-measure the forced factoring required. Total 10 sits at the
   sizing law's "~10" bound; the dated ledger row (M28 T4b gate-budget
   collision) sanctions the honest routed count ~9-10 with itemization. No
   gate skipped; every canonical line re-run fresh by me anyway.
6. **Sandbox refusals (journal's declared deviation 6) - RECORDED, correctly
   self-classified as record-not-deviation.** Byte-safe edits via the Edit tool
   are AGENTS.md's mandated path; counts via Read. Superseding evidence: the
   reviewer's fresh runs of every affected gate. The 999 dispatch-note staleness
   (journal's separate "Dispatch-note discrepancy" section) is a planner
   configuration defect, not an implementer matter - see finding 2; the
   implementer's handling (worktree beats stale note, journal the discrepancy,
   do not act) is exactly right per the F14 rule.

## Workspace-wide acceptance gates (reviewer-run, fresh)

Canonical lines and the explicit not-run statement:
`tasks/extraction-consumer-rewrite-execution-results.md` (CW-2 section).
Headlines: census `1835 passed / 0 failed / 13 ignored / 119 suites` (= CW-1
anchor `1834/0/13/119` +1, exactly the new plan_id test, nothing else moved);
tools `Ran 296 tests in 26.663s` + `OK`; `ui_smoke: PASS`;
`bench compare: 3 run(s), no regressions past 1.25x baseline` (timing gate
NOT exempt - production source changed; producer zero-diff proof verified).

## Report-evidence check (tool-output distillation contract): PASS

Journal is digest-shaped throughout: distilling entry points only
(`testrun.py --distill`, `distill.py -- ...`), verdict/canonical lines, capture
paths, exit semantics; the Red panic captured verbatim as the contract demands;
no raw dumps. Gate table itemized per-gate counts (but see finding 1).

## Findings

No blockers. Minor (none gating; none require a CW-2 change):

1. (minor, record-only) Journal gate-table tally undercounts: "Scoped testrun
   invocations: 5 (gates 1, 2, 3, 3b, 4) + 1 (gate 5) = 6" while the journal's
   own body records SEVEN scoped testrun runs (gate 3c - the post-refactor
   re-run - is documented but not in that sum). Deviation 5 carries the correct
   count (7 + 3 hygiene = 10). No gate skipped; tally arithmetic only.
2. (minor, record-only, PLANNER RELAY - M23.5 F14 class) The CW-2 dispatch
   note claimed `extraction_apply.rs` "sits at 999" lines - FALSE at every
   rev of this hand: 295 at `f73a9a1`/`ece3513` (shape section 8 measured 295
   fresh at `a38c7aaa`; journal re-measured at start rev) and **295 at HEAD
   `d3f6dbc`** (reviewer-measured). Corrected fact for the closeout: the
   "999/1000" era ended with the UTA verbatim split (module tests now in
   `tests.rs`, 903 lines post-CW-2). Related wording, same correction family:
   the handoff's "`extraction_apply.rs` +1 line" sizing and the journal's
   "adds exactly 1 line" phrasing are both off - the shipped hunk is a
   one-line REPLACEMENT (1 insertion / 1 deletion, net 0); the file stays at
   295.
3. (minor, config, CARRIED from CW-1 finding 1 - still not amended) The
   reviewer allowlist still refuses the direct `python tools/ui_smoke.py` form
   (reproduced this hop, boilerplate catch-all
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`),
   every exit-code-echo form, and now also a Write to the pre-approved temp
   dir. No gate skipped: every refused form had its sanctioned
   distill/testrun-form run to PASS. The ledger row requested by CW-1 (add the
   allowlist entries) still needs landing at the next ledger touch.
4. (minor, record-only) Kickoff section 2's ledger pointers are one row low
   (~322/~324/~325 vs the actual rows): the CW-2 discharges land on ledger
   lines 322/323/324/328 (see below); the closeout should cite actual line
   numbers, never the "approximately" ones.

## Follow-up dispositions

- Findings 2 and 3 route to `tasks/milestone_ledger.md` deferred-findings at
  the next ledger touch (finding 3 extends the CW-1 finding-1 row; finding 2
  is the F14-class dispatch-note correction relay). Finding 1 and 4 are
  closeout-note material; nothing is dropped.
- Out-of-scope observations from the implementer: none beyond the dispatched
  records; none found by the reviewer.

## Closeout-discharge facts (for the closeout hop)

The four INGRESSED deferred rows (kickoff section 2), current state:

- **Ledger:321 D6 headline** (latent vacuous-rewrite): DISCHARGED by CW-1
  (hook shipped; m19/m20 amended to fixed truth; adjudicated in the CW-1
  verdict).
- **Ledger:322 D6 follow-up oracle** (expected-modified faithfulness
  equality): DISCHARGED by CW-2 - the equality is shipped and
  machine-witnessed in required test 1 (`modified == expected_modified ==
  approved_consumers`, both sides sorted, plugin-spelling probe per the dated
  note).
- **Ledger:325 stale apply-hint relay** (hotspots.js "never runs anything"):
  OPEN -> CW-3 (shape C6).
- **Ledger:326 S4 truncation-note relay** (extraction-wizard.js): OPEN -> CW-3
  (shape C6).

The arriving one-liner rows that touched CW-2:

- **Ledger:323 A9 carve-out, `plan_id` half**: DISCHARGED by CW-2 (one-line
  `ascii_only(plan_id)` + its locking test). The `new_files` half stays
  intentionally unsanitized (reader parity) - the row itself records that
  split.
- **Ledger:324 cosmetic row**: DISCHARGED by CW-2 on the zero-diff
  verification (journal erratum section + reviewer re-verification above).
- **Ledger:328 A-4 non-numbered observations**: the no-deletions observation
  is DISCHARGED by CW-2 (claim 3 of the factored audit). The row's second
  observation (no test drives a generation swap mid-transaction) remains
  EXPLICITLY OPEN - the kickoff scope-out names it another hand's scope.

Ledger row 331's shape-era margin fact for the flip: `tests.rs` now 903
(margin 97); any future hand touching the apply test module starts there.

## Duty 1 for the CW-3 implementer (exact landing commands)

The CW-3 implementer lands THIS verdict plus the execution-results addendum
as its duty-1 commit, before authoring any CW-3 Red (execution-results is a
tracked MODIFICATION this time; verdict is untracked):

```
python tools/encoding_guard.py --all
python tools/commit_scoped.py --scope tasks/extraction-consumer-rewrite-cw2-verdict.md --scope tasks/extraction-consumer-rewrite-execution-results.md -m "docs(cw-2): reviewer verdict APPROVED + execution-results addendum (extraction-consumer-rewrite CW-2)"
```

Never `git add .`; never amend `ece3513`/`fddb200`/`d3f6dbc`. CW-3 builds on
`d3f6dbc`-plus-duty-1 as its start rev.
