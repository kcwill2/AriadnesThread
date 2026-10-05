# T0b verdict - M27 tu-index-hang-throughput: APPROVED (fixture approval granted)

Reviewer hop 2026-10-05 via `review-gate`. Handoff
`tasks/tu-index-hang-throughput-t0b-handoff.md`; commits `d6267a3`
(fixture+test+journal, 6 paths, 599 insertions) and `4ef8412` (journal errata,
journal-only); parent/journal start rev `62ce72b`. Reviewer HEAD: `4ef8412`
(verified via `checkpoint_guard.py status`: main, dirty only foreign
orchestrator plan/task docs, none in the touch-set).

## VERDICT: APPROVED

## Frozen-set approval (this hop's purpose)

APPROVAL GRANTED for `tests/fixtures/m27-nonascii-underflow/` landing via the
documented frozen-set approval path (new sibling dir only; reviewer-approved
commits `d6267a3` + `4ef8412`). Proven at this hop:
`git diff --name-only 62ce72b..HEAD -- tests/fixtures` lists ONLY the four
`m27-nonascii-underflow` files; `git diff --stat 62ce72b..HEAD --
tests/fixtures/basic tests/fixtures/expensive` -> EMPTY (byte-identical); no
new files inside either byte-locked dir; no other fixture, golden, oracle,
`crates/`, `tools/`, `tests/integration/Cargo.toml`, or
`tests/integration/tests/common/**` path appears anywhere in the range diff.

**Orchestrator duty (lands WITH this verdict):** the dated ledger note VERBATIM
in the M27 row's Notes:

`2026-10-05 fixture approval (M27 T0b): tests/fixtures/m27-nonascii-underflow/
added via the documented frozen-set approval path (reviewer-approved commit
+ this dated note); single-encoded UTF-8 word-run AND punct-run specimens;
nothing inside basic/expensive`

## Gates re-run by the reviewer (verbatim outputs)

