# SM-4 Verdict - extraction-symbol-move (strip-fold mirror + grown move-proof oracle + generation-swap relay RECORD; BOTH-HANDS HARD-CHAIN DISCHARGE)

Review-gate adjudication of handoff SM-4, contract of record
`tasks/extraction-symbol-move-sm4-handoff.md` INCLUDING its dated AMENDMENT-1
CARRIER (2026-10-24, planner) - the carrier supersedes the handoff's baseline
quotes, gate-1/gate-2 expectations, and the gate-6 census anchor. This hop
ALSO discharges the workspace-wide gates SM-3's verdict (finding M3) deferred
to it per the carrier: the verdict names BOTH chained commits - SM-3 landing
`21cf779c599fb7b38fe935303cec20b27dd8d7e0` (`21cf779`) and SM-4 landing
`e717ba0866eda1d1f8117e8f6b427ee73171e175` (`e717ba0`) - in the gate-coverage
section below. Anti-phantom law (SM-1 verdict standard, PHANTOM-NARRATION-1):
every SHA cross-checked against `.git/logs/HEAD` via the Read tool BEFORE any
gate was trusted; every number below is from MY OWN fresh runs, quoted
verbatim; zero writes outside `tasks/**`.

## Verdict: APPROVED

No blockers. The R5 window - dry-run under-reporting while apply marks - is
CLOSED at this landing, machine-witnessed. The hard chain SM-3 -> SM-4 is
DISCHARGED for BOTH hands by my fresh census. Deviations D-a..D-e ACCEPTED
(independent dispositions below). Six minor/informational findings (M1-M6);
none requires an implementer return.

## Anti-phantom verification (reflog ground truth)

`.git/logs/HEAD` read directly with the Read tool; the file ENDS at line 446
(the landing entry) - no reset, no amend, no stray commit after the landing.

Line 445 verbatim (duty-1, the SM-4 session's pre-hand doc landing):

`21cf779c599fb7b38fe935303cec20b27dd8d7e0 a33f6203777d18dbb6b60acf34c0a7b1cfc3abba kcwill2 <kcwill2@gmail.com> 1792839350 -0700	commit: docs(extraction-symbol-move): SM-3 APPROVED - land verdict + execution-results + ledger SM-3 record + SM-3/SM-4 amendments`

Line 446 verbatim (SM-4 landing):

`a33f6203777d18dbb6b60acf34c0a7b1cfc3abba e717ba0866eda1d1f8117e8f6b427ee73171e175 kcwill2 <kcwill2@gmail.com> 1792843231 -0700	commit: feat(cli): dry-run move honesty via the strip-fold mirror; grown move-proof oracle; required test 5 move truth (extraction-symbol-move SM-4)`

Both expected SHAs match the dispatch exactly. `git rev-parse HEAD` =
`e717ba0866eda1d1f8117e8f6b427ee73171e175`.
`python tools/checkpoint_guard.py status` at boot: HEAD `e717ba0`, main,
ahead 0 / behind 0, dirty = ONLY the tolerated
`extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`, untracked none.
Worktree census as recorded in the execution-results addendum.

**Adjacency MUST (carrier 2, the R5 + red-test law): HELD.** Reflog 444
(SM-3 code landing `21cf779`) -> 445 (duty-1 DOC landing `a33f620`, allowed
class) -> 446 (SM-4 code landing `e717ba0`). The SM-4 code landing is the
IMMEDIATE next code landing after SM-3's; no release in the window. The R5
under-report window was open across exactly one commit (`21cf779`) - by the
amendment's design - and is closed at `e717ba0`.

## Commit-vs-touch-set (the enforcement check)

`git log --oneline a33f620..e717ba0` = exactly ONE commit (`e717ba0`).
`git diff --name-only a33f620..e717ba0` = EXACTLY the four declared touch-set
paths, nothing else:

- `crates/cli/src/commands/ui/extraction_reader.rs` (numstat 51/24 - net +27,
  899 -> 926 lines, measured at landing; the handoff's `~910` estimate
  drifted, margin noted per shape section 8, no split owed)
- `crates/cli/src/commands/ui/extraction_apply/tests.rs` (numstat 25/14 -
  net +11, 903 -> 914 lines; the ceiling WATCH 918 holds with 4 lines of
  headroom - the journal states it and my Read measurement confirms)
- `crates/cli/src/commands/ui/extraction_apply/tests/move_proof.rs` (NEW,
  495/0; the handoff sizing said ~300-340 - declared D-2026-10-24c,
  disposition below; R2 1000 with wide margin)
- `tasks/extraction-symbol-move-sm4-journal.md` (301/0 - journal rides the
  landing commit)

Frozen-surface proofs (my own runs, each printed ZERO lines):

- `git diff --numstat a33f620..e717ba0 -- crates/cli/src/commands/ui/extraction_apply.rs`
  EMPTY - the generation-swap relay's byte proof: the touch fired at the
  TESTS module only; NO execute happened (see carry-forward).
- `git diff --numstat a33f620..e717ba0 -- crates/validate crates/lang crates/lang-cpp crates/transform crates/ui tests/fixtures tools tests/integration Cargo.lock`
  EMPTY - every earlier hand's surface (SM-1 lang, SM-2 lang-cpp, SM-3
  validate + m19/m20) zero-diff; DTO bytes (`crates/ui/src/dto/**` inside the
  zero-diff `crates/ui`) untouched, the `copy_only: move_removals == 0`
  construction survives with its computed meaning; Cargo.lock absent from the
  diff; goldens/fixtures byte-locked.

