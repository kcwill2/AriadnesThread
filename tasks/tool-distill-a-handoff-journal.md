# Handoff A journal - tool output distillation (base 363c3a8)

## M1 distilled.py + distilled_tests.py
- Red: `python tools/distilled_tests.py` -> ModuleNotFoundError: No module named 'distilled' (all 10 tests red at import; no stub used). Green: `Ran 10 tests in 0.009s OK`.
- Commit: 4227407. Deviation: digest_cargo_test/digest_cargo_build/digest_generic accept optional trailing `capture_path=None` kwarg -- the contract requires capture path inside emitted lines but the pinned signatures carry no path param; omitted when absent (pure-mode callers). Verdict strings PASS/FAIL/DRIFT on `###` header lines.
- Findings: counts line reuses `testrun.canonical_line` (grammar stays owned by testrun); distilled imports testrun module-level, testrun imports distilled at module level; the distilled<->testrun import cycle is safe because every cross-module attribute is touched only at call time.

## M2 testrun --distill + testrun_tests additions
- Red: `python tools/testrun_tests.py` -> `Ran 25 ... FAILED (failures=3, errors=1)`; red names: DistillReplayCliTests {pass_replay, fail_replay, drift_replay, timeout}. Test 15 (default-mode byte-identity lock) passes pre-change by design -- it locks existing behavior, not new. Green: `Ran 25 tests in 0.558s OK`.
- Commit: 2b0e8ef. Deviation: `--capture PATH` added to OWN_VALUE_FLAGS -- required by the "via --replay + tmp capture" test design; default capture `target/testrun-capture.txt` unchanged per contract.
- Live smoke: `--distill -p ariadne-lang` -> `7 passed / 0 failed / 1 ignored / 2 suites` last line, digest 3 lines, exit 0.
- Fixup: this journal itself shipped with em-dash glyphs in M1/M2 commits; rewritten pure ASCII here per I6.

## M3 distill.py + distill_tests.py
- Red: `python tools/distill_tests.py` -> ModuleNotFoundError: No module named 'distill' (all 8 tests red at import; no stub). First green pass `FAILED (failures=1)`: build code emitted `error[0308]` (regex group dropped the `E` prefix); fixed BUILD_ERROR_CODE in distilled.py. Final: `Ran 8 tests in 0.275s OK`.
- Commit: (this commit). Gates: `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 176 tests OK`; live `python tools/distill.py cargo check` -> `ok (exit 0, 0 warnings emitted) -> capture: target/distill-capture.txt`, exit 0; hygiene_check exit 0 (R3 `i` warn fixed in tests).
- Notes: no new deviations. `--kind auto` with `--input-file` resolves to generic (no child token to inspect); `--child-exit` alongside a live child is ignored in favor of the live exit code.
