"""Tests for tools/testrun.py (canonical cargo test reporting, T4).

Run directly: python tools\\testrun_tests.py
Or discovered: python -m unittest discover -s tools -p "*_tests.py"
Everything here is OFFLINE: fixtures are canned cargo output written to temp
files and fed through --replay or the pure parser. Real cargo is never run.
"""

import contextlib
import io
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import testrun

TESTRUN_PATH = str(Path(__file__).resolve().parent / "testrun.py")

MULTI_SUITE_OUTPUT = """\
    Finished test [unoptimized + debuginfo] target(s) in 1.20s
     Running unittests src/lib.rs (target\\debug\\deps\\ariadne_lang-7c3d9a.exe)

running 6 tests
test graph::tests::empty_graph ... ok
test lexer::tests::tokenizes_input ... ok
test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/roundtrip.rs (target\\debug\\deps\\roundtrip-2f4e8b.exe)

running 4 tests
test parse_roundtrip ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests ariadne_lang

running 2 tests
test src/lib.rs - parse (line 8) ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
"""

FAILED_GROUP_OUTPUT = """\
    Finished test [unoptimized + debuginfo] target(s) in 0.40s
     Running unittests src/lib.rs (target\\debug\\deps\\ariadne_graph-1b2c3d.exe)

running 5 tests
test cycles::tests::detects_self_loop ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/edges.rs (target\\debug\\deps\\edges-9d8e7f.exe)

running 2 tests
test edge_verdicts ... FAILED
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
"""

GARBAGE_OUTPUT = """\
error: no such command: `tset`

\tDid you mean `test`?

Usage: cargo <COMMAND>
"""

DRIFT_SHAPED_OUTPUT = """\
     Running unittests src/lib.rs (target\\debug\\deps\\ariadne_x-0a1b.exe)
test result: ok. 3 passed, 0 failed, 0 ignored
"""

HEADERS_ONLY_OUTPUT = """\
     Running unittests src/lib.rs (target\\debug\\deps\\ariadne_x-0a1b.exe)
running 0 tests
   Doc-tests ariadne_x
running 0 tests
"""


ANSI_BOLD = "\x1b[1m"
ANSI_GREEN = "\x1b[32m"
ANSI_RESET = "\x1b[0m"


def wrap_ansi(line):
    return ANSI_BOLD + ANSI_GREEN + line + ANSI_RESET


ANSI_PASS_OUTPUT = "\n".join([
    "     Finished test [unoptimized + debuginfo] target(s) in 1.20s",
    wrap_ansi(
        "     Running unittests src/lib.rs"
        " (target/debug/deps/ariadne_lang-7c3d9a.exe)"
    ),
    "",
    "running 6 tests",
    "test graph::tests::empty_graph ... ok",
    wrap_ansi(
        "test result: ok. 6 passed; 0 failed; 1 ignored;"
        " 0 measured; 0 filtered out; finished in 0.01s"
    ),
    "",
    wrap_ansi(
        "     Running tests/roundtrip.rs"
        " (target/debug/deps/roundtrip-2f4e8b.exe)"
    ),
    "",
    "running 4 tests",
    "test parse_roundtrip ... ok",
    wrap_ansi(
        "test result: ok. 4 passed; 0 failed; 0 ignored;"
        " 0 measured; 0 filtered out; finished in 0.02s"
    ),
    "",
    wrap_ansi("   Doc-tests ariadne_lang"),
    "",
    "running 2 tests",
    "test src/lib.rs - parse (line 8) ... ok",
    wrap_ansi(
        "test result: ok. 2 passed; 0 failed; 0 ignored;"
        " 0 measured; 0 filtered out; finished in 0.30s"
    ),
]) + "\n"

