# SM-3 Verdict - extraction-symbol-move (plan-id threading + m20 rider + m19 deferral lock; HARD CHAIN to SM-4)

Review-gate adjudication of handoff SM-3, contract of record
`tasks/extraction-symbol-move-sm3-handoff.md` INCLUDING its dated
AMENDMENT-1 (2026-10-24, planner) - the amendment supersedes RIDER 2, the
gate-7 census expectation, and the landing commit message. Anti-phantom law
observed: every SHA cross-checked against `.git/logs/HEAD` BEFORE any gate was
trusted; every number below is from MY OWN runs, quoted verbatim.

## Verdict: APPROVED

No blockers. Five minor/informational findings (M1-M5); none requires an
implementer return. The mid-hand STOP, the amendment execution, and all
declared deviations are ACCEPTED (independently adjudicated, dispositions
below). The landing carries EXACTLY the one workspace red the amendment
authorized, no more.

## Anti-phantom verification (reflog ground truth)

`.git/logs/HEAD` read directly with the Read tool (total 444 lines - the file
ENDS at the landing entry).

Line 443 verbatim (duty-1):

`fba3c902f1970b2673b4359c2717334c0b1d4898 790fe75900c041347d68a97a9bb7b36bfc3bc848 kcwill2 <kcwill2@gmail.com> 1792830679 -0700	commit: docs(extraction-symbol-move): SM-2 APPROVED - land verdict + execution-results + ledger SM-2 record (census 1849 verified)`

Line 444 verbatim (SM-3 landing):

`790fe75900c041347d68a97a9bb7b36bfc3bc848 21cf779c599fb7b38fe935303cec20b27dd8d7e0 kcwill2 <kcwill2@gmail.com> 1792836001 -0700	commit: feat(validate): bind the plan id into the move hook at the strip call; m20 marked-retention rider + m19 optimize-deferral lock; hard-chained to SM-4 (extraction-symbol-move SM-3)`

`git rev-parse HEAD` = `21cf779c599fb7b38fe935303cec20b27dd8d7e0`. Entries 443
and 444 are adjacent `commit:` entries - no reset, no amend, no stray commit
between duty-1 and landing. This independently buries the resume-tool stray-commit
scare (disposition D1 below): the truncated `commit_scoped.py` execution it
witnessed was a usage error (exit 2) that cannot commit, and no reflog entry
exists for one.

## Commit-vs-touch-set (the enforcement check)

Range `790fe75..21cf779` contains exactly ONE commit (reflog adjacency +
`git show` parentage). `git diff --name-only 790fe75..21cf779` returned
exactly the four declared touch-set paths - nothing else:

- `crates/validate/src/lib.rs` (numstat 15/10 - the `| 25 +-` stat form the
  dispatch quoted)
- `tests/integration/tests/m19_optimize_characterization_test.rs` (17/0 -
  PURE ADDITIONS, dispatch's `+17/-0` held)
- `tests/integration/tests/m20_apply_migrate_characterization_test.rs`
  (25/2 - the `| 27 +-` stat with the two declared deletions)
- `tasks/extraction-symbol-move-sm3-journal.md` (448/0 - journal rides the
  landing commit, 448 lines)

Cargo.lock: not in the diff. `git diff --numstat 790fe75..21cf779 -- crates/validate/tests crates/cli crates/transform crates/lang crates/lang-cpp crates/ui tests/fixtures tools` printed ZERO lines: every frozen surface zero-diff.
The planner's three uncommitted docs (handoff +209 AMENDMENT-1 append, sm4
handoff +62 carrier append, ledger +5/-2 records - per `checkpoint_guard.py
status` numstat) are NOT inside the commit; they ride their own duty (below).

## Per-commit review (single commit, judged alone)

