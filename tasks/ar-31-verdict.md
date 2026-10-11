# AR-31 Verdict - Tokenizer allocation rework (P8 MED)

Reviewer: `ses_ed45b79b2ffe16uuUCvzAfkqXJ`, 2026-10-11.
Contract: `tasks/ar-31-handoff.md`. Journal: `tasks/ar-31-handoff-journal.md`.
Range: base `b7136db` -> `089a9d1` (characterization) -> `1140ed5` (rework) ->
`c8ae8ea` (journal). Foreign commits in range: `e946f57`, `a634ae5`, `2679ebb`
(AR-26, journal-only to `tasks/ar-26-handoff-journal.md`) - excluded from
touch-set attribution, correctly disclosed in the dispatch.

## VERDICT: APPROVED

Gate evidence: `tasks/ar-31-execution-results.md` (reviewer-own stdout).

## Central adjudications

### 1. Ordering discipline (the contract's core) - VERIFIED, not taken on report

- `git show "089a9d1" -- crates/lang-cpp/src/tokenizer.rs`: the tokenizer
  delta is exactly 3 inserted lines - `#[cfg(test)]` + `mod
  fixture_parity_tests;` + separator blank. Zero logic. The tokenizer at
  `089a9d1` is the OLD char-vec tokenizer (blob `853957d` = base blob + mod
  registration).
- Suite blob identity: `git rev-parse` of
  `crates/lang-cpp/src/tokenizer/fixture_parity_tests.rs` returns
  `36bf590790cbd2d47bcbb2ce4febeabd97c9b19e` at `089a9d1`, `1140ed5`, AND
  `c8ae8ea` - byte-identical through the refactor. The lock predates the
  swap; it cannot have been written to fit the new code at commit level.
- `git diff 089a9d1 1140ed5 -- crates/lang-cpp/tests/tokenizer_oracles
  crates/lang-cpp/src/tokenizer` is EMPTY - all 19 oracles and all four
  test modules frozen across the refactor.
- Independent content check (beyond hash identity): I hand-recomputed the
  `word_run.cpp` oracle (`W|caf\xC3\xA9_caf\xC3\xA9|10|1|467|476`,
  `W|caf\xC3\xA9ine|10|11|477|484`, `P|=|10|19`, `P|;|10|29`) against the
  fixture line 10. Every column is a CHAR column (token 2 at column 11;
  byte-counting would give 13) and every start/end is a char-span with
  consistent gaps. These are OLD-tokenizer semantics; a suite "fitted" to a
  byte-counting rework could not carry these values.

### 2. Non-vacuity of the new lock - VERIFIED on merits

- Journal gate #5 (mutation): byte-length flush subtraction injected ->
  `0 passed / 1 failed / 0 ignored / 1 suites` with
  `attempt to subtract with overflow` - the exact M27 E3 message.
- Revert complete: tokenizer.rs blob `88e2de9a60f50f3bbca1f2a0554a9494c7fd9b69`
  identical at `1140ed5` and `c8ae8ea`; `checkpoint_guard.py status` shows
  zero dirty files at HEAD. No mutation residue.
- Mechanism plausibility (read-only code reasoning): `PendingWord::take_into`
  (`tokenizer.rs:92`) computes `column: column - self.chars` - the ONLY
  char-count subtraction in the rework (`delimiter_len = content_opener -
  delimiter_start` is monotone-guarded; all other arithmetic is addition).
  Swapping `self.chars` for the run's byte length re-creates E3 exactly as
  the fixture's own comment documents it ("flush_word subtracts 11 BYTES
  from a CHAR-counted column of 10"). The mutation targeted a real, load-
  bearing site.

### 3. D1 touch-set deviation (suite module location) - ACCEPTED

- Reachability claim verified independently: `crates/lang-cpp/src/lib.rs:27`
  is `mod tokenizer;` (private), and `tokenize`/`tokenize_raw`/`RawToken`
  are `pub(crate)`. An integration test under `crates/lang-cpp/tests/`
  cannot reach them without a `lib.rs` edit (explicitly outside the touch-
  set) or widening production API. The handoff's literal wording is not
  implementable as written.
