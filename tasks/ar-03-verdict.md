# AR-03 Verdict - Bounded bincode decode (S5 MED)

**APPROVED**

Review time: 2026-10-09T21:18Z+. Reviewer: fresh session, review-gate skill.
Landing commit: `de0d9e0` (single commit; parent `6d7f96b`). HEAD at review:
`932041a` on branch `shining-enthusiasm`.

## Gate transcripts (verbatim)

| Gate | Result |
|---|---|
| `python tools/testrun.py --distill -- -p ariadne-report` | `### cargo test PASS (exit 0)` ... `47 passed / 0 failed / 0 ignored / 4 suites` |
| `python tools/testrun.py --distill -- -p ariadne-report --test bincode_bounded_decode_test` | `6 passed / 0 failed / 0 ignored / 1 suites` |
| `python tools/encoding_guard.py --all` | no output; per tool contract (`return 1 if violations else 0`, tools/encoding_guard.py:173) empty output = exit 0 clean |
| `python tools/hygiene_check.py` | 2 R1/R2 entries, both pre-existing `crates/ui/src/server.rs` (out of touch-set); zero entries for any AR-03 file |
| `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 219 tests in 19.307s` / `OK` |
| `python tools/checkpoint_guard.py status` | ran first; HEAD/branch/dirty/worktree census recorded below |

## Commit-vs-touch-set: EXACT PASS

`git diff --name-only de0d9e0^..de0d9e0` returned 10 paths; every one is inside
the declared touch-set plus deviation D1 (`crates/report/src/manager.rs`, the
one-line `mod bincode_options;` registration). No path outside the touch-set.
No Cargo.toml/Cargo.lock changes (journal claim verified against the diff).

## Per-commit review (oldest-first; single commit)

`de0d9e0 fix(report): bounded bincode decode prevents allocation abort (S5)`:

- `bincode_options.rs` (NEW, 57 lines): `DECODE_BYTE_LIMIT = 16 GiB`,
  `deserialize_bounded(&[u8])`, `deserialize_bounded_reader(R: Read)`. Option
  chain `with_fixint_encoding().allow_trailing_bytes().with_limit(...)` over
  `Options::deserialize_from`. All functions well under hygiene limits.
- 16 call sites converted: discovery_io 2, graph_io 9, index_io 1, apply_io 1,
  metrics_io 2 (streaming `report.bin` via `_reader`), transform_io 1. Matches
  the journal's per-file count exactly.
- Grep-proof: zero remaining `bincode::deserialize`/`_from` CALL sites under
  `crates/report/src/manager/`; the only 2 textual matches are doc comments in
  the new helper. All 16 `bincode::serialize*` writers untouched (artifact
  bytes unchanged - honest-write byte-identity holds trivially).
- Regression suite `crates/report/tests/bincode_bounded_decode_test.rs` (NEW,
  6 tests): 3 hostile declared-length blobs (streaming-path `u64::MAX` string
  length = the S5 abort proper; slice-path huge seq length; slice-path huge
  string length), 2 honest roundtrips, 1 size-bookkeeping test. All green.

## Independent deviation adjudication

- **D1 (manager.rs mod line) - ACCEPTED.** Rust has no alternative: a
  submodule requires registration in its parent declaration file. The edit is
  exactly the one declared line; diff confirms nothing else in manager.rs.
- **D2 (fixint + allow_trailing_bytes instead of the handoff's varint
  sketch) - ACCEPTED, and the sketch would have been broken.** Verified in
  the pinned source, not from the journal: bincode-1.3.3 lib.rs:110-112 shows
  `bincode::serialize` = `DefaultOptions + with_fixint_encoding +
  allow_trailing_bytes` (all 16 writers here use these free functions), so
  the handoff's bare `options().with_limit(...)` (varint) would fail every
  honest blob. The helper's chain is the sketch's intent with the exact
  wire-compatible configuration plus the limit - byte-identity invariant
  preserved by construction and pinned by the 2 honest roundtrip tests.
- **D3 (`Options::deserialize_from` path is mandatory; slice API force-drops
  limits) - ACCEPTED, source-verified.** bincode-1.3.3 internal.rs:114
  (`deserialize_seed`): `let options =
  ::config::WithOtherLimit::new(options, Infinite);` - the slice API discards
  any configured limit, so the helper's reader path is required, not
  stylistic. The abort mechanism claimed is likewise real: de/read.rs:143-144
  `IoReader::fill_buffer` does `self.temp_buffer.resize(length, 0)` with the
  blob-declared length before reading. serde pinned 1.0.229 (Cargo.lock:1603),
  matching the journal; both failure vectors pinned by regression tests.
