# T0b journal - M27 approved minimized non-ASCII specimen fixture

Handoff: `tasks/tu-index-hang-throughput-t0b-handoff.md`. Start rev:
`62ce72bbc914549ffd10bca057d6737b8b3844b6` (HEAD at hop start; verified via
`tools/distill.py -- git rev-parse HEAD`). Working tree carried foreign
modified/untracked plan and task docs from the orchestrator; untouched and
never staged (commit uses `commit_scoped.py --only` with the exact touch-set).

This journal is pure ASCII: fixture characters are named by code point and
stored byte sequence, never by literal (repo UTF-8 authoring rule; fixtures
carry the real bytes, the prose never does).

## Inputs read (binding order)

1. Handoff (complete contract).
2. Shape plan sec. 0 (tokenizer unit-mismatch verified fact + E1/E2/E3),
   sec. 2 I12/I14, sec. 4 T0b row, sec. 7 offender-TU rule, R5.
3. `tasks/tu-index-hang-throughput-t0-evidence.md` (offender rule sec. 6,
   mechanism sec. 3: byte-length subtracted from char-counted column at
   `tokenizer.rs:311` / `:331`).
4. `tests/fixtures/basic/` tracked layout (`src/` + `include/`, no compile
   db); `m00_golden_baseline_test.rs` copy idiom; `common/mod.rs` (read-only).
5. Ledger byte-locked `tests/fixtures/**` row (this hop = approval vehicle;
   ledger itself NOT touched - planner lands the dated note with the verdict).

## Static analysis done BEFORE authoring (drives contingency handling)

Re-derived the underflow arithmetic from the live `crates/lang-cpp/src/
tokenizer.rs` @ start-rev (zero crates/ changes by this hop):

- `col` advances per CHAR (`:49,:115`). `flush_word` (`:311`) subtracts
  `word.len()` (BYTES) from the char-counted column. A word starting at
  column `s+1` underflows iff `extra_bytes(word) > s + 1`. One 2-byte char
  in a word at column 1 gives `column = 0` - a WRONG value but NO panic.
  Two extra bytes at column 1 (one word carrying TWO U+00E9) give `-1` ->
  DEBUG panic at `tokenizer.rs:311`.
- Single-char punct flushes at `:107` with `column = col + 1`;
  `flush_punct` (`:331`) subtracts the char's BYTE length. A 2-byte punct
  char (U+00AB guillemet) can never underflow (minimum `column = 0`, at
  column 1). A 3-byte punct char at column 1 gives `2 - 3 = -1` -> panic
  at `tokenizer.rs:331:21` - the EXACT E3 site. (The live-corpus E3
  trigger is consistent with a 3-byte non-word char at file start, e.g. a
  UTF-8 BOM; this fixture reproduces the class with an explicit
  character.)
- Consequence: the handoff's literal specimen shapes (one U+00E9 per word;
  guillemet mid-line) lock the run shapes but do NOT panic by themselves.
  The handoff mandates panic Red evidence (acceptance 1). Protocol
  followed: author v1 exactly per handoff spec, run Gate 1 to demonstrate
  the gap EMPIRICALLY, then apply the handoff's declared contingency (swap
  a multi-byte char that keeps the run shapes and reproduces the panic),
  recording the swapped bytes here and in the report-back. Fixture bytes
  stay one-line data changes per the contingency clause.

## Environment facts locked

- `tests/integration/Cargo.toml`: `ariadne-pipeline`, `serde_json`,
  `tempfile`, `ariadne-repo` all present as dependencies (no Cargo.toml
  edit needed; `FileId::from_relative_path` reproducible in-test for the
  shard-JSON file_id -> path map).
- `Pipeline::new` -> `compute_inputs` selects the C++ shaper by root
  markers (`CMakeLists.txt` / `compile_commands.json` / `.vcxproj`); a
  marker-free copy yields ZERO TUs. The completion test therefore writes a
  synthetic `compile_commands.json` (m18 idiom) into the TEMP COPY only -
  the committed fixture keeps the basic-style no-compile-db layout.
  The test also asserts `reanalyzed_tus == 2` so an empty corpus can never
  silently pass the lock.
