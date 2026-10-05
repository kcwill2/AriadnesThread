# T0b Handoff - M27 tu-index-hang-throughput: approved minimized non-ASCII specimen fixture

Milestone: `tu-index-hang-throughput` (M27, ledger row 2026-10-05). Single
module-dir handoff (`tests/`). Sizing law held: 1 module dir (`tests/`), 5
implementer gate invocations (<= ~10), ~180-280 expected produced lines
(<= ~1200). Workspace-wide gates belong to the reviewer hop.

Why: the E3 offender shape - non-ASCII bytes inside a WORD run AND inside a
PUNCT run - must become a locked regression corpus (I12: "the corpus that
panicked now finishes") through the DOCUMENTED frozen-set approval path:
new sibling directory only, never inside `tests/fixtures/basic` /
`.../expensive`; landing requires the reviewer-approved commit + the dated
ledger note (exact text at the bottom - orchestrator lands it with the
verdict; you do not write the ledger).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
   sec. 0 (mechanism), sec. 2 (I12), sec. 4 (T0b row), sec. 7 (offender-TU
   rule, context), R5.
3. `tasks/tu-index-hang-throughput-t0-evidence.md` (the landed T0 report -
   offender rule + mechanism sections).
4. `tests/fixtures/basic/` (layout model: `src/`+`include/`, no compile db)
   and `tests/integration/tests/m00_golden_baseline_test.rs` (fixture-copy
   idiom) + `tests/integration/tests/common/mod.rs` (helpers; READ-ONLY).
5. `tasks/milestone_ledger.md` "Byte-locked sets" row `tests/fixtures/**`
   (frozen; changes need explicit review approval - THIS hop is that
   approval's vehicle).

## Objective and invariants

Land a minimized specimen corpus + its locks:

1. `tests/fixtures/m27-nonascii-underflow/` - new sibling dir containing:
   - `src/word_run.cpp` - small TU where a word run contains a multi-byte
     character near the start of a line: identifier `caf\u{e9}_cup` (the
     U+00E9 as a SINGLE-ENCODED UTF-8 byte pair `C3 A9` in the committed
     data bytes), plus a second multi-byte word further down the same line.
   - `src/punct_run.cpp` - small TU where a punct run contains a multi-byte
     character: the U+00AB GUILLEMET `«` between ASCII operators
     (`int seam = 1 « 2;` - tokenizer-classifiable as punctuation; see the
     contingency below).
   - `include/runs.h` - trivial header both TUs include.
   - `README.md` - states: M27 T0b approved specimen (E3 shape), single
     encoded UTF-8 ON PURPOSE, never extend in place, sibling-dir rule.
   Data-file bytes are corpus content (like every fixture) - the pure-ASCII
   authored rule binds the TEST file and README (use `\u{e9}`-style escapes
   in Rust literals; the fixture bytes themselves are the point).
2. `tests/integration/tests/m27_nonascii_corpus_test.rs` (NEW):
   - `test_specimen_bytes_are_single_encoded_utf8` (NOT ignored - green at
     landing and after T2): reads both .cpp files as bytes;
     `std::str::from_utf8` OK; word file contains the `C3 A9` sequence with
     ASCII-alphanumeric bytes BOTH neighbors (word-run shape); punct file
     contains the `C2 AB` sequence with non-alphanumeric neighbors
     (punct-run shape); NO file contains the U+00C3 (`Ã`) char (the
     double-encoding signature) - proves the fixture is not mojibake;
     README + test file contain no raw non-ASCII byte (assert their bytes
     `< 0x80` count == 0).
   - `test_m27_nonascii_corpus_cold_index_completes` (ignored - RED for
     T2): copies the fixture to a temp dir (model the copy idiom used by the
     existing temp-repo-copy integration tests), runs a cold
     `ariadne_pipeline::Pipeline::new` + `ensure_index`; asserts completion
     and, reading the written shard JSONs with `serde_json`, that no stored
     token `column` exceeds its source line's CHAR count + 1 (release-wrap
     detector, I14-shape). Ignore attribute EXACTLY:
     `#[ignore = "M27 T0b lands this RED for T2: cold index panics in DEBUG
     at tokenizer.rs:311/:331 (E3 class); the T2 fix commit removes this
     #[ignore] line as its single-line rider (I12)"]`.

Contingency (record the outcome in the journal): if you observe the parser
rejecting `«` with an Err (NOT the expected tokenizer panic) during your Red
capture, replace the punct-run character with another non-word multi-byte
character that keeps a punct-run shape, note the swap + byte sequence in the
journal AND your report-back so the re-approval is visible - fixture bytes
stay a one-line data change.

Invariants: `tests/fixtures/basic/**` and `tests/fixtures/expensive/**`
byte-identical (shape R5; any diff there = auto-REJECT); zero `crates/`
change; goldens untouched; existing census stays green UNEDITED.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/tu-index-hang-throughput-t0b-journal.md` (journal; always in-set)
- `tests/fixtures/m27-nonascii-underflow/README.md` (NEW)
- `tests/fixtures/m27-nonascii-underflow/src/word_run.cpp` (NEW)
- `tests/fixtures/m27-nonascii-underflow/src/punct_run.cpp` (NEW)
- `tests/fixtures/m27-nonascii-underflow/include/runs.h` (NEW)
- `tests/integration/tests/m27_nonascii_corpus_test.rs` (NEW)

You MUST NOT touch: `tests/fixtures/basic/**`,
`tests/fixtures/expensive/**`, any other fixture; `tests/golden/**`;
`tests/integration/tests/common/mod.rs` and every existing test file
(they are the review oracles); ANY file under `crates/`, `tools/`, `src`;
`tests/integration/Cargo.toml` (deps already suffice: ariadne-pipeline,
serde_json, tempfile - verify before assuming otherwise, and if a dep is
truly missing STOP and report, do not add it silently);
`tasks/milestone_ledger.md` (planner-owned). No `git stash`, no branch
change.

## RGB protocol
1. Red: author both tests + fixtures. The byte test PASSES at landing
   (it locks fixture shape, green by design). The completes-test is
   `#[ignore]`d; its Red evidence is ONE explicit ignored run you capture:
   `python tools/distill.py -- cargo test -p ariadne-integration-tests
   --test m27_nonascii_corpus_test -- --ignored --nocapture` -> DEBUG panic
   `attempt to subtract with overflow` (the E3 class, reproduced on the
   minimized corpus). That captured failure IS the Red evidence (an ignored
   test is the sanctioned way to carry a failing lock across handoffs; it is
   NOT a weakened assertion).
2. Green: `python tools/testrun.py --distill -p ariadne-integration-tests
   --test m27_nonascii_corpus_test` -> `1 passed / 0 failed / 1 ignored`
   (the byte test passes; the ignored test is counted, never run).
3. Refactor: comments naming the byte sequences and the E3 shape; re-run.

## Hygiene (review-enforced)
Test file pure ASCII (escapes only), functions < 100 lines; fixture .cpp
files < 40 lines each (minimized on purpose); `python tools/hygiene_check.py`
exit 0.

## Frozen locks (each with its proving command)

| Lock | Proof |
|---|---|
| `tests/fixtures/basic/**`, `.../expensive/**` byte-identical | `git diff --stat <start-rev>..HEAD` shows `tests/fixtures/m27-nonascii-underflow/` as the ONLY fixture path |
| no new files inside basic/expensive (byte-locked table) | same diff-scope proof |
| existing suite green UNEDITED | scoped distill run exit 0 with unchanged pre-existing test results |
| mojibake gate: fixture passes encoding_guard as legitimate single-encoded UTF-8 | `python tools/encoding_guard.py --staged` exit 0 (reviewer also runs `--all`) |
| census delta | ignored count 4 -> 5 (the carried RED test); stated verbatim in the report so the reviewer reconciles `... / 5 ignored / ...` at the next census |

## Gates (exact commands, timeout 600000 ms each; scoped only)
Distilling entry points only (ledger row 58). Workspace-wide gates belong to
the reviewer hop.
1. Red capture: `python tools/distill.py -- cargo test -p
   ariadne-integration-tests --test m27_nonascii_corpus_test -- --ignored
   --nocapture` - capture the panic line (exit 1 EXPECTED and correct).
2. Green: `python tools/testrun.py --distill -p ariadne-integration-tests
   --test m27_nonascii_corpus_test` -> `1 passed / 0 failed / 1 ignored`,
   exit 0.
3. `python tools/hygiene_check.py` - exit 0.
4. `python tools/encoding_guard.py --staged` - exit 0 (the fixture bytes
   must clear the mojibake gate as legitimate UTF-8).
5. Commit: `python tools/distill.py -- python tools/commit_scoped.py --scope
   tests/fixtures/m27-nonascii-underflow --scope
   tests/integration/tests/m27_nonascii_corpus_test.rs --scope
   tasks/tu-index-hang-throughput-t0b-journal.md -m "test(fixtures): approved
   m27-nonascii-underflow specimen (word AND punct non-ASCII runs) + carried
   RED completion lock (M27 T0b)"` (never `git add .`).

Total implementer gate invocations: 5 (<= 10).

## Acceptance (machine-checkable; implementer hop)
1. Red capture shows the `subtract with overflow` panic on the minimized
   corpus (this also field-confirms the T0 mechanism on the fixture).
2. Scoped green line: `1 passed / 0 failed / 1 ignored` exit 0.
3. hygiene + encoding exit 0. 4. `git show --stat` = touch-set only.
Reviewer hop (the APPROVAL itself): fresh scoped run; `encoding_guard
--all`; census delta exactly +1 ignored / +1 passed; per-commit
`git show`; frozen-set status census; THEN the verdict is APPROVED and the
orchestrator lands the dated ledger note VERBATIM in the M27 row's Notes:
`2026-10-05 fixture approval (M27 T0b): tests/fixtures/m27-nonascii-underflow/
added via the documented frozen-set approval path (reviewer-approved commit
+ this dated note); single-encoded UTF-8 word-run AND punct-run specimens;
nothing inside basic/expensive`.

## Discipline clauses
- UTF-8: Write/Edit tools only; test file + README pure ASCII; fixture bytes
  via the Write tool's exact bytes (never PowerShell pipelines - AGENTS.md).
- Touch only the touch-set; if the fixture seems to break an existing
  golden/parity test (it must not - no one references it yet): report, do not
  fix, do not touch the oracle.
- Journal checkpoint after each gate.
- If context exhaustion looms: STOP, disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/tu-index-hang-throughput-t0b-handoff.md --scope ... --cmd ...`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence: the captured panic line
(cite the distill capture path; do not paste stacks). 3. Green canonical line.
4. Byte sequences actually committed (hex) + contingency note if the punct
char was swapped. 5. Out-of-scope observations. Never paste raw output.

## Reviewer-gate reference
Adjudicated via the `review-gate` skill. Extra reviewer checks: sibling-dir
rule held (basic/expensive diff-EMPTY); fixture bytes single-encoded (hex
inspect `word_run.cpp` / `punct_run.cpp`); ignore attribute wording names T2
as the un-ignorer; byte test asserts both run shapes; approval ledger note
lands with the verdict (planner lands it if the orchestrator cannot).

## Sequencing note
Parallel-safe NOW with M26 V1/V2, T0-report, and T1a/T1b (disjoint trees).
T2 consumes this fixture (its completion lock's `#[ignore]` line is T2's
single-line rider) - T2 dispatches only after BOTH this and T0-report land.
