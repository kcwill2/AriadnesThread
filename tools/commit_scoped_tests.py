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


class FailureDiagnosticTests(unittest.TestCase):
    """A failure must name a cause, even when git reports it on stdout.

    `git commit` exits 1 with an empty stderr for "nothing to commit", so a
    stderr-only message renders as a bare "failed:" with nothing after it. That
    was observed live during the five-role topology rollout, where the tool
    printed `ERROR git commit --only failed: ` and gave no clue why.
    """

    def test_describe_failure_prefers_stderr(self):
        message = commit_scoped.describe_failure(
            ("commit", "--only"), 1, "nothing to commit", "fatal: bad object",
        )
        self.assertIn("fatal: bad object", message)
        self.assertNotIn("nothing to commit", message)

    def test_describe_failure_falls_back_to_stdout(self):
        message = commit_scoped.describe_failure(
            ("commit", "--only"), 1, "nothing to commit, working tree clean", "",
        )
        self.assertIn("nothing to commit, working tree clean", message)

    def test_describe_failure_reports_exit_status_when_silent(self):
        message = commit_scoped.describe_failure(("commit", "--only"), 128, "", "")
        self.assertIn("exit 128", message)

    def test_already_committed_content_names_the_cause(self):
        """End-to-end: committing content already in HEAD must explain itself."""
        root = make_repo()
        seed_repo(root)
        write_file(root, "README.md", "# changed once\n")
        code, _, err = run_tool(
            root, "--scope", "README.md",
            "--lock-retry-delay", FAST_LOCK_DELAY, "-m", "first commit",
        )
        self.assertEqual(0, code, err)
        head_before = git(root, "rev-parse", "HEAD")

        code, _, err = run_tool(
            root, "--scope", "README.md",
            "--lock-retry-delay", FAST_LOCK_DELAY, "-m", "second commit",
        )
        self.assertEqual(2, code)
        self.assertIn("failed:", err)
        self.assertNotIn("failed: \n", err)
        self.assertNotEqual(
            "", err.strip().rsplit("failed:", 1)[-1].strip(),
            "failure message must carry a cause after 'failed:'",
        )
        self.assertEqual(head_before, git(root, "rev-parse", "HEAD"))


class FnmatchCrossDirectoryPinTests(unittest.TestCase):
    """Audit row 9, pinned AS-IS (advisory owner-decision class, no fix
    requested): scope_matches uses fnmatch.fnmatchcase, whose `*` DOES match
    the path separator -- so a written-as-one-level pattern like
    `crates/demo/*` really matches arbitrarily nested paths. Honest
    characterization: the name says which surprise is being locked, and the
    end-to-end test records exactly which paths commit from a two-directory
    fake repo."""

    def test_single_star_crosses_directory_separators(self):
        self.assertTrue(commit_scoped.scope_matches(
            "crates/demo/*", "crates/demo/src/lib.rs"))
        self.assertTrue(commit_scoped.scope_matches(
            "tools/*.py", "tools/nested/deep_test.py"))

    def test_literal_prefix_isolation_survives_the_star_crossing(self):
        # The separator-crossing star can never escape the literal directory
        # prefix in front of it -- that is what keeps single-star scopes
        # safe in practice despite the crossing.
        self.assertFalse(commit_scoped.scope_matches(
            "crates/demo/*", "crates/other/src/lib.rs"))
        self.assertFalse(commit_scoped.scope_matches(
            "crates/demo/*", "docs/demo/lib.rs"))

    def test_two_directory_repo_commits_only_the_scoped_directory(self):
        root = make_repo()
        write_file(root, "crates/alpha/src/deep.py", "ALPHA = 1\n")
        write_file(root, "crates/beta/src/deep.py", "BETA = 1\n")
        git(root, "add", "-A")
        git(root, "commit", "-q", "-m", "seed two dirs")
        write_file(root, "crates/alpha/src/deep.py", "ALPHA = 2\n")
        write_file(root, "crates/beta/src/deep.py", "BETA = 2\n")
        code, _, err = run_tool(
            root, "--scope", "crates/alpha/*",
            "--lock-retry-delay", FAST_LOCK_DELAY, "-m", "alpha only")
        self.assertEqual(0, code, err)
        committed = git(root, "show", "--name-only", "--format=", "HEAD")
        self.assertEqual(["crates/alpha/src/deep.py"], committed.split())


if __name__ == "__main__":
    unittest.main()
