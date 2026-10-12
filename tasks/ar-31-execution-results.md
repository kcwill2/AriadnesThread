# AR-31 Execution Results - Reviewer acceptance pass

Reviewer session `ses_ed45b79b2ffe16uuUCvzAfkqXJ`, 2026-10-11. All lines are
reviewer-own stdout captured this session (capture-race ruling honored: never
quoted `target/*-capture.txt` written by another run).

## Ground truth (`python tools/checkpoint_guard.py status`)

- HEAD `c8ae8ea`, branch `shining-enthusiasm`, tracked tree CLEAN (zero dirty
  files). Worktree census 10 entries, no anomaly relevant to this handoff.
- Commit order in range `b7136db..HEAD` (`git log --oneline`):
  `e946f57` (AR-26, foreign) -> `a634ae5` (AR-26, foreign) -> `2679ebb`
  (AR-26, foreign) -> `089a9d1` (AR-31 characterization) -> `1140ed5` (AR-31
  rework) -> `c8ae8ea` (AR-31 journal). Foreign rows: `tasks/
  ar-26-handoff-journal.md` only (verified by `git show --stat`).

## Acceptance criteria vs evidence

### Contract: characterization-first ordering (the core lock)

| Check | Result |
|---|---|
| `089a9d1` tokenizer.rs delta | `git show "089a9d1" -- crates/lang-cpp/src/tokenizer.rs`: exactly 3 added lines (`#[cfg(test)]` + `mod fixture_parity_tests;` + blank). Zero logic. |
| Suite blob identity | `git rev-parse "<rev>:crates/lang-cpp/src/tokenizer/fixture_parity_tests.rs"` -> `36bf590790cbd2d47bcbb2ce4febeabd97c9b19e` at `089a9d1` = `1140ed5` = `c8ae8ea`. Byte-identical UNEDITED through the refactor. |
| Oracle + sibling-module freeze | `git diff 089a9d1 1140ed5 -- crates/lang-cpp/tests/tokenizer_oracles crates/lang-cpp/src/tokenizer` -> EMPTY. |
| Oracle semantic correctness (independent) | Hand-computed `word_run.cpp` line 10 (`cafe..._cafe cafeine = cafe_cup;` with U+00E9 runs): oracle columns 1/11/19/21/29 are CHAR columns (byte-column would yield 13 for token 2); char spans 467-476, 477-484, 485-486, 487-495, 495-496 with correct 1-gap spacing. Values are OLD-tokenizer semantics - not fittable to a byte-counting rework. |
| 19 oracles present | glob `crates/lang-cpp/tests/tokenizer_oracles/**/*.txt` -> 19 files. |
| Committed module is walk+match only | Read module (162 lines): fixture walk (sorted, deterministic), render, one `#[test]` match; `tus.len() >= 15` non-vacuity guard; no generator, no allocator residue. |

### Contract: M27 locks UNEDITED + non-vacuous

| Check | Result |
|---|---|
| Zero-diff both paths + fixture | `git diff b7136db..HEAD -- crates/lang-cpp/src/tokenizer/underflow_fix_tests.rs tests/integration/tests/m27_nonascii_corpus_test.rs tests/fixtures/m27-nonascii-underflow` -> EMPTY. |
| underflow_fix_tests.rs content | Byte-escaped specimens (`\xc3\xa9`, `\xc2\xab`, `\xe2\x82\xac`, `\xe4\xb8\xad`), falsifiable bound `column in [1, line_chars+1]` (release-wrap detector) + span identity `end-start == chars().count()` + three byte-exact ASCII baseline streams. Genuine byte-vs-char coverage. |
| m27_nonascii_corpus_test.rs content | Single-encoding byte locks (flanked-sequence byte scans) + cold full-pipeline index over isolated specimen copy + recursive shard-JSON wrapped-column detector bounded by per-line char counts. Genuine. |
| Fresh runs | underflow locks inside `-p ariadne-lang-cpp` (156 green); corpus lock `2 passed / 0 failed / 0 ignored / 1 suites` (below). |

### Contract: token-stream byte-parity (the load-bearing lock) - fresh

- `python tools/testrun.py --distill -- -p ariadne-lang-cpp`
  -> `156 passed / 0 failed / 0 ignored / 2 suites`
  (= implementer's post-change line verbatim; 155 UNEDITED baseline + 1 new
  match lock; D2 re-brief reconciliation consistent.)
- Match test exercised inside that run; its fixture coverage includes the
  m27 non-ASCII specimens (byte-vs-char column axis + RawToken start/end
  char spans consumed by the M23 verbatim locator).

