# SM-5 Verdict - extraction-symbol-move: move-truth copy + S5 keying amendment + marked-retention no-rot lock

Verdict: **APPROVED** (zero blockers). Reviewer hop 2026-10-24; final hand of
the milestone. ANTI-PHANTOM LAW honored: every SHA verified against
`.git/logs/HEAD`, every number below from this reviewer's own fresh runs.

## 1. Reflog verification (verbatim, `.git/logs/HEAD` lines 435-449)

Expected duty-1 `fd96b09` (447), landing `82cf789efa92c7c311bf16367313ead4000f8f12`
(448), rider `522a770f68abda4ca5cf0dd4604108bd285de1a1` (449) = HEAD — all three
confirmed exact. Full chain (Read tool, the SM-2 M5 sanctioned route; raw
`git reflog` refused by the allow-list, boilerplate catch-all):

```
435: 83f0b7c8f98aebead0e0d807ee488b458ca2060c b201aa6ca12ca93a870a02f1ffd2e49ed32eb86f ... commit: docs(cw-3): journal landing record - docs 90beaef + landing 83f0b7c (extraction-consumer-rewrite CW-3)
436: b201aa6ca12ca93a870a02f1ffd2e49ed32eb86f 64664550d4bd672747e657c7222a63cb88d54634 ... commit: docs(extraction-consumer-rewrite): CLOSED 1837/0/13/119 @ b201aa6 - three hands APPROVED, five deferred rows discharged
437: 64664550d4bd672747e657c7222a63cb88d54634 2177a85ff200797ab08bc49b510157fabb21afbf ... commit: docs(extraction-symbol-move): kickoff + shape - move-by-default via marked comment-out, user decision of record
438: 2177a85ff200797ab08bc49b510157fabb21afbf e5c49ec6a4891d195d0cbbf14c1ca271a15a1117 ... commit: docs(extraction-symbol-move): plan - 5 serial hands SM-1..SM-5, PLANNING COMPLETE 2026-10-23
439: e5c49ec6a4891d195d0cbbf14c1ca271a15a1117 b9936000da7827e49e8dbe999f1dd65d65b75704 ... commit: feat(lang): add the neutral move_out_top_level_symbols contract with a delegating default (extraction-symbol-move SM-1)
440: b9936000da7827e49e8dbe999f1dd65d65b75704 0db2680c34515e03d4589d101dca010fd564f86d ... commit: docs(extraction-symbol-move): SM-1 re-gate APPROVED - land verdict + execution-results + incident ledger row (PHANTOM-NARRATION-1 closed)
441: 0db2680c34515e03d4589d101dca010fd564f86d f6c92feed6137c4668de0191486164f8970141f7 ... commit: feat(lang-cpp): implement move_out_top_level_symbols with the ARIADNE-EXTRACT-OUT line-prefix grammar and refusal contract (extraction-symbol-move SM-2)
442: f6c92feed6137c4668de0191486164f8970141f7 fba3c902f1970b2673b4359c2717334c0b1d4898 ... commit: docs(extraction-symbol-move): SM-2 journal landing record (reflog-verifiable SHA f6c92fe)
443: fba3c902f1970b2673b4359c2717334c0b1d4898 790fe75900c041347d68a97a9bb7b36bfc3bc848 ... commit: docs(extraction-symbol-move): SM-2 APPROVED - land verdict + execution-results + ledger SM-2 record (census 1849 verified)
444: 790fe75900c041347d68a97a9bb7b36bfc3bc848 21cf779c599fb7b38fe935303cec20b27dd8d7e0 ... commit: feat(validate): bind the plan id into the move hook at the strip call; m20 marked-retention rider + m19 optimize-deferral lock; hard-chained to SM-4 (extraction-symbol-move SM-3)
445: 21cf779c599fb7b38fe935303cec20b27dd8d7e0 a33f6203777d18dbb6b60acf34c0a7b1cfc3abba ... commit: docs(extraction-symbol-move): SM-3 APPROVED - land verdict + execution-results + ledger SM-3 record + SM-3/SM-4 amendments
446: a33f6203777d18dbb6b60acf34c0a7b1cfc3abba e717ba0866eda1d1f8117e8f6b427ee73171e175 ... commit: feat(cli): dry-run move honesty via the strip-fold mirror; grown move-proof oracle; required test 5 move truth (extraction-symbol-move SM-4)
447: e717ba0866eda1d1f8117e8f6b427ee73171e175 fd96b0947d6124ac84902070a9a1f37bbbd38b40 ... commit: docs(extraction-symbol-move): SM-4 APPROVED - land verdict + execution-results + ledger SM-4 record (census 1852, hard chain closed)
448: fd96b0947d6124ac84902070a9a1f37bbbd38b40 82cf789efa92c7c311bf16367313ead4000f8f12 ... commit: fix(ui): move-truth copy - marked retention sentences, keyed S5, marked-retention no-rot lock (extraction-symbol-move SM-5)
449: 82cf789efa92c7c311bf16367313ead4000f8f12 522a770f68abda4ca5cf0dd4604108bd285de1a1 ... commit: docs(extraction-symbol-move): SM-5 journal landing record (gates 7-9, diff audit, deviations)
```