- `crates/validate/src/lib.rs`: exactly the licensed ~10-line threading -
  `strip_moved_symbols` gains `plan_id: &str`; the strip call passes
  `&migration.plan_id`; the hook offer becomes
  `rewriter.move_out_top_level_symbols(&content, names, plan_id)`. NO
  signature change on `apply_with`/`rollback`/`verify_restored`; phase order
  (rewrites THEN strip) unchanged; `collect_move_removals` BODY byte-identical
  (its doc's FIRST sentence swaps only the hook name - the declared lift);
  snapshot/rollback regions untouched (SM-I7). Exactly the THREE declared doc
  lifts, each matching the clause's wording mandate (move-hook decides,
  refusal/copy-only answers `None`, copy fallback survives, D-M23-5/I5/I2
  references kept); new doc bytes pure ASCII (the removed em-dashes are
  deletions inside the licensed lift). Validate names no language (I2).
- **M1 closure (from the SM-2 verdict), at source AND observable level:**
  source - the strip call-site forwards names and `&migration.plan_id`
  verbatim into `move_out_top_level_symbols`; the SM-1 default
  (`crates/lang/src/rewriter.rs:118-124`) delegates to the old hook, the C++
  override (`crates/lang-cpp/src/dependency_rewriter.rs:122-128`) forwards
  `plan_id` into `move_retention::move_out_text`. Observable - the m20 rider
  asserts the disk BEGIN line byte-EQUALS
  `// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-001 symbols=Thing` (single-element
  vec equality pins count AND bytes): a dropped/swapped plan id or name at
  the call makes m20 fail. My fresh run witnesses it green. The SM-2 verdict's
  test-mandate is met by the strongest available witness (disk observation,
  not a fake). Amendment's rider-1-pin-as-M1-carrier claim: TRUE on the merits.
- No retained dual representation: the strip call offers exactly ONE hook
  (the move hook); the old hook survives only as the SM-1-designed delegation
  target, as frozen.

## Amendment compliance (AMENDMENT-1 - adjudicated independently)

- m19 committed block vs the amendment's quoted NEW bytes (handoff lines
  372-386): line-by-line identical (15 lines). Method honesty: I could not
  run an inline byte-compare script (`python -c` refused; edit grant is
  tasks/-only), so I established identity by full-text comparison of both
  renderings plus grep-verified absence of trailing whitespace in both regions
  and pure ASCII on both sides (grep `[^\x00-\x7F]`: zero matches in m19). The
  tripwire clause wording is verbatim, including "never delete it silently".
- m19 vs start rev: PURE ADDITIONS (17/0), hunks ONLY inside
  `optimize_writes_all_extracted_headers_and_rewrites_consumer_includes`;
  report-count asserts survive as diff context; test count stays 5.
  `git log -S uncomment_marked_regions -- <m19>` printed NOTHING: the
  withdrawn proof-block bytes NEVER reached git history, so the per-commit
  diff legitimately sees only the new block - the amendment's claim verified.
- Strengthening, never weakening: the block is a pure addition of constraint
  (nothing asserted before it on the original file in m19), it asserts
  EQUALITY with the pre-hand original (`thing_header_after ==
  thing_header_before`, before-capture at first optimize-read moment), and
  the tripwire FAILS if the follow-on lands. The machine-witness claim inside
  the message is TRUE by my own reads: `crates/cli/src/optimize.rs:320`
  (`PathBuf::from("generated").join(&unit.path)`), `optimize.rs:407`
  (`apply_with(..., &extraction, ...)` - the UN-relocated extraction), and
  the frozen predicate `crates/validate/src/lib.rs:362`
  (`!rewrite.new_includes.contains(&unit.path)`) - prefixed `new_includes`
  never contain the unprefixed `unit.path`, hence ZERO candidates and no hook
  offer for the optimize payload; green-at-birth logic holds.
- RGB adjudication accepted: Red witnessed at gates 2a/4b (same panic site,
  verbatim in the journal); Green witnessed FRESH BY ME (`5/0/0/1`).
  Green-at-birth here is a characterization of witnessed machine truth with
  an unfreeze tripwire - the "never weaken" law is satisfied on the merits,
  not rubber-stamped.
