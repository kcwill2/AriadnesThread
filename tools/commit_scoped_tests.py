"""Unit tests for tools/commit_scoped.py (scope-resolved, --only commits).

Run directly: python tools\\commit_scoped_tests.py
All fixtures are throwaway git repos in temp directories; nothing touches the
real repository, and no network or cargo is involved.
"""

import subprocess
import sys
import tempfile
import threading
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import commit_scoped

TOOL_PATH = str(Path(__file__).resolve().parent / "commit_scoped.py")

FAST_LOCK_DELAY = "0.05"


def git(repo, *args):
    finished = subprocess.run(
        ["git", *args],
        cwd=str(repo),
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    if finished.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {finished.stderr}")
    return finished.stdout


def make_repo():
    root = Path(tempfile.mkdtemp(prefix="scoped-"))
    git(root, "init", "-q")
    git(root, "config", "user.email", "scoped@example.invalid")
    git(root, "config", "user.name", "Scoped Tests")
    return root


def write_file(root, relative, text):
    target = Path(root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8")


def seed_repo(root):
    write_file(root, "crates/demo/src/lib.rs", "pub fn demo() {}\n")
    write_file(root, "crates/other/src/lib.rs", "pub fn other() {}\n")
    write_file(root, "README.md", "# seed\n")
    git(root, "add", "-A")
    git(root, "commit", "-q", "-m", "seed")


def run_tool(root, *args):
    finished = subprocess.run(
        [sys.executable, TOOL_PATH, *args],
        cwd=str(root),
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    return finished.returncode, finished.stdout, finished.stderr


class ScopeResolutionTests(unittest.TestCase):
    def test_commit_contains_only_scope_paths(self):
        root = make_repo()
        seed_repo(root)
        write_file(root, "crates/demo/src/lib.rs", "pub fn demo() { 1 }\n")
        write_file(root, "crates/demo/src/new.rs", "pub fn added() {}\n")
        write_file(root, "crates/other/src/lib.rs", "pub fn other() { 2 }\n")
        code, out, err = run_tool(
            root, "--scope", "crates/demo/**",
            "--lock-retry-delay", FAST_LOCK_DELAY,
            "-m", "demo only",
        )
        committed = git(root, "show", "--name-only", "--format=", "HEAD")
        self.assertEqual(0, code, err)
        self.assertEqual(
            sorted(["crates/demo/src/lib.rs", "crates/demo/src/new.rs"]),
            sorted(committed.split()),
        )
        self.assertIn("crates/demo/src/lib.rs", out)

    def test_foreign_staged_content_is_never_committed(self):
        root = make_repo()
        seed_repo(root)
        write_file(root, "crates/other/src/lib.rs", "pub fn other() { 9 }\n")
        git(root, "add", "crates/other/src/lib.rs")
        write_file(root, "crates/demo/src/lib.rs", "pub fn demo() { 3 }\n")
        code, _, err = run_tool(
            root, "--scope", "crates/demo/**",
            "--lock-retry-delay", FAST_LOCK_DELAY,
            "-m", "demo despite foreign stage",
        )
        committed = git(root, "show", "--name-only", "--format=", "HEAD")
        status = git(root, "status", "--porcelain")
        self.assertEqual(0, code, err)
        self.assertNotIn("crates/other", committed)
        self.assertIn("crates/other/src/lib.rs", status)

    def test_root_like_scopes_are_rejected(self):
        root = make_repo()
        seed_repo(root)
        write_file(root, "README.md", "# changed\n")
        for scope in (".", "./", "*", "/"):
            code, _, _ = run_tool(
                root, "--scope", scope,
                "--lock-retry-delay", FAST_LOCK_DELAY, "-m", "nope",
            )
            self.assertEqual(1, code, f"scope {scope} must be rejected")
        self.assertEqual("seed", git(root, "log", "--oneline").split()[1])

    def test_empty_scope_aborts_without_committing(self):
        root = make_repo()
        seed_repo(root)
        write_file(root, "crates/demo/src/lib.rs", "pub fn demo() { 4 }\n")
        log_before = git(root, "log", "--oneline")
        code, _, err = run_tool(
            root, "--scope", "crates/missing/**",
            "--lock-retry-delay", FAST_LOCK_DELAY, "-m", "nothing here",
        )
        self.assertEqual(1, code)
        self.assertIn("resolved to nothing", err)
        self.assertEqual(log_before, git(root, "log", "--oneline"))

    def test_multiple_scopes_are_merged(self):
        root = make_repo()
        seed_repo(root)
        write_file(root, "crates/demo/src/lib.rs", "pub fn demo() { 5 }\n")
        write_file(root, "README.md", "# merged\n")
        code, _, err = run_tool(
            root,
            "--scope", "crates/demo/**", "--scope", "README.md",
            "--lock-retry-delay", FAST_LOCK_DELAY, "-m", "two scopes",
        )
        committed = git(root, "show", "--name-only", "--format=", "HEAD")
        self.assertEqual(0, code, err)
        self.assertEqual(
            sorted(["README.md", "crates/demo/src/lib.rs"]),
            sorted(committed.split()),
        )


class LockRetryTests(unittest.TestCase):
    def test_transient_index_lock_is_retried_then_succeeds(self):
        root = make_repo()
        seed_repo(root)
        write_file(root, "crates/demo/src/lib.rs", "pub fn demo() { 6 }\n")
        lock = Path(root) / ".git" / "index.lock"
        lock.mkdir()
        threading.Timer(0.25, lambda: lock.rmdir()).start()
        code, _, err = run_tool(
            root, "--scope", "crates/demo/**",
            "--lock-retry-delay", "0.15", "-m", "after lock clears",
        )
        self.assertEqual(0, code, err)
        self.assertIn("after lock clears", git(root, "log", "--oneline"))

    def test_persistent_index_lock_gives_up_with_internal_error(self):
        root = make_repo()
        seed_repo(root)
        write_file(root, "crates/demo/src/lib.rs", "pub fn demo() { 7 }\n")
        lock = Path(root) / ".git" / "index.lock"
        lock.mkdir()
        try:
            code, _, _ = run_tool(
                root, "--scope", "crates/demo/**",
                "--lock-retry-delay", FAST_LOCK_DELAY, "-m", "never lands",
            )
        finally:
            lock.rmdir()
        self.assertEqual(2, code)


class UsageTests(unittest.TestCase):
    def test_missing_message_is_usage_error(self):
        root = make_repo()
        seed_repo(root)
        code, _, _ = run_tool(
            root, "--scope", "crates/demo/**",
            "--lock-retry-delay", FAST_LOCK_DELAY,
        )
        self.assertEqual(2, code)

    def test_missing_scope_is_usage_error(self):
        root = make_repo()
        seed_repo(root)
        code, _, _ = run_tool(
            root, "--lock-retry-delay", FAST_LOCK_DELAY, "-m", "no scope",
        )
        self.assertEqual(2, code)


if __name__ == "__main__":
    unittest.main()
