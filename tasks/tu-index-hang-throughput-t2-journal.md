# T2 journal - M27 root-cause fix: char-count column at BOTH tokenizer flush sites + same-class usize audit + DP-6 bump rider

Handoff: `tasks/tu-index-hang-throughput-t2-handoff.md`. Start rev:
`de5622f` (HEAD at hop start, verified via
`python tools/distill.py -- git log --oneline -8`; tree carried foreign
untracked/modified plan and task docs from the orchestrator - untouched,
never staged; commits use `commit_scoped.py` with the exact touch-set).

This journal is pure ASCII: multi-byte characters are named by code point
and stored byte sequence, never by literal (repo UTF-8 authoring rule).

## Inputs read (binding order)

1. Handoff (complete contract).
2. Shape plan `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
   sec. 0 (verified mechanism + E1/E2/E3), sec. 2 (I1/I7/I12/I14),
   sec. 3.4 (fix seam, audit obligation, data-integrity flag), sec. 6
   (H-B CONFIRMED = root cause), sec. 11 (DP-6).
3. `crates/lang-cpp/src/tokenizer.rs` (whole file; flush sites :299-337,
   correct `end: start + len` at :313/:333, `col` per-char at :115).
4. `crates/lang-cpp/src/parser.rs` (whole file; M25 floor guard :56-81
   read as UNTOUCHABLE - do not weaken).
5. `crates/index/src/cache.rs` :10-16 (const + epoch doc) and :331
   (assert literal).
6. `tests/integration/tests/m12_incremental_analysis_test.rs` :93-95 and
   :193-198 (pinned epoch literal + comments; M18.10 bump precedent).
7. `tasks/tu-index-hang-throughput-t0b-handoff.md` (un-ignore rider
   wording) + `tasks/tu-index-hang-throughput-t0-evidence.md` (mechanism
   sec. 3, DP-6 sec. 8, data-integrity disposition sec. 9) +
   `tasks/tu-index-hang-throughput-t0b-journal.md` (committed driver bytes
   and the proved arithmetic: a 2-byte punct char can only floor the
   column at 0; a 3-byte char at column 1 is the :331 panic driver).
8. `docs/specs/tool-output-distillation.md` (digest shapes, evidence rules).
9. `tests/integration/tests/m27_nonascii_corpus_test.rs` (the `#[ignore]`
   line at :143 to delete; wrap-detector shape mirrored in the unit locks).

## Static analysis (drives the unit test shapes; precedes any gate run)

Re-derived from the live `tokenizer.rs` @ start-rev and independently
confirmed by the T0b journal Gate-1 empirical capture:

- `col` advances per CHAR; flushes subtract BYTES (`word.len()` /
  `punct.len()`), while `end` correctly uses `chars().count()` - the unit
  mismatch is internal to the file.
- A word run with exactly ONE extra byte at column 1 floors `column` at 0
  (wrong value, no panic). TWO extra bytes at column 1 (or one 3+ byte
  char) give `-1` -> DEBUG panic at :311.
- Single-char punct flushes at :107 with `column = col + 1`; a 2-byte
  punct char (U+00AB, C2 AB) can NEVER underflow (minimum column 0); a
  3-byte punct char (U+20AC, E2 82 AC) at column 1 gives `2 - 3` ->
  DEBUG panic at :331:21 - the exact E3 site.
- Consequence for the handoff's literal test wording ("a punct run
  containing U+00AB at/near line start ... Pre-fix debug: panic"): a bare
  U+00AB cannot panic - proved empirically by the approved T0b Gate 1
  first capture. The unit source therefore combines the U+20AC panic
  driver (:331 class, same driver as the committed T0b fixture) with the
  U+00AB two-byte shape the handoff names, satisfying both clauses of the
  single test spec. Recorded here as an interpretation, not a deviation
  from scope: the :331 panic class and the U+00AB shape are both locked.
- Release profile (overflow-checks off): the same subtractions WRAP
  silently to ~usize::MAX columns - the property asserts (column within
  [1, line_chars + 1]) fail without any panic, which is what pins the
  silent wrap in the Gate 1 RED evidence.

