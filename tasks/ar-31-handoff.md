# AR-31 Handoff - Tokenizer allocation rework (P8 MED)

Wave 7 (background-capable, but same-crate serialization applies). Crate
(`lang-cpp`). Dispatch after AR-19, AR-20, AR-21 approved (lang-cpp
Cargo/lib churn settled); tokenizer.rs itself is untouched by those.

## Objective

`crates/lang-cpp/src/tokenizer.rs`: iterate `&[u8]`/slices instead of
materializing `Vec<char>` per TU; build tokens as `&str`/`StringWriter` with
capacity; single scan for the unterminated-raw-string EOF case. Preserve the
M27 byte-vs-char column underflow locks EXPLICITLY: locate both dated lock
files (grep `M27` under `crates/lang-cpp/tests` + `tasks/` dated rows) and
run them UNEDITED.

## Impact

Removes 4 B/char per-TU allocation + 2+ allocs/token flush. Highest-risk
MECHANICAL change: byte-parity locked per M27 precedent. Bench-neutral-or-
better stage timing required.

## Touch-set (hard)

- `crates/lang-cpp/src/tokenizer.rs` ONLY (~150-300 lines)
- new characterization suite under `crates/lang-cpp/tests/` (pre-change
  token-stream oracle on fixtures)
- `tasks/ar-31-handoff-journal.md`

## Constraints

- CHARACTERIZATION FIRST: capture token streams (kind, text, line, BYTE
  column) for every fixture TU pre-change; post-change byte-parity on all —
  this is the M27 lock discipline.
- The dated M27 underflow lock files run UNEDITED; journal their paths.
- Per-TU allocation-count drop measured via a temporary probe (journal
  numbers; probe deleted before commit).
- STRICT COMPILE-GREEN; pure ASCII; commit_scoped lang-cpp + journal.

## Gate commands

1. `-p ariadne-lang-cpp` (153/0/0/2 UNEDITED count + characterization suite
   counted separately — journal reconciliation)
2. m18 parity (23/0/0/1) + m00 golden + facts-path UNEDITED
3. `python tools/distill.py -- python tools/bench_compare.py` (stage/TU rows
   neutral-or-better, no >1.25x; clone-row near-ratio rows report-and-
   attribute, not yours)
4. `encoding_guard --all`; `hygiene_check.py` (zero-blocking baseline)

## Invariants / locks

- Token stream (kind/text/line/byte-col) byte-parity on all fixtures —
  characterization suite IS the lock.
- Unterminated raw string at EOF: same token + same span as today (dated
  lock green).