- m20 rider under the closed policy (kickoff sec. 7 item 2): the deleted
  bytes are EXACTLY the two declared message lines (journal-quoted start-rev
  bytes, confirmed as the only `-` lines in the diff); the assertion
  expression line `thing_impl_after.contains("Thing::Thing(ThingConfig config)"),`
  is byte-identical diff context; the manifest/outcome/tier/generated-list
  block and the CW-1 consumer-side block carry ZERO hunks (byte-identical by
  diff construction); every other test in m20 untouched; test count stays 6.
  The BEGIN/END equality, balance, reconstruction, and no-live-declaration
  proofs are all present as specified; m19 carries NO plan-id literal
  (prefix-free block), m20 owns the `PLAN-001` byte - as ordered.

## Fresh gates - this reviewer's own runs (verbatim)

Bootstrap `python tools/checkpoint_guard.py status`: HEAD `21cf779`, main,
dirty = `.vsix` + the three planner docs, untracked = the resume report,
worktree census as recorded in the execution-results addendum. All scoped
runs via `python tools/testrun.py --distill`, exit codes as the tool
reported (PASS/FAIL headers), capture `target/testrun-capture.txt` each run:

- validate: `33 passed / 0 failed / 0 ignored / 5 suites`
- m19: `5 passed / 0 failed / 0 ignored / 1 suites`
- m20: `6 passed / 0 failed / 0 ignored / 1 suites`
- delegation proof: `2 passed / 0 failed / 0 ignored / 1 suites` +
  `git diff --numstat 790fe75..21cf779 -- crates/validate/tests` ZERO lines
  (file byte-identical, green THROUGH the default forwarder = SM-I5/SM-I7)
- m10 `5 passed / 0 failed / 0 ignored / 1 suites`; m14
  `3 passed / 0 failed / 0 ignored / 1 suites`; m17
  `10 passed / 0 failed / 0 ignored / 1 suites`; m18
  `23 passed / 0 failed / 0 ignored / 1 suites`; m23
  `7 passed / 0 failed / 0 ignored / 1 suites` (each its own invocation -
  the cargo duplicate-`--test` quirk the journal named is real; my
  single-invocation runs each sampled exactly one target, corroborating it)
- lang `47 passed / 0 failed / 1 ignored / 2 suites`; lang-cpp
  `215 passed / 0 failed / 0 ignored / 2 suites` (unchanged)
- HARD-CHAIN proof, extraction_apply scoped
  (`python tools/testrun.py --distill -p ariadne-cli --lib extraction_apply`
  - adapted from the dispatch's example because NO test TARGET named
  `extraction_apply` exists in `crates/cli/tests/`; the lock is a lib unit
  test): exit 1, `12 passed / 1 failed / 0 ignored / 1 suites`. The single
  failure is the named lock, panic verbatim:
  `thread 'commands::ui::extraction_apply::tests::test_a_valid_token_applies_the_plan_end_to_end' (8584) panicked at crates\cli\src\commands\ui\extraction_apply\tests.rs:300:9:`
  `the apply must not modify any file outside the approved dry-run's consumer_file set: modified=["\\\\?\\C:\\Users\\Kurt\\AppData\\Local\\Temp\\.tmpO1Gkzl\\basic\\include\\thing.h", ...src\\thing.cpp, ...tests\\consumer.cpp"]`
- Widened corroborator, full cli lib binary
  (`python tools/testrun.py --distill -p ariadne-cli --lib`): exit 1,
  `170 passed / 1 failed / 0 ignored / 1 suites` - the ONLY red in the entire
  cli lib is the named lock (matches the journal gate-7 witness's
  `170 ok + 1 FAILED`). The failing assertion
  (`modified.iter().all(|file| approved_consumers.contains(file))`,
  tests.rs:295-304, PRE-EXISTING UNEDITED) shows the violation limited to
  `include\thing.h`: the other two modified files are the approved dry-run's
  own consumer set. No additional reds.

