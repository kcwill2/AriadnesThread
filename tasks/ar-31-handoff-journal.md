# AR-31 handoff journal - Tokenizer allocation rework (P8 MED)

Handoff: `tasks/ar-31-handoff.md`. Implementer session `ses_ed4a4e326ffep9grAvw7MES6hW`.
Base HEAD at start: `b7136db528f06d262d68ddae6ca10c8b7679c0a3` (includes AR-25
cli seam rework and AR-21 resolver gate; verified via
`python tools/distill.py -- git rev-parse HEAD`).

## Mission (from handoff + orchestrator re-brief)

Rework `crates/lang-cpp/src/tokenizer.rs` from per-TU `Vec<char>`
materialization to byte-slice iteration with capacity-built tokens and a
single-scan unterminated-raw-string EOF path.
Load-bearing lock: token-stream byte-parity (kind/text/line/column, plus the
`RawToken.start/end` char-span fields consumed by the M23 verbatim locator).
CHARACTERIZATION FIRST: capture pre-change token streams for every fixture TU;
commit the characterization suite GREEN against the OLD tokenizer; refactor;
re-run the suite UNEDITED.

## Touch-set (hard)

- `crates/lang-cpp/src/tokenizer.rs` ONLY
- new characterization suite under `crates/lang-cpp/tests/`
- `tasks/ar-31-handoff-journal.md` (this file)

## M27 lock file paths (located per contract instruction)

The handoff's grep instruction (`M27` under `crates/lang-cpp/tests`) finds
nothing because lang-cpp has NO `tests/` directory; its tokenizer test locks
live as `#[cfg(test)] mod` files under `crates/lang-cpp/src/tokenizer/`.
The two dated M27 underflow lock files are:

1. `crates/lang-cpp/src/tokenizer/underflow_fix_tests.rs` - M27 T2
   underflow-fix locks (byte-vs-char column class at both flush sites;
   debug-panic + release-wrap falsifiable bound; ASCII baseline streams).
   Runs UNEDITED inside `-p ariadne-lang-cpp`.
2. `tests/integration/tests/m27_nonascii_corpus_test.rs` - M27 T0b corpus
   lock over frozen fixture `tests/fixtures/m27-nonascii-underflow/`
   (single-encoding byte lock + cold-index completion + wrapped-column
   detector). Runs UNEDITED (test-name-scoped; fixture untouched, read-only).

Dated ledger row: `tasks/milestone_ledger.md` line 33 (M27 row, 2026-10-05)
records the root cause "byte-vs-char column subtraction at both tokenizer
flush sites" and the fixture approval dated note.

## Baselines (re-briefed; contract's 153 is stale, +2 from AR-24 kind tests)

- `python tools/testrun.py --distill -- -p ariadne-lang-cpp` -> expect
  155 passed / 0 failed / 0 ignored / 2 suites PRE-change.
- m18 parity 23/0/0/1; m00 golden + facts-path 4/0/0/2; m17 cone guard
  8/0/0/1 - all UNEDITED.
- hygiene `python tools/hygiene_check.py` -> zero blocking rows.
- AR-21's default-off `test-resolver` feature does not touch tokenizer.rs;
  default-feature run must stay 155.

## Status log

- [x] Journal created (this commit).
- [x] Pre-change scoped baseline captured:
  `python tools/testrun.py --distill -- -p ariadne-lang-cpp` ->
  `155 passed / 0 failed / 0 ignored / 2 suites` (capture:
  target/testrun-capture.txt, 231 lines; matched re-briefed 155, contract's
  153 stale as briefed).
- [x] Characterization suite + 19 oracles generated GREEN vs OLD tokenizer;
  committed at `089a9d1` BEFORE any tokenizer logic change.
- [x] Allocation probe numbers captured (old 624 / new 458), probe deleted
  before commit (both runs; `git status` verified suite file byte-identical
  to the `089a9d1` blob before the refactor commit).
