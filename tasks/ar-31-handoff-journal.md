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
- [x] Tokenizer rework landed; suite re-run UNEDITED green.
- [x] m18/m00/m17 + both M27 locks UNEDITED green.
- [ ] bench_compare neutral-or-better (LAST per re-brief - live AR-25
  reviewer lane may load the host).
- [x] encoding_guard --all/--staged + hygiene zero-blocking + strict
  compile-green (pre-refactor-commit run; re-run at closeout below).

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