(`...` = author field `kcwill2 <kcwill2@gmail.com>` + timestamp; timestamps
439-449 monotonic, no gaps, no stray commits.)

Ground truth: `checkpoint_guard status` -> HEAD `522a770`, branch `main`,
ahead=0 behind=0, dirty exactly the tolerated
`extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`, no untracked.

## 2. Commit-vs-touch-set: PASS

`git diff --name-only fd96b09..522a770` -> exactly
`crates/ui/assets/extraction-wizard.js`,
`crates/ui/tests/ui_test/extraction_wizard_render.rs`,
`tasks/extraction-symbol-move-sm5-journal.md`. All three inside the declared
touch-set; no fourth path; `Cargo.lock` absent.

Per-commit, oldest-first:
- `82cf789` (landing): 3 files, `169 insertions(+), 4 deletions(-)` =
  asset 16/4 (= 20 +/- across exactly the 5 named hunks), render file
  57/0 (PURE APPENDS), journal 96/0. Nothing else.
- `522a770` (rider): journal only, +60/-0. Total journal 156/0.

Producer/frozen zero-diff across the hand:
`git diff --numstat e717ba0..HEAD -- crates/cli crates/lang crates/lang-cpp
crates/validate crates/transform tests/golden tests/fixtures tools Cargo.lock`
-> EMPTY (stronger than the journal's narrower `crates/cli/tests` gate-7 proof,
which is itself subsumed).

## 3. Plan-literal checks (asset: five hunks vs the mandated bodies)

- New constant block: 3 comment lines + ONE-line const, inserted directly
  below `COPY_VS_MOVE_SENTENCE` (:219), byte-matching handoff item 1; the
  sentence value is contiguous on asset :224 (the lock idiom).
- :359 move branch: 1:1, byte-matches handoff item 2.
- :392 move branch: 1:1, byte-matches handoff item 3 (8-space indent,
  `As it writes, this plan retires ${escapeHtml(dryRun.move_removals || 0)}
  source declaration(s): ${escapeHtml(MOVE_RETENTION_SENTENCE)}`).
- :430 -> keyed `moveTruthLine` block: byte-matches handoff item 4 verbatim,
  all 8 lines and the `    moveTruthLine +` concat line; null-safe pending
  branch present; copy branch keeps `COPY_VS_MOVE_SENTENCE`;
  `wizardState.dryRun.move_removals` WITHOUT `|| 0` exactly as mandated.
- :575: word-diff proves the reword is ONE clause
  (`move languages` -> `languages that stage units in place`), rest of the
  line byte-identical, line stays one line.
- `COPY_VS_MOVE_SENTENCE` (asset :219): appears as context only, no +/- line.
- `wizardCopyOnly` body (:306-309): in no hunk (hunk old-ranges 218-222,
  356-362, 389-395, 423-433, 572-578).