1. RED re-capture `python tools/distill.py -- cargo test -p
   ariadne-integration-tests --test m27_nonascii_corpus_test -- --ignored
   --nocapture` -> digest `### cargo build FAIL (exit 101)`; capture
   `target/distill-capture.txt` (18 lines): `test result: FAILED. 0 passed; 1
   failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.02s` and
   `thread 'test_m27_nonascii_corpus_cold_index_completes' (17440) panicked at
   crates\lang-cpp\src\tokenizer.rs:331:21: attempt to subtract with overflow`
   - the EXACT E3 site per `tasks/tu-index-hang-throughput-t0-evidence.md:32`
   (dispatch brief's ":331:11" is a dispatcher typo; see finding 3).
2. GREEN `python tools/testrun.py --distill -p ariadne-integration-tests
   --test m27_nonascii_corpus_test` ->
   `1 passed / 0 failed / 1 ignored / 1 suites`, exit 0 (`### cargo test PASS
   (exit 0)`). Matches acceptance 2 exactly.
3. `python tools/hygiene_check.py` -> `HYGIENE_EXIT_0`; all emitted lines
   R3 warn-only, zero hits in any touched file.
4. `python tools/encoding_guard.py --all` -> `ENC_EXIT_0` (no output).
5. Commit-vs-touch-set: `git show --stat "d6267a3"` and `git show --stat
   "4ef8412"` -> only the six touch-set paths (d6267a3) and the journal
   (4ef8412). Range diff `--name-only 62ce72b..HEAD` = 6 paths, all in-set.
6. Workspace census substitute `python tools/testrun.py --distill` (full
   workspace) -> `1214 passed / 0 failed / 5 ignored / 86 suites`, exit 0.
   Reconciliation against the last ledger-recorded census
   `1213 passed / 0 failed / 4 ignored / 85 suites` @ `816d09d`: EXACTLY
   +1 passed (new byte lock) / +1 ignored (carried RED lock: 4 -> 5 per the
   reconciliation row) / +1 suite (the new test binary itself). Existing suite
   results UNEDITED (zero failures at 0 failed).
   Literal `python tools/testrun.py --baseline` -> `usage error: missing value
   for --baseline` - known unrunnable FILE-value flag, ledger line 59 errata
   (recurrence logged).
7. `python -m unittest discover -s tools -p "*_tests.py"` ->
   `Ran 188 tests in 20.477s` / `OK`.
8. `python tools/distill.py -- python tools/ui_smoke.py` -> `### generic PASS
   (exit 0)`, `boot+route checks clean (cold fixture, attempt 1, 1.0s)`,
   `ui_smoke: PASS`.
9. `python tools/distill.py -- python tools/bench_compare.py` -> `### generic
   PASS (exit 0)`, `bench compare: 3 run(s), no regressions past 1.25x
   baseline`.

## Fixture-bytes verification (hex-level, substituted routes)

- Working tree clean for all touched paths (checkpoint census), so blobs ==
  files; byte-level proof routes:
  - Green byte-lock test itself (run fresh at gate 2) asserts: both .cpp
    valid UTF-8; `C3 A9` flanked by ASCII-alphanumeric on BOTH sides in
    `word_run.cpp`; `C2 AB` flanked by non-alphanumerics in `punct_run.cpp`;
    `E2 82 AC` present; no U+00C3 signature; README + test file contain zero
    bytes >= 0x80. All pass.
  - Reviewer codepoint grep `[\x{80}-\x{10FFFF}]`: NO match in the test file,
    README, journal, `include/runs.h` (pure ASCII confirmed independently).
  - `^[\x{20AC}]` matches `punct_run.cpp:10` (`€uro seam = 1 « 2;`) - the
    journal's declared driver position, confirmed.
  - Mojibake characters `Ã`/`Â` and the `c3 82`/`c3 83`/`c3 a2 e2 82 ac`
    signatures: absent from the whole fixture dir (grep + `encoding_guard
    --all` exit 0).
- `word_run.cpp:10` = `café_café caféine = café_cup;` - the handoff's named
  identifier `café_cup` retained, a second multi-byte word (`caféine`) on the
  same line, C3 A9 between ASCII letters - all mandated shapes present.
- Static `:311` arithmetic confirmed against live `tokenizer.rs` (unmodified
  by this hop): `flush_word` at :311 computes `column - word.len()`;
  `café_café` = 9 chars / 11 bytes at column 1 -> 10 - 11 underflows. The
  captured panic is `:331:21` because the punct TU is analyzed first; both
  sites are named in the ignore attribute and both TUs are covered by the
  completion lock once T2 un-ignores it.

## Plan-literal checks

- Ignore attribute at `m27_nonascii_corpus_test.rs:143` matches the handoff's
  mandated EXACT string verbatim (single line; names T2 as the un-ignorer;
  names `tokenizer.rs:311/:331`; I12 cited).
- Fixture layout `src/`+`include/`, no compile db in the committed fixture
  (synthetic db written into the TEMP COPY only); dir + all four file names
  exact; README states E3 shape, single-encoded ON PURPOSE, never-extend,
  sibling-dir rule.
- Fixture .cpp files 10 lines each (< 40); all test functions < 100 lines;
  test file 315 lines.

## Deviation dispositions (each judged independently)

- D1 CONTINGENCY (crux) - ACCEPTED, and this verdict constitutes the
  re-approval of the swapped bytes (`C3 A9` x4 in `word_run.cpp`; `E2 82 AC`
  line-leading + retained `C2 AB` in `punct_run.cpp`). The declared mechanism
  was honored, not invented content: the implementer authored v1 literally
  first, ran the Red capture, and EMPIRICALLY demonstrated the v1 fixtures
  COMPLETED (static proof the reviewer independently confirms: one 2-byte
  char per word at column 1 floors at 0 - no underflow; a 2-byte punct char
  can never underflow). That exposed a plan self-contradiction: the
  handoff's literal specimen CANNOT satisfy its own acceptance 1 (a panic
  capture on the minimized corpus). The contingency clause was conditioned on
  a parser-Err mode; the observed mode (silent pass) is milder with the same
  remedy. The punct swap is within the clause's letter; the `word_run.cpp`
  second-`C3 A9` driver EXCEEDS its letter (the clause names only the
  punct-run character) - adjudicated on merits as an authorized
  plan-defect resolution: declared in journal AND report-back before landing,
  every mandated run-shape lock preserved, every byte hex-declared at the
  hop whose sole purpose is this approval, zero oracle touched. Undeclared,
  the same change would have been an automatic blocker; the dispatch brief
  itself now cites the swapped corpus as the thing to verify.
