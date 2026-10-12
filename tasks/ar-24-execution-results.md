# AR-24 Execution Results (reviewer acceptance pass)

Workspace state at adjudication: `checkpoint_guard status` -> HEAD `d4c608c`,
branch `shining-enthusiasm`, tracked tree clean (untracked `tasks/ar-*` docs
only), no other lane live. Reviewer runs are the authoritative fresh gates;
the journal's identical numbers were not relied on.

## Reviewer-dispatched gates (own stdout, canonical testrun lines verbatim)

| # | Command | Result line (verbatim) | Expected | Status |
|---|---|---|---|---|
| 1 | `python tools/testrun.py -p ariadne-lang-api` | `41 passed / 0 failed / 0 ignored / 4 suites` | 41/0/0/4 | MATCH |
| 2 | `python tools/testrun.py -p ariadne-lang` | `35 passed / 0 failed / 1 ignored / 2 suites` | 35/0/1/2 | MATCH (baseline exact) |
| 3 | `python tools/testrun.py -p ariadne-lang-cpp` | `155 passed / 0 failed / 0 ignored / 2 suites` | 155/0/0/2 | MATCH (153 base +1 byte-lock +1 kind) |
| 4 | `python tools/testrun.py -p ariadne-lang-scala` | `227 passed / 0 failed / 0 ignored / 2 suites` | 227/0/0/2 | MATCH (221 base +5 byte-locks +1 kind) |
| 5a | `... -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` | `23 passed / 0 failed / 0 ignored / 1 suites` | 23/0/0/1 | MATCH, file ZERO-diff |
| 5b | `... --test m17_plugin_seam_test` | `8 passed / 0 failed / 0 ignored / 1 suites` | 8/0/0/1 | MATCH (mock signature line only) |
| 5c | `... --test m24_language_identity_test` | `8 passed / 0 failed / 0 ignored / 1 suites` | 8/0/0/1 | MATCH (mock signature line only) |
| 5d | `... --test m16_traversal_test` | `2 passed / 0 failed / 0 ignored / 1 suites` | reconcile | TRUE m16 traversal census; file ZERO-diff |
| 5e | `... --test m16_clang_worker_test` | `3 passed / 0 failed / 0 ignored / 1 suites` | reconcile | TRUE m16 clang-worker census (AR-20a 2->3 shift confirmed current); file ZERO-diff |
| 5f | `... --test m00_golden_baseline_test --test m00_facts_path_parity_test` | `4 passed / 0 failed / 0 ignored / 2 suites` | 4/0/0/2 | MATCH, UNEDITED |
| 6a | `python tools/testrun.py -p ariadne-cli` | `114 passed / 0 failed / 8 ignored / 6 suites` | 114/0/8/6 | MATCH baseline exact (cli untouched) |
| 6b | `python tools/testrun.py -p ariadne-scheduler` | `26 passed / 0 failed / 0 ignored / 2 suites` | 26/0/0/2 | MATCH; AR-16 dispatch-contract locks green |
| 6c | `python tools/testrun.py -p ariadne-graph` | `129 passed / 0 failed / 0 ignored / 6 suites` | 129/0/0/6 | MATCH; builder_seam_tests green |
| 7a | `python tools/encoding_guard.py --all` | (no output - silent/clean) | silent | PASS |
| 7b | `python tools/hygiene_check.py` | zero R1/R2 rows; all printed rows R3 warn-only; none inside the AR-24 hunks (`graph/builder.rs` rows :574-669 vs hunk :452-464; `validate/lib.rs` row :104 vs hunk :798) | zero blocking | PASS |

Exit codes are not echoable in this session (chained `echo` denied by the
global bash rule); hygiene/encoding adjudicated on printed contracts per the
AR-01/02/03 precedent. The `UnicodeEncodeError` cp1252 tooling bug fired once
(`m16_traversal_test` run, traceback `tools/testrun.py:262`); re-run with
`PYTHONIOENCODING=utf-8` produced the canonical line quoted above.

## Workspace-wide acceptance gates (reviewer role; no implementer carries these)

