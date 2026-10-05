# cpp-brace-depth-underflow T1 - reviewer verdict (final, round 2)

Verdict: **APPROVED**.

Round 1 verdict (REJECTED, findings 1-7) is preserved below in condensed
form; round 2 adjudicates the remediation commit `af11113`
("require closing quote in raw-string terminator (review Finding 1)").

## Round 2 - remediation adjudication

Scope: `44dfb73..af11113` contains four commits; three (`f2e1790`, `4a65b6f`,
`df1e21e`) belong to the separate `ui-extraction-card-playbook` milestone
(own verdict track on the board) and touch no `crates/lang-cpp/**` path.
This handoff's enforcement audited `af11113` alone.

- Commit-vs-touch-set: `af11113` stat = `crates/lang-cpp/src/tokenizer.rs`
  (+1 line), `crates/lang-cpp/src/tokenizer/raw_string_tests.rs` (+13),
  `tasks/cpp-brace-depth-underflow-handoff-journal.md` (+57) - all inside
  the declared touch-set; characterization/symbol_locator/golden/fixtures/
  Cargo paths absent. PASS.
- The +1 is exactly the ordered verdict instruction:
  `&& chars[closing_quote] == '"'` added to the match arm
  (`tokenizer.rs:278`). Test added:
  `closing_paren_inside_raw_content_requires_delim_quote_terminator` with
  the reviewer fixture verbatim + the delimited `)done` variant. Plan-literal.
- Journal append: honest RGB record; Red line `147 passed / 1 failed /
  0 ignored / 1 suites` (intermediate state, not replayable post-tree;
  consistent with the reviewer's pre-guard mirror analysis predicting the
  `punct:}` leak), Green `148 passed / 0 failed / 0 ignored / 2 suites`
  independently reproduced this session.

### G4 re-verification (reviewer's own updated scan, authoritative)

Reviewer updated its own script (permitted; it is the reviewer's tool) with
a committed-mode mirror synced to `af11113`, written as an independent code
branch from the faithful mode (different condition order, same language).
Full output in execution-results Round-2 G4 section:

- `mode=committed files_below_zero=14 files_nonzero_final=71` ==
  `mode=faithful` (both), first-underflow pin stable at
  `AggregateFunctionNull.cpp:174`.
- `FINDING2 committed_vs_faithful digest_diff_count=0 state_diff_count=0` -
  real cross-branch convergence.
- `newly_negative_post_committed_vs_pre=0` (was 13);
  `fixed_by_fix(pre_neg->post_ok)=4`; `still_negative_both=14` - exactly the
  Finding 3 residual class absorbed by the parser floor.
- Bug probe `R"(f(x) } g)" h(); {}`: committed now min=0/fin=0 (was -1).
- Balanced-set digest diffs vs PRE: **17** (of 9125) - ALL 17 carry
  `raw_headers>=1` (1 to 1375; includes the spec-named `Server\ACME\API.cpp`
  with 10), ALL 17 have identical `(0, 0, None)` pre/post state,
  `bal_diff_state_changed=0`, `bal_diff_without_raw_header=0`.

### Deviations adjudicated (round 2)

- **D6 throwaway-copy route** (`target/g4_post_fix_scan.py`): **accepted**.
  The reviewer's script was left untouched and its verdict numbers were
  reproduced verbatim first; `target/` is the handoff-sanctioned throwaway
  location; the copy was verified condition-for-condition against the
  shipped source - no weakening. Caveat (recorded, no action): the copy
  folds committed and faithful into one guarded branch (its `mode` arg is
  unused in `raw_scan`), so its `FINDING2` block is a structural tautology;
  the real convergence evidence is the reviewer's independent two-branch
  scan above, which agrees.
- **D7 the 17 balanced-set digest diffs**: **accepted as intentional
  layer-1 change**. Empirical attribution (above) shows zero non-raw TU
  diffs and zero state changes; these are hazard TUs whose pre-fix quote
  flips happened to balance, and the fix removing their phantom brace
  emissions is precisely what spec 3.2 mandates. The literal "balanced-set
  digest diff empty" (spec 3.5 G4 / I3 / 4.3) cannot hold simultaneously
  with layer-1's purpose and must be reworded - folded into Finding 3.

### Round 2 gates (verbatim)

- G1: `148 passed / 0 failed / 0 ignored / 2 suites` exit 0.
- G2: `23 passed / 0 failed / 0 ignored / 1 suites` exit 0.
- G3: `2 passed / 0 failed / 0 ignored / 1 suites` exit 0.
- m17: `6 passed / 0 failed / 0 ignored / 1 suites` exit 0.
- hygiene: exit 0, R3 warn-only, none introduced by `af11113`.
- `encoding_guard.py --all`: exit 0.
- tools: `Ran 188 tests in 19.525s` / `OK`.
- census (`--distill`, substitute for the unrunnable `--baseline` literal):
  `1204 passed / 0 failed / 4 ignored / 85 suites` exit 0 (+9 vs round 1:
  +1 lang-cpp, +8 from the intervening playbook T-A).
- ui_smoke: `ui_smoke: PASS` exit 0.
- bench_compare: 5-run FAIL `plan() 1847.2 ms (+27%)`, then 5-run PASS
  `1793.3 ms` on the SAME tree - `plan()` (closure-side, not lang-cpp;
  input bytes proven unchanged by m00 goldens) sits at the 1.25x threshold
  edge and flaps run-to-run. The milestone hot-path phase `symbol_pass+prep`
  was within tolerance on every run (1489.6 = 1.17x). See Finding 7.

### Closeout queue (orchestrator, at milestone close)

1. Finding 3 (expanded): reword spec 3.5 G4(b), I3, and 4.3 "balanced-set
   digest diff empty" to the empirically checkable form actually achieved:
   committed==faithful zero diffs, zero newly-negative, zero non-raw balanced
   diffs, residual 14 TUs absorbed by the layer-2 floor.
2. Finding 4: ledger parity numbers 22/22 -> 23/23 (`23 passed / 0 failed /
   0 ignored / 1 suites`) and m17 5/5 -> 6/6.
3. Finding 5: census substitute line `1204 passed / 0 failed / 4 ignored /
   85 suites`; decide the open ledger row 56 (commit a baseline census file
   or amend AGENTS/kickoff wording).
4. Finding 7 (promoted): `bench_baseline.json` refresh on a quiet machine -
   `plan()` median now straddles 1.25x (1.236x-1.273x across today's four
   runs) and `ensure_load total` runs 17.1-17.4k against a 14413 baseline;
   without a refresh every future milestone inherits a flapping gate.
5. Operator purge of reviewer throwaways (untracked, never committed):
   `.kilo/plans/rv-g4-corpus-scan.py`, `.kilo/plans/rv_g4_corpus_scan.py`,
   `.kilo/plans/rv_g4_residual_class.py`; implementer's
   `target/g4_post_fix_scan.py` purges with the target dir.

## Round 1 verdict (superseded, condensed)

REJECTED on Finding 1: `scan_raw_string` terminator match omitted
`chars[closing_quote] == '"'`; empty-delimiter slice equality trivially
true; first content `)` terminated the literal early. G4 then showed 28
below-zero TUs vs 18 pre-fix, 13 newly negative, 824/9125 balanced digest
diffs. Findings 3-7 as renumbered above; declared deviations D1-D5 all
accepted (journal path naming, `R"DOC"` fixture, `scan_quote` extraction,
Red-reason variance, 11-vs-10 gate tally). Step-0 spec copy byte-identical
(empty `git diff --no-index`), commit isolated, locks intact throughout
both rounds.
