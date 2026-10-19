"""Unit tests for tools/encoding_guard.py (mojibake + non-UTF-8 detector).

Run directly: python tools\\encoding_guard_tests.py
Every fixture lives in a throwaway git repo in a temp directory; nothing here
touches the real repository. Mojibake fixtures are built from BYTE escapes so
this test file stays pure ASCII and never trips the very guard it exercises.
"""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import encoding_guard

GUARD_PATH = str(Path(__file__).resolve().parent / "encoding_guard.py")

# Double-encoded UTF-8 of the section sign: real bytes 0xC2 0xA7 re-decoded as
# latin-1 then re-encoded, i.e. "A-circumflex A-dieresis". Built from escapes.
MOJIBAKE_SECTION = b"\xc3\x82\xc2\xa7"
# The section sign written CORRECTLY as UTF-8.
CLEAN_SECTION = b"\xc2\xa7"
# A raw lone continuation byte that is not valid UTF-8 at all.
NOT_UTF8 = b"ok \xff\xfe end\n"


def git(repo, *args):
    finished = subprocess.run(
        ["git", *args],
        cwd=str(repo),
        capture_output=True,
        text=True,
    )
    if finished.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {finished.stderr}")
    return finished.stdout


def write_bytes(root, relative, payload):
    target = Path(root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(payload)


def run_guard(root, *args):
    finished = subprocess.run(
        [sys.executable, GUARD_PATH, *args],
        cwd=str(root),
        capture_output=True,
        text=True,
    )
    return finished.returncode, finished.stdout + finished.stderr


class MojibakeDetectionTests(unittest.TestCase):
    def test_clean_utf8_repo_is_not_flagged(self):
        with tempfile.TemporaryDirectory() as holder:
            root = Path(holder) / "clean"
            root.mkdir()
            make_repo_at(root)
            write_bytes(root, "clean.rs", b"// \xc2\xa7 section sign ok\n")
            code, out = run_guard(root, "--all")
        self.assertEqual(0, code, out)

    def test_mojibake_in_tracked_file_exits_one(self):
        with tempfile.TemporaryDirectory() as holder:
            root = Path(holder) / "dirty"
            root.mkdir()
            make_repo_at(root)
            write_bytes(root, "broken.rs", MOJIBAKE_SECTION)
            git(root, "add", "broken.rs")
            code, out = run_guard(root, "--all")
        self.assertEqual(1, code)
        self.assertIn("broken.rs", out)
        self.assertIn("mojibake", out)

    def test_non_utf8_bytes_exits_one(self):
        with tempfile.TemporaryDirectory() as holder:
            root = Path(holder) / "binary"
            root.mkdir()
            make_repo_at(root)
            write_bytes(root, "raw.txt", NOT_UTF8)
            git(root, "add", "raw.txt")
            code, out = run_guard(root, "--all")
        self.assertEqual(1, code)
        self.assertIn("raw.txt", out)
        self.assertIn("not utf-8", out)

    def test_allow_listed_plan_path_is_exempt(self):
        with tempfile.TemporaryDirectory() as holder:
            root = Path(holder) / "plans"
            root.mkdir()
            make_repo_at(root)
            write_bytes(root, ".kilo/plans/spec.md", MOJIBAKE_SECTION)
            write_bytes(root, "plans/other.md", MOJIBAKE_SECTION)
            code, out = run_guard(root, "--all")
        self.assertEqual(0, code, out)

    def test_staged_scope_ignores_unstaged_dirt(self):
        with tempfile.TemporaryDirectory() as holder:
            root = Path(holder) / "staged"
            root.mkdir()
            make_repo_at(root)
            write_bytes(root, "seed.rs", b"seed\n")
            git(root, "add", "seed.rs")
            git(root, "commit", "-q", "-m", "seed")
            write_bytes(root, "staged_bad.rs", MOJIBAKE_SECTION)
            write_bytes(root, "unstaged_bad.rs", MOJIBAKE_SECTION)
            git(root, "add", "staged_bad.rs")
            code, out = run_guard(root, "--staged")
        self.assertEqual(1, code)
        self.assertIn("staged_bad.rs", out)
        self.assertNotIn("unstaged_bad.rs", out)

    def test_default_scans_tracked_working_tree(self):
        with tempfile.TemporaryDirectory() as holder:
            root = Path(holder) / "tracked"
            root.mkdir()
            make_repo_at(root)
            write_bytes(root, "kept.rs", MOJIBAKE_SECTION)
            write_bytes(root, "untracked.rs", MOJIBAKE_SECTION)
            git(root, "add", "kept.rs")
            code, out = run_guard(root)
        self.assertEqual(1, code)
        self.assertIn("kept.rs", out)
        self.assertNotIn("untracked.rs", out)

    def test_vendored_and_binary_paths_are_skipped(self):
        with tempfile.TemporaryDirectory() as holder:
            root = Path(holder) / "vendored"
            root.mkdir()
            make_repo_at(root)
            write_bytes(root, "node_modules/pkg/bad.rs", MOJIBAKE_SECTION)
            write_bytes(root, "assets/logo.png", MOJIBAKE_SECTION + b"\x00\x01")
            code, out = run_guard(root, "--all")
        self.assertEqual(0, code, out)


class HelperTests(unittest.TestCase):
    def test_scan_blob_reports_mojibake(self):
        hits = encoding_guard.scan_blob_bytes(MOJIBAKE_SECTION)
        self.assertTrue(hits)

    def test_scan_blob_accepts_correct_utf8(self):
        self.assertEqual([], encoding_guard.scan_blob_bytes(CLEAN_SECTION))

    def test_signatures_are_built_from_bytes_not_literals(self):
        for signature in encoding_guard.MOJIBAKE_SIGNATURES:
            self.assertIsInstance(signature, bytes)


def make_repo_at(root):
    git(root, "init", "-q")
    git(root, "config", "user.email", "guard@example.invalid")
    git(root, "config", "user.name", "Guard Tests")


class UsageAndNonRepoTests(unittest.TestCase):
    """Audit row 8: the --root usage failures and the outside-a-git-repo
    error class."""

    def test_root_flag_without_value_exits_two(self):
        with tempfile.TemporaryDirectory() as holder:
            code, out = run_guard(Path(holder), "--all", "--root")
        self.assertEqual(2, code)
        self.assertIn("--root requires a path", out)

    def test_root_pointing_at_missing_path_is_usage_error(self):
        with tempfile.TemporaryDirectory() as holder:
            missing = str(Path(holder) / "no-such-dir")
            code, out = run_guard(Path(holder), "--all", "--root", missing)
        self.assertEqual(2, code)
        self.assertIn("usage:", out)

    def test_root_pointing_at_a_plain_file_is_usage_error(self):
        with tempfile.TemporaryDirectory() as holder:
            not_a_dir = Path(holder) / "file.txt"
            not_a_dir.write_text("x\n", encoding="utf-8")
            code, out = run_guard(Path(holder), "--all", "--root",
                                  str(not_a_dir))
        self.assertEqual(2, code)
        self.assertIn("usage:", out)

    def test_run_outside_git_repo_exits_two_naming_git_plumbing(self):
        with tempfile.TemporaryDirectory() as holder:
            code, out = run_guard(Path(holder), "--all")
        self.assertEqual(2, code)
        self.assertIn("git plumbing failed", out)


if __name__ == "__main__":
    unittest.main()
