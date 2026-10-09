# AR-05 Journal - Fingerprint sentinel on serialize failure (T2 MED)

Date: 2026-10-09. Implementer run under strict RGB-TDD.

## What was done

### Red

- Extracted the context-hashing step of `TuCacheFingerprint::compute` into a
  private seam `context_hash_of(context, context_description)`, first keeping
  the exact pre-fix behavior (`bincode::serialize(...).unwrap_or_default()`)
  so the regression tests could target the failure branch directly.
- Added a test-only `InjectedUnserializableContext` whose `Serialize` impl
  always returns `Err`, injecting the failure branch without a feature flag
  (permitted by the handoff: "inject a non-serializable context via a test
  type").
- Added four regression tests in `#[cfg(test)] mod tests` in `cache.rs`:
  - `serialize_failure_never_hashes_empty_bytes` - failing context must not
    hash to `SHA-256("")` (`e3b0c442...`), the old shared fallback digest.
  - `serialize_failure_hash_is_not_a_shared_constant` - two failing contexts
    with different descriptions must not share one digest.
  - `serialize_failure_hash_differs_from_honest_hash` - failure digest must
    differ from any honest digest with the same description.
  - `honest_context_digest_is_byte_identical_to_direct_bincode_hash` -
    invariant lock: honest `context_hash` equals `SHA-256(bincode bytes)` and
    flows unchanged into `compute`.
- Red digest (verbatim): `8 passed / 2 failed / 0 ignored / 1 suites`
  (capture: `target/testrun-capture.txt`). The two failures were exactly the
  predicted semantic reason: both failing hashes equalled the empty-bytes
  digest. The third failing-behavior test passed trivially under Red (an
  honest digest is never the empty digest); it is kept as an invariant
  assertion.

### Green

- `context_hash_of` now branches on the serialization result:
  - Ok: SHA-256 of the exact bincode bytes (unchanged honest path).
  - Err: a salted sentinel hash over a failure marker constant
    (`CONTEXT_SERIALIZE_FAILURE_MARKER`), a process-unique salt, the context
    description, and the serializer error message; plus an `eprintln!`
    warning naming the context.
- The salt is `OnceLock<u128>` from launch nanoseconds XOR process-id tag:
  fixed within a process, different on every process launch. A persisted
  entry written under a failing fingerprint therefore cannot validate in any
  later process - worst case is cache miss + warning, never a false hit
  (the handoff's stated worst case).
- The sentinel is "recorded into the entry" through `context_hash`, which is
  serialized inside `TuCacheFingerprint` inside `TuCacheEntry` alongside
  `combined_hash`; no new struct field, no bincode/JSON layout change, no
  epoch bump needed (failing-context digests are new values, honest digests
  are untouched).
- Green digest (verbatim): `10 passed / 0 failed / 0 ignored / 1 suites`.

### Refactor

- The sentinel logic fits in one small function plus one small salt helper;
  no duplication found, no further refactor. Re-ran scoped tests.
- Gate 1 (verbatim): `python tools/testrun.py --distill -- -p ariadne-index`
  -> `15 passed / 0 failed / 0 ignored / 2 suites`.

## What was deliberately NOT done

- `compute` public signature unchanged (call sites in
  `crates/scheduler/src/tu_store.rs` and integration tests untouched).
- `config_str.map(...).unwrap_or_default()` at the config_hash step was
  reviewed as instructed and left alone: `None` is an honest "no config
  provided" absence, not a failure path, and `Some("")` hashes to the
  non-empty SHA-256 digest, so it is already distinguishable from `None`.
  Changing the `None` encoding would alter honest digests and violate the
  byte-identical invariant.
- No `Result` return type (handoff allows sentinel; preserving the return
  type avoids touching out-of-scope call sites).
- No new crate dependency (salt uses std only; `log` not in deps, so the
  warning follows the existing `eprintln!` precedent from
  `crates/build/src/discovery.rs` and `crates/pipeline/src/stage_load.rs`).

## Deviations

- None. Touch-set respected: `crates/index/src/cache.rs` and this journal.
  The regression test lives in the in-file `#[cfg(test)] mod` (an allowed
  touch-set option) because the injection seam is private.

## Known bound (documented, accepted)

Two different contexts with the *same* name, *same* error, failing within
the *same* process could still share a failure digest. Cache entries are
already keyed by `(FileId, context_name)`, so this is no wider than the
existing entry key granularity, and cross-process false hits (the real
audit risk) are eliminated by the salt.