ANSI_FAILED_OUTPUT = "\n".join([
    "     Finished test [unoptimized + debuginfo] target(s) in 0.40s",
    wrap_ansi(
        "     Running unittests src/lib.rs"
        " (target/debug/deps/ariadne_graph-1b2c3d.exe)"
    ),
    "",
    "running 5 tests",
    "test cycles::tests::detects_self_loop ... ok",
    wrap_ansi(
        "test result: ok. 5 passed; 0 failed; 0 ignored;"
        " 0 measured; 0 filtered out; finished in 0.01s"
    ),
    "",
    wrap_ansi(
        "     Running tests/edges.rs"
        " (target/debug/deps/edges-9d8e7f.exe)"
    ),
    "",
    "running 2 tests",
    "test edge_verdicts ... FAILED",
    wrap_ansi(
        "test result: FAILED. 1 passed; 1 failed; 0 ignored;"
        " 0 measured; 0 filtered out; finished in 0.05s"
    ),
]) + "\n"

ANSI_DRIFT_SHAPED_OUTPUT = "\n".join([
    wrap_ansi(
        "     Running unittests src/lib.rs"
        " (target/debug/deps/ariadne_x-0a1b.exe)"
    ),
    wrap_ansi("test result: ok. 3 passed, 0 failed, 0 ignored"),
]) + "\n"


def write_text_file(holder, name, content):
    target = Path(holder) / name
    target.write_text(content, encoding="utf-8")
    return str(target)


def run_tool(*args):
    finished = subprocess.run(
        [sys.executable, TESTRUN_PATH, *args],
        capture_output=True,
        text=True,
    )
    return finished.returncode, finished.stdout, finished.stderr


def last_stdout_line(standard_out):
    return standard_out.rstrip("\r\n").splitlines()[-1]


class ParserTests(unittest.TestCase):
    def test_multi_binary_and_doctest_groups_are_summed(self):
        counts = testrun.parse_cargo_output(MULTI_SUITE_OUTPUT)
        self.assertEqual((12, 0, 1, 3), tuple(counts))

    def test_failed_group_is_counted(self):
        counts = testrun.parse_cargo_output(FAILED_GROUP_OUTPUT)
        self.assertEqual((6, 1, 0, 2), tuple(counts))

    def test_garbage_output_parses_to_none(self):
        self.assertIsNone(testrun.parse_cargo_output(GARBAGE_OUTPUT))

    def test_result_line_off_the_pinned_shape_parses_to_none(self):
        self.assertIsNone(testrun.parse_cargo_output(DRIFT_SHAPED_OUTPUT))

    def test_headers_without_result_lines_parse_to_none(self):
        self.assertIsNone(testrun.parse_cargo_output(HEADERS_ONLY_OUTPUT))

    def test_colorized_pass_output_parses_counts_and_suites(self):
        counts = testrun.parse_cargo_output(ANSI_PASS_OUTPUT)
        self.assertIsNotNone(counts, "colorized output must not trip drift")
        self.assertEqual((12, 0, 1, 3), tuple(counts))

    def test_canonical_line_format(self):
        counts = testrun.TestCounts(12, 0, 1, 3)
        self.assertEqual(
            "12 passed / 0 failed / 1 ignored / 3 suites",
            testrun.canonical_line(counts),
        )


