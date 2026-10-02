"""Tests for tools/distill.py (build/clippy/generic distilling CLI).

Run directly: python tools\\distill_tests.py
Or discovered: python -m unittest discover -s tools -p "*_tests.py"
Everything here is OFFLINE: canned tool outputs fed through the
--input-file/--child-exit seam; the timeout path uses an injected fake.
"""

import contextlib
import io
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import distill

DISTILL_PATH = str(Path(__file__).resolve().parent / "distill.py")

BUILD_ERROR = """\
    Checking ariadne_x v0.1.0 (C:\\repo)
error[E0308]: mismatched types
  --> src/cycles.rs:88:5
   |
88 |     let width: u32 = detect(root);
   |              ^^^   ------------- expected `u32`, found `usize`
   |
   = note: expected type `u32`
              found type `usize`

warning: build failed, waiting for other jobs to finish...
"""

WARNINGS_ONLY = """\
   Compiling ariadne_x v0.1.0 (C:\\repo)
warning: unused variable: `x`
 --> src/lib.rs:3:9
  |
  = note: `#[warn(unused_variables)]` on by default

warning: `ariadne_x` (lib) generated 15 warnings
    Finished dev [unoptimized + debuginfo] target(s) in 0.50s
"""

UNITTEST_MIX = """\
======================================================================
FAIL: test_alpha (tests.demo.DemoTest)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tests\\demo.py", line 12, in test_alpha
    self.assertEqual(1, 2)
AssertionError: 1 != 2
----------------------------------------------------------------------
Ran 1 tests in 0.001s

FAILED (failures=1)
"""

NOISE = """\
starting something
processing 1
processing 2
nothing recognizable here
it just ends
"""

OK_LINE = re.compile(r"^ok \(exit 0, (\d+) warnings emitted\) -> capture: .+$")


def write_text_file(holder, name, content):
    target = Path(holder) / name
    target.write_text(content, encoding="utf-8")
    return str(target)


def run_tool(*args):
    finished = subprocess.run(
        [sys.executable, DISTILL_PATH, *args],
        capture_output=True,
        text=True,
    )
    return finished.returncode, finished.stdout, finished.stderr


def fifteen_errors():
    blocks = []
    for index in range(15):
        blocks.append(
            "error[E0308]: mismatched types #%d\n"
            "  --> src/f.rs:%d:5\n"
            "   |\n"
            "   = note: note %d\n\n" % (index, 10 + index, index)
        )
    return "".join(blocks)


class BuildKindTests(unittest.TestCase):
    def test_error_fixture_excerpts_code_message_location(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "err.txt", BUILD_ERROR)
            capture = str(Path(holder) / "cap.txt")
            returncode, standard_out, standard_err = run_tool(
                "--kind", "build", "--input-file", fixture,
                "--child-exit", "1", "--capture", capture,
            )
        self.assertEqual(1, returncode, standard_err)
        self.assertIn("error[E0308]", standard_out)
        self.assertIn("mismatched types", standard_out)
        self.assertIn("--> src/cycles.rs:88:5", standard_out)
        self.assertIn("### cargo build FAIL (exit 1)", standard_out)

    def test_warning_only_clean_build_is_one_ok_line(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "warn.txt", WARNINGS_ONLY)
            capture = str(Path(holder) / "cap.txt")
            returncode, standard_out, standard_err = run_tool(
                "--kind", "build", "--input-file", fixture,
                "--child-exit", "0", "--capture", capture,
            )
        self.assertEqual(0, returncode, standard_err)
        body = standard_out.rstrip("\r\n").splitlines()
        self.assertEqual(1, len(body), standard_out)
        self.assertRegex(body[0], OK_LINE)
        self.assertIn("15 warnings", body[0])

    def test_fifteen_errors_capped_at_ten_blocks_with_more_line(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "many.txt", fifteen_errors())
            capture = str(Path(holder) / "cap.txt")
            returncode, standard_out, standard_err = run_tool(
                "--kind", "build", "--input-file", fixture,
                "--child-exit", "1", "--capture", capture,
            )
        self.assertEqual(1, returncode, standard_err)
        shown = [line for line in standard_out.splitlines()
                 if line.startswith("error[")]
        self.assertEqual(10, len(shown))
        self.assertIn("... and 5 more errors", standard_out)


class GenericKindTests(unittest.TestCase):
    def test_generic_kind_delegates_with_unittest_names(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "ut.txt", UNITTEST_MIX)
            capture = str(Path(holder) / "cap.txt")
            returncode, standard_out, standard_err = run_tool(
                "--kind", "generic", "--input-file", fixture,
                "--child-exit", "1", "--capture", capture,
            )
        self.assertEqual(1, returncode, standard_err)
        self.assertIn("FAIL: test_alpha (tests.demo.DemoTest)", standard_out)
        self.assertIn("capture: %s" % capture, standard_out)

    def test_fail_exit_without_recognizable_text_verdicts_fail(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "noise.txt", NOISE)
            capture = str(Path(holder) / "cap.txt")
            returncode, standard_out, standard_err = run_tool(
                "--kind", "generic", "--input-file", fixture,
                "--child-exit", "3", "--capture", capture,
            )
        self.assertEqual(1, returncode, standard_err)
        self.assertIn("### generic FAIL (exit 3)", standard_out)
        self.assertNotIn("PASS", standard_out)


class UsageAndTimeoutTests(unittest.TestCase):
    def test_no_child_and_no_input_file_exits_two(self):
        returncode, standard_out, standard_err = run_tool("--kind", "build")
        self.assertEqual(2, returncode)
        self.assertIn("usage", standard_out + standard_err)

    def test_missing_child_exit_value_exits_two(self):
        with tempfile.TemporaryDirectory() as holder:
            fixture = write_text_file(holder, "in.txt", BUILD_ERROR)
            returncode, standard_out, standard_err = run_tool(
                "--kind", "build", "--input-file", fixture, "--child-exit"
            )
        self.assertEqual(2, returncode)
        self.assertIn("usage", standard_out + standard_err)

    def test_timeout_prints_capture_path_and_exits_one(self):
        def fake_run(*args, **kwargs):
            raise subprocess.TimeoutExpired(cmd="cargo", timeout=5)

        with tempfile.TemporaryDirectory() as holder:
            capture = str(Path(holder) / "cap.txt")
            with mock.patch.object(distill.subprocess, "run", fake_run):
                captured = io.StringIO()
                with contextlib.redirect_stdout(captured):
                    return_code = distill.main(
                        ["--timeout", "5", "--capture", capture,
                         "cargo", "check"]
                    )
            self.assertEqual(1, return_code)
            self.assertIn("TIMEOUT after 5s -> capture: %s" % capture,
                          captured.getvalue())
            self.assertTrue(Path(capture).exists())


if __name__ == "__main__":
    unittest.main()
