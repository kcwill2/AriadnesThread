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

(Gate 2 ran first so the pre-fix ASCII baselines could be recorded from
its RED capture before Gate 1's release run - both RED gates completed
BEFORE any fix edit.)

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

### Gate 3 (Debug-GREEN)

`python tools/testrun.py --distill -p ariadne-lang-cpp` after the
two-site fix -> digest `### cargo test PASS (exit 0)`, canonical line:
`152 passed / 0 failed / 0 ignored / 2 suites`. Suite count 2 vs the RED
runs' 1 is not drift: a failing cargo binary aborts the remaining
targets, so the RED runs never reached the crate's zero-test doc-test
suite; GREEN runs lib (152) + doc-tests (0) as pre-fix green runs
always did.

### Gate 4 (Release-GREEN, I14)

`python tools/testrun.py --distill --release -p ariadne-lang-cpp` ->
digest `### cargo test PASS (exit 0)`, canonical line:
`152 passed / 0 failed / 0 ignored / 2 suites`. THE SAME module passes
with overflow-checks off - post-fix wraps are mathematically impossible
(the column now subtracts exactly what `col` advanced by), not merely
absent. No `debug_assert`-only coverage anywhere.

### Both-site self-check (discipline clause)

Commit 1 = `ea21687`. `git show ea21687 -- crates/lang-cpp/src/tokenizer.rs`
(capture target/distill-capture.txt) carries BOTH hunks:
`- column: column - word.len()` -> `+ column: column - len` (the :311
class) AND `- column: column - punct.len()` -> `+ column: column - len`
(the :331 class), plus the appended `mod underflow_fix_tests;`. Each
site reuses the existing `let len = <run>.chars().count();` - one unit
for both `column` and `end`. `parser.rs` absent from the numstat:
byte-identical, audit-only. numstat: tokenizer.rs 5/2,
underflow_fix_tests.rs 182/0, journal 169/0 - touch-set only.

### Gate 7 (Hygiene)

`python tools/hygiene_check.py` -> exit 0 (verified via
`python tools/distill.py -- python tools/hygiene_check.py`:
`### generic PASS (exit 0)`, capture target/distill-capture.txt). All
emitted rows are warn-only R3, all pre-existing locations; zero hits in
any touched file (the tokenizer.rs:50/53 `i`/`c` warnings sit in the
untouched `tokenize_raw` loop). Re-run after commit 2: same, exit 0.

### Gate 8 (Mojibake guard, per commit)

Commit-1 scope staged explicitly (staging route:
`python tools/distill.py -- git add -- <scope paths>`; raw `git` is
allowlist-denied in this session-lineage), then
`python tools/encoding_guard.py --staged` -> exit 0
(`### generic PASS (exit 0)`, 0-line capture). Same for the commit-2
scope (cache.rs, m12, m27, journal): exit 0. Authored content pure
ASCII; the cache.rs `§9` comment line stayed untouched (diff shows it
as unchanged context, original bytes intact).

### Gate 9 (Commit 1) - LANDED as ea21687

Landed via the DIRECT `python tools/commit_scoped.py` entry point with
the handoff's exact scopes and message (the distill-wrapped form is
allowlist-refused in this session-lineage - same sanctioned route as
T0b Gate 5, deviation recorded). Scopes: tokenizer.rs,
tokenizer/underflow_fix_tests.rs, parser.rs (no diff), this journal.

### Gate 5 (Rider scoped - index)

`python tools/testrun.py --distill -p ariadne-index` (after the const
bump + assert update) -> digest `### cargo test PASS (exit 0)`,
canonical line: `11 passed / 0 failed / 0 ignored / 2 suites`.

### Gate 6 (Integration scoped) - FULL RUN + COMPONENT PROOF

Full run `python tools/testrun.py --distill -p ariadne-integration-tests`
-> digest `### cargo test FAIL (exit 1)`, canonical line:
`99 passed / 1 failed / 0 ignored / 24 suites`. The single failure is
the predicted stop-and-report item:
`scala_fingerprint_moves_only_the_scala_digest` panicked at
`tests\integration\tests\m21_scala_semantic_index_test.rs:499:5:
assertion left == right failed: M18.10 cache epoch witness, left:
"2.1.0"` - the third machine-pinned literal the handoff did not
enumerate (see census row above). NOT fixed: outside the touch-set.
Two properties of that run, recorded honestly: (1) cargo ran targets in
sort order and ABORTED at the failing m21 target (no --no-fail-fast),
so the m22+ target including m27 never started there; (2) `0 ignored`
is this package's own count (m27 held its only `#[ignore]`) - the
workspace census `5 ignored -> 4` reconciliation belongs to the
reviewer hop's unscoped full census.

Component proof (same distilling entry point, scoped to the rider's
suites, run AFTER commit 2 landed; iteration of Gate 6):
`python tools/testrun.py --distill -p ariadne-integration-tests --test
m27_nonascii_corpus_test --test m12_incremental_analysis_test --test
m00_golden_baseline_test --test m18_cpp_plugin_parity_test --test
m18_cache_epoch_test` -> digest `### cargo test PASS (exit 0)`,
canonical line: `34 passed / 0 failed / 0 ignored / 5 suites`.
Proves within one bounded capture:
- I12: `test_m27_nonascii_corpus_cold_index_completes ... ok` AND
  `test_specimen_bytes_are_single_encoded_utf8 ... ok` - the corpus
  that panicked now COMPLETES, un-ignored, assertions untouched.