class ReplayCliTests(unittest.TestCase):
    def test_multi_suite_replay_prints_canonical_line_last(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "multi.txt", MULTI_SUITE_OUTPUT)
            returncode, standard_out, standard_err = run_tool(
                "--replay", fixture
            )
        self.assertEqual(0, returncode, standard_err)
        self.assertEqual(
            "12 passed / 0 failed / 1 ignored / 3 suites",
            last_stdout_line(standard_out),
        )

    def test_failed_group_replay_exits_one(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "failed.txt", FAILED_GROUP_OUTPUT)
            returncode, standard_out, standard_err = run_tool(
                "--replay", fixture
            )
        self.assertEqual(1, returncode, standard_err)
        self.assertEqual(
            "6 passed / 1 failed / 0 ignored / 2 suites",
            last_stdout_line(standard_out),
        )

    def test_garbage_replay_reports_drift_and_exits_two(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "garbage.txt", GARBAGE_OUTPUT)
            returncode, standard_out, standard_err = run_tool(
                "--replay", fixture
            )
        self.assertEqual(2, returncode)
        self.assertIn("parse format drift", standard_out + standard_err)

    def test_colorized_failed_output_parses_and_exits_one_via_replay(self):
        counts = testrun.parse_cargo_output(ANSI_FAILED_OUTPUT)
        self.assertIsNotNone(counts, "colorized output must not trip drift")
        self.assertEqual((6, 1, 0, 2), tuple(counts))
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(
                holder, "ansi_failed.txt", ANSI_FAILED_OUTPUT
            )
            returncode, standard_out, standard_err = run_tool(
                "--replay", fixture
            )
        self.assertEqual(1, returncode, standard_err)
        self.assertEqual(
            "6 passed / 1 failed / 0 ignored / 2 suites",
            last_stdout_line(standard_out),
        )

    def test_colorized_drift_shaped_output_still_exits_two_via_replay(self):
        self.assertIsNone(
            testrun.parse_cargo_output(ANSI_DRIFT_SHAPED_OUTPUT)
        )
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(
                holder, "ansi_drift.txt", ANSI_DRIFT_SHAPED_OUTPUT
            )
            returncode, standard_out, standard_err = run_tool(
                "--replay", fixture
            )
        self.assertEqual(2, returncode)
        self.assertIn("parse format drift", standard_out + standard_err)


class BaselineCliTests(unittest.TestCase):
    def test_matching_baseline_exits_zero(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "multi.txt", MULTI_SUITE_OUTPUT)
            baseline = write_text_file(holder, "base.txt", "12/0/1/3\n")
            returncode, standard_out, standard_err = run_tool(
                "--replay", fixture, "--baseline", baseline
            )
        self.assertEqual(0, returncode, standard_err)
        self.assertEqual(
            "12 passed / 0 failed / 1 ignored / 3 suites",
            last_stdout_line(standard_out),
        )

    def test_mismatching_baseline_prints_both_lines_and_exits_one(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "multi.txt", MULTI_SUITE_OUTPUT)
            baseline = write_text_file(holder, "base.txt", "11/0/1/3")
            returncode, standard_out, standard_err = run_tool(
                "--replay", fixture, "--baseline", baseline
            )
        self.assertEqual(1, returncode, standard_err)
        combined = standard_out + standard_err
        self.assertIn("11 passed / 0 failed / 1 ignored / 3 suites", combined)
        self.assertIn("12 passed / 0 failed / 1 ignored / 3 suites", combined)

    def test_missing_baseline_file_exits_two(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "multi.txt", MULTI_SUITE_OUTPUT)
            missing = str(Path(holder) / "no_such_baseline.txt")
            returncode, standard_out, standard_err = run_tool(
                "--replay", fixture, "--baseline", missing
            )
        self.assertEqual(2, returncode)
        self.assertTrue(standard_out + standard_err.strip())


class UsageErrorTests(unittest.TestCase):
    def test_unknown_flag_exits_two(self):
        returncode, standard_out, standard_err = run_tool("--bogus-flag")
        self.assertEqual(2, returncode)
        self.assertIn("unknown flag", standard_out + standard_err)

    def test_missing_flag_value_exits_two(self):
        returncode, standard_out, standard_err = run_tool("--timeout")
        self.assertEqual(2, returncode)
        self.assertTrue(standard_out + standard_err.strip())

    def test_non_positive_timeout_exits_two(self):
        returncode, standard_out, standard_err = run_tool("--timeout", "0")
        self.assertEqual(2, returncode)
        self.assertIn("timeout", standard_out + standard_err)

    def test_missing_replay_file_exits_two(self):
        with tempfile.TemporaryDirectory() as holder:
            missing = str(Path(holder) / "absent.txt")
            returncode, standard_out, standard_err = run_tool(
                "--replay", missing
            )
        self.assertEqual(2, returncode)
        self.assertTrue(standard_out + standard_err.strip())