- [x] Tokenizer rework landed (commit `1140ed5`); suite re-run UNEDITED green.
- [x] m18/m00/m17 + both M27 locks UNEDITED green.
- [x] bench_compare neutral-or-better (`no regressions past 1.25x baseline`).
- [x] encoding_guard --all/--staged + hygiene zero-blocking + strict
  compile-green (all runs documented in the gates section).

## Characterization (pre-change) - suite GREEN against the OLD tokenizer

Suite module: `crates/lang-cpp/src/tokenizer/fixture_parity_tests.rs`
(deviation D1 for the location rationale), registered in `tokenizer.rs` as
`#[cfg(test)] mod fixture_parity_tests;`. Oracle data:
`crates/lang-cpp/tests/tokenizer_oracles/<fixture-relative-path>.txt` -
one line per token `W|text|line|column|start|end`, token text ASCII-escaped
(`\xHH`), covering all five parity fields including the M27 byte-vs-char
column axis and the `RawToken.start/end` char spans the M23 verbatim locator
slices by. 19 oracle files = every C++ fixture TU under `tests/fixtures`
(extensions cpp/h/hpp/cc/cxx/hh/hxx): basic (5), expensive (4),
m18-external-boundary (7), m27-nonascii-underflow (3 - the non-ASCII
specimens give the suite real byte-vs-char coverage).

Ordering discipline (the contract's core): a TEMPORARY `#[ignore]`d
generator test rendered the streams against the OLD tokenizer, the match
test passed, and the match test is committed UNEDITED through the refactor:
- generator run (old tokenizer): `1 passed / 0 failed / 0 ignored / 1 suites`
- match test alone (old tokenizer):
  `1 passed / 0 failed / 0 ignored / 1 suites`
- temporary generator + allocation probe then DELETED from the module; the
  committed module contains only fixture-walk + render + match-test code.

## Allocation probe (temporary; counting global allocator in the unit-test
binary; run `--exact --ignored --test-threads=1`; report at
`target/ar31-alloc-probe.txt`)

OLD tokenizer (pre-change, HEAD `b7136db`, alloc_ops = alloc+realloc+dealloc
calls inside `tokenize_raw` per TU):

| TU | bytes | tokens | alloc_ops |
|---|---|---|---|
| basic/include/heavy.h | 229 | 35 | 50 |
| basic/include/thing.h | 234 | 32 | 45 |
| basic/include/thing_types.h | 78 | 16 | 28 |
| basic/src/thing.cpp | 135 | 32 | 45 |
| basic/tests/consumer.cpp | 139 | 35 | 49 |
| expensive/include/heavy.h | 360 | 92 | 107 |
| expensive/include/thing.h | 120 | 22 | 35 |
| expensive/src/consumer1.cpp | 101 | 20 | 33 |
| expensive/src/consumer2.cpp | 145 | 23 | 36 |
| m18-external-boundary/ext_sdk/bridge_defs.h | 14 | 3 | 12 |
| m18-external-boundary/ext_sdk/sdk_bridge.h | 40 | 5 | 15 |
| m18-external-boundary/src/core.h | 38 | 5 | 15 |
| m18-external-boundary/src/core_defs.h | 14 | 3 | 12 |
| m18-external-boundary/src/main.cpp | 132 | 34 | 48 |
| m18-external-boundary/third_party/external_lib/api.h | 37 | 5 | 15 |
| m18-external-boundary/third_party/external_lib/api_impl.h | 14 | 3 | 12 |
| m27-nonascii-underflow/include/runs.h | 301 | 15 | 28 |
| m27-nonascii-underflow/src/punct_run.cpp | 488 | 10 | 21 |
| m27-nonascii-underflow/src/word_run.cpp | 502 | 7 | 18 |
| TOTAL (19 TUs) | | | **624** |

(New-tokenizer numbers appended below after the refactor; probe deleted
before either commit lands.)

## Deviations