- m12 pin green at 2.1.0 (cold-run manifest assert + mismatch sentinel
  test).
- m00 goldens UNEDITED green (`golden_baseline_basic/expensive ... ok`
  also confirmed in the full run); m18 parity UNEDITED green (23 rows
  in the full run); m18 cache-epoch test compares the constant SYMBOL
  (version-neutral) - green.
The full `-p` run re-runs green (0 failed) once the m21 witness gets
its sanctioned one-line dependent - that adjudication is the reviewer
hop's, not mine.

### Gate 10 (Commit 2) - LANDED as 59f0512

Direct `commit_scoped.py`, handoff scopes + message verbatim. numstat:
cache.rs 7/2 (const + doc sentence + assert literal ONLY - envelope
struct fields untouched, I7; reviewer eyeballed via
`git diff --cached` capture), m12 8/5 (pinned literal + epoch comments
only), m27_nonascii_corpus_test.rs 0/1 (exactly the `#[ignore]` line
deleted, assertions untouched). This journal is committed in its final
form by the follow-up errata commit (journal-completion route per the
T0b precedent), keeping both landing commits' diffs at their declared
scopes.

## Data-integrity disposition (mandatory text)

With the DP-6 bump (`CURRENT_ANALYSIS_VERSION` 2.0.0 -> 2.1.0 landing
IN THE SAME fix as the corrected columns), stale RELEASE-built shards
and index artifacts over non-ASCII corpora - including the completed
30% of E1 - no longer fingerprint-match, so affected corpora re-index
on the next run. ACTION FOR THE USER/ORCHESTRATOR: re-index every
affected corpus after this lands; the version bump is the automatic
global invalidation (the fingerprint chain hashes CONTENT, not analyzer
code, so nothing else self-invalidates). ASCII-path corpora need no
re-index for correctness (m00/m18 parity prove byte-identical ASCII
output before and after). The column-vs-line-length shard-JSON scan
remains a CANDIDATE detector for validating existing caches only - not
a commitment, not built here.

## Deviations and interpretation notes (complete list)

1. Commit route: direct `commit_scoped.py` instead of the handoff's
   distill-wrapped form (allowlist-refused this session-lineage;
   identical scopes/messages; T0b precedent 4ef8412).
2. Staging route: distill-wrapped `git add` (raw git denied);
   `encoding_guard --staged` ran on the explicitly staged touch-set
   before each commit.
3. Punct-test shape: U+00AB alone cannot panic (proved empirically by
   the approved T0b Gate 1 capture); the unit source combines it with
   the U+20AC panic driver - satisfying both clauses of the handoff's
   own test spec ("containing U+00AB at/near line start" AND
   "Pre-fix debug: panic"). Interpretation, not scope change.
4. ASCII baselines were transcribed from pre-fix actual streams during
   Gate-2 iterations (same command), then the final Gate-2 capture
   showed exactly 3 RED failures with the ASCII lock green.
5. Gate-1 exit code: raw `cargo test --release` failure exits 101, the
   distill digest reports `### cargo build FAIL (exit 101)` -
   non-zero as required (T0b recorded the same non-substantive
   wording gap).
6. Gate 6 full run cannot reach `0 failed` while the m21 witness
   (undeclared dependent, outside touch-set) stays pinned - reported
   per the touch-set rule with the failure line quoted verbatim; the
   rider components were proven in the scoped Gate-6 iteration above.
   The one-line `2.0.0` -> `2.1.0` update at
   m21_scala_semantic_index_test.rs:499 needs a reviewer/orchestrator
   ruling (same sanctioned-rider treatment as the m12 pin).
7. m12 comment rewrite replaced the two non-ASCII punctuation chars
   (arrow/em-dash) with ASCII `->`/`-` INSIDE the rewritten comment
   lines only (pure-ASCII authoring rule; every untouched line keeps
   its original bytes; encoding_guard --staged exit 0 on the result).
8. Journal follow-up commit after Gate 10 (journal-completion errata,
   T0b precedent) - all 10 declared gates unchanged.

## Deliberate non-actions

- parser.rs untouched (audit table: no second bug) - byte-identical in
  both commits.
- `tests/golden/**`, `tests/fixtures/**` (incl. the T0b specimen),
  `crates/pipeline|cli|ui|graph|scheduler`, `progress.rs`,
  `bench_baseline.json`, `common/mod.rs`: never opened.
- m21 witness NOT edited (outside touch-set - reported).
- No goldens regenerated, no UPDATE_GOLDENS, no `git stash`, no branch
  change; foreign orchestrator docs in the worktree never staged.
- No shard-envelope field change anywhere (DP-6: the version CONSTANT
  is the only cache-visible change - reviewer eyeballed the hunks).

