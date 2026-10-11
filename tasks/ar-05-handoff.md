# AR-05 Handoff - Fingerprint sentinel on serialize failure (T2 MED)

Wave 1 / Correctness. Independent crate (`index`).

## Objective

In `cache.rs`, replace `unwrap_or_default()` on `bincode::serialize(context)`
with an explicit error OR a random-process-sentinel mix-in recorded into the entry
so failing contexts can never hash to a shared empty-bytes digest.
Add a regression test for serialize-failure behavior (inject a non-serializable context
via a test type or feature seam).

## Impact

- Eliminates a silent false-cache-hit class (stale analysis served under a wrong context).
- Worst case after fix: cache miss + error/warn (slower but correct).

## Touch-set (hard)

- `crates/index/src/cache.rs` — fix `unwrap_or_default()` at lines 69-70 and 76-77
- `crates/index/tests/**` or `#[cfg(test)] mod` — regression test
- `tasks/ar-05-handoff-journal.md` — checkpoint journal

## Constraints

- TDD: red-first. Add failing test for serialize-failure behavior before fix.
- Pure ASCII only.
- Commit via `python tools/commit_scoped.py --scope <glob> -m "<msg>"`.
- Gates via distilling entry points only.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-index` (scoped index tests)
2. `python tools/encoding_guard.py --staged`
3. `python tools/hygiene_check.py`

## Invariants / locks

- Context serialization failure must NOT produce a hash of empty bytes.
- A sentinel or explicit error prevents false cache hits.
- Honest contexts: digest byte-identical to current behavior.
- The `context_hash` must differ when context serialization fails vs. succeeds.