- Zero new `fetch(`: grep counts exactly 2 fetch call sites (asset :237 GET
  fetcher, :466 sanctioned apply POST); bound lock at
  `extraction_wizard_render.rs:355` UNEDITED (file is +0 deletions).

## 4. Locks and the amended test

- Render file +57/-0 machine-proves: the six surviving asserts of
  `test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path` and
  every pre-existing lock are byte-identical; only the appended const
  (:333-336), the two amendment clauses (:434-441), and the new lock test
  (:904-947) are new. File 890 -> 947 lines (< 1000; handoff estimated ~935,
  margin still held - recorded as a watch, no action).
- Amendment clauses byte-match the handoff-mandated asserts, including the
  amendment-4 message. The W4 lock is KEYED per kickoff sec. 7 item 4 /
  shape amendment 4: `execute.contains("wizardCopyOnly(")` +
  `execute.contains("MOVE_RETENTION_SENTENCE")`.
- New no-rot lock `test_move_sentence_states_marked_retention_truth` ships
  the mandated set: positive `contains(MOVE_RETENTION_SENTENCE)` against the
  contiguous ONE-line asset const; `wizard_function_body` carries of
  `MOVE_RETENTION_SENTENCE` in all three renderers (Impact/DryRun/Execute);
  positive `ARIADNE-EXTRACT-OUT`; absences `strips` and
  `in place for move languages`; positive staging truth clause.
- Substring-trap safety verified: `MOVE_RETENTION_SENTENCE` contains neither
  `strips` nor any old S3/S4 claim as a prefix; the new S6 clause does not
  contain `in place for move languages`; grep proves `strips` occurs NOWHERE
  in the asset (old sites :359/:392 fully replaced) and the old S6 phrase is
  gone.
- Amendment-table item 5 NO-OP holds: with the test file a pure append and
  the whole scoped suite green after the :575 edit, no pre-existing lock
  pinned the old text (a pin would be red now; none is). Binding input 6's
  conditional closed as predicted.
- A-5 S5 locks green UNEDITED (read at :644-706; the keyed S5 body keeps
  `COPY_VS_MOVE_SENTENCE`, the three verbs in order,
  `escapeHtml(planId || '')`, token/`approvalMissing`/`applyDisabled`).
  CW-3 locks, ASCII locks, theme contract, fetch bound: green UNEDITED in
  this reviewer's fresh 372-run.
- Red honesty: journal gate-2 names EXACTLY the two mandated failures with
  verbatim panic lines; the panic site 434:5 for the `wizardCopyOnly(`
  clause matches the authored file layout (assert sits at :434);
  370+2 failed = 372 = final green count. Consistent and genuine.

## 5. Copy-truth adjudication (user decision of record = the standard)

The copy states shipped behavior, machine-wise:
- Grammar: `crates/lang-cpp/src/move_retention.rs` emits
  `// ARIADNE-EXTRACT-OUT BEGIN plan=<id> symbols=<names>` /
  `// ARIADNE-EXTRACT-OUT END plan=<id>` (:31-32, pinned :376-377) -
  "markers that name the plan" is TRUE. Plan id is bound at the strip call
  (SM-3 `21cf779`; ledger line 81: m20 rider pins
  `plan=PLAN-001 symbols=Thing` observably at the strip call site).
- Count meaning: SM-4 fold mirror gives `move_removals` == names bound into
  markers (SM-I3; fixture parity measured 1 == 1, ledger line 83) -
  S4's "retires N source declaration(s)" says exactly that.
- Restore clause: uncomment + revert consumer include rewrites, or git,
  "never a one-click undo" - matches kickoff sec. 1 layer 3 (post-commit
  FULL reversibility explicitly NOT promised). R6 restore-procedure-in-the-
  sentence shipped.
- No overclaims: `one-click` appears only in the two negated forms
  (asset :223-224); `strips` zero occurrences; `instant`, `automatic
  restore`, `compiles clean` absent from the asset.