class ScopedArgumentTests(unittest.TestCase):
    def test_default_runs_workspace_and_scoped_args_are_forwarded(self):
        recorded = []

        def fake_run(cargo_args, timeout_seconds):
            recorded.append((list(cargo_args), timeout_seconds))
            return testrun.RunOutcome(text=MULTI_SUITE_OUTPUT, timed_out=False)

        with mock.patch.object(testrun, "run_cargo_command", fake_run):
            with contextlib.redirect_stdout(io.StringIO()):
                default_code = testrun.main([])
                scoped_code = testrun.main(["-p", "ariadne-lang"])
        self.assertEqual((0, 0), (default_code, scoped_code))
        self.assertEqual((["--workspace"], 600), recorded[0])
        self.assertEqual((["-p", "ariadne-lang"], 600), recorded[1])


class TimeoutTests(unittest.TestCase):
    def _run_with_outcomes(self, argv, outcomes):
        remaining = list(outcomes)

        def fake_run(cargo_args, timeout_seconds):
            return remaining.pop(0)

        with mock.patch.object(testrun, "run_cargo_command", fake_run):
            captured = io.StringIO()
            with contextlib.redirect_stdout(captured):
                return_code = testrun.main(argv)
        return return_code, captured.getvalue(), len(remaining)

    def test_timeout_without_retry_exits_one(self):
        outcomes = [testrun.RunOutcome(text="", timed_out=True)]
        return_code, standard_out, unconsumed = self._run_with_outcomes(
            ["--timeout", "3"], outcomes
        )
        self.assertEqual(1, return_code)
        self.assertIn("TIMEOUT after 3s", standard_out)
        self.assertEqual(0, unconsumed)

    def test_timeout_with_retry_reruns_once_then_verdicts(self):
        outcomes = [
            testrun.RunOutcome(text="", timed_out=True),
            testrun.RunOutcome(text=MULTI_SUITE_OUTPUT, timed_out=False),
        ]
        return_code, standard_out, unconsumed = self._run_with_outcomes(
            ["--timeout", "5", "--retry-on-timeout"], outcomes
        )
        self.assertEqual(0, return_code)
        self.assertEqual(0, unconsumed)
        self.assertIn("TIMEOUT after 5s", standard_out)
        self.assertEqual(
            "12 passed / 0 failed / 1 ignored / 3 suites",
            last_stdout_line(standard_out),
        )

    def test_second_timeout_still_exits_one(self):
        outcomes = [
            testrun.RunOutcome(text="", timed_out=True),
            testrun.RunOutcome(text="", timed_out=True),
        ]
        return_code, standard_out, unconsumed = self._run_with_outcomes(
            ["--timeout", "4", "--retry-on-timeout"], outcomes
        )
        self.assertEqual(1, return_code)
        self.assertEqual(0, unconsumed)
        self.assertIn("TIMEOUT after 4s", standard_out)


FAILING_DETAIL_OUTPUT = """\
     Running unittests src/lib.rs (target\\debug\\deps\\ariadne_graph-1b2c3d.exe)

running 3 tests
test cycles::tests::detects_self_loop ... ok
test cycles::tests::ignores_dag ... FAILED

failures:

---- cycles::tests::ignores_dag stdout ----
thread 'cycles::tests::ignores_dag' panicked at src/cycles.rs:88:5:
assertion failed: !has_cycle

failures:
    cycles::tests::ignores_dag

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
"""