- D1: characterization test module lives at
  `crates/lang-cpp/src/tokenizer/fixture_parity_tests.rs`, not
  `crates/lang-cpp/tests/` as the handoff's touch-set wording suggests.
  Reason: `tokenize`/`tokenize_raw`/`RawToken` are `pub(crate)` behind the
  private `mod tokenizer` in `lib.rs` (outside the touch-set) - an
  integration test under `tests/` cannot reach them without widening the
  crate API or editing `lib.rs`. The three existing tokenizer lock modules
  (`characterization_tests.rs`, `raw_string_tests.rs`,
  `underflow_fix_tests.rs`) live in exactly this directory; the oracle DATA
  follows the handoff wording under `crates/lang-cpp/tests/tokenizer_oracles/`.
- D2: contract gate #1 quotes `153/0/0/2`; the re-brief pins 155 (+2 AR-24
  kind tests) - 155 confirmed pre-change and governs.
- D3: the handoff's "~150-300 lines" tokenizer estimate undersizes a full
  mechanical rewrite: landed diff is 343 insertions / 221 deletions on
  tokenizer.rs (file 346 -> 467 lines, functions < 100, R1/R2 clean). The
  rework replaces every scan helper's char-slice indexing plus the main
  dispatch, so whole-file diff is inherent to the byte-parity rewrite, not
  scope creep (touch-set unchanged).
- D4: the characterization suite is one match test comparing 19 committed
  oracle files (not 19 separate tests); per-TU drift detail is reported
  inside the single assertion message. Suite counted separately from the
  155 baseline per AR-21 precedent.
- D5 (probe mechanics): per-TU allocation probe = counting `GlobalAlloc`
  wrapping `System` inside the unit-test binary, run alone via
  `--exact --ignored --test-threads=1`, report written to
  `target/ar31-alloc-probe.txt` (unique name - immune to the shared-capture
  clobber). Deleted before BOTH commits; `git status` after deletion showed
  only tokenizer.rs modified, proving the committed characterization file
  re-ran byte-identical (the UNEDITED requirement).

## Tokenizer rework (GREEN + REFACTOR) - commit `1140ed5`

What landed (tokenizer.rs ONLY):
- Main dispatch iterates a byte cursor over `&[u8]`/`&str` with lazy char
  decoding (`char_at` at guaranteed char boundaries); no `Vec<char>`
  materialization (the per-TU 4 B/char alloc is gone).
- Word runs are byte ranges with char-span bookkeeping; token text is one
  exact-size allocation from the source at flush (`source[range].to_owned()`),
  replacing per-char push + per-token `String::clone` (2+ allocs/token).
- Pending-punct buffer removed: provably empty between every iteration
  (the old push site was immediately followed by its own flush; all other
  flush sites were no-ops). Invariant documented at `tokenize_raw`.
- `scan_raw_string_with_fallback`: ONE pass over the raw region decides and
  positions BOTH locked outcomes (raw commit on first `)delim"` pattern
  hit, keeping the old priority that a terminator anywhere outranks earlier
  unescaped quotes; else the recorded first-unescaped-quote stop, else EOF)
  - the pre-rework EOF case scanned the remainder twice.
- Byte-level equivalence arguments (ASCII bytes never occur inside
  multi-byte sequences; classification still via decoded-char predicates)
  are documented at `raw_string_open` / `byte_at_eq` /
  `scan_raw_string_with_fallback`. Column accounting stays per-CHAR
  everywhere (M27 lock).

What deliberately did NOT happen:
- No parser.rs / resolver.rs / lib.rs touch; no dependency added (m17 cone
  green); no public API change (all `pub(crate)` shapes byte-identical);
  `test-resolver` feature untouched (default-feature run unaffected).
- NO live underflow/panic path was revealed by characterization or the
  m27 corpus lock - the known-history parser.rs:61 concern (memory
  2026-10-05) saw no new evidence here: every gate ran clean on all
  fixture TUs incl. the m27 non-ASCII specimens. Nothing to stop-and-report.

Post-change allocation probe (identical probe code re-added, run, deleted):
OLD TOTAL 624 -> NEW TOTAL 458 alloc-ops across the 19 fixture TUs
(-26.6%). Per-TU deltas: 50->40, 45->36, 28->19, 45->36, 49->40, 107->98,
35->26, 33->24, 36->27, 12->4, 15->7, 15->7, 12->4, 48->39, 15->7, 12->4,
28->18, 21->13, 18->9. Every TU strictly improved; smallest TUs (pure
Vec<char> + Vec growth overhead, near-zero tokens) drop ~3x.