- Amendment-4 truth verified against history, not narrative:
  `git show "fd96b09:crates/ui/assets/extraction-wizard.js"` shows old S5
  rendering `${escapeHtml(COPY_VS_MOVE_SENTENCE)}` UNCONDITIONALLY (no
  `wizardCopyOnly` call in `renderWizardStepExecute`), and the fd96b09 test
  (lines 406-429) had no execute-side keying assert - it only pinned
  presence. The copy sentence is false on the move branch (apply comments
  declarations out; "nothing is moved out of the source files" is the old
  pre-move truth). The defect was real, the Red was real, the keying fix
  matches S4's mandated structure exactly.
- CW-3 consumer-rewrite truth untouched: truncation note :386 byte-identical
  (outside every hunk) + its no-rot lock green; `COPY_VS_MOVE_SENTENCE`
  survives (true where now shown: `move_removals == 0` hook-refusal
  fallback + copy branch); hotspots.js claim-of-record block untouched
  (not in the diff).
- STAYS list (handoff item 6) matches the journal's STAYS section; no STAYS
  line was "fixed" (all outside the five hunks; deletions = 4, all accounted
  as the four CHANGE sites).

## 6. Fresh workspace gates (this reviewer's own runs; milestone-final census)

Canonical lines verbatim; capture paths as printed:

- Scoped: `python tools/testrun.py --distill -p ariadne-ui` -> exit 0,
  `372 passed / 0 failed / 0 ignored / 3 suites` (capture
  `target/testrun-capture.txt`, 409 lines).
- `--baseline` probe: REFUSED verbatim
  `usage error: missing value for --baseline` (FILE form superseded, M23.5 F9;
  `--distill` census substitute governs).
- Full census: `python tools/testrun.py --distill` -> exit 0,
  `1853 passed / 0 failed / 13 ignored / 119 suites` (capture
  `target/testrun-capture.txt`, 3213 lines) - EXACTLY the milestone
  expectation; reconciles 1837 (anchor `b201aa6`) + SM-1 2 + SM-2 10 + SM-3 0
  + SM-4 3 + SM-5 1 = 1853, zero foreign residue, suites steady 119.
- Tools suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 296 tests in 28.046s` / `OK`.
- `python tools/distill.py -- python tools/ui_smoke.py` -> exit 0,
  `boot+route checks clean (cold fixture, attempt 1, 0.7s)` /
  `ui_smoke: PASS`.
- `python tools/distill.py -- python tools/bench_compare.py` -> exit 0,
  `bench compare: 3 run(s), no regressions past 1.25x baseline` - the
  implementer's recorded WARN did NOT reproduce (finding 3).
- `python tools/distill.py -- python tools/hygiene_check.py` ->
  `### generic PASS (exit 0)`; R1/R2 clean; listed R3 warns all pre-existing,
  none in this hand's files (direct run agrees).
- `python tools/distill.py -- python tools/encoding_guard.py --all` ->
  `### generic PASS (exit 0)`, capture 0 lines (silent).
- `python tools/distill.py -- python tools/theme_token_check.py
  --check-added --base e717ba0` -> `### generic PASS (exit 0)`, silent
  (zero added hex).

Gate-availability record (ANTI-PHANTOM transparency): refused probes were
`git status --short`, `git rev-parse HEAD; git status ... | Select-Object`
(compound), and `python -c ...` (both one-line and multi-line) - all with the
boilerplate catch-all `{"permission":"bash","pattern":"*","action":"deny",
"source":"global"}`; these are allow-list gaps (ledger M5 known), not gate
skips. Worked around per the sanctioned routes: porcelain from
`checkpoint_guard status`, reflog via the Read tool, exit codes via
`distill.py`. Every gate listed above ran and is quoted from its own output;
none was substituted with weaker evidence silently.

## 7. Deviation dispositions (journal-listed, adjudicated independently)