- D2 `exit 101` vs prose "exit 1" - ACCEPTED, non-substantive (cargo suite
  failure is 101; handoff required non-zero, satisfied).
- D3 `reanalyzed_tus == 2` anti-vacuity assertion - IN-SPEC hardening:
  without it, a marker-free temp copy shapes zero C++ units and the
  completion lock would pass vacuously; strengthens a mandated assertion.
- D4 `E2 82 AC` byte-lock assertion - IN-SPEC: operationalizes the
  contingency's re-visibility clause (swapped bytes are themselves locked).
- D5 gate-5 distill-wrapped commit refused; direct `commit_scoped.py` with
  identical scopes+message landed `d6267a3`; errata recorded in journal +
  `4ef8412` - ACCEPTED (ledger line 60 config row; touch-set independently
  enforced above).
- D6 synthetic `compile_commands.json` in temp copy only - IN-SPEC necessity,
  documented env fact; committed fixture layout verified basic-style.

## Findings (numbered)

1. MINOR (config, not implementer): `git grep`/`git grep -P` refused in the
   reviewer sandbox; verbatim refusal each time: `The user has specified a
   rule which prevents you from using this specific tool call. Here are some
   of the relevant rules {"permission":"bash","pattern":"*","action":"deny",
   "source":"global"}`. The mandated `git grep -P` non-ASCII check was
   performed via permitted equivalents (codepoint grep tool + green byte
   lock + `encoding_guard --all`), all exit-clean. Recurrence of ledger
   line 60 operator-config row.
2. MINOR (config): inline `python -c` refused with the same boilerplate;
   byte inspection routed via the equivalents above (same ledger row family).
3. MINOR (dispatch-doc errata): the orchestrator's dispatch brief cites the
   E3 panic site as `tokenizer.rs:331:11`; the field evidence
   (`t0-evidence.md:32`) and the reviewer's fresh capture both say
   `:331:21`. Erratum in the dispatch wording; implementer artifacts are
   correct and unchanged.
4. MINOR (journal nit): journal Gate-1-red quotes the capture as "19 lines";
   the reviewer's re-run capture is 18 lines (PID-dependent line wrapping);
   panic line and message identical in substance. Non-substantive.

## Residual / routed observations

- The v1 no-panic first capture is journal-attested, not re-creatable (v1
  bytes never committed). Accepted: the FINAL bytes are independently verified
  above and locked green; the static arithmetic proves the v1 shapes cannot
  underflow, corroborating the attestation.
- Census 4 -> 5 ignored confirmed live at `1214/0/5/86`; carry `5 ignored /
  86 suites` as the new anchor for subsequent M27 hops. F1-class errata for
  the T0-report journal (claimed distill commit route) remains routed to the
  orchestrator per the journal's out-of-scope rider - unchanged by this hop.

## Approval conditions discharged

Ledger note quoted above lands with this verdict (planner/orchestrator, not
this reviewer - `tasks/milestone_ledger.md` was untouched by both commits,
verified in the range diff). T2 may un-ignore the completion lock as its
single-line rider after T0-report + T0b (both landed).
