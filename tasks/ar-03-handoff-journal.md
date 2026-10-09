# AR-03 Journal - Bounded bincode decode (S5 MED)

Session: 2026-10-09, implementer, worktree `shining-enthusiasm`.
Handoff: `tasks/ar-03-handoff.md`.

## What landed

- NEW `crates/report/src/manager/bincode_options.rs`: `DECODE_BYTE_LIMIT`
  (16 GiB) + `deserialize_bounded(&[u8])` + `deserialize_bounded_reader(R: Read)`.
- All 16 `bincode::deserialize*` sites in `crates/report/src/manager/` now
  decode through the helper: `discovery_io.rs` (2), `graph_io.rs` (9),
  `index_io.rs` (1), `apply_io.rs` (1), `metrics_io.rs` (2, the streaming
  `report.bin` reader via `deserialize_bounded_reader`), `transform_io.rs` (1).
  Writers (`bincode::serialize*`) untouched: artifact bytes are unchanged.
- NEW regression suite `crates/report/tests/bincode_bounded_decode_test.rs`
  (6 tests): 3 hostile declared-length blobs (huge string length on the
  streaming path - the S5 abort proper; huge seq length and huge string
  length on a slice path), 2 honest roundtrips (files.bin, report.bin),
  1 size-bookkeeping test.
- `crates/report/src/manager.rs`: one-line `mod bincode_options;` (see
  deviation D1).

## Root cause found (why the naive fix shape is wrong)

- bincode 1.3.3: the slice API (`bincode::deserialize` / `Options::deserialize`)
  force-strips any configured limit to `Infinite` internally
  (`internal::deserialize_seed`), so limits only apply via the
  `Options::deserialize_from` reader path. The helper therefore decodes
  through `deserialize_from`, taking `&[u8]` as a `Read` too.
- The real abort site was `load_load_artifacts` (streaming `report.bin`):
  bincode's `IoReader::fill_buffer` resized its temp buffer to the blob
  declared string length BEFORE reading. An 8-byte blob declaring
  `u64::MAX` panicked with `capacity overflow` in `alloc::raw_vec`
  (pre-fix RED evidence below). Slice sites could not abort in-process with
  the pinned serde 1.0.229 (capped seq size-hints + slice-bounded reads),
  but were equally unlimited; both vectors are pinned by regression tests.

## Deviations (with reasons)

- D1 (touch-set): the new submodule must be registered with a
  `mod bincode_options;` line in `crates/report/src/manager.rs`, which the
  literal glob `crates/report/src/manager/*` does not cover. This is the
  module's declaration file; the edit is that single line and nothing else.
  Included as an extra `--scope` on the scoped commit.
- D2 (helper shape): the handoff sketch was
  `DefaultOptions::new().with_limit(limit)`. `DefaultOptions` defaults to
  VARINT int encoding, but every artifact writer here uses the
  `bincode::serialize` free function, which writes FIXINT lengths; the
  sketch chain would fail to decode every honest blob. The helper therefore
  uses `DefaultOptions::new().with_fixint_encoding().allow_trailing_bytes()
  .with_limit(DECODE_BYTE_LIMIT).deserialize_from(...)` - exactly the
  `bincode::deserialize` function configuration (fixint, trailing allowed)
  plus the limit, preserving the byte-identical honest-read invariant.
- D3 (limit semantics): `with_limit` in 1.3.3 gates bytes read and declared
  string/byte buffers, NOT the seq capacity hint (serde caps that since
  1.0.178; pinned here at 1.0.229). The regression suite pins both
  vectors, so a future serde regression cannot silently reopen the hole.
- D4 (commit tool): the handoff command passes one comma-joined scope
  string; `commit_scoped.py` takes repeatable `--scope` flags and would
  match nothing on a comma-joined glob, so scopes were passed as separate
  flags: `crates/report/src/manager/*`, `crates/report/tests/*`,
  `crates/report/src/manager.rs`, `tasks/ar-03-handoff-journal.md`.

## Measured payload sizes (limit sizing evidence)

- Honest fixture-scale blobs produced by the regression tests (measured via
  one-time `--nocapture` run, capture
  `target/testrun-capture.txt`): `repository/files.bin` = 208 bytes,
  `load/report.bin` = 36 bytes.
- Largest blob scale ever recorded on this project (ledger evidence
  `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md`): sec. 4 - the
  pre-DP-2 stage-5 save-span transient of ~4096 MiB for the JSON+bin
  report pair COMBINED (N=1280 files, both forms); sec. 6 - warm
  `report.bin` ~0.9 MiB at N=1280 files, deserialize delta ~4-5x blob
  size. Real-corpus blob sizes were NOT measured (DP-5), so worst-case
  honest blob is taken <= 2 GiB (half the combined transient bound).
- Limit choice: 16 GiB = at least 8x margin over the worst blob scale
  ever measured here, while ~10^9x below the smallest absurd declared
  length (`u64::MAX` ~ 1.8e19) - hostile blobs fail fast, honest reads
  stay untouched.

## TDD record (digests verbatim)

- RED: `python tools/testrun.py --distill -- -p ariadne-report --test bincode_bounded_decode_test`
  -> `5 passed / 1 failed / 0 ignored / 1 suites`
  (capture `target/testrun-capture.txt`). Expected semantic failure:
  `load_report_blob_declaring_huge_string_length_errors_instead_of_aborting`
  panicked `capacity overflow` at `library\alloc\src\raw_vec\mod.rs:28` -
  the declared-length allocation abort (S5) in the pre-fix code. The other
  5 tests passed as honest-read guards before the fix and after.
- GREEN (full scoped crate suite):
  `python tools/testrun.py --distill -- -p ariadne-report`
  -> `47 passed / 0 failed / 0 ignored / 4 suites`.
- Measurement run (temporary instrumentation, removed after):
  `6 passed / 0 failed / 0 ignored / 1 suites`.
- REFACTOR re-run (final, post-instrumentation-removal):
  `python tools/testrun.py --distill -- -p ariadne-report`
  -> `47 passed / 0 failed / 0 ignored / 4 suites`.
- Hygiene: `python tools/hygiene_check.py` - no R1/R2 entries for any file
  touched here; the two R1/R2 lines are pre-existing `crates/ui/src/
  server.rs` items outside this touch-set. R3 warn-only only, none in new
  files.

## Deliberately NOT done

- No writer/serializer changes; no artifact format change.
- No `Cargo.toml`/lockfile change (bincode `1.3` already workspace-pinned).
- No limit added outside `crates/report` (other crates are other handoffs).
- `apply_io.rs` keeps its silent JSON-fallback-on-bin-error semantics: a
  bounded decode error falls back exactly like a corrupt-blob error did.
- No exact fixture byte-size pins in tests (only upper-bound bookkeeping),
  so unrelated struct growth cannot break the suite.
- `ui_smoke`/`bench_compare` not run: not named in this handoff's gates.
