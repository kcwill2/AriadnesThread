# AR-03 Execution Results - Bounded bincode decode (S5 MED)

Reviewer-run acceptance pass, 2026-10-09. Landing commit `de0d9e0`. Verdict:
**APPROVED** (see `tasks/ar-03-verdict.md`). All gate lines quoted verbatim;
every scoped gate re-run fresh in this session, nothing taken from the journal.

## Acceptance criteria vs evidence

| # | Criterion (handoff) | Result | Evidence line |
|---|---|---|---|
| 1 | Helper exists and every `bincode::deserialize` site in `report/src/manager/` routes through it | PASS | grep `bincode::deserialize` under `crates/report/src/manager/`: 2 matches, both doc comments in `bincode_options.rs:3,13`; zero call sites. 16 conversions in `git show de0d9e0` match journal per-file counts |
| 2 | Blob declaring huge collection length fails fast, not abort | PASS | fresh scoped run `47 passed / 0 failed / 0 ignored / 4 suites`; isolated run `6 passed / 0 failed / 0 ignored / 1 suites` incl. `load_report_blob_declaring_huge_string_length_errors_instead_of_aborting` (streaming path), `files_blob_declaring_huge_seq_length_...`, `files_blob_declaring_huge_string_length_...` |
| 3 | Honest reads byte-identical (unchanged) | PASS | same green run; honest roundtrip tests x2 green; all 16 `bincode::serialize*` writers untouched (grep); helper option chain proven wire-identical to the `bincode::deserialize` free function in bincode-1.3.3 lib.rs:177-185 + lib.rs:110-112 |
| 4 | Limit sized from observed payload sizes + margin; measurements recorded | PASS | 16 GiB; measurements recorded in journal + helper doc + test doc, sourced to `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` secs. 4/6 (~4096 MiB worst combined transient; warm report.bin ~0.9 MiB @ N=1280) => >=8x margin |
| 5 | Commit inside touch-set (hard) | PASS | `git diff --name-only de0d9e0^..de0d9e0` = 10 paths, all inside touch-set plus declared D1 mod line; no Cargo/manifest changes |
| 6 | TDD red-first with the predicted allocation-abort failure | PASS (record) | journal RED `5 passed / 1 failed / 0 ignored / 1 suites`, panic `capacity overflow` at `raw_vec/mod.rs:28`; mechanism independently verified in pinned source: bincode-1.3.3 de/read.rs:143-144 `temp_buffer.resize(length, 0)`; internal.rs:114 proves slice API drops limits to `Infinite` (reader path mandatory) |
| 7 | Scoped gate: `testrun.py --distill -- -p ariadne-report` | PASS | `### cargo test PASS (exit 0) [capture: ...\target\testrun-capture.txt, 73 lines]` / `47 passed / 0 failed / 0 ignored / 4 suites` |
| 8 | Encoding gate | PASS | `python tools/encoding_guard.py --all` - no output; tool exits 1 only on violations (encoding_guard.py:173), so empty = exit 0 clean |
| 9 | Hygiene: zero NEW violations | PASS | `python tools/hygiene_check.py` - only pre-existing `crates/ui/src/server.rs` R1/R2 (out of touch-set) + workspace-wide R3 warns; zero entries for `bincode_options.rs` / `bincode_bounded_decode_test.rs` |
| 10 | Report-evidence shape (distillation contract) | PASS | journal is digest-shaped: verbatim summary lines, distilling entry points used, capture path cited, no raw dumps |

## Workspace-wide acceptance pass (reviewer's own gates)

| Gate | Result |
|---|---|
| `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 219 tests in 19.307s` / `OK` |
| `python tools/testrun.py --baseline` | NOT RUN AS DOCUMENTED - `usage error: missing value for --baseline` (exit 2): `--baseline` takes a FILE in tools/testrun.py:60 and no test baseline file exists in the repo (verdict finding 3) |
| `python tools/testrun.py --distill` (workspace, nearest runnable substitute) | BLOCKED (exit 2 `parse format drift`) - foreign UNCOMMITTED cli edits change `run_ui` to 7 args while the clean test `m22_scala_load_analysis_test.rs:315` passes 6; HEAD blob of `run_ui` takes 6 (attribution proven). Not attributable to AR-03 (verdict finding 1) |
| `python tools/hygiene_check.py` | run; see criterion 9 |
| `python tools/encoding_guard.py --all` | run; see criterion 8 |
| `python tools/ui_smoke.py`, `python tools/bench_compare.py` | NOT RUN - not named in the AR-03 handoff gates; also unrunnable while the foreign dirty cli/ui work breaks the workspace build (verdict finding 1) |

## Explicit statements of what was not run / not verifiable

- Pre-commit `encoding_guard.py --staged` cannot be replayed after the commit;
  `--all` covers all tracked content as substitute.
- RED-phase reproduction requires checking out the pre-fix parent commit;
  write-git is denied for the reviewer. The predicted failure was instead
  verified at source level against the pinned bincode 1.3.3 crate.
- The workspace-wide `testrun` line could not be quoted because the shared
  worktree does not compile due to another session's uncommitted changes; the
  orchestrator must re-run it once that work lands. Quote when available:
  `<N> passed / <M> failed / <K> ignored / <S> suites`.
- Ground-truth note: `target/testrun-capture.txt` was overwritten by a
  concurrent process mid-review (shared-worktree capture race, verdict finding
  2); all quoted lines were parsed from each run's own stdout in-session.