Workspace-wide acceptance runs in the execution-results addendum: tools
suites `Ran 296 tests in 27.087s / OK`; `encoding_guard --all` silent;
hygiene zero R1/R2 lines.

## Deviation dispositions

- D1 resume-tool parse quirk + stray-commit-scare: ACCEPTED. `--cmd` takes
  one line token (the tool's own usage output, quoted in the resume report);
  quoting the command preserved content. The embedded-markdown `commit_scoped`
  execution was a usage error (exit 2, `error: argument --scope` - the process
  never reaches commit); HEAD truth `21cf779` + adjacent reflog entries 443/444
  prove no stray commit ever occurred. The resume report's red-m19 pre-edit
  verdict matches the journal's expected boot state.
- D2 planner-doc boot-dirt: ACCEPTED. All three extras are the ratified
  amendment's own issuance artifacts (verified by reading their working-tree
  diffs: the AMENDMENT-1 section, the SM-4 hard-chain carrier, the ledger
  STOP/amendment/follow-on rows). None is a foreign hand; none rode the landing
  (touch-set = 4 paths); leaving them uncommitted for their own duty is
  correct discipline, not improvisation.
- D3 gate-count overshoot D-row (16 measured vs 9 declared): ACCEPTED, dated,
  itemized, silence-averted per the 2026-10-06 M28 law; root cause
  independently corroborated by my own per-target gate runs. Travels to the
  ledger dated sub-block at closeout; the per-test-gates re-state is ACCEPTED
  milestone-wide per the amendment.
- D4 gate-7 census honoring: COMPLIANT. The journal's stop witness
  (`259 passed / 1 failed / 0 ignored / 3 suites`) stands as the designed-red
  record; the amendment exempts the census from this landing; my substitution
  duties (scoped set + single-red proof) discharged above.

## Findings (numbered; all minor/informational)

1. M1 - planning-phase off-by-one: the handoff quotes the SM-2 landing census
   as `1848/0/13/119`; measured anchor is `1849/0/13/119` (journal BASELINE
   CORRECTION + ledger CENSUS-BASELINE CORRECTION row, both already recorded).
   No action; SM-3 delta +0 holds under my own counts (m19 5 tests, m20 6
   tests, no target added anywhere in the four-path touch-set).
2. M2 - journal estimate drift: the amendment-execution section estimates the
   m19 block at "16 net lines"; the landing stat is `+17` (before-capture +
   blank + 15-line block + fence discipline). The journal itself defers the
   exact figure to the commit; recorded truth here. No action.
3. M3 - reviewer-side limits (stated, not concealed): full workspace census
   and `--baseline` NOT run by me (amendment-exempt; fail-fast would stop at
   the designed red anyway); `ui_smoke` and `bench_compare` NOT re-run this
   hop (not SM-3 hand gates; the journal gate-8 fresh recording stands -
   exit 1 with the standing unattributed PR-2 WARN `WARN
   get_or_build_include_graph clone (warm): 1.3 ms vs baseline 1.0 ms (+30%)`).
   Workspace-wide both-hands proof discharges at SM-4 gate 6 `1852/0/13/119`.