class DistillReplayCliTests(unittest.TestCase):
    def test_distill_pass_replay_omits_raw_lines_keeps_canonical_last(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "multi.txt", MULTI_SUITE_OUTPUT)
            capture = str(Path(holder) / "cap.txt")
            returncode, standard_out, standard_err = run_tool(
                "--distill", "--replay", fixture, "--capture", capture
            )
        self.assertEqual(0, returncode, standard_err)
        self.assertNotIn(
            "test graph::tests::empty_graph ... ok", standard_out
        )
        self.assertIn("### cargo test PASS (exit 0)", standard_out)
        self.assertIn("[capture: %s," % capture, standard_out)
        self.assertEqual(
            "12 passed / 0 failed / 1 ignored / 3 suites",
            last_stdout_line(standard_out),
        )

    def test_distill_fail_replay_excerpts_and_captures_full_text(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(
                holder, "failed_detail.txt", FAILING_DETAIL_OUTPUT
            )
            capture = str(Path(holder) / "cap.txt")
            returncode, standard_out, standard_err = run_tool(
                "--distill", "--replay", fixture, "--capture", capture
            )
            captured_text = Path(capture).read_text(encoding="utf-8")
        self.assertEqual(1, returncode, standard_err)
        self.assertIn("cycles::tests::ignores_dag", standard_out)
        self.assertIn("panicked at src/cycles.rs:88:5:", standard_out)
        self.assertEqual(FAILING_DETAIL_OUTPUT, captured_text)
        self.assertEqual(
            "1 passed / 1 failed / 0 ignored / 1 suites",
            last_stdout_line(standard_out),
        )

    def test_distill_drift_replay_reports_drift_and_exits_two(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(
                holder, "drift.txt", DRIFT_SHAPED_OUTPUT
            )
            returncode, standard_out, standard_err = run_tool(
                "--distill", "--replay", fixture,
                "--capture", str(Path(holder) / "cap.txt"),
            )
        self.assertEqual(2, returncode)
        self.assertIn("parse format drift", standard_out + standard_err)
        self.assertNotIn("PASS", standard_out)

    def test_distill_timeout_path_exits_one_unchanged(self):
        def fake_run(cargo_args, timeout_seconds):
            return testrun.RunOutcome(text="", timed_out=True)

        with tempfile.TemporaryDirectory() as holder:
            capture = Path(holder) / "cap.txt"
            with mock.patch.object(testrun, "run_cargo_command", fake_run):
                captured = io.StringIO()
                with contextlib.redirect_stdout(captured):
                    return_code = testrun.main(
                        ["--distill", "--timeout", "3",
                         "--capture", str(capture)]
                    )
            self.assertEqual(1, return_code)
            self.assertIn("TIMEOUT after 3s", captured.getvalue())
            self.assertFalse(capture.exists())

    def test_default_mode_stdout_stays_byte_identical_regression_lock(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "multi.txt", MULTI_SUITE_OUTPUT)
            captured = io.StringIO()
            with contextlib.redirect_stdout(captured):
                return_code = testrun.main(["--replay", fixture])
        self.assertEqual(0, return_code)
        self.assertEqual(
            MULTI_SUITE_OUTPUT
            + "12 passed / 0 failed / 1 ignored / 3 suites\n",
            captured.getvalue(),
        )


class DistillBaselineExitHonestyTests(unittest.TestCase):
    def test_distill_baseline_mismatch_header_agrees_with_process_exit(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "multi.txt", MULTI_SUITE_OUTPUT)
            baseline = write_text_file(holder, "base.txt", "11/0/1/3")
            capture = str(Path(holder) / "cap.txt")
            returncode, standard_out, standard_err = run_tool(
                "--distill", "--replay", fixture, "--baseline", baseline,
                "--capture", capture,
            )
        self.assertEqual(1, returncode, standard_err)
        headers = [
            line for line in standard_out.splitlines()
            if line.startswith("### cargo test")
        ]
        self.assertEqual(1, len(headers), standard_out)
        self.assertRegex(headers[0], r"^### cargo test \w+ \(exit 1\)")


if __name__ == "__main__":
    unittest.main()