## SAME-CLASS usize SUBTRACTION AUDIT (tokenizer.rs + parser.rs)

Enumeration grep (reviewer will re-run independently): pattern
`\w+ - \w+` / `-= ` / `- 1` / `- 2` / `- 3` / `.len() -` over
`crates/lang-cpp/src/tokenizer.rs` and `crates/lang-cpp/src/parser.rs`
@ start rev. Every hit disposed; non-hits are additions
(`col += 1`, `self.i += 2`, `*i += 1`) or comparisons.

| # | Site | Expression | Disposition | One-line proof |
|---|---|---|---|---|
| A1 | tokenizer.rs:233 | `chars[quote_at - 1]` | SAFE-BY-INVARIANT | short-circuit `quote_at == 0 \|\|` guarantees `quote_at >= 1` before the index |
| A2 | tokenizer.rs:269 | `content_start - delimiter_start` | SAFE-BY-INVARIANT | `content_start` starts AT `delimiter_start` (:260) and only increments (:264) - monotonic, never below |
| A3 | tokenizer.rs:311 | `column - word.len()` | FIXED (commit 1) | BYTE length subtracted from CHAR-counted column; fixed to `column - len` reusing the existing `let len = word.chars().count()` (the value `end` already uses) |
| A4 | tokenizer.rs:331 | `column - punct.len()` | FIXED (commit 1) | same class, the E3 panic site (:331:21); fixed to `column - len` reusing `punct.chars().count()` |
| A5 | parser.rs:70 | `self.brace_depth -= 1` | ALREADY-GUARDED (untouched) | M25 floor guard :61-69 returns early at depth 0; do not weaken (I14 lineage lock) |
| A6 | parser.rs:283 | `self.tokens[self.i - 1]` | SAFE-BY-INVARIANT | reached only under guard `self.i >= 2` (:281) |
| A7 | parser.rs:285 | `self.tokens[self.i - 2]` | SAFE-BY-INVARIANT | same guard `self.i >= 2` (:281) |
| A8 | parser.rs:287 | `self.tokens[self.i - 3]` | SAFE-BY-INVARIANT | inner guard `self.i >= 3` (:286) |
| A9 | parser.rs:306 | `paren_depth -= 1` | SAFE-BY-INVARIANT | loop condition `while ... && paren_depth > 0` (:301) - decrement only executes at value >= 1 |
| A10 | parser.rs:368 | `self.tokens[self.i - 1]` | SAFE-BY-INVARIANT | guard `if self.i > 0` (:367) |
| A11 | parser.rs:396 | `param_paren -= 1` | SAFE-BY-INVARIANT | loop condition `while ... && param_paren > 0` (:390) |
| A12 | parser.rs:483 | `self.tokens[self.i - 1]` | SAFE-BY-INVARIANT | guard `if self.i > 0` (:482) |

Audit outcome: FIXED x2 (the two declared flush sites),
SAFE-BY-INVARIANT x8, ALREADY-GUARDED x1. NO second real bug found ->
`parser.rs` stays byte-identical in commit 1. Nothing in the audit needs
a change beyond `tokenizer.rs`/`parser.rs`, so no R4 scope door was hit
by the audit itself.

## Machine-pinned `CURRENT_ANALYSIS_VERSION` literal census (DP-6 rider)

Grep `"2\.0\.0"` + `CURRENT_ANALYSIS_VERSION` across `crates/` and
`tests/`:

| Site | Kind | In touch-set? | Rider action |
|---|---|---|---|
| crates/index/src/cache.rs:16 | const definition | yes | `2.0.0` -> `2.1.0` + doc sentence (commit 2) |
| crates/index/src/cache.rs:331 | test assert literal | yes | literal -> `2.1.0` (commit 2) |
| tests/integration/tests/m12_incremental_analysis_test.rs:95 (+ comments :93-94, :194) | test assert literal + comments | yes | literal -> `2.1.0`, comments updated (commit 2) |
| tests/integration/tests/m27_nonascii_corpus_test.rs:143 | `#[ignore]` carrier | yes | delete exactly this line (commit 2) |
| tests/integration/tests/m21_scala_semantic_index_test.rs:499 | HARD runtime assert `assert_eq!(CURRENT_ANALYSIS_VERSION, "2.0.0", "M18.10 cache epoch witness")` inside the RUNNABLE (not ignored) `scala_fingerprint_moves_only_the_scala_digest` | NO - outside the touch-set | REPORT, DO NOT FIX (see stop-and-report below) |
| crates/report/src/manager/tests.rs:596 | comment naming the historical 1.0.0 -> 2.0.0 bump | no | untouched; stays accurate as history |
| golden manifests | no version literal present (masking keeps them version-neutral; grep of `tests/golden` for `2.0.0` = zero hits) | n/a | untouched |

