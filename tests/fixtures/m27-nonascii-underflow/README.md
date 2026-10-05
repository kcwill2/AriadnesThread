# m27-nonascii-underflow - approved E3 specimen corpus (M27 T0b)

Minimized regression specimen for the M27 `tu-index-hang-throughput`
tokenizer char/byte underflow class (field evidence E3: `attempt to
subtract with overflow` at `crates/lang-cpp/src/tokenizer.rs:331:21`).

Status: APPROVED via the documented frozen-set path only - a NEW sibling
directory, never inside `tests/fixtures/basic` or `tests/fixtures/expensive`
(byte-locked sets, see `tasks/milestone_ledger.md`). Landing requires the
reviewer-approved commit plus the dated ledger note; neither is this file.

## Contents

- `src/word_run.cpp` - a WORD run carrying multi-byte characters
  (U+00E9, UTF-8 bytes `C3 A9`). The leading identifier carries TWO `C3 A9`
  pairs at column 1: `flush_word`'s byte-length subtraction underflows the
  char-counted column (DEBUG panic at `tokenizer.rs:311`, the E3 class).
- `src/punct_run.cpp` - a PUNCT run carrying multi-byte characters. The
  line-leading U+20AC (UTF-8 bytes `E2 82 AC`) makes `flush_punct` subtract
  3 bytes from the char-counted column+1 (DEBUG panic at
  `tokenizer.rs:331:21` - the exact E3 site); the U+00AB guillemet
  (UTF-8 bytes `C2 AB`) between ASCII operators keeps the punct-run shape.
- `include/runs.h` - trivial pure-ASCII header included by both TUs.
- No compilation database: basic-style layout. The lock test injects a
  synthetic `compile_commands.json` into its TEMP COPY only, because
  marker-free repos shape zero C++ units.

## Byte policy - single-encoded UTF-8 ON PURPOSE

The `.cpp` data bytes are deliberately legitimate single-encoded UTF-8.
They are corpus content, not authored prose: do NOT "fix", normalize, or
extend these files in place. Any change re-runs the frozen-set approval
path (new sibling dir + reviewer-approved commit + dated ledger note).
The double-encoding signature (U+00C3 "A with tilde") must NEVER appear
here; `m27_nonascii_corpus_test.rs` locks that, plus the run-shape bytes.

## Locks

- Shape lock (green at landing and after T2):
  `tests/integration/tests/m27_nonascii_corpus_test.rs`
  :: `test_specimen_bytes_are_single_encoded_utf8`.
- Completion lock (lands RED, `#[ignore]`d; the T2 fix commit removes the
  `#[ignore]` line as its single-line rider, invariant I12):
  `test_m27_nonascii_corpus_cold_index_completes`.