### Contract: dated M27 + parity/m00/m17 gates UNEDITED - fresh

- `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m27_nonascii_corpus_test`
  -> `2 passed / 0 failed / 0 ignored / 1 suites`
- `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m18_cpp_plugin_parity_test`
  -> `23 passed / 0 failed / 0 ignored / 1 suites`
- `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test`
  -> `4 passed / 0 failed / 0 ignored / 2 suites`
- `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m17_plugin_seam_test`
  -> `8 passed / 0 failed / 0 ignored / 1 suites`

All five scoped lines reproduce the journal's numbers byte-for-byte.

### Contract: strict compile-green

- `python tools/distill.py -- cargo check --workspace`
  -> `ok (exit 0, 0 warnings emitted)` (my stdout; capture file shared with
  the live planner session - not quoted).

### Contract: hygiene (zero-blocking) + encoding

- `python tools/hygiene_check.py`: no R1/R2 rows at all (R3 warn-only list
  is all pre-existing files). `tokenizer.rs` and
  `fixture_parity_tests.rs` appear in NO row - the two R3 rows the OLD
  tokenizer carried (`i`, `c`) are gone and the rework/suite added none.
  Claim "R3 delta is tokenizer.rs only" holds: no other touched file
  contributes rows.
- `python tools/encoding_guard.py --all` -> silent (exit clean).

### Contract: allocation-probe discipline (temporary, deleted, report-only)

- Repo-wide grep `*.rs` for `GlobalAlloc|ar31` -> no matches anywhere
  (crates + tools + tests). No probe code in tree.
- Report exists ONLY where declared: `target/ar31-alloc-probe.txt`, 20
  lines, `TOTAL alloc_ops=458`; all 19 per-TU values equal the journal's
  post-change numbers exactly (OLD 624 -> NEW 458, -26.6%). D5 mechanics
  (ignored single test, `--exact --ignored --test-threads=1`, unique report
  path) are sound for a deterministic alloc-op count.

### Contract: bench neutral-or-better - NOT re-run (dispatch-restricted)

- Journal row (implementer stdout): `bench compare: 3 run(s), no
  regressions past 1.25x baseline` (exit 0), captured while a live AR-25
  reviewer lane ran scoped gates concurrently.
- Adjudication: contention makes the >1.25x trigger MORE sensitive, so the
  no-regression claim cannot mask a true timing regression; the quantitative
  margin, however, is not established from a contended run. Per dispatch the
  closeout owns the quiet-tree row; verdict finding 3 records the margin
  confirmation as riding that row. No reviewer bench run demanded.

### Acceptance pass - workspace tools

- `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests in 20.377s` / `OK`

## Refused bash forms (global catch-all deny, verbatim) and substitutes

Each refusal printed: `The user has specified a rule which prevents you from
using this specific tool call. Here are some of the relevant rules
{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.

1. `git rev-parse "<rev>:...fixture_parity_tests.rs" ... | Measure-Object -Line`
   (chained pipe) -> split into separate `git rev-parse` calls (both succeeded).
2. `git ls-tree -r --name-only "089a9d1" crates/lang-cpp/tests/tokenizer_oracles/`
   -> glob tool over the working tree (19 files; byte-identity independently
   proven by the empty `git diff 089a9d1 1140ed5 -- ...tokenizer_oracles`).
3. `Test-Path -LiteralPath "target\ar31-alloc-probe.txt"` -> Read tool
   (file exists, full content shown).
4. `git status --porcelain crates/lang-cpp/src` -> `checkpoint_guard.py
   status` reports `(none)` dirty + untracked census (no crates/ untracked).

No expected reviewer check was skipped: every refusal had an evidence-complete
substitute through permitted tools.

## Not run (explicit statement)

- `python tools/testrun.py --baseline`: NOT run - dispatch errata (file-mode
  tool, absent baseline file, bare invocation exits 2). Workspace census gate
  belongs to closeout per Planning Law sizing.
- `python tools/ui_smoke.py`: NOT run - dispatch: "ui_smoke/bench not
  required - closeout owns the quiet-tree row".
- `python tools/bench_compare.py`: NOT run - same dispatch restriction; see
  bench adjudication above (margin confirmation assigned to closeout).
- Release-profile `cargo test --release -p ariadne-lang-cpp` (prescribed by
  the underflow module's own doc, not by the contract gate list): NOT run;
  recorded as verdict finding 1 (parity oracles close the coverage hole in
  practice - wrapped columns cannot match the frozen char oracles).