- **D4 (repeatable `--scope` flags) - ACCEPTED.** commit_scoped.py:129
  `--scope action="append"`; a comma-joined glob string resolves to nothing
  under its fnmatch semantics. Commit content is exactly scope-respecting,
  which is the substantive check.

## TDD evidence

Journal RED: `5 passed / 1 failed / 0 ignored / 1 suites` with
`load_report_blob_declaring_huge_string_length_errors_instead_of_aborting`
panicking `capacity overflow` in `alloc::raw_vec` - the predicted
declared-length allocation abort, and exactly the failure mode independently
confirmed above in the pinned bincode source. GREEN and final re-run lines in
the journal match my fresh runs verbatim. RED cannot be re-executed from a
read-only review position (no write-git); the mechanism verification and the
self-consistent digest record are the available substitute, recorded here for
transparency.

## Limit sizing

16 GiB = >= 8x margin over the worst honest blob scale ever measured on this
project (~4096 MiB combined transient, t0a evidence secs. 4/6, cited in both
the helper doc and the test file) and ~9 orders of magnitude below the
smallest hostile declaration. Constraint "sized from observed payload sizes +
margin" MET with recorded measurements.

## Findings (non-blocking; none affect the merge)

1. **[environment, minor] Workspace acceptance gate blocked by foreign
   in-flight work in the shared worktree.** `python tools/testrun.py
   --distill` (workspace) exits 2 `parse format drift` because
   `ariadne-integration-tests` fails to compile:
   `tests\integration\tests\m22_scala_load_analysis_test.rs:315` calls
   `run_ui(...)` with 6 args while the DIRTY (uncommitted)
   `crates/cli/src/commands/ui.rs` takes 7. Attribution proven: the HEAD blob
   of `run_ui` takes 6 args (matches the clean test); the 7th parameter exists
   only in another session's working-tree edits. Not AR-03's defect - AR-03
   touches neither cli, ui, nor integration tests. The orchestrator must
   re-run the workspace gate once the in-flight cli/ui work lands or moves to
   its own worktree.
2. **[environment, minor] Shared capture race.**
   `target/testrun-capture.txt` was overwritten mid-review by a concurrent
   process (it held an `ariadne_index` run after my 73-line report capture;
   T2-style tests were running concurrently in the same worktree). Canonical
   lines above were parsed from each run's own stdout, so results stand, but
   capture-file evidence in a shared worktree is not durable. Suggest a
   per-run capture filename suffix or ledger note.
3. **[tooling docs, minor] `testrun.py --baseline` requires a FILE**
   (`OWN_VALUE_FLAGS`, tools/testrun.py:60); no baseline file exists in the
   repo (glob for `**/*baseline*` found only theme/bench baselines), so the
   documented bare `--baseline` invocation exits 2: `usage error: missing
   value for --baseline`. A plain workspace `testrun.py` run is the only
   runnable form until a test baseline file is committed.
4. **[observation] `honest_load_report_blob_roundtrips_through_bounded_decode`
   hand-builds bytes in `PersistedLoadReport` derive order** - coupling is
   documented in the test and is the point of the lock, but a field reorder
   will fail this test with a decode-mismatch, not a compile error. Acceptable
   as-is.

## Deliberately not run

- `ui_smoke.py` / `bench_compare.py`: not named in the AR-03 handoff gates
  (journal declares this), and currently unrunnable regardless - the foreign
  dirty cli/ui edits break the build (finding 1).
- RED reproduction: requires checking out `6d7f96b` (write-git denied).
- `encoding_guard.py --staged` as a pre-commit step: not replayable after the
  commit; `--all` covers all tracked content instead.

## Hygiene baseline note

`hygiene_check.py` exits nonzero workspace-wide today due to pre-existing
`crates/ui/src/server.rs` R2 (1155 lines > 1000) and R1
(`api_graph_traversal` 161 lines) - both outside AR-03's touch-set and
unchanged by it. **Zero NEW violations were introduced by AR-03**: no R1/R2
and no R3 entries for `bincode_options.rs` or `bincode_bounded_decode_test.rs`.

## Follow-up dispositions

- Findings 1-2: route to `tasks/milestone_ledger.md` deferred-findings (shared
  worktree gate-blockage and capture-collision), or resolve when the in-flight
  cli/ui session lands its work.
- Finding 3: tooling-doc correction candidate (workflow docs say `--baseline`
  bare; tool wants a file).
