"""Tests for tools/distilled.py (tool-output digest module, distill handoff T1).

Run directly: python tools\\distilled_tests.py
Or discovered: python -m unittest discover -s tools -p "*_tests.py"
Everything here is OFFLINE: canned tool outputs fed to pure digest functions.
ANSI fixtures are constructed via byte escapes, never literals (UTF-8 rule).
"""

import re
import tempfile
import unittest
from pathlib import Path

import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))

import distilled
import testrun

MULTI_SUITE_PASS = """\
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

TWO_FAILURES = """\
    Finished test [unoptimized + debuginfo] target(s) in 0.42s
     Running unittests src/lib.rs (target\\debug\\deps\\ariadne_x-1a2b.exe)

running 7 tests
test cycles::tests::detects_self_loop ... ok
test cycles::tests::ignores_dag ... FAILED
test transform::tests::drops_unused ... FAILED

failures:

---- cycles::tests::ignores_dag stdout ----
thread 'cycles::tests::ignores_dag' panicked at src/cycles.rs:88:5:
assertion failed: !has_cycle
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- transform::tests::drops_unused stdout ----
thread 'transform::tests::drops_unused' panicked at src/transform.rs:12:9:
assertion `left == right` failed
  left: 3
 right: 4

failures:
    cycles::tests::ignores_dag
    transform::tests::drops_unused

test result: FAILED. 5 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
"""

DRIFT_SHAPED = """\
     Running unittests src/lib.rs (target\\debug\\deps\\ariadne_x-0a1b.exe)
test result: ok. 3 passed, 0 failed, 0 ignored
"""

HEADERS_ONLY = """\
     Running unittests src/lib.rs (target\\debug\\deps\\ariadne_x-0a1b.exe)
running 0 tests
   Doc-tests ariadne_x
running 0 tests
"""

UNITTEST_MIX = """\
test_beta (tests.demo.DemoTest) ... ERROR

======================================================================
FAIL: test_alpha (tests.demo.DemoTest)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tests\\demo.py", line 12, in test_alpha
    self.assertEqual(1, 2)
AssertionError: 1 != 2

======================================================================
ERROR: test_beta (tests.demo.DemoTest)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "tests\\demo.py", line 16, in test_beta
    run()
FileNotFoundError: missing file
----------------------------------------------------------------------
Ran 2 tests in 0.001s

FAILED (failures=1, errors=1)
"""

THREE_TRACEBACKS = """\
step one
Traceback (most recent call last):
  File "one.py", line 1, in <module>
    main()
ValueError: first
Traceback (most recent call last):
  File "two.py", line 2, in go
    run()
TypeError: second
Traceback (most recent call last):
  File "three.py", line 3, in halt
    stop()