- Precedent verified: `characterization_tests.rs`, `raw_string_tests.rs`,
  `underflow_fix_tests.rs` all live in `crates/lang-cpp/src/tokenizer/` -
  exactly the directory the new module joined.
- The oracle DATA honors the literal wording (`crates/lang-cpp/tests/
  tokenizer_oracles/`, 19 files).
- Disposition: accepted. This is the handoff-wording-is-impossible pattern
  (AR-01/AR-02 "or new mod" lineage); the deviation is the minimum necessary
  and matches repo convention.

### 4. D3 size deviation - ACCEPTED

- `git show --numstat 1140ed5`: `343  221  crates/lang-cpp/src/tokenizer.rs`
  - the journal's number is exact, not approximate.
- Whole-file diff is inherent: every scan helper's signature and body changed
  from char-slice indexing to byte-cursor iteration (verified in the
  per-commit diff). Planning Law produced-lines budget (~1200) is met with
  margin (564 changed lines). Touch-set unchanged by the size.
- Disposition: accepted.

### 5. M27 lock preservation - VERIFIED (zero-diff AND non-vacuous)

- Both lock paths exist as journalled:
  `crates/lang-cpp/src/tokenizer/underflow_fix_tests.rs` (runs UNEDITED
  inside the 156-count `-p` suite) and
  `tests/integration/tests/m27_nonascii_corpus_test.rs` (2/0/0/1, re-run
  fresh).
- `git diff b7136db..HEAD -- <both paths> tests/fixtures/m27-nonascii-underflow`
  is EMPTY - file-ZERO-diff across the whole range, fixture included.
- Non-vacuity read: the underflow module carries byte-escaped multi-byte
  specimens (U+00E9/U+00AB/U+20AC/U+4E2D built from stored bytes), a
  falsifiable bound (`column in [1, line_chars+1]` in BOTH debug-panic and
  release-wrap profiles) plus a char-span identity (`end - start ==
  chars().count()`), and byte-exact ASCII baseline streams. The corpus test
  runs the whole pipeline cold over the specimen repo and runs a recursive
  wrapped-column detector over shard JSON against per-line char counts.
  These genuinely exercise the byte-vs-char axis.

## Per-commit review (oldest first)

- `089a9d1`: mod-registration + suite + 19 oracles + journal. No production
  logic. GREEN-vs-OLD ordering structurally proven (adjudication 1).
- `1140ed5`: tokenizer.rs only (+journal). Full diff reviewed alone.
  Equivalence holds: `take_into` reproduces `flush_word` (`column -
  chars().count()`, char spans); punct emission reproduces old
  push-then-immediate-flush at `col + 1` with len 1 (old buffer provably
  empty between iterations - visible in the removed code, the only push
  site was followed by its own flush); comment/string scans keep per-CHAR
  line/column; `scan_raw_string_with_fallback` preserves the old priority
  (terminator commit outranks earlier unescaped quote, first-terminator
  wins), the escape-state-tracked fallback stop, and the EOF outcome, with
  arithmetic re-derived to match the old two-pass accounting exactly.
  ASCII-byte equivalence argument is sound (continuation bytes are all >=
  0x80). No `Vec<char>`, no retained dual representation: the old
  `flush_word`/`flush_punct`/pending-punct are fully removed, not kept
  beside the new paths. Functions < 100 lines (longest ~89), file 471.
- `c8ae8ea`: journal only. Tokenizer blob unchanged.

## Fresh gates (reviewer own stdout, verbatim)

- `python tools/testrun.py --distill -- -p ariadne-lang-cpp` ->
  `156 passed / 0 failed / 0 ignored / 2 suites`
- `... --test m27_nonascii_corpus_test` -> `2 passed / 0 failed / 0 ignored / 1 suites`
- `... --test m18_cpp_plugin_parity_test` -> `23 passed / 0 failed / 0 ignored / 1 suites`
- `... --test m00_golden_baseline_test --test m00_facts_path_parity_test` ->
  `4 passed / 0 failed / 0 ignored / 2 suites`
- `... --test m17_plugin_seam_test` -> `8 passed / 0 failed / 0 ignored / 1 suites`
- `python tools/hygiene_check.py` -> zero blocking; no row in
  `tokenizer.rs`/`fixture_parity_tests.rs` (old `i`/`c` R3 rows gone, no new rows).