## Gate lines (own stdout, verbatim; captures named per run)

Pre-change / characterization phase:
1. `python tools/testrun.py --distill -- -p ariadne-lang-cpp`
   -> `155 passed / 0 failed / 0 ignored / 2 suites` (pre-change baseline;
   capture 231 lines).
2. generator (temporary, old tokenizer):
   `python tools/testrun.py --distill -- -p ariadne-lang-cpp tokenizer::fixture_parity_tests::ar31_generate_fixture_oracles -- --exact --ignored --test-threads=1`
   -> `1 passed / 0 failed / 0 ignored / 1 suites`.
3. match test alone vs OLD tokenizer:
   `python tools/testrun.py --distill -- -p ariadne-lang-cpp tokenizer::fixture_parity_tests::tokenizer_fixture_token_streams_match_oracles -- --exact`
   -> `1 passed / 0 failed / 0 ignored / 1 suites`.
4. full scoped with suite vs OLD tokenizer (characterization commit state):
   -> `156 passed / 0 failed / 0 ignored / 2 suites` (= 155 UNEDITED + 1
   lock; reconciliation). Characterization commit `089a9d1` proven GREEN
   against the OLD tokenizer BEFORE the refactor landed (3-line mod-reg
   diff to tokenizer.rs in that commit, zero logic change).

Mutation test (non-vacuity of the new lock): byte-length flush subtraction
injected -> match test `0 passed / 1 failed / 0 ignored / 1 suites` with
`attempt to subtract with overflow` at the flush site - the exact M27 class;
mutation reverted, green restored.

Post-refactor (UNEDITED re-runs):
5. match test (suite file byte-identical to `089a9d1` blob):
   -> `1 passed / 0 failed / 0 ignored / 1 suites`.
6. `python tools/testrun.py --distill -- -p ariadne-lang-cpp`
   -> `156 passed / 0 failed / 0 ignored / 2 suites` (includes BOTH
   underflow_lock paths: `crates/lang-cpp/src/tokenizer/underflow_fix_tests.rs`
   and the m27 integration lock below).
7. M27 lock #2 UNEDITED:
   `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m27_nonascii_corpus_test`
   -> `2 passed / 0 failed / 0 ignored / 1 suites`.
8. m18 parity UNEDITED:
   `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m18_cpp_plugin_parity_test`
   -> `23 passed / 0 failed / 0 ignored / 1 suites`.
9. m00 pair UNEDITED:
   `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test`
   -> `4 passed / 0 failed / 0 ignored / 2 suites`.
10. m17 cone guard UNEDITED (no dependency added):
    `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m17_plugin_seam_test`
    -> `8 passed / 0 failed / 0 ignored / 1 suites`.
11. bench (LAST per re-brief): `python tools/distill.py -- python tools/bench_compare.py`
    -> `bench compare: 3 run(s), no regressions past 1.25x baseline`
    (exit 0; capture target/distill-capture.txt, 17 lines; host-load
    context: live AR-25 reviewer lane ran scoped gates concurrently - no
    row exceeded 1.25x so no re-run trigger fired).
12. strict compile-green at every commit: `python tools/distill.py -- cargo check --workspace`
    -> `ok (exit 0, 0 warnings emitted)` (run before `089a9d1` and before
    `1140ed5`).
13. hygiene: `python tools/hygiene_check.py` -> ZERO blocking rows (R1/R2
    empty; R3 warn-only list byte-equal to pre-dispatch except the OLD
    tokenizer's two own R3 rows (`i`, `c`) which the rework removed - the
    pin is "zero blocking", never an R3 count).
14. encoding: `python tools/encoding_guard.py --all` silent (pre-commit,
    AR-21 precedent); `python tools/encoding_guard.py --staged` silent
    post-commit (nothing staged after scoped commit - AR-20b D2 practice).
    Authored content pure ASCII: oracle token text is `\xHH`-escaped and
    every authored .rs is ASCII-only.