Duty-1 `a33f620` verified against the journal's claim: `git show --stat` =
5 files, 708 insertions(+), 2 deletions(-) - sm3-execution-results 147+,
sm3-handoff 209+ (AMENDMENT-1), sm3-verdict 283+, sm4-handoff 62+ (the
carrier), milestone_ledger 9 +- ; `.vsix` absent (atomic).

## Per-commit review (single commit `e717ba0`, judged alone, oldest-first - full reads)

- `extraction_reader.rs`: FOUR hunks, all licensed. (1) The doc lift: states
  the strip mirror + plan-id marker provenance ("m20's `PLAN-001` byte
  truth") + "Computed, not asserted (W4)" + the honest-contributes-nothing
  refusal sentence - the handoff's demanded wording. (2) `move_removal_count`
  rewritten as the FOLD: per-path running-text `HashMap` cache, BTreeSet
  candidate order, the CURRENT fold text offered to
  `move_out_top_level_symbols(&current_text, &moved_names, &plan.id.0)`, on
  `Some` the cached text is replaced and `names.len()` added - PLAN-LITERAL
  against the carrier's binding refinement (input 7), word for word. The old
  per-candidate `remove_top_level_symbols` offer is GONE (no retained dual
  representation; the old hook survives only as the frozen SM-1 delegation
  default elsewhere). Read-only: nothing is written; the honest-absent
  degraded path (`honest_absent_dry_run`, `move_removals: 0, copy_only:
  false, includes_unavailable: true`, current :435-447) can never fire the
  fold (engine pass fails first) and is diff-EMPTY with its pin. (3) The
  declared `pub(crate)` raise of `move_removal_candidates` - mechanical,
  zero behavior, doc states SM-I6 (no public surface moves); the function
  BODY is the verbatim mirror of validate's `collect_move_removals`
  (same filters, same `BTreeSet` - I read `crates/validate/src/lib.rs:347-372`
  against `extraction_reader.rs:374-402`). (4) Required test 5: rename to
  `test_dry_run_reports_move_truth_on_the_cpp_fixture` (m19 CW-1 rename
  precedent, declared) + the flip to `assert!(dry_run.move_removals > 0)` +
  `assert!(!dry_run.copy_only)` with the mandated message substance. Tests
  1-4, 6-10 and the honest-absent pin: no hunk (hunk boundaries old
  222-245/245-273/352-365/776-797; the pin at old :877-886 is outside every
  hunk).
- `extraction_apply/tests.rs`: FOUR hunks, all inside the mod line +
  required test 1's tree (the amendment clause's licensed region): the
  `mod move_proof;` line; the PRE-apply `expectation_pre_apply` binding
  (C5.1 - computed BEFORE `applier.apply`, the gate-2 authoring bug fixed
  without weakening anything); the :300 subset clause WIDENED (not deleted)
  from `approved_consumers` to `expectation.union` with the message citing
  SM-I2 - the AMENDMENT's authorized discharge, shape-C5 wording; the audit
  helper's declared hunk: claim shapes kept, inputs widened, the surviving
  CW-2 sub-claim `expected_modified_consumers == approved_consumers`
  preserved as its own assert, plus `assert_fixture_union` (C5.2 equality);
  the A-4 no-deletions claim is diff context (byte-identical); the
  `written == approved_new_files` + `!written.is_empty()` claims (old
  :260-271) diff-EMPTY (condition 3). `expected_modified_consumers`
  (old :345-380) diff-EMPTY. Required tests 2-11 (token/mutex/epoch/
  honest-outcome/ASCII/single-delegation): ZERO hunks.