1. "None on content" (byte-exact copy) - ACCEPTED. Verified: the three
   mandated sentence sites byte-match the handoff; runtime locks prove
   asset == test const byte-equality; word-diff proves the :575 one-clause
   reword.
2. Duty-1 encoding form (`--staged` ran pre-staging with an empty index;
   `--all` superset additionally run after each commit, both silent exit 0) -
   ACCEPTED: the SM-4 ledger form was run verbatim (journal says so and the
   duty-1 commit is exactly the 3 SM-4 carrier paths); the `--all` superset
   post-landing plus this reviewer's fresh `--all` exit 0 close the evidence
   gap the pre-staging form would leave.
3. `bench_compare` exit 1 = the WARN form itself, recorded per the gate-7
   mandate rather than treated as a gate failure - ACCEPTED: the handoff
   prescribes "WARNs recorded, NOT attributed"; the tool's exit-1-on-WARN is
   its signal, and the fresh reviewer run (exit 0, no regressions)
   corroborates the PR-2 machine-class attribution (finding 3).

## 8. Findings (numbered; none blocks APPROVED)

1. MINOR, closeout-routed - gate-budget bookkeeping. Declared budget was
   "8 scoped invocations + commit cycle"; journal's own itemization lands
   9 invocations (+1 hygiene exit-code witness via distill). Inside the
   2026-10-06 law (~10 + commit-cycle trio), so "No overshoot; no D-row
   owed" is true against the law - but SM-2's 10-vs-8 precedent received a
   dated ledger D-row. Closeout should record the SM-5 budget position
   (9 vs declared 8, in-law, itemized) alongside the SM-3 16-vs-9 and SM-4
   15-vs-8 D-rows for consistency. No implementer action.
2. INFORMATIONAL, orchestration-side (not the hand) - dispatch-brief
   phrasing. The dispatch described amendment 4 as "S5 previously keyed off
   wizardCopyOnly wrongly"; git proves the pre-hand S5 never keyed off
   anything (unconditional copy sentence, no `wizardCopyOnly` call; the fd96b09
   test had zero execute-side keying asserts). The handoff and kickoff sec. 7
   item 4 ("unconditional -> keyed") are accurate. Correct the phrasing in
   the closeout ledger row; the implementer's Red and fix are genuine either
   way.
3. INFORMATIONAL, bench stance - the implementer's fresh gate-7 WARN
   (`WARN get_or_build_include_graph clone (warm): 1.3 ms vs baseline
   1.0 ms (+30%)`, exit 1, producer zero-diff proof attached) did NOT
   reproduce in this reviewer's fresh run (exit 0, warm clone median 1.0 ms,
   `no regressions past 1.25x baseline`). Non-reproduction + producer
   zero-diff attributes it to the PR-2 per-machine baseline class (SM-4
   precedent). This run closes the milestone bench stance: all five hands
   recorded fresh; zero attributable regressions milestone-wide.

## 9. Report-evidence check (skill step 8)

Journal is digest-shaped throughout: canonical `testrun` lines, two verbatim
panic lines, `ui_smoke: PASS`, `### generic PASS (exit 0)` witness, exit
codes, capture paths (`target/testrun-capture.txt` 422 lines,
`target/distill-capture.txt`), zero raw dumps; high-volume gates ride the
distilling entry points. PASS.

## 10. Follow-up dispositions (for the CLOSEOUT commit; verdict +
execution-results here ride it uncommitted)

- Ledger closeout consumes section 11 closeout facts; flip the SM-5 era
  state; discharge/relay per kickoff sec. 6.
- New dated ceiling watch row: `extraction_wizard_render.rs` 947/1000.
- Ledger row 470 fetch-site-bound invariant line: append the SM-5
  "preserved, not discharged" note (bound `<= 2` ran green UNEDITED at this
  touch; trigger stands for every future touch).
- Deferred rows unchanged and re-registered by closeout (see section 11).
- No out-of-scope observation to carry: the journal's report-only section
  names none beyond the bench WARN (dispositioned as finding 3).