STOP-AND-REPORT (filed for the reviewer hop, not fixed): the handoff names
the DP-6 dependents as "cache const + m12 pin + m27 un-ignore", but
m21_scala_semantic_index_test.rs:499 is a THIRD machine-pinned literal of
the same constant and is NOT in the touch-set. After the bump, Gate 6
(`-p ariadne-integration-tests`) must show exactly one failure there -
`scala_fingerprint_moves_only_the_scala_digest`. Per the touch-set rule
("any edit outside = blocker, report don't fix") the file is NOT edited
here. The one-line literal-only update `2.0.0` -> `2.1.0` (witness-string
wording per reviewer/orchestrator call) is a machine-pinned dependent the
handoff under-enumerated; it needs the same sanctioned-rider treatment as
the m12 pin. Not a golden/parity failure - a pinned literal the plan
miscounted. Census anchor note: the M27 ledger row anchor is
`5 ignored / 86 suites`; the m27 un-ignore rider legitimately moves the
ignored count to 4 (recorded verbatim at Gate 6).

## Gate journal

(checkpoints appended below per gate; Gate 2 ran first so the
pre-fix ASCII baselines could be recorded from its RED capture before
Gate 1's release run - both RED gates completed BEFORE any fix edit)

### Gate 2 (Debug-RED) - RED CONFIRMED

`python tools/testrun.py --distill -p ariadne-lang-cpp` (final capture)
-> digest `### cargo test FAIL (exit 1) [capture:
target/testrun-capture.txt, 185 lines]`, canonical line:
`149 passed / 3 failed / 0 ignored / 1 suites`.
- `test_e3_punct_shape_no_panic_and_bounded_columns` panicked at
  `crates\lang-cpp\src\tokenizer.rs:331:21: attempt to subtract with
  overflow` - the EXACT E3 site + message on a unit source.
- `test_word_shape_no_panic_and_bounded_columns` panicked at
  `crates\lang-cpp\src\tokenizer.rs:311:21: attempt to subtract with
  overflow` - the :311 word class.
- `test_mixed_multibyte_matrix_bounded` failed the bound:
  `column 0 outside [1, 7]` (floored column, pre-panic row).
- `test_ascii_path_tokenization_unchanged` PASSED (149 passed) - the I1
  lock passes at Red by design.

Baseline recording (Gate 2 iterations, same command, pre-fix tree): the
three ASCII baselines in `underflow_fix_tests.rs` were transcribed from
the actual pre-fix streams in the Gate-2 iterations' assert lefts
(target/testrun-capture.txt), including the known string-literal column
lag (quirky characterization value, mirrors characterization_tests.rs).

### Gate 1 (Release-RED, before fixing - pins the wrap)

`python tools/distill.py -- cargo test --release -p ariadne-lang-cpp`
-> digest `### cargo build FAIL (exit 101)`, capture
`target/distill-capture.txt` (196 lines). Release overflow-checks are
OFF, so no panics - boundedness fails on the WRAPPED columns:
- `column 18446744073709551615 outside [1, 9] on line 1 ... - token
  Punct(<euro>)` (punct test: 2usize - 3usize wraps to usize::MAX-0).
- `column 18446744073709551615 outside [1, 15] ... Word(cafe...cup)`
  (word test: 10usize - 11usize wraps).
- matrix: `column 0 outside [1, 7]` on its first floored row.
- pinned result line: `test result: FAILED. 149 passed; 3 failed; 0
  ignored`.
The silent release wrap is empirically pinned; post-fix these wraps
become mathematically impossible (same column now subtracts the char
count it advanced by).