- `extraction_apply/tests/move_proof.rs` (495 lines): the C5 oracle per the
  shape's section-8 names. Independence is real: markers parsed by the
  PUBLISHED grammar via this module's own constants (not `lang-cpp`
  internals); candidates read via the reader's own mirror; the fold
  re-runs validate's offer sequence (same order, same hook, `plan.id.0`)
  fully PRE-apply and read-only. Marker proof (C5.4): balance, pair count
  == hook-success count (with the count-equality assert PRECEDING the zip,
  so no silent truncation), plan-id binding on BEGIN and END (END asserts
  the no-symbols-field grammar), symbols field == the succeeded offer's
  names, prefix-strip reconstruction byte-equal (`uncomment_regions`,
  \r-tolerant, `split_inclusive('\n')` - byte-preserving), every marker line
  `.is_ascii()` (A9), re-offer over POST-apply text answers `None` for EVERY
  candidate covering the file (stronger than mandated: refusals included)
  with a `reoffers > 0` anti-vacuity guard. C5.5 parity:
  `assert_bound_names_total` sums `;`-split symbols over BEGIN lines vs
  `dry_run.move_removals` - the SM-I3 claim, machine-tested (green at
  landing, and RED pre-fold in the journal's gate-2 witness - falsifiable,
  not decorative). Condition 6 NOT re-proven here (rollback coverage is
  validate's `move_removal_transaction_test` green UNEDITED - my fresh
  validate run below). The non-vacuity trio exactly as named in the handoff;
  the two reject-guards use `catch_unwind` on synthetic specimens, both
  authored TERMINATED per the M3 carrier rule. Ordering determinism:
  BTreeSet/BTreeMap order and `sort()` throughout; the `HashMap` cache is
  lookup-only, never order-bearing.
- Journal: digest-shaped (canonical lines + capture paths + exit codes, zero
  raw dumps); all gate commands through `testrun.py --distill` /
  `distill.py --`; every D-row dated and itemized; the relay line verbatim.

## SM-I3 adjudication (the fold mirrors `strip_moved_symbols`)

Read against `crates/validate/src/lib.rs:386-400` (current): apply iterates
the `BTreeSet<(PathBuf, Vec<String>)>` and maintains the running text
THROUGH DISK - each `Some` is written back before the next iteration reads
the file. The reader's fold maintains the identical running text IN MEMORY
(per-path cache) and never writes. Offer sequence, order, text each offer
sees, hook, and plan-id argument are identical, so the fold "equals exactly
what the apply will bind" holds on the merits: same candidate set
(verbatim-identical derivation), same order, same fold semantics, same
offer input. The one asymmetry - apply aborts (`?`) on an unreadable file
where the fold `continue`s - is fail-CLOSED (apply aborting binds NOTHING,
so no over-report is possible) and is the pre-existing old-loop shape, not
introduced here. The fixture witness lands exactly on the SM-2 planner
verdict: ONE merged region, `symbols=Thing`, `plan=PLAN-001`; the oracle
pins the machine truth, never the shape's relayed "4 pairs". Measured
fixture count 1: witnessed three independent ways in my own runs - the
green `> 0` test-5 assert, the green C5.5 parity (bound == claim == 1), and
the green union-derivation test.