## 11. Milestone closeout facts (closeout planner consumes this section)

Per-hand chain (reflog lines 437-449 above; SHAs short = full in the blob
lines):

| Hand | Landing | Rider/carrier | Reflog | Census after |
|---|---|---|---|---|
| planning | kickoff+shape `2177a85`, plan `e5c49ec` | - | 437, 438 | anchor 1837 @ `b201aa6` (436) |
| SM-1 (redo) | `b993600` (439) | re-gate carriers `0db2680` (440) | 439/440 | 1839 (+2) |
| SM-2 | `f6c92fe` (441) | journal rider `fba3c90` (442); verdict carriers `790fe75` (443) | 441-443 | 1849 (+10) |
| SM-3 | `21cf779` (444) | verdict carriers + amendments `a33f620` (445) | 444/445 | +0, census EXEMPT (hard chain; one designed red at `tests.rs:300`) |
| SM-4 | `e717ba0` (446) | verdict carriers `fd96b09` (447) | 446/447 | 1852 (+3; discharges SM-3+SM-4, hard chain CLOSED, R5 window CLOSED) |
| SM-5 | `82cf789` (448) | journal rider `522a770` = HEAD (449) | 448/449 | 1853 (+1) = MILESTONE EXPECTATION - fresh exit 0 |

Census progression: `1837 -> 1839 -> 1849 -> (SM-3 +0, exempt, red-exact) ->
1852 -> 1853`, every delta attributed, `--distill` substitute governs,
`--baseline` FILE form refused verbatim at every hop (this hop: section 6).

Incidents: PHANTOM-NARRATION-1 CLOSED (fabricated `4e5b0b7`/`31e4e86` voided;
closure via `0db2680`; reflog-cross-check countermeasure law in force and
exercised at every hand since, including this verdict - section 1). SM-3
STOP -> AMENDMENT-1 -> landing, hard chain SM-3 -> SM-4 authorized,
discharged by SM-4's 1852 census (ledger lines 78-83; re-verified here by
adjacency of reflog 444/446 with only the sanctioned carrier 445 between -
no release window opened).

Gate-budget D-rows traveling to closeout: SM-2 10-vs-8 (D-2026-10-24 SM-2),
SM-3 16-vs-9, SM-4 15-vs-8 (both the cargo duplicate-`--test` quirk), SM-5
9-vs-declared-8 IN-LAW (finding 1 - record for consistency, no D-row owed
under the law).

Deferred rows status at closeout (all still open, re-register per kickoff
sec. 6): optimize-verb move parity (tripwire = m19 deferral lock at
`m19_optimize_characterization_test.rs:164` going red); include-closure /
compilability (deferred row 446 item ii, D-M23-12 unchanged - no tier
compiles); directive-spelling display in dry-run rows (row 446 item ii);
marked-region purge command (kickoff sec. 6); generation-swap half of row
445 RECORD-NOT-EXECUTE (SM-4 `extraction_apply.rs` diff-EMPTY honored, SM-5
cli zero-diff re-confirms; closeout appends the dated relay row); ceiling
watches `extraction_reader.rs` 926/1000 and `extraction_apply/tests.rs`
914/918 held (SM-5 touched neither), NEW watch added by this hand:
`extraction_wizard_render.rs` 947/1000.

Bench stance closed all-hands: every hand recorded fresh through the
distiller; the only WARNs (SM-4, SM-5 - both the same `get_or_build_*` warm
~1 ms class) did not reproduce in reviewer runs; producer zero-diff proofs
in every journal; attribution = PR-2 per-machine baseline class, nothing
attributed to any hand.

Decision-of-record shipped for the apply verb: move-by-default via
marked comment-out under plan-id-bound `ARIADNE-EXTRACT-OUT` markers,
dry-run parity, three-tier apply, honest wizard copy (this hand), Scala
zero-diff by delegation, never a one-click claim. Optimize verb: deferral
lock shipped (AMENDMENT-1), parity is a registered follow-on with its
machine tripwire.