RuntimeError: third
done
"""

NOISE = "\n".join("noise line %d" % index for index in range(30)) + "\n"

HEADER_LINE = re.compile(r"^---- .+ ----$")
CAPTURE_LINE = re.compile(r"^capture: .+ \(\d+ lines\)$")


def ansi_variant(text):
    raw = text.encode("utf-8")
    raw = raw.replace(b"thread ", b"\x1b[31mthread ")
    raw = raw.replace(b"note:", b"\x1b[0mnote:")
    return raw.decode("utf-8")


class CargoTestDigestTests(unittest.TestCase):
    def test_clean_multi_suite_pass_digest_is_bounded_and_quiet(self):
        counts = testrun.parse_cargo_output(MULTI_SUITE_PASS)
        lines = distilled.digest_cargo_test(MULTI_SUITE_PASS, counts, 0)
        joined = "\n".join(lines)
        self.assertLessEqual(len(lines), 5)
        self.assertIn("0 failed", joined)
        self.assertNotIn("test graph::tests::empty_graph ... ok", joined)
        self.assertIn("no failures", joined)

    def test_two_failures_excerpt_names_and_panic_lines(self):
        counts = testrun.parse_cargo_output(TWO_FAILURES)
        lines = distilled.digest_cargo_test(TWO_FAILURES, counts, 1)
        joined = "\n".join(lines)
        self.assertIn("cycles::tests::ignores_dag", joined)
        self.assertIn("transform::tests::drops_unused", joined)
        self.assertIn("panicked at src/cycles.rs:88:5:", joined)
        self.assertIn("panicked at src/transform.rs:12:9:", joined)
        self.assertIn("assertion failed: !has_cycle", joined)
        self.assertNotIn("test cycles::tests::detects_self_loop ... ok", joined)

    def test_nine_failures_capped_with_more_marker(self):
        blocks = []
        names = []
        for index in range(9):
            name = "probe::tests::failure_%d" % index
            names.append(name)
            blocks.append(
                "---- %s stdout ----\n"
                "thread '%s' panicked at src/f.rs:%d:5:\n"
                "assertion failed: probe_%d\n\n"
                % (name, name, 10 + index, index)
            )
        text = (
            "running 9 tests\n"
            + "".join(blocks)
            + "failures:\n"
            + "\n".join("    " + name for name in names)
            + "\n\ntest result: FAILED. 0 passed; 9 failed; 0 ignored;"
            " 0 measured; 0 filtered out; finished in 0.10s\n"
        )
        counts = testrun.parse_cargo_output(text)
        lines = distilled.digest_cargo_test(text, counts, 1)
        header_count = sum(1 for line in lines if HEADER_LINE.match(line))
        self.assertEqual(5, header_count)
        self.assertEqual("... and 4 more failures", lines[-1])
        joined = "\n".join(lines)
        self.assertIn("probe::tests::failure_4", joined)
        self.assertNotIn("probe::tests::failure_5", joined)

    def test_ansi_variant_digests_identically(self):
        counts = testrun.parse_cargo_output(TWO_FAILURES)
        plain = distilled.digest_cargo_test(TWO_FAILURES, counts, 1)
        colored = distilled.digest_cargo_test(ansi_variant(TWO_FAILURES), counts, 1)
        self.assertEqual(plain, colored)

    def test_drift_and_zero_result_never_claim_pass(self):
        for text in (DRIFT_SHAPED, HEADERS_ONLY):
            counts = testrun.parse_cargo_output(text)
            lines = distilled.digest_cargo_test(text, counts, 2)
            joined = "\n".join(lines)
            self.assertIn("DRIFT", joined)
            self.assertIn("parse format drift", joined)
            self.assertNotIn("PASS", joined)


class GenericDigestTests(unittest.TestCase):
    def test_generic_excerpts_unittest_failure_names(self):
        digest = distilled.digest_generic(UNITTEST_MIX, 1)
        joined = "\n".join(digest.lines)
        self.assertEqual("FAIL", digest.verdict)
        self.assertIn("FAIL: test_alpha (tests.demo.DemoTest)", joined)
        self.assertIn("ERROR: test_beta (tests.demo.DemoTest)", joined)

    def test_generic_caps_traceback_blocks_at_two(self):
        digest = distilled.digest_generic(THREE_TRACEBACKS, 1)
        joined = "\n".join(digest.lines)
        traceback_headers = [
            line for line in digest.lines
            if line.startswith("Traceback (most recent call last):")
        ]
        self.assertEqual(2, len(traceback_headers))
        self.assertIn("one.py", joined)
        self.assertIn("two.py", joined)
        self.assertNotIn("three.py", joined)

    def test_generic_head_tail_fallback_verdict_from_exit(self):
        digest = distilled.digest_generic(NOISE, 3)
        joined = "\n".join(digest.lines)
        self.assertEqual("FAIL", digest.verdict)
        self.assertIn("noise line 0", joined)
        self.assertIn("noise line 2", joined)
        self.assertIn("noise line 29", joined)
        self.assertIn("noise line 18", joined)
        self.assertNotIn("noise line 10", joined)

    def test_generic_budget_and_capture_last_line(self):
        shapes = [
            (UNITTEST_MIX, 1),
            (THREE_TRACEBACKS, 1),
            (NOISE, 3),
            ("", 0),
            ("\n".join("row %d" % row for row in range(400)), 2),
        ]
        with tempfile.TemporaryDirectory() as holder:
            capture = str(Path(holder) / "sub" / "cap.txt")
            for text, exit_code in shapes:
                digest = distilled.digest_generic(
                    text, exit_code, capture_path=capture
                )
                self.assertLessEqual(len(digest.lines), 50)
                self.assertRegex(digest.lines[-1], CAPTURE_LINE)


class WriteCaptureTests(unittest.TestCase):
    def test_write_capture_byte_exact_and_creates_parents(self):
        with tempfile.TemporaryDirectory() as holder:
            target = Path(holder) / "sub" / "dir" / "cap.txt"
            text = "one\r\ntwo\r\nthree\n"
            returned = distilled.write_capture(text, target)
            self.assertEqual(target, Path(returned))
            self.assertEqual(text.encode("utf-8"), target.read_bytes())


if __name__ == "__main__":
    unittest.main()