- `python tools/encoding_guard.py --all` -> silent (clean).
- `python tools/distill.py -- cargo check --workspace` ->
  `ok (exit 0, 0 warnings emitted)`
- `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 219 tests ... OK`
- All five test lines match the implementer's journal lines byte-for-byte.
  D2 (153->155 re-brief) consistent: 156 = 155 UNEDITED + 1 new lock.

## Deviation dispositions

- D1: ACCEPTED (adjudication 3).
- D2: ACCEPTED - baseline 155 confirmed by my own pre-existing count
  reconciliation (156 = 155 + 1).
- D3: ACCEPTED (adjudication 4).
- D4: ACCEPTED - one match test over 19 committed oracles; non-vacuity of
  the walk itself guarded (`tus.len() >= 15` assertion) and drift detail is
  per-TU. Reconciliation wording is consistent (the lock counts inside the
  156).
- D5: ACCEPTED - probe mechanics sound (single ignored test,
  `--test-threads=1`, counting wrapper); report exists ONLY at
  `target/ar31-alloc-probe.txt` (verified: 20 lines, TOTAL 458, per-TU
  values exactly matching the journal's 624->458 table). Tree probe-free:
  repo-wide grep for `GlobalAlloc|ar31` over `*.rs` -> no matches; the
  committed suite module contains only walk/render/match code (no generator,
  no allocator).

## Other implementer claims - adjudicated

- Parser.rs stop-and-report trigger: plausible non-fire accepted. The
  characterization locked 19 TUs incl. all three m27 non-ASCII specimens,
  and the m27 cold-pipeline lock re-ran green - the fixture surface that
  could reveal a live underflow was exercised and clean. No new evidence
  for the 2026-10-05 `parser.rs:61` memory.
- Bench under AR-25 reviewer concurrency: the PASS row
  (`bench compare: 3 run(s), no regressions past 1.25x baseline`) is not
  false comfort - host contention makes the 1.25x gate MORE likely to fire,
  so a true stage-timing regression could not hide behind the concurrency.
  What contention does prevent is establishing the quantitative
  neutral-or-better MARGIN. Since closeout owes a quiet-tree row anyway, no
  fresh bench was demanded here; the margin confirmation rides that row
  (finding 3, ledger candidate).
- `cargo check --workspace` 0 warnings: reproduced verbatim.
- Report-evidence check (tool-output distillation): journal is digest-shaped
  throughout - verdict/digest lines, exit codes, capture paths, distilling
  entry points only (`testrun.py --distill`, `distill.py -- ...`), zero raw
  dumps. PASS.

## Findings (all minor, no rework)

1. MINOR (closeout ledger candidate): the `underflow_fix_tests.rs` module
   doc prescribes running under BOTH debug and release profiles
   (`cargo test --release -p ariadne-lang-cpp`); the contract's gate list
   covers the default (debug) run only. Release wrap would also be caught
   by the parity oracles (a wrapped column cannot match the frozen char
   values), so no coverage hole in practice - but the milestone closeout
   should include one release-profile `-p ariadne-lang-cpp` line.
2. MINOR (journal wording): D4 says the suite is "counted separately from
   the 155 baseline", while gate #6 counts it INSIDE the 156. Both readings
   reconcile; tighten the wording on next journal touch so a future reader
   does not hunt for an out-of-suite number.
3. MINOR (closeout carries, already owned): the quiet-tree
   `bench_compare` row must confirm the rework's stage-timing margin
   (contract's "neutral-or-better"); the AR-31 pass row was captured under
   a concurrent AR-25 reviewer lane. Not a blocker per dispatch.

## Not run (declared)

- `testrun --baseline` - dispatch errata (file-mode; absent file; bare exits
  2). Workspace census gate belongs to closeout under Planning Law sizing.
- `ui_smoke` - dispatch: not required, closeout owns it.
- `bench_compare` - dispatch: not required, closeout owns the quiet-tree row.
- Four bash forms were refused by the global catch-all deny (verbatim in
  execution-results): a chained-pipe `git rev-parse | Measure-Object`, `git
  ls-tree`, `Test-Path`, `git status --porcelain <path>`. All underlying
  evidence was obtained through permitted tools (glob, Read, grep,
  checkpoint_guard) - no gate was skipped.