| Gate | Result |
|---|---|
| `python tools/testrun.py` (bare workspace census; no other lane live) | `1393 passed / 0 failed / 13 ignored / 102 suites` |
| `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 219 tests in 19.908s` / `OK` |
| `python tools/hygiene_check.py` (workspace view) | zero blocking; R3 warns all pre-existing |
| `python tools/encoding_guard.py --all` | silent (clean) |
| `python tools/testrun.py --baseline` (as documented) | NOT RUNNABLE: `usage error: missing value for --baseline` (own run). `--baseline` takes a FILE (tools/testrun.py `OWN_VALUE_FLAGS`); glob `**/*baseline*` finds only `tools/theme_token_baseline.json` / `tools/bench_baseline.json` - no test baseline file exists. Standing environment errata (AR-01/02/03/06/07 verdicts); bare census above is the established substitute. |
| `python tools/ui_smoke.py` / `python tools/bench_compare.py` | NOT RUN - not required by the AR-24 contract (gate list 1-7) and the handoff touches zero UI/runtime-surface or hot-path code; no UI/bench behavior could shift. |

## Acceptance criteria (handoff) vs what actually happened

1. **Typed seam, no `Result<_, String>` trait methods left at the plugin
   seam** - MET. Pre-change tree grep: `validate_compile` was the only seam
   `Result<_, String>`; post-change: none (`error.rs` doc-comment mention is
   text). Non-seam String fns named and left to AR-25 per the decision table.
2. **Message text byte-stable through Display** - MET. All six lock literals
   re-derived byte-exact from `cefbd6f` blobs (C++ placeholder; `COMPILE_
   DEFERRAL` with continuation-whitespace applied; three
   `exited unsuccessfully` variants; vanished-toolchain); `#[error("{message}")]`
   passthrough on all variants; 155/227 suites green including all locks;
   mutation captures in the journal are arithmetically consistent (3 byte-lock
   red / kind-only red respectively); reviewer could not re-run mutations
   (read-only constraint) - judged from evidence.
3. **error-kind branching test (match on variant, not string)** - MET.
   `plugin_error_seam_test.rs` 4 tests (incl. anyhow downcast + std-error
   downcast), cpp/scala seam kind tests all green in runs 1/3/4.
4. **STRICT COMPILE-GREEN single commit** - MET. Whole workspace + all
   integration targets built and ran green this session.
5. **m24 identity / m18 parity wording locks UNEDITED (external proof)** -
   MET IN ESSENCE: m18 file-ZERO-diff green; m24 file carries only the
   E0053-forced mock signature line, every wording/assertion byte-untouched,
   green at pinned 8/0/0/1 (D2 adjudicated ACCEPTED on independent hunk review).
6. **Touch-set discipline + commit_scoped + journal** - MET. 18/18 paths
   inside touch-set ∪ declared riders; riders R1/R2 + D1/D2/D3 all declared
   and adjudicated (§3, §7 of verdict).
7. **encoding + hygiene** - MET (silent guard; zero blocking rows).

## Not run / not possible (explicit statement)

1. `testrun.py --baseline` in the documented bare form: not runnable (usage
   error quoted above; no baseline file in repo) - bare census substituted,
   line quoted.
2. `ui_smoke.py`, `bench_compare.py`: not required by AR-24's contract; zero
   UI / load-surface changes; not run.
3. Re-running the Display/kind mutations: reviewer is read-only and cannot
   mutate the judged tree; journal mutation captures adjudicated instead.
4. `python -c`, `git grep`, chained `echo` byte-verification conveniences:
   refused by this session's global deny rule; every check completed via
   `git show` blobs, grep/glob tools, and the sanctioned `tools/*.py` entry
   points. No gate was skipped.

## Ledger notes from this pass

- True current m16 census: traversal `2 passed / 0 failed / 0 ignored /
  1 suites`, clang-worker `3 passed / 0 failed / 0 ignored / 1 suites` (both
  files UNEDITED; the AR-20a pin `2` for clang-worker remains stale at the
  ledger bank until corrected).
- Workspace census anchor post-AR-24: `1393 passed / 0 failed / 13 ignored /
  102 suites`.
- testrun.py cp1252 bare-mode echo crash re-confirmed (verdict finding 2);
  AR-20a tooling row still owed.