- `lang-cpp` has no `Err(` construction in parser/lib - the parser cannot
  reject garbage; the panic (not an Err) is the only failure mode, so the
  Red capture discriminates cleanly.
- No existing test enumerates `tests/fixtures/*` directories; the new
  sibling dir cannot perturb any census (verified by grep).
- `encoding_guard.py` signatures (`c3 82`, `c3 83`, `c3 a2 e2 82 ac`,
  double-encoded FFFD): none present in `C3 A9`, `C2 AB`, `E2 82 AC` -
  the fixture bytes clear the mojibake gate as legitimate single-encoded
  UTF-8 (Gate 4 proves it on staged blobs).
- Repo-wide existing `#[ignore]` attributes: 3 static ones found; the
  handoff states the run census as 4 ignored; this hop lands exactly one
  more (`+1 ignored / +1 passed` per the reconciliation row).

## Gate journal

### Gate 0 (setup, not counted)

Touched-set authored: README, include/runs.h, src/word_run.cpp,
src/punct_run.cpp (v1 per handoff literal spec), the new test file.

### Gate 1 (first capture) - empirical contingency trigger CONFIRMED

`python tools/distill.py -- cargo test -p ariadne-integration-tests --test
m27_nonascii_corpus_test -- --ignored --nocapture` -> exit 0,
`test test_m27_nonascii_corpus_cold_index_completes ... ok`
(capture: `target/distill-capture.txt`). The literal-spec v1 does NOT
panic and does NOT Err - it COMPLETES. This matches the static analysis
above: one 2-byte word char at column 1 floors the column at 0 (no
underflow); a 2-byte punct char can only floor at 0. A passing completion
test is a not-RED lock - unacceptable per acceptance 1. Contingency
applied (the declared mechanism; the observed failure mode is milder than
the handoff's parser-Err wording - it is a pass, same remedy):

- punct_run.cpp data line: `int seam = 1 <<guillemet>> 2;` ->
  `<euro>uro seam = 1 <<guillemet>> 2;` - driver char U+20AC (bytes
  E2 82 AC) at column 1; guillemet U+00AB (C2 AB) and its flanking spaces
  KEPT, punct-run shape intact.
- word_run.cpp data line: `int cafe<eacute>_cup = cafe<eacute>ine_total;`
  -> `cafe<eacute>_cafe<eacute> cafe<eacute>ine = cafe<eacute>_cup;` -
  the leading word carries TWO C3 A9 pairs at column 1; identifier
  cafe+acute_cup retained; a C3 A9 between ASCII letters (f|i in
  cafe+acute+ine) retained - word-run shape intact.
- Both are one-line data changes per the contingency clause; comments
  (pure ASCII) and README byte inventory updated to name the real bytes.
- Also fixed: unused-variable warning (`_fixture_temp`).

### Gate 1 (Red capture, final) - RED CONFIRMED

`python tools/distill.py -- cargo test -p ariadne-integration-tests --test
m27_nonascii_corpus_test -- --ignored --nocapture` -> digest
`### cargo build FAIL (exit 101)` (non-zero as the handoff expects; the
handoff prose says "exit 1", raw `cargo test` uses 101 for a failed suite -
recorded as a non-substantive deviation). Capture
`target/distill-capture.txt` (19 lines):
`thread 'test_m27_nonascii_corpus_cold_index_completes' (35032) panicked at
crates\lang-cpp\src\tokenizer.rs:331:21: attempt to subtract with overflow`
- the EXACT E3 site and message, reproduced on the minimized corpus.
(The punct-run TU is reached first in this unit's order, so the captured
site is :331; the word-run driver reproduces the same class at :311 - the
ignore attribute names both sites.)

### Gate 2 (Green)

`python tools/testrun.py --distill -p ariadne-integration-tests --test
m27_nonascii_corpus_test` -> digest
`1 passed / 0 failed / 1 ignored / 1 suites`, exit 0 (capture
`target/testrun-capture.txt`). Byte-shape lock passes; the completion lock
is carried ignored (counted, never run). Re-run after adding the
E2 82 AC driver-byte presence assertion: same digest, exit 0.

### Gate 3 (Hygiene)

`python tools/hygiene_check.py` -> exit 0. R1/R2 clean; the emitted R3 rows
are warn-only and all pre-existing (zero hits in any touched file).

### Gate 4 (Mojibake guard)

Touched-set staged explicitly, then
`python tools/encoding_guard.py --staged` -> exit 0, no violations: the
fixture blobs (`C3 A9`, `C2 AB`, `E2 82 AC`) clear the double-encoded
signatures as legitimate single-encoded UTF-8; README/test/journal are
pure ASCII. Re-run after the final test-file edit: exit 0.

### Gate 5 (Commit)

`python tools/distill.py -- python tools/commit_scoped.py --scope
tests/fixtures/m27-nonascii-underflow --scope
tests/integration/tests/m27_nonascii_corpus_test.rs --scope
tasks/tu-index-hang-throughput-t0b-journal.md -m "test(fixtures): approved
m27-nonascii-underflow specimen (word AND punct non-ASCII runs) + carried
RED completion lock (M27 T0b)"` - hash reported in the report-back (a
journal inside that commit cannot carry its own hash).

## Committed byte inventory (hex)

- `src/word_run.cpp`: `C3 A9` twice inside the leading identifier
  (`caf` + C3 A9 + `_caf` + C3 A9) at column 1; one `C3 A9` between ASCII
  letters `f`|`i` in the second identifier on the line; one `C3 A9` in the
  `cafe` + C3 A9 + `_cup` identifier (same line).
- `src/punct_run.cpp`: `E2 82 AC` (U+20AC driver, first byte-sequence of
  its line, column 1) + `C2 AB` (U+00AB) flanked by spaces between ASCII
  operators (guillemet shape retained).
- `include/runs.h`, `README.md`, the test file, this journal: pure ASCII
  (zero bytes >= 0x80; locked by the byte test for README + test).

## Contingency outcome (re-approval visibility)

Observed mode: v1 fixture COMPLETED (pass), not the handoff's parser-Err
wording; the declared remedy was applied. Punct driver = U+20AC
(E2 82 AC) at column 1 of its line; guillemet C2 AB retained. Word driver
= a second C3 A9 inside the column-1 identifier. Both changes are one-line
data changes. Reviewer hex inspection should expect `E2 82 AC` opening the
driver line of `punct_run.cpp`.

## Out-of-scope riders (journal-only, per mission order)

- F1 (from the previous hop's ACCEPTED findings): the T0-report journal
  `tasks/tu-index-hang-throughput-t0-report-journal.md` claims a
  distill-wrapped commit route where a direct `commit_scoped.py` route
  landed; an errata line is owed there. That file is NOT in this hop's
  touch-set and this handoff sanctions no rider section, so nothing was
  edited here; recorded out-of-scope for the orchestrator to route.

## Deliberate non-actions / deviations

- Did NOT touch: `tests/fixtures/basic/**`, `tests/fixtures/expensive/**`,
  `tests/golden/**`, any oracle test, `crates/`, `tools/`,
  `tests/integration/Cargo.toml` (deps verified sufficient),
  `tasks/milestone_ledger.md` (planner lands the dated approval note with
  the verdict). No golden/parity test references the new fixture dir;
  zero undeclared fallout observed.
- Deviation (non-substantive): handoff prose expects "exit 1" from the Red
  capture; raw `cargo test` uses 101 for a failed suite - non-zero as
  required.
- Deviation (contingency, sanctioned): v1 literal-spec fixtures completed
  without panicking (empirically, Gate 1 first capture); panic drivers were
  added via the handoff's contingency mechanism to satisfy acceptance 1.
- Deviation (hardening): the completion test additionally asserts
  `reanalyzed_tus == 2` so a silently empty corpus can never pass the
  lock (marker-free temp copies shape zero C++ units otherwise); the byte
  test additionally locks the contingency driver sequence E2 82 AC so the
  swapped bytes are themselves locked, per the contingency's re-visibility
  clause. No oracle touched by either.
- Census reconciliation for the next full count: this hop lands
  +1 ignored / +1 passed (census 4 -> 5 ignored) per the handoff's
  reconciliation row; existing suite results UNEDITED.