4. M4 - hygiene exit code not echoed (chained `echo` refused by the allow
   list - same class as the SM-2 verdict's M5 allow-list row, re-confirmed):
   my `hygiene_check.py` run printed ONLY warn-only R3 lines (tool law at
   hygiene_check.py:453: R3 blocks only via a dated ledger row; none exists)
   and zero R1/R2 - a pass by design. Refused-then-substituted attempts this
   hop, all boilerplate catch-all denies: `git status --porcelain` (plain
   `git status` ran fine), `git grep` (blob checks done via checkpoint_guard
   numstat + commit name-only instead), inline `python -c` (byte checks via
   Read/grep/`git log -S` instead).
5. M5 - informational: m20:111 carries a pre-existing legitimate em dash
   OUTSIDE every SM-3 hunk (`encoding_guard --all` silent); untouched by
   design; no action.

## Follow-ups routed (never dropped)

- SM-4 duty-1 carrier (precise duty): land TOGETHER in one scoped docs commit
  (a) the planner's three uncommitted docs -
  `tasks/extraction-symbol-move-sm3-handoff.md` (+209 AMENDMENT-1),
  `tasks/extraction-symbol-move-sm4-handoff.md` (+62 carrier),
  `tasks/milestone_ledger.md` (+5/-2, and its SM-3 row updated with THIS
  landing SHA `21cf779` + this gate), (b) my two artifacts
  `tasks/extraction-symbol-move-sm3-verdict.md` +
  `tasks/extraction-symbol-move-sm3-execution-results.md` (both UNCOMMITTED,
  written by this hop), and (c) the resume report
  `?? tasks/extraction-symbol-move-sm3-handoff-resume.md` - disposition per
  the SM-1 precedent (deleted at the duty-1 landing that carries its verdict;
  journal the deletion).
- HARD-CHAIN discharge: SM-4's gate 1 expects EXACTLY the one named red
  (journal its panic verbatim - NOT a STOP; anything beyond it IS); the C5
  growth at `extraction_apply/tests.rs` discharges it at authoring; SM-4 gate
  6 census `1852/0/13/119` proves BOTH hands; SM-4's verdict MUST name both
  landing commits (`21cf779` + SM-4's) in its gate-coverage section citing
  the carrier. Adjacency law: SM-4's code landing is the IMMEDIATE next
  landing after SM-3's (only duty-1 doc landings between; NO RELEASE in the
  window). Reviewer adjacency check owed at SM-4.
- M3/M4 carriers (SM-2 verdict): terminator-oracle rule (binding on C5.4
  byte-reconstruction) and the R-G/R-D pair awareness - both already relayed
  verbatim into the uncommitted SM-4 carrier sections 3-4; SM-4 implements
  from there.
- Follow-on "optimize-verb move parity" (path-derivation unfreeze):
  ledger FOLLOW-ON INTAKE row, 2026-10-24 (uncommitted - rides the docs
  commit above); tripwire = the m19 deferral lock going red; closeout
  re-registers per kickoff sec. 6. NOT SM-4 scope.
- D-2026-10-24 SM-3 gate-overshoot D-row (16 vs 9): carry into the ledger
  dated sub-block at closeout (alongside D-SM1-REDO-1 and the SM-2 D-row);
  the ACCEPTED per-test-gate re-state applies to all future hands.
- Gate-8 bench WARN (PR-2) stays unattributed, travels to closeout census
  attribution (standing rule).
- SM-5 awareness: dispatch baseline `1852/0/13/119` post-SM-4 (the ledger
  correction row governs; relayed numbers never govern).

## State of record after this gate

- HEAD `21cf779c599fb7b38fe935303cec20b27dd8d7e0` on `main` (reflog line 444);
  tree: `.vsix` tolerated + the three planner docs + my two verdict artifacts
  (uncommitted by design) + the resume report (untracked, disposition above).
  Reflog terminates at the landing - no stray commits.
- Scoped evidence: every gate line above is my own fresh run; the only red
  anywhere I measured is the single designed lock (cli lib 170/1, scoped
  12/1); SM-3 delta +0 holds.
- HARD-CHAIN status: main carries EXACTLY one designed workspace red
  (`test_a_valid_token_applies_the_plan_end_to_end`) per AMENDMENT-1; SM-4's
  hop discharges it and proves both hands at `1852/0/13/119`. No release in
  the window. SM-4 dispatch carrier CONFIRMED (its uncommitted AMENDMENT-1
  CARRIER section states the duty; ledger rows record it).