## HARD-CHAIN discharge (both commits named - carrier 2 + SM-3 verdict follow-up)

- SM-3 landing `21cf779c599fb7b38fe935303cec20b27dd8d7e0`: landed carrying
  EXACTLY one designed workspace red per its AMENDMENT-1 (the
  `tests.rs:300-304` subset lock; the planner's supersession of kickoff
  section 7 item 3's "tests 1-4 UNEDITED" relay phrase for that clause is
  RECORDED in the SM-3 handoff AMENDMENT-1 lines 433-437 - I read it, the
  widening is authorized, not improvisation).
- SM-4 landing `e717ba0866eda1d1f8117e8f6b427ee73171e175`: discharges it by
  the C5 widening at authoring (witnessed: the journal gate-2 panic site
  moved PAST the :300 clause to the new C5.5 parity claim - clause order in
  the test body proves :300 passed at authoring, exactly as the carrier
  predicted) plus the fold (which closes the parity at Green).
- THE DISCHARGE LINE, my own fresh run
  (`python tools/testrun.py --distill`, exit 0, capture
  `target/testrun-capture.txt` 3212 lines):
  `1852 passed / 0 failed / 13 ignored / 119 suites`
  Exit 0 at EXACTLY the carrier's landing expectation. This line proves
  SM-3's threading green WORKSPACE-WIDE and SM-4's oracle green - the
  hard chain is CLOSED for both hands. Reconciliation: anchor
  `1837/0/13/119 @ b201aa6` + SM-1 +2 + SM-2 +10 + SM-3 +0 + SM-4 +3 =
  1852 (the +3 = the declared trio, green-by-name in my scoped cli run
  `182 passed / 0 failed / 8 ignored / 8 suites`, which matches the
  journal's Green witness exactly; the 170/1 pre-hand fail-fast figure is
  the journal's SM-3-era witness, corroborated independently by the SM-3
  verdict's own run, not re-measured in this hop).

## Fresh gates - this reviewer's own runs (verbatim)

- Full census (THE discharge, both hands): `1852 passed / 0 failed / 13 ignored / 119 suites`, exit 0.
- `--baseline`: refused by the tool itself - verbatim `usage error: missing value for --baseline`; the FILE form stays superseded by the standing errata; the `--distill` substitute law governs (census run above).
- Scoped cli `python tools/testrun.py --distill -p ariadne-cli`: `182 passed / 0 failed / 8 ignored / 8 suites`, exit 0 - capture names green: `test_dry_run_reports_move_truth_on_the_cpp_fixture ... ok`, `test_a_valid_token_applies_the_plan_end_to_end ... ok`, all three `move_proof::` tests, `pin_degraded_dry_run_is_honest_absent_and_never_cached ... ok`.
- validate `-p ariadne-validate`: `33 passed / 0 failed / 0 ignored / 5 suites`, exit 0 (`move_removal_transaction_test` inside - SM-I7 condition 6, UNEDITED).
- m18 `--test m18_cpp_plugin_parity_test`: `23 passed / 0 failed / 0 ignored / 1 suites`, exit 0 (SM-I4 consumer-half, UNEDITED).
- m00 `--test m00_facts_path_parity_test`: `2 passed / 0 failed / 0 ignored / 1 suites`, exit 0 (boot parity on the UNAPPLIED fixture; R9 stated).
- Tools suites `python -m unittest discover -s tools -p "*_tests.py"`: `Ran 296 tests in 28.305s` / `OK`.
- ui_smoke: the PLAIN form `python tools/ui_smoke.py` was refused - boilerplate catch-all `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`; the sanctioned distiller form `python tools/distill.py -- python tools/ui_smoke.py`: `### generic PASS (exit 0)`, `boot+route checks clean (cold fixture, attempt 1, 1.0s)`, `ui_smoke: PASS`.
- bench `python tools/distill.py -- python tools/bench_compare.py` (fresh, this hop): `### generic PASS (exit 0)` ... `bench compare: 3 run(s), no regressions past 1.25x baseline`; the flagged row `get_or_build_include_graph clone (cold)` median `1.5` ms, warm `0.9` ms. SEE BENCH ATTRIBUTION BELOW.
- hygiene `python tools/hygiene_check.py`: output = ONLY warn-only R3 lines; exit witness via distiller `### generic PASS (exit 0)`, capture 173 lines - ALL pre-existing, ZERO lines in any of the four authored files, zero R1/R2 (the tool law: R3 blocks only via a dated ledger row; none exists).
- `python tools/encoding_guard.py --all`: silent; exit witness via distiller `### generic PASS (exit 0)`, capture 0 lines. Authored bytes pure ASCII (my own visual check of every hunk concurs).
- `python tools/theme_token_check.py --check-added --base a33f620`: silent (zero added hex literals - the hand is cli-side only).

## Bench attribution verdict (carrier duty: "if WARN persists, adjudicate")

The implementer's recorded WARN (`get_or_build_include_graph clone (cold):
1.8 ms vs baseline 1.3 ms (+38%)`, recorded NOT attributed) did NOT
reproduce in my fresh run: exit 0, NO WARN, cold median 1.5 ms (ratio 1.15,
inside the 1.25 gate), warm 0.9 ms BELOW its baseline. Adjudication, on the
merits and with the code-attribution test the dispatch demanded:

1. STRUCTURAL: the flagged span is `pipeline.get_or_build_include_graph()`
   (`crates/pipeline/src/lib.rs:462`, timed at
   `crates/cli/tests/payload_tail_bench.rs:315-318`). `crates/pipeline` is
   ZERO-DIFF across the entire SM-1..SM-4 range; the bench target file
   itself is zero-diff; and the bench harness contains NO reference to
   `dry_run`/`move_removal`/extraction-reader code - this hand's fold never
   executes inside any measured phase. A code attribution to SM-4 is not
   constructible.
2. CORROBORATION: SM-3's journal bench WARN hit a DIFFERENT row (warm,
   1.3 vs 1.0, +30%) at a landing whose surface was `crates/validate` +
   m19/m20; SM-4's hit the cold row. Two disjoint hands, two different
   1-ms-scale rows, one WARN each, neither reproducing under an idle
   machine - the signature is run-time load / stale-baseline (PR-2), not
   code.
3. VERDICT: NOT attributable to SM-4 (nor to SM-3) on the code path. The
   WARN class travels to closeout under the standing PR-2 (baseline
   refresh) deferred row, now with a fresh exit-0 recording AT the
   milestone's code head. The milestone bench stance at this hand: fresh
   recording PASS, zero WARN, exit 0.

## Deviation dispositions (independent adjudication)

- D-a (D-2026-10-24a, Red showed TWO failures where the carrier predicted
  one): ACCEPTED. The second failure was legitimately NAMED (the same
  required test 1, `test_a_valid_token_applies_the_plan_end_to_end` - the
  original RGB protocol's "EXACTLY two named failures" pair) and legitimately
  TYPED (the new C5.5 SM-I3 parity claim - which CANNOT be green pre-fold;
  the pre-fold parity IS the R5 window the carrier itself exists to close).
  The carrier's SUBSTANCE held: the :300-304 designed-red clause went GREEN
  at authoring - proven mechanically by the Red panic site moving past
  :300 to the parity claim, which executes later in the test body. The
  carrier's "EXACTLY ONE" prediction is F14-class optimistic relay (same
  class as the shape's 4-pair relay), correctly re-claimed by the journal.
  No STOP warranted (both failures named, documented-reason Red witnesses);
  no assertion weakened; the intermediate authoring-bug capture (`left: []`
  from a POST-apply expected-consumers computation) was honestly journaled
  and fixed by hoisting the bindings PRE-apply - the fix moved WHERE the
  computation happens, never WHAT is claimed.
- D-b (D-2026-10-24b, gate-budget overshoot 15 scoped invocations vs 8
  declared): ACCEPTED, dated and itemized per the 2026-10-06 M28 law
  (silence is the deviation). Root causes: two self-caused re-captures +
  the two inherited tool-surface quirks (cargo last-`--test`-keeps-one;
  the `m00_facts_path_parity` short-name relay vs the `_test` manifest
  target - my own per-target runs independently corroborate both). Travels
  to the ledger dated sub-block at closeout.
- D-e (parallel-run DRIFT, journaled inside D-b): ACCEPTED. Two concurrent
  `testrun` runs raced the shared capture file + cargo target lock (exit 2
  DRIFT); the SERIALIZED re-run is the correct fix - my hop ran every gate
  serially, zero drift, corroborating the rule.
- D-c (D-2026-10-24c, `move_proof.rs` 495 lines vs the ~300-340 sizing):
  ACCEPTED. The sizing is a planner estimate, not a ceiling; R2 (< 1000)
  holds with wide margin; every function well under 100 (longest
  ~48 - `derive_move_fold`); hygiene exit 0 confirms; the overshoot is
  declared with its causes (module-header invariant prose, granular
  per-claim helpers required for the synthetic non-vacuity pair, the
  fuller union-witness test).
- The helper-hunk reading conflict (binding input 8 "byte-identical" vs
  handoff item 4's DECLARED hunk for `assert_modified_faithfully_applied`):
  ACCEPTED as journaled. Item 4's explicit "declared hunk ... claim shapes
  kept, inputs widened" + the frozen-lock amendment clause ("the one
  changed test file is required test 1's module tree") are the operative
  text; the two claims inside the helper both SURVIVE (one input-widened,
  the CW-2 sub-claim byte-for-byte in shape with its attribution message);
  the A-4 no-deletions claim is diff context;
  `expected_modified_consumers` is diff-EMPTY. Nothing weakened.
- M4 carrier (R-G/R-D reachability): ACCEPTED no-action. The fold mirrors
  the strip's OFFER sequence and text flow only - it never re-implements a
  refusal branch (verified: the fold contains no refusal logic; refusals
  come from the hook's `None`).

## Findings (numbered; all minor/informational - none requires a return)

1. M1 (informational, resolves the carrier's open WARN): the journal's
   gate-7 line recorded exit 1 + the cold-row WARN; my fresh recording at
   the landing is exit 0 with NO WARN (cold 1.5 ms). Machine-truth
   discrepancy resolved in the code-impossibility + load direction (bench
   attribution section). Closeout should cite THIS verdict's recording for
   the SM-4 hand, not the journal's transient one.
2. M2 (gate-coverage honesty, allow-list class, re-confirmed from SM-3
   M4): the plain `python tools/ui_smoke.py` invocation is not on the bash
   allow-list (verbatim refusal: `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`);
   the sanctioned `distill.py --` form ran and PASSed. The `--baseline`
   FILE form stays refused (`usage error: missing value for --baseline`) -
   the `--distill` substitute law applied. No weaker evidence substituted
   anywhere without saying so here.
3. M3 (line-count drifts, all declared or margin-covered): `move_proof.rs`
   495 vs ~300-340 sizing (D-c); `extraction_reader.rs` 926 vs the `~910`
   estimate (margin noted in the handoff itself, no split owed, < 1000);
   `tests.rs` EXACTLY 914 with the journal's headroom statement verified (4
   to the 918 WATCH). The R1 caught-and-fixed loop (journal Gate-8/R1 row:
   test-1 body hit 103, hoist restored it; my hygiene exit 0 is the fresh
   witness - measured body 240..338 = 99 lines).
4. M4 (evidentiary, minor): the journal's gate-2 panic coordinate
   `tests\move_proof.rs:315:5` cannot be re-derived from history (the Red
   state was never committed; the hand landed as one squashed commit and
   the refactor shifted line numbers). The MESSAGE body it quotes matches
   the landing file's `assert_bound_names_total` claim verbatim and the
   `left: 1 / right: 0` direction is consistent with pre-fold reality, so
   the witness is adjudicated on message substance and clause-order
   mechanics (both check out). Future RGB protocols: a Red-state blob (even
   a stash-free `git hash-object -w` note is NOT available to implementers
   here) - practical alternative: journal BOTH coordinate and message at
   capture time and mark the squashed-landing caveat, as this journal
   effectively did.
5. M5 (informational): the `crates/ui` pre-existing unused-code warnings
   (`export.rs` `INDEX_HTML`, `api_graph.rs:755` `gen`) re-visible in
   captures - untouched (out of touch-set), report-only, exactly as the
   journal routed them.
6. M6 (planner relay, standing): the carrier's off-by-one corrections are
   now MEASURED TRUTH at my hop - SM-2 landing `1849/0/13/119`, SM-3 delta
   +0, SM-4 +3, landing `1852/0/13/119`. The `1848` relay class is dead;
   future dispatches quote `1852/0/13/119` as the post-SM4 baseline.

## Carry-forwards (never dropped)

- GENERATION-SWAP relay: RECORD verified, NOT EXECUTED -
  `extraction_apply.rs` diff-EMPTY over `a33f620..e717ba0` (my own check,
  above); the relay line sits verbatim in the landed journal. The closeout
  appends the dated relay row per kickoff section 4 (SM-5 duty).
- "optimize-verb move parity" (path-derivation unfreeze): ledger
  FOLLOW-ON row landed at `a33f620`; m19 deferral tripwire intact (m19
  green UNEDITED inside my full census); closeout re-registers per kickoff
  section 6. NOT SM-4 scope; nothing touched.
- Bench PR-2 (stale baseline) deferred row: now carries this hop's fresh
  exit-0 recording at the milestone head (finding M1).
- SM-5 dispatch carrier: baseline `1852 passed / 0 failed / 13 ignored /
  119 suites` post-SM-4 (measured truth governs, relays never do); SM-5 is
  the ui hand (C6 copy-truth + kickoff section 7 item 4's S5 sentence
  amendment). No release was made in the SM-3->SM-4 window (the milestone
  forbids shipping one; hard chain closed).
- State of record after SM-4: HEAD `e717ba0` on `main`; tree clean except
  the tolerated `.vsix`; reflog terminates at the landing.

## SM-5 duty-1 precision (this hop's artifacts)

`tasks/extraction-symbol-move-sm4-verdict.md` and
`tasks/extraction-symbol-move-sm4-execution-results.md` are UNCOMMITTED by
design (reviewer boundary: tasks-only writes, no commits). SM-5's duty-1
must therefore EXPECT FIRST a planner ledger write - `tasks/milestone_ledger.md`
dirty with the SM-4 record row (approval + this landing SHA `e717ba0` + the
discharge census) and any SM-5-handoff amendment the planner issues - and
must land, in ONE scoped docs commit: (a) the planner's dirty
`tasks/milestone_ledger.md` (SM-4 record), (b) this verdict + the
execution-results addendum, (c) any SM-5 handoff amendment alongside. The
boot porcelain at SM-5 duty-1 must match that set exactly; anything beyond
it is STOP-and-report. The ledger's SM-4 row should carry: APPROVED, the
both-hands discharge line, the bench attribution verdict (fresh exit 0, no
WARN), D-a..D-e dispositions, and the generation-swap relay-still-open note.
